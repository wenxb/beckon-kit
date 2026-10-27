struct Params { bounds: vec4<f32>, radii: vec4<f32> }
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var original: texture_2d<f32>;
@group(0) @binding(2) var foreground: texture_2d<f32>;

@vertex
fn vs_fullscreen(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}
@fragment
fn fs_clip(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let p = position.xy - params.bounds.xy;
    let half_size = params.bounds.zw * 0.5;
    let local = p - half_size;
    var radius = params.radii.x;
    if local.x > 0.0 { radius = params.radii.y; }
    if local.y > 0.0 { radius = params.radii.z; if local.x > 0.0 { radius = params.radii.w; } }
    let q = abs(local) - half_size + vec2<f32>(radius);
    let distance = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - radius;
    let coverage = clamp(0.5 - distance, 0.0, 1.0);
    let pixel = vec2<i32>(position.xy);
    return mix(textureLoad(original, pixel, 0), textureLoad(foreground, pixel, 0), coverage);
}
