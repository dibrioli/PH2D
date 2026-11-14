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

#[path = "quadro_rastreio.rs"]
pub mod rastreio;

/// Quanto um trabalho de fundo (autosave, capa a descodificar, mensagem da rede) espera, no pior
/// caso, para ser visto com o laço a dormir (estudo X-02, «prazo 250 ms»). Latência, não recurso.
pub const PRAZO: Duration = Duration::from_millis(250);

/// O intervalo da UI com a janela desfocada: 10 q/s (estudo X-09/D1, PX-25).
pub const DESFOCADA: Duration = Duration::from_millis(100);

/// Oculta e sem present (B23.3), o vsync já não segura o laço: o Documento corre a 60 q/s, o ritmo que o
/// present lhe dava (sem isto, medido: 190–256 q/s e um núcleo a 100 %).
pub const SEM_PRESENT: Duration = Duration::from_nanos(16_666_667);

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
    /// `PH2D_OCULTA_APRESENTA=0|1`: força o present com a janela oculta (A × B no mesmo binário, `mede_pen_up_4k.sh oculto`).
    forca_oculta: Option<bool>,
    /// O próximo quadro sem present ([`SEM_PRESENT`] depois do anterior).
    passo_oculto: Option<Instant>,
    /// Os produtores que se pediram DENTRO do quadro ([`pede`]), um bit por posição em
    /// [`Produtor::TODOS`]; o [`Laco::acorda`] seguinte lê-os e esvazia-os.
    pedidos: u32,
    /// O despertar mais cedo pedido dentro do quadro ([`desperta_ate`]); esvazia como `pedidos`.
    despertar: Option<Instant>,
    /// B21.1: o início do último quadro «já» (o ritmo em Mailbox, [`Laco::no_ritmo`]).
    ultimo_ja: Option<Instant>,
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
            forca_oculta: None,
            passo_oculto: None,
            pedidos: 0,
            despertar: None,
            ultimo_ja: None,
        }
    }

    /// O laço deste processo, com o smoke lido do ambiente uma vez.
    #[must_use]
    pub fn do_ambiente() -> Self {
        let smoke =
            std::env::vars_os().any(|(k, v)| e_smoke(&k.to_string_lossy(), &v.to_string_lossy()));
        let forca_oculta = std::env::var("PH2D_OCULTA_APRESENTA")
            .ok()
            .map(|v| v != "0");
        Self {
            forca_oculta,
            ..Self::novo(smoke)
        }
    }

    pub fn foco(&mut self, focada: bool) {
        self.focada = focada;
    }

    /// A janela ficou oculta (minimizada / coberta) ou voltou a ver-se.
    pub fn oculta(&mut self, oculta: bool) {
        if oculta != self.oculta {
            // Régua do `mede_pen_up_4k.sh oculto`: prova que o `Occluded` chegou (só na mudança).
            eprintln!("[quadro] oculta={oculta}");
        }
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
        let pedidos = (Produtor::TODOS.iter().enumerate())
            .filter(|(i, _)| self.pedidos & (1 << i) != 0)
            .map(|(_, p)| *p);
        for p in documento
            .iter()
            .copied()
            .chain(pedidos)
            .chain(extra)
            .take(ativos.len())
        {
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

    /// B23.3: a janela oculta não apresenta (ninguém a vê); o smoke apresenta sempre (fotografa e conta quadros).
    /// O quadro corre na mesma: o documento avança, e o quadro saltado faz `queue.submit([])` para o upload sair do staging.
    #[must_use]
    pub const fn apresenta(&self) -> bool {
        !self.oculta
            || matches!(self.forca_oculta, Some(true))
            || (self.smoke && self.forca_oculta.is_none())
    }

    /// A tradução para o laço: `None` = desenhar já; `Some(t)` = dormir até `t` (ou um evento).
    /// Sem present, o «já» é o próximo [`SEM_PRESENT`]: o quadro que acabou de correr gastou o anterior.
    #[must_use]
    pub fn acorda(&mut self, documento: &[Produtor], agora: Instant) -> Option<Instant> {
        let ritmo = ph2d_host::ritmo::intervalo();
        self.acorda_no_ritmo(documento, agora, ritmo)
    }

    /// [`Laco::acorda`] com o intervalo mínimo entre quadros explícito (`None` = o `acquire` dá o ritmo).
    #[must_use]
    pub fn acorda_no_ritmo(
        &mut self,
        documento: &[Produtor],
        agora: Instant,
        ritmo: Option<Duration>,
    ) -> Option<Instant> {
        let pedido = self.pedido(documento, agora);
        self.pedidos = 0;
        let despertar = self.despertar.take();
        rastreio::rastreia(&pedido);
        match pedido {
            Pedido::Ja(_) if !self.apresenta() => {
                let alvo = match self.passo_oculto {
                    Some(t) if t > agora => t,
                    Some(t) => t + SEM_PRESENT,
                    None => agora + SEM_PRESENT,
                };
                self.passo_oculto = Some(alvo.max(agora));
                (alvo > agora).then_some(alvo)
            }
            Pedido::Ja(_) => self.no_ritmo(agora, ritmo),
            Pedido::Ate { prazo, .. } => {
                Some(despertar.map_or(agora + prazo, |d| d.min(agora + prazo)))
            }
        }
    }

    /// B21.1: em Mailbox o quadro contínuo espera o próximo refresco DORMINDO (os eventos entram),
    /// em vez de o `acquire` do Fifo o prender; sem ritmo, «já» é já.
    fn no_ritmo(&mut self, agora: Instant, ritmo: Option<Duration>) -> Option<Instant> {
        let proximo = ritmo.zip(self.ultimo_ja).map(|(r, t)| t + r);
        if let Some(t) = proximo.filter(|t| *t > agora) {
            return Some(t);
        }
        self.ultimo_ja = Some(agora);
        None
    }

    /// Ver [`pede`].
    pub fn pede(&mut self, p: Produtor) {
        let i = Produtor::TODOS.iter().position(|q| *q == p).unwrap_or(0);
        self.pedidos |= 1 << i;
    }

    /// Ver [`desperta_ate`].
    pub fn desperta_ate(&mut self, t: Instant) {
        self.despertar = Some(self.despertar.map_or(t, |d| d.min(t)));
    }
}

/// ⭐ B23.2 — um produtor que só se sabe DENTRO do quadro (o tique da família: o tocar do Pixel, a
/// mola ao vivo, o vídeo do Quadro) pede o quadro seguinte daqui, sem uma linha na shell. Vale até ao
/// próximo [`Laco::acorda`]: quem continua a mexer pede-o de novo no quadro que esse pedido trouxe.
pub fn pede(p: Produtor) {
    no_laco(|l| l.pede(p));
}

/// ⭐ B23.2 — o laço a dormir acorda no máximo em `t` (a próxima mudança VISTA de um relógio: o
/// segundo do cronómetro, a fase do piscar), em vez de desenhar a 60 q/s para mudar uma vez por meio
/// segundo. Vale até ao próximo [`Laco::acorda`], como [`pede`].
pub fn desperta_ate(t: Instant) {
    no_laco(|l| l.desperta_ate(t));
}

thread_local! {
    static LACO: std::cell::RefCell<Laco> = std::cell::RefCell::new(Laco::do_ambiente());
}

/// O [`Laco`] do fio da UI: a shell dá-lhe os eventos da janela e os produtores do quadro.
pub fn no_laco<R>(f: impl FnOnce(&mut Laco) -> R) -> R {
    LACO.with_borrow_mut(f)
}

/// Acorda o laço a dormir de QUALQUER fio (B23.2): a shell instala-o com o `EventLoopProxy`.
pub type Despertador = std::sync::Arc<dyn Fn() + Send + Sync>;

static DESPERTADOR: std::sync::OnceLock<Despertador> = std::sync::OnceLock::new();

/// A shell, uma vez, antes do laço. Uma segunda instalação é ignorada (o laço é um só).
pub fn instala_o_despertador(f: Despertador) {
    let _ = DESPERTADOR.set(f);
}

/// Um trabalho de fundo (a rede do Quadro) acabou: o laço desenha já, sem esperar o [`PRAZO`].
pub fn acorda_o_laco() {
    if let Some(f) = DESPERTADOR.get() {
        f();
    }
}

#[cfg(test)]
#[path = "quadro_tests.rs"]
mod tests;
