//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do suplente #25 (`CameraShake` + `ShakeEmitter`)**:
//! *a composição de hoje — `Signal` + `SignalActions` + `Tween` + a câmera do jogo — já exprime
//! «isto explodiu, e a vista tremeu»?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime.** Nesta linha a pergunta já REESCREVEU quatro entregas: o **#3 `SensorZone`** estava
//! fechado por composição, a **W5 do #21** descobriu que o pintor já existia, o **#24 `Health`**
//! deixou de ser um componente, e o **#23** confirmou que o concorrente era real e não chegava.
//!
//! ⚠️ **Ela mora AQUI, na crate de FAMÍLIA, porque esta é a única que vê os três lados**: o
//! `ph2d-ecs` (a câmera, os verbos, o gerador da fábrica), o `ph2d-runtime` (a origem de um sinal) e
//! o `ph2d-tween` (o concorrente).
//!
//! ⚠️ **Sonda, não gate.** Corre com `--ignored` e IMPRIME.
//!
//! ```text
//! cargo test -p ph2d-app-components --test it \
//!     mede_o_que_a_composicao_ja_da_ao_abanao -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **A tabela de acções sabe dizer «treme»?**
//! B) **Algum canal de TWEEN alcança a vista da câmera?** — o concorrente mais forte.
//! C) **Um sinal sabe QUEM gritou?** — sem isso não há distância, e sem distância não há falloff.
//! D) **Há gerador determinístico na casa, e quantos consumidores ele tem?**
//! E) **A câmera já RENASCE ao rebobinar?**

use ph2d_ecs::{
    CameraRuntime, GameCamera, Name, SignalVerb, SimWorld, Timer, TimerRuntime, Timers, Transform,
    Tweens,
};
use ph2d_runtime::SignalOrigin;
use ph2d_tween::{Canal, Tween};

/// O manifesto da fundação — lido em tempo de COMPILAÇÃO, logo não pode envelhecer sem o ficheiro
/// mudar.
const FACTORY_RS: &str = include_str!("../../../ph2d-ecs/src/factory.rs");

#[test]
#[ignore = "sonda: imprime a medição do §5.0, não afirma uma barra"]
fn mede_o_que_a_composicao_ja_da_ao_abanao() {
    eprintln!("\n════ §5.0 — O QUE A COMPOSIÇÃO JÁ DÁ AO ABANÃO (suplente #25) ════\n");

    // ── (A) A tabela de acções sabe dizer «treme»? ────────────────────────────────────────────
    let verbos: Vec<&str> = SignalVerb::ALL
        .iter()
        .map(|v| ph2d_i18n::tr(v.label_key()))
        .collect();
    let abana = verbos
        .iter()
        .any(|v| v.to_lowercase().contains("shake") || v.to_lowercase().contains("camera"));
    eprintln!("(A) OS VERBOS DA TABELA DE ACÇÕES");
    eprintln!("    são {} : {verbos:?}", verbos.len());
    eprintln!("    algum abana a câmera? ......... {abana}");
    eprintln!("    ⇒ um sinal chega a toda a parte e NÃO sabe dizer «treme».\n");

    // ── (B) Algum canal de TWEEN alcança a vista? ─────────────────────────────────────────────
    //
    // O concorrente a sério: um tween de pose sobre a própria câmera. Ele MOVE o `Transform` — e a
    // vista não vem daí.
    let mut sim = SimWorld::new();
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        autostart: true,
        ..Timer::default()
    }]);
    let rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    let camera = sim
        .world_mut()
        .spawn((
            Name::new("Camera"),
            Transform::IDENTITY,
            GameCamera::default(),
            CameraRuntime::default(),
            Tweens(vec![Tween::linear(Canal::PositionX, 0.0, 4.0)]),
            cfg,
            rt,
        ))
        .id();
    {
        let mut rt = sim.world_mut().get_mut::<TimerRuntime>(camera).unwrap();
        rt.0[0].elapsed_us = 500_000;
    }
    let mut drive = ph2d_preview_drive::PreviewDrive::default();
    let escritas = ph2d_app_components::tween_bridge::drive_tweens(&mut sim, &mut drive, &[]);
    let pose = sim.world().get::<Transform>(camera).unwrap().translation.x;
    let vista = sim.world().get::<CameraRuntime>(camera).unwrap().center;
    let canais: Vec<&str> = Canal::ALL
        .iter()
        .map(|c| ph2d_i18n::tr(c.label_key()))
        .collect();
    eprintln!("(B) UM TWEEN SOBRE A PRÓPRIA CÂMERA (o concorrente)");
    eprintln!(
        "    canais que existem ............ {} : {canais:?}",
        canais.len()
    );
    eprintln!("    escritas do tween ............. {escritas}");
    eprintln!("    o `Transform` dela mexeu-se? .. x = {pose:.4}");
    eprintln!("    …e o CENTRO da vista? ......... {vista:?}");
    eprintln!(
        "    ⇒ a vista vem do `CameraRuntime`, que NÃO é um `Transform`: nenhum canal a alcança.\n"
    );

    // ── (C) Um sinal sabe QUEM gritou? ────────────────────────────────────────────────────────
    //
    // ⭐ Sem o sujeito não há posição; sem posição não há DISTÂNCIA, e é a distância que separa um
    // abanão posicional de um abanão global.
    let bomba = sim
        .world_mut()
        .spawn((Name::new("Bomba"), Transform::IDENTITY))
        .id();
    let origens: [(&str, SignalOrigin); 5] = [
        ("timeline", SignalOrigin::Timeline { t: 0.0 }),
        ("controlo", SignalOrigin::Control),
        (
            "contacto",
            SignalOrigin::Contact {
                source: ph2d_runtime::EntityBits(bomba.to_bits()),
                other: ph2d_runtime::EntityBits(camera.to_bits()),
            },
        ),
        (
            "morte",
            SignalOrigin::Death {
                source: ph2d_runtime::EntityBits(bomba.to_bits()),
            },
        ),
        (
            "relógio",
            SignalOrigin::Timer {
                source: ph2d_runtime::EntityBits(bomba.to_bits()),
                fires: 1,
            },
        ),
    ];
    let com_sujeito = origens.iter().filter(|(_, o)| o.quem().is_some()).count();
    eprintln!("(C) O SINAL SABE QUEM GRITOU?");
    for (nome, o) in &origens {
        eprintln!("    {nome:<10} → quem() = {:?}", o.quem());
    }
    eprintln!(
        "    {com_sujeito} de {} das amostradas têm sujeito",
        origens.len()
    );
    eprintln!(
        "    ⇒ a DISTÂNCIA é derivável: o sujeito tem `Transform`, e o falloff é uma subtracção.\n"
    );

    // ── (D) Há gerador determinístico, e quantos consumidores ele tem? ────────────────────────
    //
    // A fábrica (#11) já sorteia, e a mesma semente dá a mesma corrida — mas o gerador é PRIVADO
    // dela. ⚠️ *Um gerador com um consumidor e sem porta é a segunda cópia à espera de ser escrita.*
    let privado =
        FACTORY_RS.contains("fn proximo(&mut self)") && !FACTORY_RS.contains("pub fn proximo");
    let splitmix = FACTORY_RS.contains("0x9E37_79B9_7F4A_7C15");
    eprintln!("(D) O GERADOR DETERMINÍSTICO");
    eprintln!("    a casa tem splitmix64? ........ {splitmix}");
    eprintln!("    ele é PRIVADO da fábrica? ..... {privado}");
    eprintln!(
        "    ⇒ ele existe, é determinista e tem UM consumidor — copiá-lo seria a segunda cópia.\n"
    );

    // ── (E) A câmera renasce ao rebobinar? ────────────────────────────────────────────────────
    {
        let mut r = sim.world_mut().get_mut::<CameraRuntime>(camera).unwrap();
        r.center = [9.0, 9.0];
        r.settled = true;
    }
    let tocados = ph2d_ecs::rewind_runtime::rewind_runtime_state(sim.world_mut());
    let sobrou = sim.world().get::<CameraRuntime>(camera).is_some();
    eprintln!("(E) REBOBINAR É RENASCER");
    eprintln!("    componentes tocados ........... {tocados}");
    eprintln!("    a `CameraRuntime` sobreviveu? . {sobrou}");
    eprintln!("    ⇒ o censo existe, e um estado vivo NOVO tem de entrar nele.\n");

    eprintln!("════ FIM DA MEDIÇÃO ════\n");
}
