//! **O CACHE de geometria** que as três rotas de vetor vivo partilham — o `source.shape`
//! paramétrico, os glifos do `source.text` e os desenhos do `source.object`.
//!
//! Irmão do [`super::motion_shape_gen`], cortado pelo teto de 600 LOC da shell e por
//! ASSUNTO: lá mora *como uma forma vira geometria*, aqui mora *onde ela fica e quando é
//! largada* — que é a pergunta que um laço de jogo a correr horas faz.

use std::collections::BTreeMap;

use ph2d_vec_scene::VecPath;

/// A content-addressed cache of shape geometry: a shape's `VecPath` is interned
/// under its [`shape_key`], and an instance carries the resulting handle in its
/// `geometry_id` column (`0` stays "no geometry" — the byte-identical fallback for
/// every stream without the column). Kept across frames, so a static shape builds
/// ONCE.
///
/// ⚠️ **As entradas COM CHAVE são varridas todo quadro** ([`Self::sweep`]) — e o
/// motivo é um crash medido (Enio, 2026-08-21: `wgpu error: Out of Memory` no quadro
/// **19706** da cena `=76`). Enquanto um param de forma conduzido por fio não
/// desenhava nada, este cache só crescia quando o artista MEXIA num slider, e a nota
/// que estava aqui — *"o crescimento é um por descritor visitado, nomeado e limitado
/// pela sessão"* — era verdadeira. Com o Trim animado ele passou a crescer **uma
/// entrada por QUADRO**, e o que mata não é esta tabela (uns 500 B por entrada): é o
/// [`crate::motion_shape_bake`], que assa **uma textura de GPU por `geometry_id`**.
/// *Um cache cuja chave pode mudar a 60 Hz não é um cache — é uma fuga com memória.*
///
/// ⚠️ **O handle é um CONTADOR, nunca um índice.** Ele era `by_handle.len()`, e um
/// `Vec` indexado por posição não pode perder um elemento do meio sem renomear todos
/// os que vêm depois — era isso que bloqueava a poda. Um `BTreeMap` chaveado pelo
/// contador remove no meio sem tocar em ninguém, e o handle continua único para
/// sempre (nunca é reciclado: um id reciclado apontaria para outra forma dentro do
/// mesmo quadro em que alguém ainda o carrega).
#[derive(Default)]
pub(crate) struct VecPathStore {
    by_handle: BTreeMap<u32, VecPath>, // handle -> geometria
    handle_of: BTreeMap<String, u32>,  // key -> handle
    /// O handle seguinte a emitir. Monotónico; `0` fica reservado para *sem geometria*.
    next: u32,
    /// As chaves PEDIDAS neste quadro — o que a [`Self::sweep`] preserva.
    live: std::collections::BTreeSet<String>,
}

impl VecPathStore {
    /// The `VecPath` for a `geometry_id` handle, or `None` for 0 / unknown.
    pub(crate) fn get(&self, handle: u32) -> Option<&VecPath> {
        self.by_handle.get(&handle)
    }

    /// Intern a shape under its content key, building it once. Returns the handle
    /// (`>= 1`). Identical keys share the stored `VecPath`, and pedir a chave é o que
    /// a mantém viva na varredura do quadro.
    pub(crate) fn intern(&mut self, key: &str, build: impl FnOnce() -> VecPath) -> u32 {
        self.live.insert(key.to_owned());
        if let Some(&h) = self.handle_of.get(key) {
            return h;
        }
        self.next += 1;
        let h = self.next;
        self.by_handle.insert(h, build());
        self.handle_of.insert(key.to_owned(), h);
        h
    }

    /// O handle JÁ internado sob esta chave, ou `None`. É a metade de CONSULTA do
    /// [`intern`](Self::intern), e ela existe porque o `source.text` precisa de
    /// decidir se constrói: um glifo sem contorno (o espaço) não vira `VecPath`
    /// nenhum, e um `intern` que recebe um closure não tem como dizer *"não havia
    /// nada a guardar"*. Perguntar antes deixa a construção do lado de quem sabe
    /// desistir, sem uma segunda tabela para as ausências.
    ///
    /// ⚠️ **`&mut self`, e não por descuido:** consultar é PEDIR, e um glifo que só
    /// é consultado (porque já estava internado) tem de contar como vivo — senão a
    /// varredura apagaria exactamente as geometrias que estão a ser usadas todo
    /// quadro, e cada letra seria reconstruída em cada um deles.
    pub(crate) fn handle_for(&mut self, key: &str) -> Option<u32> {
        let h = self.handle_of.get(key).copied();
        if h.is_some() {
            self.live.insert(key.to_owned());
        }
        h
    }

    /// Store a `VecPath` with NO content key, returning a fresh handle (`>= 1`).
    /// The keyed [`intern`](Self::intern) dedups by descriptor for `source.shape`
    /// primitives; a `source.object` DOCUMENT vector has no descriptor string, and
    /// its own content-cache ([`crate::motion_object_bake::ObjectBake`], keyed by
    /// `VecPathId` + content) already decides WHEN to re-store, so this just parks
    /// the current geometry and hands back the handle the membrane emits as
    /// `geometry_id`. One entry per content CHANGE (a static object stores once).
    ///
    /// ⚠️ **Uma entrada SEM chave não é varrida** — ela não tem como dizer se ainda é
    /// pedida, e quem a governa é o `ObjectBake` (que já pareia cada `acquire` com um
    /// `release`). É por isso que a varredura é *das entradas com chave*, e não *de
    /// tudo*.
    pub(crate) fn push(&mut self, path: VecPath) -> u32 {
        self.next += 1;
        let h = self.next;
        self.by_handle.insert(h, path);
        h
    }

    /// **Esquece uma geometria SEM chave** — o par do [`Self::push`], para quem a
    /// empurrou a poder devolvê-la.
    ///
    /// ⚠️ Uma entrada sem chave não pode ser varrida (ela não sabe dizer se ainda é
    /// pedida), então o dono é que a larga — exactamente como já larga a textura dela.
    /// O `ObjectBake` chama isto nos DOIS sítios em que faz `release`: quando o objeto
    /// desaparece da cena e quando ele é re-assado. Sem isto, **mover ou girar um
    /// objeto vetorial deixava um `VecPath` morto por quadro** (a chave do bake inclui
    /// a transformação), e o comentário de lá dizia-o em voz alta: *"the store slot
    /// goes dead but is not reclaimed"*. Num laço de jogo que corre horas, «não
    /// reclamado» é «fuga».
    pub(crate) fn forget(&mut self, handle: u32) {
        self.by_handle.remove(&handle);
    }

    /// **Esquece as geometrias COM CHAVE que ninguém pediu neste quadro** e devolve os
    /// handles largados. Chamada uma vez por quadro, depois de todas as membranas
    /// publicarem ([`super::motion_externals::publish_all`]).
    ///
    /// Devolve os handles para quem tem caches por-handle os poder libertar; hoje quem
    /// consome é o [`crate::motion_shape_bake`], que segura uma TEXTURA por handle.
    pub(crate) fn sweep(&mut self) -> Vec<u32> {
        let mut dropped = Vec::new();
        self.handle_of.retain(|key, h| {
            let keep = self.live.contains(key);
            if !keep {
                dropped.push(*h);
            }
            keep
        });
        for h in &dropped {
            self.by_handle.remove(h);
        }
        self.live.clear();
        dropped
    }

    /// Quantas geometrias o store guarda — a sonda de que a varredura precisa para
    /// não ser uma alegação.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.by_handle.len()
    }
}
