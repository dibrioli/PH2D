//! ⭐⭐⭐⭐ **A PILHA ANDA COM O PLANO** (`docs/3D/30` §11 — a W2) — filho
//! (`#[path]`) de [`super`] (`tinta_da_peca.rs`): lá *quando o plano nasce,
//! quem o segura e quando morre*; aqui *a pilha de camadas que o acompanha em
//! cada um desses momentos*.
//!
//! ⛔⛔ **O invariante: a peça com plano tem pilha, e a pilha descreve o plano**
//! ([`acompanha`]). O plano da peça é a COMPOSIÇÃO da pilha; quem escreve cor
//! escreve na camada activa (a cópia de trabalho do traço) e a composição
//! segue ([`desce_do_traco`]).

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;
use ph2d_sculpt3d::tinta_fina::TintaDoTraco;

use super::{Origem, concorda_com, garante_e_diz, semente};
use crate::pilha_da_peca::PilhaDaPeca;

/// ⭐⭐⭐ **A pilha descreve o plano** — sem plano não há pilha; um plano sem
/// pilha (ou com uma de outra contagem) ganha a de UMA camada dele, e é
/// RECOMPOSTO dela: o plano da peça é sempre a composição da pilha (a cor em
/// RGBA8, a meio degrau do `f32` de que nasceu). Devolve se o plano mudou.
pub(crate) fn acompanha(tinta: Option<&mut Tinta>, pilha: &mut Option<PilhaDaPeca>) -> bool {
    match tinta {
        None => {
            pilha.take();
            false
        }
        Some(t)
            if pilha
                .as_ref()
                .is_some_and(|p| p.amostras() == t.amostras().len()) =>
        {
            false
        }
        Some(t) => {
            let p = PilhaDaPeca::de_tinta(t);
            // UMA camada opaca: o fundo nunca se lê.
            p.pinta_tinta(t, Vec::new);
            *pilha = Some(p);
            true
        }
    }
}

/// ⭐⭐⭐ **A `garante_no_orcamento` com a pilha a seguir o plano**: o que
/// estaciona leva a pilha, o que volta do estacionamento traz a sua, o que
/// nasce da semente nasce com UMA camada.
pub(crate) fn garante_com_pilha(
    mesh: &Mesh,
    tinta: &mut Option<Tinta>,
    parque: &mut Option<Tinta>,
    pilha: &mut Option<PilhaDaPeca>,
    pilha_parque: &mut Option<PilhaDaPeca>,
    nivel: Option<u8>,
    orcamento: u64,
) -> bool {
    let d = garante_e_diz(mesh, tinta, parque, nivel, orcamento);
    if d.mudou {
        let antiga = pilha.take();
        if d.origem == Origem::Parque {
            *pilha = pilha_parque.take();
        }
        if d.estacionou {
            *pilha_parque = antiga;
        }
    }
    let recomposto = acompanha(tinta.as_mut(), pilha);
    acompanha(parque.as_mut(), pilha_parque);
    d.mudou || recomposto
}

/// ⭐⭐⭐ **O pen-down: o traço recebe a cópia de trabalho da camada ACTIVA**
/// — e o plano da peça FICA na peça (é ele que a placa lê durante o traço).
/// `None` se a peça não tem plano, ou se a activa não é uma camada de pintura.
pub(crate) fn empresta_da_peca(
    obj: &mut crate::SceneObject,
    dono: crate::ObjectId,
) -> Option<TintaDoTraco> {
    if acompanha(obj.tinta.as_mut(), &mut obj.pilha) {
        obj.tinta_suja = true;
    }
    let t = obj.tinta.as_ref()?;
    let (_, w) = obj.pilha.as_mut()?.trabalho_da_activa(t)?;
    Some(TintaDoTraco::nova(w, dono.0))
}

/// ⭐⭐⭐⭐ **O quadro de um traço sobre uma camada**: as amostras que ele sujou
/// descem à camada (RGBA8) e só elas se recompõem no plano da peça. Devolve
/// `false` se a peça não está a ser pintada por camada (o caminho de antes).
/// As sujas ficam em `sujas`, para a subida à placa.
pub(crate) fn desce_do_traco(
    obj: &mut crate::SceneObject,
    fina: &mut TintaDoTraco,
    sujas: &mut Vec<u32>,
) -> bool {
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        ..
    } = obj;
    let (Some(pilha), Some(peca)) = (pilha.as_mut(), tinta.as_mut()) else {
        return false;
    };
    let Some(id) = pilha.em_traco() else {
        return false;
    };
    fina.drena_sujas(sujas);
    pilha.recebe_do_traco(id, fina.tinta(), sujas);
    let (mesh, k) = (stack.mesh(), peca.nivel());
    pilha.compoe_amostras(sujas, peca, || semente(mesh, k).amostras().to_vec());
    true
}

/// ⭐⭐⭐ **O pen-up de um traço sobre uma camada**: a última descida, a cor por
/// vértice (a projecção do plano da peça) e a cópia de trabalho deitada fora.
/// Devolve o traço de volta (`Some`) se ele não era por camada.
pub(crate) fn devolve_camada(
    obj: &mut crate::SceneObject,
    mut do_traco: TintaDoTraco,
) -> Option<TintaDoTraco> {
    if obj.pilha.as_ref().and_then(PilhaDaPeca::em_traco).is_none() {
        return Some(do_traco);
    }
    let mut sujas = Vec::new();
    desce_do_traco(obj, &mut do_traco, &mut sujas);
    if let Some(p) = obj.pilha.as_mut() {
        p.fim_do_traco();
    }
    let crate::objects::SceneObject {
        stack,
        tinta,
        tinta_suja,
        ..
    } = obj;
    if let Some(t) = tinta.as_ref()
        && concorda_com(t, stack.mesh())
    {
        let por_vertice = t.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
    }
    *tinta_suja = true;
    None
}

/// O que o balde fez na pilha.
pub(crate) enum Balde {
    /// A peça não tem plano de tinta fina — o balde pinta só a cor por vértice.
    SemCamada,
    /// A pilha existe mas o balde não pôde pintar (a activa não é de pintura,
    /// ou o plano não descreve a malha): a peça fica como está.
    Recusado,
    /// Pintou a camada `id`; `antes` é o plano dela de antes (o desfazer).
    Pintou {
        id: ph2d_tool_painter::LayerId,
        antes: Vec<u8>,
        mudou: bool,
    },
}

/// ⭐⭐⭐ **O balde sobre a camada ACTIVA** — a lei do `preenche_plano` na
/// cópia de trabalho (com a opacidade), a camada inteira de volta em RGBA8, e
/// a peça recomposta. A cor por vértice segue a peça composta.
pub(crate) fn preenche_camada(obj: &mut crate::SceneObject, cor: [f32; 3]) -> Balde {
    if acompanha(obj.tinta.as_mut(), &mut obj.pilha) {
        obj.tinta_suja = true;
    }
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        ..
    } = obj;
    let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
        return Balde::SemCamada;
    };
    let Some((id, mut w)) = pilha.trabalho_da_activa(peca) else {
        return Balde::Recusado;
    };
    pilha.fim_do_traco();
    let Some(antes) = pilha.copia_do_plano(id) else {
        return Balde::Recusado;
    };
    let Ok(mudou) = ph2d_sculpt3d::preenche::preenche_plano(&mut w, stack.mesh(), cor) else {
        return Balde::Recusado;
    };
    if mudou {
        let todas: Vec<u32> = (0..u32::try_from(w.amostras().len()).unwrap_or(u32::MAX)).collect();
        pilha.recebe_do_traco(id, &w, &todas);
        let (mesh, k) = (stack.mesh(), peca.nivel());
        pilha.pinta_tinta(peca, || semente(mesh, k).amostras().to_vec());
        if concorda_com(peca, stack.mesh()) {
            let por_vertice = peca.plano_por_vertice().to_vec();
            stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
        }
    }
    Balde::Pintou { id, antes, mudou }
}

#[cfg(test)]
#[path = "tinta_da_peca_pilha_tests.rs"]
mod tests;
