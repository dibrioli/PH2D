//! Os PASSOS da trilha do Motion e a sua carga ([`ph2d_macro::rec`]: o tipo na etiqueta `1`, os campos
//! de `2` em diante). O NÚMERO de cada tipo — de passo, de intenção, de edição — é para sempre: um
//! leitor antigo devolve o `Desconhecido` daquele nível, e o tocador recusa-o em voz alta.
//!
//! Um nó guarda-se pela ORDEM DE CRIAÇÃO ([`Alvo`]), nunca pelo id cru: a macro tem de sobreviver a
//! um documento que já tenha nós.

use super::Destino;
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

    fn varint(self) -> u64 {
        match self {
            Alvo::Existente(n) => u64::from(n) << 2,
            Alvo::Criado(n) => (u64::from(n) << 2) | 1,
            Alvo::Grupo(n) => (u64::from(n) << 2) | 2,
        }
    }

    fn de_varint(v: u64) -> Result<Self, CodecError> {
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

/// Uma intenção do grafo, já com os ids de vista decodificados. Os nomes (tipo, param) viajam como
/// texto e voltam ao `&'static` pelo registo do palco.
#[derive(Clone, Debug, PartialEq)]
pub enum Intencao {
    BeginDrag,
    EndDrag,
    MoveNodes {
        nodes: Vec<Alvo>,
        dx: f32,
        dy: f32,
    },
    DriveParam {
        from: Alvo,
        from_port: u16,
        to: Alvo,
        param: String,
    },
    PickSelection {
        node: Alvo,
        param: String,
        escolha: Escolha,
    },
    Connect {
        from: Alvo,
        from_port: u16,
        to: Alvo,
        to_port: u16,
    },
    Disconnect {
        to: Alvo,
        to_port: u16,
    },
    AddNode {
        tipo: String,
        x: f32,
        y: f32,
    },
    DeleteSelection(Vec<Alvo>),
    AddBackdrop {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    MoveBackdrop {
        id: u32,
        dx: f32,
        dy: f32,
    },
    ResizeBackdrop {
        id: u32,
        left: bool,
        dx: f32,
        dy: f32,
    },
    DeleteBackdrop(u32),
    Rename {
        alvo: Nomeado,
        nome: String,
    },
    SetBackdropColor {
        id: u32,
        cor: u8,
    },
    DuplicateSelection(Vec<Alvo>),
    SmartConnect {
        from: Alvo,
        from_port: u16,
        tipo: String,
        x: f32,
        y: f32,
    },
    SwapInChain {
        a: Alvo,
        b: Alvo,
        back_dx: f32,
        back_dy: f32,
    },
    SpliceExistingIntoWire {
        node: Alvo,
        to: Alvo,
        to_port: u16,
    },
    SmartConnectBack {
        to: Alvo,
        to_port: u16,
        tipo: String,
        x: f32,
        y: f32,
    },
    CutWires(Vec<(Alvo, u16)>),
    MoveWireEnd {
        from: Alvo,
        from_port: u16,
        old_to: Alvo,
        old_to_port: u16,
        new_to: Option<(Alvo, u16)>,
    },
    GroupSelection(Vec<Alvo>),
    Ungroup(u32),
    EnterSubgraph(u32),
    GoToLevel(Option<u32>),
    SpliceReroute {
        to: Alvo,
        to_port: u16,
        x: f32,
        y: f32,
    },
    SpliceNode {
        to: Alvo,
        to_port: u16,
        tipo: String,
        x: f32,
        y: f32,
    },
    ArrangeLayout,
    SetBypass {
        nodes: Vec<Alvo>,
        on: bool,
    },
    CopySelection(Vec<Alvo>),
    Paste,
    FixInert(Alvo),
    SetNodeHelp(bool),
    /// Um tipo que este leitor não conhece.
    Desconhecida(u64),
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

const ESC_NADA: u64 = 0;
const ESC_NAO_E_DESENHO: u64 = 1;
const ESC_SEM_NOME: u64 = 2;
const ESC_SEM_ARCO: u64 = 3;
const ESC_NOME: u64 = 4;

fn alvos(r: &mut Record, tag: u32, v: &[Alvo]) {
    for a in v {
        r.varint(tag, a.varint());
    }
}

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

impl Intencao {
    #[must_use]
    pub fn codifica(&self) -> Carga {
        let mut r = Record::new();
        let k = |r: &mut Record, n: u64| {
            r.varint(1, n);
        };
        match self {
            Intencao::BeginDrag => k(&mut r, 1),
            Intencao::EndDrag => k(&mut r, 2),
            Intencao::MoveNodes { nodes, dx, dy } => {
                k(&mut r, 3);
                alvos(&mut r, 2, nodes);
                r.f32(3, *dx).f32(4, *dy);
            }
            Intencao::DriveParam {
                from,
                from_port,
                to,
                param,
            } => {
                k(&mut r, 4);
                r.varint(2, from.varint())
                    .varint(3, u64::from(*from_port))
                    .varint(4, to.varint())
                    .str(5, param);
            }
            Intencao::PickSelection {
                node,
                param,
                escolha,
            } => {
                k(&mut r, 5);
                r.varint(2, node.varint()).str(3, param);
                let (e, nome) = match escolha {
                    Escolha::Nada => (ESC_NADA, None),
                    Escolha::NaoEDesenho => (ESC_NAO_E_DESENHO, None),
                    Escolha::SemNome => (ESC_SEM_NOME, None),
                    Escolha::SemArco => (ESC_SEM_ARCO, None),
                    Escolha::Nome(n) => (ESC_NOME, Some(n)),
                };
                r.varint(4, e);
                if let Some(n) = nome {
                    r.str(5, n);
                }
            }
            Intencao::Connect {
                from,
                from_port,
                to,
                to_port,
            } => {
                k(&mut r, 6);
                r.varint(2, from.varint())
                    .varint(3, u64::from(*from_port))
                    .varint(4, to.varint())
                    .varint(5, u64::from(*to_port));
            }
            Intencao::Disconnect { to, to_port } => {
                k(&mut r, 7);
                r.varint(2, to.varint()).varint(3, u64::from(*to_port));
            }
            Intencao::AddNode { tipo, x, y } => {
                k(&mut r, 8);
                r.str(2, tipo).f32(3, *x).f32(4, *y);
            }
            Intencao::DeleteSelection(v) => {
                k(&mut r, 9);
                alvos(&mut r, 2, v);
            }
            Intencao::AddBackdrop { x, y, w, h } => {
                k(&mut r, 10);
                r.f32(2, *x).f32(3, *y).f32(4, *w).f32(5, *h);
            }
            Intencao::MoveBackdrop { id, dx, dy } => {
                k(&mut r, 11);
                r.varint(2, u64::from(*id)).f32(3, *dx).f32(4, *dy);
            }
            Intencao::ResizeBackdrop { id, left, dx, dy } => {
                k(&mut r, 12);
                r.varint(2, u64::from(*id))
                    .bool(3, *left)
                    .f32(4, *dx)
                    .f32(5, *dy);
            }
            Intencao::DeleteBackdrop(id) => {
                k(&mut r, 13);
                r.varint(2, u64::from(*id));
            }
            Intencao::Rename { alvo, nome } => {
                k(&mut r, 14);
                let (q, v) = match alvo {
                    Nomeado::No(a) => (0, a.varint()),
                    Nomeado::Grupo(id) => (1, u64::from(*id)),
                    Nomeado::Fundo(id) => (2, u64::from(*id)),
                };
                r.varint(2, q).varint(3, v).str(4, nome);
            }
            Intencao::SetBackdropColor { id, cor } => {
                k(&mut r, 15);
                r.varint(2, u64::from(*id)).varint(3, u64::from(*cor));
            }
            Intencao::DuplicateSelection(v) => {
                k(&mut r, 16);
                alvos(&mut r, 2, v);
            }
            Intencao::SmartConnect {
                from,
                from_port,
                tipo,
                x,
                y,
            } => {
                k(&mut r, 17);
                r.varint(2, from.varint())
                    .varint(3, u64::from(*from_port))
                    .str(4, tipo)
                    .f32(5, *x)
                    .f32(6, *y);
            }
            Intencao::SwapInChain {
                a,
                b,
                back_dx,
                back_dy,
            } => {
                k(&mut r, 18);
                r.varint(2, a.varint())
                    .varint(3, b.varint())
                    .f32(4, *back_dx)
                    .f32(5, *back_dy);
            }
            Intencao::SpliceExistingIntoWire { node, to, to_port } => {
                k(&mut r, 19);
                r.varint(2, node.varint())
                    .varint(3, to.varint())
                    .varint(4, u64::from(*to_port));
            }
            Intencao::SmartConnectBack {
                to,
                to_port,
                tipo,
                x,
                y,
            } => {
                k(&mut r, 20);
                r.varint(2, to.varint())
                    .varint(3, u64::from(*to_port))
                    .str(4, tipo)
                    .f32(5, *x)
                    .f32(6, *y);
            }
            Intencao::CutWires(v) => {
                k(&mut r, 21);
                for (a, p) in v {
                    r.varint(2, a.varint()).varint(3, u64::from(*p));
                }
            }
            Intencao::MoveWireEnd {
                from,
                from_port,
                old_to,
                old_to_port,
                new_to,
            } => {
                k(&mut r, 22);
                r.varint(2, from.varint())
                    .varint(3, u64::from(*from_port))
                    .varint(4, old_to.varint())
                    .varint(5, u64::from(*old_to_port));
                if let Some((a, p)) = new_to {
                    r.varint(6, a.varint()).varint(7, u64::from(*p));
                }
            }
            Intencao::GroupSelection(v) => {
                k(&mut r, 23);
                alvos(&mut r, 2, v);
            }
            Intencao::Ungroup(id) => {
                k(&mut r, 24);
                r.varint(2, u64::from(*id));
            }
            Intencao::EnterSubgraph(id) => {
                k(&mut r, 25);
                r.varint(2, u64::from(*id));
            }
            Intencao::GoToLevel(l) => {
                k(&mut r, 26);
                if let Some(l) = l {
                    r.varint(2, u64::from(*l));
                }
            }
            Intencao::SpliceReroute { to, to_port, x, y } => {
                k(&mut r, 27);
                r.varint(2, to.varint())
                    .varint(3, u64::from(*to_port))
                    .f32(4, *x)
                    .f32(5, *y);
            }
            Intencao::SpliceNode {
                to,
                to_port,
                tipo,
                x,
                y,
            } => {
                k(&mut r, 28);
                r.varint(2, to.varint())
                    .varint(3, u64::from(*to_port))
                    .str(4, tipo)
                    .f32(5, *x)
                    .f32(6, *y);
            }
            Intencao::ArrangeLayout => k(&mut r, 29),
            Intencao::SetBypass { nodes, on } => {
                k(&mut r, 30);
                alvos(&mut r, 2, nodes);
                r.bool(3, *on);
            }
            Intencao::CopySelection(v) => {
                k(&mut r, 31);
                alvos(&mut r, 2, v);
            }
            Intencao::Paste => k(&mut r, 32),
            Intencao::FixInert(a) => {
                k(&mut r, 33);
                r.varint(2, a.varint());
            }
            Intencao::SetNodeHelp(on) => {
                k(&mut r, 34);
                r.bool(2, *on);
            }
            Intencao::Desconhecida(n) => k(&mut r, *n),
        }
        r.finish()
    }

    /// # Errors
    /// [`CodecError`] — um campo que falta ou não se lê.
    pub fn descodifica(bytes: &[u8]) -> Result<Self, CodecError> {
        let c = Campos::de(bytes)?;
        let a = |tag| c.alvo(tag);
        Ok(match c.varint(1)? {
            1 => Intencao::BeginDrag,
            2 => Intencao::EndDrag,
            3 => Intencao::MoveNodes {
                nodes: c.alvos(2)?,
                dx: c.f32(3)?,
                dy: c.f32(4)?,
            },
            4 => Intencao::DriveParam {
                from: a(2)?,
                from_port: c.u16(3)?,
                to: a(4)?,
                param: c.string(5)?,
            },
            5 => Intencao::PickSelection {
                node: a(2)?,
                param: c.string(3)?,
                escolha: match c.varint(4)? {
                    ESC_NAO_E_DESENHO => Escolha::NaoEDesenho,
                    ESC_SEM_NOME => Escolha::SemNome,
                    ESC_SEM_ARCO => Escolha::SemArco,
                    ESC_NOME => Escolha::Nome(c.string(5)?),
                    _ => Escolha::Nada,
                },
            },
            6 => Intencao::Connect {
                from: a(2)?,
                from_port: c.u16(3)?,
                to: a(4)?,
                to_port: c.u16(5)?,
            },
            7 => Intencao::Disconnect {
                to: a(2)?,
                to_port: c.u16(3)?,
            },
            8 => Intencao::AddNode {
                tipo: c.string(2)?,
                x: c.f32(3)?,
                y: c.f32(4)?,
            },
            9 => Intencao::DeleteSelection(c.alvos(2)?),
            10 => Intencao::AddBackdrop {
                x: c.f32(2)?,
                y: c.f32(3)?,
                w: c.f32(4)?,
                h: c.f32(5)?,
            },
            11 => Intencao::MoveBackdrop {
                id: c.u32(2)?,
                dx: c.f32(3)?,
                dy: c.f32(4)?,
            },
            12 => Intencao::ResizeBackdrop {
                id: c.u32(2)?,
                left: c.bool(3)?,
                dx: c.f32(4)?,
                dy: c.f32(5)?,
            },
            13 => Intencao::DeleteBackdrop(c.u32(2)?),
            14 => Intencao::Rename {
                alvo: match c.varint(2)? {
                    0 => Nomeado::No(a(3)?),
                    1 => Nomeado::Grupo(c.u32(3)?),
                    _ => Nomeado::Fundo(c.u32(3)?),
                },
                nome: c.string(4)?,
            },
            15 => Intencao::SetBackdropColor {
                id: c.u32(2)?,
                cor: u8::try_from(c.varint(3)?).map_err(|_| CodecError::BadVarint)?,
            },
            16 => Intencao::DuplicateSelection(c.alvos(2)?),
            17 => Intencao::SmartConnect {
                from: a(2)?,
                from_port: c.u16(3)?,
                tipo: c.string(4)?,
                x: c.f32(5)?,
                y: c.f32(6)?,
            },
            18 => Intencao::SwapInChain {
                a: a(2)?,
                b: a(3)?,
                back_dx: c.f32(4)?,
                back_dy: c.f32(5)?,
            },
            19 => Intencao::SpliceExistingIntoWire {
                node: a(2)?,
                to: a(3)?,
                to_port: c.u16(4)?,
            },
            20 => Intencao::SmartConnectBack {
                to: a(2)?,
                to_port: c.u16(3)?,
                tipo: c.string(4)?,
                x: c.f32(5)?,
                y: c.f32(6)?,
            },
            21 => {
                let ns = c.alvos(2)?;
                let ps = c
                    .todos(3)
                    .map(|b| u16_de(rec::as_varint(b)?))
                    .collect::<Result<Vec<_>, _>>()?;
                if ns.len() != ps.len() {
                    return Err(CodecError::Truncated);
                }
                Intencao::CutWires(ns.into_iter().zip(ps).collect())
            }
            22 => Intencao::MoveWireEnd {
                from: a(2)?,
                from_port: c.u16(3)?,
                old_to: a(4)?,
                old_to_port: c.u16(5)?,
                new_to: match c.um(6) {
                    Some(_) => Some((a(6)?, c.u16(7)?)),
                    None => None,
                },
            },
            23 => Intencao::GroupSelection(c.alvos(2)?),
            24 => Intencao::Ungroup(c.u32(2)?),
            25 => Intencao::EnterSubgraph(c.u32(2)?),
            26 => Intencao::GoToLevel(match c.um(2) {
                Some(_) => Some(c.u32(2)?),
                None => None,
            }),
            27 => Intencao::SpliceReroute {
                to: a(2)?,
                to_port: c.u16(3)?,
                x: c.f32(4)?,
                y: c.f32(5)?,
            },
            28 => Intencao::SpliceNode {
                to: a(2)?,
                to_port: c.u16(3)?,
                tipo: c.string(4)?,
                x: c.f32(5)?,
                y: c.f32(6)?,
            },
            29 => Intencao::ArrangeLayout,
            30 => Intencao::SetBypass {
                nodes: c.alvos(2)?,
                on: c.bool(3)?,
            },
            31 => Intencao::CopySelection(c.alvos(2)?),
            32 => Intencao::Paste,
            33 => Intencao::FixInert(a(2)?),
            34 => Intencao::SetNodeHelp(c.bool(2)?),
            n => Intencao::Desconhecida(n),
        })
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
struct Campos<'a>(Vec<(u32, &'a [u8])>);

impl<'a> Campos<'a> {
    fn de(bytes: &'a [u8]) -> Result<Self, CodecError> {
        rec::fields(bytes).map(Campos)
    }

    fn um(&self, tag: u32) -> Option<&'a [u8]> {
        self.0
            .iter()
            .rev()
            .find(|(t, _)| *t == tag)
            .map(|(_, b)| *b)
    }

    fn todos(&self, tag: u32) -> impl Iterator<Item = &'a [u8]> + '_ {
        self.0
            .iter()
            .filter(move |(t, _)| *t == tag)
            .map(|(_, b)| *b)
    }

    fn precisa(&self, tag: u32) -> Result<&'a [u8], CodecError> {
        self.um(tag).ok_or(CodecError::MissingField {
            record: "motion",
            tag,
        })
    }

    fn varint(&self, tag: u32) -> Result<u64, CodecError> {
        rec::as_varint(self.precisa(tag)?)
    }

    fn u32(&self, tag: u32) -> Result<u32, CodecError> {
        u32::try_from(self.varint(tag)?).map_err(|_| CodecError::BadVarint)
    }

    fn u16(&self, tag: u32) -> Result<u16, CodecError> {
        u16_de(self.varint(tag)?)
    }

    fn bool(&self, tag: u32) -> Result<bool, CodecError> {
        rec::as_bool(self.precisa(tag)?)
    }

    fn f32(&self, tag: u32) -> Result<f32, CodecError> {
        rec::as_f32(self.precisa(tag)?)
    }

    fn string(&self, tag: u32) -> Result<String, CodecError> {
        rec::as_str(self.precisa(tag)?).map(str::to_owned)
    }

    fn alvo(&self, tag: u32) -> Result<Alvo, CodecError> {
        Alvo::de_varint(self.varint(tag)?)
    }

    fn alvos(&self, tag: u32) -> Result<Vec<Alvo>, CodecError> {
        self.todos(tag)
            .map(|b| Alvo::de_varint(rec::as_varint(b)?))
            .collect()
    }
}

fn u16_de(v: u64) -> Result<u16, CodecError> {
    u16::try_from(v).map_err(|_| CodecError::BadVarint)
}
