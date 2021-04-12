use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;
use winit::dpi::{LogicalSize, PhysicalSize};
use wgpu::PowerPreference;
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::event::{Event, WindowEvent};

fn main() {
	let mut event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_inner_size(LogicalSize::new(640, 480))
		.build(&event_loop)
		.unwrap();

	let (width, height) = window.inner_size().into();

	let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

	let surface = unsafe { instance.create_surface(&window) };

	let adapter = pollster::block_on(
		instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: PowerPreference::Default,
			compatible_surface: Some(&surface)
		})
	).unwrap();

	let (device, queue) = pollster::block_on(
		adapter.request_device(&Default::default(), None)
	).unwrap();

	let mut sc_desc = wgpu::SwapChainDescriptor {
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
		format: wgpu::TextureFormat::Bgra8UnormSrgb,
		width,
		height,
		present_mode: wgpu::PresentMode::Immediate
	};

	let mut swapchain = device.create_swap_chain(&surface, &sc_desc);

	let enc_desc = wgpu::CommandEncoderDescriptor { label: None };

	let bind_group_layout =
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStage::VERTEX,
					ty: wgpu::BindingType::UniformBuffer {
						dynamic: false,
						min_binding_size: Some(std::num::NonZeroU64::new(16).unwrap())
					},
					count: None
				}
			],
			label: None
		});

	let pipeline =
		device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&device.create_pipeline_layout(
				&wgpu::PipelineLayoutDescriptor {
					label: None,
					bind_group_layouts: &[&bind_group_layout],
					push_constant_ranges: &[]
				}
			)),
			vertex_stage: wgpu::ProgrammableStageDescriptor {
				module: &device.create_shader_module(wgpu::include_spirv!("shaders/shader.vert.spv")),
				entry_point: "main"
			},
			fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
				module: &device.create_shader_module(wgpu::include_spirv!("shaders/shader.frag.spv")),
				entry_point: "main"
			}),
			rasterization_state: Some(wgpu::RasterizationStateDescriptor {
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: wgpu::CullMode::Back,
				depth_bias: 0,
				depth_bias_slope_scale: 0.,
				depth_bias_clamp: 0.,
				clamp_depth: false
			}),
			color_states: &[wgpu::ColorStateDescriptor {
				format: sc_desc.format,
				color_blend: wgpu::BlendDescriptor::REPLACE,
				alpha_blend: wgpu::BlendDescriptor::REPLACE,
				write_mask: wgpu::ColorWrite::ALL
			}],
			primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
			depth_stencil_state: None,
			vertex_state: wgpu::VertexStateDescriptor {
				index_format: wgpu::IndexFormat::Uint16,
				vertex_buffers: &[]
			},
			sample_count: 1,
			sample_mask: !0,
			alpha_to_coverage_enabled: false
		});

	let buffer =
		device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: 40,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			mapped_at_creation: false
		});

	let bind_group =
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(buffer.slice(..))
				}
			]
		});

	window.request_redraw();

	let redraw = |swapchain: &mut wgpu::SwapChain, device: &wgpu::Device, logical_size: LogicalSize<f64>| {
		let frame = swapchain.get_current_frame().unwrap();

		let mut encoder = device.create_command_encoder(&enc_desc);

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				color_attachments: &[
					wgpu::RenderPassColorAttachmentDescriptor {
						attachment: &frame.output.view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
							store: true
						}
					}
				],
				depth_stencil_attachment: None
			});

			render_pass.set_pipeline(&pipeline);
			render_pass.set_bind_group(0, &bind_group, &[]);
			render_pass.draw(0..4, 0..1);
		}

		queue.write_buffer(&buffer, 0, bytemuck::bytes_of(&[
			logical_size.width as f32, logical_size.height as f32
		]));

		queue.submit(std::iter::once(encoder.finish()));
	};

	event_loop.run_return(move |event, _, flow| {
		*flow = ControlFlow::Wait;

		match event {
			Event::WindowEvent {
				window_id: id,
				event,
				..
			} if id == window.id() => match event {
				WindowEvent::CloseRequested => *flow = ControlFlow::Exit,

				WindowEvent::Resized(new_size) => {
					sc_desc.width = new_size.width;
					sc_desc.height = new_size.height;
					swapchain = device.create_swap_chain(&surface, &sc_desc);
					redraw(&mut swapchain, &device, new_size.to_logical(window.scale_factor()));
				}

				_ => {}
			}

			Event::RedrawRequested(id) if id == window.id() => {
				let window_size = PhysicalSize::new(sc_desc.width, sc_desc.height)
					.to_logical(window.scale_factor());

				redraw(&mut swapchain, &device, window_size);
			}

			_ => {}
		}
	})
}
