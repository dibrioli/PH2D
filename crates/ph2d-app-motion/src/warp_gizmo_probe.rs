//! ⛔⛔ **A SONDA QUE DEVIA TER VINDO PRIMEIRO** — *qual das seis condições faz o gizmo do
//! `motion.bezier_warp` não existir?*
//!
//! Report do Enio, 2026-09-08, DUAS vezes: *«sumiu com o gizmo»* e, depois de eu reverter a
//! mudança que julguei ser a causa, *«ainda invisível no canvas»*. As duas curas saíram de
//! hipóteses lidas do código; nenhuma foi medida, e as duas estavam erradas.
//!
//! ⚠️ **O `resolve` tem SEIS saídas antecipadas e devolve um `Option` só.** Quem o lê sabe que o
//! gizmo não existe e **não sabe porquê** — e é isso que faz cada tentativa de cura ser um
//! palpite. Esta sonda corre a mesma porta do produto e imprime a primeira condição que falha.
//!
//! ⛔⛔⛔ **E ELA MENTIU NA PRIMEIRA CORRIDA — leia isto antes de acreditar nela.** Ela deu
//! `resolve = Some` na cena exacta do report enquanto o app dava `None`, porque **ela escolhe a
//! ROTA**: chama `advance_or_scrub_scoped`, a marcha da CPU. O app corre a rota do device, e na
//! rota **totalmente na GPU** a ponte retorna **antes** de qualquer marcha — as tomadas nunca
//! são cozidas, e é a tomada que dá a caixa envolvente ao gizmo.
//!
//! ⚠️ ***Uma sonda que escolhe a rota mede a rota que ela escolheu.*** Foi preciso o
//! `PH2D_WARP_DIAG=1` no app do dono para a resposta aparecer — e a linha que a deu foi o ramo
//! do `None`, que só existe porque eu já tinha errado duas vezes.
//!
//! ⇒ a cura vive no substrato: [`ph2d_eval_motion::MotionCookPump::cook_taps_only`], chamada na
//! rota `FullyGpu` da ponte. Esta sonda fica para a PRÓXIMA pergunta sobre as seis condições —
//! com o aviso de que ela responde pela rota da CPU.
//!
//! ```text
//! cargo test -p ph2d-host-desktop --bins --release -- --ignored --nocapture why_the_warp_gizmo_is_not_there
//! ```

use crate::motion_state::MotionState;
use crate::warp_gizmo;
use crate::warp_gizmo_fixtures as fx;
use ph2d_nodegraph::graph::NodeId;

/// `motion.grid → <nó> → motion.output`, marchada na CPU — a rota que ESTA sonda escolhe.
/// ⚠️ A montagem vive em [`fx`] justamente para o portão do device poder usar a MESMA cena
/// sem herdar esta marcha, que é o que faz a sonda mentir sobre a rota do produto.
fn cena(tipo: &str) -> (MotionState, NodeId) {
    let (mut m, no) = fx::cadeia(tipo);
    fx::marcha_na_cpu(&mut m);
    (m, no)
}

/// A cena do dono (a `=111` + `motion.bezier_warp`), marchada na CPU — ver [`cena`].
fn cena_do_dono(depois_do_mirror: bool) -> (MotionState, NodeId) {
    let (mut m, bw) = fx::cena_do_dono(depois_do_mirror);
    fx::marcha_na_cpu(&mut m);
    (m, bw)
}

#[test]
#[ignore = "sonda de diagnóstico — corra à mão"]
fn why_the_warp_gizmo_is_not_there_in_the_owners_scene() {
    let _trava = fx::trava();
    for (rotulo, depois) in [("ANTES do mirror", false), ("DEPOIS do mirror", true)] {
        let (m, no) = cena_do_dono(depois);
        eprintln!("\n  === cena 111 + bezier_warp {rotulo} ===");
        relatorio(&m, no);
    }
    eprintln!();
}

/// As seis condições, uma linha cada.
fn relatorio(m: &MotionState, no: NodeId) {
    eprintln!(
        "  1. no' de warp seleccionado ....... {:?}",
        warp_gizmo::selected_warp(m).map(|(n, _)| n)
    );
    eprintln!(
        "  2. `warp` legivel (nao por fio) ... {:?}",
        warp_gizmo::warp_amount(&m.doc.graph, no)
    );
    let up = warp_gizmo::upstream_of(&m.doc.graph, no);
    eprintln!("  3. no' a montante ................. {up:?}");
    eprintln!(
        "  4. TOMADAS armadas ................ {:?}",
        warp_gizmo::taps_for(m)
    );
    let tapped: Vec<NodeId> = m.pump.tap_streams().iter().map(|(n, _)| *n).collect();
    eprintln!("  5. tomadas que DISPARARAM ......... {tapped:?}");
    eprintln!(
        "  6. caixa envolvente ............... {:?}",
        up.and_then(|u| warp_gizmo::box_from_tap(m, u))
    );
    eprintln!(
        "  => resolve ......................... {}",
        if warp_gizmo::resolve(m, true).is_some() {
            "Some — o gizmo EXISTE"
        } else {
            "None — NAO ha' gizmo"
        }
    );
}

#[test]
#[ignore = "sonda de diagnóstico — corra à mão"]
fn why_the_warp_gizmo_is_not_there() {
    let _trava = fx::trava();
    for tipo in ["motion.four_point_warp", "motion.bezier_warp"] {
        let (m, no) = cena(tipo);
        eprintln!("\n  === {tipo} ===");
        eprintln!(
            "  1. no' de warp seleccionado ....... {:?}",
            warp_gizmo::selected_warp(&m).map(|(n, _)| n)
        );
        eprintln!(
            "  2. `warp` legivel (nao por fio) ... {:?}",
            warp_gizmo::warp_amount(&m.doc.graph, no)
        );
        let up = warp_gizmo::upstream_of(&m.doc.graph, no);
        eprintln!("  3. no' a montante ................. {up:?}");
        eprintln!(
            "  4. TOMADAS armadas ................ {:?}",
            warp_gizmo::taps_for(&m)
        );
        let tapped: Vec<NodeId> = m.pump.tap_streams().iter().map(|(n, _)| *n).collect();
        eprintln!("  5. tomadas que DISPARARAM ......... {tapped:?}");
        eprintln!(
            "  6. caixa envolvente ............... {:?}",
            up.and_then(|u| warp_gizmo::box_from_tap(&m, u))
        );
        eprintln!(
            "  => resolve ......................... {}",
            if warp_gizmo::resolve(&m, true).is_some() {
                "Some — o gizmo EXISTE"
            } else {
                "None — NAO ha' gizmo"
            }
        );
        // A rota que o planeador escolhe, que e' a suspeita numero um: um no' reivindicado
        // pela GPU pode nao deixar stream de CPU na tomada.
        let plano = ph2d_gpu_cook::plan(&m.doc.graph, &m.registry, &m.registry, m.sinks[0]);
        eprintln!(
            "  (plano: cadeia inteira no device? {})",
            plano.is_fully_gpu()
        );
    }
    eprintln!();
}
