//! Gates do CREASE — o verbo cujo desenho inteiro é um expoente.
//!
//! Irmão do `verb_tests.rs`, e separado dele por ASSUNTO e não por tamanho:
//! estes dois gates carregam fixtures próprias e caras (`uv_sphere(256,512)`
//! para resolver a largura do sulco, `(128,192)` para a assimetria do expoente),
//! e nenhum outro verbo precisa delas.
//!
//! Filho do MESMO pai, então o `use super::*` alcança as fixtures
//! compartilhadas (`sphere`, `snapshot`) — o corte não duplica nada.

use super::*;

/// O Crease cava um sulco mais ESTREITO que um Draw invertido de mesmo alcance.
///
/// ⚠️ **O controle é o Draw invertido, e é ele que torna isto um oráculo.** Mesmo
/// `reach`, mesma curva de falloff, mesma lei de acúmulo: a ÚNICA coisa que pode
/// separá-los é o expoente que se está portando (`Crease.js:68`,
/// `Math.pow(fallOff, 5)`, sobre o coeficiente de `aNormal` e só sobre ele).
///
/// Mede **razão de afiação** — `(pico / largura a meia profundidade)` do Crease
/// dividido pelo do Draw. Um pico maior não é um vinco; um vinco é pico alto
/// sobre largura pequena.
///
/// ⚠️ **A fixture compartilhada `sphere()` (32×48) é CEGA aqui.** Com raio 0,3 o
/// primeiro anel cai em `r = 0,098` e a meia-largura curada em `0,078`: a
/// medição colapsa para zero e a razão explode. Precisa de `uv_sphere(256, 512)`.
///
/// ⚠️ **O dab é `[0, 0, 1]`, que é EQUATORIAL** — o eixo dos polos é **Y**
/// (`shapes.rs`), e pôr o dab no polo daria pico Z zero e um oráculo de lixo.
///
/// ⚠️ **`Falloff::Constant` é proibido aqui, e o motivo é o INVERSO do óbvio:**
/// com ele todo vértice do disco tem `depth == pico` nos DOIS mundos, a razão dá
/// 1,000 e o gate fica **VERMELHO mesmo com a cura** — não verde por vácuo.
#[test]
fn the_crease_cuts_a_narrower_groove_than_an_inverted_draw_of_the_same_reach() {
    let radius = 0.3f32;
    let profile = |verb: Verb, invert: bool| -> (f32, f32) {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(256, 512, 1.0);
        let base = snapshot(&mesh);
        let b = Brush {
            verb,
            radius,
            strength: 1.0,
            invert,
            pinch: 0.0,
            falloff: Falloff::Smooth,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], radius),
            Symmetry::default(),
        );

        // O dab está no eixo +Z: cavar é `z` diminuir, e a distância ao eixo do
        // sulco é o raio no plano XY.
        let mut samples: Vec<(f32, f32)> = Vec::new();
        for (i, p) in mesh.positions().iter().enumerate() {
            let depth = base[i][2] - p[2];
            if depth > 1e-6 {
                samples.push(((p[0] * p[0] + p[1] * p[1]).sqrt(), depth));
            }
        }
        let peak = samples.iter().map(|s| s.1).fold(0.0f32, f32::max);
        let half = peak * 0.5;
        let width = samples
            .iter()
            .filter(|s| s.1 >= half)
            .map(|s| s.0)
            .fold(0.0f32, f32::max);
        (peak, width / radius)
    };

    let (pc, wc) = profile(Verb::Crease, false);
    let (pd, wd) = profile(Verb::Draw, true);
    println!(
        "crease pico {pc:.5} largura {wc:.3} R · draw pico {pd:.5} largura {wd:.3} R · \
         estreitamento {:.3}x · profundidade {:.3}x",
        wd / wc,
        pc / pd
    );
    assert!(
        wc > 0.0 && wd > 0.0,
        "a fixture não resolve a largura — anéis de menos"
    );
    // **A LARGURA é a propriedade do verbo**, e é o `f⁵` que a produz.
    assert!(
        wd > wc * 1.5,
        "o sulco do Crease ({wc:.3} R) tinha de ser mais estreito que o do Draw \
         invertido ({wd:.3} R)"
    );
    // ⚠️ **E a PROFUNDIDADE é uma razão de CONSTANTES, não uma barra.** O
    // `Crease.js:48` usa `intensidade · 0,07 · raio` e o `Brush.js:60` usa
    // `intensidade · raio · 0,1`: o vinco é **0,70× mais raso que o Draw, de
    // propósito**, e afirmá-lo assim é o que impede este gate de precisar de
    // recalibração toda vez que um dos dois números se mexer.
    //
    // ⚠️ **Ele era uma barra ÚNICA** (`(pico ÷ largura) do crease sobre a do
    // draw > 1,5`) e ficou vermelha em **1,451** quando as duas constantes da
    // referência entraram — porque aquele número é o PRODUTO das duas
    // propriedades (2,08 × 0,70), e uma barra sobre um produto não diz qual das
    // metades se moveu.
    let want = crate::CREASE_FRACTION / crate::REACH_FRACTION;
    let got = pc / pd;
    assert!(
        (got / want - 1.0).abs() < 0.05,
        "a profundidade do Crease é {got:.3}x a do Draw, e as constantes da \
         referência pedem {want:.3}x"
    );
}

/// O expoente do Crease come a MÁSCARA e **não** come a força.
///
/// ⚠️ **Este gate existe por causa de uma cura de UMA LINHA que compila, fica
/// verde e está errada.** Elevar o `accum` (`self.accum[s].powi(4)`) em vez da
/// forma não toca a assinatura de ninguém — e passa em toda fixture desta crate,
/// porque todas usam `strength: 1.0`, onde `accum == shape` ao bit. O preço só
/// aparece fora desse ponto: com `strength 0.5` ela entrega **3,1%** do alcance
/// onde a referência entrega 50%, porque o expoente comeria a intensidade quatro
/// vezes a mais.
///
/// A assimetria é da referência, não nossa: no `Crease.js` o `pow` cai sobre
/// `curva × máscara × alpha` (linha 67 antes da 68) e a intensidade entra depois
/// (`brushFactor`), linear nos DOIS termos. Logo um vértice meio-mascarado leva
/// `0,5⁵` do empurrão e um pincel de meia força leva `0,5`.
#[test]
fn the_crease_exponent_eats_the_mask_but_not_the_strength() {
    let radius = 0.4f32;
    let peak = |strength: f32, mask: f32| -> f32 {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(128, 192, 1.0);
        if mask > 0.0 {
            mesh.masks_mut().fill(mask);
        }
        let base = snapshot(&mesh);
        let b = Brush {
            verb: Verb::Crease,
            radius,
            strength,
            pinch: 0.0,
            falloff: Falloff::Smooth,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], radius),
            Symmetry::default(),
        );
        mesh.positions()
            .iter()
            .enumerate()
            .map(|(i, p)| base[i][2] - p[2])
            .fold(0.0f32, f32::max)
    };

    let full = peak(1.0, 0.0);
    assert!(full > 1e-4, "a fixture não cavou nada");

    // FORÇA: linear. `0,5` de força ⇒ metade do sulco.
    let half_strength = peak(0.5, 0.0) / full;
    assert!(
        (half_strength - 0.5).abs() < 0.02,
        "meia força devia dar meio sulco e deu {half_strength:.4} — o expoente \
         está comendo a intensidade (a cura de uma linha)"
    );

    // MÁSCARA: quíntica. `keep = 0,5` ⇒ `0,5⁵ ≈ 3,1%`.
    let half_mask = peak(1.0, 0.5) / full;
    assert!(
        (half_mask - 0.5f32.powi(5)).abs() < 0.005,
        "meia máscara devia dar {:.4} e deu {half_mask:.4} — o `keep` saiu de \
         dentro do expoente",
        0.5f32.powi(5)
    );
}
