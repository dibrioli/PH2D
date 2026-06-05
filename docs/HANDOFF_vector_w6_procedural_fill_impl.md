═══════════════════════════════════════════════════════════════════
HANDOFF → IMPLEMENTADOR Vector · W6 — PROCEDURAL FILL FOUNDATION (shader graph)
Autor: Implementador Vector (sessão pós-W5-ship) · 2026-06-05 · contexto separado
Wave: §9 do plano + ADR-0060 (CONGELADO) + spec normativa 05_procedural_fill.md
═══════════════════════════════════════════════════════════════════

## §0 — ⚠️ ANTES DE TUDO (git + cadência)

**Baseline = HEAD LOCAL atual** (W1–W5 shipados: origin/main `1156441`, CI 3-OS verde
2026-06-05). NÃO rebase pra origin, NÃO reset. Sanity (DIRETRIZ §0):
`git status` limpo (só docs/`.vscode` alheios) + `cargo check -p ph2d-vector-doc` verde.

**Git discipline (crítico — multi-agente):**
- Commits **SCOPED**: `git add -- <só teus paths>` · `git commit --no-verify -m "msg" -- <paths>`.
  **NUNCA** `-A`/`-a`/`git add .`/`git stash`. `git status` antes de stage; `M`/`??` alheio → reporta.
- ⚠️ **Há outro implementador (Painter) com arquivos STAGED no índice compartilhado** — commita
  SEMPRE com pathspec (`git commit ... -- <teus paths>`), nunca confie no índice global.
- `cargo fmt --all`/`-p` reformata WIP alheio → formata só os TEUS com `rustup run 1.95 rustfmt <arquivos>`
  (o pin é `rust-toolchain.toml` = 1.95; rustfmt default diverge → fmt-skew no CI).
- **Você NÃO pusha** nem roda CI. Fast mode: commit local. O Coord absorve PRCI 1×/jornada.

## §1 — Onde estamos (W1–W5 FECHADOS — NÃO refaça)

| Wave | Estado | Crates |
|---|---|---|
| W1–W2 | ✅ data model + Pencil/Shape/Select/Pen/Color/Undo | `ph2d-vector-doc`, `ph2d-vector`, `ph2d-tool-vector-*` |
| W3 | ✅ geometry graph: source + boolean (9 ops, Linesweeper) + offset + bridge `ph2d-vector-kurbo` | `ph2d-node-vector-{source,boolean,offset}` |
| W4 | ✅ 11 geometry nodes (mirror/twist/roughen/corner-round/bend/scatter/recolor/outline-stroke/hatch/warp/width-profile) | `ph2d-node-vector-<slug>` |
| W5 | ✅ pressão→variable-width (Pencil live + persisted `WidthProfile`) | `ph2d-tool-vector-pencil` + bridge |

Substrato relevante p/ TI: **`ph2d-nodegraph`** (DAG/cook — geometry nodes usam), **`ph2d-expr`**
(IR + WGSL lowering + CPU eval — **tua maior alavanca**, vide §3), **`ph2d-vector-doc`** (carrier:
`StyleTable.fills: BTreeMap<FillRef, FillSolid>` — **só solid hoje**; o teu fill graph é o upgrade).

## §2 — TUA TASK: W6 = crate foundational `ph2d-vector-fill` (ADR-0060 §2.1)

Shader-graph procedural de fills, compilável a WGSL on-the-fly, com **topology compile 1× + UBO
update por frame** (resolve crítica B Antigravity — zero compile stutter on animate). Crate ISOLADO
(drop-crate; não toca contrato alheio — vide §4). Layout (ADR-0060 §2.1):

```
crates/ph2d-vector-fill/src/
├── lib.rs          FillGraph + FillNode enum + Connection + validate (T6.1)
├── wgsl_codegen.rs DAG → WGSL string + topology hash + cache by hash (T6.2)
├── ubo.rs          FillParamsUbo (params escalares dinâmicos por frame) (T6.3)
└── cache.rs        compile cache (in-memory LRU; on-disk = §5 deferível) (T6.3)
```

### T6.1 — FillGraph DAG + FillNode enum (spec 05 §5.1)
```rust
pub struct FillGraph { pub nodes: SmallVec<[FillNode;16]>, pub connections: SmallVec<[Connection;32]>, pub output_node_id: NodeId }
pub enum FillNode { Solid{color}, LinearGradient{stops,angle}, RadialGradient{..}, MeshGradient{gradient_id},
  Pattern{pattern_ref}, ProceduralShader{shader_id}, Image{image_ref}, Noise{kind,frequency,octaves},
  Voronoi{cells,jitter}, Ramp{palette}, Mix{mode,factor}, Bump{strength}, Coord{mode}, Math{op},
  ImageSample{image_ref,uv_input}, Time, Random{seed} }   // 17 nodes — CONGELADO (§4)
pub enum NoiseKind { Simplex, Perlin, Worley, Fbm{lacunarity,persistence} }
```
- **Validação:** DAG acíclico; `output_node_id` alcançável; type-check edit-time (Color→Color, f32→f32).
- **DoD:** parse+valida; nodes pilot **Noise + Ramp + Mix** funcionam offline (CPU eval).

### T6.2 — WGSL codegen (DAG → WGSL string) + cache (spec 05 §5.3)
- Visitor que percorre o DAG topologicamente, emite uma WGSL function chain (cada node → uma expr/fn).
- **Cache key = `blake3(topology_layout + backend_id)`** — só TOPOLOGIA entra no hash; params **NÃO**
  (vão pro UBO — §T6.3). `topology hash → WGSL` memoizado.
- **DoD:** 5 fixtures codegen-gold-test passam (WGSL string estável cross-OS); cache hit-rate >95% num
  cenário de animação 60 frames (params mudam, topology não → 1 compile).

### T6.3 — Topology vs UBO split (resolve crítica B — ADR-0060 §2.3, spec §5.4)
- **Topology hash → naga compile WGSL 1×** (naga JÁ in-tree via wgpu) → cache (memory LRU; on-disk = §5).
- **Params escalares** (cor, frequency, ramp pos, time, coord) → `FillParamsUbo` atualizado per frame,
  zero-alloc (HR-3). Shader lê `@group(0)@binding(0) var<uniform> params: FillParams`.
- **Enum control via UBO indexing** (L1F2): `NoiseKind`/`CoordMode`/`MathOp` **NÃO** geram WGSL
  condicional estático (recompile!). Em vez disso emita TODAS as variantes num `switch kind_idx { … }`
  interno, e troque o `u32` no UBO → ~100µs, **zero recompile** (ADR-0060 §2.3 tem o exemplo `noise_kind_dispatch`).
- **Topology change** → compile off-thread + swap atômico; durante o compile, render usa o template anterior.
- **DoD:** gate `procedural_fill_no_recompile_on_animate` — animate 60 frames de param + enum = **0 recompilations**.

### T6.4 — Audit + fechamento W6 = **Coord-orquestrado** (espelho T3.5/T4.13). NÃO faças.
Lentes: shader compile stutter (crítica B) · cross-platform shader output bit-identical · cache invalidation.

## §2.X — 🚧 FRONTEIRA W6 vs W7 (não estoure escopo)

**NÃO faças no W6** (é W7 — 35 dias, research-grade, ADR-0060 §2.5):
- **Diffusion curve / Poisson PDE / Walk-on-Spheres / JBU upsample / tier matrix** → W7.
- Os nodes `MeshGradient` / `Pattern` / `ProceduralShader` / `Image` / `ImageSample`: **declara a variante
  no enum** (o cap é 17, não bumpar depois), mas o codegen pode **emitir placeholder + `// TODO W7/W8`**
  OU retornar um erro `FillCodegenError::NotYetImplemented(node)`. Pilot real do W6 = **Solid, Noise,
  Voronoi, Ramp, Mix, Coord, Math, Time, Random** (os procedurais puros). Documenta o que é stub.

## §3 — 🔥 REUSE (a maior alavanca — NÃO reinvente)

**`ph2d-expr`** (`crates/ph2d-expr/`) já é o substrato "compile, not interpret" (ADR-0030/0033):
- `wgsl::to_wgsl(&Expr) -> String` + `wgsl::wgsl_prelude() -> &str` — **emite WGSL** (noise, mix, …).
- `eval::eval(&Expr, &dyn Bindings) -> f32` — **CPU reference** (a semântica que a GPU deve casar).
- `Func` tem `Noise` (→`noise1`), `Mix` (`a+(b-a)t`), `Sin/Cos`, etc. `expr.rs` tem `Const/Param/Unary/Binary/Call/Select`.
- **Gotcha documentado (reusa o cuidado!):** WGSL `fract(x)=x-floor(x)` (≥0) ≠ Rust `f32::fract` (sign-preserving);
  WGSL `mix(a,b,t)=a(1-t)+bt`. `ph2d-expr/eval.rs` já casa CPU↔GPU — **siga o mesmo padrão** p/ o cross-OS bit-identity.
- **`ph2d-expr` é SCALAR (f32).** Os fill nodes precisam de `vec2`/`vec3`/`Color` (Coord→Vec2, Bump→Vec3,
  outputs Color). Então: reusa o prelude WGSL (noise/mix) + a disciplina CPU/GPU-parity + o padrão `to_wgsl`,
  mas o **visitor tipado (Color/Vec2/Vec3)** é teu (o `ph2d-expr` cobre a parte escalar/Math/Noise).

Outras peças in-tree: **`naga`** (validar/compilar a WGSL gerada — cheque a versão no lock), **`blake3`**
(cache key), **`smallvec`** (FillGraph). NÃO reescreva noise/mix WGSL do zero.

## §4 — 🚫 NÃO TOQUE (anti-colisão) + CAPS CONGELADOS

- **`ph2d-vector-doc`** (`StyleTable`/`FillRef`/`FillSolid`) — **contrato congelado** (gate
  `architecture_vector_contract_surface`). Ligar `Region.fill` a um `FillGraph` (uma variante de fill
  procedural na StyleTable) é **mudança de contrato = Coord + ADR-0056-amendment** → **reporta, não edita**.
  Teu crate `ph2d-vector-fill` é **autossuficiente** (FillGraph vive nele; a referência da region é Coord).
- **`ph2d-vector`** (renderer) / **`render_loop`** / shell — o "user aplica fill, vê **live**" exige o
  renderer executar o shader compilado numa region = **wiring cross-crate = Coord** (espelho do
  `vector_graph_bridge` dos geometry nodes). Tu entregas o crate + testes offline; o Coord plumba o smoke visual.
- **`ph2d-nodegraph`/`ph2d-expr`** — substrato congelado (ADR-0039). Reusa via API pública, não edita.
- **Caps congelados (ADR-0060 §2.7 — NÃO bumpar):** **17 fill nodes** · cache LRU **256 MB** mem / **1 GB**
  disco · hit-rate **>95%** · compile timeout off-thread **5 s**. Gate `procedural_fill_no_recompile_on_animate`.

## §5 — DECISÕES / RISCOS (decida cedo; dep nova = reporta ao Coord)

1. **`directories` crate (dep NOVA — cache on-disk, T6.3 §2.4):** NÃO está no lock → **decisão de stack →
   reporta ao Coord** antes de adicionar. **Mitigação:** T6.1 + T6.2 + a **LRU in-memory** do T6.3 NÃO
   precisam de `directories`. Faz a foundation toda com cache em memória primeiro; o on-disk (`directories`
   + path layout cross-OS Linux/macOS/Windows-UNC/OPFS) é um incremento que pode esperar a ratificação.
2. **`lru` crate vs hand-roll:** uma LRU simples (BTreeMap + fila) é trivial e evita dep; ou reporta `lru`.
3. **naga API:** cheque a versão exata no lock antes de assumir a API de `Module`/validate (pode ter mudado).
4. **Determinismo (gate cross-OS bit-identity):** CPU eval reference + WGSL casando a semântica (vide §3 gotchas);
   `#pragma fma_off` + ordered reductions quando `deterministic` (ADR-0060 §2.6). Meça em `--release`.

## §6 — Onboarding (ordem de leitura)

1. **CLAUDE.md** §0 (inegociáveis) + §5 (estado Vector) + §6 (contratos).
2. **ADR-0060** (este contrato — topology vs UBO §2.3, enum-via-UBO, caps §2.7; o Poisson §2.5 é **W7, ignora agora**).
3. **Spec `05_procedural_fill.md`** (510 linhas, normativa) — §5.1 (FillGraph), §5.2 (17 nodes), §5.3 (codegen),
   §5.4 (UBO split). **Pseudocódigo é ilustrativo** — a API real é o substrato (`ph2d-expr`/`smallvec`); construa
   contra ele (memória [project_vector_node_opaque_carrier]).
4. **`ph2d-expr`**: `wgsl.rs` (`to_wgsl`/`wgsl_prelude`) + `eval.rs` (`eval`/`noise1`/gotchas) + `expr.rs` (`Expr`/`Func`).
5. DIRETRIZ §3.A (drop-crate) + `examples-fan-out.md`. Referência de nó-crate isolado: o que eu fiz em
   `ph2d-vector-kurbo` (satélite) e `ph2d-node-vector-*` (template).

## §7 — Velocidade (inner loop) + validação

- Slot CoW: `bash scripts/slot-seed.sh slot-impl-vector` → prefixe cada cargo com o `CARGO_TARGET_DIR` impresso.
  Inner loop = **`cargo check -p ph2d-vector-fill`** só. Teste/clippy/machete/fmt(1.95) **1× no fechamento**.
- Gates do fechamento (DoD): 5 fixtures codegen-gold · cache hit-rate >95% (anim 60 frames) ·
  `procedural_fill_no_recompile_on_animate` (0 recompiles) · cross-OS bit-identity (CPU eval = WGSL semantics).
- **Meça compile/UBO em `--release`** (dev=opt0 mente). O smoke "vê live" é wiring do Coord — reporta quando
  o crate fechar: "ph2d-vector-fill pronto, commit <sha>, codegen+cache+UBO verdes. Precisa wiring no renderer? (Coord)".

QUANDO FECHAR (ou por task), reporta ao Coord com o commit local + gates verdes.
═══════════════════════════════════════════════════════════════════
