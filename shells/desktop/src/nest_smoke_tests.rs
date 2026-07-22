//! Gates da cena `PH2D_NEST_SMOKE=3` — **a cena afirma seis números; aqui eles são medidos.**
//!
//! Uma cena de smoke que monta menos do que anuncia é indistinguível de uma feature quebrada:
//! o Colorize pagou isso (a cena dizia montar 3 chaves marcadas e marcava zero, e o smoke
//! inteiro virou ruído). Estes testes dirigem a **mesma** [`super::build_library`] que o app
//! chama — não um espelho dela.

use super::{LIBRARY_END, build_library};
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

/// A lista tem TRÊS barras, e cada uma mede o que o `eprintln` diz.
///
/// A do Pause é a que importa: um container VAZIO responde
/// [`ph2d_timeline::EMPTY_CONTAINER_SECONDS`] pela porta única, e é isso que dá à instância
/// dele uma janela real em vez de uma que fica presa em zero para sempre.
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

/// A cena põe os três assets, nas janelas anunciadas, e **em velocidade 1**.
///
/// A velocidade não é decoração: `add_strip_to` deriva `speed = slice/span`, então uma janela
/// que eu tivesse escrito torta viraria câmera lenta silenciosa — a `=2` usa isso de propósito,
/// e esta cena não.
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

/// **A CASCATA** — o gesto que só esta cena expõe, e o que a cena promete que ele faz.
///
/// Apagar o Step tem de matar as instâncias nos DOIS lugares (a solta na cena e as três de
/// dentro do Walk) e re-indexar o que sobra: o Walk era 1, passa a 0, e a instância dele na
/// cena continua apontando para o WALK — não para o vizinho que escorregou para o slot.
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

/// Os dois clips keyam canais **disjuntos**, que é o que faz as duas lanes do Walk somarem em
/// vez de brigarem. Se o Step keyasse X, as três repetições voltariam ao mesmo lugar — o passo
/// não atravessaria a tela, e a cena leria como quebrada.
#[test]
fn the_two_clips_key_disjoint_channels() {
    use ph2d_timeline::PropKind;
    let doc = library();
    // O `target` de cada (objeto, propriedade) vem da tabela de bindings do documento — a
    // mesma que o `upsert_key` consultou ao escrever.
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
