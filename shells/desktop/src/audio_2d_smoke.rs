//! ⭐⭐⭐ **Smoke do som de cena** (TOP-20 #4). `PH2D_AUDIO_2D_SMOKE=1`.
//!
//! # O que esta cena prova
//!
//! Até 2026-09-09 este app tinha um subsistema de áudio inteiro — 42 efeitos, 23 presets,
//! espectral, denoise, exportação — e **um objecto da cena não podia fazer barulho**. A cena põe as
//! três metades na tela e no ouvido:
//!
//! ```text
//!   Sino (x = −3, autoplay, em ciclo) ─────────────► ouve-se À ESQUERDA, sempre
//!   Relogio (2 s, repete) ──sinal `bip`──► Sirene ─► toca À DIREITA, de dois em dois segundos
//!   Ouvinte (x = 0) ────────────────────────────────► ARRASTE-O: o som troca de ouvido
//! ```
//!
//! | objecto | o que ele é | o que se OUVE |
//! |---|---|---|
//! | **Ouvinte** | as orelhas da cena (`AudioListener2D`) | nada — ele é de onde se ouve |
//! | **Sino** | um tom contínuo, `autoplay` ligado, em ciclo | um zumbido **à esquerda** |
//! | **Sirene** | um blip, `autoplay` **desligado** | um toque **à direita** a cada 2 s |
//! | **Relogio** | um relógio de 2 s que repete | é ele que faz a Sirene tocar |
//!
//! # ⚠️ O que provar
//!
//! - **O Sino ouve-se à ESQUERDA** e a Sirene **à direita** — o pan sai da geometria, não de um
//!   knob.
//! - ⭐⭐⭐ **ARRASTE o «Ouvinte» para a esquerda, para lá do Sino.** O zumbido tem de **passar para
//!   o ouvido direito**, e ficar mais alto à medida que se aproxima. É a prova de que o som
//!   **segue** o mundo em vez de ficar congelado no instante do disparo.
//! - **Afaste o Ouvinte para muito longe:** aos 10 m (o alcance do Sino) ele **cala-se por
//!   completo** — e volta ao aproximar.
//! - ⭐⭐ **A Sirene tem `autoplay` DESLIGADO.** Ela só soa porque um sinal a manda soar: é o verbo
//!   `Play Sound` da tabela de acções, o consumidor que o item #5 deixou nomeado como impossível.
//! - ⚠️ **`Ctrl+Z` não desfaz nada disto** — som é **saída**, e este passe não escreve na cena.
//!
//! ⚠️ Se a linha `[audio-2d-smoke]` não aparecer, **PARE**: a cena não montou. E se ela disser
//! `sem dispositivo`, o problema é o servidor de som da máquina, não a cena.
//!
//! # ⚠️ Porque o ficheiro de som é ESCRITO aqui
//!
//! O `AudioSource2D` nomeia um **caminho** (o índice de assets ainda não conhece áudio), então uma
//! cena de smoke precisa de um ficheiro que exista. Ela **gera-o e grava-o** na pasta temporária —
//! o mesmo molde do `PH2D_ASE_SMOKE`, que escreve o `.ase` que vai ler: *um smoke que precisa de um
//! ficheiro que o dono tenha de arranjar é um smoke que não corre*.
//!
//! ⛔ **Este módulo é o ÚNICO sítio gateado na feature do editor de áudio**, e a razão é o
//! ENCODER: gravar um `.wav` é trabalho do `ph2d-audio-encode`, que é opcional. ⚠️ **O produto não
//! é gateado** — o componente, o registo e a ponte existem em qualquer build; só a fixtura deste
//! smoke é que precisa de saber escrever um ficheiro.

use ph2d_audio::AudioFormat;
use ph2d_core::Vec2;
use ph2d_ecs::{
    AudioListener2D, AudioSource2D, Name, SignalAction, SignalActions, SignalVerb, Timer, Timers,
    Transform,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// Grava um `.wav` gerado na pasta temporária e devolve o caminho.
fn escrever(nome: &str, data: ph2d_audio::SampleData) -> Option<String> {
    let path = std::env::temp_dir().join(nome);
    match ph2d_audio_encode::write_wav(&path, &data, ph2d_audio_encode::BitDepth::Pcm16) {
        Ok(()) => Some(path.to_string_lossy().into_owned()),
        Err(e) => {
            eprintln!("[audio-2d-smoke] nao consegui gravar {nome}: {e}");
            None
        }
    }
}

impl crate::App {
    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn audio_2d_smoke(&mut self) {
        if self.audio_2d_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_AUDIO_2D_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda não há mundo; tenta no quadro seguinte
        }
        self.audio_2d_smoke_done = true;

        let fmt = AudioFormat::stereo(48_000);
        // ⚠️ **Um tom LONGO para o ciclo e um curto para o toque** — um blip em ciclo seria uma
        // metralhadora, e um tom longo disparado por sinal empilharia vozes.
        let Some(zumbido) = escrever(
            "ph2d_smoke_sino.wav",
            ph2d_audio_desktop::signals::sine_tone(fmt, 220.0, 2.0, 0.35),
        ) else {
            return;
        };
        let Some(blip) = escrever(
            "ph2d_smoke_sirene.wav",
            ph2d_audio_desktop::signals::sine_tone(fmt, 880.0, 0.25, 0.5),
        ) else {
            return;
        };

        {
            let gfx = self.gfx.as_mut().expect("gfx");
            let world = gfx.sim.world_mut();

            // **AS ORELHAS.** Um objecto vazio com um marcador — a posição vem do `Transform`, como
            // tudo o resto. É este que o dono arrasta.
            world.spawn((
                Transform::from_translation(Vec2::new(0.0, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [0.8, 0.8], [0.95, 0.95, 0.2, 1.0]),
                Name::new("Ouvinte"),
                AudioListener2D,
            ));

            // **O ZUMBIDO à esquerda** — `autoplay` ligado e em ciclo, o único da cena que arranca
            // sozinho.
            world.spawn((
                Transform::from_translation(Vec2::new(-3.0, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [0.3, 0.5, 0.9, 1.0]),
                Name::new("Sino"),
                AudioSource2D {
                    sound: zumbido,
                    looping: true,
                    autoplay: true,
                    max_distance: 10.0,
                    // ⚠️ **Meio metro de raio não-espacializado**: sem ele, arrastar o Ouvinte
                    // rente ao Sino faria o zumbido **saltar** de um ouvido ao outro.
                    non_spatialized_radius: 0.5,
                    ..AudioSource2D::default()
                },
            ));

            // **O TOQUE à direita** — `autoplay` DESLIGADO. Só um sinal o faz soar.
            world.spawn((
                Transform::from_translation(Vec2::new(3.0, 0.0)),
                Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [0.9, 0.4, 0.3, 1.0]),
                Name::new("Sirene"),
                AudioSource2D {
                    sound: blip,
                    max_distance: 12.0,
                    ..AudioSource2D::default()
                },
            ));

            // O relógio que a manda tocar.
            world.spawn((
                Transform::from_translation(Vec2::new(3.0, -1.6)),
                Sprite::atlas(WHITE_TILE_KEY, [1.2, 0.4], [0.35, 0.35, 0.4, 1.0]),
                Name::new("Relogio"),
                Timers(vec![Timer {
                    name: "bip".into(),
                    duration_us: 2_000_000,
                    repeat: true,
                    autostart: true,
                    signal: "bip".into(),
                }]),
                SignalActions(vec![SignalAction {
                    on: "bip".into(),
                    target: "Sirene".into(),
                    verb: SignalVerb::PlaySound,
                    arg: String::new(),
                }]),
            ));
        }

        let dispositivo = if self.audio.is_some() {
            "com dispositivo"
        } else {
            "SEM DISPOSITIVO (a maquina nao abriu a saida de som)"
        };
        eprintln!(
            "[audio-2d-smoke] {dispositivo} · Sino zumbe a` ESQUERDA · Sirene toca a` DIREITA de 2 \
             em 2 s · ARRASTE o «Ouvinte» e o som troca de ouvido"
        );
    }
}
