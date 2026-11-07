//! Resto de compatibilidade com o `.ph2dproj` do shell (`project_load_read.rs`): o fio é um só.
//! Estas portas não fazem nada e saem junto com as chamadas do shell (limpeza §0.11, 07/11).

/// Não faz nada: existe só enquanto o shell a chama.
pub fn with_wire_layout<T>(_ver: u32, f: impl FnOnce() -> T) -> T {
    f()
}

/// Não faz nada: existe só enquanto o shell a abre.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct WireLayout(());

impl WireLayout {
    pub fn enter(_ver: u32) -> Self {
        Self(())
    }
}

/// Não faz nada: existe só enquanto o shell a abre.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct PilhaLayoutV23(());

impl PilhaLayoutV23 {
    pub fn enter() -> Self {
        Self(())
    }
}
