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

pub(super) const LW: u32 = 1920;
pub(super) const LH: u32 = 1080;
/// O menor lado que o laço aceita — o mesmo piso que a shell passa.
const MIN: u32 = 16;

/// ⭐⭐⭐ **QUANTOS QUADROS A RÉGUA MEDE, e fica com o MÍNIMO.**
///
/// A [`W9`](../../../docs/Render3d/03_o_plano.md) prescreve *«`1920×1080`, **mínimo de N**, A/B
/// intercalado no MESMO processo»*, e até 2026-09-21 este ficheiro cronometrava **uma** chamada.
///
/// ⚠️ **O `3` é o joelho MEDIDO e não um número escolhido:** a 1.ª chamada de uma cena nova paga a
/// compilação do programa da placa (`1,4`–`4,4 s` antes da cura da fita inerte) e a 2.ª já é
/// regime; a 3.ª existe porque, numa janela de calma real, a 2.ª ainda apanhou picos isolados
/// (cena `24` leu `176,62` e depois `9,84 ms`). *Mais do que três paga relógio sem mover a
/// mediana.*
const QUADROS_MEDIDOS: usize = 3;
/// O fundo que as duas sondas usam.
pub(super) const FUNDO: [u8; 4] = [0, 0, 0, 0];

/// Um polígono regular de `n` lados, como contorno — o perfil mais limpo que isola a variável.
///
/// ⚠️ **Regular de propósito:** um contorno desenhado à mão mistura o número de arestas com a
/// forma delas, e a pergunta aqui é só sobre o número.
pub(super) fn anel(n: u32) -> ph2d_field::Profile {
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
pub(super) fn quadro_na_cpu(
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
        &ph2d_field_render::Presentation::of(olhar),
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
///
/// # ⛔⛔⛔ COMO LER UMA REPROVAÇÃO DESTE GATE (as SETE leituras de 2026-09-19)
///
/// O MESMO commit, medido sete vezes no mesmo dia:
///
/// | corrida | perfil | contexto | veredito |
/// |---|---|---|---|
/// | conjunto `--ignored` inteiro | release | `load 47` (outra linha a compilar) | `11 de 22` ✗ |
/// | sozinha | **debug** | `ociosa 68 %` | `8 de 22` ✗ |
/// | sozinha | **debug** | `ociosa 81 %` | `10 de 22` ✗ |
/// | sozinha | **debug** | `ociosa 98 %` | `9 de 22` ✗ |
/// | conjunto inteiro, máquina calma | release | `load 7` | **`22` testes, `0` ✗** ✓ |
/// | sozinha, logo a seguir ao próprio build | release | `load 20,68` · `ociosa 92 %` | `10 de 22` ✗ |
/// | conjunto inteiro, com a linha do lado a compilar | release | **`load 130,26`** | `9 de 22` ✗ |
///
/// ⭐⭐⭐ **Nenhuma das duas réguas de calma discrimina sozinha:** `ociosa 92 %` reprovou e `load 7`
/// passou. A ociosidade é **instantânea** e lida no arranque do laço; o que atrasa as cenas é a
/// **cauda** do que acabou de correr — o compilador de release, a suíte da linha do lado — e é a
/// `loadavg` que ainda a vê. ⇒ *não corra este gate a seguir a um build; corra-o com a `loadavg`
/// abaixo de ~10.*
///
/// ⚠️ **E a assinatura da carga lê-se na TABELA por cena:** a `load 130` quase todas caem **logo
/// acima** do orçamento (`17,4`–`19,2 ms` contra `16,7`), que é um abrandamento uniforme. Uma
/// regressão de lei não move dez cenas para `1,05×` a barra ao mesmo tempo.
///
/// ⚠️ E a tabela por cena **imita a assinatura da flake sem o ser** quando o perfil muda: a cena de
/// `93` instruções mede `14 ms` em debug e `58 ms` numa release contendida, e outra de `934` faz o
/// contrário. *Compare cabeçalhos de log antes de comparar números.*
#[test]
#[ignore = "precisa de GPU — e lê relógio: leia o contexto ao lado antes de acusar um diff"]
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
    // regressão*. ⇒ ele imprime o [`contexto`], e quem o lê confere-o antes de acusar um diff.
    //
    // ⛔⛔⛔ **E a primeira coluna desse contexto é o PERFIL** (2026-09-19): três corridas em debug
    // leram `8`, `10` e `9 de 22`, a mais calma delas a `98 %` de CPU ociosa, sobre o mesmo commit
    // que a corrida de `--release` mede de outra maneira. *Uma leitura deste gate feita em debug
    // não é uma leitura deste gate.*
    println!(
        "\n  cena · quadro de MOVIMENTO a {LW}×{LH} · mínimo de {QUADROS_MEDIDOS} · {}",
        contexto()
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
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            None,
            LW,
            LH,
            false,
        ) else {
            let guardados = ph2d_field_eval::device::DeviceField::new(&doc, &reg)
                .and_then(|c| c.tape_shape())
                .map_or(0, |s| s.guardados);
            println!("  {n:>4} ·        na CPU · {guardados:>5} guardados");
            na_cpu += 1;
            continue;
        };
        // A segunda corrida é a que conta: a primeira compila o pipeline e sobe a grade.
        // ⭐⭐⭐⭐ **O MÍNIMO DE `N`, que é a régua que a `W9` prescreve por escrito** (`docs/Render3d/03`:
        // *«`1920×1080`, **mínimo de N**, A/B intercalado no MESMO processo»*). ⛔⛔ Até 2026-09-21
        // este gate cronometrava **UMA** chamada — e uma chamada de uma cena NOVA paga a compilação
        // do programa da placa, medida entre `1,4` e `4,4 s`, mais o que o escalonador der.
        //
        // ⚠️⚠️ **O custo disso está medido e é o motivo deste bloco:** numa janela de calma REAL
        // (`99`–`100 %` de CPU ociosa) cenas individuais leram-se até **`11,7×`** diferentes entre
        // duas corridas do MESMO binário, e o veredito do gate moveu-se de `8` para `10`–`12 de 22`
        // conforme a régua. *O artista arrasta o quadro N-ésimo, nunca o primeiro.*
        //
        // ⭐ **E a 1.ª chamada fica na tabela**, não escondida: ela é um preço REAL (o dono
        // reprovou-o em 2026-09-21) e uma régua que o apaga faz uma cura desaparecer com ele.
        let mut tempos = Vec::with_capacity(QUADROS_MEDIDOS);
        for _ in 0..QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let _ = crate::gpu_frame::paint(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                BG,
                None,
                LW,
                LH,
                false,
            )
            .expect("o pintor");
            #[allow(clippy::cast_possible_truncation)]
            tempos.push(t0.elapsed().as_secs_f32() * 1e3);
        }
        let primeiro = tempos[0];
        let millis = tempos.iter().copied().fold(f32::INFINITY, f32::min);
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
            "  {n:>4} · {millis:7.2} ms · 1.ª {primeiro:8.2} · {instrs:>5} instr · \
             {caras:>3} transc · {raizes:>3} sqrt · {vivos:>4} vivos · \
             {por_acerto:6.1} passos/acerto · D={d}"
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
pub(super) fn cpu_ociosa_pct() -> f32 {
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

/// ⭐⭐⭐ **O CONTEXTO DE UMA MEDIÇÃO DE RELÓGIO — a carga, a ociosidade e o PERFIL DE BUILD.**
///
/// ⛔⛔⛔ **O perfil está aqui porque ele DOMINA, e uma régua de calma que não o nomeia mente**
/// (medido 2026-09-19): três corridas seguidas do
/// [`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`] leram `8`, `10` e `9 de 22` — a
/// última com a CPU a **`98 %` ociosa**, que é a máquina mais calma das quatro medições daquele dia
/// —, e as três correram em **debug**, porque ao comando faltava `--release`. *A leitura mais calma
/// foi a pior, e a coluna que a explicava não estava na tabela.*
///
/// ⚠️ A assinatura do erro é o que se lê por cena: a MESMA cena de `93` instruções mediu `14 ms` nas
/// três corridas de debug e `58 ms` na de release, enquanto outra de `934` instruções fez o
/// contrário — *nenhum deslocamento uniforme, que é o que um custo novo no shader produziria*.
///
/// ⚠️ **Ela é uma PORTA e não um idioma:** esta linha estava escrita à mão em **nove** sítios de
/// três famílias de sondas (com a leitura do `/proc/loadavg` copiada em seis), e acrescentar a
/// coluna a uma delas deixaria as outras oito a dizer menos do que sabem.
pub(super) fn contexto() -> String {
    let carga = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    let perfil = if cfg!(debug_assertions) {
        "DEBUG ⚠️ (o relógio não vale)"
    } else {
        "release"
    };
    format!(
        "load {} · ociosa {:.0} % · perfil {perfil}",
        carga.trim(),
        cpu_ociosa_pct()
    )
}

/// ⭐⭐⭐ **NA FAIXA DO PRODUTO A PLACA GANHA COM MARGEM** — a propriedade, ao lado do tecto.
///
/// # ⚠️ Porque a faixa dele PÁRA em `128` arestas
///
/// A pior das 17 cenas do smoke mede `2 663` valores guardados — entre o polígono de `96` (`2 602`)
/// e o de `128` (`3 462`). ⇒ **esta é a faixa que o produto de facto habita**, e é a única em que a
/// vantagem da placa é robusta: `3,4×`–`12,7×` medido entre `1 %` e `99 %` de CPU ociosa.
///
/// ⛔⛔ **Acima dela a medição não sustenta gate nenhum**, e isso é um facto sobre o dispositivo e
/// não sobre a sonda: a MESMA peça de `256` arestas leu `631` e `1 283 ms` em duas corridas
/// (`1,97×` e `0,90×`). *Um gate que afirmasse uma razão ali estaria a apostar no sorteio.* Quem
/// guarda o topo é **ninguém**: a medição mostrou que a largura da fita não ordena os resultados
/// (`768` arestas perde `0,52×` e `1024` ganha `1,30×`), e a cura nomeada é um laço fechado.
///
/// ⚠️ **Ele lê relógios, logo é da família das flakes de carga** (`CLAUDE.md` §5.0): corre com
/// `#[ignore]`, imprime a carga **e a CPU ociosa**, e quem o lê julga se o número vale.
#[test]
#[ignore = "precisa de GPU — e lê relógio: leia a ociosidade ao lado"]
fn na_faixa_do_produto_a_placa_ganha_com_margem() {
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

    println!("\n  arestas · placa · CPU · razão · {}", contexto());
    let mut pior = f32::INFINITY;
    for n in [64u32, 96, 128] {
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
        let sonda = crate::gpu_frame::Sonda::default();
        // Aquecimento fora da conta: a primeira compila o pipeline.
        let _ = crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            FUNDO,
            None,
            LW,
            LH,
            false,
            sonda,
        );
        let mut v: Vec<f32> = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let p = crate::gpu_frame::paint_com(
                t,
                &doc,
                &reg,
                &cam,
                &luz,
                &surfaces,
                &ph2d_field_render::Presentation::of(olhar),
                FUNDO,
                None,
                LW,
                LH,
                false,
                sonda,
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
    }
    // ⭐ **O CONTROLO primeiro:** sem ele, um `paint_com` que devolvesse sempre a mesma imagem
    // vazia em microssegundos passaria com razões enormes.
    assert!(pior.is_finite(), "a sonda não mediu nada");
    assert!(
        pior > 2.0,
        "na faixa do produto a placa tem de ganhar com margem: pior razão {pior:.2}×"
    );
}

// ⛔⛔⛔ **AQUI VIVIA A `uma_fita_larga_demais_fica_na_cpu`, e ela saiu com o tecto** (2026-09-15).
//
// Ela afirmava que acima de um número a peça cai para a CPU. A medição no topo do slider do artista
// desmentiu a premissa: o contorno de `768` arestas perde `0,52×` de forma **reprodutível** e os
// dois vizinhos, `512` e `1024`, GANHAM (`1,15×`–`2,97×` e `1,30×`). ⇒ *a grandeza não ordena os
// resultados, e um corte que apanhasse o mau excluiria um bom.*
//
// ⚠️ **A fixtura desta gate era honesta e a conclusão dela não** — ela cercava o número com dois
// pontos medidos, e o que faltava era um TERCEIRO ponto, do outro lado. *Uma fronteira prova-se com
// a vizinhança inteira, não com os dois vizinhos que a confirmam.*
//
// O que sobra no lugar: a `na_faixa_do_produto_a_placa_ganha_com_margem` (acima) e, para os
// penhascos, um laço fechado que ninguém mediu — `docs/Render3d/05` §43.10.

/// ⭐⭐⭐⭐ **Os gates das curas da `W9`** — ver o cabeçalho do [`w9`].
#[path = "preview_device_w9_tests.rs"]
mod w9;

/// ⏱️ **As sondas da `W9`** — ver o cabeçalho do [`sondas_w9`].
#[path = "device_probes_w9.rs"]
mod sondas_w9;
