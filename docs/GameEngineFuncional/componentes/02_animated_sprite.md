# Animated Sprite

**Estado: `fechado`.** Smoke do dono OK em 07/11; decisões delegadas respondidas: ▶ não grava chave · Frames/s, Cycle e Rules não gravam · aviso de imagem trocada fica salvo até responder · censo 17 aceito se nada essencial sumiu · Look dobrado sem resumo.

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

Logo: «qual animação toca» já grava, nada a construir, e pelo clique na LINHA da lista. Cuidado: a curva escreve o `current` sem rebobinar. O ▶ deixou de gravar (decisão delegada A2, abaixo).

## O que as 9 ensinam

Godot: fatiar vendo a folha, tempo relativo por quadro; evitar o painel que some ao trocar de nó. Construct: Speed, Loop, Ping Pong ao lado dos quadros; evitar a janela que cobre a cena. GameMaker: marcador dentro da faixa. GDevelop: convite no vazio, pontos por quadro. Unreal: evitar a lista de «N members». Nenhuma das 9, nas fontes lidas, liga o estado do personagem à animação no painel: é a nossa vantagem, e a regra acesa no Play a reforça.

## Decisões do dono

**Vigentes:** um cartão Animation + editor próprio, quadros/s (25/10) · lista de regras «quando → toca» (25/10) · seletor Use (26/10) · folha fora do cartão em Animated (27/10) · Direction e overrides só com valor, velocidade negativa toca ao contrário (04/11) · Auto Key grava tudo (19/10) · chaves de Flip e Pivot (decisões 12 e 16 da [Sprite](01_sprite.md)).

**Desta revisão (06/11), todas como recomendado:**

1. **A folha fica no editor.** H/V Frames, From, To e This frame saem do cartão.
2. **Who Picks: Auto · Script · Rules.** «Me» vira «Auto», com o menu «Starts with» ao lado (revê em parte 25/10). O ▶ passa a ser só prévia: não muda com que o jogo começa.
3. **Add ▸ Animated Sprite nasce vazio**. O Character continua com o herói. ~~«Try with the sample hero»~~: Dono 07/11, no smoke: imagens recentes e herói de amostra «não fazem sentido nenhum na engine»; saíram.
4. **Marcador só no editor:** ◆ no quadro da tira; no cartão só o resumo «2 markers».
5. **Regra acesa no Play agora; ensaio com teclas depois.**
6. **Look nasce dobrado em Animated** (nova, trazida pelo levantamento).
7. Versão B + aviso de imagem trocada (da A) + botão **Split by rows** no editor.

**Delegadas (07/11), a confirmar no smoke do dono:**

- **A2, ▶ com Auto Key armado:** o ▶ é só prévia, então não grava chave de `current`, de `playing`, nem da célula do rebobinar (`sprite_grid.frame`, medido pela prova: sem ela nascia uma chave). Clicar na LINHA da lista continua a gravar `current`: é o único gesto que grava qual animação toca. Sobre uma curva, a prévia segura-se como uma pose posada à mão, até o relógio andar.
- **A chave do artista vence o Starts with enquanto existir a curva:** o Play entra no início, e o quadro seguinte da curva escreve o que a chave diz.
- **`PH2D_CARDS=0`, a lei antiga:** sem menu Starts with, a linha e o ▶ doutra linha mudam com o que começa, e o balão de Auto diz isso.
- **Cut again sem proposta só abre o editor:** não é um passo de desfazer, e o aviso fica até a grade mudar.

## O desenho aprovado

**Vazio:** cartão Image = só «Drop a sprite sheet or click to choose» (07/11). Sem Animation.

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
- **Limites destas provas:** as opções do menu pinta-as o hero por cima de tudo; o teste clica a caixa com o ponteiro e entrega o clique da opção como evento, como o do Ordering. O Play real foi corrido depois, em E6.
- **`lei_do_ritmo`, curado:** `row_inteira` (`cards_linhas.rs:40`) soma `FIELD_ROW_GAP` à linha sem rótulo, e `caixa_do_mirror` (`cards_sprite_pivot.rs`) passava-lhe o topo da fila do Mirror: a caixa nascia 6 px abaixo, e o censo media 4 contra o segmento de `SEG_PAD`. A caixa continua à direita dos botões, agora na mesma fila (decisão 23 da Sprite).
- **Editor** (`editor/tests_marks.rs`): `a_transparencia_decide_quais_fileiras_tem_desenho` · `split_by_rows_cria_uma_por_fileira_ocupada_num_so_gesto` · `fileira_ja_coberta_nao_se_duplica` · `o_marcador_poe_nomeia_pinta_e_tira_no_quadro_escolhido` · `clicar_no_campo_renomeia_o_sinal` · `o_marcador_anda_com_o_quadro` · `um_marcador_passado_do_fim_mostra_se_no_ultimo_e_tira_se` · `a_tira_rola_com_a_mao_na_borda`.
- **Sinal, menu, desfazer** (`shells/game-scenes`, `sprite_vazio_e_marcador.rs`): `o_marcador_do_editor_chega_a_regra_e_a_tabela_sem_script` (gesto do editor → tique real → regra Signal e `resolve_signal_actions`) · `o_add_nasce_vazio_e_o_character_com_o_heroi` · `o_heroi_de_amostra_tem_um_passo_de_desfazer` · `as_saidas_do_aviso_voltam_pelo_desfazer`. O desfazer aqui é a foto de antes, a porta que o undo usa; o Ctrl+Z da janela foi corrido em E6.
- **Aviso e vazio** (`sprite_corte_velho.rs`): `escolher_a_imagem_no_vazio_ja_entra_em_animated` · `o_heroi_de_amostra_entra_no_vazio_e_so_no_vazio` · `trocar_por_imagem_sem_corte_acende_o_aviso_com_os_numeros` · `imagem_que_traz_corte_nao_avisa` · `as_duas_saidas_do_aviso` · `cut_again_sem_proposta_abre_o_editor`.
- **Cartão** (`o_cartao_da_animacao.rs`): `o_marcador_no_cartao_e_o_resumo_que_abre_o_editor` · `o_vazio_do_animated_pede_a_folha_e_oferece_o_heroi` · `o_aviso_de_imagem_trocada_diz_os_numeros_e_da_as_duas_saidas`. O convite foi fotografado em E6.
- **Decisões técnicas de E3 e E4, para o smoke:** o Split junta e não apaga: o «loop» da folha inteira fica, e não conta como cobertura. Nomes `row N` em sequência, saltando os ocupados, sem olhar o número da fileira. Sem grade a propor, Cut again abre o editor. O texto com uma animação diz «this 1 animation was».
- **Catraca de `ph2d-app-sprite-anim`:** E1 e E2 já a tinham passado (8542 contra 8352); E3 e E4 chegaram a 9578. Cura por mudança: quatro ficheiros de teste foram para `shells/game-scenes` (`sprite_card_edits`, `sprite_corte_velho`, `sprite_tick_sinais`, `sprite_transport`). Fica 8227.
- **Add pede a imagem** (`fase_object_add_tests.rs`): `a_sprite_e_o_animated_nascem_a_pedir_a_imagem`, que substitui `so_a_sprite_nasce_a_pedir_a_imagem`. O mesmo painel de Assets, «Choose the image for …»; fechar deixa o convite com «Try with the sample hero».
- **E5, regra acesa.** Um campo, `RegrasEmCurso::acesas` (`personagem.rs`), reescrito a cada `anima_personagens`; o critério de parada não disparou: a física não foi tocada. `regra_acesa` lê com o relógio a correr; a shell põe o índice em `AnimCardInfo::live_rule` pelo `entity_bits` do próprio retrato. Uma regra instantânea que segura a animação continua acesa até soltar: é ela que manda. Provas: `sprite_regra_acesa.rs` (`a_regra_acesa_segue_a_condicao_que_muda` · `o_sinal_fica_aceso_enquanto_segura_a_animacao` · `outro_objeto_nao_acende_o_errado` · `a_regra_acesa_nao_entra_no_documento`) e `a_regra_acesa_pinta_a_fila_dela_e_a_etiqueta_live`. Janela real: `[animado-prova] q=… regra acesa do Player` varia entre `Some(2)` e `Some(3)` com o andar. Pausa do Play: nada aceso, porque o relógio parou.
- **Cena 5:** `PH2D_GAME_SMOKE=5`, nível 2 + cinco objetos do menu; o guião mora na folha `ph2d-sprite-anim-scene` e a shell só o executa (`janela_do_jogo_animado.rs`). `PH2D_OBJECT_ADD_PICK=object_add.game.animated_sprite` prova o Add na janela: abre Assets com «Choose the image for …» e o cartão mantém «Try with the sample hero».
- **Catraca:** a shell e `ph2d-app-game` voltaram ao tecto movendo o guião e três cabeçalhos de smoke para docs (`smoke_anim_a_andar`, `smoke_dither`, `smoke_nest`).
- **Play real** (`PH2D_GAME_SMOKE=5`, `PH2D_GAME_SMOKE_DRIVE=editor`, `PH2D_GAME_SMOKE_FOTO=animado_prova_play`): depois de ▶ em `crouch`, início `run`: no jogo toca `run`; `[jogo] Stop: documento e81510e1ee8cab8a · igual ao do Play: true`; depois do Stop, `crouch`.
- **Ctrl+Z real** (`PH2D_GAME_SMOKE_FOTO=animado_prova_desfaz`): Split by rows (1→5→1, com a aba à frente), herói de amostra (0→6→0), Cut again sem proposta e Keep the cut (aviso volta): **um Ctrl+Z cada**. ⛔ **Defeito achado por esta prova:** com a aba do editor à frente, o Ctrl+Z parava em `surface_input::undo` e nada se desfazia. Cura: `DocumentSurface::undo_is_the_projects`; prova `com_a_aba_a_frente_o_ctrl_z_vai_a_fila_do_projeto`. Cut again COM proposta só tem a prova da foto (`as_saidas_do_aviso_voltam_pelo_desfazer`).
- **Aviso no save, decidido:** `SpriteCutNotice` grava-se de propósito. É uma pergunta não respondida sobre um corte que continua errado: reabre enquanto ninguém respondeu, e respondida não volta (`o_aviso_reabre_so_enquanto_ninguem_respondeu`). Tirá-lo do save perderia também o desfazer, que usa a mesma porta.
- **Ruído na Hierarchy:** nenhum ícone novo (`inicio_e_aviso_nao_viram_ruido_na_hierarquia`, e as fotos). O «N components» do cabeçalho sobe 1 uma vez, como já subia com `SpriteAnimFrames` e `SpriteAnimCursor`: é a conta honesta e não cresce a cada gesto. Não curado de propósito.
- **Projeto antigo:** `o_projeto_antigo_abre_igual` (sem os dois componentes: abre, não os inventa, regrava os mesmos bytes, toca o `current`) e os `.ph2dproj` gravados (`um_v191_um_v192_e_um_v193_abrem_sem_kanbans`). `PROJECT_SCHEMA` não mudou.
- **Censo final:** **17 à vista e 48 no total, moldura 22** (`o_censo_dos_campos_do_animated_em_uso`, verde sobre o binário final). A meta do papel, 15 e 47, não foi atingida por 2 e 1: os três verbos do cabeçalho contam como campos e o papel os tratava como moldura. Nada a cortar sem perder poder; o dono julga no smoke.
- **Mutações:** 16 mutações, por nome de teste; **16 sangraram, 0 sobreviveram**: início não usado · ▶ escreve o início · Split sem filtrar fileira vazia · Split sem transparência · aviso com corte novo · aviso nunca · acesa sempre a 1.ª · acesa sem limpar · acesa ignora o Play · acesa esquece o seguro · uma acesa por mundo · fila errada · LIVE sem acesa · aba engole o Ctrl+Z · aviso fora do registo · Add sem pedir. Não correu a crate inteira por mutante: não confere que só o gate certo sangra.

- **Auditorias de 07/11**, cada cura com o evento real:
  - **Espelhos do registo:** `registers_sprite_alongside_ecs` e `registers_luau_script_alongside_ecs` estavam vermelhos (121 contra 119). Agora contam: `ph2d-ecs` + 1, sem literal. A varredura não achou outro espelho.
  - **Renomear preso** (`ph2d-panel-registry-init`, `o_renomear_no_cartao_preso.rs`): `clicar_fora_grava_no_objecto_do_cartao_preso` (a seleção com e sem tocador) · `a_pressao_no_proprio_cartao_nao_apaga_o_que_se_digitou`: defeito extra, `poe_estados_presos` repunha o nome de antes por cima do campo em foco. `o_campo_do_nome_so_abre_no_cartao_do_dono`. Sangram: o nome ia à seleção, ou voltava a `idle`.
  - **▶ armado** (`autokey_previa_tests.rs`, o quadro com curva, gesto, publicar e `apply_samples`): `o_play_armado_nao_grava_chave_e_o_jogo_entra_no_inicio` · `a_linha_armada_grava_chave_de_qual_toca` · `a_chave_do_artista_vence_o_starts_with`. O passe é chamado por `apply_samples`, como os irmãos: a linha `ak.previa` de `run` não corre neste teste, nem foi medida na janela.
  - **`PH2D_CARDS=0`:** `sem_cartoes_a_linha_muda_com_o_que_comeca` · `sem_cartoes_clicar_na_linha_muda_com_o_que_comeca`.
  - **Aviso:** `voltar_a_grade_antiga_nao_ressuscita_o_aviso` · `cut_again_sem_proposta_abre_o_editor`. A prova na janela `animado_prova_desfaz` perdeu o passo do Cut again sem proposta e não foi corrida de novo.
  - **Herói recusado:** `a_recusa_do_heroi_vira_aviso`, com a recusa injetada: a tabela de fábrica não falha, e a toast na shell não tem prova própria.
  - **Reordenar no Play:** o cartão aceita (nenhuma guarda no caminho de `RuleMove`, lido). `reordenar_no_play_leva_o_seguro_junto`.
  - **Play no 1.º quadro:** `o_play_comeca_no_primeiro_quadro_mesmo_ja_a_tocar_o_inicio`, com e sem `SpriteAnimStart`. Vale também ao abrir o projeto, a mesma porta: `o_projeto_antigo_abre_igual` continua verde.
  - **Suspeitas descartadas:** regras no pai com tocador no filho, a regra manda no quadro seguinte (`regras_no_pai_mandam_no_tocador_do_filho_depois_do_inicio`) · LIVE com `PH2D_CARDS=0`, cabe no piso (`sem_cartoes_o_live_cabe_na_coluna`) · nada corta no piso (`os_botoes_novos_nao_cortam_no_piso_do_painel`) · início e aviso em instância e duplicar (`inicio_e_aviso_viajam_no_duplicar_e_na_instancia`) · desfazer do herói: já provado em E6.

**Modelo do início:** campo `SpriteAnimator.start`; o aviso de imagem trocada é campo de `SpriteAnimations` (dobrados em 07/11, CLAUDE §0.11; schema 218, registro 118). Citações abaixo a `SpriteAnimStart` e `SpriteCutNotice` são história.

## Etapas

| # | O quê | Estado |
|---|---|---|
| E0 | Medir o que grava chave no Animated | feito 06/11 (tabela acima) |
| E1 | Cartão: cortes, cabeçalho de ícones, Frames/s e Speed numa linha, More, Look dobrado, nome em dois cliques, resumo de grade | feito 06/11; os campos de marcador ficam em More até E3, e o resumo «N markers» nasce com eles |
| E2 | Who Picks: Auto, «Starts with»; ▶ só prévia | feito 06/11 |
| E3 | Editor: Split by rows, ◆ marcador, tira rola | feito 06/11; More só resume os marcadores e abre o editor |
| E4 | Add vazio, aviso de imagem trocada (o botão do herói saiu em 07/11) | feito 06/11; aviso em `SpriteCutNotice`, registo `119 → 120` |
| E5 | Regra acesa no Play | feito 07/11; um campo, sem tocar a física |
| E6 | Censo, cena de smoke, fotos, item em uso no jogo | feito 07/11: `PH2D_GAME_SMOKE=5`, `=4` em uso; falta o smoke do dono |
