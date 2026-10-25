//! Gate da caneta com a ferramenta **Corte** na mão (F29).

use super::*;

fn nosnap(p: [f64; 2]) -> [f64; 2] {
    p
}

const PTW: f64 = 0.01;

/// ⭐⭐ **O clique do Corte começa SEMPRE a lâmina** — em cima de um nó, ou sobre o segmento da
/// forma seleccionada, onde a caneta comum agarra o nó / insere outro (dono 25/10: *«não consigo
/// cortar nada com ela»*). O controlo: a caneta comum, no mesmo sítio, continua a agarrar.
///
/// Mutação que deve sangrar: o `press` ignorar o `lamina`.
/// Um triângulo fechado e seleccionado, como a caneta o deixa.
fn triangulo() -> (VecScene, PenTool) {
    let mut scene = VecScene::new();
    let mut pen = PenTool::new();
    for q in [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0]] {
        pen.on_press(&mut scene, q, PTW, false, &mut nosnap);
        pen.on_release();
    }
    pen.on_press(&mut scene, [0.02, 0.0], PTW, false, &mut nosnap);
    (scene, pen)
}

#[test]
fn o_clique_do_corte_comeca_a_lamina_mesmo_sobre_a_forma() {
    for alvo in [[4.0, 0.0], [2.0, 0.0]] {
        let (mut cena_comum, mut comum) = triangulo();
        assert_ne!(
            comum.on_press(&mut cena_comum, alvo, PTW, false, &mut nosnap),
            PenClick::Started,
            "controlo: a caneta comum em {alvo:?} edita a forma"
        );

        let (mut scene, mut pen) = triangulo();
        let forma = scene.paths()[0].clone();
        assert_eq!(
            pen.on_press_blade(&mut scene, alvo, PTW, &mut nosnap),
            PenClick::Started,
            "o Corte em {alvo:?} não começou a lâmina"
        );
        assert_eq!(scene.paths().len(), 2, "a lâmina é um caminho novo");
        assert_eq!(scene.paths()[0], forma, "a forma por baixo não pode mudar");
    }
}
