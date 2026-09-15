//! ⭐⭐⭐ **REBOBINAR É RENASCER** — a porta única que repõe o estado VIVO da família `Logic`.
//!
//! # ⛔⛔ O defeito que esta porta cura, e como ele foi medido
//!
//! O molde desta casa separa **CONFIG** (componente registado, viaja no ficheiro) de **VIVO**
//! (componente **não** registado, sem `Serialize` — a cerca é o TIPO). É o desenho certo e está
//! declarado no [`crate::TimerRuntime`]. ⚠️ **Mas ele tem uma consequência que ninguém escreveu:
//! o que o undo não fotografa, o undo também não REPÕE.**
//!
//! Medido pela **superfície da API** (2026-09-15), que é mais forte que um `grep`: o timer exporta
//! `advance` · `reconcile` · `start` · `stop` e **nenhuma porta de reposição** — *ninguém podia
//! repor o relógio ao rebobinar, mesmo que quisesse*. A única varredura de rebobinar da shell é a
//! `sweep_spawned` das cópias da fábrica.
//!
//! ⇒ **um `Timer` que correu continuava corrido depois de um Reset.** É a **mesma família** do
//! report do dono sobre o rewind dos projécteis (2026-09-15, `bridge::controllers`), um nível
//! acima — e o `StateMachineRuntime` do TOP-20 #15 herdá-la-ia por construção.
//!
//! # ⚠️ A lei, e porque ela não é «pôr a zero»
//!
//! **Rebobinar é RENASCER**: o vivo volta ao que um slot ACABADO DE CRIAR recebe — e para um timer
//! com `autostart` isso é **a correr**, não parado. Pôr `TimerState::default()` deixaria um
//! `autostart` mudo para sempre, porque o [`crate::timer::reconcile`] só arma slots **novos**.
//!
//! ⇒ a resposta *«o que um slot recém-nascido recebe»* existe **uma vez só**
//! ([`crate::timer::born`]) e tem **dois leitores**: o `reconcile` e esta porta. *Uma lei escrita
//! em dois sítios ainda não é uma lei.*
//!
//! # ⚠️ A redundância com a varredura da fábrica está DITA, não suposta
//!
//! O [`crate::LifetimeRuntime`] só mede alguma coisa numa entidade com [`crate::Spawned`], e essas
//! são despejadas pela `sweep_spawned` no mesmo instante. Repô-lo aqui é, hoje, **um no-op** — e
//! fica porque a redundância declarada é barata e porque o dia em que uma vida for autorável à mão
//! já não tem de se lembrar disto (o mesmo argumento do `release_grab` no `bridge::rewind`).

use crate::{
    CameraRuntime, Entity, FactoryRuntime, LifetimeRuntime, StateMachine, StateMachineRuntime,
    TimerRuntime, Timers, World,
};

/// **Repõe o estado vivo de toda a gente, como no tique 0.** Devolve **quantos componentes** foram
/// tocados — o número que um diagnóstico imprime e que um gate lê.
///
/// ⚠️ **Chamada pelo INVARIANTE do rebobinar da shell** (`relógio no início e parado`), nunca por
/// um gancho num botão: o transporte tem mais de um caminho até ao zero.
///
/// ⚠️⚠️ **Cada membro tem o SEU «nascer», e eles não são o mesmo** — foi a primeira corrida do censo
/// que o mostrou, ao acusar dois membros que esta porta não conhecia:
///
/// | vivo | o que «nascer» é | porquê |
/// |---|---|---|
/// | [`TimerRuntime`] | [`crate::timer::born`] por slot | um `autostart` nasce **a correr**; o `Default` é parado |
/// | [`LifetimeRuntime`] | `Default` | zero microssegundos vividos |
/// | [`FactoryRuntime`] | `Default` | ⭐ **`rng: 0` quer dizer «por semear»** ⇒ a corrida seguinte **repete** a primeira |
/// | [`StateMachineRuntime`] | [`crate::state_machine::born`] | volta ao estado **inicial**, e `started = false` fá-lo anunciar a entrada outra vez |
/// | [`CameraRuntime`] | ⭐⭐ **APAGAR o componente** | o `ensure_runtime` da shell recria-o **da pose autorada**; um `Default` poria a câmera na ORIGEM |
pub fn rewind_runtime_state(world: &mut World) -> usize {
    let mut n = 0;

    // ── Os RELÓGIOS ──────────────────────────────────────────────────────────
    // ⚠️ A config entra na conta: o que um slot recebe ao nascer depende do `autostart` DELE.
    let mut q = world.query::<(&Timers, &mut TimerRuntime)>();
    for (cfg, mut rt) in q.iter_mut(world) {
        rt.0.clear();
        rt.0.extend(cfg.0.iter().map(crate::timer::born));
        n += 1;
    }

    // ── As VIDAS ─────────────────────────────────────────────────────────────
    let mut q = world.query::<&mut LifetimeRuntime>();
    for mut rt in q.iter_mut(world) {
        *rt = LifetimeRuntime::default();
        n += 1;
    }

    // ── OS CÉREBROS ──────────────────────────────────────────────────────────
    // ⭐ **O membro que nasceu DEPOIS da porta**, e que passou por ela porque o censo o obrigou —
    // que é exactamente o que esta wave existiu para conseguir. A config entra na conta: um cérebro
    // renasce no estado INICIAL dele, e com `started = false` ele volta a anunciar a entrada.
    let mut q = world.query::<(&StateMachine, &mut StateMachineRuntime)>();
    for (cfg, mut rt) in q.iter_mut(world) {
        *rt = crate::state_machine::born(cfg);
        n += 1;
    }

    // ── As FÁBRICAS ──────────────────────────────────────────────────────────
    // ⭐⭐⭐ **É aqui que mora a metade DETERMINISTA da cura:** o `total` gasto fazia uma fábrica com
    // `Max Total` **recusar-se a produzir na 2.ª corrida**, e o `rng` a continuar de onde ficara
    // fazia a 2.ª corrida de uma fábrica aleatória ser **outra corrida**. Com `rng: 0` (= por
    // semear) ela volta a semear-se da semente AUTORADA misturada com a identidade.
    let mut q = world.query::<&mut FactoryRuntime>();
    for mut rt in q.iter_mut(world) {
        *rt = FactoryRuntime::default();
        n += 1;
    }

    // ── As CÂMERAS ───────────────────────────────────────────────────────────
    // ⛔⛔ **Aqui NÃO se escreve um `Default`, e a diferença é visível:** o `center` de uma câmera
    // nasce da **pose autorada** dela (`render_loop::camera_2d::ensure_runtime`), e um `Default`
    // poria toda câmera na ORIGEM ao rebobinar. ⇒ apaga-se o componente e deixa-se a porta que já
    // sabe a resposta recriá-lo — *uma segunda resposta a «onde uma câmera começa» divergiria da
    // primeira no dia em que uma delas mudasse*.
    //
    // ⚠️ E isso devolve de graça o `settled = false`, que é o `reset_smoothing()` do oráculo: sem
    // ele a câmera **viajaria** do sítio da corrida anterior até ao alvo.
    let camaras: Vec<Entity> = world
        .query::<(Entity, &CameraRuntime)>()
        .iter(world)
        .map(|(e, _)| e)
        .collect();
    for e in camaras {
        if let Ok(mut ent) = world.get_entity_mut(e) {
            ent.remove::<CameraRuntime>();
            n += 1;
        }
    }

    n
}
