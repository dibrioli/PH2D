//! **As melhorias do cartão Ray Sensor** (GEF 07 RS, dono 15/11): a caixa «Enabled», «Thickness» e «Cone»,
//! o «Aim at» com o conta-gotas e a lista «Also ignore» — todo clique pelo ponteiro REAL sobre o rect pintado,
//! nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, edicoes, nos_dois_modos, pinta, raio};
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::ray_edits::{RayFieldEdit, RayQueixa, RayShapeKind};
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{InspectorPanel, set_cards_on, set_current_inspector_ray};

/// ⭐ «Enabled» é a primeira linha; o clique escreve o CONTRÁRIO do ecrã, e desligado a queixa diz-o.
///
/// **Mutação que deve sangrar:** `Enabled(info.enabled)` sem o `!` no `event_ray`.
#[test]
fn a_caixa_enabled_escreve_o_contrario_do_ecra() {
    nos_dois_modos(|c| {
        for ligada in [true, false] {
            let mut info = raio();
            info.enabled = ligada;
            let (mut h, mut st, r) = assente_com(&info, c, false);
            let caixa = achado(&r, i::INSP_RAY_ENABLED).expect("a caixa Enabled pintada");
            let angulo = achado(&r, i::INSP_RAY_ANGLE).expect("o Angle");
            // A primeira na ordem de leitura: acima, ou à esquerda na mesma linha (nos cartões a caixa
            // divide a linha com o campo seguinte, a compactação da casa).
            assert!(
                (caixa.y, caixa.x) < (angulo.y, angulo.x),
                "Enabled vem antes do Angle (cartões {c}): {caixa:?} {angulo:?}"
            );
            let saiu = edicoes(clica(&mut h, &mut st, caixa));
            assert_eq!(saiu, vec![RayFieldEdit::Enabled(!ligada)], "cartões {c}");
            assert_eq!(
                info.queixa() == Some(RayQueixa::Desligado),
                !ligada,
                "a queixa do desligado"
            );
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ Digitar na «Thickness» (metros) e no «Cone» (graus) chega como `Thickness`/`Cone`; os dois ganham o
/// foco sob o dedo.
///
/// **Mutação que deve sangrar:** trocar os dois braços do `ValueChanged` no `event_ray`.
#[test]
fn thickness_e_cone_chegam_pelo_teclado() {
    nos_dois_modos(|c| {
        for (id, texto, quer) in [
            (i::INSP_RAY_THICKNESS, "0.5", RayFieldEdit::Thickness(0.5)),
            (i::INSP_RAY_CONE, "45", RayFieldEdit::Cone(45.0)),
        ] {
            // Cada número só existe com a forma dele (dono 15/11).
            let mut info = raio();
            info.shape = if id == i::INSP_RAY_CONE {
                RayShapeKind::Cone
            } else {
                RayShapeKind::Thick
            };
            let (mut h, mut st, r) = assente_com(&info, c, false);
            let rect = achado(&r, id).unwrap_or_else(|| panic!("{id:?} não pintado (cartões {c})"));
            let _ = clica(&mut h, &mut st, rect);
            assert_eq!(h.store().focus_id(), Some(id), "{id:?} morto sob o dedo");
            let _ = h.drained_actions();
            for ev in h.type_into_number(id, texto) {
                let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
            }
            let saiu = edicoes(h.drained_actions());
            let bate = saiu.iter().any(|e| match (e, &quer) {
                (RayFieldEdit::Thickness(a), RayFieldEdit::Thickness(b))
                | (RayFieldEdit::Cone(a), RayFieldEdit::Cone(b)) => (a - b).abs() < 1e-4,
                _ => false,
            });
            assert!(bate, "{quer:?} não chegou: {saiu:?} (cartões {c})");
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐⭐⭐ **A «Shape» é um dropdown** (dono 15/11: *«thickness e cone são conflitantes»*), nos dois modos, por cliques
/// REAIS: fechada, nenhuma opção pintada; o clique no chip abre a lista (pintada no passe diferido) e o clique em
/// «Cone» manda `Shape(Cone)`. Por baixo, SÓ o número da forma activa: nada com Line, Thickness com Thick, Cone com
/// Cone. A lista e cada opção têm dica.
///
/// **Mutações que devem sangrar:** a opção mapeada para outra variante no `event_ray` · o cartão pintar a
/// Thickness com Line · o popover diferido não ser pintado (a opção não aparece).
#[test]
fn a_forma_e_um_dropdown_e_so_o_numero_dela_aparece() {
    nos_dois_modos(|c| {
        for (forma, espessura, cone) in [
            (RayShapeKind::Line, false, false),
            (RayShapeKind::Thick, true, false),
            (RayShapeKind::Cone, false, true),
        ] {
            let mut info = raio();
            info.shape = forma;
            info.thickness = 0.5;
            info.cone = 30.0;
            let (_, _, r) = assente_com(&info, c, false);
            assert!(
                achado(&r, i::INSP_RAY_SHAPE).is_some(),
                "a Shape pintada ({c})"
            );
            assert_eq!(
                achado(&r, i::INSP_RAY_THICKNESS).is_some(),
                espessura,
                "{forma:?}: Thickness (cartões {c})"
            );
            assert_eq!(
                achado(&r, i::INSP_RAY_CONE).is_some(),
                cone,
                "{forma:?}: Cone (cartões {c})"
            );
            set_current_inspector_ray(None);
        }
        for (n, quer) in RayShapeKind::ALL.into_iter().enumerate() {
            let (mut h, mut st, r) = assente_com(&raio(), c, false);
            assert!(
                i::INSP_RAY_SHAPE_OPTS
                    .iter()
                    .all(|&o| achado(&r, o).is_none()),
                "fechada, nenhuma opção ({c})"
            );
            let chip = achado(&r, i::INSP_RAY_SHAPE).expect("o chip Shape");
            let _ = clica(&mut h, &mut st, chip);
            let r = pinta(&mut h, &mut st, c);
            let opcao = achado(&r, i::INSP_RAY_SHAPE_OPTS[n])
                .unwrap_or_else(|| panic!("a opção {quer:?} não foi pintada (cartões {c})"));
            assert_eq!(
                edicoes(clica(&mut h, &mut st, opcao)),
                vec![RayFieldEdit::Shape(quer)],
                "cartões {c}"
            );
            for id in std::iter::once(i::INSP_RAY_SHAPE).chain(i::INSP_RAY_SHAPE_OPTS) {
                assert!(
                    h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()),
                    "{id:?} sem dica"
                );
            }
            set_current_inspector_ray(None);
        }
    });
}

/// ⭐ «Aim at»: o nome escrito chega como `AimAt`, o conta-gotas arma (e o premido desarma); com um alvo o
/// Angle e o «Turns with facing» saem (não decidem nada).
///
/// **Mutações que devem sangrar:** o `PickAimAt` sem o `!` · pintar o Angle com alvo.
#[test]
fn o_aim_at_escreve_pelo_nome_e_pelo_conta_gotas() {
    nos_dois_modos(|c| {
        let (mut h, mut st, r) = assente_com(&raio(), c, false);
        let caixa = achado(&r, i::INSP_RAY_AIM).expect("a caixa Aim at");
        let _ = clica(&mut h, &mut st, caixa);
        let _ = h.drained_actions();
        h.set_text(i::INSP_RAY_AIM, "Hero");
        let _ = h.apply_panel_event::<InspectorPanel>(
            &mut st,
            WidgetEvent::TextChanged(i::INSP_RAY_AIM),
        );
        assert_eq!(
            edicoes(h.drained_actions()),
            vec![RayFieldEdit::AimAt("Hero".into())]
        );
        for armado in [false, true] {
            let mut info = raio();
            info.aim_pick_armed = armado;
            let (mut h, mut st, r) = assente_com(&info, c, false);
            let gotas = achado(&r, i::INSP_RAY_AIM_PICK).expect("o conta-gotas do Aim at");
            assert_eq!(
                edicoes(clica(&mut h, &mut st, gotas)),
                vec![RayFieldEdit::PickAimAt(!armado)]
            );
            set_current_inspector_ray(None);
        }
        let mut mirado = raio();
        mirado.aim_at = "Hero".into();
        let (_, _, r) = assente_com(&mirado, c, false);
        assert!(
            achado(&r, i::INSP_RAY_ANGLE).is_none(),
            "com alvo o Angle sai (cartões {c})"
        );
        assert!(
            achado(&r, i::INSP_RAY_TURNS).is_none(),
            "e o Turns with facing"
        );
        assert!(achado(&r, i::INSP_RAY_AIM).is_some());
        set_current_inspector_ray(None);
    });
}

/// ⭐ «Also ignore» (no Filter): o conta-gotas arma; um chip por objecto, e o «×» do segundo tira o segundo.
///
/// **Mutações que devem sangrar:** o «×» resolver a posição pela tabela de outra lista · o `PickIgnore` sem o `!`.
#[test]
fn a_lista_also_ignore_junta_pelo_conta_gotas_e_tira_pelo_x() {
    nos_dois_modos(|c| {
        let mut info = raio();
        info.also_ignore = vec!["Crate".into(), "Box".into()];
        let (mut h, mut st, r) = assente_com(&info, c, true);
        let gotas = achado(&r, i::INSP_RAY_IGNORE_PICK).expect("o conta-gotas do Also ignore");
        assert_eq!(
            edicoes(clica(&mut h, &mut st, gotas)),
            vec![RayFieldEdit::PickIgnore(true)]
        );
        let (mut h, mut st, r) = assente_com(&info, c, true);
        let chips = i::INSP_RAY_IGNORE_CHIP.ids(2);
        let x = achado(&r, chips[1]).expect("o × do segundo chip");
        assert!(achado(&r, chips[0]).is_some(), "o primeiro chip");
        assert_eq!(
            edicoes(clica(&mut h, &mut st, x)),
            vec![RayFieldEdit::IgnoreRemove(1)]
        );
        // A lista vazia não pinta chip nenhum (e o conta-gotas fica).
        let (_, _, r) = assente_com(&raio(), c, true);
        assert!(achado(&r, chips[0]).is_none());
        assert!(achado(&r, i::INSP_RAY_IGNORE_PICK).is_some());
        set_current_inspector_ray(None);
    });
}

/// ⭐ Toda linha nova tem dica (a regra do dono para os cartões novos), o «×» de um chip incluído.
///
/// **Mutação que deve sangrar:** tirar uma das dicas novas do `populate_ray`.
#[test]
fn toda_linha_nova_tem_dica() {
    let mut info = raio();
    info.also_ignore = vec!["Crate".into()];
    let (mut h, mut st, _) = assente_com(&info, true, true);
    let _ = pinta(&mut h, &mut st, true);
    let mut ids = vec![
        i::INSP_RAY_ENABLED,
        i::INSP_RAY_THICKNESS,
        i::INSP_RAY_CONE,
        i::INSP_RAY_AIM,
        i::INSP_RAY_AIM_PICK,
        i::INSP_RAY_IGNORE_PICK,
    ];
    ids.extend(i::INSP_RAY_IGNORE_CHIP.ids(1));
    for id in ids {
        assert!(
            h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()),
            "{id:?} sem dica"
        );
    }
    set_current_inspector_ray(None);
    set_cards_on(false);
}
