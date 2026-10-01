//! ⭐⭐ **OS GATES DO RELEVO na janela do desfazer** — o impasto do Painter na
//! peça (`docs/3D/29`). Irmão (`#[path]`) do [`super`]; correm sem adaptador,
//! pela mesma razão dos gates do quarto canal.

use super::JanelaFina;
use ph2d_mesh::{Mesh, shapes};
use ph2d_mesh_colors::Tinta;
use ph2d_sculpt3d::tinta_fina::TintaDoTraco;

const COR: [f32; 3] = [0.9, 0.2, 0.1];

fn plano(mesh: &Mesh, nivel: u8) -> Tinta {
    let faces = || mesh.faces().iter().map(ph2d_mesh::Face::verts);
    Tinta::nova(mesh.vert_count(), faces(), nivel)
}

/// Um traço do Painter que pinta as amostras `idx` e, se `alt`, eleva-as
/// (com corpo cheio).
fn traco(t: Tinta, idx: &[u32], alt: Option<f32>) -> TintaDoTraco {
    let mut f = TintaDoTraco::nova(t, 0);
    for &i in idx {
        f.repinta(i, |_| COR);
        if let Some(h) = alt {
            f.eleva(i, |antes| [antes[0] + h, 1.0]);
        }
    }
    f
}

/// ⭐⭐⭐ **GATE — O `Ctrl+Z` de um traço de impasto devolve o RELEVO de antes,
/// e o `Ctrl+Shift+Z` repõe-no.** O CONTROLO de que a fixtura contém o
/// fenómeno: depois do traço o relevo existe e não é zero.
#[test]
fn o_desfazer_devolve_o_relevo_e_o_refazer_repoe_no() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let idx = [3u32, 40, 41, 97];
    let f = traco(plano(&m, 1), &idx, Some(0.02));
    let janela = JanelaFina::do_traco(&f).expect("o traço escreveu");
    let mut t = f.entregar();
    assert!(
        idx.iter().all(|&i| t.altura(i as usize) == 0.02),
        "o CONTROLO: o traço elevou as amostras"
    );

    let refazer = janela.troca(Some(&mut t)).expect("o mesmo plano");
    assert!(
        t.relevo()
            .expect("o plano tem relevo")
            .iter()
            .all(|&r| r == [0.0, 0.0]),
        "o desfazer devolveu a altura E o corpo de antes (zero)"
    );
    refazer.troca(Some(&mut t)).expect("o mesmo plano");
    assert!(
        idx.iter().all(|&i| t.espessura(i as usize) == [0.02, 1.0]),
        "o refazer repôs a altura e o corpo"
    );
}

/// ⭐⭐ **GATE — Uma pincelada de COR sobre uma peça com relevo não grava o
/// canal, e o desfazer dela não mexe no relevo.** Sem a pergunta
/// `relevo_mudou`, toda pincelada pagaria a janela das alturas — e ela só
/// passa por esta régua se o relevo de antes ficar INTACTO depois do desfazer.
#[test]
fn uma_pincelada_de_cor_nao_grava_o_relevo_nem_lhe_toca() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let mut t = plano(&m, 1);
    t.relevo_mut()[40] = [0.05, 0.5];
    let f = traco(t, &[40, 41], None);
    assert!(!f.relevo_mudou());
    let janela = JanelaFina::do_traco(&f).expect("o traço de cor escreveu");
    let so_cor = janela.bytes();
    let mut t = f.entregar();
    janela.troca(Some(&mut t)).expect("o mesmo plano");
    assert_eq!(
        t.espessura(40),
        [0.05, 0.5],
        "o desfazer da cor não mexeu no relevo"
    );

    // O CONTROLO da régua dos bytes: o mesmo traço COM impasto paga o canal.
    let f = traco(t, &[40, 41], Some(0.01));
    let janela = JanelaFina::do_traco(&f).expect("escreveu");
    assert!(
        janela.bytes() > so_cor,
        "o canal das alturas conta no tecto da história"
    );
}

/// ⭐ **GATE — Refazer um traço de impasto num plano SEM relevo cria-o.** É o
/// caso do plano reconstruído do mesmo degrau: a janela tem de caber nele.
#[test]
fn refazer_num_plano_sem_relevo_cria_o() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let f = traco(plano(&m, 1), &[5, 6], Some(0.03));
    let janela = JanelaFina::do_traco(&f).expect("escreveu");
    let mut t = f.entregar();
    let refazer = janela.troca(Some(&mut t)).expect("o mesmo plano");
    let mut limpo = plano(&m, 1);
    assert!(
        !limpo.tem_relevo(),
        "a fixtura: o plano de agora não tem relevo"
    );
    refazer.troca(Some(&mut limpo)).expect("o mesmo degrau");
    assert_eq!(limpo.altura(5), 0.03);
    assert_eq!(limpo.altura(6), 0.03);
}

/// ⭐⭐ **GATE — Um traço que só muda o CORPO (o alisar sobre o barro nu, que
/// tira tinta sem mexer na altura) é um traço que mudou o relevo**, e o
/// desfazer devolve o corpo de antes. Sem ele o registo leria só a altura e o
/// `Ctrl+Z` deixaria o corpo novo debaixo de uma altura velha.
#[test]
fn um_traco_que_so_muda_o_corpo_mudou_o_relevo() {
    let m = shapes::uv_sphere(8, 12, 1.0);
    let mut t = plano(&m, 1);
    t.relevo_mut()[40] = [0.05, 1.0];
    let mut f = TintaDoTraco::nova(t, 0);
    assert!(f.eleva(40, |antes| [antes[0], 0.25]));
    assert!(f.relevo_mudou(), "só o corpo mudou, e mudou");
    let janela = JanelaFina::do_traco(&f).expect("o traço escreveu");
    let mut t = f.entregar();
    janela.troca(Some(&mut t)).expect("o mesmo plano");
    assert_eq!(t.espessura(40), [0.05, 1.0], "o desfazer devolveu o corpo");
}
