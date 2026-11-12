# HANDOFF CONTINUAÇÃO — line/TimelinePrincipal, onda 5 (itens 11 e 12), 12/11

Plano: [17_plano_funcoes_novas.md](../17_plano_funcoes_novas.md) (§ «Onda 5» e «13 — investigação»). Leitor: a próxima LLM.

## Reabertura da linha
- Reaberta a partir do main em 12/11: os 49 commits da linha já estavam no main (integrados como cópias). Branch resetado para o main `4f46a3573c`; o HEAD antigo `effaec9f7e` ficou no branch `backup/TimelinePrincipal-pre-reabrir-2026-11-12`. Sem rebase.

## Commits desde o main (`git log 4f46a3573c..HEAD`)
- `c47816f3da` **Done rebobina a agulha** (dono 12/11): `take_done(&mut Playhead)` pausa e rebobina; `ph2d-core` virou dependência normal de `ph2d-panel-timeline`. Mutação: sem o rebobinar, `done_tells_the_shell_once_to_turn_auto_key_off` falha.
- `241285089a` memória (o Done rebobina a agulha).
- `1f69b62c35` **item 12, provas:** medido que pintura e clique já concordavam em toda condição (checkbox, aba, painel escondido, cartões, alça recolhida, Path, palco — `pointer_button`/`pointer_moved` voltam em `no_palco` antes dos ramos do editor). Testes: `while_playing_the_painted_points_and_handles_are_what_the_click_grabs` (`ph2d-app-motion`); `the_paint_and_the_click_read_the_same_curves_checkbox` e `on_the_game_stage_the_curves_are_neither_painted_nor_reached` em `shells/desktop/tests/it/the_motion_path_is_offered_only_on_the_keys_tab.rs`. Cada um com mutação que sangrou.
- `ce0983e54d` **item 11:** a aba Keys sem pilha usa `ViewClock::Editor` = `self.playhead`; `cena_do_jogo().a_correr` = `playhead.is_playing()`, logo o Play da Timeline dava os cliques do canvas ao jogo (`FocoDoJogo::premido`) e nunca chegavam a `motion_path_anchor_down`. Cura: `Tapa.animar = timeline.animating.0` em `posse_do_jogo` (`ph2d-app-components`), como o preview do Morph. O Play da barra de cima (runtime) encerra a sessão e devolve o rato ao jogo. Teste `a_previa_da_timeline_a_tocar_nao_da_o_rato_ao_jogo` (mutação sangrou). O arrasto já usava o tempo da chave agarrada.
- `d9c91af973` **abas presas em Keys:** `apply_project_with` devolve a seleção com bits NOVOS; `selection_jumps_to_keys` comparava com os antigos → `request_keys_tab`. Com o Done passando a marca de seleção, sair de Keys encerrava a sessão → restauro → de volta a Keys. Também explica o relato de 11/11 «vários UNDO → Keys». Cura: o restauro carimba `timeline_last_selected`; gate `the_restore_is_not_a_new_selection_for_the_keys_tab_jump` em `shells/desktop/tests/it/the_undo_preserves_the_vector_selection.rs` (mutação sangrou).
- A shell ficou neutra no TETO (`the_shell_only_shrinks` verde).

## Prova de fecho
- Smoke do dono OK em 12/11: Done rebobina; itens 11 e 12; abas.
- NÃO corrido ainda: clippy das crates tocadas (`ph2d-panel-timeline`, `ph2d-app-motion`, `ph2d-app-components`, `ph2d-host-desktop`). Mutações feitas inline por item. Suíte completa é do integrador.

## Smoke
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TimelinePrincipal && PH2D_ANIMACAO_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. Aba Keys, clip «pular», marcar «Edit curves on canvas», Play, arrastar losango/alça com a agulha a tocar: arrasta.
3. Ctrl+Z desfaz um passo e não salta para Keys; desmarcar o check: nada agarra; aba Arrange: nada agarra.
4. Errado: o clique vai ao jogo (nada arrasta) ou a aba volta sozinha a Keys.

## PRÓXIMO — item 13 (o easing estraga a curva do canvas)
Achados da investigação (ainda sem reprodução):
- `Interp` em `crates/ph2d-anim/src/curve.rs:20` (Hold, Linear, Eased, Bezier, `BezierW{x1,dy1,x2,dy2}`, Nearest); `Key{t,value,interp}` em `track.rs:78`. `BezierW` = cúbica no plano (u, valor): x = fração de tempo, dy = desvio ABSOLUTO de valor (`curve_weighted.rs`).
- Canvas: `drag_alca` (`motion_key_handles.rs:233`) escreve por eixo `bezier_w(1/3, c1−v0, 2/3, c2−v1)` (:281).
- UI de easing: `presets.rs:192–202` (`SetInterp`/`SetSelectedInterp`) com `Interp` novo inteiro (`presets::absolute` :271) → `intent_apply.rs:271` `track.set_interp` substitui; arrastos de alça em `graph.rs:152/190`.
- Ideia-chave: dentro de `BezierW` o caminho espacial (X,Y) é dado só pelos pontos de controle de valor (dy); o tempo, por x1,x2. Se X e Y partilham x1,x2, mudar x1,x2 mantendo dy preserva o caminho exatamente.
- Caminhos de dano suspeitos: (a) `SetInterp` troca `BezierW` e perde dy; (b) easing num eixo só desparelha X e Y; (c) o canvas fixa x em 1/3–2/3.
- Alternativa AE já existente: `MotionPath` (`crates/ph2d-timeline/src/path.rs:129`, ADR-0141 «proposto, aguardando aceite do Enio»): Position = distância no caminho com easing próprio; `has_path` desliga as alças do canvas. `PROJECT_SCHEMA` = 237 (`shells/desktop/src/project_schema.rs:459`) se o formato mudar.
- **Decisão do dono PENDENTE (perguntada 12/11):** **A** = modelo AE (uma linha «Position» = ritmo ao longo do caminho, forma no canvas, via `MotionPath`) · **B** = manter linhas X/Y; a seta muda só o ritmo (x1,x2), aplicado a X e Y juntos, forma mantida.
- Primeiro passo, qualquer que seja a escolha: um teste que reproduza qual caminho de dano (a/b/c) acontece.
