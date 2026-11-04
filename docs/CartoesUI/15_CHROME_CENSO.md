# 15 — Censo do cromo fora dos painéis

Só leitura (Rodada R2, 04/11): tokens contados nos ficheiros, caras olhadas nas fotos `maquete/r2_*` e `f_*`. Nada corrido além delas. Idioma aprovado: `maquete/f_hierarchy_ao_lado_do_inspector.png` (`CardInk`, `metrics`, `Kanban*`, `CardField`, cantos redondos, sem filete, sombra de voo). **Nenhum destes ficheiros usa `CardInk` nem `Kanban*`/`CardField`** (`ink=0` em todos).

| Peça | Ficheiro (`crates/ph2d-editor-core/src/`) | Tokens de hoje | O que a afasta |
|---|---|---|---|
| Barra de menu e lista aberta | `screens/hero/menu_bar.rs`, `menu_rows.rs`, `menu_tables*.rs`, `file_menu.rs` | `BgElev`, `Bg1`, `Text1/2`, `Accent`, `AccentSoft` | Lista em `BgElev` com filete; realce em `AccentSoft`, não `CardFieldHover` |
| Abas de cena | `screens/hero/document_tabs.rs`, `document_tabs_menu.rs` | `Accent`, `BgElev`, `Text1/2/3` | Aba ativa em bloco de acento, altura própria |
| Barra de modos (Draw…Nodes) | `screens/hero/layout_tabs.rs`, `layout_switch.rs` | `BgElev`, `Accent`, `AccentSoft`, `Text2` | O mesmo bloco de acento; devia ser o segmento do cartão (`Seg`) |
| Barra de opções do topo | `screens/hero/tool_bar.rs`, `tool_bars.rs`, `topbar/*`, `float_bar.rs` | `RailBg`, `BgElev`, `Border`, `Bg2` | Botões quadrados com legenda em caixa alta por cima (`MOVE`, `FILL`), caixas com setas ↑↓, «Stroke ▾» em texto, pontos soltos entre grupos |
| Abas de encaixe | `screens/hero/slot_tabs.rs`, `slot_tabs_face.rs`, `slot_tabs_overflow.rs` | `PanelBg`, `WindowGround`, `Bg2`, `Border`, `Text1/2` | Faixa preta de altura inteira por cima do canvas; a 1000 px corta «Color Equalizati…», «Vect…» |
| Barra vertical de tools | `screens/hero/left_rail.rs`, `left_rail/`, `chrome/rail_*.rs` | `RailBg` | A mais próxima do idioma; falta a superfície de lista e o raio de cartão; passa por baixo do HUD em janela baixa |
| HUD | `screens/hero/bottom_hud.rs`, `widget/status_bar.rs` | `BgElev`, `Border`, `Text*`, `Success` | Filete e divisórias; pílula do clássico |
| Dicas | `widget/tooltip.rs`, `screens/hero/topbar/tooltips.rs` | `Bg3`, `Border`, `Text1` | Filete sem sombra de lista |
| Menus de contexto | `widget/context_menu.rs`, `screens/hero/context_menu_overlay.rs`, `menu_row_mark.rs` | `BgElev`, `Bg2`, `Border`, `Text1/2/3` | Seis filetes; a lista suspensa do cartão já tem `card_chip::moldura_da_lista` para reusar |
| Caixas de diálogo internas | `widget/modal.rs`, `chrome/fill_modal.rs`, `onion_modal.rs`, `new_image.rs`, `new_sprite.rs`, `sheet_size.rs`, `context_menu_dialogs.rs` | `BgScrim`, `BgElev`, `Border`, `Text1/2`, `Accent` | Corpo não é cartão; `FOOTER_H` literal; campos antigos |
| Save, Export, abrir ficheiro | `rfd` na shell (`shells/desktop/src/vec_svg_export.rs`…) | sistema operativo | **Fora**: é a caixa do sistema |
| Settings | Não há caixa: linhas de menu `chrome/settings_*.rs`, `theme_menu.rs` | os do menu | Vai com a barra de menu; só lido pelos nomes |
| Paleta de comandos | `screens/hero/chrome/command_palette.rs`, `global_palette*.rs` | `NodeCat*` nas marcas | A superfície vem de fora; não olhada aberta |
| Ecrã inicial | **Não existe**: a procura por `welcome`, `start_screen`, `splash` não acha nenhum; o app abre no editor | — | Nada a converter. Criar um é decisão de produto |

## Partição em fatias de ~35 min

1. **M1 — A moldura partilhada:** `moldura_da_lista` num módulo comum, `widget/context_menu.rs`, `context_menu_overlay.rs`, `tooltip.rs`. Base das demais. Foto: botão direito numa linha da Hierarchy e uma dica.
2. **M2 — Menus:** `menu_bar.rs`, `menu_rows.rs`, `file_menu.rs`, `theme_menu.rs`, Settings. Foto: File e o submenu de Settings abertos.
3. **M3 — Topo da janela:** `layout_tabs.rs`, `document_tabs.rs`, com `Seg` e sem bloco de acento. Foto: um Quadro e um Kanban abertos em abas.
4. **M4 — Abas de encaixe:** `slot_tabs*.rs`, com a faixa a sair de cima do canvas e o nome inteiro ou só o ícone na janela estreita. Foto: 1000 e 1930 px.
5. **M5 — Barra de opções, parte 1:** `topbar/*` (Move/Rot/Scale, Space, View, Undo). Foto: Draw.
6. **M6 — Barra de opções, parte 2:** `tool_bar.rs`, `tool_bars.rs` (as caixas do Vector) e `float_bar.rs`. Foto: Vector e Board. Fatia mais arriscada: tem gates de transbordo.
7. **M7 — Barra vertical + HUD:** `left_rail*`, `rail_*`, `bottom_hud.rs`, `status_bar.rs`. Foto: 1930 e 1000 px.
8. **M8 — Caixas de diálogo:** `modal.rs` e os `chrome/*_modal.rs`, corpo em cartão pelo `CardCtx::flutuante`. Foto: Fill, Onion, New Sprite.
9. **M9 — Paleta de comandos:** olhar aberta primeiro; pode reduzir-se a M1.

**Ordem:** M1 primeiro; as outras não dependem entre si, salvo M2 sobre M1. O menu e as barras não têm maquete aprovada (só o painel tem): a referência segue a ordem que já é lei, e cada escolha fica em «Decisões tomadas sem o dono». Função e ordem das barras não mudam, só a pele.

## Estado das fatias (04/11)

| Fatia | Estado | O que entrou | O que falta |
|---|---|---|---|
| M1 | Feita | Porta `widget/lista_flutuante.rs` (`Superficie`: moldura, moldura da dica, moldura de caixa, realce, cor, veste; `cromo_em_cartoes`). Menu aberto, lista de cenas, transbordo da fila, `paint_context_menu` e dica. A lista suspensa do cartão passa pela mesma moldura | Nada |
| M2 | Feita | Título do menu aberto ou sob o rato acende no tom do cartão. A lista e o Settings já vinham da M1 | A cor de fundo da barra (`Bg1`) fica para M3, com a referência do dono |
| M8 | Feita | Moldura de caixa e tinta do cartão em todas as caixas. Corpo em linhas de cartão pela porta `screens/hero/caixa_cartao.rs` em New Image, Sheet Size, Rename, Color Mode, Send to board e Confirm delete; New Sprite e Pixel Edit (`Caixa`, campos aos pares) em `CardCtx::flutuante`. Fill com rótulo. Trilha do `Slider` nos tons do cartão. Placeholder na tinta. Fechar do Onion com ícone. Texto no meio da caixa em todo cartão | Nada |
| M3 | Feita | Porta `widget/cromo_barra.rs` (`Barra`). Abas de modo e de documento: segmentos numa pílula, sem bloco de acento. Chão da barra de menu no tom da lista | Nada |
| M4 | Feita | Fila de encaixe sem chão preto nem divisória; a escolhida é segmento. Em cartões, nome inteiro ou só o ícone, com o nome no balão; a escolhida é a última a recolher. A faixa do topo (`canvas::paint_top_band`) cobre a grade nas colunas | Nada: o título perde o espaçamento, depois desce ao corpo do rótulo, antes de se cortar (`titulo_que_cabe`) |
| M5 | Feita | `tool_rail/paint_cartao.rs`: chips planos, legendas em caixa normal (tabela `chrome.rail.caption.*`), marca de animação na linha da legenda, setas do número só sob o rato ou com foco | Nada: palavras na tabela `EM_CAIXA_NORMAL` (Cutout, Padding, Color, Equal, Raster, Upscale, Paint, Liquify, Reshape, Inpaint, Select, Ellipse, Polygon, Blow), com gate de largura. Fora de cartões, as de sempre |
| M6 | Feita | Mesmo pintor na fila do Vector; «Stroke ▾» com a seta do cartão; `float_bar` e flyouts no painel sem filete. A lista suspensa de fora de um cartão (`dropdown/popover.rs`) e `widget/popover.rs` vestem a superfície (`veste_fora_de_cartao`) | Nada |
| M7 | Feita | HUD com a superfície da lista, sem filete; o HUD cede à coluna do módulo. `left_rail.rs` (só com F9) veste o painel | Nada: numa janela baixa a coluna parte-se em colunas lado a lado, como a do Quadro (W23 A6), e o flyout abre à direita da última (`m8_coluna_partida_1000x800.png`) |
| M9 | Feita | Paleta com a moldura da caixa, grupos em `KanbanCard`, itens e busca em `CardField`, realce do item sob o rato, ícone em vez do «X» escrito, caixa de marcar do cartão, dica na moldura da lista, texto pela porta do centro. Cena `PH2D_CROMO_SMOKE=16` | Nada |

**Contraprovas da M1/M8 (04/11, 14 mutantes em 4 lotes, 14 sangraram):** `Text3`, moldura de caixa ignora o cartão, alvo do menu que se move, menu sem moldura, lista de cenas sem moldura, transbordo sem moldura, dica sem moldura, texto da dica pelo caminho antigo, título aberto antigo, New Sprite sem moldura, trilha antiga, Fill sem rótulo, unidade fora do rótulo, campo sem marca de cartão. O New Sprite sem moldura sobreviveu à 1.ª corrida (os segmentos escolhidos pintam o mesmo tom): o teste passou a contar `3` pinturas desse tom e sangrou. Restauro conferido por `git diff`. Duas corridas não contam: uma anterior, com o teste da barra vermelho sem mutante, e outra minha, cujo filtro `cromo_tests` não casava com o módulo (`context_menu_overlay_tests::cromo`) e só correu 2 testes. Esse filtro também escondeu dois vermelhos no commit `ecae7d59d5`, curados em `8f899dde78`: o HEAD intermédio tem esses dois testes vermelhos, e compila. Antes: moldura de lista ignorar o cartão, realce ignorar o cartão, forçar o modo não desligar, cor `Text2` antiga.

**Portões por nome (04/11):** verdes `every_widget_file_wires_a11y` (`hr12_widgets_a11y`, 3), `the_shell_only_shrinks` (2: os 19 da C3 cabem na folga, nada a mover), `panel_files_under_loc_cap`, `todo_campo_pintado_com_losango`, `the_new_sprite_dialog` (4), `nenhum_nome_por_cima` (2), `a_marca_tem_a_altura` (2), `toda_dica` (2), `onde_comeca_o_valor` (1). **Vermelho fora desta fatia:** `a_altura_de_abertura_de_um_painel_so_encolhe` (`painter_layers`: mede `1637` px contra `1748` na catraca). O painel encolheu e o número falta escrever; esta fatia não tocou no Painter, e o número é de quem fecha a H3. **Vermelho alheio:** `no_magic_numeric` acusa só `number_chip.rs:275`, um `0.75` que perdeu o marcador `LITERAL-PX-OK` ao ser reformatado no WIP por commitar de outro agente. Nada commitado está vermelho nesse portão.

**Contraprovas da M3–M7 (04/11, 16 mutantes em 2 lotes, 16 sangraram):** sem pílula, chão do menu antigo, chão antigo na fila de encaixe, sem segmento na escolhida, nome cortado, escolhida recolhe primeiro, fila pelo pintor de antes, escolhido sem tom, setas sempre, painel antigo, botão perde a cor da família, HUD antigo, faixa sem a guarda da fila em baixo, HUD não cede, marca ao lado da caixa, legenda crua. «Escolhida recolhe» sobreviveu à 1.ª corrida (a escolhida estava no índice 0, que já recolhia por último): o teste passou a pô-la no fim e sangrou. Cada lote aplicou vários mutantes numa só compilação, com alvos em testes diferentes: o vermelho de um teste não isola o mutante como corridas separadas isolariam. Restauro conferido por `git status`, e de novo depois de uma queda de rede a meio do trabalho.

**Portões por nome (04/11, HEAD `0c72f4617b`):** verdes `no_magic_numeric`, `every_widget_file_wires_a11y` (com `cromo_barra.rs` e `tool_rail/paint_cartao.rs` no `A11Y_OPT_OUT`: só tinta, a árvore é do pai) e, em `ph2d-panel-registry-init`, `quantas_entradas`, `nenhum_nome`, `onde_comeca`, `a_marca_tem`, `toda_dica` (25). **Vermelhos fora desta fatia:** `panel_files_under_loc_cap` (`ph2d-panel-inspector/src/populate.rs`, 602), `todo_campo_pintado_com_losango` (`ordering_cartao.rs`, `sprite_pose.rs`) e `the_shell_only_shrinks` (`ph2d-app-skeleton`, 11307 contra 11263). Nenhum destes ficheiros é meu. A suíte da crate não correu; correram por nome, na árvore final, os testes de abas, fila, coluna, HUD e cromo (147 verdes) e o `o_losango_do_chip_da_barra` de `ph2d-panel-vector` (2 verdes).

**Altura do canvas:** não muda. A régua começa em `y = 76` e a área em `y = 96`, antes e depois (fotos `m6_*`, a 1930×1040); nenhuma conta de `screens/layout.rs` foi tocada.

**Interruptor:** `PH2D_CARDS=0` devolve o de antes em todas as peças acima, salvo a coluna partida (decisão abaixo). **Foto sem rato:** `PH2D_CROMO_SMOKE=<n>` (1 menu da Hierarchy, 2 dica, 3 lista de cenas, 4 File, 5 Settings, 6 New Image, 7 New Sprite, 8 Fill, 9 Onion, 10 Sheet Size, 11 Rename, 12 Color Mode, 13 Canvas Size, 14 Confirm delete e 15 Send to board com `PH2D_BOARD_SMOKE=1` junto, 16 paleta), `screens/hero/cromo_smoke.rs`.

**Texto no meio da caixa (04/11, curado num bloco só).** Medido com foto de antes e depois, por `numpy`, linha a linha, em caixas de 23, 27 e 28 px (Widget Gallery e New Sprite):

| Texto | Caixa | Antes | Depois |
|---|---|---|---|
| `120` (campo) | 28 | +2,5 | −0,5 |
| `64` (campo) | 28 | +2,5 | −0,5 |
| `32` (campo) | 27 | +3,0 | 0,0 |
| `OK` (botão) | 27 | +3,0 | 0,0 |
| `Cancel` (botão) | 27 | +2,5 | −0,5 |
| `RGBA` (segmento) | 23 | +2,5 | −0,5 |
| `Fixed`, `All` (chip, com ascendente) | 28 | +2,0 | −1,0 |

Mecanismo: `(h − px) / 2` tratava a linha como se medisse `px`. A linha de base da Noto (fábrica desde 01/11) cai `1,07 em` abaixo do topo, arredondada pelo parley para `13` px a 12. O desvio é de fundação: em qualquer altura. A hipótese «só a 22 px» e a de `FIELD_H` estavam erradas. Cura: `paint::folga_do_texto(h, px)`, a porta única, nos 45 sítios (meio das maiúsculas pela fonte ativa, lido de `ph2d_text::cap_centre_px`, conferido contra o layout nas 3 fontes). O caminho clássico não muda. Olhados antes e depois, lado a lado: Widget Gallery, Inspector, Bones, New Sprite (`maquete/m8_texto_no_meio_antes_depois_*.png`): nada piorou; o título da lista passa a alinhar com os ícones do cabeçalho. Censo: `nenhum_pintor_centra_pela_conta_de_antes`.

**Curado (04/11):** New Sprite: unidade no rótulo («Width (px)», «Height (px)»), campos e escolhas do cartão. O «0» da foto antiga era da cena de foto, que abria o menu sem semear; agora ela abre pela porta do produto e lê `32`. Fill e Onion: trilha nos tons do cartão, rótulo no Fill. Foto: `maquete/m8_new_sprite.png`, `m8_fill.png`, `m8_onion.png`. As de antes estão no commit `6536cbb6d7`.

**Decisões tomadas sem o dono:** lista e dica em `KanbanCardRaised` com `FIELD_RADIUS` e a sombra da lista, sem filete (a lista suspensa aprovada). Caixa com `LIST_RADIUS` e as duas sombras da lista. Realce em `CardFieldHover`, como a lista suspensa. O título de menu aberto deixa o azul, para a barra não ter duas cores de estado. Nenhum alvo de clique muda de sítio, salvo os dois casos abaixo.

**Decisões tomadas sem o dono (04/11, M8):**

- New Sprite: Width e Height na mesma linha, com o rótulo por cima. A caixa ficou mais baixa e os controlos mudaram de sítio em cartões; os ids são os mesmos. `PH2D_CARDS=0` dá o de antes.
- Fill: o rótulo «Threshold» à esquerda da trilha, como no Onion. A trilha encurta pela esquerda e acaba no mesmo sítio.
- Fill e Onion **mantêm a barra**, em vez do campo numérico que a Física adotou. O cartão não tem barra própria, e estes valores são lidos da barra pela shell a cada quadro: trocar para campo faria a shell mudar, e ela não pode crescer. Preenchimento no azul do botão de destaque, trilha no tom do campo. O dono pode preferir o campo.

**Decisões tomadas sem o dono (04/11, chrome fechado):**

- O centro é o das maiúsculas, não o da caixa da linha: dá `|erro| ≤ 0,5` px nos números, contra `0,75` da linha, porque a linha de base arredonda. Texto com descendente lê-se um pouco abaixo, como em toda fonte.
- Coluna baixa parte-se em duas, sem rolagem nem «…»: é o que o Quadro já fazia (W23 A6), o Photoshop tem a mesma forma, e nenhuma ferramenta fica atrás de um gesto. Vale também em `PH2D_CARDS=0`: uma ferramenta cortada pela borda não era um visual a preservar, era inalcançável. Com lugar, é a coluna de sempre.
- Nomes em vez de siglas, só em cartões: `Cutout` (Background Removal), `Color` (Color Equalization), `Equal` (Equalize Sizes), `Raster`, `Reshape` (Transform: «Transform» pede `44` px e o passo do menor chip tem `40`). O nome inteiro continua na dica. O dono pode preferir outras palavras: é uma linha na tabela `chrome.rail.caption.*`.
- Título de lista estreita: perde o espaçamento, depois desce a `LABEL_PX`; só corta se nem assim couber.
- New Image e Sheet Size: largura `370` px em cartões; tamanhos e fundos em segmentos, com a unidade no rótulo («Size (px)»). Era uma fila de botões em que o escolhido era azul. Os ids são os mesmos.
- Send to board: «New board» passa a botão de linha inteira, sem o risco por cima. Confirm delete: frase do aviso quebra em vez de cortar.
- Canvas Size, Sprite Size, Despeckle e Hue/Saturation: campos aos pares, com a unidade no rótulo (lei da compactação).
- Paleta: as cores de categoria ficam (ensinam o mapa da biblioteca). «Bgremoval» passa a «Background Removal».

**Decisões tomadas sem o dono (04/11, M3–M7):**

- O tom de toda a faixa do topo é o da lista de cartões (`KanbanList`): uma cor só, em vez de `Bg1`, `RailBg` e `WindowGround`.
- Aba de modo, de documento e de encaixe: segmento aceso em `KanbanCardRaised`, como o `Seg` aprovado, sem azul. Aba de encaixe sem pílula: com uma aba só, a pílula seria ruído.
- Ferramenta escolhida: fundo `KanbanCardRaised` e glifo azul, em vez do bloco de azul do Figma, para os glifos de caminho, que têm tinta própria, não ficarem azuis sobre azul. O dono pode preferir o azul cheio.
- As cores de família da coluna ficam: são elas que agrupam as ferramentas.
- A legenda por cima dos chips fica, porque função e ordem não mudam. O Figma não a tem; é dúvida para o dono ([16](16_REFERENCIAS_POR_CARTAO.md)).
- `ROT` passa a «Rotate» e `SQUAR` a «Square» nos cartões, porque cabem por extenso. As palavras vêm de tabela, não de uma regra de caixa: uma regra escreveria `Ceq` e `Upsc`.
- Marca de animação na linha da legenda, como no cartão; a caixa ganha o slot inteiro. O alvo sobe junto.
- Setas do número só sob o rato ou com foco; o alvo delas fica onde estava. **Limite:** num ecrã de toque não há rato, e o toque na ponta da caixa dá um passo sem seta à vista.
- Sem animação de hover nos chips em cartões (D32a): o crescimento e o eixo do relógio ficam só em `PH2D_CARDS=0`.
- Nome de documento é do artista e continua a encurtar com «…»: sem ele a aba não diria nada.

Fotos do fecho: `maquete/m8_new_image_cartao.png`, `m8_sheet_size_cartao.png`, `m8_rename_cartao.png`, `m8_color_mode_cartao.png`, `m8_pixel_edit_cartao.png`, `m8_confirm_delete_cartao.png`, `m8_send_to_board_cartao.png`, `m8_lista_de_cenas.png`, `m8_onion_cartao.png`, `m8_coluna_partida_1000x800.png`, `m9_paleta.png`.

Fotos: `maquete/m3_antes_kanban.png`, `m3_quadro_e_kanban_em_abas.png`, `m4_abas_1000.png`, `m4_abas_1930.png`, `m5_draw.png`, `m6_antes_vector.png`, `m6_vector.png`, `m7_antes_1000.png`, `m7_coluna_e_hud_1000.png`. `PH2D_CARDS=0` fotografado na cena do Vector: barras de antes.

**Correções a este censo, medidas no código:**

- `widget/context_menu.rs` não tem chamador no produto, só o teste `the_modern_family_paints_fewer_frames`: o menu que se vê é o `context_menu_overlay.rs`.
- A lista aberta não realçava em `AccentSoft`, e sim em `Bg2`; `AccentSoft` era só o título do menu aberto e a aba de modo (M3).
- `file_menu.rs` e `chrome/settings_*.rs` são tabelas, sem tinta. `theme_menu.rs` só pinta as amostras de tema.
- `chrome/new_image.rs`, `new_sprite.rs` e `sheet_size.rs` não têm moldura: quem pinta é `context_menu_dialogs.rs` e `context_menu_new_sprite.rs`. Faltavam na linha das caixas: `context_menu_color_mode.rs`, `context_menu_pixel_edit.rs`, `send_to_board.rs` e `document_tabs_menu.rs` (Confirm delete).
- `topbar/*` só pinta com `F9` (`legacy_chrome`): a M5 de verdade é `widget/tool_rail/paint.rs`. A coluna vertical que se vê é `float_bar` + `tool_bars.rs`, não `left_rail.rs`.
- A dica cortava o próprio texto («Add entity» saía «Add ent…»), porque `paint_text_centered` reserva respiro. Curado em cartões; `PH2D_CARDS=0` continua igual.
