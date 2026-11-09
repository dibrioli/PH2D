use super::*;
use ph2d_asset::AssetDb;
use ph2d_ecs::{ExtraSheet, FrameRef, ShownSheet};
use ph2d_editor_core::ImageUseEdit;
use ph2d_render::{SheetTextures, SpriteSource, drawn_geometry, sheet_geometry};
use std::collections::BTreeMap;

const PPM: f32 = 32.0;

fn run() -> ExtraSheet {
    ExtraSheet {
        asset: ph2d_asset::AssetId::from_bytes(&[2]),
        size: [1.5, 1.5],
        anchor: [0.0, 0.75],
        hframes: 6,
        vframes: 2,
        offsets: CellOffsets::default(),
    }
}

/// `idle` (4 × 1, folha 0, os pés no pivô) e `run` (6 × 2, folha 1, maior, os pés no pivô).
fn duas() -> (World, Entity) {
    let mut w = World::new();
    let e = w
        .spawn((
            Sprite {
                anchor: [0.0, 0.5],
                ..Sprite::atlas(0, [1.0, 1.0], [1.0; 4])
            },
            SpriteGrid::new(4, 1),
            SpriteSheets([run()].into_iter().collect()),
        ))
        .id();
    (w, e)
}

fn mostra(w: &mut World, e: Entity, sheet: u8, cell: u32) {
    w.get_mut::<SpriteGrid>(e)
        .unwrap()
        .show(FrameRef::Cell { sheet, cell });
}

/// O pé do quad desenhado (a base, no meio), em metros locais a partir do pivô.
fn pe(w: &World, e: Entity) -> [f32; 2] {
    let (s, _) = drawn_geometry(w, e).unwrap();
    let a = s.resolve_anchor(PPM);
    [a[0], a[1] - s.size[1] * 0.5]
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5
}

/// ⭐ **A escala de uma folha é em volta do Pivot: os pés ficam** (GEF 28 A). A `run` 1,5× maior volta ao tamanho da
/// `idle` sem sair do chão; os acertos dela escalam junto; a `idle` não muda.
/// *Mata: escalar só o tamanho (os pés sobem) · não escalar os acertos · escalar a folha errada · aceitar `0`/NaN/`1`.*
#[test]
fn escalar_a_folha_deixa_os_pes_no_lugar() {
    let (mut w, e) = duas();
    mostra(&mut w, e, 1, 3);
    assert!(nudge_cell(&mut w, e, 1, 3, [0.3, 0.0]));
    let antes = pe(&w, e);
    assert!(scale_sheet(&mut w, e, 1, 2.0 / 3.0));
    let (s, _) = drawn_geometry(&w, e).unwrap();
    assert!(perto(s.size, [1.0, 1.0]), "{:?}", s.size);
    assert!(
        perto(pe(&w, e), [antes[0] * 2.0 / 3.0, antes[1]]),
        "os pés no chão"
    );
    assert!(
        perto(ph2d_ecs::cell_offset(&w, e, 1, 3), [0.2, 0.0]),
        "o acerto escala"
    );
    mostra(&mut w, e, 0, 0);
    assert!(
        perto(drawn_geometry(&w, e).unwrap().0.size, [1.0, 1.0]),
        "a idle fica"
    );
    for f in [0.0, -1.0, f32::NAN, f32::INFINITY, 1.0] {
        assert!(!scale_sheet(&mut w, e, 1, f), "{f}");
    }
    assert!(!scale_sheet(&mut w, e, 2, 2.0), "folha que não existe");
}

/// A folha `0` escala com o canto e o `offset` em píxeis: o Pivot RESOLVIDO escala, o pivô fica.
/// *Mata: esquecer o `offset` (o desenho anda) · esquecer o `CellOffsets` da folha 0.*
#[test]
fn escalar_a_folha_0_escala_o_pivot_resolvido() {
    let (mut w, e) = duas();
    {
        let mut s = w.get_mut::<Sprite>(e).unwrap();
        s.centered = false;
        s.offset = [4.0, -2.0];
    }
    assert!(nudge_cell(&mut w, e, 0, 1, [0.0, 0.1]));
    let a = w.get::<Sprite>(e).unwrap().resolve_anchor(PPM);
    assert!(scale_sheet(&mut w, e, 0, 2.0));
    let b = w.get::<Sprite>(e).unwrap().resolve_anchor(PPM);
    assert!(perto(b, [a[0] * 2.0, a[1] * 2.0]), "{a:?} → {b:?}");
    assert!(perto(ph2d_ecs::cell_offset(&w, e, 0, 1), [0.0, 0.2]));
}

/// ⭐ **Mover a folha anda o desenho e deixa o objeto** (o contrário do Pivot): a origem e as outras folhas ficam.
/// *Mata: mover a origem junto · mover a folha 0 quando se pede a 1.*
#[test]
fn mover_a_folha_anda_o_desenho_e_deixa_o_objeto() {
    let (mut w, e) = duas();
    let t = ph2d_ecs::Transform::default();
    w.entity_mut(e).insert(t);
    assert!(move_sheet(&mut w, e, 1, [0.25, -0.5]));
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [0.25, 0.25]);
    assert_eq!(
        w.get::<Sprite>(e).unwrap().anchor,
        [0.0, 0.5],
        "a idle fica"
    );
    assert_eq!(
        *w.get::<ph2d_ecs::Transform>(e).unwrap(),
        t,
        "o objeto fica"
    );
    assert!(move_sheet(&mut w, e, 0, [0.0, 0.1]));
    assert!(perto(w.get::<Sprite>(e).unwrap().anchor, [0.0, 0.6]));
    assert!(!move_sheet(&mut w, e, 0, [0.0, 0.0]));
    assert!(!move_sheet(&mut w, e, 0, [f32::NAN, 0.0]));
}

/// ⭐ **O acerto de um quadro desloca SÓ esse quadro, no ecrã e na caixa do gizmo** — e não a folha que o Pivot e o
/// Trim editam (`sheet_geometry`). Voltar a zero apaga a entrada; na folha 0 tira o componente.
/// *Mata: o acerto entrar no `sheet_geometry` (o Pivot gravaria o acerto como seu) · aplicar a todas as células ·
/// esquecer o `drawn_geometry` · aceitar uma célula fora da folha.*
#[test]
fn o_acerto_desloca_so_o_seu_quadro() {
    let (mut w, e) = duas();
    let d = [2.0 / PPM, -1.0 / PPM];
    assert!(nudge_cell(&mut w, e, 1, 7, d));
    mostra(&mut w, e, 1, 7);
    let base = sheet_geometry(&w, e, 1, 7).unwrap().0.anchor;
    assert_eq!(base, run().anchor, "a folha não leva o acerto");
    let desenhada = drawn_geometry(&w, e).unwrap().0.anchor;
    assert!(perto(desenhada, [base[0] + d[0], base[1] + d[1]]));
    mostra(&mut w, e, 1, 6);
    assert_eq!(
        drawn_geometry(&w, e).unwrap().0.anchor,
        base,
        "a vizinha fica"
    );
    assert!(!nudge_cell(&mut w, e, 1, 12, d), "6 × 2 = 12 células");
    assert!(!nudge_cell(&mut w, e, 0, 4, d), "4 × 1");
    assert!(nudge_cell(&mut w, e, 1, 7, [-d[0], -d[1]]));
    assert!(w.get::<SpriteSheets>(e).unwrap().0[0].offsets.is_empty());
    assert!(nudge_cell(&mut w, e, 0, 2, d));
    assert!(w.get::<CellOffsets>(e).is_some());
    assert!(nudge_cell(&mut w, e, 0, 2, [-d[0], -d[1]]));
    assert!(w.get::<CellOffsets>(e).is_none(), "zero tira o componente");
}

/// ⭐ **O acerto vale só no corte em que foi feito**: recortar a folha (outra grade) apaga-o na prática, e voltar ao
/// mesmo corte devolve-o — nenhum caminho de recorte precisa de se lembrar dele.
/// *Mata: `of` ignorar o corte (o quadro 3 de uma grade nova herdaria o acerto de outro desenho).*
#[test]
fn o_acerto_vale_so_no_corte_em_que_foi_feito() {
    let (mut w, e) = duas();
    let d = [0.1, 0.0];
    assert!(nudge_cell(&mut w, e, 0, 3, d));
    assert!(nudge_cell(&mut w, e, 1, 3, d));
    w.get_mut::<SpriteGrid>(e).unwrap().hframes = 2;
    w.get_mut::<SpriteSheets>(e).unwrap().0[0].hframes = 3;
    assert_eq!(ph2d_ecs::cell_offset(&w, e, 0, 3), [0.0, 0.0]);
    assert_eq!(ph2d_ecs::cell_offset(&w, e, 1, 3), [0.0, 0.0]);
    // Acertar no corte novo recomeça o mapa.
    assert!(nudge_cell(&mut w, e, 1, 1, d));
    let o = &w.get::<SpriteSheets>(e).unwrap().0[0].offsets;
    assert_eq!((o.cut, o.cells.len()), ([3, 2], 1));
    w.get_mut::<SpriteGrid>(e).unwrap().hframes = 4;
    assert_eq!(ph2d_ecs::cell_offset(&w, e, 0, 3), d, "o corte de volta");
}

/// ⭐ **Apagar a folha 0 promove a 1 com os acertos dela**, e o da folha apagada vai-se.
/// *Mata: a promoção esquecer os acertos · deixar o `CellOffsets` da folha apagada.*
#[test]
fn a_folha_promovida_leva_os_acertos() {
    let (mut w, e) = duas();
    assert!(nudge_cell(&mut w, e, 0, 0, [0.5, 0.0]));
    assert!(nudge_cell(&mut w, e, 1, 5, [0.0, 0.25]));
    let textura = SpriteSource::Individual { texture_id: 9 };
    w.entity_mut(e).insert((
        SheetTextures(vec![Some((run().asset, textura))]),
        ShownSheet(0),
    ));
    assert!(crate::apply::delete_shown_sheet(
        &mut w,
        e,
        &BTreeMap::new()
    ));
    assert_eq!(ph2d_ecs::cell_offset(&w, e, 0, 5), [0.0, 0.25]);
    assert_eq!(ph2d_ecs::cell_offset(&w, e, 0, 0), [0.0, 0.0]);
}

/// Os três verbos chegam pelo `apply` do cartão: escala e posição sobre a folha VISÍVEL, o acerto sobre a folha dita.
/// *Mata: o `apply` mandar os verbos para a folha 0 sempre · devolver `None` (o shell não os veria).*
#[test]
fn os_verbos_do_align_agem_na_folha_visivel() {
    let (mut w, e) = duas();
    w.entity_mut(e).insert(ShownSheet(1));
    let db = AssetDb::default();
    let apply = |w: &mut World, edit| crate::apply::apply(w, e, &db, &BTreeMap::new(), PPM, edit);
    assert_eq!(apply(&mut w, ImageUseEdit::ScaleSheet(2.0)), Some(true));
    assert_eq!(
        apply(&mut w, ImageUseEdit::MoveSheet([0.5, 0.0])),
        Some(true)
    );
    assert_eq!(
        apply(
            &mut w,
            ImageUseEdit::NudgeCell {
                sheet: 1,
                cell: 1,
                by: [0.0, 0.5]
            }
        ),
        Some(true)
    );
    let x = &w.get::<SpriteSheets>(e).unwrap().0[0];
    assert_eq!((x.size, x.anchor), ([3.0, 3.0], [0.5, 1.5]));
    assert_eq!(x.offsets.of(x.cut(), 1), [0.0, 0.5]);
    assert_eq!(
        w.get::<Sprite>(e).unwrap().size,
        [1.0, 1.0],
        "a folha 0 fica"
    );
}
