//! ⭐⭐⭐ **O MODO MISTO — cada junta defende o LADO que o artista lhe deu** (ordem do dono,
//! 2026-09-14: *«além de CCW e CW precisamos de um modo misto onde temos ossos com ângulos para os
//! dois lados, e cada osso mantém sua direção inicial»*).
//!
//! ⛔⛔ **Este modo NÃO usa o FABRIK, e a razão é medida.** Pôr o sinal de cada junta como
//! restrição dentro das varreduras do [`super::reach_fabrik`] — a forma clássica — **oscila**: a
//! ida prega a ponta no alvo e re-resolve a corrente inteira sem olhar aos sinais, a correcção
//! desfaz isso, e em dois dos doze alvos do corpus o erro da ponta **CRESCE** passagem a passagem
//! (`0,52 → 1,58` numa corrente de alcance `3`; `1,87 → 2,00` numa de `5`) — e os dois têm pose
//! exacta, achada por busca cega. ⚠️ Nem deitar a junta na fronteira nem espelhá-la cura, e o
//! amortecimento entre as duas foi varrido em `0,0 · 0,2 · 0,4 · 0,5 · 0,6 · 0,8 · 1,0` sem nenhum
//! valor resolver os dois: *não é afinação, é o laço*.
//!
//! ⭐⭐⭐ **A lei que fica é a DESCIDA JUNTA A JUNTA (CCD) com a parede de cada lado**, e ela é
//! melhor por uma propriedade e não por um número: rodar a cauda em torno de uma junta muda **essa
//! junta e mais nenhuma** (as de jusante viajam rigidamente, a de montante não se mexe), logo o
//! lado pedido vira um **intervalo fechado** naquele ângulo — e `φ = 0` está sempre dentro dele,
//! porque a pose de onde se parte é a autorada e é dela que os sinais foram lidos. ⇒ cada passo
//! **nunca piora** a ponta, e um laço que nunca piora não pode entrar em ciclo. Corpus inteiro:
//! `12` de `12` no alvo ao bit, com os `12` conjuntos de sinais intactos.
//!
//! ⚠️ **O resultado é uma função de `(raiz, comprimentos, alvo, POSE AUTORADA)`** — a mesma lei de
//! determinismo da W16, com a pose autorada no lugar do lado autorado. Quem a garante é o chamador,
//! que passa a pose do documento e não a do quadro anterior.

use std::f64::consts::PI;

/// ⭐ **A margem, em RADIANOS, com que uma junta corrigida nasce do lado certo.** Ela tem de ser
/// maior que o `STRAIGHT` com que [`super::reach_side::joint_signs`] lê uma junta como recta —
/// senão a correcção entregaria uma junta que a régua lê como **sem lado** — e pequena, porque a
/// parede que menos aperta é a que fica encostada à fronteira. `8e-3` ⇒ `0,46°`.
pub(crate) const MARGEM: f64 = 8.0 * super::reach::STRAIGHT;

/// Traz um ângulo para `(−π, π]`, que é onde o sinal de uma dobra é legível.
fn wrap(a: f64) -> f64 {
    let mut a = a % (2.0 * PI);
    if a <= -PI {
        a += 2.0 * PI;
    } else if a > PI {
        a -= 2.0 * PI;
    }
    a
}

/// O ângulo da junta `j` — do osso que ENTRA para o osso que SAI, com sinal.
fn angulo_da_junta(p: &[[f64; 2]], j: usize) -> f64 {
    let entra = (p[j][1] - p[j - 1][1]).atan2(p[j][0] - p[j - 1][0]);
    let sai = (p[j + 1][1] - p[j][1]).atan2(p[j + 1][0] - p[j][0]);
    wrap(sai - entra)
}

/// A distância da ponta ao alvo depois de rodar a cauda `φ` em torno de `piv`.
fn erro_apos(piv: [f64; 2], ponta: [f64; 2], alvo: [f64; 2], phi: f64) -> f64 {
    let (sen, cos) = phi.sin_cos();
    let v = [ponta[0] - piv[0], ponta[1] - piv[1]];
    let x = piv[0] + v[0] * cos - v[1] * sen;
    let y = piv[1] + v[0] * sen + v[1] * cos;
    (x - alvo[0]).hypot(y - alvo[1])
}

/// ⭐⭐⭐ **Resolve a corrente mantendo o sinal de cada junta.** `sinais` vem da pose AUTORADA (uma
/// entrada por junta interior); um `0.0` ali é uma junta que chegou recta e **não tem lado para
/// defender** — ela roda livre, porque impor-lhe um sinal seria inventar uma decisão a partir de
/// ruído de `f64`.
pub(crate) fn solve(
    joints: &mut [[f64; 2]],
    lengths: &[f64],
    alvo: [f64; 2],
    sinais: &[f64],
    passagens: usize,
    tolerancia: f64,
) {
    let ossos = lengths.len();
    for _ in 0..passagens {
        // ⭐⭐ **O teste de paragem vem ANTES da varredura, e isso é o que torna o modo um PONTO
        // FIXO.** Cada junta escolhe o `φ` que mais aproxima a ponta, então uma corrente que já
        // está no alvo ainda assim se mexeria — pouco, e todo quadro — se a varredura corresse
        // primeiro. ⚠️ Um rig parado a derivar de quadro para quadro é o defeito que a W16 curou no
        // lado autorado, e aqui ele voltaria por uma linha fora de ordem.
        let erro = (joints[ossos][0] - alvo[0]).hypot(joints[ossos][1] - alvo[1]);
        if erro < tolerancia {
            break;
        }
        for j in 0..ossos {
            let piv = joints[j];
            let ponta = joints[ossos];
            let braco = (ponta[0] - piv[0]).hypot(ponta[1] - piv[1]);
            let puxao = (alvo[0] - piv[0]).hypot(alvo[1] - piv[1]);
            if braco <= f64::EPSILON || puxao <= f64::EPSILON {
                continue;
            }
            let livre = wrap(
                (alvo[1] - piv[1]).atan2(alvo[0] - piv[0])
                    - (ponta[1] - piv[1]).atan2(ponta[0] - piv[0]),
            );
            // ⭐⭐ **A PAREDE desta junta.** Rodar aqui muda o ângulo desta junta e de mais nenhuma,
            // então o lado pedido é um intervalo em `φ` — e `φ = 0` está sempre dentro dele.
            let mut phi = livre;
            if j >= 1
                && let Some(&quero) = sinais.get(j - 1)
                && quero != 0.0
            {
                let actual = angulo_da_junta(joints, j);
                // ⚠️ **A margem vale nas DUAS pontas do intervalo.** Uma junta dobrada a `π` está
                // fechada sobre si mesma e o produto vectorial ali é ZERO — o lado deixa de ser
                // legível exactamente como acontece na recta. Medido: sem a margem de cima, um dos
                // doze alvos do corpus devolvia a corrente no alvo com `4` dos `5` sinais.
                let dobra = PI - MARGEM;
                let (lo, hi) = if quero > 0.0 {
                    (MARGEM - actual, dobra - actual)
                } else {
                    (-dobra - actual, -MARGEM - actual)
                };
                // ⚠️ **Escolher pelo ERRO, nunca por `clamp`.** O erro é uma sinusóide em `φ`: o
                // mínimo sobre um intervalo fechado está no interior (em `livre`) ou num extremo —
                // e quando não é o interior, é o extremo **angularmente** mais perto de `livre`,
                // que a ordem linear de um `clamp` troca sempre que os dois lados de `livre`
                // atravessam `±π`.
                //
                // ⭐ **Três candidatos chegam, e isso é uma PROVA e não uma aposta:** a pose de
                // partida é feita dos sinais que dela se leram, logo cada ângulo já está dentro da
                // sua parede e toda rotação o mantém lá ⇒ `lo ≥ MARGEM − π` e `hi ≤ π − 2·MARGEM`,
                // e o intervalo vive **inteiro** dentro de `(−π, π)`, onde o `livre` também vive.
                // ⚠️ Medido antes de o afirmar: `livre ± 2π` como candidato **nunca** venceu em
                // `900` fixturas — código defensivo sem consumidor é dívida, não segurança.
                //
                // ⚠️⚠️ **MUTAÇÃO SOBREVIVENTE, e fica registada:** trocar esta escolha por um
                // `livre.clamp(lo, hi)` não reprova gate nenhum. Ela **muda o passo** em `165` das
                // varreduras do corpus (o `clamp` escolhe o extremo errado: `−0,027` onde o mínimo
                // está em `+3,099`), e a descida absorve isso — a maior deterioração medida na pose
                // FINAL é `0,0003` numa corrente de alcance `4`, dentro da tolerância. ⛔ Fica assim
                // mesmo assim, e não por gosto: é este mínimo exacto que compra a propriedade pela
                // qual este solver substituiu o FABRIK — *um passo nunca piora* —, e um `clamp` que
                // escolhe o extremo errado devolve o ciclo pela porta dos fundos.
                phi = [lo, hi, livre]
                    .into_iter()
                    .filter(|c| (lo..=hi).contains(c))
                    .min_by(|a, b| {
                        erro_apos(piv, ponta, alvo, *a).total_cmp(&erro_apos(piv, ponta, alvo, *b))
                    })
                    .unwrap_or(0.0);
                if std::env::var_os("PH2D_TRACE").is_some() {
                    let lin = livre.clamp(lo, hi);
                    if (lin - phi).abs() > 1e-12 {
                        println!("DISCORDA junta {j}: clamp {lin} vs erro {phi}");
                    }
                }
            }
            let (sen, cos) = phi.sin_cos();
            for q in &mut joints[j + 1..] {
                let v = [q[0] - piv[0], q[1] - piv[1]];
                *q = [
                    piv[0] + v[0] * cos - v[1] * sen,
                    piv[1] + v[0] * sen + v[1] * cos,
                ];
            }
        }
    }
}
