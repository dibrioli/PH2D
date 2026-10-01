//! ⭐⭐⭐ A ampliação — a lei, na CPU (o gémeo da placa).

use super::amplia_cpu;

fn imagem(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut v = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            v.extend_from_slice(&f(x, y));
        }
    }
    v
}

/// No mesmo tamanho a ampliação é a IDENTIDADE ao bit — os pesos de Catmull-Rom em `t = 0` são
/// `(0, 1, 0, 0)`.
#[test]
fn no_mesmo_tamanho_e_a_identidade() {
    let a = imagem(13, 7, |x, y| {
        [(x * 19) as u8, (y * 31) as u8, (x * y) as u8, 255]
    });
    assert_eq!(amplia_cpu(&a, (13, 7), (13, 7)), a);
}

/// ⭐⭐⭐ **O ANTI-ANEL: junto de uma aresta não há halo.** Uma aresta `64 | 192` ampliada `2×`:
/// todo pixel cujos quatro texels mais perto são do lado escuro sai EXACTAMENTE `64`.
///
/// ⚠️ O CONTROLO: a bicúbica CRUA nesse pixel desce abaixo de `64` — sem ele, o gate passaria por
/// uma fixtura que não toca (o anel de uma aresta `0 | 255` esconde-se na saturação do byte).
#[test]
fn junto_de_uma_aresta_nao_ha_halo() {
    let (w, h) = (8u32, 4u32);
    let a = imagem(w, h, |x, _| if x < 4 { [64; 4] } else { [192; 4] });
    let b = amplia_cpu(&a, (w, h), (16, 8));
    // O pixel de saída `x = 5`: centro em `u = 5,5/2 − 0,5 = 2,25` ⇒ os texels `2` e `3`, os dois
    // escuros, e a cauda do filtro já apanha o `4`, claro.
    for y in 0..8usize {
        let o = (y * 16 + 5) * 4;
        assert_eq!(
            &b[o..o + 4],
            &[64, 64, 64, 64],
            "halo junto da aresta na linha {y}"
        );
    }
    // CONTROLO: a bicúbica crua em `u = 2,25` sobre `[64, 64, 64, 192]` (texels 1..4).
    let t: f32 = 0.25;
    let pesos = [
        -0.5 * t * t * t + t * t - 0.5 * t,
        1.5 * t * t * t - 2.5 * t * t + 1.0,
        -1.5 * t * t * t + 2.0 * t * t + 0.5 * t,
        0.5 * t * t * t - 0.5 * t * t,
    ];
    let crua: f32 = [64.0, 64.0, 64.0, 192.0]
        .iter()
        .zip(pesos)
        .map(|(v, p)| v * p)
        .sum();
    assert!(
        crua < 63.5,
        "CONTROLO: a bicúbica crua não toca nesta fixtura ({crua}) — o gate não mediria o anti-anel"
    );
}

/// Numa rampa suave a ampliação segue a rampa (não é o vizinho mais perto): o pixel do meio fica
/// ENTRE os dois texels.
#[test]
fn numa_rampa_interpola() {
    let a = imagem(4, 1, |x, _| {
        let v = (x * 60 + 30) as u8;
        [v, v, v, 255]
    });
    let b = amplia_cpu(&a, (4, 1), (8, 1));
    // saída `x = 3`: `u = 1,25` — entre `90` e `150`.
    let v = b[3 * 4];
    assert!(v > 90 && v < 150, "a rampa não foi interpolada: {v}");
}
