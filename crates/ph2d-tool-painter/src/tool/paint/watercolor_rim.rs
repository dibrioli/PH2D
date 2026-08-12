//! **O ARO VIRA A QUINA** — a distância à fronteira, para o aro da lavagem parar de sumir na axila
//! de dois traços que se cruzam (Enio, 2026-08-12, com foto; estudo em
//! [`docs/Painter/36_o_entalhe_no_cruzamento.md`]).
//!
//! ## O que estava errado, e é uma frase
//!
//! O aro é um **unsharp**: `edge = gain·(cw − inner)` com `inner = blur(hard)`. Um borrão linear
//! mede a **FRAÇÃO da vizinhança que está FORA**, e o aro quer a **DISTÂNCIA à fronteira**. Num
//! flanco reto as duas coincidem; numa quina elas divergem, e o sinal da divergência é geométrico:
//!
//! | | exterior que o disco vê | efeito |
//! |---|---|---|
//! | flanco reto | ~1/2 | o aro que o modelo calibra |
//! | quina **côncava**, por dentro | ~1/4 | aro **METADE** — ele some antes de virar a quina |
//! | quina côncava, por fora | ~3/4 de interior | lobo pálido **1,5×** — a franja come a tinta vizinha |
//!
//! Medido na cruz do report (`crossing_probe`): o ombro solitário na borda vale **0,624** e a axila
//! **0,282** — défice **0,34 ≈ 87/255**. É a cunha branca da foto.
//!
//! ## A cura, e por que ela é um TETO e não uma substituição
//!
//! `inner` passa a ser **limitado pelo que um flanco RETO daria à mesma distância**:
//!
//! ```text
//!   inner := min(blur(hard), P(sd, r))
//!   P(sd, r) = clamp((sd + r + 0.5) / (2r + 1), 0, 1)   // a resposta do box blur a um degrau
//! ```
//!
//! **Teto** (`min`) e não substituição porque a correção que a quina côncava precisa **só tem um
//! sinal**: ela precisa de MAIS aro e de MENOS franja, e as duas saem de um `inner` menor. Um `min`
//! nunca pode enfraquecer o aro em lugar nenhum — o pior que ele faz é não agir. `inner := P`
//! reescreveria todo flanco reto.
//!
//! ⚠️ **E o teto vale nos DOIS lados — a versão só-por-dentro foi construída, medida e é METADE da
//! cura.** O raciocínio que a defendia (*"por fora o teto mexeria no lobo pálido de TODA borda"*)
//! estava direcionalmente certo e **errado na magnitude**, e as duas metades da medição são estas:
//!
//! * **Só por dentro NÃO fecha a cunha.** O aro engrossa ao APROXIMAR-SE da quina (mapa: `44543` →
//!   `66542`) e o vão continua exatamente onde estava, porque o vão está FORA (`sd < 0`), onde o
//!   teto era inerte. Pior: fortalecer o aro ao lado de um vão intocado **aumenta o contraste** que
//!   faz a cunha ser vista.
//! * **Nos dois lados o vão FECHA** — no mapa da cruz, `2222` vira `333333` e o buraco de 5 px cai
//!   para 2 — e o preço num flanco RETO sobre tinta existente é **≤ 10/255 num único pixel** (o
//!   último do ombro), com os outros quinze byte-idênticos, e na direção certa (menos pálido).
//!
//! ⚠️ **Fora da lavagem o composite nem escreve**, então o lobo pálido só existe no OMBRO — é por
//! isso que o preço é de um pixel e não de uma banda. O EDGE-3 (Curtis §4.3.3) fica: o lobo
//! negativo continua lá, apenas deixa de ser amplificado por concavidade.
//!
//! ## A EDT é a que já existe
//!
//! [`super::sculpt_close::distance_inside`] é a EDT exata (Felzenszwalb-Huttenlocher, `O(área)`,
//! separável) que o fechamento do Inflate usa, e o doc dela diz que um segundo consumidor sempre foi
//! a intenção: *"duas cópias de uma EDT exata seriam duas respostas a que distância é esta"*. Esta é
//! a terceira, e ela não traz kernel novo.

use super::Region;
use super::sculpt_close::distance_inside;

/// O limiar que define a fronteira da lavagem: a meia-altura da cobertura endurecida.
const HALF: f32 = 0.5;

/// A distância ASSINADA (px) à fronteira `hard = 0.5`, positiva DENTRO. `None` quando a janela não
/// tem fronteira nenhuma (tudo dentro ou tudo fora) — ali não há aro a corrigir e o chamador pula o
/// teto inteiro em vez de carregar um campo constante.
pub(super) fn signed_distance(hard: &[f32], rw: usize, rh: usize) -> Option<Vec<f32>> {
    let n = rw * rh;
    if n == 0 || hard.len() < n {
        return None;
    }
    let mut inside = vec![0u8; n];
    let mut any_in = false;
    let mut any_out = false;
    for (m, &h) in inside.iter_mut().zip(hard.iter()) {
        if h >= HALF {
            *m = 255;
            any_in = true;
        } else {
            any_out = true;
        }
    }
    if !any_in || !any_out {
        return None;
    }
    let region = Region {
        x: 0,
        y: 0,
        w: rw as u32,
        h: rh as u32,
    };
    // UMA EDT, e o sinal vem da máscara.
    //
    // ⚠️ **A 1ª versão chamava a porta DUAS vezes** (distância de dentro ao fora, e a gêmea sobre o
    // complemento) — e medido contra o `box_blur` que o composite já paga isso custava **3,67
    // borrões**, no caminho quente. A pergunta certa é mais barata: a distância ao **CONJUNTO
    // FRONTEIRA** é UMA transformada, e de que lado o pixel está a máscara já responde de graça.
    // Medido depois da troca, costas-com-costas na mesma corrida: **1,69 borrões**.
    //
    // A semente é o pixel que TEM vizinho do outro lado (4-vizinhança): é a definição de fronteira
    // discreta, e é ela que faz `sd ≈ 0` na borda em vez de `±0,5` conforme o lado.
    let mut seed = vec![255u8; n];
    for y in 0..rh {
        for x in 0..rw {
            let i = y * rw + x;
            let me = inside[i] != 0;
            let border = (x > 0 && (inside[i - 1] != 0) != me)
                || (x + 1 < rw && (inside[i + 1] != 0) != me)
                || (y > 0 && (inside[i - rw] != 0) != me)
                || (y + 1 < rh && (inside[i + rw] != 0) != me);
            if border {
                seed[i] = 0;
            }
        }
    }
    let d = distance_inside(&seed, rw, region, 128);
    if d.len() < n {
        return None;
    }
    Some(
        (0..n)
            .map(|i| if inside[i] != 0 { d[i] } else { -d[i] })
            .collect(),
    )
}

/// **O TETO** — o `inner` que um flanco RETO daria a esta distância, para o raio de borrão `r`.
///
/// É a resposta analítica do box blur (lado `2r+1`) a um degrau: uma rampa linear de 0 em
/// `sd = −(r + 0.5)` a 1 em `sd = +(r + 0.5)`, valendo exatamente `0,5` na fronteira.
///
/// Devolve `1.0` (teto inerte) com `r = 0` — sem borrão não há o que limitar —, respondido aqui e
/// não no chamador, para o `min` do composite ser incondicional.
#[inline]
pub(super) fn straight_edge_cap(sd: f32, r: usize) -> f32 {
    if r == 0 {
        return 1.0;
    }
    let r = r as f32;
    ((sd + r + 0.5) / (2.0 * r + 1.0)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O teto vale meia-altura NA fronteira e satura a `r + 0.5` — os dois pontos que o tornam a
    /// resposta do box blur a um degrau, e não uma rampa escolhida.
    #[test]
    fn the_cap_is_the_box_blurs_own_straight_edge_ramp() {
        let r = 6usize;
        // NA fronteira o box blur ve metade do kernel dentro.
        assert!(
            (straight_edge_cap(0.0, r) - 0.5).abs() < 1e-6,
            "na fronteira: {}",
            straight_edge_cap(0.0, r)
        );
        // A rampa e' SIMETRICA em torno da fronteira — e' o mesmo degrau visto dos dois lados.
        for d in [1.0f32, 3.0, 5.0] {
            let a = straight_edge_cap(d, r);
            let b = straight_edge_cap(-d, r);
            assert!((a + b - 1.0).abs() < 1e-6, "assimetrica em {d}: {a} vs {b}");
        }
        // Satura em `±(r + 0.5)` e nunca sai de [0,1].
        assert!((straight_edge_cap(r as f32 + 0.5, r) - 1.0).abs() < 1e-6);
        assert!(straight_edge_cap(-(r as f32) - 0.5, r).abs() < 1e-6);
        assert!((straight_edge_cap(100.0, r) - 1.0).abs() < 1e-6);
        assert!(straight_edge_cap(-100.0, r).abs() < 1e-6);
        // `r = 0` nao tem borrao, logo nao tem teto.
        assert!((straight_edge_cap(3.0, 0) - 1.0).abs() < 1e-6);
        assert!((straight_edge_cap(-3.0, 0) - 1.0).abs() < 1e-6);
    }

    /// A distância assinada é positiva dentro, negativa fora, e ~0 na fronteira.
    #[test]
    fn the_signed_distance_knows_which_side_it_is_on() {
        let (rw, rh) = (32usize, 32usize);
        // Faixa horizontal: |y − 16| < 8.
        let hard: Vec<f32> = (0..rw * rh)
            .map(|i| {
                let y = (i / rw) as i32;
                if (y - 16).abs() < 8 { 1.0 } else { 0.0 }
            })
            .collect();
        let sd = signed_distance(&hard, rw, rh).expect("a faixa tem fronteira");
        let at = |x: usize, y: usize| sd[y * rw + x];
        assert!(at(16, 16) > 6.0, "o miolo esta fundo: {}", at(16, 16));
        assert!(at(16, 2) < -4.0, "fora e' negativo: {}", at(16, 2));
        assert!(at(16, 9).abs() < 2.0, "perto da borda: {}", at(16, 9));
    }

    /// Janela sem fronteira ⇒ `None`: o chamador pula o teto em vez de carregar um campo constante.
    #[test]
    fn a_window_with_no_boundary_has_no_cap() {
        assert!(signed_distance(&vec![1.0f32; 64], 8, 8).is_none());
        assert!(signed_distance(&vec![0.0f32; 64], 8, 8).is_none());
    }
}
