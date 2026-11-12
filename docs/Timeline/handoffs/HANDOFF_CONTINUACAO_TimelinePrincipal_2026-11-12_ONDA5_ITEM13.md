# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, onda 5 (item 13), 12/11

Plano: [17_plano_funcoes_novas.md](../17_plano_funcoes_novas.md) (§ «13 — desenho medido e feito»). ADR: [0141](../../architecture/decisions/0141-one-2d-channel-and-separate-axes-are-a-mode.md) agora **ACEITO**, com «Emenda — item 13». Anterior: [itens 11 e 12](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-12_ONDA5_ITENS_11_12.md). Leitor: a próxima LLM.

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

## ABERTO (visto, NÃO feito)
1. O overlay do Path e `move_path_anchor`/`move_path_tangent` na shell usam coordenadas de MUNDO como se fossem locais: sob pai movido, o caminho é desenhado/arrastado fora do lugar (pré-existente; `drag_alca` já usa `local_for_world`).
2. No smoke ANIMACAO o herói fica parado no fim do clip enquanto a agulha global passa do comprimento do clip (igual na cena 1; pré-existente).
3. A conversão por toggle/menu (`set_position_mode`) usa o repouso do binding ou zero (sem mundo).
4. Integração NÃO feita.

## PRÓXIMO
O plano 17 está completo (itens 1–13). Passo seguinte: smoke do dono, depois `/pd-linha-fechar` (DIRETRIZ §1.5.9) antes de integrar. Integração só por ordem do Enio.
