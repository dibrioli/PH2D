//! O ÚNICO contexto de placa do binário `it` — antes eram `37` `static OnceLock` (um por ficheiro).
//!
//! Porquê um só (doc 123 §6, medido com as variantes no MESMO binário, intercaladas): com um
//! contexto por ficheiro, todos seguros até ao `exit`, o binário morria com `SIGSEGV` em
//! `libnvidia-glcore` DEPOIS de todos os testes passarem em `8/8` corridas; com um só, `0/16` — com e
//! sem soltá-lo num gancho de saída (o gancho não mudou nada e saiu: ele pedia `unsafe` e uma crate
//! de excepção). A família é a do doc 121 §9.18 (F).

use std::sync::{Mutex, PoisonError};

use ph2d_gpu::GpuContext;

/// `None` = ainda não tentado; `Some(None)` = sem adaptador (não se tenta outra vez).
static PLACA: Mutex<Option<Option<GpuContext>>> = Mutex::new(None);

/// A placa partilhada pelos testes do binário, criada na primeira chamada.
pub fn try_headless_gpu() -> Option<GpuContext> {
    let mut placa = PLACA.lock().unwrap_or_else(PoisonError::into_inner);
    placa
        .get_or_insert_with(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// ⭐ **Nenhum outro ficheiro do `it` guarda um contexto de placa num `static`** (doc 123 §6): um
/// segundo contexto seguro até ao `exit` é o desenho que morria com `SIGSEGV` ao sair. Censo de
/// texto sobre `tests/it`, com o controlo de que ele vê o `static` deste ficheiro.
#[test]
fn only_this_file_holds_a_gpu_context_in_a_static() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/it");
    let mut donos = Vec::new();
    for e in std::fs::read_dir(&dir).expect("tests/it").flatten() {
        let p = e.path();
        if p.extension().is_none_or(|x| x != "rs") {
            continue;
        }
        let t = std::fs::read_to_string(&p).expect("o ficheiro");
        for (n, l) in t.lines().enumerate() {
            let codigo = l.split("//").next().unwrap_or("").trim_start();
            let declara = codigo.starts_with("static ") || codigo.starts_with("pub static ");
            if declara && codigo.contains("GpuContext") {
                donos.push(format!("{}:{}", p.file_name().unwrap_or_default().display(), n + 1));
            }
        }
    }
    assert_eq!(
        donos.len(),
        1,
        "um contexto de placa num `static` fora do `placa.rs` volta a morrer ao sair: {donos:?}"
    );
    assert!(
        donos[0].starts_with("placa.rs:"),
        "controlo: o censo tem de ver o `static` do `placa.rs`: {donos:?}"
    );
}
