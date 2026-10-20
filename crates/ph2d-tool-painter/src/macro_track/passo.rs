//! Os PASSOS da trilha do Painter e a sua carga ([`ph2d_macro::rec`]: o tipo na etiqueta `1`, os
//! campos de `2` em diante). O NÚMERO de cada tipo é para sempre: um leitor antigo que não o conhece
//! devolve [`Passo::Desconhecido`], e o tocador recusa-o em voz alta — saltá-lo mudaria a pintura.
//!
//! As posições guardam-se RELATIVAS ao quadro de referência (o tamanho da tela ao gravar, em `f64`):
//! na mesma tela voltam ao mesmo `f32`, numa tela 2× caem no mesmo lugar relativo.

use crate::tool::paint::fita::{MarcaDaAgua, RespostaDoWorker};
use ph2d_a11y::NodeId;
use ph2d_editor_core::tool::{CanvasPointer, PanelEvent, PointerPhase};
use ph2d_macro::CodecError;
use ph2d_macro::rec::{self, Record};

/// A carga codificada de um passo (o registo etiquetado que vai na `Event::payload`).
pub type Carga = Vec<u8>; // COLOR-RAW-OK: um registo etiquetado do formato da macro, não pixels

#[derive(Clone, Debug, PartialEq)]
pub enum Passo {
    Ponteiro {
        pos: [f64; 2],
        pressao: f32,
        tilt: [f32; 2],
        fase: PointerPhase,
    },
    Hover([f64; 2]),
    Tique(f32),
    Painel(PanelEvent),
    Ativa,
    Desativa,
    Desfaz,
    Refaz,
    PedeCommit,
    TiraCommit,
    Assado,
    TiraAssadoAdiado,
    Preview,
    PreviewBbox,
    PreviewSujo,
    PreviewAtual,
    Completo,
    Borracha,
    TamanhoPx(f32),
    EmpurraTamanho(i32),
    Cor([u8; 3]),
    RestringeLinha(bool),
    Warp(ph2d_painter_brush::canvas_warp::CanvasWarp),
    GridSnap(Option<[f64; 2]>),
    GridApaga(bool),
    ComporPorQuadro(bool),
    TolDePega(f32),
    Modificadores([bool; 3]),
    SeguraRascunho(bool),
    /// `wet_preset_aplicar`: os parâmetros (JSON do preset) e as texturas (PNG por id).
    PresetAplica {
        parametros: String,
        texturas: Vec<(String, Vec<u8>)>,
    },
    PresetVolumes(String),
    PresetModo(String),
    TiraPedidoPreset,
    TiraPedidoGrao,
    TiraPedidoForma,
    TexturaGrao {
        lum: Vec<u8>,
        w: u32,
        h: u32,
    },
    FormaRgba {
        rgba: Vec<u8>,
        w: u32,
        h: u32,
    },
    /// `take_brush_shape_hose_request`: o pedido do «Add image» (L19) consumido.
    TiraPedidoMangueira,
    /// `add_brush_shape_hose_image_rgba`: a imagem acrescentada à ponta de N imagens (L19).
    MangueiraRgba {
        rgba: Vec<u8>,
        w: u32,
        h: u32,
    },
    /// `take_brush_dual_image_request`: o pedido do «Dual image» (L6) consumido.
    TiraPedidoDual,
    /// `set_brush_dual_image_rgba`: a imagem da 2.ª ponta do dual brush (L6).
    DualRgba {
        rgba: Vec<u8>,
        w: u32,
        h: u32,
    },
    FalloffPonto {
        id: u8,
        d: f32,
        s: f32,
    },
    FalloffAcrescenta {
        d: f32,
        s: f32,
    },
    FalloffPega {
        id: u8,
        handle: u8,
    },
    FalloffTira(u8),
    RefrescaFonteDaForma,
    RefrescaCorDaForma,
    TiraPreviewDaFonte,
    TipoDaTextura(u8),
    TipoDaForma(u8),
    SnapDaLinha(bool),
    EscalaUniforme(bool),
    /// Parar a gravação: o motor da água volta para casa (a última marca), nos dois lados.
    Para,
    /// Uma resposta do worker da água ([`crate::tool::paint::fita`]).
    Agua(RespostaDoWorker),
    /// Uma porta que a macro não reproduz foi chamada a gravar (o nome dela).
    Recusada(String),
    /// A trilha deixou de ser a gravação no tempo (uma etapa re-cronometrada ou cuja conferência
    /// divergiu): a água larga a fita e corre no relógio fixo. Nunca gravado — só a trilha que toca o tem
    /// (`ph2d_macro::etapa`).
    SaiDaGravacao,
    /// `begin_colordrop_fill`: o C&F entra em Fill para um ColorDrop (o modo a que volta).
    ColorDropComeca(String),
    /// `set_fill_threshold`: o limiar do ColorDrop (o modal, ou o gesto parado da shell).
    LimiarDoFill(f32),
    /// `fill_commit`: o ColorDrop confirmado (o soltar do gesto parado).
    FillCommit,
    /// `restore_after_colordrop`: o C&F volta ao modo de antes.
    ColorDropRestaura,
    /// **A faixa `n` leva o pincel daqui** (W16): no instante GRAVADO de uma etapa deslocada, o pincel em
    /// vigor passa a ser o do traço dela, onde quer que ela toque. Nunca gravado — só a trilha que toca o
    /// tem (`ph2d_macro::etapa`), como o [`Passo::SaiDaGravacao`]; o número fica longe dos gravados.
    PincelDaFaixa(u32),
    /// Um tipo que este leitor não conhece (gravado por uma versão mais nova).
    Desconhecido(u64),
}

fn fase_u8(f: PointerPhase) -> u64 {
    match f {
        PointerPhase::Down => 0,
        PointerPhase::Move => 1,
        PointerPhase::Up => 2,
        PointerPhase::Hover => 3,
    }
}

fn fase_de(v: u64) -> PointerPhase {
    match v {
        0 => PointerPhase::Down,
        2 => PointerPhase::Up,
        3 => PointerPhase::Hover,
        _ => PointerPhase::Move,
    }
}

impl Passo {
    /// O ponteiro como a porta o recebeu, relativo à tela `(w, h)`.
    #[must_use]
    pub fn de_ponteiro(ev: CanvasPointer, w: f64, h: f64) -> Self {
        Passo::Ponteiro {
            pos: rel(ev.pos, w, h),
            pressao: ev.pressure,
            tilt: ev.tilt,
            fase: ev.phase,
        }
    }

    #[must_use]
    pub fn codifica(&self) -> Carga {
        let mut r = Record::new();
        let k = |r: &mut Record, n: u64| {
            r.varint(1, n);
        };
        match self {
            Passo::Ponteiro {
                pos,
                pressao,
                tilt,
                fase,
            } => {
                k(&mut r, 1);
                r.f64(2, pos[0]).f64(3, pos[1]).f32(4, *pressao);
                r.f32(5, tilt[0]).f32(6, tilt[1]).varint(7, fase_u8(*fase));
            }
            Passo::Hover(p) => {
                k(&mut r, 2);
                r.f64(2, p[0]).f64(3, p[1]);
            }
            Passo::Tique(dt) => {
                k(&mut r, 3);
                r.f32(2, *dt);
            }
            Passo::Painel(ev) => {
                k(&mut r, 4);
                match ev {
                    PanelEvent::Click(id) => {
                        r.varint(2, 1).varint(3, id.0);
                    }
                    PanelEvent::SetValue(id, v) => {
                        r.varint(2, 2).varint(3, id.0).f64(4, *v);
                    }
                    PanelEvent::Toggle(id, b) => {
                        r.varint(2, 3).varint(3, id.0).bool(5, *b);
                    }
                    PanelEvent::SelectOption(id, s) => {
                        r.varint(2, 4).varint(3, id.0).str(6, s);
                    }
                }
            }
            Passo::Ativa => k(&mut r, 5),
            Passo::Desativa => k(&mut r, 6),
            Passo::Desfaz => k(&mut r, 7),
            Passo::Refaz => k(&mut r, 8),
            Passo::PedeCommit => k(&mut r, 9),
            Passo::TiraCommit => k(&mut r, 10),
            Passo::Assado => k(&mut r, 11),
            Passo::TiraAssadoAdiado => k(&mut r, 12),
            Passo::Preview => k(&mut r, 13),
            Passo::PreviewBbox => k(&mut r, 14),
            Passo::PreviewSujo => k(&mut r, 15),
            Passo::PreviewAtual => k(&mut r, 16),
            Passo::Completo => k(&mut r, 17),
            Passo::Borracha => k(&mut r, 18),
            Passo::TamanhoPx(px) => {
                k(&mut r, 19);
                r.f32(2, *px);
            }
            Passo::EmpurraTamanho(d) => {
                k(&mut r, 20);
                r.bytes(2, &d.to_le_bytes());
            }
            Passo::Cor(c) => {
                k(&mut r, 21);
                r.bytes(2, c);
            }
            Passo::RestringeLinha(b) => {
                k(&mut r, 22);
                r.bool(2, *b);
            }
            Passo::Warp(w) => {
                k(&mut r, 23);
                let mut b = Vec::with_capacity(18 * 4);
                for v in w.linear.iter().flatten().chain(w.curve.iter().flatten()) {
                    b.extend_from_slice(&v.to_le_bytes());
                }
                r.bytes(2, &b);
            }
            Passo::GridSnap(p) => {
                k(&mut r, 24);
                if let Some(p) = p {
                    r.f64(2, p[0]).f64(3, p[1]);
                }
            }
            Passo::GridApaga(b) => {
                k(&mut r, 25);
                r.bool(2, *b);
            }
            Passo::ComporPorQuadro(b) => {
                k(&mut r, 26);
                r.bool(2, *b);
            }
            Passo::TolDePega(px) => {
                k(&mut r, 27);
                r.f32(2, *px);
            }
            Passo::Modificadores([s, c, a]) => {
                k(&mut r, 28);
                r.bool(2, *s).bool(3, *c).bool(4, *a);
            }
            Passo::SeguraRascunho(b) => {
                k(&mut r, 29);
                r.bool(2, *b);
            }
            Passo::PresetAplica {
                parametros,
                texturas,
            } => {
                k(&mut r, 30);
                r.str(2, parametros);
                for (id, png) in texturas {
                    let mut t = Record::new();
                    t.str(1, id).bytes(2, png);
                    r.bytes(3, &t.finish());
                }
            }
            Passo::PresetVolumes(j) => {
                k(&mut r, 31);
                r.str(2, j);
            }
            Passo::PresetModo(m) => {
                k(&mut r, 32);
                r.str(2, m);
            }
            Passo::TiraPedidoPreset => k(&mut r, 33),
            Passo::TiraPedidoGrao => k(&mut r, 34),
            Passo::TiraPedidoForma => k(&mut r, 35),
            Passo::TexturaGrao { lum, w, h } => {
                k(&mut r, 36);
                r.bytes(2, lum)
                    .varint(3, u64::from(*w))
                    .varint(4, u64::from(*h));
            }
            Passo::FormaRgba { rgba, w, h } => {
                k(&mut r, 37);
                r.bytes(2, rgba)
                    .varint(3, u64::from(*w))
                    .varint(4, u64::from(*h));
            }
            Passo::FalloffPonto { id, d, s } => {
                k(&mut r, 38);
                r.varint(2, u64::from(*id)).f32(3, *d).f32(4, *s);
            }
            Passo::FalloffAcrescenta { d, s } => {
                k(&mut r, 39);
                r.f32(3, *d).f32(4, *s);
            }
            Passo::FalloffPega { id, handle } => {
                k(&mut r, 40);
                r.varint(2, u64::from(*id)).varint(3, u64::from(*handle));
            }
            Passo::FalloffTira(id) => {
                k(&mut r, 41);
                r.varint(2, u64::from(*id));
            }
            Passo::RefrescaFonteDaForma => k(&mut r, 42),
            Passo::RefrescaCorDaForma => k(&mut r, 43),
            Passo::TiraPreviewDaFonte => k(&mut r, 44),
            Passo::Para => k(&mut r, 45),
            Passo::TipoDaTextura(v) => {
                k(&mut r, 46);
                r.varint(2, u64::from(*v));
            }
            Passo::TipoDaForma(v) => {
                k(&mut r, 47);
                r.varint(2, u64::from(*v));
            }
            Passo::SnapDaLinha(b) => {
                k(&mut r, 48);
                r.bool(2, *b);
            }
            Passo::EscalaUniforme(b) => {
                k(&mut r, 49);
                r.bool(2, *b);
            }
            Passo::Agua(resp) => {
                k(&mut r, 50);
                match resp {
                    RespostaDoWorker::Voltou(m) => {
                        r.varint(2, 1).varint(3, m.frame);
                        r.varint(4, m.estagio.map_or(0, |e| u64::from(e) + 1));
                    }
                    RespostaDoWorker::NaoVoltou => {
                        r.varint(2, 2);
                    }
                    RespostaDoWorker::Passos(n) => {
                        r.varint(2, 3).varint(5, *n);
                    }
                }
            }
            Passo::Recusada(porta) => {
                k(&mut r, 51);
                r.str(2, porta);
            }
            Passo::SaiDaGravacao => k(&mut r, 52),
            Passo::ColorDropComeca(modo) => {
                k(&mut r, 53);
                r.str(2, modo);
            }
            Passo::LimiarDoFill(v) => {
                k(&mut r, 54);
                r.f32(2, *v);
            }
            Passo::FillCommit => k(&mut r, 55),
            Passo::ColorDropRestaura => k(&mut r, 56),
            Passo::PincelDaFaixa(n) => {
                k(&mut r, 200);
                r.varint(2, u64::from(*n));
            }
            Passo::TiraPedidoMangueira => k(&mut r, 57),
            Passo::MangueiraRgba { rgba, w, h } => {
                k(&mut r, 58);
                r.bytes(2, rgba)
                    .varint(3, u64::from(*w))
                    .varint(4, u64::from(*h));
            }
            Passo::TiraPedidoDual => k(&mut r, 59),
            Passo::DualRgba { rgba, w, h } => {
                k(&mut r, 60);
                r.bytes(2, rgba)
                    .varint(3, u64::from(*w))
                    .varint(4, u64::from(*h));
            }
            Passo::Desconhecido(n) => k(&mut r, *n),
        }
        r.finish()
    }

    /// Lê uma carga. Campo ausente = o neutro do tipo; tipo desconhecido = [`Passo::Desconhecido`].
    pub fn descodifica(b: &[u8]) -> Result<Self, CodecError> {
        let campos = rec::fields(b)?;
        let campo = |t: u32| campos.iter().rev().find(|(k, _)| *k == t).map(|(_, v)| *v);
        let todos = |t: u32| campos.iter().filter(move |(k, _)| *k == t).map(|(_, v)| *v);
        let u = |t: u32| campo(t).map_or(Ok(0), rec::as_varint);
        let f32_ = |t: u32| campo(t).map_or(Ok(0.0), rec::as_f32);
        let f64_ = |t: u32| campo(t).map_or(Ok(0.0), rec::as_f64);
        let b_ = |t: u32| campo(t).map_or(Ok(false), rec::as_bool);
        let s_ = |t: u32| campo(t).map_or(Ok(String::new()), |v| rec::as_str(v).map(str::to_owned));
        let bytes = |t: u32| campo(t).map_or_else(Vec::new, <[u8]>::to_vec);
        let u8_ = |t: u32| u(t).map(|v| u8::try_from(v).unwrap_or(u8::MAX));
        let u32_ = |t: u32| u(t).map(|v| u32::try_from(v).unwrap_or(u32::MAX));
        Ok(match u(1)? {
            1 => Passo::Ponteiro {
                pos: [f64_(2)?, f64_(3)?],
                pressao: f32_(4)?,
                tilt: [f32_(5)?, f32_(6)?],
                fase: fase_de(u(7)?),
            },
            2 => Passo::Hover([f64_(2)?, f64_(3)?]),
            3 => Passo::Tique(f32_(2)?),
            4 => {
                let id = NodeId(u(3)?);
                match u(2)? {
                    1 => Passo::Painel(PanelEvent::Click(id)),
                    2 => Passo::Painel(PanelEvent::SetValue(id, f64_(4)?)),
                    3 => Passo::Painel(PanelEvent::Toggle(id, b_(5)?)),
                    4 => Passo::Painel(PanelEvent::SelectOption(id, s_(6)?)),
                    _ => Passo::Desconhecido(4),
                }
            }
            5 => Passo::Ativa,
            6 => Passo::Desativa,
            7 => Passo::Desfaz,
            8 => Passo::Refaz,
            9 => Passo::PedeCommit,
            10 => Passo::TiraCommit,
            11 => Passo::Assado,
            12 => Passo::TiraAssadoAdiado,
            13 => Passo::Preview,
            14 => Passo::PreviewBbox,
            15 => Passo::PreviewSujo,
            16 => Passo::PreviewAtual,
            17 => Passo::Completo,
            18 => Passo::Borracha,
            19 => Passo::TamanhoPx(f32_(2)?),
            20 => {
                let v = bytes(2);
                let a: [u8; 4] = v.as_slice().try_into().unwrap_or([0; 4]);
                Passo::EmpurraTamanho(i32::from_le_bytes(a))
            }
            21 => {
                let v = bytes(2);
                Passo::Cor([
                    v.first().copied().unwrap_or(0),
                    v.get(1).copied().unwrap_or(0),
                    v.get(2).copied().unwrap_or(0),
                ])
            }
            22 => Passo::RestringeLinha(b_(2)?),
            23 => {
                let v = bytes(2);
                let mut w = ph2d_painter_brush::canvas_warp::CanvasWarp::rest();
                let mut it = v.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c));
                for x in w
                    .linear
                    .iter_mut()
                    .flatten()
                    .chain(w.curve.iter_mut().flatten())
                {
                    if let Some(y) = it.next() {
                        *x = y;
                    }
                }
                Passo::Warp(w)
            }
            24 => Passo::GridSnap(campo(2).is_some().then_some([f64_(2)?, f64_(3)?])),
            25 => Passo::GridApaga(b_(2)?),
            26 => Passo::ComporPorQuadro(b_(2)?),
            27 => Passo::TolDePega(f32_(2)?),
            28 => Passo::Modificadores([b_(2)?, b_(3)?, b_(4)?]),
            29 => Passo::SeguraRascunho(b_(2)?),
            30 => {
                let mut texturas = Vec::new();
                for t in todos(3) {
                    let c = rec::fields(t)?;
                    let get = |k: u32| c.iter().rev().find(|(x, _)| *x == k).map(|(_, v)| *v);
                    let id = get(1).map_or(Ok(""), rec::as_str)?.to_owned();
                    texturas.push((id, get(2).map_or_else(Vec::new, <[u8]>::to_vec)));
                }
                Passo::PresetAplica {
                    parametros: s_(2)?,
                    texturas,
                }
            }
            31 => Passo::PresetVolumes(s_(2)?),
            32 => Passo::PresetModo(s_(2)?),
            33 => Passo::TiraPedidoPreset,
            34 => Passo::TiraPedidoGrao,
            35 => Passo::TiraPedidoForma,
            36 => Passo::TexturaGrao {
                lum: bytes(2),
                w: u32_(3)?,
                h: u32_(4)?,
            },
            37 => Passo::FormaRgba {
                rgba: bytes(2),
                w: u32_(3)?,
                h: u32_(4)?,
            },
            38 => Passo::FalloffPonto {
                id: u8_(2)?,
                d: f32_(3)?,
                s: f32_(4)?,
            },
            39 => Passo::FalloffAcrescenta {
                d: f32_(3)?,
                s: f32_(4)?,
            },
            40 => Passo::FalloffPega {
                id: u8_(2)?,
                handle: u8_(3)?,
            },
            41 => Passo::FalloffTira(u8_(2)?),
            42 => Passo::RefrescaFonteDaForma,
            43 => Passo::RefrescaCorDaForma,
            44 => Passo::TiraPreviewDaFonte,
            45 => Passo::Para,
            46 => Passo::TipoDaTextura(u8_(2)?),
            47 => Passo::TipoDaForma(u8_(2)?),
            48 => Passo::SnapDaLinha(b_(2)?),
            49 => Passo::EscalaUniforme(b_(2)?),
            50 => Passo::Agua(match u(2)? {
                1 => RespostaDoWorker::Voltou(MarcaDaAgua {
                    frame: u(3)?,
                    estagio: match u(4)? {
                        0 => None,
                        e => Some(u8::try_from(e - 1).unwrap_or(u8::MAX)),
                    },
                }),
                2 => RespostaDoWorker::NaoVoltou,
                3 => RespostaDoWorker::Passos(u(5)?),
                _ => return Ok(Passo::Desconhecido(50)),
            }),
            51 => Passo::Recusada(s_(2)?),
            52 => Passo::SaiDaGravacao,
            53 => Passo::ColorDropComeca(s_(2)?),
            54 => Passo::LimiarDoFill(f32_(2)?),
            55 => Passo::FillCommit,
            56 => Passo::ColorDropRestaura,
            200 => Passo::PincelDaFaixa(u32_(2)?),
            57 => Passo::TiraPedidoMangueira,
            58 => Passo::MangueiraRgba {
                rgba: bytes(2),
                w: u32_(3)?,
                h: u32_(4)?,
            },
            59 => Passo::TiraPedidoDual,
            60 => Passo::DualRgba {
                rgba: bytes(2),
                w: u32_(3)?,
                h: u32_(4)?,
            },
            n => Passo::Desconhecido(n),
        })
    }
}

/// **A etiqueta da faixa** (W16) — em qualquer passo, nunca gravada: só a trilha QUE TOCA a tem, nos
/// passos de uma etapa deslocada (`ph2d_macro::etapa`); o tocador abre um traço por faixa. Os passos usam
/// as etiquetas `1..=7`.
const FAIXA: u32 = 15;

/// A carga `carga` a tocar na faixa `n` (`0` = a do tool).
#[must_use]
pub fn na_faixa(carga: &[u8], n: u32) -> Option<Carga> {
    let campos = rec::fields(carga).ok()?;
    let mut r = Record::new();
    for (t, b) in campos.into_iter().filter(|(t, _)| *t != FAIXA) {
        r.bytes(t, b);
    }
    if n != 0 {
        r.varint(FAIXA, u64::from(n));
    }
    Some(r.finish())
}

/// A faixa de uma carga (`0` = a do tool, a de toda a gravação).
#[must_use]
pub fn faixa_de(carga: &[u8]) -> u32 {
    rec::fields(carga)
        .ok()
        .and_then(|c| c.into_iter().rev().find(|(t, _)| *t == FAIXA))
        .and_then(|(_, b)| rec::as_varint(b).ok())
        .map_or(0, |v| u32::try_from(v).unwrap_or(u32::MAX))
}

/// Uma posição de tela relativa ao quadro `(w, h)`.
#[must_use]
pub fn rel(p: [f32; 2], w: f64, h: f64) -> [f64; 2] {
    [f64::from(p[0]) / w.max(1.0), f64::from(p[1]) / h.max(1.0)]
}

/// A posição relativa de volta numa tela `(w, h)`.
#[must_use]
pub fn abs(p: [f64; 2], w: f64, h: f64) -> [f32; 2] {
    [(p[0] * w.max(1.0)) as f32, (p[1] * h.max(1.0)) as f32]
}
