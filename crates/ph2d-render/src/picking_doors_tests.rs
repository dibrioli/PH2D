//! ⭐⭐⭐ **AS PORTAS DE CANVAS da malha** — irmão do [`super::tests`] por tecto de LOC
//! (`architecture_workspace_file_loc_cap`), cortado por RESPONSABILIDADE: ali mede-se quem
//! **APONTA** (o picking e as caixas); aqui, as portas que dizem **onde a arte desenha um texel** e
//! **o que ela desenha ali**, que é o que o chrome e a tinta de um overlay precisam de saber.
//!
//! Os arneses (`spawn_at`, `give_mesh`, `posed_arm`, `fresh_sim_entity`) vêm do módulo pai — a
//! fixtura é a MESMA, de propósito: uma malha posada para `x = 3..5`, o único caso em que o quad e
//! a malha discordam.

use super::*;

/// ⭐⭐⭐ **A DIRECÇÃO QUE FALTAVA: onde é que a arte dobrada DESENHA este texel?**
///
/// ⛔⛔ Sem ela, quem pinta chrome por cima do canvas mapeia pelo afim do QUAD DE REPOUSO — e desde
/// que o ponteiro passou a ser resolvido pela malha (2026-09-14), o editor de curva do Painter
/// ficou a **agarrar num sítio e a desenhar noutro**. ⚠️ *Um controlo desenhado por um mapa e
/// agarrado por outro é um controlo morto sob o dedo.*
///
/// As duas metades: a IDA e a VOLTA fecham (é o mesmo texel), e a resposta **não é a do quad** —
/// sem a segunda, um mapa que ignorasse a malha passaria a primeira em repouso.
#[test]
fn the_drawn_mesh_says_where_a_texel_lands_and_it_is_not_the_rest_quad() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);
    give_mesh(&mut present, e, posed_arm());

    // IDA: que texel está debaixo deste ponto do mundo?
    let crate::MeshUv::Use { u, v, .. } =
        crate::mesh_uv(present.world_mut(), bits, [3.5, 0.5], true, SEM_DAB)
    else {
        panic!("o ponto está sobre a arte desenhada");
    };
    // VOLTA: onde é que a arte desenha ESSE texel? No sítio de onde se veio.
    let malha =
        crate::drawn_mesh_of(present.world(), bits).expect("a sprite desenha-se como malha");
    let p = malha.world_at_uv([u, v]).expect("o texel é desenhado");
    assert!(
        (p[0] - 3.5).abs() < 1e-4 && (p[1] - 0.5).abs() < 1e-4,
        "a ida e a volta têm de fechar no mesmo ponto, e deram {p:?}"
    );
    // ⛔ **O DISCRIMINADOR**: o quad de repouso desta sprite põe aquele texel em `(-0,5, -0,5)`.
    // Um mapa que ignorasse a malha responderia ali, e a asserção de cima **também passaria** se a
    // ida a ignorasse igualmente — é este par que separa as duas leis.
    assert!(
        (p[0] + 0.5).abs() > 1.0,
        "a resposta é a da MALHA POSADA, nunca a do quad de repouso ({p:?})"
    );

    // Fora da malha, em UV: este triângulo cobre METADE da imagem, e o canto oposto não é desenhado.
    assert_eq!(
        malha.world_at_uv([0.95, 0.05]),
        None,
        "um texel que a arte não desenha não tem sítio no ecrã"
    );

    // ⛔ O CONTROLO: uma sprite SEM malha não passa por esta porta — o chamador fica com o afim
    // dele, que é o que mantém a grelha da folha e o *Repeat Image* intocados.
    let plain = fresh_sim_entity(&mut sim);
    let pbits = spawn_at(&mut present, plain, 20.0, 0.0, [2.0, 2.0]);
    assert!(
        crate::drawn_mesh_of(present.world(), pbits).is_none(),
        "uma sprite que se desenha como QUAD não tem malha a oferecer"
    );
}

/// ⭐⭐ **A instância e a malha, para quem quer desenhar OUTRA COISA no mesmo sítio.**
///
/// A tinta da máscara da Remoção de fundo era um desenho do Vello com o afim do quad de repouso;
/// hoje é uma instância deste passe com a MESMA malha. ⛔ O caminho do Vello por recortes está
/// medido e refutado (costuras, e buffers fixos que degradam em silêncio) — ver o doc da porta.
#[test]
fn the_drawn_instance_carries_the_mesh_it_is_drawn_with() {
    let mut sim = ph2d_ecs::SimWorld::new();
    let mut present = PresentWorld::new();
    let e = fresh_sim_entity(&mut sim);
    let bits = spawn_at(&mut present, e, 0.0, 0.0, [2.0, 2.0]);

    // Controlo: um quad simples devolve a instância e NENHUMA malha — quem copiar a instância
    // desenha o quad, que é o que a arte faz.
    let (inst, malha) = crate::drawn_instance_of(present.world(), bits).expect("a sprite existe");
    assert_eq!(inst.size, [2.0, 2.0]);
    assert!(malha.is_none(), "sem `SpriteMesh` não há malha a copiar");

    give_mesh(&mut present, e, posed_arm());
    let (inst, malha) = crate::drawn_instance_of(present.world(), bits).expect("a sprite existe");
    assert_eq!(inst.size, [2.0, 2.0], "a instância é a MESMA da arte");
    assert_eq!(
        malha.map(|m| m.tris.len()),
        Some(1),
        "a malha POSADA tem de viajar com a instância: sem ela a tinta seria desenhada sobre o \
         quad de repouso enquanto a arte que ela anota está dobrada"
    );

    assert!(
        crate::drawn_instance_of(present.world(), u64::MAX).is_none(),
        "uma entidade que não é sprite não tem instância a oferecer"
    );
}
