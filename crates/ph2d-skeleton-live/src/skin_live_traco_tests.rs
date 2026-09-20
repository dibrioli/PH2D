//! ⭐⭐⭐ **O TRAÇO DE UMA FORMA PRESA AOS OSSOS SOBREVIVE AO QUADRO** — o report do dono de
//! 2026-09-19 (*«num vector linkado aos ossos não consigo mudar a espessura do stroke»*).
//!
//! ⛔ Ele vive ao lado do [`super::tests`] e não dentro dele por **tecto de LOC** (`701` contra
//! `700`). *O ficheiro é uma unidade de manutenção; a lei é a mesma.*

use super::*;
use ph2d_vec_scene::VecScene;

/// A barra da cena do dono com as duas juntas dobradas — ver [`super::alcas_tests`].
fn barra_dobrada() -> (SimWorld, VecScene, ph2d_vec_scene::VecPathId) {
    let (mut sim, scene, _m, id, ossos) = crate::barra_da_cena_tests_support::barra_da_cena();
    for &o in &ossos[1..] {
        sim.world_mut()
            .get_mut::<ph2d_ecs::Transform>(o)
            .expect("pose")
            // 30° em cada junta = a dobra de 60° do report.
            .rotation = std::f32::consts::FRAC_PI_6;
    }
    (sim, scene, id)
}

/// ⭐⭐⭐ **O TRAÇO QUE O ARTISTA PÕE NUMA FORMA PRESA AOS OSSOS SOBREVIVE AO QUADRO.**
///
/// Report do dono, 2026-09-19: *«num vector linkado aos ossos não consigo mudar a espessura do
/// stroke»*.
///
/// ⛔⛔ **A causa não era o painel:** o valor chegava ao documento e o re-cozimento da pele
/// devolvia-o. A fonte de uma pele é a **FOTOGRAFIA** tirada no `Bind`, e ela viajava pela
/// `replace_cooked`, que substitui o estilo — medido, o artista põe `0,2` e o quadro seguinte lê
/// `None`, sem um erro. *Um controlo que o produto desfaz no quadro seguinte lê-se exactamente
/// como um controlo morto.*
///
/// ⚠️ **A 2.ª asserção é o CONTROLO**: o quadro tem de continuar a escrever a GEOMETRIA, senão
/// uma pele que não faça nada passaria neste gate.
///
/// (Mutação: trocar `replace_geometry` por `replace_cooked` ⇒ RED na 1.ª.)
#[test]
fn o_traco_que_o_artista_poe_numa_forma_presa_sobrevive_ao_quadro() {
    const LARGURA: f64 = 0.2;
    let (sim, mut scene, id) = barra_dobrada();
    if let Some(p) = scene.path_mut(id) {
        p.stroke = Some(ph2d_vec_scene::StrokeSpec::new(
            ph2d_vec_scene::Rgba8::new(0, 0, 255, 255),
            LARGURA,
        ));
    }
    let ler = |sc: &VecScene| {
        sc.paths()
            .iter()
            .find(|p| p.id == id)
            .expect("a barra")
            .clone()
    };
    let antes = ler(&scene);
    crate::skin_live::recook(&sim, &mut scene);
    let depois = ler(&scene);

    assert_eq!(
        depois.stroke.as_ref().map(|s| s.width),
        Some(LARGURA),
        "o quadro desfez a espessura que o artista acabou de pôr — a fotografia do `Bind` voltou \
         a mandar no estilo do objecto"
    );
    // ⭐ O CONTROLO: o quadro escreveu de facto a geometria deste caminho.
    let mexeu = crate::test_support::pior_desvio(&antes, &depois);
    assert!(
        mexeu > 0.01,
        "o quadro nao mexeu na geometria ({mexeu}) — a pele nao correu, e a asserção acima fica \
         verde por vacuo"
    );
}
