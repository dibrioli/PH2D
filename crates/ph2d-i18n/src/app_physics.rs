//! **O QUE A FAMÍLIA app.physics DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-physics` mostra (da FÍSICA (desenhar uma junta, o leitor de carga, o cartão do corpo)), na forma `app.physics.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): o
//! diagnóstico de CONSOLA, as cenas de smoke e os **nomes por omissão de objecto** — um nome que
//! entra no `Name` é identidade durável (`stable_name_id` fecha um hash sobre ele), e traduzi-lo
//! é decisão do dono, não desta migração.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.physics.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.physics.dispatch.broke_at_n" => "{name} broke at {force_0} N",
        "app.physics.body.the_body_above" => "the body above",
        "app.physics.joint_draw.a_pulley_needs_two_bodies_its_rope_pulls_at_both" => {
            "A pulley needs two bodies — its rope pulls at both ends"
        }
        "app.physics.joint_draw.those_two_bodies_cannot_be_joined" => {
            "Those two bodies cannot be joined"
        }
        "app.physics.joint_draw.start_or_end_the_joint_on_a_body" => {
            "Start or end the joint ON a body"
        }
        "app.physics.joint_draw.a_joint_binds_two_different_bodies" => {
            "A joint binds two DIFFERENT bodies"
        }
        "app.physics.joint_draw.joint_drawing_cancelled" => "Joint drawing cancelled",
        "app.physics.joint_readout.max" => "max {force}",
        "app.physics.joint_readout.no_route" => "no route",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
