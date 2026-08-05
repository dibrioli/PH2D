//! Gates da **máquina de estados** (plano UI/UX W7).

use crate::{Machine, ObjectPose, UiState};
use ph2d_anim::{Easing, EasingFamily, EasingMode};

fn linear() -> Easing {
    Easing::new(EasingFamily::Linear, EasingMode::InOut)
}

/// idle em x=0, hover em x=10 — um objeto, um eixo, para o número ser legível.
fn machine() -> Machine {
    let at = |name: &str, x: f64| {
        let mut s = UiState::new(name);
        s.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(1)
        }];
        s
    };
    Machine::new(vec![at("idle", 0.0), at("hover", 10.0)]).expect("maquina")
}

fn x(m: &Machine) -> f64 {
    m.pose()[0].translation[0]
}

/// **A máquina não conta o tempo — ela recebe o `dt`.**
///
/// ⚠️ O gate é a metade que se pode afirmar sem um relógio: SEM `advance` nada anda, por mais que
/// o tempo de parede passe. Uma máquina com relógio próprio andaria sozinha aqui.
#[test]
fn nothing_moves_without_an_advance() {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    let before = x(&m);
    std::thread::sleep(std::time::Duration::from_millis(30));
    assert!((x(&m) - before).abs() < 1e-12, "a maquina andou sozinha");
    m.advance(0.5);
    assert!(
        (x(&m) - 5.0).abs() < 1e-9,
        "meio segundo devia estar no meio: {}",
        x(&m)
    );
}

/// **A CHEGADA É EXATA, e ela não deriva.**
///
/// ⚠️ Este é o gate que protege o trabalho do artista: sem a chegada exata cada ida-e-volta deixa
/// um resíduo de ponto flutuante, e depois de algumas dezenas de hovers o botão já não está onde
/// ele o desenhou. O oráculo é **byte-a-byte contra a pose autorada**, depois de 50 voltas com
/// `dt` deliberadamente feio.
#[test]
fn arriving_is_exact_and_does_not_drift() {
    let authored = machine();
    let idle = authored.pose()[0].clone();
    let mut m = machine();

    for _ in 0..50 {
        m.go_to(1, 0.25, linear());
        for _ in 0..7 {
            m.advance(0.037); // 7 x 0,037 = 0,259 > 0,25 — passa do fim de propósito
        }
        m.go_to(0, 0.25, linear());
        for _ in 0..7 {
            m.advance(0.037);
        }
    }
    assert!(!m.is_animating(), "sobrou uma transicao no ar");
    assert_eq!(m.current(), 0);
    assert_eq!(
        m.pose()[0],
        idle,
        "a pose derivou depois de 50 idas-e-voltas — a chegada nao e' exata"
    );
}

/// **Interromper CONTINUA DE ONDE ESTÁ** — nunca da pose autorada.
///
/// ⚠️ É o defeito que qualquer um vê e ninguém sabe nomear: sair do hover no meio da animação e a
/// cena SALTAR para a ponta antes de começar a voltar. O oráculo é o primeiro passo da volta —
/// ele tem de sair de perto de onde a ida parou, e não de 10.
#[test]
fn an_interrupted_transition_resumes_from_where_it_is() {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    m.advance(0.3);
    let caught = x(&m);
    assert!(
        (caught - 3.0).abs() < 1e-9,
        "a ida nao chegou a 3: {caught}"
    );

    m.go_to(0, 1.0, linear());
    m.advance(0.0);
    assert!(
        (x(&m) - caught).abs() < 1e-9,
        "a volta SALTOU ao comecar: de {caught} para {}",
        x(&m)
    );
    m.advance(0.5);
    assert!(
        x(&m) < caught && x(&m) > 0.0,
        "a volta nao esta' a andar de {caught} para 0: {}",
        x(&m)
    );
}

/// **Duas transições no mesmo frame NÃO EMPILHAM** — a segunda substitui a primeira.
///
/// ⚠️ Uma fila faria a UI perseguir gestos que o artista já abandonou: passar o mouse por cima de
/// cinco botões deixaria cinco animações por reproduzir, e a última terminaria segundos depois de
/// o dedo já estar noutro lugar.
///
/// ⚠️ **A fixture tem TRÊS estados, e isso é load-bearing.** A primeira versão pedia `1, 0, 1` com
/// dois estados — e ali *"a primeira ganha"* e *"a última ganha"* dão a MESMA resposta, então a
/// mutação que empilha passava por ela e sangrava noutro gate. O primeiro alvo e o último têm de
/// diferir para a pergunta sequer existir.
#[test]
fn two_transitions_in_one_frame_do_not_stack() {
    let at = |name: &str, x: f64| {
        let mut s = UiState::new(name);
        s.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(1)
        }];
        s
    };
    let mut m =
        Machine::new(vec![at("idle", 0.0), at("hover", 10.0), at("press", 20.0)]).expect("maquina");

    m.go_to(1, 1.0, linear());
    m.go_to(2, 1.0, linear());

    m.advance(1.0);
    assert!(!m.is_animating(), "sobrou transicao empilhada");
    assert_eq!(
        m.current(),
        2,
        "acabou no PRIMEIRO alvo pedido, nao no ultimo — as transicoes empilharam"
    );
    assert!((x(&m) - 20.0).abs() < 1e-9);
}

/// **Duração não-positiva é uma troca INSTANTÂNEA**, e ela chega pela mesma porta.
///
/// ⚠️ O caminho instantâneo não é um ramo próprio: ele chama a MESMA `arrive`, então não há como
/// ele divergir da chegada normal — que é onde a exatidão e a remoção de quem sai moram.
///
/// ⚠️ **A fixture tem um objeto que SAI, e isso é o que torna o gate capaz de falhar.** A primeira
/// versão movia um objeto só, e ali um ramo próprio (`overlay(tr.at(1.0))`) dá exactamente o mesmo
/// resultado que a porta certa — a mutação sobreviveu. As duas portas só divergem onde a `arrive`
/// faz algo a mais: **remover quem saiu**.
#[test]
fn a_zero_duration_lands_exactly_and_by_the_same_door() {
    let mut open = UiState::new("open");
    open.objects = vec![ObjectPose::new(1), ObjectPose::new(2)];
    let mut closed = UiState::new("closed");
    closed.objects = vec![ObjectPose {
        translation: [4.0, 0.0],
        ..ObjectPose::new(1)
    }];
    let authored = closed.objects.clone();
    let mut m = Machine::new(vec![open, closed]).expect("maquina");

    m.go_to(1, 0.0, linear());
    assert!(!m.is_animating(), "uma troca instantanea ficou no ar");
    assert_eq!(m.current(), 1);
    assert_eq!(
        m.pose(),
        authored.as_slice(),
        "a troca instantanea nao pousou no estado autorado — quem saiu ficou, ou a pose difere"
    );
}

/// **O EASING deforma o `t`, nunca o relógio.**
///
/// ⚠️ Deformar o `dt` faria a duração autorada deixar de ser a duração real — duas transições com
/// o mesmo número acabariam em instantes diferentes. O oráculo separa as duas coisas: com um ease
/// não-linear o MEIO do caminho não é o meio da distância, **e mesmo assim** a transição acaba
/// exactamente na duração pedida.
#[test]
fn the_easing_bends_the_path_not_the_clock() {
    let mut eased = machine();
    let mut lin = machine();
    let e = Easing::new(EasingFamily::Cubic, EasingMode::In);
    eased.go_to(1, 1.0, e);
    lin.go_to(1, 1.0, linear());

    eased.advance(0.5);
    lin.advance(0.5);
    assert!(
        x(&eased) < x(&lin) - 0.5,
        "um ease-in cubico devia estar bem ATRAS do linear no meio: {} vs {}",
        x(&eased),
        x(&lin)
    );

    eased.advance(0.5);
    lin.advance(0.5);
    assert!(!eased.is_animating() && !lin.is_animating());
    assert!(
        (x(&eased) - x(&lin)).abs() < 1e-12,
        "os dois deviam acabar juntos"
    );
}

/// **Ir para onde já se está, parado, é um no-op.** Um clique repetido não re-anima nada.
#[test]
fn going_where_you_already_are_is_a_no_op() {
    let mut m = machine();
    m.go_to(0, 1.0, linear());
    assert!(!m.is_animating());
    m.go_to(99, 1.0, linear());
    assert!(!m.is_animating(), "um alvo inexistente armou uma transicao");
}

/// **Quem SAI é removido na chegada** — no fim, quem saiu não está lá.
#[test]
fn what_leaves_is_gone_when_the_transition_lands() {
    let mut a = UiState::new("open");
    a.objects = vec![ObjectPose::new(1), ObjectPose::new(2)];
    let mut b = UiState::new("closed");
    b.objects = vec![ObjectPose::new(1)];
    let mut m = Machine::new(vec![a, b]).expect("maquina");

    m.go_to(1, 1.0, linear());
    m.advance(0.5);
    assert_eq!(
        m.pose().len(),
        2,
        "quem sai devia continuar visivel a meio caminho"
    );
    assert!(m.pose().iter().find(|p| p.id == 2).unwrap().opacity < 1.0);

    m.advance(0.5);
    assert_eq!(
        m.pose().len(),
        1,
        "quem saiu continuou na cena depois de chegar"
    );
    assert_eq!(m.pose()[0].id, 1);
}
