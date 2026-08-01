//! Gates do espaçamento — puros, sem malha e sem câmera.

use super::*;

/// O laço que um chamador TEM de fazer: manter a âncora, e movê-la **só** quando
/// carimbou. É a sequência que produz as propriedades abaixo; nenhuma delas é
/// observável num `walk` isolado, porque o carry é sobre a MEMÓRIA entre eventos.
fn deposit(path: &[[f32; 2]], min_spacing: f32) -> Vec<[f32; 2]> {
    let mut anchor = path[0];
    let mut out = vec![anchor];
    for &p in &path[1..] {
        if let Some(w) = walk(anchor, p, min_spacing) {
            out.extend(w);
            anchor = p;
        }
    }
    out
}

/// Amostra um segmento reto em `n` eventos — o mesmo caminho geométrico,
/// entregue em taxas de polling diferentes.
fn polled(n: usize, len: f32) -> Vec<[f32; 2]> {
    (0..=n).map(|i| [len * i as f32 / n as f32, 0.0]).collect()
}

/// A maior distância entre dois dabs consecutivos — **o que o artista vê como
/// buraco**.
fn worst_gap(dabs: &[[f32; 2]]) -> f32 {
    dabs.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .fold(0.0f32, f32::max)
}

#[test]
fn a_gesture_shorter_than_the_spacing_deposits_nothing() {
    assert_eq!(walk([0.0, 0.0], [3.0, 0.0], 5.0), None);
    // E na fronteira exata também não: `<=`, porque um dab a distância exata do
    // anterior é o passo que o gesto SEGUINTE vai dar.
    assert_eq!(walk([0.0, 0.0], [5.0, 0.0], 5.0), None);
    assert!(walk([0.0, 0.0], [5.1, 0.0], 5.0).is_some());
}

/// ⚠️ **A ENTREGA da fatia, e o oráculo é ESTRUTURAL em vez de estatístico.**
///
/// A tentação é medir *volume depositado* e exigir que duas taxas de polling
/// concordem a 5% — e esse gate é uma **flake esperando fixture nova**: a lei
/// põe o espaçamento em `[ms, 2·ms)`, então ela **não entrega** independência de
/// amostragem melhor que um fator 2, e uma métrica de volume sente isso (medido
/// no mapeamento desta wave: 1,021× num raio e 0,960× noutro, contra barra de
/// 5%). O que a lei entrega EXATAMENTE é o teto do vão — e é o vão, não o
/// volume, que o artista vê.
#[test]
fn the_deposit_is_a_fact_of_the_path_not_of_the_pointer_polling_rate() {
    const MS: f32 = 6.0;
    const LEN: f32 = 640.0;
    let mut seen = Vec::new();
    for n in [4usize, 8, 16, 64, 640] {
        let dabs = deposit(&polled(n, LEN), MS);
        let gap = worst_gap(&dabs);
        assert!(
            gap <= 2.0 * MS,
            "com {n} eventos o pior vão foi {gap}, acima do teto de {}",
            2.0 * MS
        );
        // E o traço chega ao fim: nenhum resíduo engolido.
        assert!((dabs.last().expect("dabs")[0] - LEN).abs() < MS);
        seen.push((n, dabs.len(), gap));
    }
    // O CONTROLE, sem o qual o gate não pode falhar: um dab por EVENTO (o
    // produto antes desta fatia) deixa um vão do tamanho do salto do ponteiro.
    let naive = worst_gap(&polled(4, LEN));
    assert!(
        naive > 20.0 * MS,
        "o controle tem de conter o fenômeno, e mediu {naive}"
    );
    // A contagem também converge, o que é o teto do vão dito de outro jeito.
    let counts: Vec<usize> = seen.iter().map(|s| s.1).collect();
    let (lo, hi) = (
        *counts.iter().min().expect("n"),
        *counts.iter().max().expect("n"),
    );
    assert!(
        hi <= 2 * lo,
        "as contagens por taxa de polling foram {counts:?}"
    );
}

/// O caso que o carry existe para cobrir: um gesto **lento**, cujos eventos são
/// menores que o espaçamento. Sem carry, cada evento carimbaria um dab e um
/// traço lento ficaria dez vezes mais denso que o mesmo traço rápido.
#[test]
fn a_slow_gesture_carries_its_residue_instead_of_depositing_every_event() {
    const MS: f32 = 10.0;
    // Passos de 2 px: cada um sozinho é carry puro.
    let slow = deposit(&polled(320, 640.0), MS);
    let fast = deposit(&polled(16, 640.0), MS);
    assert!(
        slow.len() <= 2 * fast.len(),
        "lento depositou {} dabs contra {} do rápido",
        slow.len(),
        fast.len()
    );
    assert!(worst_gap(&slow) <= 2.0 * MS);
}

/// O último dab pousa **no ponteiro**, não perto dele — é o que impede a ponta
/// do traço de ficar para trás da mão.
#[test]
fn the_last_dab_lands_exactly_on_the_pointer() {
    let to = [37.5, -12.25];
    let last = walk([0.0, 0.0], to, 3.0)
        .expect("anda")
        .last()
        .expect("dab");
    assert_eq!(last, to);
}

/// O espaçamento é do PINCEL: dobrar o raio dobra o passo, então a sobreposição
/// aparente é a mesma em qualquer tamanho.
#[test]
fn the_spacing_is_a_fraction_of_the_brush() {
    assert!((min_spacing(20.0) / min_spacing(10.0) - 2.0).abs() < 1e-6);
    assert!((min_spacing(100.0) - 15.0).abs() < 1e-4);
}

/// Um gesto absurdo (uma janela arrastada de canto a canto com pincel de 1 px)
/// não pode carimbar um milhão de dabs num frame.
#[test]
fn an_enormous_jump_is_bounded_instead_of_unbounded() {
    let w = walk([0.0, 0.0], [1.0e9, 0.0], 0.15).expect("anda");
    assert!(w.len() <= u32::from(u16::MAX), "{} dabs", w.len());
}

/// ⚠️ **O gate que uma mutação sobrevivente pediu, e a medição que o desenhou.**
///
/// A primeira mutação deste módulo — *acumular `t` em float, como o laço do
/// original* — passou por todos os outros gates, e a causa não era buraco de
/// oráculo: ela reproduzia só METADE do defeito. Medido sobre `n` em 1..4000:
/// somar `1/n` n vezes **não fecha em 1,0 em 3981 casos**, mas o erro é ~2e-16 e
/// o `lerp` o absorve ao converter para `f32` — invisível na POSIÇÃO. A metade
/// grande é a condição de parada: `for (i = step; i <= 1.0; i += step)` **perde
/// um dab inteiro em 1923 dos 4000**, incluindo n = 9, 11, 18, 20, 21 e 25.
///
/// Um dab a menos num gesto é um vão de `min_spacing` na PONTA do traço, e ele
/// aparece ou não conforme o número de passos — que é ruído de amostragem, o
/// que a lei do traço existe para eliminar. Daí o índice inteiro.
#[test]
fn the_walk_deposits_every_step_it_promises() {
    // Os `n` medidos como perdedores, mais vizinhos sãos como controle.
    for n in [8u32, 9, 10, 11, 12, 18, 20, 21, 25] {
        const MS: f32 = 4.0;
        // Um caminho cujo comprimento pede exatamente `n` passos: um meio-passo
        // de folga o mantém longe da fronteira do `floor`.
        let len = MS * (n as f32 + 0.5);
        let w = walk([0.0, 0.0], [len, 0.0], MS).expect("anda");
        assert_eq!(w.len(), n, "o walk prometeu {n} dabs");
        let dabs: Vec<[f32; 2]> = w.collect();
        assert_eq!(dabs.len(), n as usize, "e tem de ENTREGAR os {n}");
        assert_eq!(
            dabs.last().expect("dab")[0],
            len,
            "o último dab de {n} passos tem de pousar no ponteiro"
        );
    }
}
