mod fragment_shader;
mod vertex_shader;

use crate::renderer::{vertex_format::VertexSimple, render_queue::RenderQueue};
use std::{error::Error, sync::Arc};
use vulkano::{instance::{Instance, InstanceCreateInfo}, device::{physical::{PhysicalDevice, PhysicalDeviceType, QueueFamily}, Device, DeviceCreateInfo, QueueCreateInfo, Queue, DeviceExtensions}, image::{view::ImageView, ImageUsage, SwapchainImage}, render_pass::{Framebuffer, FramebufferCreateInfo, Subpass, RenderPass}, command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents, PrimaryAutoCommandBuffer, CommandBufferExecFuture, pool::standard::StandardCommandPoolAlloc, RenderPassBeginInfo}, pipeline::{graphics::{viewport::{Viewport, ViewportState}, vertex_input::BuffersDefinition, input_assembly::InputAssemblyState}, GraphicsPipeline}, swapchain::{Swapchain, SwapchainCreateInfo, AcquireError, SwapchainCreationError, acquire_next_image, Surface, PresentFuture, SwapchainAcquireFuture, PresentMode}, shader::ShaderModule, sync::{FlushError, self, GpuFuture, FenceSignalFuture, JoinFuture}, format::ClearValue};
use vulkano_win::VkSurfaceBuild;
use winit::{event_loop::{EventLoop}, window::{WindowBuilder, Window}};

use crate::log_debug;

use super::render_item::RenderItem;

pub struct VulkanRenderer {
    // Renderer structures
    instance: Arc<Instance>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    pipeline: Arc<GraphicsPipeline>,
    render_queue: RenderQueue,
    viewport: Viewport,
    command_buffers: Vec<Arc<PrimaryAutoCommandBuffer>>,
    surface: Arc<Surface<Window>>,
    render_pass: Arc<RenderPass>,
    framebuffers: Vec<Arc<Framebuffer>>,
    // Shaders
    vertex_shader: Arc<ShaderModule>,
    fragment_shader: Arc<ShaderModule>,
    // Frame state
    window_resized: bool,
    recreate_swapchain: bool,
    previous_fence_i: usize,
    fences: Vec<Option<Arc<FenceSignalFuture<PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture<Window>>, Arc<PrimaryAutoCommandBuffer<StandardCommandPoolAlloc>>>, Window>>>>>
}

impl VulkanRenderer {
    pub fn schedule_resize(&mut self) {
        self.window_resized = true;
    }

    #[profiling::function]
    pub fn add_mesh(&mut self, vertices: Vec<VertexSimple>, indices: Vec<u32>) {
        self.render_queue.add_render_item(RenderItem::new(
           &self.queue, vertices, indices
        ));
    }

    #[profiling::function]
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        // We cannot render anything if there is no vertex data
        if !self.render_queue.has_any_data() {
            return Ok(());
        }

        self.command_buffers = get_command_buffers(
            &self.device, &self.queue, &self.pipeline, &self.framebuffers, &self.render_queue,
        )?;

        if self.window_resized || self.recreate_swapchain {
            profiling::scope!("Window resized");
            self.recreate_swapchain = false;

            let new_dimensions = self.surface.window().inner_size();

            let (new_swapchain, new_images) = 
                match self.swapchain.recreate(SwapchainCreateInfo {
                    image_extent: new_dimensions.into(),
                    ..self.swapchain.create_info()
                }
            ) {
                Ok(r) => r,
                Err(SwapchainCreationError::ImageExtentNotSupported { .. }) => return Ok(()),
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
            self.swapchain = new_swapchain;
            self.framebuffers = get_framebuffers(&new_images, &self.render_pass);
            if self.window_resized {
                self.window_resized = false;

                self.viewport.dimensions = new_dimensions.into();
                let new_pipeline = get_pipeline(
                    &self.device, &self.vertex_shader, &self.fragment_shader, self.viewport.clone(), &self.render_pass).unwrap();
                self.command_buffers = get_command_buffers(
                    &self.device, &self.queue, &new_pipeline, &&self.framebuffers, &self.render_queue).unwrap();
            }
        }

        let (image_i, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Ok(());
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        // wait for the fence related to this image to finish (normally this would be the oldest fence)
        if let Some(image_fence) = &self.fences[image_i] {
            profiling::scope!("Wait for fence");
            image_fence.wait(None).unwrap();
        }

        let previous_future = match self.fences[self.previous_fence_i].clone() {
            // Create a NowFuture
            None => {
                let mut now = sync::now(self.device.clone());
                now.cleanup_finished();
                now.boxed()
            }
            // Use the existing FenceSignalFuture
            Some(fence) => fence.boxed(),
        };

        profiling::scope!("Join and present");
        let future = previous_future
            .join(acquire_future);
        let future = (|| {
            profiling::scope!("Then execute");
            return future
                .then_execute(self.queue.clone(), self.command_buffers[image_i].clone())
                .unwrap();
        })();
        let future = (|| {
            profiling::scope!("Then sawpchain");
            return future
                .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_i);
        })();
        let future = (|| {
            profiling::scope!("Then signal fence");
            return future
                .then_signal_fence_and_flush();
        })();
            

        {
            profiling::scope!("Match future");
        self.fences[image_i] = match future {
            Ok(value) => Some(Arc::new(value)),
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                None
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                None
            }
        };
        }

        self.previous_fence_i = image_i;
        return Ok(());
    }

    pub fn new(event_loop: &EventLoop<()>) -> Result<VulkanRenderer, Box<dyn Error>> {
        // Implement vertex buffer structs
        vulkano::impl_vertex!(VertexSimple, position);
        // Create Vulkan instance, requires Vulkan-capable OS
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        })?;
        // Build window
        let surface = WindowBuilder::new()
            .build_vk_surface(event_loop, instance.clone())?;
        // Find GPU, requires Vulkan-capable one
        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (physical_device, queue_family) = select_physical_device(&instance, &device_extensions)?;
        log_debug!("Physical GPU device capabilities: {:#?}", physical_device.properties());
        // Create render Device and get render queues
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                // here we pass the desired queue families that we want to use
                queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
                enabled_extensions: physical_device
                    .supported_extensions()
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
        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::color_attachment(), // What the images are going to be used for
                composite_alpha,
                present_mode: PresentMode::Immediate,
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
        let viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: surface.window().inner_size().into(),
            depth_range: 0.0..1.0,
        };
        // Create graphic pipeline
        let pipeline = get_pipeline(&device, &vs, &fs, viewport.clone(), &render_pass)?;

        // Prepare vertex and index buffers
        let render_queue = RenderQueue::new();

        // Prepare command buffers
        let command_buffers = get_command_buffers(
            &device,
            &queue,
            &pipeline,
            &framebuffers,
            &render_queue,
        )?;

        let frames_in_flight = images.len();
        let renderer = VulkanRenderer {
            instance: instance,
            device: device,
            queue: queue,
            swapchain: swapchain,
            pipeline: pipeline,
            render_queue: render_queue,
            command_buffers: command_buffers,
            framebuffers: framebuffers,
            fences: vec![None; frames_in_flight],
            fragment_shader: fs,
            previous_fence_i: 0,
            render_pass: render_pass,
            surface: surface,
            vertex_shader: vs,
            viewport: viewport,
            window_resized: false,
            recreate_swapchain: false
        };

        return Ok(renderer);
    }
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

#[profiling::function]
fn get_command_buffers(device: &Arc<Device>, queue: &Arc<Queue>, pipeline: &Arc<GraphicsPipeline>,
    framebuffers: &Vec<Arc<Framebuffer>>, render_queue: &RenderQueue)
    -> Result<Vec<Arc<PrimaryAutoCommandBuffer>>, Box<dyn Error>> {
    // We cannot create any command buffers if there is no vertex data
    if !render_queue.has_any_data() {
        return Ok(Default::default());
    }

    framebuffers
        .iter()
        .map(|framebuffer| -> Result<Arc<PrimaryAutoCommandBuffer>, Box<dyn Error>> {
            profiling::scope!("Command buffer creation");
            let mut builder = AutoCommandBufferBuilder::primary(
                device.clone(),
                queue.family(),
                CommandBufferUsage::OneTimeSubmit,
            )?;
            let render_pass_info = RenderPassBeginInfo::framebuffer(framebuffer.clone());
            builder
                .begin_render_pass(
                    render_pass_info,
                    SubpassContents::Inline
                )?
                .bind_pipeline_graphics(pipeline.clone());

            render_queue.draw_all(&mut builder)?;

            builder
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
