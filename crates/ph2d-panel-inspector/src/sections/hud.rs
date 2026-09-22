//! ⭐⭐⭐ **O que o Inspector mostra do HUD** (TOP-20 #20).
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `No game camera` | não há vista a que colar: o canvas fica onde o artista o pôs |
//! | `Showing …` | o que o rótulo mostra AGORA — o número derivado, já com prefixo e sufixo |
//! | `That source is not in the scene` | a fonte não existe: o texto autorado é que aparece |
//! | `No name` | o botão não publica nada — um nome em branco não é um contrato |
//! | `This button refuses the click` | está `Disabled`, e é por isso que ele não responde |
//! | `Now` | o valor VIVO do contador, que não é editável (não é documento) |
//!
//! ⚠️ **Sem eles, um HUD que está exactamente como o artista pediu lê-se como partido, com todos
//! os números certos no ecrã** — a lição do projéctil, do mover de vista de cima e do emissor.
//!
//! # ⭐ E os BLOCOS somem conforme o objecto
//!
//! Um objecto de HUD raramente tem os quatro componentes. O instantâneo diz quais existem
//! ([`InspectorHudInfo::mostra_numero`] / [`InspectorHudInfo::mostra_texto`]) e o que não tem dono
//! não é pintado — *mostrar sempre os doze campos entregaria nove controlos mortos*.

use super::*;
use ph2d_editor_core::hud_edits::{HUD_NUMBERS, HudNumber as N, HudText as T, InspectorHudInfo};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::TextKey;
use ph2d_i18n::tr;

/// O rótulo, o passo e a unidade de cada número — **pela ordem do modelo**, com gate a atar os
/// comprimentos.
const ROTULOS: [(TextKey, f64, Option<ph2d_editor_core::widget::Unit>); 3] = [
    (
        TextKey::new("panel.inspector.hud.reference_width"),
        0.5, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.hud.reference_height"),
        0.5, // LITERAL-PX-OK: metros
        Some(ph2d_editor_core::widget::Unit::Meters),
    ),
    (
        TextKey::new("panel.inspector.hud.counter_start"),
        1.0, // LITERAL-PX-OK: contagem (inteiro)
        None,
    ),
];

/// Uma linha de aviso. (Gémea da do emissor — ver a irmã.)
#[allow(clippy::too_many_arguments)]
fn warn(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
    token: ColorToken,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(token, theme),
    );
    y + font + ph2d_tokens::control_gap_px()
}

/// Um título de bloco.
fn titulo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    texto: &str,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        texto,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text3, theme),
    );
    // ⚠️ A cauda de um bloco tem UMA porta — o mesmo degrau de sempre.
    y + font + ph2d_tokens::control_gap_px()
}

/// Um segmentado. ⚠️ **A selecção vem do SNAPSHOT, nunca do store**: ler o store faria o primeiro
/// clique depois de trocar de objecto mandar o valor anterior.
#[allow(clippy::too_many_arguments)]
fn seg_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    titulo_txt: &str,
    ids: &[ph2d_a11y::NodeId],
    rotulos: &[&'static str],
    escolhido: usize,
) -> f32 {
    let row_y = titulo(scene, text_system, theme, x, w, y, titulo_txt);
    let gap = Spacing::Xs.px();
    let n = ids.len() as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, &id) in ids.iter().enumerate() {
        let rect = Rect::new(x + (cw + gap) * i as f32, row_y, cw, ph2d_tokens::ROW_H_PX);
        hit_index.register(id, rect);
        let kind = if i == escolhido {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, rotulos.get(i).copied().unwrap_or(""))
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    row_y + ph2d_tokens::row_pitch_px()
}

/// Uma caixa — **pela porta**, com a coluna do nome da SECÇÃO.
///
/// ⛔⛔ Ela era uma cópia local do [`ph2d_editor_core::property_row::paint_check_row`] com a
/// altura escrita à mão (`18`, que é a aresta da MARCA e não a altura da LINHA) — o report do
/// dono de 2026-09-21: *«apenas o checkbox tem sua moldura e ele próprio menores que o padrão»*.
#[allow(clippy::too_many_arguments)]
fn check_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    label: &str,
    on: bool,
) -> f32 {
    // ⚠️ A MESMA coluna que a [`num_row`] desta secção mede, e pela mesma tabela — senão o nome
    //    de uma linha de marcar cai num `x` e o da linha de número acima dela noutro.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &ROTULOS.map(|(chave, _, _)| chave.tr()),
    );
    ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        (id, label, on),
        seccao,
    )
}

/// Uma linha de número, pela ordem do modelo.
#[allow(clippy::too_many_arguments)]
fn num_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    which: N,
) -> f32 {
    let Some(i) = HUD_NUMBERS.iter().position(|&n| n == which) else {
        return y;
    };
    let (chave, step, unidade) = ROTULOS[i];
    // ⭐⭐ A coluna do nome é da SECÇÃO: mede-se da tabela inteira, senão ela salta de linha
    // para linha.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &ROTULOS.map(|(chave, _, _)| chave.tr()),
    );
    super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        chave.tr(),
        &[crate::ids::INSP_HUD_NUM[i]],
        step,
        unidade,
        seccao,
    )
}

/// Uma linha de texto, pela ordem do modelo.
#[allow(clippy::too_many_arguments)]
fn txt_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: usize,
    rotulo: &'static str,
) -> f32 {
    let id = crate::ids::INSP_HUD_TEXT[i];
    // ⛔⛔ **O nome vai POR CIMA, e não no `TextInput`** — a foto apanhou três campos seguidos sem
    // um único nome à vista. O `text_row` pinta o controlo na largura toda e **não desenha o
    // rótulo**; e pô-lo no `placeholder` seria pior do que nada, porque um placeholder desaparece
    // exactamente quando o campo tem valor — que é quando o artista precisa de saber o que é.
    let row_y = titulo(scene, text_system, theme, x, w, y, rotulo);
    super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        row_y,
        id,
        TextInput::new(id, ""),
    )
}

#[path = "hud_corpo.rs"]
mod irmao;
pub(crate) use irmao::*;
