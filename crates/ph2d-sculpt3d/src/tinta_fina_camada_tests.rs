//! ⭐⭐⭐ **Gates do traço sobre uma CAMADA** — o plano emprestado com o canal
//! de opacidade (`ph2d_mesh_colors::alfa`, `docs/3D/30` §11): cor
//! pré-multiplicada, a mesma lei nas quatro componentes.

use super::*;
use crate::Symmetry;
use crate::tinta_fina::TintaDoTraco;
use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_colors::Tinta;

const COR: [f32; 3] = [0.9, 0.2, 0.1];

fn plano(mesh: &Mesh, nivel: u8) -> Tinta {
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    match mesh.colors() {
        Some(c) => Tinta::semeada(c, faces(), nivel),
        None => Tinta::nova(mesh.vert_count(), faces(), nivel),
    }
}

/// Dois dabs do verbo `verb` sobre o plano `t`, centrados em `+z`.
fn traco(mesh: &mut Mesh, verb: Verb, t: Tinta) -> Tinta {
    let brush = Brush {
        verb,
        radius: 0.45,
        strength: 1.0,
        color: COR,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(mesh);
    s.tinta_fina = Some(TintaDoTraco::nova(t, 0));
    let path = [0.06, 0.0, 0.0];
    for i in 0..2u8 {
        let c = [path[0] * f32::from(i), 0.0, 1.0];
        let dab = Dab {
            path,
            ..Dab::at(c, 0.45, c)
        };
        s.dab(mesh, &brush, &dab, Symmetry::default());
    }
    s.tinta_fina
        .take()
        .map(TintaDoTraco::entregar)
        .expect("plano")
}

fn bits(a: &[[f32; 3]]) -> Vec<[u32; 3]> {
    a.iter().map(|c| c.map(f32::to_bits)).collect()
}

/// ⭐⭐⭐⭐ **GATE — Numa camada OPACA o traço é o de hoje, AO BIT**, nos três
/// verbos de cor (o que pinta e os dois que leem o anel), e a opacidade fica
/// em `1`. É a prova de que a lei com alfa É a lei de antes quando `a = 1`.
#[test]
fn numa_camada_opaca_o_traco_e_o_de_hoje_ao_bit() {
    for verb in [Verb::Paint, Verb::Blur, Verb::SmearColor] {
        let mut a = shapes::uv_sphere(16, 24, 1.0);
        crate::canal_de_teste::semeia_cor(&mut a);
        let mut b = a.clone();
        let p0 = plano(&a, 2);
        let sem = traco(&mut a, verb, p0.clone());
        let mut com = plano(&b, 2);
        let n = com.amostras().len();
        assert!(com.com_alfa(Some(vec![1.0; n])));
        let com = traco(&mut b, verb, com);
        assert_ne!(
            bits(sem.amostras()),
            bits(p0.amostras()),
            "{verb:?}: o CONTROLO, o traço pintou"
        );
        assert_eq!(
            bits(com.amostras()),
            bits(sem.amostras()),
            "{verb:?}: a cor"
        );
        let pior = com
            .alfa()
            .expect("canal")
            .iter()
            .map(|x| (x - 1.0).abs())
            .fold(0.0, f32::max);
        assert!(pior <= 1e-6, "{verb:?}: a opacidade saiu de 1 ({pior})");
    }
}

/// ⭐⭐⭐ **GATE — Numa camada TRANSPARENTE, pintar é pousar por cima**: onde
/// o traço chegou, a cor DIREITA é a do pincel e a opacidade é a cobertura
/// (com orla macia); onde não chegou, nada mudou.
#[test]
fn numa_camada_transparente_pintar_e_pousar_por_cima() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let mut t = plano(&m, 2);
    let n = t.amostras().len();
    t.amostras_mut().fill([0.0; 3]);
    assert!(t.com_alfa(Some(vec![0.0; n])));
    let t = traco(&mut m, Verb::Paint, t);
    let alfa = t.alfa().expect("canal");
    let (mut tocadas, mut orla) = (0, 0);
    for (c, &a) in t.amostras().iter().zip(alfa) {
        if a == 0.0 {
            assert_eq!(*c, [0.0; 3], "fora do traço nada muda");
            continue;
        }
        tocadas += 1;
        if a < 0.99 {
            orla += 1;
        }
        for k in 0..3 {
            assert!(
                (c[k] / a - COR[k]).abs() < 1e-5,
                "a cor direita é a do pincel ({c:?} a {a})"
            );
        }
    }
    assert!(
        tocadas > 20 && orla > 0,
        "o CONTROLO: o traço pintou ({tocadas}) com orla ({orla})"
    );
}

/// ⭐⭐⭐ **GATE — O anel mistura PRÉ-MULTIPLICADO**: um desfoque na fronteira
/// entre tinta vermelha opaca e o vazio espalha a OPACIDADE, e a cor direita
/// continua vermelha — a média de cores direitas escureceria a borda (o
/// halo preto que toda camada mal composta tem).
#[test]
fn o_anel_mistura_pre_multiplicado_e_nao_escurece_a_borda() {
    let mut m = shapes::uv_sphere(16, 24, 1.0);
    let mut t = plano(&m, 2);
    let n = t.amostras().len();
    // metade da peça (`x > 0`) vermelha opaca, a outra vazia
    let mut alfa = vec![0.0f32; n];
    let lados: Vec<bool> = {
        let mut v = vec![false; n];
        let pos = m.positions().to_vec();
        for (fi, face) in m.faces().iter().enumerate() {
            let cantos = face.verts();
            let lado = t.lado_da_face(fi) as f32;
            if cantos.len() == 3 {
                t.para_cada_amostra_tri(fi, cantos, |idx, (i, j, k)| {
                    let w = [i as f32 / lado, j as f32 / lado, k as f32 / lado];
                    let x: f32 = (0..3).map(|c| pos[cantos[c] as usize][0] * w[c]).sum();
                    v[idx as usize] = x > 0.0;
                });
            } else {
                t.para_cada_amostra_quad(fi, cantos, |idx, (i, j)| {
                    let w = crate::tinta_fina::bilinear(i as f32 / lado, j as f32 / lado);
                    let x: f32 = (0..4).map(|c| pos[cantos[c] as usize][0] * w[c]).sum();
                    v[idx as usize] = x > 0.0;
                });
            }
        }
        v
    };
    for (i, &d) in lados.iter().enumerate() {
        t.amostras_mut()[i] = if d { [1.0, 0.0, 0.0] } else { [0.0; 3] };
        alfa[i] = if d { 1.0 } else { 0.0 };
    }
    assert!(t.com_alfa(Some(alfa)));
    let t = traco(&mut m, Verb::Blur, t);
    let alfa = t.alfa().expect("canal");
    let mut meio = 0;
    for (c, &a) in t.amostras().iter().zip(alfa) {
        if a > 0.0 {
            assert!(
                (c[0] / a - 1.0).abs() < 1e-5 && c[1] == 0.0 && c[2] == 0.0,
                "vermelho direito: {c:?} a {a}"
            );
        }
        if a > 0.01 && a < 0.99 {
            meio += 1;
        }
    }
    assert!(
        meio > 0,
        "o CONTROLO: o desfoque espalhou a opacidade pela fronteira"
    );
}

/// ⭐⭐ **GATE — O balde numa camada transparente pousa a cor OPACA** onde a
/// máscara deixa (sem máscara: em toda a parte).
#[test]
fn o_balde_numa_camada_pousa_opaco() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let mut t = plano(&m, 1);
    let n = t.amostras().len();
    t.amostras_mut().fill([0.0; 3]);
    assert!(t.com_alfa(Some(vec![0.0; n])));
    assert_eq!(crate::preenche::preenche_plano(&mut t, &m, COR), Ok(true));
    assert!(t.alfa().expect("canal").iter().all(|&a| a == 1.0));
    assert!(t.amostras().iter().all(|c| *c == COR));
}
