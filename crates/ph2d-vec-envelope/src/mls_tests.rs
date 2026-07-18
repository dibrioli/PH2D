//! Gates do MLS-rigid (ADR-0129 Fatia E). Módulo irmão do [`super`] — teto de LOC.

use super::*;

/// Três pinos não-colineares, com o do meio deslocado — a configuração mínima que DEFORMA.
fn three_pins() -> Vec<Pin> {
    vec![
        [[0.0, 0.0], [0.0, 0.0]],
        [[10.0, 0.0], [10.0, 0.0]],
        [[5.0, 6.0], [6.5, 7.5]],
    ]
}

fn det(j: [[f64; 2]; 2]) -> f64 {
    j[0][0] * j[1][1] - j[0][1] * j[1][0]
}

/// **CADA PINO POUSA EXATAMENTE ONDE FOI LARGADO.** É a propriedade de interpolação do método, e é o
/// que o artista lê como "a ferramenta obedece" — se `f(pᵢ) ≠ qᵢ`, nada mais importa.
///
/// Exercita o guard [`PIN_EPS`] no caminho REAL: em `v = pᵢ` o peso seria infinito.
#[test]
fn every_pin_lands_exactly_where_it_was_dragged() {
    let pins = three_pins();
    let w = MlsWarp::new(&pins).unwrap();
    for pin in &pins {
        let got = w.map(pin[0]);
        assert!(
            (got[0] - pin[1][0]).abs() < 1e-9 && (got[1] - pin[1][1]).abs() < 1e-9,
            "o pino {:?} devia pousar em {:?}, pousou em {got:?}",
            pin[0],
            pin[1]
        );
    }
}

/// **UM PINO É TRANSLAÇÃO PURA** — critério de aceitação #5 do ADR-0129, e não um caso especial
/// escrito à mão: com um pino só `p̂ = q̂ = 0` ⇒ `S = 0`, e é o guard de `|S|` que devolve o limite
/// certo. A alternativa (Eq. 8 do paper como está) divide por zero e produz `NaN`.
#[test]
fn one_pin_is_a_pure_translation() {
    let d = [3.0, -2.0];
    let w = MlsWarp::new(&[[[4.0, 4.0], [4.0 + d[0], 4.0 + d[1]]]]).unwrap();
    for p in [[0.0, 0.0], [10.0, 6.0], [-3.0, 8.0], [4.0, 4.0]] {
        let got = w.map(p);
        assert!(
            (got[0] - (p[0] + d[0])).abs() < 1e-9 && (got[1] - (p[1] + d[1])).abs() < 1e-9,
            "1 pino devia transladar {p:?} por {d:?}, deu {got:?}"
        );
    }
    // E a jacobiana da translação é a identidade — se este ramo devolvesse lixo, o fitter travaria.
    let j = w.jacobian([1.0, 1.0]);
    assert!(
        (det(j) - 1.0).abs() < 1e-9,
        "jacobiana de translação: {j:?}"
    );
}

/// **PINOS EM REPOUSO SÃO A IDENTIDADE.** Metade presença: afirma pontos concretos IGUAIS à entrada,
/// então não fica verde num motor que devolve zero.
#[test]
fn pins_at_rest_are_the_identity() {
    let pins: Vec<Pin> = vec![
        [[0.0, 0.0], [0.0, 0.0]],
        [[10.0, 0.0], [10.0, 0.0]],
        [[5.0, 6.0], [5.0, 6.0]],
    ];
    let w = MlsWarp::new(&pins).unwrap();
    for p in [[2.0, 2.0], [7.0, 4.0], [5.0, 1.0]] {
        let got = w.map(p);
        assert!(
            (got[0] - p[0]).abs() < 1e-9 && (got[1] - p[1]).abs() < 1e-9,
            "repouso não é identidade em {p:?}: {got:?}"
        );
    }
}

/// **UM MOVIMENTO RÍGIDO GLOBAL É REPRODUZIDO EXATAMENTE.** É a *precisão rígida* que dá nome ao
/// método: mover TODOS os pinos por uma mesma rotação+translação deforma zero.
#[test]
fn a_global_rigid_motion_is_reproduced_exactly() {
    let (c, s) = (0.6_f64, 0.8_f64); // rotação exata de 3-4-5, sem transcendental
    let t = [2.0, -1.0];
    let rigid = |p: [f64; 2]| [c * p[0] - s * p[1] + t[0], s * p[0] + c * p[1] + t[1]];
    let pins: Vec<Pin> = [[0.0, 0.0], [10.0, 0.0], [5.0, 6.0], [2.0, 3.0]]
        .into_iter()
        .map(|p| [p, rigid(p)])
        .collect();
    let w = MlsWarp::new(&pins).unwrap();
    for p in [[3.0, 1.0], [8.0, 4.0], [1.0, 5.0]] {
        let (got, want) = (w.map(p), rigid(p));
        assert!(
            (got[0] - want[0]).abs() < 1e-9 && (got[1] - want[1]).abs() < 1e-9,
            "movimento rígido não reproduzido em {p:?}: {got:?} != {want:?}"
        );
    }
}

/// **COM 2 PINOS, MOVER O PAR ISOMETRICAMENTE NÃO DEFORMA NADA** (`det J = 1` em todo ponto).
///
/// É a armadilha de dia-um que o ADR-0129 registrou: duas correspondências isométricas determinam
/// uma rigidez ÚNICA, então o método devolve movimento rígido do plano inteiro. **Deformar exige o
/// 3º pino.** Quem for depurar "os pinos não fazem nada" tem de encontrar este teste antes de mexer
/// na matemática — não há o que consertar.
#[test]
fn two_pins_moved_isometrically_deform_nothing() {
    let (c, s) = (0.6_f64, 0.8_f64);
    let rigid = |p: [f64; 2]| [c * p[0] - s * p[1] + 3.0, s * p[0] + c * p[1] - 2.0];
    let pins: Vec<Pin> = [[0.0, 0.0], [10.0, 0.0]]
        .into_iter()
        .map(|p| [p, rigid(p)])
        .collect();
    let w = MlsWarp::new(&pins).unwrap();
    for p in [[2.0, 3.0], [7.0, -4.0], [5.0, 9.0]] {
        assert!(
            (det(w.jacobian(p)) - 1.0).abs() < 1e-7,
            "2 pinos isométricos deformaram em {p:?}: det={}",
            det(w.jacobian(p))
        );
    }
    // E o contraste que dá sentido ao teste: o 3º pino DEFORMA.
    let w3 = MlsWarp::new(&three_pins()).unwrap();
    assert!(
        (det(w3.jacobian([5.0, 3.0])) - 1.0).abs() > 0.01,
        "com 3 pinos deslocados o mapa devia deformar"
    );
}

/// **A JACOBIANA FECHADA BATE COM A DIFERENÇA CENTRAL.** É o gate que valida a derivação de
/// Wirtinger inteira — e o barato, porque o sintoma de errá-la é o `fit_to_bezpath` deixar de
/// convergir (o gate trava em vez de falhar).
#[test]
fn the_closed_form_jacobian_matches_finite_difference() {
    let pins = three_pins();
    let w = MlsWarp::new(&pins).unwrap();
    let step = 1e-6;
    // Longe dos pinos: perto deles os pesos ~1/d² divergem e a diferença central mede a própria
    // divergência, não o erro da derivada.
    for p in [[2.0, 2.0], [7.0, 3.0], [4.0, 1.0], [8.0, 5.0], [1.0, 4.0]] {
        let j = w.jacobian(p);
        let d = |dx: f64, dy: f64| {
            let a = w.map([p[0] + dx, p[1] + dy]);
            let b = w.map([p[0] - dx, p[1] - dy]);
            [(a[0] - b[0]) / (2.0 * step), (a[1] - b[1]) / (2.0 * step)]
        };
        let (du, dv) = (d(step, 0.0), d(0.0, step));
        for (got, want, name) in [
            (j[0][0], du[0], "dx/du"),
            (j[1][0], du[1], "dy/du"),
            (j[0][1], dv[0], "dx/dv"),
            (j[1][1], dv[1], "dy/dv"),
        ] {
            assert!(
                (got - want).abs() < 1e-4,
                "{name} em {p:?}: fechada {got}, diferença central {want}"
            );
        }
    }
}

/// **NENHUM `NaN`, em ponto nenhum** — inclusive EM CIMA de um pino e no centróide.
#[test]
fn the_map_never_produces_nan() {
    for pins in [three_pins(), vec![[[1.0, 1.0], [2.0, 3.0]]]] {
        let w = MlsWarp::new(&pins).unwrap();
        let mut probes: Vec<[f64; 2]> = pins.iter().map(|p| p[0]).collect();
        probes.extend([[5.0, 2.0], [0.0, 0.0], [-4.0, 9.0], [5.0, 6.0]]);
        for p in probes {
            let q = w.map(p);
            let j = w.jacobian(p);
            assert!(
                q[0].is_finite() && q[1].is_finite(),
                "map produziu não-finito em {p:?}: {q:?}"
            );
            assert!(
                j.iter().flatten().all(|v| v.is_finite()),
                "jacobiana não-finita em {p:?}: {j:?}"
            );
        }
    }
}

/// **AUSÊNCIA:** um deslocamento moderado de pino não dobra a arte — o gesto normal não é recusado.
#[test]
fn a_moderate_pin_pull_does_not_fold() {
    let origin = [0.0, 0.0];
    let size = [10.0, 6.0];
    assert!(
        !pins_fold(&three_pins(), origin, size),
        "um puxão moderado foi acusado de dobrar"
    );
    // E sem pino nenhum não há mapa, logo não há dobra.
    assert!(!pins_fold(&[], origin, size));
}

/// **PRESENÇA:** o MESMO amostrador vê a dobra quando ela existe — um pino atirado para o outro lado
/// da arte. Sem este irmão, o gate acima ficaria verde num `pins_fold` que responde `false` sempre.
#[test]
fn the_sampler_detects_a_fold_when_there_is_one() {
    let folded: Vec<Pin> = vec![
        [[0.0, 0.0], [0.0, 0.0]],
        [[10.0, 0.0], [10.0, 0.0]],
        // O pino do meio atravessa a base e sai muito para baixo: o plano vira do avesso.
        [[5.0, 3.0], [5.0, -30.0]],
    ];
    assert!(
        pins_fold(&folded, [0.0, 0.0], [10.0, 6.0]),
        "o amostrador não viu uma dobra grosseira"
    );
}
