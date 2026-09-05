// ph2d-chrome-sync:z=181 (dispatch priority, ADR-0107; lower = earlier)
//! **A JANELA DO INPUT MAP** — a janela flutuante que abre sobre o canvas, equivalente à do Godot
//! (plano `docs/Vector Module/30_plano_input_map.md` §0.2).
//!
//! Enio, 2026-08-24: *"precisamos do input Map completo não apenas para o jogador mas para qualquer
//! objeto do game via UI. (…) equivalente ao da godot com janela flutuante abrindo sobre o canvas"*.
//!
//! **Três** metades, colocadas — o padrão do [`super::fill_modal`], mais uma que a auditoria de
//! 2026-08-24 obrigou a existir:
//!
//! * [`paint_input_map_window`] — desenha o cartão em `store.input_map_pos()` (preso à viewport):
//!   a faixa de título arrastável, **o campo de nome + Add logo por baixo dela** (é o topo, como
//!   no Godot), e depois uma linha por acção com as ligações dela indentadas;
//! * [`apply`] — a metade de despacho, ligada ao `chrome::dispatch_all` pelo `ph2d-chrome-sync`;
//! * [`layout`] — **onde as coisas ficam**: a sequência das linhas ([`layout::BodyLine`]), as
//!   larguras, e o que a faixa do título diz. ⚠️ Ela nasceu porque a altura e o desenho eram
//!   **duas contas da mesma coisa** e divergiram — os dois reports com foto do Enio.
//!
//! # ⭐ Ele MUTA o `hero.input_map` directamente, e não pelo barramento
//!
//! O mapa é dado de **projecto** e o dono dele é o [`HeroScreen`] — o mesmo precedente das
//! `ProjectSettings`. Um `EditorAction` por gesto (criar · apagar · ligar · desligar) seria uma
//! variante por verbo para atravessar uma fronteira que **não existe**: o handler já tem
//! `&mut HeroScreen`.
//!
//! # ⛔ A escuta é um MODO, e é ela que torna o gesto possível
//!
//! `Bind…` arma [`WidgetStore::listen_for_binding`]. Enquanto isso dura, **a próxima tecla é
//! conteúdo, não atalho** — e quem tem de perguntar primeiro é o despacho de teclado. Sem essa
//! ordem, ligar `S` a uma acção **salva o projecto** e a ligação nunca acontece.

use crate::icons::IconId;
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve};
use crate::widget::{
    Button, IconButtonStyle, IconGlyph, ListItem, ListItemState, TextInput, TextInputState,
    paint_button, paint_icon_button, paint_list_item, paint_scrollbar,
    paint_slider_with_chip_layout, paint_text_input_with_buffer, slider_with_chip_min_w,
};
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{Binding, InputMap};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Radius, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// **A descrição legível de uma ligação** — a única porta: `(o que é, de que dispositivo)`.
///
/// ⚠️ Ela existe porque o pintor e o gate precisam da **mesma** frase: um texto construído no
/// pintor tornaria o gate uma segunda implementação, e as duas divergiriam no dia em que uma
/// variante nova entrasse no `Binding`.
///
/// ⛔⛔ **A primeira versão devolvia `Key 0xF702`**, e o Enio respondeu o óbvio: *"Quem vai usar são
/// artistas e não IA"*. Um código hexadecimal é a representação interna a vazar para a cara do
/// produto. Agora devolve **o nome e o dispositivo** — `("Left Arrow", "Keyboard")` —, e o
/// dispositivo vai na coluna secundária do `ListItem`, que é onde a casa põe texto de apoio.
#[must_use]
pub fn binding_label(b: Binding) -> (String, &'static str) {
    match b {
        Binding::Key(k) => (k.label(), tr("input_map.binding.key")),
        Binding::PadButton(p) => (p.label().to_string(), tr("input_map.binding.pad")),
        // ⚠️ A metade da haste diz-se com uma SETA e não com `+`/`-`: o artista empurrou para um
        // lado, e é o lado que ele reconhece.
        Binding::PadAxis { axis, positive } => (
            format!("{} {}", axis.label(), if positive { "(+)" } else { "(-)" }),
            tr("input_map.binding.axis"),
        ),
    }
}

/// **REGISTA os widgets de TODA linha do mapa** — a porta única, chamada onde a lista pode mudar.
///
/// ⚠️ **Uma lista DINÂMICA não cabe no `pre_populate`**, que é onde o resto do chrome regista: ali
/// o número de linhas é conhecido de véspera (o painel de tokens tem uma por `ColorToken`). Aqui
/// ele muda a cada acção criada ou apagada — e um widget pintado sem estar registrado é **morto sob
/// o ponteiro**, que é o defeito que esta linha já pagou duas vezes em 23/08 (os quatro chips da
/// booleana) e uma terceira nesta mesma wave (o campo de nome).
///
/// ⚠️ **Re-regista TODAS as linhas, e não só a que mudou.** É idempotente e custa uma dezena de
/// escritas; a alternativa — lembrar de chamar a coisa certa em cada um dos caminhos de mutação —
/// é a que se esquece no caminho seguinte que alguém acrescentar.
pub fn sync_input_map_rows(store: &mut WidgetStore, map: &InputMap) {
    for (row, a) in map.actions().iter().enumerate() {
        for id in [
            ids::input_map_delete_action_id(row),
            ids::input_map_listen_id(row),
        ] {
            store.register(
                id,
                crate::interaction::InteractiveState::Button {
                    state: crate::widget::ButtonState::Normal,
                },
            );
        }
        for bi in 0..a.bindings.len() {
            store.register(
                ids::input_map_delete_binding_id(row, bi),
                crate::interaction::InteractiveState::Button {
                    state: crate::widget::ButtonState::Normal,
                },
            );
        }
        // ⚠️ Os dois números da zona, **semeados do valor autorado** a cada sincronia: o
        // `set_zone` COAGE (`press_point >= dead_zone`), então arrastar um pode mover o outro — e
        // o slider tem de mostrar o valor que ficou, não o que o dedo pediu.
        // ⚠️ **O CHIP é um `NumberInput`, e não um `Button`.** Auditoria 2026-08-24: registá-lo
        // como botão fazia-o **pintar as setinhas e não aceitar dígito nenhum** — o número era
        // decoração. É a irmã exacta da cicatriz da swatch do painel de tokens (*"registá-la como
        // botão faria o clique acender o widget e nunca abrir o picker"*).
        for (id, v) in [
            (ids::input_map_deadzone_chip_id(row), a.dead_zone),
            (ids::input_map_press_point_chip_id(row), a.press_point),
        ] {
            store.register(
                id,
                crate::interaction::InteractiveState::NumberInput {
                    state: crate::widget::TextInputState::Normal,
                    value: f64::from(v),
                    // ⚠️ O buffer é o valor JÁ FORMATADO: o dispatch nunca faz crescer a `String`
                    // em construção, e é por isso que ele nasce preenchido aqui.
                    buffer: format!("{v:.3}"),
                    caret: 0,
                    last_committed: f64::from(v),
                    selection_anchor: None,
                },
            );
        }
        for (id, v) in [
            (ids::input_map_deadzone_id(row), a.dead_zone),
            (ids::input_map_press_point_id(row), a.press_point),
        ] {
            store.register(
                id,
                crate::interaction::InteractiveState::Slider {
                    state: crate::widget::SliderState::Normal,
                    value: v,
                    orientation: crate::widget::SliderOrientation::Horizontal,
                },
            );
            if let Some(crate::interaction::InteractiveState::Slider { value, .. }) =
                store.get_mut(id)
            {
                *value = v;
            }
        }
    }
}

/// **Desenha a janela** (no-op quando fechada).
///
/// ⭐ **A forma é a do Godot**, e a decisão que mais encurta a lista é esta: os **dois números
/// vivem na LINHA DA ACÇÃO**, à direita do nome — não numa linha própria. Era a linha extra por
/// acção que fazia seis acções ocuparem vinte e quatro linhas.
///
/// ⚠️ E o **`+`** que arma a escuta também está lá, no lugar do antigo botão `Bind…`: um botão de
/// texto numa linha só dele custava mais uma linha por acção e dizia menos que um ícone.
pub fn paint_input_map_window(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    map: &InputMap,
    viewport: Rect,
) {
    let Some((x, y)) = store.input_map_pos() else {
        return;
    };
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let pad_y = Spacing::Sm.px();
    let window_w = window_w();

    // ── A altura vem da PORTA, e não de uma segunda conta. ──
    //
    // ⛔ Report do Enio: *"sem scroll"*. Um cartão que cresce com a lista sai do ecrã e a última
    // acção fica inalcançável — que é pior do que uma lista curta, porque nada na tela o diz.
    //
    // ⛔⛔ **E as duas contas DIVERGIAM em 48 px**: a [`input_map_window_size`] (que a shell usa
    // para a roda e o arrasto) clampava à viewport inteira, e aqui subtraía-se mais um `Xl4` de
    // margem. O doc dela dizia *"a mesma conta que o pintor faz"* — e não era. *Uma função que se
    // diz a porta única só o é quando o outro lado a CHAMA.*
    let chrome_h = layout::chrome_h();
    let (_, total_h, max_scroll) = input_map_window_size(map, viewport.h);
    let body_h = (total_h - chrome_h).max(row_h);
    let want_body = body_h + max_scroll;

    let max_x = (viewport.x + viewport.w - window_w).max(viewport.x);
    let max_y = (viewport.y + viewport.h - total_h).max(viewport.y);
    let rect_x = x.clamp(viewport.x, max_x); // CLAMP-OK: bounds ordered + non-NaN
    let rect_y = y.clamp(viewport.y, max_y); // CLAMP-OK: bounds ordered + non-NaN
    let rect = Rect::new(rect_x, rect_y, window_w, total_h);

    // ⭐ Raio e moldura pela porta do TEMA: a janela flutuante é plana num tema moderno.
    let radius = crate::paint::frame_radius(theme, Radius::Md.px());
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    crate::paint::stroke_frame(
        scene,
        rect,
        radius,
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    // ⛔⛔ **O FUNDO DO CARTÃO ABSORVE O CLIQUE** — auditoria de 2026-08-24, o achado mais grave.
    //
    // Sem ele, clicar no espaço vazio ENTRE dois controlos da janela caía no canvas por baixo: com
    // o pincel na mão, o artista **pintava** enquanto arrumava os controlos. Um cartão flutuante
    // que deixa passar o que não consome não é uma janela — é um desenho.
    //
    // ⚠️ É registado ANTES de tudo o resto de propósito: o `HitIndex` responde com o ÚLTIMO
    // rectângulo que cobre o ponto, então o fundo tem de entrar primeiro para os controlos ficarem
    // por cima dele.
    hit_index.register(ids::INPUT_MAP_SURFACE, rect);

    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let mut cy = rect.y + pad_y;

    // ── Faixa de título = alça de arrasto, com o X à direita. ──
    //
    // ⭐⭐ **COM A ESCUTA ARMADA, a faixa DIZ-O — e diz de QUEM.**
    //
    // ⛔ A versão anterior punha esse aviso **dentro do corpo**, ao lado do nome da acção — e o
    // corpo **rola e é recortado**: bastava a acção armada sair da janela para o único sinal de
    // que o app estava à espera de uma tecla desaparecer. A faixa do título não rola nunca.
    //
    // ⚠️ E ele **SUBSTITUI** o título em vez de o acompanhar: `Input Map` é decoração (a janela
    // está à frente do artista), a escuta é urgente — e dois textos a partilhar uma faixa é
    // exactamente a colisão que o report de 24/08 fotografou.
    let listening = store.input_map_listening();
    let handle_rect = Rect::new(rect.x, rect.y, rect.w, pad_y + row_h);
    hit_index.register(ids::INPUT_MAP_HANDLE, handle_rect);
    let (title, title_tok) = match layout::title_text(map, listening) {
        Some(t) => (t, ColorToken::Accent),
        None => (tr("input_map.title").to_string(), ColorToken::Text1),
    };
    let close_rect = Rect::new(rect.x + rect.w - Spacing::Sm.px() - row_h, cy, row_h, row_h);
    paint_text(
        text_system,
        scene,
        &title,
        inner_x,
        cy + (row_h - font) * 0.5,
        font,
        // ⚠️ A largura PÁRA no botão de fechar: sem isso um nome de acção longo desenhava-se por
        // baixo do X, que é a mesma doença um nível acima.
        (close_rect.x - inner_x - Spacing::Sm.px()).max(0.0),
        resolve(title_tok, theme),
    );
    hit_index.register(ids::INPUT_MAP_CLOSE, close_rect);
    paint_icon_button(
        close_rect,
        IconGlyph::Builtin(IconId::Close),
        IconButtonStyle::Plain,
        store.button_visual(ids::INPUT_MAP_CLOSE),
        scene,
        theme,
    );
    cy += row_h + gap;

    // ── O CAMPO DO NOME + **Add** — ⭐ **EM CIMA**. ──
    //
    // ⛔⛔ **A nota anterior dizia *"em baixo, como no Godot"* e estava ERRADA sobre a
    // referência**: o *Input Map* do Godot põe o campo e o botão **no topo** do painel. Enio,
    // 2026-08-24: *"a caixa de Action name fica em cima e não embaixo do painel"*.
    //
    // ⛔⛔ **E o campo era DESENHADO À MÃO** — um rectângulo com borda e um texto por cima. Report
    // do mesmo dia: *"A caixa de texto parece morta, não se vê que o foco está nela ao clicar."*
    // Estava certo, e a causa não era o despacho: o `TextInputState::Focused` **já** era escrito
    // no `pointer_down`, e não havia **quem o lesse**. Agora é o widget da casa que pinta —
    // preenchimento, anel de foco `Accent` de 2 px, cursor, selecção e hover, de graça.
    // *A costura estava toda feita menos o último elo: quem PINTA.*
    let add_w = Spacing::Xl4.px() * 1.5; // LITERAL-PX-OK: multiplicador do token, nao um px
    let name_rect = Rect::new(inner_x, cy, inner_w - add_w - gap, row_h);
    hit_index.register(ids::INPUT_MAP_NEW_NAME, name_rect);
    let (typed, caret, anchor, ti_state) = match store.get(ids::INPUT_MAP_NEW_NAME) {
        Some(crate::interaction::InteractiveState::TextInput {
            text,
            caret,
            selection_anchor,
            state,
        }) => (text.as_str(), *caret, *selection_anchor, *state),
        _ => ("", 0, None, TextInputState::Normal),
    };
    paint_text_input_with_buffer(
        &TextInput::new(ids::INPUT_MAP_NEW_NAME, "")
            .placeholder(tr("input_map.new_name.placeholder"))
            .visual((ti_state, store.hover_live(ids::INPUT_MAP_NEW_NAME))),
        Some(typed),
        Some(caret),
        anchor,
        name_rect,
        scene,
        text_system,
        theme,
    );
    let add_rect = Rect::new(inner_x + inner_w - add_w, cy, add_w, row_h);
    hit_index.register(ids::INPUT_MAP_ADD, add_rect);
    // ⚠️ **`accent` + `visual`**: ele é a acção principal desta janela e estava a nascer cinzento
    // e **inerte sob o rato** — um botão que não acende lê-se como desligado.
    paint_button(
        &Button::new(ids::INPUT_MAP_ADD, tr("input_map.add"))
            .accent()
            .visual(store.button_visual(ids::INPUT_MAP_ADD)),
        add_rect,
        scene,
        text_system,
        theme,
    );
    cy += row_h + gap;

    // ── O CORPO: recortado no CLIQUE **e** na PINTURA. ──
    //
    // ⚠️ **São duas coisas diferentes, e o comentário anterior confundia-as.** O `push_clip` do
    // `HitIndex` só decide quem RESPONDE; a auditoria de 2026-08-24 mostrou que o conteúdo rolado
    // continuava a **DESENHAR** por cima do título e para fora do cartão. Quem recorta pixels é a
    // cena — [`VectorScene::push_layer`].
    let body = Rect::new(rect.x, cy, rect.w, body_h);
    hit_index.push_clip(body);
    scene.push_clip(&ph2d_vector::Rect::new(
        f64::from(body.x),
        f64::from(body.y),
        f64::from(body.x + body.w),
        f64::from(body.y + body.h),
    ));
    let scroll = store.input_map_scroll();
    let icon_w = Spacing::Xl2.px();
    // ⭐ **UMA lista, e o `y` de cada linha É o índice dela.** Enquanto o desenho re-derivava a
    // sequência à mão, um texto pintado fora de ordem caía por cima do vizinho — que é o
    // *"labels emboladas"* do report. Ver [`BodyLine`].
    for (i, line) in body_lines(map, listening).into_iter().enumerate() {
        #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: indice de linha cabe em f32
        let by = cy - scroll + (row_h + gap) * i as f32;
        let row = match line {
            BodyLine::NoActions => {
                paint_text(
                    text_system,
                    scene,
                    tr("input_map.empty"),
                    inner_x,
                    by + (row_h - font) * 0.5,
                    font,
                    inner_w,
                    resolve(ColorToken::Text3, theme),
                );
                continue;
            }
            // ⚠️ **UM texto por linha.** Com a escuta armada é a INSTRUÇÃO que aparece aqui, e não
            // o convite a carregar no `+` — que o artista acabou de carregar. Os dois ao mesmo
            // tempo, no mesmo `y`, era literalmente a foto do report de 24/08.
            BodyLine::Empty { armed, .. } => {
                let (msg, tok) = if armed {
                    (tr("input_map.listening"), ColorToken::Accent)
                } else {
                    (tr("input_map.no_binding"), ColorToken::Text3)
                };
                paint_text(
                    text_system,
                    scene,
                    msg,
                    inner_x + Spacing::Xl.px(),
                    by + (row_h - font) * 0.5,
                    font,
                    inner_w - Spacing::Xl.px(),
                    resolve(tok, theme),
                );
                continue;
            }
            BodyLine::Binding { row, bi } => {
                let Some(b) = map
                    .actions()
                    .get(row)
                    .and_then(|a| a.bindings.get(bi).copied())
                else {
                    continue;
                };
                paint_binding_row(
                    b,
                    ids::input_map_delete_binding_id(row, bi),
                    Rect::new(
                        inner_x + Spacing::Xl.px(),
                        by,
                        inner_w - Spacing::Xl.px() - icon_w,
                        row_h,
                    ),
                    Rect::new(rect.x + rect.w - Spacing::Sm.px() - row_h, by, row_h, row_h),
                    store,
                    hit_index,
                    scene,
                    text_system,
                    theme,
                );
                continue;
            }
            BodyLine::Action { row } => row,
        };
        let Some(action) = map.actions().get(row) else {
            continue;
        };
        // ═══ A LINHA DA ACÇÃO: nome · Dead · Press · `+` · lixo ═══
        let armed = listening == Some(action.id);
        let del = ids::input_map_delete_action_id(row);
        let listen = ids::input_map_listen_id(row);
        // ⚠️ **A largura da coluna é a do BOTÃO** (`row_h`), e não um `Spacing` parecido: com
        // `icon_w = Xl2 = 24` e o botão a medir `row_h = 28`, os dois rectângulos sobrepunham-se
        // **4 px** — e como o lixo é registado DEPOIS, ele ganhava o ponto: 14% do botão de armar
        // a escuta **apagava a acção**, sem volta. (Auditoria 2026-08-24.)
        let del_rect = Rect::new(rect.x + rect.w - Spacing::Sm.px() - row_h, by, row_h, row_h);
        let listen_rect = Rect::new(del_rect.x - row_h - Spacing::Xs.px(), by, row_h, row_h);

        paint_text(
            text_system,
            scene,
            &action.name,
            inner_x,
            by + (row_h - font) * 0.5,
            font,
            inner_w,
            resolve(ColorToken::Text1, theme),
        );

        // Os dois números, numa linha só e SEM empilhar — a largura da janela sai desta conta.
        let (lw, cw) = (zone_label_w(), zone_chip_w());
        // ⚠️ **O trilho mínimo vem do WIDGET, não de um palpite** — `slider_with_chip_is_stacked`
        // é a mesma função que o `debug_assert` abaixo consulta, e a folga de `Xs` é o que separa
        // "cabe exactamente" de "empilha por um pixel".
        // ⚠️ A largura da zona sai do PISO DO WIDGET, perguntado — não de um espelho local.
        let zone_w = (lw + slider_with_chip_min_w(cw) + Spacing::Xs.px()).ceil();
        let zone_x = listen_rect.x - zone_w * 2.0 - Spacing::Sm.px();
        for (i, (sid, cid, label, v)) in [
            (
                ids::input_map_deadzone_id(row),
                ids::input_map_deadzone_chip_id(row),
                tr("input_map.dead_zone"),
                action.dead_zone,
            ),
            (
                ids::input_map_press_point_id(row),
                ids::input_map_press_point_chip_id(row),
                tr("input_map.press_point"),
                action.press_point,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: indice 0/1
            let zx = zone_x + zone_w * i as f32;
            let zr = Rect::new(zx, by, zone_w - Spacing::Xs.px(), row_h);
            // ⚠️ **Esta asserção MUDOU de instrumento em 2026-09-02, e a pergunta é a mesma.**
            // Ela perguntava *«a janela é larga que chegue?»* pelo proxy do EMPILHAMENTO, e o proxy
            // morreu com a caixa única (o rótulo passou a viver dentro, então não há coluna externa
            // que deixe de caber) — `slider_with_chip_is_stacked` devolve `false` sempre, e mantê-la
            // aqui teria deixado **uma asserção vácua a parecer protecção viva**.
            // ⇒ o piso novo é o do widget: a coluna do valor mais uma folga de cada lado.
            debug_assert!(
                zr.w >= slider_with_chip_min_w(cw),
                "a janela ficou estreita: a linha do numero nao tem largura para o valor + folgas \
                 (report de 24/08, re-instrumentado em 02/09)"
            );
            paint_slider_with_chip_layout(
                zr,
                label,
                v,
                f64::from(v),
                None,
                sid,
                cid,
                lw,
                cw,
                store,
                hit_index,
                scene,
                text_system,
                theme,
            );
        }

        hit_index.register(listen, listen_rect);
        paint_icon_button(
            listen_rect,
            IconGlyph::Builtin(IconId::Add),
            if armed {
                IconButtonStyle::Chip
            } else {
                IconButtonStyle::Plain
            },
            store.button_visual(listen),
            scene,
            theme,
        );
        hit_index.register(del, del_rect);
        paint_icon_button(
            del_rect,
            IconGlyph::Builtin(IconId::Trash),
            IconButtonStyle::Plain,
            store.button_visual(del),
            scene,
            theme,
        );
    }
    hit_index.pop_clip();
    scene.pop_layer();

    // ── A BARRA DE ROLAGEM, quando a lista não cabe — e ela é REGISTADA. ──
    //
    // ⛔ Auditoria 2026-08-24: ela era pintada e **nunca registada** — não arrastava, não fazia
    // hover, e não tinha id nenhum. *Uma barra que não se pode agarrar é um enfeite que promete um
    // gesto.* O `visual` também passa a vir do store, senão ela nasce inerte sob o rato.
    if want_body > body_h {
        hit_index.register(
            crate::widget::INPUT_MAP_SCROLLBAR_ID,
            crate::widget::scrollbar_track_rect(body),
        );
        paint_scrollbar(
            body,
            scroll,
            want_body,
            body_h,
            store.scrollbar_visual(crate::widget::INPUT_MAP_SCROLLBAR_ID),
            scene,
            theme,
        );
    }
}

/// **UMA LIGAÇÃO, indentada** — o que ela é, de que dispositivo, e o `X` que a remove.
///
/// ⚠️ Sai do laço porque o corpo dele passou a ser um `match` sobre [`BodyLine`], e um braço com
/// trinta linhas esconde os outros três.
#[allow(clippy::too_many_arguments)]
fn paint_binding_row(
    b: Binding,
    db: ph2d_a11y::NodeId,
    li_rect: Rect,
    db_rect: Rect,
    store: &WidgetStore,
    hit_index: &mut HitIndex,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let (what, device) = binding_label(b);
    paint_list_item(
        &ListItem::new(db, what)
            .value(device)
            .state(list_state(store, db)),
        li_rect,
        scene,
        text_system,
        theme,
    );
    hit_index.register(db, db_rect);
    paint_icon_button(
        db_rect,
        IconGlyph::Builtin(IconId::Close),
        IconButtonStyle::Plain,
        store.button_visual(db),
        scene,
        theme,
    );
}

/// O estado visual de uma linha da lista — hover incluído.
///
/// ⚠️ **Vem do `WidgetStore`, e não de um `state` local**: um `ListItem` construído com o estado
/// por omissão nasce **inerte sob o rato**, e o artista lê isso como "esta linha não faz nada".
fn list_state(store: &WidgetStore, id: ph2d_a11y::NodeId) -> ListItemState {
    match store.get(id) {
        Some(crate::interaction::InteractiveState::Button {
            state: crate::widget::ButtonState::Hovered,
        }) => ListItemState::Hovered,
        Some(crate::interaction::InteractiveState::Button {
            state: crate::widget::ButtonState::Pressed,
        }) => ListItemState::Pressed,
        _ => ListItemState::Normal,
    }
}

/// **A metade de DESPACHO** — irmã por teto de LOC; ver [`apply`].
mod apply;
pub use apply::apply;

/// **ONDE AS COISAS FICAM** — a terceira irmã; ver [`layout`].
mod layout;
pub use layout::input_map_window_size;
use layout::{BodyLine, body_lines, window_w, zone_chip_w, zone_label_w};
