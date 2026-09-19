//! ⭐⭐⭐ **OS GATES DA RAMPA DE PESO** — a legibilidade medida, com as duas rampas RECUSADAS
//! calculadas dentro do próprio gate.
//!
//! ⚠️ **Os controlos vivem aqui e não numa tabela de números:** uma barra escrita à mão envelhece no
//! dia em que alguém mexe nas paradas, e ninguém saberia dizer contra o quê ela foi calibrada. Aqui
//! a pergunta é *«ela é melhor do que as duas que recusámos?»*, e as duas são construídas à frente.

use super::{PARADAS, Rampa};
use ph2d_tokens::{ColorToken, Theme};

/// Quantas amostras a régua tira — `0,05` de peso por passo, que é a resolução a que um artista
/// lê um número numa tela.
const AMOSTRAS: usize = 20;

/// OKLab de um byte-trio, pela mesma porta que a rampa usa.
fn lab(c: [u8; 3]) -> [f64; 3] {
    let (l, ch, h) = ph2d_tokens::srgb_to_oklch(c[0], c[1], c[2]);
    let r = h.to_radians();
    [l, ch * r.cos(), ch * r.sin()]
}

fn de(a: [u8; 3], b: [u8; 3]) -> f64 {
    let (x, y) = (lab(a), lab(b));
    let d: [f64; 3] = std::array::from_fn(|k| x[k] - y[k]);
    d[2].mul_add(d[2], d[1].mul_add(d[1], d[0] * d[0])).sqrt()
}

/// Os `AMOSTRAS` passos adjacentes de uma rampa qualquer, em OKLab.
fn passos(f: impl Fn(f64) -> [u8; 3]) -> Vec<f64> {
    let cs: Vec<[u8; 3]> = (0..=AMOSTRAS)
        .map(|i| f(i as f64 / AMOSTRAS as f64))
        .collect();
    (0..AMOSTRAS).map(|i| de(cs[i], cs[i + 1])).collect()
}

/// A rampa do PRODUTO, em bytes.
fn produto(t: f64) -> [u8; 3] {
    let c = Rampa::nova().cor(t);
    let [r, g, b, _] = c.to_rgba8().to_u8_array();
    [r, g, b]
}

/// ⛔ **A RECUSADA nº 1: as cinco paradas espaçadas em `t`**, que é como a rampa é publicada.
fn ingenua(t: f64) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0) * 4.0;
    let i = (t.floor() as usize).min(3);
    let u = t - i as f64;
    std::array::from_fn(|k| {
        let (a, b) = (f64::from(PARADAS[i][k]), f64::from(PARADAS[i + 1][k]));
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "u esta' em 0..1 e os extremos sao bytes, logo a soma cabe em u8"
        )]
        {
            (a + (b - a) * u).round() as u8
        }
    })
}

/// ⛔⛔ **A RECUSADA nº 3: as MESMAS paradas em OKLab, espaçadas em `t`.**
///
/// ⚠️ **Ela não estava aqui e foi uma MUTAÇÃO SOBREVIVENTE que a trouxe:** trocar a colocação por
/// arco pelo espaçamento uniforme passava o gate do planalto, porque a barra *«metade do passo
/// médio»* apanha o porte em sRGB (`0,0082`) e **não** este (`0,0378` contra uma barra de `0,0359`,
/// `5 %` de folga). ⭐ O que os separa é a UNIFORMIDADE — `0,325` contra `0,651` —, e é por isso que
/// a colocação por arco tem de ser afirmada contra ESTA e não só contra as outras duas.
fn oklab_espacada_em_t(t: f64) -> [u8; 3] {
    let r = Rampa::nova();
    let t = t.clamp(0.0, 1.0) * 4.0;
    let i = (t.floor() as usize).min(3);
    let u = t - i as f64;
    let m: [f64; 3] = std::array::from_fn(|k| r.lab[i][k] + (r.lab[i + 1][k] - r.lab[i][k]) * u);
    ph2d_tokens::oklch_to_srgb(m[0], m[1].hypot(m[2]), m[2].atan2(m[1]).to_degrees())
}

/// ⛔ **A RECUSADA nº 2: a rampa de ontem** — dois tokens semânticos, misturados em bytes sRGB.
fn dois_tons(t: f64) -> [u8; 3] {
    let frio = ColorToken::Info.resolve(Theme::Forge);
    let quente = ColorToken::Danger.resolve(Theme::Forge);
    let t = t.clamp(0.0, 1.0);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "t esta' em 0..1 e os extremos sao bytes"
    )]
    let mix = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * t).round() as u8;
    [
        mix(frio.r, quente.r),
        mix(frio.g, quente.g),
        mix(frio.b, quente.b),
    ]
}

/// ⭐⭐⭐ **NENHUM PASSO DA RAMPA É UM PLANALTO** — e a barra é DERIVADA, não escolhida.
///
/// A barra é **metade do passo médio**: uma rampa perfeitamente uniforme daria o passo médio em
/// todo o lado, e ficar abaixo de metade dele é o que faz dois pesos vizinhos serem a mesma cor.
///
/// ⛔ **É exactamente aqui que o porte INGÉNUO reprova** (o verde puro é um planalto), e o gate
/// mede-o à frente para a barra não poder ser lida como confortável.
#[test]
fn nenhum_passo_da_rampa_de_peso_e_um_planalto() {
    let p = passos(produto);
    let media: f64 = p.iter().sum::<f64>() / p.len() as f64;
    let pior = p.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        pior >= 0.5 * media,
        "a rampa tem um planalto: pior passo {pior:.4} contra metade da media ({:.4})",
        0.5 * media
    );
    // ⭐ O CONTROLO: a rampa publicada — as mesmas cores, espaçadas em `t` — REPROVA nesta barra.
    // Sem ele, alguém leria a barra acima como uma formalidade.
    let q = passos(ingenua);
    let media_q: f64 = q.iter().sum::<f64>() / q.len() as f64;
    let pior_q = q.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        pior_q < 0.5 * media_q,
        "o porte ingenuo deixou de ter planalto ({pior_q:.4} contra {:.4}) — a barra deixou de \
         discriminar as duas parametrizacoes, e este gate passou a nao afirmar nada",
        0.5 * media_q
    );
}

/// ⭐⭐⭐ **A RAMPA BATE AS DUAS QUE ELA RECUSA, no número que decide.**
///
/// ⛔⛔ **O número que decide é o PIOR passo e não o caminho total** — uma rampa pode percorrer
/// muito espaço de cor e ainda assim ter dois pesos vizinhos indistinguíveis, que é precisamente o
/// defeito do porte ingénuo (caminho `4,7 ×` o da rampa de ontem e pior passo `0,63 ×`).
#[test]
fn a_rampa_de_peso_e_mais_legivel_que_as_duas_recusadas() {
    let pior = |f: &dyn Fn(f64) -> [u8; 3]| passos(f).iter().copied().fold(f64::INFINITY, f64::min);
    let (nosso, ing, dois) = (pior(&produto), pior(&ingenua), pior(&dois_tons));
    assert!(
        nosso > dois * 2.0,
        "a rampa nova nao e' o DOBRO de legivel que a de ontem: pior passo {nosso:.4} contra \
         {dois:.4} ({:.2}x)",
        nosso / dois
    );
    assert!(
        nosso > ing * 2.0,
        "a rampa nova nao bate o porte ingenuo: {nosso:.4} contra {ing:.4}"
    );
    // ⭐ E a TERCEIRA: as mesmas paradas em OKLab, espaçadas em `t`. Ela é a que o gate do planalto
    // deixa passar, e é esta linha que torna a colocação por ARCO uma propriedade e não um detalhe.
    let plana = pior(&oklab_espacada_em_t);
    assert!(
        nosso > plana * 1.25,
        "a colocacao por ARCO deixou de comprar legibilidade sobre o espacamento uniforme:          {nosso:.4} contra {plana:.4} ({:.2}x)",
        nosso / plana
    );
}

/// ⭐ **AS PONTAS SÃO AS PARADAS, e a ida-e-volta pela porta de cor não as move.**
///
/// ⚠️ **Ela prova que a porta ([`ph2d_tokens::srgb_to_oklch`] / [`ph2d_tokens::oklch_to_srgb`]) é
/// usada com a convenção certa** — um ângulo em radianos onde ela quer graus daria uma rampa
/// plausível e errada, e nenhuma das réguas acima o veria.
#[test]
fn as_pontas_da_rampa_sao_as_paradas_declaradas() {
    assert_eq!(
        produto(0.0),
        PARADAS[0],
        "o peso zero deixou de ser a 1.a parada"
    );
    assert_eq!(
        produto(1.0),
        PARADAS[4],
        "o peso um deixou de ser a ultima parada"
    );
    // Fora da faixa satura, e nao envolve.
    assert_eq!(produto(-3.0), PARADAS[0]);
    assert_eq!(produto(9.0), PARADAS[4]);
    // ⭐ E cada parada interior é ALCANÇADA — se a posição por arco se perdesse, o verde sumia.
    for parada in PARADAS {
        let achou = (0..=200).any(|i| de(produto(f64::from(i) / 200.0), parada) < 0.02);
        assert!(
            achou,
            "a parada {parada:?} nao aparece em sitio nenhum da rampa"
        );
    }
}
