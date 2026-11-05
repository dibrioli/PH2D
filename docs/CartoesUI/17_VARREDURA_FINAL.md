# 17 — Varredura final da nova UI: o que ficou fora dos cartões

Leitor: quem continua a linha. O censo inicial viu os 29 `impl Panel` + chrome ([15](15_CHROME_CENSO.md)); aqui, o que é aba, modo ou sobreposição. Tudo olhado em foto a `1930 × 1040` (04/11), salvo onde a linha diz «não olhado». Regra: nas áreas de trabalho (canvas do Quadro, texto do editor) o canvas não muda, só o que o rodeia.

Custo: **P** até 30 min · **M** uma fatia · **G** mais de uma fatia. Ordem = mais visto e mais barato primeiro.

| # | Peça | Cena | No padrão? | O que falta | Custo | Estado |
|---|---|---|---|---|---|---|
| 1 | Área do documento (Quadro, Kanban, Planner, Editor) | `PH2D_BOARD_SMOKE=13`, `KANBAN=1`, `PLANNER=1` | **Sim** | Pára antes das duas listas (`clear_of_side_lists`); nada fica tapado | P | ✅ `v_quadro_live_private.png` |
| 2 | Faixa cinza atrás da Hierarchy numa aba de documento | `BOARD=1`, `CODE=5` | **Sim** | Chão da barra por cima da cena, até ao fim da janela (`paint_document_ground`) | P | ✅ `v_editor.png` |
| 3 | Avisos (toast) e barra de trabalho | `MACRO=1`, `MACRO=8` | **Sim** | Superfície levantada (`moldura_de_caixa`), severidade só no ícone. **Mecanismo do aviso da 1.ª linha sem corpo:** o corpo sempre se pintou, com o MESMO pixel da barra de cima, sem moldura no tema moderno: `srgb(13,15,22)` nos dois com `PH2D_CARDS=0`, `srgb(17,21,32)` nos dois com a superfície da lista. Agora `srgb(44,52,71)` sobre `srgb(17,21,32)`. `PH2D_CARDS=0` não muda. O aviso ainda tapa o chip «Object Mode» quando é alto | P | ✅ `v2_avisos_cartoes_e_antes.png` |
| 4 | Painéis do Quadro: Live, Voting, Private, Timer, Comments, Frames, Minimap | `BOARD=9`, `10`, `12`, `13`, `14`, `29` | **Sim** | `moldura_de_painel`, `secao`, `botao`. «Private» com 9 botões de largura; `Line::Toggle` é a caixa de marcar à esquerda, a linha inteira é o alvo. Comments, Frames e Minimap olhados abertos (patch temporário): sem defeito | M | ✅ `v2_private.png`, `v2_quadro_nav_abertos.png` |
| 5 | Barra de estilo do Quadro | `BOARD=28`, `29` | **Sim** | Já era o painel de `Barra` (M6); o censo inicial errou. Olhada em recorte | — | ✅ |
| 6 | Cluster do canto (Live, Present, Comments, Nav) e gavetas da barra | `BOARD=1`, `29` | **Sim** | Painel e chips de `Barra` | P | ✅ |
| 7 | Aviso de menção do Quadro | `BOARD=28` | Sim, sem foto nova | Mesma moldura; não refotografado; teste `o_aviso_de_mencao_veste_o_painel` | P | ✅ |
| 8 | Pílula de acções sobre o objecto (Vector) | `VEC_CARTOES=1`, `OBJECT_MODE=6` | **Sim** | Já era o painel de `Barra`, com raio e sombra; o censo inicial errou. Olhada em recorte | — | ✅ |
| 9 | Barra do Planner | `PLANNER=1` | **Sim**; teste `a_vista_escolhida_e_o_segmento_em_cartoes` | «Calendar» em segmento. «Month» segue azul: é a vista de Calendário do Kanban, referência aprovada | P | ✅ `v_planner.png` |
| 10 | Cartão Constraints do Inspector | `CONSTRAINT=9`, `13` | **Sim** | Números aos pares com a unidade no rótulo, alvo e «Pick» na mesma linha, tipo sem 2.º título. Influence é o campo «Influence (%)», como na Widget Gallery: chip `INSP_CONSTRAINT_FORCA_CHIP`, a barra continua a avisar o painel e o losango lê o mesmo canal | — | ✅ `v2_constraints.png` |
| 11 | Cartão Animation do Inspector | `GAME=2` | Parcial | «Remove Animation» sem o «x» (teste `remover_animacao_nao_leva_o_x_em_letra`). «Playback» e «Idle» ficam: já são `Sm` em título | P | ✅ |
| 12 | Editor de scripts: Console e Problems | `CODE=5` | **Sim** | A aba pára acima do HUD (`chrome_over`, porta que só o Kanban lia); «SCRIPTS» no título da lista. A Timeline a meio é a do layout, não mudou | P | ✅ `v2_editor.png` |
| 13 | Timeline, modos Containers e Arrange | `NEST=2`, `CODE=1` | Sim (bloco I) | «Summary» e linhas da folha são canvas; «+ Track» é uma lista com a seta em ícone (F4, `lista_em`); a letra fica só em `PH2D_CARDS=0` | P | ✅ `f4_track.png` |
| 14 | Kanban (referência aprovada) | `KANBAN=1`, `4`, `8`, `9`; `BOARD=30`, `31` | Sim | Só divergência do item 1; o relatório de importação tapa o HUD e mistura títulos de 14 px no corpo | — | — |
| 15 | Balão do tutorial «fazer» (Macros) | `MACRO=3` | **Sim** | `moldura_de_caixa` + `veste`, texto nos tons da lista. Sem o traço de foco: o anel e o fio já apontam | P | ✅ `v2_balao.png` |
| 16 | Luz 2D | `LIGHT2D=1`, `8` | Sim | Só gizmos no canvas e Inspector já em cartões | — | — |
| 17 | Macros, painel | `MACRO=8` | Sim | Na foto a 5 fps a lista ainda abre; sem defeito | — | — |
| 18 | Game: janela Play e HUD do jogo | `GAME=4 PH2D_GAME_SMOKE_DRIVE=window`, rajada | **Sim** | A janela é só o jogo, sem cromo; nada a curar. O HUD do jogo não aparece nesta cena: não olhado | — | ✅ `v2_janela_play.png` |
| 19 | Ajuda ilustrada (tooltip) e páginas de ajuda do Editor | `OBJECT_ADD_SMOKE=1 OBJECT_ADD_HOVER=object_add.game.character`; página: patch temporário | **Sim** | Dica: moldura do desenho e texto nos tons da lista. Página: já lia `Ui::do_cartao`, sem cura. `PASSO` da cena 5 não abre a página sem o `luau-lsp` na foto | P | ✅ `v2_dica.png`, `v2_ajuda.png` |
| 20 | Menus de contexto por família | `CROMO=1` | Sim (M1) | Mesma moldura; famílias não fotografadas uma a uma | — | — |
| 21 | Pintores velhos fora de `PH2D_CARDS=0` | `grep paint_panel_` | Sim, salvo 22 e 23 | Ramo antigo atrás de `cards_on()` | — | — |
| 22 | `app-sprite-anim` (editor e Region) | `ANIM=1`, patch temporário | **Sim** | Os dois pintores leem `tom()`: tons da lista em cartões, os de antes em `PH2D_CARDS=0`; o ligado é o segmento levantado. O desenho da tela não mudou | M | ✅ `v2_anim_editor.png`, `v2_region.png` |
| 23 | Audio Editor, topo; barra do Motion | `AUDIO_LOOP=1` com HOME semeado; `GPU_COOK_DEMO=142` | **Sim** | O censo estava atrasado: Audio em cartões com `cards_moldura`, pílula e migalhas do Motion com `em_cartoes`. Nada a curar. Migalha só lida, não fotografada (pede subgrafo) | — | ✅ `v2_audio.png` |

## O que falta, por ordem

1. ✅ **`paint_stroke_popover` do Vector fora da camada** (F2): `cards_lado::flutua` pinta por `por_cima` e devolve os alvos ao fechar (`alvos_desde`); vale para os painéis à esquerda, que passam pela mesma porta. Gate `the_stroke_popover_is_painted_over_the_hud`.
2. ✅ **Biblioteca de presets do Wet Paint** (H11). Medido: a lista já ia pela camada, porque `paint_dropdown_list` passa por `por_cima`. Fora dela estavam a barra do arrasto e a janela do Delete (`paint_popover`, `paint_dialogo`): a barra caía na cena do painel, por baixo da lista onde cai. Curados, com os alvos da janela por `alvo`. Por decidir: a linha arrastada dentro da lista mostra só a barra, não voa como o cartão do Kanban.
3. ✅ **Item 13** (F4). Irmão por curar, crate de outra fatia: `panel.vector.bar.stroke_more` («Stroke ▾», `ph2d-panel-vector/src/bars_control.rs:224`).
4. ✅ **`v2_dica.png`** (F4): não era artefacto do arnês. A faixa era o véu do modal (`BgScrim`, preto a 60 %) com o transform do último texto do HUD: um rect de janela inteira deslocado para `(1380, 971)`, logo só o canto se via. Medido: a origem segue o texto do HUD a `1930` e a `1500` de largura; com o véu em magenta, a faixa fica magenta; `viewport` certo e zero clips abertos. Mecanismo: `Encoding::append` (vello 0.11) faz `flags = other.flags`, e `camada_das_listas::fecha` cola uma cena vazia logo depois do HUD, apagando o `FORCE_NEXT_TRANSFORM` do glifo. Cura na porta, `VectorScene::append_transformed`, para todo preenchimento logo a seguir a um colar. Foto: `f4_dica.png`.
5. ✅ **`quantas_entradas…a_altura_de_abertura`:** `painter_layers` em `1670`, medido na H9.

## Proposta para o dono: Widget Gallery

A Galeria é referência aprovada e não foi mudada. O censo (`PENDENTES`, `widget_gallery`, 7 irmãos, 1 grudada) acusa: Linear Damping + Angular Damping, Active + Look Through, Preset + Size (px), Smart Bone + IK, Size (m) + Limit Range, Position (%) + Color, e um par sem nome; grudada: Padding (px). **Proposta:** aplicar a lei da compactação também à Galeria, para ela não ensinar o contrário dos painéis. A decisão é do dono.

## Prova (04/11)

Testes: `screens/hero/varredura_tests.rs` (10), `ph2d-panel-inspector/tests/it/seam_constraints.rs` (2 novos). **Contraprovas: 8 mutantes, 1 por compilação, 8 sangraram**, 2 deles só depois de reforçar o teste: «área só à esquerda» (a régua usava a `draw_area` de docks fixos, que já parava antes da coluna) e «chão fora do quadro» (a cor do chão também vem de outros pintores; agora procura-se o rect dele). Restauro conferido por `git status`. A barra do Planner só tem foto; o `Remove Animation`, só a tabela. `PH2D_CARDS=0` olhado em `maquete/v_quadro_cards_0.png`.

**Portões por nome, verdes:** `hr12_widgets_a11y`, `the_shell_only_shrinks`, `todo_campo_pintado_com_losango`, `panel_files_under_loc_cap`, `nenhum_layout_antigo`, `quantas_entradas`, `nenhum_nome_por_cima`, `onde_comeca_o_valor`, `a_marca_tem_a_altura`, `toda_dica`, `the_top_column_wears` (agora mede o caminho `PH2D_CARDS=0`), `board_` (281). `no_magic_numeric`: vermelho duas vezes às 17:59, verde às 18:00 e 18:01 pela mesma corrida, sem mudança minha entre elas (outros agentes editam a árvore). O relatório do vermelho não foi lido, logo a causa é suposição. A suíte das crates não correu.

**Fatia V2 (04/11).** Testes novos: `no_cartao_a_influence_e_um_campo_em_por_cento` (scrub, Esc, 50 %), `the_card_field_of_the_influence_is_the_same_channel_as_its_bar`, `a_opcao_do_private_e_a_caixa_de_marcar_em_cartoes`, `o_balao_veste_a_caixa_da_lista_em_cartoes`, `a_aba_para_acima_do_hud`, `a_dica_veste_os_tons_da_lista`, `o_editor_veste_os_tons_da_lista_em_cartoes`, `a_vista_escolhida_e_o_segmento_em_cartoes`, `o_aviso_de_mencao_veste_o_painel`, `remover_animacao_nao_leva_o_x_em_letra`. **Contraprovas: 13 mutantes, 13 sangraram**, com restauro conferido. Portões verdes: `no_magic_numeric`, `hr12_widgets_a11y`, `the_shell_only_shrinks`, `todo_campo_pintado_com_losango`, `nenhum_nome_por_cima`, `onde_comeca_o_valor`, `a_marca_tem_a_altura`, `toda_dica`, `panel_files_under_loc_cap`, `lei_da_compactacao`, `nenhum_texto_sai`. Vermelho alheio: item 5 acima. `no_magic_numeric` não falhou nesta fatia, logo o vermelho das 17:59 segue sem relatório lido. A foto da Region e a do editor pedem patch temporário, restaurado. API nova: `MockPanelHost::esc_key`, `INSP_CONSTRAINT_FORCA_CHIP`.

## Decisões tomadas sem o dono

- Um documento em aba pára antes do Inspector, como os Nodes já param. Fica mais estreito com o Inspector aberto, mesmo vazio. O dono pode preferir esconder o Inspector vazio nas abas.
- O painel Private e o cartão do cronómetro ganharam um botão de largura (`CARD_BTNS` 8 → 9), também em `PH2D_CARDS=0`: um texto cortado não era um visual a preservar.
- Aviso sem a faixa de cor em cartões: a severidade fica no ícone.
- Influence virou campo em % (decisão anterior de manter a barra revogada), como na nossa Galeria, com scrub de 1 %. O campo mostra inteiros: um valor gravado de 33,3 % aparece 33. Em `PH2D_CARDS=0` a barra fica.
- Aviso e barra de trabalho levantados: o tom da lista era o da barra de cima. O aviso de menção segue no tom da lista, porque cai sobre o quadro, nunca sobre a barra.
- A aba do Editor pára acima do HUD: fica uma faixa vazia atrás dele, em troca de não tapar a última linha nem o Console.
- O balão do tutorial perdeu o traço de foco em cartões.
- Em cartões, o ligado do editor de animações é o segmento levantado, não o azul.
- `+ Add Animation` manteve o «+»: é o idioma de `+ Lane` e `+ Track`, e há teste com o texto. Só o «x» saiu.
