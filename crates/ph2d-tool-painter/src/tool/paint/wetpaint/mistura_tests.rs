//! **A MISTURA do Wet Paint é a do Rebelle** (W1e, F6; dono 09/10: *«não acontece em tempo real mas
//! apenas no mouse up… acontece da mesma maneira sobre tinta seca e tinta úmida»*).
//!
//! A cena da sonda [`super::sonda_mistura`]: vermelho horizontal, azul vertical por cima, com o vermelho
//! MOLHADO (0,5 s) ou SECO (Dry canvas). A régua é a de `ferramentas/velatura.py`, em luz linear: `γ` =
//! onde o cruzamento cai entre «o azul por cima» (0) e «multiplicar as duas» (1); a SANGRIA = quanto o
//! vermelho a 15 px do cruzamento ganhou de azul. O Rebelle 8.3.4 na mesma cena (05 §4b): seco γ
//! `0,42–0,51` durante e depois, sangria `+1…+2`; molhado γ `0,62` durante, sangria `+12…+21`.
//!
//! ⚠️ **O controlo está DENTRO do gate:** a mesma cena com o port (γ 0, água parada no traço) tem de
//! REPROVAR a barra — sem isso um verde não diria se a régua discrimina.
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};

const W: usize = 600;
const H: usize = 300;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn media(px: &[u8], x0: usize, x1: usize, y0: usize, y1: usize) -> [f64; 3] {
    let mut s = [0.0; 3];
    let mut n = 0.0;
    for y in y0..y1 {
        for x in x0..x1 {
            for (c, v) in s.iter_mut().enumerate() {
                *v += f64::from(px[(y * W + x) * 4 + c]);
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

fn lin(c: f64) -> f64 {
    let c = c / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// O que a régua lê de uma tela: `γ` (média do G e do B — o R do cruzamento sobe com a MISTURA, e
/// aí não mede a velatura) e a sangria: a FRACÇÃO do caminho vermelho → azul (canal azul) que o vermelho a
/// 15 px do cruzamento andou desde ANTES do azul (o MESMO sítio — a textura do papel não entra). ⚠️ Fracção e
/// não níveis desde a Opacity do Rebelle (10/10): a tinta diluída tem o caminho 4× mais curto e os níveis
/// encolhiam com ela; as barras antigas (+8 e ±3 níveis sobre o caminho de 180 da tinta cheia) são as
/// mesmas em fracção (0,044 e 0,017).
#[derive(Clone, Debug)]
pub(super) struct Leitura {
    pub(super) gama: f64,
    pub(super) sangria: f64,
}

pub(super) fn le(px: &[u8], antes: &[u8]) -> Leitura {
    let cruz = media(px, 295, 306, 145, 156);
    let verm = media(px, 195, 206, 145, 156);
    let azul = media(px, 295, 306, 60, 100);
    let perto = media(px, 315, 330, 145, 156);
    let g = |c: usize| {
        let (x, v, b) = (lin(cruz[c]), lin(verm[c]), lin(azul[c]));
        (x - b) / (v * b - b)
    };
    let antes_perto = media(antes, 315, 330, 145, 156)[2];
    Leitura {
        gama: (g(1) + g(2)) / 2.0,
        sangria: (perto[2] - antes_perto) / (azul[2] - antes_perto).max(1.0),
    }
}

// A velatura das junções, o quanto tapa (`Sim::esconde`) e o reforço da tinta molhada forçados nas mesmas
// cenas (`None` = o produto). Por THREAD: os portões correm em paralelo e uma variante forçada não pode
// vazar para a cena de outro teste.
thread_local! {
    pub(super) static SONDA_VELATURA: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
    pub(super) static SONDA_ESCONDE: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
    pub(super) static SONDA_MOLHADA: std::cell::Cell<Option<f64>> = const { std::cell::Cell::new(None) };
}

/// A cena: devolve as telas ANTES do azul, DURANTE o azul (ponteiro em baixo) e 2 s DEPOIS do Up.
fn cena(seco: bool, produto: bool) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    cena_com(seco, produto, None, false)
}

/// [`cena`] pelo PRODUTO com a Live Water autorada (`viva`): o reconcile põe a água e a velatura.
fn cena_produto(seco: bool, viva: bool, cold: bool) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    cena_toda(seco, true, None, false, viva, cold)
}

/// [`cena`] com a velatura do filme no render forçada (`filme`) e a Água ao vivo (`viva`, W1g).
pub(super) fn cena_com(
    seco: bool,
    produto: bool,
    filme: Option<f64>,
    viva: bool,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    cena_toda(seco, produto, filme, viva, false, false)
}

/// A cena inteira: `viva` liga a água no traço DIRECTAMENTE no motor (as sondas); `opcao` liga a Live
/// Water autorada (o produto).
pub(super) fn cena_toda(
    seco: bool,
    produto: bool,
    filme: Option<f64>,
    viva: bool,
    opcao: bool,
    cold: bool,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let b = Biblioteca::carregar(&raiz, None);
    let p = &b
        .entradas
        .iter()
        .find(|e| e.preset.parametros.name == "Wet Round")
        .expect("o pincel de fábrica")
        .preset;
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    assert!(t.wet_preset_aplicar(p).is_empty());
    t.wet_preset_aplicar_volumes(&p.parametros.volumes);
    t.paint.wetpaint.tilt_on = false;
    t.set_brush_size_px(30.0);
    t.set_wet_agua_viva(opcao);
    if cold {
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
    }
    assert!(t.ensure_wet_session());
    if viva {
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim
            .sim_while_painting = true;
    }
    if !produto {
        // O PORT inteiro: sem velatura, sem a água no traço, sem a lei do Rebelle (o tapar e a tinta
        // molhada mais forte).
        let s = t.paint.wetpaint.session.as_mut().expect("sessão");
        s.engine.sim.glaze = 0.0;
        s.engine.sim.glaze_filme = 0.0;
        s.engine.sim.sim_while_painting = false;
        s.engine.sim.lei = ph2d_wet_paint::assenta::Lei::Porto;
        s.engine.sim.molhada_escurece = 1.0;
    }
    if let Some(f) = filme {
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim
            .glaze_filme = f;
    }
    if let Some(g) = SONDA_VELATURA.get() {
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim
            .glaze = g;
    }
    if let Some(h) = SONDA_ESCONDE.get() {
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim
            .esconde = h;
    }
    if let Some(k) = SONDA_MOLHADA.get() {
        t.paint
            .wetpaint
            .session
            .as_mut()
            .expect("sessão")
            .engine
            .sim
            .molhada_escurece = k;
    }
    t.set_brush_color_srgb8([200, 30, 30]);
    t.on_canvas_pointer(cp([100.0, 150.0], PointerPhase::Down));
    for k in 1..=133 {
        t.on_canvas_pointer(cp([100.0 + 3.0 * k as f32, 150.0], PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(cp([500.0, 150.0], PointerPhase::Up));
    for _ in 0..30 {
        t.on_tick(1000.0 / 60.0);
    }
    if seco {
        assert!(t.route_brush_wetpaint_event(&PanelEvent::Click(
            crate::ids::PAINTER_WETPAINT_DRYCANVAS
        )));
        for _ in 0..30 {
            t.on_tick(1000.0 / 60.0);
        }
    }
    let antes = t.canvas_rgba.to_vec();
    t.set_brush_color_srgb8([30, 60, 210]);
    t.on_canvas_pointer(cp([300.0, 40.0], PointerPhase::Down));
    let mut k: u16 = 0;
    while 40.0 + 3.0 * f32::from(k) < 260.0 {
        k += 1;
        t.on_canvas_pointer(cp([300.0, 40.0 + 3.0 * f32::from(k)], PointerPhase::Move));
        t.on_tick(8.0);
    }
    for _ in 0..6 {
        t.on_tick(1000.0 / 60.0);
    }
    let durante = t.canvas_rgba.to_vec();
    t.on_canvas_pointer(cp([300.0, 260.0], PointerPhase::Up));
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    (antes, durante, t.canvas_rgba.to_vec())
}

/// A barra (o Rebelle com folga): o seco VELA (γ entre 0,30 e 0,70, durante e depois) e não sangra
/// (`|sangria| ≤ 0,05` do caminho — o Rebelle diluído mede +0,02…+0,05; a barra 0,017 vinha dos 3 níveis da
/// tinta cheia); o molhado SANGRA (`≥ 0,044`) e a mistura vê-se a pintar (γ durante `≥ 0,30`). O Rebelle diluído (W1e, as fotos re-medidas 10/10): seco +0,02…+0,05, molhado **0,27** — o
/// nosso molhado anda ~0,08 desde sempre (aberto no 05 §4b «Conferência»).
pub(super) fn barra(seco: (Leitura, Leitura), molhado: (Leitura, Leitura)) -> Result<(), String> {
    let mut f = Vec::new();
    for (nome, l) in [("seco durante", &seco.0), ("seco depois", &seco.1)] {
        if !(0.30..=0.70).contains(&l.gama) {
            f.push(format!(
                "{nome}: γ {:.2} fora de 0,30–0,70 (Rebelle 0,42–0,51)",
                l.gama
            ));
        }
    }
    if seco.1.sangria.abs() > 0.05 {
        f.push(format!(
            "seco: sangria {:+.3} (Rebelle +0,02…+0,05)",
            seco.1.sangria
        ));
    }
    if molhado.1.sangria < 0.044 {
        f.push(format!(
            "molhado: sangria {:+.3} < +0,044 (Rebelle +0,27)",
            molhado.1.sangria
        ));
    }
    if molhado.0.gama < 0.30 {
        f.push(format!(
            "molhado durante: γ {:.2} < 0,30 (Rebelle 0,62)",
            molhado.0.gama
        ));
    }
    if f.is_empty() {
        Ok(())
    } else {
        Err(f.join(" · "))
    }
}

#[test]
fn azul_sobre_vermelho_vela_no_seco_e_sangra_no_molhado_como_no_rebelle() {
    let ler = |(z, a, b): (Vec<u8>, Vec<u8>, Vec<u8>)| (le(&a, &z), le(&b, &z));
    let produto = barra(ler(cena(true, true)), ler(cena(false, true)));
    let port = barra(ler(cena(true, false)), ler(cena(false, false)));
    assert!(
        port.is_err(),
        "CONTROLO: o port (γ 0, água parada no traço) passou a barra — a régua não discrimina"
    );
    if let Err(e) = produto {
        panic!("a mistura do produto falhou a barra do Rebelle: {e}");
    }
}

/// **Com a Live Water (W1g) a mistura continua na barra do Rebelle** — pelo produto (a opção autorada), sem
/// papel (o papel liso da bancada do Rebelle): a barra inteira. Com o papel Cold escolhido, o γ (a sangria
/// molhada no Cold é baixa com a água ao vivo, 05 §4b — o Rebelle não foi medido no Cold). O controlo: sem o
/// TAPAR da lei do Rebelle (`esconde` 1) o seco vai a γ 0,75 e reprova.
#[test]
fn com_a_live_water_a_mistura_continua_na_barra_do_rebelle() {
    let ler = |(z, a, b): (Vec<u8>, Vec<u8>, Vec<u8>)| (le(&a, &z), le(&b, &z));
    SONDA_ESCONDE.set(Some(1.0));
    let controlo = barra(
        ler(cena_produto(true, true, false)),
        ler(cena_produto(false, true, false)),
    );
    SONDA_ESCONDE.set(None);
    assert!(
        controlo.is_err(),
        "CONTROLO: sem o tapar a Live Water passou a barra — a régua não discrimina"
    );
    if let Err(e) = barra(
        ler(cena_produto(true, true, false)),
        ler(cena_produto(false, true, false)),
    ) {
        panic!("a mistura com a Live Water falhou a barra do Rebelle: {e}");
    }
    let (seco, molhado) = (
        ler(cena_produto(true, true, true)),
        ler(cena_produto(false, true, true)),
    );
    for (nome, g) in [("seco durante", seco.0.gama), ("seco depois", seco.1.gama)] {
        assert!(
            (0.30..=0.70).contains(&g),
            "Live Water no Cold: {nome} γ {g:.2} fora de 0,30–0,70"
        );
    }
    assert!(
        molhado.0.gama >= 0.30,
        "Live Water no Cold: molhado durante γ {:.2} < 0,30",
        molhado.0.gama
    );
}

/// **Sonda (W1f): a velatura do filme no render**, com a água só depois do Up — o γ e a sangria por
/// valor, no MESMO processo.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_velatura_do_filme() {
    let viva = std::env::var("PH2D_SONDA_VIVA").is_ok_and(|v| v == "1");
    let filmes: Vec<f64> = std::env::var("PH2D_SONDA_FILMES")
        .ok()
        .map(|v| v.split(',').filter_map(|s| s.parse().ok()).collect())
        .unwrap_or_else(|| vec![0.47, 0.35, 0.25, 0.20, 0.15, 0.0]);
    for f in filmes {
        let ler = |(z, a, b): (Vec<u8>, Vec<u8>, Vec<u8>)| (le(&a, &z), le(&b, &z));
        let (s, m) = (
            ler(cena_com(true, true, Some(f), viva)),
            ler(cena_com(false, true, Some(f), viva)),
        );
        println!(
            "[filme {f:.2} viva {viva}] seco γ durante {:.2} depois {:.2} sangria {:+.3} · molhado γ durante {:.2} depois {:.2} sangria {:+.3} · barra {:?}",
            s.0.gama,
            s.1.gama,
            s.1.sangria,
            m.0.gama,
            m.1.gama,
            m.1.sangria,
            barra((s.0.clone(), s.1.clone()), (m.0.clone(), m.1.clone())).err()
        );
    }
}
