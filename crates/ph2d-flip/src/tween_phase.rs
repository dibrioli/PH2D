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
//! contorno e a forma do meio **torce** (um "O" vira um oito no quadro do meio).
//!
//! O [`crate::tween_flip`] resolve o *sentido* do percurso (winding); isto resolve a *fase*.
//!
//! # Como
//!
//! É uma **correlação circular** — a mesma técnica que o `ph2d-vec-blend` usa no Blend do
//! vetor. O sinal, porém, é adaptado ao nosso caso: lá as formas não são pré-rotacionadas e
//! ele correlaciona POSIÇÕES; aqui a espiral ([`crate::tween_spiral`]) tira o rígido
//! *depois*, então a fase tem de ser **invariante à rotação** — correlacionamos a **virada**
//! `(sen, cos)` em cada amostra de arco (invariante a rotação/escala/translação, e sem
//! `atan2`, HR-5). A fase é a virada que faz as duas viradas coincidirem.
//!
//! # Por que é OPT-IN pela geometria
//!
//! A fase só cede da identidade (deslocamento 0) por um ganho **decisivo** ([`ACCEPT_MARGIN`]).
//! Um anel simétrico (um quadrado, um círculo) tem várias costuras igualmente boas — um custo
//! chato — e mexer nele por ruído de `f64` giraria o par e transformaria a rotação limpa que a
//! espiral faria numa reflexão. Só uma costura CLARAMENTE desalinhada (o bug reportado) reduz o
//! custo o bastante. É a mesma lição do z-order desta linha: **não se escolhe um desempate — não
//! se tem empate.**

use ph2d_core::Vec2;

/// Resolução da correlação circular (grade de arco). Irmão do `PHASE_STEPS = 256` do
/// `ph2d-vec-blend`; menor porque um traço do Flip é uma polilinha de mão, não um contorno
/// cozido, e a costura só precisa cair dentro de ~1% do perímetro. **O custo é `O(steps²)`**,
/// e por ser um TETO da grade (não do número de pontos) ele é constante — a régua
/// `the_phase_ruler` mede que anéis grandes não o movem.
const PHASE_STEPS: usize = 96;

/// Abaixo disto a fase é ruído: um triângulo tem 3 costuras plausíveis e nenhuma torce
/// visivelmente, e um "anel" de 4 pontos é grosso demais para a virada dizer alguma coisa.
/// Devolve 0 — o comportamento de sempre. (É também o que mantém byte-idênticos os gates de
/// furo do `tween`, que usam quadrados de 4 pontos.)
const MIN_RING: usize = 8;

/// Quanto o vale tem de ser DECISIVO para a identidade (deslocamento 0) ceder — o ganho de
/// `custo(0)` para o fundo, em fração da AMPLITUDE do custo (não de `custo(0)`, que pode ser
/// ~0 por ruído e daria uma razão sem sentido). Ver o cabeçalho do módulo.
const ACCEPT_MARGIN: f32 = 0.25;

/// Abaixo desta amplitude o custo é CHATO — o sinal não tem feature (um círculo tem a mesma
/// virada em todo ponto), então não há costura a achar e qualquer "melhor fase" é ruído de
/// `f64`. Bem acima do ruído (~1e-12 na grade cheia) e bem abaixo de qualquer feature real
/// (uma quina vale ~O(1)).
const FLAT_EPS: f32 = 1e-3;

/// **O deslocamento cíclico `s` em `0..n`** tal que parear `a[i]` com `b[(i + s) % n]` segue a
/// FORMA, e não a costura arbitrária do ponto 0. Ambos os anéis têm de ter `len n` e ser
/// fechados (o chamador garante). Devolve `0` — a identidade — quando a fase é ambígua, o ganho
/// é pequeno, ou o anel é pequeno demais: o alinhamento nunca é imposto.
pub(crate) fn seam_shift(a: &[Vec2], b: &[Vec2]) -> usize {
    let n = a.len();
    if n < MIN_RING || b.len() != n {
        return 0;
    }
    let steps = n.min(PHASE_STEPS);
    let ta = turning(&resample_arc(a, steps));
    let tb = turning(&resample_arc(b, steps));

    // custo(p) = Σ_k |ta[k] − tb[(k+p) % steps]|²  (distância entre os dois pares (sen,cos)).
    let cost = |p: usize| -> f32 {
        (0..steps)
            .map(|k| {
                let (ax, ay) = ta[k];
                let (bx, by) = tb[(k + p) % steps];
                let (dx, dy) = (ax - bx, ay - by);
                dx * dx + dy * dy
            })
            .sum()
    };

    let c0 = cost(0);
    let (mut best, mut cbest, mut cmax) = (0usize, c0, c0);
    for p in 1..steps {
        let c = cost(p);
        if c < cbest {
            cbest = c;
            best = p;
        }
        if c > cmax {
            cmax = c;
        }
    }

    // A identidade (deslocamento 0) só cede a um vale DECISIVO. Duas guardas:
    //  · `spread` chato (círculo/quadrado: custo igual em toda fase) ⇒ não há costura a achar,
    //    e mexer seria ruído de `f64` girando um par simétrico;
    //  · o ganho da identidade para o fundo tem de ser grande FRENTE à amplitude do custo —
    //    senão a costura já está ~alinhada e o melhor é deixá-la como está.
    let spread = cmax - cbest;
    if best == 0 || spread < FLAT_EPS || (c0 - cbest) <= ACCEPT_MARGIN * spread {
        return 0;
    }
    // `best` mora na grade `steps`; leva-o de volta ao índice `n` (arredondando).
    ((best * n + steps / 2) / steps) % n
}

/// `k` pontos igualmente espaçados por ARCO ao redor do anel FECHADO `ring` (inclui a aresta
/// de fecho). Uniformizar por arco é o que faz a correlação comparar forma-com-forma, e não a
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

/// A **virada** `(sen, cos)` em cada ponto do anel (vizinhos cíclicos) — o par unitário do
/// ângulo exterior, como o `turns` do `ph2d-vec-blend`. Invariante a rotação/escala/translação,
/// e sem `atan2` (HR-5). Numa amostra suave a virada é `(0, 1)`, que não perturba a correlação.
fn turning(ring: &[Vec2]) -> Vec<(f32, f32)> {
    let n = ring.len();
    (0..n)
        .map(|i| {
            let prev = unit(ring[i] - ring[(i + n - 1) % n]);
            let next = unit(ring[(i + 1) % n] - ring[i]);
            match (prev, next) {
                (Some(u), Some(v)) => (u.x * v.y - u.y * v.x, u.x * v.x + u.y * v.y),
                _ => (0.0, 1.0), // ponto duplicado: sem virada
            }
        })
        .collect()
}

/// O unitário de `v`, ou `None` se `v` é (quase) nulo.
fn unit(v: Vec2) -> Option<Vec2> {
    let l = v.length();
    (l > 1e-9).then(|| v / l)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    /// Um anel-blob: um círculo de raio `r` com um "nariz" (bump) na direção `+X`, começando
    /// pelo vértice de índice `start` (a COSTURA deslocada), transladado por `off`. É a arte
    /// dos testes: A e B são o MESMO blob (mesma forma no mundo), só a costura e a posição
    /// mudam — então a fase certa faz o pareamento coincidir ponto a ponto.
    fn blob(n: usize, r: f32, start: usize, off: Vec2) -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let a = ((i + start) % n) as f32 / n as f32 * TAU;
                // nariz: +50% de raio concentrado perto de a=0 (assimétrico ⇒ fase sem ambiguidade).
                let bump = 1.0 + 0.5 * (a.cos().max(0.0)).powi(6);
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

    /// 🔴 **A fase realinha a costura.** B é o MESMO blob de A, com a costura movida `k` — então
    /// existe um deslocamento que faz o pareamento coincidir ponto a ponto. A correlação tem de
    /// ACHÁ-lo (deslocamento ≠ 0) e ele tem de reduzir DRAMATICAMENTE o afastamento dos pares.
    /// É o núcleo da wave. (O valor exato não é o contrato — a correlação trabalha em ARCO, e o
    /// blob tem arco não-uniforme; o que importa é que o pareamento passa a seguir a forma.)
    ///
    /// Mutação que sangra: `seam_shift` devolvendo 0 sempre ⇒ `travel(s) == travel(0)`.
    #[test]
    fn the_seam_shift_realigns_the_seam() {
        let n = 48;
        for k in [12usize, 20, 24, 33] {
            let a = blob(n, 40.0, 0, Vec2::ZERO);
            let b = blob(n, 40.0, k, Vec2::ZERO); // MESMO blob, costura movida k
            let s = seam_shift(&a, &b);
            assert_ne!(s, 0, "k={k}: a costura desalinhada não foi detectada");
            assert!(
                travel(&a, &b, s) < 0.15 * travel(&a, &b, 0),
                "k={k}: alinhar (s={s}) mal ajudou — {:.1} vs identidade {:.1}",
                travel(&a, &b, s),
                travel(&a, &b, 0)
            );
        }
    }

    /// **É invariante à ROTAÇÃO** — a espiral tira o rígido depois, então a fase não pode
    /// absorver o giro do mundo no índice. A fase de B e a de B girado 90° têm de ser IGUAIS:
    /// o giro não muda a virada, então não pode mudar a costura escolhida.
    ///
    /// Mutação que sangra: correlacionar POSIÇÕES em vez da virada ⇒ o giro de 90° entra na
    /// conta e as duas fases divergem.
    #[test]
    fn the_signal_is_rotation_invariant() {
        let n = 48;
        let k = 16;
        let a = blob(n, 40.0, 0, Vec2::ZERO);
        let b = blob(n, 40.0, k, Vec2::ZERO);
        let b_rot: Vec<Vec2> = b.iter().map(|p| Vec2::new(-p.y, p.x)).collect(); // +90°
        let s_flat = seam_shift(&a, &b);
        assert_ne!(s_flat, 0, "premissa: a costura movida é detectada");
        assert_eq!(
            seam_shift(&a, &b_rot),
            s_flat,
            "a rotação de 90° vazou para a fase (o sinal não é invariante)"
        );
    }

    /// **Um anel simétrico não é mexido por ruído.** Um círculo (sem nariz) tem custo chato em
    /// TODA fase; a margem de aceitação mantém a identidade. Sem ela, `f64` escolheria um
    /// deslocamento qualquer e a espiral leria a rotação limpa como reflexão.
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
        // Mesmo com a costura movida, um círculo é o mesmo em toda fase ⇒ não há ganho decisivo.
        assert_eq!(seam_shift(&circle(0), &circle(12)), 0);
    }

    /// **Anel pequeno demais ⇒ identidade.** Protege os gates de furo do `tween` (quadrados de
    /// 4 pontos) e é honesto: abaixo de um octógono a fase é ruído.
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
    /// o custo é do TETO da grade, não do número de pontos. É o que justifica o `PHASE_STEPS`
    /// fixo em vez de correlacionar no `n` cru.
    #[test]
    #[ignore = "régua: custo da fase vs tamanho do anel"]
    fn the_phase_ruler() {
        // Sem relógio de parede aqui (o harness proíbe `Instant` determinístico nos gates);
        // a régua reporta o TRABALHO: `steps²` avaliações, e `steps = min(n, PHASE_STEPS)`.
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
