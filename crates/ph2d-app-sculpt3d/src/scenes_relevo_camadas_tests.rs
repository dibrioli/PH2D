//! A cena `=55` tem o FENÓMENO: a faixa de cima cruza riscas da base, e as
//! duas leis que o roteiro ensina (profundidade `0` e `Level`) mudam a peça ali.

use super::{NIVEL, faixa, relevo_da_base};
use crate::pilha_da_peca::PilhaDaPeca;

/// ⭐ **Onde a faixa cruza as riscas: com a de cima a `0` a peça é a textura de
/// baixo, com `Level` é a lomba lisa — ao bit —, e com `Add` não é nenhuma.**
#[test]
fn a_faixa_da_cena_cruza_as_riscas_e_as_duas_leis_se_veem() {
    let mesh = crate::scenes::tinta_fina::peca();
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    let tinta = ph2d_mesh_colors::Tinta::nova(mesh.vert_count(), faces(), NIVEL);
    let xs = crate::vizinhanca_da_peca::posicoes(&tinta, &mesh);
    let mut p = PilhaDaPeca::de_tinta(&tinta);
    let base = p.base().expect("base");
    let n = xs.len();
    let riscas: Vec<[f32; 2]> = xs.iter().map(|&x| relevo_da_base(x)).collect();
    let (cor, lomba): (Vec<[u8; 4]>, Vec<[f32; 2]>) = xs.iter().map(|&x| faixa(x)).unzip();
    assert!(p.pinta_camada(base, &vec![[200; 4]; n], Some(riscas.clone())));
    let cima = p.nova_camada("Layer 2").expect("cima");
    assert!(p.pinta_camada(cima, &cor, Some(lomba.clone())));

    let dentro: Vec<usize> = (0..n).filter(|&i| lomba[i][1] > 0.0).collect();
    assert!(
        dentro.len() > 2_000,
        "a faixa tem amostras ({})",
        dentro.len()
    );
    assert!(dentro.len() < n / 4, "a faixa é uma FAIXA, não a bola");
    let (lo, hi) = dentro
        .iter()
        .map(|&i| riscas[i][0])
        .fold((f32::MAX, f32::MIN), |(a, b), h| (a.min(h), b.max(h)));
    assert!(hi - lo > 0.01, "por baixo da faixa há riscas ({lo}..{hi})");

    let add = p.relevo_composto().expect("relevo");
    let mut muda = p.pilha().clone();
    muda.set_impasto_depth(cima, 0.0);
    p.troca_metadado(muda).expect("profundidade");
    let zero = p.relevo_composto().expect("relevo");
    let mut muda = p.pilha().clone();
    muda.set_impasto_depth(cima, 1.0);
    muda.toggle_impasto_composite(cima);
    p.troca_metadado(muda).expect("level");
    let level = p.relevo_composto().expect("relevo");
    for &i in &dentro {
        assert_eq!(
            zero[i][0].to_bits(),
            riscas[i][0].to_bits(),
            "a 0: a textura de baixo"
        );
        assert_eq!(
            level[i][0].to_bits(),
            lomba[i][0].to_bits(),
            "Level: a lomba enterra"
        );
    }
    // CONTROLO: o Add soma as duas — menos no fundo de uma risca, onde ela é ~`1e-10`.
    let somam = dentro.iter().filter(|&&i| add[i][0] != level[i][0]).count();
    assert!(
        somam * 10 > dentro.len() * 9,
        "o Add difere do Level em {somam} de {}",
        dentro.len()
    );
    assert_eq!(1u32 << NIVEL, 16, "o roteiro diz `16x`");
}
