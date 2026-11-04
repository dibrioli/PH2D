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
