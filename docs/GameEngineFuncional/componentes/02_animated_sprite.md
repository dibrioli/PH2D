# Animated Sprite

**Estado: `em obra`.** Desenho aprovado pelo dono em 06/11: «OK às recomendações».

Estudos lidos: crítica «Animated Sprite», «Suas perguntas», «Lista consolidada» · estudo «AnimatedSprite», «12 ideias». Código conferido em 06/11/2026, sobre `19436b23ce`.

## Como está (conferido)

Fotos de 06/11 pelas cenas `PH2D_GAME_SMOKE=4` + `PH2D_GAME_SMOKE_FOTO=uso_animada|heroi_animado|animada` e `PH2D_ANIM_SMOKE=2`.

- **Censo medido** no painel de teste, com a lista de `campos_da_sprite()` mais os do Animation: **28 à vista, 54 no total**; Player com 6 regras, **55 e 81** (cada regra conta 4 controlos). A crítica dizia 24 e 54.
- Marcador por quadro **existe e dispara** (`ph2d-ecs/src/sprite_anim_markers.rs`; `populate_anim.rs:105-134`; `tick.rs:172,212`). Só está dentro de Advanced, como número + texto. A crítica não o achou.
- Sinais de On Finish, On Loop e marcador chegam a Signal Actions sem script, mesmo em objeto sem regras: `tick.rs:212-230` → `fase_fixed_step_clocks.rs:148` → `fase_signal_outbox.rs` → `ph2d-runtime/src/lib.rs:383` → `fase_tabela_de_accoes.rs:105`.
- Regras: nomes não cortam mais (duas fileiras por regra). When = Pose, Action, Life, Signal (`inspector_model_anim_card.rs:8-13`); `personagem.rs:149-200` decide. Nenhum destaque de qual regra ganhou.
- Who Picks: Me e Script diferem só em `autoplay` (`card.rs:72-75,223-237`). Não existe «com qual começa»: `start_autoplay_animations` (`tick.rs:242-253`) toca `animator.current`, e o ▶ da lista (`event_anim_card.rs:92-95`) o reescreve. **Logo, o ▶ muda em silêncio com que o jogo começa.**
- Trocar a imagem (`image_choice.rs:55-69,103-108`): com corte salvo a grade é refeita; sem corte a grade antiga fica; as animações ficam sempre; nenhum aviso. Lido, não corrido.
- Folha nova (`make_animated`, `image_use.rs:71-90`): uma animação «loop» com todas as células (o `heroi_azul` ficou com 32).
- Pivot: Look aparece em Animated; os 9 pontos medem a **união das células**, de propósito, para o pivô não saltar quando o quadro troca (`ph2d-sprite-edit/src/pivot.rs:51-84`). Sem pivô por quadro.
- Player com Flip escreve `flip_x` a cada quadro por cima de uma chave de Flip (`personagem.rs:188-194`).
- Add ▸ Animated Sprite: herói de fábrica, 6 tags, autoplay em idle (`object_add.rs:110-135`). Add ▸ Character: o mesmo + `CharacterAnimations`, `PlatformPlayer`.
- Editor (aba, `editor/`): lista sem reordenar; folha com clicar e arrastar, células repetidas e não contíguas; fila com reordenar, apagar, tempo por quadro; só Cols×Rows. Não tem: Split by rows, Size/Separation/Offset, marcador, ida ao Pixel/Painter. A tira não rola durante o arrasto (`mod.rs:146-158`). `propose_after_drop` (`editor/mod.rs:48`) tem dois chamadores, em `shells/desktop/src/asset_drop_apply.rs:97,188` (a nota de 06/11 errou): continua a abrir o editor numa troca que propõe outra grade; o de `:188` nunca dispara, a sprite nova é Still.
- Leitores: Hold, Repeat delay, Repeat count, Speed, Frames/s, Cycle, This frame e os sinais estão vivos. `AnimFieldEdit::Autoplay` não tem gesto. Speed negativa em Once a partir do quadro 0 acaba logo, como na Godot.
- `.ase`: tags com `frame_ms`, direção e `per_frame_ms` (`shells/desktop/src/ase_import.rs:123-198`).
- A Timeline por `SpriteAnimation` só escreve `Transform` (`ph2d-timeline/src/sprite.rs:95`). O que grava chave está em **E0**, abaixo.

**Da crítica já não vale:** «sem marcador» · regras cortadas · Direction/overrides à vista · contagem 10/29 da versão B (a Sprite fechada tem 13/31 sozinha).

### E0: o que grava chave (medido em 06/11)

Sonda temporária em `ph2d-app-components` (já retirada): `canais_componente::{campos, ler, escrever}` sobre `SpriteAnimator` + `SpriteGrid` + `SpriteAnimations` reais, e `ph2d_timeline::autokey_campos` sobre cada edição. O caminho: o pintor regista o rótulo `component.field.*` (`key_mark::paint_decorator_campo`), `marcas_inspector::do_rotulo` acha o canal, `shells/presos/src/losango.rs` grava o clique, `fase_autokey.rs` publica e `autokey_pass.rs` compara. Não corrido na janela real; o Auto Key exige o painel Timeline aberto.

| Campo | Auto Key pede chave? | A curva escreve de volta? | Losango no cartão? |
|---|---|---|---|
| Speed | sim (`speed_q16`) | sim | sim, pelo rótulo `sprite_animator.3` |
| H/V Frames | sim (`sprite_grid.1\|2`) | não medido | saíram do cartão: no editor gravam pelo Auto Key, sem losango |
| Quadro | sim (`sprite_grid.3`, só a célula; `SpriteAnimCursor` fora da sonda) | não medido | não |
| Qual animação toca | **sim** (`sprite_animator.0`, texto, por índice) | sim | não |
| Who Picks | só Auto ↔ Script (`sprite_animator.2`, `autoplay`); Rules é componente que entra e sai: não | não medido | não (isenção no gate) |
| Frames/s, Cycle | **não**: `SpriteAnimations` não tem campos no catálogo | não | não (isenção no gate) |

Logo: «qual animação toca» já grava, nada a construir. Cuidado: a curva escreve o `current` sem rebobinar, e com Auto Key armado o ▶ da lista grava chave de `current` e `playing` (efeito no quadro não medido).

## O que as 9 ensinam

Godot: fatiar vendo a folha, tempo relativo por quadro; evitar o painel que some ao trocar de nó. Construct: Speed, Loop, Ping Pong ao lado dos quadros; evitar a janela que cobre a cena. GameMaker: marcador dentro da faixa. GDevelop: convite no vazio, pontos por quadro. Unreal: evitar a lista de «N members». Nenhuma das 9, nas fontes lidas, liga o estado do personagem à animação no painel: é a nossa vantagem, e a regra acesa no Play a reforça.

## Decisões do dono

**Vigentes:** um cartão Animation + editor próprio, quadros/s (25/10) · lista de regras «quando → toca» (25/10) · seletor Use (26/10) · folha fora do cartão em Animated (27/10) · Direction e overrides só com valor, velocidade negativa toca ao contrário (04/11) · Auto Key grava tudo (19/10) · chaves de Flip e Pivot (decisões 12 e 16 da [Sprite](01_sprite.md)).

**Desta revisão (06/11), todas como recomendado:**

1. **A folha fica no editor.** H/V Frames, From, To e This frame saem do cartão.
2. **Who Picks: Auto · Script · Rules.** «Me» vira «Auto», com o menu «Starts with» ao lado (revê em parte 25/10). O ▶ passa a ser só prévia: não muda com que o jogo começa.
3. **Add ▸ Animated Sprite nasce vazio**, com «Try with the sample hero». O Character continua com o herói.
4. **Marcador só no editor:** ◆ no quadro da tira; no cartão só o resumo «2 markers».
5. **Regra acesa no Play agora; ensaio com teclas depois.**
6. **Look nasce dobrado em Animated** (nova, trazida pelo levantamento).
7. Versão B + aviso de imagem trocada (da A) + botão **Split by rows** no editor.

## O desenho aprovado

**Vazio:** cartão Image = «Drop a sprite sheet or click to choose» + recentes + «Try with the sample hero». Sem Animation.

**Em uso:**

| Cartão | Conteúdo |
|---|---|
| Image | miniatura, nome, `1536 × 768 px · 8 × 4`, botão Painter; Use; recibo «Keep old size» e aviso de imagem trocada quando couber |
| Animation (N) | ícones no cabeçalho: Add · Remove · Edit Animations; prévia; lista (miniatura, nome, N frames, ▶; dois cliques renomeiam); Playback (▶, barra, Frame n / N); Frames/s e Speed na mesma linha; Cycle; **More** dobrado; **Who Picks** |
| More (dobrado) | resumo `Hold · signals`; Hold, Repeat delay, Repeat count, On Finish, On Loop; «N markers» (abre o editor); Direction e overrides só com valor |
| Who Picks | Auto + «Starts with» (abaixo dos botões) / Script / Rules; em Rules, as regras, + Rule, Run From, Flip; no Play a regra que ganhou fica acesa |
| Look, Transform, Ordering, Advanced | os da Sprite; Look dobrado com resumo |

**Editor:** a lista · a folha · a fila. Novos: **Split by rows** (uma animação por fileira ocupada) · ◆ marcador no quadro da fila, com o nome do sinal · a tira rola ao arrastar · Cols×Rows moram só aqui. This frame já está aqui.

**Aviso de imagem trocada** (Animated, imagem sem corte salvo): `New image, old cut: 8 × 4 makes cells of 32 × 32 px, and these 6 animations were made for another image.` com **Cut again** e **Keep the cut**.

## Aceitação

- **Poder:** com qual começa, à vista, e o ▶ não o muda · marcador pelo editor reagindo em regra e Signal Actions sem script · uma folha vira uma animação por fileira em um clique · ver qual regra ganhou durante o Play · nenhum poder removido: tudo o que sai do cartão continua no editor ou em More.
- **Facilidade:** de 28 à vista e 54 no total para **15 e 47** (conta no papel, confirmar no censo novo; a Sprite fechada responde por 13 e 31 com Look aberto). Sua folha com nomes tocando, de 7 gestos para 5.
- **Intuitividade:** o vazio diz o que fazer · nenhum número de célula no cartão · trocar a imagem explica o problema e dá as duas saídas · todo texto novo em inglês.

**Critério de parada:** regra acesa que custe mais de um campo novo no estado do jogo, ou que precise tocar a física, volta ao dono antes de prosseguir.

## ⛔ Recusas

| O quê | Por quê |
|---|---|
| Folha ou tira dentro do cartão (versão C) | decisão de 27/10, mantida em 06/11 |
| Ensaio com teclas nesta rodada | caro: roda o personagem sem a cena; decisão 5 |
| Aviso de regra morta | exige saber quais condições cobrem quais; fica com o ensaio |
| Colisão, âncora e pivô por quadro | fora deste item; Physics Body e Anchors |
| Fatiar por Size/Separation/Offset | a crítica não o pede; só se o dono pedir |

## Provas

- **Censo** `o_censo_dos_campos_do_animated_em_uso`: **17 à vista e 48 no total, moldura 22**, sem regras, medido depois de E4. Meta do papel: 15 e 47. Conta os três verbos do cabeçalho e o botão dos marcadores; os campos do marcador já não contavam (a fixa não tem marcador). A diferença para a meta fica para E6.
- **Cartão** (`ph2d-panel-inspector`, `tests/it/o_cartao_da_animacao.rs`): `o_corte_e_o_nome_sairam_do_cartao` · `os_verbos_moram_no_cabecalho` · `sem_animacoes_nao_ha_remove` · `frames_por_segundo_e_speed_na_mesma_linha` · `dois_cliques_renomeiam_e_o_enter_grava` · `esc_desiste_e_clicar_fora_grava` · `a_moldura_diz_o_corte_sem_cortar_no_piso` · `o_look_nasce_dobrado_so_em_animated` · `o_menu_starts_with_abre_e_escolhe_pelo_ponteiro` · `o_play_da_linha_nao_escreve_o_inicio` · `so_o_auto_tem_o_menu`.
- **Editor** (`ph2d-app-sprite-anim`): `um_intervalo_como_o_do_ase_continua_editavel_no_editor`.
- **Início** (`ph2d-app-sprite-anim`, `inspector_tests.rs`): `o_play_da_prevista_nao_muda_com_qual_o_jogo_comeca` · `escolher_o_inicio_vale_no_jogo_e_nao_mexe_na_previa` · `o_inicio_segue_o_renomear_e_cai_na_primeira_ao_apagar` · `projeto_antigo_script_e_regras_ficam_como_estao`.
- **Foto, desfazer, abrir** (`ph2d-host-game-scenes`, `sprite_starts_with.rs`): `o_starts_with_viaja_na_foto_e_o_desfazer_repoe` · `a_foto_tirada_antes_de_comecar_guarda_a_previa` · `o_projeto_antigo_abre_igual`.
- **Limites destas provas:** as opções do menu pinta-as o hero por cima de tudo; o teste clica a caixa com o ponteiro e entrega o clique da opção como evento, como o do Ordering. O Play real da janela não foi corrido: vale a ordem fotografia → início do `undo_jogo_tests`. Falta a cena de smoke e o item em uso no jogo (E6).
- **`lei_do_ritmo`, curado:** `row_inteira` (`cards_linhas.rs:40`) soma `FIELD_ROW_GAP` à linha sem rótulo, e `caixa_do_mirror` (`cards_sprite_pivot.rs`) passava-lhe o topo da fila do Mirror: a caixa nascia 6 px abaixo, e o censo media 4 contra o segmento de `SEG_PAD`. A caixa continua à direita dos botões, agora na mesma fila (decisão 23 da Sprite).
- **Editor** (`editor/tests_marks.rs`): `a_transparencia_decide_quais_fileiras_tem_desenho` · `split_by_rows_cria_uma_por_fileira_ocupada_num_so_gesto` · `fileira_ja_coberta_nao_se_duplica` · `o_marcador_poe_nomeia_pinta_e_tira_no_quadro_escolhido` · `clicar_no_campo_renomeia_o_sinal` · `o_marcador_anda_com_o_quadro` · `um_marcador_passado_do_fim_mostra_se_no_ultimo_e_tira_se` · `a_tira_rola_com_a_mao_na_borda`.
- **Sinal, menu, desfazer** (`shells/game-scenes`, `sprite_vazio_e_marcador.rs`): `o_marcador_do_editor_chega_a_regra_e_a_tabela_sem_script` (gesto do editor → tique real → regra Signal e `resolve_signal_actions`) · `o_add_nasce_vazio_e_o_character_com_o_heroi` · `o_heroi_de_amostra_tem_um_passo_de_desfazer` · `as_saidas_do_aviso_voltam_pelo_desfazer`. O desfazer aqui é a foto de antes, a porta que o undo usa; o Ctrl+Z da janela não foi corrido.
- **Aviso e vazio** (`sprite_corte_velho.rs`): `escolher_a_imagem_no_vazio_ja_entra_em_animated` · `o_heroi_de_amostra_entra_no_vazio_e_so_no_vazio` · `trocar_por_imagem_sem_corte_acende_o_aviso_com_os_numeros` · `imagem_que_traz_corte_nao_avisa` · `as_duas_saidas_do_aviso` · `cut_again_sem_proposta_abre_o_editor`.
- **Cartão** (`o_cartao_da_animacao.rs`): `o_marcador_no_cartao_e_o_resumo_que_abre_o_editor` · `o_vazio_do_animated_pede_a_folha_e_oferece_o_heroi` · `o_aviso_de_imagem_trocada_diz_os_numeros_e_da_as_duas_saidas`. O convite é o mesmo desenho do da Sprite: não foi fotografado (E6).
- **Decisões técnicas de E3 e E4, para o smoke:** o Split junta e não apaga: o «loop» da folha inteira fica, e não conta como cobertura. Nomes `row N` em sequência, saltando os ocupados, sem olhar o número da fileira. Add ▸ Animated Sprite não abre a escolha sozinho, ao contrário do Add ▸ Sprite: o convite é o que pede, e `so_a_sprite_nasce_a_pedir_a_imagem` continua. Sem grade a propor, Cut again abre o editor. O texto com uma animação diz «this 1 animation was».
- **Catraca de `ph2d-app-sprite-anim`:** E1 e E2 já a tinham passado (8542 contra 8352); E3 e E4 chegaram a 9578. Cura por mudança: quatro ficheiros de teste foram para `shells/game-scenes` (`sprite_card_edits`, `sprite_corte_velho`, `sprite_tick_sinais`, `sprite_transport`). Fica 8227.

**Modelo do início:** componente irmão `SpriteAnimStart`, e não campo do `SpriteAnimator`. O postcard é posicional e o tocador grava-se em blob: um campo novo tornaria ilegível todo blob antigo; irmão não precisa de degrau no `PROJECT_SCHEMA` (o molde de `SpriteAnimFrames` e `SpriteAnimMarkers`, `registry.rs`). Registo `118 → 119`.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E0 | Medir o que grava chave no Animated | feito 06/11 (tabela acima) |
| E1 | Cartão: cortes, cabeçalho de ícones, Frames/s e Speed numa linha, More, Look dobrado, nome em dois cliques, resumo de grade | feito 06/11; os campos de marcador ficam em More até E3, e o resumo «N markers» nasce com eles |
| E2 | Who Picks: Auto, «Starts with»; ▶ só prévia | feito 06/11 |
| E3 | Editor: Split by rows, ◆ marcador, tira rola | feito 06/11; More só resume os marcadores e abre o editor |
| E4 | Add vazio, «Try with the sample hero», aviso de imagem trocada | feito 06/11; aviso em `SpriteCutNotice`, registo `119 → 120` |
| E5 | Regra acesa no Play | a fazer |
| E6 | Censo, cena de smoke, fotos, item em uso no jogo | a fazer |
