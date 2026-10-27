use super::{Target, render_pass};
use gpui::RoundedClipBoundary;
use wgpu::util::DeviceExt;

pub(super) struct ClipRenderer {
    layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    saved: Vec<Target>,
    foreground: Option<Target>,
}
impl ClipRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rounded_clip_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(32),
                    },
                    count: None,
                },
                texture(1),
                texture(2),
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rounded_clip_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("clip.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rounded_clip_pipeline_layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rounded_clip_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_fullscreen"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_clip"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            layout,
            pipeline,
            saved: Vec::new(),
            foreground: None,
        }
    }
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        extent: [u32; 2],
        boundaries: &[RoundedClipBoundary],
    ) {
        let mut depth: usize = 0;
        let mut maximum = 0;
        for b in boundaries {
            if b.begin {
                depth += 1;
                maximum = maximum.max(depth);
            } else {
                depth = depth.checked_sub(1).expect("balanced rounded clips");
            }
        }
        assert_eq!(depth, 0);
        assert!(maximum <= 8, "maximum rounded clip depth is eight");
        // At most nine full-size textures; destroyed with this window/device.
        self.saved.truncate(maximum);
        if self.foreground.as_ref().is_some_and(|t| {
            maximum == 0 || t.texture.width() != extent[0] || t.texture.height() != extent[1]
        }) {
            self.saved.clear();
            self.foreground = None;
        }
        while self.saved.len() < maximum {
            self.saved
                .push(Target::new(device, "rounded_clip_saved", extent, format));
        }
        if maximum > 0 && self.foreground.is_none() {
            self.foreground = Some(Target::new(
                device,
                "rounded_clip_foreground",
                extent,
                format,
            ));
        }
    }
    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        scene: &Target,
        boundary: &RoundedClipBoundary,
        depth: usize,
    ) {
        let extent = scene.texture.size();
        if boundary.begin {
            encoder.copy_texture_to_texture(
                scene.texture.as_image_copy(),
                self.saved[depth].texture.as_image_copy(),
                extent,
            );
            return;
        }
        let foreground = self.foreground.as_ref().unwrap();
        encoder.copy_texture_to_texture(
            scene.texture.as_image_copy(),
            foreground.texture.as_image_copy(),
            extent,
        );
        let b = boundary.bounds;
        let r = boundary.radii;
        let params: [f32; 8] = [
            b.origin.x.0,
            b.origin.y.0,
            b.size.width.0,
            b.size.height.0,
            r.top_left.0,
            r.top_right.0,
            r.bottom_left.0,
            r.bottom_right.0,
        ];
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rounded_clip_params"),
            contents: bytemuck::cast_slice(&params),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rounded_clip_bindings"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.saved[depth].view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&foreground.view),
                },
            ],
        });
        let mut pass = render_pass(encoder, &scene.view, "rounded_clip_composite", false);
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..3, 0..1);
    }
}
