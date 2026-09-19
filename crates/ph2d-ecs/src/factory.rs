//! ⭐⭐⭐ **A FÁBRICA** — o item **#11** do TOP-20, e *«a categoria que quase nenhuma engine grande
//! tem como componente»* (`docs/Components/09_plano_spawner.md`).
//!
//! Medido nos dossiês: **Godot, Unity, Unreal, GameMaker, Construct e Bevy não têm** um spawner de
//! Inspector — o idioma é sempre `instantiate()` em código. Têm-na a Defold (*Factory*) e as
//! engines *no-code*. ⇒ o diferencial é directo.
//!
//! # ⚠️ As quatro perguntas, e a porta única de cada uma
//!
//! | pergunta | a porta | ⛔ o que NÃO é |
//! |---|---|---|
//! | **o quê** | o [`StableId`] de um MESTRE ([`Factory::master`]) | não é um caminho de ficheiro — o fluxo do produto é *Make Component* |
//! | **onde** | [`SpawnAt`] | não há componente `SpawnPoint`: **um ponto de nascimento é um objecto marcado com uma tag** |
//! | **quando** | [`Factory::on_signal`] | ⛔ **não há relógio próprio** — o ritmo vem do [`crate::Timer`], que já existe |
//! | **quanto** | `burst`, `alive_max`, `total_max` | — |
//!
//! ⭐⭐ **Porque a fábrica não tem relógio próprio** (a composição foi MEDIDA antes de se construir,
//! lei §1.8 do levantamento): um `Timer{repeat, signal}` mais `Factory{on_signal}` **já dão** *«nasce
//! um por segundo»*, e um `rate` aqui não compraria capacidade nenhuma — compraria cliques, ao preço
//! de **dois motores para a mesma lei**, com dois sítios onde *«que horas são»* pode divergir. Os
//! cliques compram-se com os **componentes requeridos** (a F0/F3 shipou-os), que puxam o timer já
//! ligado.
//!
//! # ⚠️ Esta lei não põe nada no mundo
//!
//! Ela devolve [`Birth`]es, como o [`crate::resolve_signal_actions`] devolve efeitos: quem instancia
//! é a shell, pela porta `instantiate_master`, que é a MESMA do botão *Instantiate* — e é por isso
//! que editar a receita muda as cópias vivas, que as variantes funcionam e que os documentos
//! possuídos são clonados em vez de partilhados, **sem uma linha aqui**.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{With, Without};
use serde::{Deserialize, Serialize};

use crate::lifetime::Spawned;
use crate::{Entity, StableId, World};
use ph2d_tags::{TagId, TagTree};

/// **Quantas cópias uma fábrica pode pôr no mundo num tique.**
///
/// ⚠️ **De que recurso ele é: o QUADRO.** Medido (`release`, mínimo de 9, `load 16,02`, receita de
/// 3 nós, pela porta em lote [`crate::deep_copy_subtree_many`]):
///
/// | cópias | cena de 10 000 | cena de 100 000 | de um quadro de 16,7 ms |
/// |---:|---:|---:|---:|
/// | 256 | `0,687 ms` | `0,830 ms` | 4–5 % |
/// | **1 024** | **`2,638 ms`** | **`2,833 ms`** | **16–17 %** |
/// | 4 096 | `10,637 ms` | `10,849 ms` | **64 %** |
///
/// ⇒ o degrau a seguir a este come dois terços do quadro. ⚠️ **E o preço por cópia é PLANO na cena**
/// (`2,58`–`3,25 µs`) desde a porta em lote, então o número **não depende do tamanho do projecto** —
/// o que não era verdade com a porta de série, onde a mesma conta dava `144 µs` por cópia a 100 000.
///
/// ⛔ **É uma CONTAGEM, nunca um orçamento de relógio**: um tecto em milissegundos faria o passo fixo
/// produzir um número diferente de nascimentos em cada máquina, e o replay (`physics_ecs_c9`, matriz
/// de 3 OS) divergiria. *Uma lei de simulação não pode perguntar as horas.*
pub const BURST_MAX: u32 = 1_024;

/// **ONDE a cópia nasce.**
///
/// ⛔⛔ **Não há variante `AtSpawnPoint`, e a ausência é uma RECUSA MEDIDA** (plano §2.3): o TOP-20
/// pedia um componente `SpawnPoint` *«consultável por nome/tag»*, e as **tags** (TOP-20 #9, no
/// produto desde 2026-09-14) já o exprimem — um ponto de nascimento é um objecto vazio marcado, e
/// [`crate::tags::tagged`] devolve-os **pela ordem da identidade**, que é determinista. O que sobrava
/// era um gizmo, e um componente novo para desenhar uma cruz é exactamente o item que a §5.0 do
/// `CLAUDE.md` manda medir antes de construir.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SpawnAt {
    /// Na pose de MUNDO da própria fábrica.
    #[default]
    Here,
    /// Espalhadas uniformemente numa caixa centrada nela, em metros.
    Area {
        /// Largura da caixa.
        w: f32,
        /// Altura da caixa.
        h: f32,
    },
    /// **Num objecto marcado com esta tag** — a composição que dissolve o `SpawnPoint`.
    ///
    /// ⚠️ `u64` e não [`TagId`] porque a folha das tags não fala `serde`, de propósito — a mesma
    /// razão do [`crate::SignalTarget::Tagged`].
    Tagged {
        /// A tag dos pontos de nascimento. Uma tag sem ninguém = **ninguém nasce** (silêncio).
        tag: u64,
        /// Qual deles.
        pick: Pick,
    },
}

/// **Qual dos pontos** — e as duas são as que um artista sabe explicar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pick {
    /// Um a seguir ao outro, em roda-viva. ⚠️ **O cursor é VIVO** (mora no [`FactoryRuntime`]): ele
    /// muda a cada nascimento, e no componente registado faria um passo de undo por cópia.
    #[default]
    Cycle,
    /// Ao acaso, pela semente da fábrica.
    Random,
}

/// **A fábrica** — o componente registado, e só CONFIG.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Factory {
    /// O [`StableId`] da receita. **`0` = nenhuma**, e então ela não nasce (silêncio, nunca erro —
    /// a lei do alvo que não existe, do `SignalActions`).
    pub master: u64,
    /// O sinal que a faz nascer. **Vazio = nunca** — a lei da casa, e o painel di-lo.
    pub on_signal: String,
    /// Onde.
    pub at: SpawnAt,
    /// Quantas de cada vez. Cravado em [`BURST_MAX`] na leitura.
    pub burst: u32,
    /// Quantas podem estar vivas ao mesmo tempo. **`0` = sem limite.**
    ///
    /// ⚠️ **Ao bater no tecto ela NÃO nasce; ⛔ não mata a mais velha para caber.** Matar é do
    /// [`crate::lifetime`], e uma fábrica que decidisse sozinha matar o próprio filho mais velho
    /// tomaria uma decisão que ninguém autorou.
    pub alive_max: u32,
    /// Quantas ao todo, na corrida inteira. **`0` = sem limite.**
    pub total_max: u32,
    /// Sinal publicado quando nascem cópias — **um por tique, com a contagem dentro** (a lei do
    /// `cycles` da §11 e do `fires` do timer). Vazio = calada.
    pub on_spawned: String,
    /// Sinal publicado **uma vez**, quando ela chega ao `total_max`. Vazio = calada.
    pub on_exhausted: String,
    /// ⭐ **A semente** — lei §1.6 do levantamento (*determinismo é lei da casa; Spawner/AI com seed
    /// explícita exposta na UI*). O anel GGPO e o hash de 3 OS já pagam esse preço.
    ///
    /// ⚠️ **Ela é misturada com a IDENTIDADE da fábrica** ([`FactoryRuntime::semear`]): duas
    /// fábricas com a mesma semente autorada têm de dar sequências **diferentes**, senão duas
    /// chuvas lado a lado caem exactamente no mesmo sítio.
    pub seed: u64,
    /// ⭐⭐⭐ **A cópia sai APONTADA para onde a fábrica aponta** (o gatilho, 2026-09-18).
    ///
    /// ⚠️ **A ausência disto era um defeito MUDO e foi MEDIDA:** o [`onde`] devolvia só posições, e
    /// o [`Birth`] só levava `at` ⇒ a cópia nascia com a rotação **do MOLDE**. Para tudo o que cai
    /// (uma chuva, um destroço) isso não se vê; para um PROJÉCTIL é a diferença entre uma arma e
    /// uma decoração — o `ProjectileMotion` converte `initial_speed` **mais o ângulo do corpo** numa
    /// velocidade, uma vez só, no lançamento.
    ///
    /// ⚠️ **Desligado por omissão, e não por cautela:** ligado, ele torna a rotação do molde
    /// **inalcançável**, e uma chuva cujas gotas nascem todas viradas para onde o emissor calhou
    /// estar é pior do que uma que ignora o emissor. *Quem quer mira di-lo.*
    ///
    /// ⛔ **O ângulo é o de MUNDO** ([`crate::world_transform`]), não o local: uma arma pendurada
    /// num herói que roda tem de disparar para onde o HERÓI aponta, e a pose local dela é `0`.
    pub aim_from_spawner: bool,
    /// ⭐⭐⭐ **A ABERTURA do cone, em graus** — a caçadeira (`WeaponFire`, 2026-09-19).
    ///
    /// Cada cópia da rajada sai com um desvio sorteado em `±spread_deg/2` sobre a mira.
    /// **`0` = byte-idêntico ao de antes desta wave**, e a lei DEGENERA em vez de ramificar: com
    /// zero o gerador **não é tocado**, logo nem a sequência dele se desloca.
    ///
    /// ⚠️ **A medição que o pôs AQUI e não na arma:** um espalhamento precisa de um gerador com
    /// semente, determinista e que renasça no rebobinar — e esta struct tem um
    /// ([`FactoryRuntime::rng`], semeado com a identidade dela). Pô-lo na arma criaria um **segundo
    /// gerador**, que é a forma exacta do defeito que o `splitmix64` desta crate já saiu da fábrica
    /// para não repetir.
    ///
    /// ⛔ **E ele é irmão do [`Factory::burst`], não do gatilho:** `burst` diz *quantas de cada
    /// vez* e isto diz *com que abertura* — sem os dois juntos, oito chumbos saem empilhados num
    /// só. A medição do §5.0 lê os três ângulos de uma rajada de `3` como **o MESMO**.
    pub spread_deg: f32,
}

impl Default for Factory {
    fn default() -> Self {
        Self {
            master: 0,
            on_signal: String::new(),
            at: SpawnAt::Here,
            // ⚠️ **Uma, e não zero**: `0` é o valor que não faz nada, e um componente acabado de
            // acrescentar que nasce inerte lê-se como partido (a lição do `Lifetime`).
            burst: 1,
            alive_max: 0,
            total_max: 0,
            on_spawned: String::new(),
            on_exhausted: String::new(),
            seed: 1,
            // ⚠️ **Desligado**: a mira é uma escolha, e ligá-la por omissão tornaria a
            // rotação do molde inalcançável em toda fábrica que já existe.
            aim_from_spawner: false,
            spread_deg: 0.0,
        }
    }
}

/// **O estado VIVO de uma fábrica** — ⛔ **NÃO registado, de propósito** (a razão está no
/// [`crate::TimerRuntime`]: um contador que anda a cada tique dentro de um componente registado faz
/// cada quadro com entrada virar um passo de undo).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FactoryRuntime {
    /// Quantas ela já pôs no mundo nesta corrida.
    pub total: u32,
    /// Já disse que se esgotou? ⚠️ **Vivo**, como o `running` do timer: é um facto da corrida.
    pub exhausted_said: bool,
    /// O estado do gerador. `0` = ainda não semeado.
    pub rng: u64,
    /// A roda-viva do [`Pick::Cycle`].
    pub cursor: u32,
}

impl FactoryRuntime {
    /// Semeia o gerador com a semente autorada **misturada com a identidade** — ver [`Factory::seed`].
    fn semear(&mut self, seed: u64, id: u64) {
        if self.rng == 0 {
            // ⚠️ `| 1` porque `0` é o valor «não semeado»: uma semente autorada que calhasse
            // cancelar a identidade deixaria a fábrica a re-semear em todo tique.
            self.rng = (seed ^ id.rotate_left(32)) | 1;
        }
    }

    /// O próximo número. **splitmix64** — determinista, sem dependência, e o mesmo em toda máquina.
    fn proximo(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Um `f32` em `-0,5..0,5`.
    fn meio(&mut self) -> f32 {
        // 24 bits de mantissa: o maior inteiro que um `f32` representa sem saltos.
        let bits = (self.proximo() >> 40) as u32; // 24 bits
        (bits as f32 / f32::from(1u16 << 12) / f32::from(1u16 << 12)) - 0.5
    }
}

/// **Um nascimento pedido** — a shell instancia e põe a cópia aqui.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Birth {
    /// Quem a pediu (e o `by` do [`Spawned`] sai da identidade dela).
    pub factory: Entity,
    /// A receita.
    pub master: u64,
    /// A pose de MUNDO onde a cópia aterra.
    pub at: [f32; 2],
    /// **Para onde ela aponta**, em radianos de MUNDO. `None` = a cópia fica com a rotação do
    /// MOLDE, que é o caminho de omissão e é byte-idêntico ao de antes desta wave.
    pub aim: Option<f32>,
}

/// **O que um tique de fábricas produziu.**
///
/// ⚠️ **Três listas e não uma**: nascer, dizer que nasceu e dizer que acabou são factos de naturezas
/// diferentes, e o consumidor de cada um é outro (o instanciador, o outbox, o outbox).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FactoryTick {
    /// As cópias a pôr no mundo, pela ordem da identidade das fábricas.
    pub births: Vec<Birth>,
    /// `(fábrica, sinal, quantas)` — **um evento por fábrica**, com a contagem dentro.
    pub spawned: Vec<(Entity, String, u32)>,
    /// `(fábrica, sinal)` — ela chegou ao `total_max` **agora**.
    pub exhausted: Vec<(Entity, String)>,
}

/// Põe o estado vivo em quem tem fábrica e ainda não o tem. Idempotente (ver [`crate::lifetime::reconcile_lifetimes`]).
pub fn reconcile_factories(world: &mut World) {
    let novos: Vec<Entity> = world
        .query_filtered::<Entity, (With<Factory>, Without<FactoryRuntime>)>()
        .iter(world)
        .collect();
    for e in novos {
        world.entity_mut(e).insert(FactoryRuntime::default());
    }
}

/// **Quantas cópias vivas cada fábrica tem** — a varredura que o [`Spawned::by`] existe para tornar
/// possível sem um segundo índice.
#[must_use]
pub fn alive_by_factory(world: &mut World) -> std::collections::BTreeMap<u64, u32> {
    let mut out = std::collections::BTreeMap::new();
    let mut q = world.query::<&Spawned>();
    for s in q.iter(world) {
        *out.entry(s.by).or_insert(0) += 1;
    }
    out
}

/// ⭐⭐⭐ **O TIQUE DAS FÁBRICAS** — que cópias os sinais deste quadro pedem.
///
/// # ⚠️ A ordem é DETERMINISTA, e não é a da query
///
/// Ela sai do [`StableId`] de cada fábrica, como a do [`crate::resolve_signal_actions`]: a ordem do
/// arquétipo muda quando um componente é inserido, e um replay que instanciasse noutra ordem daria
/// outros ids às cópias.
///
/// # ⛔ Uma fábrica sem sinal, sem receita ou com uma tag vazia é SILÊNCIO
///
/// Nunca um erro de motor e nunca um nascimento na origem: é a lei do alvo que não existe. O que diz
/// ao artista o que falta é o **painel**.
#[must_use]
pub fn tick_factories(world: &mut World, tree: &TagTree, fired: &[&str]) -> FactoryTick {
    reconcile_factories(world);
    let mut out = FactoryTick::default();
    if fired.is_empty() {
        return out;
    }
    let vivos = alive_by_factory(world);
    // 1. As fábricas, com a identidade — a query segura `&mut`, então copia-se o que é preciso.
    let mut q = world.query::<(Entity, &Factory)>();
    let mut candidatas: Vec<(Entity, Factory)> = q
        .iter(world)
        .filter(|(_, f)| !f.on_signal.is_empty() && fired.contains(&f.on_signal.as_str()))
        .map(|(e, f)| (e, f.clone()))
        .collect();
    if candidatas.is_empty() {
        return out;
    }
    {
        let mundo: &World = world;
        candidatas.sort_by_key(|(e, _)| mundo.get::<StableId>(*e).map_or(u64::MAX, |s| s.0));
    }
    // 2. Cada uma, pela ordem da identidade.
    for (e, f) in candidatas {
        let id = world.get::<StableId>(e).map_or(0, |s| s.0);
        let mut estado = world.get::<FactoryRuntime>(e).copied().unwrap_or_default();
        estado.semear(f.seed, id);
        let quantas = quantas_nascem(&f, &estado, vivos.get(&id).copied().unwrap_or(0));
        if quantas > 0 && f.master != 0 {
            // ⚠️ **A mira sai da MESMA travessia da árvore que a posição** — uma segunda leitura
            // do `world_transform` poderia responder de um quadro diferente se alguém movesse o
            // dreno do reparent para o meio.
            let mira = f
                .aim_from_spawner
                .then(|| crate::world_transform(world, e).map(|t| t.rotation))
                .flatten();
            let poses = onde(world, tree, e, &f, &mut estado, quantas);
            let nasceram = u32::try_from(poses.len()).unwrap_or(u32::MAX);
            for at in poses {
                // ⚠️ **O sorteio é POR CÓPIA e DEPOIS do `onde`**, senão duas chumbadas da mesma
                // rajada partilhariam o desvio. ⛔ E com `spread_deg == 0` o gerador **não é
                // tocado**: é isso que torna o neutro byte-idêntico, incluindo a sequência que uma
                // `SpawnAt::Area` no mesmo tique vai ler.
                let aim = match mira {
                    Some(m) if f.spread_deg != 0.0 => {
                        Some(m + estado.meio() * f.spread_deg.to_radians())
                    }
                    outro => outro,
                };
                out.births.push(Birth {
                    factory: e,
                    master: f.master,
                    at,
                    aim,
                });
            }
            if nasceram > 0 {
                estado.total = estado.total.saturating_add(nasceram);
                if !f.on_spawned.is_empty() {
                    out.spawned.push((e, f.on_spawned.clone(), nasceram));
                }
            }
        }
        // ⚠️ **O esgotamento é uma TRAVESSIA**, como a morte: sem a bandeira, uma fábrica cheia
        // publicava o sinal dela a cada sinal de entrada, para sempre.
        if f.total_max > 0 && estado.total >= f.total_max && !estado.exhausted_said {
            estado.exhausted_said = true;
            if !f.on_exhausted.is_empty() {
                out.exhausted.push((e, f.on_exhausted.clone()));
            }
        }
        world.entity_mut(e).insert(estado);
    }
    out
}

/// Quantas cabem, dados os dois limites e o tecto do quadro.
fn quantas_nascem(f: &Factory, estado: &FactoryRuntime, vivas: u32) -> u32 {
    let mut n = f.burst.min(BURST_MAX);
    if f.alive_max > 0 {
        n = n.min(f.alive_max.saturating_sub(vivas));
    }
    if f.total_max > 0 {
        n = n.min(f.total_max.saturating_sub(estado.total));
    }
    n
}

/// As poses de mundo de `quantas` cópias.
fn onde(
    world: &World,
    tree: &TagTree,
    fabrica: Entity,
    f: &Factory,
    estado: &mut FactoryRuntime,
    quantas: u32,
) -> Vec<[f32; 2]> {
    let pose = crate::world_transform(world, fabrica);
    let base = pose.map_or([0.0, 0.0], |t| [t.translation.x, t.translation.y]);
    match f.at {
        SpawnAt::Here => vec![base; quantas as usize],
        SpawnAt::Area { w, h } => (0..quantas)
            .map(|_| {
                let dx = estado.meio() * w;
                let dy = estado.meio() * h;
                [base[0] + dx, base[1] + dy]
            })
            .collect(),
        SpawnAt::Tagged { tag, pick } => {
            let pontos = crate::tags::tagged(world, tree, TagId(tag));
            if pontos.is_empty() {
                return Vec::new();
            }
            (0..quantas)
                .map(|_| {
                    let i = match pick {
                        Pick::Cycle => {
                            let i = estado.cursor as usize % pontos.len();
                            estado.cursor = estado.cursor.wrapping_add(1);
                            i
                        }
                        Pick::Random => (estado.proximo() % pontos.len() as u64) as usize,
                    };
                    crate::world_transform(world, pontos[i])
                        .map_or(base, |t| [t.translation.x, t.translation.y])
                })
                .collect()
        }
    }
}

#[cfg(test)]
#[path = "factory_tests.rs"]
mod tests;
