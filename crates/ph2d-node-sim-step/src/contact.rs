//! ⭐⭐ **O PASSO RESOLVE O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13):
//! *«colidem sozinhas»*; e desde o doc 121 §9.20 (06/10, *«usar o motor da casa»*) pelo `rapier2d`
//! da Física ([`ph2d_contact_world`]).
//!
//! Todo motor faz num `step` duas coisas: integra e resolve contatos. Este nó integra a metade da
//! VELOCIDADE (a força, o amortecimento, o limite, o giro amortecido); as peças que declaram colisor
//! (as colunas `collider` / `collider_box`, escritas por quem desenha) vão para o mundo de contacto,
//! que resolve os contactos e integra a POSIÇÃO e o ângulo delas — uma integração só. Sem colisor
//! nada aqui corre, e o passo é o de sempre.
//!
//! O que o dono aprovou e continua de pé, agora pelo mundo (doc 109–111): o colisor é o que a peça
//! DECLAROU (disco ou caixa, escalado pelo `size`, girado pelo `rot`, com `Offset`); a rotação está
//! destravada e o `Lock Rotation` (`inv_inertia = 0`) trava-a; o material é `Friction` (`√(a·b)`),
//! `Bounce` (`max`) e `Rolling` (a nossa lei, depois do passo — o rapier não a tem); a taça do
//! `sim.collide` é um colisor FIXO do mesmo mundo. ⛔ A lei por colunas que aqui esteve
//! (`ph2d_contact::separate` + `impulsos`, `8` sub-passos) saiu: a mesma pilha custava `20`–`30×`
//! (o oráculo, doc 121 §9.19).

/// O passo do mundo de contacto para este tique — `None` quando ninguém declara colisor (e a
/// memória some com o colisor).
pub(crate) fn resolve(
    pedido: &ph2d_contact_world::Pedido<'_>,
    estado: &mut ph2d_contact_world::Estado<'_>,
    mundo: &mut Option<ph2d_contact_world::Mundo>,
) -> Option<ph2d_contact_world::Feito> {
    ph2d_contact_world::passo(mundo, pedido, estado)
}

#[cfg(test)]
#[path = "contact_tests.rs"]
mod contact_tests;

/// As SONDAS — irmã dos gates pelo tecto de LOC e por responsabilidade; ver o cabeçalho delas.
#[cfg(test)]
#[path = "contact_probes.rs"]
mod contact_probes;
