//! As sub-fases do undo/redo do motor (B18.11), cronometradas SÓ em teste ou com a feature `sonda-fases` (o
//! `dev-dependency` do Painter, para a `sonda_as_fases_do_ctrl_z_wet_a_4k`): fora disso as marcas não fazem nada.

/// Fecha a fase `nome`: o tempo desde a marca anterior (ou o [`abre`]).
#[inline]
pub fn marca(_nome: &'static str) {
    #[cfg(any(test, feature = "sonda-fases"))]
    relogio::marca(_nome);
}

/// Recomeça o tempo da fase seguinte (a primeira marca do motor não conta o que veio antes dele).
#[inline]
pub fn abre() {
    #[cfg(any(test, feature = "sonda-fases"))]
    relogio::abre();
}

#[cfg(any(test, feature = "sonda-fases"))]
pub use relogio::{censo, liga, tira};

#[cfg(any(test, feature = "sonda-fases"))]
mod relogio {
    use std::cell::{Cell, RefCell};
    use std::time::{Duration, Instant};

    thread_local! {
        static LIGADO: Cell<bool> = const { Cell::new(false) };
        static DESDE: Cell<Option<Instant>> = const { Cell::new(None) };
        static SOMA: RefCell<Vec<(&'static str, Duration)>> = const { RefCell::new(Vec::new()) };
        static CENSO: RefCell<Vec<(&'static str, usize, usize)>> = const { RefCell::new(Vec::new()) };
    }

    /// Liga o cronómetro neste fio (a sonda), até ao [`tira`].
    pub fn liga() {
        LIGADO.with(|l| l.set(true));
    }

    pub(super) fn abre() {
        if LIGADO.with(Cell::get) {
            DESDE.with(|d| d.set(Some(Instant::now())));
        }
    }

    pub(super) fn marca(nome: &'static str) {
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

    /// As fases somadas por nome desde o [`liga`]; desliga.
    pub fn tira() -> Vec<(&'static str, Duration)> {
        LIGADO.with(|l| l.set(false));
        DESDE.with(|d| d.set(None));
        SOMA.with(|s| std::mem::take(&mut *s.borrow_mut()))
    }

    /// Ligado: soma `(plano, tiles iguais ao bit à grade, tiles com o plano)` do registo que a troca vai aplicar.
    pub(crate) fn soma_censo(plano: &'static str, igual: bool, tem: bool) {
        if !LIGADO.with(Cell::get) || !tem {
            return;
        }
        CENSO.with(|c| {
            let mut c = c.borrow_mut();
            match c.iter_mut().find(|e| e.0 == plano) {
                Some(e) => {
                    e.1 += usize::from(igual);
                    e.2 += 1;
                }
                None => c.push((plano, usize::from(igual), 1)),
            }
        });
    }

    /// O censo somado desde a última chamada.
    pub fn censo() -> Vec<(&'static str, usize, usize)> {
        CENSO.with(|c| std::mem::take(&mut *c.borrow_mut()))
    }

    pub(crate) fn ligado() -> bool {
        LIGADO.with(Cell::get)
    }
}

#[cfg(any(test, feature = "sonda-fases"))]
pub(crate) use relogio::{ligado, soma_censo};
