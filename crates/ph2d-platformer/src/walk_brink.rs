//! **A TRAVA DE BEIRADA** (`W-Brink`) — o `bCanWalkOffLedges` do Unreal, e as
//! duas perguntas que ela faz.
//!
//! ⚠️ **Corte por RESPONSABILIDADE, e a linha é real:** o pai [`super`] responde
//! *quanto empurrar*, e isto responde *até onde é seguro andar*. A trava mexe no
//! ALVO da caminhada e em mais nada — o `clamp_target` mora na
//! [`crate::Brink`], que é o SENTIDO —, então o que sobra aqui é o alcance e a
//! porta do custo.

use super::WalkConfig;

impl WalkConfig {
    /// **Quão à frente a perna pergunta pelo chão** (`W-Brink`), metros — a
    /// distância que o sensor da trava desloca antes de castar.
    ///
    /// # ⚠️ Ela é DERIVADA, e o knob que ela substituiu foi medido a morrer
    ///
    /// O primeiro corte desta wave a autorava. A sonda mediu o preço: a 8 m/s um
    /// alcance de `0,30` deixa o personagem **CAIR** e um de `0,60` o segura, e
    /// a fronteira entre os dois é exactamente `v²/(2·a) = 0,533`. Ou seja *o
    /// valor certo do knob era função de OUTROS DOIS knobs* — a forma que este
    /// repo já removeu uma vez (o Conserve do sculpt, cujo slider correto valia
    /// 168% no centro de outro controle). Um artista que subisse a **Speed**
    /// veria o personagem voltar a cair de patamares, sem nada na tela a
    /// explicar.
    ///
    /// ⚠️ **Ela é METADE do alcance, e a outra metade é GEOMETRIA que esta crate
    /// não conhece:** quem casta soma a **meia-largura do corpo**, porque a
    /// pergunta certa é *"quando eu parar, ainda haverá chão onde a minha BORDA
    /// estiver?"* — e a borda vive meia-largura à frente do centro. Sem essa
    /// parcela o alcance é exactamente a distância de paragem, ou seja **o caso
    /// de fronteira**: o personagem trava no instante em que a perna deixa de o
    /// segurar. Medido a 2 m/s, ele acabava equilibrado num pé só sobre o lábio
    /// **e caía na mesma** — e as outras velocidades escapavam por um fio, pelo
    /// bónus de mudança de direção. Cada metade mora com quem a sabe.
    ///
    /// ⚠️ **Da velocidade AUTORADA, nunca da viva** — a autorada é constante no
    /// regime, então a beirada fica onde está; uma derivada do passo VIVO
    /// deslocaria o ponto de parada a cada oscilação de velocidade, e o artista
    /// leria como *"a quina muda de sítio"*.
    ///
    /// ⚠️ **CONSERVADORA de propósito, e por duas vias:** o fator de mudança de
    /// direção ([`Self::MAX_TURN_BOOST`]) dá até **2×** de orçamento quando o
    /// alvo está longe, então a travagem real é mais curta que esta conta; e o
    /// `brake_scale` **não entra** porque ele só age com o eixo SOLTO, e quem
    /// anda para uma beirada está a empurrar.
    ///
    /// ⚠️ **O `grip` também NÃO entra**, e é a limitação honesta: no gelo a
    /// travagem é mais longa que esta distância e o personagem pode escorregar
    /// para fora. É o que gelo significa — e pô-lo aqui devolveria a beirada
    /// móvel que o parágrafo acima recusa.
    ///
    /// Zero quando não há o que derivar (`speed` ou `acceleration` não
    /// positivos): sem passo não há para onde andar para fora.
    #[must_use]
    pub fn ledge_look(&self) -> f32 {
        if !self.speed.is_finite()
            || self.speed <= 0.0
            || !self.acceleration.is_finite()
            || self.acceleration <= 0.0
        {
            return 0.0;
        }
        self.speed * self.speed / (2.0 * self.acceleration)
    }

    /// O teto do fator de mudança de direção — 2×, o do `bevy-tnua` num 180°.
    pub const MAX_TURN_BOOST: f32 = 2.0;

    /// O cosseno do limite de rampa.
    ///
    /// ⚠️ **Por `libm`, nunca pelo `std`**: este número entra no `physics_ecs_c9`
    /// pela W7, e a lei de determinismo do módulo é que 1 ulp de diferença entre
    /// dois sistemas operacionais é um bug, não ruído.
    #[must_use]
    pub fn max_slope_cos(&self) -> f32 {
        libm::cosf(self.max_slope_deg.clamp(0.0, 90.0) * core::f32::consts::PI / 180.0)
    }
}

/// **Vale a pena perguntar ao mundo onde a beirada está?** (`W-Brink`)
///
/// ⚠️ **Porta única, pelo motivo do irmão [`crate::corner_probe_wanted`]:** a
/// ponte pergunta-a para decidir se casta a perna outra vez, e o gate de custo
/// pergunta-a para provar que uma cena que não arma a trava não paga um raio.
/// Duas cópias divergiriam no dia em que uma ganhasse um caso especial, e o
/// sintoma seria a trava ora existir ora não, sem nada na tela a explicar.
///
/// - Com a trava **armada**, porque quem deixa andar para fora não precisa de
///   saber onde a beirada está.
/// - Com **alcance positivo**, porque zero deixaria o sensor a perguntar pelo
///   chão debaixo dos próprios pés — que a perna já respondeu.
/// - Com o dedo a **empurrar**, porque a trava só sabe cortar um alvo, e o alvo
///   de quem não empurra já é zero.
/// - **No chão**, porque no ar não há patamar de que se ande para fora.
#[must_use]
pub fn brink_probe_wanted(cfg: &WalkConfig, grounded: bool, drive: f32) -> bool {
    !cfg.walk_off_ledges && cfg.ledge_look() > 0.0 && grounded && drive != 0.0 && drive.is_finite()
}
