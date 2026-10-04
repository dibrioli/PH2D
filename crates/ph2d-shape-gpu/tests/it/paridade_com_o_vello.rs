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
use ph2d_vector::{Affine, BezPath, Brush, Cap, Circle, Color, Fill, Join, Shape, Stroke};

pub(super) const LADO: u32 = 512;

pub(super) fn gpu() -> Option<GpuContext> {
    GpuContext::new(GpuContext::default_instance(), None).ok()
}

/// Uma estrela de cinco pontas como polígono (a `source.shape` de fábrica), em unidades de
/// extensão `1`.
pub(super) fn estrela() -> BezPath {
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

/// Um caminho ABERTO com quinas nos dois sentidos — pontas e juntas.
pub(super) fn zigue_zague() -> BezPath {
    let mut bp = BezPath::new();
    bp.move_to((-0.5, 0.2));
    bp.line_to((-0.2, -0.3));
    bp.line_to((0.0, 0.25));
    bp.line_to((0.25, -0.25));
    bp.line_to((0.5, 0.3));
    bp
}

pub(super) fn circulo() -> BezPath {
    Circle::new((0.0, 0.0), 0.5).to_path(0.1)
}

/// Um ANEL: dois círculos no mesmo caminho, lidos pela regra even-odd — o preenchimento tem DUAS
/// correntes. ⚠️ doc 121 §9.3: os blocos de segmentos somam um bloco todo à esquerda do pixel pelas
/// PONTAS da corrente, o que só vale se ela não partir dentro do bloco; o anel é a forma que parte.
pub(super) fn anel() -> BezPath {
    let mut bp = Circle::new((0.0, 0.0), 0.5).to_path(0.1);
    for el in Circle::new((0.0, 0.0), 0.28).to_path(0.1).elements() {
        bp.push(*el);
    }
    bp
}

/// Uma cópia: posição, lado, ângulo e cor — e o ASPECTO (`altura / largura`), que é `1` numa cópia
/// conforme e outra coisa sob escala NÃO uniforme (doc 121 W4).
pub(super) struct Copia {
    pub(super) pos: [f32; 2],
    pub(super) lado: f32,
    pub(super) ang: f32,
    pub(super) tint: [f32; 4],
    pub(super) aspecto: f32,
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
                aspecto: 1.0,
            });
        }
    }
    v
}

/// Um gerador determinista (as cópias não podem mudar entre corridas).
pub(super) fn copias(n: usize, lado_min: f32, lado_max: f32, semente: u64) -> Vec<Copia> {
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
                aspecto: 1.0,
            }
        })
        .collect()
}

/// As mesmas cópias, ESTICADAS: o aspecto de cada uma entre `0,35` e `2,8`, nos dois sentidos.
pub(super) fn esticadas(n: usize, lado_min: f32, lado_max: f32, semente: u64) -> Vec<Copia> {
    let mut v = copias(n, lado_min, lado_max, semente);
    for (i, c) in v.iter_mut().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "uma fixtura pequena")]
        let k = (i as f32 * 0.618_034).fract();
        c.aspecto = 0.35 + 2.45 * k;
    }
    v
}

pub(super) fn basis(ang: f32) -> [f32; 4] {
    let (s, c) = ang.sin_cos();
    [c, s, -s, c]
}

/// A pose de mundo de uma cópia, EXACTAMENTE como o shader a compõe (`pos + basis·(q·size)`).
fn pose(c: &Copia) -> Affine {
    let [b0, b1, b2, b3] = basis(c.ang);
    let (sx, sy) = (c.lado, c.lado * c.aspecto);
    Affine::new([
        f64::from(b0 * sx),
        f64::from(b1 * sx),
        f64::from(b2 * sy),
        f64::from(b3 * sy),
        f64::from(c.pos[0]),
        f64::from(c.pos[1]),
    ])
}

pub(super) struct Forma<'a> {
    pub(super) bp: &'a BezPath,
    pub(super) regra: FillRule,
    pub(super) traco: Option<(Stroke, [f32; 4])>,
    /// A linha que o traço segue, quando não é o contorno do preenchimento (um caminho ABERTO).
    pub(super) linha: Option<&'a BezPath>,
    /// As MARCAS do traço: um preenchimento com a cor dele (`ShapeInput::stroke_fills`).
    pub(super) marcas: Option<&'a BezPath>,
}

/// Os bytes crus de uma textura (`px` bytes por pixel).
pub(super) fn bytes_de_textura(gpu: &GpuContext, tex: &wgpu::Texture, px: u32) -> Vec<u8> {
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

pub(super) fn textura(
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
            let linha = forma.linha.unwrap_or(forma.bp);
            // ⭐ A LEI DA CASA sob escala não uniforme (`ph2d_vec_render::stroke_uniform`, bug #27),
            // CHAMADA: a geometria atravessa o afim, a caneta não — REDONDA, de largura `w·√|det|`,
            // o tracejado escala com ela e ajusta-se ao contorno do ecrã (doc 121 §9.9).
            match ph2d_vec_render::pen_for(s, t) {
                (None, xf) => cena.stroke(s, xf, &Brush::Solid(Color::new(*cor)), None, linha),
                (Some(mut pen), xf) => {
                    let tela = t * linha.clone();
                    ph2d_vec_render::ajusta_no_ecra(&mut pen, &tela);
                    cena.stroke(&pen, xf, &Brush::Solid(Color::new(*cor)), None, &tela);
                }
            }
        }
    }
    // ⛔ O `VelloPass` do PRODUTO e não um `vello::Renderer` montado aqui: a escolha do
    // anti-aliasing pertence a quem possui o renderer (`ph2d-render`), e um gate que a nomeasse
    // mediria contra uma escolha que o produto pode deixar de fazer (gate
    // `the_pass_aa_is_never_chosen_by_a_text_preference`, que apanhou a 1.ª redacção no fecho).
    let mut r = ph2d_render::VelloPass::new(gpu, wgpu::TextureFormat::Rgba8Unorm, (LADO, LADO))
        .expect("o VelloPass do produto nasce");
    r.render_and_readback(gpu, &cena, (LADO, LADO))
        .expect("o Vello desenha")
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
pub(super) fn separa(c: [f32; 4]) -> [u8; 4] {
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
pub(super) fn pelo_passe(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    cs: &[Copia],
    format: wgpu::TextureFormat,
) -> Vec<u8> {
    // ⭐ doc 121 §9.6: o regime do PRODUTO — a capacidade já cresceu para o total medido e TODAS as
    // cópias têm as arestas no ecrã. O 1.º quadro (a mistura com o caminho de sempre) é medido pelo
    // gate `contorno_calculado`, par contra par.
    let (img, com, (pedido, cap)) = pelo_passe_celulas(
        gpu,
        forma,
        cs,
        format,
        true,
        QUADROS_DO_PRODUTO,
        0.0,
        u64::MAX,
    )
    .pop()
    .expect("um quadro");
    assert_eq!(
        com as usize,
        cs.len(),
        "o passe nao chegou ao regime do produto: nem todas as copias tem as arestas no ecra"
    );
    assert!(
        pedido <= cap,
        "o passe nao chegou ao regime do produto: as celulas pediram {pedido} com capacidade {cap}"
    );
    img
}

/// Quadros até ao regime do produto: os totais das arestas e das células são copiados no 1.º,
/// mapeados no 2.º e colhidos no 3.º, que já desenha com a capacidade nova (`contorno.rs`, doc 121
/// §9.12 — o «5.º» das listas do §9.8 saiu com elas).
const QUADROS_DO_PRODUTO: usize = 3;

/// O passe com o CONTORNO CALCULADO ligado ou não (doc 121 §9.5), durante `quadros` quadros do
/// MESMO passe: por quadro, a imagem e quantas cópias ganharam o contorno (lido de volta — as duas
/// imagens são quase iguais, logo nenhuma régua de pixel distingue o caminho que correu).
///
/// ⚠️ **Quadros e não um:** a capacidade das arestas cresce para o total MEDIDO, lido de volta dois
/// quadros depois (`contorno.rs`); no 1.º quadro uma cópia que não cabe cai no caminho de sempre.
/// Cada quadro é submetido e esperado antes do seguinte — é o que o `draw` exige.
pub(super) fn pelo_passe_com(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    cs: &[Copia],
    format: wgpu::TextureFormat,
    contorno: bool,
    quadros: usize,
) -> Vec<(Vec<u8>, u32)> {
    // ⚠️ doc 121 §9.6: a régua de PIXEL mede o caminho novo em TODAS as cópias — também nas
    // conformes pequenas, que o produto deixa no caminho de sempre (a rota tem gate próprio).
    pelo_passe_rota(gpu, forma, cs, format, contorno, quadros, 0.0)
}

/// O passe com a área mínima de uma cópia CONFORME para ir pelas arestas no ecrã (doc 121 §9.6).
pub(super) fn pelo_passe_rota(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    cs: &[Copia],
    format: wgpu::TextureFormat,
    contorno: bool,
    quadros: usize,
    area_minima_conforme: f32,
) -> Vec<(Vec<u8>, u32)> {
    pelo_passe_celulas(
        gpu,
        forma,
        cs,
        format,
        contorno,
        quadros,
        area_minima_conforme,
        u64::MAX,
    )
    .into_iter()
    .map(|(img, com, _)| (img, com))
    .collect()
}

/// O passe com um TECTO nas células (doc 121 §9.12): por quadro, a imagem, quantas cópias ganharam o
/// contorno e `(células pedidas, capacidade delas)` — pedido acima da capacidade ⇒ alguma cópia foi
/// desenhada pelo caminho de sempre.
#[expect(
    clippy::too_many_arguments,
    reason = "as portas do passe, uma por régua"
)]
pub(super) fn pelo_passe_celulas(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    cs: &[Copia],
    format: wgpu::TextureFormat,
    contorno: bool,
    quadros: usize,
    area_minima_conforme: f32,
    celulas_no_maximo: u64,
) -> Vec<(Vec<u8>, u32, (u64, u64))> {
    pelo_passe_em_etapas(
        gpu,
        forma,
        &[(cs, quadros)],
        format,
        contorno,
        area_minima_conforme,
        celulas_no_maximo,
    )
}

/// O MESMO passe por várias ETAPAS — cada uma um conjunto de cópias desenhado durante uns quadros —,
/// como numa cena animada, onde as cópias mudam de quadro para quadro e os buffers do passe ficam
/// com o que o quadro anterior lá escreveu. Por quadro, o mesmo que [`pelo_passe_celulas`].
pub(super) fn pelo_passe_em_etapas(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    etapas: &[(&[Copia], usize)],
    format: wgpu::TextureFormat,
    contorno: bool,
    area_minima_conforme: f32,
    celulas_no_maximo: u64,
) -> Vec<(Vec<u8>, u32, (u64, u64))> {
    pelo_passe_observado(
        gpu,
        forma,
        etapas,
        format,
        (contorno, area_minima_conforme, celulas_no_maximo),
        &mut |_, _| {},
    )
}

/// O mesmo, com `observa` chamado sobre o passe depois de cada quadro (os instrumentos dele).
pub(super) fn pelo_passe_observado(
    gpu: &GpuContext,
    forma: &Forma<'_>,
    etapas: &[(&[Copia], usize)],
    format: wgpu::TextureFormat,
    (contorno, area_minima_conforme, celulas_no_maximo): (bool, f32, u64),
    observa: &mut dyn FnMut(&GpuContext, &ShapePass),
) -> Vec<(Vec<u8>, u32, (u64, u64))> {
    let traco = forma.traco.as_ref().map(|(s, cor)| StrokeInput {
        path: forma.linha.unwrap_or(forma.bp),
        style: s,
        color: *cor,
    });
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((forma.bp, forma.regra)),
        strokes: traco.into_iter().collect(),
        stroke_fills: forma.marcas.into_iter().collect(),
    })
    .expect("a forma prepara");
    let mut p = ShapePass::new(gpu, format);
    p.com_contorno(contorno);
    p.area_minima_conforme(area_minima_conforme);
    p.limita_as_celulas(celulas_no_maximo);
    p.set_geometries(gpu, [(7u32, &g)]);
    let tex = textura(gpu, wgpu::TextureUsages::RENDER_ATTACHMENT, format);
    let vista = tex.create_view(&wgpu::TextureViewDescriptor::default());
    #[expect(clippy::cast_precision_loss, reason = "LADO é 512")]
    let alvo = [LADO as f32, LADO as f32];
    let mut saida = Vec::new();
    for &(cs, quadros) in etapas {
        let insts: Vec<ShapeInstance> = cs
            .iter()
            .map(|c| ShapeInstance {
                pos: c.pos,
                size: [c.lado, c.lado * c.aspecto],
                basis: basis(c.ang),
                anchor: [0.0, 0.0],
                geometry: 7,
                _pad: 0,
                tint: c.tint,
            })
            .collect();
        p.upload_instances(gpu, &insts);
        // Um clone do handle: o `draw` muta o passe (o contorno, doc 121 §9.5).
        let copias = p.uploaded().expect("carregou").clone();
        let n = u32::try_from(insts.len()).expect("cabem");
        for _ in 0..quadros {
            let mut enc = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
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
                    buffer: &copias,
                    count: u32::try_from(insts.len()).expect("cabem"),
                },
            );
            gpu.queue.submit(Some(enc.finish()));
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            let (com, _) = p.copias_com_contorno(gpu, n);
            let celulas = p.celulas_do_ultimo_quadro(gpu);
            observa(gpu, &p);
            let px = if format == wgpu::TextureFormat::Rgba16Float {
                let b = bytes_de_textura(gpu, &tex, 8);
                b.as_chunks::<8>()
                    .0
                    .iter()
                    .flat_map(|px| {
                        let h = |i: usize| f16(u16::from_le_bytes([px[2 * i], px[2 * i + 1]]));
                        separa([h(0), h(1), h(2), h(3)])
                    })
                    .collect()
            } else {
                bytes_de_textura(gpu, &tex, 4)
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .flat_map(|px| {
                        let f = |i: usize| f32::from(px[i]) / 255.0;
                        separa([f(0), f(1), f(2), f(3)])
                    })
                    .collect()
            };
            saida.push((px, com, celulas));
        }
    }
    saida
}

/// O que a comparação devolve: o pior desvio de alfa, o pior de cor (separada, onde `α ≥ 64`), e
/// quantos pixels desviam mais de `1` no alfa.
#[derive(Debug)]
pub(super) struct Desvio {
    pub(super) alfa_max: u8,
    pub(super) cor_max: u8,
    pub(super) alfa_acima_de_1: usize,
    /// Pixels em que a cor separada desvia mais de `4` num canal.
    cor_acima_de_4: usize,
    /// O pixel do pior desvio de cor, e os dois valores lá (Vello, passe).
    pior_cor: (usize, [u8; 4], [u8; 4]),
    pub(super) pixels_com_tinta: usize,
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
    for (i, (v, p)) in vello
        .as_chunks::<4>()
        .0
        .iter()
        .zip(passe.as_chunks::<4>().0.iter())
        .enumerate()
    {
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

pub(super) fn corre(nome: &str, forma: &Forma<'_>, cs: &[Copia]) -> Option<Desvio> {
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
        // ⭐ doc 121 §9.3 — o anel junta as duas famílias de borda curva: o alfa dos isolados (a
        // borda do furo é uma borda isolada) e a cor dos sobrepostos. MEDIDO igual ao último
        // dígito com o laço ANTIGO (segmento a segmento): alfa `61`, cor `53`/`54`, `4 239` px.
        "aneis even-odd (curvas)" => (100, 60),
        "estrela com traco" => (32, 8),
        // ⭐⭐ doc 121 W4 — o traço sob escala NÃO uniforme, contra a lei da casa. Ver `ESTICADO`.
        "circulo esticado com traco" | "zigue-zague esticado, redondo" => (100, 100),
        // ⭐⭐ doc 121 §9.4 — a FAIXA fecha a quina interior com a área VERDADEIRA, e o Vello soma a
        // sobreposição dos dois troços nos pixels de borda (`quina_exacta`: `0,296` de sobreconta,
        // a faixa a `0,002` da verdade). Medido: `40`/`25` → `73`/`49`, e o defeito lê `255`/`228`.
        "estrela esticada com traco" => (100, 60),
        // O mesmo, com o traço FINO (`quina_exacta`: `0,693` verdadeiro · `0,694` faixa · `1,0` somado).
        // Medido: `41`/`24` → `67`/`67`, e o defeito lê `255`/`241`.
        "traco fino esticado" => (100, 100),
        // As duas cercas da faixa (§9.4). Estrelas de `10`–`24 px` com traço grosso: o laço ANTIGO
        // (uma peça por junta) já lia alfa `79` · cor `20` · `887` px aqui — é o regime em que o
        // traço é do tamanho da forma; a faixa lê `71` · `28` · `1 497` (os cantos interiores que
        // passam a ser a área verdadeira). A barra é a das curvas, e é a MUTAÇÃO que a justifica.
        "estrelas pequenas esticadas, traco grosso" | "estrela esticada, limite 2" => (100, 60),
        n if n.contains("esticad") => (64, 40),
        _ => (2, 4),
    }
}

/// ⭐⭐ **O traço sob escala NÃO uniforme (doc 121 W4) — o vale, MEDIDO** (2026-09-30, RTX, meio-float).
/// A régua é a lei da casa (`stroke_uniform`: a geometria transformada, a caneta REDONDA de largura
/// `w·√|det|`). A linha de CONTROLO é o passe SEM o eixo (a caneta elíptica do contorno expandido no
/// espaço local — o bug #27), medida com o eixo desligado no shader:
///
/// | família | com o eixo: alfa · cor · px `> 1` | sem o eixo (o defeito) |
/// |---|---|---|
/// | estrela, esquadria | `73` · `49` · `133` (a faixa, §9.4 — antes `40` · `25` · `29`) | `255` · `228` · `26 526` |
/// | círculo | `66` · `68` · `7 277` (`3,0 %`) | `255` · `216` · `23 637` (`9,8 %`) |
/// | zigue-zague, chanfro + pontas quadradas | `22` · `8` · `2` | `255` · `211` · `37 119` |
/// | zigue-zague, redondo | `52` · `39` · `762` | `255` · `224` · `30 742` |
/// | traço FINO (`0,01`) | `67` · `67` · `382` (a faixa, §9.4 — antes `41` · `23` · `211`) | `255` · `241` · `28 754` |
///
/// ⚠️ **O que sobra nas curvas e nas juntas redondas é a família dos círculos**: o Vello aplana o
/// contorno no ecrã e deixa-o até `0,25 px` para DENTRO; o passe aplana o eixo no espaço local e os
/// leques com a mesma flecha — a borda difere até `~0,25 px` (alfa `64`), o mesmo vale dos
/// `circulos isolados`. ⇒ barras `100`/`100` nas curvas, `64`/`40` nos polígonos, e em todas a
/// fracção de pixels com alfa `> 1` abaixo de `5 %` (o defeito lê `≥ 9,8 %`).
const ESTICADO_FRACCAO_MAX: f64 = 0.05;

/// ⭐⭐⭐ **As famílias do passe, contra o Vello, pixel a pixel** — e a mesma área pintada.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn o_passe_desenha_o_que_o_vello_desenha() {
    let (est, pent, circ, zz, an) = (estrela(), pentagrama(), circulo(), zigue_zague(), anel());
    let casos: Vec<(&str, Forma<'_>, Vec<Copia>)> = vec![
        (
            "estrelas pequenas",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: None,
            },
            copias(400, 6.0, 40.0, 1),
        ),
        (
            "estrelas grandes",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: None,
            },
            copias(12, 120.0, 900.0, 2),
        ),
        (
            "circulos (curvas)",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: None,
            },
            copias(200, 8.0, 300.0, 3),
        ),
        (
            "circulos isolados",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: None,
            },
            grelha_isolada(),
        ),
        (
            "aneis even-odd (curvas)",
            Forma {
                bp: &an,
                linha: None,
                regra: FillRule::EvenOdd,
                marcas: None,
                traco: None,
            },
            copias(60, 30.0, 400.0, 11),
        ),
        (
            "pentagrama even-odd",
            Forma {
                bp: &pent,
                linha: None,
                regra: FillRule::EvenOdd,
                marcas: None,
                traco: None,
            },
            copias(40, 30.0, 200.0, 4),
        ),
        (
            "estrela com traco",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.06).with_join(Join::Miter),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            copias(40, 40.0, 220.0, 5),
        ),
        // ⭐⭐ doc 121 W4 — o traço sob escala NÃO uniforme, contra a lei da casa.
        (
            "estrela esticada com traco",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.06).with_join(Join::Miter),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 6),
        ),
        (
            "circulo esticado com traco",
            Forma {
                bp: &circ,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.08).with_join(Join::Miter),
                    [0.9, 0.2, 0.1, 1.0],
                )),
            },
            esticadas(40, 30.0, 200.0, 7),
        ),
        (
            "zigue-zague esticado, chanfro e pontas quadradas",
            Forma {
                bp: &zz,
                linha: Some(&zz),
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.07)
                        .with_join(Join::Bevel)
                        .with_caps(Cap::Square),
                    [0.1, 0.5, 0.2, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 8),
        ),
        (
            "zigue-zague esticado, redondo",
            Forma {
                bp: &zz,
                linha: Some(&zz),
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.07)
                        .with_join(Join::Round)
                        .with_caps(Cap::Round),
                    [0.3, 0.1, 0.6, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 9),
        ),
        (
            "traco fino esticado",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.01).with_join(Join::Miter),
                    [0.0, 0.0, 0.0, 1.0],
                )),
            },
            esticadas(60, 40.0, 220.0, 10),
        ),
        // ⭐ doc 121 §9.4 — as duas cercas da faixa numa QUINA que nenhuma estrela grande toca:
        // traço GROSSO em estrelas PEQUENAS (a esquadria recua mais de metade do troço — a faixa
        // cede à junta) e o LIMITE da esquadria abaixo da ponta (a faixa cede ao chanfro).
        (
            "estrelas pequenas esticadas, traco grosso",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.12).with_join(Join::Miter),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(120, 10.0, 24.0, 12),
        ),
        (
            "estrela esticada, limite 2",
            Forma {
                bp: &est,
                linha: None,
                regra: FillRule::NonZero,
                marcas: None,
                traco: Some((
                    Stroke::new(0.06)
                        .with_join(Join::Miter)
                        .with_miter_limit(2.0),
                    [0.1, 0.1, 0.1, 1.0],
                )),
            },
            esticadas(40, 40.0, 220.0, 13),
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
        if nome.contains("esticad") {
            #[expect(clippy::cast_precision_loss, reason = "contagens de pixels")]
            let fr = d.alfa_acima_de_1 as f64 / d.pixels_com_tinta as f64;
            assert!(
                fr <= ESTICADO_FRACCAO_MAX,
                "{nome}: {:.1} % dos pixels desviam no alfa -- a caneta nao e' a da casa: {d:?}",
                100.0 * fr
            );
        }
    }
    assert!(algum);
}

/// Uma CRUZ: côncava e com arestas exactamente verticais quando a cópia não roda.
fn cruz() -> BezPath {
    let (a, b) = (0.5, 0.16);
    let mut bp = BezPath::new();
    bp.move_to((-b, -a));
    for (x, y) in [
        (b, -a),
        (b, -b),
        (a, -b),
        (a, b),
        (b, b),
        (b, a),
        (-b, a),
        (-b, b),
        (-a, b),
        (-a, -b),
        (-b, -b),
    ] {
        bp.line_to((x, y));
    }
    bp.close_path();
    bp
}

/// ⭐⭐⭐ **UMA ARESTA VERTICAL LONGE DO PIXEL NÃO VIRA `NaN`** (doc 121 §9.2, a linha da `=127`).
///
/// A cobertura portada do Vello divide por `xmax − xmin` e conta com um `−1e-6` que só sobrevive em
/// coordenadas de ladrilho (`16 px`); aqui elas são relativas ao pixel, e numa aresta VERTICAL a
/// dezenas de píxeis à esquerda a conta vira `0/0`. ⚠️ **O traço deixou de a expor** quando passou
/// a saltar as peças que não tocam no pixel — e foi a prova de mutação que o disse: sem a guarda o
/// gate da estrela alinhada ficou VERDE. ⇒ o caso que ainda a expõe é o PREENCHIMENTO de uma forma
/// CÔNCAVA alinhada aos eixos: nas fileiras dos braços de cima e de baixo, os pixéis à direita das
/// duas arestas verticais e FORA da cruz leem o `NaN` e pintavam-se.
///
/// ⚠️ As cópias NÃO rodam (é o que põe as arestas na vertical) e são grandes (a aresta fica longe o
/// bastante para o `−1e-6` se perder no `f32`).
#[test]
#[ignore = "precisa de adapter de GPU"]
fn uma_aresta_vertical_longe_do_pixel_nao_vira_nan() {
    let cz = cruz();
    let forma = Forma {
        bp: &cz,
        linha: None,
        regra: FillRule::NonZero,
        marcas: None,
        traco: None,
    };
    let cs: Vec<Copia> = (0..4)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "quatro copias")]
            let k = k as f32;
            Copia {
                pos: [128.0 + 256.0 * (k % 2.0), 128.0 + 256.0 * (k / 2.0).floor()],
                lado: 180.0 + 20.0 * k,
                ang: 0.0,
                tint: [0.2, 0.4, 0.8, 1.0],
                aspecto: 1.0,
            }
        })
        .collect();
    let Some(d) = corre("cruz alinhada", &forma, &cs) else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    assert!(d.pixels_com_tinta > 50_000, "controlo: a cena pinta pouco");
    assert!(
        d.alfa_max <= 2 && d.cor_max <= 4,
        "a cruz alinhada desvia do Vello (alfa {}, cor {}) — o NaN da aresta vertical voltou",
        d.alfa_max,
        d.cor_max
    );
}
