//! ⭐ **O meio e o slot são da FERRAMENTA na mão** (dono 24/10): o Blur/Smear/Clone/Máscara só no Digital, a
//! Borracha com o seu pincel, a sua mão e três meios; cada ferramenta com a sua gaveta de presets.

use super::PaintMedia;
use crate::FerramentaDoPincel as F;
use crate::tool::PainterTool;
use crate::tool::paint::PaintMode;
use ph2d_brush_presets::Parametros;

const SO_DIGITAL: [&str; 4] = ["blur", "smear", "clone", "mask"];

/// Vermelho antes: com a água armada o Blur dizia Wet Paint, e o painel pintava o Wet inteiro (Blend, Color,
/// Composite e o Paint Mode, as capturas do dono).
#[test]
fn as_ferramentas_de_um_meio_so_sao_digitais_com_a_agua_armada() {
    for fio in SO_DIGITAL {
        let mut t = PainterTool::default();
        t.set_paint_media(PaintMedia::WetPaint);
        t.set_paint_tool_mode(fio);
        let b = t.brush_settings();
        assert_eq!(t.paint_media(), PaintMedia::Digital, "{fio}");
        assert_eq!(b.media, PaintMedia::Digital.to_u8(), "{fio}");
        assert!(!b.wetpaint, "{fio}: o painel pintaria a secção do Wet");
        assert_eq!(
            b.meios,
            1 << PaintMedia::Digital.to_u8(),
            "{fio}: o menu Paint Mode sai"
        );
        // O meio que a ferramenta não tem não se escolhe…
        t.set_paint_media(PaintMedia::Watercolor);
        assert_eq!(t.paint_media(), PaintMedia::Digital, "{fio}");
        assert_eq!(t.active_paint_mode_id(), fio, "{fio}: a ferramenta fica");
        // …e o arme do Pincel sobrevive à volta.
        t.set_paint_tool_mode("brush");
        assert_eq!(
            t.paint_media(),
            PaintMedia::WetPaint,
            "{fio}: a volta ao Pincel"
        );
    }
}

/// Vermelho antes: a Borracha partilhava o slot do Pincel (o tamanho dele) e a mão (a pressão → opacidade dele).
#[test]
fn a_borracha_tem_o_seu_pincel_e_a_sua_mao() {
    let mut t = PainterTool::default();
    t.set_brush_size_px(40.0);
    t.paint.dynamics.strength_pressure = true;
    t.set_paint_tool_mode("eraser");
    assert!(
        (t.brush_size_px() - 40.0).abs() > 1.0,
        "a Borracha nasce com o seu tamanho"
    );
    assert!(
        !t.paint.dynamics.strength_pressure,
        "a mão da Borracha: a pressão não leva a força"
    );
    assert!(
        t.paint.dynamics.size_pressure,
        "…leva o tamanho (Procreate/Krita)"
    );
    t.set_brush_size_px(25.0);
    t.set_paint_tool_mode("brush");
    assert!(
        (t.brush_size_px() - 40.0).abs() < 1e-3,
        "o Pincel volta com o seu"
    );
    assert!(t.paint.dynamics.strength_pressure, "…e com a sua mão");
    t.set_paint_tool_mode("eraser");
    assert!(
        (t.brush_size_px() - 25.0).abs() < 1e-3,
        "a Borracha lembra o seu"
    );
}

#[test]
fn a_borracha_tem_tres_meios_e_continua_borracha_em_cada_um() {
    let mut t = PainterTool::default();
    t.set_paint_tool_mode("eraser");
    let b = t.brush_settings();
    assert_eq!(
        b.meios,
        [
            PaintMedia::Digital,
            PaintMedia::Watercolor,
            PaintMedia::WetPaint
        ]
        .iter()
        .fold(0, |a, m| a | 1 << m.to_u8())
    );
    t.set_paint_media(PaintMedia::Impasto);
    assert_eq!(
        t.paint_media(),
        PaintMedia::Digital,
        "o Impasto fica de fora"
    );
    for m in [
        PaintMedia::WetPaint,
        PaintMedia::Watercolor,
        PaintMedia::Digital,
        PaintMedia::WetPaint,
    ] {
        t.set_paint_media(m);
        assert_eq!(t.paint_media(), m);
        assert!(t.paint.eraser, "{m:?}: a Borracha continua na mão");
        assert_eq!(t.brush_settings().ferramenta(), F::Borracha, "{m:?}");
        let modo = if m == PaintMedia::WetPaint {
            PaintMode::WetPaint
        } else {
            PaintMode::Paint
        };
        assert_eq!(t.paint.paint_mode, modo, "{m:?}");
    }
    // A Watercolor da Borracha é dela: o Pincel continua no meio dele.
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_paint_tool_mode("brush");
    assert_eq!(t.paint_media(), PaintMedia::Digital);
    t.set_paint_tool_mode("eraser");
    assert_eq!(t.paint_media(), PaintMedia::Watercolor);
}

/// Vermelho antes: a Borracha vinda do Blur com a água armada caía no Digital com o Wet no menu.
#[test]
fn a_borracha_vinda_do_blur_com_a_agua_armada_e_molhada() {
    let mut t = PainterTool::default();
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_paint_tool_mode("blur");
    t.set_paint_tool_mode("eraser");
    assert_eq!(t.paint.paint_mode, PaintMode::WetPaint);
    assert_eq!(t.paint_media(), PaintMedia::WetPaint);
}

/// ⭐ A gaveta: o preset guardado com a ferramenta na mão volta a ela, num pincel novo, no meio dela.
#[test]
fn o_preset_de_uma_ferramenta_volta_a_ela() {
    for (fio, f, meio) in [
        ("blur", F::Blur, PaintMedia::Digital),
        ("smear", F::Smear, PaintMedia::Digital),
        ("clone", F::Clone, PaintMedia::Digital),
        ("eraser", F::Borracha, PaintMedia::Digital),
        ("eraser", F::Borracha, PaintMedia::Watercolor),
        ("eraser", F::Borracha, PaintMedia::WetPaint),
    ] {
        let mut t = PainterTool::default();
        t.set_paint_tool_mode(fio);
        t.set_paint_media(meio);
        t.set_brush_size_px(33.0);
        let (p, _) = t.preset_capturar(&Parametros::novo("Digital", "User", "x"));
        assert_eq!(crate::ferramenta_do_preset(&p), f, "{fio} {meio:?}");
        let mut n = PainterTool::default();
        n.set_brush_size_px(12.0);
        let preset = ph2d_brush_presets::Preset {
            parametros: p,
            origem: ph2d_brush_presets::Origem::Utilizador,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas: Default::default(),
        };
        assert!(n.preset_aplicar(&preset).is_ok());
        assert_eq!(
            n.ferramenta_dos_presets(),
            Some(f),
            "{fio} {meio:?}: a ferramenta"
        );
        assert_eq!(n.paint_media(), meio, "{fio} {meio:?}: o meio");
        assert!(
            (n.brush_size_px() - 33.0).abs() < 0.5,
            "{fio} {meio:?}: o pincel"
        );
        // O slot do Pincel não foi escrito (no Wet o Pincel volta ao slot do fluido — a água é UMA sessão).
        let r = n.paint.brush_by_mode[PaintMode::Paint.slot()].radius_px;
        n.set_paint_tool_mode("brush");
        assert!(
            (n.paint.brush.radius_px - r).abs() < 1e-3 || meio == PaintMedia::WetPaint,
            "{fio} {meio:?}: o Pincel intacto"
        );
        assert!(
            (r - 12.0).abs() < 1e-3,
            "{fio} {meio:?}: o Pincel intacto ({r})"
        );
    }
}

/// As borrachas de fábrica de antes do campo eram pincéis Erase Alpha: são da Borracha. O Smear do FLUIDO é do Smear da
/// água (dono 25/10: cada ferramenta da água tem a sua gaveta).
#[test]
fn os_presets_de_antes_do_campo_acham_a_sua_ferramenta() {
    let mut p = Parametros::novo("Digital", "Hard", "Hard Eraser");
    p.paint_mode = Some("Paint".into());
    p.brush
        .insert("blend".into(), serde_json::json!("EraseAlpha"));
    assert_eq!(crate::ferramenta_do_preset(&p), F::Borracha);
    let mut w = Parametros::novo("WetPaint", "Blend", "x");
    w.paint_mode = Some("Smear".into());
    assert_eq!(crate::ferramenta_do_preset(&w), F::SmearMolhado);
    w.paint_mode = Some("Paint".into());
    assert_eq!(crate::ferramenta_do_preset(&w), F::Pincel);
    let mut d = Parametros::novo("Digital", "Blend", "Blur");
    d.paint_mode = Some("Blur".into());
    assert_eq!(crate::ferramenta_do_preset(&d), F::Blur);
}
