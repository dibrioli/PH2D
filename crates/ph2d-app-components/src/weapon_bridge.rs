//! ⭐⭐⭐ **A PONTE DA ARMA** — os factos que a lei pura devolveu viram munição escrita e sinais
//! publicados.
//!
//! A fronteira é a mesma da fábrica e da tabela de acções: *a lei devolve FACTOS, a ponte
//! aplica-os.* O [`ph2d_ecs::weapon::avanca`] não sabe o que é um mundo — ele recebe a
//! [`Municao`] que esta ponte leu e devolve um [`ph2d_ecs::Tiro`].
//!
//! # ⚠️ O PENTE é lido e escrito NA PRÓPRIA ENTIDADE
//!
//! O [`ph2d_ecs::counter::soma`] **soma todos** os contadores com um nome — é a pergunta certa
//! para um placar e a errada para uma escrita: um `-1` teria de escolher um dono. ⇒ a arma lê e
//! escreve o [`ph2d_ecs::Counter`] que vive **nela**, e o catálogo declara-o (`requires`).
//!
//! ⭐ O HUD continua a mostrá-lo pela soma global, e é isso que faz *«a munição aparece no placar»*
//! custar zero linhas — duas perguntas diferentes sobre o mesmo dado, cada uma com a porta dela.
//!
//! # ⚠️ A ORDEM das armas é a da IDENTIDADE
//!
//! Como na fábrica: a iteração entre arquétipos do bevy **não é prometida**, e duas armas a
//! disparar no mesmo tique têm de produzir sempre a mesma sequência de sinais — senão o replay
//! determinista (`physics_ecs_c9`) diverge entre máquinas.

use ph2d_ecs::counter::{self, Ambito};
use ph2d_ecs::{
    Counter, CounterRuntime, Entity, Municao, SimWorld, StableId, WeaponFire, WeaponRuntime,
};

/// **O que um tique de armas produziu.**
///
/// ⚠️ **Três listas e não uma**, pela razão do `FactoryTick`: disparar, ficar seco e acabar de
/// recarregar são factos de naturezas diferentes, e o artista liga cada um a outra coisa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeaponTick {
    /// `(arma, sinal)` de cada tiro — é este que a [`ph2d_ecs::Factory`] ouve.
    pub disparos: Vec<(Entity, String)>,
    /// `(arma, sinal)` de cada clique seco.
    pub secas: Vec<(Entity, String)>,
    /// `(arma, sinal)` de cada pente que ficou cheio.
    pub recarregadas: Vec<(Entity, String)>,
}

impl WeaponTick {
    /// Quantos factos ao todo — o número que o log do smoke imprime.
    #[must_use]
    pub fn len(&self) -> usize {
        self.disparos.len() + self.secas.len() + self.recarregadas.len()
    }

    /// Nenhum facto neste tique.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Garante o [`WeaponRuntime`] de toda arma — o irmão do `reconcile_factories`.
fn reconcile(sim: &mut SimWorld) {
    let mundo = sim.world_mut();
    let sem: Vec<Entity> = {
        let mut q = mundo.query_filtered::<Entity, (
            bevy_ecs::prelude::With<WeaponFire>,
            bevy_ecs::prelude::Without<WeaponRuntime>,
        )>();
        q.iter(mundo).collect()
    };
    for e in sem {
        mundo.entity_mut(e).insert(ph2d_ecs::weapon_born());
    }
}

/// ⭐⭐ **Um tique de todas as armas.**
///
/// `fired` são os nomes que soaram neste tique — os mesmos que a fábrica e a tabela de acções
/// leem, porque o barramento é um só (ADR-0075).
///
/// ⚠️ **Com o relógio parado ela não corre**: uma arma é da CORRIDA, e sem esta cerca cada tecla
/// escrita num campo do editor gastaria munição.
#[must_use]
pub fn frame(sim: &mut SimWorld, playing: bool, dt_us: u64, fired: &[&str]) -> WeaponTick {
    let mut out = WeaponTick::default();
    if !playing {
        return out;
    }
    reconcile(sim);

    // 1. As armas, pela ordem da IDENTIDADE — a query segura `&mut`, então copia-se o que é preciso.
    let mut armas: Vec<(Entity, WeaponFire, u64)> = {
        let mundo = sim.world_mut();
        let mut q = mundo.query::<(Entity, &WeaponFire)>();
        q.iter(mundo)
            .map(|(e, w)| (e, w.clone(), 0u64))
            .collect::<Vec<_>>()
    };
    {
        let mundo: &bevy_ecs::world::World = sim.world();
        for a in &mut armas {
            a.2 = mundo.get::<StableId>(a.0).map_or(u64::MAX, |s| s.0);
        }
    }
    armas.sort_by_key(|a| a.2);

    for (e, cfg, _) in armas {
        // 2. O pente DESTA arma. ⚠️ Um `ammo_counter` vazio é munição INFINITA; ⛔ não é zero balas.
        let alvo = cfg.ammo_counter.trim();
        let mun = if alvo.is_empty() {
            Municao::default()
        } else {
            let mundo: &bevy_ecs::world::World = sim.world();
            // ⭐⭐ **PELA PORTA, desde 2026-09-20** ([`ph2d_ecs::counter::soma`] com
            // [`Ambito::Objecto`]). Ela nasceu a somar a cena inteira e esta ponte escrevia a
            // leitura por-objecto à mão — *uma lei escrita em dois sítios ainda não é uma lei*, e
            // a segunda cópia apareceu no dia em que a vigia precisou da mesma pergunta.
            //
            // ⚠️ O `cheio` continua a vir do `Counter` **desta** entidade: ele é o `start` da
            // CONFIG e não um valor vivo, logo não é coisa que uma soma responda.
            match (
                mundo.get::<Counter>(e),
                counter::soma(mundo, alvo, Ambito::Objecto(e)),
            ) {
                (Some(c), Some(tem)) => Municao {
                    tem,
                    cheio: c.start,
                    existe: true,
                    // ⚠️ O depósito é resolvido a seguir, num passo próprio — aqui ainda é
                    // «infinito», que é o que um `reserve_counter` vazio produz.
                    ..Municao::default()
                },
                // ⚠️ O contador que o artista nomeou não está NESTA entidade: a arma fica com
                // munição infinita em vez de ficar inerte — a lei do alvo que não existe, da
                // tabela de acções. O painel é quem o diz.
                _ => Municao::default(),
            }
        };

        // 2-bis. ⭐⭐⭐ **O DEPÓSITO** — um contador NOMEADO em qualquer sítio da cena, e não neste
        // objecto: uma entidade tem **um** `Counter` e o pente já o ocupa.
        //
        // ⛔⛔ **Um nome que dois objectos carregam é RECUSADO** (`dono_unico` devolve `None`) e a
        // arma cai em reserva infinita — *somar dez depósitos é exacto, tirar cinco a dez não é*.
        // O painel é quem separa este silêncio do de um campo vazio.
        let deposito = cfg.reserve_counter.trim();
        let dono_reserva = (!deposito.is_empty())
            .then(|| counter::dono_unico(sim.world(), deposito))
            .flatten();
        let mun = match dono_reserva {
            // ⚠️ **Pela MESMA porta do pente** (`Ambito::Objecto` sobre o dono), que é o que faz
            // um depósito no 1.º quadro da cena já valer o `start` que o artista escreveu.
            Some(d) => Municao {
                reserva: counter::soma(sim.world(), deposito, Ambito::Objecto(d)).unwrap_or(0),
                reserva_existe: true,
                ..mun
            },
            None => mun,
        };

        let pediu_tiro =
            !cfg.on_signal.trim().is_empty() && fired.contains(&cfg.on_signal.as_str());
        let pediu_recarga =
            !cfg.reload_on.trim().is_empty() && fired.contains(&cfg.reload_on.as_str());

        let mut st = sim
            .world()
            .get::<WeaponRuntime>(e)
            .copied()
            .unwrap_or_default();
        let t = ph2d_ecs::weapon_avanca(&cfg, &mut st, mun, dt_us, pediu_tiro, pediu_recarga);
        sim.world_mut().entity_mut(e).insert(st);

        // 3. A munição, se ela existe e mudou.
        if mun.existe
            && t.municao != mun.tem
            && let Some(mut rt) = sim.world_mut().get_mut::<CounterRuntime>(e)
        {
            rt.value = t.municao;
        }

        // 3-bis. O DEPÓSITO, se ele existe e mudou.
        //
        // ⚠️ **Ele pode ainda não ter `CounterRuntime`** — o `dono_unico` não o exige, de propósito
        // (um depósito no 1.º quadro já tem dono). ⇒ escreve-se o componente, como o
        // `add_to_counter` da tabela de acções já faz. *Duas leituras diferentes do «por estrear»
        // dariam uma arma que come a primeira recarga.*
        if mun.reserva_existe
            && t.reserva != mun.reserva
            && let Some(d) = dono_reserva
            && let Ok(mut ent) = sim.world_mut().get_entity_mut(d)
        {
            if let Some(mut rt) = ent.get_mut::<CounterRuntime>() {
                rt.value = t.reserva;
            } else {
                ent.insert(CounterRuntime { value: t.reserva });
            }
        }

        // 4. Os factos. ⚠️ Um sinal vazio fica CALADO — a lei da casa.
        if t.disparou && !cfg.on_fire.trim().is_empty() {
            out.disparos.push((e, cfg.on_fire.clone()));
        }
        if t.seca && !cfg.on_empty.trim().is_empty() {
            out.secas.push((e, cfg.on_empty.clone()));
        }
        if t.recarregou && !cfg.on_reloaded.trim().is_empty() {
            out.recarregadas.push((e, cfg.on_reloaded.clone()));
        }
    }
    out
}

#[cfg(test)]
#[path = "weapon_bridge_tests.rs"]
mod tests;
