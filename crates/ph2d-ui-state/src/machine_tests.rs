//! Gates da **máquina de estados** (plano UI/UX W7).

use crate::{Machine, ObjectPose, Spring, StateRole, UiState};
use ph2d_anim::{Easing, EasingFamily, EasingMode};

fn linear() -> Easing {
    Easing::new(EasingFamily::Linear, EasingMode::InOut)
}

/// idle em x=0, hover em x=10 — um objeto, um eixo, para o número ser legível.
fn at(role: StateRole, x: f64) -> UiState {
    let mut s = UiState::new(role);
    s.objects = vec![ObjectPose {
        translation: [x, 0.0],
        ..ObjectPose::new(1)
    }];
    s
}

fn states() -> Vec<UiState> {
    vec![at(StateRole::Default, 0.0), at(StateRole::Hover, 10.0)]
}

fn machine() -> Machine {
    Machine::new(states()).expect("maquina")
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
    let at = |role: StateRole, x: f64| {
        let mut s = UiState::new(role);
        s.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(1)
        }];
        s
    };
    let mut m = Machine::new(vec![
        at(StateRole::Default, 0.0),
        at(StateRole::Hover, 10.0),
        at(StateRole::Pressed, 20.0),
    ])
    .expect("maquina");

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
    let mut open = UiState::new(StateRole::Default);
    open.objects = vec![ObjectPose::new(1), ObjectPose::new(2)];
    let mut closed = UiState::new(StateRole::Hover);
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
    let mut a = UiState::new(StateRole::Default);
    a.objects = vec![ObjectPose::new(1), ObjectPose::new(2)];
    let mut b = UiState::new(StateRole::Hover);
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

/// **Um papel que ninguém gravou recua para o Default.**
///
/// ⚠️ É o que torna a lista de papéis OPCIONAL. Sem o recuo, um botão que autora só o Hover
/// ficaria **preso no hover** ao ser apertado — e autorar um papel a mais passaria a ser um
/// requisito escondido de autorar todos.
#[test]
fn a_role_nobody_recorded_falls_back_to_the_default() {
    let at = |role: StateRole, x: f64| {
        let mut s = UiState::new(role);
        s.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(1)
        }];
        s
    };
    // Só Default e Hover: o Pressed nunca foi gravado.
    let mut m = Machine::new(vec![
        at(StateRole::Default, 0.0),
        at(StateRole::Hover, 10.0),
    ])
    .expect("maquina");

    m.go_to_role(StateRole::Hover, 0.0, linear());
    assert_eq!(x(&m), 10.0);

    m.go_to_role(StateRole::Pressed, 0.0, linear());
    assert_eq!(
        x(&m),
        0.0,
        "apertar um botao sem estado de Pressed devia voltar ao repouso, nao ficar preso"
    );
    assert_eq!(m.current(), 0);
}

/// **E se nem o Default existe, `go_to_role` não faz nada** — em vez de escolher um estado ao
/// acaso, que é como uma cena inteira salta ao primeiro hover.
///
/// ⚠️ **A fixture não tem Default DE PROPÓSITO, e tem DOIS estados.** A lista é ordenada por
/// papel, então *"o Default"* e *"o primeiro da lista"* coincidem sempre que o Default existe —
/// uma coincidência que deixaria um recuo escrito como `states[0]` verde em toda a suíte. Só uma
/// lista SEM Default separa as duas respostas.
#[test]
fn without_a_default_a_role_request_is_a_no_op() {
    let at = |role: StateRole, x: f64| {
        let mut s = UiState::new(role);
        s.objects = vec![ObjectPose {
            translation: [x, 0.0],
            ..ObjectPose::new(1)
        }];
        s
    };
    let mut m = Machine::new(vec![
        at(StateRole::Hover, 10.0),
        at(StateRole::Pressed, 20.0),
    ])
    .expect("maquina");
    m.go_to(1, 0.0, linear());
    assert_eq!(x(&m), 20.0);

    m.go_to_role(StateRole::Disabled, 0.0, linear());
    assert_eq!(
        x(&m),
        20.0,
        "um recuo para `states[0]` levou a cena para um estado que ninguem pediu"
    );
    assert!(!m.is_animating());
}

/// **A INTERRUPÇÃO do default não tem parada, e é por isso que não há solver de mola.**
///
/// ⚠️ O que uma mola dá e uma curva não dá é **continuidade de velocidade**. Medido (a sonda
/// `measure_spring`): revertendo a 30% do caminho, a volta arranca a **1,34×** a velocidade com
/// que a ida chegava sob o `Cubic Out` que shipa — o olho não separa isso de 1,00× —, enquanto o
/// `InOut` arranca a **0,00×** (a cena PARA e recomeça, o *stutter* que faz alguém pedir um
/// solver) e o `Elastic` a **7,02×** (estalo).
///
/// ⚠️ **E a FORMA de mola já está no catálogo:** `Elastic Out` mede pico 1,373 / assenta 0,631 /
/// 4 travessias contra 1,309 / 0,600 / 3 de uma mola macia — a mesma animação. O solver não se
/// justifica; o que ele compraria é exactamente o regime que este gate proíbe.
///
/// ⚠️ **Quem mover o default reconfere a nota** (§0): escolher `InOut` aqui torna a parada
/// alcançável, e o item da mola volta à mesa.
#[test]
fn the_default_curve_reverses_without_stopping_dead() {
    const AT: f64 = 0.30;
    const H: f64 = 1e-4;
    let y = |e: Easing, u: f64| e.eval(u);

    let d = crate::DEFAULT_EASING;
    let incoming = (y(d, AT + H) - y(d, AT - H)) / (2.0 * H);
    let outgoing = (y(d, H) - y(d, 0.0)) / H * y(d, AT);
    let ratio = outgoing.abs() / incoming.abs();
    assert!(
        ratio > 0.5,
        "o default reverte a {ratio:.2}x — a cena PARA no meio do gesto e recomeça, que e' o \
         stutter para o qual uma mola de verdade existe"
    );
    assert!(
        ratio < 2.5,
        "o default reverte a {ratio:.2}x — a volta ESTALA, arrancando muito mais rapido do que a \
         ida chegava"
    );

    // ⚠️ **O CONTROLE:** a família que de facto para tem de medir a parada, senão o gate acima
    // seria verde por a razão ser insensível ao que ela julga.
    let inout = Easing {
        family: EasingFamily::Cubic,
        mode: EasingMode::InOut,
    };
    let stop = ((y(inout, H) - y(inout, 0.0)) / H * y(inout, AT)).abs()
        / ((y(inout, AT + H) - y(inout, AT - H)) / (2.0 * H)).abs();
    assert!(
        stop < 0.1,
        "o `InOut` deixou de parar ({stop:.2}x) — o controle deste gate dissolveu-se e a razao \
         acima passou a nao poder falhar"
    );
}

/// **REPRO (Enio, 2026-08-05): o Show que não fazia nada.**
///
/// *"Se a animação de hover foi interrompida pelo usuário e o usuário aperta Show Default, a
/// animação não acontece até que o usuário aperte Show hover para finalizar a animação hover."*
///
/// ⚠️ A causa não estava no pedido, estava na PERGUNTA. O guard de [`Machine::go_to`] comparava o
/// **rótulo** (`target == current`) — um proxy para *"a cena já mostra este estado"* —, e o proxy
/// **expira no instante em que um voo é abortado**: `current` continua a nomear o estado de onde
/// se saiu enquanto a pose viva está a meio caminho do outro. O guard passa a perguntar pela
/// POSE, que é a coisa de que ele sempre falou.
#[test]
fn an_aborted_flight_does_not_deafen_the_next_show() {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    m.advance(0.3);
    let mid = x(&m);
    assert!(
        mid > 0.1 && mid < 9.9,
        "a fixture nao contem o fenomeno: nada foi interrompido a meio ({mid})"
    );

    // O ABORTO REAL: o artista re-grava enquanto a transição corre (a tabela muda debaixo da
    // máquina). O voo morre, a pose viva fica onde estava, e `current` ainda diz `Default`.
    m.retarget(vec![
        at(StateRole::Default, 0.0),
        at(StateRole::Hover, 20.0),
    ]);
    assert!(!m.is_animating(), "a fixture nao abortou o voo");
    assert_eq!(m.current(), 0, "a fixture nao reproduz o estado do repro");

    // E agora o pedido que não fazia nada.
    m.go_to(0, 1.0, linear());
    m.advance(1.0);
    assert!(
        x(&m).abs() < 1e-9,
        "o Show do papel em que a maquina DIZ estar foi recusado, e a cena ficou parada a meio \
         caminho do outro: x = {}",
        x(&m)
    );
}

/// **Pedir o estado que a cena JÁ mostra continua a não animar — e passa a registá-lo.**
///
/// ⚠️ É a outra metade do gate acima, e sem ela a cura seria *"anime sempre"*: um Show do papel
/// vivo tem de continuar barato (nenhuma transição, nenhum `Plan`). O que muda é que a máquina
/// **assume** o papel — senão o readout do painel acenderia o nome de onde ela saiu.
#[test]
fn showing_the_pose_the_scene_already_wears_is_free_and_still_lands() {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    m.advance(1.0);
    assert_eq!(m.current(), 1);

    m.go_to(1, 1.0, linear());
    assert!(!m.is_animating(), "animou para a pose em que ja estava");
    assert_eq!(m.current(), 1, "perdeu o papel vivo");

    // E um papel cuja pose é IDÊNTICA à viva também não anima — mas a máquina passa a nomeá-lo,
    // que é o que o painel lê.
    let mut m = Machine::new(vec![at(StateRole::Default, 0.0), at(StateRole::Hover, 0.0)])
        .expect("maquina");
    m.go_to(1, 1.0, linear());
    assert!(!m.is_animating(), "animou de x para x");
    assert_eq!(
        m.current(),
        1,
        "a maquina nao assumiu o papel que o artista pediu"
    );
}

/// **Re-alinhar à MESMA tabela não é uma mudança, e não pode matar um voo.**
///
/// ⚠️ A ponte re-alinha a cada pedido (o artista re-grava, e sem isso o Show seguinte animaria
/// para a pose antiga). Mas `retarget` abortava **incondicionalmente**, então cada clique em Show
/// destruía a transição em curso antes sequer de a examinar — o aborto que o gate acima
/// reproduz. Abortar é a resposta certa quando a tabela MUDOU; quando ela é a mesma, é trabalho
/// destruído por nada.
#[test]
fn re_aligning_to_an_unchanged_table_keeps_the_flight_alive() {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    m.advance(0.3);

    m.retarget(states());
    assert!(
        m.is_animating(),
        "re-alinhar a uma tabela IDENTICA abortou a transicao em curso"
    );
    m.advance(1.0);
    assert!((x(&m) - 10.0).abs() < 1e-9, "o voo nao chegou: {}", x(&m));
}

// ─────────────────────────────────────────────────────────────────────────────
// A MOLA (W7m) — uma OPÇÃO, e o easing fica intacto.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma máquina a caminho do hover **por CURVA** — a fixture do gate de não-regressão.
fn two_state_machine() -> Machine {
    let mut m = machine();
    m.go_to(1, 1.0, linear());
    m
}

/// ⭐⭐ **O caminho de CURVA é BYTE-IDÊNTICO com a mola no mundo.**
///
/// ⚠️ É a ordem do Enio num gate — *"não prejudique nada do sistema de easing"*. Ele compara a
/// trajetória inteira, quadro a quadro, contra os números que a máquina produz sem que nada de
/// mola exista no caminho; um gate que só olhasse o ENDPOINT ficaria verde sobre uma curva
/// deformada no meio, que é onde o artista de facto olha.
#[test]
fn the_curve_path_is_byte_identical_with_the_spring_in_the_world() {
    let mut m = two_state_machine();
    let mut curve = Vec::new();
    for _ in 0..20 {
        m.advance(1.0 / 60.0);
        curve.push(m.pose()[0].translation[0]);
    }
    // A MESMA máquina, o MESMO pedido — e a mola existe no binário, só não foi escolhida.
    let mut again = two_state_machine();
    for (i, want) in curve.iter().enumerate() {
        again.advance(1.0 / 60.0);
        let got = again.pose()[0].translation[0];
        assert!(
            (got - want).to_bits() == 0.0_f64.to_bits() || (got - want).abs() == 0.0,
            "o quadro {i} divergiu: {got} contra {want} — a mola tocou no caminho de curva"
        );
    }
}

/// ⭐ **A mola CHEGA à pose exata** — e é o `arrive` que a põe lá.
///
/// ⚠️ Uma mola converge assintoticamente: sem o critério de assentamento a máquina nunca chamaria
/// o `arrive`, a pose ficaria a um resíduo do alvo **para sempre**, e cada ida-e-volta deixaria
/// esse resíduo — depois de algumas dezenas de hovers o botão já não estaria onde o artista o
/// desenhou.
#[test]
fn a_spring_arrives_exactly_and_stops_animating() {
    let mut m = two_state_machine();
    m.go_to_spring(1, Spring::default());
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
        if !m.is_animating() {
            break;
        }
    }
    assert!(!m.is_animating(), "a mola nunca assentou");
    assert_eq!(
        m.pose()[0].translation[0],
        10.0,
        "ela parou perto do alvo em vez de NELE — o `arrive` nao correu"
    );
    assert_eq!(m.current(), 1);
}

/// ⭐⭐ **O que a mola COMPRA: reverter no meio CARREGA o MOMENTO.**
///
/// ⚠️ É a wave inteira, e o oráculo é o **SINAL do primeiro quadro**, não uma magnitude: com
/// momento o objeto **continua para onde ia** e só depois volta; sem ele, ele inverte na hora.
/// Uma curva não sabe fazer isto — ela recomeça em `t = 0`, e num `InOut` isso é velocidade
/// **zero** (medido: 0,00×, a cena para e arranca de novo).
///
/// ⚠️ **A primeira versão deste gate era VÁCUA e escondeu um defeito meu.** Ela comparava contra
/// uma máquina "fria" que ia de `x = 0` para o estado em `x = 0` — um caminho de comprimento
/// ZERO, que o atalho do `go_to` resolve sem voar. O controle media 0,000000 e `moved > 0` passava
/// sempre. Com ele vazio, a minha implementação reusava a MAGNITUDE da velocidade no eixo novo em
/// vez de a PROJETAR — o sinal saía invertido, e a reversão arrancava para trás mais depressa em
/// vez de carregar momento.
#[test]
fn reversing_a_spring_mid_flight_carries_its_momentum() {
    let mut m = machine();
    m.go_to_spring(1, Spring::default());
    for _ in 0..12 {
        m.advance(1.0 / 60.0);
    }
    assert!(m.is_animating(), "a fixture tem de reverter EM VOO");
    let before = x(&m);
    assert!(
        before > 0.5 && before < 9.5,
        "a fixture tem de reverter NO MEIO (x = {before})"
    );

    m.go_to_spring(0, Spring::default());
    m.advance(1.0 / 60.0);
    let after = x(&m);
    assert!(
        after > before,
        "o objeto inverteu na hora ({before:.4} -> {after:.4}) — sem momento, a mola e' uma \
         curva com outro nome"
    );

    // E ele CHEGA: o momento atrasa a volta, não a impede.
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
        if !m.is_animating() {
            break;
        }
    }
    assert!(!m.is_animating(), "a volta nunca assentou");
    assert_eq!(x(&m), 0.0, "ela nao pousou no alvo");
}

/// **A mola honra o recuo para o `Default`** — a lei que torna a lista de papéis opcional vale
/// para os dois motores, e ela é escrita UMA vez (`Machine::resolve`).
#[test]
fn the_spring_falls_back_to_default_like_the_curve_does() {
    let mut m = two_state_machine();
    // `Pressed` não está gravado; o recuo leva ao Default, que é o índice 0.
    m.go_to_spring(1, Spring::default());
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
    }
    m.go_to_role_spring(StateRole::Pressed, Spring::default());
    for _ in 0..600 {
        m.advance(1.0 / 60.0);
        if !m.is_animating() {
            break;
        }
    }
    assert_eq!(
        m.current(),
        0,
        "a mola nao recuou para o Default — um hospedeiro com mola ficaria preso onde um sem \
         ela nao fica"
    );
}
