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

/// ⏱️⭐⭐⭐⭐ **A PRIMEIRA CHAMADA DE UMA CENA PAGA A COMPILAÇÃO — e é ela que o gate vermelho mede.**
///
/// O [`com_o_dispositivo_a_maioria_das_cenas_e_nitida_em_movimento`] cronometra **UMA** chamada a
/// [`crate::gpu_frame::paint`] por cena e chama-lhe *«o quadro de movimento»*. ⛔⛔ Mas a primeira
/// chamada de uma cena compila o programa da placa: o cache do pintor tem por chave o **TEXTO** do
/// shader, e o texto leva a FITA da peça — logo *cada cena é um texto novo e um compilador inteiro*.
/// O quadro que o artista arrasta é o **N-ésimo**, com tudo compilado.
///
/// ⚠️⚠️ **E a `W9` já prescreve a régua certa, por escrito**
/// ([`03`](../../../docs/Render3d/03_o_plano.md)): *«`1920×1080`, **mínimo de N**, A/B intercalado
/// no MESMO processo»*. O gate não o faz — *uma régua que o próprio plano do módulo corrige e
/// ninguém emendou*.
///
/// ⇒ esta sonda pinta a MESMA cena três vezes seguidas e põe as três a par. Se a 1.ª for a cara e
/// as outras baratas, o vermelho é da régua.
#[test]
#[ignore = "sonda de diagnóstico: separa a compilação do custo do quadro"]
fn diag_o_primeiro_quadro_de_uma_cena_paga_a_compilacao() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!(
        "\n  cena · quadro de MOVIMENTO a {LW}×{LH} · {}",
        contexto()
    );
    println!("  cena ·      1.ª ·      2.ª ·      3.ª ·  1.ª/mín");
    let (mut pior_razao, mut nitidas_pela_1a, mut nitidas_pelo_min) = (0.0f32, 0usize, 0usize);
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
        let mut ts = Vec::new();
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let saiu = crate::gpu_frame::paint(
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
            );
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            if saiu.is_none() {
                break;
            }
            ts.push(ms);
        }
        if ts.len() < 3 {
            println!("  {n:>4} ·        na CPU");
            continue;
        }
        let minimo = ts.iter().copied().fold(f32::INFINITY, f32::min);
        let razao = ts[0] / minimo;
        pior_razao = pior_razao.max(razao);
        if ts[0] <= PREVIEW_BUDGET_MS {
            nitidas_pela_1a += 1;
        }
        if minimo <= PREVIEW_BUDGET_MS {
            nitidas_pelo_min += 1;
        }
        println!(
            "  {n:>4} · {:>7.2} · {:>7.2} · {:>7.2} · {razao:>7.2}x",
            ts[0], ts[1], ts[2]
        );
    }
    println!(
        "\n  ⇒ pela 1.ª chamada: {nitidas_pela_1a} nítidas · pelo MÍNIMO de 3: {nitidas_pelo_min} \
         · pior razão 1.ª/mín {pior_razao:.2}x\n"
    );
}

/// ⏱️⭐⭐⭐⭐ **O QUE O ARTISTA PAGA: arrastar um número, ou ACRESCENTAR uma forma?**
///
/// A [`diag_o_primeiro_quadro_de_uma_cena_paga_a_compilacao`] mede `1,4`–`4,4 s` na primeira
/// pintura de cada cena e `6`–`130 ms` nas seguintes, e o custo **não é da CPU**: a 2.ª chamada
/// refaz a MESMA fita (a [`crate::gpu_frame::paint`] reconstrói o `DeviceField` a cada chamada) e
/// custa `12 ms`. ⇒ *o segundo e meio é a placa a compilar o programa.*
///
/// ⛔⛔ **E a pergunta que decide se isto é um defeito de PRODUTO ou uma nota de bancada é outra:**
/// o cache do pintor tem por chave o **TEXTO** do shader, e o texto leva a FITA da peça. Um
/// **arrasto de slider** muda os números (o armazém `k`) e o texto fica igual; **acrescentar uma
/// forma** muda a árvore, logo muda a fita, logo muda o texto. *Se for assim, toda mudança de
/// ESTRUTURA custa um segundo e meio, e nenhum doc deste módulo o diz.*
///
/// Esta sonda põe as duas lado a lado: a mesma peça com o RAIO mudado, e uma peça com uma forma a
/// mais.
#[test]
#[ignore = "sonda de diagnóstico: separa mudar um NÚMERO de mudar a ESTRUTURA"]
fn diag_mudar_um_numero_contra_acrescentar_uma_forma() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };

    let uma_bola = |r: f32| {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                Primitive::Sphere { radius: r },
                Xform::at(0.0, 0.0, 0.0),
            )],
            NodeId(0),
        )
        .expect("a bola")
    };
    let pinta = |doc: &FieldDoc| -> f32 {
        let t0 = std::time::Instant::now();
        let saiu = crate::gpu_frame::paint(
            t,
            doc,
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
        );
        assert!(saiu.is_some(), "o pintor recusou a peça");
        #[allow(clippy::cast_possible_truncation)]
        let ms = t0.elapsed().as_secs_f32() * 1e3;
        ms
    };

    println!("\n  {}", contexto());
    println!("  gesto                              ·       ms");
    println!("  ───────────────────────────────────·─────────");
    println!(
        "  bola r=0,50, 1.ª vez (compila)     · {:>8.2}",
        pinta(&uma_bola(0.50))
    );
    println!(
        "  a MESMA, outra vez                 · {:>8.2}",
        pinta(&uma_bola(0.50))
    );
    println!(
        "  ARRASTAR o raio: r=0,60            · {:>8.2}",
        pinta(&uma_bola(0.60))
    );
    println!(
        "  arrastar outra vez: r=0,70         · {:>8.2}",
        pinta(&uma_bola(0.70))
    );

    // ⚠️ A forma a mais é o que muda a ÁRVORE. Duas folhas unidas: é a estrutura mais barata que
    // deixa de ser a fita de uma bola.
    let duas = FieldDoc::new(
        vec![
            ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.5 }, Xform::at(-0.3, 0.0, 0.0)),
            ph2d_field_eval::leaf(Primitive::Sphere { radius: 0.4 }, Xform::at(0.3, 0.0, 0.0)),
            ph2d_field::Node {
                xform: Xform::IDENTITY,
                kind: ph2d_field::NodeKind::Combine {
                    op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                    children: vec![NodeId(0), NodeId(1)],
                },
                mods: Vec::new(),
                verb: None,
            },
        ],
        NodeId(2),
    )
    .expect("as duas");
    println!(
        "  ACRESCENTAR uma forma (1.ª vez)    · {:>8.2}",
        pinta(&duas)
    );
    println!(
        "  a MESMA peça de duas, outra vez    · {:>8.2}",
        pinta(&duas)
    );
    println!(
        "  voltar à bola r=0,50               · {:>8.2}\n",
        pinta(&uma_bola(0.50))
    );
}

/// ⭐⭐⭐⭐ **A FITA DA PEÇA SAI DO SHADER DO PINTOR E A IMAGEM NÃO MUDA UM BYTE.**
///
/// Ver [`ph2d_field_gpu::paint::PaintSetup::le_o_campo`] para o grafo de chamadas que o decidiu:
/// no quadro de MOVIMENTO, com nada a ler a curvatura, o `pinta` e o `pinta_bordas` alcançam a fita
/// por **nenhum** caminho que corra. ⇒ substituí-la por uma constante é byte-idêntico *por
/// construção*, e o que se compra é o texto do shader deixar de mudar com a peça.
///
/// ⚠️ **A cena leva CHÃO de propósito.** A primeira medição desta cura correu sem ele, e o chão é
/// exactamente onde um segundo caminho para a fita poderia estar escondido (o `ceu_do_chao` chama
/// `field()`): ele corre no passe da MARCHA, que continua a levar a fita, e esta metade é o que o
/// afirma em vez de o supor.
///
/// ⛔ **O CONTROLO vem primeiro:** uma imagem toda de fundo é trivialmente igual a outra imagem
/// toda de fundo. *Sem ele, apagar a peça faria este gate passar.*
#[test]
#[ignore = "precisa de GPU"]
fn a_fita_inerte_no_pintor_nao_muda_um_byte() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(0);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let chao = Some(ph2d_field_render::Ground { height: -1.0 });
    let pinta = |fita_inerte: bool| {
        crate::gpu_frame::paint_com(
            t,
            &doc,
            &reg,
            &cam,
            &luz,
            &surfaces,
            &ph2d_field_render::Presentation::of(olhar),
            BG,
            chao,
            LW,
            LH,
            false,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };
    let com = pinta(true);
    let sem = pinta(false);

    // ── O CONTROLO: a peça está na imagem ─────────────────────────────────────────────────────
    let do_fundo = sem
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p == BG)
        .count();
    let total = sem.rgba.len() / 4;
    assert!(
        do_fundo * 10 < total * 9 && do_fundo * 10 > total,
        "CONTROLO: {do_fundo} de {total} píxeis são o fundo — uma imagem quase vazia (ou quase \
         cheia) é trivialmente igual a outra, e este gate não afirmaria nada"
    );

    assert_eq!(
        com.rgba, sem.rgba,
        "a fita inerte mudou a imagem — alguma coisa no quadro de MOVIMENTO LÊ o campo da peça \
         dentro do pintor, e o predicado `le_o_campo` não a nomeia"
    );
}

/// ⭐⭐⭐⭐ **E A ECONOMIA MEDE-SE PELA CONTA, porque é invisível à imagem.**
///
/// O gate irmão afirma que as duas rotas dão a MESMA imagem — e é exactamente isso que torna a cura
/// impossível de medir por valor: *uma cura apagada também não muda a imagem*. ⇒ a régua é o
/// [`ph2d_field_gpu::trace::Tracer::compiled`], a contagem de pipelines que o cache já compilou.
///
/// **O mecanismo:** o cache tem por chave o TEXTO do shader. Com a fita da peça lá dentro, cada
/// estrutura nova é um texto novo e compila **tudo**; com ela fora, só os passes da MARCHA — que
/// levam a fita de verdade — é que recompilam.
///
/// ⚠️ **A segunda peça tem de ser outra ESTRUTURA e não outro número:** arrastar um raio já não
/// recompilava nada antes desta cura (o texto não muda, só o armazém `k`), e uma fixtura assim
/// mediria zero dos dois lados.
///
/// **Medido (2026-09-21): `SEM a cura cresceu 4 pipelines · COM a cura cresceu 2`** — os dois que
/// ficam são os da MARCHA (`centro_e_luz` e `bordas`), que levam a fita de verdade; os dois que
/// saem são o `pinta` e o `pinta_bordas`.
///
/// ⛔⛔ **ELE LÊ UM CONTADOR GLOBAL, logo não sobrevive a correr em PARALELO com os irmãos.** O
/// [`ph2d_field_gpu::trace::Tracer`] vive num `Arc<Mutex<_>>` do módulo e o `compiled()` conta o
/// processo inteiro: sob `cargo test` (threads dentro de UM processo) um irmão a pintar no meio do
/// `cresce` entra nesta conta. ⭐ Sob o `nextest` — que é o portão desta casa e dá **um processo
/// por teste** — ele é sólido. *A prova de mutação desta wave leu «SOBREVIVEU» duas vezes por
/// causa disto, e a cura foi `--test-threads=1` no ARNÊS, nunca uma barra mais frouxa aqui.*
#[test]
#[ignore = "precisa de GPU"]
fn a_fita_inerte_faz_o_cache_acertar_na_peca_seguinte() {
    use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    // ⛔⛔⛔ **OS DOIS LADOS TÊM DE SER PRIMITIVAS DIFERENTES, e a 1.ª redacção deste gate era um
    // VÁCUO por causa disso.** Ela variava o RAIO entre os dois lados — e um raio é uma
    // **constante da fita** (`Instr::Const` → `k[i]`), logo duas bolas de raios diferentes dão o
    // MESMO texto de shader. ⇒ o 2.º lado acertava no cache que o 1.º acabara de encher, lia
    // crescimento **ZERO**, e `com < sem` passava até com a cura apagada. *Uma mutação SOBREVIVENTE
    // mostrou-o: o gate media o cache já quente, não a cura.*
    let uma = |k: bool| {
        FieldDoc::new(
            vec![ph2d_field_eval::leaf(
                if k {
                    Primitive::Sphere { radius: 0.5 }
                } else {
                    Primitive::Box {
                        half: [0.4, 0.4, 0.4],
                        round: 0.0,
                        chamfer: 0.0,
                    }
                },
                Xform::at(0.0, 0.0, 0.0),
            )],
            NodeId(0),
        )
        .expect("a peça de uma")
    };
    let duas = |k: bool| {
        let folha = |x: f32| {
            ph2d_field_eval::leaf(
                if k {
                    Primitive::Sphere { radius: 0.4 }
                } else {
                    Primitive::Box {
                        half: [0.3, 0.3, 0.3],
                        round: 0.0,
                        chamfer: 0.0,
                    }
                },
                Xform::at(x, 0.0, 0.0),
            )
        };
        FieldDoc::new(
            vec![
                folha(-0.3),
                folha(0.3),
                ph2d_field::Node {
                    xform: Xform::IDENTITY,
                    kind: ph2d_field::NodeKind::Combine {
                        op: ph2d_field::Op::Union(ph2d_field::Blend::Sharp),
                        children: vec![NodeId(0), NodeId(1)],
                    },
                    mods: Vec::new(),
                    verb: None,
                },
            ],
            NodeId(2),
        )
        .expect("a peça de duas")
    };
    let pinta = |doc: &FieldDoc, fita_inerte: bool| {
        crate::gpu_frame::paint_com(
            t,
            doc,
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
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor");
    };
    let conta = || t.lock().expect("o traçador").compiled();

    let cresce = |esfera: bool, fita_inerte: bool| {
        pinta(&uma(esfera), fita_inerte);
        let antes = conta();
        pinta(&duas(esfera), fita_inerte);
        conta() - antes
    };
    // ⚠️ **Cada lado na sua FAMÍLIA de primitiva**, pela razão do bloco acima: assim as quatro
    // peças são quatro textos, e o crescimento que cada lado lê é o dele.
    let sem = cresce(true, false);
    let com = cresce(false, true);
    println!("  SEM a cura cresceu {sem} pipelines · COM a cura cresceu {com}");

    assert!(
        sem > 0,
        "CONTROLO: sem a cura, mudar a ESTRUTURA da peça compilou {sem} pipelines — se é zero, a \
         fixtura não muda o texto do shader e o gate mede o nada"
    );
    assert!(
        com < sem,
        "com a fita inerte a peça seguinte compilou {com} pipelines e sem ela {sem} — a cura não \
         está a tirar a peça do texto do pintor"
    );
}

/// ⭐⭐⭐⭐ **A METADE QUE TORNA AS OUTRAS DUAS LOAD-BEARING: quem LÊ o campo continua a levá-lo.**
///
/// As duas irmãs medem o caminho em que a fita SAI. Sozinhas, elas ficam verdes sobre uma cura que
/// a tirasse **sempre** — e aí o jade (que lê a curvatura) e o ricochete (que marcha) passariam a
/// ler um campo constante, em silêncio. ⇒ *este gate corre os dois regimes em que a fita TEM de
/// ficar, e afirma que a porta é inerte lá.*
///
/// ⛔⛔ **E o CONTROLO de cada metade é a própria fixtura:** um material que não lê a curvatura, ou
/// um quadro de movimento, não distinguem uma cura certa de uma cura apagada. Por isso a 1.ª metade
/// usa **subsuperfície MACIÇA** (a condição exacta do `Surface::reads_curvature`) e a 2.ª o quadro
/// **ASSENTE** (onde o `ao_rays` abre).
#[test]
#[ignore = "precisa de GPU"]
fn quem_le_o_campo_continua_a_leva_lo_no_shader() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    let doc = crate::smoke::scene(1);
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    let pinta = |mats: &[ph2d_material::Surface], assente: bool, fita_inerte: bool| {
        let surfaces = ph2d_field_render::Surfaces {
            all: mats,
            owners: None,
        };
        crate::gpu_frame::paint_com(
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
            assente,
            crate::gpu_frame::Sonda {
                fita_inerte,
                ..crate::gpu_frame::Sonda::default()
            },
        )
        .expect("o pintor")
    };

    // ── 1. O JADE: subsuperfície MACIÇA é a condição exacta do `Surface::reads_curvature` ──────
    let jade = [ph2d_material::OpenPbr {
        subsurface_weight: 1.0,
        geometry_thin_walled: false,
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    assert!(
        jade[0].reads_curvature(),
        "CONTROLO: a fixtura do jade não lê a curvatura — a metade abaixo mediria o nada"
    );
    assert_eq!(
        pinta(&jade, false, true).rgba,
        pinta(&jade, false, false).rgba,
        "com um material que LÊ a curvatura a porta tem de ser inerte — se a imagem muda, a cura \
         está a tirar a fita a quem precisa dela"
    );

    // ── 2. O QUADRO ASSENTE: ali o `ao_rays` abre e o `assa_sondas` MARCHA ─────────────────────
    let liso = [ph2d_material::OpenPbr::default().prepare()];
    assert!(
        !liso[0].reads_curvature(),
        "CONTROLO: o material de omissão lê a curvatura — então a metade 2 não isola o ricochete"
    );
    assert_eq!(
        pinta(&liso, true, true).rgba,
        pinta(&liso, true, false).rgba,
        "no quadro ASSENTE a porta tem de ser inerte — o `assa_sondas` marcha o campo, e uma fita \
         constante dá-lhe um mundo cheio"
    );
}

/// ⏱️⭐⭐⭐⭐ **A FITA INERTE TAMBÉM PAGA NO RELÓGIO DO QUADRO? — o A/B no MESMO processo.**
///
/// A cura da fita inerte (`docs/Render3d/03` §W9) foi construída para o **COMPILADOR**: ela tira a
/// peça do texto do pintor e o cache passa a acertar (`1 406 → 74 ms` ao acrescentar uma forma).
/// ⭐ **Mas ela também encolhe o kernel que corre**, e o modelo de custo do quadro de movimento diz
/// que o custo segue o TAMANHO da fita (`R² 0,803` com `instruções + transcendentes + raízes`, e
/// `R² 0,165` com os passos da marcha). ⇒ *se o modelo estiver certo, a cura tem um segundo prémio
/// que ninguém pediu.*
///
/// ⚠️ **O A/B corre no MESMO processo e INTERCALADO**, que é a régua que a `W9` prescreve: subtrair
/// dois relógios de corridas separadas dá a soma dos ruídos. E cada lado tira o **MÍNIMO de
/// [`QUADROS_MEDIDOS`]**, senão isto mede a compilação.
///
/// ⛔ **A ordem é `sem` → `com` dentro de cada cena**, e as duas primeiras chamadas de cada lado
/// pagam a compilação daquele texto: é por isso que o mínimo é obrigatório aqui, e não um luxo.
#[test]
#[ignore = "sonda de diagnóstico: mede se a fita inerte paga no relógio do quadro"]
fn diag_a_fita_inerte_no_relogio_do_quadro() {
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador — saltado");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let olhar = ph2d_view_transform::Look::default();
    const BG: [u8; 4] = [0, 0, 0, 0];
    println!("\n  {}", contexto());
    println!("  cena ·  com a fita ·  sem a fita ·  razão ·  instr · transc");
    let (mut melhor, mut pior) = (f32::INFINITY, 0.0f32);
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
        let minimo = |fita_inerte: bool| {
            (0..QUADROS_MEDIDOS)
                .map(|_| {
                    let t0 = std::time::Instant::now();
                    let saiu = crate::gpu_frame::paint_com(
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
                        crate::gpu_frame::Sonda {
                            fita_inerte,
                            ..crate::gpu_frame::Sonda::default()
                        },
                    );
                    #[allow(clippy::cast_possible_truncation)]
                    let ms = t0.elapsed().as_secs_f32() * 1e3;
                    saiu.map(|_| ms)
                })
                .collect::<Option<Vec<f32>>>()
                .map(|v| v.into_iter().fold(f32::INFINITY, f32::min))
        };
        let (Some(sem), Some(com)) = (minimo(false), minimo(true)) else {
            println!("  {n:>4} ·        na CPU");
            continue;
        };
        let campo = ph2d_field_eval::device::DeviceField::new(&doc, &reg).expect("a peça");
        let fita = campo.tape_wgsl().expect("a fita");
        let instrs = fita.source.lines().count();
        let transc = fita
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
        let razao = sem / com;
        melhor = melhor.min(razao);
        pior = pior.max(razao);
        println!(
            "  {n:>4} · {com:8.2} ms · {sem:8.2} ms · {razao:5.2}x · {instrs:>6} · {transc:>6}"
        );
    }
    println!("  ⇒ a razão sem/com vai de {melhor:.2}x a {pior:.2}x\n");
}
