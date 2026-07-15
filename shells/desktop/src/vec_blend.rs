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
//! forma). A correspondência é 100% AUTOMÁTICA — o motor escolhe o sentido de menor custo e o
//! melhor casamento de quinas sozinho.
//!
//! Houve **dois** botões de escape manual, **Rotate Match** e **Reverse Match**, e os dois foram
//! removidos (2026-07-14) por serem bugs de design: o Reverse invertia o winding e colapsava a
//! forma; o Rotate rodava a correspondência às cegas e produzia torção. **O ajuste, no modelo vivo,
//! é editar as formas-fonte** — girar/mover/escalar uma adapta os intermediários —, não um botão.
//!
//! > ⚠️ Este é o modelo DESTRUTIVO (a `BlendSession` produz paths reais). O **painel já NÃO o
//! > usa** — o botão "Blend" cria o **Blend Object VIVO** (ADR-0122, `crate::blend_live`): um objeto
//! > único, não-destrutivo, com as fontes sempre editáveis. Este `apply` sobrevive só para os
//! > smokes de correspondência (`PH2D_BUILD_SMOKE=7/8/9`, que mostram star→circle etc.); a remoção
//! > completa (com os smokes repontados ao vivo) é uma limpeza posterior.
//!
//! As **fontes sobrevivem** (ao contrário da booleana, que consome os operandos) — é o Blend do
//! Illustrator: os passos nascem ENTRE elas, no z delas.

use ph2d_vec_scene::{VecPathId, VecScene, VecXforms};

/// O blend: as duas fontes + o que ele produziu (re-roda quando o artista mexe no Steps/Stack).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlendSession {
    /// As duas fontes, em z (fundo → topo). Elas **não** são consumidas.
    pub(crate) a: VecPathId,
    pub(crate) b: VecPathId,
    pub(crate) steps: u32,
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

/// (Re)cria os passos do blend DESTRUTIVO entre as duas formas fechadas selecionadas — o modelo
/// legado, hoje só chamado pelos smokes de correspondência (`PH2D_BUILD_SMOKE=7/8/9`). O painel
/// passou a usar o Blend Object VIVO ([`crate::blend_live`]).
///
/// Exige exatamente DUAS fechadas (três não têm um "entre" definido, e adivinhar seria pior que
/// recusar). Um passo de undo por chamada. `steps`/`stack_up` vêm do chamador.
#[allow(clippy::too_many_arguments)] // o shell destruturado passa cada ref separada
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &mut ph2d_vec_edit::PenTool,
    xforms: &VecXforms,
    session: &mut Option<BlendSession>,
    steps: u32,
    stack_up: bool,
) {
    // **De quem são as fontes?** Depois de um Blend a seleção são os PASSOS (o `select_many` no
    // fim), então enquanto ela for a que a sessão produziu o artista está iterando no MESMO blend
    // (as fontes são as dela); senão, são as duas formas fechadas selecionadas.
    let prev = session.take();
    let iterating = prev.as_ref().is_some_and(|s| selection_is_produced(pen, s));
    let pair = if iterating {
        prev.as_ref().map(|s| (s.a, s.b))
    } else {
        two_selected_closed(scene, pen)
    };
    let Some((a, b)) = pair else {
        eprintln!("[ph2d-vec] blend: selecione exatamente DUAS regioes fechadas");
        return;
    };
    // Re-rodar sobre as MESMAS duas formas carrega o `produced` (o laço de remoção tira os passos
    // velhos da cena); sobre outras, começa do zero (senão re-rodar empilharia um jogo novo).
    let prev = prev.filter(|s| s.a == a && s.b == b);
    let mut next = BlendSession {
        a,
        b,
        steps,
        stack_up,
        produced: prev.map(|s| s.produced).unwrap_or_default(),
    };

    let (Some(a), Some(b)) = (world(scene, xforms, next.a), world(scene, xforms, next.b)) else {
        eprintln!("[ph2d-vec] blend: uma das formas sumiu");
        return;
    };
    let made = ph2d_vec_blend::steps(&a, &b, next.steps as usize);
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
    eprintln!("[ph2d-vec] blend: {} passo(s)", next.produced.len());
    *session = Some(next);
}

/// A seleção é (parte d)o que ESTA sessão produziu?
///
/// É a pergunta *"o artista está iterando neste blend, ou escolhendo outras formas?"*. Depois de um
/// Blend a seleção são os **passos** (o `select_many` no fim do `apply`), então um `Run` que só
/// soubesse ler "duas formas fechadas" recusaria o 2º clique — e, com `Steps = 2`, blendaria os
/// próprios passos.
///
/// Seleção **vazia** não é iteração: sem nada selecionado, o artista não está apontando para blend
/// nenhum.
fn selection_is_produced(pen: &ph2d_vec_edit::PenTool, s: &BlendSession) -> bool {
    let sel = pen.selected_paths();
    !sel.is_empty() && sel.iter().all(|id| s.produced.contains(id))
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
