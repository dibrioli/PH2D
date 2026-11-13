//! Bug #14: a ponte impõe atrito 0 + `Min` à forma de todo corpo com `PlatformPlayer`
//! (`ph2d_physics_ecs::atrito_ignorado`), então Friction e Friction Combine dessa forma não têm leitor.
//! Controlo morto não fica à vista: nas duas faces (cartões e `PH2D_CARDS=0`), no corpo e na forma filha,
//! as linhas não são pintadas nem registadas; Bounce e Bounce Combine continuam vivos ao clique real.
//! **Mutação que deve sangrar:** `paint_bounce_friction`/`paint_combine_rows` ignorarem `com_atrito`.

use ph2d_a11y::NodeId;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit as E};
use ph2d_panel_inspector::ids::*;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, set_cards_on, set_current_inspector_physics,
};

use crate::o_cartao_da_fisica::{achado, cena, clica, pinta, sai};
use crate::seam_physics::with_body;

fn mortos() -> Vec<NodeId> {
    let mut v = vec![INSP_PHYS_FRICTION];
    v.extend(INSP_PHYS_FRIC_COMBINE);
    v
}

fn player(info: InspectorPhysicsInfo) -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        friction_ignored: true,
        ..info
    }
}

fn da_filha() -> InspectorPhysicsInfo {
    InspectorPhysicsInfo {
        has_body: false,
        part_owner: "Player".into(),
        shape_of: "Player".into(),
        ..with_body()
    }
}

fn dica() -> &'static str {
    ph2d_i18n::tr("card.insp.phys.no_friction")
}

/// Cartões: com o player, nenhuma linha morta; o Bounce e a regra dele respondem ao clique.
fn cartoes(nome: &str, info: InspectorPhysicsInfo) {
    let (mut host, mut st) = cena(player(info.clone()), [1.0, 1.0]);
    let (r, avisos) = ph2d_panel_inspector::censo_dos_avisos::medindo(|| pinta(&mut host, &mut st));
    for id in mortos() {
        assert!(achado(&r, id).is_none(), "{nome}: {id:?} pintado no player");
    }
    assert!(
        avisos.iter().any(|a| a.texto == dica()),
        "{nome}: sem a dica do atrito: {avisos:?}"
    );
    assert!(
        achado(&r, INSP_PHYS_RESTITUTION).is_some(),
        "{nome}: Bounce"
    );
    let max = INSP_PHYS_REST_COMBINE[3];
    let caixa = achado(&r, max).unwrap_or_else(|| panic!("{nome}: Bounce Combine"));
    assert_eq!(
        clica(&mut host, &mut st, caixa),
        [EditorAction::InspectorPhysicsEdit {
            entity_bits: info.entity_bits,
            edit: E::RestitutionCombine(3),
        }],
        "{nome}: o clique no Max do Bounce Combine"
    );
    sai();
    // O controlo: sem player, as linhas continuam.
    let (mut host, mut st) = cena(info, [1.0, 1.0]);
    let (r, avisos) = ph2d_panel_inspector::censo_dos_avisos::medindo(|| pinta(&mut host, &mut st));
    for id in mortos() {
        assert!(achado(&r, id).is_some(), "{nome}: {id:?} sumiu sem player");
    }
    assert!(
        !avisos.iter().any(|a| a.texto == dica()),
        "{nome}: dica sem player"
    );
    sai();
}

#[test]
fn o_corpo_do_player_nao_mostra_o_atrito_nos_cartoes() {
    cartoes("Corpo", with_body());
}

#[test]
fn a_forma_filha_do_player_nao_mostra_o_atrito_nos_cartoes() {
    cartoes("Filha", da_filha());
}

#[test]
fn a_forma_do_player_nao_mostra_o_atrito_na_face_classica() {
    use ph2d_editor_core::zones::Rect;
    let viewport = Rect::new(0.0, 0.0, 320.0, 2400.0);
    for (info, vivo) in [(player(with_body()), false), (with_body(), true)] {
        set_cards_on(false);
        let mut host = crate::host_em_metros();
        let mut st = InspectorState::default();
        set_current_inspector_physics(Some(info));
        let r = host.paint::<InspectorPanel>(&mut st, viewport);
        assert!(achado(&r, INSP_PHYS_RESTITUTION).is_some(), "Bounce");
        assert!(
            achado(&r, INSP_PHYS_REST_COMBINE[0]).is_some(),
            "Bounce Combine"
        );
        for id in mortos() {
            assert_eq!(
                achado(&r, id).is_some(),
                vivo,
                "{id:?} com player = {}",
                !vivo
            );
        }
        set_current_inspector_physics(None);
    }
}
