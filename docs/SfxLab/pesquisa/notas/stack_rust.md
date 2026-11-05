# Rust and native stack for a real-time, node-graph SFX synthesis module (as of 2026-11-05)

Method: release dates, versions and declared licences come from the crates.io API, queried 2026-11-05. Repository activity comes from the GitHub API, same day. Where noted, licences and code were read from downloaded crate tarballs or raw repository files. Nothing was compiled or benchmarked in this pass, and Faust, Csound and Cmajor are not installed on this machine, so every real-time and performance claim below is the project's own or a paper's, not a measurement of ours.

## Rust synthesis and DSP libraries: licence, last release, features, real-time safety, graph support, determinism, performance

### Takeaway
FunDSP is the only maintained, permissive crate here with a real graph, lock-free hot-swap and anti-aliasing tools. The other graph-capable crates are copyleft (HexoDSP, synfx-dsp), dormant (Knyst, Glicol) or not synth libraries (Firewheel has no instruments). No cross-library performance benchmark was found.

### Cited Findings

Inventory (latest stable release; crates.io declared licence):

| Crate | Version / date | Licence | Graph? | State and suitability |
|---|---|---|---|---|
| fundsp | 0.23.0 / 2026-01-07 | MIT OR Apache-2.0 | Yes: `Net`, combinators, `Sequencer` | Last repo push 2026-03-03, 1,230 stars. Best fit |
| dasp | 0.11.0 / 2020-05-29 | MIT OR Apache-2.0 | `dasp_graph` not inspected | Last repo push 2026-01-12, no release for 6 years. Primitives only |
| knyst | 0.5.1 / 2024-11-25 | MIT OR Apache-2.0 | Yes, dynamic | Last push 2024-11-25, 44 stars. Dormant |
| glicol / glicol_synth | 0.13.5 / 2024-04-23 | crates.io says "non-standard"; repo `LICENSE` is MIT | Yes, live-coding DSL | Last push 2025-04-06. Language first, not an engine |
| rust-synth | 0.23.0 / 2026-04-23 | MIT | Via FunDSP | A terminal application, not a library. 230 downloads |
| synfx-dsp | 0.5.6 / 2024-01-04 | **GPL-3.0-or-later** | No | Excluded by licence |
| hexodsp | 0.2.2 / 2024-01-04 | **GPL-3.0-or-later** | Yes, modular | Excluded by licence |
| twang | 0.9.0 / 2022-10-23 | Apache-2.0 OR BSL-1.0 OR MIT | None seen in README | Stale |
| oxisynth | 0.1.0 / 2025-05-25 | **LGPL-2.1** | No | SoundFont player, off target |
| nih_plug | Not on crates.io | ISC, with **GPLv3 VST3 bindings** | No, plugin framework | Maintenance mode |
| rubberband (bindings) | 0.1.5 / 2026-09-18 | **GPL-2.0-or-later** | No | Copyleft. C++ library is also GPL |
| rubato | 5.0.1 / 2026-10-01 | MIT OR Apache-2.0 | No | Resampler; permissive |
| realfft | 3.5.1 / 2026-10-26 | MIT | No | Already in the PH2D workspace |
| rustfft | 6.4.1 / 2025-09-18 | MIT OR Apache-2.0 | No | Transitive via realfft |

Sources for the table: [fundsp](https://crates.io/crates/fundsp), [dasp](https://crates.io/crates/dasp), [knyst](https://crates.io/crates/knyst), [glicol](https://crates.io/crates/glicol), [Glicol LICENSE](https://github.com/chaosprint/glicol/blob/main/LICENSE), [rust-synth](https://crates.io/crates/rust-synth), [synfx-dsp](https://crates.io/crates/synfx-dsp), [hexodsp](https://crates.io/crates/hexodsp), [twang](https://crates.io/crates/twang), [oxisynth](https://crates.io/crates/oxisynth), [rubberband](https://crates.io/crates/rubberband), [rubato](https://crates.io/crates/rubato), [realfft](https://crates.io/crates/realfft), [rustfft](https://crates.io/crates/rustfft).

FunDSP detail:
- `Net` splits into a frontend and a backend: "The frontend handles changes to the network, while the real-time safe backend renders audio." Nodes hot-swap with `replace`, and `crossfade` exists "to avoid clicks" — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- Buffers are fixed at 64 samples (`MAX_BUFFER_SIZE`), and use `f32x8` from the `wide` crate. All samples are `f32`, and `prelude64` uses 64-bit internal state — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- `Sequencer` can "start and stop rendering nodes with sample accuracy". The README also says "the number of commits in flight is limited to 256". Cycles built through `Net` raise an error — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- `allocate` preallocates and "should be called last before entering a real-time context". Wavetable oscillators allocate global wavetables — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- Anti-aliasing tiers: wavetable and DSF are high quality (DSF "somewhat expensive"), PolyBLEP is "a fast approximation with fair quality". `oversample` runs a node at 2x — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- Generator phases are seeded from network structure and node location, so identical networks sound identical alone but differ when combined. Noise takes an explicit `seed` — [FunDSP README](https://github.com/SamiPerttu/fundsp).
- `src/realnet.rs` holds a `thingbuf` channel, and sends old versions back to the frontend "for deallocation". `src/lib.rs` implements `sin`, `exp`, `cos`, `tan` for `f32`/`f64` through the `libm` crate, and there are zero `mul_add` calls in `src/` — [fundsp 0.23.0 crate artefact](https://static.crates.io/crates/fundsp/fundsp-0.23.0.crate).
- `src/oversample.rs` carries a 43-tap table named `HALFBAND_MIN` — [fundsp 0.23.0 crate artefact](https://static.crates.io/crates/fundsp/fundsp-0.23.0.crate).

Others:
- Knyst describes itself as a "real time audio synthesis framework"; its `process` "must not allocate or wait", and its README lists "sample accurate node and parameter change scheduling" — [knyst 0.5.1 crate artefact](https://static.crates.io/crates/knyst/knyst-0.5.1.crate).
- Glicol: "users can send message to each node in real-time for interaction" — [glicol_synth 0.13.5 crate artefact](https://static.crates.io/crates/glicol_synth/glicol_synth-0.13.5.crate).
- rust-synth is a "Terminal modular ambient synthesizer — FunDSP + Ratatui" — [rust-synth 0.23.0 crate artefact](https://static.crates.io/crates/rust-synth/rust-synth-0.23.0.crate).
- NIH-plug: "the plugin framework is currently in maintenance mode", with a community fork pointed to at codeberg.org/BillyDM/nih-plug. The framework is ISC, but "the VST3 bindings used by `nih_export_vst3!()` are licensed under the GPLv3 license". Last push 2026-09-03 — [NIH-plug README](https://github.com/robbert-vdh/nih-plug).
- `assert_no_alloc` 1.1.2 (2021-08-03, BSD-1-Clause) panics on audio-thread allocation; Kira exposes it as a feature "mainly for Kira's own developers" — [crates.io](https://crates.io/crates/assert_no_alloc); [Kira docs](https://docs.rs/kira/latest/kira/).
- Rubber Band's C++ library is GPL-2.0 on GitHub, and 4.0.0 is packaged here as GPL-2.0-or-later (`pacman -Qi`) — [rubberband repo](https://github.com/breakfastquay/rubberband).

### Inferences
- FunDSP is the baseline. Its seed rule has a trap for an editable graph: a phase seeded from "location" can change when an unrelated node is added, so a saved SFX could change sound after an edit. PH2D should set every seed explicitly from a stable node ID. This needs confirming by a test.
- FunDSP's fixed 64-sample block makes a commit land on a block boundary (about 1.3 ms at 48 kHz), not on a sample, unless it goes through `Sequencer`.
- LGPL-2.1 oxisynth links statically in a Rust binary, which makes the relinking obligation awkward. It is off target anyway.
- GPL crates (HexoDSP, synfx-dsp, rubberband) may only be used as a separately run oracle, under CLAUDE.md §0.9, never as a dependency, and their source should not be read.
- rubato plus a permissive pitch or time algorithm written in-house covers most SFX needs without Rubber Band.

### Gaps
- No independent, like-for-like benchmark across these libraries was found. A PH2D measurement is needed.
- FunDSP's `Float for f32x8` (`src/lib.rs:596`) was not read, so whether its SIMD math is deterministic is unknown.
- `dasp_graph`'s state, and `rust-synth`, `twang` real-time behaviour, were not audited line by line. HexoDSP's README was not usable for real-time claims: a grep of the crate's README found no allocation or lock-free statements.
- No audit proved that any "no allocation" claim holds in practice.

## Compiled DSP languages: Faust, Cmajor, SOUL, Csound, and Faust's physical models

### Takeaway
Faust's generated code does **not** carry the compiler's licence, and the compiler is now LGPL, not GPL. Physical models in `physmodels.lib` can be compiled into proprietary code by an explicit written exception. Faust fits as a build-time source or an oracle, not as the engine for a user-edited graph. Cmajor is GPLv3/commercial, and SOUL has been dormant since 2022.

### Cited Findings

Faust licence, from the artefacts:
- The compiler's `COPYING.txt` (master-dev, version 2.92.2) says LGPL-2.1 or later — [Faust COPYING.txt](https://github.com/grame-cncm/faust/blob/master-dev/COPYING.txt).
- Latest tagged release is 2.88.0 (2026-09-09), and the repo was pushed 2026-11-05 — [GitHub releases](https://github.com/grame-cncm/faust/releases).
- FAQ, verbatim: "The LGPL license of the compiler _doesn't_ apply to the code generated by the compiler." "The license of the code generated by the Faust compiler depends only on the licenses of the input files." "You can freely use Faust to develop proprietary software." "You should therefore check the licenses of the Faust libraries used and the architecture files." — [Faust FAQ](https://faustdoc.grame.fr/manual/faq/).
- `physmodels.lib` (version 1.2.14) exception, verbatim: "EXCEPTION TO THE LGPL LICENSE : As a special exception, you may create a larger FAUST program which directly or indirectly imports this library file and still distribute the compiled code generated by the FAUST compiler, or a modified version of this compiled code, under your own copyright and license. This EXCEPTION TO THE LGPL LICENSE explicitly grants you the right to freely choose the license for the resulting compiled code. In particular the resulting compiled code has no obligation to be LGPL or GPL. For example you are free to choose a commercial or closed source license or any other license if you decide so." — [physmodels.lib](https://github.com/grame-cncm/faustlibraries/blob/master/physmodels.lib).
- The same exception text appears in `maths.lib`, `oscillators.lib`, `noises.lib` and part of `reverbs.lib` — [faustlibraries](https://github.com/grame-cncm/faustlibraries).
- Licences vary per function in some libraries. `filters.lib`: "Each function in this library has its own license", many `LicenseRef-STK-4.3`. `instruments.lib` is "Synthesis Tool Kit 4.3 (MIT style license)". `reverbs.lib` mentions a CC-BY-4.0 term with the note "TODO: if/when Romain agrees!" — [filters.lib](https://github.com/grame-cncm/faustlibraries/blob/master/filters.lib); [instruments.lib](https://github.com/grame-cncm/faustlibraries/blob/master/instruments.lib); [reverbs.lib](https://github.com/grame-cncm/faustlibraries/blob/master/reverbs.lib).
- Architecture files have a separate exception: "you may create a larger work that contains this FAUST architecture section and distribute that work under terms of your choice, so long as this FAUST architecture section is not modified." Some are plain GPL-3+ with no exception, for example `architecture/csound.cpp` — [Debian copyright for Faust 2.79.3](https://sources.debian.org/src/faust/2.79.3%2Bds-2/debian/copyright).
- **Source conflict:** Debian's file for 2.79.3 lists the default `*` as GPL-2+ and `oscillators.lib` as plain LGPL, whereas current master's `COPYING.txt` says LGPL and `oscillators.lib` contains the exception — [Debian copyright](https://sources.debian.org/src/faust/2.79.3%2Bds-2/debian/copyright); [oscillators.lib](https://github.com/grame-cncm/faustlibraries/blob/master/oscillators.lib). Current upstream artefacts should govern, pinned to a commit.
- An older Faust doc noted no indication of this licensing situation in the compiler's output — [Faust libraries copyright.md, Debian mirror](https://sources.debian.org/src/faust/2.30.5~ds0-2/libraries/doc/docs/copyright.md).

Faust's Rust backend:
- `rust_code_container.cpp` rejects `-quad`, OpenCL, CUDA, OpenMP and the scheduler, and has scalar and vector containers (`RustVectorCodeContainer`) — [rust_code_container.cpp](https://github.com/grame-cncm/faust/blob/master-dev/compiler/generator/rust/rust_code_container.cpp).
- Its math table maps C functions onto `F32::abs`, `F32::acos`, `F32::asin`, `F32::atan`, `F32::atan2` and so on — [rust_instructions.hh](https://github.com/grame-cncm/faust/blob/master-dev/compiler/generator/rust/rust_instructions.hh).
- `faust2jackrust` and `faust2portaudiorust` exist for Rust binaries — [Faust tools doc](https://faustdoc.grame.fr/manual/tools/).
- A Faust mailing-list reply said the backend began in 2017, and that generated code was far from optimal and needed work to match C++. It is old and undated here — [faudiostream-users](https://mail-archive.com/faudiostream-users@lists.sourceforge.net/msg02803.html).
- `faust-build` and `faust-types` are at 0.2.1 (2024-11-20), MIT OR Apache-2.0, low use, no repository field — [faust-build](https://crates.io/crates/faust-build).

Faust physical models:
- `physmodels.lib` holds ready instruments including `djembe`, `marimba` and `strikeModel`, built from `modeFilter` banks; `modeInterpRes` caps at `min(40,nModes)` modes — [physmodels.lib](https://github.com/grame-cncm/faustlibraries/blob/master/physmodels.lib).
- `mesh2faust` uses a Vega subtree under BSD-3-Clause — [Debian copyright](https://sources.debian.org/src/faust/2.79.3%2Bds-2/debian/copyright).

Cmajor, SOUL, Csound:
- Cmajor: "published under a dual GPLv3 (or later)/Commercial license", and "If you want to embed the Cmajor engine in a non-GPL product, we offer a range of commercial licensing options, from free personal licenses to enterprise-level plans". "When you use the Cmajor tools to generate C++ from your own Cmajor code, then that generated C++ is yours to use however you wish." Release 1.0.3209, 2026-09-24 — [Cmajor LICENSE.md](https://github.com/cmajor-lang/cmajor/blob/main/LICENSE.md); [releases](https://github.com/cmajor-lang/cmajor/releases).
- SOUL: last push 2022-07-21; GitHub cannot identify its licence (`NOASSERTION`) — [soul-lang/SOUL](https://github.com/soul-lang/SOUL).
- Csound: LGPL-2.1, release 7.0.0 on 2026-10-21 — [csound COPYING](https://github.com/csound/csound/blob/develop/COPYING); [releases](https://github.com/csound/csound/releases).

### Inferences
- The exception covers **compiled code generated by the Faust compiler**. A hand translation of a `.lib` into Rust is not obviously covered and could be a derivative work under LGPL. Safer routes are committing compiler output, or independently implementing from published physics and measured parameters.
- A Faust `.dsp` compiles to a fixed topology, so it cannot be the engine of a graph edited at runtime. Embedding `libfaust` would add a C++/LLVM JIT, in tension with ADR-0075 (monorepo Rust, no runtime plugins). Hence: Faust at build time for fixed "kernels", generated Rust committed to a drop-crate, with a licence header per kernel.
- Faust suits CLAUDE.md §0.9: run it as an oracle over our inputs, store the output as a fixture, and compare step by step.
- Generated Rust calling `F32::...` methods reaches the non-deterministic std functions (see the determinism question). It would need its math redirected to `ph2d-det-math` to pass PH2D's gate. This is seen for the `abs`/`acos`/`asin`/`atan`/`atan2` entries; `sin`, `exp` are presumed the same.
- Cmajor adds a C++ step and offers no Rust output, so it is a poor fit. Its engine would need a paid or GPL route.
- Csound's LGPL C library is a poor fit for a static Rust monorepo, and useful at most as an external oracle.

### Gaps
- The Faust exception carries no explicit attribution rule for output, so counsel might want a notice anyway; this is not legal advice.
- Rust backend quality in 2026 was not measured (Faust is not installed). No recent benchmark was found.
- Commercial pricing of Cmajor was not read. SOUL's own licence file was not read.

## Runtime in games: playback engines, WebAudio-style graphs, and runtime synthesis versus baking

### Takeaway
cpal is already PH2D's device layer and is healthy. Firewheel is the closest thing to a game-oriented graph engine, Kira is the best-kept mixer but not an arbitrary graph, and web-audio-api is the spec-shaped option. PH2D already has its own real-time mixer, so the module should feed that mixer. Baking should be the default in games, with live synthesis for small parameter-driven voices.

### Cited Findings

| Engine | Version / date | Licence | Shape |
|---|---|---|---|
| cpal | 0.18.2 / 2026-08-16 | Apache-2.0 | Device I/O, optional `realtime` priority feature |
| rodio | 0.23.0 / 2026-10-11 | MIT OR Apache-2.0 | Playback; internals not inspected |
| kira | 0.12.6 / 2026-10-28 | MIT OR Apache-2.0 | Game mixer tracks, effects, clocks |
| oddio | 0.7.4 / 2023-10-15 | MIT OR Apache-2.0 | Sans-I/O, 3D; last push 2023-10-15 |
| firewheel | 0.14.0 / 2026-09-05 | MIT OR Apache-2.0 | DAG engine; last push 2026-11-01 |
| web-audio-api | 1.7.0 / 2026-08-08 | MIT | W3C Web Audio API in Rust |

Sources: [cpal](https://crates.io/crates/cpal), [rodio](https://crates.io/crates/rodio), [kira](https://crates.io/crates/kira), [oddio](https://crates.io/crates/oddio), [firewheel](https://crates.io/crates/firewheel), [web-audio-api](https://crates.io/crates/web-audio-api).

- Firewheel "supports any directed, acyclic graph", and "Properly respects realtime constraints (no mutexes!)". It offers silence optimisations, a custom node API, a data-driven parameter API "friendly to ECS's", and `no_std` — [Firewheel README](https://github.com/BillyDM/firewheel).
- Firewheel's non-goals include being a DAW engine and "Built-in synthesizer instruments" — [Firewheel design doc](https://github.com/BillyDM/firewheel/blob/main/DESIGN_DOC.md).
- Firewheel "is currently planned to be upstreamed into the Bevy game engine", and "development has shifted to focus more on the needs of Bevy", though the core stays usable without Bevy beyond `bevy_platform` — [Firewheel README](https://github.com/BillyDM/firewheel). A design review noted significant work remains before upstreaming; the review is undated, and no official Bevy announcement was found — [HackMD review](https://hackmd.io/@bevy/firewheel_and_seedling).
- Kira has mixer tracks and sub-tracks, effects (compressor, delay, distortion, EQ, filter, reverb and others), modulators, tweens, clocks and a custom `Sound` trait. The docs do not describe arbitrary routing — [Kira docs](https://docs.rs/kira/latest/kira/); [kira 0.12.6 crate artefact](https://static.crates.io/crates/kira/kira-0.12.6.crate).
- Oddio is "Sans I/O" and "efficient and wait-free" — [oddio crate artefact](https://static.crates.io/crates/oddio/oddio-0.7.4.crate).
- web-audio-api aims to "stick to the official W3C spec as close as possible", with cpal default and cubeb optional — [web-audio-api crate artefact](https://static.crates.io/crates/web-audio-api/web-audio-api-1.7.0.crate).
- PH2D: `ph2d-audio` is the RT mixer and "must stay allocation-free and predictable", under HR-3 (no alloc) and HR-5 (no transcendentals); `ph2d-audio-spectral` is the only crate allowed an FFT dependency (ADR-0122) — `/home/enio/Documentos/Projetos/PH2D/crates/ph2d-audio-spectral/src/lib.rs`.

Bake versus runtime:
- A thesis reports that procedural synthesis "saves a considerable amount of memory space" at higher CPU load, and that it "will not replace sample-based sound design in the foreseeable future, but will certainly expand and enrich it" — [Stolz, HdM Stuttgart](https://curdt.home.hdm-stuttgart.de/PDF/Stolz.pdf).
- A 2024 mobile-game thesis using MetaSounds reported low CPU use and storage efficiency, and did not cover RAM in depth — [Theseus thesis](https://www.theseus.fi/handle/10024/856266).
- One MetaSounds plugin offers both modes: run graphs at runtime for dynamic control, or "render them to wave files to reduce CPU load" — [Fab listing](https://www.fab.com/listings/738ae468-a616-4051-ae03-853b53bc2de4?lang=fr).
- Extra layers add CPU overhead that matters most on mobile — [Splice](https://splice.com/blog/?p=16269).

### Inferences
- Do not add a second playback engine. The synthesis graph should appear as one more source for `ph2d-audio`, and Firewheel's schedule-compile model is a design to study more than a dependency to take.
- Firewheel is a real alternative if PH2D wanted to replace its mixer, but its roadmap now follows Bevy's, a dependency risk for a non-Bevy project.
- The game-export pipeline should be per-asset: **baked** (default, deterministic, cheap), **baked with variations** (N seeds rendered at build time), **live** (only when a parameter is driven by gameplay). The same graph should serve all three, so "bake" is just the offline render.

### Gaps
- No primary per-voice CPU figures for MetaSounds or comparable engines were found. Epic's docs were not consulted.
- rodio's internals and `web-audio-api`'s offline context were not inspected: the README grep found no "offline" mention.

## Graph architecture: hard real-time execution, updates, anti-aliasing, bit-identical rendering, SIMD

### Takeaway
The established pattern is: edit on a control thread, compile to an immutable schedule, send it over a wait-free queue, swap at a block boundary with a crossfade, and return the old one for deallocation off the audio thread. Bit-identical rendering is achievable in Rust only with fixed-math discipline, because std transcendentals, runtime-dispatched SIMD and FFT planners all break it. PH2D already has the law and the gate for the libm half.

### Cited Findings

Real-time rules, verbatim from Bencina's list: "Don't allocate or deallocate memory"; "Don't lock a mutex"; "Don't read or write to the filesystem or otherwise perform i/o."; "Don't call OS functions that may block waiting for something"; "Don't execute any code that has unpredictable or poor worst-case timing behavior"; "Don't call _any_ code that does or may do any of the above"; "Don't call any code that you don't trust to follow these rules". It recommends "lock-free FIFO queues to communicate commands and data between real-time and non-real-time contexts" — [Ross Bencina, mirror of rossbencina.com, updated 13 August 2011](https://hn.jakechampion.name/reader/http://www.rossbencina.com/code/real-time-audio-programming-101-time-waits-for-nothing). The original host refused connection during this research.

Compile and swap in practice:
- Firewheel: "the audio graph is 'compiled' into a schedule and sent to the executor over the message channel". `update` runs once a frame, and "Flushing events as a group like this ensures that events that are expected to happen on the same process cycle don't happen on different process cycles"; a failed compile returns an error and no new schedule is created — [Firewheel design doc](https://github.com/BillyDM/firewheel/blob/main/DESIGN_DOC.md).
- Firewheel events carry `EventDelay::DelayUntilSeconds`, on an `f64` clock read from the OS audio API where possible — [Firewheel design doc](https://github.com/BillyDM/firewheel/blob/main/DESIGN_DOC.md).
- FunDSP's backend receives new versions and returns old ones through `thingbuf`, with commits capped at 256 in flight — [fundsp artefact](https://static.crates.io/crates/fundsp/fundsp-0.23.0.crate); [README](https://github.com/SamiPerttu/fundsp).
- Candidate primitives: `rtrb` 0.4.1 (2026-10-25) and `basedrop` 0.1.3 (2025-10-29), both permissive — [rtrb](https://crates.io/crates/rtrb); [basedrop](https://crates.io/crates/basedrop).
- NIH-plug makes sample-accurate automation opt-in via `Plugin::SAMPLE_ACCURATE_AUTOMATION` — [NIH-plug README](https://github.com/robbert-vdh/nih-plug).

Anti-aliasing:
- A DAFx 2017 paper on a polygonal oscillator reports that four-point polyBLAMP gave about 20 dB SNR improvement, corresponding to 2–4× oversampling at 25× lower computational complexity. This is specific to that oscillator — [DAFx paper archive](https://dafx.de/paper-archive/details/4nfXx-67F78HSIzBoSbRgA).
- Users report that PolyBLEP is quasi-bandlimited and aliases near Nyquist, and that 2× oversampling mostly clears it. This is forum evidence — [KVR thread](https://www.kvraudio.com/forum/viewtopic.php?p=5269992).
- Waveshaper antiderivative anti-aliasing was introduced by Parker, Zavalishin and Le Bivic, DAFx-2016 — [DAFx archive](https://dafx.de/paper-archive/details/vem_XXF5qBbfiWOH2RVVAA). Each new nonlinearity needs its antiderivative derived — [DAFx 2025 paper 30](https://dafx25.dii.univpm.it/wp-content/uploads/2025/08/DAFx25_paper_30.pdf). Second-order forms add latency and slight high-frequency rolloff, per one library's docs — [math-dsp adaa.rs](https://docs.rs/math-dsp/0.5.13/src/math_audio_dsp/adaa.rs.html).

Determinism:
- Rust's std on `sin`, `exp`, `powf`: "The precision of this function is non-deterministic. This means it varies by platform, Rust version, and can even differ within the same execution from one invocation to the next." — [f32 docs](https://doc.rust-lang.org/std/primitive.f32.html).
- `sqrt` and `mul_add` are "guaranteed to be the rounded infinite-precision result", "guaranteed not to change" — [f32 docs](https://doc.rust-lang.org/std/primitive.f32.html). PH2D's own crate reaches the same conclusion — `/home/enio/Documentos/Projetos/PH2D/crates/ph2d-det-math/src/lib.rs`. A third-party crate instead avoids `mul_add` on the grounds that it would break its guarantee, which is a conflict with the std docs unless it refers to unfused fallbacks — [alice-det-math](https://docs.rs/alice-det-math).
- NaN bit patterns: "Rust does not currently guarantee that the bit patterns of NaN are preserved over arithmetic operations, and they are not guaranteed to be portable or even fully deterministic!" — [f32 docs](https://doc.rust-lang.org/std/primitive.f32.html).
- `wide`'s `f32x8::mul_add` is fused with `avx`+`fma`, plain `(self * a) + b` with `avx` alone, and per-lane `mul_add` otherwise, chosen at compile time — [wide 1.7.1 artefact](https://static.crates.io/crates/wide/wide-1.7.1.crate).
- `pulp` dispatches "to equivalent vectorized versions based on the features detected at runtime" — [pulp 0.22.3 README](https://docs.rs/pulp).
- RustFFT's planner "will automatically switch to faster AVX-accelerated algorithms" given `avx` and `fma`, likewise SSE4.1 and NEON. `FftPlannerScalar` exists. `RealFftPlanner::new` builds on the auto-detecting `FftPlanner`, though a constructor taking a given planner exists — [rustfft README](https://github.com/ejmahler/RustFFT); [rustfft 6.4.1 `plan.rs`](https://docs.rs/rustfft/6.4.1/rustfft/); [realfft 3.5.1 `lib.rs`](https://docs.rs/realfft/3.5.1/realfft/).
- Firewheel names its denormal flush `unsafe_flush_denormals_to_zero` — [firewheel 0.14.0 artefact](https://static.crates.io/crates/firewheel/firewheel-0.14.0.crate).
- PH2D already has `ph2d-det-math`, written after Bug #58 because "o byte do motor mudava com a glibc instalada", plus the gate `o_motor_nao_chama_a_libm_do_sistema`, and pins `libm = "=0.2.16"` across the workspace — `/home/enio/Documentos/Projetos/PH2D/crates/ph2d-det-math/src/lib.rs`; `/home/enio/Documentos/Projetos/PH2D/crates/ph2d-det-math/Cargo.toml`. FunDSP declares `libm` 0.2.15, so it can resolve to the same pinned version — [fundsp artefact](https://static.crates.io/crates/fundsp/fundsp-0.23.0.crate).

### Inferences
Recommended design, for the report writer to treat as reasoned proposal rather than sourced fact:

1. **Edit → compile → swap.** The ECS holds the editable graph. A control-thread system topologically sorts it, assigns buffers and allocates all state, producing an immutable `CompiledPatch`. It crosses by `rtrb`, swaps at a block boundary, and the old patch returns by a second queue for off-thread drop.
2. **No clicks.** Parameter edits only change values, smoothed per block or per sample, with no swap. A structural edit keeps state for nodes with matching stable IDs and crossfades only the changed output for 5–20 ms (this range is a starting point to measure, not a sourced value).
3. **Sample accuracy.** Split a block at event offsets, so nodes render sub-blocks. This keeps a cheap render at 64 samples and makes results independent of the device block size, which is required for determinism.
4. **Cycles.** Both FunDSP's `Net` and Firewheel reject graph cycles. An explicit feedback node with one block of delay, or a per-sample subgraph compiled at block size 1, is needed. Waveguides and Karplus-Strong need the latter.
5. **Anti-aliasing per node.** PolyBLEP/BLAMP for live oscillators; wavetable or DSF at higher quality in offline render; ADAA for waveshapers where an antiderivative exists; local oversampling around whatever remains. An offline render can afford a higher tier, but the live preview then differs from the bake, so the choice should be visible to the user.
6. **Canonical bit-identical render.** Fixed internal rate and block; all transcendentals via `ph2d-det-math` (or table lookups built from it); no runtime-dispatched SIMD (rules out `pulp` and the auto `FftPlanner` for canonical output, so use `FftPlannerScalar`); fixed target features across shipped binaries and no `avx`-without-`fma` `wide` build; explicit seeds; canonicalise NaN, or clamp output and forbid NaN reaching state; a defined denormal policy that does not rely on CPU flags; a single-threaded or order-fixed schedule; and a golden-hash gate on x86_64 and aarch64.
7. **Two engines risk.** Live preview may use SIMD; if so, its output is not the canonical output. Keeping the same code path everywhere is simpler, and should be the default until measurement says otherwise.

### Gaps
- FTZ/DAZ equivalence across x86 and aarch64 was not sourced.
- No source was found on a Rust audio engine proving bit-identical output across architectures; this would be new ground to verify by gate.
- Ideal crossfade length and block size were not found as measured values.
- The Välimäki polynomial transition-region method was not reached by search.
- Whether LLVM autovectorisation can alter float results was not sourced. Rust is understood not to contract float operations, but no citation was obtained.

## GPU for audio: modal synthesis, spectral batches, wgpu compute, and visualisation

### Takeaway
GPUs do reach hundreds of thousands of modes, but the same study shows one CPU core reaching about 65–70 thousand. SFX patches are far below that, so GPU synthesis has no strong case in the live path, and its latency tail and float behaviour both argue against it for canonical output. The clear GPU uses are spectrogram display and, possibly, non-canonical batch previews.

### Cited Findings
- 2025 Frontiers study, 48 kHz, 256-sample typical buffer, budget of 0.8 s per second of audio. **CPU:** 65–70k modes on one thread (M1/M2), up to 420k on 8 threads of an M2 Max. **GPU:** RTX 4070 494k (256) and 618k (512); M2 GPU 124k and 108k. Latency at 256 samples: CUDA 48k modes averaged 0.22 ms (max 0.62), 768k averaged 2.21 ms (p95 2.78, max 3.46); Metal 12.5k modes p50/p95 1.38/2.45 ms, 192k modes 3.78/8.21 ms. Data transfer was 12% of wall-clock time, and CPU-side summation 17% — [Frontiers in Signal Processing, 2025](https://www.frontiersin.org/articles/10.3389/frsip.2025.1522604/full).
- Its conclusion: GPUs gave the highest throughput, and multi-core CPUs "performed unexpectedly well" and are the practical choice for smaller models (paraphrase of the fetched summary) — [Frontiers, 2025](https://www.frontiersin.org/articles/10.3389/frsip.2025.1522604/full).
- Skare and Abel (DAFx 2019) ran GPU crash cymbals with over 2,000 modes per piece — [DAFx archive](https://dafx.de/paper-archive/details/qe8SzTmE_uIgihtLswWFFA).
- Skare (DAFx-20): "at 256 samples at 44.1kHz, we must process a buffer of audio in less than 5.6ms"; "We note this constraint is not met for maximum laten[cy]". Reported figures include means near 8.4–8.8 ms and maxima of 12.35–16.38 ms for heavy cases on a GTX 1080 Ti; "For inexpensive but parallel GPU computation kernels, I/O dominates scaling." Also, "the number of audio signal paths is closer to CPU core count" — [DAFx-20 paper 73](http://dafx2020.mdw.ac.at/proceedings/papers/DAFx2020_paper_73.pdf). The column reading of these table values comes from PDF text extraction and should be checked against the PDF.
- A zero-latency design must do transfer, kernel and transfer back inside one callback, and pipelining helps only if one or two buffers of latency are accepted — [DAFx-20 paper 73](http://dafx2020.mdw.ac.at/proceedings/papers/DAFx2020_paper_73.pdf).
- DAFx 2024 proposes GPU benchmarks for real-time audio "with particularly close attention to latency tail distribution" — [DAFx24 paper 56](https://dafx.de/paper-archive/2024/papers/DAFx24_paper_56.pdf).
- GPU Audio's SDK reports buffers as low as 96 samples, and its vendor session claims 30–50 µs latency. This is a vendor claim — [Sound On Sound](https://www.soundonsound.com/news/gpu-audios-sdk-released); [Embedded Online Conference](https://embeddedonlineconference.com/session/Accelerated_Audio_Computing_Leveraging_GPUs_for_Modern_DSP_Workflows).
- wgpu readback: mapping is requested after submit, and completes by polling the device; `map_async` takes a callback — [learn-wgpu 0.13 notes](https://git.blob42.xyz/Archives/learn-wgpu/raw/commit/677300a5e96fa0b06ee7c3b4e09eb9bd6fee2e88/docs/news/0.13/readme.md); [vyre-driver-wgpu readback](https://docs.rs/crate/vyre-driver-wgpu/0.6.1/source/src/engine/record_and_readback/readback.rs).
- PH2D already has a spectrogram in `ph2d-audio-spectral`, computed on the control thread — `/home/enio/Documentos/Projetos/PH2D/crates/ph2d-audio-spectral/src/lib.rs`.

### Inferences
- Scaling linearly from the study, 2,000 modes is about 3% of a single 65–70k-mode core. This is an extrapolation across hardware, and PH2D must measure it, per CLAUDE.md §0. Under §0's rule the ceiling may be raised if a measured GPU path shows extraordinary benefit, but the evidence does not favour it for SFX.
- The audio thread must never wait on a device poll. So a live GPU voice would need a dedicated worker plus at least one extra buffer of latency, and a fallback for machines with a busy or weak GPU. Games also occupy the GPU with rendering, which is outside the conditions tested in these studies.
- Best uses: **(a)** spectrogram and waveform drawing as a Vello/wgpu texture or compute pass, display only and no effect on audio bytes; **(b)** rendering many preview variations for a variation picker, labelled non-canonical, with the final file rendered on CPU.
- A mode-bank node should be built on CPU with SIMD first. A GPU path is a later optional measurement.

### Gaps
- No measurement was found of wgpu compute round-trip for audio, either the distribution or the tail. A small PH2D probe would answer this.
- WGSL float rules for exactness across vendors were not sourced here; bit-identical GPU output should be presumed unavailable until a gate shows otherwise.
- The underlying API of GPU Audio's SDK was not confirmed, and no ADC talk evidence was found.

## Plugin hosting and export: CLAP and VST3 licence terms, and whether it is worthwhile

### Takeaway
Licences no longer block either format: CLAP and the VST3 SDK are both MIT. Hosting third-party plugins conflicts with PH2D's architecture and with determinism, so it should not be built. A CLAP export, as an optional separate build target, is reasonable later but not core.

### Cited Findings
- CLAP's `LICENSE` is MIT ("Copyright (c) 2021 Alexandre BIQUE"), last push 2026-09-14. It "provides a stable ABI" with backward compatibility: "a plugin binary compiled with CLAP 1.x can be loaded by any other CLAP 1.y" — [CLAP LICENSE](https://github.com/free-audio/clap/blob/main/LICENSE); [CLAP README](https://github.com/free-audio/clap).
- VST3 SDK `LICENSE.txt` is MIT, "Copyright (c) 2026, Steinberg Media Technologies GmbH". Last push 2026-08-11 — [vst3sdk LICENSE.txt](https://github.com/steinbergmedia/vst3sdk/blob/master/LICENSE.txt).
- The change came with VST 3.8, reported in October and November 2025, replacing a dual proprietary/GPLv3 licence — [Sound On Sound](https://www.soundonsound.com/news/steinberg-adopt-mit-license-vst3); [CDM](https://cdm.link/open-steinberg-vst3-and-asio/); [Libre Arts](https://librearts.org/2025/11/steinberg-relicenses-vst3-and-asio/).
- **ASIO** gained GPLv3 alongside its proprietary licence: copyleft, so derivatives must be GPLv3 — [CDM](https://cdm.link/open-steinberg-vst3-and-asio/). `web-audio-api` and cpal both expose an ASIO option — [web-audio-api artefact](https://static.crates.io/crates/web-audio-api/web-audio-api-1.7.0.crate).
- Rust bindings: `clack-host`/`clack-plugin` 0.2.0 (2026-09-12), `clap-sys` 0.5.0 (2025-01-03), `vst3` 0.3.0 (2025-12-07), all permissive — [clack-host](https://crates.io/crates/clack-host); [clack-plugin](https://crates.io/crates/clack-plugin); [clap-sys](https://crates.io/crates/clap-sys); [vst3](https://crates.io/crates/vst3).
- NIH-plug's `nih_export_vst3!()` bindings are GPLv3 "unless replaced" — [NIH-plug README](https://github.com/robbert-vdh/nih-plug).
- Firewheel lists basic CLAP hosting as an unchecked item, and notes it is impossible in WASM — [Firewheel design doc](https://github.com/BillyDM/firewheel/blob/main/DESIGN_DOC.md).
- ADR-0075: PH2D decouples by ECS, not by runtime plugins or WASM — `/home/enio/Documentos/Projetos/PH2D/docs/architecture/decisions/0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md`.

### Inferences
- **Hosting: no.** Loading arbitrary native code is a runtime plugin by definition. It also breaks bit-identical render, cannot ship inside a game, and brings crash isolation and GUI-embedding work. Changing this would need an owner decision and a new ADR.
- **Export: optional and later.** A `cdylib` built from a drop-crate does not load anything into PH2D, so it respects ADR-0075. It lets sound designers use PH2D patches in a DAW. `clack-plugin` over CLAP is the cleanest licence path. Prefer it to NIH-plug, given maintenance mode and the GPLv3 VST3 bindings. Exporting WAV files already meets the primary SFX need.
- Keep ASIO features off in every dependency to avoid GPLv3 reaching the binary.

### Gaps
- CLAP's event timing and thread rules were not read, because the README fetched does not cover them; the headers would.
- Demand for a CLAP export among PH2D's users is unknown, and it is an owner decision.
- The `vst3` crate's bindings were not audited for derivation from GPL material; only the declared licence was read.
