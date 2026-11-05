//! **Cada pincel tem o seu Smooth Edges** (dono 04/11): o knob grava-se no preset por chave normal, sem a chave é `0`, a
//! miniatura lê o valor do preset, e a macro de quando ele era da tela toca igual a uma com o mesmo valor.

use super::*;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_brush_presets::{Miniatura, Origem, Parametros, Preset};
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
use serde_json::Value;

const W: usize = 360;
const H: usize = 200;
const CHAVE: &str = KNOB_DEFS[Knob::SmoothEdges as usize].key;

fn tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![240u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t.set_wet_relogio_fixo(true);
    t.set_brush_size_px(14.0);
    t
}

fn traco(t: &mut PainterTool, y: f32) {
    let p = |k: f32, phase| CanvasPointer {
        pos: [30.0 + 300.0 * k, y],
        pressure: 0.9,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(0.0, PointerPhase::Down));
    for i in 1..=24 {
        t.on_canvas_pointer(p(i as f32 / 24.0, PointerPhase::Move));
        t.on_tick(8.0);
    }
    t.on_canvas_pointer(p(1.0, PointerPhase::Up));
    t.on_tick(0.0);
}

/// Os valores do campo da célula que não são 0.
fn pousados(t: &PainterTool) -> Vec<f32> {
    t.paint
        .wetpaint
        .session
        .as_ref()
        .expect("uma sessão viva")
        .engine
        .active_grid()
        .aparencia
        .iter()
        .map(|a| a.suave)
        .filter(|&v| v != 0.0)
        .collect()
}

fn preset(parametros: Parametros, texturas: std::collections::BTreeMap<String, Vec<u8>>) -> Preset {
    Preset {
        parametros,
        origem: Origem::default(),
        miniatura: Miniatura {
            largura: 1,
            altura: 1,
            rgba: vec![0; 4],
        },
        texturas,
    }
}

#[test]
fn o_preset_leva_o_smooth_edges_de_ida_e_volta() {
    let mut a = tool();
    a.poe_o_suave(0.36);
    let (prm, texturas) = a.wet_preset_capturar(&Parametros::novo("WetPaint", "Teste", "Suave"));
    assert_eq!(prm.engine.get(CHAVE), Some(&0.36), "a captura");
    let mut b = tool();
    assert!(b.wet_preset_aplicar(&preset(prm, texturas)).is_empty());
    assert_eq!(b.paint.wetpaint.knobs.get(Knob::SmoothEdges), 0.36);
}

#[test]
fn o_preset_sem_a_chave_e_zero() {
    let mut a = tool();
    a.poe_o_suave(0.36);
    let (mut prm, texturas) =
        a.wet_preset_capturar(&Parametros::novo("WetPaint", "Teste", "Suave"));
    assert!(prm.engine.remove(CHAVE).is_some(), "a chave vinha");
    a.wet_preset_aplicar(&preset(prm, texturas));
    assert_eq!(a.paint.wetpaint.knobs.get(Knob::SmoothEdges), 0.0);
}

/// ⭐ **Aplicar um pincel com o valor: o próximo traço pousa-o na célula; aplicar outro sem ele: 0.** CONTROLO: repetir o
/// mesmo no 2.º traço faz o campo crescer, logo o 2.º traço toca nele. O preset leva a chave em memória: a receita de
/// fábrica prova-a `cada_pincel_traz_o_seu_smooth_edges`.
#[test]
fn aplicar_o_wet_liner_pousa_0_12_e_outro_pincel_pousa_0() {
    let mut liner = de_fabrica_ou_candidato("Wet Liner");
    liner.parametros.engine.insert(CHAVE.to_owned(), 0.12);
    let cena = |segundo: &Preset| {
        let mut t = tool();
        assert!(t.wet_preset_aplicar(&liner).is_empty());
        t.set_brush_size_px(14.0);
        traco(&mut t, 50.0);
        let um = pousados(&t);
        assert!(!um.is_empty(), "o Wet Liner não pousou nada");
        assert!(
            um.iter().all(|&v| v == 0.12),
            "o Wet Liner pousou outro valor"
        );
        assert!(t.wet_preset_aplicar(segundo).is_empty());
        t.set_brush_size_px(14.0);
        traco(&mut t, 150.0);
        (um.len(), pousados(&t).len())
    };
    let (um, dois) = cena(&de_fabrica_ou_candidato("Thick Impasto"));
    assert_eq!(um, dois, "o Thick Impasto pousou um Smooth Edges");
    let (um, dois) = cena(&liner);
    assert!(dois > um, "o controlo: o 2.º traço não tocou no campo");
}

/// **O Smooth Edges na miniatura é o do preset**, nunca o do painel: a miniatura de um preset a 0,52 difere da do mesmo a
/// 0, e o valor na mão de quem pede não entra nela.
#[test]
fn a_miniatura_pinta_com_o_valor_do_preset() {
    let mut a = tool();
    let capta = |a: &PainterTool| a.wet_preset_capturar(&Parametros::novo("WetPaint", "T", "S"));
    let (zero, tx) = capta(&a);
    a.poe_o_suave(0.52);
    let (suave, _) = capta(&a);
    let mini = |prm: &Parametros| crate::miniatura_do_preset(&preset(prm.clone(), tx.clone())).rgba;
    let difs = mini(&zero)
        .iter()
        .zip(&mini(&suave))
        .filter(|(x, y)| x != y)
        .count();
    assert!(difs > 0, "a miniatura não lê o valor do preset");
}

/// A macro de agora e, por cima dela, a de quando o Smooth Edges era da tela: o preset sem a chave e `wet.smooth_edges`.
fn estados(valor: f64) -> (String, String, std::collections::BTreeMap<String, Vec<u8>>) {
    let mut t = tool();
    t.poe_o_suave(valor);
    let (nova, tx) = t.macro_estado();
    let mut v: Value = serde_json::from_str(&nova).expect("o JSON");
    assert!(
        v["wet"].get("smooth_edges").is_none(),
        "a macro nova não o grava"
    );
    assert_eq!(v["preset"]["engine"][CHAVE].as_f64(), Some(valor));
    v["preset"]["engine"]
        .as_object_mut()
        .expect("o mapa")
        .remove(CHAVE);
    v["wet"]["smooth_edges"] = serde_json::json!(valor);
    (nova, v.to_string(), tx)
}

fn toca(estado: &str, tx: &std::collections::BTreeMap<String, Vec<u8>>) -> PainterTool {
    let mut t = tool();
    t.macro_estado_aplica(estado, tx.clone()).expect("aplica");
    t
}

/// ⭐ **A macro antiga toca igual a uma com o mesmo valor**, ao byte e com o valor na célula, o traço depois do estado.
/// CONTROLO: a 0 a tela é outra (o valor age).
#[test]
fn a_macro_antiga_toca_igual_a_uma_com_o_mesmo_valor() {
    let (nova, antiga, tx) = estados(0.36);
    let pinta = |estado: &str| {
        let mut t = toca(estado, &tx);
        traco(&mut t, 100.0);
        t.wet_recomposite_full();
        t
    };
    let (a, b) = (pinta(&nova), pinta(&antiga));
    let difs = |x: &PainterTool, y: &PainterTool| {
        x.canvas_rgba
            .iter()
            .zip(y.canvas_rgba.iter())
            .filter(|(p, q)| p != q)
            .count()
    };
    assert_eq!(difs(&a, &b), 0, "a macro antiga pintou outra coisa");
    assert!(pousados(&b).iter().all(|&v| v == 0.36), "o valor na célula");
    let (zero, ..) = estados(0.0);
    assert!(difs(&a, &pinta(&zero)) > 100, "o controlo: o valor não age");
}

/// **Ordem em relação ao traço**: a macro antiga que pega num pincel a meio mantém o valor, que na altura era da tela; a
/// nova fica com o do pincel. O que ela muda pelo painel a meio vale para os traços seguintes.
#[test]
fn a_macro_antiga_mantem_o_valor_ao_pegar_num_pincel_a_meio() {
    let (nova, antiga, tx) = estados(0.36);
    let sem_chave = de_fabrica_ou_candidato("Thick Impasto");
    assert!(
        !sem_chave.parametros.engine.contains_key(CHAVE),
        "a fixture"
    );
    let mut a = toca(&antiga, &tx);
    assert!(a.wet_preset_aplicar(&sem_chave).is_empty());
    assert_eq!(a.wet_smooth_edges(), 0.36, "a macro antiga perdeu o valor");
    assert!(
        a.route_brush_wetpaint_event(&ph2d_editor_core::tool::PanelEvent::SetValue(
            crate::ids::PAINTER_WETPAINT_SMOOTH_EDGES,
            0.2
        ))
    );
    assert!(a.wet_preset_aplicar(&sem_chave).is_empty());
    assert_eq!(a.wet_smooth_edges(), 0.2, "a edição a meio da macro antiga");
    let mut n = toca(&nova, &tx);
    assert!(n.wet_preset_aplicar(&sem_chave).is_empty());
    assert_eq!(n.wet_smooth_edges(), 0.0, "a macro nova é por pincel");
}
