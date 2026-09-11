//! ⭐⭐⭐ **A LEI DO RECÉM-NASCIDO — a tinta não pode APARECER quando uma geração vira.**
//!
//! Report do Enio (2026-08-31): *"Dá pequenos pulos, não é perfeitamente liso"*.
//!
//! # Por que nenhuma régua desta crate o via
//!
//! O `probe_flicker` e o `probe_drift` medem **um escalar de TAMANHO** (largura média de Cauchy,
//! span de eixo, centroide) e perguntam se a série dele é uma recta. A lei do `crate::build`
//! **normaliza exactamente essa grandeza** — ela resolve o factor de comprimento para o tamanho
//! cair na rampa recta. ⇒ *a régua partilhava a lei do produto*, e um espelho não acusa.
//!
//! A grandeza que o produto **não** normaliza é a **TINTA**: a soma dos comprimentos desenhados.
//! Medida, ela saltava `+69,5 %` **num passo do arrasto** no `Bush`.
//!
//! # A lei, e ela é publicada (ABOP §6.2, Prusinkiewicz & Lindenmayer)
//!
//! Um sistema temporizado exige duas condições de continuidade sobre a função de crescimento:
//!
//! - **R2 / eq. (6.3):** `g(a, β) = Σ g(bᵢ, αᵢ)` — no instante da divisão o comprimento do pai
//!   **reparte-se** pelos filhos. Eles retraçam o que já estava desenhado ⇒ a figura não muda.
//! - **eq. (6.11):** um ápice **lateral** acabado de formar tem comprimento **zero** e taxa de
//!   crescimento **zero**.
//!
//! O nosso `build` dava a TODA a geração nova o mesmo número. ⇒ o material que ninguém retraça
//! — um ramo que a produção acabou de abrir, ou um segmento nascido de um ápice que não
//! desenhava — **aparecia de uma vez**.
//!
//! # A partição do corpus, MEDIDA (`examples/probe_pops.rs --refine`)
//!
//! A régua é o **discriminador salto/movimento**: mede-se a diferença através da fronteira, e
//! depois com o passo **4× mais fino**. Movimento encolhe com o passo; um salto **não**.
//!
//! | molde | tem tinta NOVA? | antes: `h` → `h/4` | depois: `h` → `h/4` |
//! |---|---|---|---|
//! | `Bush` (`F -> F[+F][J]F[-F]F`) | sim, ramos laterais | **`38,5×` → `150,3×`** | `1,75×` → `2,28×` |
//! | `Weed` (`X -> F[+X][J]F[-X]+X`) | sim, o `X` não desenha | **`42,4×` → `166,5×`** | `1,30×` → `1,42×` |
//! | `Koch`, `Dragon` | não | `1,08×` → `1,17×` | **idênticos** |
//! | `Tree`, `Fern`, `Wild`, `Sprig` | crescem pela PONTA | `1,3×`–`1,8×` | **idênticos** |
//!
//! ⭐ **A razão a MULTIPLICAR por ~3,9 ao afinar 4× é a assinatura exacta de uma
//! descontinuidade** — a diferença não depende do passo. Depois da cura ela multiplica por
//! `1,1`–`1,3`, que é a classe do controlo.
//!
//! # E a mesma coisa medida na VIRAGEM nua (`--turn`), que é a forma que estes gates usam
//!
//! Salto relativo da tinta ao atravessar `G = 4`, em `ε = 8e-3` e `ε = 1e-3` (o passo cai `8×`):
//!
//! | molde | antes: salto → salto | encolhe | depois: salto → salto | encolhe |
//! |---|---|---|---|---|
//! | `Bush` | **`0,6933` → `0,6699`** | **`0,966×`** | `0,0214` → `0,0027` | `0,125×` |
//! | `Weed` | **`0,5587` → `0,5467`** | **`0,979×`** | `0,0134` → `0,0017` | `0,125×` |
//! | `Koch` | `0,0161` → `0,0020` | `0,125×` | **os mesmos bits** | `0,125×` |
//! | `Dragon` | `0,0037` → `0,0005` | `0,124×` | **os mesmos bits** | `0,124×` |
//! | `Tree`/`Fern`/`Wild`/`Sprig` | `0,0011`–`0,0049` | `0,125×` | **os mesmos bits** | `0,125×` |
//!
//! ⭐⭐ `0,125` é o valor **teórico** de movimento (o passo caiu `8×`), e seis dos oito já o
//! davam. Os dois que davam `0,97` tinham **67 %** e **55 %** da tinta a aparecer num intervalo
//! de largura `1e-3` — e nenhuma taxa de quadros esconde isso.

use ph2d_node_source_lsystem::{PRESETS, Preset, param, probe_build};
use ph2d_nodegraph::attr::Column;

/// A tinta: a soma dos comprimentos desenhados. **Não** é o que o produto normaliza, e é por
/// isso que ela pode acusá-lo.
fn ink(p: &Preset, generations: f32) -> f32 {
    let s = probe_build(
        p.axiom,
        p.rules,
        generations,
        &[
            (param::ANGLE, p.angle),
            (param::STEP, p.step),
            (param::WIDTH, p.width),
        ],
    );
    match s.get("len") {
        Some(Column::Scalar(v)) => v.iter().sum(),
        _ => 0.0,
    }
}

/// A geração inteira em que se mede a viragem: baixa o bastante para toda a gramática do corpus
/// a alcançar dentro do orçamento, e alta o bastante para a figura já ter ramos.
fn turn_at(p: &Preset) -> f32 {
    p.generations.clamp(2.0, 4.0)
}

/// `|tinta(G + ε) − tinta(G)| / tinta(G)` — o salto RELATIVO da tinta ao virar a geração `G`.
fn relative_jump(p: &Preset, eps: f32) -> f32 {
    let g = turn_at(p);
    let a = ink(p, g);
    let b = ink(p, g + eps);
    if a <= 1e-6 { 0.0 } else { (b - a).abs() / a }
}

/// ⭐⭐⭐ **O GATE DO PRODUTO.** Nenhuma gramática do corpus faz a tinta APARECER quando uma
/// geração vira.
///
/// ⚠️ **A barra não é um número escolhido: é a assinatura.** Uma descontinuidade tem salto
/// **independente do passo**, logo `salto(ε/8) / salto(ε)` fica em `~1`; movimento encolhe com
/// o passo, logo a razão fica em `1/8 = 0,125` — que é o valor **teórico**, e é exactamente o
/// que os oito moldes dão hoje (`0,124`–`0,125`). Sem a lei do recém-nascido, o `Bush` dava
/// **`0,966`** e o `Weed` **`0,979`**: o salto inteiro sobrevive a dividir o passo por oito.
/// A barra fica em **`0,5`** — `4×` acima do medido e `1,9×` abaixo do defeito.
#[test]
fn no_grammar_makes_its_ink_appear_when_a_generation_turns() {
    let (mut worst, mut who) = (0.0f32, "");
    for p in PRESETS {
        let (big, small) = (relative_jump(p, 8e-3), relative_jump(p, 1e-3));
        // Um molde cujo salto já é ruído de `f32` não tem razão para dizer nada.
        if big < 1e-4 {
            continue;
        }
        let shrink = small / big;
        assert!(
            shrink < 0.5,
            "{}: a tinta salta {:.4} com ε=8e-3 e ainda {:.4} com ε=1e-3 \
             (encolhe {:.3}× onde movimento encolheria 0,125×) — isto é uma DESCONTINUIDADE, \
             não velocidade. Ver a lei do recém-nascido em `turtle::newborn_law`.",
            p.label,
            big,
            small,
            shrink,
        );
        if shrink > worst {
            (worst, who) = (shrink, p.label);
        }
    }
    assert!(
        worst > 0.0,
        "nenhum molde do corpus foi medido — a população esvaziou-se e o gate ficou vazio"
    );
    // ⚠️ A metade que impede a varredura de se esvaziar em silêncio: se um dia TODOS os moldes
    // caírem no `continue`, o `assert` acima dispara. Este imprime quem é o pior vivo.
    println!("pior encolhimento do corpus: {who} {worst:.3}");
}

/// **E o salto ABSOLUTO também tem de ser pequeno** — a assinatura sozinha aprovaria um degrau
/// minúsculo que nunca encolhe, e reprovaria uma figura que anda depressa e é lisa.
///
/// ⚠️ Barra derivada dos DOIS lados: com a lei de hoje o pior do corpus é **`0,0027`** (`Bush`)
/// e o melhor defeito conhecido é **`0,5467`** (`Weed` sem ela). A barra fica em **`0,04`** — a
/// média geométrica das duas classes é `0,038`, logo ela está `15×` acima do pior medido e
/// `14×` abaixo do defeito mais fraco. *Uma barra a meio caminho geométrico não é escolhida: é
/// onde as duas populações a põem.*
#[test]
fn the_ink_that_a_generation_turn_adds_is_a_sliver_not_a_step() {
    for p in PRESETS {
        let j = relative_jump(p, 1e-3);
        assert!(
            j < 0.04,
            "{}: virar a geração acrescenta {:.4} da tinta de uma vez (barra 0,04)",
            p.label,
            j,
        );
    }
}

/// **A PONTA estica LINEARMENTE** — e este gate existe porque uma mutação SOBREVIVEU.
///
/// ⛔⛔ O braço que cresce pela PONTA passa `lat = 1`, o que torna a lei do recém-nascido
/// **inerte** ali: tudo o que é novo já estica de zero. Eu escrevi essa inércia num comentário
/// e **nada a defendia** — trocar `1.0` por `frac` dá aos recém-nascidos um perfil `frac²`
/// (devagar-devagar-depressa) e a suíte inteira ficava verde. *Uma inércia declarada sem gate é
/// uma afirmação, não uma propriedade.*
///
/// ⚠️ A régua é a TINTA ACRESCENTADA, que é exactamente linear por construção quando a lei é
/// linear: a `frac` fixo o conjunto de módulos não muda, e só os mais novos escalam. Medido, os
/// quatro dão **`0,2500` / `0,5000` / `0,7500`** — exacto, não aproximado.
///
/// ⛔⛔ **Só vale para quem cresce pela PONTA, e a família SAI DO PRODUTO** (`probe_grows_by_refining`),
/// nunca de uma lista escrita à mão: quem REFINA tem o comprimento **normalizado** para o
/// TAMANHO cair na rampa recta, e a tinta dele não é linear por construção (o `Koch` dá `0,317`
/// a meio, e está certo). *Um gate com a sua própria lista de moldes discorda do nó no dia em
/// que uma gramática muda de família* — é a lição que a `probe_growth_ratio` já pagou.
#[test]
fn the_tip_stretches_linearly_and_not_slowly_then_fast() {
    for p in PRESETS {
        let framing = [
            (param::ANGLE, p.angle),
            (param::STEP, p.step),
            (param::WIDTH, p.width),
        ];
        if ph2d_node_source_lsystem::probe_grows_by_refining(p.axiom, p.rules, &framing) {
            continue;
        }
        let g = turn_at(p);
        let base = ink(p, g);
        let full = ink(p, g + 1.0) - base;
        if full.abs() < 1e-4 {
            continue;
        }
        for f in [0.25f32, 0.5, 0.75] {
            let got = (ink(p, g + f) - base) / full;
            println!("{:8} f={f:.2}  acrescentado/total = {got:.4}", p.label);
            assert!(
                (got - f).abs() < 0.01,
                "{}: a {f:.2} da geração já cresceu {got:.4} dela (esperado ~{f:.2}) — \
                 a rampa do recém-nascido deixou de ser linear",
                p.label,
            );
        }
    }
}

/// **`Grow Angle` desligado = o degrau inteiro de sempre** — e este gate também nasce de uma
/// mutação sobrevivente.
///
/// ⛔ Naquele braço tudo vai a `1.0`, o que quer dizer *«desenha a geração seguinte inteira»*.
/// Pôr `lat = 0` ali **apaga os ramos laterais da figura** e a suíte ficava verde. A afirmação
/// «byte a byte» estava escrita num comentário e em nenhum teste.
///
/// ⚠️ A igualdade é EXACTA e não aproximada, e é isso que a torna uma cerca: com
/// `step_scale = 1` (o default) o `powf` do passo devolve `1.0` ao bit para qualquer expoente,
/// logo a única diferença possível entre `g + frac` e `g + 1` é a lei do crescimento.
///
/// ⛔⛔ **Só vale para quem REFINA, e a família SAI DO PRODUTO** — a irmã desta cerca está no
/// gate acima. Num molde que cresce pela PONTA o `Grow Angle` é **inerte por construção** (o
/// braço dele nem o lê), e o comprimento continua a esticar de zero: ali `g + 0,37` **não** é
/// `g + 1`, e está certo. *A 1.ª redacção destes dois gates varria os oito moldes e os dois
/// reprovavam — cada um sobre a família do outro.*
#[test]
fn with_grow_angle_off_a_fractional_generation_draws_the_whole_next_one() {
    for p in PRESETS {
        let framing = [
            (param::ANGLE, p.angle),
            (param::STEP, p.step),
            (param::WIDTH, p.width),
        ];
        if !ph2d_node_source_lsystem::probe_grows_by_refining(p.axiom, p.rules, &framing) {
            continue;
        }
        let g = turn_at(p);
        let cook = |x: f32| {
            probe_build(
                p.axiom,
                p.rules,
                x,
                &[
                    (param::ANGLE, p.angle),
                    (param::STEP, p.step),
                    (param::WIDTH, p.width),
                    (param::CONTINUOUS_ANGLE, 0.0),
                ],
            )
        };
        let (a, b) = (cook(g + 0.37), cook(g + 1.0));
        assert_eq!(
            a.count(),
            b.count(),
            "{}: a geração fraccionária tem outra contagem",
            p.label
        );
        for (name, col) in a.columns() {
            let other = b.get(name);
            assert!(
                other.map(|o| o == col).unwrap_or(false),
                "{}: a coluna `{name}` difere entre `g+0,37` e `g+1` com o Grow Angle desligado \
                 — o degrau deixou de ser o degrau inteiro",
                p.label
            );
        }
    }
}
