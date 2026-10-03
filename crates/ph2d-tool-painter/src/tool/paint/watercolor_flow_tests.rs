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
        let (r, g) =
            reguas(&Amostrador::flow(s, &Imagens::default(), NoiseTile::NONE).expect("procedural"));
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
        Imagens::default(),
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

/// ⭐ **O memo das estatísticas conhece a IMAGEM, não só as settings.** Duas imagens de Flow com as
/// MESMAS settings (`kind: Image`) e conteúdo deslocado `+20` níveis normalizam para o MESMO mapa — a
/// média de cada uma é a dela. Com a chave antiga (só settings) a segunda herdava a média da
/// primeira, e o mapa saía todo puxado para um lado.
///
/// **Mutação que tem de sangrar:** tirar o `id_img` da [`Amostrador::chave`].
#[test]
fn trocar_a_imagem_nao_herda_as_estatisticas_da_anterior() {
    let (w, h) = (64u32, 64u32);
    let a: Vec<u8> = (0..w * h)
        .map(|i| (100.0 + 60.0 * (((i % w) as f32 + (i / w) as f32) / 7.0).sin()) as u8)
        .collect();
    let b: Vec<u8> = a.iter().map(|&v| v + 20).collect();
    let s = TextureSettings {
        kind: TextureKind::Image,
        ..ph2d_painter_brush::BrushSpec::default().edge_flow
    };
    let ia = Imagens {
        fluxo: Some(ImageMask {
            lum: &a,
            width: w,
            height: h,
        }),
        fluxo_versao: 41,
        ..Imagens::default()
    };
    let ib = Imagens {
        fluxo: Some(ImageMask {
            lum: &b,
            width: w,
            height: h,
        }),
        fluxo_versao: 42,
        ..Imagens::default()
    };
    let fa = Amostrador::flow(s, &ia, NoiseTile::NONE).expect("imagem");
    let fb = Amostrador::flow(s, &ib, NoiseTile::NONE).expect("imagem");
    let (ea, eb) = (fa.estat(), fb.estat());
    let mut pior = 0.0f32;
    for p in 0..200i64 {
        let (x, y) = (p * 7 % 61, p * 13 % 59);
        let (ua, va) = fa.unidade(&ea, x, y);
        let (ub, vb) = fb.unidade(&eb, x, y);
        pior = pior.max((ua - ub).abs()).max((va - vb).abs());
    }
    assert!(
        pior < 0.02,
        "a 2.ª imagem herdou as estatísticas da 1.ª — desvio {pior:.3}"
    );
}

/// A pré-visualização desenha o padrão com o Size que o motor AMOSTRA: o do artista vezes a escala
/// normalizada — a mesma conta que o [`Amostrador::flow`] faz (o Classic e a imagem: o Size verbatim).
///
/// **Mutação que tem de sangrar:** o `size_efetivo_do_flow` devolver o Size do artista.
#[test]
fn o_size_do_preview_e_o_que_o_motor_amostra() {
    for &k in &ph2d_painter_brush::FLOW_KINDS[1..] {
        let mut s = ph2d_painter_brush::BrushSpec::default().edge_flow;
        s.kind = k;
        s.size = [2.0, 2.0];
        let imagens = Imagens::default();
        let Some(Amostrador::Textura { s: amostrado, .. }) =
            Amostrador::flow(s, &imagens, NoiseTile::NONE)
        else {
            panic!("{k:?}: um padrão é uma textura");
        };
        assert_eq!(size_efetivo_do_flow(&s), amostrado[0].size, "{k:?}");
    }
    let s = ph2d_painter_brush::BrushSpec::default().edge_flow;
    assert_eq!(size_efetivo_do_flow(&s), s.size, "o Classic");
}

/// ⭐ **O Classic é um padrão como os outros**: o Flow Size escala e o Flow Angle roda as coordenadas
/// do ruído, pela lei das texturas (`R(−θ)·p·Size`). No neutro não há transformação nenhuma — o
/// [`warp_offset`] verbatim (a régua do Classic ao byte, gravada antes da wave, continua a passar).
///
/// **Mutação que tem de sangrar:** o `Classico::le` ignorar o Size (ou o Angle).
#[test]
fn o_classic_tem_size_e_angle() {
    let mut f = ph2d_painter_brush::BrushSpec::default().edge_flow;
    assert!(
        Classico::de(&f, NoiseTile::NONE).is_none(),
        "o neutro é o warp_offset verbatim"
    );
    f.size = [2.0, 2.0];
    let c = Classico::de(&f, NoiseTile::NONE).expect("Size 2");
    for (x, y) in [(3.0f32, 5.0f32), (40.5, 17.25), (101.0, 63.0)] {
        assert_eq!(
            c.le(x, y),
            warp_offset(2.0 * x, 2.0 * y, NoiseTile::NONE),
            "Size 2 em ({x}, {y})"
        );
    }
    f.size = [1.0, 1.0];
    f.angle_deg = 90;
    let c = Classico::de(&f, NoiseTile::NONE).expect("Angle 90");
    let [cs, sn] = angle_basis(90);
    for (x, y) in [(3.0f32, 5.0f32), (40.5, 17.25)] {
        let (qx, qy) = (x * cs + y * sn, -x * sn + y * cs);
        assert_eq!(
            c.le(x, y),
            warp_offset(qx, qy, NoiseTile::NONE),
            "Angle 90 em ({x}, {y})"
        );
    }
}

/// A pré-visualização do Classic é o ruído do motor no enquadramento da faixa partilhada (~3 ladrilhos
/// de 256 px na largura): o pixel `(px, py)` mostra o canal X em `q = base·256·Size`.
#[test]
fn o_preview_do_classic_e_o_ruido_do_motor() {
    let (w, h) = (140u32, 70u32);
    let mut buf = vec![0u8; (w * h * 4) as usize];
    render_classic_flow_preview(2.0, 0, &mut buf, w, h);
    let step = 3.0 / w as f32;
    for (px, py) in [(0u32, 0u32), (71, 33), (139, 69)] {
        let (bu, bv) = ((px as f32 + 0.5) * step, (py as f32 + 0.5) * step);
        let q = TEX_TILE_BASE_PX * 2.0;
        let x = warp_offset(bu * q, bv * q, NoiseTile::NONE).0;
        let g = ((0.5 + 0.5 * x).clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
        assert_eq!(buf[((py * w + px) * 4) as usize], g, "({px}, {py})");
    }
}

/// Sem Paper Edge, a janela do composite não cresce (o caminho de hoje, ao byte).
#[test]
fn sem_paper_edge_o_alcance_e_zero() {
    let b = ph2d_painter_brush::BrushSpec::default();
    assert_eq!(
        alcance_do_papel(&b, &[], Imagens::default(), NoiseTile::NONE),
        0.0
    );
    let com = ph2d_painter_brush::BrushSpec {
        paper_edge: 1.0,
        ..b
    };
    let a = alcance_do_papel(&com, &[], Imagens::default(), NoiseTile::NONE);
    assert!(a > 0.5 && a < 32.0, "alcance do papel interno a 1: {a}");
}
