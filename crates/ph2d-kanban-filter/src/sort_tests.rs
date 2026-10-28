use ph2d_kanban_model::{CardId, CardRef, Due, Moment, Record, mirror_card};

use super::*;
use crate::test_support::{DAY, NOW, add_card, add_list};

fn titles(set: &KanbanSet, k: KanbanId, l: ListId) -> Vec<String> {
    set.entries_in(k, l)
        .into_iter()
        .map(|e| set.card(e.shows(k)).unwrap().title.clone())
        .collect()
}

fn due(d: i64) -> Option<Due> {
    Some(Due {
        at: Moment(NOW.0 + d * DAY),
        done: false,
        remind_before_min: None,
    })
}

fn scene() -> (KanbanSet, KanbanId, ListId) {
    let mut set = KanbanSet::default();
    let k = set.create("K");
    let l = add_list(&mut set, k, "L");
    let rows = [
        ("delta", 3, Some(2)),
        ("Álvaro", 1, None),
        ("charlie", 4, Some(1)),
        ("bravo", 2, Some(5)),
    ];
    for (t, created, d) in rows {
        add_card(&mut set, k, l, t, Moment(created), |c| {
            c.due = d.and_then(due)
        });
    }
    (set, k, l)
}

fn sorted(by: SortBy) -> Vec<String> {
    let (mut set, k, l) = scene();
    let ops = sort_list(&set, k, l, by);
    set.apply(k, ops);
    titles(&set, k, l)
}

#[test]
fn sort_list_by_created_name_and_due() {
    assert_eq!(
        sorted(SortBy::Newest),
        ["charlie", "delta", "bravo", "Álvaro"]
    );
    assert_eq!(
        sorted(SortBy::Oldest),
        ["Álvaro", "bravo", "delta", "charlie"]
    );
    assert_eq!(
        sorted(SortBy::Name),
        ["Álvaro", "bravo", "charlie", "delta"]
    );
    assert_eq!(
        sorted(SortBy::Due),
        ["charlie", "delta", "bravo", "Álvaro"],
        "sem data no fim"
    );
}

#[test]
fn sort_rewrites_only_the_cards_that_move() {
    let (mut set, k, l) = scene();
    // delta, Álvaro, charlie, bravo → por nome: só «delta» e mais um mudam de sítio.
    let keys = sort_keys(&set, k, l, SortBy::Name);
    assert_eq!(
        keys.len(),
        2,
        "fica parada a maior subsequência já em ordem"
    );
    let ops = sort_list(&set, k, l, SortBy::Name);
    set.apply(k, ops);
    assert!(
        sort_list(&set, k, l, SortBy::Name).is_empty(),
        "ordenada: nada a escrever"
    );
}

#[test]
fn a_mirror_sorts_by_its_original() {
    let (mut set, k, l) = scene();
    let other = set.create("O");
    let ol = add_list(&mut set, other, "X");
    let zulu = add_card(&mut set, other, ol, "aardvark", Moment(9), |_| {});
    let at = set.place_at(k, l, 9, None).unwrap();
    let of = CardRef {
        kanban: other,
        card: zulu,
    };
    for (kk, ops) in mirror_card(&set, of, &at).unwrap().into_steps() {
        set.apply(kk, ops);
    }
    let ops = sort_list(&set, k, l, SortBy::Name);
    set.apply(k, ops);
    assert_eq!(
        titles(&set, k, l),
        ["aardvark", "Álvaro", "bravo", "charlie", "delta"]
    );
}

// ---- R1: ordenar é O(n log n), com o MESMO resultado da versão O(n²) de antes ----

/// Gerador determinista (LCG).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).unwrap() % n.max(1)
    }
}

/// A versão de antes (O(n²)), o oráculo do resultado.
fn longest_kept_quadratic(pos: &[&FracKey], rank: &[usize]) -> Vec<bool> {
    let n = rank.len();
    let mut len = vec![1_usize; n];
    let mut prev = vec![usize::MAX; n];
    for i in 0..n {
        for j in 0..i {
            if rank[j] < rank[i] && pos[j] < pos[i] && len[j] + 1 > len[i] {
                len[i] = len[j] + 1;
                prev[i] = j;
            }
        }
    }
    let mut keep = vec![false; n];
    let mut at = (0..n).max_by_key(|&i| (len[i], std::cmp::Reverse(i)));
    while let Some(i) = at {
        keep[i] = true;
        at = (prev[i] != usize::MAX).then(|| prev[i]);
    }
    keep
}

/// `n` cartões numa lista só, por um lote: posições com EMPATES (`distinct` chaves) e criação ao
/// acaso; `reversed` = criados do mais novo para o mais velho (o pior caso de «mais antigo primeiro»).
fn big(n: usize, distinct: usize, seed: u64, reversed: bool) -> (KanbanSet, KanbanId, ListId) {
    let mut set = KanbanSet::default();
    let k = set.create("K");
    let l = add_list(&mut set, k, "L");
    let mut rng = Lcg(seed);
    let mut keys = vec![FracKey::between(None, None)];
    while keys.len() < distinct.max(1) {
        let last = keys.last().cloned();
        keys.push(FracKey::between(last.as_ref(), None));
    }
    let ids = set.get(k).unwrap().mint_n(n);
    let ops = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| {
            let pos = if distinct >= n {
                keys[i].clone()
            } else {
                keys[rng.next(distinct)].clone()
            };
            let created = if reversed {
                (n - i) as i64
            } else {
                rng.next(50) as i64
            };
            let title = format!("t{}", rng.next(40));
            let mut c = ph2d_kanban_model::Card::new(CardId(id), l, pos, &title, Moment(created));
            c.due = (rng.next(3) > 0).then(|| due(rng.next(9) as i64)).flatten();
            KanbanOp::put(Record::card(c))
        })
        .collect();
    set.apply(k, ops);
    (set, k, l)
}

/// ⭐ O mesmo `keep` da versão de antes, linha a linha — com empates de posição (os cartões criados
/// no mesmo sítio ao mesmo tempo), em listas pequenas e médias, pelos quatro critérios.
#[test]
fn the_fast_sort_keeps_exactly_the_rows_the_quadratic_one_kept() {
    let mut rng = Lcg(0x5027);
    for round in 0..400 {
        let n = rng.next(80);
        let distinct = 1 + rng.next(n + 1);
        let (set, k, l) = big(n, distinct, round, false);
        let kd = set.get(k).unwrap();
        let rows: Vec<(FracKey, &ph2d_kanban_model::Card)> = set
            .entries_in(k, l)
            .into_iter()
            .map(|e| {
                (
                    kd.entry_pos(e).unwrap().clone(),
                    set.card(e.shows(k)).unwrap(),
                )
            })
            .collect();
        for by in [SortBy::Newest, SortBy::Oldest, SortBy::Name, SortBy::Due] {
            let mut want: Vec<usize> = (0..rows.len()).collect();
            want.sort_by(|&i, &j| cmp(by, rows[i].1, rows[j].1));
            let mut rank = vec![0; rows.len()];
            for (r, &i) in want.iter().enumerate() {
                rank[i] = r;
            }
            let pos: Vec<&FracKey> = rows.iter().map(|r| &r.0).collect();
            assert_eq!(
                longest_kept(&pos, &rank),
                longest_kept_quadratic(&pos, &rank),
                "n={n} distinct={distinct} {by:?}"
            );
            let mut sorted = set.clone();
            sorted.apply(k, sort_list(&set, k, l, by));
            let got: Vec<usize> = sorted
                .entries_in(k, l)
                .into_iter()
                .map(|e| rows.iter().position(|r| r.1.id == e.shows(k).card).unwrap())
                .collect();
            let order: Vec<_> = got.iter().map(|&i| rows[i].1).collect();
            assert!(
                order
                    .windows(2)
                    .all(|w| cmp(by, w[0], w[1]) != Ordering::Greater),
                "n={n} {by:?}: a lista não ficou ordenada"
            );
        }
    }
}

/// ⭐ R1 — o pior caso (lista ao contrário: só um cartão fica parado) cresce como n·log n, não n²: ×8
/// cartões custam bem menos que ×64. E as chaves novas ficam CURTAS (bissecção, não uma fila de
/// «logo acima do anterior» que cresce um carácter a cada ~6).
#[test]
fn sorting_a_reversed_list_grows_n_log_n_not_n_squared() {
    let time = |n: usize| {
        let (set, k, l) = big(n, n, 1, true);
        let mut best = std::time::Duration::MAX;
        let mut longest = 0;
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let keys = sort_keys(&set, k, l, SortBy::Oldest);
            best = best.min(t.elapsed());
            assert_eq!(keys.len(), n - 1);
            longest = keys
                .iter()
                .map(|(_, p)| p.as_str().len())
                .max()
                .unwrap_or(0);
        }
        (best, longest)
    };
    let (small, _) = time(2_000);
    let (large, longest) = time(16_000);
    let ratio = large.as_secs_f64() / small.as_secs_f64().max(1e-9);
    assert!(
        ratio < 24.0,
        "×8 cartões custaram ×{ratio:.1} ({small:?} → {large:?})"
    );
    assert!(longest <= 12, "chave nova com {longest} caracteres");
}

/// O `sort_keys` de antes, verbatim (O(n²) no `keep` e na busca do vizinho de cima; chaves em fila).
fn sort_keys_quadratic(
    set: &KanbanSet,
    kanban: KanbanId,
    list: ListId,
    by: SortBy,
) -> Vec<(Entry, FracKey)> {
    let k = set.get(kanban).unwrap();
    let rows: Vec<(Entry, FracKey, &ph2d_kanban_model::Card)> = set
        .entries_in(kanban, list)
        .into_iter()
        .filter_map(|e| Some((e, k.entry_pos(e)?.clone(), set.card(e.shows(kanban))?)))
        .collect();
    let mut want: Vec<usize> = (0..rows.len()).collect();
    want.sort_by(|&i, &j| cmp(by, rows[i].2, rows[j].2));
    let mut rank = vec![0; rows.len()];
    for (r, &i) in want.iter().enumerate() {
        rank[i] = r;
    }
    let keep = longest_kept_quadratic(&rows.iter().map(|r| &r.1).collect::<Vec<_>>(), &rank);
    let mut out = Vec::new();
    let mut lo: Option<FracKey> = None;
    for (r, &i) in want.iter().enumerate() {
        if keep[i] {
            lo = Some(rows[i].1.clone());
            continue;
        }
        let hi = want[r + 1..]
            .iter()
            .find(|&&j| keep[j])
            .map(|&j| &rows[j].1);
        let pos = FracKey::between(lo.as_ref(), hi);
        lo = Some(pos.clone());
        out.push((rows[i].0, pos));
    }
    out
}

/// A medição de `MEDICOES.md`: 20 000 cartões (ao contrário e ao acaso), a versão de antes e a de
/// agora no MESMO processo, intercaladas, o mínimo de 3. `cargo test --profile smoke … -- --ignored`.
#[test]
#[ignore = "medição"]
fn measure_sort_twenty_thousand_cards() {
    for (name, reversed, by) in [
        ("ao contrário", true, SortBy::Oldest),
        ("ao acaso", false, SortBy::Name),
    ] {
        let (set, k, l) = big(20_000, 20_000, 1, reversed);
        let (mut old, mut new) = (f64::MAX, f64::MAX);
        let (mut old_len, mut new_len) = (0, 0);
        for _ in 0..3 {
            let t = std::time::Instant::now();
            let o = std::hint::black_box(sort_keys_quadratic(&set, k, l, by));
            old = old.min(t.elapsed().as_secs_f64());
            let t = std::time::Instant::now();
            let n = std::hint::black_box(sort_keys(&set, k, l, by));
            new = new.min(t.elapsed().as_secs_f64());
            assert_eq!(o.len(), n.len(), "as mesmas linhas mudam");
            old_len = o.iter().map(|x| x.1.as_str().len()).max().unwrap_or(0);
            new_len = n.iter().map(|x| x.1.as_str().len()).max().unwrap_or(0);
        }
        println!(
            "20000 {name}: antes {old:.4} s (chave até {old_len}) · agora {new:.4} s (chave até {new_len})"
        );
    }
}
