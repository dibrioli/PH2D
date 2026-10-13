//! ⭐⭐⭐ **A MISTURA EM GRUPO PELA ROTA DO DISPOSITIVO** (B6 · P4, família B do Motion) — a cerca de
//! um sink que mistura em grupo (doc 118 W4).
//!
//! As FORMAS do cozimento vão à placa de formas, que mistura por cópia como o Vello
//! ([`crate::motion_shape_placa::Misturas`]); desde o P4 (doc 132 §4) as IMAGENS de uma saída em grupo
//! também ([`ph2d_gpu_cook::GpuCook::separa_as_imagens_do_grupo`] → `motion_shape_placa::imagens`), com
//! o MESMO alcance e no MESMO tom. ⇒ a decisão já não depende de valores que só a placa vê: nem a
//! memória da CPU nem a vigia que devolvia o sink um a dois quadros depois. Um sink em grupo vai ao
//! dispositivo quando a placa compõe a mistura dele ([`Misturas::composicao`] — as MESMAS regras da
//! rota da CPU) e a camada pousa no mundo UMA vez ([`PousaDoQuadro`]); o `Scene` fora da camada
//! isolada fica na CPU (as cópias juntam-se em `Normal`, e a ordem entre imagens e formas é a das
//! linhas, que o dispositivo separa).
//!
//! As corridas (uma por faixa de sink nos buffers das formas e das imagens) saem de
//! [`crate::motion_shape_placa::corridas_do_dispositivo`], sem leitura de volta.

use crate::motion_shape_placa::{Misturas, MotivoDoVello, PousaDoQuadro};
use crate::motion_state::MotionState;

/// A camada teria de pousar no mundo mais de uma vez.
pub(crate) const RECUSA_MUNDO_PARTIDO: &str = "CPU: a mistura com o mundo pede mais de uma colagem da camada (saidas que pousam de modos diferentes, ou mais de um Scene) -- so a cena vectorial o sabe (doc 132)";
/// O `Scene` fora da camada isolada: a ordem das linhas entre imagens e formas.
pub(crate) const RECUSA_SCENE_NO_DISPOSITIVO: &str = "CPU: o sink mistura o grupo com o mundo (Scene) -- as copias juntam-se em Normal pela ordem das linhas, e o dispositivo separa imagens de formas (doc 132)";
/// Multiply sem a 2.ª saída do fragmento.
pub(crate) const RECUSA_MULTIPLY_SEM_SEGUNDA_SAIDA: &str =
    "CPU: o sink multiplica em grupo e este dispositivo nao tem DUAL_SOURCE_BLENDING";
/// As misturas por cópia desligadas na placa.
pub(crate) const RECUSA_MISTURAS_DESLIGADAS: &str =
    "CPU: o sink mistura em grupo com as misturas por copia da placa desligadas";

/// **Os sinks que misturam em grupo podem ir ao dispositivo?** — `None` = vão; senão o motivo da
/// CPU. `em_grupo` = um por [`MotionState::sinks`]. ⚠️ A pousa junta TODOS os sinks: um que não
/// mistura pousa em `Normal`, e ao lado de um que pousa com o mundo partiria a camada.
pub(crate) fn recusa_da_mistura_em_grupo(
    motion: &MotionState,
    misturas: Misturas,
    em_grupo: &[bool],
) -> Option<&'static str> {
    if !em_grupo.iter().any(|&g| g) {
        return None;
    }
    let isolada = motion.pump.camada_isolada();
    let mut pousa = PousaDoQuadro::default();
    for (&sink, &grupo) in motion.sinks.iter().zip(em_grupo) {
        let mistura = crate::motion_shape_placa::mistura_no_quadro(
            &motion.doc.graph,
            &motion.sinks,
            sink,
            isolada,
        );
        let c = match misturas.composicao(0, &mistura) {
            Ok(c) if grupo && c.cena.is_some() => return Some(RECUSA_SCENE_NO_DISPOSITIVO),
            Ok(c) => c,
            Err(m) => return Some(motivo(m)),
        };
        if let Err(m) = pousa.junta(&c) {
            return Some(motivo(m));
        }
    }
    None
}

fn motivo(m: MotivoDoVello) -> &'static str {
    match m {
        MotivoDoVello::Desligadas => RECUSA_MISTURAS_DESLIGADAS,
        MotivoDoVello::MundoPartido => RECUSA_MUNDO_PARTIDO,
        MotivoDoVello::CenaComImagens => RECUSA_SCENE_NO_DISPOSITIVO,
        MotivoDoVello::SemMultiplica => RECUSA_MULTIPLY_SEM_SEGUNDA_SAIDA,
    }
}
