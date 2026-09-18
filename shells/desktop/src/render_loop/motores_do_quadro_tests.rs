//! ⭐⭐⭐ **A FIAÇÃO do gatilho** (suplente #24) — os dois factos que a lei pura não pode afirmar.
//!
//! ⚠️⚠️ **Porque este ficheiro existe:** os seis gates da lei vivem na `ph2d-ecs` e entram por
//! [`ph2d_ecs::dispara_gatilhos`], que fica **ABAIXO** da decisão que só a shell toma — *quando é
//! que o motor corre*. ⭐ A outra metade (*de onde vêm as amostras*) mudou-se para a crate da
//! família com a função que a responde (`ph2d_app_components::trigger_bridge`), porque ela é PURA:
//! aqui fica só a COMPOSIÇÃO. A casa já pagou essa forma pelo menos duas vezes (a
//! entrega do teclado ao mover de vista de cima, §10 do handoff do #13; o replay que dirigia um
//! controlador e não os outros dois, §9 do #14), e as duas vezes com a suíte VERDE por cima.
//!
//! ⛔ E nenhuma das duas é observável de fora: o [`super::gatilhos`] é privado
//! **de propósito** (quem o chama é o corpo do outbox, uma vez), logo a régua tem de morar aqui.

use ph2d_ecs::{ActionEdge, ActionTriggerRow, SignalOnAction, SimWorld};
use ph2d_input::{ActionState, Binding, InputMap, InputState, Key};
use ph2d_runtime::{SignalOutbox, SignalReader};

use ph2d_app_components::trigger_bridge::amostras_das_accoes;

use super::{Relogio, gatilhos};

/// O ESPAÇO, que é a tecla que o prólogo do smoke liga.
const ESPACO: Key = Key(0x20);
const ACCAO: &str = "fire";
const SINAL: &str = "tiro";

/// Um mapa com UMA acção ligada ao espaço. ⚠️ **Não é o `with_player_defaults`** — ele tem sete
/// acções e nenhuma é disparar, que é exactamente a medição que obriga o prólogo a criar esta.
fn mapa() -> InputMap {
    let mut m = InputMap::new();
    let id = m.create(ACCAO);
    if let Some(a) = m.get_mut(id) {
        a.bindings.push(Binding::Key(ESPACO));
    }
    m
}

/// O estado das acções depois de resolver um tique por entrada de `premido`, do repouso.
///
/// ⚠️ **O percurso é uma SEQUÊNCIA e nunca um instante:** o `just_pressed` é a diferença entre o
/// tique de agora e o anterior, logo uma resolução só mede uma borda contra o VAZIO e não
/// distingue *«acabou de carregar»* de *«já estava em baixo»* — que é a propriedade inteira do
/// `Press`.
fn estado(premido: &[bool]) -> ActionState {
    let m = mapa();
    let mut dev = InputState::new();
    let mut st = ActionState::new();
    for &p in premido {
        dev.begin_frame();
        if p {
            dev.keyboard.handle_key_down(ESPACO);
        } else {
            dev.keyboard.handle_key_up(ESPACO);
        }
        st.tick(&m, &dev);
    }
    st
}

/// Um mundo com um gatilho `Press` na acção `fire`.
fn mundo(edge: ActionEdge, accao: &str) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn(SignalOnAction(vec![ActionTriggerRow {
        action: accao.to_owned(),
        edge,
        signal: SINAL.to_owned(),
    }]));
    sim
}

/// Os nomes publicados por uma corrida de [`gatilhos`].
fn publicados(sim: &mut SimWorld, playing: bool, st: &ActionState) -> Vec<String> {
    let mut out = SignalOutbox::new();
    let mut leitor = SignalReader::at(&out);
    let accoes = amostras_das_accoes(&mapa(), st);
    gatilhos(
        sim,
        &mut out,
        &Relogio::do_quadro(playing, 1, 1.0 / 60.0),
        &accoes,
    );
    out.read(&mut leitor).map(|s| s.name.to_string()).collect()
}

/// ⭐⭐⭐ **A CERCA DO RELÓGIO — parado, o teclado é do editor.**
///
/// ⚠️ **É a metade que decide se o app é usável:** o gatilho do smoke ouve o ESPAÇO, e sem esta
/// cerca cada espaço que o artista escreve num campo publicaria um sinal.
#[test]
fn com_o_relogio_parado_o_gatilho_nao_fala_e_a_andar_fala() {
    let st = estado(&[false, true]);

    let mut sim = mundo(ActionEdge::Press, ACCAO);
    assert!(
        publicados(&mut sim, false, &st).is_empty(),
        "com a corrida PARADA nenhum sinal pode sair"
    );

    let mut sim = mundo(ActionEdge::Press, ACCAO);
    assert_eq!(
        publicados(&mut sim, true, &st),
        vec![SINAL.to_owned()],
        "com a corrida A ANDAR o toque tem de chegar ao barramento"
    );
}

/// ⭐⭐ **E a lei da acção inexistente atravessa a fiação INTEIRA** — o nome errado fica calado nas
/// três arestas, e a que interessa é o `Release` (as outras duas ficariam caladas por acidente).
#[test]
fn um_nome_que_o_mapa_nao_conhece_fica_calado_ate_no_release() {
    let st = estado(&[false, true]);
    for edge in ActionEdge::ALL {
        let mut sim = mundo(edge, "fier");
        assert!(
            publicados(&mut sim, true, &st).is_empty(),
            "{edge:?}: um nome desconhecido tem de ficar calado"
        );
    }
}
