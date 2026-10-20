//! **O formato 9 e a equipa sem ids** (W8–W9): mensagens e versões identificavam-se pela posição. O
//! leitor dá-lhes ids pelo contador da equipa — cada fio, depois as mensagens dele pela ordem, e no
//! fim as versões —, e o contador fica acima de todos. Os fios renumeram-se com elas (o `ThreadId`
//! antigo só vivia na sessão).

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{
    Board, BoardId, BoardSet, Camera, ElementId, Message, MessageId, Pin, Team, Thread, ThreadId,
    Version, VersionId,
};

#[derive(Deserialize)]
struct MessageV9 {
    author: String,
    text: String,
    at: i64,
    edited: bool,
}

#[derive(Deserialize)]
struct ThreadV9 {
    _id: ThreadId,
    pin: Pin,
    messages: Vec<MessageV9>,
    resolved: bool,
    read_by: Vec<String>,
}

#[derive(Deserialize)]
struct VersionV9 {
    name: String,
    at: i64,
    bytes: Vec<u8>,
}

/// A `Team` dos formatos 8–9.
#[derive(Deserialize)]
pub(super) struct TeamV9 {
    threads: Vec<ThreadV9>,
    versions: Vec<VersionV9>,
    hidden_slides: BTreeSet<ElementId>,
    _next_thread: u64,
}

impl From<TeamV9> for Team {
    fn from(old: TeamV9) -> Self {
        let mut team = Team::default();
        team.hidden_slides = old.hidden_slides;
        for t in old.threads {
            let id = ThreadId(team.mint());
            let messages = t
                .messages
                .into_iter()
                .map(|m| Message {
                    id: MessageId(team.mint()),
                    author: m.author,
                    text: m.text,
                    at: m.at,
                    edited: m.edited,
                    mentions: Vec::new(),
                })
                .collect();
            team.threads.push(Thread {
                id,
                pin: t.pin,
                messages,
                resolved: t.resolved,
                read_by: t.read_by,
            });
        }
        for v in old.versions {
            let id = VersionId(team.mint());
            team.versions.push(Version {
                id,
                name: v.name,
                at: v.at,
                bytes: v.bytes,
                files: BTreeSet::new(),
            });
        }
        team
    }
}

#[derive(Deserialize)]
struct BoardV9 {
    id: BoardId,
    name: String,
    camera: Camera,
    doc: super::DocOld<super::v10::ElementV10>,
    sketch: bool,
    team: TeamV9,
}

#[derive(Deserialize)]
struct SetV9 {
    _version: u32,
    boards: Vec<BoardV9>,
    next_id: u64,
}

/// Os bytes de um `BoardSet` no formato 9 (W9), já no formato actual: a equipa com ids.
pub(crate) fn read_v9(bytes: &[u8]) -> Result<BoardSet, String> {
    let old: SetV9 = postcard::from_bytes(bytes).map_err(|e| e.to_string())?;
    let boards = old
        .boards
        .into_iter()
        .map(|b| {
            let votes = super::v10::votes_v10(&b.doc);
            let mut doc = super::doc_old(b.doc, &super::v10::element_v10);
            doc.reindex();
            let mut team: Team = b.team.into();
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
