//! **Agrupar notas** (W7, o *Cluster* do Miro — `docs/MiroClone/05_facilitacao.md` §2 C1–C9): por
//! cor, por etiqueta ou por palavra; cada grupo é um FRAME (o «container» do Miro), as notas numa
//! grelha de ⌈√n⌉ colunas com as folgas MEDIDAS nas imagens oficiais (§1.2), os frames numa fila.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_board_model::{BoardDoc, BoardOp, Element, ElementId, FracKey, History};
use ph2d_board_words::{Keyword, detect, keywords};

use crate::{Editor, Notice};

/// C3 — as folgas entre notas `[horizontal, vertical]` em fracção do lado (médias de `cluster_01` e
/// `cluster_02`: 0,155/0,18 e 0,275/0,27 — a vertical leva a sombra da nota).
pub const CLUSTER_GAP: [f64; 2] = [0.17, 0.27];
/// C3 — a margem do frame `[lados, cima, baixo]` em fracção do lado (0,31/0,30 · 0,33/0,32 ·
/// 0,44/0,42).
pub const CLUSTER_PAD: [f64; 3] = [0.30, 0.32, 0.43];
/// C3 — o vão entre frames em fracção do lado (0,41 e 0,38).
pub const CLUSTER_BETWEEN: f64 = 0.40;
/// C8 — quantas palavras leva o título de um grupo por palavra (`cluster_02`: três).
pub const CLUSTER_TITLE_WORDS: usize = 3;

/// Por onde se agrupa (C1; o «Author» e o «Sentiment» do Miro ficam de fora, estudo §3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterBy {
    Color,
    Tag,
    Keyword,
}

/// Os títulos que vêm da interface (já traduzidos): o grupo das notas sem etiqueta e o do resto das
/// palavras.
#[derive(Clone, Debug)]
pub struct ClusterNames {
    pub no_tag: String,
    pub uncategorized: String,
}

/// Um grupo: o título e as notas (índices na ordem de leitura; `true` = vai uma CÓPIA, C6).
struct Group {
    title: String,
    notes: Vec<(usize, bool)>,
}

impl Editor {
    /// ⭐ Agrupa as notas seleccionadas (e as dos frames seleccionados) por `by`: UM passo, os frames
    /// novos ficam seleccionados. `false` = menos de duas notas (o aviso).
    pub fn cluster(
        &mut self,
        doc: &mut BoardDoc,
        history: &mut History,
        by: ClusterBy,
        names: &ClusterNames,
    ) -> bool {
        let ids: Vec<ElementId> = self.selection.iter().copied().collect();
        let picked: Vec<Element> = crate::organize::with_descendants(doc, &ids)
            .into_iter()
            .filter_map(|id| doc.get(id))
            .filter(|e| e.shape().is_some_and(|s| s.kind.is_note()))
            .filter(|e| !e.locked() && !e.is_hidden())
            .cloned()
            .collect();
        if picked.len() < 2 {
            self.notice = Some(Notice::NothingToCluster);
            return false;
        }
        let notes = crate::notes_layout::reading_order(&picked);
        let mut groups = match by {
            ClusterBy::Color => by_color(&notes),
            ClusterBy::Tag => by_tag(&notes, &names.no_tag),
            ClusterBy::Keyword => by_keyword(&notes, &names.uncategorized),
        };
        // C4 — a ordem dos grupos: a primeira aparição na ordem de leitura.
        groups.sort_by_key(|g| g.notes.iter().map(|(i, _)| *i).min());
        let (ops, frames) = self.lay_out(doc, &notes, &groups);
        history.apply(doc, ops);
        self.selection = frames;
        true
    }

    /// C2/C3 — os frames e as notas no sítio: o lote e os ids dos frames.
    fn lay_out(
        &self,
        doc: &mut BoardDoc,
        notes: &[Element],
        groups: &[Group],
    ) -> (Vec<BoardOp>, BTreeSet<ElementId>) {
        let s = notes
            .iter()
            .map(ph2d_board_model::mark_side)
            .fold(0.0, f64::max);
        let [gx, gy] = CLUSTER_GAP.map(|f| f * s);
        let [px, pt, pb] = CLUSTER_PAD.map(|f| f * s);
        let x0 = notes
            .iter()
            .map(|e| e.aabb()[0])
            .fold(f64::INFINITY, f64::min);
        let y0 = notes
            .iter()
            .map(|e| e.aabb()[1])
            .fold(f64::INFINITY, f64::min);
        // Todas na mesma casa ⇒ os frames novos moram nela.
        let home = notes[0]
            .frame
            .filter(|f| notes.iter().all(|e| e.frame == Some(*f)));
        let mut z = doc.z_on_top();
        let mut next_z = || {
            let k = z.clone();
            z = FracKey::between(Some(&k), None);
            k
        };
        let (mut ops, mut frames, mut x) = (Vec::new(), BTreeSet::new(), x0);
        for g in groups {
            let n = g.notes.len();
            let cols = (n as f64).sqrt().ceil() as usize;
            let col_w = g.notes.iter().map(|(i, _)| notes[*i].w).fold(0.0, f64::max);
            let rows: Vec<f64> = g
                .notes
                .chunks(cols)
                .map(|r| r.iter().map(|(i, _)| notes[*i].h).fold(0.0, f64::max))
                .collect();
            let w = 2.0 * px + cols as f64 * col_w + (cols - 1) as f64 * gx;
            let h = pt + rows.iter().sum::<f64>() + (rows.len() - 1) as f64 * gy + pb;
            let mut frame = self.new_frame(doc, [x, y0, w, h]);
            if let Some(f) = frame.as_frame_mut() {
                f.name.clone_from(&g.title);
            }
            frame.frame = home;
            frame.z = next_z();
            frames.insert(frame.id);
            let fid = frame.id;
            ops.push(BoardOp::Put(frame));
            let mut y = y0 + pt;
            for (r, row) in g.notes.chunks(cols).enumerate() {
                for (c, (i, copy)) in row.iter().enumerate() {
                    let mut el = notes[*i].clone();
                    if *copy {
                        el.id = doc.mint_id();
                        el.version = 0;
                        el.z = next_z();
                        el.set_votes(0);
                    }
                    let to = [x + px + c as f64 * (col_w + gx), y];
                    el.translate([to[0] - el.x, to[1] - el.y]);
                    el.frame = Some(fid);
                    // Um grupo não atravessa frames (o Miro, W5): a nota sai dos grupos dela.
                    el.groups_mut().clear();
                    ops.push(BoardOp::Put(el));
                }
                y += rows[r] + gy;
            }
            x += w + CLUSTER_BETWEEN * s;
        }
        (ops, frames)
    }
}

/// C5 — um grupo por cor, sem título.
fn by_color(notes: &[Element]) -> Vec<Group> {
    let mut map: BTreeMap<[u8; 4], Vec<(usize, bool)>> = BTreeMap::new();
    for (i, el) in notes.iter().enumerate() {
        let c = el.style().fill.map_or([0; 4], |c| c.0);
        map.entry(c).or_default().push((i, false));
    }
    map.into_values()
        .map(|notes| Group {
            title: String::new(),
            notes,
        })
        .collect()
}

/// C6 — um grupo por etiqueta (o nome dela), a nota com várias DUPLICADA nas outras; as sem
/// etiqueta no grupo `no_tag`.
fn by_tag(notes: &[Element], no_tag: &str) -> Vec<Group> {
    let mut order: Vec<String> = Vec::new();
    let mut map: BTreeMap<String, Vec<(usize, bool)>> = BTreeMap::new();
    let mut none = Vec::new();
    for (i, el) in notes.iter().enumerate() {
        if el.tags().is_empty() {
            none.push((i, false));
        }
        for (k, t) in el.tags().iter().enumerate() {
            if !map.contains_key(&t.name) {
                order.push(t.name.clone());
            }
            map.entry(t.name.clone()).or_default().push((i, k > 0));
        }
    }
    let mut out: Vec<Group> = order
        .into_iter()
        .map(|title| {
            let notes = map.remove(&title).unwrap_or_default();
            Group { title, notes }
        })
        .collect();
    if !none.is_empty() {
        out.push(Group {
            title: no_tag.to_owned(),
            notes: none,
        });
    }
    out
}

/// C7/C8 — por palavra: a raiz partilhada pelo maior número de notas soltas (≥ 2; empate pela
/// primeira aparição) leva-as todas; repete; o resto em `rest`.
fn by_keyword(notes: &[Element], rest: &str) -> Vec<Group> {
    let texts: Vec<&str> = notes.iter().map(note_text).collect();
    let lang = detect(&texts);
    let words: Vec<Vec<Keyword>> = texts.iter().map(|t| keywords(t, lang)).collect();
    let keys: Vec<BTreeSet<&str>> = words
        .iter()
        .map(|w| w.iter().map(|k| k.key.as_str()).collect())
        .collect();
    // A primeira aparição de cada raiz: (nota, posição na nota).
    let mut first: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for (i, w) in words.iter().enumerate() {
        for (j, k) in w.iter().enumerate() {
            first.entry(k.key.as_str()).or_insert((i, j));
        }
    }
    let mut loose: BTreeSet<usize> = (0..notes.len()).collect();
    let mut out = Vec::new();
    loop {
        let mut count: BTreeMap<&str, usize> = BTreeMap::new();
        for i in &loose {
            for k in &keys[*i] {
                *count.entry(k).or_default() += 1;
            }
        }
        let best = count
            .into_iter()
            .filter(|(_, n)| *n >= 2)
            .min_by_key(|(k, n)| (std::cmp::Reverse(*n), first[k]));
        let Some((key, _)) = best else {
            break;
        };
        let members: Vec<usize> = loose
            .iter()
            .copied()
            .filter(|i| keys[*i].contains(key))
            .collect();
        for i in &members {
            loose.remove(i);
        }
        out.push(Group {
            title: keyword_title(&members, &words, &first),
            notes: members.into_iter().map(|i| (i, false)).collect(),
        });
    }
    if !loose.is_empty() {
        out.push(Group {
            title: rest.to_owned(),
            notes: loose.into_iter().map(|i| (i, false)).collect(),
        });
    }
    out
}

/// C8 — as [`CLUSTER_TITLE_WORDS`] raízes mais frequentes nas notas do grupo (empate pela primeira
/// aparição), cada uma na forma escrita mais frequente, com `#`.
fn keyword_title(
    members: &[usize],
    words: &[Vec<Keyword>],
    first: &BTreeMap<&str, (usize, usize)>,
) -> String {
    let mut notes_with: BTreeMap<&str, usize> = BTreeMap::new();
    let mut surfaces: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for i in members {
        let mut seen = BTreeSet::new();
        for k in &words[*i] {
            if seen.insert(k.key.as_str()) {
                *notes_with.entry(k.key.as_str()).or_default() += 1;
            }
            surfaces.entry(k.key.as_str()).or_default().push(&k.surface);
        }
    }
    let mut ranked: Vec<(&str, usize)> = notes_with.into_iter().collect();
    ranked.sort_by_key(|(k, n)| (std::cmp::Reverse(*n), first[k]));
    ranked
        .into_iter()
        .take(CLUSTER_TITLE_WORDS)
        .map(|(k, _)| format!("#{}", most_frequent(&surfaces[k])))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A forma mais frequente (empate: a que apareceu primeiro).
fn most_frequent<'a>(forms: &[&'a str]) -> &'a str {
    let mut best = (0, forms[0]);
    for (i, f) in forms.iter().enumerate() {
        let n = forms.iter().filter(|g| *g == f).count();
        if n > best.0 && !forms[..i].contains(f) {
            best = (n, f);
        }
    }
    best.1
}

fn note_text(el: &Element) -> &str {
    el.shape().map_or("", |s| s.text.as_str())
}
