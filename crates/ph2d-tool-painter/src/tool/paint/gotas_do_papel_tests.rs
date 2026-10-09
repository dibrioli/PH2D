//! **As gotas do papel no painel** (W1c, auditoria de 08/10): o Reset do Paper, o Ctrl+Z, a lei de
//! inércia contra a do motor e a gravidade, que é da tela e de mais ninguém.

use super::super::media::PaintMedia;
use super::*;
use crate::ids::*;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_wet_paint::drips::DripPaper;
use ph2d_wet_paint::tuning::Knob;

fn molhado() -> PainterTool {
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; 16 * 16 * 4], 16, 16);
    t.set_paint_media(PaintMedia::WetPaint);
    t
}

/// As quatro rows das gotas, cada uma mexida pela porta do painel.
fn mexidas() -> [(&'static str, PanelEvent); 4] {
    [
        (
            "Create Drips",
            PanelEvent::Click(PAINTER_WETPAINT_DRIPS_CREATE),
        ),
        ("Easy Drips", PanelEvent::Click(PAINTER_WETPAINT_DRIPS_EASY)),
        (
            "Drip Size",
            PanelEvent::SetValue(PAINTER_WETPAINT_DRIP_SIZE, 9.0),
        ),
        (
            "Drip Length",
            PanelEvent::SetValue(PAINTER_WETPAINT_DRIP_LENGTH, 2.0),
        ),
    ]
}

/// ⭐ **O Reset do Paper repõe as gotas** — elas vivem na secção dele.
#[test]
fn o_reset_do_paper_repoe_as_gotas() {
    for (nome, mexe) in mexidas() {
        let mut t = molhado();
        t.handle_panel_event(mexe);
        assert_ne!(t.gotas(), DripPaper::DEFAULT, "{nome}: a row nem mexeu");
        t.handle_panel_event(PanelEvent::Click(PAINTER_WATERCOLOR_PAPER_RESET));
        assert_eq!(
            t.gotas(),
            DripPaper::DEFAULT,
            "{nome}: o Reset do Paper deixou as gotas"
        );
    }
    // Na fábrica o Reset não deixa um passo vazio no histórico.
    let mut t = molhado();
    assert!(!t.can_undo(), "fixture: o histórico já tinha passos");
    t.handle_panel_event(PanelEvent::Click(PAINTER_WATERCOLOR_PAPER_RESET));
    assert!(
        !t.can_undo(),
        "o Reset de fábrica deixou um passo que não muda nada"
    );
    // Só onde as rows se oferecem: no Digital o Reset do Paper não toca nas gotas do documento.
    let mut t = molhado();
    t.set_paint_media(PaintMedia::Digital);
    let gotas = DripPaper {
        create: false,
        ..DripPaper::DEFAULT
    };
    t.set_gotas(gotas);
    t.handle_panel_event(PanelEvent::Click(PAINTER_WATERCOLOR_PAPER_RESET));
    assert_eq!(
        t.gotas(),
        gotas,
        "o Reset do Paper do Digital mexeu nas gotas"
    );
}

/// ⭐ **Ctrl+Z desfaz as gotas** — cada row é um passo, e o arrasto de um campo é UM passo.
#[test]
fn o_desfazer_devolve_as_gotas() {
    for (nome, mexe) in mexidas() {
        let mut t = molhado();
        t.handle_panel_event(mexe);
        let mexida = t.gotas();
        assert_ne!(mexida, DripPaper::DEFAULT, "{nome}: a row nem mexeu");
        assert!(t.undo_last(), "{nome}: não houve passo a desfazer");
        assert_eq!(
            t.gotas(),
            DripPaper::DEFAULT,
            "{nome}: o Ctrl+Z não as devolveu"
        );
        assert!(t.redo_last(), "{nome}: não houve passo a refazer");
        assert_eq!(t.gotas(), mexida, "{nome}: o refazer não as repôs");
    }
    let mut t = molhado();
    for v in [6.0, 7.0, 8.0, 8.0] {
        t.handle_panel_event(PanelEvent::SetValue(PAINTER_WETPAINT_DRIP_SIZE, v));
    }
    assert!(t.undo_last());
    assert_eq!(
        t.gotas(),
        DripPaper::DEFAULT,
        "o arrasto do Drip Size é um passo só"
    );
    assert!(!t.undo_last(), "o valor repetido do arrasto não é passo");
}

/// ⭐ **A linha esmaecida das gotas pergunta ao MOTOR** — em cada tela (desligada, cada anel, três
/// direções), com e sem Create Drips: o Create Drips é inerte ⇔ a gravidade do motor é nula, e a
/// forma da gota é inerte ⇔ o motor não deixa gota nenhuma deslizar (`drips::can_slide` sobre
/// `Sim::gravity`). O anel 0 com a tela «ligada» tem gravidade 0 — e esmaece.
#[test]
fn a_lei_de_inercia_das_gotas_e_a_do_motor() {
    let mut motor = ph2d_wet_paint::painter::Engine::new(8, 8);
    let mut falhas = Vec::new();
    for create in [true, false] {
        for spoke in [0u8, 3, 7] {
            for ring in 0u8..=8 {
                for ligada in [true, false] {
                    let mut t = molhado();
                    if !create {
                        t.handle_panel_event(PanelEvent::Click(PAINTER_WETPAINT_DRIPS_CREATE));
                    }
                    t.handle_panel_event(PanelEvent::SetValue(
                        PAINTER_WETPAINT_TILT_RING,
                        f64::from(ring),
                    ));
                    t.handle_panel_event(PanelEvent::SetValue(
                        PAINTER_WETPAINT_TILT_SPOKE,
                        f64::from(spoke),
                    ));
                    if !ligada {
                        t.handle_panel_event(PanelEvent::Click(PAINTER_WETPAINT_TILT_TOGGLE));
                    }
                    let d = ph2d_wet_paint::sim::tilt_dir_for_spoke(spoke);
                    motor.set_knob(Knob::Gravity, t.wet_knobs().get(Knob::Gravity));
                    motor.sim.tilt_on = ligada;
                    motor.sim.tilt_dir_x = d[0];
                    motor.sim.tilt_dir_y = d[1];
                    motor.sim.tilt_scale = f64::from(ring) / 4.0;
                    motor.sim.gotas = Some(t.gotas());
                    let g = motor.sim.gravity(&motor.tuning);
                    let i = t.inercias();
                    let caso = format!("create={create} anel={ring} raio={spoke} ligada={ligada}");
                    if i.inerte(Dependente::GotasDaTela) != (g == [0.0, 0.0]) {
                        falhas.push(format!("{caso}: Create Drips mente (gravidade {g:?})"));
                    }
                    let desliza = create && ph2d_wet_paint::drips::can_slide(g);
                    if i.inerte(Dependente::FormaDaGota) == desliza {
                        falhas.push(format!("{caso}: a forma da gota mente (desliza={desliza})"));
                    }
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "{} estado(s) em que a linha das gotas mente:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// ⭐ **Nenhum id da Gravity é encaminhado** — ela é da obra (`KNOBS_DA_OBRA`): o dial da tela é a
/// porta única. Nem a fileira do ajuste (slider, chip, reset), nem o reset do grupo Physics.
#[test]
fn nenhuma_porta_do_ajuste_escreve_a_gravidade() {
    let gravidade = |t: &PainterTool| t.wet_knobs().get(Knob::Gravity);
    let mut t = molhado();
    let fabrica = gravidade(&t);
    t.handle_panel_event(PanelEvent::SetValue(wet_tuning_slider_id("gravity"), 0.04));
    t.handle_panel_event(PanelEvent::SetValue(wet_tuning_chip_id("gravity"), 0.03));
    assert_eq!(gravidade(&t), fabrica, "a fileira da Gravity escreveu-a");

    // A obra com outra gravidade (nenhuma porta do produto a escreve; o teste escreve o campo).
    t.paint.wetpaint.knobs.set(Knob::Gravity, 0.02);
    t.paint.wetpaint.knobs.set(Knob::Brake, 3.0);
    t.handle_panel_event(PanelEvent::Click(wet_tuning_reset_id("gravity")));
    assert_eq!(
        gravidade(&t),
        0.02,
        "o reset da fileira da Gravity escreveu-a"
    );
    t.handle_panel_event(PanelEvent::Click(WET_TUNING_GROUP_RESETS[2]));
    assert_eq!(
        t.wet_knobs().get(Knob::Brake),
        WetKnobs::DEFAULT.get(Knob::Brake),
        "o reset do grupo Physics nem chegou (controlo)"
    );
    assert_eq!(
        gravidade(&t),
        0.02,
        "o reset do grupo Physics escreveu a Gravity"
    );
}
