//! **Expand** na shell — os cliques de *Offset Path* e *Outline Stroke*.
//!
//! O motor é `ph2d_vec_boolean::expand`; aqui mora só o que é de DOCUMENTO: quais paths, em
//! que z, com que pose, e um passo de undo para o gesto inteiro.
//!
//! **Por-path, não N-ário** (≠ booleana): offsetar três formas selecionadas offseta as três,
//! cada uma na sua. Uma operação N-ária aqui seria "funda tudo e offsete o resultado", que
//! é outra coisa e que o artista consegue pedindo Union antes.

use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::{
    LineJoin, OffsetSide, VecPathId, VecScene, VecXforms, WidthProfile, bake_xform, xform_of,
};

/// Qual comando o clique pediu.
// Sem `Eq`: o perfil carrega `f64`. `PartialEq` basta — ninguém usa isto como chave.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum Expand {
    /// A borda anda `d` (negativo encolhe), com quinas em `join`, no(s) contorno(s) `side`.
    Offset { join: LineJoin, side: OffsetSide },
    /// O traço vira forma preenchida.
    OutlineStroke,
    /// O traço vira forma preenchida com a largura VARIANDO pelo `profile` — o Power Stroke.
    PowerStroke { profile: WidthProfile },
}

/// O id do painel → o comando, ou `None` se o id não é nosso. Porta única: o dreno pergunta
/// para saber se ATENDE, e a mesma resposta diz o QUE fazer.
///
/// A junção vem do PAINEL (`ph2d_panel_vector::expand_join`) e não de uma cópia daqui: uma
/// segunda tabela divergiria no dia em que aparecesse um 4º estilo de quina.
pub(crate) fn expand_for_id(id: ph2d_editor::NodeId) -> Option<Expand> {
    if id == ph2d_editor::ids::VECTOR_EXPAND_OFFSET_PATH {
        Some(Expand::Offset {
            join: offset_join(),
            side: offset_side(),
        })
    } else if id == ph2d_editor::ids::VECTOR_EXPAND_OUTLINE_STROKE {
        Some(Expand::OutlineStroke)
    } else if id == ph2d_editor::ids::VECTOR_EXPAND_POWER_STROKE {
        // O perfil vem dos sliders, e quem os lê é a `render_loop` (é ela que tem o store).
        // Aqui só se diz QUAL comando é; o `profile` é preenchido lá, como o `d` do offset.
        Some(Expand::PowerStroke {
            profile: WidthProfile::UNIFORM,
        })
    } else {
        None
    }
}

/// O código do painel → o estilo de quina. **Porta única** — o painel, o componente
/// [`ph2d_ecs::VecOffset`] e o motor falam o mesmo `u8`, e a tradução mora aqui só uma vez:
/// uma 2ª tabela divergiria no dia em que aparecesse um 4º estilo de quina.
pub(crate) fn join_of_code(code: u8) -> LineJoin {
    match code {
        1 => LineJoin::Round,
        2 => LineJoin::Bevel,
        _ => LineJoin::Miter,
    }
}

/// O código do painel → o contorno que anda. Porta única, como a junção.
pub(crate) fn side_of_code(code: u8) -> OffsetSide {
    match code {
        0 => OffsetSide::Outer,
        1 => OffsetSide::Inner,
        _ => OffsetSide::Both,
    }
}

/// A junção do Offset, lida do PAINEL (`ph2d_panel_vector::expand_join`).
pub(crate) fn offset_join() -> LineJoin {
    join_of_code(ph2d_panel_vector::expand_join())
}

/// Qual contorno o Offset move, lido do PAINEL.
pub(crate) fn offset_side() -> OffsetSide {
    side_of_code(ph2d_panel_vector::expand_side())
}

/// Aplica `cmd` a cada path SELECIONADO. `d` é a distância do offset (ignorada pelo
/// Outline Stroke). Um passo de undo para o gesto inteiro; re-seleciona o que saiu.
///
/// ⚠️ **Um passo de undo, não um por forma** — desfazer "o Expand" tem de custar um Ctrl+Z,
/// não tantos quantos objetos estavam selecionados (a lição que o bake da física pagou).
pub(crate) fn apply_vec_expand(
    scene: &mut VecScene,
    history: &mut History,
    pen: &mut PenTool,
    xforms: &VecXforms,
    cmd: Expand,
    d: f64,
) {
    let pre = scene.clone(); // UM passo de undo para o gesto inteiro
    let ids = pen.selected_paths().to_vec();
    if expand_selection(scene, pen, xforms, &ids, |_| Some((cmd, d))) {
        history.push_undo(pre);
        eprintln!("[ph2d-vec] expand {cmd:?}: ok");
    } else {
        eprintln!("[ph2d-vec] expand {cmd:?}: nada a converter na seleção");
    }
}

/// O NÚCLEO — offseta/converte cada path de `ids` no lugar (remove+insere) e re-seleciona
/// o que saiu. Devolve `true` se mudou algo. **Sem undo**: quem chama decide.
///
/// `cmd_for` responde *"o que faço com ESTE caminho?"* — `None` o deixa em paz. É por-caminho
/// (e não um comando único) porque o Offset agora é VIVO: cada forma carrega o seu
/// [`ph2d_ecs::VecOffset`], e materializar a seleção tem de honrar o de cada uma. Um segundo
/// laço para o caso vivo seria a 2ª porta por onde a geometria do offset entra no documento —
/// exatamente a doença que esta linha já pagou.
pub(crate) fn expand_selection(
    scene: &mut VecScene,
    pen: &mut PenTool,
    xforms: &VecXforms,
    ids: &[VecPathId],
    cmd_for: impl Fn(VecPathId) -> Option<(Expand, f64)>,
) -> bool {
    // Os z's dos alvos, de trás para a frente — cada path é substituído no lugar (o
    // total pode crescer), então se percorre da frente para trás.
    let mut zs: Vec<usize> = ids
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    if zs.is_empty() {
        return false;
    }

    let mut produced: Vec<VecPathId> = Vec::new();
    let mut touched = false;
    for &z in zs.iter().rev() {
        let Some(src) = scene.paths().get(z).cloned() else {
            continue;
        };
        let Some((cmd, d)) = cmd_for(src.id) else {
            produced.push(src.id); // fora do comando: fica onde está, e segue selecionado
            continue;
        };
        // ADR-0111: a pose vive no `Transform` da entidade e o resultado nasce world-space,
        // então o operando é assado no MUNDO antes de entrar no motor — como na booleana.
        // ⚠️ **A fonte entra COZIDA** (quina viva + pilha de efeitos): materializar tem de
        // congelar o que se VÊ. Sem o `cooked()` o Apply devolveria a curva crua offsetada e a
        // forma saltaria no clique — a mesma lei do `bake_cooked`.
        let mut world = src.cooked().into_owned();
        bake_xform(&mut world, &xform_of(xforms, src.id));

        let results = match cmd {
            Expand::Offset { join, side } => ph2d_vec_boolean::offset_path(&world, d, join, side),
            Expand::OutlineStroke => ph2d_vec_boolean::outline_stroke(&world),
            Expand::PowerStroke { profile } => ph2d_vec_boolean::power_stroke(&world, &profile),
        };
        if results.is_empty() {
            // Nada a fazer nesta forma (sem traço; ou um offset que a ANIQUILA). A fonte fica
            // onde está e segue selecionada — e, no caso vivo, o componente fica com ela: não
            // há geometria para materializar, e apagar a arte do artista num clique de "Apply"
            // seria a pior resposta possível a "não há nada aqui".
            produced.push(src.id);
            continue;
        }
        touched = true;

        // O Outline Stroke converte o TRAÇO. Se a forma também tinha PREENCHIMENTO, essa
        // região continua existindo e fica no lugar dela, agora sem traço — é o grupo de
        // dois objetos que o Illustrator produz. Sem fill não sobra nada da original.
        // Os dois comandos que assam TINTA convertem o traço e deixam o miolo em paz.
        let bakes_ink = matches!(cmd, Expand::OutlineStroke | Expand::PowerStroke { .. });
        let keep_fill = bakes_ink && src.fill.is_some();
        scene.remove_path(src.id);
        let mut at = z;
        if keep_fill {
            let mut base = world.clone();
            base.stroke = None;
            // `insert_path` cunha id novo (o antigo saiu com o `remove_path`) — guarda o
            // que ELE devolveu, senão a re-seleção aponta para um path que não existe.
            produced.push(scene.insert_path(at, base));
            at += 1;
        }
        for r in results {
            produced.push(scene.insert_path(at, r));
            at += 1;
        }
    }
    if touched {
        pen.select_many(&produced);
    }
    touched
}

/// **A escala do slider de Offset** — metade do maior lado da bbox de MUNDO da seleção.
///
/// O slider fala FRAÇÃO (`params::slider_to_offset_frac`) e `d = fração × esta escala`. A
/// meia-dimensão é o que dá sentido aos DOIS extremos do curso: o inradius de qualquer
/// forma é ≤ maxdim/2, então **−100% aniquila garantido** (não há d mais negativo que
/// ainda mostre algo); e a +100% o eixo maior cresce exatamente 2× (**dobrar**), com as
/// quinas — onde o join mora — na vizinhança da tela. A faixa fixa antiga (±4 de mundo)
/// entregava o gesto natural a regimes join-inertes — o report de 2026-07-20 ("se
/// selecionar Round, não consegue mudar"); a história completa em
/// `params::OFFSET_FRAC_MIN`.
///
/// Porta ÚNICA: o arrasto vivo e o botão Apply Offset perguntam AQUI — duas cópias
/// divergiriam no dia em que a lei mudasse. ⚠️ Não precisa mais ser CONGELADA no grab: o
/// preview deixou de churnar a cena, então a bbox das FONTES não se move durante o arrasto.
/// Multi-seleção usa a bbox da UNIÃO (um slider, um número). Seleção vazia/degenerada cai
/// em `1.0` — inerte de toda forma (`zs.is_empty()`/`results` vazio no expand).
pub(crate) fn offset_scale(scene: &VecScene, pen: &PenTool, xforms: &VecXforms) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for id in pen.selected_paths() {
        if let Some((l, h)) = scene.path_world_curve_bbox(xforms, *id) {
            lo = [lo[0].min(l[0]), lo[1].min(l[1])];
            hi = [hi[0].max(h[0]), hi[1].max(h[1])];
        }
    }
    let maxdim = (hi[0] - lo[0]).max(hi[1] - lo[1]);
    if maxdim.is_finite() && maxdim > 0.0 {
        maxdim * 0.5
    } else {
        1.0
    }
}

/// Os dois knobs do Offset como o PAINEL os guarda (`join`, `side`) — a chave de mudança que o
/// frame observa para retunar os offsets VIVOS da seleção. Porta única.
#[must_use]
pub(crate) fn expand_knobs() -> (u8, u8) {
    (
        ph2d_panel_vector::expand_join(),
        ph2d_panel_vector::expand_side(),
    )
}

#[cfg(test)]
#[path = "vec_expand_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "vec_expand_scale_tests.rs"]
mod scale_tests;
