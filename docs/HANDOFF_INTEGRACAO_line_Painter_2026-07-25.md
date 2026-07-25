# Handoff de INTEGRAÇÃO — `line/Painter`, a jornada de 2026-07-23..25

**Para:** o agente integrador (por ordem EXPLÍCITA do Enio, dada em 2026-07-25).
**De:** a `line/Painter`. **Estado:** ✅ fechada, **todos os smokes aprovados pelo Enio**.

> ## O resumo
>
> **~64 commits, seis waves, um só assunto de risco.** Cinco delas são perf/GPU e já têm handoff próprio
> (§3); a sexta é a que o Enio smokou em 25/07 e é a única que **muda comportamento de produto**: a tinta
> atravessando uma máscara emplumada saía **craquelada**, e a cura passou por duas rodadas —
> (a) o `keep` deixou de ser composto **por batch** (a força da proteção era um fato sobre a taxa de
> polling do mouse) e (b) deixou de ser aplicado **por traço** (a proteção **erodia**: oito passadas e a
> máscara não protegia mais nada). Hoje ele vale **UMA vez por texel**, sobre a tinta acumulada livremente,
> e a época dura o que a **proteção** durar.
>
> **`main` NÃO andou desde a base** ⇒ a integração é **fast-forward**, sem rebase e sem conflito.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| base (merge-base com `main`) | `df91ef6ec` |
| commits | **~64** — a fonte é `git rev-list --count main..HEAD`; ⚠️ **este handoff é um deles**, então um número cravado aqui envelhece no instante em que é commitado |
| `main` à frente da base | **0 commits** ⇒ `git merge --ff-only line/Painter` |
| superfície | **80 arquivos**, +10 823 / −777 |
| HEAD | o tip da branch — um sha literal aqui se auto-invalidaria |

⚠️ **O histórico NÃO é um caminho reto, e isso é deliberado:** três leis foram construídas e **revertidas**
dentro da linha (§13.6 envelope · §13.7 época · §13.9 lei do canal). Um squash é defensável (o estado final
é o que importa); **manter o histórico também é** — ele é a prova de que cada uma foi tentada, que é o
argumento que impede a quarta tentativa de repetir a primeira.

## 2. O que muda de COMPORTAMENTO (a lista inteira)

| # | mudança | onde |
|---|---|---|
| 1 | **A proteção vale UMA vez por texel** — deixa de seguir a taxa de polling do mouse | `paint/mask.rs`, `paint/stamp_route.rs` |
| 2 | **A proteção nunca ERODE** — repetir converge no `keep`, não passa dele | idem |
| 3 | **Smear/Blur/Clone sobre zona protegida leem a tinta IRRESTRITA** (semântica de layer mask) em vez do display | idem — consequência da (1), **nomeada e smokada** |
| 4 | O compositor GPU aceita **máscara e clipping** como ops (documento comum sai da CPU) | `ph2d-render/layer_compositor` |
| 5 | O orçamento de camadas vem do **DISPOSITIVO** (8 → 16 a 4K) | idem |
| 6 | O compositor GPU re-envia **só a região suja** da camada | idem |
| 7 | Um traço de máscara toma a **via parcial** com upload CHEIO | `paint/stamp_route.rs` + bridge |
| 8 | A pintura **para de copiar o canvas inteiro** por movimento | `tool/mod.rs`, `layers/preview.rs` |
| 9 | A transferência sRGB vira **tabela** (os 2 knobs EXPERIMENTAL do wet: 20-34× → 2-3×) | `ph2d-wet-paint::colorops` |

**Nada mais.** As três leis revertidas não deixam resíduo de comportamento; o que sobreviveu delas é
**higiene** (uma cópia do predicado de cobertura, uma da aritmética do cap) e **documentação do que já foi
tentado**.

## 3. As seis waves, e onde está o detalhe de cada uma

| wave | commits | handoff |
|---|---|---|
| doc 24 — a transferência sRGB tabelada | 1..8 | [`..._wet_transfer_2026-07-23.md`](HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md) |
| GPU Ondas 1+2 — máscara/clipping como ops, orçamento do dispositivo | 9..17 | [`..._gpu_ondas_1_2_2026-07-23.md`](HANDOFF_INTEGRACAO_line_Painter_gpu_ondas_1_2_2026-07-23.md) |
| Onda 5a — a pintura para de copiar o canvas | 18..19 | [`..._gpu_onda5a_paint_nocopy_2026-07-24.md`](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5a_paint_nocopy_2026-07-24.md) |
| Onda 5b — upload parcial da região suja | 20..21 | [`..._gpu_onda5b_partial_layer_upload_2026-07-24.md`](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5b_partial_layer_upload_2026-07-24.md) |
| Onda 5c — a máscara na via parcial + `PH2D_PAINT_PERF` | 22..29 | [`..._gpu_onda5c_mask_partial_lane_2026-07-24.md`](HANDOFF_INTEGRACAO_line_Painter_gpu_onda5c_mask_partial_lane_2026-07-24.md) |
| **A máscara e o gate de proteção** (§13.6 → §13.13) | 30..63 | **esta seção §4** + [doc 25](Painter/25_avaliacao_gpu.md) §13.6-§13.13 + [BUGS #17](Painter/BUGS_painter.md) |

## 4. A wave de 25/07 — o gate de proteção

### 4.1 O que estava errado (medido, não inferido)

| | 4 eventos/traço | 60 eventos/traço |
|---|---|---|
| tinta que sobrevivia em `keep = 0,5` | 0,886 | **0,992** |
| serra do contorno da TINTA | 0,061 px | **0,164 px** |
| posição do contorno | — | andava **4 px** |

`restore_protected_region` puxava os texels protegidos de volta **uma vez por BATCH**, contra o snapshot
daquele batch ⇒ `(1−keep)^N` com **`N` = a taxa de polling**. E o que sobrava visível era recortado pelos
**RETÂNGULOS dos batches** — o degrau axis-aligned da foto do Enio.

Curado isso, o smoke devolveu *"sanou quase 85%"*. Os 15% tinham causa exata: cada TRAÇO era escalado por
`keep`, então `N` traços deixavam passar `1 − (1−keep)^N` ⇒ o contorno **dentava** (previsão aritmética
**1,64 px** contra **1,68 medido**) e, pior, **a proteção erodia**: em `keep = 0,522`, `N=4` deixava passar
**0,949** e `N=8` deixava **1,000**.

### 4.2 O desenho

`GateSession { base, free, preview_patch, layer, scratch_gen, witness }` no `PainterTool`, ao lado do
`canvas_rgba` e dos 3 planos de relevo (é plano canvas-shaped, e `paint.rs` está no teto de 700 LOC).
`base` é `Arc` clone (refcount); `free` é *o que a pintura irrestrita teria produzido*, **trocado para
dentro do `canvas_rgba`** durante o stamp (o mesmo truque do scratch da máscara) ⇒ **toda rota** pinta o que
pintaria sem gate nenhum. O que se vê é `free·keep + base·(1−keep)`, com `keep = proteção × seleção`.

⚠️ **É a época do §13.7 (revertida), e a diferença é a MÁQUINA que a fecha:** os **22 escritores
estrangeiros de canvas** que ela commitava à mão viraram **UMA pergunta** no topo de todo batch — *algo
mudou debaixo de mim?* — respondida por **três testemunhas** (a camada · a geração do scratch · o
`pixel_clock`). Enumeração apodrece; testemunha não.

### 4.3 Resultado

| | antes | depois |
|---|---|---|
| tinta em `keep≈0,5` a 4 vs 60 eventos | 0,886 vs 0,992 | **0,800 nas duas** |
| erosão em `keep=0,522` após 8 traços | **1,000** | **0,522** |
| pente da fronteira (fresca / 15 passadas) | 1,68 / 0,60 px | **0,05 / 0,12 px** |
| rampa da TINTA ÷ rampa da MÁSCARA | 0,876 | **1,000 — idênticas** |
| serra do contorno | 0,164 px | **0,042 px** |

A rampa da tinta virar **exatamente** a da máscara é a assinatura da lei (com a tinta livre saturada, o
display é função pura do `keep`). Confirmado por render-and-look.

## 5. Superfícies de risco

| pergunta | resposta |
|---|---|
| **contratos congelados** (§6 do CLAUDE.md) | **NENHUM tocado.** `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` — gate `architecture_tool_contract_surface` **verde (4 testes)**. `NodeOp`/`OpResolver`/`NodeManifest` e a superfície do `ph2d-vector-doc` não têm um único diff. |
| **schemas** | **NENHUM bumpou.** `PROJECT_SCHEMA` **29** (conferido no fonte, não por auto-relato). A `GateSession` é transiente por construção — há gate provando que ela morre no undo. |
| **ids / tokens / i18n** | **zero** ids novos, zero consts de UI, zero chaves de i18n, **zero entradas em lista ordenada**. |
| **foundational** | `ph2d-render` (o compositor de camadas) e `ph2d-painter-effects` (gpu_codes) — waves 4-6, com handoff próprio. Ambos **aditivos**. |
| **deps** | **nenhuma nova** (`Cargo.toml`/`Cargo.lock` intocados) ⇒ machete/deny/audit sem superfície nova. |
| **ADR** | **NENHUM novo** — logo **nenhuma disputa de número** (a armadilha do 0130→0131 da física e do 0134 do gpu-nodes não se aplica aqui). O único tocado é o [`0109-rayon-exception-watercolor-composite`](architecture/decisions/0109-rayon-exception-watercolor-composite.md), que **já existe no `main`** (`3a9392fb1`) e recebe uma **emenda de 34 linhas** (o composite row-parallel do wet). Conferido no fonte, não por auto-relato. |

**Símbolos que podem colidir:**

| símbolo | forma | crate |
|---|---|---|
| `GateSession` | `pub(crate) struct` | `ph2d-tool-painter::tool` |
| `PainterTool::{stamp_dabs_gated, end_gate_session, bump_mask_scratch_gen}` | `pub(super)`/`pub(crate)` | `tool/paint/mask.rs` |
| `region::region_pixels` | `pub(super) fn` | `tool/paint/region.rs` |
| `stroke_cover::cover_add` | `pub(crate) fn` | `ph2d-painter-brush` |
| `PainterTool::stroke_cover_wanted` | `pub(super) fn` | `tool/paint/stamp_route.rs` |
| `mask_scratch_gen` | campo de `PainterTool` | `ph2d-tool-painter` |
| `PH2D_MASK_SMOKE` | env var nova | `shells/desktop/src/mask_smoke.rs` |

⚠️ **Aposentados (código MORTO removido, e os 5 doc-links reapontados):** `snapshot_region`,
`restore_protected_region`, `restore_deselected_region`.

## 6. Gate de fechamento

| | |
|---|---|
| `scripts/nextest-impacted.sh` | **4074 testes, 4074 passaram** (debug e release) |
| `cargo test -p ph2d-tool-painter` | 834 passaram |
| `cargo fmt --check` (pin **1.95**) | limpo |
| `cargo clippy --all-targets` | **0 warnings** nas crates tocadas |
| LOC caps | verdes nos **dois** gates (`architecture_workspace_file_loc_cap` **e** `shells/desktop/tests/file_loc_caps.rs`, que **não** roda com `cargo test -p`) |
| `arch_safe_clamp_only` | verde |
| `typos` | **zero hits novos** (os 7 do `BUGS_painter.md` são falso-positivos de português **pré-existentes**, todos fora da entrada nova) |

⚠️ **Flake conhecida e PRÉ-EXISTENTE — ela apareceu no fechamento desta linha, e a inocência é verificável:**
`ph2d-timeline::nesting_clock the_cost_of_depth_is_linear_not_explosive` é gate de RAZÃO sensível a carga.
Na varredura ela falhou uma vez e **passou isolada na sequência**; e esta linha toca **ZERO arquivos** em
`ph2d-timeline` (`git diff --name-only main...HEAD -- crates/ph2d-timeline` = vazio). **Re-rode sozinho
antes de suspeitar do merge.**

## 7. O smoke

```fish
cd <árvore> && env PH2D_MASK_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

**Aprovado pelo Enio em 2026-07-25**, nas duas rodadas. A cena não arma nada além do canvas (nem o modo,
nem o pincel) e **imprime o que montou**. O passo 4 é a estrela: pintar atravessando a zona protegida muitas
vezes, **lento e depois rápido** — a fronteira tem de ficar igual nas duas velocidades **e não avançar por
mais que se insista**.

## 8. Aberto (nomeado, não contrabandeado)

| item | estado |
|---|---|
| **o endurecimento da borda da MÁSCARA** (3,53 px numa passada → 1,38 em quinze) | 🔎 **ABERTO**, é o OUTRO eixo. As **duas** leis de acúmulo já foram tentadas e cada uma tem seu artefato (produto = endurece · envelope = contas), então a cura **não** é a lei da cobertura (doc 25 §13.10.4). ⚠️ A rampa da tinta agora rastreia a da máscara **exatamente**, então curá-lo cura os dois. |
| **custo do pen-down de um traço protegido** | medido: 7,5 ms @2048² (contra 3,3 sem proteção) e 24,3 @4096² (contra 11,7). O clone canvas-sized é amortizado pela PROTEÇÃO inteira, não por gesto; o **move** é plano na tela (1,14 / 1,12 ms) e **gateado por RAZÃO**. Receita da wave de perf (semeadura lazy por TILE + reuso da alocação) escrita na §13.12.5 — **não** feita dentro de uma wave de correção. |
| métodos de SHAPE em modo máscara não pintam nada | pré-existente (o roteador de shape intercepta o Down antes do `paint_begin`; scratch com 0 bytes) |
| Bug #11 (Per-Layer Color, linhas retangulares intermitentes) | dormente, pré-existente |

---

## 9. Para o integrador, em ordem

1. `git merge --ff-only line/Painter` (a base não andou — se andou desde este handoff, **rebase** e
   re-rode o gate da árvore combinada: os arch-gates de `shells/desktop/tests/` **só** correm na varredura
   impactada, e um fechamento por `cargo test -p` não os alcança).
2. `./scripts/ship.sh` e corrija todo `✗` antes de qualquer push.
