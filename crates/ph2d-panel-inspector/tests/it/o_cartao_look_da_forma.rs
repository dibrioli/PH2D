//! **O cartão Look da forma de colisão** (GEF02 onda 8): nasce só no objeto «Collision Shape»; a grade de nove pontos,
//! os campos X e Y (na unidade do projeto) e a Opacity chegam ao barramento como as edições `Look*` da física.

use ph2d_editor_core::ShapeLookFacts;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit as E};
use ph2d_panel_inspector::InspectorPanel;
use ph2d_panel_inspector::ids::*;

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::with_body;

fn forma(look: ShapeLookFacts) -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        has_body: false,
        has_collider: true,
        part_owner: "Torso".into(),
        shape_of: "Torso".into(),
        look,
        ..with_body()
    }
}

fn ligado() -> ShapeLookFacts {
    ShapeLookFacts {
        on: true,
        ..ShapeLookFacts::NONE
    }
}

fn edicoes(a: &[EditorAction]) -> Vec<E> {
    a.iter()
        .filter_map(|x| match x {
            EditorAction::InspectorPhysicsEdit { edit, .. } => Some(*edit),
            _ => None,
        })
        .collect()
}

/// ⭐ Só o objeto «Collision Shape» tem o cartão: o corpo e a forma de antes (`on = false`) não.
#[test]
fn o_look_nasce_so_na_forma_de_colisao() {
    for (info, tem) in [
        (forma(ligado()), true),
        (forma(ShapeLookFacts::NONE), false),
        (with_body(), false),
    ] {
        let (mut host, mut st) = cena(info, [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        assert_eq!(achado(&r, INSP_SHAPE_PIVOT[4]).is_some(), tem, "a grade");
        assert_eq!(
            achado(&r, INSP_SHAPE_OPACITY_CHIP).is_some(),
            tem,
            "a Opacity"
        );
        sai();
    }
}

/// ⭐ Cada um dos nove pontos é o seu `[coluna, linha]`, a linha de cima é a 0.
#[test]
fn cada_ponto_da_grade_poe_o_pivo_nele() {
    for (i, &id) in INSP_SHAPE_PIVOT.iter().enumerate() {
        let (mut host, mut st) = cena(forma(ligado()), [1.0, 1.0]);
        let r = pinta(&mut host, &mut st);
        let q = achado(&r, id).expect("o ponto");
        let a = clica(&mut host, &mut st, q);
        assert_eq!(
            edicoes(&a),
            vec![E::LookPivotPoint([(i % 3) as u8, (i / 3) as u8])],
            "ponto {i}"
        );
        sai();
    }
}

/// Em Custom os campos estão à vista e o número digitado (m) é o eixo dele; em px, `200` = `200 / ppm` m.
#[test]
fn os_campos_x_e_y_escrevem_o_pivo_na_unidade_do_projeto() {
    let custom = ShapeLookFacts {
        ponto: None,
        ..ligado()
    };
    let (mut host, mut st) = cena(forma(custom), [1.0, 1.0]);
    host.project_mut().display_unit = ph2d_editor_core::project::DisplayUnit::Pixels;
    let ppm = host.project_mut().pixels_per_meter;
    let r = pinta(&mut host, &mut st);
    assert!(
        achado(&r, INSP_SHAPE_PIVOT_XY[0]).is_some(),
        "Custom abre os campos"
    );
    let _ = host.drained_actions();
    for ev in host.type_into_number(INSP_SHAPE_PIVOT_XY[1], "200") {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let e = edicoes(&host.drained_actions());
    let Some(E::LookPivotY(v)) = e.first().copied() else {
        panic!("o Y não chegou: {e:?}");
    };
    assert!((v * ppm - 200.0).abs() < 1e-3, "{v} m × {ppm}");
    sai();
}

/// O chip da Opacity em % chega à física como `0..1`.
#[test]
fn a_opacidade_em_percentagem_chega_como_fracao() {
    let (mut host, mut st) = cena(forma(ligado()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let _ = host.drained_actions();
    for ev in host.type_into_number(INSP_SHAPE_OPACITY_CHIP, "40") {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let e = edicoes(&host.drained_actions());
    assert!(
        e.iter()
            .any(|x| matches!(x, E::LookOpacity(v) if (*v - 0.4).abs() < 1e-6)),
        "{e:?}"
    );
    sai();
}

/// ⭐ A Color: clicar na amostra abre o seletor nela; a cor escolhida no seletor chega à física no quadro seguinte.
#[test]
fn a_cor_escolhida_no_seletor_chega_como_edicao() {
    let (mut host, mut st) = cena(forma(ligado()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let q = achado(&r, INSP_SHAPE_TINT_SWATCH).expect("a amostra Color");
    let _ = clica(&mut host, &mut st, q);
    assert_eq!(
        host.store().picker_target(),
        Some(INSP_SHAPE_TINT_SWATCH),
        "o seletor abriu nela"
    );
    host.store_mut()
        .set_widget_color(INSP_SHAPE_TINT_SWATCH, [255, 0, 0, 255]);
    let _ = host.drained_actions();
    let _ = pinta(&mut host, &mut st);
    let e = edicoes(&host.drained_actions());
    assert!(e.contains(&E::LookTint([1.0, 0.0, 0.0, 1.0])), "{e:?}");
    sai();
}

/// ⭐ Dono 10/11: sem o «…», o «Pivot from centre» está sempre à vista (também num dos nove pontos), X por cima de Y,
/// ao lado da grade e da altura dela.
#[test]
fn os_campos_do_pivo_estao_sempre_a_vista_ao_lado_da_grade() {
    let (mut host, mut st) = cena(forma(ligado()), [1.0, 1.0]);
    let r = pinta(&mut host, &mut st);
    let (g, x, y) = (
        achado(&r, INSP_SHAPE_PIVOT[2]).expect("o canto de cima à direita da grade"),
        achado(&r, INSP_SHAPE_PIVOT_XY[0]).expect("o X"),
        achado(&r, INSP_SHAPE_PIVOT_XY[1]).expect("o Y"),
    );
    assert!(x.x > g.x + g.w, "o X à direita da grade: {x:?} {g:?}");
    assert!(
        (x.x - y.x).abs() < 0.5 && y.y > x.y + x.h * 0.5,
        "o Y por baixo do X: {x:?} {y:?}"
    );
    // Dono 10/11: a grade tem a altura dos dois campos juntos (o canto de cima e o de baixo).
    let fundo = achado(&r, INSP_SHAPE_PIVOT[6]).expect("o canto de baixo à esquerda");
    assert!((g.y - x.y).abs() < 0.5, "a grade começa no X: {g:?} {x:?}");
    let (fg, fy) = (fundo.y + fundo.h, y.y + y.h);
    assert!((fg - fy).abs() < 0.5, "a grade acaba no Y: {fg} contra {fy}");
    sai();
}

/// Sem botões de reset no Look da forma (dono 10/11: «retire esses botões»); o chip não desce de 25 %.
#[test]
fn sem_resets_e_o_piso_da_opacidade() {
    let (mut host, mut st) = cena(forma(ligado()), [1.0, 1.0]);
    let _ = pinta(&mut host, &mut st);
    let _ = host.drained_actions();
    for ev in host.type_into_number(INSP_SHAPE_OPACITY_CHIP, "5") {
        let _ = host.apply_panel_event::<InspectorPanel>(&mut st, ev);
    }
    let e = edicoes(&host.drained_actions());
    assert!(
        e.iter()
            .all(|x| matches!(x, E::LookOpacity(v) if *v >= ShapeLookFacts::OPACIDADE_MINIMA)),
        "{e:?}"
    );
    sai();
}
