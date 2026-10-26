//! **A secção DRAWING RULES do Inspector, do lado do documento** (W4.5, `docs/BlenderConstraints/04_W4_cad.md`
//! §3): o instantâneo das regras cujo DONO ([`Lei::dono`]) é o path escolhido, e as edições que a secção pede — a
//! medida nova (a folha re-resolve a partir do desenho de AGORA) e apagar uma regra. O painel pede pelo
//! barramento; a shell entrega o pedido em [`pede`] e chama [`frame`] uma vez por quadro, depois do dreno.
//!
//! Undo: a edição muda o `VecScene` no quadro do clique, e a captura do fim do quadro faz dele UM passo. Auto Key:
//! a medida é um canal publicado (`ph2d_vec_entities::medida::campos`) — o diff do canal grava a chave, como em
//! qualquer campo do vetor.

use std::cell::RefCell;
use std::collections::BTreeSet;

use ph2d_ecs::canal_cad::{MEDIDA, chave};
use ph2d_ecs::{Entity, SimWorld, VecPathRef};
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::cad_edits::{
    CadFieldEdit, EstadoDaRegra, InspectorCadInfo, InspectorRegraRow,
};
use ph2d_vec_constraints::{EstadoRegra, Folha, resolve};
use ph2d_vec_entities::medida::no_canal;
use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, RegraId};
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms, xform_of};

use crate::cad_cores::CadCores;

thread_local! {
    static PEDIDOS: RefCell<Vec<CadFieldEdit>> = const { RefCell::new(Vec::new()) };
}

/// O dreno do barramento entrega aqui o que a secção pediu; [`frame`] aplica-o.
pub fn pede(e: CadFieldEdit) {
    PEDIDOS.with(|p| p.borrow_mut().push(e));
}

/// ⭐ **Um quadro da secção**: aplica os pedidos e publica o instantâneo do path escolhido. `xf` = os afins deste
/// quadro (`ph2d_vec_entities::transform::build`).
pub fn frame(
    sim: &SimWorld,
    scene: &mut VecScene,
    vec: &crate::state::VecState,
    xf: &VecXforms,
    hero: &HeroScreen,
) {
    for e in PEDIDOS.with(|p| std::mem::take(&mut *p.borrow_mut())) {
        // `false` = a regra já saiu (um 2.º clique sobre o instantâneo velho): o próximo quadro já não a mostra.
        let _ = aplica(scene, xf, e);
    }
    let info = hero
        .gizmo
        .selection
        .and_then(|b| info(sim, scene, &vec.cad_cores, xf, b));
    ph2d_panel_inspector::set_current_inspector_cad(info);
}

/// A cor da linha: a da última análise ([`CadCores::ve`]); sem ela (a estrutura acabou de mudar), a marca REF.
fn estado(r: &RegraCad, e: Option<EstadoRegra>) -> EstadoDaRegra {
    match e {
        Some(EstadoRegra::Conflito) => EstadoDaRegra::Conflito,
        Some(EstadoRegra::Redundante) => EstadoDaRegra::Redundante,
        Some(EstadoRegra::Ref | EstadoRegra::SemAlvo) => EstadoDaRegra::Ref,
        Some(EstadoRegra::Ok) => EstadoDaRegra::Ok,
        None if r.referencia => EstadoDaRegra::Ref,
        None => EstadoDaRegra::Ok,
    }
}

/// O instantâneo da secção para a entidade `bits`: as regras cujo dono é o path dela, pela ordem do documento.
/// `None` = não é um path, ou não é dono de regra nenhuma (a secção não aparece). Uma regra ainda sem id (nasceu
/// neste quadro) espera o assentamento.
#[must_use]
pub fn info(
    sim: &SimWorld,
    scene: &VecScene,
    cores: &CadCores,
    xf: &VecXforms,
    bits: u64,
) -> Option<InspectorCadInfo> {
    let path = sim
        .world()
        .get::<VecPathRef>(Entity::try_from_bits(bits)?)?
        .0;
    if !scene.regras().iter().any(|r| r.lei.dono() == Some(path)) {
        return None;
    }
    let analise = cores.ve(scene, xf);
    let rows: Vec<InspectorRegraRow> = scene
        .regras()
        .iter()
        .enumerate()
        .filter(|(_, r)| r.lei.dono() == Some(path) && r.id != RegraId::NOVO)
        .map(|(i, r)| {
            let k = chave(MEDIDA, r.id.0);
            InspectorRegraRow {
                id: r.id.0,
                nome: crate::cad_marks::chave_do_nome(&r.lei),
                estado: estado(r, analise.as_ref().and_then(|a| a.regras.get(i).copied())),
                medida: no_canal(&r.lei),
                graus: matches!(r.lei, Lei::Angulo(..)),
                referencia: r.referencia,
                canal: [k.familia, k.campo, k.i, k.j],
            }
        })
        .collect();
    (!rows.is_empty()).then_some(InspectorCadInfo {
        entity_bits: bits,
        rows,
    })
}

/// ⭐ **Aplica uma edição da secção.** A medida vale o que se escreveu (graus → radianos num ângulo; uma distância e
/// um raio não são negativos) e a folha resolve-se a partir do desenho de AGORA; uma REF não se escreve. Apagar tira
/// a regra. Devolve se o documento mudou.
pub fn aplica(scene: &mut VecScene, xf: &VecXforms, e: CadFieldEdit) -> bool {
    match e {
        CadFieldEdit::Apagar(id) => {
            let antes = scene.regras().len();
            let alvo = RegraId(id);
            scene
                .regras_mut()
                .retain(|r| r.id != alvo || alvo == RegraId::NOVO);
            scene.regras().len() != antes
        }
        CadFieldEdit::Medida(id, v) => {
            let Some(r) = scene.regra_mut(RegraId(id)).filter(|r| !r.referencia) else {
                return false;
            };
            let angulo = matches!(r.lei, Lei::Angulo(..));
            let Some(d) = r.lei.medida_mut() else {
                return false;
            };
            let v = f64::from(v);
            *d = if angulo { v.to_radians() } else { v.max(0.0) };
            resolve_a_folha(scene, xf);
            true
        }
    }
}

/// Resolve todas as regras sobre todo path citado, partindo do desenho, e escreve a solução de volta.
fn resolve_a_folha(scene: &mut VecScene, xf: &VecXforms) {
    let ids: BTreeSet<VecPathId> = scene
        .regras()
        .iter()
        .flat_map(|r| r.lei.pontos())
        .map(PontoCad::path)
        .collect();
    let res = {
        let folha = Folha::new(
            ids.iter()
                .filter_map(|&id| scene.path(id).map(|p| (p, xform_of(xf, id)))),
        );
        resolve(&folha, scene.regras(), None, None)
    };
    for &id in &ids {
        if let Some(p) = scene.path_mut(id) {
            res.solucao.aplica(p, xform_of(xf, id));
        }
    }
}

#[cfg(test)]
#[path = "cad_inspector_tests.rs"]
mod tests;
