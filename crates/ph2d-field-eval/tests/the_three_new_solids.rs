//! ⭐⭐ **O que as três formas da W101 PROMETEM, medido na superfície** — cone, cápsula, prisma.
//!
//! # ⚠️ A régua é o campo, não o desenho
//!
//! Um gate que comparasse duas construções nossas ficaria cego a qualquer mutação que afectasse as
//! duas do mesmo modo — a lição que a W97 pagou. Aqui cada afirmação é medida contra um **facto
//! geométrico independente**: o raio da secção a uma altura dada, a distância a um ponto conhecido,
//! o apótema de um polígono regular.
//!
//! ⚠️ E o `‖∇f‖` **não** se mede aqui: ele tem gate próprio e derivado
//! ([`every_primitive_honours_the_march`](every_primitive_honours_the_march.rs)), porque a pergunta
//! *«a marcha é segura?»* é sobre a família toda e não sobre estas três.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_eval::Field;

fn field_of(p: Primitive) -> Field {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça");
    Field::new(&doc)
}

/// O raio da secção em `z`: onde o campo cruza zero ao andar para fora no eixo X.
///
/// ⚠️ **Bissecção sobre o SINAL**, e não uma leitura do valor: o valor é a distância à superfície
/// mais próxima, que junto a uma quina não é a distância radial. O cruzamento é que é a secção.
fn section_radius(f: &Field, z: f64) -> f64 {
    let (mut lo, mut hi) = (0.0_f64, 4.0_f64);
    assert!(
        f.at(lo, 0.0, z) < 0.0,
        "o eixo tinha de estar DENTRO em z={z}"
    );
    assert!(f.at(hi, 0.0, z) > 0.0, "a 4 unidades tinha de estar fora");
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if f.at(mid, 0.0, z) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// ⭐⭐⭐ **UM CONE INTERPOLA OS DOIS RAIOS LINEARMENTE — E O FILETE NÃO O ENGORDA.**
///
/// ⚠️ A régua é a **secção medida a cinco alturas**, contra a reta que os dois raios autorados
/// definem. Um gate que só olhasse as pontas passaria com uma parede curva no meio.
///
/// # ⚠️ A segunda metade veio de uma MUTAÇÃO QUE SOBREVIVEU
///
/// A primeira versão media só `round = 0`, e a mutação que troca o recuo exacto da parede
/// (`round·√(1+m²)`) pelo ingénuo (`round`) **passou a suíte inteira** — precisamente o termo que a
/// W101 acrescentou, sem ninguém a olhar. *Gatear a forma sem filete é gatear metade do módulo cujo
/// argumento é o filete.*
///
/// ⭐ A lei é a mesma da caixa e do cilindro: **arredondar não cresce a peça**. Na parede inclinada
/// isso só é verdade se o recuo for perpendicular — com o recuo ingénuo o cone sai
/// `round·(√(1+m²) − 1)` mais gordo, que aqui é `0,004` contra uma barra de `0,001`.
#[test]
fn a_cone_interpolates_its_two_radii() {
    let (b, t, h) = (0.5_f64, 0.2_f64, 0.4_f64);
    for round in [0.0_f64, 0.06] {
        let f = field_of(Primitive::Cone {
            bottom: b as f32,
            top: t as f32,
            half_height: h as f32,
            round: round as f32,
        });
        // ⚠️ **A faixa é a do MEIO**, e não as pontas: junto às tampas o filete arqueia a
        // silhueta de propósito (é o que ele é), e a lei da reta vale onde a parede é a parede.
        for frac in [-0.5, -0.25, 0.0, 0.25, 0.5] {
            let z = frac * h;
            let esperado = 0.5 * (b + t) + (t - b) / (2.0 * h) * z;
            let medido = section_radius(&f, z);
            assert!(
                (medido - esperado).abs() < 1.0e-3,
                "filete {round}: a {frac:+.2} da meia-altura o cone mede {medido:.4} e a reta dos \
                 dois raios pede {esperado:.4} — arredondar não pode mudar o tamanho"
            );
        }
    }
}

/// ⭐⭐ **O cone FECHADO fecha** — `top = 0` põe o ápice exactamente no topo, e não perto dele.
///
/// ⚠️ **A metade que faz este gate valer é a de baixo**: sem ela, um campo que fosse `+1` em toda
/// parte passaria a primeira afirmação. O ponto de dentro tem de estar dentro.
#[test]
fn a_closed_cone_ends_in_a_point() {
    let h = 0.4_f64;
    let f = field_of(Primitive::Cone {
        bottom: 0.5,
        top: 0.0,
        half_height: h as f32,
        round: 0.0,
    });
    // Um cabelo acima do topo, no eixo: fora.
    assert!(
        f.at(0.0, 0.0, h + 1.0e-3) > 0.0,
        "acima do ápice tem de estar fora"
    );
    // Um cabelo abaixo: dentro. ⚠️ E a folga é `1e-3` porque a peça mede `0,8` — três ordens de
    // grandeza abaixo dela.
    assert!(
        f.at(0.0, 0.0, h - 1.0e-3) < 0.0,
        "logo abaixo do ápice tem de estar dentro — senão a peça não chega lá acima"
    );
    // E a secção some ao chegar ao topo: a 1 % da meia-altura ela é ~1 % do raio de base.
    let quase = section_radius(&f, h * 0.99);
    assert!(
        quase < 0.5 * 0.02,
        "a 1 % do ápice a secção mede {quase:.4} — um cone que não afina não é um cone"
    );
}

/// ⭐⭐ **A CÁPSULA é o segmento engrossado, e a ponta dela está a `h + r`** — não a `√(h²+r²)`.
///
/// ⚠️ Este é o número que o `bounding_radius` erra se alguém escrever uma hipotenusa ali, e o erro
/// é **silencioso**: a caixa do mundo fica pequena e a peça sai cortada só nas pontas.
#[test]
fn a_capsule_is_a_thickened_segment() {
    let (r, h) = (0.25_f64, 0.4_f64);
    let f = field_of(Primitive::Capsule {
        radius: r as f32,
        half_height: h as f32,
    });
    // No meio, a secção é o próprio raio.
    assert!((section_radius(&f, 0.0) - r).abs() < 1.0e-4);
    // A ponta está a `h + r` no eixo, e o campo ali é zero.
    assert!(
        f.at(0.0, 0.0, h + r).abs() < 1.0e-4,
        "a ponta tinha de estar exactamente a h + r"
    );
    // ⭐ **E o campo FORA é a distância exacta** — a cápsula é a única das três da W101 que não
    // perde nada, e isto mede-o: um ponto a `d` da ponta lê `d`.
    let d = 0.3;
    assert!(
        (f.at(0.0, 0.0, h + r + d) - d).abs() < 1.0e-4,
        "fora da ponta o campo tinha de ser a distância exacta"
    );
}

/// ⭐⭐⭐ **O PRISMA mede o CIRCUNRAIO na quina e o APÓTEMA na parede** — e a razão entre os dois é
/// `cos(π/n)`.
///
/// ⚠️ **É esta a afirmação que separa as duas convenções.** Com o apótema a valer `radius`, um
/// prisma trocado por um cilindro do mesmo número **cresceria**; a nossa escolha inscreve-o.
#[test]
fn a_prism_wears_its_radius_on_the_corner() {
    for n in [3_u32, 5, 6, 8, 12] {
        let r = 0.45_f64;
        let f = field_of(Primitive::Prism {
            sides: n,
            bottom: r as f32,
            top: r as f32,
            half_height: 0.3,
            round: 0.0,
        });
        // A parede está a `r·cos(π/n)` do eixo, na direção de meio setor.
        let apotema = r * (std::f64::consts::PI / f64::from(n)).cos();
        let a = std::f64::consts::PI / f64::from(n);
        let (px, py) = (a.cos(), a.sin());
        assert!(
            f.at(px * apotema, py * apotema, 0.0).abs() < 1.0e-4,
            "n={n}: o meio da parede tinha de estar na superfície"
        );
        // ⭐ E a quina, na direção `0`, está a `r`.
        assert!(
            f.at(r, 0.0, 0.0).abs() < 1.0e-4,
            "n={n}: a quina tinha de estar a `radius` do eixo"
        );
    }
}

/// ⭐ **O prisma CONVERGE para o cilindro** — a metade *visual* da razão do teto de lados.
///
/// ⚠️ **A primeira redação deste doc dizia «o `MAX_PRISM_SIDES` NÃO é um limite de custo», e a
/// medição refutou-a**: a `measure_prism_sides` leu `3,80×` o cilindro a 32 lados e `10,93×` a 96 —
/// as paredes são uma cadeia de `max`, e o caminho crítico cresce com `n`. O teto é dos **dois**
/// recursos, e este gate prende só o que é aritmética pura (a outra metade é um relógio, e um
/// relógio não entra num gate desta workstation).
#[test]
fn the_prism_ceiling_is_where_the_corner_stops_showing() {
    let desvio = |n: u32| 1.0 - (std::f64::consts::PI / f64::from(n)).cos();
    assert!(
        desvio(16) > 0.015 && desvio(16) < 0.025,
        "a 16 lados a quina desvia {:.4} do raio",
        desvio(16)
    );
    let no_teto = desvio(ph2d_field::MAX_PRISM_SIDES);
    assert!(
        no_teto < 0.006,
        "no teto de {} lados o desvio é {no_teto:.4} — se ele ainda se vê, o teto está baixo demais \
         pela razão que o doc dá",
        ph2d_field::MAX_PRISM_SIDES
    );
}

// ─────────────────────── W102: a pirâmide, a cunha e o arco ───────────────────────

/// ⭐⭐⭐ **A PIRÂMIDE é o prisma com o topo a zero** — e a secção interpola linearmente, como o cone.
///
/// ⚠️ A régua é o **apótema** medido a várias alturas (a parede está a `raio·cos(π/n)` do eixo),
/// contra a reta que os dois raios autorados definem. É a mesma lei do cone, e é isso que prova que
/// a fórmula é **uma** — não duas parecidas.
/// # ⚠️ A metade do FILETE veio de uma mutação que sobreviveu — a MESMA que o cone já tinha tido
///
/// A primeira versão media só `round = 0`, e trocar o recuo perpendicular da parede
/// (`round·√(1+m²)`) pelo ingénuo (`round`) **passava**. O erro é `0,0014` numa barra de `0,001`:
/// mede-se, e vê-se. *Gatear a forma sem filete é gatear metade do módulo cujo argumento é o filete*
/// — escrito na W101 e repetido na W102.
#[test]
fn a_pyramid_is_the_prism_whose_top_closes() {
    let (b, t, h, n) = (0.45_f64, 0.0_f64, 0.4_f64, 4_u32);
    let k = (std::f64::consts::PI / f64::from(n)).cos();
    // A direcção do meio da parede (meio sector).
    let a = std::f64::consts::PI / f64::from(n);
    let (px, py) = (a.cos(), a.sin());
    let parede = |f: &Field, z: f64| {
        let (mut lo, mut hi) = (0.0_f64, 4.0_f64);
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if f.at(px * mid, py * mid, z) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    };
    // ⭐ **Com e sem filete.** ⚠️ O caso com filete usa um TRONCO e não a pirâmide fechada: junto ao
    // ápice o filete arqueia tudo, e a lei da reta vale onde a parede é a parede.
    for (topo, round) in [(t, 0.0_f64), (0.18, 0.05)] {
        let f = field_of(Primitive::Prism {
            sides: n,
            bottom: b as f32,
            top: topo as f32,
            half_height: h as f32,
            round: round as f32,
        });
        for frac in [-0.5_f64, -0.25, 0.0, 0.25, 0.5] {
            let z = frac * h;
            let esperado = (0.5 * (b + topo) + (topo - b) / (2.0 * h) * z) * k;
            let medido = parede(&f, z);
            assert!(
                (medido - esperado).abs() < 1.0e-3,
                "filete {round}: a {frac:+.2} da meia-altura a parede está a {medido:.4} e a reta \
                 pede {esperado:.4} — arredondar não pode mudar o tamanho"
            );
        }
    }
    let f = field_of(Primitive::Prism {
        sides: n,
        bottom: b as f32,
        top: t as f32,
        half_height: h as f32,
        round: 0.0,
    });
    // ⭐ E ela FECHA: a 1 % do topo a secção é ~1 % da base.
    let quase = parede(&f, h * 0.99);
    assert!(
        quase < b * 0.02,
        "a 1 % do ápice a parede está a {quase:.4}"
    );
}

/// ⭐⭐ **A CUNHA é cheia num lado e a zero no outro**, e o corte passa pela origem.
///
/// ⚠️ **A metade que faz o gate valer é a terceira**: sem ela, uma caixa inteira (sem corte nenhum)
/// passaria as duas primeiras.
#[test]
fn a_wedge_is_full_on_one_side_and_gone_on_the_other() {
    let (hx, hy, hz) = (0.45_f64, 0.3, 0.35);
    let f = field_of(Primitive::Wedge {
        half: [hx as f32, hy as f32, hz as f32],
        round: 0.0,
    });
    // No lado cheio, o canto de cima está dentro.
    assert!(
        f.at(-hx * 0.9, 0.0, hz * 0.9) < 0.0,
        "o canto alto do lado cheio tem de estar DENTRO"
    );
    // No lado vazio, o mesmo canto está fora.
    assert!(
        f.at(hx * 0.9, 0.0, hz * 0.9) > 0.0,
        "o canto alto do lado fino tem de estar FORA — senão não há corte"
    );
    // ⭐ E o plano passa pela ORIGEM: o centro está exactamente na superfície do corte, que aqui é o
    // ponto mais fundo do sólido nessa direcção.
    assert!(
        f.at(0.0, 0.0, 0.0) <= 0.0,
        "o centro do nó tem de estar dentro (ou na superfície) — o corte passa por ele"
    );
    // ⛔ **O CONTROLE**: um pouco acima do centro já está fora, senão o corte não passa pela origem.
    assert!(
        f.at(0.0, 0.0, hz * 0.2) > 0.0,
        "acima do centro tem de estar fora — o plano do corte passa pela origem"
    );
    // ⭐⭐⭐ **E OS DOIS CANTOS DO CORTE ESTÃO NA SUPERFÍCIE** — é o que fixa a INCLINAÇÃO do plano.
    //
    // ⚠️ Veio de uma mutação que sobreviveu: trocar `(hz/d, hx/d)` por `(hx/d, hz/d)` inclina o
    // plano de outra maneira e **passa as quatro afirmações acima**, porque nenhuma delas mede um
    // ponto cuja resposta dependa da inclinação. *Um gate que só olha «cheio de um lado, vazio do
    // outro» aceita qualquer corte que separe os dois lados.*
    for (x, z) in [(-hx, hz), (hx, -hz)] {
        assert!(
            f.at(x, 0.0, z).abs() < 1.0e-4,
            "o canto ({x:.2}, {z:.2}) tinha de estar NA superfície — é ele que fixa a inclinação"
        );
    }
}

/// ⭐⭐⭐ **O ARCO cobre exactamente o ângulo pedido** — e a régua é o azimute onde o tubo existe.
///
/// ⚠️ **Os dois lados do sector são medidos**: até meia volta ele é interseção, acima é união, e um
/// gate que só olhasse um dos regimes deixaria o outro sem defesa. *Um valor não é uma família.*
#[test]
fn a_torus_arc_covers_exactly_the_angle_asked() {
    for angle in [
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
        std::f64::consts::PI * 1.5,
    ] {
        let (major, minor) = (0.4_f64, 0.12);
        let f = field_of(Primitive::TorusArc {
            major: major as f32,
            minor: minor as f32,
            angle: angle as f32,
            round: 0.0,
        });
        let dentro = |t: f64| f.at(major * t.cos(), major * t.sin(), 0.0) < 0.0;
        // No meio do sector: dentro. Um cabelo fora dele: fora.
        assert!(
            dentro(0.0),
            "ângulo {angle:.3}: o centro do arco tem de existir"
        );
        assert!(
            dentro(angle * 0.5 - 0.05),
            "ângulo {angle:.3}: perto da ponta ainda é arco"
        );
        assert!(
            !dentro(angle * 0.5 + 0.05),
            "ângulo {angle:.3}: passada a ponta o tubo tem de acabar"
        );
        // ⭐ E o lado oposto — o que o sector NÃO cobre.
        assert!(
            !dentro(std::f64::consts::PI),
            "ângulo {angle:.3}: o lado oposto não pode existir num arco de menos de uma volta"
        );
    }
}

/// ⭐ **Uma volta inteira é o toro inteiro** — e o corte não é construído.
///
/// ⚠️ Sem esta afirmação, um `angle` de `2π` cairia no ramo do sector e deixaria uma **fenda
/// invisível** que o artista só descobria ao exportar.
#[test]
fn a_full_sweep_is_the_whole_torus() {
    let f = field_of(Primitive::TorusArc {
        major: 0.4,
        minor: 0.12,
        angle: std::f32::consts::TAU,
        round: 0.0,
    });
    for i in 0..64 {
        let t = std::f64::consts::TAU * f64::from(i) / 64.0;
        assert!(
            f.at(0.4 * t.cos(), 0.4 * t.sin(), 0.0) < 0.0,
            "uma volta inteira não pode ter fenda nenhuma (azimute {t:.3})"
        );
    }
}
