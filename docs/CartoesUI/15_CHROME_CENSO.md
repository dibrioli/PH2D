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

**Ordem:** M1 primeiro; as outras não dependem entre si, salvo M2 sobre M1. Antes de M3 a M7, pedir ao dono a referência de cada barra: o menu e as barras não têm maquete aprovada (só o painel tem). Sem a resposta, M1, M2, M8 e M9 podem seguir.

## Estado das fatias (04/11)

| Fatia | Estado | O que entrou | O que falta |
|---|---|---|---|
| M1 | Feita | Porta `widget/lista_flutuante.rs` (`Superficie`: moldura, moldura da dica, moldura de caixa, realce, cor, veste; `cromo_em_cartoes`). Menu aberto, lista de cenas, transbordo da fila, `paint_context_menu` e dica. A lista suspensa do cartão passa pela mesma moldura | Nada |
| M2 | Feita | Título do menu aberto ou sob o rato acende no tom do cartão. A lista e o Settings já vinham da M1 | A cor de fundo da barra (`Bg1`) fica para M3, com a referência do dono |
| M8 | Parcial | Moldura de caixa e tinta do cartão em New Image, New Sprite, Sheet Size, Color Mode, Pixel Edit, Rename, Fill, Onion, Send to board, Confirm delete e `paint_modal` | Corpo em `CardCtx::flutuante`; `FOOTER_H` literal; ver «Aberto» |
| M3–M7 | Não feitas | | Esperam a referência do dono para cada barra |
| M9 | Não feita | | Olhar a paleta aberta |

**Contraprovas (corrida interrompida pelo prazo):** sangraram moldura de lista ignorar o cartão, realce ignorar o cartão, forçar o modo não desligar, cor `Text2` antiga (só `lista_flutuante`). Por correr, com o teste novo das caixas: `Text3`, moldura de caixa, alvo do menu que se move, menu e lista de cenas sem moldura, transbordo sem moldura (sem teste próprio: a fila não semeia no mock), dica sem moldura, título aberto antigo, New Sprite sem moldura. A moldura da dica deu erro de compilação alheio. Uma 1.ª corrida destas ficou inválida: o teste da barra estava vermelho sem mutante.

**Interruptor:** `PH2D_CARDS=0` devolve o de antes em todas as peças acima. **Foto sem rato:** `PH2D_CROMO_SMOKE=<n>` (1 menu da Hierarchy, 2 dica, 3 lista de cenas, 4 File, 5 Settings, 6 New Image, 7 New Sprite, 8 Fill, 9 Onion), `screens/hero/cromo_smoke.rs`.

**Aberto na M8, visto nas fotos `m8_*`:**

- O botão do cartão a `22 px` de altura (`ROW_H`) tem o texto cerca de 2 px baixo. A conta é a do cartão, onde o botão mede `28 px`; a cura é a linha de `FIELD_H` que vem com o `CardCtx::flutuante`.
- New Sprite: «0 px» dentro da caixa (a unidade devia estar no rótulo) e campos antigos.
- Fill: o `Slider` sem rótulo visível e sem o idioma do campo. Onion: os mesmos sliders.
- O campo de texto da lista de cenas ficou em `CardField`, mas o placeholder é o antigo.

**Fora da M1 de propósito:** `widget/popover.rs` (seletor de data, editor rico) e a lista suspensa de fora de um cartão (`dropdown/popover.rs`) continuam na moldura de antes. As linhas dessas listas leem o token cru. Vão com a M6, que tem as listas da barra de opções.

**Decisões tomadas sem o dono:** lista e dica em `KanbanCardRaised` com `FIELD_RADIUS` e a sombra da lista, sem filete (a lista suspensa aprovada). Caixa com `LIST_RADIUS` e as duas sombras da lista. Realce em `CardFieldHover`, como a lista suspensa. O título de menu aberto deixa o azul, para a barra não ter duas cores de estado. Nenhum alvo de clique muda de sítio.

**Correções a este censo, medidas no código:**

- `widget/context_menu.rs` não tem chamador no produto, só o teste `the_modern_family_paints_fewer_frames`: o menu que se vê é o `context_menu_overlay.rs`.
- A lista aberta não realçava em `AccentSoft`, e sim em `Bg2`; `AccentSoft` era só o título do menu aberto e a aba de modo (M3).
- `file_menu.rs` e `chrome/settings_*.rs` são tabelas, sem tinta. `theme_menu.rs` só pinta as amostras de tema.
- `chrome/new_image.rs`, `new_sprite.rs` e `sheet_size.rs` não têm moldura: quem pinta é `context_menu_dialogs.rs` e `context_menu_new_sprite.rs`. Faltavam na linha das caixas: `context_menu_color_mode.rs`, `context_menu_pixel_edit.rs`, `send_to_board.rs` e `document_tabs_menu.rs` (Confirm delete).
- A dica cortava o próprio texto («Add entity» saía «Add ent…»), porque `paint_text_centered` reserva respiro. Curado em cartões; `PH2D_CARDS=0` continua igual.
