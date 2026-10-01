//! Os gates do [RELEVO](crate::relevo) — irmão (`#[path]`) do `lib.rs`.
//!
//! ⚠️ A régua da leitura é a COR: as três passam pelos mesmos pesos, logo uma
//! altura (e um corpo) igual a um canal da cor tem de se ler IGUAL AO BIT a esse
//! canal, em todo ponto. Uma segunda redacção da interpolação divergia aqui.
//! A régua de ENDEREÇO (um nó da retícula lê a amostra que lá está) vem ao
//! lado, porque a primeira sozinha aprovaria duas leituras erradas iguais.

use crate::{Tinta, sitio_quad, sitio_tri};

fn valor(i: usize) -> f32 {
    let h = (i as u32).wrapping_mul(2_654_435_761);
    (h & 0xffff) as f32 / 65_535.0 - 0.5
}

/// `3 × 3` células, a última fila partida em triângulos — as duas formas.
fn grelha() -> (usize, Vec<Vec<u32>>) {
    const N: u32 = 3;
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

/// Um plano com cor e relevo: a altura é o canal VERMELHO da cor e o corpo o
/// VERDE, os três distintos por amostra.
fn com_relevo(nivel: u8) -> (Vec<Vec<u32>>, Tinta) {
    let (verts, faces) = grelha();
    let mut t = Tinta::nova(verts, faces.iter().map(|f| &f[..]), nivel);
    for (i, a) in t.amostras_mut().iter_mut().enumerate() {
        *a = [valor(i), valor(i + 7), valor(i + 13)];
    }
    let rg: Vec<[f32; 2]> = t.amostras().iter().map(|c| [c[0], c[1]]).collect();
    t.relevo_mut().copy_from_slice(&rg);
    (faces, t)
}

/// ⭐ **Sem impasto, o relevo não existe e não pesa um byte.**
#[test]
fn sem_relevo_nao_ha_alturas_e_nao_se_paga_nada() {
    let (verts, faces) = grelha();
    let mut t = Tinta::nova(verts, faces.iter().map(|f| &f[..]), 3);
    assert!(!t.tem_relevo());
    assert!(t.relevo().is_none());
    assert_eq!(t.espessura(0), [0.0; 2]);
    assert_eq!(t.espessura_tri(6, &faces[6], [0.2, 0.3, 0.5]), [0.0; 2]);
    let sem = t.footprint_bytes();
    let n = t.amostras().len();
    assert!(
        t.relevo_mut().iter().all(|&h| h == [0.0; 2]),
        "nasce a zero"
    );
    assert!(t.tem_relevo());
    assert!(
        t.footprint_bytes() >= sem + n * size_of::<[f32; 2]>(),
        "o relevo conta no peso da peça (a fila de desfazer soma bytes)"
    );
}

/// ⭐⭐⭐ **A altura e o corpo lêem-se pelos MESMOS pesos da cor, ao bit** — em
/// pontos dentro das faces das duas formas e nos quatro degraus mais baixos.
#[test]
fn a_altura_le_com_os_pesos_da_cor_ao_bit() {
    for nivel in 0..=3 {
        let (faces, t) = com_relevo(nivel);
        let mut pontos = 0;
        for (fi, f) in faces.iter().enumerate() {
            for s in 0..9 {
                let a = (s as f32 + 0.37) / 9.3;
                let b = ((s * 5 % 9) as f32 + 0.21) / 9.7;
                if f.len() == 3 {
                    let bar = [a * (1.0 - b), b, (1.0 - a) * (1.0 - b)];
                    let (e, c) = (t.espessura_tri(fi, f, bar), t.cor_tri(fi, f, bar));
                    assert_eq!(e[0].to_bits(), c[0].to_bits(), "nível {nivel}, face {fi}");
                    assert_eq!(
                        e[1].to_bits(),
                        c[1].to_bits(),
                        "corpo: nível {nivel}, face {fi}"
                    );
                    assert_eq!(t.altura_tri(fi, f, bar).to_bits(), e[0].to_bits());
                } else {
                    let uv = [a, b];
                    let (e, c) = (t.espessura_quad(fi, f, uv), t.cor_quad(fi, f, uv));
                    assert_eq!(e[0].to_bits(), c[0].to_bits(), "nível {nivel}, face {fi}");
                    assert_eq!(
                        e[1].to_bits(),
                        c[1].to_bits(),
                        "corpo: nível {nivel}, face {fi}"
                    );
                    assert_eq!(t.altura_quad(fi, f, uv).to_bits(), e[0].to_bits());
                }
                pontos += 1;
            }
        }
        assert!(pontos > 50, "a fixtura tem de ter pontos: {pontos}");
    }
}

/// ⭐⭐ **Um nó da retícula lê a amostra que lá está** — a régua de ENDEREÇO,
/// sem interpolação nenhuma (a de cima sozinha aprovaria duas leituras erradas
/// iguais).
#[test]
fn um_no_da_reticula_le_a_altura_que_la_esta() {
    let (faces, t) = com_relevo(2);
    let a = t.relevo().expect("com relevo");
    for (fi, f) in faces.iter().enumerate() {
        let l = t.lado_da_face(fi);
        let lf = l as f32;
        if f.len() == 3 {
            for i in 0..=l {
                for j in 0..=(l - i) {
                    let k = l - i - j;
                    let idx = t.indice_de(fi, f, sitio_tri(l, i, j, k)) as usize;
                    let bar = [i as f32 / lf, j as f32 / lf, k as f32 / lf];
                    let lida = t.espessura_tri(fi, f, bar);
                    for e in 0..2 {
                        assert!(
                            (lida[e] - a[idx][e]).abs() <= 1e-6,
                            "face {fi} ({i},{j},{k})"
                        );
                    }
                }
            }
        } else {
            for j in 0..=l {
                for i in 0..=l {
                    let idx = t.indice_de(fi, f, sitio_quad(l, i, j)) as usize;
                    let lida = t.espessura_quad(fi, f, [i as f32 / lf, j as f32 / lf]);
                    for e in 0..2 {
                        assert!((lida[e] - a[idx][e]).abs() <= 1e-6, "face {fi} ({i},{j})");
                    }
                }
            }
        }
    }
}

/// ⛔ **Um relevo com o tamanho errado é RECUSADO e nada muda.**
#[test]
fn um_relevo_do_tamanho_errado_e_recusado() {
    let (_, mut t) = com_relevo(1);
    let antes = t.clone();
    let n = t.amostras().len();
    assert!(!t.com_relevo(Some(vec![[1.0; 2]; n + 1])));
    assert_eq!(t, antes, "a recusa não mexeu em nada");
    assert!(t.com_relevo(Some(vec![[0.5, 0.25]; n])));
    assert_eq!(t.altura(3), 0.5);
    assert_eq!(t.corpo(3), 0.25);
    assert!(t.com_relevo(None), "retirar o relevo é sempre possível");
    assert!(!t.tem_relevo());
}

/// ⭐ **Levar o plano a um degrau só leva o relevo com ele** — e um plano sem
/// relevo continua sem.
#[test]
fn a_uniformizada_leva_o_relevo() {
    let (verts, faces) = grelha();
    let ks: Vec<u8> = (0..faces.len()).map(|f| (f % 3) as u8).collect();
    let mut t =
        Tinta::graduada(verts, faces.iter().map(|f| &f[..]), &ks, 2).expect("descreve a malha");
    for (i, a) in t.amostras_mut().iter_mut().enumerate() {
        *a = [valor(i), valor(i + 3), 0.0];
    }
    let sem = t
        .uniformizada(faces.iter().map(|f| &f[..]))
        .expect("descreve");
    assert!(!sem.tem_relevo(), "sem relevo não se inventa um");

    let rg: Vec<[f32; 2]> = t.amostras().iter().map(|c| [c[0], c[1]]).collect();
    t.relevo_mut().copy_from_slice(&rg);
    let u = t
        .uniformizada(faces.iter().map(|f| &f[..]))
        .expect("descreve");
    let a = u.relevo().expect("o relevo foi com o plano");
    for (i, c) in u.amostras().iter().enumerate() {
        assert_eq!(a[i][0].to_bits(), c[0].to_bits(), "amostra {i}");
        assert_eq!(a[i][1].to_bits(), c[1].to_bits(), "corpo: amostra {i}");
    }
}
