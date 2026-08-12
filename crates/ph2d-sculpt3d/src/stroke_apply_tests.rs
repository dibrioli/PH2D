//! **O APLICADOR ÚNICO** — o gate da identidade em que a lei do peso-no-alvo se
//! apoia.
//!
//! [`toward`] escreve `t − (t − b)·(1 − a)` e mais nada. Três verbos já carimbam
//! `a = 1` ([`Grip::Hook`], [`Grip::Turn`]) e para eles o alvo **é** a posição
//! final: se a expressão não devolvesse `t` ao bit, a paridade daqueles kernels
//! morreria **no aplicador**, não no kernel — e nenhum gate de verbo saberia
//! dizer isso, porque todos eles medem deslocamento com tolerância.
//!
//! ⚠️ **Este gate nasceu VERMELHO sobre a forma anterior** (`b + (t − b)·a`) e o
//! defeito era de UM ULP: um Twist escrevia `1,3164502e-8` onde o alvo dizia
//! `1,3164501e-8`. A tabela das três formas e a razão da escolha estão no doc de
//! [`toward`]; aqui elas viram asserção.
//!
//! ⚠️ **E a medição de projeto que dizia que a forma antiga bastava era do
//! REGIME ERRADO** — ela gerava os pares como `t = fl(b + d)`, onde a subtração
//! é livre de erro e a identidade é garantida por construção. O alvo do produto
//! é uma expressão inteira arredondada **uma vez**; contra o `base` ele é um
//! float independente. É exatamente por isso que os gates abaixo tomam os pares
//! do PRODUTO e não de uma fórmula minha: uma fixture que fabrica o valor pela
//! mesma aritmética que vai testar não contém o fenômeno.

use super::*;

/// Os verbos cujo alvo já é a posição final — a tabela de grips do `dab_core` é
/// quem os define, e este `assert` recusa que a lista aqui envelheça.
fn unit_accum_verbs() -> Vec<Verb> {
    let list: Vec<Verb> = Verb::ALL
        .iter()
        .copied()
        .filter(|v| matches!(v.grip(), crate::Grip::Hook | crate::Grip::Turn(_)))
        .collect();
    assert!(
        !list.is_empty(),
        "nenhum verbo carimba accum = 1: a tabela de grips mudou e este gate \
         passou a medir vácuo"
    );
    list
}

/// **A ENTREGA:** quem põe o peso no alvo recebe o alvo de volta, ao BIT.
///
/// ⚠️ **O oráculo é o próprio `target` do traço, e é isso que o torna um gate do
/// produto e não um espelho.** Ele não recomputa o que o verbo devia dar — ele
/// pergunta ao motor o que o verbo deu, e exige que o aplicador escreva
/// exatamente aquilo. Um gate que re-derivasse o alvo mediria a minha leitura do
/// kernel, que é a coisa que nunca falha.
#[test]
fn the_applicator_writes_the_target_verbatim_when_the_weight_lives_in_it() {
    let c = [0.0, 0.0, 1.0];
    let mut checked = 0usize;
    for verb in unit_accum_verbs() {
        // Vários raios e forças: os pares `(base, alvo)` têm de varrer a faixa
        // de deslocamento que o produto produz, não um ponto dela.
        for radius in [0.15f32, 0.4, 0.8] {
            for strength in [0.15f32, 0.5, 1.0] {
                let mut mesh = sphere();
                let brush = Brush {
                    verb,
                    radius,
                    strength,
                    ..Brush::default()
                };
                let mut s = SculptStroke::default();
                s.begin(&mesh);
                let n = s.dab(
                    &mut mesh,
                    &brush,
                    &dab_for(verb, c, radius),
                    Symmetry::default(),
                );
                assert!(n > 0, "{verb:?} r={radius} f={strength}: dab inerte");
                for &v in s.last_moved() {
                    let slot = s.slot[v as usize] as usize;
                    assert_eq!(
                        s.accum[slot], 1.0,
                        "{verb:?}: um verbo de alvo-final tem de carimbar accum = 1"
                    );
                    assert_eq!(
                        mesh.positions()[v as usize],
                        s.target[slot],
                        "{verb:?} r={radius} f={strength} v={v}: o aplicador não \
                         devolveu o alvo ao bit"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(
        checked > 1000,
        "fixture magra demais: só {checked} vértices"
    );
}

/// **O CONTROLE, e sem ele o gate acima é verde por vácuo.**
///
/// Se o aplicador devolvesse o alvo para TODO mundo, a asserção de cima passaria
/// sem dizer nada sobre `accum = 1`. Um verbo de carimbo (`accum = w < 1`) tem
/// de escrever um ponto **entre** o `base` e o alvo — é isso que dá dentes à
/// afirmação de que a identidade é consequência do peso, não do aplicador.
#[test]
fn a_stamp_verb_lands_short_of_its_target_and_that_is_what_gives_the_gate_teeth() {
    let c = [0.0, 0.0, 1.0];
    let radius = 0.4;
    let mut mesh = sphere();
    let brush = Brush {
        verb: Verb::Draw,
        radius,
        strength: 0.5,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(&mut mesh, &brush, &dab_at(c, radius), Symmetry::default());
    let short = s
        .last_moved()
        .iter()
        .filter(|&&v| {
            let slot = s.slot[v as usize] as usize;
            mesh.positions()[v as usize] != s.target[slot]
        })
        .count();
    assert!(
        short > 0,
        "nenhum vértice ficou aquém do alvo: o `accum` deixou de atenuar e o \
         gate irmão virou tautologia"
    );
}

/// **NENHUMA FORMA É EXATA NAS TRÊS PONTAS** — a cerca de Chesterton do
/// [`toward`], executável, e a razão de a expressão ser ancorada no ALVO.
///
/// Em aritmética exata as três são a mesma coisa, e é por isso que a primeira é
/// a que se escreve por reflexo. Este gate conta as divergências das três nas
/// três propriedades e afirma **qual par o produto precisa** — para que trocar
/// de forma custe um vermelho em vez de um ulp que ninguém vê.
#[test]
fn no_applicator_form_is_exact_at_all_three_ends_and_this_is_the_pair_we_need() {
    fn base_anchored(b: f32, t: f32, a: f32) -> f32 {
        b + (t - b) * a
    }
    fn split(b: f32, t: f32, a: f32) -> f32 {
        b * (1.0 - a) + t * a
    }

    // Os pares REAIS, colhidos do produto no dia em que o gate irmão nasceu
    // vermelho: um Twist na esfera de teste, `base` e alvo do mesmo vértice.
    //
    // ⚠️ **O segundo não é degenerado, e é ele que fecha o argumento.** O
    // primeiro é um vértice junto ao eixo (coordenadas de `1e-8`), onde é fácil
    // atribuir a divergência a *"número pequeno demais"*; o segundo é um vértice
    // de coordenada `0,115` girado para `−0,063` — a escala em que o artista de
    // fato esculpe.
    for &(b, t) in &[
        (-4.371_139e-8f32, 1.316_450_1e-8f32),
        (1.151_137_7e-1, -6.264_330_4e-2),
    ] {
        assert_eq!(toward(b, t, 1.0), t, "a forma que shipa é exata na ponta");
        assert_ne!(
            base_anchored(b, t, 1.0),
            t,
            "o par que motivou a troca deixou de divergir: se a forma ancorada \
             no `base` voltou a bastar, é o doc do `toward` que tem de ser \
             re-medido, não este gate que tem de ser apagado"
        );
    }

    // E o CENSO, que é o que torna a escolha legível: cada forma perde numa
    // ponta diferente, e as duas que a que shipa acerta são as duas que o
    // produto promete ao artista.
    let mut state = 0x9e37_79b9_7f4a_7c15u64;
    let mut rnd = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    let (mut base_misses_one, mut split_misses_still) = (0usize, 0usize);
    let mut ours_misses_zero = 0usize;
    let n = 60_000;
    for _ in 0..n {
        let b = (rnd() * 2.0 - 1.0) as f32;
        let t = (rnd() * 2.0 - 1.0) as f32;
        let a = rnd() as f32;

        // O que a lei EXIGE, e o que a forma que shipa entrega.
        assert_eq!(toward(b, t, 1.0), t, "a = 1 tem de pousar no alvo");
        assert_eq!(toward(b, b, a), b, "alvo igual ao base não pode mover");

        // O que as outras duas perdem.
        if base_anchored(b, t, 1.0) != t {
            base_misses_one += 1;
        }
        if split(b, b, a) != b {
            split_misses_still += 1;
        }
        // E o que a NOSSA perde — nomeado, não escondido: a ponta que o produto
        // nunca pede, porque `apply_positions` só percorre `moved`.
        if toward(b, t, 0.0) != b {
            ours_misses_zero += 1;
        }
    }
    let pct = |x: usize| 100.0 * x as f64 / n as f64;
    assert!(
        pct(base_misses_one) > 20.0,
        "a forma ancorada no base parou de errar em `a = 1` ({} de {n}): ou a \
         medição da tabela envelheceu, ou este gate deixou de ter sujeito",
        base_misses_one
    );
    assert!(
        pct(split_misses_still) > 5.0,
        "a forma partida parou de mover um vértice cujo alvo é ele mesmo ({} de \
         {n}): era ela que derrubava o `a_dab_with_no_gesture_moves_nothing`",
        split_misses_still
    );
    assert!(
        pct(ours_misses_zero) > 20.0,
        "a forma que shipa passou a ser exata em `a = 0` ({} de {n}) — se isso \
         é verdade, a tabela do `toward` ficou desatualizada para MELHOR e a \
         nota sobre a ponta que ninguém pede pode cair",
        ours_misses_zero
    );
}

/// **A PREMISSA da ponta que a forma escolhida abre mão:** o produto nunca pede
/// `a = 0`.
///
/// ⚠️ Sem este gate, o doc do [`toward`] estaria trocando uma exatidão por outra
/// com base numa afirmação sobre o `dab_core` que ninguém confere. Ele é medido
/// sobre **todo** verbo, porque a lei de quem entra em `moved` é do laço, não do
/// verbo — e é o laço que pode mudar.
#[test]
fn no_vertex_reaches_the_applicator_with_zero_weight() {
    let c = [0.0, 0.0, 1.0];
    let radius = 0.5;
    let mut seen = 0usize;
    for verb in Verb::ALL {
        let mut mesh = sphere();
        let brush = Brush {
            verb,
            radius,
            strength: 0.5,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        let n = s.dab(
            &mut mesh,
            &brush,
            &dab_for(verb, c, radius),
            Symmetry::default(),
        );
        // ⚠️ **Por verbo, e não só no total:** um verbo inerte não contribui
        // vértice nenhum, e o total de outro cobriria o buraco em silêncio.
        assert!(
            n > 0,
            "{verb:?}: dab inerte, o gate mediria vácuo neste verbo"
        );
        for &v in s.last_moved() {
            let slot = s.slot[v as usize] as usize;
            assert!(
                s.accum[slot] > 0.0,
                "{verb:?} v={v}: peso zero chegou ao aplicador, e é exatamente a \
                 ponta em que a forma escolhida não é exata"
            );
            seen += 1;
        }
    }
    assert!(seen > 500, "fixture magra demais: só {seen} vértices");
}
