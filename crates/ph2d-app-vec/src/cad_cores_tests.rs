//! O piscar dos lados (smoke do dono 27/10): uma regra ANIMADA muda de valor em todo quadro, e as cores da mesma
//! estrutura não podem sumir enquanto a análise nova corre.

use super::*;
use ph2d_vec_scene::VecVertex;
use ph2d_vec_scene::cad::Lei;

fn cena() -> (VecScene, PontoCad, PontoCad) {
    let mut scene = VecScene::new();
    let path = scene.push_path(VecPath {
        verts: [[0.0, 0.0], [2.0, 0.0], [2.0, 1.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
        ..VecPath::default()
    });
    let (a, b) = (
        PontoCad::Vert(path, VertId(1)),
        PontoCad::Vert(path, VertId(2)),
    );
    scene
        .regras_mut()
        .push(RegraCad::nova(Lei::Distancia(a, b, 2.0)));
    scene.normaliza_ids();
    (scene, a, b)
}

/// Espera a análise da cena de AGORA (a thread é real; o prazo é largo para a máquina carregada).
fn espera(c: &CadCores, scene: &VecScene, xf: &VecXforms) -> Arc<Estado> {
    for _ in 0..2_000 {
        if let Some(e) = c.ve(scene, xf)
            && i_pronto_e(c, scene)
        {
            return e;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    panic!("a análise não chegou em 10 s");
}

fn i_pronto_e(c: &CadCores, scene: &VecScene) -> bool {
    let i = c.interno.lock().unwrap_or_else(PoisonError::into_inner);
    i.pronto.as_ref().is_some_and(|(k, _)| *k == chave(scene))
}

/// Mutação que deve sangrar: devolver `None` no fim de `ve` (o comportamento de antes).
#[test]
fn an_animated_value_keeps_the_colours_until_the_new_analysis_arrives() {
    let (mut scene, a, b) = cena();
    let xf = VecXforms::default();
    let c = CadCores::default();
    let antes = espera(&c, &scene, &xf);
    scene.regras_mut()[0].lei = Lei::Distancia(a, b, 2.5);
    let agora = c
        .ve(&scene, &xf)
        .expect("os lados sumiram num quadro de valor animado");
    assert!(
        Arc::ptr_eq(&antes, &agora),
        "a resposta mostrada é a última que chegou"
    );
    let nova = espera(&c, &scene, &xf);
    assert!(
        !Arc::ptr_eq(&antes, &nova),
        "a análise do valor novo chega e substitui"
    );
}

/// A cor de OUTRA estrutura nunca se mostra: uma regra a mais espera a sua análise.
#[test]
fn a_new_rule_never_shows_the_colours_of_the_old_structure() {
    let (mut scene, a, b) = cena();
    let xf = VecXforms::default();
    let c = CadCores::default();
    espera(&c, &scene, &xf);
    scene
        .regras_mut()
        .push(RegraCad::nova(Lei::Coincidente(a, b)));
    scene.normaliza_ids();
    assert!(c.ve(&scene, &xf).is_none());
}
