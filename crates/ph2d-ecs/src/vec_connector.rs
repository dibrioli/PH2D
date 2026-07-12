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
    /// **Curva** — a MESMA rota ortogonal, suavizada num spline.
    ///
    /// Não é uma curva à parte, e essa é a decisão: uma cúbica única de A a B (o que o Visio
    /// faz) fica bonita entre duas caixas e **atravessa qualquer coisa no meio**, porque nunca
    /// passou pelo roteador. Aqui a busca A\* é a mesma, os obstáculos são os mesmos, e só a
    /// escrita da geometria muda — as dobras do cotovelo viram pontos de controle. O desvio de
    /// obstáculo sai de graça, e não há um segundo roteador para manter em pé.
    Curved = 2,
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
    /// **O jetty**: o quanto a linha avança reta antes de poder dobrar (o que a faz "sair para
    /// cima" antes de virar, em vez de dobrar colada na caixa).
    ///
    /// `None` = **automático**, derivado do tamanho das caixas — e é por isso que é um
    /// `Option` e não um `f32` com default: uma forma do PH2D tem ~1 unidade de mundo, mas
    /// nada impede uma de ter 50, e um jetty fixo ficaria invisível numa e gigante na outra.
    /// `Some(v)` = o usuário fixou o valor no painel, e ele para de acompanhar o tamanho.
    pub jetty: Option<f32>,
    /// **O spread**: o deslocamento do ponto de saída ao longo da face da forma, que separa
    /// dois conectores no mesmo par de caixas. `None` = automático (derivado do
    /// [`Self::parallel_index`]); `Some(v)` = fixado no painel.
    pub spread: Option<f32>,
    /// **O raio das quinas da rota** — as dobras do cotovelo, em unidades de mundo. `0` =
    /// afiado (o default do fluxograma clássico).
    ///
    /// Não é `Option` como os dois acima: não há um "raio automático" que faça sentido. Zero
    /// já é a resposta certa, e é a que o desenho técnico espera.
    pub corner_radius: f32,
    /// **Os pontos de passagem** — o escape manual, em MUNDO. Vazio = a rota é toda automática.
    ///
    /// O roteador é bom, e ainda assim vai escolher um caminho feio em algum diagrama: é a
    /// natureza de uma heurística. Sem uma saída manual, a única alternativa do usuário seria
    /// mover as formas até a linha se comportar — desenhar o diagrama em função do roteador, e
    /// não o contrário.
    ///
    /// A rota é então **costurada por trechos**: `início → w₁ → w₂ → … → fim`, cada trecho
    /// roteado pela mesma busca, com os mesmos obstáculos. Um waypoint não é um caso especial
    /// no roteador — é uma ponta SOLTA no meio do caminho, e o roteador já sabe o que é isso.
    ///
    /// **Em MUNDO, e não na régua de uma forma** (ao contrário do [`Anchor::Port`]): um ponto
    /// de passagem não pertence a forma nenhuma — ele é uma escolha sobre o ESPAÇO por onde a
    /// linha corre. Amarrá-lo a uma das duas caixas o faria migrar quando aquela caixa se
    /// mexesse, o que não é o que ninguém quer dizer ao fincar um ponto no meio da tela.
    #[serde(default)]
    pub waypoints: Vec<[f64; 2]>,
    /// **O braço dos handles da curva** (só a rota [`RouteKind::Curved`]): o quanto o controle da
    /// cúbica se afasta do ponto, em frações do segmento. `0` = a curva volta a ser a polilinha;
    /// `1/3` (o default) é o spline cardinal canônico — passa pelos pontos sem escapar deles;
    /// acima disso ela abre de propósito.
    ///
    /// É o "mais perto ou mais longe do ponto" do painel. Existe porque a curvatura certa é uma
    /// questão de gosto e de escala do diagrama, e nenhum número fixo acerta as duas.
    #[serde(default = "default_curve_arm")]
    pub curve_arm: f32,
}

/// O braço canônico do spline cardinal: **um terço do segmento** — o spline que passa pelos
/// pontos sem escapar deles.
pub const DEFAULT_CURVE_ARM: f32 = 1.0 / 3.0;

/// O default do [`VecConnector::curve_arm`]. Precisa ser uma função porque o default de `serde`
/// para um `f32` é ZERO — e braço zero é uma curva que não curva.
fn default_curve_arm() -> f32 {
    DEFAULT_CURVE_ARM
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
            jetty: None,
            spread: None,
            corner_radius: 0.0,
            waypoints: Vec::new(),
            curve_arm: default_curve_arm(),
        }
    }

    /// **As ESTAÇÕES da rota**: as duas pontas com os waypoints entre elas.
    ///
    /// Um waypoint entra como uma ponta **solta** (`ConnectorEnd::Free`) — e é essa a ideia
    /// toda. O roteador já sabe rotear de e para uma ponta solta; não há um caso especial de
    /// "waypoint" em lugar nenhum. A rota vira a costura dos trechos entre estações
    /// consecutivas.
    #[must_use]
    pub fn stations(&self) -> Vec<ConnectorEnd> {
        let mut out = Vec::with_capacity(self.waypoints.len() + 2);
        out.push(self.start);
        out.extend(self.waypoints.iter().map(|&at| ConnectorEnd::Free { at }));
        out.push(self.end);
        out
    }

    /// **O spread automático**, a partir do [`Self::parallel_index`]: `0, +1, −1, +2, −2 …`
    /// passos, em torno do centro da face.
    ///
    /// Alternar os lados (em vez de empilhar todos para o mesmo, que era o que o índice cru
    /// fazia) mantém o feixe **centrado**: com três conectores, o do meio sai reto e os outros
    /// dois o ladeiam. Empilhando, os três saíam tortos para o mesmo lado.
    #[must_use]
    pub fn auto_spread(index: u8, step: f64) -> f64 {
        let k = f64::from(u32::from(index).div_ceil(2)); // 0, 1, 1, 2, 2, …
        let sign = if index % 2 == 1 { 1.0 } else { -1.0 };
        sign * k * step
    }

    /// O spread efetivo: o do painel, se o usuário o fixou; senão o automático.
    #[must_use]
    pub fn spread_or(&self, step: f64) -> f64 {
        self.spread
            .map_or_else(|| Self::auto_spread(self.parallel_index, step), f64::from)
    }

    /// O jetty efetivo: o do painel, se o usuário o fixou; senão o `auto` que o chamador
    /// derivou do tamanho das caixas.
    #[must_use]
    pub fn jetty_or(&self, auto: f64) -> f64 {
        self.jetty.map_or(auto, f64::from)
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

    /// **O feixe de conectores paralelos nasce CENTRADO.** Os índices alternam os lados da
    /// face (`0, +1, −1, +2, −2 …`), então com três conectores o do meio sai reto e os outros
    /// dois o ladeiam. Empilhar todos para o mesmo lado — que é o que o índice cru fazia —
    /// deixaria os três tortos, e o primeiro (o caso comum!) já sairia fora do centro.
    #[test]
    fn parallel_connectors_fan_out_around_the_centre_of_the_face() {
        let s = |i| VecConnector::auto_spread(i, 0.5);
        assert_eq!(s(0), 0.0, "o unico conector sai pelo CENTRO da face");
        assert_eq!((s(1), s(2)), (0.5, -0.5), "o 2o e o 3o ladeiam o 1o");
        assert_eq!((s(3), s(4)), (1.0, -1.0));
        // A soma telescopa: o feixe é simétrico, logo seu centro de massa é o centro da face.
        let sum: f64 = (0..=6).map(s).sum();
        assert!(sum.abs() < 1e-12, "o feixe nao pode pender para um lado");
    }

    /// `None` = automático; `Some` = o valor que o usuário fixou no painel. É o que permite ao
    /// jetty acompanhar o tamanho da forma até o dia em que alguém decide o contrário.
    #[test]
    fn a_pinned_value_wins_over_the_automatic_one() {
        let mut c = VecConnector::between(1, 2);
        assert_eq!(c.jetty_or(0.3), 0.3, "sem pino, vale o automatico");
        assert_eq!(c.spread_or(0.5), 0.0);
        // Valores exatos em binário: um `-0.2f32` alargado para `f64` NÃO é `-0.2f64`, e o
        // teste falharia por causa do alargamento, não do código.
        c.jetty = Some(0.75);
        c.spread = Some(-0.25);
        assert_eq!(c.jetty_or(0.3), 0.75);
        assert_eq!(c.spread_or(0.5), -0.25);
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
