//! Os gates da lei do brilho.

use super::*;

/// ⭐⭐⭐ **A OMISSÃO É A IDENTIDADE AO BIT** — as duas metades, porque as causas são diferentes.
///
/// Uma é o interruptor (o passe devolve cedo), a outra é a intensidade a zero com ele LIGADO.
/// *Sem a segunda, um `intensity = 0` podia somar um `NaN` vindo de um halo mal formado e nada
/// nesta suíte o via.*
#[test]
fn a_omissao_e_a_identidade_ao_bit() {
    let base: Vec<[f32; 3]> = (0..64 * 64)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let f = i as f32;
            [f * 0.001, 2.0 - f * 0.0005, 0.25]
        })
        .collect();

    for (rot, b) in [
        ("desligado", Bloom::default()),
        (
            "ligado e mudo",
            Bloom {
                enabled: true,
                intensity: 0.0,
                ..Bloom::default()
            },
        ),
    ] {
        let mut q = base.clone();
        apply(&mut q, 64, 64, &b);
        for (i, (a, c)) in base.iter().zip(&q).enumerate() {
            assert_eq!(
                a.map(f32::to_bits),
                c.map(f32::to_bits),
                "{rot}: o pixel {i} mudou"
            );
        }
    }
}

/// ⭐⭐ **O JOELHO A ZERO É O CORTE DURO, EXACTO** — é isto que faz dele um controlo e não uma
/// aproximação permanente.
#[test]
fn o_joelho_a_zero_e_o_corte_duro_exacto() {
    let b = Bloom {
        knee: 0.0,
        threshold: 1.0,
        ..Bloom::default()
    };
    for i in 0..400 {
        #[allow(clippy::cast_precision_loss)]
        let l = i as f32 * 0.01;
        let saiu = bright([l, l * 0.5, l * 0.25], &b);
        let esperado = (l - 1.0).max(0.0);
        if l > 0.0 {
            let w = esperado / l;
            assert_eq!(
                saiu[0].to_bits(),
                (l * w).to_bits(),
                "l = {l}: o corte duro não é exacto"
            );
        } else {
            assert_eq!(saiu, [0.0; 3], "l = 0 tem de dar zero");
        }
    }
}

/// ⭐ **O CORTE preserva a MATIZ** — o que brilha é a cor do pixel, não o canal que passou.
///
/// ⚠️ Um vermelho saturado a `4,0` tem de brilhar tanto como um branco a `4,0`: a luminância é a de
/// PICO, e uma média ponderada leria `0,8` no vermelho.
#[test]
fn o_corte_preserva_a_matiz_e_usa_o_pico() {
    let b = Bloom {
        threshold: 1.0,
        knee: 0.0,
        ..Bloom::default()
    };
    let vermelho = bright([4.0, 0.0, 0.0], &b);
    let branco = bright([4.0, 4.0, 4.0], &b);
    assert!(vermelho[0] > 0.0, "um vermelho forte tem de brilhar");
    assert_eq!(
        vermelho[0].to_bits(),
        branco[0].to_bits(),
        "o pico decide: os dois têm o mesmo canal vermelho"
    );
    assert_eq!(vermelho[1], 0.0, "a matiz é a do pixel");
}

/// ⭐⭐⭐ **O RAIO DO HALO DOBRA POR NÍVEL** — a geometria portada do oráculo (`docs/Render3d/12` §3.4).
///
/// A régua é a **largura a meia altura** de um quadrado aceso sobre fundo preto, com **um** nível
/// aceso de cada vez. ⚠️ Ela mede o PRODUTO (o `apply`) e não o `downsample` — *uma régua sobre a
/// peça interna não afirma nada sobre a cadeia que o quadro corre*.
#[test]
fn o_raio_do_halo_dobra_por_nivel() {
    const W: usize = 256;
    let mut meias = Vec::new();
    for k in 0..4 {
        let mut niveis = [0.0f32; Bloom::LEVELS];
        niveis[k] = 1.0;
        let b = Bloom {
            enabled: true,
            threshold: 1.0,
            knee: 0.0,
            intensity: 1.0,
            levels: niveis,
        };
        let mut q = vec![[0.0f32; 3]; W * W];
        for y in W / 2 - 8..W / 2 + 8 {
            for x in W / 2 - 8..W / 2 + 8 {
                q[y * W + x] = [8.0; 3];
            }
        }
        apply(&mut q, W, W, &b);
        let linha: Vec<f32> = (W / 2 + 8..W).map(|x| q[(W / 2) * W + x][0]).collect();
        let pico = linha[0];
        assert!(pico > 0.0, "nível {k}: não há halo nenhum");
        let meia = linha.iter().take_while(|v| **v >= pico * 0.5).count();
        meias.push(meia);
    }
    for k in 1..meias.len() {
        #[allow(clippy::cast_precision_loss)]
        let razao = meias[k] as f32 / meias[k - 1] as f32;
        assert!(
            (1.5..=3.0).contains(&razao),
            "do nível {} para o {k} o raio a meia altura fez {razao:.2}× \
             (medido no oráculo: ~2×). meias = {meias:?}",
            k - 1
        );
    }
}

/// ⛔ **A CADEIA É DERIVADA DA RESOLUÇÃO** — um nível cujo borrão não cabe no quadro mede a
/// moldura, e é por isso que o oráculo acende três dos sete.
#[test]
fn a_cadeia_e_derivada_da_resolucao() {
    assert_eq!(levels_that_fit(0, 0), 0, "sem quadro não há cadeia");
    assert_eq!(levels_that_fit(8, 8), 1, "8 px dá um degrau");
    assert!(
        levels_that_fit(1920, 1080) >= 6,
        "um quadro de produto tem de ter pelo menos seis degraus, e tem {}",
        levels_that_fit(1920, 1080)
    );
    assert!(
        levels_that_fit(100_000, 100_000) <= Bloom::LEVELS,
        "a cadeia nunca passa do que a struct declara"
    );
}

/// ⭐⭐ **UM NÍVEL MUDO NÃO É UM NÍVEL AUSENTE** — ele continua a alimentar o seguinte.
///
/// ⚠️ Esta é a metade que uma implementação «saltar o nível cujo peso é zero» quebraria em
/// silêncio: a fábrica do oráculo tem o **nível 1 a zero** e os seguintes acesos.
#[test]
fn um_nivel_mudo_continua_a_alimentar_o_seguinte() {
    const W: usize = 128;
    let mut niveis = [0.0f32; Bloom::LEVELS];
    niveis[2] = 1.0; // o 1.º e o 2.º ficam a zero
    let b = Bloom {
        enabled: true,
        threshold: 1.0,
        knee: 0.0,
        intensity: 1.0,
        levels: niveis,
    };
    let mut q = vec![[0.0f32; 3]; W * W];
    for y in W / 2 - 4..W / 2 + 4 {
        for x in W / 2 - 4..W / 2 + 4 {
            q[y * W + x] = [8.0; 3];
        }
    }
    apply(&mut q, W, W, &b);
    let fora = q[(W / 2) * W + W / 2 + 20][0];
    assert!(
        fora > 0.0,
        "com só o 3.º nível aceso ainda tem de haver halo a 20 px — leu {fora}"
    );
}
