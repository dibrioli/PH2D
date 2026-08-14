//! Os gates da integração cinemática (W-KinMove).
//!
//! ⚠️ Módulo FILHO por `#[path]`: é isso que mantém `use super::*` a alcançar o
//! que não é `pub`.
use super::*;

const UP: Vec2 = [0.0, 1.0];
const G: Vec2 = [0.0, -9.81];
const DT: f32 = 1.0 / 60.0;

fn still() -> KinematicState {
    KinematicState::default()
}

fn resting() -> KinematicState {
    KinematicState {
        grounded: true,
        ..KinematicState::default()
    }
}

/// Chão PLANO e PARADO — o que uma fixture que diz *"em repouso no chão"* tem
/// de fornecer.
///
/// ⚠️ **Ela nasceu da §8.3.** O `the_ground_absorbs_only_what_points_into_it`
/// passava `None` como amostra enquanto afirmava que o personagem estava no
/// chão, e isso só funcionava porque a absorção perguntava ao
/// [`KinematicState::grounded`] sozinho. Agora ela pede as DUAS respostas, e uma
/// fixture que declara chão sem o fornecer descreve *"a tocar numa parede"* —
/// que é precisamente o caso oposto.
fn flat() -> GroundSample {
    floor_moving([0.0, 0.0])
}

/// Chão PLANO que se move — a única fixture que estas leis precisam do sensor.
fn floor_moving(gv: Vec2) -> GroundSample {
    GroundSample {
        grip: 1.0,
        distance: 0.5,
        normal: [0.0, 1.0],
        ground_velocity: gv,
        one_way: false,
    }
}

/// **No ar, a gravidade é aplicada AQUI** — e é a assimetria central: o solver
/// não a aplica a um corpo cinemático.
#[test]
fn gravity_is_integrated_by_this_law_when_airborne() {
    let (st, wanted) = kinematic_advance(still(), Motor::default(), None, G, UP, DT, Fluid::DRY);
    assert!(
        (st.velocity[1] - G[1] * DT).abs() < 1e-6,
        "um tique de queda livre tem de dar {} e deu {}",
        G[1] * DT,
        st.velocity[1]
    );
    assert!(wanted[1] < 0.0, "e o deslocamento pedido aponta para baixo");
}

/// **No chão, a componente que aponta PARA o chão é absorvida** — e sem ela um
/// personagem parado numa rampa desliza (ver o aviso do módulo, com o número).
///
/// ⚠️ **A metade oposta é o pulo:** subir com o raio ainda a ver o chão é
/// exactamente o tique da decolagem, e zerar o eixo inteiro mataria o salto.
#[test]
fn the_ground_absorbs_only_what_points_into_it() {
    let ground = flat();
    let mut st = resting();
    for _ in 0..600 {
        st = kinematic_advance(st, Motor::default(), Some(&ground), G, UP, DT, Fluid::DRY).0;
    }
    assert_eq!(
        st.velocity[1], 0.0,
        "dez segundos parado no chao nao podem acumular queda"
    );

    let takeoff = KinematicState {
        velocity: [0.0, 5.0],
        grounded: true,
    };
    let (up_st, _) = kinematic_advance(
        takeoff,
        Motor::default(),
        Some(&ground),
        G,
        UP,
        DT,
        Fluid::DRY,
    );
    assert!(
        up_st.velocity[1] > 4.8,
        "o tique da decolagem ve' o chao e NAO pode ser zerado: {}",
        up_st.velocity[1]
    );
}

/// **E no AR ela não age** — o controle que separa *"pousei"* de *"a perna
/// alcança"*, e a razão de a régua do `grounded` ser corrigida sob Snap.
#[test]
fn nothing_is_absorbed_while_airborne() {
    let (st, _) = kinematic_advance(still(), Motor::default(), None, G, UP, DT, Fluid::DRY);
    assert!(
        (st.velocity[1] - G[1] * DT).abs() < 1.0e-6,
        "no ar a gravidade tem de sobreviver: {}",
        st.velocity[1]
    );
}

/// **O integrador paga o CONTATO e não o ATRITO** (K7) — e as duas metades são
/// o par que impede a plataforma de ser contada duas vezes.
///
/// ⚠️ A metade do VAGÃO nasceu deste gate ao contrário: ele afirmava que o
/// deslocamento *"tem de levar o vagão"*, o que é verdade sobre esta função
/// isolada e **falso sobre o produto** — a [`crate::walk`] já leva o personagem
/// ao referencial do chão pela tangente, e somar aqui dava 1,98× (a tabela está
/// no doc do [`ground_carry`]). Um gate que mede uma função sem a lei que a
/// alimenta pina a metade errada de um par.
///
/// A metade do ELEVADOR é a que sobrevive intacta: nenhuma tração empurra ao
/// longo da normal, então o contato é dívida do integrador — e ele a paga no
/// DESLOCAMENTO sem a escrever na velocidade guardada.
#[test]
fn the_integrator_owes_the_contact_and_not_the_traction() {
    // VAGÃO: velocidade tangente ao chão -- a caminhada é quem a paga.
    let wagon = floor_moving([3.0, 0.0]);
    let (st, wanted) = kinematic_advance(
        resting(),
        Motor::default(),
        Some(&wagon),
        G,
        UP,
        DT,
        Fluid::DRY,
    );
    assert!(
        wanted[0].abs() < 1e-9,
        "a tangente e' da caminhada; o integrador nao pode soma-la de novo: {}",
        wanted[0]
    );

    // ELEVADOR: velocidade ao longo da normal -- ninguem mais a paga.
    let lift = floor_moving([0.0, 3.0]);
    let (st_lift, w_lift) = kinematic_advance(
        resting(),
        Motor::default(),
        Some(&lift),
        G,
        UP,
        DT,
        Fluid::DRY,
    );
    assert!(
        (w_lift[1] - 3.0 * DT).abs() < 1e-6,
        "o contato tem de levantar o personagem: {}",
        w_lift[1]
    );
    assert_eq!(
        st.velocity[0], 0.0,
        "e a velocidade PROPRIA nao pode herdar o vagao"
    );
    assert!(
        st_lift.velocity[1] <= 0.0,
        "nem o elevador: a subida e' deslocamento deste tique, nao posse ({})",
        st_lift.velocity[1]
    );
}

/// **Contra uma parede, a velocidade própria PARA — e não inverte.**
///
/// ⚠️ Sem esta lei o personagem soma `+v` tique após tique e sai disparado no
/// instante em que a parede acaba; com uma subtração sem teto ele saltaria para
/// trás. O gate mede as duas.
#[test]
fn a_blocked_body_stops_and_does_not_bounce() {
    let st = KinematicState {
        velocity: [4.0, 0.0],
        grounded: true,
    };
    let wanted = [4.0 * DT, 0.0];
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], true, DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "encostado numa parede a velocidade propria tem de ir a ZERO"
    );

    // E um bloqueio maior que a velocidade não a inverte.
    let hard = kinematic_settle(st, [10.0 * DT, 0.0], [0.0, 0.0], true, DT);
    assert_eq!(hard.velocity[0], 0.0, "e nunca trocar de sinal");
}

/// **Um bloqueio que impede a PLATAFORMA não é uma velocidade minha a
/// corrigir** — o artefato que a regra ingênua produz.
///
/// ⚠️ Nasceu deste raciocínio e não de um relatório: parado sobre um vagão e
/// prensado contra uma parede, `v −= (pedido − efetivo)/dt` deixa
/// `v = −velocidade_do_vagão`, e o personagem **dispara para trás** ao sair
/// dele. É a mesma classe de defeito que o `−gv` de um `settle` cego produz em
/// qualquer controlador cinemático escrito sem o teste de sinal.
#[test]
fn a_platform_blocked_by_a_wall_does_not_owe_the_character_velocity() {
    // ⚠️ A fixture é um ELEVADOR e não um vagão desde que o `ground_carry`
    // passou a pagar só o contato: com a plataforma tangente o integrador não
    // pede deslocamento nenhum, e um bloqueio de zero não contém o fenômeno.
    let gv = [0.0, 3.0];
    let lift = floor_moving(gv);
    let (st, wanted) = kinematic_advance(
        resting(),
        Motor::default(),
        Some(&lift),
        G,
        UP,
        DT,
        Fluid::DRY,
    );
    assert!(
        wanted[1] > 0.0,
        "a fixture tem de CONTER o fenomeno: o pedido subiu {}",
        wanted[1]
    );
    // O teto impede tudo.
    let settled = kinematic_settle(st, wanted, [0.0, 0.0], true, DT);
    assert!(
        settled.velocity[1] <= 0.0,
        "prensado contra o teto, o elevador nao pode DEVER velocidade -- \
         a regra ingenua daria {} e deu {}",
        -gv[1],
        settled.velocity[1]
    );
}

/// **Deslizar não é ser bloqueado** — a componente onde a velocidade própria é
/// zero passa intacta.
///
/// ⚠️ Sem isto, uma rampa que converte queda em movimento lateral seria lida
/// como *"a parede me parou"* e o deslize morreria no primeiro tique.
#[test]
fn sliding_along_a_slope_is_not_absorbed() {
    let st = KinematicState {
        velocity: [0.0, -1.0],
        grounded: false,
    };
    let wanted = [0.0, -DT];
    // A rampa desviou metade da queda para o lado.
    let effective = [0.5 * DT, -0.5 * DT];
    let settled = kinematic_settle(st, wanted, effective, true, DT);
    assert_eq!(
        settled.velocity[0], 0.0,
        "o eixo em que ele nao empurrava nao pode ganhar correcao"
    );
    assert!(
        (settled.velocity[1] + 0.5).abs() < 1e-6,
        "e a queda tem de sobrar so' o que de facto aconteceu: {}",
        settled.velocity[1]
    );
}

/// **A porta da absorção — parada, e nos três casos que ela distingue.**
///
/// Ela é `pub` porque tem DOIS consumidores (o integrador e a ponte, que a chama
/// antes da lei); este gate pina o que ela responde a cada um.
///
/// ⚠️ **O piso passado é ZERO, e isso É a afirmação** (§8.1): com piso zero a
/// lei reduz LITERALMENTE à de antes dele — este gate é, hoje, o pino de que
/// chão plano não mudou um bit.
#[test]
fn the_supported_velocity_drops_only_what_the_ground_holds() {
    // No chão, a caminho do chão: some a componente ao longo de `up`.
    let held = supported_velocity([2.0, -5.0], true, UP, 0.0);
    assert!(
        (held[1]).abs() < 1e-6,
        "a queda tem de sair inteira: {held:?}"
    );
    assert!(
        (held[0] - 2.0).abs() < 1e-6,
        "e o eixo do chao passa intacto: {held:?}"
    );
    // No AR o valor é verbatim — é ali que a queda de facto acontece.
    assert_eq!(
        supported_velocity([2.0, -5.0], false, UP, 0.0),
        [2.0, -5.0],
        "no ar nada e' absorvido"
    );
    // A SAIR do chão (um pulo) também é verbatim, senão a decolagem morre.
    assert_eq!(
        supported_velocity([2.0, 7.0], true, UP, 0.0),
        [2.0, 7.0],
        "subir nao e' cair"
    );
}

/// **O PISO NUNCA É POSITIVO, e a SUBIDA fica byte-idêntica** (§8.1).
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE**: tirar o `min(0.0)` do
/// [`surface_descent`] passava na suíte inteira, e o doc ao lado dele afirmava
/// que sem ele o personagem *"seria lançado ladeira acima"*. Medido, ele não é
/// lançado — ele **sobe 1,2-1,7% mais rápido**, uma mudança de comportamento
/// pequena, plausível e **não medida por ninguém antes de ser pedida**.
///
/// O que o `min` entrega é a promessa da wave: *chão plano e SUBIDA não mudam
/// um bit*. É isso que este gate afirma, e é o que a mutação quebra.
#[test]
fn the_floor_never_lifts_and_a_climb_is_untouched() {
    // Uma rampa que SOBE para a direita: a tangente aponta para cima.
    let n = [-0.4226, 0.9063]; // ~25°
    for v in [[6.0_f32, 2.5], [0.5, 0.2], [0.0, 0.0], [-6.0, -2.5]] {
        let floor = surface_descent(v, n, UP);
        assert!(
            floor <= 0.0,
            "o piso da absorcao nunca pode ser positivo (v = {v:?} deu {floor})"
        );
    }
    // E numa superfície plana ele é ZERO EXATO — é este valor que faz a lei
    // reduzir, termo a termo, à de antes do piso.
    assert_eq!(
        surface_descent([9.0, -3.0], UP, UP),
        0.0,
        "chao plano tem de dar piso zero EXATO"
    );
}

/// **A ABSORÇÃO PEDE AS DUAS RESPOSTAS** (§8.3) — e a metade que faltava é a do
/// `footing`.
///
/// ⚠️ **O defeito que ele existe para pegar:** encostado numa superfície que a
/// lei RECUSOU por inclinação, o integrador diz *"toquei"* e a absorção comia a
/// gravidade inteira — o personagem ficava imóvel numa rampa de 60°, e o
/// `max_slope` que o artista escreve deixava de significar o que diz.
///
/// A fixture é o par mínimo que os distingue: **o mesmo estado**, a mesma
/// gravidade, e só a resposta do `footing` a mudar.
#[test]
fn nothing_is_absorbed_on_a_surface_the_law_refused() {
    let ground = flat();

    // CONTROLE: com chão a lei absorve, como sempre.
    let (on_floor, _) = kinematic_advance(
        resting(),
        Motor::default(),
        Some(&ground),
        G,
        UP,
        DT,
        Fluid::DRY,
    );
    assert_eq!(
        on_floor.velocity[1], 0.0,
        "o controle tem de absorver: com chão a queda sai inteira"
    );

    // E sem chão ACEITO — o `grounded` do integrador segue verdadeiro, porque
    // ele DE FACTO tocou — a gravidade tem de sobreviver, senão não sobra
    // deslocamento para o deslizamento do controlador redirecionar.
    let (steep, wanted) =
        kinematic_advance(resting(), Motor::default(), None, G, UP, DT, Fluid::DRY);
    assert!(
        (steep.velocity[1] - G[1] * DT).abs() < 1.0e-6,
        "tocar numa rampa recusada nao e' estar no chao: a queda tem de sobreviver ({})",
        steep.velocity[1]
    );
    assert!(
        wanted[1] < 0.0,
        "e o deslocamento pedido tem de apontar para baixo ({})",
        wanted[1]
    );
}

// ── A ÁGUA (W-KinFluid) ──────────────────────────────────────────────────────

/// Uma poça que carrega `s` pesos deste corpo, com resistência `d`.
fn water(s: f32, d: f32) -> Fluid {
    Fluid {
        buoyed: Buoyed(s),
        drag: d,
        push: [0.0, 0.0],
    }
}

/// Uma correnteza: o mesmo meio, com um empurrão de `a` m/s² em `+X`.
fn current(s: f32, d: f32, a: f32) -> Fluid {
    Fluid {
        push: [a, 0.0],
        ..water(s, d)
    }
}

/// **O AR SECO é o mundo de antes desta wave, AO BIT.**
///
/// ⚠️ O gate que carrega a rede de segurança inteira: `Fluid::DRY` tem
/// `gravity_share() == 1.0` e `drag == 0.0`, e `x * 1.0` é `x` em IEEE-754 — não
/// há aproximação a acumular numa cena sem poça. É por isto que os 139 gates que
/// já existiam continuaram verdes sem um oráculo tocado.
#[test]
fn dry_air_is_the_world_before_this_wave_to_the_bit() {
    let start = KinematicState {
        velocity: [3.0, -2.0],
        grounded: false,
    };
    let motor = Motor {
        accel: [1.0, 0.5],
        boost: [0.0, 0.25],
    };
    let (dry, w_dry) = kinematic_advance(start, motor, None, G, UP, DT, Fluid::DRY);
    // O mesmo passo, feito à mão sem termo de fluido nenhum.
    let want = [
        start.velocity[0] + (G[0] + motor.accel[0]) * DT + motor.boost[0],
        start.velocity[1] + (G[1] + motor.accel[1]) * DT + motor.boost[1],
    ];
    assert_eq!(
        dry.velocity, want,
        "seco, a lei tem de ser a expressão de antes termo a termo"
    );
    assert_eq!(w_dry, [want[0] * DT, want[1] * DT]);
    assert_eq!(Fluid::DRY.gravity_share(), 1.0);
    assert_eq!(Fluid::default(), Fluid::DRY);
}

/// **Boiar em equilíbrio é gravidade ZERO, e ir mais fundo é SUBIR.**
///
/// ⚠️ **A segunda metade é a que o clamp da consulta tornava inexprimível** — com
/// a razão capada em `1` o melhor que a lei conseguia era a linha do meio, e o
/// personagem ficava pendurado onde parasse. Um corpo que boia tem de acelerar
/// para CIMA quando está submerso, que é o que o corpo dinâmico faz na mesma
/// poça.
#[test]
fn the_fluid_scales_gravity_and_a_buoyant_body_rises() {
    let s = KinematicState::default();
    let dv = |f: Fluid| {
        kinematic_advance(s, Motor::default(), None, G, UP, DT, f)
            .0
            .velocity[1]
    };

    let dry = dv(Fluid::DRY);
    assert!(dry < 0.0, "seco ele cai ({dry})");

    let half = dv(water(0.5, 0.0));
    assert!(
        (half - dry * 0.5).abs() < 1.0e-6,
        "meia carga tem de dar meia queda: {half} contra {}",
        dry * 0.5
    );

    let afloat = dv(water(1.0, 0.0));
    assert_eq!(afloat, 0.0, "à tona o peso é inteiramente carregado");

    let cork = dv(water(4.0, 0.0));
    assert!(
        cork > 0.0 && (cork - (-dry * 3.0)).abs() < 1.0e-6,
        "uma rolha 4× menos densa SOBE a 3 g ({cork})"
    );
}

/// **O meio RESISTE, com a mesma aritmética do `effector::apply`.**
///
/// ⚠️ Sem ele o empuxo é uma mola sem amortecimento — medido no produto, o
/// personagem cinemático oscilava **2,90 m** de amplitude entre o 3.º e o 6.º
/// segundo com `AreaDrag 0`, contra **1,44** com o `0,6` da fixture (que é o
/// mesmo número que o corpo DINÂMICO faz: 1,4408 contra 1,4394).
#[test]
fn the_medium_resists_with_the_solvers_own_law() {
    let fast = KinematicState {
        velocity: [4.0, -6.0],
        grounded: false,
    };
    // Sem gravidade, para isolar o amortecimento do resto do passo.
    let g0 = [0.0, 0.0];
    let (wet, _) = kinematic_advance(fast, Motor::default(), None, g0, UP, DT, water(0.0, 3.0));
    let k = 1.0 / (1.0 + 3.0 * DT);
    assert!((wet.velocity[0] - fast.velocity[0] * k).abs() < 1.0e-6);
    assert!((wet.velocity[1] - fast.velocity[1] * k).abs() < 1.0e-6);
    // E os DOIS eixos, porque um meio não escolhe direção.
    assert!(
        wet.velocity[0].abs() < fast.velocity[0].abs()
            && wet.velocity[1].abs() < fast.velocity[1].abs(),
        "a resistência é isotrópica"
    );

    // CONTROLE: arrasto zero não toca um bit.
    let (dry, _) = kinematic_advance(fast, Motor::default(), None, g0, UP, DT, Fluid::DRY);
    assert_eq!(dry.velocity, fast.velocity);
}

/// **O motor NÃO é escalado pelo empuxo, e a ordem é o gate.**
///
/// ⚠️ O `motor.accel` traz o cancelamento de gravidade que a lei do pulo autorou,
/// e ele foi calculado contra a gravidade CHEIA. Escalar os dois juntos pagaria o
/// empuxo **duas vezes** num pulo dentro d'água — e o sintoma seria um personagem
/// que salta mais alto quanto mais fundo estiver, que é a amplificação
/// paramétrica que o W-Submerged existe para impedir.
#[test]
fn the_buoyancy_scales_gravity_and_leaves_the_motor_alone() {
    let s = KinematicState::default();
    // Um motor que cancela a gravidade exactamente, como a `gravity_hold` faz.
    let hold = Motor {
        accel: [0.0, -G[1]],
        boost: [0.0, 0.0],
    };
    let (v, _) = kinematic_advance(s, hold, None, G, UP, DT, water(1.0, 0.0));
    // Gravidade anulada pelo empuxo + o motor a somar `-G[1]` = só o motor sobra.
    assert!(
        (v.velocity[1] - (-G[1]) * DT).abs() < 1.0e-6,
        "o motor tem de sobreviver inteiro ao empuxo ({})",
        v.velocity[1]
    );
}

// ── O EMPURRÃO DA ZONA (W-ZoneForce) ─────────────────────────────────────────

/// **A correnteza acelera, e acelera pelo NÚMERO que recebeu.**
///
/// Ela chega como aceleração já dividida pela massa (a consulta o fez, com a massa
/// REAL do solver), então a lei só a soma — e o que ela soma tem de ser exatamente
/// `push · dt`, sem coeficiente escondido.
#[test]
fn a_current_accelerates_by_exactly_what_it_was_told() {
    let start = KinematicState {
        velocity: [0.0, 0.0],
        grounded: false,
    };
    let (dry, _) = kinematic_advance(start, Motor::default(), None, G, UP, DT, Fluid::DRY);
    let (wet, _) = kinematic_advance(
        start,
        Motor::default(),
        None,
        G,
        UP,
        DT,
        current(0.0, 0.0, 5.0),
    );
    assert_eq!(
        wet.velocity[0] - dry.velocity[0],
        5.0 * DT,
        "o empurrao entra como `a · dt` e nada mais"
    );
    assert_eq!(
        wet.velocity[1], dry.velocity[1],
        "e nao toca no eixo que ele nao empurra"
    );
}

/// **O empuxo NÃO escala o empurrão da zona.**
///
/// ⚠️ O gate que separa dois números que a mesma struct carrega: o
/// [`Fluid::gravity_share`] pesa a GRAVIDADE — o empuxo é uma força para cima
/// proporcional ao PESO —, e o empurrão de uma corrente não tem relação nenhuma com o
/// peso do corpo. Escalá-lo junto faria a correnteza afrouxar exatamente onde a água
/// carrega mais, que é o oposto do que a água faz.
///
/// A mutação que ele mata (`+ fluid.push[i]` para dentro do parêntese do
/// `gravity_share`) deixa TODOS os outros gates de fluido verdes.
#[test]
fn buoyancy_does_not_weigh_the_zones_push() {
    let start = KinematicState {
        velocity: [0.0, 0.0],
        grounded: false,
    };
    let mut seen = Vec::new();
    for lift in [0.0f32, 1.0, 4.0] {
        let (next, _) = kinematic_advance(
            start,
            Motor::default(),
            None,
            G,
            UP,
            DT,
            current(lift, 0.0, 5.0),
        );
        seen.push(next.velocity[0]);
    }
    assert_eq!(
        seen[0], seen[1],
        "o empurrao lateral e o mesmo boiando em equilibrio"
    );
    assert_eq!(
        seen[1], seen[2],
        "e o mesmo numa agua que o levanta quatro vezes o peso"
    );
    assert_eq!(seen[0], 5.0 * DT, "e vale o que a consulta disse");
}

/// **O MEIO resiste ao que a correnteza acabou de dar** — a ordem, não a soma.
///
/// O solver aplica o impulso da zona e o arrasto dela no MESMO passe, nessa ordem
/// (`effector::apply`), então a lei tem de fazer o mesmo: acelerar e só então frear.
/// Frear antes deixaria o primeiro tique de correnteza passar sem resistência nenhuma.
#[test]
fn the_medium_resists_what_the_current_just_gave() {
    let start = KinematicState {
        velocity: [0.0, 0.0],
        grounded: false,
    };
    let (next, _) = kinematic_advance(
        start,
        Motor::default(),
        None,
        [0.0, 0.0],
        UP,
        DT,
        current(0.0, 3.0, 5.0),
    );
    let want = (5.0 * DT) / (1.0 + 3.0 * DT);
    assert!(
        (next.velocity[0] - want).abs() < 1e-7,
        "acelera e SO ENTAO freia: {} contra {want}",
        next.velocity[0]
    );
}
