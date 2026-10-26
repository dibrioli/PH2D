//! As cores do CAD chegam da análise fora do fio do quadro, só quando a estrutura muda; um conflito escreve a ficha
//! «apague uma destas» com o nome de cada regra envolvida.

use super::*;
use crate::cad_cores::CadCores;
use ph2d_vec_scene::cad::{RegraCad, SegCad};
use ph2d_vec_scene::{VecPathId, VertId, rectangle};

fn v(p: VecPathId, i: u32) -> PontoCad {
    PontoCad::Vert(p, VertId(i))
}

fn retangulo() -> (VecScene, VecPathId) {
    let mut scene = VecScene::new();
    let r = scene.push_path(rectangle([0.0, 0.0], [10.0, 5.0]));
    *scene.regras_mut() = vec![
        RegraCad::nova(Lei::Horizontal(SegCad(v(r, 1), v(r, 2)))),
        RegraCad::nova(Lei::Vertical(SegCad(v(r, 2), v(r, 3)))),
        RegraCad::nova(Lei::Fixo(v(r, 1), [0.0, 0.0])),
    ];
    (scene, r)
}

#[test]
fn the_colours_come_from_an_analysis_off_the_frame_thread_only_when_the_structure_changes() {
    let (mut scene, r) = retangulo();
    let xf = VecXforms::default();
    let cores = CadCores::default();
    assert!(cores.ve(&scene, &xf).is_none(), "o 1.º quadro não espera");
    cores.espera();
    let e = cores.ve(&scene, &xf).expect("a análise chegou");
    assert_eq!(e.regras, vec![EstadoRegra::Ok; 3]);
    assert_eq!(e.pontos[&v(r, 1)], Liberdade::Definido, "o fixo");
    assert_eq!(e.pontos[&v(r, 3)], Liberdade::Livre);

    // Arrastar mexe posições, não a estrutura: as cores ficam sem análise nova.
    scene.path_mut(r).unwrap().verts[2].anchor = [12.0, 7.0];
    assert!(cores.ve(&scene, &xf).is_some());

    // Uma regra nova é estrutura: até a análise voltar, nada (nunca a cor velha de outra estrutura).
    scene
        .regras_mut()
        .push(RegraCad::nova(Lei::Distancia(v(r, 1), v(r, 2), 10.0)));
    assert!(cores.ve(&scene, &xf).is_none());
    cores.espera();
    assert_eq!(cores.ve(&scene, &xf).expect("chegou").regras.len(), 4);

    // Mudar uma medida sem mudar quantas regras há também é estrutura.
    scene.regras_mut()[3] = RegraCad::nova(Lei::Distancia(v(r, 1), v(r, 2), 12.0));
    assert!(cores.ve(&scene, &xf).is_none(), "a cor da medida antiga");
}

#[test]
fn a_conflict_writes_which_rules_to_delete() {
    let (mut scene, r) = retangulo();
    // Uma V e uma medida num segmento H: só cabem com o segmento a zero, e a medida diz 10.
    scene.regras_mut().extend([
        RegraCad::nova(Lei::Vertical(SegCad(v(r, 1), v(r, 2)))),
        RegraCad::nova(Lei::Distancia(v(r, 1), v(r, 2), 10.0)),
    ]);
    let (xf, view) = (VecXforms::default(), VecViewState::default());
    let cores = CadCores::default();
    cores.ve(&scene, &xf);
    cores.espera();
    let e = cores.ve(&scene, &xf).expect("a análise chegou");
    let em_conflito: Vec<usize> = (0..5)
        .filter(|&i| e.regras[i] == EstadoRegra::Conflito)
        .collect();
    assert!(
        em_conflito.contains(&3),
        "a V num segmento H: {:?}",
        e.regras
    );

    let p = plano(&scene, &view, &xf, Affine::IDENTITY, &e);
    let nomes: Vec<String> = em_conflito
        .iter()
        .map(|&i| nome(&scene.regras()[i].lei))
        .collect();
    let (texto, _) = p.conflito.expect("a ficha do conflito");
    assert_eq!(
        texto,
        ph2d_i18n::tr_with("app.vec.cad.conflito", &[("regras", &nomes.join(" · "))])
    );
    assert!(texto.contains(&ph2d_i18n::tr("app.vec.cad.lei.vertical").to_string()));
    let vermelhos = p
        .tracos
        .iter()
        .filter(|t| t.2 == ColorToken::Danger)
        .count();
    assert_eq!(
        vermelhos,
        em_conflito.len(),
        "um traço vermelho por regra em conflito"
    );
    assert_eq!(p.pontos.len(), 3, "um losango por ponto citado");
}

/// ⛔ CONTROLO: sem conflito, nenhuma ficha; o fixo (um ponto só) não tem traço.
#[test]
fn without_a_conflict_there_is_no_chip() {
    let (scene, _) = retangulo();
    let (xf, view) = (VecXforms::default(), VecViewState::default());
    let cores = CadCores::default();
    cores.ve(&scene, &xf);
    cores.espera();
    let e = cores.ve(&scene, &xf).expect("chegou");
    let p = plano(&scene, &view, &xf, Affine::IDENTITY, &e);
    assert!(p.conflito.is_none());
    assert_eq!(p.tracos.len(), 2);
    assert!(p.tracos.iter().all(|t| t.2 == ColorToken::Info));
}
