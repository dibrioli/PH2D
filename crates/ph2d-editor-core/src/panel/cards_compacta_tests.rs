use super::*;
use crate::panel::cards::censo;
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
        censo::fora_da_linha("Equalize", 10.0, crate::zones::Rect::new(0.0, y, 257.0, 28.0));
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
