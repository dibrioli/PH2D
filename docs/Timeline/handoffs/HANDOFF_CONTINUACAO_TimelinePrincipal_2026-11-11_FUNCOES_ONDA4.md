# Continuação — `line/TimelinePrincipal`, plano 17 (funções novas), onda 4 (item 10, container aberto no lugar, C2) — código pronto, SEM smoke do dono

- Worktree `Worktrees/line-TimelinePrincipal` · branch `line/TimelinePrincipal` · Modo L · commits sobre o handoff da onda 3 (`94601ca058`): `51c4f24359` (Done não larga a seleção e desliga o Auto Key), `d9979a6de1` (Done pausa a agulha antes da sessão), `3f26b994b5` (modelo do item 10), `cbfeb0b565` (painel do item 10).
- Plano: [`17_plano_funcoes_novas.md`](../17_plano_funcoes_novas.md) — «Onda 4 — desenho medido» · maquete [C2](../fotos/maquete_C2.png).
  Onda anterior: [`..._FUNCOES_ONDA3.md`](HANDOFF_CONTINUACAO_TimelinePrincipal_2026-11-11_FUNCOES_ONDA3.md) — **smoke do dono OK (11/11)**, incluindo as duas correções abaixo.
- Estado: testes por nome verdes; fmt + clippy `-D warnings` limpos em `ph2d-timeline`, `ph2d-panel-timeline`, `ph2d-i18n`, `ph2d-host-desktop`. **Sem smoke do dono** desta onda.

## Respostas do dono ao smoke da onda 3 (já aplicadas)

- `51c4f24359`: o Done não larga a seleção (restauro com a marca, não pelo caminho do load) e desliga o Auto Key.
- `d9979a6de1`: o Done pausa a agulha antes da sessão — a tocar, o gesto desfazia o Done no mesmo quadro. Teste `done_while_playing_stays_done` (`animando_tests.rs`); a mutação sangra.

## Item 10 (C2) — modelo (`ph2d-timeline`)

- `src/no_lugar.rs`: `InstanceMap{container,lane,t0,t1,voltas}`, `Volta{t0,t1,u0,u1,reversed}`, `instance_map`, `NoLugarView`/`NoLugarLane`/`NoLugarStrip`; `TimelineIntent::NoLugar{instancia, intent: Box}`. Conversão: ponto = afim, duração = só escala; aplica trocando `edit_path` para o container e restaurando. `TimelineState.no_lugar`.
- `src/snapshot_lane.rs`: `lane_view` extraída de `snapshot.rs`. `nest_map.rs`: `voltas_of` (`window_of` = o caso de 1 volta).
- O snapshot publica a aberta e as irmãs da mesma lane, em tempo de cena. Cópias internas das strips sem clamp (o painel recorta à janela). Volta invertida troca as bordas in/out.
- `SetContainerLoop` embrulhado escreve o doc e não toca a agulha da cena (sem undo, como o original).
- ⚠️ `intent.rs` e `intent_apply.rs` estão em **699/700 LOC** (teto): qualquer linha nova ali precisa de corte.
- Testes `tests/it/no_lugar.rs` (6): `no_lugar_o_mapa_da_instancia_da_cada_volta` · `_o_snapshot_publica_a_aberta_e_as_irmas_da_mesma_lane` · `_mover_na_cena_cai_no_tempo_do_container_e_a_irma_acompanha` · `_o_loop_do_container_converte` · `_uma_intencao_fora_da_familia_nao_muda_nada` · `_o_undo_desfaz_a_edicao_de_dentro`.
- Mutações que sangram: tirar `/escala` → move vermelho; t sem converter → 3 vermelhos.

## Item 10 — painel (`ph2d-panel-timeline`)

- Novos: `src/no_lugar.rs`, `src/no_lugar_paint.rs`, `src/ids/timeline_no_lugar.rs`. Editados: `geom.rs` (`stack_bands` desloca as lanes de baixo; `no_lugar_bands`), `strip_drag.rs`, `loop_drag.rs`, `ruler.rs`, `stack_lane_paint.rs`, `faixas_coluna.rs`, `breadcrumb.rs`, `event.rs`, `paint.rs`, `populate.rs`, `state.rs` (`caminho()` = `(edit_path(), no_lugar())`). i18n: `panel.timeline.no_lugar.same_piece`.
- Aberto = DERIVADO de `strip_escolhida` (Arrange, cartões, fonte container); Esc / clique no vazio fecham. Lanes de dentro acertam como `TimelineHitKind::Strip` com `lane = lanes.len()+k` (sem variante nova no editor-core).
- Irmãs: corpos lisos, esmaecidas pela lane silenciada, só a dica (`a_irma_so_da_a_dica_e_nao_se_arrasta`). Moldura laranja, peso e olho nas lanes de dentro, barra do loop do container.
- Testes `tests/it/no_lugar_seam.rs` (6). Mutações que sangram: `MoveStrip` cru → (b) vermelho; sem deslocar fileiras → (a); irmãs registadas editáveis → vermelho.
- Verdes por nome: `os_cartoes` 35, `nesting_seam`, `arrange_sources_seam`, `containers_tab_seam`, `strip_paint` 12, `stack_lane_paint` 15, `every_word`, `loc`, `literal`.
- Shell: UMA linha em `render_loop/fase_timeline_view.rs`: `(self.timeline.edit_path, self.timeline.no_lugar) = ph2d_panel_timeline::state::caminho();` (duas viraram uma). LOC da shell **189 040 ≤ TETO 189 041**.

## Fotos

[41](../fotos/41_no_lugar.png) · [41 lado a lado](../fotos/41_lado_a_lado_c2.png) · [42 pausado](../fotos/42_no_lugar_pausado.png) · [42 lado a lado](../fotos/42_lado_a_lado_c2.png).

## Superfície de colisão e contadores

- Crates: `ph2d-timeline` (`no_lugar`, `snapshot_lane`, `snapshot`, `nest_map`, `intent*`, `state`, `lib`, `tests/it`), `ph2d-panel-timeline` (arquivos acima), `ph2d-i18n` (timeline). Shell: só `fase_timeline_view.rs`. Docs: plano 17, fotos.
- Contadores (DELTA): `PROJECT_SCHEMA` **sem mudança**; nenhum registo/canal novo; shell +0 líquido (no teto).
- Um merge pode partir: quem constrói `TimelineState` (campo `no_lugar`); quem casa `TimelineIntent` exaustivamente (variante `NoLugar`); testes de altura/fileiras do painel (`stack_bands` agora desloca); quem chama `state::caminho()` (agora devolve par).

## Diferenças que ficam contra a maquete (deliberadas)

- A pílula «plays a – b» reusa o badge âmbar da fileira 1 (maquete: fileira 2).
- Nosso cabeçalho tem abas Keys/Containers/Arrange, não o breadcrumb «Scene › Run & jump»: esse crumb não acende.
- O container da cena não tem faixa de loop, logo a barra de loop não aparece na foto.
- O preenchimento da janela usa o token suave do badge (mais cinza que o castanho da maquete).
- O peso de Upper body dentro mostra 1,00 (dado da cena) contra 0,6 da maquete.

## Abertos

- Smoke do dono desta onda.
- Onda 5 (itens 11–13, curvas no canvas) é o **FIM da fila**: só por ordem do dono.
- Linha NÃO integrada; nunca push.

## Smoke (do dono, por fazer)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_NEST_SMOKE=4 ./target/smoke/ph2d-host-desktop` (aba Arrange, tocando).
2. Pause. Clique o primeiro «Run & jump»: abrem-se por baixo as faixas Base (walk, jump) e Upper body (swing), em 1–3 s, dentro de uma moldura laranja. O segundo «Run & jump» (4–6 s) mostra o mesmo, esmaecido.
3. Arraste «jump» dentro: a cópia esmaecida mexe junto. Ctrl+Z devolve. Esc fecha.
   Errado se: a irmã esmaecida deixa-se arrastar; a edição de dentro cai em tempo errado (deslocada); o Ctrl+Z não desfaz; as faixas de baixo ficam cobertas.
