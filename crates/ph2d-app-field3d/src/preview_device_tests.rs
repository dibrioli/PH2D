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
//!
//! # ⛔⛔ E a coluna `vivos` desta tabela é HISTÓRICA (2026-09-14)
//!
//! Ela foi medida antes do [`ph2d_field_eval::tape_schedule`], e os `296`/`464` dela eram uma
//! propriedade da **ordem de emissão** da fita, não das peças. Hoje as mesmas cenas medem `69` e
//! `30`, e a pior das 17 mede `89`. ⇒ *a linha «a fita **e** a ocupação» descreve um programa que já
//! não existe* — a ocupação deixou de ser uma das causas, e o que sobra do perfil é o **tamanho**
//! da fita. A tabela fica porque é o diagnóstico que separou as causas; `docs/Render3d/05` §43 tem
//! a que a substituiu.

use super::{Measured, PREVIEW_BUDGET_MS, preview_size};

const LW: u32 = 1920;
const LH: u32 = 1080;
/// O menor lado que o laço aceita — o mesmo piso que a shell passa.
const MIN: u32 = 16;
/// O fundo que as duas sondas usam.
const FUNDO: [u8; 4] = [0, 0, 0, 0];

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

/// ⭐⭐⭐ **O QUADRO INTEIRO NA CPU** — o que o dispositivo de facto SUBSTITUI.
///
/// ⛔⛔ **A 1.ª redacção destas sondas media só o `trace` do lado da CPU** e o quadro **PINTADO** do
/// lado do dispositivo. ⇒ pedia-se à placa o G-buffer **mais** a sombra, o sombreamento e as bordas,
/// e à CPU só o G-buffer — *a travessia saía cedo demais, e o tecto derivado dela era conservador
/// pelo motivo errado*.
///
/// O quadro de CPU do produto são **três** passos ([`crate::smoke_draw_thread`]): traçar, a sombra
/// directa, e pintar. É esse que se mede aqui. ⚠️ Do lado do dispositivo a sombra sai da MESMA
/// marcha, e é por isso que ela não aparece lá como um passo separado.
fn quadro_na_cpu(
    doc: &ph2d_field::FieldDoc,
    reg: &ph2d_field_eval::hybrid::Registry,
    cam: &ph2d_field_render::Orbit,
    luz: &[ph2d_field_render::PointLamp],
    surfaces: &ph2d_field_render::Surfaces<'_>,
    olhar: ph2d_view_transform::Look,
) -> usize {
    let mundos: Vec<[f32; 3]> = luz.iter().map(|l| l.world).collect();
    let g = ph2d_field_render::trace(doc, reg, cam, LW, LH);
    let sh = ph2d_field_render::shadow_pass(doc, reg, cam, &g, &mundos);
    let sem_ecra: [ph2d_field_render::Lamp; 0] = [];
    ph2d_field_render::shade_render(
        &g,
        cam,
        surfaces,
        &ph2d_field_render::Lighting {
            lamps: &sem_ecra,
            points: luz,
            sky: &crate::render_light::StudioSky,
            shadows: Some(&sh),
        },
        olhar,
        FUNDO,
    )
    .len()
}

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

    // ⚠️⚠️ **ESTE GATE DIVIDE UM RELÓGIO POR UM ORÇAMENTO, logo é da família das flakes de carga**
    // (`CLAUDE.md` §5.0). Medido 2026-09-15: com a CPU a **`0 %`** ociosa ele lê `2 de 18` e com a
    // máquina livre lê a mesma cena `0` a `11,5` em vez de `19,0 ms` — *quase todas as cenas caem
    // logo ACIMA do orçamento, que é a assinatura de um abrandamento uniforme e não de uma
    // regressão*. ⇒ ele imprime a **ociosidade**, e quem o lê confere-a antes de acusar um diff.
    println!(
        "\n  cena · quadro de MOVIMENTO a {LW}×{LH} · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    let mut medidas = 0;
    let mut na_cpu = 0usize;
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
        //
        // ⛔⛔ **E uma cena recusada é IMPRESSA, não saltada.** A primeira redacção fazia
        // `continue` e o denominador do gate encolhia em silêncio: a cena `5` saiu da tabela no dia
        // em que o tecto desceu, e a linha *«13 de 17»* leu-se como se a peça tivesse melhorado.
        // *Uma população que muda por baixo de uma razão é a catraca que vira licença.*
        let Some(_) = crate::gpu_frame::paint(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false,
        ) else {
            let guardados = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
                .and_then(|c| c.tape_shape())
                .map_or(0, |s| s.guardados);
            println!("  {n:>4} ·        na CPU · {guardados:>5} guardados");
            na_cpu += 1;
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
        "  ⇒ {nitidas} de {medidas} cenas do dispositivo correm NÍTIDAS em movimento · \
         {na_cpu} ficam na CPU · orçamento {PREVIEW_BUDGET_MS} ms"
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
/// ⭐⭐ **A FRACÇÃO DE CPU OCIOSA** — a régua da calma que o `loadavg` não é.
///
/// ⛔⛔ Medido 2026-09-15: esta máquina fica em `loadavg 5`–`7` com a CPU a **`94 %` ociosa** (o
/// `loadavg` do Linux conta também quem espera por I/O, e a média decai devagar depois de uma
/// suíte). ⇒ um limiar sobre ele **recusa uma máquina calma**, que é o modo de falha mais caro de
/// uma régua de calma: ela lê-se exactamente como *«a máquina nunca acalmou»*.
///
/// ⚠️ Ela **imprime-se**, não decide: quem lê a tabela é quem julga se o relógio vale.
fn cpu_ociosa_pct() -> f32 {
    let campos = || -> Option<(u64, u64)> {
        let s = std::fs::read_to_string("/proc/stat").ok()?;
        let l = s.lines().next()?;
        let v: Vec<u64> = l
            .split_whitespace()
            .skip(1)
            .filter_map(|x| x.parse().ok())
            .collect();
        // `idle` é o 4.º campo e `iowait` o 5.º — os dois contam como «não a trabalhar».
        Some((
            v.iter().sum(),
            v.get(3).copied()? + v.get(4).copied().unwrap_or(0),
        ))
    };
    let Some((t0, i0)) = campos() else {
        return f32::NAN;
    };
    std::thread::sleep(std::time::Duration::from_millis(300));
    let Some((t1, i1)) = campos() else {
        return f32::NAN;
    };
    #[allow(clippy::cast_precision_loss)]
    let (dt, di) = ((t1 - t0) as f32, (i1 - i0) as f32);
    if dt <= 0.0 { f32::NAN } else { di / dt * 100.0 }
}

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

    println!(
        "\n  arestas · guardados · vivos CRU→ESC · quadro CRU→ESC a 1920×1080 · CPU · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    let mut pontos: Vec<(u32, usize, usize, f32)> = Vec::new();
    for n in [32u32, 64, 96, 128, 192, 256, 384] {
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

        let retrato = |escalonar: bool| {
            let campo = ph2d_field_eval::device::DeviceField::new_com(&doc, &reg, escalonar)
                .expect("a peça");
            let forma = campo.tape_shape().expect("a forma");
            (
                campo.tape_wgsl().expect("a fita").source.lines().count(),
                forma.vivos,
                forma.guardados,
            )
        };
        let ((instrs, vivos, guardados), (_, vivos_cru, _)) = (retrato(true), retrato(false));

        // ⭐⭐⭐ **O MESMO QUADRO NAS DUAS ORDENS DA MESMA FITA** — ver
        // [`ph2d_field_eval::tape_schedule`]. ⚠️ **A/B na MESMA corrida e na mesma máquina**: a
        // coluna crua é a que diz se o escalonador comprou relógio, e compará-la com um número
        // escrito noutro dia mediria a carga daquele dia.
        let mede = |escalonar: bool| {
            let sonda = crate::gpu_frame::Sonda { escalonar };
            // Aquecimento fora da conta: a primeira compila o pipeline. ⚠️ **Sem o tecto** — esta
            // é a sonda que o calibra, e ela tem de atravessar o degrau para o poder ver.
            let _ = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
            );
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let p = crate::gpu_frame::paint_com(
                    t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
                )
                .expect("o pintor");
                std::hint::black_box(p.rgba.len());
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let ms = mede(true);
        let ms_cru = mede(false);
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
        // ⭐⭐⭐ **E A TRAVESSIA COM A CPU** — o número que o §42 declarou não-medível.
        //
        // ⚠️⚠️ **Ele só vale com a máquina CALMA, e a linha imprime a carga ao lado.** A mesma
        // medição a `load 91` leu `71` e `482 ms` para o mesmo traçado: *uma régua que varia `7×`
        // entre corridas do mesmo código não mede código.* ⇒ quem ler esta coluna lê primeiro o
        // `load` do cabeçalho — senão está a medir quem mais está a usar a máquina.
        let mut c: Vec<f32> = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            // ⚠️ **O QUADRO INTEIRO** — ver [`quadro_na_cpu`]. Medir só o traçado aqui pedia à
            // placa mais trabalho do que à CPU.
            std::hint::black_box(quadro_na_cpu(&doc, &reg, &cam, &luz, &surfaces, olhar));
            #[allow(clippy::cast_possible_truncation)]
            c.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        c.sort_by(f32::total_cmp);
        println!(
            "  {n:>7} · {guardados:>9} · {vivos_cru:>5} → {vivos:>4} · {ms_cru:>8.2} → {ms:>7.2} ms \
             ({:>4.1}×) · {por_aresta:>6.3} ms/aresta · CPU {:>8.2} · {:>5.2}×",
            ms_cru / ms,
            c[0],
            c[0] / ms
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

/// ⭐⭐⭐ **A PLACA GANHA EM TODA A FAIXA MEDÍVEL** — a propriedade que substituiu um TECTO.
///
/// # ⛔⛔⛔ O que estava aqui antes, e porquê saiu
///
/// Aqui vivia a `uma_fita_larga_demais_fica_na_cpu`: a cerca que mandava para a CPU qualquer peça
/// acima de um tecto (`MAX_VIVOS = 743`, depois `358`, depois `MAX_GUARDADOS = 3 463`). Os três
/// números saíram da MESMA sonda, e ela pedia aos dois motores trabalhos **diferentes** — o quadro
/// pintado inteiro à placa, só o traçado à CPU — com o traçador e o material a compilar a `opt-0`.
///
/// ⭐ Curadas as duas metades, a placa ganha em toda a faixa (`1,3×`–`7,8×`, com um empate no
/// penhasco de `192` arestas) e nas 17 cenas do produto (`2,6×`–`98×`). ⇒ *uma cerca que nunca pode
/// disparar não é uma cerca.* Tabelas: a nota no lugar do `MAX_GUARDADOS`, na [`ph2d_field_gpu`].
///
/// # ⚠️ O que este gate afirma
///
/// A PROPRIEDADE que o tecto tentava codificar num número: **a placa não perde de forma
/// significativa, e ganha no contorno mais largo que a sonda mede.** ⛔ Ele é deliberadamente
/// FROUXO no pior ponto (o penhasco lê `0,96×`–`1,04×`, um empate) e exigente no mais largo —
/// *um gate que exigisse vitória em todo ponto reprovaria sobre produto correto*.
///
/// ⚠️ **Ele lê relógios, logo é da família das flakes de carga** (`CLAUDE.md` §5.0): corre com
/// `#[ignore]`, imprime a carga **e a CPU ociosa**, e quem o lê julga se o número vale.
#[test]
#[ignore = "precisa de GPU — e lê relógio: leia a ociosidade ao lado"]
fn a_placa_ganha_em_toda_a_faixa_medivel() {
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

    println!(
        "\n  arestas · placa · CPU · razão · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
    let mut pior = f32::INFINITY;
    let mut mais_largo = 0.0f32;
    for n in [64u32, 128, 192, 256] {
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
        let sonda = crate::gpu_frame::Sonda { escalonar: true };
        // Aquecimento fora da conta: a primeira compila o pipeline.
        let _ = crate::gpu_frame::paint_com(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, LW, LH, false, sonda,
        );
        let mut v: Vec<f32> = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let p = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, FUNDO, LW, LH, false, sonda,
            )
            .expect("o pintor");
            std::hint::black_box(p.rgba.len());
            #[allow(clippy::cast_possible_truncation)]
            v.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        v.sort_by(f32::total_cmp);
        let mut c: Vec<f32> = Vec::new();
        for _ in 0..2 {
            let t0 = std::time::Instant::now();
            std::hint::black_box(quadro_na_cpu(&doc, &reg, &cam, &luz, &surfaces, olhar));
            #[allow(clippy::cast_possible_truncation)]
            c.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        c.sort_by(f32::total_cmp);
        let razao = c[0] / v[0];
        println!("  {n:>7} · {:>8.2} · {:>8.2} · {razao:>5.2}×", v[0], c[0]);
        pior = pior.min(razao);
        mais_largo = razao;
    }
    // ⭐ **O CONTROLO primeiro:** sem ele, um `paint_com` que devolvesse sempre a mesma imagem
    // vazia em microssegundos passaria com razões enormes.
    assert!(
        mais_largo.is_finite() && pior.is_finite(),
        "a sonda não mediu nada"
    );
    assert!(
        pior > 0.85,
        "a placa perdeu por mais do que o penhasco medido: pior razão {pior:.2}×"
    );
    assert!(
        mais_largo > 1.2,
        "no contorno mais largo a placa tem de ganhar com margem: {mais_largo:.2}×"
    );
}

/// ⭐⭐⭐ **AS PEÇAS REAIS NOS DOIS MOTORES** — a sonda que impede o tecto de ser calibrado numa
/// família e aplicado a outra.
///
/// # ⛔⛔ Por que ela existe
///
/// O [`ph2d_field_gpu::MAX_GUARDADOS`] sai de um **polígono extrudado de N arestas**, que é a
/// família que o `+ Extrude` produz e a única em que se pode varrer uma variável só. ⚠️ Mas quem o
/// tecto decide são as **cenas do produto**, e essas têm furos, aros arredondados, booleanas e
/// torno — *um tecto calibrado numa família e aplicado a outra é uma procuração*.
///
/// ⇒ esta sonda mede, cena a cena, o quadro nos DOIS motores, e imprime de que lado do tecto cada
/// uma cai. Uma cena cuja razão seja `> 1` e que o tecto RECUSE é um defeito de calibração —
/// e uma que ele aceite com razão `< 1` é o defeito que ele existe para impedir.
#[test]
#[ignore = "medição — precisa de GPU"]
fn mede_as_cenas_reais_nos_dois_motores() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  cena · guardados · placa · CPU · razão · load {} · ociosa {:.0} %",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim(),
        cpu_ociosa_pct()
    );
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
        let Some(guardados) = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
            .and_then(|c| c.tape_shape())
            .map(|s| s.guardados)
        else {
            continue;
        };
        // ⚠️ **SEM o tecto** — esta é a sonda que o calibra.
        let sonda = crate::gpu_frame::Sonda { escalonar: true };
        let mede = |f: &dyn Fn()| {
            let mut v: Vec<f32> = Vec::new();
            for _ in 0..3 {
                let t0 = std::time::Instant::now();
                f();
                #[allow(clippy::cast_possible_truncation)]
                v.push(t0.elapsed().as_secs_f32() * 1e3);
            }
            v.sort_by(f32::total_cmp);
            v[0]
        };
        let na_placa = || {
            if let Some(p) = crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
            ) {
                std::hint::black_box(p.rgba.len());
            }
        };
        if crate::gpu_frame::paint_com(
            t, &doc, &reg, &cam, &luz, &surfaces, olhar, BG, LW, LH, false, sonda,
        )
        .is_none()
        {
            println!("  {n:>4} · {guardados:>9} · (a placa recusa)");
            continue;
        }
        let ms_gpu = mede(&na_placa);
        let na_cpu = || {
            std::hint::black_box(quadro_na_cpu(&doc, &reg, &cam, &luz, &surfaces, olhar));
        };
        let ms_cpu = mede(&na_cpu);
        println!(
            "  {n:>4} · {guardados:>9} · {ms_gpu:>8.2} · {ms_cpu:>8.2} · {:>5.2}×",
            ms_cpu / ms_gpu
        );
    }
}
