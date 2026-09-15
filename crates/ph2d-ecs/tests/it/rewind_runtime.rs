//! ⭐⭐⭐ **REBOBINAR É RENASCER** — os gates da porta que repõe o estado VIVO (TOP-20 #15, W0).
//!
//! O defeito e a medição que o achou estão no cabeçalho de [`ph2d_ecs::rewind_runtime`]. Aqui
//! mede-se o produto, e o censo impede que o quarto runtime seja esquecido — que é a forma como
//! esta família já mordeu **três** vezes na ponte da física.

use ph2d_ecs::{LifetimeRuntime, Timer, TimerRuntime, Timers, World};

fn cena(autostart: bool) -> (World, ph2d_ecs::Entity) {
    let mut w = World::new();
    let t = Timers(vec![Timer {
        name: "recarga".into(),
        duration_us: 1_000_000,
        repeat: false,
        autostart,
        signal: "pronto".into(),
    }]);
    let mut rt = TimerRuntime::default();
    ph2d_ecs::timer::reconcile(&t, &mut rt);
    let e = w.spawn((t, rt)).id();
    (w, e)
}

fn estado(w: &World, e: ph2d_ecs::Entity) -> ph2d_ecs::TimerState {
    w.get::<TimerRuntime>(e).expect("o relogio").0[0]
}

/// ⭐⭐⭐ **Um relógio que correu volta ao princípio** — o report que esta wave antecipou.
///
/// **Mutação que deve sangrar:** o corpo do `rewind_runtime_state` a devolver `0` sem tocar em nada.
#[test]
fn um_timer_corrido_volta_ao_principio_ao_rebobinar() {
    let (mut w, e) = cena(true);
    // Uma corrida: ele dispara e, sendo *one-shot*, PÁRA.
    let mut rt = w.get_mut::<TimerRuntime>(e).expect("o relogio");
    let cfg = Timer {
        name: "recarga".into(),
        duration_us: 1_000_000,
        repeat: false,
        autostart: true,
        signal: "pronto".into(),
    };
    let fora = ph2d_ecs::timer::advance(&cfg, &mut rt.0[0], 1_500_000);
    assert_eq!(
        fora.fires, 1,
        "a fixtura tem de ter CORRIDO, senao mede-se nada"
    );
    let depois = estado(&w, e);
    assert!(
        depois.elapsed_us > 0 || !depois.running,
        "a corrida tem de deixar marca: {depois:?}"
    );

    let tocados = ph2d_ecs::rewind_runtime::rewind_runtime_state(&mut w);
    assert!(
        tocados >= 1,
        "a porta tem de ter tocado no relogio: {tocados}"
    );

    let agora = estado(&w, e);
    assert_eq!(
        agora.elapsed_us, 0,
        "no tique 0 nenhum relogio correu — e um Reset devolve o tique 0"
    );
}

/// ⭐⭐ **E RENASCER não é «pôr a zero»** — a metade que um `Default` teria apagado.
///
/// Um `autostart` nasce **a correr**; `TimerState::default()` é **parado**, e o `reconcile` só arma
/// slots NOVOS ⇒ com a cura errada um Reset deixaria o relógio mudo para o resto da sessão.
///
/// **Mutação que deve sangrar:** `rt.0.extend(… TimerState::default())` no lugar do `born`.
#[test]
fn rebobinar_renasce_o_autostart_e_nao_o_deixa_parado() {
    for autostart in [true, false] {
        let (mut w, e) = cena(autostart);
        {
            let mut rt = w.get_mut::<TimerRuntime>(e).expect("o relogio");
            rt.0[0].elapsed_us = 900_000;
            rt.0[0].running = false;
        }
        ph2d_ecs::rewind_runtime::rewind_runtime_state(&mut w);
        let agora = estado(&w, e);
        assert_eq!(agora.elapsed_us, 0, "o progresso volta ao zero");
        assert_eq!(
            agora.running, autostart,
            "um autostart={autostart} tem de renascer como um slot NOVO, e nao como o `Default`"
        );
    }
}

/// ⭐ **A vida também** — hoje um no-op (a varredura da fábrica já as despejou), e a redundância
/// está DITA no cabeçalho da porta em vez de suposta.
#[test]
fn a_vida_tambem_passa_pela_porta() {
    let mut w = World::new();
    let e = w
        .spawn(LifetimeRuntime {
            elapsed_us: 7_000_000,
        })
        .id();
    ph2d_ecs::rewind_runtime::rewind_runtime_state(&mut w);
    assert_eq!(
        w.get::<LifetimeRuntime>(e).expect("a vida").elapsed_us,
        0,
        "uma vida que passou pelo rebobinar volta ao zero"
    );
}

/// ⭐⭐⭐ **O CENSO: nenhum estado vivo fica de fora da porta.**
///
/// ⚠️⚠️ **Ele existe porque a cura de um NOME repete-se e a de uma PORTA não.** A ponte da física
/// pagou esta família **três** vezes (o `drive_players` na W7, o `drive_topdown` no #13 e o
/// `drive_projectiles` no #14, os dois últimos descobertos por report do dono): um membro novo é
/// ligado a um sítio e esquecido no outro, **em silêncio**.
///
/// A régua é o **sufixo `Runtime` num `struct` desta crate** — que é exactamente como esta casa
/// nomeia «estado vivo, não registado» desde o `TimerRuntime`.
///
/// **Mutação que deve sangrar:** apagar o bloco das vidas do `rewind_runtime_state`.
#[test]
fn todo_estado_vivo_desta_crate_passa_pela_porta_do_rebobinar() {
    let dir = format!("{}/src", env!("CARGO_MANIFEST_DIR"));
    let porta = std::fs::read_to_string(format!("{dir}/rewind_runtime.rs")).expect("a porta");

    let mut achados: Vec<String> = Vec::new();
    for entrada in std::fs::read_dir(&dir).expect("o src") {
        let caminho = entrada.expect("entrada").path();
        if caminho.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let src = std::fs::read_to_string(&caminho).expect("ficheiro");
        for linha in src.lines() {
            let l = linha.trim();
            // ⚠️ Só a DECLARAÇÃO, e sem comentários: este repo explica as curas por escrito, e um
            // censo cru leria cada explicação como uma declaração.
            if let Some(resto) = l.strip_prefix("pub struct ")
                && let Some(nome) = resto.split(['(', ' ', '{', '<']).next()
                && nome.ends_with("Runtime")
            {
                achados.push(nome.to_string());
            }
        }
    }
    achados.sort();
    achados.dedup();
    // ⚠️ **Piso de POPULAÇÃO** — sem ele, renomear a convenção faz o censo varrer zero e
    // `esquecidos.is_empty()` fica trivialmente verdadeiro (a espécie de gate partido que fica
    // VERDE, §2.7 do HOWTO).
    assert!(
        achados.len() >= 2,
        "o censo tem de achar os estados vivos desta crate; achou {achados:?}"
    );

    let esquecidos: Vec<&String> = achados.iter().filter(|n| !porta.contains(*n)).collect();
    assert!(
        esquecidos.is_empty(),
        "estado(s) VIVO(s) que o rebobinar nao repoe: {esquecidos:?} — de {achados:?}. \
         Um Reset devolveria o mundo do tique 0 com a memoria de outra corrida, em silencio."
    );
}

/// ⭐⭐⭐ **A FÁBRICA esgotada volta a produzir, e a aleatória REPETE a corrida.**
///
/// Os dois membros que o censo acusou na **primeira** corrida dele — e o segundo é uma cura de
/// DETERMINISMO: com o `rng` a continuar de onde ficou, a 2.ª corrida de uma fábrica aleatória é
/// outra corrida, que é o report do dono sobre os projécteis um nível acima.
///
/// **Mutação que deve sangrar:** apagar o bloco das fábricas do `rewind_runtime_state`.
#[test]
fn uma_fabrica_gasta_volta_ao_principio_ao_rebobinar() {
    let mut w = World::new();
    let e = w
        .spawn(ph2d_ecs::FactoryRuntime {
            total: 99,
            exhausted_said: true,
            rng: 0x1234_5678_9abc_def0,
            cursor: 7,
        })
        .id();
    ph2d_ecs::rewind_runtime::rewind_runtime_state(&mut w);
    let rt = *w.get::<ph2d_ecs::FactoryRuntime>(e).expect("a fabrica");
    assert_eq!(rt, ph2d_ecs::FactoryRuntime::default());
    assert_eq!(
        rt.rng, 0,
        "`rng = 0` e' «por semear» — e' isso que faz a 2.a corrida REPETIR a 1.a"
    );
    assert_eq!(
        rt.total, 0,
        "uma fabrica com Max Total gasto tem de voltar a produzir"
    );
}

/// ⭐⭐ **A CÂMERA é APAGADA, e não posta a zero** — a lei que um `Default` teria quebrado.
///
/// O `center` de uma câmera nasce da **pose autorada**, e essa resposta vive na shell
/// (`ensure_runtime`). Escrever um `Default` aqui poria toda câmera na ORIGEM ao rebobinar.
///
/// **Mutação que deve sangrar:** trocar o `remove::<CameraRuntime>()` por `*rt = Default::default()`.
#[test]
fn a_camera_e_apagada_para_renascer_da_pose_autorada() {
    let mut w = World::new();
    let e = w
        .spawn(ph2d_ecs::CameraRuntime {
            center: [30.0, 40.0],
            anchor: [30.0, 40.0],
            last_target: Some([31.0, 41.0]),
            velocity: [5.0, 0.0],
            settled: true,
        })
        .id();
    ph2d_ecs::rewind_runtime::rewind_runtime_state(&mut w);
    assert!(
        w.get::<ph2d_ecs::CameraRuntime>(e).is_none(),
        "o vivo da camera SAI, para a porta que sabe a pose autorada o recriar — um `Default` \
         poria a camera na origem, e um `center` mantido faria a corrida seguinte comecar onde a \
         anterior acabou"
    );
}
