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
