//! O acabamento R7a da vista A (a revisão lado a lado com a maquete): o pé diz UMA frase pela prioridade, com o
//! destaque na linha dela e as teclas repetidas pela ordem do mapa; a lista rola por linhas e o pé encosta na
//! última; o «+» só sob o ponteiro (ou na linha sem tecla); a linha à escuta contornada; o Add pequeno; a tecla
//! procurada no amarelo do destaque; o chip da manche «Pad · Left Stick» com a seta.

use super::lista::onze;
use super::{AREA, Mesa};
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::Key;
use ph2d_tokens::ColorToken;

fn lista(map: ph2d_input::InputMap) -> Mesa {
    let mut me = Mesa::aberta(View::List);
    me.hero.input_map = map;
    me.frame();
    me
}

fn text_rect(me: &Mesa, s: &str) -> Option<Rect> {
    me.tab()
        .doc
        .census
        .texts
        .iter()
        .find(|t| t.1 == s)
        .map(|t| t.0)
}

fn row(me: &Mesa, name: &str) -> Rect {
    let id = me.hero.input_map.id(name).expect("a acção");
    me.tab()
        .doc
        .rect_of(ListHit::Row(id).id())
        .expect("a linha pintada")
}

#[test]
fn o_pe_nomeia_as_teclas_repetidas_pela_ordem_do_mapa() {
    let me = lista(ph2d_input::InputMap::with_player_defaults());
    let want = tr("input_map_list.shared_many").replace("{keys}", "Up Arrow, Down Arrow and S");
    let names: Vec<&String> = me.tab().doc.census.texts.iter().map(|t| &t.1).collect();
    assert!(
        text_rect(&me, &want).is_some(),
        "pela ordem da lista: {names:?}"
    );
}

#[test]
fn o_pe_diz_uma_frase_so_pela_prioridade() {
    let zones = tr("input_map_list.zones_lead");
    let mut me = lista(onze());
    assert_eq!(me.tab().doc.census.foot.len(), 1, "uma frase");
    assert!(
        text_rect(&me, zones).is_some(),
        "com a manche à vista: a nota das zonas"
    );
    assert!(
        text_rect(&me, "11 actions · 16 keys.").is_none(),
        "o resumo cede o lugar"
    );
    let reload = me.hero.input_map.id("reload").expect("onze");
    let plus = me.at(ListHit::Listen(reload).id());
    me.click(plus);
    assert_eq!(me.tab().doc.census.foot.len(), 1, "uma frase à escuta");
    assert!(
        text_rect(&me, "Listening for reload.").is_some(),
        "a escuta vence"
    );
    assert!(
        text_rect(&me, zones).is_none(),
        "a nota das zonas cede à escuta"
    );
}

#[test]
fn na_aba_estreita_o_destaque_do_pe_fica_na_linha_da_frase() {
    for (map, lead) in [
        (onze(), tr("input_map_list.zones_lead").to_owned()),
        (
            ph2d_input::InputMap::with_player_defaults(),
            "8 actions · 14 keys.".to_owned(),
        ),
    ] {
        let mut me = lista(map);
        let estreita = Rect {
            w: 690.0, // LITERAL-PX-OK: a aba central a 1280×720 com a Hierarchy e o Inspector (foto A1)
            ..AREA
        };
        me.paint_in(estreita);
        let c = &me.tab().doc.census;
        let i = c
            .texts
            .iter()
            .position(|t| t.1 == lead)
            .expect("o destaque");
        let (lr, next) = (c.texts[i].0, c.texts.get(i + 1).expect("o resto").0);
        assert!(
            (next.y - lr.y).abs() < 0.5,
            "«{lead}» sozinho na linha: {lr:?} × {next:?}"
        );
        assert!(next.x > lr.x, "o resto começa depois do destaque");
        assert_eq!(c.elided(), Vec::<&str>::new());
    }
}

#[test]
fn a_lista_que_rola_encosta_o_pe_na_ultima_linha_e_rola_por_linhas() {
    let mut me = lista(onze());
    let short = Rect {
        h: 420.0, // LITERAL-PX-OK: uma aba baixa, para a lista rolar
        ..AREA
    };
    me.paint_in(short);
    let top = row(&me, "move_left").y;
    let gap = row(&me, "move_right").y - (top + row(&me, "move_left").h);
    let bottom = |me: &Mesa| {
        me.hero
            .input_map
            .actions()
            .iter()
            .filter_map(|a| me.tab().doc.rect_of(ListHit::Row(a.id).id()))
            .map(|r| r.y + r.h)
            .fold(0.0_f32, f32::max)
    };
    let foot = me.tab().doc.census.foot[0];
    assert!(
        (foot.y - bottom(&me) - gap).abs() < 0.5,
        "faixa vazia entre a última linha ({}) e o pé ({})",
        bottom(&me),
        foot.y
    );
    me.tab_mut().doc.list.scroll = row(&me, "move_left").h;
    me.paint_in(short);
    let first = me
        .hero
        .input_map
        .actions()
        .iter()
        .filter_map(|a| me.tab().doc.rect_of(ListHit::Row(a.id).id()))
        .map(|r| r.y)
        .fold(f32::MAX, f32::min);
    assert!(
        (first - top).abs() < 0.5,
        "a 1.ª linha à vista encosta no topo: {first} × {top}"
    );
    let foot = me.tab().doc.census.foot[0];
    assert!(
        (foot.y - bottom(&me) - gap).abs() < 0.5,
        "o pé encosta também a rolar"
    );
}

#[test]
fn o_mais_so_aparece_sob_o_ponteiro_ou_na_linha_sem_tecla() {
    let mut me = lista(onze());
    let reload = me.hero.input_map.id("reload").expect("onze");
    let jump = me.hero.input_map.id("jump").expect("fábrica");
    assert_eq!(
        me.tab().doc.census.plus,
        vec![reload],
        "em repouso só a linha sem tecla"
    );
    let r = row(&me, "jump");
    me.hover((r.x + r.w * 0.5, r.y + r.h * 0.5));
    let plus = &me.tab().doc.census.plus;
    assert!(plus.contains(&jump) && plus.contains(&reload), "{plus:?}");
}

#[test]
fn a_linha_a_escuta_e_contornada_no_acento() {
    let mut me = lista(onze());
    let reload = me.hero.input_map.id("reload").expect("onze");
    assert!(me.tab().doc.census.outlined.is_empty());
    let plus = me.at(ListHit::Listen(reload).id());
    me.click(plus);
    assert_eq!(me.tab().doc.census.outlined, vec![row(&me, "reload")]);
    assert_eq!(
        me.tab().doc.census.dashed.len(),
        1,
        "e a caixa tracejada no lugar"
    );
    assert!(
        text_rect(&me, tr("input_map_list.listen_key")).is_some(),
        "sem comando: «Press a key.»"
    );
}

#[test]
fn o_add_e_um_botao_pequeno_a_direita_do_nome() {
    let me = lista(onze());
    let add = me.tab().doc.rect_of(ids::ADD).expect("o Add");
    let name = me.tab().doc.rect_of(ids::NEW_NAME).expect("o nome");
    assert!(add.x >= name.x + name.w, "à direita: {add:?} × {name:?}");
    assert!(add.w * 4.0 < name.w, "pequeno: {} × {}", add.w, name.w);
}

#[test]
fn a_tecla_procurada_enche_se_do_amarelo_sem_mudar_de_tamanho() {
    let mut me = lista(onze());
    let before = text_rect(&me, "Up Arrow").expect("o chip").w;
    let search = me.at(ids::SEARCH);
    me.click(search);
    let up = Key::named_in_script("up").expect("seta")[0];
    assert!(me.physical(up));
    let lit: Vec<_> = me
        .tab()
        .doc
        .census
        .filled
        .iter()
        .filter(|f| f.1 == ColorToken::InputMapHighlight)
        .collect();
    assert_eq!(lit.len(), 2, "o Up Arrow do jump e do move_up: {lit:?}");
    assert!(
        (text_rect(&me, "Up Arrow").expect("o chip").w - before).abs() < 0.5,
        "o mesmo tamanho"
    );
}

#[test]
fn o_chip_da_manche_diz_pad_e_a_manche_com_a_seta() {
    let me = lista(onze());
    let c = &me.tab().doc.census;
    let ml = row(&me, "move_left");
    let inside = |r: Rect| r.y >= ml.y && r.y + r.h <= ml.y + ml.h;
    let pad = text_rect(&me, tr("input_map_list.pad_tag")).expect("a etiqueta «Pad»");
    let stick = text_rect(&me, tr("input_map_list.left_stick")).expect("«Left Stick»");
    assert!(
        inside(pad) && inside(stick) && pad.x < stick.x,
        "«Pad» antes de «Left Stick»"
    );
    assert!(
        text_rect(&me, tr(ph2d_input::GamepadAxis::LeftStickX.label_key())).is_none(),
        "sem o «X»"
    );
    assert!(
        c.icons.iter().any(|(r, _)| r.x > stick.x && inside(*r)),
        "a seta depois do nome"
    );
    let dead = tr("input_map_list.dead");
    assert!(
        text_rect(&me, dead).is_some() && !c.semibold.iter().any(|s| s == dead),
        "rótulo em peso normal"
    );
}
