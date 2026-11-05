//! A revisão de capricho do Inspector, por gesto (`docs/CartoesUI/18_REVISAO_DE_CAPRICHO.md`): Instance,
//! Properties, Script, conta-gotas, Constraints e o número em repouso. Cada id é clicado no meio da caixa
//! que o cartão pinta, e dá o mesmo `Click` que o painel de antes dava.

use super::lei_da_compactacao::mede;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_inspector::ids as i;
use ph2d_ui_testkit::MockPanelHost;

/// O Inspector armado, pintado; `f` recebe a mão e os rects da última pintura.
fn armado(f: impl FnOnce(&mut MockPanelHost, &dyn Fn(NodeId) -> Rect)) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let mut host = MockPanelHost::new().in_theme(ph2d_tokens::Theme::Dark);
    // O clique passa pelo registo: fora deste bloco, que o tranca.
    ph2d_editor_core::panel::with_registry(|reg| {
        let painel = reg
            .panels_mut()
            .iter_mut()
            .find(|p| p.manifest.id == "inspector")
            .expect("o inspector");
        super::o_inspector_armado::arma_tudo();
        painel.populate(host.store_mut());
        let _ = mede(&mut host, painel);
        super::o_inspector_armado::desarma_tudo();
    });
    let regs = host.registos_da_ultima_pintura();
    let rect = move |id: NodeId| {
        regs.iter()
            .rev()
            .find(|(n, _)| *n == id)
            .map_or_else(|| panic!("{id:?} não foi pintado"), |(_, r)| *r)
    };
    f(&mut host, &rect);
}

#[track_caller]
fn clica_no_meio(host: &mut MockPanelHost, id: NodeId, r: Rect) {
    let ev = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "o clique em {id:?} deu {ev:?}"
    );
}

/// Instance e Properties: botões com a altura de todo controlo, de ponta a ponta do corpo, e o clique de
/// cada id. *Mutação: tirar o `register` de `Mao::botao`, ou voltar a `paint_instance_card` ⇒ vermelho.*
#[test]
fn instance_and_properties_answer_at_field_height() {
    armado(|host, rect| {
        let botoes = [
            i::INSP_INSTANCE_OPEN_PREFAB,
            i::INSP_INSTANCE_APPLY_ADDED[0],
            i::INSP_INSTANCE_RESTORE_PIECE[0],
            i::INSP_INSTANCE_CLEAR_ORPHANS,
        ];
        let w = rect(botoes[0]).w;
        for id in botoes {
            let r = rect(id);
            assert!((r.h - m::FIELD_H).abs() < 0.01 && r.w == w, "{id:?}: {r:?}");
            clica_no_meio(host, id, r);
        }
        let x = rect(i::INSP_INSTANCE_DROP_ORPHAN[0]);
        assert!((x.w - m::FIELD_H).abs() < 0.01 && (x.h - m::FIELD_H).abs() < 0.01);
        assert!(
            (x.x + x.w - (rect(botoes[0]).x + w)).abs() < 0.5,
            "o × encosta à direita"
        );
        clica_no_meio(host, i::INSP_INSTANCE_DROP_ORPHAN[0], x);
        for id in &i::INSP_INSTANCE_AXIS_OPTION[0][..2] {
            let r = rect(*id);
            clica_no_meio(host, *id, r);
        }
    });
}

/// Constraints e Script: os botões dividem a linha, na mesma altura. *Mutação: `par = false` ⇒ vermelho.*
#[test]
fn constraints_and_script_buttons_share_their_rows() {
    armado(|host, rect| {
        for fila in [
            &[
                i::INSP_CONSTRAINT_SUBIR[0],
                i::INSP_CONSTRAINT_DESCER[0],
                i::INSP_CONSTRAINT_APAGAR[0],
            ][..],
            &[i::INSP_CONSTRAINT_BAKE, i::INSP_CONSTRAINT_ADD][..],
            &[i::INSP_SCRIPT_BROWSE, i::INSP_SCRIPT_EDIT][..],
        ] {
            let a = rect(fila[0]);
            let mut fim = a.x;
            for &id in fila {
                let r = rect(id);
                let mesma = (r.y - a.y).abs() < 0.01 && (r.h - m::FIELD_H).abs() < 0.01;
                assert!(mesma, "{id:?}: {r:?} contra {a:?}");
                assert!(r.x >= fim, "{id:?} por cima do vizinho");
                fim = r.x + r.w;
                clica_no_meio(host, id, r);
            }
        }
    });
}

/// O conta-gotas mora dentro da caixa do alvo, grande, e responde. *Mutação: `caixa_de_alvo` devolve o
/// `icone` que recebeu ⇒ vermelho.*
#[test]
fn the_eyedroppers_live_inside_their_box() {
    armado(|host, rect| {
        for id in [
            i::INSP_JOINT_PICK_A,
            i::INSP_JOINT_PICK_B,
            i::INSP_WHEEL_MOUNT_PICK,
            i::INSP_WHEEL_ROPE_PICK,
            i::INSP_CAMERA_TARGET_PICK,
        ] {
            let r = rect(id);
            assert_eq!((r.w, r.h), (m::GRID_CELL, m::GRID_CELL), "{id:?}");
            clica_no_meio(host, id, r);
        }
    });
}
