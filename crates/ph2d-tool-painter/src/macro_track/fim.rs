//! **O fim de uma trilha que toca**: todos os passos da trilha com as edições das etapas, e a água
//! assentada ([`super::etapas::QUADROS_ATE_A_AGUA_DO_FIM_PARAR`]) — a porta da imagem do fim da gravação
//! ([`super::etapas::imagens_da_gravacao`]) e do documento acabado que aplicar uma macro instala (W18).

use super::encaixe::Encaixe;
use super::etapas::{LeitorDoPainter, QUADROS_ATE_A_AGUA_DO_FIM_PARAR};
use super::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::PaintedDocument;
use ph2d_macro::Track;
use std::sync::atomic::{AtomicBool, Ordering};

/// Uma imagem RGBA8 e o tamanho dela.
type Imagem = (Vec<u8>, u32, u32);

/// O tocador de `track` no fim, com o encaixe `encaixe`, e a imagem dele no estado inicial (vazia sem
/// tela). `Ok(None)` = `parar`.
///
/// # Errors
/// O [`ErroDaMacro`] da reprodução.
pub(crate) fn tocador_no_fim(
    track: &Track,
    encaixe: Option<Encaixe>,
    parar: &AtomicBool,
) -> Result<Option<(TocadorDoPainter, Imagem)>, ErroDaMacro> {
    let tocavel = track
        .tocavel(&LeitorDoPainter)
        .map_err(ErroDaMacro::Edicao)?;
    let mut toc = TocadorDoPainter::novo(
        &tocavel.initial,
        tocavel.events.iter().map(|e| e.payload.as_slice()),
    )?;
    toc.poe_encaixe(encaixe);
    // ⛔ Nunca o `run_full`: numa pilha trivial ele LEVA a tela, e o fim saía vazio (W7).
    let inicio = toc
        .tool()
        .imagem_aplicada()
        .unwrap_or_else(|| (Vec::new(), 0, 0));
    for e in &tocavel.events {
        if parar.load(Ordering::Relaxed) {
            return Ok(None);
        }
        toc.aplica(&e.payload)?;
    }
    toc.passa_ao_relogio_fixo();
    let t = toc.tool_mut();
    for _ in 0..QUADROS_ATE_A_AGUA_DO_FIM_PARAR {
        if t.wet_assentada() {
            break;
        }
        if parar.load(Ordering::Relaxed) {
            return Ok(None);
        }
        t.drena_sem_compor();
        ph2d_editor_core::tool::Tool::on_tick(t, super::roteiro::DT_MS);
    }
    Ok(Some((toc, inicio)))
}

/// **O documento no fim da trilha que toca**, com a água assentada — aplicar uma macro «novo» instala-o
/// no editor (a pintura acabada); por cima, com o `encaixe` do objecto Macro.
///
/// # Errors
/// O [`ErroDaMacro`] da reprodução.
pub fn documento_no_fim(
    trilha: &Track,
    encaixe: Option<Encaixe>,
) -> Result<PaintedDocument, ErroDaMacro> {
    let (toc, _) = tocador_no_fim(trilha, encaixe, &AtomicBool::new(false))?
        .ok_or(ErroDaMacro::FioCaiu("macro-fim"))?;
    Ok(toc.tool().documento_vivo(0))
}
