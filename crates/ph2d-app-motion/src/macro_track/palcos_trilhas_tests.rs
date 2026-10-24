//! W19 §2.6 — vários Motion no mesmo objecto Macro: cada trilha do Motion toca no seu palco, sobre o
//! seu objecto; duas sobre o MESMO objecto tocam a mais recente que já começou.

use super::gate_tests::{QUADROS, pelo_nucleo, retrato, sessao_em};
use super::palcos::PalcosDasMacros;
use crate::motion_mundo::MundoDaCena;
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, SimWorld};
use ph2d_macro::{MacroTime, Nascimento};

fn em(q: u32) -> f64 {
    f64::from(q) * 0.016
}

fn nasce(nome: &str) -> Option<Nascimento> {
    Some(Nascimento {
        inicio: MacroTime(0),
        nome: nome.to_owned(),
        local: [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0],
        pai: None,
    })
}

/// Toca `doc` em `q` e devolve os donos tocados e o retrato de cada palco que toca.
fn quadro(sim: &mut SimWorld, palcos: &mut PalcosDasMacros, q: u32) -> (Vec<u64>, Vec<Vec<u32>>) {
    let e = sim
        .world_mut()
        .query::<(ph2d_ecs::Entity, &Macro)>()
        .iter(sim.world())
        .next()
        .expect("o objecto Macro")
        .0;
    sim.world_mut()
        .entity_mut(e)
        .insert(MacroTempo { segundos: em(q) });
    let tocados = palcos.segue(sim.world_mut(), &MundoDaCena::default(), None);
    let retratos = palcos.fontes().map(|(m, _)| retrato(m)).collect();
    (tocados.into_iter().collect(), retratos)
}

fn cena(doc: &ph2d_macro::MacroDoc) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn(Macro {
        doc: ph2d_macro::codec::write(doc),
        modo: ModoDaMacro::Fiel,
    });
    sim
}

/// ⛔ VERMELHO antes da W19: o `truncate(1)` deixava só a 1.ª trilha do Motion tocar.
#[test]
fn dois_motion_no_mesmo_objeto_macro_tocam_os_dois() {
    let g = sessao_em(None);
    let mut doc = pelo_nucleo(&g);
    let mut b = doc.tracks[0].clone();
    (doc.tracks[0].target, doc.tracks[0].nasce) = (7, nasce("A"));
    (b.target, b.nasce) = (8, nasce("B"));
    doc.tracks.push(b);
    let mut sim = cena(&doc);
    let mut palcos = PalcosDasMacros::default();
    for q in [QUADROS / 2, QUADROS - 1] {
        let (tocados, _) = quadro(&mut sim, &mut palcos, q);
        assert_eq!(tocados, [7, 8], "quadro {q}: os dois objectos tocam");
        // Com alvo, o palco coze só os sinks do objecto (os da sessão não têm dono, o retrato é vazio):
        // confere-se que cada um seguiu o relógio.
        let t = super::em_cena::instante(em(q));
        assert_eq!(palcos.tocando(), [(7, t), (8, t)], "quadro {q}");
    }
}

/// ⛔ VERMELHO antes da W19: o objecto mostrava sempre a 1.ª trilha, mesmo depois de a 2.ª começar.
#[test]
fn o_mesmo_motion_toca_a_trilha_mais_recente_que_ja_comecou() {
    let g = sessao_em(None);
    // Sem alvo (`0` = todos), o retrato é o da sessão.
    let mut doc = pelo_nucleo(&g);
    let desde = QUADROS + 20;
    let mut b = doc.tracks[0].clone();
    for ev in &mut b.events {
        ev.t = MacroTime(ev.t.0 + MacroTime::from_ms(u64::from(desde) * 16).0);
    }
    b.continua = Some(0);
    doc.tracks.push(b);
    let mut sim = cena(&doc);
    let mut palcos = PalcosDasMacros::default();
    let fim = (QUADROS - 1) as usize;
    let meio = QUADROS / 2;
    // (quadro, retrato esperado): a 1.ª até a 2.ª começar; depois a 2.ª; e de volta (scrub) a 1.ª.
    for (q, r) in [
        (meio, meio as usize),
        (desde - 1, fim),
        (desde + meio, meio as usize),
        (QUADROS - 1, fim),
        (meio, meio as usize),
    ] {
        let (tocados, vistos) = quadro(&mut sim, &mut palcos, q);
        assert_eq!(tocados, [0], "quadro {q}");
        assert_eq!(
            vistos,
            [g.retratos[r].clone()],
            "quadro {q}: o palco que toca"
        );
    }
    assert_ne!(
        g.retratos[fim], g.retratos[meio as usize],
        "controlo: retratos distintos"
    );
}
