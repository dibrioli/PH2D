//! Agrupar notas (W7, C1–C9) — `docs/MiroClone/05_facilitacao.md` §2.
//!
//! Números do Miro transcritos (§1.2, `cluster_01` com notas de 142 px e `cluster_02` de 126): 5 notas →
//! 3 colunas, 4 → 2, 3 → 2, 2 → 2, 1 → 1 (⌈√n⌉); folga horizontal 22/142 e 23/126, passo vertical
//! 181/142 e 160/126; margem 44/142 e 38/126 dos lados, 47/142 e 40/126 em cima, 62/142 e 53/126 em
//! baixo; entre containers 58/142 e 48/126; a ordem «Q1 · no tag · Q2» (a primeira aparição).

use super::*;
use ph2d_board_model::Stamp;

const S: f64 = 199.0;

fn names() -> ClusterNames {
    ClusterNames {
        no_tag: "no tag".into(),
        uncategorized: "#uncategorized".into(),
    }
}

impl World {
    fn cluster(&mut self, ids: &[ElementId], by: ClusterBy) -> bool {
        self.ed.select(&self.doc, ids.iter().copied());
        self.ed.cluster(&mut self.doc, &mut self.h, by, &names())
    }

    /// Os frames novos (a selecção) da esquerda para a direita: `(nome, notas dentro, caixa)`.
    fn clusters(&self) -> Vec<(String, Vec<ElementId>, [f64; 4])> {
        let mut v: Vec<(String, Vec<ElementId>, [f64; 4])> = self
            .ed
            .selection()
            .iter()
            .map(|f| {
                let el = self.el(*f);
                let mut kids: Vec<&Element> =
                    self.doc.descendants(*f).iter().map(|id| self.el(*id)).collect();
                kids.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.x.total_cmp(&b.x)));
                (
                    el.as_frame().unwrap().name.clone(),
                    kids.iter().map(|e| e.id).collect(),
                    [el.x, el.y, el.w, el.h],
                )
            })
            .collect();
        v.sort_by(|a, b| a.2[0].total_cmp(&b.2[0]));
        v
    }

    fn texts(&self, ids: &[ElementId]) -> Vec<String> {
        ids.iter()
            .map(|id| self.el(*id).shape().unwrap().text.as_str().to_owned())
            .collect()
    }
}

/// C1: agrupar precisa de duas notas (as formas não contam).
#[test]
fn cluster_needs_two_notes() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let r = w.rect([300.0, 0.0, 100.0, 100.0]);
    assert!(!w.cluster(&[a, r], ClusterBy::Color));
    assert_eq!(w.ed.take_notice(), Some(Notice::NothingToCluster));
    assert_eq!(w.doc.live_len(), 2);
}

/// ⭐ C2/C3/C5: por cor, um frame por cor SEM título, numa fila a partir do canto da selecção; dentro,
/// ⌈√n⌉ colunas com as folgas e margens medidas; frames a 0,40 do lado; UM passo.
#[test]
fn the_cluster_grid_is_the_measured_one() {
    let mut w = world();
    let mut yellow = Vec::new();
    for i in 0..5 {
        yellow.push(w.sticky([1000.0 + 250.0 * f64::from(i), 700.0], "y", 2, &[]));
    }
    let green = w.sticky([100.0, 50.0], "g", 4, &[]);
    let mut all = yellow.clone();
    all.push(green);
    assert!(w.cluster(&all, ClusterBy::Color));
    let c = w.clusters();
    assert_eq!(c.len(), 2);
    assert!(c.iter().all(|(n, _, _)| n.is_empty()), "C5: por cor não há título");
    // A verde aparece primeiro na leitura (está mais acima) ⇒ o 1.º frame, no canto da selecção.
    let (_, gk, g) = &c[0];
    assert_eq!(*gk, [green]);
    assert_eq!([g[0], g[1]], [100.0, 50.0], "C2: o canto superior esquerdo da selecção");
    let [px, pt, pb] = CLUSTER_PAD.map(|f| f * S);
    assert!(close(g[2], 2.0 * px + S) && close(g[3], pt + S + pb));
    let (_, yk, y) = &c[1];
    assert!(close(y[0], g[0] + g[2] + CLUSTER_BETWEEN * S), "o vão entre frames");
    assert!(close(y[1], g[1]), "alinhados em cima");
    let [gx, gy] = CLUSTER_GAP.map(|f| f * S);
    assert!(close(y[2], 2.0 * px + 3.0 * S + 2.0 * gx), "5 notas → 3 colunas");
    assert!(close(y[3], pt + 2.0 * S + gy + pb), "… e 2 filas");
    let first = w.el(yk[0]);
    assert!(close(first.x, y[0] + px) && close(first.y, y[1] + pt));
    let fourth = w.el(yk[3]);
    assert!(close(fourth.y, y[1] + pt + S + gy) && close(fourth.x, first.x));
    assert!(yk.iter().all(|id| w.el(*id).frame == Some(w.ed.selection().iter().copied().find(|f| w.doc.descendants(*f).contains(id)).unwrap())));
    assert!(w.cmd(Command::Undo));
    assert_eq!(w.doc.live_len(), 6, "UM passo: os frames saem e as notas voltam");
    assert!(close(w.el(green).x, 100.0));
}

/// ⭐ C4: a ordem dos grupos e das notas é a da LEITURA (de cima, da esquerda) — nem alfabética, nem
/// por tamanho, nem «o resto no fim» (o Miro: «Q1 · no tag · Q2»).
#[test]
fn clusters_keep_reading_order() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "Idea 4", 1, &["Q1"]);
    let b = w.sticky([300.0, 0.0], "Idea 3", 4, &[]);
    let c = w.sticky([600.0, 0.0], "Idea 1", 1, &["Q2"]);
    let d = w.sticky([0.0, 300.0], "Idea 2", 1, &["Q1"]);
    let e = w.sticky([300.0, 300.0], "Idea 5", 1, &["Q2"]);
    assert!(w.cluster(&[e, d, c, b, a], ClusterBy::Tag));
    let titles: Vec<String> = w.clusters().into_iter().map(|c| c.0).collect();
    assert_eq!(titles, ["Q1", "no tag", "Q2"]);
    assert_eq!(w.clusters()[0].1, [a, d]);
}

/// ⭐ C6: uma nota com duas etiquetas vai para os dois grupos (no 2.º, uma CÓPIA, com os carimbos e sem
/// votos); as sem etiqueta no «no tag».
#[test]
fn a_note_with_two_tags_is_in_two_clusters() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "both", 1, &["Q1", "Q2"]);
    let mut el = w.el(a).clone();
    el.set_votes(3);
    el.add_stamp(Stamp::Heart, 1);
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    let b = w.sticky([300.0, 0.0], "none", 1, &[]);
    assert!(w.cluster(&[a, b], ClusterBy::Tag));
    let c = w.clusters();
    assert_eq!(c.iter().map(|c| c.0.as_str()).collect::<Vec<_>>(), ["Q1", "Q2", "no tag"]);
    assert_eq!(c[0].1, [a]);
    let copy = c[1].1[0];
    assert_ne!(copy, a);
    assert_eq!(w.texts(&[copy]), ["both"]);
    assert_eq!(w.el(copy).votes(), 0);
    assert_eq!(w.el(copy).stamps().len(), 1);
    assert_eq!(c[2].1, [b]);
    assert_eq!(w.doc.live_len(), 3 + 3);
}

/// ⭐ C7: por palavra — as notas que partilham uma raiz juntam-se (a raiz partilhada pelo maior número
/// primeiro); maiúsculas, plurais, o apóstrofo curvo e os acentos não separam; o resto em
/// «#uncategorized». Entradas: as do oráculo (`facilitacao_oracle/entradas/notas_w7.json`).
#[test]
fn notes_sharing_a_root_cluster_together() {
    let mut w = world();
    let texts = [
        "Faster BUILDS please",
        "Build times are too slow",
        "Slow builds block the Team",
        "More pairing sessions",
        "Pair programming helped a lot",
        "Team lunch on Fridays",
        "The team’s morale is HIGH",
        "Café meetings & CAFE coffee",
    ];
    let ids: Vec<ElementId> = texts
        .iter()
        .enumerate()
        .map(|(i, t)| w.sticky([250.0 * i as f64, 0.0], t, 1, &[]))
        .collect();
    assert!(w.cluster(&ids, ClusterBy::Keyword));
    let c = w.clusters();
    let groups: Vec<Vec<String>> = c.iter().map(|g| w.texts(&g.1)).collect();
    assert_eq!(groups[0], [texts[0], texts[1], texts[2]], "build: três notas, a maior");
    assert_eq!(groups[1], [texts[3], texts[4]], "pair (pairing/Pair)");
    assert_eq!(groups[2], [texts[5], texts[6]], "team (Team/team’s) — as soltas que sobram");
    assert_eq!(groups[3], [texts[7]], "o resto");
    assert_eq!(c[3].0, "#uncategorized");
}

/// ⭐ C8: o título de um grupo por palavra são as TRÊS raízes mais frequentes nele, cada uma na forma
/// escrita mais frequente, com `#`.
#[test]
fn a_keyword_title_has_three_words() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "Slow builds hurt", 1, &[]);
    let b = w.sticky([250.0, 0.0], "slow builds again", 1, &[]);
    let c = w.sticky([500.0, 0.0], "Builds slow, hurt", 1, &[]);
    assert!(w.cluster(&[a, b, c], ClusterBy::Keyword));
    assert_eq!(w.clusters()[0].0, "#slow #builds #hurt");
}

/// ⭐ C9: uma nota travada fica onde está (e fora dos grupos).
#[test]
fn a_locked_note_is_left_alone() {
    let mut w = world();
    let a = w.sticky([0.0, 0.0], "a", 1, &[]);
    let b = w.sticky([300.0, 0.0], "b", 1, &[]);
    let l = w.sticky([600.0, 0.0], "l", 1, &[]);
    let mut el = w.el(l).clone();
    el.set_locked(true);
    w.h.apply(&mut w.doc, vec![BoardOp::Put(el)]);
    assert!(w.cluster(&[a, b, l], ClusterBy::Color));
    assert_eq!(w.clusters()[0].1, [a, b]);
    assert!(close(w.el(l).x, 600.0) && w.el(l).frame.is_none());
}
