//! Os gates da resposta de contacto — a metade que faz a peça ROLAR (doc 109 §7).

use super::*;

/// Um disco de raio `0,5` pousado num chão horizontal: a normal é para cima e a alavanca da
/// tangente é `−R` (o ponto está debaixo do centro).
const RAIO: f32 = 0.5;
const BRACO_T: f32 = -RAIO;
/// `I = m·r²/2` ⇒ `invI = 2/r²` com massa 1, que é a que este nó usa.
const INV_I: f32 = 2.0 / (RAIO * RAIO);
const CIMA: [f32; 2] = [0.0, 1.0];

fn resposta(atrito: f32, rola: bool) -> Resposta {
    Resposta {
        salto: 0.0,
        atrito,
        // ⚠️ ZERO de propósito: estes gates são sobre o atrito TANGENCIAL, e um rolamento vivo
        // mexeria no `spin` que eles medem. Quem mede o rolamento é [`rolando`].
        rolar: 0.0,
        rolamento: rola.then_some((BRACO_T, INV_I)),
    }
}

/// A mesma resposta com o atrito de ROLAMENTO ligado (doc 109 §7.10).
fn rolando(atrito: f32, rolar: f32) -> Resposta {
    Resposta {
        rolar,
        ..resposta(atrito, true)
    }
}

/// ⭐⭐⭐ **A RESPOSTA AO REPORT** (*«os círculos não rotacionam com a colisão»*): um disco que
/// desliza para a DIREITA sobre um chão passa a girar no sentido dos ponteiros — e o controlo é a
/// MESMA chamada sem alavanca, que não gira nada.
#[test]
fn a_declared_disc_that_slides_gains_spin_and_a_point_does_not() {
    let (mut p, mut v) = ([0.0, 0.0], [1.0, -0.1]);
    let d_spin = respond(&mut p, &mut v, 0.0, CIMA, 0.0, &resposta(1.0, true));
    assert!(
        d_spin < -1.0,
        "a deslizar para a direita o disco tem de ganhar giro NEGATIVO, e ganhou {d_spin}"
    );
    assert!(v[0] < 1.0, "e a derrapagem tem de diminuir: {v:?}");

    let (mut p2, mut v2) = ([0.0, 0.0], [1.0, -0.1]);
    let sem = respond(&mut p2, &mut v2, 0.0, CIMA, 0.0, &resposta(1.0, false));
    assert_eq!(sem, 0.0, "um PONTO nao tem alavanca: nao roda");
}

/// ⛔ **UM PONTO FICA NA LEI DE SEMPRE, AO BIT** — o sangramento tangencial, escrito inline.
///
/// ⚠️ É o gate que impede a lei nova de escapar para toda cena que este nó já shipou: sem forma
/// declarada nada aqui pode mudar um bit.
#[test]
fn the_undeclared_path_is_the_tangential_bleed_verbatim() {
    for v0 in [[3.0_f32, -2.0_f32], [-1.5, -0.25], [0.0, -7.0]] {
        for atrito in [0.0_f32, 0.35, 1.0] {
            let (mut p, mut v) = ([0.0_f32, 0.0], v0);
            respond(&mut p, &mut v, 0.0, CIMA, 0.1, &resposta(atrito, false));
            // A lei de sempre, VERBATIM — e a forma importa: `vn_out·n + tangent·keep` não é
            // `out · keep`. Num `v` com `x` negativo e `atrito = 1` a segunda dá `−0,0` onde a
            // primeira dá `+0,0`, e isto é um gate de BITS.
            let (mut fp, mut fv) = ([0.0_f32, 0.0], v0);
            let n = CIMA;
            fp[0] += n[0] * 0.1;
            fp[1] += n[1] * 0.1;
            let vn = fv[0] * n[0] + fv[1] * n[1];
            let bounce = (1.0 + 0.0) * vn;
            let out = [fv[0] - bounce * n[0], fv[1] - bounce * n[1]];
            let vn_out = out[0] * n[0] + out[1] * n[1];
            let tangent = [out[0] - vn_out * n[0], out[1] - vn_out * n[1]];
            let keep = 1.0 - atrito;
            fv = [
                vn_out * n[0] + tangent[0] * keep,
                vn_out * n[1] + tangent[1] * keep,
            ];
            assert_eq!(
                (
                    p[0].to_bits(),
                    p[1].to_bits(),
                    v[0].to_bits(),
                    v[1].to_bits()
                ),
                (
                    fp[0].to_bits(),
                    fp[1].to_bits(),
                    fv[0].to_bits(),
                    fv[1].to_bits()
                ),
                "v0={v0:?} atrito={atrito}"
            );
        }
    }
}

/// ⭐⭐ **O GIRO AUTO-CORRIGE-SE** — o atrito lê a velocidade do PONTO (`v·t + ω·bt`), então uma
/// bola a girar depressa demais é travada pelo mesmo termo que a pôs a girar.
///
/// ⚠️ **Sem esse `ω·bt` ela acelerava para sempre**: o `angular_damping` do `sim.step` nasce em
/// `1` (sem arrasto), logo nada mais no quadro a travaria.
#[test]
fn a_disc_spinning_too_fast_is_slowed_by_the_same_friction() {
    // A girar muito mais depressa do que o rolamento pediria, com a mesma derrapagem.
    let (mut p, mut v) = ([0.0, 0.0], [1.0, -0.1]);
    let d_spin = respond(&mut p, &mut v, -400.0, CIMA, 0.0, &resposta(1.0, true));
    assert!(
        d_spin > 1.0,
        "um giro excessivo tem de ser TRAVADO (delta positivo), e leu {d_spin}"
    );
}

/// ⭐ **COULOMB: o atrito não pode passar de `μ · jn`** — é o que o torna física e não cola. Com o
/// impulso normal a valer `0,1` e `μ = 0,2`, o tecto é `0,02` por muito que a peça derrape.
#[test]
fn the_tangential_impulse_never_exceeds_the_coulomb_ceiling() {
    // ⚠️ `5` e não `50`: a diferença de dois números de módulo 50 em `f32` já vale `1e-6`, e a
    // barra mediria a subtracção em vez da lei. O pedido continua a ser 150× o tecto.
    let (mut p, mut v) = ([0.0, 0.0], [5.0, -0.05]);
    let antes = v[0];
    respond(&mut p, &mut v, 0.0, CIMA, 0.0, &resposta(0.2, true));
    let tirou = antes - v[0];
    // `jn = (1 + salto) · |vn| = 0,05`; o tecto e' `0,2 · 0,05 = 0,01`.
    assert!(
        (tirou - 0.01).abs() < 1e-5,
        "o tecto de Coulomb e' 0,01 e foi tirado {tirou}"
    );
}

/// **A QUEDA DE UMA BOLA SOBRE UM PLANO, integrada à mão** — a sonda das duas medições do §7.9.
///
/// ⚠️ As condições são as da tabela do [`ph2d_nodegraph::attr::BOUNCE_MAX`], de propósito:
/// gravidade `4`, `dt = 1/60`, largada de `0,75`, **sem laço** — só assim as duas tabelas se
/// comparam. O contacto é um PONTO (sem alavanca): o salto não depende dela, e assim a sonda mede
/// a lei do salto e nada mais.
fn pico_da_queda(salto: f32, segundos: f32) -> f32 {
    const G: f32 = 4.0;
    const DT: f32 = 1.0 / 60.0;
    let (mut p, mut v) = ([0.0_f32, 0.75], [0.0_f32, 0.0]);
    let mut pico = p[1];
    for _ in 0..((segundos / DT) as usize) {
        v[1] -= G * DT;
        p[0] += v[0] * DT;
        p[1] += v[1] * DT;
        if p[1] < 0.0 {
            let r = Resposta {
                salto,
                atrito: 0.0,
                rolar: 0.0,
                rolamento: None,
            };
            let fundo = -p[1];
            respond(&mut p, &mut v, 0.0, CIMA, fundo, &r);
        }
        pico = pico.max(p[1]);
    }
    pico
}

/// **SONDA — o que o `Bounce` do OBSTÁCULO faz acima de `1`** (doc 109 §7.9).
///
/// ```text
/// cargo test -p ph2d-node-sim-collide --lib probe_the_obstacle_bounce -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_the_obstacle_bounce_above_one() {
    eprintln!("\n  Bounce │ pico em 40 s (largada de 0,75, g=4, sem laço)");
    for salto in [0.0_f32, 0.5, 1.0, 1.25, 1.5, 2.0, 3.0] {
        let pico = pico_da_queda(salto, 40.0);
        eprintln!("  {salto:>6.2} │ {pico:>10.3}");
    }
}

/// ⭐ **E `μ = 0` é gelo: nem trava nem roda.**
#[test]
fn ice_neither_slows_nor_rolls() {
    let (mut p, mut v) = ([0.0, 0.0], [1.0, -0.1]);
    let d_spin = respond(&mut p, &mut v, 0.0, CIMA, 0.0, &resposta(0.0, true));
    assert_eq!(d_spin, 0.0);
    assert_eq!(v[0], 1.0, "gelo nao trava a derrapagem");
}

/// **UMA BOLA A ROLAR NUM CHÃO PLANO** — a sonda do tecto do rolamento (doc 109 §7.10).
///
/// ⚠️ Nas condições da cena `=115`: raio `0,2`, gravidade `4`, `dt = 1/60`, a partir de `1 u/s`
/// **já a rolar** (`ω = v/R`, que é onde o atrito tangencial deixa de ter deslize a opor — é
/// exactamente aí que a bola *«rola para sempre»*). Devolve os segundos até parar, ou `None`.
fn segundos_ate_parar(rolar: f32) -> Option<f32> {
    const G: f32 = 4.0;
    const DT: f32 = 1.0 / 60.0;
    const R: f32 = 0.2;
    let inv_i = 2.0 / (R * R);
    let (mut p, mut v) = ([0.0_f32, 0.0], [1.0_f32, 0.0]);
    let mut spin = -(v[0] / R) * GRAUS; // a rolar para a direita
    for k in 0..(60 * 30) {
        v[1] -= G * DT;
        p[0] += v[0] * DT;
        p[1] += v[1] * DT;
        if p[1] < 0.0 {
            let fundo = -p[1];
            let r = Resposta {
                salto: 0.0,
                atrito: 1.0,
                rolar,
                rolamento: Some((-R, inv_i)),
            };
            spin += respond(&mut p, &mut v, spin, CIMA, fundo, &r);
        }
        if v[0].abs() < 0.01 {
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de tiques pequena")]
            return Some(k as f32 * DT);
        }
    }
    None
}

/// **SONDA — quanto tempo uma bola leva a parar** (doc 109 §7.10, o tecto do `ROLLING_MAX`).
///
/// ```text
/// cargo test -p ph2d-node-sim-collide --lib probe_the_rolling_ball_stops -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda de medicao, nao um gate"]
fn probe_the_rolling_ball_stops() {
    eprintln!("\n  rolamento │ pára em (s)");
    for rolar in [
        0.0_f32, 0.02, 0.05, 0.1, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 8.0,
    ] {
        match segundos_ate_parar(rolar) {
            Some(s) => eprintln!("  {rolar:>9.2} │ {s:>8.2}"),
            None => eprintln!("  {rolar:>9.2} │      --- (rola para sempre)"),
        }
    }
}

/// ⭐⭐⭐ **A RESPOSTA À PONTA DO §7.7** (*«com `angular_damping = 1` ela rola para sempre num chão
/// infinito»*): com o número ligado a bola PÁRA sozinha, e o controlo é a mesma bola sem ele, que
/// ainda rola ao fim de 30 s.
///
/// ⚠️ **O obstáculo do arnês não declara rolamento nenhum** — e é esse o desenho: se este número se
/// combinasse por par como o atrito, ele daria `0` aqui e em toda cena que existe.
#[test]
fn a_rolling_ball_stops_by_itself_and_without_the_knob_it_never_does() {
    assert_eq!(
        segundos_ate_parar(0.0),
        None,
        "sem rolamento a bola tem de rolar para sempre -- e' a lei de antes desta coluna"
    );
    let com = segundos_ate_parar(0.1).expect("com rolamento ela pára");
    assert!(
        (3.0..4.5).contains(&com),
        "a `0,10` ela pára em ~3,7 s (tabela do `ROLLING_MAX`), e parou em {com}"
    );
}

/// ⛔ **O impulso é limitado PELA NORMAL, e nunca inverte a rotação** — as duas metades, porque só
/// a primeira distingue a lei de um `jr` sem tecto.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate afirmava só a segunda, e uma mutação SOBREVIVEU:** apagar o
/// `clamp` faz o impulso matar **todo** o giro num tique — e isso continua a nunca o inverter. *O
/// que o `clamp` compra não é o sinal, é o TECTO*: um toque de raspão não pode parar um pião.
#[test]
fn the_rolling_impulse_is_bounded_by_the_normal_and_never_reverses_the_spin() {
    // Um giro enorme contra um toque pequeno: `jn = 0,5`, `bt = −R`, `μr = 0,1`.
    let (mut p, mut v) = ([0.0, 0.0], [0.0_f32, -0.5]);
    let so_atrito = respond(&mut p, &mut v, -300.0, CIMA, 0.0, &resposta(1.0, true));
    let (mut p2, mut v2) = ([0.0, 0.0], [0.0_f32, -0.5]);
    let com = respond(&mut p2, &mut v2, -300.0, CIMA, 0.0, &rolando(1.0, 0.1));
    // O tecto, escrito à mão: `μr · jn · |bt| · invI`, em graus.
    let teto = 0.1 * 0.5 * BRACO_T.abs() * INV_I * GRAUS;
    assert!(
        ((com - so_atrito) - teto).abs() < 1e-3,
        "o rolamento tirou {} e o tecto pela normal é {teto}",
        com - so_atrito
    );
    assert!(
        (-300.0 + com).abs() > 100.0,
        "um toque de raspão não pode parar um pião: o giro ficou em {}",
        -300.0 + com
    );
    // E a outra metade: por muito alto que seja o número, ele só TRAVA.
    for rolar in [1.5_f32, 8.0, 1e6] {
        let (mut p, mut v) = ([0.0, 0.0], [0.0_f32, -0.5]);
        let antes = -300.0_f32;
        let d = respond(&mut p, &mut v, antes, CIMA, 0.0, &rolando(1.0, rolar));
        let depois = antes + d;
        assert!(
            depois * antes >= 0.0 && depois.abs() <= antes.abs() + 1e-3,
            "rolar={rolar}: o giro passou de {antes} para {depois} -- ele só pode ser TRAVADO"
        );
    }
}

/// ⭐⭐ **O rolamento lê o `ω` DEPOIS do tangencial** — os dois escrevem a mesma grandeza, e lidos do
/// mesmo `ω` este desfaria parte do giro que aquele acabou de dar.
///
/// ⚠️ A régua é um disco **a derrapar sem girar**: ali o `ω` de entrada é `0`, logo uma leitura
/// anterior ao atrito daria `jr = 0` e o rolamento seria **inerte neste tique** (`com == sem`). Com
/// a leitura certa ele come parte do giro acabado de nascer.
///
/// ⚠️ **`0,5` e não `1,0`, e a diferença ensina a lei:** a `1,0` o tecto do rolamento vale
/// exactamente o momento que o atrito produziu (`0,05` os dois), e o giro sai **zero ao bit** —
/// cancelamento total, que é legítimo e indistinguível de um `jr` a mais. A `0,5` a lei é parcial e
/// o gate mede uma FRACÇÃO, não um acidente de igualdade.
#[test]
fn the_rolling_reads_the_spin_the_friction_just_wrote() {
    let (mut p, mut v) = ([0.0, 0.0], [1.0, -0.1]);
    let sem = respond(&mut p, &mut v, 0.0, CIMA, 0.0, &resposta(1.0, true));
    let (mut p2, mut v2) = ([0.0, 0.0], [1.0, -0.1]);
    let com = respond(&mut p2, &mut v2, 0.0, CIMA, 0.0, &rolando(1.0, 0.5));
    assert!(sem < 0.0, "o controlo tem de girar: {sem}");
    assert!(
        com > sem && com < 0.0,
        "com rolamento o giro nascido neste tique tem de ser MENOR em módulo e do MESMO lado \
         ({com} contra {sem}) -- `com == sem` seria o rolamento a ler o `ω` de ENTRADA, que aqui \
         é zero"
    );
    assert!(
        (com - sem * 0.5).abs() < 1e-3,
        "e a fracção é metade do tecto: esperava {}, leu {com}",
        sem * 0.5
    );
}
