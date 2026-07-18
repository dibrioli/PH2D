//! Gates da ponte da seção Effects. **Nenhum nomeia um efeito**: eles varrem a tabela do
//! motor, então um tipo novo é coberto sem que este arquivo saiba que ele existe.

use super::*;
use ph2d_vec_scene::{VecPath, VecVertex};

fn scene_with_square() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let id = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    });
    (scene, id)
}

/// A seção é POR-CAMINHO: com zero ou dois selecionados não há referente.
#[test]
fn the_section_governs_exactly_one_selected_path() {
    assert_eq!(sole_path(&[]), None);
    assert_eq!(sole_path(&[7]), Some(7));
    assert_eq!(sole_path(&[7, 9]), None, "dois: a pilha não tem referente");
}

/// **Pôr QUALQUER efeito não pode mudar o desenho.** Varre a tabela inteira — um tipo novo
/// que nascesse a fazer alguma coisa faria a forma saltar no clique, e este gate o apanha sem
/// ser reescrito.
#[test]
fn adding_any_effect_does_not_move_a_single_point() {
    for kind in 0..PathEffect::KINDS.len() {
        let (mut scene, id) = scene_with_square();
        let before = scene.path(id).expect("path").cooked().into_owned();
        add(&mut scene, id, kind);
        assert_eq!(stack_view(&scene, id).len(), 1, "o efeito {kind} entrou");
        let after = scene.path(id).expect("path").cooked().into_owned();
        assert_eq!(
            before.verts,
            after.verts,
            "{} moveu a forma ao ser adicionado",
            PathEffect::KINDS[kind]
        );
    }
}

/// Um tipo fora da tabela é no-op — não entra um efeito errado nem há pânico.
#[test]
fn an_unknown_kind_adds_nothing() {
    let (mut scene, id) = scene_with_square();
    add(&mut scene, id, PathEffect::KINDS.len() + 3);
    assert!(stack_view(&scene, id).is_empty());
}

/// **A pilha respeita o teto** — e o teto é o mesmo que o painel registra.
#[test]
fn the_stack_stops_at_the_ceiling() {
    let (mut scene, id) = scene_with_square();
    for _ in 0..MAX_PATH_EFFECTS + 3 {
        add(&mut scene, id, 0);
    }
    assert_eq!(stack_view(&scene, id).len(), MAX_PATH_EFFECTS);
}

/// **Reordenar TROCA de lugar**, e nas bordas é no-op.
#[test]
fn reordering_swaps_and_the_edges_are_inert() {
    let (mut scene, id) = scene_with_square();
    add(&mut scene, id, 0);
    add(&mut scene, id, 1);
    let labels =
        |s: &VecScene| -> Vec<&'static str> { stack_view(s, id).iter().map(|r| r.label).collect() };
    let original = labels(&scene);
    assert_eq!(original.len(), 2);

    reorder(&mut scene, id, 0, true); // subir a primeira: nada
    assert_eq!(labels(&scene), original, "subir na borda de cima é no-op");
    reorder(&mut scene, id, 1, false); // descer a última: nada
    assert_eq!(labels(&scene), original, "descer na borda de baixo é no-op");

    reorder(&mut scene, id, 0, false);
    let swapped = labels(&scene);
    assert_eq!(swapped[0], original[1], "a ordem trocou");
    assert_eq!(swapped[1], original[0]);
}

/// **O snapshot descreve o que o motor declara** — nome, faixa, tipo e valor, para todo tipo.
/// É o gate que garante que o painel desenha o efeito CERTO sem o conhecer.
#[test]
fn the_snapshot_mirrors_what_the_engine_declares() {
    for kind in 0..PathEffect::KINDS.len() {
        let (mut scene, id) = scene_with_square();
        add(&mut scene, id, kind);
        let rows = stack_view(&scene, id);
        let fx = PathEffect::from_kind(kind).expect("kind");
        assert_eq!(rows[0].label, fx.label());
        assert_eq!(rows[0].params.len(), fx.params().len());
        for (i, (view, decl)) in rows[0].params.iter().zip(fx.params().iter()).enumerate() {
            assert_eq!(view.name, decl.name);
            assert_eq!(view.min, decl.min);
            assert_eq!(view.max, decl.max);
            assert_eq!(view.toggle, decl.toggle);
            assert!((view.value - fx.get(i)).abs() < 1e-12);
        }
    }
}

/// **O track NORMALIZADO vira o valor do documento pela faixa do EFEITO.**
///
/// É a fronteira de unidades: o painel manda `0..1` e não conhece a faixa; se a conversão
/// vivesse lá, haveria duas cópias da faixa e elas divergiriam no primeiro efeito com faixa
/// diferente — que é exatamente o que o Zig Zag trouxe (`Size` vai a 100, `Ridges` a 64).
#[test]
fn the_normalised_track_lands_on_the_effects_own_range() {
    for kind in 0..PathEffect::KINDS.len() {
        let (mut scene, id) = scene_with_square();
        add(&mut scene, id, kind);
        let decls = PathEffect::from_kind(kind).expect("kind").params().to_vec();
        for (p, d) in decls.iter().enumerate() {
            set_param(&mut scene, id, 0, p, 1.0);
            let top = stack_view(&scene, id)[0].params[p].value;
            assert!(
                (top - d.max).abs() < 1e-9,
                "track 1.0 em {}::{} devia dar {}, deu {top}",
                PathEffect::KINDS[kind],
                d.name,
                d.max
            );
            set_param(&mut scene, id, 0, p, 0.0);
            let bottom = stack_view(&scene, id)[0].params[p].value;
            assert!((bottom - d.min).abs() < 1e-9);
        }
    }
}

/// A caixinha alterna, e o `is_toggle` distingue-a do slider — sem isso um clique perdido num
/// slider viraria um toggle silencioso.
#[test]
fn a_toggle_flips_and_a_slider_is_not_a_toggle() {
    let (mut scene, id) = scene_with_square();
    // O tipo que TEM caixinha, achado pela tabela e não por nome.
    let Some((kind, param)) = (0..PathEffect::KINDS.len()).find_map(|k| {
        PathEffect::from_kind(k)?
            .params()
            .iter()
            .position(|d| d.toggle)
            .map(|p| (k, p))
    }) else {
        return; // nenhum efeito tem caixinha hoje — o gate dorme, não mente
    };
    add(&mut scene, id, kind);
    assert!(is_toggle(&scene, id, 0, param));
    assert!(
        !is_toggle(&scene, id, 0, usize::MAX),
        "índice fora não é toggle"
    );
    let before = stack_view(&scene, id)[0].params[param].value;
    toggle_param(&mut scene, id, 0, param);
    let after = stack_view(&scene, id)[0].params[param].value;
    assert!((before - after).abs() > 0.5, "a caixinha tem de alternar");
}
