//! O toque preparado (Bug #72) não muda um byte, e o 1.º toque recebe mesmo o que foi adiantado.

use super::preparo::{liga_no_teste, usados_no_teste};
use super::sonda_bordas_longe::por_nome;
use crate::tool::PainterTool;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

const LADO: usize = 240;

fn tool(nome: &str) -> PainterTool {
    let raiz = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes");
    let lib = Biblioteca::carregar(&raiz, None);
    let mut t = PainterTool::default();
    t.abre_a_sprite(vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_tool_mode("wetpaint");
    t.preset_aplicar(por_nome(&lib, nome)).expect("preset");
    t.set_brush_size_px(30.0);
    t
}

/// Os tiques de quem espera o artista: até o fio do preparo acabar (o relógio fixo desliga o preparo).
fn espera_o_preparo(t: &mut PainterTool) {
    for _ in 0..500 {
        t.on_tick(1.0 / 60.0);
        if t.preparo_pronto() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("o preparo não acabou");
}

fn traco(t: &mut PainterTool, y: f32) {
    let p = |x: f32, phase| CanvasPointer {
        pos: [x, y],
        pressure: 0.8,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(30.0, PointerPhase::Down));
    for k in 1..=16 {
        t.on_canvas_pointer(p(30.0 + 10.0 * k as f32, PointerPhase::Move));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(p(200.0, PointerPhase::Up));
    for _ in 0..8 {
        t.on_tick(1000.0 / 60.0);
    }
}

fn pinta(t: &mut PainterTool) -> Vec<u8> {
    t.set_wet_relogio_fixo(true);
    traco(t, 80.0);
    traco(t, 150.0);
    let _ = t.take_preview_arc();
    t.canvas_rgba.to_vec()
}

#[test]
fn o_toque_preparado_pinta_os_mesmos_bytes_e_recebe_o_motor_e_o_papel() {
    for nome in ["Wet Wash", "Wet Round", "Wet Liner", "Marker Bullet"] {
        liga_no_teste(false);
        let referencia = pinta(&mut tool(nome));
        assert_ne!(
            referencia,
            vec![255u8; LADO * LADO * 4],
            "{nome}: não pintou"
        );
        liga_no_teste(true);
        let mut t = tool(nome);
        espera_o_preparo(&mut t);
        let tela = pinta(&mut t);
        assert_eq!(
            usados_no_teste(),
            (1, 1),
            "{nome}: o toque não usou o preparo"
        );
        liga_no_teste(false);
        assert!(tela == referencia, "{nome}: o preparo mudou a tinta");
    }
}

#[test]
fn o_papel_de_outra_chave_nao_entra() {
    liga_no_teste(false);
    let mut r = tool("Wet Wash");
    r.paint.brush.paper_depth *= 0.5;
    let referencia = pinta(&mut r);
    liga_no_teste(true);
    let mut t = tool("Wet Wash");
    espera_o_preparo(&mut t);
    // O pincel muda entre o preparo e o toque: o motor serve, o plano não.
    t.paint.brush.paper_depth *= 0.5;
    let tela = pinta(&mut t);
    assert_eq!(usados_no_teste(), (1, 0));
    liga_no_teste(false);
    assert!(tela == referencia);
}

/// A grade muda entre o preparo e o toque: o motor preparado não entra, e o toque faz a conta dele.
#[test]
fn o_motor_de_outra_grade_nao_entra() {
    liga_no_teste(false);
    let mut r = tool("Wet Wash");
    r.paint.wetpaint.grid_ratio = 2;
    let referencia = pinta(&mut r);
    liga_no_teste(true);
    let mut t = tool("Wet Wash");
    t.paint.wetpaint.grid_ratio = 1;
    espera_o_preparo(&mut t);
    t.paint.wetpaint.grid_ratio = 2;
    let tela = pinta(&mut t);
    assert_eq!(usados_no_teste(), (0, 0));
    liga_no_teste(false);
    assert!(tela == referencia);
}

/// O motor preparado de outra grade não fica a ocupar memória durante a sessão.
#[test]
fn o_toque_nao_deixa_motor_preparado_durante_a_sessao() {
    liga_no_teste(true);
    let mut t = tool("Wet Wash");
    t.paint.wetpaint.grid_ratio = 1;
    espera_o_preparo(&mut t);
    t.paint.wetpaint.grid_ratio = 2;
    let _ = pinta(&mut t);
    let pronto = t.preparo_pronto();
    liga_no_teste(false);
    assert!(!pronto);
}

/// Papel com textura: o plano preparado tem de ser o que o toque calcularia, ao byte.
#[test]
fn o_papel_preparado_com_textura_pinta_os_mesmos_bytes() {
    let com_papel = |nome: &str, dente: f32| {
        let mut t = tool(nome);
        t.set_brush_paper_kind(ph2d_painter_brush::TextureKind::PaperCold.to_u8());
        t.set_substrate_depth(0.7);
        t.paint.brush.paper_depth = dente;
        t
    };
    let mut sentiram = 0;
    for nome in ["Wet Wash", "Wet Round", "Wet Liner", "Marker Bullet"] {
        liga_no_teste(false);
        let referencia = pinta(&mut com_papel(nome, 0.8));
        // Um pincel que não lê o dente é cego a esta prova.
        sentiram += usize::from(pinta(&mut com_papel(nome, 0.4)) != referencia);
        liga_no_teste(true);
        let mut t = com_papel(nome, 0.8);
        espera_o_preparo(&mut t);
        let tela = pinta(&mut t);
        assert_eq!(usados_no_teste(), (1, 1), "{nome}");
        liga_no_teste(false);
        assert!(
            tela == referencia,
            "{nome}: o papel preparado mudou a tinta"
        );
    }
    assert!(sentiram > 0, "nenhum pincel leu o dente: a prova é cega");
}

#[test]
fn fora_do_wet_o_preparo_larga_a_grade() {
    liga_no_teste(true);
    let mut t = tool("Wet Round");
    espera_o_preparo(&mut t);
    t.paint.paint_mode = crate::tool::paint::PaintMode::Paint;
    t.on_tick(1.0 / 60.0);
    let pronto = t.preparo_pronto();
    liga_no_teste(false);
    assert!(!pronto);
}
