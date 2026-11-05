//! ⭐ **O scrub num cartão de COMPONENTE preso, um a um** (`docs/CartoesUI/10`, smoke 3): o Fill, o Stroke
//! e o Bone tinham censo, os componentes não. Toda pega de cada cartão: anda, o quadro da seleção não a
//! devolve, o Esc repõe.
#![cfg(all(feature = "panel-inspector", feature = "panel-hierarchy"))]

use super::cartoes_mao::{clica, pinta};
use super::o_alfinete_fill_stroke_c2::scrub_do_preso::{MesaDoPreso, censo};
use super::o_bloco5_abertos::escolhe;
use super::o_bloco8_player_solto::{hero_com_aberto, solto_de};
use super::o_inspector_armado::{arma_tudo, fisica_armada};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids::{self as i, INSP_PANEL};
use ph2d_editor_core::interaction::loose_pin_of;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_inspector as insp;

/// A seleção é outro objecto, parado: trocá-la a meio do gesto larga-o, de propósito (`sync.rs`). O que
/// ela publica é o número da fixtura: o defeito é voltar à partida.
const OUTRO: u64 = 0xB;

struct MesaComp(HeroScreen);

impl MesaDoPreso for MesaComp {
    fn hero(&mut self) -> &mut HeroScreen {
        &mut self.0
    }
    fn quadros(&mut self, n: usize) {
        for _ in 0..n {
            arma_tudo();
            let pins = self.0.store.loose_pins_of(INSP_PANEL);
            insp::com_cada_preso(&pins, |_| arma_tudo());
            escolhe(OUTRO, "Other", 1.0);
            self.0
                .store
                .set_card_subject(INSP_PANEL, Some((OUTRO, "Other".to_owned())));
            pinta(&mut self.0, 1);
        }
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
    let mut m = MesaComp(h);
    m.quadros(4);
    m
}

/// Os que, armados, não mostram campo com pega (medido 04/11). Um que ganhe pega, ou um dos outros que
/// a perca, acusa-se aqui.
const SEM_PEGA: [&str; 4] = ["Action", "Tags", "Sequence", "Trigger"];

/// Toda pega do cartão: anda, o quadro não a devolve, o Esc repõe.
fn censo_do_cartao(nome: &str, sec: NodeId) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut m = preso(sec);
    let (n, erros) = censo(&mut m, sec, nome);
    eprintln!("CENSO {nome} {n}");
    assert_eq!(n == 0, SEM_PEGA.contains(&nome), "{nome}: {n} pegas");
    assert!(erros.is_empty(), "{nome}: {n} pegas\n{erros:#?}");
}

/// Um teste por cartão, porque o paralelo é o de PROCESSOS do nextest: em fios, a pintura espera pelo
/// cadeado do registo de painéis (26 cartões em 32 fios: 52 s; em série: 50 s).
macro_rules! cartoes {
    ($($teste:ident $nome:literal $sec:ident,)*) => {
        /// ⭐ O censo: cada cartão de componente, um a um.
        /// *Mutação: a caixa tem três guardas (a do store, `vivas` em `sync.rs` e a mão em cada `sync_*`), e uma
        /// sozinha sobrevive; as três em `sync_physics` ⇒ vermelho em Joint e Wheel: «levou a caixa de 1300 a 100».*
        mod every_scrub_handle_of_each_pinned_component_card_survives_the_frames {
            $(#[test]
            fn $teste() {
                super::censo_do_cartao($nome, super::i::$sec);
            })*
        }
    };
}

cartoes! {
    physics "Physics" INSP_LIVE_PHYSICS_SECTION,
    joint "Joint" INSP_LIVE_JOINT_SECTION,
    wheel "Wheel" INSP_LIVE_WHEEL_SECTION,
    player "Player" INSP_LIVE_PLAYER_SECTION,
    timer "Timer" INSP_LIVE_TIMER_SECTION,
    action "Action" INSP_LIVE_ACTION_SECTION,
    audio "Audio" INSP_LIVE_AUDIO_SECTION,
    camera "Camera" INSP_LIVE_CAMERA_SECTION,
    tags "Tags" INSP_LIVE_TAGS_SECTION,
    factory "Factory" INSP_LIVE_FACTORY_SECTION,
    lifecycle "Lifecycle" INSP_LIVE_LIFECYCLE_SECTION,
    topdown "TopDown" INSP_LIVE_TOPDOWN_SECTION,
    projectile "Projectile" INSP_LIVE_PROJECTILE_SECTION,
    ray "Ray" INSP_LIVE_RAY_SECTION,
    state_machine "StateMachine" INSP_LIVE_SM_SECTION,
    script "Script" INSP_LIVE_SCRIPT_SECTION,
    particles "Particles" INSP_LIVE_PARTICLES_SECTION,
    hud "Hud" INSP_LIVE_HUD_SECTION,
    sequence "Sequence" INSP_LIVE_SEQ_SECTION,
    watch "Watch" INSP_LIVE_WATCH_SECTION,
    trigger "Trigger" INSP_LIVE_TRIGGER_SECTION,
    tween "Tween" INSP_LIVE_TWEEN_SECTION,
    path_follow "PathFollow" INSP_LIVE_PATHFOLLOW_SECTION,
    shake "Shake" INSP_LIVE_SHAKE_SECTION,
    emitter "Emitter" INSP_LIVE_EMITTER_SECTION,
    weapon "Weapon" INSP_LIVE_WEAPON_SECTION,
}
