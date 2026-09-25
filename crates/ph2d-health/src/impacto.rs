//! ⭐⭐⭐ **A LEI DO IMPACTO** (plano 28, W5) — a PAUSA no golpe (*hitstop*) e o PISCAR da
//! invencibilidade. Pura, sem dependências, como o resto desta crate.
//!
//! # ⭐⭐ A pausa NÃO é estado da simulação — é o relógio do jogo a parar
//!
//! Um golpe que pesa congela o jogo inteiro umas centésimas de segundo (a pesquisa: `~0,05 s`, e
//! `~0,15 s` no golpe final). ⇒ a pausa **retém tempo de parede** antes de ele chegar ao acumulador
//! do passo fixo ([`Pausa::consome`]): durante ela nenhum tique corre — nem a física, nem os
//! relógios, nem as animações, nem a régua. *É o que um jogo de luta faz, e é por isso que ela não
//! precisa de ir para a fita nem para o anel:* um replay não tem tempo de parede, logo não há pausa
//! a reproduzir, e o que ele recalcula é exactamente a mesma corrida de tiques.
//!
//! # ⭐ O piscar é FUNÇÃO do tempo desde o golpe
//!
//! [`pisca_visivel`] lê o relógio que a vida já guarda ([`crate::Vida::desde_golpe_s`]) — nenhum
//! relógio novo, e por isso ele é exacto num scrub (o tique que se vê é o tique que a vida tem).

/// **A pausa no golpe** — quanto tempo de parede ainda falta reter.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pausa {
    resta_s: f64,
}

impl Pausa {
    /// Uma pausa parada.
    #[must_use]
    pub const fn new() -> Self {
        Self { resta_s: 0.0 }
    }

    /// **Pede uma pausa de `s` segundos.** ⭐ Pausas NÃO se somam — fica a MAIOR das que estão por
    /// cumprir: dez balas no mesmo instante são UM golpe que pesa, não dez vezes o congelamento (a
    /// lei dos jogos de luta). Um número não finito ou `≤ 0` é inerte.
    pub fn pede(&mut self, s: f64) {
        if s.is_finite() && s > 0.0 && s > self.resta_s {
            self.resta_s = s;
        }
    }

    /// **Consome o tempo de parede de um quadro** e devolve o que chega ao jogo. A parte retida é o
    /// que falta da pausa; o resto passa inteiro — ⚠️ a pausa não ARREDONDA a tiques, e é o
    /// acumulador do passo fixo a jusante que guarda a fracção (senão uma pausa de `0,05 s` a
    /// `144 Hz` roubava tiques a mais ou a menos conforme o arredondamento de cada quadro).
    #[must_use]
    pub fn consome(&mut self, wall_dt: f64) -> f64 {
        if !(wall_dt.is_finite() && wall_dt > 0.0) {
            return 0.0;
        }
        let retido = wall_dt.min(self.resta_s);
        self.resta_s -= retido;
        wall_dt - retido
    }

    /// Quanto falta.
    #[must_use]
    pub const fn resta_s(&self) -> f64 {
        self.resta_s
    }

    /// **Esquece a pausa** — rebobinar é renascer, e uma pausa pedida por um golpe da corrida
    /// anterior congelaria o arranque da seguinte.
    pub fn limpa(&mut self) {
        self.resta_s = 0.0;
    }
}

/// **O objecto está VISÍVEL neste instante do piscar?**
///
/// `desde_golpe_s` é o relógio da vida, `invencivel` diz se a janela ainda está aberta, e
/// `meio_periodo_s` é quanto dura cada metade do piscar (`≤ 0` = não pisca).
///
/// ⭐ **Começa VISÍVEL**: a primeira metade depois do golpe mostra o objecto, e é aí que o clarão
/// branco (o `Tween` no canal `Silhueta`, que a casa já tem) se vê — começar escondido comeria o
/// clarão.
#[must_use]
pub fn pisca_visivel(desde_golpe_s: f64, invencivel: bool, meio_periodo_s: f64) -> bool {
    if !invencivel || !(meio_periodo_s.is_finite() && meio_periodo_s > 0.0) {
        return true;
    }
    if !(desde_golpe_s.is_finite() && desde_golpe_s >= 0.0) {
        return true;
    }
    // ⚠️ `floor` e não um contador: é o que faz o piscar ser função do relógio (exacto num scrub).
    #[allow(clippy::cast_possible_truncation)]
    let metade = (desde_golpe_s / meio_periodo_s).floor() as i64;
    metade % 2 == 0
}

#[cfg(test)]
#[path = "impacto_tests.rs"]
mod tests;
