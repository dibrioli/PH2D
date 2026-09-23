//! Os gates do [`crate::assar`] — ver o cabeçalho dele para o mecanismo.

use crate::{Recusa, Tinta, assar, cantos};

/// Uma peça com **as duas espécies de face** e uma aresta partilhada.
///
/// ⚠️ **As duas espécies têm de estar na MESMA fixtura:** a disposição de um
/// triângulo (`(u, v) = (j, k)`, metade do ladrilho) e a de um quad (`(i, j)`,
/// o ladrilho todo) são leis diferentes, e uma fixtura de uma espécie só deixa
/// a outra sem régua.
fn peca() -> (usize, Vec<Vec<u32>>) {
    let faces = vec![
        vec![0, 1, 2],    // triângulo
        vec![2, 1, 3],    // triângulo, partilha a aresta 1–2
        vec![3, 1, 4, 5], // quad
    ];
    (6, faces)
}

/// Uma cor distinta por amostra — é ela que torna as réguas deste ficheiro
/// capazes de acusar um endereço trocado.
fn semeia(t: &mut Tinta) {
    let n = t.amostras().len();
    for (i, a) in t.amostras_mut().iter_mut().enumerate() {
        let x = i as f32 / n as f32;
        *a = [x, 1.0 - x, (x * 7.0).fract()];
    }
}

fn tinta(nivel: u8) -> (Tinta, Vec<Vec<u32>>) {
    let (v, faces) = peca();
    let mut t = Tinta::nova(v, faces.iter().map(Vec::as_slice), nivel);
    semeia(&mut t);
    (t, faces)
}

fn byte(c: f32) -> u8 {
    (c.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// O texel que uma coordenada aponta — a volta exacta da [`crate::assar`].
fn texel(a: &crate::Assado, uv: [f32; 2]) -> [u8; 4] {
    let s = a.lado_px as f32;
    let x = (uv[0] * s - 0.5).round() as u32;
    let y = ((1.0 - uv[1]) * s - 0.5).round() as u32;
    let p = (y as usize) * (a.lado_px as usize) + (x as usize);
    [
        a.rgba[p * 4],
        a.rgba[p * 4 + 1],
        a.rgba[p * 4 + 2],
        a.rgba[p * 4 + 3],
    ]
}

/// ⭐⭐⭐ **A corrente inteira numa asserção: o canto de uma face lê a cor do
/// VÉRTICE dela.**
///
/// Endereço → ladrilho → `uv` → texel. ⛔ Um canto trocado, um `v` não
/// invertido ou um ladrilho na coluna errada reprovam todos aqui.
///
/// ⚠️ **O CONTROLO vem dentro:** sem cores distintas isto passaria com a
/// disposição espelhada, logo o gate exige que a peça tenha mais cores
/// diferentes do que faces.
#[test]
fn o_canto_de_uma_face_le_a_cor_do_vertice() {
    for nivel in [0u8, 1, 3] {
        let (t, faces) = tinta(nivel);
        let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("a peça assa");
        let mut vistas = std::collections::BTreeSet::new();
        for (f, face) in faces.iter().enumerate() {
            let n = cantos(face);
            let base = a.off_uv[f] as usize;
            assert_eq!(
                a.off_uv[f + 1] as usize - base,
                n,
                "a face {f} tem {n} cantos e o assado deu-lhe outro número"
            );
            for (c, &vert) in face.iter().enumerate().take(n) {
                let v = vert as usize;
                let esperado = t.amostras()[v];
                let lido = texel(&a, a.uv[base + c]);
                assert_eq!(
                    [lido[0], lido[1], lido[2]],
                    [byte(esperado[0]), byte(esperado[1]), byte(esperado[2])],
                    "nível {nivel}, face {f}, canto {c} (vértice {v})"
                );
                assert_eq!(lido[3], 255, "o alfa de um texel com amostra é opaco");
                vistas.insert(lido);
            }
        }
        assert!(
            vistas.len() > faces.len(),
            "CONTROLO: a fixtura tem de ter cores distintas ({} vistas)",
            vistas.len()
        );
    }
}

/// ⭐⭐ **A fronteira PARTILHADA sobrevive ao assado.**
///
/// As duas faces que partilham a aresta `1–2` escrevem a MESMA amostra, logo os
/// dois ladrilhos leem a mesma cor nela. ⛔ *É esta propriedade que faz a
/// costura ser invisível em cor* — e ela é a diferença de espécie para o Ptex,
/// que duplica a fronteira com valores próprios.
#[test]
fn os_dois_lados_de_uma_aresta_leem_a_mesma_cor() {
    let (t, faces) = tinta(3);
    let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("a peça assa");
    // O vértice `1` é canto da face `0` e da face `1`.
    let c0 = faces[0].iter().position(|v| *v == 1).expect("canto");
    let c1 = faces[1].iter().position(|v| *v == 1).expect("canto");
    let u0 = a.uv[a.off_uv[0] as usize + c0];
    let u1 = a.uv[a.off_uv[1] as usize + c1];
    assert_ne!(u0, u1, "CONTROLO: são ladrilhos diferentes");
    assert_eq!(
        texel(&a, u0),
        texel(&a, u1),
        "a amostra é UMA e os dois ladrilhos têm de a ler igual"
    );
}

/// ⭐⭐⭐ **A DILATAÇÃO: todo ponto de uma face amostra texels COM COR.**
///
/// ⛔ Sem ela a borda de toda face — e a hipotenusa de todo triângulo — sai com
/// uma linha escura: um `uv` sobre a borda cai ENTRE dois centros de texel,
/// logo a bilinear lê um bloco `2×2` que inclui um texel de fora do ladrilho.
///
/// ⚠️⚠️ **A régua entra pela porta PÚBLICA e não recopia a disposição:** ela
/// varre o INTERIOR e a BORDA de cada face em coordenadas da própria face,
/// interpola os `uv` dos cantos (que é o que um motor faz) e confere o bloco
/// que a bilinear leria. *Uma régua que recalculasse o ladrilho afirmaria sobre
/// a cópia dela, nunca sobre o assado.*
///
/// ⚠️ **O CONTROLO é o piso:** se a textura estivesse toda coberta por amostras
/// isto passaria com a dilatação apagada.
#[test]
fn todo_ponto_de_uma_face_amostra_texels_com_cor() {
    let (t, faces) = tinta(2);
    let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("a peça assa");
    assert!(
        a.relatorio.com_amostra < a.relatorio.texels,
        "CONTROLO: tem de haver texels sem amostra ({} de {})",
        a.relatorio.com_amostra,
        a.relatorio.texels
    );
    let w = a.lado_px as i64;
    let vazio = |x: i64, y: i64| {
        if x < 0 || y < 0 || x >= w || y >= w {
            return true;
        }
        let p = ((y * w + x) as usize) * 4;
        a.rgba[p..p + 4] == [0u8, 0, 0, 0]
    };
    const N: usize = 12;
    let mut pontos = 0usize;
    for (f, face) in faces.iter().enumerate() {
        let n = cantos(face);
        let cs = &a.uv[a.off_uv[f] as usize..a.off_uv[f + 1] as usize];
        for pa in 0..=N {
            for pb in 0..=N {
                let (x, y) = (pa as f32 / N as f32, pb as f32 / N as f32);
                let uv = if n == 3 {
                    if x + y > 1.0 {
                        continue;
                    }
                    [
                        cs[0][0] + (cs[1][0] - cs[0][0]) * x + (cs[2][0] - cs[0][0]) * y,
                        cs[0][1] + (cs[1][1] - cs[0][1]) * x + (cs[2][1] - cs[0][1]) * y,
                    ]
                } else {
                    let mistura = |k: usize| {
                        cs[0][k] * (1.0 - x) * (1.0 - y)
                            + cs[1][k] * x * (1.0 - y)
                            + cs[2][k] * x * y
                            + cs[3][k] * (1.0 - x) * y
                    };
                    [mistura(0), mistura(1)]
                };
                // O canto inferior-esquerdo do bloco `2×2` que a bilinear lê.
                let bx = (uv[0] * w as f32 - 0.5).floor() as i64;
                let by = ((1.0 - uv[1]) * w as f32 - 0.5).floor() as i64;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    assert!(
                        !vazio(bx + dx, by + dy),
                        "face {f}, ponto ({x:.2}, {y:.2}): o texel \
                         ({}, {}) do bloco bilinear está por preencher",
                        bx + dx,
                        by + dy
                    );
                }
                pontos += 1;
            }
        }
    }
    assert!(pontos > 200, "piso de população: {pontos} pontos");
}

/// ⛔ **A recusa NOMEIA os dois números** — sem eles o artista não sabe se
/// baixa o detalhe ou se corta a peça.
#[test]
fn a_recusa_nomeia_o_que_pediu_e_o_que_ha() {
    let (t, faces) = tinta(4);
    let e = assar(&t, faces.iter().map(Vec::as_slice), 8).expect_err("não cabe em 8");
    match e {
        Recusa::NaoCabe { preciso, tecto } => {
            assert!(preciso > tecto, "{preciso} <= {tecto}");
            assert_eq!(tecto, 8);
            assert!(e.to_string().contains(&preciso.to_string()));
        }
        Recusa::SemFaces => panic!("a peça tem faces"),
    }
    let vazia = Tinta::nova(0, std::iter::empty(), 0);
    assert_eq!(
        assar(&vazia, std::iter::empty(), 4096),
        Err(Recusa::SemFaces)
    );
}

/// ⭐ **Ao nível base o assado é a cor POR VÉRTICE, e nada mais.**
///
/// ⚠️ É o gémeo da lei que a [`crate::enderecos`] declara sobre o bloco zero:
/// *uma família nova cujo caso base é o produto que já shipa não precisa de uma
/// migração* — e aqui isso lê-se como **o assado de uma peça sem tinta fina ser
/// a textura que a cor de vértice já era**.
#[test]
fn o_nivel_base_leva_a_cor_por_vertice() {
    let (t, faces) = tinta(0);
    assert_eq!(t.lado(), 1, "o nível base tem lado 1");
    let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("assa");
    for (f, face) in faces.iter().enumerate() {
        for (c, &vert) in face.iter().enumerate().take(cantos(face)) {
            let v = vert as usize;
            let lido = texel(&a, a.uv[a.off_uv[f] as usize + c]);
            assert_eq!(
                lido[0],
                byte(t.plano_por_vertice()[v][0]),
                "face {f} canto {c}"
            );
        }
    }
}
