//! ⭐⭐⭐ **A PARIDADE DE PIXEL contra o Vello** — as MESMAS cópias, rasterizadas pelo desenhador de
//! hoje (o Vello, `AaConfig::Area`) e pelo passe instanciado, comparadas pixel a pixel.
//!
//! ⚠️ **As duas saídas vivem em espaços de alfa diferentes, e a comparação converte:** o Vello grava
//! a cor SEPARADA do alfa (`fine.wgsl`: `rgba_sep = fg.rgb / fg.a`), e o passe grava-a
//! pré-multiplicada (é o que a mistura por hardware sabe compor). A régua compara o ALFA em todo
//! pixel e a cor SEPARADA onde o alfa é grande o bastante para ela significar alguma coisa num
//! byte (`≥ 64`): abaixo disso um pixel pré-multiplicado de 8 bits já não guarda a cor.
//!
//! ⚠️ **O que as duas rotas fazem de DIFERENTE, e é por isso que a barra não é zero:** o Vello
//! aplana as curvas NO ECRÃ (tolerância `0,25 px`) e o passe aplana-as no espaço LOCAL por níveis
//! (erro no ecrã `≤ 0,25 px`); o traço, o Vello expande-o na placa e o passe no kurbo. Os
//! segmentos não são os mesmos, logo um pixel de borda pode diferir.
//!
//! ```text
//! cargo test -p ph2d-shape-gpu --test it -- --ignored --nocapture
//! ```

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{
    Copias, FillRule, ShapeGeometry, ShapeInput, ShapeInstance, ShapePass, ShapeView, StrokeInput,
};
use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Join, Shape, Stroke};

const LADO: u32 = 512;

fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Uma estrela de cinco pontas como polígono (a `source.shape` de fábrica), em unidades de
/// extensão `1`.
fn estrela() -> BezPath {
    let mut bp = BezPath::new();
    for i in 0..10 {
        let r = if i % 2 == 0 { 0.5 } else { 0.2 };
        let a = std::f64::consts::PI * (f64::from(i) / 5.0) - std::f64::consts::FRAC_PI_2;
        let p = (r * a.cos(), r * a.sin());
        if i == 0 {
            bp.move_to(p);
        } else {
            bp.line_to(p);
        }
    }
    bp.close_path();
    bp
}

/// Um pentagrama AUTO-INTERSECTADO — o caso em que as duas regras de preenchimento discordam.
fn pentagrama() -> BezPath {
    let mut bp = BezPath::new();
    for i in 0..5 {
        let a = std::f64::consts::PI * 0.8 * f64::from(i) - std::f64::consts::FRAC_PI_2;
        let p = (0.5 * a.cos(), 0.5 * a.sin());
        if i == 0 {
            bp.move_to(p);
        } else {
            bp.line_to(p);
        }
    }
    bp.close_path();
    bp
}

fn circulo() -> BezPath {
    Circle::new((0.0, 0.0), 0.5).to_path(0.1)
}

/// Uma cópia: posição, lado, ângulo e cor.
struct Copia {
    pos: [f32; 2],
    lado: f32,
    ang: f32,
    tint: [f32; 4],
}

/// Uma GRELHA de cópias opacas que não se tocam, de lados crescentes — a fixtura em que o desvio
/// de ALFA é exactamente o desvio de COBERTURA (sem sobreposição, o alfa de um pixel é a área dele
/// coberta pela única cópia que lá passa).
fn grelha_isolada() -> Vec<Copia> {
    let mut v = Vec::new();
    let passo = 64.0f32;
    for j in 0..8 {
        for i in 0..8 {
            #[expect(clippy::cast_precision_loss, reason = "oito por oito")]
            let (x, y) = (i as f32, j as f32);
            v.push(Copia {
                pos: [passo * (x + 0.5) + 0.37 * y, passo * (y + 0.5) + 0.21 * x],
                lado: 8.0 + 6.9 * (x + 8.0 * y) * 0.125,
                ang: 0.29 * (x + 8.0 * y),
                tint: [0.2, 0.4, 0.8, 1.0],
            });
        }
    }
    v
}

/// Um gerador determinista (as cópias não podem mudar entre corridas).
fn copias(n: usize, lado_min: f32, lado_max: f32, semente: u64) -> Vec<Copia> {
    let mut s = semente;
    let mut r = || {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        #[expect(clippy::cast_precision_loss, reason = "um gerador de fixturas")]
        let v = (s >> 40) as f32 / (1u64 << 24) as f32;
        v
    };
    (0..n)
        .map(|_| {
            #[expect(clippy::cast_precision_loss, reason = "LADO é 512")]
            let l = LADO as f32;
            Copia {
                pos: [r() * l, r() * l],
                lado: lado_min + r() * (lado_max - lado_min),
                ang: r() * std::f32::consts::TAU,
                tint: [r(), r(), r(), 0.35 + 0.65 * r()],
            }
        })
        .collect()
}

fn basis(ang: f32) -> [f32; 4] {
    let (s, c) = ang.sin_cos();
    [c, s, -s, c]
}

/// A pose de mundo de uma cópia, EXACTAMENTE como o shader a compõe (`pos + basis·(q·lado)`).
fn pose(c: &Copia) -> Affine {
    let [b0, b1, b2, b3] = basis(c.ang);
    Affine::new([
        f64::from(b0 * c.lado),
        f64::from(b1 * c.lado),
        f64::from(b2 * c.lado),
        f64::from(b3 * c.lado),
        f64::from(c.pos[0]),
        f64::from(c.pos[1]),
    ])
}

struct Forma<'a> {
    bp: &'a BezPath,
    regra: FillRule,
    traco: Option<(Stroke, [f32; 4])>,
}

/// Os bytes crus de uma textura (`px` bytes por pixel).
fn bytes_de_textura(gpu: &GpuContext, tex: &wgpu::Texture, px: u32) -> Vec<u8> {
    let bpr = LADO * px;
    let buf = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leitura"),
        size: u64::from(bpr * LADO),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    enc.copy_texture_to_buffer(
        tex.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buf,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bpr),
                rows_per_image: Some(LADO),
            },
        },
        wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    buf.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = buf.slice(..).get_mapped_range().to_vec();
    buf.unmap();
    v
}

fn textura(
    gpu: &GpuContext,
    usage: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("alvo"),
        size: wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: usage | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    })
}

/// O Vello: cada cópia um `fill` (e um `stroke`), pela ordem.
fn pelo_vello(gpu: &GpuContext, forma: &Forma<'_>, cs: &[Copia]) -> Vec<u8> {
    let mut cena = vello::Scene::new();
    let regra = match forma.regra {
        FillRule::NonZero => Fill::NonZero,
        FillRule::EvenOdd => Fill::EvenOdd,
    };
    for c in cs {
        let t = pose(c);
        cena.fill(regra, t, &Brush::Solid(Color::new(c.tint)), None, forma.bp);
        if let Some((s, cor)) = &forma.traco {
            cena.stroke(s, t, &Brush::Solid(Color::new(*cor)), None, forma.bp);
        }
    }
    let mut r = vello::Renderer::new(
        &gpu.device,
        vello::RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::area_only(),
            num_init_threads: None,
            pipeline_cache: None,
        },
    )
    .expect("o Vello cria o renderer");
    let tex = textura(
        gpu,
        wgpu::TextureUsages::STORAGE_BINDING,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
    r.render_to_texture(
        &gpu.device,
        &gpu.queue,
        &cena,
        &vista,
        &vello::RenderParams {
            base_color: Color::TRANSPARENT,
            width: LADO,
            height: LADO,
            antialiasing_method: vello::AaConfig::Area,
        },
    )
    .expect("o Vello desenha");
    bytes_de_textura(gpu, &tex, 4)
}

/// Meio-flutuante → `f32` (a leitura de um alvo `Rgba16Float`, sem trazer a crate `half`).
fn f16(b: u16) -> f32 {
    let sinal = if b & 0x8000 != 0 { -1.0 } else { 1.0 };
    let exp = i32::from((b >> 10) & 0x1f);
    let man = f32::from(b & 0x3ff);
    sinal
        * match exp {
            0 => man * 2f32.powi(-24),
            31 => f32::INFINITY,
            e => (1.0 + man / 1024.0) * 2f32.powi(e - 15),
        }
}

/// A cor SEPARADA de um pixel pré-multiplicado, quantizada como o Vello a quantiza.
fn separa(c: [f32; 4]) -> [u8; 4] {
    let q = |v: f32| {
        #[expect(clippy::cast_possible_truncation, reason = "já limitado a um byte")]
        let b = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        b
    };
    let a = c[3];
    if a <= 0.0 {
        return [0, 0, 0, 0];
    }
    [q(c[0] / a), q(c[1] / a), q(c[2] / a), q(a)]
}

/// O passe: uma geometria, as cópias, uma chamada — sobre um alvo de `format`, devolvido com a
/// cor SEPARADA, como o Vello a grava.
fn pelo_passe(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    cs: &[Copia],
    format: wgpu::TextureFormat,
) -> Vec<u8> {
    let traco = forma.traco.as_ref().map(|(s, cor)| StrokeInput {
        path: forma.bp,
        style: s,
        color: *cor,
    });
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((forma.bp, forma.regra)),
        strokes: traco.into_iter().collect(),
        stroke_fills: Vec::new(),
    })
    .expect("a forma prepara");
    let mut p = ShapePass::new(gpu, format);
    p.set_geometries(gpu, [(7u32, &g)]);
    let insts: Vec<ShapeInstance> = cs
        .iter()
        .map(|c| ShapeInstance {
            pos: c.pos,
            size: [c.lado, c.lado],
            basis: basis(c.ang),
            anchor: [0.0, 0.0],
            geometry: 7,
            _pad: 0,
            tint: c.tint,
        })
        .collect();
    p.upload_instances(gpu, &insts);
    let tex = textura(gpu, wgpu::TextureUsages::RENDER_ATTACHMENT, format);
    let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    #[expect(clippy::cast_precision_loss, reason = "LADO é 512")]
    let alvo = [LADO as f32, LADO as f32];
    p.draw(
        gpu,
        &mut enc,
        &vista,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ShapeView {
            lin: [1.0, 0.0, 0.0, 1.0],
            t: [0.0, 0.0],
            alvo,
        },
        Copias {
            buffer: p.uploaded().expect("carregou"),
            count: u32::try_from(insts.len()).expect("cabem"),
        },
    );
    gpu.queue.submit(Some(enc.finish()));
    if format == wgpu::TextureFormat::Rgba16Float {
        let b = bytes_de_textura(gpu, &tex, 8);
        b.as_chunks::<8>().0.iter()
            .flat_map(|px| {
                let h = |i: usize| f16(u16::from_le_bytes([px[2 * i], px[2 * i + 1]]));
                separa([h(0), h(1), h(2), h(3)])
            })
            .collect()
    } else {
        bytes_de_textura(gpu, &tex, 4)
            .as_chunks::<4>().0.iter()
            .flat_map(|px| {
                let f = |i: usize| f32::from(px[i]) / 255.0;
                separa([f(0), f(1), f(2), f(3)])
            })
            .collect()
    }
}

/// O que a comparação devolve: o pior desvio de alfa, o pior de cor (separada, onde `α ≥ 64`), e
/// quantos pixels desviam mais de `1` no alfa.
#[derive(Debug)]
struct Desvio {
    alfa_max: u8,
    cor_max: u8,
    alfa_acima_de_1: usize,
    /// Pixels em que a cor separada desvia mais de `4` num canal.
    cor_acima_de_4: usize,
    /// O pixel do pior desvio de cor, e os dois valores lá (Vello, passe).
    pior_cor: (usize, [u8; 4], [u8; 4]),
    pixels_com_tinta: usize,
}

fn compara(vello: &[u8], passe: &[u8]) -> Desvio {
    let mut d = Desvio {
        alfa_max: 0,
        cor_max: 0,
        alfa_acima_de_1: 0,
        cor_acima_de_4: 0,
        pior_cor: (0, [0; 4], [0; 4]),
        pixels_com_tinta: 0,
    };
    for (i, (v, p)) in vello.as_chunks::<4>().0.iter().zip(passe.as_chunks::<4>().0.iter()).enumerate() {
        let da = v[3].abs_diff(p[3]);
        d.alfa_max = d.alfa_max.max(da);
        if da > 1 {
            d.alfa_acima_de_1 += 1;
        }
        if v[3] > 0 || p[3] > 0 {
            d.pixels_com_tinta += 1;
        }
        if v[3] >= 64 && p[3] >= 64 {
            let pior = (0..3).map(|k| v[k].abs_diff(p[k])).max().unwrap_or(0);
            if pior > d.cor_max {
                d.cor_max = pior;
                d.pior_cor = (i, [v[0], v[1], v[2], v[3]], [p[0], p[1], p[2], p[3]]);
            }
            if pior > 4 {
                d.cor_acima_de_4 += 1;
            }
        }
    }
    d
}

/// Grava as duas imagens (`PH2D_PARIDADE_DIR`) para o olho — um desvio grande lê-se melhor numa
/// imagem do que num número.
fn grava(dir: &std::path::Path, nome: &str, rgba: &[u8]) {
    let mut ppm = format!("P6\n{LADO} {LADO}\n255\n").into_bytes();
    for px in rgba.as_chunks::<4>().0.iter() {
        // Sobre branco, com o alfa separado (o Vello) ou pré-multiplicado (o passe) já em `px`.
        ppm.extend_from_slice(&px[..3]);
    }
    let _ = std::fs::write(dir.join(format!("{nome}.ppm")), ppm);
}

fn corre(nome: &str, forma: &Forma<'_>, cs: &[Copia]) -> Option<Desvio> {
    let gpu = gpu()?;
    let v = pelo_vello(&gpu, forma, cs);
    let p8 = pelo_passe(&gpu, forma, cs, wgpu::TextureFormat::Rgba8Unorm);
    eprintln!("  {nome:<28} alvo de 8 bits: {:?}", compara(&v, &p8));
    let p = pelo_passe(&gpu, forma, cs, wgpu::TextureFormat::Rgba16Float);
    if let Ok(dir) = std::env::var("PH2D_PARIDADE_DIR") {
        let dir = std::path::Path::new(&dir);
        let chave = nome.replace(' ', "_");
        grava(dir, &format!("{chave}_vello"), &v);
        grava(dir, &format!("{chave}_passe"), &p);
    }
    let conta = |x: &[u8]| x.as_chunks::<4>().0.iter().filter(|p| p[3] > 0).count();
    eprintln!(
        "  {nome:<28} vello pinta {} px · passe pinta {} px",
        conta(&v),
        conta(&p)
    );
    let d = compara(&v, &p);
    eprintln!("  {nome:<28} alvo de meio-float: {d:?}");
    Some(d)
}

/// **As barras, por família — cada uma saída de um VALE MEDIDO** (2026-09-29, RTX, alvo de
/// meio-float, que é o do produto).
///
/// ⭐ **Polígonos** (estrelas, pentagrama): as arestas são as MESMAS nas duas rotas, e o que sobra é
/// arredondamento — alfa `1`, cor `2`–`3`. Barra `2` / `4`.
///
/// ⭐⭐ **Curvas** (círculos): as duas rotas aplanam a curva de maneira diferente, cada uma dentro de
/// `0,25 px` dela, logo a borda difere até uns `0,25 px` de cobertura. Varrida a tolerância do passe
/// sobre os círculos ISOLADOS (onde o alfa é a cobertura, sem sobreposição):
///
/// | tolerância do passe (px) | alfa máx. | pixels a mais que o Vello |
/// |---:|---:|---:|
/// | `1/64` | `77` | `+952` |
/// | `1/16` | `71` | `+828` |
/// | **`1/4` (a do Vello, a que shipa)** | **`65`** | `+329` |
/// | `1/2` | `69` | `+305` |
/// | `1` (grosso demais) | **`163`** | `+99` |
///
/// ⚠️⚠️ **Mais fino afasta do Vello, e não aproxima:** o Vello deixa a curva até `0,25 px` para
/// DENTRO (as cordas de um convexo ficam do lado de dentro), e um passe mais fino converge para a
/// curva VERDADEIRA — a contagem de pixels pintados sobe com a finura. ⇒ o passe shipa a MESMA
/// tolerância do Vello (a nitidez que o dono já aprovou), e a barra `100` separa todo aplanamento
/// nítido (`≤ 77`) do grosso demais (`163`). Na cena sobreposta a cor do pior pixel de borda lê
/// `37`–`39` nítido e `80` grosso ⇒ barra `60`.
///
/// ⚠️ **Traço:** a junta em esquadria expandida pelo kurbo e pela placa do Vello difere em `25`
/// pixels de `178 656`, alfa `17`. Barra `32`.
fn barra(nome: &str) -> (u8, u8) {
    match nome {
        "circulos (curvas)" => (4, 60),
        "circulos isolados" => (100, 4),
        "estrela com traco" => (32, 8),
        _ => (2, 4),
    }
}

/// ⭐⭐⭐ **As famílias do passe, contra o Vello, pixel a pixel** — e a mesma área pintada.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_passe_desenha_o_que_o_vello_desenha() {
    let (est, pent, circ) = (estrela(), pentagrama(), circulo());
    let casos: Vec<(&str, Forma<'_>, Vec<Copia>)> = vec![
        (
            "estrelas pequenas",
            Forma {
                bp: &est,
                regra: FillRule::NonZero,
                traco: None,
            },
            copias(400, 6.0, 40.0, 1),
        ),
        (
            "estrelas grandes",
            Forma {
                bp: &est,
                regra: FillRule::NonZero,
                traco: None,
            },
            copias(12, 120.0, 900.0, 2),
        ),
        (
            "circulos (curvas)",
            Forma {
                bp: &circ,
                regra: FillRule::NonZero,
                traco: None,
            },
            copias(200, 8.0, 300.0, 3),
        ),
        (
            "circulos isolados",
            Forma {
                bp: &circ,
                regra: FillRule::NonZero,
                traco: None,
            },
            grelha_isolada(),
        ),
        (
            "pentagrama even-odd",
            Forma {
                bp: &pent,
                regra: FillRule::EvenOdd,
                traco: None,
            },
            copias(40, 30.0, 200.0, 4),
        ),
        (
            "estrela com traco",
            Forma {
                bp: &est,
                regra: FillRule::NonZero,
                traco: Some((
                    Stroke::new(0.06).with_join(Join::Miter),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            copias(40, 40.0, 220.0, 5),
        ),
    ];
    let mut algum = false;
    for (nome, forma, cs) in &casos {
        let Some(d) = corre(nome, forma, cs) else {
            eprintln!("sem adaptador — nada a medir");
            return;
        };
        algum = true;
        assert!(
            d.pixels_com_tinta > 1000,
            "{nome}: a fixtura quase nao desenha"
        );
        let (alfa, cor) = barra(nome);
        assert!(
            d.alfa_max <= alfa && d.cor_max <= cor,
            "{nome}: o passe desenha outra coisa que o Vello (barra alfa {alfa}, cor {cor}): {d:?}"
        );
    }
    assert!(algum);
}
