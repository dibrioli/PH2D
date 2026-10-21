//! ⭐ **As barras do quadro** (MiroClone, W1–W2) — a barra CURTA de ferramentas (decisão do dono 4)
//! encostada à esquerda da área, a grelha de todas as formas que ela abre, e a barra de ESTILO que
//! flutua por cima da selecção (preenchimento, contorno, espessura, traço, cantos, opacidade, letra;
//! numa seta, a rota e as pontas em vez do preenchimento e dos cantos).
//!
//! ⚠️ Os ids são DERIVADOS (salto XOR + índice na tabela [`items`]), como os das abas de quadro: a
//! tabela é a única fonte, e `populate` regista-os todos (pintado ⇒ clicável).
//! ⚠️ O ícone de cada forma é o CONTORNO dela (`ph2d_board_geom::outline`) — a mesma geometria que o
//! quadro desenha, nunca um segundo desenho.

use super::HeroScreen;
use crate::interaction::{InteractiveState, WidgetEvent, WidgetStore};
use crate::widget::ButtonState;
use crate::widget::panel_chrome::HIGHLIGHTER_RGBA;
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_board_edit::Tool;
use ph2d_board_model::{Dash, Head, Rgba, Route, ShapeType};
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;

/// ⛔ O salto tem os 16 bits do MEIO diferentes dos das abas de quadro (`…_0000_0003`): os dois
/// lados XOR-am números pequenos, e saltos que só diferem nos bits de baixo COLIDEM (medido: o
/// «mais formas» era a aba do quadro 1).
const SALT: u64 = 0xb0a2_d7ab_5ba2_0000;

/// O balão, a selecção e os ícones (módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_look.rs"]
mod look;
pub use look::shape_name_key;
use look::{
    doc_color, ink, is_current, paint_item, paint_panel, selected_arrow, selected_kind,
    selected_style, selection_box, shown, stroke, swatch, tooltip_key,
};

/// As notas e o texto (W3, módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_notes.rs"]
mod notes;
pub use notes::{flyout_open, flyout_rects};

/// Onde fica cada controlo (módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_layout.rs"]
mod layout;
use layout::{btn_px, gap};
pub use layout::{shapes_rects, style_rects, toolbar_rects};

/// Organizar (W5, módulo filho): o Frame, agrupar, travar, ordem, alinhar e o aviso das recusas.
#[path = "board_bar_organize.rs"]
mod organize;
pub use organize::OrgItem;
#[path = "board_bar_heads.rs"]
mod heads;
#[path = "board_bar_mind.rs"]
mod mind;
pub use mind::MindItem;
#[path = "board_bar_facilitate.rs"]
mod fac;
pub use fac::FacItem;

/// As duas grelhas das formas: a ferramenta e o «Forma» que troca o tipo (módulo filho).
#[path = "board_bar_shapes.rs"]
mod shapes;
pub use shapes::reshape_rects;

/// A caneta e o rascunho (W4, módulo filho: o ficheiro estava no tecto de LOC).
#[path = "board_bar_pen.rs"]
mod pen;
pub use pen::{PenItem, pen_box};

use ph2d_board_edit::{FONT_SIZES, STROKE_WIDTHS as WIDTHS};
/// As opacidades oferecidas (%).
const OPACITIES: [u8; 4] = [25, 50, 75, 100];
/// Os preenchimentos claros (os pastéis do marcador) e as tintas fortes do contorno.
const PASTELS: std::ops::Range<usize> = 0..5;
const VIVIDS: std::ops::Range<usize> = 5..9;
/// O PRETO do contorno (pedido do dono, 08/10: *«para setas e shapes não temos a opção de cor
/// preta»*) — um índice depois das cores do marcador; é preto do DOCUMENTO, nunca a tinta do tema
/// (que se desenha clara num quadro escuro).
const BLACK: usize = HIGHLIGHTER_RGBA.len();
// LITERAL-COLOR-OK: o preto do documento, uma cor do artista.
const BLACK_RGBA: [u8; 4] = [0, 0, 0, 255];

/// A cor de documento `i` das amostras: as do marcador e, a seguir, o preto.
fn palette(i: usize) -> Rgba {
    Rgba(HIGHLIGHTER_RGBA.get(i).copied().unwrap_or(BLACK_RGBA))
}

/// Um controlo das barras.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Item {
    Tool(Tool),
    MoreShapes,
    /// W23 — o «…» do grupo de pontas `i` (`0` início, `1` fim): abre a grelha de todas.
    MoreHeads(usize),
    Pick(ShapeType),
    /// `None` = sem preenchimento; `Some(i)` = o pastel `i` do marcador.
    Fill(Option<usize>),
    /// `None` = sem contorno; `Some(None)` = a tinta do tema; `Some(Some(i))` = a forte `i`.
    Stroke(Option<Option<usize>>),
    Width(usize),
    Dash(Dash),
    Round(bool),
    Opacity(usize),
    Font(usize),
    Route(Route),
    /// `(0 = início · 1 = fim, desenho)`.
    Head(usize, Head),
    /// A cor `i` das 16 das notas do Miro (`STICKY_COLORS`).
    NoteColor(usize),
    /// O tamanho P/M/G das notas (`NOTE_SCALES`).
    NoteSize(usize),
    /// Nota quadrada (`false`) ou larga (`true`).
    NoteWide(bool),
    /// O modo EM MASSA: uma ideia por linha, cada linha uma nota.
    Bulk,
    /// Negrito, itálico, sublinhado, riscado.
    Mark(ph2d_board_model::Mark),
    /// A cor da letra de uma FORMA: `None` = a da forma; `Some(i)` = a forte `i` do marcador.
    TextColor(Option<usize>),
    /// Um controlo da caneta (W4).
    Pen(PenItem),
    /// O botão «Rascunho ↔ Final»: a selecção, ou o quadro inteiro sem selecção.
    Sketch,
    /// Organizar (W5): agrupar, travar, ordem, alinhar, a cor de um frame.
    Organize(OrgItem),
    /// Planear (W6): os modelos e os controlos de um nó de mapa mental.
    Mind(MindItem),
    /// Facilitar (W7): cronómetro, votação, carimbos, agrupar.
    Fac(FacItem),
    /// «Forma»: abre a grelha que troca o TIPO das formas seleccionadas.
    Reshape,
    /// W15 — o preto e branco das imagens seleccionadas (aceso se estão todas).
    Gray,
    /// W16 — espelhar as imagens seleccionadas: `0` na horizontal (Shift+H), `1` na vertical (Shift+V).
    Flip(usize),
    /// W16 — recortar a imagem seleccionada (o mesmo que o duplo-clique ou `Enter`).
    Crop,
    /// W18 — um vídeo do YouTube pelo link da área de transferência (ADR-0185).
    Video,
}

/// As pontas que a barra oferece: as do fluxograma e do diagrama de quadro. (O documento guarda as
/// oito do catálogo — as de UML entram pelo ficheiro, não pela barra estreita.)
const HEADS: [Head; 5] = [
    Head::None,
    Head::Arrow,
    Head::Triangle,
    Head::Circle,
    Head::Bar,
];
const ROUTES: [Route; 3] = [Route::Straight, Route::Elbow, Route::Curved];

/// O que está seleccionado decide que grupos a barra de estilo mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selected {
    Shapes,
    Arrows,
    Both,
    /// Só notas (ou pilhas).
    Notes,
    /// A escrever numa nota.
    TextNote,
    /// A escrever numa forma.
    TextShape,
    /// Só traços da caneta.
    Ink,
    /// Só frames (W5).
    Frames,
    /// Um nó de mapa mental (W6).
    Mind,
    /// Só imagens (W14).
    Images,
    /// Só vídeos (W20).
    Videos,
}

/// Os atalhos da barra curta, na ordem em que se pintam (a Nota logo a seguir à mão: é o coração
/// do brainstorm).
const TOOLBAR: [Item; 18] = [
    Item::Tool(Tool::Select),
    Item::Tool(Tool::Hand),
    Item::Tool(Tool::Shape(ShapeType::Sticky)),
    mind::MIND_TOOL,
    mind::TEMPLATES,
    organize::FRAME_TOOL,
    Item::Tool(Tool::Connector),
    pen::PEN_TOOL,
    Item::Tool(Tool::Shape(ShapeType::Rectangle)),
    Item::Tool(Tool::Shape(ShapeType::Ellipse)),
    Item::Tool(Tool::Shape(ShapeType::Diamond)),
    Item::Tool(Tool::Shape(ShapeType::Triangle)),
    Item::MoreShapes,
    Item::Sketch,
    fac::TIMER,
    fac::VOTE,
    fac::PRIVATE,
    fac::COMMENT,
];

/// ⭐ **A tabela** — todo controlo das barras, por grupo da barra de estilo. O índice é o id.
#[must_use]
pub fn items() -> Vec<Item> {
    let mut v: Vec<Item> = TOOLBAR.to_vec();
    v.extend(ShapeType::ALL.iter().map(|t| Item::Pick(*t)));
    v.extend(notes::flyout_items());
    v.extend(pen::flyout_items());
    v.extend(mind::flyout_items());
    v.extend(fac::flyout_items());
    for it in [
        Selected::Shapes,
        Selected::Arrows,
        Selected::Notes,
        Selected::TextShape,
        Selected::Ink,
        Selected::Frames,
        Selected::Mind,
        Selected::Images,
        Selected::Videos,
    ]
    .into_iter()
    .flat_map(style_groups)
    .flatten()
    {
        if !v.contains(&it)
            && !matches!(
                it,
                Item::Reshape | Item::Gray | Item::Flip(_) | Item::Crop | Item::MoreHeads(_)
            )
        {
            v.push(it);
        }
    }
    // No FIM, pela ordem em que chegaram: os ids dos outros não se movem.
    v.extend([
        shapes::RESHAPE,
        Item::Gray,
        Item::Flip(0),
        Item::Flip(1),
        Item::Crop,
    ]);
    // W18 — no FIM também.
    v.push(Item::Video);
    // W23 — a grelha das pontas, no FIM.
    v.extend(heads::extra_items());
    v
}

/// W18 — a cauda da barra curta (depois do [`TOOLBAR`], para os ids de antes não se moverem).
pub(super) const TOOLBAR_TAIL: [Item; 1] = [Item::Video];

/// Os grupos da barra de estilo, da esquerda para a direita, para o que está seleccionado.
#[must_use]
pub fn style_groups(sel: Selected) -> Vec<Vec<Item>> {
    let fill: Vec<Item> = std::iter::once(Item::Fill(None))
        .chain(PASTELS.map(|i| Item::Fill(Some(i))))
        .collect();
    let stroke: Vec<Item> = [Item::Stroke(None), Item::Stroke(Some(None))]
        .into_iter()
        .chain(VIVIDS.map(|i| Item::Stroke(Some(Some(i)))))
        .chain(std::iter::once(Item::Stroke(Some(Some(BLACK)))))
        .collect();
    let width: Vec<Item> = (0..WIDTHS.len()).map(Item::Width).collect();
    let dash = vec![
        Item::Dash(Dash::Solid),
        Item::Dash(Dash::Dashed),
        Item::Dash(Dash::Dotted),
    ];
    let opacity: Vec<Item> = (0..OPACITIES.len()).map(Item::Opacity).collect();
    let font: Vec<Item> = (0..FONT_SIZES.len()).map(Item::Font).collect();
    let org = organize::groups(sel);
    let mut groups: Vec<Vec<Item>> = match sel {
        Selected::Shapes => vec![
            vec![shapes::RESHAPE],
            fill,
            stroke,
            width,
            dash,
            vec![Item::Round(false), Item::Round(true)],
            opacity,
            font,
        ]
        .into_iter()
        .chain(notes::shape_text_groups())
        .collect(),
        Selected::Arrows => vec![
            // Uma seta sem traço não se vê: o «sem contorno» não entra.
            stroke
                .into_iter()
                .filter(|it| *it != Item::Stroke(None))
                .collect(),
            width,
            dash,
            ROUTES.iter().map(|r| Item::Route(*r)).collect(),
            HEADS
                .iter()
                .map(|h| Item::Head(0, *h))
                .chain([heads::more(0)])
                .collect(),
            HEADS
                .iter()
                .map(|h| Item::Head(1, *h))
                .chain([heads::more(1)])
                .collect(),
            opacity,
            font,
        ],
        Selected::Both => vec![
            stroke
                .into_iter()
                .filter(|it| *it != Item::Stroke(None))
                .collect(),
            width,
            dash,
            opacity,
            font,
        ],
        Selected::Notes => notes::note_groups()
            .into_iter()
            .chain([fac::note_group()])
            .collect(),
        Selected::TextNote => notes::text_groups(false),
        Selected::TextShape => notes::text_groups(true),
        Selected::Ink => pen::ink_groups(),
        Selected::Frames => vec![organize::frame_fill_group(), fac::frame_group()],
        Selected::Mind => return mind::groups(),
        // Os cantos e a opacidade (o que o Excalidraw oferece numa imagem: «Edges» e «Opacity»); a
        // moldura (W15: o contorno, a espessura e o traço das formas) e o preto e branco.
        Selected::Images => vec![
            vec![Item::Round(false), Item::Round(true)],
            stroke,
            width,
            dash,
            opacity,
            vec![Item::Gray],
            vec![Item::Flip(0), Item::Flip(1), Item::Crop],
        ],
        // W20 (o dono: «as opções de decoração como um retângulo») — os grupos do rectângulo que o
        // Excalidraw dá a um `embeddable` (`sonda_w20.mjs` E1: Background, Stroke, width, style, Edges,
        // Opacity). O fundo fica por trás do tocador e a borda à volta dele (`ph2d-board-render::video`).
        Selected::Videos => vec![
            fill,
            stroke,
            width,
            dash,
            vec![Item::Round(false), Item::Round(true)],
            opacity,
        ],
    };
    groups.extend(org);
    groups
}

/// O id do controlo `item`.
#[must_use]
pub fn node(item: Item) -> NodeId {
    let i = items()
        .iter()
        .position(|x| *x == item)
        .expect("está na tabela") as u64;
    NodeId(SALT ^ (i + 1))
}

fn item_of(id: NodeId) -> Option<Item> {
    let i = (id.0 ^ SALT).checked_sub(1)?;
    items().get(usize::try_from(i).ok()?).copied()
}

/// Regista todos os controlos (pintado ⇒ clicável). Chamado pelo `document_tabs::populate`.
pub fn populate(store: &mut WidgetStore) {
    for it in items() {
        store.register(
            node(it),
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
        if let Some(key) = tooltip_key(it) {
            store.set_tooltip(node(it), tr(key));
        }
    }
}

/// ⭐ Pinta as barras do quadro activo. `area` é a área do quadro no ecrã.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    hero: &mut HeroScreen,
    area: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
) {
    let theme = hero.theme;
    let shapes_open = hero.documents.live.shapes_open;
    let templates_open = mind::sync_gallery(&mut hero.documents.live);
    let fac_now = fac::Now::of(hero);
    let Some((board, live)) = hero.documents.active_parts() else {
        return;
    };
    // W23 B15 — o aviso com botão, se é o mais novo, tapa o da recusa.
    let alert_on = super::board_alert::shown(live).is_some();
    let ed = super::board_view::editor(&mut live.editor, theme);
    let tool = ed.tool;
    let sel_style = selected_style(ed, &board.doc);
    let arrow = selected_arrow(ed, &board.doc);
    let what = selected_kind(ed, &board.doc);
    let now = notes::Now::of(ed, &board.doc);
    let org = organize::Now::of(ed, &board.doc);
    if let Some(n) = ed.take_notice() {
        live.notice = Some((n.key(), std::time::Instant::now()));
    }
    // W17 — uma imagem que não se lê avisa na tela (uma vez por ficheiro), nunca só fica partida.
    if !live.render_cache.images.take_failed().is_empty() {
        live.notice = Some(("board.images.unreadable", std::time::Instant::now()));
    }
    let notice = live
        .notice
        .filter(|(_, t)| t.elapsed().as_secs_f32() < crate::toast::Toast::DEFAULT_TTL_S)
        .filter(|_| !alert_on);
    let area_w = super::board_view::area_of(area);
    // A escrever, a barra fica (as marcas do texto); só um arrasto a esconde.
    let sel_box = (!ed.is_busy())
        .then(|| selection_box(ed, &board.doc, &board.camera, area_w))
        .flatten();
    let tools = toolbar_rects(area);
    paint_panel(scene, &tools, theme);
    let note_tool = Item::Tool(Tool::Shape(ShapeType::Sticky));
    let sketch_on = look::sketch_on(ed, &board.doc, board.sketch);
    for (it, r) in tools {
        let on = matches!(it, Item::Tool(t) if t == tool)
            || (it == Item::MoreShapes && shapes_open)
            || (it == note_tool && notes::flyout_open(tool))
            || (it == pen::PEN_TOOL && pen::flyout_open(tool))
            || (it == Item::Sketch && sketch_on)
            || (it == organize::FRAME_TOOL && tool == Tool::Frame)
            || mind::is_on(it, ed, &board.doc, templates_open) == Some(true)
            || fac_now.is_on(it) == Some(true);
        paint_item(
            scene,
            text_system,
            theme,
            &mut hero.hit_index,
            &hero.store,
            it,
            r,
            on,
        );
    }
    if shapes_open {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        shapes::paint_tool_grid(scene, text_system, theme, hit, store, area, tool);
    }
    if notes::flyout_open(tool) {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        notes::paint_flyout(scene, text_system, theme, hit, store, area, tool, &now);
    }
    if pen::flyout_open(tool) {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        pen::paint_flyout(scene, text_system, theme, hit, store, area, ed);
    }
    if templates_open {
        let (hit, store) = (&mut hero.hit_index, &hero.store);
        mind::paint_gallery(scene, text_system, theme, hit, store, area);
    }
    if let Some((key, _)) = notice {
        organize::paint_notice(scene, text_system, theme, area, key);
    }
    fac::sync_flyouts(
        &mut live.fac,
        matches!(what, Some(Selected::Notes | Selected::Frames)),
    );
    let reshape_now = shapes::current(ed, &board.doc);
    let bar_has_reshape = what == Some(Selected::Shapes)
        && shapes::shows(shapes::RESHAPE, ed, &board.doc) == Some(true);
    shapes::sync(live, sel_box.is_some() && bar_has_reshape);
    let reshape_open = live.reshape.is_some();
    heads::sync(live, sel_box.is_some() && what == Some(Selected::Arrows));
    let heads_open = live.heads.as_ref().map(|(_, i)| *i);
    let ed = super::board_view::editor(&mut live.editor, theme);
    let (Some(mut sel), Some(style), Some(what)) = (sel_box, sel_style, what) else {
        return;
    };
    // A pega de RODAR e os pontos azuis ficam acima da moldura: a barra sobe acima deles (na foto
    // de 06/10 tapava a pega).
    let m = super::board_view::metrics();
    let knob = (m.dot + m.handle) as f32;
    sel.y -= knob;
    sel.h += knob;
    mind::sync_tooltip(&mut hero.store, ed, &board.doc);
    let groups: Vec<Vec<Item>> = style_groups(what)
        .into_iter()
        .map(|g| {
            g.into_iter()
                .filter(|it| org.shows(*it) != Some(false))
                .filter(|it| mind::shows(*it, ed, &board.doc) != Some(false))
                .filter(|it| fac::shows(*it, ed, &board.doc) != Some(false))
                .filter(|it| shapes::shows(*it, ed, &board.doc) != Some(false))
                // W16 — o recorte só aparece quando o clique recorta: UMA imagem editável.
                .filter(|it| *it != Item::Crop || ed.can_crop(&board.doc))
                .collect()
        })
        .filter(|g: &Vec<Item>| !g.is_empty())
        .collect();
    let bar = layout::style_rects_of(area, sel, groups);
    paint_panel(scene, &bar, theme);
    for (it, r) in bar.clone() {
        if it == shapes::RESHAPE {
            let (hit, store) = (&mut hero.hit_index, &hero.store);
            shapes::paint_button(scene, hit, store, theme, r, (reshape_open, reshape_now));
            continue;
        }
        let on = (it == Item::Gray)
            .then(|| ed.images_gray(&board.doc) == Some(true))
            .or_else(|| mind::is_on(it, ed, &board.doc, templates_open))
            .or_else(|| fac_now.is_on(it))
            .or_else(|| org.is_on(it, style.fill))
            .or_else(|| notes::is_current(it, Some(&style), &now))
            .or_else(|| pen::is_current(it, ed, Some(style.stroke_width), theme))
            .unwrap_or_else(|| is_current(it, &style, arrow.as_ref(), theme));
        paint_item(
            scene,
            text_system,
            theme,
            &mut hero.hit_index,
            &hero.store,
            it,
            r,
            on,
        );
    }
    let (hit, store) = (&mut hero.hit_index, &hero.store);
    fac::paint_flyout(scene, text_system, theme, hit, store, &bar, fac_now);
    if reshape_open {
        let now = reshape_now;
        shapes::paint_reshape_grid(scene, text_system, theme, hit, store, area, &bar, now);
    }
    if let Some(i) = heads_open {
        let now = arrow.as_ref().map(|c| c.heads[i]);
        heads::paint_grid(scene, text_system, theme, hit, store, area, &bar, (i, now));
    }
}

/// ⭐ Os cliques nas barras. Corre no pré-despacho (ids derivados não chegam a um handler de chrome).
pub fn apply_event(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let (WidgetEvent::Click(id) | WidgetEvent::DoubleClick(id)) = event else {
        return false;
    };
    let Some(it) = item_of(id) else {
        return false;
    };
    let theme = hero.theme;
    let reshaping = shapes::toggle(&mut hero.documents.live, it);
    heads::toggle(&mut hero.documents.live, it);
    if let Item::MoreHeads(_) = it {
        return true;
    }
    mind::toggle_gallery(&mut hero.documents.live, it);
    fac::toggle_flyouts(&mut hero.documents.live, it);
    if fac::apply(hero, it) {
        return true;
    }
    // K1 — escolher outra ferramenta larga a de comentar (uma de cada vez).
    if matches!(
        it,
        Item::Tool(_) | Item::Pick(_) | Item::Pen(pen::PenItem::Kind(_))
    ) {
        super::board_nav::team::disarm_comment_tool(hero);
    }
    if it == Item::Organize(OrgItem::Tag) {
        super::board_nav::open_tag_field(hero);
        return true;
    }
    // W18 — o app lê a área de transferência (com prazo, fora daqui) e põe o vídeo, ou diz porquê não.
    if it == Item::Video {
        hero.documents.live.video_ask = true;
        return true;
    }
    if shapes::apply(hero, it, reshaping) {
        return true;
    }
    let Some((board, live)) = hero.documents.active_parts() else {
        return true;
    };
    let history = live.histories.entry(board.id).or_default();
    let ed = super::board_view::editor(&mut live.editor, theme);
    let center = [board.camera.center_x, board.camera.center_y];
    let (doc, sketch) = (&mut board.doc, &mut board.sketch);
    if notes::apply(ed, doc, history, it, center)
        || pen::apply(ed, doc, history, sketch, theme, it)
        || organize::apply(ed, doc, history, it)
        || mind::apply(ed, doc, history, it, center)
    {
        if let Some(n) = ed.take_notice() {
            live.notice = Some((n.key(), std::time::Instant::now()));
        }
        return true;
    }
    match it {
        Item::Tool(t) => ed.tool = t,
        Item::Fill(_) => {
            // A letra acompanha o fundo: escolher um preenchimento escolhe a tinta que se lê nele
            // (sem fundo, a do tema — a mesma do quadro).
            let c = swatch(it, theme);
            let ink = c.map_or_else(
                || super::board_view::default_style().text_color,
                Rgba::readable_ink,
            );
            ed.set_style(doc, history, |s| {
                s.fill = c;
                s.text_color = ink;
            });
        }
        Item::Stroke(_) => {
            let c = swatch(it, theme);
            ed.set_style(doc, history, |s| s.stroke = c);
        }
        Item::Width(i) => ed.set_style(doc, history, |s| s.choose_width(WIDTHS[i])),
        Item::Dash(d) => ed.set_style(doc, history, |s| s.dash = d),
        Item::Round(r) => ed.set_style(doc, history, |s| s.round = r),
        Item::Opacity(i) => ed.set_style(doc, history, |s| s.opacity = OPACITIES[i]),
        Item::Font(i) => ed.set_font_size(doc, history, FONT_SIZES[i]),
        Item::Route(r) => ed.set_route(doc, history, r),
        Item::Gray => {
            ed.toggle_gray(doc, history);
        }
        Item::Flip(axis) => {
            ed.flip_images(doc, history, axis);
        }
        Item::Crop => {
            ed.begin_crop(doc, history);
        }
        Item::Head(i, h) => ed.set_head(doc, history, i, h),
        Item::MoreShapes
        | Item::MoreHeads(_)
        | Item::Pick(_)
        | Item::Reshape
        | Item::NoteColor(_)
        | Item::NoteSize(_)
        | Item::NoteWide(_)
        | Item::Bulk
        | Item::Mark(_)
        | Item::TextColor(_)
        | Item::Pen(_)
        | Item::Sketch
        | Item::Organize(_)
        | Item::Mind(_)
        | Item::Fac(_)
        | Item::Video => {}
    }
    true
}

#[cfg(test)]
#[path = "board_bar_tests.rs"]
mod tests;
