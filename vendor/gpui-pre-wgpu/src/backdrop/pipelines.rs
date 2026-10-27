pub(super) struct Pipelines {
    pub layout: wgpu::BindGroupLayout,
    pub present_layout: wgpu::BindGroupLayout,
    pub downsample: wgpu::RenderPipeline,
    pub horizontal: wgpu::RenderPipeline,
    pub vertical: wgpu::RenderPipeline,
    pub composite: wgpu::RenderPipeline,
    pub present: wgpu::RenderPipeline,
}

pub(super) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Pipelines {
    let texture_entry = |binding| wgpu::BindGroupLayoutEntry {
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
        label: Some("backdrop_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                        super::geometry::Params,
                    >() as u64),
                },
                count: None,
            },
            texture_entry(1),
            texture_entry(2),
        ],
    });
    let present_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("backdrop_present_layout"),
        entries: &[texture_entry(0)],
    });
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("backdrop_shaders"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
    });
    let present_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("backdrop_present_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("present.wgsl").into()),
    });
    let pipeline = |label, vertex, fragment, target, bind_layout, shader: &wgpu::ShaderModule| {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[Some(bind_layout)],
            immediate_size: 0,
        });
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some(vertex),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some(fragment),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    // The composite shader mixes with a snapshot instead of
                    // source-over blending the backdrop a second time.
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        })
    };
    Pipelines {
        downsample: pipeline(
            "backdrop_downsample",
            "vs_fullscreen",
            "fs_downsample",
            super::WORK_FORMAT,
            &layout,
            &module,
        ),
        horizontal: pipeline(
            "backdrop_horizontal",
            "vs_fullscreen",
            "fs_horizontal",
            super::WORK_FORMAT,
            &layout,
            &module,
        ),
        vertical: pipeline(
            "backdrop_vertical",
            "vs_fullscreen",
            "fs_vertical",
            super::WORK_FORMAT,
            &layout,
            &module,
        ),
        composite: pipeline(
            "backdrop_composite",
            "vs_fullscreen",
            "fs_composite",
            format,
            &layout,
            &module,
        ),
        present: pipeline(
            "backdrop_present",
            "vs_fullscreen",
            "fs_present",
            format,
            &present_layout,
            &present_module,
        ),
        layout,
        present_layout,
    }
}
