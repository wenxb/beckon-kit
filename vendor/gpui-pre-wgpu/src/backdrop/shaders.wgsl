struct Params {
    viewport: vec2<f32>,
    origin: vec2<f32>,
    extent: vec2<f32>,
    samples: vec2<f32>,
    bounds: vec4<f32>,
    clip: vec4<f32>,
    corners: vec4<f32>,
    effect: vec4<f32>, // sigma, opacity, sRGB target, unused
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var source: texture_2d<f32>;
@group(0) @binding(2) var snapshot: texture_2d<f32>;

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

fn to_linear(value: vec3<f32>) -> vec3<f32> {
    return select(pow((value + 0.055) / 1.055, vec3<f32>(2.4)), value / 12.92, value <= vec3<f32>(0.04045));
}

fn to_srgb(value: vec3<f32>) -> vec3<f32> {
    let value_safe = max(value, vec3<f32>(0.0));
    return select(1.055 * pow(value_safe, vec3<f32>(1.0 / 2.4)) - 0.055, value_safe * 12.92, value_safe <= vec3<f32>(0.0031308));
}

fn linear_pixel(coordinate: vec2<i32>) -> vec4<f32> {
    let pixel = textureLoad(source, clamp(coordinate, vec2<i32>(0), vec2<i32>(params.extent) - 1), 0);
    if (params.effect.z != 0.0) {
        return pixel;
    }
    // Accumulated render-target colors are premultiplied. Decode straight RGB
    // before filtering, avoiding dark edges around translucent content.
    let straight = pixel.rgb / max(pixel.a, 0.00001);
    return vec4<f32>(to_linear(clamp(straight, vec3<f32>(0.0), vec3<f32>(1.0))) * pixel.a, pixel.a);
}

@fragment
fn fs_downsample(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let step = params.extent / params.samples;
    let start = floor(position.xy) * step;
    let end = start + step;
    let lower = vec2<i32>(floor(start));
    let upper = vec2<i32>(ceil(end));
    var result = vec4<f32>(0.0);
    // Average every covered source pixel. A fixed sparse tap pattern aliases
    // stripes and text when a large sigma or fractional DPI changes this step.
    for (var y = lower.y; y < upper.y; y += 1) {
        for (var x = lower.x; x < upper.x; x += 1) {
            let pixel = vec2<f32>(f32(x), f32(y));
            let weight = min(end, pixel + 1.0) - max(start, pixel);
            result += linear_pixel(vec2<i32>(x, y)) * weight.x * weight.y;
        }
    }
    return result / (step.x * step.y);
}

fn work_pixel(coordinate: vec2<i32>) -> vec4<f32> {
    return textureLoad(source, clamp(coordinate, vec2<i32>(0), vec2<i32>(params.samples) - 1), 0);
}

fn gaussian(position: vec2<f32>, axis: vec2<i32>, step: f32) -> vec4<f32> {
    let sigma = max(params.effect.x / step, 0.001);
    let radius = i32(min(ceil(sigma * 3.0), 24.0));
    var total = 0.0;
    var result = vec4<f32>(0.0);
    for (var index = -radius; index <= radius; index += 1) {
        let distance = f32(index) / sigma;
        let weight = exp(-0.5 * distance * distance);
        result += work_pixel(vec2<i32>(position) + axis * index) * weight;
        total += weight;
    }
    return result / total;
}

@fragment
fn fs_horizontal(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return gaussian(position.xy, vec2<i32>(1, 0), params.extent.x / params.samples.x);
}

@fragment
fn fs_vertical(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return gaussian(position.xy, vec2<i32>(0, 1), params.extent.y / params.samples.y);
}

fn filtered(position: vec2<f32>) -> vec4<f32> {
    let coordinate = (position - params.origin) * params.samples / params.extent - 0.5;
    let base = vec2<i32>(floor(coordinate));
    let fraction = fract(coordinate);
    let upper = mix(work_pixel(base), work_pixel(base + vec2<i32>(1, 0)), fraction.x);
    let lower = mix(work_pixel(base + vec2<i32>(0, 1)), work_pixel(base + vec2<i32>(1, 1)), fraction.x);
    return mix(upper, lower, fraction.y);
}

fn coverage(position: vec2<f32>) -> f32 {
    let center = params.bounds.xy + params.bounds.zw * 0.5;
    let right = position.x > center.x;
    let bottom = position.y > center.y;
    let top_radius = select(params.corners.x, params.corners.y, right);
    let bottom_radius = select(params.corners.w, params.corners.z, right);
    let radius = select(top_radius, bottom_radius, bottom);
    let inset = abs(position - center) - params.bounds.zw * 0.5 + radius;
    let distance = length(max(inset, vec2<f32>(0.0))) + min(max(inset.x, inset.y), 0.0) - radius;
    return clamp(0.5 - distance, 0.0, 1.0);
}

@fragment
fn fs_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    if (any(position.xy < params.clip.xy) || any(position.xy >= params.clip.zw)) {
        discard;
    }
    let amount = coverage(position.xy) * params.effect.y;
    if (amount == 0.0) {
        discard;
    }
    let original = textureLoad(snapshot, vec2<i32>(position.xy - params.origin), 0);
    var blurred = filtered(position.xy);
    if (params.effect.z == 0.0) {
        let straight = blurred.rgb / max(blurred.a, 0.00001);
        blurred = vec4<f32>(to_srgb(straight) * blurred.a, blurred.a);
    }
    return mix(original, blurred, amount);
}
