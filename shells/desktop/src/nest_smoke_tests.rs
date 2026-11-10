//! Gates das cenas `PH2D_NEST_SMOKE=3..5`: o que o `eprintln` afirma, medido sobre as MESMAS
//! [`super::build_library`] e [`super::build_montagem`] que o app chama (não um espelho).

use super::{LIBRARY_END, build_library, build_montagem};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, container_bar_seconds};

/// Um doc montado como a cena `=3` monta.
fn library() -> TimelineDoc {
    let mut doc = TimelineDoc::default();
    build_library(&mut doc, 1);
    doc
}

/// Quantas strips de `source` existem na pilha de `host`.
fn count_of(doc: &TimelineDoc, host: StackHost, source: StripSource) -> usize {
    doc.host_stack(host)
        .into_iter()
        .flatten()
        .flat_map(|l| l.strips.iter())
        .filter(|s| s.source == source)
        .count()
}

/// A lista tem TRÊS barras; a do Pause (VAZIO) mede [`ph2d_timeline::EMPTY_CONTAINER_SECONDS`]
/// pela porta única, o que dá à instância dele uma janela real.
#[test]
fn the_library_holds_three_containers_sized_one_three_and_two() {
    let doc = library();
    let names: Vec<&str> = doc.containers().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        ["Step", "Walk", "Pause"],
        "a lista da aba Containers"
    );

    let bar = |ix: usize| {
        let interior = doc
            .container_stack(ix)
            .into_iter()
            .flatten()
            .flat_map(|l| l.strips.iter())
            .fold(0.0_f64, |acc, s| acc.max(s.t_end));
        container_bar_seconds(interior)
    };
    assert!((bar(0) - 1.0).abs() < 1e-9, "Step mede 1 s: {}", bar(0));
    assert!((bar(1) - 3.0).abs() < 1e-9, "Walk mede 3 s: {}", bar(1));
    assert!(
        (bar(2) - 2.0).abs() < 1e-9,
        "o Pause e' VAZIO e mede 2 s pela porta unica: {}",
        bar(2)
    );
}

/// O aninhamento é real: o Walk contém o Step, três vezes, e é isso que dá a trilha de três
/// níveis (`[ Scene ][ Walk ][ Step ]`) que a cena manda olhar.
#[test]
fn the_walk_contains_the_step_three_times() {
    let doc = library();
    let step = StripSource::Container(0);
    assert_eq!(
        count_of(&doc, StackHost::Container(1), step),
        3,
        "tres instancias do Step dentro do Walk"
    );
    assert_eq!(
        doc.container_stack(1).map(<[_]>::len),
        Some(2),
        "duas lanes no Walk (Steps + Glide) — os canais esparsos que somam"
    );
}

/// A cena põe os três assets nas janelas anunciadas e **em velocidade 1** (`add_strip_to` deriva
/// `speed = slice/span`: uma janela torta viraria câmera lenta silenciosa).
#[test]
fn the_scene_places_each_asset_at_its_own_rate() {
    let doc = library();
    let lanes = doc.host_stack(StackHost::Document).expect("a cena");
    let strips: Vec<_> = lanes.iter().flat_map(|l| l.strips.iter()).collect();
    assert_eq!(strips.len(), 3, "Walk + Pause + Step soltos na cena");

    let windows: Vec<(f64, f64)> = strips.iter().map(|s| (s.t_start, s.t_end)).collect();
    assert_eq!(windows, [(0.0, 3.0), (3.0, 5.0), (5.0, LIBRARY_END)]);

    for s in &strips {
        assert!(
            (s.speed - 1.0).abs() < 1e-9,
            "instancia em {:?} devia tocar em velocidade 1, esta em {}",
            (s.t_start, s.t_end),
            s.speed
        );
    }
}

/// **A CASCATA**: apagar o Step mata as instâncias nos DOIS lugares e re-indexa o resto — a do
/// Walk (1 → 0) continua a apontar para o WALK, não para o vizinho que escorregou para o slot.
#[test]
fn deleting_the_leaf_asset_cascades_and_renumbers() {
    let mut doc = library();
    assert_eq!(
        count_of(&doc, StackHost::Document, StripSource::Container(0)),
        1
    );

    assert!(doc.remove_container(0), "a lixeira do Step");

    let names: Vec<&str> = doc.containers().iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Walk", "Pause"], "o Step saiu da lista");
    assert_eq!(
        count_of(&doc, StackHost::Container(0), StripSource::Container(0)),
        0,
        "as tres instancias de dentro do Walk morreram junto"
    );

    let scene: Vec<StripSource> = doc
        .host_stack(StackHost::Document)
        .expect("a cena")
        .iter()
        .flat_map(|l| l.strips.iter())
        .map(|s| s.source)
        .collect();
    assert_eq!(
        scene,
        [StripSource::Container(0), StripSource::Container(1)],
        "sobram Walk e Pause, re-indexados 1->0 e 2->1"
    );
    assert_eq!(
        doc.containers()[0].name,
        "Walk",
        "e o indice 0 e' o WALK — a instancia nao adotou o vizinho que escorregou"
    );
}

/// Os dois clips keyam canais **disjuntos**: as lanes do Walk somam (com X no Step, as três
/// repetições voltariam ao mesmo lugar e a cena leria como quebrada).
#[test]
fn the_two_clips_key_disjoint_channels() {
    use ph2d_timeline::PropKind;
    let doc = library();
    // O `target` de cada (objeto, propriedade): a tabela de bindings que o `upsert_key` usou.
    let target_of = |prop: PropKind| {
        doc.bindings()
            .iter()
            .find(|b| b.prop == prop)
            .map(|b| b.target)
            .expect("binding")
    };
    let keyed = |clip: usize, prop: PropKind| {
        doc.clips()[clip]
            .clip
            .track(target_of(prop))
            .is_some_and(|t| !t.keys().is_empty())
    };
    assert!(keyed(0, PropKind::TranslationY), "Step keya Y");
    assert!(!keyed(0, PropKind::TranslationX), "Step NAO keya X");
    assert!(keyed(1, PropKind::TranslationX), "Glide keya X");
    assert!(!keyed(1, PropKind::TranslationY), "Glide NAO keya Y");
}

/// A cena `=4`/`=5` afirma: Run & jump 2×, Idle pack não posto, Step 1×; faixas Base 1.0, Upper
/// body 0.6, FX muda; e cada peça a velocidade 1.
#[test]
fn a_montagem_poe_cada_container_onde_o_eprintln_diz() {
    let mut doc = TimelineDoc::default();
    build_montagem(&mut doc, 1);
    let clips: Vec<_> = doc.clips().iter().map(|c| c.name.clone()).collect();
    assert_eq!(clips.join(" "), "idle walk jump swing shake");
    let nomes: Vec<_> = doc.containers().iter().map(|c| c.name.clone()).collect();
    assert_eq!(nomes.join(", "), "Run & jump, Idle pack, Step");
    assert_eq!([0, 1, 2].map(|i| doc.container_uses(i)), [2, 0, 1]);
    assert_eq!(
        doc.view_authored_end(None, false),
        Some(super::MONTAGEM_END),
        "o fim da cena"
    );
    let cena = doc.host_stack(StackHost::Document).expect("a cena");
    let faixas: Vec<_> = cena.iter().map(|l| (l.weight, l.muted)).collect();
    assert_eq!(faixas, [(1.0, false), (0.6, false), (1.0, true)]);
    let mut pecas = cena.iter().flat_map(|l| &l.strips);
    assert!(pecas.all(|s| (s.speed - 1.0).abs() < 1e-9), "velocidade 1");
}
