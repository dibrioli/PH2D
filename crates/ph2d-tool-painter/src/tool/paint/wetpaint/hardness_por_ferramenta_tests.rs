//! **Cada ferramenta com a sua Hardness** (dono 05/11; Bug #65 A5): a Borracha tem a dela, os modos partilham uma e as
//! ferramentas da água têm a do corpo.
use super::same_as_paper_volume_tests::{H, W, traco_com};
use crate::tool::PainterTool;
use crate::tool::paint::WetTool;
use crate::tool::paint::wetpaint_preset::de_fabrica_ou_candidato;
use ph2d_editor_core::Tool;
use ph2d_editor_core::tool::{PanelEvent, RasterEditTool};

fn tela() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; W * H * 4], W as u32, H as u32);
    t.set_paint_tool_mode("wetpaint");
    t
}

/// Pelo evento que o controlo da tela dispara.
fn mexe(t: &mut PainterTool, v: f64) {
    t.handle_panel_event(PanelEvent::SetValue(
        crate::ids::PAINTER_WETPAINT_HARDNESS,
        v,
    ));
}

#[track_caller]
fn vale(t: &PainterTool, v: f64, o_que: &str) {
    let h = t.paint.wetpaint.knobs.hardness();
    assert!((h - v).abs() < 1e-6, "{o_que}: Hardness {h}, esperava {v}");
}

#[test]
fn o_pincel_e_a_borracha_tem_cada_um_a_sua_hardness() {
    let mut t = tela();
    mexe(&mut t, 0.2);
    t.set_paint_tool_mode("eraser");
    mexe(&mut t, 0.9);
    vale(&t, 0.9, "a Borracha com a dela");
    t.set_paint_tool_mode("wetpaint");
    vale(&t, 0.2, "o Pincel voltou com a dele");
    t.set_paint_tool_mode("eraser");
    vale(&t, 0.9, "a Borracha voltou com a dela");
}

/// Mexer na da Borracha não muda um byte do traço do Pincel.
#[test]
fn a_hardness_da_borracha_nao_muda_o_traco_do_pincel() {
    let e = de_fabrica_ou_candidato("Wet Round");
    let (_, base) = traco_com(&e, |_| {});
    let (_, t) = traco_com(&e, |t| {
        t.set_paint_tool_mode("eraser");
        mexe(t, 0.05);
        t.set_paint_tool_mode("wetpaint");
    });
    let difs = base
        .canvas_rgba
        .iter()
        .zip(t.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(difs, 0, "o Pincel pintou outro traço");
}

/// Os modos partilham uma só: voltar da Borracha por outro modo não traz o valor velho desse modo.
#[test]
fn voltar_da_borracha_por_outro_modo_nao_traz_a_hardness_velha() {
    let mut t = tela();
    mexe(&mut t, 0.7);
    t.set_paint_tool_mode("mask");
    t.set_paint_tool_mode("wetpaint");
    mexe(&mut t, 0.2);
    t.set_paint_tool_mode("eraser");
    mexe(&mut t, 0.9);
    t.set_paint_tool_mode("mask");
    t.set_paint_tool_mode("wetpaint");
    vale(&t, 0.2, "o Pincel depois de Borracha → Mask");
}

#[test]
fn as_ferramentas_da_agua_ficam_com_a_sua() {
    let mut t = tela();
    mexe(&mut t, 0.2);
    t.set_paint_tool_mode(WetTool::Blend.fio());
    mexe(&mut t, 0.5);
    t.set_paint_tool_mode("eraser");
    mexe(&mut t, 0.9);
    t.set_paint_tool_mode(WetTool::Blend.fio());
    vale(&t, 0.5, "o Blend");
    t.set_paint_tool_mode(WetTool::Paint.fio());
    vale(&t, 0.2, "o Pincel");
    t.set_paint_tool_mode("eraser");
    vale(&t, 0.9, "a Borracha");
}

/// «Sync with other tools»: ligado, é uma só; ao desligar, cada uma parte do valor à vista.
#[test]
fn ligadas_as_ferramentas_a_hardness_e_uma_so() {
    let mut t = tela();
    mexe(&mut t, 0.2);
    t.toggle_link_shared_settings();
    t.set_paint_tool_mode("eraser");
    vale(&t, 0.2, "ligada, a Borracha vê a do Pincel");
    mexe(&mut t, 0.9);
    t.set_paint_tool_mode("wetpaint");
    vale(&t, 0.9, "ligado, o Pincel vê a da Borracha");
    t.toggle_link_shared_settings();
    t.set_paint_tool_mode("eraser");
    vale(&t, 0.9, "desligada, a Borracha parte do valor à vista");
}

#[test]
fn a_macro_leva_as_duas_hardness() {
    for na_borracha in [false, true] {
        let mut vivo = tela();
        mexe(&mut vivo, 0.2);
        vivo.set_paint_tool_mode("eraser");
        mexe(&mut vivo, 0.9);
        if !na_borracha {
            vivo.set_paint_tool_mode("wetpaint");
        }
        let (estado, texturas) = vivo.macro_estado();
        let mut tocado = tela();
        tocado
            .macro_estado_aplica(&estado, texturas)
            .expect("aplica");
        let (na_mao, outra, modo) = if na_borracha {
            (0.9, 0.2, "wetpaint")
        } else {
            (0.2, 0.9, "eraser")
        };
        vale(&tocado, na_mao, "a da ferramenta na mão");
        tocado.set_paint_tool_mode(modo);
        vale(&tocado, outra, "a da outra ferramenta");
    }
}

/// Uma macro de antes tocava as duas com a Hardness única que gravou.
#[test]
fn a_macro_antiga_toca_as_duas_com_a_que_gravou() {
    for na_borracha in [false, true] {
        let mut vivo = tela();
        if na_borracha {
            vivo.set_paint_tool_mode("eraser");
        }
        mexe(&mut vivo, 0.3);
        let (estado, texturas) = vivo.macro_estado();
        let mut v: serde_json::Value = serde_json::from_str(&estado).expect("o JSON");
        let wet = v["wet"].as_object_mut().expect("wet");
        assert!(wet.remove("hardness_dos_modos").is_some(), "controlo");
        assert!(wet.remove("hardness_da_borracha").is_some(), "controlo");
        let mut tocado = tela();
        tocado.set_paint_tool_mode("eraser");
        mexe(&mut tocado, 0.8);
        tocado.set_paint_tool_mode("wetpaint");
        tocado
            .macro_estado_aplica(&v.to_string(), texturas)
            .expect("aplica");
        vale(&tocado, 0.3, "a ferramenta da gravação");
        tocado.set_paint_tool_mode(if na_borracha { "wetpaint" } else { "eraser" });
        vale(&tocado, 0.3, "a outra");
    }
}
