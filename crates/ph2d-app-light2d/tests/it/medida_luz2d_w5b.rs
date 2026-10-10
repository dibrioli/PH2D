//! **A rodada da W5b — o mundo FORA da grade para a luz direta** (F4 da fila do dono,
//! `docs/Luz2D/01_plano.md` §7.1; método `docs/DevOps/MEDIR_VELOCIDADE.md`): um lote, as variantes no
//! MESMO processo (configurações do solver, nunca `const` recompilada), intercaladas com a ordem
//! rodada, vale o MÍNIMO. À mão:
//!
//! ```text
//! PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-light2d --profile smoke --test it \
//!     medida_luz2d_w5b -- --ignored --nocapture --test-threads=1
//! ```
//!
//! A. o custo de placa por nível × variante × cena, na vista de fábrica (tudo na grade) e na do dono
//!    (1,5 m: o muro e a lâmpada grande fora dela), com os opacos como quads e como sprites por texel,
//!    e com 7 · 8 lâmpadas diretas fora da grade (o teto `LAMPADAS_DIRETAS_MAX`);
//! B. a precisão: o caso do dono (W5a × W5b) e a fenda de um sprite fora da tela por lado da máscara;
//! C. o PISO das curas recusadas, pelo gabarito (CPU, exato): (a) o mapa de sombra polar vê do CENTRO
//!    da lâmpada — o disco como um ponto; (b) a captura grossa e conservadora — o opaco dilatado por
//!    ½ e 1 texel dela, com o texel da caixa (vista ∪ lâmpada) ÷ 2048.

use bevy_ecs::world::World;
use ph2d_app_light2d::{
    captura_do_quadro, fina_do_quadro, grade_do_quadro, qualidade::degrau_do, smoke,
};
use ph2d_editor_core::luz_edits::NivelDaLuz;
use ph2d_gpu::GpuContext;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{CenaDeLuz, Emissor, Oclusor};
use ph2d_light2d_gpu::{Amostragem, CapturaDeSprites, Config, Quem, Solver, SpriteNaC0, TemposMs};
use ph2d_render::Camera2d;
use vello::kurbo::Affine;

use crate::comum::{fluencia_lida, gpu};
use crate::medida_luz2d_w4b::loadavg;

const FHD: (u32, u32) = (1920, 1080);
const RODADAS: usize = 9;
const NIVEIS: [(&str, NivelDaLuz); 3] = [
    ("Low", NivelDaLuz::Baixa),
    ("Medium", NivelDaLuz::Media),
    ("High", NivelDaLuz::Alta),
];
/// A vista do dono (a foto da cena 4 em que a luz da lâmpada grande passava o muro).
const DONO: ([f32; 2], f32) = ([-2.6, -2.0], 1.5);

fn afim(vista: ([f32; 2], f32)) -> Affine {
    Camera2d::new(vista.0, vista.1).world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

fn fabrica() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// A configuração do PRODUTO (a do `LuzDoQuadro::gravar`) para o nível e a vista, com a variante.
fn config(gpu: &GpuContext, nivel: NivelDaLuz, a: Affine, distantes: bool, mascara: u32) -> Config {
    let d = degrau_do(nivel);
    Config {
        fina: fina_do_quadro(a, d),
        captura: captura_do_quadro(a, d),
        distantes,
        mascara_px: mascara,
        ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, d).expect("grade"))
    }
}

fn solver(gpu: &GpuContext, cfg: Config, a: Affine) -> Solver {
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (i, f) = ph2d_app_light2d::regiao_da_janela(s.grade_da_fluencia(), a, FHD);
    s.regiao_fina(i, f);
    s
}

/// A cena 4 sem a ambiente, com `extra` lâmpadas pequenas na sala da grande (fora da grade na vista
/// do dono) — o teto de lâmpadas diretas.
fn cena4(extra: usize) -> World {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 4));
    let amb = w
        .query::<(bevy_ecs::entity::Entity, &ph2d_ecs::Name)>()
        .iter(&w)
        .find(|(_, n)| n.0 == "Luz ambiente")
        .map(|(e, _)| e)
        .expect("ambiente");
    w.despawn(amb);
    for k in 0..extra {
        let mut t = ph2d_ecs::Transform::IDENTITY;
        t.translation.x = 1.5 + 0.7 * k as f32;
        t.translation.y = if k % 2 == 0 { 2.5 } else { -3.0 };
        w.spawn((
            ph2d_light2d_ecs::Luz {
                intensidade: 3.0,
                tamanho_m: 0.15,
                ..ph2d_light2d_ecs::Luz::default()
            },
            t,
            ph2d_ecs::GlobalTransform::from_transform(t),
        ));
    }
    w
}

/// Uma textura `l × a` com estes píxeis RGBA8.
fn textura(gpu: &GpuContext, l: u32, a: u32, rgba: &[u8]) -> wgpu::Texture {
    let t = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("medida w5b"),
        size: wgpu::Extent3d {
            width: l,
            height: a,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    gpu.queue.write_texture(
        t.as_image_copy(),
        rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * l),
            rows_per_image: Some(a),
        },
        wgpu::Extent3d {
            width: l,
            height: a,
            depth_or_array_layers: 1,
        },
    );
    t
}

/// Os oclusores da cena como SPRITES por texel de uma textura opaca (o caminho do produto: a
/// máscara do alfa), o quad deles substituído.
fn como_sprites<'a>(cena: &CenaDeLuz, vista: &'a wgpu::TextureView) -> CapturaDeSprites<'a> {
    let mut c = CapturaDeSprites {
        texturas: vec![(vista, Amostragem::Bilinear)],
        ..Default::default()
    };
    for (k, o) in cena.oclusores.iter().enumerate() {
        let v = &o.vertices;
        let p = |i: usize| [f64::from(v[i][0]), f64::from(v[i][1])];
        let (o3, o2, o0) = (p(3), p(2), p(0));
        c.sprites.push(SpriteNaC0 {
            origem: o3,
            eixo_s: [o2[0] - o3[0], o2[1] - o3[1]],
            eixo_t: [o0[0] - o3[0], o0[1] - o3[1]],
            uv: [0.0, 0.0, 1.0, 1.0],
            emissao: [0.0; 3],
            opacidade: o.opacidade,
            premultiplicada: false,
            textura: 0,
        });
        c.substituidos.insert(Quem::Oclusor(k));
    }
    c
}

#[test]
#[ignore = "a rodada da W5b: corre-se à mão (cabeçalho)"]
fn medida_luz2d_w5b() {
    let Some(gpu) = gpu() else {
        eprintln!("medida: sem placa");
        return;
    };
    eprintln!(
        "# Rodada W5b · {} · perfil {} · loadavg antes {}",
        gpu.adapter.get_info().name,
        if cfg!(debug_assertions) {
            "DEBUG"
        } else {
            "otimizado (smoke)"
        },
        loadavg()
    );
    custo(&gpu);
    precisao(&gpu);
    piso_das_recusas();
    eprintln!("# loadavg depois {}", loadavg());
}

/// A. O custo de placa.
fn custo(gpu: &GpuContext) {
    let branca = textura(gpu, 1, 1, &[255; 4]);
    let vb = branca.create_view(&Default::default());
    let cenas: Vec<(&str, CenaDeLuz, bool)> = vec![
        ("2 lâmpadas · quads", cena_de_luz(&cena4(0)), false),
        ("2 · sprites", cena_de_luz(&cena4(0)), true),
        ("7 · sprites", cena_de_luz(&cena4(5)), true),
        ("8 · sprites", cena_de_luz(&cena4(6)), true),
    ];
    let mut variantes: Vec<(String, Solver)> = Vec::new();
    for (nome, nivel) in NIVEIS {
        let (af, ad) = (fabrica(), afim(DONO));
        variantes.push((
            format!("{nome} · fábrica · W5a"),
            solver(gpu, config(gpu, nivel, af, false, 256), af),
        ));
        variantes.push((
            format!("{nome} · fábrica · W5b"),
            solver(gpu, config(gpu, nivel, af, true, 256), af),
        ));
        variantes.push((
            format!("{nome} · dono · W5a (o defeito)"),
            solver(gpu, config(gpu, nivel, ad, false, 256), ad),
        ));
        variantes.push((
            format!("{nome} · dono · W5b máscara 256"),
            solver(gpu, config(gpu, nivel, ad, true, 256), ad),
        ));
        variantes.push((
            format!("{nome} · dono · W5b máscara 512"),
            solver(gpu, config(gpu, nivel, ad, true, 512), ad),
        ));
    }
    let n = variantes.len() * cenas.len();
    let mut minimo: Vec<Option<TemposMs>> = vec![None; n];
    let mut objetos = vec![(0, 0); n];
    for rodada in 0..RODADAS {
        for k in 0..n {
            let i = (k + rodada * 7) % n;
            let (v, c) = (i / cenas.len(), i % cenas.len());
            let (_, cena, por_texel) = &cenas[c];
            let sp = if *por_texel {
                como_sprites(cena, &vb)
            } else {
                CapturaDeSprites::default()
            };
            let s = &mut variantes[v].1;
            let mut r = Default::default();
            for _ in 0..2 {
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                r = s.gravar_com_sprites(gpu, &mut enc, cena, &sp);
                gpu.queue.submit([enc.finish()]);
            }
            let r: ph2d_light2d_gpu::RelatorioC0 = r;
            objetos[i] = (r.distantes, r.mascaras);
            let t = s.tempos(gpu).expect("TIMESTAMP_QUERY");
            if minimo[i].as_ref().is_none_or(|m| t.total() < m.total()) {
                minimo[i] = Some(t);
            }
        }
    }
    eprintln!("\n## A. Custo de placa (ms, MÍNIMO de {RODADAS} rodadas intercaladas), 1920×1080");
    eprintln!(
        "| variante | {} | C3 vis · fina · máscaras (8 · sprites) | objetos · máscaras (8) | MB alocados |",
        cenas.iter().map(|c| c.0).collect::<Vec<_>>().join(" | ")
    );
    eprintln!("|---|{}---|---|---|", "---|".repeat(cenas.len()));
    for (v, (nome, s)) in variantes.iter().enumerate() {
        let t = |c: usize| minimo[v * cenas.len() + c].clone().expect("medido");
        let tot: Vec<String> = (0..cenas.len())
            .map(|c| format!("{:.2}", t(c).total()))
            .collect();
        let t8 = t(3);
        eprintln!(
            "| {nome} | {} | {:.2} · {:.2} · {:.3} | {:?} | {:.1} |",
            tot.join(" | "),
            t8.c3_visibilidade,
            t8.c3_fina,
            t8.c3_mascaras,
            objetos[v * cenas.len() + 3],
            s.pass_list().bytes_alocados() as f64 / 1e6
        );
    }
    eprintln!(
        "(orçamento do governador: {:.2} ms)",
        ph2d_app_light2d::qualidade::ORCAMENTO_MS
    );
}

/// O pior erro de `f` nos `pontos` contra o gabarito, da maior luz `escala`.
fn pior(f: &crate::comum::Fluencia, pontos: &[[f64; 2]], gab: &[f64], escala: f64) -> f64 {
    pontos
        .iter()
        .zip(gab)
        .map(|(&p, g)| (f.amostra(p) - g).abs() / escala)
        .fold(0.0, f64::max)
}

/// B. A precisão.
fn precisao(gpu: &GpuContext) {
    eprintln!("\n## B. Precisão (pior erro de um ponto, da luz sem o opaco; Low)");
    // 1. O caso do dono: o pan da vista de 1,5 m, W5a × W5b.
    let w = cena4(0);
    let cena = cena_de_luz(&w);
    let pontos = [[-2.8, -1.0], [-2.5, -1.8], [-2.2, -2.3], [-1.2, -2.0]];
    let gab: Vec<f64> = pontos
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&cena, p)[1])
        .collect();
    let sem: CenaDeLuz = {
        let mut c = cena.clone();
        c.oclusores
            .retain(|o| o.vertices.iter().all(|v| v[0].abs() > 0.11));
        c
    };
    let escala = pontos
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&sem, p)[1])
        .fold(0.0, f64::max);
    for (nome, distantes) in [("W5a (sem distantes)", false), ("W5b", true)] {
        let mut e = 0.0_f64;
        for i in 0..=8 {
            let a = afim(([-3.4 + 0.2 * i as f32, -2.0], 1.5));
            let mut s = solver(gpu, config(gpu, NivelDaLuz::Baixa, a, distantes, 256), a);
            s.quadro(gpu, &cena);
            e = e.max(pior(&fluencia_lida(gpu, &s), &pontos, &gab, escala));
        }
        eprintln!("- o muro na vista do dono (pan de 9 vistas), {nome}: **{e:.4}**");
    }
    // 2. A fenda de um sprite fora da tela, por lado da máscara.
    let lampada = Emissor::Disco {
        centro: [-8.0, 0.0],
        raio: 0.2,
        radiancia: [1.0; 3],
        alcance_m: f32::INFINITY,
    };
    let ret = |a: f32, b: f32| Oclusor {
        vertices: vec![[-5.2, a], [-4.8, a], [-4.8, b], [-5.2, b]],
        opacidade: 1.0,
        albedo: [0.0; 3],
    };
    let gabarito = CenaDeLuz {
        emissores: vec![lampada.clone()],
        oclusores: vec![ret(-1.5, -0.1), ret(0.1, 1.5)],
        luzes: 1,
        ..Default::default()
    };
    let quad = CenaDeLuz {
        oclusores: vec![ret(-1.5, 1.5)],
        ..gabarito.clone()
    };
    let so_lampada = CenaDeLuz {
        oclusores: vec![],
        ..gabarito.clone()
    };
    let pontos = [
        [1.2, -0.6],
        [1.2, -0.3],
        [1.5, 0.0],
        [1.5, 0.2],
        [1.8, 0.4],
        [1.8, 0.7],
    ];
    let gab: Vec<f64> = pontos
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&gabarito, p)[1])
        .collect();
    let escala = pontos
        .iter()
        .map(|&p| ph2d_light2d_ref::fluencia(&so_lampada, p)[1])
        .fold(0.0, f64::max);
    let rgba: Vec<u8> = (0..15)
        .flat_map(|j| [90, 90, 90, if j == 7 { 0 } else { 255 }])
        .collect();
    let t = textura(gpu, 1, 15, &rgba);
    let vt = t.create_view(&Default::default());
    let sp = como_sprites(&quad, &vt);
    for lado in [16, 32, 64, 128, 256, 512] {
        let mut e = 0.0_f64;
        for i in 0..=8 {
            let a = afim(([0.9 + 0.15 * i as f32, 0.0], 1.5));
            let mut s = solver(gpu, config(gpu, NivelDaLuz::Baixa, a, true, lado), a);
            let mut enc = gpu.device.create_command_encoder(&Default::default());
            s.gravar_com_sprites(gpu, &mut enc, &quad, &sp);
            gpu.queue.submit([enc.finish()]);
            e = e.max(pior(&fluencia_lida(gpu, &s), &pontos, &gab, escala));
        }
        eprintln!("- a fenda do sprite (1×15, a fenda de 1 linha), máscara {lado}: **{e:.4}**");
    }
}

/// O polígono `v` dilatado por `h` (cada aresta empurrada para fora — um retângulo alinhado).
fn dilatado(o: &Oclusor, h: f32) -> Oclusor {
    let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for v in &o.vertices {
        lo = [lo[0].min(v[0]), lo[1].min(v[1])];
        hi = [hi[0].max(v[0]), hi[1].max(v[1])];
    }
    let (lo, hi) = ([lo[0] - h, lo[1] - h], [hi[0] + h, hi[1] + h]);
    Oclusor {
        vertices: vec![lo, [hi[0], lo[1]], hi, [lo[0], hi[1]]],
        opacidade: o.opacidade,
        albedo: [0.0; 3],
    }
}

/// C. O piso das curas recusadas, pelo gabarito: os pontos da parede da grande (penumbra da lâmpada
/// grande) e a mesma geometria com a lâmpada a 50 m.
fn piso_das_recusas() {
    eprintln!(
        "\n## C. O piso das curas recusadas (gabarito, CPU) — pior erro, da luz sem a parede"
    );
    let w = cena4(0);
    let cena = cena_de_luz(&w);
    let pontos = [
        [4.8, -1.6],
        [5.1, -2.2],
        [5.4, -1.0],
        [5.4, -2.2],
        [5.7, -1.0],
    ];
    let parede = cena
        .oclusores
        .iter()
        .position(|o| o.vertices.iter().all(|v| (2.39..=2.61).contains(&v[0])))
        .expect("a parede da grande");
    let fl = |c: &CenaDeLuz, p: [f64; 2]| ph2d_light2d_ref::fluencia(c, p)[1];
    let sem = CenaDeLuz {
        oclusores: cena
            .oclusores
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != parede)
            .map(|(_, o)| o.clone())
            .collect(),
        ..cena.clone()
    };
    let gab: Vec<f64> = pontos.iter().map(|&p| fl(&cena, p)).collect();
    let escala = pontos.iter().map(|&p| fl(&sem, p)).fold(0.0, f64::max);
    let erro = |c: &CenaDeLuz| {
        pontos
            .iter()
            .zip(&gab)
            .map(|(&p, g)| (fl(c, p) - g).abs() / escala)
            .fold(0.0, f64::max)
    };
    // (a) o mapa polar vê do CENTRO: a lâmpada como um ponto da mesma força ao longe (F = 2·R·L/d).
    let como_ponto = CenaDeLuz {
        emissores: cena
            .emissores
            .iter()
            .map(|e| match e {
                Emissor::Disco {
                    centro,
                    raio,
                    radiancia,
                    alcance_m,
                } => Emissor::Ponto {
                    centro: *centro,
                    intensidade: radiancia.map(|r| r * 2.0 * raio),
                    alcance_m: *alcance_m,
                },
                e => e.clone(),
            })
            .collect(),
        ..cena.clone()
    };
    eprintln!(
        "- (a) o mapa polar do centro (resolução infinita), a lâmpada de 0,6 m: **{:.3}**",
        erro(&como_ponto)
    );
    // (b) a captura grossa e conservadora: o texel da caixa (vista do dono ∪ lâmpada) ÷ 2048.
    for (rotulo, caixa) in [
        ("lâmpada a ~6 m (a caixa da cena 4, 6,7 m)", 6.7_f32),
        ("lâmpada a 50 m", 50.0),
    ] {
        let h = caixa / 2048.0;
        for (q, f) in [("½ texel", 0.5_f32), ("1 texel", 1.0)] {
            let mut c = cena.clone();
            c.oclusores[parede] = dilatado(&cena.oclusores[parede], h * f);
            eprintln!(
                "- (b) captura conservadora, {rotulo}: texel {:.1} mm, dilatada {q}: **{:.4}**",
                h * 1000.0,
                erro(&c)
            );
        }
    }
}

/// A2 à parte (a 1.ª rodada leu até +12 % entre duas variantes do MESMO caminho — o ruído ou o custo
/// do `DISTANTES` com as listas vazias?): só a vista de fábrica, W5a × W5b, 25 rodadas.
#[test]
#[ignore = "a rodada da W5b, A2: corre-se à mão"]
fn medida_luz2d_w5b_vazio() {
    let Some(gpu) = gpu() else { return };
    eprintln!("# W5b A2 · loadavg antes {}", loadavg());
    let branca = textura(&gpu, 1, 1, &[255; 4]);
    let vb = branca.create_view(&Default::default());
    let cenas = [cena_de_luz(&cena4(0)), cena_de_luz(&cena4(6))];
    let mut variantes: Vec<(String, Solver)> = Vec::new();
    for (nome, nivel) in [NIVEIS[0], NIVEIS[2]] {
        let af = fabrica();
        for (v, d) in [("W5a", false), ("W5b", true)] {
            variantes.push((
                format!("{nome} · fábrica · {v}"),
                solver(&gpu, config(&gpu, nivel, af, d, 256), af),
            ));
        }
    }
    let n = variantes.len() * cenas.len();
    let mut minimo = vec![f64::INFINITY; n];
    for rodada in 0..25 {
        for k in 0..n {
            let i = (k + rodada * 3) % n;
            let (v, c) = (i / cenas.len(), i % cenas.len());
            let sp = como_sprites(&cenas[c], &vb);
            let s = &mut variantes[v].1;
            for _ in 0..2 {
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                s.gravar_com_sprites(&gpu, &mut enc, &cenas[c], &sp);
                gpu.queue.submit([enc.finish()]);
            }
            minimo[i] = minimo[i].min(s.tempos(&gpu).expect("TIMESTAMP_QUERY").total());
        }
    }
    eprintln!("| variante | 2 lâmpadas | 8 lâmpadas |\n|---|---|---|");
    for (v, (nome, _)) in variantes.iter().enumerate() {
        eprintln!(
            "| {nome} | {:.3} | {:.3} |",
            minimo[v * 2],
            minimo[v * 2 + 1]
        );
    }
    eprintln!("# loadavg depois {}", loadavg());
}

/// A3 à parte: o custo com N sprites FORA da grade no caminho da lâmpada (o laço da C3 é por objeto
/// em cada raio e em cada cone). A cena 4 na vista do dono, mais N caixotes de 0,15 m espalhados na
/// sala da grande (fora da grade, dentro do caminho até a lâmpada grande).
#[test]
#[ignore = "a rodada da W5b, A3: corre-se à mão"]
fn medida_luz2d_w5b_muitos() {
    let Some(gpu) = gpu() else { return };
    eprintln!("# W5b A3 · loadavg antes {}", loadavg());
    let branca = textura(&gpu, 1, 1, &[255; 4]);
    let vb = branca.create_view(&Default::default());
    let base = cena_de_luz(&cena4(0));
    let com = |n: usize| {
        let mut c = base.clone();
        for k in 0..n {
            // Uma espiral determinística na sala da grande (x ∈ [0,3; 2,3], y ∈ [−3,5; 3,5]).
            let a = k as f32 * 2.399_963;
            let r = (k as f32 + 0.5).sqrt() / (n as f32).sqrt();
            let (x, y) = (1.3 + r * a.cos() * 0.95, r * a.sin() * 3.4);
            let l = 0.075;
            c.oclusores.push(Oclusor {
                vertices: vec![
                    [x - l, y - l],
                    [x + l, y - l],
                    [x + l, y + l],
                    [x - l, y + l],
                ],
                opacidade: 1.0,
                albedo: [0.0; 3],
            });
        }
        c
    };
    let ns = [0_usize, 10, 50, 200];
    let cenas: Vec<CenaDeLuz> = ns.iter().map(|&n| com(n)).collect();
    let ad = afim(DONO);
    let mut s = solver(&gpu, config(&gpu, NivelDaLuz::Baixa, ad, true, 256), ad);
    let mut minimo = vec![f64::INFINITY; ns.len()];
    let mut objetos = vec![0; ns.len()];
    for rodada in 0..15 {
        for k in 0..ns.len() {
            let i = (k + rodada) % ns.len();
            let sp = como_sprites(&cenas[i], &vb);
            for _ in 0..2 {
                let mut enc = gpu.device.create_command_encoder(&Default::default());
                objetos[i] = s
                    .gravar_com_sprites(&gpu, &mut enc, &cenas[i], &sp)
                    .distantes;
                gpu.queue.submit([enc.finish()]);
            }
            minimo[i] = minimo[i].min(s.tempos(&gpu).expect("TIMESTAMP_QUERY").total());
        }
    }
    eprintln!("| caixotes fora da grade | objetos distantes | ms (Low, dono) |\n|---|---|---|");
    for (k, n) in ns.iter().enumerate() {
        eprintln!("| {n} | {} | {:.3} |", objetos[k], minimo[k]);
    }
    eprintln!("# loadavg depois {}", loadavg());
}
