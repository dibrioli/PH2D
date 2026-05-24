---
name: ph2d-engine
description: Onboarding completo para a PH2D — Power House Game Engine, uma engine 2D de alta performance escrita em Rust com shells nativas finas para PC/Mac/iPad/iOS/Android. Use esta skill SEMPRE que o usuário mencionar PH2D, Power House, "a engine", trabalhar no editor, escrever código de subsistemas (rendering, física, fluidos, shaders, vetorial, SDFs, iluminação, networking, editor UI, scripting, áudio, MCP, acessibilidade, i18n), discutir arquitetura, tomar decisões de stack, ou pedir ajuda com qualquer parte do projeto da game engine 2D do usuário. Ative também quando o usuário falar em comparar/superar Godot ou Unity em 2D, ou quando aparecerem nomes de crates como wgpu, vello, kurbo, parley, harfrust, skrifa, rapier, bevy_ecs, winit, naga, taffy, gilrs, wasmtime, quinn, mlua (Luau) ou referências a WGSL/Slang/Metal/WebGPU/WebTransport/Apple Pencil/MCP no contexto do projeto dele. Esta é a fonte de verdade sobre vision, stack, convenções e invariantes do projeto — consulte antes de propor qualquer mudança arquitetural ou escolha de dependência. Para detalhes específicos de decisões individuais, consulte os ADRs em `docs/architecture/decisions/`.
---

# PH2D — Power House Game Engine — LLM Onboarding

> **PH2D** (Power House 2D). Engine 2D de altíssima performance, sem teto para artistas, com IA tratada como first-class user.

**Versão deste documento:** 2.14 — 2026-05-19 (**Wave 9 fechada — Multi-Agent UI Hardening**: arquitetura entrega "perfeição" pelos 2 critérios operacionais do Enio — (1) multi-agente paralelo sem colisão de fluxo crítico, (2) UI núcleo único de source-of-truth. **Eixo A**: `hero.rs::apply_event` (412 LOC inline) decomposto em [`screens/hero/chrome/`](crates/ph2d-editor-core/src/screens/hero/chrome/) com 11 handlers (theme/radius/view_toggles/rail_tools/rail_panels/io_menu/settings_ppm/settings_unit/scene_picker/image_tools_toggle/image_actions) + `dispatch_all`; hero.rs 1334→976 LOC; adicionar TopBar action = drop arquivo + 2 linhas em `chrome/mod.rs`, zero edit em hero.rs. `architecture_register_all_alphabetical` gateia ordem em Cargo.toml + `register_all()` de `ph2d-tool-registry-init`. **Eixo B**: [DIRETRIZ §4.2](docs/IntegracaoMultiAgente/DIRETRIZ.md) reescrita com cookbook completo de widget (template Rust + 5 mandamentos gateados); `architecture_widget_showcase_coverage` força todo widget aparecer no showcase ou opt-out justificado; `architecture_widget_loc_cap` cap 500 LOC por widget. **Eixo C**: `mockup_tokens_exist` em ph2d-tokens — todo `var(--*)` em mockups resolve via `docs/design/styles/*.css` ou `tokens.json`; descobriu gap real `--border-soft`, adicionado em `tokens.css` como `color-mix` cascade-derived. **Bug fix bônus** (pré-existente): submenu Display Unit dead code — faltavam `CTX_MENU_SETTINGS_UNIT` + `CTX_MENU_UNIT_METERS/PIXELS` em `populate_global_context_menu` → Click event nunca emitido; registrados. Smart cascade: `cascade_anchor()` em `chrome/mod.rs` usa `HeroScreen::last_viewport` para flipar submenu pra esquerda quando direita não cabe. Clamp viewport no painter (`context_menu_overlay::clamp_to_viewport`) como safety net. Workspace cargo test verde; smoke do Enio OK. Baseline anterior: 2.13 — 2026-05-19 — ADR-0029 Wave 8 fechado)

**Operação multi-agente:** Modelo atual (2 papéis + fluxo invertido) em [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](../docs/IntegracaoMultiAgente/DIRETRIZ.md). Narrativa histórica completa do problema multi-agente e as 4 waves de solução em [`docs/archive/multi-agente-pre-v6.0/Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md`](../docs/archive/multi-agente-pre-v6.0/Migracao/PARALLEL_AGENTS_PROBLEM_AND_SOLUTION.md).
**Idioma canônico do projeto:** português brasileiro (código em inglês, comentários em inglês curto, conversa de design em pt-BR).

## 1. Visão em uma frase

Uma engine 2D em Rust que: (a) renderiza vetorial, SDFs, fluidos e iluminação global em tempo real via GPU compute em qualquer plataforma moderna; (b) tem editor unificado em PC/Mac/iPad com UX nativa real (Apple Pencil incluso); (c) exporta apps nativos para iOS/Android/PC/Web com performance idêntica ao código manual; (d) é projetada desde o dia 1 para ser programada por LLMs com qualidade equivalente a humanos.

**Posicionamento:** superar Godot e Unity em 2D em três eixos onde ambos são fracos — qualidade vetorial/SDF, ferramentas de artista, e produtividade com agentes de IA.

## 2. Glossário e termos-chave

Lido cedo porque o resto do documento usa.

- **ABI** — Application Binary Interface. Layout binário estável das structs/funções que cruzam FFI.
- **ADR** — Architecture Decision Record. Documento curto justificando uma decisão; em `docs/architecture/decisions/NNNN-titulo.md`.
- **Core** — código Rust compartilhado, platform-agnostic. Vive em `crates/ph2d-*`.
- **ECS** — Entity Component System. Padrão arquitetural; aqui via `bevy_ecs` standalone.
- **Edition 2024** — edição da linguagem Rust (estabilizada em 1.85). Habilita `unsafe extern`, async closures, RPIT lifetime capture novo.
- **FFI** — Foreign Function Interface. Boundary `extern "C"` entre core Rust e shells nativas.
- **FLIP/PIC** — método híbrido grid-particle para fluidos.
- **Frame budget** — tempo total disponível para um frame: 16.6 ms a 60 Hz, 8.3 ms a 120 Hz (iPad ProMotion).
- **GGPO** — biblioteca/algoritmo seminal de rollback netcode. Usamos a ideia, não o código.
- **Handle opaco** — `u64` ou newtype que script/MCP recebe; mapeia internamente para `Entity` ou índice em pool.
- **Hot path** — código que roda dentro do frame: `render_graph`, `physics_step`, `audio_callback`, `editor_layout`. Não pode alocar.
- **HR-N** — Hard Rule número N. Ver §9.
- **IME** — Input Method Editor. Composição de texto para CJK, indispensável em produtos sérios.
- **Lockstep** — modo de netcode determinístico onde todos clientes simulam o mesmo input, peer-to-peer.
- **MCP** — Model Context Protocol (Anthropic). Servidor embutido que expõe operações da engine a LLMs.
- **MSRV** — Minimum Supported Rust Version. Aqui: **1.92** (toolchain pinada em 1.95; bumped em M11 para vello 0.8 + wgpu 28).
- **MSL/SPIR-V** — Metal Shading Language e SPIR-V (alvos de compilação a partir de WGSL via naga).
- **Platform-agnostic** — código que não conhece o SO. Toda interação com SO passa por trait `PlatformHost`.
- **PlatformHost** — trait expondo serviços do SO (FS, IME, file picker, gamepad, áudio device, etc.) para o core.
- **ProMotion** — display Apple a 120 Hz com refresh adaptativo (iPad Pro 2017+, iPhone 13 Pro+).
- **RC** — Radiance Cascades, técnica de GI 2D (Sannikov 2023).
- **Rollback** — modo de netcode que permite re-simular frames passados ao receber input atrasado.
- **SDF** — Signed Distance Field.
- **Shell** — camada nativa fina por plataforma (Swift, Kotlin, Rust+winit). ~5–10% da base de código.
- **WGSL** — WebGPU Shading Language (linguagem primária de shading aqui).
- **XPBD** — Extended Position Based Dynamics (Müller 2020). Soft body, cloth, rope.

## 3. Não-objetivos (importante)

Decisões deliberadas que economizam complexidade e foco. **Cada "não" aqui é uma decisão consciente; reverter exige ADR.**

- **Não** é engine 3D. 2.5D (sprites stack, parallax depth, normal maps) sim; cenas 3D completas não.
- **Não** suporta plataformas legadas. Sem OpenGL, sem D3D11. Ver §4 para a matriz exata.
- **Não** roda em hardware antigo. Ver §4.
- **Não** é "general purpose game framework." É opinionada: se você quer ECS diferente, scripting diferente, network diferente — fork.
- **Não** tem GUI immediate mode no produto distribuído. `egui` só é aceitável em ferramentas internas, profilers in-app e debug overlays — nunca compilado em build de release público.
- **Não** suporta scripting em N linguagens. **Luau strict** é a única linguagem de gameplay first-class (ratificado em ADR-0019). WASM aceita qualquer linguagem que produza WASM, mas a API canônica é Luau.
- **Não** suporta backwards-compat infinito. SemVer estrito; quebras agrupadas em majors anuais (ver §12.3).
- **Não** persegue paridade de funcionalidades com Unity/Godot. Persegue **superioridade nos eixos onde elas são fracas**.

## 4. Plataformas mínimas e GPU alvo

Tabela definitiva. Hardware abaixo disso não é alvo — feature won't fix.

| Plataforma | OS mínimo | API GPU mínima | GPU mínima | Memória app | Display alvo |
|---|---|---|---|---|---|
| iOS / iPadOS | iOS/iPadOS 17 | Metal 3 | Apple A14 (iPhone 12, iPad Air 4ª gen) | 1.5 GB | 60–120 Hz ProMotion |
| macOS | macOS 14 Sonoma | Metal 3 | Apple Silicon (M1+) ou AMD GCN5+ | 4 GB | até 120 Hz |
| Android | Android 13 (API 33) | Vulkan 1.3 | Adreno 660 / Mali-G78 / Xclipse 920+ | 2 GB | 60–120 Hz |
| Windows | Windows 11 | D3D12 (FL 12_1) ou Vulkan 1.3 | NVIDIA Turing+ / AMD RDNA1+ / Intel Arc Alchemist+ | 4 GB | 60–240 Hz |
| Linux | kernel 6.1 + Mesa 24+ | Vulkan 1.3 | igual a Windows | 4 GB | 60–240 Hz |
| Web | Chrome 121+, Safari 18+, Firefox 141+ | WebGPU | qualquer com WebGPU exposto | 1 GB heap | 60 Hz |

**Notas críticas:**
- iOS NÃO suporta Vulkan nativo. Em iOS o backend é Metal direto via wgpu — não MoltenVK.
- Android Vulkan 1.3 corta devices pre-2024. Decisão consciente — alvo de mercado é "smartphone moderno", não "menor denominador comum".
- Web é alvo first-class mas com restrições próprias (ver §11.12).
- Intel iGPUs em laptops "2018+" geralmente caem fora pelo `Memory Available` ou compute throughput. Documente em ADR-0007.

## 5. Stack canônico (versões pinadas)

Versões verificadas em **2026-05-09** (pós-M11). Toolchain: `rust-toolchain.toml` channel `1.95`, MSRV `1.92`, resolver `"3"`. Adicionar dep fora desta tabela exige justificativa em PR + ADR se for não-trivial.

| Camada | Tecnologia | Crate / Lib | Versão | Status / Notas |
|---|---|---|---|---|
| Linguagem core | Rust 2024 edition | — | MSRV **1.92** (toolchain pinada em 1.95) | `unsafe` requer justificativa em comentário; resolver = "3" |
| GPU abstração | wgpu | `wgpu` | `28` | **Downgrade de 29 → 28 em M11** para alinhar com vello 0.8. Único path; sem fallback OpenGL |
| GPU baixo nível (interop shell) | wgpu-hal | `wgpu-hal` | `28` | Apenas em FFI shell↔core, isolado em `ph2d-gpu::interop` (não wired ainda; M14+) |
| Shading runtime | WGSL via naga | `naga` | acompanha `wgpu 28` | Backends: SPIR-V, MSL, HLSL, GLSL |
| Shading autoria avançada | Slang (opcional) | `shader-slang` | `0.1.x` (experimental) | Não wired ainda |
| Vetorial GPU | Vello | `vello` | `0.8` (**alpha**, `default-features = false` + `wgpu`) | Rasterização 100% compute. Acessado via `vello::kurbo` / `vello::peniko` re-exports (kurbo 0.12, peniko 0.6) — declarar como dep direta arrisca version skew. Risco arquitetural — ver ADR-0004 |
| Curvas / Bézier | kurbo | `kurbo` | `0.12` (via `vello::kurbo`) | Hit-test, offset, fitting. Boolean ops via `linesweeper`, NÃO `kurbo::PathOps` (não existe) |
| Boolean ops vetorial | linesweeper | `linesweeper` | `beta` | Não wired ainda; M13+ |
| Text shaping | parley + harfrust + skrifa | `parley` | `0.6` (alpha) | Shaping, BiDi, fallback. Integra nativamente com Vello via `parley::Layout` + `vello::Scene` |
| Text editing widget | parley editor (ou custom) | `parley` | `0.6` | IME passa pelo `PlatformHost` (HR-1); não wired ainda |
| ECS | bevy_ecs (standalone) | `bevy_ecs` | `0.18` | Sem o resto do Bevy. Plano de upgrade documentado em ADR-0003-rev2 (Accepted) |
| Math | glam | `glam` | `0.30` | SIMD habilitado |
| Janela / input desktop | winit | `winit` | `0.30` | **Apenas** em shell desktop; nunca no core. iOS/Android usam shells nativas |
| UI layout | taffy / custom zones | `taffy` (planejada) / [`ph2d-editor::zones`](crates/ph2d-editor/src/zones.rs) (M12) | `0.10` (planejada) / 4-zone próprio (atual) | Layout 4-zonas Procreate-inspired escrito direto em ph2d-editor (ADR-0023). taffy entra se complexidade demandar (M13+) |
| Acessibilidade | AccessKit | `accesskit` | `0.24` | M12 wired em [`ph2d-a11y`](crates/ph2d-a11y/). Adapters por OS (`accesskit_macos`/`accesskit_windows`/`accesskit_unix`) ficam em shells |
| Rígidos | Rapier 2D | `rapier2d` | `0.28` (`default-features = false` + `dim2`/`f32`/`enhanced-determinism`) | Determinístico em modo lockstep, fixed timestep. M10 |
| Soft body / cloth / rope | XPBD próprio em compute | `ph2d-physics-soft` (interno, **stub**) | — | Müller 2020. Modo determinístico via fallback CPU (ver §11.5). M13+ |
| Fluidos | FLIP/PIC híbrido em compute | `ph2d-fluids` (interno, **stub**) | — | Não-determinístico por padrão; opt-out em modos com rollback. M13+ |
| Iluminação | Radiance Cascades 2D | `ph2d-light` (interno, **stub**) | — | Sannikov 2023; Holographic RC (2025) em roadmap. M13+ |
| Scripting (gameplay) | Luau strict via mlua | `mlua` | `0.10` (feature `luau`) | Runtime por mundo; GC incremental p99 ~0.005ms (medido C10). Ratificado ADR-0019. M7 wired |
| Hot path script | WASM | `wasmtime` | `44` (planejada) | Winch (rápido instantiate) padrão; Cranelift opt-in para AAA. Não wired ainda; M13+ |
| Networking transporte | QUIC | `quinn` | `0.11` (planejada) | Desktop/mobile. Não wired (`ph2d-net` é stub) |
| Networking web | WebTransport-over-HTTP/3 | `web-transport-quinn` | `0.11` (planejada) | Crate auxiliar — quinn puro NÃO é WebTransport |
| Áudio mixer | rodio + cpal | `rodio`, `cpal` | atual (planejadas) | `ph2d-audio` é stub. M13+ |
| Gamepad | gilrs (desktop), nativo (mobile) | `gilrs` | `0.11` | M8 wired em shells/desktop |
| Serialização binária | postcard | `postcard` | `1` | Assets, snapshots, save files |
| Serialização texto | serde JSON | `serde`, `serde_json` | atual | Apenas dev (cenas, configs); não shipping |
| Asset hash | blake3 | `blake3` | `1` | Conteúdo-endereçado (HR-6). M6 wired |
| Logging | tracing | `tracing`, `tracing-subscriber` | `0.1`/atual | Spans estruturados |
| Profiling in-app | puffin | `puffin` | atual (planejada) | Editor overlay; sem release |
| Profiling externo | tracy | `tracy-client` | atual (planejada) | Apenas com feature `tracy` |
| Erros | thiserror (libs) / anyhow (apps) | `thiserror`, `anyhow` | `2`/`1` | Nunca panic em código de produção (HR-4 implica) |
| Alocação em pool | bumpalo | `bumpalo` | atual | Hot path; reset por frame |
| Channels | crossbeam-channel | `crossbeam-channel` | atual | Comunicação entre threads (game/render/audio/IO) |
| Imagens | image | `image` | `0.25` (`default-features = false` + `png`) | M6 fixtures (PNG procedural). Outros formatos M13+ |
| i18n | fluent-rs | `fluent`, `fluent-bundle` | atual (planejada) | `ph2d-i18n` é stub. M13+ |

**Regra:** dependências fora desta tabela exigem justificativa em PR. Adicionar deps é caro — propagam em build time, supply chain, footprint.

## 6. Arquitetura: 1 core + 3 shells + 1 web target

```
┌──────────────────────────────────────────────────────────────────────┐
│                  SHELLS (finas, ~5–10% código)                       │
├──────────────┬──────────────────────┬──────────────────┬─────────────┤
│  Desktop     │  iPad / iOS          │  Android         │  Web        │
│  winit+wgpu  │  SwiftUI + MTKView   │  Kotlin +        │  WebGPU +   │
│  (Rust)      │  (Swift, ~3–5k LOC)  │  SurfaceView     │  WebTrans.  │
│              │  + GameController    │  (Kotlin,        │  (TS shell  │
│              │  + UIPencil          │  ~3–5k LOC)      │  ~1k LOC)   │
└──────┬───────┴──────────┬───────────┴────────┬─────────┴──────┬──────┘
       │                  │                    │                │
       │   FFI fino: eventos in, frame request out
       │   IME, gamepad, haptics, file picker, a11y tree
       │                  │                    │                │
┌──────▼──────────────────▼────────────────────▼────────────────▼──────┐
│                    CORE (Rust, ~85–90% código)                       │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Editor UI (custom retained-mode em Vello + parley + taffy)     │ │
│  │ Exporta árvore de acessibilidade via PlatformHost              │ │
│  └────────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Scene / ECS (bevy_ecs) │ Asset DB (blake3) │ Undo/Redo │ i18n  │ │
│  └────────────────────────────────────────────────────────────────┘ │
│  ┌──────────┬────────┬──────────┬──────────┬──────────┬──────────┐ │
│  │ Renderer │ Vector │ Lighting │ Physics  │ Audio    │ Net      │ │
│  │ (wgpu)   │+SDF+txt│ (RC 2D)  │ +Fluids  │ DSP      │ QUIC/WT  │ │
│  └──────────┴────────┴──────────┴──────────┴──────────┴──────────┘ │
│  ┌────────────────────────────────────────────────────────────────┐ │
│  │ Scripting (Luau / WASM) + MCP server (com governance)          │ │
│  └────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────┘
```

**Invariante crítico:** o core não conhece a plataforma (HR-1). Zero `#[cfg(target_os = ...)]` em código de subsistema. Toda interação com SO passa pela trait `PlatformHost` exposta pela shell. Web é uma "plataforma" como outra qualquer; `#[cfg(target_arch = "wasm32")]` é tolerado **apenas** em `ph2d-host` e em backends de transporte de `ph2d-net`.

## 7. Layout do repositório

Estado real verificado em **2026-05-09**. Legenda: ✅ implementado e wired no shell desktop; 🟡 implementado parcialmente (M13 em curso); ⏳ stub aguardando projeto-piloto.

```
_PH2D_definitiva/
├── Cargo.toml                    # workspace (resolver "3", edition 2024, MSRV 1.92)
├── rust-toolchain.toml           # toolchain channel 1.95
├── clippy.toml                   # workspace lints (HashMap ban per ADR-0022)
├── deny.toml                     # cargo-deny licenses + bans + advisories
├── crates/
│   ├── ph2d-core/                # ✅ M2 — math (glam), FixedStep, MemoryBudget, panic hook
│   ├── ph2d-host/                # ✅ M1 — trait PlatformHost (HostHandler, KeyEvent, PointerEvent)
│   ├── ph2d-ecs/                 # ✅ M4 — bevy_ecs 0.18 + SimWorld/PresentWorld + extract! macro (ADR-0021)
│   ├── ph2d-gpu/                 # ✅ M3 — wgpu 28 wrapper (GpuContext, SurfaceContext, FrameTarget, TransientPool) — ADR-0020
│   ├── ph2d-render/              # ✅ M5 — sprite renderer + VelloPass overlay (1000-sprite demo)
│   ├── ph2d-vector/              # ✅ M11 — vello 0.8 wrapper (VectorScene); re-exporta kurbo + peniko
│   ├── ph2d-text/                # ✅ M11 — parley 0.6 wrapper (TextSystem)
│   ├── ph2d-sdf/                 # ⏳ stub — SDFs animados, raymarching (M13+)
│   ├── ph2d-light/               # ⏳ stub — Radiance Cascades (M13+)
│   ├── ph2d-physics/             # ✅ M10 — rapier2d 0.28 + enhanced-determinism + cross-OS hash test
│   ├── ph2d-physics-soft/        # ⏳ stub — XPBD compute + fallback CPU (M13+)
│   ├── ph2d-fluids/              # ⏳ stub — FLIP/PIC compute (M13+)
│   ├── ph2d-audio/               # ⏳ stub — mixer, DSP, voice management (M13+)
│   ├── ph2d-asset/               # ✅ M6 — AssetDb (blake3 content-addressed) + AssetWatcher + ReloadEvent
│   ├── ph2d-script/              # ✅ M7 — Luau (mlua 0.10) ScriptHost + Scheduler + reset+restore
│   ├── ph2d-net/                 # ⏳ stub — QUIC + WebTransport, rollback, lockstep (M13+)
│   ├── ph2d-input/               # ✅ M8 — pure-data Event/InputState/Pencil (gilrs adapter na shell)
│   ├── ph2d-tokens/              # ✅ M12 — design tokens semânticos (color/type/spacing) — ADR-0023
│   ├── ph2d-editor/              # ✅ M12 — Layout 4-zonas + FloatingPanel + ZenMode + ToastQueue + ToolRegistry + paint trait + BrushTool + MoveTool — ADR-0023
│   ├── ph2d-mcp/                 # ✅ M9 — MCP server skeleton (JSON-RPC 2.0 dispatcher, tool registry)
│   ├── ph2d-i18n/                # ⏳ stub — Fluent runtime (M13+)
│   ├── ph2d-a11y/                # ✅ M12 — AccessKit 0.24 (Tree, NodeBuilder, Live) — ADR-0023
│   ├── ph2d-save/                # ⏳ stub — snapshot, replay, migration (M13+)
│   └── ph2d-telemetry/           # ⏳ stub — crash reporting, opt-in metrics (M13+)
├── shells/
│   ├── desktop/                  # ✅ winit 0.30 + wgpu 28 demo bin (integra M1/M5/M6/M7/M8/M12)
│   ├── ipad/                     # ⏳ não criada — Xcode project + SwiftUI + UIPencil + GameController (M14+)
│   ├── android/                  # ⏳ não criada — Gradle + Kotlin (M14+)
│   └── web/                      # ⏳ não criada — TS bootstrap + wasm-pack + Service Worker (M14+)
├── tools/
│   ├── ph2d-bindgen/             # ✅ M9 — gera .d.luau + schema MCP de catálogo (HR-10 enforced em CI)
│   ├── shader-cooker/            # ⏳ não criada — WGSL → SPIR-V/MSL/HLSL via naga (M13+)
│   └── asset-cooker/             # ⏳ não criada — importação batch determinística (M13+)
├── runtime/
│   └── luau/                     # ⏳ tipos .d.luau gerados (popular em M13+ quando catálogo estabilizar)
├── docs/
│   ├── architecture/decisions/   # ADRs 0003..0042 (0030..0041 = node + tool isolation)
│   ├── plans/                    # planos vigentes (node-waves, wave-11-carry-overs)
│   ├── IntegracaoMultiAgente/    # DIRETRIZ.md + briefing-node-crate.md + examples-fan-out.md
│   ├── HANDOFF_node_system.md    # tracker vivo do fan-out de nodes
│   ├── design/                   # design system: PROMPT_CLAUDE_DESIGN.md + component-library.html (vide §11.9)
│   ├── scripting/                # exemplos Luau + MCP prompts (c6/c15/c16 do spike)
│   ├── spike/                    # plano + report do spike fechado (histórico)
│   └── archive/                  # plans-completed/, handoffs-completed/, migracao-waves-completed/, multi-agente-pre-v6.0/
├── tests/
│   └── spike/                    # fixtures do spike de scripting (parte do workspace ainda)
└── .github/workflows/            # spike.yml (CI principal) + miri.yml — clippy + fmt + nextest + deny + audit + machete + typos + bindgen-check + cross-OS hash + MSRV
```

**24 crates total** (de §6: 1 core + 23 subsistemas/ferramentas). 14 implementados (✅ ou parcial), 10 stubs (⏳) aguardando M13+.

## 8. Feature flags canônicas

Cargo features são explosivas em combinação. Esta é a lista canônica; combinação fora dessa matriz não é suportada.

| Feature | Default | Descrição | Mutuamente exclusiva com |
|---|---|---|---|
| `editor` | off em release público | Compila editor UI completo | — |
| `mcp-server` | off em release de jogo | Embute MCP server | — |
| `tracy` | off | Liga `tracy-client` | `puffin-only` |
| `puffin-only` | on em editor | Apenas puffin overlay | `tracy` |
| `slang` | off | Habilita autoria Slang | — |
| `web` | exclusivo | Compila para wasm32 | `desktop`, `mobile` |
| `desktop` | exclusivo | Habilita winit, file picker desktop | `web`, `mobile` |
| `mobile` | exclusivo | Habilita IME mobile, sandbox FS | `web`, `desktop` |
| `dev-overlays` | on em editor | Debug HUDs, frame-time meter | — |
| `headless-server` | off | Build sem render/audio (server auth) | `editor`, `dev-overlays` |
| `det-physics` | on quando `mode=lockstep` | XPBD em fallback CPU; FLIP/PIC desligado | — |

**CI matrix:** o pipeline testa cada combinação válida (cartesian product das exclusivas × subset das ortogonais). Combinação que não foi testada não é suportada.

## 9. Hard rules — invariantes inegociáveis

**Cada hard rule é citável por ID (`HR-N`).** Cada uma carrega `Rule | Rationale | Enforced by`.

### HR-1 — Core é platform-agnostic
**Rule:** zero `#[cfg(target_os)]` em `ph2d-*` exceto `ph2d-host` e `ph2d-net::transport`. Nada de `std::fs::File`, `std::env`, sockets diretos no core. Tudo passa pela trait `PlatformHost`.
**Rationale:** permite iPad sandbox, Android SAF, Web OPFS, server auth headless — sem fork.
**Enforced by:** teste em CI (`tests/architecture/no_os_in_core.rs`) que faz grep por padrões proibidos nos crates listados; falha o build.

### HR-2 — `unsafe` requer justificativa escrita
**Rule:** todo bloco `unsafe` precisa de comentário acima explicando POR QUÊ é necessário e QUAIS invariantes garantem soundness.
**Rationale:** `unsafe` em engine é inevitável (FFI, GPU hal interop), mas inspecionar dezenas de blocos sem contexto é como auditar criptografia no escuro.
**Enforced by:** clippy custom lint (`ph2d-clippy::undocumented-unsafe`); CI quebra. Clippy `-D missing-safety-doc` para `pub unsafe fn`.

### HR-3 — Sem alocação dinâmica no hot path
**Rule:** dentro de `render_graph`, `physics_step`, `audio_callback`, `editor_layout` — zero `Box::new`, `Vec::push` que realoque, `String::from`, `HashMap::insert` que rehash. Use `bumpalo` (reset por frame), pools pré-alocados, `SmallVec` com capacidade fixa, ou ring buffers.
**Rationale:** alocação no hot path traz jitter imprevisível e leak surface; em audio callback, é causa raiz de glitch audível.
**Enforced by:** bench em `tests/budget/no_alloc_hot_path.rs` usa `dhat-rs` para contar allocs durante 10 frames sintéticos; falha se contar > 0 em hot paths marcados.

### HR-4 — Frame budget é sagrado
**Rule:** 16.6 ms a 60 Hz, 8.3 ms a 120 Hz. Cada subsistema declara seu sub-budget no `Plugin::init`. Estourar budget sem flag explícita `#[allow(budget_overrun = "razão")]` é bug.

Sub-budgets default (60 Hz, hardware mediano da matriz §4):

| Subsistema | Budget (ms) | Notas |
|---|---|---|
| Input + ECS scheduler | 0.5 | bevy_ecs scheduler overhead |
| Physics rígidos (rapier) | 1.5 | Fixed step 60 Hz |
| Physics soft (XPBD) | 2.0 | Compute, escala com partículas |
| Fluidos (FLIP/PIC) | 2.0 | Opt-in; off por default |
| Lighting (Radiance Cascades) | 2.5 | 6 cascades, configurável |
| Render principal | 3.5 | Sprites + vector + SDF + post |
| Editor UI overlay | 1.0 | Apenas em build com `editor` |
| Scripts (Luau+WASM) | 1.5 | Inclui FFI overhead |
| GC step Luau | 1.0 | p99 medido em C10: 0.005ms (folgadíssimo); manter budget para regressão |
| Audio mixer | <0.1 | Roda em thread separada (HR mas listado para clareza) |
| Folga | 1.5 | Para spikes |
| **Total 60 Hz** | **16.1** | |

A 120 Hz: corte FLIP/PIC, reduz cascades de 6 para 4, ECS roda metade dos systems não-críticos a 60 Hz interpolando.

**Rationale:** sem orçamento por subsistema, todo mundo gasta "só 1 ms" e o frame estoura.
**Enforced by:** `frame-budget-bench` em CI (hardware fixo: GitHub Actions Linux + um Mac mini M2 hospedado), gera baseline em git, falha em regressão > 5%.

### HR-5 — Determinismo onde prometido
**Rule:** quando o projeto declara modo determinístico (`Rollback`, `Lockstep`, replay), valem todas as regras abaixo:
- `f32` operações em ordem fixa, sem reordenamento por SIMD count-dependente.
- `mul_add`/FMA proibido em código determinístico (varia entre archs).
- Sem `fast-math`, `-ffast-math`, ou flags equivalentes.
- RNG com seed explícita (`Pcg64Mcg` recomendado), nunca `thread_rng`.
- GPU compute **proibido** em pipeline determinístico — XPBD cai para fallback CPU; FLIP/PIC desligado; Radiance Cascades aceito apenas porque é puramente visual (não influi em estado simulado).
- Reduções em GPU não-determinísticas; se entrarem em estado simulado, é bug.
- Iteração de `std::collections::HashMap`/`HashSet` **proibida** em simulation crates (SipHash com seed random por instância — ordem de iteração diverge cross-platform). Use `bevy_ecs::EntityHashMap`, `BTreeMap`, ou `Vec` indexed. Enforced via `clippy.toml` workspace-wide; ver [ADR-0022](../docs/architecture/decisions/0022-no-hashmap-in-simulation.md).

**Rationale:** rollback netcode quebra silenciosamente quando dois clientes divergem por 1 ULP em float; debugar isso em produção é pesadelo.
**Enforced by:** `tests/determinism/replay_cross_platform.rs` roda fixture de 600 ticks em Linux/Mac/Windows e compara hash do estado final.

### HR-6 — Asset = hash blake3
**Rule:** todo asset é content-addressed. Identidade = blake3 do conteúdo cooked. Paths são apenas índices humanos. Renomear arquivo NÃO invalida referências.
**Rationale:** refactoring de árvore de assets sem quebrar cenas/saves; cache hit determinístico em CI; integridade verificável em runtime.
**Enforced by:** `ph2d-asset::AssetDb::insert` exige hash; APIs que aceitam path o resolvem para handle e gravam o handle, não o path.

### HR-7 — Editor é a engine
**Rule:** mesma codebase, mesmo binário com feature flag `editor`. Em release público de jogo, `editor=off`, `mcp-server=off`, `dev-overlays=off`.
**Clarificação importante:** a feature `editor` deve cortar 100% do código de editor do binário final. CI mede o tamanho do binário com e sem a feature; diferença é o "custo do editor".
**Rationale:** sem fork "engine vs runtime" o editor é exatamente WYSIWYG; novas APIs aparecem para gameplay e ferramenta simultaneamente.
**Enforced by:** `tests/architecture/editor_feature_isolation.rs` builda com `--no-default-features --features release-game` e verifica símbolos do editor ausentes; CI falha se grep encontra.

### HR-8 — Scripts e MCP só falam handles opacos
**Rule:** Luau, WASM e MCP nunca recebem ponteiros, nunca enxergam layout interno. APIs expõem `Entity`, `Handle<T>`, `AssetId` — todos `u64` ou newtypes equivalentes. Tentativa de exfiltrar pointer é UB e CVE.
**Rationale:** sandbox é parte do modelo de segurança; um bug em script não pode corromper memória da engine.
**Enforced by:** revisão obrigatória de PRs que tocam `ph2d-script::bindings::*` ou `ph2d-mcp::tools::*`. Lista de tipos permitidos em FFI script é mantida em `ph2d-script/SAFE_TYPES.md`.

### HR-9 — GC em janelas explícitas
**Rule:** Luau roda em runtime dedicado por mundo. `lua.gc_step_kbytes(1)` é chamado entre frames pelo scheduler, em janela dedicada (sub-budget ~1 ms).

**Status (medido em C10 do spike, ADR-0019):** GC incremental do Luau é **muito** mais eficiente que QuickJS. Em fixture com 10k tabelas + 1k coroutines + per-frame allocation, p99 de step pause = **0.005 ms** (~277× abaixo do budget). Full `gc_collect()` upper bound: 0.015 ms. **Sem necessidade de mover lógica para WASM por causa de GC** — apenas por iteration overhead (vide HR sobre Luau ~60× Rust em C2).

**Rationale:** GC pause não pode estragar frame; Luau (mark-incremental, ref-count híbrido) cumpre folgadamente.
**Enforced by:** `tests/budget/luau_gc.rs` (port do c10_gc_stress) mede pause máximo em fixture; regressão > 0.5 ms warning, > 1.5 ms falha.

### HR-10 — MCP é first-class
**Rule:** toda API exposta a Luau é exposta a MCP. Se LLM não consegue fazer X, humano com Luau também não consegue. `ph2d-bindgen` gera schema MCP a partir das mesmas anotações `#[lua_export]`.
**Rationale:** o LLM é primeiro classe usuário, não bolt-on; a paridade força APIs limpas.
**Enforced by:** CI roda `cargo run -p ph2d-bindgen -- check` que verifica que cada `#[lua_export]` tem schema MCP correspondente.

### HR-11 — Mutações destrutivas via MCP exigem confirmação
**Rule:** ferramentas MCP marcadas `destructive: true` no schema (`scene_delete`, `asset_delete`, `project_clear`, `migration_run`) só executam com:
- token de confirmação humana (gerado por UI, válido por 5 min, single-use), OU
- flag `--unsafe-mcp` ligada explicitamente no servidor (modo CI/dev).

Toda mutação destrutiva grava em `audit.log` (JSON Lines, append-only): timestamp, agente, ferramenta, parâmetros, hash do estado antes/depois.
**Rationale:** "MCP first-class" sem governance é vetor de ataque adversarial; LLM pode ser enganado por prompt injection vindo de asset, conteúdo de scene, ou texto em chat.
**Enforced by:** `ph2d-mcp::governance::Guard` é envolto em todo handler destrutivo via macro `#[mcp_destructive]`; teste em `tests/security/mcp_governance.rs` tenta executar sem token e espera falha.

### HR-12 — UI custom popula árvore de acessibilidade
**Rule:** todo widget do editor (e qualquer UI que o jogo distribuído queira marcar acessível) gera nó na `AccessibilityTree` mantida em `ph2d-a11y`. Shells consomem essa árvore e a publicam:
- iOS: `UIAccessibility` / `AXUIElement`
- Android: `AccessibilityNodeInfo`
- Windows: UIA
- macOS: `NSAccessibility`
- Web: ARIA via DOM proxy

**Rationale:** UI nativa vem acessível de graça; UI custom em Vello tem custo escondido enorme aqui — ignorar bloqueia AppStore review e quebra a UE Accessibility Act 2025.
**Enforced by:** lint customizada exige que todo widget público implemente trait `Accessible`; CI verifica.

### HR-13 — Subsistemas declaram memory budget
**Rule:** cada `Plugin::init` retorna `MemoryBudget { vram_mb, ram_mb, heap_script_mb }`. Na inicialização, `ph2d-core` soma e checa contra limite da plataforma (§4); se estourar, recusa o boot com erro claro.
**Rationale:** OOM em iOS é jetsam silencioso; descobrir budget total no produto distribuído é cedo demais.
**Enforced by:** unit test em `ph2d-core::budget::test_total_under_platform_min` simula a matriz §4.

### HR-14 — Save format é versionado e migrável
**Rule:** todo struct que vai a save game tem campo `version: u32` no início. Migração de versão N → N+1 é função pura `fn migrate_v{N}_to_v{N+1}(old: VN) -> Result<V{N+1}>`. Sem migração, build de release não compila para um diff que muda schema.
**Rationale:** save corruption é uma das piores classes de bug em jogo distribuído; jogador perde progresso, review tanka.
**Enforced by:** macro `#[derive(Saveable)]` exige campo version; teste `tests/save/migration_chain.rs` carrega saves de N=1 até atual.

### HR-15 — Strings de UI passam por i18n
**Rule:** zero string hardcoded em UI de produção (editor distribuído ou jogo). Tudo via `t!("identifier")` que resolve em Fluent bundle. Strings de erro técnico (logs, panics, dev tools) são exceção — ficam em inglês.
**Rationale:** sem disciplina desde o dia 1, a localização vira projeto de meses; com, é commodity.
**Enforced by:** lint custom procura literais string em chamadas de widget; whitelisted em logs e panic messages.

### HR-16 — Storage lateral é serializável e determinístico, ou não existe
**Rule:** estruturas de estado fora do ECS (FSM, BT, Dialogue tables via `ph2d.fsm.state_table(entity)` e similares) só aceitam tipos POD-like: `number`, `boolean`, `string`, `Entity`/`Component` handles, e nested tables com mesma restrição (max depth 16). Proibidos: `function`, `userdata`, `thread` (coroutine), metatables custom. Iteração para serialização usa ordem alfabética de chaves (não a ordem de inserção/hash).
**Rationale:** save/restore (HR-14), snapshot rollback (HR-5) e hot reload (reset+restore) precisam serializar AMBOS — ECS world e storage lateral. Lua iteration é não-determinística por padrão; closures e userdata não são serializáveis. Sem disciplina desde o dia 1, save corruption é silenciosa e descoberta em produção.
**Enforced by:** API restritiva em `ph2d-script::lateral_storage` rejeita tipos proibidos em mlua bridge; teste `tests/determinism/lateral_storage_replay.rs` com fixture state_table-heavy. Lint custom proíbe uso de `pairs()` em vez de `pairs_sorted()` em pipeline determinístico.

### HR-17 — Examples canônicos compilam em CI
**Rule:** todo example em `docs/scripting/examples/` (~30 entradas no v0.1) compila com `luau-analyze` em strict mode e roda em fixture sintético. PR que altera API canônica e quebra example exige update do example no mesmo PR.
**Rationale:** LLM é o único programador. Documentação desatualizada é fricção catastrófica — LLM lê doc, gera código baseado nele, código falha. Examples curados em training data on-the-fly só funcionam se garantidamente corretos.
**Enforced by:** `tests/scripting/examples_compile.rs` carrega cada arquivo `.luau` em `docs/scripting/examples/`, valida com `luau-analyze --strict`, executa em runtime fixture; CI quebra na primeira falha.

### HR-18 — Crescimento bounded em shell binaries
**Rule:** Arquivos em `shells/<plataforma>/src/` respeitam caps de tamanho:
- Qualquer arquivo `.rs`: **≤ 600 LOC** (excluindo `tests/` e arquivos declarados como tabelas em comentário `// ph2d-loc-cap: table`).
- Qualquer função: **≤ 200 LOC** (corpo entre `{` e `}` do top-level fn).
- `main.rs` de qualquer shell: **≤ 400 LOC** — contém apenas struct App, impl ApplicationHandler, fn main, e tests inline.

Crescimento de funcionalidade acontece por adição de módulo `mod X;` (arquivo novo abaixo do cap), nunca por inflação de função ou arquivo existente.

**Rationale:** god-files são hostis a multi-agente (superfície de conflito), a LLM (excesso de contexto por janela), e a auditoria (complexidade ciclomática inauditável). Bound estrito força decomposição contínua por responsabilidade. Pré-migração 2026-05-16, `shells/desktop/src/main.rs` tinha 3463 LOC com `render_frame()` (1825 LOC) e `window_event()` (706 LOC) violando todos os caps — o PR de decomposição (ADR-0027) extraiu `init.rs`, `input_dispatch.rs`, `hero_intents.rs` reduzindo `main.rs` a 2421 LOC (transitional; cap ativa quando dispatcher genérico full landar).

**Enforced by:** `shells/desktop/tests/file_loc_caps.rs` (Wave 2 PR 11.9, **ativo** desde 2026-05-16). File-level cap (600 LOC) ativo; function-level cap (200 LOC) pendente parser real. Exceções por `// ph2d-loc-cap: <razão>` no topo do arquivo (primeiras 20 linhas; uso raro, requer justificativa em PR). **Exceções ativas hoje (pós Wave 3.2, 2026-05-17 noite): ZERO.** Test secundário `loc_cap_exceptions_inventory` imprime `HR-18 loc-cap exceptions inventory: NONE (cap fully active)` em CI logs. Histórico: pre-Wave-3.1 carregava 2 exceções (`main.rs` 2421 LOC, `hero_intents.rs` 696 LOC); Wave 3.1 stage A retirou hero_intents marker via split em directory module; Wave 3.1 stage C lifted render_frame body criando 3º marker temporário (`render_loop.rs` 1603 LOC); Wave 3.2 retirou OS DOIS markers restantes: stage A splitou render_loop em 7 sub-files, stage B splitou main.rs em `app_state.rs` (struct defs) + `input_handlers.rs` (3 grandes impl App methods).

## 10. Convenções de código

### 10.1 Rust style
- `cargo fmt` obrigatório (`rustfmt.toml`: `style_edition = "2024"`, `max_width = 100`).
- `cargo clippy -- -D warnings` em CI.
- Módulos: `snake_case`. Tipos: `PascalCase`. Constantes: `SCREAMING_SNAKE`.
- Erros: cada crate tem `Error` enum próprio com `thiserror`. App layer compõe com `anyhow`.
- Documentação: `///` em todo `pub`. Exemplos compilados (`cargo test --doc`) onde fizer sentido.
- Async: **proibido no core** exceto em `ph2d-asset::loader` (IO) e `ph2d-net` (sockets). Sync por default. Sem tokio no workspace; runtime async é `pollster` para casos pontuais.

### 10.2 Naming patterns
- Componentes ECS: substantivo singular. `Position`, `Velocity`, `SpriteRenderer`.
- Sistemas: verbo + objeto. `update_physics`, `render_sprites`.
- Resources: substantivo + sufixo descritivo. `AssetDb`, `RenderContext`, `InputState`.
- Eventos: passado. `EntitySpawned`, `AssetLoaded`.
- Traits: capacidade. `Renderable`, `Serializable`, `PlatformHost`.

### 10.3 FFI boundary (core ↔ shell)
- Tudo passa por `extern "C"` em `ph2d_host_ffi`.
- Tipos atravessando: apenas `#[repr(C)]` POD ou handles opacos `u64`.
- Nunca `Vec`, `String`, `&str` cruzando FFI. Use `*const u8 + len`.
- Lifetime: shell empresta buffer ao core dentro do callback; core não retém após retorno.
- Erros: returncodes `i32` + função separada `ph2d_last_error()` para detalhes (thread-local).
- **Versionamento:** toda struct de FFI começa com `version: u32`. Adicionar campo cresce versão; remover campo é major-bump no SDK shell.

### 10.4 wgpu-hal interop (shell entrega texture)
Tema sensível: para iOS receber `id<MTLTexture>` da shell e fazer wgpu renderizar nele, usamos `wgpu::hal` direto — API insegura.

- Isolado em `ph2d-gpu::interop::{metal, vulkan, dx12}`.
- Cada arquivo é `unsafe`-pesado e tem ADR (`ADR-0008-shell-texture-interop.md`) descrevendo invariantes.
- Esse é o único lugar do core onde `target_os` é tolerado, mas dentro de `ph2d-gpu` (não `ph2d-render`).

### 10.5 Shaders (WGSL)
- Um arquivo por pipeline. Não monolitos.
- Includes via preprocessor próprio em `ph2d-gpu`. Convenção: `#include "common/lighting.wgsl"`.
- Constantes via `override` quando possível (specialization), senão push constants ≤ 128 bytes, senão uniform buffer.
- Compute: workgroup size sempre potência de 2, documentada no topo. Subgroup operations apenas com fallback para devices sem suporte.
- **`PipelineLayoutDescriptor` sempre explícito.** `layout: PipelineLayout::Auto` proibido — quebra reuso entre pipelines similares e falha silenciosamente ao editar WGSL. Convenção `@group(0)` frame, `@group(1)` material, `@group(2)` draw (per toji.dev guide).
- **Determinismo:** shaders que entram em pipeline determinístico não usam `dpdx`/`dpdy`, não usam `pow` com base negativa, não confiam em ordem de execução de invocations.

### 10.6 Luau API
- Nomes idiomáticos Luau (snake_case), não traduções de Rust. `entity.position` (acesso), `ph2d.spawn(...)` (call).
- Tipos `.d.luau` gerados automaticamente do core via `ph2d-bindgen` (saída em `runtime/luau/`). Não escrever à mão.
- Coroutines (`ph2d.wait(seconds)` via `coroutine.yield`) para qualquer coisa que cruze frame boundary. Não bloqueie.
- Mensageria estilo Defold (`ph2d.message_send` / `ph2d.message_handler`) para desacoplamento entity-local.
- Strings de UI passam por `t!()` (HR-15) — wrapper Luau gera chamada Fluent.

## 11. Subsistemas — pontos críticos

### 11.1 Rendering {#rendering}
Pipeline base: clear → shadow/light pass (compute) → opaque sprites (depth-sorted) → vector layer (Vello) → SDF layer → particles → post → UI overlay.

Render graph é declarativo. Adicione passes via `RenderGraphBuilder`, nunca chamando `wgpu::CommandEncoder` diretamente fora do graph. Recursos transientes são alias-ados automaticamente.

**Convenção de coordenadas — explícito:**
- **World space:** Y-**up**, origem em metro 0,0 livre. `f32` em metros.
- **Texture/screen space:** Y-**down**, origem top-left, pixels.
- **Vello space:** Y-**down**, origem top-left (convenção PostScript/SVG, herdada de Vello/kurbo).
- O flip Y-up→Y-down é aplicado **uma vez** na projection matrix do main pass e em qualquer matrix passada ao Vello. Documentado em `ph2d-render::projection`. Nunca misture os dois sistemas dentro da mesma função.

### 11.2 Vetorial e SDF
Vello renderiza qualquer path Bézier que `kurbo::BezPath` produz.

Para **edição vetorial**, use `ph2d-vector::Document` (Bézier paths + transforms + boolean ops). Boolean ops via `linesweeper` (estável o suficiente para uso, mas marcado alpha — features que dependem disso herdam o status).

**Hit-testing:** use método `nearest()` do trait `kurbo::ParamCurveNearest` em cada segmento (não existe `kurbo::nearest` como função livre). Nunca rasterize-then-pick.

**SDF animados:** gerados em compute pass dedicado para textura `r16float` (verifique `Features::FLOAT16_SUPPORTED`; em devices que não suportam, fallback para `r16unorm`). Consumidos por shader de raymarching ou usados como mask. Nunca SDF em CPU em runtime.

### 11.3 Text e tipografia
**Stack:** `parley` (layout), `harfrust` (shaping), `skrifa` (font parsing), Vello (rasterização). Linebender mantém tudo, integração nativa.

**O que oferecemos:**
- Bidi (RTL para árabe/hebraico) via Unicode Bidi Algorithm (UAX #9).
- Complex scripts (devanagari, thai, brahmic, CJK).
- Emoji color (COLR/CPAL e CBDT/CBLC) e variation sequences.
- Variable fonts.
- Font fallback chain configurável; default por plataforma vem do `PlatformHost::system_fonts()`.
- Subpixel positioning.

**Text editing widget:**
- Cursor, seleção, undo/redo per-widget.
- IME composing string passa pelo `PlatformHost` via `ph2d_event_ime_*` (ver §13).
- Widget de texto editável é o componente mais complexo do editor; é trabalho contínuo, não "pronto na v1".

**Licenciamento de fontes:** assets de fonte importados ganham campo `license` no manifest. MCP `asset_import` valida (ou recusa se ausente); release build falha se algum asset de fonte não tem licença declarada.

### 11.4 Iluminação (Radiance Cascades)
Implementação segue **Sannikov 2023** (paper original, ExileCon). Roadmap: avaliar **Holographic Radiance Cascades** (Osborne et al., 2025) para reduzir penumbra de luzes distantes — em ADR-0009.

Parâmetros default:
- 6 cascades.
- Cascade 0 interval: configurável; padrão 4 px na resolução interna de iluminação.
- Angular resolution dobra por cascade; spatial resolution diminui pela metade — mantém memória aproximadamente constante.
- Tudo em compute. Output: `rgba16float` radiance map composto no main pass.

Emissivos vêm de canal alfa de sprites + emission textures. Sombras derivam do occluder map (em `r8unorm`, 1 byte/pixel — não tente "1 bit" na mesma textura; se quiser compactar, empacote 8 pixels por byte explicitamente em `r8uint`).

### 11.5 Physics
**Dois mundos coexistem:** Rapier (rígidos, fixed step 60 Hz) e XPBD (soft/cloth/rope, mesma freq).

- **Modo padrão:** XPBD em compute (GPU), acoplado a Rapier via "pinned constraints" — XPBD lê pose de Rapier no início do step, devolve forças no final. Acoplamento é one-way em modo determinístico.
- **Modo determinístico (`det-physics` feature):** XPBD cai para fallback CPU pure-Rust (`ph2d-physics-soft::cpu_backend`); Rapier usa modo lockstep. Performance cai ~3× mas é reprodutível bit-a-bit cross-platform.

**Fluidos FLIP/PIC:**
- Grid 256² ou 512² (configurável), partículas por célula 4–8.
- Não interage com rígidos por padrão; opt-in via voxelization do collider.
- **Não-determinístico** (reduções em compute). Em modo determinístico: desligado.

### 11.6 Áudio
`rodio` para mixing de alto nível, `cpal` para device backend.

- Mixer roda em thread separada (callback do `cpal`).
- HR-3 vale dobrado aqui: alocar no callback = glitch garantido.
- DSP custom em `ph2d-audio::dsp`: filtros, reverb (FDN), spatial 2D (HRTF opcional).
- Voice management via pool fixo (default: 64 voices simultâneas). Política de stealing: oldest-quietest.
- Hot reload de samples: feita off-thread, swap atômico no fim do frame.

### 11.7 Scripting

**Status:** Ratificado por spike 2026-05 (vide [ADR-0019](docs/architecture/decisions/0019-spike-scripting-output.md)). Linguagem canônica: **Luau strict via mlua 0.10** (não TypeScript/QuickJS). ECS canônico: `bevy_ecs = "0.18"` ([ADR-0003-rev2](docs/architecture/decisions/0003-ecs-choice.md)).

**Luau runtime (canônico):**
- `mlua 0.10` com feature `luau`. Runtime por mundo (single-player) ou por sessão (multiplayer authoritative server).
- Sandbox em dois níveis: trusted (project scripts) vs untrusted (asset scripts).
- Bytecode pré-compilado no ship build (Compiler com `optimization_level=2`, `debug_level=0`). Cold start 1.5–2× mais rápido que source. **Nota:** size com gzip on-top é equiparável ou pior que source gzipped — bytecode otimiza time + anti-tamper, não size.
- GC incremental: `gc_step_kbytes(1)` por frame mantém p99 < 0.01ms (medido em fixture 10k tabelas). HR-9 cumprido folgadamente.
- Coroutines como primitiva temporal canônica; `ph2d.wait(seconds)` via `coroutine.yield`. Scheduler PH2D resume em tick (dt fixed) — p99 = 1 frame (16.67ms a 60Hz).

**Hot reload (Defold-style reset+restore):**
- Snapshot do World via `postcard` + hash `blake3`. Reset = drop World; restore = re-spawn de snapshot.
- 100% determinístico em fixture 200 entities × 3 components (medido C4: 100/100 hash matches, freeze p99 0.3ms).
- Estado canônico vai no World, não em closures Lua. Coroutines pendentes mid-flight são descartadas com warning ao recarregar.
- HR-16 storage lateral: `state_table(entity)` aceita apenas tipos POD-like; `pairs_sorted()` obrigatório em pipeline determinístico.

**Mensageria estilo Defold:**
- `ph2d.message_send(target, message, payload)` + `ph2d.message_handler(message, fn)`.
- Hash interning para nomes de mensagem; FIFO same-sender→same-target.
- Schema opcional em dev (Cargo feature `mcp-schema`); off em release.

**WASM (hot path):**
- Para systems CPU-bound (pathfinding, AI, simulação custom): payload primitivo Luau→Rust→wasmtime mede p99 = 0.21µs (folgadíssimo vs threshold 1µs em C12).
- Bindings via `wit-bindgen` + Component Model. NÃO `wasm-bindgen` (esse é específico de browser/JS host).
- Wasmtime 44 com Winch (instanciação ~µs, código menos otimizado, default) vs Cranelift (instanciação lenta, código rápido, opt-in via feature `wasm-aot`).
- Bridge canônica: **Luau chama Rust; Rust chama WASM** (single FFI boundary).

**Performance trade-off (medido C2):**
- Luau iteração ~60× mais lenta que Rust nativo (loop puro, 1k entries × 5 fields).
- **Implicação:** Luau é para gameplay scene logic (FSM, dialogue, scripted events, tweens) — não hot path iteration de muitas entities. Lógica iterativa heavy → Rust system ou WASM.

### 11.8 Networking
Três modos selecionáveis por projeto, **não combináveis dentro da mesma sessão**:

- `Rollback` — GGPO-style, input delay configurável, history buffer 60 frames. Requer determinismo (HR-5 + `det-physics`).
- `Lockstep` — determinístico puro, peer-to-peer, ideal para RTS. Mesmas exigências.
- `ServerAuth` — tick rate independente de framerate, client prediction + reconciliation. Não exige determinismo.

**Transporte:**
- Desktop/mobile: QUIC via `quinn`.
- Web: WebTransport-over-HTTP/3 via `web-transport-quinn` (não é o mesmo wire format que QUIC puro).
- `ph2d-net::Transport` é trait que esconde a diferença; o subsistema que chama não sabe se está rodando native ou web.

**Snapshot/restore:** essencial para Rollback. ECS expõe `World::snapshot(&Reflect)` → `Bytes` (postcard) e `World::restore(&Bytes)`. Componentes precisam derivar `Reflect` para entrar no snapshot.

### 11.9 Editor UI
Retained-mode próprio em Vello + parley. Não egui no produto final (HR-7).

**Estado em M13 (2026-05-10):** [`ph2d-editor`](crates/ph2d-editor/) implementa o esqueleto canvas-first 4-zonas (ADR-0023) + biblioteca completa de componentes UI:
- [`Layout`](crates/ph2d-editor/src/zones.rs) — 4 zonas (TopLeft EDIT / TopRight CREATE / Sidebar modulators / Center 100% canvas) + ZenMode toggle + sidebar mirror
- [`FloatingPanel`](crates/ph2d-editor/src/floating_panel.rs) — Procreate-style draggable tool drawer com `PanelControl` enum (Slider/Toggle/RadioGroup/ColorSwatch/Action)
- [`icons`](crates/ph2d-editor/src/icons.rs) — 89 IconId variants (Lucide-derived), 24×24 viewBox, parsed via `BezPath::from_svg` em `cmd_to_path`
- [`paint`](crates/ph2d-editor/src/paint.rs) — Vello lowering (`Paint` trait, `paint_text` via parley→vello, `fill_rounded_rect`/`stroke_rect`/`stroke_rounded_rect`/`paint_icon`)
- [`widget`](crates/ph2d-editor/src/widget/) — biblioteca completa: cada widget = data + state enum + tokens + `a11y::Node` + `paint_X` helper colocalizado:
  - **Atomic**: Button (Default/Accent/Danger/IconOnly + Loading), Checkbox (3-state), TextInput **(interativo: insert/Backspace/arrows + cursor caret pintado quando focused via `paint_text_input_with_buffer`)**, TextArea, NumberInput **(interativo, canonical M14.A: (1) digitação filtrada `[0-9.eE+-]` + Enter commit + Esc revert; (2) clique-segurar nas setas ▲/▼ → continuous-hold via `dispatch_tick` (250 ms initial delay, 30 ms repeat — macOS Aqua feel); (3) clique + arrastar no corpo → drag-slider Blender-style com axis-lock travado no primeiro Move que cruza 4 px (eixo H = 50 step/px, V = 5 step/px, Shift = ×0.001 fine); buffer atualiza em tempo real durante drag; (4) linkagem two-way com Slider via `WidgetStore::link_slider_number`. Estado interno: `NumberInputDragState` + `NumberStepperHoldState` em `interaction::state`)**, Slider (Horizontal/Vertical + ticks; drag mirror em NumberInput linkado), Toggle, RadioGroup (Horizontal/Vertical/Segmented; `paint_radio_group_with_labels` desenha labels — Segmented sem labels seria pill vazio), ColorSwatch (3 sizes + alpha checker), ProgressBar (Determinate/Indeterminate), Spinner, Avatar (Circle/Square), Divider, Tag (5 tones)
  - **Compound**: Tabs (Ghost/Segmented), Dropdown, Combobox (filtered), Vector3Editor (R/G/B-tinted X/Y/Z labels), ListItem, Card, Tooltip, ContextMenu (com separators)
  - **Surfaces**: Modal (over `BgScrim`), Popover, restyled ToastQueue (severity icon + accent stripe + neutral body)
  - **Complex**: TreeView (BTreeSet expand/select), ColorPicker (5 modes; Classic ships v1), BlenderColorPicker (wheel HSV real via sweep+radial gradient peniko, valor vertical com gradiente cor→preto, Linear/Perceptual + RGB/HSV segmented com labels, 4 sliders RGB/HSV, hex parse `#RGB[A]`/`#RRGGBB[AA]`, paletas editáveis; saída `ColorValue { rgba, oklch }`; wheel e value clicáveis via `paint_blender_color_picker_with_store` + `apply_blender_*_pick`)
  - **Composição** (M13 hero sprint): PillGroup, ToolRail (icon+compound entries), StatusBar (segmented HUD), SectionHeader (count chip + collapsible) — primitivos da tela hero `02-editor-main`.
  - **Showcase region** (deep polish 2026-05-10): [`screens/hero/showcase.rs`](crates/ph2d-editor/src/screens/hero/showcase.rs) — painel ancorado bottom-left do canvas que exercita 18+ widgets em uso funcional simultâneo (Card+ListItem+Divider, Vector3Editor, ProgressBar det+ind, Spinner, Avatar Circle+Square, Tag, RadioGroup Segmented, Dropdown, Combobox, Checkbox Indeterminate, TextInput, TextArea, ContextMenu, Popover, SectionHeader, Slider Vertical, Modal mini); BlenderColorPicker no bottom-right do canvas. Skip silencioso quando viewport pequeno demais.
- [`screens::hero`](crates/ph2d-editor/src/screens/hero.rs) — composição da tela `02-editor-main.html` em `paint_hero_screen`. Layout regions (TopBar/LeftRail/Inspector/Hierarchy/BottomHUD/canvas/overlay) renderizadas com fixture content em [`screens/hero/fixture.rs`](crates/ph2d-editor/src/screens/hero/fixture.rs). Habilitada via `PH2D_HERO_SCREEN=1 cargo run -p ph2d-host-desktop`. Interativa via [`interaction`](crates/ph2d-editor/src/interaction/) per ADR-0024.
- [`interaction`](crates/ph2d-editor/src/interaction/) — input pipeline + retained widget state (ADR-0024). `WidgetStore` (BTreeMap pré-populado), `HitIndex` (SmallVec inline 128), `dispatch_pointer/dispatch_key/dispatch_text_input` (eventos via arena bumpalo). HR-3 zero-alloc enforced em [`tests/interaction_no_alloc.rs`](crates/ph2d-editor/tests/interaction_no_alloc.rs) com dhat-rs.
- [`Tool` + `ToolRegistry`](crates/ph2d-editor/src/tool.rs) — contrato canônico (id/label/icon/build_panel/activate/handle_panel_event)
- [`tools::BrushTool`](crates/ph2d-editor/src/tools/brush.rs) + [`tools::MoveTool`](crates/ph2d-editor/src/tools/move_tool.rs) — implementações seed
- [`ZenMode`](crates/ph2d-editor/src/zen.rs) + [`ToastQueue`](crates/ph2d-editor/src/toast.rs)

**Out of scope até M13+:** QuickMenu radial (ADR-0023 §6), gesture-mapping editor UI (§4), Single-Touch Companion overlay, dock complexo, timeline, node graph editor, text editor widget — todos viram após design system canônico estabilizar.

**Layout solver:** zones próprias por enquanto (matemática trivial 4-zonas); `taffy` 0.10 entra se complexidade demandar (formulários longos, listas virtualizadas).

Input passa pelo trait do `ph2d-input` que abstrai mouse/touch/Pencil pure-data. Shell desktop usa `gilrs` adapter (M8). Pencil pressure/tilt são primeiros-classe — não emulados como mouse.

**Acessibilidade (M12 wired):** cada widget implementa `accesskit::Node` builder via [`ph2d-a11y`](crates/ph2d-a11y/) (HR-12). Editor sem acessibilidade não passa em CI. Adapters por OS (`accesskit_macos` / `accesskit_windows` / `accesskit_unix`) ficam nas shells.

**i18n:** UI strings via Fluent (HR-15). Bundle padrão em `crates/ph2d-editor/locales/` quando i18n entrar (M13+; ph2d-i18n é stub atualmente).

**Design system canônico (M13, entregue 2026-05-09):**
Pacote oficial em [`docs/design/`](docs/design/), gerado pelo Claude Design a partir do brief em [`PROMPT_CLAUDE_DESIGN.md`](docs/design/PROMPT_CLAUDE_DESIGN.md). Conteúdo:
- [`tokens.json`](docs/design/tokens.json) — 4 temas OKLCH (`forge-sdf` default, `paint-studio`, `sunstone`, `blueprint`) + typography + spacing + radius + shadow + motion + z-stack. **Source of truth** para codegen do crate `ph2d-tokens`.
- [`component-library.html`](docs/design/component-library.html) — 30+ widgets × 7 estados (Normal/Hover/Pressed/Focused/Disabled/Active/Selected) tematizados ao vivo via tweaks panel.
- [`screens/`](docs/design/screens/) — 17 telas iPad 12.9 (1366×1024): welcome, editor-main (hero), place-tool, select-tool, asset-browser, hierarchy, inspector, color-picker, component-editor, script-editor, console, quickmenu, zen-mode, play-mode, build-export, prefs, search-global.
- [`icons/`](docs/design/icons/) — 87 SVGs Lucide-derived (ISC license), 24×24 viewbox, 1.5pt stroke, currentColor — convertem direto para `vello::kurbo::BezPath`.
- 4 specs canônicos: [`interactions.md`](docs/design/interactions.md), [`gestures.md`](docs/design/gestures.md), [`animation.md`](docs/design/animation.md), [`accessibility.md`](docs/design/accessibility.md).
- [`audit.md`](docs/design/audit.md) — auto-auditoria do entregue (P1: temas não-default têm bg dark hardcoded em 13 das 17 telas — fix de 2-3h; P2: aspect ratios extras só em tela 02).
- [`styles/`](docs/design/styles/) — CSS vars derivados (consumo browser sem rodar codegen). `tweaks-panel.jsx` + `index.html` para navegação interativa.
- [`component-library-v2-legacy.html`](docs/design/component-library-v2-legacy.html) — mockup pré-canonical (sdf3d-studio inspiration), preservado para contexto histórico.

**Implementação do design em Vello (M13):**
1. ✅ Import do pacote em `docs/design/`.
2. ✅ Codegen `ph2d-tokens` a partir de tokens.json (4 themes, OKLCH→sRGB, semantic slots).
3. ✅ Port dos 89 SVGs para módulo `ph2d-editor::icons` (IconId enum + cmd_to_path).
3.5. ✅ Biblioteca completa de componentes (27 widgets, 259 testes) — vide §11.9 lista por categoria.
4. ✅ Tela 02-editor-main composta em `screens::hero` (4 primitivos novos + composer + fixture). Render via `PH2D_HERO_SCREEN=1` env var.
5. ✅ Input pipeline ADR-0024 (`interaction` module + WidgetStore + dispatch + HR-3 zero-alloc bench). Hero responde a hover/click/drag/keyboard; clicar Hierarchy row muda Inspector title.
6. ⏳ Resolver P1/P2 do audit (telas não-canônicas).
7. ⏳ Telas 03-17 (asset browser, hierarchy, inspector standalone, etc) — escopo aberto pós projeto-piloto.

### 11.10 Asset pipeline
Pipeline **deterministic + reproducible**: mesmo input + mesma versão de cooker = mesmo blake3 do output.

**Importadores suportados (v1):**

| Formato | Crate | Output cooked |
|---|---|---|
| PNG, JPEG, WebP, AVIF | `image` | Texture (BC7 desktop, ASTC mobile, ETC2 fallback) |
| EXR | `exr` | Texture HDR (`rgba16float` ou `rgba32float`) |
| SVG | `usvg` + conversão para `BezPath` | Vector document |
| TTF, OTF, WOFF2 | `skrifa` | Font asset com licença obrigatória |
| WAV, OGG, Opus | nativo Rust | Audio asset (passthrough Opus, recompress WAV→Opus se config) |
| Aseprite | `asefile` | Sprite atlas + animation clips |
| Tiled | `tiled` | Tilemap |
| Spine, DragonBones | `ph2d-importers::spine`/`dragonbones` (custom) | Skeletal animation |
| Lottie | `lottie-rs` | Vector animation |
| glTF (apenas para 2.5D normal maps + bones) | `gltf` | Subset 2.5D |

**Adicionar importador novo:** ver §14.

**Texture compression:**
- Desktop: BC7 (RGBA), BC6H (HDR), BC4 (mask).
- Mobile (iOS/Android): ASTC 6x6 default; 4x4 para UI/sprites críticos.
- Web: BC + ASTC (depende de browser feature query); fallback para `rgba8unorm`.

**Atlas packing:** offline, em `asset-cooker`. Heurística: max-rect com guillotine.

**Streaming:** texturas grandes (> 4 MB) marcadas `streamable: true`; carregadas em mip-chain progressiva.

**Hot reload:**
- Arquivo source muda → cooker re-importa → novo blake3 → registry atualiza (path → handle) — **handle muda**, mas referências por path em scenes resolvidas em load time pegam o novo handle. Snapshots em uso recebem swap atômico no fim do frame.
- Renomeação não invalida (HR-6) — paths são índice, hash é identidade.

### 11.11 MCP server + governance
Embutido. Expõe ferramentas:

**Read-only:**
- `scene_query`, `entity_get`, `component_list`
- `asset_browse`, `asset_inspect`
- `shader_compile_check`, `shader_inspect`
- `runtime_state`, `runtime_log_tail`
- `script_lint`

**Mutative (não-destrutivas):**
- `scene_create_entity`, `component_set`
- `asset_import` (gera novo asset; não deleta existentes)
- `script_run` (em sandbox, com timeout)

**Destructive (HR-11):**
- `scene_delete_entity`, `scene_clear`
- `asset_delete`
- `project_clear`
- `migration_run`

Cada destructive operação:
1. Recebe `confirmation_token` no payload.
2. Token validado contra `ph2d-mcp::tokens` (single-use, 5 min).
3. Se válido: snapshot do estado pré-operação salvo em `audit/`, operação executa, audit log registra.
4. Se inválido: retorna erro pedindo token humano OU presença de `--unsafe-mcp` no servidor.

**Multi-agent:** mutações detêm lock advisory por path/entity. Conflito: erro com hint para retry.

**Schema MCP** em `crates/ph2d-mcp/SCHEMA.md`, gerado por `ph2d-bindgen`.

### 11.12 Web target
First-class, com restrições:

- **Threading:** sem `SharedArrayBuffer` por default — requer headers HTTP `Cross-Origin-Opener-Policy: same-origin` e `Cross-Origin-Embedder-Policy: require-corp` no servidor que serve o app. Sem isso, `rayon` cai para single-thread fallback automático em runtime; `ph2d-physics` perde paralelismo, `ph2d-asset` loader serializa. Documentar explicitamente em README do projeto-piloto que deploy precisa configurar esses headers (Vercel/Cloudflare/nginx — todos suportam via headers config). **Não silenciar:** logar warning se em runtime web detectar ausência (`window.crossOriginIsolated === false`).
- **Asset loading:** `fetch` streaming + cache em OPFS (`Origin Private File System`). Hot reload via WebSocket dev server.
- **Audio:** Web Audio API atrás do `ph2d-audio`. Latência tipicamente pior que native (~20 ms vs ~5 ms).
- **Wasm size budget:** core compilado deve ficar abaixo de 8 MB gzipped na configuração default (`web` feature). Medido em CI.
- **Service Worker:** opcional, gerencia offline + cache de assets.
- **WebGPU adapter selection:** `high-performance` por default; user override via query string `?adapter=low-power` para devmobile testing.

## 12. Cross-cutting concerns

### 12.1 Memory budgets
Cada subsistema declara budget em `Plugin::init` (HR-13). Tabela default por plataforma (em MB):

| Subsistema | iPad / iPhone | Android med | Desktop | Web |
|---|---|---|---|---|
| Render textures+meshes | 350 | 400 | 1200 | 200 |
| Audio buffers | 30 | 30 | 80 | 20 |
| Physics state | 20 | 20 | 80 | 10 |
| Lighting (RC) | 80 | 80 | 200 | 50 |
| Asset DB cache | 200 | 250 | 1000 | 150 |
| ECS world | 50 | 50 | 200 | 30 |
| Script heap (Luau) | 64 | 64 | 128 | 32 |
| WASM linear memory | 64 | 64 | 256 | 64 |
| Editor UI | 80 (apenas iPad) | — | 200 | 80 |
| Working / temp | 60 | 60 | 200 | 50 |
| **Total app target** | **~1000** | **~1000** | **~3500** | **~700** |

Margem para OS deixada explícita no doc-string de `MemoryBudget::platform_max`.

### 12.2 Concurrency model
Threads canônicas:

- **Game thread (main):** ECS scheduler do bevy_ecs, lógica de jogo, scripts. Pode ser multi-threaded internamente via bevy_ecs (parallel systems).
- **Render thread:** comando GPU encoding + present. Recebe `PresentWorld` via channel `crossbeam` no fim do frame de game.
- **Audio thread:** `cpal` callback. Sample-accurate, sem alocação. Comunica com game via lock-free queue (`crossbeam` ArrayQueue).
- **IO thread pool:** `rayon`-based. Asset loading, hot reload, save IO. `rayon` é a única lib de paralelismo permitida; tokio é proibido no core.
- **Script thread (opcional):** WASM heavy pode rodar em thread dedicada com message passing; default é game thread.

**Two-world model (separação por TIPOS, não só threads):** PH2D usa `SimWorld` (estado simulado canônico) + `PresentWorld` (estado de presentation, render, animation, editor) em `bevy_ecs::World` distintos. Ponte one-way via `extract!` macro. Traits `SimComponent`/`PresentComponent` enforce em compile-time que sistema de presentation não muta estado simulado (HR-5, HR-7). Vide [ADR-0021](../docs/architecture/decisions/0021-simulation-presentation-boundary.md).

**Async morre na fronteira da shell:** o Web "respira" async (fetch, requestAnimationFrame), mas o core PH2D é síncrono por design (HR e §10.1). Único async tolerado é `ph2d-asset::loader` (carregamento off-thread) e `ph2d-net::transport` (sockets). Tudo o mais é fixed-step síncrono.

Luau (mlua) é single-threaded por design — runtime fica preso à game thread (mesmo modelo que QuickJS antigo).

`parking_lot::RwLock`: usado raramente, apenas em `AssetDb` e `Registry`. Hot path proíbe (HR-3 deriva).

### 12.3 Estabilidade e versionamento
**SemVer:**
- 0.x até a primeira release pública. `0.x.y` aceita quebras em `x`.
- 1.0 = compromisso de estabilidade. APIs públicas seguem SemVer estrito.
- Quebras agrupadas em majors; major bump por ano calendário no máximo.

**Deprecation:**
- API marcada `#[deprecated(since = "X.Y", note = "...")]`.
- Permanece por 2 minor releases (~6 meses) antes de remoção.
- Luau API: doc-comment `--- @deprecated` no `.d.luau`; warning em runtime via `ph2d.warn_deprecated(name)`.

**ABI FFI shell:**
- Cada struct começa com `version: u32`.
- Adicionar campo: incrementa versão; shells antigas ignoram.
- Remover/mudar tipo: major-bump no SDK shell, requer update sincronizado.

**Save format:** ver HR-14.

### 12.4 Save/load e persistência
- Snapshot do mundo ECS via `Reflect` + postcard.
- Replays: input log + seed + versão de engine; replay reproduz estado bit-a-bit em modo determinístico.
- Cross-platform: save no iPad abre no PC. Endianness fixa (little-endian no wire).
- Encryption: opcional via `ring` (AES-GCM); chave derivada de Game Center / Play Games user ID se desejado.
- Migration: HR-14.
- Recovery: corrupção detectada via blake3 do save; oferece carregar último checkpoint conhecido bom.

### 12.5 Observabilidade e crash handling
**Em produção:**
- Crash reporter: integração com `sentry-rust` opcional (feature `crash-reporter`).
- Symbolicação: dSYM (iOS), .symbols (Android), .pdb (Windows) uploaded em release pipeline.
- Telemetry opt-in obrigatório por privacidade. Default: off. Usuário liga em settings.
- Log policy: `tracing` com filter `info` em release; rotacionado in-device (max 10 MB, 5 arquivos).
- Performance counters: amostragem 1% em produção, agrega métricas de frame time, GC pause, OOM warnings.

**Apple privacy manifest (`PrivacyInfo.xcprivacy`):**
- Declarado em `shells/ipad/PrivacyInfo.xcprivacy`.
- Lista APIs "required reason" usadas (file timestamp, system boot time, user defaults).
- Sem isso, AppStore rejeita.

**Panic handling:**
- Global panic hook em `ph2d-core::panic` registra: stack trace, ECS state hash, último frame ID.
- Tenta save de emergência em arena reservada (1 MB pré-alocados).
- Em release: report + restart graceful se possível; em debug: trap.

**OOM:**
- iOS: `applicationDidReceiveMemoryWarning` → `host_low_memory()` → core dropa caches não-essenciais (atlas LRU, audio decompressed).
- Android: similar via `onTrimMemory`.

**GPU device lost / surface lifecycle:**
- wgpu lifetime pode quebrar (drivers crashing, backgrounding mobile).
- Protocolo formal de recovery por variant de `wgpu::SurfaceError` (Lost/Outdated/Suboptimal/Timeout/OutOfMemory) + handshake background→foreground em mobile + interação com modo determinístico: vide [ADR-0020](../docs/architecture/decisions/0020-surface-lifecycle.md). `SurfaceContext::acquire_frame()` em `ph2d-gpu` é o único caminho público.

### 12.6 Acessibilidade e i18n
- HR-12 e HR-15 são obrigatórias.
- A11y tree mantida em `ph2d-a11y::Tree`; nodes contém role, label, value, actions, bounds.
- Shells convertem para API nativa.
- Editor distribuído passa em testes automatizados de acessibilidade (axe-like) em CI.
- i18n via Fluent: bundle `.ftl` por idioma, em `locales/<lang>.ftl`. Default: pt-BR + en-US.
- Tradução comunitária aceita via PR; ADR-0011 governa fluxo.
- Reduced motion respeitado em transições e particles.
- Color contrast WCAG AA mínimo no editor; AAA quando possível.

### 12.7 Build, CI e release
- **Toolchain:** `rust-toolchain.toml` no root pina versão exata.
- **Workspace:** `cargo build` builda tudo; cada shell tem build próprio.
- **iOS:** `cargo lipo` + Xcode. Code signing automatizado via fastlane. TestFlight para staging.
- **Android:** `cargo ndk` + Gradle. ABI splits: arm64-v8a (primary), x86_64 (testing). NDK r28+.
- **Web:** `wasm-pack` + bundler (Vite ou esbuild). Deploy em CDN.
- **Reproducible builds:** `SOURCE_DATE_EPOCH` fixado, `Cargo.lock` versionado, dependências verificadas via `cargo-deny`.
- **Supply chain:** `cargo-audit` em CI; `cargo-deny` proíbe deps não-listadas em `deny.toml`.
- **Artefatos de release:**
  - Core: `.lib`/`.a`/`.dylib`/`.so` por target.
  - Shells: `.ipa`/`.apk`/`.aab`/`.exe`/`.app`.
  - Runtime Luau types: `runtime/luau/dist/` (tipo `.d.luau` + examples) publicado como tarball / git tag (escopo `@ph2d/runtime`). NPM continua válido apenas para o web shell bootstrap em `shells/web/`.
  - Asset cooker: binário portátil por OS de dev.

## 13. Fronteira Rust ↔ Shell — eventos e callbacks

Eventos shell → core (Swift/Kotlin/JS call para Rust):

```c
// Input
ph2d_event_pointer(x, y, pressure, tilt_x, tilt_y, kind, source, timestamp_ns);
ph2d_event_key(keycode, modifiers, kind, timestamp_ns);
ph2d_event_gamepad_button(gamepad_id, button, kind, timestamp_ns);
ph2d_event_gamepad_axis(gamepad_id, axis, value, timestamp_ns);
ph2d_event_pencil_squeeze(intensity, timestamp_ns);
ph2d_event_pencil_double_tap(timestamp_ns);
ph2d_event_pencil_hover(x, y, distance, timestamp_ns);

// Lifecycle
ph2d_event_resize(width, height, scale_factor);
ph2d_event_lifecycle(kind);  // foreground, background, low_memory, will_terminate

// IME (text input com composição CJK / acentuação)
ph2d_event_ime_begin();
ph2d_event_ime_compose(text_ptr, len, cursor_pos);
ph2d_event_ime_commit(text_ptr, len);
ph2d_event_ime_end();
```

Core → shell (Rust call para Swift/Kotlin via callback table):

```c
host_request_redraw();
host_open_url(*const u8, len);
host_show_keyboard(kind);
host_hide_keyboard();
host_haptic(kind, intensity);  // light, medium, heavy, success, warning, error
host_file_picker(filter_ptr, filter_len, callback_token);
host_file_save(suggested_name_ptr, len, data_ptr, data_len, callback_token);
host_a11y_update(tree_ptr, len);  // serializado em postcard, shell decodifica
host_low_memory_handled();  // ack após core liberar caches
```

Render: shell entrega `id<MTLTexture>` (iOS), `vk::Image` (Android), `wgpu::SurfaceTexture` (desktop) ao core no callback de frame. Core renderiza diretamente, devolve. Detalhes do interop em `ph2d-gpu::interop` (§10.4).

**Pencil específico (iPad):**
- `UIPencilInteraction` capturado em Swift, eventos squeeze/double-tap/hover viram chamadas dedicadas.
- Predictive touch via `predictedTouches` ativo em ink mode; **desligado** em UI mode (predição quebra snapping/gizmo).
- Latência alvo: ~9 ms end-to-end em iPad ProMotion (target documentado pela Apple desde iPadOS 13). Vale para qualquer iPad ProMotion, não exclusivo M-series; M4 + Pencil Pro aproxima 7 ms em casos otimizados.

## 14. Padrões para tasks comuns

**Adicionar um componente novo:**
1. Definir struct em crate apropriado, derivar `Component`, `Reflect`, `Saveable`.
2. Registrar no `TypeRegistry` no plugin do crate.
3. Adicionar ao schema Luau via `#[lua_export]` se exposto a script.
4. Se afeta render, adicionar ao extract phase em `ph2d-render`.
5. Se vai a save, adicionar entrada de versão (HR-14).
6. Doctest mínimo no `///`.

**Adicionar um shader:**
1. Arquivo em `crates/<crate>/shaders/<name>.wgsl`.
2. Embed via `include_str!` ou via `ph2d-gpu::ShaderRegistry`.
3. Pipeline em `pipelines.rs` do crate.
4. Bind group layout reutiliza `CommonBindGroups` se possível.
5. Adicionar ao matrix de cross-compile test em CI.

**Adicionar uma API ao Luau:**
1. Função Rust em `ph2d-script::bindings::<area>`.
2. Atributo `#[lua_export]`.
3. Tipo `.d.luau` regenerado automaticamente via `cargo run -p ph2d-bindgen`.
4. Documentar com Luau doc-comment (`--- @param`, `--- @return`) no atributo.
5. Verificar se faz sentido como ferramenta MCP (HR-10).

**Adicionar uma ferramenta MCP:**
1. Função em `ph2d-mcp::tools::<area>`.
2. Anotar `#[mcp_tool(name = "...", destructive = true|false)]`.
3. Schema gerado via `ph2d-bindgen`.
4. Se destructive: macro `#[mcp_destructive]` adiciona governance check.
5. Teste em `tests/mcp/<area>_<tool>.rs`.

**Adicionar um importador de asset:**
1. Crate em `tools/asset-cooker/src/importers/<format>.rs`.
2. Implementar trait `Importer { fn import(input: &Path) -> Result<CookedAsset> }`.
3. Registrar em `importers::registry`.
4. Adicionar fixture em `tests/fixtures/import/<format>/`.
5. Teste de determinismo (mesmo input → mesmo blake3).
6. Documentar em `docs/asset-pipeline.md`.

**Adicionar uma string i18n:**
1. Adicionar entrada em `locales/en-US.ftl` (master).
2. Adicionar tradução em `locales/pt-BR.ftl`.
3. Usar `t!("identifier", args...)` no código.
4. CI checa que toda chave usada existe em todos bundles core.

**Adicionar uma tool ao editor (fan-out via codegen, [ADR-0040](docs/architecture/decisions/0040-tool-as-isolated-feature-crate.md) fechado 2026-05-22):**

A receita virou **3 passos**: largar a pasta + rodar o sync + verificar. Sem edit central, sem variant novo de `EditorAction`. O contrato `Tool`/`RasterEditTool`/`PanelEvent` em `crates/ph2d-editor-core/src/tool.rs` está **congelado** (caps em `crates/ph2d-editor-core/tests/architecture_tool_contract_surface.rs`).

1. **Largue o crate** em `crates/ph2d-tool-<slug>/` (o glob de `workspace.members` cobre — NÃO edite o `Cargo.toml` raiz):
   - `Cargo.toml`: deps mínimas (`ph2d-tool-registry` + `ph2d-editor-core` p/ `Tool`/`FloatingPanel` se stateful + dom-específicas).
   - `src/lib.rs`: `pub const MANIFEST: ToolManifest = …;` + `pub fn register(reg: &mut Registry)` + `pub fn make() -> Box<dyn Tool>` se stateful.
   - `src/tool.rs` se stateful: `pub struct <Slug>Tool` + `impl Tool` (no mínimo `id` / `label` / `icon_slug` / `build_panel` / `as_any_mut`; sobrescreva `handle_panel_event` rotando NodeIds do panel docado → `apply_ui_edit(<UiEdit>::X)`; `is_default = true` apenas no Brush).
   - `src/icon.rs`: BezPath (Lucide 24×24 — placeholder OK, depois substitui pelo SVG real).
   - `src/algorithm.rs` se pure-Rust core (BgRemoval / Trim / etc).

2. **Rode o sync** (regenera `register_all` + `register_all_tools` + `Cargo` deps de `ph2d-tool-registry-init` a partir do scan das pastas):
   ```
   cargo run -p ph2d-tool-sync
   ```

3. **Verifique os 3 gates** (segundos, antes de subir):
   ```
   cargo test -p ph2d-tool-registry-init           # 3 staleness gates
   cargo test -p ph2d-editor-core --test architecture_tool_contract_surface  # contrato congelado
   cargo test -p ph2d-tool-<slug>                  # seu test
   ```

**Exemplos canônicos:**
- One-shot stateless: [`crates/ph2d-tool-trim-transparency/`](crates/ph2d-tool-trim-transparency/) ou [`make-square`](crates/ph2d-tool-make-square/) (só manifest + algorithm + icon, sem `impl Tool`).
- Stateful leve: [`crates/ph2d-tool-padding/`](crates/ph2d-tool-padding/).
- Stateful completo: [`crates/ph2d-tool-bgremoval/`](crates/ph2d-tool-bgremoval/) (preview cap + `RasterEditTool` + protect-mask + eyedropper via downcast).

**O que VOCÊ NÃO TOCA**: `Cargo.toml` raiz, `crates/ph2d-tool-registry-init/` (gerado), `crates/ph2d-editor-core/src/tool.rs` (contrato congelado — cap-bump exige amendment de ADR-0040), `EditorAction` (sem variant per-tool — use os 4 genéricos: `ActivateTool`, `OneShotImageOp`, `ToolPanelEvent`, `CancelActiveTool`).

**Se a tool tem panel docado próprio**: o `crates/ph2d-panel-<slug>/` é OUTRO crate (vide DIRETRIZ §3.2); ele pushea `EditorAction::ToolPanelEvent(PanelEvent::SetValue|Click(id, …))` e o shell rota via `Tool::handle_panel_event` automaticamente.

**REGRA DURA — UI canônica = Widget Gallery (DIRETRIZ §4.2):** o painel `ph2d-panel-widget-gallery` (seed em `crates/ph2d-editor-core/src/screens/hero/pre_populate.rs`, showcase em `crates/ph2d-editor-core/src/widget/showcase/`) é a ÚNICA fonte de verdade para layout, registro e wiring de widgets. Painel novo COPIA literalmente — não inventa. Em especial:

- **Slider + chip pareados → SEMPRE** `store.link_slider_number(slider, chip)` no populate. Sem o link você acaba escrevendo mirror manual em `apply_event` que dessincroniza entre frames e o clamp `0..1` não engata. Storage de chip e slider ficam no MESMO espaço `0..1`; unidade natural ("2.00", "+0.30", "8") vai via `display_override` no `paint_slider_with_chip_layout` (paint-only).
- **Chip pill (`paint_number_chip`, sem setinhas) → SEMPRE** `store.mark_chip_no_stepper(chip)` no populate. O dispatch carve uma coluna de 16-22 px no lado direito de TODO `NumberInput` como hit-zone de stepper; pra pill (sem arrows visíveis) isso vira phantom continuous-hold (`number_stepper_hold` arma e `dispatch_tick` incrementa a cada 30 ms com cursor parado).
- **Tempo real no canvas (game engine!) → tool stateful que altera pixels expõe `take_params_dirty()` + `preview_rgba()`**, e o shell tem `render_loop/<slug>_bridge.rs` espelhando `shells/desktop/src/render_loop/bgremoval_preview.rs` — refresh do cache `Arc<Vec<u8>>` quando `take_params_dirty()`, paint via `vector_scene.draw_image_rgba` sobre o footprint da sprite, cache zerado em Apply/deactivate. Sem isso o painel muda valor e canvas fica congelado.
- **`apply_event` é forwarder thin**: ouve `ValueChanged(slider_id_OR_chip_id)`, lê `store.slider(slider_id).value`, emite `PanelEvent::SetValue(slider_id, track)`. Slider/chip já estão sincronizados via `link_slider_number` — não escreva mirror manual.

Cada uma dessas regras já queimou ≥1× na slot 1 do Color Equalization (commits `3bf8806`, `903d63c`, `2f58b73`, `7b5f7c1`). Coordenador bounce se faltar qualquer item do checklist em DIRETRIZ §4.2.

## 15. Anti-patterns (NÃO faça)

- ❌ `Arc<Mutex<T>>` em hot path. Use `parking_lot::RwLock` raramente, prefira channels (`crossbeam`) ou ECS.
- ❌ `async fn` no core sem necessidade. Sync por default; async só em `ph2d-asset::loader` e `ph2d-net`.
- ❌ Strings como identificadores em runtime. Use `Handle<T>` ou `Entity`.
- ❌ Singleton globais. Use Resources do ECS.
- ❌ Macro mágica que esconde lógica. Macros pra reduzir boilerplate ok; pra esconder controle de fluxo, não.
- ❌ Adicionar dependência sem revisão. Cada crate é supply chain.
- ❌ Misturar pixel space e world space na mesma função.
- ❌ `if cfg!(target_os = ...)` no core. Vai pra trait `PlatformHost`.
- ❌ Reimplementar algo que `kurbo`/`glam`/`bevy_ecs` já fazem bem.
- ❌ Performance otimização sem profile. Measure first; `tracing` + `puffin` ou `tracy`.
- ❌ Hardcoded string em UI (HR-15).
- ❌ Save sem migração (HR-14).
- ❌ MCP destructive sem token (HR-11).
- ❌ Widget sem `Accessible` (HR-12).
- ❌ `unwrap()` em código não-test. Em prototipagem, `expect("razão clara")`; em produção, propaga.
- ❌ Confiar em ordem de execução de invocations em compute shader determinístico.
- ❌ `bind_group_layout` derivado por reflection (`layout: PipelineLayout::Auto`) em pipeline novo. Sempre `PipelineLayoutDescriptor` explícito (§10.5).
- ❌ `wgpu::Surface::get_current_texture()` direto sem matchear todas as variantes `SurfaceError`. Use `SurfaceContext::acquire_frame()` em `ph2d-gpu` — único caminho público (ADR-0020).
- ❌ Criar `RenderPipeline` ou `BindGroup` dentro de `RenderGraph` node execution. Só em init / on-resize. Cache agressivo em `ph2d-gpu::pipeline_cache`.
- ❌ Iteração de `std::HashMap`/`HashSet` em código que serializa state lateral, gera snapshot determinístico, ou roda em lockstep/rollback. HR-5 + ADR-0022; lint via `clippy.toml`.
- ❌ Re-exportar tipos `wgpu::*` ou `winit::*` na API pública de qualquer crate. Único re-export legítimo é `ph2d-gpu` para tipos cosméticos como `wgpu::TextureFormat` quando necessário; arquitetura interna fica isolada (Comfy post-mortem).
- ❌ GPU compute em qualquer cálculo cujo output entra em `SimWorld` (HR-5; ADR-0021 reforça por TIPO via `SimComponent` trait).
- ❌ Resize não-coalescido no shell desktop. Descartar resize events intermediários do mesmo frame (wgpu issues #2301/#3868/#5353).
- ❌ Async runtime no core além de `ph2d-asset::loader` e `ph2d-net::transport`. **Async morre na fronteira da shell** (§12.2).
- ❌ Assumir `SharedArrayBuffer` no web target sem confirmar headers COOP/COEP. Sempre fallback single-thread elegante (§11.12).
- ❌ **God-file em `shells/*/src/`** (HR-18). Crescimento por inflação de função existente em vez de extração para módulo novo. Aplica especialmente a `main.rs`, `render_frame()`, `window_event()`, `resumed()` — todos historicamente alvos de incremento descontrolado pré-ADR-0027.
- ❌ **Manual NodeId range allocation**. `screens/hero/ids.rs` antigamente alocava ranges 100..199 / 200..299 / etc à mão; convention-by-discovery substitui isso por `hash_node_id("tool.<slug>")` em `ph2d-tool-registry::node_id` (FNV-1a 64-bit const fn, collision-detected at registry build). Chrome fixo legacy retém consts; chrome derivado de tool-crates usa hash.
- ❌ **Editar registries centrais ao adicionar tool**: `lib.rs::pub use`, `tools/mod.rs`, `widget.rs`, `icons.rs` enum, `screens/hero/fixture.rs::topbar_clusters()`, `Cargo.toml` raiz. A receita canônica é `crates/ph2d-tool-<slug>/` com `pub fn register` + `pub const MANIFEST` + UMA linha em `crates/ph2d-tool-registry-init/src/lib.rs::register_all`. Vide ADR-0027 + plano `docs/Migracao/2026-05-convention-by-discovery.md` Apêndice A.

## 16. Estratégia de testes

| Tipo | Onde | Quando roda |
|---|---|---|
| Unit | `src/` `#[cfg(test)]` | `cargo test` |
| Doctest | `///` examples | `cargo test --doc` |
| Integration | `tests/<crate>/` | `cargo test` |
| Property-based | `tests/proptest/` via `proptest` | `cargo test` |
| Golden image render | `tests/golden/` | CI Linux + Mac mini M2; SSIM ≥ 0.995 vs baseline |
| Frame budget bench | `tests/budget/` via `criterion` + `ph2d-bench` | CI; baseline em git, regressão > 5% falha |
| Determinism replay | `tests/determinism/` | CI Linux + Mac + Windows; hash do estado final compara |
| Shader cross-compile | `tests/shaders/` | CI; naga compila WGSL → SPIR-V/MSL/HLSL/GLSL e diff contra reference |
| Fuzz | `tests/fuzz/` via `cargo-fuzz` | Diário em CI dedicado; targets: postcard parser, MCP request, script bridge, importadores |
| Architecture | `tests/architecture/` | CI; greps + stub-imports verificam HR-1, HR-7 |
| MCP governance | `tests/security/mcp_governance.rs` | CI; tenta destructive sem token |
| A11y | `tests/a11y/` | CI; widget tree validation |
| Cross-shell smoke | `tests/shells/` | CI Mac runner roda iPad sim, Linux runner roda Android emulator |

**Aprovação de baseline golden image:** mudança requer revisão humana + ADR se a mudança visual é arquitetural; senão PR review aprova `update-baseline` flag.

## 17. Definition of done

Toda mudança não-trivial precisa:

- [ ] Compila sem warnings em todos os targets ativos.
- [ ] `cargo test` passa, incluindo doctests.
- [ ] `cargo clippy -- -D warnings` clean.
- [ ] Golden image atualizado se afeta render output.
- [ ] Frame budget bench rodado se afeta hot path; sem regressão > 5%.
- [ ] Determinism replay passa se mexe em estado simulado.
- [ ] Documentação `///` em novos `pub`.
- [ ] Schema MCP regenerado se adicionou `#[lua_export]` (HR-10).
- [ ] Migration script se mudou save format (HR-14).
- [ ] Strings novas em UI passam por Fluent (HR-15).
- [ ] Widget novo implementa `Accessible` (HR-12).
- [ ] Memory budget atualizado se subsistema muda footprint (HR-13).
- [ ] Se cruza FFI: smoke test em pelo menos uma shell.
- [ ] Se afeta API Luau: `.d.luau` regenerado via `cargo run -p ph2d-bindgen` (HR-10).
- [ ] Changelog entry em `CHANGELOG.md` para mudanças user-facing.
- [ ] ADR criado se mudança arquitetural; ADR linkado no PR.

## 18. Quando ficar em dúvida

Hierarquia de decisão (ordem importa):

1. **Performance no hot path** > tudo
2. **Determinismo onde prometido** > conveniência
3. **Segurança (sandbox, MCP governance)** > facilidade
4. **Acessibilidade** > UX bonita
5. **UX nativa de iPad** > uniformidade de codebase
6. **APIs estáveis** > APIs elegantes
7. **Reproducibilidade de build** > velocidade de build
8. **Compreensibilidade por LLM** > brevidade clever

Se uma decisão não cabe em nenhuma das 8 acima e não está clara: **pergunte ao Enio antes de implementar**. Não adivinhe arquitetura. Abra issue com label `arch-question` ou pingue diretamente.

## 19. ADRs — política e índice

ADRs vivem em `docs/architecture/decisions/NNNN-titulo-em-kebab-case.md`. Numeração monotônica.

**Template:**

```markdown
# ADR-NNNN: Título

**Status:** Proposed | Accepted | Superseded by ADR-XXXX | Deprecated
**Data:** YYYY-MM-DD
**Decisor(es):** Enio + ...

## Contexto
O quê e porquê precisa de decisão.

## Decisão
O que decidimos.

## Consequências
Positivas, negativas, neutras.

## Alternativas consideradas
Listar com motivo de rejeição.
```

**ADRs canônicos (status real, atualizado 2026-05-09):**

| # | Título | Status |
|---|---|---|
| ADR-0001 | Editor é a engine | esperado (HR-7 cobre por enquanto) |
| ADR-0002 | Rust + Vello + wgpu como pilar | esperado (§5 + §6 cobrem por enquanto) |
| ADR-0003-rev2 | ECS choice — bevy_ecs 0.18 | **Accepted** ([0003-ecs-choice.md](../docs/architecture/decisions/0003-ecs-choice.md)) |
| ADR-0004 | Vello em alpha: risco aceito e mitigações | esperado |
| ADR-0005 | ~~TypeScript como gameplay primário~~ Luau ratificado | superseded por ADR-0019 |
| ADR-0006 | MCP first-class + governance | esperado (HR-10/HR-11 cobrem; ph2d-mcp skeleton em [crates/ph2d-mcp/](../crates/ph2d-mcp/)) |
| ADR-0007 | Hardware mínimo (matriz §4) | esperado |
| ADR-0008 | Shell texture interop via wgpu-hal | esperado (citado em §10.4 e ADR-0020) |
| ADR-0009 | Roadmap Holographic Radiance Cascades | esperado |
| ADR-0010 | Heap script default 64 MB; AAA opt-in | esperado (§12.1 cobre números) |
| ADR-0011 | Tradução comunitária e governança i18n | esperado |
| ADR-0019 | Spike scripting output (Luau ratificado) | **Accepted** ([0019-spike-scripting-output.md](../docs/architecture/decisions/0019-spike-scripting-output.md)) |
| ADR-0020 | Surface lifecycle e device-lost recovery | **Accepted** ([0020-surface-lifecycle.md](../docs/architecture/decisions/0020-surface-lifecycle.md)) |
| ADR-0021 | Fronteira simulation ↔ presentation (SubWorld) | **Accepted** ([0021-simulation-presentation-boundary.md](../docs/architecture/decisions/0021-simulation-presentation-boundary.md)) |
| ADR-0022 | Banimento HashMap em simulation crates | **Accepted** ([0022-no-hashmap-in-simulation.md](../docs/architecture/decisions/0022-no-hashmap-in-simulation.md)) |
| ADR-0023 | UI/UX baseline — Procreate-style canvas-first + WCAG 2.2 AA + AccessKit | **Accepted** ([0023-ui-ux-baseline.md](../docs/architecture/decisions/0023-ui-ux-baseline.md)) |
| ADR-0024 | Editor input pipeline + retained widget state (Modelo B + plano HR-3 zero-alloc) | **Accepted** ([0024-editor-input-and-widget-state.md](../docs/architecture/decisions/0024-editor-input-and-widget-state.md)) |
| ADR-0027 | Convention-by-discovery + Shell decomposition + HR-18 (tool-as-crate, registry-init, manifest-driven chrome) | **Accepted** ([0027-convention-by-discovery.md](../docs/architecture/decisions/0027-convention-by-discovery.md)) |
| ADR-0028 | Wave 2 — `build.rs` codegen (tokens + icons) + design canonical TOMLs + lint guards + HR-18 ativo; **Wave 4 extends**: spacing/radius/stroke/density/chrome top-level + typography codegen + `StrokeToken` enum + `design_token_sync` cross-val + `no_literal_color` non-hex paths + `no_magic_numeric` lint (warn mode) | **Accepted** ([0028-wave-2-codegen-design-canonical.md](../docs/architecture/decisions/0028-wave-2-codegen-design-canonical.md)) |

ADRs proibidos sem rever este SKILL: qualquer um que mexa em HR-1 a HR-17.

## 20. Última nota para a LLM lendo isso

Este projeto é opinionado por design. Quando algo parecer estranho, há provavelmente uma razão registrada em ADR (`docs/architecture/decisions/`). Leia o ADR antes de propor mudança contrária. Se não houver ADR, é candidato a virar um.

Quando você (LLM) implementar algo:

1. Cite a HR aplicável no commit message ("HR-3: pool pré-alocado em vez de Vec").
2. Se não consegue satisfazer uma HR, isso é sinal de que ou o design precisa de ADR ou você está fazendo a tarefa errada — pare e pergunte.
3. Memory budget e frame budget são contratos. Estourar sem flag é bug, mesmo que o teste passe.
4. Compreensibilidade > brevidade clever. Outra LLM vai ler isso.
5. Quando documentar API nova, escreva pensando na próxima LLM, não no humano. Concrete > abstract; example > prose.

A barra é alta porque o projeto pretende ser melhor que Unity e Godot em 2D. Tratamento de gambiarra é "obrigado, mas não, obrigado".
