//! ⭐ **Quadro a pedido** (B23.2/B23.3, PX-24/PX-25): a lei que decide se o laço desenha JÁ ou
//! dorme até um prazo, e porquê. Função pura: a shell junta os produtores activos do quadro e
//! traduz o [`Pedido`] em `ControlFlow::Poll` / `WaitUntil`.
//!
//! Três classes. **Documento**: acumula `dt` no byte (a secagem do papel faz `carry.min(255)` num
//! passo só, o traço aberto assenta por quadro), logo NUNCA dorme, com foco ou sem ele. **Isento**:
//! Play, som, Motion/Timeline a tocar e smokes que contam quadros: a 60 q/s mesmo desfocada (decisão
//! PX-25). **UI**: com foco, desenha já; desfocada, a [`DESFOCADA`] por quadro.
//! Sem ninguém a pedir, dorme [`PRAZO`]: o despertador que olha relógios e trabalhos de fundo.

use std::time::{Duration, Instant};

/// Quanto um trabalho de fundo (autosave, capa a descodificar, mensagem da rede) espera, no pior
/// caso, para ser visto com o laço a dormir (estudo X-02, «prazo 250 ms»). Latência, não recurso.
pub const PRAZO: Duration = Duration::from_millis(250);

/// O intervalo da UI com a janela desfocada: 10 q/s (estudo X-09/D1, PX-25).
pub const DESFOCADA: Duration = Duration::from_millis(100);

/// Quem pode pedir um quadro. Cada variante tem a sua [`Classe`] (o `match` é exaustivo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Produtor {
    /// Traço com o botão premido e a caneta parada (o assentamento corre por quadro).
    TracoAberto,
    /// `canvas_wet` ainda com tinta: a secagem é por `dt`, não por evento.
    PapelMolhado,
    /// A simulação do Wet Paint a correr.
    SimWet,
    /// Uma leitura da placa (`map_async`) por colher.
    LeituraDaPlaca,
    /// A folha ao vivo com uma actualização devida (B10).
    FolhaDevida,
    /// O jogo no palco ou na janela do jogo.
    Play,
    /// Áudio a tocar.
    Som,
    /// Motion ou Timeline a tocar.
    Animacao,
    /// Um smoke que conduz a cena ou conta quadros (`PH2D_EXIT_AFTER_FRAMES`).
    Smoke,
    /// Uma aba de família pede outro quadro (`Documents::wants_frame`).
    Aba,
    /// Um toast à vista (envelhece por `dt`).
    Toast,
    /// Uma transição da UI a correr (`UiMachine::is_animating`).
    UiAnima,
    /// Uma sessão ao vivo do Quadro (rede).
    Rede,
    /// Um evento de entrada há menos de [`PRAZO`].
    Entrada,
}

/// A classe decide se o produtor dorme desfocado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classe {
    Documento,
    Isento,
    Ui,
}

impl Produtor {
    pub const TODOS: [Produtor; 14] = [
        Produtor::TracoAberto,
        Produtor::PapelMolhado,
        Produtor::SimWet,
        Produtor::LeituraDaPlaca,
        Produtor::FolhaDevida,
        Produtor::Play,
        Produtor::Som,
        Produtor::Animacao,
        Produtor::Smoke,
        Produtor::Aba,
        Produtor::Toast,
        Produtor::UiAnima,
        Produtor::Rede,
        Produtor::Entrada,
    ];

    #[must_use]
    pub const fn classe(self) -> Classe {
        match self {
            Produtor::TracoAberto
            | Produtor::PapelMolhado
            | Produtor::SimWet
            | Produtor::LeituraDaPlaca
            | Produtor::FolhaDevida => Classe::Documento,
            Produtor::Play | Produtor::Som | Produtor::Animacao | Produtor::Smoke => Classe::Isento,
            Produtor::Aba
            | Produtor::Toast
            | Produtor::UiAnima
            | Produtor::Rede
            | Produtor::Entrada => Classe::Ui,
        }
    }
}

/// Porque o laço dorme: o sono é uma parada NOMEADA, nunca um silêncio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Parada {
    /// Ninguém pediu: o despertador de [`PRAZO`].
    Ocioso,
    /// Só a UI pediu, com a janela desfocada: [`DESFOCADA`].
    Desfocada,
    /// A janela está oculta (`WindowEvent::Occluded`): a UI não tem quem a veja, dorme o [`PRAZO`].
    Oculta,
}

/// A resposta da lei.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pedido {
    /// Desenhar já (laço contínuo), por causa deste produtor.
    Ja(Produtor),
    /// Dormir até `prazo` depois de agora (ou até um evento), por `porque`.
    Ate { prazo: Duration, porque: Parada },
}

/// O que a shell sabe neste quadro.
#[derive(Clone, Copy, Debug)]
pub struct Estado<'a> {
    pub ativos: &'a [Produtor],
    pub focada: bool,
}

/// ⭐ A lei: o primeiro produtor por ordem de classe (Documento, Isento, Ui) decide.
#[must_use]
pub fn quer_quadro(e: &Estado<'_>) -> Pedido {
    let primeiro = |c: Classe| e.ativos.iter().copied().find(|p| p.classe() == c);
    if let Some(p) = primeiro(Classe::Documento).or_else(|| primeiro(Classe::Isento)) {
        return Pedido::Ja(p);
    }
    match primeiro(Classe::Ui) {
        Some(p) if e.focada => Pedido::Ja(p),
        Some(_) => Pedido::Ate {
            prazo: DESFOCADA,
            porque: Parada::Desfocada,
        },
        None => Pedido::Ate {
            prazo: PRAZO,
            porque: Parada::Ocioso,
        },
    }
}

/// O que a shell guarda entre quadros e a lei não sabe: o foco, a última entrada e se este
/// processo é um smoke. A shell só lhe dá os eventos e os produtores do documento.
#[derive(Clone, Copy, Debug)]
pub struct Laco {
    focada: bool,
    ultima_entrada: Option<Instant>,
    smoke: bool,
    oculta: bool,
}

/// Um smoke conta quadros ou conduz a cena: qualquer `PH2D_*SMOKE*`/`*DEMO*`,
/// `PH2D_EXIT_AFTER_FRAMES`, ou `PH2D_QUADRO_A_PEDIDO=0` (o laço contínuo de antes, para comparar).
#[must_use]
pub fn e_smoke(nome: &str, valor: &str) -> bool {
    let p = nome.strip_prefix("PH2D_");
    p.is_some_and(|r| r.contains("SMOKE") || r.contains("DEMO") || r == "EXIT_AFTER_FRAMES")
        || (nome == "PH2D_QUADRO_A_PEDIDO" && valor == "0")
}

impl Laco {
    #[must_use]
    pub const fn novo(smoke: bool) -> Self {
        Self {
            focada: true,
            ultima_entrada: None,
            smoke,
            oculta: false,
        }
    }

    /// O laço deste processo, com o smoke lido do ambiente uma vez.
    #[must_use]
    pub fn do_ambiente() -> Self {
        let smoke =
            std::env::vars_os().any(|(k, v)| e_smoke(&k.to_string_lossy(), &v.to_string_lossy()));
        Self::novo(smoke)
    }

    pub fn foco(&mut self, focada: bool) {
        self.focada = focada;
    }

    /// A janela ficou oculta (minimizada / coberta) ou voltou a ver-se.
    pub fn oculta(&mut self, oculta: bool) {
        self.oculta = oculta;
    }

    pub fn entrada(&mut self, agora: Instant) {
        self.ultima_entrada = Some(agora);
    }

    /// A lei sobre os produtores da shell mais os do laço (smoke, entrada há menos de [`PRAZO`]).
    #[must_use]
    pub fn pedido(&self, documento: &[Produtor], agora: Instant) -> Pedido {
        let mut ativos = [Produtor::Entrada; 16];
        let mut n = 0;
        let recente = self
            .ultima_entrada
            .is_some_and(|t| agora.saturating_duration_since(t) < PRAZO);
        let deste = [(self.smoke, Produtor::Smoke), (recente, Produtor::Entrada)];
        let extra = deste.iter().filter(|(sim, _)| *sim).map(|(_, p)| *p);
        for p in documento.iter().copied().chain(extra).take(ativos.len()) {
            ativos[n] = p;
            n += 1;
        }
        let lei = quer_quadro(&Estado {
            ativos: &ativos[..n],
            focada: self.focada,
        });
        match lei {
            Pedido::Ja(p) if p.classe() != Classe::Ui => lei,
            _ if self.oculta => Pedido::Ate {
                prazo: PRAZO,
                porque: Parada::Oculta,
            },
            _ => lei,
        }
    }

    /// A tradução para o laço: `None` = desenhar já; `Some(t)` = dormir até `t` (ou um evento).
    #[must_use]
    pub fn acorda(&self, documento: &[Produtor], agora: Instant) -> Option<Instant> {
        match self.pedido(documento, agora) {
            Pedido::Ja(_) => None,
            Pedido::Ate { prazo, .. } => Some(agora + prazo),
        }
    }
}

thread_local! {
    static LACO: std::cell::RefCell<Laco> = std::cell::RefCell::new(Laco::do_ambiente());
}

/// O [`Laco`] do fio da UI: a shell dá-lhe os eventos da janela e os produtores do quadro.
pub fn no_laco<R>(f: impl FnOnce(&mut Laco) -> R) -> R {
    LACO.with_borrow_mut(f)
}

#[cfg(test)]
#[path = "quadro_tests.rs"]
mod tests;
