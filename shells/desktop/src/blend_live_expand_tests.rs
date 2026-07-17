//! Testes do **Expand / Release** (ADR-0128 Fase D) — irmão de `blend_live_tests.rs`, separado pelo
//! teto de 600 LOC. Reusa `scene_with_blend` de lá (`super::tests`, `pub(super)`).
//!
//! O gate que carrega a fase é `expand_materializes_exactly_what_the_overlay_drew`. O Expand promete
//! entregar **o que está na tela**, e a única maneira de provar isso é comparar com o que o overlay
//! desenhou — geometria e estilo, byte a byte. Um gate que só CONTASSE as formas produzidas ficaria
//! verde com uma segunda porta de cozedura, e a divergência apareceria no olho do artista: as formas
//! saltariam no clique.

use super::tests::scene_with_blend;
use super::*;

/// O path `p` com o `id` trocado — o `id` é do documento (o passo desenhado nunca teve um), e é o
/// único campo que pode legitimamente diferir entre o desenhado e o materializado.
fn with_id(p: &VecPath, id: VecPathId) -> VecPath {
    let mut q = p.clone();
    q.id = id;
    q
}

/// Um pen com `sel` selecionado — é por ele que o Expand/Release sabem de que blend se fala.
fn pen_with(sel: &[VecPathId]) -> ph2d_vec_edit::PenTool {
    let mut pen = ph2d_vec_edit::PenTool::new();
    pen.select_many(sel);
    pen
}

/// Roda o `recook` (**o que a TELA mostra**) e em seguida o `expand`, devolvendo `(passos
/// desenhados, paths materializados, run de z, sim, scene, map, fontes)`.
///
/// `bend` autora o spine com um pico no meio. Sem isso o `offsets` sai vazio e o caminho do spine
/// AUTORADO nunca é exercitado — o fixture só prova o que contém, e o Expand de um blend cuja curva
/// foi editada é justamente o caso em que uma 2ª porta divergiria.
#[allow(clippy::type_complexity)]
fn overlay_then_expand(
    bend: bool,
) -> (
    Vec<VecPath>,
    Vec<VecPath>,
    Vec<Vec<VecPathId>>,
    SimWorld,
    VecScene,
    VecEntityMap,
    Vec<VecPathId>,
) {
    const STEPS: u32 = 3;
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(2, STEPS);
    if bend {
        let e = Entity::from_bits(map[&spine]);
        sim.world_mut()
            .get_mut::<VecBlend>(e)
            .expect("blend")
            .spine_authored = true;
        let p = scene.path_mut(spine).expect("spine");
        p.verts = [[0.0, 0.0], [2.0, 3.0], [4.0, 0.0]]
            .iter()
            .map(|&q| VecVertex::corner(q))
            .collect();
    }

    // O overlay: `out` é [passos do elo0, fonte1]. Os passos são o prefixo.
    let mut out = Vec::new();
    let xf = crate::vec_transform::build(&sim, &map);
    recook(
        &mut sim,
        &mut scene,
        &map,
        &xf,
        &mut BlendSpines::new(),
        &mut out,
    );
    let drawn: Vec<VecPath> = out[..STEPS as usize].to_vec();

    let before: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    let mut pen = pen_with(&[spine]);
    let xf = crate::vec_transform::build(&sim, &map);
    let runs = expand(&mut sim, &mut scene, &map, &xf, &mut pen);
    let made: Vec<VecPath> = scene
        .paths()
        .iter()
        .filter(|p| !before.contains(&p.id))
        .cloned()
        .collect();
    (drawn, made, runs, sim, scene, map, src.to_vec())
}

/// **A promessa da fase:** o que o Expand assa é EXATAMENTE o que o overlay desenhava — mesma
/// geometria, mesmo estilo interpolado, mesma ordem. Elas saem da mesma função (`cook_links`), e é
/// este gate que impede alguém de "otimizar" isso em duas.
///
/// Os passos nascem em MUNDO e sem pose (a entidade só nasce no `sync` seguinte, na identidade),
/// então comparar o path da cena com o passo do overlay é comparar mundo com mundo.
#[test]
fn expand_materializes_exactly_what_the_overlay_drew() {
    let (drawn, made, ..) = overlay_then_expand(false);
    assert_eq!(made.len(), drawn.len(), "um path por passo desenhado");
    for (m, d) in made.iter().zip(&drawn) {
        assert_eq!(with_id(m, d.id), *d, "o passo assado != o passo desenhado");
    }
}

/// O mesmo, com o spine EDITADO (o pico no meio): os passos que fluem pela curva têm de ser
/// materializados NA CURVA. É o caso em que o `offsets` não é vazio — sem este gate, o Expand podia
/// ignorar o spine autorado e o teste da reta ficaria verde do mesmo jeito.
#[test]
fn expand_of_a_bent_spine_materializes_the_steps_on_the_curve() {
    let (drawn, made, ..) = overlay_then_expand(true);
    assert_eq!(made.len(), drawn.len());
    for (m, d) in made.iter().zip(&drawn) {
        assert_eq!(with_id(m, d.id), *d);
    }
    // E a curva de fato levantou os passos (senão isto seria o teste da reta com outro nome).
    let ys: Vec<f64> = made
        .iter()
        .map(|p| p.verts.iter().map(|v| v.anchor[1]).sum::<f64>() / p.verts.len() as f64)
        .collect();
    assert!(
        ys.iter().any(|y| *y > 0.5),
        "o spine com pico em (2,3) tem de levantar algum passo; ys={ys:?}"
    );
}

/// O Expand devolve a sequência de z do resultado, **fundo → topo**: fonte0 → passos → fonte1. É a
/// pilha que o overlay já desenhava, agora escrita na árvore (`vec_restack`) — quem manda no z é ela,
/// não a ordem do vetor da cena.
#[test]
fn expand_returns_the_z_run_with_the_steps_between_the_sources() {
    let (_, made, runs, _sim, _scene, _map, src) = overlay_then_expand(false);
    assert_eq!(runs.len(), 1, "um blend tocado, um run");
    let mut want = vec![src[0]];
    want.extend(made.iter().map(|p| p.id));
    want.push(src[1]);
    assert_eq!(
        runs[0], want,
        "fonte0 · passos · fonte1, de baixo para cima"
    );
}

/// **O objeto vivo morre e as FONTES ficam** — expandir não consome os operandos (≠ booleana). O
/// spine sai da cena, e é isso que faz o `sync` despawnar a entidade e levar o `VecBlend` junto: sem
/// isso sobraria um path invisível e órfão na Hierarquia.
#[test]
fn expand_kills_the_blend_object_and_keeps_the_sources() {
    let (_, _, _, mut sim, mut scene, mut map, src) = overlay_then_expand(false);
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    assert_eq!(
        sim.world_mut()
            .query::<&VecBlend>()
            .iter(sim.world())
            .count(),
        0,
        "o objeto vivo morreu com o spine"
    );
    for id in &src {
        assert!(
            scene.paths().iter().any(|p| p.id == *id),
            "a fonte {id} sobrevive ao Expand"
        );
    }
}

/// **Release:** os passos somem, as fontes ficam, e nada é materializado. Como os passos são
/// VIRTUAIS, soltar o blend é remover o spine — o resto é consequência.
///
/// A seleção passa às FONTES: o spine acabou de sair da cena, e uma seleção apontando para um id
/// morto faz o painel falar de um objeto que não existe.
#[test]
fn release_drops_the_object_keeps_the_sources_and_makes_nothing() {
    let (mut sim, mut scene, map, spine, src) = scene_with_blend(2, 3);
    let mut pen = pen_with(&[spine]);
    assert!(release(&sim, &mut scene, &map, &mut pen));
    assert_eq!(
        scene.paths().len(),
        src.len(),
        "sobram SÓ as fontes (nenhum passo assado, e o spine saiu)"
    );
    for id in &src {
        assert!(scene.paths().iter().any(|p| p.id == *id));
    }
    let mut sel = pen.selected_paths().to_vec();
    sel.sort_unstable();
    let mut want = src.to_vec();
    want.sort_unstable();
    assert_eq!(sel, want, "a seleção passa às fontes");
    // E o objeto morre de verdade no sync (o `VecBlend` vai junto com a entidade do spine).
    let mut map2 = map.clone();
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map2);
    assert_eq!(
        sim.world_mut()
            .query::<&VecBlend>()
            .iter(sim.world())
            .count(),
        0
    );
}

/// Expand **e** Release respondem a uma FORMA-fonte selecionada, não só à linha — a mesma porta
/// (`blends_touched_by`) do Steps e do Reset Spine. No modo Select a linha nem é selecionável, então
/// sem isto os dois botões ficariam inertes justo no modo em que o artista trabalha.
#[test]
fn expand_and_release_answer_to_a_selected_source_shape() {
    // Release pela fonte.
    let (sim, mut scene, map, _spine, src) = scene_with_blend(2, 2);
    let mut pen = pen_with(&[src[0]]);
    assert!(release(&sim, &mut scene, &map, &mut pen));

    // Expand pela fonte.
    let (mut sim, mut scene, map, _spine, src) = scene_with_blend(2, 2);
    let xf = crate::vec_transform::build(&sim, &map);
    let mut pen = pen_with(&[src[1]]);
    let runs = expand(&mut sim, &mut scene, &map, &xf, &mut pen);
    assert_eq!(runs.len(), 1, "uma forma do blend seleciona o blend");
    assert_eq!(scene.paths().len(), 4, "2 fontes + 2 passos (o spine saiu)");
}

/// Uma seleção que não toca blend nenhum não expande nem solta nada — e, no caso do Expand, não
/// devolve run de z (o chamador não enfileira nada e diz ao artista o que fazer).
#[test]
fn a_selection_outside_any_blend_expands_and_releases_nothing() {
    let (mut sim, mut scene, map, _spine, _src) = scene_with_blend(2, 2);
    let outsider = scene.push_path(ph2d_vec_scene::rectangle([20.0, 20.0], [21.0, 21.0]));
    let n = scene.paths().len();
    let xf = crate::vec_transform::build(&sim, &map);
    let mut pen = pen_with(&[outsider]);
    assert!(expand(&mut sim, &mut scene, &map, &xf, &mut pen).is_empty());
    assert!(!release(&sim, &mut scene, &map, &mut pen));
    assert_eq!(scene.paths().len(), n, "a cena não foi tocada");
}
