# Handoff de continuação — editor de animações, onda 1 (07/11)

**Estado: `no smoke`.** Onda 1 do [estudo](../estudo_editor_de_animacoes_2026-11-07.md) feita em `line/GameEngineFuncional`; as conferências estão na seção «Conferido na onda 1» dele. Comitar: este é o 16.º commit da linha (15 antes). ⚠️ A rodada anterior (aba Scene, 5 botões, pausa, tela cheia, limpeza) tem smoke OK do dono mas **não passou pelos testes de fechamento e não tem handoff**. A linha ainda deve o fechamento completo (`/pd-linha-fechar`). Integrar só por ordem do Enio.

## O que mudou

- **Tela nova** no layout da Crítica: lista à esquerda (título, amostra de cor, nome, nº de quadros, + Add, Remove, Split by rows); folha em cima com Cols/Rows no título e a faixa «looks like»; cartão de baixo com título «row 1 · 6 frames» + «600 ms total», tira, Frames/s, This frame (x), Marker (botão + sinal), Repeat · Once · Ping-pong. Sem prévia, sem ◆. `editor/layout.rs` reescrito; `paint.rs`; novos `editor/bands.rs` e `editor/paint_sheet.rs`.
- **Saíram:** o ▶ do editor (`toggle_play`, `playing()`; o ▶ do cartão faz o mesmo) e o campo de tempo por miniatura. This frame age no quadro escolhido, «—» sem quadro; a miniatura mostra `2x` só se o multiplicador ≠ 1.
- **Cor:** `AnimationTag.color: u8` no FIM, `ANIM_COLORS = 8`, escolhida em `SpriteAnimations::insert` (menos usada, menor índice); paleta `Kanban*Bold` em `bands.rs`; `InspectorAnimRow.color`. **PROJECT_SCHEMA 218→219.**
- **Faixas:** `cell_users` (aberta primeiro, lê o arrasto em curso), `shown_bands` (`BANDS_MAX=3`, resto «+N»), `hovered_names` (lista completa), `AnimDoc.painted_cells` para os gates.
- **Aba estreita:** `one_line` falso põe o marcador na fila de baixo; teste a 1200, 700 e 420 px. ⚠️ A 240 px (o teste antigo) já não se garante.
- **Once de UMA célula** agora acaba e grita On Finish (`advance_span_visiting`, guarda `lo == hi` só sem fim marcado ou com bounce). O teste antigo dizia «acaba no 1.º tique», falso: o passo dura o quadro. Agora `a_single_cell_loops_never_and_finishes_when_it_has_an_end`.
- **Testes do editor** movidos pela catraca para `shells/game-scenes/tests/it/editor_animacoes/{mod,tira,marcas,cores}.rs`; `editor::paint::tom` virou `pub`. Tecto de `ph2d-app-sprite-anim`: 8352 → 7552 (medido 7404).
- **Cura alheia:** `o_cabecalho_da_hierarquia_...` estava VERMELHO desde a limpeza 36ce5b06e4 (contava `SpriteAnimStart`, que virou campo): 7/9 → 6/8.

## Oráculo Godot 4.7.2 (`AnimatedSprite2D`, tem `LOOP_PINGPONG`)

`crates/ph2d-ecs/tests/fixtures/anim_godot/` + `tests/it/the_anim_ends_match_godot.rs`: 5 verdes, 2 `#[ignore]`. **Bate:** ordem do ping-pong (0 1 2 3 2 1 0), a ponta uma vez, Once pára no último, Once sem empate tique a tique, Once de 1 quadro.
**⛔ Recusado de propósito (decisão técnica, 07/11):** o Godot troca 1 tique depois no empate exato (100 ms/25 ms: fim 16 × 17), e a ponta do ping-pong dele depende do passo (25 ms → 150 ms; 12,5 → 125; 1 → 101); o nosso dura 100 ms em qualquer passo. `AnimationPlayer` não é comparável passo a passo. Não reabrir sem medir.

## Provas (por nome, todas verdes)

fmt ✓; clippy das crates tocadas ✓. `ph2d-ecs`: the_anim_ends_match_godot 5 (+2 ignorados), a_single_cell, cada_animacao_nasce, both_components_round_trip, markers 12. `game-scenes`: editor_animacoes 41, sprite_vazio_e_marcador 4, sprite_starts_with 5, sprite_corte_velho 7. `panel-inspector`: seam_anim 48, every_painted_id 1, o_cartao_da_animacao 20. `registry-init`: o_inspector_armado 3. `desktop`: project_schema 1, the_anchor_section 11. `render`: hero_sheet 9. `editor-core`: the_shell_only_shrinks, architecture_claude_md, toast_reading. `i18n`: chave/key/orfa.
**Mutações: 10/10 sangraram** (next_color 0, insert sem cor, BANDS_MAX 5, extra 0, aberta sem prioridade, row_cells no lugar de shown_frames, take(3) nos nomes, This frame sem quadro, painted extra 0, one_line sempre). ⚠️ A mutação da guarda de uma célula (`lo == hi`) **NÃO foi corrida**.
**Fotos:** `fotografa_cena.sh` com `PH2D_GAME_SMOKE=5 PH2D_GAME_SMOKE_FOTO=animado_editor`. Apanhou dois defeitos que os testes não viam: faixas invisíveis (tokens base a 0,45) e «+2» lido «…» (agora gate em `a_celula_empilha_ate_tres_faixas_e_o_resto_vira_mais_n`).
**Não provado:** clique na janela real. Os gestos entraram por `AnimDoc::press` com sondas temporárias, já retiradas. Suíte completa: só na integração.

## Smoke do dono (passos)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=5 target/smoke/ph2d-host-desktop`
2. Na Hierarchy, clicar em «Split Sheet». No cartão Animation do Inspector, clicar no ícone Edit Animations. Abre a aba do editor.
3. Conferir a tela: lista à esquerda, folha em cima com Cols/Rows, cartão de baixo com tira, Frames/s, This frame, Marker, Repeat · Once · Ping-pong.
4. Clicar outra animação na lista: o cartão de baixo troca e o título muda («row N · M frames»).
5. Clicar + Add, depois clicar a mesma célula da folha 3 vezes (com animações diferentes): a célula empilha até 3 faixas coloridas e mostra «+N» no resto.
Deu errado se: faixas invisíveis ou lidas como «…»; campo de tempo por miniatura ou ▶ no editor (não devem existir); aba estreita cortando o marcador.

## Aberto

- **Onda 2 (estudo §4):** arrasto numerado; a tira não rola no arrasto pela FOLHA; **⚠️ defeito real:** reordenar/apagar na tira não leva `per_frame_ms` junto (`card.rs:167`); seleção múltipla, duplicar, inverter, duração em vários quadros.
- **Onda 3:** grade com margem, células vazias, aviso, passo a passo.
- **A levar ao dono:** selecionar outro objeto com o editor aberto troca o painel; digitar 24 lê 23.8 (`frame_ms` inteiro).
- Link do CLAUDE.md §5 e o «`PH2D_GAME_SMOKE=1..5`»: do fecho da linha.

## Acabamento pedido pelo dono, depois do 1.º smoke visual (07/11)

Dono, olhando a tela ao lado da maquete: «faltou capricho». A tela passou a usar as medidas dos cartões (`ph2d_card_stack::metrics`: `FIELD_H` 28, `FIELD_RADIUS`, `FIELD_GAP`) e o corpo `TypeToken::Base`. Sem grade nem contorno na folha, fundo cinza neutro, folha alinhada ao topo e à esquerda, e os cartões abraçam a folha. A aberta é a faixa de baixo e mais grossa. A ordem só se numera durante o arrasto. Tira baixa: `THUMB_OVER_CELL` 1.25 → 0.77 revê a decisão de 25/10 («um pouco maior») por ordem da maquete; `a_miniatura_da_faixa_e_mais_baixa_que_a_celula_da_folha`. O × só aparece com o rato no quadro. O marcador mora na fila do título (`inline`), ou numa fila própria se não couber. Paleta na ordem da maquete. Região continua com `button`/`field` em `Sm`. Dependência nova: `ph2d-card-stack`.

⚠️ As mutações foram corridas ANTES deste acabamento: `one_line` já não existe (agora `inline`), e a da fila estreita não foi repetida.

Dono, 07/11: Cols e Rows descem para a fila de campos do cartão de baixo (`four`; em aba estreita, dois a dois); a folha fica só com o título.
