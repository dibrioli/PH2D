# Handoff de integração — `line/Painter` · transferência sRGB tabelada

**Para:** o agente integrador (DIRETRIZ §1.5.9). **Data:** 2026-07-23.

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | `e71f9e4c9` |
| base do fork | `df91ef6ec` (= `main` no momento da abertura; rebase feito na abertura) |
| commits | **6** |

Commits, em ordem (têm dependência entre si — integre a sequência inteira):

1. `117023207` — `perf(wet)`: a transferência sRGB vira tabela.
2. `ffcebccb1` — `test(wet)`: gates de precisão e de razão.
3. `bfc2cffa9` — `refactor(wet)`: uma porta só no forward do K–M, um `R_FLOOR` só.
4. `c086bd8f1` — `docs(wet)`: doc 24 + este handoff + a entrada na §5.
5. `da124a5e9` — `docs(memory)`: a lição do ponto fixo.
6. `e71f9e4c9` — `perf(wet)`: o composite vira **row-parallel** (ADR-0109 emenda 2).

⚠️ **O commit 6 é a 2ª rodada**, depois de o smoke reprovar (*"ainda muito
lento, mais lento que HTML JS"*). Ele é o que traz o frame do produto de 20 para
31 fps a 1024², e é onde está a emenda do ADR.

## 2. Foundational / compartilhado tocado

**Nenhum foundational.** O diff vive em `crates/ph2d-wet-paint/` +
`crates/ph2d-tool-painter/src/tool/paint/wetpaint*` + a emenda no
`docs/architecture/decisions/0109-*.md`. Sem `editor-core`, sem `ph2d-core`,
sem `shells/*`, sem `tokens`, sem i18n, sem registry gerado.

⚠️ **`ph2d-tool-painter` ganhou o fan-out por linhas do composite** (rayon, que
essa crate **já** tinha por ADR-0109) — registrado como **emenda 2** naquele ADR,
não contrabandeado. O engine continua sem thread-pool e sem dep nova.

Arquivos: `ph2d-wet-paint/src/colorops.rs` · `src/colorops/transfer.rs`
(**novo**) · `src/render.rs` · 4 testes novos · `ph2d-tool-painter/src/tool/
paint/wetpaint.rs` (imports) + `wetpaint/composite.rs` (o fan-out) ·
`docs/architecture/decisions/0109-*.md` (emenda 2, **append**).

⚠️ O arquivo novo é um **módulo irmão** (`colorops/transfer.rs`), o padrão de
isolamento da DIRETRIZ §1.5.2.1 — não engordou nenhum arquivo compartilhado.

## 3. Símbolos que podem COLIDIR com outra linha

Nada numerado, nada em lista ordenada, nada global. Sem id de widget, sem
`NodeId`, sem token, sem chave i18n, sem variant de enum compartilhado.

Superfície pública da crate, para grep do integrador:

| símbolo | |
|---|---|
| `colorops::transfer` (módulo) | **novo** |
| `transfer::{srgb_to_linear, linear_to_srgb}` | **movidos** de `colorops` (re-exportados pelo caminho antigo ⇒ nenhum chamador quebra) |
| `transfer::{srgb255_to_linear, srgb255_of_linear, ks_of_srgb255}` | **novos** (portas de domínio 0..255) |
| `transfer::{srgb_to_linear_exact, linear_to_srgb_exact, ks_of_srgb255_exact}` | **novos** (referência `libm`, usada pelos gates) |
| `colorops::km_mix_channel_linear` | **REMOVIDO** — ficou sem chamador nenhum no workspace (era `pub`, logo sem warning) |
| `colorops::ks_of_reflectance` + `colorops::R_FLOOR` | **REMOVIDOS** (privados; única usuária era a função acima) |

Consumidores da crate: `ph2d-tool-painter` e `ph2d-panel-wet-tuning` — **nenhum
dos dois usa `colorops::` diretamente** (verificado por grep). `cargo check
--workspace` verde.

## 4. Contratos congelados encostados

**NENHUM.** `Tool` / `RasterEditTool` / `CanvasPaintTool` / `PanelEvent` intactos
— a linha toca `ph2d-tool-painter` mas **só o corpo** de `wetpaint/composite.rs`
(nenhum método de trait, nenhum variant de `PanelEvent`). Nós/vector idem.

**Nenhum schema bumpou:** `PROJECT_SCHEMA` fica **29**, `DOC_VERSION` intacto,
`VEC_SCENE` intacto. `BrushSpec` não é serde e não mudou.

## 5. O que só o `ship.sh` pega

- `cargo fmt` e `clippy --all-targets` rodados nas DUAS crates e **limpos**;
  `typos` limpo nos arquivos novos.
- **Nenhuma dep nova** — nenhum `Cargo.toml` tocado, `Cargo.lock` intocado (o
  rayon do fan-out já era dependência do `ph2d-tool-painter` por ADR-0109).
  Logo `machete`/`deny`/`audit` não têm superfície nova.
- Gates de workspace rodados aqui (não caem no `cargo test -p`):
  `architecture_workspace_file_loc_cap` ✓ · `arch_safe_clamp_only` ✓ ·
  `ph2d-editor-core` inteiro ✓ · `shells/desktop::file_loc_caps` ✓.
- `scripts/nextest-impacted.sh`: **3476 passaram, 0 falharam**.
- LOC: maior arquivo tocado é `render.rs` com 447/700.

## 6. O que smoke-testar

**`PH2D_WETPAINT_SMOKE=1 cargo run -p ph2d-host-desktop --release`**
→ dropdown **Paint Mode → Wet Paint** → seção **Wet Paint** → **Tuning** →
grupo **EXPERIMENTAL**.

1. **Pigment mixing (K–M)** — ligue e pinte. Antes ele derrubava o app para ~8 fps
   numa lavagem grande; agora o custo num traço real é **0,06 ms/tick**. O que
   olhar: a mistura continua **subtrativa** (amarelo + azul lê VERDE, não cinza)
   e a fluidez não muda ao ligar.
2. **Glaze layering** — ligue sobre tinta molhada por cima de tinta seca (dê
   **Fast dry** e pinte por cima). O empilhamento por refletância deve continuar
   com a mesma aparência de antes; o que mudou é o custo (7,3×).
3. ⚠️ **O par ligado/desligado deve ficar igual ao que era** — esta wave é
   performance, não aparência. Se a cor mudar visivelmente, é regressão.
   O composite paralelo é **byte-idêntico** (a medição compara os buffers).
4. **Os dois DESLIGADOS** (o default) estão pinados byte a byte pelo fingerprint,
   então não precisam de smoke — mas se algo parecer diferente ali, é grave.

**Não smokado por mim:** só rodei os gates headless; nenhuma janela foi aberta.

## 7. Ordem / dependências

Os 6 commits são sequenciais. ⚠️ O ponto de conflito plausível com outra linha
é **`docs/architecture/decisions/0109-*.md`** (append no fim — resolução é a
união) e `wetpaint/composite.rs`. Em `ph2d-wet-paint` o conflito seria em
`colorops.rs`/`render.rs`, e a resolução é **semântica** (a porta única do §3),
não textual.

## 8. Aberto (nomeado, não escondido)

- **O SOLVER é agora 89% do frame** com Pigment mixing ligado (28,8 de 32,2 ms a
  1024²) e é **serial POR SEMÂNTICA** (ADR-0134) — não há row-parallelism a
  aplicar nele. É o teto atual, e o que sobra ali é piso de algoritmo (15
  lookups + 3 `sqrt` por célula no `advect`, 18 + 6 no `drying_pass`).
- **O flood com K–M ligado fica em 18,9 ms contra o kill de 12 ms** do caminho
  default. Era 122,8 ms. A cena é o *upper bound* declarado pelo ADR-0134, não o
  caso típico, mas o número **está acima da barra** e fica nomeado.
- **O canvas do smoke (1024²) tem 2,6× as células do reference JS (900×450).**
  Boa parte do *"mais lento que o HTML/JS"* é isso — na medida do reference nós
  somos ~3× mais rápidos que a barra dele. Se o alvo é paridade percebida, o
  tamanho do canvas é uma decisão de PRODUTO, não de perf.
- A cadeia settle→rewet do `drying_pass` reconverte `susp_rgb` (~33% do custo de
  K–M naquela passada); fechar exige K/S através de `lift_settled`, que é porta
  **compartilhada** com as tools do doc 23 ⇒ seria segunda porta com numérica
  diferente. **Não feito, de propósito.**
- `render_region` (onde entrou a entrada preguiçosa) **não tem chamador de
  produto** — é o *reference look* da SPEC §13. Anotado no próprio arquivo.

Detalhe completo, com as tabelas de medição e a **alternativa medida e
rejeitada**: [`docs/Painter/24_transferencia_srgb_tabelada.md`](Painter/24_transferencia_srgb_tabelada.md).
