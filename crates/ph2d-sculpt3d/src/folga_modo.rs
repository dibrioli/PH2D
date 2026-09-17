//! ⭐⭐⭐ **COMO A FOLGA DO *PROJECTAR NA CENA* ENTRA NA DISTÂNCIA** — irmão
//! (`#[path]`) do [`crate::SmearMode`] e do [`crate::TrimForma`], cortado pelo
//! mesmo assunto: *o selector é a lei, não um argumento dela*.
//!
//! ⚠️⚠️ **Isto existe porque as DUAS leis estão certas, cada uma num caso** — e
//! o dono decidiu (17/09) que *«cada modo com opção, com um botão para mudar o
//! modo»* em vez de uma escolher por ele.
//!
//! A folga é subtraída de um `d` **COM SINAL**, e daí nascem as duas leituras:
//!
//! | situação | [`FolgaModo::DoAlvo`] | [`FolgaModo::Simetrica`] |
//! |---|---|---|
//! | vão `+0,5`, folga `0,1` | `+0,4` — pára a `0,1` do alvo | `+0,4` — igual |
//! | vão `+0,5`, folga `0,6` | **`−0,1`** — o barro **AFASTA-SE** | **`0,0`** — não anda |
//! | vão `−0,5` (atrás), folga `0,1` | **`−0,6`** — **ULTRAPASSA** o alvo | **`−0,4`** — pára a `0,1` |
//!
//! ⭐ **Sob o rótulo *«distância mínima»* a 1.ª linha da 2.ª coluna é DEFENSÁVEL**
//! (pedir `0,6` de folga com o alvo a `0,5` só se cumpre recuando `0,1`), e a
//! 2.ª **não é**: um acerto para trás faz a folga **crescer** a excursão em vez
//! de a travar, e o barro passa para o outro lado da peça que ele devia tocar.
//! ⇒ *reproduzir o alvo reproduz um defeito, e divergir quebra a memória
//! muscular de quem vem dele* — as duas frases que o põem, e é por isso que a
//! resposta é um **botão** e não um veredito meu.
//!
//! ⛔⛔ **A ORDEM contra a escolha do vencedor é load-bearing e NÃO é desta
//! porta:** a folga entra **depois** de o candidato mais perto estar escolhido
//! (ver [`crate::projectar::distancia`]). Subtraí-la antes faria a competição
//! ser entre números já encolhidos, e um alvo longe podia ganhar de um perto
//! por causa de uma folga que é a mesma para os dois. *Esta porta recebe o
//! vencedor, nunca os candidatos.*

/// **Como a folga entra na distância** — ver o cabeçalho.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FolgaModo {
    /// **A do alvo:** `d − folga`, sobre um `d` com sinal.
    ///
    /// ⚠️ É o valor de FÁBRICA, e a razão é o corpus: as `14` fixturas vivas do
    /// oráculo foram gravadas com esta lei, logo é ela que a paridade mede. ⛔
    /// Trocar o default tornaria a bancada uma medição de outro pincel — que é
    /// exactamente o que aconteceu quando o espaçamento do afiado mudou de dono
    /// (seis gates de paridade vermelhos de uma vez).
    #[default]
    DoAlvo,
    /// **Simétrica:** `sign(d) · max(0, |d| − folga)`.
    ///
    /// Trava nos **dois** sentidos e **nunca inverte** o sentido do movimento:
    /// uma folga maior que o vão deixa o vértice **quieto** em vez de o afastar,
    /// e um acerto para trás pára **antes** do alvo em vez de o ultrapassar.
    Simetrica,
}

impl FolgaModo {
    /// Os dois, na ordem em que o painel os pinta — o de fábrica primeiro.
    pub const ALL: [Self; 2] = [Self::DoAlvo, Self::Simetrica];

    /// O rótulo do chip (a UI da casa é inglesa).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::DoAlvo => "Signed",
            Self::Simetrica => "Symmetric",
        }
    }

    /// **A PORTA ÚNICA** — aplica a folga ao `d` do vencedor.
    ///
    /// ⚠️ **Com `folga = 0` as duas leis são a IDENTIDADE, ao bit**, e é isso
    /// que torna o modo invisível no ponto neutro: `d − 0 = d` e
    /// `sign(d)·max(0, |d| − 0) = d` para todo `d` finito, incluindo o `−0,0`
    /// (cujo `signum` é `−1` e cujo `abs` é `0,0` ⇒ `−0,0`). *Um modo que
    /// mudasse alguma coisa no neutro não seria um modo, seria um segundo
    /// produto.*
    #[must_use]
    pub fn aplica(self, d: f32, folga: f32) -> f32 {
        match self {
            Self::DoAlvo => d - folga,
            Self::Simetrica => d.signum() * (d.abs() - folga).max(0.0),
        }
    }
}

#[cfg(test)]
#[path = "folga_modo_tests.rs"]
mod tests;
