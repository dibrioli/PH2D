//! ⭐ **Quadro a pedido** (B23.2/B23.3, PX-24/PX-25): a lei que decide se o laço desenha JÁ ou
//! dorme até um prazo, e porquê. Função pura: a shell junta os produtores activos do quadro e
//! traduz o [`Pedido`] em `ControlFlow::Poll` / `WaitUntil`.
//!
//! Três classes. **Documento**: acumula `dt` no byte (a secagem do papel faz `carry.min(255)` num
//! passo só, o traço aberto assenta por quadro), logo NUNCA dorme, com foco ou sem ele. **Isento**:
//! Play, som, Motion/Timeline a tocar e smokes que contam quadros: a 60 q/s mesmo desfocada (decisão
//! PX-25). **UI**: com foco, desenha já; desfocada, a [`DESFOCADA`] por quadro.
//! Sem ninguém a pedir, dorme [`PRAZO`]: o despertador que olha relógios e trabalhos de fundo.

use std::time::Duration;

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

#[cfg(test)]
#[path = "quadro_tests.rs"]
mod tests;
