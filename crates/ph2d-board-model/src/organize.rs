//! **Organizar** (W5): frames, grupos, etiquetas — os tipos e as perguntas puras sobre a árvore de
//! frames. As leis vêm do help do Miro e do oráculo Excalidraw
//! (`docs/MiroClone/ferramentas/miro_organizar_notas.md`, README do oráculo §W5).
//!
//! A pertença a um frame é um ponteiro do FILHO (`Element::frame`, o frame mais interior), como o
//! `frameId` do Excalidraw: mover ou apagar um frame pergunta [`BoardDoc::descendants`]. Frames
//! aninham-se (um frame tem `frame` = o de fora); [`BoardDoc::can_enter`] recusa os ciclos.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{BoardDoc, Element, ElementId, ElementKind, Rgba, Style};

/// Um grupo: os elementos que o têm no seu `groups` movem-se e seleccionam-se juntos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GroupId(pub u64);

/// Uma etiqueta de nota (o Miro: até [`MAX_TAGS`] por nota, nome e cor).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub color: Rgba,
}

/// O que organiza um elemento além da casa: grupos, travar, link, etiquetas — raros, por isso numa
/// caixa à parte ([`Element::org`] é `None` em quase todos).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Org {
    /// Os grupos, do de DENTRO ao de FORA (o `groupIds` do Excalidraw).
    pub groups: Vec<GroupId>,
    /// Travado: não se move, não se edita, não se apaga; fora do laço e do `Ctrl+A`.
    pub locked: bool,
    /// O elemento para onde este leva (o «Link to» do Miro: um só por elemento).
    pub link: Option<ElementId>,
    /// As etiquetas (só as notas as mostram; até [`MAX_TAGS`]).
    pub tags: Vec<Tag>,
}

/// A cor de nascença de um frame (o Miro: *«The default color of a frame is white»*).
pub const FRAME_FILL: [u8; 4] = [255, 255, 255, 255];

/// Quantas etiquetas uma nota leva (help do Miro, «Sticky notes»: *«up to 8 tags to each sticky»*).
pub const MAX_TAGS: usize = 8;

/// Um frame: uma área com nome e cor que leva os filhos. Fica sempre ATRÁS dos objectos (help do
/// Miro, «Working with objects»); a chave `z` dele ordena só os frames entre si — a ordem do
/// painel de Frames.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// Nome dado pelo artista (dado, não i18n).
    pub name: String,
    /// O `fill` é a cor do frame (`None` = transparente; a de nascença é branca, o Miro: *«The
    /// default color of a frame is white»*) — o MESMO estilo das formas, para a barra de cores o
    /// servir sem caso à parte.
    pub style: Style,
}

impl Element {
    /// O frame, se este elemento é um.
    #[must_use]
    pub fn as_frame(&self) -> Option<&Frame> {
        match &self.kind {
            ElementKind::Frame(f) => Some(f),
            _ => None,
        }
    }

    /// O frame, mutável.
    pub fn as_frame_mut(&mut self) -> Option<&mut Frame> {
        match &mut self.kind {
            ElementKind::Frame(f) => Some(f),
            _ => None,
        }
    }

    /// O grupo de FORA (o que um clique selecciona), se há.
    #[must_use]
    pub fn outer_group(&self) -> Option<GroupId> {
        self.groups().last().copied()
    }

    /// Os grupos, do de dentro ao de fora.
    #[must_use]
    pub fn groups(&self) -> &[GroupId] {
        self.org.as_ref().map_or(&[], |o| &o.groups)
    }

    #[must_use]
    pub fn locked(&self) -> bool {
        self.org.as_ref().is_some_and(|o| o.locked)
    }

    #[must_use]
    pub fn link(&self) -> Option<ElementId> {
        self.org.as_ref().and_then(|o| o.link)
    }

    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        self.org.as_ref().map_or(&[], |o| &o.tags)
    }

    /// A caixa de organizar, criada no primeiro uso.
    pub fn org_mut(&mut self) -> &mut Org {
        self.org.get_or_insert_with(Box::default)
    }

    pub fn groups_mut(&mut self) -> &mut Vec<GroupId> {
        &mut self.org_mut().groups
    }

    pub fn set_locked(&mut self, on: bool) {
        if on || self.org.is_some() {
            self.org_mut().locked = on;
        }
    }

    pub fn set_link(&mut self, to: Option<ElementId>) {
        if to.is_some() || self.org.is_some() {
            self.org_mut().link = to;
        }
    }

    pub fn tags_mut(&mut self) -> &mut Vec<Tag> {
        &mut self.org_mut().tags
    }

    /// A caixa (sem rotação: um frame nunca roda) contém `p`.
    #[must_use]
    pub fn box_contains(&self, [px, py]: [f64; 2]) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    /// A caixa deste elemento contém INTEIRA a caixa envolvente (`aabb`) de `other`.
    #[must_use]
    pub fn box_contains_all_of(&self, other: &Element) -> bool {
        let [x0, y0, x1, y1] = other.aabb();
        x0 >= self.x && y0 >= self.y && x1 <= self.x + self.w && y1 <= self.y + self.h
    }
}

impl BoardDoc {
    /// Refaz o índice dos frames (depois de ler de bytes).
    pub(crate) fn reindex(&mut self) {
        self.frames = self
            .elements
            .values()
            .filter(|e| !e.deleted && e.as_frame().is_some())
            .map(|e| e.id)
            .collect();
    }

    /// Há algum frame vivo? (O desenho salta o andaime dos frames num quadro sem nenhum.)
    #[must_use]
    pub fn has_frames(&self) -> bool {
        !self.frames.is_empty()
    }

    /// Os frames vivos, por id (o índice: sem passar pelo quadro inteiro).
    pub fn frames(&self) -> impl Iterator<Item = &Element> {
        self.frames.iter().filter_map(|id| self.get(*id))
    }

    /// Quantos frames há por fora de `id` (0 = no quadro).
    #[must_use]
    pub fn frame_depth(&self, id: ElementId) -> usize {
        self.frame_chain(id).len()
    }

    /// Os frames que contêm `id`, do mais interior ao de fora. Um ponteiro para um frame apagado ou
    /// um ciclo (que nenhuma op escreve) param a cadeia.
    #[must_use]
    pub fn frame_chain(&self, id: ElementId) -> Vec<ElementId> {
        let mut out = Vec::new();
        let mut at = self.get(id).and_then(|e| e.frame);
        while let Some(f) = at {
            if out.contains(&f) || f == id {
                break;
            }
            let Some(fe) = self.get(f).filter(|e| e.as_frame().is_some()) else {
                break;
            };
            out.push(f);
            at = fe.frame;
        }
        out
    }

    /// Tudo o que vive dentro de `frame`, a qualquer profundidade (filhos, netos…), por id.
    #[must_use]
    pub fn descendants(&self, frame: ElementId) -> BTreeSet<ElementId> {
        self.live()
            .filter(|e| e.frame.is_some() && self.frame_chain(e.id).contains(&frame))
            .map(|e| e.id)
            .collect()
    }

    /// `child` pode passar a viver dentro de `frame` (nenhum ciclo: `frame` não é `child` nem está
    /// dentro dele).
    #[must_use]
    pub fn can_enter(&self, child: ElementId, frame: ElementId) -> bool {
        child != frame
            && self.get(frame).is_some_and(|f| f.as_frame().is_some())
            && !self.frame_chain(frame).contains(&child)
    }

    /// O frame mais INTERIOR debaixo de `p` (o de maior profundidade; num empate, o de cima), fora
    /// de `skip` e do que vive dentro de `skip`.
    #[must_use]
    pub fn deepest_frame_at(&self, p: [f64; 2], skip: &BTreeSet<ElementId>) -> Option<ElementId> {
        self.frames()
            .filter(|e| e.box_contains(p) && !skip.contains(&e.id))
            .map(|e| (self.frame_chain(e.id), e))
            .filter(|(chain, _)| !chain.iter().any(|f| skip.contains(f)))
            .max_by(|(ca, a), (cb, b)| ca.len().cmp(&cb.len()).then(a.z.cmp(&b.z)))
            .map(|(_, e)| e.id)
    }

    /// Os frames vivos na ordem em que se desenham: por profundidade (o de fora primeiro, por
    /// baixo) e, na mesma profundidade, pelo `z` (a ordem do painel).
    #[must_use]
    pub fn frames_in_draw_order(&self) -> Vec<&Element> {
        let mut v: Vec<(usize, &Element)> =
            self.frames().map(|e| (self.frame_depth(e.id), e)).collect();
        v.sort_by(|(da, a), (db, b)| da.cmp(db).then(a.z.cmp(&b.z)).then(a.id.cmp(&b.id)));
        v.into_iter().map(|(_, e)| e).collect()
    }

    /// Os frames vivos na ordem do painel de Frames (o `z`).
    #[must_use]
    pub fn frames_in_panel_order(&self) -> Vec<&Element> {
        let mut v: Vec<&Element> = self.frames().collect();
        v.sort_by(|a, b| a.z.cmp(&b.z).then(a.id.cmp(&b.id)));
        v
    }

    /// A caixa de TUDO o que está no quadro (as formas, as notas, os traços e os frames; as setas
    /// vivem entre eles) — o que o minimapa mostra e o «zoom to fit» enquadra. `None` = vazio.
    #[must_use]
    pub fn extent(&self) -> Option<[f64; 4]> {
        let mut it = self
            .live()
            .filter(|e| e.connector().is_none())
            .map(Element::aabb);
        let first = it.next()?;
        Some(it.fold(first, |a, b| {
            [
                a[0].min(b[0]),
                a[1].min(b[1]),
                a[2].max(b[2]),
                a[3].max(b[3]),
            ]
        }))
    }

    /// Os membros vivos do grupo `g`.
    #[must_use]
    pub fn group_members(&self, g: GroupId) -> Vec<ElementId> {
        self.live()
            .filter(|e| e.groups().contains(&g))
            .map(|e| e.id)
            .collect()
    }

    /// Um id de grupo ainda não usado neste quadro (partilha o contador dos elementos).
    pub fn mint_group(&mut self) -> GroupId {
        GroupId(self.mint_id().0)
    }
}

#[cfg(test)]
#[path = "organize_tests.rs"]
mod tests;
