//! **A grelha 3×3 por-região da §5** — irmã de [`super::slice_nine`] por CAP de ficheiro (600).
//!
//! ⚠️ **O corte é por TAMANHO, e o que saiu foi uma peça inteira**: a tabela de células, as
//! letras, a legenda e o pintor. Cortar pelo meio de uma peça é o que faz um ficheiro irmão virar
//! um segundo sítio onde procurar a mesma coisa.
//!
//! A grelha é a **única verdade** sobre o que cada uma das nove regiões faz, desde que o modo
//! `Tiled` foi retirado (2026-08-22): antes dele sair, uma célula em `S` repetia se o Draw Mode
//! fosse `Tiled`, e a letra mentia.

use super::slice_nine::hint;
use super::*;
use ph2d_editor_core::screens::hero::InspectorSliceInfo;

const CELL: f32 = 26.0; // LITERAL-PX-OK: lado de uma célula da grelha 3x3
/// Lado da grelha, em células. Não é uma medida de desenho: é a aridade do 9-slice.
const GRID: usize = 3;

/// **A grelha 3×3 que esta seção pinta: `(coluna, linha)` por região**, na ordem de
/// `SliceRegion::ALL` (TL · T · TR · L · R · BL · B · BR), saltando o miolo.
///
/// ⚠️ **É `pub` para que o gate a possa LER.** A primeira versão deste ficheiro tinha a tabela
/// privada e o gate da shell tinha uma cópia igual — e uma mutação que trocava duas células aqui
/// passava **verde**, porque o gate comparava a sua cópia com o motor em vez de comparar ESTA.
/// *Um gate que guarda a sua própria cópia mede-se a si mesmo.*
pub const REGION_CELLS: [(usize, usize); 8] = [
    (0, 0),
    (1, 0),
    (2, 0),
    (0, 1),
    (2, 1),
    (0, 2),
    (1, 2),
    (2, 2),
];

/// A inicial de cada `TileRegionMode`, tags `0..=3`. ASCII de propósito (sem tofu).
pub const REGION_LETTERS: [&str; 4] = ["S", "R", "M", "-"];

/// As letras de um CANTO — `[desenhado, apagado]`.
///
/// ⚠️ **`F` de FIXO, e não `S` de stretch, porque um canto não estica nem repete.** Ele fica no
/// tamanho intrínseco: é essa a razão de existir do 9-slice. Reaproveitar o `S` ali fazia a
/// legenda («S stretch») afirmar sobre o canto o contrário do que ele faz — e escondia que as
/// suas únicas duas posições são desenhar e não desenhar (auditoria 2026-08-22).
pub const CORNER_LETTERS: [&str; 2] = ["F", "-"];

/// A célula `i` da grelha é um dos quatro cantos? Deriva de [`REGION_CELLS`], nunca de uma
/// segunda cópia da tabela.
pub fn is_corner_cell(i: usize) -> bool {
    REGION_CELLS
        .get(i)
        .is_some_and(|&(col, row)| col != 1 && row != 1)
}

/// A legenda da grelha 3×3.
///
/// ⚠️ **Ela diz duas coisas que a versão anterior escondia** (auditoria 2026-08-22): que os
/// cantos são fixos e só ligam/desligam, e que o miolo não tem `blank` (isso é o `Fill Center`).
/// A terceira ressalva que ela teve por um dia — «em Tiled, S repete» — **deixou de ser
/// precisa**: o `Tiled` foi retirado, e agora a letra diz sempre o que a região faz.
const REGION_LEGEND: &str = "Corners F fixed (on/off). Edges + centre: S stretch, R repeat, \
                             M mirror, - blank.";

/// A grelha 3×3 dos modos por-região. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn region_grid(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorSliceInfo,
) -> f32 {
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        "Per-region tiling",
        x,
        y + (label_h - label_font) * 0.5,
        label_font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let grid_y = y + label_h;
    let gap = Spacing::Xs.px();
    for (i, &(col, row)) in REGION_CELLS.iter().enumerate() {
        let id = ids::INSP_SLICE_REGION[i];
        let rect = Rect::new(
            x + (CELL + gap) * col as f32,
            grid_y + (CELL + gap) * row as f32,
            CELL,
            CELL,
        );
        hit_index.register(id, rect);
        // Divergente na seleção: a célula não afirma modo nenhum.
        let tag = usize::from(info.tile_modes[i]);
        let letter = if info.mixed.tile_modes {
            "?"
        } else if is_corner_cell(i) {
            // Um canto só tem duas posições, e é `F` — fixo — não `S`.
            CORNER_LETTERS[usize::from(tag == 3)]
        } else {
            REGION_LETTERS[tag.min(REGION_LETTERS.len() - 1)]
        };
        let btn = Button::new(id, letter)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(id));
        paint_button(&btn, rect, scene, text_system, theme);
    }
    // **O MIOLO é a nona célula, e é clicável desde 2026-08-22.** Ele era só um rótulo, e por
    // isso espelhar era inalcançável **na maior área ladrilhada** — a que mais mostra a emenda
    // entre dois ladrilhos (smoke do Enio). Com o miolo apagado ele mostra `-` e não age: quem
    // manda nisso é o `Fill Center`, e duas portas para o mesmo estado divergem.
    let mid = Rect::new(x + CELL + gap, grid_y + CELL + gap, CELL, CELL);
    hit_index.register(ids::INSP_SLICE_CENTRE, mid);
    let mid_letter = if !info.fill_center {
        "-"
    } else if info.mixed.tile_modes {
        "?"
    } else {
        REGION_LETTERS[usize::from(info.centre_tile_mode).min(REGION_LETTERS.len() - 1)]
    };
    paint_button(
        &Button::new(ids::INSP_SLICE_CENTRE, mid_letter)
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_SLICE_CENTRE)),
        mid,
        scene,
        text_system,
        theme,
    );
    let grid_h = CELL * GRID as f32 + gap * (GRID - 1) as f32;
    // **Os dois atalhos**, à direita da grelha e à altura dela — é o sítio em que eles se leem
    // como «faz isto às nove», e não como um modo à parte. Eles são a conveniência que o antigo
    // `Tiled` dava, agora a escrever na grelha em vez de a reinterpretar por trás.
    let presets_x = x + (CELL + gap) * GRID as f32 + Spacing::Sm.px();
    let presets_w = (w - (presets_x - x)).max(0.0);
    for (i, (id, label)) in [
        (ids::INSP_SLICE_ALL_TILE, "Tile all"),
        (ids::INSP_SLICE_ALL_STRETCH, "Stretch all"),
    ]
    .into_iter()
    .enumerate()
    {
        let rect = Rect::new(presets_x, grid_y + (CELL + gap) * i as f32, presets_w, CELL);
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, label)
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    let legend_y = grid_y + grid_h + Spacing::Xs.px();
    legend_y + hint(scene, text_system, theme, x, w, legend_y, REGION_LEGEND)
}
