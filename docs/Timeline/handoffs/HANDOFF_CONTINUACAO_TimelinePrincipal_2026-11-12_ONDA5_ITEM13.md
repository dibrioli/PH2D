# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, onda 5 (item 13), 12/11

Plano: [17_plano_funcoes_novas.md](../17_plano_funcoes_novas.md) (§ «13 — desenho medido e feito»). ADR: [0141](../../architecture/decisions/0141-timeline-position-is-one-2d-channel-and-separate-axes-are-a-mode.md) agora **ACEITO**, com «Emenda — item 13». Anterior: [itens 11 e 12](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-12_ONDA5_ITENS_11_12.md). Leitor: a próxima LLM.

## Decisão do dono (12/11)
Modelo **A** (After Effects): uma linha «Position» = ritmo ao longo do caminho (`MotionPath`); a FORMA do arco mora no caminho; o easing só muda o ritmo.

## Commit
- `5bb5a0d388` `feat(timeline): item 13 — o easing só muda o ritmo; a forma do arco mora no caminho`. Único commit de código desde o handoff dos itens 11/12 (`git log cb160faecb..HEAD` mostra também os docs `69ee96b470`).

## Reprodução (primeiro passo, feito)
- `choosing_an_easing_after_bending_the_arc_keeps_the_arc` (`ph2d-app-motion`, `motion_key_handles_tests.rs`) estava **VERMELHO** antes da cura: distância de Hausdorff do arco 0,269.
- Caminhos de dano: **(a)** `SetInterp`/`SetSelectedInterp` trocava a `BezierW` por eixo e perdia os `dy`; **(b)** easing num eixo só desparelhava X e Y. **(c)** o canvas fixar x em 1/3–2/3 NÃO danifica a forma.

## O que mudou
- `drag_alca` (`motion_key_handles.rs`) converte X/Y → Position por `separate_to_path_with_rest` (`ph2d-timeline/src/path_convert.rs`) **no mesmo passo de undo**, move a tangente da âncora (`move_path_tangent`, `doc_path.rs`) e seleciona a chave Position. O easing seguinte cai na chave Position.
- A conversão guarda a FORMA: cúbica ajustada a 4 amostras (0, 1/3, 2/3, 1); Smooth/Corner por colinearidade; Hold fica reto.
- A conversão passa por **TODO clip** do objeto, nos dois sentidos. Antes, `unbind` só removia a track do clip ativo e deixava tracks órfãs nos outros clips.
- Eixo sem track = repouso do binding; senão, a pose. `rewrite_path_key_values` passou a `pub(crate)`.
- O overlay do caminho obedece a «Edit curves on canvas»: `motion_key_points::caminho_visivel(curves)` = `curves || !cards_on()`. `motion_key_points::draw` desenha também o caminho. Novo `motion_key_points::curve_hit` (duplo-clique).
- Painel: `request_curves_on_canvas()` (`state_requests.rs`), consumido em `paint.rs`.
- Gates da shell atualizados: `the_motion_path_is_offered_only_on_the_keys_tab.rs` (DOORS; o teste do checkbox agora vê seen=3) e `the_motion_path_anchor_is_drawn_and_dragged.rs`. Também tocados: `despacho_metodos_modos_e_alcas.rs`, `animacao_smoke.rs`, `fase_canvas_overlays.rs`.
- Cena de smoke **`PH2D_ANIMACAO_SMOKE=7`** (`motion_key_handles::smoke_7`): «pular» com o arco de decolagem entortado + Cubic In-Out, checkbox marcado. Fotos: [smoke 1](../fotos/17_item13_smoke_1.png) · [smoke 7](../fotos/17_item13_smoke_7.png).

## Contadores (DELTA)
- Shell neutra (+13/−13); `the_shell_only_shrinks` verde.
- Formato intocado: `PROJECT_SCHEMA` **237**, `DOC_VERSION` **25** (nada a recontar).
- Sem tool/node/painel novo, sem contrato §6 tocado, sem ADR novo (o 0141 mudou de estado e ganhou emenda).

## Superfície de colisão (o que um merge pode partir)
- `ph2d-timeline`: `path_convert.rs` (reescrito, +355/−…), `doc_path.rs` (visibilidade), `path_convert_tests.rs`.
- `ph2d-app-motion`: `motion_key_handles.rs` (`drag_alca` reescrito), `motion_key_points.rs`, os testes.
- `ph2d-panel-timeline`: `paint.rs`, `state.rs`, `state_requests.rs`.
- Shell: `animacao_smoke.rs`, `fase_canvas_overlays.rs`, `despacho_metodos_modos_e_alcas.rs` e dois gates em `shells/desktop/tests/it/`. Outra linha que adicione cena `PH2D_ANIMACAO_SMOKE` colide no número 7 — conte, não escolha.
- `docs/Timeline/17_plano_funcoes_novas.md` e o ADR-0141 (append).

## Provas de mutação (todas sangraram, exceto a nota em M1)
- M1 `handles` a zero → vermelho em `converting_keeps_the_shape_the_canvas_bent` e `bending_one_segment_keeps_the_shape_of_the_rest`. **Sobreviveu** no teste de reprodução (compara estados pós-conversão); a prova dele é ter estado vermelho no código antigo.
- M2 só o clip ativo → vermelho `converting_carries_every_clip_of_the_object`.
- M3 `caminho_visivel = true` → vermelho `the_path_obeys_the_curves_checkbox_like_the_handles`.
- M4 sem `move_path_tangent` → vermelho `dragging_the_out_handle_...` e `under_a_moved_and_rotated_parent_...`.
- M5 repouso → 0 → vermelho `converting_carries_every_clip...`.

## Prova de fecho
- `cargo fmt --all` e `clippy -D warnings` em `ph2d-timeline`, `ph2d-app-motion`, `ph2d-panel-timeline`, `ph2d-app-components`, `ph2d-host-desktop`: limpos.
- Testes por nome, todos verdes: `ph2d-app-motion` `motion_key_handles` 10 · `motion_key_points` 6 · `motion_path` 19; `ph2d-timeline` `path_convert` 10; `ph2d-panel-timeline` seam 131; `ph2d-host-desktop` `the_motion_path` 12 · `the_shell_only_shrinks` 4 · `position_mode` 1.
- A suíte completa é do integrador; não corrida aqui.

## Smoke (dono)
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Timeline, aba Keys, chip «pular», marcar «Edit curves on canvas»; arrastar uma alça redonda do arco do herói.
3. As linhas «Position X/Y» viram uma só «Position» e o arco entorta. Botão direito na primeira chave de Position, escolher um easing: o arco **mantém a forma**, só a velocidade muda (os pontos se juntam nas pontas). Ctrl+Z desfaz o entortar num só passo. Desmarcar a caixa: caminho e alças somem e nada agarra.
4. Errado se: o arco muda de forma depois do easing, X/Y continuam separados, ou «andar» deixa de andar.
- Atalho: `PH2D_ANIMACAO_SMOKE=7` já abre entortado e com easing.

## Smoke do dono 12/11: OK, com um defeito — curado (`e2f75acee5`)

- Relato: «o objeto tem z-index maior que o ponto da curva; se tento mover o ponto, é o objeto que se move».
- Mecanismo (medido com arrasto REAL: `xdotool` na Xvfb via `docs/CodeEditor/ferramentas/smoke/cena14/fotos_q3.sh` com `PH2D_ANIMACAO_SMOKE=7 CENA=0 ARRASTA="861 520 950 420"`): a âncora do fim cai no centro do objeto, sobre o `GIZMO_PIVOT`. `over_canvas_or_gizmo` (`shells/desktop/src/vec_text_reopen.rs`) perguntava `gizmo_kind_for_id`, que exclui o pivô ⇒ `false` ⇒ `motion_path_anchor_down` nunca era chamado. Não era z-index: o desenho já estava por cima.
- Cura: a porta `ph2d_editor_core::gizmo::is_gizmo_id` (pivô incluído). Gate `the_canvas_guard_counts_the_pivot_as_part_of_the_gizmo` (`the_motion_path_is_offered_only_on_the_keys_tab.rs`), mutação sangrou. Foto depois: [ponto final arrastado](../fotos/17_item13_ponto_final_arrastado.png). Shell −1 linha; clippy da shell limpo; vizinhos por nome verdes (gizmo 71, the_motion_path 13, double_click 1, shrinks 4).

## Os 3 abertos — curados (12/11)

1. **Path sob pai movido/rodado/escalado.** Mecanismo: as âncoras/tangentes do `MotionPath` são LOCAIS (viram `Transform.translation`), mas `marks`/`anchor_screen`/`tangent_screen`/`motion_path_curve_hit` as pintavam como MUNDO e a shell escrevia o ponto de mundo direto em `move_path_anchor`/`move_path_tangent`. Cura por UMA porta: `motion_key_points::world_for_local` (local→mundo pelo pai posado no instante, `parent_world_transform_with` + `pose_at`; o inverso exato de `local_for_world`) em todo o overlay — a âncora `i` usa o pai no instante da chave `i` (`motion_path_overlay::anchor_time`), a fita/pontos o pai em cada `t`; o hit da curva projeta em local pelo pai da 1.ª chave. O arrasto de âncora/alça saiu da shell para `motion_key_points::drag_path` (via `drag_grab`, mundo→local), coerente com `drag_alca`; as funções do overlay recebem `world`. Shell: −17 linhas. Teste `motion_key_handles_tests::under_a_moved_and_rotated_parent_the_path_is_painted_grabbed_and_dragged_in_world` (pai (2,−1), rot 0,6, escala (1,5;0,75)): âncora pintada/agarrada = `world_pose_at` no tempo da chave; ponta da alça = onde o ponteiro a deixou; arrastar âncora 1 a (5,−2) põe o objeto (mundo) lá; idem a alça. Mutações: `world_for_local` devolve `p` ⇒ vermelho (l. 483, âncora fora do objeto); `drag_path` sem `local_for_world` ⇒ vermelho (l. 497, objeto não vai ao alvo). Restaurado ⇒ verde. Gate de texto `the_drag_writes_through_the_documents_single_door` passa a ler `motion_key_points.rs`. Limite: com pai ANIMADO durante o clip, o duplo-clique na curva usa o pai da 1.ª chave (desenho e âncoras são exatos por instante).
2. **Herói parado no fim do clip (`ANIMACAO_SMOKE=1|7`).** Medido (`view_clock_tests::a_loop_set_only_on_the_clock_is_lost_on_the_view_edge_and_the_documents_survives`): a cena punha o laço `(0, 2)` SÓ no `Playhead`; o `arm_view_loop` rearma o relógio do DOCUMENTO na borda de vista (entrar na aba Keys) ⇒ `loop_range() = None` e a agulha a 12 s após 12 s de corrida; o clip congela no fim (por desenho: o relógio é livre, dono 25/07). Era **defeito da CENA**, não do app (o app laça o que o documento laça). Na `=7` havia um segundo erro: o clip ativo é «pular» (1 s) sob um laço de 2 s. Cura: `ph2d_timeline::loop_whole_active_clip` (`intent_loop_sync.rs:96`) escreve o laço `[0, fim do clip ativo]` no documento (as duas vistas) e arma o relógio; a cena (`animacao_smoke.rs:167`) chama-o DEPOIS do `smoke_7` (que troca o clip ativo). Mutação: o helper só no relógio ⇒ vermelho (l. 207); restaurado ⇒ verde.
3. **Convert to Motion Path sem mundo.** Mecanismo: `set_position_mode` → `separate_to_path` com repouso `[0, 0]`: eixo sem binding (ou sem `rest` capturado) caía a 0. Cura: `ph2d_timeline::apply_intent_in(world, …)` (`intent_apply_path.rs:61`) lê o repouso do `Transform` do objeto para `ConvertPositionMode` e delega o resto a `apply_intent`; `set_position_mode(entity, to_path, repouso)` passa-o a `separate_to_path_with_rest`; o dreno da shell (`timeline_bridge.rs:172`) usa `apply_intent_in` (linha neutra). Teste `path_convert_tests::the_convert_intent_keeps_the_real_pose_of_the_axis_that_was_never_animated`: objeto em (5,3), só X animado, pose em t = 0; 0,25; 0,5; 1 igual antes/depois. Mutação: repouso `[0,0]` ⇒ vermelho (l. 397); restaurado ⇒ verde. O `apply_intent` sem mundo continua com `[0,0]` (só testes/smokes sem mundo).

Fecho: `cargo fmt --all`; clippy `-D warnings` limpo em `ph2d-app-motion`, `ph2d-timeline`, `ph2d-host-desktop`; por nome verdes: `ph2d-app-motion motion_key_handles|motion_key_points|motion_path`, `ph2d-timeline path_convert|view_clock`, `ph2d-host-desktop the_motion_path|shrinks|gizmo`; `cargo build --profile smoke -p ph2d-host-desktop` ok. Foto da prova do item 1: não tirada (sem cena com pai rodado; criar uma não coube no prazo).

## ABERTO (visto, NÃO feito)
1. Integração NÃO feita.
2. Smoke do dono dos 3 itens acima; foto do item 1 (cena com pai rodado) por fazer.

## PRÓXIMO
O plano 17 está completo (itens 1–13). Passo seguinte: smoke do dono, depois `/pd-linha-fechar` (DIRETRIZ §1.5.9) antes de integrar. Integração só por ordem do Enio.
