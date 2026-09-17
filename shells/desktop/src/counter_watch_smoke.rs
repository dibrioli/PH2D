//! ⭐⭐⭐ **O prólogo da cena da VIGIA DO CONTADOR** — o que decide a ORDEM do quadro fica na shell;
//! o CORPO da cena vive em [`ph2d_app_components::counter_watch_smoke`].
//!
//! ⚠️ **A auto-conferência existe porque a FOTO não decide esta pergunta** — uma imagem mostra um
//! instante, e *«desapareceu ao chegar a zero»* é uma propriedade de um INTERVALO. É a lei que a
//! wave da cutscene pagou com três fotografias correctas sobre uma cena que não corria.

use ph2d_ecs::{CounterRuntime, Visibility};

/// Quantos quadros a conferência amostra.
///
/// ⚠️ **O número sai da CENA, não de um palpite:** três vidas a `1,2 s` cada gastam `3,6 s`, e a
/// 60 Hz isso são `216` quadros — mais uma folga para o `Hide` chegar pela tabela de acções.
const AMOSTRAS: u32 = 300;

impl crate::App {
    /// No prólogo do quadro. No-op sem a env.
    pub(crate) fn counter_watch_smoke(&mut self) {
        if std::env::var_os("PH2D_COUNTERWATCH_SMOKE").is_none() {
            return;
        }
        if self.components.smokes.counter_watch {
            self.counter_watch_smoke_traz_o_inspector();
            self.counter_watch_smoke_confere();
            return;
        }
        if self.gfx.is_none() {
            return; // ainda sem mundo; tenta no quadro seguinte
        }
        self.components.smokes.counter_watch = true;
        self.counter_watch_smoke_monta();
    }

    fn counter_watch_smoke_monta(&mut self) {
        let Some(mut cx) = self.components_ctx() else {
            return;
        };
        ph2d_app_components::counter_watch_smoke::counter_watch_smoke(&mut cx);

        let (heroi, controlo) = {
            let gfx = self.gfx.as_ref().expect("gfx");
            let acha = |nome: &str| {
                let world = gfx.sim.world();
                world
                    .try_query::<(ph2d_ecs::Entity, &ph2d_ecs::Name)>()
                    .and_then(|mut q| {
                        q.iter(world)
                            .find(|(_, n)| n.0 == nome)
                            .map(|(e, _)| e.to_bits())
                    })
            };
            (acha("Heroi"), acha("Controlo (sem vigia)"))
        };
        // ⭐ **Escolhe o HERÓI** — é a secção dele que o dono vem ver.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("inspector", true);
            hero.gizmo.selection = heroi;
        }
        self.components.smokes.counter_watch_raise = 3;
        self.components.counter_watch = crate::app_state::CounterWatchShell {
            resta: AMOSTRAS,
            heroi: heroi.unwrap_or_default(),
            controlo: controlo.unwrap_or_default(),
            heroi_sumiu: false,
            controlo_sumiu: false,
            luzes_heroi: 0,
            luzes_controlo: 0,
            min_heroi: i64::MAX,
            min_controlo: i64::MAX,
        };
        // ⚠️ **O relógio TEM de andar**: a vigia avalia no passo fixo, e é o `Timer` que tira as
        // vidas. Com o transporte parado o dono vê dois quadrados imóveis e lê *«não faz nada»*.
        self.playhead.rewind();
        self.playhead.play();
        eprintln!(
            "[counterwatch-smoke] heroi={heroi:?} controlo={controlo:?} — a conferir durante \
             {AMOSTRAS} quadros…"
        );
    }

    /// Traz o Inspector à frente por alguns quadros — ver o irmão da cutscene.
    fn counter_watch_smoke_traz_o_inspector(&mut self) {
        if self.components.smokes.counter_watch_raise == 0 {
            return;
        }
        self.components.smokes.counter_watch_raise -= 1;
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.bump_panel_z(ph2d_editor_core::ids::INSP_PANEL);
        }
    }

    /// ⭐⭐⭐ **A cena confere-se a si mesma: o herói SUMIU e o controlo NÃO** — e os dois chegaram
    /// a zero.
    ///
    /// ⭐ **As duas metades, e a segunda é o que torna a primeira uma prova.** Sem o mínimo do
    /// controlo, um herói que sumisse por qualquer outra razão leria como aprovação; sem o
    /// controlo ainda visível, um `Hide` que apanhasse os dois também.
    fn counter_watch_smoke_confere(&mut self) {
        if self.components.counter_watch.resta == 0 {
            return;
        }
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let world = gfx.sim.world();
        let ler = |bits: u64| {
            let e = ph2d_ecs::Entity::from_bits(bits);
            (
                world.get::<CounterRuntime>(e).map(|c| c.value),
                world.get::<Visibility>(e).map(|v| v.hidden),
            )
        };
        let (vh, sh) = ler(self.components.counter_watch.heroi);
        let (vc, sc) = ler(self.components.counter_watch.controlo);
        // ⭐⭐ **As LUZES são a metade que o dono de facto VÊ** — e é por elas que a cena se explica
        // sem um número na tela. ⚠️ Contam-se por NOME, que é o que a tabela de acções endereça.
        let apagadas = |prefixo: &str| -> usize {
            // ⚠️ `try_query` porque ele lê `&World` — e um mundo que nunca viu um `Visibility`
            // responde «nenhuma», que é a resposta certa.
            let world = gfx.sim.world();
            world
                .try_query::<(&ph2d_ecs::Name, &Visibility)>()
                .map_or(0, |mut q| {
                    q.iter(world)
                        .filter(|(n, v)| n.0.starts_with(prefixo) && v.hidden)
                        .count()
                })
        };
        let (lh, lc) = (
            apagadas(ph2d_app_components::counter_watch_smoke::HEROI_LUZ),
            apagadas(ph2d_app_components::counter_watch_smoke::CONTROLO_LUZ),
        );
        let s = &mut self.components.counter_watch;
        if let Some(v) = vh {
            s.min_heroi = s.min_heroi.min(v);
        }
        if let Some(v) = vc {
            s.min_controlo = s.min_controlo.min(v);
        }
        s.heroi_sumiu |= sh.unwrap_or(false);
        s.controlo_sumiu |= sc.unwrap_or(false);
        s.luzes_heroi = s.luzes_heroi.max(lh);
        s.luzes_controlo = s.luzes_controlo.max(lc);
        s.resta -= 1;
        if s.resta > 0 {
            return;
        }
        let veredito = s.min_heroi <= 0
            && s.min_controlo <= 0
            && s.heroi_sumiu
            && !s.controlo_sumiu
            && s.luzes_heroi == 3
            && s.luzes_controlo == 0;
        eprintln!(
            "[counterwatch-smoke] heroi: chegou a {}, {} luz(es) apagada(s) (tem de ser 3), {} · \
             controlo: chegou a {}, {} luz(es) apagada(s) (tem de ser 0), {} · veredito: {}",
            s.min_heroi,
            s.luzes_heroi,
            if s.heroi_sumiu { "SUMIU" } else { "ficou" },
            s.min_controlo,
            s.luzes_controlo,
            if s.controlo_sumiu { "SUMIU" } else { "ficou" },
            if veredito { "SIM" } else { "NAO" }
        );
    }
}
