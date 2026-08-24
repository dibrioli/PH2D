// ph2d-chrome-sync:z=181 (dispatch priority, ADR-0107; lower = earlier)
//! **A JANELA DO INPUT MAP** — a janela flutuante que abre sobre o canvas, equivalente à do Godot
//! (plano `docs/Vector Module/30_plano_input_map.md` §0.2).
//!
//! Enio, 2026-08-24: *"precisamos do input Map completo não apenas para o jogador mas para qualquer
//! objeto do game via UI. (…) equivalente ao da godot com janela flutuante abrindo sobre o canvas"*.
//!
//! Duas metades, colocadas — o padrão do [`super::fill_modal`]:
//!
//! * [`paint_input_map_window`] — desenha o cartão em `store.input_map_pos()` (preso à viewport),
//!   com a faixa de título arrastável, o campo de nome, o **Add**, e uma linha por acção com as
//!   ligações dela por baixo;
//! * [`apply`] — a metade de despacho, ligada ao `chrome::dispatch_all` pelo `ph2d-chrome-sync`.
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

use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{fill_rounded_rect, paint_text, resolve, stroke_rounded_rect};
use crate::icons::IconId;
use crate::widget::{
    Button, IconButtonStyle, IconGlyph, ListItem, ListItemState, paint_button, paint_icon_button,
    paint_list_item, paint_scrollbar, paint_slider_with_chip_layout, slider_with_chip_is_stacked,
};
use crate::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::{Binding, InputAction, InputMap};
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
            format!(
                "{} {}",
                axis.label(),
                if positive { "(+)" } else { "(-)" }
            ),
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

/// **A largura do cartão, DERIVADA dos tokens** — não um número cravado.
///
/// ⛔⛔ **Report do Enio (2026-08-24, com foto): *"estreito e sem scroll"*.** A primeira largura
/// (`9 × Xl4` = 432 px) foi calculada para uma linha que já não existe — e era estreita **por
/// baixo do limiar do widget**: o `paint_slider_with_chip` **EMPILHA o rótulo numa linha própria**
/// quando o espaço aperta ([`slider_with_chip_is_stacked`]), e o doc dele diz *"quem chama tem de
/// avançar"*. Eu não avancei ⇒ os números caíam **por cima da acção seguinte** e a última saía do
/// cartão. *Um widget que muda de forma sob pressão exige que quem o coloca lhe pergunte a altura.*
///
/// A largura agora é **`13 × Spacing::Xl4`** (medido: `Xl4 = 48` ⇒ **624 px**), e sai da linha mais
/// larga, que é a da **ACÇÃO**:
///
/// | parte | largura |
/// |---|---|
/// | nome da acção | ~130 |
/// | dois `slider_with_chip` (rótulo 36 + número 48 + trilho mínimo 60 + folgas) | `2 × 160` = 320 |
/// | os dois ícones (`+` e lixo) | `2 × 28` = 56 |
/// | margens e folgas | ~40 |
///
/// ⇒ ~546 px de conteúdo, e o `13 × Xl4` é o degrau de token que o cobre com folga para um nome
/// de acção longo. ⚠️ O trilho mínimo **é do widget** — abaixo dele o rótulo empilha, e foi isso.
fn window_w() -> f32 {
    Spacing::Xl4.px() * 13.0 // LITERAL-PX-OK: multiplicador do token, derivacao acima
}

/// A coluna do rótulo dos dois números. ⚠️ **Mais estreita que a `DEFAULT_LABEL_W` (70) de
/// propósito**: "Dead" e "Press" são curtos, e os 70 px do default empurravam a linha para além do
/// limiar de empilhamento — que foi o defeito da foto.
fn zone_label_w() -> f32 {
    Spacing::Xl4.px() * 0.75 // LITERAL-PX-OK: fraccao do token
}

/// A coluna do número. Vem do próprio `number_input`, que é quem sabe quanto um número ocupa.
/// O trilho mínimo que o `slider_with_chip` exige antes de **empilhar** o rótulo.
///
/// ⚠️ **É o espelho do `SLIDER_CHIP_MIN_SLIDER_W` do widget**, e está aqui porque a conta da
/// largura da janela precisa dele ANTES de pintar. O `debug_assert` do pintor e o gate
/// `the_zone_numbers_never_stack_at_the_windows_width` são os dois guardas de que os dois números
/// concordam.
const ZONE_MIN_TRACK: f32 = 60.0; // LITERAL-PX-OK: espelho do piso do widget (ver acima)

fn zone_chip_w() -> f32 {
    Spacing::Xl4.px() // LITERAL-PX-OK: a coluna do numero, um degrau de token
}

/// **A ALTURA de uma acção**, em linhas: a linha da acção + uma por ligação (ou a face vazia).
///
/// ⚠️ **Uma função, e os dois lados chamam-na** — o cálculo da altura do cartão e o pintor. Duas
/// contas da mesma coisa divergem, e o sintoma é exactamente o da foto: a janela com um tamanho e
/// o conteúdo com outro.
fn rows_of(a: &InputAction) -> usize {
    1 + a.bindings.len().max(1)
}

/// Quantas linhas o corpo do cartão tem, incluindo os divisores.
fn body_rows(map: &InputMap) -> usize {
    if map.is_empty() {
        return 1;
    }
    map.actions().iter().map(rows_of).sum()
}

/// **O TAMANHO da janela e o TETO da rolagem** — `(largura, altura, rolagem máxima)`.
///
/// ⚠️ **A mesma conta que o pintor faz**, e é por isso que ela mora numa função: a shell precisa do
/// rectângulo para saber se a roda é dela, e duas contas da mesma coisa divergem — com o sintoma a
/// ser *"a roda funciona no meio da janela e não na ponta"*.
///
/// ⚠️ **A altura é a CLAMPADA — a mesma que o pintor desenha.** A primeira versão devolvia a
/// pedida e o doc dela afirmava o contrário; a auditoria de 2026-08-24 mostrou que a roda e o
/// arrasto passavam a testar um rectângulo que **não está na tela** assim que a lista transborda.
/// *Um doc que afirma o que a função não faz é pior que nenhum doc.*
#[must_use]
pub fn input_map_window_size(map: &InputMap, viewport_h: f32) -> (f32, f32, f32) {
    let row_h = ROW_H_PX;
    let gap = Spacing::Xs.px();
    let chrome_h = Spacing::Sm.px() * 2.0 + (row_h + gap) * 2.0;
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de linhas
    let want_body = (row_h + gap) * body_rows(map) as f32;
    // ⛔⛔ **O TETO É O TRANSBORDO, não o conteúdo inteiro** — auditoria 2026-08-24, apanhado por
    // QUATRO lentes independentes. Com o conteúdo inteiro, a roda levava a lista `body_h` px para
    // ALÉM do fim: o cartão ficava **vazio** e nada na tela dizia como voltar.
    //
    // ⚠️ E a altura devolvida é a **CLAMPADA**, a mesma que o pintor desenha. A versão anterior
    // devolvia a pedida, e o doc dela afirmava que clampava — então a roda e o arrasto testavam um
    // rectângulo que **não é o que está na tela** assim que a lista passa da viewport.
    let want_h = chrome_h + want_body;
    let h = want_h.min(viewport_h.max(chrome_h + row_h));
    let body_h = (h - chrome_h).max(row_h);
    (window_w(), h, (want_body - body_h).max(0.0))
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

    // ── A altura: título + corpo + rodapé, e o corpo é CLAMPADO à viewport. ──
    //
    // ⛔ Report do Enio: *"sem scroll"*. Um cartão que cresce com a lista sai do ecrã e a última
    // acção fica inalcançável — que é pior do que uma lista curta, porque nada na tela o diz.
    let chrome_h = pad_y * 2.0 + (row_h + gap) * 2.0;
    #[allow(clippy::cast_precision_loss)] // LITERAL-PX-OK: contagem de linhas cabe em f32
    let want_body = (row_h + gap) * body_rows(map) as f32;
    let max_body = (viewport.h - chrome_h - Spacing::Xl4.px()).max(row_h);
    let body_h = want_body.min(max_body);
    let total_h = chrome_h + body_h;

    let max_x = (viewport.x + viewport.w - window_w).max(viewport.x);
    let max_y = (viewport.y + viewport.h - total_h).max(viewport.y);
    let rect_x = x.clamp(viewport.x, max_x); // CLAMP-OK: bounds ordered + non-NaN
    let rect_y = y.clamp(viewport.y, max_y); // CLAMP-OK: bounds ordered + non-NaN
    let rect = Rect::new(rect_x, rect_y, window_w, total_h);

    let radius = Radius::Md.px();
    fill_rounded_rect(scene, rect, radius, resolve(ColorToken::BgElev, theme));
    stroke_rounded_rect(scene, rect, radius, 1.0, resolve(ColorToken::Border, theme));
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
    let handle_rect = Rect::new(rect.x, rect.y, rect.w, pad_y + row_h);
    hit_index.register(ids::INPUT_MAP_HANDLE, handle_rect);
    paint_text(
        text_system,
        scene,
        tr("input_map.title"),
        inner_x,
        cy + (row_h - font) * 0.5,
        font,
        inner_w,
        resolve(ColorToken::Text1, theme),
    );
    let close_rect = Rect::new(rect.x + rect.w - Spacing::Sm.px() - row_h, cy, row_h, row_h);
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
    let mut by = cy - scroll;

    if map.is_empty() {
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
    }

    let listening = store.input_map_listening();
    let icon_w = Spacing::Xl2.px();
    // A coluna do NOME da acção — e o indicador de escuta começa logo a seguir a ela.
    let name_w = Spacing::Xl4.px() * 2.5; // LITERAL-PX-OK: multiplicador do token
    for (row, action) in map.actions().iter().enumerate() {
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
        let zone_w = (lw + cw + Spacing::Sm.px() * 2.0 + ZONE_MIN_TRACK + Spacing::Xs.px()).ceil();
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
            debug_assert!(
                !slider_with_chip_is_stacked(zr.w, lw, cw),
                "o numero empilhou: a janela ficou estreita e a linha vai vazar (report de 24/08)"
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
        by += row_h + gap;

        // ═══ O INDICADOR DA ESCUTA — ⛔ auditoria 2026-08-24, e é o «não tem indicadores do que
        // fazer em seguida» na letra. ═══
        //
        // A primeira versão só o pintava dentro de `if bindings.is_empty()` — e o caso canónico do
        // smoke é o OPOSTO (`jump` nasce com DUAS ligações). Armar a escuta numa acção já ligada
        // não escrevia uma palavra: o único sinal era o fundo de um glifo de 24 px a mudar.
        //
        // ⇒ o texto vive AO LADO DO NOME da acção, sempre que ela está armada. É a linha que o
        // artista já está a olhar, e ela cabe sem custar linha nenhuma.
        if armed {
            paint_text(
                text_system,
                scene,
                tr("input_map.listening"),
                inner_x + name_w,
                by + (row_h - font) * 0.5,
                font,
                inner_w,
                resolve(ColorToken::Accent, theme),
            );
        }

        // ═══ As LIGAÇÕES, indentadas ═══
        if action.bindings.is_empty() {
            paint_text(
                text_system,
                scene,
                tr("input_map.no_binding"),
                inner_x + Spacing::Xl.px(),
                by + (row_h - font) * 0.5,
                font,
                inner_w,
                resolve(ColorToken::Text3, theme),
            );
            by += row_h + gap;
        }
        for (bi, b) in action.bindings.iter().enumerate() {
            let (what, device) = binding_label(*b);
            let db = ids::input_map_delete_binding_id(row, bi);
            let li_rect = Rect::new(
                inner_x + Spacing::Xl.px(),
                by,
                inner_w - Spacing::Xl.px() - icon_w,
                row_h,
            );
            paint_list_item(
                &ListItem::new(db, what)
                    .value(device)
                    .state(list_state(store, db)),
                li_rect,
                scene,
                text_system,
                theme,
            );
            let db_rect = Rect::new(rect.x + rect.w - Spacing::Sm.px() - row_h, by, row_h, row_h);
            hit_index.register(db, db_rect);
            paint_icon_button(
                db_rect,
                IconGlyph::Builtin(IconId::Close),
                IconButtonStyle::Plain,
                store.button_visual(db),
                scene,
                theme,
            );
            by += row_h + gap;
        }
    }
    hit_index.pop_clip();
    scene.pop_layer();

    // ── A BARRA DE ROLAGEM, quando a lista não cabe — e ela é REGISTADA. ──
    //
    // ⛔ Auditoria 2026-08-24: ela era pintada e **nunca registada** — não arrastava, não fazia
    // hover, e não tinha id nenhum. *Uma barra que não se pode agarrar é um enfeite que promete um
    // gesto.* O `visual` também passa a vir do store, senão ela nasce inerte sob o rato.
    if want_body > body_h {
        hit_index.register(crate::widget::INPUT_MAP_SCROLLBAR_ID, crate::widget::scrollbar_track_rect(body));
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

    // ── O RODAPÉ: nome novo + Add. ⭐ **Em BAIXO, como no Godot** — criar é o gesto raro, e no
    // topo ele empurrava a lista para longe do olho a cada abertura. ──
    let fy = rect.y + rect.h - pad_y - row_h;
    let add_w = Spacing::Xl4.px() * 1.5; // LITERAL-PX-OK: multiplicador do token, nao um px
    let name_rect = Rect::new(inner_x, fy, inner_w - add_w - gap, row_h);
    hit_index.register(ids::INPUT_MAP_NEW_NAME, name_rect);
    let typed = store.text(ids::INPUT_MAP_NEW_NAME).unwrap_or_default();
    stroke_rounded_rect(
        scene,
        name_rect,
        Radius::Sm.px(),
        1.0,
        resolve(ColorToken::Border, theme),
    );
    paint_text(
        text_system,
        scene,
        if typed.is_empty() {
            tr("input_map.new_name.placeholder")
        } else {
            typed
        },
        name_rect.x + Spacing::Sm.px(),
        fy + (row_h - font) * 0.5,
        font,
        name_rect.w,
        resolve(
            if typed.is_empty() {
                ColorToken::Text3
            } else {
                ColorToken::Text1
            },
            theme,
        ),
    );
    let add_rect = Rect::new(inner_x + inner_w - add_w, fy, add_w, row_h);
    hit_index.register(ids::INPUT_MAP_ADD, add_rect);
    paint_button(
        &Button::new(ids::INPUT_MAP_ADD, tr("input_map.add")),
        add_rect,
        scene,
        text_system,
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
