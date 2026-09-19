//! ⏱️⭐⭐⭐ **A RÉGUA DO TERMINADOR** — o report do dono de 2026-09-18 (*«em `Thin Walled: Solid` não
//! há transição suave entre a área iluminada e a área sombreada da esfera, mas uma linha dura»*,
//! duas fotos da cena `=33` com a seta em cima do vinco).
//!
//! # ⚠️ A fixtura é a CENA QUE ELE CORREU, e a régua DECOMPÕE em vez de julgar
//!
//! A 1.ª redacção desta sonda montava uma bola sozinha na origem e binava a luminância por `N·L`.
//! Ela leu o perfil a atravessar `N·L = 0` **liso** e o pior salto a `0,78` — ⇒ *a fixtura não
//! continha o fenómeno*, que é a lei que esta casa já pagou cinco vezes.
//!
//! O que esta sonda faz: reconstrói o quadro do dono (a `=33`, o chão, o céu, a sombra, a
//! curvatura) e, sobre os píxeis da ESFERA, acha o pixel de maior **segunda diferença** da
//! luminância e imprime a vizinhança dele com **todos os canais lado a lado** — `N·L` · a
//! curvatura que o campo entrega · a visibilidade da lâmpada · a luminância.
//!
//! ⭐ *O canal cujo salto COINCIDE com o da luminância é a causa; os outros ficam ilibados com
//! número.*

use ph2d_field_render::{Orbit, Surfaces};

/// O material maciço com que o dono a viu: `Subsurface` no máximo, `Thin Walled: Solid`.
fn macico() -> ph2d_material::OpenPbr {
    ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    }
}

/// ⏱️ **SONDA — de que canal é o vinco.**
#[test]
#[ignore = "sonda de diagnóstico: a régua do report de 18/09"]
fn sonda_o_terminador_da_esfera() {
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do dono");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (w, h) = (320u32, 240u32);
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        // ⚠️⚠️ **A radiância passa pela PORTA do produto** — a 1.ª redacção escreveu
        // `[luz.intensity; 3]`, sem a cor e sem o `π` do [`crate::lights::radiance_at_one`], e a
        // lâmpada saiu `π×` fraca: o céu dominava e o terminador era uma rampa de `99` a `108`
        // onde o produto tem um degrau. *Uma sonda que reescreve a conversão do produto mede outro
        // programa.*
        radiance_at_one: crate::lights::radiance_at_one(luz),
    };
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let (right, up, fwd) = cam.basis();
    let para_mundo =
        |v: [f32; 3]| [0, 1, 2].map(|k| v[0] * right[k] + v[1] * up[k] + v[2] * fwd[k]);

    let opaco = ph2d_material::OpenPbr {
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    for (nome, m, com_sombra) in [
        ("opaco  · sombra SIM", opaco, true),
        ("maciço · sombra SIM", macico(), true),
        ("maciço · sombra NÃO", macico(), false),
    ] {
        let mut g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        let mats = [m.prepare()];
        let surfaces = Surfaces {
            all: &mats,
            owners: None,
        };
        if surfaces
            .all
            .iter()
            .any(ph2d_material::Surface::reads_curvature)
            && let Some(b) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        {
            let mut eval = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
            g.curvature = ph2d_field_render::curvatura::do_gbuffer(
                &mut eval,
                &g,
                ph2d_field_render::curvatura::eps_para(b.radius),
            );
        }
        let uma = [onde];
        let lampadas: &[[f32; 3]] = if com_sombra { &uma } else { &[] };
        let sh = ph2d_field_render::shadow_pass_on(&doc, &reg, &cam, &g, lampadas, chao);
        let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
        let px = ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &[lampada],
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        );

        // Os píxeis da ESFERA — ela vive em `x > 0,1` no mundo.
        let da_bola = |i: usize| g.hit[i] && g.point[i][0] > 0.1;
        let lum = |i: usize| {
            let b = i * 4;
            0.2126 * f32::from(px[b])
                + 0.7152 * f32::from(px[b + 1])
                + 0.0722 * f32::from(px[b + 2])
        };
        let ndl = |i: usize| {
            let p = g.point[i];
            let d = [onde[0] - p[0], onde[1] - p[1], onde[2] - p[2]];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let n = para_mundo(g.normal[i]);
            (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / r.max(1e-9)
        };

        let (wu, hu) = (w as usize, h as usize);
        // ⭐⭐⭐ **A JANELA É FIXA nas três células** — ela é a que o canal da sombra nomeou, e uma
        // janela que se move com o máximo de cada célula não as deixa comparar.
        const LINHA: usize = 99;
        const COLUNA: usize = 284;
        let tapados = (0..wu * hu)
            .filter(|&i| da_bola(i) && sh.at(0, i) < 0.99)
            .count();
        let total = (0..wu * hu).filter(|&i| da_bola(i)).count();
        println!("\n  ── {nome} ── {tapados} de {total} pixels da bola com vis < 0,99");
        println!("     x  ·    N·L  ·  vis  ·   lum  · salto");
        let mut ant = f32::NAN;
        for x in COLUNA - 5..(COLUNA + 6).min(wu) {
            let i = LINHA * wu + x;
            if !da_bola(i) {
                continue;
            }
            let l = lum(i);
            println!(
                "   {x:>3}  · {:>6.3} · {:>5.3} · {l:>6.1}  · {:>+6.1}",
                ndl(i),
                sh.at(0, i),
                if ant.is_nan() { 0.0 } else { l - ant }
            );
            ant = l;
        }

        // ⭐⭐⭐ **A MAGNITUDE sobre o TERMINADOR INTEIRO**, e não uma linha só: a cava segue a
        // elipse do terminador, logo é isso que o olho lê como uma LINHA.
        let mut saltos: Vec<f32> = Vec::new();
        for y in 0..hu {
            for x in 1..wu - 1 {
                let (a, b, c) = (y * wu + x - 1, y * wu + x, y * wu + x + 1);
                if !da_bola(a) || !da_bola(b) || !da_bola(c) || ndl(b).abs() > 0.15 {
                    continue;
                }
                saltos.push((lum(a) - 2.0 * lum(b) + lum(c)).abs());
            }
        }
        saltos.sort_by(f32::total_cmp);
        let q = |f: f32| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let k = ((saltos.len() as f32 - 1.0) * f).round() as usize;
            saltos.get(k).copied().unwrap_or(0.0)
        };
        println!(
            "     banda |N·L| <= 0,15 ({} px): 2.ª dif p50 {:.2} · p99 {:.2} · máx {:.2}",
            saltos.len(),
            q(0.5),
            q(0.99),
            q(1.0)
        );
    }

    // ── PARTE B — a ACNE isolada: uma bola SOZINHA ───────────────────────────────────────────
    //
    // ⭐⭐⭐ **Um corpo CONVEXO não se pode tapar a si próprio** (é a frase que o próprio passe de
    // sombra escreve), logo aqui toda visibilidade abaixo de `1` é ACNE e mede-se sem ter de a
    // separar de sombra legítima — na `=33` a lâmina pode tapar a bola a sério.
    let so_a_bola = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.42 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a bola sozinha");
    let g = ph2d_field_render::trace(&so_a_bola, &reg, &cam, w, h);
    let sh = ph2d_field_render::shadow_pass_on(&so_a_bola, &reg, &cam, &g, &[onde], None);
    let (wu, hu) = (w as usize, h as usize);
    let (mut tapados, mut total, mut pior) = (0usize, 0usize, 1.0f32);
    let mut banda = (1.0f32, -1.0f32);
    for i in 0..wu * hu {
        if !g.hit[i] {
            continue;
        }
        total += 1;
        let v = sh.at(0, i);
        if v < 0.99 {
            tapados += 1;
            pior = pior.min(v);
            let p = g.point[i];
            let d = [onde[0] - p[0], onde[1] - p[1], onde[2] - p[2]];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let n = para_mundo(g.normal[i]);
            let c = (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / r.max(1e-9);
            banda = (banda.0.min(c), banda.1.max(c));
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let pct = 100.0 * tapados as f32 / total.max(1) as f32;
    println!(
        "\n  ── BOLA SOZINHA (convexa: toda sombra aqui é ACNE) ──\n     {tapados} de {total} \
         pixels ({pct:.1} %) com vis < 0,99 · pior vis {pior:.3} · a banda vive em N·L ∈ [{:.3}, \
         {:.3}]",
        banda.0, banda.1
    );

    // ── PARTE C — a PAREDE FINA com a luz ATRÁS ──────────────────────────────────────────────
    //
    // ⛔⛔ **É aqui que a cura óbvia pode partir a outra metade da wave:** o `translucent` lê
    // `−N·L` de propósito, e com a luz atrás a face que se VÊ tem `N·L < 0`. Um raio de sombra
    // lançado dessa face para a lâmpada atravessa a própria folha ⇒ se ele contar, a folha APAGA.
    let folha = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Box {
                half: [0.45, 0.42, 0.015],
                round: 0.012,
                chamfer: 0.0,
            },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a folha sozinha");
    // A luz ATRÁS: o lugar de abertura espelhado na profundidade da vista.
    let atras = {
        let o = crate::lights::opening_place(&cam);
        [
            o[0] - 2.0 * (o[0] * fwd[0] + o[1] * fwd[1] + o[2] * fwd[2]) * fwd[0],
            o[1] - 2.0 * (o[0] * fwd[0] + o[1] * fwd[1] + o[2] * fwd[2]) * fwd[1],
            o[2] - 2.0 * (o[0] * fwd[0] + o[1] * fwd[1] + o[2] * fwd[2]) * fwd[2],
        ]
    };
    let fina = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: true,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let mats = [fina.prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let g = ph2d_field_render::trace(&folha, &reg, &cam, w, h);
    let lampada = ph2d_field_render::PointLamp {
        world: atras,
        radiance_at_one: crate::lights::radiance_at_one(luz),
    };
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    for (nome, usa) in [("sem sombra", false), ("com sombra", true)] {
        let uma = [atras];
        let sh = ph2d_field_render::shadow_pass_on(
            &folha,
            &reg,
            &cam,
            &g,
            if usa { &uma[..] } else { &[][..] },
            None,
        );
        let px = ph2d_field_render::shade_render(
            &g,
            &cam,
            &surfaces,
            &ph2d_field_render::Lighting {
                lamps: &sem_ecra,
                points: &[lampada],
                sky: &crate::render_light::StudioSky,
                shadows: Some(&sh),
            },
            ph2d_view_transform::Look::default(),
            [0, 0, 0, 0],
        );
        let (mut soma, mut n, mut vmin) = (0.0f64, 0usize, 1.0f32);
        for i in 0..g.hit.len() {
            if !g.hit[i] {
                continue;
            }
            let b = i * 4;
            soma += f64::from(
                0.2126 * f32::from(px[b])
                    + 0.7152 * f32::from(px[b + 1])
                    + 0.0722 * f32::from(px[b + 2]),
            );
            n += 1;
            vmin = vmin.min(sh.at(0, i));
        }
        #[allow(clippy::cast_precision_loss)]
        let m = soma / n.max(1) as f64;
        println!(
            "  ── FOLHA · luz ATRÁS · {nome} ── lum média {m:.1} sobre {n} px · vis mínima {vmin:.3}"
        );
    }
}

/// ⏱️⭐⭐⭐ **A RÉGUA QUE SEPARA O LADO APROVADO DO REPROVADO** — o 2.º report de 18/09.
///
/// ⚠️⚠️ O dono mandou DUAS fotos e disse *«o resultado em Thin Walled é melhor (mais suave a
/// transição)»* ⇒ **ele aprovou uma das duas metades da wave**, e é nela que a barra se calibra.
/// A régua de ecrã que eu usei antes mede o realce especular e a sombra do vizinho; esta mede a
/// coisa que ele aponta: a **curvatura do perfil `lum(N·L)`**, que é o que o olho lê como vinco.
#[test]
#[ignore = "sonda de diagnóstico: o vinco que o dono aponta"]
fn sonda_o_vinco_contra_o_lado_aprovado() {
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do dono");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (onde, luz) = crate::lights::opening_light(&cam);
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let (right, up, fwd) = cam.basis();
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    println!("  material            ·  pior curvatura do perfil · onde (N·L) · 2.º pior · onde");
    for (nome, m) in [
        ("opaco             ", base),
        (
            "maciço  (Solid)   ",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                ..base
            },
        ),
        (
            "parede fina (APROV)",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: true,
                ..base
            },
        ),
    ] {
        let (g, _, px) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao,
            sem_ceu: false,
        });
        const FAIXAS: usize = 200;
        let (mut soma, mut conta) = ([0.0f64; FAIXAS], [0usize; FAIXAS]);
        for i in 0..g.hit.len() {
            if !g.hit[i] || g.point[i][0] <= 0.1 {
                continue;
            }
            let p = g.point[i];
            let d = [onde[0] - p[0], onde[1] - p[1], onde[2] - p[2]];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let n = [0, 1, 2].map(|k| {
                g.normal[i][0] * right[k] + g.normal[i][1] * up[k] + g.normal[i][2] * fwd[k]
            });
            let c = ((n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / r.max(1e-9)).clamp(-1.0, 1.0);
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            let k = (((c + 1.0) * 0.5 * FAIXAS as f32) as usize).min(FAIXAS - 1);
            let b = i * 4;
            soma[k] += f64::from(
                0.2126 * f32::from(px[b])
                    + 0.7152 * f32::from(px[b + 1])
                    + 0.0722 * f32::from(px[b + 2]),
            );
            conta[k] += 1;
        }
        let mut perfil: Vec<(f32, f32)> = Vec::new();
        for k in 0..FAIXAS {
            if conta[k] >= 20 {
                #[allow(clippy::cast_precision_loss)]
                let c = (k as f32 + 0.5) / FAIXAS as f32 * 2.0 - 1.0;
                #[allow(clippy::cast_precision_loss)]
                perfil.push((c, (soma[k] / conta[k] as f64) as f32));
            }
        }
        let mut vincos: Vec<(f32, f32)> = perfil
            .windows(3)
            .map(|w| ((w[2].1 - 2.0 * w[1].1 + w[0].1).abs(), w[1].0))
            .collect();
        vincos.sort_by(|a, b| b.0.total_cmp(&a.0));
        println!(
            "  {nome}  ·  {:>22.3} · {:>10.3} · {:>7.3} · {:>6.3}",
            vincos[0].0, vincos[0].1, vincos[1].0, vincos[1].1
        );
        // A vizinhança do terminador, em unidades de `N·L`.
        let mut linha = String::new();
        for (c, l) in &perfil {
            if c.abs() <= 0.22 {
                linha.push_str(&format!(" {c:+.3}:{l:.1}"));
            }
        }
        println!("      perfil:{linha}");
    }
    println!(
        "  ⚠️ sin(π/32) = {:.4} — o x da 1.ª amostra da quadratura do `integrate_burley`",
        (core::f32::consts::PI / 32.0).sin()
    );
}

/// ⏱️⭐⭐⭐ **A LEI SOZINHA, sem cena e sem ruído** — varre `N·L` finamente e mede a curvatura da
/// resposta. É a única régua que separa *«a lei é lisa e a cena é que é ruidosa»* de *«a lei tem
/// vincos»*, e ela põe o lado APROVADO (parede fina) ao lado do reprovado (maciço).
#[test]
#[ignore = "sonda de diagnóstico: a lei sozinha"]
fn sonda_a_lei_sozinha() {
    const N: usize = 4001;
    let curva = |m: ph2d_material::OpenPbr, k: f32| -> Vec<f32> {
        let s = m.prepare().at_curvature(k);
        (0..N)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let c = -1.0 + 2.0 * (i as f32) / (N as f32 - 1.0);
                let sin = (1.0 - c * c).max(0.0).sqrt();
                // `n` em z, a luz no plano `xz` com `N·L = c`; o olho de frente.
                let r = s.direct(
                    [0.0, 0.0, 1.0],
                    [0.0, 0.0, 1.0],
                    [sin, 0.0, c],
                    [1.0, 1.0, 1.0],
                );
                0.2126 * r[0] + 0.7152 * r[1] + 0.0722 * r[2]
            })
            .collect()
    };
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    };
    // A escala da bola da `=33`: raio `0,42` ⇒ curvatura `1/0,42`.
    let k = 1.0f32 / 0.42;
    println!(
        "  (curvatura {k:.3}; sin(π/32) = {:.4})",
        (core::f32::consts::PI / 32.0).sin()
    );
    println!(
        "  material                      ·   pior 2.ª dif · onde (N·L) ·  largura do envolvimento"
    );
    for (nome, m) in [
        ("opaco                       ", base),
        (
            "maciço  raio 1,0 (omissão)  ",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                ..base
            },
        ),
        (
            "maciço  raio 0,1            ",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                subsurface_radius: 0.1,
                ..base
            },
        ),
        (
            "maciço  raio 4,0            ",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                subsurface_radius: 4.0,
                ..base
            },
        ),
        (
            "parede fina (APROVADA)      ",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: true,
                ..base
            },
        ),
    ] {
        let y = curva(m, k);
        let passo = 2.0 / (N as f32 - 1.0);
        let mut pior = (0.0f32, 0.0f32);
        for i in 1..N - 1 {
            #[allow(clippy::cast_precision_loss)]
            let c = -1.0 + 2.0 * (i as f32) / (N as f32 - 1.0);
            // ⚠️ As pontas do varrimento são o PÓLO de trás e o de frente — ali a curva acaba, e um
            // extremo de domínio não é um vinco que alguém veja numa esfera.
            if c.abs() > 0.97 {
                continue;
            }
            let d2 = (y[i + 1] - 2.0 * y[i] + y[i - 1]).abs() / (passo * passo);
            if d2 > pior.0 {
                pior = (d2, c);
            }
        }
        // A LARGURA do envolvimento: onde a resposta passa de 5 % para 95 % do máximo.
        let topo = y.iter().copied().fold(0.0f32, f32::max).max(1e-9);
        let onde = |f: f32| {
            #[allow(clippy::cast_precision_loss)]
            y.iter()
                .position(|&v| v >= f * topo)
                .map_or(f32::NAN, |i| -1.0 + 2.0 * (i as f32) / (N as f32 - 1.0))
        };
        println!(
            "  {nome}  · {:>12.1} · {:>10.4} ·  N·L de {:+.3} a {:+.3}  ({:.3})",
            pior.0,
            pior.1,
            onde(0.05),
            onde(0.95),
            onde(0.95) - onde(0.05)
        );
    }
}

/// ⏱️⭐⭐⭐ **A FOTO** — a lição que esta casa já pagou duas vezes: *uma suíte verde não prova que o
/// que o dono VÊ mudou*. Grava `.ppm` em `$PH2D_TERM_DUMP` (ou no `/tmp`) para se poder OLHAR.
#[test]
#[ignore = "sonda de diagnóstico: grava a imagem para se poder olhar"]
fn sonda_fotografa_o_terminador() {
    let dir = std::env::var("PH2D_TERM_DUMP").unwrap_or_else(|_| "/tmp".to_owned());
    // ⭐⭐⭐ **A pergunta que decide o que a linha É:** com `PH2D_TERM_SO_A_BOLA=1` a LÂMINA sai da
    // cena e fica a bola sozinha, no mesmo enquadramento e com a mesma luz. Se a linha desaparecer,
    // ela é a SOMBRA da placa; se ficar, é o TERMINADOR. *Nenhuma régua desta jornada respondeu a
    // isto, e as duas curas foram desenhadas sem a resposta.*
    let doc = if std::env::var("PH2D_TERM_SO_A_BOLA").is_ok() {
        ph2d_field::FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                ph2d_field::Primitive::Sphere { radius: 0.42 },
                ph2d_field::Xform {
                    translation: [0.55, 0.0, 0.0],
                    ..ph2d_field::Xform::IDENTITY
                },
            )],
            ph2d_field::NodeId(0),
        )
        .expect("a bola sozinha")
    } else {
        crate::smoke::scenes::edge::cena_33().expect("a cena do dono")
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    // ⚠️ O dono ZOOMOU na bola: o enquadramento tem de ser o dele, senão a foto mostra outra coisa.
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let (onde, luz) = crate::lights::opening_light(&cam);
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let solid = |r: f32| ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        subsurface_radius: r,
        ..base
    };
    for (nome, m) in [
        ("opaco", base),
        ("solid", solid(1.0)),
        ("r010", solid(0.1)),
        ("r030", solid(0.3)),
        (
            "fina",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: true,
                ..base
            },
        ),
    ] {
        let (g, _, px) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao,
            sem_ceu: false,
        });
        let mut ppm = format!("P6\n{} {}\n255\n", g.width, g.height).into_bytes();
        for i in 0..(g.width * g.height) as usize {
            ppm.extend_from_slice(&px[i * 4..i * 4 + 3]);
        }
        let caminho = format!("{dir}/terminador_{nome}.ppm");
        std::fs::write(&caminho, ppm).expect("gravar");
        println!("  gravado {caminho}");
    }
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
//  O GATE — três metades, e cada uma é um defeito MEDIDO
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O quadro do dono em CPU: devolve `(gbuffer, sombras, bytes)`.
/// ⭐⭐⭐ **O CÉU APAGADO** — a condição em que a comparação com o oráculo tem UMA incógnita só.
///
/// Com o céu ligado há duas coisas a casar (o ambiente e a lâmpada) e a nossa `StudioSky` é um
/// gradiente que o oráculo não reproduz. Com ele apagado sobra **só a lâmpada**, e as duas leis de
/// queda são a mesma (`1/r²`) ⇒ **um único factor de escala** liga os dois lados, e tudo o que
/// sobrar depois de o ajustar é a LEI. *Uma comparação com duas incógnitas livres não afirma nada.*
struct CeuPreto;

impl ph2d_material::Environment for CeuPreto {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        [0.0; 3]
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        [0.0; 3]
    }
}

struct Quadro<'a> {
    doc: &'a ph2d_field::FieldDoc,
    m: ph2d_material::OpenPbr,
    cam: &'a Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    com_sombra: bool,
    chao: Option<ph2d_field_render::Ground>,
    /// ⭐ Apaga o céu — a condição da comparação com o oráculo. Ver [`CeuPreto`].
    sem_ceu: bool,
}

/// A vista é a mesma nos dois gates — a cena é que muda.
const W: u32 = 320;
const H: u32 = 240;

fn quadro(
    q: &Quadro<'_>,
) -> (
    ph2d_field_render::Gbuffer,
    ph2d_field_render::Shadows,
    Vec<u8>,
) {
    let (doc, m, cam, onde, luz, com_sombra, chao) =
        (q.doc, q.m, q.cam, q.onde, q.luz, q.com_sombra, q.chao);
    let (w, h) = (W, H);
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let mut g = ph2d_field_render::trace(doc, &reg, cam, w, h);
    let mats = [m.prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    if surfaces
        .all
        .iter()
        .any(ph2d_material::Surface::reads_curvature)
        && let Some(b) = ph2d_field_eval::bounds::bounding_ball(doc, &reg)
    {
        let mut eval = ph2d_field_eval::hybrid::Hybrid::new(doc, &reg);
        g.curvature = ph2d_field_render::curvatura::do_gbuffer(
            &mut eval,
            &g,
            ph2d_field_render::curvatura::eps_para(b.radius),
        );
    }
    let uma = [onde];
    let sh = ph2d_field_render::shadow_pass_on(
        doc,
        &reg,
        cam,
        &g,
        if com_sombra { &uma[..] } else { &[][..] },
        chao,
    );
    // ⭐⭐⭐ **A BORDA MOLE, pela MESMA porta que o app usa** — ⚠️ a 1.ª redacção desta sonda
    // derivava a distância de espalhamento aqui, o que a fazia a segunda resposta à mesma pergunta.
    let mut sh = sh;
    if let Some(espalha) = crate::materials::maior_espalhamento(&surfaces) {
        let raio = ph2d_field_render::sss_shadow::raio_em_pixeis(cam, h, espalha);
        let canais = (0..1)
            .map(|l| ph2d_field_render::sss_shadow::blur_por_canal(&g, sh.lamp_channel(l), raio))
            .collect();
        sh.set_soft(canais);
    }
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    let preto = CeuPreto;
    let estudio = crate::render_light::StudioSky;
    let ceu: &(dyn ph2d_material::Environment + Sync) = if q.sem_ceu { &preto } else { &estudio };
    let px = ph2d_field_render::shade_render(
        &g,
        cam,
        &surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: &[ph2d_field_render::PointLamp {
                world: onde,
                radiance_at_one: crate::lights::radiance_at_one(luz),
            }],
            sky: ceu,
            shadows: Some(&sh),
        },
        ph2d_view_transform::Look::default(),
        [0, 0, 0, 0],
    );
    (g, sh, px)
}

/// ⛔⛔⛔ **RECUSA MEDIDA — «marchar o raio de costas» foi construída, fotografada e REVERTIDA.**
///
/// O passe de sombra escreve `vis = 1,0` para todo ponto de costas para a luz, o que **trunca** no
/// terminador a sombra que um vizinho projecta. Isso é um defeito real, e o comentário desse filtro
/// previa-o por escrito. A cura — lançar o raio na mesma e só contar o que estiver depois de ele
/// SAIR do próprio corpo, com o `t` do estimador de penumbra recontado a partir da saída — foi
/// construída inteira, com o gémeo em WGSL, as 6 paridades verdes e 4 mutações a sangrar.
///
/// ⛔ **E a FOTO reprovou-a.** Na cena `=33`, com o enquadramento do dono:
///
/// | | o que se vê |
/// |---|---|
/// | antes | a borda da sombra é **limpa**, embora dura |
/// | com a cura | a borda alarga **e ganha um FIO escuro SERRILHADO** por cima |
///
/// ⚠️ O serrilhado é a assinatura da causa: **um `if` por pixel** (`N·L <= 0`) escolhia entre duas
/// maneiras de calcular a mesma grandeza, e *a fronteira entre elas desenha-se*. Perto do
/// terminador o raio de costas rasa a própria peça durante `~√(2R·ε)` antes de sair, logo o `t`
/// dele reconta tarde e a penumbra sai mais dura que a do vizinho de frente.
///
/// ⛔ **E apagar o ramo (um só caminho, todo raio a partir do ponto) é PIOR:** sem o ergue pela
/// normal volta a **acne** — a foto mostra riscos claros ao longo do terminador, e a banda passa de
/// `p99 3,71` para `13,06`.
///
/// ⇒ *a truncagem é um defeito INVISÍVEL nesta cena e o fio é VISÍVEL*, logo shipa-se a truncagem.
/// A cura de fundo é outra e está nomeada no [`10` §11.6]: **a visibilidade que um termo
/// TRANSMISSIVO lê tem de ser borrada pela distância de espalhamento** — é isso que faz a sombra
/// num jade ter a borda mole, e nenhuma das duas referências o escreve.
///
/// Os dois gates abaixo ficam: eles são as propriedades que aquela cura teria partido, e é por eles
/// que uma segunda tentativa sabe onde bate.
/// ⭐⭐⭐ **A FOLHA COM A LUZ ATRÁS NÃO SE APAGA** — a metade que a cura óbvia partia.
///
/// Marchar o raio de costas como os outros **cura a esfera e mata a folha**: ele atravessa a
/// própria lâmina, lê-a como obstáculo, e a luminância média cai de `83,7` para **`73,8`** com a
/// `vis` mínima em `0,000` (medido). ⇒ o raio só conta o que estiver **depois de ele sair**.
///
/// ⚠️ A barra é `1 %` da média porque a resposta certa é a IGUALDADE: não há nada entre a folha e
/// a luz senão ela própria.
#[test]
fn a_folha_com_a_luz_atras_nao_se_apaga() {
    let cam = Orbit::default();
    let (_, luz) = crate::lights::opening_light(&cam);
    let (_, _, fwd) = cam.basis();
    let o = crate::lights::opening_place(&cam);
    let k = o[0] * fwd[0] + o[1] * fwd[1] + o[2] * fwd[2];
    let atras = [0, 1, 2].map(|i| o[i] - 2.0 * k * fwd[i]);
    let folha = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Box {
                half: [0.45, 0.42, 0.015],
                round: 0.012,
                chamfer: 0.0,
            },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a folha");
    let fina = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: true,
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let media = |com_sombra: bool| {
        let (g, _, px) = quadro(&Quadro {
            doc: &folha,
            m: fina,
            cam: &cam,
            onde: atras,
            luz,
            com_sombra,
            chao: None,
            sem_ceu: false,
        });
        let (mut soma, mut n) = (0.0f64, 0usize);
        for i in 0..g.hit.len() {
            if !g.hit[i] {
                continue;
            }
            let b = i * 4;
            soma += f64::from(
                0.2126 * f32::from(px[b])
                    + 0.7152 * f32::from(px[b + 1])
                    + 0.0722 * f32::from(px[b + 2]),
            );
            n += 1;
        }
        assert!(n > 5_000, "só {n} pixels de folha — a fixtura não a mostra");
        #[allow(clippy::cast_precision_loss)]
        let m = soma / n as f64;
        m
    };
    let (sem, com) = (media(false), media(true));
    assert!(
        (com - sem).abs() <= sem * 0.01,
        "a folha com a luz ATRÁS lê {com:.1} com sombra contra {sem:.1} sem — ela está a ler o \
         próprio corpo como obstáculo, e é isso que apaga a luz que a atravessa"
    );
}

/// ⭐⭐⭐ **UM CORPO CONVEXO NÃO SE TAPA A SI PRÓPRIO** — o piso que as outras duas não vêem.
///
/// ⚠️ Sem esta metade, uma «cura» que pusesse toda a metade escura de toda peça em sombra passaria
/// nas outras duas. Medido: `0` de `12 924` pixels com `vis < 0,99` numa esfera sozinha, antes e
/// depois — *a cura não pode ganhar isto escurecendo o mundo.*
#[test]
fn um_corpo_convexo_nao_se_tapa_a_si_proprio() {
    let cam = Orbit::default();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (onde, _) = crate::lights::opening_light(&cam);
    let bola = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.42 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("a bola sozinha");
    let g = ph2d_field_render::trace(&bola, &reg, &cam, 320, 240);
    let sh = ph2d_field_render::shadow_pass_on(&bola, &reg, &cam, &g, &[onde], None);
    let (mut tapados, mut total) = (0usize, 0usize);
    for i in 0..g.hit.len() {
        if !g.hit[i] {
            continue;
        }
        total += 1;
        if sh.at(0, i) < 0.99 {
            tapados += 1;
        }
    }
    assert!(
        total > 5_000,
        "só {total} pixels de peça — a fixtura não a mostra"
    );
    assert_eq!(
        tapados, 0,
        "{tapados} de {total} pixels de uma esfera SOZINHA vêm sombreados — um convexo não se \
         pode tapar a si próprio, logo isto é o corpo a ser lido como obstáculo de si mesmo"
    );
}

/// ⭐⭐⭐ **A BORDA DA SOMBRA NUM JADE É MOLE, E A DO OPACO AO LADO CONTINUA DURA.**
///
/// # O report de 2026-09-18 e o experimento que o diagnosticou
///
/// O dono apontou uma linha dura na esfera com `Thin Walled: Solid`. O que a decidiu foi tirar a
/// LÂMINA da cena: **sem ela a bola sai lisa** ⇒ a linha é a borda da SOMBRA que a placa lança, e
/// ela é dura porque a luz é um **ponto**. ⛔ Certo como geometria, errado como produto: num jade a
/// luz entra fora da sombra e espalha-se por baixo da superfície **para dentro** dela.
///
/// # As três metades, e porque nenhuma chega sozinha
///
/// 1. **o jade amacia** — a quebra na banda do terminador cai;
/// 2. **o OPACO não se mexe, ao bit** — sem isto, borrar a visibilidade de toda a gente passaria
///    aqui e apagaria a sombra do app inteiro;
/// 3. **o jade continua a TER sombra** — sem isto, uma «cura» que pusesse `vis = 1` em todo o lado
///    lia a banda lisíssima e passava.
#[test]
fn a_borda_da_sombra_num_jade_e_mole_e_a_do_opaco_continua_dura() {
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do dono");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (onde, luz) = crate::lights::opening_light(&cam);
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        ..ph2d_material::OpenPbr::default()
    };
    let jade = ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        ..base
    };
    let medida = |m: ph2d_material::OpenPbr| {
        let (g, _, px) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao,
            sem_ceu: false,
        });
        let (wu, hu) = (W as usize, H as usize);
        let (right, up, fwd) = cam.basis();
        let da_bola = |i: usize| g.hit[i] && g.point[i][0] > 0.1;
        let lum = |i: usize| {
            let b = i * 4;
            0.2126 * f32::from(px[b])
                + 0.7152 * f32::from(px[b + 1])
                + 0.0722 * f32::from(px[b + 2])
        };
        let ndl = |i: usize| {
            let p = g.point[i];
            let d = [onde[0] - p[0], onde[1] - p[1], onde[2] - p[2]];
            let r = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let n = [0, 1, 2].map(|k| {
                g.normal[i][0] * right[k] + g.normal[i][1] * up[k] + g.normal[i][2] * fwd[k]
            });
            (n[0] * d[0] + n[1] * d[1] + n[2] * d[2]) / r.max(1e-9)
        };
        // A quebra na banda onde a sombra da placa corta a bola, e o CONTRASTE que lá vive.
        let mut saltos: Vec<f32> = Vec::new();
        let (mut claro, mut escuro) = (0.0f32, f32::MAX);
        for y in 0..hu {
            for x in 1..wu - 1 {
                let (a, b, c) = (y * wu + x - 1, y * wu + x, y * wu + x + 1);
                if !da_bola(a) || !da_bola(b) || !da_bola(c) || ndl(b).abs() > 0.15 {
                    continue;
                }
                saltos.push((lum(a) - 2.0 * lum(b) + lum(c)).abs());
                claro = claro.max(lum(b));
                escuro = escuro.min(lum(b));
            }
        }
        assert!(saltos.len() > 2_000, "só {} px na banda", saltos.len());
        saltos.sort_by(f32::total_cmp);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let k = ((saltos.len() as f32 - 1.0) * 0.99).round() as usize;
        (saltos[k], claro - escuro, px)
    };

    let (dura_op, _, bytes_op) = medida(base);
    let (mole_jade, contraste, _) = medida(jade);

    // (1) o jade amacia — medido `9,21` antes e `1,36` depois; a barra sai do vale.
    assert!(
        mole_jade <= 4.0,
        "a quebra na banda do jade lê p99 {mole_jade:.2} contra a barra 4,0 — a borda da sombra \
         voltou a ser dura (o opaco, que não tem espalhamento, lê {dura_op:.2})"
    );
    // (2) ⚠️ o OPACO não pode ter mudado UM BYTE — ele não tem espalhamento, logo não assa canal.
    let sem_sss = ph2d_material::OpenPbr {
        subsurface_weight: 0.0,
        ..base
    };
    let (_, _, controlo) = medida(sem_sss);
    assert_eq!(
        bytes_op, controlo,
        "o material sem subsuperfície mudou de bytes — a borda mole está a alcançar quem não a pediu"
    );
    // (3) e o jade CONTINUA a ter sombra: sem isto, `vis = 1` em todo o lado passaria em (1).
    assert!(
        contraste >= 8.0,
        "o contraste através da banda do jade é {contraste:.1} — a sombra foi apagada em vez de \
         amaciada, e a metade (1) não distingue as duas"
    );
}

/// ⏱️⭐⭐⭐ **OS NÚMEROS DA NOSSA CENA, para um oráculo externo os reproduzir.**
///
/// ⚠️ Eles saem das PORTAS (`Orbit::basis`, `eye_distance`, `lights::opening_light`), nunca
/// re-derivados num script — *um oráculo alimentado com números re-derivados mede outro programa*,
/// que é o erro que esta linha já pagou quatro vezes em dois dias.
#[test]
#[ignore = "sonda: imprime o enquadramento para o oráculo externo"]
fn sonda_os_numeros_da_cena() {
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let (right, up, fwd) = cam.basis();
    let dist = cam.eye_distance();
    let olho = dist.map(|d| [0, 1, 2].map(|i| cam.target[i] + fwd[i] * d));
    let (onde, luz) = crate::lights::opening_light(&cam);
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    println!("CAM_TARGET={:?}", cam.target);
    println!("CAM_HALF_EXTENT={}", cam.half_extent);
    println!("CAM_RIGHT={right:?}");
    println!("CAM_UP={up:?}");
    println!("CAM_FWD={fwd:?}");
    println!("CAM_EYE={olho:?}");
    println!("CAM_EYE_DIST={dist:?}");
    println!("LAMP_POS={onde:?}");
    println!("LAMP_INTENSITY={}", luz.intensity);
    println!("LAMP_COLOR={:?}", luz.color);
    println!("LAMP_RADIANCE={:?}", crate::lights::radiance_at_one(luz));
    println!("GROUND_Y={:?}", ph2d_field_render::lowest_point(&doc, &reg));
    println!("W={W} H={H}");
}

/// ⭐⭐ **Lê um PFM** — e as DUAS convenções dele partem um consumidor em SILÊNCIO.
///
/// ⛔⛔ **A 1.ª redacção desta função tinha os dois defeitos ao mesmo tempo, e a saída dela era
/// lixo que passava por número:** ela lia `f32::from_le_bytes` sobre dados **BIG-endian**, e o
/// laço que parseia o cabeçalho parava aos **três** campos (`PF`, largura, altura) ⇒ a linha da
/// **escala nunca era lida**, o offset dos dados ficava errado, e o «desvio de forma de `96 %`»
/// que esta sonda imprimiu durante uma jornada inteira era isso.
///
/// As convenções, medidas pelo agente E sobre os ficheiros que ele produziu:
/// - **escala `> 0` ⇒ BIG-endian** (`< 0` ⇒ little);
/// - **as linhas vêm de BAIXO para CIMA** — a primeira linha do ficheiro é a de baixo da imagem.
///
/// ⚠️ **Ela RECUSA em voz alta** em vez de devolver lixo: magia errada, cabeçalho curto, tamanho
/// que não fecha, ou valores não-finitos ⇒ `None`. *Um leitor que devolve lixo plausível é pior que
/// um que falha.*
fn le_pfm(caminho: &str) -> Option<(usize, usize, Vec<[f32; 3]>)> {
    let bytes = std::fs::read(caminho).ok()?;
    // Os QUATRO campos do cabeçalho, e não três.
    let mut campos: Vec<String> = Vec::new();
    let mut i = 0usize;
    while campos.len() < 4 {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let ini = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if ini == i {
            return None;
        }
        campos.push(String::from_utf8_lossy(&bytes[ini..i]).to_string());
    }
    i += 1; // o único byte branco a seguir à escala
    if campos[0] != "PF" {
        return None;
    }
    let (w, h): (usize, usize) = (campos[1].parse().ok()?, campos[2].parse().ok()?);
    let escala: f32 = campos[3].parse().ok()?;
    let big = escala > 0.0;
    if i + w * h * 12 != bytes.len() {
        return None;
    }
    let mut px = vec![[0.0f32; 3]; w * h];
    for y in 0..h {
        // ⚠️ A linha `0` do FICHEIRO é a de BAIXO da imagem.
        let linha_no_ficheiro = h - 1 - y;
        for x in 0..w {
            let b = i + (linha_no_ficheiro * w + x) * 12;
            px[y * w + x] = [0, 1, 2].map(|k| {
                let q = [
                    bytes[b + k * 4],
                    bytes[b + k * 4 + 1],
                    bytes[b + k * 4 + 2],
                    bytes[b + k * 4 + 3],
                ];
                if big {
                    f32::from_be_bytes(q)
                } else {
                    f32::from_le_bytes(q)
                }
            });
        }
    }
    px.iter()
        .all(|c| c.iter().all(|v| v.is_finite() && *v >= 0.0))
        .then_some((w, h, px))
}

/// ⏱️⭐⭐⭐ **NÓS CONTRA A VERDADE**, nas condições em que a comparação tem UMA incógnita.
///
/// # ⭐⭐⭐ O desenho, e porque cada cerca está lá
///
/// A 1.ª tentativa comparou as duas imagens com o céu ligado, o chão posto e o especular vivo, e
/// leu `96 %` de desvio de forma — *uma comparação com três incógnitas livres não afirma nada*.
/// Aqui:
///
/// | cerca | porquê |
/// |---|---|
/// | **céu PRETO nos dois** | sobra só a lâmpada, e as duas leis de queda são a MESMA (`1/r²`) |
/// | **sem chão** | o ricochete do chão é luz que o oráculo distribui de outra maneira |
/// | **especular MORTO** | o lóbulo GGX nosso e o do oráculo não são o mesmo, e não é ele que se mede |
/// | **uma escala só** | com o resto casado, um único factor liga os dois lados — *o que sobrar é a LEI* |
///
/// ⚠️ **O oráculo entra pelo NOSSO olhar** (`Look::apply`): comparar um linear com um já tonemapado
/// não afirma nada sobre a forma. E o factor de escala É a `exposure_stops`, logo ajustá-la é
/// ajustar a escala — não há segundo botão.
///
/// ⚠️ A varredura é por **COLUNA** (a borda corre quase na horizontal — §13.4 do `docs/Render3d/10`)
/// e a borda localiza-se pelo canal que a produz (a visibilidade), com o limiar no **meio da faixa
/// medida**: a penumbra da placa nunca chega ao preto, logo `0,5` não é atravessado.
///
/// ⚠️ **As fixturas são acto do agente E** (`$PH2D_VERDADE`), nunca desta janela — INC-R1.
#[test]
#[ignore = "sonda: precisa do oráculo em $PH2D_VERDADE"]
fn sonda_nos_contra_a_verdade() {
    let dir = std::env::var("PH2D_VERDADE").ok();
    let base = ph2d_material::OpenPbr {
        subsurface_color: [0.75, 0.35, 0.35],
        base_color: [0.75, 0.35, 0.35],
        // ⚠️ O especular morre dos DOIS lados — ver a tabela acima.
        specular_weight: 0.0,
        ..ph2d_material::OpenPbr::default()
    };
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    let dump = std::env::var("PH2D_TERM_DUMP").ok();

    for (nome, m) in [
        ("opaco", base),
        (
            "jade",
            ph2d_material::OpenPbr {
                subsurface_weight: 1.0,
                geometry_thin_walled: false,
                ..base
            },
        ),
    ] {
        let (g, sh, nossos) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao: None,
            sem_ceu: true,
        });
        let (w, h) = (W as usize, H as usize);
        if let Some(d) = &dump {
            let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
            for i in 0..w * h {
                ppm.extend_from_slice(&nossos[i * 4..i * 4 + 3]);
            }
            let _ = std::fs::write(format!("{d}/calib_{nome}.ppm"), ppm);
        }
        let na_bola = |i: usize| g.hit[i] && g.point[i][0] > 0.1;
        const COLUNA: usize = 160;
        let nosso: Vec<(usize, f32)> = (0..h)
            .filter(|&y| na_bola(y * w + COLUNA))
            .map(|y| {
                let b = (y * w + COLUNA) * 4;
                (
                    y,
                    0.2126 * f32::from(nossos[b])
                        + 0.7152 * f32::from(nossos[b + 1])
                        + 0.0722 * f32::from(nossos[b + 2]),
                )
            })
            .collect();
        let (lo, hi) = nosso.iter().fold((f32::MAX, f32::MIN), |(a, b), p| {
            let v = sh.at(0, p.0 * w + COLUNA);
            (a.min(v), b.max(v))
        });
        println!(
            "  {nome}: {} px na coluna · vis de {lo:.3} a {hi:.3} · luminância de {:.1} a {:.1}",
            nosso.len(),
            nosso.iter().map(|p| p.1).fold(f32::MAX, f32::min),
            nosso.iter().map(|p| p.1).fold(f32::MIN, f32::max)
        );
        let Some(dir) = &dir else { continue };
        let meio = 0.5 * (lo + hi);
        let Some(centro) = (hi - lo > 0.05)
            .then(|| {
                nosso.windows(2).find_map(|par| {
                    let (a, b) = (par[0].0, par[1].0);
                    ((sh.at(0, a * w + COLUNA) - meio).signum()
                        != (sh.at(0, b * w + COLUNA) - meio).signum())
                    .then_some(b)
                })
            })
            .flatten()
        else {
            println!("    a coluna não atravessa a borda da sombra");
            continue;
        };
        const MEIA: usize = 45;
        let janela: Vec<usize> = nosso
            .iter()
            .map(|p| p.0)
            .filter(|y| y.abs_diff(centro) <= MEIA)
            .collect();
        let nossa_em = |y: usize| nosso.iter().find(|p| p.0 == y).map_or(0.0, |p| p.1);
        // ⭐ Varre as energias que o E produziu e fica com a que melhor casa: a energia do oráculo
        // e a nossa escala são a MESMA incógnita, e resolvê-la duas vezes seria dar-lhe dois botões.
        let mut melhor: Option<(f32, String, f32, Vec<f32>)> = None;
        for e in [5, 10, 20, 40] {
            let caminho = format!("{dir}/ref_{nome}_e{e}.pfm");
            let Some((ow, oh, linear)) = le_pfm(&caminho) else {
                continue;
            };
            assert_eq!((ow, oh), (w, h), "o oráculo {caminho} tem outro tamanho");
            for passo in -96..96 {
                #[allow(clippy::cast_precision_loss)]
                let stops = passo as f32 * 0.125;
                let olhar = ph2d_view_transform::Look {
                    exposure_stops: stops,
                    ..ph2d_view_transform::Look::default()
                };
                let vals: Vec<f32> = janela
                    .iter()
                    .map(|&y| {
                        let d = olhar.apply(linear[y * w + COLUNA]);
                        255.0
                            * (0.2126 * d[0].clamp(0.0, 1.0)
                                + 0.7152 * d[1].clamp(0.0, 1.0)
                                + 0.0722 * d[2].clamp(0.0, 1.0))
                    })
                    .collect();
                let erro: f32 = janela
                    .iter()
                    .zip(&vals)
                    .map(|(&y, v)| (v - nossa_em(y)).abs())
                    .sum();
                if melhor.as_ref().is_none_or(|b| erro < b.0) {
                    melhor = Some((erro, format!("e{e} {stops:+.2}st"), stops, vals));
                }
            }
        }
        // ⭐ A imagem do oráculo pelo NOSSO olhar, para se poder OLHAR lado a lado — a foto é o
        // árbitro, e foi ela que apanhou as duas curas refutadas desta jornada.
        if let (Some(d), Some((_, _, stops, _))) = (&dump, melhor.as_ref()) {
            for e in [5, 10, 20, 40] {
                let Some((_, _, linear)) = le_pfm(&format!("{dir}/ref_{nome}_e{e}.pfm")) else {
                    continue;
                };
                let olhar = ph2d_view_transform::Look {
                    exposure_stops: *stops,
                    ..ph2d_view_transform::Look::default()
                };
                let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
                for cru in linear.iter().take(w * h) {
                    for canal in olhar.apply(*cru) {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        ppm.push((canal.clamp(0.0, 1.0) * 255.0 + 0.5) as u8);
                    }
                }
                let _ = std::fs::write(format!("{d}/verdade_{nome}_e{e}.ppm"), ppm);
            }
        }
        let Some((_, como, _, verdade)) = melhor else {
            println!("    sem oráculo para `{nome}` em {dir}");
            continue;
        };
        let cru_nosso: Vec<f32> = janela.iter().map(|&y| nossa_em(y)).collect();
        let largura = |v: &[f32]| -> f32 {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let onde = |f: f32| {
                let alvo = lo + f * (hi - lo);
                v.iter()
                    .enumerate()
                    .min_by(|p, q| (p.1 - alvo).abs().total_cmp(&(q.1 - alvo).abs()))
                    .map_or(0, |p| p.0)
            };
            #[allow(clippy::cast_precision_loss)]
            let d = (onde(0.9) as f32 - onde(0.1) as f32).abs();
            d
        };
        let normaliza = |v: &[f32]| -> Vec<f32> {
            let (lo, hi) = v
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
            let d = (hi - lo).max(1e-6);
            v.iter().map(|x| (x - lo) / d).collect()
        };
        let (a, b) = (normaliza(&cru_nosso), normaliza(&verdade));
        let forma: f32 = a
            .iter()
            .zip(&b)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        let bytes: f32 = cru_nosso
            .iter()
            .zip(&verdade)
            .map(|(x, y)| (x - y).abs())
            .fold(0.0, f32::max);
        println!(
            "    borda em y={centro} · casou com {como} · largura 10–90%: NÓS {:.0} px · VERDADE \
             {:.0} px · desvio da FORMA {:.1} % · pior byte {bytes:.1}",
            largura(&cru_nosso),
            largura(&verdade),
            100.0 * forma
        );
    }
}

/// ⏱️⭐⭐⭐ **A VARREDURA DA COR: nós contra a verdade, em três profundidades.**
///
/// O primeiro veredito mediu UM ponto (`Subsurface Radius = 1,0`) e leu os dois a mover a cor em
/// sentidos opostos. ⛔ *Um desvio medido num ponto é um ponto, não uma lei* ⇒ esta sonda varre
/// três profundidades, e em duas famílias:
///
/// - **`r`**: o raio por canal do produto (`1 : 0,5 : 0,25` escalado) — o que o artista tem;
/// - **`g`**: o mesmo raio nos **três** canais — o controlo que separa *«a cor muda com a
///   PROFUNDIDADE»* de *«a cor muda porque cada canal viaja o seu»*.
///
/// ⚠️ **Os dois lados são lidos no MESMO espaço** (o nosso olhar, com a exposição ajustada por
/// célula): `R/B` em linear e em ecrã não é o mesmo número, e comparar um com o outro seria a
/// quinta régua mal calibrada desta jornada.
///
/// ⚠️ **Piso de ruído do oráculo, medido pelo agente E: `~1e-4` absoluto** (`≈0,2 %` da média) — o
/// Cycles em CPU **não é bit-reprodutível entre invocações**. Nada abaixo disso é lei.
/// ⭐⭐⭐ **A RÉGUA DA COR — `R/B` da região iluminada, em BYTES do NOSSO olhar.**
///
/// ⛔⛔ **Ela NÃO se mede em linear, e a diferença é grande o suficiente para inverter um
/// veredito.** Uma esfera lambertiana de `base_color = (0,75 · 0,35 · 0,35)` sob luz branca devolve
/// `R/B = 0,75/0,35 = 2,143` em LINEAR — foi exactamente o que a janela da Unreal mediu, a quatro
/// dígitos, no controlo opaco dela. O nosso controlo lê **`1,33`** porque a transformação de vista
/// comprime a razão antes de ela chegar ao ecrã. *Comparar um com o outro seria medir a curva de
/// exibição e chamar-lhe material.*
///
/// ⚠️ A máscara é `soma dos três bytes > 30` — a região **iluminada**, e a mesma dos dois lados.
fn razao_rb(px: &[u8]) -> (f32, usize) {
    let (mut r, mut b, mut n) = (0.0f64, 0.0f64, 0usize);
    for i in 0..(W as usize) * (H as usize) {
        let q = i * 4;
        let soma = u32::from(px[q]) + u32::from(px[q + 1]) + u32::from(px[q + 2]);
        if soma > 30 {
            r += f64::from(px[q]);
            b += f64::from(px[q + 2]);
            n += 1;
        }
    }
    #[allow(clippy::cast_possible_truncation)]
    ((r / b.max(1e-9)) as f32, n)
}

/// Um quadro linear pelo NOSSO olhar, com a exposição dada.
fn em_bytes(linear: &[[f32; 3]], stops: f32) -> Vec<u8> {
    let (w, h) = (W as usize, H as usize);
    let olhar = ph2d_view_transform::Look {
        exposure_stops: stops,
        ..ph2d_view_transform::Look::default()
    };
    let mut bytes = vec![0u8; w * h * 4];
    for (i, cru) in linear.iter().take(w * h).enumerate() {
        for (k, canal) in olhar.apply(*cru).into_iter().enumerate() {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                bytes[i * 4 + k] = (canal.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
    bytes
}

/// ⭐⭐ **Põe um quadro de oráculo no nosso olhar com a MESMA POPULAÇÃO iluminada que o nosso.**
///
/// ⛔ Sem isto a máscara mede **regiões diferentes** dos dois lados, e o `R/B` de cada uma é de
/// outra coisa — foi esse o defeito que fez a §14.3 escrever *«a cor move-se no sentido oposto»*
/// sobre um ponto só, e que a §14.4 corrigiu. A exposição é a **única** incógnita livre da
/// comparação (o céu está apagado, e as duas leis de queda são `1/r²`), logo ajustá-la é honesto e
/// ajustar qualquer outra coisa não seria.
///
/// ⚠️ Devolve também os `stops` escolhidos: *um ajuste que não se imprime é um grau de liberdade
/// escondido*.
fn casa_a_populacao(linear: &[[f32; 3]], alvo_n: usize) -> (Vec<u8>, f32) {
    let mut melhor = (usize::MAX, 0.0f32);
    for passo in -96..96 {
        #[allow(clippy::cast_precision_loss)]
        let stops = passo as f32 * 0.125;
        let (_, n) = razao_rb(&em_bytes(linear, stops));
        if n.abs_diff(alvo_n) < melhor.0 {
            melhor = (n.abs_diff(alvo_n), stops);
        }
    }
    (em_bytes(linear, melhor.1), melhor.1)
}

/// ⭐ A UNREAL como TERCEIRO contendor — ela mede-se com a régua **deste** módulo, de propósito.
///
/// ⚠️ É um módulo-filho por `#[path]` e não uma crate nem um irmão de `lib.rs`: assim ele vê o
/// `le_pfm`, o `quadro`, a `razao_rb` e a `casa_a_populacao` **sem que nada aqui abra
/// visibilidade** — e, sobretudo, *sem uma segunda cópia da régua*. Três colunas medidas por três
/// funções diferentes não são uma comparação.
#[path = "unreal_contendor_tests.rs"]
mod unreal_contendor;

/// ⭐ **A LEI DA COR** — ordem do dono depois do veredito a quatro colunas: *«atacamos agora a cor»*.
///
/// Mesmo desenho do irmão: módulo-filho por `#[path]`, para usar a régua deste módulo sem abrir
/// visibilidade e sem uma segunda cópia dela.
#[path = "cor_da_profundidade_tests.rs"]
mod cor_da_profundidade;

#[test]
#[ignore = "sonda: precisa do lote 2 do oráculo em $PH2D_VERDADE2"]
fn sonda_a_varredura_da_cor() {
    let Ok(dir) = std::env::var("PH2D_VERDADE2") else {
        println!("sem $PH2D_VERDADE2 — saltado");
        return;
    };
    let mut cam = Orbit::default();
    cam.half_extent *= 0.42;
    cam.target = [0.55, 0.0, 0.0];
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena");
    let (onde, luz) = crate::lights::opening_light(&cam);
    println!("  raio · família ·   NOSSO R/B ·  VERDADE R/B · o que isso quer dizer");
    for (tag, raio, escala) in [
        ("r010", 0.1f32, [1.0f32, 0.5, 0.25]),
        ("r030", 0.3, [1.0, 0.5, 0.25]),
        ("r100", 1.0, [1.0, 0.5, 0.25]),
        ("g010", 0.1, [1.0, 1.0, 1.0]),
        ("g030", 0.3, [1.0, 1.0, 1.0]),
        ("g100", 1.0, [1.0, 1.0, 1.0]),
    ] {
        let m = ph2d_material::OpenPbr {
            subsurface_weight: 1.0,
            geometry_thin_walled: false,
            subsurface_color: [0.75, 0.35, 0.35],
            base_color: [0.75, 0.35, 0.35],
            specular_weight: 0.0,
            subsurface_radius: raio,
            subsurface_radius_scale: escala,
            ..ph2d_material::OpenPbr::default()
        };
        let (_, _, nossos) = quadro(&Quadro {
            doc: &doc,
            m,
            cam: &cam,
            onde,
            luz,
            com_sombra: true,
            chao: None,
            sem_ceu: true,
        });
        let (nosso_rb, nosso_n) = razao_rb(&nossos);
        let Some((_, _, linear)) = le_pfm(&format!("{dir}/ref_jade_{tag}_e5.pfm")) else {
            println!("  {tag}: sem oráculo");
            continue;
        };
        let (bytes, _stops) = casa_a_populacao(&linear, nosso_n);
        let (verdade_rb, verdade_n) = razao_rb(&bytes);
        println!(
            "  {:.2} · {} · {nosso_rb:>10.2} · {verdade_rb:>11.2} · {} ({nosso_n} vs {verdade_n} px)",
            raio,
            if escala[1] < 1.0 {
                "por canal"
            } else {
                "IGUAIS  "
            },
            if (nosso_rb - verdade_rb).abs() < 0.15 {
                "concordam"
            } else if (nosso_rb - 1.0).signum() == (verdade_rb - 1.0).signum() {
                "mesmo sentido, magnitude diferente"
            } else {
                "SENTIDOS OPOSTOS"
            }
        );
    }
}
