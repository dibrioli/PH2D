//! **Os gates do cartão Ray Sensor que a auditoria de 15/11 pediu** (`componentes/07_auditoria_2026-11-15.md`):
//! o «Ignore up to» com o conta-gotas (C8, dono 15/11), o losango de toda escolha do catálogo (U1) e a dica
//! de toda linha (U2) — clique REAL sobre o rect pintado, nos cartões e fora deles.

use crate::o_cartao_da_fisica::{achado, clica};
use crate::seam_ray::{assente_com, edicoes, nos_dois_modos, raio};
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
        assert!(
            achado(&r, i::INSP_RAY_UP_TO).is_none(),
            "sem a caixa, a linha não existe ({c})"
        );
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
        assert!(
            h.store().tooltip_for(id).is_some_and(|t| !t.is_empty()),
            "{id:?} sem dica"
        );
    }
    set_current_inspector_ray(None);
    ph2d_panel_inspector::set_cards_on(false);
}

thread_local! {
    static TEXTOS: std::cell::RefCell<Vec<(String, [f32; 4])>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn ouve(t: &str, r: [f32; 4], _: f32) {
    TEXTOS.with(|v| v.borrow_mut().push((t.to_owned(), r)));
}

/// Pinta `info` e devolve (a caixa do «Ignore up to», os textos pintados, quantos traços a cena emite).
fn pinta_o_topo(
    info: &ph2d_editor_core::ray_edits::InspectorRayInfo,
    c: bool,
) -> (
    ph2d_editor_core::zones::Rect,
    Vec<(String, [f32; 4])>,
    usize,
) {
    TEXTOS.with(|v| v.borrow_mut().clear());
    ph2d_editor_core::paint::ouve_o_texto(Some(ouve));
    let (mut h, mut st, r) = assente_com(info, c, true);
    ph2d_editor_core::paint::ouve_o_texto(None);
    let caixa = achado(&r, i::INSP_RAY_UP_TO).expect("a caixa do Ignore up to");
    let cena = h.paint_vector_scene::<InspectorPanel>(&mut st, crate::seam_ray::VIEWPORT);
    set_current_inspector_ray(None);
    (
        caixa,
        TEXTOS.with(|v| v.borrow().clone()),
        cena.probe_strokes().0.len(),
    )
}

/// ⭐⭐ **O erro mora na CAIXA do «Ignore up to»** (dono 15/11), nos dois modos: um nome fora da cadeia (ou que
/// ninguém tem) pinta UMA linha logo abaixo da caixa a dizer porquê, e a caixa ganha a moldura de erro do tema
/// (um traço a mais); um nome válido não pinta nenhuma das duas.
///
/// **Mutações que sangram:** o painel não pintar a linha do erro · não pintar a moldura.
#[test]
fn a_caixa_do_ignore_up_to_mostra_o_erro() {
    nos_dois_modos(|c| {
        let mut certo = raio();
        certo.ignore_up_to = "Truck".into();
        let (_, t, tracos_certo) = pinta_o_topo(&certo, c);
        assert!(
            !t.iter()
                .any(|(s, _)| s.contains("not above") || s.starts_with("No object")),
            "um nome válido não se queixa ({c}): {t:?}"
        );

        let mut fora = certo.clone();
        fora.ignore_up_to = "Crate".into();
        fora.ignore_up_to_off = true;
        let (caixa, t, tracos_fora) = pinta_o_topo(&fora, c);
        let frase = "Crate is not above this ray, so it is ignored.";
        let onde = t
            .iter()
            .rev()
            .find(|(s, _)| s == frase)
            .map(|(_, r)| *r)
            .unwrap_or_else(|| panic!("a linha do erro ({c}): {t:?}"));
        assert!(
            onde[1] >= caixa.y + caixa.h - 0.5 && onde[1] <= caixa.y + caixa.h * 3.0,
            "logo abaixo da caixa ({c}): {onde:?} sob {caixa:?}"
        );
        assert!(
            tracos_fora > tracos_certo,
            "a moldura de erro na caixa ({c}): {tracos_fora} contra {tracos_certo}"
        );

        let mut ninguem = certo.clone();
        ninguem.ignore_up_to = "Nobody".into();
        ninguem.ignore_up_to_unknown = true;
        let (_, t, tracos_ninguem) = pinta_o_topo(&ninguem, c);
        assert!(
            t.iter().any(|(s, _)| s == "No object is called Nobody."),
            "o nome de ninguém ({c}): {t:?}"
        );
        assert!(tracos_ninguem > tracos_certo, "a moldura ({c})");

        let mut longo = fora.clone();
        longo.ignore_up_to = "A truck with a very long name that never ends at all".repeat(3);
        let (_, t, _) = pinta_o_topo(&longo, c);
        let (s, r) = t
            .iter()
            .rev()
            .find(|(s, _)| s.ends_with("is not above this ray, so it is ignored."))
            .unwrap_or_else(|| panic!("a frase inteira, com o nome elidido ({c}): {t:?}"));
        assert!(
            s.contains('…') && (r[3] - onde[3]).abs() < 0.5,
            "uma linha ({c}): {s} {r:?} contra {onde:?}"
        );
    });
}
