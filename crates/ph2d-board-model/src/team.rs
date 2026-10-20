//! **A equipa assíncrona** (W8, `docs/MiroClone/06_equipa_e_saida.md` §2): os comentários em fio, as
//! versões nomeadas e os frames que não se apresentam — DOCUMENTO do quadro, mas fora do desfazer e das
//! cópias de elementos (K9): a conversa é SOBRE o quadro.

use std::collections::BTreeSet;
use std::io::Write as _;

use serde::{Deserialize, Serialize};

use crate::{Board, BoardDoc, ElementId, ElementKind, SESSION_ID_BIT};

/// Identidade de um fio dentro do seu quadro. Nunca reusada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThreadId(pub u64);

/// Identidade de uma mensagem dentro do seu quadro (formato 10): a posição na lista não serve — duas
/// respostas ao mesmo tempo colidem —, nem `(at, autor)` (segundos, e o nome é texto livre). O
/// `Default` (0) é «nenhuma»: o [`Team::mint`] nunca o cunha.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MessageId(pub u64);

/// Identidade de uma versão nomeada dentro do seu quadro (formato 10). O `Default` (0) é «nenhuma».
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct VersionId(pub u64);

/// Um passo da equipa que se desfaz com o quadro (o dono, 10/10: *«não temos undo para Resolve»*):
/// resolver/reabrir e mover a bolha. Escrever, responder e apagar mensagens continuam fora do desfazer
/// (o idioma do Miro: um comentário é conversa, não desenho).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TeamOp {
    /// O fio fica resolvido (`true`) ou aberto.
    Resolve(ThreadId, bool),
    /// O fio passa a estar preso aqui.
    Pin(ThreadId, Pin),
}

/// Onde um fio está preso (K2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pin {
    /// Num objecto, em fracção `[u, v]` da caixa dele ANTES de rodar: anda, roda e escala com ele.
    Element { id: ElementId, at: [f64; 2] },
    /// Num ponto do mundo do quadro.
    Point([f64; 2]),
}

/// ⭐ W23 B14 — **quem uma menção chama** (doc 17 §2.3): uma pessoa (o nome dela, a identidade de
/// pessoa do quadro — a do `author` e do `read_by`, K6) ou toda a gente. É a CHAVE da menção; o texto
/// da mensagem mostra o [`Who::label`] dela (uma chave e um texto do mesmo tipo é defeito à espera).
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Who {
    Person(String),
    Everyone,
}

impl Who {
    /// O que o texto mostra: `@Rui`, `@everyone`.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Person(n) => format!("@{n}"),
            Self::Everyone => EVERYONE.to_owned(),
        }
    }

    /// Chama `me`?
    #[must_use]
    pub fn calls(&self, me: &str) -> bool {
        match self {
            Self::Person(n) => n == me,
            Self::Everyone => true,
        }
    }
}

/// O texto da menção de toda a gente (o `@board` do Miro; dado, não i18n — é o que se escreve).
pub const EVERYONE: &str = "@everyone";

/// Uma mensagem de um fio (K4): texto simples, o nome de quem a escreveu e o instante (segundos
/// UNIX — a vista mostra «há quanto tempo», que não pede fuso).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub author: String,
    pub text: String,
    pub at: i64,
    pub edited: bool,
    /// W23 B14 (formato 16) — quem ela chama. Só fica quem o texto ainda mostra
    /// ([`Message::keep_mentions`]).
    pub mentions: Vec<Who>,
}

impl Message {
    /// Chama `me` (pelo nome, ou a toda a gente) — e não é minha.
    #[must_use]
    pub fn calls(&self, me: &str) -> bool {
        self.author != me && self.mentions.iter().any(|w| w.calls(me))
    }

    /// ⭐ Depois de o texto mudar: só fica chamado quem o texto ainda mostra. Função do texto ⇒ cada
    /// par chega à mesma lista sem a lista viajar (o `TeamWrite::Text` leva só o texto).
    pub fn keep_mentions(&mut self) {
        let text = &self.text;
        self.mentions.retain(|w| text.contains(&w.label()));
        self.mentions.sort();
        self.mentions.dedup();
    }
}

/// Um fio de comentários: a 1.ª mensagem e as respostas.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Thread {
    pub id: ThreadId,
    pub pin: Pin,
    pub messages: Vec<Message>,
    pub resolved: bool,
    /// Quem já o leu (K6): o «novo» do Miro é por PESSOA — o colega que abre o ficheiro vê a azul o
    /// que ainda não leu.
    pub read_by: Vec<String>,
}

/// Uma versão nomeada (R1): os bytes de um [`crate::BoardSet`] de UM quadro (com a versão de formato
/// dentro — o leitor dos formatos antigos abre-a), comprimidos com deflate 1 (§4.1: 2,1 MB e 14,5 ms
/// para 100 mil elementos).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub name: String,
    pub at: i64,
    pub bytes: Vec<u8>,
    /// W14 — os ficheiros de imagem que ela mostra: os bytes ficam no quadro enquanto ela existir
    /// ([`Board::files_in_use`]); a versão não os copia.
    pub files: BTreeSet<crate::FileId>,
}

/// O que a equipa guarda num quadro.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub threads: Vec<Thread>,
    pub versions: Vec<Version>,
    /// Os frames marcados «Hide from presentation» (P1).
    pub hidden_slides: BTreeSet<ElementId>,
    /// O contador dos ids de fios, mensagens, versões e votações fora de sessão ([`Team::mint`]).
    next_id: u64,
    /// W13 — as votações acabadas, pela ordem em que acabaram (`poll.rs`, ADR-0183).
    pub polls: Vec<crate::Poll>,
    /// W13 — a sessão cujo resultado o quadro mostra (o selo); `None` = nenhuma.
    pub shown: Option<crate::PollId>,
    /// Em sessão ao vivo: os ids sorteados ([`Team::set_session`]).
    #[serde(skip)]
    session: crate::session::Lottery,
}

/// Uma mensagem nova: só fica chamado quem o texto mostra.
fn message(id: MessageId, author: &str, text: &str, who: &[Who], at: i64) -> Message {
    let mut m = Message {
        id,
        author: author.to_owned(),
        text: text.to_owned(),
        at,
        edited: false,
        mentions: who.to_vec(),
    };
    m.keep_mentions();
    m
}

impl Thread {
    /// Este fio está por ler para `me` (K6): não é meu, e eu ainda não o abri.
    #[must_use]
    pub fn unread_for(&self, me: &str) -> bool {
        !self.read_by.iter().any(|r| r == me)
    }

    /// ⭐ W23 B14 — **a mensagem `i` é PARA `me`** (a porta do aviso, do selo e do filtro): chama-me,
    /// ou é uma resposta de outra pessoa num fio onde eu já tinha escrito (o «Feed» do Miro; dono 20/10).
    #[must_use]
    pub fn for_me_at(&self, i: usize, me: &str) -> bool {
        let Some(m) = self.messages.get(i) else {
            return false;
        };
        m.author != me && (m.calls(me) || self.messages[..i].iter().any(|p| p.author == me))
    }

    /// Há neste fio alguma mensagem para `me` ([`Thread::for_me_at`]).
    #[must_use]
    pub fn for_me(&self, me: &str) -> bool {
        (0..self.messages.len()).any(|i| self.for_me_at(i, me))
    }

    /// Onde a bolha aponta, no mundo; `None` = o objecto apagado ou escondido (K3: o fio some com ele
    /// e volta com o desfazer).
    #[must_use]
    pub fn anchor(&self, doc: &BoardDoc) -> Option<[f64; 2]> {
        self.pin.anchor(doc)
    }
}

impl Pin {
    /// Onde o pino aponta, no mundo; `None` = o objecto apagado ou escondido.
    #[must_use]
    pub fn anchor(&self, doc: &BoardDoc) -> Option<[f64; 2]> {
        match *self {
            Pin::Point(p) => Some(p),
            Pin::Element { id, at } => {
                let el = doc.get(id).filter(|e| !e.is_hidden())?;
                Some(el.rotate([el.x + at[0] * el.w, el.y + at[1] * el.h]))
            }
        }
    }

    /// O pino de um clique em `p` sobre `on` (o objecto de cima sob ele, se há): numa seta (sem caixa:
    /// a geometria é a rota) fica no ponto.
    #[must_use]
    pub fn at(doc: &BoardDoc, on: Option<ElementId>, p: [f64; 2]) -> Self {
        let Some(el) = on.and_then(|id| doc.get(id)) else {
            return Pin::Point(p);
        };
        if matches!(el.kind, ElementKind::Connector(_)) || el.w <= 0.0 || el.h <= 0.0 {
            return Pin::Point(p);
        }
        let q = el.unrotate(p);
        Pin::Element {
            id: el.id,
            at: [(q[0] - el.x) / el.w, (q[1] - el.y) / el.h],
        }
    }
}

impl Team {
    /// ⭐ A porta ÚNICA dos ids da equipa (fios, mensagens e versões partilham-na): o contador local;
    /// numa sessão ao vivo, sorteado com o [`SESSION_ID_BIT`] e sem colidir com nenhum que já exista
    /// (o idioma de [`BoardDoc::mint_id`]).
    pub fn mint(&mut self) -> u64 {
        while let Some(r) = self.session.draw() {
            let id = r | SESSION_ID_BIT;
            if !self.uses(id) {
                return id;
            }
        }
        self.next_id += 1;
        self.next_id
    }

    fn uses(&self, id: u64) -> bool {
        self.versions.iter().any(|v| v.id.0 == id)
            || self.polls.iter().any(|p| p.id.0 == id)
            || self
                .threads
                .iter()
                .any(|t| t.id.0 == id || t.messages.iter().any(|m| m.id.0 == id))
    }

    /// Entra (`Some(semente)`) ou sai (`None`) de uma sessão ao vivo (ADR-0181 §5). O
    /// [`Board::set_session`] liga o documento e a equipa juntos.
    pub fn set_session(&mut self, seed: Option<u64>) {
        self.session = crate::session::Lottery::new(seed);
    }

    /// A equipa do formato 10 (sem votações): o leitor dele (`legacy_v10.rs`) não vê os campos privados.
    pub(crate) fn from_v10(
        threads: Vec<Thread>,
        versions: Vec<Version>,
        hidden_slides: BTreeSet<ElementId>,
        next_id: u64,
    ) -> Self {
        Self {
            threads,
            versions,
            hidden_slides,
            next_id,
            ..Self::default()
        }
    }

    /// Abre um fio novo com a 1.ª mensagem; quem o escreve já o leu. Texto vazio (só espaços) não abre.
    pub fn open(&mut self, pin: Pin, author: &str, text: &str, at: i64) -> Option<ThreadId> {
        self.open_calling(pin, author, text, &[], at)
    }

    /// [`Team::open`] a chamar `who` (W23 B14; só fica quem o texto mostra).
    pub fn open_calling(
        &mut self,
        pin: Pin,
        author: &str,
        text: &str,
        who: &[Who],
        at: i64,
    ) -> Option<ThreadId> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        let id = ThreadId(self.mint());
        let msg = MessageId(self.mint());
        self.threads.push(Thread {
            id,
            pin,
            messages: vec![message(msg, author, text, who, at)],
            resolved: false,
            read_by: vec![author.to_owned()],
        });
        Some(id)
    }

    #[must_use]
    pub fn thread(&self, id: ThreadId) -> Option<&Thread> {
        self.threads.iter().find(|t| t.id == id)
    }

    pub fn thread_mut(&mut self, id: ThreadId) -> Option<&mut Thread> {
        self.threads.iter_mut().find(|t| t.id == id)
    }

    /// Responde a um fio; os outros passam a tê-lo por ler (só quem responde o leu).
    pub fn reply(&mut self, id: ThreadId, author: &str, text: &str, at: i64) -> bool {
        self.reply_calling(id, author, text, &[], at)
    }

    /// [`Team::reply`] a chamar `who` (W23 B14).
    pub fn reply_calling(
        &mut self,
        id: ThreadId,
        author: &str,
        text: &str,
        who: &[Who],
        at: i64,
    ) -> bool {
        let text = text.trim();
        if text.is_empty() || self.thread(id).is_none() {
            return false;
        }
        let msg = MessageId(self.mint());
        let Some(t) = self.thread_mut(id) else {
            return false;
        };
        t.messages.push(message(msg, author, text, who, at));
        t.read_by = vec![author.to_owned()];
        true
    }

    /// Marca o fio lido por `me` (K6).
    pub fn mark_read(&mut self, id: ThreadId, me: &str) {
        if let Some(t) = self.thread_mut(id)
            && t.unread_for(me)
        {
            t.read_by.push(me.to_owned());
        }
    }

    /// Edita a mensagem `msg` a chamar também `who` (W23 B14): fica chamado quem o texto mostra.
    pub fn edit_calling(
        &mut self,
        id: ThreadId,
        msg: MessageId,
        me: &str,
        text: &str,
        who: &[Who],
    ) -> bool {
        if !self.edit(id, msg, me, text) {
            return false;
        }
        if let Some(m) = self
            .thread_mut(id)
            .and_then(|t| t.messages.iter_mut().find(|m| m.id == msg))
        {
            m.mentions.extend(who.iter().cloned());
            m.keep_mentions();
        }
        true
    }

    /// Edita a mensagem `msg` — só a do próprio autor (K4). Vazio não edita.
    pub fn edit(&mut self, id: ThreadId, msg: MessageId, me: &str, text: &str) -> bool {
        let text = text.trim();
        let Some(m) = self
            .thread_mut(id)
            .and_then(|t| t.messages.iter_mut().find(|m| m.id == msg))
            .filter(|m| m.author == me && !text.is_empty())
        else {
            return false;
        };
        if m.text != text {
            m.text = text.to_owned();
            m.edited = true;
            m.keep_mentions();
        }
        true
    }

    /// ⭐ W23 B14 — quem se pode chamar com `@` (fora `me`): os autores das mensagens, quem leu os
    /// fios e quem já foi chamado neste quadro, mais `present` (quem está na sessão agora), sem
    /// repetir, por ordem de nome.
    #[must_use]
    pub fn people(&self, me: &str, present: &[String]) -> Vec<String> {
        let mut v: Vec<String> = self
            .threads
            .iter()
            .flat_map(|t| {
                t.messages
                    .iter()
                    .flat_map(|m| {
                        std::iter::once(m.author.clone()).chain(m.mentions.iter().filter_map(
                            |w| match w {
                                Who::Person(n) => Some(n.clone()),
                                Who::Everyone => None,
                            },
                        ))
                    })
                    .chain(t.read_by.iter().cloned())
            })
            .chain(present.iter().cloned())
            .filter(|n| !n.trim().is_empty() && n != me)
            .collect();
        v.sort_by_key(|n| n.to_lowercase());
        v.dedup();
        v
    }

    /// Apaga a mensagem `msg` (só a do próprio autor); apagar a 1.ª apaga o fio inteiro (K4).
    pub fn delete(&mut self, id: ThreadId, msg: MessageId, me: &str) -> bool {
        let Some(at) = self.threads.iter().position(|t| t.id == id) else {
            return false;
        };
        let messages = &self.threads[at].messages;
        let Some(n) = messages.iter().position(|m| m.id == msg) else {
            return false;
        };
        if messages[n].author != me {
            return false;
        }
        if n == 0 {
            self.threads.remove(at);
        } else {
            self.threads[at].messages.remove(n);
        }
        true
    }

    /// Resolve ou reabre (K5).
    pub fn set_resolved(&mut self, id: ThreadId, resolved: bool) -> bool {
        self.thread_mut(id).map(|t| t.resolved = resolved).is_some()
    }

    /// ⭐ Aplica um passo DESFAZÍVEL da equipa e devolve o inverso (`None` = o fio não existe ou nada
    /// mudou — não é passo). O histórico do quadro guarda-o ao lado dos lotes do documento
    /// ([`crate::History::record_team`]): o `Ctrl+Z` desfaz resolver e mover pela ordem em que se fez.
    pub fn apply(&mut self, op: TeamOp) -> Option<TeamOp> {
        match op {
            TeamOp::Resolve(id, to) => {
                let t = self.thread_mut(id)?;
                (t.resolved != to)
                    .then(|| TeamOp::Resolve(id, std::mem::replace(&mut t.resolved, to)))
            }
            TeamOp::Pin(id, to) => {
                let t = self.thread_mut(id)?;
                (t.pin != to).then(|| TeamOp::Pin(id, std::mem::replace(&mut t.pin, to)))
            }
        }
    }

    /// Esconde (ou volta a mostrar) um frame na apresentação (P1).
    pub fn set_slide_hidden(&mut self, frame: ElementId, hidden: bool) {
        if hidden {
            self.hidden_slides.insert(frame);
        } else {
            self.hidden_slides.remove(&frame);
        }
    }
}

/// O que se guarda de um quadro numa versão: o documento, o rascunho e «não apresentar» (R4).
#[derive(Serialize)]
struct BoardRef<'a> {
    id: crate::BoardId,
    name: &'a str,
    camera: crate::Camera,
    doc: &'a BoardDoc,
    sketch: bool,
    team: &'a Team,
    /// Sempre vazio: os bytes das imagens ficam no quadro ([`Version::files`]).
    files: &'a crate::Files,
}

/// Os bytes de um `BoardSet` de um quadro, sem o clonar (o postcard é posicional: o MESMO desenho
/// de campos que [`crate::BoardSet`] e [`Board`] — gate `a_version_is_the_bytes_of_a_one_board_set`).
#[derive(Serialize)]
struct SetRef<'a> {
    version: u32,
    /// Uma FATIA (sequência com comprimento, como o `Vec`): um `[T; 1]` é um tuplo sem ele.
    boards: &'a [BoardRef<'a>],
    next_id: u64,
}

impl Board {
    /// Entra (`Some(semente)`) ou sai (`None`) de uma sessão ao vivo: o documento e a equipa, com
    /// sementes distintas derivadas da mesma (os dois sorteios nunca andam juntos).
    pub fn set_session(&mut self, seed: Option<u64>) {
        self.doc.set_session(seed);
        self.team.set_session(seed.map(crate::session::splitmix64));
    }

    /// Guarda uma versão (R1) com `name` e o instante `at`. ⚠️ Comprime: 14,5 ms a 100 mil elementos.
    pub fn save_version(&mut self, name: String, at: i64) {
        let bytes = self.version_bytes();
        let id = VersionId(self.team.mint());
        let files = self.doc.image_files();
        self.team.versions.push(Version {
            id,
            name,
            at,
            bytes,
            files,
        });
    }

    /// Os bytes de uma versão deste quadro (o que [`Version::read`] lê de volta).
    #[must_use]
    pub fn version_bytes(&self) -> Vec<u8> {
        // R4/P1 — a versão leva o documento, o rascunho e «não apresentar» (é do documento); nunca os
        // comentários nem as outras versões.
        let kept = Team {
            hidden_slides: self.team.hidden_slides.clone(),
            ..Team::default()
        };
        let one = [BoardRef {
            id: self.id,
            name: &self.name,
            camera: crate::Camera::default(),
            doc: &self.doc,
            sketch: self.sketch,
            team: &kept,
            files: &crate::Files::new(),
        }];
        let set = SetRef {
            version: crate::FORMAT_VERSION,
            boards: &one,
            next_id: self.id.0,
        };
        let raw = postcard::to_allocvec(&set).unwrap_or_default();
        let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(1));
        z.write_all(&raw)
            .and_then(|()| z.finish())
            .unwrap_or_default()
    }
}

impl Version {
    /// O quadro guardado (o documento, o rascunho e os frames fora da apresentação), pelo leitor de
    /// TODOS os formatos.
    ///
    /// # Errors
    /// Bytes corrompidos, ou de um formato que este build não lê.
    pub fn read(&self) -> Result<Board, String> {
        let mut z = flate2::write::DeflateDecoder::new(Vec::new());
        let raw = z
            .write_all(&self.bytes)
            .and_then(|()| z.finish())
            .map_err(|e| e.to_string())?;
        let b = crate::BoardSet::from_bytes(&raw)?
            .into_boards()
            .into_iter()
            .next()
            .ok_or("empty version")?;
        Ok(b)
    }
}

#[cfg(test)]
#[path = "team_tests.rs"]
mod tests;
