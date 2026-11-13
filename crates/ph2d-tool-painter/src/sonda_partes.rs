//! As partes de um gesto para a sonda (n.º 7 item 3): ms e memória nova DESTE fio (`/proc/thread-self/stat`) desde a
//! marca anterior. Só em `cfg(test)` e só com [`liga`]; fora disso [`parte`] não faz nada.

#[cfg(test)]
thread_local! {
    static PARTES: std::cell::RefCell<Option<(std::time::Instant, u64, Vec<(&'static str, f64, f64)>)>> =
        const { std::cell::RefCell::new(None) };
}

/// As faltas de página menores deste fio (campo 10) × 4 KiB = memória nova tocada por ele.
#[cfg(test)]
fn faltas_do_fio() -> u64 {
    let s = std::fs::read_to_string("/proc/thread-self/stat").unwrap_or_default();
    let resto = s.rsplit_once(") ").map_or("", |(_, r)| r);
    resto
        .split(' ')
        .nth(7)
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

/// Começa a contar neste fio.
#[cfg(test)]
pub(crate) fn liga() {
    PARTES
        .with(|p| *p.borrow_mut() = Some((std::time::Instant::now(), faltas_do_fio(), Vec::new())));
}

/// Pára e devolve `(nome, ms, MB do fio)` de cada parte.
#[cfg(test)]
pub(crate) fn tira() -> Vec<(&'static str, f64, f64)> {
    PARTES.with(|p| p.borrow_mut().take().map(|(_, _, v)| v).unwrap_or_default())
}

/// Fecha a parte `nome` (o que correu desde a marca anterior).
#[inline]
pub(crate) fn parte(nome: &'static str) {
    #[cfg(test)]
    PARTES.with(|p| {
        if let Some((t, f, v)) = p.borrow_mut().as_mut() {
            let agora = faltas_do_fio();
            v.push((
                nome,
                t.elapsed().as_secs_f64() * 1e3,
                (agora - *f) as f64 * 4096.0 / 1e6,
            ));
            *t = std::time::Instant::now();
            *f = faltas_do_fio();
        }
    });
    #[cfg(not(test))]
    let _ = nome;
}
