//! ⭐⭐ **O CONTACTO E A OCLUSÃO PRÓPRIA de cada objeto do Render** — do campo DELE (as unidades
//! dele, não o grupo), amostrado CRU num [`ph2d_contacto::Volume`] de [`LADO_VOLUME`]`³` sobre a
//! caixa da malha. Do mesmo volume saem a oclusão própria por vértice (os raios de
//! [`ph2d_contacto::visibilidade_propria`]) e a grelha do céu que ele tapa aos OUTROS objetos
//! ([`ph2d_contacto::Grade`]), lida pela placa com a pose de agora — mover não refaz nada.
//!
//! ⛔ A oclusão de Quilez (`5` passos ao longo da normal, no campo do GRUPO) saiu: medida contra o
//! Cycles errava `0,05–0,15` em média (`0,09–0,22` nos vincos) e contava as vizinhas do grupo — que
//! agora vêm da grelha delas, pela pose de AGORA (gates em `ph2d-contacto`; a sonda
//! `tests::sonda_oclusao_da_malha` mede-a nas malhas reais).

use ph2d_field::FieldDoc;
use ph2d_field_eval::hybrid::Registry;

/// As amostras por aresta do volume de um objeto.
pub const LADO_VOLUME: usize = 64;

/// ⭐ Assa a oclusão própria em `malha.ao` e devolve a grelha do contacto (`None` sem campo).
pub fn assa(
    doc: &FieldDoc,
    reg: &Registry,
    malha: &mut crate::malha_render_tri::MalhaPronta,
) -> Option<ph2d_contacto::Grade> {
    assa_com(doc, reg, malha, LADO_VOLUME)
}

/// [`assa`] com `lado` amostras por aresta do volume.
pub fn assa_com(
    doc: &FieldDoc,
    reg: &Registry,
    malha: &mut crate::malha_render_tri::MalhaPronta,
    lado: usize,
) -> Option<ph2d_contacto::Grade> {
    let p = &malha.posicoes;
    if p.is_empty() {
        return None;
    }
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for q in p {
        for e in 0..3 {
            lo[e] = lo[e].min(q[e]);
            hi[e] = hi[e].max(q[e]);
        }
    }
    // Dois passos de margem: a superfície fica dentro do volume com folga para a trilinear.
    let maior = (0..3)
        .map(|e| hi[e] - lo[e])
        .fold(0.0, f32::max)
        .max(1.0e-4);
    let m = 2.0 * maior / (lado - 1) as f32;
    let (lo, hi) = (lo.map(|c| c - m), hi.map(|c| c + m));
    let mut falhou = false;
    let vol = ph2d_contacto::Volume::de(lo, hi, lado, |pts| {
        ph2d_field_eval::par::valores(doc, reg, pts).unwrap_or_else(|_| {
            falhou = true;
            vec![1.0e3; pts.len()]
        })
    });
    if falhou {
        return None;
    }
    let centro: [f32; 3] = std::array::from_fn(|e| 0.5 * (lo[e] + hi[e]));
    let raio = p
        .iter()
        .map(|q| {
            (0..3)
                .map(|e| (q[e] - centro[e]).powi(2))
                .sum::<f32>()
                .sqrt()
        })
        .fold(0.0, f32::max);
    let diag = (0..3).map(|e| (hi[e] - lo[e]).powi(2)).sum::<f32>().sqrt();
    malha.ao = ph2d_contacto::visibilidade_propria(&vol, &malha.posicoes, &malha.normais, diag);
    Some(ph2d_contacto::Grade::constroi(&vol, centro, raio))
}

#[cfg(test)]
#[path = "malha_render_contacto_tests.rs"]
mod tests;
