//! ⭐⭐⭐ **A FIGURA DA MOLA — a única do tutorial 2 que é uma CURVA NO TEMPO.**
//!
//! Irmã de [`super::animadores_figures`] por RESPONSABILIDADE, não por tamanho: as outras doze
//! figuras são **uma nuvem num instante** (ou o rasto de vários), e nenhuma delas conseguiria
//! mostrar a diferença entre *«chega e pára»* e *«chega, passa e volta»* — que é exactamente o
//! que a W1 do ciclo 2 acrescentou. Aqui o eixo horizontal é o **tempo**, e é por isso que a
//! moldura, o traço e a régua desta figura são todos outros.
//!
//! ⚠️ O corte foi imposto pelo teto de 600 LOC do shell (HR-18) — e a linha por onde ele passou
//! foi escolhida pela pergunta que cada metade responde, nunca pelo número de linhas.

use super::animadores_figures::{DT, liga, no_com, pontos, realimenta};
use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::Cook;

/// ⭐⭐⭐ **A FIGURA DA MOLA — e ela não é uma nuvem: é uma CURVA NO TEMPO.**
///
/// Uma mola não tem forma no espaço; ela tem um *jeito de chegar*. As outras onze figuras
/// mostram uma fila num instante, e nenhuma delas conseguiria mostrar a diferença entre
/// «chega e pára» e «chega, passa e volta» — que é exactamente o que a W1 acrescentou.
///
/// A cena: um alvo que SALTA (a onda quadrada) e três molas a persegui-lo.
/// ⚠️ **A afirmação da figura é medida antes de ela ser escrita:** com `Bounce = 0` a mola não
/// pode passar do alvo, e com `Bounce` alto tem de passar. Uma figura que ensinasse o contrário
/// é pior que nenhuma.
pub(super) fn dump_spring_curves(dir: &std::path::Path) {
    const TIQUES: usize = 260;
    const ALVO: f32 = 90.0;
    let corre = |params: &[(&str, f32)]| -> (Vec<f32>, Vec<f32>) {
        let mut m = MotionState::new();
        let feed = no_com(&mut m, "motion.grid", &[("rows", 1.0), ("cols", 1.0)]);
        let osc = no_com(
            &mut m,
            "motion.oscillator",
            &[
                ("channel", 1.0),
                ("wave", 2.0),
                ("frequency", 0.35),
                ("amplitude", ALVO),
                ("phase_stagger", 0.0),
            ],
        );
        let mola = no_com(&mut m, "motion.spring", params);
        let out = m.doc.graph.add_node("motion.output".to_string());
        liga(&mut m, feed, 0, osc, 0, false);
        liga(&mut m, osc, 0, mola, 0, false);
        realimenta(&mut m, mola);
        liga(&mut m, mola, 0, out, 0, false);
        // O alvo, na mesma cadeia sem a mola — para a figura mostrar o que ela persegue.
        let mut ma = MotionState::new();
        let fa = no_com(&mut ma, "motion.grid", &[("rows", 1.0), ("cols", 1.0)]);
        let oa = no_com(
            &mut ma,
            "motion.oscillator",
            &[
                ("channel", 1.0),
                ("wave", 2.0),
                ("frequency", 0.35),
                ("amplitude", ALVO),
                ("phase_stagger", 0.0),
            ],
        );
        let ob = ma.doc.graph.add_node("motion.output".to_string());
        liga(&mut ma, fa, 0, oa, 0, false);
        liga(&mut ma, oa, 0, ob, 0, false);
        let (mut cm, mut ca) = (Cook::new(), Cook::new());
        let (mut ys, mut alvo) = (Vec::new(), Vec::new());
        for k in 0..TIQUES {
            let t = k as f64 * DT;
            ys.extend(pontos(&mut cm, &m, out, t).first().map(|p| p[1]));
            alvo.extend(pontos(&mut ca, &ma, ob, t).first().map(|p| p[1]));
        }
        (ys, alvo)
    };
    let (fisica, alvo) = corre(&[("channel", 1.0)]);
    let (seco, _) = corre(&[
        ("channel", 1.0),
        ("mode", 1.0),
        ("duration", 0.6),
        ("bounce", 0.0),
    ]);
    let (salta, _) = corre(&[
        ("channel", 1.0),
        ("mode", 1.0),
        ("duration", 0.6),
        ("bounce", 0.6),
    ]);
    let pico = |v: &[f32]| v.iter().copied().fold(f32::MIN, f32::max);
    assert!(
        pico(&seco) <= ALVO * 1.005,
        "`Bounce = 0` NAO pode passar do alvo, e a figura promete isso: {:.2} contra {ALVO}",
        pico(&seco)
    );
    assert!(
        pico(&salta) > ALVO * 1.05,
        "`Bounce = 0,6` TEM de passar do alvo, senao a figura nao mostra salto nenhum: {:.2}",
        pico(&salta)
    );
    // ⚠️⚠️ **O tempo e o valor NÃO estão na mesma unidade**, e a 1.ª versão desta figura pôs os
    // dois no `viewBox` cru: `4,33` segundos contra `261` de excursão, num quadro de 620×260 —
    // o eixo do tempo colapsou e a figura saiu como **um traço vertical**. O tempo é mapeado
    // para a largura da figura, que é o que um gráfico faz.
    const LARGURA: f32 = 500.0;
    /// ⚠️ A altura é **fixa**: os dois eixos deste gráfico têm unidades diferentes (segundos e
    /// pixels), então nada os obriga a partilhar a escala — e a figura ajustada aos dados saía
    /// com `2,6` de altura por `1` de largura, tomando uma página inteira do PDF.
    const ALTURA: f32 = 190.0;
    // ⚠️ **A moldura sai dos DADOS.** A 1.ª versão fixou-a em `±1,45 × alvo` e o modo `Physics`
    // — que a `tension 8` / `friction 1,5` é bem mais saltitante que o `Bounce = 0,6` — saiu
    // **por fora do quadro**: a figura mostrava três curvas e cortava a que mais salta.
    let (y0, y1) = [&fisica, &seco, &salta, &alvo]
        .iter()
        .flat_map(|v| v.iter())
        .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
    let folga = (y1 - y0) * 0.06;
    let (y0, y1) = (y0 - folga, y1 + folga);
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {LARGURA:.2} {ALTURA:.2}\" \
         width=\"620\" height=\"{:.0}\" role=\"img\">\n\
         <rect width=\"{LARGURA:.2}\" height=\"{ALTURA:.2}\" rx=\"9\" fill=\"#141317\"/>\n",
        620.0 * ALTURA / LARGURA
    );
    let mut linha = |v: &[f32], cor: &str, largura: f32, tracejado: &str| {
        let d: String = v
            .iter()
            .enumerate()
            .map(|(k, y)| {
                let x = k as f32 * LARGURA / (TIQUES - 1) as f32;
                let yy = (y1 - y) * ALTURA / (y1 - y0);
                format!("{}{x:.3},{yy:.3}", if k == 0 { "M" } else { "L" })
            })
            .collect::<Vec<_>>()
            .join(" ");
        s.push_str(&format!(
            "<path d=\"{d}\" fill=\"none\" stroke=\"{cor}\" stroke-width=\"{largura}\" \
             stroke-linejoin=\"round\"{tracejado}/>\n"
        ));
    };
    linha(&alvo, "#4b4560", 2.6, " stroke-dasharray=\"7 5\"");
    linha(&fisica, "#6f86c9", 2.1, "");
    linha(&seco, "#7ee0b0", 2.1, "");
    linha(&salta, "#ffcf7a", 2.1, "");
    s.push_str("</svg>\n");
    let path = dir.join("spring.svg");
    std::fs::write(&path, s).expect("escrever a curva da mola");
    eprintln!(
        "  {:<12} │ pico seco {:.1} · pico com salto {:.1} (alvo {ALVO}) │ {}",
        "spring",
        pico(&seco),
        pico(&salta),
        path.display()
    );
}
