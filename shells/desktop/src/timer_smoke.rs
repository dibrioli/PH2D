//! ⭐⭐⭐ **Smoke do `Timer`** (TOP-20 #2). `PH2D_TIMER_SMOKE=1`.
//!
//! Três objectos na tela, cada um com um relógio autorado, e **nada mais**: nenhuma colisão,
//! nenhuma timeline a correr, nenhum script. É essa a prova — até 2026-09-08 o único produtor de
//! sinal autorável da cena era um **contacto da física**, e portanto *nada podia acontecer por si*.
//!
//! | objecto | o timer | o que se vê |
//! |---|---|---|
//! | **Batida** | 1 s, **repete**, sinal `batida` | um toast por segundo, para sempre |
//! | **Recarga** | 3 s, **uma vez**, sinal `arma_pronta` | UM toast aos 3 s, e nunca mais |
//! | **Mudo** | 2 s, repete, **sem nome de sinal** | **nada** — ele cumpre o período e cala-se |
//!
//! # O que provar na tela
//!
//! - **Um toast por segundo** (`Signal: batida`) — o relógio corre sozinho.
//! - **Um único** `Signal: arma_pronta`, aos 3 s. Se ele repetir, o *one-shot* não parou.
//! - **Nunca** um toast do «Mudo». ⚠️ É a lei da §11: *um produtor sem nome não fala, em vez de
//!   falar com um nome vazio* — e o gate mede o campo, esta cena mede a ponte.
//! - **`Ctrl+Z` não desfaz o tique.** Carregue nele à vontade enquanto os toasts sobem: a fila de
//!   undo tem de estar vazia (nada a desfazer). É a separação CONFIG/vivo — se o relógio estivesse
//!   no componente registado, **cada quadro** seria um passo.
//!
//! ⚠️ **Ligue `PH2D_SIGNAL_LOG=1` junto**: no terminal cada linha diz a ORIGEM
//! (`<- timer do objecto N, 1 periodo(s)`), lida por um cursor DIFERENTE do dos toasts. Dois
//! consumidores, um canal.
//!
//! ⚠️ Se a linha `[timer-smoke]` não aparecer, **PARE**: a cena não montou.
//!
//! # ⭐ E a partir da W3 a cena também prova o PAINEL
//!
//! Escolher qualquer um dos três objectos abre a secção **Timers** no Inspector, com a lista, o
//! `+ Add Timer` / `x Remove Timer` e os cinco campos do que está aberto. É a metade que faltava:
//! até 2026-09-08 o componente anexava-se pela paleta e **nada aparecia**, que é indistinguível de
//! ele não ter sido anexado.
//!
//! ⚠️ **A cena é o oráculo do painel**, e é por isso que os três objectos são diferentes uns dos
//! outros: mudar a duração da *Batida* muda o ritmo dos toasts **enquanto se olha**, ligar o
//! `Repeat` da *Recarga* fá-la falar para sempre, e escrever um nome no campo *Signal* do *Mudo*
//! tira-o do silêncio. *Um painel que se prova sobre um objecto parado prova metade.*

use ph2d_core::Vec2;
use ph2d_ecs::{Name, Timer, Timers, Transform};
use ph2d_render::Sprite;

/// Um objecto com um timer só, na posição `y`.
fn one(
    world: &mut ph2d_ecs::World,
    name: &str,
    y: f32,
    tint: [f32; 4],
    duration_us: u64,
    repeat: bool,
    signal: &str,
) {
    world.spawn((
        Transform::from_translation(Vec2::new(0.0, y)),
        Sprite::atlas(0, [1.2, 0.6], tint),
        Name::new(name),
        Timers(vec![Timer {
            name: name.to_string(),
            duration_us,
            repeat,
            autostart: true,
            signal: signal.to_string(),
        }]),
        // ⚠️⚠️ **NADA de relógio aqui, e a ausência é a correcção de 2026-09-08.** A 1.ª versão
        // semeava um `TimerRuntime::default()` *«porque a cena não passa pelo load»* — e um
        // runtime **vazio e por armar** é exactamente o estado em que os três timers ficavam
        // mudos: a cena montava, imprimia a linha abaixo, e produzia **zero** sinais.
        //
        // ⇒ quem cria e arma é o `start_autostart_timers`, que passou a correr por quadro. Uma
        // entidade nascida aqui é igual a uma nascida da paleta ou de uma cópia — que é
        // precisamente o que esta cena tem de provar.
    ));
}

impl crate::App {
    /// No prólogo do quadro, uma vez. No-op sem a env.
    pub(crate) fn timer_smoke(&mut self) {
        if self.timer_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_TIMER_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // ainda não há mundo; tenta no quadro seguinte
        }
        self.timer_smoke_done = true;
        {
            let gfx = self.gfx.as_mut().expect("gfx");
            let world = gfx.sim.world_mut();
            one(
                world,
                "Batida",
                2.0,
                [0.2, 0.8, 0.4, 1.0],
                1_000_000,
                true,
                "batida",
            );
            one(
                world,
                "Recarga",
                0.0,
                [0.9, 0.6, 0.1, 1.0],
                3_000_000,
                false,
                "arma_pronta",
            );
            one(
                world,
                "Mudo",
                -2.0,
                [0.5, 0.5, 0.55, 1.0],
                2_000_000,
                true,
                "",
            );
        }
        // ⚠️ **A linha que o doc manda procurar** — uma cena que monta em silêncio é uma cena que
        // o smoke julga errado.
        eprintln!(
            "[timer-smoke] 3 objectos: Batida (1s, repete) · Recarga (3s, uma vez) · Mudo (2s, sem sinal)"
        );
    }
}
