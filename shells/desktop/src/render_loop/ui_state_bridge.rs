//! **A ponte dos ESTADOS de UI** (plano UI/UX W7) — quem faz a cena ANDAR entre duas poses.
//!
//! # O que esta ponte dirige, e o que ela deliberadamente NÃO dirige
//!
//! Ela dirige o botão **Show**: o artista pede um papel e a cena **transita** até ele, com a
//! duração que ele autorou. É assim que ele vê o tween que gravou, em vez de apenas confiar que
//! ele existe.
//!
//! ⚠️ **Ela não é dirigida pelo mouse DAQUI, e as duas razões continuam de pé:** um hover que
//! animasse a forma enquanto o artista trabalha tornaria o editor inutilizável (é por isso que o
//! Figma põe a interação num **modo de apresentação** separado), e o undo deste editor é por
//! **DIFF do mundo**, então uma pose escrita por hover viraria passo de undo a cada passagem do
//! rato.
//!
//! ⇒ quem liga o rato é o **MODO DE PREVIEW** ([`super::ui_preview`], W7r): ele resolve as duas
//! de uma vez — enquanto corre, o gesto de edição não existe e o undo não regista, e ao sair o
//! mundo volta exactamente ao que era. Ele PEDE por esta mesma porta ([`request`]), então não há
//! um segundo caminho para *"pôr a cena nesta pose"*.
//!
//! # O relógio é o do FRAME, e não um relógio próprio
//!
//! `advance(dt)` recebe o `dt` que o `FixedStep` já produziu. É a lição W4.T7 do Motion, onde o
//! `MotionTransport` **morreu**: dois relógios divergem, e o modo de falha é a UI a andar noutra
//! velocidade que a cena.

use std::collections::BTreeMap;

use ph2d_ui_state::{Machine, StateRole, StateSets};
use ph2d_vec_scene::{VecPathId, VecScene};

use ph2d_ecs::SimWorld;

use crate::vec_entities::VecEntityMap;

/// As máquinas VIVAS, por hospedeiro.
///
/// ⚠️ **Elas não são serializadas, e não podem ser:** uma máquina é *onde a cena está agora*, e
/// o documento guarda *onde as poses são*. Salvar a máquina faria um projeto reabrir a meio
/// caminho de um hover.
pub(crate) type UiMachines = BTreeMap<VecPathId, Machine>;

/// **Pede a `host` que vá para `role`.** Cria a máquina se ela ainda não existe.
///
/// ⚠️ A máquina nasce parada no primeiro estado gravado, e é por isso que a construção lê a
/// tabela inteira: partir de um estado inventado faria a primeira transição vir de uma pose que
/// ninguém autorou.
pub(crate) fn request(
    machines: &mut UiMachines,
    states: &StateSets,
    host: VecPathId,
    role: StateRole,
) {
    let (duration, easing) = states.timing(host);
    let m = machines
        .entry(host)
        .or_insert_with(|| Machine::new(states.get(host).to_vec()).unwrap_or_else(empty_machine));
    // ⚠️ **Re-alinha ANTES de pedir**, todas as vezes: o artista re-grava, e a tabela muda
    // debaixo de uma máquina que já existe. Sem isto o Show seguinte animaria para a pose ANTIGA,
    // e o botão pareceria ignorar a gravação que acabou de acontecer.
    m.retarget(states.get(host).to_vec());
    m.go_to_role(role, duration, easing);
}

/// Uma máquina sem estado nenhum — o caso em que o hospedeiro perdeu as poses entre o pedido e o
/// frame. `go_to_role` nela é um no-op, que é a resposta certa.
fn empty_machine() -> Machine {
    // `Machine::new` recusa uma lista vazia de propósito; o `Default` do estado é o mínimo que
    // torna a máquina construível e inerte.
    Machine::new(vec![ph2d_ui_state::UiState::new(StateRole::Default)])
        .expect("uma lista de um estado nunca é vazia")
}

/// Roda um frame: anda as máquinas e escreve as poses.
///
/// Devolve `true` se **alguma** máquina está em voo — o chamador usa para suprimir o registro de
/// undo, que é o mesmo tratamento que um gesto em andamento recebe.
///
/// ⚠️ **A supressão é o que impede um passo de undo POR FRAME.** Sem ela, cada quadro de uma
/// transição de 150 ms seria um estado diferente do mundo e o diff registraria todos; o artista
/// pediria *"desfazer o Show"* e teria de apertar Ctrl+Z nove vezes. Quando a máquina CHEGA, a
/// cena está numa pose autorada e o diff registra **um** passo — que é exactamente o que *"eu
/// mostrei o hover"* deve custar.
pub(crate) fn dispatch(
    machines: &mut UiMachines,
    states: &mut StateSets,
    sim: &mut SimWorld,
    scene: &mut VecScene,
    map: &VecEntityMap,
    dt: f64,
) -> bool {
    // **Uma forma apagada leva os estados dela.** Sem isto o documento acumularia poses de
    // objetos que ninguém vê, e elas viajariam no arquivo para sempre.
    //
    // ⚠️ E no MESMO frame do apagamento, de propósito: a tabela viaja no `ProjectState`, então
    // limpá-la um frame depois seria um SEGUNDO passo de undo — o artista apagaria o botão e
    // precisaria de dois Ctrl+Z.
    if !states.is_empty() {
        states.retain_hosts(|id| scene.paths().iter().any(|p| p.id == id));
    }
    // Um hospedeiro que perdeu todas as poses perde a máquina: ela descreveria um caminho entre
    // estados que já não existem.
    machines.retain(|host, _| !states.get(*host).is_empty());

    let mut animating = false;
    for m in machines.values_mut() {
        let was = m.is_animating();
        m.advance(dt);
        // ⚠️ Escreve enquanto ANDA e no frame em que CHEGA. Parar de escrever no instante da
        // chegada deixaria a cena no penúltimo `t`, e a promessa de chegada EXATA morreria a um
        // frame do fim.
        if was {
            for p in m.pose() {
                crate::vec_ui_state_edit::install(sim, scene, map, p);
            }
        }
        animating |= m.is_animating();
    }
    animating
}

/// Que papel a cena mostra para `host` — o índice, para o painel.
#[must_use]
pub(crate) fn live_role(machines: &UiMachines, host: Option<VecPathId>) -> Option<usize> {
    let h = host?;
    let m = machines.get(&h)?;
    // ⚠️ Em voo o que interessa é PARA ONDE ela vai, não de onde saiu: o artista pediu aquele
    // papel, e o readout que dissesse o de partida piscaria o nome errado durante a transição.
    let i = m.target().unwrap_or_else(|| m.current());
    // ⚠️ O índice da MÁQUINA é posição na lista GRAVADA; o painel pinta a lista dos QUATRO
    // papéis. Devolver um sem traduzir acenderia a linha errada assim que um papel do meio não
    // estivesse gravado.
    let role = m.role(i)?;
    StateRole::ALL.iter().position(|&r| r == role)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_anim::{Easing, EasingFamily, EasingMode};
    use ph2d_ui_state::{ObjectPose, UiState};

    fn table(host: VecPathId) -> StateSets {
        let mut s = StateSets::default();
        for (role, x) in [(StateRole::Default, 0.0), (StateRole::Hover, 10.0)] {
            let mut st = UiState::new(role);
            st.objects = vec![ObjectPose {
                translation: [x, 0.0],
                ..ObjectPose::new(host)
            }];
            s.set(host, st);
        }
        s.set_easing(host, Easing::new(EasingFamily::Linear, EasingMode::InOut));
        s
    }

    fn x(machines: &UiMachines, host: VecPathId) -> f64 {
        machines[&host].pose()[0].translation[0]
    }

    /// **REPRO DO PRODUTO (Enio, 2026-08-05): apertar Show e nada acontecer.**
    ///
    /// A sequência é a que o artista faz — Show Hover, interromper a meio com Show Default — e a
    /// resposta tem de ser *a cena volta ao Default*, não *a cena fica parada até você pedir o
    /// Hover outra vez*.
    ///
    /// ⚠️ Este gate mora na PONTE de propósito: o defeito só existia na composição
    /// `retarget` + `go_to`, e nenhum dos dois sozinho o mostrava — a ponte é o único sítio onde
    /// os dois são chamados na ordem em que o produto os chama.
    #[test]
    fn interrupting_a_show_with_another_show_moves_the_scene() {
        let host: VecPathId = 1;
        let states = table(host);
        let mut machines = UiMachines::new();

        request(&mut machines, &states, host, StateRole::Hover);
        machines.get_mut(&host).unwrap().advance(0.05);
        let mid = x(&machines, host);
        assert!(
            mid > 0.1 && mid < 9.9,
            "a fixture nao interrompeu nada a meio: {mid}"
        );

        request(&mut machines, &states, host, StateRole::Default);
        assert!(
            machines[&host].is_animating(),
            "o Show Default nao produziu transicao nenhuma: a cena ficou parada em x = {mid}"
        );
        machines.get_mut(&host).unwrap().advance(1.0);
        assert!(
            x(&machines, host).abs() < 1e-9,
            "a cena nao voltou ao Default: x = {}",
            x(&machines, host)
        );
    }
}
