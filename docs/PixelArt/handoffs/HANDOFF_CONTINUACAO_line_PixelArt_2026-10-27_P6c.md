# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6c (as 3 correcções do smoke da P6b) — 2026-10-27

> Leitor: a próxima LLM que assume a linha numa janela nova. A P6c está **fechada** e o smoke do dono
> (27/10) deu **OK**; trouxe **um pedido novo — a próxima onda, P6d (§3)**. A **linha continua**
> (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado. Lei do módulo:
> [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (§12, emenda P6c)
> · [plano](../02_plano_modulo.md) (§5 P6) · anterior: [P6b](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6b.md) (§4 e §6 continuam valendo) · [P6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) · [P4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou). Commits da onda: `5e7f7f3c23` (feat P6c) · `61622bd58f` (style: `cargo fmt` de ficheiros da linha fora do formato desde ondas anteriores — `smoke_camadas`, `smoke_quadros`, `tests/it/paleta.rs`, `quadros.rs`, `panel-pixel-tools/lib.rs`) · `0614a942f3` (fix clippy) · `b16762989e` (fix do passo vertical dos diálogos).

## 2. O que a P6c fez

1. **Ícone do laço poligonal.** `IconId::PolygonLasso` (`docs/design/icons/polygon-lasso.svg`, original: o laço do Lucide com arestas rectas e vértices marcados), entre `polygon` e `prefab` no enum e em `icons_tests.rs ALL_ICONS`. `Move`, `PixelRectFilled`, `PixelEllipseFilled` ficaram na tabela (biblioteca; sem uso no Pixel).
2. **Botão de grupo GENÉRICO.** `PixelToolGroup {Select, Shape}` (`members()`), `PixelTool::group()`, `PixelGroups` (cara = última usada por grupo via `chose(t)`, `face(g)`, `open: Option<PixelToolGroup>` — um flyout de cada vez) substituem `marquee`/`marquee_open`/`MARQUEES`/`is_marquee`. `PixelRequest::Group(g)` substitui `MarqueeGroup` (está nas listas de `aplica_paleta` e `assenta_antes`). `ids::GROUPS` + `PIXEL_SHAPE_GROUP` (`pixel.tool.shape_group`); `column.rs` põe o botão no lugar do 1.º membro de cada grupo; `bars()` → flyout do grupo aberto, `grouped` = os dois ids. Shape = Rectangle, Ellipse, RegularPolygon, Star; Line, Curve, Contour, Polygon ficam soltos. `PixelTool` e os grupos moram agora em `editor-core/src/pixel_panel/tools.rs` (o `mod.rs` passava o tecto de 700 LOC).
3. **Fill para as 4 formas.** `FilledRectangle`/`FilledEllipse` saíram: a caixa «Fill» (`shape_fill`) vale para as 4 (`gesture.rs` usa `o.shape_fill`; `paint_formas.rs` mostra-a para as quatro). U = Rectangle, Shift+U = Ellipse (não alterna). Cenas 2 (`HOUSE` ganhou o bool fill) e 10 (`SHAPES`, tipo `smoke_formas::Forma`) com Fill por peça.
4. **Move fora.** `PixelTool::Move` saiu (ALL 23 → 20; `ids::TOOLS` 20): o Transform move (arrastar por dentro). `V` = `PixelRequest::Tool(Transform)` como o botão (levanta já; sem selecção, o cel inteiro); atalho exibido «Transform (V)» (a chave `pixel.shortcut_ctrl` saiu). Ctrl+arrasto com uma ferramenta de selecção continua (`solta::press_move`). A amostragem Fast/RotSprite aparece sempre com a moldura. i18n: saíram `pixel.tool.move`, `pixel.hint.move`, `pixel.tool.filled_rectangle`, `pixel.tool.filled_ellipse`, `pixel.shortcut_ctrl`; as dicas do retângulo/elipse/forma dizem «Fill … in the Pixel panel».
5. **Fecho da onda (clippy, gates).** `PixelRequest::Options(Box<PixelOptions>)` (`large_enum_variant` em `PixelRequest` e `EditorAction`: a variante cresceu com `PixelShapes` na P6b). `ph2d-panel-pixel-timeline` `paint_grid.rs` (needless_range_loop) e `paint_props.rs` (clamp): vermelhos antigos, o clippy não corria desde a P0. Os 3 diálogos do Pixel (`context_menu_pixel_edit.rs`, `_color_mode.rs`, `_new_sprite.rs`) passam o passo vertical para `ph2d_tokens::row_pitch_px()`/`control_gap_px()` (4 → 3 px entre linhas; o horizontal fica `Spacing::Xs`) — o `pixel_edit` reprovava `the_row_pitch_is_never_written_at_the_painting_site` e `the_tail_of_a_block_is_never_written_at_the_painting_site` (vindo da P6).

**Cena `PH2D_PIXEL_SMOKE=11`** (`crates/ph2d-app-pixel/src/smoke_ferramentas.rs`, sprite 128×80 RGBA): retângulo oco/cheio e elipse oca/cheia (mesma ferramenta, Fill desligado/ligado), a figura com uma selecção poligonal (`Mask::polygon`), a Elipse na mão com Fill ligado, a cara das selecções = laço poligonal, o flyout das formas aberto. A sonda de nível desconhecido (`tests/it/edit.rs`) passou ao 12; `max_level` 11. Fotos `smoke_P6c_cena11_2026-10-26.png` e `smoke_P6c_cena11_dono_2026-10-26.png` (mesmo dir; `layout.txt` sha256 `5c62c7d1…` igual antes/depois; placa livre: 0,8 de 16 GB). Na arrumação do dono a coluna, 3 botões mais curta, já mostra as duas cores e o Swap — **FECHA** o aberto da P6b §6 «cores escondidas debaixo da timeline».

## 3. Smoke do dono (27/10): OK — pedido novo, a P6d

Nas palavras dele: «os ajustes devem ser em tempo real no gizmo igual ao módulo vector» e «use os mesmos gizmos do vector».

**Leitura:** depois de desenhada, a forma do Pixel fica VIVA (paramétrica, a flutuar sobre o cel até Enter/outra ferramenta), com o gizmo de caixa (escala/giro) e as alças próprias da forma do Vector (raio do canto, lados, raio interno da estrela, sweep…); os campos do painel re-cozinham-na ao vivo; ao assentar rasteriza em células (a lei de `formas::cells`).

**O que a pesquisa no código achou (RECONFIRA com o `explorador` antes de desenhar):**
- Vector: a forma vive em `VecShape::Param {kind,w,h,values}`; campos → `crates/ph2d-app-vec/src/shape_params.rs` (`apply_shape_field` :115, `edit_selected_shape` :204); re-cozimento por alça `shape_live.rs:35`.
- A LEI das alças, sem UI, está em `crates/ph2d-tool-vector/src/shape_handles.rs` (`handles_of` :149, `handle_point` :259, `handle_grip` :291, `drag` :469 devolve `ShapeValues`, `reset` :649) + `shape_handles_table.rs`.
- O glue press/drag/draw em `crates/ph2d-app-vec/src/shape_handles_live.rs` (Gate :35, Target, grips, view :215, press :341, drag :372, draw :538) é da família, não reutilizável: o Pixel refaz o glue.
- Pintura: `crates/ph2d-vec-render/src/shape_nodes.rs:63` `draw_shape_handles` (+`HandleFace` :25). Gizmo de caixa: `ph2d-editor-core/src/gizmo/paint.rs` (`paint_sprite_gizmo` :168, `sprite_gizmo_hit_at` :181), já usado pelo Pixel na moldura solta.
- Camadas: `architecture_no_dependency_climbs_a_layer` permite `ph2d-app-pixel → ph2d-tool-vector` (família → ferramenta); `ph2d-app-pixel/Cargo.toml:14` proíbe `ph2d-app-painter` e o contrato `Tool=12` (usar só as funções puras de `shape_handles`, nunca o `Tool`); `ph2d-app-vec` não pode ser dependido; `architecture_cycle_prevention` **POR CONFERIR**.

Cena de smoke da P6d: a **12** (a sonda de nível desconhecido passa ao 13).

## 4. Superfície de colisão NOVA (soma-se à [P6b §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6b.md))

- `ph2d-editor-core`: `icons.rs`, `icons_tests.rs`, `pixel_panel/{mod,shapes,tools}.rs` (**`tools.rs` NOVO**), `screens/hero/context_menu_{pixel_edit,color_mode,new_sprite}.rs`.
- `ph2d-i18n`: `pixel.rs`, `pixel_selecao.rs`.
- `ph2d-panel-pixel-tools`: `column.rs`, `ids.rs`, `seam.rs`, `paint_formas.rs`, `paint_selecao.rs`. `ph2d-panel-pixel-timeline`: `paint_grid.rs`, `paint_props.rs`. `docs/design/icons/polygon-lasso.svg`.
- ⚠️ Um merge pode partir: quem use `PixelRequest::Options(o)` sem `Box`, `MarqueeGroup`, `marquee`/`marquee_open`, `PixelTool::Move`/`Filled*`, `PixelTool::MARQUEES`/`is_marquee`; e a altura dos 3 diálogos encolheu 1 px por linha.

## 5. Prova (por nome, verde)

- `ph2d-app-pixel`: `scene_11_shows_each_correction`, `each_group_holds_its_last_tool_and_opens_its_four`, `the_fill_box_fills_the_rectangle_and_the_ellipse`, `scene_10`, `formas::`, `selecao::` 14, `desenho::` 11, `paleta::` 19, `ctrl_drag_and_the_transform_tool_lift_the_selection_and_enter_drops_it`, `the_transform_tool_lifts_on_press`.
- `ph2d-panel-pixel-tools`: `seam_pixel` 24, `seam_pixel_formas` 3 (`the_four_shapes_have_fill…`), `seam_pixel_selecao` 7, `seam_pixel_cor` 7. `ph2d-panel-registry-init`: `the_pixel` 3 (a coluna clicada com cada flyout aberto). `ph2d-panel-pixel-timeline`: `seam::` 5.
- `ph2d-editor-core`: `icons::`, `pixel` 54, `new_sprite` 4, `color_mode` 6, `workspace_file_loc_cap`, `the_shell_only_shrinks`, `no_dependency_climbs` 3, `panel_crates_depend_only`, `the_row_pitch_is_never…` e `the_tail_of_a_block…` (4). `ph2d-i18n`: `nenhum_motor`.
- `cargo fmt --all --check` limpo; clippy `-D warnings --all-targets` verde em app-pixel, editor-core, i18n, panel-pixel-tools, panel-pixel-timeline, panel-registry-init.
- `PH2D_SUITE=onda nextest-impacted`: 19 837 testes, 3 vermelhos — os 2 gates de passo (curados no §2.5) e `ph2d-host-code-scenes a_cena_3_do_editor_conduzida::a_cena_3_conduzida_passo_a_passo` (30 s; passa sozinho com load 5,8; durante a bateria o load era 30–43: flake de carga, não desta linha). Vermelho antigo conhecido: `a_altura_de_abertura_de_um_painel_so_encolhe` (P0/P6).

## 6. ABERTO

- **A P6d (§3).**
- Inclinar só pelo campo Skew (o gizmo não inclina).
- O flyout de 4 quebra em 3+1 (widget partilhado; igual ao das selecções na P6b).
- O resto do [P6 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) e do [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).
- **Mutação no fecho da LINHA** (somar às da P6b §6): `PixelGroups::chose`/`Group(g)`, `shape_fill` para Rectangle/Ellipse, `V` → Transform.

## 7. Próximo item

P6d — as formas do Pixel vivas com os gizmos do Vector: ler primeiro, com o `explorador`, os ficheiros do §3 e conferir `architecture_cycle_prevention`; cena 12. Fotos de fábrica e com `HOME=/home/enio` (copiar antes `~/.ph2d/layout.txt`; `nvidia-smi` antes).
