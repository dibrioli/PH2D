//! ⭐⭐⭐ **O GRÃO DA OCLUSÃO** — a sonda que o report do dono de 2026-09-14 exigiu.
//!
//! A foto dele aponta para dentro de um furo e diz *«baixíssima qualidade»*. O que ali se vê é
//! **ruído de estimador**: a oclusão são `OCCLUSION_PASSES` raios BINÁRIOS sorteados por pixel, e
//! um estimador binário de `N` amostras tem desvio-padrão `√(p(1−p)/N)` — a `N = 16` e `p = 0,5`
//! isso é **`12,5 %` da faixa, por pixel**.
//!
//! # ⚠️ Duas colunas, porque são DUAS perguntas
//!
//! - **`|Δ ref|`** — a distância à resposta convergida (`1024` raios), pixel a pixel. Para um
//!   estimador sem viés isto **é** o ruído; para um estimador determinístico é o VIÉS.
//! - **`grão`** — `|ao(i) − média dos 8 vizinhos|` sobre **remendos lisos** (os nove pixels acertam
//!   a peça, as nove normais dentro de `8°`). É o que o OLHO vê: um viés suave de `5 %` é
//!   invisível e um erro aleatório de `5 %` por pixel é berrante.
//!
//! ⚠️ **A régua do grão precisa do seu próprio controlo** e ele está na tabela: a linha de `1024`
//! raios tem de ler `grão ≈ 0`. Se ela lesse alto, a régua estaria a medir a oclusão VERDADEIRA a
//! variar entre vizinhos — *uma régua que acusa o lado convergido não acusa ruído nenhum*.
//!
//! ```text
//! cargo run --release -p ph2d-field-gpu --example ao_grain
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

/// ⭐ **A peça da FOTO** — a cena 1 do smoke (três cilindros em união suave) com um furo passante.
/// O furo é o sujeito: é lá dentro que a seta vermelha do dono aponta.
fn peca_da_foto() -> FieldDoc {
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
    .expect("a peça da foto é um documento válido")
}

fn luz_do_rig(cam: &Orbit) -> [f32; 3] {
    let (right, up, toward_eye) = cam.basis();
    let ecra = [-0.5566703_f32, 0.6634139, 0.5];
    let r = 2.0 * cam.half_extent;
    [0, 1, 2].map(|i| {
        cam.target[i] + r * (ecra[0] * right[i] + ecra[1] * up[i] + ecra[2] * toward_eye[i])
    })
}

/// ⭐⭐⭐ **O céu do rig** — uma cúpula chapada, que é o que a peça da foto tem de indirecto.
struct Ceu([f32; 3]);
impl ph2d_material::Environment for Ceu {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        self.0
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        self.0
    }
}

fn quantil(v: &mut [f32], q: f64) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let i = ((v.len() - 1) as f64 * q).round() as usize;
    v[i]
}

fn main() {
    // ⚠️ O quadro ASSENTE do modelador é `1920×1080` — a decisão do número de cones tem de sair
    // daí, e não de uma resolução de conveniência.
    let escala: u32 = std::env::var("PH2D_AO_W")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(960);
    let (w, h) = (escala, escala * 9 / 16);
    #[allow(non_snake_case)]
    let (W, H) = (w, h);
    const REF: u32 = 1024;

    let doc = peca_da_foto();
    let campo = ph2d_field_eval::Field::new(&doc);
    let Some(fita) = campo.tape_wgsl() else {
        println!("a peça não tem fita — nada a medir");
        return;
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    // O enquadramento da foto: perto, com o furo a encher o quadro.
    let cam = Orbit {
        half_extent: std::env::var("PH2D_AO_EXT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.42),
        ..Orbit::default()
    };
    let (right, up, fwd) = cam.basis();
    let screen = Screen::new(W, H, cam.half_extent);
    let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
        .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
    let passo = ph2d_field_eval::safe_march_step(&doc);
    let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
    let nitidez = ph2d_field_render::Sharpness::for_frame(cam.half_extent, W.min(H) as usize);
    let setup = |raios: u32| ph2d_field_gpu::trace::MarchSetup {
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
        lamp: luz_do_rig(&cam),
        ball_center: bola.center,
        ball_radius: bola.radius,
        ao_rays: raios,
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
        println!("sem adaptador de GPU — sonda saltada");
        return;
    };

    // ⭐⭐⭐ **O RIG DE SOMBREAMENTO — o caminho por onde o dono vê.** A oclusão multiplica só o
    // INDIRECTO, e o furo da foto é escuro; é depois da curva sRGB que um erro linear pequeno num
    // pixel escuro vira uma dezena de níveis.
    let ceu = Ceu([0.30, 0.33, 0.38]);
    let superficie = ph2d_material::OpenPbr {
        base_color: [0.82, 0.82, 0.84],
        specular_roughness: 0.35,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare();
    let todos = [superficie];
    let pontuais = [ph2d_field_render::PointLamp {
        world: luz_do_rig(&cam),
        radiance_at_one: [0.85, 0.82, 0.78],
    }];
    let pinta = |g: &ph2d_field_render::Gbuffer, sh: &ph2d_field_render::Shadows| {
        ph2d_field_render::shade_render(
            g,
            &cam,
            &ph2d_field_render::Surfaces {
                all: &todos,
                owners: None,
            },
            &ph2d_field_render::Lighting {
                lamps: &[],
                points: &pontuais,
                sky: &ceu,
                shadows: Some(sh),
            },
            ph2d_view_transform::Look::default(),
            [40, 40, 40, 255],
        )
    };

    // A REFERÊNCIA convergida.
    let dev_ref = tracer.frame(&fita, setup(REF), W, H);
    let (g_ref, sh_ref) = dev_ref.to_cpu(&cam, screen);
    let ao_ref: Vec<f32> = (0..g_ref.hit.len()).map(|i| sh_ref.ambient_at(i)).collect();
    let px_ref = pinta(&g_ref, &sh_ref);

    // ⭐ Os REMENDOS LISOS: o pixel e os 8 vizinhos acertam, e as 9 normais concordam.
    let w = W as usize;
    let lisos: Vec<usize> = (0..g_ref.hit.len())
        .filter(|&i| {
            let (x, y) = (i % w, i / w);
            if x == 0 || y == 0 || x + 1 >= w || y + 1 >= H as usize {
                return false;
            }
            let n0 = g_ref.normal[i];
            (0..9).all(|k| {
                let j = i + (k / 3) * w + (k % 3) - w - 1;
                if !g_ref.hit[j] {
                    return false;
                }
                let n = g_ref.normal[j];
                let d = (n0[0] * n[0] + n0[1] * n[1] + n0[2] * n[2]).clamp(-1.0, 1.0);
                d.acos().to_degrees() < 8.0
            })
        })
        .collect();
    let na_peca = g_ref.hit.iter().filter(|h| **h).count();
    println!(
        "peça: {na_peca} pixels · remendos lisos: {} ({:.1} % da peça) · alcance {:.3}",
        lisos.len(),
        100.0 * lisos.len() as f64 / na_peca as f64,
        ph2d_field_render::OCCLUSION_REACH * cam.half_extent
    );
    println!();
    println!(
        "  raios ·  ms  ·  |Δ ref| p50 ·  p99  ·  máx  ·  GRÃO p50 ·  p99  ·  máx  ·  BYTES p99 · máx"
    );

    for raios in [16u32, 32, 48, 64, 96, REF] {
        let t0 = std::time::Instant::now();
        let dev = tracer.frame(&fita, setup(raios), W, H);
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let (g, sh) = dev.to_cpu(&cam, screen);
        let ao: Vec<f32> = (0..g_ref.hit.len()).map(|i| sh.ambient_at(i)).collect();
        let px = pinta(&g, &sh);
        // ⚠️ **O byte é o que o dono vê.** Comparado com o quadro convergido, no MESMO pixel.
        let mut bytes: Vec<f32> = lisos
            .iter()
            .map(|&i| {
                let d = (0..3)
                    .map(|c| (i32::from(px[i * 4 + c]) - i32::from(px_ref[i * 4 + c])).abs())
                    .max()
                    .unwrap_or(0);
                #[allow(clippy::cast_precision_loss)]
                {
                    d as f32
                }
            })
            .collect();

        let mut dref: Vec<f32> = lisos.iter().map(|&i| (ao[i] - ao_ref[i]).abs()).collect();
        let mut grao: Vec<f32> = lisos
            .iter()
            .map(|&i| {
                let mut s = 0.0f32;
                for k in 0..9 {
                    if k == 4 {
                        continue;
                    }
                    s += ao[i + (k / 3) * w + (k % 3) - w - 1];
                }
                (ao[i] - s / 8.0).abs()
            })
            .collect();
        println!(
            "  {raios:5} · {ms:5.1} ·   {:9.4} · {:6.4} · {:6.4} ·  {:8.4} · {:6.4} · {:6.4} ·   \
             {:7.1} · {:4.0}",
            quantil(&mut dref, 0.50),
            quantil(&mut dref, 0.99),
            quantil(&mut dref, 1.0),
            quantil(&mut grao, 0.50),
            quantil(&mut grao, 0.99),
            quantil(&mut grao, 1.0),
            quantil(&mut bytes, 0.99),
            quantil(&mut bytes, 1.0),
        );
    }
    // ⭐ **E as IMAGENS** — a régua diz um número e o report do dono é uma FOTO. As duas têm de
    // concordar, senão a régua está a medir outra coisa.
    if let Ok(dir) = std::env::var("PH2D_AO_DUMP") {
        let quantos: u32 = std::env::var("PH2D_AO_N")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(ph2d_field_render::OCCLUSION_PASSES);
        let dev16 = tracer.frame(&fita, setup(quantos), W, H);
        let (g16, sh16) = dev16.to_cpu(&cam, screen);
        let escreve = |nome: &str, rgb: &dyn Fn(usize) -> [u8; 3]| {
            let mut buf = format!("P6\n{W} {H}\n255\n").into_bytes();
            for i in 0..(W * H) as usize {
                buf.extend_from_slice(&rgb(i));
            }
            std::fs::write(format!("{dir}/{nome}.ppm"), buf).expect("escrever o dump");
        };
        let px16 = pinta(&g16, &sh16);
        escreve(format!("quadro_{quantos}").as_str(), &|i| {
            [px16[i * 4], px16[i * 4 + 1], px16[i * 4 + 2]]
        });
        escreve("quadro_ref", &|i| {
            [px_ref[i * 4], px_ref[i * 4 + 1], px_ref[i * 4 + 2]]
        });
        // A oclusão CRUA, esticada: o que a régua vê.
        let cinza = |v: f32| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let b = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            [b, b, b]
        };
        escreve(format!("oclusao_{quantos}").as_str(), &|i| {
            cinza(sh16.ambient_at(i))
        });
        // A NORMAL, que ninguém tinha olhado: se ela salta entre vizinhos, a oclusão é inocente.
        escreve("normal_16", &|i| {
            let n = g16.normal[i];
            [0, 1, 2].map(|c| {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    ((n[c] * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0 + 0.5) as u8
                }
            })
        });
        println!("imagens em {dir}");
    }

    println!();
    println!(
        "⚠️ a linha de {REF} raios é o CONTROLO da régua do grão: ela tem de ler ~0, senão a régua \
         está a medir a oclusão verdadeira a variar entre vizinhos."
    );
}
