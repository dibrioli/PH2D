//! Dynamic per-frame instance buffer.
//!
//! Holds the `RenderInstance` array uploaded each frame. Grows
//! geometrically (×2) when `len > capacity`. Never shrinks (M5
//! intentionally simple — shrinking heuristics land if M6 finds them
//! load-bearing for memory budget per HR-13).
//!
//! Upload path: [`InstanceBuffer::upload`] uses `Queue::write_buffer`
//! (LLM1 audit "use queue.write_buffer for streaming uploads, not
//! map_async + memcpy" — keeps the data path off the device timeline
//! for sub-1MB uploads typical of a 2D scene).

use crate::sprite::RenderInstance;
use ph2d_gpu::GpuContext;

pub struct InstanceBuffer {
    buffer: wgpu::Buffer,
    /// Capacity in *instances* (not bytes).
    capacity: u32,
    /// Live count from the most recent upload.
    len: u32,
}

impl InstanceBuffer {
    pub fn new(gpu: &GpuContext, initial_capacity: u32) -> Self {
        let capacity = initial_capacity.max(1);
        let buffer = Self::allocate(gpu, capacity);
        Self {
            buffer,
            capacity,
            len: 0,
        }
    }

    fn allocate(gpu: &GpuContext, capacity: u32) -> wgpu::Buffer {
        let size = (capacity as u64) * (std::mem::size_of::<RenderInstance>() as u64);
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ph2d-render instance buffer"),
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    /// Upload `instances` into the buffer, growing if needed.
    /// Returns the count uploaded (== `instances.len()`).
    pub fn upload(&mut self, gpu: &GpuContext, instances: &[RenderInstance]) -> u32 {
        let needed = instances.len() as u32;
        if needed > self.capacity {
            let mut new_cap = self.capacity.max(1);
            while new_cap < needed {
                new_cap = new_cap.saturating_mul(2);
            }
            self.buffer = Self::allocate(gpu, new_cap);
            self.capacity = new_cap;
        }
        if needed > 0 {
            gpu.queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(instances));
        }
        self.len = needed;
        needed
    }

    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    pub fn len(&self) -> u32 {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn growth_doubles_until_fit() {
        // Pure-math sanity for the grow loop (no GPU needed).
        let mut cap: u32 = 4;
        let needed: u32 = 17;
        while cap < needed {
            cap = cap.saturating_mul(2);
        }
        assert_eq!(cap, 32);
    }
}
