//! **Os formatos 9–10 do ELEMENTO e o 10 inteiro** (W13, ADR-0183): o `Org` ainda com os votos da
//! última votação (`Org::votes`, W7) e a equipa sem histórico. Os votos que o ficheiro trazia viram UMA
//! sessão, mostrada ([`crate::Team::adopt_old_votes`]).

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::{DocOld, VersionV11, doc_old, version_v11};
use crate::{Board, BoardId, BoardSet, Camera, Element, ElementId, FracKey, Team, Thread};

/// O `Org` dos formatos 9–10: com os votos.
#[derive(Deserialize)]
struct OrgV10 {
    groups: Vec<crate::GroupId>,
    locked: bool,
    link: Option<ElementId>,
    tags: Vec<crate::Tag>,
    mind: Option<crate::Mind>,
    votes: u32,
    stamps: Vec<crate::StampCount>,
    seed: Option<u32>,
}

/// O elemento dos formatos 9–10: o do 13 com o `Org` antigo.
#[derive(Deserialize)]
pub(super) struct ElementV10 {
    id: ElementId,
    kind: super::v13::KindV13,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    angle: f64,
    z: FracKey,
    frame: Option<ElementId>,
    org: Option<Box<OrgV10>>,
    version: u64,
    nonce: u32,
    deleted: bool,
}

pub(super) fn element_v10(e: ElementV10) -> Element {
    let mut el = Element::bare(e.id, e.z, e.kind.into(), [e.x, e.y, e.w, e.h]);
    el.angle = e.angle;
    el.frame = e.frame;
    el.org = e.org.map(|o| {
        Box::new(crate::Org {
            groups: o.groups,
            locked: o.locked,
            link: o.link,
            tags: o.tags,
            mind: o.mind,
            stamps: o.stamps,
            seed: o.seed,
        })
    });
    (el.version, el.nonce, el.deleted) = (e.version, e.nonce, e.deleted);
    el
}

/// Os votos dos elementos vivos de um documento dos formatos 9–10.
pub(super) fn votes_v10(d: &DocOld<ElementV10>) -> BTreeMap<ElementId, u32> {
    super::old_votes(d, |e| (e.deleted, e.org.as_ref().map_or(0, |o| o.votes)))
}

/// A `Team` do formato 10 (W11): com ids, sem votações.
#[derive(Deserialize)]
struct TeamV10 {
    threads: Vec<Thread>,
    versions: Vec<VersionV11>,
    hidden_slides: BTreeSet<ElementId>,
    next_id: u64,
}

#[derive(Deserialize)]
struct BoardV10 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<ElementV10>,
    sketch: bool,
    team: TeamV10,
}

#[derive(Deserialize)]
struct SetV10 {
    _version: u32,
    boards: Vec<BoardV10>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 10 (W11–W12), já no formato actual: os votos viram o histórico.
pub(crate) fn read_v10(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV10 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let votes = votes_v10(&b.doc);
            let mut doc = doc_old(b.doc, &element_v10);
            doc.reindex();
            let t = b.team;
            let versions = t.versions.into_iter().map(version_v11).collect();
            let mut team = Team::from_v10(t.threads, versions, t.hidden_slides, t.next_id);
            team.adopt_old_votes(votes);
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team,
                files: crate::Files::new(),
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{FracKey, Rgba, RichText, Shape, ShapeType, Style};

    /// ⭐ W13 (prova de mutação, M6; ADR-0183 §9): só os votos dos elementos VIVOS de um ficheiro antigo
    /// viram a sessão — um voto numa nota apagada não tinha selo e não entra.
    #[test]
    fn only_the_living_bring_their_old_votes() {
        let sticky = Shape {
            kind: ShapeType::Sticky,
            style: Style::new(None, None, Rgba([0, 0, 0, 255])),
            text: RichText::plain("x"),
        };
        let el = |id: u64, votes: u32, deleted: bool| ElementV10 {
            id: ElementId(id),
            kind: super::super::v13::KindV13::Shape(sticky.clone()),
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            angle: 0.0,
            z: FracKey::between(None, None),
            frame: None,
            org: Some(Box::new(OrgV10 {
                groups: Vec::new(),
                locked: false,
                link: None,
                tags: Vec::new(),
                mind: None,
                votes,
                stamps: Vec::new(),
                seed: None,
            })),
            version: 0,
            nonce: 0,
            deleted,
        };
        let doc = DocOld {
            elements: BTreeMap::from([
                (ElementId(1), el(1, 3, false)),
                (ElementId(2), el(2, 5, true)),
                (ElementId(3), el(3, 0, false)),
            ]),
            next_id: 3,
            top_z: None,
        };
        assert_eq!(votes_v10(&doc), BTreeMap::from([(ElementId(1), 3)]));
    }
}
