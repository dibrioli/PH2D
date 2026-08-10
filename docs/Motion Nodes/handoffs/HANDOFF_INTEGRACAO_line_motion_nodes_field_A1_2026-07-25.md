# HANDOFF DE INTEGRAÇÃO — `line/motion-nodes` (família `field.*` + A1 curva) — 2026-07-25

> Para o **agente integrador** (a linha NÃO integra a si mesma — §0.7). A linha está
> **FECHADA e VERDE**; este doc é o que ela entrega. Ordem do Enio: integrar ao main.
> Reaberta após a integração dos deformers (2026-07-23); tudo aqui é `main..line/motion-nodes`.

---

## 1. O que a linha entrega

Duas frentes, ambas smokadas e aprovadas pelo Enio:

**(A) A família `field.*` (D1 — o falloff que compõe)** — 5 nós-crate novos, todos
GPU-resident bit-exato (paridade CPU×GPU na RTX):
- `field.index_range` (banda ordinal por rank) · `field.box` (campo espacial + rotação) ·
  `field.combine` (o composer) · `field.radial_sweep` (radar angular, herda o gizmo) ·
  `field.remap` (o **remapper** do `falloff` — a peça-chave da fatoração D1).
- **Gizmo de canvas dos campos espaciais** (D9, `field_gizmo.rs` — espelho do FlipSelection) +
  fix do **drift crônico** (o chrome projetava full-window sob a cena na banda do split).

**(B) A1 — a CURVA como param (a linguagem dos params, W-A)** — em 3 fatias:
- **A1-core**: crate nova **`ph2d-curve`** (leaf, dep-free) + `ParamWidget::Curve` + o **5º
  contour (Curve)** do `field.remap` (forma na coluna de text param, `eval` na CPU).
- **A1-ui**: **editor de curva ARRASTÁVEL** no painel `motion-params` (gráfico com alças +
  `+`/`−` + interp Linear/Smooth/Hold) — reusou o primitivo pronto
  `InteractiveState::CurvePoint` (o editor de falloff do Painter), então foi **painel-only**.
- **A1-gpu**: **canal de LUT** — a curva agora cozinha no **DEVICE** (o demo `=22` saiu de
  HYBRID/CPU → **FULLY GPU, 5 stages**). Detalhe em §3.

Commits: `git log main..line/motion-nodes --oneline` (24 commits, de `d805cc2a2` doc 63 a
`59604c982` fechamento).

---

## 2. Estado de contrato / schema / ADR — NADA a decidir

- **Contrato congelado INTACTO:** `NodeOp=2` / `OpResolver=1` / `NodeManifest=8` — gate
  `architecture_contract_surface` VERDE (conferido, não auto-relato). O A1 usa **text params**
  (params vivem no `Graph`, não no `NodeManifest`) e o A1-gpu é **side-metadata no registry**,
  exatamente o padrão dos canais `grid`/`reduces` que já existiam. **Não exige ADR.**
- **Sem bump de `PROJECT_SCHEMA` nem `DOC_VERSION`** (a curva é text param, a LUT é derivada).
  ⚠️ Isto significa que a linha **não entra na disputa de número de schema** com outras linhas.
- **Sem ADR novo** (`docs/architecture/decisions/` intocado) ⇒ **sem colisão de número de ADR**.
- **`ph2d-node-registry`** (registry de TIPOS de nó) — **NÃO** é o `ph2d-ecs` (registry de
  COMPONENTES) que physics/vector bumpam. Sem relação, sem colisão.

---

## 3. Superfície FOUNDATIONAL tocada (append-only, projetada para isolamento — ADR-0107)

**`ph2d-nodegraph` (`src/gpu.rs`)** — o ponto de extensão do A1-gpu:
- `pub struct LutSpec { name, text_key, resolution, fill: fn(&str, &mut [f32]) }` — a `fill`
  é um **fn-pointer no crate do NÓ**, então o substrato fica agnóstico de curva (ADR-0126).
- `KernelResolver::luts(&self, ty) -> &'static [LutSpec]` com **default `&[]`** — irmão exato
  de `reduces`. **Zero churn nos 32 kernels existentes.**

**`ph2d-node-registry` (`src/lib.rs`)** — `register_luts` + campo `luts: BTreeMap<…>` +
resolver `fn luts`. Espelho de `register_reduces`.

**`ph2d-gpu-cook`** — o sequenciador aprendeu a LUT:
- ⚠️ **`codegen::kernel_module` mudou de assinatura: 6 → 7 args** (ganhou `luts: &[LutSpec]`
  após `reduces`). **Este é o único `pub fn` cross-cutting da linha.** Todos os chamadores já
  atualizados: `encode.rs` (produto), `tests/gpu_cpu_parity.rs`, `codegen_tests.rs`,
  `tests/generated_wgsl_validates.rs`, `shells/desktop/tests/motion_gpu_kernel_budgets.rs`.
- `encode.rs`: `encode_kernel_stage` ganhou o param `luts`, anexa os buffers ao bind group
  **após os reduces**; `build_luts` (lê o text param via `graph.node_text_param_overrides`,
  sobe a tabela `f32`). `lib.rs`: `lut_hold` (segura até o submit), construído **fora do loop
  de sweep** (a curva é invariante ao sweep).

**Crate nova `ph2d-curve`** — o workspace usa `members = ["crates/*", …]` (glob), então é
**auto-incluída** (root `Cargo.toml` intocado). `field.remap` a declara em `[dependencies]`;
`registry-init` ganhou os 5 nós de campo (dep + `register`).

### O risco de merge, nomeado
A **única** aresta que pode conflitar com outra linha é `kernel_module` / o sequenciador
`ph2d-gpu-cook` (se alguma linha paralela também o tocou). As **sentinelas do gate da árvore
combinada** que pegam isso: `generated_wgsl_validates` (valida a WGSL de TODO kernel via naga,
incluindo o accessor `rm_curve_sample`) e `motion_gpu_kernel_budgets` (orçamento de uniforme
com a LUT). Se as duas passam na árvore combinada, o sequenciador fundiu limpo.
⚠️ **Fan-out do registry-init** (memória `feedback_fanout_registry_init_friction`): os 5 nós
de campo têm os 2 testes manuais de registro; confira que sobreviveram ao rebase.

---

## 4. Gate de fechamento — VERDE (rodado no tip da linha, 2026-07-25)

| Gate | Comando | Resultado |
|---|---|---|
| Contrato congelado | `cargo test -p ph2d-nodegraph --test architecture_contract_surface` | ✅ 3/3 |
| WGSL valida (naga) | `cargo test -p ph2d-gpu-cook --test generated_wgsl_validates` | ✅ (todo kernel + a LUT) |
| Paridade CPU×GPU (unit) | `cargo test -p ph2d-node-field-remap` | ✅ 22/22 |
| LOC workspace | `cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap` | ✅ 2/2 |
| **LOC shell (HR-18)** | `cargo test -p ph2d-host-desktop --test file_loc_caps` | ✅ 2/2 (após split — ver ⚠️) |
| Clippy shell | `cargo clippy -p ph2d-host-desktop --all-targets` | ✅ 0 warnings |
| Clippy crates A1 | `cargo clippy -p ph2d-nodegraph -p ph2d-node-registry -p ph2d-gpu-cook -p ph2d-node-field-remap --all-targets` | ✅ 0 |
| Gates GPU do shell | `cargo test -p ph2d-host-desktop gpu_coverage_census` (e os `*_is_fully_gpu`) | ✅ (=22 imprime **FULLY GPU**) |

⚠️ **O `file_loc_caps` do shell nasceu RED no fechamento** (a família de campos empurrou
`motion_state_gpu_demos.rs` 511→918 e `_tests.rs` 519→698). **Fix: SPLIT por família** (o
padrão já usado nos deformers ao lado): nasceram `motion_state_gpu_field_demos.rs` (417) e
`motion_state_gpu_field_tests.rs` (189); os originais voltaram a 512/520. ⚠️ **Nota para o ship
do integrador:** o `file_loc_caps` mora em `shells/desktop/tests/` e só roda na varredura
impactada — o `ship.sh` o cobre, mas um `cargo test -p` por crate NÃO (a mesma família do miss
que a `line/physics`/`line/Vector` documentaram). **Rode o `ship.sh` inteiro.**

### Paridade RTX (`#[ignore]` — precisa de adapter)
```
cargo test -p ph2d-gpu-cook --test gpu_cpu_parity -- --ignored field_remap_curve_contour_matches_the_cpu_on_the_device
```
Medido: **`max |Δtint| = 2.38e-7` em 25.600 instâncias** (o tent linear = lerp exato). O ε do
gate é 6e-3 (o pior-caso do canto do pico, ~4e-3). Todos os `field_*_kernel_matches_the_cpu…`
irmãos também são `#[ignore]` + RTX.

---

## 5. Smokes (todos `--release`; todos APROVADOS pelo Enio nesta jornada)

```
cd <worktree ou main pós-integração> && env PH2D_GPU_COOK_DEMO=<N> cargo run -p ph2d-host-desktop --release
```
- **`=17`** field.index_range (banda ordinal) · **`=18`** field.box · **`=19`** field.combine ·
  **`=20`** field.radial_sweep (estrela de 6 pontas — smoke aprovado) · **`=21`** field.remap
  (bandas, contour Quantize — aprovado).
- **`=22`** o **Curve contour**: selecione o nó `field.remap` no grafo → o painel à direita mostra
  o **editor de curva arrastável** (A1-ui, aprovado); e o anel/campo agora renderiza **100% no
  device** (A1-gpu, aprovado — visualmente idêntico à CPU, `=22` = FULLY GPU no censo).

Gizmo dos campos: entre num demo espacial (=18/=20), selecione o nó → a caixa/leque do gizmo
aparece sobre a cena (o fix do drift garante que ele fica alinhado na banda do split).

---

## 6. Aberto (NÃO para esta integração — trabalho de sessão futura da linha)

- **Cauda W-B**: spline/shape/noise **field sources** (nós novos) + **STRENGTH audit** por-nó +
  verificação dos **neutros D12**. É o próximo passo do plano (doc 63 §W-B).
- **Editor de gradiente do `motion.color-ramp`** — ENFILEIRADO no plano (doc 63 §W-I, **I9**):
  o único análogo real do editor de curva (gradiente por 6 sliders crus), reusa o mesmo
  `CurvePoint` + o swatch OKLCH. Ordem do Enio: fila, não agora.
- **Drift residual** (pan-rate + sprite-picking durante o Motion) — avaliar com o Enio.
- **A1-gpu, follow-up medido (não bloqueante):** a LUT é reconstruída por frame (parse + 256
  `eval`) para todo `field.remap`, mesmo em contour ≠ Curve. Custo sub-µs; um cache por-string
  é otimização futura, não necessária.

---

## 7. Resumo em uma linha para o integrador

Rebase `line/motion-nodes` no main → `scripts/foundational-integrate.sh` (gate da árvore
combinada; as sentinelas são `generated_wgsl_validates` + `motion_gpu_kernel_budgets`) →
`./scripts/ship.sh` inteiro (o `file_loc_caps` do shell só roda aí) → push → babysit CI.
**Sem ADR, sem bump de schema, contrato intacto.** A única aresta cross-cutting é a assinatura
`kernel_module` 6→7 em `ph2d-gpu-cook`.
