//! ⭐⭐ **Os TIPOS de dano: a resistência de uma vida a um tipo, e o dano que DURA** (plano 28, W6).
//!
//! # De onde vem a lei
//!
//! O GDevelop (o oráculo das W0–W5) **não tem tipos**. A referência de licença aberta que os tem é o
//! addon *Health, HitBoxes, HurtBoxes* do Godot (cluttered-code, **MIT**), e ele foi **CORRIDO** sem
//! interface sobre casos nossos (`docs/Components/ferramentas/godot_health_tipos/`, `21` casos). O
//! que ele decidiu e esta lei copia:
//!
//! | caso | o alvo | a casa |
//! |---|---|---|
//! | taxa `0` | dano `0` — **imune** | igual |
//! | taxa `0,5` / `2` | metade / dobro | igual |
//! | taxa **negativa** | dano `0` (⚠️ **não cura**) | igual — [`Taxa::fator`] |
//! | **absorver** | um interruptor À PARTE (`convert_affect = HEAL`): o dano vira cura, `× taxa` | igual — [`Taxa::absorve`] |
//! | absorver com a vida CHEIA | nada (nem o sinal de cura) | igual — a porta da cura já o faz |
//! | dois modificadores do mesmo golpe | multiplicam-se | ⛔ uma linha por tipo (a primeira ganha — ver a nota) |
//!
//! ⛔ **Divergência DECLARADA:** o alvo tem vida INTEIRA e arredonda cada golpe (`10 × 0,33 = 3`,
//! `10 × 0,25 = 3`, `2 × 0,25 = 1`); a casa é o porte do GDevelop, com vida `f64`, e **não
//! arredonda** — três golpes de `3,3` tiram `9,9` e não `9`.
//!
//! # Onde a taxa entra no pipeline
//!
//! Depois da armadura e antes do escudo (a ordem do plano §3, que é onde o RPG Maker põe a taxa
//! do elemento: sobre o resultado da defesa). O alvo não tem armadura, logo não decide esta metade.
//!
//! # O dano que DURA ([`Aflicoes`])
//!
//! Um golpe que o pede deixa uma **aflição** do tipo dele (o veneno, a queimadura): `por_s` pontos
//! por segundo durante `dur_s`, entregues em PULSOS a cada `intervalo_s`. Nenhuma referência da
//! pesquisa tem isto embutido (o RPG Maker faz *«% por turno»*), logo as leis abaixo são NOSSAS e
//! cada uma tem gate:
//!
//! - **o total é `por_s × dur_s`, sempre** — o último pulso entrega a fracção que falta, e um
//!   intervalo que não divide a duração não rouba nem acrescenta dano;
//! - **reaplicar RENOVA e nunca adia**: a duração passa ao maior dos dois restos e a taxa à maior
//!   das duas, **com a fase do pulso intacta** — um relógio de pulso reposto a cada reaplicação
//!   deixaria um inimigo atingido depressa sem levar pulso nenhum;
//! - **um tipo, uma aflição**: dois venenos não se somam (o segundo renova o primeiro); fogo e
//!   veneno correm lado a lado;
//! - **um pulso não é um golpe** ([`crate::Vida::pulso`]): não esquiva, não passa pela armadura,
//!   não é travado pela invencibilidade **nem a arma** — senão um herói envenenado ficava invencível
//!   aos inimigos, e o veneno faria o contrário do que se lhe pede.

/// ⭐ **A resposta de uma vida a um tipo de dano** — o multiplicador e se ela o ABSORVE.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Taxa {
    /// `1` = normal · `0` = imune · `0,5` = resiste · `2` = fraca. Negativo ou não-finito conta `0`.
    pub mult: f64,
    /// O dano deste tipo CURA (`dano × mult`) — o elemental de fogo que o fogo alimenta.
    pub absorve: bool,
}

impl Taxa {
    /// Sem resistência nenhuma — o golpe passa como passava antes de haver tipos, **ao bit**
    /// (`d × 1,0` é exactamente `d` em `f64`).
    pub const NEUTRA: Self = Self {
        mult: 1.0,
        absorve: false,
    };

    /// O multiplicador que se aplica: **negativo ou não-finito é `0`** (o oráculo: uma taxa
    /// negativa não cura — curar é o [`Self::absorve`], um interruptor à parte).
    #[must_use]
    pub fn fator(self) -> f64 {
        if self.mult.is_finite() && self.mult > 0.0 {
            self.mult
        } else {
            0.0
        }
    }
}

/// Uma aflição viva — o veneno, a queimadura. Ver o cabeçalho.
#[derive(Clone, Debug, PartialEq)]
pub struct Aflicao {
    /// O tipo (o nome que o artista escreveu no dano, já DOBRADO por quem chama).
    pub tipo: String,
    /// Pontos por segundo.
    pub por_s: f64,
    /// Quanto falta, em segundos.
    pub resta_s: f64,
    /// Cada quanto tempo pulsa, em segundos.
    pub intervalo_s: f64,
    /// Quanto falta para o próximo pulso.
    ate_pulso_s: f64,
    /// O tempo desde o último pulso — é ele que faz o último pulso levar só a fracção que falta.
    acumulado_s: f64,
}

/// O que sobra de um relógio quando ele "chegou": abaixo disto o resto é arredondamento de somar
/// `dt` de `f64` (`180` passos de `1/60` deixam `~1e-15`), nunca tempo que o artista pediu.
const CHEGOU_S: f64 = 1e-9;

/// ⭐ **As aflições de UMA vida** — o estado que o anel de checkpoints guarda ao lado da [`crate::Vida`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Aflicoes(pub Vec<Aflicao>);

/// Um PULSO: o tipo e quantos pontos ele leva (antes da taxa da vida).
#[derive(Clone, Debug, PartialEq)]
pub struct Pulso {
    /// O tipo da aflição que pulsou.
    pub tipo: String,
    /// Os pontos, antes da resistência.
    pub pontos: f64,
}

impl Aflicoes {
    /// Aplica (ou RENOVA) uma aflição. Um pedido com um número não-finito ou `≤ 0` não faz nada, e
    /// um intervalo `≤ 0` pulsa a cada tique (a lava).
    pub fn aplica(&mut self, tipo: &str, por_s: f64, dur_s: f64, intervalo_s: f64) {
        let bom = |x: f64| x.is_finite() && x > 0.0;
        if !bom(por_s) || !bom(dur_s) {
            return;
        }
        let intervalo_s = if intervalo_s.is_finite() {
            intervalo_s.max(0.0)
        } else {
            0.0
        };
        if let Some(a) = self.0.iter_mut().find(|a| a.tipo == tipo) {
            // ⛔ A FASE fica: repor o `ate_pulso_s` aqui deixaria um golpe repetido adiar o pulso
            // para sempre.
            a.resta_s = a.resta_s.max(dur_s);
            a.por_s = a.por_s.max(por_s);
            return;
        }
        self.0.push(Aflicao {
            tipo: tipo.to_owned(),
            por_s,
            resta_s: dur_s,
            intervalo_s,
            ate_pulso_s: intervalo_s,
            acumulado_s: 0.0,
        });
    }

    /// O tempo anda `dt_s` e devolve os pulsos deste passo, pela ordem das aflições. As que
    /// acabaram saem.
    pub fn anda(&mut self, dt_s: f64) -> Vec<Pulso> {
        let mut pulsos = Vec::new();
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return pulsos;
        }
        for a in &mut self.0 {
            let mut t = dt_s;
            while t > 0.0 && a.resta_s > CHEGOU_S {
                let passo = t.min(a.resta_s).min(if a.intervalo_s > 0.0 {
                    a.ate_pulso_s
                } else {
                    t
                });
                t -= passo;
                a.resta_s -= passo;
                a.ate_pulso_s -= passo;
                a.acumulado_s += passo;
                let acabou = a.resta_s <= CHEGOU_S;
                if acabou || a.intervalo_s <= 0.0 || a.ate_pulso_s <= CHEGOU_S {
                    // O último pulso leva o RESTO: somar o que o arredondamento deixou em
                    // `resta_s` é o que faz o total fechar em `por_s × dur_s`.
                    let tempo = a.acumulado_s + if acabou { a.resta_s.max(0.0) } else { 0.0 };
                    pulsos.push(Pulso {
                        tipo: a.tipo.clone(),
                        pontos: a.por_s * tempo,
                    });
                    a.acumulado_s = 0.0;
                    a.ate_pulso_s = a.intervalo_s;
                    if acabou {
                        a.resta_s = 0.0;
                    }
                }
            }
        }
        self.0.retain(|a| a.resta_s > CHEGOU_S);
        pulsos
    }

    /// Cura TODAS as aflições (a morte, um recomeço).
    pub fn limpa(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
#[path = "tipos_tests.rs"]
mod tests;
