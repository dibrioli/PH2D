//! ⭐⭐⭐ **A MISTURA EM GRUPO PELA ROTA DO DISPOSITIVO** (B6, família B do Motion) — a cerca que
//! substitui a recusa incondicional de um sink que mistura em grupo (doc 118 W4).
//!
//! O dispositivo entrega as IMAGENS ao passe de sprites, que só sabe a mistura de hardware (luz
//! linear, sem alcance) — por isso o grupo inteiro ia à CPU. Mas as FORMAS do cozimento vão à placa
//! de formas, que desde o B5 mistura por cópia como o Vello ([`crate::motion_shape_placa::Misturas`]).
//! ⇒ um sink que mistura em grupo vai ao dispositivo quando:
//! - a placa compõe a mistura dele ([`Misturas::da_mistura`] — as MESMAS regras da rota da CPU:
//!   `Copies` sempre; `Everything` só na camada isolada; `Multiply` só com `DUAL_SOURCE_BLENDING`);
//! - as linhas dele são TODAS formas. O `geometry_id` é um VALOR da placa ⇒ a memória da CPU (a
//!   última corrente que a bomba cozeu para o sink) decide antes de cozer — sem ela, a CPU coze
//!   primeiro —, e a vigia do cozimento ([`ph2d_gpu_cook::GpuCook::toma_imagens_vigiadas`]) devolve
//!   o sink à CPU no quadro seguinte se uma imagem aparecer depois.
//!
//! As corridas (uma por faixa de sink no buffer das formas) saem de
//! [`crate::motion_shape_placa::corridas_do_dispositivo`], sem leitura de volta.

use crate::motion_shape_placa::{Misturas, MotivoDoVello};
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::graph::NodeId;
use ph2d_nodegraph::value::CookValue;

/// Uma linha de imagem num sink que mistura em grupo — o passe de sprites não a mistura certo.
pub(crate) const RECUSA_IMAGEM_EM_GRUPO: &str = "CPU: uma linha de IMAGEM num sink que mistura em grupo -- o passe de sprites nao sabe o alcance nem o tom (doc 118)";
/// O 1.º quadro de um grupo: a CPU ainda não o cozeu, logo não se sabe se tem imagens.
pub(crate) const RECUSA_GRUPO_SEM_MEMORIA: &str = "CPU: o sink mistura em grupo e a CPU ainda nao o cozeu -- este quadro mostra se ele e' so' de formas";
/// O grupo mistura com o mundo por baixo: a camada da placa começa vazia.
pub(crate) const RECUSA_MISTURA_PEDE_O_MUNDO: &str = "CPU: o sink mistura em grupo com o mundo por baixo (fora da camada isolada do Motion) -- so a cena vectorial o sabe (doc 118)";
/// Multiply sem a 2.ª saída do fragmento.
pub(crate) const RECUSA_MULTIPLY_SEM_SEGUNDA_SAIDA: &str =
    "CPU: o sink multiplica em grupo e este dispositivo nao tem DUAL_SOURCE_BLENDING";
/// As misturas por cópia desligadas na placa.
pub(crate) const RECUSA_MISTURAS_DESLIGADAS: &str =
    "CPU: o sink mistura em grupo com as misturas por copia da placa desligadas";

/// **Este sink mistura em grupo e pode ir ao dispositivo?** — `None` = vai; senão o motivo da CPU.
/// `em_grupo` = um por [`MotionState::sinks`]; `imagens_vistas` = a última leitura da vigia.
pub(crate) fn recusa_da_mistura_em_grupo(
    motion: &MotionState,
    misturas: Misturas,
    em_grupo: &[bool],
    imagens_vistas: u32,
) -> Option<&'static str> {
    let isolada = motion.pump.camada_isolada();
    for (&sink, _) in motion.sinks.iter().zip(em_grupo).filter(|(_, g)| **g) {
        let mistura = crate::motion_shape_placa::mistura_no_quadro(
            &motion.doc.graph,
            &motion.sinks,
            sink,
            isolada,
        );
        if let Err(m) = misturas.da_mistura(0, &mistura) {
            return Some(match m {
                MotivoDoVello::Desligadas => RECUSA_MISTURAS_DESLIGADAS,
                MotivoDoVello::PedeOMundo => RECUSA_MISTURA_PEDE_O_MUNDO,
                MotivoDoVello::SemMultiplica => RECUSA_MULTIPLY_SEM_SEGUNDA_SAIDA,
            });
        }
        match linhas_todas_forma(&motion.pump.cook, sink) {
            None => return Some(RECUSA_GRUPO_SEM_MEMORIA),
            Some(false) => return Some(RECUSA_IMAGEM_EM_GRUPO),
            Some(true) if imagens_vistas > 0 => return Some(RECUSA_IMAGEM_EM_GRUPO),
            Some(true) => {}
        }
    }
    None
}

/// **A memória da CPU diz que as linhas deste sink são todas formas?** — a corrente que a bomba
/// cozeu da última vez (`Cook::peek`). ⚠️ `None` = sem memória (a CPU nunca cozeu o sink): a CPU
/// coze o primeiro quadro, e o seguinte decide com ela.
pub(crate) fn linhas_todas_forma(cook: &ph2d_nodegraph::cook::Cook, sink: NodeId) -> Option<bool> {
    let Some([CookValue::Instances(st), ..]) = cook.peek(sink) else {
        return None;
    };
    // `> 0,5` — o `MediaColumns::at` da CPU e o baixamento do dispositivo.
    Some(match st.get("geometry_id") {
        Some(Column::Scalar(v)) => v.iter().all(|&g| g > 0.5),
        _ => st.is_empty(),
    })
}
