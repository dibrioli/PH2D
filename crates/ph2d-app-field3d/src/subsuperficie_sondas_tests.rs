//! ⏱️ **AS SONDAS DO TERMINADOR** — as réguas que DECOMPÕEM o report, e que não afirmam nada.
//!
//! Irmão de ASSUNTO do [`super`], cortado dele em 2026-09-18 pelo tecto de LOC. ⛔ *Split por
//! responsabilidade, nunca uma entrada no `FILE_OVERAGE_OK`* (`CLAUDE.md` §2).
//!
//! # ⚠️ A partição é «o que DIAGNOSTICA» contra «o que AFIRMA»
//!
//! As quatro daqui são todas `#[ignore]`: elas imprimem tabelas e fotografam quadros, e **nenhuma
//! tem uma asserção sobre o produto**. O vizinho guarda o contrário — a moldura (`quadro`) e os
//! três gates que reprovam. *Um ficheiro em que uma sonda e uma lei se leem iguais é onde uma lei
//! passa a `#[ignore]` sem ninguém dar por isso.*
//!
//! # ⚠️ E a lei que as quatro pagaram
//!
//! A 1.ª redacção da primeira montava uma bola sozinha na origem e binava a luminância por `N·L`.
//! Ela leu o perfil a atravessar `N·L = 0` **liso** ⇒ *a fixtura não continha o fenómeno*, que é a
//! lei que esta casa já pagou cinco vezes. ⭐ O que elas fazem hoje é reconstruir **o quadro do
//! dono** e imprimir os canais **lado a lado**, porque *o canal cujo salto COINCIDE com o da
//! luminância é a causa, e os outros ficam ilibados com número*.

use ph2d_field_render::{Orbit, Surfaces};

// A moldura e o material vivem no PAI: uma sonda mede o que a lei produz, e não uma segunda cópia.
use super::{Quadro, macico, quadro};

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
