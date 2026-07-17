//! **Pricing a shipping target is EXPORT work, and an edit frame must not do it.**
//!
//! The bug this file exists to keep dead (Enio, 2026-07-16): *"1 seg e meio para mudar ganho"*.
//! Clicking Gain on a 3-minute clip cost **1562 ms**, of which **1549 ms (99.2%)** was
//! `editor_publish_platforms` — three `conform`s and three **real encodes** of the whole clip, on
//! the UI thread, to redraw a three-line readout. The DSP the user actually asked for was 25 ms.
//!
//! ## Why the cache never helped
//!
//! Every readout here is keyed on `SampleData::version`, and **an edit moves the version by
//! definition** — that is what an edit *is*. So the cache hit on every frame except the one frame
//! that mattered: the click. The old docstring said so out loud ("a cache hit on all but the frame
//! after the buffer actually changed") without noticing that the frame after the buffer changed is
//! the frame the user is waiting on.
//!
//! ## What the gates below pin
//!
//! Two halves, and **both are needed** — the absence gate alone goes green on a readout that never
//! prices anything at all (`feedback_absence_gate_needs_a_presence_sibling`):
//!
//! 1. **Absence**: an edit frame does not price. Proven by *measurement*, against the real seam's
//!    policy function — not by reading the source.
//! 2. **Presence**: pricing still happens when the section is open, and the number is still right.
//!
//! Plus the sizing cap (a long clip is O(cap), not O(clip)) and the honesty of `disk_exact`.

use ph2d_audio::{AudioFormat, ChannelLayout, SampleData};
use ph2d_audio_encode::{Codec, PLATFORMS, cost};
use std::time::Instant;

/// The product's own frame, read as text.
///
/// The state machine's gates (`audio::editor::pricing::tests`) prove that *given* a `visible` of
/// false nothing is priced. They cannot prove the shell ever passes false — a call site that hard-
/// wired `true` would leave every one of them green while the bug walked back in. This is the
/// arch-gate idiom the line already uses for frame order
/// (`the_z_projection_reads_the_tree_after_the_sync`): assert on the file that ships.
const RENDER_LOOP: &str = include_str!("../src/render_loop/mod.rs");

/// **The pricing call is gated on somebody actually looking.**
///
/// The bug was not that pricing was slow — it was that it happened at all, on an edit frame, for a
/// section that ships **folded**. The gate has two halves and both must be asked: the panel can be
/// visible with the Delivery section collapsed, which is the *default*, so `is_panel_visible` alone
/// would still pay the 1549 ms for rows nobody can see.
#[test]
fn the_edit_frame_only_prices_when_the_delivery_section_is_open() {
    let call = RENDER_LOOP.find("editor_publish_platforms(").expect(
        "`editor_publish_platforms` vanished from the render loop -- if it was renamed, \
                 update this gate, and check the visibility gate came with it",
    );
    let arg_start = call + "editor_publish_platforms(".len();
    let arg_end = RENDER_LOOP[arg_start..]
        .find(')')
        .map(|i| arg_start + i)
        .expect("unbalanced call");
    let arg = RENDER_LOOP[arg_start..arg_end].trim();

    assert_ne!(
        arg, "true",
        "the pricing gate is hard-wired open. Every unit gate on the state machine stays green \
         and the edit frame goes back to paying 1549 ms (ADR-0125) to redraw three strings that \
         are folded away by default."
    );

    // The gate is computed just above the call; both halves have to be in it.
    let window = &RENDER_LOOP[call.saturating_sub(1_200)..arg_end];
    assert!(
        window.contains("is_panel_visible(\"audio_editor\")"),
        "the gate does not ask whether the audio editor panel is even on screen"
    );
    assert!(
        window.contains("AEDIT_SEC_DELIVERY"),
        "the gate does not ask whether the Delivery SECTION is unfolded -- the panel is routinely \
         open with this section collapsed (it ships that way), and that is the case the bug was \
         reported in"
    );
}

/// The smoke's clip, rebuilt here: **noise plus a voiced tone**, mono at 48 kHz.
///
/// The noise is not decoration. A codec's cost is a function of how compressible the audio is, so
/// a fixture of silence (or of a pure tone) encodes to almost nothing almost instantly and would
/// understate every number in this file by an order of magnitude — the measurement would pass
/// while the product stalled. This mirrors `audio::editor::ml_smoke::noisy_clip`, which is the
/// clip Enio actually had open when he timed the click.
fn noisy_clip(secs: f32) -> SampleData {
    let tau = std::f32::consts::TAU;
    let frames = (48_000.0 * secs) as usize;
    let mut state = 0x5EEDu64;
    let mut hiss = move || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 40) as f32 / 8_388_608.0 - 1.0
    };
    SampleData::from_fn(frames, AudioFormat::new(48_000, ChannelLayout::Mono), |i| {
        let t = i as f32 / 48_000.0;
        let env = 0.5 + 0.5 * (tau * 2.0 * t).sin();
        let voice: f32 = (1..=6)
            .map(|k| (tau * 150.0 * k as f32 * t).sin() / k as f32)
            .sum::<f32>()
            * env
            * 0.20;
        (voice + hiss() * 0.20).clamp(-1.0, 1.0)
    })
}

/// What `editor_publish_platforms` does when it decides to price: conform into each target's real
/// format, then really encode it. Reproduced rather than called because the shell's `price` is
/// private to a binary crate — the *shape* is what is being timed, and it is one line.
fn price_every_platform(data: &SampleData) -> usize {
    PLATFORMS
        .iter()
        .filter_map(|p| {
            let conformed = ph2d_audio_edit::conform(data, p.format());
            cost(&conformed, p.codec, p.quality).ok()
        })
        .count()
}

/// **The measurement the whole line turns on.** Prints the breakdown Enio's click paid for.
///
/// Not an assertion: it is the number that says whether the fix worked, and it is here so the next
/// person can re-run the exact thing that was measured rather than trust a figure in a handoff.
/// Run it with:
///
/// ```text
/// cargo test --release -p ph2d-host-desktop --test audio_pricing_is_export_work_not_edit_work -- --ignored --nocapture
/// ```
#[test]
#[ignore = "measurement, not a gate: prints timings, run with --release --nocapture"]
fn measure_what_a_gain_click_used_to_cost() {
    let data = noisy_clip(180.0);
    println!(
        "\nfixture: {} frames, {:.1} MB, mono 48 kHz",
        data.frame_count(),
        (std::mem::size_of_val(data.samples()) as f64) / 1_048_576.0
    );

    for p in PLATFORMS {
        let t = Instant::now();
        let conformed = ph2d_audio_edit::conform(&data, p.format());
        let conform_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let c = cost(&conformed, p.codec, p.quality).expect("priced");
        let cost_ms = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "  {:8} conform {conform_ms:7.1} ms  cost({:?}) {cost_ms:7.1} ms  -> {} B (exact: {})",
            p.name, p.codec, c.disk_bytes, c.disk_exact
        );
    }

    let t = Instant::now();
    price_every_platform(&data);
    let platforms_ms = t.elapsed().as_secs_f64() * 1e3;

    let t = Instant::now();
    let _ = cost(&data, Codec::Wav16, 0.5);
    let delivery_wav_ms = t.elapsed().as_secs_f64() * 1e3;

    let t = Instant::now();
    let _ = cost(&data, Codec::Opus, 0.5);
    let delivery_opus_ms = t.elapsed().as_secs_f64() * 1e3;

    // What the user actually asked for, on the real API the shell calls.
    let mut clip = ph2d_audio_edit::EditClip::new(data.clone());
    let t = Instant::now();
    clip.apply_gain(1.1);
    let gain_ms = t.elapsed().as_secs_f64() * 1e3;

    println!(
        "\n  --- what the UI thread pays for one Gain click ---\n  \
         apply_gain (no selection, O(clip))  {gain_ms:7.1} ms   <- the work the user asked for\n\n  \
         --- what pricing costs, WHEREVER it runs ---\n  \
         price every platform                {platforms_ms:7.1} ms\n  \
         cost(Wav16)  (the default codec)    {delivery_wav_ms:7.1} ms\n  \
         cost(Opus)   (one Prev-arrow click) {delivery_opus_ms:7.1} ms\n\n  \
         BEFORE ADR-0125 every line above landed on the UI thread, on the edit frame:\n  \
         {:.0} ms of click. AFTER, the pricing is gated on the Delivery section being\n  \
         open (it ships FOLDED) and runs on a worker when it is -- so the click is\n  \
         apply_gain and nothing else.\n",
        gain_ms + platforms_ms + delivery_wav_ms
    );
}

/// **The presence sibling: the rows the worker computes are still RIGHT.**
///
/// Everything else in this file is about *not* doing work. A readout that prices nothing is very
/// fast and completely useless, and it would satisfy every absence gate here
/// (`feedback_absence_gate_needs_a_presence_sibling`). So: the three targets still price, they
/// still disagree about RAM — which is the entire reason a platform is a *format* and not a codec
/// — and the capped figure still admits what it is.
#[test]
fn the_priced_rows_still_say_something_true_about_each_target() {
    let data = noisy_clip(1.0);
    let priced: Vec<_> = PLATFORMS
        .iter()
        .map(|p| {
            let conformed = ph2d_audio_edit::conform(&data, p.format());
            (
                p.name,
                cost(&conformed, p.codec, p.quality).expect("priced"),
            )
        })
        .collect();
    assert_eq!(priced.len(), 3, "a target stopped pricing");

    // Mobile conforms the AUDIO (24 kHz mono), so it is the only one that buys memory back. If a
    // refactor ever prices the unconformed master under three codecs, this is what notices.
    let mobile = priced[0].1.ram_bytes;
    let desktop = priced[1].1.ram_bytes;
    assert!(
        mobile * 3 < desktop,
        "Mobile holds {mobile} B against Desktop's {desktop} B -- the rows are being priced from \
         the same clip, and the readout is back to printing the same RAM three times"
    );

    // ...and a long clip's lossy figure is an estimate that says so, on the platform whose codec
    // was the one running uncapped.
    let long = noisy_clip(30.0);
    let desktop_p = PLATFORMS.iter().find(|p| p.name == "Desktop").unwrap();
    assert_eq!(
        desktop_p.codec,
        Codec::Opus,
        "Desktop stopped shipping Opus"
    );
    let c = cost(
        &ph2d_audio_edit::conform(&long, desktop_p.format()),
        desktop_p.codec,
        desktop_p.quality,
    )
    .expect("priced");
    assert!(
        !c.disk_exact,
        "a 30 s Opus figure claims to be exact -- either the cap is gone (and the UI thread is \
         paying for it) or an estimate is being presented as a measurement"
    );
    assert!(c.disk_bytes > 0, "the capped figure is empty");
}

/// **Head to head, on the same buffer, with no cap on either.** The claim this replaces was in the
/// source: *"Opus is bitrate-driven and fast, so the honest number is also the cheap one"*, which
/// is why Opus was exempted from the cap that Vorbis has had all along.
///
/// It is backwards. `unsafe-libopus` is libopus **transpiled by c2rust with none of the C SIMD**
/// (ADR-0116), so "libopus is fast in C" does not transfer to this build. The exemption was
/// reasoning about a codebase we do not link.
#[test]
#[ignore = "measurement, not a gate: prints timings, run with --release --nocapture"]
fn measure_opus_against_vorbis_head_to_head() {
    for secs in [2.0f32, 10.0] {
        let d = noisy_clip(secs);
        let t = Instant::now();
        let ogg = ph2d_audio_encode::encode_ogg(&d, 0.5).expect("ogg").len();
        let ogg_ms = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        let opus = ph2d_audio_encode::encode_opus(&d, 0.5).expect("opus").len();
        let opus_ms = t.elapsed().as_secs_f64() * 1e3;
        println!(
            "  {secs:5.1} s: vorbis {ogg_ms:7.1} ms ({ogg} B) | opus {opus_ms:7.1} ms ({opus} B) \
             -> opus is {:.1}x the vorbis time",
            opus_ms / ogg_ms
        );
    }
}
