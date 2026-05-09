#![forbid(unsafe_code)]
//! C9 extension binary — runs the canonical 50-body lockstep
//! fixture and prints the deterministic hash to stdout.
//!
//! CI runs this on Linux, macOS, and Windows; the determinism job
//! collects the three hashes and compares — must be byte-identical
//! per HR-5 + the M10 plan gate.
//!
//! Output format (stable, parsed by CI shell script):
//! ```text
//! physics-c9 step_count: 120
//! physics-c9 body_count: 51
//! physics-c9 hash: <hex64>
//! ```

use ph2d_physics::PhysicsWorld;

const STEPS: u32 = 120; // 2 seconds @ 60 Hz — long enough for collisions to develop.
const N_DYNAMIC: u32 = 50;

fn main() {
    let mut w = PhysicsWorld::new();
    // Static floor.
    w.add_static_cuboid(0.0, 0.0, 50.0, 0.1);
    // 50 dynamic circles in a 10×5 grid above the floor.
    for i in 0..N_DYNAMIC {
        let row = (i / 10) as f32;
        let col = (i % 10) as f32;
        w.add_dynamic_circle(col * 0.6 - 2.7, 5.0 + row * 0.6, 0.25, 1.0);
    }

    for _ in 0..STEPS {
        w.step();
    }

    let snapshots = w.body_snapshots();
    let hash = w.deterministic_hash();
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();

    // Stable output — CI parses these three lines.
    println!("physics-c9 step_count: {}", w.step_count());
    println!("physics-c9 body_count: {}", snapshots.len());
    println!("physics-c9 hash: {hex}");
}
