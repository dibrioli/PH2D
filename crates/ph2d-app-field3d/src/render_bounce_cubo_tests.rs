//! ⭐⭐⭐ **AS SONDAS NA FACE DO CUBO** — a segunda metade do report de 2026-09-17 (*«completamente
//! imprestável. Parece mais um reflexo mal feito»*).
//!
//! # Por que um ficheiro irmão
//!
//! O [`super::render_bounce_vaso_tests`] mede o INTERIOR do vaso (os terraços, a escada do borrão,
//! de onde vem o degrau); estas três medem a **face plana do cubo ao lado dele**, que é onde o
//! segundo report apontou — e elas partilham uma fixtura própria, a `vaso_e_cubo`.
//!
//! ⛔ **Split, nunca allowlist** (`CLAUDE.md` §5.0): o ficheiro passou o tecto de LOC da workspace,
//! e a fronteira já estava escrita nele — *duas fixturas, dois assuntos*.

use ph2d_field_render::{Orbit, Surfaces};

/// ⭐⭐⭐ **A FOTO DO DONO DE 2026-09-17 (2.ª ronda): o vaso VERMELHO com um cubo BRANCO encostado.**
///
/// *«Absolutamente nenhuma qualidade. Completamente imprestável. Parece mais um reflexo mal
/// feito»* — e a seta dele aponta à FACE do cubo virada ao vaso, onde a luz devolvida sai como uma
/// imagem esborratada e com arestas.
///
/// ⚠️ Esta fixtura existe porque a anterior (o vaso sozinho) não continha o receptor que ele
/// fotografou: uma **face PLANA** ao lado de um emissor colorido, que é onde uma soma de projecções
/// se lê como imagem.
fn vaso_e_cubo() -> (
    ph2d_field::FieldDoc,
    Vec<ph2d_field::FieldDoc>,
    Vec<ph2d_material::Surface>,
    Orbit,
) {
    use ph2d_field::{Node, NodeId, NodeKind, Op, Primitive, Xform};
    let vaso = crate::smoke::scenes::vaso(ph2d_field::DEFAULT_PROFILE_RESOLUTION);
    let vaso_no = vaso.nodes()[0].clone();
    // ⚠️ O cubo vai para o lado de onde a câmera VÊ a face dele virada ao vaso — derivado da base
    // da câmera, e não escrito: a 1.ª redacção pô-lo a `+x` e a face ficou de costas para a lente
    // (`0` pixels), porque a câmera de omissão olha de `+x`.
    let mut cam = Orbit::default();
    let (_, _, olho) = cam.basis();
    let lado = if olho[0] >= 0.0 { -1.0f32 } else { 1.0 };
    let frente = if olho[2] >= 0.0 { 1.0f32 } else { -1.0 };
    let cubo = ph2d_field_eval::leaf(
        Primitive::Box {
            half: [0.35, 0.35, 0.35],
            round: 0.03,
            chamfer: 0.0,
        },
        Xform::at(0.74 * lado, -0.10, 0.10 * frente),
    );
    let folhas = [vaso_no, cubo];
    let mut nos: Vec<Node> = folhas.to_vec();
    nos.push(Node::new(
        Xform::IDENTITY,
        NodeKind::Combine {
            op: Op::Union(ph2d_field::Blend::Sharp),
            children: vec![NodeId(0), NodeId(1)],
        },
    ));
    let doc = ph2d_field::FieldDoc::new(nos, NodeId(2)).expect("vaso e cubo");
    let postas = folhas
        .iter()
        .map(|n| ph2d_field::FieldDoc::new(vec![n.clone()], NodeId(0)).expect("a folha posta"))
        .collect();
    let mats = vec![
        ph2d_material::OpenPbr {
            base_color: [0.80, 0.08, 0.06],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
        ph2d_material::OpenPbr {
            base_color: [0.90, 0.90, 0.90],
            ..ph2d_material::OpenPbr::default()
        }
        .prepare(),
    ];
    cam.target = [0.35 * lado, 0.0, 0.0];
    cam.half_extent = 1.0;
    (doc, postas, mats, cam)
}

/// ⏱️⭐⭐⭐ **A RÉGUA NA FACE DO CUBO — e a soma aberta direcção a direcção numa LINHA dela.**
#[test]
#[ignore = "sonda de diagnóstico: a foto do cubo, direcção a direcção"]
fn sonda_o_reflexo_mal_feito_na_face_do_cubo() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 256.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    // A FACE: os pixels do cubo (dono 1) cuja normal aponta para o vaso, em mundo.
    // A normal do g-buffer é de VISTA; a base da câmera (a mesma que o pintor usa) devolve-a ao mundo.
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let na_face: Vec<bool> = (0..g.hit.len())
        .map(|i| {
            g.hit[i] && donos.at(g.point[i]) == Some(1) && {
                let v = g.normal[i];
                let nx = right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2];
                nx * para_o_vaso > 0.9
            }
        })
        .collect();
    let face_px = na_face.iter().filter(|b| **b).count();
    println!("  a face do cubo virada ao vaso: {face_px} px");
    assert!(face_px > 500, "a face não está na tela");

    let cru = |dirs: u32| {
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], dirs)
    };
    // Um g-buffer só com a face, para a régua dos terraços medir só ela.
    // O `Gbuffer` não é `Clone`; traçar outra vez a `256²` custa nada e dá a régua um g-buffer
    // com a MÁSCARA da face.
    let mut gf = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    gf.hit.copy_from_slice(&na_face);
    let convergida = cru(1024);
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let media_face = {
        let (mut s, mut n) = (0.0f64, 0usize);
        for (i, &face) in na_face.iter().enumerate() {
            if face {
                s += f64::from(lum(&convergida, i));
                n += 1;
            }
        }
        (s / n.max(1) as f64) as f32
    };
    println!("  ricochete médio na face (convergida, 1024 dir): {media_face:.5}");
    println!("  direcções ·  terraços p99 / máx NA FACE  ·  erro médio vs convergida (% da média)");
    for dirs in [48u32, 96, 256] {
        let c = ph2d_field_render::blur_bounce(&g, &cru(dirs));
        let (p99, mx) = ph2d_field_render::banda::terracos(&gf, &|i| lum(&c, i));
        let (mut e, mut n) = (0.0f64, 0usize);
        for (i, &face) in na_face.iter().enumerate() {
            if face {
                e += f64::from((lum(&c, i) - lum(&convergida, i)).abs());
                n += 1;
            }
        }
        let erro = 100.0 * (e / n.max(1) as f64) as f32 / media_face.max(1e-9);
        println!("  {dirs:>9} · {p99:>9.4} / {mx:>8.4} · {erro:>6.2} %");
    }
    let ceu = ph2d_field_render::blur_occlusion(
        &g,
        &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, 48),
    );
    let (cp, cm) = ph2d_field_render::banda::terracos(&gf, &|i| ceu[i]);
    println!("  o CÉU na mesma face, para comparar: {cp:.4} / {cm:.4}");

    // ── A SOMA ABERTA numa linha da face: cada direcção como uma FITA de '#' (contribui) e '.' ──
    let linhas_da_face: Vec<usize> = (0..h as usize)
        .filter(|y| {
            (0..w as usize)
                .filter(|x| na_face[y * w as usize + x])
                .count()
                > 40
        })
        .collect();
    let y = linhas_da_face[linhas_da_face.len() / 2];
    let xs: Vec<usize> = (0..w as usize)
        .filter(|x| na_face[y * w as usize + x])
        .collect();
    let (x0, x1) = (xs[0], *xs.last().unwrap());
    println!(
        "  linha y={y}, x∈[{x0},{x1}] ({} px) — uma fita por direcção que contribui:",
        xs.len()
    );
    let dirs = 48u32;
    let mut fitas = 0usize;
    let mut arestas = 0usize;
    for k in 0..dirs {
        let f = ph2d_field_render::bounce::bounce_slice(
            &doc,
            &reg,
            &cam,
            &g,
            &surfaces,
            &[lampada],
            k,
            1,
            dirs,
        );
        let pesa = xs.iter().any(|&x| f.weight[y * w as usize + x] > 0.0);
        if !pesa {
            continue;
        }
        fitas += 1;
        let mut fita = String::new();
        let mut antes = false;
        for (m, &x) in xs.iter().enumerate() {
            let s = f.sum[y * w as usize + x];
            let acende = (s[0] + s[1] + s[2]) > 1e-6;
            if m > 0 && acende != antes {
                arestas += 1;
            }
            antes = acende;
            if m % 2 == 0 {
                fita.push(if acende { '#' } else { '.' });
            }
        }
        println!("  {k:>3} {fita}");
    }
    println!(
        "  {fitas} direcções contribuem nesta linha e trocam de resposta {arestas} vezes ao longo dela"
    );
}

/// ⏱️⭐⭐⭐ **AS SONDAS contra a recolha POR PIXEL, na face do cubo e no vaso — as duas contra a
/// mesma convergida.** A pergunta que decide a wave: *a interpolação entre sondas apaga a estrutura
/// que o dono fotografou, e a que preço de exactidão?*
#[test]
#[ignore = "sonda de decisão: sondas contra por-pixel"]
fn sonda_as_sondas_contra_o_por_pixel() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (256u32, 256u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 256.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let na_face: Vec<bool> = (0..g.hit.len())
        .map(|i| {
            g.hit[i] && donos.at(g.point[i]) == Some(1) && {
                let v = g.normal[i];
                let nx = right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2];
                nx * para_o_vaso > 0.9
            }
        })
        .collect();
    let no_vaso: Vec<bool> = (0..g.hit.len())
        .map(|i| g.hit[i] && donos.at(g.point[i]) == Some(0))
        .collect();
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let convergida =
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 1024);
    let mascara = |m: &[bool]| {
        let mut gm = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
        gm.hit.copy_from_slice(m);
        gm
    };
    let (gf, gv) = (mascara(&na_face), mascara(&no_vaso));
    let regua = |nome: &str, c: &[[f32; 3]]| {
        for (rot, m, gm) in [("face", &na_face, &gf), ("vaso", &no_vaso, &gv)] {
            let (p99, mx) = ph2d_field_render::banda::terracos(gm, &|i| lum(c, i));
            let (mut e, mut s) = (0.0f64, 0.0f64);
            for (i, &dentro) in m.iter().enumerate() {
                if dentro {
                    e += f64::from((lum(c, i) - lum(&convergida, i)).abs());
                    s += f64::from(lum(&convergida, i));
                }
            }
            #[allow(clippy::cast_possible_truncation)]
            let erro = (100.0 * e / s.max(1e-9)) as f32;
            println!("  {nome:>28} · {rot} · terraços {p99:>7.4} / {mx:>7.4} · erro {erro:>6.2} %");
        }
    };
    let ceu = ph2d_field_render::blur_occlusion(
        &g,
        &ph2d_field_render::occlusion(&doc, &reg, &cam, &g, 48),
    );
    for (rot, gm) in [("face", &gf), ("vaso", &gv)] {
        let (p99, mx) = ph2d_field_render::banda::terracos(gm, &|i| ceu[i]);
        println!(
            "  {:>28} · {rot} · terraços {p99:>7.4} / {mx:>7.4}",
            "o CÉU (48 cones)"
        );
    }
    let atual = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 48),
    );
    regua("por pixel, 48 dir + 2 borrões", &atual);
    // ⚠️ A dureza da visibilidade ficou BINÁRIA por medição (a suave leu `0,67`/`0,76` na face
    // contra `0,32`), e o borrão de duas passagens é o que apaga os vincos da interpolação —
    // ver o cabeçalho das `probes`. O que se mede aqui é o TAMANHO da grelha e o que os nove
    // coeficientes custam contra a soma directa.
    for (n, dirs) in [(16usize, 256u32), (24, 256), (32, 256)] {
        let t0 = std::time::Instant::now();
        let grid = ph2d_field_render::probes::bake_probes(
            &doc,
            &reg,
            &cam,
            &surfaces,
            &[lampada],
            n,
            dirs,
            256,
        );
        let assar = t0.elapsed();
        let dentro = grid.inside.iter().filter(|b| **b).count();
        println!(
            "  sondas {n}³ × {dirs} dir · {dentro} dentro · assar {:.0} ms",
            assar.as_secs_f64() * 1e3
        );
        for directa in [true, false] {
            let t1 = std::time::Instant::now();
            let sondas =
                ph2d_field_render::probes::gather_probes_por(&doc, &reg, &cam, &g, &grid, directa);
            let recolher = t1.elapsed().as_secs_f64() * 1e3;
            let como = if directa {
                "soma directa"
            } else {
                "9 coeficientes"
            };
            regua(
                &format!("{n}³ {como} ({recolher:.0} ms) + 2 borrões"),
                &ph2d_field_render::blur_bounce(&g, &sondas),
            );
        }
    }
    // ── a ESTRUTURA do resíduo (janela 4 px): a régua que separa as duas leis ─────────────────
    println!("  estrutura de média frequência do resíduo (RMS / média, janela 4 px), face · vaso:");
    let estr = |c: &[[f32; 3]]| -> (f32, f32) {
        (
            ph2d_field_render::banda::estrutura(&gf, &|i| lum(c, i), &|i| lum(&convergida, i), 4),
            ph2d_field_render::banda::estrutura(&gv, &|i| lum(c, i), &|i| lum(&convergida, i), 4),
        )
    };
    for dirs in [48u32, 256] {
        let c = ph2d_field_render::blur_bounce(
            &g,
            &ph2d_field_render::bounce::bounce_pass(
                &doc,
                &reg,
                &cam,
                &g,
                &surfaces,
                &[lampada],
                dirs,
            ),
        );
        let (ef, ev) = estr(&c);
        println!(
            "  {:>22} · {ef:.4} · {ev:.4}",
            format!("por pixel {dirs} dir")
        );
    }
    for n in [16usize, 24, 32] {
        let grid = ph2d_field_render::probes::bake_probes(
            &doc,
            &reg,
            &cam,
            &surfaces,
            &[lampada],
            n,
            256,
            256,
        );
        let c = ph2d_field_render::blur_bounce(
            &g,
            &ph2d_field_render::probes::gather_probes(&doc, &reg, &cam, &g, &grid),
        );
        let (ef, ev) = estr(&c);
        println!("  {:>22} · {ef:.4} · {ev:.4}", format!("sondas {n}³"));
    }

    // ── o PERFIL numa linha da face: o que o olho vê ─────────────────────────────────────────
    let linhas: Vec<usize> = (0..h as usize)
        .filter(|y| {
            (0..w as usize)
                .filter(|x| na_face[y * w as usize + x])
                .count()
                > 40
        })
        .collect();
    let y = linhas[linhas.len() / 2];
    let xs: Vec<usize> = (0..w as usize)
        .filter(|x| na_face[y * w as usize + x])
        .collect();
    let grid = ph2d_field_render::probes::bake_probes(
        &doc,
        &reg,
        &cam,
        &surfaces,
        &[lampada],
        24,
        256,
        256,
    );
    let sondas = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::probes::gather_probes(&doc, &reg, &cam, &g, &grid),
    );
    let escala = xs
        .iter()
        .map(|&x| lum(&convergida, y * w as usize + x))
        .fold(0.0f32, f32::max)
        .max(1e-9);
    let perfil = |nome: &str, c: &[[f32; 3]]| {
        let s: String = xs
            .iter()
            .step_by(2)
            .map(|&x| {
                let v = (lum(c, y * w as usize + x) / escala * 9.0)
                    .round()
                    .clamp(0.0, 9.0);
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let d = v as u8;
                char::from(b'0' + d)
            })
            .collect();
        println!("  {nome:>12} {s}");
    };
    println!(
        "  o perfil da luz devolvida ao longo da linha y={y} da face (0..9 = fracção do máximo):"
    );
    perfil("convergida", &convergida);
    perfil("por pixel", &atual);
    perfil("sondas 24³", &sondas);
}

/// ⭐⭐⭐ **AS SONDAS NÃO DESENHAM A PEÇA NA FACE DO CUBO** — o gate do report (*«um reflexo mal
/// feito»*), pela régua que apanha o mecanismo ([`ph2d_field_render::banda::estrutura`]).
///
/// # ⭐ De onde a barra sai (`sonda_as_sondas_contra_o_por_pixel`, `256²`, janela `4 px`)
///
/// | lei | estrutura na face |
/// |---|---:|
/// | por pixel, `48` dir (a lei do report) | `0,106` |
/// | por pixel, `256` dir (`5,3×` o preço) | `0,045` |
/// | **sondas `32³`** | **`0,035`** |
///
/// ⇒ a barra é **`0,06`**: no vale entre o que as sondas leem e o que a lei do report lia, e
/// abaixo até do por-pixel cinco vezes mais caro. ⚠️ **O CONTROLO é metade do gate:** a lei do
/// report tem de ler ACIMA da barra nesta fixtura — senão a fixtura não contém o fenómeno e a
/// asserção sobre as sondas não afirma nada (a armadilha da paridade sobre três formas convexas).
#[test]
fn as_sondas_nao_desenham_a_peca_na_face_do_cubo() {
    let (doc, postas, mats, cam) = vaso_e_cubo();
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let (w, h) = (128u32, 128u32);
    let g = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    let donos = ph2d_field_eval::owners::Owners::new(
        &postas,
        &reg,
        ph2d_field_render::hit_tolerance(cam.half_extent, 128.0),
    );
    let surfaces = Surfaces {
        all: &mats,
        owners: Some(&donos),
    };
    let (onde, luz) = crate::lights::opening_light(&cam);
    let lampada = ph2d_field_render::PointLamp {
        world: onde,
        radiance_at_one: [luz.intensity; 3],
    };
    let (right, up, fwd) = cam.basis();
    let para_o_vaso = if fwd[0] >= 0.0 { 1.0f32 } else { -1.0 };
    let mut gf = ph2d_field_render::trace(&doc, &reg, &cam, w, h);
    for i in 0..gf.hit.len() {
        gf.hit[i] = g.hit[i] && donos.at(g.point[i]) == Some(1) && {
            let v = g.normal[i];
            (right[0] * v[0] + up[0] * v[1] + fwd[0] * v[2]) * para_o_vaso > 0.9
        };
    }
    let face = gf.hit.iter().filter(|b| **b).count();
    assert!(face > 200, "a face do cubo não está na tela: {face} px");
    let lum = |c: &[[f32; 3]], i: usize| (c[i][0] + c[i][1] + c[i][2]) / 3.0;
    let verdade =
        ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 512);
    let estrutura = |c: &[[f32; 3]]| {
        ph2d_field_render::banda::estrutura(&gf, &|i| lum(c, i), &|i| lum(&verdade, i), 4)
    };
    const BARRA: f32 = 0.06;
    let report = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::bounce::bounce_pass(&doc, &reg, &cam, &g, &surfaces, &[lampada], 48),
    );
    let sondas = ph2d_field_render::blur_bounce(
        &g,
        &ph2d_field_render::probes::probe_bounce(&doc, &reg, &cam, &g, &surfaces, &[lampada]),
    );
    let (er, es) = (estrutura(&report), estrutura(&sondas));
    println!("  estrutura na face · lei do report {er:.4} · sondas {es:.4} · barra {BARRA}");
    assert!(
        er > BARRA,
        "o CONTROLO caiu: a lei do report lê {er:.4} nesta fixtura, abaixo da barra — a face já não \
         contém o fenómeno e o gate não pode afirmar nada sobre as sondas"
    );
    assert!(
        es <= BARRA,
        "as sondas desenham a peça na face do cubo: estrutura {es:.4} contra a barra {BARRA} (a lei \
         do report lia {er:.4})"
    );
}
