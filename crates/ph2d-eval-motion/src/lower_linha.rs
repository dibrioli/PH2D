//! **O que se lê de UMA linha** — os acessores de coluna e as duas leis de mistura por linha,
//! partidos do `lower.rs` no tecto de LOC (doc 118 §9) ao longo da costura que já lá estava: aqui
//! não há corrente nem sink, só *«o que diz a linha `i` desta coluna?»*.

use crate::{Column, RenderInstance};

/// O `flip_uv` de UMA linha: a coluna `blend` quando ela existe e diz alguma coisa, senão o
/// do sink (`fallback`).
///
/// ⚠️ **`0` na coluna quer dizer *"o do sink"*, não `Normal`** — ver a nota no chamador. E o
/// número é arredondado e limitado pelo mesmo teto que o `sink_blend_tag` usa (o array de
/// pipelines do renderer), porque um valor fora da faixa vindo de um `value.*` qualquer não
/// pode escolher um pipeline que não existe.
///
/// ⚠️ **A leitura é o [`degrau_de_mistura`]**, a MESMA que a rota vectorial usa (doc 118 §9): as
/// duas médias de uma linha não podem ler a coluna de duas maneiras.
#[must_use]
pub(super) fn blend_at(col: Option<&Column>, i: usize, fallback: u32) -> u32 {
    match degrau_de_mistura(col, i) {
        0 => fallback,
        d => RenderInstance::pack_blend_bits(d - 1),
    }
}

/// ⭐⭐ **O DEGRAU de mistura de UMA linha** (doc 118 §9 W8) — a escada da coluna `blend` (`0` = o
/// do sink, `m + 1` = o modo `m`), arredondada e limitada ao array de pipelines do renderer; um valor
/// não-finito ou `< 0,5` é o do sink. ⚠️ Devolve o DEGRAU e não o tag: guardar o tag cru faria o `0`
/// (Mix) de uma linha rebaixar o modo do sink em silêncio — a identidade de junção que o
/// `motion.trail` já pagou.
#[must_use]
pub(super) fn degrau_de_mistura(col: Option<&Column>, i: usize) -> u8 {
    let v = scalar_at(col, i, 0.0);
    if !v.is_finite() || v < 0.5 {
        return 0;
    }
    let top = ph2d_render::pipeline::BLEND_PIPELINE_COUNT as f32;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "clampado a 1..=BLEND_PIPELINE_COUNT antes do cast"
    )]
    let degrau = v.round().clamp(1.0, top) as u8;
    degrau
}

/// O pedaço `cell = [escala_u, escala_v, desloc_u, desloc_v]` do rectângulo `uv = [u0, v0, u1, v1]`
/// — a MESMA conta que o `sprite.wgsl` faz por fragmento (`mix(uv.xy, uv.zw, local·escala + desloc)`),
/// feita uma vez nos dois cantos.
#[must_use]
pub fn uv_do_pedaco(uv: [f32; 4], cell: [f32; 4]) -> [f32; 4] {
    // ⚠️ A identidade devolve o rectângulo AO BIT: `u0 + (u1 − u0)` pode errar um ulp, e o recorte
    // arredonda para píxeis com `floor`/`ceil` — um ulp pode ganhar uma coluna.
    if cell == RenderInstance::IDENTITY_UV_XFORM {
        return uv;
    }
    let (w, h) = (uv[2] - uv[0], uv[3] - uv[1]);
    [
        uv[0] + w * cell[2],
        uv[1] + h * cell[3],
        uv[0] + w * (cell[2] + cell[0]),
        uv[1] + h * (cell[3] + cell[1]),
    ]
}

pub(super) fn scalar_at(c: Option<&Column>, i: usize, default: f32) -> f32 {
    match c {
        Some(Column::Scalar(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
pub(super) fn vec2_at(c: Option<&Column>, i: usize, default: [f32; 2]) -> [f32; 2] {
    match c {
        Some(Column::Vec2(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}
pub(super) fn vec4_at(c: Option<&Column>, i: usize, default: [f32; 4]) -> [f32; 4] {
    match c {
        Some(Column::Vec4(v)) => v.get(i).copied().unwrap_or(default),
        _ => default,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A escada da coluna `blend`, lida UMA vez para as duas médias** (doc 118 §9 W8).
    ///
    /// ⚠️ A metade que importa é a última: o [`blend_at`] das sprites É o degrau empacotado — as duas
    /// rotas não podem ler a coluna de duas maneiras.
    #[test]
    fn o_degrau_da_linha_e_o_mesmo_nas_duas_medias() {
        let col = Column::Scalar(vec![0.0, 0.4, 1.0, 4.0, 3.6, 99.0, -2.0, f32::NAN]);
        let top = u8::try_from(ph2d_render::pipeline::BLEND_PIPELINE_COUNT).expect("cabe");
        let esperado = [0, 0, 1, 4, 4, top, 0, 0];
        for (i, e) in esperado.iter().enumerate() {
            assert_eq!(degrau_de_mistura(Some(&col), i), *e, "linha {i}");
            let sprite = blend_at(Some(&col), i, 0xDEAD);
            if *e == 0 {
                assert_eq!(sprite, 0xDEAD, "linha {i}: degrau 0 e' o do sink");
            } else {
                assert_eq!(sprite, RenderInstance::pack_blend_bits(e - 1), "linha {i}");
            }
        }
        assert_eq!(degrau_de_mistura(None, 0), 0, "sem coluna, o do sink");
    }

    /// **O lowering vectorial leva o degrau de cada linha** — e sem coluna, `0` ao bit.
    #[test]
    fn a_linha_vectorial_leva_o_proprio_modo() {
        let s = crate::Stream::new(2)
            .with("geometry_id", Column::Scalar(vec![3.0, 3.0]))
            .with("blend", Column::Scalar(vec![0.0, 4.0]));
        let mut out = Vec::new();
        super::super::lower_to_vector_instances_onto(&s, ph2d_render::SinkStyle::PLAIN, &mut out);
        assert_eq!(
            out.iter().map(|v| v.blend_linha).collect::<Vec<_>>(),
            [0, 4]
        );
        let sem = crate::Stream::new(1).with("geometry_id", Column::Scalar(vec![3.0]));
        out.clear();
        super::super::lower_to_vector_instances_onto(&sem, ph2d_render::SinkStyle::PLAIN, &mut out);
        assert_eq!(out[0].blend_linha, 0, "CONTROLE: sem coluna, o do sink");
    }
}
