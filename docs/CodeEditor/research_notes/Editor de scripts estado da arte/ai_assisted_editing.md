# AI-assisted code editing: state of the art (Oct 2026) and evidence on speed and errors

> Research notes for the PH2D integrated script editor. Compiled 2026-10-15. Every finding is dated and
> tagged **[independent]** (academic / third-party research), **[vendor]** (the claim comes from the
> company that sells the tool or a related service), or **[survey]** (self-reported). Where only
> secondary coverage was reachable, that is said explicitly.

## 1. Interaction patterns and their leaders (what is new in 2025–2026)

### Takeaway
The field has split into two different families. (a) **Low-latency, small, task-specific models in the
edit loop**: ghost-text completion, which then grew into *next-edit prediction* (Cursor Tab, Copilot
NES, Zed Zeta, Sweep). (b) **Large-model agents** that plan, edit many files, run things and iterate
(Cursor Agent/Composer, Copilot agent, Claude Code, Zed agent panel via ACP, Roblox Assistant). What
was new in 2025–26: custom next-edit models trained with RL on accept/reject data, explicit "plan →
human approves → build" modes, parallel agents in isolated worktrees, open protocols (MCP, ACP), and
provenance specs (Agent Trace).

### Cited Findings
**Next-edit prediction (the 2025 successor to ghost text)**
- Zed launched "edit prediction" on **2025-02-13**, powered by **Zeta**, an open model derived from
  **Qwen2.5-Coder-7B**, fine-tuned with LoRA SFT then DPO. Its open dataset was bootstrapped from ~50
  synthetic examples, then ~400 high-quality team examples, plus ~150 hand-picked DPO examples **[vendor]**
  — [Zed blog](https://zed.dev/blog/edit-prediction)
- Zeta UX: `Tab` accepts a predicted edit and repeated `Tab` applies follow-up edits. **When LSP
  completions are visible, the prediction is NOT previewed until the user holds `Alt/Option`**, so the
  two suggestion sources never fight over `Tab`. On Linux the default is `alt-l`, because window
  managers reserve `alt-tab` **[vendor, 2025-02-13]** — [Zed blog](https://zed.dev/blog/edit-prediction)
- Zed later shipped a second-generation model ("Zeta2", open-weight). This is documented only in
  secondary sources and a conference talk title. Zed docs list Zeta as the default provider, with
  Copilot, Supermaven and Codestral as alternatives **[secondary]** —
  [Zed docs mirror](https://git.secluded.site/zed/blob/v0.224.6/docs/src/ai/overview.md),
  [AI Engineer org page](https://ai.engineer/orgs/zed)
- Cursor Tab predicts the user's *next action*, runs on every user action (>400 M requests/day), and
  shows a suggestion **only when confident enough**. Cursor trains it with **online RL (policy
  gradient)** on accept/reject signals, rolling out new models several times a day, so that the model
  *itself* stops producing weak suggestions. This replaces scoring and filtering suggestions after
  generation **[vendor]** — [Cursor blog "Tab RL"](https://cursor.com/pt-BR/blog/tab-rl)
- **GitHub Copilot Next Edit Suggestions (NES)**, 2025-11-20. Frontier models were "too slow or too low
  quality" for in-editor use, so GitHub trained a small custom model (SFT, then RL with an LLM grader).
  **Training on pull-request data failed**: those models were over-cautious and worse than a vanilla
  LLM, because PRs show only final states, have no timing, and have few "don't edit" examples. Curated
  **editing sessions** worked better **[vendor]** —
  [GitHub blog](https://github.blog/ai-and-ml/github-copilot/evolving-github-copilots-next-edit-suggestions-through-custom-model-training/)
- NES A/B numbers against production. November release: **shown rate −24.5 %, acceptance +26.5 %, hide
  rate −25.6 %**. In other words, *fewer* suggestions, accepted *more* often. Early feedback called the
  model "too eager", and the cure was adding "no-edit" training samples and raising the intervention
  threshold **[vendor]** —
  [GitHub blog](https://github.blog/ai-and-ml/github-copilot/evolving-github-copilots-next-edit-suggestions-through-custom-model-training/)
- NES is reported GA in VS Code and Visual Studio and in preview for JetBrains **[secondary]** —
  [agentpatterns.ai](https://agentpatterns.ai/tools/copilot/next-edit-suggestions/)
- Sweep open-sourced **Sweep Next-Edit 1.5B** (Apache-2.0, Qwen2.5-Coder base, 8k context). Claims:
  "<500 ms locally on a laptop", sub-100 ms warm on its own TensorRT-LLM server, and better than models
  4× its size on next-edit benchmarks **[vendor]** —
  [Sweep blog](https://blog.sweep.dev/posts/oss-next-edit),
  [Hugging Face](https://huggingface.co/sweepai/sweep-next-edit-1.5B)

**Agents and plan/review modes**
- **Cursor 2.0** (2025-10-29) introduced the in-house **Composer** model ("4× faster than similarly
  intelligent models", per Cursor), a dedicated multi-file diff-review interface, and up to 8 parallel
  agents, each in its own git worktree or remote machine **[secondary coverage of vendor claims]** —
  [Shuttle blog](https://www.shuttle.dev/blog/2025/10/31/cursor-2.0),
  [CometAPI summary](https://www.cometapi.com/cursor-2-0-what-changed-and-why-it-matters/)
- **Roblox Assistant** (2026 docs) offers **Planning Mode** (`/plan`). It produces a Markdown plan the
  user can edit, and "Plans never auto-execute or time out into acceptance": work runs only when the
  user clicks *Build*. It also writes, edits and explains scripts, edits instances in bulk, and has a
  screen-capture sub-agent that looks at the viewport in Edit and Play mode **[vendor docs]** —
  [Roblox Assistant guide](https://create.roblox.com/docs/assistant/guide)
- Agent protocols: **ACP (Agent Client Protocol)**, created by Zed (Apache-2.0, repo June 2025, modelled
  on LSP), lets any external agent (Claude Code, Codex, Gemini CLI…) run inside the editor's UI.
  JetBrains adopted it in late 2025 and launched an ACP Registry in Jan 2026 **[vendor/secondary]** —
  [zed.dev/acp](https://zed.dev/acp),
  [The Register 2025-10-07](https://www.theregister.com/2025/10/07/jetbrains_acp_vs_code/),
  [JetBrains blog 2026-02-19 (JA)](https://blog.jetbrains.com/ja/blog/2026/02/19/acp-agent-client-protocol-jetbrains-ide/)
- **Provenance**: Cursor proposed **Agent Trace** (v0.1.0-draft, Jan 2026), JSON trace records that tie
  line ranges to human / AI / mixed / unknown contributors and to a VCS revision, with content hashes so
  attribution survives code moves. Thoughtworks Radar listed it as "Assess" in Apr 2026 **[vendor spec +
  independent radar]** — [agent-trace.dev](https://agent-trace.dev/),
  [InfoQ 2026-02](https://www.infoq.com/news/2026/02/agent-trace-cursor),
  [Thoughtworks Radar](https://www.thoughtworks.com/radar/platforms/agent-trace)

**How people actually use assistants (usability research)**
- Barke, James & Polikarpova, "Grounded Copilot" (OOPSLA 2023, Distinguished Paper; 20 participants):
  use is **bimodal**. In *acceleration* mode the programmer knows what to write and the AI saves
  keystrokes. In *exploration* mode the programmer is unsure and uses the AI to explore options. The two
  modes need different UI **[independent]** —
  [arXiv 2206.15000](https://arxiv.org/pdf/2206.15000),
  [MSR page](https://www.microsoft.com/en-us/research/publication/grounded-copilot-how-programmers-interact-with-code-generating-models/)
- Liang, Yang & Myers, ICSE 2024 (survey of 410 developers): the main motivations are fewer keystrokes,
  finishing faster and recalling syntax; brainstorming is much less common. The main reasons for *not*
  using the tools are output that misses functional or non-functional requirements, and difficulty
  *controlling* the tool. The authors recommend "minimal cognitive effort interactions" **[independent]** —
  [ICSE 2024 listing](https://conf.researchr.org/details/icse-2024/icse-2024-research-track/3/A-Large-Scale-Survey-on-the-Usability-of-AI-Programming-Assistants-Successes-and-Cha),
  [arXiv 2303.17125](https://www.arxiv.org/pdf/2303.17125)

### Inferences
- The industry direction since 2025 is **fewer, better-timed suggestions** (NES: shown −24.5 %,
  accepted +26.5 %; Cursor: "only when confident"). Over-eager suggestions are a measured annoyance and
  distraction. The quality bar is "silent unless sure".
- Every successful pattern keeps a **single, explicit accept gesture** (Tab, Build, accept diff).
  Zed's choice to hide the prediction while LSP completions are open is a concrete precedent for
  arbitrating between a deterministic source (LSP) and a probabilistic one (model).
- The acceleration/exploration split maps well onto a simple editor. *Acceleration* is inline and
  instant (completion / next edit). *Exploration* is a separate, explicitly invoked surface (ask /
  explain / plan).

### Gaps
- No primary 2026 sources were fetched for the agent modes of GitHub Copilot, Claude Code, Windsurf
  Cascade, JetBrains Junie or the Zed agent panel; their feature sets are known only from secondary
  mentions (e.g. ACP agent lists).
- No independent measurement of inline-chat (Cmd-K) edit-with-diff vs. agent mode was found.
- No independent evaluation of next-edit prediction's effect on speed or errors was found. All numbers
  are vendor A/B acceptance metrics, which measure *acceptance*, not correctness or task time.
- No study isolating AI-written tests or AI code review *as features inside an editor* was found
  (CodeRabbit's data, §2, is about AI-authored PRs, not about AI reviewers' effectiveness).

## 2. Evidence: real effect on speed and errors, and the conditions that make AI help or hurt

### Takeaway
Controlled studies conflict because conditions differ. Help shows up on **well-specified, greenfield
or unfamiliar tasks** (Copilot HTTP server +55.8 %; Google enterprise task ~21 %). Harm or no gain
shows up for **experts in large, familiar, high-standard codebases** (METR 2025: −19 %). At org scale,
AI raises throughput but **lowers delivery stability** (DORA 2024/2025) and increases duplication,
churn, review load and defect density (GitClear, Faros, CodeRabbit). Perceived speedup is consistently
much larger than measured speedup. For **learners**, delegating code generation to AI measurably hurts
comprehension and debugging skill (Anthropic RCT 2026).

### Cited Findings
**Controlled trials (speed)**
- **METR RCT** (published 2025-07-10; data Feb–Jun 2025). 16 experienced OSS developers worked on 246
  real issues in their own repos (avg 22k+ stars, 1M+ LOC), mainly with Cursor Pro + Claude 3.5/3.7
  Sonnet. Tasks took **19 % longer with AI** (CI +2 % to +39 %). Developers predicted a 24 % speedup
  and still *believed* afterwards they had been 20 % faster. METR suggests AI does worse in settings
  with "very high quality standards or many implicit requirements" (docs, test coverage, linting), and
  says AI "may help less experienced developers or those working in unfamiliar codebases"
  **[independent]** —
  [METR 2025](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/)
- **METR follow-up** (2026-02-24). Late-2025 data: returning developers −18 % time, i.e. a speedup (CI
  −38 % to +9 %); new recruits −4 % (CI −15 % to +9 %). METR calls this data **unreliable**: developers
  refused to work without AI, withheld tasks they didn't want to do unaided (30–50 % skipped some),
  pay fell from $150/h to $50/h, and time tracking broke when people ran agents in parallel. METR
  believes developers are "likely … more sped up from AI tools now", but the data is "only very weak
  evidence" for the size, and probably a lower bound **[independent]** —
  [METR 2026](https://metr.org/blog/2026-02-24-uplift-update/)
- **Peng, Kalliamvakou, Cihon & Demirer** (arXiv 2302.06590, Feb 2023): the Copilot group finished a
  "write an HTTP server in JavaScript" task **55.8 % faster** than control. This was a single,
  well-specified greenfield task, and the authors are GitHub / Microsoft researchers
  **[vendor-affiliated research]** — [arXiv](https://arxiv.org/pdf/2302.06590),
  [MSR](https://www.microsoft.com/en-us/research/publication/the-impact-of-ai-on-developer-productivity-evidence-from-github-copilot)
- **Google enterprise RCT** (arXiv 2410.12944, ICSE-SEIP 2025): 96 Google engineers, three internal AI
  features, one complex enterprise task. Best estimate **~21 % faster**, with a "large" confidence
  interval. The authors warn the result may not transfer to other tools or periods
  **[vendor-affiliated research]** — [arXiv](https://arxiv.org/html/2410.12944v2)
- **Anthropic RCT on skill formation** (2026-01-29): 52 engineers learning the Trio library. The AI
  group was ~2 min faster, which is **not statistically significant**. Its follow-up quiz averaged
  **~50 % vs ~67 %** for hand-coders, a 17 point gap, with the largest gap on **debugging**. Those who
  used AI for *conceptual questions* scored ≥65 %; those who *delegated code generation* scored <40 %
  **[vendor-run research; numbers via secondary coverage]** —
  [Anthropic](https://www.anthropic.com/research/AI-assistance-coding-skills),
  [InfoQ 2026-02](https://infoq.com/news/2026/02/ai-coding-skill-formation/)

**Organisation-scale and code-quality evidence**
- **DORA 2025** (2025-09-23; ~5,000 respondents): 90 % use AI at work and >80 % believe it raised their
  productivity. Unlike 2024, AI adoption now correlates *positively* with throughput, but it **still
  correlates negatively with delivery stability**. "AI doesn't fix a team; it amplifies what's already
  there." Strong controls (automated testing, mature version control, fast feedback loops) are named as
  what keeps acceleration from exposing weaknesses downstream. 30 % report little or no trust in
  AI-generated code **[survey, Google]** —
  [Google Cloud blog](https://cloud.google.com/blog/products/ai-machine-learning/announcing-the-2025-dora-report),
  [full PDF](https://services.google.com/fh/files/misc/2025_state_of_ai_assisted_software_development.pdf)
- DORA 2024 (prior year): each +25 % AI adoption was associated with −1.5 % throughput and **−7.2 %
  delivery stability** **[survey; via secondary]** —
  [TechTarget](https://www.techtarget.com/searchsoftwarequality/news/366631712/Google-DORA-Software-delivery-caught-up-to-AI-coding-tools)
- DORA 2025 groups teams into seven clusters. Reportedly only two ("pragmatic performers",
  "harmonious high-achievers") raise throughput without a higher change-failure rate **[secondary
  reading of the report]** —
  [Rob Bowley 2025-10-01](https://blog.robbowley.net/2025/10/01/dora-2025-ai-assisted-dev-report-some-benefit-most-dont/)
- **GitClear 2025** (~211 M changed lines, 2020–2024): duplicated code blocks rose **~8×** in 2024;
  moved (refactored) lines fell from ~25 % (2021) to **<10 %** (2024), and copy/paste exceeded moved
  code for the first time; churn went from 3.3 % (2021) to 5.7 % (2024). The data is correlational
  (GitClear sells analytics) **[vendor-independent of AI tools, correlational]** —
  [LeadDev](https://leaddev.com/technical-direction/how-ai-generated-code-accelerates-technical-debt),
  [i-programmer](https://www.i-programmer.info/news/105-artificial-intelligence/17871-gitclear-reveals-ais-negative-impact-on-code-quality.html)
- **Faros AI** telemetry (Jul 2025; >10k devs, 1,255 teams): high-adoption teams completed +21 % tasks
  and merged +98 % PRs, but **review time rose +91 %**, PR size +154 % and bugs per developer +9 %;
  "any correlation … evaporates at the company level". The data is correlational, and Faros's own FAQ
  gives different numbers (+51.3 % PR size, +54 % bugs), so the figures are inconsistent
  **[vendor analytics]** —
  [Faros blog](https://www.faros.ai/blog/translating-ai-powered-developer-velocity-into-business-outcomes),
  [Faros FAQ](https://www.faros.ai/comparison/github-copilot-vs-codeium)
- **CodeRabbit** (2025-12-17; 470 OSS PRs): AI-coauthored PRs had **10.83 issues vs 6.45** (≈1.7×),
  with 1.4× more critical issues, ~+75 % logic/correctness issues, 1.5–2× security issues and >3×
  readability issues. The "human" label is not guaranteed, and CodeRabbit sells AI review
  **[vendor]** —
  [CodeRabbit report](https://coderabbit.ai/blog/state-of-ai-vs-human-code-generation-report),
  [The Register](https://www.theregister.com/2025/12/17/ai_code_bugs/)

**Security**
- Pearce et al., "Asleep at the Keyboard?" (2021; CACM Feb 2025): 89 CWE Top-25 scenarios, 1,689
  Copilot programs, **~40 % vulnerable**. This measured the 2021 Copilot **[independent]** —
  [arXiv 2108.09293](https://arxiv.org/pdf/2108.09293)
- Veracode GenAI Code Security Report (2025-07-30): 80 tasks, >100 LLMs. Code introduced OWASP-Top-10
  vulnerabilities in **45 %** of cases, failing to prevent XSS in 86 % and log injection in 88 %.
  "Syntax accuracy has improved, security has stayed flat" over model generations **[vendor]** —
  [Veracode press release](https://www.veracode.com/press-release/ai-generated-code-poses-major-security-risks-in-nearly-half-of-all-development-tasks-veracode-research-reveals/)
- Package hallucination (Spracklen et al., USENIX Security 2025): **19.7 %** of 2.23 M samples
  referenced a non-existent package (open models 21.7 %, commercial 5.2 %), and 43 % of hallucinated
  names recurred on every one of 10 reruns, which makes them predictable attack targets. A 2026 preprint
  finds 4.6–6.1 % on frontier models from Oct 2025 – Mar 2026 **[independent; 2026 one is a
  preprint]** — [CSA note](https://labs.cloudsecurityalliance.org/research/csa-research-note-slopsquatting-ai-supply-chain-20260419/),
  [arXiv 2605.17062](https://arxiv.org/pdf/2605.17062)

**Trust (survey)**
- Stack Overflow 2025 (>49k respondents): 84 % use or plan to use AI (76 % in 2024). **Trust in
  accuracy is ~33 %** (43 % in 2024), **46 % actively distrust** (31 % in 2024), and 3 % "highly trust".
  The top frustration is "almost right, but not quite" (66 %), and 45.2 % say debugging AI code takes
  longer. Favourability is 60 % (72 % in 2024, 77 % in 2023). One outlet reports 29 %/40 % for trust,
  a conflict noted here **[survey]** —
  [InfoWorld](https://infoworld.com/article/4031673/ai-use-among-software-developers-grows-but-trust-remains-an-issue-stack-overflow-survey.html),
  [ShiftMag](https://shiftmag.dev/stack-overflow-survey-2025-ai-5653/),
  [ADTmag](https://adtmag.com/blogs/watersworks/2026/01/stack-overflow-survey.aspx)

### Inferences
- **Conditions where AI helps:** the task is well specified and self-contained (greenfield, small
  scope); the domain is unfamiliar (new API or library, *if* the goal is output and not learning);
  strong automatic verification exists downstream (tests, version control, fast feedback, as in DORA).
  **Where it hurts:** large implicit-requirement codebases, expert users already fast, review-bound
  pipelines, "almost right" output that costs debugging time, security-sensitive APIs, and **learners**
  who delegate generation.
- The bottleneck moves from writing to **reviewing and verifying**: Faros +91 % review time, METR's
  review/correction overhead, Stack Overflow's "almost right". An editor that lowers *verification*
  cost captures the benefit; one that only lowers *typing* cost risks importing the defects.
- The perception gap (METR: felt +20 %, measured −19 %) means user satisfaction or acceptance rate is
  **not** a valid success metric for PH2D's AI features. Measure task time and defects directly.
- Game scripting for PH2D users is closer to the "unfamiliar API, small self-contained script" regime
  (where AI helps) than to the METR regime. But the users are partly **learners**, which is exactly the
  regime where Anthropic's RCT shows delegation hurts.

### Gaps
- DORA 2025's exact list of the seven "AI Capabilities" was not retrievable as text (it is an image in
  the blog). Only "clear AI stance", "healthy data ecosystems" and "user-centric focus" are confirmed.
- No independent RCT on game-scripting or visual / creative-tool scripting was found.
- No 2025–26 independent RCT of agent-mode (vs. completion) productivity with reliable data; METR's
  attempt failed on selection effects.
- Faros figures are internally inconsistent across its own pages and could not be reconciled.

## 3. Error-limiting design: diff-first review, small reversible edits, grounding in type checker/LSP, automatic verification, provenance

### Takeaway
The strongest evidence-backed levers are: **constraining generation with the type system**
(compilation errors more than halved, PLDI 2025); **explicit human approval of plans and diffs** (no
auto-accept); **automatic compiler/test feedback** before a change is shown; **suggesting less but
better** (NES); and **provenance records** so AI-origin code can be audited later (Agent Trace, still a
draft). Natural-language explanations alone do *not* reliably reduce error-fixing time.

### Cited Findings
- **Type-constrained decoding** (Mündler, He, Wang, Sen, Song, Vechev; PLDI 2025, arXiv 2504.09246):
  prefix automata plus a search over inhabitable types enforce well-typedness *during* generation. This
  **reduces compilation errors by more than half** and improves functional correctness on HumanEval /
  MBPP synthesis, translation and repair. It holds across model sizes, including >30B open models, and
  is implemented for TypeScript **[independent]** —
  [arXiv](https://arxiv.org/pdf/2504.09246),
  [PLDI listing](https://pldi25.sigplan.org/details/pldi-2025-papers/25/Type-Aware-Constraining-for-LLM-Code-Generation)
- Plans that never auto-apply: Roblox Planning Mode "Plans never auto-execute or time out into
  acceptance"; the user edits the plan and clicks *Build*. Asset and mesh generation shows previews
  before insertion **[vendor docs, 2026]** —
  [Roblox Assistant guide](https://create.roblox.com/docs/assistant/guide)
- A dedicated multi-file diff-review UI was a headline feature of Cursor 2.0 (Oct 2025)
  **[secondary]** — [Shuttle blog](https://www.shuttle.dev/blog/2025/10/31/cursor-2.0)
- Agent isolation: Cursor 2.0 runs each parallel agent in its own git worktree or remote machine, so a
  change is reversible by discarding the workspace **[secondary]** —
  [Shuttle blog](https://www.shuttle.dev/blog/2025/10/31/cursor-2.0)
- Closed-loop verification inside the engine: the Roblox Studio MCP server (Feb 2026 update) added
  `get_console_output`, `start_stop_play`, `run_script_in_play_mode` and `get_studio_mode`. Agents can
  then run the game, read logs and fix, rather than hand over unverified code **[secondary coverage of a
  vendor update]** —
  [MoguraVR](https://www.moguravr.com/roblox-studio-mcp-server-agentic-update/),
  [Roblox MCP docs](https://create.roblox.com/docs/en-us/studio/mcp.md)
- Suppressing low-value suggestions improves outcomes: NES added "no-edit" training samples and raised
  thresholds after "too eager" feedback; the hide rate fell 25.6 % **[vendor]** —
  [GitHub blog](https://github.blog/ai-and-ml/github-copilot/evolving-github-copilots-next-edit-suggestions-through-custom-model-training/)
- Provenance: Agent Trace (Jan 2026 draft) records human / AI / mixed per line range, linked to a
  commit, with content hashes that survive refactors; storage can be files or git notes
  **[vendor spec]** — [InfoQ](https://www.infoq.com/news/2026/02/agent-trace-cursor),
  [agent-trace.dev](https://agent-trace.dev/)
- DORA 2025: automated testing, mature version control and fast feedback loops are what stop AI
  acceleration from becoming instability **[survey]** —
  [Google Cloud blog](https://cloud.google.com/blog/products/ai-machine-learning/announcing-the-2025-dora-report)
- Hallucinated identifiers and packages are frequent and *repeatable* (19.7 % of samples; 43 % of the
  invented names reappear on every one of 10 reruns). Grounding names against a real symbol table or
  registry is necessary **[independent]** —
  [CSA note](https://labs.cloudsecurityalliance.org/research/csa-research-note-slopsquatting-ai-supply-chain-20260419/)

### Inferences
- For PH2D, which controls its own scripting language, API surface and runtime, **constraining model
  output to the editor's own parser and type checker is unusually cheap and strong**. The language and
  the API surface are closed and known, which makes constrained decoding far easier than for TypeScript.
  An AI suggestion that does not type-check against the live API should never be shown.
- Natural verification loop: suggestion → parse/type-check (instant) → optional headless run or test
  of the script → *then* show the diff. Show the diff only after the machine has verified what it can.
- Small reversible edits fit the existing PH2D undo model (one diff-based undo queue, per CLAUDE.md
  §5.1 "Editor / shell"). Each AI application should be **one undo step**, visibly labelled.
- Provenance can be cheap in a closed tool: tag AI-inserted ranges in the script document and show a
  subtle gutter mark until the user edits or "adopts" them. Agent Trace is a candidate format, but it is
  a draft (v0.1.0) and should not be depended on yet.

### Gaps
- No controlled study was found that measures whether diff-first review *actually* reduces accepted
  defects vs. inline auto-apply (the practice is widespread; the evidence is anecdotal or vendor).
- No user study of provenance marking (does a visible AI mark change review behaviour?) was found.
- No measurement of LSP / type-aware *completion* (as opposed to whole-function generation) on error
  rates for 2025–26 commercial tools.

## 4. Local vs cloud models for editors; latency budgets, privacy; MCP and ACP

### Takeaway
Inline completion and next-edit prediction are now done by **small (1.5B–7B) specialised open models**
with latency targets of **~100–200 ms p50 and <500 ms p90**. These models can run locally (a 1.5B Q4
needs ~1.2 GB of VRAM), which removes the privacy and network dependency for the inline tier. Chat,
explain and agent tiers still use large cloud models. **MCP** (tools and context for models, now under
the Linux Foundation's Agentic AI Foundation) and **ACP** (plugging external agents into an editor UI,
LSP-like) are the de-facto integration standards as of 2026.

### Cited Findings
- Zeta latency targets: **p50 < 200 ms, p90 < 500 ms**, via speculative decoding with n-gram matching
  against the input text (edits mostly copy the input) and edge-routed GPUs (Baseten, NA + EU)
  **[vendor, 2025-02]** — [Zed blog](https://zed.dev/blog/edit-prediction)
- Sweep Next-Edit 1.5B: "<500 ms on a laptop"; sub-100 ms warm on a server with n-gram speculation and
  early cancellation; Apache-2.0 GGUF; community Q4_K_M ≈ 1.2 GB VRAM **[vendor + community]** —
  [Sweep blog](https://blog.sweep.dev/posts/oss-next-edit),
  [HF](https://huggingface.co/sweepai/sweep-next-edit-1.5B),
  [llmrun listing](https://llmrun.dev/model/xenova-sweep-next-edit-1-5b)
- GitHub found frontier models either too slow (large) or too weak (small, generic) for the edit loop,
  and trained a task-specific small model **[vendor, 2025-11]** —
  [GitHub blog](https://github.blog/ai-and-ml/github-copilot/evolving-github-copilots-next-edit-suggestions-through-custom-model-training/)
- Both open next-edit models (Zeta and Sweep) are fine-tunes of **Qwen2.5-Coder**, which is the de-facto
  base for small open code models **[vendor]** — [Zed blog](https://zed.dev/blog/edit-prediction),
  [HF](https://huggingface.co/sweepai/sweep-next-edit-1.5B)
- Zed supports pluggable prediction providers: Zeta (default), Copilot, Supermaven, Codestral
  **[secondary]** —
  [Zed docs mirror](https://git.secluded.site/zed/blob/v0.224.6/docs/src/ai/overview.md)
- **MCP** was donated by Anthropic to the **Agentic AI Foundation** (a directed fund of the Linux
  Foundation, co-founded with Block and OpenAI) on **2025-12-09**. At that point it had >10,000 active
  public servers, 97 M monthly SDK downloads, and first-class clients in ChatGPT, Cursor, Gemini,
  Copilot and VS Code **[vendor announcement]** —
  [Anthropic](https://www.anthropic.com/news/donating-the-model-context-protocol-and-establishing-of-the-agentic-ai-foundation)
- **ACP**: Apache-2.0, by Zed, modelled on LSP; adopted by JetBrains (late 2025; registry Jan 2026) and
  Google; one source says it was the headline feature of Zed 1.0 (2026-04-29, unverified)
  **[vendor/secondary]** — [zed.dev/acp](https://zed.dev/acp),
  [alternativeto](https://alternativeto.net/news/2025/10/jetbrains-will-collaborate-with-zed-to-bring-the-agent-client-protocol-acp-to-their-ides),
  [rywalker](https://rywalker.com/research/zed-agent-client-protocol)
- Game engines expose themselves via MCP: **Roblox Studio has a built-in MCP server** that lets external
  AI clients explore the data model, write scripts, run Luau and test in play mode; Assistant also
  supports bring-your-own-key for external LLMs **[vendor docs + secondary]** —
  [Roblox MCP docs](https://create.roblox.com/docs/en-us/studio/mcp.md),
  [MoguraVR](https://www.moguravr.com/roblox-studio-mcp-server-agentic-update/)

### Inferences
- A **two-tier architecture** is the evidence-aligned choice. (1) An optional *local* small model (or
  none) for inline next-edit, under a ~200 ms budget and always subordinate to the deterministic LSP.
  (2) An optional, explicitly invoked large model for explain, plan or generate. Tier 2 can be cloud or
  BYO-key and must never block typing.
- For PH2D (Rust, desktop, privacy-sensitive creative work), exposing the editor and engine as an
  **MCP server** (like Roblox Studio) gives users "bring your own agent" (Claude Code, Codex…) *without*
  PH2D shipping or maintaining a model. Speaking **ACP** would let an external agent appear inside
  PH2D's own panel. Both are open standards with broad adoption, so they add little lock-in risk.
- Because PH2D owns the language, a tiny local model fine-tuned on PH2D-script edit sessions (GitHub's
  lesson: *editing sessions*, not final diffs) is feasible later. It is not needed for v1.

### Gaps
- No independent latency-vs-acceptance curve was found (i.e. at what ms delay suggestions stop being
  accepted); the 200/500 ms targets are vendor targets, not measured thresholds.
- No source fetched on Codestral or Qwen3-Coder local performance numbers in 2026; JetBrains Mellum was
  not covered by the results.
- ACP governance (foundation or single vendor) was not confirmed.

## 5. Domain-specific assistants in game engines and creative tools

### Takeaway
The leaders (Roblox Assistant, Unity AI) converged in 2025–26 on **an assistant that knows the engine's
data model and acts on it**, not merely a code chat. Roblox leads, with plan → approve → build,
runtime access (play, logs, viewport capture) and an MCP server. Unity replaced Muse with Unity AI
(Assistant + Generators) in 6.2. Godot core ships **no** built-in AI; that space is filled by
third-party plugins and AI-native forks. Independent evidence of outcomes in these tools is essentially
absent; everything is vendor documentation or marketing.

### Cited Findings
- **Roblox Assistant** (docs ©2026): writes, edits and *explains* scripts (including user code); bulk
  instance edits; asset search with **previews before insertion**; Planning Mode with editable plans
  that never auto-execute; a viewport screen-capture sub-agent in Edit and Play mode. The docs admit
  generated scripts "might not function flawlessly" **[vendor docs]** —
  [Roblox Assistant guide](https://create.roblox.com/docs/assistant/guide)
- Roblox Planning Mode reportedly arrived in April 2026, with an agent that "plans, implements, runs,
  captures screenshots, reads logs, and iterates" **[secondary]** —
  [rowatcher](https://rowatcher.com/news/roblox-ai-tools-in-2026-what-s-actually-changed-for-developers)
- Roblox Studio MCP server: built into Studio; the Feb 2026 update added play-mode run and console-log
  tools; BYO-key for external LLMs **[vendor + secondary]** —
  [Roblox MCP docs](https://create.roblox.com/docs/en-us/studio/mcp.md),
  [MoguraVR](https://www.moguravr.com/roblox-studio-mcp-server-agentic-update/)
- **Unity**: Muse was retired on 2025-10-01 and replaced by **Unity AI** in Unity 6.2+. *Assistant* (a
  project-context helper that "generates pre-compiled code" and executes agentic actions such as batch
  renames and placing objects), *Generators* (sprites, textures, animation, sound, mostly third-party
  models), and *Inference Engine* (ex-Sentis, runtime local inference). Unity warned of **user liability
  on copyright** for generated content **[vendor + press]** —
  [Unity AI page](https://activation.unity3d.com/ai),
  [Digital Production 2025-08-22](https://digitalproduction.com/2025/08/22/unity-6-2-welcomes-ai-but-pace-caution-user-liability-on-copyright/),
  [Unity Muse docs](https://docs.unity3d.com/Packages/com.unity.muse.chat@1.1/manual/features-chat.html)
- **Godot**: stock Godot has no built-in generative AI and none on the public roadmap as of mid-2026
  (per a vendor of an AI-native Godot fork, so not neutral). Third-party plugins exist: "Godot AI
  Assistant" (Groq / Llama 3.3 70B), "AI Assistant Hub" (open source, BYO key or local model), and
  Ziva. Most plugins can edit but cannot run, observe and self-correct from the live runtime
  **[vendor-biased secondary]** —
  [Summer Engine blog](https://www.summerengine.com/blog/does-godot-have-ai.md),
  [Godot Asset Store](https://store.godotengine.org/asset/champ-gaming/godot-ai-assistant/)

### Inferences
- What differentiates engine assistants from generic code chat is **grounding in the live scene and
  runtime**: the data model, play mode, logs, viewport. PH2D already has scene, runtime and smoke
  infrastructure, so it can offer this grounding natively (e.g. "explain this error" with the actual
  entity and component values at the moment it fired).
- Roblox's "plan never auto-executes" and "preview before insert" are the error-limiting patterns
  worth copying. They give AI assistance without silent mutations.
- Unity's copyright-liability caveat is a reminder that *generated assets* carry legal risk that
  *explanations* do not.

### Gaps
- **No independent evaluation** (speed, error rate, retention) of Roblox Assistant, Unity AI or Godot
  plugins was found; all evidence is vendor docs or marketing.
- Adobe's creative-tool AI (Firefly / scripting) was not covered by the searches.
- Usage statistics for Roblox Assistant (adoption, acceptance) were not found in primary sources.

## 6. For a beginner-friendly game-scripting editor: which AI features are valuable and which are risky

### Takeaway
The evidence favours AI as a **tutor and explainer, grounded in deterministic tools**, over AI as an
**author**. Valuable, low-risk: concept and docs Q&A grounded in the API reference; explanations that
*augment* (never replace) precise compiler messages; snippet generation that is type-checked before it
is shown and inserted as one undoable, marked change. Risky: free-form generation that beginners accept
without understanding (measured skill loss); over-eager inline suggestions (distraction); agents that
mutate the project without an approval step; AI-invented API names. Notably, **LLM-rewritten error
messages did not speed novices up** in a controlled study; expert-written messages did.

### Cited Findings
- **Santos & Becker, "Not the Silver Bullet"** (UKICER 2024, arXiv 2409.18661): 106 novices, six buggy
  C programs, three randomised message types (GCC, expert handwritten, GPT-4). GPT-4 explanations beat
  compiler messages on time-to-fix in **only 1 of 6 tasks** and slowed students down in one. Students
  *preferred* the GPT-4 messages but did not debug faster. **Expert handwritten messages beat both** on
  objective and subjective measures **[independent]** — [arXiv](https://arxiv.org/pdf/2409.18661)
- **dcc --help** (SIGCSE 2024): LLM error explanations in CS1/CS2, 2,565 students, >64,000 uses in 10
  weeks. Explanations were conceptually accurate in **90 % of compile-time** and **75 % of run-time**
  cases, but **often ignored the instruction not to give the solution code** **[independent]** —
  [arXiv 2308.11873](https://arxiv.org/abs/2308.11873v2),
  [SIGCSE 2024](https://sigcse2024.sigcse.org/details/sigcse-ts-2024-Papers-1/69/dcc-help-Transforming-the-Role-of-the-Compiler-by-Generating-Context-Aware-Error-E)
- GPT-4 hints in an intro course (arXiv 2403.12737) helped on most syntax errors but **not on a
  type-comprehension error**; only ~56 % of students rated them very or extremely useful
  **[independent]** — [arXiv](https://arxiv.org/pdf/2403.12737)
- Anthropic RCT (2026-01): AI use while learning a new library gave **−17 points on comprehension**,
  worst on debugging. *Conceptual-question* use preserved learning (≥65 %); *delegating code
  generation* did not (<40 %) **[vendor-run research]** —
  [Anthropic](https://www.anthropic.com/research/AI-assistance-coding-skills)
- METR 2025 notes AI "may help less experienced developers or those working in unfamiliar codebases"
  (a hypothesis, not a finding of that study) **[independent]** —
  [METR](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/)
- Liang et al. 2024: developers value reduced keystrokes and *recalling syntax*; they struggle to
  *control* the tool; the authors recommend minimal-cognitive-effort interactions **[independent]** —
  [ICSE 2024](https://conf.researchr.org/details/icse-2024/icse-2024-research-track/3/A-Large-Scale-Survey-on-the-Usability-of-AI-Programming-Assistants-Successes-and-Cha)
- "Almost right, but not quite" is the #1 frustration (66 %), and 45 % say AI code is slower to debug
  **[survey]** — [InfoWorld](https://infoworld.com/article/4031673/ai-use-among-software-developers-grows-but-trust-remains-an-issue-stack-overflow-survey.html)
- Over-eager suggestions are a measured problem (Copilot NES "too eager", then fewer suggestions with
  higher acceptance) **[vendor]** —
  [GitHub blog](https://github.blog/ai-and-ml/github-copilot/evolving-github-copilots-next-edit-suggestions-through-custom-model-training/)
- Hallucinated names are frequent and repeatable **[independent]** —
  [CSA note](https://labs.cloudsecurityalliance.org/research/csa-research-note-slopsquatting-ai-supply-chain-20260419/)

### Inferences (design principles for PH2D, derived from the evidence above)
1. **Deterministic first, AI second.** Invest first in excellent hand-authored error messages and
   LSP-grade completion, hover and docs. That is where the controlled evidence is strongest (expert
   messages beat GPT-4). AI explanation is an *optional expansion* below the precise message, never a
   replacement.
2. **Nothing reaches the document unverified.** Every AI-produced snippet is parsed and type-checked
   against the live PH2D API (ideally by constrained decoding, since the grammar and API are closed)
   before it is shown. Unknown identifiers are impossible by construction.
3. **Nothing reaches the document unapproved.** Show a diff or preview; apply as **one undo step**;
   multi-step changes go through an editable plan that never auto-executes (Roblox pattern).
4. **Silent unless sure.** If inline next-edit exists at all, it is high-threshold, never competes with
   LSP completion for `Tab` (Zed's modifier-key arbitration), and can be switched off globally. Default
   OFF for beginners is defensible given the learning evidence.
5. **Tutor mode over author mode for learners.** Prefer "explain this", "what does this API do", "why
   did this error happen *here*" (grounded in runtime values) over "write it for me". The Anthropic RCT
   shows that conceptual use preserves skill and delegation erodes it. "Don't give the solution" prompts
   are unreliable (dcc --help), so enforce that in the product, not in the prompt.
6. **Mark provenance.** Tag AI-inserted ranges (gutter mark, cleared when the user edits or adopts the
   code), so review and debugging can tell AI origin apart. Watch Agent Trace as a future
   interchange format.
7. **Ground in the scene and runtime.** The unique value of an in-engine assistant is access to live
   state (entities, component values, logs, play mode), as in Roblox. Expose this via an **MCP server**
   so external agents can use it, rather than building a bespoke agent.
8. **Measure task time and defects, not satisfaction.** Perceived speedup is unreliable (METR:
   +20 % felt vs −19 % measured). Any AI feature in PH2D should ship with an A/B on time-to-working-
   script and error counts.
- **Most valuable for beginners (ranked by evidence plus risk):** (a) docs/API Q&A grounded in PH2D's
  own reference; (b) "explain this error in context" as an *add-on* to a precise deterministic message;
  (c) intent → snippet, type-checked and previewed, inserted as one marked undo step. **Riskiest:** (d)
  free agent mode editing the project; (e) always-on ghost text for learners; (f) AI-generated
  *assets* (copyright liability, per Unity) and AI-generated *tests* presented as proof of correctness
  (no evidence found either way).

### Gaps
- No study was found specifically on beginners in **game-scripting** editors (Roblox Luau, GDScript,
  Unity C#) measuring AI's effect on learning or error rates.
- No evidence was found on whether AI-generated tests reduce or merely relocate errors for beginners.
- No controlled comparison of "explain error" with vs. without access to *runtime state* (the engine
  advantage) was found; principle 7 is an inference from vendor direction, not measured.
