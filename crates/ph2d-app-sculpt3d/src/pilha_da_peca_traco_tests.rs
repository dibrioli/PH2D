//! Gates do traço sobre a pilha (`docs/3D/30` §11, W2).

use super::*;
use ph2d_mesh::shapes;

/// ⭐⭐⭐⭐ **GATE — A ida e volta ao byte é a IDENTIDADE** em todo píxel com
/// opacidade, e um transparente é `[0; 4]`. É o que deixa o desfazer guardar
/// a cópia `f32` e devolver os bytes exactos da camada.
#[test]
fn a_ida_e_volta_ao_byte_e_a_identidade() {
    for a in 1..=255u8 {
        for v in 0..=255u8 {
            for px in [
                [v, 0, 0, a],
                [0, v, 0, a],
                [0, 0, v, a],
                [v, 255 - v, v / 2, a],
            ] {
                let (c, al) = de_bytes(px);
                assert_eq!(para_bytes(c, al), px, "{px:?}");
            }
        }
    }
    assert_eq!(
        para_bytes(de_bytes([9, 9, 9, 0]).0, 0.0),
        [0; 4],
        "um transparente não guarda cor"
    );
}

/// A peça da fixtura: um plano semeado e a pilha de UMA camada dele, mais uma
/// camada de cima transparente (activa).
fn peca() -> (Tinta, PilhaDaPeca, LayerId, LayerId) {
    let mut m = shapes::octahedron(1.0);
    for i in 0..m.vert_count() {
        m.colors_mut()[i] = [0.2 + 0.1 * i as f32, 0.4, 0.6];
    }
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let mut t = Tinta::semeada(m.colors().expect("cor"), faces(), 4);
    t.relevo_mut()[3] = [0.01, 1.0];
    let mut p = PilhaDaPeca::de_tinta(&t);
    let base = p.base().expect("base");
    let cima = p.nova_camada("cima").expect("camada");
    p.define_modo(cima, BlendMode::Multiply);
    (t, p, base, cima)
}

/// ⭐⭐⭐ **GATE — O traço desce à camada ACTIVA e só a ela**: a cópia de
/// trabalho é a camada (transparente aqui), as amostras sujas descem em
/// RGBA8, a base fica intacta — e o relevo desce à BASE.
#[test]
fn o_traco_desce_a_camada_activa_e_so_a_ela() {
    let (t, mut p, base, cima) = peca();
    let base_antes = p.plano(base).expect("base").clone();
    let (id, mut w) = p.trabalho_da_activa(&t).expect("a activa é um raster");
    assert_eq!(id, cima);
    assert!(
        w.alfa().expect("canal").iter().all(|&a| a == 0.0),
        "a camada nova é transparente"
    );
    let idx = [5u32, 6, 7, 40];
    for &i in &idx {
        w.amostras_mut()[i as usize] = [0.3, 0.15, 0.05];
        w.define_opacidade(i as usize, 0.5);
        w.relevo_mut()[i as usize] = [0.02, 0.5];
    }
    p.recebe_do_traco(id, &w, &idx);
    assert_eq!(p.fim_do_traco(), Some(cima));
    let n = p.amostras();
    let px = p.plano(cima).expect("cima").rgba8(n);
    for &i in &idx {
        let o = i as usize * 4;
        assert_eq!(
            &px[o..o + 4],
            &para_bytes([0.3, 0.15, 0.05], 0.5),
            "amostra {i}"
        );
    }
    assert_eq!(
        px.iter().filter(|&&b| b != 0).count(),
        idx.len() * 4,
        "só as sujas"
    );
    let b = p.plano(base).expect("base");
    assert_eq!(b.rgba8(n), base_antes.rgba8(n), "a base fica intacta");
    assert_eq!(
        b.relevo().map(|r| r[5]),
        Some([0.02, 0.5]),
        "o relevo desce à base"
    );
    assert_eq!(
        b.relevo().map(|r| r[3]),
        Some([0.01, 1.0]),
        "e o de antes fica"
    );
}

/// ⭐⭐⭐⭐ **GATE — Recompor só as amostras sujas é o pedaço da peça inteira**:
/// depois de mudar a camada de cima em amostras espalhadas (corridas e
/// isoladas), a recomposição incremental dá, AO BIT, o que a peça inteira
/// recomposta dá — e não toca nas outras.
#[test]
fn recompor_amostras_e_o_pedaco_da_peca_inteira() {
    let (t, mut p, _, cima) = peca();
    let mut incremental = t.clone();
    p.pinta_tinta(&mut incremental, || panic!("a base é opaca"));
    let idx: Vec<u32> = (100..140).chain([7, 9, 300, 301, 302, 999]).collect();
    let (_, mut w) = p.trabalho_da_activa(&t).expect("activa");
    for &i in &idx {
        w.amostras_mut()[i as usize] = [0.5, 0.25, 0.125];
        w.define_opacidade(i as usize, 0.75);
    }
    p.recebe_do_traco(cima, &w, &idx);
    let antes = incremental.clone();
    p.compoe_amostras(&idx, &mut incremental, || panic!("a base é opaca"));
    let mut inteira = t.clone();
    p.pinta_tinta(&mut inteira, || panic!("a base é opaca"));
    let bits = |a: &[[f32; 3]]| a.iter().map(|c| c.map(f32::to_bits)).collect::<Vec<_>>();
    assert_eq!(bits(incremental.amostras()), bits(inteira.amostras()));
    let mudaram = antes
        .amostras()
        .iter()
        .zip(incremental.amostras())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        mudaram,
        idx.len(),
        "o CONTROLO: o Multiply mudou as sujas e só elas"
    );
}

/// ⭐⭐ **GATE — As trocas do desfazer devolvem o que lá estava**, e trocar
/// duas vezes é a identidade (janela, relevo e plano inteiro).
#[test]
fn as_trocas_do_desfazer_sao_involucoes() {
    let (_, mut p, base, cima) = peca();
    let original = p.clone();
    let idx = [1u32, 2, 3];
    let antes = p
        .troca_janela(cima, &idx, &[[1, 2, 3, 4]; 3])
        .expect("janela");
    assert_ne!(p, original);
    p.troca_janela(cima, &idx, &antes).expect("de volta");
    let r = p.troca_relevo(&idx, &[[0.5, 0.5]; 3]).expect("relevo");
    p.troca_relevo(&idx, &r).expect("de volta");
    let plano = p.copia_do_plano(base).expect("plano");
    let velho = p.troca_plano(base, vec![7; plano.len()]).expect("troca");
    p.troca_plano(base, velho).expect("de volta");
    assert_eq!(p, original);
    assert!(
        p.troca_janela(LayerId(777), &idx, &antes).is_none(),
        "camada que não existe"
    );
    assert!(
        p.troca_plano(base, vec![0; 3]).is_none(),
        "plano do tamanho errado"
    );
}
