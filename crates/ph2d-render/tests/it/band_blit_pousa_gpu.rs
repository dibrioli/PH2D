//! ⭐⭐ **A POUSA com modo** (ciclo 20 W4b do Motion, doc 129 §3.5): a colagem de uma faixa com
//! `Soma`/`Multiplica`/`Tela` pela mistura de HARDWARE pinta a lei ([`Pousa::lei`]) — com o fundo
//! meio transparente também (o 2.º passe do `Multiplica` lê o alfa ORIGINAL do fundo) — e a lei é a
//! que o Vello pinta hoje para a mistura do `Output` (o oráculo, sobre fundo opaco).
//!
//! Rodar: `PH2D_GPU=1 cargo test -p ph2d-render --test it band_blit_pousa -- --ignored`.

use ph2d_render::{BandBlit, BandSource, Pousa, VelloPass};
use vello::kurbo::{Affine, Rect};
use vello::peniko::{BlendMode, Color, Compose, Fill, Mix};

use crate::fx_stack_common::{make_src, readback, try_headless_gpu};

/// Fundos PRÉ-MULTIPLICADOS `(cor, alfa)` — opacos e meio transparentes.
const FUNDOS: [(f32, f32); 5] = [(0.0, 1.0), (0.25, 1.0), (0.6, 1.0), (1.0, 1.0), (0.2, 0.5)];
/// Faixas RETAS `(cor, alfa)`, a convenção do Vello (`BandSource::Vector`).
const FAIXAS: [(f32, f32); 6] = [
    (0.2, 1.0),
    (0.5, 1.0),
    (0.9, 1.0),
    (0.2, 0.3),
    (0.5, 0.6),
    (0.9, 0.3),
];
/// Largura de cada célula do oráculo do Vello (arestas inteiras: sem anti-serrilhado a ler).
const CEL: u32 = 8;

fn byte(x: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let v = (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    v
}

fn q(x: f32) -> f32 {
    f32::from(byte(x)) / 255.0
}

fn casos() -> Vec<((f32, f32), (f32, f32))> {
    FUNDOS
        .iter()
        .flat_map(|&f| FAIXAS.iter().map(move |&s| (f, s)))
        .collect()
}

/// Um alvo `Rgba8Unorm` de `w × 1` com o fundo de cada caso (cinzento, alfa no 4.º canal).
fn alvo(gpu: &ph2d_gpu::GpuContext, cs: &[((f32, f32), (f32, f32))]) -> wgpu::Texture {
    let w = cs.len() as u32;
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("test alvo da pousa"),
        size: wgpu::Extent3d {
            width: w,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let bytes: Vec<u8> = cs
        .iter()
        .flat_map(|&((c, a), _)| [byte(c), byte(c), byte(c), byte(a)])
        .collect();
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &t,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &bytes,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(w * 4),
            rows_per_image: Some(1),
        },
        wgpu::Extent3d {
            width: w,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    t
}

/// Cola a faixa de cada caso sobre o fundo dele, com `pousa`; devolve `[cinza, alfa]` por caso.
fn pousa_na_placa(
    gpu: &ph2d_gpu::GpuContext,
    blit: &BandBlit,
    cs: &[((f32, f32), (f32, f32))],
    pousa: Pousa,
) -> Vec<[u8; 2]> {
    let w = cs.len() as u32;
    let fonte: Vec<u8> = cs
        .iter()
        .flat_map(|&(_, (c, a))| [byte(c), byte(c), byte(c), byte(a)])
        .collect();
    let src = make_src(gpu, w, 1, &fonte);
    let dst = alvo(gpu, cs);
    let vista = |t: &wgpu::Texture| t.create_view(&wgpu::TextureViewDescriptor::default());
    blit.blit_pousa(
        gpu,
        &vista(&dst),
        &vista(&src),
        BandSource::Vector,
        None,
        pousa,
    );
    readback(gpu, &dst, w, 1)
        .chunks(4)
        .map(|p| [p[0], p[3]])
        .collect()
}

/// A lei sobre os valores QUANTIZADOS que a placa de facto recebe.
fn lei(pousa: Pousa, ((fc, fa), (sc, sa)): ((f32, f32), (f32, f32))) -> [u8; 2] {
    let (b, ab) = (q(fc), q(fa));
    let (c, a) = (q(sc), q(sa));
    let o = pousa.lei([b, b, b, ab], [c * a, c * a, c * a, a]);
    [byte(o[0]), byte(o[3])]
}

#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_hardware_blend_paints_the_law_of_each_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[band_blit_pousa] sem adapter — skip");
        return;
    };
    let blit = BandBlit::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
    let cs = casos();
    let mut leituras = Vec::new();
    for pousa in Pousa::ALL {
        let lido = pousa_na_placa(&gpu, &blit, &cs, pousa);
        let mut pior = 0;
        for (i, caso) in cs.iter().enumerate() {
            let alvo = lei(pousa, *caso);
            for k in 0..2 {
                let d = (i32::from(lido[i][k]) - i32::from(alvo[k])).abs();
                assert!(
                    d <= 1,
                    "{pousa:?}, fundo {:?} faixa {:?}: lido {:?} contra a lei {alvo:?}",
                    caso.0,
                    caso.1,
                    lido[i]
                );
                pior = pior.max(d);
            }
        }
        eprintln!("[band_blit_pousa] {pousa:?}: pior {pior} nível contra a lei");
        leituras.push(lido);
    }
    // O CONTROLO: na fixtura os quatro modos separam-se, também no fundo MEIO TRANSPARENTE (onde
    // um `Multiplica` de um passe só — o que ignora o alfa do fundo — erraria).
    for i in 0..4 {
        for j in (i + 1)..4 {
            let separa = (0..cs.len())
                .map(|k| (i32::from(leituras[i][k][0]) - i32::from(leituras[j][k][0])).abs())
                .max()
                .unwrap_or(0);
            assert!(
                separa > 8,
                "{:?} e {:?} só se separam por {separa} níveis — a fixtura não os distingue",
                Pousa::ALL[i],
                Pousa::ALL[j]
            );
        }
    }
    // O de um passe só esquece o termo `s·(1 − αb)`: o caso que mais o pesa.
    let ingenuo = |((fc, _), (sc, sa)): ((f32, f32), (f32, f32))| {
        let (b, c, a) = (q(fc), q(sc), q(sa));
        byte(c * a * b + b * (1.0 - a))
    };
    let (i, caso) = cs
        .iter()
        .enumerate()
        .filter(|(_, c)| c.0.1 < 1.0)
        .max_by(|(_, x), (_, y)| (x.1.0 * x.1.1).total_cmp(&(y.1.0 * y.1.1)))
        .expect("um caso de fundo meio transparente");
    let m = Pousa::ALL.iter().position(|p| *p == Pousa::Multiplica).unwrap_or(0);
    assert!(
        (i32::from(leituras[m][i][0]) - i32::from(ingenuo(*caso))).abs() > 8,
        "a fixtura não separa o `Multiplica` exacto do de um passe só"
    );
}

/// O par do Vello que a mistura do `Output` usa hoje (`ph2d_vec_render::blend::vello_blend`).
fn do_vello(pousa: Pousa) -> BlendMode {
    match pousa {
        Pousa::Normal => BlendMode::new(Mix::Normal, Compose::SrcOver),
        Pousa::Soma => BlendMode::new(Mix::Normal, Compose::Plus),
        Pousa::Multiplica => BlendMode::new(Mix::Multiply, Compose::SrcOver),
        Pousa::Tela => BlendMode::new(Mix::Screen, Compose::SrcOver),
    }
}

/// ⭐ **O ORÁCULO**: a mesma faixa sobre o mesmo fundo OPACO, pelo Vello com o modo da camada.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_hardware_blend_is_what_vello_paints_for_the_same_mode() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("[band_blit_pousa] sem adapter — skip");
        return;
    };
    let cs: Vec<_> = casos().into_iter().filter(|c| c.0.1 >= 1.0).collect();
    let (w, h) = (cs.len() as u32 * CEL, CEL);
    let Ok(mut vello) = VelloPass::new(&gpu, wgpu::TextureFormat::Bgra8UnormSrgb, (w, h)) else {
        eprintln!("[band_blit_pousa] sem VelloPass — skip");
        return;
    };
    let blit = BandBlit::new(&gpu, wgpu::TextureFormat::Rgba8Unorm);
    for pousa in Pousa::ALL {
        let mut cena = vello::Scene::new();
        for (i, &((fc, _), (sc, sa))) in cs.iter().enumerate() {
            let x = f64::from(i as u32 * CEL);
            let r = Rect::new(x, 0.0, x + f64::from(CEL), f64::from(h));
            let cinza = |c: f32, a: f32| Color::from_rgba8(byte(c), byte(c), byte(c), byte(a));
            cena.fill(Fill::NonZero, Affine::IDENTITY, cinza(fc, 1.0), None, &r);
            cena.push_layer(Fill::NonZero, do_vello(pousa), 1.0, Affine::IDENTITY, &r);
            cena.fill(Fill::NonZero, Affine::IDENTITY, cinza(sc, sa), None, &r);
            cena.pop_layer();
        }
        let px = vello
            .render_and_readback(&gpu, &cena, (w, h))
            .expect("o readback do Vello");
        let placa = pousa_na_placa(&gpu, &blit, &cs, pousa);
        let mut pior = 0;
        for (i, caso) in cs.iter().enumerate() {
            let centro = ((h / 2 * w + i as u32 * CEL + CEL / 2) * 4) as usize;
            let d = (i32::from(px[centro]) - i32::from(placa[i][0])).abs();
            assert!(
                d <= 1,
                "{pousa:?}, fundo {:?} faixa {:?}: o Vello pinta {} e a pousa {}",
                caso.0,
                caso.1,
                px[centro],
                placa[i][0]
            );
            pior = pior.max(d);
        }
        eprintln!("[band_blit_pousa] {pousa:?}: pior {pior} nível contra o Vello");
    }
}

/// A lei na CPU: o `Normal` é o `over`, e cada modo com faixa TRANSPARENTE devolve o fundo.
#[test]
fn the_law_is_over_for_normal_and_identity_for_a_clear_band() {
    let b = [0.3, 0.3, 0.3, 0.7];
    for pousa in Pousa::ALL {
        assert_eq!(pousa.lei(b, [0.0; 4]), b, "{pousa:?} com faixa vazia");
    }
    let s = [0.2, 0.2, 0.2, 0.4];
    let over = Pousa::Normal.lei(b, s);
    assert!((over[0] - (0.2 + 0.3 * 0.6)).abs() < 1e-6);
    assert!((over[3] - (0.4 + 0.7 * 0.6)).abs() < 1e-6);
}
