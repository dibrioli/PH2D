//! ⭐ W5s — a traça da holográfica que SALTA o vazio (`Config::hrc_salta_o_vazio`) e um fio por sonda
//! (`Config::hrc_fio_por_sonda`): o intervalo curto (T0–T2) que só cruza ladrilhos sem cobertura da captura
//! é o vácuo exato. As réguas: quantos raios são
//! vazios (o contador na placa, que saiu da árvore: handoff W5s §1) e os cortes contra o antes no MESMO
//! processo — a fluência ao BIT e o relógio da `c2` (o mínimo de 7 rodadas intercaladas). Os casos: os da W5q (cenas
//! 1–7), o traço do Flip e as formas do Motion pela CAMADA (o contrato `CamadaDeOclusao`, discos de área
//! exata), sprites com alfa (um que tapa e um que brilha) e a cena CHEIA (todo ladrilho com cobertura).

use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_light2d_ecs::CenaDeLuz;
use ph2d_light2d_gpu::{
    Amostragem, CamadaDeOclusao, CapturaDeSprites, Config, FORMATO_DA_OCLUSAO, Grade, SpriteNaC0,
};
use vello::kurbo;

use crate::medida_luz2d_w5o::{mediana, minimo};
use crate::medida_luz2d_w5p::Vista;
use crate::o_traco_do_flip::DiscosExatos;

type Discos = Vec<([f64; 2], f64, [u8; 3])>;

/// Um caso: as cenas das duas metades dos quadros, a câmera, os discos da camada de uma família (vazio =
/// nenhuma) e se leva os dois sprites com alfa.
pub(crate) struct Caso {
    pub(crate) rotulo: String,
    pub(crate) cena: CenaDeLuz,
    pub(crate) cena2: CenaDeLuz,
    pub(crate) a: kurbo::Affine,
    discos: Discos,
    sprites: bool,
}

fn cena_de(n: u32) -> CenaDeLuz {
    let mut w = bevy_ecs::world::World::new();
    assert!(ph2d_app_light2d::smoke::encenar(&mut w, n));
    ph2d_light2d_ecs::coleta::cena_de_luz(&w)
}

/// As formas do Motion da cena 8 como a camada da família as daria: a «Pile» (discos empilhados sob a
/// lâmpada) e o «Swarm» (cópias pequenas espalhadas).
fn formas_do_motion() -> Discos {
    let mut d = Vec::new();
    for i in 0..12_u32 {
        let (col, lin) = (f64::from(i % 4), f64::from(i / 4));
        d.push((
            [0.4 + 0.55 * col + 0.2 * lin, -1.0 + 0.5 * lin],
            0.18 + 0.03 * f64::from(i % 3),
            [200, 90 + 10 * (i % 5) as u8, 60],
        ));
    }
    for i in 0..30_u32 {
        let t = f64::from(i);
        d.push((
            [3.0 + 0.09 * t, 1.8 + 0.6 * (0.7 * t).sin()],
            0.05,
            [80, 160, 220],
        ));
    }
    d
}

pub(crate) fn casos() -> Vec<Caso> {
    let mut v: Vec<Caso> = crate::medida_luz2d_w5q::casos()
        .into_iter()
        .map(|(rotulo, cena, cena2, a)| Caso {
            rotulo,
            cena,
            cena2,
            a,
            discos: Vec::new(),
            sprites: false,
        })
        .collect();
    let fabrica = crate::medida_luz2d_w5e::fabrica((1920, 1080));
    let mut flip: Discos = ph2d_app_light2d::smoke::DISCOS_DE_TINTA
        .iter()
        .map(|d| (d.centro, d.raio, d.cor))
        .collect();
    flip.extend(
        ph2d_app_light2d::smoke::onda_a_mao()
            .into_iter()
            .map(|(c, r)| (c, r, [30, 30, 40])),
    );
    v.push(Caso {
        rotulo: "cena 7 com o traço do Flip".into(),
        cena: cena_de(7),
        cena2: cena_de(7),
        a: fabrica,
        discos: flip,
        sprites: false,
    });
    v.push(Caso {
        rotulo: "cena 8 com as formas do Motion e sprites com alfa".into(),
        cena: cena_de(8),
        cena2: cena_de(8),
        a: fabrica,
        discos: formas_do_motion(),
        sprites: true,
    });
    // A cena CHEIA: a cena 1 sob um TAPETE de discos que acende todo ladrilho da grade (`camadas`) — o
    // pior caso do salto (nenhum raio salta; a máscara custa).
    v.push(Caso {
        rotulo: "cena cheia".into(),
        cena: cena_de(1),
        cena2: cena_de(1),
        a: fabrica,
        discos: Vec::new(),
        sprites: true,
    });
    v
}

fn textura(gpu: &ph2d_gpu::GpuContext, (w, h): (u32, u32)) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("w5s camada"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMATO_DA_OCLUSAO,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

/// As texturas de um caso na grade da vista: a camada da família e o disco dos sprites.
pub(crate) struct Camadas {
    grade: Grade,
    camada: Option<wgpu::TextureView>,
    disco: Option<wgpu::TextureView>,
}

pub(crate) fn camadas(gpu: &ph2d_gpu::GpuContext, v: &Vista, c: &Caso) -> Camadas {
    let cfg = v.vista.config();
    let g = cfg.grade;
    let mundo = [
        g.origem_m[0],
        g.origem_m[1],
        g.origem_m[0] + f64::from(g.largura) * g.texel_m,
        g.origem_m[1] + f64::from(g.altura) * g.texel_m,
    ];
    let pintar = |tamanho: (u32, u32), mundo: [f64; 4], discos: Discos| {
        let t = textura(gpu, tamanho);
        let vista = t.create_view(&Default::default());
        let mut f = DiscosExatos {
            entidade: 1,
            discos,
        };
        assert!(f.desenhar(gpu, &vista, tamanho, mundo, &[1]));
        vista
    };
    let fator = cfg.captura.max(1);
    // O tapete da cena cheia: um disco de raio 1,5 texel a cada 6 texels da grade (todo ladrilho 8×8 tem
    // cobertura).
    let mut discos = c.discos.clone();
    if c.rotulo == "cena cheia" {
        for i in (3..g.largura).step_by(6) {
            for j in (3..g.altura).step_by(6) {
                discos.push((g.centro(i, j), 1.5 * g.texel_m, [120, 120, 130]));
            }
        }
    }
    let camada =
        (!discos.is_empty()).then(|| pintar((g.largura * fator, g.altura * fator), mundo, discos));
    let disco = c.sprites.then(|| {
        pintar(
            (64, 64),
            [0.0, 0.0, 1.0, 1.0],
            vec![([0.5, 0.5], 0.45, [255; 3])],
        )
    });
    Camadas {
        grade: g,
        camada,
        disco,
    }
}

/// Os sprites da C0 de um caso: a camada cobre a grade (como `outras.rs`); os do disco são um que tapa
/// e um que brilha.
pub(crate) fn sprites(c: &Camadas) -> CapturaDeSprites<'_> {
    let g = c.grade;
    let mut s = CapturaDeSprites {
        texturas: Vec::new(),
        sprites: Vec::new(),
        substituidos: Default::default(),
        distantes: Vec::new(),
    };
    let base = SpriteNaC0 {
        origem: [0.0; 2],
        eixo_s: [0.0; 2],
        eixo_t: [0.0; 2],
        uv: [0.0, 0.0, 1.0, 1.0],
        emissao: [0.0; 3],
        opacidade: 1.0,
        premultiplicada: true,
        textura: 0,
        albedo: [1.0; 3],
        cor_em_srgb: false,
    };
    if let Some(v) = &c.camada {
        let (lx, ly) = (
            f64::from(g.largura) * g.texel_m,
            f64::from(g.altura) * g.texel_m,
        );
        s.sprites.push(SpriteNaC0 {
            origem: [g.origem_m[0], g.origem_m[1] + ly],
            eixo_s: [lx, 0.0],
            eixo_t: [0.0, -ly],
            textura: s.texturas.len(),
            ..base
        });
        s.texturas.push((v, Amostragem::Exata));
    }
    if let Some(v) = &c.disco {
        let k = s.texturas.len();
        s.texturas.push((v, Amostragem::Bilinear));
        for (centro, emissao) in [([-3.5, -1.2], [0.0; 3]), ([4.2, -1.5], [3.0, 1.5, 0.6])] {
            s.sprites.push(SpriteNaC0 {
                origem: [centro[0] - 0.6, centro[1] + 0.6],
                eixo_s: [1.2, 0.0],
                eixo_t: [0.0, -1.2],
                emissao,
                textura: k,
                ..base
            });
        }
    }
    s
}

fn variante(salta: bool, sonda: bool) -> impl Fn(Config) -> Config {
    move |c: Config| Config {
        hrc_salta_o_vazio: salta,
        hrc_fio_por_sonda: sonda,
        ..c
    }
}

/// `(rótulo, salta o vazio, um fio por sonda)`: o antes, cada corte sozinho e os dois.
const VARIANTES: [(&str, bool, bool); 4] = [
    ("antes", false, false),
    ("salta", true, false),
    ("sonda", false, true),
    ("os dois", true, true),
];

/// ⭐ W5s — os dois cortes da traça contra o antes, no MESMO processo: a fluência ao BIT (80 quadros)
/// e o relógio da `c2` (o mínimo de 7 rodadas intercaladas). Controlo: os dois sem a máscara escrita
/// (todo raio salta) mudam bytes.
#[test]
#[ignore = "rodada de medição W5s (placa): os cortes da traça ao bit e o custo"]
fn medida_luz2d_w5s_salta() {
    let Some(gpu) = crate::comum::gpu() else {
        return;
    };
    eprintln!(
        "W5s salta · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
    let dif = |x: &[u8], y: &[u8]| x.iter().zip(y).filter(|(a, b)| a != b).count();
    let mut todos_ao_bit = true;
    let mut vistos = 0;
    for nivel in [NivelDaLuz::Baixa, NivelDaLuz::Alta] {
        for c in &casos() {
            let mut vs =
                VARIANTES.map(|(_, s, f)| Vista::nova_com(&gpu, c.a, nivel, &variante(s, f)));
            let cam = camadas(&gpu, &vs[0], c);
            let sp = sprites(&cam);
            let correr = |v: &mut Vista| {
                for q in 0..80 {
                    v.quadro_com(&gpu, if q < 40 { &c.cena } else { &c.cena2 }, &sp);
                }
            };
            vs.iter_mut().for_each(correr);
            let bytes = vs
                .each_ref()
                .map(|v| crate::comum::ler(&gpu, v.vista.fluencia(), 8));
            let diferentes: Vec<usize> = bytes[1..].iter().map(|b| dif(&bytes[0], b)).collect();
            todos_ao_bit &= diferentes.iter().all(|&d| d == 0);
            let mut m = Vista::nova_com(&gpu, c.a, nivel, &variante(true, true));
            m.vista.hrc_sem_mascara();
            correr(&mut m);
            let controlo = dif(&crate::comum::ler(&gpu, m.vista.fluencia(), 8), &bytes[0]);
            vistos += usize::from(controlo > 0);
            drop(m);
            let mut c2: [Vec<f64>; 4] = Default::default();
            for rodada in 0..7 {
                for j in 0..4 {
                    let i = (j + rodada) % 4;
                    for _ in 0..3 {
                        vs[i].quadro_com(&gpu, &c.cena2, &sp);
                    }
                    let t = vs[i].vista.tempos(&gpu).expect("TIMESTAMP_QUERY");
                    c2[i].push(t.c2_total());
                }
            }
            let tempos: Vec<String> = VARIANTES
                .iter()
                .zip(&c2)
                .map(|((r, _, _), t)| format!("{r} {:.3} ({:.3})", minimo(t), mediana(t)))
                .collect();
            eprintln!(
                "{nivel:?} · {} · bytes diferentes {diferentes:?} de {} (controlo {controlo}) · c2 mín (med) {} ms",
                c.rotulo,
                bytes[0].len(),
                tempos.join(" · ")
            );
        }
    }
    assert!(
        vistos > 0,
        "o controlo (sem a máscara) não mudou nenhum caso: a régua não vê o salto"
    );
    eprintln!(
        "W5s salta · ao bit em todos: {todos_ao_bit} · o controlo mudou {vistos} casos · load {}",
        crate::medida_luz2d_w4b::loadavg().trim()
    );
}
