//! **O container aberto NO LUGAR** (doc 17 item 10, maquete C2): no Arrange, uma instância de
//! container abre as lanes de dentro logo abaixo da dela, no tempo da CENA; as irmãs da MESMA lane
//! (o mesmo `StripSource::Container`) mostram o mesmo conteúdo esmaecido, e editar dentro muda o
//! container — logo as irmãs também (o *symbol* do Harmony).
//!
//! Três peças, uma conta: [`instance_map`] (o mapa de cada volta, de `nest_map::voltas_of`),
//! [`no_lugar_view`] (o que o painel pinta) e [`aplicar`] (a edição em tempo de cena convertida
//! para o tempo do container num sítio só, [`para_o_local`]).

use crate::doc::TimelineDoc;
use crate::intent::TimelineIntent;
use crate::nest_map::voltas_of;
use crate::snapshot::StripView;
use crate::state::TimelineState;
use crate::{EnterStep, StripId};
use ph2d_core::Playhead;

/// Uma volta de uma instância: `[u0, u1]` do container (o tempo de dentro) tocado em `[t0, t1]` da
/// cena. `reversed` = a volta de VOLTA de um ping-pong (`u1 < u0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Volta {
    /// Segundo da cena onde a volta começa.
    pub t0: f64,
    /// Segundo da cena onde ela acaba.
    pub t1: f64,
    /// Segundo do container em `t0`.
    pub u0: f64,
    /// Segundo do container em `t1`.
    pub u1: f64,
    /// A volta lê o container de trás para a frente.
    pub reversed: bool,
}

impl Volta {
    /// O segundo da cena em que esta volta lê `u` do container (afim, sem grampo).
    #[must_use]
    pub fn scene(&self, u: f64) -> f64 {
        self.t0 + (u - self.u0) * (self.t1 - self.t0) / (self.u1 - self.u0)
    }
}

/// **O mapa de uma instância de container** da pilha da cena: onde ela mora e cada volta que
/// cabe na janela (velocidade, recorte, Loop/PingPong). `voltas` nunca é vazio.
#[derive(Clone, Debug, PartialEq)]
pub struct InstanceMap {
    /// O container que a instância toca.
    pub container: usize,
    /// A lane da cena onde a instância mora.
    pub lane: usize,
    /// A janela da instância na cena: início.
    pub t0: f64,
    /// Fim (exclusivo).
    pub t1: f64,
    /// As voltas, por ordem.
    pub voltas: Vec<Volta>,
}

impl InstanceMap {
    /// Segundos da cena por segundo do container (`1 / velocidade`).
    #[must_use]
    pub fn escala(&self) -> f64 {
        let v = self.voltas[0];
        (v.t1 - v.t0) / (v.u1 - v.u0)
    }

    /// O segundo do container na cena `t`, pela PRIMEIRA volta (afim, sem grampo).
    #[must_use]
    pub fn local(&self, t: f64) -> f64 {
        let v = self.voltas[0];
        v.u0 + (t - v.t0) / self.escala()
    }

    /// O segundo da cena em que a primeira volta lê `u` — o inverso de [`Self::local`].
    #[must_use]
    pub fn scene(&self, u: f64) -> f64 {
        self.voltas[0].scene(u)
    }
}

fn mapa_de(container: usize, lane: usize, s: &crate::ClipStrip) -> Option<InstanceMap> {
    let voltas: Vec<Volta> = voltas_of(s)
        .into_iter()
        .map(|(t0, t1, u0, u1, reversed)| Volta {
            t0,
            t1,
            u0,
            u1,
            reversed,
        })
        .collect();
    (!voltas.is_empty()).then_some(InstanceMap {
        container,
        lane,
        t0: s.t_start,
        t1: s.t_end,
        voltas,
    })
}

/// O mapa da fita `instancia` da pilha da CENA — `None` se ela não existe, não toca um container,
/// ou não tem eixo (fatia-pose, velocidade ≤ 0).
#[must_use]
pub fn instance_map(doc: &TimelineDoc, instancia: StripId) -> Option<InstanceMap> {
    doc.stack().iter().enumerate().find_map(|(lane, l)| {
        let s = l.strips.iter().find(|s| s.id == instancia)?;
        mapa_de(usize::from(s.source.container_index()?), lane, s)
    })
}

/// O container aberto no lugar, como o painel o pinta.
#[derive(Clone, Debug, PartialEq)]
pub struct NoLugarView {
    /// A instância aberta.
    pub instancia: StripId,
    /// A lane da cena onde ela mora (as de dentro entram logo abaixo).
    pub lane: usize,
    /// O container.
    pub container: usize,
    /// O nome dele.
    pub nome: String,
    /// A janela da instância na cena (a pílula «plays a – b»).
    pub janela: (f64, f64),
    /// O loop do container, na cena pela primeira volta da aberta.
    pub loop_cena: Option<(f64, f64)>,
    /// As lanes de dentro, de baixo para cima.
    pub lanes: Vec<NoLugarLane>,
}

/// Uma lane de dentro do container.
#[derive(Clone, Debug, PartialEq)]
pub struct NoLugarLane {
    /// Nome.
    pub name: String,
    /// Muda.
    pub muted: bool,
    /// Peso, `[0, 1]`.
    pub weight: f64,
    /// Uma cópia por fita × volta × instância, já em tempo de cena.
    pub strips: Vec<NoLugarStrip>,
}

/// Uma fita de dentro, numa volta de uma instância.
#[derive(Clone, Debug, PartialEq)]
pub struct NoLugarStrip {
    /// A fita em tempo de CENA (pontos pela volta, durações pela escala).
    pub view: StripView,
    /// A instância de fora a que esta cópia pertence.
    pub instancia: StripId,
    /// `true` = a primeira volta da aberta (editável); `false` = irmã ou volta seguinte (esmaecida).
    pub aberta: bool,
}

/// A fita `sv` (tempo do container) vista pela volta `v` (tempo da cena).
fn na_cena(sv: &StripView, v: &Volta) -> StripView {
    let e = ((v.t1 - v.t0) / (v.u1 - v.u0)).abs();
    let mut o = sv.clone();
    (o.t_start, o.t_end) = (v.scene(sv.t_start), v.scene(sv.t_end));
    (o.blend_in, o.blend_out) = (sv.blend_in * e, sv.blend_out * e);
    (o.lead_in, o.lead_out) = (sv.lead_in * e, sv.lead_out * e);
    o.marks = sv.marks.map(|m| m * e);
    o.volta = sv.volta * e;
    if v.reversed {
        // Lida de trás para a frente, a borda de entrada é a de saída.
        std::mem::swap(&mut o.t_start, &mut o.t_end);
        std::mem::swap(&mut o.blend_in, &mut o.blend_out);
        std::mem::swap(&mut o.lead_in, &mut o.lead_out);
        std::mem::swap(&mut o.ease_locked_in, &mut o.ease_locked_out);
        std::mem::swap(&mut o.curve_in, &mut o.curve_out);
    }
    o
}

/// O [`NoLugarView`] do quadro: só no Arrange (nem Keys nem dentro de um container) e só quando
/// [`TimelineState::no_lugar`] resolve uma instância.
pub(crate) fn no_lugar_view(state: &TimelineState, keys_mode: bool) -> Option<NoLugarView> {
    if keys_mode || !state.edit_path.is_empty() {
        return None;
    }
    let doc = &state.doc;
    let instancia = state.no_lugar?;
    let map = instance_map(doc, instancia)?;
    let named = doc.containers().get(map.container)?;
    // A aberta primeiro; depois as irmãs da MESMA lane, por ordem.
    let fora = &doc.stack()[map.lane];
    let toca =
        |s: &&crate::ClipStrip| s.source.container_index().map(usize::from) == Some(map.container);
    let mut instancias: Vec<&crate::ClipStrip> = fora.strips.iter().filter(toca).collect();
    instancias.sort_by_key(|s| s.id != instancia);
    let (range, ping) = doc.container_loop(map.container);
    let lanes = named
        .stack
        .iter()
        .map(|l| {
            let base = crate::snapshot_lane::lane_view(doc, l, range, ping);
            let mut strips = Vec::new();
            for inst in &instancias {
                let Some(m) = mapa_de(map.container, map.lane, inst) else {
                    continue;
                };
                for (k, v) in m.voltas.iter().enumerate() {
                    let (lo, hi) = (v.u0.min(v.u1), v.u0.max(v.u1));
                    strips.extend(
                        base.strips
                            .iter()
                            .filter(|sv| sv.t_start < hi && sv.t_end > lo)
                            .map(|sv| NoLugarStrip {
                                view: na_cena(sv, v),
                                instancia: inst.id,
                                aberta: inst.id == instancia && k == 0,
                            }),
                    );
                }
            }
            NoLugarLane {
                name: base.name,
                muted: base.muted,
                weight: base.weight,
                strips,
            }
        })
        .collect();
    Some(NoLugarView {
        instancia,
        lane: map.lane,
        container: map.container,
        nome: named.name.clone(),
        janela: (map.t0, map.t1),
        loop_cena: named.loop_range.map(|(a, b)| (map.scene(a), map.scene(b))),
        lanes,
    })
}

/// **A conversão cena → container**, num sítio só: pontos pela [`InstanceMap::local`], durações por
/// [`InstanceMap::escala`]. `None` = intenção fora da família (fita, lane, loop do container).
pub(crate) fn para_o_local(map: &InstanceMap, intent: TimelineIntent) -> Option<TimelineIntent> {
    use TimelineIntent as I;
    let p = |t: f64| map.local(t);
    let d = |s: f64| s / map.escala();
    Some(match intent {
        I::AddStrip {
            lane,
            source,
            t_start,
            t_end,
        } => I::AddStrip {
            lane,
            source,
            t_start: p(t_start),
            t_end: p(t_end),
        },
        I::MoveStrip {
            lane,
            to_lane,
            id,
            t_start,
        } => I::MoveStrip {
            lane,
            to_lane,
            id,
            t_start: p(t_start),
        },
        I::TrimStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => I::TrimStrip {
            lane,
            id,
            edge,
            t: p(t),
            from: p(from),
        },
        I::StretchStrip {
            lane,
            id,
            edge,
            t,
            from,
        } => I::StretchStrip {
            lane,
            id,
            edge,
            t: p(t),
            from: p(from),
        },
        I::SetStripEase {
            lane,
            id,
            edge,
            seconds,
        } => I::SetStripEase {
            lane,
            id,
            edge,
            seconds: d(seconds),
        },
        I::SetStripLead {
            lane,
            id,
            edge,
            seconds,
        } => I::SetStripLead {
            lane,
            id,
            edge,
            seconds: d(seconds),
        },
        I::SetContainerLoop {
            range, ping_pong, ..
        } => I::SetContainerLoop {
            container: map.container,
            range: range.map(|(a, b)| (p(a), p(b))),
            ping_pong,
        },
        // Sem tempo: passam como vieram.
        keep @ (I::RemoveStrip { .. }
        | I::DuplicateStrip { .. }
        | I::SetStripLoop { .. }
        | I::SetStripSpeed { .. }
        | I::SetStripCurve { .. }
        | I::SetLaneMuted { .. }
        | I::SetLaneWeight { .. }
        | I::SetLaneMode { .. }) => keep,
        _ => return None,
    })
}

/// Aplica `intent` (tempo de cena) DENTRO do container de `instancia`: converte e roteia pela
/// pilha do container, com o `edit_path` trocado só durante a chamada. O undo é o da edição de
/// dentro (`edit_at`), igual a qualquer outra.
pub(crate) fn aplicar(
    state: &mut TimelineState,
    playhead: &mut Playhead,
    instancia: StripId,
    intent: TimelineIntent,
) {
    let Some(map) = instance_map(&state.doc, instancia) else {
        return;
    };
    let Some(dentro) = para_o_local(&map, intent) else {
        return;
    };
    if let TimelineIntent::SetContainerLoop {
        container,
        range,
        ping_pong,
    } = dentro
    {
        // O relógio entregue é o da CENA (Arrange): escrever o loop sem o sincronizar nele.
        state.doc.set_container_loop(container, range, ping_pong);
        state.doc.clamp_loops_to_lengths();
        return;
    }
    let passo = EnterStep {
        container: map.container,
        lane: map.lane,
        strip: Some(instancia),
    };
    let caminho = std::mem::replace(&mut state.edit_path, vec![passo]);
    crate::apply_intent(state, playhead, dentro);
    state.edit_path = caminho;
}
