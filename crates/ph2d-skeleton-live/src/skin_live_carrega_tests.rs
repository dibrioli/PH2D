//! A4 — ao abrir um projecto, a forma presa com efeitos vivos coze-os sem mover um pixel.

use crate::skin_desenho::Leis;
use crate::skin_live::tests::osso;
use crate::test_support::pior_desvio_do_desenho;
use ph2d_skeleton_ecs::SkinBind;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::{ShapeKind, VecPath, VecScene, cook};

/// Uma forma presa SEM efeitos (o Bind de antes da F51 não os cozia) e a pilha viva posta depois —
/// o estado de um projecto anterior —, a ponta dobrada a `60°`.
fn projecto_antigo(
    forma: VecPath,
    efeito: PathEffect,
) -> (ph2d_ecs::SimWorld, VecScene, u64, ph2d_vec_scene::VecPathId) {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut scene = VecScene::new();
    let mut map = ph2d_vec_entities::entities::VecEntityMap::new();
    let id = scene.push_path(forma);
    ph2d_vec_entities::entities::sync(&mut sim, &mut scene, &mut map);
    let raiz = osso(&mut sim, "Root", [0.0, 5.0], 20.0, None);
    let ponta = osso(&mut sim, "Tip", [20.0, 0.0], 20.0, Some(raiz));
    assert_eq!(crate::skin_live::bind(&mut sim, &mut scene, &map, &[id], Some(raiz)), 1);
    scene.path_mut(id).expect("path").effects = vec![FxEntry::new(efeito)];
    sim.world_mut()
        .get_mut::<ph2d_ecs::Transform>(ponta)
        .expect("Transform")
        .rotation = 60f32.to_radians();
    (sim, scene, map[&id], id)
}

fn desenho(sim: &ph2d_ecs::SimWorld, scene: &VecScene, id: ph2d_vec_scene::VecPathId) -> VecPath {
    crate::skin_live::recook_leis(sim, &mut scene.clone(), Leis::do_ambiente())
        .remove(&id)
        .expect("desenho")
}

/// ⭐⭐⭐ **GATE — cozer no carregamento não move um pixel, esvazia a pilha e marca a fonte; uma 2.ª
/// vez não faz nada.** Num *Twist* sobre um rectângulo e num *Zig Zag* sobre um de quinas redondas.
///
/// ⛔ **O CONTROLO:** a cura ingénua — a mesma geometria cozida com o campo da FONTE (o domínio de
/// antes do efeito) — move o desenho (o rasgo da F50-d).
#[test]
fn cozer_no_carregamento_nao_move_um_pixel() {
    let casos = [
        (
            cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[]),
            PathEffect::Twist(ph2d_vec_scene::fx_twist::TwistSpec { angle: 40.0 }),
        ),
        (
            cook(ShapeKind::RoundRect, [0.0, 0.0], [40.0, 10.0], &[5.0]),
            PathEffect::ZigZag(ph2d_vec_scene::fx_zigzag::ZigZagSpec {
                amplitude: 6.0,
                ridges: 24.0,
                ..Default::default()
            }),
        ),
    ];
    for (forma, efeito) in casos {
        let (mut sim, mut scene, bits, id) = projecto_antigo(forma, efeito);
        let e = ph2d_ecs::Entity::from_bits(bits);
        let antes = desenho(&sim, &scene, id);
        let antiga = crate::skinned_mesh::le(&sim.world().get::<SkinBind>(e).expect("pele").source)
            .expect("fonte");
        assert!(!antiga.efeitos_cozidos, "a fixtura não é um projecto antigo");
        // O CONTROLO: a geometria cozida com o campo da fonte.
        let ingenuo = {
            let mut viva = antiga.path.clone();
            viva.effects = scene.path(id).expect("path").effects.clone();
            let caminho = crate::skin_desenho_voltas::parte_nas_voltas(viva.cooked().into_owned());
            let campo = antiga.campo.clone().expect("campo");
            let pesos = ph2d_vec_skin::pesos::pesos_dos_pontos(&caminho, &campo);
            let g = crate::skinned_mesh::SkinnedPath {
                path: caminho,
                pesos,
                campo: Some(campo),
                efeitos_cozidos: true,
            };
            let original = sim.world().get::<SkinBind>(e).expect("pele").clone();
            let mut skin = original.clone();
            skin.source = crate::skinned_mesh::grava(&g).expect("grava");
            sim.world_mut().entity_mut(e).insert(skin);
            let mut scene2 = scene.clone();
            scene2.path_mut(id).expect("path").effects.clear();
            let d = pior_desvio_do_desenho(&antes, &desenho(&sim, &scene2, id));
            sim.world_mut().entity_mut(e).insert(original);
            d
        };
        assert_eq!(crate::skin_live::coze_os_efeitos_presos(&mut sim, &mut scene), 1);
        let depois = desenho(&sim, &scene, id);
        let pior = pior_desvio_do_desenho(&antes, &depois);
        println!("  desvio ao cozer {pior:.2e} · a cura ingénua {ingenuo:.3}");
        assert!(ingenuo > 1e-3, "o CONTROLO: o campo da fonte não muda o desenho ({ingenuo})");
        assert!(pior < 1e-9, "cozer no carregamento moveu o desenho em {pior}");
        assert!(scene.path(id).expect("path").effects.is_empty(), "a pilha ficou na cena");
        let nova = crate::skinned_mesh::le(&sim.world().get::<SkinBind>(e).expect("pele").source)
            .expect("fonte");
        assert!(nova.efeitos_cozidos, "a fonte não ficou marcada");
        assert_eq!(crate::skin_live::coze_os_efeitos_presos(&mut sim, &mut scene), 0, "2.ª vez");
    }
}

/// ⭐ **GATE de costura — a shell coze ANTES de desenhar a pele**, no quadro (se viesse depois, o 1.º
/// quadro desenhava a fonte velha com a pilha já vazia: o efeito sumia por um quadro).
#[test]
fn a_shell_coze_antes_de_desenhar_a_pele() {
    let fase = include_str!("../../../shells/desktop/src/render_loop/fase_vector_view_and_drives.rs");
    let coze = fase.find("coze_os_efeitos_presos(sim, vec_scene)");
    let pele = fase.find("recook_desenhando(sim, vec_scene)");
    assert!(pele.is_some(), "o CONTROLO: a fase já não desenha a pele aqui");
    assert!(coze.is_some_and(|c| Some(c) < pele), "a fase não coze antes da pele ({coze:?}, {pele:?})");
}
