//! As INTENÇÕES do grafo na carga de um [`super::passo::Passo::Grafo`]: o tipo na etiqueta `1` (os
//! números `1..=34` são para sempre; um leitor antigo devolve [`Intencao::Desconhecida`]), os campos de
//! `2` em diante. Irmão de [`super::passo`] pelo tecto de linhas de um ficheiro.

use super::passo::{Alvo, Campos, Carga, Escolha, Nomeado, u16_de};
use ph2d_macro::CodecError;
use ph2d_macro::rec::{self, Record};

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
