//! ⭐⭐⭐ **O CICLO DE VIDA** — o item **#12** do TOP-20, e a higiene sem a qual a fábrica (#11) e o
//! projéctil (#14) **vazam** (`docs/Components/09_plano_spawner.md` §2.6).
//!
//! # ⭐⭐⭐ A lei que este módulo estabelece: **a morte só alcança o que NASCEU numa corrida**
//!
//! É o espelho, um nível acima, da lei que o `ph2d-preview-drive` já paga: ali *o que um motor
//! escreve num componente é pré-visualização*; aqui *o que um motor PÕE no mundo* é
//! pré-visualização — e, pela mesma razão, **uma corrida nunca apaga documento**.
//!
//! O mecanismo é exacto: a física move objectos autorados e a captura **repõe** a pose autorada.
//! Apagar não tem valor a repor — o ledger troca valores, não existências. Um `Lifetime` que
//! apagasse um objecto desenhado destruiria trabalho com um `Ctrl+Z` que não o traz de volta.
//!
//! ⇒ tudo aqui exige [`Spawned`], e um objecto autorado com estes componentes é **inerte**, com o
//! painel a dizê-lo (a metade honesta do `Timer`: *«This timer never starts»*). ⚠️ **E isso não faz
//! deles controlos mortos:** eles vivem na **RECEITA** e correm nas **CÓPIAS** — um mestre está
//! escondido por construção (*Make Component*), então o sítio onde se autoram é exactamente o sítio
//! onde fazem efeito.
//!
//! # ⚠️ A morte é ADIADA, e isso veio do oráculo
//!
//! Medido no Godot 4.7.2 sem interface
//! ([`ferramentas/godot_spawn_lifetime_probe.gd`](../../../docs/Components/ferramentas/godot_spawn_lifetime_probe.gd)):
//! logo depois de `queue_free()` o nó está **válido, na árvore, filho do pai e na consulta de
//! grupo**; só no quadro seguinte desaparece. ⇒ estas funções devolvem **factos** ([`Death`]) e
//! não tocam no mundo: quem morreu continua a existir para toda consulta até ao dreno do fim do
//! quadro. *É isso que torna um laço de iteração seguro sem uma regra escrita em lado nenhum.*
//!
//! # ⚠️ E o relógio carrega o resto, também medido no oráculo
//!
//! Um período de `0,105 s` a 60 Hz (`6,3` tiques) dá no Godot os períodos `[6,6,7,6,6,7,…]`, média
//! `6,273` — ele **subtrai** o período e guarda o resto, em vez de re-zerar (que daria `7,7,7`). É
//! a mesma lei que o [`crate::timer::advance`] já implementa, e aqui ela reaparece por o consumidor
//! ser outro. ⛔ Não a reescreva: uma vida é um *one-shot*, logo o resto só importa ao INSTANTE da
//! morte, e é ele que este módulo acerta.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{With, Without};
use serde::{Deserialize, Serialize};

use crate::{Entity, StableId, Transform, World};

/// **Quem pôs esta entidade no mundo** — o [`StableId`] da fábrica, e o tique em que ela nasceu.
///
/// ⭐⭐⭐ **NÃO é um componente registado, e a ausência é a decisão** — é ela que faz a lei do §2.1
/// do plano existir: *o que nasce numa corrida não é documento*. Registá-lo poria cada cópia no
/// ficheiro e na pilha de `Ctrl+Z`, a 60 Hz.
///
/// ⚠️ **A porta fecha-se pelo TIPO, não por um gate:** registar exige `Serialize`, e sem ele o
/// registo **não compila** — o precedente do [`crate::TimerRuntime`], onde uma prova de mutação
/// mostrou que a ausência era load-bearing.
///
/// ⚠️ **O `by` não é decoração:** é ele que responde *«quantos vivos esta fábrica tem?»* numa
/// varredura só, sem um segundo índice a manter coerente com o mundo depois de todo restore de undo
/// — a razão pela qual o [`crate::stable_id`] recusa por escrito um mapa `nome → entidade`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Spawned {
    /// O [`StableId`] da fábrica que a pôs aqui. `0` = ninguém a reclama.
    pub by: u64,
    /// O tique do passo fixo em que ela nasceu.
    ///
    /// ⚠️ **Ele existe para a LEI D do oráculo**: *o recém-nascido não tica no tique em que nasce*
    /// (medido: nascido no tique `3`, tem `1` tique no `4`). Sem isto, uma cópia com vida de um
    /// tique morreria antes de alguém a ver.
    pub born_tick: u64,
}

/// **Viva tanto tempo, e morra** — o TTL, o vazamento nº 1 de quem começa.
///
/// ⚠️ **Microssegundos e `u64`**, como o [`crate::Timer`] e pela mesma razão: o replay tem de
/// reproduzir o instante da morte, e um acumulador em vírgula flutuante não é bit-idêntico entre
/// sistemas.
#[derive(Component, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lifetime {
    /// Quanto ela vive, em microssegundos. **`0` = não morre** — a mesma recusa embutida do
    /// `Timer`: um campo por preencher não pode ser um gesto destrutivo.
    pub duration_us: u64,
    /// O sinal publicado na morte. **Vazio = calada** — a lei da casa (*um produtor sem nome não
    /// fala, em vez de falar com um nome vazio*).
    pub on_death: String,
}

impl Default for Lifetime {
    fn default() -> Self {
        Self {
            // ⚠️ **Dois segundos, e não zero**: `0` é o valor que NÃO mata, e um componente
            // acabado de acrescentar que não faz nada lê-se como partido.
            duration_us: 2_000_000,
            on_death: String::new(),
        }
    }
}

/// **O relógio de uma vida** — ⛔ **NÃO registado, de propósito** (a razão está no [`Spawned`]).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LifetimeRuntime {
    /// Microssegundos vividos.
    pub elapsed_us: u64,
}

/// **Morra ao sair do ecrã DO JOGO**, com uma margem em metros.
///
/// ⚠️⚠️ **Contra a [`crate::GameCamera`] activa, nunca contra a vista do editor** — uma corrida que
/// dependesse de onde o artista rolou o ecrã seria outra corrida em cada máquina. ⇒ **sem câmera de
/// jogo na cena este componente não mede nada**, e o painel di-lo em vez de o esconder.
///
/// ⭐ **E ele é geométrico puro, não um notificador de render** — a decisão foi forçada pelo
/// oráculo: o `VisibleOnScreenNotifier2D` do Godot vive no servidor de render e **não dispara
/// nenhuma vez sem janela** (medido duas vezes, `is_on_screen() = false` mesmo na origem com
/// câmera). Ganha-se com isso o que ele não tem: **isto testa-se sem GPU**.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct DestroyOutside {
    /// Quanto o rectângulo da câmera cresce antes de a morte valer, em metros. `0` = a borda exacta.
    ///
    /// ⚠️ **Uma margem NEGATIVA encolhe o rectângulo** e mataria coisas à vista — é cravada em `0`
    /// na leitura, e não no campo: cravar no campo faria o painel devolver um número diferente do
    /// que o artista escreveu.
    pub margin: f32,
}

impl Default for DestroyOutside {
    fn default() -> Self {
        // ⚠️ **Meio metro de folga, e não zero:** uma bala que morre na borda exacta desaparece
        // **à vista** no instante em que a câmera treme. A margem de omissão é o que torna a morte
        // invisível, que é o ponto do componente.
        Self { margin: 0.5 }
    }
}

/// **De que morreu.** ⚠️ Duas causas e não um `bool`: o diagnóstico do smoke lê-as, e um dia o
/// painel vai querer dizer *«morreu de velha»* contra *«saiu do ecrã»*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathCause {
    /// A vida dela acabou.
    Aged,
    /// Ela saiu do rectângulo da câmera.
    Outside,
}

/// **Um facto de morte** — quem, porquê, e o que ele publica.
///
/// ⚠️ **Nada aqui toca no mundo** (ver o cabeçalho): a remoção é do dreno da shell, uma vez por
/// quadro, DEPOIS de todos os produtores.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Death {
    /// Quem morreu.
    pub entity: Entity,
    /// O sinal a publicar. Vazio = calada.
    pub signal: String,
    pub why: DeathCause,
}

/// **Põe o relógio em quem nasceu com uma vida e ainda não o tem.**
///
/// ⚠️ **Idempotente e no caminho do tique**, como o `reconcile` dos timers: o nascimento é o sítio
/// natural para o inserir, e esta varredura é a rede para todo caminho que ponha um [`Spawned`] sem
/// passar por lá. Ela custa `O(quem tem Lifetime sem relógio)`, que em regime é **zero** — a query
/// é `Without<LifetimeRuntime>`, e quem já o tem vive noutro arquétipo.
pub fn reconcile_lifetimes(world: &mut World) {
    let novos: Vec<Entity> = world
        .query_filtered::<Entity, (With<Lifetime>, With<Spawned>, Without<LifetimeRuntime>)>()
        .iter(world)
        .collect();
    for e in novos {
        world.entity_mut(e).insert(LifetimeRuntime::default());
    }
}

/// ⭐⭐ **O TIQUE DAS VIDAS** — avança `dt_us` e devolve quem morreu.
///
/// # ⚠️ A ordem é DETERMINISTA, e não é a da query
///
/// Ela sai do [`StableId`] de quem morre, como a do [`crate::resolve_signal_actions`]: a ordem do
/// arquétipo muda quando um componente é inserido, e um replay que publique os sinais de morte
/// noutra ordem diverge.
///
/// # ⚠️ O recém-nascido não envelhece no tique em que nasce
///
/// É a **lei D do oráculo**, medida. `tick_now` é o tique que acabou de correr; quem nasceu nele
/// fica de fora. ⛔ Sem isto uma cópia com `duration_us` menor que um tique morreria no mesmo
/// quadro em que nasceu, e o artista veria *nada*.
#[must_use]
pub fn tick_lifetimes(world: &mut World, dt_us: u64, tick_now: u64) -> Vec<Death> {
    reconcile_lifetimes(world);
    if dt_us == 0 {
        return Vec::new();
    }
    // ⚠️ **A identidade NÃO entra na query** — é a lei que a porta das tags pagou em 2026-09-14:
    // um `Option<&StableId>` num `try_query` faz a consulta inteira responder «ninguém» num mundo
    // que nunca viu o componente. Aqui a query regista-o (é `&mut World`), mas a forma fica a
    // mesma de propósito: *a identidade lê-se por ACERTO, e os acertos são poucos.*
    let mut colhidos: Vec<Death> = Vec::new();
    let mut q = world.query::<(Entity, &Lifetime, &mut LifetimeRuntime, &Spawned)>();
    for (e, vida, mut relogio, nascimento) in q.iter_mut(world) {
        if vida.duration_us == 0 || nascimento.born_tick >= tick_now {
            continue;
        }
        // ⭐⭐ **A morte é a TRAVESSIA, não o estado** — e foi um gate que o disse: sem isto uma
        // cópia já morta publicava o sinal dela **em todo tique seguinte**, porque quem a remove é
        // o dreno da shell, um quadro depois. É a lei do oráculo pelo outro lado: no Godot um
        // segundo `queue_free()` não faz nada, porque a bandeira já está posta — a nossa bandeira é
        // o próprio relógio ter passado a duração.
        let antes = relogio.elapsed_us;
        relogio.elapsed_us = relogio.elapsed_us.saturating_add(dt_us);
        if antes < vida.duration_us && relogio.elapsed_us >= vida.duration_us {
            colhidos.push(Death {
                entity: e,
                signal: vida.on_death.clone(),
                why: DeathCause::Aged,
            });
        }
    }
    ordem_da_identidade(world, colhidos)
}

/// A ordem em que os factos de morte saem: a da [`StableId`], nunca a da query.
fn ordem_da_identidade(world: &World, mortes: Vec<Death>) -> Vec<Death> {
    let mut com_id: Vec<(u64, Death)> = mortes
        .into_iter()
        .map(|d| (world.get::<StableId>(d.entity).map_or(u64::MAX, |s| s.0), d))
        .collect();
    com_id.sort_by_key(|(id, _)| *id);
    com_id.into_iter().map(|(_, d)| d).collect()
}

/// ⭐⭐ **A COLHEITA DO FORA-DO-ECRÃ** — quem saiu do rectângulo `centro ± meia`, crescido pela
/// margem de cada um.
///
/// ⚠️ **O rectângulo entra por parâmetro** e não se lê a câmera aqui dentro: é o que torna esta lei
/// testável sem mundo de câmera nenhum, e é o que impede uma segunda resposta a *«qual é a câmera
/// activa?»* (a porta é [`crate::camera_2d::active_camera_of`]).
///
/// ⚠️ **O ponto testado é a posição de MUNDO** ([`crate::world_transform`]), nunca a local: uma
/// bala filha de uma nave leria a posição dela em relação à nave e nunca sairia do ecrã.
#[must_use]
pub fn reap_outside(world: &mut World, center: [f32; 2], half: [f32; 2]) -> Vec<Death> {
    let mut q =
        world.query_filtered::<(Entity, &DestroyOutside), (With<Spawned>, With<Transform>)>();
    // ⚠️ Os candidatos primeiro: a pose de mundo pede `&World`, e a query segura `&mut`.
    let candidatos: Vec<(Entity, f32)> = q.iter(world).map(|(e, d)| (e, d.margin)).collect();
    let mundo: &World = world;
    let mut colhidos: Vec<Death> = Vec::new();
    for (e, margem) in candidatos {
        let Some(t) = crate::world_transform(mundo, e) else {
            continue;
        };
        let m = margem.max(0.0);
        let p = t.translation;
        if (p.x - center[0]).abs() > half[0] + m || (p.y - center[1]).abs() > half[1] + m {
            colhidos.push(Death {
                entity: e,
                signal: String::new(),
                why: DeathCause::Outside,
            });
        }
    }
    ordem_da_identidade(mundo, colhidos)
}

#[cfg(test)]
#[path = "lifetime_tests.rs"]
mod tests;
