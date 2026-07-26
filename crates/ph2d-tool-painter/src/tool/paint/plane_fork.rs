//! **Forking a canvas-sized plane, in parallel** — the hitch at the start of every stroke.
//!
//! A gesture that edits a plane in place needs two things at once: a **frozen** copy of what the stroke
//! started from (`pre`, so the render is idempotent and the knobs can re-render) and a **live** buffer to
//! write into. The session takes the frozen one as an `Arc` clone — refcount, not copy — and the first
//! write then calls `Arc::make_mut`, which sees a second owner and copies the whole plane. That copy is
//! genuinely necessary; there is no way to have a snapshot and a mutable buffer without one.
//!
//! What was not necessary is doing it on one thread. Measured on this machine, forking one `f32` plane:
//!
//! ```text
//!   2048²  (16,8 MB)   serial  0,54 ms    parallel  0,32 ms    1,7×
//!   4096²  (67,1 MB)   serial 10,88 ms    parallel  3,34 ms    3,3×
//! ```
//!
//! ⚠️ Note the shape of that table: **four times the data costs twenty times the time.** The cost is not
//! bandwidth, it is the fresh allocation — 67 MB of pages faulted in on first touch, one at a time. That
//! is also why the parallel version wins by more at 4K than at 2K: the faults spread across threads too.
//!
//! It is the whole of the measured set-up hitch. The sculpt's session open cost 12,9–15,1 ms at 4096²
//! against 5,7–6,0 at 2048², and one plane fork is 10,88 ms of it — the only cost in the module that grows
//! when the artist enlarges the canvas (every kernel here is bounded by the brush footprint and is flat in
//! canvas size).
//!
//! Three tools pay it, which is why this is a shared door rather than a sculpt detail: the **sculpt**
//! (`heights`, plus `covers`/`mats`/`canvas_rgba` when Inflate moves matter), the **Reshape** warp and the
//! **Smear** (both re-render the canvas and the three relief planes from a frozen session baseline).

use rayon::prelude::*;
use std::sync::Arc;

/// Below this many elements the fork is sub-millisecond and the choice cannot matter (measured: 0,54 ms
/// for 4,2 M elements), so the threshold only exists to keep rayon's fork overhead away from small
/// canvases. Sibling of `sculpt_close::PAR_MIN`, which exists for the same reason and was measured to make
/// the Inflate *slower* below it.
const PAR_MIN: usize = 1 << 20;

/// `Arc::make_mut` for a canvas-sized plane, with the copy parallelised.
///
/// Semantically **identical** to `Arc::make_mut` — same value, same aliasing rules, and the returned
/// buffer is uniquely owned either way. The only difference is which threads do the copying, so this is
/// byte-identical by construction: it is a copy, and a copy has one right answer.
///
/// When there is no second owner it delegates untouched — no allocation, no threads, no cost. When there
/// is one but the plane is small it also delegates, because rayon's fork would outweigh the memcpy.
pub(super) fn fork_par<T>(arc: &mut Arc<Vec<T>>) -> &mut Vec<T>
where
    T: Copy + Send + Sync,
{
    // `get_mut` is the question `make_mut` asks anyway; asking it first is what lets us choose HOW to
    // copy rather than only whether to. (Phrased as `is_none` so the borrow ends before the copy.)
    if Arc::get_mut(arc).is_none() && arc.len() >= PAR_MIN {
        let fresh: Vec<T> = arc.par_iter().copied().collect();
        *arc = Arc::new(fresh);
    }
    // Now either uniquely owned (we just replaced it) or small/unshared — so this never copies twice.
    Arc::make_mut(arc)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A fork is a copy, and a copy has one right answer.** The parallel path must produce exactly what
    /// `Arc::make_mut` produces — value-identical, and uniquely owned afterwards so the caller may write.
    ///
    /// Run over a length that clears `PAR_MIN` (so the parallel path actually executes — the trap the
    /// ADR-0120 lesson names: an optimisation nobody exercises is green code that never runs) and one that
    /// does not.
    #[test]
    fn a_parallel_fork_is_byte_identical_to_the_serial_one() {
        for n in [PAR_MIN + 1_000, 64] {
            let src: Vec<f32> = (0..n).map(|i| (i as f32) * 0.25 - 3.0).collect();

            let mut a = Arc::new(src.clone());
            let keep_a = Arc::clone(&a); // force the second-owner path
            let forked = fork_par(&mut a).clone();

            let mut b = Arc::new(src.clone());
            let keep_b = Arc::clone(&b);
            let expected = Arc::make_mut(&mut b).clone();

            assert_eq!(forked, expected, "n = {n}");
            assert_eq!(forked, src, "n = {n}: the fork changed the contents");
            // The originals are untouched — the whole point of freezing one.
            assert_eq!(*keep_a, src);
            assert_eq!(*keep_b, src);
        }
    }

    /// **With no second owner it does not allocate at all.** The common case inside a stroke is that the
    /// plane was already forked by an earlier dab; paying a copy per dab instead of per stroke is the
    /// regression this guards.
    #[test]
    fn an_unshared_plane_is_not_copied() {
        let mut a: Arc<Vec<f32>> = Arc::new(vec![1.0; PAR_MIN + 1_000]);
        let before = a.as_ptr();
        let got = fork_par(&mut a);
        assert_eq!(got.as_ptr(), before, "an unshared plane was copied anyway");
    }

    /// **The fast path has to be proven by the clock, because nothing else can see it.**
    ///
    /// The two gates above cannot tell the branches apart, and that is not an oversight — it is the
    /// point: a fork is a copy, so the parallel path is *semantically identical* by construction. There
    /// is no value, no pointer and no refcount that differs (`Arc::make_mut` also leaves a uniquely
    /// owned buffer at a fresh address). A behavioural gate here would be the serial path measured
    /// against itself and green forever — the trap ADR-0120 documented and ADR-0124 then hit a second
    /// time, hiding in its own undo oracle.
    ///
    /// So the claim is timed, and asserted as a **RATIO** rather than a wall-clock bar: `ci-test`
    /// compiles at `opt-level=1` and this machine is documented as drifting ~3× across a session, so an
    /// absolute millisecond bar would be measuring the profile and the weather. The ratio survives both.
    ///
    /// Measured at 4096² (67 MB): serial 10,88 ms, parallel 3,34 ms — 3,3×. The bar is set well under
    /// that so a loaded machine cannot flake it, while a fork that silently stopped being parallel
    /// (delegating straight to `Arc::make_mut`) lands at 1,0× and fails.
    #[test]
    #[ignore = "perf measurement — run with --release --ignored"]
    fn the_parallel_fork_is_actually_faster_than_the_serial_one() {
        use std::time::Instant;
        const N: usize = 4096 * 4096;
        let src: Arc<Vec<f32>> = Arc::new(vec![0.5; N]);
        let best = |mut f: Box<dyn FnMut() -> f64>| (0..3).map(|_| f()).fold(f64::MAX, f64::min);

        let s = Arc::clone(&src);
        let serial = best(Box::new(move || {
            let mut a = Arc::clone(&s);
            let _keep = Arc::clone(&s);
            let t0 = Instant::now();
            let m = Arc::make_mut(&mut a);
            std::hint::black_box(&m[0]);
            t0.elapsed().as_secs_f64() * 1000.0
        }));
        let p = Arc::clone(&src);
        let parallel = best(Box::new(move || {
            let mut a = Arc::clone(&p);
            let _keep = Arc::clone(&p);
            let t0 = Instant::now();
            let m = fork_par(&mut a);
            std::hint::black_box(&m[0]);
            t0.elapsed().as_secs_f64() * 1000.0
        }));
        eprintln!(
            "[fork] 4096² plane: serial {serial:.2} ms · parallel {parallel:.2} ms · \
             {:.1}×",
            serial / parallel
        );
        assert!(
            serial / parallel > 1.5,
            "the parallel fork bought {:.1}× over `Arc::make_mut` (serial {serial:.2} ms, parallel \
             {parallel:.2} ms). Below 1.5× the fast path is not running — check that `PAR_MIN` still \
             sits under a canvas-sized plane",
            serial / parallel
        );
    }

    /// **A porta do pen-down** — que o depósito de PIGMENTO atravesse esta função, e não o `make_mut` cru.
    ///
    /// ⚠️ Este gate é arquitetural porque o defeito é **invisível ao comportamento**: as duas rotas produzem
    /// os mesmos bytes (é o que o gate acima prova), então trocar uma pela outra deixa a suíte inteira verde e
    /// custa **três vezes o tempo** no gesto que o artista mais sente. Medido no pen-down a 4096², pincel
    /// digital: **10,3 ms com o `make_mut` cru contra 3,6 ms por aqui** (`measure_impasto_cost::
    /// the_first_stroke_latency`); o pen-down do impasto, 18,6 -> 12,2.
    ///
    /// O escopo é o `stamp_cache` de propósito: é ele que escreve o canvas no **pen-down**, e o pen-down é o
    /// único sítio onde o `Arc` do canvas tem um segundo dono garantido (o `stroke_undo` que o `paint_begin`
    /// acabou de tirar) ⇒ o primeiro `make_mut` do traço **sempre** copia a tela inteira.
    #[test]
    fn the_pigment_deposit_forks_the_canvas_in_parallel() {
        let src = include_str!("stamp_cache.rs");
        // Controle positivo: o alvo tem de EXISTIR, senão o gate passa por não achar nada (a falha que o
        // `the_shape_slot_goes_through_the_shape_door` do Flow pegou em si mesmo).
        let through = src.matches("fork_par(&mut self.canvas_rgba)").count();
        assert!(
            through >= 5,
            "controle: o stamp_cache tem de escrever o canvas pela porta paralela ({through} sitios)"
        );
        let raw = src.matches("Arc::make_mut(&mut self.canvas_rgba)").count();
        assert_eq!(
            raw, 0,
            "o deposito de pigmento nao pode forkar o canvas SERIALMENTE: {raw} sitio(s) com `make_mut` cru \
             (as duas rotas dao os mesmos bytes, entao isto nao acende em teste de comportamento nenhum \
             -- custa 3x o tempo do pen-down e passa despercebido)"
        );
    }
}
