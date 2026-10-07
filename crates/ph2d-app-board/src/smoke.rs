//! **As cenas de smoke do QUADRO** — `PH2D_BOARD_SMOKE=<n>`.
//!
//! | n | o que ensina |
//! |---|---|
//! | 1 | as abas: dois quadros já criados e o 1.º aberto, com uma grelha de rectângulos coloridos — a roda dá zoom à volta do cursor, arrastar move a vista, `Scene` devolve a cena |
//! | 3 | as SETAS (W2): o mesmo fluxograma LIGADO por setas CURVAS (as do Miro: presas ao centro, seguem as caixas), o «sim» com rótulo, o «não» que volta por baixo e o «saltar» em arco por cima (presas a pontos fixos, com rótulo), e uma nota ligada a «mais tarde» por uma seta com UM ponto de ajuste, seleccionada — o ponto oco arrasta-se, a bolinha a meio de cada trecho cria outro, duplo-clique apaga; numa caixa, os pontos azuis criam a seguinte já ligada (`Ctrl+→` também) |
//! | 4 | as NOTAS (W3): uma fila de ideias (a do meio com negrito, itálico, sublinhado e riscado), uma nota que CRESCEU com o texto, os tamanhos P · M · G e a larga, as 16 cores do Miro, uma PILHA de onde se arrastam notas, uma forma com uma palavra a vermelho, e um monte desarrumado SELECCIONADO — a pega de quatro pontos (canto de cima à direita) arruma-o em grelha; `N` é a ferramenta Nota, escrever com uma nota seleccionada escreve nela, `Tab` faz a seguinte |
//! | 5 | a CANETA e o RASCUNHO (W4): o fluxograma num quadro em RASCUNHO (formas e setas à mão, preenchimentos às riscas) com desenhos da caneta por cima — uma volta vermelha, o marcador amarelo, um visto — e a caneta já na mão (o painel dela aberto: caneta, marcador, as duas borrachas, o laser, as três predefinições, a cor e a espessura); o botão ondulado ao fundo da barra da esquerda troca o quadro inteiro (ou a selecção) para FINAL e de volta; `P` caneta · `E` borracha · `K` laser |
//! | 2 | as FORMAS (W1): um fluxograma com texto dentro (início → recolher ideias → «boa ideia?» → construir → fim), um passo rodado, um tracejado e um meio transparente; por baixo, o catálogo das 18 formas com o nome de cada uma — seleccionar, mover, redimensionar, rodar, duplo-clique para escrever, a barra curta à esquerda e a de estilo por cima da selecção |

use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, BoardSet, Connector, Dash, Element, ElementId, End, Mark, Rgba,
    RichText, Route, STICKY_COLORS, STICKY_SIDE, STICKY_WIDE, Shape, ShapeType, Style,
};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::screens::hero::board_view::{self, default_style};
use ph2d_editor_core::screens::hero::{board_bar, document_tabs};
use ph2d_editor_core::widget::panel_chrome::HIGHLIGHTER_RGBA;
use ph2d_tokens::{ColorToken, Spacing};

/// A cena 5 (W4), num ficheiro filho.
#[path = "smoke_pen.rs"]
mod pen;

/// O roteador desta família — o maior nível que o `match` de [`stage_armed_smokes`] responde.
pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_BOARD_SMOKE",
    max_level: 5,
}];

/// Encena o que o dono armou por variável de ambiente. Inerte sem ela.
pub fn stage_armed_smokes(hero: &mut HeroScreen) {
    let Some(level) = std::env::var("PH2D_BOARD_SMOKE")
        .ok()
        .and_then(|v| v.trim().parse::<u32>().ok())
    else {
        return;
    };
    match level {
        1 => scene_two_boards(hero),
        2 => scene_shapes(hero),
        3 => scene_arrows(hero),
        4 => scene_notes(hero),
        5 => pen::scene_pen(hero),
        _ => eprintln!("[board] PH2D_BOARD_SMOKE={level}: não há esta cena (1..=5)"),
    }
}

/// Cena 1 — dois quadros, o 1.º com uma grelha de 6×4 rectângulos nas cores de acento do tema.
fn scene_two_boards(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let first = set.create(default_name(1));
    set.create(default_name(2));
    let board = set.get_mut(first).expect("acabou de nascer");
    let tones = [
        ColorToken::Accent,
        ColorToken::Success,
        ColorToken::Warn,
        ColorToken::Danger,
    ];
    let cell = f64::from(Spacing::Xl4.px()) * 2.0;
    let gap = f64::from(Spacing::Xl2.px());
    for row in 0..4_u8 {
        for col in 0..6_u8 {
            let c = tones[usize::from((row + col) % 4)].resolve(hero.theme);
            let mut style = default_style(hero.theme);
            style.fill = Some(Rgba([c.r, c.g, c.b, c.a]));
            style.stroke = None;
            let shape = Shape {
                kind: ShapeType::Rectangle,
                style,
                text: Default::default(),
            };
            let bx = [
                f64::from(col) * (cell + gap) - 3.0 * (cell + gap),
                f64::from(row) * (cell + gap) - 2.0 * (cell + gap),
                cell,
                cell,
            ];
            let el = Element::new_shape(board.doc.mint_id(), board.doc.z_on_top(), shape, bx);
            BoardOp::Put(el).apply(&mut board.doc);
        }
    }
    document_tabs::load(hero, set);
    hero.documents.activate(Some(first));
}

fn default_name(n: usize) -> String {
    ph2d_i18n::tr_with("board.tab.default_name", &[("n", &n)])
}

/// Cena 2 — as formas da W1: um fluxograma com texto e o catálogo das 18 formas.
fn scene_shapes(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let base = default_style(hero.theme);
    let pastel = |i: usize| Some(Rgba(HIGHLIGHTER_RGBA[i]));
    // (forma, texto, preenchimento, x) — a fila do fluxograma, da esquerda para a direita.
    let flow = [
        (ShapeType::Pill, "board.smoke.start", pastel(2), 0.0),
        (
            ShapeType::Rectangle,
            "board.smoke.collect",
            pastel(0),
            220.0,
        ),
        (
            ShapeType::Diamond,
            "board.smoke.good_idea",
            pastel(3),
            440.0,
        ),
        (ShapeType::Rectangle, "board.smoke.build", pastel(4), 680.0),
        (ShapeType::Pill, "board.smoke.end", pastel(1), 900.0),
    ];
    let mut decision = None;
    for (i, (kind, key, fill, x)) in flow.into_iter().enumerate() {
        let mut style = base.clone();
        style.fill = fill;
        style.text_color = fill.map_or(style.text_color, Rgba::readable_ink);
        style.round = kind == ShapeType::Rectangle;
        if i == 3 {
            style.dash = Dash::Dashed;
        }
        if i == 4 {
            style.opacity = 50;
            style.dash = Dash::Dotted;
            style.stroke_width = 3.0;
        }
        let shape = Shape {
            kind,
            style,
            text: ph2d_i18n::tr(key).into(),
        };
        let h = if kind == ShapeType::Diamond {
            140.0
        } else {
            90.0
        };
        let mut el = Element::new_shape(
            board.doc.mint_id(),
            board.doc.z_on_top(),
            shape,
            [x, 70.0 - h / 2.0, 180.0, h],
        );
        if i == 1 {
            el.angle = -0.08;
        }
        if kind == ShapeType::Diamond {
            decision = Some(el.id);
        }
        BoardOp::Put(el).apply(&mut board.doc);
    }
    // O catálogo: as 18 formas em três filas de seis, cada uma com o nome dentro.
    for (i, kind) in ShapeType::ALL.iter().enumerate() {
        let (col, row) = ((i % 6) as f64, (i / 6) as f64);
        let mut style = base.clone();
        style.font_size = 14.0;
        let shape = Shape {
            kind: *kind,
            style,
            text: ph2d_i18n::tr(board_bar::shape_name_key(*kind)).into(),
        };
        let bx = [col * 190.0, 240.0 + row * 140.0, 160.0, 110.0];
        let el = Element::new_shape(board.doc.mint_id(), board.doc.z_on_top(), shape, bx);
        BoardOp::Put(el).apply(&mut board.doc);
    }
    board.camera.center_x = 540.0;
    board.camera.center_y = 300.0;
    board.camera.zoom = 0.9;
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    // O losango abre seleccionado: as pegas e a barra de estilo estão à vista desde o início.
    board_view::select(hero, decision);
}

/// Uma forma com texto no quadro; devolve o id.
fn put_shape(
    doc: &mut BoardDoc,
    kind: ShapeType,
    style: Style,
    key: &str,
    bx: [f64; 4],
) -> ElementId {
    let shape = Shape {
        kind,
        style,
        text: ph2d_i18n::tr(key).into(),
    };
    let el = Element::new_shape(doc.mint_id(), doc.z_on_top(), shape, bx);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

/// Uma seta CURVA (a de nascença, a do Miro) entre duas pontas, pelos `points`, com rótulo
/// (`""` = sem); devolve o id.
fn put_arrow(
    doc: &mut BoardDoc,
    a: End,
    b: End,
    style: &Style,
    label: &str,
    points: &[[f64; 2]],
) -> ElementId {
    let mut c = Connector::new(a, b, Route::Curved, style.clone());
    c.style.fill = None;
    c.waypoints = points.to_vec();
    if !label.is_empty() {
        c.label = ph2d_i18n::tr(label).to_owned();
    }
    let el = Element::new_connector(doc.mint_id(), doc.z_on_top(), c);
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

/// Cena 3 — as setas da W2: o fluxograma da cena 2 LIGADO, com o «não» a voltar atrás e uma nota.
fn scene_arrows(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let base = default_style(hero.theme);
    let doc = &mut board.doc;
    let tinted = |i: usize, kind: ShapeType| {
        let mut s = base.clone();
        let fill = Rgba(HIGHLIGHTER_RGBA[i]);
        s.fill = Some(fill);
        s.text_color = fill.readable_ink();
        s.round = kind == ShapeType::Rectangle;
        s
    };
    // (forma, texto, cor, x, largura, altura) — a fila, da esquerda para a direita, centrada em y = 0.
    let flow = [
        (ShapeType::Pill, "board.smoke.start", 2, 0.0, 160.0, 80.0),
        (
            ShapeType::Rectangle,
            "board.smoke.collect",
            0,
            260.0,
            180.0,
            90.0,
        ),
        (
            ShapeType::Diamond,
            "board.smoke.good_idea",
            3,
            540.0,
            180.0,
            140.0,
        ),
        (
            ShapeType::Rectangle,
            "board.smoke.build",
            4,
            820.0,
            180.0,
            90.0,
        ),
        (ShapeType::Pill, "board.smoke.end", 1, 1100.0, 160.0, 80.0),
    ];
    let ids: Vec<ElementId> = flow
        .iter()
        .map(|&(kind, key, tone, x, w, h)| {
            put_shape(doc, kind, tinted(tone, kind), key, [x, -h / 2.0, w, h])
        })
        .collect();
    let center = |target| End::Bound {
        target,
        anchor: Anchor::Center,
    };
    let fixed = |target, uv| End::Bound {
        target,
        anchor: Anchor::Fixed(uv),
    };
    for (i, w) in ids.windows(2).enumerate() {
        let label = if i == 2 { "board.smoke.yes" } else { "" };
        put_arrow(doc, center(w[0]), center(w[1]), &base, label, &[]);
    }
    // O «saltar»: do topo do início ao topo do construir — um arco por cima das duas do meio.
    let top = |t| fixed(t, [0.5, 0.0]);
    put_arrow(
        doc,
        top(ids[0]),
        top(ids[3]),
        &base,
        "board.smoke.skip",
        &[],
    );
    // O «não»: de baixo do losango ao fundo da recolha — a volta por baixo da fila.
    let bottom = |t| fixed(t, [0.5, 1.0]);
    put_arrow(
        doc,
        bottom(ids[2]),
        bottom(ids[1]),
        &base,
        "board.smoke.no",
        &[],
    );
    // A nota, ligada a «mais tarde» por uma seta com UM ponto de ajuste (seleccionada: o ponto oco
    // e as bolinhas do meio à vista, e a barra de estilo dela abaixo da fila).
    let note = tinted(2, ShapeType::Rectangle);
    let n = put_shape(
        doc,
        ShapeType::Rectangle,
        note,
        "board.smoke.notes",
        [260.0, 300.0, 300.0, 80.0],
    );
    let later = tinted(1, ShapeType::Pill);
    let l = put_shape(
        doc,
        ShapeType::Pill,
        later,
        "board.smoke.later",
        [900.0, 300.0, 160.0, 80.0],
    );
    let bent = put_arrow(doc, center(n), center(l), &base, "", &[[730.0, 450.0]]);
    board.camera.center_x = 630.0;
    board.camera.center_y = 90.0;
    board.camera.zoom = 0.85;
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    // A seta com o ponto de ajuste abre seleccionada: o ponto oco e as bolinhas do meio à vista.
    board_view::select(hero, [bent]);
}

/// Uma nota de cor `c` (índice das 16), do tipo `kind`, com o canto em `[x, y]` e largura `w`; a
/// letra escala com a largura (a M tem a de nascença). Devolve o id.
fn put_note(
    doc: &mut BoardDoc,
    kind: ShapeType,
    c: usize,
    text: RichText,
    [x, y, w]: [f64; 3],
) -> ElementId {
    let fill = Rgba(STICKY_COLORS[c]);
    let base = if kind == ShapeType::StickyWide {
        STICKY_WIDE
    } else {
        STICKY_SIDE
    };
    let mut style = Style::new(Some(fill), None, fill.readable_ink());
    style.font_size = ph2d_board_model::DEFAULT_FONT_SIZE * w / base;
    let h = w * kind.note_aspect().unwrap_or(1.0);
    let el = Element::new_shape(
        doc.mint_id(),
        doc.z_on_top(),
        Shape { kind, style, text },
        [x, y, w, h],
    );
    let id = el.id;
    BoardOp::Put(el).apply(doc);
    id
}

fn note_text(key: &str) -> RichText {
    RichText::plain(ph2d_i18n::tr(key))
}

/// Cena 4 — as NOTAS da W3 (o coração do brainstorm), no idioma do Miro.
fn scene_notes(hero: &mut HeroScreen) {
    let mut set = BoardSet::default();
    let id = set.create(default_name(1));
    let board = set.get_mut(id).expect("acabou de nascer");
    let doc = &mut board.doc;
    let m = STICKY_SIDE;
    let gap = m * 0.1;
    let (sticky, wide) = (ShapeType::Sticky, ShapeType::StickyWide);
    // A fila de ideias (a do meio ensina as quatro marcas do texto, uma por linha).
    let mut marked = String::new();
    let mut ranges = Vec::new();
    for (i, (key, mark)) in [
        ("board.smoke.note.bold", Mark::Bold),
        ("board.smoke.note.italic", Mark::Italic),
        ("board.smoke.note.underline", Mark::Underline),
        ("board.smoke.note.strike", Mark::Strike),
    ]
    .into_iter()
    .enumerate()
    {
        if i > 0 {
            marked.push('\n');
        }
        let start = marked.len();
        marked.push_str(ph2d_i18n::tr(key));
        ranges.push((start..marked.len(), mark));
    }
    let mut rich = RichText::plain(&marked);
    for (r, mark) in ranges {
        rich.toggle(r, mark);
    }
    let row = [
        note_text("board.smoke.note.type"),
        rich,
        note_text("board.smoke.note.tab"),
    ];
    let mut fit = Vec::new();
    for (i, text) in row.into_iter().enumerate() {
        let x = i as f64 * (m + gap);
        fit.push(put_note(doc, sticky, 1, text, [x, 0.0, m]));
    }
    // A que CRESCEU para baixo com o texto (a decisão 2 do dono).
    let grown = put_note(
        doc,
        sticky,
        8,
        note_text("board.smoke.note.grows"),
        [3.0 * (m + gap) + 2.0 * gap, 0.0, m],
    );
    fit.push(grown);
    // A pilha (laranja), com o que ela faz escrito por baixo.
    let sx = 4.0 * (m + gap) + 4.0 * gap;
    put_note(
        doc,
        ShapeType::StickyStack,
        3,
        RichText::default(),
        [sx, 0.0, m],
    );
    let mut hint = default_style(hero.theme);
    hint.stroke = None;
    put_shape(
        doc,
        ShapeType::Rectangle,
        hint.clone(),
        "board.smoke.note.stack",
        [sx - gap, m + 2.0 * gap, m + 3.0 * gap, 70.0],
    );
    // Os tamanhos P · M · G e a larga, alinhados por baixo.
    let y2 = 2.2 * m;
    let mut x = 0.0;
    for (scale, key) in [
        (0.75, "board.smoke.note.small"),
        (1.0, "board.smoke.note.medium"),
        (1.5, "board.smoke.note.large"),
    ] {
        let w = m * scale;
        put_note(doc, sticky, 4, note_text(key), [x, y2, w]);
        x += w + gap;
    }
    put_note(
        doc,
        wide,
        12,
        note_text("board.smoke.note.wide"),
        [x, y2, STICKY_WIDE],
    );
    // As 16 cores do Miro, pequenas, em duas linhas de oito.
    let y3 = y2 + 1.5 * m + 3.0 * gap;
    let small = m * 0.5;
    for c in 0..STICKY_COLORS.len() {
        let (col, r) = ((c % 8) as f64, (c / 8) as f64);
        put_note(
            doc,
            sticky,
            c,
            RichText::default(),
            [col * (small + gap), y3 + r * (small + gap), small],
        );
    }
    // Uma FORMA guarda a cor da letra por trecho (as notas do Miro não).
    let mut shape_style = default_style(hero.theme);
    shape_style.round = true;
    let s = put_shape(
        doc,
        ShapeType::Rectangle,
        shape_style,
        "board.smoke.note.red_word",
        [sx + m + 4.0 * gap, 0.0, 260.0, 110.0],
    );
    if let Some(sh) = doc.get(s).and_then(|e| e.shape()).cloned() {
        let mut sh = sh;
        let word = ph2d_i18n::tr("board.smoke.note.red");
        if let Some(at) = sh.text.as_str().find(word) {
            let red = Some(Rgba(STICKY_COLORS[11]));
            sh.text.restyle(at..at + word.len(), |mk| mk.color = red);
            let mut el = doc.get(s).cloned().expect("acabou de entrar");
            el.kind = ph2d_board_model::ElementKind::Shape(sh);
            BoardOp::Put(el).apply(doc);
        }
    }
    // O monte DESARRUMADO, seleccionado: a pega de quatro pontos arruma-o.
    let mx = sx + m + 4.0 * gap;
    let my = 1.3 * m;
    let messy = [
        (0.0, 30.0, 6, "board.smoke.note.m1"),
        (230.0, -10.0, 9, "board.smoke.note.m2"),
        (480.0, 60.0, 13, "board.smoke.note.m3"),
        (60.0, 260.0, 5, "board.smoke.note.m4"),
        (330.0, 230.0, 10, "board.smoke.note.m5"),
        (560.0, 300.0, 7, "board.smoke.note.m6"),
    ];
    let pile: Vec<ElementId> = messy
        .iter()
        .map(|&(dx, dy, c, key)| {
            put_note(doc, sticky, c, note_text(key), [mx + dx, my + dy, m * 0.9])
        })
        .collect();
    fit.extend(pile.iter().copied());
    board.camera.center_x = 980.0;
    board.camera.center_y = 430.0;
    board.camera.zoom = 0.62;
    document_tabs::load(hero, set);
    hero.documents.activate(Some(id));
    board_view::fit_later(hero, fit);
    board_view::select(hero, pile);
}
