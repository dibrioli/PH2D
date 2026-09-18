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

// ─────────────────────────────────────────────────────────────────────────────────────────────
//  O GATE — três metades, e cada uma é um defeito MEDIDO
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// O quadro do dono em CPU: devolve `(gbuffer, sombras, bytes)`.
struct Quadro<'a> {
    doc: &'a ph2d_field::FieldDoc,
    m: ph2d_material::OpenPbr,
    cam: &'a Orbit,
    onde: [f32; 3],
    luz: ph2d_field_ecs::FieldLight,
    com_sombra: bool,
    chao: Option<ph2d_field_render::Ground>,
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
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
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
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        ph2d_view_transform::Look::default(),
        [0, 0, 0, 0],
    );
    (g, sh, px)
}

/// ⭐⭐⭐ **A SOMBRA DE UM VIZINHO NÃO É TRUNCADA NO TERMINADOR** — o report de 2026-09-18.
///
/// # O defeito, o mecanismo e as barras
///
/// Na cena `=33` a LÂMINA projecta sombra sobre a ESFERA. O passe de sombra escrevia `vis = 1,0`
/// para todo ponto com `N·L ≤ 0` (*«de costas para a luz: sem raio»*), o que é inofensivo enquanto
/// todo consumidor multiplicar por `max(N·L, 0)` — e **falso** desde que a subsuperfície MACIÇA
/// existe, que é o primeiro desta casa a ler luz do lado escuro. ⇒ a sombra acabava a meio, num
/// degrau de **UM pixel**.
///
/// | medido na banda `\|N·L\| <= 0,15` | 2.ª diferença p99 |
/// |---|---|
/// | **o defeito** (o `continue` que escrevia `1,0`) | **9,21** |
/// | marchar o raio como os outros (refutado — ver a folha) | 2,14 |
/// | **hoje** (sair do próprio corpo + recontar o `t`) | **3,71** |
/// | a MESMA cena sem lâmpada nenhuma (o controlo liso) | **1,00** |
///
/// A barra de `6,0` sai do **vale entre `3,71` e `9,21`**, e ⚠️ **o controlo é metade do gate**:
/// sem ele, uma mutação que apagasse a sombra toda lia `1,00` e passava.
#[test]
fn a_sombra_de_um_vizinho_nao_e_truncada_no_terminador() {
    const BARRA: f32 = 6.0;
    const BARRA_DO_CONTROLO: f32 = 1.5;
    let doc = crate::smoke::scenes::edge::cena_33().expect("a cena do dono");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (w, h) = (W, H);
    let (onde, luz) = crate::lights::opening_light(&cam);
    let chao = ph2d_field_render::lowest_point(&doc, &reg)
        .map(|height| ph2d_field_render::Ground { height });
    let (right, up, fwd) = cam.basis();

    let p99 = |com_sombra: bool| {
        let (g, _, px) = quadro(&Quadro {
            doc: &doc,
            m: macico(),
            cam: &cam,
            onde,
            luz,
            com_sombra,
            chao,
        });
        let (wu, hu) = (w as usize, h as usize);
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
        assert!(
            saltos.len() > 2_000,
            "só {} pixels na banda do terminador — a fixtura não contém o fenómeno",
            saltos.len()
        );
        saltos.sort_by(f32::total_cmp);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let k = ((saltos.len() as f32 - 1.0) * 0.99).round() as usize;
        saltos[k]
    };

    let controlo = p99(false);
    assert!(
        controlo <= BARRA_DO_CONTROLO,
        "o CONTROLO (a mesma cena sem lâmpada) lê {controlo:.2} — a régua não tem uma referência \
         lisa, e sem ela a barra não afirma nada"
    );
    let com = p99(true);
    assert!(
        com <= BARRA,
        "a quebra na banda do terminador lê p99 {com:.2} contra a barra {BARRA:.1} (o controlo \
         liso lê {controlo:.2}) — a sombra do vizinho voltou a ser truncada no terminador"
    );
}

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
