//! **A imagem do fim da gravação, fora do quadro** (W7 item 1, `docs/Macros/04_plano_W7.md`; W8 itens
//! 2b/2c/4, `docs/Macros/05_plano_W8.md`): a régua da reescrita de uma etapa editada — só ela a pede
//! desde a W9 (a mistura é pela marca). Calcula-se num fio próprio por versão da trilha QUE TOCA (o estado inicial e os eventos com as
//! edições das etapas; o nome e a dica não a mudam), com a água assentada; largar o trabalho pára o
//! fio. Um erro da reprodução ou um pânico do fio dizem-se ao artista ([`FimDaGravacao::erro`]).

use ph2d_macro::Track;
use ph2d_tool_painter::macro_track::etapas::{LeitorDoPainter, ModeloDoFim, imagens_da_gravacao};
use ph2d_tool_painter::macro_track::tocar::ErroDaMacro;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

/// Uma imagem RGBA8 premultiplicada, com o tamanho.
#[derive(Clone)]
pub(super) struct Imagem {
    pub(super) rgba: Vec<u8>,
    pub(super) w: u32,
    pub(super) h: u32,
}

/// A imagem do fim da gravação, premultiplicada; o fim straight (o que se escreve numa sprite) e o
/// documento do fim — a régua da reescrita de uma etapa editada (W8 item 2b).
pub(super) struct Gravacao {
    pub(super) fim: Imagem,
    pub(super) fim_reto: Vec<u8>,
    pub(super) modelo: ModeloDoFim,
}

/// `Ok(None)` = parou (o trabalho largou-se).
pub(super) type Resultado = Result<Option<Arc<Gravacao>>, ErroDaMacro>;

/// O cálculo do fim de uma trilha (o do produto, [`calcula`]; os gates trocam-no).
pub(super) type Calculo = fn(&Track, &AtomicBool) -> Resultado;

enum Estado {
    PorArrancar(Track),
    Corre {
        parar: Arc<AtomicBool>,
        fio: JoinHandle<Resultado>,
    },
    Pronto(Result<Arc<Gravacao>, ErroDaMacro>),
}

pub(super) struct FimDaGravacao {
    /// A versão da trilha que toca ([`versao_da_trilha`]).
    pub(super) versao: u64,
    /// A versão da gravação crua ([`versao_crua`]).
    pub(super) crua: u64,
    estado: Estado,
}

impl Drop for FimDaGravacao {
    fn drop(&mut self) {
        if let Estado::Corre { parar, .. } = &self.estado {
            parar.store(true, Ordering::Relaxed);
        }
    }
}

/// O hash da gravação CRUA (o estado inicial e os eventos, sem as edições das etapas): a mesma em duas
/// versões ⇒ a mudança foi uma edição de etapa (a reescrita da imagem, W8 item 2b).
pub(super) fn versao_crua(t: &Track) -> u64 {
    let mut h = DefaultHasher::new();
    t.initial.hash(&mut h);
    for e in &t.events {
        e.t.hash(&mut h);
        e.payload.hash(&mut h);
    }
    h.finish()
}

/// O hash do que a imagem do fim lê da trilha: o estado inicial e os eventos da trilha que toca. Uma
/// edição que não se toca entra como a trilha crua (a reprodução dela dá o erro).
pub(super) fn versao_da_trilha(t: &Track) -> u64 {
    let mut h = DefaultHasher::new();
    t.initial.hash(&mut h);
    let tocavel = t.tocavel(&LeitorDoPainter);
    for e in tocavel.as_ref().map_or(&t.events, |x| &x.events) {
        e.t.hash(&mut h);
        e.payload.hash(&mut h);
    }
    tocavel.is_ok().hash(&mut h);
    h.finish()
}

pub(super) fn calcula(t: &Track, parar: &AtomicBool) -> Resultado {
    let Some(g) = imagens_da_gravacao(t, parar)? else {
        return Ok(None);
    };
    let mut rgba = g.fim.clone();
    ph2d_render::premul::premultiply_rgba8(&mut rgba);
    Ok(Some(Arc::new(Gravacao {
        fim: Imagem {
            rgba,
            w: g.w,
            h: g.h,
        },
        fim_reto: g.fim,
        modelo: g.modelo,
    })))
}

/// O nome do fio (o que o pânico diz).
const FIO: &str = "macro-fim";

impl FimDaGravacao {
    pub(super) fn de(track: Track) -> Self {
        Self {
            versao: versao_da_trilha(&track),
            crua: versao_crua(&track),
            estado: Estado::PorArrancar(track),
        }
    }

    /// Arranca o fio (uma vez) e colhe o que ele acabou. `Some` = a imagem chegou.
    pub(super) fn colhe(&mut self) -> Option<&Arc<Gravacao>> {
        self.colhe_com(calcula)
    }

    /// [`Self::colhe`] com o cálculo `f` (os gates fazem-no falhar).
    pub(super) fn colhe_com(&mut self, f: Calculo) -> Option<&Arc<Gravacao>> {
        if let Estado::PorArrancar(_) = &self.estado
            && let Estado::PorArrancar(t) = std::mem::replace(
                &mut self.estado,
                Estado::Pronto(Err(ErroDaMacro::FioCaiu(FIO))),
            )
        {
            let parar = Arc::new(AtomicBool::new(false));
            let p = Arc::clone(&parar);
            // O fio que não arranca fica o erro acima.
            if let Ok(fio) = std::thread::Builder::new()
                .name(FIO.into())
                .spawn(move || f(&t, &p))
            {
                self.estado = Estado::Corre { parar, fio };
            }
        }
        if let Estado::Corre { fio, .. } = &self.estado
            && fio.is_finished()
            && let Estado::Corre { fio, .. } = std::mem::replace(
                &mut self.estado,
                Estado::Pronto(Err(ErroDaMacro::FioCaiu(FIO))),
            )
        {
            self.estado = Estado::Pronto(match fio.join() {
                Ok(Ok(Some(g))) => Ok(g),
                Ok(Err(e)) => Err(e),
                // Parar só se pede ao largar o trabalho: aqui é um pânico ou o fio que parou sozinho.
                Ok(Ok(None)) | Err(_) => Err(ErroDaMacro::FioCaiu(FIO)),
            });
        }
        match &self.estado {
            Estado::Pronto(Ok(g)) => Some(g),
            _ => None,
        }
    }

    /// Porque a imagem do fim não veio: a reprodução falhou ou o fio caiu.
    pub(super) fn erro(&self) -> Option<&ErroDaMacro> {
        match &self.estado {
            Estado::Pronto(Err(e)) => Some(e),
            _ => None,
        }
    }

    /// O fio acabou (com a imagem ou sem ela).
    #[cfg(test)]
    pub(super) fn acabou(&self) -> bool {
        matches!(self.estado, Estado::Pronto(_))
    }
}
