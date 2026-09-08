//! ⭐⭐⭐ **OS GATES DAS DUAS CURVAS COM ESPESSURA** (W136) — a Bezier quadrática e a onda em anel.
//!
//! # ⚠️ O gate nº 7 é o que esta wave existiu para escrever
//!
//! A onda passou por **quatro** construções, e três foram recusadas por medição. A que as separou
//! não foi o gradiente nem a secção: foi perguntar **de que FORMA é o filete do aro**. Com o divisor
//! antes da junta ele era uma **elipse** esticada `lip` vezes — a peça estava correcta, o campo era
//! um minorante honesto, e o controlo mentia sobre quanto removia.
//!
//! *Uma régua que só pergunta «a superfície está no sítio?» não vê um filete oval.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform, wave_thickness_ceiling};
use ph2d_field_eval::{Field, ops_curve};

fn campo(p: Primitive) -> Field {
    let doc = FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("o documento tem de aceitar a peça");
    Field::new(&doc)
}

fn onda(radius: f32, amplitude: f32, lobes: u32, round: f32) -> Primitive {
    Primitive::CircleWave {
        radius,
        amplitude,
        lobes,
        thickness: wave_thickness_ceiling(radius, amplitude) * 0.30,
        half_height: 0.12,
        round,
        chamfer: 0.0,
    }
}

/// A distância à Bezier por varredura densa — o oráculo, e ele não partilha uma linha com a cúbica.
fn dist_a_curva(px: f64, py: f64, a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    let n = 100_000_usize;
    let mut melhor = f64::INFINITY;
    for i in 0..=n {
        let t = i as f64 / n as f64;
        let u = 1.0 - t;
        let x = u * u * a[0] + 2.0 * t * u * b[0] + t * t * c[0];
        let y = u * u * a[1] + 2.0 * t * u * b[1] + t * t * c[1];
        melhor = melhor.min((px - x).hypot(py - y));
    }
    melhor
}

/// ⭐⭐⭐ **A BEZIER É A CURVA QUE PROMETE** — contra a varredura densa, em cinco arranjos.
#[test]
fn the_bezier_is_the_curve_it_promises() {
    for (a, b, c) in [
        ([-0.35, -0.20], [0.0, 0.45], [0.35, -0.20]),
        ([-0.30, 0.18], [0.0, -0.18], [0.30, 0.18]),
        ([-0.40, 0.10], [0.10, -0.40], [0.40, 0.30]),
        ([-0.40, 0.00], [0.00, 0.02], [0.40, 0.00]),
        ([-0.20, -0.30], [0.00, 0.40], [0.20, -0.30]),
    ] {
        let f = Field::from_tree(&ops_curve::bezier_dist2(
            &fidget::context::Tree::x(),
            &fidget::context::Tree::y(),
            a,
            b,
            c,
        ));
        let mut pior = 0.0_f64;
        for i in 0..22 {
            for j in 0..22 {
                let x = -0.5 + f64::from(i) / 21.0;
                let y = -0.5 + f64::from(j) / 21.0;
                let lido = f.at(x, y, 0.0);
                assert!(
                    lido.is_finite(),
                    "o campo devolveu NÃO-FINITO em ({x}, {y})"
                );
                pior = pior.max((lido.max(0.0).sqrt() - dist_a_curva(x, y, a, b, c)).abs());
            }
        }
        assert!(
            pior < 1.0e-4,
            "a Bezier {a:?} {b:?} {c:?} erra {pior:.2e} contra a varredura densa"
        );
    }
}

/// ⭐⭐ **A PARÁBOLA É A BEZIER** — e é por isso que ela não é uma primitiva à parte.
///
/// Em `[−w, w]`, `y = k·x²` tem extremos `(±w, kw²)` e tangentes de declive `∓2kw`; elas encontram-se
/// em `(0, −k w²)`, que é o ponto de controlo.
#[test]
fn the_parabola_is_a_bezier_with_the_points_in_the_right_place() {
    for (k, w) in [(1.0_f64, 0.4_f64), (2.0, 0.3), (0.5, 0.45), (4.0, 0.25)] {
        let (a, b, c) = ([-w, k * w * w], [0.0, -k * w * w], [w, k * w * w]);
        let mut pior = 0.0_f64;
        for i in 0..=500 {
            let t = f64::from(i) / 500.0;
            let u = 1.0 - t;
            let x = u * u * a[0] + 2.0 * t * u * b[0] + t * t * c[0];
            let y = u * u * a[1] + 2.0 * t * u * b[1] + t * t * c[1];
            pior = pior.max((y - k * x * x).abs());
        }
        assert!(
            pior < 1.0e-12,
            "com k = {k} e w = {w} a Bezier desvia-se {pior:.2e} da parábola — se isto reprovar, a \
             PARÁBOLA deixou de ser exprimível pela Bezier e volta à fila como forma própria"
        );
    }
}

/// ⭐⭐ **OS TRÊS PONTOS EM LINHA DÃO UM SEGMENTO, e a distância é EXACTA** — o ramo do host.
#[test]
fn a_straight_bezier_is_a_segment() {
    let (a, b, c) = ([-0.30, -0.10], [0.0, 0.0], [0.30, 0.10]);
    let f = Field::from_tree(&ops_curve::bezier_dist2(
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
        a,
        b,
        c,
    ));
    let mut pior = 0.0_f64;
    for i in 0..30 {
        for j in 0..30 {
            let x = -0.5 + f64::from(i) / 29.0;
            let y = -0.5 + f64::from(j) / 29.0;
            // o oráculo do segmento, à mão
            let (ex, ey) = (c[0] - a[0], c[1] - a[1]);
            let t = (((x - a[0]) * ex + (y - a[1]) * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
            let d = (x - a[0] - t * ex).hypot(y - a[1] - t * ey);
            pior = pior.max((f.at(x, y, 0.0).max(0.0).sqrt() - d).abs());
        }
    }
    assert!(
        pior < 1.0e-9,
        "com os três pontos em linha a distância tinha de ser a do segmento, e erra {pior:.2e}"
    );
}

/// ⭐⭐ **OS DOIS RAMOS DA CÚBICA ENCONTRAM-SE SEM DEGRAU** — em `h = 0` a raiz é DUPLA e as duas
/// fórmulas dão o mesmo ponto, e é isso que torna o selector duro legítimo.
///
/// ⚠️ A régua é a mesma do nó (W134): o salto **encolhe** com o passo; uma descontinuidade não.
#[test]
fn the_two_cubic_branches_meet_without_a_step() {
    let (a, b, c) = ([-0.35, -0.20], [0.0, 0.45], [0.35, -0.20]);
    let f = Field::from_tree(&ops_curve::bezier_dist2(
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
        a,
        b,
        c,
    ));
    let mut anterior = f64::INFINITY;
    for passo in [1.0e-2_f64, 1.0e-3, 1.0e-4] {
        let mut salto = 0.0_f64;
        for i in 0..200 {
            let x = -0.45 + 0.9 * f64::from(i) / 199.0;
            for y in [-0.3_f64, 0.0, 0.25] {
                let (u, v) = (f.at(x, y, 0.0), f.at(x + passo, y, 0.0));
                if u.is_finite() && v.is_finite() {
                    // o salto por unidade de passo — num campo contínuo ele é limitado
                    salto = salto.max((v.sqrt() - u.sqrt()).abs());
                }
            }
        }
        assert!(
            salto < anterior * 0.6 || salto < 1.0e-9,
            "o salto não encolheu com o passo ({salto:.3e} contra {anterior:.3e}) — isso é uma \
             DESCONTINUIDADE entre os dois ramos da cúbica, não erro de amostragem"
        );
        anterior = salto;
    }
}

/// ⭐⭐⭐ **O FILETE DO ARO DA ONDA É REDONDO, E NÃO UMA ELIPSE** — o gate que esta wave existiu para
/// escrever.
///
/// ⛔ Com o divisor aplicado à banda **antes** da junta, a junta (que supõe duas distâncias
/// verdadeiras) devolvia um arco esticado `lip` vezes na direcção da parede: recuo em `z` `0,0179`
/// contra `0,0711` em `ρ`, com `lip = 4,03`. A peça estava certa e o **controlo mentia**.
#[test]
fn the_wave_rim_fillet_is_round_not_oval() {
    let (radius, amplitude, lobes) = (0.38_f32, 0.11_f32, 7_u32);
    let thickness = f64::from(wave_thickness_ceiling(radius, amplitude) * 0.30);
    let half_height = 0.12_f64;
    for frac in [0.25_f64, 0.5] {
        #[allow(clippy::cast_possible_truncation)]
        let round = (thickness.min(half_height) * frac) as f32;
        let f = campo(onda(radius, amplitude, lobes, round));
        let phi = std::f64::consts::FRAC_PI_2 / f64::from(lobes);
        let r_crista = f64::from(radius + amplitude);
        let (mut z_fim, mut rho_fim) = (0.0, r_crista);
        for i in 0..=4000 {
            let z = f64::from(i) / 4000.0 * half_height;
            let rr = r_crista + thickness - 1.0e-5;
            if f.at(rr * phi.cos(), rr * phi.sin(), z) <= 0.0 {
                z_fim = z;
            }
            let rr2 = r_crista + f64::from(i) / 4000.0 * thickness;
            if f.at(rr2 * phi.cos(), rr2 * phi.sin(), half_height - 1.0e-5) <= 0.0 {
                rho_fim = rr2;
            }
        }
        let (rz, rrho) = (half_height - z_fim, r_crista + thickness - rho_fim);
        let razao = rrho / rz;
        assert!(
            (razao - 1.0).abs() < 0.12,
            "o filete do aro recua {rrho:.5} em ρ e {rz:.5} em z (razão {razao:.2}) — ele é uma \
             ELIPSE, e um controlo que recua mais numa direcção do que na outra mente sobre quanto \
             remove"
        );
        assert!(
            (rz - f64::from(round)).abs() < f64::from(round) * 0.12,
            "o recuo em z é {rz:.5} e o pedido foi {round} — o filete não entrega o que promete"
        );
    }
}

/// ⭐⭐ **DENTRO DO FURO O CAMPO É PLANO** — as duas metades da lei do piso e do tecto.
///
/// ⛔ Sem elas, `‖∇(ρ − R(φ))‖ = √(1 + (R'/ρ)²)` explode junto ao eixo, e o divisor — tomado onde há
/// matéria — não o segura. Medido antes da cura: `2,46` no representante e `13,67` no tecto.
#[test]
fn the_wave_field_is_flat_deep_inside_the_hole() {
    let f = campo(onda(0.38, 0.11, 7, 0.0));
    let mut pior = 0.0_f64;
    for i in 0..30 {
        for j in 0..30 {
            for k in 0..8 {
                let rho = 0.005 + 0.05 * f64::from(i) / 29.0;
                let phi = -3.1 + 6.2 * f64::from(j) / 29.0;
                let z = -0.08 + 0.16 * f64::from(k) / 7.0;
                let g = f.gradient_norm(rho * phi.cos(), rho * phi.sin(), z, 1.0e-5);
                if g.is_finite() {
                    pior = pior.max(g);
                }
            }
        }
    }
    assert!(
        pior < 1.02,
        "fundo dentro do furo o campo lê ‖∇f‖ = {pior:.4} — o termo angular voltou a escapar"
    );
}

/// ⭐⭐ **A COSTURA DO `atan2` NÃO EXISTE** — e a razão é `lobes` ser INTEIRO.
#[test]
fn the_seam_of_the_angle_does_not_crack_the_wave() {
    for lobes in [1_u32, 3, 8] {
        let f = campo(onda(0.38, 0.11, lobes, 0.0));
        let mut anterior = f64::INFINITY;
        for passo in [1.0e-2_f64, 1.0e-3, 1.0e-4] {
            let mut salto = 0.0_f64;
            for k in 0..20 {
                let z = -0.10 + 0.20 * f64::from(k) / 19.0;
                for rho in [0.28_f64, 0.38, 0.46] {
                    salto = salto.max((f.at(-rho, passo, z) - f.at(-rho, -passo, z)).abs());
                }
            }
            assert!(
                salto < anterior * 0.5 || salto < 1.0e-9,
                "o salto na costura não encolheu com o passo ({salto:.3e} contra {anterior:.3e}, \
                 {lobes} lóbulos) — isso é uma DESCONTINUIDADE"
            );
            anterior = salto;
        }
    }
}

/// ⭐⭐ **UM LÓBULO ESCRITO COM FRACÇÃO ATERRA NUM LÓBULO** — a lei da W128 paga pela representação.
#[test]
fn a_lobe_count_written_with_a_fraction_lands_on_a_lobe() {
    let mut p = onda(0.38, 0.11, 6, 0.0);
    assert!(
        ph2d_field::set_dim(&mut p, 0, 2, 0.0).is_err(),
        "a faixa dos lóbulos não oferece o zero, logo a porta tem de o RECUSAR"
    );
    for (escrito, esperado) in [
        (6.6_f32, 7_u32),
        (6.4, 6),
        (9999.0, ph2d_field::MAX_WAVE_LOBES),
    ] {
        ph2d_field::set_dim(&mut p, 0, 2, escrito).expect("a escrita");
        let Primitive::CircleWave { lobes, .. } = p else {
            unreachable!()
        };
        assert_eq!(
            lobes, esperado,
            "escrever {escrito} devia aterrar em {esperado}"
        );
    }
}

/// ⭐⭐ **O TECTO DA ESPESSURA É O VÃO ATÉ AO EIXO**, e subir a amplitude re-assenta-a.
#[test]
fn raising_the_amplitude_reseats_the_thickness() {
    let mut p = Primitive::CircleWave {
        radius: 0.50,
        amplitude: 0.05,
        lobes: 6,
        thickness: wave_thickness_ceiling(0.50, 0.05) * 0.99,
        half_height: 0.12,
        round: 0.0,
        chamfer: 0.0,
    };
    ph2d_field::set_dim(&mut p, 0, 1, 0.40).expect("a escrita da amplitude");
    let Primitive::CircleWave {
        amplitude,
        thickness,
        ..
    } = p
    else {
        unreachable!()
    };
    assert!(
        thickness <= wave_thickness_ceiling(0.50, amplitude),
        "a espessura {thickness} não seguiu a amplitude {amplitude}"
    );
    // ⚠️ E a peça tem de continuar a ser aceite pelo documento.
    let _ = campo(p);
}

/// A peça que o censo constrói tem de caber num documento — a rede contra o representante deixar de
/// ser derivado do tecto.
#[test]
fn the_two_curves_of_the_census_are_pieces_the_document_accepts() {
    let _ = campo(Primitive::Bezier {
        a: [-0.40, -0.20],
        b: [0.0, 0.50],
        c: [0.40, -0.20],
        thickness: 0.09,
        half_height: 0.14,
        round: 0.0,
        chamfer: 0.0,
    });
    let _ = campo(onda(0.40, 0.11, 7, 0.0));
}
