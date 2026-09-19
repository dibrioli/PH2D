//! Os gates do passe do brilho, **pelo caminho do produto** ([`crate::shade_render`]).

use crate::{Gbuffer, Lighting, Orbit, Presentation, Surfaces, shade_render};
use ph2d_material::{Environment, OpenPbr};

struct Ceu([f32; 3]);

impl Environment for Ceu {
    fn radiance(&self, _dir: [f32; 3], _alpha: f32) -> [f32; 3] {
        self.0
    }
    fn irradiance(&self, _n: [f32; 3]) -> [f32; 3] {
        self.0
    }
}

const FUNDO: [u8; 4] = [0, 0, 0, 255];

/// ⭐⭐⭐ **O FUNDO QUE O PRODUTO USA** — e a razão de este ficheiro ter dois.
///
/// ⛔⛔ **Todos os gates deste ficheiro nasceram sobre o [`FUNDO`] OPACO, e é por isso que nenhum
/// deles viu o report do dono de 2026-09-19** (*«não se percebe o efeito ao redor das esferas»*):
/// o modelador traça com `[0, 0, 0, 0]` — o quadro é composto sobre o canvas, que é quem desenha o
/// cinzento e a grelha — e **o halo mora, por definição, onde a peça não está**, logo onde a
/// cobertura é `0`. *Uma fixtura com fundo opaco não pode conter o fenómeno:* ali o alfa é `255` em
/// todo o quadro e nunca chega a ser a grandeza que decide.
///
/// ⚠️ **É o valor do produto, não um valor escolhido** — ver o `BACKGROUND` do
/// `ph2d-app-field3d`, e o gate daquela crate que o mede pela cena a sério.
const FUNDO_TRANSPARENTE: [u8; 4] = [0, 0, 0, 0];
const LADO: u32 = 96;

/// Um disco aceso ao centro de um quadro escuro — a fixtura que **contém o fenómeno**.
///
/// ⚠️ **Um quadro sem nada aceso não testa o brilho**, e um todo aceso não tem onde mostrar o halo:
/// é preciso a fronteira entre os dois.
fn disco() -> Gbuffer {
    let n = (LADO * LADO) as usize;
    let mut hit = vec![false; n];
    let (c, r) = (f64::from(LADO) / 2.0, f64::from(LADO) / 6.0);
    for y in 0..LADO {
        for x in 0..LADO {
            let (dx, dy) = (f64::from(x) - c, f64::from(y) - c);
            if dx.mul_add(dx, dy * dy) <= r * r {
                hit[(y * LADO + x) as usize] = true;
            }
        }
    }
    Gbuffer {
        width: LADO,
        height: LADO,
        hit,
        normal: vec![[0.0, 0.0, 1.0]; n],
        point: vec![[0.0; 3]; n],
        curvature: Vec::new(),
        curvature_style: Vec::new(),
        edges: Vec::new(),
    }
}

fn pinta(pres: &Presentation) -> Vec<u8> {
    pinta_sobre(pres, FUNDO)
}

/// O mesmo quadro, sobre o fundo que o chamador pedir — ver [`FUNDO_TRANSPARENTE`].
fn pinta_sobre(pres: &Presentation, fundo: [u8; 4]) -> Vec<u8> {
    let g = disco();
    let ceu = Ceu([0.0; 3]);
    // ⭐ Uma peça que EMITE — é o que põe a luz acima do limiar sem depender de lâmpada nenhuma.
    let so = [OpenPbr {
        base_color: [0.0; 3],
        emission_luminance: 40.0,
        emission_color: [1.0, 0.9, 0.7],
        ..OpenPbr::default()
    }
    .prepare()];
    shade_render(
        &g,
        &Orbit::default(),
        &Surfaces {
            all: &so,
            owners: None,
        },
        &Lighting {
            lamps: &[],
            points: &[],
            sky: &ceu,
            shadows: None,
        },
        pres,
        fundo,
    )
}

/// ⭐ Um brilho LIGADO com os params que o chamador pedir — a porta das fixturas deste ficheiro.
fn ligado(p: ph2d_bloom::BloomParams) -> ph2d_bloom::Bloom {
    ph2d_bloom::Bloom {
        enabled: true,
        params: p,
    }
}

fn com(bloom: ph2d_bloom::Bloom) -> Presentation {
    Presentation {
        bloom,
        ..Presentation::of(ph2d_view_transform::Look::default())
    }
}

/// ⭐⭐⭐ **A OMISSÃO É A IMAGEM DE SEMPRE, AO BIT** — a metade que protege tudo o que já shipou.
#[test]
fn a_omissao_e_a_imagem_de_sempre_ao_bit() {
    let base = pinta(&Presentation::of(ph2d_view_transform::Look::default()));
    let com_campo = pinta(&com(ph2d_bloom::Bloom::default()));
    assert_eq!(base, com_campo, "o brilho desligado mudou a imagem");
}

/// ⭐⭐⭐ **O OLHAR MANDA O PRETO EM PRETO** — a propriedade de que o [`super::soma_halo`] depende,
/// gateada **onde ela vive** e não onde é consumida.
///
/// ⛔ Ela nasceu de uma prova de mutação: aquele passe subtraía `look([0,0,0])` do halo *«porque o
/// olhar tem um desvio para o preto»*, e a mutação que apagava a subtracção **sobreviveu**. Medido,
/// `look([0,0,0])` é `[0,0,0]` ao bit — a subtracção era morta e saiu.
///
/// ⚠️ **A régua varre o espaço INTEIRO do olhar, não o de omissão:** as duas transformações × uma
/// escada de exposições que inclui os extremos e o que não é número. *Um gate escrito só sobre o
/// `Look::default()` afirmaria sobre uma célula de uma tabela e leria-se como afirmando sobre ela
/// toda* — e é a tabela toda que este passe assume.
#[test]
fn o_olhar_manda_o_preto_em_preto() {
    use ph2d_view_transform::{Look, ViewTransform};
    let mut celulas = 0usize;
    for view in ViewTransform::ALL {
        for stops in [
            -32.0,
            -8.0,
            -1.0,
            0.0,
            1.0,
            8.0,
            32.0,
            f32::NAN,
            f32::INFINITY,
        ] {
            let z = Look {
                exposure_stops: stops,
                view,
            }
            .apply([0.0; 3]);
            assert_eq!(
                z.map(f32::to_bits),
                [0u32; 3],
                "look({view:?}, {stops} stops) levou o preto a {z:?} — o passe do brilho \
                 ([`super::soma_halo`]) soma `look(halo)` CRU e conta com isto"
            );
            celulas += 1;
        }
    }
    assert!(
        celulas >= 18,
        "piso de população: a varredura leu só {celulas} células do olhar"
    );
}

/// ⭐⭐ **O INTERRUPTOR DO QUADRO É A LEI DA CRATE, e não uma segunda resposta** — ida-e-volta sobre
/// um corpus que tem os dois lados.
///
/// ⛔ Ele nasceu de uma mutação SOBREVIVENTE: cravar `Presentation::blooms()` em `true` não muda um
/// byte, porque o [`ph2d_bloom::halo`] tem a **própria** guarda e devolve um halo mudo ⇒ *as duas
/// guardas são redundantes na IMAGEM e não no RELÓGIO*, e o que a de fora compra — o buffer de cena
/// não nascer — é um custo, que um gate de bytes não vê.
///
/// ⚠️ *A cura não é medir o relógio* (seria mais um membro da família de flakes sob fan-out): é
/// medir que a porta **delega**, que é a afirmação que o doc dela faz.
#[test]
fn o_interruptor_do_quadro_e_a_lei_da_crate() {
    let corpus = [
        ("omissão", ph2d_bloom::Bloom::default()),
        ("ligado", ligado(ph2d_bloom::BloomParams::default())),
        (
            "ligado e mudo",
            ligado(ph2d_bloom::BloomParams {
                intensity: 0.0,
                ..ph2d_bloom::BloomParams::default()
            }),
        ),
        (
            "ligado e sem raio",
            ligado(ph2d_bloom::BloomParams {
                radius: 0.0,
                ..ph2d_bloom::BloomParams::default()
            }),
        ),
    ];
    let (mut sim, mut nao) = (0usize, 0usize);
    for (rot, b) in corpus {
        let esperado = b.contributes();
        assert_eq!(
            com(b).blooms(),
            esperado,
            "{rot}: a porta do quadro discordou da lei da crate"
        );
        if esperado { sim += 1 } else { nao += 1 }
    }
    // ⚠️ Sem os DOIS lados, cravar a porta numa constante passava: um corpus só de «não contribui»
    // aprova um `false` cravado, e um só de «contribui» aprova um `true`.
    assert!(
        sim >= 1 && nao >= 2,
        "o corpus tem de conter os dois lados — leu {sim} a contribuir e {nao} a não contribuir"
    );
}

/// ⭐⭐⭐ **O HALO PASSA PELO OLHAR** — a lei que o cabeçalho do [`super`] afirma, agora com régua.
///
/// ⚠️⚠️ **Ela só é OBSERVÁVEL sob [`ph2d_view_transform::ViewTransform::Neutral`], e isso é um facto
/// sobre o olhar e não sobre o brilho:** o `Standard` corta cada canal em `1`, logo abaixo do branco
/// ele é a identidade e acima dele o byte satura de qualquer maneira ⇒ *com o olhar de omissão,
/// aplicá-lo ao halo ou não dá o MESMO ficheiro*. Um gate escrito na omissão leria verde sobre um
/// passe que soma o halo cru, que é precisamente a composição que o `docs/Render3d/10` §11.3 recusou
/// por medição.
#[test]
fn o_halo_passa_pelo_olhar() {
    let olhar = ph2d_view_transform::Look {
        exposure_stops: 0.0,
        view: ph2d_view_transform::ViewTransform::Neutral,
    };
    let pres = Presentation {
        bloom: ph2d_bloom::Bloom::default(),
        ..Presentation::of(olhar)
    };
    let base = pinta(&pres);

    // Um halo FORTE, onde as duas leis se separam: o olhar comprime, o cru não.
    let forte = vec![[3.0f32; 3]; (LADO * LADO) as usize];
    let (mut pelo_olhar, mut cru) = (base.clone(), base.clone());
    crate::brilho::soma_halo(&mut pelo_olhar, &forte, &pres);
    let (pixeis, _) = cru.as_chunks_mut::<4>();
    for (px, h) in pixeis.iter_mut().zip(&forte) {
        for (canal, &acrescimo) in px.iter_mut().zip(h) {
            let b = ph2d_color::srgb::srgb_to_linear_byte(*canal);
            *canal = ph2d_color::srgb::linear_to_srgb_byte(b + acrescimo);
        }
    }
    let diferentes = pelo_olhar
        .as_chunks::<4>()
        .0
        .iter()
        .zip(cru.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert!(
        diferentes > 1000,
        "sob o olhar Neutral um halo de 3,0 tem de sair COMPRIMIDO e não cru — \
         só {diferentes} píxeis separam as duas leis"
    );
    // ⚠️ E o sentido importa: comprimir dá MENOS luz que somar cru, nunca mais.
    assert!(
        pelo_olhar
            .as_chunks::<4>()
            .0
            .iter()
            .zip(cru.as_chunks::<4>().0.iter())
            .all(|(a, b)| a[0] <= b[0] && a[1] <= b[1] && a[2] <= b[2]),
        "o olhar comprime: nenhum canal pode sair mais claro do que a soma crua"
    );
}

/// ⭐⭐ **UM HALO NULO É A IDENTIDADE AO BIT** — o caso nulo, que um gate de «com o brilho ligado a
/// imagem muda» nunca vê.
#[test]
fn um_halo_nulo_e_a_identidade_ao_bit() {
    let pres = com(ph2d_bloom::Bloom::default());
    let base = pinta(&pres);
    let mut mexido = base.clone();
    let nulo = vec![[0.0f32; 3]; (LADO * LADO) as usize];
    crate::brilho::soma_halo(&mut mexido, &nulo, &pres);
    assert_eq!(base, mexido, "um halo de zeros mudou bytes");
}

/// ⭐⭐⭐ **O BRILHO ACENDE FORA DA PEÇA** — a lei do produto, e o CONTROLO ao lado.
///
/// O controlo é o mesmo quadro com o brilho desligado: sem ele, um gate que só olhasse para o
/// resultado não distinguiria *«o halo acendeu»* de *«a peça já era grande»*.
#[test]
fn o_brilho_acende_fora_da_peca() {
    let sem = pinta(&com(ph2d_bloom::Bloom::default()));
    let com_brilho = pinta(&com(ligado(ph2d_bloom::BloomParams::default())));

    let g = disco();
    let (mut acesos, mut dentro) = (0usize, 0usize);
    for i in 0..(LADO * LADO) as usize {
        let (a, b) = (sem[i * 4], com_brilho[i * 4]);
        if g.hit[i] {
            dentro += usize::from(b >= a);
        } else if b > a {
            acesos += 1;
        }
        assert!(b >= a, "o brilho ESCURECEU o pixel {i} ({a} → {b})");
    }
    assert!(
        acesos > 200,
        "o halo mal chegou ao fundo: só {acesos} píxeis acenderam"
    );
    assert!(dentro > 0, "a peça também tem de receber o halo");
}

/// ⛔ **O QUE NÃO PASSA DO LIMIAR NÃO BRILHA** — e o quadro fica AO BIT, **com o CONTROLO ao lado**.
///
/// ⚠️ A régua é a **imagem inteira**: um limiar que gateasse *quase* deixaria um halo fraco, e uma
/// barra em «quase zero» aceitá-lo-ia. *O corte é duro, e o gate mede-o como duro.*
///
/// ⛔⛔ **A metade de baixo é o que torna a de cima uma afirmação.** A 1.ª redacção tinha só o
/// limiar alto, e *quase tudo naquele caminho devolve preto*: a guarda do corte, a higiene do olhar
/// que come um canal negativo, e o `d <= 0` da soma. ⇒ um quadro byte-idêntico é compatível com o
/// limiar a funcionar E com metade do passe morto. **O controlo prova que a fixtura CONTÉM o
/// fenómeno** — com o limiar abaixo do pico da cena, a mesma bateria tem de mover píxeis.
#[test]
fn o_que_nao_passa_do_limiar_nao_brilha() {
    let sem = pinta(&com(ph2d_bloom::Bloom::default()));

    let alto = pinta(&com(ligado(ph2d_bloom::BloomParams {
        threshold: 1.0e6,
        knee: 0.0,
        ..ph2d_bloom::BloomParams::default()
    })));
    assert_eq!(
        sem, alto,
        "com o limiar acima de toda a luz da cena a imagem tem de ficar ao bit"
    );

    // ⭐ O CONTROLO: o MESMO caminho, com o limiar debaixo do pico da peça acesa.
    let baixo = pinta(&com(ligado(ph2d_bloom::BloomParams {
        threshold: 0.1,
        knee: 0.0,
        ..ph2d_bloom::BloomParams::default()
    })));
    let movidos = sem
        .as_chunks::<4>()
        .0
        .iter()
        .zip(baixo.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a[..3] != b[..3])
        .count();
    assert!(
        movidos > 200,
        "controlo: com o limiar EM BAIXO a cena tem de acender — só {movidos} píxeis mudaram, \
         logo a metade de cima deste gate não estava a afirmar nada"
    );
}

/// ⭐⭐⭐ **A SONDA DOS TECTOS** — `#[ignore]`, e o que ela imprime é o que a tabela das fileiras cita.
///
/// ⚠️ **§0.0: antes de escrever um tecto, MEÇA** — e a auditoria da camada de estilo (`11` §10.8)
/// apanhou **três de cinco** dos tectos dela como palpite, um com o número errado por `4×`.
///
/// # ⛔⛔ A 1.ª redacção desta sonda MEDIA EM BYTES e SATUROU — a lição que o `12` §3.3 já escrevia
///
/// Ela contava *quantos píxeis do fundo mudaram* e *o byte de pico*, e sobre a peça acesa as duas
/// colunas leram **`8 419` e `255` em toda a varredura**: o fundo inteiro, e o topo da faixa. *Uma
/// régua que mede a SAÍDA do produto herda o tecto da saída do produto* — e a §3.3 daquele plano diz
/// exactamente isto sobre a varredura do oráculo, três semanas antes.
///
/// ⇒ a régua é o **HALO em linear**, onde ele nasce: a soma sobre o fundo, o pico, e o **RAIO** até
/// `1/255` a partir da borda da peça — as três colunas com que o oráculo foi medido, logo as três em
/// que os dois lados são comparáveis.
///
/// ```text
/// cargo test -p ph2d-field-render --lib os_tectos_do_brilho -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sonda: imprime a tabela dos tectos, não afirma"]
fn os_tectos_do_brilho() {
    let g = disco();
    let (w, h) = (LADO as usize, LADO as usize);
    let pres = com(ph2d_bloom::Bloom::default());
    let cena = crate::brilho::campo_de_cena(
        &g,
        &Orbit::default(),
        &Surfaces {
            all: &[OpenPbr {
                base_color: [0.0; 3],
                emission_luminance: 40.0,
                emission_color: [1.0, 0.9, 0.7],
                ..OpenPbr::default()
            }
            .prepare()],
            owners: None,
        },
        &Lighting {
            lamps: &[],
            points: &[],
            sky: &Ceu([0.0; 3]),
            shadows: None,
        },
        &pres,
    );
    let pico_da_cena = cena
        .iter()
        .map(|p| p[0].max(p[1]).max(p[2]))
        .fold(0.0f32, f32::max);

    // A régua: soma linear do halo FORA da peça, o pico dele, e até onde ele chega.
    let mede = |b: ph2d_bloom::Bloom| {
        let halo = ph2d_bloom::halo(&cena, w, h, &b);
        if halo.is_empty() {
            return (0.0, 0.0, 0usize);
        }
        let (mut soma, mut pico, mut raio) = (0.0f64, 0.0f32, 0usize);
        let (cx, cy) = (w / 2, h / 2);
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if g.hit[i] {
                    continue;
                }
                let v = halo[i][0].max(halo[i][1]).max(halo[i][2]);
                soma += f64::from(v);
                pico = pico.max(v);
                if v >= 1.0 / 255.0 {
                    let (dx, dy) = ((x as f64 - cx as f64), (y as f64 - cy as f64));
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let d = dx.hypot(dy).round() as usize;
                    raio = raio.max(d);
                }
            }
        }
        (soma, f64::from(pico), raio)
    };
    let ligado = |p: ph2d_bloom::BloomParams| ph2d_bloom::Bloom {
        enabled: true,
        params: p,
    };
    let f = ph2d_bloom::BloomParams::default;
    let linha = |rot: String, b: ph2d_bloom::Bloom| {
        let (s, p, r) = mede(b);
        println!("{rot:>10} {s:>12.2} {p:>10.4} {r:>8}");
    };
    let cab = |t: &str| {
        println!("\n=== {t} ===");
        println!(
            "{:>10} {:>12} {:>10} {:>8}",
            "valor", "soma", "pico", "raio"
        );
    };

    println!(
        "\npico da CENA = {pico_da_cena:.3} · níveis que cabem a {w}×{h}: {}",
        ph2d_bloom::levels_that_fit(w, h)
    );
    cab("LIMIAR (os outros de fábrica)");
    for v in [0.0, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0] {
        linha(
            format!("{v:.2}"),
            ligado(ph2d_bloom::BloomParams {
                threshold: v,
                ..f()
            }),
        );
    }
    cab("JOELHO (numa fixtura CHATA ele é inerte — ver `a_sonda_do_joelho`, que usa uma RAMPA)");
    for v in [0.0, 0.25, 1.0, 4.0, 16.0] {
        linha(
            format!("{v:.3}"),
            ligado(ph2d_bloom::BloomParams { knee: v, ..f() }),
        );
    }
    cab("INTENSIDADE");
    for v in [0.0, 0.1, 0.4, 0.8, 1.0, 2.0, 4.0, 8.0] {
        linha(
            format!("{v:.2}"),
            ligado(ph2d_bloom::BloomParams {
                intensity: v,
                ..f()
            }),
        );
    }
    cab("RAIO (o botão do TAMANHO — o nosso modelo tem UM, não sete pesos)");
    for v in [0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0] {
        linha(
            format!("{v:.2}"),
            ligado(ph2d_bloom::BloomParams { radius: v, ..f() }),
        );
    }
    cab("SATURAÇÃO");
    for v in [0.0, 0.25, 0.5, 1.0] {
        linha(
            format!("{v:.2}"),
            ligado(ph2d_bloom::BloomParams {
                saturation: v,
                ..f()
            }),
        );
    }
    cab("TECTO DO CORTE (0 = desligado)");
    for v in [0.0, 2.0, 8.0, 32.0, 128.0] {
        linha(
            format!("{v:.2}"),
            ligado(ph2d_bloom::BloomParams { clamp: v, ..f() }),
        );
    }
}

/// ⭐⭐⭐ **A LUZ DO HALO LEVA COBERTURA CONSIGO — senão o compositor apaga-a.**
///
/// # ⛔⛔⛔ O report do dono de 2026-09-19, e porque nenhum gate o via
///
/// *«Talvez devido a total falta de atmosfera não se possa perceber o efeito ao redor das esferas»*
/// — com a foto. Medido pelo caminho do produto: o [`super::soma_halo`] **acendia** `2 738 165` de
/// `5 215 704` canais do quadro, com saltos até `255` bytes, e **a tela não mostrava nada**.
///
/// A causa não é a lei do halo: é que ele mora onde a peça **não** está, e ali o quadro do
/// modelador tem **cobertura zero**. O passe somava a luz e deixava o alfa como estava — e o que
/// chega ao ecrã é esse quadro **composto** sobre o canvas. *Luz com cobertura zero é luz que o
/// compositor multiplica por nada.*
///
/// ⚠️ **A lei que fica:** o halo é uma camada de LUZ, e uma camada tem cobertura. Ela entra como
/// `c + (1 − c)·a` — a mesma composição que a sombra do chão já faz em
/// [`crate::ground_shade::shadowed_background`], onde o preto que tapa o fundo sobe o alfa pela
/// mesma conta. *A sombra já sabia disto; a luz é que não sabia.*
///
/// ⚠️ **As DUAS metades são obrigatórias.** Sem o controlo (o mesmo quadro sem brilho), um gate que
/// só exigisse `alfa > 0` passaria com um passe que pusesse o quadro inteiro opaco.
#[test]
fn a_luz_do_halo_leva_cobertura_senao_o_compositor_apaga_a() {
    let sem = pinta_sobre(&com(ph2d_bloom::Bloom::default()), FUNDO_TRANSPARENTE);
    let comb = pinta_sobre(
        &com(ligado(ph2d_bloom::BloomParams::default())),
        FUNDO_TRANSPARENTE,
    );
    let g = disco();
    let (mut acesos, mut cobertos, mut fundo_sem_cobertura) = (0usize, 0usize, 0usize);
    for i in 0..(LADO * LADO) as usize {
        if g.hit[i] {
            continue;
        }
        // ⚠️ **O CONTROLO**: sem brilho, um pixel de fundo é transparente — é isso que faz a
        // metade de cima ser uma afirmação sobre o HALO e não sobre a fixtura.
        assert_eq!(
            sem[i * 4 + 3],
            0,
            "o controlo falhou: o fundo do pixel {i} já vinha coberto"
        );
        let luz = comb[i * 4].max(comb[i * 4 + 1]).max(comb[i * 4 + 2]);
        if luz > 0 {
            acesos += 1;
            if comb[i * 4 + 3] > 0 {
                cobertos += 1;
            } else {
                fundo_sem_cobertura += 1;
            }
        }
    }
    assert!(
        acesos > 200,
        "a fixtura não contém o fenómeno: só {acesos} píxeis de fundo acenderam"
    );
    assert_eq!(
        fundo_sem_cobertura, 0,
        "{fundo_sem_cobertura} de {acesos} píxeis acesos têm cobertura ZERO — o compositor apaga-os"
    );
    assert_eq!(acesos, cobertos, "luz acesa sem cobertura");
}

/// ⭐⭐ **E A COBERTURA NUNCA DESCE, NEM A PEÇA DEIXA DE SER OPACA** — a metade que impede a cura
/// de ser *«põe tudo a 255»* ou *«come a silhueta»*.
#[test]
fn a_cobertura_so_sobe_e_a_peca_continua_opaca() {
    let sem = pinta_sobre(&com(ph2d_bloom::Bloom::default()), FUNDO_TRANSPARENTE);
    let comb = pinta_sobre(
        &com(ligado(ph2d_bloom::BloomParams::default())),
        FUNDO_TRANSPARENTE,
    );
    let g = disco();
    let mut longe_intacto = 0usize;
    for i in 0..(LADO * LADO) as usize {
        assert!(
            comb[i * 4 + 3] >= sem[i * 4 + 3],
            "o brilho DESCOBRIU o pixel {i} ({} → {})",
            sem[i * 4 + 3],
            comb[i * 4 + 3]
        );
        if g.hit[i] {
            assert_eq!(
                comb[i * 4 + 3],
                255,
                "a peça deixou de ser opaca no pixel {i}"
            );
        }
    }
    // ⭐⭐ **E A COBERTURA CAI** — a metade que impede a cura de ser um VÉU sobre o quadro todo.
    //
    // ⚠️ A régua não é *«o canto fica em zero»*: o halo desta cadeia chega mesmo ao canto de um
    // quadro de `96` (a 1.ª redacção exigia zero e reprovou sobre a cura CERTA). É a **queda** que
    // se mede — medido: `255` junto à peça contra `1` no canto, e as barras ficam largas à volta
    // disso.
    let borda = (LADO / 2 + LADO / 6 + 2) as usize;
    let junto = comb[((LADO / 2) as usize * LADO as usize + borda) * 4 + 3];
    let canto = comb[3];
    longe_intacto += usize::from(canto <= 16);
    assert!(
        junto >= 128,
        "a cura ficou inerte: junto à peça a cobertura é só {junto}"
    );
    assert!(
        longe_intacto > 0,
        "o halo virou um VÉU: o canto do quadro ficou com cobertura {canto}"
    );
}

/// ⭐⭐ **E SEM BRILHO O FUNDO TRANSPARENTE FICA AO BIT** — a irmã da
/// [`a_omissao_e_a_imagem_de_sempre_ao_bit`], sobre o fundo que o produto usa.
///
/// ⚠️ Ela é o que impede a cura de mexer no alfa de quem nunca ligou o brilho — e o caminho de
/// omissão deste módulo é **este** fundo, não o opaco.
#[test]
fn sem_brilho_o_fundo_transparente_fica_ao_bit() {
    let base = pinta_sobre(
        &Presentation::of(ph2d_view_transform::Look::default()),
        FUNDO_TRANSPARENTE,
    );
    let campo = pinta_sobre(&com(ph2d_bloom::Bloom::default()), FUNDO_TRANSPARENTE);
    assert_eq!(
        base, campo,
        "o brilho desligado mudou o quadro transparente"
    );
}

/// ⭐⭐ **O ALFA ATRAVESSA A LEI SEM PERDER UM BYTE** — a propriedade que substituiu uma guarda.
///
/// ⛔ A 1.ª redacção da cobertura tinha um `if c > 0.0` à volta dela, com o doc a dizer que sem ele
/// *«a ida e volta pelo `f32` moveria bytes no caminho de omissão»*. Medido: ela **não move** — os
/// `256` valores voltam ao mesmo byte. ⇒ a guarda era provadamente morta, e o que ela alegava
/// proteger mora aqui, onde uma mudança de codificação (outro arredondamento, meia precisão)
/// reprova em voz alta em vez de aparecer como deriva de um byte numa imagem.
#[test]
fn o_alfa_atravessa_a_lei_sem_perder_um_byte() {
    for x in 0u8..=255 {
        let a = f32::from(x) / 255.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let volta = (0.0f32.mul_add(-a, 0.0 + a) * 255.0).ceil() as u8;
        assert_eq!(
            volta, x,
            "o alfa {x} voltou como {volta} com cobertura zero"
        );
    }
}

/// ⭐⭐⭐ **A COBERTURA É O CANAL MAIS FORTE DA LUZ, NUNCA A SOMA DELES** — e só um halo TINGIDO o
/// mede.
///
/// ⛔ Esta fixtura nasceu de uma **mutação SOBREVIVENTE**: trocar `max(r, g, b)` por `r + g + b`
/// passava a bateria inteira, porque todo o resto deste ficheiro usa um halo **branco**, onde os
/// três canais são iguais e a soma é só `3×` o máximo — e a diferença desaparece no `clamp`.
/// *Um corpus de uma cor só não testa uma lei que fala de canais.*
///
/// ⚠️ **E a lei não é uma escolha de gosto:** num quadro pré-multiplicado a cor tem de caber na
/// cobertura (`rgb ≤ a`), e `max` é a MENOR cobertura que a cumpre. Com a soma, um halo alaranjado
/// de meia força sairia **opaco** e taparia a grelha que o artista quer ver por baixo.
#[test]
fn a_cobertura_e_o_canal_mais_forte_e_nao_a_soma() {
    let pres = com(ligado(ph2d_bloom::BloomParams {
        // ⭐ Uma tinta bem desigual — é ela que separa `max` de `soma`.
        tint: [1.0, 0.25, 0.10, 1.0],
        ..ph2d_bloom::BloomParams::default()
    }));
    let comb = pinta_sobre(&pres, FUNDO_TRANSPARENTE);
    let g = disco();
    let mut medidos = 0usize;
    for i in 0..(LADO * LADO) as usize {
        if g.hit[i] {
            continue;
        }
        // ⚠️ O fundo era `[0,0,0,0]`, logo o byte guardado É a luz somada, em sRGB.
        let canais = [comb[i * 4], comb[i * 4 + 1], comb[i * 4 + 2]];
        let maior = canais
            .iter()
            .map(|&c| ph2d_color::srgb::srgb_to_linear_byte(c))
            .fold(0.0f32, f32::max);
        if maior <= 0.0 {
            continue;
        }
        medidos += 1;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let esperado = (maior.clamp(0.0, 1.0) * 255.0).ceil() as u8;
        let visto = comb[i * 4 + 3];
        // ⚠️ `±2` é a quantização dos DOIS lados (a cor volta de sRGB, a cobertura é linear) — não
        // uma folga escolhida: com a SOMA o desvio medido é de dezenas de bytes.
        assert!(
            visto.abs_diff(esperado) <= 2,
            "pixel {i}: cobertura {visto}, canal mais forte pedia {esperado}"
        );
    }
    assert!(
        medidos > 200,
        "a fixtura não contém o fenómeno: só {medidos} píxeis tingidos"
    );
}
