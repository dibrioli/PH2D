//! Os gates da SUSPENSÃO da pele — *pintar achata a arte* (ordem do dono, 2026-09-15).
//!
//! ⚠️ Ele é filho do [`super`] para herdar as fixturas daquele arnês (`sprite`, `tinta`,
//! `instancia_de`, as escalas): uma cópia delas divergiria no primeiro ajuste.

use super::*;

/// ⭐⭐⭐ **A SPRITE SUSPENSA NÃO RECEBE MALHA — e a vizinha recebe.**
///
/// Ordem do dono, 2026-09-15: *«inative a possibilidade de pintar sobre malha deformada por ossos;
/// ao entrar no Painter a imagem deixa a deformação, e ao sair ela retorna»*.
///
/// ⚠️⚠️ **As DUAS metades, e a segunda é a que importa:** sem a vizinha, um `suspensa` que
/// desligasse a deformação de TODA a cena passaria — e a 2.ª mídia deixava de existir enquanto
/// alguém pintasse. *Uma suspensão sem controlo é um interruptor geral com nome de excepção.*
///
/// ⭐ **E o regresso é medido no mesmo teste**, porque ele é a outra metade da ordem: basta deixar
/// de suspender, e a malha volta no quadro seguinte sem estado nenhum a repor.
#[test]
fn a_sprite_suspensa_nao_recebe_malha_e_a_vizinha_recebe() {
    let mut sim = SimWorld::default();
    let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let raiz = Entity::from_bits(osso);
    let faz = || sprite(4.0, 2.0, 0.0, 0.0);
    let nasce = |sim: &mut SimWorld| {
        let e = sim.world_mut().spawn((Transform::IDENTITY, faz())).id();
        assert!(crate::skin_live::bind_image(
            sim,
            e,
            &tinta(40, 20, 4, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            Some(raiz),
        ));
        e
    };
    let a = nasce(&mut sim);
    let b = nasce(&mut sim);

    let quadro = |sim: &SimWorld, suspensa: Option<Entity>| -> (bool, bool) {
        let mut present = PresentWorld::new();
        let pa = present
            .world_mut()
            .spawn((SimRef(a), instancia_de(&faz())))
            .id();
        let pb = present
            .world_mut()
            .spawn((SimRef(b), instancia_de(&faz())))
            .id();
        attach_skin_meshes(
            sim,
            &mut present,
            PPM,
            None,
            PX_POR_METRO,
            suspensa.map(Entity::to_bits),
        );
        (
            present.world().get::<SpriteMesh>(pa).is_some(),
            present.world().get::<SpriteMesh>(pb).is_some(),
        )
    };

    assert_eq!(
        quadro(&sim, None),
        (true, true),
        "sem ninguem a pintar, as duas imagens presas deformam"
    );
    assert_eq!(
        quadro(&sim, Some(a)),
        (false, true),
        "a suspensa tinha de ficar SEM malha (para ser pintada achatada) e a VIZINHA tinha de \
         continuar deformada — senao isto e' um interruptor geral com nome de excepcao"
    );
    assert_eq!(
        quadro(&sim, None),
        (true, true),
        "ao deixar de suspender, a deformacao volta no quadro seguinte — sem estado a repor"
    );
}
