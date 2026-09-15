//! ⭐⭐⭐ **PARA ONDE VÃO OS MILISSEGUNDOS DE UM QUADRO** — a sonda que o report do dono de
//! 2026-09-15 exigiu (*«performance bem aquém de Unreal e outros renders de tempo real»*).
//!
//! # O que ela separa, e porquê por DIFERENÇAS
//!
//! O compute e a cópia de leitura vivem no **mesmo `submit`** ([`ph2d_field_gpu::trace`]), logo um
//! relógio à volta dele mede os dois juntos. ⇒ as fases saem de diferenças entre corridas que só
//! mudam **uma** coisa, e a leitura é medida **à parte**, no mesmo volume de bytes.
//!
//! ⚠️ **Nenhuma linha desta sonda está no caminho do produto** — ela chama as mesmas portas.
//!
//! ```text
//! cargo run --release -p ph2d-field-gpu --example frame_budget
//! ```
use ph2d_field::{Blend, FieldDoc, Node, NodeId, NodeKind, Op, Primitive, Xform};
use ph2d_field_render::{Orbit, Screen};

fn leaf(p: Primitive, x: Xform) -> Node {
    Node {
        xform: x,
        kind: NodeKind::Leaf(p),
        mods: Vec::new(),
        verb: None,
    }
}
fn combine(op: Op, children: Vec<NodeId>) -> Node {
    Node {
        xform: Xform::IDENTITY,
        kind: NodeKind::Combine { op, children },
        mods: Vec::new(),
        verb: None,
    }
}

/// A peça da foto do dono: a cruz de três cilindros com um furo passante.
fn peca() -> FieldDoc {
    let s = std::f32::consts::FRAC_1_SQRT_2;
    let cyl = |rot: [f32; 4]| {
        leaf(
            Primitive::Cylinder {
                radius: 0.22,
                half_height: 0.78,
                round: 0.05,
                chamfer: 0.0,
            },
            Xform {
                rotation: rot,
                ..Xform::IDENTITY
            },
        )
    };
    FieldDoc::new(
        vec![
            cyl([0.0, 0.0, 0.0, 1.0]),
            cyl([s, 0.0, 0.0, s]),
            cyl([0.0, s, 0.0, s]),
            combine(
                Op::Union(Blend::Exact { radius: 0.12 }),
                vec![NodeId(0), NodeId(1), NodeId(2)],
            ),
            leaf(
                Primitive::Cylinder {
                    radius: 0.17,
                    half_height: 1.2,
                    round: 0.0,
                    chamfer: 0.0,
                },
                Xform {
                    rotation: [s, 0.0, 0.0, s],
                    ..Xform::IDENTITY
                },
            ),
            combine(
                Op::Difference(Blend::Exact { radius: 0.03 }),
                vec![NodeId(3), NodeId(4)],
            ),
        ],
        NodeId(5),
    )
    .expect("a peça")
}

struct Ceu([f32; 3]);
impl ph2d_material::Environment for Ceu {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        self.0
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        self.0
    }
}

/// ⚠️ **O MÍNIMO de `n` corridas, com o `/proc/loadavg` ao lado** — o `CLAUDE.md` §5.0 proíbe
/// leitura de relógio desta máquina acima de `load ~5`, e a mediana de uma máquina ocupada mede a
/// máquina.
fn minimo(n: usize, mut f: impl FnMut()) -> f64 {
    let mut melhor = f64::INFINITY;
    for _ in 0..n {
        let t = std::time::Instant::now();
        f();
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
    }
    melhor
}

fn main() {
    let escala: u32 = std::env::var("PH2D_AO_W")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1920);
    let (w, h) = (escala, escala * 9 / 16);
    println!(
        "{w}×{h} · loadavg {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or("?")
    );

    let doc = peca();
    let campo = ph2d_field_eval::Field::new(&doc);
    let Some(fita) = campo.tape_wgsl() else {
        return;
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = Orbit::default();
    let (right, up, fwd) = cam.basis();
    let screen = Screen::new(w, h, cam.half_extent);
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let passo = ph2d_field_eval::safe_march_step(&doc);
    let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
    let nitidez = ph2d_field_render::Sharpness::for_frame(cam.half_extent, w.min(h) as usize);
    let luz = {
        let (r, u, t) = cam.basis();
        let ecra = [-0.5566703_f32, 0.6634139, 0.5];
        let raio = 2.0 * cam.half_extent;
        [0, 1, 2].map(|i| cam.target[i] + raio * (ecra[0] * r[i] + ecra[1] * u[i] + ecra[2] * t[i]))
    };
    let setup = |cones: u32| ph2d_field_gpu::trace::MarchSetup {
        antialias: true,
        half_extent: cam.half_extent,
        half_px: screen.half(),
        target: cam.target,
        right,
        up,
        fwd,
        ortho_start: ph2d_field_render::ORTHO_START,
        eye_distance: cam.eye_distance().unwrap_or(0.0),
        hit_eps: nitidez.hit,
        normal_eps: nitidez.normal,
        lamps: {
            // ⚠️ A cauda fica a zero: só as `n_lamps` primeiras são lidas.
            let mut v = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
            v[0] = luz;
            v
        },
        n_lamps: 1,
        ball_center: bola.center,
        ball_radius: bola.radius,
        ao_rays: cones,
        ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
        edge_cos: ph2d_field_render::EDGE_COS,
        step: passo,
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
            / passo.clamp(f32::EPSILON, 1.0))
        .ceil() as u32,
        t_max: ph2d_field_render::T_MAX,
    };

    let Some(mut tracer) = ph2d_field_gpu::trace::Tracer::new() else {
        println!("sem adaptador — saltada");
        return;
    };
    // Um quadro para compilar o shader; ele não entra em medição nenhuma.
    let _ = tracer.frame(&fita, &[], setup(0), w, h);

    let cheio = minimo(5, || {
        std::hint::black_box(tracer.frame(
            &fita,
            &[],
            setup(ph2d_field_render::OCCLUSION_PASSES),
            w,
            h,
        ));
    });
    let sem_ao = minimo(5, || {
        std::hint::black_box(tracer.frame(&fita, &[], setup(0), w, h));
    });

    // ⭐ A LEITURA, medida à parte: o mesmo volume de bytes que o quadro devolve.
    let n = u64::from(w) * u64::from(h);
    let bytes = n * 24; // `centro` 16 B + `luz` 8 B
    let leitura = tracer.mede_leitura(bytes, 5);

    // O que a CPU faz depois: reconstruir o ponto, suavizar, e PINTAR.
    let dev = tracer.frame(&fita, &[], setup(ph2d_field_render::OCCLUSION_PASSES), w, h);
    let converter = minimo(5, || {
        std::hint::black_box(dev.to_cpu(&cam, screen));
    });
    let (g, sh) = dev.to_cpu(&cam, screen);
    // ⭐ As DUAS metades do `to_cpu`, separadas: reconstruir o ponto e suavizar a oclusão.
    let suavizar = minimo(5, || {
        std::hint::black_box(ph2d_field_render::blur_occlusion(&g, &dev.ambient));
    });
    let ceu = Ceu([0.30, 0.33, 0.38]);
    let sup = [ph2d_material::OpenPbr {
        base_color: [0.82, 0.82, 0.84],
        specular_roughness: 0.35,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let pontuais = [ph2d_field_render::PointLamp {
        world: luz,
        radiance_at_one: [0.85, 0.82, 0.78],
    }];
    let pintar = minimo(5, || {
        std::hint::black_box(ph2d_field_render::shade_render(
            &g,
            &cam,
            &ph2d_field_render::Surfaces {
                all: &sup,
                owners: None,
            },
            &ph2d_field_render::Lighting {
                lamps: &[],
                points: &pontuais,
                sky: &ceu,
                shadows: Some(&sh),
            },
            ph2d_view_transform::Look::default(),
            [40, 40, 40, 255],
        ));
    });

    // ⭐ As peças do `to_cpu`, uma a uma — duas hipóteses caíram por adivinhação antes disto.
    let clonar_normal = minimo(5, || {
        std::hint::black_box(dev.normal.clone());
    });
    let clonar_sombra = minimo(5, || {
        std::hint::black_box(dev.shadow.clone());
    });
    let raios = cam.rays();
    let so_o_laco = minimo(5, || {
        let mut ponto: Vec<[f32; 3]> = Vec::with_capacity(dev.t.len());
        for y in 0..h as usize {
            let py = y as f32 + 0.5;
            for x in 0..w as usize {
                let t = dev.t[y * w as usize + x];
                let (uu, vv) = screen.plane_at(x as f32 + 0.5, py);
                let (o, d) = raios.at_plane(uu, vv);
                ponto.push([o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t]);
            }
        }
        std::hint::black_box(ponto);
    });
    // V1: o `plane_at` com RECÍPROCO em vez de duas divisões por pixel.
    let escala_u = cam.half_extent / screen.half();
    let (meio_x, meio_y) = (w as f32 * 0.5, h as f32 * 0.5);
    let v1 = minimo(5, || {
        let mut ponto: Vec<[f32; 3]> = Vec::with_capacity(dev.t.len());
        for y in 0..h as usize {
            let vv = -(y as f32 + 0.5 - meio_y) * escala_u;
            for x in 0..w as usize {
                let t = dev.t[y * w as usize + x];
                let uu = (x as f32 + 0.5 - meio_x) * escala_u;
                let (o, d) = raios.at_plane(uu, vv);
                ponto.push([o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t]);
            }
        }
        std::hint::black_box(ponto);
    });
    // V2: V1 mais a NORMALIZAÇÃO dobrada — `p = olho + dvec · (t/len)`, uma divisão em vez de três.
    let olho = raios.eye.unwrap_or(raios.target);
    let v2 = minimo(5, || {
        let mut ponto: Vec<[f32; 3]> = Vec::with_capacity(dev.t.len());
        for y in 0..h as usize {
            let vv = -(y as f32 + 0.5 - meio_y) * escala_u;
            for x in 0..w as usize {
                let t = dev.t[y * w as usize + x];
                let uu = (x as f32 + 0.5 - meio_x) * escala_u;
                let dv = [0, 1, 2]
                    .map(|i| raios.target[i] + raios.right[i] * uu + raios.up[i] * vv - olho[i]);
                let k = t / (dv[0] * dv[0] + dv[1] * dv[1] + dv[2] * dv[2]).sqrt();
                ponto.push([
                    olho[0] + dv[0] * k,
                    olho[1] + dv[1] * k,
                    olho[2] + dv[2] * k,
                ]);
            }
        }
        std::hint::black_box(ponto);
    });
    let so_alocar = minimo(5, || {
        let mut ponto: Vec<[f32; 3]> = Vec::with_capacity(dev.t.len());
        for _ in 0..dev.t.len() {
            ponto.push([0.0; 3]);
        }
        std::hint::black_box(ponto);
    });

    println!();
    println!("  fase                                    ms");
    println!(
        "  quadro do dispositivo, {:2} cones      {cheio:7.2}",
        ph2d_field_render::OCCLUSION_PASSES
    );
    println!(
        "    dos quais OCLUSÃO                  {:7.2}",
        cheio - sem_ao
    );
    println!(
        "    dos quais LEITURA de volta         {leitura:7.2}   ({:.1} MB)",
        bytes as f64 / 1e6
    );
    println!(
        "    dos quais marcha+sombra+bordas     {:7.2}",
        sem_ao - leitura
    );
    println!("  reconstruir o ponto + suavizar        {converter:7.2}");
    println!("    dos quais SUAVIZAR a oclusão       {suavizar:7.2}");
    println!(
        "    dos quais reconstruir o PONTO      {:7.2}",
        converter - suavizar
    );
    println!("      · so' o laco (24 MB)              {so_o_laco:7.2}");
    println!("      · V1: reciproco no plane_at       {v1:7.2}");
    println!("      · V2: V1 + normalizacao dobrada   {v2:7.2}");
    println!("      · so' ENCHER 24 MB de zeros       {so_alocar:7.2}");
    println!("      · clonar a normal (24 MB)         {clonar_normal:7.2}");
    println!("      · clonar a sombra (8 MB)          {clonar_sombra:7.2}");
    println!("  PINTAR na CPU                         {pintar:7.2}");
    println!("  ──────────────────────────────────────────────");
    println!(
        "  ponta a ponta                         {:7.2}",
        cheio + converter + pintar
    );

    // ⭐⭐⭐ **O QUADRO DE MOVIMENTO**, que desde 2026-09-15 também passa pelo dispositivo: a
    // resolução de pré-visualização (divisor `3`) e o anti-serrilhado DESLIGADO (a lei da W73).
    let (mw, mh) = (w / 3, h / 3);
    let tela_m = Screen::new(mw, mh, cam.half_extent);
    let nitidez_m = ph2d_field_render::Sharpness::for_frame(cam.half_extent, mw.min(mh) as usize);
    let mut setup_m = setup(ph2d_field_render::OCCLUSION_PASSES);
    setup_m.antialias = false;
    setup_m.half_px = tela_m.half();
    setup_m.hit_eps = nitidez_m.hit;
    setup_m.normal_eps = nitidez_m.normal;
    let _ = tracer.frame(&fita, &[], setup_m, mw, mh);
    let mov_placa = minimo(5, || {
        std::hint::black_box(tracer.frame(&fita, &[], setup_m, mw, mh));
    });
    let dev_m = tracer.frame(&fita, &[], setup_m, mw, mh);
    let mov_cpu = minimo(5, || {
        std::hint::black_box(dev_m.to_cpu(&cam, tela_m));
    });
    let (gm, shm) = dev_m.to_cpu(&cam, tela_m);
    let mov_pintar = minimo(5, || {
        std::hint::black_box(ph2d_field_render::shade_render(
            &gm,
            &cam,
            &ph2d_field_render::Surfaces {
                all: &sup,
                owners: None,
            },
            &ph2d_field_render::Lighting {
                lamps: &[],
                points: &pontuais,
                sky: &ceu,
                shadows: Some(&shm),
            },
            ph2d_view_transform::Look::default(),
            [40, 40, 40, 255],
        ));
    });
    let mov = mov_placa + mov_cpu + mov_pintar;
    println!();
    println!("  QUADRO DE MOVIMENTO ({mw}×{mh}, sem anti-serrilhado, COM oclusão)");
    println!("    placa                               {mov_placa:7.2}");
    println!("    reconstruir + suavizar              {mov_cpu:7.2}");
    println!("    pintar                              {mov_pintar:7.2}");
    println!("    ──────────────────────────────────────────────");
    println!(
        "    ponta a ponta                       {mov:7.2}   ⇒ {:5.1} quadros por segundo",
        1000.0 / mov
    );
}
