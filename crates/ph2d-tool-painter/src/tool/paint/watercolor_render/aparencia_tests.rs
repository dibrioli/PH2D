//! B16.6: o tom linear pela tabela dá o bit da conta (`srgb_to_linear_unit`, o caminho de antes).

use super::super::super::watercolor_lut::luts;
use super::tom_linear;
use ph2d_color::srgb::srgb_to_linear_unit;

fn igual(v: f32) -> bool {
    tom_linear(v, luts()).to_bits() == srgb_to_linear_unit(v).to_bits()
}

#[test]
fn o_tom_linear_pela_tabela_da_o_bit_da_conta() {
    let lut = luts();
    // Os 256 bytes: o caminho da tabela, e com os mesmos bits.
    for k in 0..=255u8 {
        let v = f32::from(k) / 255.0;
        assert!(igual(v), "byte {k}");
        assert_eq!(tom_linear(v, lut).to_bits(), lut.s2l[k as usize].to_bits());
    }
    // A base sobre o chão com qualquer alfa (a orla), como o texel a monta, e tons quaisquer.
    let mut s = 0x9E37_79B9u32;
    let mut prox = || {
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        s
    };
    let mut fora = 0usize;
    for _ in 0..400_000 {
        let (b, c, a) = (prox() as u8, prox() as u8, prox() as u8);
        let ab = f32::from(a) / 255.0;
        let v = (f32::from(b) * ab + f32::from(c) * (1.0 - ab)) / 255.0;
        assert!(igual(v), "base {b} chão {c} alfa {a}");
        let w = (prox() >> 8) as f32 / (1u32 << 24) as f32;
        assert!(igual(w), "tom {w}");
        fora += usize::from((v * 255.0).fract() != 0.0);
    }
    assert!(fora > 100_000, "a orla tem de exercer a conta: {fora}");
    for v in [-0.0f32, -1e-7, 1.0 + f32::EPSILON, 2.0, f32::NAN, f32::INFINITY, 0.04045] {
        assert_eq!(
            tom_linear(v, lut).to_bits(),
            srgb_to_linear_unit(v).to_bits(),
            "{v}"
        );
    }
}
