//! **Os modelos** (W6, T1–T6 de `docs/MiroClone/04_planear_mapa_e_modelos.md` §2): começar um
//! planeamento a partir de um modelo. Cada um é feito das peças que o quadro já tem — frames, notas,
//! formas, setas presas e um mapa mental — com o conteúdo dos modelos do Miro (as páginas dos modelos,
//! `ferramentas/planear_pesquisa_miro_e_apps.md` §modelos), à escala do mundo (as notas de 199).
//! Entra no CENTRO da vista, seleccionado, num passo de desfazer; o texto vem de quem chama (o
//! `ph2d-i18n`, inglês como o resto da interface).

use std::collections::BTreeSet;

use crate::Editor;
use ph2d_board_model::{
    Anchor, BoardDoc, BoardOp, Connector, Element, ElementId, End, FRAME_FILL, FracKey, Frame,
    History, MIND_COLORS, MIND_ROOT_STROKE, Mind, Rgba, RichText, Route, STICKY_COLORS,
    STICKY_SIDE, STICKY_WIDE, Shape, ShapeType, Side, Style,
};

/// Um modelo da galeria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    Retrospective,
    Brainstorm,
    Kanban,
    MindMap,
    Flowchart,
    Journey,
}

impl Template {
    /// Os seis, pela ordem da galeria.
    pub const ALL: [Template; 6] = [
        Template::Retrospective,
        Template::Brainstorm,
        Template::Kanban,
        Template::MindMap,
        Template::Flowchart,
        Template::Journey,
    ];

    /// A chave i18n do nome (a galeria e o frame que o leva).
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Template::Retrospective => "board.tpl.retro",
            Template::Brainstorm => "board.tpl.brainstorm",
            Template::Kanban => "board.tpl.kanban",
            Template::MindMap => "board.tpl.mindmap",
            Template::Flowchart => "board.tpl.flowchart",
            Template::Journey => "board.tpl.journey",
        }
    }
}

/// O vão entre peças de um modelo (e a margem dentro de um frame), em unidades do mundo.
const GAP: f64 = 40.0;
/// A altura de um título ou pergunta dentro de um frame.
const LABEL_H: f64 = 60.0;
/// A letra dos títulos, das perguntas e dos rótulos (a «M» do quadro).
const LABEL_FONT: f64 = ph2d_board_model::DEFAULT_FONT_SIZE;
/// A cor de cada coluna da retrospectiva (Start · Stop · Continue · Action items), índices de
/// [`STICKY_COLORS`]: verde-claro, rosa, azul-claro, amarelo — as do Miro, por coluna.
const RETRO_COLORS: [usize; 4] = [4, 8, 12, 1];
/// Uma cor por pessoa no brainwriting (amarelo, verde, azul, violeta).
const PEOPLE_COLORS: [usize; 4] = [1, 4, 12, 10];
/// Uma cor por estágio no mapa de jornada.
const STAGE_COLORS: [usize; 4] = [12, 4, 1, 8];

/// O construtor: ids e chaves de z novos, por ordem (o primeiro fica atrás).
struct Build<'a> {
    doc: &'a mut BoardDoc,
    z: FracKey,
    els: Vec<Element>,
    ink: Rgba,
}

impl Build<'_> {
    fn next_z(&mut self) -> FracKey {
        self.z = self.doc.key_between(Some(&self.z), None);
        self.z.clone()
    }

    fn frame(&mut self, name: String, bx: [f64; 4]) -> ElementId {
        let fill = Rgba(FRAME_FILL);
        let f = Frame {
            name,
            style: Style::new(Some(fill), None, fill.readable_ink()),
        };
        let (id, z) = (self.doc.mint_id(), self.next_z());
        self.els.push(Element::new_frame(id, z, f, bx));
        id
    }

    fn shape(
        &mut self,
        kind: ShapeType,
        style: Style,
        text: &str,
        bx: [f64; 4],
        home: Option<ElementId>,
    ) -> ElementId {
        let shape = Shape {
            kind,
            style,
            text: RichText::plain(text),
        };
        let (id, z) = (self.doc.mint_id(), self.next_z());
        let mut el = Element::new_shape(id, z, shape, bx);
        el.frame = home;
        self.els.push(el);
        id
    }

    /// Um título/pergunta: só texto.
    fn label(&mut self, text: &str, bx: [f64; 4], home: Option<ElementId>) -> ElementId {
        let mut style = Style::new(None, None, self.ink);
        style.font_size = LABEL_FONT;
        self.shape(ShapeType::Rectangle, style, text, bx, home)
    }

    /// Uma nota da cor `c` (índice de [`STICKY_COLORS`]) com o canto em `[x, y]`.
    fn note(
        &mut self,
        c: usize,
        text: &str,
        [x, y]: [f64; 2],
        wide: bool,
        home: Option<ElementId>,
    ) -> ElementId {
        let color = Rgba(STICKY_COLORS[c]);
        let (kind, w) = if wide {
            (ShapeType::StickyWide, STICKY_WIDE)
        } else {
            (ShapeType::Sticky, STICKY_SIDE)
        };
        let mut style = Style::new(Some(color), None, color.readable_ink());
        style.font_size = crate::notes::note_font(kind, w);
        self.shape(kind, style, text, [x, y, w, STICKY_SIDE], home)
    }

    /// Uma forma do fluxograma centrada em `c`.
    fn flow(&mut self, kind: ShapeType, text: &str, c: [f64; 2], style: &Style) -> ElementId {
        let [w, h] = crate::CLICK_SIZE;
        self.shape(
            kind,
            style.clone(),
            text,
            [c[0] - w / 2.0, c[1] - h / 2.0, w, h],
            None,
        )
    }

    /// Uma seta presa ao centro de `a` e de `b`.
    fn arrow(
        &mut self,
        a: ElementId,
        b: ElementId,
        route: ph2d_board_model::Route,
        label: String,
        style: &Style,
    ) {
        let bound = |target| End::Bound {
            target,
            anchor: Anchor::Center,
        };
        let mut c = Connector::new(bound(a), bound(b), route, style.clone());
        c.captions = Connector::captions_of(label);
        let (id, z) = (self.doc.mint_id(), self.next_z());
        self.els.push(Element::new_connector(id, z, c));
    }
}

impl Editor {
    /// ⭐ Insere o modelo `t` centrado em `center`, seleccionado — UM passo. `tr` traduz as chaves
    /// `board.tpl.*`. Devolve os ids seleccionados.
    pub fn insert_template(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        t: Template,
        center: [f64; 2],
        tr: &dyn Fn(&'static str) -> String,
    ) -> Vec<ElementId> {
        self.commit_text(doc, history);
        let z = doc.z_on_top();
        let mut b = Build {
            doc,
            z,
            els: Vec::new(),
            ink: self.style.text_color,
        };
        let picked = match t {
            Template::Retrospective => retro(&mut b, tr),
            Template::Brainstorm => brainstorm(&mut b, tr),
            Template::Kanban => kanban(&mut b, tr),
            Template::Flowchart => flowchart(&mut b, tr, &self.style),
            Template::Journey => journey(&mut b, tr),
            Template::MindMap => self.mind_template(&mut b, tr),
        };
        let mut els = b.els;
        // O mapa mental arruma-se ANTES de centrar (num documento de rascunho com só ele): senão o
        // centro era o dos nós empilhados na raiz (auditoria W6).
        if els.iter().any(|e| e.mind().is_some()) {
            let mut scratch = BoardDoc::default();
            let _ = ph2d_board_model::apply_batch(
                &mut scratch,
                els.iter().cloned().map(BoardOp::Put).collect(),
            );
            for el in &mut els {
                if let Some(laid) = scratch.get(el.id) {
                    (el.x, el.y) = (laid.x, laid.y);
                }
            }
        }
        let boxes: Vec<[f64; 4]> = els
            .iter()
            .filter(|e| e.connector().is_none())
            .map(Element::aabb)
            .collect();
        let bb = boxes.iter().fold(
            [
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ],
            |a, b| {
                [
                    a[0].min(b[0]),
                    a[1].min(b[1]),
                    a[2].max(b[2]),
                    a[3].max(b[3]),
                ]
            },
        );
        let d = [
            center[0] - (bb[0] + bb[2]) / 2.0,
            center[1] - (bb[1] + bb[3]) / 2.0,
        ];
        // M18: o mapa do modelo mora no frame onde é inserido (os nós nasceram longe dele).
        let home = doc.deepest_frame_at(center, &BTreeSet::new());
        for el in &mut els {
            if el.connector().is_none() {
                el.translate(d);
            }
            if el.mind().is_some() {
                el.frame = home;
            }
        }
        history.apply(doc, els.into_iter().map(BoardOp::Put).collect());
        self.tool = crate::Tool::Select;
        self.selection = picked.iter().copied().collect::<BTreeSet<_>>();
        picked
    }

    /// T4: uma raiz com quatro tópicos (dois de cada lado) e dois subtópicos no primeiro — o layout
    /// do documento arruma-os.
    fn mind_template(
        &mut self,
        b: &mut Build,
        tr: &dyn Fn(&'static str) -> String,
    ) -> Vec<ElementId> {
        let mut node = |b: &mut Build, mind: Mind, text: String| {
            let mut el = self.mind_node_unfitted(b.doc, mind, [0.0, 0.0], &text);
            el.z = b.next_z();
            let id = el.id;
            b.els.push(el);
            id
        };
        let root_mind = Mind::new(
            None,
            FracKey::between(None, None),
            Side::After,
            Rgba(MIND_ROOT_STROKE),
        );
        let root = node(b, root_mind, tr("board.tpl.mindmap.root"));
        let mut order = FracKey::between(None, None);
        let mut first = None;
        for (i, color) in MIND_COLORS.iter().take(4).enumerate() {
            order = b.doc.key_between(Some(&order), None);
            let side = if i % 2 == 0 {
                Side::After
            } else {
                Side::Before
            };
            let m = Mind::new(Some(root), order.clone(), side, Rgba(*color));
            let id = node(b, m, format!("{} {}", tr("board.tpl.mindmap.topic"), i + 1));
            first.get_or_insert(id);
        }
        let first = first.expect("quatro tópicos");
        let mut order = FracKey::between(None, None);
        for i in 0..2 {
            order = b.doc.key_between(Some(&order), None);
            let m = Mind::new(
                Some(first),
                order.clone(),
                Side::After,
                Rgba(MIND_COLORS[0]),
            );
            node(
                b,
                m,
                format!("{} {}", tr("board.tpl.mindmap.subtopic"), i + 1),
            );
        }
        vec![root]
    }
}

/// T1: Start · Stop · Continue · Action items — quatro frames lado a lado, cada um com a pergunta e
/// quatro notas vazias da cor da coluna.
fn retro(b: &mut Build, tr: &dyn Fn(&'static str) -> String) -> Vec<ElementId> {
    let cols = [
        ("board.tpl.retro.start", "board.tpl.retro.q_start"),
        ("board.tpl.retro.stop", "board.tpl.retro.q_stop"),
        ("board.tpl.retro.continue", "board.tpl.retro.q_continue"),
        ("board.tpl.retro.actions", "board.tpl.retro.q_actions"),
    ];
    let fw = 2.0 * STICKY_SIDE + 3.0 * GAP;
    let fh = LABEL_H + 2.0 * STICKY_SIDE + 4.0 * GAP;
    let mut out = Vec::new();
    for (i, (name, q)) in cols.iter().enumerate() {
        let x = i as f64 * (fw + GAP);
        let f = b.frame(tr(name), [x, 0.0, fw, fh]);
        b.label(&tr(q), [x + GAP, GAP, fw - 2.0 * GAP, LABEL_H], Some(f));
        for k in 0..4 {
            let (c, r) = ((k % 2) as f64, (k / 2) as f64);
            let at = [
                x + GAP + c * (STICKY_SIDE + GAP),
                2.0 * GAP + LABEL_H + r * (STICKY_SIDE + GAP),
            ];
            b.note(RETRO_COLORS[i], "", at, false, Some(f));
        }
        out.push(f);
    }
    out
}

/// T2: brainwriting — um frame, uma coluna por pessoa (o nome por cima), três ideias cada.
fn brainstorm(b: &mut Build, tr: &dyn Fn(&'static str) -> String) -> Vec<ElementId> {
    let n = PEOPLE_COLORS.len() as f64;
    let fw = n * STICKY_SIDE + (n + 1.0) * GAP;
    let fh = LABEL_H + 3.0 * STICKY_SIDE + 5.0 * GAP;
    let f = b.frame(tr("board.tpl.brainstorm.frame"), [0.0, 0.0, fw, fh]);
    for (i, c) in PEOPLE_COLORS.iter().enumerate() {
        let x = GAP + i as f64 * (STICKY_SIDE + GAP);
        let who = format!("{} {}", tr("board.tpl.brainstorm.participant"), i + 1);
        b.label(&who, [x, GAP, STICKY_SIDE, LABEL_H], Some(f));
        for r in 0..3 {
            let y = 2.0 * GAP + LABEL_H + r as f64 * (STICKY_SIDE + GAP);
            b.note(*c, "", [x, y], false, Some(f));
        }
    }
    vec![f]
}

/// T3: kanban — Backlog · In progress · Done, raias de duas equipas (os rótulos à esquerda).
fn kanban(b: &mut Build, tr: &dyn Fn(&'static str) -> String) -> Vec<ElementId> {
    let cols = [
        "board.tpl.kanban.backlog",
        "board.tpl.kanban.progress",
        "board.tpl.kanban.done",
    ];
    let lane = STICKY_SIDE + GAP;
    let fw = STICKY_WIDE + 2.0 * GAP;
    let fh = LABEL_H + 2.0 * lane + GAP;
    let lx = -(STICKY_WIDE / 2.0 + GAP);
    let mut out = Vec::new();
    for (r, team) in ["A", "B"].iter().enumerate() {
        let y = LABEL_H + GAP + r as f64 * lane;
        let name = format!("{} {team}", tr("board.tpl.kanban.team"));
        out.push(b.label(&name, [lx, y, STICKY_WIDE / 2.0, STICKY_SIDE], None));
    }
    for (i, name) in cols.iter().enumerate() {
        let x = i as f64 * (fw + GAP);
        let f = b.frame(tr(name), [x, 0.0, fw, fh]);
        let cards = [2, 1, 0][i];
        for r in 0..cards {
            let y = LABEL_H + GAP + r as f64 * lane;
            b.note(1, "", [x + GAP, y], true, Some(f));
        }
        out.push(f);
    }
    out
}

/// T5: fluxograma — Start → Process → Decision →(Yes) Process → End, e (No) → Fix → de volta ao
/// primeiro Process; setas em cotovelo presas ao centro.
fn flowchart(b: &mut Build, tr: &dyn Fn(&'static str) -> String, style: &Style) -> Vec<ElementId> {
    let step = crate::CLICK_SIZE[1] + crate::NEXT_GAP;
    let side = crate::CLICK_SIZE[0] + crate::NEXT_GAP;
    let start = b.flow(
        ShapeType::Pill,
        &tr("board.tpl.flow.start"),
        [0.0, 0.0],
        style,
    );
    let p1 = b.flow(
        ShapeType::Rectangle,
        &tr("board.tpl.flow.process"),
        [0.0, step],
        style,
    );
    let dec = b.flow(
        ShapeType::Diamond,
        &tr("board.tpl.flow.decision"),
        [0.0, 2.0 * step],
        style,
    );
    let p2 = b.flow(
        ShapeType::Rectangle,
        &tr("board.tpl.flow.process"),
        [0.0, 3.0 * step],
        style,
    );
    let end = b.flow(
        ShapeType::Pill,
        &tr("board.tpl.flow.end"),
        [0.0, 4.0 * step],
        style,
    );
    let fix = b.flow(
        ShapeType::Rectangle,
        &tr("board.tpl.flow.fix"),
        [side, 2.0 * step],
        style,
    );
    let mut out = vec![start, p1, dec, p2, end, fix];
    let first = b.els.len();
    for (a, c, label) in [
        (start, p1, String::new()),
        (p1, dec, String::new()),
        (dec, p2, tr("board.tpl.flow.yes")),
        (p2, end, String::new()),
        (dec, fix, tr("board.tpl.flow.no")),
        (fix, p1, String::new()),
    ] {
        b.arrow(a, c, Route::Elbow, label, style);
    }
    out.extend(b.els[first..].iter().map(|e| e.id));
    out
}

/// T6: mapa de jornada — um frame com a Persona, quatro estágios em colunas e cinco linhas
/// (Actions, Touchpoints, Emotions, Pain points, Opportunities), uma nota por célula da cor do estágio.
fn journey(b: &mut Build, tr: &dyn Fn(&'static str) -> String) -> Vec<ElementId> {
    let stages = [
        "board.tpl.journey.stage1",
        "board.tpl.journey.stage2",
        "board.tpl.journey.stage3",
        "board.tpl.journey.stage4",
    ];
    let rows = [
        "board.tpl.journey.row1",
        "board.tpl.journey.row2",
        "board.tpl.journey.row3",
        "board.tpl.journey.row4",
        "board.tpl.journey.row5",
    ];
    let cell = STICKY_SIDE + GAP;
    let label_w = STICKY_SIDE;
    let top = 2.0 * GAP + STICKY_SIDE + LABEL_H;
    let fw = GAP + label_w + GAP + stages.len() as f64 * cell;
    let fh = top + rows.len() as f64 * cell + GAP;
    let f = b.frame(tr("board.tpl.journey"), [0.0, 0.0, fw, fh]);
    b.note(
        10,
        &tr("board.tpl.journey.persona"),
        [GAP, GAP],
        false,
        Some(f),
    );
    let x0 = 2.0 * GAP + label_w;
    for (i, s) in stages.iter().enumerate() {
        let x = x0 + i as f64 * cell;
        b.label(
            &tr(s),
            [x, GAP + STICKY_SIDE + GAP / 2.0, STICKY_SIDE, LABEL_H],
            Some(f),
        );
        for r in 0..rows.len() {
            b.note(
                STAGE_COLORS[i],
                "",
                [x, top + r as f64 * cell],
                false,
                Some(f),
            );
        }
    }
    for (r, k) in rows.iter().enumerate() {
        b.label(
            &tr(k),
            [GAP, top + r as f64 * cell, label_w, STICKY_SIDE],
            Some(f),
        );
    }
    vec![f]
}

#[cfg(test)]
#[path = "templates_tests.rs"]
mod tests;
