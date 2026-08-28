//! **O PATHFINDER** — o vocabulário do ARTISTA sobre o motor de regiões (plano 25 §8, W5).
//!
//! Tínhamos quatro das dez operações do painel Pathfinder (`Union`/`Intersect`/`Subtract`/
//! `Exclude`); esta wave acrescenta **quatro**, e nenhuma delas traz geometria nova: são
//! **composições** do fold que já existe.
//!
//! | op | o que é |
//! |---|---|
//! | **Minus Back** | a frente menos a união de tudo o que está ATRÁS — a fatia de z invertida |
//! | **Trim** | cada forma menos a união do que está ACIMA dela; todas sobrevivem |
//! | **Crop** | cada forma ∩ a do TOPO, e o topo é descartado — ele foi a moldura |
//! | **Merge** | Trim, e depois as de MESMO preenchimento viram uma |
//!
//! # Dois enums, e eles não são uma segunda resposta
//!
//! [`crate::BoolOp`] é o vocabulário do **MOTOR** — as quatro operações de conjunto que o
//! `linesweeper` entende, e é o que o Shape Builder e o Expand consomem. [`PathfinderOp`] é o
//! vocabulário do **ARTISTA** — o comando que ele escolhe no painel. Os quatro primeiros coincidem
//! e a tradução é uma função só ([`PathfinderOp::as_bool`]); os quatro novos **não existem** no
//! motor, porque não são operações de conjunto: são *receitas* sobre elas.
//!
//! A diferença que torna isto necessário em vez de cerimônia: as quatro primeiras **dobram N
//! formas numa região**; as quatro novas são **por-fonte** — Trim devolve tantas formas quantas
//! entraram (menos as que ficaram vazias), e cada uma mantém o SEU estilo. Metê-las no `BoolOp`
//! daria dois significados a `apply_many`.
//!
//! # As duas que ficam FORA, nomeadas
//!
//! **Divide** exige a varredura N-ária única (`Topology::from_paths`) — via [`crate::Arrangement`]
//! são `2^N` regiões, o que no teto de 16 formas vira SEGUNDOS. **Outline** exige saída de caminho
//! **ABERTO**, hoje estruturalmente impossível (a conversão de volta crava `closed: true`).

use ph2d_vec_scene::VecPath;

use crate::{BoolOp, SweepFailed, apply_many, apply_many_checked};

/// O comando do painel Pathfinder. Oito das dez (ver o doc do módulo para as duas que faltam).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PathfinderOp {
    Union,
    Subtract,
    Intersect,
    Exclude,
    /// A da FRENTE menos a união de tudo o que está atrás.
    MinusBack,
    /// Cada forma menos a união do que está ACIMA dela — todas sobrevivem, sem sobreposição.
    Trim,
    /// Cada forma ∩ a do TOPO; o topo é descartado (ele foi a moldura, não conteúdo).
    Crop,
    /// Trim, e depois as de MESMO preenchimento viram uma.
    Merge,
}

impl PathfinderOp {
    /// A operação de conjunto equivalente, quando existe. As quatro receitas devolvem `None` — e
    /// é isso que as separa: elas não são operações de conjunto, são composições delas.
    #[must_use]
    pub fn as_bool(self) -> Option<BoolOp> {
        match self {
            PathfinderOp::Union => Some(BoolOp::Union),
            PathfinderOp::Subtract => Some(BoolOp::Subtract),
            PathfinderOp::Intersect => Some(BoolOp::Intersect),
            PathfinderOp::Exclude => Some(BoolOp::Exclude),
            _ => None,
        }
    }
}

/// **A porta única do Pathfinder.** `paths` em ordem de **z (fundo → topo)**, tudo em MUNDO.
///
/// Devolve as formas-resultado, também em ordem de z. Vazio (`Ok`) quando a operação não produz
/// nada — interseção de disjuntos, menos de duas entradas.
///
/// ⚠️ **`Ok(vec![])` e `Err` são coisas DIFERENTES, e é por isso que esta porta é falível.** As
/// duas pintam o mesmo nada na tela; sem a distinção, o artista não sabe se a operação não tinha
/// resposta ou se o motor desistiu — e o `linesweeper` autodeclara-se *early beta*.
///
/// # Errors
/// [`SweepFailed`] quando o motor recusa a entrada.
pub fn pathfinder(paths: &[&VecPath], op: PathfinderOp) -> Result<Vec<VecPath>, SweepFailed> {
    if paths.len() < 2 {
        return Ok(Vec::new());
    }
    if let Some(b) = op.as_bool() {
        return apply_many_checked(paths, b);
    }
    Ok(match op {
        PathfinderOp::MinusBack => minus_back(paths),
        PathfinderOp::Trim => trim(paths),
        PathfinderOp::Crop => crop(paths),
        PathfinderOp::Merge => merge(paths),
        _ => Vec::new(),
    })
}

/// **Minus Back** — a da FRENTE menos a união de tudo o que está atrás.
///
/// É o `Subtract` com a pilha de z **invertida**, e mais nada. ⚠️ Só a geometria se inverte: o
/// `apply_many` doa o estilo do ÚLTIMO da lista, que na lista invertida é o de TRÁS — e o que
/// sobrevive é a forma da FRENTE. Sem a re-estampagem, o resultado vestiria a roupa de um objeto
/// que foi consumido.
fn minus_back(paths: &[&VecPath]) -> Vec<VecPath> {
    let mut rev: Vec<&VecPath> = paths.to_vec();
    rev.reverse();
    let front = paths.last().expect("len >= 2");
    let mut out = apply_many(&rev, BoolOp::Subtract);
    for p in &mut out {
        wear(p, front);
    }
    out
}

/// **Trim** — cada forma menos a união do que está ACIMA dela.
///
/// Todas sobrevivem (as que ficam vazias somem: elas estavam inteiramente escondidas), cada uma
/// com o SEU estilo, e nenhuma sobreposição fica. A de cima não é tocada — não há nada acima dela.
///
/// ⚠️ **Divergência declarada do Illustrator:** ele REMOVE os traços das formas trimadas. Nós
/// mantemos. O argumento é o de sempre neste repo — apagar em silêncio uma propriedade que o
/// artista autorou é destruir trabalho —, e quem quiser o traço fora tem a swatch None a um
/// clique. Se o smoke pedir a fidelidade, é uma linha.
fn trim(paths: &[&VecPath]) -> Vec<VecPath> {
    let mut out = Vec::new();
    for (i, src) in paths.iter().enumerate() {
        let above = &paths[i + 1..];
        if above.is_empty() {
            out.push((*src).clone());
            continue;
        }
        let mut args: Vec<&VecPath> = vec![src];
        args.extend_from_slice(above);
        for mut piece in apply_many(&args, BoolOp::Subtract) {
            wear(&mut piece, src);
            out.push(piece);
        }
    }
    out
}

/// **Crop** — cada forma ∩ a do TOPO, e o topo é descartado.
///
/// A do topo foi a **moldura**, não conteúdo: mantê-la seria devolver a moldura junto com o
/// recorte, que é o oposto do que a operação diz. Cada peça mantém o estilo da sua fonte.
fn crop(paths: &[&VecPath]) -> Vec<VecPath> {
    let (top, rest) = paths.split_last().expect("len >= 2");
    let mut out = Vec::new();
    for src in rest {
        for mut piece in apply_many(&[src, top], BoolOp::Intersect) {
            wear(&mut piece, src);
            out.push(piece);
        }
    }
    out
}

/// **Merge** — Trim, e depois as de MESMO preenchimento viram uma.
///
/// A classe de equivalência é o `fill` (o `VecPath.fill` já deriva `PartialEq`), que é a regra do
/// Illustrator: *"merges adjacent or overlapping objects filled with the same color"*. Depois do
/// Trim não há sobreposição nenhuma, então o que a união faz é **soldar vizinhas**.
///
/// ⚠️ **Duas da mesma cor que NÃO se tocam continuam DUAS** — o motor agrupa por CONTENÇÃO, e
/// componentes desconexas saem como caminhos separados. Eu esperava o contrário e a medição me
/// corrigiu; é também o que o Illustrator faz (*"merges adjacent or overlapping objects"* — duas
/// ilhas não são nenhum dos dois).
///
/// ⚠️ A ordem de saída segue o **z da primeira forma de cada classe**, e não a ordem em que as
/// classes foram descobertas: o artista ordenou aquilo, e uma operação de limpeza não pode
/// re-embaralhar a pilha.
fn merge(paths: &[&VecPath]) -> Vec<VecPath> {
    let trimmed = trim(paths);
    let mut classes: Vec<Vec<VecPath>> = Vec::new();
    for p in trimmed {
        if let Some(c) = classes.iter_mut().find(|c| c[0].fill == p.fill) {
            c.push(p);
        } else {
            classes.push(vec![p]);
        }
    }
    let mut out = Vec::new();
    for class in classes {
        if class.len() < 2 {
            out.extend(class);
            continue;
        }
        let refs: Vec<&VecPath> = class.iter().collect();
        let united = apply_many(&refs, BoolOp::Union);
        // Uma união que degenera devolve as fontes intactas: perder arte porque o motor não achou
        // resposta é o modo de falha que este repo não aceita.
        if united.is_empty() {
            out.extend(class);
        } else {
            let donor = class.last().expect("len >= 2").clone();
            out.extend(united.into_iter().map(|mut p| {
                wear(&mut p, &donor);
                p
            }));
        }
    }
    out
}

/// Veste `p` com o estilo de `src`. Uma porta só — o `apply_many` doa o estilo do path do TOPO da
/// lista que recebeu, e nas quatro receitas quem manda é outra forma.
fn wear(p: &mut VecPath, src: &VecPath) {
    p.fill.clone_from(&src.fill);
    p.stroke.clone_from(&src.stroke);
    p.effects.clone_from(&src.effects);
}

#[cfg(test)]
#[path = "pathfinder_tests.rs"]
mod tests;
