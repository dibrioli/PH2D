//! **A equipa dos formatos 10–15** (W11–W23 bloco 5): a mensagem SEM as menções — o 16 deu-lhe
//! `Message::mentions` (W23 B14, doc 17 §2.3). O quadro do 15 lê-se pelo `legacy_v16.rs`.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::{ElementId, Message, MessageId, Pin, Poll, PollId, Team, Thread, ThreadId, Version};

/// A mensagem dos formatos 10–15.
#[derive(Deserialize)]
pub(super) struct MessageV15 {
    id: MessageId,
    author: String,
    text: String,
    at: i64,
    edited: bool,
}

/// O fio dos formatos 10–15.
#[derive(Deserialize)]
pub(super) struct ThreadV15 {
    id: ThreadId,
    pin: Pin,
    messages: Vec<MessageV15>,
    resolved: bool,
    read_by: Vec<String>,
}

impl From<ThreadV15> for Thread {
    fn from(t: ThreadV15) -> Self {
        Self {
            id: t.id,
            pin: t.pin,
            messages: t
                .messages
                .into_iter()
                .map(|m| Message {
                    id: m.id,
                    author: m.author,
                    text: m.text,
                    at: m.at,
                    edited: m.edited,
                    mentions: Vec::new(),
                })
                .collect(),
            resolved: t.resolved,
            read_by: t.read_by,
        }
    }
}

/// Os fios dos formatos 10–15, já de hoje.
pub(super) fn threads_v15(v: Vec<ThreadV15>) -> Vec<Thread> {
    v.into_iter().map(Thread::from).collect()
}

/// A `Team` dos formatos 12–15 (a 11 tinha outra `Version`).
#[derive(Deserialize)]
pub(super) struct TeamV15 {
    threads: Vec<ThreadV15>,
    versions: Vec<Version>,
    hidden_slides: BTreeSet<ElementId>,
    next_id: u64,
    polls: Vec<Poll>,
    shown: Option<PollId>,
}

impl From<TeamV15> for Team {
    fn from(t: TeamV15) -> Self {
        let mut team = Self::from_v10(
            threads_v15(t.threads),
            t.versions,
            t.hidden_slides,
            t.next_id,
        );
        (team.polls, team.shown) = (t.polls, t.shown);
        team
    }
}
