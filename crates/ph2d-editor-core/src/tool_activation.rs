//! ⭐⭐ **O QUE UM PEDIDO DE ACTIVAÇÃO DE FERRAMENTA FAZ** — a lei, e os seus dois chamadores.
//!
//! Um pedido chega com o id de uma ferramenta; a resposta depende de **que cluster** o manifesto
//! dela declara e de o modo de imagem estar ligado:
//!
//! - `image_tools` só se activam com o toggle IMG ligado (os pills só existem então, e o atalho
//!   `Digit3` tem de respeitar o mesmo);
//! - `vector_tools` · `motion_tools` · `flip_tools` activam-se sempre (o pill é activação directa)
//!   e **alternam**: pedir a que já está activa larga-a e volta à de omissão;
//! - um id que nenhum manifesto `Stateful` declara (o `move`, por exemplo) **não** se activa por
//!   aqui.
//!
//! # ⚠️ Por que mora aqui e não na shell
//!
//! Ela viveu inline no dreno do quadro (`fase_image_tool_activation`) enquanto o dreno era o único
//! leitor. O segundo leitor é o **arranque** ([`crate::screens::hero::layout_switch::install_at_startup`]):
//! o layout gravado tem de pegar a ferramenta dele ANTES do primeiro quadro, e com a MESMA lei —
//! uma cópia da lista de clusters ao lado da outra seria duas respostas à mesma pergunta, e a que
//! o artista vê é a que envelhece.

/// Os clusters cujos manifestos `Stateful` um pedido de activação pode pegar.
const CLUSTERS: [&str; 4] = ["image_tools", "vector_tools", "motion_tools", "flip_tools"];

/// **A resposta da lei a um pedido.** Ver o cabeçalho do módulo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationGate {
    /// O cluster do manifesto `Stateful` com este id — `None` se nenhum o declara.
    pub cluster: Option<&'static str>,
    /// O pedido pode activar a ferramenta agora.
    pub gate_on: bool,
    /// Pedir a ferramenta que JÁ está activa larga-a (o pill alterna). Os `image_tools` ficam de
    /// fora: quem manda neles é o toggle IMG.
    pub toggles_off: bool,
}

/// **A lei.** `image_mode_on` é o toggle IMG da barra do topo.
///
/// ⚠️ Sem registo de manifestos instalado (`crate::installed_registry`) nenhum id tem cluster, e a
/// resposta é *não activa* — o mesmo que o dreno respondia.
#[must_use]
pub fn activation_gate(tool_id: &str, image_mode_on: bool) -> ActivationGate {
    let cluster = crate::installed_registry().and_then(|reg| {
        CLUSTERS.into_iter().find(|&name| {
            reg.cluster(name).iter().any(|m| {
                m.id == tool_id
                    && matches!(m.handler, crate::registry::ToolHandler::Stateful { .. })
            })
        })
    });
    let gate_on = match cluster {
        Some("image_tools") => image_mode_on,
        Some("vector_tools" | "motion_tools" | "flip_tools") => true,
        _ => false,
    };
    ActivationGate {
        cluster,
        gate_on,
        toggles_off: matches!(
            cluster,
            Some("vector_tools" | "motion_tools" | "flip_tools")
        ),
    }
}
