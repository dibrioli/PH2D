use super::*;

/// ⭐ A mão de fábrica é a `Dynamics`/`PenTilt` de sempre ao bit (o neutro da L2).
#[test]
fn a_mao_de_fabrica_e_a_de_sempre() {
    let (d, t) = de_fabrica();
    assert_eq!(d, Dynamics::default());
    assert_eq!(t, PenTilt::default());
}

/// Os ids do cartão Pen são os das linhas, pela ordem e pela chave.
#[test]
fn cada_linha_tem_o_seu_id_pela_chave() {
    for (l, id) in LINHAS_DA_MAO
        .into_iter()
        .zip(crate::ids::PAINTER_PEN_FIELDS)
    {
        let pela_chave =
            ph2d_tool_registry::hash_node_id_runtime(&format!("painter_brush.pen.{}", l.chave()));
        assert_eq!(id, pela_chave, "{}", l.chave());
    }
    assert_eq!(MAO_DE_FABRICA, LINHAS_DA_MAO.map(LinhaDaMao::omissao));
}

/// ⭐ **A pressão do Digital afina como a do Wet** (L2): as mesmas linhas no cartão de cada meio dão a
/// MESMA `Dynamics` e a MESMA `PenTilt` ao traço — a lei é uma, os armazéns é que são dois. Na Watercolor
/// as linhas da opacidade não se escrevem (o meio não as lê) e o resto é igual.
#[test]
fn a_pressao_digital_afina_como_a_do_wet() {
    let valores = [
        (Knob::PressureSize, 0.5),
        (Knob::PressureSizeCurve, 0.7),
        (Knob::PressureOpacity, 0.4),
        (Knob::PressureOpacityCurve, 1.5),
        (Knob::PressureFloor, 0.1),
        (Knob::PressureCeiling, 0.9),
        (Knob::PressureSmoothing, 0.3),
        (Knob::PulledString, 12.0),
        (Knob::DabRotation, 2.0),
        (Knob::TiltStart, 20.0),
        (Knob::TiltFull, 50.0),
        (Knob::TiltWidth, 1.5),
        (Knob::TiltLength, 2.0),
        (Knob::TiltSmooth, 0.4),
        (Knob::TiltLag, 30.0),
    ];
    let mut wet = super::super::super::PainterTool::default();
    wet.set_paint_media(PaintMedia::WetPaint);
    for (k, v) in valores {
        wet.set_mao(LinhaDaMao::Knob(k), v);
    }
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let mut t = super::super::super::PainterTool::default();
        t.set_paint_media(m);
        for (k, v) in valores {
            t.set_mao(LinhaDaMao::Knob(k), v);
        }
        let (d, w) = (t.dinamica_do_traco(), wet.dinamica_do_traco());
        let (mut ti, wi) = (t.inclinacao_do_pincel(), wet.inclinacao_do_pincel());
        if m == PaintMedia::Watercolor {
            // A Shape «Automatic» (a de fábrica da Watercolor) não lê a pegada deitada.
            assert_eq!(
                (ti.width, ti.length, ti.rotation),
                (1.0, 1.0, DabRotation::None)
            );
            (ti.width, ti.length, ti.rotation) = (wi.width, wi.length, wi.rotation);
        }
        assert_eq!(ti, wi, "{m:?}");
        assert_eq!(
            (d.size_min, d.size_curve, d.pen),
            (w.size_min, w.size_curve, w.pen),
            "{m:?}"
        );
        if m == PaintMedia::Watercolor {
            assert!(
                !d.strength_pressure,
                "a Watercolor não lê a opacidade do dab"
            );
        } else {
            assert_eq!(d, w, "{m:?}");
        }
    }
}

/// Uma MACRO gravada antes da L2 (sem a chave `mao`, e a `pen_tilt` sem `opacity`) toca com a mão de FÁBRICA,
/// nunca com a de agora.
#[test]
fn uma_macro_sem_a_mao_volta_a_mao_de_fabrica() {
    let mut d = Dynamics::default();
    let mut t = PenTilt::default();
    for l in LINHAS_DA_MAO {
        let (lo, hi, _) = l.faixa();
        escreve(&mut d, &mut t, l, lo + (hi - lo) * 0.3);
    }
    dinamica_de_json(&mut d, &serde_json::Value::Null);
    assert_eq!(d, Dynamics::default());
    let mut b = ph2d_painter_brush::BrushSpec::default();
    b.pen_tilt.opacity = 0.7;
    let mut m = super::super::wetpaint_preset::json::pincel_para_json(&b, &["pen_tilt"]);
    m.get_mut("pen_tilt")
        .and_then(serde_json::Value::as_object_mut)
        .unwrap()
        .remove("opacity");
    let lida = super::super::wetpaint_preset::json::pincel_de_json(&m, &["pen_tilt"], b);
    assert_eq!(
        lida.pen_tilt.opacity, 0.0,
        "a pen_tilt antiga não traz a opacidade de agora"
    );
}

/// ⭐ **As linhas que nasceram depois da fábrica só vão para o preset MEXIDAS** (a curva em S, a pressão leve —
/// J3 canetas 2.ª onda): o pincel de fábrica não as grava (todo preset de antes fica ao byte), e mexida, grava-a.
#[test]
fn as_linhas_novas_so_vao_para_o_preset_mexidas() {
    use super::super::wetpaint_preset::nome_do_meio;
    use ph2d_brush_presets::Parametros;
    let novas = [
        LinhaDaMao::PressureSCurve,
        LinhaDaMao::PressureJitter,
        LinhaDaMao::SpeedLag,
        LinhaDaMao::SpeedHardness,
        LinhaDaMao::PressureHardness,
    ];
    let mut t = super::super::super::PainterTool::default();
    t.set_paint_media(PaintMedia::Digital);
    let base = Parametros::novo(nome_do_meio(PaintMedia::Digital), "g", "n");
    let (p, _) = t.preset_capturar(&base);
    for l in novas {
        assert!(
            !p.engine.contains_key(l.chave()),
            "{} no preset de fábrica",
            l.chave()
        );
        t.set_mao(l, 0.25);
    }
    let (p, _) = t.preset_capturar(&base);
    for l in novas {
        assert_eq!(
            p.engine.get(l.chave()).copied(),
            Some(0.25),
            "{} mexida e não gravada",
            l.chave()
        );
    }
}

/// Ler e voltar a escrever cada linha não muda um bit dos campos (o painel e o preset leem por [`le`]).
#[test]
fn ler_e_escrever_a_mao_e_a_identidade() {
    let (mut d, mut t) = de_fabrica();
    for (i, l) in LINHAS_DA_MAO.into_iter().enumerate() {
        let (lo, hi, _) = l.faixa();
        escreve(&mut d, &mut t, l, lo + (hi - lo) * (i as f64 + 1.0) / 20.0);
    }
    let (d0, t0) = (d, t);
    for l in LINHAS_DA_MAO {
        let v = le(&d0, &t0, l);
        escreve(&mut d, &mut t, l, v);
    }
    assert_eq!((d, t), (d0, t0));
}

/// ⭐ **O Reset do cartão Pen tem DUAS portas** (dono 25/10): «Reset to default» é o Reset de sempre ao bit (o
/// `Click` que as macros gravaram); «Reset to brush preset» devolve a mão que o preset activo traz — e uma linha que
/// ele NÃO lista volta ao neutro, a mesma lei de aplicá-lo. Sem preset, ou noutra gaveta, a segunda não mexe.
#[test]
fn o_reset_da_mao_tem_duas_portas() {
    use super::super::wetpaint_preset::nome_do_meio;
    use crate::ids::{PAINTER_PEN_RESET, PAINTER_PEN_RESET_NEUTRO, PAINTER_PEN_RESET_PRESET};
    use ph2d_editor_core::tool::PanelEvent;
    type T = super::super::super::PainterTool;
    let opcao = |o: u8| PanelEvent::SelectOption(PAINTER_PEN_RESET, o.to_string());
    let mexe = |t: &mut T| {
        for l in LINHAS_DA_MAO {
            let (lo, hi, _) = l.faixa();
            t.set_mao(l, lo + (hi - lo) * 0.37);
        }
    };
    let estado = |t: &T| {
        (
            t.mao(),
            t.paint.dynamics,
            t.paint.brush.pen_tilt,
            t.paint.wetpaint.knobs,
        )
    };
    // O preset fixa DUAS linhas que os dois meios leem; as outras 21 não lista.
    let (ps, ts) = (
        LinhaDaMao::Knob(Knob::PressureSize),
        LinhaDaMao::Knob(Knob::TiltStart),
    );
    for m in [PaintMedia::Digital, PaintMedia::WetPaint] {
        // A porta do neutro = o Click de sempre, ao bit.
        let (mut a, mut b) = (T::default(), T::default());
        for t in [&mut a, &mut b] {
            t.set_paint_media(m);
            mexe(t);
        }
        a.route_mao_event(&PanelEvent::Click(PAINTER_PEN_RESET));
        assert!(b.route_mao_event(&opcao(PAINTER_PEN_RESET_NEUTRO)));
        assert_eq!(estado(&a), estado(&b), "{m:?}: o neutro");
        assert_eq!(b.mao(), MAO_DE_FABRICA, "{m:?}");

        // Sem preset activo a porta do preset existe e não mexe.
        let mut t = T::default();
        t.set_paint_media(m);
        mexe(&mut t);
        let antes = estado(&t);
        assert!(!t.mao_do_preset_disponivel(), "{m:?}");
        assert!(t.route_mao_event(&opcao(PAINTER_PEN_RESET_PRESET)));
        assert_eq!(estado(&t), antes, "{m:?}: sem preset nada muda");

        // Com o preset aplicado: a mão dele, e as linhas que ele não lista no neutro.
        let mut prm = ph2d_brush_presets::Parametros::novo(nome_do_meio(m), "g", "n");
        prm.engine.insert(ps.chave().to_owned(), 0.5);
        prm.engine.insert(ts.chave().to_owned(), 20.0);
        let preset = ph2d_brush_presets::Preset {
            parametros: prm,
            origem: ph2d_brush_presets::Origem::Utilizador,
            miniatura: ph2d_brush_presets::Miniatura {
                largura: 1,
                altura: 1,
                rgba: vec![0; 4],
            },
            texturas: Default::default(),
        };
        assert!(t.preset_aplicar(&preset).is_ok());
        assert!(t.mao_do_preset_disponivel(), "{m:?}: aplicar lembra a mão");
        mexe(&mut t);
        t.route_mao_event(&opcao(PAINTER_PEN_RESET_PRESET));
        let mao = t.mao();
        for (i, l) in LINHAS_DA_MAO.into_iter().enumerate() {
            if !l.lida_por(m) {
                continue;
            }
            let quer = if l == ps {
                0.5
            } else if l == ts {
                20.0
            } else {
                l.omissao()
            };
            assert_eq!(mao[i], quer, "{m:?}: {}", l.chave());
        }

        // Outra gaveta (outro meio): o preset não é o do pincel na mão.
        t.set_paint_media(PaintMedia::Impasto);
        assert!(!t.mao_do_preset_disponivel(), "{m:?}: noutra gaveta");
        // E a ponte que diz «nenhum activo» apaga-a.
        t.set_paint_media(m);
        assert!(t.mao_do_preset_disponivel(), "{m:?}: de volta à gaveta");
        t.set_mao_do_preset(None);
        assert!(!t.mao_do_preset_disponivel(), "{m:?}: a ponte apagou-a");
    }
}
