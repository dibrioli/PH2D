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
        rolamento: rola.then_some((BRACO_T, INV_I)),
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

/// ⭐ **E `μ = 0` é gelo: nem trava nem roda.**
#[test]
fn ice_neither_slows_nor_rolls() {
    let (mut p, mut v) = ([0.0, 0.0], [1.0, -0.1]);
    let d_spin = respond(&mut p, &mut v, 0.0, CIMA, 0.0, &resposta(0.0, true));
    assert_eq!(d_spin, 0.0);
    assert_eq!(v[0], 1.0, "gelo nao trava a derrapagem");
}
