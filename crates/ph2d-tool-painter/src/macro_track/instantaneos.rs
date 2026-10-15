//! **Os instantâneos de uma macro do Painter em cena** (W5c, `docs/Macros/02_plano_W5.md` §3): o tocador
//! copiado no INÍCIO de cada etapa, para o recuo do laço, o scrub para trás e a edição de uma etapa
//! recomeçarem do instantâneo mais próximo em vez do estado inicial (medido: tocar do zero custa 245 ms
//! por segundo de macro a 1024², 537 a 2048²).
//!
//! O tecto é o do razão da RAM ([`ph2d_core::razao`]: `1/32` da RAM física, partilhado por todas as macros
//! do processo). Passou dele, a macro que grava DOBRA os seus: larga um em cada dois e passa a tirar um
//! em cada `2·passo` etapas — a cobertura fica uniforme, nunca um buraco no fim.

use super::tocar::TocadorDoPainter;
use ph2d_core::razao::{Conta, Razao, ram_fisica, tecto_pela_ram};
use ph2d_macro::MacroPlayer;
use std::sync::{Arc, LazyLock};

/// **A RESERVA do tecto** onde a RAM não se lê (Windows): o `1/32` da máquina mais pequena da tabela
/// (8 GiB, doc 132 §7) — cabe um instantâneo a 2048² (228 MB, medido).
pub const RESERVA_BYTES: usize = 256 << 20;

/// O razão de TODOS os instantâneos de macros do Painter do processo.
#[must_use]
pub fn razao() -> Arc<Razao> {
    static R: LazyLock<Arc<Razao>> =
        LazyLock::new(|| Razao::new(tecto_pela_ram(ram_fisica(), RESERVA_BYTES)));
    Arc::clone(&R)
}

pub(super) struct Foto {
    /// A etapa em cujo início foi tirado (a ordem dela na trilha).
    pub(super) etapa: usize,
    pub(super) player: MacroPlayer,
    pub(super) tocador: TocadorDoPainter,
    bytes: usize,
}

pub(super) struct Instantaneos {
    conta: Conta,
    /// Por etapa, crescente.
    fotos: Vec<Foto>,
    /// Tira-se no início das etapas múltiplas disto (dobra a cada vez que o razão passa o tecto).
    passo: usize,
}

impl Instantaneos {
    pub(super) fn em(razao: Arc<Razao>) -> Self {
        Self {
            conta: Conta::em(razao),
            fotos: Vec::new(),
            passo: 1,
        }
    }

    /// Esta etapa leva instantâneo e ainda não o tem?
    pub(super) fn quer(&self, etapa: usize) -> bool {
        etapa.is_multiple_of(self.passo) && !self.fotos.iter().any(|f| f.etapa == etapa)
    }

    /// Guarda o instantâneo da `etapa` e dobra enquanto o razão passar o tecto.
    pub(super) fn guarda(&mut self, etapa: usize, player: MacroPlayer, tocador: TocadorDoPainter) {
        let bytes = tocador.bytes_retidos();
        let at = self.fotos.partition_point(|f| f.etapa < etapa);
        self.fotos.insert(
            at,
            Foto {
                etapa,
                player,
                tocador,
                bytes,
            },
        );
        self.declara();
        while self.conta.acima() && !self.fotos.is_empty() {
            self.passo *= 2;
            let passo = self.passo;
            self.fotos.retain(|f| f.etapa.is_multiple_of(passo));
            self.declara();
            if self.fotos.len() <= 1 && self.conta.acima() {
                // Nem um cabe: a macro toca do estado inicial.
                self.fotos.clear();
                self.declara();
                break;
            }
        }
    }

    fn declara(&mut self) {
        self.conta.declara(self.fotos.iter().map(|f| f.bytes).sum());
    }

    /// O instantâneo mais tardio cujos passos despachados acabam até `ate` (o tempo da macro, µs).
    pub(super) fn ate(&self, ate: ph2d_macro::MacroTime) -> Option<&Foto> {
        self.fotos.iter().rev().find(|f| f.player.time() <= ate)
    }

    /// O instantâneo mais tardio que despachou no máximo `n` eventos.
    pub(super) fn ate_feitos(&self, n: usize) -> Option<&Foto> {
        self.fotos.iter().rev().find(|f| f.player.dispatched() <= n)
    }

    /// Fica com os instantâneos que `mantem` aceita, refeitos por ele (`None` = larga).
    pub(super) fn refaz(&mut self, mut mantem: impl FnMut(&Foto) -> Option<Foto>) {
        let velhas = std::mem::take(&mut self.fotos);
        self.fotos = velhas.iter().filter_map(&mut mantem).collect();
        self.declara();
    }

    pub(super) fn foto(etapa: usize, player: MacroPlayer, tocador: TocadorDoPainter) -> Foto {
        let bytes = tocador.bytes_retidos();
        Foto {
            etapa,
            player,
            tocador,
            bytes,
        }
    }

    #[must_use]
    pub(super) fn etapas(&self) -> Vec<usize> {
        self.fotos.iter().map(|f| f.etapa).collect()
    }

    #[must_use]
    pub(super) fn passo(&self) -> usize {
        self.passo
    }
}
