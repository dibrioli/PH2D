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
    super::rows::aviso(
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
                ("dist", &crate::unidade_mundo::distancia_na_frase(i.sees_at)),
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
        ],
    );
    // ⚠️ **A origem e a direcção são UMA linha de DOIS campos cada** — elas são um ponto e um
    // vector, e parti-las em quatro linhas faria o artista procurar o par.
    for (label, ids, step, unidade) in [
        (
            TextKey::new("component.field.ray_sensor.0"),
            [crate::ids::INSP_RAY_ORIGIN_X, crate::ids::INSP_RAY_ORIGIN_Y],
            0.1, // LITERAL-PX-OK: passo de arrasto em METROS, locais
            Some(ph2d_editor_core::widget::Unit::Meters),
        ),
        (
            TextKey::new("component.field.ray_sensor.1"),
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
    // Reach e a camada: no cartão, irmãos curtos que dividem a linha (lei 1); fora dele, um por linha. A camada é a
    // MESMA lista do collider (nomes, contorno da cor, o losango do campo) e «Sees», a linha da matriz do mundo
    // dessa camada (dono 08/11: «os mesmos campos»).
    let [(xa, wa), (xb, wb)] = if crate::cards::active() {
        ph2d_editor_core::panel::cards::corpo::colunas_iguais(x, w)
    } else {
        [(x, w), (x, w)]
    };
    let depois_do_alcance = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        xa,
        wa,
        cur_y,
        TextKey::new("component.field.ray_sensor.2"),
        &[crate::ids::INSP_RAY_REACH],
        0.1, // LITERAL-PX-OK: passo de arrasto em METROS
        Some(ph2d_editor_core::widget::Unit::Meters),
        seccao,
    );
    let topo_da_camada = if crate::cards::active() {
        cur_y
    } else {
        depois_do_alcance
    };
    let mut t = super::physics_cartao::Tinta {
        scene,
        text: text_system,
        theme,
        hit: hit_index,
        store,
        x: xb,
        w: wb,
    };
    let camada = u8::try_from(i.layer)
        .ok()
        .filter(|&c| usize::from(c) < crate::ids::INSP_PHYS_LAYERS)
        .unwrap_or_default();
    let rotulo = TextKey::new("component.field.ray_sensor.3");
    let quem = &super::physics_colide::DO_RAIO;
    let depois_da_camada = super::camada_por_nome::linha_da_camada(
        &mut t,
        topo_da_camada,
        quem,
        camada,
        rotulo.into(),
        seccao,
    );
    cur_y = depois_do_alcance.max(depois_da_camada);
    (t.x, t.w) = (x, w);
    cur_y = botoes_do_raio(&mut t, cur_y, usize::from(camada));
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

/// Os botões do Ray no lugar e idioma dos do collider (emenda 4, dono 08/11), só em cartões como o cartão Collision
/// Layers; renomeando, os campos de todas as camadas e «Done» tomam o lugar de «Sees».
fn botoes_do_raio(t: &mut super::physics_cartao::Tinta<'_>, y: f32, camada: usize) -> f32 {
    let sees = tr("panel.inspector.ray.sees");
    if !crate::cards::active() {
        return super::physics_colide::fila_da_matriz(
            t,
            y,
            &super::physics_colide::DO_RAIO,
            camada,
            sees,
        );
    }
    if renomeia_o_raio(t.store) {
        let y = super::physics_colide::rename(t, y, camada);
        let feito = tr("card.insp.phys.layer_rename_done");
        let (r, y) = t.botao(crate::ids::INSP_RAY_LAYER_RENAME_DONE, feito, y);
        t.hit.register(crate::ids::INSP_RAY_LAYER_RENAME_DONE, r);
        return y;
    }
    let y = t.dois_botoes(
        y,
        (
            crate::ids::INSP_RAY_LAYER_RENAME,
            tr("card.insp.phys.layer_rename"),
        ),
        (
            crate::ids::INSP_RAY_EDIT_MATRIX,
            tr("card.insp.phys.edit_matrix"),
        ),
    );
    super::physics_colide::fila_da_matriz(t, y, &super::physics_colide::DO_RAIO, camada, sees)
}

/// O Ray está no modo Rename (só por escolha: o estado nunca semeado é «fechado»).
#[must_use]
pub(crate) fn renomeia_o_raio(store: &ph2d_editor_core::interaction::WidgetStore) -> bool {
    store.collapsed_choice(crate::ids::INSP_RAY_LAYER_RENAMING) == Some(false)
}
