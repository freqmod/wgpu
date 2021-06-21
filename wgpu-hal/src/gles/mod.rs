#![allow(unused_variables)]

#[cfg(not(target_arch = "wasm32"))]
mod egl;

mod adapter;
mod conv;

#[cfg(not(target_arch = "wasm32"))]
use self::egl::{Instance, Surface};

use std::{ops::Range, sync::Arc};

#[derive(Clone)]
pub struct Api;
pub struct Context;
pub struct Encoder;
#[derive(Debug)]
pub struct Resource;

type DeviceResult<T> = Result<T, crate::DeviceError>;

impl crate::Api for Api {
    type Instance = Instance;
    type Surface = Surface;
    type Adapter = Adapter;
    type Device = Device;

    type Queue = Queue;
    type CommandEncoder = Encoder;
    type CommandBuffer = Resource;

    type Buffer = Resource;
    type Texture = Resource;
    type SurfaceTexture = Resource;
    type TextureView = Resource;
    type Sampler = Resource;
    type QuerySet = Resource;
    type Fence = Resource;

    type BindGroupLayout = Resource;
    type BindGroup = Resource;
    type PipelineLayout = Resource;
    type ShaderModule = Resource;
    type RenderPipeline = Resource;
    type ComputePipeline = Resource;
}

bitflags::bitflags! {
    /// Flags that affect internal code paths but do not
    /// change the exposed feature set.
    struct PrivateCapability: u32 {
        /// Support explicit layouts in shader.
        const EXPLICIT_LAYOUTS_IN_SHADER = 0x00002000;
    }
}

type TextureFormat = u32;

#[derive(Debug, Clone, Copy)]
enum VertexAttribKind {
    Float,   // glVertexAttribPointer
    Integer, // glVertexAttribIPointer
    Double,  // glVertexAttribLPointer
}

struct FormatDescription {
    tex_internal: u32,
    tex_external: u32,
    data_type: u32,
    num_components: u8,
    va_kind: VertexAttribKind,
}

struct AdapterShared {
    context: glow::Context,
    private_caps: PrivateCapability,
}

pub struct Adapter {
    shared: Arc<AdapterShared>,
}

pub struct Device {
    shared: Arc<AdapterShared>,
    main_vao: glow::VertexArray,
}

pub struct Queue {
    shared: Arc<AdapterShared>,
    features: wgt::Features,
}

impl crate::Queue<Api> for Queue {
    unsafe fn submit(
        &mut self,
        command_buffers: &[&Resource],
        signal_fence: Option<(&mut Resource, crate::FenceValue)>,
    ) -> DeviceResult<()> {
        Ok(())
    }
    unsafe fn present(
        &mut self,
        surface: &mut Surface,
        texture: Resource,
    ) -> Result<(), crate::SurfaceError> {
        Ok(())
    }
}

impl crate::Device<Api> for Device {
    unsafe fn exit(self) {}
    unsafe fn create_buffer(&self, desc: &crate::BufferDescriptor) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_buffer(&self, buffer: Resource) {}
    unsafe fn map_buffer(
        &self,
        buffer: &Resource,
        range: crate::MemoryRange,
    ) -> DeviceResult<crate::BufferMapping> {
        Err(crate::DeviceError::Lost)
    }
    unsafe fn unmap_buffer(&self, buffer: &Resource) -> DeviceResult<()> {
        Ok(())
    }
    unsafe fn flush_mapped_ranges<I>(&self, buffer: &Resource, ranges: I) {}
    unsafe fn invalidate_mapped_ranges<I>(&self, buffer: &Resource, ranges: I) {}

    unsafe fn create_texture(&self, desc: &crate::TextureDescriptor) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_texture(&self, texture: Resource) {}
    unsafe fn create_texture_view(
        &self,
        texture: &Resource,
        desc: &crate::TextureViewDescriptor,
    ) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_texture_view(&self, view: Resource) {}
    unsafe fn create_sampler(&self, desc: &crate::SamplerDescriptor) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_sampler(&self, sampler: Resource) {}

    unsafe fn create_command_encoder(
        &self,
        desc: &crate::CommandEncoderDescriptor<Api>,
    ) -> DeviceResult<Encoder> {
        Ok(Encoder)
    }
    unsafe fn destroy_command_encoder(&self, encoder: Encoder) {}

    unsafe fn create_bind_group_layout(
        &self,
        desc: &crate::BindGroupLayoutDescriptor,
    ) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_bind_group_layout(&self, bg_layout: Resource) {}
    unsafe fn create_pipeline_layout(
        &self,
        desc: &crate::PipelineLayoutDescriptor<Api>,
    ) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_pipeline_layout(&self, pipeline_layout: Resource) {}
    unsafe fn create_bind_group(
        &self,
        desc: &crate::BindGroupDescriptor<Api>,
    ) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_bind_group(&self, group: Resource) {}

    unsafe fn create_shader_module(
        &self,
        desc: &crate::ShaderModuleDescriptor,
        shader: crate::ShaderInput,
    ) -> Result<Resource, crate::ShaderError> {
        Ok(Resource)
    }
    unsafe fn destroy_shader_module(&self, module: Resource) {}
    unsafe fn create_render_pipeline(
        &self,
        desc: &crate::RenderPipelineDescriptor<Api>,
    ) -> Result<Resource, crate::PipelineError> {
        Ok(Resource)
    }
    unsafe fn destroy_render_pipeline(&self, pipeline: Resource) {}
    unsafe fn create_compute_pipeline(
        &self,
        desc: &crate::ComputePipelineDescriptor<Api>,
    ) -> Result<Resource, crate::PipelineError> {
        Ok(Resource)
    }
    unsafe fn destroy_compute_pipeline(&self, pipeline: Resource) {}

    unsafe fn create_query_set(
        &self,
        desc: &wgt::QuerySetDescriptor<crate::Label>,
    ) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_query_set(&self, set: Resource) {}
    unsafe fn create_fence(&self) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn destroy_fence(&self, fence: Resource) {}
    unsafe fn get_fence_value(&self, fence: &Resource) -> DeviceResult<crate::FenceValue> {
        Ok(0)
    }
    unsafe fn wait(
        &self,
        fence: &Resource,
        value: crate::FenceValue,
        timeout_ms: u32,
    ) -> DeviceResult<bool> {
        Ok(true)
    }

    unsafe fn start_capture(&self) -> bool {
        false
    }
    unsafe fn stop_capture(&self) {}
}

impl crate::CommandEncoder<Api> for Encoder {
    unsafe fn begin_encoding(&mut self, label: crate::Label) -> DeviceResult<()> {
        Ok(())
    }
    unsafe fn discard_encoding(&mut self) {}
    unsafe fn end_encoding(&mut self) -> DeviceResult<Resource> {
        Ok(Resource)
    }
    unsafe fn reset_all<I>(&mut self, command_buffers: I) {}

    unsafe fn transition_buffers<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::BufferBarrier<'a, Api>>,
    {
    }

    unsafe fn transition_textures<'a, T>(&mut self, barriers: T)
    where
        T: Iterator<Item = crate::TextureBarrier<'a, Api>>,
    {
    }

    unsafe fn fill_buffer(&mut self, buffer: &Resource, range: crate::MemoryRange, value: u8) {}

    unsafe fn copy_buffer_to_buffer<T>(&mut self, src: &Resource, dst: &Resource, regions: T) {}

    unsafe fn copy_texture_to_texture<T>(
        &mut self,
        src: &Resource,
        src_usage: crate::TextureUse,
        dst: &Resource,
        regions: T,
    ) {
    }

    unsafe fn copy_buffer_to_texture<T>(&mut self, src: &Resource, dst: &Resource, regions: T) {}

    unsafe fn copy_texture_to_buffer<T>(
        &mut self,
        src: &Resource,
        src_usage: crate::TextureUse,
        dst: &Resource,
        regions: T,
    ) {
    }

    unsafe fn begin_query(&mut self, set: &Resource, index: u32) {}
    unsafe fn end_query(&mut self, set: &Resource, index: u32) {}
    unsafe fn write_timestamp(&mut self, set: &Resource, index: u32) {}
    unsafe fn reset_queries(&mut self, set: &Resource, range: Range<u32>) {}
    unsafe fn copy_query_results(
        &mut self,
        set: &Resource,
        range: Range<u32>,
        buffer: &Resource,
        offset: wgt::BufferAddress,
    ) {
    }

    // render

    unsafe fn begin_render_pass(&mut self, desc: &crate::RenderPassDescriptor<Api>) {}
    unsafe fn end_render_pass(&mut self) {}

    unsafe fn set_bind_group(
        &mut self,
        layout: &Resource,
        index: u32,
        group: &Resource,
        dynamic_offsets: &[wgt::DynamicOffset],
    ) {
    }
    unsafe fn set_push_constants(
        &mut self,
        layout: &Resource,
        stages: wgt::ShaderStage,
        offset: u32,
        data: &[u32],
    ) {
    }

    unsafe fn insert_debug_marker(&mut self, label: &str) {}
    unsafe fn begin_debug_marker(&mut self, group_label: &str) {}
    unsafe fn end_debug_marker(&mut self) {}

    unsafe fn set_render_pipeline(&mut self, pipeline: &Resource) {}

    unsafe fn set_index_buffer<'a>(
        &mut self,
        binding: crate::BufferBinding<'a, Api>,
        format: wgt::IndexFormat,
    ) {
    }
    unsafe fn set_vertex_buffer<'a>(&mut self, index: u32, binding: crate::BufferBinding<'a, Api>) {
    }
    unsafe fn set_viewport(&mut self, rect: &crate::Rect<f32>, depth_range: Range<f32>) {}
    unsafe fn set_scissor_rect(&mut self, rect: &crate::Rect<u32>) {}
    unsafe fn set_stencil_reference(&mut self, value: u32) {}
    unsafe fn set_blend_constants(&mut self, color: &wgt::Color) {}

    unsafe fn draw(
        &mut self,
        start_vertex: u32,
        vertex_count: u32,
        start_instance: u32,
        instance_count: u32,
    ) {
    }
    unsafe fn draw_indexed(
        &mut self,
        start_index: u32,
        index_count: u32,
        base_vertex: i32,
        start_instance: u32,
        instance_count: u32,
    ) {
    }
    unsafe fn draw_indirect(
        &mut self,
        buffer: &Resource,
        offset: wgt::BufferAddress,
        draw_count: u32,
    ) {
    }
    unsafe fn draw_indexed_indirect(
        &mut self,
        buffer: &Resource,
        offset: wgt::BufferAddress,
        draw_count: u32,
    ) {
    }
    unsafe fn draw_indirect_count(
        &mut self,
        buffer: &Resource,
        offset: wgt::BufferAddress,
        count_buffer: &Resource,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }
    unsafe fn draw_indexed_indirect_count(
        &mut self,
        buffer: &Resource,
        offset: wgt::BufferAddress,
        count_buffer: &Resource,
        count_offset: wgt::BufferAddress,
        max_count: u32,
    ) {
    }

    // compute

    unsafe fn begin_compute_pass(&mut self, desc: &crate::ComputePassDescriptor) {}
    unsafe fn end_compute_pass(&mut self) {}

    unsafe fn set_compute_pipeline(&mut self, pipeline: &Resource) {}

    unsafe fn dispatch(&mut self, count: [u32; 3]) {}
    unsafe fn dispatch_indirect(&mut self, buffer: &Resource, offset: wgt::BufferAddress) {}
}
