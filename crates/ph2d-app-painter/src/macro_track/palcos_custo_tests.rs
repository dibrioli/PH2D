//! **SONDA — o custo do quadro com N palcos do Painter a tocar** (W4, a prova do bloco; régua
//! `docs/DevOps/MEDIR_VELOCIDADE.md`): N sprites com o Splash da cena `PH2D_MACRO_SMOKE=1` (1024² e
//! 2048²) conduzidas no mesmo instante — `PalcosDoPainter::segue` + `sobe` para um `SpriteRenderer`
//! real, a placa sincronizada no fim do quadro; blocos intercalados por ordem rodada, o MÍNIMO.
//! O trabalho muda de quadro para quadro (a água gravada vem aos lotes), logo cada bloco repete o MESMO
//! troço da macro — um recuo fora da régua — e a régua é a SOMA do bloco, nunca um quadro.
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-painter --profile smoke --lib --
//! --ignored --nocapture macro_track::palcos_custo`

use super::PalcosDoPainter;
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, SimWorld};
use ph2d_editor_core::toast::ToastQueue;
use ph2d_gpu::GpuContext;
use ph2d_render::SpriteRenderer;
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;

const LADOS: [u32; 2] = [1024, 2048];
const NS: [usize; 3] = [1, 4, 16];
const RODADAS: usize = 7;
const BLOCO: usize = 20;
/// O troço medido começa aqui (a 1.ª pincelada a pousar; a água corre até ao fim).
const INICIO_S: f64 = 0.2;

struct Variante {
    lado: u32,
    n: usize,
    /// A sprite branca do produto (o papel: a composição parcial).
    no_papel: bool,
    sim: SimWorld,
    palcos: PalcosDoPainter,
    t: f64,
    /// Por bloco: (segue, sobe) em ms por quadro.
    ms: Vec<(f64, f64)>,
}

fn quadro(v: &mut Variante, r: &mut SpriteRenderer, gpu: &GpuContext) -> (f64, f64) {
    let t = v.t;
    let mut q = v.sim.world_mut().query::<&mut MacroTempo>();
    for mut m in q.iter_mut(v.sim.world_mut()) {
        m.segundos = t;
    }
    let t0 = std::time::Instant::now();
    v.palcos.segue(v.sim.world_mut());
    let segue = t0.elapsed().as_secs_f64() * 1e3;
    let trocas = v
        .palcos
        .sobe(r, &mut ToastQueue::default(), None, Default::default());
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    let total = t0.elapsed().as_secs_f64() * 1e3;
    assert_eq!(trocas.len(), v.n, "cada palco troca a sua sprite");
    v.t += 1.0 / 60.0;
    (segue, total - segue)
}

fn min_med(v: &[f64]) -> (f64, f64) {
    let mut s = v.to_vec();
    s.sort_by(f64::total_cmp);
    (s[0], s[s.len() / 2])
}

#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_n_palcos_do_painter() {
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let mut vs = Vec::new();
    for lado in LADOS {
        let t0 = std::time::Instant::now();
        let (bytes, d) = grava_o_splash(lado, 0).expect("grava");
        eprintln!(
            "{lado}²: gravar {d:.2} s de macro custou {:.0} ms ({} KiB)",
            t0.elapsed().as_secs_f64() * 1e3,
            bytes.len() / 1024
        );
        let papel = ph2d_tool_painter::macro_track::roteiro::grava_no_papel(lado, 0, 4)
            .expect("grava")
            .0;
        for (n, bytes, no_papel) in NS
            .iter()
            .map(|&n| (n, bytes.clone(), false))
            .chain([(1, papel, true)])
        {
            let mut sim = SimWorld::new();
            for _ in 0..n {
                sim.world_mut().spawn((
                    Macro {
                        doc: bytes.clone(),
                        modo: ModoDaMacro::Fiel,
                    },
                    MacroTempo { segundos: INICIO_S },
                ));
            }
            vs.push(Variante {
                lado,
                n,
                no_papel,
                sim,
                palcos: PalcosDoPainter::default(),
                t: INICIO_S,
                ms: Vec::new(),
            });
        }
    }
    // Aquecimento fora da régua: o 1.º quadro toca do zero até ao início e sobe a 1.ª tela.
    for v in &mut vs {
        let t0 = std::time::Instant::now();
        quadro(v, &mut r, &gpu);
        eprintln!(
            "{}² N={:>2}: 1.º quadro {:.1} ms",
            v.lado,
            v.n,
            t0.elapsed().as_secs_f64() * 1e3
        );
        for _ in 0..3 {
            quadro(v, &mut r, &gpu);
        }
    }
    let k = vs.len();
    for rodada in 0..RODADAS {
        for i in 0..k {
            let v = &mut vs[(rodada + i) % k];
            v.t = INICIO_S - 1.0 / 60.0;
            quadro(v, &mut r, &gpu);
            let (mut segue, mut sobe) = (0.0, 0.0);
            for _ in 0..BLOCO {
                let (a, b) = quadro(v, &mut r, &gpu);
                segue += a;
                sobe += b;
            }
            #[expect(clippy::cast_precision_loss, reason = "BLOCO pequeno")]
            let q = BLOCO as f64;
            v.ms.push((segue / q, sobe / q));
        }
    }
    eprintln!(
        "\n   tela | N palcos | mín ms/quadro | mediana | mín por palco | (segue + sobe, no mínimo)"
    );
    for v in &vs {
        let totais: Vec<f64> = v.ms.iter().map(|(a, b)| a + b).collect();
        let (min, med) = min_med(&totais);
        let i = totais.iter().position(|t| *t == min).unwrap_or(0);
        #[expect(clippy::cast_precision_loss, reason = "N pequeno")]
        let por = min / v.n as f64;
        eprintln!(
            "  {:>4}² | {:>8} | {min:>13.3} | {med:>7.3} | {por:>13.3} | {:.3} + {:.3}{}",
            v.lado,
            v.n,
            v.ms[i].0,
            v.ms[i].1,
            if v.no_papel { " · no papel" } else { "" }
        );
    }
    eprintln!("load depois: {}", carga().trim());
}

/// O quadro mais caro do Splash a 1024², do início ao fim e de volta ao início (o laço): onde a cena
/// trava.
#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_o_quadro_mais_caro_do_splash() {
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let (bytes, d) = grava_o_splash(1024, 0).expect("grava");
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Macro {
            doc: bytes,
            modo: ModoDaMacro::Fiel,
        },
        MacroTempo { segundos: 0.0 },
    ));
    let mut v = Variante {
        lado: 1024,
        n: 1,
        no_papel: false,
        sim,
        palcos: PalcosDoPainter::default(),
        t: 0.0,
        ms: Vec::new(),
    };
    let mut vistos = Vec::new();
    while v.t <= d + 0.1 {
        let t = v.t;
        let (a, b) = quadro(&mut v, &mut r, &gpu);
        vistos.push((a + b, a, t));
    }
    v.t = 0.0;
    let (a, b) = quadro(&mut v, &mut r, &gpu);
    eprintln!("o recuo do laço (fim → 0): {:.1} ms (segue {a:.1})", a + b);
    vistos.sort_by(|x, y| y.0.total_cmp(&x.0));
    for (ms, segue, t) in vistos.iter().take(6) {
        eprintln!("t={t:.3} s: {ms:.1} ms (segue {segue:.1})");
    }
}

/// **W5e: subir a tela de um palco** — premultiplicar + substituir a textura INTEIRA (o `sobe` de hoje)
/// contra só um retângulo de `f` da área (recortar + premultiplicar + `replace_individual_pixels_region`),
/// a placa sincronizada; blocos intercalados, o MÍNIMO.
#[test]
#[ignore = "placa: sonda de custo, precisa de adapter de GPU"]
fn sonda_subir_a_tela() {
    const FRACCOES: [f64; 3] = [1.0, 0.25, 0.05];
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("load antes: {}", carga().trim());
    let gpu = GpuContext::new(GpuContext::default_instance(), None).expect("adaptador de GPU");
    let atlas = ph2d_render::TextureAtlas::new(&gpu, 64);
    let mut r = SpriteRenderer::new(gpu.clone(), ph2d_render::GameRt::FORMAT, atlas, 8);
    let mut vs: Vec<(u32, f64, u32, Vec<u8>, f64)> = Vec::new();
    for lado in LADOS {
        let n = (lado * lado * 4) as usize;
        let tela: Vec<u8> = (0..n).map(|i| (i % 251) as u8).collect();
        let id = r.acquire_individual(lado, lado, &tela).expect("ranhura");
        for f in FRACCOES {
            vs.push((lado, f, id, tela.clone(), f64::MAX));
        }
    }
    let mut premul = Vec::new();
    let k = vs.len();
    for rodada in 0..RODADAS {
        for i in 0..k {
            let (lado, f, id, tela, best) = &mut vs[(rodada + i) % k];
            let t0 = std::time::Instant::now();
            for _ in 0..BLOCO {
                if *f >= 1.0 {
                    premul.clear();
                    premul.extend_from_slice(tela);
                    ph2d_render::premul::premultiply_rgba8(&mut premul);
                    r.replace_individual_pixels(*id, *lado, *lado, &premul)
                        .expect("sobe");
                } else {
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "lado"
                    )]
                    let w = (f64::from(*lado) * f.sqrt()) as u32;
                    premul.clear();
                    for y in 0..w {
                        let a = ((y * *lado) * 4) as usize;
                        premul.extend_from_slice(&tela[a..a + (w * 4) as usize]);
                    }
                    ph2d_render::premul::premultiply_rgba8(&mut premul);
                    r.replace_individual_pixels_region(*id, 0, 0, w, w, &premul)
                        .expect("sobe");
                }
                let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            }
            #[expect(clippy::cast_precision_loss, reason = "BLOCO pequeno")]
            let ms = t0.elapsed().as_secs_f64() * 1e3 / BLOCO as f64;
            *best = best.min(ms);
        }
    }
    eprintln!("load depois: {}", carga().trim());
    for (lado, f, _, _, best) in &vs {
        eprintln!(
            "[w5e] subir {lado}², {:>3.0} % da área: {best:.2} ms (mínimo)",
            f * 100.0
        );
    }
}

/// **W9 A3: o custo do quadro com a água a correr** (`docs/Macros/06_auditoria_W8_record.md` A3: a W8
/// relia a imagem e subia a ranhura inteira a cada quadro da água — 6,6 ms média / 12,5 ms máx, `smoke`,
/// só CPU). A 1024², gravado no Painter ligado com a água no worker real, o `liga_a_marca` + `sobe` (o
/// duplo de CPU das ranhuras, como na auditoria) em duas variantes do MESMO processo, blocos
/// intercalados por ordem rodada, quadros ao ritmo do editor (60 Hz: o worker devolve a água entre
/// eles), a média e o máximo como na auditoria: sem marca, com uma pincelada marcada (a imagem relê-se
/// a cada passo da água e só a caixa dela sobe) e, W10, com a pincelada em Digital (a cobertura parcial:
/// a mistura lê o que estava por baixo em cada texel da borda).
///
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-painter --profile smoke --lib -- --ignored
/// --nocapture sonda_o_custo_da_marca`
#[test]
#[ignore = "sonda de custo (W9 A3)"]
fn sonda_o_custo_da_marca() {
    use super::palcos_tests::Placa;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, Tool};
    use ph2d_tool_painter::PainterTool;
    const LADO: u32 = 1024;
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    struct V {
        nome: &'static str,
        /// O clipe toca (o palco muda a cada quadro e a mistura corre sempre) ou fica em 0.
        toca: Option<ph2d_ecs::Entity>,
        q: u32,
        world: ph2d_ecs::World,
        palcos: PalcosDoPainter,
        placa: Placa,
        t: PainterTool,
        ms: Vec<f64>,
    }
    let nova = |nome, marcada: Option<ph2d_tool_painter::PaintMedia>, toca: bool| {
        let mut world = ph2d_ecs::World::new();
        let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
        let (mut t, doc, _) =
            ph2d_tool_painter::macro_track::roteiro::grava_no_papel_ligado_com_o_worker(
                LADO,
                e.to_bits(),
                4,
            )
            .expect("grava");
        world.entity_mut(e).insert((
            Macro {
                doc,
                modo: ModoDaMacro::Fiel,
            },
            MacroTempo { segundos: 0.0 },
        ));
        let mut v = V {
            nome,
            toca: toca.then_some(e),
            q: 0,
            world,
            palcos: PalcosDoPainter::default(),
            placa: Placa::default(),
            t: PainterTool::default(),
            ms: Vec::new(),
        };
        let mut nunca = |_: u64, _: &mut Placa| None;
        v.palcos.segue(&mut v.world);
        v.palcos
            .liga_a_marca(Some(&mut t), &mut nunca, &mut v.placa);
        if let Some(meio) = marcada {
            t.set_paint_media(meio);
            t.set_brush_size_px(24.0);
            for i in 0..=8u16 {
                let phase = match i {
                    0 => PointerPhase::Down,
                    8 => PointerPhase::Up,
                    _ => PointerPhase::Move,
                };
                t.on_canvas_pointer(CanvasPointer {
                    pos: [100.0 + 40.0 * f32::from(i), 900.0],
                    pressure: 1.0,
                    tilt: [0.0, 0.0],
                    phase,
                });
                drop(t.take_preview_arc());
                t.on_tick(16.667);
            }
        }
        v.t = t;
        v
    };
    use ph2d_tool_painter::PaintMedia::{Digital, WetPaint};
    let mut vs = [
        nova("água, sem marca", None, false),
        nova("água, pincelada marcada", Some(WetPaint), false),
        nova("Play, pincelada Wet (o pa)", Some(WetPaint), true),
        nova("Play, pincelada parcial (Digital)", Some(Digital), true),
    ];
    let quadro = |v: &mut V| -> f64 {
        // O ritmo do editor: o worker devolve a água entre quadros.
        std::thread::sleep(std::time::Duration::from_millis(16));
        drop(v.t.take_preview_arc());
        v.t.on_tick(16.667);
        let mut nunca = |_: u64, _: &mut Placa| None;
        if let Some(e) = v.toca {
            v.q += 1;
            v.world.get_mut::<MacroTempo>(e).expect("tempo").segundos = f64::from(v.q % 240) / 60.0;
        }
        let t0 = std::time::Instant::now();
        v.palcos.segue(&mut v.world);
        v.palcos
            .liga_a_marca(Some(&mut v.t), &mut nunca, &mut v.placa);
        let trocas = v.palcos.sobe(
            &mut v.placa,
            &mut ToastQueue::default(),
            None,
            super::Atuais {
                painter: Some(&v.t),
                le_sprite: Some(&mut nunca),
            },
        );
        assert_eq!(trocas.len(), 1);
        t0.elapsed().as_secs_f64() * 1e3
    };
    for v in &mut vs {
        quadro(v);
    }
    eprintln!("load antes: {}", carga().trim());
    let k = vs.len();
    for rodada in 0..RODADAS * 2 {
        for i in 0..k {
            let v = &mut vs[(rodada + i) % k];
            for _ in 0..BLOCO {
                let ms = quadro(v);
                v.ms.push(ms);
            }
        }
    }
    eprintln!("load depois: {}", carga().trim());
    for v in &mut vs {
        let correu = !v.t.agua_parada();
        let marcados = v.t.marca_do_artista().map_or(0, |m| m.quantos());
        v.ms.sort_by(f64::total_cmp);
        #[expect(clippy::cast_precision_loss, reason = "centenas de quadros")]
        let media = v.ms.iter().sum::<f64>() / v.ms.len() as f64;
        eprintln!(
            "[w9 A3] {LADO}² {:<26} liga+sobe por quadro: média {media:.3} ms · mediana {:.3} · máx \
             {:.3} ({} quadros a 60 Hz) · água ainda a correr: {correu} · marcados: {marcados} · \
             subidas inteiras {} · parciais {}",
            v.nome,
            v.ms[v.ms.len() / 2],
            v.ms[v.ms.len() - 1],
            v.ms.len(),
            v.placa.substituidas,
            v.placa.regioes
        );
    }
}

/// **W10: o que a cobertura parcial custa à mistura** (`docs/Macros/08_plano_W10_cobertura.md` §3, a
/// linha do custo): a mesma caixa de 512² numa imagem de 1024², coberta por inteiro (a cópia da W9) e em
/// parte (`atual + (1 − c)·(palco − B)` com o `B` premultiplicado por texel), blocos intercalados por
/// ordem rodada no MESMO processo; o mínimo de cada.
///
/// `bash scripts/ph2d-run.sh cargo test -p ph2d-app-painter --profile smoke --lib -- --ignored
/// --nocapture sonda_o_custo_da_mistura_parcial`
#[test]
#[ignore = "sonda de custo (W10)"]
fn sonda_o_custo_da_mistura_parcial() {
    use ph2d_tool_painter::macro_track::marca::MarcaDoArtista;
    const LADO: u32 = 1024;
    const CAIXA: (u32, u32, u32, u32) = (256, 256, 512, 512);
    let marca = |a: f32| {
        let mut m = MarcaDoArtista::nova(LADO, LADO, 1);
        let (x0, y0, w, h) = CAIXA;
        m.compoe_muitos((y0..y0 + h).flat_map(|y| {
            (x0..x0 + w).map(move |x| ((y * LADO + x) as usize, a, [90, 120, 200, 255]))
        }));
        m
    };
    let variantes = [("cheia (W9)", marca(1.0)), ("parcial (W10)", marca(0.5))];
    let atual = vec![200u8; (LADO * LADO * 4) as usize];
    let palco = vec![40u8; (CAIXA.2 * CAIXA.3 * 4) as usize];
    let mut melhor = [f64::MAX; 2];
    let mut out = palco.clone();
    for rodada in 0..RODADAS * 4 {
        for i in 0..2 {
            let k = (rodada + i) % 2;
            for _ in 0..BLOCO {
                out.copy_from_slice(&palco);
                let t0 = std::time::Instant::now();
                super::mistura_na_caixa(&mut out, &variantes[k].1, &atual, LADO, CAIXA);
                melhor[k] = melhor[k].min(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
    }
    for (k, (nome, _)) in variantes.iter().enumerate() {
        eprintln!(
            "[w10] mistura de 512² {nome:<14}: {:.3} ms (mínimo)",
            melhor[k]
        );
    }
}
