# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-07 — A NOVA UI DO VECTOR (D10, F1–F3)

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Tarefa: [`vetor_nova_ui/03_plano.md`](../vetor_nova_ui/03_plano.md) §4 (F0 + F1–F3), decisões
> fechadas em [`00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md) D10.

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **base (merge-base = `main`)** `2a925859d`
  (a linha tinha sido integrada e reaberta sobre o `main`; `git cherry` = 0 no início).
- Commits: `f6217180a` docs (estudo + plano + D10, trazidos do primário) · `afae11474` a obra ·
  `10c1fd0eb` curas do gate + fecho documental · `3dd997a61` clippy · e o commit deste doc.

## §0 — Para o `CLAUDE.md` §5.1 (≤ 700 bytes; gate `architecture_claude_md_cabe_no_orcamento`)

UI/UX: «Último:» → este ficheiro; `PH2D_OBJECT_MODE_SMOKE=1|4|6|7|8` → `1|4|6|7|8|9`; acrescentar
«Vetor em cinco lugares (D10)». Vector + Esqueleto: nada (o módulo não mudou de natureza).

## §1 — O que a linha entregou

| lugar | o que é | onde mora |
|---|---|---|
| **①** coluna flutuante | 17 ferramentas por FAMÍLIA (cor = `BarFamily::tint`, matizes `NodeCat*`), ícone por ferramenta, dica com a tecla; em baixo Fill · Stroke · `X` · `Shift+X` · `D`; o catálogo de formas é o FLYOUT da Forma (contornos cozidos pela mesma porta do canvas) | tabela `ph2d_panel_vector::bars::TOOLS`; pintor `ph2d_editor_core::screens::hero::float_bar::tool_bars` |
| **②** a fila de cima | o produtor do vetor entra na `tool_bar` (`ToolBars::row_lead` antes dos chips do gizmo, `row_tail` depois); por modo: Node (marquee, tipo de nó, Join/Close/Reverse/Weld), Pencil (Fidelity/Stabilizer + fonte da largura), Cut (Aplicar/Descartar com lâmina), Simetria (onde se desenha, ou ligada); Fill/Stroke/largura sempre; **Snap** e **⚙ Path** como pulldowns do módulo (`set_area_commands`) | `bars::tool_bars`/`bars::module_menus`; `tool_bar::bar_rail` |
| **③** painel | ordem nova (Alinhar no topo, Transform, Aparência, Fill, Stroke, específico do tipo, Efeitos; Avançado/Contêiner/Interação **recolhidos por omissão** via `set_collapsed_if_unchosen`); **saíram** `Tool`, `Shape` (catálogo), `Pencil`, `Symmetry`, `Snap`, `Path` (41 → 35 secções) | `paint_body_plan::corpo`, `COLLAPSED_BY_DEFAULT` |
| **④** ações da seleção | booleanas (≥2), arranjo, caminho (fora do Node); com seleção em toda ferramenta que não é de clique preciso (`bars::PRECISE`); **fixar** (vai ao topo da área) e **desligar** (*Ver ▸ Selection Actions* religa); as duas escolhas gravam-se por layout no `layout.txt` | `float_bar::{strip_rects,pinned_strip_rects}`, `chrome::selection_actions`, `layout_persist` |
| **⑤** rodapé | no layout *Vetor*, a pílula mostra a dica da ferramenta (17 dicas, cada oração medida no despacho do gesto), `x · y` do ponteiro, encaixe, zoom | `bottom_hud::{module_hint, paint_tool_hint_hud}`, `ph2d_app_vec::bars_publish` |
| **teclas** | padrão Illustrator: `V A P N M L T · Shift+M Shift+W Shift+O · K W C I · X Shift+X D · Ctrl+Alt+U/I/D/X`; **uma tecla = um clique no mesmo id** (`hero.apply_event`); ferramenta **temporária** = a tecla auto-repetiu antes de soltar (o atraso do SO, sem limiar inventado); sobre a timeline as letras cedem (`K` = chave) | tabela `ph2d_panel_vector::tool_keys`; aplicação `ph2d_app_vec::tool_keys`; costura `keyboard_cadeia.rs` |
| **conta-gotas** | 17.º `DrawMode::Eyedropper` (tecla `I`): o clique copia fill + stroke do caminho para a ferramenta e a seleção (pelos setters do seletor) | `ph2d_app_vec::eyedropper`, `vector_bridge::adopt_sample`, braço no `despacho_clique_vetor_premido` |
| **widget genérico** | `float_bar` (coluna, grelha de flyout, barra junto da seleção, painel, botão, amostra); **a barra do Quadro migrou para ele** (geometria e botão; as caras ficam do Quadro) | `screens/hero/float_bar.rs` |

**Decisões técnicas tomadas na obra (delegadas, padrão-ouro):**
- **ORDEM = PRIORIDADE do `⋯`, e a 1.ª pergunta é «funciona atrás do `⋯`?»**: à frente as amostras e
  todos os CAMPOS (incl. Segments da simetria); depois os botões do gesto; na cauda a simetria e os
  pulldowns. Medido: no mini a largura do traço ia para o `⋯`, onde não se edita.
- **Os chips MOVE/ROT/SCALE ficam** (decisão do dono de 05/10): em Edit eles vêm DEPOIS do gesto e
  vão para o `⋯` nos iPads; no desktop do dono cabe tudo (F0). *Se o dono os quiser fora do Edit, é
  pergunta a ele.*
- **`L`, `K`, `W` eram teclas GLOBAIS** (timeline, chave, física): com o vetor na mão passam a
  Elipse, Balde e Mistura — o mesmo precedente do `T` (que já roubava o painel Tokens). Sobre a
  timeline cedem. ⚠️ Comunicado ao dono no relatório; se ele preferir outra regra, a tabela é uma.
- **O frequente em dois lugares** (plano §2): as amostras estão em ① e ②; booleanas/arranjo/nós
  ficam também em ③ «Avançado». O gate *uma porta* declara a lista (`FREQUENT`).

## §2 — F0, medido (antes de código; tornou-se gate)

| | iPad 12,9 | iPad 11 | iPad mini | dono 1930 |
|---|---:|---:|---:|---:|
| largura útil da fila | 738 | 643 | 609 | 1302 |
| ① 17 + cor (alto) | cabe | cabe | cabe | cabe |
| Edit+Node: na fila / atrás do `⋯` | 19 / 11 | 16 / 13 | 15 / 15 | tudo |
| Lápis + simetria radial | 15 / 15 | 11 / 18 | 11 / 18 | tudo |

Gate: `ph2d-panel-registry-init/tests/it/the_vector_bars_answer_a_real_pointer.rs`
(`the_longest_vector_rows_fit_one_line_and_only_operable_entries_overflow`).

## §3 — Foundational tocado e contratos

- **`ph2d-editor-core`**: `interaction::{tool_bars, ToolBars, BarButton, BarFace, BarFamily}` (novo) e o
  campo `WidgetStore::tool_bars` (+ `tool_bars_ops.rs`: `set_tool_bars`, `open_picker_on`,
  `note_pointer`/`last_pointer`, prefs da ④); `widget::ToolRailEntry::Number` (variante NOVA — todo
  `match` exaustivo noutra linha parte ALTO) e `Compound::active`; `screens::hero::float_bar` (+
  `tool_bars`), `board_bar` migrado; `tool_bar::bar_split` passa a perguntar `horizontal_lines` (a mesma pergunta do pintor —
  refactor por construção: a quebra que o motivou no 1.º rascunho do gate vinha do TESTE usar
  `aw − 16` em vez do `content_rect`, não do produto);
  `bottom_hud::{module_hint, paint_tool_hint_hud}`; `GridView::{world_to_screen, screen_to_world}`;
  `ids`: `SELECTION_ACTIONS_PIN/OFF`, `MENUBAR_VIEW_SELECTION_ACTIONS`; `ModuleTruth::SelectionActions`
  (`MODULE_TRUTHS` 17 → 18, `OWN_ROWS` 8 → 9, menu *View* +1 linha); `chrome::selection_actions`
  (z=46, `dispatch_all` regenerado pelo `ph2d-chrome-sync`); `icons.rs` +29 ícones (167) e os testes
  cortados para `icons_tests.rs` (tecto de LOC).
- **`ph2d-tool-vector`**: `DrawMode::Eyedropper` (16 → 17; o censo `the_list_and_the_enum_agree…`
  conta-o), `ids::{VECTOR_MODE_EYEDROPPER, VECTOR_COLOUR_FOCUS/SWAP/DEFAULT}` (`ids/vector_colour.rs`).
- **Contratos congelados (§6): nenhum.** `PanelEvent` intocado (as teclas são cliques); `Tool=12` intocado.

## §4 — Superfície de colisão

Crates: `ph2d-editor-core` (acima) · `ph2d-panel-vector` (`bars`, `tool_keys`, `shape_outlines` ← era
`paint_catalog`, `symmetry_segments` ← era `paint_symmetry`; **apagados** `paint_path_ops.rs`,
`paint_pencil.rs`; `paint_modes`, `paint_arrange`, `paint_body_plan`, `section_scope` (sem
`Scope::Modes`), `state*`, `event*`, `populate*`; testes: `the_bars_carry_the_gesture.rs` novo,
`host_aberto()` em `tests/it/main.rs`, **apagados** `seam_pencil.rs`, `seam_snap.rs` e os testes de
costura dos controlos que mudaram de casa) · `ph2d-app-vec` (`bars_publish`, `tool_keys`,
`eyedropper`, `VecState::tool_hold`, cena 9 no `vector_mode`) · `ph2d-tool-vector` · `ph2d-i18n`
(`vector_bars.rs` novo na cadeia; `chrome_menus`, `chrome_panes`) · `ph2d-panel-registry-init`
(teste ponta-a-ponta + dev-deps `ph2d-tool-vector`, `ph2d-symmetry`). Shell:
`input_dispatch/keyboard_cadeia.rs` (bloco das letras), `despacho_clique_vetor_premido.rs` (conta-gotas),
`vec_text.rs` (−`vec_text_toggle_mode`), `layout_persist.rs` (+2 campos). Gates tocados:
`the_painted_control_reaches_a_consumer` (cura do `in_use_item`), `the_menu_bar_relocates…`
(`OUTSIDE_WINDOW_TOGGLES` 5 → 6), `popover_band` (4 popovers), `every_name_the_engine…` (`bars.rs`),
`seam::every_section_header…` (41 → 35), censo de palavras (`TABLES` + `vector_bars.rs`).

**Contadores (DELTA sobre `2a925859d`) — somam entre linhas, contam-se:**
- `VECTOR_SECTIONS` 41 → **35** · `DrawMode::ALL` 16 → **17** · ícones 138 → **167**.
- `MODULE_TRUTHS` 17 → **18** · `OWN_ROWS` 8 → **9** · `OUTSIDE_WINDOW_TOGGLES` 5 → **6**.
- i18n: `+vector_bars.rs` (≈45 chaves `panel.vector.{hint,bar,key}.*`, `panel.vector.mode.eyedropper`);
  `+chrome.menu.selection_actions`, `+chrome.selection_actions.{pin,off}`, `+chrome.hud.tool_hint`.
- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+9**.
- `PROJECT_SCHEMA`: **inalterado** (o `layout.txt` é por máquina, não é o projecto).

**O que um merge pode partir (ALTO):** quem faça `match` exaustivo sobre `ToolRailEntry` ou `DrawMode`;
quem leia `paint_catalog`/`paint_symmetry`/`Scope::Modes`/`section_scope::{CATALOG,PENCIL,symmetry}`;
quem chame `float_bar::strip_rects` (4.º parâmetro: o lado). **Em silêncio:** outra linha que escreva
`set_area_commands` em todo quadro com o canvas do vetor (hoje o vetor é o único escritor; ele limpa
a parte dele no quadro em que larga a mão); um painel novo que pinte um id `VECTOR_*` que as barras
já pintam (o gate *uma porta* só vê as barras).

## §5 — Gate final

Atestado do `verificador` sobre `10c1fd0eb` (+ a cura `3dd997a61`, clippy do workspace limpo a seguir),
`load` 118 → 31 durante a corrida:
- `fmt --check` ✓ · `CARGO_BUILD_WARNINGS=deny check --workspace --all-targets` ✓ · clippy como o
  `ship.sh` (`--features ph2d-spike/bevy_ecs -- -D warnings`) ✓.
- `nextest-impacted` (`BASE=2a925859d`, `NO_FAIL_FAST=1`): **16 769 / 16 769**.
- Censos da árvore 12/12 (114/114) · `doc-index --check` ✓ · `panel_files_under_loc_cap`,
  `workspace_src_files_under_loc_cap` ✓.
- Catraca da shell `the_shell_only_shrinks`: **185 674 / TECTO 189 041** (folga 3 367). Delta da linha
  sobre a base (185 537): **+137 = +46 em `src`** (a costura das letras, o braço do conta-gotas, dois
  campos da persistência; `vec_text_toggle_mode` saiu) **+ 91 em `tests`**.
- **Mutação 15/15 a sangrar** (agente `mutacao`, verde visto antes de cada uma, árvore limpa no fim):
  a tabela da ① (Eyedropper), a lâmina do Corte, a simetria radial, controlo sem casa (Reverse),
  porta do painel (Swap), id pintado duas vezes, dobra por omissão, hit do botão, fixar, prioridade da
  fila (a largura ia ao `⋯` no mini), tecla do Balde, ferramenta temporária, cedência à timeline,
  rodapé só no layout Vetor, e a cura do `in_use_item`.

## §6 — Smoke do dono

Ver o relatório ao dono (passos numerados). Foto: [`depois_01`](../vetor_nova_ui/fotos/depois_01_cena9_caneta_na_mao_com_duas_formas.png).

## §7 — O que fica ABERTO

- **F4** (radial com a ① do vetor; `Ctrl+K` que mostra onde o comando mora) e **F5** (iPad:
  modificadores na tela, QuickShape, gestos) — próxima jornada, depois do smoke.
- ✅ **Ícones 50 % maiores (pedido do dono, 08/10, `f23bf3f9a`)**: a cara dos botões do vetor (①, ④,
  flyout) desenha `1,5×` o miolo no MESMO botão de 24 px (`float_bar::module_face_rect`,
  `MODULE_FACE_GROWTH`); o Quadro não mudou. Gate `a_module_icon_draws_half_again_bigger_inside_the_same_button`
  (mutação `1.0` sangra; a 1.ª redacção comparava com a própria constante e a mutação sobrevivia).
- A **ferramenta temporária** está provada por teste (tecla com/sem auto-repetição); só o smoke do
  dono diz se o atraso de repetição do SO dele é o limiar que ele espera.

## §8 — Binário de smoke (2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.26s
```
