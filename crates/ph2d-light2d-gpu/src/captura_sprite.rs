//! **C0 dos SPRITES por texel** (W3) — a geometria e o material de um sprite vêm da TEXTURA dele: o
//! alfa de cada texel é a cobertura, a cor × o multiplicador é a emissão do dono, a opacidade do
//! `BloqueiaLuz` é a transmissão de dentro. Os MESMOS três alvos e a MESMA lei do dono das formas
//! (`c0_dono.wgsl`); o quad que a `CenaDeLuz` tem para o sprite (o que o gabarito vê) sai da C0.

use std::collections::BTreeSet;
use std::ops::Range;

use bytemuck::{Pod, Zeroable};
use ph2d_light2d_ecs::Rgb;

use crate::captura::{Quem, caixa};
use crate::config::Grade;

/// Um sprite na C0, em coordenadas do MUNDO (o solver leva-o à grade de cada quadro).
#[derive(Clone, Debug, PartialEq)]
pub struct SpriteNaC0 {
    /// `mundo(s, t) = origem + s·eixo_s + t·eixo_t`, com o quad em `[0, 1]²` e `(0, 0)` no canto de
    /// cima à esquerda (o `quad_uv` do sprite).
    pub origem: [f64; 2],
    pub eixo_s: [f64; 2],
    pub eixo_t: [f64; 2],
    /// O UV em `(s, t) = (0, 0)` e `(1, 1)` — as inversões já aplicadas.
    pub uv: [f32; 4],
    /// O multiplicador da cor do texel na emissão (zero = não emite).
    pub emissao: Rgb,
    /// 1 num emissor (opaco); a do `BloqueiaLuz` num oclusor.
    pub opacidade: f32,
    /// A textura já guarda a cor pré-multiplicada pelo alfa.
    pub premultiplicada: bool,
    /// Índice em [`CapturaDeSprites::texturas`].
    pub textura: usize,
    /// ⭐ W5c: o multiplicador da cor do texel no ALBEDO do rebote (zero = não reflete).
    pub albedo: Rgb,
    /// ⭐ W5c: a textura guarda a cor em sRGB (a camada do vetor do Vello); a de um sprite já é lida
    /// linear.
    pub cor_em_srgb: bool,
}

/// Como a C0 lê uma textura nas 8×8 subamostras de cada texel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Amostragem {
    /// Bilinear — a textura de um sprite, em qualquer escala.
    Bilinear,
    /// O texel EXATO (sem filtro) — uma textura alinhada à grade cujo píxel já É a cobertura de
    /// área (a camada do vetor que o Vello desenha); filtrá-la borraria a silhueta.
    Exata,
}

/// Os sprites do quadro para a C0, com as texturas que eles amostram.
#[derive(Default)]
pub struct CapturaDeSprites<'a> {
    pub texturas: Vec<(&'a wgpu::TextureView, Amostragem)>,
    pub sprites: Vec<SpriteNaC0>,
    /// As formas da `CenaDeLuz` que estes sprites SUBSTITUEM (o quad deles).
    pub substituidos: BTreeSet<Quem>,
    /// ⭐ W5b (F4): os que só existem FORA da grade, para a luz direta (um caminho vetorial que tapa,
    /// sobre o desenho do Vello na moldura dele) — nenhuma captura os desenha.
    pub distantes: Vec<SpriteNaC0>,
}

/// Um sprite como o `c0_sprite.wgsl` o lê (112 bytes).
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct FormaSprite {
    pub caixa: [f32; 4],
    pub s: [f32; 4],
    pub t: [f32; 4],
    pub uv: [f32; 4],
    pub cor: [f32; 4],
    pub opacidade: f32,
    pub premultiplicada: f32,
    pub srgb: f32,
    /// ⭐ W5d: o alfa da textura já é ÁREA (lida exata, a camada do vetor) — a cobertura é a média dele.
    pub alfa_e_area: f32,
    pub albedo: [f32; 4],
}

/// Os sprites na grade `g`, AGRUPADOS por textura (um desenho por lote): `(formas, lotes)`, com cada
/// lote `(textura, intervalo em formas)`. Um sprite degenerado ou fora da grade não entra.
pub(crate) fn na_grade(
    c: &CapturaDeSprites<'_>,
    g: &Grade,
) -> (Vec<FormaSprite>, Vec<(usize, Range<u32>)>) {
    let mut ordem: Vec<&SpriteNaC0> = c.sprites.iter().collect();
    ordem.sort_by_key(|s| s.textura);
    let em_texels = |p: [f64; 2]| {
        [
            (p[0] - g.origem_m[0]) / g.texel_m,
            (p[1] - g.origem_m[1]) / g.texel_m,
        ]
    };
    let mut formas = Vec::with_capacity(ordem.len());
    let mut lotes: Vec<(usize, Range<u32>)> = Vec::new();
    for s in ordem {
        // texel(s, t) = o + s·a + t·b
        let o = em_texels(s.origem);
        let a = [s.eixo_s[0] / g.texel_m, s.eixo_s[1] / g.texel_m];
        let b = [s.eixo_t[0] / g.texel_m, s.eixo_t[1] / g.texel_m];
        let det = a[0] * b[1] - a[1] * b[0];
        if !(det.is_finite() && det.abs() > 1e-12) {
            continue;
        }
        let cantos = [
            o,
            [o[0] + a[0], o[1] + a[1]],
            [o[0] + b[0], o[1] + b[1]],
            [o[0] + a[0] + b[0], o[1] + a[1] + b[1]],
        ];
        let min = cantos
            .iter()
            .fold([f64::INFINITY; 2], |m, p| [m[0].min(p[0]), m[1].min(p[1])]);
        let max = cantos.iter().fold([f64::NEG_INFINITY; 2], |m, p| {
            [m[0].max(p[0]), m[1].max(p[1])]
        });
        let Some(cx) = caixa(min, max, g) else {
            continue;
        };
        // (s, t) = M⁻¹ (p − o), M = [a b] em colunas.
        let s_ = [b[1] / det, -b[0] / det];
        let t_ = [-a[1] / det, a[0] / det];
        let lin = |r: [f64; 2]| {
            [
                r[0] as f32,
                r[1] as f32,
                (-(r[0] * o[0] + r[1] * o[1])) as f32,
                0.0,
            ]
        };
        let [er, eg, eb] = s.emissao;
        let i = u32::try_from(formas.len()).expect("sprites cabem em u32");
        formas.push(FormaSprite {
            caixa: cx,
            s: lin(s_),
            t: lin(t_),
            uv: s.uv,
            cor: [er, eg, eb, 0.0],
            opacidade: s.opacidade,
            premultiplicada: if s.premultiplicada { 1.0 } else { 0.0 },
            srgb: if s.cor_em_srgb { 1.0 } else { 0.0 },
            alfa_e_area: match c.texturas.get(s.textura).map(|t| t.1) {
                Some(Amostragem::Exata) => 1.0,
                _ => 0.0,
            },
            albedo: [s.albedo[0], s.albedo[1], s.albedo[2], 0.0],
        });
        match lotes.last_mut() {
            Some((t, r)) if *t == s.textura => r.end = i + 1,
            _ => lotes.push((s.textura, i..i + 1)),
        }
    }
    (formas, lotes)
}
