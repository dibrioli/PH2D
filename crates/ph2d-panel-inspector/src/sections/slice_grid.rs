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
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

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

/// A inicial de cada `TileRegionMode`, tags `0..=3` — **CHAVES, nunca letras**.
///
/// ⚠️⚠️ Até 2026-09-18 estas eram as letras escritas à mão, e o censo desta crate ficava VERDE
/// sobre elas: o `is_language` exige DUAS letras SEGUIDAS (senão acusaria todo identificador),
/// logo uma letra sozinha é invisível para ele **por construção**.
/// ⭐⭐ E elas são INICIAIS de palavras — a legenda da grelha
/// (`panel.inspector.slice.corners_f_fixed_on_off`) explica *«S stretch, R repeat, M mirror»* e
/// já vivia na tabela: *traduzida a legenda e não as letras, ela passava a explicar letras que a
/// grelha nunca mostra.* ⇒ as duas viajam juntas.
/// ⚠️ Em inglês ficam ASCII de propósito (a célula é pequena e um glifo largo não cabe).
pub const REGION_LETTERS: [TextKey; 4] = [
    TextKey::new("panel.inspector.slice.letter_stretch"),
    TextKey::new("panel.inspector.slice.letter_repeat"),
    TextKey::new("panel.inspector.slice.letter_mirror"),
    TextKey::new("panel.inspector.slice.letter_blank"),
];

/// A marca de uma célula cuja selecção é MISTA — ela não afirma modo nenhum.
pub const MIXED_LETTER: TextKey = TextKey::new("panel.inspector.slice.letter_mixed");

/// As letras de um CANTO — `[desenhado, apagado]`.
///
/// ⚠️ **`F` de FIXO, e não `S` de stretch, porque um canto não estica nem repete.** Ele fica no
/// tamanho intrínseco: é essa a razão de existir do 9-slice. Reaproveitar o `S` ali fazia a
/// legenda («S stretch») afirmar sobre o canto o contrário do que ele faz — e escondia que as
/// suas únicas duas posições são desenhar e não desenhar (auditoria 2026-08-22).
pub const CORNER_LETTERS: [TextKey; 2] = [
    TextKey::new("panel.inspector.slice.letter_fixed"),
    TextKey::new("panel.inspector.slice.letter_blank"),
];

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
const REGION_LEGEND: TextKey = TextKey::new("panel.inspector.slice.corners_f_fixed_on_off");

/// O nome da letra de uma célula, para a legenda ao lado da grade: uma fila por fila da grade.
const LEGENDA: [TextKey; GRID] = [
    TextKey::new("panel.inspector.slice.legend_fixed"),
    TextKey::new("panel.inspector.slice.legend_stretch"),
    TextKey::new("panel.inspector.slice.legend_mirror"),
];

/// A letra que uma célula mostra agora.
fn letra_da_celula(i: usize, info: &InspectorSliceInfo) -> TextKey {
    let tag = usize::from(info.tile_modes[i]);
    if info.mixed.tile_modes {
        MIXED_LETTER
    } else if is_corner_cell(i) {
        CORNER_LETTERS[usize::from(tag == 3)]
    } else {
        REGION_LETTERS[tag.min(REGION_LETTERS.len() - 1)]
    }
}

/// A letra do miolo: ele mostra `-` e não age sem o `Fill Center`.
fn letra_do_miolo(info: &InspectorSliceInfo) -> TextKey {
    if !info.fill_center {
        TextKey::new("panel.inspector.slice.letter_blank")
    } else if info.mixed.tile_modes {
        MIXED_LETTER
    } else {
        REGION_LETTERS[usize::from(info.centre_tile_mode).min(REGION_LETTERS.len() - 1)]
    }
}

/// ⭐ **A grade num cartão** (dono 04/11, foto do 9-Slice): células da altura de todo controlo com o vão
/// de toda linha, a legenda ao lado, fila a fila, e os dois atalhos numa fila própria, nas colunas do cartão.
#[allow(clippy::too_many_arguments)]
fn grade_no_cartao(
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
    use ph2d_card_stack::metrics as m;
    let topo = super::rows::rotulo_do_bloco(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        tr("panel.inspector.slice.per_region_tiling"),
    );
    let passo = m::FIELD_H + m::FIELD_ROW_GAP;
    let celula = |col: usize, row: usize| {
        Rect::new(
            x + passo * col as f32,
            topo + passo * row as f32,
            m::FIELD_H,
            m::FIELD_H,
        )
    };
    for (i, &(col, row)) in REGION_CELLS.iter().enumerate() {
        let id = ids::INSP_SLICE_REGION[i];
        let rect = celula(col, row);
        hit_index.register(id, rect);
        paint_button(
            &Button::new(id, letra_da_celula(i, info).tr())
                .kind(ButtonKind::Default)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    let mid = celula(1, 1);
    hit_index.register(ids::INSP_SLICE_CENTRE, mid);
    paint_button(
        &Button::new(ids::INSP_SLICE_CENTRE, letra_do_miolo(info).tr())
            .kind(ButtonKind::Default)
            .visual(store.button_visual(ids::INSP_SLICE_CENTRE)),
        mid,
        scene,
        text_system,
        theme,
    );
    let lx = x + passo * GRID as f32 + m::FIELD_GAP;
    let font = TypeToken::Sm.px();
    for (row, chave) in LEGENDA.iter().enumerate() {
        let c = celula(0, row);
        paint_text(
            text_system,
            scene,
            chave.tr(),
            lx,
            c.y + ph2d_editor_core::paint::folga_do_texto(c.h, font),
            font,
            (x + w - lx).max(0.0),
            resolve(ColorToken::Text3, theme),
        );
    }
    let fundo = topo + passo * GRID as f32 - m::FIELD_ROW_GAP;
    super::rows::fila_de_botoes(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        fundo,
        &[
            (
                true,
                ids::INSP_SLICE_ALL_TILE,
                tr("panel.inspector.slice.tile_all"),
            ),
            (
                true,
                ids::INSP_SLICE_ALL_STRETCH,
                tr("panel.inspector.slice.stretch_all"),
            ),
        ],
    )
}

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
    if ph2d_editor_core::widget::card_ink::card_body().is_some() {
        return grade_no_cartao(scene, text_system, theme, hit_index, store, x, w, y, info);
    }
    // ⛔⛔ Fora do cartão a linha fica com o nome por cima: o controlo é a grelha mais os dois atalhos,
    // à direita dela, e a coluna do controlo só lhes chega com o painel acima de `336 px`.
    let label_font = TypeToken::Sm.px();
    let label_h = label_font + Spacing::Xs.px();
    paint_text(
        text_system,
        scene,
        tr("panel.inspector.slice.per_region_tiling"),
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
        let letter = letra_da_celula(i, info);
        let btn = Button::new(id, letter.tr())
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
    let mid_letter = letra_do_miolo(info);
    paint_button(
        &Button::new(ids::INSP_SLICE_CENTRE, mid_letter.tr())
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
        (
            ids::INSP_SLICE_ALL_TILE,
            tr("panel.inspector.slice.tile_all"),
        ),
        (
            ids::INSP_SLICE_ALL_STRETCH,
            tr("panel.inspector.slice.stretch_all"),
        ),
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
    legend_y
        + hint(
            scene,
            text_system,
            theme,
            x,
            w,
            legend_y,
            REGION_LEGEND.tr(),
        )
}
