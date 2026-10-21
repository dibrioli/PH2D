//! O texto que a CURVA da timeline muda redesenha-se ([`super::super::recook`]).

use super::*;

/// ⭐ **A curva que muda a receita de um texto redesenha-o** (plano 35 bloco 2): o `aplicar` do
/// vetor devolve o texto que mudou e a porta de sempre re-coze-o com a receita nova — sem isto o
/// tamanho/família/conteúdo animados mudavam o documento e a tela ficava parada.
#[test]
fn a_curve_on_a_text_recipe_redraws_the_text() {
    use ph2d_ecs::canal::familia::VETOR;
    use ph2d_ecs::canal_vetor as c;
    let mut f = fixture();
    f.recook();
    let largura = |f: &Fix| {
        let xs = f
            .scene
            .paths()
            .iter()
            .find(|p| p.id == f.text)
            .expect("texto")
            .verts_all()
            .map(|v| v.anchor[0])
            .collect::<Vec<_>>();
        xs.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            - xs.iter().copied().fold(f64::INFINITY, f64::min)
    };
    let antes = largura(&f);
    let e = f.text_entity();
    f.sim
        .world_mut()
        .get_resource_or_insert_with(ph2d_ecs::CanaisEscritos::default)
        .escrever(
            e.to_bits(),
            ph2d_ecs::FieldKey::new(VETOR, c::TEXTO_TAMANHO, 0, 0),
            1.5,
        );
    let mut conducao = ph2d_app_vec::canais::ConducaoVetor::default();
    let feito = ph2d_app_vec::canais::aplicar(&mut f.sim, &mut f.scene, &mut conducao);
    assert_eq!(feito.textos, vec![(f.text, e)]);
    for (id, e) in feito.textos {
        recook(&mut f.sim, &mut f.scene, &f.map, id, e);
    }
    let depois = largura(&f);
    assert!(
        depois > antes * 2.5,
        "o tamanho 0,5 -> 1,5 triplica a largura do texto: {antes} -> {depois}"
    );
}
