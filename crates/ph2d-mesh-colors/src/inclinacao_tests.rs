//! Os gates da [INCLINAÇÃO POR AMOSTRA](crate::inclinacao) — irmão (`#[path]`)
//! do `lib.rs`.
//!
//! ⚠️ A fixtura é uma grelha PLANA mas NÃO alinhada aos eixos (uma rotação com
//! corte e inclinação em `z`): com ela alinhada, um `∇u` trocado com um `∇v`
//! leria igual. A altura é uma função LISA do ponto no objecto, logo a
//! inclinação verdadeira é conhecida — a componente tangente de `∇h`.

use crate::inclinacao::Inclinacoes;
use crate::{Tinta, sitio_quad};

const N: u32 = 3;

/// `3 × 3` células, a última fila partida em triângulos — as duas formas.
fn grelha() -> (Vec<[f32; 3]>, Vec<Vec<u32>>) {
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
    let mut pos = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            pos.push(mapa([i as f32, j as f32]));
        }
    }
    (pos, faces)
}

/// O plano da grelha no objecto — nem alinhado, nem ortogonal.
fn mapa([x, y]: [f32; 2]) -> [f32; 3] {
    [0.9 * x + 0.3 * y, -0.2 * x + 1.1 * y, 0.25 * x + 0.15 * y]
}

fn altura(p: [f32; 3]) -> f32 {
    0.2 * (1.7 * p[0] + 0.4 * p[2]).sin() * (1.3 * p[1]).cos()
}

/// A inclinação VERDADEIRA: `∇h` sem a componente da normal do plano.
fn verdade(p: [f32; 3]) -> [f32; 3] {
    let (s, c) = (
        (1.7 * p[0] + 0.4 * p[2]).sin(),
        (1.7 * p[0] + 0.4 * p[2]).cos(),
    );
    let (sy, cy) = ((1.3 * p[1]).sin(), (1.3 * p[1]).cos());
    let g = [0.2 * 1.7 * c * cy, -0.2 * 1.3 * s * sy, 0.2 * 0.4 * c * cy];
    let (a, b) = (mapa([1.0, 0.0]), mapa([0.0, 1.0]));
    let n = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let nn = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    let d = (g[0] * n[0] + g[1] * n[1] + g[2] * n[2]) / nn;
    [g[0] - d * n[0], g[1] - d * n[1], g[2] - d * n[2]]
}

/// Um plano com a [`altura`] pousada em cada amostra, onde ela cai.
fn plano(nivel: u8) -> (Vec<[f32; 3]>, Vec<Vec<u32>>, Tinta) {
    let (pos, faces) = grelha();
    let mut t = Tinta::nova(pos.len(), faces.iter().map(|f| &f[..]), nivel);
    let mut alt = vec![[0.0f32, 1.0]; t.amostras().len()];
    for (f, c) in faces.iter().enumerate() {
        let l = t.lado_da_face(f);
        let p = |k: usize| pos[c[k] as usize];
        if c.len() == 3 {
            t.para_cada_amostra_tri(f, c, |i, ijk| {
                alt[i as usize][0] =
                    altura(crate::amostragem::posicao_tri(p(0), p(1), p(2), l, ijk));
            });
        } else {
            t.para_cada_amostra_quad(f, c, |i, ij| {
                alt[i as usize][0] = altura(crate::amostragem::posicao_quad(
                    [p(0), p(1), p(2), p(3)],
                    l,
                    ij,
                ));
            });
        }
    }
    t.relevo_mut().copy_from_slice(&alt);
    (pos, faces, t)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// A lei POR CÉLULA (a do `tinta.wgsl` até 02/10): a derivada exacta da
/// bilinear, nos parâmetros, levada ao objecto pelos lados do quad plano.
fn por_celula(t: &Tinta, face: usize, c: &[u32], pos: &[[f32; 3]], uv: [f32; 2]) -> [f32; 3] {
    let l = t.lado_da_face(face);
    let lf = l as f32;
    let (cu, cv) = (uv[0] * lf, uv[1] * lf);
    let (i, j) = (
        (cu.floor() as u32).min(l - 1),
        (cv.floor() as u32).min(l - 1),
    );
    let (fu, fv) = (cu - i as f32, cv - j as f32);
    let h = |i, j| t.altura(t.indice_de(face, c, sitio_quad(l, i, j)) as usize);
    let (ha, hb, hd, he) = (h(i, j), h(i + 1, j), h(i + 1, j + 1), h(i, j + 1));
    let hu = lf * ((1.0 - fv) * (hb - ha) + fv * (hd - he));
    let hv = lf * ((1.0 - fu) * (he - ha) + fu * (hd - hb));
    // No quad plano em paralelogramo, `∂p/∂u = b − a` e `∂p/∂v = d − a`; o
    // gradiente é `hu·e_u + hv·e_v` com `(e_u, e_v)` a base DUAL.
    let p = |k: usize| pos[c[k] as usize];
    let du = [p(1)[0] - p(0)[0], p(1)[1] - p(0)[1], p(1)[2] - p(0)[2]];
    let dv = [p(3)[0] - p(0)[0], p(3)[1] - p(0)[1], p(3)[2] - p(0)[2]];
    let (uu, uv_, vv) = (
        du[0] * du[0] + du[1] * du[1] + du[2] * du[2],
        du[0] * dv[0] + du[1] * dv[1] + du[2] * dv[2],
        dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2],
    );
    let det = uu * vv - uv_ * uv_;
    let (a, b) = ((hu * vv - hv * uv_) / det, (hv * uu - hu * uv_) / det);
    [
        a * du[0] + b * dv[0],
        a * du[1] + b * dv[1],
        a * du[2] + b * dv[2],
    ]
}

/// ⭐⭐⭐ **GATE — a inclinação lida é CONTÍNUA ao atravessar as células, e a lei
/// por célula reprova a MESMA régua** (o CONTROLO de que a régua mede o degrau).
///
/// Uma linha de `4 000` passos atravessa o quad do meio na diagonal: o maior
/// salto entre dois passos vizinhos, na lei por amostra e na por célula.
#[test]
fn a_inclinacao_lida_e_continua_e_a_lei_por_celula_reprova_a_mesma_regua() {
    let (pos, faces, t) = plano(3);
    let inc = Inclinacoes::nova(&t, |f| &faces[f][..], &pos);
    let (f, c) = (4usize, &faces[4][..]);
    let ponto = |k: usize| {
        let s = k as f32 / 4000.0;
        [0.05 + 0.9 * s, 0.3 + 0.4 * s]
    };
    let mut salto = (0.0f32, 0.0f32);
    for k in 0..4000 {
        let (a, b) = (ponto(k), ponto(k + 1));
        let n = dist(
            t.inclinacao_quad(f, c, a, inc.por_amostra()),
            t.inclinacao_quad(f, c, b, inc.por_amostra()),
        );
        let o = dist(por_celula(&t, f, c, &pos, a), por_celula(&t, f, c, &pos, b));
        salto = (salto.0.max(n), salto.1.max(o));
    }
    eprintln!(
        "salto por amostra {:.3e} · por célula {:.3e}",
        salto.0, salto.1
    );
    assert!(
        salto.1 > 100.0 * salto.0,
        "a régua não separa as duas leis: por amostra {:.2e}, por célula {:.2e}",
        salto.0,
        salto.1
    );
    assert!(
        salto.0 < 5e-4,
        "a lei por amostra salta {:.2e} num passo",
        salto.0
    );
}

/// ⭐⭐⭐ **GATE — a inclinação por amostra é a VERDADEIRA**, nos dois degraus e
/// nas duas formas — sem esta metade, uma lei que devolvesse zero (contínua!)
/// passaria o gate da continuidade, e um `∇u` trocado também.
#[test]
fn a_inclinacao_por_amostra_e_a_da_superficie() {
    // ⚠️ A régua é o pior ponto do miolo das faces relativo ao maior declive
    //    da fixtura. Medido em 02/10: `0,0158` no nível 3 e `0,0040` no 4 — o
    //    erro cai `4×` quando a célula cai `2×` (ordem DOIS), e os tectos são o
    //    dobro do medido. A lei por célula é de ordem um.
    let mut razoes = Vec::new();
    for (nivel, tecto) in [(3u8, 0.03f32), (4, 0.008)] {
        let (pos, faces, t) = plano(nivel);
        let inc = Inclinacoes::nova(&t, |f| &faces[f][..], &pos);
        let mut pior = 0.0f32;
        let mut maior = 0.0f32;
        for (f, c) in faces.iter().enumerate() {
            let p = |k: usize| pos[c[k] as usize];
            for a in 1..8 {
                for b in 1..8 {
                    let (x, y) = (a as f32 / 8.0, b as f32 / 8.0);
                    let (lida, onde) = if c.len() == 3 {
                        if x + y >= 1.0 {
                            continue;
                        }
                        let bar = [1.0 - x - y, x, y];
                        let w = |e: usize| bar[0] * p(0)[e] + bar[1] * p(1)[e] + bar[2] * p(2)[e];
                        (
                            t.inclinacao_tri(f, c, bar, inc.por_amostra()),
                            [w(0), w(1), w(2)],
                        )
                    } else {
                        let w = |e: usize| {
                            (p(0)[e] * (1.0 - x) + p(1)[e] * x) * (1.0 - y)
                                + (p(3)[e] * (1.0 - x) + p(2)[e] * x) * y
                        };
                        (
                            t.inclinacao_quad(f, c, [x, y], inc.por_amostra()),
                            [w(0), w(1), w(2)],
                        )
                    };
                    let v = verdade(onde);
                    pior = pior.max(dist(lida, v));
                    maior = maior.max(dist(v, [0.0; 3]));
                }
            }
        }
        eprintln!(
            "nível {nivel}: erro {pior:.4} · declive {maior:.4} · razão {:.4}",
            pior / maior
        );
        assert!(
            pior < tecto * maior,
            "nível {nivel}: erro {pior:.4} contra o declive {maior:.4} (tecto {tecto})"
        );
        razoes.push(pior / maior);
    }
    assert!(
        razoes[1] * 3.0 < razoes[0],
        "o erro não cai com a célula: {razoes:?}"
    );
}

/// ⭐⭐ **GATE — os dois lados de uma aresta da malha lêem a MESMA inclinação**:
/// quad com quad e quad com triângulo. É a continuidade que a lei por célula
/// não tinha nem DENTRO da face.
#[test]
fn os_dois_lados_de_uma_aresta_leem_a_mesma_inclinacao() {
    let (pos, faces, t) = plano(3);
    let inc = Inclinacoes::nova(&t, |f| &faces[f][..], &pos);
    let g = inc.por_amostra();
    for k in 0..=20 {
        let s = k as f32 / 20.0;
        // A aresta 5→9: lado `b→c` da face 3 (`u = 1`), lado `d→a` da face 4 (`u = 0`).
        let a = t.inclinacao_quad(3, &faces[3], [1.0, s], g);
        let b = t.inclinacao_quad(4, &faces[4], [0.0, s], g);
        assert!(dist(a, b) < 1e-5, "quad|quad em s={s}: {a:?} contra {b:?}");
        // A aresta 9→10: lado `d→c` do quad 4 (`v = 1`), lado `a→b` do triângulo 8.
        let a = t.inclinacao_quad(4, &faces[4], [s, 1.0], g);
        let b = t.inclinacao_tri(8, &faces[8], [1.0 - s, s, 0.0], g);
        assert!(dist(a, b) < 1e-5, "quad|tri em s={s}: {a:?} contra {b:?}");
    }
}

/// ⭐⭐⭐ **GATE — a atualização por PEDAÇOS dá a inteira**, e diz TODAS as
/// amostras que mudou (as que o device tem de receber). Duas rondas sobre o
/// mesmo estado, para a época das marcas ser exercida.
#[test]
fn a_atualizacao_por_pedacos_da_a_inteira_e_diz_o_que_mudou() {
    let (pos, faces, mut t) = plano(3);
    let cantos = |f: usize| &faces[f][..];
    let mut inc = Inclinacoes::nova(&t, cantos, &pos);
    let mut mudadas = Vec::new();
    // Sujas dos três blocos: um vértice, amostras de aresta, interiores — e um
    // repetido.
    let n = t.amostras().len() as u32;
    let v = t.topologia().verts() as u32;
    for ronda in 0..2u32 {
        let sujas = [5u32, v + 3 + ronda, v + 7, n - 1 - ronda, n / 2, n / 2];
        for &s in &sujas {
            t.relevo_mut()[s as usize][0] += 0.37 + ronda as f32;
        }
        let antes = inc.por_amostra().to_vec();
        inc.atualiza(&t, &cantos, &pos, &sujas, &mut mudadas);
        let inteira = Inclinacoes::nova(&t, cantos, &pos);
        for (i, (a, b)) in inc
            .por_amostra()
            .iter()
            .zip(inteira.por_amostra())
            .enumerate()
        {
            assert!(
                dist(*a, *b) < 1e-5,
                "ronda {ronda}, amostra {i}: {a:?} contra {b:?}"
            );
            if dist(antes[i], *b) > 1e-6 {
                assert!(
                    mudadas.contains(&(i as u32)),
                    "a amostra {i} mudou e não foi dita"
                );
            }
        }
    }
}

/// ⭐ **Sem relevo, a inclinação é nula** e nada se acumula.
#[test]
fn sem_relevo_a_inclinacao_e_nula() {
    let (pos, faces) = grelha();
    let t = Tinta::nova(pos.len(), faces.iter().map(|f| &f[..]), 2);
    let inc = Inclinacoes::nova(&t, |f| &faces[f][..], &pos);
    assert!(inc.serve(&t));
    assert!(inc.por_amostra().iter().all(|g| *g == [0.0; 3]));
}

/// ⭐⭐⭐ **GATE — a peça que muda de FORMA atualiza-se pelos VÉRTICES**: o
/// índice da amostra de um vértice é o dele, e as faces dele são as que a
/// mudança tocou. Mover dois vértices (um de quad, um da fronteira com os
/// triângulos) e atualizar por eles dá a inteira.
#[test]
fn mover_vertices_e_atualizar_por_eles_da_a_inteira() {
    let (mut pos, faces, t) = plano(3);
    let cantos = |f: usize| &faces[f][..];
    let mut inc = Inclinacoes::nova(&t, cantos, &pos);
    for v in [5usize, 10] {
        pos[v][2] += 0.4;
        pos[v][0] -= 0.1;
    }
    let mut mudadas = Vec::new();
    inc.atualiza(&t, &cantos, &pos, &[5, 10], &mut mudadas);
    let inteira = Inclinacoes::nova(&t, cantos, &pos);
    let mut mudou = 0;
    for (i, (a, b)) in inc
        .por_amostra()
        .iter()
        .zip(inteira.por_amostra())
        .enumerate()
    {
        assert!(dist(*a, *b) < 1e-5, "amostra {i}: {a:?} contra {b:?}");
        mudou += usize::from(mudadas.contains(&(i as u32)));
    }
    assert!(
        mudou > 0,
        "o CONTROLO: mover vértices tinha de mudar inclinações"
    );
}

/// ⭐⭐ **GATE — o plano com POUCA altura nasce só das faces que a têm, e dá o
/// mesmo que refazer tudo** (o 1.º toque de impasto não paga o plano inteiro).
#[test]
fn um_plano_com_pouca_altura_nasce_igual_ao_inteiro() {
    let (pos, faces, mut t) = plano(3);
    let n = t.amostras().len();
    for (i, r) in t.relevo_mut().iter_mut().enumerate() {
        if i % 97 != 3 {
            r[0] = 0.0;
        }
    }
    let cantos = |f: usize| &faces[f][..];
    let esparsa = Inclinacoes::nova(&t, cantos, &pos);
    let mut inteira = esparsa.clone();
    inteira.recalcula(&t, &cantos, &pos);
    let mut nao_nulas = 0;
    for (i, (a, b)) in esparsa
        .por_amostra()
        .iter()
        .zip(inteira.por_amostra())
        .enumerate()
    {
        assert!(
            dist(*a, *b) < 1e-5,
            "amostra {i} de {n}: {a:?} contra {b:?}"
        );
        nao_nulas += usize::from(*b != [0.0; 3]);
    }
    assert!(
        nao_nulas > 0,
        "o CONTROLO: a fixtura tinha de ter inclinação"
    );
}

/// ⭐⭐⭐ **GATE — as faces em PARALELO dão o mesmo AO BIT que numa thread só**:
/// cada face soma no rascunho dela e os rascunhos juntam-se pela ordem das
/// faces. No nível 6 a fixtura tem `12 × 4 096` células — acima do limiar, logo
/// o caminho paralelo é o que corre (e uma atualização também).
#[test]
fn as_faces_em_paralelo_dao_o_mesmo_ao_bit() {
    let (pos, faces, mut t) = plano(6);
    let cantos = |f: usize| &faces[f][..];
    let mut uma = Inclinacoes::com_threads(&t, cantos, &pos, 1);
    let mut oito = Inclinacoes::com_threads(&t, cantos, &pos, 8);
    assert_eq!(uma.por_amostra(), oito.por_amostra(), "a peça inteira");
    let n = t.amostras().len() as u32;
    let sujas: Vec<u32> = (0..n).step_by(5).collect();
    for &s in &sujas {
        t.relevo_mut()[s as usize][0] += 0.1;
    }
    let (mut m1, mut m8) = (Vec::new(), Vec::new());
    uma.atualiza(&t, &cantos, &pos, &sujas, &mut m1);
    oito.atualiza(&t, &cantos, &pos, &sujas, &mut m8);
    assert_eq!(m1, m8);
    assert_eq!(uma.por_amostra(), oito.por_amostra(), "a atualização");
}
