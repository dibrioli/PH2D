//! **A selagem da réplica** ([ADR-0182] §2–§3), módulo filho de [`super`]: com o modo privado ligado, o
//! TEXTO de uma nota e o TAMANHO dela (que cresce com o texto — diria o comprimento) saem selados, só
//! para o cofre do anfitrião; a posição continua pública (com o tamanho de antes). O texto de uma nota
//! escondida de mim não se escreve aqui (a escrita desfaz-se ao ser capturada — negrito, desfazer,
//! colar, qualquer caminho). O retrato e as versões levam o que os OUTROS têm.
//!
//! [ADR-0182]: ../../../docs/architecture/decisions/0182-o-que-e-de-uma-pessoa-mora-no-cofre-do-anfitriao.md

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{Board, BoardOp, Element, ElementId, Field, RichText, Version};

use super::{Lamport, OutKey, Replica, Write};

/// O registo selado de uma escrita (o texto ou a caixa de uma nota), e o carimbo.
pub(crate) fn sealed_field(w: &Write) -> Option<(ElementId, Field, Lamport)> {
    match w {
        Write::Set {
            id,
            field: field @ (Field::Text | Field::Box),
            at,
            ..
        } => Some((*id, *field, *at)),
        _ => None,
    }
}

pub(super) fn is_note(el: &Element) -> bool {
    el.shape().is_some_and(|s| s.kind.is_note())
}

/// O modo privado nesta ponta.
#[derive(Clone, Debug, Default)]
pub(super) struct Sealed {
    pub(super) on: bool,
    /// Por enviar ao cofre.
    pub(super) out: Vec<Write>,
    /// (nota, registo) meus por revelar → o carimbo do último selado.
    pub(super) mine: BTreeMap<(ElementId, Field), Lamport>,
    /// (nota, registo) → o que os OUTROS têm (o valor e o carimbo).
    pub(super) public: BTreeMap<(ElementId, Field), (Vec<u8>, Lamport)>,
    /// As notas de outros com o texto escondido de mim.
    pub(super) unread: BTreeSet<ElementId>,
}

/// O que uma mudança de um registo vira.
pub(super) enum Seal {
    Public(Write),
    Sealed(Write),
    /// A posição pública (com o tamanho de antes) e a caixa inteira selada, com o carimbo seguinte.
    Split(Write, Write),
    /// Escrita no texto de uma nota que não leio: desfaz-se.
    Revert,
}

impl Replica {
    /// Liga/desliga a selagem (o modo privado da sala).
    pub fn set_sealing(&mut self, on: bool) {
        self.sealed.on = on;
    }

    /// As notas de outros com o texto escondido de mim (não se escreve nelas).
    pub fn set_unread(&mut self, ids: BTreeSet<ElementId>) {
        self.sealed.unread = ids;
    }

    /// As escritas seladas por enviar ao cofre.
    pub fn take_sealed(&mut self) -> Vec<Write> {
        std::mem::take(&mut self.sealed.out)
    }

    /// As minhas notas por revelar.
    #[must_use]
    pub fn my_sealed(&self) -> BTreeSet<ElementId> {
        self.sealed.mine.keys().map(|(id, _)| *id).collect()
    }

    /// Estas notas foram reveladas: o que tinham selado já é de todos (e não se reenvia depois de uma
    /// queda: ou saiu, ou perdeu para um mais novo e nunca se mostra).
    pub fn released(&mut self, ids: impl IntoIterator<Item = ElementId>) {
        let ids: BTreeSet<ElementId> = ids.into_iter().collect();
        let gone: Vec<((ElementId, Field), Lamport)> = self
            .sealed
            .mine
            .iter()
            .filter(|((id, _), _)| ids.contains(id))
            .map(|(k, at)| (*k, *at))
            .collect();
        for ((id, f), at) in gone {
            self.sealed.mine.remove(&(id, f));
            let key = OutKey::Set(id, f);
            if self.outbox.get(&key).and_then(sealed_field) == Some((id, f, at)) {
                self.outbox.remove(&key);
            }
        }
        self.sealed.public.retain(|(id, _), _| !ids.contains(id));
    }

    /// O carimbo de um registo como os OUTROS o têm (o de um registo meu selado é o de antes dele): o
    /// anfitrião compara-o com uma escrita selada que chega.
    #[must_use]
    pub fn public_stamp(&self, id: ElementId, f: Field) -> Lamport {
        if self.sealed.mine.contains_key(&(id, f))
            && let Some((_, at)) = self.sealed.public.get(&(id, f))
        {
            return *at;
        }
        self.stamp(id, f)
    }

    /// Uma nota que NASCE com texto no modo privado: nasce vazia para todos, e o texto segue selado
    /// com o carimbo seguinte (que esta função tira).
    pub(super) fn seal_born(&mut self, born: &mut Element, at: Lamport) -> Option<Write> {
        let text =
            (self.sealed.on && is_note(born) && born.shape().is_some_and(|s| !s.text.is_empty()))
                .then(|| born.read(Field::Text))
                .flatten()?;
        if let Some(s) = born.shape_mut() {
            s.text = RichText::default();
        }
        let id = born.id;
        let blank = born.read(Field::Text).unwrap_or_default();
        self.sealed
            .public
            .entry((id, Field::Text))
            .or_insert((blank, at));
        let at = self.tick();
        Some(Write::Set {
            id,
            field: Field::Text,
            value: text,
            at,
        })
    }

    /// O que a mudança do registo `f` de `before` para `now` vira (`at` já tirado).
    pub(super) fn seal_set(
        &mut self,
        before: &Element,
        now: &Element,
        f: Field,
        value: Vec<u8>,
        at: Lamport,
    ) -> Seal {
        let id = now.id;
        if f == Field::Text && self.sealed.unread.contains(&id) {
            return Seal::Revert;
        }
        let set = |value: Vec<u8>, at: Lamport| Write::Set {
            id,
            field: f,
            value,
            at,
        };
        let mine = self.sealed.mine.contains_key(&(id, Field::Text));
        let seal =
            self.sealed.on && is_note(now) && (f == Field::Text || (f == Field::Box && mine));
        if !seal {
            return Seal::Public(set(value, at));
        }
        let old = (before.read(f).unwrap_or_default(), self.stamp(id, f));
        let (public, _) = self.sealed.public.entry((id, f)).or_insert(old).clone();
        if f == Field::Text {
            return Seal::Sealed(set(value, at));
        }
        // A caixa: a posição nova com o tamanho que os outros conhecem.
        let mut shown = now.clone();
        let mut known = now.clone();
        known.write(Field::Box, &public);
        (shown.w, shown.h) = (known.w, known.h);
        let pv = shown.read(Field::Box).unwrap_or_default();
        if pv == public {
            return Seal::Sealed(set(value, at));
        }
        self.sealed.public.insert((id, f), (pv.clone(), at));
        let sealed_at = self.tick();
        Seal::Split(set(pv, at), set(value, sealed_at))
    }

    /// Uma escrita mais nova de um registo meu selado chegou: a minha perdeu (e não se marca mais).
    pub(super) fn overtaken(&mut self, id: ElementId, f: Field, at: Lamport) {
        if self
            .sealed
            .mine
            .get(&(id, f))
            .is_some_and(|mine| at > *mine)
        {
            self.sealed.mine.remove(&(id, f));
            self.sealed.public.remove(&(id, f));
        }
    }

    /// O quadro como os OUTROS o têm (os meus registos selados com o valor público), e os carimbos
    /// deles. `None` = nada meu selado (o quadro é o mesmo).
    pub(super) fn masked(
        &self,
        board: &Board,
    ) -> Option<(Board, Vec<((ElementId, Field), Lamport)>)> {
        let hide: Vec<_> = self
            .sealed
            .mine
            .keys()
            .filter_map(|k| self.sealed.public.get(k).map(|p| (*k, p)))
            .collect();
        if hide.is_empty() {
            return None;
        }
        let mut b = board.clone();
        mask_doc(&mut b, hide.iter().map(|(k, (v, _))| (*k, v)));
        Some((b, hide.iter().map(|(k, (_, at))| (*k, *at)).collect()))
    }

    /// Uma versão guardada com o modo privado leva o quadro como TODOS o vêem (a da autora também:
    /// uma versão é do quadro da sala).
    pub(super) fn mask_version(&self, v: &mut Version) {
        if self.sealed.mine.is_empty() {
            return;
        }
        let Ok(mut b) = v.read() else {
            return;
        };
        let hide = self
            .sealed
            .mine
            .keys()
            .filter_map(|k| self.sealed.public.get(k).map(|(val, _)| (*k, val)));
        mask_doc(&mut b, hide);
        v.bytes = b.version_bytes();
    }
}

fn mask_doc<'a>(b: &mut Board, hide: impl Iterator<Item = ((ElementId, Field), &'a Vec<u8>)>) {
    for ((id, f), value) in hide {
        if let Some(mut el) = b.doc.get_any(id).cloned() {
            el.write(f, value);
            let _ = BoardOp::Put(el).apply(&mut b.doc);
        }
    }
}
