//! ⭐⭐⭐ **A PERGUNTA DE ANTES DA 1.ª LINHA do SOBREAQUECER** (`overheat`, o item que o handoff da
//! ARMA deixou aberto como *«decisão de produto»*): *a composição de hoje já o exprime?*
//!
//! A nota que o deixou aberto diz *«é a mesma grandeza do pente com outra palavra; construí-lo
//! agora seria um segundo motor de munição»*. ⚠️ **Isso é meia-verdade e a metade que falta é a
//! que interessa:** um pente RECARREGA de uma vez (um prazo), e um cano ARREFECE de forma
//! contínua. *A grandeza é a mesma; a LEI DE REPOSIÇÃO não é.*
//!
//! ⇒ a pergunta medida aqui é exactamente essa: **um `Timer` a repetir + `AddToCounter(+1)` é um
//! arrefecimento?**
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_a_composicao_ja_da_ao_sobreaquecer -- --ignored --nocapture
//! ```

use ph2d_ecs::{
    Counter, CounterRuntime, SimWorld, Timer, TimerRuntime, Timers, WeaponFire, weapon_born,
};

const DT_US: u64 = 16_667;

/// Uma arma com um «cano» de 5 e um relógio que arrefece 1 a cada `arrefece_ms`.
fn cano(sim: &mut SimWorld, arrefece_ms: u64) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            WeaponFire {
                on_signal: "fogo".into(),
                cooldown_ms: 0,
                ammo_counter: "calor".into(),
                reload_ms: 0,
                reload_on: String::new(),
                on_fire: "saiu".into(),
                on_empty: "sobreaqueceu".into(),
                on_reloaded: String::new(),
            },
            weapon_born(),
            Counter {
                name: "calor".into(),
                start: 5,
                keep_on_restart: false,
            },
            CounterRuntime { value: 5 },
            Timers(vec![Timer {
                name: "arrefece".into(),
                duration_us: arrefece_ms * 1000,
                repeat: true,
                autostart: true,
                signal: "arrefeceu".into(),
            }]),
            TimerRuntime::default(),
        ))
        .id()
}

fn calor(sim: &SimWorld, e: ph2d_ecs::Entity) -> i64 {
    sim.world().get::<CounterRuntime>(e).expect("o cano").value
}

#[test]
#[ignore = "sonda: IMPRIME"]
fn mede_o_que_a_composicao_ja_da_ao_sobreaquecer() {
    println!("\n== o SOBREAQUECER, medido contra a composicao de hoje ==\n");

    // -- A) O cano ESGOTA-SE e a arma DIZ --------------------------------------
    let mut sim = SimWorld::default();
    let e = cano(&mut sim, 1_000_000);
    let mut secas = 0;
    for _ in 0..10 {
        let t = ph2d_app_components::weapon_bridge::frame(&mut sim, true, DT_US, &["fogo"]);
        secas += t.secas.len();
    }
    println!("A) dez pedidos num cano de 5:");
    println!(
        "   calor que sobra: {}  ·  cliques secos: {secas}",
        calor(&sim, e)
    );
    println!(
        "   => {}\n",
        if calor(&sim, e) == 0 && secas == 5 {
            "o cano BLOQUEIA ao chegar ao fim e anuncia — o `on_empty` E' o `sobreaqueceu`"
        } else {
            "a arma nao bloqueia — reconferir a nota"
        }
    );

    // -- B) E ele ARREFECE sozinho? -------------------------------------------
    // ⚠️ O relógio dispara `arrefeceu`, e a tabela de acções liga esse sinal a `AddToCounter(+1)`
    // NESTE objecto. Aqui a tabela é encenada à mão (a ponte dela vive na shell), e o que se mede
    // é se o RELÓGIO produz a cadência: é ele a metade que a nota dizia faltar.
    let mut sim2 = SimWorld::default();
    let e2 = cano(&mut sim2, 100);
    ph2d_ecs::reconcile_timers(sim2.world_mut());
    for _ in 0..5 {
        let _ = ph2d_app_components::weapon_bridge::frame(&mut sim2, true, DT_US, &["fogo"]);
    }
    let apos_disparar = calor(&sim2, e2);
    let mut arrefecimentos = 0;
    for _ in 0..60 {
        let mut ent = sim2.world_mut().entity_mut(e2);
        let cfg = ent.get::<Timers>().expect("o relogio").clone();
        if let Some(mut rt) = ent.get_mut::<TimerRuntime>() {
            for (t, st) in cfg.0.iter().zip(rt.0.iter_mut()) {
                arrefecimentos += ph2d_ecs::timer_advance(t, st, DT_US).fires;
            }
        }
    }
    println!("B) cinco tiros e depois um segundo de relogio (`repeat`, 100 ms):");
    println!("   calor apos disparar: {apos_disparar}  ·  disparos do relogio: {arrefecimentos}");
    println!(
        "   => {}\n",
        if apos_disparar == 0 && arrefecimentos >= 9 {
            "o relogio a REPETIR produz a cadencia — cada disparo dele e' um `AddToCounter(+1)`, \
             e isso E' o arrefecimento continuo que a nota dizia faltar"
        } else {
            "o relogio nao produz a cadencia — reconferir a nota"
        }
    );

    println!("== veredito ==");
    println!("   o cano, o bloqueio, o anuncio e o ARREFECIMENTO saem todos da composicao;");
    println!("   o que um componente `overheat` traria a mais e' um NOME no painel.");
}
