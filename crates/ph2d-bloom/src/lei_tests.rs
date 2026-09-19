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

/// ⭐⭐⭐ **A ARRUMAÇÃO VAI E VOLTA** — e com os DOIS lados, porque uma que só vai é meia arrumação.
///
/// ⚠️ O corpus tem o interruptor nas duas posições e números fora do comum de propósito: *uma
/// ida-e-volta medida só no valor de fábrica afirma sobre uma célula e lê-se como afirmando sobre a
/// tabela toda.*
#[test]
fn a_arrumacao_vai_e_volta() {
    let corpus = [
        ("fábrica", Bloom::default()),
        (
            "ligado e torto",
            Bloom {
                enabled: true,
                threshold: 2.75,
                knee: 0.125,
                intensity: 1.5,
                levels: [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7],
            },
        ),
        (
            "tudo a zero",
            Bloom {
                enabled: false,
                threshold: 0.0,
                knee: 0.0,
                intensity: 0.0,
                levels: [0.0; Bloom::LEVELS],
            },
        ),
    ];
    for (rot, b) in corpus {
        let volta = Bloom::unpack(&b.pack());
        assert_eq!(volta.enabled, b.enabled, "{rot}: o interruptor perdeu-se");
        assert_eq!(
            volta.pack().map(f32::to_bits),
            b.pack().map(f32::to_bits),
            "{rot}: a volta não é a ida"
        );
    }
    // ⚠️ E a CONTAGEM é derivada, nunca escrita: `4` números mais os níveis.
    assert_eq!(Bloom::SLOTS, 4 + Bloom::LEVELS);
    assert_eq!(Bloom::default().pack().len(), Bloom::SLOTS);
}

/// ⛔ **O SANEAMENTO acontece na PORTA, e o que não é número vira FÁBRICA e não zero.**
///
/// ⚠️ *Uma recusa não pode ser mais destrutiva do que o pedido:* um `NaN` no limiar saneado para `0`
/// poria a cena inteira a brilhar — o oposto do que quem escreveu o `NaN` podia querer.
#[test]
fn o_saneamento_devolve_a_fabrica_e_nunca_o_zero() {
    let doente = Bloom {
        enabled: true,
        threshold: f32::NAN,
        knee: f32::INFINITY,
        intensity: -3.0,
        levels: [f32::NAN, -1.0, 0.4, 0.0, 0.0, 0.0, 0.0],
    }
    .sanitized();
    assert_eq!(
        doente.threshold,
        Bloom::THRESHOLD,
        "o NaN não virou fábrica"
    );
    assert_eq!(doente.knee, Bloom::KNEE, "o infinito não virou fábrica");
    // ⭐ Um NEGATIVO é um número: ele tem cerca (`0`), não vale o valor de fábrica.
    assert_eq!(
        doente.intensity, 0.0,
        "um negativo é um número e corta em zero"
    );
    assert_eq!(
        doente.levels[0],
        Bloom::NIVEIS[0],
        "o nível NaN não virou fábrica"
    );
    assert_eq!(doente.levels[1], 0.0, "um nível negativo corta em zero");
    assert_eq!(doente.levels[2], 0.4, "um nível são não pode ser tocado");
    // ⭐ E o CONTROLO: um brilho já são sai AO BIT.
    let sao = Bloom {
        enabled: true,
        ..Bloom::default()
    };
    assert_eq!(
        sao.sanitized().pack().map(f32::to_bits),
        sao.pack().map(f32::to_bits),
        "o saneamento mexeu num brilho que já estava são"
    );
}

/// ⭐⭐⭐ **A SONDA DO JOELHO** — `#[ignore]`, sobre uma RAMPA, que é a única fixtura onde ele existe.
///
/// ⛔⛔ **A sonda dos tectos do consumidor mediu o joelho e leu o MESMO número nas nove células**
/// (`3 423,70` de `0` a `16`), e a leitura ingénua disso é *«o joelho está morto»*. Ele não está: a
/// fixtura de lá é um disco de brilho **CHATO** a `40`, e com o limiar em `1` todo pixel dela tem
/// `duro = 39` — muito acima de qualquer joelho. *O joelho molda a PASSAGEM entre «não brilha» e
/// «brilha», logo uma fixtura sem píxeis na passagem não o contém.*
///
/// ⇒ a régua é uma **RAMPA** de luminância que atravessa o limiar, e o que se mede é a soma do
/// corte ao longo dela.
///
/// ```text
/// cargo test -p ph2d-bloom --lib a_sonda_do_joelho -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela do joelho, não afirma"]
fn a_sonda_do_joelho() {
    const N: usize = 2001;
    // A rampa: luminância de `0` a `4`, com o limiar em `1`.
    let rampa: Vec<f32> = (0..N)
        .map(|i| {
            #[allow(clippy::cast_precision_loss)]
            let f = i as f32;
            f * 4.0 / (N - 1) as f32
        })
        .collect();
    println!(
        "\n{:>8} {:>12} {:>14} {:>14} {:>12}",
        "joelho", "soma", "1.ª luz em", "cheio em", "largura"
    );
    for k in [0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
        let b = Bloom {
            threshold: 1.0,
            knee: k,
            ..Bloom::default()
        };
        let mut soma = 0.0f64;
        let (mut primeira, mut cheio) = (None, None);
        for &l in &rampa {
            let w = bright([l, l, l], &b)[0];
            soma += f64::from(w);
            if w > 0.0 && primeira.is_none() {
                primeira = Some(l);
            }
            // «cheio» = o corte duro alcança o corte com joelho a menos de 1 %.
            if w > 0.0 && (w - (l - 1.0).max(0.0)).abs() <= 0.01 * w && cheio.is_none() && l > 1.0 {
                cheio = Some(l);
            }
        }
        let (p, c) = (primeira.unwrap_or(f32::NAN), cheio.unwrap_or(f32::NAN));
        println!("{k:>8.3} {soma:>12.3} {p:>14.4} {c:>14.4} {:>12.4}", c - p);
    }
}
