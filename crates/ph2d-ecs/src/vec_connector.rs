//! **O conector** — a linha que gruda em duas formas e as segue.
//!
//! Espelha o padrão da *Live Shape* ([`crate::VecShape`]): o componente guarda a **relação**
//! (a quem cada ponta se prende, e como a rota é desenhada); a geometria é uma **função pura**
//! dela, re-cozida a cada frame pela shell. Consequência de graça: undo e save cobrem o
//! conector sem uma linha a mais, porque ambos capturam o mundo ECS + a cena vetorial.
//!
//! # A decisão que não é de gosto: o alvo é um `VecPathId`, nunca bits de entidade
//!
//! `Entity::to_bits()` é um id de **alocação**. O undo restaura o mundo **respawnando** as
//! entidades — com bits novos. (É por isso que o `WorldSnapshot` precisa de um
//! `canonicalize()` que ordena por conteúdo: os bits não são estáveis nem entre dois estados
//! logicamente iguais.)
//!
//! Guardar o alvo por bits, portanto, significa: o usuário desfaz qualquer coisa, e **todos os
//! conectores se soltam**. O `VecPathId` do documento vetorial, ao contrário, sobrevive ao
//! clone (undo) e ao postcard (save) — é a única identidade estável que existe hoje.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Onde uma ponta do conector se prende.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ConnectorEnd {
    /// **Solta**: um ponto em MUNDO. É a ponta que o usuário largou no vazio — e o destino de
    /// toda ponta cujo alvo foi apagado: ela **congela onde estava**, em vez de a linha sumir
    /// ou colapsar. (É o que o draw.io faz, e é o comportamento que não assusta.)
    Free { at: [f64; 2] },
    /// **Presa** a um objeto do documento (o `VecPathId`, cru — o ECS não conhece a cena
    /// vetorial, e não precisa: para ele é um `u64` opaco e estável).
    Bound { target: u64, anchor: Anchor },
}

/// Como a ponta acha o ponto de saída na forma.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    /// **Flutuante** (a âncora verde do draw.io, o *dynamic glue* do Visio): o lado de saída é
    /// **re-escolhido a cada frame**, em função de onde a outra forma está. Arraste a caixa
    /// para cima e a linha passa a sair pelo topo, sozinha. É o default, e o mais útil.
    Floating,
    /// **Porta fixa** (a âncora azul, o *static glue*, o *magnet* do Figma): um ponto
    /// **normalizado** na caixa LOCAL da forma. Local ⇒ gira e escala junto com ela
    /// (ADR-0111), de graça.
    Port { u: f32, v: f32 },
}

/// Como a rota é desenhada. **Append-only** (vai para o save).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[repr(u8)]
pub enum RouteKind {
    /// Reta, de borda a borda.
    Straight = 0,
    /// **Ortogonal** — o cotovelo do fluxograma. O default: é o que faz o desenho parecer um
    /// diagrama, e não um emaranhado de retas.
    #[default]
    Orthogonal = 1,
}

/// **O conector.** A entidade que o carrega também tem um [`crate::VecPathRef`]: o componente
/// é a relação, o `VecPath` é a geometria cozida a partir dela.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecConnector {
    pub start: ConnectorEnd,
    pub end: ConnectorEnd,
    pub route: RouteKind,
    /// Índice entre os conectores que ligam **o mesmo par** de formas. `0` = o único. Serve
    /// para dois conectores paralelos não se sobreporem — sem ele, o segundo desaparece
    /// exatamente por baixo do primeiro, e o usuário jura que ele não foi criado.
    pub parallel_index: u8,
}

impl SimComponent for VecConnector {}

impl VecConnector {
    /// Um conector novo entre dois objetos, com as duas âncoras flutuantes (o default).
    #[must_use]
    pub fn between(from: u64, to: u64) -> Self {
        Self {
            start: ConnectorEnd::Bound {
                target: from,
                anchor: Anchor::Floating,
            },
            end: ConnectorEnd::Bound {
                target: to,
                anchor: Anchor::Floating,
            },
            route: RouteKind::default(),
            parallel_index: 0,
        }
    }

    /// O objeto a que uma ponta se prende, se ela se prende a algum.
    #[must_use]
    pub fn target_of(end: &ConnectorEnd) -> Option<u64> {
        match end {
            ConnectorEnd::Bound { target, .. } => Some(*target),
            ConnectorEnd::Free { .. } => None,
        }
    }

    /// As duas pontas se prendem à MESMA forma? Aí não há rota a buscar: é um laço.
    #[must_use]
    pub fn is_self_loop(&self) -> bool {
        match (Self::target_of(&self.start), Self::target_of(&self.end)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    /// **A ponta cujo alvo sumiu vira SOLTA, no lugar onde estava.**
    ///
    /// Chamado quando um objeto é apagado. As alternativas seriam apagar o conector junto
    /// (destrutivo: o usuário perde uma ligação que talvez quisesse religar) ou deixá-lo
    /// apontando para um id morto (a linha voa para a origem, ou some). Congelar é o que o
    /// draw.io faz — e é o único que preserva o trabalho.
    ///
    /// `true` se alguma ponta foi congelada.
    pub fn freeze_missing(
        &mut self,
        alive: impl Fn(u64) -> bool,
        start_at: [f64; 2],
        end_at: [f64; 2],
    ) -> bool {
        let mut hit = false;
        for (end, at) in [(&mut self.start, start_at), (&mut self.end, end_at)] {
            if let ConnectorEnd::Bound { target, .. } = end
                && !alive(*target)
            {
                *end = ConnectorEnd::Free { at };
                hit = true;
            }
        }
        hit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_connector_between_two_shapes_floats_on_both_ends() {
        let c = VecConnector::between(7, 9);
        assert_eq!(VecConnector::target_of(&c.start), Some(7));
        assert_eq!(VecConnector::target_of(&c.end), Some(9));
        assert!(matches!(
            c.start,
            ConnectorEnd::Bound {
                anchor: Anchor::Floating,
                ..
            }
        ));
        assert!(!c.is_self_loop());
        assert!(VecConnector::between(7, 7).is_self_loop());
    }

    /// **Apagar uma forma não pode matar o conector.** A ponta órfã CONGELA onde estava — a
    /// linha continua na tela, largada, e o usuário pode religá-la. Apagá-la junto destruiria
    /// trabalho; deixá-la apontando para um id morto faria a linha voar para a origem.
    #[test]
    fn deleting_a_shape_freezes_the_orphaned_end_instead_of_killing_the_line() {
        let mut c = VecConnector::between(7, 9);
        // O objeto 9 foi apagado; ele estava em (10, 5).
        let changed = c.freeze_missing(|id| id != 9, [0.0, 0.0], [10.0, 5.0]);
        assert!(changed);
        assert_eq!(
            c.end,
            ConnectorEnd::Free { at: [10.0, 5.0] },
            "a ponta orfa tem de CONGELAR onde estava, nao sumir"
        );
        // A outra ponta não foi tocada.
        assert_eq!(VecConnector::target_of(&c.start), Some(7));
        // E congelar é idempotente: rodar de novo não muda nada.
        assert!(!c.freeze_missing(|id| id != 9, [0.0, 0.0], [99.0, 99.0]));
        assert_eq!(c.end, ConnectorEnd::Free { at: [10.0, 5.0] });
    }
}
