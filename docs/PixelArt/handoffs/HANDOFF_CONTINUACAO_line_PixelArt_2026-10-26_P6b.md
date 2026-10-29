# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6b (as 4 correcções do smoke da P6) — 2026-10-26

> Leitor: a próxima LLM que assume a linha numa janela nova. A P6b está **fechada**; o smoke do dono
> (26/10, após as fotos) deu **3 correcções novas — são a próxima onda, P6c (§3)**. A **linha continua**
> (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado. Lei do módulo:
> [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (§12, emenda P6b)
> · [plano](../02_plano_modulo.md) (§5 P6) · anterior: [P6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) (§4 e §6 continuam valendo) · [P4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou). Commits desta onda: `7ade8ad0ba` (feat P6b) e `c393bf7574` (fix da foto da cena 10).

## 2. O que a P6b fez (as 4 correcções do smoke da P6)

1. **Ícones.** Varinha = `IconId::WandSparkles` (Lucide wand-sparkles, ISC, `docs/design/icons/wand-sparkles.svg`); Move ganhou `IconId::Move` (`move.svg`); elipse de selecção `IconId::CircleDashed` (`circle-dashed.svg`). A ordem do enum = a dos ficheiros (circle < circle-dashed).
2. **Botão de grupo das 4 selecções.** `ids::PIXEL_SELECT_GROUP`, `PixelPanelInfo.marquee`/`marquee_open`, `PixelRequest::MarqueeGroup` (a última na mão + abre/fecha), `column::bars(info) -> ToolBars` com flyout; `ToolBars.grouped` (campo novo, opt-in) pinta o triângulo no canto (`screens/hero/tool_bars.rs`, `more_mark`). `PixelTool::MARQUEES`/`is_marquee`; `set_tool` lembra a última e fecha o flyout. ⚠️ `aplica_paleta` e `assenta_antes` listam os pedidos que NÃO são deles: `MarqueeGroup` teve de entrar nas duas (um teste apanhou o pedido engolido).
3. **Gizmo do sprite da casa na moldura solta.** `Solta::gizmo(to_screen, janela, cursor) -> GizmoView` (caixa = fonte 0..w×0..h, model = matriz da moldura → ecrã com Y invertido, câmera = o próprio ecrã); hit = `gizmo::sprite_gizmo_hit_at` (novo, `editor-core/gizmo/paint.rs`: pinta num `HitIndex` privado — a família tem despacho próprio); `alca()` traduz ids `GIZMO_*` para `Handle` do núcleo; a conta continua `Affine::drag` (células inteiras). Modificadores do gizmo: Shift proporção/15°, Ctrl do centro; inclinação só pelo campo Skew. `draw_transform_frame`/`HANDLE_PX` saíram do `ph2d-pixel-render`. `PixelTool::Transform` (Ctrl+T, rótulo por `pixel.shortcut_ctrl`): escolhê-la ou premir com ela levanta (selecção ou cel inteiro); `E::Transform` põe-na na mão. `frame.rs`: «sair do Draw assenta» só se `was_editing.is_some()` (antes, a cena levantava antes do 1.º Draw e o gizmo nunca aparecia).
4. **Formas do Vector.** `editor-core/pixel_panel/shapes.rs` (`PixelShape {Rectangle→RoundRect, Ellipse, Polygon, Star}`, `PixelShapeValues` em bits para o `Eq`, `PixelShapes`; defaults = os do Vector com o raio do RoundRect a 0); `PixelOptions.shapes` + `shape_fill` substituem `corner_radius` (chave `panel.pixel.corner` apagada); `PixelTool::RegularPolygon` e `Star` novos (sem atalho, `PixelTool::named()`). `ph2d-app-pixel/src/formas.rs`: `ph2d_vec_scene::cook` com Y espelhado, achatado a 0,5 célula, `polygon_cells` do núcleo, furos par-ímpar entre contornos; com campos neutros o retângulo e a elipse continuam `rounded_rect_cells`/Zingl (prova por igualdade). Painel: `seam_formas.rs`/`paint_formas.rs` leem `ph2d_tool_vector::shapes::SHAPES` (panel→tool permitido); os NOMES vêm da ponte `ph2d_panel_vector::nomes_do_motor::campo` injectada pelo `ph2d-panel-registry-init` (`set_field_names`) — painel ↛ painel (gates `architecture_cycle_prevention`, `no_dependency_climbs_a_layer`) e o gate `a_fronteira_dos_motores` do `ph2d-i18n` só aceita pontes dentro de crates `ph2d-panel-*`. Caras das formas novas na coluna = `BarFace::Outline` do cook (`column::face`). Deps novas: editor-core +`ph2d-vec-scene` (já no grafo), panel-pixel-tools +`ph2d-tool-vector` +`ph2d-vec-scene`, app-pixel +`ph2d-vec-scene`.

**Cena `PH2D_PIXEL_SMOKE=10`** (`smoke_formas.rs`, sprite 128×80): as 4 formas, a figura a 2× girada 30° no gizmo, estrela na mão, flyout aberto com a Elliptical na cara. Fotos `smoke_P6b_cena10_2026-10-26.png` e `_dono_` (`layout.txt` sha256 `ee9932ad…` igual antes/depois). ⚠️ A placa estava cheia pelo ollama do dono (13,8 GB): TODA foto saía cinzenta com «Out of Memory» do vello; a Radeon não apresenta em `kwin --virtual` (`Surface not configured`). Antes de fotografar: `nvidia-smi`.

## 3. Smoke do dono (26/10, após as fotos) — a P6c, NESTA ordem

1. «O ícone de seleção poligonal deveria ser melhor» → `PixelTool::PolygonLasso` usa `IconId::VectorPen` (`column.rs` `icon()`). Fazer um ícone próprio de laço poligonal (polilinha fechada tracejada com vértices), no estilo dos SVG da casa (stroke 1.5, 24×24), original ou Lucide se houver.
2. «As shapes devem compartilhar um só espaço na barra como as seleções» → **DECISÃO DO DONO (respondida 26/10):** UM botão de grupo com as 4 formas (Retângulo, Elipse, Polígono regular, Estrela) + triângulo + flyout, como as selecções. `FilledRectangle` e `FilledEllipse` DEIXAM de ser ferramentas — viram a caixa «Fill» do painel (`shape_fill` passa a valer para as 4); o U deixa de alternar cheio/vazio (decidir: U = retângulo, Shift+U = elipse; Fill pelo painel). Linha, Curva, Contorno e Polígono livre continuam soltos. Reusar o mecanismo do grupo das selecções (generalizar `MarqueeGroup`/`marquee` para um grupo por família, sem duplicar).
3. «Transform e move são redundantes. Basta transform» → tirar `PixelTool::Move` da coluna; o Transform faz o mover (arrastar por dentro já move); o V passa a escolher o Transform; Ctrl+arrasto com uma ferramenta de selecção continua a levantar e mover. Cuidar dos testes que usam `Tool::Move`: `tests/it/selecao.rs` (`the_move_tool_and_ctrl_drag…`), `desenho.rs` (contagem de ferramentas), `seam_pixel`/`seam_pixel_selecao`, `the_pixel_column_answers_a_real_pointer`.

Cena de smoke da P6c: a **11** (a sonda de nível desconhecido em `tests/it/edit.rs` passa ao 12).

## 4. Superfície de colisão NOVA (soma-se à [P6 §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md))

- `ph2d-editor-core`: `Cargo.toml`, `gizmo/{mod,paint}.rs`, `icons.rs`, `icons_tests.rs`, `interaction/tool_bars.rs` (`ToolBars.grouped`), `screens/hero/tool_bars.rs`, `pixel_panel/{mod,shapes}.rs`.
- `ph2d-i18n`: `pixel.rs`, `pixel_selecao.rs`. `ph2d-panel-vector/src/bars.rs` (`grouped: Vec::new()`). `ph2d-panel-registry-init/src/lib.rs` (`set_field_names`). `ph2d-pixel-render`: `overlay`, `lib`. Shell: `render_loop/fase_pixel_panel.rs` (encurtou). `docs/design/icons` (3 SVG). **`Cargo.lock` mudou.**
- ⚠️ Um merge pode partir: `ToolBars` com campo novo (todo literal fora da casa precisa de `grouped`); a lista de pedidos de `aplica_paleta`/`assenta_antes`; quem ainda use `draw_transform_frame`.

## 5. Prova (por nome, verde)

`formas::` 6 (app-pixel) · `seam_pixel_formas` 3 · `seam_pixel` 24 (panel) · `the_pixel_column_answers_a_real_pointer` + `the_pixel_shape_fields_are_named_by_the_vector_panels_bridge` (registry-init) · `icons::` 8 · `architecture_no_dependency_climbs_a_layer` · `panel_crates_depend_only_on_editor_core` · `nenhum_motor_publica_um_rotulo_cru` · `every_word_this_panel` · `every_name_the_engine_publishes` (panel-vector) · `selecao::` 14 · `the_handles_scale_rotate` (passa com o gizmo da casa) · `architecture_workspace_file_loc_cap` · `the_shell_only_shrinks`. Vermelho conhecido e antigo: `a_altura_de_abertura_de_um_painel_so_encolhe` (inspector 956/831, painter_layers 1620/1570; passa sozinho, reprova com o módulo — P0/P6). Clippy não corrido (só no fecho).

## 6. ABERTO

- **A P6c (§3).**
- Na arrumação do dono as duas cores e o Swap do fim da coluna ficam escondidos debaixo da linha do tempo (já na foto da P6): a coluna precisa de caber ou rolar.
- Inclinar só pelo campo Skew (o gizmo não inclina).
- O resto do [P6 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) e do [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).
- **Mutação no fecho** (somar às anteriores): `formas::cells` neutra vs Vector, `MarqueeGroup`, `alca()`, `frame` `was_editing`.

## 7. Próximo item

P6c em 3 passos, na ordem do §3, cena 11. Fotos de fábrica e com `HOME=/home/enio` (copiar antes `~/.ph2d/layout.txt`; conferir `nvidia-smi` antes).
