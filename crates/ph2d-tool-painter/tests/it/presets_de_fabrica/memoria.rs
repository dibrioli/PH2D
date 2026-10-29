//! A MEMÓRIA da lei da largura (Dabber, L13) e as cópias do Wet (spray, simetria) — partidas de
//! `presets_de_fabrica.rs` pelo tecto de 700 linhas.

use super::*;

/// O pico de memória do processo (`VmHWM`), em MiB.
pub(super) fn pico_mib() -> f64 {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find_map(|l| l.strip_prefix("VmHWM:"))
        .and_then(|r| r.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
        / 1024.0
}

/// **Sonda da MEMÓRIA da lei da largura** (J2 Dabber, L13): UM raio por processo (`VmHWM` é do processo), o traço
/// de [`largura_de`] com `PH2D_SONDA_PRESET` (Dabber), `PH2D_SONDA_RAIO`, `PH2D_SONDA_CRUA` (1), `PH2D_SONDA_JITTER`
/// (o do preset) e `PH2D_SONDA_QUADROS` (os da lei; `0` = só o gesto). Imprime o pico depois do gesto e depois da
/// água, as lanes e as janelas. O portão [`a_memoria_do_dabber_cabe`] corre-a num processo filho.
#[test]
#[ignore = "sonda: a memória da lei da largura, um raio por processo"]
fn sonda_memoria_da_lei() {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    let var = |k: &str| std::env::var(k).ok();
    let nome = var("PH2D_SONDA_PRESET").unwrap_or_else(|| "Dabber".into());
    let raio: f32 = var("PH2D_SONDA_RAIO").map_or(300.0, |v| v.parse().unwrap());
    let crua = var("PH2D_SONDA_CRUA").as_deref() != Some("0");
    let quadros: u32 = var("PH2D_SONDA_QUADROS").map_or(QUADROS_DA_LEI, |v| v.parse().unwrap());
    let mut p = ficheiros_do_wet()
        .into_iter()
        .map(|(_, b)| ler(&b).unwrap())
        .find(|p| p.parametros.name == nome)
        .expect("o preset de fábrica");
    // `PH2D_SONDA_RECEITA=1`: o preset da RECEITA de hoje (antes de o regerar), com a tabela do ficheiro.
    if var("PH2D_SONDA_RECEITA").as_deref() == Some("1") {
        let r = receitas().find(|r| r.nome == nome).expect("a receita");
        let tabela = std::mem::take(&mut p.parametros.width_table);
        p.parametros = parametros(r);
        p.parametros.width_table = tabela;
    }
    // `PH2D_SONDA_OLEO=0`: o mesmo preset sem a caixa Real Oil (J6: o custo do óleo é a diferença).
    if var("PH2D_SONDA_OLEO").as_deref() == Some("0") {
        p.parametros.engine.remove(CHAVE_REAL_OIL);
    }
    if let Some(j) = var("PH2D_SONDA_JITTER") {
        p.parametros.brush.insert(
            "jitter".into(),
            serde_json::json!(j.parse::<f64>().unwrap()),
        );
    }
    let escala = if crua { (raio / 500.0).max(1.0) } else { 1.0 };
    let h = ((raio * 6.0) as u32 + 80).max(120);
    let w = crate::sonda_gestos::W;
    let cy = h as f32 * 0.5;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&direito(&p)).is_empty());
    if crua {
        t.set_wet_largura_crua(escala);
    }
    t.set_brush_size_px(raio / escala);
    t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
    let t0 = std::time::Instant::now();
    crate::sonda_gestos::gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
    let gesto = pico_mib();
    let mut janelas = Vec::new();
    t.wet_sonda_motor(|e| {
        janelas = e
            .lane_trails
            .iter()
            .map(ph2d_wet_paint::trail::Trail::window_half_for_measure)
            .collect();
    });
    for _ in 0..quadros {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, ph) = t.run_full();
    println!(
        "[memoria] {nome} raio {raio} crua {crua} · tela {w}×{h} · gesto {gesto:.0} MiB · água {:.0} MiB · lanes {} \
         · meias-larguras {janelas:?} · largura {:.1} · {:.1} s",
        pico_mib(),
        janelas.len(),
        largura_que_se_ve(&px, pw, ph),
        t0.elapsed().as_secs_f64()
    );
}

/// ⭐ O binário que corre AGORA, para o re-executar num filho. Nunca o CAMINHO (`current_exe`): o cargo religa o mesmo
/// `it-<hash>` (outra corrida, o `exame.sh` ao lado) gravando um ficheiro novo por cima, o inode deste processo fica
/// «(deleted)» e o filho dava `NotFound` — ou, pior, corria o código NOVO. `/proc/self/exe` abre o inode que corre.
fn o_proprio_binario() -> std::path::PathBuf {
    if cfg!(target_os = "linux") {
        "/proc/self/exe".into()
    } else {
        std::env::current_exe().unwrap()
    }
}

/// O pico do GESTO (MiB) e as lanes da [`sonda_memoria_da_lei`] num processo filho (o `VmHWM` é do processo).
fn memoria_num_filho(raio: f32) -> (f64, usize) {
    let out = std::process::Command::new(o_proprio_binario())
        .args([
            "presets_de_fabrica::memoria::sonda_memoria_da_lei",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env("PH2D_SONDA_RAIO", raio.to_string())
        .env("PH2D_SONDA_QUADROS", "0")
        .env_remove("PH2D_SONDA_JITTER")
        .env_remove("PH2D_SONDA_PRESET")
        .output()
        .expect("o processo filho da sonda");
    let txt = String::from_utf8_lossy(&out.stdout);
    let linha = txt
        .split("[memoria]")
        .nth(1)
        .unwrap_or_else(|| panic!("a sonda não respondeu a raio {raio}:\n{txt}"));
    let num = |k: &str| -> f64 {
        linha
            .split(k)
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap()
    };
    println!("[memoria]{}", linha.lines().next().unwrap_or(""));
    (num(" gesto "), num(" lanes ") as usize)
}

/// Os tectos do PICO do gesto do Dabber na lei crua da largura (a do gerador), em MiB, medidos 20/10 (L13, smoke,
/// um processo por raio; 03_MOTORES §4 L13):
///
/// | raio | limiar `r` (o mutante: 4 lanes) | uma ponta = uma lane | tecto |
/// |---|---|---|---|
/// | 300  | 298 | 165 | 230 |
/// | 1000 | 2 966 | 1 221 | 1 800 |
///
/// (O Dabber de antes — lâmina, Jitter 0,5 — com o limiar `r`: 467 a raio 300 e 3 955 a 1 100, 6–7 lanes.)
///
/// J6 (dono 28/10): o Dabber de fábrica traz o **Real Oil** — o campo do óleo soma ~+60 B por célula ao gesto. Medido
/// 28/10 (smoke, um processo por raio, `PH2D_SONDA_OLEO`):
///
/// | raio | sem óleo | com óleo | mutante (limiar `r`) com óleo | tecto |
/// |---|---|---|---|---|
/// | 300  | 152 | 270 | 309 (2 lanes) | 290 |
/// | 1000 | 940 | 1 513 | 2 029 (2 lanes) | 1 800 |
///
/// ⚠️ Essa tabela é do perfil `smoke`; o portão corre no `ci-test` (`nextest-impacted.sh`), onde o MESMO código pesa mais
/// (em `d94e19a57e`, o commit do tecto 290: 303 MiB — o portão nasceu vermelho no perfil dele). Medido 29/10 no HEAD da
/// J6 (RAM do pico do gesto, um processo por raio):
///
/// | raio | ci-test sem óleo | ci-test com óleo | smoke com óleo | mutante (limiar `r`) ci-test | tecto |
/// |---|---|---|---|---|---|
/// | 300  | 165 | 295 | 286 | 322 (2 lanes) | 310 |
/// | 1000 | 968 | 1 563 | 1 497 | 2 058 (2 lanes) | 1 800 |
const TECTO_DA_MEMORIA_MIB: [(f32, f64); 2] = [(300.0, 310.0), (1000.0, 1800.0)];

/// ⭐ **A MEMÓRIA do Dabber cabe** (L13, EXAME §3.6): o Jitter espalha UMA ponta — o traço inteiro numa lane — e o
/// pico do gesto a raio 300 e 1000 fica abaixo do tecto medido. Mutação: o limiar da lane sem o Jitter
/// (`BrushSpec::limiar_da_lane` = `r`) parte o traço em lanes e passa os dois tectos.
#[test]
fn a_memoria_do_dabber_cabe() {
    for (raio, tecto) in TECTO_DA_MEMORIA_MIB {
        let (pico, lanes) = memoria_num_filho(raio);
        assert_eq!(
            lanes, 1,
            "raio {raio}: o Jitter partiu o traço em {lanes} lanes"
        );
        assert!(
            pico <= tecto,
            "raio {raio}: pico do gesto {pico:.0} MiB > tecto {tecto:.0}"
        );
    }
}

/// O que [`wet_com_copias`] mede: a tinta por metade (esquerda, direita), as lanes, os dabs e os que pousaram com
/// peso ZERO (a opacidade do dab no motor, `Dab::opacity`, já com o pigmento por comprimento).
struct Copias {
    tinta: [f64; 2],
    lanes: usize,
    dabs: usize,
    sem_pigmento: usize,
}

/// O Wet Round (o pincel que nasce sem preset: `pigmentSpacing` > 0, o pigmento por comprimento) com Jitter 0,5,
/// o Count e a simetria dados, um traço de `pts` a raio 16.
fn wet_com_copias(count: u32, simetria: bool, pts: &[[f32; 2]]) -> Copias {
    use ph2d_editor_core::{Tool, tool::RasterEditTool};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (w, h) = (crate::sonda_gestos::W, crate::sonda_gestos::H);
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    t.set_paint_media(PaintMedia::WetPaint);
    let mut p = t
        .wet_preset_capturar(&Parametros::novo(MEIO_DO_PRESET, "g", "copias"))
        .0;
    p.brush.insert("jitter".into(), serde_json::json!(0.5));
    p.brush
        .insert("spray_count".into(), serde_json::json!(count));
    let preset = Preset {
        parametros: p,
        origem: Origem::Fabrica,
        miniatura: ph2d_brush_presets::Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![255; 4],
        },
        texturas: Default::default(),
    };
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(&preset).is_empty());
    if simetria {
        t.toggle_symmetry_enabled();
    }
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
    let (dabs, zero) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (d2, z2) = (Arc::clone(&dabs), Arc::clone(&zero));
    t.wet_sonda_motor(|e| {
        e.on_dab = Some(Box::new(move |_b, d| {
            d2.fetch_add(1, Ordering::Relaxed);
            if d.opacity <= 1e-9 {
                z2.fetch_add(1, Ordering::Relaxed);
            }
        }));
    });
    crate::sonda_gestos::gesto(&mut t, pts, 3.0, |_| 1.0);
    let mut lanes = 0;
    t.wet_sonda_motor(|e| lanes = e.lane_trails.len());
    for _ in 0..60 {
        t.on_tick(1000.0 / 60.0);
    }
    let (px, pw, _) = t.run_full();
    let mut tinta = [0.0; 2];
    for (i, c) in px.as_chunks::<4>().0.iter().enumerate() {
        tinta[usize::from(i as u32 % pw >= pw / 2)] += f64::from(255 - c[0]);
    }
    Copias {
        tinta,
        lanes,
        dabs: dabs.load(Ordering::Relaxed),
        sem_pigmento: zero.load(Ordering::Relaxed),
    }
}

/// ⭐ **Cada marca do spray é uma ponta** (L13): no Wet com Count 3 e Jitter, as três marcas de cada ponto
/// pousam pigmento — cada uma na sua lane. Com a lane só pela distância (`r + 2·alcance`) as três caíam
/// na mesma e a 2.ª e a 3.ª, sem caminho andado, pousavam com peso 0 (o mutante: 3 → 1 lane, 250 de 2 100 dabs sem
/// pigmento; a tinta do traço ×2,50 → ×2,23 da do Count 1, porque o rasto leva a tinta de uma marca à outra).
#[test]
fn cada_marca_do_spray_pousa_no_wet() {
    let pts = [[150.0, 320.0], [850.0, 320.0]];
    let (um, tres) = (
        wet_com_copias(1, false, &pts),
        wet_com_copias(3, false, &pts),
    );
    let razao = (tres.tinta[0] + tres.tinta[1]) / (um.tinta[0] + um.tinta[1]);
    println!(
        "[copias] spray: Count 1 {} lane(s) · Count 3 {} lanes, {} de {} dabs sem pigmento · tinta ×{razao:.2}",
        um.lanes, tres.lanes, tres.sem_pigmento, tres.dabs
    );
    assert_eq!(um.lanes, 1, "Count 1 com Jitter: uma ponta, uma lane");
    assert_eq!(tres.lanes, 3, "Count 3: uma lane por marca");
    assert_eq!(
        tres.sem_pigmento, 0,
        "{} de {} marcas não pousaram pigmento",
        tres.sem_pigmento, tres.dabs
    );
    assert!(razao >= 2.0, "Count 3 pousa ×{razao:.2} a tinta do Count 1");
}

/// ⭐ **As duas cópias da simetria pintam as duas** (L13): o traço vertical a 12 px do eixo, com Jitter — as cópias
/// caem a < `r + 2·alcance` uma da outra e, só pela distância, partilhavam a lane: o rasto saltava entre elas e
/// dabs pousavam com peso 0. Mutação (a cópia ignorada): 1 lane, 71 de 790 dabs sem pigmento — sangra.
#[test]
fn as_copias_da_simetria_pintam_as_duas_no_wet() {
    let x = crate::sonda_gestos::W as f32 / 2.0 + 12.0;
    let c = wet_com_copias(1, true, &[[x, 120.0], [x, 520.0]]);
    let r = c.tinta[0].min(c.tinta[1]) / c.tinta[0].max(c.tinta[1]);
    println!(
        "[copias] simetria: {} lanes, {} de {} dabs sem pigmento · tinta esquerda {:.0} · direita {:.0} (razão {r:.3})",
        c.lanes, c.sem_pigmento, c.dabs, c.tinta[0], c.tinta[1]
    );
    assert_eq!(c.lanes, 2, "a simetria: uma lane por cópia");
    assert_eq!(
        c.sem_pigmento, 0,
        "{} de {} dabs da simetria não pousaram pigmento",
        c.sem_pigmento, c.dabs
    );
    // As cópias sobrepõem-se a 12 px do eixo e a 2.ª pousa sobre a tinta molhada da 1.ª: a razão é física, não 1.
    assert!(r >= 0.85, "uma cópia da simetria pinta {r:.3} da outra");
}
