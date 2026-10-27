//! A scene-local, GPU-only blur path. Nothing is captured from the desktop.

mod clip;
mod geometry;
mod pipelines;

use geometry::{Geometry, Params};
use gpui::{BackdropBlur, DevicePixels, Size};
use wgpu::util::DeviceExt;

const WORK_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

struct Target {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Target {
    fn new(
        device: &wgpu::Device,
        label: &str,
        extent: [u32; 2],
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture, view }
    }

    fn fits(&self, extent: [u32; 2]) -> bool {
        self.texture.width() >= extent[0] && self.texture.height() >= extent[1]
    }
}

struct Scratch {
    snapshot: Target,
    first: Target,
    second: Target,
}

pub(crate) struct BackdropRenderer {
    pipelines: Option<pipelines::Pipelines>,
    format: wgpu::TextureFormat,
    scene: Option<Target>,
    scratch: Option<Scratch>,
    clip: Option<clip::ClipRenderer>,
}

impl BackdropRenderer {
    pub fn new(context: &crate::WgpuContext, format: wgpu::TextureFormat) -> Option<Self> {
        let required = wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST;
        if [format, WORK_FORMAT].into_iter().any(|format| {
            !context
                .adapter
                .get_texture_format_features(format)
                .allowed_usages
                .contains(required)
        }) {
            return None;
        }
        Some(Self {
            pipelines: None,
            format,
            scene: None,
            scratch: None,
            clip: None,
        })
    }

    pub fn invalidate_targets(&mut self) {
        self.scene = None;
        self.scratch = None;
        self.clip = None;
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        size: Size<DevicePixels>,
        blurs: &[BackdropBlur],
        clips: &[gpui::RoundedClipBoundary],
    ) {
        // Windows with no glass keep the original renderer's startup cost.
        self.pipelines
            .get_or_insert_with(|| pipelines::new(device, self.format));
        let extent = [size.width.0 as u32, size.height.0 as u32];
        let same_size = self.scene.as_ref().is_some_and(|target| {
            target.texture.width() == extent[0] && target.texture.height() == extent[1]
        });
        if !same_size {
            self.scene = Some(Target::new(device, "backdrop_scene", extent, self.format));
            self.scratch = None;
        }
        if !clips.is_empty() {
            self.clip
                .get_or_insert_with(|| clip::ClipRenderer::new(device, self.format))
                .prepare(device, self.format, extent, clips);
        } else {
            self.clip = None;
        }
        let mut capture = [1, 1];
        let mut samples = [1, 1];
        for geometry in blurs.iter().filter_map(|blur| Geometry::new(blur, size)) {
            for axis in 0..2 {
                capture[axis] = capture[axis].max(geometry.extent[axis]);
                samples[axis] = samples[axis].max(geometry.samples[axis]);
            }
        }
        let fits = self
            .scratch
            .as_ref()
            .is_some_and(|scratch| scratch.snapshot.fits(capture) && scratch.first.fits(samples));
        if !fits {
            let maximum = device.limits().max_texture_dimension_2d;
            let grow = |extent: [u32; 2], previous: Option<&Target>| {
                let old = previous.map_or([0, 0], |target| {
                    [target.texture.width(), target.texture.height()]
                });
                std::array::from_fn(|axis| {
                    extent[axis]
                        .max(old[axis])
                        .next_multiple_of(64)
                        .min(maximum)
                })
            };
            let capture = grow(
                capture,
                self.scratch.as_ref().map(|scratch| &scratch.snapshot),
            );
            let samples = grow(samples, self.scratch.as_ref().map(|scratch| &scratch.first));
            self.scratch = Some(Scratch {
                snapshot: Target::new(device, "backdrop_snapshot", capture, self.format),
                first: Target::new(device, "backdrop_filter_a", samples, WORK_FORMAT),
                second: Target::new(device, "backdrop_filter_b", samples, WORK_FORMAT),
            });
        }
    }

    pub fn scene_view(&self) -> wgpu::TextureView {
        self.scene
            .as_ref()
            .expect("prepared backdrop scene")
            .view
            .clone()
    }

    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        blur: &BackdropBlur,
        size: Size<DevicePixels>,
    ) {
        let Some(mut geometry) = Geometry::new(blur, size) else {
            return;
        };
        geometry.params.effect[2] = f32::from(self.format.is_srgb());
        let pipelines = self
            .pipelines
            .as_ref()
            .expect("prepared backdrop pipelines");
        let scratch = self.scratch.as_ref().expect("prepared backdrop scratch");
        let scene = self.scene.as_ref().expect("prepared backdrop scene");
        encoder.copy_texture_to_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &scene.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: geometry.origin[0],
                    y: geometry.origin[1],
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            scratch.snapshot.texture.as_image_copy(),
            wgpu::Extent3d {
                width: geometry.extent[0],
                height: geometry.extent[1],
                depth_or_array_layers: 1,
            },
        );
        // Each encoded region owns immutable uniforms. Queue-writing one shared
        // uniform repeatedly would make every region use the final region's data.
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("backdrop_region_params"),
            contents: bytemuck::bytes_of(&geometry.params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind = |source: &wgpu::TextureView| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("backdrop_region_bindings"),
                layout: &pipelines.layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(source),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&scratch.snapshot.view),
                    },
                ],
            })
        };
        for (pipeline, source, destination) in [
            (
                &pipelines.downsample,
                &scratch.snapshot.view,
                &scratch.first.view,
            ),
            (
                &pipelines.horizontal,
                &scratch.first.view,
                &scratch.second.view,
            ),
            (
                &pipelines.vertical,
                &scratch.second.view,
                &scratch.first.view,
            ),
        ] {
            let bindings = bind(source);
            let mut pass = render_pass(encoder, destination, "backdrop_filter", true);
            pass.set_viewport(
                0.0,
                0.0,
                geometry.samples[0] as f32,
                geometry.samples[1] as f32,
                0.0,
                1.0,
            );
            pass.set_scissor_rect(0, 0, geometry.samples[0], geometry.samples[1]);
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &bindings, &[]);
            pass.draw(0..3, 0..1);
        }
        let bindings = bind(&scratch.first.view);
        let mut pass = render_pass(encoder, &scene.view, "backdrop_composite", false);
        let [x, y, width, height] = geometry.scissor;
        pass.set_scissor_rect(x, y, width, height);
        pass.set_pipeline(&pipelines.composite);
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..3, 0..1);
    }

    pub fn encode_clip(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        boundary: &gpui::RoundedClipBoundary,
        depth: usize,
    ) {
        self.clip.as_ref().expect("prepared rounded clips").encode(
            device,
            encoder,
            self.scene.as_ref().unwrap(),
            boundary,
            depth,
        );
    }

    pub fn present(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        frame: &wgpu::TextureView,
    ) {
        let scene = self.scene.as_ref().expect("prepared backdrop scene");
        let pipelines = self
            .pipelines
            .as_ref()
            .expect("prepared backdrop pipelines");
        let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("backdrop_present_bindings"),
            layout: &pipelines.present_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&scene.view),
            }],
        });
        let mut pass = render_pass(encoder, frame, "backdrop_present", true);
        pass.set_pipeline(&pipelines.present);
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn render_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    label: &str,
    clear: bool,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: if clear {
                    wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                } else {
                    wgpu::LoadOp::Load
                },
                store: wgpu::StoreOp::Store,
            },
            depth_slice: None,
        })],
        depth_stencil_attachment: None,
        ..Default::default()
    })
}

// Keep the CPU and shader record layouts aligned without unsafe casts.
const _: () = assert!(std::mem::size_of::<Params>() == 96);
