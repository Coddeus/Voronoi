use std::{fs::File, io::BufWriter, path::Path, sync::Arc, time::Instant};
use fastrand::f32;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer}, command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo, CopyImageToBufferInfo, PrimaryAutoCommandBuffer, PrimaryCommandBufferAbstract, RenderPassBeginInfo
    }, descriptor_set::{allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet}, device::{
        physical::PhysicalDeviceType, Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags
    }, format::Format, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage, SampleCount}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo}, memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, padded::Padded, pipeline::{
        compute::ComputePipelineCreateInfo, graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::VertexInputState,
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        }, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo
    }, render_pass::{Framebuffer, FramebufferCreateInfo, Subpass}, sync::GpuFuture, VulkanLibrary
};


const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const FRAMES_NUM: u32 = 60 * 10; // 10 seconds
const FRAMERATE: f32 = 60.0;
//
const POINTS_NUM: u32 = 30;
const POINTS_SPEED: f32 = 0.5;


mod vert_s {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/vert.glsl",
    }
}
mod frag_s {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/frag.glsl",
    }
}
mod update_s {
    vulkano_shaders::shader! {
        ty: "compute",
        path: "src/shaders/update.glsl",
    }
}

// Draw frame of a Voronoi animation video.
fn main() {

    // ---------------------------------------------------------------------------------------------------- Vulkan base initialization, with sample_rate_shading device feature

    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
            ..Default::default()
        },
    )
    .unwrap();

    let device_extensions = DeviceExtensions {
        ..DeviceExtensions::empty()
    };
    let (physical_device, queue_family_index) = instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|p| p.supported_extensions().contains(&device_extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .position(|q| q.queue_flags.intersects(QueueFlags::GRAPHICS))
                .map(|i| (p, i as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            PhysicalDeviceType::Other => 4,
            _ => 5,
        })
        .unwrap();

    println!(
        "Using device: {} (type: {:?})",
        physical_device.properties().device_name,
        physical_device.properties().device_type,
    );

    let (device, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            enabled_extensions: device_extensions,
            enabled_features: Features {
                sample_rate_shading: true,
                ..Default::default()
            },
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            ..Default::default()
        },
    )
    .unwrap();
    let queue = queues.next().unwrap();

    // ---------------------------------------------------------------------------------------------------- Allocators

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
    let cb_allocator = StandardCommandBufferAllocator::new(device.clone(), Default::default());
    let ds_allocator = StandardDescriptorSetAllocator::new(device.clone(), Default::default());

    // ---------------------------------------------------------------------------------------------------- Buffers & push constants

    // Push constants struct
    #[derive(BufferContents, Clone)]
    #[repr(C)]
    struct General {
        resolution: [f32; 2],
        //
        time: f32,
        delta_time: f32,
        //
        points_num: u32,
        points_speed: f32,
    }

    // Points Buffer
    #[derive(BufferContents, Debug)]
    #[repr(C)]
    struct Point {
        pos: [f32; 2],
        dir: [f32; 2],
        color: [f32; 4],
    }

    let all_points = (0..POINTS_NUM).into_iter().map(|_| {
        let col = f32();
        Point {
            pos: [f32() * WIDTH as f32 / HEIGHT as f32, f32()],
            dir: [f32()*2.0-1.0, f32()*2.0-1.0],
            color: [f32(), f32(), f32(), 1.0],
        }
}   ).collect::<Vec<Point>>();
    println!("{:?}", all_points);
    let points_buffer: Subbuffer<[Point]> = create_buffer(
        queue.clone(), 
        memory_allocator.clone(),
        &cb_allocator,
        BufferUsage::TRANSFER_DST | BufferUsage::STORAGE_BUFFER,
        all_points.into_iter(),
        POINTS_NUM as u64
    );

    // Output buffer, converted to png image
    let buf = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_DST,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_RANDOM_ACCESS,
            ..Default::default()
        },
        (0..WIDTH * HEIGHT * 4).map(|_| 0u8),
    )
    .unwrap();

    // ---------------------------------------------------------------------------------------------------- Images

    // Multisample image
    let intermediary = ImageView::new_default(
        Image::new(
            memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [WIDTH, HEIGHT, 1],
                usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                samples: SampleCount::Sample8,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .unwrap(),
    )
    .unwrap();

    // Final image & view
    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [WIDTH, HEIGHT, 1],
            usage: ImageUsage::TRANSFER_SRC
                | ImageUsage::TRANSFER_DST
                | ImageUsage::COLOR_ATTACHMENT
                | ImageUsage::STORAGE,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();
    let view = ImageView::new_default(image.clone()).unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            intermediary: {
                format: Format::R8G8B8A8_UNORM,
                samples: 8,
                load_op: DontCare,
                store_op: DontCare,
            },
            color: {
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
                load_op: DontCare,
                store_op: Store,
            },
        },
        pass: {
            color: [intermediary],
            color_resolve: [color],
            depth_stencil: {},
        },
    )
    .unwrap();

    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![intermediary, view],
            ..Default::default()
        },
    )
    .unwrap();

    let draw_pipeline = {
        let vs = vert_s::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = frag_s::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let vertex_input_state = VertexInputState::default();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        let subpass = Subpass::from(render_pass, 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState {
                    rasterization_samples: subpass.num_samples().unwrap(),
                    ..Default::default()
                }),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };
    let draw_descriptor_set: Arc<PersistentDescriptorSet> = PersistentDescriptorSet::new(
        &ds_allocator,
        draw_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap()
            .clone(),
        [
            WriteDescriptorSet::buffer(0, points_buffer.clone()),
        ],
        [

        ],
    )
    .unwrap();


    let update_pipeline: Arc<ComputePipeline> = {
        let cs = update_s::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let stage = PipelineShaderStageCreateInfo::new(cs);
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .unwrap()
    };
    let update_descriptor_set: Arc<PersistentDescriptorSet> = PersistentDescriptorSet::new(
        &ds_allocator,
        update_pipeline
            .layout()
            .set_layouts()
            .get(0)
            .unwrap()
            .clone(),
        [
            WriteDescriptorSet::buffer(0, points_buffer.clone()),
        ],
        [

        ],
    )
    .unwrap();

    let viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [WIDTH as f32, HEIGHT as f32],
        depth_range: 0.0..=1.0,
    };

    let start = Instant::now();
    (0..FRAMES_NUM).into_iter().for_each(|n| {
        let current_instant = Instant::now();
        let general_data: General = General {
            resolution: [WIDTH as f32, HEIGHT as f32],
            //
            time: (current_instant-start).as_secs_f32(),
            delta_time: 1.0 / FRAMERATE,
            //
            points_num: POINTS_NUM,
            points_speed: POINTS_SPEED,
        };

        let mut update_builder = AutoCommandBufferBuilder::primary(
            &cb_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        update_builder
            .push_constants(
                update_pipeline.layout().clone(),
                0,
                general_data.clone())
            .unwrap()
            .bind_pipeline_compute(update_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                update_pipeline.layout().clone(),
                0,
                update_descriptor_set.clone(),
            )
            .unwrap()
            .dispatch([POINTS_NUM/64 + 1, 1, 1])
            .unwrap();
        let update_command_buffer = update_builder.build().unwrap();
        let finished = update_command_buffer.execute(queue.clone()).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let mut draw_builder = AutoCommandBufferBuilder::primary(
            &cb_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();
        draw_builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![None, None], // Some([0.0, 0.0, 0.0, 1.0].into())
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                Default::default(),
            )
            .unwrap()
            .set_viewport(0, [viewport.clone()].into_iter().collect())
            .unwrap()
            .bind_pipeline_graphics(draw_pipeline.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                draw_pipeline.layout().clone(),
                0,
                draw_descriptor_set.clone(),
            )
            .unwrap()
            .push_constants(
                draw_pipeline.layout().clone(),
                0,
                general_data
            )
            .unwrap()
            .draw(6, 1, 0, 0)
            .unwrap()
            .end_render_pass(Default::default())
            .unwrap()
            .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image.clone(), buf.clone()))
            .unwrap();
        let draw_command_buffer = draw_builder.build().unwrap();
        let finished = draw_command_buffer.execute(queue.clone()).unwrap();
        finished
            .then_signal_fence_and_flush()
            .unwrap()
            .wait(None)
            .unwrap();

        let buffer_content = buf.read().unwrap();
        let filename = format!("output/{:09}.png", n);
        let path = Path::new(filename.as_str());
        let file = File::create(path).unwrap();
        let w = &mut BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, WIDTH, HEIGHT); // Width is 2 pixels and height is 1.
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&buffer_content).unwrap();

        if let Ok(path) = path.canonicalize() {
            println!("Saved to {} in {:?}", path.display(), start.elapsed());
        }
    })
}


/// Creates a device-local buffer with the given `len` and `iter` iterated data. `usage` must contain `BufferUsage::TRANSFER_DST`
fn create_buffer<T, I>(
    queue: Arc<Queue>,
    memory_allocator: Arc<dyn MemoryAllocator>,
    cb_allocator: &StandardCommandBufferAllocator,
    usage: BufferUsage,
    iter: I,
    len: u64,
) -> Subbuffer<[T]>
where
    T: BufferContents,
    I: IntoIterator<Item = T>,
    I::IntoIter: ExactSizeIterator,
{
    let buffer: Subbuffer<[T]> = Buffer::new_slice::<T>(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        },
        len,
    )
    .expect("failed to create buffer");

    let staging_buffer: Subbuffer<[T]> = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        iter,
    )
    .expect("failed to create staging_buffer");

    let mut cbb: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
        AutoCommandBufferBuilder::primary(
            cb_allocator,
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .expect("failed to create cbb");

    cbb
        .copy_buffer(CopyBufferInfo::buffers(
            staging_buffer.clone(),
            buffer.clone(),
        ))
        .unwrap();

    let copy_command_buffer: Arc<PrimaryAutoCommandBuffer> = cbb.build().unwrap();

    copy_command_buffer
        .execute(queue.clone())
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .wait(None /* timeout */)
        .unwrap();


    buffer
}