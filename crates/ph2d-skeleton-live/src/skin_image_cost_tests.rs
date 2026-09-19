//! ⏱️ **O QUE A PELE DE IMAGEM CUSTA À CPU NUM QUADRO** — irmão do [`super`] pelo tecto de LOC,
//! cortado por RESPONSABILIDADE: *a 2.ª mídia desenha o que deve* é uma pergunta, *quanto ela custa
//! por peça* é outra, e a segunda é uma sonda sem barra.
//!
//! ⚠️ Filho do arnês do irmão (`#[path]`) para herdar as fixturas dele — uma cópia divergiria no
//! primeiro ajuste.

use super::*;

/// A cena CHEIA: `n` imagens do tamanho da cena do smoke (`512 × 320` px opacos, a malha de bind do
/// PRODUTO), presas à mesma corrente de três ossos dobrada `graus` por junta. Devolve o mundo, as
/// instâncias desenhadas e as peças que cada imagem guarda.
fn cena_cheia(n: usize, graus: f32) -> (SimWorld, PresentWorld, usize) {
    const LARG: u32 = 512;
    const ALT: u32 = 320;
    let largura = f64::from(LARG) / f64::from(PPM);
    let passo = largura / 3.0;
    let mut sim = SimWorld::default();
    let mut ossos = Vec::new();
    for k in 0..3 {
        let x0 = -largura / 2.0 + passo * f64::from(k);
        let pai = ossos.last().copied();
        let b = crate::bone::create(&mut sim, pai, [x0, 0.0], [x0 + passo, 0.0]).expect("osso");
        ossos.push(Entity::from_bits(b));
    }
    let s = sprite(LARG as f32 / PPM, ALT as f32 / PPM, 0.0, 0.0);
    let tinta = tinta(LARG, ALT, 0, 0);
    let mut present = PresentWorld::new();
    let mut guardadas = 0;
    for _ in 0..n {
        let e = sim.world_mut().spawn((Transform::IDENTITY, s)).id();
        assert!(crate::skin_image_bind::bind_image(
            &mut sim,
            e,
            &tinta,
            [LARG, ALT],
            PPM,
            GridOptions::default(),
            ossos.first().copied(),
        ));
        guardadas = mesh_of(&sim, e).expect("malha").tris.len();
        present.world_mut().spawn((SimRef(e), instancia_de(&s)));
    }
    for osso in ossos.iter().skip(1) {
        sim.world_mut()
            .get_mut::<Transform>(*osso)
            .expect("Transform")
            .rotation += graus.to_radians();
    }
    (sim, present, guardadas)
}

/// As peças que o quadro de facto entregou, com a lei `modo`.
fn entregues(sim: &SimWorld, present: &mut PresentWorld) -> usize {
    let ids: Vec<Entity> = present
        .world_mut()
        .query_filtered::<Entity, With<SpriteMesh>>()
        .iter(present.world())
        .collect();
    for p in ids {
        present.world_mut().entity_mut(p).remove::<SpriteMesh>();
    }
    attach_skin_meshes(sim, present, PPM, &[]);
    present
        .world_mut()
        .query::<&SpriteMesh>()
        .iter(present.world())
        .map(|m| m.tris.len())
        .sum()
}

/// ⭐⭐⭐ **A MALHA ASSADA É UM CHÃO QUE O TAMANHO DA CENA NÃO ERODE.**
///
/// ⚠️⚠️ **A premissa deste gate mudou em 2026-09-17, e o nome dele com ela.** Ele chamava-se
/// `o_smooth_alisa_em_qualquer_cena` e existia para responder ao report que abriu a F9 (*«uma cena
/// com muita arte presa fica com o `Smooth` igual ao `Fast`»*). No mesmo dia mediu-se que o `Fast` e
/// o `Smooth` desenhavam a mesma coisa a menos de `0,04 px`, o dono mandou apagar a fileira, e a
/// escolha deixou de existir. ⇒ o que resta a afirmar não é uma comparação entre duas leis: é que a
/// **única** lei que ficou entrega a malha do BIND e que essa entrega **não depende de quantas
/// imagens a cena tem**.
///
/// ⚠️ **O chão sai da LEI do produto** ([`crate::skin_bake::assar`]) sobre a malha desta cena, e não
/// de um número escrito aqui: *um chão escrito à mão envelhece no dia em que a tolerância mudar, e
/// ela é DERIVADA da diagonal da arte.*
#[test]
fn a_malha_assada_e_o_chao_em_qualquer_cena() {
    let mut chaos = Vec::new();
    for n in [1_usize, 4, 8] {
        let (sim, mut present, por_imagem) = cena_cheia(n, 60.0);
        let arte = sim
            .world()
            .iter_entities()
            .find(|er| crate::skin_image::is_skinned_image(sim.world(), er.id()))
            .map(|er| er.id())
            .expect("ha' arte presa");
        let sm = skinned_mesh_of(&sim, arte).expect("malha guardada");
        let chao = crate::skin_bake::assar(&sm.mesh, &sm.pesos, sm.ossos())
            .map_or(por_imagem, |(m, _)| m.tris.len());
        let desenhadas = entregues(&sim, &mut present);
        assert!(
            desenhadas >= chao * n,
            "com {n} imagens o quadro entregou {desenhadas}, abaixo do chao de {} ({chao} por \
             imagem) — a densidade voltou a depender do tamanho da cena",
            chao * n
        );
        chaos.push(desenhadas / n);
    }
    // ⛔ E o CONTROLO: a entrega POR IMAGEM é a mesma nas três — sem ele a metade de cima passaria
    // com uma lei que entrega mais por imagem quando a cena é pequena.
    assert!(
        chaos.windows(2).all(|w| w[0] == w[1]),
        "a malha por imagem mudou com o tamanho da cena: {chaos:?}"
    );
}
