//! **O AGACHAR** (W15) — os gates do produto, pela porta do artista.
//!
//! A lei pura tem os dela em `ph2d-platformer::crouch`; aqui a pergunta é outra:
//! *o que o personagem de facto FAZ quando o botão de baixo é segurado num mundo
//! com rapier, um chão e uma marquise*.

#[path = "platform_crouch_rig.rs"]
mod rig_fixture;

use ph2d_physics_ecs::{InputTape, PlayerInput};
use rig_fixture::{
    BODY_HALF, CROUCH_HEIGHT, CROUCH_SPEED, FLOAT_HEIGHT, crouch_right, pose, rig, walk_right,
};

/// O fundo da marquise da fixture — ver o gate que a escolheu.
const OVERHANG_BOTTOM: f32 = 1.35;

/// **A silhueta baixa exactamente o que foi autorado** — e é isto que o agachar
/// É, num número.
///
/// ⚠️ **O collider NÃO muda de forma**, e este gate é a prova: o topo do corpo
/// desce pelo MESMO delta que a perna encurtou (`1,602 → 1,102`, autorado
/// `0,500`). Se o gesto reescrevesse a cápsula, o delta seria outro — e a
/// premissa que a W-Compound derrubou (*"um corpo tem exactamente um
/// collider"*) voltaria a ser perguntada.
///
/// ⚠️ **Mutação medida:** ignorar o agachar em `ride_for` deixa o topo em
/// **1,602** nos dois casos — o botão não faz nada.
#[test]
fn crouching_lowers_the_silhouette_by_the_authored_amount() {
    let mut up = rig(CROUCH_HEIGHT, None);
    up.run(0, 90, walk_right());
    let (_, uy) = pose(&up.sim);

    let mut low = rig(CROUCH_HEIGHT, None);
    low.run(0, 90, crouch_right());
    let (_, ly) = pose(&low.sim);

    let want = FLOAT_HEIGHT - CROUCH_HEIGHT;
    let got = uy - ly;
    assert!(
        (got - want).abs() < 0.02,
        "a silhueta baixou {got:.3} m para os {want:.3} autorados \
         (topo {:.3} -> {:.3})",
        uy + BODY_HALF,
        ly + BODY_HALF
    );
}

/// **Ele passa sob um teto que o para de pé** — o gesto inteiro, do ponto de
/// vista do artista.
///
/// ⚠️ **O CONTROLE são as duas outras células:** um teto ALTO deixa passar de pé
/// (senão o gate mediria *"a marquise para toda a gente"*), e o mesmo teto baixo
/// para quem não agacha. Medido (2026-08-05): de pé **x = 4,80** contra agachado
/// **x = 9,97**, e com o teto a `1,70` de pé chega a **29,77**.
///
/// ⚠️ O `4,80` não é um número arbitrário: a marquise começa em `5,0` e a cápsula
/// tem raio `0,2` — ele para exactamente encostado nela.
#[test]
fn he_fits_under_a_ceiling_that_stops_him_standing() {
    let travel = |bottom: f32, input: PlayerInput| {
        let mut r = rig(CROUCH_HEIGHT, Some(bottom));
        r.run(0, 300, input);
        pose(&r.sim).0
    };

    let standing = travel(OVERHANG_BOTTOM, walk_right());
    let crouched = travel(OVERHANG_BOTTOM, crouch_right());
    let high = travel(1.70, walk_right());

    assert!(
        standing < 5.5,
        "de pe' a marquise tinha de o parar na entrada: x={standing:.2}"
    );
    assert!(
        crouched > 9.0,
        "agachado ele tinha de PASSAR: x={crouched:.2}"
    );
    assert!(
        high > 20.0,
        "o CONTROLE falhou: sob um teto alto ele tem de passar de pe' \
         (x={high:.2}) -- senao este gate mede a marquise, nao o agachar"
    );
}

/// **Agachado ele anda mais devagar** — o segundo número do par autorado.
///
/// Medido (2026-08-05, 120 tiques): **11,765 m** de pé contra **3,973 m**
/// agachado, razão **0,338** — a razão das velocidades autoradas é `2/6 = 0,333`,
/// e a diferença é a rampa de aceleração.
#[test]
fn a_crouched_walk_is_slower() {
    let mut up = rig(CROUCH_HEIGHT, None);
    up.run(0, 120, walk_right());
    let (ux, _) = pose(&up.sim);

    let mut low = rig(CROUCH_HEIGHT, None);
    low.run(0, 120, crouch_right());
    let (lx, _) = pose(&low.sim);

    let want = CROUCH_SPEED / 6.0;
    let got = lx / ux;
    assert!(
        (got - want).abs() < 0.05,
        "razao {got:.3} para os {want:.3} autorados ({lx:.3} contra {ux:.3} m)"
    );
}

/// **Sob a marquise, soltar o botão NÃO o levanta** — e sair de baixo dela,
/// sim.
///
/// ⚠️ É o gate que torna verificável a razão de existir um [`CrouchState`]: o
/// estado não é uma função pura do botão.
///
/// ⚠️ **Mutação medida:** ignorar o sensor de teto (`stuck = false`) faz o corpo
/// subir para dentro da pedra e o solver empurra-o de volta — o centro sai de
/// **0,602** para **~0,85**, que é o topo encostado no fundo da laje.
#[test]
fn releasing_under_the_overhang_does_not_stand_him_up() {
    let mut r = rig(CROUCH_HEIGHT, Some(OVERHANG_BOTTOM));
    // Agacha e caminha até estar bem debaixo da marquise.
    let t = r.run(0, 260, crouch_right());
    let (x, y) = pose(&r.sim);
    assert!(
        x > 5.5,
        "a fixture tem de o pôr DEBAIXO da marquise: x={x:.2}"
    );
    assert!((y - CROUCH_HEIGHT).abs() < 0.05, "e agachado: y={y:.3}");

    // Solta o botão, ainda debaixo dela.
    let t = r.run(t, 60, walk_right());
    let (_, stuck) = pose(&r.sim);
    assert!(
        (stuck - CROUCH_HEIGHT).abs() < 0.05,
        "sob o teto ele tinha de continuar agachado: y={stuck:.3}"
    );

    // ⚠️ E o outro lado: fora da marquise ele levanta-se. Sem esta metade, um
    // agachar que ficasse preso para sempre passaria.
    let mut out = rig(CROUCH_HEIGHT, Some(OVERHANG_BOTTOM));
    let ot = out.run(0, 260, crouch_right());
    out.player_cfg(|p| p.crouch_height = 0.0);
    out.run(ot, 60, walk_right());
    let (_, freed) = pose(&out.sim);
    assert!(
        freed > CROUCH_HEIGHT + 0.2,
        "com a capacidade desligada ele tem de voltar a ficar de pe': y={freed:.3}"
    );
    let _ = t;
}

/// **A capacidade desligada é a cena de antes desta wave** — e o botão pode ser
/// martelado o run inteiro.
#[test]
fn with_the_capability_off_the_down_button_moves_nothing() {
    let mut held = rig(0.0, None);
    held.run(0, 120, crouch_right());
    let a = pose(&held.sim);

    let mut quiet = rig(0.0, None);
    quiet.run(0, 120, walk_right());
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

/// **O AGACHAR SOBREVIVE A UM SCRUB** — a prova de que o estado dele viaja no
/// ring da fita.
///
/// ⚠️ **É o gate que paga o `PlayerState`** (o mesmo raciocínio do irmão do
/// arranque): se o agachar morasse num segundo mapa da ponte, ele teria de ser
/// acrescentado àquele ring **à mão**, e esquecê-lo daria uma resposta que
/// depende de o cache ter o âncora.
///
/// ⚠️ **A cena tem de conter o fenómeno, e é a marquise que o contém:** só sob
/// um teto é que *lembrar-se de estar agachado* muda o resultado — em campo
/// aberto o botão solto levanta-o de qualquer maneira, e as duas versões
/// concordariam.
///
/// ⚠️ **E a comparação é feita DEPOIS de continuar dali, não no instante do
/// alvo** — a 1ª versão media a pose logo após o scrub e a mutação
/// **SOBREVIVEU**, pelo mesmo motivo estrutural que já tinha custado uma rodada
/// ao gate do arranque: no instante do alvo a pose vem do `restore` do rapier e
/// está certa em qualquer caso; o que a memória do controlador estraga é o que
/// vem A SEGUIR. Um gate de scrub que não CONTINUA não testa o ring, testa o
/// restore.
///
/// ⚠️ **Mutação medida:** semear só as metades do PULO e do ARRANQUE faz o
/// personagem levantar-se para dentro da laje nos tiques seguintes — o centro
/// sai de **0,602 para 0,850** (o topo encostado no fundo dela) e ele ainda
/// ganha **2,1 m** de avanço, porque de pé anda três vezes mais depressa.
#[test]
fn a_scrub_across_an_anchor_remembers_that_he_is_crouched() {
    /// Onde o botão é solto — já bem debaixo da marquise.
    const RELEASE: u64 = 260;
    /// O alvo do scrub, com pelo menos um âncora do ring entre ele e o aperto.
    const MID: u64 = 300;
    /// Quantos tiques correr DEPOIS do alvo — ver o aviso acima.
    const AFTER: u64 = 40;

    let tape = || {
        let mut t = InputTape::new();
        for k in 1..=400 {
            t.record(
                k,
                PlayerInput {
                    drive: 1.0,
                    down: k <= RELEASE,
                    ..PlayerInput::default()
                },
            );
        }
        t
    };

    let straight = {
        let mut r = rig(CROUCH_HEIGHT, Some(OVERHANG_BOTTOM));
        let mut tp = tape();
        for k in 1..=MID + AFTER {
            r.bridge.dispatch_with_tape(&mut r.sim, true, k, &mut tp);
        }
        pose(&r.sim)
    };
    assert!(
        (straight.1 - CROUCH_HEIGHT).abs() < 0.05,
        "a fixture tem de continuar AGACHADA e presa sob a laje: {straight:?}"
    );

    let mut r = rig(CROUCH_HEIGHT, Some(OVERHANG_BOTTOM));
    let mut tp = tape();
    for k in 1..=400 {
        r.bridge.dispatch_with_tape(&mut r.sim, true, k, &mut tp);
    }
    // Volta ao alvo...
    r.bridge.dispatch_with_tape(&mut r.sim, true, MID, &mut tp);
    // ...e SEGUE dali, que é onde a memória do controlador se faz sentir.
    for k in MID + 1..=MID + AFTER {
        r.bridge.dispatch_with_tape(&mut r.sim, true, k, &mut tp);
    }
    let scrubbed = pose(&r.sim);

    assert!(
        (straight.0 - scrubbed.0).abs() < 0.02 && (straight.1 - scrubbed.1).abs() < 0.02,
        "o scrub nao reproduziu o agachar: reto {straight:?} contra scrub {scrubbed:?}"
    );
}
