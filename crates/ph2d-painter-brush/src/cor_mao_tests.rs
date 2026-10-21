//! Os portões da cor pela mão (L11). A lei medida: `docs/Painter/Pinceis/06_LEIS_DAS_LACUNAS.md` §4.

use super::*;
use crate::{BrushSpec, Dab, Dynamics, Stroke, StrokePoint};

const AZUL: [f32; 3] = [23.0 / 255.0, 101.0 / 255.0, 179.0 / 255.0];

/// Um recto de 880 px numa rampa de pressão 0 → 1 (a `rampa_0_1` da bateria).
fn rampa(c: CorDaMao) -> Vec<Dab> {
    let s = BrushSpec {
        radius_px: 10.0,
        color: AZUL,
        space_attenuation: false,
        stabilizer: 0.0,
        cor_mao: c,
        ..Default::default()
    };
    let d = Dynamics {
        size_pressure: false,
        ..Default::default()
    };
    let mut st = Stroke::new(s, d, 1);
    let (mut all, mut out) = (Vec::new(), Vec::new());
    let p = |i: usize| StrokePoint {
        pos: [60.0 + i as f32 * 4.0, 300.0],
        pressure: i as f32 / 220.0,
    };
    st.begin(p(0), &mut out);
    all.extend_from_slice(&out);
    for i in 1..=220 {
        st.extend(p(i), &mut out);
        all.extend_from_slice(&out);
    }
    st.finish(&mut out);
    all.extend_from_slice(&out);
    all
}

/// ⭐ **`sem_cor_secundaria_a_cor_e_a_de_hoje`**: a neutra não toca a cor nem o fluxo (todo dab é a cor do
/// pincel, ao bit), e a secundária sozinha (sem quantidade) não age.
#[test]
fn sem_cor_secundaria_a_cor_e_a_de_hoje() {
    let mut rng = 3u64;
    assert_eq!(CorDaMao::NEUTRA.aplica(AZUL, 0.2, &mut rng), AZUL);
    assert_eq!(rng, 3, "a neutra não sorteia");
    let so_cor = CorDaMao {
        secondary: [1.0, 0.9, 0.1],
        ..CorDaMao::NEUTRA
    };
    assert!(!so_cor.ativa());
    for d in rampa(so_cor) {
        assert_eq!(d.color, AZUL);
    }
    assert_eq!(BrushSpec::default().cor_mao, CorDaMao::NEUTRA);
}

/// ⭐ **`a_pressao_leva_a_cor_a_secundaria`** (Procreate *Color pressure → Secondary*): na mão leve a cor é a
/// secundária, na pressão cheia a primária, e o caminho é a recta entre as duas pela pressão.
#[test]
fn a_pressao_leva_a_cor_a_secundaria() {
    let amarelo = [1.0, 0.9, 0.1];
    let c = CorDaMao {
        secondary: amarelo,
        pressure_secondary: 1.0,
        ..CorDaMao::NEUTRA
    };
    let mut rng = 0u64;
    let leve = c.aplica(AZUL, 0.0, &mut rng);
    let cheia = c.aplica(AZUL, 1.0, &mut rng);
    let meio = c.aplica(AZUL, 0.5, &mut rng);
    for k in 0..3 {
        assert!((leve[k] - amarelo[k]).abs() < 1e-6);
        assert!((cheia[k] - AZUL[k]).abs() < 1e-6);
        assert!((meio[k] - 0.5 * (AZUL[k] + amarelo[k])).abs() < 1e-6);
    }
    // Pelo traço: a rampa vai do amarelo ao azul.
    let d = rampa(c);
    let (a, z) = (d[2].color, d[d.len() - 3].color);
    assert!(a[0] > 0.8 && z[2] > 0.6, "{a:?} → {z:?}");
    // O sorteio da secundária (Photoshop fg/bg): cada dab entre a primária e a secundária, e variam.
    let j = CorDaMao {
        secondary: amarelo,
        secondary_jitter: 1.0,
        ..CorDaMao::NEUTRA
    };
    let v: Vec<f32> = rampa(j).iter().map(|d| d.color[0]).collect();
    assert!(v.iter().all(|r| (AZUL[0] - 1e-6..=1.0 + 1e-6).contains(r)));
    assert!(v.windows(2).any(|w| w[0] != w[1]));
}

/// ⭐ **`a_pressao_leva_a_cor`** — o MyPaint `long_grass` (06 §4): da pressão 0 a 0,9 a saturação vai de
/// 0,29 a 0,61 e o valor de 0,62 a 0,91, a matiz fica (± 3°). Aqui a cor de partida é a da pressão cheia do
/// oráculo (S 0,645 · V 0,94) e `s 0,55 · v 0,33` dão a curva por decil ± 0,05 (o ruído entre decis vizinhos).
#[test]
fn a_pressao_leva_a_cor() {
    let base = hsv_to_rgb([0.3, 0.645, 0.94]);
    let c = CorDaMao {
        pressure_sat: 0.55,
        pressure_val: 0.33,
        ..CorDaMao::NEUTRA
    };
    let mut rng = 0u64;
    // (pressão, S, V) do long_grass nos extremos medidos; o meio pela recta do oráculo.
    for (p, s_or, v_or) in [(0.0, 0.29, 0.62), (0.45, 0.45, 0.765), (0.9, 0.61, 0.91)] {
        let hsv = rgb_to_hsv(c.aplica(base, p, &mut rng));
        assert!((hsv[1] - s_or).abs() <= 0.05, "p {p}: S {} × {s_or}", hsv[1]);
        assert!((hsv[2] - v_or).abs() <= 0.05, "p {p}: V {} × {v_or}", hsv[2]);
        assert!((hsv[0] - 0.3).abs() * 360.0 <= 3.0, "a matiz fica");
    }
}

/// As marcas DELIBERADAS (Drag Dot, Grid Stamp…) cortam todo o sorteio por dab — e a cor pela mão também
/// (revisão do bloco C): a cor é a do pincel em qualquer pressão.
#[test]
fn a_marca_deliberada_nao_leva_a_cor_pela_mao() {
    let c = CorDaMao {
        secondary: [1.0, 0.9, 0.1],
        pressure_secondary: 1.0,
        // O Drag Dot força a pressão cheia: é o SORTEIO da secundária que o corte tem de calar.
        secondary_jitter: 1.0,
        ..CorDaMao::NEUTRA
    };
    for m in [crate::StrokeMethod::DragDot, crate::StrokeMethod::GridStamp] {
        let s = BrushSpec {
            radius_px: 10.0,
            color: AZUL,
            stroke_method: m,
            cor_mao: c,
            ..Default::default()
        };
        assert!(!m.allows_jitter());
        let mut st = Stroke::new(s, Dynamics::default(), 1);
        let (mut all, mut out) = (Vec::new(), Vec::new());
        let p = |x: f32| StrokePoint {
            pos: [x, 50.0],
            pressure: 0.1,
        };
        st.begin(p(40.0), &mut out);
        all.extend_from_slice(&out);
        for i in 1..20 {
            st.extend(p(40.0 + 7.0 * i as f32), &mut out);
            all.extend_from_slice(&out);
        }
        st.finish(&mut out);
        all.extend_from_slice(&out);
        assert!(!all.is_empty(), "{m:?} pousa a marca");
        assert!(all.iter().all(|d| d.color == AZUL), "{m:?}: a cor mudou pela mão");
    }
}

/// ⭐ **O viés do sorteio de cor** (J4 `texturas`, o Pointillism do Krita): com `vies` cada dab sai da cor deslocada
/// (matiz +0,02, saturação −0,1, valor +0,12 em HSV), à mesma pressão em todo o traço; desligado é a cor de sempre.
#[test]
fn o_vies_desloca_a_cor_de_cada_dab() {
    let base = rampa(CorDaMao::NEUTRA);
    let com = rampa(CorDaMao {
        vies: [0.02, -0.1, 0.12],
        ..CorDaMao::NEUTRA
    });
    let hb = crate::jitter::rgb_to_hsv(AZUL);
    for d in &com {
        let h = crate::jitter::rgb_to_hsv(d.color);
        assert!((h[0] - (hb[0] + 0.02)).abs() < 1e-3, "matiz {} × {}", h[0], hb[0] + 0.02);
        assert!((h[1] - (hb[1] - 0.1)).abs() < 1e-3, "saturação {} × {}", h[1], hb[1] - 0.1);
        assert!((h[2] - (hb[2] + 0.12)).abs() < 1e-3, "valor {} × {}", h[2], hb[2] + 0.12);
    }
    assert!(base.iter().all(|d| d.color == AZUL), "desligado: a cor de sempre");
}
