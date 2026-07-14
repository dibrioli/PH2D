# HANDOFF de INTEGRAÇÃO — `line/Painter` (2026-07-13)

> ⛔ **HISTÓRICO — a linha foi INTEGRADA na `main` em 2026-07-13.**
> O documento vivo é [`HANDOFF_line_Painter_continuacao_2026-07-14.md`](HANDOFF_line_Painter_continuacao_2026-07-14.md).
> **O smoke do Enio segue PENDENTE** — é o item nº 1, antes de W4.

> **Para o AGENTE INTEGRADOR** (DIRETRIZ §1.5.9). Este é o documento operacional da linha inteira.
> Detalhe técnico do sculpt: [`HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md`](HANDOFF_line_Painter_sculpt_integracao_2026-07-13.md).
> Plano: [`docs/Painter/18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md).

---

## 0. ⛔ LEIA ISTO PRIMEIRO — a linha NÃO foi validada no produto

**O smoke do Enio está PENDENTE. Ele o fará amanhã (14/07).**

A linha está **verde em todo gate** e **não foi vista rodando**. Nesta linha específica essa distinção já
custou caro duas vezes:

* O rig de luzes shipou **morto sob o mouse** com todos os testes verdes (pintava, registrava hit-rect, e
  o `populate` nunca deu `InteractiveState`).
* O card do Sculpt shipou com um **bug de DESIGN** pinado por um gate verde, bem escrito e
  mutation-proven — o Enio derrubou em uma frase no 1º smoke.

> **NÃO INTEGRE ANTES DO SMOKE**, a menos que o Enio mande explicitamente. Se ele mandar integrar antes,
> integre — mas registre no commit de merge que a linha entrou **sem smoke**.

---

## 1. A linha, em números

| | |
|---|---|
| **Branch** | `line/Painter` @ **`7ce0496d`** |
| **Base (merge-base)** | `4cd8ef13` |
| **Commits acima da main** | **12** |
| **Diff** | 71 arquivos, **+9185 / −478** |
| **`main` andou?** | **Sim, 3 commits** — todos só em `project-memory/*.md` (memórias que EU escrevi durante a linha) |
| **Merge a seco (`git merge-tree main HEAD`)** | ✅ **LIMPO — zero conflitos**, inclusive o `MEMORY.md` |
| **Contratos congelados (CLAUDE.md §6)** | **NENHUM tocado** |

⚠️ **Merge limpo no texto pode estar quebrado por dentro** ([[feedback_clean_text_merge_can_be_semantically_broken]]).
O gate da árvore COMBINADA (`scripts/foundational-integrate.sh`) continua obrigatório — o `merge-tree`
limpo só diz que não há conflito textual.

---

## 2. O que a linha entrega (duas metades — não é só o sculpt)

### Metade A — o MATERIAL da tinta (5 commits, `aac2b6a3`..`1d63d89d`)

* **Material per-pixel**: Roughness / Metallic / Wax por pixel (`mats`, 7 B/px), + LUT 2D de especular.
  **O `Shine` deixou de ser global** e virou propriedade da TINTA.
* **Cor do Wax**: a luz que atravessa a tinta volta com a cor dela (um FILTRO, não uma fonte).
* **Fix: a UI do rig de luzes estava MORTA sob o mouse** — pintava, registrava hit-rect, e não estava no
  `WidgetStore`. Daí nasceu o harness de clique do `ph2d-ui-testkit` (§4).
* **Fix: o undo esquecia o `mats`** — e o buraco **se escondia na tela vazia** (cobertura zero ⇒ a luz pesa
  o material obsoleto por zero). Só fala em tinta-sobre-tinta.
* Toggle **"Adjust Last Stroke"** no card Body.

### Metade B — o SCULPT do relevo (W1+W2+W3, 5 commits, `4433fd4a`..`7ce0496d`)

**Oito verbos, uma expressão** (`h = pre + k·Δ`): Smooth · Sharpen · Flatten · Scrape · Fill · Chisel ·
Layer · Inflate. Novo `PaintMode::Sculpt` + chip **SCULP** no rail + card no painel `painter-layers`.

Detalhe no handoff do sculpt. O que o INTEGRADOR precisa saber:
* O sculpt **escreve `h` e só `h`** — não toca RGBA, `covers` nem `mats`. Não pode colidir com a metade A.
* Ele pendura no choke point que a COR já usa (`stamp_dabs_inner`) e **retorna antes das rotas de cor**.
* **Clay / Clay Strips / Draw Sharp NÃO existem como chips**, de propósito (são achados, não gaps — §13.2
  do handoff do sculpt). Se um revisor perguntar "cadê o Clay", a resposta é: *é o Flatten com Offset
  positivo, e os dois knobs já estão na tela.*

---

## 3. ⚠️ Os NÚMEROS QUE SOMAM entre linhas (o risco de merge nº 1)

O `merge-tree` deu limpo **hoje**. Se outra linha tocou o rail ou o `PaintMode`, o valor certo **não existe
em nenhum dos dois lados do conflito** — **CONTE, não escolha** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).

| Símbolo | Onde | Antes → Depois |
|---|---|---|
| `PAINTER_RAIL_TOOL_IDS` | `ph2d-editor-core/src/ids/chrome/rail_painter.rs` | `[NodeId; 11]` → **`[NodeId; 12]`** |
| `PAINTER_TOOLS` | `ph2d-editor-core/src/screens/hero/left_rail.rs` | `[…; 10]` → **`[…; 11]`** |
| `PAINT_MODE_COUNT` | `ph2d-tool-painter/src/tool/paint/paint_mode.rs` | `9` → **`10`** |
| **`PROJECT_SCHEMA`** | `shells/desktop/src/project.rs` | **`7` → `9`** ⚠️ **formato de SAVE** |

**O `PROJECT_SCHEMA` é o mais perigoso.** Dois bumps: v8 (`mats` entrou no `PaintedDocument`) e v9 (o
MESMO `mats` mudou de FORMA — 4 B → 7 B, a cor do Wax). É **postcard posicional**: se outra linha também
bumpou, o número final é `max(nossos, deles) + o que faltar`, e os dois layouts têm que coexistir ou o
save antigo sai com o material vindo dos bytes da cobertura.

Estes NÃO somam (são internos ao sculpt, e nenhuma outra linha os conhece):
`SCULPT_MODE_COUNT = 8` · `PAINTER_SCULPT_MODE_IDS: [NodeId; 8]` · `PAINTER_SCULPT_FIELDS: [NodeId; 4]`.
Eles são **append-only por contrato**: o discriminante É o índice do segmented, e reordenar re-liga a
memória muscular de todo artista em silêncio.

---

## 4. Superfície FOUNDATIONAL tocada (ADR-0107)

| Crate | O quê | Risco de colisão |
|---|---|---|
| **`ph2d-painter-brush`** | **2 módulos NOVOS**: `plane.rs` (o fit de mínimos quadrados) e `sculpt.rs` (os acumuladores de dab). `height.rs` ganhou `grain_groove()` `pub(crate)`; `material.rs`/`spec.rs`/`spec_default.rs` mudaram pelo MATERIAL. | **Baixo p/ o sculpt** (módulos irmãos append-only; `HeightDab`/`accumulate_dab_height` intactos). **Médio p/ o material** — `spec.rs` é o `BrushSpec`, e ele é largo. |
| **`ph2d-editor-core`** | Arquivo NOVO `ids/chrome/painter_sculpt.rs` + `painter_impasto.rs` (material) + rail + `chrome/mod.rs`. | **Os 2 arrays do rail** (§3). O resto é append. |
| **`ph2d-editor-core/tests`** | 1 entrada em `RECONCILES_VIA` (`arch_mode_has_reconcile.rs`). | Lista append-only. |
| **`ph2d-ui-testkit`** | **`MockPanelHost::click_at`** — o harness que dirige o `dispatch_pointer` REAL. **+2 deps** (`ph2d-host`, `bumpalo`). | **É COMPARTILHADO.** Nasceu da lição do rig morto: *um widget não está pronto quando PINTA; está pronto quando um teste CLICA nele.* Se outra linha também mexeu no testkit, funda com cuidado — mas a API é aditiva. |
| **`shells/desktop`** | `project.rs` (o `PROJECT_SCHEMA`, §3) + `impasto_smoke.rs` (só doc + o canvas do smoke). | `project.rs` = o schema. |

---

## 5. Gates: o que EU rodei (e o que NÃO rodei)

Rodados na árvore da linha, verdes:

| Gate | Resultado |
|---|---|
| `cargo test --workspace` | ✅ **6655 passed / 0 failed** |
| `cargo clippy --workspace --all-targets --all-features` | ✅ **0** (fora o pré-existente da §6) |
| `rustup run 1.95 cargo fmt --all --check` | ✅ limpo |
| `typos` | ✅ limpo |
| `cargo machete` | ✅ limpo |
| Perf (sculpt, `--release --ignored`) | ✅ SMOOTH 3,1 · SCRAPE 2,9 · INFLATE 2,3 ms/move (alvo ≤4, kill 8), **plano entre 2048² e 4096²** |
| **Mutações** | ✅ **W1: 5/5 · W2: 10/10 · W3: 11/11** — todo gate tem o vermelho PROVADO |

**NÃO rodei** (é do ship, e o ship é do Enio): `nextest --cargo-profile ci-test` · `cargo audit`.

⚠️ **Espere 2-4 iterações no `ship.sh` mesmo assim** ([[project_integrator_ship_catches_latents_budget_iterations]]) — o gate por-linha não é o ship.

---

## 6. 🔴 DUAS coisas JÁ VERMELHAS na `main` — não são desta linha

Confirmadas rodando na `main` limpa. **Não perca tempo culpando a linha.**

### 6.1 — `cargo deny check` **FALHA na main** (exit 1)

```
deny.toml:107 → "RUSTSEC-2023-0089"  ─ no crate matched advisory criteria
advisories FAILED, bans ok, licenses ok, sources ok
```

O `atomic-polyfill` **ainda está no `Cargo.lock`** — quem mudou foi o **advisory-db** (o RUSTSEC saiu/mudou
upstream), deixando o `ignore` **órfão**, e o `deny` erra em ignore não-casado. É o drift previsto em
[[feedback_ship_parity_gaps_ci_only]].

**Fix (1 linha, e é da `main`, não da linha):** remover as 3 linhas do bloco em `deny.toml:105-107`
(comentário + o id). **Deliberadamente NÃO fiz na linha** — `deny.toml` é arquivo que TODA linha paralela
poderia editar, e carregar a mesma edição em cada uma é fabricar conflito para um one-liner que a `main`
possui.

### 6.2 — 1 warning de clippy em `tests/spike/src/bin/c11_flecs.rs:64`

*"casting to the same type is unnecessary (`i32` → `i32`)"*. Está na `main` desde `cf62198e`. Minha linha
não toca `tests/`.

---

## 7. Dois gates que passam por NÃO OLHAR (pré-existentes, documentados, não corrigidos)

Não são regressão desta linha, mas um gate que passa por não olhar é **pior** que gate nenhum — ele dá
sensação de cobertura. Quem for capear a superfície de painel começa por aqui:

* **`architecture_panel_wiring_parity`** lê o conjunto registrado **só de `src/populate.rs`** e nunca abre
  os irmãos `populate_*.rs`. Os dois lados saem vazios ⇒ **verde independentemente de qualquer coisa**.
  Vale igual pro `populate_deform` (o precedente). A cobertura REAL aqui é `tests/seam_sculpt.rs`, que
  **clica de verdade**.
* **`node_id_collisions`** é lista **mantida à mão**. Não tem os ids novos do Sculpt — e também não tem
  **nenhum** `PAINTER_DEFORM_*`, `PAINTER_IMPASTO_*`, `PAINTER_MASK_*` nem `PAINTER_SEL_*`.

---

## 8. Ordem de execução sugerida

1. **Espere o smoke do Enio** (§0). Se vermelho, a linha volta pra mim antes de integrar.
2. `git fetch && git rebase main` na linha (main só andou em `project-memory/` ⇒ deve ser trivial).
3. `bash scripts/foundational-integrate.sh` — o **gate da árvore COMBINADA**. É o único que cruza a linha
   com as outras; um `check --workspace` da árvore fundida é o que pega o merge limpo-mas-quebrado.
4. **Varra marcadores de conflito em CADA commit**, não só na árvore final
   ([[feedback_sweep_conflict_markers_every_commit]]).
5. Conferir os 4 números da §3 **contando**, não escolhendo.
6. `./scripts/ship.sh` — **espere a §6.1 (deny) explodir**; o fix é da `main`.
7. Push + babysit só sob ordem explícita do Enio.

---

## 9. O que fica ABERTO (não é dívida — é fila)

* **W4 — a família ADVECTIVA** (Grab · Pinch · Nudge · Rotate · Thumb): **não construir motor novo.** Fazer
  o motor do **Deform** carregar os planos do relevo (`h` + `covers` + `mats`) junto do RGBA. Destrava
  **cinco pincéis de uma vez**. Decisão de superfície a tomar: sub-modos do Sculpt ou toggle "afeta o
  relevo" no Deform? **Recomendação: o segundo** — um motor, um lugar.
* **W5 — Conserve** (a *bow wave*): pra onde vai a tinta raspada. O kernel **já computa** o volume
  deslocado (`sculpt_displaced_volume`, gateado) ⇒ é um **flag**, não uma reescrita.
* **Nenhum doc de `docs/Painter/` cobre a metade A** (material/wax/rig). Está só nas mensagens de commit.
  **A entrada do Painter no CLAUDE.md §5 precisa ser atualizada** com o material per-pixel, o Shine que
  mudou de dono, e o Sculpt.
* Herdados, dormentes: Bug #11 (Per-Layer Color, listras) e o handoff de perf de camadas-como-brush.
* **A TINTA EMPURRADA (o Push)** segue no fim da fila, como o Enio deixou. (Repare: o Conserve do Scrape e
  o Push são o **mesmo problema pelos dois lados**.)

---

## 10. As 4 armadilhas que ESTA linha pagou (leia antes de mexer no código dela)

| Armadilha | O que aconteceu |
|---|---|
| **Um widget não está pronto quando PINTA — está pronto quando um TESTE CLICA nele** | O rig de luzes shipou morto com tudo verde. E a de 2ª ordem: registrar como `Checkbox` emite `Toggled`, que o `event.rs` **não encaminha** ⇒ registrado e **ainda morto**. Daí o `MockPanelHost::click_at`. |
| **Um gate VERDE pode pinar um bug de DESIGN** | Gates provam que o código faz o que você **DISSE**; nenhum gate diz que o que você disse está errado. Só o smoke do Enio enxerga essa classe. |
| **Geometria sobre eixos de unidades diferentes** | `x` é texel, `h` é carga de tinta. `tan(36°)` cru inclinava o plano em 0,73 load/texel — **4× o teto do campo**. Toda grandeza geométrica cruza `DEPTH_UNIT_PX` na entrada. |
| **`invalidate_composite()` no caminho quente** | 148 ms/move (37× o kill) contra baseline 0,0. O aviso está em letras garrafais no `impasto::sync_relief_flags` e eu andei direto nele. `mark_dirty` e **nada mais**. |
