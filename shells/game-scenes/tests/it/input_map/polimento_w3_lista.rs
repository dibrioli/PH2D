//! W3 da lista (dono 10/11: «na lista os efeitos hover e clique ainda estão pobres. Capriche»): a linha, o chip, o
//! × do chip, o «No key», o «+» e o lixo pintam repouso, pairado e premido DIFERENTES, no quadro (D32, sem fade),
//! sem mexer um rect; arrastar para fora devolve o olhar; o × só com o chip pairado ou focado; o Tab chega a todos.

use ph2d_a11y::NodeId;
use ph2d_app_input_map::View;
use ph2d_app_input_map::census::ControlLook;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_editor_core::doc_surface::{Key as DocKey, PointerLook};
use ph2d_editor_core::zones::Rect;
use ph2d_input::{ActionId, Binding, InputMap, Key};

use super::{Mesa, em_cada_modo_de_texto};

/// As áreas REAIS da aba no app (a 1600 e a 1280), medidas na shell.
const AREAS: [Rect; 2] = [
    Rect {
        x: 308.0, // LITERAL-PX-OK: a área real da aba a 1600×1000
        y: 28.0,  // LITERAL-PX-OK: idem
        w: 988.0, // LITERAL-PX-OK: idem
        h: 890.0, // LITERAL-PX-OK: idem
    },
    Rect {
        x: 297.0, // LITERAL-PX-OK: a área real da aba a 1280×720
        y: 28.0,  // LITERAL-PX-OK: idem
        w: 690.0, // LITERAL-PX-OK: idem
        h: 604.0, // LITERAL-PX-OK: idem
    },
];

/// «jump» com o J e «crouch» sem tecla nenhuma.
fn mesa() -> (Mesa, ActionId, ActionId) {
    let mut m = Mesa::aberta(View::List);
    let mut map = InputMap::new();
    let k = map.create("jump");
    if let Some(a) = map.get_mut(k) {
        a.bindings.push(Binding::Key(Key(u32::from('J'))));
    }
    let e = map.create("crouch");
    m.hero.input_map = map;
    m.frame();
    (m, k, e)
}

fn look(m: &Mesa, id: NodeId) -> Option<ControlLook> {
    m.tab().doc.census.look(id)
}

/// Um ponto sobre o NOME da linha (nenhum controlo por cima: só a linha).
fn on_name(m: &Mesa, a: ActionId) -> (f32, f32) {
    let r = m.tab().doc.rect_of(ListHit::Row(a).id()).expect("a linha");
    (r.x + r.w * 0.02, r.y + r.h * 0.5)
}

/// Fora de toda linha (o campo do nome novo).
fn fora(m: &Mesa) -> (f32, f32) {
    m.at(ids::NEW_NAME)
}

/// Um alvo: quem, o id, onde se carrega (`(0, 0)` = o centro dele) e onde ele fica em REPOUSO.
type Alvo = (&'static str, NodeId, (f32, f32), (f32, f32));

/// Os seis alvos: o id, onde se carrega e onde ele fica em REPOUSO (à vista, sem o ponteiro nele).
fn alvos(m: &Mesa, k: ActionId, e: ActionId) -> Vec<Alvo> {
    let chip = m
        .tab()
        .doc
        .rect_of(ListHit::Chip(k, 0).id())
        .expect("o chip");
    let chip_esq = (chip.x + 2.0, chip.y + chip.h * 0.5); // LITERAL-PX-OK: a borda esquerda do chip
    vec![
        ("linha", ListHit::Row(k).id(), on_name(m, k), fora(m)),
        ("chip", ListHit::Chip(k, 0).id(), chip_esq, fora(m)),
        ("×", ListHit::RemoveBinding(k, 0).id(), (0.0, 0.0), chip_esq),
        ("No key", ListHit::Listen(e).id(), (0.0, 0.0), fora(m)),
        ("+", ListHit::Listen(k).id(), (0.0, 0.0), on_name(m, k)),
        ("lixo", ListHit::Trash(k).id(), (0.0, 0.0), on_name(m, k)),
    ]
}

#[test]
fn cada_controlo_da_linha_pinta_repouso_pairado_e_premido_diferentes_no_quadro() {
    let (mut m, k, e) = mesa();
    for (quem, id, at, rest_at) in alvos(&m, k, e) {
        m.hover(rest_at);
        let at = if at == (0.0, 0.0) { m.at(id) } else { at };
        let rest = look(&m, id).unwrap_or_else(|| panic!("{quem}: à vista em repouso"));
        m.hover(at);
        let hover = look(&m, id).expect("pairado");
        assert_ne!(
            rest.fill, hover.fill,
            "{quem}: o pairado muda o fundo NO quadro"
        );
        m.down(at);
        let press = look(&m, id).expect("premido");
        assert_ne!(hover.fill, press.fill, "{quem}: o premido afunda");
        assert!(
            rest.rect == hover.rect && hover.rect == press.rect,
            "{quem}: nenhum estado mexe o rect"
        );
        m.hover(rest_at);
        assert_eq!(
            look(&m, id),
            Some(rest),
            "{quem}: arrastar para fora devolve o repouso"
        );
        m.hover(at);
        assert_eq!(
            look(&m, id),
            Some(press),
            "{quem}: de volta, premido outra vez"
        );
        m.up(rest_at);
        assert!(
            m.tab_mut().doc.take_pending().is_empty(),
            "{quem}: o Up fora cancela"
        );
        m.hover(at);
        assert_eq!(look(&m, id), Some(hover), "{quem}: sem botão, só o pairado");
    }
    assert!(m.hero.store.input_map_listening().is_none(), "nada escutou");
}

#[test]
fn a_linha_pairada_ganha_fio_e_a_que_escuta_ganha_a_barra_do_acento() {
    let (mut m, k, _) = mesa();
    let row = ListHit::Row(k).id();
    m.hover(fora(&m));
    assert_eq!(
        look(&m, row).and_then(|l| l.edge),
        None,
        "repouso sem aresta"
    );
    let chip = m.at(ListHit::Chip(k, 0).id());
    m.hover(chip);
    let l = look(&m, row).expect("a linha");
    assert!(
        l.edge.is_some(),
        "o ponteiro num chip também levanta a linha"
    );
    let chip_l = look(&m, ListHit::Chip(k, 0).id()).expect("o chip");
    assert!(
        chip_l.edge.is_some(),
        "o chip pairado ganha o fio do acento"
    );
    m.click(on_name(&m, k));
    assert!(
        m.hero.store.input_map_listening().is_none(),
        "clicar na linha NÃO escuta (acidental)"
    );
    assert_eq!(m.tab().doc.focused(), Some(row), "clicar na linha foca-a");
    assert!(m.tab_mut().doc.take_pending().is_empty(), "nenhuma edição");
    m.click(m.at(ListHit::Listen(k).id()));
    assert_eq!(m.hero.store.input_map_listening(), Some(k), "o «+» escuta");
    let l = look(&m, row).expect("a linha");
    assert!(l.marker, "a linha à escuta tem a barra do acento");
    assert_eq!(l.edge, Some(ph2d_tokens::ColorToken::Accent));
}

#[test]
fn o_x_so_aparece_com_o_chip_pairado_ou_focado() {
    let (mut m, k, _) = mesa();
    let x = ListHit::RemoveBinding(k, 0).id();
    let chip = ListHit::Chip(k, 0).id();
    m.hover(on_name(&m, k));
    assert!(look(&m, x).is_none(), "a linha pairada não basta");
    m.hover(m.at(chip));
    assert!(look(&m, x).is_some(), "o chip pairado mostra o ×");
    m.hover(m.at(x));
    assert!(
        look(&m, x).is_some_and(|l| l.fill.is_some()),
        "o × pairado enche-se"
    );
    m.hover(fora(&m));
    assert!(look(&m, x).is_none(), "sem o ponteiro some");
    tab_ate(&mut m, chip);
    assert!(look(&m, x).is_some(), "o chip focado mostra o ×");
}

fn tab_ate(m: &mut Mesa, alvo: NodeId) -> Vec<NodeId> {
    let mut vistos = Vec::new();
    for _ in 0..64 {
        m.key(DocKey::Tab, None);
        // Os campos de texto focam no store; os controlos, no anel da aba.
        let Some(f) = m.tab().doc.focused().or(m.tab().doc.focus()) else {
            continue;
        };
        vistos.push(f);
        if f == alvo {
            return vistos;
        }
    }
    panic!("o Tab não chegou a {alvo:?}: {vistos:?}");
}

#[test]
fn o_tab_chega_a_linha_chip_x_mais_e_lixo_e_enter_na_linha_so_foca() {
    let (mut m, k, _) = mesa();
    let vistos = tab_ate(&mut m, ListHit::Trash(k).id());
    for id in [
        ListHit::Row(k).id(),
        ListHit::Chip(k, 0).id(),
        ListHit::RemoveBinding(k, 0).id(),
        ListHit::Listen(k).id(),
    ] {
        assert!(vistos.contains(&id), "{id:?} fora da ordem do Tab");
    }
    assert!(
        look(&m, ListHit::Trash(k).id()).is_some(),
        "o lixo focado à vista"
    );
    tab_ate(&mut m, ListHit::Row(k).id());
    m.key(DocKey::Enter, None);
    assert!(
        m.hero.store.input_map_listening().is_none(),
        "Enter na linha não escuta"
    );
    assert_eq!(
        m.tab().doc.focused(),
        Some(ListHit::Row(k).id()),
        "Enter na linha deixa o foco (o anel) nela"
    );
    assert!(m.tab_mut().doc.take_pending().is_empty(), "nenhuma edição");
}

#[test]
fn a_mao_em_todos_os_controlos_da_linha() {
    let (mut m, k, e) = mesa();
    m.hover(m.at(ListHit::Chip(k, 0).id()));
    for id in [
        ListHit::Row(k).id(),
        ListHit::Chip(k, 0).id(),
        ListHit::RemoveBinding(k, 0).id(),
        ListHit::Listen(k).id(),
        ListHit::Listen(e).id(),
        ListHit::Trash(k).id(),
    ] {
        let (x, y) = if id == ListHit::Row(k).id() {
            on_name(&m, k)
        } else {
            m.at(id)
        };
        assert_eq!(m.tab().doc.look_at(x, y), Some(PointerLook::Hand), "{id:?}");
        let r = m.tab().doc.rect_of(id).expect("o alvo");
        if id != ListHit::Row(k).id()
            && id != ListHit::Chip(k, 0).id()
            && id != ListHit::Listen(e).id()
        {
            assert!(
                r.w >= 24.0 && r.h >= 24.0,
                "{id:?}: alvo de ícone ≥ 24 px: {r:?}"
            ); // LITERAL-PX-OK: o mínimo
        }
    }
}

#[test]
fn nenhum_texto_cortado_com_o_x_a_vista_nas_areas_reais() {
    em_cada_modo_de_texto(|| {
        let mut m = Mesa::aberta(View::List);
        m.hero.input_map = super::lista::onze();
        m.frame();
        let jump = m.hero.input_map.id("jump").expect("fábrica");
        tab_ate(&mut m, ListHit::Chip(jump, 0).id());
        for area in AREAS {
            m.paint_in(area);
            let cortados = m.tab().doc.census.elided().join(" | ");
            assert!(cortados.is_empty(), "{area:?}: {cortados}");
            assert!(
                look(&m, ListHit::RemoveBinding(jump, 0).id()).is_some(),
                "o × à vista"
            );
        }
    });
}

/// Dono 10/11 («o botão fechar deve ter mais destaque»): o × é um emblema redondo DENTRO do chip — neutro forte em
/// repouso, o perigo pairado, o perigo premido —, e nenhum estado mexe o alvo nem a face.
#[test]
fn o_x_do_chip_e_um_emblema_que_se_acende_no_perigo() {
    use ph2d_tokens::ColorToken;
    let (mut m, k, _) = mesa();
    let x = ListHit::RemoveBinding(k, 0).id();
    let chip_id = ListHit::Chip(k, 0).id();
    let chip = m.tab().doc.rect_of(chip_id).expect("o chip");
    let chip_esq = (chip.x + 2.0, chip.y + chip.h * 0.5); // LITERAL-PX-OK: a borda esquerda do chip
    let face = |m: &Mesa| {
        let b = &m.tab().doc.census.badges;
        let [(id, r)] = b.as_slice() else {
            panic!("um só emblema à vista: {b:?}");
        };
        assert_eq!(*id, x);
        *r
    };
    m.hover(chip_esq);
    let rest = look(&m, x).expect("o × à vista com o chip pairado");
    let f = face(&m);
    assert!(
        f.x >= chip.x
            && f.y >= chip.y
            && f.x + f.w <= chip.x + chip.w
            && f.y + f.h <= chip.y + chip.h,
        "o emblema mora dentro do chip: {f:?} em {chip:?}"
    );
    assert!(
        (f.w - f.h).abs() < 0.01 && f.w < rest.rect.w,
        "redondo e menor que o alvo"
    );
    assert!(rest.rect.w >= 24.0 && rest.rect.h >= 24.0); // LITERAL-PX-OK: o alvo mínimo
    assert_eq!(
        rest.fill,
        Some(ColorToken::BorderStrong),
        "repouso: neutro forte"
    );
    assert_eq!(rest.ink, ColorToken::Text1);
    let at = m.at(x);
    m.hover(at);
    let hover = look(&m, x).expect("pairado");
    assert_eq!(hover.fill, Some(ColorToken::Danger), "pairado: o perigo");
    assert_eq!(
        hover.ink,
        ColorToken::AccentFg,
        "a tinta clara de quem se enche"
    );
    m.down(at);
    let press = look(&m, x).expect("premido");
    assert_eq!(
        press.fill,
        Some(ColorToken::DangerPress),
        "premido: o perigo premido (onda 3, item 5)"
    );
    assert_eq!(press.ink, ColorToken::AccentFg);
    assert!(
        rest.fill != hover.fill && hover.fill != press.fill && rest.fill != press.fill,
        "três fundos"
    );
    assert!(
        rest.rect == hover.rect && hover.rect == press.rect,
        "o alvo não mexe"
    );
    assert_eq!(face(&m), f, "a face não mexe");
    assert_eq!(m.tab().doc.rect_of(chip_id), Some(chip), "o chip não mexe");
    m.up(chip_esq);
    assert!(
        m.tab_mut().doc.take_pending().is_empty(),
        "o Up fora cancela"
    );
}
