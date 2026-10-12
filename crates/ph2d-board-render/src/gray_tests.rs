use super::*;

/// ⭐ Paridade com a lei canónica: o caminho rápido (a tabela, a cor repetida) dá EXACTAMENTE o
/// `SrgbRgba::to_linear().luminance()` do `ph2d-color` re-codificado, numa grelha do cubo inteiro
/// (passo 5: 52³ cores) e em todos os cinzentos.
#[test]
fn the_gray_is_the_canonical_luminance_bit_for_bit() {
    let mut px = Vec::new();
    let steps: Vec<u8> = (0..=255).step_by(5).collect();
    for &r in &steps {
        for &g in &steps {
            for &b in &steps {
                px.extend([r, g, b, 200]);
            }
        }
    }
    for v in 0..=255u8 {
        px.extend([v, v, v, 7]);
    }
    let src = px.clone();
    to_gray(&mut px);
    for (o, s) in px.chunks(4).zip(src.chunks(4)) {
        let want = ph2d_color::SrgbRgba::new(s[0], s[1], s[2], s[3])
            .to_linear()
            .luminance();
        let want = linear_to_srgb_byte(want);
        assert_eq!(o, [want, want, want, s[3]], "{s:?}");
    }
}

/// Um cinzento fica o mesmo (os coeficientes somam 1), e as primárias dão os pesos de Rec. 709.
#[test]
fn grays_stay_and_primaries_weigh_by_rec709() {
    for v in 0..=255u8 {
        assert_eq!(gray_of([v, v, v]), v);
    }
    // 0,2126 · 0,7152 · 0,0722 de luz → sRGB.
    assert_eq!(gray_of([255, 0, 0]), linear_to_srgb_byte(0.2126));
    assert_eq!(gray_of([0, 255, 0]), linear_to_srgb_byte(0.7152));
    assert_eq!(gray_of([0, 0, 255]), linear_to_srgb_byte(0.0722));
    assert_eq!(gray_of([255, 0, 0]), 127);
}

/// RÉGUA (à mão, `--release`): o preto e branco de uma fotografia de 4000 × 3000 de ruído (o pior caso:
/// nenhuma cor repetida seguida) — corre no fio da descodificação, fora do desenho. Imprime o MÍNIMO
/// de 5 corridas. Medido 11/10 (`load 69`, outra linha a compilar): **78 ms** ⇒ nenhuma tabela nem
/// GPU (um JPEG dessa fotografia descodifica-se em mais).
#[test]
#[ignore = "régua de tempo (à mão, --release)"]
fn regua_cinza_4000x3000() {
    let mut seed = 0x1234_5678u32;
    let src: Vec<u8> = (0..4000 * 3000 * 4)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as u8
        })
        .collect();
    let best = (0..5)
        .map(|_| {
            let mut px = src.clone();
            let t = std::time::Instant::now();
            to_gray(&mut px);
            t.elapsed()
        })
        .min()
        .unwrap();
    println!("to_gray 4000×3000 (ruído): mínimo {best:?}");
}
