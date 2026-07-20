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

/// A junção do Offset, lida do PAINEL (`ph2d_panel_vector::expand_join`). Porta única: o
/// clique e o drag ao vivo perguntam à mesma, senão divergiriam num 4º estilo de quina.
pub(crate) fn offset_join() -> LineJoin {
    match ph2d_panel_vector::expand_join() {
        1 => LineJoin::Round,
        2 => LineJoin::Bevel,
        _ => LineJoin::Miter,
    }
}

/// Qual contorno o Offset move, lido do PAINEL. Porta única, como a junção.
pub(crate) fn offset_side() -> OffsetSide {
    match ph2d_panel_vector::expand_side() {
        0 => OffsetSide::Outer,
        1 => OffsetSide::Inner,
        _ => OffsetSide::Both,
    }
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
    // Os z's dos selecionados, de trás para a frente — mexer neles de TRÁS para a FRENTE
    // manteria os índices válidos, mas aqui cada path é substituído no lugar (o total pode
    // crescer), então se percorre ao contrário: da frente para trás.
    let mut zs: Vec<usize> = pen
        .selected_paths()
        .iter()
        .filter_map(|id| scene.paths().iter().position(|p| p.id == *id))
        .collect();
    zs.sort_unstable();
    zs.dedup();
    if zs.is_empty() {
        eprintln!("[ph2d-vec] expand: nada selecionado");
        return;
    }

    let pre = scene.clone(); // UM passo de undo para o gesto inteiro
    let mut produced: Vec<VecPathId> = Vec::new();
    let mut touched = false;
    for &z in zs.iter().rev() {
        let Some(src) = scene.paths().get(z).cloned() else {
            continue;
        };
        // ADR-0111: a pose vive no `Transform` da entidade e o resultado nasce world-space,
        // então o operando é assado no MUNDO antes de entrar no motor — como na booleana.
        let mut world = src.clone();
        bake_xform(&mut world, &xform_of(xforms, src.id));

        let results = match cmd {
            Expand::Offset { join, side } => ph2d_vec_boolean::offset_path(&world, d, join, side),
            Expand::OutlineStroke => ph2d_vec_boolean::outline_stroke(&world),
            Expand::PowerStroke { profile } => ph2d_vec_boolean::power_stroke(&world, &profile),
        };
        if results.is_empty() {
            continue; // nada a fazer nesta forma (sem traço, offset que a some)
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
    if !touched {
        eprintln!("[ph2d-vec] expand {cmd:?}: nada a converter na seleção");
        return;
    }
    history.push_undo(pre);
    pen.select_many(&produced);
    eprintln!("[ph2d-vec] expand {cmd:?}: ok ({} path[s])", produced.len());
}

#[cfg(test)]
#[path = "vec_expand_tests.rs"]
mod tests;
