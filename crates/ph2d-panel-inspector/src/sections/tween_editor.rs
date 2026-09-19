//! ⭐⭐⭐ **O EDITOR do tween aberto** (suplente #22) — os campos daquele que a lista escolheu.
//!
//! ⚠️ **A moldura é o [`super::tween`]** (o cabeçalho, a lista e os dois botões); aqui vive só o que
//! descreve UM tween. *A lista escolhe; o editor mostra* — a frase que o cabeçalho da irmã já
//! escrevia antes de o corte existir.
//!
//! # ⭐⭐ A QUEIXA vem antes dos números
//!
//! | aviso | o que se passa |
//! |---|---|
//! | `no timer at this slot` | ⛔ **o tween não tem relógio** — ele nem chega a correr |
//! | `the timer at this slot has no duration` | ⛔ há relógio e ele **nunca dispara** (duração `0`) |
//! | `this object has no sprite` | o canal escreve num campo do `Sprite`, e não há nenhum |
//! | `from and to are the same` | ele corre e **não move nada** |
//! | `a silhouette that holds stays lit` | ⚠️ funciona, e quase de certeza não é o que se quer |
//!
//! ⚠️ **Os dois primeiros são de outra espécie que os dois últimos:** ali o tween **não corre**,
//! aqui ele corre. *Dizer «ele não move nada» a quem não tem relógio é mandá-lo resolver a metade
//! errada* — a lei da recusa dos pincéis.
//!
//! ⛔⛔ **A ordem NÃO vive aqui**, e é isso que a torna testável: ela é a porta
//! [`InspectorTweenRow::queixa`], e o gate dela corre **sem um device**.
//!
//! # ⭐⭐⭐ Onde mora o TEMPO — a pergunta do dono, em 2026-09-19
//!
//! *«onde selecciono o tempo?»*, no smoke da `PH2D_TWEEN_SMOKE=1`. ⛔ **Um tween é uma função PURA
//! do relógio, logo ele não tem duração nenhuma** — quem a tem é o `Timers[i]` do MESMO índice. O
//! painel sabia-o e **não o dizia**, e a pergunta é a medição disso.
//!
//! ⚠️ **A linha é um READOUT e nunca um segundo campo**, e a razão é o próprio modelo: aquele
//! relógio **não é privado deste tween** (ele publica um sinal, ele arranca uma cutscene, ele conta
//! para uma fábrica), logo editá-lo daqui mudaria em silêncio o que mais estivesse pendurado nele.
//! *Duas superfícies sobre um valor divergem no dia em que uma ganhar clamp* — a lei que os três
//! chips de `Detail` da escultura pagaram.
//!
//! # ⚠️ A curva é onze botões, e por isso são TRÊS fileiras
//!
//! A coluna do Inspector tem ~300 px; onze numa fileira dão ~25 px cada, e um rótulo que não cabe é
//! um chip que o artista não lê. *O corte não é do modelo — as `33` curvas do motor continuam todas
//! alcançáveis —, é da LARGURA.*

use super::tween::{ROW_H, warn};
use super::*;
use ph2d_editor_core::tween_edits::{InspectorTweenRow, TweenQueixa};
use ph2d_i18n::{tr, tr_with};
use ph2d_tween::{AoAcabar, Canal};

/// Quantos chips cabem numa fileira da coluna do Inspector — ver o cabeçalho.
const CHIPS_POR_FILEIRA: usize = 4;

/// **A CHAVE de cada queixa — a PORTA, e não um `match` dentro do pintor.**
///
/// ⛔ Ela traduz o enum da lei numa chave de i18n, e é o único sítio onde as duas coisas se tocam:
/// a lei não conhece a língua e o pintor não decide a ordem.
#[must_use]
const fn chave_da_queixa(q: TweenQueixa) -> &'static str {
    match q {
        TweenQueixa::SemRelogio => "panel.inspector.tween.no_timer_at_this_slot",
        TweenQueixa::RelogioSemDuracao => "panel.inspector.tween.the_timer_here_has_no_duration",
        TweenQueixa::SemSprite => "panel.inspector.tween.this_object_has_no_sprite",
        TweenQueixa::Inerte => "panel.inspector.tween.from_and_to_are_the_same",
        TweenQueixa::SilhuetaQueFica => "panel.inspector.tween.a_silhouette_that_holds_stays_lit",
    }
}

/// **Uma fileira de chips**, de `ids[de..ate]`. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
fn fileira(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
    base: usize,
) -> f32 {
    let gap = Spacing::Xs.px();
    #[allow(clippy::cast_precision_loss)]
    let n = ids_.len().max(1) as f32;
    let cw = ((w - gap * (n - 1.0)) / n).max(0.0);
    for (i, (&id, text)) in ids_.iter().zip(labels.iter()).enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let rect = Rect::new(x + (cw + gap) * i as f32, y, cw, ROW_H);
        hit_index.register(id, rect);
        // ⚠️ **A selecção vem do SNAPSHOT**, nunca do store: o store guarda o visual do botão, e
        // ler dali qual está aceso faria o realce sobreviver à troca de objecto.
        let kind = if base + i == sel {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        };
        paint_button(
            &Button::new(id, *text)
                .kind(kind)
                .visual(store.button_visual(id)),
            rect,
            scene,
            text_system,
            theme,
        );
    }
    // ⚠️ **O passo de uma linha vem da PORTA** — há gate contra a segunda resposta.
    y + ph2d_tokens::row_pitch_px()
}

/// **Um grupo de chips com rótulo**, partido em fileiras de [`CHIPS_POR_FILEIRA`].
#[allow(clippy::too_many_arguments)]
fn grupo(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    label: &str,
    ids_: &[NodeId],
    labels: &[&str],
    sel: usize,
) -> f32 {
    let font = TypeToken::Sm.px();
    paint_text(
        text_system,
        scene,
        label,
        x,
        y,
        font,
        w,
        resolve(ColorToken::Text2, theme),
    );
    let mut cur_y = y + font + Spacing::Xs.px();
    for (bloco, chunk) in ids_.chunks(CHIPS_POR_FILEIRA).enumerate() {
        let base = bloco * CHIPS_POR_FILEIRA;
        cur_y = fileira(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            chunk,
            &labels[base..base + chunk.len()],
            sel,
            base,
        );
    }
    // ⚠️ **A cauda de um bloco vem da PORTA**, e a última fileira já trouxe o passo dela.
    cur_y + ph2d_tokens::control_gap_px()
}

/// ⭐⭐⭐ **O bloco do RELÓGIO** — a duração e os dois interruptores, dentro da secção do tween.
/// Devolve o `y` seguinte.
///
/// ⛔⛔ **Eles escrevem no MESMO `Timers[i]` que a secção TIMERS, pela MESMA porta.** Não é uma
/// segunda superfície sobre um valor (a armadilha que os três chips de `Detail` da escultura
/// pagaram): é a porta com **dois chamadores**, como o teclado e o menu do `project_io`. Quem
/// clampa, quem satura e quem recusa continua a ser um só.
///
/// ⚠️ **A legenda NOMEIA o timer** porque ele não é privado deste tween — ele pode estar a arrancar
/// uma cutscene, a alimentar uma fábrica ou a publicar um sinal, e o artista tem de saber que é o
/// mesmo objecto que vê na outra secção.
#[allow(clippy::too_many_arguments)]
fn relogio(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTweenRow,
    slot: usize,
) -> f32 {
    let mut cur_y = warn(
        scene,
        text_system,
        theme,
        x,
        w,
        y,
        &tr_with(
            "panel.inspector.tween.clock_is_timer",
            &[("n", &(slot + 1))],
        ),
        ColorToken::Text3,
    );
    let sec = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        1,
        &[tr("panel.inspector.tween.duration_seconds")],
    );
    cur_y = super::rows::fields_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.duration_seconds"),
        &[crate::ids::INSP_TWEEN_DURACAO],
        0.1, // LITERAL-PX-OK: passo de scrub em SEGUNDOS, como o da secção TIMERS
        None,
        sec,
    );
    // ⚠️ **O valor das caixas vem do SNAPSHOT**, nunca do store: ler dali faria a caixa sobreviver
    // à troca de objecto — a lei que a §11 escreveu para o `Playing`.
    ph2d_editor_core::property_row::paint_check_rows(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        &[
            (
                crate::ids::INSP_TWEEN_REPEAT,
                tr("panel.inspector.tween.repeat"),
                row.repeat,
            ),
            (
                crate::ids::INSP_TWEEN_AUTOSTART,
                tr("panel.inspector.tween.autostart"),
                row.autostart,
            ),
        ],
        sec,
    )
}

/// O editor do tween aberto. Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(super) fn editor(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &InspectorTweenRow,
    tem_sprite: bool,
    slot: usize,
) -> f32 {
    let mut cur_y = y;
    // ⚠️ **A QUEIXA primeiro** — quem não vê nada mexer não quer afinar uma curva.
    //
    // ⚠️ **O `n` vai SEMPRE**, e o [`ph2d_i18n::tr_with`] ignora o que a frase não pede: as quatro
    // queixas que não nomeiam o relógio ficam byte a byte como eram, e a quinta (*«o timer N não tem
    // duração»*) diz a QUAL ir. *Um argumento a mais é grátis; um `match` a decidir quem o recebe
    // seria a segunda tabela sobre a mesma pergunta.*
    if let Some(q) = row.queixa(tem_sprite) {
        cur_y = warn(
            scene,
            text_system,
            theme,
            x,
            w,
            cur_y,
            &tr_with(chave_da_queixa(q), &[("n", &(slot + 1))]),
            ColorToken::Text3,
        );
    }
    // ⭐⭐⭐ **Os PRESETS primeiro** — eles reescrevem tudo o que vem a seguir, e é isso que os põe
    // em cima: *um botão que muda os cinco campos abaixo dele lê-se; um que os muda acima, não.*
    let presets: Vec<&str> = ph2d_tween::Preset::ALL.iter().map(|p| p.label()).collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.preset"),
        &crate::ids::INSP_TWEEN_PRESET,
        &presets,
        // ⚠️ **NENHUM fica aceso**, e é a decisão: um preset não é um MODO — depois do clique ele
        // desaparece e sobram os cinco campos. Acender um deles prometeria um estado que o
        // componente não guarda, e ele mentiria no instante em que o artista afinasse um número.
        usize::MAX,
    );
    // ⭐⭐⭐ **O RELÓGIO, AQUI** — ver o cabeçalho deste ficheiro.
    //
    // ⚠️ **Vem DEPOIS dos presets de propósito:** eles reescrevem a duração, e vê-la mudar debaixo
    // do botão é o que prova ao artista que UM clique fez as duas coisas. ⛔ E o bloco inteiro
    // desaparece quando não há relógio, porque aí quem fala é a queixa — *duas superfícies sobre a
    // mesma ausência ensinam que são dois problemas*.
    if row.duracao_us.is_some() {
        cur_y = relogio(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            x,
            w,
            cur_y,
            row,
            slot,
        );
    }
    let canal = Canal::from_tag(row.canal);
    let canais: Vec<&str> = Canal::ALL.iter().map(|c| c.label()).collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.channel"),
        &crate::ids::INSP_TWEEN_CANAL,
        &canais,
        canal.tag() as usize,
    );

    // ⭐ **Uma componente ou quatro — DERIVADO do canal**, nunca uma segunda lista.
    let n = canal.aridade();
    let seccao = ph2d_editor_core::property_row::Seccao::medida(
        text_system,
        n,
        &[
            tr("panel.inspector.tween.from"),
            tr("panel.inspector.tween.to"),
        ],
    );
    for (label, ids_) in [
        (tr("panel.inspector.tween.from"), &crate::ids::INSP_TWEEN_DE),
        (tr("panel.inspector.tween.to"), &crate::ids::INSP_TWEEN_PARA),
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
            &ids_[..n],
            0.05, // LITERAL-PX-OK: passo de arrasto — adimensional numa cor, metros numa pose
            None,
            seccao,
        );
    }

    let familias: Vec<&str> = ph2d_anim::EasingFamily::ALL
        .iter()
        .map(|f| f.label())
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.curve"),
        &crate::ids::INSP_TWEEN_FAMILIA,
        &familias,
        row.familia as usize,
    );
    let modos: Vec<&str> = ph2d_anim::EasingMode::ALL
        .iter()
        .map(|m| m.label())
        .collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.ease"),
        &crate::ids::INSP_TWEEN_MODO,
        &modos,
        row.modo as usize,
    );
    let fins: Vec<&str> = AoAcabar::ALL.iter().map(|a| a.label()).collect();
    cur_y = grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.when_done"),
        &crate::ids::INSP_TWEEN_AO_ACABAR,
        &fins,
        row.ao_acabar as usize,
    );
    // ⭐⭐⭐ **O CICLO — o *ping-pong* que o dono pediu em 2026-09-19.**
    //
    // ⚠️ **Ele vem DEPOIS do `When Done` de propósito:** os dois falam do tempo, e a ordem é a da
    // pergunta que o artista faz — *o que acontece DENTRO de uma volta* lê-se depois de *o que
    // acontece no FIM*, porque é o fim que ele já conhece do resto do painel.
    let ciclos: Vec<&str> = ph2d_tween::Ciclo::ALL.iter().map(|c| c.label()).collect();
    grupo(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        x,
        w,
        cur_y,
        tr("panel.inspector.tween.cycle"),
        &crate::ids::INSP_TWEEN_CICLO,
        &ciclos,
        row.ciclo as usize,
    )
}
