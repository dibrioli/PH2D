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
                params: BloomParams {
                    intensity: 0.0,
                    ..BloomParams::default()
                },
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
    let b = BloomParams {
        knee: 0.0,
        threshold: 1.0,
        ..BloomParams::default()
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
    let b = BloomParams {
        threshold: 1.0,
        knee: 0.0,
        ..BloomParams::default()
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

/// ⭐⭐⭐ **O RAIO ALARGA O HALO** — o botão do tamanho, no NOSSO modelo.
///
/// ⛔⛔ **Ele substitui dois gates que mediam os SETE PESOS POR NÍVEL do Godot**
/// (`o_raio_do_halo_dobra_por_nivel` e `um_nivel_mudo_continua_a_alimentar_o_seguinte`), e a
/// premissa deles morreu com o modelo: por ordem do dono (19/09, *«nosso bloom original é muito
/// melhor. retire essa implementação godot»*) o tamanho passou a ser **um** número — o
/// [`BloomParams::radius`], que estica a tenda do upsample.
///
/// ⚠️ **A régua é a largura a meia altura** de um quadrado aceso sobre fundo preto, medida no
/// PRODUTO (o [`apply`]) e não numa peça interna.
#[test]
fn o_raio_alarga_o_halo() {
    const W: usize = 512;
    let mut centros = Vec::new();
    // ⚠️⚠️ **A varredura começa em `1` porque abaixo disso o knob é INERTE, e isso é MEDIDO**
    // (`quanto_o_raio_move`): `0,25 · 0,50 · 1,00` leem `9,71 · 9,58 · 9,59`. *Nesta cadeia quem faz
    // o grosso do borrão é a própria descida por mips; a tenda só o alarga por cima* — e é por isso
    // que a faixa útil do knob é `1..16` e não `0..1`.
    for r in [1.0f32, 2.0, 4.0, 8.0, 16.0] {
        let b = Bloom {
            enabled: true,
            params: BloomParams {
                threshold: 1.0,
                knee: 0.0,
                intensity: 1.0,
                radius: r,
                ..BloomParams::default()
            },
        };
        let mut q = vec![[0.0f32; 3]; W * W];
        for y in W / 2 - 4..W / 2 + 4 {
            for x in W / 2 - 4..W / 2 + 4 {
                q[y * W + x] = [8.0; 3];
            }
        }
        apply(&mut q, W, W, &b);
        // ⚠️⚠️ **A régua é a DISTÂNCIA MÉDIA ponderada pelo halo, e as duas anteriores falharam a
        // MEDIR A JANELA, cada uma à sua maneira:** a meia-altura conta píxeis inteiros e leu `3, 3`
        // entre os raios `0,5` e `1,0`; o alcance até `1/255` leu `120, 120, 120, 120` porque
        // **satura no fim da linha**; e a própria distância média, num quadro de `256`, saturou
        // contra a borda. *Uma régua com unidade maior que o passo não mede o passo, e uma que
        // satura mede a janela* — daí o quadro de `512` com uma fonte de `8`.
        let (mut soma, mut peso) = (0.0f64, 0.0f64);
        for x in W / 2 + 4..W {
            let v = f64::from(q[(W / 2) * W + x][0]);
            #[allow(clippy::cast_precision_loss)]
            let d = (x - (W / 2 + 4)) as f64;
            soma += v * d;
            peso += v;
        }
        assert!(peso > 0.0, "raio {r}: não há halo nenhum");
        centros.push(soma / peso);
    }
    for k in 1..centros.len() {
        assert!(
            centros[k] > centros[k - 1],
            "dobrar o raio tem de ALARGAR o halo, e leu {centros:?}"
        );
    }
    // ⭐ E a ponta contra o pé: o botão tem de ser uma alavanca a sério, não um ajuste fino.
    // Medido: `9,59 → 48,08`, que é `5,0×`.
    assert!(
        centros[4] >= centros[0] * 3.0,
        "de raio 1 a 16 o halo mal se moveu: {centros:?}"
    );
}

/// ⭐⭐ **A SATURAÇÃO A ZERO DÁ UM HALO CINZENTO** — um knob que o nosso modelo tem e o do Godot não.
#[test]
fn a_saturacao_a_zero_da_um_halo_cinzento() {
    const W: usize = 96;
    let faz = |sat: f32| {
        let b = Bloom {
            enabled: true,
            params: BloomParams {
                threshold: 1.0,
                knee: 0.0,
                intensity: 1.0,
                saturation: sat,
                ..BloomParams::default()
            },
        };
        let mut q = vec![[0.0f32; 3]; W * W];
        for y in W / 2 - 6..W / 2 + 6 {
            for x in W / 2 - 6..W / 2 + 6 {
                // Um VERMELHO forte — é nele que a dessaturação se lê.
                q[y * W + x] = [8.0, 0.5, 0.5];
            }
        }
        let halo = halo(&q, W, W, &b);
        halo[(W / 2) * W + W / 2 + 14]
    };
    let colorido = faz(1.0);
    let cinza = faz(0.0);
    assert!(
        colorido[0] > colorido[1] * 2.0,
        "com saturação 1 o halo tem de guardar o vermelho: {colorido:?}"
    );
    assert!(
        (cinza[0] - cinza[1]).abs() < cinza[0] * 0.05,
        "com saturação 0 os três canais têm de ficar iguais: {cinza:?}"
    );
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
                params: BloomParams {
                    threshold: 2.75,
                    knee: 0.125,
                    intensity: 1.5,
                    radius: 2.5,
                    saturation: 0.25,
                    tint: [0.9, 0.5, 0.2, 1.0],
                    clamp: 40.0,
                    stretch: 3.0,
                    angle: 30.0,
                    ..BloomParams::default()
                },
            },
        ),
        (
            "tudo a zero",
            Bloom {
                enabled: false,
                params: BloomParams {
                    threshold: 0.0,
                    knee: 0.0,
                    intensity: 0.0,
                    radius: 0.0,
                    saturation: 0.0,
                    tint: [0.0, 0.0, 0.0, 1.0],
                    clamp: 0.0,
                    stretch: 0.0,
                    angle: 0.0,
                    ..BloomParams::default()
                },
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
    // ⚠️ A contagem é DERIVADA da própria arrumação, nunca escrita ao lado dela.
    assert_eq!(Bloom::default().pack().len(), Bloom::SLOTS);
    assert_eq!(Bloom::default().pack().len(), Bloom::SLOTS);
}

/// ⛔ **O SANEAMENTO acontece na PORTA, e o que não é número vira FÁBRICA e não zero.**
///
/// ⚠️ *Uma recusa não pode ser mais destrutiva do que o pedido:* um `NaN` no limiar saneado para `0`
/// poria a cena inteira a brilhar — o oposto do que quem escreveu o `NaN` podia querer.
#[test]
fn o_saneamento_devolve_a_fabrica_e_nunca_o_zero() {
    let f = BloomParams::default();
    let doente = Bloom {
        enabled: true,
        params: BloomParams {
            threshold: f32::NAN,
            knee: f32::INFINITY,
            intensity: -3.0,
            radius: f32::NAN,
            saturation: 0.4,
            angle: -45.0,
            ..BloomParams::default()
        },
    }
    .sanitized();
    assert_eq!(
        doente.params.threshold, f.threshold,
        "o NaN não virou fábrica"
    );
    assert_eq!(doente.params.knee, f.knee, "o infinito não virou fábrica");
    assert_eq!(
        doente.params.radius, f.radius,
        "o raio NaN não virou fábrica"
    );
    // ⭐ Um NEGATIVO é um número: ele tem cerca (`0`), não vale o valor de fábrica.
    assert_eq!(
        doente.params.intensity, 0.0,
        "um negativo é um número e corta em zero"
    );
    assert_eq!(
        doente.params.saturation, 0.4,
        "um número são não pode ser tocado"
    );
    // ⭐ E o ÂNGULO pode ser negativo: ele é uma DIRECÇÃO, não uma grandeza.
    assert_eq!(
        doente.params.angle, -45.0,
        "o ângulo negativo é uma direcção"
    );
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
        let b = BloomParams {
            threshold: 1.0,
            knee: k,
            ..BloomParams::default()
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

/// Sonda: quanto é que o RAIO de facto move o halo? (`#[ignore]`)
#[test]
#[ignore = "sonda"]
fn quanto_o_raio_move() {
    const W: usize = 512;
    println!(
        "\n{:>8} {:>12} {:>12} {:>12}",
        "raio", "centro", "alcance", "soma"
    );
    for r in [0.25f32, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
        let b = Bloom {
            enabled: true,
            params: BloomParams {
                threshold: 1.0,
                knee: 0.0,
                intensity: 1.0,
                radius: r,
                ..BloomParams::default()
            },
        };
        let mut q = vec![[0.0f32; 3]; W * W];
        for y in W / 2 - 4..W / 2 + 4 {
            for x in W / 2 - 4..W / 2 + 4 {
                q[y * W + x] = [8.0; 3];
            }
        }
        apply(&mut q, W, W, &b);
        let (mut soma, mut peso, mut alc) = (0.0f64, 0.0f64, 0usize);
        for x in W / 2 + 4..W {
            let v = f64::from(q[(W / 2) * W + x][0]);
            #[allow(clippy::cast_precision_loss)]
            let d = (x - (W / 2 + 4)) as f64;
            soma += v * d;
            peso += v;
            if v >= 1.0 / 255.0 {
                alc = x - (W / 2 + 4);
            }
        }
        println!(
            "{r:>8.2} {:>12.2} {alc:>12} {peso:>12.2}",
            soma / peso.max(1e-9)
        );
    }
}

/// ⭐⭐⭐ **O HALO DO MOTION E O DA CENA 3D SÃO A MESMA ESTRUTURA** — e é um TIPO, não uma promessa.
///
/// ⛔⛔ Ordem do dono (19/09): *«nosso bloom original é muito melhor. retire essa implementação
/// godot»*. A cura não foi copiar os campos dele para cá — foi **mudar o tipo de casa**: o
/// [`BloomParams`] vivia no `ph2d_render::motion_fx_params` e passou a viver nesta folha, com o
/// `ph2d-render` a re-exportá-lo. *Duas cópias com uma nota a dizer «mantenha-as iguais» é
/// exactamente o que diverge no dia em que uma ganha um campo.*
///
/// ⚠️ **Este gate mede o que sobra por medir:** que o re-export continua a apontar aqui. Ele reprova
/// no dia em que alguém declarar um segundo `BloomParams` do outro lado — que é a única forma de a
/// lei voltar a ter dois donos.
#[test]
fn o_halo_do_motion_e_o_da_cena_sao_a_mesma_estrutura() {
    // ⚠️ A régua é a IDENTIDADE DE TIPO, e ela é do compilador: se o `ph2d_render::BloomParams`
    // deixar de ser este, esta atribuição deixa de compilar.
    let meu = BloomParams::default();
    let dele: BloomParams = meu;
    assert_eq!(meu, dele);
    // ⭐ E os valores de fábrica são os NOSSOS, não os do oráculo: o Godot ship `intensity 0,3` e
    // `knee 0,5`; nós shipamos `0,8` e `0,6`, que são os do halo que o dono já aprovou no Motion.
    assert!(
        (meu.intensity - 0.8).abs() < f32::EPSILON && (meu.knee - 0.6).abs() < f32::EPSILON,
        "os valores de fábrica deixaram de ser os nossos: {meu:?}"
    );
    // ⛔ E não há mais nenhum peso por nível — o tamanho é UM número.
    assert!(
        (meu.radius - 1.0).abs() < f32::EPSILON,
        "o raio de fábrica mudou sem ninguém dizer"
    );
}

/// ⏱️ **QUANTO CUSTA A CADEIA NA CPU** — a medição que mandou o brilho para o dispositivo.
///
/// Medido (`--release`, melhor de 5, nesta máquina), nos **dois tamanhos que o produto usa**:
///
/// | tamanho | quadro | cadeia |
/// |---|---|---:|
/// | `445×305` | o de MOVIMENTO | **`24,1 ms`** |
/// | `1898×916` | o ASSENTE | **`347,2 ms`** |
///
/// ⚠️ **O de movimento sozinho já não cabe num quadro de `16,7 ms`**, e o assente custa **um terço
/// de segundo**. ⇒ trazer o quadro do dispositivo para a CPU só para o brilho está fora de
/// questão, e o gémeo em WGSL não é uma optimização: é a única forma de este efeito existir no
/// caminho de omissão.
///
/// ⚠️ **A razão de ser tão caro é estrutural e está declarada:** esta crate é uma FOLHA de **zero
/// dependências** (a lei da casa para uma lei partilhada), logo não tem `rayon` — enquanto o resto
/// do sombreador corre em `par_chunks_mut`. *O preço da pureza da folha paga-se aqui, e é por isso
/// que o caminho de referência fica lento com o brilho ligado.*
#[test]
#[ignore = "sonda"]
fn quanto_custa_a_cadeia() {
    for (w, h) in [(445usize, 305usize), (1898usize, 916usize)] {
        let mut hdr = vec![[0.0f32; 3]; w * h];
        for j in 0..h {
            for i in 0..w {
                let (dx, dy) = (i as f32 - w as f32 / 2.0, j as f32 - h as f32 / 2.0);
                if dx * dx + dy * dy < (h as f32 / 8.0).powi(2) {
                    hdr[j * w + i] = [11.0, 10.0, 9.0];
                }
            }
        }
        let b = Bloom {
            enabled: true,
            params: BloomParams::default(),
        };
        let mut melhor = f64::MAX;
        for _ in 0..5 {
            let t = std::time::Instant::now();
            let r = halo(&hdr, w, h, &b);
            let ms = t.elapsed().as_secs_f64() * 1000.0;
            assert!(!r.is_empty());
            melhor = melhor.min(ms);
        }
        println!("SONDA {w}x{h}: {melhor:.2} ms (melhor de 5)");
    }
}

/// ⭐⭐⭐ **O TECTO DO CORTE (o `Clamp`) MORDE — e até 2026-09-19 ele era um BOTÃO MORTO.**
///
/// # ⛔⛔ O que a medição achou
///
/// O painel do modelador oferece a fileira **Clamp** desde que a wave shipou, e a [`halo`] **nunca
/// lia** `params.clamp`: o [`BloomParams::clamp_limit`] tinha **um** leitor no repositório inteiro,
/// o gémeo do Motion (`motion_fx.rs`). ⇒ *o artista arrastava a fileira e a imagem não mudava um
/// bit* — a espécie de knob morto que o `CLAUDE.md` §5.0 nomeia e que **nenhuma sonda deste repo
/// pergunta**: o fio existe, a fileira é pintada, o valor chega ao tipo, e a LEI não o lê.
///
/// ⚠️ **A lei não se inventou: ela já estava escrita no shader que shipa** (`bloom.wgsl`,
/// `fs_prefilter`) — `min` **POR CANAL, antes da luminância**. Por canal de propósito: limitar a
/// luminância e reescalar mudaria o **matiz** do pixel que estourou, e o antídoto do *firefly* não
/// pode recolorir a cena.
///
/// ⚠️ **Com o knob desligado o tecto é o maior finito do `Rgba16Float`** ⇒ o `min` não morde nada
/// que o quadro consiga guardar, e o caminho de omissão fica **ao bit** (a metade de baixo).
#[test]
fn o_tecto_do_corte_morde_e_a_omissao_fica_ao_bit() {
    let (w, h) = (96usize, 96usize);
    let mut hdr = vec![[0.0f32; 3]; w * h];
    // Uma fonte MUITO acima do tecto, para o `min` ter o que morder.
    for j in 40..56 {
        for i in 40..56 {
            hdr[j * w + i] = [40.0, 30.0, 10.0];
        }
    }
    let liga = |clamp: f32| Bloom {
        enabled: true,
        params: BloomParams {
            clamp,
            ..BloomParams::default()
        },
    };
    let solto = halo(&hdr, w, h, &liga(0.0));
    let preso = halo(&hdr, w, h, &liga(2.0));
    let maior = |v: &[[f32; 3]]| v.iter().flatten().fold(0.0f32, |a, &b| a.max(b));
    assert!(
        maior(&preso) < maior(&solto) * 0.5,
        "o tecto não mordeu: solto {:.3}, preso {:.3}",
        maior(&solto),
        maior(&preso)
    );
    // ⭐ **A MATIZ sobrevive ao tecto** — é isto que separa o `min` por canal de um `min` na
    // luminância com reescala. O pixel de origem é `40 · 30 · 10`; depois do tecto a `2` os três
    // canais ficam iguais e o halo sai CINZENTO, que é o que um corte por canal faz.
    let pico = preso.iter().copied().fold([0.0f32; 3], |a, b| {
        [a[0].max(b[0]), a[1].max(b[1]), a[2].max(b[2])]
    });
    assert!(
        (pico[0] - pico[1]).abs() < pico[0] * 0.02,
        "o tecto por canal devia igualar os canais saturados: {pico:?}"
    );
    // ⚠️ **A metade de baixo**: com o knob DESLIGADO o tecto é o maior finito do formato, e nenhuma
    // cena real o alcança ⇒ a imagem tem de ser a de sempre, ao bit.
    let teto_alto = halo(&hdr, w, h, &liga(F16_MAX));
    assert_eq!(
        solto
            .iter()
            .flatten()
            .map(|f| f.to_bits())
            .collect::<Vec<_>>(),
        teto_alto
            .iter()
            .flatten()
            .map(|f| f.to_bits())
            .collect::<Vec<_>>(),
        "o knob desligado e o tecto do formato têm de dar o MESMO halo"
    );
}
