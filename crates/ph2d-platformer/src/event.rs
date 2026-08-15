//! **O que ACONTECEU neste tique** — as transições do player, o canal irmão do
//! [`crate::PlayerView`].
//!
//! # ⚠️ Dois canais, e a distinção não é arrumação
//!
//! A [`crate::PlayerView`] é estado **CONTÍNUO** — *o personagem está no chão,
//! olha para a direita, agacha*. Um evento é uma **TRANSIÇÃO** — *acabou de
//! aterrar* — e vale por exactamente um tique.
//!
//! Colapsá-los num só canal custa nas duas direcções. Um consumidor que só tem
//! o estado tem de guardar a cópia dele para diferençar, e cada consumidor
//! guardaria a sua (duas respostas à mesma pergunta, divergindo no dia em que
//! uma delas esqueça um tique). Um consumidor que só tem eventos não sabe
//! desenhar o primeiro quadro: nada aconteceu ainda, e mesmo assim o
//! personagem está de pé, virado para algum lado.
//!
//! # ⚠️ O evento nasce DENTRO do laço de tiques
//!
//! Um dispatch pode dever vários tiques (catch-up, scrub, um quadro lento), e
//! um diff entre dois quadros da shell **perde os do meio**: um pulo que sai e
//! aterra dentro do mesmo dispatch não teria acontecido. É literalmente o
//! defeito que o `W-TickContacts` mediu no canal de contatos — lá o primeiro
//! pouso de uma queda de 3 m não gerava evento nenhum — e a cura é a mesma:
//! comparar por TIQUE, nunca por quadro.
//!
//! # ⚠️ Quase tudo é derivável do par de vistas; o pulo NÃO é
//!
//! *Aterrou*, *chegou ao ápice*, *entrou na água* — todos saem de comparar duas
//! vistas consecutivas. **Que tipo de pulo saiu, não**: a lei sabe se o pé
//! empurrou o chão, se gastou uma carga do ar ou se a parede o lançou, e de
//! fora só se pode adivinhar por *"ele estava no chão no tique anterior?"* —
//! que responde **não** para o pulo do ar E para o de parede, ou seja erra
//! exactamente no caso que se queria distinguir. Por isso o
//! [`crate::PlayerStep`] o **declara** ([`crate::JumpKind`]) e este módulo
//! apenas o repassa.

use crate::{FootingKind, JumpKind, PlayerStep, PlayerView, Vec2, relative_along};

/// **Uma coisa que aconteceu ao personagem neste tique.**
///
/// ⚠️ Sem `PartialOrd`/ordem implícita: a lista é uma sequência de fatos do
/// mesmo tique, e nada aqui promete que um venha antes do outro.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PlayerEvent {
    /// **O pé encontrou chão.**
    ///
    /// A `speed` é a velocidade de DESCIDA **relativa ao chão** (m/s, sempre
    /// ≥ 0) medida no tique em que a lei aceitou a superfície — antes de o
    /// motor daquele tique correr. É ela que dimensiona a poeira e o som.
    ///
    /// ⚠️ **Relativa ao chão, nunca ao mundo**: aterrar num elevador que sobe
    /// a 5 m/s enquanto se cai a 5 m/s é um pouso SUAVE, e uma medida absoluta
    /// diria o contrário — a mesma razão que fez a [`relative_rise`] nascer.
    Landed {
        /// Metros por segundo, para baixo, relativos ao chão.
        speed: f32,
    },
    /// **A cabeça bateu** — o espelho do [`Self::Landed`], na outra ponta.
    ///
    /// A `speed` é a velocidade de SUBIDA (m/s, sempre ≥ 0) no tique em que o
    /// sensor relatou o bloqueio, medida antes de o motor daquele tique correr
    /// — a mesma régua e o mesmo instante do pouso.
    ///
    /// ⚠️ **ABSOLUTA, não relativa ao teto**, e a diferença para o irmão é
    /// honesta em vez de esquecida: a vista publica a velocidade do CHÃO
    /// ([`PlayerView::ground_velocity`]) e não a de um teto, então não há
    /// segundo corpo contra o qual subtrair. E não é uma omissão que morde: o
    /// bit `ceiling` só existe com o personagem NO AR
    /// ([`crate::ceiling_fact_wanted`] exige `!grounded`), onde a velocidade
    /// de chão publicada é `[0, 0]` — logo a forma relativa que o `Landed` usa
    /// **já reduz** à absoluta aqui, e as duas leis dão o mesmo número. Um teto
    /// que se MOVE (um elevador por cima) precisaria de uma `ceiling_velocity`
    /// na vista, o que é outro sensor e outra wave.
    Bonked {
        /// Metros por segundo, para cima.
        speed: f32,
    },
    /// **Um pulo SAIU**, e de onde — ver o aviso do módulo.
    Jumped {
        /// Chão, ar ou parede.
        kind: JumpKind,
    },
    /// **O ápice** — a subida relativa cruzou de positiva para não-positiva com
    /// o personagem no ar (o `bNotifyApex` do Unreal).
    ///
    /// ⚠️ **Só no ar, de propósito.** Sem essa guarda, um elevador que
    /// desacelera sob os pés do personagem parado publica um ápice — e o que
    /// chegou ao topo foi o elevador, não ele.
    Apex,
    /// **O arranque começou.**
    Dashed,
    /// **A trava do pendurar armou.**
    LedgeGrabbed,
    /// **Entrou na água.**
    EnteredWater,
    /// **Saiu da água.**
    LeftWater,
}

/// **As transições entre duas vistas consecutivas**, empurradas em `out`.
///
/// `before` é a vista do tique ANTERIOR e `step` é o tique que acabou de
/// correr — o par que o laço de tiques tem na mão.
///
/// ⚠️ **Quem chama é dono do "não há anterior".** No primeiro tique de uma
/// corrida — e no primeiro depois de um scrub ou de a simulação ser desarmada
/// — não existe vista anterior, e a resposta certa é **evento nenhum**: um
/// evento aqui é estritamente uma transição, e sem um "antes" não há transição
/// a relatar, só uma a inventar. É por isso que esta função **exige** o
/// `before` em vez de o aceitar como `Option`: o caso "não há anterior" fica
/// impossível de esquecer, porque não há como o exprimir.
///
/// ⚠️ Isto difere do canal de CONTATOS de propósito, e a razão é o que cada um
/// afirma: um relatório de contato é um fato sobre o mundo **agora** (*isto
/// está a tocar naquilo*), então uma baseline vazia reportar tudo no primeiro
/// tique é honesto. `Landed` afirma um **verbo no passado**.
pub fn events_between(
    before: &PlayerView,
    step: &PlayerStep,
    up: Vec2,
    out: &mut Vec<PlayerEvent>,
) {
    let after = &step.view;

    // O pulo é o único que a lei DECLARA em vez de a comparação derivar.
    if let Some(kind) = step.jump {
        out.push(PlayerEvent::Jumped { kind });
    }

    // ── O POUSO ──────────────────────────────────────────────────────────────
    // ⚠️ A rampa recusada conta como "não era chão": quem escorrega por uma
    // encosta de 60° e alcança um patamar ATERRA, e a postura `Steep` existe
    // precisamente para não ser confundida com chão (W9).
    if after.footing == FootingKind::Ground && before.footing != FootingKind::Ground {
        out.push(PlayerEvent::Landed {
            speed: (-rise(after, up)).max(0.0),
        });
    }

    // ── A BATIDA DE CABEÇA ───────────────────────────────────────────────────
    // ⚠️ **É a BORDA de um booleano que a vista já publica** — mecanicamente da
    // família das travas lá em baixo, e escrito aqui porque se LÊ como o espelho
    // do pouso: as duas pontas do mesmo eixo, uma ao lado da outra.
    //
    // ⚠️ **A borda basta porque o bit se apaga sozinho:** o sensor do teto só é
    // consultado a SUBIR (`ceiling_fact_wanted`), então assim que o solver mata
    // a subida o bit cai — não há estado pegajoso a segurar um evento aceso.
    if after.ceiling && !before.ceiling {
        out.push(PlayerEvent::Bonked {
            speed: rise(after, up).max(0.0),
        });
    }

    // ── O ÁPICE ──────────────────────────────────────────────────────────────
    if after.footing == FootingKind::Airborne && rise(before, up) > 0.0 && rise(after, up) <= 0.0 {
        out.push(PlayerEvent::Apex);
    }

    // ── AS TRAVAS ────────────────────────────────────────────────────────────
    // Cada uma é a BORDA de um booleano que a vista já publica.
    if after.dashing && !before.dashing {
        out.push(PlayerEvent::Dashed);
    }
    if after.ledging && !before.ledging {
        out.push(PlayerEvent::LedgeGrabbed);
    }
    if after.swimming != before.swimming {
        out.push(if after.swimming {
            PlayerEvent::EnteredWater
        } else {
            PlayerEvent::LeftWater
        });
    }
}

/// A subida relativa que a vista descreve.
///
/// ⚠️ **Não é uma segunda fórmula** — é a [`relative_along`], a MESMA que a
/// [`crate::relative_rise`] da lei envolve; o que difere é de onde vem a
/// velocidade do chão (a vista publica-a, a lei lê-a da amostra). O `up` chega
/// por parâmetro porque é do mundo, e uma vista não o carrega.
fn rise(view: &PlayerView, up: Vec2) -> f32 {
    relative_along(view.velocity, view.ground_velocity, up)
}

#[cfg(test)]
#[path = "event_tests.rs"]
mod tests;
