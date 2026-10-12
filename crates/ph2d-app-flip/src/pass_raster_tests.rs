//! O quadro actual de um objecto Flip fora do ecrã (W15): a caixa (CPU) e a imagem (placa, `#[ignore]`).

use super::*;
use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};

/// Um objecto com um traço vermelho de `(0,0)` a `(2,0)`, raio `0,1`, no quadro 0; e um 2.º traço
/// noutra chave (quadro 6) que NÃO pode entrar na imagem do quadro 0.
fn doc_line() -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Line");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let layer = obj.add_layer("L");
    let line = |to: f32| {
        let mut s = FlipStroke::new();
        for x in [0.0, to] {
            s.push_point(Point {
                pos: Vec2::new(x, 0.0),
                width: 0.1,
                opacity: 1.0,
                color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            });
        }
        s
    };
    let d = obj
        .insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    obj.drawing_mut(d).unwrap().strokes.push(line(2.0));
    let d6 = obj
        .insert_frame(layer, 6, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    obj.drawing_mut(d6).unwrap().strokes.push(line(9.0));
    (doc, oid)
}

fn at(f: i32) -> Playhead {
    let mut p = Playhead::new(1.0 / 12.0);
    p.pause();
    p.seek(f64::from(f) / 12.0);
    p
}

/// ⭐ A caixa é a do QUADRO ACTUAL (a chave do quadro 6 não entra), alargada pelo raio, e escala
/// com o `model` do objecto (posição e espessura). (Mutação: ignorar o raio ⇒ `y0 = 0` — RED.)
#[test]
fn the_box_is_the_current_frames_strokes_with_their_radius_posed() {
    let (doc, oid) = doc_line();
    let boxed = |model: Xform| {
        let models = [(oid, model)];
        let (layers, _) = collect_layers(
            &doc,
            &at(0),
            None,
            FlipTarget::default(),
            &models,
            None,
            None,
        );
        world_box(&layers).expect("um traço")
    };
    let b = boxed(Xform::IDENTITY);
    let close = |a: [f64; 4], e: [f64; 4]| a.iter().zip(e).all(|(x, y)| (x - y).abs() < 1e-6);
    assert!(close(b, [-0.1, -0.1, 2.1, 0.1]), "{b:?}");
    let b2 = boxed(Xform([2.0, 0.0, 0.0, 2.0, 5.0, 1.0]));
    assert!(close(b2, [4.8, 0.8, 9.2, 1.2]), "{b2:?}");
}

fn gpu() -> Option<GpuContext> {
    GpuContext::new(wgpu::Instance::default(), None).ok()
}

/// ⭐⭐ A imagem pela placa: `100 px` por unidade dão `2,2 × 0,2` unidades + 2 px de orla =
/// `222 × 22`; o meio do traço é vermelho opaco (alfa reto: o vermelho não escurece), o canto é
/// transparente (a ponta é redonda), e sem traço neste quadro não há imagem.
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn the_current_frame_comes_out_at_the_editors_pixel_ruler() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — nada a medir");
        return;
    };
    let (doc, oid) = doc_line();
    let mut render = FlipRenderer::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let mut compose = FlipCompose::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let img = current_frame(
        &doc,
        oid,
        Xform::IDENTITY,
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    )
    .expect("o quadro 0 tem traço");
    assert_eq!(img.size, [222, 22]);
    let px = |x: u32, y: u32| {
        let i = ((y * img.size[0] + x) * 4) as usize;
        [
            img.rgba[i],
            img.rgba[i + 1],
            img.rgba[i + 2],
            img.rgba[i + 3],
        ]
    };
    let mid = px(111, 11);
    assert!(
        mid[3] == 255 && mid[0] > 240 && mid[1] < 16 && mid[2] < 16,
        "{mid:?}"
    );
    assert_eq!(
        px(0, 0)[3],
        0,
        "o canto da caixa fica fora da ponta redonda"
    );
    let wide = current_frame(
        &doc,
        oid,
        Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]),
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    )
    .expect("escalado");
    assert_eq!(
        wide.size,
        [442, 42],
        "o objecto escalado 2× sai com o dobro"
    );
    let empty = FlipDoc::new();
    let none = current_frame(
        &empty,
        oid,
        Xform::IDENTITY,
        &at(0),
        &mut render,
        &mut compose,
        100.0,
        &gpu,
    );
    assert_eq!(none, Err(RasterError::Empty));
}

/// ⭐ W16 — o ladrilho: o do ecrã quando as fatias cabem nele (a leitura não realoca o scratch do
/// ecrã); senão o maior quadrado cuja leitura inteira (`4·camadas + 32` bytes por píxel) cabe no
/// orçamento, nunca maior que a imagem nem que a textura. 8 camadas a 8192² com 1 GiB: 4096 (de uma
/// vez eram 8 fatias de 256 MiB — falhava).
#[test]
fn the_tile_is_the_screens_when_the_layers_fit_in_it() {
    const GIB: u64 = 1 << 30;
    assert_eq!(
        tile_for((8192, 8192), 8, Some((2560, 1440)), GIB, 8192),
        (2560, 1440)
    );
    assert_eq!(tile_for((8192, 8192), 8, None, GIB, 8192), (4096, 4096));
    assert_eq!(tile_for((300, 100), 8, None, GIB, 8192), (300, 100));
    // 200 camadas não cabem no ecrã (72 fatias de 2560×1440 em 1 GiB): o quadrado do orçamento.
    let (w, h) = tile_for((8192, 8192), 200, Some((2560, 1440)), GIB, 8192);
    assert_eq!(w, h);
    assert!(u64::from(w * h) * (4 * 200 + 32) <= GIB);
    assert_eq!(
        tile_for((9000, 9000), 1, None, 64 * GIB, 8192),
        (8192, 8192)
    );
}

/// Um «L»: um traço vermelho de `(0,0)` a `(2,0)` e um azul de `(0,0)` a `(0,1)`, raio `0,1`.
fn doc_l() -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("L");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    let layer = obj.add_layer("L");
    let d = obj
        .insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe)
        .unwrap();
    for (to, c) in [([2.0, 0.0], [1.0, 0.0, 0.0]), ([0.0, 1.0], [0.0, 0.0, 1.0])] {
        let mut s = FlipStroke::new();
        for p in [[0.0, 0.0], to] {
            s.push_point(Point {
                pos: Vec2::new(p[0], p[1]),
                width: 0.1,
                opacity: 1.0,
                color: Rgba::new(c[0], c[1], c[2], 1.0),
            });
        }
        obj.drawing_mut(d).unwrap().strokes.push(s);
    }
    (doc, oid)
}

/// ⭐⭐ W16 — em ladrilhos (de tamanhos ímpares, com a borda a meio) sai a MESMA imagem que de uma vez:
/// o traço azul sobe à esquerda (o mundo sobe quando a linha desce).
#[test]
#[ignore = "precisa de adapter GPU; roda com --ignored"]
fn tiles_read_the_same_frame_as_one_shot() {
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — nada a medir");
        return;
    };
    let (doc, oid) = doc_l();
    let mut render = FlipRenderer::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let mut compose = FlipCompose::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let models = [(oid, Xform::IDENTITY)];
    let (mut layers, _) = collect_layers(
        &doc,
        &at(0),
        None,
        FlipTarget::default(),
        &models,
        None,
        None,
    );
    for l in &mut layers {
        l.depth = 1.0;
    }
    let mut read = |tile: Option<(u32, u32)>| {
        read_frame(&layers, &mut render, &mut compose, 100.0, &gpu, |out| {
            tile.unwrap_or(out)
        })
        .expect("o «L»")
    };
    let one = read(None);
    let tiled = read(Some((37, 13)));
    assert_eq!(one.size, [222, 122]);
    assert_eq!(tiled.size, one.size);
    let worst = one
        .rgba
        .iter()
        .zip(&tiled.rgba)
        .map(|(a, b)| a.abs_diff(*b))
        .max()
        .unwrap();
    assert!(worst <= 2, "um ladrilho diverge em {worst}");
    let px = |x: u32, y: u32| {
        let i = ((y * one.size[0] + x) * 4) as usize;
        [one.rgba[i], one.rgba[i + 2], one.rgba[i + 3]]
    };
    let up = px(11, 30);
    assert!(
        up[1] > 240 && up[2] == 255,
        "azul em cima à esquerda: {up:?}"
    );
    let right = px(200, 111);
    assert!(
        right[0] > 240 && right[2] == 255,
        "vermelho em baixo à direita: {right:?}"
    );
}

// ── MEDIÇÃO (não é gate): o custo do «Send to board…» de um Flip ─────────────────────────────────
// Uso: `docs/MiroClone/ferramentas/memoria_flip/corre.sh` (um processo por variante; a saída vai para
// `saida_<data>.txt`). Variante: `PH2D_MEDE_CAMADAS=<n> PH2D_MEDE_LADO=<px>`, `--ignored --nocapture`.

/// Um objecto com `n` camadas, cada uma a cobrir o quadrado `[0,10]²` com 10 traços horizontais de
/// raio `0,5` (cor própria por camada) — a caixa do quadro é exactamente `10 × 10` unidades.
fn doc_cheio(n: usize) -> (FlipDoc, FlipObjectId) {
    let mut doc = FlipDoc::new();
    let oid = doc.push_object("Cheio");
    let obj = doc.object_mut(oid).unwrap();
    obj.fps = 12.0;
    for k in 0..n {
        let layer = obj.add_layer(format!("L{k}"));
        let d = obj
            .insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe)
            .unwrap();
        let c = Rgba::new(k as f32 / n as f32, 0.5, 1.0 - k as f32 / n as f32, 1.0);
        for row in 0..10 {
            let mut s = FlipStroke::new();
            for x in [0.5, 9.5] {
                s.push_point(Point {
                    pos: Vec2::new(x, row as f32 + 0.5),
                    width: 0.5,
                    opacity: 1.0,
                    color: c,
                });
            }
            obj.drawing_mut(d).unwrap().strokes.push(s);
        }
    }
    (doc, oid)
}

/// `VmRSS`/`VmHWM` em MiB (`/proc/self/status`).
fn proc_kb(campo: &str) -> f64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    s.lines()
        .find(|l| l.starts_with(campo))
        .and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok())
        .map_or(-1.0, |kb| kb / 1024.0)
}

/// Zera o pico de RSS do processo (`clear_refs` = 5, Linux ≥ 4.0).
fn zera_pico_rss() {
    let _ = std::fs::write("/proc/self/clear_refs", "5");
}

/// A memória de placa deste PID e a da placa inteira segundo o `nvidia-smi` (a tabela de processos
/// cobre os `G` do Vulkan, que o `--query-compute-apps` não lista). `None` = o PID não aparece.
fn nvsmi_mib(pid: u32) -> Option<(f64, f64)> {
    let out = std::process::Command::new("nvidia-smi").output().ok()?;
    let txt = String::from_utf8_lossy(&out.stdout);
    let p = pid.to_string();
    let placa = txt.lines().find(|l| l.contains("MiB /")).and_then(|l| {
        let i = l.find("MiB /")?;
        l[..i].split_whitespace().last()?.parse().ok()
    })?;
    let proc = txt.lines().find_map(|l| {
        let t: Vec<&str> = l.trim_matches('|').split_whitespace().collect();
        (t.len() >= 6 && t[3] == p)
            .then(|| t.last()?.strip_suffix("MiB")?.parse().ok())
            .flatten()
    })?;
    Some((proc, placa))
}

/// Amostrador lateral do `nvidia-smi` (~20 Hz): `(ms desde o início, MiB do PID, MiB da placa)`.
struct Amostrador {
    para: std::sync::Arc<std::sync::atomic::AtomicBool>,
    fio: Option<std::thread::JoinHandle<Vec<(f64, f64, f64)>>>,
}

impl Amostrador {
    fn novo(t0: std::time::Instant) -> Self {
        let para = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let p = para.clone();
        let pid = std::process::id();
        let fio = std::thread::spawn(move || {
            let mut v = Vec::new();
            while !p.load(std::sync::atomic::Ordering::Relaxed) {
                let t = t0.elapsed().as_secs_f64() * 1e3;
                if let Some((m, g)) = nvsmi_mib(pid) {
                    v.push((t, m, g));
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            v
        });
        Self {
            para,
            fio: Some(fio),
        }
    }

    fn fecha(mut self) -> Vec<(f64, f64, f64)> {
        self.para.store(true, std::sync::atomic::Ordering::Relaxed);
        self.fio.take().unwrap().join().unwrap_or_default()
    }
}

const MIB: f64 = 1024.0 * 1024.0;

/// Bytes alocados pelo `gpu-allocator` do wgpu (o que o dispositivo segura em recursos).
fn alocado_mib(gpu: &GpuContext) -> f64 {
    gpu.device
        .generate_allocator_report()
        .map_or(-1.0, |r| r.total_allocated_bytes as f64 / MIB)
}

/// As alocações vivas agrupadas por rótulo, em MiB (só as ≥ 1 MiB).
fn por_rotulo(gpu: &GpuContext) -> Vec<(String, f64, usize)> {
    let Some(r) = gpu.device.generate_allocator_report() else {
        return Vec::new();
    };
    let mut m: std::collections::BTreeMap<String, (u64, usize)> = Default::default();
    for a in r.allocations {
        let e = m.entry(a.name).or_default();
        e.0 += a.size;
        e.1 += 1;
    }
    let mut v: Vec<_> = m
        .into_iter()
        .map(|(k, (b, n))| (k, b as f64 / MIB, n))
        .filter(|x| x.1 >= 1.0)
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1));
    v
}

/// UM quadro do ecrã (o `compose` do passe, sem o blit) com a câmara em `centro`, esperado até a
/// placa acabar; devolve ms.
#[allow(clippy::too_many_arguments)]
fn quadro_do_ecra(
    doc: &FlipDoc,
    oid: FlipObjectId,
    render: &mut FlipRenderer,
    fc: &mut FlipCompose,
    comp: &mut FlipComposite,
    ecra: (u32, u32),
    centro: [f32; 2],
    gpu: &GpuContext,
) -> f64 {
    let models = [(oid, Xform::IDENTITY)];
    let (layers, _) = collect_layers(
        doc,
        &at(0),
        None,
        FlipTarget::default(),
        &models,
        None,
        None,
    );
    let camera = Camera2d::new(centro, 12.0);
    let cam = ph2d_flip_render::camera::camera_raw(&camera, WindowSize::new(ecra.0, ecra.1));
    let t = std::time::Instant::now();
    assert!(compose(&layers, render, fc, comp, &cam, centro, ecra, gpu));
    let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    t.elapsed().as_secs_f64() * 1e3
}

/// ⭐ A medição: pico de placa (nvidia-smi + alocador do wgpu), pico de RSS e tempo de
/// `current_frame` para `PH2D_MEDE_CAMADAS` camadas a `PH2D_MEDE_LADO` píxeis; e o que acontece ao
/// scratch partilhado do ecrã nos quadros seguintes.
#[test]
#[ignore = "medição na placa; corre por docs/MiroClone/ferramentas/memoria_flip/corre.sh"]
fn mede_memoria_do_quadro_actual_do_flip() {
    let env = |k: &str, d: u32| {
        std::env::var(k)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(d)
    };
    let (n, lado) = (
        env("PH2D_MEDE_CAMADAS", 1) as usize,
        env("PH2D_MEDE_LADO", 2048),
    );
    let ecra = (2560u32, 1440u32);
    let t0 = std::time::Instant::now();
    let amostras = Amostrador::novo(t0);
    let ms = || t0.elapsed().as_secs_f64() * 1e3;
    let mut janelas: Vec<(String, f64, f64)> = Vec::new();
    let Some(gpu) = gpu() else {
        eprintln!("sem adapter GPU — nada a medir");
        let _ = amostras.fecha();
        return;
    };
    let info = gpu.adapter.get_info();
    println!(
        "VARIANTE camadas={n} lado={lado} ecra={}x{} adapter={} {:?} motor_novo={} max_tex={}",
        ecra.0,
        ecra.1,
        info.name,
        info.backend,
        new_engine_armed(),
        gpu.device.limits().max_texture_dimension_2d
    );
    let (doc, oid) = doc_cheio(n);
    let mut render = FlipRenderer::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    let mut fc = FlipCompose::new(&gpu.device, ph2d_render::GameRt::FORMAT);
    fc.set_walk_engine(&gpu.device, new_engine_armed());
    let mut ecomp = FlipComposite::new(&gpu);
    // O ecrã em regime: 5 quadros com a câmara a panhar (cada um re-rasteriza as camadas).
    let mut pan = 0.0f32;
    let mut regime = f64::INFINITY;
    for _ in 0..5 {
        pan += 0.01;
        let t = quadro_do_ecra(
            &doc,
            oid,
            &mut render,
            &mut fc,
            &mut ecomp,
            ecra,
            [5.0 + pan, 5.0],
            &gpu,
        );
        regime = regime.min(t);
    }
    std::thread::sleep(std::time::Duration::from_millis(400));
    let base_aloc = alocado_mib(&gpu);
    let base_rss = proc_kb("VmRSS:");
    println!(
        "BASE ecra_em_regime: alocador={base_aloc:.1} MiB rss={base_rss:.1} MiB quadro_pan_min={regime:.2} ms"
    );
    for (nome, mib, k) in por_rotulo(&gpu) {
        println!("  base  {mib:9.1} MiB  x{k:<3} {nome}");
    }
    let ppw = 1.0e6_f32; // acima do tecto: a imagem desce até `max_texture_dimension_2d`
    let ppw = if lado >= gpu.device.limits().max_texture_dimension_2d {
        ppw
    } else {
        (lado as f32 - 2.5) / 10.0
    };
    let (mut t_cf, mut t_memo, mut t_realoca, mut t_depois) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for rep in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(300));
        zera_pico_rss();
        let rss0 = proc_kb("VmRSS:");
        let a = ms();
        let t = std::time::Instant::now();
        let r = current_frame(
            &doc,
            oid,
            Xform::IDENTITY,
            &at(0),
            &mut render,
            &mut fc,
            ppw,
            &gpu,
        );
        let dt = t.elapsed().as_secs_f64() * 1e3;
        let b = ms();
        let hwm = proc_kb("VmHWM:");
        let tamanho = r.as_ref().map(|f| f.size).map_err(|e| *e);
        drop(r);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        janelas.push((format!("current_frame#{rep}"), a, b));
        let fica = alocado_mib(&gpu);
        println!(
            "CF#{rep} resultado={tamanho:?} t={dt:.1} ms rss_antes={rss0:.1} pico_rss={hwm:.1} (+{:.1}) MiB alocador_depois={fica:.1} MiB (+{:.1} sobre a base)",
            hwm - rss0,
            fica - base_aloc
        );
        if rep == 0 {
            for (nome, mib, k) in por_rotulo(&gpu) {
                println!("  depois_cf {mib:9.1} MiB  x{k:<3} {nome}");
            }
        }
        t_cf.push(dt);
        // O quadro seguinte do ecrã SEM mexer em nada (a frescura não re-rasteriza).
        let a = ms();
        let tm = quadro_do_ecra(
            &doc,
            oid,
            &mut render,
            &mut fc,
            &mut ecomp,
            ecra,
            [5.0 + pan, 5.0],
            &gpu,
        );
        janelas.push((format!("ecra_memo#{rep}"), a, ms()));
        let memo_aloc = alocado_mib(&gpu);
        // O quadro seguinte COM pan: re-rasteriza ⇒ o scratch volta ao tamanho do ecrã.
        pan += 0.01;
        let a = ms();
        let tr = quadro_do_ecra(
            &doc,
            oid,
            &mut render,
            &mut fc,
            &mut ecomp,
            ecra,
            [5.0 + pan, 5.0],
            &gpu,
        );
        janelas.push((format!("ecra_realoca#{rep}"), a, ms()));
        let real_aloc = alocado_mib(&gpu);
        pan += 0.01;
        let td = quadro_do_ecra(
            &doc,
            oid,
            &mut render,
            &mut fc,
            &mut ecomp,
            ecra,
            [5.0 + pan, 5.0],
            &gpu,
        );
        println!(
            "ECRA#{rep} memo={tm:.2} ms (alocador {memo_aloc:.1} MiB) | pan_realoca={tr:.2} ms (alocador {real_aloc:.1} MiB) | pan_seguinte={td:.2} ms"
        );
        t_memo.push(tm);
        t_realoca.push(tr);
        t_depois.push(td);
    }
    let min = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    // A DECOMPOSIÇÃO: a mesma sequência do `current_frame` com o compositor desta leitura VIVO no
    // instante do pico (depois do `compose`, antes do `read_rgba8`), lida pelo alocador.
    {
        let models = [(oid, Xform::IDENTITY)];
        let (mut layers, _) = collect_layers(
            &doc,
            &at(0),
            None,
            FlipTarget::default(),
            &models,
            None,
            None,
        );
        layers.retain(|l| l.cache_key.0 == oid.0 && l.drawing.is_some());
        let [x0, y0, x1, y1] = world_box(&layers).unwrap();
        let max = f64::from(gpu.device.limits().max_texture_dimension_2d);
        let p = f64::from(ppw).min((max - 2.0) / (x1 - x0).max(y1 - y0));
        let px = |v: f64| (v * p - 1e-6).ceil().max(1.0) as u32 + 2;
        let (w, h) = (px(x1 - x0), px(y1 - y0));
        let center = [((x0 + x1) * 0.5) as f32, ((y0 + y1) * 0.5) as f32];
        let camera = Camera2d::new(center, (f64::from(h) / p) as f32);
        let cam = ph2d_flip_render::camera::camera_raw(&camera, WindowSize::new(w, h));
        let mut comp = FlipComposite::new(&gpu);
        let ok = compose(
            &layers,
            &mut render,
            &mut fc,
            &mut comp,
            &cam,
            center,
            (w, h),
            &gpu,
        );
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let pico = alocado_mib(&gpu);
        let padded = (w * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let leitura = f64::from(padded) * f64::from(h) / MIB;
        println!(
            "PICO_DECOMPOSTO {w}x{h} compose_ok={ok} alocador_no_pico={pico:.1} MiB (+{:.1} sobre a base) + buffer_de_leitura={leitura:.1} MiB ⇒ {:.1} MiB",
            pico - base_aloc,
            pico - base_aloc + leitura
        );
        for (nome, mib, k) in por_rotulo(&gpu) {
            println!("  pico  {mib:9.1} MiB  x{k:<3} {nome}");
        }
        println!(
            "  cpu   dummy={:.1} MiB  imagem={:.1} MiB (w·h·4 cada)",
            (w as f64) * (h as f64) * 4.0 / MIB,
            (w as f64) * (h as f64) * 4.0 / MIB
        );
        // O RSS SÓ da leitura (`read_rgba8`: buffer mapeado + a cópia para o `Vec`).
        if ok && let Some(tex) = comp.compositor.output_texture() {
            zera_pico_rss();
            let rss0 = proc_kb("VmRSS:");
            let t = std::time::Instant::now();
            let img = read_rgba8(&gpu, tex, w, h);
            let dt = t.elapsed().as_secs_f64() * 1e3;
            let hwm = proc_kb("VmHWM:");
            let com_img = proc_kb("VmRSS:");
            drop(img);
            println!(
                "  leitura read_rgba8: t={dt:.1} ms pico_rss=+{:.1} MiB (com a imagem viva +{:.1}, depois de a largar +{:.1})",
                hwm - rss0,
                com_img - rss0,
                proc_kb("VmRSS:") - rss0
            );
        }
        drop(comp);
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
    std::thread::sleep(std::time::Duration::from_millis(300));
    let v = amostras.fecha();
    let antes: Vec<_> = v.iter().take_while(|s| s.0 < janelas[0].1).collect();
    let (base_p, base_g) = antes.last().map_or((-1.0, -1.0), |s| (s.1, s.2));
    let pico_p = v.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
    let pico_g = v.iter().map(|s| s.2).fold(f64::NEG_INFINITY, f64::max);
    println!(
        "NVSMI amostras={} PID: base={base_p:.0} pico={pico_p:.0} (+{:.0}) MiB · PLACA: base={base_g:.0} pico={pico_g:.0} (+{:.0}) MiB",
        v.len(),
        pico_p - base_p,
        pico_g - base_g
    );
    for (nome, a, b) in &janelas {
        let dentro: Vec<_> = v
            .iter()
            .filter(|s| s.0 >= *a - 60.0 && s.0 <= *b + 60.0)
            .collect();
        let pp = dentro.iter().map(|s| s.1).fold(f64::NEG_INFINITY, f64::max);
        let pg = dentro.iter().map(|s| s.2).fold(f64::NEG_INFINITY, f64::max);
        println!(
            "  janela {nome:<16} {:.0} ms  amostras={} pico PID={pp:.0} placa={pg:.0} MiB",
            b - a,
            dentro.len()
        );
    }
    let depois = v.last().map_or(-1.0, |s| s.2);
    println!(
        "RESUMO camadas={n} lado={lado} cf_min={:.1} ms ecra_memo_min={:.2} ms ecra_realoca_min={:.2} ms ecra_pan_regime_min={:.2} ms (antes {regime:.2}) nvsmi_placa_fim={depois:.0} MiB",
        min(&t_cf),
        min(&t_memo),
        min(&t_realoca),
        min(&t_depois)
    );
}
