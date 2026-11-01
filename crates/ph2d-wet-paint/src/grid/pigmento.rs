//! O lado ESPECTRAL da tinta (E10b, «Pigment mix»): por célula `[filme, aberto]` de [`Espectro`] (112 B), num vector
//! LATERAL que só nasce quando um pincel com `p > 0` pousa ([`garante`]); com `p = 0` em toda a tela não existe e a
//! cor é a de [`super::srgb_0_255`] AO BIT. Ainda SEM campo em `Grid` (o `grid.rs` e as portas de transferência
//! são da ligação seguinte): aqui vivem as operações por célula que essa ligação chama. Doc:
//! `docs/Painter/Arquitetura/E10b_pigment_mix.md`.

use super::CelulaTinta;
use ph2d_paint_tinta::oklab::oklab_para_srgb;
use ph2d_paint_tinta::{Espectro, Soma, cor_pigmento, transfere_pigmento};

/// O lado espectral de uma célula: `[filme, aberto]`, pesado pelos MESMOS volumes de [`CelulaTinta`].
pub type LadoEspectral = [Espectro; 2];

/// O lado espectral de uma célula que já tinha tinta: as cores dela com `p = 0` (a única subida às bandas).
#[must_use]
pub fn migra(c: &CelulaTinta) -> LadoEspectral {
    [Espectro::da_soma(&c.filme), Espectro::da_soma(&c.aberto)]
}

/// O campo lateral; se ainda não existia, nasce migrado da tinta que a grade tem.
pub fn garante<'a>(
    lado: &'a mut Vec<LadoEspectral>,
    tinta: &[CelulaTinta],
) -> &'a mut [LadoEspectral] {
    if lado.is_empty() {
        *lado = tinta.iter().map(migra).collect();
    }
    lado
}

/// Move a fracção `f` da fatia `de` para `para` na tinta E no lado espectral (vazio = só a tinta, como hoje).
pub fn transfere(
    de: (&mut Soma, Option<&mut Espectro>),
    para: (&mut Soma, Option<&mut Espectro>),
    f: f32,
) {
    match (de.1, para.1) {
        (Some(ed), Some(ep)) => {
            let _ = transfere_pigmento((de.0, ed), (para.0, ep), f);
        }
        (None, None) => {
            let _ = ph2d_paint_tinta::transfere(de.0, para.0, f);
        }
        _ => unreachable!("o lado espectral existe para a grade inteira ou para nenhuma célula"),
    }
}

/// A cor sRGB `0..255` da fatia com o lado espectral (o irmão de [`super::srgb_0_255`]).
#[must_use]
pub fn srgb_0_255(s: &Soma, e: Option<&Espectro>) -> Option<[f32; 3]> {
    let c = match e {
        Some(e) => cor_pigmento(s, e)?,
        None => s.cor()?,
    };
    Some(oklab_para_srgb(c.lab).map(|x| (x * 255.0).clamp(0.0, 255.0)))
}

#[cfg(test)]
mod tests {
    use super::super::{soma_da_massa, srgb_0_255 as srgb_hoje};
    use super::*;

    /// Com `p = 0` o lado espectral (nascido por migração ou pousado) dá a cor de hoje AO BIT, também depois de mover.
    #[test]
    fn tinta_pigmento_p0_igual_ao_bit_no_motor() {
        let cores = [
            [40.0, 90.0, 220.0],
            [245.0, 205.0, 25.0],
            [0.0; 3],
            [255.0; 3],
            [200.0, 30.0, 60.0],
        ];
        let mut tinta: Vec<CelulaTinta> = cores
            .iter()
            .map(|&c| CelulaTinta {
                filme: soma_da_massa(c, 0.3, 0.0, 600.0),
                aberto: soma_da_massa(c, 0.1, 1.0, 250.0),
                ..CelulaTinta::NEUTRA
            })
            .collect();
        let mut lado = Vec::new();
        let _ = garante(&mut lado, &tinta);
        for i in 0..cores.len() - 1 {
            let (a, b) = tinta.split_at_mut(i + 1);
            let (la, lb) = lado.split_at_mut(i + 1);
            transfere(
                (&mut a[i].filme, Some(&mut la[i][0])),
                (&mut b[0].filme, Some(&mut lb[0][0])),
                0.4,
            );
        }
        for (c, l) in tinta.iter().zip(&lado) {
            for (s, e) in [(&c.filme, &l[0]), (&c.aberto, &l[1])] {
                let (x, y) = (srgb_0_255(s, Some(e)), srgb_hoje(s));
                assert_eq!(
                    x.map(|v| v.map(f32::to_bits)),
                    y.map(|v| v.map(f32::to_bits))
                );
            }
        }
        assert_eq!(core::mem::size_of::<LadoEspectral>(), 112);
    }
}
