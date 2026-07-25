//! **O catálogo ÚNICO dos estilos de warp** (Enio 2026-07-25) — o *Effect > Warp* do Illustrator.
//!
//! Os mesmos nove estilos vivem em DOIS lugares do editor, e antes deste catálogo eram duas listas
//! que divergiram (o "Wave" do efeito era o "Flag" do Envelope; um tinha Fisheye/Rise, o outro
//! ArcUpper/ArcLower/Flag/Squeeze). Agora há **um** enum, e os dois consumidores o re-exportam:
//!
//! - o **efeito Warp** (`ph2d-vec-scene::fx_warp_presets`) usa [`WarpStyle::deform`] — o estilo como
//!   um campo `R² → R²` sobre a posição normalizada `[-1, 1]²`;
//! - os **presets do Envelope** (`ph2d-ecs::vec_envelope`) usam [`WarpStyle::cage`] — o MESMO estilo
//!   como uma gaiola (barrigas de lado + deslocamento de canto).
//!
//! Como são o MESMO tipo, não podem mais divergir — e um gate de paridade no shell prova que as
//! duas seções oferecem a lista idêntica.
//!
//! # Duas representações do mesmo estilo
//!
//! Um estilo é uma ideia (*"arqueia"*, *"abaúla"*); cada seção a realiza no seu meio. A maioria
//! (Arc/ArcUpper/ArcLower/Bulge/Flag/Wave/Squeeze) mapeia limpo nos dois. **Fisheye** é uma lente
//! RADIAL do interior no campo, e uma gaiola de 4 cantos só a aproxima (todos os lados incham — o
//! *Inflate*). **Rise** é um CISALHAMENTO: no campo é `v += b·u`, e na gaiola precisa **mover os
//! cantos de cima** — por isso o [`CageSpec`] carrega `shift` além de `bows` (o preset do Envelope
//! deixou de "nunca mover um canto"; a garantia de não-dobra sobrevive porque um shear puro é um
//! paralelogramo, sempre convexo).
//!
//! # A ORDEM é contrato de save
//!
//! ⚠️ Os sete primeiros variants estão na ordem exata do antigo `EnvelopeWarp` (`Arc, ArcUpper,
//! ArcLower, Bulge, Flag, Wave, Squeeze`), porque o `VecEnvelope.warp` é serializado em postcard
//! (posicional) e já viaja em projetos salvos. Fisheye e Rise são **apendados**. Mover um variant
//! relê saves antigos como o estilo errado, em silêncio.

use serde::{Deserialize, Serialize};

/// Abaixo desta dobra o estilo é o ponto neutro (a identidade).
const EPS: f64 = 1e-12;

/// **Os nove estilos de warp.** Ver o cabeçalho do módulo para a ordem (contrato de save).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarpStyle {
    /// O banana: o meio sobe/desce em relação às pontas.
    Arc,
    /// Só a borda de CIMA arqueia.
    ArcUpper,
    /// Só a borda de BAIXO arqueia.
    ArcLower,
    /// Almofada: os quatro lados abaúlam de uma vez.
    Bulge,
    /// Onda com os dois lados EM FASE — a forma inteira ondula (o antigo "Wave" do efeito).
    Flag,
    /// Onda com os dois lados em CONTRAFASE — a forma aperta e alarga.
    Wave,
    /// A cintura afina: os lados verticais entram.
    Squeeze,
    /// Lente esférica: o centro incha, a borda encolhe (na gaiola, uma aproximação — *Inflate*).
    Fisheye,
    /// Inclina a forma da esquerda para a direita — o cisalhamento.
    Rise,
}

/// **A gaiola de um estilo**, no quadrado unitário — o que o preset do Envelope consome.
///
/// `bows`: a barriga de cada lado (`[BL→BR, BR→TR, TR→TL, TL→BL]`, 2 controles cada) ao longo da
/// normal EXTERNA, em ±1. `shift`: o deslocamento de cada CANTO (`[BL, BR, TR, TL]`) em ±1 — zero
/// para todos os estilos menos o Rise, cujo shear move as duas quinas de cima.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CageSpec {
    pub bows: [[f64; 2]; 4],
    pub shift: [[f64; 2]; 4],
}

const O: [f64; 2] = [0.0, 0.0];

impl WarpStyle {
    /// Todos os estilos, na ordem do menu (e do contrato de save).
    pub const ALL: &'static [WarpStyle] = &[
        WarpStyle::Arc,
        WarpStyle::ArcUpper,
        WarpStyle::ArcLower,
        WarpStyle::Bulge,
        WarpStyle::Flag,
        WarpStyle::Wave,
        WarpStyle::Squeeze,
        WarpStyle::Fisheye,
        WarpStyle::Rise,
    ];

    /// O rótulo que a UI mostra (inglês, como todo label destes painéis).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            WarpStyle::Arc => "Arc",
            WarpStyle::ArcUpper => "Arc Upper",
            WarpStyle::ArcLower => "Arc Lower",
            WarpStyle::Bulge => "Bulge",
            WarpStyle::Flag => "Flag",
            WarpStyle::Wave => "Wave",
            WarpStyle::Squeeze => "Squeeze",
            WarpStyle::Fisheye => "Fisheye",
            WarpStyle::Rise => "Rise",
        }
    }

    /// **O estilo como CAMPO** — o deslocamento em espaço normalizado `[-1, 1]²`, `b` = dobra em
    /// `[-1, 1]`. É o que o efeito Warp aplica a cada ponto reamostrado.
    ///
    /// Toda fórmula reduz à identidade em `b == 0` (o neutro no-op da pilha de efeitos).
    #[must_use]
    pub fn deform(self, u: f64, v: f64, b: f64) -> (f64, f64) {
        use std::f64::consts::{PI, SQRT_2};
        match self {
            // O meio (`u≈0`) levanta `b`; as pontas (`|u|≈1`) ficam.
            WarpStyle::Arc => (u, b.mul_add(1.0 - u * u, v)),
            // O arco pesado só na metade de CIMA (`(v+1)/2`) / de BAIXO (`(1-v)/2`).
            WarpStyle::ArcUpper => (u, v + b * (1.0 - u * u) * (v + 1.0) * 0.5),
            WarpStyle::ArcLower => (u, v + b * (1.0 - u * u) * (1.0 - v) * 0.5),
            // Cada eixo escala pelo quão central é o outro ⇒ os quatro lados abaúlam juntos.
            WarpStyle::Bulge => (u * (1.0 + b * (1.0 - v * v)), v * (1.0 + b * (1.0 - u * u))),
            // EM FASE: a coluna inteira sobe por `b·sin(π·u)` (o antigo "Wave").
            WarpStyle::Flag => (u, b.mul_add((PI * u).sin(), v)),
            // CONTRAFASE: a espessura oscila — cima e baixo vão a lados opostos.
            WarpStyle::Wave => (u, v * (1.0 + b * (PI * u).sin())),
            // A cintura (`v≈0`) afina; as bordas (`|v|≈1`) ficam.
            WarpStyle::Squeeze => (u * (1.0 - b * (1.0 - v * v)), v),
            // Escala radial que decai com o raio, normalizado pelo canto (`√2`).
            WarpStyle::Fisheye => {
                let r = (u * u + v * v).sqrt();
                let s = 1.0 + b * (1.0 - (r / SQRT_2).min(1.0));
                (u * s, v * s)
            }
            // Cisalhamento vertical linear em x.
            WarpStyle::Rise => (u, b.mul_add(u, v)),
        }
    }

    /// **O estilo como GAIOLA** — as barrigas de lado + o shear de canto que o preset do Envelope
    /// carimba (escaladas pela dobra e pela amplitude MEDIDA do lado do Envelope).
    #[must_use]
    pub fn cage(self) -> CageSpec {
        let bows = match self {
            // baixo para fora + cima para dentro: o meio inteiro afunda (o banana).
            WarpStyle::Arc => [[1.0, 1.0], O, [-1.0, -1.0], O],
            WarpStyle::ArcUpper => [O, O, [1.0, 1.0], O],
            WarpStyle::ArcLower => [[1.0, 1.0], O, O, O],
            WarpStyle::Bulge => [[1.0, 1.0], O, [1.0, 1.0], O],
            WarpStyle::Flag => [[-1.0, 1.0], O, [-1.0, 1.0], O],
            WarpStyle::Wave => [[-1.0, 1.0], O, [1.0, -1.0], O],
            WarpStyle::Squeeze => [O, [-1.0, -1.0], O, [-1.0, -1.0]],
            // TODOS os lados para fora — a barrica que aproxima a lente (o Inflate).
            WarpStyle::Fisheye => [[1.0, 1.0], [1.0, 1.0], [1.0, 1.0], [1.0, 1.0]],
            // O shear não enverga lado nenhum — ele mora no `shift`.
            WarpStyle::Rise => [O, O, O, O],
        };
        // Só o Rise move canto: as duas quinas de CIMA (`TR`, `TL`, índices 2 e 3) para a direita.
        let shift = if matches!(self, WarpStyle::Rise) {
            [O, O, [1.0, 0.0], [1.0, 0.0]]
        } else {
            [O; 4]
        };
        CageSpec { bows, shift }
    }

    /// Um estilo tem shear (move canto)? Só o Rise — o Envelope pergunta isto para saber se a
    /// garantia de não-dobra de barriga (que só vale com cantos fixos) precisa da via do shear.
    #[must_use]
    pub fn shears(self) -> bool {
        self.cage().shift.iter().any(|s| s != &O)
    }
}

/// Sem dobra não há deformação — o neutro tem de ser identidade (o no-op da pilha de efeitos).
#[must_use]
pub fn is_neutral(bend: f64) -> bool {
    bend.abs() <= EPS
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
