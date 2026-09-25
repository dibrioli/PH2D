//! ⭐⭐⭐ **O IMPACTO** (plano 28, W5) — o que faz um golpe PESAR, do lado da apresentação: a pausa
//! no golpe (*hitstop*) e os números de dano a subir.
//!
//! O empurrão e o piscar **não** moram aqui, e a razão é a espécie de cada um:
//!
//! | peça | é | mora |
//! |---|---|---|
//! | empurrão | estado de SIMULAÇÃO (o replay tem de o refazer) | a ponte da vida, no passo da física |
//! | piscar | função do relógio da LEI (exacto num scrub) | a ponte da vida + a porta do desenho |
//! | pausa no golpe | RELÓGIO DE PAREDE retido antes do acumulador | aqui (e a shell a consome) |
//! | números | apresentação que nasce de um facto e envelhece | aqui |
//!
//! ⭐ **A pausa congela o JOGO INTEIRO e não um corpo**: ela retém tempo de parede antes do
//! acumulador do passo fixo, logo física, relógios, animações e a própria vida ficam parados o
//! mesmo tempo — e não entra no anel nem na fita, porque não é estado de simulação: *o jogo não
//! anda* é a ausência de tiques, e um replay que refizesse os mesmos tiques dá o mesmo mundo.
//!
//! ⚠️ **Os números envelhecem com o relógio do JOGO**, nunca com o de parede: durante a pausa o
//! número que acabou de nascer fica pendurado — é isso que o faz ler-se como parte do golpe.

use bevy_ecs::world::World;
use ph2d_ecs::{Entity, SimWorld};
use ph2d_health::Pausa;
use ph2d_host::WindowSize;
use ph2d_physics_ecs::{Damage, Health, HealthEvent, HealthEventKind};
use ph2d_render::Camera2d;

/// Quanto vive um número, em segundos de JOGO.
///
/// ⚠️ **Faixa de PRODUTO e não limite de recurso** (§0.0 do roteador — dizê-lo é a única forma
/// honesta): um número tem de durar o bastante para se LER (um valor de dois algarismos lê-se em
/// ~0,3 s) e sair antes do golpe seguinte de uma rajada normal (`0,2`–`1 s` entre golpes). Não há
/// recurso a medir aqui — há um olho.
pub const VIDA_DO_NUMERO_S: f32 = 0.8;

/// Quanto sobe um número na vida dele, em ALTURAS do próprio número — ele sai de cima do alvo
/// sem tapar o seguinte. Faixa de produto, pela mesma razão do [`VIDA_DO_NUMERO_S`].
pub const SUBIDA_EM_ALTURAS: f32 = 1.6;

/// A fracção final da vida em que o número desvanece (antes disso é opaco).
pub const FRACCAO_DO_DESVANECER: f32 = 0.4;

/// **Um número a subir** — onde nasceu, o texto, e há quanto tempo.
#[derive(Clone, Debug, PartialEq)]
pub struct Numero {
    /// Onde nasceu, em MUNDO (por cima do alvo).
    pub nasceu: [f32; 2],
    /// O que diz.
    pub texto: String,
    /// A cor, RGBA linear.
    pub cor: [f32; 4],
    /// A altura, em metros.
    pub tamanho: f32,
    /// Há quanto tempo de JOGO nasceu, em segundos.
    pub idade: f32,
}

impl Numero {
    /// Onde está AGORA, em mundo — sobe a direito, desacelerando (o arco de quem foi atirado).
    #[must_use]
    pub fn onde(&self) -> [f32; 2] {
        let t = (self.idade / VIDA_DO_NUMERO_S).clamp(0.0, 1.0);
        // `1 − (1 − t)²`: rápido ao nascer, parado ao morrer — o número ASSENTA para ser lido.
        let subida = 1.0 - (1.0 - t) * (1.0 - t);
        [
            self.nasceu[0],
            self.nasceu[1] + subida * SUBIDA_EM_ALTURAS * self.tamanho,
        ]
    }

    /// A opacidade AGORA — `1` até ao desvanecer, depois a descer linear até `0`.
    #[must_use]
    pub fn alfa(&self) -> f32 {
        let t = (self.idade / VIDA_DO_NUMERO_S).clamp(0.0, 1.0);
        let inicio = 1.0 - FRACCAO_DO_DESVANECER;
        if t <= inicio {
            1.0
        } else {
            ((1.0 - t) / FRACCAO_DO_DESVANECER).clamp(0.0, 1.0)
        }
    }
}

/// **O estado do impacto** — a pausa que falta reter e os números vivos. Fora do mundo: nada disto
/// é documento nem simulação.
#[derive(Debug, Default)]
pub struct ImpactoState {
    /// A pausa no golpe que ainda falta reter.
    pub pausa: Pausa,
    numeros: Vec<Numero>,
}

/// ⭐ **Quanto pausar por estes golpes** — o MAIOR pedido, nunca a soma: três inimigos atingidos
/// pelo mesmo golpe pesam como um (somar congelaria o jogo três vezes mais por um golpe só).
///
/// - golpe que ENTRA (vida ou escudo) → o [`Damage::hitstop_s`] de QUEM BATE;
/// - morte → o [`Health::death_hitstop_s`] de QUEM MORRE (o chefe pesa mais que o morcego);
/// - esquiva, cura → nada.
///
/// ⚠️ Um golpe vindo de um VERBO da tabela tem como fonte o próprio alvo, que não tem `Damage`: não
/// pesa pela espada (não há espada), só pela morte.
#[must_use]
pub fn pausa_pedida(w: &World, eventos: &[HealthEvent]) -> f64 {
    let mut pedido = 0.0_f64;
    for ev in eventos {
        let s = match ev.kind {
            HealthEventKind::Damaged { .. } | HealthEventKind::Shielded { .. } => {
                w.get::<Damage>(ev.source).map_or(0.0, |d| d.hitstop_s)
            }
            HealthEventKind::Died => w
                .get::<Health>(ev.target)
                .map_or(0.0, |h| h.death_hitstop_s),
            _ => 0.0,
        };
        if s.is_finite() && f64::from(s) > pedido {
            pedido = f64::from(s);
        }
    }
    pedido
}

/// O texto de um número — o dano ARREDONDADO ao inteiro, e nunca `0` (um golpe que entrou e tirou
/// `0,3` diz `1`: um número que diz zero lê-se como *«não entrou»*, que é mentira).
#[must_use]
pub fn texto_do_dano(quanto: f64) -> String {
    let n = quanto.round().max(1.0);
    format!("{n:.0}")
}

impl ImpactoState {
    /// **Ouve os factos de vida de um dispatch** — pede a pausa e faz nascer os números.
    pub fn ouve(&mut self, sim: &SimWorld, eventos: &[HealthEvent]) {
        self.pausa.pede(pausa_pedida(sim.world(), eventos));
        for ev in eventos {
            let quanto = match ev.kind {
                HealthEventKind::Damaged { amount } | HealthEventKind::Shielded { amount } => {
                    amount
                }
                _ => continue,
            };
            if !(quanto.is_finite() && quanto > 0.0) {
                continue;
            }
            let Some(h) = sim.world().get::<Health>(ev.target) else {
                continue;
            };
            if !h.numbers || !(h.numbers_size.is_finite() && h.numbers_size > 0.0) {
                continue;
            }
            let Some(pos) = centro_de(sim, ev.target) else {
                continue;
            };
            self.numeros.push(Numero {
                // Nasce uma altura dele acima do centro do alvo.
                nasceu: [pos[0], pos[1] + h.numbers_size],
                texto: texto_do_dano(quanto),
                cor: h.numbers_color,
                tamanho: h.numbers_size,
                idade: 0.0,
            });
        }
    }

    /// **Retém tempo de parede** — devolve o `dt` que o acumulador do passo fixo deve receber.
    ///
    /// ⚠️ **Só com o relógio A ANDAR**: pausado, não há jogo a congelar, e reter ali comeria o tempo
    /// de um passo manual (o `step` do transporte) sem ninguém ter pedido.
    pub fn retem(&mut self, wall_dt: f64, a_correr: bool) -> f64 {
        if a_correr {
            self.pausa.consome(wall_dt)
        } else {
            wall_dt
        }
    }

    /// **Envelhece os números** pelo relógio do JOGO (`dt` desse quadro) e esquece os que morreram.
    pub fn anda(&mut self, dt_s: f32) {
        if !(dt_s.is_finite() && dt_s > 0.0) {
            return;
        }
        for n in &mut self.numeros {
            n.idade += dt_s;
        }
        self.numeros.retain(|n| n.idade < VIDA_DO_NUMERO_S);
    }

    /// Os números vivos, pela ordem em que nasceram.
    #[must_use]
    pub fn numeros(&self) -> &[Numero] {
        &self.numeros
    }

    /// **Rebobinar é renascer** — a pausa acaba e os números saem.
    pub fn rewind(&mut self) -> usize {
        let n = self.numeros.len() + usize::from(self.pausa.resta_s() > 0.0);
        self.numeros.clear();
        self.pausa.limpa();
        n
    }

    /// **Pinta os números** por cima da cena, pela porta de texto da casa.
    ///
    /// ⚠️ A `janela` tem de ser a da CENA (a banda), nunca a do quadro — a lei do mapeamento
    /// mundo↔tela que esta casa já pagou cinco vezes.
    pub fn pinta(
        &self,
        camera: &Camera2d,
        janela: WindowSize,
        vector_scene: &mut ph2d_vector::VectorScene,
        text_system: &mut ph2d_text::TextSystem,
    ) {
        for n in &self.numeros {
            let p = n.onde();
            let (x, y) = camera.world_to_screen(p, janela);
            let (_, y_topo) = camera.world_to_screen([p[0], p[1] + n.tamanho], janela);
            let px = (y - y_topo).abs();
            if !(px.is_finite() && px >= 1.0) {
                continue;
            }
            let a = n.alfa();
            let cor = [n.cor[0], n.cor[1], n.cor[2], n.cor[3] * a];
            // A caixa é larga o bastante para 6 algarismos à altura pedida (um algarismo tem
            // ~0,6 altura de largura) e alta de UMA linha.
            let largura = px * 4.0;
            let rect =
                ph2d_editor_core::zones::Rect::new(x - largura * 0.5, y - px * 0.5, largura, px);
            ph2d_editor_core::paint::paint_text_centered(
                text_system,
                vector_scene,
                &n.texto,
                rect,
                px,
                ph2d_vector::Color::new(cor),
            );
        }
    }
}

/// O centro de um objecto em MUNDO (a translação da matriz global dele).
fn centro_de(sim: &SimWorld, e: Entity) -> Option<[f32; 2]> {
    let t = ph2d_ecs::transform_inverse::world_transform(sim.world(), e)?;
    let m = ph2d_ecs::GlobalTransform::from_transform(t).matrix;
    let p = [m.z_axis.x, m.z_axis.y];
    (p[0].is_finite() && p[1].is_finite()).then_some(p)
}

#[cfg(test)]
#[path = "impacto_tests.rs"]
mod tests;
