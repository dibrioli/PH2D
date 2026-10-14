//! **As intenções do grafo ↔ a gravação** — e o CENSO delas (gate 4 do plano): cada variante de
//! [`GraphIntent`] tem UMA decisão em [`decide`]. O `match` não tem curinga, então uma variante nova
//! sem decisão não compila; o gate `a_decisao_das_intencoes_nao_tem_curinga` guarda que continue assim.

use super::passo::{Alvo, Escolha, Intencao, Nomeado};
use super::tocar::{ErroDaMacro, Nomes};
use crate::motion_state::FormaEscolhida;
use ph2d_panel_motion_graph::{GraphIntent, RenameTarget, SUBGRAPH_VIEW_TAG};

#[derive(Clone, Debug, PartialEq)]
pub enum Decisao {
    Grava(Intencao),
    /// Resolvida no dreno de PARAMS, que a grava já resolvida — gravá-la aqui aplicá-la-ia duas vezes.
    Encaminha,
    /// Vista, transporte ou leitura: não muda o documento (o relógio viaja no quadro).
    Vista,
    /// Usada a gravar, a macro não a reproduz: o tocador pára ali, em voz alta.
    Recusa(&'static str),
}

/// Um id de VISTA do painel (um nó, ou o cartão de um grupo com a etiqueta).
#[must_use]
pub fn alvo_de_vista(view: u32, base: u32) -> Alvo {
    if view & SUBGRAPH_VIEW_TAG == 0 {
        Alvo::de_no(view, base)
    } else {
        Alvo::Grupo(view & !SUBGRAPH_VIEW_TAG)
    }
}

/// O id de vista de um alvo no palco.
#[must_use]
pub fn vista_de_alvo(a: Alvo, base: u32) -> Option<u32> {
    match a {
        Alvo::Grupo(sid) => Some(sid | SUBGRAPH_VIEW_TAG),
        _ => a.no(base),
    }
}

#[must_use]
pub fn escolha_de(f: &FormaEscolhida) -> Escolha {
    match f {
        FormaEscolhida::Nada => Escolha::Nada,
        FormaEscolhida::NaoEDesenho => Escolha::NaoEDesenho,
        FormaEscolhida::SemNome => Escolha::SemNome,
        FormaEscolhida::SemArco => Escolha::SemArco,
        FormaEscolhida::Nome(n) => Escolha::Nome(n.clone()),
    }
}

#[must_use]
pub fn forma_de(e: &Escolha) -> FormaEscolhida {
    match e {
        Escolha::Nada => FormaEscolhida::Nada,
        Escolha::NaoEDesenho => FormaEscolhida::NaoEDesenho,
        Escolha::SemNome => FormaEscolhida::SemNome,
        Escolha::SemArco => FormaEscolhida::SemArco,
        Escolha::Nome(n) => FormaEscolhida::Nome(n.clone()),
    }
}

/// **A decisão de cada intenção**, já resolvida contra a forma que o botão «Use selected path» vê e a
/// gravação (`base`, a cópia gravada).
#[must_use]
pub fn decide(i: &GraphIntent, base: u32, forma: &FormaEscolhida, clip_gravado: bool) -> Decisao {
    use GraphIntent as G;
    use Intencao as I;
    let v = |id: u32| alvo_de_vista(id, base);
    let vs = |ids: &[u32]| ids.iter().map(|&id| v(id)).collect::<Vec<_>>();
    Decisao::Grava(match i {
        G::BeginDrag => I::BeginDrag,
        G::EndDrag => I::EndDrag,
        G::MoveNodes { nodes, dx, dy } => I::MoveNodes {
            nodes: vs(nodes),
            dx: *dx,
            dy: *dy,
        },
        G::DriveParam {
            from_node,
            from_port,
            to_node,
            param,
        } => I::DriveParam {
            from: v(*from_node),
            from_port: *from_port,
            to: v(*to_node),
            param: (*param).to_owned(),
        },
        G::SetParam { .. }
        | G::SetTextParam { .. }
        | G::PickFile { .. }
        | G::StepChoice { .. }
        | G::PickChoice { .. } => return Decisao::Encaminha,
        G::PickSelection { node, param } => I::PickSelection {
            node: v(*node),
            param: (*param).to_owned(),
            escolha: escolha_de(forma),
        },
        G::ToggleParamSection { .. }
        | G::OpenLibrary { .. }
        | G::SetSplit { .. }
        | G::SetSplitVertical { .. }
        | G::TogglePlay
        | G::SetProbe { .. }
        // A recusa de uma largada no corpo de um nó só diz PORQUÊ ao artista (um aviso): o documento
        // não muda, e o palco não tem a quem avisar.
        | G::DropRefused { .. } => return Decisao::Vista,
        G::Connect {
            from_node,
            from_port,
            to_node,
            to_port,
        } => I::Connect {
            from: v(*from_node),
            from_port: *from_port,
            to: v(*to_node),
            to_port: *to_port,
        },
        G::Disconnect { to_node, to_port } => I::Disconnect {
            to: v(*to_node),
            to_port: *to_port,
        },
        G::AddNode { type_name, x, y } => I::AddNode {
            tipo: (*type_name).to_owned(),
            x: *x,
            y: *y,
        },
        G::DeleteSelection { nodes } => I::DeleteSelection(vs(nodes)),
        // A ajuda dos nós decide se o lote se cura (`heal_setup`): é entrada do documento.
        G::SetNodeHelp(on) => I::SetNodeHelp(*on),
        G::AddBackdrop { x, y, w, h } => I::AddBackdrop {
            x: *x,
            y: *y,
            w: *w,
            h: *h,
        },
        G::MoveBackdrop { id, dx, dy } => I::MoveBackdrop {
            id: *id,
            dx: *dx,
            dy: *dy,
        },
        G::ResizeBackdrop { id, left, dx, dy } => I::ResizeBackdrop {
            id: *id,
            left: *left,
            dx: *dx,
            dy: *dy,
        },
        G::DeleteBackdrop { id } => I::DeleteBackdrop(*id),
        G::Rename { target, name } => I::Rename {
            alvo: match target {
                RenameTarget::Node(id) => Nomeado::No(v(*id)),
                RenameTarget::Subgraph(id) => Nomeado::Grupo(*id),
                RenameTarget::Backdrop(id) => Nomeado::Fundo(*id),
            },
            nome: name.clone(),
        },
        G::SetBackdropColor { id, color } => I::SetBackdropColor {
            id: *id,
            cor: *color,
        },
        G::DuplicateSelection { nodes } => I::DuplicateSelection(vs(nodes)),
        G::SmartConnect {
            from_node,
            from_port,
            to_type,
            x,
            y,
        } => I::SmartConnect {
            from: v(*from_node),
            from_port: *from_port,
            tipo: (*to_type).to_owned(),
            x: *x,
            y: *y,
        },
        G::SwapInChain {
            a,
            b,
            back_dx,
            back_dy,
        } => I::SwapInChain {
            a: v(*a),
            b: v(*b),
            back_dx: *back_dx,
            back_dy: *back_dy,
        },
        G::SpliceExistingIntoWire {
            node,
            to_node,
            to_port,
        } => I::SpliceExistingIntoWire {
            node: v(*node),
            to: v(*to_node),
            to_port: *to_port,
        },
        G::SmartConnectBack {
            to_node,
            to_port,
            to_type,
            x,
            y,
        } => I::SmartConnectBack {
            to: v(*to_node),
            to_port: *to_port,
            tipo: (*to_type).to_owned(),
            x: *x,
            y: *y,
        },
        G::CutWires { targets } => I::CutWires(targets.iter().map(|&(n, p)| (v(n), p)).collect()),
        G::MoveWireEnd {
            from_node,
            from_port,
            old_to_node,
            old_to_port,
            new_to,
        } => I::MoveWireEnd {
            from: v(*from_node),
            from_port: *from_port,
            old_to: v(*old_to_node),
            old_to_port: *old_to_port,
            new_to: new_to.map(|(n, p)| (v(n), p)),
        },
        G::GroupSelection { nodes } => I::GroupSelection(vs(nodes)),
        G::Ungroup { id } => I::Ungroup(*id),
        // O nível decide onde nasce o que se cria a seguir (`subgraph::adopt_new`).
        G::EnterSubgraph { id } => I::EnterSubgraph(*id),
        G::GoToLevel { level } => I::GoToLevel(*level),
        G::SpliceReroute {
            to_node,
            to_port,
            x,
            y,
        } => I::SpliceReroute {
            to: v(*to_node),
            to_port: *to_port,
            x: *x,
            y: *y,
        },
        G::SpliceNode {
            to_node,
            to_port,
            to_type,
            x,
            y,
        } => I::SpliceNode {
            to: v(*to_node),
            to_port: *to_port,
            tipo: (*to_type).to_owned(),
            x: *x,
            y: *y,
        },
        G::ArrangeLayout => I::ArrangeLayout,
        G::SetBypass { nodes, on } => I::SetBypass {
            nodes: vs(nodes),
            on: *on,
        },
        G::CopySelection { nodes } => I::CopySelection(vs(nodes)),
        G::Paste if clip_gravado => I::Paste,
        G::Paste => return Decisao::Recusa(super::COLAR_DE_ANTES),
        G::FixInert { node } => I::FixInert(v(*node)),
    })
}

/// **A intenção gravada de volta à do painel**, no palco: os alvos pelos ids do palco, os nomes pelo
/// `&'static` do registo.
///
/// # Errors
/// [`ErroDaMacro`] — um alvo que não cabe, um nome que este build não tem, um tipo desconhecido.
pub fn para_grafo(i: &Intencao, base: u32, nomes: &Nomes) -> Result<GraphIntent, ErroDaMacro> {
    use GraphIntent as G;
    use Intencao as I;
    let v = |a: Alvo| vista_de_alvo(a, base).ok_or(ErroDaMacro::Alvo);
    let vs = |v_: &[Alvo]| v_.iter().map(|a| v(*a)).collect::<Result<Vec<_>, _>>();
    let n = |s: &str| nomes.get(s);
    Ok(match i {
        I::BeginDrag => G::BeginDrag,
        I::EndDrag => G::EndDrag,
        I::MoveNodes { nodes, dx, dy } => G::MoveNodes {
            nodes: vs(nodes)?,
            dx: *dx,
            dy: *dy,
        },
        I::DriveParam {
            from,
            from_port,
            to,
            param,
        } => G::DriveParam {
            from_node: v(*from)?,
            from_port: *from_port,
            to_node: v(*to)?,
            param: n(param)?,
        },
        I::PickSelection { node, param, .. } => G::PickSelection {
            node: v(*node)?,
            param: n(param)?,
        },
        I::Connect {
            from,
            from_port,
            to,
            to_port,
        } => G::Connect {
            from_node: v(*from)?,
            from_port: *from_port,
            to_node: v(*to)?,
            to_port: *to_port,
        },
        I::Disconnect { to, to_port } => G::Disconnect {
            to_node: v(*to)?,
            to_port: *to_port,
        },
        I::AddNode { tipo, x, y } => G::AddNode {
            type_name: n(tipo)?,
            x: *x,
            y: *y,
        },
        I::DeleteSelection(a) => G::DeleteSelection { nodes: vs(a)? },
        I::AddBackdrop { x, y, w, h } => G::AddBackdrop {
            x: *x,
            y: *y,
            w: *w,
            h: *h,
        },
        I::MoveBackdrop { id, dx, dy } => G::MoveBackdrop {
            id: *id,
            dx: *dx,
            dy: *dy,
        },
        I::ResizeBackdrop { id, left, dx, dy } => G::ResizeBackdrop {
            id: *id,
            left: *left,
            dx: *dx,
            dy: *dy,
        },
        I::DeleteBackdrop(id) => G::DeleteBackdrop { id: *id },
        I::Rename { alvo, nome } => G::Rename {
            target: match alvo {
                Nomeado::No(a) => RenameTarget::Node(v(*a)?),
                Nomeado::Grupo(id) => RenameTarget::Subgraph(*id),
                Nomeado::Fundo(id) => RenameTarget::Backdrop(*id),
            },
            name: nome.clone(),
        },
        I::SetBackdropColor { id, cor } => G::SetBackdropColor {
            id: *id,
            color: *cor,
        },
        I::DuplicateSelection(a) => G::DuplicateSelection { nodes: vs(a)? },
        I::SmartConnect {
            from,
            from_port,
            tipo,
            x,
            y,
        } => G::SmartConnect {
            from_node: v(*from)?,
            from_port: *from_port,
            to_type: n(tipo)?,
            x: *x,
            y: *y,
        },
        I::SwapInChain {
            a,
            b,
            back_dx,
            back_dy,
        } => G::SwapInChain {
            a: v(*a)?,
            b: v(*b)?,
            back_dx: *back_dx,
            back_dy: *back_dy,
        },
        I::SpliceExistingIntoWire { node, to, to_port } => G::SpliceExistingIntoWire {
            node: v(*node)?,
            to_node: v(*to)?,
            to_port: *to_port,
        },
        I::SmartConnectBack {
            to,
            to_port,
            tipo,
            x,
            y,
        } => G::SmartConnectBack {
            to_node: v(*to)?,
            to_port: *to_port,
            to_type: n(tipo)?,
            x: *x,
            y: *y,
        },
        I::CutWires(t) => G::CutWires {
            targets: t
                .iter()
                .map(|(a, p)| v(*a).map(|n| (n, *p)))
                .collect::<Result<_, _>>()?,
        },
        I::MoveWireEnd {
            from,
            from_port,
            old_to,
            old_to_port,
            new_to,
        } => G::MoveWireEnd {
            from_node: v(*from)?,
            from_port: *from_port,
            old_to_node: v(*old_to)?,
            old_to_port: *old_to_port,
            new_to: match new_to {
                Some((a, p)) => Some((v(*a)?, *p)),
                None => None,
            },
        },
        I::GroupSelection(a) => G::GroupSelection { nodes: vs(a)? },
        I::Ungroup(id) => G::Ungroup { id: *id },
        I::EnterSubgraph(id) => G::EnterSubgraph { id: *id },
        I::GoToLevel(level) => G::GoToLevel { level: *level },
        I::SpliceReroute { to, to_port, x, y } => G::SpliceReroute {
            to_node: v(*to)?,
            to_port: *to_port,
            x: *x,
            y: *y,
        },
        I::SpliceNode {
            to,
            to_port,
            tipo,
            x,
            y,
        } => G::SpliceNode {
            to_node: v(*to)?,
            to_port: *to_port,
            to_type: n(tipo)?,
            x: *x,
            y: *y,
        },
        I::ArrangeLayout => G::ArrangeLayout,
        I::SetBypass { nodes, on } => G::SetBypass {
            nodes: vs(nodes)?,
            on: *on,
        },
        I::CopySelection(a) => G::CopySelection { nodes: vs(a)? },
        I::Paste => G::Paste,
        I::FixInert(a) => G::FixInert { node: v(*a)? },
        I::SetNodeHelp(on) => G::SetNodeHelp(*on),
        I::Desconhecida(k) => return Err(ErroDaMacro::IntencaoDesconhecida(*k)),
    })
}
