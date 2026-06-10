//! GPU round-trip gate for the timestamp pass profiler (PH2D_FLUID_PROFILE).
//!
//! Exercises the three risky pieces on the real backend (Metal locally):
//! 1. EMPTY marker passes carrying begin-only / end-only timestamp writes
//!    (the copy/Vello span idiom — Apple Silicon is stage-boundary-only);
//! 2. query reuse across frames while the prior frame's resolve readback is
//!    still in flight (the pipelined ring);
//! 3. `resolve_query_set` over a range that includes span begins whose end
//!    landed in a later submit.
//!
//! Pass criterion: zero uncaptured device errors across `WINDOW`+ frames
//! (a wgpu validation error on any of the above fires the error handler).
//! `#[ignore]` — needs a real adapter (no GPU on CI); run with `--ignored`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
#[ignore = "requires a GPU adapter (no GPU on CI); run with --ignored on a dev machine"]
fn profiler_round_trip_zero_validation_errors() {
    let instance = wgpu::Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    })) else {
        return; // no adapter on this machine — nothing to assert
    };
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        eprintln!("adapter lacks TIMESTAMP_QUERY — profiler inert on this hardware, skipping");
        return;
    }
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("pass-profiler gate device"),
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .expect("request_device");

    let errored = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&errored);
    device.on_uncaptured_error(Arc::new(move |e: wgpu::Error| {
        eprintln!("uncaptured device error: {e}");
        flag.store(true, Ordering::Release);
    }));

    ph2d_gpu::pass_profiler::init_forced(&device, &queue);

    // A real compute workload to time (an ALU-heavy kernel over 1M threads):
    // proves the per-pass timestamps measure actual execution — all-zero
    // samples would also pass the validation-only criterion. (A bare COPY
    // bracketed by empty marker passes is NOT asserted on: with no hazard
    // chain Metal may overlap the blit with the markers, legally reading ~0.)
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gate busy kernel"),
        source: wgpu::ShaderSource::Wgsl(
            r#"
            @group(0) @binding(0) var<storage, read_write> out: array<f32>;
            @compute @workgroup_size(64)
            fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
                var acc = f32(gid.x);
                for (var i = 0u; i < 256u; i = i + 1u) {
                    acc = acc * 1.0000001 + 0.5;
                }
                out[gid.x % 65536u] = acc;
            }
            "#
            .into(),
        ),
    });
    let busy_out = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gate busy out"),
        size: 65536 * 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    let busy_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("gate busy pipeline"),
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    let busy_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("gate busy bind"),
        layout: &busy_pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: busy_out.as_entire_binding(),
        }],
    });
    let size = 64 * 1024 * 1024;
    let src = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gate copy src"),
        size,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let dst = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("gate copy dst"),
        size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    // 400 frames — under heavy GPU load the readback ring saturates and skips
    // samples, so well over the 120-SAMPLE report window is needed to exercise
    // the print path.
    for _ in 0..400 {
        // (1) a "real" instrumented pass — empty is fine, timestamps still write.
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("gate frame encoder"),
        });
        {
            let mut pass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("gate instrumented pass"),
                timestamp_writes: ph2d_gpu::pass_profiler::compute_writes("gate.pass"),
            });
            pass.set_pipeline(&busy_pipeline);
            pass.set_bind_group(0, &busy_bind, &[]);
            pass.dispatch_workgroups(16384, 1, 1);
        }
        // (2) a copy-span bracket inside the same encoder, around real work.
        let span = ph2d_gpu::pass_profiler::copy_span_begin(&mut enc, "gate.copy");
        enc.copy_buffer_to_buffer(&src, 0, &dst, 0, size);
        if let Some(t) = span {
            ph2d_gpu::pass_profiler::copy_span_end(&mut enc, t);
        }
        queue.submit([enc.finish()]);
        // (3) a cross-submit span (the Vello bracket idiom).
        let span = ph2d_gpu::pass_profiler::span_begin(&device, &queue, "gate.span");
        if let Some(t) = span {
            ph2d_gpu::pass_profiler::span_end(&device, &queue, t);
        }
        ph2d_gpu::pass_profiler::end_frame(&device, &queue);
        // Drain per frame: the app's present backpressure throttles the CPU to
        // GPU speed; without a swapchain this wait stands in for it (otherwise
        // the queue runs arbitrarily deep and the readback ring just skips).
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
    }
    let _ = device.poll(wgpu::PollType::wait_indefinitely());
    ph2d_gpu::pass_profiler::end_frame(&device, &queue);
    let _ = device.poll(wgpu::PollType::wait_indefinitely());

    assert!(
        !errored.load(Ordering::Acquire),
        "pass profiler produced wgpu validation errors (see stderr above)"
    );
}
