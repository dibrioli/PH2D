//! **A equipa assíncrona** (W8, `docs/MiroClone/06_equipa_e_saida.md` §2): os comentários em fio, as
//! versões nomeadas e os frames que não se apresentam — DOCUMENTO do quadro, mas fora do desfazer e das
//! cópias de elementos (K9): a conversa é SOBRE o quadro.

use std::collections::BTreeSet;
use std::io::Write as _;

use serde::{Deserialize, Serialize};

use crate::{Board, BoardDoc, ElementId, ElementKind};

/// Identidade de um fio dentro do seu quadro. Nunca reusada.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ThreadId(pub u64);

/// Onde um fio está preso (K2).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Pin {
    /// Num objecto, em fracção `[u, v]` da caixa dele ANTES de rodar: anda, roda e escala com ele.
    Element { id: ElementId, at: [f64; 2] },
    /// Num ponto do mundo do quadro.
    Point([f64; 2]),
}

/// Uma mensagem de um fio (K4): texto simples, o nome de quem a escreveu e o instante (segundos
/// UNIX — a vista mostra «há quanto tempo», que não pede fuso).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub author: String,
    pub text: String,
    pub at: i64,
    pub edited: bool,
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
    pub name: String,
    pub at: i64,
    pub bytes: Vec<u8>,
}

/// O que a equipa guarda num quadro.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Team {
    pub threads: Vec<Thread>,
    pub versions: Vec<Version>,
    /// Os frames marcados «Hide from presentation» (P1).
    pub hidden_slides: BTreeSet<ElementId>,
    next_thread: u64,
}

impl Thread {
    /// Este fio está por ler para `me` (K6): não é meu, e eu ainda não o abri.
    #[must_use]
    pub fn unread_for(&self, me: &str) -> bool {
        !self.read_by.iter().any(|r| r == me)
    }

    /// Onde a bolha aponta, no mundo; `None` = o objecto apagado ou escondido (K3: o fio some com ele
    /// e volta com o desfazer).
    #[must_use]
    pub fn anchor(&self, doc: &BoardDoc) -> Option<[f64; 2]> {
        match self.pin {
            Pin::Point(p) => Some(p),
            Pin::Element { id, at } => {
                let el = doc.get(id).filter(|e| !e.is_hidden())?;
                Some(el.rotate([el.x + at[0] * el.w, el.y + at[1] * el.h]))
            }
        }
    }
}

impl Pin {
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
    /// Abre um fio novo com a 1.ª mensagem; quem o escreve já o leu. Texto vazio (só espaços) não abre.
    pub fn open(&mut self, pin: Pin, author: &str, text: &str, at: i64) -> Option<ThreadId> {
        let text = text.trim();
        if text.is_empty() {
            return None;
        }
        self.next_thread += 1;
        let id = ThreadId(self.next_thread);
        self.threads.push(Thread {
            id,
            pin,
            messages: vec![Message {
                author: author.to_owned(),
                text: text.to_owned(),
                at,
                edited: false,
            }],
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
        let text = text.trim();
        let Some(t) = self.thread_mut(id).filter(|_| !text.is_empty()) else {
            return false;
        };
        t.messages.push(Message {
            author: author.to_owned(),
            text: text.to_owned(),
            at,
            edited: false,
        });
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

    /// Edita a mensagem `n` — só a do próprio autor (K4). Vazio não edita.
    pub fn edit(&mut self, id: ThreadId, n: usize, me: &str, text: &str) -> bool {
        let text = text.trim();
        let Some(m) = self
            .thread_mut(id)
            .and_then(|t| t.messages.get_mut(n))
            .filter(|m| m.author == me && !text.is_empty())
        else {
            return false;
        };
        if m.text != text {
            m.text = text.to_owned();
            m.edited = true;
        }
        true
    }

    /// Apaga a mensagem `n` (só a do próprio autor); apagar a 1.ª apaga o fio inteiro (K4).
    pub fn delete(&mut self, id: ThreadId, n: usize, me: &str) -> bool {
        let Some(at) = self.threads.iter().position(|t| t.id == id) else {
            return false;
        };
        if self.threads[at].messages.get(n).is_none_or(|m| m.author != me) {
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

    /// Esconde (ou volta a mostrar) um frame na apresentação (P1).
    pub fn set_slide_hidden(&mut self, frame: ElementId, hidden: bool) {
        if hidden {
            self.hidden_slides.insert(frame);
        } else {
            self.hidden_slides.remove(&frame);
        }
    }
}

/// O que se guarda de um quadro numa versão: o documento e o rascunho (R4) — nunca a equipa.
#[derive(Serialize)]
struct BoardRef<'a> {
    id: crate::BoardId,
    name: &'a str,
    camera: crate::Camera,
    doc: &'a BoardDoc,
    sketch: bool,
    team: &'a Team,
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
    /// Guarda uma versão (R1) com `name` e o instante `at`. ⚠️ Comprime: 14,5 ms a 100 mil elementos.
    pub fn save_version(&mut self, name: String, at: i64) {
        let empty = Team::default();
        let one = [BoardRef {
            id: self.id,
            name: &self.name,
            camera: crate::Camera::default(),
            doc: &self.doc,
            sketch: self.sketch,
            team: &empty,
        }];
        let set = SetRef {
            version: crate::FORMAT_VERSION,
            boards: &one,
            next_id: self.id.0,
        };
        let raw = postcard::to_allocvec(&set).unwrap_or_default();
        let mut z = flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::new(1));
        let bytes = z.write_all(&raw).and_then(|()| z.finish()).unwrap_or_default();
        self.team.versions.push(Version { name, at, bytes });
    }
}

impl Version {
    /// O documento e o rascunho guardados (pelo leitor de TODOS os formatos).
    ///
    /// # Errors
    /// Bytes corrompidos, ou de um formato que este build não lê.
    pub fn read(&self) -> Result<(BoardDoc, bool), String> {
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
        Ok((b.doc, b.sketch))
    }
}

#[cfg(test)]
#[path = "team_tests.rs"]
mod tests;
