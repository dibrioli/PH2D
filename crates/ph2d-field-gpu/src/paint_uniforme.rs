//! ⭐⭐⭐ **A ARRUMAÇÃO DO UNIFORME DO PINTOR** — os bytes que o `struct Pintor` do
//! [`crate::paint_wgsl`] lê, por esta ordem exacta.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, forçado pelo tecto de LOC** e melhor por isso: o irmão responde
//! *«que passes correm e por que ordem»* e isto responde *«que números é que eles leem»*. ⛔ Um
//! uniforme lido com a compensação errada **não estoura: PINTA** — e por isso a ordem daqui é a lei,
//! e um campo acrescentado ao MEIO move tudo o que vem depois.

use crate::paint::PaintSetup;

/// Os bytes do uniforme, e a contagem de bordas que a montagem também precisa.
pub(crate) fn arruma(
    pintor: &PaintSetup<'_>,
    bordas: u64,
    n_mats: u32,
    tem_foscas: u32,
) -> (Vec<u8>, u32) {
    let bg = pintor.background;
    let a = f32::from(bg[3]) / 255.0;
    let mut u: Vec<u8> = Vec::with_capacity(64 + crate::trace::MAX_LAMPS * 16);
    for f in [
        pintor.stops,
        pintor.pixel_world,
        pintor.curv_eps,
        // ⭐ O `w` do `knobs` era um slot morto e passa a ser o raio da PEÇA — ver
        // [`PaintSetup::piece_radius`].
        pintor.piece_radius,
        // ⚠️ **O fundo da BORDA é LINEAR e PRÉ-MULTIPLICADO** — a média das quatro amostras corre
        // em linear de ecrã, e o alfa entra nela como as outras três componentes.
        ph2d_color::srgb::srgb_to_linear_byte(bg[0]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[1]) * a,
        ph2d_color::srgb::srgb_to_linear_byte(bg[2]) * a,
        a,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    #[allow(clippy::cast_possible_truncation)]
    let n_bordas = bordas as u32;
    let empacotado = u32::from(bg[0])
        | (u32::from(bg[1]) << 8)
        | (u32::from(bg[2]) << 16)
        | (u32::from(bg[3]) << 24);
    // ⚠️ **`modo2.y` é a contagem de células do campo do chão**, e `0` ali quer dizer «campo
    // vazio»: é o que mantém o quadro sem esta wave byte a byte o de sempre.
    #[allow(clippy::cast_possible_truncation)]
    let n_chao = pintor.ground_bounce.n as u32;
    for v in [
        pintor.view,
        n_bordas,
        empacotado,
        n_mats,
        tem_foscas,
        n_chao,
        // ⭐⭐⭐ **O ESTILO LÊ A CURVATURA?** — `modo2.z`, e é ele que faz a tinta por aresta deixar
        // de ser um knob morto: sem esta bandeira o shader só perguntaria ao MATERIAL, e a grandeza
        // que o botão escolhe nunca seria medida (`ph2d_style::Style::reads_curvature`).
        u32::from(pintor.style.reads_curvature()),
        // ⭐⭐⭐ **O QUADRO GUARDA A CENA?** — `modo2.w`. Ver o binding 7 do `paint_wgsl`: a
        // bandeira é uniforme em todo o despacho, logo o ramo é de graça, e sem ela o caminho de
        // omissão pagaria uma escrita de `16 B` por pixel para lado nenhum.
        u32::from(pintor.bloom.contributes()),
    ] {
        u.extend_from_slice(&v.to_le_bytes());
    }
    for f in [
        pintor.ground_bounce.origin[0],
        pintor.ground_bounce.origin[1],
        pintor.ground_bounce.step,
        pintor.ground_bounce.height,
    ] {
        u.extend_from_slice(&f.to_le_bytes());
    }
    // ⚠️ **O array vai INTEIRO** — a mesma razão do `MarchSetup::lamps`: um `array<vec4, 8>` de
    // uniforme tem tamanho fixo.
    for r in pintor.lamp_radiance {
        for f in r {
            u.extend_from_slice(&f.to_le_bytes());
        }
        u.extend_from_slice(&0f32.to_le_bytes());
    }
    // ⭐⭐⭐ **A CAMADA DE ESTILO, no FIM do uniforme** — e a posição é deliberada: acrescentar um
    // campo ao meio moveria a compensação de tudo o que vem depois, e um uniforme lido com a
    // compensação errada não estoura, **pinta**.
    //
    // ⚠️ **O [`ph2d_style::wgsl::pack`] é a porta**: ele arruma E saneia, logo nenhum `NaN` de
    // painel chega ao dispositivo por alguém se ter esquecido de uma chamada.
    let mut bloco = ph2d_style::wgsl::pack(&pintor.style);
    // ⭐⭐⭐ **A posição que o `pack` deixa a zero de propósito** — ver
    // [`ph2d_style::wgsl::EPS_DO_ESTILO`]: o passo não é do `Style` (ele precisa do raio da PEÇA),
    // e é a montagem que o escreve.
    bloco[ph2d_style::wgsl::EPS_DO_ESTILO] = pintor.curv_eps_estilo;
    for f in bloco {
        u.extend_from_slice(&f.to_le_bytes());
    }
    (u, n_bordas)
}
