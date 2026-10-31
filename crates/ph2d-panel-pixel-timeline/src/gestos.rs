//! **Os arrastos da grelha** (P6h), pela lei medida no Aseprite Trial
//! (`docs/PixelArt/ferramentas/oraculo_timeline.md`): clique num cabeçalho escolhe esse quadro e
//! arrastar escolhe um intervalo contíguo; Shift/Ctrl+clique ACRESCENTA um quadro solto (nunca
//! preenche o vão, nunca retira); arrastar sobre os cels escolhe o rectângulo camadas × quadros;
//! arrastar uma ponta de tag larga-a no quadro debaixo do cursor; arrastar a moldura do intervalo
//! move os quadros dele para a fronteira mais próxima.

use crate::PixelTimelinePanelState;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{
    GesturePhase, PixelTimelineGesture, PixelTimelineHitKind as H,
};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::pixel_panel::{
    PixelFrameCol, PixelPanelInfo, PixelRange, PixelRangeKind as K, PixelRequest,
};

/// A geometria da grelha pintada (colunas e filas), guardada pelo pintor.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Geo {
    pub(crate) gx: f32,
    pub(crate) col_w: f32,
    pub(crate) first: usize,
    pub(crate) rows_y: f32,
    pub(crate) pitch: f32,
}

impl Geo {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "≥ 0, presa"
    )]
    fn col(&self, x: f32, n: usize) -> usize {
        let c = ((x - self.gx) / self.col_w).floor().max(0.0) as usize + self.first;
        c.min(n.saturating_sub(1))
    }

    /// A fronteira entre colunas mais próxima de `x` (`0..=n`).
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "≥ 0, presa"
    )]
    fn fronteira(&self, x: f32, n: usize) -> usize {
        (((x - self.gx) / self.col_w).round().max(0.0) as usize + self.first).min(n)
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "≥ 0, presa"
    )]
    fn fila(&self, y: f32, n: usize) -> usize {
        (((y - self.rows_y) / self.pitch).floor().max(0.0) as usize).min(n.saturating_sub(1))
    }
}

/// O arrasto em curso.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Arrasto {
    /// Pelos cabeçalhos, desde a coluna `origem`.
    Cabecalhos { origem: usize },
    /// Pelos cels, desde `(fila, coluna)`.
    Cels { origem: (usize, usize) },
    /// A ponta de uma tag, agora sobre a coluna `alvo`.
    Ponta {
        index: usize,
        right: bool,
        alvo: usize,
    },
    /// A moldura do intervalo, agora sobre a fronteira `alvo`; o rato premiu em `de` e está em `x`
    /// (o bloco voa com ele, [`crate::paint_grid`]).
    Bloco { alvo: usize, de: f32, x: f32 },
}

/// A tag `index` como fica com o arrasto em curso (a pré-visualização da barra).
pub(crate) fn tag_previsto(
    st: &PixelTimelinePanelState,
    index: usize,
    from: u32,
    to: u32,
) -> (u32, u32) {
    match st.arrasto {
        Some(Arrasto::Ponta {
            index: i,
            right,
            alvo,
        }) if i == index => {
            let alvo = u32::try_from(alvo).unwrap_or(u32::MAX);
            if right {
                (from, alvo.max(from))
            } else {
                (alvo, to.max(alvo))
            }
        }
        _ => (from, to),
    }
}

/// Durante o arrasto do bloco: a ordem prevista (a lei de `move_block`), o rato e onde premiu.
pub(crate) fn bloco_previsto(
    st: &PixelTimelinePanelState,
    info: &PixelPanelInfo,
) -> Option<(PixelPanelInfo, f32, f32)> {
    let Some(Arrasto::Bloco { alvo, de, x }) = st.arrasto else {
        return None;
    };
    let em = |f: &PixelFrameCol| info.range.frames.contains(&f.id);
    let frames = ph2d_pixel_core::frames::block_order(&info.frames, em, alvo);
    Some((
        PixelPanelInfo {
            frames,
            ..info.clone()
        },
        de,
        x,
    ))
}

fn pos(info: &PixelPanelInfo, frame: u32) -> Option<usize> {
    info.frames.iter().position(|f| f.id == frame)
}

fn quadros(info: &PixelPanelInfo, a: usize, b: usize) -> Vec<u32> {
    info.frames[a.min(b)..=a.max(b)]
        .iter()
        .map(|f| f.id)
        .collect()
}

/// O intervalo com mais um quadro (e, para cels, mais uma camada): Shift/Ctrl+clique.
fn acrescenta(r: &PixelRange, kind: K, layer: Option<u32>, frame: u32) -> PixelRange {
    let mut out = if r.kind == kind {
        r.clone()
    } else {
        PixelRange {
            kind,
            ..PixelRange::default()
        }
    };
    if !out.frames.contains(&frame) {
        out.frames.push(frame);
    }
    if let Some(l) = layer.filter(|l| !out.layers.contains(l)) {
        out.layers.push(l);
    }
    out
}

/// O pedido de um gesto (o arrasto avança em `st`).
fn pedidos(
    st: &mut PixelTimelinePanelState,
    info: &PixelPanelInfo,
    g: PixelTimelineGesture,
) -> Vec<PixelRequest> {
    let n = info.frames.len();
    let Some(geo) = st.geo.clone().filter(|_| info.editing && n > 0) else {
        return Vec::new();
    };
    let soma = g.mods.shift || g.mods.cmd;
    let fim = matches!(g.phase, GesturePhase::End | GesturePhase::Click);
    match (g.kind, g.phase) {
        (H::Header { frame }, GesturePhase::Begin) => {
            let Some(c) = pos(info, frame) else {
                return Vec::new();
            };
            let range = if soma {
                acrescenta(&info.range, K::Frames, None, frame)
            } else {
                st.arrasto = Some(Arrasto::Cabecalhos { origem: c });
                PixelRange {
                    kind: K::Frames,
                    layers: Vec::new(),
                    frames: vec![frame],
                }
            };
            vec![PixelRequest::FrameSelect(frame), PixelRequest::Range(range)]
        }
        (H::Cel { layer, frame }, GesturePhase::Begin) => {
            let (Some(c), Some(f)) = (
                pos(info, frame),
                info.layers.iter().position(|r| r.id == layer),
            ) else {
                return Vec::new();
            };
            let range = if soma {
                acrescenta(&info.range, K::Cels, Some(layer), frame)
            } else {
                st.arrasto = Some(Arrasto::Cels { origem: (f, c) });
                PixelRange {
                    kind: K::Cels,
                    layers: vec![layer],
                    frames: vec![frame],
                }
            };
            vec![
                PixelRequest::CelSelect { layer, frame },
                PixelRequest::Range(range),
            ]
        }
        (H::TagEdge { index, right }, GesturePhase::Begin) => {
            let index = usize::from(index);
            let Some(t) = info.tags.get(index) else {
                return Vec::new();
            };
            let alvo = usize::try_from(if right { t.to } else { t.from }).unwrap_or(0);
            st.arrasto = Some(Arrasto::Ponta { index, right, alvo });
            Vec::new()
        }
        (H::RangeOutline, GesturePhase::Begin) => {
            st.arrasto = Some(Arrasto::Bloco {
                alvo: geo.fronteira(g.x, n),
                de: g.x,
                x: g.x,
            });
            Vec::new()
        }
        (_, GesturePhase::Update | GesturePhase::End) => {
            let r = match st.arrasto {
                Some(Arrasto::Cabecalhos { origem }) => {
                    let frames = quadros(info, origem, geo.col(g.x, n));
                    (frames != info.range.frames).then(|| {
                        PixelRequest::Range(PixelRange {
                            kind: K::Frames,
                            layers: Vec::new(),
                            frames,
                        })
                    })
                }
                Some(Arrasto::Cels { origem: (f0, c0) }) => {
                    let f1 = geo.fila(g.y, info.layers.len());
                    let layers = info.layers[f0.min(f1)..=f0.max(f1)]
                        .iter()
                        .map(|r| r.id)
                        .collect();
                    let range = PixelRange {
                        kind: K::Cels,
                        layers,
                        frames: quadros(info, c0, geo.col(g.x, n)),
                    };
                    (range != info.range).then_some(PixelRequest::Range(range))
                }
                Some(Arrasto::Ponta { index, right, .. }) => {
                    let alvo = geo.col(g.x, n);
                    st.arrasto = Some(Arrasto::Ponta { index, right, alvo });
                    let t = info.tags.get(index).filter(|_| fim);
                    t.map(|t| {
                        let (from, to) = tag_previsto(st, index, t.from, t.to);
                        PixelRequest::TagRange { index, from, to }
                    })
                }
                Some(Arrasto::Bloco { de, .. }) => {
                    let alvo = geo.fronteira(g.x, n);
                    st.arrasto = Some(Arrasto::Bloco { alvo, de, x: g.x });
                    fim.then(|| PixelRequest::RangeMove {
                        before: u32::try_from(alvo).unwrap_or(u32::MAX),
                    })
                }
                None => None,
            };
            if fim {
                st.arrasto = None;
            }
            r.into_iter().collect()
        }
        (_, GesturePhase::Click) => {
            st.arrasto = None;
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// Drena os gestos deste quadro e empurra os pedidos (antes de pintar: a grelha mostra já o fim).
pub(crate) fn drena(st: &mut PixelTimelinePanelState, ctx: &mut PaintCtx, info: &PixelPanelInfo) {
    let gestos: Vec<PixelTimelineGesture> = ctx
        .host
        .store_mut()
        .drain_pixel_timeline_gestures()
        .collect();
    for g in gestos {
        for r in pedidos(st, info, g) {
            ctx.host.bus_mut().push(EditorAction::Pixel(r));
        }
    }
}
