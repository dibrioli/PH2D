//! **Os gates do cartão Ray Sensor que a auditoria de 15/11 pediu** (`componentes/07_auditoria_2026-11-15.md`):
//! o «Ignore up to» com o conta-gotas (C8, dono 15/11), o losango de toda escolha do catálogo (U1) e a dica
//! de toda linha (U2) — clique REAL sobre o rect pintado, nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, edicoes, nos_dois_modos, raio};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::ray_edits::{RayFieldEdit, RaySee};
use ph2d_editor_core::widget::key_mark;
use ph2d_panel_inspector::ids as i;
use ph2d_panel_inspector::{InspectorPanel, set_current_inspector_ray};

/// ⭐⭐ **«Ignore up to»**: só existe com «Ignore my other parts»; o conta-gotas mora DENTRO da ponta da
/// caixa, o clique arma e (armado) desarma; escrever «Truck» manda o NOME.
///
/// **Mutações que sangram:** pintar a linha sem o `if i.ignore_parts` · o clique mandar sempre `PickUpTo(true)`
/// · o `TextChanged` do campo não chegar ao `IgnoreUpTo`.
#[test]
fn o_ignore_up_to_escreve_pelo_nome_e_pelo_conta_gotas() {
    nos_dois_modos(|c| {
        let mut sem = raio();
        sem.ignore_parts = false;
        let (_, _, r) = assente_com(&sem, c, true);
        assert!(achado(&r, i::INSP_RAY_UP_TO).is_none(), "sem a caixa, a linha não existe ({c})");
        set_current_inspector_ray(None);

        for armado in [false, true] {
            let mut info = raio();
            info.up_to_pick_armed = armado;
            let (mut h, mut st, r) = assente_com(&info, c, true);
            let caixa = achado(&r, i::INSP_RAY_UP_TO).expect("a caixa do Ignore up to");
            let gotas = achado(&r, i::INSP_RAY_UP_TO_PICK).expect("o conta-gotas");
            assert!(
                gotas.x >= caixa.x && gotas.x + gotas.w <= caixa.x + caixa.w + 0.5,
                "o conta-gotas dentro da caixa ({c}): {gotas:?} em {caixa:?}"
            );
            assert_eq!(
                edicoes(clica(&mut h, &mut st, gotas)),
                vec![RayFieldEdit::PickUpTo(!armado)],
                "cartões {c}, armado {armado}"
            );
            set_current_inspector_ray(None);
        }

        let (mut h, mut st, r) = assente_com(&raio(), c, true);
        let caixa = achado(&r, i::INSP_RAY_UP_TO).expect("a caixa");
        let _ = clica(&mut h, &mut st, caixa);
        let _ = h.drained_actions();
        // A tecla deixa o texto no buffer e o despacho emite `TextChanged` (o braço LÊ o buffer).
        h.set_text(i::INSP_RAY_UP_TO, "Truck");
        let ev = ph2d_editor_core::interaction::WidgetEvent::TextChanged(i::INSP_RAY_UP_TO);
        let _ = h.apply_panel_event::<InspectorPanel>(&mut st, ev);
        let saiu = edicoes(h.drained_actions());
        assert_eq!(
            saiu.last(),
            Some(&RayFieldEdit::IgnoreUpTo("Truck".into())),
            "cartões {c}: {saiu:?}"
        );
        set_current_inspector_ray(None);
    });
}

/// Todo campo do catálogo que o cartão do raio pinta, com a chave dele.
fn campos() -> Vec<(ph2d_a11y::NodeId, &'static str)> {
    let mut v = vec![
        (i::INSP_RAY_ANGLE, "component.field.ray_sensor.0"),
        (i::INSP_RAY_REACH, "component.field.ray_sensor.1"),
        (i::INSP_RAY_LAYER, "component.field.ray_sensor.2"),
        (i::INSP_RAY_TURNS, "component.field.ray_sensor.3"),
        (i::INSP_RAY_IGNORE_BODY, "component.field.ray_sensor.4"),
        (i::INSP_RAY_IGNORE_PARTS, "component.field.ray_sensor.5"),
        (i::INSP_RAY_SEE_TAG, "component.field.ray_sensor.7"),
        (i::INSP_RAY_WALLS_BLOCK, "component.field.ray_sensor.8"),
        (i::INSP_RAY_HIT_TRIGGERS, "component.field.ray_sensor.9"),
        (i::INSP_RAY_STARTS_INSIDE, "component.field.ray_sensor.10"),
        (i::INSP_RAY_ALL_HITS, "component.field.ray_sensor.11"),
        (i::INSP_RAY_UP_TO, "component.field.ray_sensor.12"),
        (i::INSP_RAY_ON_ENTER, "component.field.ray_signals.0"),
        (i::INSP_RAY_ON_EXIT, "component.field.ray_signals.1"),
    ];
    v.extend(i::INSP_RAY_ONLY_SEE.map(|id| (id, "component.field.ray_sensor.6")));
    v
}

/// ⭐⭐ **U1 — todo campo do catálogo pintado aparece em `rotulos_pintados()`**, nos dois modos: é daí
/// que nasce o losango (e a chave à mão). «Only see» não aparecia em nenhum.
///
/// **Mutação que sangra:** tirar o `regista_rotulo` do `paint_choice_row`.
#[test]
fn cada_campo_do_catalogo_pintado_tem_o_rotulo() {
    nos_dois_modos(|c| {
        let mut info = raio();
        info.only_see = RaySee::OnlyTag;
        key_mark::set_key_marks(std::collections::BTreeMap::new());
        let _ = assente_com(&info, c, true);
        let pintados = key_mark::rotulos_pintados();
        for (id, chave) in campos() {
            assert!(
                pintados.contains(&(id, chave)),
                "{chave} ({id:?}) sem rótulo pintado (cartões {c})"
            );
        }
        set_current_inspector_ray(None);
    });
}

/// ⭐ **U2 — toda linha do cartão do raio tem dica** (regra do dono para os cartões novos), cabeçalhos dos
/// grupos incluídos.
///
/// **Mutação que sangra:** tirar uma entrada da tabela de dicas do `populate_ray`.
#[test]
fn toda_linha_do_cartao_ray_tem_dica() {
    let mut info = raio();
    info.only_see = RaySee::OnlyTag;
    let (h, _, _) = assente_com(&info, true, true);
    let mut ids: Vec<_> = campos().into_iter().map(|(id, _)| id).collect();
    ids.extend([
        i::INSP_RAY_UP_TO_PICK,
        i::INSP_RAY_LAYER_RENAME,
        i::INSP_RAY_EDIT_MATRIX,
        i::INSP_RAY_FOLD_FILTER,
        i::INSP_RAY_FOLD_SIGNALS,
    ]);
    for id in ids {
        assert!(h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()), "{id:?} sem dica");
    }
    set_current_inspector_ray(None);
    ph2d_panel_inspector::set_cards_on(false);
}
