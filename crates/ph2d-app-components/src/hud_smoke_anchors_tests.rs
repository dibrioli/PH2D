//! Os gates do [`Canto`] — *a peça está do lado a que ela se prende?*
//!
//! ⛔⛔ **Eles nascem de um report do dono (2026-09-20):** *«em expand, a depender da altura e
//! largura da janela, a pontuação e o rótulo da ronda podem ir para seu próprio lado ou para o lado
//! oposto e até se cruzar. Os demais modos OK.»*
//!
//! ⚠️ **O gate que devia ter apanhado isto media só a REGRA** (`as_duas_pecas_de_baixo_...` lia as
//! fracções `min` deste ficheiro e comparava-as) — *ele nunca perguntou ONDE a peça está*. Com a
//! regra certa e a posição do lado oposto ele fica verde sobre uma cena que se atravessa.

use super::{BASE, Canto};
use crate::hud_bridge::{View, anchor_frame_of};
use ph2d_ecs::{Fit, UiCanvas, VecAnchors};

/// A caixa da cena do HUD, no modo em que a wave das âncoras vive.
fn canvas() -> UiCanvas {
    UiCanvas {
        ref_w: super::REF_W,
        ref_h: super::REF_H,
        fit: Fit::Expand,
    }
}

/// **Onde a peça deste canto aterra em MUNDO**, pelas portas do produto: a caixa efectiva do
/// canvas, a regra do filho, e a escala que conduziu a raiz.
///
/// ⚠️ `local_x` entra por ARGUMENTO para o controlo poder medir a autoria ESPELHADA — sem isso o
/// gate não conteria o fenómeno que existe para impedir.
fn aterra_em(canto: Canto, local_x: f64, hw: f32, hh: f32) -> f64 {
    let vista = View {
        center: [0.0, 0.0],
        half: [hw, hh],
    };
    let (agora, escala) = anchor_frame_of(canvas(), vista).expect("a caixa da cena e' utilizavel");
    let regra = VecAnchors {
        min: [canto.fraccao(), 0.0],
        max: [canto.fraccao(), 0.0],
        base: BASE,
    };
    // A ponta mínima e a máxima andam o mesmo (é um PINO), logo o filho translada.
    let [dmin, _] = regra.delta_local(agora);
    escala[0] * (local_x + dmin[0])
}

/// Os aspectos varridos: `16:9` (onde não há banda nenhuma) até a uma janela muito larga.
const ASPECTOS: [(f32, f32); 7] = [
    (8.0, 4.5),
    (10.0, 4.5),
    (12.0, 4.5),
    (14.0, 4.5),
    (16.0, 4.5),
    (20.0, 4.5),
    (4.5, 8.0),
];

/// ⭐⭐⭐ **A peça é autorada do lado a que ela se prende** — e não por promessa: o sinal da posição
/// SAI da fracção da regra (ver [`Canto::local`]).
///
/// **Mutação que deve sangrar:** trocar o sinal, ou cravá-lo num dos lados.
#[test]
fn a_peca_de_um_canto_e_autorada_desse_lado() {
    for canto in [Canto::Esquerda, Canto::Direita] {
        let x = f64::from(canto.local()[0]);
        let para_a_direita = canto.fraccao() > 0.5;
        assert!(
            (x > 0.0) == para_a_direita,
            "{canto:?} prende-se a fraccao {} e e' autorada em x = {x}",
            canto.fraccao()
        );
    }
    // ⚠️ O CONTROLO: as duas têm de ficar em lados DIFERENTES. Sem ele, um `local` que devolvesse
    // sempre o mesmo sinal passaria metade deste gate.
    assert!(
        Canto::Esquerda.local()[0] * Canto::Direita.local()[0] < 0.0,
        "os dois cantos foram autorados do mesmo lado"
    );
}

/// ⭐⭐⭐ **A ORDEM das duas peças no ecrã não depende do aspecto da janela** — a pontuação fica
/// SEMPRE à direita da contagem, e a folga entre elas só cresce.
///
/// É a frase do report, ao contrário. **Mutação que deve sangrar:** espelhar a autoria de uma
/// delas (que é exactamente o estado de antes desta cura).
#[test]
fn as_duas_pecas_nunca_trocam_de_lado_nem_se_cruzam() {
    let esq = f64::from(Canto::Esquerda.local()[0]);
    let dir = f64::from(Canto::Direita.local()[0]);
    for (hw, hh) in ASPECTOS {
        let contagem = aterra_em(Canto::Esquerda, esq, hw, hh);
        let pontos = aterra_em(Canto::Direita, dir, hw, hh);
        assert!(
            pontos > contagem,
            "a {hw}x{hh}: os pontos aterraram em {pontos:.3} e a contagem em {contagem:.3} — \
             elas cruzaram-se"
        );
        assert!(
            contagem < 0.0 && pontos > 0.0,
            "a {hw}x{hh}: contagem {contagem:.3} / pontos {pontos:.3} — alguma saiu do lado dela"
        );
    }
}

/// ⛔⛔ **O CONTROLO do gate de cima: com a autoria ESPELHADA elas TROCAM de ordem no varrimento.**
///
/// ⚠️ Sem esta metade, o irmão acima seria verde sobre uma régua que não contém o fenómeno — e é
/// precisamente essa a história desta wave: o gate anterior media a regra, nunca a posição.
///
/// Os números são os que a sonda deu com a autoria de então (ver a tabela no doc do [`Canto`]):
/// a `16:9` a pontuação lia `−3,50` e a contagem `+4,00`; a `4,0` de aspecto, `+6,50` e `−6,00`.
#[test]
fn o_controlo_a_autoria_espelhada_de_facto_se_cruza() {
    let mut ordens = Vec::new();
    for (hw, hh) in ASPECTOS {
        // ESPELHADA: a contagem (presa à esquerda) autorada à direita, e vice-versa.
        let contagem = aterra_em(Canto::Esquerda, f64::from(Canto::Direita.local()[0]), hw, hh);
        let pontos = aterra_em(Canto::Direita, f64::from(Canto::Esquerda.local()[0]), hw, hh);
        ordens.push(pontos > contagem);
    }
    assert!(
        ordens.contains(&true) && ordens.contains(&false),
        "a autoria espelhada devia TROCAR de ordem ao longo do varrimento, e leu {ordens:?} — \
         a regua nao contem o fenomeno que o gate irmao existe para impedir"
    );
}

/// ⭐⭐ **E a PORTA que a cena chama dá a regra da ESQUERDA à contagem** — o elo que faltava entre
/// o tipo e o emparelhamento da cena.
///
/// ⚠️ Sem ele, trocar os dois `insert` do [`super::prende_os_cantos`] passava por **todos** os
/// outros gates: o tipo continua coerente, a cena continua a chamar com os argumentos certos, e as
/// duas peças ficam presas ao canto errado. *Uma lei verificada nas duas pontas ainda pode ser
/// contrariada no meio.*
#[test]
fn prende_os_cantos_da_a_regra_da_esquerda_a_contagem() {
    let mut world = bevy_ecs::world::World::new();
    let contagem = world.spawn_empty().id();
    let pontos = world.spawn_empty().id();
    super::prende_os_cantos(&mut world, contagem, pontos);
    for (e, canto, quem) in [
        (contagem, Canto::Esquerda, "contagem"),
        (pontos, Canto::Direita, "pontos"),
    ] {
        let regra = world
            .get::<VecAnchors>(e)
            .unwrap_or_else(|| panic!("a {quem} ficou sem regra nenhuma"));
        assert_eq!(
            *regra,
            canto.regra(),
            "a {quem} recebeu a regra do canto errado"
        );
    }
}
