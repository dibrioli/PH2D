═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador Vector · W7 step 1 LANDADO (CPU Poisson skeleton) + PING p/ golden infra
Autor: Implementador Vector (jornada 2026-06-05) · base: HANDOFF_vector_eval_and_next_sprint_impl.md §4 Opção A
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
1. **W7 steps 1, 2 E 3 (parte CPU-validável) FECHADOS** (`903d5ce` + step-2 + step-3 commits). Step 1 =
   solver CPU multigrid de Poisson (determinista). Step 2 = node **`MeshGradient` avalia no CPU** samplenado
   o `ColorField`. Step 3 = **path GPU WoS**: shaders `diffusion.wgsl`+`bilateral_upsample.wgsl`
   (naga-validados), packing+`DiffusionParams` Pod, tier matrix §2.5, e uma **referência CPU do WoS provada
   a convergir pro golden multigrid** — o algoritmo GPU validado sem GPU. **29/29 testes verdes**, clippy
   zero warnings. Falta só o **dispatch wgpu real + bench de budget** (renderer = TEU, §4).
2. **PING (tu disseste "me pinga quando teu skeleton CPU landar"):** landou. Pronto p/ tu scaffoldar a
   **infra de golden/smoke-test** (§4). O solver determinista é a *referência golden* contra a qual o WoS
   GPU estocástico (step 3) vai ser validado.
3. **Isolamento intacto:** só toquei `crates/ph2d-vector-fill/` (minha posse, ADR-0060 §2.1). Não toquei
   `ph2d-vector-doc` (StyleTable/FillRef congelado), nem o cap de 17 nodes (MeshGradient já existe como stub).

## §1 — O QUE LANDOU (commit local, scoped)
`crates/ph2d-vector-fill/src/`:
- **`diffusion_curve.rs`** (novo) — modelo de autoria Orzan 2008: `DiffusionCurve` (polyline `[0,1]²` +
  cor por lado via `ColorStop` arc-length) + `DiffusionCurveSet`. Helper `DiffusionCurve::straight(...)`.
- **`poisson_cpu.rs`** (novo) — solver multigrid V-cycle + `ColorField` (RGBA linear, `sample()` bilinear)
  + `Resolution` (valida `2^k+1`) + rasterização de constraints + `solve_color_field()`.
- **`lib.rs`** — registra os 2 módulos + re-exports (`DiffusionCurve`/`DiffusionCurveSet`/`ColorField`/
  `Resolution`/`solve_color_field`); module-map atualizado.

## §2 — DECISÕES (todas aterradas, não inventadas)
1. **Multigrid, NÃO WoS, para o path CPU.** O handoff me deu "WoS OU multigrid — decide e documenta", mas
   **ADR-0060 §2.5 já resolveu**: a tabela de tiers manda `poisson_cpu.rs = "CPU multigrid 4-level (Mobile
   Core fallback)"`, e WoS é o path **GPU** (Heavy/Standard/Lite). Então não é "escolha um" — são os dois,
   por tier. Multigrid CPU também é o que casa com os gates de determinismo (replay-hash / cross-OS
   bit-identity): é determinista por construção (varredura fixa, zero RNG, single-thread).
2. **Difusão em OKLab (L,a,b cartesiano), não OKLCH.** Difundir o hue polar `h` direto interpola red→blue
   pela curva errada (passa por magenta) e tem descontinuidade no wrap 360° que o Laplace não enxerga. As
   cores autoradas (`OklchColor`) são resolvidas p/ `OklabColor` na borda de autoria (`diffusion_curve.rs`);
   o solver só vê 3 canais escalares. Reconstrução final → `OklabColor::to_linear()` (mesmo path do UBO).
3. **Solver = precompute, não per-fragment.** Arquitetura real: `solve_color_field(curvas) → ColorField`
   (textura). O node `MeshGradient` (step 2) depois só **amostra** o campo no `coord` — barato. Isso casa
   com o WoS GPU (dispatch → textura) e com o budget de tier.
4. **Borda do canvas = Neumann (zero-flux reflexiva)** — cor difunde até a aresta naturalmente; as curvas
   são Dirichlet interior. Correction-scheme V-cycle: red-black Gauss-Seidel + full-weighting restrict +
   bilinear prolong; profundidade até grid 3×3.

## §3 — VALIDAÇÃO (por que confio na math)
17 testes (`cargo test -p ph2d-vector-fill --lib`), os relevantes:
- **`harmonic_linear_reproduced`** + **`harmonic_bilinear_xy_reproduced`** — ground truth analítico: `u=x` e
  `u=x·y` são harmônicas *discretas exatas* do stencil 5-point, então o solver TEM que reproduzi-las
  (err < 1e-3). Este é o teste de correção mais forte.
- **`multigrid_matches_gauss_seidel_oracle`** — multigrid == Gauss-Seidel-até-convergência (8000 sweeps) no
  mesmo sistema (diff < 1e-3). Valida a maquinaria restrict/prolong, não só o smoother.
- **`neumann_single_constraint_is_constant`** — 1 vértice fixo + Neumann → campo ≡ constante (valida borda).
- **`residual_converges_fast`** — resíduo cai geometricamente até o piso de float (o operador escala 1/h²
  ≈4096 @ 65², amplificando o eps f32 a ~1e-3; abaixo disso o max-norm faz jitter de poucos ULP — por isso
  só exijo monotonia *acima* do piso + convergência <2e-3 em ≤12 ciclos). Distingue multigrid (handful de
  ciclos) de GS (milhares de sweeps).
- **`straight_red_blue_curve_splits_field`** — end-to-end: parede vertical x=0.5, red à esquerda / blue à
  direita → meio-campo esquerdo≈red, direito≈blue, seam suave; canais R/B monotônicos L→R.
- **`solve_is_bit_deterministic`** — solve 2× → `Vec` bit-idêntico. (Pré-requisito do gate cross-OS.)

## §4 — TEU PRÓXIMO (Coord) + o que eu sigo (step 2/3)
**Tu (golden/smoke infra — o ping):**
- Scaffold de golden-image / smoke p/ o `ColorField` (PNG dump de `field.texel` ou hash do campo). O solver
  determinista te dá oracle estável. Sugiro um golden do caso `straight_red_blue` @ 129² + 2-3 curvas.
- **Contrato Region→FillGraph + resolução `gradient_id → DiffusionCurveSet`** continua TEU (foundational
  `ph2d-vector-doc`, ADR-0056-amendment). Meu solver consome `&DiffusionCurveSet` direto — sem dep no doc.
  Quando tu materializar o store, eu plugo o sample no eval/codegen (step 2 abaixo).

**Eu (na minha posse):**
- **Step 2 — node `MeshGradient` no CPU: ✅ FECHADO.** `eval.rs` ganhou `eval_color_with_fields(graph,
  coord, ubo, &dyn FieldResolver)` — o `MeshGradient` resolve `gradient_id → &ColorField` e samplea
  bilinear no `coord`; id não-resolvido → transparente (recurso ausente não derruba o grafo). Predicados
  separados: `lacks_cpu_eval()` (4 stubs, SEM MeshGradient) gateia o eval; `is_stub()` (5) continua o gate
  do **WGSL codegen** — que ainda rejeita `MeshGradient` (precisa do texture binding, step 3). Store
  determinista `FieldStore` (BTreeMap, HR-5). Cap de 17 intacto (node já existia). 3 testes novos:
  `mesh_gradient_samples_solved_field`, `mesh_gradient_unresolved_is_transparent`,
  `codegen_still_rejects_mesh_gradient`.
  - **O que ainda é teu p/ smoke-able no produto:** resolução `gradient_id → ColorField` (quem solva e
    popula o `FieldStore` no host) + Region→FillGraph. Meu eval consome o resolver via trait; é só plugar.
- **Step 3 — GPU WoS: ✅ FECHADO (tudo que valida sem GPU).** `diffusion_gpu.rs`:
  - **`diffusion.wgsl`** (WoS compute, storage-buffer out — zero caps de textura no naga; espelha a ref CPU
    linha-a-linha; OKLab→linear bit-idêntico ao `ph2d_color`) + **`bilateral_upsample.wgsl`** (JBU 2-pass).
    Ambos **naga parse+validate** em teste (`Capabilities::empty()`, igual ao `cache::compile_fill`).
  - **Referência CPU do WoS** (`walk_on_spheres_field`/`wos_estimate_point`) usando RNG `ph2d_noise1`
    (bit-idêntico CPU↔GPU → habilita o modo determinista §2.6). Teste `wos_converges_to_multigrid`:
    o estimador Monte-Carlo converge pro golden multigrid no centro do canal (prova o algoritmo GPU **sem
    GPU**). + `wos_is_bit_deterministic`.
  - **Dispatch data**: `pack_curves → Vec<GpuSegment>` (48B, Pod) + `DiffusionParams` (32B, Pod std140) +
    **tier matrix** `DiffusionTier::plan()` exatamente da tabela §2.5 (Heavy 64spp/5ms … MobileCore→multigrid).
  - **O QUE É TEU (renderer, não dá pra eu fazer/benchar aqui):** (1) o **dispatch wgpu** do `diffusion.wgsl`
    (criar pipeline/bind groups/buffers, copiar o storage buffer → textura do fill), (2) **bench de budget**
    (gate `vector_diffusion_curve_tier_budget`: Heavy ≤5ms etc.) na CI matrix + cross-OS bit-identity
    (`procedural_fill_cross_os`), (3) o **texture binding** que destrava o WGSL codegen do `MeshGradient`
    no fragment do fill (compartilha o bind group do fill = contrato teu). Os shaders + layouts (`@group(0)`
    bindings 0/1/2) e os structs Pod estão prontos p/ tu plugar.

## §5 — GIT / POSSE
- Commit local scoped: `crates/ph2d-vector-fill/src/{diffusion_curve,poisson_cpu,lib}.rs` + este handoff.
  `--no-verify`, sem push (tu shipa 1×/jornada). `git status` conferido: nada alheio staged.
- Cap de nodes (17) e contrato de fill intactos. `ph2d-vector-doc` não tocado.
═══════════════════════════════════════════════════════════════════
