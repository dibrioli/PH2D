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
use ph2d_pixel_core::frames;

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
    /// Um colchete da faixa do onion ([`crate::onion_faixa`]).
    Onion { right: bool },
    /// A onda da faixa de som `index` ([`crate::som`]): premida em `x0` quando começava em
    /// `inicio_ms`, agora em `ms` (a prévia; o passo só ao soltar).
    Som {
        index: usize,
        x0: f32,
        inicio_ms: i32,
        ms: i32,
    },
    /// A moldura do intervalo, agora sobre a fronteira `alvo`; o rato premiu em `de` e está em `a`
    /// (o bloco voa com ele, [`crate::paint_grid`]); `copy` = Ctrl premido agora.
    Bloco {
        alvo: usize,
        de: (f32, f32),
        a: (f32, f32),
        copy: bool,
    },
}

/// O bloco agarrado como a grelha o pinta: a ordem prevista, o deslocamento do voo e o vão.
pub(crate) struct Voo {
    pub(crate) info: PixelPanelInfo,
    pub(crate) dx: f32,
    pub(crate) dy: f32,
    /// As células `(fila, coluna)` onde cai; `None` = a coluna inteira.
    pub(crate) vao: Vec<(Option<usize>, usize)>,
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

/// Durante o arrasto do bloco: a ordem prevista (a lei de `move_block` / `copy_block`; um
/// intervalo de cels não mexe nos quadros), o voo e o vão.
pub(crate) fn bloco_previsto(st: &PixelTimelinePanelState, info: &PixelPanelInfo) -> Option<Voo> {
    let Some(Arrasto::Bloco { alvo, de, a, copy }) = st.arrasto else {
        return None;
    };
    if info.range.kind == K::Cels {
        let geo = st.geo.as_ref()?;
        let vao = cels_destinos(info, geo, de, a)?
            .into_iter()
            .map(|(_, fila, c)| (Some(fila), c))
            .collect();
        let (dx, dy) = (a.0 - de.0, a.1 - de.1);
        let info = info.clone();
        return Some(Voo { info, dx, dy, vao });
    }
    let em = |f: &PixelFrameCol| info.range.frames.contains(&f.id);
    let k = info.frames.iter().filter(|f| em(f)).count();
    let at = frames::landing(&info.frames, em, alvo, copy);
    let frames = if copy {
        // As cópias ainda não têm id: um id que nada usa, para a moldura ficar só nos originais.
        let mut v = frames::block_copy_order(&info.frames, em, alvo);
        for (j, f) in v[at..at + k].iter_mut().enumerate() {
            f.id = u32::MAX - u32::try_from(j).unwrap_or(0);
        }
        v
    } else {
        frames::block_order(&info.frames, em, alvo)
    };
    Some(Voo {
        info: PixelPanelInfo {
            frames,
            ..info.clone()
        },
        dx: a.0 - de.0,
        dy: 0.0,
        vao: (at..at + k).map(|c| (None, c)).collect(),
    })
}

/// O pedido de um intervalo de cels largado: destinos na grelha em `pairs`, à direita do último
/// quadro em `beyond` (`(camada, k)` = o `k`-ésimo quadro novo, que o Trial CRIA:
/// `round3_gaps.cels_dropped_right_of_last_frame`).
fn cels_move(
    info: &PixelPanelInfo,
    geo: &Geo,
    de: (f32, f32),
    a: (f32, f32),
    copy: bool,
) -> Option<PixelRequest> {
    let n = info.frames.len();
    let (mut pairs, mut beyond) = (Vec::new(), Vec::new());
    for ((l0, f0), fila, c) in cels_destinos(info, geo, de, a)? {
        let l = info.layers[fila].id;
        match info.frames.get(c) {
            Some(f) => pairs.push(((l0, f0), (l, f.id))),
            None => beyond.push(((l0, f0), (l, u32::try_from(c - n).ok()?))),
        }
    }
    Some(PixelRequest::CelsMove {
        pairs,
        beyond,
        copy,
    })
}

/// Um destino de cel: `((camada, quadro) de, fila, coluna)`.
type CelDestino = ((u32, u32), usize, usize);

/// Os destinos `((camada, quadro) de, fila, coluna)` de um intervalo de cels arrastado de `de` até
/// `a`: anda as colunas e as filas que o rato percorreu (arredondadas — agarra-se a moldura na
/// divisa entre colunas); as filas e a 1.ª coluna prendem-se à grelha, à direita a coluna passa
/// do último quadro. `None` se algum destino cai numa fila de grupo: o Trial PERDE esses cels
/// (`round3_gaps.cels_dropped_on_group_row`); nós recusamos, para não perder dados.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "colunas e filas da grelha"
)]
fn cels_destinos(
    info: &PixelPanelInfo,
    geo: &Geo,
    de: (f32, f32),
    a: (f32, f32),
) -> Option<Vec<CelDestino>> {
    let (n, m) = (info.frames.len(), info.layers.len());
    let cols: Vec<usize> = info
        .range
        .frames
        .iter()
        .filter_map(|&f| pos(info, f))
        .collect();
    let filas: Vec<usize> = (0..m)
        .filter(|&i| info.range.layers.contains(&info.layers[i].id))
        .collect();
    let presa = |d: isize, v: &[usize], len: usize| {
        let lo = v.iter().min().map_or(0, |&x| -(x as isize));
        let hi = v.iter().max().map_or(0, |&x| (len - 1 - x) as isize);
        ph2d_editor_core::math::safe_clamp_ord(d, lo, hi)
    };
    let dc = ((a.0 - de.0) / geo.col_w).round() as isize;
    let dc = dc.max(presa(dc, &cols, n));
    let df = presa(((a.1 - de.1) / geo.pitch).round() as isize, &filas, m);
    let mut out = Vec::new();
    for &i in &filas {
        let fila = i.checked_add_signed(df)?;
        if info.layers[fila].group {
            return None;
        }
        for &c in &cols {
            let de = (info.layers[i].id, info.frames[c].id);
            out.push((de, fila, c.checked_add_signed(dc)?));
        }
    }
    Some(out)
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
        (H::OnionEdge { right }, GesturePhase::Begin) => {
            st.arrasto = Some(Arrasto::Onion { right });
            Vec::new()
        }
        (H::AudioTrack { index }, GesturePhase::Begin) => {
            let index = usize::from(index);
            if let Some(l) = ph2d_pixel_audio::canal::linhas().get(index) {
                let (x0, inicio_ms, ms) = (g.x, l.inicio_ms, l.inicio_ms);
                st.arrasto = Some(Arrasto::Som {
                    index,
                    x0,
                    inicio_ms,
                    ms,
                });
            }
            Vec::new()
        }
        (H::RangeOutline, GesturePhase::Begin) => {
            st.arrasto = Some(Arrasto::Bloco {
                alvo: geo.fronteira(g.x, n),
                de: (g.x, g.y),
                a: (g.x, g.y),
                copy: g.mods.cmd,
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
                Some(Arrasto::Onion { right }) => {
                    crate::onion_faixa::arrasta(info, right, geo.col(g.x, n));
                    None
                }
                Some(Arrasto::Som {
                    index,
                    x0,
                    inicio_ms,
                    ..
                }) => {
                    let d = crate::som::ms_de(info, &geo, g.x) - crate::som::ms_de(info, &geo, x0);
                    #[expect(clippy::cast_possible_truncation, reason = "ms de um arrasto")]
                    let ms = inicio_ms.saturating_add(d.round() as i32);
                    st.arrasto = Some(Arrasto::Som {
                        index,
                        x0,
                        inicio_ms,
                        ms,
                    });
                    if fim && ms != inicio_ms {
                        let e = ph2d_pixel_audio::faixas::Edicao::Move(index, ms);
                        ph2d_pixel_audio::canal::pede(ph2d_pixel_audio::canal::Pedido::Edita(e));
                    }
                    None
                }
                Some(Arrasto::Bloco { de, .. }) => {
                    let (alvo, a, copy) = (geo.fronteira(g.x, n), (g.x, g.y), g.mods.cmd);
                    st.arrasto = Some(Arrasto::Bloco { alvo, de, a, copy });
                    if !fim {
                        None
                    } else if info.range.kind == K::Cels {
                        cels_move(info, &geo, de, a, copy)
                    } else {
                        let before = u32::try_from(alvo).unwrap_or(u32::MAX);
                        Some(PixelRequest::RangeMove { before, copy })
                    }
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
