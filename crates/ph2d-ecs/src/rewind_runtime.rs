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
    CameraRuntime, Counter, CounterRuntime, CounterWatch, CounterWatchRuntime, Entity,
    FactoryRuntime, LifetimeRuntime, StateMachine, StateMachineRuntime, TimerRuntime, Timers,
    WeaponRuntime, World,
};

/// ⭐⭐⭐ **PORQUE é que o vivo está a renascer** — e a distinção existe porque há **duas**
/// travessias do zero, com respostas diferentes numa grandeza.
///
/// Ela entra na **assinatura** da [`rewind_runtime_state`] de propósito: um parâmetro por omissão
/// deixaria o chamador novo herdar a resposta de outro, que é exactamente o defeito que esta porta
/// existe para não ter. *Esquecer o motivo é erro de compilação.*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Renascimento {
    /// **O transporte voltou ao início** — o botão *Rewind*, um scrub da régua até zero.
    ///
    /// Aqui o destino é o **DOCUMENTO**: tudo volta ao que o ficheiro diz, sem excepção. ⛔ É por
    /// isso que o [`Counter::keep_on_restart`] **não** é honrado neste motivo — um contador que
    /// sobrevivesse a um rewind seria estado de uma corrida a contaminar o estado autorado.
    Rebobinar,
    /// **Um verbo pediu outra corrida** ([`crate::SignalVerb::RestartRun`]).
    ///
    /// Aqui o relógio continua a andar e o destino é *«outra vez»*, não *«como estava gravado»* —
    /// e é o único motivo em que o artista escolhe o que ATRAVESSA.
    Recomecar,
}

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
/// | [`CounterRuntime`] | ⭐ o **`start` da config** | a primeira espécie que LÊ a config: um `Default` poria todos a zero e apagaria as três vidas que o artista autorou. ⭐⭐ **E a única que lê o MOTIVO**: com [`Counter::keep_on_restart`] ele atravessa um recomeço e **não** um rebobinar |
/// | [`CounterWatchRuntime`] | [`crate::counter_watch::born`] por slot | ⭐⭐ `held = false` **re-arma a aresta**: sem isso a 2.ª corrida nunca voltaria a anunciar a morte, porque a condição já estava satisfeita quando a 1.ª acabou |
/// | [`WeaponRuntime`] | [`crate::weapon::born`] | ⭐ «pronta a disparar»; ⛔ o PENTE **não** é reposto aqui — ele é um [`Counter`], e a linha acima já o enche |
pub fn rewind_runtime_state(world: &mut World, motivo: Renascimento) -> usize {
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

    // ── Os CONTADORES ────────────────────────────────────────────────────────
    // ⭐ **Nascer aqui é o `start` da CONFIG** — a primeira espécie desta porta que não é uma
    // constante. Sem isto a 2.ª corrida começaria com os pontos da primeira.
    //
    // ⭐⭐⭐ **E é a ÚNICA espécie desta tabela que lê o MOTIVO** (`keep_on_restart`, 2026-09-20):
    // *«outra vida, mesma pontuação»*. ⚠️ A cerca é o motivo e não um `if` no campo — num
    // `Rebobinar` o destino é o DOCUMENTO e nada o atravessa, e escrever as duas travessias iguais
    // faria a régua arrastada até ao princípio mostrar a pontuação da corrida anterior.
    let mut q = world.query::<(&Counter, &mut CounterRuntime)>();
    for (cfg, mut rt) in q.iter_mut(world) {
        if motivo == Renascimento::Recomecar && cfg.keep_on_restart {
            continue;
        }
        rt.value = cfg.start;
        n += 1;
    }

    // ── As VIGIAS ────────────────────────────────────────────────────────────
    // ⭐⭐ **Re-armar a ARESTA é o trabalho todo.** Uma corrida que acabou com as vidas a zero
    // deixou a regra `AtMost 0` com `held = true`; sem passar por aqui, a 2.ª corrida começaria
    // com a condição «já satisfeita» e **nunca mais anunciaria a morte** — o mesmo defeito que o
    // `started = false` do cérebro cura um parágrafo acima, e que a fábrica pagou com o `total`.
    let mut q = world.query::<(&CounterWatch, &mut CounterWatchRuntime)>();
    for (cfg, mut rt) in q.iter_mut(world) {
        rt.0.clear();
        rt.0.resize(cfg.0.len(), crate::counter_watch::born());
        n += 1;
    }

    // ── As ARMAS ─────────────────────────────────────────────────────────────
    // ⭐ **Nascer é «pronta a disparar»**: a cadência zera e uma recarga a meio é CANCELADA — senão
    // a 2.ª corrida começaria a meio de uma animação de recarregar que ninguém pediu.
    // ⚠️ **O PENTE não é reposto aqui**, e a ausência é a lei: ele é um [`Counter`], e o bloco dos
    // contadores acima já o enche do `start`. *Repô-lo nos dois sítios seria a segunda resposta a
    // «quantas balas tem uma arma que renasce».*
    let mut q = world.query::<&mut WeaponRuntime>();
    for mut rt in q.iter_mut(world) {
        *rt = crate::weapon::born();
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

    // ── O ABANÃO ─────────────────────────────────────────────────────────────
    // ⚠️ **Aqui o `Default` É a resposta certa**, ao contrário da irmã logo acima: o vivo do abanão
    // não deriva nada da pose nem da config — trauma zero e relógio zero é literalmente *«esta
    // câmera não está a tremer»*. ⛔ Sem esta entrada, rebobinar a meio de uma explosão deixaria a
    // vista a acabar de tremer o abanão da corrida ANTERIOR, com o relógio no zero.
    let mut q = world.query::<&mut crate::CameraShakeRuntime>();
    for mut rt in q.iter_mut(world) {
        *rt = crate::CameraShakeRuntime::default();
        n += 1;
    }

    n
}
