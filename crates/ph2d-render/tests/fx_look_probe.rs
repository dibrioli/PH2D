//! **RENDER-AND-LOOK da pilha de FX raster** — a sonda, não um gate.
//!
//! Irmã do `push_look_probe` do Painter: um gate afirma um NÚMERO, e há defeitos cujo oráculo é a
//! FOTO (o pente do Bevel, a linha tracejada do Feather, a serrilha do contorno). Ela desenha uma
//! ESTRELA — arestas oblíquas, pontas agudas e reentrâncias, que é onde os três artefatos vivem —
//! e escreve um PPM por cena.
//!
//! ```text
//! cd <worktree> && PH2D_FX_LOOK_DIR=/tmp/fx cargo test -p ph2d-render \
//!     --test fx_look_probe -- --ignored --nocapture
//! ```
//!
//! Converter para ver: `magick /tmp/fx/<cena>.ppm /tmp/fx/<cena>.png`.

use ph2d_ecs::FxOp;
use ph2d_render::{FxOpGpu, FxStackPass, make_output_texture};

mod fx_stack_common;
use fx_stack_common::{make_src, readback, try_headless_gpu};

const W: u32 = 512;
const H: u32 = 512;
/// Supersampling da estrela — a rampa de AA tem de parecer com a que o Vello produz, senão a sonda
/// mede a própria rasterização em vez do efeito.
const SS: u32 = 4;

/// A cor da estrela do smoke (`fx_raster_smoke.rs`), para a foto ser comparável à do Enio.
const AMBER: [f32; 3] = [235.0, 175.0, 60.0];

/// Cobertura analítica de uma estrela de 5 pontas (raio interno 0.45), por supersampling.
fn star_alpha(w: u32, h: u32) -> Vec<f32> {
    let cx = f64::from(w) * 0.5;
    let cy = f64::from(h) * 0.5;
    let r_out = f64::from(w.min(h)) * 0.40;
    let r_in = r_out * 0.45;
    // Os 10 vértices, alternando externo/interno, começando pela ponta de cima.
    let mut poly = Vec::with_capacity(10);
    for i in 0..10 {
        let a = -std::f64::consts::FRAC_PI_2 + f64::from(i) * std::f64::consts::PI / 5.0;
        let r = if i % 2 == 0 { r_out } else { r_in };
        poly.push((cx + r * a.cos(), cy + r * a.sin()));
    }
    let inside = |px: f64, py: f64| -> bool {
        let mut hit = false;
        let n = poly.len();
        for i in 0..n {
            let (x0, y0) = poly[i];
            let (x1, y1) = poly[(i + 1) % n];
            if (y0 > py) != (y1 > py) {
                let t = (py - y0) / (y1 - y0);
                if px < x0 + t * (x1 - x0) {
                    hit = !hit;
                }
            }
        }
        hit
    };
    let step = 1.0 / f64::from(SS);
    let mut out = vec![0.0f32; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let mut acc = 0u32;
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = f64::from(x) + (f64::from(sx) + 0.5) * step;
                    let py = f64::from(y) + (f64::from(sy) + 0.5) * step;
                    if inside(px, py) {
                        acc += 1;
                    }
                }
            }
            out[(y * w + x) as usize] = acc as f32 / f64::from(SS * SS) as f32;
        }
    }
    out
}

/// Os dez vértices da estrela, na MESMA ordem e nos MESMOS números que a cobertura usa — se as
/// duas descrições divergirem, o campo descreve uma forma e o raster desenha outra.
fn star_poly() -> Vec<(f64, f64)> {
    let (cx, cy) = (f64::from(W) * 0.5, f64::from(H) * 0.5);
    let r_out = f64::from(W.min(H)) * 0.40;
    let r_in = r_out * 0.45;
    (0..10)
        .map(|i| {
            let a = -std::f64::consts::FRAC_PI_2 + f64::from(i) * std::f64::consts::PI / 5.0;
            let r = if i % 2 == 0 { r_out } else { r_in };
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// A MESMA silhueta, mas picada em pedaços COLINEARES — é o que o produto de facto entrega: as
/// arestas do documento são cúbicas DEGENERADAS, e o achatador as subdivide (medido no app: 150
/// segmentos para uma estrela de 10 arestas). Se o campo depender da densidade, é aqui que aparece.
fn star_segments_dense(per_edge: usize) -> Vec<[f32; 4]> {
    let p = star_poly();
    let mut out = Vec::new();
    for i in 0..p.len() {
        let (x0, y0) = p[i];
        let (x1, y1) = p[(i + 1) % p.len()];
        for k in 0..per_edge {
            let t0 = k as f64 / per_edge as f64;
            let t1 = (k + 1) as f64 / per_edge as f64;
            out.push([
                (x0 + (x1 - x0) * t0) as f32,
                (y0 + (y1 - y0) * t0) as f32,
                (x0 + (x1 - x0) * t1) as f32,
                (y0 + (y1 - y0) * t1) as f32,
            ]);
        }
    }
    out
}

/// A SILHUETA em segmentos, no espaço de texel — o que a wave da semeadura exata alimenta.
fn star_segments() -> Vec<[f32; 4]> {
    let p = star_poly();
    (0..p.len())
        .map(|i| {
            let (x0, y0) = p[i];
            let (x1, y1) = p[(i + 1) % p.len()];
            [x0 as f32, y0 as f32, x1 as f32, y1 as f32]
        })
        .collect()
}

/// A estrela em RGBA de alfa **RETO** — que é o que o Vello de facto entrega.
///
/// ⚠️ **`byte · cobertura` era o que esta sonda montava, e é uma fonte que o produto nunca
/// produz.** Foi por isso que ela não reproduzia o contorno tracejado do smoke sob condição
/// nenhuma: uma sonda cuja FONTE não é a do produto responde perguntas sobre outra coisa. Com
/// esta forma, o modo `PH2D_FX_VELLO=1` e o analítico concordam texel a texel na banda.
fn star_src(gpu: &ph2d_gpu::GpuContext) -> (wgpu::Texture, Vec<f32>) {
    let a = star_alpha(W, H);
    let mut bytes = vec![0u8; (W * H * 4) as usize];
    for (i, &cov) in a.iter().enumerate() {
        let o = i * 4;
        for c in 0..3 {
            bytes[o + c] = AMBER[c].round().clamp(0.0, 255.0) as u8;
        }
        bytes[o + 3] = (cov * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    (make_src(gpu, W, H, &bytes), a)
}

/// PPM P6 sobre um fundo cinza-escuro (o do app), para a foto ser lida como o artista a vê.
///
/// ⚠️ A saída do passe é RGBA **RETO** (o `cs_resolve` des-premultiplica), então o `over` é
/// `a·rgb + (1−a)·bg`. Compor como premultiplicado clareia toda borda parcial — e uma sonda que
/// mente na borda é uma sonda inútil justamente onde estes efeitos vivem.
///
/// ⚠️ **E o `over` corre em LUZ LINEAR**, porque é isso que o Vello faz com esta textura quando o
/// app a desenha. Compor em sRGB aqui produziria uma foto que ninguém vê — a sonda é um ORÁCULO DE
/// APARÊNCIA, então ela tem de errar exatamente onde o produto erra e acertar onde ele acerta. A
/// transferência vem do `ph2d_color`, a mesma que o resto do app usa: uma cópia local seria uma
/// segunda resposta a *o que é sRGB*.
fn write_ppm(dir: &str, name: &str, px: &[u8]) {
    use ph2d_color::srgb::{linear_to_srgb_byte, srgb_to_linear_byte};
    let bg = [0x2c_u8, 0x2e, 0x33];
    let mut body = Vec::with_capacity((W * H * 3) as usize);
    for i in 0..(W * H) as usize {
        let o = i * 4;
        let a = f32::from(px[o + 3]) / 255.0;
        for c in 0..3 {
            let lin = a * srgb_to_linear_byte(px[o + c]) + (1.0 - a) * srgb_to_linear_byte(bg[c]);
            body.push(linear_to_srgb_byte(lin));
        }
    }
    let path = format!("{dir}/{name}.ppm");
    let mut f = std::fs::File::create(&path).expect("criar ppm");
    use std::io::Write;
    write!(f, "P6\n{W} {H}\n255\n").expect("cabecalho");
    f.write_all(&body).expect("corpo");
    eprintln!("[fx-look] {path}");
}

fn op(kind: u8, sigma_px: f32, tint: [f32; 4], offset_px: [i32; 2]) -> FxOpGpu {
    FxOpGpu {
        kind,
        sigma_px,
        offset_px,
        tint,
        opacity: 1.0,
        mode: if FxOp::spec(kind).modes.is_empty() {
            0
        } else {
            FxOp::new(kind).mode
        },
        blend: 0,
        noise_scale_px: 0.0,
        detail: 1,
        seed: 0,
        grow_px: 0.0,
    }
}

#[test]
#[ignore = "sonda de olho; roda com --ignored e PH2D_FX_LOOK_DIR"]
fn probe_fx_render_and_look() {
    let Some(dir) = std::env::var("PH2D_FX_LOOK_DIR").ok() else {
        eprintln!("[fx-look] defina PH2D_FX_LOOK_DIR=<dir>");
        return;
    };
    std::fs::create_dir_all(&dir).expect("dir");
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[fx-look] sem adapter — skip");
        return;
    };
    let mut pass = FxStackPass::new(&gpu);
    // ⚠️ **Quem RASTERIZA importa.** A sonda nasceu com supersampling próprio, e o produto usa o
    // Vello — se as duas rampas de AA diferirem, o campo (que vem da GEOMETRIA) fica exato para uma
    // forma e o raster desenha outra, e a discordância lê como dente. `PH2D_FX_VELLO=1` põe a sonda
    // no rasterizador do produto para que a comparação seja honesta.
    let vello_scratch = if std::env::var("PH2D_FX_VELLO").is_ok() {
        let mut scratch =
            ph2d_render::VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (W, H))
                .expect("scratch");
        let mut shape = vello::Scene::new();
        let mut bp = vello::kurbo::BezPath::new();
        for (i, (x, y)) in star_poly().iter().enumerate() {
            if i == 0 {
                bp.move_to((*x, *y));
            } else {
                bp.line_to((*x, *y));
            }
        }
        bp.close_path();
        shape.fill(
            vello::peniko::Fill::NonZero,
            vello::kurbo::Affine::IDENTITY,
            vello::peniko::Color::from_rgba8(235, 175, 60, 255),
            None,
            &bp,
        );
        scratch
            .render_to_intermediate(
                &gpu,
                &shape,
                (W, H),
                vello::peniko::Color::TRANSPARENT,
                false,
            )
            .expect("scratch render");
        Some(scratch)
    } else {
        None
    };
    let (own, _cov) = star_src(&gpu);
    let src = vello_scratch
        .as_ref()
        .map_or(&own, |s| s.intermediate_texture());

    let white = [1.0, 1.0, 1.0, 1.0];
    let black = [0.0, 0.0, 0.0, 1.0];
    // ⚠️ A matriz do INNER SHADOW: os DOIS modos × com e sem deslocamento. Uma sombra interna é
    // exatamente a figura que um único print não decide — o defeito pode estar no modo, no
    // deslocamento ou no perfil, e só a matriz separa os três.
    let inner = |mode: u8, off: [i32; 2]| {
        let mut o = op(FxOp::INNER_SHADOW, 20.0, black, off);
        o.mode = mode;
        o
    };
    // Os DOIS modos do halo externo, lado a lado: o vão entre pontas da estrela é onde eles
    // discordam, e é a única figura que decide se a escolha vale a pena existir.
    let glow = |mode: u8| {
        let mut o = op(FxOp::GLOW, 20.0, white, [0, 0]);
        o.mode = mode;
        o
    };
    let scenes: [(&str, Vec<FxOpGpu>); 14] = [
        ("00_plain", vec![]),
        ("01_feather", vec![op(FxOp::FEATHER, 24.0, white, [0, 0])]),
        ("02_bevel", vec![op(FxOp::BEVEL, 20.0, black, [-12, 12])]),
        ("03_outline", vec![op(FxOp::OUTLINE, 8.0, white, [0, 0])]),
        (
            "04_inner_shadow",
            vec![op(FxOp::INNER_SHADOW, 20.0, black, [0, 0])],
        ),
        (
            "05_inner_prox_off0",
            vec![inner(FxOp::MODE_PROXIMITY, [0, 0])],
        ),
        (
            "06_inner_cont_off0",
            vec![inner(FxOp::MODE_CONTOUR, [0, 0])],
        ),
        (
            "07_inner_prox_off",
            vec![inner(FxOp::MODE_PROXIMITY, [18, -18])],
        ),
        (
            "08_inner_cont_off",
            vec![inner(FxOp::MODE_CONTOUR, [18, -18])],
        ),
        (
            "09_inner_glow",
            vec![op(FxOp::INNER_GLOW, 20.0, white, [0, 0])],
        ),
        ("10_glow_prox", vec![glow(FxOp::MODE_PROXIMITY)]),
        ("11_glow_cont", vec![glow(FxOp::MODE_CONTOUR)]),
        // ⚠️ O bevel do report do Enio cobre a forma INTEIRA — regime que a cena 02 (sigma 20)
        // nunca exercita. Um bevel largo faz a banda alcançar o EIXO MEDIAL, onde a distância
        // troca de aresta mais próxima; é o único lugar onde o campo exato tem descontinuidade.
        (
            "12_bevel_big",
            vec![op(FxOp::BEVEL, 90.0, black, [-12, 12])],
        ),
        (
            "13_bevel_huge",
            vec![op(FxOp::BEVEL, 200.0, black, [-12, 12])],
        ),
    ];
    // A FONTE, como o passe a ve^ — se a premultiplicacao dela nao valer, todo o resto herda.
    write_ppm(&dir, "0_src", &readback(&gpu, src, W, H));
    for (name, ops) in scenes {
        let dst = make_output_texture(&gpu, W, H);
        let dense = std::env::var("PH2D_FX_DENSE").is_ok();
        // ⚠️ `PH2D_FX_RASTER=1` NEGA a silhueta — o caminho do JFA sobre o raster, para o qual o
        // campo cai quando ninguém sabe responder pela geometria. Sem esta chave a sonda nunca o
        // desenha, e foi por isso que o pente do bevel viveu sem foto: ele **só existe ali**.
        //
        // Era onde TODA forma com traço caía, e é o que a wave da silhueta resolvida fechou — mas
        // o caminho continua vivo (forma complexa demais, silhueta que ninguém resolveu), então a
        // chave continua sendo a única maneira de o VER.
        let segs = if std::env::var("PH2D_FX_RASTER").is_ok() {
            Vec::new()
        } else if dense {
            star_segments_dense(15)
        } else {
            star_segments()
        };
        pass.run(&gpu, src, &dst, W, H, &ops, &segs);
        let px = readback(&gpu, &dst, W, H);
        write_ppm(&dir, name, &px);
    }
}
