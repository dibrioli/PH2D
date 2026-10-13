//! Os PASSOS da trilha do Motion e a sua carga ([`ph2d_macro::rec`]: o tipo na etiqueta `1`, os campos
//! de `2` em diante). O NÚMERO de cada tipo — de passo, de intenção, de edição — é para sempre: um
//! leitor antigo devolve o `Desconhecido` daquele nível, e o tocador recusa-o em voz alta.
//!
//! Um nó guarda-se pela ORDEM DE CRIAÇÃO ([`Alvo`]), nunca pelo id cru: a macro tem de sobreviver a
//! um documento que já tenha nós.

use super::Destino;
pub use super::passo_grafo::Intencao;
use ph2d_macro::CodecError;
use ph2d_macro::rec::{self, Record};
use ph2d_nodegraph::graph::{Edge, NodeId};

/// A carga codificada de um passo (o registo etiquetado que vai na `Event::payload`).
pub type Carga = Vec<u8>; // COLOR-RAW-OK: um registo etiquetado do formato da macro, não pixels

/// Um alvo do grafo como a gravação o guarda.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alvo {
    /// Um nó que já existia ao armar: o id dele.
    Existente(u32),
    /// O `k`-ésimo id cunhado depois de armar (o id é `base + k`).
    Criado(u32),
    /// Um grupo (o cartão de um subgrafo), pelo id do grupo.
    Grupo(u32),
}

impl Alvo {
    /// Um id de NÓ (nunca um cartão), relativo ao `next_id` ao armar.
    #[must_use]
    pub fn de_no(id: u32, base: u32) -> Self {
        if id >= base {
            Alvo::Criado(id - base)
        } else {
            Alvo::Existente(id)
        }
    }

    /// O id do nó no palco; `None` para um grupo.
    #[must_use]
    pub fn no(self, base: u32) -> Option<u32> {
        match self {
            Alvo::Existente(id) => Some(id),
            Alvo::Criado(k) => base.checked_add(k),
            Alvo::Grupo(_) => None,
        }
    }

    pub(super) fn varint(self) -> u64 {
        match self {
            Alvo::Existente(n) => u64::from(n) << 2,
            Alvo::Criado(n) => (u64::from(n) << 2) | 1,
            Alvo::Grupo(n) => (u64::from(n) << 2) | 2,
        }
    }

    pub(super) fn de_varint(v: u64) -> Result<Self, CodecError> {
        let n = u32::try_from(v >> 2).map_err(|_| CodecError::BadVarint)?;
        match v & 3 {
            0 => Ok(Alvo::Existente(n)),
            1 => Ok(Alvo::Criado(n)),
            2 => Ok(Alvo::Grupo(n)),
            _ => Err(CodecError::BadVarint),
        }
    }
}

/// O que o botão «Use selected path» viu no mundo — a entrada que o braço lê.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Escolha {
    Nada,
    NaoEDesenho,
    SemNome,
    SemArco,
    Nome(String),
}

/// O que um `Rename` nomeia.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Nomeado {
    No(Alvo),
    Grupo(u32),
    Fundo(u32),
}

/// Uma edição de param já resolvida (o caminho de um `PickFile` já escolhido).
#[derive(Clone, Debug, PartialEq)]
pub enum IntencaoDeParam {
    SetParam {
        node: Alvo,
        param: String,
        value: f64,
    },
    SetTextParam {
        node: Alvo,
        param: String,
        value: String,
    },
    ResetParam {
        node: Alvo,
        param: String,
    },
    Desconhecida(u64),
}

/// Uma escrita num param feita FORA das intenções (um gizmo, o conta-gotas da cor).
#[derive(Clone, Debug, PartialEq)]
pub enum Escrita {
    Valor {
        node: Alvo,
        param: String,
        valor: Option<f32>,
    },
    Texto {
        node: Alvo,
        param: String,
        valor: Option<String>,
    },
}

/// O que tocou a bomba do cozimento FORA de um lote gravado.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Toque {
    /// Houve um `mark_dirty` (uma edição: o anel do scrub foi deitado fora).
    pub editou: bool,
    /// A bomba ficou suja sem edição (`mark_dirty_keeping_ring`).
    pub sujou: bool,
}

impl Toque {
    #[must_use]
    pub fn vazio(self) -> bool {
        !self.editou && !self.sujou
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Passo {
    /// Um lote do dreno do grafo (um quadro), no nível e com a ajuda dos nós de então.
    Grafo {
        nivel: Option<u32>,
        ajuda: bool,
        intencoes: Vec<Intencao>,
    },
    /// Um lote do dreno de params, no destino que a selecção decidiu.
    Params {
        destino: Destino,
        intencoes: Vec<IntencaoDeParam>,
    },
    /// Params mudados por fora das intenções (o resto do documento igual).
    Escritas(Vec<Escrita>),
    /// O documento inteiro mudou por fora das intenções (desfazer, refazer, um objeto copiado): o
    /// texto canónico, o `next_id` e a ordem viva das arestas, que o texto não guarda.
    Documento {
        texto: String,
        proximo_id: u32,
        arestas: Vec<Edge>,
    },
    /// A bomba foi tocada por fora de um lote.
    Toque(Toque),
    /// O cozimento de um quadro: o relógio do playhead, e se a bomba chegou suja.
    Quadro {
        tempo: f64,
        a_tocar: bool,
        dt: f64,
        sujo: bool,
    },
    /// Uma porta que a macro não reproduz foi usada a gravar (o porquê).
    Recusada(String),
    /// Um tipo que este leitor não conhece (gravado por uma versão mais nova).
    Desconhecido(u64),
}

// — os números, para sempre —
const P_GRAFO: u64 = 1;
const P_PARAMS: u64 = 2;
const P_ESCRITAS: u64 = 3;
const P_DOCUMENTO: u64 = 4;
const P_TOQUE: u64 = 5;
const P_QUADRO: u64 = 6;
const P_RECUSADA: u64 = 7;

fn destino_varint(d: Destino) -> u64 {
    match d {
        Destino::No => 0,
        Destino::Fundo(id) => (u64::from(id) << 2) | 1,
        Destino::Cartao(id) => (u64::from(id) << 2) | 2,
    }
}

fn destino_de(v: u64) -> Result<Destino, CodecError> {
    let n = u32::try_from(v >> 2).map_err(|_| CodecError::BadVarint)?;
    match v & 3 {
        0 => Ok(Destino::No),
        1 => Ok(Destino::Fundo(n)),
        2 => Ok(Destino::Cartao(n)),
        _ => Err(CodecError::BadVarint),
    }
}

impl IntencaoDeParam {
    #[must_use]
    pub fn codifica(&self) -> Carga {
        let mut r = Record::new();
        match self {
            IntencaoDeParam::SetParam { node, param, value } => {
                r.varint(1, 1)
                    .varint(2, node.varint())
                    .str(3, param)
                    .f64(4, *value);
            }
            IntencaoDeParam::SetTextParam { node, param, value } => {
                r.varint(1, 2)
                    .varint(2, node.varint())
                    .str(3, param)
                    .str(4, value);
            }
            IntencaoDeParam::ResetParam { node, param } => {
                r.varint(1, 3).varint(2, node.varint()).str(3, param);
            }
            IntencaoDeParam::Desconhecida(n) => {
                r.varint(1, *n);
            }
        }
        r.finish()
    }

    /// # Errors
    /// [`CodecError`].
    pub fn descodifica(bytes: &[u8]) -> Result<Self, CodecError> {
        let c = Campos::de(bytes)?;
        Ok(match c.varint(1)? {
            1 => IntencaoDeParam::SetParam {
                node: c.alvo(2)?,
                param: c.string(3)?,
                value: rec::as_f64(c.precisa(4)?)?,
            },
            2 => IntencaoDeParam::SetTextParam {
                node: c.alvo(2)?,
                param: c.string(3)?,
                value: c.string(4)?,
            },
            3 => IntencaoDeParam::ResetParam {
                node: c.alvo(2)?,
                param: c.string(3)?,
            },
            n => IntencaoDeParam::Desconhecida(n),
        })
    }
}

impl Escrita {
    fn codifica(&self) -> Carga {
        let mut r = Record::new();
        match self {
            Escrita::Valor { node, param, valor } => {
                r.varint(1, 1).varint(2, node.varint()).str(3, param);
                if let Some(v) = valor {
                    r.f32(4, *v);
                }
            }
            Escrita::Texto { node, param, valor } => {
                r.varint(1, 2).varint(2, node.varint()).str(3, param);
                if let Some(v) = valor {
                    r.str(4, v);
                }
            }
        }
        r.finish()
    }

    fn descodifica(bytes: &[u8]) -> Result<Option<Self>, CodecError> {
        let c = Campos::de(bytes)?;
        let node = c.alvo(2)?;
        let param = c.string(3)?;
        Ok(match c.varint(1)? {
            1 => Some(Escrita::Valor {
                node,
                param,
                valor: c.um(4).map(rec::as_f32).transpose()?,
            }),
            2 => Some(Escrita::Texto {
                node,
                param,
                valor: c
                    .um(4)
                    .map(|b| rec::as_str(b).map(str::to_owned))
                    .transpose()?,
            }),
            _ => None,
        })
    }
}

impl Passo {
    #[must_use]
    pub fn codifica(&self) -> Carga {
        let mut r = Record::new();
        match self {
            Passo::Grafo {
                nivel,
                ajuda,
                intencoes,
            } => {
                r.varint(1, P_GRAFO).bool(3, *ajuda);
                if let Some(n) = nivel {
                    r.varint(2, u64::from(*n));
                }
                for i in intencoes {
                    r.bytes(4, &i.codifica());
                }
            }
            Passo::Params { destino, intencoes } => {
                r.varint(1, P_PARAMS).varint(2, destino_varint(*destino));
                for i in intencoes {
                    r.bytes(3, &i.codifica());
                }
            }
            Passo::Escritas(v) => {
                r.varint(1, P_ESCRITAS);
                for e in v {
                    r.bytes(2, &e.codifica());
                }
            }
            Passo::Documento {
                texto,
                proximo_id,
                arestas,
            } => {
                r.varint(1, P_DOCUMENTO)
                    .str(2, texto)
                    .varint(3, u64::from(*proximo_id));
                for e in arestas {
                    let mut a = Record::new();
                    a.varint(1, u64::from(e.from.0.0))
                        .varint(2, u64::from(e.from.1))
                        .varint(3, u64::from(e.to.0.0))
                        .varint(4, u64::from(e.to.1))
                        .bool(5, e.delayed);
                    r.bytes(4, &a.finish());
                }
            }
            Passo::Toque(t) => {
                r.varint(1, P_TOQUE).bool(2, t.editou).bool(3, t.sujou);
            }
            Passo::Quadro {
                tempo,
                a_tocar,
                dt,
                sujo,
            } => {
                r.varint(1, P_QUADRO)
                    .f64(2, *tempo)
                    .bool(3, *a_tocar)
                    .f64(4, *dt)
                    .bool(5, *sujo);
            }
            Passo::Recusada(porta) => {
                r.varint(1, P_RECUSADA).str(2, porta);
            }
            Passo::Desconhecido(n) => {
                r.varint(1, *n);
            }
        }
        r.finish()
    }

    /// # Errors
    /// [`CodecError`].
    pub fn descodifica(bytes: &[u8]) -> Result<Self, CodecError> {
        let c = Campos::de(bytes)?;
        Ok(match c.varint(1)? {
            P_GRAFO => Passo::Grafo {
                nivel: c.um(2).map(|_| c.u32(2)).transpose()?,
                ajuda: c.bool(3)?,
                intencoes: c
                    .todos(4)
                    .map(Intencao::descodifica)
                    .collect::<Result<_, _>>()?,
            },
            P_PARAMS => Passo::Params {
                destino: destino_de(c.varint(2)?)?,
                intencoes: c
                    .todos(3)
                    .map(IntencaoDeParam::descodifica)
                    .collect::<Result<_, _>>()?,
            },
            P_ESCRITAS => {
                let mut v = Vec::new();
                for b in c.todos(2) {
                    match Escrita::descodifica(b)? {
                        Some(e) => v.push(e),
                        // Uma espécie de escrita mais nova: saltá-la mudaria o documento.
                        None => return Ok(Passo::Desconhecido(P_ESCRITAS)),
                    }
                }
                Passo::Escritas(v)
            }
            P_DOCUMENTO => Passo::Documento {
                texto: c.string(2)?,
                proximo_id: c.u32(3)?,
                arestas: c
                    .todos(4)
                    .map(|b| {
                        let a = Campos::de(b)?;
                        Ok(Edge {
                            from: (NodeId(a.u32(1)?), a.u16(2)?),
                            to: (NodeId(a.u32(3)?), a.u16(4)?),
                            delayed: a.bool(5)?,
                        })
                    })
                    .collect::<Result<_, CodecError>>()?,
            },
            P_TOQUE => Passo::Toque(Toque {
                editou: c.bool(2)?,
                sujou: c.bool(3)?,
            }),
            P_QUADRO => Passo::Quadro {
                tempo: rec::as_f64(c.precisa(2)?)?,
                a_tocar: c.bool(3)?,
                dt: rec::as_f64(c.precisa(4)?)?,
                sujo: c.bool(5)?,
            },
            P_RECUSADA => Passo::Recusada(c.string(2)?),
            n => Passo::Desconhecido(n),
        })
    }
}

/// Os campos de um registo, com a lei do formato: campo único — vence a última; lista — todos, pela
/// ordem do ficheiro.
pub(super) struct Campos<'a>(Vec<(u32, &'a [u8])>);

impl<'a> Campos<'a> {
    pub(super) fn de(bytes: &'a [u8]) -> Result<Self, CodecError> {
        rec::fields(bytes).map(Campos)
    }

    pub(super) fn um(&self, tag: u32) -> Option<&'a [u8]> {
        self.0
            .iter()
            .rev()
            .find(|(t, _)| *t == tag)
            .map(|(_, b)| *b)
    }

    pub(super) fn todos(&self, tag: u32) -> impl Iterator<Item = &'a [u8]> + '_ {
        self.0
            .iter()
            .filter(move |(t, _)| *t == tag)
            .map(|(_, b)| *b)
    }

    pub(super) fn precisa(&self, tag: u32) -> Result<&'a [u8], CodecError> {
        self.um(tag).ok_or(CodecError::MissingField {
            record: "motion",
            tag,
        })
    }

    pub(super) fn varint(&self, tag: u32) -> Result<u64, CodecError> {
        rec::as_varint(self.precisa(tag)?)
    }

    pub(super) fn u32(&self, tag: u32) -> Result<u32, CodecError> {
        u32::try_from(self.varint(tag)?).map_err(|_| CodecError::BadVarint)
    }

    pub(super) fn u16(&self, tag: u32) -> Result<u16, CodecError> {
        u16_de(self.varint(tag)?)
    }

    pub(super) fn bool(&self, tag: u32) -> Result<bool, CodecError> {
        rec::as_bool(self.precisa(tag)?)
    }

    pub(super) fn f32(&self, tag: u32) -> Result<f32, CodecError> {
        rec::as_f32(self.precisa(tag)?)
    }

    pub(super) fn string(&self, tag: u32) -> Result<String, CodecError> {
        rec::as_str(self.precisa(tag)?).map(str::to_owned)
    }

    pub(super) fn alvo(&self, tag: u32) -> Result<Alvo, CodecError> {
        Alvo::de_varint(self.varint(tag)?)
    }

    pub(super) fn alvos(&self, tag: u32) -> Result<Vec<Alvo>, CodecError> {
        self.todos(tag)
            .map(|b| Alvo::de_varint(rec::as_varint(b)?))
            .collect()
    }
}

pub(super) fn u16_de(v: u64) -> Result<u16, CodecError> {
    u16::try_from(v).map_err(|_| CodecError::BadVarint)
}
