//! A VEZ da placa: um processo de uma árvore do `scripts/ph2d-run.sh` só pede o adaptador depois de
//! o porteiro dessa árvore ter a placa — duas linhas na placa ao mesmo tempo penduraram o driver
//! (14/09) e estragam a medição uma da outra (Luz 2D, 17/10). Protocolo, medições e recusas:
//! `docs/DevOps/A_VEZ_DA_PLACA.md`; o outro cliente é o `scripts/vez-da-placa.sh`.
//!
//! Fora da porta não há porteiro (`PH2D_GPU_VEZ` ausente) e nada espera: o produto, o app aberto à
//! mão pelo dono, a CI.

use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// Pede a vez da placa e espera por ela. Idempotente: só a 1.ª chamada do processo pede.
///
/// ⚠️ Antes de `request_adapter`: é o adaptador que põe o processo na tabela da placa (medido 18/10
/// no `nvidia-smi`: a `Instance` sozinha não aparece; depois do `request_adapter`, sim).
///
/// # Panics
/// Se o porteiro desistiu (`PH2D_GPU_ESPERA` esgotado): um teste de placa sem a placa não é verde.
pub fn vez_da_placa() {
    static VEZ: OnceLock<()> = OnceLock::new();
    VEZ.get_or_init(|| {
        let Some(dir) = std::env::var_os("PH2D_GPU_VEZ") else {
            return;
        };
        if let Err(porque) = pede_em(Path::new(&dir)) {
            panic!("[placa] {porque}");
        }
    });
}

/// O protocolo, sobre a pasta do porteiro: `porta` (FIFO: «pid nome»), `pronto`, `recusa`,
/// `porteiro` (o pid dele). `Err` = o texto da recusa.
pub fn pede_em(dir: &Path) -> Result<(), String> {
    let pronto = dir.join("pronto");
    if pronto.exists() {
        return Ok(());
    }
    let porteiro_vivo = || {
        std::fs::read_to_string(dir.join("porteiro"))
            .is_ok_and(|p| Path::new("/proc").join(p.trim()).exists())
    };
    let sem_vez =
        |porque: &str| eprintln!("⚠️ [placa] {porque} ({}) — sigo SEM a vez", dir.display());
    if !porteiro_vivo() {
        sem_vez("o porteiro já não existe");
        return Ok(());
    }
    let nome = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default();
    // Leitura E escrita: num FIFO esse `open` não bloqueia — um porteiro que morra agora não pendura.
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.join("porta"))
        .and_then(|mut porta| writeln!(porta, "{} {nome}", std::process::id()))
        .map_err(|e| format!("não consegui pedir a vez em {}: {e}", dir.display()))?;
    loop {
        if pronto.exists() {
            return Ok(()); // quanto se esperou, di-lo o porteiro (o stderr de um teste é capturado)
        }
        if let Ok(recusa) = std::fs::read_to_string(dir.join("recusa")) {
            return Err(recusa);
        }
        if !porteiro_vivo() {
            sem_vez("o porteiro morreu à espera");
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
