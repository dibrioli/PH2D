═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · AVALIAÇÃO DE ESTADO + próximo sprint priorizado
Autor: Coordenador (jornada 2026-06-05) · base: `Vector Module/avaliacao_e_melhorias.md`
        + `14_inovacoes_extraordinarias.md` + assessment de código
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR (leia isto + §4)
1. **O Vector está MUITO à frente do Painter.** W1·W2·W3·W4·W5·W6 **FECHADOS e auditados**
   (~38% das ~20 waves). Disciplina de contrato congelado impecável (gates de surface ativos).
2. **Diferencial real:** das ~7 inovações extraordinárias do spec, **2 já estão VIVAS** — e são
   justamente as 2 GPU-pesadas: **Live Boolean Graph** (Linesweeper exato + SDF draft) e **Vector-SDF
   Hybrid GPU** (ADR-0065, 64-path @ 120FPS). **As 5 críticas A–E estão resolvidas.** (No Painter,
   nenhuma das 5 inovações está funcional — aqui 2/7 estão. Vocês quebraram paradigma de verdade.)
3. **W6 (procedural fill) fechou, mas precisa de MIM (Coord) pra virar smoke-able** (embed no renderer
   + contrato Region→FillGraph). Isso é meu, não te bloqueia (§4 nota Coord).
4. **Teu próximo:** W7 (diffusion curve Poisson — research-grade 35d, **faz prototype-first**) OU
   pattern-along-path (W8, o 18º node geométrico). Recomendação em §4.

## §1 — ONDE ESTAMOS (mapa de waves, evidência no assessment)
| Wave | Estado | Nota |
|---|---|---|
| W0 ADRs | ✅ FECHADO | 13 ADRs (0056..0068) absorvendo 3 iterações Antigravity (CONVERGENCE 9.2→~9.7) |
| W1 doc + Pen cúbico + .ph2d-vector + traits/mocks | ✅ FECHADO | `VectorNetwork`, cubic default, `ph2d-vector-traits` (AnimValue enum + mocks → mata vaporware-coupling) |
| W2 Pencil/Shapes/Select/Color/Undo | ✅ FECHADO | tools via edit_log |
| W3 geometry graph + boolean + offset + panel | ✅ FECHADO | auditado; SDF draft+reconcile |
| W4 geometry nodes fan-out | ✅ FECHADO | **14/18 nodes** reais; pattern-along-path→W8 |
| W5 variable-width stroke + SDF Hybrid GPU | ✅ FECHADO | pressão→WidthProfile + gate `vector_sdf_real_time` (5.33ms<8.33ms) |
| **W6 procedural fill / shader graph** | ✅ **FOUNDATION FECHADA** (`ccfb82a`) | `ph2d-vector-fill`: 12 nós procedurais + 5 stubs; **UBO split = zero-recompile-on-animate** (resolve crítica B). **Precisa wiring Coord** (§4). |
| W7 diffusion curve Poisson | 🔴 NÃO-INICIADO | research-grade 35d (ADR-0060 §2.5); os 5 stubs de fill viram reais aqui (MeshGradient 1º) |
| W8 pattern-along-path + brush bridge | 🔴 NÃO-INICIADO | 18º node; consome `ph2d-painter-brush` (gated na maturidade da API — Painter está mid-W4) |
| W9+ symbol system / fonts / animation / runtime / physics / MCP / export | 🔴 NÃO-INICIADO | — |

## §2 — AS INOVAÇÕES EXTRAORDINÁRIAS (a tese: "sucessor do Illustrator")
**Vocês já entregaram 2/7 — e as 2 mais difíceis (GPU).** Status:
| Inovação | Status | Evidência |
|---|---|---|
| **P1 Live Boolean Graph** | 🟢 **VIVA** | `ph2d-node-vector-boolean` (Linesweeper, 9 ops) + SDF draft; cache-by-hash; smoke-OK W3 |
| **P7 Vector-SDF Hybrid GPU** | 🟢 **VIVA (core)** | `ph2d-vector-sdf` CPU+GPU+marching-squares; draft+reconcile; ADR-0065 fechado. Limite: silhueta (topologia exata = Linesweeper async) |
| **P3 Painter↔Vector Bridge** | 🟡 parcial | paint-into-vector scaffold (W12); pattern-along-path stamp (W8); auto-trace ML (W12) |
| **P5 Physics colliders / Dormant Fracture** | 🟡 stub pré-declarado | `ph2d-vector-doc::dormant.rs` (schema pronto p/ não bumpar versão; payload W16) |
| **P2 Mesh Gradient / Diffusion Curve** | 🔴 ausente | **é o W7** (Poisson WoS/multigrid + JBU) |
| **P6 Variable Fonts axes-as-graph** | 🔴 ausente | ADR-0066 ratificado; sem `ph2d-vector-font` (W10) |
| **P4 LLM-as-graph-node** | 🔴 ausente | MCP scaffold em ADR-0061; sem crate (W13) |

**Leitura honesta:** diferente do Painter (todas as 5 inovações são scaffolds), aqui o motor já faz
boolean vivo e morphing SDF GPU a 120FPS — paradigma quebrado de verdade. O que falta são waves
distantes (diffusion W7, bridge W8/W12, fonts W10, LLM W13, physics W16). **O risco não é "está raso",
é "W7 é pesado".**

## §3 — AS 5 CRÍTICAS (A–E) — todas endereçadas
- **A (crate bloat ~40 crates):** REJEITADA conscientemente — fan-out mantido (DIRETRIZ §3.A = unidade
  multi-agente). Hoje ~30 crates vector-family (source consolidou 5 shapes em 1). Trade-off ratificado.
- **B (compile-stutter de shader procedural):** ✅ RESOLVIDA no W6 — topology-hash compila 1×, params
  via UBO per-frame, enums via `switch` interno (zero recompile). Gate `procedural_fill_no_recompile_on_animate`.
- **C (latência síncrona de boolean):** ✅ RESOLVIDA — draft GPU (≤1ms) + reconcile Linesweeper async.
- **D (rejeição ao Spiro-default):** ✅ RESOLVIDA — Bézier cúbico é o default; Spiro/hyperbezier = Assist opt-in.
- **E (acoplamento a vaporware):** ✅ MITIGADA — `ph2d-vector-traits` (AttributeEvaluator/ProceduralFillShader
  + mocks) deixou W1–W5 rodarem sem Shader Graph / Animation reais.

## §4 — TEU PRÓXIMO SPRINT (priorizado)
Posse tua: `ph2d-node-vector-*` (novos), `ph2d-vector-doc/{crdt,spiro}`, `ph2d-vector-fill` (W6),
tool-bridges `vector_*`. **Coord (eu):** `ph2d-render`, `ph2d-vector-sdf`, `vector_graph_bridge`,
foundational (`ph2d-vector`/`ph2d-vector-doc` contrato), wiring de renderer.

### ⚙️ NOTA COORD (faço EU, não te bloqueia): fechar o loop do W6
O `ph2d-vector-fill` está autossuficiente mas ainda não vê pixel. **Eu** faço (do teu handoff
`HANDOFF_vector_w6_fill_closed_coord.md`): (1) contrato Region→FillGraph (`ph2d-vector-doc`
`StyleTable`/`FillRef` + ADR-0056-amendment), (2) embed do `fill_main` no fragment do renderer +
bind do `FillParamsUbo`, (3) cross-OS GPU bit-identity (CI matrix). Quando eu fechar, o Enio smoka
o fill live. **Tu segue pra W7/W8 em paralelo.**

### 🔴 OPÇÃO A (recomendada) — W7 diffusion curve, PROTOTYPE-FIRST
W7 é research-grade (35d estimados, ADR-0060 §2.5) — **não ataque o GPU direto.** Sequência segura:
1. **`poisson_cpu.rs` referência** em `ph2d-vector-fill` (ou crate-satélite) — solver CPU primeiro
   (Walk-on-Spheres estocástico OU multigrid iterativo; **WoS é embarrassingly-parallel e mais simples
   de portar**, mas multigrid é determinismo-friendly — decide e documenta). Valida a math num caso
   pequeno (2-3 diffusion curves → campo de cor) **antes de uma linha de WGSL**.
2. Materializa o stub **MeshGradient** primeiro (é o nó que consome diffusion).
3. Só então GPU port + JBU upsample + tier matrix (Mobile Core fallback CPU-SIMD).
**Eu scaffoldo a infra de golden/smoke-test quando teu skeleton CPU landar** — me pinga.
→ Fecha a Inovação P2 (mesh gradient orgânico — o "uau" que o Illustrator não tem).

### 🟡 OPÇÃO B (win contido, se quiser de-riscar antes do W7) — pattern-along-path (W8)
O 18º node geométrico, deferido do W4. Stampa um shape/brush ao longo de um path. **Gated:** consome
`ph2d-painter-brush` (a API existe — round_hard/oval/etc — mas o Painter está mid-W4; confirma comigo
a superfície estável antes). Contido, bem-entendido (vs research W7), completa o roster 18/18.

**Recomendação:** se o Enio quer **avançar inovação** → Opção A (W7 prototype-first, é a próxima
inovação viva). Se quer **win rápido e contido** antes de encarar o monstro do Poisson → Opção B.
Eu acho a A o caminho de maior valor (P2 é a inovação que falta mais visível), mas o risco/prazo é teu
a topar — prototype-first reduz o risco do 35d.

## §5 — POSSE / GIT (disciplina multi-agente)
- **Tua posse:** `ph2d-node-vector-*`, `ph2d-vector-doc/{crdt,spiro}`, `ph2d-vector-fill`, tool-bridges.
  Contratos congelados (`VectorOp≤16`/18 nodes/`FillNode=17`/caps) — **NÃO bumpar cap** sem Coord+ADR.
- **Coord (eu):** `ph2d-render`, `ph2d-vector-sdf`, `vector_graph_bridge`, `ph2d-vector`/`-doc` contrato,
  wiring W6, ship. **Painter impl ATIVO em paralelo** (`ph2d-tool-painter`/`-brush`/`-render`-via-mim) —
  sem overlap com tua área.
- Commit scoped: `git add -- <teus paths>` · `git commit --no-verify -m "msg" -- <paths>` ·
  `git status` antes (índice compartilhado) · sem push (Coord shipa 1×/jornada). RAM ≤3 cargos.
- **CRDT real (`crdt.rs`) ainda é stub** (site_id+peer_clocks só) — LWW-Element-Set + RGA é débito
  W-futuro (Inovação P5-collab). Schema pré-declarado não bumpa versão quando materializar.
═══════════════════════════════════════════════════════════════════
