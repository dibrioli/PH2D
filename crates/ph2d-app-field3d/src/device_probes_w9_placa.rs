//! ⏱️⭐⭐⭐⭐ **O QUADRO NA PLACA: onde o tempo mora** — as sondas de dispositivo da wave que achou o
//! kernel magro do matcap (`docs/Render3d/03_o_plano.md` §W9, «o kernel que hospeda a marcha»).
//!
//! Filha da [`super`] (as sondas de CPU de onde os passos acontecem), porque partilha a `laje` —
//! e partiu-se dela por tecto de LOC, pela fronteira CPU / placa.
//!
//! O que cada uma respondeu, pela ordem em que a pergunta andou: o ORÁCULO do estado da arte
//! (`fidget-wgpu`) é `5`–`160×` mais LENTO nas nossas peças · o INTERPRETADOR de fita custa
//! `15`–`35×` por instrução · a ESTRUTURA do kernel (cópias da peça) não pesa · o CPU do pedido não
//! pesa · e a MESMA marcha num kernel magro custava `18`–`31×` menos que o quadro ⇒ a cura.

use super::laje;

/// ⏱️ **Exporta as cenas para o ORÁCULO `fidget-wgpu`** (Keeter; MPL-2.0 ⇒ corre-se FORA da árvore,
/// ligado, nunca copiado). Grava em `PH2D_ORACULO_DIR` o documento de cada cena (`postcard`) e uma
/// linha de texto com a câmara de omissão (`alvo · direita · cima · para o olho · meia-altura`) e a
/// bola da peça — o bastante para o oráculo enquadrar a MESMA vista.
#[test]
#[ignore = "exporta ficheiros para o oráculo"]
fn diag_exporta_as_cenas_para_o_oraculo() {
    let Ok(dir) = std::env::var("PH2D_ORACULO_DIR") else {
        println!("PH2D_ORACULO_DIR por definir");
        return;
    };
    let cam = ph2d_field_render::Orbit::default();
    let (r, u, e) = cam.basis();
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
        std::fs::write(
            format!("{dir}/cena_{cena}.postcard"),
            postcard::to_allocvec(&doc).expect("postcard"),
        )
        .expect("gravar o documento");
        let f = |v: [f32; 3]| format!("{} {} {}", v[0], v[1], v[2]);
        std::fs::write(
            format!("{dir}/cena_{cena}.txt"),
            format!(
                "{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
                f(cam.target),
                f(r),
                f(u),
                f(e),
                cam.half_extent,
                f(bola.center),
                bola.radius
            ),
        )
        .expect("gravar a câmara");
    }
    println!("exportadas para {dir}");
}

/// ⏱️⭐⭐⭐⭐ **Sonda: quanto custa uma instrução INTERPRETADA contra uma COMPILADA** (GPU).
///
/// O factor que decide a poda por região: ela só é construível com um interpretador (as peças
/// complexas pedem milhares de fitas podadas por quadro), e o ganho é
/// `razão da poda × este factor`. `2 M` pontos uniformes na caixa da peça, a mesma fita dos dois
/// lados, e a resposta comparada ponto a ponto.
#[test]
#[ignore = "sonda de GPU"]
fn diag_interpretada_contra_compilada() {
    println!(
        "\n  cena · operações · registos · compilada ms · interpretada ms · FACTOR · ns/op \
         compilada · ns/op interpretada · pior desvio"
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let mut st = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            #[allow(clippy::cast_precision_loss)]
            {
                (st >> 11) as f32 / (1u64 << 53) as f32
            }
        };
        let pontos: Vec<[f32; 3]> = (0..1920 * 1080)
            .map(|_| std::array::from_fn(|a| lo[a] + rnd() * (hi[a] - lo[a])))
            .collect();
        let Some(b) = ph2d_field_gpu::interp_bench::mede(&doc, &pontos, 8) else {
            println!("  {cena:4} · sem placa ou sem bytecode");
            continue;
        };
        #[allow(clippy::cast_precision_loss)]
        let por_op = |ms: f64| ms * 1e6 / (pontos.len() as f64 * b.operacoes.max(1) as f64);
        println!(
            "  {cena:4} · {:>5} · {:>4} · {:>7.3} · {:>7.3} · {:>5.2}× · {:.4} · {:.4} · {:.2e}",
            b.operacoes,
            b.registos,
            b.ms_compilado,
            b.ms_interpretado,
            b.ms_interpretado / b.ms_compilado,
            por_op(b.ms_compilado),
            por_op(b.ms_interpretado),
            b.pior_desvio
        );
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda: para onde vai o quadro do matcap** — CPU do pedido, peça a peça, contra a placa.
///
/// A irmã acima mede a fita inteira do nó em `2 M` pontos a `0,22 ms`, e o quadro dele custa
/// `~59 ms` com `~10` avaliações por pixel. ⇒ o tempo NÃO mora na avaliação; esta sonda parte o
/// quadro do produto (`pinta_matcap`) nas peças que o pedido paga na CPU e no despacho da placa.
#[test]
#[ignore = "sonda de GPU"]
fn diag_para_onde_vai_o_quadro() {
    let reg0 = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    const W: u32 = 1920;
    const H: u32 = 1080;
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    drop(reg0);
    println!(
        "\n  {}\n  cena · quadro inteiro · campo · fita wgsl · bola · passo · encolhe · PLACA (matcap_frame) \
         [ms, mínimo de 5]",
        super::super::super::contexto()
    );
    let min_de = |f: &mut dyn FnMut()| {
        let mut m = f64::INFINITY;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            f();
            m = m.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        m
    };
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let inteiro = min_de(&mut || {
            let _ = crate::gpu_frame::pinta_matcap(t, &doc, &reg, &cam, &mc, W, H);
        });
        let campo = min_de(&mut || {
            let _ = ph2d_field_eval::device::DeviceField::new_com(&doc, &reg, true);
        });
        let dc = ph2d_field_eval::device::DeviceField::new_com(&doc, &reg, true).expect("campo");
        let fita = min_de(&mut || {
            let _ = dc.tape_wgsl();
        });
        let bola = min_de(&mut || {
            let _ = ph2d_field_eval::bounds::bounding_ball(&doc, &reg);
        });
        let passo = min_de(&mut || {
            let _ = ph2d_field_eval::safe_march_step(&doc);
        });
        let encolhe = min_de(&mut || {
            let _ = ph2d_field_eval::field_shrink(&doc, &reg);
        });
        let (c, f, setup) = crate::gpu_frame::pedido(
            &doc,
            &reg,
            &cam,
            &[],
            None,
            ph2d_field_gpu::trace::MAX_LAMPS,
            crate::gpu_frame::Sonda::default(),
            W,
            H,
            None,
            false,
        )
        .expect("o pedido");
        let placa = min_de(&mut || {
            let mut g = t.lock().expect("o traçador");
            let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, W, H);
        });
        println!(
            "  {cena:4} · {inteiro:>7.2} · {campo:>6.2} · {fita:>5.2} · {bola:>6.2} · {passo:>5.2} · \
             {encolhe:>6.2} · {placa:>7.2}"
        );
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda: o que a ESTRUTURA do kernel custa à mesma fita** (GPU) — ver
/// [`ph2d_field_gpu::interp_bench::ESTRUTURAS`]. Se a normal escrita à parte custar muito mais que
/// a mesma normal num laço, o defeito é o TAMANHO do código (cópias da peça), não a avaliação.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_que_a_estrutura_do_kernel_custa() {
    let nomes: Vec<&str> = ph2d_field_gpu::interp_bench::ESTRUTURAS
        .iter()
        .map(|e| e.0)
        .collect();
    println!(
        "\n  cena · operações · {} [ms, 2 M pontos, mínimo de 8]",
        nomes.join(" · ")
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let mut st = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            #[allow(clippy::cast_precision_loss)]
            {
                (st >> 11) as f32 / (1u64 << 53) as f32
            }
        };
        let pontos: Vec<[f32; 3]> = (0..1920 * 1080)
            .map(|_| std::array::from_fn(|a| lo[a] + rnd() * (hi[a] - lo[a])))
            .collect();
        let ops = ph2d_field_eval::Field::new(&doc)
            .tape_bytecode()
            .map_or(0, |b| b.operacoes);
        let Some(v) = ph2d_field_gpu::interp_bench::mede_estrutura(&doc, &pontos, 8) else {
            continue;
        };
        let txt: Vec<String> = v.iter().map(|m| format!("{m:>7.3}")).collect();
        println!("  {cena:4} · {ops:>5} · {}", txt.join(" · "));
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda: o quadro da placa contra a RESOLUÇÃO** — separa o custo por pixel do custo
/// fixo por quadro (compilação, envios, leituras). Imprime também quantos pipelines o traçador
/// compilou durante as corridas medidas (tem de ser `0` depois do aquecimento).
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_quadro_contra_a_resolucao() {
    let cam = ph2d_field_render::Orbit::default();
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    println!(
        "\n  {}\n  cena · resolução · placa ms (mín de 5) · compilações durante a medida",
        super::super::super::contexto()
    );
    for cena in [28u32, 5] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        for (w, h) in [
            (1920u32, 1080u32),
            (960, 540),
            (480, 270),
            (240, 135),
            (64, 36),
        ] {
            let (c, f, setup) = crate::gpu_frame::pedido(
                &doc,
                &reg,
                &cam,
                &[],
                None,
                ph2d_field_gpu::trace::MAX_LAMPS,
                crate::gpu_frame::Sonda::default(),
                w,
                h,
                None,
                false,
            )
            .expect("o pedido");
            {
                let mut g = t.lock().expect("o traçador");
                let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, w, h);
            }
            let antes = t.lock().expect("o traçador").compiled();
            let mut m = f64::INFINITY;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let mut g = t.lock().expect("o traçador");
                let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, w, h);
                m = m.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            let depois = t.lock().expect("o traçador").compiled();
            println!("  {cena:4} · {w}×{h} · {m:>7.2} · {}", depois - antes);
        }
    }
}

/// Tempo de CPU desta thread, em ms (utime + stime de `/proc/thread-self/stat`, em ticks de 10 ms —
/// grosso, logo só vale somado sobre muitos quadros).
fn cpu_da_thread_ms() -> f64 {
    let s = std::fs::read_to_string("/proc/thread-self/stat").unwrap_or_default();
    let depois = s.rsplit(')').next().unwrap_or("");
    let campos: Vec<&str> = depois.split_whitespace().collect();
    let tique = |i: usize| {
        campos
            .get(i)
            .and_then(|x| x.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    // Depois do `)` o 1.º campo é o estado (3.º do ficheiro); utime é o 14.º e stime o 15.º.
    (tique(11) + tique(12)) * 10.0
}

/// ⏱️⭐⭐⭐⭐ **Sonda: o quadro da placa — CPU desta thread contra o relógio**, 100 quadros.
/// Se a CPU ≈ relógio o custo é do nosso processo; se o relógio for muito maior, é espera da placa.
#[test]
#[ignore = "sonda de GPU"]
fn diag_cpu_contra_relogio_do_quadro() {
    let cam = ph2d_field_render::Orbit::default();
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    println!("\n  cena · resolução · relógio ms/quadro · CPU da thread ms/quadro");
    for cena in [28u32, 5] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        for (w, h) in [(64u32, 36u32)] {
            let (c, f, setup) = crate::gpu_frame::pedido(
                &doc,
                &reg,
                &cam,
                &[],
                None,
                ph2d_field_gpu::trace::MAX_LAMPS,
                crate::gpu_frame::Sonda::default(),
                w,
                h,
                None,
                false,
            )
            .expect("o pedido");
            let mut g = t.lock().expect("o traçador");
            let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, w, h);
            // ⚠️ Em LOTES de 10, a imprimir cada um: a 1.ª redacção corria 100 seguidos e ficou
            // 10 min a 99 % de CPU — o custo CRESCE quadro a quadro, e só lotes o mostram.
            for lote in 0..4 {
                let (c0, t0) = (cpu_da_thread_ms(), std::time::Instant::now());
                for _ in 0..10 {
                    let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, w, h);
                }
                let rel = t0.elapsed().as_secs_f64() * 1e3 / 10.0;
                let cpu = (cpu_da_thread_ms() - c0) / 10.0;
                println!("  {cena:4} · {w}×{h} · lote {lote} · {rel:>8.2} · {cpu:>8.2}");
            }
        }
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda: a MESMA marcha num kernel MAGRO contra o quadro do produto** (GPU).
///
/// Os raios do produto a `1920×1080` (recorte `march_clip`, passo, orçamento, limiares) num kernel
/// que só marcha e tira a normal. Se o magro custar uma fracção do produto, o que pesa é o KERNEL
/// que hospeda a marcha (registos, ocupação), não a marcha.
#[test]
#[ignore = "sonda de GPU"]
fn diag_a_marcha_magra_contra_o_produto() {
    use ph2d_field_render::{MAX_STEPS, Orbit, Screen, Sharpness, T_MAX};
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = Orbit::default();
    let screen = Screen::new(W, H, cam.half_extent);
    let sharp = Sharpness::for_frame(cam.half_extent, W.min(H) as usize);
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    println!(
        "\n  {}\n  cena · marcha MAGRA ms · quadro do PRODUTO ms · razão",
        super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg).expect("a bola");
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let budget =
            ((MAX_STEPS as f32) * shrink.max(1.0) / passo.clamp(f32::EPSILON, 1.0)).ceil() as u32;
        let mut raios = Vec::with_capacity((W * H) as usize);
        for y in 0..H {
            for x in 0..W {
                #[allow(clippy::cast_precision_loss)]
                let (u, v) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (o, d) = cam.ray_at_plane(u, v);
                let (a, b) = laje(o, d, lo, hi).unwrap_or((1.0, 0.0));
                raios.push([o[0], o[1], o[2], d[0], d[1], d[2], a, b.min(T_MAX)]);
            }
        }
        let Some(magra) = ph2d_field_gpu::interp_bench::mede_marcha_magra(
            &doc,
            &raios,
            sharp.hit,
            passo,
            budget,
            sharp.normal,
            8,
        ) else {
            continue;
        };
        let (c, f, setup) = crate::gpu_frame::pedido(
            &doc,
            &reg,
            &cam,
            &[],
            None,
            ph2d_field_gpu::trace::MAX_LAMPS,
            crate::gpu_frame::Sonda::default(),
            W,
            H,
            None,
            false,
        )
        .expect("o pedido");
        let mut produto = f64::INFINITY;
        let mut g = t.lock().expect("o traçador");
        let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, W, H);
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let _ = g.matcap_frame(&f, c.sculpts(), setup, &mc, W, H);
            produto = produto.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        drop(g);
        println!(
            "  {cena:4} · {magra:>7.2} · {produto:>7.2} · {:>5.1}×",
            produto / magra
        );
    }
}

/// ⏱️⭐⭐⭐ **Sonda: as PEÇAS do quadro do matcap depois do kernel magro** (GPU).
///
/// O quadro magro ainda custa `3,6`–`6,7×` a marcha magra. Esta sonda parte o que sobra: o quadro
/// inteiro · o mesmo sem a segunda passagem da silhueta (`Sonda::bordas = false`) · e a leitura de
/// uma imagem RGBA8 do mesmo tamanho pelo barramento (`Tracer::mede_leitura`).
#[test]
#[ignore = "sonda de GPU"]
fn diag_as_pecas_do_quadro_magro() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = ph2d_field_render::Orbit::default();
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let leitura = t
        .lock()
        .expect("o traçador")
        .mede_leitura(u64::from(W) * u64::from(H) * 4, 8);
    println!(
        "\n  {}\n  leitura de uma imagem RGBA8 {W}×{H}: {leitura:.2} ms\n  cena · quadro ms · sem a \
         borda ms · a borda ms [mínimo de 5]",
        super::super::super::contexto()
    );
    let min_de = |f: &mut dyn FnMut()| {
        f();
        let mut m = f64::INFINITY;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            f();
            m = m.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        m
    };
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let com = min_de(&mut || {
            let _ = crate::gpu_frame::pinta_matcap(t, &doc, &reg, &cam, &mc, W, H);
        });
        let sonda = crate::gpu_frame::Sonda {
            bordas: false,
            ..crate::gpu_frame::Sonda::default()
        };
        let sem = min_de(&mut || {
            let _ = crate::gpu_frame::pinta_matcap_com(t, &doc, &reg, &cam, &mc, W, H, sonda);
        });
        println!("  {cena:4} · {com:>7.2} · {sem:>7.2} · {:>6.2}", com - sem);
    }
}

/// ⏱️⭐⭐⭐ **Sonda: o quadro do RENDER contra o do MATCAP, cena a cena** (GPU) — a pergunta *«o
/// modo Render paga o mesmo kernel pesado que o matcap pagava?»*. Uma lâmpada, o material de
/// omissão, sem chão; o quadro de movimento (`assente = false`) e o assente.
#[test]
#[ignore = "sonda de GPU"]
fn diag_o_render_contra_o_matcap() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = ph2d_field_render::Orbit::default();
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let pres = ph2d_field_render::Presentation::of(olhar);
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    println!(
        "\n  {}\n  cena · matcap ms · render em movimento ms · render assente ms [mínimo de 5]",
        super::super::super::contexto()
    );
    let min_de = |f: &mut dyn FnMut()| {
        f();
        let mut m = f64::INFINITY;
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            f();
            m = m.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        m
    };
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let matcap = min_de(&mut || {
            let _ = crate::gpu_frame::pinta_matcap(t, &doc, &reg, &cam, &mc, W, H);
        });
        let render = |assente| {
            min_de(&mut || {
                let _ = crate::gpu_frame::paint(
                    t,
                    &doc,
                    &reg,
                    &cam,
                    &luz,
                    &surfaces,
                    &pres,
                    [0, 0, 0, 0],
                    None,
                    W,
                    H,
                    assente,
                );
            })
        };
        let movimento = render(false);
        let assente = render(true);
        println!("  {cena:4} · {matcap:>7.2} · {movimento:>7.2} · {assente:>7.2}");
    }
}

/// ⏱️⭐⭐⭐⭐ **A oclusão do céu marchada numa GRADE** — ver o cabeçalho do [`ceu`].
#[path = "device_probes_w9_ceu.rs"]
mod ceu;
