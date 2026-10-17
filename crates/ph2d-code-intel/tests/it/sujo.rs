//! A faixa suja de um lote de edições SEQUENCIAIS é uma troca única que leva o texto de antes ao de
//! depois — sobre lotes aleatórios (o `didChange` incremental depende disto).

use ph2d_code_buffer::{Point, TextEdit};
use ph2d_code_intel::Dirty;

fn ponto(s: &str, b: usize) -> Point {
    let antes = &s[..b];
    let row = antes.matches('\n').count();
    let col = b - antes.rfind('\n').map_or(0, |i| i + 1);
    Point { row, col }
}

#[test]
fn a_faixa_suja_reconstroi_o_texto_novo_em_lotes_aleatorios() {
    let alfabeto = ["a", "é", "\n", "xy", ""];
    for semente in 0..2_000u64 {
        let mut rng = fastrand::Rng::with_seed(semente);
        let antes: String = (0..rng.usize(0..40))
            .map(|_| alfabeto[rng.usize(0..4)])
            .collect();
        let mut agora = antes.clone();
        let mut edits = Vec::new();
        for _ in 0..rng.usize(1..5) {
            let fronteiras: Vec<usize> = (0..=agora.len())
                .filter(|b| agora.is_char_boundary(*b))
                .collect();
            let mut a = fronteiras[rng.usize(0..fronteiras.len())];
            let mut b = fronteiras[rng.usize(0..fronteiras.len())];
            if a > b {
                std::mem::swap(&mut a, &mut b);
            }
            let ins = alfabeto[rng.usize(0..alfabeto.len())];
            let start = ponto(&agora, a);
            let old_end = ponto(&agora, b);
            agora.replace_range(a..b, ins);
            edits.push(TextEdit {
                start_byte: a,
                old_end_byte: b,
                new_end_byte: a + ins.len(),
                start,
                old_end,
                new_end: ponto(&agora, a + ins.len()),
            });
        }
        let d = Dirty::of(&edits, antes.len()).expect("lote não vazio");
        let mut refeito = antes.clone();
        refeito.replace_range(d.old.clone(), &agora[d.new.clone()]);
        assert_eq!(refeito, agora, "semente {semente}: {d:?} sobre {antes:?}");
    }
}
