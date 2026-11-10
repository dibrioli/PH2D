//! Gates da banda de rampa (P13 L) sem placa: cada texel de rampa leva `(rampa, degrau)`, o resto
//! tapa com `a = 254`, e sem «Light: Palette» nada é alocado.

use super::rampa::{FORA_DA_RAMPA, texeis_rampa};
use super::*;
use ph2d_pixel_rampa::luz::LutDaLuz;

fn objeto(s: &Sprite) -> ObjetoPixel<'_> {
    ObjetoPixel {
        bits: 1,
        sprite: s,
        quadro: s.frames()[0].id,
        stamp: 1,
        img_mundo: Affine::IDENTITY,
        lin: [1.0, 0.0, 0.0, 1.0],
        sem_luz: false,
    }
}

/// A cor (única) da fixture `heroi_p14`.
const COR: [u8; 4] = [70, 120, 200, 255];

/// A fixture `heroi_p14` com a rampa `cores` (entram na paleta).
fn com_rampa(cores: &[[u8; 4]]) -> Sprite {
    let mut s = ph2d_pixel_cena::fixture::heroi_p14();
    let op = ph2d_pixel_rampa::op_rampa_nova(&s, "r", cores).expect("rampa nova");
    ph2d_pixel_core::apply(&mut s, op).unwrap();
    s
}

#[test]
fn a_banda_de_rampa_marca_cada_texel_da_rampa() {
    let escura = [20, 40, 90, 255];
    for (cores, esperado) in [
        (vec![escura, COR], [3, 1, 0, 255]),
        (vec![escura, [200, 220, 250, 255]], [0, 0, 0, FORA_DA_RAMPA]),
    ] {
        let s = com_rampa(&cores);
        let o = objeto(&s);
        let lut = LutDaLuz::da_paleta(s.palette(), s.ramps());
        let banda = texeis_rampa(&o, &lut, 3);
        let mut cor = Vec::new();
        ph2d_pixel_core::compose_rgba(&s, o.quadro, None, &mut cor);
        let mut opacos = 0;
        for (c, b) in cor.as_chunks::<4>().0.iter().zip(banda.as_chunks::<4>().0) {
            match c[3] {
                0 => assert_eq!(*b, [0; 4], "transparente"),
                _ => {
                    assert_eq!((*c, *b), (COR, esperado), "rampa {cores:?}");
                    opacos += 1;
                }
            }
        }
        assert!(opacos > 0, "controlo: a fixture tem cor");
    }
}

#[test]
fn sem_light_palette_a_banda_de_rampa_nao_existe() {
    let s0 = ph2d_pixel_cena::fixture::heroi_p14();
    let s = com_rampa(&[[20, 40, 90, 255], COR]);
    assert!(banda_rampa(&[objeto(&s)], &[false], Affine::IDENTITY).is_none());
    assert!(banda_rampa(&[objeto(&s0)], &[true], Affine::IDENTITY).is_none());
    let (_, lut) = banda_rampa(&[objeto(&s)], &[true], Affine::IDENTITY).expect("com Palette");
    assert_eq!(lut.rampas.len(), 1);
    let (_, lut) =
        banda_rampa(&[objeto(&s), objeto(&s)], &[true, true], Affine::IDENTITY).expect("dois");
    assert_eq!(
        lut.rampas.len(),
        2,
        "as rampas do 2.º vêm depois das do 1.º"
    );
}
