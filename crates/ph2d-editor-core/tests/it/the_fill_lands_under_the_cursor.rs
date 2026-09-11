//! ⭐⭐⭐ **O preenchimento acaba DEBAIXO do dedo — e a barra é `1 px`, não uma fracção.**
//!
//! Report do Enio, 2026-09-03, no smoke da caixa única no app inteiro: *«temos um offset e drift em
//! relação ao cursor»*.
//!
//! # O mecanismo
//!
//! A caixa tem **duas** leis de geometria, escritas em subsistemas diferentes:
//!
//! - o **pintor** ([`widget::paint_property_box`]) enche `fill_w = surface.w * t`;
//! - o **despacho de ponteiro** (`interaction::dispatch::number_input::update_drag_value`) faz
//!   `t = (px − rect.x) / rect.w`, onde `rect` é o que o chamador **registou** no [`HitIndex`].
//!
//! Enquanto os dois `rect` forem o mesmo, a borda do preenchimento cai exactamente no cursor. A 1.ª
//! redacção da linha do produto registava *a caixa menos a coluna do valor* — por um raciocínio
//! plausível (*«ali o clique é para escrever»*) que ignora o segundo papel do rect: ele não decide
//! só **se** o gesto é meu, ele diz **por quanto dividir**. ⇒ todo valor saía multiplicado por
//! `w/(w − pad − chip_w)` = **1,62×** num painel a `220 px`, e o erro **cresce com a distância à
//! borda esquerda** — que é precisamente *offset + drift*.
//!
//! # ⚠️ Porque é que este gate corre o GESTO
//!
//! Comparar `hit.rect(SLIDER)` com `surface_rect(...)` seria uma asserção **sobre a mesma
//! expressão dos dois lados** — verde por construção no dia em que alguém reintroduzisse a conta
//! errada *no despacho*. Aqui o teste carrega o ponteiro a sério e lê o valor que o **store** ficou
//! a ter; a única coisa que liga a leitura ao pintor é a lei `x = rect.x + rect.w * t`, que é a que
//! o pintor de facto usa.
//!
//! **Mutação que deve sangrar:** voltar a `Rect::new(rect.x, rect.y, chip_rect.x − rect.x, rect.h)`
//! no `register` do `slider_with_chip` — o `1,62×` reaparece e as três posições reprovam.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{
    HitIndex, InteractiveState, WidgetStore, dispatch_pointer, format_number,
};
use ph2d_editor_core::widget::{
    DEFAULT_CHIP_W, DEFAULT_LABEL_W, SliderOrientation, SliderState, TextInputState,
    paint_slider_with_chip_layout, surface_rect,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerEvent, PointerKind, PointerSource};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// ⚠️ **Este ficheiro afirma sobre o REDESENHO**, que desde 2026-09-03 é opcional
/// (`PH2D_UI_NEW=1`) — o caminho de omissão é a UI antiga, por ordem do dono. ⇒ cada medição
/// escolhe a aparência explicitamente. ⛔ Sem isto os gates mediriam o clássico com o nome do
/// redesenho: **verdes, e sobre outra coisa**.
fn redesign() {
    ph2d_editor_core::paint::set_ui_look(ph2d_tokens::UiLook::Redesign);
}

const SLIDER: NodeId = NodeId(1);
const CHIP: NodeId = NodeId(2);

/// A linha, na largura de um Inspector real.
const ROW: Rect = Rect {
    x: 40.0,
    y: 100.0,
    w: 240.0,
    h: 22.0,
};

fn down(x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind: PointerKind::Down,
        source: PointerSource::Mouse,
        button: ph2d_host::PointerButton::Primary,
        timestamp_ns: 0,
    }
}

/// Pinta a linha do produto e devolve `(store, hits)` prontos para o gesto.
fn painted_row(value: f32) -> (WidgetStore, HitIndex) {
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value,
            orientation: SliderOrientation::Horizontal,
        },
    );
    store.register(
        CHIP,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(value),
            buffer: format_number(f64::from(value)),
            caret: 0,
            last_committed: f64::from(value),
            selection_anchor: None,
        },
    );
    let mut hits = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    paint_slider_with_chip_layout(
        ROW,
        "Geometry Offset",
        value,
        f64::from(value),
        None,
        SLIDER,
        CHIP,
        DEFAULT_LABEL_W,
        DEFAULT_CHIP_W,
        &store,
        &mut hits,
        &mut scene,
        &mut text,
        Theme::Forge,
    );
    (store, hits)
}

/// Onde o pintor põe a borda do preenchimento para um dado `t` — a lei do `paint_surface`,
/// escrita aqui uma vez para o gate a poder comparar com o cursor.
fn fill_edge_x(t: f32) -> f32 {
    // ⚠️ **`FORM_ROWS_SHOW_DECORATOR`, não um `false` escrito à mão.** Este gate reprovou no dia em
    // que a coluna de animação ligou (2026-09-03) — e estava CERTO a reprovar: o produto passou a
    // pintar e a registar sobre uma superfície `14 px` mais estreita, e um modelo com o valor
    // cravado media outra caixa. *Um gate que fixa uma constante do produto mede a versão dele que
    // já não corre.*
    let s = surface_rect(ROW, ph2d_editor_core::widget::FORM_ROWS_SHOW_DECORATOR);
    s.x + s.w * t
}

/// **A borda do preenchimento cai debaixo do cursor, em três sítios do curso.**
///
/// ⚠️ Os `px` ficam **fora da coluna do valor** de propósito: ali o alvo é o campo numérico (o
/// `HitIndex` resolve em ordem inversa de registo), e um `Down` seria um clique-para-escrever.
#[test]
fn the_fill_lands_under_the_cursor() {
    redesign();
    let arena = Bump::new();
    for frac in [0.1_f32, 0.35, 0.6] {
        let px = ROW.x + ROW.w * frac;
        let (mut store, hits) = painted_row(0.0);
        let _ = dispatch_pointer(&mut store, &hits, down(px, ROW.y + ROW.h * 0.5), &arena);
        // ⚠️ `slider(...)`, **não** `slider_visual(...)`: o segundo devolve o relógio de hover
        // (`hover_live`), que satura em `1.0` a arrastar — a 1.ª redacção deste gate leu-o e mediu
        // uma animação a fingir que media o valor.
        let (_, t) = store.slider(SLIDER).expect("a trilha existe");
        let edge = fill_edge_x(t);
        assert!(
            (edge - px).abs() <= 1.0,
            "o preenchimento acaba em {edge:.1} com o cursor em {px:.1} \
             (t = {t:.4}, esperado {frac:.4}): o alvo de arrasto e a superficie pintada \
             sao rects diferentes, e o erro CRESCE com a distancia a borda esquerda"
        );
    }
}

/// **O CONTROLO: a mesma medição sobre um rect ERRADO reprova.**
///
/// ⚠️ Sem isto o teste acima poderia estar a comparar duas expressões da mesma conta e passar
/// sempre. Aqui reproduzimos o registo defeituoso — a caixa **menos a coluna do valor** — e
/// exigimos que a barra de `1 px` o **apanhe**: se não apanhasse, o gate de cima não estaria a
/// medir nada.
#[test]
fn the_narrow_registration_that_caused_the_report_would_fail_this_gate() {
    redesign();
    let arena = Bump::new();
    let px = ROW.x + ROW.w * 0.6;
    let mut store = WidgetStore::with_capacity(4);
    store.register(
        SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    let mut hits = HitIndex::default();
    // O rect da 1.ª redacção: a caixa menos a coluna do valor (`pad + chip_w`).
    let surface = surface_rect(ROW, ph2d_editor_core::widget::FORM_ROWS_SHOW_DECORATOR);
    let narrow_w = surface.w - (ph2d_tokens::Spacing::Md.px() + DEFAULT_CHIP_W);
    hits.register(SLIDER, Rect::new(ROW.x, ROW.y, narrow_w, ROW.h));
    let _ = dispatch_pointer(&mut store, &hits, down(px, ROW.y + ROW.h * 0.5), &arena);
    let (_, t) = store.slider(SLIDER).expect("a trilha existe");
    let edge = fill_edge_x(t);
    assert!(
        (edge - px).abs() > 1.0,
        "o registo ESTREITO devia deslocar a tinta {:.1} px do cursor, e deslocou {:.1}: \
         a barra deste gate nao mede o defeito reportado",
        ROW.w / narrow_w,
        (edge - px).abs()
    );
}

/// **A coluna do valor continua a ser do CHIP** — o clique lá escreve, não arrasta.
///
/// ⚠️ É a metade que torna seguro registar o slider por baixo do número inteiro: o [`HitIndex`]
/// resolve em ordem **inversa** de registo, e o `slider_with_chip` regista o chip **depois**.
/// Trocar essa ordem faria o número deixar de ser editável em todo o app, sem nenhum outro sinal.
#[test]
fn the_value_column_still_belongs_to_the_chip() {
    redesign();
    let (_, hits) = painted_row(0.5);
    let vx = ROW.x + ROW.w - ph2d_tokens::Spacing::Md.px() - DEFAULT_CHIP_W * 0.5;
    assert_eq!(
        hits.hit(vx, ROW.y + ROW.h * 0.5),
        Some(CHIP),
        "o ponto sobre o numero tem de resolver para o campo numerico, nao para a trilha"
    );
    assert_eq!(
        hits.hit(ROW.x + 4.0, ROW.y + ROW.h * 0.5),
        Some(SLIDER),
        "o ponto sobre o ROTULO tem de arrastar: e' o modelo do Blender que o dono escolheu"
    );
}
