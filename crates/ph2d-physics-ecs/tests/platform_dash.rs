//! **O ARRANQUE** (W14) — os gates do produto, pela porta do artista.
//!
//! A lei pura tem os dela em `ph2d-platformer::dash`; aqui a pergunta é outra:
//! *o que o personagem de facto FAZ quando o botão é apertado num mundo com
//! rapier, gravidade e um chão*.

#[path = "platform_dash_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::{InputTape, PlayerInput};
use rig_fixture::{DASH_SPEED, DASH_TIME, dash_right, pose, rig, walk_right};

/// Quantos tiques de 1/60 s o arranque desta fixture ocupa.
const DASH_TICKS: u64 = 9;

/// **O arranque cobre a distância AUTORADA** — `speed × time`, e é esse o número
/// que o artista julga.
///
/// ⚠️ **O CONTROLE é a mesma cena a andar**, e sem ele o gate não distinguiria
/// um arranque de uma caminhada com outro nome. Medido (2026-08-05): arranque
/// **2,700 m** contra **0,900 m** a andar, nos mesmos 9 tiques.
///
/// ⚠️ **Mutação medida:** somar o alvo em vez de o definir (`boost = target`, sem
/// subtrair a velocidade) faz o percurso passar a depender da velocidade com que
/// se chegou — a caminhada de 6 m/s soma-se ao arranque e ele cobre **3,6 m**
/// para os 2,7 autorados, ou seja o número da UI deixa de descrever o que
/// acontece.
#[test]
fn a_dash_covers_the_authored_distance() {
    let mut r = rig(DASH_SPEED, 0.9);
    let t = r.run(0, 40, walk_right());
    let (x0, _) = pose(&r.sim);
    r.run(t, DASH_TICKS, dash_right());
    let (x1, _) = pose(&r.sim);
    let went = x1 - x0;
    let want = DASH_SPEED * DASH_TIME;

    let mut c = rig(0.0, 0.9);
    let ct = c.run(0, 40, walk_right());
    let (cx0, _) = pose(&c.sim);
    c.run(ct, DASH_TICKS, walk_right());
    let (cx1, _) = pose(&c.sim);
    let walked = cx1 - cx0;

    assert!(
        (went - want).abs() < want * 0.05,
        "o arranque cobriu {went:.3} m para os {want:.3} autorados"
    );
    assert!(
        went > walked * 2.0,
        "isto nao e' um arranque, e' uma caminhada: {went:.3} contra {walked:.3}"
    );
}

/// **Um arranque no AR mantém a altura** — é o que o torna uma linha reta.
///
/// ⚠️ **A metade que este gate mede é SUB-TIQUE**, e vale dizê-lo: o boost já põe
/// a velocidade vertical relativa em zero no topo de cada tique, então o que
/// falta é o que a gravidade faz DENTRO dele. Medido (2026-08-05): **0,0023 m**
/// de desvio em 9 tiques, contra **0,1104 m** de queda livre nos mesmos — 48×.
///
/// ⚠️ **Mutação medida:** devolver `[0, 0]` no `accel` do `dash_burst` (e com ele
/// o `gravity_hold`) faz o desvio subir para **0,0222 m** — quase dez vezes. É o
/// defeito que a W11 mediu na rampa, aqui: a velocidade certa e o DESLOCAMENTO
/// errado.
#[test]
fn an_airborne_dash_holds_its_altitude() {
    let mut r = rig(DASH_SPEED, 6.0);
    let t = r.run(0, 20, walk_right());
    let (_, y0) = pose(&r.sim);
    r.run(t, DASH_TICKS, dash_right());
    let (_, y1) = pose(&r.sim);
    let drift = (y1 - y0).abs();
    let free_fall = 0.5 * 9.81 * (DASH_TICKS as f32 / 60.0).powi(2);
    assert!(
        drift < 0.005,
        "o arranque sagou {drift:.4} m (a queda livre seria {free_fall:.4})"
    );
}

/// **UM arranque por tempo-de-voo** — o segundo, ainda no ar, é recusado; o pé
/// no chão devolve-o.
///
/// ⚠️ A recuperação está em zero nesta fixture de propósito: com ela ligada, a
/// recusa teria duas causas possíveis e o gate não distinguiria a que ele existe
/// para medir.
///
/// ⚠️ **O oráculo é a DIFERENÇA contra um controle, e a primeira versão media o
/// deslocamento cru — VERDE-sobre-nada pelo motivo oposto ao habitual.** Depois
/// de um arranque o corpo **continua a 18 m/s** (nada lhe tira a velocidade; o
/// controle aéreo trava-a devagar), então a janela seguinte cobria **1,981 m**
/// de pura inércia — e o gate lia isso como *"o segundo arranque saiu"*. A
/// asserção certa compara a MESMA corrida com e sem o segundo aperto: o que
/// sobra é o efeito do botão, e nada mais.
#[test]
fn a_second_dash_in_the_same_flight_is_refused_and_the_ground_gives_it_back() {
    /// Corre o roteiro e devolve `(quanto andou na 2ª janela — no AR, quanto
    /// andou na 3ª — já no CHÃO)`.
    fn run(second_press: bool, third_press: bool) -> (f32, f32) {
        let mut r = rig(DASH_SPEED, 6.0);
        r.player_cfg(|p| p.dash_cooldown = 0.0);
        // O primeiro arranque, no ar, e depois o botão SOLTO — sem borda não há
        // segundo aperto para recusar.
        let t = r.run(0, 10, walk_right());
        let t = r.run(t, DASH_TICKS, dash_right());
        let t = r.run(t, 4, walk_right());
        // ── A 2ª janela: ainda no ar ──
        let (x0, _) = pose(&r.sim);
        let hold = if second_press {
            dash_right()
        } else {
            walk_right()
        };
        let t = r.run(t, DASH_TICKS, hold);
        let (x1, _) = pose(&r.sim);
        // Aterra e assenta.
        let t = r.run(t, 200, walk_right());
        let (_, y) = pose(&r.sim);
        assert!(y < 1.5, "a fixture tem de ter POUSADO: {y:.3}");
        // ── A 3ª janela: já no chão ──
        let (x2, _) = pose(&r.sim);
        let hold = if third_press {
            dash_right()
        } else {
            walk_right()
        };
        r.run(t, DASH_TICKS, hold);
        let (x3, _) = pose(&r.sim);
        (x1 - x0, x3 - x2)
    }

    let (air_press, _) = run(true, false);
    let (air_quiet, ground_quiet) = run(false, false);
    let (_, ground_press) = run(false, true);

    assert!(
        (air_press - air_quiet).abs() < 0.05,
        "o segundo arranque no AR tinha de ser recusado: {air_press:.3} contra \
         {air_quiet:.3} sem apertar"
    );
    assert!(
        ground_press - ground_quiet > DASH_SPEED * DASH_TIME * 0.5,
        "depois de tocar o chao o botao tinha de valer outra vez: {ground_press:.3} \
         contra {ground_quiet:.3} sem apertar"
    );
}

/// **O ARRANQUE SOBREVIVE A UM SCRUB** — a prova de que o estado dele viaja no
/// ring da fita.
///
/// ⚠️ **É o gate que torna verificável a razão de o `PlayerState` existir.** Se o
/// arranque morasse num segundo mapa da ponte, ele teria de ser acrescentado
/// àquele ring **à mão** — e esquecê-lo daria uma resposta que depende de o
/// cache ter o âncora: sem erro, sem aviso, e visível só como *"o arranque some
/// quando arrasto a régua"*.
///
/// ⚠️ **A fixture custou DUAS correções, e as duas são a mesma doença:** a
/// mutação era invisível porque o estado que ela largava era, por acidente da
/// cena, IGUAL ao que ela mantinha.
///
/// 1. **O alvo caía antes do arranque.** Com o aperto no tique 41 e o alvo no
///    46, a ÂNCORA do ring (a cada `STRIDE = 10` tiques) cai no 40 — antes de
///    tudo —, e ali o estado do arranque é o default de qualquer maneira. O
///    gate provava só que o replay corre.
/// 2. **A corrida não ia longe o bastante para POUSAR.** O que distingue as
///    duas versões é uma coisa só, a CARGA: o seed correcto lembra-se de que
///    ela foi gasta no ar, e o errado deixa o valor de agora. Correndo só até o
///    tique 140 o personagem ainda estava a cair (o próprio arranque mata a
///    velocidade vertical e atrasa a queda), então "agora" também dizia
///    *gasta* — os dois concordavam, e a mutação passava. Correndo até pousar,
///    "agora" diz *cheia*, e aí divergem.
///
/// ⚠️ **Mutação medida:** semear só a metade do PULO faz o scrub divergir em
/// **0,80 m** — o arranque fantasma que a carga ressuscitada dispara no tique
/// 72.
#[test]
fn a_scrub_across_an_anchor_remembers_that_the_dash_was_spent() {
    /// O aperto que GASTA a carga, no ar.
    const SPEND: u64 = 38;
    /// O segundo aperto, que tem de ser recusado.
    const RETRY: u64 = 72;
    /// O alvo do scrub — depois do segundo aperto, e com uma âncora do ring
    /// entre ele e o primeiro.
    const MID: u64 = 75;

    let tape = || {
        let mut t = InputTape::new();
        for k in 1..=200 {
            t.record(
                k,
                PlayerInput {
                    drive: 1.0,
                    // ⚠️ Um tique cada, e não uma janela: a lei deriva a BORDA,
                    // e um botão segurado não encadeia arranques.
                    dash: k == SPEND || k == RETRY,
                    ..PlayerInput::default()
                },
            );
        }
        t
    };

    // ⚠️ Alto o bastante para nunca pousar dentro da janela medida: o pé no chão
    // REPÕE a carga, e com ele o segundo aperto seria legítimo — o gate mediria
    // outra coisa.
    let start_y = 20.0;

    let straight = {
        let mut r = rig(DASH_SPEED, start_y);
        let mut tp = tape();
        for k in 1..=MID {
            r.bridge.dispatch_with_tape(&mut r.sim, true, k, &mut tp);
        }
        pose(&r.sim)
    };

    // A mesma corrida, mas passando muito à frente e voltando.
    let mut r = rig(DASH_SPEED, start_y);
    let mut tp = tape();
    for k in 1..=220 {
        r.bridge.dispatch_with_tape(&mut r.sim, true, k, &mut tp);
    }
    r.bridge.dispatch_with_tape(&mut r.sim, true, MID, &mut tp);
    let scrubbed = pose(&r.sim);

    assert!(
        (straight.0 - scrubbed.0).abs() < 0.01 && (straight.1 - scrubbed.1).abs() < 0.01,
        "o scrub nao reproduziu o arranque: reto {straight:?} contra scrub {scrubbed:?}"
    );
}

/// **A capacidade desligada é a cena de antes desta wave** — e o botão pode ser
/// martelado o run inteiro.
///
/// ⚠️ É a prova de PRODUTO do opt-in, e ela é mais forte do que a da lei: ali o
/// motor era comparado num tique, aqui é a TRAJECTÓRIA inteira contra a de quem
/// nunca tocou no botão.
#[test]
fn with_the_capability_off_the_button_moves_nothing() {
    let mut pressed = rig(0.0, 0.9);
    pressed.run(0, 90, dash_right());
    let a = pose(&pressed.sim);

    let mut quiet = rig(0.0, 0.9);
    quiet.run(0, 90, walk_right());
    let b = pose(&quiet.sim);

    assert_eq!(
        a, b,
        "com a capacidade desligada o botao nao pode mover um bit"
    );
    assert!(
        a.0 > 1.0,
        "e a fixture tem de ter ANDADO, senao compara dois parados"
    );
}
