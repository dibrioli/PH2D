//! Traços de edição aleatórios: cada troca aplicada e invertida devolve o texto BYTE A BYTE, as
//! `TextEdit` descrevem o texto novo, e desfazer tudo / refazer tudo percorre a mesma história.

use ph2d_code_buffer::{Buffer, Change, ChangeSet, EditKind, Point, Text};

const ALFABETO: &[&str] = &["a", "b", " ", "\n", "\t", "é", "😀", "end", "\nfunction"];

fn boundary(s: &str, rng: &mut fastrand::Rng) -> usize {
    let mut b = rng.usize(0..=s.len());
    while !s.is_char_boundary(b) {
        b -= 1;
    }
    b
}

fn random_changes(s: &str, rng: &mut fastrand::Rng) -> ChangeSet {
    let n = rng.usize(1..=4);
    let mut cuts: Vec<usize> = (0..n * 2).map(|_| boundary(s, rng)).collect();
    cuts.sort_unstable();
    let changes = cuts
        .chunks(2)
        .map(|p| Change {
            range: p[0]..p[1],
            insert: (0..rng.usize(0..3))
                .map(|_| ALFABETO[rng.usize(0..ALFABETO.len())])
                .collect(),
        })
        .collect();
    ChangeSet::new(changes).expect("cortes ordenados nunca se sobrepõem")
}

fn naive_apply(s: &str, cs: &ChangeSet) -> String {
    let mut out = s.to_string();
    for c in cs.changes().iter().rev() {
        out.replace_range(c.range.clone(), &c.insert);
    }
    out
}

fn naive_point(s: &str, b: usize) -> Point {
    let row = s[..b].matches('\n').count();
    let col = b - s[..b].rfind('\n').map_or(0, |i| i + 1);
    Point { row, col }
}

#[test]
fn aplicar_e_inverter_devolve_o_texto_byte_a_byte() {
    let mut rng = fastrand::Rng::with_seed(0xC0DE);
    let mut s = String::from("local x = 1\nfunction update(self, dt)\n\tprint(x)\nend\n");
    for _ in 0..2_000 {
        let mut b = Buffer::new(&s);
        let before = b.text().clone();
        let cs = random_changes(&s, &mut rng);
        let expected = naive_apply(&s, &cs);
        let inverse = cs.invert(&before);
        let inserts: Vec<String> = cs
            .changes()
            .iter()
            .rev()
            .map(|c| c.insert.clone())
            .collect();
        let edits = b.apply(cs, None, EditKind::Other);
        assert_eq!(b.to_string(), expected);
        // As TextEdit, aplicadas em sequência a uma String, dão o mesmo texto e os pontos certos.
        let mut seq = s.clone();
        assert_eq!(edits.len(), inserts.len());
        for (e, ins) in edits.iter().zip(&inserts) {
            assert_eq!(e.start, naive_point(&seq, e.start_byte));
            assert_eq!(e.old_end, naive_point(&seq, e.old_end_byte));
            assert_eq!(e.new_end_byte - e.start_byte, ins.len());
            seq.replace_range(e.start_byte..e.old_end_byte, ins);
            assert_eq!(e.new_end, naive_point(&seq, e.new_end_byte));
        }
        assert_eq!(seq, expected);
        let mut back = Buffer::new(&expected);
        back.apply(inverse, None, EditKind::Other);
        assert_eq!(back.to_string(), s, "inverter não devolveu o texto");
        s = Text::new(&expected).to_string();
        if s.len() > 4_000 {
            s.truncate(boundary(&s[..2_000], &mut rng));
        }
    }
}

#[test]
fn desfazer_tudo_e_refazer_tudo_percorrem_a_mesma_historia() {
    let mut rng = fastrand::Rng::with_seed(7);
    let mut b = Buffer::new("print(1)\n");
    let mut snapshots = vec![b.to_string()];
    for _ in 0..300 {
        let cs = random_changes(&b.to_string(), &mut rng);
        // Um ChangeSet vazio (inserção vazia num ponto) não é passo: não entra no histórico.
        if !b.apply(cs, None, EditKind::Other).is_empty() {
            snapshots.push(b.to_string());
        }
    }
    for want in snapshots.iter().rev().skip(1) {
        assert!(!b.undo().is_empty());
        assert_eq!(&b.to_string(), want);
    }
    assert!(b.undo().is_empty(), "o histórico tinha passos a mais");
    for want in snapshots.iter().skip(1) {
        b.redo();
        assert_eq!(&b.to_string(), want);
    }
}
