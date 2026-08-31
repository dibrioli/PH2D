//! ⭐⭐⭐ **A POEIRA DE IMPACTO do chrome** (estudo de UI viva, D2) — *o gesto confirma-se pelo olho*.
//!
//! # Porque isto é uma crate-folha e não o simulador do documento
//!
//! ⛔⛔ O estudo pede *"partículas de feedback **do motor que já temos**"*, tamanho **G** — e isso
//! foi **medido falso** em 2026-08-24: o motor que existe é o simulador do **DOCUMENTO**
//! (`ph2d-eval-motion` + `ph2d-gpu-cook`, cozido por quadro a partir de um **grafo de nós**), e no
//! chrome não há canal de partículas nenhum. Ligar um ao outro pediria grafo, cook e documento para
//! uma faísca de doze pontos num encaixe — *arquitectura errada*.
//!
//! ⇒ o que a D2 de facto pede é um **burst local no relógio de UI**, e é isso que isto é: aritmética
//! pura, sem ECS, sem GPU, sem alocação por quadro.
//!
//! # ⭐⭐ A LEI: uma faísca CONFIRMA o que a mão fez, nunca ANUNCIA o que o app decidiu
//!
//! É a mesma frase que o [`crate::motion`] e o som de UI já obedecem, e ela tem duas metades:
//!
//! - **só o que a MÃO causou emite** — um encaixe que pousa, uma junção que fecha. O que o app faz
//!   sozinho (um quadro, uma transição a assentar, um painel a publicar) é **mudo**;
//! - **o HOVER nunca emite.** Passar o rato por uma fileira é o gesto mais barato do editor;
//!   faiscá-lo transforma navegar em fogo de artifício.
//!
//! # ⛔ A CERCA já existia, e o papel também
//!
//! [`crate::motion::Role::Decoration`] diz, desde que nasceu: *"Enfeite (rasto, **partícula**,
//! corda). **Ausente em Discreto** — ausente, não atenuado."* Ele foi reservado e nunca teve
//! consumidor. ⇒ este módulo **não traz cerca nova**: ele pergunta ao `UiMotion` que já existe, e
//! herda de graça o `reduced_motion` e o carácter.
//!
//! ⚠️ *Ausente, não atenuado* é load-bearing: em Discreto não há uma faísca curta — não há faísca.

/// Quantas partículas uma faísca emite.
///
/// ⚠️ **É pequeno de propósito, e o número tem dono:** o estudo pede *"faísca no ponto EXACTO do
/// encaixe"*, não uma explosão. Doze pontos lêem-se como **um** evento; cem lêem-se como um efeito,
/// e um efeito compete com o desenho do artista — que é a coisa que o chrome existe para não fazer.
pub const SPARKS: usize = 12;

/// Quanto tempo uma faísca vive, em segundos.
///
/// ⚠️ Do lado curto da faixa que o `motion` já usa: uma confirmação que dura mais que o gesto deixa
/// de confirmar e passa a **anunciar**.
pub const LIFE_S: f32 = 0.42;

/// **UMA faísca** — o estado inteiro cabe em duas coordenadas e um relógio.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Burst {
    /// Onde a mão encostou, em pixels de ecrã.
    pub at: [f32; 2],
    /// Idade, em segundos. Nasce em `0` e morre em [`LIFE_S`].
    pub age: f32,
    /// A semente — duas faíscas no mesmo sítio não podem sair idênticas.
    pub seed: u32,
}

impl Burst {
    /// Nasce em `at`. ⚠️ A `seed` é do CHAMADOR: este módulo não tem relógio nem aleatório, e é
    /// isso que o torna testável sem arnês.
    #[must_use]
    pub fn new(at: [f32; 2], seed: u32) -> Self {
        Self { at, age: 0.0, seed }
    }

    /// Envelheceu para além da vida?
    #[must_use]
    pub fn dead(&self) -> bool {
        self.age >= LIFE_S
    }

    /// **Onde está a partícula `i`, e com que opacidade** — `None` quando a faísca já morreu.
    ///
    /// A lei é balística e sem estado: posição em função da IDADE, não integrada passo a passo.
    /// ⭐ É isso que a torna **imune ao ritmo do quadro** — um quadro perdido não desloca a faísca,
    /// e é a mesma razão pela qual o traço do Painter é fato do CAMINHO e não da amostragem.
    ///
    /// ⚠️ A opacidade cai com o **quadrado** do tempo normalizado: linearmente a faísca parece
    /// desaparecer de repente no fim, porque o olho lê luminância e não alfa.
    #[must_use]
    pub fn spark(&self, i: usize) -> Option<([f32; 2], f32)> {
        if self.dead() || i >= SPARKS {
            return None;
        }
        let t = self.age / LIFE_S;
        // Ângulo e velocidade DERIVADOS da semente e do índice — um gerador de baixa qualidade
        // basta e não paga uma dependência nem um estado por partícula.
        let h = (self.seed ^ (i as u32).wrapping_mul(0x9E37_79B9)).wrapping_mul(0x85EB_CA6B);
        #[allow(clippy::cast_precision_loss)]
        let ang = (h >> 8) as f32 / 16_777_216.0 * std::f32::consts::TAU;
        #[allow(clippy::cast_precision_loss)]
        let vel = 34.0 + ((h >> 3) & 0xFF) as f32 / 255.0 * 26.0;
        // Arrasto: a faísca perde a velocidade que ganhou, e o `1 - (1-t)^2` é a integral disso.
        let s = vel * (1.0 - (1.0 - t) * (1.0 - t));
        let queda = 26.0 * t * t;
        Some((
            [
                self.at[0] + ang.cos() * s,
                self.at[1] + ang.sin() * s + queda,
            ],
            (1.0 - t) * (1.0 - t),
        ))
    }
}

/// **O campo de faíscas do chrome** — o que o quadro guarda entre eventos.
#[derive(Default, Debug)]
pub struct BurstField {
    vivas: Vec<Burst>,
}

impl BurstField {
    /// **Arma uma faísca em `at`** — no-op quando o carácter a proíbe.
    ///
    /// ⛔ A cerca é a do [`crate::motion::Role::Decoration`], que já existe: *ausente em Discreto,
    /// ausente sob reduced motion*. ⚠️ **Perguntar aqui e não em cada sítio que arma** é o que
    /// impede o quinto chamador de nascer sem ela.
    pub fn emit(&mut self, motion: &crate::motion::UiMotion, at: [f32; 2], seed: u32) {
        if motion.law(crate::motion::Role::Decoration).is_none() {
            return;
        }
        self.vivas.push(Burst::new(at, seed));
    }

    /// Envelhece `dt` segundos e enterra o que morreu.
    ///
    /// ⚠️ O `dt` é o do CHROME (`chrome_dt`), e não o do quadro: um diálogo modal congela o laço, e
    /// uma faísca não envelhece enquanto nada é desenhado. É a mesma lei que o `crate::modal` do
    /// shell já impõe aos toasts.
    pub fn tick(&mut self, dt: f32) {
        for b in &mut self.vivas {
            b.age += dt;
        }
        self.vivas.retain(|b| !b.dead());
    }

    /// As faíscas vivas, para quem desenha.
    #[must_use]
    pub fn live(&self) -> &[Burst] {
        &self.vivas
    }

    /// Vazio? — o caminho comum, e quem desenha sai cedo por aqui.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vivas.is_empty()
    }
}

#[cfg(test)]
#[path = "motion_burst_tests.rs"]
mod tests;
