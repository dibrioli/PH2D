//! Os gates da [`Tinta::uniformizada`] — irmão (`#[path]`) do `lib.rs`.
//!
//! ⛔⛔ **O ORÁCULO NÃO É A LEITURA.** A conversão escreve com a
//! [`Tinta::cor_tri`]/[`Tinta::cor_quad`], logo compará-la com elas mediria a
//! função contra si mesma. A régua aqui é a AMOSTRA GUARDADA no nó
//! correspondente da retícula da face — só endereços, nenhuma interpolação.

use crate::{Tinta, sitio_quad, sitio_tri};

fn cor(i: usize) -> [f32; 3] {
    let h = (i as u32).wrapping_mul(2_654_435_761);
    [
        (h & 0xff) as f32 / 255.0,
        ((h >> 8) & 0xff) as f32 / 255.0,
        ((h >> 16) & 0xff) as f32 / 255.0,
    ]
}

/// Uma grelha de `4 × 4` células em que a ÚLTIMA fila está partida em
/// triângulos — as DUAS formas de face, a partilhar arestas entre si.
fn grelha() -> (usize, Vec<Vec<u32>>) {
    const N: u32 = 4;
    let v = |i: u32, j: u32| j * (N + 1) + i;
    let mut faces = Vec::new();
    for j in 0..N {
        for i in 0..N {
            let (a, b, c, d) = (v(i, j), v(i + 1, j), v(i + 1, j + 1), v(i, j + 1));
            if j == N - 1 {
                faces.push(vec![a, b, c]);
                faces.push(vec![a, c, d]);
            } else {
                faces.push(vec![a, b, c, d]);
            }
        }
    }
    (((N + 1) * (N + 1)) as usize, faces)
}

/// Um plano graduado sobre a [`grelha`], com amostras todas distintas.
fn graduado(pedido: u8, niveis: impl Fn(usize) -> u8) -> (Vec<Vec<u32>>, Tinta) {
    let (verts, faces) = grelha();
    let ks: Vec<u8> = (0..faces.len()).map(niveis).collect();
    let mut t = Tinta::graduada(verts, faces.iter().map(|f| &f[..]), &ks, pedido)
        .expect("a lista descreve a malha");
    for (i, a) in t.amostras_mut().iter_mut().enumerate() {
        *a = cor(i);
    }
    (faces, t)
}

/// ⭐⭐⭐⭐ **GATE — ONDE A FACE ESTÁ NO DEGRAU PEDIDO OU ACIMA, A CONVERSÃO É
/// AO BIT, INCLUSIVE NAS ARESTAS QUE ELA PARTILHA COM UMA FACE MAIS GROSSA.**
///
/// ⚠️⚠️ **A segunda metade é a da ORDEM:** uma aresta entre uma face abaixo do
/// pedido e outra acima é escrita pelas duas, e só a fina tem as amostras
/// verdadeiras. Com a ordem trocada a grossa escreve por último e **interpola
/// por cima** da verdadeira — e este gate reprova, porque confere TODOS os nós
/// da face fina, arestas incluídas.
///
/// ⚠️ **O CONTROLO vem primeiro:** sem faces dos DOIS lados do pedido a metade
/// da ordem não tem o que medir.
#[test]
fn a_conversao_e_exacta_onde_a_face_esta_no_pedido_ou_acima() {
    let pedido = 2u8;
    let (faces, g) = graduado(pedido, |i| [0u8, 3, 1, 4][i % 4]);
    let (abaixo, acima) = (0..faces.len()).fold((0, 0), |(a, b), f| {
        if g.topologia().nivel_de(f) < pedido {
            (a + 1, b)
        } else {
            (a, b + 1)
        }
    });
    assert!(
        abaixo > 0 && acima > 0,
        "CONTROLO: a fixtura tem de ter faces abaixo ({abaixo}) e acima ({acima}) do pedido"
    );

    let u = g
        .uniformizada(faces.iter().map(|f| &f[..]))
        .expect("as faces descrevem o plano");
    assert_eq!(
        u.lado_uniforme(),
        Some(1 << pedido),
        "o plano novo é UNIFORME, no pedido"
    );
    assert_eq!(u.nivel(), pedido);

    let l = 1u32 << pedido;
    let mut conferidas = 0usize;
    for (fi, f) in faces.iter().enumerate() {
        let kf = g.topologia().nivel_de(fi);
        if kf < pedido {
            continue;
        }
        let passo = 1u32 << (kf - pedido);
        let lf = l * passo;
        let cantos = &f[..crate::cantos(f)];
        let mut ver = |novo: crate::Sitio, velho: crate::Sitio| {
            let a = u.amostras()[u.indice_de(fi, cantos, novo) as usize];
            let b = g.amostras()[g.indice_de(fi, cantos, velho) as usize];
            assert_eq!(
                a.map(f32::to_bits),
                b.map(f32::to_bits),
                "face {fi} (nível {kf}), {novo:?}: a conversão não devolveu a amostra guardada"
            );
            conferidas += 1;
        };
        if cantos.len() == 3 {
            for i in 0..=l {
                for j in 0..=(l - i) {
                    let k = l - i - j;
                    ver(
                        sitio_tri(l, i, j, k),
                        sitio_tri(lf, i * passo, j * passo, k * passo),
                    );
                }
            }
        } else {
            for j in 0..=l {
                for i in 0..=l {
                    ver(sitio_quad(l, i, j), sitio_quad(lf, i * passo, j * passo));
                }
            }
        }
    }
    assert!(conferidas > 100, "CONTROLO: só {conferidas} nós conferidos");
}

/// ⭐⭐ **GATE — UM PLANO JÁ UNIFORME SAI AO BIT.** A conversão corre em todo
/// load de um ficheiro graduado, e é idempotente para quem não precisa dela.
#[test]
fn um_plano_uniforme_sai_ao_bit() {
    let (faces, g) = graduado(3, |_| 3);
    assert!(
        g.lado_uniforme().is_some(),
        "CONTROLO: esta fixtura é uniforme"
    );
    let u = g
        .uniformizada(faces.iter().map(|f| &f[..]))
        .expect("as faces descrevem o plano");
    assert_eq!(
        u.amostras()
            .iter()
            .map(|c| c.map(f32::to_bits))
            .collect::<Vec<_>>(),
        g.amostras()
            .iter()
            .map(|c| c.map(f32::to_bits))
            .collect::<Vec<_>>()
    );
}

/// ⭐⭐ **GATE — UMA LISTA QUE NÃO DESCREVE O PLANO É RECUSADA**, nas duas
/// maneiras: faces a menos, e uma face com outro número de cantos.
#[test]
fn uma_lista_que_nao_descreve_o_plano_e_recusada() {
    let (faces, g) = graduado(1, |i| (i % 3) as u8);
    assert!(
        g.uniformizada(faces.iter().take(faces.len() - 1).map(|f| &f[..]))
            .is_none(),
        "uma face a menos"
    );
    let mut troca = faces.clone();
    let q = troca
        .iter()
        .position(|f| f.len() == 4)
        .expect("a grelha tem quads");
    troca[q].truncate(3);
    assert!(
        g.uniformizada(troca.iter().map(|f| &f[..])).is_none(),
        "um quad lido como triângulo"
    );
    assert!(
        g.uniformizada(faces.iter().map(|f| &f[..])).is_some(),
        "CONTROLO: a lista certa passa"
    );
}
