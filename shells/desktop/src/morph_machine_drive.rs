//! **A MÁQUINA DE MORPH A CORRER** (plano 32 W5) — quem faz a forma virar a outra.
//!
//! # ⚠️ Ela só corre num MODO, e isso não é conservadorismo
//!
//! A condição de uma seta é uma **acção do Input Map**, isto é, uma tecla. Se a máquina escutasse
//! enquanto o artista edita, carregar em `Z` faria a forma mudar **e** o que quer que o `Z` faça no
//! editor — os dois, sem que nada na tela explicasse. É o argumento do [`crate::render_loop::ui_preview`]
//! (*"um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável"*)
//! com outro dispositivo de entrada, e a resposta é a mesma: **um modo**.
//!
//! # ⛔⛔ O PLAYHEAD ERA A PORTA E DEIXOU DE SER — e a diferença é uma medição, não um gosto
//!
//! A W5 escreveu que *"o modo já existe: neste editor, o jogo a correr é o playhead a andar"*, e
//! era um argumento bom sobre a coisa errada. **O playhead não tranca o teclado do editor.** Com
//! ele a andar, as teclas continuam a chegar aos atalhos — então a mesma tecla morfa a forma **e**
//! faz o que ela faz no editor, que é exactamente o que a nota dizia estar a evitar.
//!
//! Enio, 2026-08-25, depois do smoke: *"precisamos de um modo preview (com botão) como o de states
//! de animação pois senão temos conflitos de atalhos (como setas do teclado movendo as formas)"*.
//!
//! ⇒ a porta é o **interruptor `Preview`** da seção *Morph States*, e ele **toma o teclado**
//! (`input_dispatch::keyboard`, logo depois do retrato dos dispositivos e antes de todo atalho).
//! ⛔ **Uma porta, não duas:** deixar o playhead a dirigir também manteria o conflito viva na porta
//! que não tranca nada — e *duas portas para o mesmo modo divergem em silêncio*.
//!
//! ⚠️ *Um modo cuja entrada não exclui os outros consumidores não é um modo — é mais um produtor.*
//!
//! # ⚠️ O que ela escreve é PRÉ-VISUALIZAÇÃO, e o undo não a vê
//!
//! A máquina escreve **dois** campos do `VecMorph` — o par e o `t` —, e os dois passam pelo ledger
//! ([`crate::preview_drive`]). ⛔ **O `Driver::MorphT` sozinho não bastava:** ele cobre o `t` e
//! **só** o `t`, e sem o `MorphPair` uma transição durante a reprodução entraria no undo como se o
//! artista tivesse re-ligado as fontes à mão.
//!
//! # ⚠️ E parar o relógio DEVOLVE a forma autorada
//!
//! Isso não custa código: é o que o ledger faz por construção — a captura repõe o autorado, e ao
//! largar as máquinas a cena volta ao que o artista desenhou. *Sair restaura o MUNDO, nunca «vá
//! para o estado inicial»* — que moveria o desenho dele.

use std::collections::BTreeMap;

use ph2d_ecs::{Entity, SimWorld, VecMorph, VecMorphMachine};
use ph2d_input::{ActionState, Input, InputMap};
use ph2d_morph_machine::MorphMachine;

use crate::preview_drive::{Driven, PreviewDrive};

/// As máquinas VIVAS, por entidade de Morph.
///
/// ⚠️ **Não são serializadas, e não podem ser:** uma máquina é *onde a forma está agora*, e o
/// documento guarda *quais são as setas*. Salvá-la faria um projecto reabrir a meio de uma
/// transição. Mesma lei, palavra por palavra, das `UiMachines`.
///
/// ⚠️ `BTreeMap` e não `HashMap` — a espinha do determinismo deste repo (lint estrutural).
pub(crate) type MorphMachines = BTreeMap<u64, MorphMachine>;

/// **Um quadro da máquina.** Devolve quantas máquinas correram.
///
/// `active` é o **modo de pré-visualização**: falso ⇒ as máquinas são **largadas** e nada é escrito
/// (o ledger devolve o autorado sozinho, na próxima captura).
///
/// ⚠️ **O nome é `active` e não `playing` de propósito** — ele deixou de ser o playhead na W9, e um
/// parâmetro que continuasse a chamar-se `playing` faria a próxima leitura procurar o transporte.
pub(crate) fn tick(
    machines: &mut MorphMachines,
    sim: &mut SimWorld,
    map: &InputMap,
    actions: &ActionState,
    active: bool,
    dt: f64,
    drive: &mut PreviewDrive,
) -> usize {
    if !active {
        // ⭐ **Largar é a restauração.** Não há «voltar ao estado inicial» aqui: o que o artista vê
        // ao parar é o que ele DESENHOU, e quem o repõe é o ledger — pela mesma porta que já repõe
        // a pose do solver e o relógio da §11.
        machines.clear();
        return 0;
    }
    let hosts: Vec<(u64, ph2d_morph_machine::MorphGraph)> = sim
        .world_mut()
        .query::<(Entity, &VecMorphMachine)>()
        .iter(sim.world())
        .map(|(e, m)| (e.to_bits(), m.graph.clone()))
        .collect();
    // Uma máquina cuja entidade morreu (ou perdeu as setas) some junto — senão ela sobreviveria ao
    // objecto e o mapa cresceria para sempre. Mesma varredura das `UiMachines`.
    machines.retain(|k, _| hosts.iter().any(|(h, _)| h == k));

    let input = Input::new(map, actions);
    let mut ran = 0;
    for (bits, graph) in hosts {
        let e = Entity::from_bits(bits);
        let m = machines
            .entry(bits)
            .or_insert_with(|| MorphMachine::new(&graph));
        // ⚠️ **Só o que ACABOU de ser carregado dispara.** Com `pressed` uma tecla segurada
        // re-disparava a cada quadro e a máquina saltaria a cadeia inteira num piscar de olhos.
        for a in m.live_actions(&graph) {
            if input.just_pressed(a) {
                m.fire(&graph, a);
                break;
            }
        }
        m.advance(&graph, dt);
        let (pair, t) = ([m.pair().0, m.pair().1], m.t());
        // ⚠️ **O ledger primeiro, a escrita depois** — ele precisa do valor ANTES para saber o que
        // repor. Escrever e só então registar guardaria o valor do motor como se fosse o autorado.
        write_driven(sim, e, drive, Driven::MorphPair(pair));
        write_driven(sim, e, drive, Driven::MorphT(t));
        ran += 1;
    }
    ran
}

/// Regista o valor ANTES no ledger e escreve o novo — a porta única das duas metades.
fn write_driven(sim: &mut SimWorld, e: Entity, drive: &mut PreviewDrive, after: Driven) {
    let Some(before) = Driven::read(after.driver(), sim, e) else {
        return;
    };
    if before == after {
        return;
    }
    drive.driven(e, before, after);
    let Some(mut m) = sim.world_mut().get_mut::<VecMorph>(e) else {
        return;
    };
    match after {
        Driven::MorphPair(p) => m.sources = p,
        Driven::MorphT(v) => m.t = v,
        _ => {}
    }
}

#[cfg(test)]
#[path = "morph_machine_drive_tests.rs"]
mod tests;
