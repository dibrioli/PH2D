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

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua (a regra que o `Brush::curva_inerte` da escultura pagou) e o pintor
/// não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: RayQueixa) -> &'static str {
    match q {
        RayQueixa::Desligado => "panel.inspector.ray.this_ray_is_off",
        RayQueixa::SemAlcance => "panel.inspector.ray.the_reach_is_zero",
        RayQueixa::EscolhaATag => "panel.inspector.ray.pick_a_tag",
        RayQueixa::TagApagada => "panel.inspector.ray.tag_is_gone",
        RayQueixa::CamadaVazia => "panel.inspector.ray.nothing_in_this_layer",
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
    if i.sees_all.len() > 1 {
        return todos_os_acertos(scene, text_system, theme, x, w, y, &i.sees_all);
    }
    let dist = crate::unidade_mundo::distancia_na_frase(i.sees_at);
    // RS-4: as tags da forma acertada (o «material») entre parênteses, quando ela as tem.
    let frase = if i.sees_tags.is_empty() {
        tr_with(
            "panel.inspector.ray.sees_x_at_y_m",
            &[("name", &i.sees.as_str()), ("dist", &dist)],
        )
    } else {
        tr_with(
            "panel.inspector.ray.sees_x_on_t_at_y_m",
            &[
                ("name", &i.sees.as_str()),
                ("tags", &i.sees_tags.as_str()),
                ("dist", &dist),
            ],
        )
    };
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &frase,
        ColorToken::Text1,
    )
}

/// Quantos acertos a leitura lista antes do «and N more» (o resto conta-se, não se pinta).
const ACERTOS_NA_LEITURA: usize = 4;

/// ⭐ Com «See all hits»: uma linha por acerto, do mais perto ao mais longe (a lista de acertos do
/// depurador de física do Godot), e «and N more» quando passam de [`ACERTOS_NA_LEITURA`].
fn todos_os_acertos(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    todos: &[(String, f32)],
) -> f32 {
    let mut cur_y = y;
    for (n, (nome, d)) in todos.iter().take(ACERTOS_NA_LEITURA).enumerate() {
        let chave = if n == 0 {
            "panel.inspector.ray.sees_x_at_y_m"
        } else {
            "panel.inspector.ray.and_x_at_y_m"
        };
        let dist = crate::unidade_mundo::distancia_na_frase(*d);
        let frase = tr_with(chave, &[("name", &nome.as_str()), ("dist", &dist)]);
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &frase,
            ColorToken::Text1,
        );
    }
    if todos.len() > ACERTOS_NA_LEITURA {
        let resto = (todos.len() - ACERTOS_NA_LEITURA).to_string();
        cur_y = super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with("panel.inspector.ray.and_n_more", &[("n", &resto.as_str())]),
            ColorToken::Text3,
        );
    }
    cur_y
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
        cur_y = super::rows::aviso(
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
        cur_y = super::rows::aviso(
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
            tr("component.field.ray_sensor.0"),
            tr("component.field.ray_sensor.1"),
            tr("component.field.ray_sensor.2"),
            tr("component.field.ray_sensor.3"),
            tr("component.field.ray_sensor.6"),
            tr("component.field.ray_sensor.7"),
            tr("component.field.ray_sensor.12"),
            tr("component.field.ray_sensor.13"),
            tr("component.field.ray_sensor.14"),
            tr("component.field.ray_sensor.15"),
            tr("component.field.ray_sensor.16"),
            tr("component.field.ray_sensor.17"),
            tr("component.field.ray_sensor.18"),
            tr("component.field.ray_signals.0"),
            tr("component.field.ray_signals.1"),
        ],
    );
    // RS-6: a caixa «Enabled» é a primeira linha (a do Godot e do Unity: ligar vem antes de afinar).
    cur_y = ph2d_editor_core::property_row::paint_check_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        (
            crate::ids::INSP_RAY_ENABLED,
            TextKey::new("component.field.ray_sensor.13"),
            i.enabled,
        ),
        seccao,
    );
    cur_y = super::ray_rumo::rumo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        i,
        seccao,
    );
    cur_y = super::ray_rumo::alcance_e_feixe(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        i,
        seccao,
    );
    // O grupo «Layers» (dono 15/11): a Layer e o Sees juntos, numa secção própria.
    cur_y = super::ray_camadas::grupo_das_camadas(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        i,
        seccao,
    );
    // O QUEM ele vê (versão C, dono 15/11): nasce fechado, com o resumo.
    cur_y = super::ray_filtro::grupo_do_filtro(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        i,
        seccao,
    );
    sinais(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        seccao,
    )
}

/// O grupo «Signals» (B): ligam-se noutro lugar, e a caixa diz quem ouve.
#[allow(clippy::too_many_arguments)]
fn sinais(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    let mut cur_y = y;
    // Os sinais num grupo próprio (B): ligam-se noutro lugar, e a caixa diz quem ouve.
    let g = super::grupo_dobravel::grupo_dobravel(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        crate::ids::INSP_RAY_FOLD_SIGNALS,
        tr("panel.inspector.ray.signals"),
    );
    if !g.aberto() {
        return g.fecha(store, scene, hit_index, cur_y);
    }
    cur_y = g.y;
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
        TextKey::new("component.field.ray_signals.0"),
        crate::ids::INSP_RAY_ON_ENTER,
        TextInput::new(crate::ids::INSP_RAY_ON_ENTER, "")
            .placeholder(tr("panel.inspector.ray.signal_when_it_starts_seeing_u")),
        seccao,
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
        TextKey::new("component.field.ray_signals.1"),
        crate::ids::INSP_RAY_ON_EXIT,
        TextInput::new(crate::ids::INSP_RAY_ON_EXIT, "")
            .placeholder(tr("panel.inspector.ray.signal_when_it_stops_seeing_u")),
        seccao,
    );
    g.fecha(store, scene, hit_index, cur_y)
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
        tr("component.ray_sensor.name"),
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

/// O Ray está no modo Rename (só por escolha: o estado nunca semeado é «fechado»).
#[must_use]
pub(crate) fn renomeia_o_raio(store: &ph2d_editor_core::interaction::WidgetStore) -> bool {
    store.collapsed_choice(crate::ids::INSP_RAY_LAYER_RENAMING) == Some(false)
}
