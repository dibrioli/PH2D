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
        // ⭐ A exaustividade é o censo: uma recusa nova **não compila** até
        //   alguém dizer o que ela significa aqui.
        Recusa::SemFaces => panic!("a peça tem faces"),
        Recusa::NaoDescreve { .. } => panic!("o plano descreve esta malha"),
    }
    let vazia = Tinta::nova(0, std::iter::empty(), 0);
    assert_eq!(
        assar(&vazia, std::iter::empty(), 4096),
        Err(Recusa::SemFaces)
    );
}

/// ⭐⭐⭐⭐ **O EMPACOTADOR: um plano GRADUADO assa, e cada face fica no ladrilho
/// do tamanho DELA.**
///
/// ⚠️ **A régua entra pela porta pública:** ela mede, nos `uv` que saem, a
/// LARGURA do ladrilho de cada face — e ela tem de ser a da retícula daquela
/// face, `lado + 1` texels. ⛔ *Um assado que empacotasse tudo ao tamanho de
/// uma face qualquer daria UV certas na mesma e perderia amostras em silêncio;
/// é a largura, e não a cor, que o acusa.*
///
/// ⚠️ **E o CONTROLO é a face grossa:** sem ele este gate passaria com todos os
/// ladrilhos do tamanho do MAIOR, que é o desperdício que o empacotador existe
/// para não ter.
#[test]
fn um_plano_graduado_assa_e_cada_face_leva_o_ladrilho_dela() {
    let faces: Vec<Vec<u32>> = vec![vec![0, 1, 2], vec![1, 3, 2]];
    let it = || faces.iter().map(Vec::as_slice);

    let graduado = Tinta::graduada(4, it(), &[1, 3], 1).expect("dois níveis");
    let a = assar(&graduado, it(), 4096).expect("um plano graduado ASSA");

    // A largura do ladrilho de cada face, lida dos `uv` que saem.
    let largura = |f: usize| -> f32 {
        let c = &a.uv[a.off_uv[f] as usize..a.off_uv[f + 1] as usize];
        (c[1][0] - c[0][0]).abs() * a.lado_px as f32
    };
    // Face 0 está ao nível `1` (lado `2`) e a face 1 ao `3` (lado `8`).
    assert!(
        (largura(0) - 2.0).abs() < 1e-3,
        "a face grossa devia ocupar 2 texels de largura, ocupa {:.3}",
        largura(0)
    );
    assert!(
        (largura(1) - 8.0).abs() < 1e-3,
        "a face fina devia ocupar 8 texels de largura, ocupa {:.3}",
        largura(1)
    );
    assert_eq!(
        a.relatorio.ladrilho,
        8 + 1 + 2 * crate::assar::FOLGA_EM_TEXELS
    );
}

/// ⛔⛔⛔ **NENHUM LADRILHO PISA OUTRO — e é esta a lei que a cor não acusa.**
///
/// ⚠️⚠️ Duas faces empilhadas no mesmo sítio dão `uv` perfeitamente válidas e
/// uma textura em que a segunda escreve por cima da primeira: *o canto de cada
/// uma continua a ler a cor do vértice dela se elas partilharem o vértice*, e
/// os gates de COR passam. O que as separa é a GEOMETRIA da disposição.
///
/// ⭐ A régua entra pela porta pública: o rectângulo de cada face sai dos `uv`
/// dos cantos dela, **crescido da folga**, que é o ladrilho inteiro.
#[test]
fn nenhum_ladrilho_pisa_outro() {
    // Uma tira de seis triângulos com quatro níveis diferentes — a mistura é o
    // que obriga o empacotador a decidir alguma coisa.
    let mut pos = 0usize;
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..6u32 {
        faces.push(vec![3 * i, 3 * i + 1, 3 * i + 2]);
        pos += 3;
    }
    let niveis = [0u8, 3, 1, 3, 2, 0];
    let it = || faces.iter().map(Vec::as_slice);
    let t = Tinta::graduada(pos, it(), &niveis, 1).expect("seis níveis");
    let a = assar(&t, it(), 4096).expect("assa");

    let folga = crate::assar::FOLGA_EM_TEXELS as f32;
    let s = a.lado_px as f32;
    let rect = |f: usize| -> (f32, f32, f32, f32) {
        let c = &a.uv[a.off_uv[f] as usize..a.off_uv[f + 1] as usize];
        let xs: Vec<f32> = c.iter().map(|p| p[0] * s).collect();
        // ⚠️ O `v` do ficheiro conta de BAIXO; aqui só interessam extremos.
        let ys: Vec<f32> = c.iter().map(|p| (1.0 - p[1]) * s).collect();
        let mn = |v: &[f32]| v.iter().copied().fold(f32::INFINITY, f32::min);
        let mx = |v: &[f32]| v.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        (
            mn(&xs) - folga - 0.5,
            mn(&ys) - folga - 0.5,
            mx(&xs) + folga + 0.5,
            mx(&ys) + folga + 0.5,
        )
    };
    for f in 0..faces.len() {
        let (x0, y0, x1, y1) = rect(f);
        assert!(
            x0 >= -1e-3 && y0 >= -1e-3 && x1 <= s + 1e-3 && y1 <= s + 1e-3,
            "face {f}: o ladrilho ({x0:.2},{y0:.2})..({x1:.2},{y1:.2}) sai da textura de {s}"
        );
        for g in (f + 1)..faces.len() {
            let (u0, v0, u1, v1) = rect(g);
            let pisa = x0 < u1 - 1e-3 && u0 < x1 - 1e-3 && y0 < v1 - 1e-3 && v0 < y1 - 1e-3;
            assert!(
                !pisa,
                "as faces {f} e {g} partilham texels: \
                 ({x0:.2},{y0:.2})..({x1:.2},{y1:.2}) contra ({u0:.2},{v0:.2})..({u1:.2},{v1:.2})"
            );
        }
    }
    // ⚠️ CONTROLO: a fixtura tem de ter ladrilhos de tamanhos DIFERENTES,
    //    senão isto mede uma grelha e não um empacotador.
    let larguras: std::collections::BTreeSet<u32> = (0..faces.len())
        .map(|f| {
            let (x0, _, x1, _) = rect(f);
            (x1 - x0).round() as u32
        })
        .collect();
    assert!(
        larguras.len() >= 3,
        "CONTROLO: a fixtura devia ter vários tamanhos de ladrilho ({larguras:?})"
    );
}

/// ⭐⭐⭐ **E com um plano UNIFORME o empacotador devolve a GRELHA DE SEMPRE.**
///
/// ⚠️⚠️ **É esta metade que dá direito a haver UM caminho em vez de dois:** o
/// lado tem de ser `ceil(√n) × ladrilho`, que é a conta que este ficheiro fazia
/// à mão antes da P2. *Uma cura que muda o caso que já shipa não é uma cura.*
#[test]
fn com_um_nivel_so_o_empacotador_devolve_a_grelha_de_antes() {
    for nivel in [0u8, 1, 3] {
        let (t, faces) = tinta(nivel);
        let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("a peça assa");
        let ladrilho = (1u32 << nivel) + 1 + 2 * crate::assar::FOLGA_EM_TEXELS;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let cols = (faces.len() as f64).sqrt().ceil() as u32;
        assert_eq!(
            a.lado_px,
            cols * ladrilho,
            "nível {nivel}: o lado tem de ser o da grelha de antes"
        );
        assert_eq!(a.relatorio.ladrilho, ladrilho);
    }
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
    assert_eq!(
        t.lado_uniforme().expect("a fixtura e' uniforme"),
        1,
        "o nível base tem lado 1"
    );
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

/// ⛔⛔⛔ **UM PLANO QUE NÃO DESCREVE A MALHA TEM DE RECUSAR, NUNCA ESTOURAR.**
///
/// ⚠️⚠️ **É a lei que esta casa já pagou com um `panic` na cara do dono** (o
/// report de 21/09, `topo.rs:239 — index out of bounds: the len is 196608 but
/// the index is 196608`): a `Topologia` guarda `4` entradas por face, logo uma
/// lista de faces mais longa do que a que a construiu indexa **fora de
/// alcance**. A cura de então foi a porta [`crate::Topologia::descreve`] e o
/// `payload` a devolver VEREDITO — e o assado nasceu, uma wave depois, **sem
/// nenhuma das duas**.
///
/// ⛔ **E a metade CURTA é a pior**, como lá: com MENOS faces nada sai de
/// alcance, o laço acaba sozinho e a textura fica com tinta **válida no sítio
/// errado**, em silêncio.
#[test]
fn um_plano_que_nao_descreve_a_malha_recusa_em_vez_de_estourar() {
    let (v, faces) = peca();
    let mut t = Tinta::nova(v, faces.iter().map(Vec::as_slice), 2);
    semeia(&mut t);

    // O CONTROLO: com a malha que a construiu, ela assa.
    assert!(
        assar(&t, faces.iter().map(Vec::as_slice), 4096).is_ok(),
        "CONTROLO: a malha certa tem de assar"
    );

    // A metade que ESTOURA: mais faces do que o plano conhece.
    let mais: Vec<Vec<u32>> = faces.iter().cloned().chain([vec![0, 1, 2]]).collect();
    assert_eq!(
        assar(&t, mais.iter().map(Vec::as_slice), 4096),
        Err(Recusa::NaoDescreve {
            faces: mais.len(),
            plano: faces.len(),
        }),
        "com MAIS faces o assado tem de recusar — o caminho de antes era um panic"
    );

    // A metade MUDA: menos faces. Nada sai de alcance e a tinta iria para o
    // sítio errado sem uma queixa.
    let menos: Vec<Vec<u32>> = faces[..faces.len() - 1].to_vec();
    assert_eq!(
        assar(&t, menos.iter().map(Vec::as_slice), 4096),
        Err(Recusa::NaoDescreve {
            faces: menos.len(),
            plano: faces.len(),
        }),
        "com MENOS faces o assado fica truncado e ninguém acusa: tem de recusar"
    );
}

/// ⭐ **O que sai no ficheiro são TRÊS canais, e a cobertura fica em casa.**
///
/// ⚠️ **A metade que importa é a segunda:** o `rgba` **mantém** o alfa, porque
/// é dele que os gates deste ficheiro tiram *«este texel recebeu alguma
/// coisa?»*. Sem ela alguém «simplificaria» o assado para três canais e
/// apagaria a régua de toda a família de graça.
#[test]
fn o_ficheiro_leva_cor_e_a_cobertura_fica_em_casa() {
    let (t, faces) = tinta(2);
    let a = assar(&t, faces.iter().map(Vec::as_slice), 4096).expect("assa");
    let rgb = a.rgb();
    assert_eq!(rgb.len(), a.rgba.len() / 4 * 3);
    for (i, p) in a.rgba.as_chunks::<4>().0.iter().enumerate() {
        assert_eq!(&rgb[i * 3..i * 3 + 3], &p[..3], "o texel {i} mudou de cor");
    }
    // A cobertura continua a existir do lado do assado — e com as DUAS metades
    // presentes, senão a régua deixa de discriminar.
    assert!(
        a.rgba.as_chunks::<4>().0.iter().any(|p| p[3] == 255),
        "nada coberto"
    );
    assert!(
        a.rgba.as_chunks::<4>().0.iter().any(|p| p[3] == 0),
        "nada por cobrir"
    );
}
