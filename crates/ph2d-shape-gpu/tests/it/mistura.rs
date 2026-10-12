//! ⭐⭐ **A MISTURA POR CÓPIA, PIXEL A PIXEL** (B5) — cópias SOBREPOSTAS desenhadas por corridas
//! ([`ShapePass::draw_em_corridas`]) contra a composição do Vello escrita à mão: a lei W3C de cada
//! mistura e os cortes a `1` onde o Vello os faz (o fundo de cada camada sai de 8 bits; o alfa do
//! `Plus` é cortado; a cor do `Plus` passa solta ao `Normal` seguinte). A paridade contra o Vello DE
//! VERDADE vive na rota (`ph2d-app-motion`, `motion_shape_placa_mistura_gpu_tests`); aqui mede-se o
//! PASSE: os estados fixos, a 2.ª saída do Multiplica, os cortes e o rascunho do grupo.

use ph2d_gpu::GpuContext;
use ph2d_shape_gpu::{
    Copias, Corrida, FillRule, Mistura, ShapeGeometry, ShapeInput, ShapeInstance, ShapePass,
    ShapeView,
};
use ph2d_vector::BezPath;

const LADO: u32 = 64;
const FORMATO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// Um quadrado de lado `1` centrado na origem.
fn quadrado() -> BezPath {
    let mut bp = BezPath::new();
    bp.move_to((-0.5, -0.5));
    bp.line_to((0.5, -0.5));
    bp.line_to((0.5, 0.5));
    bp.line_to((-0.5, 0.5));
    bp.close_path();
    bp
}

/// Uma cópia do quadrado de `x0..x1` em pixels (altura `8..56`), com a tinta `tint` (reta).
fn copia(x0: f32, x1: f32, tint: [f32; 4]) -> ShapeInstance {
    ShapeInstance {
        pos: [(x0 + x1) * 0.5, 32.0],
        size: [x1 - x0, 48.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        anchor: [0.0, 0.0],
        geometry: 1,
        _pad: 0,
        tint,
    }
}

/// O fixture: quatro faixas que se sobrepõem — `A` em `8..40`, `B` em `24..56`, `C` em `16..48` e
/// `D` em `4..60` — com alfas parciais e somas acima de `1`.
fn copias() -> Vec<ShapeInstance> {
    vec![
        copia(8.0, 40.0, [0.8, 0.2, 0.4, 0.7]),
        copia(24.0, 56.0, [0.3, 0.9, 0.6, 0.8]),
        copia(16.0, 48.0, [0.9, 0.7, 0.2, 0.9]),
        copia(4.0, 60.0, [0.5, 0.6, 0.9, 0.35]),
    ]
}

/// A composição W3C de `s` sobre `b`, pré-multiplicadas — a tabela do `mistura.rs`.
fn compoe(b: [f32; 4], s: [f32; 4], m: Mistura) -> [f32; 4] {
    let (ab, a_s) = (b[3], s[3]);
    let sobre = a_s + ab * (1.0 - a_s);
    match m {
        Mistura::Normal => std::array::from_fn(|k| s[k] + b[k] * (1.0 - a_s)),
        Mistura::Soma => [s[0] + b[0], s[1] + b[1], s[2] + b[2], (a_s + ab).min(1.0)],
        Mistura::Tela => {
            let c = |k: usize| s[k] + b[k] * (1.0 - s[k]);
            [c(0), c(1), c(2), sobre]
        }
        Mistura::Multiplica => {
            let c = |k: usize| s[k] * (1.0 - ab) + b[k] * (1.0 - a_s + s[k]);
            [c(0), c(1), c(2), sobre]
        }
    }
}

fn corta(c: [f32; 4]) -> [f32; 4] {
    c.map(|v| v.min(1.0))
}

/// O que o Vello compõe no pixel de coluna `x`: as `corridas` em ordem, cada cópia com a sua mistura
/// e o grupo isolado num fundo limpo. `solto` = a cor pode estar acima de `1`.
fn o_vello(x: f32, cs: &[ShapeInstance], corridas: &[Corrida]) -> [f32; 4] {
    let cobre = |c: &ShapeInstance| (x - c.pos[0]).abs() < c.size[0] * 0.5;
    let passo = |st: &mut [f32; 4], solto: &mut bool, c: &ShapeInstance, m: Mistura| {
        if !cobre(c) {
            return;
        }
        if m != Mistura::Normal && std::mem::take(solto) {
            *st = corta(*st);
        }
        let t = c.tint;
        *st = compoe(*st, [t[0] * t[3], t[1] * t[3], t[2] * t[3], t[3]], m);
        *solto |= m == Mistura::Soma;
    };
    let (mut st, mut solto) = ([0.0f32; 4], false);
    let mut i = 0;
    while i < corridas.len() {
        let g = corridas[i].grupo;
        let mut j = i + 1;
        while g != 0 && j < corridas.len() && corridas[j].grupo == g {
            j += 1;
        }
        let (mut gs, mut gsolto) = ([0.0f32; 4], false);
        for c in &corridas[i..j] {
            for k in c.copias.clone() {
                if g == 0 {
                    passo(&mut st, &mut solto, &cs[k as usize], c.mistura);
                } else {
                    passo(&mut gs, &mut gsolto, &cs[k as usize], c.mistura);
                }
            }
        }
        if g != 0 {
            if std::mem::take(&mut solto) {
                st = corta(st);
            }
            st = compoe(st, gs, Mistura::Normal);
            solto = gsolto;
        }
        i = j;
    }
    if solto { corta(st) } else { st }
}

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

/// O passe desenha as `corridas` num alvo limpo; devolve a linha do meio, pré-multiplicada.
fn pelo_passe(gpu: &GpuContext, cs: &[ShapeInstance], corridas: &[Corrida]) -> Vec<[f32; 4]> {
    let mut passe = ShapePass::new(gpu, FORMATO);
    let geo = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((&quadrado(), FillRule::NonZero)),
        strokes: Vec::new(),
        stroke_fills: Vec::new(),
    })
    .expect("o quadrado tem geometria");
    passe.set_geometries(gpu, [(1, &geo)]);
    passe.upload_instances(gpu, cs);
    let alvo = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: LADO,
            height: LADO,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let vista = alvo.create_view(&wgpu::TextureViewDescriptor::default());
    let buffer = passe.uploaded().expect("as copias subiram").clone();
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    #[expect(clippy::cast_possible_truncation, reason = "quatro copias")]
    let count = cs.len() as u32;
    passe.draw_em_corridas(
        gpu,
        &mut enc,
        &vista,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ShapeView {
            lin: [1.0, 0.0, 0.0, 1.0],
            t: [0.0, 0.0],
            alvo: [LADO as f32, LADO as f32],
        },
        Copias {
            buffer: &buffer,
            count,
        },
        corridas,
    );
    let bpr = LADO * 8;
    let leitura = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(bpr * LADO),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    enc.copy_texture_to_buffer(
        alvo.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &leitura,
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
    leitura.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let v = leitura.slice(..).get_mapped_range().to_vec();
    let linha = (LADO / 2 * bpr) as usize;
    v[linha..linha + bpr as usize]
        .as_chunks::<8>()
        .0
        .iter()
        .map(|px| std::array::from_fn(|k| f16(u16::from_le_bytes([px[2 * k], px[2 * k + 1]]))))
        .collect()
}

fn corrida(copias: std::ops::Range<u32>, mistura: Mistura, grupo: u32) -> Corrida {
    Corrida {
        copias,
        mistura,
        grupo,
    }
}

/// O pior desvio da linha do meio contra o modelo, nas colunas inteiras de cada faixa (as bordas
/// dos quadrados caem em colunas inteiras: cobertura `0` ou `1`).
#[expect(clippy::cast_precision_loss, reason = "colunas de 64")]
fn desvio(gpu: &GpuContext, cs: &[ShapeInstance], corridas: &[Corrida]) -> f32 {
    let linha = pelo_passe(gpu, cs, corridas);
    (0..LADO as usize).fold(0.0f32, |m, x| {
        let quer = o_vello(x as f32 + 0.5, cs, corridas);
        (0..4).fold(m, |m, k| m.max((linha[x][k] - quer[k]).abs()))
    })
}

/// ⭐⭐ **Cada mistura × cada alcance, com as cópias SOBREPOSTAS na mesma chamada**: tudo `Everything`
/// (uma corrida), a mistura depois de um `Normal`, o grupo isolado (`Copies`) depois de conteúdo, o
/// grupo como 1.º conteúdo, a linha com modo próprio dentro do grupo, e a Soma seguida de `Normal`
/// (a cor solta). Controlo: a mesma cena desenhada em `Normal` tem de falhar o modelo da mistura —
/// senão a régua não vê a mistura.
#[test]
#[ignore = "precisa de adapter de GPU"]
fn cada_mistura_por_copia_compoe_como_o_vello() {
    let Some(gpu) = GpuContext::new(GpuContext::default_instance(), None).ok() else {
        eprintln!("sem adapter — o gate não correu");
        return;
    };
    let cs = copias();
    let passe = ShapePass::new(&gpu, FORMATO);
    let mut viu = 0;
    for m in [Mistura::Soma, Mistura::Tela, Mistura::Multiplica] {
        if !passe.sabe(m) {
            eprintln!("  {m:?}: o dispositivo nao a desenha — fica no Vello");
            continue;
        }
        let casos: [(&str, Vec<Corrida>); 6] = [
            ("everything", vec![corrida(0..4, m, 0)]),
            (
                "depois de normal",
                vec![corrida(0..1, Mistura::Normal, 0), corrida(1..4, m, 0)],
            ),
            (
                "grupo depois de conteudo",
                vec![corrida(0..1, Mistura::Normal, 0), corrida(1..4, m, 7)],
            ),
            (
                "grupo primeiro",
                vec![corrida(0..3, m, 7), corrida(3..4, m, 0)],
            ),
            (
                "linha no grupo",
                vec![
                    corrida(0..1, Mistura::Normal, 0),
                    corrida(1..2, m, 7),
                    corrida(2..3, Mistura::Tela, 7),
                    corrida(3..4, m, 7),
                ],
            ),
            (
                "e normal por cima",
                vec![corrida(0..3, m, 0), corrida(3..4, Mistura::Normal, 0)],
            ),
        ];
        for (nome, corridas) in casos {
            let d = desvio(&gpu, &cs, &corridas);
            eprintln!("  {m:?} / {nome}: desvio {d:.5}");
            assert!(d < 2e-3, "{m:?} / {nome}: o passe desvia {d} do Vello");
            viu += 1;
        }
        let normal = [corrida(0..4, Mistura::Normal, 0)];
        let linha = pelo_passe(&gpu, &cs, &normal);
        #[expect(clippy::cast_precision_loss, reason = "colunas de 64")]
        let controlo = (0..LADO as usize).fold(0.0f32, |acc, x| {
            let quer = o_vello(x as f32 + 0.5, &cs, &[corrida(0..4, m, 0)]);
            (0..4).fold(acc, |acc, k| acc.max((linha[x][k] - quer[k]).abs()))
        });
        assert!(
            controlo > 0.05,
            "controlo: em Normal a cena nao se distingue de {m:?} ({controlo})"
        );
    }
    assert!(
        viu >= 12,
        "controlo: Soma e Tela correm em toda placa ({viu} casos)"
    );
}
