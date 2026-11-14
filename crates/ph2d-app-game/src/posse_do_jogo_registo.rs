//! A régua do roteiro conduzido da posse do teclado (`docs/CodeEditor/ferramentas/conduz_posse/`):
//! `PH2D_POSSE_LOG=<ficheiro>` só ESCREVE o que [`crate::posse_do_jogo`] decidiu — nenhuma decisão a lê.

use std::io::Write;
use std::sync::{Mutex, OnceLock};

fn escreve(linha: std::fmt::Arguments) {
    static F: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    let f = F.get_or_init(|| {
        std::env::var_os("PH2D_POSSE_LOG")
            .and_then(|p| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .ok()
            })
            .map(Mutex::new)
    });
    if let Some(f) = f
        && let Ok(mut f) = f.lock()
    {
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        let _ = writeln!(f, "{ms} {linha}");
    }
}

pub(crate) fn posse(a_correr: bool, a_vista: bool, tem: bool) {
    escreve(format_args!(
        "posse a_correr={a_correr} a_vista={a_vista} jogo_tem={tem}"
    ));
}

pub(crate) fn tecla(nome: &dyn std::fmt::Debug, d: crate::posse_do_jogo::Destino) {
    escreve(format_args!(
        "tecla {nome:?} jogo={} consumida={} transporte={:?}",
        d.jogo, d.consumida, d.transporte
    ));
}

pub(crate) fn clique(botao: &dyn std::fmt::Debug, na_cena: bool, so: bool) {
    escreve(format_args!(
        "clique {botao:?} na_cena={na_cena} so_do_jogo={so}"
    ));
}

/// A seleção do editor, quando muda — a prova de que um clique na cena a correr não selecciona.
pub fn selecao(agora: Option<u64>) {
    static ANTES: Mutex<Option<Option<u64>>> = Mutex::new(None);
    if let Ok(mut antes) = ANTES.lock()
        && *antes != Some(agora)
    {
        *antes = Some(agora);
        escreve(format_args!("selecao {agora:?}"));
    }
}
