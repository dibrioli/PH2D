//! As leis do mapa mental no documento (W6, `docs/MiroClone/04_planear_mapa_e_modelos.md` §2).
//!
//! Números do Miro — a imagem oficial do modelo «Mind Map» (`Mind_Map_View-thumb-web.png`, 1440×960,
//! letra 14) medida ao píxel (04 §1.3), centros em y:
//!
//! | nó | y | da raiz (472,5) |
//! |---|---|---|
//! | «Use cases» (7 folhas, 175…499) | 337 | −135,5 |
//! | «More than a mind map» (4 folhas, 553…716) | 634,5 | +162 |
//! | «Availability» (1 folha, 770) | 770 | +297,5 |
//! | «Robust collaborative features» (6 folhas) | 337 | −135 (o lado direito centra-se por si) |
//! | «Mind mapping features» (Different themes ×2, mais 3) | 634 | +162 |
//!
//! Passo 54 entre folhas e entre primos; 122 de texto a texto entre níveis (104 de caixa a caixa).
//! ⛔ Divergências escolhidas, com o porquê no estudo §3: o vão da prof. 1 dos oráculos (61–67 no
//! mind-elixir, 44 no simple-mind-map) — o Miro mede 54 a qualquer profundidade; recolher sem
//! rearrumar (os dois oráculos rearrumam).
//! Tolerância: 1 px contra o Miro (o nosso passo de uma linha é 53,94, a letra dele é outra); `R`
//! nas nossas próprias contas.

use std::collections::BTreeSet;

use super::*;
use crate::{History, Shape, ShapeType, Style};

const R: f64 = 1e-9;
/// A altura de um nó de uma linha: a linha de 16,94 do nosso moldado a 14 + as duas folgas.
const LINE_H: f64 = 16.939_453 + 2.0 * MIND_PAD[1];

struct Map {
    doc: BoardDoc,
    h: History,
}

fn ink() -> Rgba {
    Rgba([30, 30, 30, 255])
}

fn shape(kind: ShapeType) -> Shape {
    let mut style = Style::new(None, None, ink());
    style.font_size = MIND_FONT;
    Shape {
        kind,
        style,
        text: crate::RichText::default(),
    }
}

impl Map {
    /// Uma raiz pílula 166 × 66 com o centro em (0, 0).
    fn new() -> (Self, ElementId) {
        let mut doc = BoardDoc::default();
        let id = doc.mint_id();
        let mut el = Element::new_shape(
            id,
            doc.z_on_top(),
            shape(ShapeType::Pill),
            [-83.0, -33.0, 166.0, 66.0],
        );
        el.set_mind(Some(Mind::new(
            None,
            FracKey::between(None, None),
            Side::After,
            ink(),
        )));
        let mut m = Self {
            doc,
            h: History::default(),
        };
        m.h.apply(&mut m.doc, vec![BoardOp::Put(el)]);
        (m, id)
    }

    /// Um nó de `w × h` debaixo de `parent`, o último dos irmãos, nascido longe (o layout põe-no).
    fn add_sized(&mut self, parent: ElementId, side: Side, w: f64, h: f64) -> ElementId {
        let last = self
            .doc
            .mind_children(parent)
            .last()
            .and_then(|c| self.doc.get(*c))
            .and_then(|e| e.mind())
            .map(|m| m.order.clone());
        let id = self.doc.mint_id();
        let mut el = Element::new_shape(
            id,
            self.doc.z_on_top(),
            shape(ShapeType::Rectangle),
            [9_999.0, 9_999.0, w, h],
        );
        el.set_mind(Some(Mind::new(
            Some(parent),
            FracKey::between(last.as_ref(), None),
            side,
            ink(),
        )));
        self.h.apply(&mut self.doc, vec![BoardOp::Put(el)]);
        id
    }

    fn add(&mut self, parent: ElementId, side: Side) -> ElementId {
        self.add_sized(parent, side, 100.0, LINE_H)
    }

    fn el(&self, id: ElementId) -> &Element {
        self.doc.get(id).expect("vivo")
    }

    fn cy(&self, id: ElementId) -> f64 {
        self.el(id).center()[1]
    }

    fn edit(&mut self, id: ElementId, f: impl FnOnce(&mut Element)) {
        let mut el = self.el(id).clone();
        f(&mut el);
        self.h.apply(&mut self.doc, vec![BoardOp::Put(el)]);
    }
}

/// ⭐ M1–M4: o mapa oficial do Miro, refeito nó a nó, cai a < 1 px dos centros medidos — o pai no
/// meio da faixa, cada lado da raiz centrado nela por si, o passo de 54 entre folhas e entre primos.
#[test]
fn the_miro_template_map_lands_where_the_miro_draws_it() {
    let (mut m, root) = Map::new();
    let use_cases = m.add(root, Side::Before);
    let more = m.add(root, Side::Before);
    let avail = m.add(root, Side::Before);
    let robust = m.add(root, Side::After);
    let features = m.add(root, Side::After);
    let leaves = |m: &mut Map, p: ElementId, n: usize| -> Vec<ElementId> {
        (0..n).map(|_| m.add(p, Side::After)).collect()
    };
    let uc = leaves(&mut m, use_cases, 7);
    leaves(&mut m, more, 4);
    leaves(&mut m, avail, 1);
    leaves(&mut m, robust, 6);
    let themes = m.add(features, Side::After);
    leaves(&mut m, themes, 2);
    leaves(&mut m, features, 3);
    let k = 53.939_453 / 54.0;
    for (id, miro) in [
        (use_cases, -135.5),
        (more, 162.0),
        (avail, 297.5),
        (robust, -135.0),
        (features, 162.0),
    ] {
        let got = m.cy(id) - m.cy(root);
        assert!(
            (got - miro * k).abs() < 1.0,
            "{id:?}: {got} contra o Miro {miro} (× {k})"
        );
    }
    for w in uc.windows(2) {
        assert!((m.cy(w[1]) - m.cy(w[0]) - 53.939_453).abs() < 1e-3);
    }
}

#[test]
fn the_parent_sits_in_the_middle_of_its_childrens_band() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let kids: Vec<ElementId> = (0..3).map(|_| m.add(a, Side::After)).collect();
    let mid = (m.cy(kids[0]) + m.cy(kids[2])) / 2.0;
    assert!((m.cy(a) - mid).abs() < R);
    assert!(
        (m.cy(a) - m.cy(root)).abs() < R,
        "um filho só fica à altura da raiz"
    );
}

/// M2: entre faixas de pais diferentes o vão é o mesmo que entre irmãos.
#[test]
fn cousins_step_like_siblings() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let b = m.add(root, Side::After);
    let a1 = m.add(a, Side::After);
    let a2 = m.add(a, Side::After);
    let b1 = m.add(b, Side::After);
    let step = m.cy(a2) - m.cy(a1);
    assert!((step - (LINE_H + MIND_SIBLING_GAP)).abs() < R);
    assert!(
        (m.cy(b1) - m.cy(a2) - step).abs() < R,
        "primo a primo = irmão a irmão"
    );
}

/// M3: o filho começa um vão depois da borda de FORA do pai — e os filhos de pais com larguras
/// diferentes NÃO se alinham numa coluna.
#[test]
fn a_child_starts_one_level_gap_past_its_parents_outer_edge() {
    let (mut m, root) = Map::new();
    let wide = m.add_sized(root, Side::Before, 240.0, LINE_H);
    let narrow = m.add_sized(root, Side::Before, 60.0, LINE_H);
    let right = m.add(root, Side::After);
    let w1 = m.add(wide, Side::After);
    let n1 = m.add(narrow, Side::After);
    let r = m.el(root).clone();
    assert!((m.el(right).x - (r.x + r.w + MIND_LEVEL_GAP)).abs() < R);
    assert!((m.el(wide).x + m.el(wide).w - (r.x - MIND_LEVEL_GAP)).abs() < R);
    assert!(
        (m.el(narrow).x + m.el(narrow).w - (r.x - MIND_LEVEL_GAP)).abs() < R,
        "a borda de DENTRO dos irmãos alinha"
    );
    assert!((m.el(w1).x + m.el(w1).w - (m.el(wide).x - MIND_LEVEL_GAP)).abs() < R);
    assert!((m.el(n1).x + m.el(n1).w - (m.el(narrow).x - MIND_LEVEL_GAP)).abs() < R);
    assert!((m.el(w1).x - m.el(n1).x).abs() > 100.0);
}

/// M4: um lado cheio não empurra o outro.
#[test]
fn each_side_of_the_root_is_centred_on_it() {
    let (mut m, root) = Map::new();
    for _ in 0..5 {
        m.add(root, Side::Before);
    }
    let lone = m.add(root, Side::After);
    assert!((m.cy(lone) - m.cy(root)).abs() < R);
}

/// M10: recolher esconde os descendentes (fora de `live`) SEM mexer em ninguém; expandir devolve.
#[test]
fn collapsing_hides_without_moving_anyone() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let a1 = m.add(a, Side::After);
    let a11 = m.add(a1, Side::After);
    let b = m.add(root, Side::After);
    let before: Vec<[f64; 2]> = [a, a1, a11, b]
        .iter()
        .map(|id| [m.el(*id).x, m.el(*id).y])
        .collect();
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = true);
    let live: BTreeSet<ElementId> = m.doc.live().map(|e| e.id).collect();
    assert!(live.contains(&a) && live.contains(&b));
    assert!(!live.contains(&a1) && !live.contains(&a11), "o ramo some");
    assert_eq!(m.doc.live_len(), 3);
    let after: Vec<[f64; 2]> = [a, a1, a11, b]
        .iter()
        .map(|id| [m.doc.elements[id].x, m.doc.elements[id].y])
        .collect();
    assert_eq!(before, after, "ninguém se mexe");
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = false);
    assert_eq!(m.doc.live_len(), 5);
}

/// M10: um filho que nasce debaixo de um ramo recolhido nasce escondido; o escondido volta depois de
/// ler os bytes.
#[test]
fn hidden_follows_the_veil_through_new_children_and_the_bytes() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = true);
    let a1 = m.add(a, Side::After);
    assert!(m.doc.elements[&a1].is_hidden());
    let mut set = crate::BoardSet::default();
    let b = set.create("Mapa".into());
    set.get_mut(b).unwrap().doc = m.doc.clone();
    let back = crate::BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(back, set);
    assert!(back.boards()[0].doc.elements[&a1].is_hidden());
}

/// M11: sem auto layout, os filhos ficam onde a mão os pôs — e andam com o pai quando o layout o
/// move.
#[test]
fn a_parent_without_auto_layout_carries_its_children() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let a1 = m.add(a, Side::After);
    m.edit(a, |el| el.mind_mut().unwrap().manual = true);
    m.edit(a1, |el| el.translate([30.0, 200.0]));
    let off = [m.el(a1).x - m.el(a).x, m.el(a1).y - m.el(a).y];
    m.add(root, Side::After);
    let moved = m.el(a).y;
    let now = [m.el(a1).x - m.el(a).x, m.el(a1).y - m.el(a).y];
    assert!((now[0] - off[0]).abs() < R && (now[1] - off[1]).abs() < R);
    assert!(moved < -1.0, "o irmão novo empurrou o pai para cima");
}

/// M16: no vertical, as mesmas leis com os eixos trocados.
#[test]
fn the_vertical_layout_is_the_same_law_turned() {
    let (mut m, root) = Map::new();
    m.edit(root, |el| el.mind_mut().unwrap().vertical = true);
    let a = m.add_sized(root, Side::After, 100.0, LINE_H);
    let b = m.add_sized(root, Side::After, 60.0, LINE_H);
    let r = m.el(root).clone();
    assert!((m.el(a).y - (r.y + r.h + MIND_LEVEL_GAP)).abs() < R);
    let gap = m.el(b).x - (m.el(a).x + m.el(a).w);
    assert!((gap - MIND_SIBLING_GAP).abs() < R);
    let mid = (m.el(a).x + m.el(b).x + m.el(b).w) / 2.0;
    assert!((mid - r.center()[0]).abs() < R);
}

/// M17: o layout entra no MESMO passo de desfazer — desfazer o filho novo devolve os irmãos ao
/// sítio de antes, e refazer volta a abri-los.
#[test]
fn undo_brings_back_the_layout_of_before() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let y0 = m.el(a).y;
    let b = m.add(root, Side::After);
    assert!(m.el(a).y < y0);
    assert!(m.h.undo(&mut m.doc));
    assert!(m.doc.get(b).is_none());
    assert!((m.el(a).y - y0).abs() < R);
    assert!(m.h.redo(&mut m.doc));
    assert!(m.el(a).y < y0);
}

/// M17: a raiz não se move, e uma raiz arrastada leva o mapa (o layout parte dela).
#[test]
fn the_root_anchors_the_map() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let ax = m.el(a).x;
    m.edit(root, |el| el.translate([500.0, 0.0]));
    assert!((m.el(root).x - 417.0).abs() < R);
    assert!((m.el(a).x - (ax + 500.0)).abs() < R);
}

/// Apagar um nó deixa os filhos órfãos como raízes do seu ramo (não ficam escondidos nem perdidos).
#[test]
fn an_orphan_is_the_root_of_its_branch() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = true);
    let a1 = m.add(a, Side::After);
    m.h.apply(&mut m.doc, vec![BoardOp::Delete(a)]);
    assert_eq!(m.doc.mind_root(a1), Some(a1));
    assert!(!m.doc.elements[&a1].is_hidden());
}

/// Um ciclo (uma fusão estragada) não pendura ninguém.
#[test]
fn a_cycle_does_not_hang_the_layout() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let b = m.add(a, Side::After);
    m.edit(a, |el| el.mind_mut().unwrap().parent = Some(b));
    let _ = m.doc.mind_root(a);
    let _ = layout(&m.doc, a);
    let _ = m.doc.mind_depth(b);
}

/// ⭐ Um ficheiro do formato 5 (W5) continua a abrir — gravado pelo build de `ca8f35143` (o último do
/// formato 5): um frame, uma nota dentro dele num grupo, travada, com link e etiqueta, e um
/// rectângulo com texto. Nada é nó de mapa.
#[test]
fn a_format_5_file_still_opens_and_nothing_in_it_is_a_mind_node() {
    let set = crate::BoardSet::from_bytes(V5_BYTES).expect("o formato 5 lê-se");
    let b = &set.boards()[0];
    assert_eq!(b.name, "Org5");
    assert_eq!(b.camera.zoom, 0.5);
    let els = b.doc.live_in_z_order();
    assert_eq!(els.len(), 3);
    assert!(els[0].as_frame().is_some());
    assert!(b.doc.has_frames(), "o índice dos frames refaz-se");
    let note = els[1];
    assert_eq!(note.frame, Some(els[0].id));
    assert!(note.locked() && note.groups().len() == 1 && note.link() == Some(els[0].id));
    assert_eq!(note.tags()[0].name, "urgente");
    assert_eq!(els[2].shape().unwrap().text.as_str(), "caixa");
    assert!(els.iter().all(|e| e.mind().is_none()));
    let back = crate::BoardSet::from_bytes(&set.to_bytes().unwrap()).unwrap();
    assert_eq!(
        back, set,
        "relido e regravado no formato actual, fica igual"
    );
}

/// Gravados pelo build do formato 5 (`ca8f35143`, um teste temporário que escreveu `to_bytes`).
const V5_BYTES: &[u8] = include_bytes!("../fixtures/format_v5.bin");

/// ⭐ M13: o ramo é a curva medida do Miro — da borda de fora do pai à de dentro do filho, a meia
/// altura, tangentes horizontais, braço 0,39 do vão (40,5 em 104, cinco ramos da imagem, rms ≤ 0,97).
#[test]
fn a_branch_is_the_measured_miro_curve() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    let b = m.add(root, Side::Before);
    let r = m.el(root).clone();
    let [p0, p1, p2, p3] = branch(&m.doc, a).expect("um ramo");
    assert!((p0[0] - (r.x + r.w)).abs() < R && (p0[1] - r.center()[1]).abs() < R);
    assert!((p3[0] - m.el(a).x).abs() < R && (p3[1] - m.cy(a)).abs() < R);
    assert!((p1[0] - p0[0] - MIND_ARM * MIND_LEVEL_GAP).abs() < R && p1[1] == p0[1]);
    assert!((p3[0] - p2[0] - MIND_ARM * MIND_LEVEL_GAP).abs() < R && p2[1] == p3[1]);
    assert!(
        ((MIND_ARM * MIND_LEVEL_GAP) - 40.5).abs() < 0.2,
        "o braço do Miro"
    );
    let [q0, _, _, q3] = branch(&m.doc, b).expect("o do outro lado");
    assert!((q0[0] - r.x).abs() < R && (q3[0] - (m.el(b).x + m.el(b).w)).abs() < R);
    assert!(branch(&m.doc, root).is_none());
    m.edit(root, |el| el.mind_mut().unwrap().collapsed = true);
    assert!(
        branch(&m.doc, a).is_none(),
        "um ramo escondido não se desenha"
    );
}

/// O número do selo lê-se sobre a cor de cada ramo (a tinta legível, não o branco: o verde-água
/// dava 1,4:1 — auditoria W6). Contraste WCAG ≥ 3 (texto grande).
#[test]
fn every_branch_colour_has_a_readable_badge_ink() {
    let lum = |c: Rgba| c.luminance();
    for c in MIND_COLORS.iter().copied().chain([MIND_ROOT_STROKE]) {
        let (bg, ink) = (Rgba(c), Rgba(c).readable_ink());
        let (a, b) = (lum(bg), lum(ink));
        let ratio = (a.max(b) + 0.05) / (a.min(b) + 0.05);
        assert!(ratio >= 3.0, "{c:?}: {ratio:.2}");
    }
}

/// M10 com um ramo ALTO (o sobrevivente 5 da prova de mutação): recolher um nó com três filhos — a
/// faixa dele é maior que ele — não move o irmão de baixo (o recolhido continua a reservar a faixa).
#[test]
fn collapsing_a_tall_branch_moves_no_neighbour() {
    let (mut m, root) = Map::new();
    let a = m.add(root, Side::After);
    for _ in 0..3 {
        m.add(a, Side::After);
    }
    let b = m.add(root, Side::After);
    let by = m.el(b).y;
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = true);
    assert!((m.el(b).y - by).abs() < R, "o irmão não se mexe");
}

/// Os descendentes de um frame incluem os nós ESCONDIDOS de um ramo recolhido dentro dele (o
/// sobrevivente 26: apagar, copiar, mover e adoptar perguntam por eles).
#[test]
fn a_frames_descendants_include_the_hidden_nodes() {
    let (mut m, root) = Map::new();
    let paper = Rgba([255, 255, 255, 255]);
    let frame = crate::Frame {
        name: "f".into(),
        style: Style::new(Some(paper), None, ink()),
    };
    let f = Element::new_frame(
        m.doc.mint_id(),
        m.doc.z_on_top(),
        frame,
        [-500.0, -500.0, 2_000.0, 2_000.0],
    );
    let fid = f.id;
    m.h.apply(&mut m.doc, vec![BoardOp::Put(f)]);
    let a = m.add(root, Side::After);
    let a1 = m.add(a, Side::After);
    for id in [root, a, a1] {
        m.edit(id, |el| el.frame = Some(fid));
    }
    m.edit(a, |el| el.mind_mut().unwrap().collapsed = true);
    assert!(m.doc.elements[&a1].is_hidden());
    assert!(m.doc.descendants(fid).contains(&a1));
}
