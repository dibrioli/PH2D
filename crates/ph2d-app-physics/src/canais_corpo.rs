//! ⭐⭐ **A família FISICA nos canais** (plano 35 §A.5) — os campos do corpo, do colisor e da zona
//! que a §11 *Physics Body* do Inspector edita: [`publicar`] o que o mundo vale em `CanaisLidos`,
//! [`aplicar`] o que a curva mandou em `CanaisEscritos` (com o eco).
//!
//! ⭐ **A escrita é a da edição do painel, as DUAS metades dela**: o campo vira o controlo da §11
//! que o mostra (a opção de uma escolha, ou o número na unidade do painel) e passa pela porta
//! única do evento ([`ph2d_panel_inspector::physics_edit_for`] — as mesmas guardas: Dynamic-only,
//! sensor-only, a forma que o painel mostra) e depois pelo funil do Shell
//! ([`crate::physics_apply::apply_physics_edit`] + `apply_editor_commands`) — as conversões de
//! forma, os pisos, a presença dos opcionais.
//!
//! ⚠️ **Runtime-truth (ADR-0131 D2)**: daí ao rapier o caminho é o da edição do painel — o
//! `reconcile_structure` da ponte re-descreve um corpo cujo `BodyDesc` mudou **em repouso**
//! (antes do 1.º passo, depois de rebobinar). Com a simulação a correr, a curva escreve o
//! componente e o corpo vivo não muda, exactamente como uma edição no painel nesse instante.
//!
//! ⭐ **A escrita da curva é pré-visualização**: o valor AUTORADO de cada campo que a curva move
//! fica em [`ConducaoFisica`] (a irmã da `ConducaoVetor`), e a fotografia do undo e do save vê-o.

use std::collections::{BTreeMap, BTreeSet};

use ph2d_ecs::canal::familia::FISICA;
use ph2d_ecs::canal_fisica as f;
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::{InspectorPhysicsInfo, PhysicsFieldEdit};
use ph2d_panel_inspector::ids as insp;

/// O controlo da §11 que mostra um campo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Porta {
    /// Um número (o valor na unidade do painel).
    Numero(NodeId),
    /// Uma escolha: a opção `i` é o valor `i`.
    Escolha(&'static [NodeId]),
}

fn porta(campo: u16) -> Option<Porta> {
    use Porta::{Escolha, Numero};
    Some(match campo {
        f::TIPO => Escolha(&insp::INSP_PHYS_KIND),
        f::CCD => Escolha(&insp::INSP_PHYS_CCD),
        f::TRAVA_ROTACAO => Escolha(&insp::INSP_PHYS_LOCKROT),
        f::TRAVA_X => Escolha(&insp::INSP_PHYS_LOCKX),
        f::TRAVA_Y => Escolha(&insp::INSP_PHYS_LOCKY),
        f::ESCALA_GRAVIDADE => Numero(insp::INSP_PHYS_GRAVITY_SCALE),
        f::DOMINANCIA => Numero(insp::INSP_PHYS_DOMINANCE),
        f::VEL_X => Numero(insp::INSP_PHYS_LINVEL_X),
        f::VEL_Y => Numero(insp::INSP_PHYS_LINVEL_Y),
        f::VEL_ANGULAR => Numero(insp::INSP_PHYS_ANGVEL),
        f::AMORT_LINEAR => Numero(insp::INSP_PHYS_LINEAR_DAMPING),
        f::AMORT_ANGULAR => Numero(insp::INSP_PHYS_ANGULAR_DAMPING),
        f::AMORT_MODO => Escolha(&insp::INSP_PHYS_DAMPMODE),
        f::MASSA_MANUAL => Escolha(&insp::INSP_PHYS_MASSMODE),
        f::MASSA => Numero(insp::INSP_PHYS_MASS),
        f::FORMA => Escolha(&insp::INSP_PHYS_SHAPE),
        f::RAIO => Numero(insp::INSP_PHYS_RADIUS),
        f::MEIO_X => Numero(insp::INSP_PHYS_HALF_X),
        f::MEIO_Y => Numero(insp::INSP_PHYS_HALF_Y),
        f::CAPSULA => Numero(insp::INSP_PHYS_CAP_HALF_H),
        f::DESLOC_X => Numero(insp::INSP_PHYS_OFFSET_X),
        f::DESLOC_Y => Numero(insp::INSP_PHYS_OFFSET_Y),
        f::DENSIDADE => Numero(insp::INSP_PHYS_DENSITY),
        f::RESTITUICAO => Numero(insp::INSP_PHYS_RESTITUTION),
        f::ATRITO => Numero(insp::INSP_PHYS_FRICTION),
        f::COMBINA_RESTITUICAO => Escolha(&insp::INSP_PHYS_REST_COMBINE),
        f::COMBINA_ATRITO => Escolha(&insp::INSP_PHYS_FRIC_COMBINE),
        f::CAMADA => Escolha(&insp::INSP_PHYS_LAYER),
        f::SENSOR => Escolha(&insp::INSP_PHYS_SENSOR),
        f::UMA_VIA => Escolha(&insp::INSP_PHYS_ONEWAY),
        f::SEM_PAREDE => Escolha(&insp::INSP_PHYS_WALLMAT),
        f::ADERENCIA => Numero(insp::INSP_PHYS_WALK_GRIP),
        f::ESTEIRA => Numero(insp::INSP_PHYS_WALK_BELT),
        f::FORCA_X => Numero(insp::INSP_PHYS_FORCE_X),
        f::FORCA_Y => Numero(insp::INSP_PHYS_FORCE_Y),
        f::FORCA_NO_MUNDO => Escolha(&insp::INSP_PHYS_FORCE_AXES),
        f::ZONA_ARRASTO => Numero(insp::INSP_PHYS_AREA_DRAG),
        f::ZONA_DENSIDADE => Numero(insp::INSP_PHYS_AREA_DENSITY),
        f::ZONA_ARRASTO_FORMA => Numero(insp::INSP_PHYS_AREA_FORM_DRAG),
        f::ZONA_TORQUE => Numero(insp::INSP_PHYS_AREA_TORQUE),
        f::ZONA_QUEDA => Numero(insp::INSP_PHYS_AREA_FALLOFF),
        _ => return None,
    })
}

const fn k(campo: u16) -> FieldKey {
    FieldKey::new(FISICA, campo, 0, 0)
}

fn bit(b: bool) -> f32 {
    if b { 1.0 } else { 0.0 }
}

/// Quanto vale `campo` no retrato da §11 — `None` fora do vocabulário.
#[must_use]
pub fn ler(info: &InspectorPhysicsInfo, campo: u16) -> Option<f32> {
    Some(match campo {
        f::TIPO => f32::from(info.kind_tag),
        f::CCD => bit(info.ccd),
        f::TRAVA_ROTACAO => bit(info.lock_rotation),
        f::TRAVA_X => bit(info.lock_x),
        f::TRAVA_Y => bit(info.lock_y),
        f::ESCALA_GRAVIDADE => info.gravity_scale,
        f::DOMINANCIA => f32::from(info.dominance),
        f::VEL_X => info.linvel[0],
        f::VEL_Y => info.linvel[1],
        f::VEL_ANGULAR => info.angvel,
        f::AMORT_LINEAR => info.linear_damping,
        f::AMORT_ANGULAR => info.angular_damping,
        f::AMORT_MODO => f32::from(info.damp_mode_tag),
        f::MASSA_MANUAL => bit(info.mass_manual),
        f::MASSA => info.mass,
        f::FORMA => f32::from(info.shape_tag),
        f::RAIO => info.radius,
        f::MEIO_X => info.half_x,
        f::MEIO_Y => info.half_y,
        f::CAPSULA => info.cap_half_height,
        f::DESLOC_X => info.offset[0],
        f::DESLOC_Y => info.offset[1],
        f::DENSIDADE => info.density,
        f::RESTITUICAO => info.restitution,
        f::ATRITO => info.friction,
        f::COMBINA_RESTITUICAO => f32::from(info.restitution_combine_tag),
        f::COMBINA_ATRITO => f32::from(info.friction_combine_tag),
        f::CAMADA => f32::from(info.layer),
        f::SENSOR => bit(info.is_sensor),
        f::UMA_VIA => bit(info.one_way),
        f::SEM_PAREDE => bit(info.no_wall_cling),
        f::ADERENCIA => info.walk_grip,
        f::ESTEIRA => info.walk_belt,
        f::FORCA_X => info.force[0],
        f::FORCA_Y => info.force[1],
        f::FORCA_NO_MUNDO => bit(info.force_world_axes),
        f::ZONA_ARRASTO => info.area_drag,
        f::ZONA_DENSIDADE => info.area_density,
        f::ZONA_ARRASTO_FORMA => info.area_form_drag,
        f::ZONA_TORQUE => info.area_torque,
        f::ZONA_QUEDA => info.area_falloff,
        _ => return None,
    })
}

/// O índice de uma escolha de `n` opções (`n ≤ 64`, a maior lista da §11): arredonda e prende.
fn opcao(v: f32, n: usize) -> u8 {
    let max = u8::try_from(n.saturating_sub(1)).unwrap_or(u8::MAX);
    if v.is_finite() {
        v.round().clamp(0.0, f32::from(max)) as u8
    } else {
        0
    }
}

/// ⭐ **A edição da §11 que escreve `v` em `campo`** — o controlo do painel e a porta única do
/// evento dele (as guardas do painel). `None` quando o painel não aceitaria.
#[must_use]
pub fn edicao(info: &InspectorPhysicsInfo, campo: u16, v: f32) -> Option<PhysicsFieldEdit> {
    match porta(campo)? {
        Porta::Numero(id) => {
            // O painel fala graus na rotação inicial; o componente, radianos.
            let ui = if campo == f::VEL_ANGULAR {
                v.to_degrees()
            } else {
                v
            };
            ph2d_panel_inspector::physics_edit_for(id, Some(ui), info)
        }
        Porta::Escolha(ops) => {
            let id = *ops.get(usize::from(opcao(v, ops.len())))?;
            ph2d_panel_inspector::physics_edit_for(id, None, info)
        }
    }
}

/// `true` se a §11 mostra `campo` para este objecto agora — a face do corpo, ou a da PEÇA (um
/// colisor sem corpo: forma, medidas, deslocamento, densidade); e a densidade OU a massa, pela
/// fonte da massa. O resto é a guarda do próprio evento.
#[must_use]
pub fn mostrado(info: &InspectorPhysicsInfo, campo: u16) -> bool {
    if !info.has_collider {
        return false;
    }
    let da_peca = matches!(
        campo,
        f::FORMA
            | f::RAIO
            | f::MEIO_X
            | f::MEIO_Y
            | f::CAPSULA
            | f::DESLOC_X
            | f::DESLOC_Y
            | f::DENSIDADE
    );
    if !info.has_body && !da_peca {
        return false;
    }
    let massa_manual = info.mass_is_read && info.mass_manual;
    match campo {
        f::DENSIDADE if massa_manual => false,
        f::MASSA => massa_manual,
        _ => ler(info, campo).is_some_and(|v| edicao(info, campo, v).is_some()),
    }
}

/// O retrato da §11 de `bits` — só os números (os factos que só a shell vê ficam neutros).
#[must_use]
pub fn retrato(sim: &SimWorld, bits: u64) -> Option<InspectorPhysicsInfo> {
    crate::inspector::body::build_physics_info(
        sim.world(),
        bits,
        0,
        0,
        0,
        false,
        0,
        (0.0, 0.0),
        0,
        &ph2d_tags::TagTree::default(),
    )
}

/// O valor que o documento guardaria para `v` (o índice de uma escolha).
fn normal(campo: u16, v: f32) -> f32 {
    match porta(campo) {
        Some(Porta::Escolha(ops)) => f32::from(opcao(v, ops.len())),
        _ => v,
    }
}

/// Escreve `v` em `campo` pelos dois funis do painel; devolve `(antes, ficou)`.
fn escrever(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    bits: u64,
    campo: u16,
    v: f32,
) -> Option<(f32, f32)> {
    let info = retrato(sim, bits)?;
    let antes = ler(&info, campo)?;
    if !mostrado(&info, campo) || antes == normal(campo, v) {
        return Some((antes, antes));
    }
    let Some(ed) = edicao(&info, campo, v) else {
        return Some((antes, antes));
    };
    let fila = EditorCommandQueue::new();
    crate::physics_apply::apply_physics_edit(sim, bits, ed, &fila, registry);
    // Uma recusa do registo deixa o componente como estava — o eco diz o que ficou.
    let _ = apply_editor_commands(sim.world_mut(), &fila, registry);
    let ficou = retrato(sim, bits).and_then(|i| ler(&i, campo))?;
    Some((antes, ficou))
}

/// ⭐ **Publica o que cada corpo vale AGORA** (através do eco) — os campos que a §11 mostra —,
/// esquecido antes o que cada entidade publicou desta família.
pub fn publicar(sim: &mut SimWorld, entidades: &[u64]) {
    let lotes: Vec<(u64, FieldKey, f32)> = entidades
        .iter()
        .filter_map(|&b| Some((b, retrato(sim, b)?)))
        .flat_map(|(b, info)| {
            f::CAMPOS
                .iter()
                .filter(|&&c| mostrado(&info, c))
                .filter_map(|&c| Some((b, k(c), ler(&info, c)?)))
                .collect::<Vec<_>>()
        })
        .collect();
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for &b in entidades {
        lidos.esquecer(b, FISICA);
    }
    for (b, kk, v) in lotes {
        lidos.publicar(b, kk, v);
    }
}

/// ⭐ **Aplica aos corpos o que a curva mandou neste quadro**, regista o eco e o autorado.
/// Devolve `true` se algum corpo mudou.
pub fn aplicar(
    sim: &mut SimWorld,
    registry: &ComponentRegistry,
    conducao: &mut ConducaoFisica,
) -> bool {
    let Some(mut escritas) = sim
        .world_mut()
        .get_resource_mut::<CanaisEscritos>()
        .map(|mut w| w.drenar(FISICA))
    else {
        return false;
    };
    // As escolhas antes dos números: a forma decide que medidas existem, o tipo e o sensor
    // decidem que campos a §11 aceita, a fonte da massa decide se a massa é lida.
    escritas.sort_by_key(|&(e, kk, _)| (e, kk.info().is_none_or(|i| !i.discreto), kk));
    let mut mudou = false;
    let mut ecos = Vec::with_capacity(escritas.len());
    for (e, kk, v) in escritas {
        if kk.i != 0 || kk.j != 0 {
            continue;
        }
        if let Some((antes, ficou)) = escrever(sim, registry, e, kk.campo, v) {
            if antes != ficou {
                conducao.conduziu(e, kk, antes);
                mudou = true;
            }
            ecos.push((e, kk, v, ficou));
        }
    }
    let mut lidos = sim
        .world_mut()
        .get_resource_or_insert_with(CanaisLidos::default);
    for (e, kk, v, ficou) in ecos {
        lidos.eco(e, kk, v, ficou);
    }
    mudou
}

/// **A condução dos corpos pela curva** — guarda o valor AUTORADO de cada campo que a curva está a
/// mover, para que o undo e o save o fotografem em vez do instante da reprodução. A semântica é a
/// da `ConducaoVetor`: só se declara o que MUDOU neste quadro; [`Self::settle`] esquece quem não
/// foi declarado.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ConducaoFisica {
    autorado: BTreeMap<(u64, FieldKey), f32>,
    neste_quadro: BTreeSet<(u64, FieldKey)>,
}

impl ConducaoFisica {
    /// A curva moveu `(e, kk)`; `antes` era o valor do documento (o primeiro de uma corrida fica).
    pub fn conduziu(&mut self, e: u64, kk: FieldKey, antes: f32) {
        self.autorado.entry((e, kk)).or_insert(antes);
        self.neste_quadro.insert((e, kk));
    }

    /// Esquece quem não foi conduzido neste quadro (chamado onde o `PreviewDrive::settle` é).
    pub fn settle(&mut self) {
        let neste = std::mem::take(&mut self.neste_quadro);
        self.autorado.retain(|k, _| neste.contains(k));
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.autorado.is_empty()
    }

    /// Repõe o AUTORADO para a fotografia; devolve o vivo para [`Self::repor`]. As escolhas antes
    /// dos números, como no [`aplicar`].
    pub fn substituir(
        &self,
        sim: &mut SimWorld,
        registry: &ComponentRegistry,
    ) -> Vec<(u64, FieldKey, f32)> {
        let mut vivo = Vec::with_capacity(self.autorado.len());
        for (e, kk, v) in ordenar(self.autorado.iter().map(|(&(e, kk), &v)| (e, kk, v))) {
            if let Some((antes, _)) = escrever(sim, registry, e, kk.campo, v) {
                vivo.push((e, kk, antes));
            }
        }
        vivo
    }

    /// Devolve o vivo que [`Self::substituir`] tirou.
    pub fn repor(
        &self,
        sim: &mut SimWorld,
        registry: &ComponentRegistry,
        vivo: Vec<(u64, FieldKey, f32)>,
    ) {
        for (e, kk, v) in ordenar(vivo.into_iter()) {
            let _ = escrever(sim, registry, e, kk.campo, v);
        }
    }
}

fn ordenar(it: impl Iterator<Item = (u64, FieldKey, f32)>) -> Vec<(u64, FieldKey, f32)> {
    let mut v: Vec<_> = it.collect();
    v.sort_by_key(|&(e, kk, _)| (e, kk.info().is_none_or(|i| !i.discreto), kk));
    v
}

#[cfg(test)]
#[path = "canais_corpo_tests.rs"]
mod tests;
