#![forbid(unsafe_code)]
//! `ph2d-flip-reshape` — **os pincéis de escultura de traço** do Flip (ADR-0114 W5),
//! clean-room do sculpt do Grease Pencil 5.2 (`docs/Flip/02 §7`).
//!
//! Remodelar um traço já desenhado: alisar o tremor, empurrar uma curva, agarrar e
//! arrastar um trecho, apertar, torcer, engrossar, apagar, bagunçar. Oito pincéis,
//! todos sobre a MESMA infra: **um raio, uma força, uma curva de queda**.
//!
//! ## As três decisões que definem a sensação (e que não se re-derivam)
//!
//! **1. A dose é por AMOSTRA de input, não por tempo.** Mover devagar aplica mais —
//! é assim que o GP "sente", e é o que dá controle fino. Um fork que gerasse amostras
//! por *timer* mudaria a sensação de TODOS os pincéis de uma vez (§7).
//!
//! **2. A máscara define O QUE; o traço define QUANTO.** O conjunto de traços que o
//! gesto pode tocar é **congelado no pen-down** ([`Session::begin`]): arrastar para
//! fora nunca recruta um traço novo no meio do gesto. Sem isso, o pincel "descobre"
//! geometria enquanto anda e o resultado depende do caminho do mouse.
//!
//! **3. Em 2D-ortográfico a projeção colapsa.** O GP esculpe em espaço de TELA e
//! converte o delta de volta ao objeto; aqui a câmera é uma similaridade (escala
//! uniforme), então distâncias e ângulos são os mesmos nos dois espaços e **tudo
//! roda em espaço LOCAL** do objeto. A única constante que carrega a unidade de
//! pixel é a amplitude do [`ReshapeKind::Randomize`] — e ela é convertida
//! explicitamente ([`ReshapeParams::px_to_local`]).
//!
//! HR-5: zero transcendentais (o `sqrt` é exato em IEEE-754; a rotação do Twist usa
//! Taylor — ver `brushes::rotate_small`).

pub mod blur;
mod brushes;

pub use blur::{Ends, binomial, binomial_uniform};

use ph2d_core::Vec2;
use ph2d_flip::FlipStroke;

/// Os pincéis. (**Clone é um COMANDO, não um pincel** — os modos contínuos do GPv2
/// são admitidamente quebrados e foram removidos lá; aqui ele nasce como copiar/colar
/// de traços, fora desta crate.)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum ReshapeKind {
    /// Alisa o traço (kernel binomial; a influência é o peso de mistura).
    #[default]
    Smooth,
    /// Empurra os pontos na direção do movimento do cursor.
    Push,
    /// **Agarra** um trecho e o carrega: a máscara E os pesos são congelados no
    /// pen-down, e o conjunto agarrado nunca é reavaliado.
    Grab,
    /// Aperta os pontos em direção ao cursor (invertido: infla).
    Pinch,
    /// Torce rigidamente ao redor do cursor.
    Twist,
    /// Engrossa a linha (invertido: afina).
    Thickness,
    /// Aumenta a opacidade (invertido: apaga aos poucos).
    Strength,
    /// Bagunça a posição, perpendicular ao movimento do cursor.
    Randomize,
}

impl ReshapeKind {
    /// Todos, na ordem em que o painel os mostra.
    pub const ALL: [ReshapeKind; 8] = [
        ReshapeKind::Smooth,
        ReshapeKind::Push,
        ReshapeKind::Grab,
        ReshapeKind::Pinch,
        ReshapeKind::Twist,
        ReshapeKind::Thickness,
        ReshapeKind::Strength,
        ReshapeKind::Randomize,
    ];

    /// O rótulo do painel (inglês — a UI do app é inglês, sempre).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ReshapeKind::Smooth => "Smooth",
            ReshapeKind::Push => "Push",
            ReshapeKind::Grab => "Grab",
            ReshapeKind::Pinch => "Pinch",
            ReshapeKind::Twist => "Twist",
            ReshapeKind::Thickness => "Thickness",
            ReshapeKind::Strength => "Strength",
            ReshapeKind::Randomize => "Randomize",
        }
    }

    /// O pincel tem direção (o invert, via Ctrl, faz o OPOSTO)?
    ///
    /// Nos que não têm, o Ctrl é inerte — e isso é honesto, não um esquecimento:
    /// "alisar ao contrário" ou "empurrar ao contrário do cursor" não significam
    /// nada (empurrar para trás é só mover o cursor para trás).
    #[must_use]
    pub fn has_direction(self) -> bool {
        matches!(
            self,
            ReshapeKind::Pinch
                | ReshapeKind::Twist
                | ReshapeKind::Thickness
                | ReshapeKind::Strength
        )
    }
}

/// O que o pincel usa, por amostra.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct InputSample {
    /// O cursor, em espaço **LOCAL** do objeto.
    pub pos: Vec2,
    /// O quanto o cursor andou desde a amostra anterior (local). Zero no pen-down.
    pub delta: Vec2,
    /// Pressão da caneta `0..=1` (o mouse manda `1.0`).
    pub pressure: f32,
}

/// Os parâmetros do pincel (o que o painel expõe + o que o shell converte).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ReshapeParams {
    pub kind: ReshapeKind,
    /// Raio do pincel, em unidades **LOCAIS** (o shell converte de px de tela).
    pub radius: f32,
    /// Força `0..=1` — o `alpha` do brush do GP.
    pub strength: f32,
    /// Ctrl: faz o oposto (só nos pincéis com [`ReshapeKind::has_direction`]).
    pub invert: bool,
    /// **Falloff multiframe** — o multiplicador da influência quando o MESMO gesto
    /// esculpe vários quadros de uma vez (o quadro ativo vale `1.0`, os vizinhos
    /// caem com a distância temporal). Hoje o shell manda sempre `1.0` (a tira
    /// seleciona um quadro); ele está na assinatura desde o início **de propósito** —
    /// retrofitá-lo depois exigiria tocar os oito pincéis (§7, T5.7).
    pub frame_falloff: f32,
    /// Unidades locais por pixel de tela. Só o [`ReshapeKind::Randomize`] precisa
    /// (a amplitude dele é em PIXELS no GP; ver `brushes::randomize`).
    pub px_to_local: f32,
}

impl Default for ReshapeParams {
    fn default() -> Self {
        Self {
            kind: ReshapeKind::default(),
            radius: 1.0,
            strength: 0.5,
            invert: false,
            frame_falloff: 1.0,
            px_to_local: 1.0,
        }
    }
}

/// **A influência do pincel num ponto** — o funil por onde os oito passam
/// (`brush_point_influence`, `paint_common.cc:98`).
///
/// ```text
/// influência = força · pressão · falloff_multiframe · queda(distância / raio)
/// ```
///
/// A `queda` é o **smoothstep** em `p = 1 - d/r`: vale 1 no centro do pincel, 0 na
/// borda, e tem derivada **zero nas duas pontas** — é por isso que a marca do pincel
/// não tem degrau nem no meio nem na borda. Polinomial (HR-5).
#[must_use]
pub fn influence(p: &ReshapeParams, s: &InputSample, point: Vec2) -> f32 {
    let r = p.radius;
    if r <= 0.0 {
        return 0.0;
    }
    let d = point - s.pos;
    let dist = (d.x * d.x + d.y * d.y).sqrt();
    if dist >= r {
        return 0.0;
    }
    let t = 1.0 - dist / r; // 1 no centro, 0 na borda
    let falloff = t * t * (3.0 - 2.0 * t); // smoothstep
    p.strength.clamp(0.0, 1.0) * s.pressure.clamp(0.0, 1.0) * p.frame_falloff * falloff
}

/// Um gesto de escultura em curso.
///
/// Nasce no pen-down ([`Session::begin`]) — que é onde a **máscara congela** — e
/// recebe uma [`Session::apply`] por amostra de input.
pub struct Session {
    /// Os traços que este gesto pode tocar (índices no desenho), congelados no down.
    mask: Vec<usize>,
    /// **Grab**: `(traço, anel, ponto, peso)` congelados no down, com `pressure = 1.0`
    /// (o GP fixa a pressão aqui — `sculpt_grab.cc:188`). O conjunto agarrado nunca é
    /// reavaliado: é o que faz o Grab *carregar* um trecho em vez de recrutar pontos
    /// novos a cada milímetro. O `anel` é `None` para o contorno e `Some(k)` para o
    /// k-ésimo buraco de uma região.
    grab: Vec<(usize, Option<usize>, usize, f32)>,
    /// Contador de amostras — a semente do Randomize é **re-semeada por amostra**
    /// (parado, o pincel faz um passeio browniano; é assim no GP).
    sample_no: u64,
}

/// Uma **REGIÃO**: um preenchimento (com seus buracos) ou um fechamento de gap. O
/// contorno dela não é line-art.
///
/// **A escultura MOVE as regiões** — e isso não é um detalhe, é a diferença entre "a
/// cor acompanha a linha" e "a cor fica para trás" (smoke do Enio 2026-07-13, com o
/// Suzanne do Blender ao lado). No Grease Pencil o sculpt edita **todas** as curvas
/// (`retrieve_editable_strokes`: a única exclusão é material travado), e o
/// preenchimento é a triangulação dos pontos da própria curva — então mexer nos pontos
/// re-tria o fill no mesmo frame. É por isso que lá "line e fill parecem um só".
///
/// O que NÃO se esculpe numa região são os atributos: o `width` do contorno de um fill
/// é a **dilatação da cor por baixo da linha** (BUGS #15), não a espessura de um traço
/// — engrossá-lo com o Thicken empurraria a cor para fora do desenho. Idem a opacidade.
/// (A borracha continua não mordendo regiões: lá o critério é outro — ela remove tinta,
/// e uma região não tem nenhuma.)
fn is_region(s: &FlipStroke) -> bool {
    s.hide_stroke
}

/// O pincel mexe em ATRIBUTOS (largura, opacidade) em vez de posição?
fn edits_attributes(kind: ReshapeKind) -> bool {
    matches!(kind, ReshapeKind::Thickness | ReshapeKind::Strength)
}

impl Session {
    /// Pen-down: congela a máscara (e, no Grab, os pesos).
    ///
    /// **A máscara é "tudo o que tem geometria no desenho ativo"** — line-art E regiões
    /// (é o que o GP faz: `retrieve_editable_strokes` só exclui material travado). Um
    /// pincel que movesse a linha e deixasse a cor para trás não seria uma ferramenta de
    /// escultura; seria uma ferramenta de quebrar o desenho.
    ///
    /// O auto-masking mais fino do GP (por traço sob o cursor, por material, pela
    /// seleção) depende de um modelo de SELEÇÃO, que é o Edit Mode — o pacote seguinte.
    /// O que importa já vale: o conjunto é resolvido UMA vez, no down.
    #[must_use]
    pub fn begin(strokes: &[FlipStroke], p: &ReshapeParams, s: &InputSample) -> Self {
        let mask: Vec<usize> = strokes
            .iter()
            .enumerate()
            .filter(|(_, st)| st.len() >= 2)
            // Os pincéis de ATRIBUTO (largura/opacidade) não tocam regiões: ali o
            // `width` é a dilatação da cor, não a espessura de uma linha.
            .filter(|(_, st)| !(edits_attributes(p.kind) && is_region(st)))
            .map(|(i, _)| i)
            .collect();
        let mut grab = Vec::new();
        if p.kind == ReshapeKind::Grab {
            // Pressão FIXA em 1.0 no congelamento (o GP faz isso): o peso de cada
            // ponto agarrado não pode depender de quão forte a caneta estava
            // apertada no instante do toque.
            let frozen = InputSample {
                pressure: 1.0,
                ..*s
            };
            for &si in &mask {
                let st = &strokes[si];
                // `ring = None` é o contorno; `Some(k)` é o k-ésimo buraco (o "O" tem
                // um — e se o buraco não fosse agarrado junto, ele ficaria para trás e a
                // rosquinha viraria uma mancha).
                for (pi, &pos) in st.positions().iter().enumerate() {
                    let w = influence(p, &frozen, pos);
                    if w > 0.0 {
                        grab.push((si, None, pi, w));
                    }
                }
                for (k, hole) in st.holes.iter().enumerate() {
                    for (pi, &pos) in hole.iter().enumerate() {
                        let w = influence(p, &frozen, pos);
                        if w > 0.0 {
                            grab.push((si, Some(k), pi, w));
                        }
                    }
                }
            }
        }
        Self {
            mask,
            grab,
            sample_no: 0,
        }
    }

    /// Uma amostra do gesto. Devolve `true` se o documento mudou.
    pub fn apply(
        &mut self,
        strokes: &mut [FlipStroke],
        p: &ReshapeParams,
        s: &InputSample,
    ) -> bool {
        self.sample_no += 1;
        if p.kind == ReshapeKind::Grab {
            return brushes::grab(strokes, &self.grab, s.delta);
        }
        let n = self.sample_no;
        let mut changed = false;
        for &si in &self.mask {
            let Some(st) = strokes.get_mut(si) else {
                continue;
            };
            changed |= match p.kind {
                ReshapeKind::Thickness => brushes::thickness(st, p, s),
                ReshapeKind::Strength => brushes::strength(st, p, s),
                // Os pincéis de POSIÇÃO valem para o contorno **e para os buracos**: um
                // buraco que ficasse para trás abriria a rosquinha.
                _ => {
                    let closed = st.closed;
                    let mut hit = brushes::position(st.positions_mut(), p, s, closed, n);
                    for hole in &mut st.holes {
                        hit |= brushes::position(hole, p, s, true, n);
                    }
                    hit
                }
            };
        }
        changed
    }

    /// Quantos pontos o Grab agarrou (diagnóstico + testes).
    #[must_use]
    pub fn grabbed(&self) -> usize {
        self.grab.len()
    }
}

#[cfg(test)]
mod tests;
