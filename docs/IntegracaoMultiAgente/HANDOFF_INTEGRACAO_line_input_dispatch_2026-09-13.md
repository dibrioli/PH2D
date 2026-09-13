# HANDOFF de INTEGRAÇÃO — `line/input-dispatch` (cliques, arrastos, roda e teclas)

> Modo L. Branch `line/input-dispatch`, worktree `Worktrees/line-input-dispatch`. **Nada integrado, nada enviado.**
> Leitor: o agente INTEGRADOR (e a próxima LLM que tocar no despacho de entrada).
> ⛔ **Nenhum produto mudou:** cada peça MOVE um corpo verbatim (prova por texto contra o HEAD) e a ORDEM em que os
> consumidores reclamam um evento é a mesma — os gates que a medem leem o despacho EMENDADO (`input_text`, §4).

## §1 · Identidade

| | |
|---|---|
| branch | `line/input-dispatch` |
| HEAD | `c51a055c8` (o `fim_do_item` na forma do `rustfmt`) — o handoff entra no commit seguinte |
| merge-base com `main` | `29ff6576e` — o `main` andou UM commit desde então (`0bfee712e`, só `docs/IntegracaoMultiAgente/ESTADO_W2_2026-09-12.md`: sem colisão) |
| commits da linha | 67 (+ o do handoff) |

## §2 · O veredito, em números

| grandeza | antes (base) | depois |
|---|---:|---:|
| `input_dispatch.rs::on_mouse_input` (`fn_loc_caps`) | 3 102 | **≤ 200, fora da lista** |
| `input_dispatch/gizmo_drag.rs::advance_gizmo_drag` | 708 | **≤ 200, fora da lista** |
| `input_dispatch/keyboard.rs::key_input` | 544 | **≤ 200, fora da lista** |
| `input_dispatch.rs::on_cursor_moved` | 375 | **≤ 200, fora da lista** |
| `input_handlers.rs::handle_editor_key` | 358 | **≤ 200, fora da lista** |
| ficheiro `input_dispatch.rs` (`file_loc_caps`) | 7 115 (7 117 no bloco) | **545, fora da lista** |
| ficheiro `input_dispatch/gizmo_drag.rs` | 816 | **258, fora da lista** |
| ficheiros do território acima de 600 | 2 | **0** (o maior: `despacho_clique_gizmo.rs` 582) |
| entradas novas em qualquer lista | — | **0** |
| linhas `.rs` em `shells/desktop` (quota da linha: +1 200) | 193 205 | **194 363 (+1 158)** |

## §3 · Ficheiros tocados FORA do território (e porquê)

O território é `shells/desktop/src/input_dispatch.rs` · `shells/desktop/src/input_dispatch/**` ·
`shells/desktop/src/input_handlers.rs`. Fora dele:

- `shells/desktop/tests/it/input_text.rs` (**NOVO**, a lente) + `tests/it/main.rs` (+1 `mod input_text;`).
- `shells/desktop/tests/it/fn_loc_caps.rs` · `file_loc_caps.rs`: SÓ as entradas do território desceram ou saíram,
  sem reordenar (cerca §2.3). ⚠️ Colisão TEXTUAL esperada com as outras duas linhas nas mesmas listas.
- 52 gates em `shells/desktop/tests/it/` trocaram a linha que OBTÉM o texto (`include_str!`/`read_to_string` → a
  lente) — §5.1; e seis gates da shell tiveram uma agulha ou uma janela re-apontada, cada uma com mutação (§5.2).
- `crates/ph2d-editor-core/tests/it/the_rail_names_a_consumer_for_every_chip.rs`: o `ReadBy` do `TOOL_PIVOT`
  segue o leitor para `input_dispatch/despacho_clique_gizmo.rs` (só essa entrada).
- ⚠️ **Gates lidos por DUAS linhas (cerca §2.4):** `the_highlight_has_one_source.rs` — esta linha editou SÓ a
  entrada `ARMED` do despacho (a unidade `DISPATCH`, §5.2); o `FRAME` é da `line/render-bodies`.

Nenhum `mod` novo no `main.rs`, nenhum campo novo na `App`, nenhum contrato congelado, nenhum schema, nenhum
registo, nenhuma dependência nova.

## §4 · O MÉTODO — ramos pela mesma ordem, e a lente que os lê

**O molde:** cada secção de uma função gigante muda-se VERBATIM para um método `impl crate::App {
pub(super) fn ramo_x(&mut self, …) -> bool }` num ficheiro IRMÃO, chamado no sítio exacto por
`if self.ramo_x(..) { return; }`. ⛔ Nenhum gancho genérico (nada de `Vec<Box<dyn Handler>>`, trait de reclamante ou
registo de callbacks).

- **O «consumido?»:** dentro do ramo, `return;` do corpo vira `return true;` (SÓ no código, nunca em comentários
  ou strings — `ramo.py` separa código de prosa); a cauda devolve `false`. Um ramo cujo braço sempre devolvia
  vira UNITÁRIO (sem sinal) e o braço mantém o `return;` depois da chamada.
- **O empréstimo longo do bloco do gizmo** (`if let Some(gfx) = self.gfx.as_mut() && let Some(hero) = …`): cada ramo
  RE-EMPRESTA dentro de si com o mesmo `if let`, e é chamado DEPOIS do último uso do empréstimo de fora (NLL). Os
  valores que atravessam vão por parâmetro `Copy` ou num contexto nomeado e `Copy`: `AlvoDoClique`
  (`despacho_clique_gizmo.rs`, 8 campos) e `EscritaDoGizmo` (`gizmo_drag_escrita.rs`, 8 campos) — zero clones,
  zero alocações novas por evento.
- **Um `match` partido pelo braço `_`:** os braços seguintes vão, pela mesma ordem, para um `match code {}` dentro de
  um ramo, com o re-empréstimo `let Some(gfx) = self.gfx.as_mut() else { return; };`.
- **Módulos filhos por `#[path]` no PAI** (para o `input_dispatch.rs` não crescer): `keyboard.rs` →
  `keyboard_cadeia.rs`; `input_handlers.rs` → `input_dispatch/handlers_teclas_editor.rs`; `gizmo_drag.rs` →
  `gizmo_drag_escrita.rs` + `gizmo_drag_calculo.rs`.
- **O CORTE do índice** (a última etapa): testes, métodos auxiliares e funções livres mudam-se para
  `input_dispatch/despacho_*.rs`; os testes com o MESMO nome de módulo (`#[path]`), os métodos privados passam a
  `pub(super)` (troca contada), e os itens `pub(crate)` voltam ao índice por `pub(crate) use` — os caminhos
  `crate::input_dispatch::…` do resto da shell ficam iguais.

**A lente** — `shells/desktop/tests/it/input_text.rs`, gémea do `frame_text.rs`:
- portas `mouse_input` · `cursor_moved` · `mouse_wheel` · `key_input` · `editor_key` · `gizmo_drag`, e `file(rel)` /
  `dispatch()` / `keyboard()` / `handlers()` / `gizmo_drag_file()`: o texto com cada `self.ramo_*(` EMENDADO pelo corpo
  do ramo, recursivo, e as definições de ramo com o corpo apagado; a emenda escreve o `return true;` de volta como
  `return;` (o texto emendado fica IGUAL ao original);
- `dispatch()` = o índice + todo `input_dispatch/despacho_*.rs` por ordem de nome;
- marcadores ASCII (`/* [[ramo nome]] */`, `// [[ficheiro rel]]`) — ⚠️ o `⟧` original tinha 3 bytes e caiu no corte
  de 700 bytes de um gate (`03be3d27c`);
- `fim_do_item(rest)`: o fim de um item de `impl` é o PRIMEIRO irmão em qualquer visibilidade ou o fecho do `impl`;
- autotestes (8 — são o ONLY-B esperado da prova, todos em `ph2d-host-desktop::it input_text::`):
  `every_door_is_whole_and_every_ramo_is_called` · `the_splice_follows_calls_in_order_and_skips_fields_and_prose` ·
  `a_call_split_by_rustfmt_is_still_a_call_and_only_self_calls` ·
  `inline_turns_the_consumed_signal_into_the_doors_return_and_nothing_else` ·
  `a_ramo_definition_is_blanked_and_its_signature_kept` · `the_end_of_an_item_is_the_first_sibling_in_any_visibility` ·
  `the_reconstructed_dispatch_still_has_every_region` (PISO: as 14 regiões do ficheiro de 13/09) ·
  `a_called_ramo_that_is_not_found_fails_loud` (`should_panic`).
- ⚠️ **A posição no `dispatch()` é ordem de execução SÓ para os handlers.** O corte mudou a ordem das REGIÕES
  (antes: livres · métodos · despacho · testes; depois: despacho · métodos · testes · livres): uma agulha que
  compare um ajudante com o despacho mede ENDEREÇOS. Os três blocos de métodos mantêm a ordem entre si pelo nome
  dos ficheiros (`janela` < `modos` < `picks`); os três de funções livres ficaram em ordem INVERSA entre si.

## §5 · Os gates re-apontados, e a mutação de cada um

### §5.1 · A troca da FONTE (52 gates + a lente + o `main.rs`, `47069ce3a`)

Os gates trocaram só a linha que obtém o texto; nenhuma asserção mudou. Prova em três metades: base 810/810; a lente
VAZIA reprova 125 testes, e os que passam nos 52 módulos e tocam a lente são TRÊS ausências; INJECÇÃO nessas três
(`the_scene_order_door_is_gone_from_the_shell`, `the_hover_pick_happens_exactly_once`,
`the_generic_translate_does_not_reseed_a_joints_anchors`) reprova as três. Ficaram de propósito no ficheiro:
`src/layout_scroll_gesture_tests.rs` (a roda fica no índice) e `src/morph_arrow_seam_tests.rs` (os três statements
que ele ordena ficam no índice do `key_input`).

### §5.2 · As agulhas re-apontadas

| gate | o que media mal | cura | mutação (restauro byte a byte + `touch`) |
|---|---|---|---|
| `the_highlight_has_one_source::only_the_listed_gestures_arm_a_sound` (`1fd48e021`) | contava `pending_ui_sound = Some` POR FICHEIRO, `("src/input_dispatch.rs", 2)` | a unidade `DISPATCH` = `input_dispatch.rs` + `despacho_*.rs`, com piso (índice + ≥ 1 ramo) e a metade dos intrusos a não acusar a unidade | M1 uma 3.ª agulha num `despacho_*` → reprova pela contagem · M2 a agulha em `keyboard.rs` → reprova pelos intrusos |
| `the_node_press_freezes_a_live_shape_recipe` (`b5851ac9f`) | fim da janela = `None => {` com 28 espaços e `unwrap_or(len)` (esticava em silêncio) | o próximo `None => {` a qualquer coluna, e a falta dele REPROVA | N1 o `freeze_shape_recipe` movido para o braço `None =>` → reprova |
| `the_pencil_owns_its_whole_gesture` press (`b5851ac9f`) | fim = `// Modo Connect` com 20 espaços pelo `window()` (cai no fim do texto) | sem coluna, e uma asserção exige que o fim exista | P1 a semente `pencil_hand.begin` movida para o Connect → reprova |
| `joint_anchor_gizmo::the_joint_anchor_down_opens_the_anchor_drag_for_its_side` (`ccc60f80e`) | o argumento `joint,` com 28 espaços à frente | `args.lines().any(\|l\| l.trim() == "joint,")` | `joint,` → `joint.clone(),` reprova; a mutação histórica do gate reprova |
| `the_rail_names_a_consumer_for_every_chip` (`ccc60f80e`) | `ReadBy(input_dispatch.rs)` | `ReadBy(input_dispatch/despacho_clique_gizmo.rs)` | com o caminho antigo reprova |
| `pulley_wheel_handles::the_rope_eyedropper_arms_a_modal_pick_against_the_route` (achado da AUDITORIA, depois do corte) | `rope_guard < generic \|\| generic < fn wheel_rope_pick_click`, com o `generic` a casar num método auxiliar (`fn canvas_pick` e `self.begin_selection` não existem): verdadeiro por construção antes (2.ª metade) e depois (1.ª) | a ordem no CLIQUE emendado (`input_text::mouse_input`), entre `if self.wheel_rope_pick.is_some()` e o picking real `crate::hover_highlight::pick_objects_at(`, sem `\|\|` | `mut_pulley.py`: um picking genérico escrito ANTES do guard (texto, dois binários) — asserção velha SOBREVIVE, cura MATA |
| `the_preview_owns_the_pointer_and_the_undo::body_of` · `um_aperto_no_canvas_larga_o_teclado_do_painel::corpo_do_on_mouse_input` · `the_pencil_owns_its_whole_gesture` move (`8e782e68f`) | o fim do método por UMA visibilidade: com os métodos a `pub(super)` noutro ficheiro, a janela esticaria por cima dele | `input_text::fim_do_item` | M1 sem a fronteira `pub(super)` → o gate da lente reprova · M2 sem o fecho do `impl` → reprova |

**E a mutação de PRODUTO, depois do corte** (`mut_janela_lapis.py`): a lente lê o fonte em tempo de execução, então
a mutação é só de TEXTO e corre contra dois binários `it` já compilados — um com a fronteira VELHA, outro com a
CURA. Apagar o `screen_to_world` do corpo do `vec_pencil_drag_move`: controle (os dois passam sem mutação) ·
fronteira velha **SOBREVIVE** (a janela estica até outro ficheiro e acha um `screen_to_world` alheio) · cura
**MATA**. ⚠️ Para as duas janelas do `on_mouse_input` não existe HOJE mutação de produto que as distinga: o que a
fronteira velha passaria a varrer são assinaturas de ramo apagadas pela lente e cabeçalhos de ficheiro — a cura é
preventiva, e está provada na porta (M1/M2 acima).

### §5.3 · O levantamento que decidiu que eram SÓ estes

`sonda_ordem.py` (fora da árvore): nos 42 gates que leem o `dispatch()`, cada agulha cuja 1.ª ou última ocorrência
muda de região com o corte, e cada par que inverte a ordem, foi lido no gate. O corte muda a ordem das regiões
(hoje: livres · métodos · despacho · testes; depois: despacho · métodos · testes · livres), e todos os outros
gates comparam posições DENTRO do despacho, afirmam presença ou contagem, ou abrem janela num texto que se muda
inteiro (a ordem entre os três blocos de métodos é preservada pelos nomes dos ficheiros). ⚠️ Uma fraqueza
PRÉ-EXISTENTE que esta sonda só NOMEOU — o `||` do `pulley_wheel_handles`, verdadeiro por construção — a auditoria
de costura mediu e ela foi CURADA (§5.2): *nomear um gate vácuo e deixá-lo vácuo é a mesma licença que ele dava.*

## §6 · A tabela peça → ficheiro → linhas

Derivada do `git log --numstat 29ff6576e..HEAD` (`tabela_pecas.py`, fora da árvore), sem as linhas das duas listas de
LOC, que descem em quase todos os commits. Cada commit compila e passa a suíte da shell sozinho.

| commit | peça | ficheiros (+/−, sem as listas de LOC) |
|---|---|---|
| `47069ce3a` | o despacho de entrada como TEXTO pela ordem em que corre (`input_text`) -- 52 gates re-apontados ANTES de extrair | `it/a_frames_handle_resizes_it_and_does_not_scale_it.rs +1/-3` · `it/a_placed_instance_lands_a_screen_step_from_its_main.rs +1/-6` · `it/a_refused_gesture_speaks_on_screen.rs +6/-1` · `… +51 ficheiros` |
| `97d7c4c00` | o pan do meio + a barra lateral saem do `on_mouse_input` para um ramo (3102 -> 3016) | `input_dispatch.rs +3/-87` · `input_dispatch/despacho_clique_reclamantes.rs +97/-0` |
| `1fd48e021` | as recusas da trava do Painter contam-se no DESPACHO inteiro, nao no `input_dispatch.rs` | `it/the_highlight_has_one_source.rs +31/-6` |
| `17581e380` | o fim do arrasto do gizmo sai do braco Up do `on_mouse_input` (3016 -> 2935) | `input_dispatch.rs +3/-82` · `input_dispatch/despacho_clique_largar.rs +90/-0` |
| `5f6451a00` | o braco Up do gizmo inteiro vira um ramo -- o largar do botao primario (2935 -> 2811) | `input_dispatch.rs +1/-126` · `input_dispatch/despacho_clique_largar.rs +131/-0` |
| `145b7fd09` | o arrasto que o pick de canvas abre sai para um ramo (on_mouse_input 2672) | `input_dispatch.rs +3/-139` · `input_dispatch/despacho_clique_pick.rs +164/-0` |
| `0246b6cf6` | a selecao que o pick de canvas decide sai para um ramo (on_mouse_input 2580) | `input_dispatch.rs +1/-93` · `input_dispatch/despacho_clique_pick.rs +104/-0` |
| `740f8c0f5` | o pick de canvas inteiro vira um ramo -- hits, ordem e ciclo (on_mouse_input 2477) | `input_dispatch.rs +1/-104` · `input_dispatch/despacho_clique_pick.rs +110/-0` |
| `101093c2a` | a alca do gizmo (escala/rotacao) sai para um ramo com sinal (on_mouse_input 2333) | `input_dispatch.rs +3/-145` · `input_dispatch/despacho_clique_gizmo.rs +170/-0` |
| `9015a5464` | a cadeia do pen-down (alca / pick de canvas) vira um ramo, e o hit viaja num `AlvoDoClique` (on_mouse_input 2326) | `input_dispatch.rs +20/-27` · `input_dispatch/despacho_clique_gizmo.rs +67/-0` |
| `ccc60f80e` | o pivo, a ancora de junta e a roldana saem para um ramo -- e dois gates deixam de medir o ENDERECO (on_mouse_input 2165) | `editor-core it/the_rail_names_a_consumer_for_every_chip.rs +1/-1` · `input_dispatch.rs +1/-162` · `input_dispatch/despacho_clique_gizmo.rs +175/-0` · `it/joint_anchor_gizmo.rs +3/-1` |
| `5da066316` | o bloco do gizmo do `on_mouse_input` inteiro vira ramos -- o indice fica com o guarda e dois braços (on_mouse_input 2004) | `input_dispatch.rs +2/-163` · `input_dispatch/despacho_clique_gizmo.rs +170/-0` |
| `757a8ebb0` | os reclamantes antes do gizmo (Fill, modais, conta-gotas, protecção, curvas, Grid Stamp, pincel) viram um ramo (on_mouse_input 1856) | `input_dispatch.rs +8/-156` · `input_dispatch/despacho_clique_reclamantes.rs +170/-0` |
| `b5851ac9f` | duas janelas deixam de depender da COLUNA -- o fim delas esticava em silencio ao mudar de casa | `it/the_node_press_freezes_a_live_shape_recipe.rs +8/-2` · `it/the_pencil_owns_its_whole_gesture.rs +10/-3` |
| `5bf3a8fa3` | o botao direito da ferramenta vetorial sai para um ramo sem sinal (on_mouse_input 1806) | `input_dispatch.rs +3/-51` · `input_dispatch/despacho_clique_vetor.rs +67/-0` |
| `0a947fb95` | o release da caneta e da forma sai para um ramo (on_mouse_input 1721) | `input_dispatch.rs +3/-86` · `input_dispatch/despacho_clique_vetor_solto.rs +95/-0` |
| `8ed678ffb` | os gestos que se fecham no Up da ferramenta vetorial saem para um ramo (on_mouse_input 1599) | `input_dispatch.rs +1/-123` · `input_dispatch/despacho_clique_vetor_solto.rs +125/-0` |
| `d4efffef6` | o osso que nasce do arrasto sai para um ramo (on_mouse_input 1522) | `input_dispatch.rs +1/-78` · `input_dispatch/despacho_clique_vetor_solto.rs +79/-0` |
| `b6e019722` | a caneta e a forma do press vetorial saem para um ramo (on_mouse_input 1364) | `input_dispatch.rs +3/-159` · `input_dispatch/despacho_clique_vetor_premido.rs +175/-0` |
| `5e780f0fa` | as quinas (Fillet/Chamfer) do press vetorial saem para um ramo (on_mouse_input 1286) | `input_dispatch.rs +1/-79` · `input_dispatch/despacho_clique_vetor_premido.rs +74/-0` |
| `f03b6aaef` | o Trim, o Balde e o Osso do press vetorial saem para um ramo (on_mouse_input 1163) | `input_dispatch.rs +1/-124` · `input_dispatch/despacho_clique_vetor_premido.rs +128/-0` |
| `1eeffea82` | o topo da prioridade do press vetorial sai para um ramo (on_mouse_input 1019) | `input_dispatch.rs +1/-145` · `input_dispatch/despacho_clique_vetor_premido.rs +152/-0` |
| `9c707daa9` | o Shift da ferramenta vetorial sai para um ramo sem sinal (on_mouse_input 978) | `input_dispatch.rs +1/-42` · `input_dispatch/despacho_clique_vetor.rs +45/-0` |
| `c5cc8b4d7` | o bloco da ferramenta vetorial do `on_mouse_input` inteiro vira ramos -- o indice fica com uma chamada (on_mouse_input 906) | `input_dispatch.rs +2/-74` · `input_dispatch/despacho_clique_vetor.rs +88/-0` |
| `f218bd5ca` | os Up que fecham as alcas independentes de modo saem para um ramo (on_mouse_input 853) | `input_dispatch.rs +3/-54` · `input_dispatch/despacho_clique_select.rs +78/-0` |
| `e5d3171cf` | os picks modais da fisica, o desenho de junta e as alcas do texto e das fichas saem para um ramo (on_mouse_input 724) | `input_dispatch.rs +1/-130` · `input_dispatch/despacho_clique_select.rs +144/-0` |
| `e8d5f8acd` | o Set Center, o duplo-clique no texto e os picks modais do Select e do osso saem para um ramo (on_mouse_input 608) | `input_dispatch.rs +1/-117` · `input_dispatch/despacho_clique_select.rs +132/-0` |
| `0ab0ab6de` | os press do Flip e dos gizmos de pose, seleccao, warp e field saem para um ramo (on_mouse_input 465) | `input_dispatch.rs +3/-144` · `input_dispatch/despacho_clique_flip.rs +169/-0` |
| `750f8de49` | a preview e os Up que fecham gestos em curso saem para um ramo (on_mouse_input 329) | `input_dispatch.rs +1/-137` · `input_dispatch/despacho_clique_flip.rs +151/-0` |
| `179bc619a` | o editor de audio sai para um ramo com o mesmo cfg na chamada (on_mouse_input 262) | `input_dispatch.rs +4/-69` · `input_dispatch/despacho_clique_prologo.rs +88/-0` |
| `03be3d27c` | os marcadores da lente passam a ASCII -- um byte de 3 do `⟧` caia no corte de 700 de um gate | `it/input_text.rs +9/-9` |
| `95b41e8a8` | a navegacao 3D e a alca do gizmo de ancora saem para um ramo (on_mouse_input 222) | `input_dispatch.rs +1/-41` · `input_dispatch/despacho_clique_prologo.rs +53/-0` |
| `f56053510` | o aperto no canvas que solta o teclado do painel sai para um ramo sem sinal (on_mouse_input <= 200, saiu do tecto) | `input_dispatch.rs +1/-29` · `input_dispatch/despacho_clique_prologo.rs +33/-0` |
| `96edd436b` | o arrasto da biblioteca sai para um ramo sem sinal (on_mouse_input <= 200, saiu do tecto) | `input_dispatch.rs +1/-39` · `input_dispatch/despacho_clique_prologo.rs +43/-0` |
| `bb5b1baa4` | os arrastos vetoriais do movimento do cursor saem para um ramo (on_cursor_moved 302) | `input_dispatch.rs +3/-74` · `input_dispatch/despacho_mover.rs +90/-0` |
| `10c860935` | os arrastos de pintura, do Flip, dos gizmos de no e dos cartoes saem para um ramo (on_cursor_moved 214) | `input_dispatch.rs +1/-89` · `input_dispatch/despacho_mover.rs +97/-0` |
| `ec15a6fd9` | o topo dos arrastos do movimento do cursor sai para um ramo -- o on_cursor_moved fica com a cabeca, o pan e o encaminhamento (on_cursor_moved <= 200, saiu do tecto) | `input_dispatch.rs +1/-97` · `input_dispatch/despacho_mover.rs +107/-0` |
| `6652f0bb3` | a cauda das cadeias do teclado sai para um ramo (key_input 480) | `input_dispatch/keyboard.rs +5/-65` · `input_dispatch/keyboard_cadeia.rs +92/-0` |
| `ac4fbfea6` | a seleccao de nos, o nudge e os acordes do teclado saem para um ramo (key_input 324) | `input_dispatch/keyboard.rs +1/-157` · `input_dispatch/keyboard_cadeia.rs +170/-0` |
| `ffde7586d` | o texto vetorial, o Delete do Flip e as teclas nuas do vetor saem para um ramo (key_input 224) | `input_dispatch/keyboard.rs +1/-101` · `input_dispatch/keyboard_cadeia.rs +115/-0` |
| `fb01635a4` | as teclas 3D saem para um ramo -- o key_input fica com a cabeca, o observador e o encaminhamento (key_input <= 200, saiu do tecto) | `input_dispatch/keyboard.rs +1/-41` · `input_dispatch/keyboard_cadeia.rs +53/-0` |
| `ffa54b911` | a vista e o transporte do handle_editor_key saem para um ramo pelo braco _ (handle_editor_key 259) | `input_dispatch/handlers_teclas_editor.rs +125/-0` · `input_handlers.rs +5/-100` |
| `32aa8d50f` | os paineis e o desfazer do handle_editor_key saem para um ramo -- o audio_clipboard vai com os bracos que o leem (handle_editor_key <= 200, saiu do tecto) | `input_dispatch/handlers_teclas_editor.rs +113/-0` · `input_handlers.rs +1/-98` |
| `34f5c2064` | o braco de grupo da escrita do gizmo sai para um ramo (advance_gizmo_drag 605) | `input_dispatch/gizmo_drag.rs +5/-104` · `input_dispatch/gizmo_drag_escrita.rs +124/-0` |
| `67abf2642` | a cadeia de escrita do gizmo sai para um ramo, e os valores viajam num EscritaDoGizmo (advance_gizmo_drag 478) | `input_dispatch/gizmo_drag.rs +12/-139` · `input_dispatch/gizmo_drag_escrita.rs +170/-0` |
| `5ffb023e4` | os factores e a rotacao continua do gizmo saem para um ramo (advance_gizmo_drag 346) | `input_dispatch/gizmo_drag.rs +5/-133` · `input_dispatch/gizmo_drag_calculo.rs +152/-0` |
| `337a52704` | a camera, os modificadores e o snap que dao o new_t do gizmo saem para um ramo (advance_gizmo_drag <= 200, saiu do tecto) | `input_dispatch/gizmo_drag.rs +1/-149` · `input_dispatch/gizmo_drag_calculo.rs +156/-0` |
| `a9c84177d` | o MovePivot do gizmo sai para um ramo -- o advance_gizmo_drag fica com o preludio e os dois bracos (advance_gizmo_drag <= 200, saiu do tecto) | `input_dispatch/gizmo_drag.rs +7/-63` · `input_dispatch/gizmo_drag_calculo.rs +72/-0` |
| `8e782e68f` | o fim de um metodo na lente e o primeiro irmao em qualquer visibilidade -- tres janelas deixam de esticar quando o indice for cortado | `it/input_text.rs +26/-0` · `it/the_pencil_owns_its_whole_gesture.rs +4/-1` · `it/the_preview_owns_the_pointer_and_the_undo.rs +4/-5` · `it/um_aperto_no_canvas_larga_o_teclado_do_painel.rs +3/-4` |
| `5f28c9325` | os testes do eixo espectral saem do indice para um ficheiro, com o mesmo nome (input_dispatch.rs 3887) | `input_dispatch.rs +2/-49` · `input_dispatch/despacho_testes_espectro.rs +51/-0` |
| `aa1890555` | os testes da seta de redimensionar saem do indice para um ficheiro, com o mesmo nome (input_dispatch.rs 3843) | `input_dispatch.rs +2/-46` · `input_dispatch/despacho_testes_cursor.rs +47/-0` |
| `afe951e43` | o modulo de testes do despacho sai do indice para um ficheiro, com o mesmo nome (input_dispatch.rs 3385) | `input_dispatch.rs +2/-460` · `input_dispatch/despacho_testes.rs +462/-0` |
| `9f0a70932` | os metodos de picks e arrastos saem do indice para um impl irmao (input_dispatch.rs 2826) | `input_dispatch.rs +2/-561` · `input_dispatch/despacho_metodos_picks_e_arrastos.rs +569/-0` |
| `6120a7e90` | os metodos de modos e alcas saem do indice para um impl irmao (input_dispatch.rs 2280) | `input_dispatch.rs +2/-548` · `input_dispatch/despacho_metodos_modos_e_alcas.rs +553/-0` |
| `4b6294c6f` | os metodos de janela e vetor saem do indice para um impl irmao (input_dispatch.rs 1757) | `input_dispatch.rs +2/-525` · `input_dispatch/despacho_metodos_janela_e_vetor.rs +533/-0` |
| `e5782d469` | o select_wheel_at sai do indice para um ficheiro proprio, porque ao lado do ramo que o chama nao cabe (input_dispatch.rs 1724) | `input_dispatch.rs +3/-36` · `input_dispatch/despacho_clique_roldana.rs +39/-0` |
| `62be688b9` | o freq_at_y vai para junto dos metodos de audio que o leem (input_dispatch.rs 1715) | `input_dispatch.rs +3/-12` · `input_dispatch/despacho_metodos_picks_e_arrastos.rs +12/-0` |
| `768fc7224` | o resize_cursor_for_edges vai para junto do metodo que o le (input_dispatch.rs 1700) | `input_dispatch.rs +3/-18` · `input_dispatch/despacho_metodos_janela_e_vetor.rs +18/-0` |
| `f3dd033cd` | as funcoes livres de alinhar, distribuir, forma e transformacao saem do indice (input_dispatch.rs 1327) | `input_dispatch.rs +10/-383` · `input_dispatch/despacho_vetor_alinhar_e_forma.rs +388/-0` |
| `037b8ef2f` | as funcoes livres de tinta e gradiente saem do indice (input_dispatch.rs 951) | `input_dispatch.rs +7/-383` · `input_dispatch/despacho_vetor_gradiente.rs +387/-0` |
| `4feb39227` | as funcoes livres da booleana, vertices, duplicar, ordem, espelho, rotacao e fecho saem do indice (input_dispatch.rs 549) | `input_dispatch.rs +9/-411` · `input_dispatch/despacho_vetor_ops.rs +414/-0` |
| `864954446` | o cabecalho do indice deixa de citar o tecto numerado que ja nao tem | `input_dispatch.rs +5/-9` |
| `6f23ad774` | o ramo que calcula escala e rotacao do gizmo chama-se `ramo_gizmo_escala_e_rotacao` -- o typos lia o nome antigo | `input_dispatch/gizmo_drag_calculo.rs +2/-2` |
| `527b42e1a` | a prosa que esta linha escreveu encolhe -- o delta da shell media +1 253 contra a quota de +1 200 | 20 ficheiros (cabeçalhos `//!`, os `///` dos `mod despacho_*`, docs da lente, três comentários de gate) |
| `b01e10aec` | a ordem do conta-gotas de corda mede-se no CLIQUE emendado -- o `\|\|` antigo passava por construção (auditoria) | `it/pulley_wheel_handles.rs` |
| `27b435a4a` | o comentário do `use` do `freq_at_y` volta a estar em cima do item dele (auditoria) | `input_dispatch.rs` |
| `c51a055c8` | o `fim_do_item` na forma do `rustfmt` -- o `cargo fmt --check` do fecho reprovou | `it/input_text.rs` |

⚠️ Os hashes `e5782d469` · `62be688b9` · `768fc7224` são os REPARADOS da c07–c09 (§7): os originais
(`5da344d2b` · `6bbcbf443` · `3c68adcef`) ficaram só no reflog desta worktree e não são para usar.

## §7 · Construído, medido e REVERTIDO

- **O ramo que declarava o módulo no `input_dispatch.rs`** (K6): o `mod` novo levou o índice de 3 934 para 3 936 e o
  tecto de ficheiro reprovou ⇒ o módulo passou a ser declarado no PAI por `#[path]` (`keyboard.rs`), e os dois do
  gizmo também.
- **Os marcadores Unicode da lente** (`⟦ ⟧`): um corte de 700 bytes de um gate caiu no meio do `⟧` e PANICOU ⇒ ASCII.
- **`use super::*` em ficheiros novos que não o precisam** (G1, M5, c05): `unused import` ⇒ tirado; `pre_super.py`
  decide antes do corte quais o precisam — e ele erra num sentido só, o conservador: casa a PALAVRA e não o
  caminho, e deu dois falsos positivos (c10 `winit::keyboard::ModifiersState` lido como o `mod keyboard`; c12
  `ph2d_host::WindowSize` lido como o `use` do índice), os dois apanhados pelo check limpo antes de qualquer commit.
- **O `select_wheel_at` ao lado do único ramo que o chama** (c07, 1.ª redacção): MEDIDO, o
  `despacho_clique_gizmo.rs` passava de 582 para 618 linhas e o `shell_files_respect_hr18_loc_cap` reprovou ⇒ a
  peça foi desfeita inteira (`git checkout` dos três ficheiros) e refeita com a função num ficheiro próprio,
  `despacho_clique_roldana.rs`.
- **Os `use` que devolvem ao índice uma função que só os TESTES leem por `super::`** (c08 `freq_at_y`, c09
  `resize_cursor_for_edges`): com a função no mesmo ficheiro do chamador, o `use` no índice seria `unused import` no
  build do BINÁRIO ⇒ `#[cfg(test)]` (e `all(test, feature = "panel-audio-editor")` no do áudio), decidido ANTES de
  correr por `sonda_reexport.py` (cada nome devolvido, contado em código não-teste fora do bloco dele).
- ⛔ **O commit da c07 nasceu SEM o ficheiro novo** (o destino mudou na spec e não no caminho que o ciclo passa ao
  `git commit`): o índice declarava `mod despacho_clique_roldana;` e a árvore do commit não o tinha ⇒ três commits
  não compilavam sozinhos. Reparado ANTES de seguir: `git reset --keep` para a c07, `--amend` só com o ficheiro,
  `cherry-pick` da c08 e da c09; a árvore final difere da c09 antiga EXACTAMENTE nesse ficheiro (`git diff --stat`),
  e cada árvore reparada é a mesma que a suíte testou no ciclo dela. ⇒ *um caminho de commit que não sai da spec é
  uma segunda resposta à pergunta «para onde vai?», e foi ela que envelheceu.*
- **A prova de movimento reprovou a c10 com o corpo intacto**: o `pub(super) ` a mais passou uma assinatura da
  largura do `rustfmt`, que a partiu em linhas com VÍRGULA FINAL, e a régua era só «módulo espaço em branco» ⇒ a
  régua passa a ler `,)` `,]` `,}` como `)` `]` `}` (a mesma tolerância da `verbatim.py` da `line/render-loop`), e a
  peça foi desfeita e corrida de novo.
- ⛔ **A QUOTA estourou no portão do fim, e só ele a mediu:** `find shells/desktop -name "*.rs" … | wc -l` deu
  194 458 contra 193 205 na base = **+1 253**, acima da quota de +1 200. O custo era todo de linhas NOVAS (o que muda
  de casa soma zero): a lente (475), assinaturas e embrulhos dos ramos, as duas structs de contexto e a PROSA. ⇒ o
  corte foi SÓ na prosa que esta linha escreveu (cabeçalhos `//!` de 13 ficheiros de ramos, 6–9 linhas cada, que
  repetiam a mesma frase; os `///` dos 18 `mod despacho_*` que repetiam o `//!` de cada ficheiro; os docs da lente;
  três comentários de janela) — nenhum corpo mudado, nenhuma asserção. Depois: **194 357 = +1 152**, e **194 363 = +1 158** no HEAD final (as curas da auditoria e o `rustfmt`; folga 42). ⚠️ *Um delta que só se
  mede no fim não avisa a meio: esta linha devia tê-lo corrido depois de cada bloco de peças.*
- **O `typos` reprovou um NOME:** o nome antigo do ramo da escala e rotação partia-se numa palavra que o `typos` lê
  como erro de inglês ⇒ `ramo_gizmo_escala_e_rotacao` (dois sítios; a lente acha os ramos pelo prefixo, e nenhum gate
  os nomeia).
- **O `cargo fmt --check` — que nenhum portão do bloco §3 corre — reprovou a lente:** a lista das cinco fronteiras do
  `fim_do_item` escrita numa linha; o `rustfmt` quere-a vertical ⇒ `c51a055c8`. Sem ele o `ship.sh` do integrador
  reprovaria.

## §8 · As premissas do bloco de abertura que a medição derrubou (e as que confirmou)

- ⛔ **«15 testes fazem `include_str!("../../src/input_dispatch.rs")`»** — o número que decidia o trabalho era
  outro: **52** módulos de gate liam como TEXTO o despacho, o teclado, os handlers ou o gizmo (re-apontados de uma
  vez em `47069ce3a`), mais UM do lado das famílias (`the_rail…`, §5.2) e TRÊS janelas que só o corte faria
  esticar (§5.2). As categorias do bloco (15 · 7 · 33 · ~20) sobrepõem-se e não se somam.
- ⛔ **«Mova o bloco do gizmo INTEIRO para um método; corte o interior depois, passando `gfx`/`hero` e os campos
  disjuntos como PARÂMETROS»** — não foi preciso nenhum `&mut` de `gfx`/`hero` por parâmetro: o bloco cortou-se de
  DENTRO para fora, cada ramo re-empresta `self.gfx`/`hero_screen` com o mesmo `if let`, e a chamada fica depois do
  último uso do empréstimo de fora (NLL). Só atravessam valores `Copy` (`AlvoDoClique`, 8 campos).
- ⛔ **«Gates de FAMÍLIA que leem a shell pelo caminho moram FORA dela (três reprovaram na ponta da
  `render-loop`)»** — aqui, MEDIDO com `git grep "shells/desktop/src" -- crates tools`: UM precisou de mudar (o
  `ReadBy` do `the_rail…`); o `ph2d-app-field3d::shape_palette_tests` lê o `keyboard_field3d.rs`, que esta linha não
  tocou.
- ✅ **«`keyboard.rs` tem `#[path]` para `keyboard_tail.rs`»** — confirmado, e o ramo novo do teclado seguiu o mesmo
  molde (`keyboard_cadeia.rs`, declarado no PAI): um `mod` no `input_dispatch.rs` levou o índice acima da entrada
  dele e o tecto de ficheiro reprovou (§7).
- ✅ **«o `the_key_blocks_ask_whether_the_keys_are_live` varre `keyboard*.rs` por prefixo»** — confirmado; o
  `keyboard_cadeia.rs` mantém o prefixo e fica na varredura. O `handlers_teclas_editor.rs` nunca esteve nessa família
  (era o `input_handlers.rs`).
- ✅ **«a soltura e o blur TÊM de ficar antes do 1.º `return`»** — mantido: saem em ramos SEM sinal, no mesmo sítio, e
  `the_grab_is_wired_to_the_pointer` / `um_aperto_no_canvas_larga_o_teclado_do_painel` medem-no no texto emendado.
- ✅ **«`the_node_ops_are_wired` diz que o `on_mouse_input` é o ÚLTIMO `fn` do `impl`»** — mantido: o
  `#[cfg(test)] #[path] mod tests;` fica logo a seguir ao `impl`, que é a fronteira que o gate usa.
- ⚠️ **Os tamanhos do bloco** foram medidos em `4b170862b`; na base desta linha (`29ff6576e`) o `input_dispatch.rs`
  tinha **7 115** linhas (o bloco dizia 7 117), o `keyboard.rs` 598 e o `input_handlers.rs` 523.

## §9 · O fim da linha (corrido nesta árvore)

Corridos três vezes pelo `fecho.sh` (fora da árvore), a última sobre `27b435a4a` (depois das curas da auditoria); o
único commit posterior, `c51a055c8`, é só `rustfmt`, e levou check, suíte e `fmt --check` próprios. Cada saída
INTEIRA em `target/prova/*.log`, e o veredito contado (nunca um `tail`). Load: 3,4–6,5.

| portão (bloco §3, «O FIM DA LINHA») | resultado |
|---|---|
| `cargo nextest list --workspace --cargo-profile ci-test` + `nextest-list-diff.py` | antes 22 723 · depois 22 731 · **ONLY-A 0** · MOVED 0 · **ONLY-B 8** = os 8 autotestes da lente (§4), nomeados |
| `cargo test -p ph2d-host-desktop --test it` À PARTE | **811 passed · 0 failed** · 6 ignored |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | saiu 0 · **0** linhas `warning`/`error` |
| `cargo check --workspace --all-targets` | saiu 0 · **0** avisos |
| `bash scripts/check-standalone-optional.sh` | saiu 0 · 10 crates com dependência interna opcional, todas ✓ |
| `bash scripts/check-workflow-packages.sh` | saiu 0 · 32 nomes citados contra 359 membros |
| `cargo machete` | nenhuma dependência sem uso |
| `typos --force-exclude` nos 87 ficheiros tocados (+ este handoff) | **0** (a 1.ª corrida acusou o nome antigo de um ramo, §7) |
| delta de linhas da shell contra `target/prova/shell_antes.txt` | 193 205 → 194 363 = **+1 158** ≤ 1 200 (a 1.ª corrida deu +1 253, §7) |
| `cargo fmt -p ph2d-host-desktop --check` + `rustfmt --check` nos 86 `.rs` tocados (é do `ship.sh`, fora do bloco §3) | **0** diferenças (a 1.ª acusou o `fim_do_item` da lente → `c51a055c8`) |
| suíte da shell (bins + it, `ci-test`) a cada commit | 2 234 / 2 234 no último |
| auditoria ≥ 2 lentes sobre o diff acumulado | duas lentes INDEPENDENTES (subagentes só-leitura, sem `cargo`), fixadas no `864954446`: **nenhuma mudança de comportamento**; 2 achados curados (§9.1) |

### §9.1 · A auditoria

- **Lente 1 — CORRECÇÃO** (cada função partida reconstituída, ramos emendados, e comparada token a token com a
  base). 27 ramos com sinal: 164 `return;` → `return true;`, zero `return false` novos, todos acabam em `false`.
  17 sem sinal verificados caminho a caminho (13 sem `return` de topo; os dois do vetor seguidos de `return true;`;
  os dois do `handle_editor_key` só com o `else { return; }` de um `None` que o chamador já devolvia). Nenhum
  parâmetro `mut`, nenhum valor por cópia lido depois de uma reatribuição; o `audio_clipboard` lido no braço `_` é
  `&self` sem efeitos e só X/C/V o leem. 137 de 138 funções de fora dos handlers token-idênticas (a outra só ganhou
  a vírgula do `rustfmt`), 48 visibilidades todas privado → `pub(super)`, zero atributos fora do sítio, todo o
  caminho `crate::input_dispatch::…` resolve. ⇒ **achado:** o comentário do `use` de teste do `freq_at_y` ficou por
  cima do `use` da roldana (o `rustfmt` ordenou os `use` por baixo dele) — curado com um grupo próprio.
- **Lente 2 — COSTURA/ORDEM** (a sequência de consumidores de cada handler, base contra HEAD emendado).
  `on_cursor_moved` 1 447 = 1 447 tokens, `on_mouse_wheel` 420 = 420, `key_input` 1 863 = 1 863; o `on_mouse_input`
  e o `advance_gizmo_drag` diferem só em vírgulas/chavetas do `rustfmt`, no `cfg` duplicado do áudio, e nos guardas
  repetidos + as structs de contexto (seguros: cada chamada aninhada é o último statement, e o único
  `self.gfx = None` do território é o `on_close_request`). As solturas e o blur antes do 1.º `return` (tokens 32–175
  contra 236). 44 ramos, cada um chamado UMA vez, zero órfãos. As contagens de `.clone()` · `collect` · `format!` ·
  `to_vec` · `vec!` · `to_string` · `Vec::new` · `Box::new` iguais na base e no HEAD. ⇒ **achado:** o
  `pulley_wheel_handles` media endereços (§5.2) — curado com prova de mutação.
- ⚠️ **O que as duas lentes disseram que FALTA e fica para o integrador:** nenhum gate VERSIONADO prova que o
  despacho emendado é o da base (a lente só serve os gates de ordem); a prova vive em `.cauda-input-dispatch/`
  (`ramo.py`, `mover.py`) e nos scripts das lentes no scratchpad — o bloco §6.3 já manda versionar os instrumentos
  em `scripts/` depois das fusões.

⚠️ **O que só a árvore COMBINADA pode reprovar:** as duas listas numeradas e o `the_highlight_has_one_source.rs`
são editados pelas três linhas da refatoração; e o `TETO_LOC` soma os três deltas (esta linha: +1 158 sobre a base
193 205).

## §10 · A superfície de colisão (`collision-surface.sh`, colado)

⚠️ Referência, não evidência: o integrador RE-CORRE-o antes de fundir (DIRETRIZ §1.5.9). Corrido do primário sobre
esta worktree a 2026-09-13, com o HEAD `864954446`:

```
SUPERFÍCIE DE COLISÃO — line/input-dispatch contra main
  merge-base 29ff6576e   ·   62 commit(s)   ·   86 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        128   (base: 128)
      └ tripla do gate               (128, 13, 22)   (base: (128, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  86   (base: 86)
    ph2d-script (espelho)                  86   (base: 86)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0169   próximo livre: 0170
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
    nenhum arquivo da linha passa do teto
```

⚠️ **O que ela NÃO mostra e é a colisão real desta linha:** as duas listas numeradas (`fn_loc_caps.rs` e
`file_loc_caps.rs`) e o `the_highlight_has_one_source.rs` são editados pelas TRÊS linhas da refatoração — conflito
TEXTUAL, resolvido lendo o valor medido na árvore combinada (bloco §6.1 e §6.4).

## §11 · O smoke (os gestos, em CADA ferramenta)

O smoke é repetir o que o dono já usa e ver TUDO IGUAL — esta linha não tem nada novo para ver. Comando:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-input-dispatch && cargo run -p ph2d-host-desktop --profile smoke
```

Em cada linha: **clicar · arrastar · rodar a roda · as teclas**. O que tem de acontecer é o de sempre; o sinal de
defeito é um gesto que deixa de responder, responde duas vezes, ou é «comido» por outra ferramenta (a ORDEM entre
consumidores é o que estas peças podiam partir).

| onde | clicar | arrastar | roda | teclas |
|---|---|---|---|---|
| **Vetor — Select** | escolher forma; Shift soma; clicar no vazio limpa | mover a forma; gizmo: escalar pelos cantos, rodar, mover o pivô; caixa de seleção | zoom no canvas | setas (nudge), Delete, Ctrl+C/X/V, Ctrl+D, Ctrl+G / Ctrl+Shift+G, Esc |
| **Vetor — Node** | escolher nó; clicar no vazio abre o rectângulo | arrastar nó e alças; rectângulo de nós (Shift soma) | zoom | Delete nó, Esc |
| **Vetor — Pen / Pencil / Shape** | plantar âncoras; fechar no 1.º ponto | lápis desenha a mão livre; forma nasce do arrasto (Shift/Alt restringem) | zoom | Enter/Esc terminam; botão DIREITO aborta o traço |
| **Vetor — Width / Trim / Balde / Cut / Frame / Connect / Quinas / Osso** | Width: clique solto não deixa parada; direito apaga parada · Trim/Balde: clique aplica o realce · Osso: cria e agarra alças | Width/quinas/osso/moldura: arrastar as alças | zoom | Esc |
| **Texto no caminho / padrão no caminho / motion path** | a alça e as fichas agarram antes da forma de trás; direito na âncora abre o menu | arrastar a alça, as fichas, a âncora e as tangentes; duplo-clique na curva insere ponto | zoom | Ctrl+Z desfaz o arrasto |
| **Flip** | pintar um traço; a tira de quadros | traço contínuo; arrastar quadros | zoom | Delete do quadro, atalhos do Flip |
| **Esqueleto** | pick do alvo (Smart Bone) consome o clique; pinos | arrastar osso / pose / FK / IK | zoom | Esc desarma o pick |
| **Física** (`PH2D_PHYSICS_SMOKE`, qualquer cena com corpos) | Play e clicar num corpo (mão); explosão/atração no vazio; roldana seleciona-se clicando nela | arrastar o corpo em Play; âncora de junta; desenhar junta | zoom | Esc |
| **3D (escultura e modelador)** | pen-down esculpe; chips do painel e voltar ao barro | traço; órbita/pan da câmera; gizmo 3D | zoom da câmera | Delete, Ctrl+Z / Ctrl+Shift+Z (depois de tocar num chip numérico, TÊM de continuar vivos) |
| **Áudio** (editor) | cursor no tempo; caixa espectral | arrastar seleção / scrub | zoom da onda | copiar/colar do áudio com o rato sobre ele |
| **Pintura (Painter)** | pincel, conta-gotas, protecção, Grid Stamp, curvas | traço; arrastar o falloff | tamanho/zoom | Delete da âncora → figura → falloff |
| **Hierarquia** | escolher linha; renomear | reordenar/reparentar arrastando | rolar a lista | Delete, Ctrl+D, Ctrl+Z |
| **Painéis e chrome** | botões, menus, abas; a paleta de nós (`A`) | bordas das colunas; janelas flutuantes (Input Map) | rolar painéis | Esc fecha menus; teclas do painel focado não vazam para o canvas |
| **Canvas geral** | botão do MEIO faz pan; clique fora de menu fecha-o | pan com o botão do meio; arrastar da biblioteca para o canvas | zoom | Ctrl+S / Ctrl+O, Ctrl+Z / Ctrl+Y |

**O smoke fica compilado (regra I):** `rm -rf target/*/incremental` (19 GB do `debug/incremental`) e depois
`cargo build -p ph2d-host-desktop --profile smoke` duas vezes — a 1.ª, fria, em 55 s; e, depois das duas curas da
auditoria, a 2.ª corrida sobre a árvore final:

```
    Finished `smoke` profile [optimized] target(s) in 0.19s

real	0m0,246s
```
