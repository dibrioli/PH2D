//! ⭐⭐⭐ **O DIVISOR DEPOIS DO DISPOSITIVO, e DE QUE é feito o quadro que sobra.**
//!
//! # ⚠️ Porque isto precisa de ser medido e não deduzido
//!
//! O divisor não é uma constante: ele é um **laço fechado** que divide o relógio do último traçado
//! pelos pixels dele e escolhe o maior tamanho que cabe no orçamento. ⇒ quando o traçado mudou de
//! motor, o divisor mudou de resposta **sozinho**.
//!
//! ⛔⛔ *A tabela do doc do [`super`] é de 2026-08-22 e diz `D=3` para a cena mais pesada.* Ela foi
//! medida com a CPU a fazer `121 ms`; deixá-la a governar o produto depois de a placa fazer o mesmo
//! quadro em milissegundos seria **o caminho mais lento a definir o tecto do mais rápido**
//! (`CLAUDE.md` §0.0).
//!
//! # ⭐⭐⭐ E o achado: as cenas lentas têm causas DIFERENTES, e três colunas separam-nas
//!
//! Medido a `1920×1080` (as colunas contadas valem a qualquer carga; a do relógio não):
//!
//! | cena | fita | vivos | passos/acerto | o que a torna lenta |
//! |---|---:|---:|---:|---|
//! | `2` · cubo | `31` | `8` | `16` | — (a mais barata) |
//! | `4` · perfil DESENHADO extrudado | **`2 899`** | `296` | `79` | a **fita**: um `min` por aresta do contorno |
//! | `5` · o mesmo perfil TORNEADO | **`2 972`** | **`464`** | `42` | a fita **e** a ocupação |
//! | `27` | `854` | `110` | `55` | — (cabe) |
//! | `28` · superfórmula | `766` | `34` | **`410`** | o **minorante**: o campo não é uma distância honesta |
//! | `30` | `358` | `31` | `191` | o minorante |
//!
//! ⚠️⚠️ **Três hipóteses minhas caíram antes desta tabela existir**, e cada uma parecia suficiente:
//! *«é o tamanho da fita»* (a `28` tem `766` e custa como a `4`, que tem `2 899`), *«são as
//! transcendentais»* (a `25` tem `40` e custa `13 ms`) e *«é o passo da marcha»* (a `5` e a `27` têm
//! o mesmo passo e a mesma cerca). ⇒ **a grandeza que decide é a que se conta DIRECTO** — passos por
//! acerto —, e ela só apareceu quando parei de a inferir por procuração.
//!
//! ⭐ **As duas alavancas são separadas e não se substituem:** encurtar a fita do perfil não tira um
//! passo à superfórmula, e apertar o minorante dela não tira uma instrução ao perfil.

use super::{Measured, PREVIEW_BUDGET_MS, preview_size};

const LW: u32 = 1920;
const LH: u32 = 1080;
/// O menor lado que o laço aceita — o mesmo piso que a shell passa.
const MIN: u32 = 16;

/// ⭐⭐⭐ **COM A PLACA, A MAIORIA DAS CENAS PASSA A SER NÍTIDA EM MOVIMENTO** — e a tabela diz quais
/// não, com a causa de cada uma.
///
/// # ⚠️ Porque a barra é «a maioria» e não «todas»
///
/// Exigir `D = 1` em **todas** seria afirmar uma propriedade da MÁQUINA, não da lei: numa placa
/// mais fraca a mesma árvore escolheria `D = 2` e o gate reprovaria sobre produto correto. ⇒ o que
/// ele prende é o que a wave comprou — que o quadro de movimento deixou de ser grosso **por
/// omissão** — mais o piso que impede a tabela de medir nada.
///
/// ⛔ **E as cenas que ficam grossas não são uma lista escrita à mão:** elas saem da medição, com as
/// três colunas que dizem **porquê**. Ver a nota do módulo.
#[test]
#[ignore = "precisa de GPU"]
fn com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];

    println!(
        "\n  cena · quadro de MOVIMENTO a {LW}×{LH} · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let mut medidas = 0;
    let mut grossas = Vec::new();
    for n in 0..crate::smoke::scenes::CENAS {
        if crate::smoke::scenes::PODADAS.contains(&n) {
            continue;
        }
        let doc = crate::smoke::scene(n);
        let reg = crate::smoke::sampled_registry();
        let cam = ph2d_field_render::Orbit::default();
        let luz = [crate::gpu_frame::tests_lampada(&cam)];
        let surfaces = ph2d_field_render::Surfaces {
            all: &materiais,
            owners: None,
        };
        // ⚠️ **`antialias = false`** — é o quadro de MOVIMENTO, e a lei da W73 manda-o saltar a
        // borda re-amostrada. Medir o assente aqui daria um número que este laço nunca vê.
        // ⚠️ **COM o tecto** — esta tabela mede o que o PRODUTO faz, e o produto tem a cerca.
        let Some(_) = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false,
        ) else {
            continue;
        };
        // A segunda corrida é a que conta: a primeira compila o pipeline e sobe a grade.
        let t0 = std::time::Instant::now();
        let _ = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false,
        )
        .expect("o pintor");
        #[allow(clippy::cast_possible_truncation)]
        let millis = t0.elapsed().as_secs_f32() * 1e3;
        // ⭐⭐⭐ **E QUANTO DISSO É A FITA, que se compila na CPU a cada quadro.** Sem esta coluna a
        // tabela diz «a placa é lenta nesta peça» sobre um custo que a placa não paga.
        let t1 = std::time::Instant::now();
        let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        #[allow(clippy::cast_possible_truncation)]
        let _fita_ms = t1.elapsed().as_secs_f32() * 1e3;
        let m = Measured {
            pixels: u64::from(LW) * u64::from(LH),
            millis,
        };
        let (w, _) = preview_size((LW, LH), Some(m), PREVIEW_BUDGET_MS, MIN);
        let d = LW / w.max(1);
        medidas += 1;
        // ⭐ **O TAMANHO da fita** — uma linha por instrução, e é ela que o shader corre por passo
        // de marcha. É a coluna que diz se o custo é da PEÇA ou do passe.
        let instrs = fita.source.lines().count();
        // ⭐⭐⭐ **De QUE a fita é feita** — as transcendentais custam muito mais que um `min` numa
        // placa, e uma tabela que conta só instruções diz que duas peças iguais custam o mesmo.
        let caras = fita
            .source
            .lines()
            .filter(|l| {
                [
                    "sin(", "cos(", "tan(", "asin(", "acos(", "atan", "exp(", "log(", "pow(",
                ]
                .iter()
                .any(|f| l.contains(f))
            })
            .count();
        let raizes = fita.source.matches("sqrt(").count();
        // ⭐⭐⭐ **O PASSO e o ORÇAMENTO** — a marcha anda `d · passo` por iteração, logo um passo
        // pequeno é mais iterações **por raio**. Uma tabela que só conta instruções diz que duas
        // peças com a mesma fita custam o mesmo, e elas não custam.
        // ⭐⭐⭐ **OS VALORES VIVOS AO MESMO TEMPO** — o scratch por thread. Numa placa é ele que
        // decide a OCUPAÇÃO (quantas threads cabem num multiprocessador), e uma fita larga põe o
        // compilador a **derramar** registos para memória. *Uma tabela que conta instruções não vê
        // isso, e duas peças com o mesmo número de instruções não custam o mesmo.*
        let vivos = campo.tape_shape().map_or(0, |t| t.vivos);
        // ⭐⭐⭐ **QUANTAS VEZES CADA RAIO AVALIA O CAMPO** — a grandeza directa, e a única que não é
        // uma procuração. Um campo que é um mau minorante de distância faz TODO raio rastejar, e
        // isso não aparece em contagem de instruções nenhuma.
        //
        // ⚠️ Medida na CPU (o contador é dela) sobre uma tela pequena: o que se quer é a RAZÃO
        // amostras/pixel, que não depende do tamanho.
        use std::sync::atomic::Ordering;
        ph2d_field_render::STEP_SAMPLES.store(0, Ordering::Relaxed);
        let g = ph2d_field_render::trace(&doc, &reg, &cam, 96, 54);
        let acertos = g.hit.iter().filter(|h| **h).count().max(1);
        #[allow(clippy::cast_precision_loss)]
        let por_acerto =
            ph2d_field_render::STEP_SAMPLES.load(Ordering::Relaxed) as f32 / acertos as f32;
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let orcamento = ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
            / passo.clamp(f32::EPSILON, 1.0))
        .ceil() as u32;
        // ⚠️ O passo e a cerca ficam medidos e **fora da tabela**: as três hipóteses que eles
        // encarnavam caíram (ver a nota do módulo), e deixá-los na coluna sugeriria que explicam.
        let _ = (passo, orcamento);
        println!(
            "  {n:>4} · {millis:7.2} ms · {instrs:>5} instr · {caras:>3} transc · \
             {raizes:>3} sqrt · {vivos:>4} vivos · {por_acerto:6.1} passos/acerto · D={d}"
        );
        if d > 1 {
            grossas.push((n, millis, d));
        }
    }
    assert!(medidas > 10, "só {medidas} cenas foram medidas");
    // ⭐ **A barra é a MAIORIA**, e ela prende o que a wave comprou: antes do dispositivo, a cena
    // mais pesada custava `121 ms` na CPU e **toda** cena de tamanho real ia a `D = 3`.
    let nitidas = medidas - grossas.len();
    println!(
        "  ⇒ {nitidas} de {medidas} cenas correm NÍTIDAS em movimento · orçamento {PREVIEW_BUDGET_MS} ms"
    );
    assert!(
        nitidas * 2 > medidas,
        "só {nitidas} de {medidas} cenas são nítidas em movimento — estas pedem um quadro grosso: \
         {grossas:?}"
    );
}

/// ⏱️⭐⭐⭐ **O PREÇO DE UMA ARESTA DE PERFIL, NA PLACA** — a medição que decide se vale a pena
/// tirar o contorno da fita.
///
/// # ⚠️ Porque ela vem ANTES de qualquer construção
///
/// A cura candidata é a que a CPU já usa: o contorno deixa de ser uma cadeia de `min` desenrolada e
/// passa a ser uma **consulta** ([`ph2d_field_eval`], `profile_index`). Ali ela vale `155 ns → 40`,
/// isto é `3,9×`. ⛔ Mas esse número é de CPU-contra-CPU, e a placa tem um segundo problema que a
/// CPU não tem: **a ocupação** — `2 972` instruções desenroladas deixam `464` valores vivos, e um
/// multiprocessador que só cabe com poucos fios corre a passo.
///
/// ⇒ o que esta sonda mede é a **INCLINAÇÃO**: quanto custa cada aresta a mais, na placa. Se ela for
/// linear e íngreme, a cura vale o que a CPU diz e mais; se ela saturar, o custo é outro e a cura
/// seria construída contra a grandeza errada.
#[test]
#[ignore = "medição — precisa de GPU"]
fn mede_o_preco_de_uma_aresta_de_perfil() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];

    /// Um polígono regular de `n` lados, como contorno — o perfil mais limpo que isola a variável.
    ///
    /// ⚠️ **Regular de propósito:** um contorno desenhado à mão mistura o número de arestas com a
    /// forma delas, e a pergunta aqui é só sobre o número.
    fn anel(n: u32) -> ph2d_field::Profile {
        let pts = (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                // ⚠️ Um raio que ONDULA: um círculo perfeito deixa o compilador dobrar arestas
                // iguais, e a sonda mediria um polígono que ninguém desenha.
                let r = 0.35 + 0.06 * (a * 3.0).cos();
                [r * a.cos(), r * a.sin()]
            })
            .collect();
        ph2d_field::Profile::new(vec![pts], ph2d_field::FillRule::NonZero, 1.0e-4).expect("o anel")
    }

    println!(
        "\n  arestas · instruções · vivos · quadro a 1920×1080 · load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    let mut pontos: Vec<(u32, usize, usize, f32)> = Vec::new();
    for n in [32u32, 64, 128, 136, 144, 152, 160, 192, 256] {
        let doc = ph2d_field::FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                ph2d_field::Primitive::Extrude {
                    profile: anel(n),
                    half_height: 0.25,
                    round: 0.0,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::IDENTITY,
            )],
            ph2d_field::NodeId(0),
        )
        .expect("a peça extrudada");

        let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        let instrs = fita.source.lines().count();
        let vivos = campo.tape_shape().map_or(0, |s| s.vivos);

        // Aquecimento fora da conta: a primeira compila o pipeline.
        // ⚠️ **Sem o tecto** — ver [`crate::gpu_frame::paint_com_tecto`]: esta é a sonda que o
        // calibra, e ela tem de atravessar o degrau para o poder ver.
        let _ = crate::gpu_frame::paint_com_tecto(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, false,
        );
        let mut v: Vec<f32> = Vec::new();
        for _ in 0..5 {
            let t0 = std::time::Instant::now();
            let p = crate::gpu_frame::paint_com_tecto(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, false,
            )
            .expect("o pintor");
            std::hint::black_box(p.rgba.len());
            #[allow(clippy::cast_possible_truncation)]
            v.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        v.sort_by(f32::total_cmp);
        let ms = v[0];
        // ⛔⛔ **E O MESMO QUADRO NA CPU** — a pergunta que esta sonda tem de responder não é «quanto
        // custa na placa», é **«a placa ficou mais lenta que o caminho que ela substituiu?»**. Uma
        // tabela só com a coluna nova não sabe dizer se a wave foi uma regressão.
        // ⚠️⚠️ **A coluna da CPU SAIU, e a razão fica escrita:** a máquina esteve a `load 80–100` a
        // jornada inteira (outra linha a correr a suíte dela), e ali o mesmo traçado leu `71` e
        // `482 ms`. *Uma régua que varia `7×` entre corridas do mesmo código não mede código.*
        //
        // ⭐ O que fica é a coluna que NÃO depende da carga: o custo **por aresta** na placa. Ele é
        // plano enquanto a fita cabe nos registos e cresce quando ela deixa de caber — e é essa
        // quebra, e não uma comparação, que nomeia o recurso.
        #[allow(clippy::cast_precision_loss)]
        let por_aresta = ms / n as f32;
        println!(
            "  {n:>7} · {instrs:>10} · {vivos:>5} · {ms:>8.2} ms · {por_aresta:>6.3} ms/aresta"
        );
        pontos.push((n, instrs, vivos, ms));
    }

    // ⭐ **A inclinação, entre os dois extremos** — o número que decide a wave.
    let (n0, _, _, ms0) = pontos[0];
    let (n1, _, _, ms1) = *pontos.last().expect("há pontos");
    println!(
        "  ⇒ de {n0} para {n1} arestas: {:.1}× o relógio · {:.4} ms por aresta",
        ms1 / ms0,
        (ms1 - ms0) / (n1 - n0) as f32
    );
    assert!(
        ms1 > ms0,
        "o relógio não cresceu com as arestas ({ms0:.2} → {ms1:.2}) — a sonda não isolou a variável"
    );
}

/// ⛔⛔⛔ **UMA FITA LARGA DEMAIS FICA NA CPU** — a cerca que impede esta linha de piorar a peça
/// desenhada do artista.
///
/// # ⚠️ O que ela afirma, e porque o CONTROLO vem primeiro
///
/// Acima do degrau da ocupação ([`ph2d_field_gpu::MAX_VIVOS`]) o dispositivo mede **`0,20×`** a CPU
/// — cinco vezes pior. ⇒ o quadro cai para a CPU, que é linear em toda a faixa.
///
/// ⛔ **Sem o controlo, um `return None` constante passaria este gate** e o dispositivo deixaria de
/// desenhar seja o que for — a forma mais cara de um gate verde.
#[test]
#[ignore = "precisa de GPU"]
fn uma_fita_larga_demais_fica_na_cpu() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];

    // ⚠️ **Os dois lados do degrau saem da MESMA família**, e é isso que faz a comparação valer:
    // trocar também a forma mediria duas coisas ao mesmo tempo.
    let peca = |n: u32| {
        let pts = (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                let r = 0.35 + 0.06 * (a * 3.0).cos();
                [r * a.cos(), r * a.sin()]
            })
            .collect();
        ph2d_field::FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                ph2d_field::Primitive::Extrude {
                    profile: ph2d_field::Profile::new(
                        vec![pts],
                        ph2d_field::FillRule::NonZero,
                        1.0e-4,
                    )
                    .expect("o anel"),
                    half_height: 0.25,
                    round: 0.0,
                    chamfer: 0.0,
                },
                ph2d_field::Xform::IDENTITY,
            )],
            ph2d_field::NodeId(0),
        )
        .expect("a peça extrudada")
    };
    let vivos = |doc: &ph2d_field::FieldDoc| {
        ph2d_field_eval::device::DeviceField::new(doc, &reg)
            .and_then(|c| c.tape_shape())
            .map_or(0, |s| s.vivos)
    };
    let toma = |doc: &ph2d_field::FieldDoc| {
        crate::gpu_frame::paint(
            t, doc, &reg, &cam, &luz, &surfaces, olhar, BG, 96, 54, false,
        )
        .is_some()
    };

    let (leve, pesada) = (peca(64), peca(256));
    let (vl, vp) = (vivos(&leve), vivos(&pesada));
    println!(
        "  leve {vl} vivos · pesada {vp} vivos · tecto {}",
        ph2d_field_gpu::MAX_VIVOS
    );
    // ⭐ **A fixtura tem de estar dos DOIS lados do degrau** — senão o gate compara duas peças do
    // mesmo lado e passa sem afirmar nada.
    assert!(
        vl <= ph2d_field_gpu::MAX_VIVOS && vp > ph2d_field_gpu::MAX_VIVOS,
        "a fixtura não cerca o tecto: leve {vl}, pesada {vp}, tecto {}",
        ph2d_field_gpu::MAX_VIVOS
    );
    // ⭐ O CONTROLO primeiro.
    assert!(
        toma(&leve),
        "o dispositivo recusou a peça LEVE — a cerca comeu tudo"
    );
    assert!(
        !toma(&pesada),
        "o dispositivo tomou a peça PESADA — acima do degrau ele mede 0,20× a CPU"
    );
}
