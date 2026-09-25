//! ⭐⭐⭐ **O que o Inspector mostra da ARMA.**
//!
//! # ⭐⭐⭐ A LEITURA VIVA é a razão de esta secção existir
//!
//! Dois números e seis nomes cabiam numa tabela genérica. O que não cabe é **quantas balas ela tem
//! AGORA** e **se está a recarregar** — que não vêm de campo nenhum: vêm do `CounterRuntime` e do
//! `WeaponRuntime`, onde a corrida vive. É a mesma razão pela qual a secção do raio mostra *o que
//! ele vê agora*, e é ela que transforma oito campos numa ferramenta que se afina **a olhar**.
//!
//! # ⭐⭐ E a QUEIXA vem antes dos números
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `this weapon has no trigger` | ⛔ `on_signal` vazio ⇒ ela **nunca** dispara |
//! | `the shot goes nowhere` | ela dispara e nada nasce — falta o sinal que a fábrica ouve |
//! | `the magazine is not here` | ⛔ há um nome de pente e **nenhum contador neste objecto** |
//! | `it is dry for good` | o pente está a zero e não há recarga — a única que pode ser o desenho |
//!
//! ⚠️ **As duas primeiras são de outra espécie que as duas últimas:** ali a arma **não corre**;
//! aqui ela corre e não produz. *Dizer «o pente não está aqui» a quem não tem gatilho é mandá-lo
//! resolver a metade errada* — a lei da recusa dos pincéis.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorWeaponInfo::queixa`], e o gate dela corre **sem um device**. *Quatro `if` dentro de um
//! pintor só se medem com uma janela, e um gate `#[ignore]` é um gate que o CI nunca corre.*

use super::*;
use ph2d_editor_core::weapon_edits::{
    InspectorWeaponInfo, WEAPON_STEP_MS as PASSO_MS, WeaponQueixa,
};
use ph2d_editor_core::widget::SectionFold;
use ph2d_i18n::{tr, tr_with};

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua (a regra que o `Brush::curva_inerte` da escultura pagou) e o pintor
/// não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: WeaponQueixa) -> &'static str {
    match q {
        WeaponQueixa::SemGatilho => "panel.inspector.weapon.this_weapon_has_no_trigger",
        WeaponQueixa::SemSaida => "panel.inspector.weapon.the_shot_goes_nowhere",
        WeaponQueixa::PenteAusente => "panel.inspector.weapon.the_magazine_is_not_here",
        WeaponQueixa::DepositoAusente => "panel.inspector.weapon.the_depot_is_not_reachable",
        WeaponQueixa::SecaParaSempre => "panel.inspector.weapon.it_is_dry_for_good",
        WeaponQueixa::DepositoVazio => "panel.inspector.weapon.the_depot_is_empty",
    }
}

/// **Quantas balas ela tem AGORA** — a linha que faz esta secção valer a pena.
///
/// ⚠️ **`Text1` quando há pente, `Text3` quando está a recarregar**, e a diferença não é enfeite:
/// uma leitura viva que muda a cada tiro tem de se distinguir de um aviso parado, senão o olho lê as
/// duas como a mesma coisa.
#[allow(clippy::too_many_arguments)]
fn leitura(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    i: &InspectorWeaponInfo,
) -> f32 {
    if i.recarregando {
        return super::rows::aviso(
            scene,
            text_system,
            theme,
            x,
            w,
            y,
            tr("panel.inspector.weapon.reloading_u"),
            ColorToken::Text3,
        );
    }
    let Some(n) = i.municao else {
        return y;
    };
    super::rows::aviso(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &tr_with(
            "panel.inspector.weapon.x_of_y_rounds",
            &[("have", &n.to_string()), ("full", &i.pente.to_string())],
        ),
        ColorToken::Text1,
    )
}

/// Uma linha de texto da secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn nome_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    id: ph2d_a11y::NodeId,
    rotulo: &str,
    dica: &str,
    seccao: ph2d_editor_core::property_row::Seccao,
) -> f32 {
    // ⛔ **Pela porta que já existe** (`anim_rows::text_row`): copiá-la seria a terceira resposta a
    // *«como se desenha um campo de texto de uma row do Inspector?»*, e o doc dela diz que é
    // precisamente na cópia que a lei do `placeholder` se perde.
    super::anim_rows::text_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        y,
        rotulo,
        id,
        TextInput::new(id, "").placeholder(dica),
        seccao,
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
    i: &InspectorWeaponInfo,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada acontecer não quer afinar uma cadência.
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
            tr("panel.inspector.weapon.the_clock_is_stopped_u_a_weapon_only_fires_during_a_run"),
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
            tr("panel.inspector.weapon.cooldown_ms"),
            tr("panel.inspector.weapon.reload_ms"),
        ],
    );
    for (label, id) in [
        (
            tr("panel.inspector.weapon.cooldown_ms"),
            crate::ids::INSP_WEAPON_COOLDOWN,
        ),
        (
            tr("panel.inspector.weapon.reload_ms"),
            crate::ids::INSP_WEAPON_RELOAD_MS,
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
            &[id],
            PASSO_MS,
            Some(ph2d_editor_core::widget::Unit::Milliseconds),
            seccao,
        );
    }
    // ⚠️ **Os SETE nomes, e vazio = calado** — a regra do `SignalOnHit`, palavra por palavra.
    for (id, rotulo, dica) in [
        (
            crate::ids::INSP_WEAPON_ON_SIGNAL,
            tr("panel.inspector.weapon.on_signal_label"),
            tr("panel.inspector.weapon.signal_that_pulls_the_trigger_u"),
        ),
        (
            crate::ids::INSP_WEAPON_ON_FIRE,
            tr("panel.inspector.weapon.on_fire_label"),
            tr("panel.inspector.weapon.signal_it_publishes_on_each_shot_u"),
        ),
        (
            crate::ids::INSP_WEAPON_AMMO,
            tr("panel.inspector.weapon.magazine_label"),
            tr("panel.inspector.weapon.counter_that_is_the_magazine_u"),
        ),
        (
            crate::ids::INSP_WEAPON_RELOAD_ON,
            tr("panel.inspector.weapon.reload_on_label"),
            tr("panel.inspector.weapon.signal_that_reloads_it_u"),
        ),
        (
            crate::ids::INSP_WEAPON_ON_EMPTY,
            tr("panel.inspector.weapon.on_empty_label"),
            tr("panel.inspector.weapon.signal_on_the_dry_click_u"),
        ),
        (
            crate::ids::INSP_WEAPON_ON_RELOADED,
            tr("panel.inspector.weapon.on_reloaded_label"),
            tr("panel.inspector.weapon.signal_when_the_magazine_is_full_u"),
        ),
        // ⭐⭐ **O DEPÓSITO, colado ao pente** — eles são a mesma pergunta em dois degraus, e
        // separá-los por três sinais faria o artista ler o segundo como sendo de outro assunto.
        (
            crate::ids::INSP_WEAPON_RESERVE,
            tr("panel.inspector.weapon.reserve_label"),
            tr("panel.inspector.weapon.counter_that_is_the_depot_u"),
        ),
    ] {
        cur_y = nome_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            id,
            rotulo,
            dica,
            seccao,
        );
    }
    cur_y
}

/// Pinta a secção. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_weapon_section(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    info: &InspectorWeaponInfo,
) -> f32 {
    let header_h = TypeToken::Md.px() + Spacing::Md.px(); // LITERAL-PX-OK: banda do cabeçalho
    let header = section_header(
        store,
        ph2d_editor_core::ids::INSP_LIVE_WEAPON_SECTION,
        tr("panel.inspector.weapon.weapon"),
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
        ph2d_editor_core::ids::INSP_LIVE_WEAPON_SECTION,
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
