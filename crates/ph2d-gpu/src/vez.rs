//! A VEZ da placa: um processo de uma árvore do `scripts/ph2d-run.sh` só pede o adaptador depois de
//! o porteiro dessa árvore lhe dar a vez — duas linhas na placa ao mesmo tempo penduraram o driver
//! (14/09) e estragam a medição uma da outra (Luz 2D, 17/10). Protocolo, medições e recusas:
//! `docs/DevOps/A_VEZ_DA_PLACA.md`; o outro cliente é o `scripts/vez-da-placa.sh`.
//!
//! Há duas filas quando a máquina tem uma segunda placa (a integrada): a **principal** (a das
//! medições, das fotos, do app) e a **de testes**. Só um binário de teste pode ir para a de testes.
//!
//! Fora da porta não há porteiro (`PH2D_GPU_VEZ` ausente) e nada espera: o produto, o app aberto à
//! mão pelo dono, a CI.

use std::io::Write;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// A fila a que um processo pediu a vez.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Placa {
    Principal,
    Testes,
}

impl Placa {
    fn porta(self) -> &'static str {
        match self {
            Self::Principal => "porta",
            Self::Testes => "porta-testes",
        }
    }
    fn vez(self) -> &'static str {
        match self {
            Self::Principal => "vez",
            Self::Testes => "vez-testes",
        }
    }
    fn recusa(self) -> &'static str {
        match self {
            Self::Principal => "recusa",
            Self::Testes => "recusa-testes",
        }
    }
    fn porteiro(self) -> &'static str {
        match self {
            Self::Principal => "porteiro",
            Self::Testes => "porteiro-testes",
        }
    }
}

/// Pacotes cujos testes só passam na placa principal (23/10, na Radeon 610M: 7 testes da Luz 2D
/// mudam texels, alocam além do teto ou perdem a lei de custo; A_VEZ_DA_PLACA.md §7).
const PRESOS_A_PRINCIPAL: &[&str] = &["ph2d-light2d-", "ph2d-app-light2d"];

static PRINCIPAL: OnceLock<()> = OnceLock::new();
static TESTES: OnceLock<()> = OnceLock::new();

/// Pede a vez da placa PRINCIPAL e espera por ela. Idempotente: só a 1.ª chamada do processo pede.
///
/// ⚠️ Antes de `request_adapter`: é o adaptador que põe o processo na tabela da placa (medido 18/10
/// no `nvidia-smi`: a `Instance` sozinha não aparece; depois do `request_adapter`, sim).
///
/// # Panics
/// Se o porteiro desistiu (`PH2D_GPU_ESPERA` esgotado): um teste de placa sem a placa não é verde.
pub fn vez_da_placa() {
    pede_uma_vez(Placa::Principal);
}

/// A vez para o `GpuContext`: a fila de testes quando [`escolhe`] a dá, senão a principal.
/// Um processo que já tem a principal fica nela — quem segura a principal nunca espera a de testes,
/// e é isso que impede duas árvores de se trancarem uma à outra.
pub fn vez_da_placa_ou_a_de_testes() -> Placa {
    let placa = if PRINCIPAL.get().is_some() {
        Placa::Principal
    } else {
        std::env::var_os("PH2D_GPU_VEZ").map_or(Placa::Principal, |d| escolhe(Path::new(&d)))
    };
    pede_uma_vez(placa);
    placa
}

fn pede_uma_vez(placa: Placa) {
    let vez = match placa {
        Placa::Principal => &PRINCIPAL,
        Placa::Testes => &TESTES,
    };
    vez.get_or_init(|| {
        let Some(dir) = std::env::var_os("PH2D_GPU_VEZ") else {
            return;
        };
        if let Err(porque) = pede_em(Path::new(&dir), placa) {
            panic!("[placa] {porque}");
        }
    });
}

/// A fila de testes só para um binário de teste (em `target/*/deps/`, com o `CARGO_PKG_NAME` que o
/// cargo e o nextest lhe dão) de um pacote não preso, numa árvore que não mede (`PH2D_GPU=1`) nem
/// a recusou (`PH2D_PLACA=principal`), e só se o porteiro dela abriu a `porta-testes`.
pub fn escolhe(dir: &Path) -> Placa {
    let var = |k: &str| std::env::var(k).unwrap_or_default();
    let teste = std::env::current_exe()
        .is_ok_and(|e| e.parent().is_some_and(|p| p.ends_with("deps")));
    let pacote = var("CARGO_PKG_NAME");
    let livre = teste
        && !pacote.is_empty()
        && !PRESOS_A_PRINCIPAL.iter().any(|p| pacote.starts_with(p))
        && var("PH2D_GPU") != "1"
        && var("PH2D_PLACA") != "principal"
        && dir.join(Placa::Testes.porta()).exists();
    if livre {
        Placa::Testes
    } else {
        Placa::Principal
    }
}

/// O protocolo, sobre a pasta do porteiro, por fila: a porta (FIFO: «pid nome»), `<vez>.<pid>` (a
/// vez chegou a este processo, até ele acabar), a recusa, o pid do porteiro dela. `Err` = o texto
/// da recusa.
///
/// ⚠️ Pede SEMPRE, mesmo com a árvore já na placa: o porteiro larga a vez quando o último pedinte
/// acaba, e só segura por quem conhece.
pub fn pede_em(dir: &Path, placa: Placa) -> Result<(), String> {
    let pronto = dir.join(format!("{}.{}", placa.vez(), std::process::id()));
    let porteiro_vivo = || {
        std::fs::read_to_string(dir.join(placa.porteiro()))
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
    // A linha vai num ÚNICO `write` (< PIPE_BUF, logo atómico): o `writeln!` num `File` escreve-a aos
    // pedaços, e um porteiro sob carga lia o pid sem o resto (23/10: 2 de 548 pendurados 180 s).
    let pedido = format!("{} {nome}\n", std::process::id());
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(dir.join(placa.porta()))
        .and_then(|mut porta| porta.write_all(pedido.as_bytes()))
        .map_err(|e| format!("não consegui pedir a vez em {}: {e}", dir.display()))?;
    loop {
        if pronto.exists() {
            return Ok(()); // quanto se esperou, di-lo o porteiro (o stderr de um teste é capturado)
        }
        if let Ok(recusa) = std::fs::read_to_string(dir.join(placa.recusa())) {
            return Err(recusa);
        }
        if !porteiro_vivo() {
            sem_vez("o porteiro morreu à espera");
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
