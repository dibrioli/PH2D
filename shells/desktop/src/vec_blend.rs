//! **Blend** — os passos intermediários entre duas formas (o Blend do Illustrator).
//!
//! O motor é a [`ph2d_vec_blend`]; aqui mora só a **política**: quem são as duas formas, o que
//! acontece com elas, e o que a seleção vira.
//!
//! # A sessão existe por causa do ESCAPE, e o escape existe porque o automático erra
//!
//! A correspondência entre duas formas — *que ponto de A vira que ponto de B?* — é o problema
//! que **ninguém** resolveu (`docs/Vector Module/20_*` §1.3: o GSAP tem `shapeIndex` manual **e**
//! uma ferramenta de debug que admite o erro; o Corel pede pro usuário clicar um nó em cada
//! forma). Quando o automático erra, a forma **gira** ou **vira do avesso** no meio do caminho.
//!
//! Um blend destrutivo de tiro único deixaria o artista assim: blend, ver o erro, Ctrl+Z, mexer
//! num número, blend de novo — às cegas. Então o blend **guarda a sessão** (as duas fontes, as
//! opções, e o que ele produziu): **Rotate Match** e **Reverse Match** apagam os passos anteriores
//! e re-rodam na hora. O erro se conserta **à vista**, e é isso que separa o escape de um enfeite.
//!
//! As **fontes sobrevivem** (ao contrário da booleana, que consome os operandos) — é o Blend do
//! Illustrator: os passos nascem ENTRE elas, no z delas.

use ph2d_vec_blend::BlendOpts;
use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

/// O blend vivo: o que re-rodar quando o artista mexe no escape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlendSession {
    /// As duas fontes, em z (fundo → topo). Elas **não** são consumidas.
    pub(crate) a: VecPathId,
    pub(crate) b: VecPathId,
    pub(crate) steps: u32,
    pub(crate) opts: BlendOpts,
    /// Cada passo nasce **acima** do anterior (`true`) ou **abaixo** (`false`).
    pub(crate) stack_up: bool,
    /// Os passos que a última rodada produziu — é o que a próxima apaga.
    pub(crate) produced: Vec<VecPathId>,
}

impl BlendSession {
    /// A sequência inteira em z (**fundo → topo**): as duas fontes e os passos entre elas.
    ///
    /// **As fontes entram na conta**, e não é detalhe: um blend cuja 1ª intermediária fica DEBAIXO
    /// da forma que a originou não lê como uma transição — lê como bagunça (Enio, smoke). A ordem
    /// de z de uma sequência é parte do resultado, não um efeito colateral de quem nasceu primeiro.
    pub(crate) fn stack(&self) -> Vec<VecPathId> {
        let mut z = Vec::with_capacity(self.produced.len() + 2);
        z.push(self.a);
        z.extend(self.produced.iter().copied());
        z.push(self.b);
        if !self.stack_up {
            z.reverse();
        }
        z
    }
}

/// O que o painel pede.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BlendAction {
    /// **Blend**: (re)cria os passos entre as duas formas selecionadas.
    Run,
    /// **Rotate Match**: roda a correspondência em uma âncora e re-roda.
    Rotate,
    /// **Reverse Match**: inverte o sentido de percurso de B e re-roda.
    Reverse,
    /// **Stack Up**: o checkbox — cada passo acima do anterior, ou abaixo. Re-empilha na hora.
    StackUp(bool),
}

/// O botão do painel → a ação (`None` para qualquer outro id). Pura, gateada.
pub(crate) fn action_for_id(id: ph2d_editor::NodeId, stack_up: bool) -> Option<BlendAction> {
    if id == ph2d_editor::ids::VECTOR_BLEND_STACK_UP {
        Some(BlendAction::StackUp(!stack_up)) // o clique ALTERNA
    } else if id == ph2d_editor::ids::VECTOR_BLEND_RUN {
        Some(BlendAction::Run)
    } else if id == ph2d_editor::ids::VECTOR_BLEND_ROTATE {
        Some(BlendAction::Rotate)
    } else if id == ph2d_editor::ids::VECTOR_BLEND_REVERSE {
        Some(BlendAction::Reverse)
    } else {
        None
    }
}

/// Aplica a ação. `steps` vem do slider do painel.
///
/// - **Run** abre uma sessão nova com as duas formas fechadas selecionadas (exige exatamente
///   duas: três formas não têm um "entre" definido, e adivinhar seria pior que recusar).
/// - **Rotate/Reverse** mexem na sessão ABERTA. Sem sessão, não fazem nada — e dizem por quê.
///
/// Um passo de undo por ação (o re-rodar é uma edição como outra qualquer).
#[allow(clippy::too_many_arguments)] // o shell destruturado passa cada ref separada
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &VecXforms,
    session: &mut Option<BlendSession>,
    action: BlendAction,
    steps: u32,
    stack_up: bool,
) {
    let mut next = match (action, session.take()) {
        (BlendAction::Run, prev) => {
            let Some((a, b)) = two_selected_closed(scene, pen) else {
                eprintln!("[ph2d-vec] blend: selecione exatamente DUAS regioes fechadas");
                return;
            };
            // Re-rodar sobre as MESMAS duas formas mantém o escape que o artista já ajustou —
            // clicar Blend de novo não deveria jogar fora o trabalho dele.
            let opts = prev
                .filter(|s| s.a == a && s.b == b)
                .map_or(BlendOpts::default(), |s| s.opts);
            BlendSession {
                a,
                b,
                steps,
                opts,
                stack_up,
                produced: Vec::new(),
            }
        }
        (_, None) => {
            eprintln!("[ph2d-vec] blend: nao ha blend aberto — clique Blend primeiro");
            return;
        }
        (BlendAction::Rotate, Some(mut s)) => {
            s.opts.offset = s.opts.offset.wrapping_add(1);
            s
        }
        (BlendAction::Reverse, Some(mut s)) => {
            s.opts.reverse = !s.opts.reverse;
            s
        }
        (BlendAction::StackUp(up), Some(mut s)) => {
            s.stack_up = up;
            s
        }
    };
    next.steps = steps;
    if !matches!(action, BlendAction::StackUp(_)) {
        next.stack_up = stack_up;
    }

    let (Some(a), Some(b)) = (world(scene, xforms, next.a), world(scene, xforms, next.b)) else {
        eprintln!("[ph2d-vec] blend: uma das formas sumiu");
        return;
    };
    let made = ph2d_vec_blend::steps(&a, &b, next.steps as usize, next.opts);
    if made.is_empty() {
        eprintln!("[ph2d-vec] blend: resultado vazio (forma degenerada?)");
        return;
    }

    let pre = scene.clone(); // UM passo de undo por ação — inclusive o re-rodar
    for id in &next.produced {
        scene.remove_path(*id);
    }
    // Os passos entram logo ACIMA da forma de trás: eles ficam entre as duas, que é onde o olho
    // os procura. (A busca é feita DEPOIS da remoção — o índice de antes já não vale.)
    let at = scene
        .paths()
        .iter()
        .position(|p| p.id == next.a)
        .map_or(0, |z| z + 1);
    next.produced = made
        .into_iter()
        .enumerate()
        .map(|(k, p)| scene.insert_path(at + k, p))
        .collect();
    history.push_undo(pre);
    if std::env::var_os("PH2D_BLEND_LOG").is_some() {
        for id in &next.produced {
            let Some(pth) = scene.paths().iter().find(|p| p.id == *id) else {
                continue;
            };
            let n = pth.verts.len();
            let curved = pth
                .verts
                .iter()
                .filter(|v| {
                    let d = |h: [f64; 2], a: [f64; 2]| {
                        (h[0] - a[0]).abs() > 1e-9 || (h[1] - a[1]).abs() > 1e-9
                    };
                    d(v.in_handle, v.anchor) || d(v.out_handle, v.anchor)
                })
                .count();
            eprintln!(
                "[blend] passo {id}: {n} vertices · {curved} com alca de CURVA (0 = poligono de quinas afiadas)"
            );
        }
    }
    pen.select_many(&next.produced);
    eprintln!(
        "[ph2d-vec] blend: {} passo(s) · offset={} reverse={}",
        next.produced.len(),
        next.opts.offset,
        next.opts.reverse
    );
    *session = Some(next);
}

/// As DUAS formas fechadas selecionadas, em z (fundo, topo). `None` se não forem exatamente 2.
fn two_selected_closed(
    scene: &VecScene,
    pen: &ph2d_vec_edit::PenTool,
) -> Option<(VecPathId, VecPathId)> {
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id && p.closed))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    match zs[..] {
        [lo, hi] => Some((scene.paths()[lo].id, scene.paths()[hi].id)),
        _ => None,
    }
}

/// A forma assada no MUNDO (ADR-0111: as duas podem ter poses diferentes, e o resultado vive num
/// frame só — como na booleana).
fn world(scene: &VecScene, xforms: &VecXforms, id: VecPathId) -> Option<ph2d_vec_scene::VecPath> {
    let mut p = scene.paths().iter().find(|p| p.id == id)?.clone();
    ph2d_vec_scene::bake_xform(&mut p, &ph2d_vec_scene::xform_of(xforms, id));
    Some(p)
}

#[cfg(test)]
#[path = "vec_blend_tests.rs"]
mod tests;
