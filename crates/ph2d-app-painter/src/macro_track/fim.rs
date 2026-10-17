//! **A imagem ao fim da gravação, fora do quadro** (W7 item 1, `docs/Macros/04_plano_W7.md`): a régua
//! de «o artista pintou depois do Parar». Calcula-se num fio próprio por versão da trilha CRUA do
//! Painter (o estado inicial e os eventos: editar uma etapa não a muda); largar o trabalho pára o fio.

use ph2d_macro::Track;
use ph2d_tool_painter::macro_track::etapas::imagem_no_fim_da_gravacao;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// Uma imagem RGBA8 premultiplicada, com o tamanho.
pub(super) struct Imagem {
    pub(super) rgba: Vec<u8>,
    pub(super) w: u32,
    pub(super) h: u32,
}

/// `None` = a reprodução falhou ou parou: fica o palco sozinho.
type Resultado = Option<Arc<Imagem>>;

enum Estado {
    PorArrancar(Track),
    Corre {
        parar: Arc<AtomicBool>,
        fio: JoinHandle<Resultado>,
    },
    Pronto(Resultado),
}

pub(super) struct FimDaGravacao {
    /// A versão da trilha crua ([`versao_da_trilha`]).
    pub(super) versao: u64,
    estado: Estado,
}

impl Drop for FimDaGravacao {
    fn drop(&mut self) {
        if let Estado::Corre { parar, .. } = &self.estado {
            parar.store(true, Ordering::Relaxed);
        }
    }
}

/// O hash do que a imagem do fim lê da trilha: o estado inicial e os eventos.
pub(super) fn versao_da_trilha(t: &Track) -> u64 {
    let mut h = DefaultHasher::new();
    t.initial.hash(&mut h);
    for e in &t.events {
        e.t.hash(&mut h);
        e.payload.hash(&mut h);
    }
    h.finish()
}

fn calcula(t: &Track, parar: &AtomicBool) -> Resultado {
    let (mut rgba, w, h) = imagem_no_fim_da_gravacao(t, parar).ok()??;
    ph2d_render::premul::premultiply_rgba8(&mut rgba);
    Some(Arc::new(Imagem { rgba, w, h }))
}

impl FimDaGravacao {
    pub(super) fn de(track: Track) -> Self {
        Self {
            versao: versao_da_trilha(&track),
            estado: Estado::PorArrancar(track),
        }
    }

    /// Arranca o fio (uma vez) e colhe o que ele acabou. `Some` = a imagem chegou.
    pub(super) fn colhe(&mut self) -> Option<&Arc<Imagem>> {
        if let Estado::PorArrancar(_) = &self.estado
            && let Estado::PorArrancar(t) =
                std::mem::replace(&mut self.estado, Estado::Pronto(None))
        {
            let parar = Arc::new(AtomicBool::new(false));
            let p = Arc::clone(&parar);
            self.estado = match std::thread::Builder::new()
                .name("macro-fim".into())
                .spawn(move || calcula(&t, &p))
            {
                Ok(fio) => Estado::Corre { parar, fio },
                // O fio não arrancou: fica o palco sozinho.
                Err(_) => Estado::Pronto(None),
            };
        }
        if let Estado::Corre { fio, .. } = &self.estado
            && fio.is_finished()
            && let Estado::Corre { fio, .. } =
                std::mem::replace(&mut self.estado, Estado::Pronto(None))
        {
            self.estado = Estado::Pronto(fio.join().ok().flatten());
        }
        match &self.estado {
            Estado::Pronto(r) => r.as_ref(),
            _ => None,
        }
    }

    /// O fio acabou (com a imagem ou sem ela).
    #[cfg(test)]
    pub(super) fn acabou(&self) -> bool {
        matches!(self.estado, Estado::Pronto(_))
    }
}
