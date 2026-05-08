//! C2.1 spike — mensageria throughput.
//!
//! Threshold (docs/spike/2026-05-plan.md L52-53): 100k mensagens/frame
//! dentro de sub-budget de 1.5 ms (HR-4).
//!
//! Cenário: 100 entities como senders, 100 como receivers (1000 pares),
//! 100k messages/frame total. Cada msg: id pré-interned, payload u64.
//! 60 frames de bench, mede p99 do tempo total send+dispatch por frame.

use ph2d_script::MessageBus;
use std::time::Instant;

const SENDERS: u64 = 100;
const RECEIVERS: u64 = 100;
const MSGS_PER_FRAME: usize = 100_000;
const FRAMES: usize = 60;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() { return 0.0; }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx]
}

fn main() {
    let mut bus = MessageBus::with_capacity(MSGS_PER_FRAME * 2, 16);

    // Pre-intern message names + register a noop handler.
    let damage_id = bus.intern("damage");
    let pickup_id = bus.intern("pickup");
    let interact_id = bus.intern("interact");

    let counter = std::cell::Cell::new(0u64);
    // Use a raw pointer trick to share `counter` across handlers without
    // the lifetime ceremony — for the bench only.
    let ctr_ptr: *const std::cell::Cell<u64> = &counter;
    bus.on(damage_id, Box::new(move |msg: &ph2d_script::Message| {
        unsafe { (*ctr_ptr).set((*ctr_ptr).get() + msg.payload); }
    }));
    bus.on(pickup_id, Box::new(move |_msg: &ph2d_script::Message| {
        unsafe { (*ctr_ptr).set((*ctr_ptr).get() + 1); }
    }));
    bus.on(interact_id, Box::new(move |_msg: &ph2d_script::Message| {
        unsafe { (*ctr_ptr).set((*ctr_ptr).get() + 1); }
    }));

    let ids = [damage_id, pickup_id, interact_id];

    let mut frame_ms: Vec<f64> = Vec::with_capacity(FRAMES);
    let mut grand_total = 0usize;

    for frame in 0..FRAMES {
        let t0 = Instant::now();
        // Send 100k messages
        for i in 0..MSGS_PER_FRAME {
            let sender = (i as u64) % SENDERS;
            let target = (i as u64) % RECEIVERS;
            let id = ids[i % 3];
            let payload = (frame as u64).wrapping_mul(1000) + i as u64;
            bus.send(sender, target, id, payload);
        }
        let processed = bus.dispatch_all();
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;
        frame_ms.push(elapsed);
        grand_total += processed;
    }

    frame_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = percentile(&frame_ms, 0.50);
    let p95 = percentile(&frame_ms, 0.95);
    let p99 = percentile(&frame_ms, 0.99);
    let max = *frame_ms.last().unwrap();

    println!("=== C2.1 messaging throughput ({MSGS_PER_FRAME} msg/frame × {FRAMES} frames) ===");
    println!("processed total: {grand_total} (expected {})", MSGS_PER_FRAME * FRAMES);
    println!("counter (smoke check): {}", counter.get());
    println!("frame send+dispatch time:");
    println!("  p50:  {p50:.3} ms");
    println!("  p95:  {p95:.3} ms");
    println!("  p99:  {p99:.3} ms");
    println!("  max:  {max:.3} ms");
    println!("threshold p99 ≤ 1.5 ms (HR-4 sub-budget)");

    let pass = p99 <= 1.5;
    if pass {
        println!("\nC2.1 PASS — 100k msg/frame dentro de 1.5ms");
    } else {
        println!("\nC2.1 FAIL — p99 {p99:.3}ms excede 1.5ms");
        std::process::exit(1);
    }
}
