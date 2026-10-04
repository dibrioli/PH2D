//! **A CORRENTE de uma cena do osso** — a cadeia pela ordem e o osso do meio, num irmão do
//! [`super`] pelo tecto de LOC (2026-10-03).

/// **A cadeia do primeiro osso ao último**, descendo pelo 1.º filho que é osso.
///
/// ⛔ Ela existe porque a [`ph2d_skeleton_live::esqueletos::ossos_desde`] devolve o CONJUNTO
/// ordenado por `to_bits` — bom para uma régua, errado para uma FRASE.
pub(super) fn cadeia_em_ordem(
    sim: &ph2d_ecs::SimWorld,
    raiz: ph2d_ecs::Entity,
) -> Vec<ph2d_ecs::Entity> {
    let mut out = vec![raiz];
    let mut e = raiz;
    while let Some(f) = sim.world().get::<ph2d_ecs::Children>(e).and_then(|c| {
        c.iter()
            .find(|c| sim.world().get::<ph2d_skeleton_ecs::Bone>(**c).is_some())
    }) {
        e = *f;
        out.push(e);
    }
    out
}

/// ⭐⭐ **O OSSO DO MEIO DE UMA CADEIA** — o que o roteiro do pincel de peso manda escolher.
///
/// ⛔⛔ **Ela é uma porta e não duas linhas no sítio onde é usada, e a razão é uma mutação
/// SOBREVIVENTE:** com a derivação inline, o gate que a julga tinha de a **copiar** — e uma cópia
/// julga a cópia. Trocar o índice para `.last()` no produto deixava o gate verde.
///
/// ⚠️ **Porque o MEIO e não a ponta:** medido por fotografia (2026-09-19) — da ponta, a parte
/// visível do braço pintado lê-se quase toda azul, porque a zona que ela governa sozinha cai atrás
/// do painel. O do meio tem território dos dois lados, e a rampa inteira cabe no enquadramento.
pub(super) fn osso_do_meio(
    sim: &ph2d_ecs::SimWorld,
    raiz: ph2d_ecs::Entity,
) -> Option<ph2d_ecs::Entity> {
    let cadeia = ph2d_skeleton_live::esqueletos::ossos_desde(sim, raiz);
    cadeia.get(cadeia.len() / 2).copied()
}
