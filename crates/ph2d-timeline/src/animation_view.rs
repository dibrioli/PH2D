//! ⭐⭐ **A ANIMAÇÃO EM VISTA** (doc ObjectsAndComponents 01 §6, escolha 9.1 = A).
//!
//! A Timeline mostra a animação do objecto seleccionado, ou a do ancestral mais próximo que tenha
//! uma, ou a da Cena — e o cabeçalho diz de quem é (*«Animação de: Herói ▾»*). As chaves vão para
//! a animação em vista (escolha do dono, 08/10): o clip activo é sempre um dela.
//!
//! Seguir a selecção é VISTA, não edição: trocar o clip activo por aqui não é passo de undo (um
//! clique num objecto não pode encher a pilha da timeline). O ▾ fixa outra animação até a
//! selecção mudar.

use ph2d_core::Playhead;

use crate::doc::SCENE;
use crate::intent_apply::edit;
use crate::intent_apply::intent_loop_sync::sync_loop;
use crate::snapshot::TimelineViewSnapshot;
use crate::state::TimelineState;

/// O que o cabeçalho e o Inspector pedem à animação de um objecto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationIntent {
    /// Mostrar a animação do dono ([`SCENE`] = a da Cena) — o ▾; fica até a selecção mudar.
    Show(u64),
    /// Dar a `owner` o componente Animação: o primeiro clip dele (4 s), e a vista vai para lá.
    Add(u64),
    /// Tirar o componente Animação de `owner`: os clips todos, num passo de undo.
    Remove(u64),
}

/// A vista: de quem é a animação que o painel mostra (sessão, nunca gravada).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnimationView {
    /// O dono em vista ([`SCENE`] por omissão).
    pub owner: u64,
    /// A selecção primária do último quadro ([`SCENE`] = nenhuma).
    pub selected: u64,
    /// O ▾ fixou `(selecção, dono)`: vale enquanto a selecção for aquela.
    pub pin: Option<(u64, u64)>,
}

/// O cabeçalho *«Animação de: X ▾»* — os nomes vêm do mundo, e quem o enche é a shell
/// (`ph2d-timeline-persist::publish_animation_header`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnimationHeader {
    /// O dono em vista ([`SCENE`] = a Cena).
    pub owner: u64,
    /// As escolhas do ▾, de fora para dentro: a Cena (nome vazio — o painel diz «Cena» pela
    /// i18n), cada ancestral com Animação, e o seleccionado se tiver uma.
    pub choices: Vec<(u64, String)>,
    /// O seleccionado SEM Animação — o ▾ oferece criá-la nele.
    pub offer: Option<(u64, String)>,
}

/// De quem é a animação que a Timeline mostra: o seleccionado, ou o ancestral mais próximo com
/// Animação (`ancestors` do pai para a raiz), ou a Cena.
#[must_use]
pub fn owner_for(state: &TimelineState, selected: Option<u64>, ancestors: &[u64]) -> u64 {
    selected
        .into_iter()
        .chain(ancestors.iter().copied())
        .find(|&e| state.doc.has_animation(e))
        .unwrap_or(SCENE)
}

/// **O passo de cada quadro** (antes dos pedidos do painel): a vista segue a selecção, salvo o ▾
/// fixado sobre ESTA selecção.
pub fn follow(
    state: &mut TimelineState,
    playhead: &mut Playhead,
    selected: Option<u64>,
    ancestors: &[u64],
) {
    let sel = selected.unwrap_or(SCENE);
    state.view.selected = sel;
    let wanted = match state.view.pin {
        Some((s, o)) if s == sel && (o == SCENE || state.doc.has_animation(o)) => o,
        _ => {
            state.view.pin = None;
            owner_for(state, selected, ancestors)
        }
    };
    show(state, playhead, wanted);
}

/// Põe `owner` em vista; o clip activo passa a ser um dele (o primeiro), se ainda não for.
fn show(state: &mut TimelineState, playhead: &mut Playhead, owner: u64) {
    let owner = if owner == SCENE || state.doc.has_animation(owner) {
        owner
    } else {
        SCENE
    };
    state.view.owner = owner;
    if state.doc.clip_belongs_to(state.doc.active_index(), owner) {
        return;
    }
    let first = state.doc.clips_of(owner).next();
    if let Some(i) = first {
        state.doc.set_active(i);
        // Um `KeyId` só vale no clip que o deu — a mesma razão do `SetActiveClip`.
        state.selection.clear();
        sync_loop(&state.doc, playhead, state.keys_mode);
    }
}

/// Aplica um [`AnimationIntent`] (chamado pelo `apply_intent`).
pub(crate) fn apply(state: &mut TimelineState, playhead: &mut Playhead, intent: AnimationIntent) {
    match intent {
        AnimationIntent::Show(owner) => {
            state.view.pin = Some((state.view.selected, owner));
            show(state, playhead, owner);
        }
        AnimationIntent::Add(owner) => {
            let nova = owner != SCENE && !state.doc.has_animation(owner);
            if let (true, Err(r)) = (nova, state.doc.clip_room(owner)) {
                state.refusals.push(r);
            } else if nova {
                edit(state, |doc, sel| {
                    let name = doc.fresh_clip_name_in(owner);
                    if let Some(i) = doc.add_clip_for(name, owner) {
                        doc.set_clip_length_override(i, Some(crate::DEFAULT_DURATION_SECONDS));
                        doc.set_active(i);
                        sel.clear();
                    }
                });
            }
            state.view.pin = Some((state.view.selected, owner));
            show(state, playhead, owner);
        }
        AnimationIntent::Remove(owner) => {
            edit(state, |doc, sel| {
                if doc.remove_animation(owner) > 0 {
                    sel.clear();
                }
            });
            state.view.pin = None;
            // A Cena por agora; o `follow` do quadro seguinte acha o ancestral certo.
            show(state, playhead, SCENE);
        }
    }
}

/// **A linha da binding `target` é desta animação?** Sim se algum clip dela tem a track — ou se
/// NENHUM clip a tem (uma binding nova, à espera da 1.ª chave, aparece em qualquer vista). O dope
/// sheet do Herói não lista o que só a Cena anima, e vice-versa.
pub(crate) fn row_belongs(
    doc: &crate::TimelineDoc,
    owner: u64,
    target: ph2d_anim::AnimTarget,
) -> bool {
    let mut keyed_somewhere = false;
    for (i, c) in doc.clips().iter().enumerate() {
        if c.clip.track(target).is_some() {
            if doc.clip_belongs_to(i, owner) {
                return true;
            }
            keyed_somewhere = true;
        }
    }
    !keyed_somewhere
}

/// A lista do seletor: os clips da animação em vista, com o índice do documento de cada um.
pub(crate) fn fill_clips(snap: &mut TimelineViewSnapshot, state: &TimelineState) {
    snap.clips.clear();
    snap.clip_index.clear();
    for i in state.doc.clips_of(state.view.owner) {
        snap.clips.push(state.doc.clips()[i].name.clone());
        snap.clip_index.push(i);
    }
    snap.active_clip = state.doc.active_index();
}

impl TimelineViewSnapshot {
    /// A linha do seletor do clip activo (o `active_clip` é um índice do DOCUMENTO).
    #[must_use]
    pub fn active_row(&self) -> usize {
        self.clip_index
            .iter()
            .position(|&i| i == self.active_clip)
            .unwrap_or(0)
    }

    /// O nome do clip `index` do documento, se estiver na lista em vista.
    #[must_use]
    pub fn clip_name(&self, index: usize) -> Option<&str> {
        let row = self.clip_index.iter().position(|&i| i == index)?;
        self.clips.get(row).map(String::as_str)
    }
}
