//! **B25.2 (T1-10) — o byte do lote em bandas não depende do número de fios.** A ponta reta (`dab_quina`)
//! rodada a 45° chega a `√2·r` no eixo; a porta de região que só somava `r + 1` rejeitava o dab nas bandas
//! que a quina alcança e cortava os cantos numa linha horizontal — num ponto que muda com os fios.

use super::{FIOS, stamp_plain_dabs_banded_with};
use ph2d_painter_brush::{BrushSpec, Dab, DirtyRect};

const W: u32 = 256;
const H: u32 = 256;

/// A ponta reta a 45°, dura: a quina inteira, onde o alcance é o maior (`√2`).
fn ponta_reta() -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        hardness: 1.0,
        dab_quina: 1.0,
        dab_angle_deg: 45,
        color: [0.1, 0.2, 0.8],
        ..BrushSpec::default()
    }
}

/// Uma coluna de dabs espaçados mais que o alcance: cada fronteira de banda cai entre dois dabs, onde só
/// os cantos chegam.
#[allow(clippy::cast_precision_loss)]
fn coluna() -> Vec<Dab> {
    (0..12)
        .map(|i| Dab {
            center: [128.0, 20.0 + i as f32 * 19.0],
            radius_px: 12.0,
            coverage: 0.8,
            color: [(i % 5) as f32 / 5.0, 0.3, 0.9],
            color_shift: [0.0; 3],
            rotation: [1.0, 0.0],
            dir: [0.0, 1.0],
            arc_len: 0.0,
            stroke_radius_px: 12.0,
            step: [0.0, 0.0],
            ante: None,
            tilt: None,
        })
        .collect()
}

fn carimba(fios: Option<usize>, brush: &BrushSpec, dabs: &[Dab]) -> (Vec<u8>, Option<DirtyRect>) {
    let mut buf = vec![255u8; (W as usize) * (H as usize) * 4];
    FIOS.with(|f| f.set(fios));
    let min_area = if fios.is_some() { 0 } else { usize::MAX };
    let r = stamp_plain_dabs_banded_with(&mut buf, W, H, dabs, brush, false, None, min_area);
    FIOS.with(|f| f.set(None));
    (buf, r)
}

fn difere(a: &[u8], b: &[u8]) -> usize {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0)
        .filter(|(x, y)| x != y)
        .count()
}

/// 1 fio (o laço serial, o oráculo) == 2 == 32 fios, ao byte; e o retângulo devolvido cobre tudo o que mudou.
/// O vermelho corre no mesmo processo: com a porta de ANTES (`PORTA_DO_DISCO`) o lote a 32 fios corta os cantos.
#[test]
fn a_ponta_reta_rodada_da_o_mesmo_byte_a_1_2_e_32_fios() {
    let dabs = coluna();
    let (serial, _) = carimba(None, &ponta_reta(), &dabs);
    super::PORTA_DO_DISCO.with(|p| p.set(true));
    let (antes, _) = carimba(Some(32), &ponta_reta(), &dabs);
    super::PORTA_DO_DISCO.with(|p| p.set(false));
    let cortados = difere(&serial, &antes);
    eprintln!("porta do disco a 32 fios: {cortados} píxeis divergem do serial");
    assert!(
        cortados > 0,
        "controlo: a porta de antes tinha de cortar a quina"
    );
    for brush in [
        ponta_reta(),
        BrushSpec {
            dab_angle_deg: 30,
            dab_quina: 0.6,
            ..ponta_reta()
        },
    ] {
        let (serial, _) = carimba(None, &brush, &dabs);
        for fios in [2usize, 32] {
            let (banda, r) = carimba(Some(fios), &brush, &dabs);
            let difere = difere(&serial, &banda);
            assert_eq!(
                difere, 0,
                "{difere} píxeis divergem do laço serial a {fios} fios (quina {}, {}°)",
                brush.dab_quina, brush.dab_angle_deg
            );
            let fora = banda
                .as_chunks::<4>()
                .0
                .iter()
                .enumerate()
                .filter(|(_, p)| **p != [255; 4])
                .filter(|(i, _)| {
                    let (x, y) = ((*i as u32) % W, (*i as u32) / W);
                    !r.is_some_and(|r| x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)
                })
                .count();
            assert_eq!(
                fora, 0,
                "{fora} píxeis pintados fora do retângulo a {fios} fios"
            );
        }
    }
}
