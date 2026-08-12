//! **OS KNOBS DE COMO A FORMA É LIDA** — irmão do [`super::rows`], cortado pelo
//! mesmo eixo que o `paint/brush.rs`: *o que o pincel FAZ* × *como a forma se
//! MOSTRA*.
//!
//! ⚠️ **Nenhum destes é um knob que o verbo armou**, e é por isso que o
//! interruptor `Basic`/`Pro` não os alcança (§2.3 do plano): a cavidade, a
//! lâmpada e os dois AOs descrevem a LEITURA da escultura, não o pincel em mãos.
//! Esconder um deles seria esconder um número que ninguém escolheu por você.

use ph2d_editor_core::ids;

use super::always;
use super::types::{Place, Row};
use crate::state::{Sculpt3dUi, UiLevel};

/// **Só com a luz do DOCUMENTO em uso.**
///
/// ⚠️ Um matcap é sombreamento função apenas da normal de VISTA: ele não lê o
/// rig, por definição. Deixar as duas pistas de lâmpada pintadas sob um matcap
/// seriam **dois controles que não fazem nada** — e não um pouco: o artista
/// arrastaria o ângulo da luz olhando uma escultura que não se move, que é a
/// forma mais cara de descobrir o que um modo significa.
fn under_the_rig(u: &Sculpt3dUi) -> bool {
    u.matcap.is_none()
}

/// Como a forma é LIDA — a cavidade e a lâmpada.
pub(super) static SHADING: &[Row] = &[
    Row {
        label: "panel.sculpt3d.cavity",
        slider: ids::SCULPT3D_CAVITY,
        chip: ids::SCULPT3D_CAVITY_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.cavity,
        set: |u, v| u.cavity = v,
        show: always,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.light_az",
        slider: ids::SCULPT3D_LIGHT_AZ,
        chip: ids::SCULPT3D_LIGHT_AZ_NUM,
        min: 0.0,
        // 359 e não 360: os dois extremos seriam o MESMO azimute, e uma pista
        // cujas duas pontas significam a mesma coisa tem um degrau invisível.
        max: 359.0, // LITERAL-PX-OK: graus de azimute, nao metrica de design
        step: 5.0,  // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| u.light_az_deg,
        set: |u, v| u.light_az_deg = v,
        show: under_the_rig,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.light_elev",
        slider: ids::SCULPT3D_LIGHT_ELEV,
        chip: ids::SCULPT3D_LIGHT_ELEV_NUM,
        // ⚠️ O piso é o do RESOLVEDOR de luz, não um número escolhido aqui:
        // abaixo dele a resposta plana vai a zero e o modelo relativo dividiria
        // por ~0. Um literal aqui seria a segunda cópia dele.
        min: MIN_ELEV_DEG_F32,
        max: 90.0, // LITERAL-PX-OK: graus de elevacao (o zenite), nao metrica de design
        step: 5.0, // LITERAL-PX-OK: passo em graus
        decimals: 0,
        get: |u| u.light_elev_deg,
        set: |u, v| u.light_elev_deg = v,
        show: under_the_rig,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.env",
        slider: ids::SCULPT3D_ENV,
        chip: ids::SCULPT3D_ENV_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.env,
        set: |u, v| u.env = v,
        // ⚠️ **Só sob o rig, e pela razão mais forte desta tabela: um matcap JÁ
        // É um ambiente.** Ele é uma esfera de iluminação capturada — o piso, o
        // céu e o realce dele vêm todos da mesma imagem —, então o termo do
        // estúdio não entra naquele caminho (o `mesh.wgsl` nem chega ao piso) e
        // o slider seria um controle que não faz nada. É a mesma lei das duas
        // pistas de lâmpada logo abaixo.
        show: under_the_rig,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.ao",
        slider: ids::SCULPT3D_AO,
        chip: ids::SCULPT3D_AO_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.ao,
        set: |u, v| u.ao = v,
        show: always,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.ssao",
        slider: ids::SCULPT3D_SSAO,
        chip: ids::SCULPT3D_SSAO_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.ssao,
        set: |u, v| u.ssao = v,
        show: always,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.sss",
        slider: ids::SCULPT3D_SSS,
        chip: ids::SCULPT3D_SSS_NUM,
        min: 0.0,
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.sss,
        set: |u, v| u.sss = v,
        show: always,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
    Row {
        label: "panel.sculpt3d.sss_scatter",
        slider: ids::SCULPT3D_SSS_SCATTER,
        chip: ids::SCULPT3D_SSS_SCATTER_NUM,
        min: 0.0,
        // ⚠️ **O teto é 1,0 = "a luz atravessa a peça inteira"**, e ele não é um
        // limite de recurso: é onde a grandeza deixa de descrever um sólido. O
        // default MEDIDO é 0,25 (`sss::SCATTER_FRACTION`), e a faixa existe para
        // o artista decidir o LOOK — que é a única pergunta que a medição não
        // responde.
        max: 1.0,
        step: 0.05, // LITERAL-PX-OK: passo de um knob adimensional, não métrica de layout
        decimals: 2,
        get: |u| u.sss_scatter,
        set: |u, v| u.sss_scatter = v,
        // ⚠️ **Só com o espalhamento LIGADO.** Com a força em zero a tabela nem é
        // consultada, então este slider não moveria um pixel — e um controle que
        // não faz nada é o que esta casa varre a cada wave. É a mesma lei do
        // `Plane Offset`, que só existe nos verbos que leem um plano.
        show: |u| u.sss > 0.0,
        level: UiLevel::Basic,
        place: Place::Knobs,
    },
];

/// O piso da elevação, lido do dono dele.
const MIN_ELEV_DEG_F32: f32 = ph2d_light::MIN_ELEV_DEG as f32; // CLAMP-OK: piso do resolvedor de luz
