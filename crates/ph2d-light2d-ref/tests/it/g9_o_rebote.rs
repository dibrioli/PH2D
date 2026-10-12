//! ⭐ **G9 no gabarito** (W5c, a lei do plano §2.4): o rebote contra uma fórmula FECHADA, a sala
//! vermelha e verde que tinge o chão branco, e a convergência do passo `h` e da série.

use std::f64::consts::PI;

use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_ref::{ALBEDO_MAX, OpcoesDoRebote, Rebote, fluencia, luz_em};

use crate::comum::{cena, disco};

fn caixa(x0: f32, y0: f32, x1: f32, y1: f32, albedo: [f32; 3]) -> Oclusor {
    Oclusor {
        vertices: vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]],
        opacidade: 1.0,
        albedo,
    }
}

fn op(passo_m: f64, rebotes: u32) -> OpcoesDoRebote {
    OpcoesDoRebote { passo_m, rebotes }
}

/// A parede longa sob o céu: de `x = −50` a `50`, topo em `y = 0`.
fn meio_ceu(albedo: f32) -> CenaDeLuz {
    let mut c = cena(vec![], vec![caixa(-50.0, -1.0, 50.0, 0.0, [albedo; 3])]);
    c.ceu = [1.0; 3];
    c
}

/// ⭐ **A fórmula fechada.** No topo da parede o ponto `x⁺` vê o céu na metade de cima (`π·L`) e a
/// própria face na de baixo (`π·L_p`): `L_p = a·(π·L + π·L_p)/2π` ⇒ `L_p = a·L/(2 − a)`, igual em toda
/// a face. A `d` acima do centro: `F = π·L + 2·atan(d/50)·L + (π − 2·atan(d/50))·L_p`.
/// (Controlo: sem rebote a resposta muda de `≈ 1,03` — mil vezes o teto.)
#[test]
fn o_meio_ceu_diante_de_uma_parede_longa_e_a_formula_fechada() {
    for (a, d) in [(0.5_f32, 1.0_f64), (0.25, 3.0), (1.0, 0.5)] {
        let c = meio_ceu(a);
        let r = Rebote::resolver(&c, op(0.5, 40));
        let a_ = f64::from(a.min(ALBEDO_MAX));
        let lp = a_ / (2.0 - a_);
        let lasca = 2.0 * (d / 50.0).atan();
        let esperado = PI + lasca + (PI - lasca) * lp;
        let f = r.fluencia([0.0, d])[0];
        assert!(
            (f - esperado).abs() <= 1e-6,
            "a = {a}, d = {d}: {f} × {esperado}"
        );
        let sem = fluencia(&c, [0.0, d])[0];
        assert!((sem - (PI + lasca)).abs() <= 1e-9, "sem rebote {sem}");
    }
}

/// Sem rebotes, a série é a fluência de sempre — o MESMO bit (a mesma `linha`).
#[test]
fn sem_rebotes_e_a_fluencia_de_sempre() {
    let c = sala();
    let r = Rebote::resolver(&c, op(0.05, 0));
    for p in pontos_do_chao() {
        assert_eq!(r.fluencia(p), fluencia(&c, p), "{p:?}");
    }
    assert_eq!(r.residuo(), 0.0);
}

/// A sala fechada (S8): chão e teto brancos, parede esquerda VERMELHA, direita VERDE, uma lâmpada
/// branca perto do teto. Interior `[0, 4] × [0, 2,5]`, paredes de 0,2 m.
pub fn sala() -> CenaDeLuz {
    let branco = [1.0; 3];
    cena(
        vec![disco([2.0, 2.2], 0.1, [1.0; 3], f32::INFINITY)],
        vec![
            caixa(-0.2, -0.2, 4.2, 0.0, branco),
            caixa(-0.2, 2.5, 4.2, 2.7, branco),
            caixa(-0.2, 0.0, 0.0, 2.5, [1.0, 0.0, 0.0]),
            caixa(4.0, 0.0, 4.2, 2.5, [0.0, 1.0, 0.0]),
        ],
    )
}

/// Junto ao chão: perto da vermelha, no meio, perto da verde.
pub fn pontos_do_chao() -> [[f64; 2]; 3] {
    [[0.3, 0.05], [2.0, 0.05], [3.7, 0.05]]
}

fn razao(l: [f64; 3], a: usize, b: usize) -> f64 {
    l[a] / l[b]
}

/// ⭐ **O chão branco tinge-se** perto de cada parede com a cor DELA; no meio, as duas empatam por
/// simetria. Sem rebote (o controlo) a luz é branca em todo ponto.
#[test]
fn a_sala_vermelha_e_verde_tinge_o_chao() {
    let c = sala();
    let r = Rebote::resolver(&c, op(0.02, 200));
    let [vermelho, meio, verde] = pontos_do_chao().map(|p| r.luz_em(p));
    let [sv, sm, sg] = pontos_do_chao().map(|p| luz_em(&c, p));
    for s in [sv, sm, sg] {
        assert!(
            s[0] == s[1] && s[1] == s[2],
            "sem rebote a luz é branca: {s:?}"
        );
    }
    assert!(
        razao(vermelho, 0, 1) > 1.3,
        "perto da vermelha {vermelho:?}"
    );
    assert!(razao(verde, 1, 0) > 1.3, "perto da verde {verde:?}");
    assert!(
        (razao(meio, 0, 1) - 1.0).abs() < 1e-3,
        "no meio as duas empatam {meio:?}"
    );
    // E o rebote ACRESCENTA luz (nunca a tira).
    for (com, sem) in [(vermelho, sv), (meio, sm), (verde, sg)] {
        assert!((0..3).all(|k| com[k] >= sem[k]), "{com:?} × {sem:?}");
    }
}

/// ⭐ **O passo converge**: metade do `h` muda a luz do chão muito menos que o próprio rebote (medido:
/// `h` 0,04 × 0,02 ≤ 8,5e-5 do rebote; o teto 2e-4).
#[test]
fn o_passo_dos_elementos_converge() {
    let c = sala();
    let grosso = Rebote::resolver(&c, op(0.04, 200));
    let fino = Rebote::resolver(&c, op(0.02, 200));
    for p in pontos_do_chao() {
        let (g, f, s) = (grosso.luz_em(p), fino.luz_em(p), luz_em(&c, p));
        for k in 0..3 {
            let rebote = f[k] - s[k];
            assert!(
                (g[k] - f[k]).abs() <= 2e-4 * rebote.abs().max(1e-12) + 1e-12,
                "{p:?} canal {k}: h 0,04 {} × h 0,02 {} (rebote {rebote})",
                g[k],
                f[k]
            );
        }
    }
}

/// ⭐ **A série converge**, e o teto do albedo segura uma sala fechada TODA branca (com `a = 1` ela
/// divergiria): o resíduo cai a cada rebote e um albedo acima do teto responde o mesmo que o teto.
#[test]
fn a_serie_converge_e_o_teto_segura_a_sala_branca() {
    let mut branca = sala();
    for o in &mut branca.oclusores {
        o.albedo = [1.0; 3];
    }
    let mut no_teto = branca.clone();
    for o in &mut no_teto.oclusores {
        o.albedo = [ALBEDO_MAX; 3];
    }
    let poucos = Rebote::resolver(&branca, op(0.05, 50));
    let muitos = Rebote::resolver(&branca, op(0.05, 400));
    assert!(
        muitos.residuo() < 1e-9 * poucos.residuo(),
        "{} → {}",
        poucos.residuo(),
        muitos.residuo()
    );
    let teto = Rebote::resolver(&no_teto, op(0.05, 400));
    let p = pontos_do_chao()[1];
    assert_eq!(muitos.fluencia(p), teto.fluencia(p));
    assert!(muitos.fluencia(p).iter().all(|f| f.is_finite()));
}

/// G10 do rebote: duas corridas, o mesmo bit.
#[test]
fn o_rebote_e_deterministico() {
    let c = sala();
    let a = Rebote::resolver(&c, op(0.05, 30));
    let b = Rebote::resolver(&c, op(0.05, 30));
    for p in pontos_do_chao() {
        assert_eq!(a.fluencia(p), b.fluencia(p));
    }
}

/// ⭐ W5k — **o translúcido devolve a sua parte, `α·a`** (contínuo em `α`; até a W5k o 0,99 não
/// refletia nada e a sala perdia a cor de uma vez) e o que brilha não reflete.
#[test]
fn o_translucido_reflete_a_sua_parte_e_o_que_brilha_nao() {
    let com_emissor = |opacidade: f32| {
        let mut c = sala();
        for o in &mut c.oclusores {
            o.opacidade = opacidade;
        }
        c.emissores.push(Emissor::Poligono {
            vertices: vec![[1.0, 1.0], [1.2, 1.0], [1.2, 1.2], [1.0, 1.2]],
            radiancia: [0.5; 3],
            alcance_m: f32::INFINITY,
        });
        c
    };
    let (c99, c1) = (com_emissor(0.99), com_emissor(1.0));
    let r99 = Rebote::resolver(&c99, op(0.05, 20));
    let r1 = Rebote::resolver(&c1, op(0.05, 20));
    assert_eq!(r99.n_elementos(), Rebote::resolver(&sala(), op(0.05, 20)).n_elementos());
    for p in pontos_do_chao() {
        let (a, b) = (r99.fluencia(p), r1.fluencia(p));
        for c in 0..3 {
            assert!((a[c] - b[c]).abs() <= 0.02 * b[c], "{p:?} canal {c}: {a:?} a 0,99 × {b:?} a 1");
        }
    }
    // Controlo: a 0,99 o chão continua tingido pela parede vermelha (o rebote não sumiu).
    let p = pontos_do_chao()[0];
    assert!(razao(r99.luz_em(p), 0, 1) > 1.3);
    assert!(razao(fluencia(&c99, p), 0, 1) < 1.01);
}

/// Sonda (à mão, `--ignored --nocapture`): os números que os portões acima leem.
#[test]
#[ignore = "sonda: imprime"]
fn sonda_do_rebote() {
    let c = sala();
    for (h, n) in [
        (0.08, 200),
        (0.04, 200),
        (0.02, 200),
        (0.02, 10),
        (0.02, 30),
    ] {
        let t = std::time::Instant::now();
        let r = Rebote::resolver(&c, op(h, n));
        let ls = pontos_do_chao().map(|p| r.luz_em(p));
        println!(
            "h {h} rebotes {n}: {} elementos, resíduo {:.3e}, {:.0} ms · chão {ls:?}",
            r.n_elementos(),
            r.residuo(),
            t.elapsed().as_secs_f64() * 1e3
        );
    }
    println!("sem rebote: {:?}", pontos_do_chao().map(|p| luz_em(&c, p)));
}
