# 17 — Varredura final da nova UI: o que ficou fora dos cartões

Leitor: quem continua a linha. O censo inicial viu os 29 `impl Panel` + chrome ([15](15_CHROME_CENSO.md)); aqui, o que é aba, modo ou sobreposição. Tudo olhado em foto a `1930 × 1040` (04/11), salvo onde a linha diz «não olhado». Regra: nas áreas de trabalho (canvas do Quadro, texto do editor) o canvas não muda, só o que o rodeia.

Custo: **P** até 30 min · **M** uma fatia · **G** mais de uma fatia. Ordem = mais visto e mais barato primeiro.

| # | Peça | Cena | No padrão? | O que falta | Custo | Estado |
|---|---|---|---|---|---|---|
| 1 | Área do documento (Quadro, Kanban, Planner, Editor) | `PH2D_BOARD_SMOKE=13`, `KANBAN=1`, `PLANNER=1` | **Sim** | Pára antes das duas listas (`clear_of_side_lists`); nada fica tapado | P | ✅ `v_quadro_live_private.png` |
| 2 | Faixa cinza atrás da Hierarchy numa aba de documento | `BOARD=1`, `CODE=5` | **Sim** | Chão da barra por cima da cena, até ao fim da janela (`paint_document_ground`) | P | ✅ `v_editor.png` |
| 3 | Avisos (toast) e barra de trabalho | `MACRO=1`, `MACRO=8` | **Sim** | Superfície da lista, severidade só no ícone. O aviso da 1.ª linha saía sem corpo; agora tem. **A causa não foi achada**: a cura foi trocar o pintor. O aviso ainda tapa o chip «Object Mode» da fila quando é alto | P | ✅ `v_avisos.png` |
| 4 | Painéis do Quadro: Live, Voting, Private, Timer, Comments, Frames, Minimap | `BOARD=9`, `10`, `12`, `13`, `14`, `29` | **Sim** | `moldura_de_painel`, `secao`, `botao`. «Private» passou a 9 botões de largura; `Line::Toggle` segue com círculo em vez da caixa de marcar; Comments, Frames e Minimap abertos não foram olhados | M | ✅ `v_quadro_voting.png` |
| 5 | Barra de estilo do Quadro | `BOARD=28`, `29` | **Sim** | Já era o painel de `Barra` (M6); o censo inicial errou. Olhada em recorte | — | ✅ |
| 6 | Cluster do canto (Live, Present, Comments, Nav) e gavetas da barra | `BOARD=1`, `29` | **Sim** | Painel e chips de `Barra` | P | ✅ |
| 7 | Aviso de menção do Quadro | `BOARD=28` | Sim, sem foto nova | Mesma moldura; não refotografado e sem teste próprio | P | ✅ |
| 8 | Pílula de acções sobre o objecto (Vector) | `VEC_CARTOES=1`, `OBJECT_MODE=6` | **Sim** | Já era o painel de `Barra`, com raio e sombra; o censo inicial errou. Olhada em recorte | — | ✅ |
| 9 | Barra do Planner | `PLANNER=1` | **Sim**; sem teste | «Calendar» em segmento. «Month» segue azul: é a vista de Calendário do Kanban, referência aprovada | P | ✅ `v_planner.png` |
| 10 | Cartão Constraints do Inspector | `CONSTRAINT=9`, `13` | **Quase** | Números aos pares com a unidade no rótulo, alvo e «Pick» na mesma linha, tipo sem 2.º título. **Falta:** a barra azul de Influence virar campo (pede chip derivado e ids novos) | P o que falta | 🟨 `v_constraints.png` |
| 11 | Cartão Animation do Inspector | `GAME=2` | Parcial | «Remove Animation» sem o «x». «Playback» e «Idle» ficam: já são `Sm` em título | P | 🟨 |
| 12 | Editor de scripts: Console e Problems | `CODE=5` | Parcial | HUD por cima da faixa, Timeline a meio; título «Scripts» fora do idioma da lista | P | ⬜ |
| 13 | Timeline, modos Containers e Arrange | `NEST=2`, `CODE=1` | Sim (bloco I) | «Summary» e linhas da folha são canvas; «+ Track ▾» com seta em letra | P | ⬜ |
| 14 | Kanban (referência aprovada) | `KANBAN=1`, `4`, `8`, `9`; `BOARD=30`, `31` | Sim | Só divergência do item 1; o relatório de importação tapa o HUD e mistura títulos de 14 px no corpo | — | — |
| 15 | Balão do tutorial «fazer» (Macros) | `screens/hero/chrome/tutorial_balao.rs` | **Não olhado** | Lê `BgElev`, moldura e `paint_button` antigos | P | ⬜ |
| 16 | Luz 2D | `LIGHT2D=1`, `8` | Sim | Só gizmos no canvas e Inspector já em cartões | — | — |
| 17 | Macros, painel | `MACRO=8` | Sim | Na foto a 5 fps a lista ainda abre; sem defeito | — | — |
| 18 | Game: janela Play e HUD do jogo | `GAME=1`, foto de 2.ª janela | **Não olhado** | Janela do jogo e HUD no palco | ? | ⬜ |
| 19 | Ajuda ilustrada (tooltip) e páginas de ajuda do Editor | `CODE=5` com passo | **Não olhado** | `widget/illustration` | ? | ⬜ |
| 20 | Menus de contexto por família | `CROMO=1` | Sim (M1) | Mesma moldura; famílias não fotografadas uma a uma | — | — |
| 21 | Pintores velhos fora de `PH2D_CARDS=0` | `grep paint_panel_` | Sim, salvo 22 e 23 | Ramo antigo atrás de `cards_on()` | — | — |
| 22 | `app-sprite-anim` (editor e Region) | `editor/paint.rs`, `region/paint.rs` | **Não olhado** | Lê `Bg*` e `Border` direto | M | ⬜ |
| 23 | Audio Editor, topo; barra do Motion (`paint_chrome`, `paint_breadcrumb`, `paint_menu`) | `audio_overlay.rs` | **Não olhado** | Lê `Bg*`, `Border` direto | M | ⬜ |

## O que os próximos agentes pegam, por ordem

1. **Item 10, a Influence em campo:** molde «Slider sem chip» da [receita](11_RECEITA_PAINEL_EM_CARTOES.md).
2. **Item 15, balão do tutorial:** `moldura_de_caixa` + `veste` em `chrome/tutorial_balao.rs`; foto `PH2D_MACRO_SMOKE=3`.
3. **Item 4, o resto:** `Line::Toggle` do Private para a caixa de marcar; olhar Comments, Frames e Minimap abertos.
4. **Item 12:** o HUD tapa a faixa Console do Editor; «Scripts» fora do idioma da lista.
5. **Itens 18, 19, 22, 23:** olhar primeiro. Nenhum foi fotografado.

## Prova (04/11)

Testes: `screens/hero/varredura_tests.rs` (10), `ph2d-panel-inspector/tests/it/seam_constraints.rs` (2 novos). **Contraprovas: 8 mutantes, 1 por compilação, 8 sangraram**, 2 deles só depois de reforçar o teste: «área só à esquerda» (a régua usava a `draw_area` de docks fixos, que já parava antes da coluna) e «chão fora do quadro» (a cor do chão também vem de outros pintores; agora procura-se o rect dele). Restauro conferido por `git status`. A barra do Planner só tem foto; o `Remove Animation`, só a tabela. `PH2D_CARDS=0` olhado em `maquete/v_quadro_cards_0.png`.

**Portões por nome, verdes:** `hr12_widgets_a11y`, `the_shell_only_shrinks`, `todo_campo_pintado_com_losango`, `panel_files_under_loc_cap`, `nenhum_layout_antigo`, `quantas_entradas`, `nenhum_nome_por_cima`, `onde_comeca_o_valor`, `a_marca_tem_a_altura`, `toda_dica`, `the_top_column_wears` (agora mede o caminho `PH2D_CARDS=0`), `board_` (281). `no_magic_numeric`: vermelho duas vezes às 17:59, verde às 18:00 e 18:01 pela mesma corrida, sem mudança minha entre elas (outros agentes editam a árvore). O relatório do vermelho não foi lido, logo a causa é suposição. A suíte das crates não correu.

## Decisões tomadas sem o dono

- Um documento em aba pára antes do Inspector, como os Nodes já param. Fica mais estreito com o Inspector aberto, mesmo vazio. O dono pode preferir esconder o Inspector vazio nas abas.
- O painel Private e o cartão do cronómetro ganharam um botão de largura (`CARD_BTNS` 8 → 9), também em `PH2D_CARDS=0`: um texto cortado não era um visual a preservar.
- Aviso sem a faixa de cor em cartões: a severidade fica no ícone.
- Influence mantém a barra por agora, nos tons do cartão (precedente: Fill e Onion).
- `+ Add Animation` manteve o «+»: é o idioma de `+ Lane` e `+ Track`, e há teste com o texto. Só o «x» saiu.
