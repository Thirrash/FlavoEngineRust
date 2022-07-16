mod fragment_shader;
mod vertex_shader;

use std::{error::Error, sync::Arc};
use bytemuck::{Pod, Zeroable};
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily}, Device, DeviceCreateInfo, QueueCreateInfo, Queue, DeviceExtensions}, buffer::{CpuAccessibleBuffer, BufferUsage, TypedBufferAccess}, image::{view::ImageView, ImageUsage, SwapchainImage}, render_pass::{Framebuffer, FramebufferCreateInfo, Subpass, RenderPass}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents, PrimaryAutoCommandBuffer}, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline}, swapchain::{Swapchain, SwapchainCreateInfo, AcquireError, self, SwapchainCreationError}, shader::ShaderModule, sync::{FlushError, self, GpuFuture, FenceSignalFuture}};
use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::{EventLoop, ControlFlow}, window::{WindowBuilder, Window}, event::{Event, WindowEvent}};

#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct VertexSimple {
    position: [f32; 2]
}

pub fn run_render_loop() -> Result<(), Box<dyn Error>> {
    // Implement vertex buffer structs
    vulkano::impl_vertex!(VertexSimple, position);

    // Create Vulkan instance, requires Vulkan-capable OS
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(InstanceCreateInfo {
        enabled_extensions: required_extensions,
        ..Default::default()
    })?;
    // Create window
    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())?;
    // Find GPU, requires Vulkan-capable one
    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (physical_device, queue_family) = select_physical_device(&instance, &device_extensions)?;
    // Create render Device and get render queues
    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            // here we pass the desired queue families that we want to use
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            enabled_extensions: physical_device
                .required_extensions()
                .union(&device_extensions),
            ..Default::default()
        },
    )?;
    // Extract render queue
    let queue = queues.next()
        .ok_or("No render queue available")?;
    // Create swapchain
    let surface_capabilities = physical_device
        .surface_capabilities(&surface, Default::default())?;
    let dimensions = surface.window().inner_size();
    let composite_alpha = surface_capabilities.supported_composite_alpha.iter().next()
        .ok_or("No supported composite alphas")?;
    let image_format = Some(
        physical_device.surface_formats(&surface, Default::default())?[0].0,
    );
    let (mut swapchain, images) = Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: surface_capabilities.min_image_count + 1, // How many buffers to use in the swapchain
            image_format,
            image_extent: dimensions.into(),
            image_usage: ImageUsage::color_attachment(), // What the images are going to be used for
            composite_alpha,
            ..Default::default()
        },
    )?;
    // Create render pass object
    let render_pass = get_render_pass(&device, &swapchain)?;
    // Create framebuffers
    let framebuffers = get_framebuffers(&images, &render_pass);
    // Compile shaders
    let vs = vertex_shader::vs::load(device.clone())?;
    let fs = fragment_shader::fs::load(device.clone())?;
    // Prepare viewport
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: surface.window().inner_size().into(),
        depth_range: 0.0..1.0,
    };
    // Create graphic pipeline
    let pipeline = get_pipeline(&device, &vs, &fs, viewport.clone(), &render_pass)?;

    // Prepare vertex buffer
    let vertex1 = VertexSimple { position: [-0.5, -0.5] };
    let vertex2 = VertexSimple { position: [ 0.0,  0.5] };
    let vertex3 = VertexSimple { position: [ 0.5, -0.25] };
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )?;

    // Prepare command buffers
    let mut command_buffers = get_command_buffers(
        &device,
        &queue,
        &pipeline,
        &framebuffers,
        &vertex_buffer,
    )?;

    let mut window_resized = false;
    let mut recreate_swapchain = false;

    let frames_in_flight = images.len();
    let mut fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
    let mut previous_fence_i = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => {
            window_resized = true;
        }
        Event::MainEventsCleared => {
            if window_resized || recreate_swapchain {
                recreate_swapchain = false;

                let new_dimensions = surface.window().inner_size();

                let (new_swapchain, new_images) = match swapchain.recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..swapchain.create_info()
                }) {
                    Ok(r) => r,
                    Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return,
                    Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                };
                swapchain = new_swapchain;
                let new_framebuffers = get_framebuffers(&new_images, &render_pass);
                if window_resized {
                    window_resized = false;

                    viewport.dimensions = new_dimensions.into();
                    let new_pipeline = get_pipeline(
                        &device, &vs, &fs, viewport.clone(), &render_pass).unwrap();
                    command_buffers = get_command_buffers(
                        &device, &queue, &new_pipeline, &new_framebuffers,&vertex_buffer).unwrap();
                }
            }

            let (image_i, suboptimal, acquire_future) =
                match swapchain::acquire_next_image(swapchain.clone(), None) {
                    Ok(r) => r,
                    Err(AcquireError::OutOfDate) => {
                        recreate_swapchain = true;
                        return;
                    }
                    Err(e) => panic!("Failed to acquire next image: {:?}", e),
                };

            if suboptimal {
                recreate_swapchain = true;
            }

            // wait for the fence related to this image to finish (normally this would be the oldest fence)
            if let Some(image_fence) = &fences[image_i] {
                image_fence.wait(None).unwrap();
            }

            let previous_future = match fences[previous_fence_i].clone() {
                // Create a NowFuture
                None => {
                    let mut now = sync::now(device.clone());
                    now.cleanup_finished();
                    now.boxed()
                }
                // Use the existing FenceSignalFuture
                Some(fence) => fence.boxed(),
            };

            let future = previous_future
                .join(acquire_future)
                .then_execute(queue.clone(), command_buffers[image_i].clone())
                .unwrap()
                .then_swapchain_present(queue.clone(), swapchain.clone(), image_i)
                .then_signal_fence_and_flush();

            fences[image_i] = match future {
                Ok(value) => Some(Arc::new(value)),
                Err(FlushError::OutOfDate) => {
                    recreate_swapchain = true;
                    None
                }
                Err(e) => {
                    println!("Failed to flush future: {:?}", e);
                    None
                }
            };

            previous_fence_i = image_i;
        }
        _ => (),
    });
}

fn get_render_pass(device: &Arc<Device>, swapchain: &Arc<Swapchain<Window>>) -> Result<Arc<RenderPass>, Box<dyn Error>> {
    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: swapchain.image_format(),  // set the format the same as the swapchain
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )?;

    return Ok(render_pass);
}

fn get_framebuffers(images: &[Arc<SwapchainImage<Window>>], render_pass: &Arc<RenderPass>) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}


fn get_pipeline(device: &Arc<Device>, vs: &Arc<ShaderModule>, fs: &Arc<ShaderModule>,
    viewport: Viewport, render_pass: &Arc<RenderPass>) -> Result<Arc<GraphicsPipeline>, Box<dyn Error>> {
    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<VertexSimple>())
        .vertex_shader(vs.entry_point("main").ok_or("Couldn't bind vertex shader")?, ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").ok_or("Couldn't bind fragment shader")?, ())
        .render_pass(Subpass::from(render_pass.clone(), 0).ok_or("Could't set render pass subpass")?)
        .build(device.clone())?;
    return Ok(pipeline);
}

fn get_command_buffers(device: &Arc<Device>, queue: &Arc<Queue>, pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>, vertex_buffer: &Arc<CpuAccessibleBuffer<[VertexSimple]>>)
    -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, Box<dyn Error>> {
    framebuffers
        .iter()
        .map(|framebuffer| -> Result<Arc<PrimaryAutoCommandBuffer>, Box<dyn Error>> {
            let mut builder = AutoCommandBufferBuilder::primary(
                device.clone(),
                queue.family(),
                CommandBufferUsage::MultipleSubmit,  // don't forget to write the correct buffer usage
            )?;
            builder
                .begin_render_pass(
                    framebuffer.clone(),
                    SubpassContents::Inline,
                    vec![[0.0, 0.0, 1.0, 1.0].into()],
                )?
                .bind_pipeline_graphics(pipeline.clone())
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .draw(vertex_buffer.len() as u32, 1, 0, 0)?
                .end_render_pass()?;
            return Ok(Arc::new(builder.build()?));
        }).collect()
}

fn select_physical_device<'a>(instance: &'a Arc<Instance>, device_extensions: &DeviceExtensions)
    -> Result<(PhysicalDevice<'a>, QueueFamily<'a>), Box<dyn Error>> {
    let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
        .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
        .filter_map(|p| {
            p.queue_families()
                .find(|&q| q.supports_graphics())
                .map(|q| (p, q))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
        }).ok_or("Couldn't find proper GPU")?;

    return Ok((physical_device, queue_family));
}
