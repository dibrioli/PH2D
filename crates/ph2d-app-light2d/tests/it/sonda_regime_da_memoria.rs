//! ⭐ W5x — **o regime da memória da `c2` do High** (handoff W5w §4 → W5x): a mesma cena dá o piso ou
//! ~2× conforme a montagem. Duas sondas à mão sobre a [`SondaDeMemoria`] (mínimo de despachos):
//!
//! - [`sonda_regime_da_memoria_isolada`] — buffers soltos de 48 MiB: frescos (com e sem `COPY_SRC`),
//!   reciclados, um dedicado, herdados de texturas que morreram (render limpa, render crua, armazém) e
//!   vizinhos de uma textura viva; a banda de ler depois de cada história do conteúdo.
//! - [`sonda_regime_da_memoria_na_holografica`] — a vista do app (cena 5) renasce `K` vezes; por
//!   geração, a `c2` ao lado do tempo de ler e de saltar em CADA buffer grande da holográfica, sozinho.

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_light2d_gpu::{Config, Medida, Solver, SondaDeMemoria};

use crate::comum::gpu;
use crate::medida_luz2d_w5e::fabrica;
use crate::medida_luz2d_w5v::{App, cena_do_smoke};

const MIB: u64 = 1024 * 1024;

/// Uma vista nova sobre o entorno que fica — o que o app monta quando o solver da vista renasce.
fn nova_vista(
    gpu: &GpuContext,
    a: vello::kurbo::Affine,
    alvo: (u32, u32),
    com: &dyn Fn(Config) -> Config,
    entorno: Option<&Solver>,
) -> Solver {
    let dg = ph2d_app_light2d::qualidade::degrau_do(NivelDaLuz::Alta);
    let g = ph2d_app_light2d::grade_do_quadro(a, alvo, dg).expect("a grade");
    let mut v = Solver::novo(
        gpu,
        com(ph2d_app_light2d::config_da_vista(&gpu.device, g, a, dg)),
    )
    .expect("solver da vista");
    let (i, f) = ph2d_app_light2d::regiao_da_janela(v.grade_da_fluencia(), a, alvo);
    v.regiao_fina(i, f);
    v.somar_entorno(gpu, entorno);
    v
}

/// Um buffer com os usos dos da holográfica (`STORAGE | COPY_DST`), mais `COPY_SRC` com `src`.
fn buffer(d: &wgpu::Device, tam: u64, nome: &'static str, src: bool) -> wgpu::Buffer {
    let usos = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
    d.create_buffer(&wgpu::BufferDescriptor {
        label: Some(nome),
        size: tam,
        usage: if src {
            usos | wgpu::BufferUsages::COPY_SRC
        } else {
            usos
        },
        mapped_at_creation: false,
    })
}

fn texturas_de_24_mib(
    d: &wgpu::Device,
    n: usize,
    usage: wgpu::TextureUsages,
) -> Vec<wgpu::Texture> {
    (0..n)
        .map(|_| {
            d.create_texture(&wgpu::TextureDescriptor {
                label: Some("sonda/textura"),
                size: wgpu::Extent3d {
                    width: 2048,
                    height: 1536,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage,
                view_formats: &[],
            })
        })
        .collect()
}

fn limpar_por_render(gpu: &GpuContext, texturas: &[wgpu::Texture]) {
    let mut enc = gpu
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    for t in texturas {
        let vista = t.create_view(&wgpu::TextureViewDescriptor::default());
        enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sonda limpa"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &vista,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    gpu.queue.submit([enc.finish()]);
}

#[test]
#[ignore = "sonda W5x (placa): a compressão e a herança das texturas em buffers soltos"]
fn sonda_regime_da_memoria_isolada() {
    const TAM: u64 = 48 * MIB;
    const N: usize = 6;
    let Some(gpu) = gpu() else { return };
    let Some(sonda) = SondaDeMemoria::nova(&gpu) else {
        eprintln!("sem TIMESTAMP_QUERY");
        return;
    };
    let d = &gpu.device;
    // Os frescos ficam vivos enquanto os outros nascem.
    let mut todos: Vec<(String, wgpu::Buffer)> = (0..N)
        .map(|k| (format!("fresco {k}"), buffer(d, TAM, "sonda/fresco", false)))
        .chain((0..N).map(|k| {
            (
                format!("fresco+src {k}"),
                buffer(d, TAM, "sonda/fresco_src", true),
            )
        }))
        .collect();
    let largados: Vec<wgpu::Buffer> = (0..N)
        .map(|_| buffer(d, TAM, "sonda/largado", false))
        .collect();
    drop(largados);
    let _ = d.poll(wgpu::PollType::wait_indefinitely());
    todos.extend((0..N).map(|k| {
        (
            format!("reciclado {k}"),
            buffer(d, TAM, "sonda/reciclado", false),
        )
    }));
    todos.push((
        "dedicado".into(),
        buffer(d, 320 * MIB, "sonda/dedicado", false),
    ));
    // Herdado: texturas (24 MiB) nascem e morrem — um buffer pequeno segura o bloco — e os buffers nascem
    // no lugar delas. Render limpa · render crua (nunca usada) · armazém crua.
    let usos_rt = wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING;
    let usos_arm = wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING;
    for (rotulo, usos, limpar) in [
        ("herdado rt", usos_rt, true),
        ("herdado cru", usos_rt, false),
        ("herdado arm", usos_arm, false),
    ] {
        let texturas = texturas_de_24_mib(d, 2 * N, usos);
        let _segura = buffer(d, MIB, "sonda/segura", false);
        if limpar {
            limpar_por_render(&gpu, &texturas);
        }
        drop(texturas);
        let _ = d.poll(wgpu::PollType::wait_indefinitely());
        todos.extend((0..N).map(|k| {
            (
                format!("{rotulo} {k}"),
                buffer(d, TAM, "sonda/herdado", false),
            )
        }));
    }
    // Vizinho: uma textura de render VIVA e limpa; os buffers nascem ao lado dela.
    let viva = texturas_de_24_mib(d, 1, usos_rt);
    limpar_por_render(&gpu, &viva);
    todos.extend((0..N).map(|k| {
        (
            format!("vizinho {k}"),
            buffer(d, TAM, "sonda/vizinho", false),
        )
    }));
    let limpar = |b: &wgpu::Buffer| {
        let mut enc = d.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        enc.clear_buffer(b, 0, None);
        gpu.queue.submit([enc.finish()]);
    };
    // A banda de LER (GB/s) depois de cada história do conteúdo, na ordem: o zero do nascimento (o
    // `wgpu` limpa o que nunca foi escrito) · `clear_buffer` · zero por shader · ruído por shader ·
    // `clear_buffer` de novo por cima do ruído · e ler aos saltos (ms) sobre esse último.
    eprintln!(
        "buffer        · GB/s ler: nascido · clear · zero shader · ruído shader · clear depois · ms saltar"
    );
    for (nome, b) in &todos {
        let gbs = |ms: f64| b.size() as f64 / (ms * 1e-3) / 1e9;
        let nascido = sonda.ms(b, Medida::Ler, 3);
        limpar(b);
        let clear = sonda.ms(b, Medida::Ler, 3);
        let _ = sonda.ms(b, Medida::EscreverZero, 1);
        let zero = sonda.ms(b, Medida::Ler, 3);
        let _ = sonda.ms(b, Medida::EscreverRuido, 1);
        let ruido = sonda.ms(b, Medida::Ler, 3);
        limpar(b);
        let depois = sonda.ms(b, Medida::Ler, 3);
        let saltar = sonda.ms(b, Medida::Saltar, 3);
        eprintln!(
            "{nome:13} · {:6.0} · {:6.0} · {:6.0} · {:6.0} · {:6.0} · {saltar:6.3}",
            gbs(nascido),
            gbs(clear),
            gbs(zero),
            gbs(ruido),
            gbs(depois)
        );
    }
}

#[test]
#[ignore = "sonda W5x (placa): a c2 por geração contra cada buffer da holográfica sozinho"]
fn sonda_regime_da_memoria_na_holografica() {
    const AQUECE: usize = 8;
    const QUADROS: usize = 16;
    let Some(gpu) = gpu() else { return };
    let Some(sonda) = SondaDeMemoria::nova(&gpu) else {
        eprintln!("sem TIMESTAMP_QUERY");
        return;
    };
    let geracoes: usize = std::env::var("PH2D_W5X_GERACOES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);
    let duas = std::env::var("PH2D_W5X_DUAS").map_or(true, |v| v.trim() != "0");
    let berco = std::env::var("PH2D_W5X_BERCO").map_or(true, |v| v.trim() != "0");
    let alvo = std::env::var("PH2D_W5X_ALVO").map_or((2560, 1080), |v| {
        let (l, a) = v.split_once('x').expect("PH2D_W5X_ALVO = <l>x<a>");
        (l.parse().expect("largura"), a.parse().expect("altura"))
    });
    // `PH2D_W5X_COMO_O_APP=1`: só a VISTA renasce (o entorno fica) e larga-se com quadros em voo — o
    // `PH2D_LIGHT2D_RENASCE` do app.
    let como_o_app = std::env::var("PH2D_W5X_COMO_O_APP").is_ok_and(|v| v.trim() == "1");
    let cena = cena_do_smoke(5);
    let a = fabrica(alvo);
    let com = |c: Config| Config {
        hrc_nasce_duas_vezes: duas,
        hrc_berco: berco,
        ..c
    };
    eprintln!(
        "W5x · alvo {}×{} · High · duas {duas} · berço {berco} · como o app {como_o_app} · {geracoes} gerações",
        alvo.0, alvo.1
    );
    let mut viva: Option<App> = None;
    for g in 0..geracoes {
        let mut app = match viva.take() {
            Some(App { vista, entorno }) if como_o_app => {
                drop(vista);
                let vista = nova_vista(&gpu, a, alvo, &com, entorno.as_ref());
                App { vista, entorno }
            }
            _ => App::novo(&gpu, a, alvo, NivelDaLuz::Alta, &com),
        };
        for _ in 0..AQUECE {
            app.quadro(&gpu, &cena);
        }
        let c2 = (0..QUADROS)
            .filter_map(|_| {
                app.quadro(&gpu, &cena);
                app.vista.tempos(&gpu).map(|t| t.c2_total())
            })
            .fold(f64::INFINITY, f64::min);
        let mut linha = Vec::new();
        let (mut soma_ler, mut soma_saltar) = (0.0, 0.0);
        for (nome, b) in app.vista.buffers_da_holografica() {
            let (tl, ts) = (sonda.ms(b, Medida::Ler, 3), sonda.ms(b, Medida::Saltar, 3));
            soma_ler += tl;
            soma_saltar += ts;
            linha.push(format!(
                "{} {:.0}M l{tl:.3} s{ts:.3}",
                nome.trim_start_matches("luz2d/buf/hrc_"),
                b.size() as f64 / MIB as f64,
            ));
        }
        eprintln!(
            "g{g:<2} c2 {c2:6.2} · Σler {soma_ler:6.3} Σsaltar {soma_saltar:6.3} · {}",
            linha.join(" · ")
        );
        if como_o_app {
            // Quadros em voo quando a vista se larga.
            for _ in 0..3 {
                app.quadro(&gpu, &cena);
            }
            viva = Some(app);
        }
    }
}
