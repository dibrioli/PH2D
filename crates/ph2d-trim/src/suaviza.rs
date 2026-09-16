//! **SUAVIZAR O TRAÇO** — o tremor da mão sai, a forma que ela desenhou fica.
//!
//! Ordem do dono (2026-09-15): *«O laço merece um grau de suavização do traço»*.
//!
//! # ⚠️ Porque NÃO é um laplaciano simples
//!
//! A média dos vizinhos aplicada a um anel **FECHADO** encolhe-o — é a mesma
//! difusão que arredonda uma malha, e num contorno de corte isso significa que o
//! artista desenha uma linha e a ferramenta corta **por dentro** dela. Medido
//! aqui (anel de `128` pontos, `64` pares de passagens): o laplaciano puro perde
//! **`14,28 %`** da área; a lei que fica perde **`0,00 %`**.
//!
//! ⭐ A lei é o par de passagens de **Taubin**: uma encolhe (`λ > 0`) e a
//! seguinte devolve (`μ < −λ`), e a composição das duas deixa as frequências
//! baixas — *a forma* — praticamente onde estavam, levando só as altas — *o
//! tremor*. ⛔ Não há aqui nenhuma constante de compensação a olho: o par
//! `(λ, μ)` **é** a compensação.

/// Quantos PARES de passagens o grau `1` gasta.
///
/// ⭐⭐⭐ **O número sai do JOELHO de uma curva medida, não de um gosto.**
/// Varrido sobre dois traços sintéticos — um círculo com tremor de `3 px` (o que
/// uma mão faz num arrasto rápido) e um quadrado **LIMPO**, cujos cantos são o
/// que esta lei pode destruir —, com o espaçamento dos pontos igual ao do
/// próprio gesto (`PASSO_MINIMO_PX = 4,0`):
///
/// | pares | tremor que sobra | a área moveu | canto perdido | CONTROLO: laplaciano puro |
/// |---|---|---|---|---|
/// | `1` | `63,3 %` | `0,05 %` | `0,71 px` | `0,21 %` |
/// | `8` | `38,5 %` | `0,04 %` | `1,61 px` | `1,93 %` |
/// | `32` | `27,9 %` | `0,02 %` | `2,45 px` | `7,43 %` |
/// | **`64`** | **`24,1 %`** | **`0,00 %`** | **`2,98 px`** | `14,29 %` |
/// | `128` | `20,6 %` | `0,02 %` | `3,59 px` | `26,54 %` |
/// | `192` | `19,1 %` | `0,03 %` | **`4,00 px`** | `37,04 %` |
///
/// ⇒ **o extremo tem régua própria e ela é do próprio gesto:** a `192` o canto
/// desloca-se **exactamente** o `PASSO_MINIMO_PX` — a distância com que o laço
/// guarda pontos —, logo dali para cima a lei apaga feição que o traço ainda
/// conseguia representar. E o ganho já achatou muito antes: de `32` para `128`
/// são `4 ×` o trabalho para tirar mais `7` pontos percentuais de tremor,
/// pagando `1,5 ×` o canto.
///
/// ⚠️ **A `64` o canto move-se `2,98 px`, três quartos dessa resolução** — e a
/// folga é de propósito: a medição corre no espaçamento MÍNIMO, e um arrasto
/// rápido guarda pontos mais afastados, onde a mesma contagem de passagens
/// alcança mais longe em pixels.
///
/// ⛔ **A coluna do CONTROLO é o que justifica a lei inteira:** o laplaciano
/// puro, o mesmo número de passagens, encolhe a área **`14,28 %`** contra
/// `0,00 %`. Num contorno de corte isso quer dizer cortar por DENTRO da linha
/// que o artista desenhou.
///
/// ⚠️ **O custo NÃO é o tecto:** `64` pares são `128` passagens sobre algumas
/// centenas de pontos, **uma vez por gesto** — invisível a qualquer escala que
/// uma mão produza.
pub const PARES_MAX: usize = 64;

/// O passo que encolhe.
const LAMBDA: f32 = 0.5;
/// O passo que devolve.
///
/// ⛔⛔⛔ **ELE ERA `−0,52` E A AFIRMAÇÃO AO LADO DELE ESTAVA ERRADA.** O doc
/// dizia *«mais negativo que `−λ` de propósito — com `μ = −λ` as duas passagens
/// cancelam-se e a lei vira a identidade»*, e **uma mutação SOBREVIVENTE** (pôr
/// `μ = −0,5`) mandou medir. As duas passagens **não** se cancelam: a segunda
/// mede a média sobre o anel **já suavizado**, e a composição de um par tem
/// resposta `H(k) = (1 − λk)(1 − μk)` com `k = 1 − cos θ ∈ [0, 2]`.
///
/// ⭐⭐⭐ **Com `μ = −λ` isso é `H = 1 − (λk)²`: monótona, com ganho `1` nas
/// frequências baixas e ZERO na mais alta** (a `λ = 0,5`, `H(2) = 0`) — isto é,
/// o tremor é aniquilado e a forma fica. Com `μ` **mais** negativo aparece um
/// termo linear positivo e a lei passa a **AMPLIFICAR** as harmónicas baixas,
/// que é o alisador a inventar forma que o artista não desenhou:
///
/// | `μ` | uma harmónica baixa (ordem 3) | tremor que sobra | a área moveu | canto perdido |
/// |---|---|---|---|---|
/// | **`−0,50`** | **`−0,2 %`** | **`2,3 %`** | **`0,006 %`** | `2,45 px` |
/// | `−0,505` | `+0,0 %` | `2,6 %` | `0,045 %` | `2,34 px` |
/// | `−0,52` | **`+0,6 %`** | `4,5 %` | `0,160 %` | `2,01 px` |
/// | `−0,55` | **`+1,8 %`** | `9,4 %` | `0,391 %` | `1,29 px` |
///
/// ⇒ `−0,50` ganha em **três** das quatro colunas, e a que perde — o canto —
/// continua muito abaixo da resolução do próprio gesto
/// (`2,45 px` contra os `4,0` com que o laço guarda pontos). *A escolha não é
/// um ponto de uma varredura: é o único valor em que a resposta é monótona.*
const MU: f32 = -LAMBDA;

/// **Suaviza um anel FECHADO de pontos de ecrã.**
///
/// `grau` em `0..=1`. ⭐ **`0` devolve o anel emprestado, byte-idêntico** — é a
/// lei desta casa para todo knob novo, e aqui ela é de graça: com `grau = 0` os
/// dois passos têm factor nulo e cada um é a identidade.
///
/// ⚠️ **O anel é FECHADO**: o vizinho do último é o primeiro. Tratá-lo como
/// aberto prenderia as duas pontas e deixaria uma bossa exactamente onde o
/// artista fechou o laço — o sítio onde a mão mais treme.
#[must_use]
pub fn suaviza(anel: &[[f32; 2]], grau: f32) -> std::borrow::Cow<'_, [[f32; 2]]> {
    let g = if grau.is_finite() {
        grau.clamp(0.0, 1.0)
    } else {
        0.0
    };
    // ⚠️ Menos de três pontos não é um anel; e um grau nulo é o pedido de nada.
    if g <= 0.0 || anel.len() < 3 {
        return std::borrow::Cow::Borrowed(anel);
    }
    let mut a = anel.to_vec();
    let mut b = a.clone();
    for _ in 0..PARES_MAX {
        passo(&a, &mut b, LAMBDA * g);
        passo(&b, &mut a, MU * g);
    }
    std::borrow::Cow::Owned(a)
}

/// Uma passagem: cada ponto anda `k` do caminho até à média dos vizinhos.
///
/// ⚠️ **Buffer DUPLO, nunca no sítio.** Um Gauss-Seidel leria o valor novo de um
/// vizinho e o velho do outro, e a saída passaria a depender de **onde o anel
/// começa** — que é uma propriedade do gesto, não da forma.
fn passo(de: &[[f32; 2]], para: &mut [[f32; 2]], k: f32) {
    let n = de.len();
    for i in 0..n {
        let (a, c) = (de[(i + n - 1) % n], de[(i + 1) % n]);
        let m = [(a[0] + c[0]) * 0.5, (a[1] + c[1]) * 0.5];
        para[i] = [
            de[i][0] + k * (m[0] - de[i][0]),
            de[i][1] + k * (m[1] - de[i][1]),
        ];
    }
}

#[cfg(test)]
#[path = "suaviza_tests.rs"]
mod tests;
