//! ⭐⭐⭐ **AS SONDAS do modo Render** — os números que decidiram as waves, não as leis.
//!
//! # Porque é um ficheiro irmão
//!
//! O [`super`] afirma **LEIS** (uma lâmpada de intensidade `1` chega como `π`; o céu é mais claro
//! em cima; a irradiância é a média cosseno da radiância). Isto **MEDE** — quanto custa o modo, o
//! que a vista `Neutral` faria ao matcap, o que o material por objecto custa ao sombreamento,
//! quanto do material este céu deixa passar. São dois assuntos com ritmos diferentes: uma lei muda
//! quando a física muda, uma sonda quando alguém quer um número.
//!
//! ⛔ O corte foi forçado pelo tecto de `700` linhas — *corte por responsabilidade, nunca uma
//! entrada no `FILE_OVERAGE_OK`* (`CLAUDE.md` §5.0).

use super::*;

/// ⏱️ **SONDA (`--ignored`): o que o modo RENDER custa, e que luz ele DEVOLVE** — com o rig, o céu,
/// o material e o olhar **do produto**.
///
/// ⛔⛔ **Ela mudou de crate em 2026-09-13, e a mudança É a correcção.** Ela nasceu na
/// `ph2d-field-render`, que **não alcança** este ficheiro — e por isso escrevia a luz à mão: uma
/// lâmpada com a direcção em literal e um céu **CONSTANTE**, onde o produto tem um [`StudioSky`]
/// que é um GRADIENTE. ⇒ a tabela de luz do `docs/Render3d/05` §6 media um programa que ninguém
/// corre, e errava no sentido optimista (`7,1 %` da peça cortada a `0` stops, contra os `10,9 %`
/// reais). *Uma segunda cópia escrita à mão do valor que uma porta produz.*
///
/// ⚠️ **Não é um gate — não há barra aqui.** É a medição que responde *«isto ainda é interactivo?»*
/// e *«a peça sai preta ou estourada?»* antes de o dono olhar para ela. Corra-a com a máquina calma
/// (`CLAUDE.md` §5.0: nenhum relógio desta workstation vale acima de `load ~5`).
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_render_mode_costs_and_paints() {
    use ph2d_field_render::{Lighting, Matcap, Orbit, shade_render, trace};
    use ph2d_view_transform::{Look, ViewTransform};
    use std::time::Instant;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let t = Instant::now();
    let g = trace(&doc, &reg, &cam, w, h);
    let trace_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⭐ **A luz é a do PRODUTO, pelas mesmas portas que o `smoke_draw` chama.**
    let lamps = lamps(&ph2d_light::LightRig::default());
    let so = [ph2d_material::OpenPbr::default().prepare()];
    // ⚠️ **Um material só, e sem `owners`** — esta sonda mede a LUZ, e a lei do material por
    // objecto tem gates próprios (`materials_tests`).
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    let light = Lighting {
        lamps: &lamps,
        points: &[],
        sky: &StudioSky,
        shadows: None,
    };
    let t = Instant::now();
    let render_px = shade_render(
        &g,
        &cam,
        &surface,
        &light,
        &ph2d_field_render::Presentation::of(Look::default()),
        BG,
    );
    let render_ms = t.elapsed().as_secs_f64() * 1e3;

    let texels = vec![0.5_f32; 2 * 2 * 3];
    let m = Matcap {
        side: 2,
        rgb_linear: &texels,
    };
    let t = Instant::now();
    let _ = ph2d_field_render::shade(&g, &m, BG);
    let matcap_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⚠️ **Saturado é os TRÊS canais em `255`**, e não só o verde: um canal no tecto ainda tem cor,
    // e o que o olho lê como «branco chapado» é a peça a perder a forma nos três.
    let stats = |px: &[u8]| -> (f64, u32, u32, usize) {
        let (mut soma, mut verde, mut branco, mut n) = (0.0_f64, 0_u32, 0_u32, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            soma += f64::from(p[1]);
            verde += u32::from(p[1] == 255);
            branco += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
        }
        (soma / n as f64, verde, branco, n)
    };
    let (_, _, branco0, pixels) = stats(&render_px);
    println!("esfera {w}x{h} · {pixels} pixels de peça · traçado {trace_ms:.1} ms");
    println!(
        "sombrear: matcap (2x2 texels) {matcap_ms:.2} ms · render {render_ms:.2} ms · \
         branco chapado {branco0}"
    );
    for stops in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        let de = |view| {
            let look = Look {
                exposure_stops: stops,
                view,
            };
            stats(&shade_render(
                &g,
                &cam,
                &surface,
                &light,
                &ph2d_field_render::Presentation::of(look),
                BG,
            ))
        };
        let (s_media, s_verde, s_branco, _) = de(ViewTransform::Standard);
        let (n_media, n_verde, n_branco, _) = de(ViewTransform::Neutral);
        println!(
            "  {stops:+.0} stop · Standard media {s_media:6.1} verde=255 {s_verde:6} branco {s_branco:6} \
             · Neutral media {n_media:6.1} verde=255 {n_verde:6} branco {n_branco:6}"
        );
    }
}

/// ⏱️ **SONDA — o que a vista `Neutral` faria ao MATCAP**, que é a metade da pergunta do dono que
/// ninguém tinha medido (`docs/Render3d/05` §8).
///
/// # ⚠️ A pergunta, e porque ela NÃO é só sobre o modo Render
///
/// A peça sai com `8,4 %` em branco chapado no olhar de omissão, e as duas saídas escritas são *a
/// vista passa a `Neutral`* ou *a exposição desce um stop*. ⛔ **Mas o olhar é da CENA, e o matcap
/// também passa por ele** (o doc do `shade_with` escreve-o, e é a *Color Management* do Blender):
/// trocar a omissão muda **o que o modelador sempre mostrou**, não só o modo novo.
///
/// ⚠️ E a razão de a omissão ser `Standard` está escrita no `ph2d-view-transform`: *«com exposição
/// `0` e luz dentro de `0..=1` ela devolve a entrada»* — isto é, **ela foi escolhida para o matcap**,
/// que é uma fotografia em `0..=1`. A `Neutral` dobra tudo acima do joelho (`0,76`), logo o preço da
/// troca mora exactamente nos texels claros.
///
/// ⇒ esta sonda mede esse preço no **asset da casa**, texel a texel.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_neutral_would_do_to_the_matcap() {
    use ph2d_view_transform::{Look, ViewTransform};

    let side = ph2d_mesh_render::matcap::MATCAPS[0].side as usize;
    let bytes = ph2d_mesh_render::matcap::decode(0);
    let padrao = Look::default();
    let neutra = Look {
        exposure_stops: 0.0,
        view: ViewTransform::Neutral,
    };
    let escura = Look {
        exposure_stops: -1.0,
        view: ViewTransform::Standard,
    };

    let mut deltas: std::collections::BTreeMap<i32, Vec<i32>> = std::collections::BTreeMap::new();
    let mut n = 0usize;
    let mut acima_do_joelho = 0usize;
    let mut cortados = 0usize;
    let (mut mudados_neutra, mut pior_neutra) = (0usize, 0i32);
    let (mut mudados_escura, mut pior_escura) = (0usize, 0i32);
    for texel in bytes.as_chunks::<8>().0.iter() {
        let mut lin = [0.0f32; 3];
        for (c, v) in lin.iter_mut().enumerate() {
            *v =
                half::f16::from_bits(u16::from_le_bytes([texel[c * 2], texel[c * 2 + 1]])).to_f32();
        }
        n += 1;
        if lin.iter().any(|c| *c > 0.76) {
            acima_do_joelho += 1;
        }
        if lin.iter().any(|c| *c >= 1.0) {
            cortados += 1;
        }
        let byte = |l: Look| l.apply(lin).map(ph2d_color::srgb::linear_to_srgb_byte);
        let base = byte(padrao);
        for (outra, mudados, pior) in [
            (neutra, &mut mudados_neutra, &mut pior_neutra),
            (escura, &mut mudados_escura, &mut pior_escura),
        ] {
            let b = byte(outra);
            let d = (0..3)
                .map(|c| i32::from(b[c]) - i32::from(base[c]))
                .max_by_key(|d| d.abs())
                .unwrap_or(0);
            if d != 0 {
                *mudados += 1;
            }
            if d.abs() > pior.abs() {
                *pior = d;
            }
            deltas
                .entry(outra.view as u8 as i32 * 1000 + i32::from(outra.exposure_stops as i8))
                .or_default()
                .push(d.abs());
        }
    }
    let pct = |k: usize| k as f64 / n as f64 * 100.0;
    println!("matcap {side}² = {n} texels");
    println!(
        "  acima do joelho da Neutral (0,76): {:5.1} %",
        pct(acima_do_joelho)
    );
    println!(
        "  já cortados pela Standard (≥ 1,0): {:5.1} %",
        pct(cortados)
    );
    println!(
        "  Neutral   : {:5.1} % dos texels mudam · pior {pior_neutra:+} bytes",
        pct(mudados_neutra)
    );
    println!(
        "  −1 stop   : {:5.1} % dos texels mudam · pior {pior_escura:+} bytes",
        pct(mudados_escura)
    );
    // ⚠️ **O PIOR não diz se se vê** — um extremo num punhado de texels é ruído; o que decide é a
    // MEDIANA e o p95 do desvio, que é o que o olho percorre numa face grande.
    for (chave, mut v) in deltas {
        v.sort_unstable();
        let q = |f: f64| v[((v.len() - 1) as f64 * f) as usize];
        println!(
            "  {:9}: |Δ| p50 {:3} · p95 {:3} · p99 {:3} bytes",
            if chave >= 1000 {
                "Neutral"
            } else {
                "−1 stop"
            },
            q(0.5),
            q(0.95),
            q(0.99)
        );
    }
}

/// ⏱️ **SONDA — o que o MATERIAL POR OBJECTO custa ao sombreamento**, por número de folhas.
///
/// # ⚠️ A pergunta que eu mexi e não tinha reconferido (`CLAUDE.md` §0.0)
///
/// A [`measure_what_the_render_mode_costs_and_paints`] mede o modo Render com **um** material
/// (`owners: None`) — o caminho de omissão. Mas desde 13/09 o sombreamento resolve **um dono por
/// pixel** quando a peça tem mais de uma folha, e isso é trabalho novo dentro do laço mais quente do
/// quadro. *Quem acrescenta um custo ao caminho quente mede-o no caminho quente, e não numa sonda ao
/// lado.*
///
/// ⚠️ A sonda de 13/09 mediu a resolução **isolada** (`1,6 ms` a 16 folhas sobre `26 100` px). Isto é
/// outra coisa: o `shade_render` inteiro, com e sem donos, sobre a mesma peça.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_material_per_object_costs_the_shading() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    use std::time::Instant;

    const BG: [u8; 4] = [0, 0, 0, 0];
    let (w, h) = (640u32, 360u32);
    let cam = Orbit::default();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let lamps = lamps(&ph2d_light::LightRig::default());
    let olhar = crate::shading::OPENING_LOOK;
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("load: {}", carga.split_whitespace().next().unwrap_or("?"));
    println!("folhas ·  peça px ·  sem donos ·  com donos ·  delta ·  % de um quadro");
    for k in [2usize, 4, 8, 16] {
        let lado = (k as f32).sqrt().ceil() as usize;
        let passo = 0.9 / lado as f32;
        let mut nodes: Vec<ph2d_field::Node> = (0..k)
            .map(|i| ph2d_field::Node {
                xform: ph2d_field::Xform::at(
                    ((i % lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    ((i / lado) as f32 - (lado - 1) as f32 * 0.5) * passo,
                    0.0,
                ),
                kind: ph2d_field::NodeKind::Leaf(ph2d_field::Primitive::Sphere {
                    radius: passo * 0.45,
                }),
                mods: Vec::new(),
                verb: None,
            })
            .collect();
        nodes.push(ph2d_field::Node {
            xform: ph2d_field::Xform::IDENTITY,
            kind: ph2d_field::NodeKind::Combine {
                op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                children: (0..k).map(|i| ph2d_field::NodeId(i as u32)).collect(),
            },
            mods: Vec::new(),
            verb: None,
        });
        let doc = ph2d_field::FieldDoc::new(nodes, ph2d_field::NodeId(k as u32)).expect("a grelha");
        let g = trace(&doc, &reg, &cam, w, h);
        let px = g.hit.iter().filter(|b| **b).count();

        // As folhas postas no mundo, como a `materials::Table` as constrói.
        let postas: Vec<ph2d_field::FieldDoc> = (0..k)
            .map(|i| {
                ph2d_field::FieldDoc::new(vec![doc.nodes()[i].clone()], ph2d_field::NodeId(0))
                    .expect("a folha")
            })
            .collect();
        let owners = ph2d_field_eval::owners::Owners::new(
            &postas,
            &reg,
            ph2d_field_render::hit_tolerance(cam.half_extent, w.min(h) as f32),
        );
        let so: Vec<ph2d_material::Surface> = (0..k)
            .map(|i| {
                crate::materials::surface_of(ph2d_field_ecs::FieldMaterial {
                    base_color: [i as f32 / k as f32, 0.5, 0.8],
                    ..ph2d_field_ecs::FieldMaterial::default()
                })
            })
            .collect();
        let light = Lighting {
            lamps: &lamps,
            points: &[],
            sky: &StudioSky,
            shadows: None,
        };
        // ⚠️ A mediana de 5, com um aquecimento antes — e o MÍNIMO ao lado, porque esta máquina não
        // desce de `load ~7` (a nota do `project-memory`).
        let med = |owners: Option<&ph2d_field_eval::owners::Owners>| -> (f64, f64) {
            let s = ph2d_field_render::Surfaces { all: &so, owners };
            let _ = shade_render(
                &g,
                &cam,
                &s,
                &light,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
            );
            let mut v: Vec<f64> = (0..5)
                .map(|_| {
                    let t = Instant::now();
                    let _ = shade_render(
                        &g,
                        &cam,
                        &s,
                        &light,
                        &ph2d_field_render::Presentation::of(olhar),
                        BG,
                    );
                    t.elapsed().as_secs_f64() * 1e3
                })
                .collect();
            v.sort_by(f64::total_cmp);
            (v[0], v[v.len() / 2])
        };
        let (sem, _) = med(None);
        let (com, _) = med(Some(&owners));
        println!(
            "{k:6} · {px:8} · {sem:9.3} · {com:9.3} · {:6.3} · {:6.1} %",
            com - sem,
            com / 16.7 * 100.0
        );
    }
}

/// ⭐⭐⭐ **A LEI DO LÓBULO** vive no irmão, por assunto e pelo tecto de LOC — ver
/// [`field3d_render_light_lobe_tests`](self::lobe).
#[path = "render_light_lobe_tests.rs"]
mod lobe;

/// ⏱️ **SONDA — quanto do MATERIAL este céu deixa ver** (a `W3` do [plano](../../../docs/Render3d/03_o_plano.md)).
///
/// # A pergunta
///
/// O plano diz que a `W3` é *«o que faz o metal existir»*. Antes de a construir, a premissa mede-se:
/// **sob o céu de hoje — uma rampa linear de duas cores — o que é que o artista vê quando mexe no
/// `Metalness` e no `Roughness`?**
///
/// Três réguas, todas em BYTES sobre os pixels da peça (logo independentes do relógio e da carga):
///
/// 1. **o deslocamento** — `|Δ|` médio e máximo contra o material de referência;
/// 2. **a repartição** — quanto da luz é a LÂMPADA e quanto é o CÉU (o céu trocado por preto, e a
///    lâmpada apagada);
/// 3. ⭐ **a ESTRUTURA** — o `|∇²|` médio do verde sobre os pixels de miolo. *Um espelho sob um
///    estúdio a sério tem arestas; sob uma rampa não tem nada para reflectir, e a rampa filtrada
///    continua a ser uma rampa.*
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_how_much_of_the_material_this_sky_lets_through() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    /// Um céu APAGADO — a metade da repartição que isola a lâmpada.
    struct Black;
    impl ph2d_material::Environment for Black {
        fn radiance(&self, _d: [f32; 3], _a: f32) -> [f32; 3] {
            [0.0; 3]
        }
        fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
            [0.0; 3]
        }
    }

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let acesas = lamps(&ph2d_light::LightRig::default());
    let apagadas: Vec<ph2d_field_render::Lamp> = Vec::new();
    // ⭐ **A sonda da PREMISSA mede os DOIS céus**, senão ela deixa de medir a premissa no dia em que
    // o céu muda — que é exactamente o que aconteceu a 14/09.
    let antes = crate::studio::Studio::bare_ramp();
    let depois = crate::studio::Studio::of_the_product();

    let pinta = |m: ph2d_material::OpenPbr,
                 lamps: &[ph2d_field_render::Lamp],
                 sky: &(dyn ph2d_material::Environment + Sync)| {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let light = Lighting {
            lamps,
            points: &[],
            sky,
            shadows: None,
        };
        shade_render(
            &g,
            &cam,
            &surface,
            &light,
            &ph2d_field_render::Presentation::of(crate::shading::OPENING_LOOK),
            BG,
        )
    };

    // A média do verde sobre a peça.
    let media = |px: &[u8]| -> f64 {
        let (mut s, mut n) = (0.0_f64, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if g.hit[i] {
                s += f64::from(p[1]);
                n += 1;
            }
        }
        s / n as f64
    };
    // `|Δ|` médio e máximo sobre os TRÊS canais, só onde há peça.
    let delta = |a: &[u8], b: &[u8]| -> (f64, u32) {
        let (mut s, mut mx, mut n) = (0.0_f64, 0_u32, 0_usize);
        let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
        for i in 0..ca.len() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            for c in 0..3 {
                let d = u32::from(ca[i][c].abs_diff(cb[i][c]));
                s += f64::from(d);
                mx = mx.max(d);
            }
        }
        (s / (3 * n) as f64, mx)
    };
    // ⭐ A ESTRUTURA: `|∇²|` do verde, só onde os quatro vizinhos também são peça.
    let estrutura = |px: &[u8]| -> f64 {
        let c = px.as_chunks::<4>().0;
        let (mut s, mut n) = (0.0_f64, 0_usize);
        let (w, h) = (w as usize, h as usize);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                let viz = [i - 1, i + 1, i - w, i + w];
                if !g.hit[i] || viz.iter().any(|&j| !g.hit[j]) {
                    continue;
                }
                let lap =
                    viz.iter().map(|&j| f64::from(c[j][1])).sum::<f64>() - 4.0 * f64::from(c[i][1]);
                s += lap.abs();
                n += 1;
            }
        }
        s / n as f64
    };

    let prata = |rough: f32, metal: f32| ph2d_material::OpenPbr {
        base_metalness: metal,
        base_color: [0.95, 0.95, 0.95],
        specular_roughness: rough,
        ..ph2d_material::OpenPbr::default()
    };

    for (nome, ceu) in [
        (
            "A RAMPA NUA (o céu de antes de 14/09)",
            &antes as &(dyn ph2d_material::Environment + Sync),
        ),
        ("O ESTÚDIO (rampa + caixa de luz)", &depois),
    ] {
        println!("\n════ {nome} ════");
        println!(
            "esfera {w}x{h} · olhar do PRODUTO ({:?}) + a lâmpada de omissão\n",
            crate::shading::OPENING_LOOK.view
        );
        println!("METAL (metalness 1, base 0,95) — varrendo a rugosidade contra a referência 0,30");
        println!("rugosidade ·  média · |Δ| médio · |Δ| máx · estrutura |∇²|");
        let ref_metal = pinta(prata(0.30, 1.0), &acesas, ceu);
        for r in [0.05_f32, 0.10, 0.20, 0.30, 0.50, 0.70, 1.00] {
            let px = pinta(prata(r, 1.0), &acesas, ceu);
            let (dm, dx) = delta(&px, &ref_metal);
            println!(
                "     {r:.2} · {:6.1} · {dm:9.2} · {dx:6} · {:9.3}",
                media(&px),
                estrutura(&px)
            );
        }

        println!("\nDIELÉCTRICO (metalness 0, base 0,95) — o mesmo varrimento, referência 0,30");
        let ref_diel = pinta(prata(0.30, 0.0), &acesas, ceu);
        for r in [0.05_f32, 0.30, 1.00] {
            let px = pinta(prata(r, 0.0), &acesas, ceu);
            let (dm, dx) = delta(&px, &ref_diel);
            println!(
                "     {r:.2} · {:6.1} · {dm:9.2} · {dx:6} · {:9.3}",
                media(&px),
                estrutura(&px)
            );
        }

        println!("\nMETALNESS 0 → 1, à mesma rugosidade");
        for r in [0.05_f32, 0.30, 1.00] {
            let (dm, dx) = delta(
                &pinta(prata(r, 0.0), &acesas, ceu),
                &pinta(prata(r, 1.0), &acesas, ceu),
            );
            println!("     rugosidade {r:.2} · |Δ| médio {dm:6.2} · máx {dx}");
        }

        println!(
            "\nO TAMANHO DO REALCE — quantos pixels se movem mais de 8 bytes ao ir de 0,30 a 0,05"
        );
        for (nome, metal) in [("dieléctrico", 0.0_f32), ("metal", 1.0)] {
            let a = pinta(prata(0.30, metal), &acesas, ceu);
            let b = pinta(prata(0.05, metal), &acesas, ceu);
            let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
            let mut n = 0_usize;
            let mut peca = 0_usize;
            for i in 0..ca.len() {
                if !g.hit[i] {
                    continue;
                }
                peca += 1;
                if (0..3).any(|c| ca[i][c].abs_diff(cb[i][c]) > 8) {
                    n += 1;
                }
            }
            println!(
                "  {nome:12} · {n} de {peca} pixels ({:.2} %)",
                100.0 * n as f64 / peca as f64
            );
        }

        // ⭐⭐ **O CONTROLO DA RÉGUA DA ESTRUTURA** — um céu com uma ARESTA. Sem ele, um `|∇²|` que lê o
        // mesmo número em tudo é indistinguível de uma régua CEGA (`CLAUDE.md` §5.0).
        struct Aresta;
        impl ph2d_material::Environment for Aresta {
            fn radiance(&self, d: [f32; 3], _a: f32) -> [f32; 3] {
                if d[1] > 0.2 { [3.0; 3] } else { [0.05; 3] }
            }
            fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
                [0.4; 3]
            }
        }
        println!("\nCONTROLO DA RÉGUA — o mesmo material sob um céu com ARESTA (não pré-filtrado)");
        println!("material   · estrutura sob a rampa · sob a aresta");
        for (nome, m) in [
            ("metal 0,05", prata(0.05, 1.0)),
            ("metal 1,00", prata(1.00, 1.0)),
            ("dieléctrico", prata(0.30, 0.0)),
        ] {
            println!(
                "{nome:12} · {:19.3} · {:12.3}",
                estrutura(&pinta(m, &acesas, ceu)),
                estrutura(&pinta(m, &acesas, &Aresta)),
            );
        }

        println!("\nDE ONDE VEM A LUZ (média do verde na peça)");
        println!("material        · tudo · só a lâmpada · só o céu");
        for (nome, m) in [
            ("metal 0,05", prata(0.05, 1.0)),
            ("metal 0,30", prata(0.30, 1.0)),
            ("metal 1,00", prata(1.00, 1.0)),
            ("dieléctrico   ", prata(0.30, 0.0)),
        ] {
            println!(
                "{nome:15} · {:4.0} · {:12.0} · {:8.0}",
                media(&pinta(m, &acesas, ceu)),
                media(&pinta(m, &acesas, &Black)),
                media(&pinta(m, &apagadas, ceu)),
            );
        }
    }
}
