//! **O formato 11** (W13): o de hoje sem imagens — o quadro sem `files` e a versão sem a lista dos
//! ficheiros que mostra. Também a versão de todos os formatos 8–11 ([`VersionV11`]).

use std::collections::BTreeSet;

use serde::Deserialize;

use super::{DocOld, doc_old};
use crate::{
    Board, BoardId, BoardSet, Camera, ElementId, Files, Poll, PollId, Team, Thread, Version,
    VersionId,
};

/// A versão nomeada dos formatos 8–11: sem a lista dos ficheiros (não havia imagens).
#[derive(Deserialize)]
pub(super) struct VersionV11 {
    id: VersionId,
    name: String,
    at: i64,
    bytes: Vec<u8>,
}

pub(super) fn version_v11(v: VersionV11) -> Version {
    Version {
        id: v.id,
        name: v.name,
        at: v.at,
        bytes: v.bytes,
        files: BTreeSet::new(),
    }
}

#[derive(Deserialize)]
struct TeamV11 {
    threads: Vec<Thread>,
    versions: Vec<VersionV11>,
    hidden_slides: BTreeSet<ElementId>,
    next_id: u64,
    polls: Vec<Poll>,
    shown: Option<PollId>,
}

#[derive(Deserialize)]
struct BoardV11 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: DocOld<super::v13::ElementV13>,
    sketch: bool,
    team: TeamV11,
}

#[derive(Deserialize)]
struct SetV11 {
    _version: u32,
    boards: Vec<BoardV11>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 11 (W13), já no formato actual.
pub(crate) fn read_v11(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV11 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let mut doc = doc_old(b.doc, &super::v13::element_v13);
            doc.reindex();
            let t = b.team;
            let mut team = Team::from_v10(
                t.threads,
                t.versions.into_iter().map(version_v11).collect(),
                t.hidden_slides,
                t.next_id,
            );
            (team.polls, team.shown) = (t.polls, t.shown);
            Board {
                id: b.id,
                name: b.name,
                camera: b.camera,
                doc,
                sketch: b.sketch,
                team,
                files: Files::new(),
            }
        })
        .collect();
    Ok(BoardSet::from_parts(boards, old.next_id))
}
