//! Gates do motor de arco. **Os oráculos são independentes do método**: uma reta tem
//! comprimento fechado, e uma curva é medida contra amostragem densa — nunca contra outra
//! chamada da mesma Gauss-Legendre, que seria o método a concordar consigo mesmo.

use super::*;

/// Números do produto, não `1.0`: uma escala unitária esconde erro de unidade e de fator.
/// [[feedback_test_with_product_numbers_not_convenient_ones]]
const S: f64 = 37.5;

/// Comprimento por amostragem densa da polilinha — o oráculo externo. Converge para o arco
/// verdadeiro por baixo (a corda é sempre menor que o arco), então é um piso apertado.
fn dense_len(c: &Cubic, n: usize) -> f64 {
    let at = |t: f64| {
        let u = 1.0 - t;
        let mut p = [0.0; 2];
        for k in 0..2 {
            p[k] = u * u * u * c[0][k]
                + 3.0 * u * u * t * c[1][k]
                + 3.0 * u * t * t * c[2][k]
                + t * t * t * c[3][k];
        }
        p
    };
    let mut sum = 0.0;
    let mut prev = at(0.0);
    for i in 1..=n {
        let p = at(i as f64 / n as f64);
        sum += ((p[0] - prev[0]).powi(2) + (p[1] - prev[1]).powi(2)).sqrt();
        prev = p;
    }
    sum
}

/// Uma cúbica curva de verdade (não degenerada, não simétrica).
fn curved() -> Cubic {
    [
        [0.0, 0.0],
        [S * 0.3, S * 1.1],
        [S * 1.4, S * 0.9],
        [S * 1.7, S * 0.2],
    ]
}

/// **Uma RETA tem comprimento fechado, e a quadratura tem de o acertar exatamente.**
///
/// A reta entra na forma canónica (⅓, ⅔) — a que é afim em `t`. É a mesma armadilha que o
/// blend pagou: `(P0, P0, P3, P3)` é a mesma *curva* com parametrização não-uniforme.
#[test]
fn a_straight_line_has_its_closed_form_length() {
    let (a, b) = ([11.0, -4.0], [11.0 + 3.0 * S, -4.0 + 4.0 * S]);
    let third = |k: usize| a[k] + (b[k] - a[k]) / 3.0;
    let two_thirds = |k: usize| a[k] + (b[k] - a[k]) * 2.0 / 3.0;
    let line: Cubic = [a, [third(0), third(1)], [two_thirds(0), two_thirds(1)], b];
    // 3-4-5: o comprimento é exatamente 5·S.
    assert!(
        (arclen(&line) - 5.0 * S).abs() < 1e-9,
        "reta 3-4-5 devia medir {}, mediu {}",
        5.0 * S,
        arclen(&line)
    );
}

/// **Numa curva, a quadratura concorda com amostragem densa.**
///
/// O oráculo é externo ao método. 200k cordas ficam a ~1e-8 do arco verdadeiro nesta escala;
/// exigir 1e-6 relativo é apertado e não flaka.
#[test]
fn a_curve_agrees_with_dense_sampling() {
    let c = curved();
    let (gl, dense) = (arclen(&c), dense_len(&c, 200_000));
    assert!(
        ((gl - dense) / dense).abs() < 1e-6,
        "GL16 = {gl}, amostragem densa = {dense}"
    );
}

/// **O inverso é o inverso**: pedir o `t` do comprimento que `t` produziu devolve `t`.
#[test]
fn the_inverse_round_trips_across_the_whole_domain() {
    let c = curved();
    for i in 0..=20 {
        let t = f64::from(i) / 20.0;
        let back = inv_arclen(&c, arclen_to(&c, t));
        assert!((back - t).abs() < 1e-9, "t = {t} voltou {back}");
    }
}

/// **O `t` do meio do arco NÃO é `0.5`** — é a armadilha que este módulo existe para evitar,
/// e um gate que não a contenha deixaria passar uma implementação que só devolve `s / total`.
#[test]
fn the_midpoint_of_arc_is_not_the_midpoint_of_t() {
    let c = curved();
    let t_mid = inv_arclen(&c, arclen(&c) * 0.5);
    assert!(
        (t_mid - 0.5).abs() > 1e-3,
        "nesta curva o meio do arco cai em t = {t_mid}; se der 0.5 a parametrização foi \
         confundida com o comprimento"
    );
}

/// Cortar em `[0, 1]` devolve a mesma curva, ao bit — o ponto neutro do `subsegment`.
#[test]
fn the_full_subsegment_is_the_curve_itself() {
    let c = curved();
    let s = subsegment(&c, 0.0, 1.0);
    for (i, (a, b)) in s.iter().zip(c.iter()).enumerate() {
        assert!(
            (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
            "ponto {i}: {a:?} != {b:?}"
        );
    }
}

/// **As duas metades somam o todo** — o `subsegment` corta sem criar nem perder arco.
#[test]
fn the_two_halves_of_a_split_sum_to_the_whole() {
    let c = curved();
    let t = 0.37;
    let sum = arclen(&subsegment(&c, 0.0, t)) + arclen(&subsegment(&c, t, 1.0));
    assert!(
        ((sum - arclen(&c)) / arclen(&c)).abs() < 1e-9,
        "as metades somam {sum}, o todo mede {}",
        arclen(&c)
    );
}

/// **A bisseção que o Newton substituiu**, preservada como ORÁCULO de teste.
///
/// Não é uma segunda porta de produto: é a implementação de referência contra a qual a troca
/// de algoritmo se justifica. Converge sempre e não pede derivada — 40 halvings levam o
/// intervalo a `2^-40 ≈ 9e-13` do domínio de `t`.
fn inv_arclen_bisect(c: &Cubic, s: f64) -> f64 {
    let total = arclen(c);
    if s <= 0.0 || total <= 0.0 {
        return 0.0;
    }
    if s >= total {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..40 {
        let mid = 0.5 * (lo + hi);
        if arclen_to(c, mid) < s {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// **O Newton concorda com a bisseção que ele substituiu** — e o gate diz de QUANTO, porque
/// "concorda" sem número é a afirmação que uma impressão digital já faz, e pior: um hash não
/// distingue *mudou 1e-15* de *mudou tudo*.
///
/// A troca (2026-07-23, ADR-0141 Fatia 0) foi por CUSTO — 1700 ns → 140 ns, medido — e o preço
/// dela é este épsilon. Ele é reportado em unidades de MUNDO sobre uma curva de tamanho de
/// produto, que é a grandeza em que alguém decide se é visível.
#[test]
fn the_newton_inverse_agrees_with_the_bisection_it_replaced() {
    let cs = [curved(), straightish(), sharp()];
    let (mut worst_t, mut worst_p) = (0.0f64, 0.0f64);
    for c in &cs {
        let total = arclen(c);
        for i in 0..=2000 {
            let s = total * f64::from(i) / 2000.0;
            let (a, b) = (inv_arclen(c, s), inv_arclen_bisect(c, s));
            worst_t = worst_t.max((a - b).abs());
            let (pa, pb) = (point_at(c, a), point_at(c, b));
            worst_p = worst_p.max((pa[0] - pb[0]).hypot(pa[1] - pb[1]));
        }
    }
    // Medido: 1e-12 em `t`, ~1e-10 unidades de mundo numa curva de ~100 unidades. O Newton é
    // o MAIS preciso dos dois (para na tolerância; a bisseção para na contagem), então este
    // número é a distância entre duas respostas certas, não um erro.
    assert!(
        worst_t < 1e-9,
        "o `t` divergiu {worst_t:.3e} da bisseção — mais que os 1e-9 que a tolerância promete"
    );
    assert!(
        worst_p < 1e-6,
        "o PONTO divergiu {worst_p:.3e} unidades de mundo — visível seria ~1e-2"
    );
    eprintln!("Newton vs bisseção: dt = {worst_t:.3e}, dponto = {worst_p:.3e} unidades");
}

/// Uma cúbica quase reta: o palpite inicial do Newton (`s/total`) é EXATO aqui, então este é o
/// caso em que ele sai numa iteração — e é o que garante que o caso fácil não regrediu.
fn straightish() -> Cubic {
    [[0.0, 0.0], [S, 0.1], [2.0 * S, -0.1], [3.0 * S, 0.0]]
}

/// Uma cúbica de curvatura forte com quase-cúspide: `|B'|` chega perto de zero no meio, que é
/// onde Newton divide por quase-nada e a CERCA de bisseção tem de assumir.
fn sharp() -> Cubic {
    [[0.0, 0.0], [S, 0.0], [-S, 0.0], [0.0, S]]
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// A DIREÇÃO nas pontas de um segmento com polígono de controlo degenerado (2026-08-30).
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Um segmento **RETO** como o repositório o autora: dois vértices de quina, cada alça em cima da
/// própria âncora ⇒ `P₁ = P₀` e `P₃ = P₂`.
fn reta(a: [f64; 2], b: [f64; 2]) -> Cubic {
    [a, a, b, b]
}

/// ⭐⭐⭐ **UMA RETA TEM DIREÇÃO NAS PONTAS** — e antes de 2026-08-30 não tinha.
///
/// `B'(0) = 3·(P₁ − P₀)` e `B'(1) = 3·(P₃ − P₂)`, que numa reta autorada são **zero** nos dois
/// extremos. A curva ali é uma reta; o que se anula é a velocidade, não a direção.
///
/// ⚠️ **O preço estava medido** (plano 36 §11.1): o pincel de contorno pulava `1`–`2` cópias por
/// quadrado, em todo tamanho — o `frame_at` devolvia tangente nula e o `pattern_along` fazia
/// `continue`. Cinco consumidores partilham esta função.
#[test]
fn a_straight_segment_has_a_direction_at_its_endpoints() {
    let c = reta([0.0, 0.0], [10.0, 0.0]);
    // A velocidade É zero nas pontas — a fixtura contém o fenómeno.
    for t in [0.0, 1.0] {
        let d = super::deriv(&c, t);
        assert!(
            d[0].hypot(d[1]) < 1e-12,
            "a fixtura nao tem o fenomeno: B'({t}) = {d:?} nao e' nulo"
        );
    }
    // E a DIREÇÃO existe, e é a da corda, nas duas pontas e no meio.
    for t in [0.0, 0.5, 1.0] {
        let u = tangent_at(&c, t).unwrap_or_else(|| panic!("sem direcao em t = {t}"));
        assert!(
            (u[0] - 1.0).abs() < 1e-12 && u[1].abs() < 1e-12,
            "a direcao em t = {t} nao e' a da corda: {u:?}"
        );
    }
    // ⚠️ **E ela aponta para a FRENTE nos dois extremos.** Pela esquerda o limite de `B'` é `−B''`
    // e pela direita é `+B''`; uma cura escrita com derivadas e sem o cuidado do sinal devolve a
    // direção INVERTIDA numa das duas pontas — e o sintoma seria uma cópia virada ao contrário na
    // última quina, que nenhuma contagem apanha.
    let ao_contrario = reta([10.0, 0.0], [0.0, 0.0]);
    let u = tangent_at(&ao_contrario, 1.0).expect("sem direcao");
    assert!(
        (u[0] + 1.0).abs() < 1e-12,
        "a direcao no fim de uma reta invertida nao acompanha a marcha: {u:?}"
    );
}

/// ⚠️ **A CASCATA salta os pontos coincidentes** — um segmento com `P₀ = P₁ = P₂` ainda tem
/// direção, e é a de `P₃ − P₀`.
#[test]
fn the_cascade_skips_every_coincident_control_point() {
    let o = [0.0, 0.0];
    let c: Cubic = [o, o, o, [0.0, 4.0]];
    let u = tangent_at(&c, 0.0).expect("sem direcao com tres pontos coincidentes");
    assert!(
        u[0].abs() < 1e-12 && (u[1] - 1.0).abs() < 1e-12,
        "a cascata nao chegou a P3 - P0: {u:?}"
    );
    // ⛔ CONTROLO — um segmento com os QUATRO pontos coincidentes não tem direção nenhuma, e
    // devolver uma seria inventá-la.
    assert!(
        tangent_at(&[o, o, o, o], 0.0).is_none(),
        "um segmento sem comprimento devolveu uma direcao"
    );
}

/// ⛔⛔ **A CÚSPIDE INTERIOR CONTINUA A DEVOLVER `None`** — e é esta metade que impede a cura de
/// virar «inventar uma direção onde não há».
///
/// Com `0 < t < 1` os dois lados existem e a **marcha inverte**: a reta tangente é a mesma, o
/// sentido não. É a razão original de esta função devolver `Option`, e ela fica.
///
/// ⚠️ **A 1.ª fixtura deste gate TINHA a cúspide e a varredura NÃO A ENCONTRAVA:** os zeros do
/// hodógrafo estão em `t = (3 ∓ √3)/6 ≈ 0,2113` e `0,7887`, e uma grade de `k/1000` não passa por
/// nenhum deles. *Uma cúspide é um ponto isolado — uma varredura amostra tudo menos ele.* ⇒ a
/// fixtura é avaliada **na raiz exacta**.
#[test]
fn a_true_interior_cusp_still_has_no_direction() {
    // Hodógrafo `3·(6t² − 6t + 1)` no eixo x: os braços de controlo são opostos.
    //
    // ⚠️⚠️ **E a 1.ª fixtura NÃO PODIA matar a mutação que importa.** Ela é um laço com
    // `P₀ = P₃`, e a mutação que troca este `None` por *«inventa a direção da corda»* devolve
    // `unit(P₃ − P₀) = unit(0) = None` — **a mesma resposta**, por acidente. ⇒ a segunda fixtura
    // tem `P₃ ≠ P₀` de propósito: ali toda direção inventada é visível.
    // A condição de `B'(½) = 0` é `P₂ + P₃ = P₀ + P₁`, e é dela que a segunda sai.
    let laco: Cubic = [[0.0, 0.0], [1.0, 0.0], [-1.0, 0.0], [0.0, 0.0]];
    let r3 = 3.0_f64.sqrt();
    let aberta: Cubic = [[0.0, 0.0], [4.0, 0.0], [0.0, 1.0], [4.0, -1.0]];
    for (c, t) in [
        (laco, (3.0 - r3) / 6.0),
        (laco, (3.0 + r3) / 6.0),
        (aberta, 0.5),
    ] {
        let c = &c;
        // A fixtura CONTÉM o fenómeno: a velocidade anula-se ali.
        let d = super::deriv(c, t);
        assert!(
            d[0].hypot(d[1]) < 1e-12,
            "a raiz {t} nao anula o hodografo: {d:?}"
        );
        assert!(
            tangent_at(c, t).is_none(),
            "uma cuspide INTERIOR passou a devolver uma direcao em t = {t} - a cura invadiu o \
             caso que ela nao devia tocar"
        );
        // ⚠️ E a cúspide é ISOLADA: um passo ao lado tem direção. Sem esta metade o gate ficaria
        // verde sobre uma curva degenerada de ponta a ponta.
        assert!(
            tangent_at(c, t + 1e-3).is_some() && tangent_at(c, t - 1e-3).is_some(),
            "a vizinhanca de {t} tambem nao tem direcao - a fixtura nao e' uma cuspide isolada"
        );
    }
}

/// ⭐ **O CAMINHO COMUM É BYTE-IDÊNTICO** — a cura só se vê onde `B'` se anula.
#[test]
fn an_ordinary_curve_is_untouched_by_the_cure() {
    let c: Cubic = [[0.0, 0.0], [1.0, 3.0], [5.0, 3.0], [6.0, 0.0]];
    for k in 0..=100 {
        let t = f64::from(k) / 100.0;
        let d = super::deriv(&c, t);
        let n = (d[0] * d[0] + d[1] * d[1]).sqrt();
        assert!(n > 1e-12, "a fixtura tem uma cuspide e nao devia");
        let esperado = [d[0] / n, d[1] / n];
        assert_eq!(
            tangent_at(&c, t),
            Some(esperado),
            "a tangente do caminho comum mudou em t = {t}"
        );
    }
}

/// ⚠️⚠️ **AS DUAS PONTAS TÊM CASCATAS DIFERENTES, e numa RETA isso não se vê.**
///
/// Numa reta `[a, a, b, b]` a cascata de `t = 0` (`P₁−P₀ → P₂−P₀`) e a de `t = 1`
/// (`P₃−P₂ → P₃−P₁`) devolvem **a mesma** direção — logo trocá-las é uma mutação que
/// **sobrevive** a todo gate escrito sobre uma reta.
///
/// ⇒ a fixtura é **assimétrica**: degenerada só no FIM, e curva. Ali as duas cascatas dão
/// direções diferentes, e só a de `t = 1` acompanha a marcha.
#[test]
fn the_two_ends_have_different_cascades_and_a_straight_line_hides_it() {
    // `P₃ = P₂` ⇒ `B'(1) = 0`; o começo é ordinário.
    let c: Cubic = [[0.0, 0.0], [2.0, 1.0], [6.0, 0.0], [6.0, 0.0]];
    assert!(
        super::deriv(&c, 1.0)[0].hypot(super::deriv(&c, 1.0)[1]) < 1e-12,
        "a fixtura nao e' degenerada no fim"
    );
    let u = tangent_at(&c, 1.0).expect("sem direcao no fim");
    // `P₃ − P₁ = (4, −1)`, normalizado.
    let esperado = {
        let n = 4.0_f64.hypot(-1.0);
        [4.0 / n, -1.0 / n]
    };
    assert!(
        (u[0] - esperado[0]).abs() < 1e-12 && (u[1] - esperado[1]).abs() < 1e-12,
        "a ponta final nao usou a cascata dela: {u:?} contra {esperado:?}"
    );
    // ⚠️ **A metade que dá sujeito ao gate**: a cascata do OUTRO extremo daria outra direção.
    let do_comeco = {
        let n = 2.0_f64.hypot(1.0);
        [2.0 / n, 1.0 / n]
    };
    assert!(
        (u[0] - do_comeco[0]).abs() > 1e-3 || (u[1] - do_comeco[1]).abs() > 1e-3,
        "as duas cascatas dao a mesma resposta nesta fixtura - trocar uma pela outra passaria"
    );
}

/// ⛔⛔ **O `t` NUNCA CHEGA À PONTA — ele chega PERTO dela**, e foi isto que reprovou a 1.ª cura.
///
/// O prefixo de arco de um contorno é somado por **quadratura**: o comprimento de um segmento reto
/// de `2` sai `2,000000000000000_4`. Quem pergunta pelo arco `2,0` cai no segmento **anterior**, em
/// `t = 0,999999999999999_8` — e ali `|B'| ≈ 2,6e-15`: **abaixo** do piso do versor e **acima** de
/// zero. Uma cura que testasse `t >= 1.0` não abriria, e o buraco voltava.
///
/// ⚠️ **Medido na árvore, não de mesa** (plano 36 §11.2-bis): com a 1.ª cura o quadrado de lado `7`
/// ficava a zero buracos e os de lado `2` e `12` mantinham `4 de 4` quinas sem tangente. *Duas
/// fixturas do mesmo desenho, e só uma via o defeito.*
#[test]
fn the_parameter_lands_near_the_end_not_on_it_and_the_cure_has_to_reach_there() {
    // ⚠️⚠️ **A fixtura é ASSIMÉTRICA de propósito, e uma RETA não serviria.** Numa reta as duas
    // cascatas devolvem a mesma direção, então uma cura que caísse no ramo ERRADO passaria — a
    // mutação `t < 0.5` -> `t <= 0.0` **sobreviveu** à 1.ª redacção deste gate por isso. Aqui só o
    // COMEÇO é degenerado: cair no ramo do fim devolve `None`, que é visível.
    let c: Cubic = [[0.0, 0.0], [0.0, 0.0], [3.0, 1.0], [6.0, 0.0]];
    let t = f64::EPSILON;
    // A fixtura CONTÉM o fenómeno: velocidade não-nula, mas abaixo do piso do versor.
    let d = super::deriv(&c, t);
    let n = d[0].hypot(d[1]);
    assert!(
        n > 0.0 && n < 1e-12,
        "a fixtura nao reproduz o `t` quase-na-ponta: |B'| = {n:e}"
    );
    let u = tangent_at(&c, t).expect("sem direcao a um epsilon do comeco");
    let esperado = {
        let m = 3.0_f64.hypot(1.0);
        [3.0 / m, 1.0 / m]
    };
    assert!(
        (u[0] - esperado[0]).abs() < 1e-9 && (u[1] - esperado[1]).abs() < 1e-9,
        "a direcao a um epsilon do comeco nao saiu da cascata DELE: {u:?} contra {esperado:?}"
    );
    // ⚠️ **A metade que dá sujeito**: o outro extremo NÃO é degenerado, então o ramo do fim não
    // teria resposta nenhuma para dar aqui.
    assert!(
        super::deriv(&c, 1.0)[0].hypot(super::deriv(&c, 1.0)[1]) > 1e-12,
        "a fixtura ficou degenerada nos dois extremos - a mutacao do ramo volta a ser invisivel"
    );
    // E a reta também é servida, pelo mesmo mecanismo (aqui os dois ramos concordam).
    let r = reta([0.0, 0.0], [10.0, 0.0]);
    let ur = tangent_at(&r, 1.0 - f64::EPSILON).expect("sem direcao a um epsilon do fim");
    assert!((ur[0] - 1.0).abs() < 1e-12, "no fim de uma reta: {ur:?}");
}
