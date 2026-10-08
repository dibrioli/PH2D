//! **Organizar** (W5) no editor: agrupar, travar, ordem, alinhar e distribuir, e a pertença a um
//! frame (quem entra, quem sai, quem vai junto). As leis e as fontes:
//! `docs/MiroClone/ferramentas/miro_organizar_notas.md` (o produto) e o README do oráculo Excalidraw
//! §W5 (o número, fixtures `saidas/organizar_*`).
//!
//! - **Unidade**: o que se move junto — um elemento solto, um grupo (o de FORA) ou um frame com tudo o
//!   que vive dentro dele. Alinhar e distribuir trabalham por unidade (oráculo C2: o grupo alinha-se
//!   inteiro).
//! - **Travado**: não se move, não se apaga, não se edita; um clique selecciona-o (para o destravar,
//!   o Miro), o laço e o `Ctrl+A` não.
//! - **Frame**: sempre ATRÁS dos objectos e fora da ordem (o Miro); a pertença decide-se pelo
//!   PONTEIRO ao largar (oráculo A4/A5) e, ao desenhar um frame, ele adopta o que fica INTEIRO dentro
//!   (A1).

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, FracKey, GroupId, History};

use crate::{Editor, Pointer, live, union_aabb};
use ph2d_board_model::ShapeType;

/// Para onde vai a selecção na ordem de desenho (as teclas do Miro: `PgUp`/`PgDn` e com `Shift`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Order {
    /// À frente de tudo.
    Front,
    /// Atrás de tudo (mas à frente dos frames, que estão sempre atrás).
    Back,
    /// À frente do objecto que está logo à frente dela (e que lhe toca).
    Forward,
    /// Atrás do objecto que está logo atrás dela (e que lhe toca).
    Backward,
}

/// Alinhar pela caixa da selecção (oráculo C1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    CenterH,
    Right,
    Top,
    CenterV,
    Bottom,
}

/// O eixo de distribuir.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Uma recusa que o editor devolve à vista (que a mostra como aviso — nunca só no terminal).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// Agrupar elementos de frames diferentes (o Miro: *«You can't group … objects located in
    /// different frames»*).
    GroupAcrossFrames,
    /// Mexer (mover, apagar, editar) em algo travado.
    Locked,
    /// Uma nota já tem as [`ph2d_board_model::MAX_TAGS`] etiquetas.
    TooManyTags,
    /// O modo de ligar está aberto: o próximo clique escolhe o destino.
    LinkPick,
    /// O destino do link foi apagado (o Miro: *«the linked item has been deleted from this
    /// board»*).
    LinkGone,
}

impl Notice {
    /// Todos — o gate de que cada um tem frase.
    pub const ALL: [Notice; 5] = [
        Notice::GroupAcrossFrames,
        Notice::Locked,
        Notice::TooManyTags,
        Notice::LinkPick,
        Notice::LinkGone,
    ];

    /// A chave i18n da frase que o artista lê.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Notice::GroupAcrossFrames => "board.notice.group_across_frames",
            Notice::Locked => "board.notice.locked",
            Notice::TooManyTags => "board.notice.too_many_tags",
            Notice::LinkPick => "board.notice.link_pick",
            Notice::LinkGone => "board.notice.link_gone",
        }
    }
}

impl Editor {
    /// O que um clique em `id` selecciona: o grupo de FORA dele, ou — dentro de um grupo em que se
    /// entrou com duplo-clique — o grupo logo abaixo desse, ou o elemento só (oráculo B).
    pub(crate) fn pick(&self, doc: &BoardDoc, id: ElementId) -> BTreeSet<ElementId> {
        let Some(el) = doc.get(id) else {
            return BTreeSet::new();
        };
        match self.pick_group(el) {
            Some(g) => doc.group_members(g).into_iter().collect(),
            None => BTreeSet::from([id]),
        }
    }

    /// O grupo que um clique em `el` apanha (`None` = o elemento só).
    fn pick_group(&self, el: &Element) -> Option<GroupId> {
        let inside = self
            .entered
            .and_then(|g| el.groups().iter().position(|x| *x == g));
        match inside {
            Some(0) => None,
            Some(i) => Some(el.groups()[i - 1]),
            None => el.outer_group(),
        }
    }

    /// Duplo-clique num membro de um grupo seleccionado: entra um nível. `true` = entrou.
    pub(crate) fn enter_group(&mut self, doc: &BoardDoc, id: ElementId) -> bool {
        let Some(el) = doc.get(id) else {
            return false;
        };
        let Some(g) = self.pick_group(el) else {
            return false;
        };
        if !self.selection.contains(&id) {
            return false;
        }
        self.entered = Some(g);
        self.selection = self.pick(doc, id);
        true
    }

    /// Sai de qualquer grupo em que se tinha entrado (um clique no vazio — oráculo B).
    pub(crate) fn leave_groups(&mut self) {
        self.entered = None;
    }

    /// Os seleccionados que se podem mexer (os destravados).
    pub(crate) fn movable(&self, doc: &BoardDoc) -> Vec<Element> {
        self.selected(doc)
            .into_iter()
            .filter(|el| !el.locked())
            .cloned()
            .collect()
    }

    /// `Ctrl+G`: a selecção passa a ser um grupo (o novo fica por FORA dos que já tinham). Recusa se
    /// os elementos vivem em frames diferentes. UM passo.
    pub(crate) fn group(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let sel = self.selected(doc);
        if sel.len() < 2 {
            return false;
        }
        let homes: BTreeSet<Option<ElementId>> = sel.iter().map(|el| el.frame).collect();
        if homes.len() > 1 {
            self.notice = Some(Notice::GroupAcrossFrames);
            return false;
        }
        let ids: Vec<ElementId> = sel.iter().map(|el| el.id).collect();
        let g = doc.mint_group();
        let ops = ids
            .iter()
            .filter_map(|id| doc.get(*id).cloned())
            .map(|mut el| {
                el.groups_mut().push(g);
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        self.entered = None;
        true
    }

    /// `Ctrl+Shift+G`: tira o grupo de FORA de cada seleccionado (oráculo B5). UM passo.
    pub(crate) fn ungroup(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let ops: Vec<BoardOp> = self
            .selected(doc)
            .into_iter()
            .filter(|el| !el.groups().is_empty())
            .map(|el| {
                let mut el = el.clone();
                el.groups_mut().pop();
                BoardOp::Put(el)
            })
            .collect();
        let any = !ops.is_empty();
        history.apply(doc, ops);
        self.entered = None;
        any
    }

    /// `Ctrl+Shift+L`: trava a selecção — ou destrava-a, se já estava toda travada. UM passo.
    pub(crate) fn toggle_lock(&mut self, doc: &mut BoardDoc, history: &mut History) -> bool {
        let sel = self.selected(doc);
        if sel.is_empty() {
            return false;
        }
        let on = !sel.iter().all(|el| el.locked());
        let ops = sel
            .into_iter()
            .filter(|el| el.locked() != on)
            .map(|el| {
                let mut el = el.clone();
                el.set_locked(on);
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        true
    }

    /// Muda a selecção de lugar na ordem de desenho. Os frames não entram (estão sempre atrás); a
    /// selecção mantém a ordem relativa dela. UM passo.
    pub(crate) fn reorder(&mut self, doc: &mut BoardDoc, history: &mut History, o: Order) -> bool {
        let sel: Vec<Element> = self
            .editable(doc)
            .into_iter()
            .filter(|el| el.as_frame().is_none())
            .cloned()
            .collect();
        if sel.is_empty() {
            return false;
        }
        let ids: BTreeSet<ElementId> = sel.iter().map(|el| el.id).collect();
        let others: Vec<&Element> = doc
            .live_in_z_order()
            .into_iter()
            .filter(|el| el.as_frame().is_none() && !ids.contains(&el.id))
            .collect();
        let (lo, hi) = match o {
            Order::Front => (others.last().map(|e| e.z.clone()), None),
            Order::Back => (None, others.first().map(|e| e.z.clone())),
            Order::Forward | Order::Backward => {
                let Some((lo, hi)) = step_slot(&sel, &others, o == Order::Forward) else {
                    return false;
                };
                (lo, hi)
            }
        };
        let keys = keys_between(lo.as_ref(), hi.as_ref(), sel.len());
        let ops = sel
            .into_iter()
            .zip(keys)
            .map(|(mut el, z)| {
                el.z = z;
                BoardOp::Put(el)
            })
            .collect();
        history.apply(doc, ops);
        true
    }

    /// Alinha as unidades da selecção pela caixa dela (oráculo C1/C2/C4: grupos e frames inteiros, a
    /// caixa de um rodado é a envolvente). UM passo.
    pub(crate) fn align(&mut self, doc: &mut BoardDoc, history: &mut History, a: Align) -> bool {
        let units = self.units(doc);
        if units.len() < 2 {
            return false;
        }
        let all = union_boxes(units.iter().map(|u| u.bbox));
        let moves = units
            .iter()
            .map(|u| {
                let b = u.bbox;
                let d = match a {
                    Align::Left => [all[0] - b[0], 0.0],
                    Align::Right => [all[2] - b[2], 0.0],
                    Align::CenterH => [(all[0] + all[2] - b[0] - b[2]) / 2.0, 0.0],
                    Align::Top => [0.0, all[1] - b[1]],
                    Align::Bottom => [0.0, all[3] - b[3]],
                    Align::CenterV => [0.0, (all[1] + all[3] - b[1] - b[3]) / 2.0],
                };
                (u, d)
            })
            .collect::<Vec<_>>();
        self.apply_unit_moves(doc, history, &moves)
    }

    /// Distribui as unidades com vãos IGUAIS entre caixas, as duas das pontas paradas (oráculo C3:
    /// larguras 41/113/29/87 em 337..787 ⇒ vão 60). Menos de três: nada. UM passo.
    pub(crate) fn distribute(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        ax: Axis,
    ) -> bool {
        let mut units = self.units(doc);
        if units.len() < 3 {
            return false;
        }
        let (a0, a1) = match ax {
            Axis::Horizontal => (0, 2),
            Axis::Vertical => (1, 3),
        };
        units.sort_by(|u, v| u.bbox[a0].total_cmp(&v.bbox[a0]));
        let first = units[0].bbox[a0];
        let last = units
            .iter()
            .map(|u| u.bbox[a1])
            .fold(f64::NEG_INFINITY, f64::max);
        let used: f64 = units.iter().map(|u| u.bbox[a1] - u.bbox[a0]).sum();
        let gap = (last - first - used) / (units.len() - 1) as f64;
        let mut at = first;
        let mut moves = Vec::new();
        for u in &units {
            let d = at - u.bbox[a0];
            moves.push((u, if a0 == 0 { [d, 0.0] } else { [0.0, d] }));
            at += u.bbox[a1] - u.bbox[a0] + gap;
        }
        self.apply_unit_moves(doc, history, &moves)
    }

    fn apply_unit_moves(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        moves: &[(&Unit, [f64; 2])],
    ) -> bool {
        let ops: Vec<BoardOp> = moves
            .iter()
            .filter(|(_, d)| d[0] != 0.0 || d[1] != 0.0)
            .flat_map(|(u, d)| {
                u.members.iter().filter_map(|id| doc.get(*id)).map(|el| {
                    let mut el = el.clone();
                    el.translate(*d);
                    BoardOp::Put(el)
                })
            })
            .collect();
        let any = !ops.is_empty();
        history.apply(doc, ops);
        any
    }

    /// As unidades destravadas da selecção (sem setas: seguem as formas a que se prendem).
    fn units(&self, doc: &BoardDoc) -> Vec<Unit> {
        let mut by_key: BTreeMap<UnitKey, Vec<ElementId>> = BTreeMap::new();
        for el in self.selected(doc) {
            if el.locked() || el.connector().is_some() {
                continue;
            }
            let key = match el.outer_group() {
                Some(g) if self.entered != Some(g) => UnitKey::Group(g),
                _ => UnitKey::One(el.id),
            };
            by_key.entry(key).or_default().push(el.id);
        }
        by_key
            .into_values()
            .map(|ids| {
                let members = with_descendants(doc, &ids);
                let bbox = union_aabb(members.iter().filter_map(|id| doc.get(*id)));
                Unit {
                    members: members.into_iter().collect(),
                    bbox,
                }
            })
            .collect()
    }
}

impl Editor {
    /// ⭐ Um frame agarra-se por QUALQUER sítio dele que esteja vazio — o miolo, a borda ou o título
    /// por cima (ordem do dono, 08/10: *«transformar como as outras shapes clicando e arrastando
    /// dentro deles, desde que num espaço vazio»*; antes era só borda e título, o Miro). Quem chama
    /// pergunta-o DEPOIS dos objectos, por isso o que está dentro ganha. O mais interior primeiro.
    pub(crate) fn frame_hit(&self, doc: &BoardDoc, p: Pointer) -> Option<ElementId> {
        let tol = self.metrics.handle / 2.0 * p.px;
        let band = self.metrics.frame_title * p.px;
        doc.frames_in_draw_order()
            .into_iter()
            .rev()
            .find(|f| {
                // No espaço do frame (rodado com ele).
                let [x, y] = if f.angle == 0.0 {
                    p.world
                } else {
                    f.unrotate(p.world)
                };
                let (x0, y0, x1, y1) = (f.x, f.y, f.x + f.w, f.y + f.h);
                let inside = x >= x0 - tol && x <= x1 + tol && y >= y0 - tol && y <= y1 + tol;
                let title = x >= x0 && x <= x1 && y >= y0 - band && y <= y0;
                inside || title
            })
            .map(|f| f.id)
    }

    /// O que um arrasto da selecção leva: os destravados e TUDO o que vive dentro dos frames deles —
    /// travado incluído (o frame leva o que é dele; travar impede só de o mexer SOZINHO — smoke do
    /// dono, 08/10: *«a nota com o cadeado não segue o frame»*).
    pub(crate) fn moving(&self, doc: &BoardDoc) -> Vec<Element> {
        let free: Vec<ElementId> = self.movable(doc).iter().map(|el| el.id).collect();
        crate::organize::with_descendants(doc, &free)
            .into_iter()
            .filter_map(|id| doc.get(id))
            .cloned()
            .collect()
    }

    /// Um elemento acabado de nascer onde a ferramenta começou: forma, nota ou frame — a viver no
    /// frame mais interior debaixo do INÍCIO do arrasto (oráculo A1b–A1e).
    pub(crate) fn born(
        &self,
        doc: &mut BoardDoc,
        kind: ShapeType,
        frame: bool,
        start: [f64; 2],
        bx: [f64; 4],
    ) -> Element {
        let home = doc.deepest_frame_at(start, &BTreeSet::new());
        let mut el = if frame {
            self.new_frame(doc, bx)
        } else {
            self.new_shape(doc, kind, bx)
        };
        el.frame = home;
        el
    }

    /// Um frame novo na caixa dada, branco, com o nome de nascença do próximo número.
    pub(crate) fn new_frame(&self, doc: &mut BoardDoc, bx: [f64; 4]) -> Element {
        let n = doc.live().filter(|e| e.as_frame().is_some()).count() + 1;
        let fill = ph2d_board_model::Rgba(ph2d_board_model::FRAME_FILL);
        let frame = ph2d_board_model::Frame {
            name: (self.frame_name)(n),
            style: ph2d_board_model::Style::new(Some(fill), None, fill.readable_ink()),
        };
        Element::new_frame(doc.mint_id(), doc.z_on_top(), frame, bx)
    }
}

/// [`settle_frame`] aplicado ao vivo, com o estado de antes de cada mudança no `undo` do gesto.
pub(crate) fn settle_live(doc: &mut BoardDoc, frame: ElementId, undo: &mut Vec<BoardOp>) {
    for el in settle_frame(doc, frame) {
        if let Some(before) = doc.get(el.id).cloned() {
            undo.push(BoardOp::Put(before));
        }
        live(doc, el);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UnitKey {
    Group(GroupId),
    One(ElementId),
}

/// O que se move junto, e a caixa envolvente dele.
struct Unit {
    members: Vec<ElementId>,
    bbox: [f64; 4],
}

fn union_boxes(bs: impl Iterator<Item = [f64; 4]>) -> [f64; 4] {
    bs.fold(
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
    )
}

fn overlaps(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] <= b[2] && b[0] <= a[2] && a[1] <= b[3] && b[1] <= a[3]
}

/// `ids` e tudo o que vive dentro dos frames entre eles, sem repetir (um filho seleccionado junto
/// com o frame dele move-se UMA vez).
pub(crate) fn with_descendants(doc: &BoardDoc, ids: &[ElementId]) -> BTreeSet<ElementId> {
    let mut out: BTreeSet<ElementId> = ids.iter().copied().collect();
    for id in ids {
        if doc.get(*id).is_some_and(|e| e.as_frame().is_some()) {
            out.extend(doc.descendants(*id));
        }
        // Um nó de mapa mental leva o ramo dele (W6, M9/M12): apagar, mover, copiar.
        if doc.get(*id).is_some_and(|e| e.mind().is_some()) {
            out.extend(doc.mind_descendants(*id));
        }
    }
    out
}

/// `n` chaves de z crescentes estritamente entre `lo` e `hi`.
fn keys_between(lo: Option<&FracKey>, hi: Option<&FracKey>, n: usize) -> Vec<FracKey> {
    let mut out: Vec<FracKey> = Vec::with_capacity(n);
    for _ in 0..n {
        let k = FracKey::between(out.last().or(lo), hi);
        out.push(k);
    }
    out
}

/// O lugar de um passo à frente (ou atrás): logo depois do primeiro objecto ACIMA da selecção que
/// lhe toca — inteiro, se é de um grupo — e antes do seguinte. `None` = nenhum lhe toca desse lado
/// (o passo não teria efeito à vista).
fn step_slot(
    sel: &[Element],
    others: &[&Element],
    forward: bool,
) -> Option<(Option<FracKey>, Option<FracKey>)> {
    let bbox = union_aabb(sel.iter());
    let top = sel.iter().map(|e| &e.z).max()?;
    let bottom = sel.iter().map(|e| &e.z).min()?;
    let touching = |e: &&&Element| overlaps(e.aabb(), bbox);
    let pass = if forward {
        others.iter().find(|e| e.z > *top && touching(e))?
    } else {
        others.iter().rev().find(|e| e.z < *bottom && touching(e))?
    };
    // Um vizinho de um grupo salta-se inteiro (oráculo D5a).
    let block: Vec<&&Element> = match pass.outer_group() {
        Some(g) => others.iter().filter(|e| e.groups().contains(&g)).collect(),
        None => vec![pass],
    };
    if forward {
        let edge = block.iter().map(|e| &e.z).max()?;
        let next = others.iter().find(|e| e.z > *edge).map(|e| e.z.clone());
        Some((Some(edge.clone()), next))
    } else {
        let edge = block.iter().map(|e| &e.z).min()?;
        let prev = others
            .iter()
            .rev()
            .find(|e| e.z < *edge)
            .map(|e| e.z.clone());
        Some((prev, Some(edge.clone())))
    }
}

/// ⭐ **A pertença ao largar** (oráculo A4/A5): cada raiz movida passa a viver no frame mais
/// interior debaixo do PONTEIRO — ou solta, se ele está fora de todos. Um frame não entra em si
/// nem no que vive dentro dele. Devolve os elementos que mudaram de casa (para pôr ao vivo).
pub(crate) fn rehome_on_drop(
    doc: &BoardDoc,
    roots: &[ElementId],
    pointer: [f64; 2],
) -> Vec<Element> {
    let skip: BTreeSet<ElementId> = roots.iter().copied().collect();
    let home = doc.deepest_frame_at(pointer, &skip);
    roots
        .iter()
        .filter_map(|id| doc.get(*id))
        .filter(|el| el.connector().is_none())
        .filter(|el| el.frame != home)
        .filter(|el| home.is_none_or(|f| doc.can_enter(el.id, f)))
        .map(|el| {
            let mut el = el.clone();
            el.frame = home;
            el
        })
        .collect()
}

/// As raízes de um conjunto movido: os que não vivem dentro de um frame que também se move.
pub(crate) fn roots_of(doc: &BoardDoc, ids: &BTreeSet<ElementId>) -> Vec<ElementId> {
    ids.iter()
        .copied()
        .filter(|id| !doc.frame_chain(*id).iter().any(|f| ids.contains(f)))
        .collect()
}

/// ⭐ **O frame acabado de desenhar (ou redimensionado) acerta a casa dos vizinhos** (oráculo A1/A8):
/// adopta o que vive na MESMA casa que ele e lhe fica INTEIRO dentro, e solta para a casa dele o
/// filho que ficou INTEIRO fora. Devolve os elementos que mudaram.
pub(crate) fn settle_frame(doc: &BoardDoc, frame: ElementId) -> Vec<Element> {
    let Some(f) = doc.get(frame).cloned() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for el in doc.live() {
        if el.id == frame || el.connector().is_some() {
            continue;
        }
        let fully_in = f.box_contains_all_of(el);
        if el.frame == f.frame && fully_in && doc.can_enter(el.id, frame) {
            let mut el = el.clone();
            el.frame = Some(frame);
            out.push(el);
        } else if el.frame == Some(frame) && !overlaps(el.aabb(), f.aabb()) {
            let mut el = el.clone();
            el.frame = f.frame;
            out.push(el);
        }
    }
    out
}
