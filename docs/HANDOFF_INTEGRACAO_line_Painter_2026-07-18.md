# HANDOFF DE INTEGRAÇÃO — `line/Painter` (2026-07-18, 2ª rodada)

> Para o **agente integrador**. DIRETRIZ §1.5.9. A linha está **fechada e parada**; não integrou nem
> pushou nada.
>
> ⚠️ **Substitui o conteúdo anterior deste arquivo** (1ª rodada: 6 commits sobre `cdc3acc1` — luz na GPU,
> bola limitada, closing do Inflate). Aquela rodada **já integrou**; esta é a de cima dela.
>
> **Smoke APROVADO pelo Enio** nesta rodada: Smear · Chisel · Inflate · Conserve removido · Accumulate
> escondido no impasto.

## 1. Identidade

| | |
|---|---|
| Branch | `line/Painter` |
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` |
| HEAD | `e80689f4` |
| Base do fork (merge-base com `main`) | `389676f9` |
| Commits | **28** |
| Diff | 57 arquivos |
| Árvore | limpa · `clippy --workspace --all-targets` **0 warnings** · `fmt` aplicado · teto de LOC verde |
| Suítes | **workspace 7651 passed / 0 failed** (nextest, `--cargo-profile ci-test`) |

⚠️ **`main` não se moveu desde o fork** (`389676f9` é o HEAD dela). Enquanto isso for verdade, esta linha é
**fast-forward puro** — sem rebase, sem resíduo para o Mergiraf.

## 2. Foundational / compartilhado tocado, e por quê

Nove arquivos fora de `ph2d-tool-painter` / `ph2d-painter-brush`:

| arquivo | o quê | natureza |
|---|---|---|
| `ph2d-editor-core/src/ids/chrome/painter_sculpt.rs` | **REMOVE** `PAINTER_SCULPT_CONSERVE`; `PAINTER_SCULPT_CLICKS` **12 → 11** | ❌ **subtrativo** |
| `ph2d-panel-painter-layers/src/paint_sculpt.rs` | tira a row do Conserve do card | subtrativo |
| `ph2d-panel-painter-layers/src/populate.rs` | tira o registro do Conserve | subtrativo |
| `ph2d-panel-painter-layers/src/brush_fallback.rs` | tira `sculpt_conserve` / `sculpt_conserves` | subtrativo |
| `ph2d-panel-painter-layers/src/paint_brush.rs` | esconde o Accumulate com impasto (`&& !brush.impasto`) | +1 linha |
| `ph2d-panel-painter-layers/tests/seam_sculpt.rs` | remove o seam gate do Conserve | subtrativo |
| `ph2d-panel-painter-layers/tests/seam.rs` | **+1 gate** (presença/ausência do Accumulate) | aditivo |
| `shells/desktop/src/render_loop/push_look_probe.rs` | a sonda deixa de armar o Conserve | 2 linhas |
| `CLAUDE.md` | §5 do Painter | 🔴 texto — **toda linha edita o §5** |

**Nenhuma crate nova. Nenhuma dependência nova** (`git diff -- '*Cargo.toml'` vazio ⇒ `machete`/`deny` não
têm o que reclamar por parte desta linha). `Cargo.lock` intocado.

## 3. Símbolos que podem colidir (§1.5.5)

**A linha não introduz id, const, variant nem token novo — ela REMOVE um.** Para o grep de mesmo-símbolo:

* **`PAINTER_SCULPT_CONSERVE` — removido.** Se outra linha o referenciar, o merge não compila. A resolução
  é **remover a referência**, não ressuscitar o id: o Conserve saiu por ordem explícita do Enio depois do
  smoke, e ressuscitá-lo devolve um checkbox que o produto não tem mais.
* **`PAINTER_SCULPT_CLICKS` — o TAMANHO do array mudou (12 → 11).** É aqui que uma linha que acrescentou um
  chip ao card do Sculpt colide. O conserto é o **número**, não o conteúdo.
* `BrushSettings` perdeu `sculpt_conserve` / `sculpt_conserves`; `SculptSnap` perdeu `bank`.
* `SculptState.last_bank_center` → **`last_dab_center`** (renomeado, mesmo tipo). Quem grepar o nome antigo
  não acha; o campo continua existindo e agora serve ao **eixo do V do Chisel**.
* `HeightFields` e `accumulate_dab_height` estão **idênticos à `main`** — o Accumulate no relevo foi
  construído e **revertido** (`1e22c1e0` / `82461682`). Verificado por grep: `stroke_accum`, `live_accum` e
  `pass_normalizer` = **0 ocorrências**.

## 4. Contratos congelados (§4)

**Nenhum encostado.** `Tool` / `RasterEditTool` / `CanvasPaintTool` / `PanelEvent` e `NodeOp` /
`OpResolver` / `NodeManifest` intactos; os gates `architecture_tool_contract_surface` e
`architecture_contract_surface` passam sem mudança. **Nenhum ADR é exigido por esta linha.**

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

* **`fmt` / `typos` pré-fork:** a linha rodou `cargo fmt --all`, que pode ter reformatado arquivos que ela
  não tocou logicamente. Drift acusado no ship é isso.
* **`deny` / `audit`:** sem deps novas, mas o RUSTSEC avança sozinho — advisory publicado depois do fork
  aparece no ship e **não é regressão desta linha**
  ([[project_integration_prefork_lines_ship_drift]]).
* **`nextest-impacted`** não cobre o que a linha não tocou; o ship roda a suíte inteira.
* ⚠️ Um `✗` do ship pode ser **o ambiente**, não o código
  ([[feedback_a_ship_x_can_be_the_environment_not_the_code]]).

## 6. Ordem, dependências e o que smoke-testar

**Integre a linha INTEIRA.** Os 28 commits são sequenciais e há um par `feat` + `Revert` no meio
(`1e22c1e0` / `82461682`) que só faz sentido junto — cherry-pick solto quebra.

**Smoke aprovado nesta rodada** (não precisa repetir): Smear como campo · âncora do Push · Chisel (sulco
paralelo ao traço) · Inflate (defaults Sharper + 8 px) · Conserve removido · Accumulate escondido no
impasto.

**AINDA SEM SMOKE — o integrador deve devolver ao Enio:**

* **luz do impasto na GPU** ([handoff](HANDOFF_line_Painter_gpu_light_2026-07-18.md));
* **closing morfológico do Inflate** ([handoff](HANDOFF_line_Painter_inflate_closing_2026-07-18.md));
* **Filter Layer / Filter Stroke** do Sculpt (W5b).

## 7. Riscos, na ordem em que eu olharia

1. **O id do Conserve é o único ponto subtrativo em foundational.** É onde uma 2ª linha colide (§3).
2. **`CLAUDE.md` §5** — o de sempre: toda linha edita o mesmo parágrafo. O conteúdo desta rodada é
   auto-contido (entradas do Conserve removido, Chisel, Inflate, Accumulate) e pode ser fundido por união.
3. **Três gates de perf novos são `#[ignore]`** (`warp_perf_kill_criterion`, `sculpt_perf_kill_criterion`
   corrigido, `smear_perf_kill_criterion`) — **não rodam no CI**, por desenho. Rodar à mão:
   `--release -- --ignored`.

## 8. O que a linha deixa aberto (não bloqueia a integração)

* **Relevo do papel** — barreira aberta pelo Enio, mas o item mudou de forma: é **extração de substrato com
  ADR na frente** ([doc 19](Painter/19_relevo_do_papel_investigacao.md)).
* **Accumulate no relevo** — construído, reprovado no smoke, revertido. Desenho e medições em
  [doc 20](Painter/20_accumulate_na_mesma_pincelada.md), **para ninguém reconstruir sem saber que já foi
  tentado**.
* **Pen-up do Inflate** (render duplo, ~3,7 ms/traço) — diagnosticado e recusado por medição
  ([handoff da linha §9.6](HANDOFF_line_Painter_smear_field_2026-07-18.md)).

---

**Resumo:** linha `Painter` pronta — HEAD `e80689f4`, **28 commits**, base `389676f9`, fast-forward puro.
Foundational: só o id `PAINTER_SCULPT_CONSERVE` (removido, com o array de clicks 12→11), 4 arquivos do
painel, a sonda do shell e o `CLAUDE.md` §5. **Zero contrato congelado, zero dep nova, zero símbolo novo.**
**Aguardo ordem de integração.**
