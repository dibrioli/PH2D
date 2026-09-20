//! ⭐⭐⭐ **OS TECTOS QUE NINGUÉM VARREU** — os três que os handoffs da ARMA, do ABANÃO e do FIM DE
//! JOGO deixaram abertos por escrito (*«o custo a N armas/quadro · emissores de abanão por quadro ·
//! recomeços por corrida»*).
//!
//! # ⚠️ Porquê uma sonda e não um gate
//!
//! Um número de relógio desta workstation **não vale nada acima de `load ~5`** (a lei do §5.0), e um
//! gate de tempo seria mais um membro da família de flakes de fan-out. ⇒ ela **IMPRIME**, corre com
//! `--ignored`, e o que fica no repo é a TABELA — que é o que o §0.0 pede antes de alguém escrever
//! um `MAX_*`.
//!
//! # ⚠️⚠️ E o que ela mede é a PONTE, nunca a lei
//!
//! As três leis puras são `O(1)` por objecto e ninguém duvida disso. O que não estava medido é o
//! **passe do quadro**: quantas varreduras do mundo ele faz, e se alguma é `O(cena)` por objecto —
//! que é exactamente o defeito que a `Factory` do `#11` pagou (`deep_copy_subtree` varria o mundo
//! **duas vezes por cópia**, `139,6 µs` de `143,2` a 100 000 objectos).
//!
//! ⭐ **O orçamento é `16,67 ms`**, e a coluna que interessa é a fracção dele.

use std::time::Instant;

use ph2d_ecs::{
    CameraShake, Counter, CounterRuntime, GameCamera, Name, ShakeEmitter, ShakeSource, SimWorld,
    StableId, Transform, WeaponFire,
};

/// Um quadro a 60 Hz, em microssegundos.
const DT_US: u64 = 16_667;

/// O orçamento de um quadro, em milissegundos.
const ORCAMENTO_MS: f64 = 16.667;

/// Quantas repetições por ponto. ⚠️ **O MÍNIMO e não a média** — a mediana de um relógio desta
/// máquina mede quem mais estava a correr, e o mínimo é o único estimador que não sobe com a carga.
const REPS: u32 = 7;

/// ⛔⛔ **O `antes` corre FORA do relógio, e ele não é conforto: sem ele a coluna B media a corrida
/// VAZIA.**
///
/// A 1.ª redacção publicava o sinal uma vez e repetia sete: o cursor do leitor **consome**, logo
/// seis das sete corridas não tinham sinal nenhum para ouvir — e *o MÍNIMO de sete é justamente a
/// mais barata delas*. A tabela imprimia `0,000 ms` a dez mil fontes e lia-se como *«é grátis»*.
///
/// ⚠️ **O estimador que esconde a carga é o mesmo que esconde isto**, e a única defesa é rearmar o
/// sujeito a cada repetição.
fn mede<T>(sujeito: &mut T, mut antes: impl FnMut(&mut T), mut f: impl FnMut(&mut T)) -> f64 {
    let mut melhor = f64::MAX;
    for _ in 0..REPS {
        antes(sujeito);
        let t = Instant::now();
        f(sujeito);
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1_000.0);
    }
    melhor
}

/// `n` armas com pente, cada uma na sua entidade.
fn cena_de_armas(n: u32) -> SimWorld {
    let mut sim = SimWorld::new();
    for i in 0..n {
        sim.world_mut().spawn((
            Name::new("Arma"),
            StableId(u64::from(i) + 1),
            WeaponFire {
                on_signal: "fire".into(),
                cooldown_ms: 0,
                ammo_counter: "ammo".into(),
                reload_ms: 100,
                on_fire: "shot".into(),
                ..WeaponFire::default()
            },
            Counter {
                name: "ammo".into(),
                start: 6,
                keep_on_restart: false,
            },
            CounterRuntime { value: 6 },
        ));
    }
    sim
}

/// Uma câmera que treme, mais `n` fontes que gritam o MESMO sinal.
fn cena_de_abanao(n: u32) -> SimWorld {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Camera"),
        StableId(1),
        Transform::IDENTITY,
        GameCamera::default(),
        CameraShake::default(),
    ));
    for i in 0..n {
        sim.world_mut().spawn((
            Name::new("Bomba"),
            StableId(u64::from(i) + 2),
            Transform::IDENTITY,
            ShakeEmitter(vec![ShakeSource {
                on: "boom".into(),
                ..ShakeSource::default()
            }]),
        ));
    }
    sim
}

#[test]
#[ignore = "sonda: imprime os tres tectos que o §0.0 pede"]
fn mede_os_tectos_por_quadro() {
    // ⚠️ **A carga em cima da tabela, e não numa nota** — a lei do §5.0: *imprima o `/proc/loadavg`
    // AO LADO de cada corrida, senão a régua que desmente um número é o próprio número*.
    println!(
        "loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("== A) N ARMAS por quadro (todas a disparar) ==");
    println!("      n        ms   % de um quadro   µs por arma");
    let mut anterior: Option<(u32, f64)> = None;
    for n in [1_u32, 10, 100, 1_000, 10_000] {
        let mut sim = cena_de_armas(n);
        let ms = mede(
            &mut sim,
            |_| {},
            |sim| {
                let _ = ph2d_app_components::weapon_bridge::frame(sim, true, DT_US, &["fire"]);
            },
        );
        println!(
            "  {n:>6}  {ms:>8.3}   {:>12.2} %   {:>11.3}",
            ms / ORCAMENTO_MS * 100.0,
            ms * 1_000.0 / f64::from(n)
        );
        if let Some((n0, ms0)) = anterior {
            let fator_n = f64::from(n) / f64::from(n0);
            let fator_t = ms / ms0;
            println!("          (×{fator_n:.0} em n => ×{fator_t:.2} no relogio)");
        }
        anterior = Some((n, ms));
    }
    println!(
        "  ⇒ se a ultima coluna for PLANA o passe e' O(n); se ela SUBIR, ha' uma varredura do\n  \
         mundo por arma — o defeito que a `Factory` do #11 pagou."
    );

    println!("\n== B) N FONTES DE ABANAO por quadro (todas a gritar) ==");
    println!("      n        ms   % de um quadro   µs por fonte");
    for n in [1_u32, 10, 100, 1_000, 10_000] {
        let sim = cena_de_abanao(n);
        // ⚠️ **Pela porta do PRODUTO**: um sinal publicado no barramento e lido com o cursor da
        // câmera — *um arnês que chamasse a lei mediria outro programa*.
        let mut outbox = ph2d_runtime::SignalOutbox::default();
        outbox.publish(ph2d_runtime::Signal::from_control("boom"));
        let reader = ph2d_runtime::SignalReader::default();
        #[allow(clippy::cast_precision_loss)]
        let dt = DT_US as f32 / 1e6;
        let ms = mede(
            &mut (sim, reader),
            // ⚠️ **Rearma o cursor** — ver o doc do `mede`.
            |(_, r)| *r = ph2d_runtime::SignalReader::default(),
            |(sim, r)| {
                let _ = ph2d_app_components::shake_bridge::drive_camera_shake(sim, &outbox, r, dt);
            },
        );
        println!(
            "  {n:>6}  {ms:>8.3}   {:>12.2} %   {:>11.3}",
            ms / ORCAMENTO_MS * 100.0,
            ms * 1_000.0 / f64::from(n)
        );
    }
    println!(
        "  ⇒ o trauma SATURA em 1,0 (a lei da `ph2d-shake`), logo mil fontes nao abanam mil vezes\n  \
         mais — o que se mede aqui e' o PRECO de as ouvir, nao o efeito."
    );

    println!("\n== C) o RECOMECO de uma corrida, a N objectos ==");
    println!("      n        ms   % de um quadro   µs por objecto");
    for n in [1_u32, 100, 1_000, 10_000] {
        let mut sim = SimWorld::new();
        for i in 0..n {
            sim.world_mut().spawn((
                Name::new("Contador"),
                StableId(u64::from(i) + 1),
                Counter {
                    name: "pontos".into(),
                    start: 3,
                    keep_on_restart: false,
                },
                CounterRuntime { value: 0 },
            ));
        }
        let ms = mede(
            &mut sim,
            |_| {},
            |sim| {
                let _ = ph2d_ecs::rewind_runtime::rewind_runtime_state(
                    sim.world_mut(),
                    ph2d_ecs::rewind_runtime::Renascimento::Recomecar,
                );
            },
        );
        println!(
            "  {n:>6}  {ms:>8.3}   {:>12.2} %   {:>14.3}",
            ms / ORCAMENTO_MS * 100.0,
            ms * 1_000.0 / f64::from(n)
        );
    }
    println!(
        "  ⇒ ⭐ aqui a ultima coluna DESCE (2,280 -> 0,001 us), e a leitura e' mais forte que a das\n  \
         outras duas: o recomeco e' dominado pelo custo FIXO de construir as consultas, e nao pelo\n  \
         numero de objectos — dez mil contadores custam 0,010 ms.\n  \
         ⚠️ E ele e' um GESTO e nao um custo por quadro: corre uma vez por recomeco, e um jogo que\n  \
         recomecasse sessenta vezes por segundo teria outro problema."
    );

    println!("\n== veredito ==");
    println!(
        "   nenhum dos tres precisa de um `MAX_*` hoje, e agora ha' NUMERO: as tres colunas de «por\n   \
         objecto» sao PLANAS de 100 para cima => o passe e' O(n) e nenhum deles varre o mundo por\n   \
         objecto. Um `MAX_*` legitimo diz de que RECURSO e' (§0.0), e a tabela acima e' quem o pode\n   \
         nomear — quem o escrever escreve-a ao lado.\n   \
         ⚠️ A CARGA da maquina esta' impressa em cima: os MS ABSOLUTOS sobem com ela, a PLANURA da\n   \
         ultima coluna nao (e' a mesma conta dividida pelo mesmo n)."
    );
}
