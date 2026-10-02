//! ⭐⭐⭐ **O VOCABULÁRIO DA NAVEGAÇÃO** (plano 30, W4) — o instantâneo e a edição das secções NAV
//! REGION e NAV AGENT, num módulo abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! ⚠️ **Duas secções e UM vocabulário**, pelo precedente do [`crate::vida_edits`]: um objecto pode
//! ter as duas (uma arena que é também o agente, raro mas legal), e uma edição é sempre de um campo
//! de um dos dois componentes — duas enums partiriam o despacho em dois sem ganhar nada.
//!
//! # ⭐⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! O agente **PEDE** e o mover **ANDA** (plano 30 §2.1): a navegação escreve a intenção no canal do
//! `TopDownPlayer`, e a ponte SALTA quem não tem um mover que a ouça. ⇒ *«pus o agente e ele não
//! anda»* tem OITO causas, e cada uma tem uma cura diferente:
//!
//! | queixa | o que se passa | cura |
//! |---|---|---|
//! | `SemCorpo` | a ponte só conduz quem tem corpo | um `RigidBody` |
//! | `SemMover` | ninguém anda pelo agente | um `TopDownPlayer` |
//! | `MoverLeTeclado` | o mover ouve as setas, e os dois falariam ao mesmo tempo | desligar *Default Controls* |
//! | `ComPlataforma` | um `PlatformPlayer` ganha a pose | tirar um dos dois |
//! | `Desligado` | o artista desligou-o | ligar *Active* |
//! | `SemAlvo` | ele não vai a lado nenhum | escolher um alvo |
//! | `AlvoPerdido` | ninguém tem esse nome | escrever um nome que exista |
//! | `SemForma` | (patrulha) nenhuma forma desenhada tem esse nome | escrever o nome de uma forma |
//! | `ForaDaRegiao` | não há malha onde ele está | uma `NavRegion` à volta dele |
//!
//! ⚠️ **Da mais ESPECÍFICA para a mais geral** — a lei da recusa dos pincéis: dizer *«sem alvo»* a
//! quem não tem mover é mandá-lo resolver a metade errada. As quatro primeiras são as mesmas
//! perguntas que a ponte faz para decidir se conduz (`bridge::nav`), e por isso a ordem não é estilo.
//!
//! # ⭐⭐ E a leitura VIVA é o que faz a secção valer a pena
//!
//! *«Moving · 3,20 m to go»* não vem de campo nenhum: vem do `NavNow` que a ponte publica no mundo.
//! E o raio DERIVADO do colisor (o caso comum) só existe ali — sem ele o `0` do campo é um mistério.

/// Snapshot das secções NAV REGION e NAV AGENT da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorNavInfo {
    pub entity_bits: u64,
    /// A secção NAV REGION — `None` se a entidade não tem a região.
    pub region: Option<InspectorNavRegion>,
    /// A secção NAV AGENT — `None` se a entidade não tem o agente.
    pub agent: Option<InspectorNavAgent>,
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// Os campos da região.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InspectorNavRegion {
    /// Meia-largura, em metros.
    pub half_w: f32,
    /// Meia-altura, em metros.
    pub half_h: f32,
    /// As camadas cujos corpos estáticos bloqueiam (bit `i` = camada `i`).
    pub obstacle_layers: u8,
}

impl InspectorNavRegion {
    /// ⭐ **A queixa da região** — `None` quando não há nenhuma.
    #[must_use]
    pub fn queixa(&self) -> Option<RegionQueixa> {
        if self.half_w <= 0.0 || self.half_h <= 0.0 {
            return Some(RegionQueixa::SemTamanho);
        }
        if self.obstacle_layers == 0 {
            return Some(RegionQueixa::SemParedes);
        }
        None
    }
}

/// Por que uma região pode não fazer o que o artista espera.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RegionQueixa {
    /// Uma das meias-extensões é zero — a malha seria vazia, e nenhum agente lá anda.
    SemTamanho,
    /// Nenhuma camada bloqueia: as paredes não são paredes para os agentes.
    SemParedes,
}

/// Para onde o agente vai — o segmentado do painel.
///
/// ⚠️ **A POSIÇÃO em [`NavAlvoModo::ALL`] é a tag do clique**, como em todo segmentado desta casa.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NavAlvoModo {
    /// Sem alvo — fica parado.
    Nenhum,
    /// Um objecto, pelo NOME.
    Objecto,
    /// Um ponto fixo do mundo.
    Ponto,
    /// (W6) O mais perto que pertence a uma TAG.
    Tag,
    /// (W6) A PATRULHA pelos pontos de uma forma desenhada, pelo NOME dela.
    Patrulha,
}

impl NavAlvoModo {
    /// Os cinco, na ordem do segmentado. ⚠️ Apendados no fim: a posição é a tag do clique.
    pub const ALL: [Self; 5] = [
        Self::Nenhum,
        Self::Objecto,
        Self::Ponto,
        Self::Tag,
        Self::Patrulha,
    ];
}

/// O estado da condução, como o painel o lê.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NavEstado {
    /// Sem alvo, ou desligado.
    Parado,
    /// A seguir um caminho até ao alvo.
    AAndar,
    /// A seguir o caminho até ao ponto mais perto de um alvo que não se alcança.
    Parcial,
    /// No alvo.
    Chegou,
    /// Não há caminho nenhum.
    SemCaminho,
}

/// A leitura VIVA de um agente — `None` no snapshot antes do 1.º tique ou quando a ponte o salta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NavAgora {
    pub estado: NavEstado,
    /// O que falta andar, em metros.
    pub restante: f32,
    /// O raio com que ele procura o caminho — o autorado, ou o DERIVADO do colisor.
    pub raio: f32,
}

/// Os campos do agente, mais as perguntas que decidem se a ponte o conduz.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorNavAgent {
    pub alvo_modo: NavAlvoModo,
    /// O nome do alvo (modo `Objecto`) ou da forma (modo `Patrulha`) — vazio quando ninguém tem o
    /// id guardado.
    pub alvo_nome: String,
    /// O id guardado não é o nome de ninguém (o alvo foi apagado ou renomeado) — na `Patrulha`, de
    /// nenhuma forma DESENHADA.
    pub alvo_perdido: bool,
    /// (W6) A tag do modo `Tag` (`0` = por escolher).
    pub alvo_tag: u64,
    /// O ponto do alvo (modo `Ponto`), em metros.
    pub alvo_ponto: [f32; 2],
    /// O raio autorado — ⚠️ `0` é DERIVADO do colisor.
    pub radius: f32,
    pub arrive: f32,
    pub repath: f32,
    pub stuck_after: f32,
    pub active: bool,
    /// Desvia dos outros corpos que andam (plano 30, W5).
    pub avoidance: bool,
    pub on_arrived: String,
    pub on_no_path: String,
    pub on_stuck: String,
    pub has_body: bool,
    pub has_mover: bool,
    /// O `TopDownPlayer` ouve o teclado (`default_controls`).
    pub mover_reads_keys: bool,
    pub has_platformer: bool,
    /// Alguma `NavRegion` contém o agente.
    pub in_region: bool,
    pub agora: Option<NavAgora>,
}

impl InspectorNavAgent {
    /// ⭐⭐⭐ **A queixa, da mais ESPECÍFICA para a mais geral** — ver o cabeçalho do módulo.
    ///
    /// ⚠️ **Uma PORTA e não oito `if` no pintor**: o painel pinta a frase e o gate mede-a sem pintar
    /// nada (a lei do `InspectorRayInfo::queixa`).
    #[must_use]
    pub fn queixa(&self) -> Option<AgentQueixa> {
        if !self.has_body {
            return Some(AgentQueixa::SemCorpo);
        }
        if !self.has_mover {
            return Some(AgentQueixa::SemMover);
        }
        if self.mover_reads_keys {
            return Some(AgentQueixa::MoverLeTeclado);
        }
        if self.has_platformer {
            return Some(AgentQueixa::ComPlataforma);
        }
        if !self.active {
            return Some(AgentQueixa::Desligado);
        }
        match self.alvo_modo {
            NavAlvoModo::Nenhum => return Some(AgentQueixa::SemAlvo),
            NavAlvoModo::Objecto if self.alvo_perdido => return Some(AgentQueixa::AlvoPerdido),
            NavAlvoModo::Objecto if self.alvo_nome.is_empty() => {
                return Some(AgentQueixa::SemAlvo);
            }
            NavAlvoModo::Patrulha if self.alvo_perdido => return Some(AgentQueixa::SemForma),
            NavAlvoModo::Patrulha if self.alvo_nome.is_empty() => {
                return Some(AgentQueixa::SemAlvo);
            }
            NavAlvoModo::Tag if self.alvo_tag == 0 => return Some(AgentQueixa::SemAlvo),
            _ => {}
        }
        if !self.in_region {
            return Some(AgentQueixa::ForaDaRegiao);
        }
        None
    }
}

/// As oito razões pelas quais um agente pode não andar — ver o cabeçalho do módulo.
///
/// ⛔ **Um enum e não uma chave de i18n** — a lei não conhece a língua (o `RayQueixa` pagou-a).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AgentQueixa {
    SemCorpo,
    SemMover,
    MoverLeTeclado,
    ComPlataforma,
    Desligado,
    SemAlvo,
    AlvoPerdido,
    ForaDaRegiao,
    /// (W6) Nenhuma forma desenhada tem o nome da patrulha.
    SemForma,
}

/// Uma edição de um campo das secções NAV REGION ou NAV AGENT.
#[derive(Clone, Debug, PartialEq)]
pub enum NavFieldEdit {
    HalfW(f32),
    HalfH(f32),
    /// ⚠️ A MÁSCARA inteira, já com o bit trocado — quem a calcula é o despacho, a partir do
    /// SNAPSHOT (nunca do store).
    ObstacleLayers(u8),
    AlvoModo(NavAlvoModo),
    /// ⚠️ O NOME cru — quem apara é quem lê (a regra do alvo do projéctil).
    AlvoNome(String),
    AlvoX(f32),
    AlvoY(f32),
    Radius(f32),
    Arrive(f32),
    Repath(f32),
    StuckAfter(f32),
    Active(bool),
    OnArrived(String),
    OnNoPath(String),
    OnStuck(String),
    /// (W5) Desviar dos outros — apendado.
    Avoidance(bool),
    /// (W6) A tag do modo `Tag` — apendado.
    AlvoTag(u64),
}

#[cfg(test)]
#[path = "nav_edits_tests.rs"]
mod tests;
