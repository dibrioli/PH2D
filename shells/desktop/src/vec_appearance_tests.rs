//! Gates da aparência do objecto — a lei da selecção múltipla e a porta que escreve.

use super::{set_blend, set_opacity};
// ⚠️ A publicacao da vista mudou-se para o `vec_paint_stack` quando ela passou a levar a PILHA.
use crate::vec_paint_stack::published;
use ph2d_vec_scene::{BlendMode, Opacity, VecPath, VecScene, VecVertex};

fn cena() -> (VecScene, Vec<u64>) {
    let mut scene = VecScene::default();
    let ids: Vec<u64> = (0..2)
        .map(|_| {
            scene.push_path(VecPath {
                verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
                    .map(VecVertex::corner)
                    .to_vec(),
                closed: true,
                ..VecPath::default()
            })
        })
        .collect();
    (scene, ids)
}

/// ⭐⭐⭐ **MOSTRA O PRIMÁRIO, ESCREVE EM TODOS** — a lei da selecção múltipla desta janela.
///
/// ⚠️ Mutação que tem de sangrar: publicar a partir da ÚLTIMA forma (o readout passaria a descrever
/// uma forma que o artista não apontou — tocar num filho selecciona o grupo, e a primeira é a que
/// ele tocou), ou escrever só na primeira (o gesto pareceria funcionar e metade da selecção ficaria
/// para trás).
#[test]
fn the_panel_reads_the_primary_and_the_gesture_writes_them_all() {
    let (mut scene, ids) = cena();
    scene.path_mut(ids[0]).expect("a").opacity = Opacity::new(0.25);
    scene.path_mut(ids[1]).expect("b").opacity = Opacity::new(0.75);
    let a = published(&scene, &ids).expect("ha' seleccao");
    assert!(
        (a.opacity - 0.25).abs() < 1e-6,
        "o readout e' do PRIMARIO, nao de outro da lista"
    );

    assert!(set_opacity(&mut scene, &ids, 0.5));
    for id in &ids {
        assert!(
            (scene
                .paths()
                .iter()
                .find(|p| p.id == *id)
                .expect("a forma")
                .opacity
                .get()
                - 0.5)
                .abs()
                < 1e-6,
            "a escrita tem de alcancar TODA a seleccao"
        );
    }
}

/// **Sem selecção não há seção** — `None` esconde-a, em vez de mostrar sliders que não descrevem
/// nada.
#[test]
fn an_empty_selection_publishes_nothing() {
    let (scene, _) = cena();
    assert!(published(&scene, &[]).is_none());
    assert!(
        published(&scene, &[999]).is_none(),
        "um id que a cena nao tem tambem nao publica — a forma pode ter sido apagada neste quadro"
    );
}

/// **Escrever o MESMO valor devolve `false`** — e é o que impede um passo de undo por quadro
/// enquanto o painel republica o que já lá está.
#[test]
fn writing_the_same_value_reports_no_change() {
    let (mut scene, ids) = cena();
    assert!(set_blend(&mut scene, &ids, BlendMode::Multiply.to_u8()));
    assert!(
        !set_blend(&mut scene, &ids, BlendMode::Multiply.to_u8()),
        "a 2.a escrita do mesmo modo nao mudou nada"
    );
    assert!(
        !set_opacity(&mut scene, &ids, 1.0),
        "e a opacidade neutra ja' era a de fabrica"
    );
}

/// ⭐ **O CÓDIGO que viaja do painel é o do MODO** — e um código de lixo cai no neutro em vez de
/// estourar.
///
/// ⚠️ `from_u8` fora da faixa devolve `Normal` (a lei do vocabulário), e é o que mantém o caminho
/// honesto quando um ficheiro do futuro traz um modo que este build não conhece.
#[test]
fn an_out_of_range_code_falls_back_to_normal() {
    let (mut scene, ids) = cena();
    set_blend(&mut scene, &ids, 200);
    assert_eq!(
        scene.paths()[0].blend,
        BlendMode::Normal,
        "um codigo desconhecido nao pode virar um modo qualquer"
    );
}
