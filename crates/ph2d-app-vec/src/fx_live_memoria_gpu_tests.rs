//! **F35 — a memória de placa que o FX segura depois do zoom extremo.** A cena 41 nas condições do
//! dono (2560×1080, a roda do zoom de abertura ao máximo e de volta) e os bytes que o `FxLive` segura
//! em cada fase. Plano: `docs/UI_New_and_Simple/F35_plano.md`.

use ph2d_render::{Camera2d, VelloPass};
use ph2d_vec_render::{BrushArts, DilatedPaints, LiveGeometry, PatternTiles, WidgetSkins};
use ph2d_vec_scene::{VecViewState, VecXforms};
use ph2d_vector::VectorScene;

const JANELA: ph2d_host::WindowSize = ph2d_host::WindowSize {
    width: 2560,
    height: 1080,
};
/// A altura da câmera de abertura (`Camera2d::default`).
const ABERTURA: f32 = 10.0;

struct Banco {
    gpu: ph2d_gpu::GpuContext,
    principal: VelloPass,
    fx: crate::fx_live::FxLive,
    scene: ph2d_vec_scene::VecScene,
    sim: ph2d_ecs::SimWorld,
    map: ph2d_vec_entities::entities::VecEntityMap,
}

impl Banco {
    fn novo() -> Option<Self> {
        let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).ok()?;
        let principal = VelloPass::new(
            &gpu,
            wgpu::TextureFormat::Rgba8Unorm,
            (JANELA.width, JANELA.height),
        )
        .ok()?;
        let (scene, sim, map, _) = super::gpu::nuvem_da_cena_41();
        Some(Self {
            gpu,
            principal,
            fx: crate::fx_live::FxLive::default(),
            scene,
            sim,
            map,
        })
    }

    /// Um quadro do app: o recook do FX, o desenho da cena e o render principal.
    fn quadro(&mut self, c: Camera2d) {
        let cam = c.world_to_screen_affine(JANELA);
        let tela = (JANELA.width, JANELA.height);
        let vista = [0.0, 0.0, f64::from(tela.0), f64::from(tela.1)];
        let (view, pats) = (VecViewState::default(), PatternTiles::new());
        let (xf, live) = (VecXforms::new(), LiveGeometry::new());
        self.fx.recook(
            &self.scene,
            &self.sim,
            &self.map,
            &xf,
            &live,
            &live,
            &view,
            &pats,
            cam,
            vista,
            &self.gpu,
            wgpu::TextureFormat::Rgba8Unorm,
            &mut self.principal,
        );
        let mut alvo = VectorScene::new();
        ph2d_vec_render::dispatch(
            &self.scene,
            &view,
            &xf,
            &live,
            self.fx.images(),
            &WidgetSkins::new(),
            &pats,
            &BrushArts::new(),
            &DilatedPaints::new(),
            cam,
            &mut alvo,
        );
        self.principal
            .render_to_intermediate(&self.gpu, alvo.inner(), tela, ph2d_vector::Color::TRANSPARENT)
            .expect("render");
        let _ = self.gpu.device.poll(wgpu::PollType::wait_indefinitely());
    }
}

/// A VRAM deste processo segundo o driver (`nvidia-smi`), em MiB — inclui o que o Vello segura.
fn vram_do_processo_mib() -> Option<u64> {
    let out = std::process::Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    let eu = std::process::id().to_string();
    String::from_utf8_lossy(&out.stdout).lines().find_map(|l| {
        let (pid, mib) = l.split_once(',')?;
        (pid.trim() == eu).then(|| mib.trim().parse().ok())?
    })
}

/// A memória da placa AMD (VRAM + GTT, a máquina inteira) segundo o `amdgpu`, em MiB. A vez da placa
/// corre os testes na Radeon de bordo, onde o `nvidia-smi` não vê o processo.
fn amdgpu_mib() -> Option<u64> {
    let le = |f: &str| -> Option<u64> {
        std::fs::read_to_string(format!("/sys/class/drm/card0/device/{f}"))
            .ok()?
            .trim()
            .parse()
            .ok()
    };
    Some((le("mem_info_vram_used")? + le("mem_info_gtt_used")?) >> 20)
}

/// A roda de `de` a `ate` (alturas de câmera), um entalhe por quadro, centrada em `centro`.
fn roda(de: f32, ate: f32, centro: [f32; 2]) -> Vec<Camera2d> {
    let fator = if ate < de {
        Camera2d::WHEEL_ZOOM_PER_NOTCH
    } else {
        1.0 / Camera2d::WHEEL_ZOOM_PER_NOTCH
    };
    let mut h = de;
    let mut v = Vec::new();
    loop {
        h = Camera2d::zoomed(h, fator);
        v.push(Camera2d::new(centro, h));
        if (ate < de && h <= ate) || (ate >= de && h >= ate) {
            return v;
        }
    }
}

/// O pior caso da G11: a sombra de borrão enorme leva a janela ao lado da placa (8192).
fn sombra_grande(b: &mut Banco) {
    let sombra = ph2d_ecs::FxOp {
        radius: 2.0,
        ..ph2d_ecs::FxOp::new(ph2d_ecs::FxOp::DROP_SHADOW)
    };
    let ids: Vec<_> = b.scene.paths().iter().map(|p| p.id).collect();
    crate::fx_live::set_filter(
        &mut b.sim,
        &b.map,
        &ids,
        Some(ph2d_ecs::VecFilter::from_ops(vec![sombra])),
    );
}

/// O passeio do dono: a abertura, a roda ao máximo, o pan lá em cima, a roda de volta (a acabar
/// EXACTAMENTE na abertura) e os quadros parados — `N - 1` e depois o `N`-ésimo.
fn fases() -> Vec<(&'static str, Vec<Camera2d>)> {
    let centro = [0.0, 0.0];
    let abertura = Camera2d::new(centro, ABERTURA);
    let mut volta = roda(Camera2d::ZOOM_MIN_HEIGHT_WORLD, ABERTURA, centro);
    volta.push(abertura);
    let n = crate::fx_live::QUADROS_PARADOS as usize;
    vec![
        ("abertura", vec![abertura; 10]),
        (
            "roda até o máximo",
            roda(ABERTURA, Camera2d::ZOOM_MIN_HEIGHT_WORLD, centro),
        ),
        (
            "pan no máximo",
            (0..40)
                .map(|q| {
                    Camera2d::new(
                        [0.01 * q as f32, 0.003 * q as f32],
                        Camera2d::ZOOM_MIN_HEIGHT_WORLD,
                    )
                })
                .collect(),
        ),
        ("roda de volta", volta),
        ("parado N-1 quadros", vec![abertura; n - 1]),
        ("parado o N-ésimo", vec![abertura; 1]),
    ]
}

/// O fim de cada fase: os bytes do FX (`[trabalho, saída, mostradas]`) e o maior lote do scratch.
fn percorre(b: &mut Banco, imprime: bool) -> Vec<([u64; 3], (u32, u32))> {
    let mut fins = Vec::new();
    for (nome, camaras) in fases() {
        for c in camaras {
            b.quadro(c);
        }
        let bytes = b.fx.gpu_bytes();
        let lote = b.fx.pico_lote();
        if imprime {
            let [t, s, m] = bytes.map(|v| v >> 20);
            eprintln!(
                "[f35] {nome:<20} trabalho {t} MiB · saída {s} MiB · mostradas {m} MiB · \
                 lote {lote:?} · VRAM do processo {:?} MiB · amdgpu {:?} MiB",
                vram_do_processo_mib(),
                amdgpu_mib()
            );
        }
        fins.push((bytes, lote));
    }
    fins
}

/// **A sonda** — a tabela do plano F35 §1: o que fica preso por fase (`F35_SOMBRA_GRANDE=1` = o
/// pior caso).
/// `bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-vec --lib fx_memoria_sonda -- --ignored --nocapture`
#[test]
#[ignore = "needs a real GPU device; sonda F35 (a memória presa)"]
fn fx_memoria_sonda() {
    let Some(mut b) = Banco::novo() else {
        return;
    };
    let info = b.gpu.adapter.get_info();
    eprintln!("[f35] placa: {} ({:?})", info.name, info.backend);
    if std::env::var_os("F35_SOMBRA_GRANDE").is_some() {
        sombra_grande(&mut b);
    }
    eprintln!(
        "[f35] antes do FX: VRAM do processo {:?} MiB · amdgpu {:?} MiB",
        vram_do_processo_mib(),
        amdgpu_mib()
    );
    percorre(&mut b, true);
}

/// **G13 (placa) — depois do zoom extremo, PARADO, o FX devolve o que sobra.** O passeio do dono com
/// o filtro da cena 41 e com o pior caso (a sombra que leva a janela a 8192): `N - 1` quadros parados
/// ainda seguram o pico (a roda não solta a meio do gesto); no `N`-ésimo o trabalho, a saída e o lote
/// do scratch voltam ao que a abertura segura.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn after_the_extreme_zoom_a_still_scene_gives_the_memory_back() {
    for pior in [false, true] {
        let Some(mut b) = Banco::novo() else {
            return;
        };
        if pior {
            sombra_grande(&mut b);
        }
        let fins = percorre(&mut b, false);
        let [abertura, _, pico, _, quase, solto] = fins[..] else {
            panic!("seis fases");
        };
        let transitorio = |f: ([u64; 3], (u32, u32))| f.0[0] + f.0[1];
        let area = |l: (u32, u32)| u64::from(l.0) * u64::from(l.1);
        assert!(
            transitorio(pico) > 2 * transitorio(abertura),
            "pior={pior}: o zoom extremo devia pedir mais que a abertura: {pico:?} × {abertura:?}"
        );
        assert!(
            transitorio(quase) == transitorio(pico),
            "pior={pior}: soltou antes de N quadros parados: {quase:?} × {pico:?}"
        );
        assert!(
            transitorio(solto) <= transitorio(abertura) && area(solto.1) <= area(abertura.1),
            "pior={pior}: parado, o FX segura {solto:?}; a abertura segurava {abertura:?}"
        );
    }
}

/// **Sonda do preço de soltar** — quanto custa recriar o que o encolhimento solta: o renderer do
/// scratch (`VelloPass::new`) e o passe da pilha, o MÍNIMO de 7 (MEDIR_VELOCIDADE).
#[test]
#[ignore = "needs a real GPU device; sonda F35 (o preço de recriar)"]
fn fx_memoria_sonda_do_preco() {
    let Some(b) = Banco::novo() else {
        return;
    };
    let mut vello = f64::MAX;
    let mut pilha = f64::MAX;
    for _ in 0..7 {
        let t = std::time::Instant::now();
        let p = VelloPass::new(&b.gpu, wgpu::TextureFormat::Rgba8Unorm, (2560, 1080)).expect("v");
        let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
        vello = vello.min(t.elapsed().as_secs_f64() * 1e3);
        drop(p);
        let t = std::time::Instant::now();
        let s = ph2d_render::FxStackPass::new(&b.gpu);
        let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
        pilha = pilha.min(t.elapsed().as_secs_f64() * 1e3);
        drop(s);
    }
    eprintln!("[f35-preco] VelloPass::new {vello:.1} ms · FxStackPass::new {pilha:.1} ms (mínimo de 7)");
}

/// **G16 (placa) — o fim de um episódio sem memória REFAZ as texturas do FX.** Uma textura criada
/// sob recusa é inválida para sempre, e o memo nunca mais a re-coze: a forma filtrada ficava fora da
/// tela depois de a placa voltar (foto F35 §2). Sobe a geração (`ph2d_gpu::memoria`) e confere que a
/// textura registada da forma é outra, e que a forma continua com imagem.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn the_end_of_an_out_of_memory_episode_rebuilds_the_fx_textures() {
    let Some(mut b) = Banco::novo() else {
        return;
    };
    let id = b.scene.paths().first().expect("nuvem").id;
    let abertura = Camera2d::new([0.0, 0.0], ABERTURA);
    for _ in 0..3 {
        b.quadro(abertura);
    }
    let antes = b.fx.textura_de(id).cloned().expect("textura da nuvem");
    // Um episódio inteiro no vigia do processo: uma recusa e o fim dela.
    let v = &ph2d_gpu::memoria::VIGIA;
    let g0 = v.geracao();
    v.anota_em(0);
    let _ = v.mudou_em(0);
    let _ = v.mudou_em(10 * ph2d_gpu::memoria::JANELA_MS);
    assert_eq!(v.geracao(), g0 + 1, "o episódio devia ter acabado");
    b.quadro(abertura);
    let depois = b.fx.textura_de(id).cloned().expect("textura da nuvem depois");
    assert!(
        antes != depois && b.fx.images().len() == 1,
        "o fim do episódio não refez a textura do FX (a mesma: {})",
        antes == depois
    );
}
