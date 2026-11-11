use super::*;
use crate::panel::cards::censo;
use crate::panel::cards::corpo;
use crate::property_row::cartao;
use crate::widget::card_field::Glyph;
use ph2d_tokens::Theme;

struct Mesa {
    scene: VectorScene,
    ts: TextSystem,
    store: WidgetStore,
    hit: HitIndex,
}

fn mesa() -> Mesa {
    Mesa {
        scene: VectorScene::new(),
        ts: TextSystem::without_system_fonts(),
        store: WidgetStore::default(),
        hit: HitIndex::default(),
    }
}

fn ctx(f: &mut Mesa, w: f32) -> CardCtx<'_> {
    CardCtx::flutuante(
        &mut f.scene,
        &mut f.ts,
        &f.store,
        &mut f.hit,
        CardInk::of_list(Theme::default()),
        0.0,
        w,
    )
}

fn campo<'a>(id: NodeId, rotulo: &'a str) -> Cell<'a> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text("Z"),
    })
    .label(rotulo)
}

fn marca(id: NodeId, nome: &str) -> Cell<'_> {
    Cell::new(Widget::Check {
        id,
        label: Rotulo::from(nome),
        value: CheckboxValue::Unchecked,
        off: false,
    })
}

const A: NodeId = NodeId(0x4b_0001);
const B: NodeId = NodeId(0x4b_0002);
const C: NodeId = NodeId(0x4b_0003);

/// ⭐ Lei 1: dois curtos em colunas iguais, a caixa de marcar alinhada com a CAIXA do campo.
/// *Mutação: `pair` sempre empilhado.*
#[test]
fn a_short_pair_shares_the_line_in_equal_columns() {
    let mut f = mesa();
    let w = 257.0;
    let fim = ctx(&mut f, w).pair(campo(A, "Emissive"), marca(B, "Centered"), 0.0);
    let (ra, rb) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(B).unwrap());
    assert_eq!(ra.y, rb.y, "a marca fica na linha da caixa do campo");
    assert_eq!(ra.w, rb.w, "colunas iguais");
    assert!(rb.x > ra.x + ra.w);
    assert_eq!(fim, ra.y + m::FIELD_H);
}

/// ⭐ Lei 5: na coluna estreita o par desce para duas linhas e nada se corta.
/// *Mutação: `tudo_cabe` sempre verdadeiro.*
#[test]
fn a_pair_stacks_when_the_column_is_narrow() {
    let mut f = mesa();
    let ((), censo) = censo::grava(|| {
        ctx(&mut f, 170.0).pair(campo(A, "Emissive"), marca(B, "Show Behind Parent"), 0.0);
    });
    let (ra, rb) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(B).unwrap());
    assert!(rb.y >= ra.y + ra.h, "{ra:?} {rb:?}");
    assert_eq!(ra.w, 170.0);
    assert!(censo.textos_cortados().is_empty(), "{:?}", censo.textos);
}

/// ⭐ Lei 7: três botões que não cabem numa linha descem; nenhum corta o texto.
/// *Mutação: `row` sem `desce`.*
#[test]
fn buttons_that_do_not_fit_go_down_a_line() {
    let mut f = mesa();
    let botao = |id, text| {
        Cell::new(Widget::Button {
            id,
            text,
            primary: false,
        })
    };
    let (fim, censo) = censo::grava(|| {
        ctx(&mut f, 190.0).row(
            &[
                botao(A, "Wet canvas"),
                botao(B, "Dry canvas"),
                botao(C, "Fast dry"),
            ],
            0.0,
        )
    });
    let (ra, rc) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(C).unwrap());
    assert!(rc.y > ra.y, "o 3.º desce: {ra:?} {rc:?}");
    assert_eq!(fim, rc.y + m::FIELD_H);
    assert!(censo.textos_cortados().is_empty(), "{:?}", censo.textos);
}

/// ⭐ Lei 7: a escolha quebra em filas; rótulo só na primeira, e o segmento mede a palavra inteira.
/// *Mutação: `escolha_em_linhas` a pintar tudo numa linha.*
#[test]
fn a_choice_that_does_not_fit_wraps_into_rows() {
    let mut f = mesa();
    let seg = |id, text| Seg {
        id,
        icon: None,
        text,
        on: false,
    };
    let segs = [
        seg(A, "Wet canvas"),
        seg(B, "Dry canvas"),
        seg(C, "Hand-packed"),
    ];
    let (_, censo) = censo::grava(|| {
        ctx(&mut f, 160.0).row(&[Cell::new(Widget::Choice(&segs)).label("Whole")], 0.0)
    });
    let (ra, rc) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(C).unwrap());
    assert!(rc.y > ra.y);
    assert!(censo.textos_cortados().is_empty(), "{:?}", censo.textos);
}

/// O corpo que pede linhas soltas à largura do corpo, como os componentes do Inspector.
fn corre_fluxo(f: &mut Mesa, w: f32, pedidos: impl FnOnce(&mut CardCtx<'_>, f32) -> f32) -> f32 {
    let mut c = ctx(f, w);
    crate::widget::card_ink::com(c.ink, || {
        cartao::com(crate::panel::cards::corpo::LINHAS, || {
            fluir(&mut c, |c| pedidos(c, 0.0))
        })
    })
}

/// ⭐ Dois pedidos curtos seguidos saem lado a lado; o 3.º fica sozinho, a toda a largura, e o fundo
/// que o corpo recebeu é o que se pintou. *Mutação: `pede` a nunca juntar.*
#[test]
fn the_flow_pairs_neighbours_and_paints_the_last_one_alone() {
    let mut f = mesa();
    let w = 257.0;
    let fim = corre_fluxo(&mut f, w, |c, y| {
        let y = c.check(A, "Centered", false, y);
        let y = c.check(B, "Region", true, y);
        c.check(C, "Top Level", false, y)
    });
    let (ra, rb, rc) = (
        f.hit.rect_for(A).unwrap(),
        f.hit.rect_for(B).unwrap(),
        f.hit.rect_for(C).unwrap(),
    );
    assert_eq!(ra.y, rb.y, "o par");
    assert!(rb.x > ra.x);
    assert!(rc.y > ra.y);
    assert_eq!(rc.w, w, "o último fica sozinho");
    assert_eq!(fim, rc.y + rc.h, "o fundo devolvido é o da pintura");
}

/// ⭐ Um salto no `y` (outra coisa pintou-se no meio) separa os dois pedidos.
#[test]
fn the_flow_does_not_pair_across_something_painted_between() {
    let mut f = mesa();
    corre_fluxo(&mut f, 257.0, |c, y| {
        let y = c.check(A, "Centered", false, y);
        c.check(B, "Region", true, y + m::FIELD_H * 2.0)
    });
    let (ra, rb) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(B).unwrap());
    assert!(rb.y > ra.y + ra.h);
}

/// ⭐ O censo acusa o par que ficou separado e a linha grudada, e não acusa o par correcto.
#[test]
fn the_census_accuses_a_lone_sibling_and_a_glued_line() {
    let mut f = mesa();
    censo::na_secao(Some(A));
    let (_, mau) = censo::grava(|| {
        let mut c = ctx(&mut f, 257.0);
        let y = c.row_inteira(&[campo(A, "Tint")], 0.0);
        let y = c.row_inteira(&[campo(B, "Self Tint")], y);
        censo::fora_da_linha(
            "Equalize",
            10.0,
            crate::zones::Rect::new(0.0, y, 257.0, 28.0),
        );
    });
    assert_eq!(mau.irmaos_separados().len(), 1, "{:?}", mau.linhas);
    assert_eq!(mau.linhas_grudadas().len(), 1, "{:?}", mau.linhas);
    let (_, bom) = censo::grava(|| {
        ctx(&mut f, 257.0).pair(campo(A, "Tint"), campo(B, "Self Tint"), 0.0);
    });
    censo::na_secao(None);
    assert!(bom.irmaos_separados().is_empty(), "{:?}", bom.linhas);
    assert!(bom.linhas_grudadas().is_empty());
}

/// Uma lista suspensa sob um rótulo pedido à parte (`paint_label_row`) é um controlo curto: duas seguidas
/// acusam irmãos separados. *Mutação: `card_chip::chip` sem `controlo_curto` ⇒ o par não é visto.*
#[test]
fn a_chip_under_a_label_row_counts_as_short() {
    let mut f = mesa();
    let ink = CardInk::of_list(Theme::default());
    censo::na_secao(Some(A));
    let (_, c) = censo::grava(|| {
        let Mesa { scene, ts, .. } = &mut f;
        for (nome, y) in [("When", 0.0), ("Compare", 51.0)] {
            let linha = super::super::corpo::rotulo_em_cima(scene, ts, ink, 0.0, 257.0, y, nome);
            crate::widget::paint_card_chip(scene, ts, ink, linha.control, "Equal", false);
        }
    });
    censo::na_secao(None);
    assert_eq!(c.irmaos_separados().len(), 1, "{:?}", c.linhas);
}

/// O texto da lista entra no que a linha pede: duas listas longas demais para meia coluna não são irmãs
/// separadas. *Mutação: `controlo_curto` sem somar o `pede` ⇒ acusadas.*
#[test]
fn a_chip_too_long_for_half_a_column_is_not_a_separated_sibling() {
    let mut f = mesa();
    let ink = CardInk::of_list(Theme::default());
    censo::na_secao(Some(A));
    let longo = "An option name far too long to share the width of a single card column";
    let (_, c) = censo::grava(|| {
        let Mesa { scene, ts, .. } = &mut f;
        for (nome, y) in [("When", 0.0), ("Compare", 51.0)] {
            let linha = super::super::corpo::rotulo_em_cima(scene, ts, ink, 0.0, 257.0, y, nome);
            crate::widget::paint_card_chip(scene, ts, ink, linha.control, longo, false);
        }
    });
    censo::na_secao(None);
    assert!(c.irmaos_separados().is_empty(), "{:?}", c.linhas);
}

/// Uma lista fora da coluna do rótulo não é a dele. *Mutação: `sob` sempre verdadeiro ⇒ acusadas.*
#[test]
fn a_chip_outside_the_label_column_does_not_shorten_it() {
    let mut f = mesa();
    let ink = CardInk::of_list(Theme::default());
    censo::na_secao(Some(A));
    let (_, c) = censo::grava(|| {
        let Mesa { scene, ts, .. } = &mut f;
        for (nome, y) in [("When", 0.0), ("Compare", 51.0)] {
            let linha = super::super::corpo::rotulo_em_cima(scene, ts, ink, 100.0, 157.0, y, nome);
            let fora = crate::zones::Rect {
                x: 0.0,
                ..linha.control
            };
            crate::widget::paint_card_chip(scene, ts, ink, fora, "Equal", false);
        }
    });
    censo::na_secao(None);
    assert!(c.irmaos_separados().is_empty(), "{:?}", c.linhas);
}

/// Um rótulo responde por UMA lista: a segunda sob ele não lhe muda o pedido.
/// *Mutação: `rotulo_livre` por gastar ⇒ a longa conta, e o par curto deixa de ser acusado.*
#[test]
fn a_label_answers_for_its_first_chip_only() {
    let mut f = mesa();
    let ink = CardInk::of_list(Theme::default());
    censo::na_secao(Some(A));
    let longo = "An option name far too long to share the width of a single card column";
    let (_, c) = censo::grava(|| {
        let Mesa { scene, ts, .. } = &mut f;
        for (nome, y) in [("When", 0.0), ("Compare", 51.0)] {
            let linha = super::super::corpo::rotulo_em_cima(scene, ts, ink, 0.0, 257.0, y, nome);
            crate::widget::paint_card_chip(scene, ts, ink, linha.control, "Equal", false);
            crate::widget::paint_card_chip(scene, ts, ink, linha.control, longo, false);
        }
    });
    censo::na_secao(None);
    assert_eq!(c.irmaos_separados().len(), 1, "{:?}", c.linhas);
}

/// Sem lista por baixo, o rótulo continua longo: uma grelha ou um grupo de amostras não se dividem.
#[test]
fn a_label_row_without_a_chip_stays_long() {
    let mut f = mesa();
    let ink = CardInk::of_list(Theme::default());
    censo::na_secao(Some(A));
    let (_, c) = censo::grava(|| {
        let Mesa { scene, ts, .. } = &mut f;
        for (nome, y) in [("When", 0.0), ("Compare", 51.0)] {
            super::super::corpo::rotulo_em_cima(scene, ts, ink, 0.0, 257.0, y, nome);
        }
    });
    censo::na_secao(None);
    assert!(c.irmaos_separados().is_empty(), "{:?}", c.linhas);
}

/// Um corpo pintado por `card_ink::com` fora da lista (a captura do Painter) conta, e cada captura
/// é uma secção. *Mutação: `entra_na_tinta` calada ⇒ duas capturas juntam-se e acusam uma grudada.*
#[test]
fn a_captured_body_is_measured_as_its_own_section() {
    let ink = CardInk::of_list(Theme::default());
    let linha = |y: f32| censo::linha_antiga_em_captura((0.0, 257.0), y, y + 20.0);
    let (_, c) = censo::grava(|| {
        crate::widget::card_ink::com(ink, || {
            linha(0.0);
            linha(23.0);
        });
        crate::widget::card_ink::com(ink, || linha(43.0));
    });
    assert_eq!(c.linhas.len(), 3);
    assert!(c.linhas.iter().all(|l| l.em_captura));
    assert_eq!(c.linhas_grudadas().len(), 1, "{:?}", c.linhas);
}

fn duas_opcoes<'a>(a: NodeId, b: NodeId) -> [Seg<'a>; 2] {
    [
        Seg {
            id: a,
            icon: None,
            text: "Off",
            on: true,
        },
        Seg {
            id: b,
            icon: None,
            text: "On",
            on: false,
        },
    ]
}

/// ⭐ Foto `r4_physics_body`: Trigger + One-Way e Mass + Density desciam um por linha. Uma escolha de duas
/// opções é curta: divide a linha com a vizinha, e com um número. *Mutação: `corpo::choices` sem o pedido.*
#[test]
fn a_two_option_choice_shares_the_line_in_the_flow() {
    let mut f = mesa();
    corre_fluxo(&mut f, 257.0, |c, y| {
        let (a, b) = (duas_opcoes(A, B), duas_opcoes(C, NodeId(0x4b_0004)));
        let (k1, k2) = ([A, B], [C, NodeId(0x4b_0004)]);
        let y = corpo::choices(
            c.scene,
            c.text_system,
            c.store,
            c.hit_index,
            c.ink,
            c.x,
            c.w,
            y,
            "Trigger",
            &k1,
            &a,
        );
        corpo::choices(
            c.scene,
            c.text_system,
            c.store,
            c.hit_index,
            c.ink,
            c.x,
            c.w,
            y,
            "One-Way",
            &k2,
            &b,
        )
    });
    let (ra, rc) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(C).unwrap());
    assert_eq!(ra.y, rc.y, "as duas escolhas na mesma linha");
    assert!(rc.x > ra.x + ra.w);
}

/// ⭐ Lei 2: com três opções de texto a escolha fica com a linha inteira, e a seguinte começa depois.
/// *Mutação: `linhas.len() == 1` sem o `ESCOLHA_CURTA_MAX`.*
#[test]
fn a_three_option_choice_keeps_its_line() {
    let mut f = mesa();
    corre_fluxo(&mut f, 257.0, |c, y| {
        let tres = [
            Seg {
                id: A,
                icon: None,
                text: "Ball",
                on: true,
            },
            Seg {
                id: B,
                icon: None,
                text: "Box",
                on: false,
            },
            Seg {
                id: NodeId(0x4b_0005),
                icon: None,
                text: "Pill",
                on: false,
            },
        ];
        let (dois, k1, k2) = (
            duas_opcoes(C, NodeId(0x4b_0004)),
            [A, B, NodeId(0x4b_0005)],
            [C, NodeId(0x4b_0004)],
        );
        let y = corpo::choices(
            c.scene,
            c.text_system,
            c.store,
            c.hit_index,
            c.ink,
            c.x,
            c.w,
            y,
            "Shape",
            &k1,
            &tres,
        );
        corpo::choices(
            c.scene,
            c.text_system,
            c.store,
            c.hit_index,
            c.ink,
            c.x,
            c.w,
            y,
            "One-Way",
            &k2,
            &dois,
        )
    });
    assert!(f.hit.rect_for(C).unwrap().y > f.hit.rect_for(A).unwrap().y);
}

/// ⭐ O censo lê a escolha de duas opções como curta, e a fila sem nome de uma grelha como longa.
/// *Mutação: `curta` sem o braço de `Choice` ⇒ o par separado passa calado.*
#[test]
fn the_census_counts_a_two_option_choice_as_short_only_with_a_name() {
    let medir = |rotulo: bool| {
        let mut f = mesa();
        censo::na_secao(Some(A));
        let (_, c) = censo::grava(|| {
            let mut k = ctx(&mut f, 257.0);
            let (s1, s2) = (duas_opcoes(A, B), duas_opcoes(C, NodeId(0x4b_0004)));
            let celula = |s, nome| {
                let cel = Cell::new(Widget::Choice(s));
                if rotulo { cel.label(nome) } else { cel }
            };
            let y = k.row_inteira(&[celula(&s1, "Trigger")], 0.0);
            k.row_inteira(&[celula(&s2, "One-Way")], y);
        });
        censo::na_secao(None);
        c.irmaos_separados().len()
    };
    assert_eq!(medir(true), 1, "a escolha com nome é curta");
    assert_eq!(medir(false), 0, "a fila de uma grelha não tem vizinho");
}

/// O aviso de um painel conta na distância entre linhas. *Mutação: `frase` sem a linha.*
#[test]
fn a_phrase_counts_for_the_distance_between_rows() {
    censo::na_secao(Some(A));
    let (_, c) = censo::grava(|| {
        censo::frase((0.0, 100.0), 0.0, 10.0, "a");
        censo::frase((0.0, 100.0), 11.0, 20.0, "b");
    });
    censo::na_secao(None);
    assert_eq!(c.linhas_grudadas().len(), 1, "{:?}", c.linhas);
}

/// O corpo a abrir recorta o que passa da banda, e o pedido em espera pinta-se por fora do corpo:
/// não pode saltar à vista antes da dobra. *Mutação: tirar o recorte de `fluir` ⇒ vermelho.*
#[test]
fn what_waits_in_the_flow_stays_clipped_while_the_section_opens() {
    let pinta_a = |aberto: f32| {
        let mut f = mesa();
        f.store.set_section_open_live(A, aberto);
        let chave = crate::widget::chave_da_altura(A);
        f.store.remember_section_body_h(chave, 100.0);
        censo::na_secao(Some(A));
        corre_fluxo(&mut f, 257.0, |c, y| c.check(B, "Region", true, y));
        censo::na_secao(None);
        f.hit.rect_for(B).is_some()
    };
    assert!(pinta_a(1.0), "aberto, o pedido pinta-se");
    assert!(!pinta_a(0.1), "a abrir, o que não cabe na banda não se vê");
}

/// `property_row` lê o fluxo por `corpo_a_fluir`, abaixo do `panel`: ligado só durante o corpo.
/// *Mutação: `marca_fluxo(true)` a dizer falso ⇒ vermelho.*
#[test]
fn the_flow_is_visible_below_the_panel_only_while_the_body_runs() {
    use crate::widget::card_ink::corpo_a_fluir;
    let mut f = mesa();
    assert!(!corpo_a_fluir());
    let mut c = ctx(&mut f, 257.0);
    let mut dentro = false;
    let _ = fluir(&mut c, |_| {
        dentro = corpo_a_fluir();
        0.0
    });
    assert!(dentro, "o corpo corre com o fluxo ligado");
    assert!(!corpo_a_fluir(), "o fluxo desliga ao sair do corpo");
}

/// O leitor do fluxo: no corpo a fluir `paint_check_rows` empilha e deixa o par ao fluxo; fora dele, no
/// cartão, emparelha sozinha. *Mutação: `marca.rs` sem `corpo_a_fluir` ⇒ vermelho (sobrevivia antes deste).*
#[test]
fn check_rows_leave_the_pairing_to_the_flow_only_while_the_body_runs() {
    use crate::property_row::paint_check_rows;
    use crate::widget::{Seccao, card_ink};
    let caixas = [(A, "On", false), (B, "Up", false)];
    let pinta = |c: &mut CardCtx<'_>| {
        let (theme, w) = (Theme::default(), c.w);
        let sec = Seccao::medida(c.text_system, 1, &["On", "Up"]);
        paint_check_rows(
            c.scene,
            c.text_system,
            theme,
            c.hit_index,
            c.store,
            c.x,
            w,
            0.0,
            &caixas,
            sec,
        )
    };
    let mesma_fila = |fluindo: bool| {
        let mut f = mesa();
        f.scene.record();
        card_ink::com(CardInk::of_list(Theme::default()), || {
            let mut c = ctx(&mut f, 257.0);
            let _ = if fluindo {
                fluir(&mut c, pinta)
            } else {
                pinta(&mut c)
            };
        });
        f.hit.rect_for(A).unwrap().y == f.hit.rect_for(B).unwrap().y
    };
    assert!(mesma_fila(false), "fora do fluxo, o cartão emparelha");
    assert!(!mesma_fila(true), "no fluxo, quem emparelha é ele");
}

/// Numa coluna de duas, a marca da caixa anda com o nome: no fim da coluna lia-se da vizinha (05 §8).
/// *Mutação: `colada` sempre falso em `check_em` ⇒ vermelho.*
#[test]
fn the_mark_of_a_check_in_a_pair_walks_with_its_name() {
    let mut f = mesa();
    f.scene.record();
    let celula = |id, chave| {
        Cell::new(Widget::Check {
            id,
            label: Rotulo::from(ph2d_i18n::TextKey::new(chave)),
            value: CheckboxValue::Unchecked,
            off: false,
        })
    };
    {
        let mut c = ctx(&mut f, 257.0);
        c.row_inteira(
            &[celula(A, "card.bones.smart"), celula(B, "card.bones.ik")],
            0.0,
        );
    }
    let ra = f.hit.rect_for(A).unwrap();
    let (fim, k) = (f64::from(ra.x + ra.w), f64::from(m::KEY_MARK_BOX));
    let pontos: Vec<f64> = f
        .scene
        .take_record()
        .into_iter()
        .filter_map(|op| match op {
            ph2d_vector::DrawOp::Fill { path, .. } => {
                use ph2d_vector::Shape as _;
                let b = path.bounding_box();
                (b.width() <= k && b.height() <= k && b.x0 < fim && b.x0 > f64::from(ra.x))
                    .then_some(b.x1)
            }
            _ => None,
        })
        .collect();
    assert!(!pontos.is_empty(), "a marca não se pintou");
    assert!(
        pontos.iter().all(|x| *x < fim - k),
        "marca no fim da coluna: {pontos:?}, fim {fim}"
    );
}

/// ⭐ **A escolha num PAR ou num COMPACTO também quebra em filas** (auditoria 10/11, Paint Mixing: Pigment e Color mixing
/// num `pair`). Na coluna estreita o par desce um por linha — e a escolha sozinha que não cabe tem de quebrar como na
/// [`CardCtx::row`], em vez de cortar os segmentos. *Mutação: o par/compacto a pintar cada célula por `row_inteira`.*
#[test]
fn a_choice_alone_in_a_pair_or_compact_row_still_wraps() {
    let seg = |id, text| Seg {
        id,
        icon: None,
        text,
        on: false,
    };
    let segs = [
        seg(A, "Wet canvas"),
        seg(B, "Dry canvas"),
        seg(C, "Hand-packed"),
    ];
    for compacto in [false, true] {
        let mut f = mesa();
        let (_, censo) = censo::grava(|| {
            let mut c = ctx(&mut f, 160.0);
            let escolha = Cell::new(Widget::Choice(&segs)).label("Whole");
            let outro = campo(NodeId(0x4b_0009), "Amount");
            if compacto {
                c.compact(&[escolha, outro], 0.0)
            } else {
                c.pair(escolha, outro, 0.0)
            }
        });
        let (ra, rc) = (f.hit.rect_for(A).unwrap(), f.hit.rect_for(C).unwrap());
        assert!(
            rc.y > ra.y,
            "compacto={compacto}: a escolha não quebrou {ra:?} {rc:?}"
        );
        assert!(
            censo.textos_cortados().is_empty(),
            "compacto={compacto}: {:?}",
            censo.textos
        );
    }
}
