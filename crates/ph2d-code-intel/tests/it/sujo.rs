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

/// Sem as edições (o texto trocado por fora), a troca é o prefixo e o sufixo comuns: refaz o texto
/// novo, corta só em fronteiras de carácter (`é`/`ê` partilham o 1.º byte) e textos iguais dão
/// `None`.
#[test]
fn a_troca_entre_dois_textos_refaz_o_novo_em_fronteiras_de_caracter() {
    use ph2d_code_buffer::Text;
    // `é`/`ê` partilham o 1.º byte (C3) e `é`/`©` o último (A9): os dois cortes a meio de carácter.
    let alfabeto = ["a", "é", "ê", "©", "\n", "😀", "xy"];
    for semente in 0..2_000u64 {
        let mut rng = fastrand::Rng::with_seed(semente);
        let mut gera = |n: usize| -> String {
            (0..n)
                .map(|_| alfabeto[rng.usize(0..alfabeto.len())])
                .collect()
        };
        let (pre, meio_a, meio_b, suf) = (gera(5), gera(3), gera(3), gera(5));
        let antes = format!("{pre}{meio_a}{suf}");
        let agora = format!("{pre}{meio_b}{suf}");
        let (ta, tb) = (Text::new(&antes), Text::new(&agora));
        let Some(d) = Dirty::between(&ta, &tb) else {
            assert_eq!(antes, agora, "semente {semente}");
            continue;
        };
        assert!(antes.is_char_boundary(d.old.start) && antes.is_char_boundary(d.old.end));
        assert!(agora.is_char_boundary(d.new.start) && agora.is_char_boundary(d.new.end));
        let mut refeito = antes.clone();
        refeito.replace_range(d.old.clone(), &agora[d.new.clone()]);
        assert_eq!(refeito, agora, "semente {semente}: {d:?}");
        // Mínima: o que fica de fora é igual nos dois (o prefixo comum nunca é cortado sem razão).
        assert!(
            d.old.start >= antes.len().min(pre.len()).saturating_sub(4),
            "semente {semente}"
        );
    }
    assert_eq!(Dirty::between(&Text::new("abc"), &Text::new("abc")), None);
    let d = Dirty::between(&Text::new("xé"), &Text::new("xê")).unwrap();
    assert_eq!((d.old, d.new), (1..3, 1..3));
    let d = Dirty::between(&Text::new("éx"), &Text::new("©x")).unwrap();
    assert_eq!((d.old, d.new), (0..2, 0..2));
}
