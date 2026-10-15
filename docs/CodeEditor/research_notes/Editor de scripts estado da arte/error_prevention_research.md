# Editor techniques that prevent/reduce human programming errors and speed up programming — evidence map (state of the art, Oct 2026)

> Scope: research notes for the PH2D integrated script/code editor (Rust, text-first). Every fact carries
> an inline source. "Folklore" = widely repeated, no peer-reviewed support found in this session.
> Populations matter: most HCI/CS-ed studies are on **novices/students with small N**; most industrial
> numbers are **professionals, large N, observational**. Read the effect sizes with that in mind.

## 1. Structure editors and hybrids (Hazel, MPS, frame-based Stride/Strype, blocks→text, tylr, Pantograph): fewer errors? at what usability cost?

### Takeaway
Structure/hybrid editors reliably eliminate *syntax* errors (by construction), and frame-based editing measurably cuts time spent on syntax and in non-compiling states for novices — but every controlled study of "pure" structure editing for people who already know text shows a speed/viscosity cost (Pantograph 1.4× slower; MPS needs "substantial experience"). The 2023–2025 frontier (tylr "gradual structure editing", Hazel "total error localization with holes") converges on the **hybrid**: keep text-like input, but make the editor *always* hold a well-formed, typed model with explicit holes/obligations — which is directly portable to a text-first editor.

### Cited Findings
- **Stride (frame-based) vs Java text, middle-school students, Greenfoot:** Stride group spent *significantly less time making purely syntactic edits* and *significantly less time with non-compilable code*, progressed faster and completed more objectives; frustration/satisfaction similar between groups (exact N and statistics not on the abstract page). — [Price, Brown, Lipovac, Barnes, Kölling, ICER 2016](https://isnap.csc.ncsu.edu/home/public/publication/price-2016-icer), DOI [10.1145/2960310.2960319](https://doi.org/10.1145/2960310.2960319)
- A Blackbox-log-based analysis by Brown, Altadmri & Kölling *predicts* a clear advantage of frame-based over text editing (prediction from logs, not a controlled experiment). — [search summary pointing to kar.kent.ac.uk records](https://kar.kent.ac.uk/56259/)
- Strype (frame-based **Python**, browser) reached 1.0 in 2024 (Weill-Tessier, Brown, Kölling); I found **no peer-reviewed evaluation of Strype itself**. — [KCL proged blog, Sept 2024](https://blogs.kcl.ac.uk/proged/2024/09/); [CAS 2024](https://computingatschool.org.uk/news-blogs/2024/november/teaching-python-programming-with-strype)
- **JetBrains MPS (projectional) controlled experiment**, 19 participants (14 CS students + 5 industrial devs): editing is efficient for basic tasks but strategies and typical errors differ from text; complex tasks are efficient only with "substantial experience" and AST understanding; explicit trade-off between *fewer typing mistakes* and *increased complexity of code editing*. — [Berger, Völter, Jensen, Dangprasert, Siegmund, FSE 2016, DOI 10.1145/2950290.2950315](https://unpaywall.org/10.1145%2F2950290.2950315); [GI 2018 abstract](https://dl.gi.de/handle/20.500.12116/21147)
- **Blocks vs text (high school, Pencil Code, blocks / text / hybrid for 5 weeks then 10 weeks Java):** blocks students scored better on assessments initially, but after transitioning to Java there was *no significant difference* between modalities; hybrid (blocks that turn into text) outperformed both in certain dimensions. — [Weintrop & Wilensky, Computers & Education 142 (2019)](https://www.terpconnect.umd.edu/~weintrop/papers/Weintrop&Wilensky_2019_C&E.pdf); [IDC 2017 hybrid paper](https://ccl.northwestern.edu/2017/Weintrop_Wilensky_IDC_2017.pdf); [2021 write-up](https://terpconnect.umd.edu/~weintrop/papers/Weintrop_RaspPi_2021.pdf)
- **Hazel — typed holes, live evaluation of incomplete programs (Hazelnut Live):** empty holes stand for missing code; *non-empty holes* wrap static/dynamic type inconsistencies; evaluation proceeds *around* holes instead of stopping; core calculus mechanized in Agda. — [Omar, Voysey, Chugh, Hammer, POPL 2019 (arXiv 1805.00155)](https://ar5iv.labs.arxiv.org/html/1805.00155); [POPL'19 page](https://popl19.sigplan.org/event/popl-2019-research-papers-live-functional-programming-with-typed-holes)
- **Hazel — Total Type Error Localization and Recovery with Holes (POPL 2024, Distinguished Paper):** "marked lambda calculus" — a *total* procedure that marks every error as a non-empty hole so the rest of the program keeps typing; bidirectional typing makes error localization *predictable from local information*; constraint solving offers partial hole fills that let the user move the error. — [POPL 2024 page](https://popl24.sigplan.org/details/POPL-2024-popl-research-papers/70/Total-Type-Error-Localization-and-Recovery-with-Holes); [UMich award note](https://cse.engin.umich.edu/stories/cse-researchers-win-distinguished-paper-award-at-popl-2024)
- **teen tylr — "Gradual Structure Editing with Obligations" (VL/HCC 2023):** allows *temporary disassembly* of structure for text-like editing while generating syntactic *obligations* that guarantee reassembly; within-subjects lab study (10 participants, P0–P9, vs a text editor and MPS): resolved *most but not all* usability problems of prior structure editors and achieved *competitive performance with text on most tasks*. — [Moon, Blinn, Omar, VL/HCC 2023 pre-print](https://hazel.org/papers/teen-tylr-vlhcc2023.pdf); [program page](https://conf.researchr.org/details/vlhcc-2023/vlhcc-2023-research-papers/8/Gradual-Structure-Editing-with-Obligations)
- **tall tylr — "Syntactic Completions with Material Obligations" (OOPSLA 2025):** user study with 9 participants: 8/9 at least somewhat agreed it was easy to use; 7 said obligations (ghost delimiters/placeholders) *helped while writing*, but 5 said they *got in the way while modifying*, and half found placeholders visually distracting; ≥5 tried to delete "ghosts" directly. — [arXiv 2508.16848](https://arxiv.org/pdf/2508.16848); [SPLASH 2025](https://2025.splashcon.org/details/OOPSLA/209/Syntactic-Completions-with-Material-Obligations)
- **Pantograph (POPL 2025)** — typed structure editor that edits by cutting/pasting typed one-hole contexts ("type diffs") to fight *viscosity*: study with 13 upper-level FP students (11 analysed), 1-hour tutorial, 12 tasks split between Pantograph and a Monaco text editor: Pantograph was **1.4× slower** (geometric mean) overall; most participants could solve tasks; a common failure was trying to use it like a text editor. — [Prinz, Blanchette, Lampropoulos, arXiv 2411.16571](https://arxiv.org/pdf/2411.16571); [POPL 2025](https://popl25.sigplan.org/details/POPL-2025-popl-research-papers/28/Pantograph-A-Fluid-and-Typed-Structure-Editor)
- Recent Hazel-based tools (Hazel Deriver 2025; Hazel tables, SPLASH 2026) have only tiny studies (N≈7). — [Hazel Deriver arXiv 2506.10781](https://arxiv.org/pdf/2506.10781); [SPLASH 2026 "Lively Typed Tables"](https://2026.splashcon.org/details/oopsla-2026/127/Interactive-Data-Analysis-with-Lively-Typed-Tables)

### Inferences
- For PH2D (users = artists/game designers, mostly novice-to-intermediate programmers, but power users exist): a pure projectional editor is high-risk (speed cost, learning curve); a **text-first editor with an always-valid error-tolerant model** (tree-sitter-style incremental parse + Hazel-style "every error becomes a hole, everything else keeps working") captures most of the error-prevention benefit without the viscosity cost.
- Frame-based evidence suggests *structural insertion of statement skeletons* (e.g., inserting an `if`/`for`/event handler as a template with typed slots) is where structure helps most; free expression text inside slots is where text wins.
- tall-tylr's mixed result on placeholders → make ghost/auto-closed delimiters *deletable and low-contrast*, and never block modification.

### Gaps
- No peer-reviewed numbers for tree-sitter structural editing in Helix / Emacs Combobulate (product features; no usability studies found).
- Exact N/effect sizes of the Stride ICER 2016 and Altadmri et al. COMPSAC 2016 ("The cost of syntax and how to avoid it") were not retrievable in this session.
- No evaluation of Strype found.

## 2. Compiler/diagnostic message design (Becker WG 2019, Elm/Rust, enhanced messages, LLM explanations, placement)

### Takeaway
The evidence on "enhanced" messages is genuinely **mixed**: some large-N classroom studies show fewer and fewer repeated errors, others show no effect, and GPT-4 explanations beat stock messages in only 1 of 6 tasks in a realistic study. What *is* consistent: developers do read messages, it costs them 13–25% of task time, readability predicts performance, and pure *presentation* (layout, pointing at the source span) changes comprehension. The Elm/Rust "show the user's own code with labelled spans" design is industry best practice but is folklore-grade in terms of controlled evidence.

### Cited Findings
- ITiCSE 2019 Working Group (Becker, Denny, Pettit et al.), 34 pages, 219 references: recommendations include increasing readability, reducing cognitive load, providing context, positive tone, showing solutions/hints. — [Becker et al., "Compiler Error Messages Considered Unhelpful", DOI 10.1145/3344429.3372508](https://neverworkintheory.org/2021/09/02/compiler-error-messages-considered-unhelpful.html)
- **Positive:** Becker 2016 — CS1 Java, ~200 students, ~50,000 errors: enhanced messages associated with fewer overall errors, fewer errors per student, fewer repeated errors; 8 specific messages had significant effects. — [UCD repository](https://researchrepository.ucd.ie/entities/publication/d4b1c9ce-e7ff-48e7-b9b9-f0ebe0c15e8e); [CORE (CSE journal version)](https://core.ac.uk/works/78318990)
- **Null:** Denny, Luxton-Reilly, Carpenter 2014 — controlled study of enhanced syntax messages with examples of error and fix: *no significant effect*. — [ITiCSE '14, DOI 10.1145/2591708.2591748](https://api.crossref.org/works/10.1145%2F2591708.2591748); Pettit, Homer, Gee 2017 "Do Enhanced Compiler Error Messages Help Students? Results Inconclusive" — no measurable benefit. — [summary via Frontiers 2021 review](https://www.frontiersin.org/journals/psychology/articles/10.3389/fpsyg.2021.768962/full)
- **LLM explanations, realistic setting:** within-subjects, n=106 students fixing 6 buggy C programs with stock vs expert-handwritten vs GPT-4 messages: GPT-4 beat stock messages in **only 1 of 6 tasks** (time-to-fix); **handwritten explanations outperformed both**, objectively and subjectively. — [Santos & Becker, UKICER 2024, arXiv 2409.18661](https://arxiv.org/pdf/2409.18661)
- **Do developers read them?** Eye-tracking, 56 students, Java/Eclipse: participants read messages; reading difficulty is comparable to reading source code and *significantly predicts task performance*; 13–25% of task time is spent reading error messages (figures via review). — [Barik et al., ICSE 2017, DOI 10.1109/ICSE.2017.59, reviewed](https://neverworkintheory.org/2021/09/20/do-developers-read-compiler-error-messages.html)
- **Presentation alone matters:** three purely visual variants of one error message (based on visual-perception theory) gave substantial improvements in comprehension and resolution in an online experiment. — [Khandwala et al., CHI 2019 LBW, "The Impact of 'Cosmetic' Changes on the Usability of Error Messages"](https://research.google/pubs/the-impact-of-cosmetic-changes-on-the-usability-of-error-messages/)
- **Refactoring-tool errors:** graphical "Refactoring Annotations" on the code let programmers understand the cause of a refused refactoring faster and more accurately than textual messages (two experiments). — [Murphy-Hill & Black, IEEE TSE 2012 "Programmer-Friendly Refactoring Errors"](https://works.bepress.com/andrew_black/35)
- **Elm (2015):** show the code *as the user wrote it* instead of `file:43:22`; later versions add hints with examples and links. — [Czaplicki, "Compiler Errors for Humans"](https://elm-lang.org/news/compiler-errors-for-humans); [Elm "Compilers as Assistants"](https://elm-lang.org/blog/compilers-as-assistants)
- **Rust RFC 1644:** lighter format focused on user code, labels placed on source spans, noise removed, explicitly inspired by Elm; Elm/Rust also expose machine-readable (JSON) diagnostics so editors don't parse prose. — [RFC 1644](https://rust-lang.github.io/rfcs/1644-default-and-expanded-rustc-errors.html); [Rust internals discussion](https://internals.rust-lang.org/t/compiler-errors-for-humans/2310)

### Inferences
- PH2D should invest in **structure of the message** (span labels on the user's code, one primary cause, a concrete fix/quick-fix) more than in friendly prose; the strongest single lever with evidence is *placement + visual design*, and the second is *a hand-written, per-error explanation for the top-N errors* (Becker's 8 messages; handwritten > GPT-4).
- Measure which errors PH2D users actually hit (log error codes) and hand-write explanations for the top ~20–30, Becker-style; LLM explanations only as an optional secondary layer.
- Diagnostics should be a structured data type (code, primary span, secondary spans, suggested edits) feeding both inline rendering and quick-fixes — the Rust/LSP shape.

### Gaps
- No controlled study found comparing **inline (squiggle/inline-text) vs panel** placement of errors for professionals; Khandwala 2019 is the closest evidence.
- No controlled evaluation of Rust's or Elm's message style vs. classic gcc-style found (industry folklore, though highly plausible).

## 3. Static typing / static analysis in the editor

### Takeaway
Static type checking catches a measurable, non-trivial fraction of *shipped* bugs (≈15% of public JS bugs, at ~2 annotation tokens per bug) and controlled experiments show static types help when using undocumented APIs and fixing type errors — but not semantic errors, and not uniformly faster. For an editor, the actionable lever is **live, error-tolerant type feedback while typing** (Hazel's total marking), not just a batch compile.

### Cited Findings
- 400 fixed public JavaScript bugs; annotating only the fix region (≤10 min per bug): **Flow detected 59, TypeScript 58 → ~15%**; average annotation "token tax" **1.7 tokens (Flow) / 2.4 (TypeScript)** per detected bug; authors call public bugs a *conservative* measure (bugs that survived testing/review). — [Gao, Bird, Barr, ICSE 2017, "To Type or Not to Type"](https://earlbarr.com/publications/typestudy.pdf)
- Controlled experiment, 27 subjects, undocumented API: static types faster for some tasks, slower for others. — [Mayer et al., OOPSLA 2012, DOI 10.1145/2384616.2384666](https://www.doi.org/10.1145/2384616.2384666)
- Maintainability experiment: static types beneficial for understanding undocumented code and fixing *type* errors, *not* for fixing semantic errors. — [Hanenberg et al., EMSE 2014](https://pleiad.cl/papers/2014/hanenbergAl-emse2014.pdf)
- Static typing's benefit is *strengthened* with explicit documentation (API usability experiment). — [Endrikat, Hanenberg, Robbes, Stefik 2014, via search summary](https://upapers.dcc.uchile.cl/index/publications/view/250878)
- Error-tolerant typing for editors: Hazel's POPL 2024 total marking keeps type information available everywhere despite errors, which is what enables completions and LLM context in broken programs. — [POPL 2024](https://popl24.sigplan.org/details/POPL-2024-popl-research-papers/70/Total-Type-Error-Localization-and-Recovery-with-Holes)
- Type + binding context of a hole, supplied from the language server to an LLM ("static retrieval"), improves generated code; authors propose **ChatLSP**, an LSP extension for this. — [Blinn, Li, Kim, Omar, OOPSLA 2024, DOI 10.1145/3689728, arXiv 2409.00921](https://arxiv.org/pdf/2409.00921)

### Inferences
- If PH2D's scripting language is chosen/designed in-house, choose one with **static types + strong local inference** (Flow beat TS on token tax because of inference): types must be nearly free to write.
- Run the checker continuously on an error-tolerant tree so that one error never blanks out completions/diagnostics for the rest of the file.

### Gaps
- No primary evidence retrieved for "linting reduces bugs" or for "unsafe-line indicators" effects; Ray et al. (FSE 2014) language-quality study and its failed reproduction (Berger et al., TOPLAS 2019) were not re-verified here.

## 4. Immediate feedback / live programming

### Takeaway
Live programming (runtime values shown continuously next to the code) consistently **speeds up tracing and validation** and is preferred by users, and it reduces over-reliance on AI-generated code; but with N=237 CS1 students it produced **no difference in learning outcomes**, and an older experiment found continuous feedback no better than on-demand feedback (both much better than none). Liveness is a speed/validation tool, not a correctness guarantee.

### Cited Findings
- **Projection Boxes (CHI 2020):** runtime values shown in configurable boxes per line; lab study: users found boxes and their configurability useful and were *not distracted* by always-on visualization; wide variation in preferences → configurability is key. — [Lerner, CHI 2020, DOI 10.1145/3313831.3376494](https://cseweb.ucsd.edu/~lerner/pb/)
- **Projection Boxes in CS1 (SIGCSE 2022)**, within-subjects, N=237: no significant difference in post-test learning (Lab F: d=.04; Lab W: d=.01); live-programming students completed *some code-tracing tasks faster* (p<.001 on several tasks); students liked it more and rated it more helpful. — [Huang, Ferdowsi, Selvaraj, Soosai Raj, Lerner, SIGCSE 2022](https://cseweb.ucsd.edu/~lerner/papers/proj-boxes-edu-sigcse2022.pdf)
- **Live programming to validate AI code (Leap)**, between-subjects N=17: LP lowered the cost of inspecting runtime values; participants were more successful at judging AI suggestions, with lower cognitive load in some tasks; all three over-reliance cases were in the No-LP group. — [Ferdowsi et al., arXiv 2306.09541](https://arxiv.org/pdf/2306.09541)
- Follow-on "loop-datavoid problem": unfinished loops produce no useful runtime data → "Focused Live Programming with Loop Seeds". — [Lerner project page](https://cseweb.ucsd.edu/~lerner/pb/)
- **On-demand vs continuous feedback** (Hundhausen & Brown, JVLC 2007): any feedback better than none; *no significant difference* between continuous live and button-press feedback (via secondary citation). — [JVLC 18(6) record](https://dblp.dagstuhl.de/pid/81/3105.html); [secondary account in CodeProber study, arXiv 2502.20540](https://arxiv.org/pdf/2502.20540)
- **Babylonian-style programming:** live *examples* attached to parameterized code give insight into run-time behaviour; evaluation was scenario reproduction + response-time measurement, not a user study. — [Rauch et al., ‹Programming› 2019](https://programming-journal.org/2019/3/9/); [arXiv 1902.00549](https://arxiv.org/pdf/1902.00549)
- **Tanimoto liveness levels:** L1 informative → L2 run on request → L3 auto re-run after edit → L4 live (code replaced in running program); 2013 extension L5 tactically predictive, L6 strategically predictive. — [Tanimoto, LIVE 2013](https://homes.cs.washington.edu/~tanimoto/site/wp-content/uploads/2024/10/LiveProgramming2013.pdf); [Horowitz & Heer survey, arXiv 2303.06777](https://arxiv.org/pdf/2303.06777)
- **Whyline** (ask "why/why not did X happen" about program output): novices with Whyline were 2× faster than experts without it on one task; journal version: 3× success rate, 2× speed vs breakpoint debuggers; across prototypes 20–800% faster debugging. — [Ko & Myers, ICSE 2008](https://faculty.washington.edu/ajko/papers/Ko2008JavaWhyline.pdf); [dissertation](https://faculty.washington.edu/ajko/papers/Ko2008Dissertation.pdf)

### Inferences
- PH2D already has a running game/scene: it can reach **Tanimoto L4** (hot-reload into the running scene) cheaply, which is its biggest structural advantage over generic editors. Pair it with **projection-box-style inline values** for the current entity/frame, and a Whyline-style "why did this value change?" query backed by the ECS/runtime signal log.
- Make the live display **configurable and on-demand-capable** (Hundhausen: continuous isn't better than on-demand; Lerner: preferences vary widely).

### Gaps
- Bret Victor ("Inventing on Principle", "Learnable Programming") and McDirmid ("Usable Live Programming", 2013) are design essays/prototypes without controlled evaluations — treat as influential design arguments, not evidence (not re-fetched in this session).

## 5. Code completion (classic and ML/LLM): what reduces vs induces errors

### Takeaway
Semantically-checked, single-line ML completion gives a modest but solid, large-N productivity gain (6% shorter iteration time at Google); unconstrained LLM generation shows large speedups in toy tasks (55.8%) but a **19% slowdown** for experienced developers on their own real repositories, plus a high rate of insecure suggestions. The error-reducing ingredient is **semantic/type filtering of suggestions**; the error-inducing ingredient is **unvalidated multi-line output**.

### Cited Findings
- Google: 10k+ employees, 3 months, 8 languages vs control: **6% reduction in coding iteration time** with single-line hybrid semantic ML completion; ~3% of new code characters come from accepted ML suggestions; suggestions are semantically checked (reference resolution, method invocation, type check). — [Google Research blog, July 2022](https://research.google/blog/ml-enhanced-code-completion-improves-developer-productivity/)
- Single-line completion acceptance rate cited ~25% (later Meta paper citing Google). — [Meta, "AI-assisted Code Authoring at Scale", arXiv 2305.12050](https://arxiv.org/pdf/2305.12050)
- Copilot RCT (write an HTTP server in JS): treatment **55.8% faster**; heterogeneous effects (larger for less experienced). — [Peng, Kalliamvakou, Cihon, Demirer, arXiv 2302.06590](https://arxiv.org/pdf/2302.06590)
- METR RCT, 16 experienced OSS developers, 246 tasks in their own repos, early-2025 tools (Cursor + Claude 3.5/3.7): AI allowed → **19% longer** completion time; developers *believed* they were ~20% faster. — [METR via Simon Willison](https://simonwillison.net/2025/Jul/12/ai-open-source-productivity/); [The Decoder](https://the-decoder.com/ai-coding-can-make-developers-slower-even-if-they-feel-faster/)
- Copilot security: ~40% of generated programs in CWE-targeted scenarios vulnerable (paper's own tally: 477 valid programs, 44% containing a CWE). — [Pearce et al., "Asleep at the Keyboard?", arXiv 2108.09293](https://arxiv.org/pdf/2108.09293)
- Hazel: LLM completion contextualized with the hole's type and typing context from the language server reduces broken output. — [Blinn et al., OOPSLA 2024](https://arxiv.org/pdf/2409.00921)
- Classic learned completion (Best Matching Neighbours on receiver usage context) beat mainstream alphabetical/type-only completion on relevance; user study in Eclipse. — [Bruch, Monperrus, Mezini, ESEC/FSE 2009](https://hal.archives-ouvertes.fr/hal-01575348)

### Inferences
- PH2D completion: **type-filtered first** (only show members/values that type-check at the cursor), ranked by usage frequency in the project (Bruch-style; cheap, local, deterministic), with snippet templates that insert typed holes for arguments (Hazel/tylr). Any LLM layer must be gated by the same type checker and shown with live values (Leap) to counter over-reliance.

### Gaps
- No controlled study retrieved on snippet/placeholder templates or parameter hints specifically.
- Precision/recall numbers for Bruch 2009 not retrieved.

## 6. Refactoring tools: under-use and discoverability fixes

### Takeaway
Developers perform ~90% of refactorings by hand even when a tool exists, leave ~90% of tool defaults unchanged, and even refactoring-tool authors refactor manually ~89% of the time. The causes are interaction friction (selection, menus, wizards, configuration) and opaque refusals; designs that put the action *at the code* (cues, drag-and-drop, graphical error annotations) measurably reduce errors/configuration time.

### Cited Findings
- Four datasets, >13,000 developers, 240,000 tool-assisted refactorings: **almost 90% of refactorings performed manually**; ~90% of refactoring-tool configuration defaults unchanged; ~40% of tool refactorings occur in batches; commit messages don't reliably indicate refactoring; Rename is by far the most-used tool. — [Murphy-Hill, Parnin, Black, ICSE 2009 / TSE 2012](https://people.scs.carleton.ca/~jeanpier/Fall2021/Topic%204-%20About%20Smells%20and%20Refactoring/5-%20HowWeRefactor-icse09.pdf)
- Of 145 refactorings with tool support observed in Eclipse's own CVS history (the toolsmiths), **89% could not be linked to any tool use**. — [same paper, §3.8](https://people.scs.carleton.ca/~jeanpier/Fall2021/Topic%204-%20About%20Smells%20and%20Refactoring/5-%20HowWeRefactor-icse09.pdf)
- Marking menus and "refactoring cues" proposed to remove selection/menu/wizard friction. — [Murphy-Hill & Black, "High Velocity Refactorings in Eclipse", 2007](https://www.cs.mcgill.ca/~martin/etx2007/papers/1.pdf)
- Graphical refactoring-error annotations → faster, more accurate understanding of why a refactoring was refused. — [Murphy-Hill & Black, TSE 2012](https://works.bepress.com/andrew_black/35)
- Drag-and-drop refactoring was rated more intuitive and reduced configuration time and error rates vs Eclipse's standard tools. — [Lee et al., DNDRefactoring, Illinois (ICSE 2013)](https://ideals.illinois.edu/items/34069/bitstreams/109457/data.pdf)

### Inferences
- PH2D: expose refactorings as **contextual quick-fixes at the cursor** (light-bulb / intention actions) with zero-config defaults (90% never change them), make *Rename* (the one people use) rock-solid and cross-asset (scripts ↔ scene names ↔ components, using `StableId`), and show refusals graphically on the code.

### Gaps
- No controlled study found specifically measuring the "light bulb"/intention-action UI's effect on refactoring frequency (industry design, evidence by analogy only).

## 7. Human error taxonomies mapped to editor features

### Takeaway
Ko & Myers (2005) adapt Reason's human-error model to programming: errors are **chains of cognitive breakdowns** across skill-based (inattention/overattention slips and lapses), rule-based (wrong/misapplied rule) and knowledge-based (wrong plan/model) activity. Each class needs a different editor countermeasure: slips → make the error impossible or instantly visible; rule errors → contextual guidance/quick-fixes; knowledge errors → live values, why-questions, explanations.

### Cited Findings
- Framework: software errors = chains of cognitive breakdowns; skill-based (routine, proceduralized; *inattention* and *overattention* breakdowns), rule-based, and knowledge-based (forming plans, high-level decisions) breakdowns, after Reason. — [Ko & Myers, JVLC 2005](https://faculty.washington.edu/ajko/papers/Ko2004SoftwareErrorsFramework.pdf)
- Example inattention breakdown in the paper: programmer fails to complete an increment statement after an interruption — a classic slip. — [Ko & Myers 2005](https://faculty.washington.edu/ajko/papers/Ko2004SoftwareErrorsFramework.pdf)
- Study: 4 novice Alice programmers observed 90–215 min each; breakdowns implementing expressions (~33%) and in debugging (~18%) were frequent causes (secondary course summary, verify against paper). — [GMU paper summary](https://people.cs.gmu.edu/~tlatoza/teaching/swe795s17/paperSummaries/A%20Framework%20and%20Methodology.pdf)
- The study directly inspired the Whyline. — [Ko & Myers 2005 abstract](https://faculty.washington.edu/ajko/papers/Ko2004SoftwareErrorsFramework.pdf)

### Inferences (proposed mapping — design inference, not measured)
| Error class | Typical programming instance | Editor countermeasure (evidence pointer) |
|---|---|---|
| Skill slip (typo, unclosed delimiter, forgotten increment) | syntax errors, misspelled identifiers | auto-closing/obligations (tylr), type-filtered completion (Google), frames/templates (Stride) |
| Skill lapse (forgot to update a copy, interrupted edit) | copy-paste inconsistency | rename/multi-cursor, local history + selective undo (Azurite), unsaved-state indicators |
| Rule-based mistake (wrong API use, wrong type) | wrong argument, null misuse | static types live (Gao: ~15%), quick-fixes, hand-written error explanations (Becker) |
| Knowledge-based mistake (wrong mental model of runtime) | off-by-one, wrong frame order | live values / projection boxes (Lerner), Whyline-style why-queries (Ko), examples (Babylonian) |
| Over-reliance on generated code | accepting buggy LLM output | type-gate + live values on suggestions (Leap, Hazel Assistant) |

### Gaps
- No quantitative study found that measures *which editor feature* removes *which* Reason class; the mapping is an inference.

## 8. Cognitive load / comprehension: syntax highlighting, semantic highlighting, folding, fonts, ligatures

### Takeaway
Evidence for syntax highlighting is weak and mixed: the largest experiment (N=390 novices) found **no comprehension benefit**; a tiny eye-tracking study (N=10) found faster completion that shrinks with experience. Fonts, ligatures, semantic highlighting and folding have essentially **no peer-reviewed effect evidence** found — folklore.

### Cited Findings
- 390 undergraduates, Java, within-subject highlight/no-highlight: **no evidence that syntax highlighting improves novice comprehension**; authors suggest the colour channel might be better used for other feedback. — [Hannebauer, Hesenius, Gruhn, EMSE 23(5) 2018, DOI 10.1007/s10664-017-9579-0](https://unpaywall.org/10.1007%2FS10664-017-9579-0)
- N=10, eye tracker, within-subjects: highlighting **significantly improves task completion time**, effect weakens with experience. — [Sarkar, PPIG 2015](https://advait.org/files/sarkar_2015_syntax_colouring.pdf)
- Eye-tracking: fixations/regressions higher for black-and-white code but not significantly; students prefer colour. — [Beelders & du Plessis, JEMR 2015](https://bop.unibe.ch/JEMR/article/view/2429)
- Sonic Pi study: highlighting improves completion time in writing and debugging tasks. — [Dimitri, PPIG 2015](https://ppig.org:443/files/2015-PPIG-26th-Dimitri.pdf)

### Inferences
- Following Hannebauer's suggestion, PH2D could spend colour on **semantic signals that prevent errors** (type/kind of identifier, unresolved name, mutable vs immutable, entity references) rather than lexical categories — a defensible design hypothesis, not evidence.

### Gaps
- No studies found on ligatures, code fonts, semantic (vs lexical) highlighting, or code folding effects on errors. Treat all claims there as folklore.

## 9. Undo / history safety and "never lose work"

### Takeaway
Selective undo of fine-grained edits makes backtracking **2× faster** than conventional undo/VCS in a lab study; programmers backtrack often and mostly by hand. Undo trees/local history are product practice with little controlled evidence beyond Azurite.

### Cited Findings
- Azurite (Eclipse plug-in): selective undo of fine-grained code changes, even when they're out of the linear undo stack and not in VCS; lab study: programmers used it successfully and were **twice as fast** as with conventional features. — [Yoon & Myers, ICSE 2015](https://kilthub.cmu.edu/articles/journal_contribution/Supporting_Selective_Undo_in_a_Code_Editor/6470474); [Azurite project](https://www.cs.cmu.edu/~azurite/index.html)
- Longitudinal backtracking study exists (VL/HCC 2014, pp. 101–108) and an exploratory study of backtracking strategies (CHASE 2012); numeric findings not retrieved here. — [Azurite research page](https://www.cs.cmu.edu/~azurite/research.html)

### Inferences
- PH2D already has a DIFF-based undo queue (`ProjectState`); the script editor should log fine-grained edits per file (persisted, survives crash) and offer **region-selective undo** plus a timeline of edits — directly Azurite.

### Gaps
- Frequency numbers from Yoon & Myers 2014 not retrieved. Vim undo-tree, JetBrains Local History and autosave are well-known product features but no controlled studies were found in this session.

## 10. Recent (2023–2026) editor-innovation papers

### Takeaway
The 2023–2026 research frontier is (a) **gradual/fluid structure editing** that keeps text-like input (teen/tall tylr 2023/2025, Pantograph 2025), (b) **error-tolerant typing as an editor service** (Hazel POPL 2024) and its use to **ground LLMs** (OOPSLA 2024, ChatLSP), and (c) **live programming as the validation layer for AI code** (Leap). Empirical studies remain small (N=9–17).

### Cited Findings
- teen tylr (VL/HCC 2023), tall tylr (OOPSLA 2025), Pantograph (POPL 2025), Hazel total marking (POPL 2024), Hazel Assistant/ChatLSP (OOPSLA 2024), Leap (2023) — see sections 1, 3, 4, 5 for sources.
- CHI 2025 "code shaping": editing code by sketching free-form annotations on code and console output, interpreted by an AI; 3-stage study with 18 programmers. — [arXiv 2502.03719](https://arxiv.org/html/2502.03719v2)
- Property probes (CodeProber) in an educational setting, 2025. — [arXiv 2502.20540](https://arxiv.org/pdf/2502.20540)

### Inferences
- "Beyond state of the art" for PH2D is plausibly the **combination** nobody ships yet in a game editor: text-first input + always-valid typed model with holes + L4 liveness into the running scene + type-gated completion/LLM + Whyline-style why-queries over the runtime log.

### Gaps
- A systematic sweep of CHI/UIST/ICSE/FSE 2025–2026 proceedings was not possible within this session's budget; there may be relevant 2025–2026 papers (e.g., on inline diagnostics or next-edit prediction) not captured here.
