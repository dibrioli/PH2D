//! Smoke do dono 09/11 (cena 14): o Stopped toma no máximo um terço da aba sozinho, o puxador de
//! cima muda-lhe a altura entre o mínimo e dois terços, e os Cards rolam por dentro.

use ph2d_a11y::NodeId;
use ph2d_code_debugcard::{CardState, CardStyle, DebugCard, StackEntry, VarRow};
use ph2d_editor_core::doc_surface::PointerLook;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::CodeView;
use ph2d_panel_code::debug_card::DebugCardView;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ABA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 900.0,
    h: 900.0,
};

fn parado(n: usize) -> DebugCard {
    let var = |i: usize| VarRow {
        key: format!("v{i}"),
        name: format!("v{i}"),
        depth: 0,
        value: i.to_string(),
        kind: Default::default(),
        field_count: 0,
        expandable: false,
        old: None,
    };
    DebugCard {
        state: CardState::Paused,
        line: 3,
        locals: (0..n).map(var).collect(),
        stack: vec![StackEntry {
            function: "update".into(),
            line: 3,
        }],
        can_step: true,
        ..DebugCard::default()
    }
}

/// Pinta `c` com a altura que o quadro pede na aba, encostado ao fundo dela.
fn pinta(v: &mut DebugCardView, c: &DebugCard) -> Rect {
    let h = v.height(c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    let ids = [NodeId(1), NodeId(2)];
    v.paint(
        c,
        ids,
        r,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    r
}

#[test]
fn o_stopped_tem_tecto_de_um_terco_e_o_puxador_vai_do_minimo_a_dois_tercos() {
    let (c, mut v) = (parado(40), DebugCardView::default());
    let r = pinta(&mut v, &c);
    assert!((r.h - ABA.h / 3.0).abs() < 1.0, "um terço sozinho: {}", r.h);
    let poucas = DebugCardView::default().height(&parado(1), ABA);
    assert!(poucas < r.h, "o natural abaixo do tecto: {poucas}");
    // Carregar no puxador, subir 200 px: cresce 200; o ponteiro é a mão.
    let g = v.painted.grip.expect("o puxador");
    let (gx, gy) = (g.x + g.w * 0.5, g.y + g.h * 0.5);
    assert_eq!(v.look(gx, gy), Some(PointerLook::Grab));
    assert!(v.press(gx, gy).is_some());
    assert!(v.drag_to(gx, gy - 200.0));
    assert_eq!(v.look(0.0, 0.0), Some(PointerLook::Grabbing), "a arrastar");
    assert!(v.release());
    assert!((pinta(&mut v, &c).h - (r.h + 200.0)).abs() < 1.0);
    // Muito para cima: dois terços; muito para baixo: o mínimo (a faixa + 4 linhas).
    for (dy, h) in [(-2000.0, ABA.h * 2.0 / 3.0), (2000.0, poucas.min(r.h))] {
        let g = v.painted.grip.expect("o puxador");
        let gy = g.y + g.h * 0.5;
        assert!(v.press(gx, gy).is_some() && v.drag_to(gx, gy + dy) && v.release());
        let now = pinta(&mut v, &c).h;
        assert!(now <= h + 1.0 && now > 0.0, "{dy}: {now} (até {h})");
    }
}

/// Smoke do dono 09/11: nos Cards cada cartão rola por si (a roda sobre o self não mexe nos
/// Locals), preso ao seu conteúdo, sem linhas a meio, com o indicador só no que transborda.
#[test]
fn os_cards_rolam_cada_um_por_si_e_a_roda_fora_do_quadro_nao_e_dele() {
    let mut c = parado(2);
    c.this = parado(40).locals;
    let mut v = DebugCardView::default();
    v.style = CardStyle::Cards;
    let r = pinta(&mut v, &c);
    let cards = v.painted.cards.clone();
    assert_eq!(cards.len(), 3, "Locals, self, Call stack");
    assert!(
        cards[0].bar.is_none() && cards[2].bar.is_none(),
        "cabem: sem indicador"
    );
    let bar = cards[1].bar.expect("o self transborda: indicador");
    assert!(
        bar.x + bar.w <= cards[1].rect.x + cards[1].rect.w,
        "à direita, dentro dele"
    );
    assert!(cards[0].max == 0.0 && cards[1].max > 0.0);
    let inteiras = |v: &DebugCardView| {
        v.painted.values.iter().all(|(_, f)| {
            v.painted
                .cards
                .iter()
                .any(|b| f.y >= b.rect.y && f.y + f.h <= b.rect.y + b.rect.h)
        })
    };
    assert!(inteiras(&v), "nenhuma linha a meio");
    let shown = v.painted.values.len();
    assert!(shown > 2 && shown < 42, "só as que cabem: {shown}");
    assert!(
        !v.wheel(r.x + 10.0, r.y - 50.0, -100.0),
        "acima do quadro é do código"
    );
    let (sx, sy) = {
        let b = cards[1].rect;
        (b.x + b.w * 0.5, b.y + b.h * 0.5)
    };
    assert!(v.wheel(sx, sy, -100.0));
    assert!(
        v.cards_scroll[1] > 0.0 && v.cards_scroll[0] == 0.0,
        "{:?}",
        v.cards_scroll
    );
    let b0 = cards[0].rect;
    assert!(v.wheel(b0.x + b0.w * 0.5, b0.y + b0.h * 0.5, -100.0));
    assert_eq!(v.cards_scroll[0], 0.0, "os Locals cabem: não rolam");
    assert!(v.wheel(sx, sy, -1e6));
    assert!(
        (v.cards_scroll[1] - cards[1].max).abs() < 1e-3,
        "preso ao fim"
    );
    pinta(&mut v, &c);
    assert!(inteiras(&v));
    let vals: Vec<&str> = v.painted.values.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(&vals[..2], ["0", "1"], "os Locals ficaram");
    assert_eq!(vals.last().copied(), Some("39"), "o self chegou ao fim");
    assert!(!vals[2..].contains(&"0"), "a primeira do self saiu de cima");
}

/// Smoke do dono 09/11 («não consigo arrastar o novo scroll»): a barra de um cartão dos Cards
/// arrasta-se pelo polegar (proporcional, presa aos fins) e salta pela trilha; só esse cartão rola
/// e o puxador de cima não mexe.
#[test]
fn a_barra_de_um_cartao_arrasta_pelo_polegar_e_salta_pela_trilha() {
    use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
    let mut c = parado(2);
    c.this = parado(40).locals;
    let mut v = DebugCardView::default();
    v.style = CardStyle::Cards;
    let _ = pinta(&mut v, &c);
    let b = v.painted.cards[1];
    let hit = b.track.hit().expect("o self transborda");
    assert!(hit.w >= ph2d_tokens::Spacing::Sm.px());
    assert!(v.painted.cards[0].track.hit().is_none(), "os Locals cabem");
    let track = scrollbar_track_rect(b.track.body);
    let thumb = scrollbar_thumb_rect(track, 0.0, b.track.content_h, b.track.visible_h);
    let range = track.h - thumb.h;
    let (x, y0) = (hit.x + hit.w * 0.5, thumb.y + thumb.h * 0.5);
    assert!(v.press(x, y0).is_some());
    assert_eq!(v.cards_scroll[1], 0.0, "carregar no polegar não rola");
    assert!(v.bar_dragging());
    assert!(v.drag_to(x, y0 + range * 0.5));
    assert!(
        (v.cards_scroll[1] - b.max * 0.5).abs() < 1e-2,
        "{:?}",
        v.cards_scroll
    );
    assert!(v.drag_to(x, y0 + range * 10.0));
    assert!((v.cards_scroll[1] - b.max).abs() < 1e-3, "preso ao fim");
    assert!(v.cards_scroll[0] == 0.0 && v.user_h.is_none(), "só o self");
    assert!(v.release() && !v.bar_dragging());
    assert!(!v.drag_to(x, y0), "largado");
    let _ = pinta(&mut v, &c);
    assert!(v.press(x, track.y + 1.0).is_some());
    assert_eq!(v.cards_scroll[1], 0.0, "a trilha no topo salta ao início");
    assert!(v.release());
}

/// Smoke do dono 09/11 (cena 14): «arrastar com o mouse, mostrando a seta do cursor na vertical». Por
/// cima da barra da Sheet (e da de um cartão) o ponteiro é ↕; agarrado o polegar, fica ↕ mesmo com o
/// rato fora da barra, e a rolagem segue o rato; largado, fora da barra volta a não pedir nada.
#[test]
fn a_barra_da_sheet_e_a_de_um_cartao_pedem_o_ponteiro_vertical() {
    use ph2d_editor_core::widget::{scrollbar_thumb_rect, scrollbar_track_rect};
    let c = parado(40);
    let mut v = DebugCardView::default();
    let _ = pinta(&mut v, &c);
    let b = v.painted.table.as_ref().expect("Sheet").0.bar();
    let hit = b.hit().expect("a Sheet transborda");
    let thumb = scrollbar_thumb_rect(scrollbar_track_rect(b.body), 0.0, b.content_h, b.visible_h);
    let (x, y) = (hit.x + hit.w * 0.5, thumb.y + thumb.h * 0.5);
    let fora = (b.body.x + 1.0, b.body.y + 1.0);
    assert_eq!(v.look(x, y), Some(PointerLook::ResizeNs), "sobre a barra");
    assert_eq!(v.look(fora.0, fora.1), None, "sobre uma célula");
    assert!(v.press(x, y).is_some() && v.table.bar_dragging());
    assert!(v.drag_to(fora.0, y + (b.body.h - thumb.h) * 0.5));
    assert!(v.table.scroll > 0.0, "a rolagem segue o rato");
    assert_eq!(
        v.look(fora.0, fora.1),
        Some(PointerLook::ResizeNs),
        "a arrastar, fora dela"
    );
    assert!(v.release());
    assert_eq!(v.look(fora.0, fora.1), None, "largado");
    let mut c = parado(2);
    c.this = parado(40).locals;
    v.style = CardStyle::Cards;
    let _ = pinta(&mut v, &c);
    let hit = v.painted.cards[1].track.hit().expect("o self transborda");
    assert_eq!(
        v.look(hit.x + hit.w * 0.5, hit.y + 1.0),
        Some(PointerLook::ResizeNs)
    );
}

/// Smoke do dono 09/11 (cena 14): «coloque um separador vertical onde o cursor vai pegar». Entre a
/// tabela e a coluna do lado há uma faixa à direita da calha da barra (as zonas não se tocam): por
/// cima e a arrastar o ponteiro é ↔; para a esquerda a coluna do lado cresce e a tabela encolhe,
/// presas aos mínimos (a tabela guarda Value e Type inteiros); a largura fica no `DebugCardView`.
#[test]
fn o_separador_da_sheet_arrasta_de_lado_e_pede_o_ponteiro_horizontal() {
    let c = parado(40);
    let mut v = DebugCardView::default();
    let _ = pinta(&mut v, &c);
    let (lay, s) = (
        v.painted.table.clone().expect("Sheet").0,
        v.painted.split.expect("o separador"),
    );
    let bar = lay.bar().hit().expect("a Sheet transborda");
    assert!(
        bar.x + bar.w < s.hit.x,
        "a barra e o separador não se tocam: {bar:?} {:?}",
        s.hit
    );
    assert!(
        s.hit.w > ph2d_tokens::StrokeToken::Thin.px() * 2.0,
        "agarra mais largo que a linha"
    );
    let (x, y) = (s.hit.x + s.hit.w * 0.5, s.hit.y + s.hit.h * 0.5);
    assert_eq!(
        v.look(x, y),
        Some(PointerLook::ResizeEw),
        "sobre o separador"
    );
    assert_eq!(
        v.look(bar.x + 1.0, y),
        Some(PointerLook::ResizeNs),
        "a barra continua ↕"
    );
    let (table_w, side_w) = (lay.area.w, s.side_w);
    assert!(v.press(x, y).is_some());
    assert!(v.drag_to(x - 100.0, y + 30.0));
    assert_eq!(
        v.look(x - 100.0, y),
        Some(PointerLook::ResizeEw),
        "a arrastar, fora dele"
    );
    assert!(v.release());
    assert_eq!(v.side_w, Some(side_w + 100.0));
    let _ = pinta(&mut v, &c);
    let lay = v.painted.table.clone().expect("Sheet").0;
    assert!(
        (lay.area.w - (table_w - 100.0)).abs() < 1e-3,
        "a tabela fica com o resto"
    );
    // Presos: a tabela nunca abaixo do mínimo (Value e Type inteiros), o lado nunca abaixo do título.
    for dx in [-5000.0, 5000.0] {
        let s = v.painted.split.expect("o separador");
        let (x, y) = (s.hit.x + 1.0, s.hit.y + 1.0);
        assert!(v.press(x, y).is_some() && v.drag_to(x + dx, y) && v.release());
        let _ = pinta(&mut v, &c);
        let (lay, s) = (
            v.painted.table.clone().expect("Sheet").0,
            v.painted.split.expect("sep"),
        );
        let w = |k: &str| lay.col(k).map_or(0.0, |c| c.w);
        assert!(
            s.side_w >= s.min - 1e-3 && s.side_w <= s.max + 1e-3,
            "{s:?}"
        );
        assert!(
            w("name") >= ph2d_ui_table::Metrics::standard().min_col - 1e-3,
            "{dx}: {:?}",
            lay.cols
        );
        assert!(
            lay.cols.iter().map(|c| c.w).sum::<f32>() <= lay.area.w + 1e-3,
            "{dx}: Type cortada"
        );
    }
    // Os Cards não têm a divisão tabela | lado: nenhum separador.
    v.style = ph2d_code_debugcard::CardStyle::Cards;
    let _ = pinta(&mut v, &c);
    assert!(v.painted.split.is_none());
}

/// Dono 10/11: em «Continue F8» e «Next line F10» o rótulo fica no meio do botão (pelas maiúsculas)
/// e a tecla, mais pequena, na MESMA linha de base dele — lido na tinta.
#[test]
fn o_rotulo_do_botao_fica_no_meio_e_a_tecla_na_linha_de_base_dele() {
    let (c, mut v) = (parado(1), DebugCardView::default());
    let h = v.height(&c, ABA);
    let r = Rect::new(ABA.x, ABA.y + ABA.h - h, ABA.w, h);
    let (mut scene, mut ts) = (VectorScene::new(), TextSystem::without_system_fonts());
    v.paint(
        &c,
        [NodeId(1), NodeId(2)],
        r,
        &mut CodeView::new(),
        &mut scene,
        &mut ts,
        Theme::default(),
    );
    let runs = crate::linhas_de_base(&scene);
    let (font, small) = (
        ph2d_tokens::TypeToken::Sm.px(),
        ph2d_tokens::TypeToken::Xs.px(),
    );
    let labels = [
        (ph2d_i18n::tr("code.debug.continue"), font),
        (ph2d_i18n::tr("code.debug.next_line"), font),
    ];
    assert_eq!(v.painted.buttons.len(), 2, "Continue e Next line");
    for (i, &(_, br)) in v.painted.buttons.iter().enumerate() {
        let dentro: Vec<_> = runs
            .iter()
            .filter(|(_, x, y)| *x >= br.x && *x <= br.x + br.w && *y >= br.y && *y <= br.y + br.h)
            .collect();
        let base_of = |px: f32| {
            dentro
                .iter()
                .find(|(s, ..)| (*s - px).abs() < 1e-3)
                .map(|t| t.2)
                .expect("o rótulo e a tecla pintaram")
        };
        let (rotulo, tecla) = (base_of(font), base_of(small));
        assert!(
            (rotulo - tecla).abs() < 0.25,
            "botão {i}: rótulo na base {rotulo}, tecla em {tecla}"
        );
        let meio = crate::meio_das_maiusculas(&mut ts, labels[i].0, labels[i].1, rotulo);
        let alvo = br.y + br.h * 0.5;
        assert!((meio - alvo).abs() < 0.75, "botão {i}: {meio} × {alvo}");
    }
}
