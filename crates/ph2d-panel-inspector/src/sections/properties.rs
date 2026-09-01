//! ⭐⭐⭐ **O CARTÃO DE PROPRIEDADES** — *«o que este objecto DIZ que é»*, logo abaixo do cartão de
//! instância.
//!
//! Ver [`ph2d_editor_core::screens::hero::InspectorPropertiesInfo`] para o buraco que ele fecha
//! (report do Enio, 2026-08-31: *«quando mudo o conteúdo entre `{}` o inspector não muda»*).
//!
//! # ⚠️ Uma fileira, duas espécies
//!
//! | quantos valores | como se pinta | porquê |
//! |---|---|---|
//! | `> 1` | chips, o vigente em `Accent` | há para onde ir, e o clique **troca** de versão |
//! | `1` | o valor em texto | *um valor que não leva a lado nenhum não é oferecido* |
//!
//! ⛔ **Pintar o valor único como botão aceso seria um controlo morto** — a espécie que a caça de
//! 2026-08-30 registou 34 vezes: o clique existe, o artista carrega, e nada acontece.

use super::*;
use ph2d_editor_core::screens::hero::InspectorPropertiesInfo;

/// Que fatia da largura o NOME da propriedade leva.
///
/// ⚠️ **Uma fracção com tecto, e não um número fixo**: num painel estreito um rótulo fixo comeria a
/// fileira toda e os chips ficariam ilegíveis; num painel largo um rótulo proporcional afastaria a
/// pergunta das respostas. ⇒ ele cresce até caber `Size`/`State` e pára.
const AXIS_LABEL_FRACTION: f32 = 0.28; // LITERAL-PX-OK: proporção do rótulo, domínio do cartão

/// O tecto do rótulo, em px.
const AXIS_LABEL_MAX_PX: f32 = 72.0; // LITERAL-PX-OK: tecto do rótulo, domínio do cartão

/// A margem de dentro do cartão — irmã da do cartão de instância.
const CARD_PAD: f32 = 8.0; // LITERAL-PX-OK: inset do cartão, irmão do BODY_PAD do corpo

/// ⭐ **O rótulo do modo PLANO** — o nome que a fileira leva quando a família não se declara em
/// eixos.
///
/// ⚠️ **Ele mora AQUI, e não na lei** (HR-15): a lei devolve o nome vazio e o painel nomeia-o,
/// porque é o painel que o portão da HR-15 varre. Pô-lo em `screens/hero/variant_axes.rs` era uma
/// string de UI numa camada fora do alcance do gate — *uma regra fora do caminho de quem executa
/// não existe*. ⏳ Ele migra com os irmãos deste ficheiro quando o Fluent chegar.
const FLAT_AXIS_LABEL: &str = "Variant";

/// O título do cartão — a palavra, sem o nome.
///
/// ⚠️ **Ele existe por causa do caso que motivou o cartão**: num objecto solto não há cartão de
/// instância acima, e duas linhas soltas a dizer `Size  Small` não dizem de onde vêm. *O artista
/// escreveu as chaves no NOME — o cartão tem de o ligar de volta ao que ele escreveu.*
///
/// ⚠️ **Quem lhe acrescenta o nome é o [`card_title`]**, e o nome é o do objecto SELECIONADO
/// (Enio, 2026-08-31) — ver o doc de `InspectorPropertiesInfo::source_name`, que também guarda a
/// decisão anterior e por que ela virou.
const CARD_TITLE: &str = "Properties";

// ⭐⭐⭐ **Os rótulos do gesto de GRAVAR** (Enio, 2026-09-01). ⚠️ Moram aqui pela razão do
// [`FLAT_AXIS_LABEL`]: é o painel que o portão da HR-15 varre, e ⏳ migram juntos quando o Fluent
// chegar.
const SAVE_VARIATION_LABEL: &str = "Save Variation…";
const PROPERTY_LABEL: &str = "Property";
const NEW_PROPERTY_LABEL: &str = "New…";
const NEW_PROPERTY_NAME_LABEL: &str = "Name it";
/// ⚠️ *«Como se chama o que já existe»* — a pergunta sem a qual a fileira nova nasce com um botão
/// em branco, e uma fileira de um valor só nem sequer é oferecida.
const EXISTING_LABEL: &str = "Current is";
const VALUE_LABEL: &str = "This one is";
const SAVE_CONFIRM_LABEL: &str = "Save";
const SAVE_CANCEL_LABEL: &str = "Cancel";
const UPDATE_LABEL: &str = "Update";
/// ⚠️ **O sinal de que o clique CRIA em vez de escolher** — *acender é escolher; `+` é criar*.
const MISSING_SUFFIX: &str = " +";

/// ⭐⭐⭐ **O campo que reescreve o valor vigente**, no rect do chip que ele substitui.
///
/// ⚠️ **Quem SEMEIA é o despacho, não este pintor** — ao contrário do molde do navegador de
/// assets, aqui o valor a semear (`v.label`) está em mãos no sítio do clique, e semear aqui
/// obrigaria a re-semear ou a guardar uma bandeira de «já abri». *O campo nasce cheio e
/// seleccionado, e o pintor só o desenha.*
fn paint_value_field(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    host: Rect,
) {
    paint_field(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        host,
        ids::INSP_INSTANCE_VALUE_EDIT,
    );
}

/// ⭐ **Um campo de texto do cartão** — uma porta só para os quatro que existem.
///
/// ⚠️ Escrever a mesma dúzia de linhas por campo seria quatro sítios a discordar sobre o que é um
/// campo — foi o que o `paint_value_field` fez sozinho até 2026-09-01.
fn paint_field(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    host: Rect,
    id: ph2d_a11y::NodeId,
) {
    let (ti_state, text, caret, anchor) = match store.get(id) {
        Some(InteractiveState::TextInput {
            state,
            text,
            caret,
            selection_anchor,
        }) => (*state, text.clone(), *caret, *selection_anchor),
        _ => (TextInputState::Focused, String::new(), 0, None),
    };
    let input = TextInput::new(id, "").visual((ti_state, store.hover_live(id)));
    paint_text_input_with_buffer(
        &input,
        Some(text.as_str()),
        Some(caret),
        anchor,
        host,
        scene,
        text_system,
        theme,
    );
    hit_index.register(id, host);
}

/// ⭐ **A frase do título** — uma porta, porque a ALTURA e o DESENHO têm de ler a mesma.
///
/// ⚠️ Enquanto ela estava escrita no meio do pintor, medir era impossível sem a repetir — e duas
/// cópias de uma frase são duas frases no dia em que uma mudar.
fn card_title(info: &InspectorPropertiesInfo) -> String {
    match info.source_name.as_deref() {
        Some(n) => format!("{CARD_TITLE} of \u{201c}{n}\u{201d}"),
        None => CARD_TITLE.to_string(),
    }
}

/// Pinta o cartão. Devolve o `y` de baixo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_properties_card(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorPropertiesInfo,
    // ⭐ Qual eixo está a ser reescrito, se algum — ver `InspectorState::value_edit`.
    // ⭐ O valor em reescrita (identidade: entidade + nome do eixo) — ver
    // `InspectorState::value_edit`.
    editing: Option<crate::state::ValueEdit>,
    // ⭐ O formulário de *Salvar Variação…*, se aberto — ver `InspectorState::save_draft`.
    draft: Option<crate::state::SaveDraft>,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let font = TypeToken::Base.px();
    let small = TypeToken::Sm.px();
    let line = font + Spacing::Xs.px();
    // ⚠️ **A altura é MEDIDA antes de pintar** — o fundo vai primeiro, senão cobre o texto.
    // ⛔ E as fileiras contam-se pelo que a TABELA DE IDS endereça, não pelo que o modelo traz: o
    // pintor salta o que passa do teto, e uma altura que contasse o vector inteiro deixaria um vão
    // vazio no fim do cartão.
    let painted = info.rows.len().min(ids::MAX_INSTANCE_AXES);
    let rows = painted + usize::from(info.beyond > 0);
    // ⛔⛔ **O TÍTULO é MEDIDO, não contado** (auditoria de 2026-08-31, achado A2). Ele carrega o
    // nome que o artista escreveu (`Properties of "…"`) e quebra quando não cabe — e a 1.ª fileira
    // de propriedade era pintada por cima da 2.ª linha dele. É o mesmo defeito que o cartão irmão
    // curou duas horas antes: *uma cura escrita num dos dois irmãos deixa o outro a repeti-la.*
    let title = card_title(info);
    let title_h = super::text_h(
        text_system,
        &title,
        small,
        (w - CARD_PAD * 2.0).max(0.0),
        line,
    );
    // ⭐⭐⭐ **O gesto de GRAVAR ocupa linhas, e elas contam-se ANTES de pintar** — a altura do
    // cartão é medida, e uma fileira esquecida aqui é pintada por cima do que vem depois.
    let mine = draft.as_ref().filter(|d| d.entity_bits == info.entity_bits);
    let save_rows = if mine.is_some() {
        // propriedade + nome + [nova: nome da propriedade e como se chama o que existe] + botões
        if mine.is_some_and(|d| d.property.is_none()) {
            5
        } else {
            3
        }
    } else {
        usize::from(info.pending > 0)
    };
    let card_h = CARD_PAD * 2.0 + title_h + line * (rows + save_rows) as f32;
    fill_rounded_rect(
        scene,
        Rect::new(x, y, w, card_h),
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );

    let tx = x + CARD_PAD;
    let tw = (w - CARD_PAD * 2.0).max(0.0);
    let mut ty = y + CARD_PAD;
    paint_text(
        text_system,
        scene,
        &title,
        tx,
        ty,
        small,
        tw,
        resolve(ColorToken::Text2, theme),
    );
    ty += title_h;

    for (a, ax) in info.rows.iter().enumerate() {
        let Some(row_ids) = ids::INSP_INSTANCE_AXIS_OPTION.get(a) else {
            break;
        };
        let label_w = (tw * AXIS_LABEL_FRACTION).min(AXIS_LABEL_MAX_PX);
        let axis_label = if ax.name.is_empty() {
            FLAT_AXIS_LABEL
        } else {
            ax.name.as_str()
        };
        paint_text(
            text_system,
            scene,
            axis_label,
            tx,
            ty + (line - small) * 0.5,
            small,
            label_w,
            resolve(ColorToken::Text2, theme),
        );
        let chips_x = tx + label_w;
        let chips_w = (tw - label_w).max(0.0);
        // ⛔ **UM valor é TEXTO, não um botão** — ver o cabeçalho deste ficheiro.
        if ax.options.len() < 2 {
            if let Some(only) = ax.options.first() {
                paint_text(
                    text_system,
                    scene,
                    &only.label,
                    chips_x,
                    ty + (line - font) * 0.5,
                    font,
                    chips_w,
                    resolve(ColorToken::Text1, theme),
                );
            }
            ty += line;
            continue;
        }
        let n = ax.options.len();
        let gap = Spacing::Xs.px();
        let cw = ((chips_w - gap * (n.saturating_sub(1)) as f32) / n as f32).max(0.0);
        for (i, v) in ax.options.iter().enumerate() {
            let Some(&id) = row_ids.get(i) else {
                break;
            };
            let host = Rect::new(chips_x + (cw + gap) * i as f32, ty, cw, line);
            // ⛔⛔ **O rótulo cabe no chip, ou é CORTADO** (auditoria de 2026-08-31, achado A3).
            //
            // O `MAX_INSTANCE_AXIS_VALUES = 8` é um tecto de **tabela de ids**, e o recurso que se
            // esgota primeiro é outro: a LARGURA. Medido na aritmética do painel — `304 px` de
            // Inspector menos as margens dão `tw ≈ 268`, o rótulo do eixo leva `72`, e oito chips
            // com `4` de intervalo ficam a **≈ 21 px cada** para um texto de `13 px`. O
            // `paint_button` centra com `max_width = rect.w`, logo o rótulo **quebra dentro do
            // botão** e o `y` centrado fica negativo sobre uma fileira de `17 px` ⇒ sangra para as
            // vizinhas. *É o mesmo «labels emboladas» do report, um eixo abaixo.*
            //
            // ⚠️ **Cortar não substitui medir o tecto** — ele continua a ser de ids, e a largura
            // continua a ser o recurso a apertar quando alguém quiser mais de 8. Isto garante só
            // que o cartão nunca desenha por cima de si próprio.
            //
            // ⭐⭐⭐ **E o VIGENTE, em edição, é um CAMPO** — ver o doc de
            // `ids::INSP_INSTANCE_VALUE_EDIT`. Ele ocupa o rect do chip, e é o próprio chip que
            // some: *o valor muda-se onde ele se lê.*
            if v.current
                && editing
                    .as_ref()
                    .is_some_and(|e| e.entity_bits == info.entity_bits && e.axis == ax.name)
            {
                paint_value_field(scene, text_system, theme, hit_index, store, host);
                continue;
            }
            // ⚠️ **Regista-se só o que se PINTA** (auditoria de 2026-08-31, A7): o chip vigente
            // em edição vira campo, e um hit-rect registado sem pintura por baixo é o que toda
            // sonda de «pintado?» lê errado — só não mordia porque o campo, registado depois no
            // mesmo rect, ganhava por ser o de cima.
            hit_index.register(id, host);
            // ⭐⭐⭐ **A combinação que não existe vem com `+`** (plano §2.3-bis): o clique CRIA-a a
            // partir da versão vigente. ⛔ As três alternativas foram pesadas — não fazer nada é o
            // chip morto sob o dedo; aproximar faz o app acender um valor e mostrar outro; recusar
            // manda o artista fazer à mão o que a app sabe fazer. *Esmaecido com `+` é o que impede
            // o gesto de ser uma surpresa.*
            let shown = if v.missing {
                format!("{}{MISSING_SUFFIX}", v.label)
            } else {
                v.label.clone()
            };
            let label = ph2d_editor_core::text_elide::fit(text_system, &shown, font, cw);
            let button = Button::new(id, label)
                // ⚠️ A vigente é `Accent` — é o **estado**, e não uma decoração: sem ela a fileira
                // mostra as opções e esconde a resposta.
                .kind(if v.current {
                    ButtonKind::Accent
                } else {
                    ButtonKind::Default
                })
                .visual(store.button_visual(id));
            paint_button(&button, host, scene, text_system, theme);
        }
        ty += line;
    }

    // ⚠️ **O que a tabela de ids não endereça é ESCRITO** — nunca truncado em silêncio. É a mesma
    // lei do `beyond` da fileira de variants do vetor: *um catálogo que some é um catálogo em que o
    // artista deixa de confiar.*
    if info.beyond > 0 {
        paint_text(
            text_system,
            scene,
            &format!("{} more not shown", info.beyond),
            tx,
            ty,
            small,
            tw,
            resolve(ColorToken::Text2, theme),
        );
        ty += line;
    }
    paint_save(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        info,
        mine,
        tx,
        tw,
        ty,
        line,
        small,
        font,
    );
    y + card_h + SECTION_BOTTOM_PAD_PX
}

/// ⭐⭐⭐ **O gesto de GRAVAR UMA VERSÃO** — o botão, e o formulário que ele abre.
///
/// Enio, 2026-09-01: *«Ao criar e modificar uma instância surge no card um botão do tipo "Salvar
/// Variação"… com o momento de colocar o nome que vai gerar o botão seletor da variação.»*
///
/// # ⚠️ Criar a PRIMEIRA propriedade e criar a SEGUNDA são o MESMO formulário
///
/// As duas precisam das mesmas três respostas — *como se chama a propriedade* · *como se chama o
/// que já existe* · *como se chama isto que acabei de fazer*. ⇒ o selector tem as propriedades da
/// família mais **«New…»**, e escolher «New…» faz crescer os dois campos de cima. *Duas portas para
/// o mesmo gesto divergiriam no dia em que uma delas fosse corrigida.*
#[allow(clippy::too_many_arguments)]
fn paint_save(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    info: &InspectorPropertiesInfo,
    draft: Option<&crate::state::SaveDraft>,
    tx: f32,
    tw: f32,
    mut ty: f32,
    line: f32,
    small: f32,
    font: f32,
) {
    let label_w = (tw * AXIS_LABEL_FRACTION).min(AXIS_LABEL_MAX_PX);
    let Some(draft) = draft else {
        // ⛔ **O botão só existe quando há o que gravar** — sem modificação não há versão a criar.
        if info.pending == 0 {
            return;
        }
        // ⭐⭐⭐ **DOIS botões, e o par é a resposta certa** (plano §2.3, passo 7): sem o *Update*,
        // gravar uma correcção obriga a criar uma versão a mais; sem o *Save*, toda experiência
        // sobrescreve a versão que já existia. ⚠️ O *Update* só aparece quando há uma versão
        // NOMEADA a actualizar.
        let gap = Spacing::Xs.px();
        let (update_w, save_x) = match info.follows.as_deref() {
            Some(_) => (((tw - gap) * 0.5).max(0.0), tx + (tw + gap) * 0.5),
            None => (0.0, tx),
        };
        if let Some(name) = info.follows.as_deref() {
            let host = Rect::new(tx, ty, update_w, line);
            hit_index.register(ids::INSP_INSTANCE_UPDATE_VERSION, host);
            let raw = format!("{UPDATE_LABEL} \u{201c}{name}\u{201d}");
            let b = Button::new(
                ids::INSP_INSTANCE_UPDATE_VERSION,
                ph2d_editor_core::text_elide::fit(text_system, &raw, font, update_w),
            )
            .visual(store.button_visual(ids::INSP_INSTANCE_UPDATE_VERSION));
            paint_button(&b, host, scene, text_system, theme);
        }
        let host = Rect::new(save_x, ty, tw - (save_x - tx), line);
        hit_index.register(ids::INSP_INSTANCE_SAVE_VARIATION, host);
        let b = Button::new(
            ids::INSP_INSTANCE_SAVE_VARIATION,
            ph2d_editor_core::text_elide::fit(text_system, SAVE_VARIATION_LABEL, font, host.w),
        )
        .kind(ButtonKind::Accent)
        .visual(store.button_visual(ids::INSP_INSTANCE_SAVE_VARIATION));
        paint_button(&b, host, scene, text_system, theme);
        return;
    };

    // Fileira 1 — a propriedade: as que a família tem, mais «New…».
    paint_text(
        text_system,
        scene,
        PROPERTY_LABEL,
        tx,
        ty + (line - small) * 0.5,
        small,
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    let chips_x = tx + label_w;
    let chips_w = (tw - label_w).max(0.0);
    let shown = info.declared.len().min(ids::MAX_INSTANCE_AXES);
    let n = shown + 1;
    let gap = Spacing::Xs.px();
    let cw = ((chips_w - gap * (n - 1) as f32) / n as f32).max(0.0);
    for i in 0..n {
        // ⛔⛔ **O «New…» tem id PRÓPRIO, e não a posição seguinte** — apanhado pelo gate de
        // costura em 2026-09-01. Com o id posicional ele mudava de identidade conforme quantas
        // propriedades a família tem, e numa família com quatro passava a partilhar o id da
        // última: *dois controlos diferentes a responder pelo mesmo nome, e o dreno a escolher
        // pelo índice.*
        let (id, raw) = if i < shown {
            (ids::INSP_INSTANCE_SAVE_PROP[i], info.declared[i].as_str())
        } else {
            (
                ids::INSP_INSTANCE_SAVE_PROP[ids::MAX_INSTANCE_AXES],
                NEW_PROPERTY_LABEL,
            )
        };
        let host = Rect::new(chips_x + (cw + gap) * i as f32, ty, cw, line);
        hit_index.register(id, host);
        let current = draft.property == (i < shown).then_some(i);
        let b = Button::new(
            id,
            ph2d_editor_core::text_elide::fit(text_system, raw, font, cw),
        )
        .kind(if current {
            ButtonKind::Accent
        } else {
            ButtonKind::Default
        })
        .visual(store.button_visual(id));
        paint_button(&b, host, scene, text_system, theme);
    }
    ty += line;

    // Fileiras 2 e 3 — só quando a propriedade NASCE agora.
    if draft.property.is_none() {
        for (label, id) in [
            (NEW_PROPERTY_NAME_LABEL, ids::INSP_INSTANCE_SAVE_NEW_PROP),
            (EXISTING_LABEL, ids::INSP_INSTANCE_SAVE_EXISTING),
        ] {
            paint_text(
                text_system,
                scene,
                label,
                tx,
                ty + (line - small) * 0.5,
                small,
                label_w,
                resolve(ColorToken::Text2, theme),
            );
            paint_field(
                scene,
                text_system,
                theme,
                hit_index,
                store,
                Rect::new(chips_x, ty, chips_w, line),
                id,
            );
            ty += line;
        }
    }

    // O NOME desta versão — é ele que vira o botão seletor.
    paint_text(
        text_system,
        scene,
        VALUE_LABEL,
        tx,
        ty + (line - small) * 0.5,
        small,
        label_w,
        resolve(ColorToken::Text2, theme),
    );
    paint_field(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        Rect::new(chips_x, ty, chips_w, line),
        ids::INSP_INSTANCE_SAVE_VALUE,
    );
    ty += line;

    // ⚠️ **Desistir existe de propósito** — um formulário sem saída obriga o artista a gravar algo
    // para se ver livre dele.
    let half = ((tw - gap) * 0.5).max(0.0);
    for (i, (id, label, kind)) in [
        (
            ids::INSP_INSTANCE_SAVE_CONFIRM,
            SAVE_CONFIRM_LABEL,
            ButtonKind::Accent,
        ),
        (
            ids::INSP_INSTANCE_SAVE_CANCEL,
            SAVE_CANCEL_LABEL,
            ButtonKind::Default,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let host = Rect::new(tx + (half + gap) * i as f32, ty, half, line);
        hit_index.register(id, host);
        let b = Button::new(id, label.to_string())
            .kind(kind)
            .visual(store.button_visual(id));
        paint_button(&b, host, scene, text_system, theme);
    }
}
