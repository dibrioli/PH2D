//! ⭐⭐⭐ **O que o Inspector mostra do RAIO** (suplente #21).
//!
//! # ⭐⭐⭐ A LEITURA VIVA é a razão de esta secção existir
//!
//! Quatro números e dois nomes cabiam numa tabela genérica. O que não cabe é **o que o raio vê
//! AGORA** — o nome de quem ele acertou e a que distância —, que não vem de campo nenhum: vem do
//! mapa da ponte, onde a corrida vive. É a mesma razão pela qual o `ProbeState` do platformer
//! existe, e é ela que transforma seis campos numa ferramenta que se afina **a olhar**.
//!
//! # ⭐⭐ E a QUEIXA vem antes dos números
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `the direction is zero` | ⛔ **nulo não é um raio** — a porta do motor devolve `None` |
//! | `the reach is zero` | ele nasce e morre no mesmo ponto |
//! | `this ray says nothing` | ele vê e não conta a ninguém |
//! | `sees nothing right now` | está a olhar e não há nada no alcance — a única que é **normal** |
//!
//! ⚠️ **Os dois primeiros são de outra espécie que os dois últimos:** ali o raio **não corre**, aqui
//! ele corre e não encontra. *Dizer «não vê nada» a quem tem a direcção a zero é mandá-lo resolver a
//! metade errada* — a lei da recusa dos pincéis.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorRayInfo::queixa`], e o gate dela corre **sem um device**. *Quatro `if` dentro de um
//! pintor só se medem com uma janela, e um gate `#[ignore]` é um gate que o CI nunca corre.*

use super::*;
use ph2d_editor_core::ray_edits::{InspectorRayInfo, RayQueixa};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

/// Uma linha de aviso. Devolve o `y` seguinte. (Gémea da do irmão projéctil.)
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

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua (a regra que o `Brush::curva_inerte` da escultura pagou) e o pintor
/// não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: RayQueixa) -> &'static str {
    match q {
        RayQueixa::SemDireccao => "panel.inspector.ray.the_direction_is_zero",
        RayQueixa::SemAlcance => "panel.inspector.ray.the_reach_is_zero",
        RayQueixa::Mudo => "panel.inspector.ray.this_ray_says_nothing",
        RayQueixa::NaoVeNada => "panel.inspector.ray.sees_nothing_right_now",
    }
}

/// **O que ele vê AGORA** — a linha que faz esta secção valer a pena.
///
/// ⚠️ **`Text1` quando vê, `Text3` quando não vê**, e a diferença não é enfeite: uma leitura viva
/// que muda de valor a cada quadro tem de se distinguir de um aviso parado, senão o olho lê as duas
/// como a mesma coisa.
#[allow(clippy::too_many_arguments)]
fn leitura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
) -> f32 {
    if i.sees.is_empty() {
        return y;
    }
    warn(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &tr_with(
            "panel.inspector.ray.sees_x_at_y_m",
            &[
                ("name", &i.sees.as_str()),
                ("dist", &format!("{:.2}", i.sees_at)),
            ],
        ),
        ColorToken::Text1,
    )
}

/// O corpo da secção — os CONTROLOS.
#[allow(clippy::too_many_arguments)]
fn corpo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorRayInfo,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar uma camada.
    if let Some(q) = i.queixa() {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr(chave_da_queixa(q)),
            ColorToken::Text3,
        );
    }
    cur_y = leitura(scene, text_system, theme, x, w, cur_y, i);
    if !i.clock_playing {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.ray.the_clock_is_stopped_u_the_reading_is_the_pose_you_see"),
            ColorToken::Text3,
        );
    }

    // ⭐⭐ **A coluna do nome é da SECÇÃO** (`line/UIUX`, 2026-09-15), e os nomes são os da secção
    //    INTEIRA — *uma coluna que salta quando uma linha aparece é uma coluna por linha com outro
    //    nome*.
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        2,
        &[
            tr("panel.inspector.ray.origin"),
            tr("panel.inspector.ray.direction"),
            tr("panel.inspector.ray.reach_m"),
            tr("panel.inspector.ray.layer"),
        ],
    );
    // ⚠️ **A origem e a direcção são UMA linha de DOIS campos cada** — elas são um ponto e um
    // vector, e parti-las em quatro linhas faria o artista procurar o par.
    for (label, ids, step, unidade) in [
        (
            tr("panel.inspector.ray.origin"),
            [crate::ids::INSP_RAY_ORIGIN_X, crate::ids::INSP_RAY_ORIGIN_Y],
            0.1, // LITERAL-PX-OK: passo de arrasto em METROS, locais
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            tr("panel.inspector.ray.direction"),
            [crate::ids::INSP_RAY_DIR_X, crate::ids::INSP_RAY_DIR_Y],
            0.1, // LITERAL-PX-OK: passo de arrasto de uma DIRECÇÃO, adimensional
            None,
        ),
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &ids,
            step,
            unidade,
            seccao,
        );
    }
    for (label, id, step, unidade) in [
        (
            tr("panel.inspector.ray.reach_m"),
            crate::ids::INSP_RAY_REACH,
            0.1, // LITERAL-PX-OK: passo de arrasto em METROS
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            tr("panel.inspector.ray.layer"),
            crate::ids::INSP_RAY_LAYER,
            1.0,
            None,
        ), // LITERAL-PX-OK: índice
    ] {
        cur_y = super::rows::fields_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            label,
            &[id],
            step,
            unidade,
            seccao,
        );
    }
    // ⚠️ **Os dois NOMES, e vazio = calado** — a regra do `SignalOnHit`, palavra por palavra.
    //
    // ⛔ **Pela porta que já existe** (`anim_rows::text_row`): copiá-la seria a terceira resposta a
    // *«como se desenha um campo de texto de uma row do Inspector?»*, e o doc dela diz que é
    // precisamente na cópia que a lei do `placeholder` se perde — o gate do HR-15 conta
    // `.placeholder("…")` no código de widget, e uma string passada por argumento sai da vista dele.
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_RAY_ON_ENTER,
        TextInput::new(crate::ids::INSP_RAY_ON_ENTER, "")
            .placeholder(tr("panel.inspector.ray.signal_when_it_starts_seeing_u")),
    );
    cur_y = super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_RAY_ON_EXIT,
        TextInput::new(crate::ids::INSP_RAY_ON_EXIT, "")
            .placeholder(tr("panel.inspector.ray.signal_when_it_stops_seeing_u")),
    );
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_ray_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorRayInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION,
        tr("panel.inspector.ray.ray_sensor"),
    );
    paint_section_header(
        &header,
        Rect::new(x, y, w, header_h),
        scene,
        text_system,
        theme,
    );
    let Some(fold) = SectionFold::begin(
        store,
        ph2d_editor_core::ids::INSP_LIVE_RAY_SECTION,
        x,
        w,
        y + header_h,
        scene,
        hit_index,
    ) else {
        return y + header_h;
    };
    let mut cur_y = y + header_h;
    if info.selected_count > 1 {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            tr("panel.inspector.ray.editing_the_primary_selection_only"),
            ColorToken::Text3,
        );
    }
    cur_y = corpo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        info,
    );
    fold.finish(store, scene, hit_index, cur_y)
}
