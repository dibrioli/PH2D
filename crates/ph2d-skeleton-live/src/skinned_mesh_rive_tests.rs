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
use super::serpentina_tests::serpentina_para_teste;

fn p50(v: &mut [f64]) -> f64 {
    v.sort_by(f64::total_cmp);
    v.get(v.len() / 2).copied().unwrap_or(0.0)
}

/// SONDA — as duas leis lado a lado, na dobra que o dono fotografou.
#[test]
fn diag_d_a_lei_do_rive_contra_a_de_hoje() {
    println!("\n{:=<92}", "");
    println!("SONDA · A LEI DO RIVE (só pontos de controlo) CONTRA A DE HOJE (ajuste das alças)");
    println!("{:=<92}", "");
    println!(
        "{:>6} | {:<26} | {:>10} | {:>9} {:>9} | {:>7} {:>7} | {:>7} {:>7}",
        "dobra", "lei", "serpent.", "ouro p90", "máx", "κ+ p90", "máx", "quebra", "máx"
    );
    for graus in [30.0_f32, 60.0, 90.0, 120.0] {
        let mut p = b_palco(true);
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let rest = b_amostra_com(&p.fonte, DENSO);
        let ouro = ideal_denso(&p, &pele, &rest, ph2d_vec_skin::curva::lei_c1_activa());
        let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
        for (rot, path) in [
            ("HOJE (ajuste + conciliação)", p.produto(true, true)),
            ("RIVE (só pontos)", p.produto(false, true)),
        ] {
            let v = b_amostra_com(&path, DENSO);
            let (_, b, c) = b_perfil(&v, &ouro);
            let (kp, km) = b_excesso_de_curvatura(&rest, &v, &ouro);
            let mut q = b_quebra_nos(&path);
            let qmax = q.iter().copied().fold(0.0_f64, f64::max);
            println!(
                "{graus:>5}° | {rot:<26} | {:>10.6} | {b:>9.5} {c:>9.5} | {kp:>7.2} {km:>7.2} | {:>7.3} {qmax:>7.3}",
                serpentina_para_teste(&rest, &v),
                p50(&mut q),
            );
        }
        let (_, b, c) = b_perfil(&chao, &ouro);
        let (kp, km) = b_excesso_de_curvatura(&rest, &chao, &ouro);
        println!(
            "{graus:>5}° | {:<26} | {:>10.6} | {b:>9.5} {c:>9.5} | {kp:>7.2} {km:>7.2} | {:>7} {:>7}",
            "o CHÃO do modelo",
            serpentina_para_teste(&rest, &chao),
            "—",
            "—"
        );
    }
    println!("{:=<92}", "");
}

/// SONDA — o defeito que o AJUSTE foi construído para curar, re-medido com os nós de hoje.
///
/// Em 2026-09-19 a F30 mediu *«pintar peso entre os vértices não faz nada»* como `0,000000` pelo
/// caminho dos pontos de controlo. Isso foi medido com a contagem de nós de então.
#[test]
fn diag_d_uma_mancha_entre_dois_nos_move_a_arte() {
    use ph2d_skeleton_ecs::{CorreccaoDePeso, SkinBind};
    let base = b_palco(true);
    let cozido = base.fonte.cooked();
    let (v, _) = cozido.contour(0).expect("contorno");
    let n = v.len();
    // O espaçamento típico entre nós, e o MEIO da aresta mais comprida do miolo.
    let mut esp: Vec<f64> = (0..n - 1)
        .map(|k| (v[k + 1].anchor[0] - v[k].anchor[0]).hypot(v[k + 1].anchor[1] - v[k].anchor[1]))
        .collect();
    let (mut kmax, mut lmax) = (0usize, 0.0_f64);
    for (k, &d) in esp.iter().enumerate() {
        if d > lmax {
            (kmax, lmax) = (k, d);
        }
    }
    let meio = [
        f64::midpoint(v[kmax].anchor[0], v[kmax + 1].anchor[0]),
        f64::midpoint(v[kmax].anchor[1], v[kmax + 1].anchor[1]),
    ];
    println!("\n{:=<92}", "");
    println!(
        "SONDA · UMA MANCHA ENTRE DOIS NÓS — {n} nós, espaçamento p50 {:.4}, maior aresta {lmax:.4}",
        p50(&mut esp)
    );
    println!(
        "mancha no meio da aresta {kmax} em [{:.3}, {:.3}]",
        meio[0], meio[1]
    );
    println!("{:=<92}", "");
    println!(
        "{:>7} | {:<20} | {:>12} {:>12}",
        "raio", "lei", "máx move", "p50 move"
    );
    for raio in [0.20_f64, 0.40, 0.80, 1.60] {
        for (rot, curva) in [("RIVE (só pontos)", false), ("HOJE (ajuste)", true)] {
            let mut p = b_palco(true);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            p.dobra_em_s(90.0);
            let sem = b_amostra_com(&p.produto(curva, true), DENSO);
            let osso = {
                let b = p.sim.world().get::<SkinBind>(p.alvo).expect("pele");
                b.tendons[0].bone
            };
            {
                let mut b = p.sim.world_mut().get_mut::<SkinBind>(p.alvo).expect("pele");
                b.correcoes.push(CorreccaoDePeso {
                    bone: osso,
                    centro: meio,
                    raio,
                    especie: ph2d_skeleton::Especie::Soma(1.0),
                });
            }
            let com = b_amostra_com(&p.produto(curva, true), DENSO);
            let mut d: Vec<f64> = sem
                .iter()
                .zip(&com)
                .map(|(a, b)| (a[0] - b[0]).hypot(a[1] - b[1]))
                .collect();
            let mx = d.iter().copied().fold(0.0_f64, f64::max);
            println!("{raio:>7.2} | {rot:<20} | {mx:>12.6} {:>12.6}", p50(&mut d));
        }
    }
    println!("{:=<92}", "");
}

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
