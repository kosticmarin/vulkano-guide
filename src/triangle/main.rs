use std::sync::Arc;

use image::{ImageBuffer, Rgba};
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    command_buffer::{CommandBuffer, SubpassContents},
    device::DeviceExtensions,
    sync::GpuFuture,
};
use vulkano::{command_buffer::AutoCommandBufferBuilder, instance::PhysicalDevice};
use vulkano::{
    command_buffer::DynamicState, instance::InstanceExtensions, pipeline::viewport::Viewport,
};
use vulkano::{
    device::Device,
    format::Format,
    image::{Dimensions, StorageImage},
};
use vulkano::{device::Features, framebuffer::Framebuffer};
use vulkano::{framebuffer::Subpass, instance::Instance, pipeline::GraphicsPipeline};

fn main() {
    // Initialization
    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("failed to create vulkan instance");

    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("no device available");

    // Device creation
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("couldn't find a graphical queue family");

    let (device, mut queues) = {
        Device::new(
            physical,
            &Features::none(),
            &DeviceExtensions::none(),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .expect("failed device creation")
    };

    let queue = queues.next().unwrap();

    let image = StorageImage::new(
        device.clone(),
        Dimensions::Dim2d {
            width: 1024,
            height: 1024,
        },
        Format::R8G8B8A8Unorm,
        Some(queue.family()),
    )
    .unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("Failed to create buffer");

    #[derive(Default, Clone, Copy)]
    struct Vertex {
        position: [f32; 2],
    }

    vulkano::impl_vertex!(Vertex, position);

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };
    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: Format::R8G8B8A8Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    let framebuffer = Arc::new(
        Framebuffer::start(render_pass.clone())
            .add(image.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
#version 450
layout(location = 0) in vec2 position;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
#version 450
layout(location = 0) out vec4 f_color;
void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}"
        }
    }

    let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
    let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let dynamic_state = DynamicState {
        viewports: Some(vec![Viewport {
            origin: [0.0, 0.0],
            dimensions: [1024.0, 1024.0],
            depth_range: 0.0..1.0,
        }]),
        ..DynamicState::none()
    };

    let mut builder =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap();

    builder
        .begin_render_pass(
            framebuffer.clone(),
            SubpassContents::Inline,
            vec![[0.0, 0.0, 1.0, 1.0].into()],
        )
        .unwrap()
        .draw(
            pipeline.clone(),
            &dynamic_state,
            vertex_buffer.clone(),
            (),
            (),
        )
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(image.clone(), buf.clone())
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let finished = command_buffer.execute(queue.clone()).unwrap();
    finished
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None)
        .unwrap();

    let buffer_content = buf.read().unwrap();
    let image = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
    image.save("image.png").unwrap();
}
