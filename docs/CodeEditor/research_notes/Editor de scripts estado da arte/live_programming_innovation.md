# Live programming, visual feedback and debugging: the "wow" features for a game-script editor (state as of 2026-10-15)

> Scope: features from code editors, research systems and creative-coding tools that could make PH2D's
> integrated script editor (next to a running, deterministic 2D scene) a programmer's dream.
> Each item covers what it does, who built it, the evidence, and how it would apply to PH2D.
>
> **Source-marking convention.** URLs marked **[fetched]** were returned by searches run on
> 2026-10-15 and their content was read. URLs marked **[bk]** come from the researcher's background
> knowledge: they are well-known canonical pages, but nobody re-checked them in this session. The
> report writer should treat [bk] claims as needing a light check before quoting exact wording or numbers.
> **Maturity tags:** SHIPPED = in a product people use · RESEARCH = paper/prototype · DEMO = talk/demo only.

## 1. Live values in code (inline values, projection boxes, examples, previews, reactive notebooks)

### Takeaway
Showing runtime values *in the code's own lines* is proven and shipped (Quokka, Wallaby, JetBrains,
Xcode Previews). The research edge is *configurable* and *example-driven* views: Projection Boxes and
Babylonian Programming. A game editor has an advantage nobody else has: the running scene is always
there to supply the values.

### Cited Findings
- **Quokka.js (SHIPPED, Wallaby.js team):** a scratchpad that shows runtime values next to the code as you type. Typing a variable name logs its value. By default only the last value is shown, with an option to show all values. Coverage appears in the gutter and updates as you type — [Quokka docs](https://quokkajs.com/docs/) [fetched]; [Quokka vs Wallaby vs Console Ninja](https://quokkajs.com/docs/quokka-vs-wallaby.html) [fetched]
- **Wallaby.js (SHIPPED):** a continuous test runner that shows coverage in the left margin, updated in real time. Red squares mark lines that caused an error or failed expectation. Pink squares mark lines on the execution path of a failing test. Pinned ("sticky") inline values "survive code changes and are updated with their latest values, as you type". A 2026 "Zen Mode" hides the indicators while tests keep running in the background — [Wallaby VS Code docs](https://wallabyjs.com/docs/intro/vs-code/) [fetched]; [Wallaby newsletter Jan 2026](https://wallabyjs.com/mailinglist/archive/22-Jan-2026.html) [fetched]
- **Projection Boxes (RESEARCH, Sorin Lerner, CHI 2020, DOI 10.1145/3313831.3376494):** start from the program's full runtime semantics (every value of every variable at every loop iteration), then apply a user-chosen *projection* to show a subset in a box next to the code. Varying the projection reproduces earlier visualisations and creates new ones. In the study, users found the boxes and their configurability useful and were *not distracted* by always-on visualisation. Programmer preferences varied widely, so the view must be configurable — [Projection Boxes project page](https://cseweb.ucsd.edu/~lerner/pb/) [fetched]
- **Follow-ups to Projection Boxes:** the "loop-datavoid problem" (no test input reaches an incomplete loop, so there is no live data) led to *Loop Seeds*. *SnipPy* (UIST 2020) combines Projection Boxes with programming-by-example. A SIGCSE 2022 study used the tool in a Python CS1 course with N=237 students — [Lerner PB page](https://cseweb.ucsd.edu/~lerner/pb/) [fetched]; [SnipPy](https://snippy.goto.ucsd.edu) [fetched]; [SIGCSE 2022 paper](https://cseweb.ucsd.edu/~lerner/papers/proj-boxes-edu-sigcse2022.pdf) [fetched]. A 2023 paper also uses live programming to *validate AI-generated code* — [arXiv 2306.09541](https://arxiv.org/pdf/2306.09541) [fetched]
- **Babylonian-style Programming (RESEARCH, Rauch, Rein, Ramson, Lincke, Hirschfeld, HPI; ‹Programming› 2019, Vol. 3 Issue 3):** live *examples* are embedded in general-purpose source code. Each example is a named set of concrete inputs plus probes, and the editor shows the probed values for each example inline. The goal is to bring example-based live programming from toy domains to multi-module programs — [Programming Journal](https://programming-journal.org/2019/3/9/) [fetched]; [arXiv 1902.00549](https://arxiv.org/pdf/1902.00549) [fetched]
- **Babylonian follow-ups (HPI):** Babylonian/S (Squeak/Smalltalk implementation); *Example Mining*, which helps create examples (2022); *Probe Log*, which visualises control flow across procedures (2023); *Exploriants*, microversioning on top of Babylonian Programming (2025) — [Example Mining 2022](https://www.hpi.uni-potsdam.de/swa/publications/media/KrebsReinHirschfeld_2022_ExampleMiningAssistingExampleCreationToEnhanceCodeComprehension_AcmDL.pdf) [fetched]; [Probe Log 2023](https://www.hpi.uni-potsdam.de/swa/publications/media/KrebsReinBergsiekUrbanHirschfeld_2023_ProbeLogVisualizingTheControlFlowOfBabylonianProgramming_AcmDL.pdf) [fetched]; [Exploriants arXiv 2502.20535](https://www.arxiv.org/pdf/2502.20535) [fetched]
- **A cross-cutting view of runtime behaviour in a live environment (2024):** a later paper that adds an across-the-whole-run perspective to example-based live programming — [arXiv 2403.02428](https://arxiv.org/pdf/2403.02428) [fetched]
- **Property probes (2025):** an educational study of "property probes" (live probes over AST attributes, CodeProber) — [arXiv 2502.20540](https://arxiv.org/pdf/2502.20540) [fetched]. A Lund student project added Projection Boxes to CodeProber and reports better usability with minimal overhead — [Paulsson & Tjernström 2024](https://fileadmin.cs.lth.se/courses/edan70/CompilerProjects/2024/Reports/paulsson-tjernstrom.pdf) [fetched]
- **JetBrains inline debugger values (SHIPPED):** while paused at a breakpoint, IntelliJ-family IDEs show variable values as end-of-line hints in the editor ("Show values inline") — [IntelliJ docs: Examine suspended program](https://www.jetbrains.com/help/idea/examining-suspended-program.html) [bk]
- **Light Table (SHIPPED 2012–c.2016, now dormant; Chris Granger):** Kickstarter-funded IDE with "instarepl" and inline watches that showed values as code ran. It was a direct product attempt at Bret Victor's ideas — [lighttable.com](http://lighttable.com/) [bk]; [Kickstarter](https://www.kickstarter.com/projects/ibdknox/light-table) [bk]
- **Python Tutor (SHIPPED, Philip Guo, 2010–):** step-by-step visualisation of stack frames and heap objects as boxes and arrows, forward and back. Widely used in teaching — [pythontutor.com](https://pythontutor.com/) [bk]
- **Swift Playgrounds / Xcode Playgrounds and SwiftUI Previews (SHIPPED, Apple):** results sidebar with per-line values, value-history graphs, and live views. `#Preview` renders a view from code with no app launch — [Apple: Previews in Xcode](https://developer.apple.com/documentation/swiftui/previews-in-xcode) [bk]
- **Observable notebooks (SHIPPED):** reactive cells. Changing one cell re-runs only its dependants, in topological order, like a spreadsheet — [Observable: Reactivity](https://observablehq.com/@observablehq/how-observable-runs) [bk]
- **Jupyter (SHIPPED):** cell-by-cell REPL with rich display. Known critique: hidden state from out-of-order execution, which reactive notebooks such as Observable and marimo fix — [marimo: reactive notebooks](https://marimo.io/) [bk]
- **Bret Victor, "Learnable Programming" (2012, essay):** argues the environment should show the data, show state over time, and let you "create by reacting". It critiques Khan Academy's live-coding environment — [worrydream.com/LearnableProgramming](https://worrydream.com/LearnableProgramming/) [bk]

### Inferences (application to PH2D)
- **Scene-as-example (strongest idea).** In Babylonian terms, the running scene *is* the example. Every `self.velocity`, `hp`, or `target` line can show the value for the *selected object* in the scene. Clicking another object switches the "example". Projection-Box style, a box can also show a mini-table over several objects or over the last N frames. Nobody else has a live scene next to the editor, so this is PH2D's unique "wow".
- **Per-frame history sparklines.** Projection Boxes show all loop iterations. In a game the natural axis is *frames*, so each inline value can expand into a sparkline over the last N frames (Xcode Playgrounds already do value-history graphs per line).
- **Pinned values that survive edits** (Wallaby "sticky values") are essential once hot reload exists: the probe stays anchored to the expression across reloads.
- **Configurability is necessary** (Lerner: preferences vary widely). Keep the "supersimple surface" with *one default* (last value, selected object), then let users widen the projection on demand.
- **Loop-datavoid has a game analogue:** a code path that the current scene never reaches, such as the "on death" handler. Example *seeds* map to "spawn a test situation" (a saved scene snapshot plus inputs) attached to the function, similar to Babylonian examples.

### Gaps
- No quantitative productivity numbers (task time, error rate) for Projection Boxes or Babylonian were fetched. Only qualitative study summaries were read.
- No source was found on how inline values perform at scale (thousands of objects × probes per frame) in any game engine.

## 2. Hot reload / live coding in games and creative coding

### Takeaway
Hot code reload is table-stakes in modern engines (Godot script sync, Unreal Live Coding, Unity via a
third-party tool and a new preview package). Rust now has a credible general solution: Dioxus
**Subsecond** (2025). The hard part is never swapping the code. It is **state migration** (struct
layout changes, statics, constructors), and every engine documents that limit.

### Cited Findings
- **Dioxus Subsecond (SHIPPED, experimental; Dioxus 0.7, 2025):** a library for hot-patching *any* running Rust program (it is not tied to Dioxus UI) — "game engines, servers, and other long-running programs". It recompiles only the changed functions into a patch dylib, loads it into the running process, and updates a jump table. Calls go through `subsecond::call`. Enabled with `dx serve --hotpatch`. The 0.7 release claims support on web (WASM), desktop (macOS/Linux/Windows) and mobile (iOS/Android). It also adds "ThinLinking" for faster dev builds — [docs.rs subsecond](https://docs.rs/subsecond/) [fetched]; [Dioxus 0.7 release blog](https://dioxuslabs.com/blog/release-070) [fetched]; [Dioxus hot-reload guide](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload) [fetched]
- **Subsecond limitations:** official docs say it is experimental and "may cause crashes or unexpected behavior". A third-party summary lists: no struct size/alignment changes, thread-locals reset, only the tip crate can be patched. Verify these against the official docs — [Dioxus hot-reload guide](https://dioxuslabs.com/learn/0.7/essentials/ui/hotreload) [fetched]; [third-party skill summary](https://playbooks.com/skills/nevaberry/nevaberry-plugins/dioxus) [fetched, low authority]. Dioxus 0.7 coverage (Nov 2025): [AlternativeTo news](https://alternativeto.net/news/2025/11/dioxus-0-7-brings-hot-patching-wgpu-rendering-and-axum-integration) [fetched]
- **Bevy:** the Bevy community integrated Subsecond for hot-patching systems (the Bevy 0.17 era, 2025) — [Bevy 0.17 release notes](https://bevy.org/news/bevy-0-17/) [bk, verify]. This matters because Bevy is the closest analogue to PH2D (a Rust ECS where systems are functions).
- **Godot (SHIPPED):** the debugger inspects a running game. With "Synchronize Script Changes" on, editor edits to scripts are reloaded into the running project. "Synchronize Scene Changes" replicates scene edits. The **Remote** scene tree lets you view and edit node properties in the running game. Limit: breakpoints don't fire in code running on threads (4.0 docs). A user report says hot reload only works when the game was launched from the Godot editor, not from VS Code — [Godot 4.0 debugging overview](https://docs.godotengine.org/en/4.0/tutorials/scripting/debug/overview_of_debugging_tools.html) [fetched]; [Godot forum thread](https://forum.godotengine.org/t/hot-reloading-with-vscode-and-also-debugging-with-vscode/73968) [fetched]
- **Unreal Engine Live Coding (SHIPPED):** recompiles C++ while the editor or game runs. *Object Reinstancing* (on by default) replaces live instances so large structural changes apply. It covers UCLASS/UFUNCTION/USTRUCT/UENUM/UDELEGATE types, including generated Blueprint nodes. Defaults changed in a `.cpp` constructor do not reach existing instances. With reinstancing off, adding functions or variables or refactoring heavily "usually crashes". Code holding raw pointers to reinstanced objects needs `ReloadReinstancingCompleteDelegate` / `ReloadCompleteDelegate` — [Epic docs: Live Coding](https://dev.epicgames.com/documentation/en-us/unreal-engine/using-live-coding-to-recompile-unreal-engine-applications-at-runtime) [fetched]
- **Unreal's older Hot Reload** corrupted Blueprint assets (values reset, broken pins, "HOTRELOADED" classes). Live Coding avoids this — [Unreal community wiki](https://unrealcommunity.wiki/live-compiling-in-unreal-projects-tp14jcgs) [fetched, UE4-era]
- **Unity:** for faster iteration you can disable domain reload on entering Play mode, but static state then survives and must be reset by hand — [Unity 6.5 manual: domain reloading](https://docs.unity3d.com/6000.5/Documentation/Manual/domain-reloading.html) [fetched]. **Third-party "Hot Reload for Unity"** (Asset Store, ~2023–) applies C# edits in Play/Edit mode with no domain reload, including adding methods and fields. It is paid for larger teams — [Asset Store](https://assetstore-fallback.unity.com/packages/tools/utilities/hot-reload-edit-code-without-compiling-254358) [fetched]; [Unity Discussions launch thread](https://discussions.unity.com/t/hot-reload-edit-code-without-compiling/906893) [fetched]. **Unity's own preview package** (`com.unity.pipeline`): tag a method `[HotReload]`, edit the body while in Play mode, and save. It runs in Editor Play Mode and Mono dev builds. The 0.6 docs add an interpreter backend for IL2CPP dev builds — [com.unity.pipeline 0.6 hot reload](https://docs.unity3d.com/Packages/com.unity.pipeline@0.6/manual/hot-reload.html) [fetched; preview, not a stable feature]
- **Processing Tweak Mode (SHIPPED, Gal Sasson, GSoC 2013):** hard-coded numbers in a sketch are underlined and can be dragged while the sketch runs, and the window updates in real time. Users reported that drag precision was hard to control and asked for mouse-wheel input — [GSoC 2013 archive](https://google-melange.com/archive/gsoc/2013/orgs/processing/projects/galsasson.html) [fetched]; [Processing forum: wheel vs drag](https://forum.processing.org/two/discussion/8738/) [fetched]
- **Handmade Hero (DEMO/teaching, Casey Muratori, 2014–):** the game is split into a platform exe plus a game DLL. The DLL is reloaded when it changes, and all game state lives in one memory block owned by the platform layer, so it survives the reload. Combined with input-recording loops, this gives "live loop editing" — [Handmade Hero episode guide](https://guide.handmadehero.org/) [bk]
- **Fyrox (Rust engine, SHIPPED feature):** Fyrox 0.34 (2024) added code hot reloading for plugins via dylib reload with state (de)serialisation — [Fyrox blog](https://fyrox.rs/blog/) [bk, verify version]
- **REPL-driven development, Clojure/Lisp (SHIPPED):** the program is evaluated form by form into a running image from the editor (CIDER, Calva) — [Clojure: REPL-aided development](https://clojure.org/guides/repl/introduction) [bk]
- **Smalltalk/Pharo image (SHIPPED):** the IDE and the program are one live object image. You edit methods inside the debugger and resume ("fix and continue"). The image persists the whole state — [pharo.org](https://pharo.org/) [bk]
- **Erlang hot code loading (SHIPPED, telecom-grade):** two versions of a module (current and old) coexist. Fully qualified calls switch to the new version, and `code_change/3` in OTP behaviours migrates state explicitly — [Erlang docs: Compilation and Code Loading](https://www.erlang.org/doc/system/code_loading.html) [bk]
- **Live-coding performance (SHIPPED):** TidalCycles (Alex McLean) and Sonic Pi (Sam Aaron) re-evaluate code on beat boundaries without stopping the music, so changes land in musical time — [tidalcycles.org](https://tidalcycles.org/) [bk]; [sonic-pi.net](https://sonic-pi.net/) [bk]
- **Shadertoy (SHIPPED):** a GLSL shader recompiles on edit, with live preview and uniform time controls (pause, rewind to 0) — [shadertoy.com](https://www.shadertoy.com/) [bk]
- **Bret Victor, "Inventing on Principle" (DEMO, CUSEC, 20 Jan 2012):** principle: "creators need an immediate connection to what they're creating". Demos include a platformer where code edits immediately change the game, a time scrubber, and onion-skinned *trajectories* of the player across time. The original Vimeo video was deleted. Commenters note that games rarely have the deterministic time travel the demo relies on — [worrydream.com/InventingOnPrinciple](https://worrydream.com/InventingOnPrinciple/) [fetched]; [Future of Coding discussion](https://history.futureofcoding.org/history/weekly/2023/06/W3/linking-together.html) [fetched]; [Coding Horror](https://blog.codinghorror.com/visualizing-code-to-fail-faster/) [fetched]

### Inferences (application to PH2D)
- **Pick the reload unit first.** The ECS north star (systems don't call each other; components plus events) makes **systems the natural hot-swap unit**, the same choice Bevy+Subsecond and Handmade Hero made. Keeping all state in components/resources, never in script-local statics, removes most migration pain.
- **State migration must be explicit and safe.** Every engine documents it: Unreal constructor defaults, Unity statics, Subsecond struct layout. A PH2D design could detect a component-schema change on reload and either migrate (by field name, with defaults, like Erlang `code_change`) or refuse with a message the artist can read, rather than crash.
- **Two tiers.** (a) Script-language hot reload (if scripts run in an interpreted/JIT VM, the swap is trivial and sub-frame). (b) Rust-native hot patching via Subsecond for engine/crate code during development. Owner's "supersimple surface" → tier (a) is what users see.
- **Reload on a frame boundary,** like Tidal/Sonic Pi land changes on a beat: apply the patch between ticks so the deterministic step never sees half-old, half-new code.
- **Tweak-mode lesson:** dragging numbers needs modifier-based precision (Shift = fine, Ctrl = coarse) or wheel input. Processing users complained about overshoot.

### Gaps
- No primary measurement was found of Subsecond patch latency for a game-sized Rust workspace (only the "subsecond" claim). PH2D's monorepo of 300+ crates would need its own measurement.
- Bevy's Subsecond integration status and Fyrox's hot-reload version were not re-fetched [bk].
- Godot 4.5/4.6 changes to script hot reload were not verified (the fetched docs are 4.0-era).

## 3. Time-travel, record/replay and omniscient debugging

### Takeaway
Record/replay is mature for native code (rr, WinDbg TTD, UDB) and has a strong "omniscient" layer
(Pernosco: dataflow back in time). Web replay (Replay.io) proved the UX but the company kept
pivoting. **A deterministic game engine can get time travel almost for free** by recording inputs plus
periodic snapshots and re-simulating. This is the best fit for PH2D, whose physics is deterministic
(BTreeMap, scalar math).

### Cited Findings
- **rr (SHIPPED, Mozilla → Pernosco-sponsored):** a record-and-replay debugger for Linux that extends gdb with reverse execution (reverse-continue, reverse-step, and watchpoints backwards) — [rr-project.org](https://rr-project.org/) [bk]; [Android toolchain rr page](https://android.kernel.org/toolchain/rr/) [fetched]
- **Pernosco (SHIPPED, cloud; co-founded 2016 by Robert O'Callahan et al.):** an *omniscient* debugger built on rr recordings. Its **dataflow** feature follows a value backwards through memory and registers, including copies, and shows a clickable chain. Explanations stop at an immediate, I/O, or arithmetic origin. This is fast *because* the whole execution is pre-indexed. Scaling omniscience to very compute-heavy workloads is still an open problem — [O'Callahan: Explaining dataflow in Pernosco (2019)](https://robert.ocallahan.org/2019/11/explaining-dataflow-in-pernosco.html) [fetched]; [O'Callahan: DEBT workshop (2024)](https://robert.ocallahan.org/2024/10/debt-workshop.html) [fetched]; [Omniscient JS debugging in Pernosco (2020)](https://robert.ocallahan.org/2020/05/omniscient-js-debugging-in-pernosco.html) [fetched]
- **Pernosco is still active in 2025:** a SCAM 2025 talk on "Visualizing Program State in the Pernosco Debugger" and an ICSME 2025 hands-on briefing on "Advanced Debugging with rr and Pernosco" — [SCAM 2025](https://conf.researchr.org/details/scam-2025/scam-2025-plenary-events/2/Visualizing-Program-State-in-the-Pernosco-Debugger) [fetched]; [ICSME 2025](https://conf.researchr.org/details/icsme-2025/icsme-2025-technical-briefings/3/Hands-On-Technical-Briefing-Advanced-Debugging-With-rr-And-Pernosco-) [fetched]. A 2021 ECOOP/ISSTA talk argued for "Ubiquitous Omniscient Debugging" — [ECOOP/ISSTA 2021](https://conf.researchr.org/details/ecoop-issta-2021/ecoop-issta-2021-rebase/1/Ubiquitous-Omniscient-Debugging) [fetched]
- **Replay.io (SHIPPED then repeatedly pivoted):** time-travel debugging for the browser. It pivoted from bug reports to flaky tests (spring 2022), discontinued Replay Test Suites (July 2024), and prototyped full-stack replay (Sept 2024). A third-party listing says "Replay QA" launched on 17 May 2026 as an AI agent that runs tests in a real browser with deterministic recordings. That suggests the debugger was de-emphasised rather than shut down. Not confirmed by a primary Replay announcement — [Replay blog: A new direction](https://replay.io/blog/a-new-direction) [fetched]; [Replay: Time travel debugging as a service](https://blog.replay.io/time-travel-debugging-as-a-service) [fetched]; [everydev.ai listing](https://www.everydev.ai/developers/replayio) [fetched, aggregator]; [The New Stack](https://thenewstack.io/the-time-travel-method-of-debugging-software/) [fetched]
- **WinDbg Time Travel Debugging (SHIPPED, Microsoft):** records a process trace and lets you step backwards. Traces can be queried with LINQ-style data-model queries (for example, every call to a function across the trace) — [Microsoft Learn: TTD overview](https://learn.microsoft.com/en-us/windows-hardware/drivers/debuggercmds/time-travel-debugging-overview) [bk]
- **UDB (SHIPPED, Undo.io):** a commercial reverse debugger for Linux C/C++, gdb-compatible — [undo.io](https://undo.io/) [bk]
- **Bret Victor's time-scrubbing game demo (DEMO, 2012)** relies on determinism. The Future of Coding community notes that most games don't have it — [Future of Coding](https://history.futureofcoding.org/history/weekly/2023/06/W3/linking-together.html) [fetched]
- **Deterministic replay in shipped games:** lockstep RTS and fighting games (rollback netcode, GGPO) store only inputs and re-simulate — [GGPO](https://www.ggpo.net/) [bk]. Godot and Unity offer pause/frame-step in the editor (Unity "Step" button; Godot "Pause"/"Next frame" in the embedded game in 4.4+) — [Unity manual: Play mode controls](https://docs.unity3d.com/Manual/GameView.html) [bk, verify]

### Inferences (application to PH2D)
- **"Scene DVR" is feasible and a flagship.** With deterministic physics and an ECS, record per-frame inputs (user input, RNG seed, wall-clock-independent time) plus a snapshot every K frames. Then scrub the timeline: restore the nearest snapshot and re-simulate. This gives Bret-Victor-style scrubbing for *real* scenes, not a demo. PH2D already has a timeline UI that could host the scrubber.
- **Edit-in-the-past ("what if").** Scrub to frame 412, change the jump-force constant, and re-simulate forward from the snapshot. Onion-skin the old trajectory against the new one (the Inventing-on-Principle "trajectory" demo, made real).
- **Lightweight omniscience at ECS granularity.** Full Pernosco-style instruction recording is heavy. A component-level write log ("who wrote `Transform.x` on entity 37, which frame, which system, which script line") gives most of the dataflow value cheaply: click a value and see "last written by `enemy_ai.rs:42` at frame 410".
- **Determinism caveats to gate:** hash-ordered iteration (already avoided with BTreeMap), float non-associativity across SIMD/CPU/GPU paths, and wall-clock reads in scripts. The script API should only expose sim time.

### Gaps
- No primary source was found on Replay.io's 2025–2026 debugger status (only an aggregator).
- No data was found on the memory/CPU cost of per-frame snapshotting in 2D engines comparable to PH2D. It needs a PH2D measurement.

## 4. Moldable development and custom object views

### Takeaway
Glamorous Toolkit (GT) shows that **cheap, per-type custom views** (about 9 lines each) change how
developers understand objects. A game editor's "objects" (entities, components, sprites, paths,
state machines) are ideal subjects for inline custom views.

### Cited Findings
- **Moldable Inspector (RESEARCH → SHIPPED in Pharo/GT; Chiş, Nierstrasz, Syrel, Gîrba, 2015):** interviews with 16 developers and a survey of 62 more showed a need for several high-level views of an object, depending on object and task. Result: 131+ custom views for 84 object types across 15 applications, at **9.2 lines of code per view on average**. It replaced the standard inspector in Pharo 4 alpha — [Chiş et al., Moldable Inspector (PDF)](https://scg.unibe.ch/archive/papers/Chis15a-MoldableInspector.pdf) [fetched]
- **Glamorous Toolkit (SHIPPED, feenk):** "the moldable development environment". The inspector shows domain-specific views (for example, a file's formatted JSON or a directory's tree) next to the raw view. GT's own image has "thousands" of contextual-view and search extensions — [GT book: The Moldable Development Environment](https://book.gtoolkit.com/the-moldable-development-environment-7d4vgu73hjq8fsi6s3wy8wqpt) [fetched]; [GT book: Glamorous Toolkit as a case study](https://book.gtoolkit.com/glamorous-toolkit-as-a-case-study-3dsfofgfz37c2pvbv11o7o1qb) [fetched]; [GT Inspector views for files](https://book.gtoolkit.com/gt-inspector--views-for-files-c4ynfh3u8pi41r7vbbpdroa5k) [fetched]; [Quantifying the need for custom tools](https://book.gtoolkit.com/quantifying-the-need-for-custom-tools-b8yxhb1599idviq6g7oz8m9xm) [fetched]
- **GT examples:** GT pairs contextual views with *examples* (methods that return a configured object and double as tests and as documentation) — [GT book case study](https://book.gtoolkit.com/glamorous-toolkit-as-a-case-study-3dsfofgfz37c2pvbv11o7o1qb) [fetched]
- **Godot Remote inspector (SHIPPED):** a generic property inspector over the running game's nodes — [Godot debugging overview](https://docs.godotengine.org/en/4.0/tutorials/scripting/debug/overview_of_debugging_tools.html) [fetched]

### Inferences (application to PH2D)
- **Per-component custom views in the hover/inline popup.** A `Path` component shows its curve, a `Sprite` a thumbnail, `Health` a bar, a state machine its diagram with the current state lit, a physics body its collider outline. Each view is a small, registered drawing function, following the ECS drop-crate rule: each component crate registers its own view (GT's ~9-line-per-view economy).
- **The same view in three places:** the editor hover, the inspector panel, and the inline probe box. One definition, three consumers.
- **GT-style examples = smoke scenes.** A function annotated with an example scene is documentation, test, and live-value source at once (it connects to §1 Babylonian and §7 testing).

### Gaps
- No usage or productivity data was found for GT outside its authors' own reports. Evidence is mostly from the creators.

## 5. Visual inline widgets inside code (numbers, colours, curves, vectors, assets)

### Takeaway
Drag-to-tweak numbers and inline colour swatches are proven, beloved and cheap. Richer inline widgets
(curves, vectors, angles, sprite thumbnails, sound) are mostly demos or editor plugins, so this is
open ground for a game editor.

### Cited Findings
- **Processing Tweak Mode (SHIPPED, 2013):** drag hard-coded numbers in running code; the sketch updates live. Precision complaints → wheel requested — [GSoC 2013](https://google-melange.com/archive/gsoc/2013/orgs/processing/projects/galsasson.html) [fetched]; [forum](https://forum.processing.org/two/discussion/8738/) [fetched]; [forum: edit while running](https://forum.processing.org/two/discussion/10584/) [fetched]
- **Bret Victor's demos (DEMO):** number scrubbing, inline colour pickers, and dragging a value while the picture updates, in "Inventing on Principle" (2012) and "Learnable Programming" (2012) — [Inventing on Principle](https://worrydream.com/InventingOnPrinciple/) [fetched]; [Learnable Programming](https://worrydream.com/LearnableProgramming/) [bk]
- **Tangle (SHIPPED library, Bret Victor 2011):** "reactive documents" in which numbers in text are draggable and update dependent values — [worrydream.com/Tangle](https://worrydream.com/Tangle/) [bk]
- **Godot script editor (SHIPPED):** shows inline colour swatches for `Color(...)` literals with a picker (Godot 4.x) — [Godot docs: script editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html) [bk, verify the feature version]
- **VS Code colour decorators (SHIPPED):** colour swatches with a picker for CSS and colour literals; extensions add them for other languages — [VS Code: CSS colour picker](https://code.visualstudio.com/docs/languages/css) [bk]
- **Xcode / Swift Playgrounds colour and image literals (SHIPPED, Apple):** `#colorLiteral(...)` and `#imageLiteral(...)` render as a swatch or thumbnail in the code — [Swift docs: literal expressions](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/expressions/) [bk]
- **Hazel livelits (RESEARCH, Omar et al., PLDI 2021, "Filling Typed Holes with Live GUIs"):** user-defined GUIs (colour picker, slider, data frame) that *are* expressions in the code, typed and composable, with live feedback — [arXiv: Filling Typed Holes with Live GUIs](https://arxiv.org/abs/2104.01326) [bk]

### Inferences (application to PH2D)
- **Typed inline widgets driven by the engine's type registry:** `Color` → swatch plus palette picker (shared with the Painter palette); `Vec2` → mini arrow gizmo, draggable *in the scene* too; angle → dial; `Curve`/easing → inline curve editor (shared with the timeline graph editor); asset refs → sprite thumbnail / audio play button; durations → a bar against the frame rate.
- **Bidirectional:** dragging the gizmo in the scene rewrites the literal in code, and editing the literal moves the gizmo (Victor's direct manipulation, Hazel livelits). Literals stay plain text in the file, so diffs and merges stay clean.
- **Livelits suggest a principled form:** the widget *is* a typed literal, so it can never produce an ill-typed value. This is an error-prevention feature, not just eye-candy.
- **Respect the UI rules:** in PH2D, inline widgets must use design tokens and i18n like any widget (CLAUDE.md §0.3).

### Gaps
- No usability study of inline widget density in code (when it helps vs clutters) was found.
- Hazel livelits details were not re-fetched [bk].

## 6. Program visualisation and spatial code navigation

### Takeaway
Working-set and canvas UIs (Code Bubbles, Code Canvas, Debugger Canvas) have controlled-study
evidence and an industrial trial, but they never displaced tabbed editors. Zed's *multibuffers* are a
shipped, lighter version. For game scripts, the biggest wins are **diagrams generated from code that
light up live**: state machines, event/signal flow, and system order.

### Cited Findings
- **Code Bubbles (RESEARCH, Bragdon et al., Brown; CHI 2010 pp. 2503–2512, plus an ICSE 2010 demo):** an IDE built from "bubbles" (fragments: functions, notes, debugger views) on a pannable 2D canvas, organised as working sets. A quantitative study found it **reduced code-understanding time and navigation interactions** compared with a widely used IDE (Eclipse) across two controlled tasks — [Code Bubbles CHI 2010 (PDF)](https://www.eecs.ucf.edu/~jjl/pubs/p2503-bragdon.pdf) [fetched]; [UCF record](https://stars.library.ucf.edu/scopus2010/1077) [fetched]; [ICSE 2010 demo](https://stars.library.ucf.edu/scopus2010/1056) [fetched]
- **Code Canvas / Debugger Canvas (RESEARCH → shipped as a VS power tool; Microsoft Research, DeLine et al.; ICSE 2012 industrial report):** Code Bubbles–style canvas applied to *debugging* in Visual Studio. The report includes lab participants, usage data and surveys — [MSR: Debugger Canvas](https://www.microsoft.com/en-us/research/publication/debugger-canvas-industrial-experience-with-the-code-bubbles-paradigm/) [fetched]; [ICSE 2012 DOI](https://doi.ieeecomputersociety.org/10.1109/ICSE.2012.6227113) [fetched]; [DeLine Code Canvas paper (PDF)](https://microsoft.com/en-us/research/wp-content/uploads/2016/02/p1064-213-deline.pdf) [fetched]
- **Probe Log (RESEARCH, HPI 2023):** visualises control flow across multiple procedures for Babylonian examples — [Probe Log](https://www.hpi.uni-potsdam.de/swa/publications/media/KrebsReinBergsiekUrbanHirschfeld_2023_ProbeLogVisualizingTheControlFlowOfBabylonianProgramming_AcmDL.pdf) [fetched]
- **Pernosco program-state visualisation (2025 talk)** — [SCAM 2025](https://conf.researchr.org/details/scam-2025/scam-2025-plenary-events/2/Visualizing-Program-State-in-the-Pernosco-Debugger) [fetched]
- **Zed multibuffers (SHIPPED):** a single editable buffer composed of excerpts from many files (search results, diagnostics, references). Edits apply to the source files — [Zed docs: multibuffers](https://zed.dev/docs/multibuffers) [bk]

### Inferences (application to PH2D)
- **Code ↔ scene linking:** click an entity and see the scripts and systems that touch it, as a multibuffer of the relevant excerpts. Click a script line and the affected entities highlight in the scene.
- **Auto-diagrams from code, animated live:** state machines (current state highlighted, transitions flash when taken), signal/event flow between objects (PH2D already has a signal runtime with per-consumer cursors), and the per-frame system order (a gated invariant in PH2D). This is projection-box thinking at architecture scale.
- **A board/canvas mode** (PH2D has a Miro-like board) can host Code Bubbles–style working sets. Note the risk: canvas IDEs had good lab results but low long-term adoption, so keep it optional.

### Gaps
- No adoption numbers were found for Debugger Canvas or Code Bubbles after their studies. The decline of canvas IDEs is inferred, not sourced.
- The exact numeric study results in the Code Bubbles PDF were not extracted.

## 7. Testing in the editor (continuous testing, examples as tests, snapshots, properties)

### Takeaway
Continuous inline testing (Wallaby, NCrunch) is proven and loved by its users. **Examples that are
simultaneously tests, docs and live-value sources** (GT examples, Babylonian examples) are the
research frontier. Deterministic replays make *game-scene snapshot tests* unusually cheap and
reliable.

### Cited Findings
- **Wallaby.js (SHIPPED):** runs tests continuously as you type. Gutter coverage, red/pink markers for failing lines and failing paths, inline errors, sticky values — [Wallaby VS Code docs](https://wallabyjs.com/docs/intro/vs-code/) [fetched]; [Wallaby newsletter 2021](https://wallabyjs.com/mailinglist/archive/03-Mar-2021.html) [fetched]
- **NCrunch (SHIPPED, .NET):** automated concurrent continuous testing, with coverage markers in the code margin — [ncrunch.net](https://www.ncrunch.net/) [bk]
- **GT examples (SHIPPED):** example methods return objects, so they act as tests, documentation and inspector starting points — [GT case study](https://book.gtoolkit.com/glamorous-toolkit-as-a-case-study-3dsfofgfz37c2pvbv11o7o1qb) [fetched]
- **Babylonian examples** as executable, inline example values — [Programming Journal 2019](https://programming-journal.org/2019/3/9/) [fetched]
- **Unison caches test results by content hash:** an unchanged definition (and its dependencies) never re-runs its tests — [Unison: The big idea](https://www.unison-lang.org/docs/the-big-idea/) [fetched]; [The Register 2019](https://www.theregister.com/2019/09/26/unison_programming_language) [fetched]
- **Snapshot testing (SHIPPED):** Jest snapshots; in Rust, `insta` with a review TUI (`cargo insta review`) — [insta.rs](https://insta.rs/) [bk]
- **Property-based testing (SHIPPED):** QuickCheck / proptest / Hypothesis — [proptest book](https://proptest-rs.github.io/proptest/) [bk]

### Inferences (application to PH2D)
- **"Scene tests" = recorded replays plus assertions.** Because the sim is deterministic, a recorded play session is a regression test. Re-run it after an edit and diff the end state, or diff per-frame component values. Show the result inline next to the script ("3 recorded sessions still pass; session 2 diverges at frame 318 — view").
- **Content-hash test caching** (Unison idea) maps to PH2D: hash the script AST plus the hashes of its dependencies, and skip replays whose inputs didn't change.
- **Property tests over scenes:** fuzz input sequences and assert invariants ("player never leaves bounds", "hp ≥ 0"). Determinism makes every failure reproducible as a seed.

### Gaps
- No evidence was found of a game engine shipping continuous inline replay-tests in its script editor. This looks like open ground, but absence is not proven.

## 8. Error prevention in live contexts (safe reload, infinite loops, watchdogs, typed holes)

### Takeaway
Live systems need three guards: **(1) evaluation that survives incomplete or broken code** (Hazel's
typed holes), **(2) budgets/interrupts for runaway scripts** (instruction counting or interrupt
callbacks), and **(3) explicit state migration on reload** (Erlang `code_change`, Unreal
reinstancing, Unity static-reset rules).

### Cited Findings
- **Hazel / Hazelnut Live (RESEARCH, Omar, Voysey, Chugh, Hammer, POPL 2019):** incomplete programs contain *holes*. Empty holes stand in for missing code; non-empty holes are "membranes" around type errors. Evaluation **proceeds around holes** instead of halting, tracking each hole's closure, and supports **fill-and-resume** (fill a hole without restarting the evaluation). The calculus is mechanised in Agda. The authors call it the first "maximally live" environment for a general-purpose functional language. Later work: the marked lambda calculus (error localisation and recovery) and pattern matching with holes — [arXiv 1805.00155](https://arxiv.org/pdf/1805.00155) [fetched]; [POPL 2019 page](https://popl19.sigplan.org/event/popl-2019-research-papers-live-functional-programming-with-typed-holes) [fetched]; [NSF PAR record](https://par.nsf.gov/biblio/10602723-live-functional-programming-typed-holes) [fetched]; [Live pattern matching with typed holes](https://par.nsf.gov/biblio/10603235-live-pattern-matching-typed-holes) [fetched]
- **Runaway-script guards in Lua hosts:** a line/count hook with a clock check is the classic approach, but an always-on hook slows code. Better: arm the hook from a timer when the deadline passes. In Lua 5.4.0 a hook-based guard can fail to fire in tight loops (trap-flag check), and signals don't interrupt C functions or coroutines. HAProxy yields every N Lua instructions and benchmarked N from 10 to 1,000,000, choosing **10,000** as the balance between overhead and responsiveness — [lua-l 2020-05](https://lua-users.org/lists/lua-l/2020-05/msg00283.html) [fetched]; [lua-l 2002-06](https://lua-users.org/lists/lua-l/2002-06/msg00411.html) [fetched]; [HAProxy patch](https://git01.mediatek.com/plugins/gitiles/haproxy/+/ee9f802c4fb69aecd613a576aaf587bc8f982929%5E%21) [fetched]
- **Luau (Roblox):** exposes an `interrupt` callback in `lua_Callbacks`, which the host uses to stop long-running scripts (Roblox's "script timeout") — [Luau GitHub (lua.h)](https://github.com/luau-lang/luau) [bk, not confirmed by search]
- **Wasm fuel / epoch interruption (SHIPPED in Wasmtime):** deterministic instruction "fuel" or cheaper epoch-based interruption for sandboxed guests — [Wasmtime docs: interrupting execution](https://docs.wasmtime.dev/examples-interrupting-wasm.html) [bk]. (PH2D's ADR-0075 rejects WASM as the *decoupling* mechanism; this is only a reference for sandboxing technique.)
- **State-migration rules:** Unreal reinstancing and its delegates [§2]; Unity's disabled domain reload needs a static-state reset [§2]; Erlang `code_change/3` [§2, bk]
- **Unreal's old Hot Reload corrupted assets,** an example of unsafe reload — [community wiki](https://unrealcommunity.wiki/live-compiling-in-unreal-projects-tp14jcgs) [fetched]

### Inferences (application to PH2D)
- **Per-script instruction budget, deterministic.** Count VM instructions (deterministic, unlike wall-clock), so an "exceeded budget" result is itself reproducible in replay. On overrun, suspend that script's system for the frame, keep the scene running, and highlight the loop line in the editor. Per CLAUDE.md §0.0 the budget must be *measured* (HAProxy's 10,000 is a reference point, not PH2D's number).
- **Never stop the world on a typo:** Hazel's lesson. While a script doesn't parse or typecheck, keep running the *last good* version of that system and mark the broken region in the editor (a cheaper approximation of evaluating around holes).
- **Reload transaction:** compile → typecheck → schema-diff components → migrate or refuse → swap at a frame boundary. All steps are visible in a small status chip, never only a terminal message (PH2D's own lesson: a refusal only the terminal sees is a mute button).
- **Watchdog for the editor itself:** the script VM runs so that a hang cannot freeze the editor UI. Snapshots (§3) allow rollback to before the bad frame.

### Gaps
- The Luau interrupt API details were not verified in this session.
- No comparative measurement was found of instruction-count overhead across embeddable VMs (Lua, Luau, Rhai, Wasm fuel).

## 9. Notable 2023–2026 projects and what to take from each

### Takeaway
The 2023–2026 frontier is *liveness everywhere*: holes-and-livelits (Hazel), content-addressed code
(Unison), hot-patching for Rust (Subsecond 2025), dynamic documents (Ink & Switch Potluck → Embark),
and omniscient-debugging UX (Pernosco). Several commercial "live" ventures pivoted (Replay.io, Darklang).
Shipped value concentrates in tooling around existing languages.

### Cited Findings
- **Hazel (RESEARCH, active):** full-scale live functional environment, typed holes, livelits — [arXiv 1805.00155](https://arxiv.org/pdf/1805.00155) [fetched]; [hazel.org](https://hazel.org/) [bk]
- **Unison (SHIPPED language/platform):** every definition is identified by a hash of its syntax tree (512-bit SHA3, per a 2019 article), covering its dependencies. Names are separate metadata, so renames are free. Claims: "no builds" (perfect incremental compilation with a shared cache in the codebase format), cached test results, no dependency conflicts. Unison Cloud is the commercial platform — [Unison: The big idea](https://www.unison-lang.org/docs/the-big-idea/) [fetched]; [Unison README](https://cdn.jsdelivr.net/gh/unisonweb/unison@trunk/README.md) [fetched]; [The Register 2019](https://www.theregister.com/2019/09/26/unison_programming_language) [fetched]; [SoftwareMill: code as hashes](https://softwaremill.com/trying-out-unison-part-1-code-as-hashes/) [fetched]
- **Dioxus Subsecond (SHIPPED experimental, 2025):** see §2 — [Dioxus 0.7 blog](https://dioxuslabs.com/blog/release-070) [fetched]
- **Ink & Switch Potluck (RESEARCH, SPLASH 2022 LIVE track):** "gradual enrichment" from text documents to apps. Live searches extract structure from freeform text, formulas compute over it, and results appear as dynamic annotations in the document. Used for recipes, workouts, chores — [Ink & Switch: Potluck](https://www.inkandswitch.com/project/potluck/) [fetched]
- **Ink & Switch Embark (RESEARCH, c. 2023–2024; Sonnentag, Litt, Obenauer):** dynamic documents for making plans, a follow-up to Potluck's design problems. A third-party (possibly AI-generated) video summary describes three primitives: mentions (structured data in an outline), formulas, and views bound to outline nodes — [Ink & Switch dispatch 001](https://www.inkandswitch.com/newsletter/dispatch-001/) [fetched]; [video summary](https://youtubesummary.com/summary/MccJdr61xnc) [fetched, low authority]. Geoffrey Litt has since moved to Notion — [Litt talk page](https://luma.com/rc-dynamic) [fetched]
- **Pernosco (SHIPPED, active 2025):** see §3 — [SCAM 2025](https://conf.researchr.org/details/scam-2025/scam-2025-plenary-events/2/Visualizing-Program-State-in-the-Pernosco-Debugger) [fetched]
- **Replay.io:** pivots from 2022 to 2026, see §3 — [Replay blog](https://replay.io/blog/a-new-direction) [fetched]
- **Darklang (SHIPPED then rebooted):** "deployless" backend platform with trace-driven development. Real request traces are shown inline in the editor, and code runs against them. The company announced a rewrite/reboot (2023) and later pivoted toward an AI/CLI focus — [darklang.com](https://darklang.com/) [bk]; [Darklang blog](https://blog.darklang.com/) [bk, verify 2024–2026 status]
- **Val Town (SHIPPED):** social, instantly deployed TypeScript "vals" with live run results — [val.town](https://www.val.town/) [bk]
- **Natto (DEMO/SHIPPED web tool, Paul Shen):** a canvas of JS cells with live, spatially laid-out evaluation. Shen also presented at SPLASH 2022 (LIVE) — [natto.dev](https://natto.dev/) [bk]; [Paul Shen SPLASH profile](https://2021.icse-conferences.org/profile/splash-2022/paulshen) [fetched]
- **Enso (SHIPPED, formerly Luna):** dual visual-node and textual language for data, with live per-node visualisations — [enso.org](https://enso.org/) [bk]
- **Kaleidoscope:** no reliable source was found in this session for the "Kaleidoscope" project the brief mentions (an ambiguous name). Left as a gap.
- **LIVE workshop (SPLASH):** the annual venue for this research (Potluck was presented there) — [Ink & Switch Potluck page](https://www.inkandswitch.com/project/potluck/) [fetched]; [LIVE 2021 topics](https://lively-kernel.org/lively4S2/lively4-seminars/Live2021/Live21_Topics.pdf) [fetched]

### Inferences (application to PH2D) — consolidated feature catalogue, ranked by fit × feasibility

| # | Feature | Proven by | Fit for PH2D |
|---|---|---|---|
| 1 | **Inline live values for the *selected scene object*, with per-frame sparkline history** | Quokka/Wallaby (shipped), Projection Boxes, Babylonian, Xcode history graphs | Unique: the scene supplies the examples. Core "wow" |
| 2 | **Hot reload at frame boundary, unit = system/script, with schema-diff state migration** | Godot sync, Unreal Live Coding + reinstancing, Subsecond, Erlang | Core. ECS makes the unit obvious |
| 3 | **Scene DVR: scrub, rewind, edit-in-the-past, onion-skinned trajectories** | rr/TTD (native), Bret Victor (demo), rollback netcode | Flagship. Feasible *because* PH2D is deterministic |
| 4 | **Typed inline widgets** (colour, Vec2 gizmo bidirectional with scene, curve, angle, asset thumbnail, sound) | Tweak Mode, Godot/VS Code swatches, Swift literals, Hazel livelits | High delight, low risk. Reuses PH2D widgets |
| 5 | **Never stop on a typo: last-good version keeps running; deterministic instruction budget** | Hazel (research), Lua/Luau hosts, Wasmtime fuel | Error prevention, "fewer errors" |
| 6 | **Component-level "who wrote this value" dataflow** | Pernosco dataflow (omniscient) | High debugging value, cheap at ECS granularity |
| 7 | **Moldable per-component views** (in hover, inspector and probe) | GT / Moldable Inspector (9.2 LOC/view) | Fits the drop-crate rule |
| 8 | **Replay-based scene tests shown inline, hash-cached** | Wallaby/NCrunch, GT examples, Unison test cache, insta | Continuous safety net |
| 9 | **Live diagrams from code** (state machines, signal flow, system order) | Probe Log, Pernosco viz, Code Bubbles | Medium. Reuses the board/timeline |
| 10 | **Code ↔ scene cross-highlighting and multibuffer of "everything touching this entity"** | Zed multibuffers, Code Bubbles/Canvas | Medium |

- **Main design tension:** "supersimple surface" vs configurability (Lerner found preferences vary widely). Resolution: one sensible default per feature (last value, selected object, current frame), with depth revealed only on hover or expand.
- **Main technical prerequisite shared by #1, #3, #5, #6 and #8:** a script VM or system runner that is deterministic, instruction-countable, and can snapshot and restore component state. Decide it first.

### Gaps
- "Kaleidoscope": not identified.
- Darklang, Val Town, Natto and Enso status in 2025–2026 was not re-verified in this session [bk].
- No 2025–2026 Strange Loop/Handmade material was fetched. Strange Loop ended in 2023 (bk, not verified here).
- No user-reception metrics (surveys, retention) were found for any live-values tool beyond qualitative study reports and vendor docs.
