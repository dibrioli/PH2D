//! As fases do Ctrl+Z com a tinta viva, cronometradas SÓ em teste (`sonda_as_fases_do_ctrl_z_wet_a_4k`): fora dos
//! testes as marcas não fazem nada.

#[cfg(test)]
mod relogio {
    use std::cell::{Cell, RefCell};
    use std::time::{Duration, Instant};

    thread_local! {
        static DESDE: Cell<Option<Instant>> = const { Cell::new(None) };
        static SOMA: RefCell<Vec<(&'static str, Duration)>> = const { RefCell::new(Vec::new()) };
    }

    pub(crate) fn comeca() {
        DESDE.with(|d| d.set(Some(Instant::now())));
    }

    pub(crate) fn marca(nome: &'static str) {
        let Some(desde) = DESDE.with(Cell::get) else {
            return;
        };
        let agora = Instant::now();
        SOMA.with(|s| {
            let mut s = s.borrow_mut();
            match s.iter_mut().find(|e| e.0 == nome) {
                Some(e) => e.1 += agora - desde,
                None => s.push((nome, agora - desde)),
            }
        });
        DESDE.with(|d| d.set(Some(agora)));
    }

    pub(crate) fn tira() -> Vec<(&'static str, Duration)> {
        DESDE.with(|d| d.set(None));
        SOMA.with(|s| std::mem::take(&mut *s.borrow_mut()))
    }
}

/// `comeca` abre o cronómetro do passo; as marcas somam por nome até a sonda as `tira`r.
#[cfg(test)]
pub(crate) use relogio::{comeca, tira};

/// Fecha a fase `nome`: o tempo desde a marca anterior.
#[inline]
pub(crate) fn marca(_nome: &'static str) {
    #[cfg(test)]
    relogio::marca(_nome);
}
