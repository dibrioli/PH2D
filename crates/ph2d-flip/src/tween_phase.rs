//! **Alinhamento de FASE da costura em traço FECHADO** (o item aberto do `04 §2` / doc 11 §7).
//!
//! Módulo irmão do [`crate::tween`] e do [`crate::tween_flip`] — e é uma responsabilidade
//! à parte, exatamente como o comentário do `opposite_winding` prescreveu (*"wave própria,
//! não um `if` a mais no auto-flip"*).
//!
//! # O que estava errado
//!
//! Num anel, o ponto 0 é uma **costura arbitrária** (onde o artista fechou o traço), não uma
//! ponta como num traço aberto. O tween pareia `a[i]` com `b[i]`, então se A e B são o MESMO
//! anel desenhado a partir de pontos 0 diferentes, o pareamento fica girado ao longo do
//! contorno e a forma do meio **torce**.
//!
//! O [`crate::tween_flip`] resolve o *sentido* do percurso (winding); isto resolve a *fase*.
//!
//! # Como — e por que sobre o TRAJETO, não a virada
//!
//! É uma **correlação circular** (a técnica do `phase_only` do `ph2d-vec-blend`), e — como lá —
//! sobre as **posições centradas no centróide**: a fase é a que **sobrepõe** as duas formas com
//! o MENOR deslocamento (o critério do flubber).
//!
//! ⚠️ A 1ª versão correlacionava a **virada** `(sen, cos)`, invariante à rotação, no raciocínio
//! de que "a espiral tira o rígido depois". Estava errado, e o Enio pegou: para um anel, **um
//! giro e um deslocamento de costura são a MESMA coisa**, e a virada, cega ao giro, escolhia uma
//! fase que fazia o ajuste (Umeyama) achar uma ROTAÇÃO — a espiral então varria um ARCO em vez de
//! deslizar. Um deslocamento de fase de `s` vértices já vira ~`s·360/n` graus de giro no ajuste,
//! então até uma costura 2 vértices fora (o que QUALQUER par de chaves de mão tem) arqueava.
//!
//! Correlacionar o TRAJETO conserta os dois: a fase de menor deslocamento faz o ajuste ver
//! ~translação (o blob DESLIZA, que é o que o artista quer ao mover), e um círculo tem um mínimo
//! CLARO em `s=0` (não o custo chato da virada), então ele não é mexido por ruído.
//!
//! # Por que NÃO há margem de aceitação (a lição do arco)
//!
//! A 1ª versão (virada) tinha uma margem: só cedia da identidade por um vale decisivo, para não
//! "mexer à toa". Com o TRAJETO essa margem é NOCIVA: como o mínimo nunca aumenta o deslocamento
//! (a identidade é sempre um candidato), a fase só pode REDUZIR a rotação que o ajuste vê — e uma
//! margem só a impediria de endireitar uma costura moderadamente deslocada (off≈4 vértices ⇒ ~28°
//! de giro deixado ⇒ arco). Então a fase toma sempre o menor trajeto; a única guarda é o anel
//! degenerado (todos os pontos coincidem) e o anel pequeno ([`MIN_RING`]).

use ph2d_core::Vec2;

/// Resolução da correlação circular (grade de arco). Irmão do `PHASE_STEPS = 256` do
/// `ph2d-vec-blend`; menor porque um traço do Flip é uma polilinha de mão. **O custo é
/// `O(steps²)`** e, por ser um TETO da grade (não do número de pontos), constante no tamanho do
/// anel — a régua `the_phase_ruler` o mede.
const PHASE_STEPS: usize = 96;

/// Abaixo disto a fase é ruído: um triângulo tem 3 costuras plausíveis, e um "anel" de 4 pontos é
/// grosso demais. Devolve 0 — o comportamento de sempre (e o que mantém byte-idênticos os gates
/// de furo do `tween`, que usam quadrados de 4 pontos).
const MIN_RING: usize = 8;

/// **O deslocamento cíclico `s` em `0..n`** tal que parear `a[i]` com `b[(i + s) % n]` sobrepõe
/// as duas formas com o menor deslocamento (o TRAJETO). Ambos os anéis têm de ter `len n` e ser
/// fechados (o chamador garante). Devolve `0` — a identidade — quando a fase é ambígua, o ganho é
/// pequeno, ou o anel é pequeno demais: o alinhamento nunca é imposto, e por construção **nunca
/// aumenta o trajeto** (a identidade é sempre um candidato).
pub(crate) fn seam_shift(a: &[Vec2], b: &[Vec2]) -> usize {
    let n = a.len();
    if n < MIN_RING || b.len() != n {
        return 0;
    }
    let steps = n.min(PHASE_STEPS);
    let sa = centered(&resample_arc(a, steps));
    let sb = centered(&resample_arc(b, steps));

    // custo(s) = Σ_k |sa[k] − sb[(k+s) % steps]|²  — o TRAJETO sob o deslocamento `s` (as formas
    // já centradas ⇒ a translação entre A e B não enviesa a escolha; sobra a correlação de FORMA).
    let cost = |s: usize| -> f32 {
        (0..steps)
            .map(|k| (sa[k] - sb[(k + s) % steps]).length_squared())
            .sum()
    };

    let (mut best, mut cbest, mut cmax) = (0usize, cost(0), cost(0));
    for s in 1..steps {
        let c = cost(s);
        if c < cbest {
            cbest = c;
            best = s;
        }
        if c > cmax {
            cmax = c;
        }
    }

    // ⚠️ **SEM margem de aceitação, e é a lição do arco.** Com o TRAJETO, o mínimo NUNCA aumenta o
    // deslocamento (a identidade é sempre um candidato), então a fase só pode reduzir a rotação
    // que o ajuste vê — nunca introduzi-la. Uma margem aqui só IMPEDIRIA a fase de endireitar uma
    // costura moderadamente deslocada (off≈4 ⇒ ~28° de giro deixado ⇒ arco). A única guarda real é
    // o anel degenerado (`spread ≈ 0`: todos os pontos coincidem, nada a alinhar).
    if best == 0 || (cmax - cbest) <= f32::EPSILON {
        return 0;
    }
    // `best` mora na grade `steps`; leva-o de volta ao índice `n` (arredondando).
    ((best * n + steps / 2) / steps) % n
}

/// `k` pontos igualmente espaçados por ARCO ao redor do anel FECHADO `ring` (inclui a aresta de
/// fecho). Uniformizar por arco é o que faz a correlação comparar forma-com-forma, e não a
/// densidade acidental de vértices de um lado contra o do outro.
fn resample_arc(ring: &[Vec2], k: usize) -> Vec<Vec2> {
    let n = ring.len();
    if n == 0 || k == 0 {
        return vec![Vec2::ZERO; k];
    }
    let seg: Vec<f32> = (0..n)
        .map(|i| (ring[(i + 1) % n] - ring[i]).length())
        .collect();
    let total: f32 = seg.iter().sum();
    if total <= 0.0 {
        return vec![ring[0]; k]; // anel degenerado (todos os pontos coincidem)
    }
    let step = total / k as f32;
    let mut out = Vec::with_capacity(k);
    let mut si = 0usize; // segmento atual (0..n; `n-1` é o de fecho)
    let mut base = 0.0f32; // arco acumulado ATÉ o início do segmento `si`
    for j in 0..k {
        let target = j as f32 * step;
        while si + 1 < n && base + seg[si] <= target {
            base += seg[si];
            si += 1;
        }
        let f = ((target - base) / seg[si].max(1e-12)).clamp(0.0, 1.0);
        let (p, q) = (ring[si], ring[(si + 1) % n]);
        out.push(p + (q - p) * f);
    }
    out
}

/// Subtrai o centróide (a translação entre A e B não é assunto da FASE — a espiral a resolve).
fn centered(ring: &[Vec2]) -> Vec<Vec2> {
    let n = ring.len();
    if n == 0 {
        return Vec::new();
    }
    let c = ring.iter().fold(Vec2::ZERO, |a, &p| a + p) / n as f32;
    ring.iter().map(|&p| p - c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    /// Um anel-blob: um círculo de raio `r` com um "nariz" (bump) na direção `+X`, começando pelo
    /// vértice `start` (a COSTURA), transladado por `off`. A e B usam o MESMO blob (mesma forma no
    /// mundo), só a costura e a posição mudam — então a fase certa faz o pareamento coincidir.
    fn blob(n: usize, r: f32, start: usize, off: Vec2) -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let a = ((i + start) % n) as f32 / n as f32 * TAU;
                let bump = 1.0 + 0.5 * (a.cos().max(0.0)).powi(6); // nariz assimétrico
                off + Vec2::new(a.cos() * r * bump, a.sin() * r * bump)
            })
            .collect()
    }

    /// O quanto os pontos pareados se afastam sob o deslocamento `sh` (a régua de "torceu?").
    fn travel(a: &[Vec2], b: &[Vec2], sh: usize) -> f32 {
        let n = a.len();
        (0..n)
            .map(|i| (a[i] - b[(i + sh) % n]).length_squared())
            .sum()
    }

    /// 🔴 **A fase realinha uma costura MUITO deslocada.** B é o MESMO blob de A com a costura
    /// movida `k` — o pareamento por índice torceria. A correlação acha o deslocamento (≠ 0) e ele
    /// reduz DRAMATICAMENTE o afastamento dos pares. É o núcleo da wave.
    ///
    /// Mutação que sangra: `seam_shift → 0` ⇒ `travel(s) == travel(0)`.
    #[test]
    fn the_seam_shift_realigns_a_moved_seam() {
        let n = 48;
        for k in [16usize, 20, 24, 28] {
            let a = blob(n, 40.0, 0, Vec2::ZERO);
            let b = blob(n, 40.0, k, Vec2::ZERO);
            let s = seam_shift(&a, &b);
            assert_ne!(s, 0, "k={k}: a costura muito deslocada não foi detectada");
            assert!(
                travel(&a, &b, s) < 0.15 * travel(&a, &b, 0),
                "k={k}: alinhar (s={s}) mal ajudou — {:.1} vs identidade {:.1}",
                travel(&a, &b, s),
                travel(&a, &b, 0)
            );
        }
    }

    /// 🔴 **A fase NUNCA aumenta o trajeto** — a invariante que mata o bug do arco. O trajeto sob
    /// o deslocamento devolvido é `≤` o da identidade, para QUALQUER par (a identidade é sempre um
    /// candidato do `min`). A versão da virada VIOLAVA isso: otimizava a virada, não o trajeto, e
    /// devolvia deslocamentos que AUMENTAVAM o afastamento (o giro espúrio que arqueava).
    ///
    /// Mutação que sangra: devolver um deslocamento fixo `n/6` (um giro espúrio) ⇒ trajeto sobe.
    #[test]
    fn the_phase_never_increases_the_travel() {
        let n = 50;
        // Vários deslocamentos de costura, inclusive os PEQUENOS que o caso de mão tem.
        for k in [0usize, 1, 2, 3, 5, 8, 12, 20, 30] {
            let a = blob(n, 40.0, 0, Vec2::ZERO);
            let b = blob(n, 40.0, k, Vec2::new(120.0, 0.0)); // mesma forma, movida (como o Enio)
            let s = seam_shift(&a, &b);
            assert!(
                travel(&a, &b, s) <= travel(&a, &b, 0) + 1e-3,
                "k={k}: a fase (s={s}) AUMENTOU o trajeto — {:.1} vs identidade {:.1} (arco espúrio)",
                travel(&a, &b, s),
                travel(&a, &b, 0)
            );
        }
    }

    /// **Um anel simétrico não é mexido.** Um círculo tem um mínimo de trajeto CLARO em `s=0` (não
    /// o custo chato da virada) ⇒ a identidade vence, e a rotação limpa da espiral não vira giro.
    #[test]
    fn a_symmetric_ring_is_left_alone() {
        let n = 48;
        let circle = |start: usize| -> Vec<Vec2> {
            (0..n)
                .map(|i| {
                    let a = ((i + start) % n) as f32 / n as f32 * TAU;
                    Vec2::new(a.cos() * 40.0, a.sin() * 40.0)
                })
                .collect()
        };
        assert_eq!(seam_shift(&circle(0), &circle(0)), 0);
    }

    /// **Anel pequeno demais ⇒ identidade.** Protege os gates de furo do `tween` (quadrados de 4
    /// pontos) e é honesto: abaixo de um octógono a fase é ruído.
    #[test]
    fn a_small_ring_is_left_alone() {
        let square = |s: usize| {
            let base = [
                Vec2::new(-10.0, -10.0),
                Vec2::new(10.0, -10.0),
                Vec2::new(10.0, 10.0),
                Vec2::new(-10.0, 10.0),
            ];
            (0..4).map(|i| base[(i + s) % 4]).collect::<Vec<_>>()
        };
        assert_eq!(seam_shift(&square(0), &square(2)), 0);
    }

    /// **A régua** (`cargo test -p ph2d-flip --release the_phase_ruler -- --ignored --nocapture`):
    /// o custo é do TETO da grade, não do número de pontos.
    #[test]
    #[ignore = "régua: custo da fase vs tamanho do anel"]
    fn the_phase_ruler() {
        println!("\n  n pontos   steps (grade)   avaliações da correlação (steps²)");
        for n in [16usize, 48, 100, 200, 400, 800] {
            let steps = n.min(PHASE_STEPS);
            println!("  {n:^8}   {steps:^13}   {:^12}", steps * steps);
        }
        println!(
            "\n  o custo satura em {}² = {} — um anel de 800 pontos custa o mesmo que um de 96.\n",
            PHASE_STEPS,
            PHASE_STEPS * PHASE_STEPS
        );
    }
}
