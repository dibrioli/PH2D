//! ⭐⭐⭐ **O VOCABULÁRIO DA NAVEGAÇÃO** (plano 30, W4) — o instantâneo e a edição das secções NAV
//! REGION e NAV AGENT, num módulo abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! ⭐ (W7) **Mais duas secções no MESMO vocabulário**: NAV COST AREA (a lama, a zona proibida) e NAV
//! LINK (o teleporte, a porta de um sentido) — e o agente ganha *Avoid Harm*.
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
//! | `ParadoPorAccao` | um `Stop Navigation` parou-o nesta corrida | um `Start`, ou recomeçar |
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
    /// (W7) A secção NAV COST AREA — `None` se a entidade não tem a área.
    pub cost_area: Option<InspectorNavCostArea>,
    /// (W7) A secção NAV LINK — `None` se a entidade não tem o atalho.
    pub link: Option<InspectorNavLink>,
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
    /// ⭐ A DAR PASSAGEM: outro corpo cortou-lhe o pedido e o desvio tirou-lhe mais de metade da
    /// rapidez pelo caminho. Sem isto um agente travado pela multidão lia-se *Moving*.
    pub dando_passagem: bool,
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
    /// (W7) Evita as zonas que o FEREM — um `Damage` parado que o `Health` dele sente.
    pub avoid_harm: bool,
    /// (W7) Tem um `Health` — sem ele nada o fere, e o *Avoid Harm* não muda nada (o painel diz).
    pub has_health: bool,
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
    /// ⭐ A ORDEM de um verbo nesta corrida (`None` = nenhum falou; vale o *Active*) — ela manda
    /// mais que o autorado, e a queixa lê-a: sem ela o painel dizia *«Switched off»* de um agente
    /// desligado que um `Start` pôs a andar.
    pub ordem: Option<bool>,
    /// O nome do alvo que um `Start` lhe deu — vazio = o autorado.
    pub alvo_da_ordem: String,
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
        match self.ordem {
            Some(false) => return Some(AgentQueixa::ParadoPorAccao),
            None if !self.active => return Some(AgentQueixa::Desligado),
            _ => {}
        }
        // ⚠️ Um `Start` com nome deu-lhe OUTRO alvo: as faltas do autorado não são as de agora.
        if self.ordem == Some(true) && !self.alvo_da_ordem.is_empty() {
            return (!self.in_region).then_some(AgentQueixa::ForaDaRegiao);
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

    /// ⭐ **Um `Start` pô-lo a andar CONTRA o autorado** (a caixa *Active* desmarcada, ou outro alvo)
    /// — `Some(nome do alvo da ordem)`, vazio quando o alvo é o autorado. O painel di-lo: a caixa e o
    /// campo do alvo mostram o que o artista escreveu, não o que corre.
    #[must_use]
    pub fn posto_a_andar_por_accao(&self) -> Option<&str> {
        (self.ordem == Some(true) && (!self.active || !self.alvo_da_ordem.is_empty()))
            .then_some(self.alvo_da_ordem.as_str())
    }

    /// ⭐ (W7) *Avoid Harm* ligado num agente SEM `Health`: nada o fere, logo ele não evita nada —
    /// a caixa marcada não muda o caminho, e o painel tem de o DIZER (a ponte lê o `Health`).
    #[must_use]
    pub fn evitar_dano_nao_muda_nada(&self) -> bool {
        self.avoid_harm && !self.has_health
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
    /// Um `Stop Navigation` parou-o nesta corrida (um `Start` volta a pô-lo a andar).
    ParadoPorAccao,
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
    /// (W7) Evitar as zonas que o ferem — apendado.
    AvoidHarm(bool),
    /// (W7) O custo da área, em múltiplos do chão — quem aplica põe o piso [`NAV_AREA_COST_MIN`].
    CostAreaCost(f32),
    /// (W7) A área proibida (um furo para todos).
    CostAreaForbidden(bool),
    /// (W7) O NOME cru da saída do atalho — quem apara e resolve é quem lê (a regra do `AlvoNome`).
    LinkTo(String),
    LinkTwoWay(bool),
    LinkTeleport(bool),
    /// (W7) O custo a mais de atravessar, em metros de chão (`>= 0`).
    LinkCost(f32),
    LinkOnCrossed(String),
}

/// ⭐ (W7) **O piso do custo de uma área** — a lei só pede `> 0` (o custo multiplica um comprimento);
/// `0,01` = o chão cem vezes mais barato, a estrada que o agente prefere sempre. O painel e o dreno
/// leem o MESMO número.
pub const NAV_AREA_COST_MIN: f32 = 0.01;

/// ⭐ (W7) **Os campos da ÁREA DE CUSTO**, mais a pergunta que decide se ela faz alguma coisa.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InspectorNavCostArea {
    pub cost: f32,
    pub forbidden: bool,
    /// Tem `RigidBody` e `Collider` — a forma da área é o colisor, e sem ele a ponte não a vê.
    pub has_shape: bool,
    /// O corpo é `Dynamic` — só um corpo parado (estático, ou cinemático quieto) recorta a malha.
    pub body_moves: bool,
    /// (W19) O raio do menor corpo que não cabe nela (`NavCostAreaNow`, da ponte) — `None` = cabe.
    pub too_narrow_for: Option<f32>,
}

impl InspectorNavCostArea {
    /// A queixa da área — `None` quando ela recorta a malha. Da mais específica para a mais geral.
    #[must_use]
    pub fn queixa(&self) -> Option<CostAreaQueixa> {
        if !self.has_shape {
            return Some(CostAreaQueixa::SemForma);
        }
        if self.body_moves {
            return Some(CostAreaQueixa::CorpoQueAnda);
        }
        // Só uma BARATA recua para dentro (uma cara estreita continua a valer: o corpo paga-a ao tocar).
        if !self.forbidden && self.cost < 1.0 && self.too_narrow_for.is_some() {
            return Some(CostAreaQueixa::MaisEstreitaQueOCorpo);
        }
        None
    }
}

/// Por que uma área de custo pode não fazer nada.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CostAreaQueixa {
    /// Sem `RigidBody` + `Collider`: a área não tem forma.
    SemForma,
    /// Um corpo `Dynamic` nunca é um obstáculo parado — a ponte salta-o.
    CorpoQueAnda,
    /// (W19) Mais BARATA que o chão e mais estreita que o corpo de quem anda: nenhum corpo cabe inteiro
    /// nela, e ela não muda caminho nenhum (o raio em [`InspectorNavCostArea::too_narrow_for`]).
    MaisEstreitaQueOCorpo,
}

/// ⭐ (W7) **Os campos do ATALHO** — a entrada é esta entidade; a saída, quem tem o nome.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorNavLink {
    /// O nome da saída — vazio quando ninguém tem o id guardado.
    pub to_nome: String,
    /// O id guardado não é o nome de ninguém (a saída foi apagada ou renomeada).
    pub to_perdido: bool,
    pub two_way: bool,
    pub teleport: bool,
    pub cost: f32,
    pub on_crossed: String,
}

impl InspectorNavLink {
    /// A queixa do atalho — o perdido vem antes do vazio (a lei do alvo `Objecto`).
    #[must_use]
    pub fn queixa(&self) -> Option<LinkQueixa> {
        if self.to_perdido {
            return Some(LinkQueixa::SaidaPerdida);
        }
        if self.to_nome.is_empty() {
            return Some(LinkQueixa::SemSaida);
        }
        None
    }
}

/// Por que um atalho pode não levar a lado nenhum — a ponte salta-o nos dois casos.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LinkQueixa {
    /// Nenhuma saída escrita.
    SemSaida,
    /// Ninguém tem esse nome.
    SaidaPerdida,
}

#[cfg(test)]
#[path = "nav_edits_tests.rs"]
mod tests;
