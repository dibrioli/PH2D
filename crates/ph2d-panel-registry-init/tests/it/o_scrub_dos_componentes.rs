//! ⭐ **O scrub num cartão de COMPONENTE preso, um a um** (`docs/CartoesUI/10`, smoke 3): a cura do Weight
//! é na porta do store, comum a todos, mas só o Fill, o Stroke e o Bone tinham censo. Toda pega de cada
//! cartão: anda, o quadro da seleção não a devolve, o Esc repõe.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{clica, pinta};
use super::o_alfinete_fill_stroke_c2::scrub_do_preso::{MesaDoPreso, censo};
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{arma_tudo, fisica_armada};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::{self as i, INSP_PANEL};
use ph2d_editor_core::interaction::loose_pin_of;
use ph2d_editor_core::screens::hero::{HeroScreen, InspectorNameInfo};
use ph2d_panel_inspector as insp;

/// A seleção é outro objecto; o que ela publica nas secções é o mesmo número da fixtura, e basta: o
/// defeito é o quadro devolver a caixa ao valor da partida.
const OUTRO: u64 = 0xB;

struct MesaComp(HeroScreen, u64);

impl MesaDoPreso for MesaComp {
    fn hero(&mut self) -> &mut HeroScreen {
        &mut self.0
    }
    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            arma_tudo();
            let pins = self.0.store.loose_pins_of(INSP_PANEL);
            insp::com_cada_preso(&pins, |_| arma_tudo());
            insp::set_current_inspector_name(Some(InspectorNameInfo {
                entity_bits: OUTRO,
                name: "Other".to_owned(),
            }));
            self.0
                .store
                .set_card_subject(INSP_PANEL, Some((OUTRO, "Other".to_owned())));
            pinta(&mut self.0, 1);
        }
        let _ = self.1;
    }
}

/// O cartão `sec` solto e preso ao objecto da fixtura, com outro escolhido.
fn preso(sec: NodeId) -> MesaComp {
    arma_tudo();
    let a = fisica_armada().entity_bits;
    let mut h = hero_com_aberto(sec);
    solto_de(&mut h, sec, 300.0);
    clica(&mut h, loose_pin_of(sec));
    assert_eq!(
        h.store.loose_card(sec).and_then(|c| c.pin),
        Some(a),
        "{sec:?}"
    );
    let mut m = MesaComp(h, a);
    m.quadros(4);
    m
}

/// `(nome, secção, pegas medidas)`. O número conta-se: uma pega nova entra no censo e acusa-se aqui.
const CARTOES: [(&str, NodeId); 26] = [
    ("Physics", i::INSP_LIVE_PHYSICS_SECTION),
    ("Joint", i::INSP_LIVE_JOINT_SECTION),
    ("Wheel", i::INSP_LIVE_WHEEL_SECTION),
    ("Player", i::INSP_LIVE_PLAYER_SECTION),
    ("Timer", i::INSP_LIVE_TIMER_SECTION),
    ("Action", i::INSP_LIVE_ACTION_SECTION),
    ("Audio", i::INSP_LIVE_AUDIO_SECTION),
    ("Camera", i::INSP_LIVE_CAMERA_SECTION),
    ("Tags", i::INSP_LIVE_TAGS_SECTION),
    ("Factory", i::INSP_LIVE_FACTORY_SECTION),
    ("Lifecycle", i::INSP_LIVE_LIFECYCLE_SECTION),
    ("TopDown", i::INSP_LIVE_TOPDOWN_SECTION),
    ("Projectile", i::INSP_LIVE_PROJECTILE_SECTION),
    ("Ray", i::INSP_LIVE_RAY_SECTION),
    ("StateMachine", i::INSP_LIVE_SM_SECTION),
    ("Script", i::INSP_LIVE_SCRIPT_SECTION),
    ("Particles", i::INSP_LIVE_PARTICLES_SECTION),
    ("Hud", i::INSP_LIVE_HUD_SECTION),
    ("Sequence", i::INSP_LIVE_SEQ_SECTION),
    ("Watch", i::INSP_LIVE_WATCH_SECTION),
    ("Trigger", i::INSP_LIVE_TRIGGER_SECTION),
    ("Tween", i::INSP_LIVE_TWEEN_SECTION),
    ("PathFollow", i::INSP_LIVE_PATHFOLLOW_SECTION),
    ("Shake", i::INSP_LIVE_SHAKE_SECTION),
    ("Emitter", i::INSP_LIVE_EMITTER_SECTION),
    ("Weapon", i::INSP_LIVE_WEAPON_SECTION),
];

/// ⭐ O censo: cada cartão de componente, um a um.
#[test]
fn every_scrub_handle_of_each_pinned_component_card_survives_the_frames() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut erros = Vec::new();
    let mut medidas = Vec::new();
    for (nome, sec) in CARTOES {
        let mut m = preso(sec);
        let (n, e) = censo(&mut m, sec, nome);
        medidas.push((nome, n));
        erros.extend(e);
    }
    eprintln!("CENSO COMPONENTES {medidas:?}");
    let total: usize = medidas.iter().map(|(_, n)| n).sum();
    assert!(total > 0, "a régua não achou pega nenhuma");
    assert!(erros.is_empty(), "{medidas:?}\n{erros:#?}");
}
