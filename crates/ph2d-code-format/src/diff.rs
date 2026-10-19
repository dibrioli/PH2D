//! The minimal edits from a text to its formatted version: a line diff, then a character diff inside
//! each changed hunk — so a cursor stays next to the same characters (only the spaces around it move),
//! like any other edit of the buffer.

use similar::{Algorithm, ChangeTag, DiffTag, TextDiff};
use std::ops::Range;

/// Replace `range` (bytes of the OLD text) with `insert`. Sorted, never overlapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub insert: String,
}

/// The edits that turn `old` into `new` (empty = the same text).
#[must_use]
pub fn changes(old: &str, new: &str) -> Vec<Edit> {
    let mut out = Vec::new();
    if old == new {
        return out;
    }
    // Lines compared WITHOUT their whitespace: a formatter mostly moves spaces, so a re-indented line
    // pairs with its old self and only the pair is refined (all 7 500 lines re-indented: 265 → 3.8 ms,
    // medicoes_onda2/I13.md).
    let (ol, nl): (Vec<&str>, Vec<&str>) = (
        old.split_inclusive('\n').collect(),
        new.split_inclusive('\n').collect(),
    );
    let key = |l: &&str| l.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let (ok, nk): (Vec<String>, Vec<String>) =
        (ol.iter().map(key).collect(), nl.iter().map(key).collect());
    let olds = starts(&ol);
    let news = starts(&nl);
    for op in similar::capture_diff_slices(Algorithm::Myers, &ok, &nk) {
        let (o, n) = (op.old_range(), op.new_range());
        let ob = olds[o.start]..olds[o.end];
        let nb = news[n.start]..news[n.end];
        if op.tag() == DiffTag::Equal && old[ob.clone()] == new[nb.clone()] {
            continue;
        }
        if o.len() == n.len() {
            // Line for line (a re-indent, quotes): each pair on its own, linear in the hunk.
            for (i, j) in o.zip(n) {
                refine(
                    &old[olds[i]..olds[i + 1]],
                    &new[news[j]..news[j + 1]],
                    olds[i],
                    &mut out,
                );
            }
        } else {
            refine(&old[ob.clone()], &new[nb], ob.start, &mut out);
        }
    }
    out
}

/// The byte where each token starts, plus the end.
fn starts(slices: &[&str]) -> Vec<usize> {
    let mut v = Vec::with_capacity(slices.len() + 1);
    let mut at = 0;
    v.push(0);
    for s in slices {
        at += s.len();
        v.push(at);
    }
    v
}

/// The character diff of one hunk, as edits at `base` + the byte in `old`. The common head and tail
/// are cut first (a re-indent is then one edit, without the diff).
fn refine(old: &str, new: &str, base: usize, out: &mut Vec<Edit>) {
    let head: usize = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    let (o, n) = (&old[head..], &new[head..]);
    let tail: usize = o
        .chars()
        .rev()
        .zip(n.chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum();
    let (o, n) = (&o[..o.len() - tail], &n[..n.len() - tail]);
    let base = base + head;
    if o.is_empty() || n.is_empty() {
        if !(o.is_empty() && n.is_empty()) {
            out.push(Edit {
                range: base..base + o.len(),
                insert: n.to_owned(),
            });
        }
        return;
    }
    let chars = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .diff_chars(o, n);
    let mut at = base;
    let mut open: Option<Edit> = None;
    for c in chars.iter_all_changes() {
        let v = c.value();
        match c.tag() {
            ChangeTag::Equal => {
                out.extend(open.take());
                at += v.len();
            }
            ChangeTag::Delete => {
                let e = open.get_or_insert(Edit {
                    range: at..at,
                    insert: String::new(),
                });
                e.range.end = at + v.len();
                at += v.len();
            }
            ChangeTag::Insert => open
                .get_or_insert(Edit {
                    range: at..at,
                    insert: String::new(),
                })
                .insert
                .push_str(v),
        }
    }
    out.extend(open);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(old: &str, edits: &[Edit]) -> String {
        let mut s = old.to_owned();
        for e in edits.iter().rev() {
            s.replace_range(e.range.clone(), &e.insert);
        }
        s
    }

    #[test]
    fn as_edicoes_levam_ao_texto_formatado_e_sao_minimas() {
        let casos = [
            ("local x = 'a'\n", "local x = \"a\"\n"),
            ("if a then return end\n", "if a then\n  return\nend\n"),
            (
                "ph2d.property(\"a\", 1)   -- x\n    local y=1\n",
                "ph2d.property(\"a\", 1) -- x\nlocal y = 1\n",
            ),
            ("a\nb\nc", "a\nB\nc\nd\n"),
            ("", "x\n"),
            ("x\n", ""),
            ("é😀 'ç'\n", "é😀 \"ç\"\n"),
        ];
        for (old, new) in casos {
            let e = changes(old, new);
            assert_eq!(apply(old, &e), new, "{old:?}");
            for w in e.windows(2) {
                assert!(w[0].range.end <= w[1].range.start, "{e:?}");
            }
            for x in &e {
                assert!(old.is_char_boundary(x.range.start) && old.is_char_boundary(x.range.end));
            }
        }
        // As aspas: só as duas aspas mudam, o miolo da string fica.
        assert_eq!(
            changes("local x = 'abc'\n", "local x = \"abc\"\n"),
            vec![
                Edit {
                    range: 10..11,
                    insert: "\"".into()
                },
                Edit {
                    range: 14..15,
                    insert: "\"".into()
                },
            ]
        );
        assert!(changes("igual\n", "igual\n").is_empty());
    }
}
