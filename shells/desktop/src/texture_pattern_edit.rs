//! **AUTORAR a lei de um padrão de textura** (plano 33, W5) — a porta única entre a secção
//! *Pattern* do painel e o documento.
//!
//! ⚠️⚠️ **NÃO confundir com o [`crate::pattern_live`]**, que é o *Pattern Along Path* (plano 23).
//!
//! # Uma porta, um passo de undo
//!
//! Todo controlo da secção desagua aqui, e cada mudança é **um** passo de undo — a mesma disciplina
//! do `apply_vec_set_fill_kind`. E o `if` de igualdade no fim é o que impede um passo espúrio quando
//! o slider re-publica o valor que já lá estava (o defeito que fazia todo quadro virar undo).

use ph2d_vec_pattern::{PatternMode, TileKind};
use ph2d_vec_scene::{Paint, PatternFill, PatternSource, VecScene};

/// O que a secção *Pattern* pede ao documento.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TexPatCmd {
    /// Trocar o reticulado (`0` Grid · `1` Brick · `2` Column · `3` Hex).
    Tile(u8),
    /// Trocar a lei de repetição (`0` Tile · `1` Mirror · `2` Clamp).
    Mode(u8),
    /// O desfasamento é `1/n`.
    OffsetDenom(f64),
    /// ⭐ **UM eixo do tamanho** (`0` = largura, `1` = altura) e o estado do CADEADO.
    ///
    /// ⛔ Era `Size(f64)` — o lado maior, com o aspecto sempre preservado. O Enio pediu (2026-08-27)
    /// para poder achatar a arte **de propósito**, e a protecção mudou de lei imposta para gesto
    /// escolhido. ⚠️ O cadeado viaja **no comando** porque ele é da SESSÃO (a shell é a dona), e a
    /// lei que ele governa vive na porta única `PatternFill::set_axis`.
    Axis(u8, f64, bool),
    /// ⭐ **A FASE dentro de UMA repetição**, em percentagem, no eixo `0` (X do padrão) ou `1` (Y).
    ///
    /// ⚠️ Substitui a alça de MOVER do plano 33 W6, retirada por decisão do Enio (2026-08-27:
    /// *"não ficou legal. vamos retirar e deixar os ajustes apenas no painel"*). O tamanho e a
    /// rotação já tinham slider; a posição não tinha nenhum, e sem isto retirar as alças teria
    /// tirado ao artista uma coisa que ele fazia.
    Shift(u8, f64),
    /// O vão acrescentado, em unidades de mundo (negativo = sobreposição).
    Gap(f64),
    /// A rotação do padrão, em graus.
    Angle(f64),
    /// Trocar a ARTE, mantendo a lei.
    Source(PatternSource),
}

/// O reticulado que o índice do painel nomeia. ⚠️ Porta única: o painel oferece por índice, e a
/// tradução vive **aqui**, num sítio só.
fn tile_of(i: u8) -> TileKind {
    match i {
        1 => TileKind::BrickRow,
        2 => TileKind::BrickCol,
        3 => TileKind::Hex,
        _ => TileKind::Grid,
    }
}

/// O índice do painel para um reticulado — a gémea de [`tile_of`].
#[must_use]
pub(crate) fn tile_index(k: TileKind) -> u8 {
    match k {
        TileKind::Grid => 0,
        TileKind::BrickRow => 1,
        TileKind::BrickCol => 2,
        TileKind::Hex => 3,
    }
}

fn mode_of(i: u8) -> PatternMode {
    match i {
        1 => PatternMode::Mirror,
        2 => PatternMode::Clamp,
        _ => PatternMode::Tile,
    }
}

/// O índice do painel para uma lei de repetição — a gémea de [`mode_of`].
#[must_use]
pub(crate) fn mode_index(m: PatternMode) -> u8 {
    match m {
        PatternMode::Tile => 0,
        PatternMode::Mirror => 1,
        PatternMode::Clamp => 2,
    }
}

/// **Troca a ARTE do padrão da forma `host`** — a porta do Picker (W7), que resolve por ID e não
/// pela seleção.
///
/// ⚠️ Existe ao lado do [`apply`] porque o alvo é **capturado no arm** do pick: o clique seguinte
/// cai noutra forma, e ela passa a ser a selecionada. Ler a seleção aqui apontaria o padrão para a
/// forma errada — exactamente o *"escolhendo a si mesmo"* que o Picker foi criado para eliminar.
pub(crate) fn set_source(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    host: ph2d_vec_scene::VecPathId,
    source: PatternSource,
) -> bool {
    let Some(Paint::Pattern(cur)) = scene.path(host).and_then(|p| p.fill.as_ref()) else {
        return false;
    };
    if cur.source == source {
        return false;
    }
    let mut next = (**cur).clone();
    next.source = source;
    let pre = scene.clone();
    if let Some(path) = scene.path_mut(host) {
        path.fill = Some(Paint::Pattern(Box::new(next)));
        history.push_undo(pre);
        return true;
    }
    false
}

/// Aplica `cmd` ao padrão da forma selecionada. No-op silencioso quando não há forma, quando ela
/// não tem padrão, ou quando o valor já era esse.
pub(crate) fn apply(
    scene: &mut VecScene,
    history: &mut ph2d_vec_edit::History,
    pen: &ph2d_vec_edit::PenTool,
    cmd: TexPatCmd,
) {
    let Some(sel) = pen.selected() else {
        return;
    };
    let Some(Paint::Pattern(cur)) = scene.path(sel).and_then(|p| p.fill.as_ref()) else {
        return;
    };
    let mut next: PatternFill = (**cur).clone();
    match cmd {
        TexPatCmd::Tile(i) => next.kind = tile_of(i),
        // ⚠️⚠️ **O enquadramento do `Clamp` NÃO se escreve — ele é DERIVADO no desenho.**
        //
        // A 1.ª cura escrevia `size`/`origin` ao entrar no modo, e o report seguinte do Enio
        // apanhou-a: *"quando volta para tile o aspecto fica de clamp até mudar o parâmetro
        // Size"*. Escrever destruía a lei que o artista tinha afinado, e voltar não a devolvia —
        // um modo de APRESENTAÇÃO não pode consumir o documento.
        //
        // A lei vive agora em `PatternFill::placement_in`, que enquadra só enquanto o modo é
        // `Clamp` e devolve a colocação autorada em qualquer outro. *O que é vista não se grava.*
        TexPatCmd::Mode(i) => next.mode = mode_of(i),
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        TexPatCmd::OffsetDenom(n) => next.offset_denom = n.clamp(1.0, 255.0).round() as u8,
        // ⚠️ Pela porta ÚNICA do tamanho (`ph2d-vec-scene`). Com o cadeado, os dois eixos escalam
        // pelo MESMO factor — a razão ACTUAL sobrevive, e não a natural da arte: voltar ao aspecto
        // da imagem desfaria o achatamento que o artista acabou de autorar.
        TexPatCmd::Axis(axis, v, lock) => next.set_axis(usize::from(axis), v, lock),
        // ⚠️ **A base da fase é o canto da CAIXA da forma** — o mesmo canto em que a colocação
        // nasce (`texture_pattern_pick::default_placement`). Sem uma referência ligada à forma, a
        // fase de um padrão dependeria de onde a forma está no mundo.
        TexPatCmd::Shift(axis, pct) => {
            if let Some((lo, _)) = scene.path_bbox(sel) {
                next.set_shift_axis(lo, usize::from(axis), pct / 100.0);
            }
        }
        TexPatCmd::Gap(v) => next.gap = [v, v],
        TexPatCmd::Angle(deg) => next.angle = deg.to_radians(),
        TexPatCmd::Source(s) => next.source = s,
    }
    if next == **cur {
        return;
    }
    let pre = scene.clone();
    if let Some(path) = scene.path_mut(sel) {
        path.fill = Some(Paint::Pattern(Box::new(next)));
        history.push_undo(pre);
    }
}

#[cfg(test)]
#[path = "texture_pattern_edit_tests.rs"]
mod tests;

/// `PH2D_PATTERN_LOG=1` — o diagnóstico do padrão.
///
/// ⚠️ **Por EVENTO, nunca por quadro.** A primeira sonda que esta linha escreveu (a do morfo) foi
/// devolvida com *"há milhares de logs"*: um log por quadro afoga o que se quer ler.
fn log_on() -> bool {
    std::env::var("PH2D_PATTERN_LOG").is_ok_and(|v| v != "0")
}

/// Imprime o que a forma selecionada TEM neste instante — a tinta, o traço e a caixa.
///
/// ⚠️ Existe por causa do report de 2026-08-27 (*"pattern anula stroke"* / *"o contorno não volta ao
/// trocar pattern por solid"*) que **não reproduziu** em nenhum gate: o documento preserva o traço
/// nas duas trocas, o `restyle_selected_strokes` nunca o apaga, e a rota de desenho encoda os dois
/// caminhos. ⇒ *o que falta não é ler mais código, é um INSTRUMENTO* — foi assim que a máquina de
/// estados do Morph fechou.
pub(crate) fn log_shape(tag: &str, scene: &VecScene, pen: &ph2d_vec_edit::PenTool) {
    if !log_on() {
        return;
    }
    let Some(sel) = pen.selected() else {
        eprintln!("[pattern] {tag}: nenhuma forma selecionada");
        return;
    };
    let Some(path) = scene.path(sel) else {
        eprintln!("[pattern] {tag}: a forma {sel} sumiu da cena");
        return;
    };
    let tinta = match &path.fill {
        None => "SEM preenchimento".to_string(),
        Some(Paint::Solid(c)) => format!("Solid({},{},{},{})", c.r, c.g, c.b, c.a),
        Some(Paint::Linear { .. }) => "Linear".into(),
        Some(Paint::Radial { .. }) => "Radial".into(),
        Some(Paint::MultiPoint { .. }) => "MultiPoint".into(),
        Some(Paint::Pattern(p)) => format!(
            "Pattern(kind={:?} mode={:?} size={:?} gap={:?} origin={:?} alpha={})",
            p.kind, p.mode, p.size, p.gap, p.origin, p.alpha
        ),
    };
    let traco = path.stroke.as_ref().map_or("SEM traco".to_string(), |s| {
        format!(
            "Stroke(cor={},{},{},{} largura={} align={:?} dash={})",
            s.color().r,
            s.color().g,
            s.color().b,
            s.color().a,
            s.width,
            s.align,
            s.dash.is_some()
        )
    });
    eprintln!(
        "[pattern] {tag}: forma {sel} · {tinta} · {traco} · fechada={} · contornos={} · bbox={:?}",
        path.closed,
        1 + path.subpaths.len(),
        scene.path_bbox(sel)
    );
}
