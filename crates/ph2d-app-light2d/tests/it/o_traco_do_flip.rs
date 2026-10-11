//! ⭐⭐ **A cena 7 — o traço do Flip bloqueia a luz como a imagem** (W5e; plano §2.4: a C0 captura o
//! Flip). Os discos de tinta entram no documento do Flip pela porta do smoke; a entidade do objeto
//! (a que a sincronização do Flip cria) ganha o «bloqueia a luz»; e pelo caminho do produto (a
//! `CamadaDeOclusao` do Flip, o rasterizador do traço numa camada da grade) cada disco tem as FACES e
//! a luz dele bate com a da IMAGEM gêmea no espelho em relação à lâmpada — a régua da cena 6.
//!
//! Controlo na MESMA corrida: a mesma cena sem a família do Flip na luz (a W5d) — o traço não
//! bloqueia, a face de trás recebe tanto quanto a da lâmpada.

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro, smoke};
use ph2d_ecs::{FlipObjectRef, GlobalTransform, Name, PresentWorld, SimRef, Transform};
use ph2d_flip::FlipDoc;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_render::{DISC_TILE_PX, Sprite, disc_tile_pixels};

use crate::comum::Fluencia;

/// Os pontos das faces ficam a esta fração do DIÂMETRO menor, de cada lado do centro (a da cena 6).
const PROFUNDIDADE: f64 = 0.35;
/// A face virada para a lâmpada é pelo menos isto × a de trás (o da cena 6).
pub(crate) const CONTRASTE_DAS_FACES: f64 = 10.0;

/// O que a régua do ESPELHO lê de uma forma desenhada e da imagem gêmea dela (`y → −y`, a lâmpada na
/// origem): as faces (a da lâmpada e a de trás, `[desenho, imagem]`) e a variação ao longo de anéis
/// dentro e logo fora, relativa (um risco ou uma faceta é variação que a imagem não tem).
pub(crate) struct Espelho {
    pub frente: [f64; 2],
    pub tras: [f64; 2],
    pub variacao: f64,
}

/// A régua do espelho (cenas 6 e 7) — `centro` e `semi` do DESENHO; a imagem está em `[x, −y]`.
pub(crate) fn espelho(f: &Fluencia, centro: [f64; 2], semi: [f64; 2]) -> Espelho {
    let k = PROFUNDIDADE * 2.0 * semi[0].min(semi[1]);
    let mut frente = [0.0; 2];
    let mut tras = [0.0; 2];
    for (i, c) in [centro, [centro[0], -centro[1]]].into_iter().enumerate() {
        let n = c[0].hypot(c[1]);
        let u = [-c[0] / n, -c[1] / n];
        frente[i] = f.amostra([c[0] + u[0] * k, c[1] + u[1] * k]);
        tras[i] = f.amostra([c[0] - u[0] * k, c[1] - u[1] * k]);
    }
    let mut tv = [0.0_f64; 2];
    for (f_anel, fora) in [(0.35, 0.0), (0.65, 0.0), (0.9, 0.0), (1.0, 0.03)] {
        let (d, im): (Vec<f64>, Vec<f64>) = (0..720)
            .map(|k| {
                let t = std::f64::consts::TAU * f64::from(k) / 720.0;
                let (sn, cs) = t.sin_cos();
                let n = (semi[1] * cs).hypot(semi[0] * sn);
                let p = [
                    centro[0] + f_anel * semi[0] * cs + fora * semi[1] * cs / n,
                    centro[1] + f_anel * semi[1] * sn + fora * semi[0] * sn / n,
                ];
                (f.amostra(p), f.amostra([p[0], -p[1]]))
            })
            .unzip();
        for k in 0..720 {
            let em = |v: &[f64], j: isize| v[(k as isize + j).rem_euclid(720) as usize];
            tv[0] += (em(&d, 1) - em(&d, 0)).abs();
            tv[1] += (em(&im, 1) - em(&im, 0)).abs();
        }
    }
    Espelho {
        frente,
        tras,
        variacao: (tv[0] - tv[1]) / tv[1],
    }
}

/// ⭐ Uma família de teste que desenha DISCOS com a área EXATA de cada píxel (16×16 subamostras),
/// premultiplicados na cor linear e guardados em sRGB — o que o contrato `CamadaDeOclusao` pede de um
/// traço redondo.
pub(crate) struct DiscosExatos {
    pub entidade: u64,
    /// `(centro, raio, cor sRGB)`.
    pub discos: Vec<([f64; 2], f64, [u8; 3])>,
}

fn linear(c: u8) -> f32 {
    let c = f32::from(c) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

impl ph2d_light2d_gpu::CamadaDeOclusao for DiscosExatos {
    fn desenhar(
        &mut self,
        gpu: &ph2d_gpu::GpuContext,
        alvo: &wgpu::TextureView,
        (w, h): (u32, u32),
        [x0, y0, x1, y1]: [f64; 4],
        quem: &[u64],
    ) -> bool {
        if !quem.contains(&self.entidade) {
            return false;
        }
        const N: u32 = 16;
        let (pw, ph) = ((x1 - x0) / f64::from(w), (y1 - y0) / f64::from(h));
        let mut px = vec![0_u8; (w * h * 4) as usize];
        let srgb = |v: f32| {
            let v = v.clamp(0.0, 1.0);
            let e = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (e * 255.0).round() as u8
        };
        for &(c, r, cor) in &self.discos {
            let lin = cor.map(linear);
            let i0 = (((c[0] - r - x0) / pw).floor().max(0.0)) as u32;
            let i1 = (((c[0] + r - x0) / pw).ceil() as u32).min(w);
            let j0 = (((y1 - c[1] - r) / ph).floor().max(0.0)) as u32;
            let j1 = (((y1 - c[1] + r) / ph).ceil() as u32).min(h);
            for j in j0..j1 {
                for i in i0..i1 {
                    let mut dentro = 0;
                    for sj in 0..N {
                        for si in 0..N {
                            let x = x0 + (f64::from(i) + (f64::from(si) + 0.5) / f64::from(N)) * pw;
                            let y = y1 - (f64::from(j) + (f64::from(sj) + 0.5) / f64::from(N)) * ph;
                            if (x - c[0]).hypot(y - c[1]) <= r {
                                dentro += 1;
                            }
                        }
                    }
                    let a = dentro as f32 / (N * N) as f32;
                    let k = ((j * w + i) * 4) as usize;
                    for (canal, v) in lin.iter().enumerate() {
                        px[k + canal] = srgb(v * a);
                    }
                    px[k + 3] = (a * 255.0).round() as u8;
                }
            }
        }
        gpu.queue.write_texture(
            alvo.texture().as_image_copy(),
            &px,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        true
    }
}

/// A face da lâmpada do TRAÇO (discos de área exata, a camada na captura lida exata) contra a da
/// IMAGEM, relativo — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`, 30 quadros com o rebote): −0,38 % e
/// +0,38 %; as faces 45 × e 38 ×. Pelo traço do Flip de verdade (antes de o portão sair da família):
/// −0,55 % e +0,19 %. A tolerância é a da cena 6.
const TOL_TRACO_IMAGEM: f64 = 0.01;
/// A variação dos anéis do traço acima da da imagem — MEDIDO: +2,0 % e −1,4 % (a camada lida pelo meio
/// na grade fina dava +13 e +18 %: recusada). A da cena 6.
const TOL_TV_TRACO_IMAGEM: f64 = 0.08;

#[test]
fn a_cena_7_ensina_o_traco_do_flip() {
    let Some(gpu) = crate::comum::gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 7));
    let mut doc = FlipDoc::default();
    assert!(
        smoke::semear_flip(&mut doc, 6).is_none(),
        "só a 7 semeia o Flip"
    );
    let id = smoke::semear_flip(&mut doc, 7).expect("o objeto do traço");
    assert_eq!(
        smoke::marcar_flip(&mut w, &doc, id),
        Some(id),
        "antes da sincronização do Flip não há entidade a marcar"
    );
    // A sincronização do Flip (`entities::sync`): a entidade do objeto, com o nome dele.
    let e = w
        .spawn((
            Transform::IDENTITY,
            Name::new(smoke::NOME_DO_TRACO),
            FlipObjectRef(id.0),
        ))
        .id();
    assert_eq!(smoke::marcar_flip(&mut w, &doc, id), None);
    let cena = cena_de_luz(&w);
    assert_eq!(cena.outros.len(), 1, "o traço bloqueia");
    assert!(cena.outros[0].reflete, "e recebe a luz");

    let mut r = crate::comum::renderer(&gpu);
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = r
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(bevy_ecs::entity::Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<ph2d_light2d_ecs::BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let mut i = crate::comum::quad(disco, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(1920, 1080));
    // A família do Flip, aqui, é a de DISCOS EXATOS (a área de cada píxel): a luz não pode depender de
    // outra família. Que o Flip desenha essa mesma área prova-o o teste de contrato DELE
    // (`ph2d-app-flip`, `pass_luz_tests.rs`).
    let mut exatos = DiscosExatos {
        entidade: e.to_bits(),
        discos: smoke::DISCOS_DE_TINTA
            .iter()
            .map(|d| (d.centro, d.raio, d.cor))
            .collect(),
    };
    let mut medir = |com_flip: bool, present: &mut PresentWorld| {
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        for _ in 0..30 {
            assert!(l.preparar(&w, afim, (1920, 1080)));
            if com_flip {
                l.gravar_com(&gpu, present, &r, &mut [&mut exatos]);
            } else {
                l.gravar(&gpu, present, &r);
            }
        }
        crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"))
    };
    let sem = medir(false, &mut present);
    let f = medir(true, &mut present);
    let mut falhas = Vec::new();
    for d in &smoke::DISCOS_DE_TINTA {
        let semi = [d.raio; 2];
        let controlo = espelho(&sem, d.centro, semi);
        assert!(
            controlo.frente[0] < CONTRASTE_DAS_FACES * controlo.tras[0],
            "controlo: sem o Flip na luz o {} não devia ter faces ({:.3} × {:.3})",
            d.nome,
            controlo.frente[0],
            controlo.tras[0]
        );
        let m = espelho(&f, d.centro, semi);
        let paridade = (m.frente[0] - m.frente[1]) / m.frente[1];
        eprintln!(
            "cena 7 · {}: faces traço {:.3} × {:.3} · imagem {:.3} × {:.3} · traço × imagem {:+.2} % · anéis {:+.1} %",
            d.nome,
            m.frente[0],
            m.tras[0],
            m.frente[1],
            m.tras[1],
            100.0 * paridade,
            100.0 * m.variacao
        );
        if m.frente[0] < CONTRASTE_DAS_FACES * m.tras[0] {
            falhas.push(format!("{} sem faces", d.nome));
        }
        if paridade.abs() > TOL_TRACO_IMAGEM {
            falhas.push(format!("{}: traço × imagem {paridade:+.3}", d.nome));
        }
        if m.variacao > TOL_TV_TRACO_IMAGEM {
            falhas.push(format!("{}: anéis {:+.3}", d.nome, m.variacao));
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join(" · "));
}

/// O erro relativo do blit do Flip contra a conta na CPU — MEDIDO + 10 % (RTX 5060 Ti, a cena 7 a
/// 480×270): `1,34e-3` em 1 680 canais; o do sprite é `1,21e-3` (`paridade_luz_recebida`): o alvo é
/// `f16` e o filtro bilinear da placa tem 8 bits de fração.
const TOL_TRACO_RECEBE: f64 = 1.48e-3;

/// ⭐ **O traço do Flip RECEBE a luz como o sprite** (W5e): o `fs_blit` do Flip concatena o
/// `receber.wgsl` — cada pixel é a cor linear × `F(mundo)/2π`, a mesma conta do sprite e da colagem,
/// medida contra a fluência lida de volta. Controlo na mesma corrida: sem luz, o blit de sempre (a cor
/// linear, byte a byte entre duas corridas apagadas).
#[test]
fn o_traco_do_flip_recebe_a_luz_como_o_sprite() {
    use ph2d_flip_render::{FlipCompose, SLICE_FORMAT};
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    let alvo = (480_u32, 270_u32);
    let afim = ph2d_render::Camera2d::default()
        .world_to_screen_affine(ph2d_host::WindowSize::new(alvo.0, alvo.1));
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 7));
    let r = crate::comum::renderer(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    for _ in 0..3 {
        assert!(l.preparar(&w, afim, alvo));
        l.gravar(&gpu, &mut PresentWorld::new(), &r);
    }
    let f = crate::comum::fluencia_lida(&gpu, l.solver().expect("solver"));
    let cor_srgb = [191_u8, 140, 64];
    let tex = |formato, usos| {
        gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cena 7 blit"),
            size: wgpu::Extent3d {
                width: alvo.0,
                height: alvo.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: formato,
            usage: usos,
            view_formats: &[],
        })
    };
    let src = tex(
        SLICE_FORMAT,
        wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    );
    let pixels: Vec<u8> = (0..alvo.0 * alvo.1)
        .flat_map(|_| [cor_srgb[0], cor_srgb[1], cor_srgb[2], 255])
        .collect();
    gpu.queue.write_texture(
        src.as_image_copy(),
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(alvo.0 * 4),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width: alvo.0,
            height: alvo.1,
            depth_or_array_layers: 1,
        },
    );
    let mut compose = FlipCompose::new(&gpu.device, wgpu::TextureFormat::Rgba16Float);
    let mut corre = |luz: bool| {
        let destino = tex(
            wgpu::TextureFormat::Rgba16Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let na_tela = luz.then(|| l.na_tela()).flatten();
        compose.receber_luz(&gpu, na_tela.as_ref().map(|(r, m)| (r, *m)));
        compose.blit(
            &gpu.device,
            &gpu.queue,
            &src,
            &destino.create_view(&wgpu::TextureViewDescriptor::default()),
        );
        crate::comum::ler(&gpu, &destino, 8)
    };
    let (apagada, outra_apagada, acesa) = (corre(false), corre(false), corre(true));
    assert_eq!(
        apagada, outra_apagada,
        "controlo: o blit apagado é determinístico"
    );
    let canal = |px: &[u8], k: usize| {
        f64::from(half::f16::from_le_bytes([px[2 * k], px[2 * k + 1]]).to_f32())
    };
    let lin = cor_srgb.map(|c| {
        let c = f64::from(c) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let mapa = ph2d_app_light2d::tela_para_mundo(afim);
    let mut pior = 0.0_f64;
    let mut pontos = 0;
    for py in (8..alvo.1 - 8).step_by(13) {
        for px in (8..alvo.0 - 8).step_by(17) {
            let i = ((py * alvo.0 + px) * 8) as usize;
            let (a, b) = (&apagada[i..i + 8], &acesa[i..i + 8]);
            let (x, y) = (f64::from(px) + 0.5, f64::from(py) + 0.5);
            let mundo = [
                mapa[0][0] * x + mapa[0][1] * y + mapa[0][2],
                mapa[1][0] * x + mapa[1][1] * y + mapa[1][2],
            ];
            for (c, &esperado_apagado) in lin.iter().enumerate() {
                assert!(
                    (canal(a, c) - esperado_apagado).abs() <= 2e-3,
                    "controlo: sem luz o blit é a cor linear ({} × {esperado_apagado})",
                    canal(a, c)
                );
                let luz = f.amostra_canal(mundo, c) / std::f64::consts::TAU;
                let esperado = esperado_apagado * luz;
                if esperado > 1e-3 {
                    pior = pior.max((canal(b, c) - esperado).abs() / esperado);
                    pontos += 1;
                }
            }
        }
    }
    eprintln!("o traço recebe: {pontos} canais, o pior erro relativo {pior:.2e}");
    assert!(pontos > 100, "poucos pontos acesos ({pontos})");
    assert!(
        pior <= TOL_TRACO_RECEBE,
        "o blit do Flip não recebe a luz como o sprite: {pior:.2e}"
    );
}
