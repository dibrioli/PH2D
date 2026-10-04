//! ⭐⭐⭐ **O RELEVO ATRAVÉS DOS AJUSTES DE VIZINHANÇA** (`docs/3D/30` §20; decisão do dono de 04/10) —
//! o Gaussiano e a Nitidez borram e afiam o relevo das camadas POR BAIXO deles como borram e afiam a
//! cor delas, no 2D e na peça, por ESTA porta.
//!
//! - *quem entra* é o [`LayerStack::relief_plan`]: as camadas pela ordem da dobra, os ajustes que agem
//!   no relevo e os âmbitos dos grupos — um ajuste dentro de um grupo age no que o GRUPO fez ao relevo
//!   por baixo dele, como o compositor da cor o faz à cor do grupo;
//! - *como* é o [`fold_relief_through`]: a dobra de sempre ([`fold_relief_step`]) amostra a amostra, e
//!   em cada ajuste o passa-baixo da vizinhança (a grelha no 2D, a retícula na peça) sobre a altura e
//!   o corpo, pesado pela opacidade e pela máscara do ajuste.
//!
//! O Brilho e as Sombras/Realces são operações de TOM (limiar e quantidades em luminância): não agem
//! no relevo ([`ReliefEffect::Tone`]) e o painel di-lo. Sem ajuste que aja, quem dobra usa a dobra por
//! amostra de sempre, ao bit.

use super::fold_relief_step;
use super::{LayerId, LayerKind, LayerStack, MAX_GROUP_DEPTH, RELIEF_FOLD_SEED, ReliefComposite};
use ph2d_painter_effects::adjustments::{AdjustmentKind, AdjustmentParams, Neighbourhood};
use rayon::prelude::*;

/// O que um ajuste faz ao relevo por baixo dele.
#[derive(Clone, Copy, Debug)]
pub enum ReliefFilter {
    /// O passa-baixo da vizinhança, com o raio do ajuste.
    Blur { radius: f32 },
    /// A máscara de nitidez: `base + quantidade·(base − passa-baixo)`.
    Sharpen { radius: f32, amount: f32 },
}

/// Como um TIPO de ajuste se relaciona com o relevo — a pergunta do painel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliefEffect {
    /// Borra ou afia o relevo das camadas por baixo.
    Acts,
    /// É de vizinhança mas é uma operação de TOM: o relevo não muda.
    Tone,
    /// Não é de vizinhança (por píxel, ou lê o plano da imagem).
    Not,
}

impl ReliefFilter {
    /// O filtro de um ajuste com estes parâmetros — `None` se ele não age no relevo ou é a identidade
    /// (raio `0`, quantidade `0`).
    #[must_use]
    pub fn of(kind: AdjustmentKind, params: &AdjustmentParams) -> Option<Self> {
        match (kind, params) {
            (AdjustmentKind::GaussianBlur, AdjustmentParams::GaussianBlur(p)) if p.radius > 0.0 => {
                Some(Self::Blur { radius: p.radius })
            }
            (AdjustmentKind::Sharpen, AdjustmentParams::Sharpen(p))
                if p.amount != 0.0 && p.radius > 0.0 =>
            {
                Some(Self::Sharpen {
                    radius: p.radius,
                    amount: p.amount,
                })
            }
            _ => None,
        }
    }

    fn bits(self) -> (u8, u32, u32) {
        match self {
            Self::Blur { radius } => (0, radius.to_bits(), 0),
            Self::Sharpen { radius, amount } => (1, radius.to_bits(), amount.to_bits()),
        }
    }
}

/// ⭐ **O que este tipo de ajuste faz ao relevo** — o painel diz porquê nos de TOM.
#[must_use]
pub fn relief_effect(kind: AdjustmentKind) -> ReliefEffect {
    match kind {
        AdjustmentKind::GaussianBlur | AdjustmentKind::Sharpen => ReliefEffect::Acts,
        AdjustmentKind::Bloom | AdjustmentKind::ShadowsHighlights => ReliefEffect::Tone,
        _ => ReliefEffect::Not,
    }
}

/// Um passo do plano da dobra.
#[derive(Clone, Copy, Debug)]
pub enum ReliefStep {
    /// Uma camada de pintura entra (se tiver relevo), com a profundidade e o modo dela.
    Layer {
        id: LayerId,
        depth: f32,
        composite: ReliefComposite,
    },
    /// Um grupo visível abre um âmbito.
    Open,
    /// …e fecha-o.
    Close,
    /// Um ajuste que age no relevo do seu âmbito.
    Filter {
        id: LayerId,
        filter: ReliefFilter,
        opacity: f32,
        /// A máscara do ajuste e se ela está invertida.
        mask: Option<(LayerId, bool)>,
    },
}

/// ⚠️ A igualdade é AO BIT (`-0` não é `0`): o plano é a assinatura da dobra da peça, e um sinal de
/// zero que mudasse sem redobrar deixava o relevo de antes.
impl PartialEq for ReliefStep {
    fn eq(&self, o: &Self) -> bool {
        match (*self, *o) {
            (
                Self::Layer {
                    id: a,
                    depth: da,
                    composite: ca,
                },
                Self::Layer {
                    id: b,
                    depth: db,
                    composite: cb,
                },
            ) => a == b && da.to_bits() == db.to_bits() && ca == cb,
            (Self::Open, Self::Open) | (Self::Close, Self::Close) => true,
            (
                Self::Filter {
                    id: a,
                    filter: fa,
                    opacity: oa,
                    mask: ma,
                },
                Self::Filter {
                    id: b,
                    filter: fb,
                    opacity: ob,
                    mask: mb,
                },
            ) => a == b && fa.bits() == fb.bits() && oa.to_bits() == ob.to_bits() && ma == mb,
            _ => false,
        }
    }
}

/// O plano tem algum ajuste que age no relevo?
#[must_use]
pub fn relief_plan_filters(plan: &[ReliefStep]) -> bool {
    plan.iter().any(|s| matches!(s, ReliefStep::Filter { .. }))
}

impl LayerStack {
    /// ⭐⭐ **O plano da dobra do relevo** — pela visibilidade do compositor da cor: uma camada ou um
    /// grupo escondido não entra; um ajuste entra com a opacidade dele se estiver visível e agir.
    #[must_use]
    pub fn relief_plan(&self) -> Vec<ReliefStep> {
        let mut out = Vec::new();
        self.push_relief_plan(self.root(), &mut out, 0);
        out
    }

    fn push_relief_plan(&self, ids: &[LayerId], out: &mut Vec<ReliefStep>, depth: usize) {
        if depth > MAX_GROUP_DEPTH {
            return;
        }
        for &id in ids.iter().rev() {
            let Some(l) = self.get(id) else { continue };
            if !l.visible {
                continue;
            }
            match &l.kind {
                LayerKind::Raster(_) | LayerKind::Texture(_) => out.push(ReliefStep::Layer {
                    id,
                    depth: l.impasto_depth,
                    composite: l.impasto_composite,
                }),
                LayerKind::Group(g) => {
                    out.push(ReliefStep::Open);
                    self.push_relief_plan(&g.children, out, depth + 1);
                    out.push(ReliefStep::Close);
                }
                LayerKind::Adjustment(a) => {
                    if !a.visible || a.opacity <= 0.0 || l.opacity <= 0.0 {
                        continue;
                    }
                    let Some(filter) = ReliefFilter::of(a.kind, &a.params) else {
                        continue;
                    };
                    let mask = a.mask.map(LayerId).and_then(|m| match &self.get(m)?.kind {
                        LayerKind::Mask(mk) => Some((m, mk.inverted)),
                        _ => None,
                    });
                    out.push(ReliefStep::Filter {
                        id,
                        filter,
                        opacity: a.opacity.clamp(0.0, 1.0),
                        mask,
                    });
                }
                LayerKind::Mask(_) => {}
            }
        }
    }
}

/// As amostras de relevo de quem dobra (o 2D: píxeis; a peça: amostras da retícula).
pub trait ReliefSamples: Sync {
    /// Quantas amostras.
    fn len(&self) -> usize;
    /// Sem amostras?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// A camada tem relevo?
    fn has(&self, id: LayerId) -> bool;
    /// `(altura própria, cobertura)` da camada na amostra `i`.
    fn at(&self, id: LayerId, i: usize) -> (f32, f32);
    /// O valor `0..1` da máscara `id` por amostra (sem a inversão), se houver.
    fn mask(&self, id: LayerId) -> Option<Vec<f32>>;
}

/// Um âmbito aberto: a altura quando ele abriu e o corpo das camadas dele.
struct Ambito {
    antes: Option<Vec<f32>>,
    corpo: Vec<f32>,
}

/// ⭐⭐⭐ **A dobra através dos ajustes** — `(altura, corpo)` por amostra, ou `None` sem camada com
/// relevo. A altura é a da dobra de sempre; o corpo é o máximo das coberturas, e os dois passam pelos
/// ajustes do seu âmbito.
#[must_use]
pub fn fold_relief_through(
    plan: &[ReliefStep],
    src: &dyn ReliefSamples,
    nb: &dyn Neighbourhood,
) -> Option<(Vec<f32>, Vec<f32>)> {
    let n = src.len();
    let camadas: Vec<LayerId> = plan
        .iter()
        .filter_map(|s| match *s {
            ReliefStep::Layer { id, .. } if src.has(id) => Some(id),
            _ => None,
        })
        .collect();
    if camadas.is_empty() {
        return None;
    }
    let cover_max: Vec<f32> = (0..n)
        .into_par_iter()
        .map(|i| {
            camadas
                .iter()
                .map(|&id| src.at(id, i).1.clamp(0.0, 1.0))
                .fold(0.0f32, f32::max)
        })
        .collect();
    let mut h = vec![RELIEF_FOLD_SEED; n];
    let mut ambitos = vec![Ambito {
        antes: None,
        corpo: vec![0.0; n],
    }];
    for step in plan {
        match *step {
            ReliefStep::Layer {
                id,
                depth,
                composite,
            } => {
                if !src.has(id) {
                    continue;
                }
                let corpo = &mut ambitos.last_mut().expect("o âmbito de fora").corpo;
                h.par_iter_mut()
                    .zip(corpo.par_iter_mut())
                    .enumerate()
                    .for_each(|(i, (hh, b))| {
                        let (own, c) = src.at(id, i);
                        *hh = fold_relief_step(
                            *hh,
                            own,
                            depth,
                            composite,
                            c.clamp(0.0, 1.0),
                            cover_max[i],
                        );
                        *b = b.max(c);
                    });
            }
            ReliefStep::Open => ambitos.push(Ambito {
                antes: Some(h.clone()),
                corpo: vec![0.0; n],
            }),
            ReliefStep::Close => {
                if ambitos.len() > 1 {
                    let dentro = ambitos.pop().expect("aberto").corpo;
                    let fora = &mut ambitos.last_mut().expect("o de fora").corpo;
                    for (f, d) in fora.iter_mut().zip(dentro) {
                        *f = f.max(d);
                    }
                }
            }
            ReliefStep::Filter {
                filter,
                opacity,
                mask,
                ..
            } => {
                let amb = ambitos.last_mut().expect("o âmbito de fora");
                // O que ESTE âmbito fez ao relevo: no de fora é a altura inteira (sem subtracção, que
                // trocaria o sinal de um zero); num grupo, a diferença desde que ele abriu.
                let d0: Vec<f32> = match &amb.antes {
                    None => h.clone(),
                    Some(a) => h.iter().zip(a).map(|(x, y)| x - y).collect(),
                };
                let mut d = d0.clone();
                let mut b = amb.corpo.clone();
                aplica(filter, &mut d, &mut b, nb);
                let m = mask.and_then(|(id, inv)| {
                    src.mask(id).map(|v| {
                        v.into_iter()
                            .map(|x| if inv { 1.0 - x } else { x })
                            .collect::<Vec<_>>()
                    })
                });
                for i in 0..n {
                    let t = opacity * m.as_ref().map_or(1.0, |v| v[i].clamp(0.0, 1.0));
                    if t <= 0.0 {
                        continue;
                    }
                    h[i] += (d[i] - d0[i]) * t;
                    amb.corpo[i] += (b[i] - amb.corpo[i]) * t;
                }
            }
        }
    }
    while ambitos.len() > 1 {
        let dentro = ambitos.pop().expect("aberto").corpo;
        let fora = &mut ambitos.last_mut().expect("o de fora").corpo;
        for (f, d) in fora.iter_mut().zip(dentro) {
            *f = f.max(d);
        }
    }
    Some((h, ambitos.pop().expect("o de fora").corpo))
}

/// O filtro sobre a altura e o corpo — o MESMO passa-baixo da cor (`Neighbourhood::blur2`).
fn aplica(f: ReliefFilter, h: &mut [f32], corpo: &mut [f32], nb: &dyn Neighbourhood) {
    match f {
        ReliefFilter::Blur { radius } => nb.blur2(radius, h, corpo),
        ReliefFilter::Sharpen { radius, amount } => {
            let (bh, bc) = (h.to_vec(), corpo.to_vec());
            nb.blur2(radius, h, corpo);
            for (x, b) in h.iter_mut().zip(&bh) {
                *x = b + amount * (b - *x);
            }
            for (x, b) in corpo.iter_mut().zip(&bc) {
                *x = (b + amount * (b - *x)).clamp(0.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
#[path = "relief_through_tests.rs"]
mod tests;
