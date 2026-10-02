//! Os gates de [`super`] — a força e a escala dos padrões, a junção, a lei do `feDisplacementMap`.

use super::*;

/// O RMS do mapa de um Flow e o RMS do gradiente dele, na mesma grelha do [`Amostrador::estat`].
fn reguas(a: &Amostrador) -> (f32, f32) {
    let e = a.estat();
    let ponto = a.grelha();
    let (mut s, mut g) = (0.0f64, 0.0f64);
    for j in 0..N {
        for i in 0..N {
            let (x, y) = (ponto(i) + 3, ponto(j) + 5);
            let (u, v) = a.unidade(&e, x, y);
            let (ux, vx) = a.unidade(&e, x + 1, y);
            let (uy, vy) = a.unidade(&e, x, y + 1);
            s += f64::from(u * u + v * v);
            for d in [ux - u, uy - u, vx - v, vy - v] {
                g += f64::from(d) * f64::from(d);
            }
        }
    }
    let n = (2 * N * N) as f64;
    ((s / n).sqrt() as f32, (g / n).sqrt() as f32)
}

/// ⭐ **As duas réguas do Ragged Edge valem o mesmo em todo padrão**: o RMS do deslocamento e o do
/// GRADIENTE dele ficam a ±25 % dos do Classic (o `clamp` a `[−1, 1]` e a grelha finita comem um
/// pouco). É isto que faz «Ragged Edge 24 · Flow Size 1» significar a mesma força e a mesma escala.
///
/// **Mutação que tem de sangrar:** tirar o `rms_classic()` do [`Amostrador::unidade`], ou o
/// `escala_do_flow` do [`Amostrador::flow`].
#[test]
fn todo_padrao_tem_a_forca_e_a_escala_do_classic() {
    let (rc, gc) = classic();
    for &k in &ph2d_painter_brush::FLOW_KINDS[1..] {
        let mut s = ph2d_painter_brush::BrushSpec::default().edge_flow;
        s.kind = k;
        for (i, p) in ph2d_painter_brush::param_specs(k).iter().enumerate() {
            s.params[i] = p.default;
        }
        let (r, g) = reguas(&Amostrador::flow(s, NoiseTile::NONE));
        assert!(
            (r / rc - 1.0).abs() < 0.25,
            "{k:?}: RMS {r:.3} contra o Classic {rc:.3}"
        );
        assert!(
            (g / gc - 1.0).abs() < 0.25,
            "{k:?}: gradiente {g:.4} contra o Classic {gc:.4}"
        );
    }
}

/// A lei do deslocamento é a do `feDisplacementMap` do SVG: a borda no texel `p` LÊ a cobertura em
/// `p + d(p)`, e o mapa lido entre texels é bilinear — nos texels inteiros devolve o valor gravado.
#[test]
fn o_mapa_e_bilinear_e_exacto_nos_texels() {
    let m = Mapa {
        d: vec![[0.0, 1.0], [1.0, 0.0], [0.5, -0.5], [-1.0, 0.25]],
        x0: 10,
        y0: 20,
        w: 2,
        h: 2,
    };
    assert_eq!(m.le(10.0, 20.0), (0.0, 1.0));
    assert_eq!(m.le(11.0, 21.0), (-1.0, 0.25));
    let (a, b) = m.le(10.5, 20.5);
    assert!(
        (a - 0.125).abs() < 1e-6 && (b - 0.1875).abs() < 1e-6,
        "({a}, {b})"
    );
}

/// ⭐ **A junção de dois Flows não degraua.** Dois donos lado a lado (Classic à esquerda, Wood à
/// direita da coluna 60) e o deslocamento lido ao longo de cada linha: o maior salto entre texels
/// vizinhos tem de ficar perto do de DENTRO de cada lado, e muito abaixo do salto discreto (o
/// controlo: a mesma composição com os pesos desligados, que é o defeito que eles curam).
#[test]
fn a_juncao_de_dois_flows_nao_degraua() {
    let spec = ph2d_painter_brush::BrushSpec::default();
    let a = WetStrokeStyle::capture(&spec, 0.0);
    let mut b = a;
    b.edge_flow.kind = TextureKind::Wood;
    let (w, h) = (120usize, 40usize);
    let owner: Vec<u8> = (0..w * h).map(|i| if i % w < 60 { 1 } else { 2 }).collect();
    let (f, _) = EdgeFlow::build(
        &b,
        &[a, b],
        Some(&owner),
        None,
        w,
        (0, 0, w, h),
        (0, 0, w, h),
        NoiseTile::NONE,
    );
    let salto = |fl: &EdgeFlow, de: usize, ate: usize| {
        let mut m = 0.0f32;
        for y in 0..h {
            for x in de..ate {
                let d = |x: usize| {
                    fl.desloca(
                        owner[y * w + x],
                        x as f32,
                        y as f32,
                        x as f32,
                        y as f32,
                        24.0,
                    )
                };
                let ((ax, ay), (bx, by)) = (d(x), d(x + 1));
                m = m.max((bx - ax).abs().max((by - ay).abs()));
            }
        }
        m
    };
    let dentro = salto(&f, 5, 40).max(salto(&f, 80, 115));
    let juncao = salto(&f, 40, 80);
    let discreto = EdgeFlow { pesos: None, ..f };
    let degrau = salto(&discreto, 58, 61);
    assert!(
        degrau > 4.0 * dentro,
        "controlo: sem pesos a junção TEM de degrauar ({degrau:.2} × {dentro:.2})"
    );
    assert!(
        juncao < 2.0 * dentro,
        "a junção degraua: {juncao:.2} px contra {dentro:.2} dentro de cada lado"
    );
}

/// ⭐ **A lei do deslocamento é a do `feDisplacementMap`** — o Inkscape 1.4.4 CORRIDO sobre um disco e
/// um mapa NOSSOS (`docs/Painter/ferramentas/oraculo_deslocamento`), e refeito aqui com o
/// [`Mapa::le`] e o `sample_bilinear` do motor: a borda no texel `p` lê a cobertura em `p + A·u(p)`.
/// Medido: 0 de 4 678 texels em desacordo; o sinal trocado discorda em 2 373 (o controlo abaixo).
#[test]
fn a_lei_do_deslocamento_e_a_do_fedisplacementmap() {
    const W: usize = 128;
    const A: f32 = 12.0;
    let disco: &[u8] =
        include_bytes!("../../../../../docs/Painter/ferramentas/oraculo_deslocamento/disco.u8");
    let rg: &[u8] =
        include_bytes!("../../../../../docs/Painter/ferramentas/oraculo_deslocamento/mapa_rg.u8");
    let ink: &[u8] = include_bytes!(
        "../../../../../docs/Painter/ferramentas/oraculo_deslocamento/saida_inkscape_1.4.4.u8"
    );
    let cov: Vec<f32> = disco.iter().map(|&c| f32::from(c) / 255.0).collect();
    let m = Mapa {
        d: rg
            .chunks(2)
            .map(|c| {
                [
                    2.0 * f32::from(c[0]) / 255.0 - 1.0,
                    2.0 * f32::from(c[1]) / 255.0 - 1.0,
                ]
            })
            .collect(),
        x0: 0,
        y0: 0,
        w: W,
        h: W,
    };
    let desacordo = |sinal: f32| {
        (0..W * W)
            .filter(|&i| {
                let (x, y) = ((i % W) as f32, (i / W) as f32);
                let (u, v) = m.le(x, y);
                let c = sample_bilinear(&cov, W, W, x + sinal * A * u, y + sinal * A * v);
                (c > 0.5) != (ink[i] > 127)
            })
            .count()
    };
    let coberto = ink.iter().filter(|&&c| c > 127).count();
    assert!(
        coberto > 4000,
        "a fixtura do Inkscape não tem o disco ({coberto})"
    );
    assert!(
        desacordo(-1.0) > 1000,
        "controlo: o sinal trocado tem de discordar ({})",
        desacordo(-1.0)
    );
    assert!(
        desacordo(1.0) <= 2,
        "a lei discorda do Inkscape em {} texels",
        desacordo(1.0)
    );
}

/// Sem Paper Edge, a janela do composite não cresce (o caminho de hoje, ao byte).
#[test]
fn sem_paper_edge_o_alcance_e_zero() {
    let b = ph2d_painter_brush::BrushSpec::default();
    assert_eq!(alcance_do_papel(&b, &[], None, NoiseTile::NONE), 0.0);
    let com = ph2d_painter_brush::BrushSpec {
        paper_edge: 1.0,
        ..b
    };
    let a = alcance_do_papel(&com, &[], None, NoiseTile::NONE);
    assert!(a > 0.5 && a < 32.0, "alcance do papel interno a 1: {a}");
}
