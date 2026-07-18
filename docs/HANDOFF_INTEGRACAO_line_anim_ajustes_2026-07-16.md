# Handoff de INTEGRAÇÃO — `line/anim-ajustes` (DIRETRIZ §1.5.9)

> A linha está **fechada**. Ela **não integra nem faz ship** — este documento vai ao
> **agente integrador**, por ordem explícita do Enio (CLAUDE §0.7).

## 1. Identidade

| | |
|---|---|
| Branch | `line/anim-ajustes` |
| Worktree | `Worktrees/line-anim/` |
| HEAD | `cc6c11c8` |
| Base do fork (merge-base com `main`) | `cdc3acc1` |
| Commits | **13** |
| Contratos congelados encostados | **NENHUM** — os 3 gates (`architecture_contract_surface`, `_tool_`, `_vector_`) rodados no HEAD, verdes |
| Deps novas | **NENHUMA** (`git diff --stat main...HEAD -- '*Cargo.toml'` vazio) |

## 2. Foundational / compartilhado tocado (tudo fora de `ph2d-timeline` + `ph2d-panel-timeline`)

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-editor-core/src/widget/text_input.rs` | ⚠️ **O widget de TODO o app.** Campo de texto virou viewport de UMA linha: layout sem quebra (`f32::INFINITY`), clip na caixa interna, scroll seguindo o caret. Antes ele passava a largura da caixa como `max_width` e o texto **embrulhava pra fora do campo**. | **Não** — muda a aparência de todo `TextInput` que transborda (que estava quebrado) |
| `shells/desktop/src/app_state.rs` | +2 campos em `App`: `clip_playhead: Playhead`, `last_timeline_keys_mode: bool` | Aditivo |
| `shells/desktop/src/main.rs` | Init dos 2 campos acima | Aditivo |
| `shells/desktop/src/render_loop/mod.rs` | Avança o `clip_playhead`; carimba `timeline.keys_mode`; na TROCA DE ABA sincroniza o loop da vista que ficou ativa | Aditivo, mas **insere linhas no meio do laço de frame** — ver §5 |
| `shells/desktop/src/render_loop/timeline_bridge.rs` | `duration()` virou `doc.view_end_seconds(timeline.keys_mode)` | Substituição de 1 linha |
| `shells/desktop/src/render_loop/timeline_bridge_tests.rs` | Split (602→264 LOC, cap 600) | — |
| `shells/desktop/src/render_loop/timeline_bridge_k_tests.rs` | **ARQUIVO NOVO** — metade K/solo dos testes da ponte | Novo |
| `shells/desktop/src/project.rs` | O load sincroniza os **dois** relógios (`playhead` e `clip_playhead`) | Aditivo |
| `shells/desktop/src/sim_populate.rs` | **`populate_sim_live` REMOVIDA** (o demo de 8 entidades da hierarquia) | **Remoção** |
| `shells/desktop/src/init.rs` | Deixou de chamar a função acima — o editor abre com cena VAZIA | **Remoção** |
| `docs/architecture/decisions/0115-*.md` | +29 linhas (emenda R8: a aba Keys tem playhead próprio) | Aditivo |
| `project-memory/MEMORY.md` + 1 memória nova | Índice + `feedback_an_impossible_inverse_is_a_reason_for_a_second_clock_not_a_readonly_control.md` | ⚠️ **Só ADICIONE** ao índice — [[feedback_a_shared_list_is_merged_against_todays_main]] |

## 3. Símbolos que podem COLIDIR (grepar por mesmo-símbolo, §1.5.5)

Nenhum `NodeId(N)` novo, nenhum token de cor novo, nenhuma chave i18n nova.
O que existe é **superfície pública nova** em `ph2d-timeline`:

- `DOC_VERSION: u32 = 7` — ⚠️ **valor único e ordenado.** Outra linha que também tenha
  bumpado o `DOC_VERSION` = **conflito de NÚMERO, não de texto**: o merge pode sair limpo com
  as duas alegando `7`. O certo é **somar** os bumps, não escolher um
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- `ClipStrip.marks: [f64; 4]` e `ClipStrip.lead_in: f64` — **campos APENDADOS** (postcard é
  posicional). Se outra linha apendou campo no MESMO struct, a ordem final decide o formato:
  concatene na ordem dos bumps de versão.
- `NamedClip.keys_loop_range` / `keys_loop_ping_pong` — idem, apendados.
- `pub use stack::{… , mark_index}` e `pub use apply::{apply_active_clip, …}` — linhas de
  re-export em `lib.rs`, alvo clássico de conflito textual trivial.
- `TimelineIntent::TrimStrip` / `::StretchStrip` ganharam campo **`from: f64`** — quem
  construir esses intents fora da linha quebra na compilação (bom: é erro, não silêncio).
- Módulos novos: `ph2d-timeline/src/strip_edge_edit.rs`, `.../doc_extent.rs` (filho de `doc.rs`),
  `ph2d-panel-timeline/src/strip_paint_tests.rs`.
- `TimelineState.keys_mode: bool` (campo novo, lido por `apply_intent`).

## 4. Contratos congelados

**Nenhum.** Os 3 gates rodam verdes no HEAD. Nada de `NodeOp`/`OpResolver`/`NodeManifest`,
`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`, nem `ph2d-vector-doc`/`-traits`.

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

1. **`render_loop/mod.rs` é ímã de merge** — três linhas foram inseridas no meio do laço de
   frame. Um merge textual limpo aqui pode estar **semanticamente errado** se outra linha
   reordenou o laço: o `keys_mode` é lido do painel e tem de ser carimbado **antes** do drain de
   intents. Confira a ORDEM, não só o texto ([[feedback_clean_text_merge_can_be_semantically_broken]]).
2. **fmt/typos pré-fork** — o gate de tofu (`no_tofu_glyphs`) pegou uma `→` num literal de teste
   DESTA linha; rode-o na árvore combinada, não só por-crate.
3. **LOC caps** — 4 arquivos foram splitados exatamente no limite nesta linha
   (`intent_apply.rs`, `doc.rs`, `timeline_bridge_tests.rs`, `strip_paint.rs`). O `cargo fmt` da
   integração **re-expande** e pode reestourá-los: **fmt ANTES de medir**
   ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]). Os gates de LOC moram na
   `ph2d-editor-core` e no shell — **não rodam** com `cargo test -p ph2d-timeline`.
4. **clippy latente / RUSTSEC / machete** — sem deps novas, mas a árvore combinada é outra árvore.

## 6. Ordem, dependências e o que smoke-testar

**Ordem:** os 13 commits são sequenciais e não reordenáveis. Dependências duras:
`b164e703` (playhead da aba Keys) é pré-requisito de `656ba73a` (loop por-vista) e de
`16914605` (fim por-vista); `91836ec9` (lead-in) é pré-requisito de `9481eefe` e do
gizmo de fade em `fd71b0dd`.

**Smokado e APROVADO pelo Enio:** playhead da aba Keys · loop por-vista + alça de mover no topo
da régua · fade pra fora (travel fade) · gizmos de quina (verde/vermelho, em L) · change bars ·
`F` enquadrando todas as strips · loop/go-to-end até a última strip.

⚠️ **NÃO smokado:**
- **`510f5063` — o campo de texto de UMA linha.** É o widget compartilhado: vale conferir
  **qualquer** campo de texto do app (rename de clip/marker/camada, campos do Inspector) com um
  nome longo — deve cortar na caixa e rolar com o caret, nunca embrulhar.
- **`cc6c11c8` — o editor abre VAZIO.** Confirmar que a hierarquia nasce sem linhas e que as
  cenas de smoke (`PH2D_STACK_SMOKE=1`, `PH2D_BUILD_SMOKE=…`, painter, flip) seguem
  populando o que precisam.

**Cena pronta:** `PH2D_STACK_SMOKE=1 cargo run -p ph2d-host-desktop` → abra **L**, aba **Arrange**.
