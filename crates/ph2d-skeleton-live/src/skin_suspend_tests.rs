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
        assert!(crate::skin_image_bind::bind_image(
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

    let quadro = |sim: &SimWorld, suspensas: &[Entity]| -> (bool, bool) {
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
            &suspensas.iter().map(|e| e.to_bits()).collect::<Vec<_>>(),
        );
        (
            present.world().get::<SpriteMesh>(pa).is_some(),
            present.world().get::<SpriteMesh>(pb).is_some(),
        )
    };

    assert_eq!(
        quadro(&sim, &[]),
        (true, true),
        "sem ninguem a pintar, as duas imagens presas deformam"
    );
    assert_eq!(
        quadro(&sim, &[a]),
        (false, true),
        "a suspensa tinha de ficar SEM malha (para ser pintada achatada) e a VIZINHA tinha de \
         continuar deformada — senao isto e' um interruptor geral com nome de excepcao"
    );
    assert_eq!(
        quadro(&sim, &[]),
        (true, true),
        "ao deixar de suspender, a deformacao volta no quadro seguinte — sem estado a repor"
    );
}

/// Três imagens presas ao mesmo osso — para os gates do CONJUNTO e do soltar.
fn tres_presas() -> (SimWorld, [Entity; 3]) {
    let mut sim = SimWorld::default();
    let osso = crate::bone::create(&mut sim, None, [-2.0, 0.0], [2.0, 0.0]).expect("osso");
    let raiz = Entity::from_bits(osso);
    let mut nasce = || {
        let e = sim
            .world_mut()
            .spawn((Transform::IDENTITY, sprite(4.0, 2.0, 0.0, 0.0)))
            .id();
        assert!(crate::skin_image_bind::bind_image(
            &mut sim,
            e,
            &tinta(40, 20, 4, 4),
            [40, 20],
            PPM,
            GridOptions::default(),
            Some(raiz),
        ));
        e
    };
    let es = [nasce(), nasce(), nasce()];
    (sim, es)
}

/// Que imagens recebem malha neste quadro, com estas suspensas.
fn com_malha(sim: &SimWorld, es: &[Entity], suspensas: &[u64]) -> Vec<bool> {
    let mut present = PresentWorld::new();
    let ps: Vec<Entity> = es
        .iter()
        .map(|e| {
            present
                .world_mut()
                .spawn((SimRef(*e), instancia_de(&sprite(4.0, 2.0, 0.0, 0.0))))
                .id()
        })
        .collect();
    attach_skin_meshes(sim, &mut present, PPM, suspensas);
    ps.iter()
        .map(|p| present.world().get::<SpriteMesh>(*p).is_some())
        .collect()
}

/// ⭐⭐ **A SUSPENSÃO É UM CONJUNTO** (regra F6-s, 2026-09-16): o Padding, o Upscale e o Equalize
/// Sizes editam a SELECÇÃO inteira, e toda imagem selecionada fica plana — a de fora continua
/// dobrada.
///
/// (Mutação: só o 1.º elemento do conjunto ser suspenso ⇒ RED.)
#[test]
fn every_suspended_image_is_flat_and_the_rest_stays_bent() {
    let (sim, [a, b, c]) = tres_presas();
    assert_eq!(
        com_malha(&sim, &[a, b, c], &[a.to_bits(), c.to_bits()]),
        vec![false, true, false],
        "as duas suspensas tinham de ficar planas e a do meio dobrada"
    );
    assert_eq!(com_malha(&sim, &[a, b, c], &[]), vec![true, true, true]);
}

/// ⭐⭐ **SOLTAR tira a pele de uma imagem presa, e de mais nada** (regra F6-s: uma ferramenta que
/// muda o tamanho ou a margem, aplicada, quebra a ligação com os ossos).
///
/// (Mutações: o soltar sem a guarda da imagem presa ⇒ RED no controlo; o soltar que não remove ⇒
/// RED.)
#[test]
fn releasing_an_image_takes_its_skin_and_nothing_else() {
    let (mut sim, [a, b, _]) = tres_presas();
    assert!(
        release_image(&mut sim, a.to_bits()),
        "a imagem estava presa"
    );
    assert!(!is_skinned_image(sim.world(), a), "a pele tinha de sair");
    assert!(is_skinned_image(sim.world(), b), "a vizinha continua presa");
    assert!(
        !release_image(&mut sim, a.to_bits()),
        "soltar outra vez nao tem o que soltar"
    );
    // ⛔ O CONTROLO: uma entidade com pele e SEM sprite (uma forma vectorial presa) não é tocada —
    // ela solta-se pela porta que devolve a geometria autorada.
    let pele = sim
        .world()
        .get::<ph2d_skeleton_ecs::SkinBind>(b)
        .cloned()
        .expect("pele da vizinha");
    let forma = sim.world_mut().spawn((Transform::IDENTITY, pele)).id();
    assert!(!release_image(&mut sim, forma.to_bits()));
    assert!(
        sim.world()
            .get::<ph2d_skeleton_ecs::SkinBind>(forma)
            .is_some(),
        "a porta das imagens tirou a pele de uma entidade que nao e' imagem"
    );
    // A imagem solta volta a ser desenhada como quad.
    assert_eq!(com_malha(&sim, &[a, b], &[]), vec![false, true]);
}
