//! A5-a — a máscara da tinta e o anel da arte.

use super::*;

/// ⭐⭐ **GATE — a máscara responde como o alfa**, pixel a pixel, numa grelha com corridas e buracos.
#[test]
fn a_mascara_responde_como_o_alfa() {
    let (w, h) = (13_u32, 5_u32);
    let alfa: Vec<u8> = (0..w * h)
        .map(|i| if (i * 7 + i / w) % 5 < 2 { 0 } else { 200 })
        .collect();
    let m = Mascara::do_alfa(&alfa, w, h, 1);
    for y in 0..h {
        for x in 0..w {
            let p = [f64::from(x) + 0.5, f64::from(y) + 0.5];
            assert_eq!(m.tem(p), alfa[(y * w + x) as usize] >= 1, "({x}, {y})");
        }
    }
    assert!(
        !m.tem([-0.5, 1.0]) && !m.tem([13.5, 1.0]),
        "fora da célula não há tinta"
    );
}

/// Uma malha quadrada `[0, 40]²` em dois triângulos.
fn quadrado() -> Mesh2d {
    Mesh2d {
        rest: vec![[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]],
        tris: vec![[0, 1, 2], [0, 2, 3]],
        size: [40, 40],
    }
}

/// ⭐⭐ **GATE — o anel da arte ENCOSTA cada ponto da borda à primeira tinta pela NORMAL da aresta**:
/// um disco de raio `12` no centro do quadrado (a tampa dentro das células). Um ponto cuja normal
/// encontra o disco fica a `≤ 1` pixel dele; um cuja normal passa ao lado (perto dos cantos) fica
/// na borda — a marcha é axial. ⛔ **Os CONTROLOS:** a borda da malha fica a `≥ 8` pixels do disco,
/// e os que encostam são os `~4 × 24` pontos cuja normal o cruza.
#[test]
fn o_anel_da_arte_encosta_a_borda_a_tinta() {
    let m = quadrado();
    let alfa: Vec<u8> = (0..40 * 40)
        .map(|i| {
            let (x, y) = (f64::from(i % 40) + 0.5, f64::from(i / 40) + 0.5);
            if (x - 20.0).hypot(y - 20.0) <= 12.0 {
                255
            } else {
                0
            }
        })
        .collect();
    let mascara = Mascara::do_alfa(&alfa, 40, 40, 1);
    let aneis = crate::skin_image_fecho::aneis_da_borda(&m.tris);
    let arte = anel_da_arte(&m, &mascara, &aneis);
    assert_eq!(arte.len(), 1);
    assert!(
        arte[0].len() >= 150,
        "um ponto por pixel da borda ({})",
        arte[0].len()
    );
    let mut encostados = 0;
    for p in &arte[0] {
        let t = m.tris[p.tri as usize].map(|v| m.rest[v as usize]);
        let q = [0, 1]
            .map(|c| (1.0 - p.uv[0] - p.uv[1]) * t[0][c] + p.uv[0] * t[1][c] + p.uv[1] * t[2][c]);
        let d = (q[0] - 20.0).hypot(q[1] - 20.0) - 12.0;
        if d <= 1.0 {
            encostados += 1;
            continue;
        }
        // Na borda, e a normal (a do lado onde está) passa ao lado do disco.
        let longe_do_eixo = if q[1].min(40.0 - q[1]) < 1e-6 {
            (q[0] - 20.0).abs()
        } else if q[0].min(40.0 - q[0]) < 1e-6 {
            (q[1] - 20.0).abs()
        } else {
            0.0
        };
        assert!(
            longe_do_eixo >= 11.0,
            "um ponto do anel ficou a {d:.2} px do disco em {q:?}"
        );
    }
    assert!(
        encostados >= 90,
        "só {encostados} de {} pontos encostaram",
        arte[0].len()
    );
    let borda_min = m
        .rest
        .iter()
        .map(|q| (q[0] - 20.0).hypot(q[1] - 20.0) - 12.0)
        .fold(f64::MAX, f64::min);
    assert!(
        borda_min >= 8.0,
        "o CONTROLO: a borda da malha está a {borda_min:.2} do disco"
    );
}
