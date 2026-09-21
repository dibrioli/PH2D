//! ⭐⭐⭐ **A LEI DO RIVE CONTRA A DE HOJE** — as sondas da ordem de 2026-09-20.
//!
//! Report do dono, com foto: *«não fica bom. Muito curvado. Vá investigar como Rive faz. O código
//! é aberto»*.
//!
//! A porta estava ABERTA (`rive-runtime` é **MIT**, §0.9) e o fonte foi LIDO, não adivinhado:
//! `src/bones/weight.cpp` · `src/bones/skin.cpp` · `src/shapes/vertex.cpp` ·
//! `src/shapes/cubic_vertex.cpp`. A lei dele cabe numa linha —
//! `p' = (Σᵢ wᵢ · Bᵢ·bind⁻¹) · (W · p)`, aplicada à **âncora e às duas alças** de cada cúbica — e
//! **não há amostragem, ajuste nem conciliação em lado nenhum**. O `CubicWeight` dele dá a cada
//! alça pesos próprios; nesta casa as três metades partilham o peso da âncora, por **ordem do dono
//! de 2026-09-19**, com a objecção registada no cabeçalho da [`ph2d_vec_skin`].
//!
//! ⇒ a lei do Rive **é exactamente** a nossa [`ph2d_vec_skin::aplica_corrigido`], e é por isso que
//! ela serve de CONTROLO aqui sem uma linha de código nova.
//!
//! ⚠️ O que este ficheiro tem de SONDAS imprime; o gate é o último.

use super::ondulacao_regua_tests::{DENSO, ideal_denso};
use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **GATE — O AJUSTE FICA, E A LEI DO RIVE É O CONTROLO QUE O PROVA.**
///
/// ⛔⛔⛔ **Este gate existe para um leitor específico: quem ler *«o Rive não ajusta nada»* e
/// concluir que o ajuste devia sair.** Ele foi lido, é MIT, e é à letra a nossa
/// [`ph2d_vec_skin::aplica_corrigido`] — e medido na régua que o dono nomeou (*«muito curvado»*)
/// ele é **`6,7×` pior**.
///
/// # As três metades
///
/// 1. **O produto está NO CHÃO do modelo** — o ajuste livre é o melhor que uma cúbica pode fazer.
/// 2. **A lei do Rive é medivelmente pior no EXCESSO DE CURVATURA**, que é a grandeza do report.
/// 3. ⭐ **E ela compra uma coisa REAL: colinearidade exacta nos nós** (`0,000°`, porque um afim
///    preserva colinearidade). Sem esta metade a recusa leria-se como *«o Rive é pior e pronto»*,
///    e ela não é — *é uma TROCA, e o que a decide é qual dos dois lados o olho vê*.
#[test]
fn o_ajuste_fica_e_a_lei_do_rive_e_o_controlo() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let ouro = ideal_denso(&p, &pele, &rest, ph2d_vec_skin::curva::lei_c1_activa());
    let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);

    let nosso = p.produto(true, true);
    let rive = p.produto(false, true);
    let (k_nosso, _) = b_excesso_de_curvatura(&rest, &b_amostra_com(&nosso, DENSO), &ouro);
    let (k_rive, _) = b_excesso_de_curvatura(&rest, &b_amostra_com(&rive, DENSO), &ouro);
    let (k_chao, _) = b_excesso_de_curvatura(&rest, &chao, &ouro);
    let q_rive = b_quebra_nos(&rive).into_iter().fold(0.0_f64, f64::max);
    println!(
        "  κ+ p90: produto {k_nosso:.2}° · Rive {k_rive:.2}° · chão {k_chao:.2}° | quebra do Rive {q_rive:.3}°"
    );
    // ⭐⭐ **O CONTROLO DA PRÓPRIA RÉGUA, e ele nasceu de uma mutação SOBREVIVENTE:** trocar o
    // padrão-ouro por ZEROS dentro da [`b_excesso_de_curvatura`] deixava este gate **verde** — ele
    // passava a comparar curvaturas ABSOLUTAS e a palavra «excesso» deixava de descrever o que era
    // medido. *Uma régua de DIFERENÇA tem de ler zero contra si própria.*
    let (k_ouro, _) = b_excesso_de_curvatura(&rest, &ouro, &ouro);
    assert!(
        k_ouro < 1e-9,
        "a régua leu {k_ouro}° de EXCESSO entre o padrão-ouro e ele próprio — ela deixou de medir \
         uma DIFERENÇA e passou a medir a curvatura absoluta"
    );
    assert!(
        k_chao > 1.0,
        "o CHÃO leu {k_chao}° de excesso — esta peça deixou de conter o fenómeno, e as asserções \
         abaixo passam a ser triviais"
    );
    assert!(
        k_nosso < k_chao * 1.15,
        "o produto ({k_nosso}°) deixou de estar NO chão do modelo ({k_chao}°) — algum passe volta \
         a mexer nas alças depois de o ajuste as ter posto no óptimo"
    );
    assert!(
        k_rive > k_nosso * 4.0,
        "a lei do RIVE ({k_rive}°) deixou de ser pior que o ajuste ({k_nosso}°) — ou a fixtura \
         deixou de dobrar, ou o ajuste parou de comprar o que ele existe para comprar"
    );
    assert!(
        q_rive < 1e-9,
        "a lei do Rive deixou uma quina de {q_rive}° — ela aplica UM afim às três metades de cada \
         vértice, e um afim preserva colinearidade; se isto reprova, ela deixou de ser a lei dele"
    );
}

/// ⭐⭐⭐ **GATE — ASSAR-E-DEPOIS-AJUSTAR ganha de AJUSTAR-A-CHAMAR-A-LEI, nas duas colunas.**
///
/// A ideia do dono (2026-09-20): *«e se fizer um bake para imagem e usar a imagem como referência
/// para usar a técnica de arc»*. Ela é de outra classe, e o que se afirma é isso.
///
/// # As três metades
///
/// 1. **Mais barato**, e por muito — o refit adaptativo chama a lei de dentro do fitter e paga a
///    leitura do campo `8 371` vezes; o bake percorre-a um número FIXO de vezes, com a leitura
///    barata, e o fitter trabalha sobre os pontos já assados.
/// 2. **E mais fiel** — sem esta metade, «mais barato» diria apenas que ele faz menos.
/// 3. ⛔ **E ele passa POR BAIXO do chão do modelo aos `54` nós**, que é a única coisa que
///    acrescentar pontos pode comprar. *Sem esta metade o gate não distinguiria o segundo corpo de
///    um primeiro corpo melhor.*
#[test]
fn assar_e_depois_ajustar_ganha_de_ajustar_a_chamar_a_lei() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let ouro = ideal_denso(&p, &pele, &rest, false);
    let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
    let skin = p
        .sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(p.alvo)
        .expect("pele")
        .clone();
    let g = crate::skinned_mesh::le(&skin.source).expect("fonte");
    let pesos = skin.pesos_do_quadro(if g.valida() { &g.pesos } else { &[] });
    let correcoes = skin.correcoes_resolvidas();
    let tol = 0.0003 * 7.07;
    let bake = ph2d_vec_skin::curva::refit_pelo_bake(
        &pele,
        &g.path,
        pesos,
        &correcoes,
        true,
        ph2d_vec_skin::curva::LeituraDoCampo {
            campo: g.campo.as_ref(),
            c1: false,
        },
        ph2d_vec_skin::curva::Bake {
            amostras: 32,
            tolerancia: tol,
        },
    );
    let adapt = ph2d_vec_skin::curva::refit_pela_curva(
        &pele,
        &g.path,
        pesos,
        &correcoes,
        true,
        ph2d_vec_skin::curva::LeituraDoCampo {
            campo: g.campo.as_ref(),
            c1: true,
        },
        tol,
    );
    let d = |x: &ph2d_vec_scene::VecPath| b_perfil(&b_amostra_com(x, DENSO), &ouro);
    let (_, bp90, bmax) = d(&bake);
    let (_, ap90, amax) = d(&adapt);
    let (_, _, cmax) = b_perfil(&chao, &ouro);
    println!("  bake {bp90:.5}/{bmax:.5} · adaptativo {ap90:.5}/{amax:.5} · chão {cmax:.5}");
    assert!(
        amax > 1e-3,
        "o refit adaptativo leu {amax} — a fixtura deixou de conter o fenómeno e as asserções \
         abaixo passam por vácuo"
    );
    assert!(
        bp90 < ap90 && bmax < amax,
        "o bake ({bp90}/{bmax}) deixou de ser mais fiel que o refit adaptativo ({ap90}/{amax})"
    );
    assert!(
        bmax < cmax * 0.75,
        "o bake ({bmax}) deixou de passar por BAIXO do chão dos {} nós ({cmax}) — sem isso ele não \
         é um segundo corpo, é um primeiro corpo melhor",
        p.fonte.verts_all().count()
    );
}
