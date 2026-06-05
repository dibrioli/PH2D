═══════════════════════════════════════════════════════════════════
HANDOFF → COORDENADOR · Vector W6 — PROCEDURAL FILL FOUNDATION **FECHADO**
Autor: Implementador Vector (sessão W6) · 2026-06-05
Origem: docs/HANDOFF_vector_w6_procedural_fill_impl.md (T6.1–T6.3)
═══════════════════════════════════════════════════════════════════

## TL;DR
`ph2d-vector-fill` pronto. Commit local **`ccfb82a`** (scoped: só o crate + Cargo.lock,
nada alheio). Codegen + cache + UBO + CPU-eval verdes. **Precisa wiring no renderer
(Coord).** Não pushei (fast mode).

## O que landou (T6.1–T6.3, ADR-0060 §2.1–§2.3)
Crate ISOLADO `crates/ph2d-vector-fill/` (drop-crate; glob-membered, zero edit central):
- **lib.rs** — `FillGraph` (DAG SmallVec) + `FillNode` (enum **17 CONGELADO**) + `Connection`
  + `FillType` + enums de controle (`NoiseKind`/`CoordMode`/`MathOp`/`BlendMode`) + `validate()`
  (acíclico / type-check / single-driver / output=Color / cap 16).
- **wgsl_codegen.rs** — DAG→WGSL **SSA em `fill_main`** (cada nó = 1 `let v{id}`, dependency
  order, dead nodes podados) + **topology hash** blake3 (só tags+conexões+output+backend;
  param/enum NÃO entram) + naga validate (wrap fragment p/ checar binding path).
- **ubo.rs** — `FillParamsUbo` `#[repr(C, align(16))]` Pod, std140-clean (vec4-lanes; sem
  `array<f32,N>`-stride-trap). `from_graph()` + setters zero-alloc (`as_bytes()` = `bytes_of`).
- **cache.rs** — LRU hand-rolled (BTreeMap, **HR-5/ADR-0022** — sem HashMap; sem `lru`/`directories`
  dep) + `compile_fill()` (codegen+naga) + contadores `hits()`/`compiles()`/`hit_rate()`.
- **eval.rs** — evaluador CPU de referência (espelha cada helper WGSL linha-a-linha; lê o MESMO
  UBO → paridade por construção; `noise1` reusado do `ph2d-expr`).

**12 nós procedurais implementados** (codegen+CPU real): Solid, LinearGradient, RadialGradient,
Noise(Simplex/Perlin/Worley/Fbm), Voronoi, Ramp, Mix(6 modos), Bump, Coord, Math(14 ops), Time,
Random. **5 stubs** (`NotYetImplemented`, variante no enum p/ não-bumpar o cap): MeshGradient,
Pattern, ProceduralShader, Image, ImageSample.

## Gates verdes (DoD)
- **5 codegen-gold fixtures** — `tests/golden/*.wgsl` committed; string WGSL determinística/estável.
- **naga-valida os 12 nós** (`tests/codegen.rs`, todas as variantes de noise/blend).
- **cache hit-rate >95%** — 1 compile / 60 frames animados = 0.983.
- **`procedural_fill_no_recompile_on_animate`** — animar freq **+ enum noise-kind** por 60 frames =
  **1 compile** (`tests/gates.rs`).
- 33 testes (dev) + 5 gates (`--release`) · clippy `--all-targets` / rustfmt-1.95 / machete limpos.

## ⚠️ PRECISA COORD (cross-crate / contrato / dep — fora do meu isolamento)
1. **Region→FillGraph reference** = mudança no contrato CONGELADO `ph2d-vector-doc`
   (`StyleTable`/`FillRef`, gate `architecture_vector_contract_surface`) → **Coord + ADR-0056-amendment**.
   Meu crate é autossuficiente; o `FillGraph` vive nele — falta a region apontar pra ele.
2. **Smoke "vê live"** = embed do `fill_main` no pipeline de fill do `ph2d-vector` (renderer) + bind
   do `FillParamsUbo` em `@group(0)@binding(0)` + update per-frame. Espelho do `vector_graph_bridge`
   dos geometry nodes. **Contrato de embed:** o WGSL é embeddable (struct+helpers+`fn fill_main(coord)->vec4`,
   SEM entry point — Coord injeta no fragment dele). Group/binding podem precisar rebind.
3. **Cross-OS bit-identity GPU** (det-mode, ADR-0060 §2.6) — render nas 3 OS + hash. É GPU-test (CI matrix),
   não unit. Minha paridade é **formula-level** (CPU=WGSL por construção; `noise1` integer-hash é bit-idêntico;
   o FMA-ordering float é o det-mode opt-in posterior).
4. **On-disk cache** (`directories` dep, spec §5.5.1 Windows-UNC) — **dep NOVA = decisão de stack**.
   Deixei in-memory only (satisfaz todos os gates W6). Quando ratificar, é incremento sobre `cache.rs`.

## Decisões que tomei (padrão-ouro, documentadas no código)
- **Codegen SSA** (não function-per-node do pseudocódigo) — DAG-correto, sem recomputar subárvores
  compartilhadas; helpers **só-os-usados** (shader enxuto) mas cada enum com TODAS as variantes no
  `switch` (recompile-free intacto).
- **Bump** (Vec3) é implementado mas **inalcançável em W6** (nenhum nó consome Vec3; output=Color) —
  fica forward-compat até existir nó de lighting/material. CPU-eval devolve normal flat (sem dpdx).
- **Coord World/Screen** = identity até o renderer fornecer matrizes (dispatch via UBO já pronto → vira
  UBO-write, não recompile).
- **12 vs 5**: implementei TODOS os procedurais puros (gradients/bump inclusos), stub só os 5
  resource-bound (precisam W7 diffusion / W8 brush bridge / texture binding). Documentado em `lib.rs §W6 scope`.

## Próximo (W7, NÃO toquei — ADR-0060 §2.5, 35 dias research-grade)
Diffusion curve Poisson (WoS/multigrid) + JBU upsample + tier matrix + `poisson_cpu.rs`/`diffusion.wgsl`.
Os 5 stubs viram reais aí (MeshGradient primeiro).
═══════════════════════════════════════════════════════════════════
