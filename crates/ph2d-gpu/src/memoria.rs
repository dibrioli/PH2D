//! **A placa sem memória** (F35 §2, `docs/UI_New_and_Simple/F35_plano.md`): quem recusa memória (um
//! render do Vello sob escopo, ou o observador de erros do dispositivo) ANOTA aqui, e o app pergunta
//! se o estado mudou para avisar o artista — o editor em branco não pode ser silencioso.
//!
//! Estado do processo (um app, um dispositivo): «sem memória» = a última recusa foi há menos de
//! [`JANELA_MS`]. Quem recupera deixa de recusar, e o estado volta sozinho.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

/// Quanto tempo depois da última recusa a placa ainda conta como «sem memória»: maior que a espera
/// entre tentativas do `VelloPass` (500 ms), para o estado não piscar enquanto ele tenta.
pub const JANELA_MS: u64 = 3_000;

/// O vigia de um processo; [`VIGIA`] é o do app, os testes fazem o seu.
pub struct Vigia {
    faltas: AtomicU64,
    /// Instante da última recusa, em ms desde [`origem`] mais 1 (`0` = nunca).
    ultima: AtomicU64,
    relatado: AtomicBool,
    /// Sobe a cada FIM de episódio: o que foi criado sob recusa é inválido para sempre, e quem guarda
    /// recursos de placa compara a geração para os refazer (o FX, o atlas).
    geracao: AtomicU64,
}

impl Vigia {
    pub const fn new() -> Self {
        Self {
            faltas: AtomicU64::new(0),
            ultima: AtomicU64::new(0),
            relatado: AtomicBool::new(false),
            geracao: AtomicU64::new(0),
        }
    }

    /// Uma recusa de memória, no instante `agora_ms`.
    pub fn anota_em(&self, agora_ms: u64) {
        self.faltas.fetch_add(1, Ordering::Relaxed);
        self.ultima.store(agora_ms + 1, Ordering::Relaxed);
    }

    /// Quantas recusas desde o arranque.
    pub fn faltas(&self) -> u64 {
        self.faltas.load(Ordering::Relaxed)
    }

    /// A placa está sem memória no instante `agora_ms`.
    pub fn sem_memoria_em(&self, agora_ms: u64) -> bool {
        match self.ultima.load(Ordering::Relaxed) {
            0 => false,
            u => agora_ms + 1 < u + JANELA_MS,
        }
    }

    /// `Some(estado)` só quando o estado mudou desde a última pergunta — o aviso sai UMA vez por
    /// episódio, e o fim dele também.
    pub fn mudou_em(&self, agora_ms: u64) -> Option<bool> {
        let agora = self.sem_memoria_em(agora_ms);
        let mudou = (self.relatado.swap(agora, Ordering::Relaxed) != agora).then_some(agora);
        if mudou == Some(false) {
            self.geracao.fetch_add(1, Ordering::Relaxed);
        }
        mudou
    }

    /// Quantos episódios de falta já acabaram (ver o campo).
    pub fn geracao(&self) -> u64 {
        self.geracao.load(Ordering::Relaxed)
    }
}

impl Default for Vigia {
    fn default() -> Self {
        Self::new()
    }
}

/// O vigia do processo.
pub static VIGIA: Vigia = Vigia::new();

fn origem() -> Instant {
    static O: OnceLock<Instant> = OnceLock::new();
    *O.get_or_init(Instant::now)
}

fn agora_ms() -> u64 {
    u64::try_from(origem().elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// Anota uma recusa de memória no vigia do processo.
pub fn anota_falta_de_memoria() {
    VIGIA.anota_em(agora_ms());
}

/// A placa do processo está sem memória agora.
pub fn sem_memoria() -> bool {
    VIGIA.sem_memoria_em(agora_ms())
}

/// O estado do processo mudou desde a última pergunta (ver [`Vigia::mudou_em`]).
pub fn sem_memoria_mudou() -> Option<bool> {
    let mudou = VIGIA.mudou_em(agora_ms());
    if let Some(sem) = mudou {
        eprintln!(
            "[gpu-memoria] {} ({} recusas desde o arranque)",
            if sem { "a placa ficou SEM memória" } else { "a placa voltou a ter memória" },
            VIGIA.faltas()
        );
    }
    mudou
}

/// A geração da placa do processo (ver [`Vigia::geracao`]).
pub fn geracao() -> u64 {
    VIGIA.geracao()
}

/// Corre `f` sob um escopo de `OutOfMemory`: a recusa é ANOTADA e devolvida em vez de ir ao
/// observador de erros do dispositivo (o chamador decide o que fazer, ex.: refazer o renderer).
pub fn sob_escopo_de_memoria<R>(
    device: &wgpu::Device,
    f: impl FnOnce() -> R,
) -> (R, Option<wgpu::Error>) {
    let escopo = device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
    let r = f();
    let erro = pollster::block_on(escopo.pop());
    if erro.is_some() {
        anota_falta_de_memoria();
    }
    (r, erro)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **G14 — o aviso sai uma vez por episódio e o fim dele também.** Uma recusa liga o estado; as
    /// recusas seguintes (o `VelloPass` a tentar de 500 em 500 ms) não o repetem; sem recusa durante
    /// [`JANELA_MS`] ele desliga, uma vez.
    #[test]
    fn the_out_of_memory_warning_comes_once_per_episode_and_so_does_its_end() {
        let v = Vigia::new();
        assert_eq!(v.mudou_em(0), None, "sem recusa não há aviso");
        v.anota_em(100);
        assert_eq!(v.mudou_em(100), Some(true));
        for t in (600..=2_600).step_by(500) {
            v.anota_em(t);
            assert_eq!(v.mudou_em(t), None, "a recusa em {t} repetiu o aviso");
        }
        assert!(v.sem_memoria_em(2_600 + JANELA_MS - 2));
        assert_eq!(v.mudou_em(2_600 + JANELA_MS - 2), None);
        assert_eq!(v.geracao(), 0, "a geração só sobe no FIM do episódio");
        assert_eq!(v.mudou_em(2_600 + JANELA_MS), Some(false), "o fim do episódio");
        assert_eq!(v.geracao(), 1);
        assert_eq!(v.mudou_em(2_600 + JANELA_MS + 1_000), None);
        assert_eq!(v.faltas(), 6);
    }
}
