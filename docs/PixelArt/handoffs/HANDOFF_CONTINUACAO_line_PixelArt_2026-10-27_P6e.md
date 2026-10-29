# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6e (o catálogo inteiro do Vector no Pixel) — 2026-10-27

> Leitor: a próxima LLM que assume a linha. A P6e está **feita**; o smoke do dono (cena 13) está
> **POR FAZER**. A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) §12 (emenda P6e)
> · anterior: [P6d](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6d.md) (§6 continua valendo; smoke OK) · [P6c](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou). Commit da onda: `fcfe1db7d2` (feat P6e).
- Pedido do dono (27/10, verbatim): *«Traga todas as outras shapes presentes em vector para pixel mantendo os mesmos gizmos maravilhosos»*.

## 2. O que a P6e fez

1. **`PixelShape` = uma `ShapeKind` do Vector** (`ph2d-editor-core/src/pixel_panel/shapes.rs`): newtype; `ALL` vem de `ph2d_vec_scene::ALL_SHAPES` (49); consts `RECTANGLE` (= `RoundRect`, o `U` do Aseprite), `ELLIPSE`, `POLYGON`, `STAR`. `PixelShapes` guarda uma gaveta por DISCRIMINANTE (`SLOTS` = 50; o 14 é lacuna do enum). `defaults`: o raio de MUNDO do Vector (`DEFAULT_CORNER_RADIUS`, campo 0 de `RoundRect` e `SpeechRect`) nasce a zero — lista nomeada porque o `Bars` da `PredefinedProcess` também vale `0,12` (fracção), e o teste do painel varre TODO campo `Px` de TODA forma.
2. **`PixelTool::Shape(PixelShape)`** substitui `Rectangle/Ellipse/RegularPolygon/Star` (`tools.rs`). `PixelTool::ALL` = 16 simples + 49 formas (formas depois da Curva, na ordem do enum). `PixelToolGroup::members()` saiu → `first()` (cara de fábrica) e, no painel, `column::members(g)` = `ids::TOOLS` filtrado por grupo. `PixelTool::name()`: o do Aseprite, ou o do Vector pela ponte.
3. **Painel** (`ph2d-panel-pixel-tools`): `ids::TOOLS` é `LazyLock<Vec>` — as formas na ordem de `ph2d_tool_vector::shapes::SHAPES` (a do flyout do Vector), id `pixel.tool.shape.<u16>`; os outros mantêm o hash do `label_key`. A cara de cada forma é `ph2d_tool_vector::shape_thumbs::thumb_outline` — o módulo `shape_outlines` do `ph2d-panel-vector` **subiu** para lá (`git mv`; o Vector usa-o de lá). A coluna põe o botão de grupo na 1.ª ferramenta do grupo que aparece. Sem «Fill» nas abertas. Nomes: `nomes_do_motor::forma` passou a `pub`; o registo injecta-a (`set_shape_names` → `ph2d_editor_core::pixel_panel::set_shape_names(shape_name)`), como a dos campos.
4. **Abertas** (`formas.rs`): contorno com `closed == false` (de `trim_tool::contours_of`) traça-se com `line_path` entre vértices, sem fechar nem interior (também as arestas de dentro do cubo/cilindro); a forma com `ShapeKind::is_closed() == false` (Line, Arc, Spiral, NoteBracket, Brace — a lei do Vector) nunca enche, mesmo com o Fill ligado. `neutra` ganhou o `Rectangle` do Vector sem canto (= retângulo do Aseprite).
5. **Viva**: nada mudou em `viva.rs` — a P6d já era genérica (`handles_of(kind)`); as 46 formas com alças no Vector têm-nas no Pixel; Line, Pill e Junction não têm (lei do Vector).
6. i18n: saem `pixel.tool.{rectangle,ellipse,regular_polygon,star}`; entra `pixel.tool.shape` («Shape», só sem a ponte). Os textos `pixel.hint.rectangle/ellipse` ficam para `U`/`Shift+U`; as outras usam `pixel.hint.shape`.

**Cena `PH2D_PIXEL_SMOKE=13`** (`crates/ph2d-app-pixel/src/smoke_catalogo.rs`, 160×110 RGBA): grelha 3×3 assentada (Spiral, Arc, Pie, ArrowBent, Document, Thought, Heart, IsoCube, Brace — as sete famílias do catálogo, três abertas em traço) e uma engrenagem cheia VIVA à direita com a caixa e as alças; Engrenagem na mão. `max_level` 13; a sonda de nível desconhecido (`tests/it/edit.rs`) passou ao 14. Fotos: `smoke_P6e_cena13_2026-10-27.png` (fábrica) e `smoke_P6e_cena13_dono_2026-10-27.png` (`HOME=/home/enio`; `layout.txt` sha256 `5c62c7d1…` igual antes/depois; placa: 1,5 de 16 GB).

## 3. Smoke do dono — POR FAZER

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=13 ./target/smoke/ph2d-host-desktop`
2. Abre o desenho com nove formas já pintadas e, à direita, uma engrenagem vermelha com a caixa e as pontinhas.
3. Arraste uma pontinha da engrenagem (os dentes, o furo); mude «Teeth» no painel «Pixel» à direita: muda na hora. Enter pinta.
4. Clique no botão das formas na barra da esquerda (a engrenagem): abre a lista com TODAS as formas do Vector, na mesma ordem e com os mesmos desenhos. Escolha outra (seta, balão, coração, cubo…), arraste no desenho: nasce com as pontinhas dela.
5. Escolha a Espiral, o Arco ou a Linha e arraste: ao Enter fica só a linha fina, sem enchimento (não há «Fill» no painel para elas).
6. Deu errado se: falta uma forma que o Vector tem; uma forma nasce sem as pontinhas que tem no Vector; a espiral/arco sai cheia; Ctrl+Z não desfaz a forma inteira.

## 4. Superfície de colisão NOVA (soma-se à P6d §4)

- `ph2d-editor-core`: `pixel_panel/{shapes,tools,mod}.rs` — ⚠️ quem usa `PixelTool::Rectangle/Ellipse/RegularPolygon/Star`, `PixelShape::Rectangle…` ou `PixelToolGroup::members()` parte (agora `PixelTool::RECTANGLE`/`ELLIPSE`/`Shape(_)`, `PixelShape::RECTANGLE…`, `first()`). `PixelTool::ALL` mudou de tamanho.
- `ph2d-panel-vector`: `shape_outlines.rs` **saiu** (→ `ph2d-tool-vector/src/shape_thumbs.rs`); `bars.rs`, `lib.rs`, `paint_modes.rs`, `nomes_do_motor.rs` (`forma` pub). `ph2d-tool-vector`: `lib.rs`, `shape_thumbs.rs` NOVO.
- `ph2d-panel-pixel-tools`: `ids.rs` (`TOOLS` não é mais `const` array: `for … in ids::TOOLS` → `.iter()`), `column.rs`, `seam.rs`, `seam_formas.rs`, `paint_formas.rs`, `lib.rs`. `ph2d-panel-registry-init`: `lib.rs`. `ph2d-i18n`: `pixel.rs`, `pixel_selecao.rs`.
- `ph2d-app-pixel`: `formas.rs`, `input.rs`, `smoke*.rs`, `lib.rs`; testes `catalogo.rs` NOVO, `viva.rs` (`Mesa` `pub(crate)`).

## 5. Prova (por nome, verde)

- `ph2d-app-pixel` `catalogo` 3 NOVOS: `every_vector_shape_is_born_live_with_the_vector_handles_and_settles_as_one_step` (as 49: viva, nº de alças = `handles_of`, ≥ 1 à vista, nada pintado antes do Enter, um passo de desfazer; controlo ≥ 40 com alças), `an_open_shape_settles_as_a_one_cell_stroke_even_with_fill_on` (as 5 abertas: pintado == traço de `formas::cells`, interior vazio, o Vector cozinha-as sem contorno fechado; controlo Heart enche), `scene_13_samples_every_catalog_family_and_leaves_a_live_gear_with_its_handles`. E `viva` 8, `formas` 8, `desenho` 11, `edit` 20, `selecao` 14, `camadas` 5, `quadros` 4.
- `ph2d-panel-pixel-tools` `seam_pixel` 26 (NOVO `the_shape_flyout_is_the_vector_catalog_in_its_order_with_its_faces`; os dois testes de campos e de Fill passaram das 4 às 49), `the_hint` 2. `ph2d-panel-registry-init` `the_pixel` 4 (NOVO `the_pixel_shapes_are_named_as_in_the_vector_flyout`; o do ponteiro clica as 65 ferramentas). `ph2d-tool-vector` `outline` 3 + `shape_handles` 40; `ph2d-panel-vector` `shape` 29; `ph2d-app-vec` `shape_handles` 22; `ph2d-app-macros` `cenas_do_smoke` 2; `ph2d-i18n` `key` 3; `ph2d-editor-core` `architecture` 118.
- `cargo fmt --all --check` limpo; `cargo-check-narrow ph2d-host-desktop` ✓. **Clippy NÃO corrido** (fecho da linha).

## 6. ABERTO

- **O smoke do dono da P6e (§3).**
- Duas alças da engrenagem estacionam fora da caixa (o `PARK_PX` do Vector, para campos sem lugar na forma) — é a lei do Vector; confirmar no smoke se o dono as lê.
- O painel Pixel mostra «Shape» duas vezes (pincel «Shape» e a secção da forma) — herdado, não da P6e.
- Os abertos da [P6d §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6d.md) (etiqueta do arrasto, `outline_only`, barra X/Y/W/H, gravar/Ctrl+Z com forma viva) e anteriores.
- **Mutações no fecho da LINHA** (somar às da P6b–P6d): o `continue` das abertas em `formas::celulas`; o `&& is_closed()` do `cheia`; o `matches!(RoundRect | SpeechRect)` dos defaults; o `filter` por grupo em `column::members`; o `return y` do Fill nas abertas.

## 7. Próximo item

O smoke do dono da cena 13; depois o que ele pedir.

## Veredito do smoke (dono, 27/10): **OK**

Cena 13 aprovada («Muito bom. Vamos seguir.»), sem reparo às alças estacionadas fora da caixa. Sem
pedido novo: a próxima onda (**P6f**) fecha os abertos da forma viva herdados da
[P6d §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6d.md) — a etiqueta do arrasto do Vector
(`face::readout`), a caixa só-contorno durante o arrasto de uma alça (`outline_only`), a barra da
moldura (X/Y/W/H/ângulo) a mostrar e editar a forma viva, gravar o projecto com uma forma viva
(assenta antes), e Ctrl+Z com forma viva.
