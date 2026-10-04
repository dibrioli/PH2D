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

use super::{concorda_com, garante_e_diz};
use crate::pilha_da_peca::PilhaDaPeca;

/// De onde veio o plano que a [`super::garante_e_diz`] deixou na peça.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Origem {
    /// Nada mudou.
    Ficou,
    /// Voltou do estacionamento.
    Parque,
    /// Nasceu da semente.
    Semente,
    /// A peça ficou sem plano.
    Nenhum,
}

/// O que a [`super::garante_e_diz`] fez — a pilha de camadas segue-o
/// (`tinta_da_peca_pilha::garante_com_pilha`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Desfecho {
    pub(crate) mudou: bool,
    pub(crate) origem: Origem,
    /// O plano que estava foi para o estacionamento.
    pub(crate) estacionou: bool,
}

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
    let Some(pilha) = obj.pilha.as_mut().filter(|_| obj.tinta.is_some()) else {
        return false;
    };
    let Some(id) = pilha.em_traco() else {
        return false;
    };
    fina.drena_sujas(sujas);
    pilha.recebe_do_traco(id, fina.tinta(), sujas);
    recompoe_sujas(obj, sujas);
    true
}

/// ⭐⭐⭐⭐ **As amostras `sujas` de uma camada mudaram: a peça segue-as** — só
/// elas, na CPU (a composição é ponto a ponto); ou, se a pilha lê VIZINHOS
/// (`docs/3D/30` §14), a peça inteira na PLACA — a vizinha de uma suja também
/// muda — com o plano da CPU atrasado. O relevo das sujas desce sempre.
pub(crate) fn recompoe_sujas(obj: &mut crate::SceneObject, sujas: &[u32]) {
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        compor_na_placa,
        ..
    } = obj;
    let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
        return;
    };
    let (mesh, k) = (stack.mesh(), peca.nivel());
    let vizinhos = pilha.le_a_vizinhanca();
    if vizinhos {
        pilha.garante_vizinhanca(peca, mesh);
    }
    let p = &*pilha;
    p.compoe_amostras(sujas, peca, || p.fundo_semeado(mesh, k));
    if vizinhos {
        pilha.atrasa(peca);
        *compor_na_placa = true;
    }
}

/// ⭐⭐ **A cor por vértice da peça composta** — com uma pilha que lê vizinhos a
/// cor gravada pode ser de um gesto antes (ela acompanhou o prefixo atrasado);
/// o plano aberto é a composição exacta, e as duas têm de ser iguais.
pub(crate) fn cor_por_vertice_da_composta(obj: &mut crate::SceneObject) {
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        ..
    } = obj;
    if let (Some(peca), Some(p)) = (tinta.as_ref(), pilha.as_ref())
        && p.le_a_vizinhanca()
        && !p.atrasada()
        && concorda_com(peca, stack.mesh())
    {
        let por_vertice = peca.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
    }
}

/// ⭐⭐⭐ **O plano da CPU em dia com a pilha** — a composição inteira (a
/// referência) — e, se a pilha lê vizinhos, a cor por vértice com ele: o
/// prefixo dos vértices também estava atrasado, e as duas têm de continuar
/// iguais (o estacionamento compara-as, `desparqueia`).
pub(crate) fn em_dia(obj: &mut crate::SceneObject) {
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        cores_sujas,
        ..
    } = obj;
    let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
        return;
    };
    if !pilha.atrasada() {
        return;
    }
    pilha.em_dia(peca, stack.mesh());
    if pilha.le_a_vizinhanca() && concorda_com(peca, stack.mesh()) {
        let por_vertice = peca.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
        *cores_sujas = true;
    }
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

/// ⭐⭐⭐⭐ **A peça volta a ser a composição da pilha** — inteira (o painel
/// mudou o metadado ou a estrutura, o balde pintou uma camada, um desfazer
/// deles) — e a cor por vértice segue-a.
///
/// ⭐ **Quem compõe a peça inteira é a PLACA** (`docs/3D/30` §13): aqui só o
/// prefixo dos vértices se compõe na CPU (a cor por vértice), o resto do plano
/// da CPU fica ATRASADO e o `sync_mesh` compõe-na no compositor do Painter.
/// Quem precisa da peça inteira na CPU pede-a por [`em_dia`].
pub(crate) fn recompoe(obj: &mut crate::SceneObject) {
    if !recompoe_o_plano(obj) {
        return;
    }
    let crate::objects::SceneObject {
        stack,
        tinta,
        cores_sujas,
        ..
    } = obj;
    if let Some(peca) = tinta.as_ref()
        && concorda_com(peca, stack.mesh())
    {
        let por_vertice = peca.plano_por_vertice().to_vec();
        stack.mesh_mut().colors_mut().copy_from_slice(&por_vertice);
        *cores_sujas = true;
    }
}

/// ⭐⭐ **Só o PLANO volta a ser a composição da pilha** (na placa; na CPU o
/// prefixo dos vértices) — a cor por vértice fica como está: o desfazer do
/// balde repõe-na ele mesmo, ao bit. `false` se a peça não tem pilha.
pub(crate) fn recompoe_o_plano(obj: &mut crate::SceneObject) -> bool {
    let crate::objects::SceneObject {
        stack,
        tinta,
        pilha,
        compor_na_placa,
        ..
    } = obj;
    let (Some(peca), Some(pilha)) = (tinta.as_mut(), pilha.as_mut()) else {
        return false;
    };
    pilha.garante_vizinhanca(peca, stack.mesh());
    pilha.atrasa(peca);
    *compor_na_placa = true;
    true
}

/// ⭐⭐⭐ **A peça INTEIRA para quem a lê na CPU** (assar, exportar, doar ao
/// 2D) — o `plano` se ele está em dia com a pilha (o caso de sempre fora de um
/// gesto do painel), senão uma cópia composta agora, na CPU: a REFERÊNCIA.
pub(crate) fn para_ler<'a>(
    obj: &crate::SceneObject,
    plano: &'a Tinta,
) -> std::borrow::Cow<'a, Tinta> {
    match obj.pilha.as_ref() {
        Some(p) if p.atrasada() && p.amostras() == plano.amostras().len() => {
            let mut fresco = plano.clone();
            let (mesh, k) = (obj.stack.mesh(), plano.nivel());
            // Com vizinhos a pilha precisa da vizinhança DESTA malha — uma
            // cópia ganha-a se a da peça não serve (ler não muda a peça).
            let mut copia = None;
            let p = if p.le_a_vizinhanca() {
                let mut c = p.clone();
                c.garante_vizinhanca(plano, mesh);
                &*copia.insert(c)
            } else {
                p
            };
            p.pinta_tinta(&mut fresco, || p.fundo_semeado(mesh, k));
            std::borrow::Cow::Owned(fresco)
        }
        _ => std::borrow::Cow::Borrowed(plano),
    }
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
        recompoe(obj);
    }
    Balde::Pintou { id, antes, mudou }
}

#[cfg(test)]
#[path = "tinta_da_peca_pilha_tests.rs"]
mod tests;
