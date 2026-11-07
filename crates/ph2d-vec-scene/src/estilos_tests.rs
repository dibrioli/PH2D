use ph2d_item_id::ItemId;

use crate::{Paint, PaintEntry, PaintKind, Rgba8, StrokePaint, StrokeSpec, VecScene, rectangle};

const AZUL: Rgba8 = Rgba8::new(0x4C, 0x8D, 0xF0, 0xFF);
const LARANJA: Rgba8 = Rgba8::new(0xE8, 0x74, 0x3B, 0xFF);

/// Uma cena com duas formas, cada uma com um preenchimento (id 1) e um contorno (id 2).
fn cena() -> (VecScene, u64, u64) {
    let mut s = VecScene::new();
    let mut ids = Vec::new();
    for _ in 0..2 {
        let mut p = rectangle([0.0, 0.0], [10.0, 10.0]);
        let mut f = PaintEntry::fill(Paint::solid(Rgba8::new(0, 0, 0, 255)));
        f.id = ItemId(1);
        let mut t = PaintEntry::fill(Paint::solid(Rgba8::new(0, 0, 0, 255)));
        t.kind = PaintKind::Stroke(StrokeSpec::new(Rgba8::new(0, 0, 0, 255), 1.0));
        t.id = ItemId(2);
        p.paints = vec![f, t];
        ids.push(s.push_path(p));
    }
    (s, ids[0], ids[1])
}

fn cor(s: &VecScene, p: u64, item: u32) -> Option<Rgba8> {
    let e = s.path(p)?.paints.iter().find(|e| e.id == ItemId(item))?;
    match &e.kind {
        PaintKind::Fill(Paint::Solid(c)) => Some(*c),
        PaintKind::Stroke(StrokeSpec {
            paint: StrokePaint::Solid(c),
            ..
        }) => Some(*c),
        _ => None,
    }
}

/// ⭐ D30: editar o estilo muda TODA camada ligada (preenchimento e contorno, em formas diferentes);
/// a solta não muda.
#[test]
fn editing_a_style_recolours_every_linked_layer() {
    let (mut s, a, b) = cena();
    let id = s.add_estilo("Brand blue", AZUL);
    assert!(s.link_estilo((a, ItemId(1)), id));
    assert!(s.link_estilo((b, ItemId(2)), id));
    assert_eq!(cor(&s, a, 1), Some(AZUL), "ligar pinta com a cor do estilo");
    assert_eq!(s.usos_vivos(id), 2);
    assert!(s.set_estilo_color(id, LARANJA));
    assert_eq!(cor(&s, a, 1), Some(LARANJA));
    assert_eq!(cor(&s, b, 2), Some(LARANJA));
    assert_eq!(
        cor(&s, b, 1),
        Some(Rgba8::new(0, 0, 0, 255)),
        "a solta mudou"
    );
    // Soltar deixa a cor e corta a ligação.
    assert!(s.unlink_estilo((a, ItemId(1))));
    assert!(s.set_estilo_color(id, AZUL));
    assert_eq!(cor(&s, a, 1), Some(LARANJA));
    assert_eq!(cor(&s, b, 2), Some(AZUL));
}

/// Apagar o estilo deixa as camadas com a cor; uma camada vive num estilo só.
#[test]
fn deleting_a_style_keeps_the_colour_and_a_layer_lives_in_one_style() {
    let (mut s, a, _) = cena();
    let x = s.add_estilo("Ink", AZUL);
    let y = s.add_estilo("Paper", LARANJA);
    s.link_estilo((a, ItemId(1)), x);
    s.link_estilo((a, ItemId(1)), y);
    assert_eq!(s.estilo_de((a, ItemId(1))).map(|e| e.id), Some(y));
    assert_eq!(s.usos_vivos(x), 0);
    let d = s.duplicate_estilo(y, "Paper copy").unwrap();
    assert_eq!(s.estilos()[2].id, d, "a cópia vem logo depois do original");
    assert_eq!(s.estilo(d).unwrap().usos, vec![], "a cópia nasce sem usos");
    assert!(s.remove_estilo(y));
    assert_eq!(cor(&s, a, 1), Some(LARANJA));
    assert_eq!(s.estilo_de((a, ItemId(1))), None);
}

/// A forma viva (com estilos) volta igual pelo `to_bytes`/`from_bytes`.
#[test]
fn a_vec_scene_com_estilos_volta_igual() {
    let (s, _, _) = cena();
    let mut com = s.clone();
    let id = com.add_estilo("Grass", LARANJA);
    com.link_estilo((s.paths[0].id, ItemId(1)), id);
    let volta = VecScene::from_bytes(&com.to_bytes().unwrap()).unwrap();
    assert_eq!(volta, com, "a forma viva perde os estilos");
}

/// ⭐ Copiar/colar, Ctrl+D e o Duplicate da Hierarchy passam por `copy_paths` + `paste_clip`: a
/// cópia nasce LIGADA ao mesmo estilo (editar o estilo pinta as duas); a solta continua solta.
#[test]
fn paste_and_duplicate_keep_the_style_link() {
    let (mut s, a, _) = cena();
    let id = s.add_estilo("Brand blue", AZUL);
    assert!(s.link_estilo((a, ItemId(1)), id));
    let clip = s.copy_paths(&[a]);
    let novo = s.paste_clip(&clip, 5.0, 0.0)[0];
    assert_eq!(s.estilo_de((novo, ItemId(1))).map(|e| e.id), Some(id));
    assert!(s.estilo_de((novo, ItemId(2))).is_none(), "a solta ligou-se");
    assert!(s.set_estilo_color(id, LARANJA));
    assert_eq!(cor(&s, novo, 1), Some(LARANJA));
    assert_eq!(s.usos_vivos(id), 2);
    // O estilo apagado (ou outro que reuse o número) não religa a cópia.
    assert!(s.remove_estilo(id));
    let outro = s.paste_clip(&clip, 0.0, 5.0)[0];
    assert!(s.estilo_de((outro, ItemId(1))).is_none());
}

/// Uma forma com as duas tintas de base (preta) e nenhuma camada.
fn cena_de_base() -> (VecScene, u64) {
    let mut s = VecScene::new();
    let mut p = rectangle([0.0, 0.0], [10.0, 10.0]);
    p.fill = Some(Paint::Solid(PRETO));
    p.stroke = Some(StrokeSpec::new(PRETO, 1.0));
    p.paints.clear();
    let id = s.push_path(p);
    (s, id)
}

const PRETO: Rgba8 = Rgba8::new(0, 0, 0, 255);
const BF: ItemId = ItemId::BASE_FILL;
const BS: ItemId = ItemId::BASE_STROKE;

fn bases(s: &VecScene, p: u64) -> (Option<Rgba8>, Option<Rgba8>) {
    let f = s.path(p).unwrap();
    (
        f.cor_da_linha(crate::LINHA_BASE_FILL),
        f.cor_da_linha(crate::LINHA_BASE_STROKE),
    )
}

/// ⭐ Editar o estilo repinta as DUAS tintas de base, também escondidas; `fill`/`stroke` continuam
/// `None` (o olho fica fechado) e a alfa fica a da base (a Opacity da linha).
#[test]
fn editing_a_style_repaints_the_base_paints_even_hidden() {
    let (mut s, a) = cena_de_base();
    let id = s.add_estilo("Brand blue", AZUL);
    assert!(s.link_estilo((a, BF), id));
    assert!(s.link_estilo((a, BS), id));
    assert_eq!(bases(&s, a), (Some(AZUL), Some(AZUL)), "ligar não pintou");
    assert_eq!(s.usos_vivos(id), 2);
    assert!(s.set_estilo_color(id, LARANJA));
    assert_eq!(bases(&s, a), (Some(LARANJA), Some(LARANJA)));
    let p = s.path_mut(a).unwrap();
    assert!(p.set_fill_hidden(true) && p.set_stroke_hidden(true));
    assert_eq!(s.usos_vivos(id), 2, "a escondida deixou de contar");
    assert!(s.set_estilo_color(id, AZUL));
    assert!(!s.solta_bases_mexidas(), "esconder soltou");
    let p = s.path(a).unwrap();
    assert!(p.fill.is_none() && p.stroke.is_none(), "o olho abriu");
    assert_eq!(
        bases(&s, a),
        (Some(AZUL), Some(AZUL)),
        "a escondida não mudou"
    );
    // A alfa é a da base: repintar e voltar a ligar mantêm-na.
    let meia = Rgba8::new(AZUL.r, AZUL.g, AZUL.b, 128);
    s.path_mut(a).unwrap().fill_hidden = Some(Paint::Solid(meia));
    assert!(!s.solta_bases_mexidas(), "a Opacity soltou o estilo");
    assert!(s.set_estilo_color(id, LARANJA));
    let esperada = Rgba8::new(LARANJA.r, LARANJA.g, LARANJA.b, 128);
    assert_eq!(bases(&s, a).0, Some(esperada));
}

/// Apagar o estilo deixa a base com a cor, solta.
#[test]
fn deleting_a_style_keeps_the_base_colour() {
    let (mut s, a) = cena_de_base();
    let id = s.add_estilo("Ink", AZUL);
    s.link_estilo((a, BF), id);
    assert!(s.remove_estilo(id));
    assert_eq!(bases(&s, a).0, Some(AZUL));
    assert_eq!(s.estilo_de((a, BF)), None);
}

/// ⭐ Copiar/colar leva a ligação da base: a cópia nasce ligada e o estilo pinta as duas.
#[test]
fn paste_keeps_the_link_of_the_base_paints() {
    let (mut s, a) = cena_de_base();
    let id = s.add_estilo("Brand blue", AZUL);
    assert!(s.link_estilo((a, BF), id));
    let clip = s.copy_paths(&[a]);
    let novo = s.paste_clip(&clip, 5.0, 0.0)[0];
    assert_eq!(s.estilo_de((novo, BF)).map(|e| e.id), Some(id));
    assert!(s.estilo_de((novo, BS)).is_none(), "a base solta ligou-se");
    assert!(s.set_estilo_color(id, LARANJA));
    assert_eq!(bases(&s, novo).0, Some(LARANJA));
    assert_eq!(s.usos_vivos(id), 2);
}

/// A base ligada sobrevive ao ficheiro, sem degrau novo: o `Uso` já era `(path, ItemId)`.
#[test]
fn a_linked_base_round_trips_through_bytes() {
    let (mut s, a) = cena_de_base();
    let id = s.add_estilo("Grass", LARANJA);
    s.link_estilo((a, BF), id);
    s.link_estilo((a, BS), id);
    let volta = VecScene::from_bytes(&s.to_bytes().unwrap()).unwrap();
    assert_eq!(volta, s);
    assert_eq!(volta.estilo_de((a, BS)).map(|e| e.id), Some(id));
}

/// ⭐ D30: o que mexe na base por fora solta-a; o estilo a repintar e a Opacity não. Uma base que
/// sai mata o uso: ao voltar ela nasce solta.
#[test]
fn a_touched_base_detaches_and_a_gone_base_kills_the_use() {
    let (mut s, a) = cena_de_base();
    let id = s.add_estilo("Brand blue", AZUL);
    let ligar = |s: &mut VecScene| {
        s.link_estilo((a, BF), id);
        s.link_estilo((a, BS), id);
    };
    ligar(&mut s);
    assert!(s.set_estilo_color(id, LARANJA));
    assert!(!s.solta_bases_mexidas(), "o estilo a repintar soltou");

    // Outra cor à mão: só a tocada se solta.
    s.path_mut(a).unwrap().fill = Some(Paint::Solid(PRETO));
    assert!(s.solta_bases_mexidas());
    assert_eq!(s.estilo_de((a, BF)), None, "a cor à mão não soltou o Fill");
    assert!(s.estilo_de((a, BS)).is_some(), "o Stroke não foi tocado");
    assert_eq!(bases(&s, a).0, Some(PRETO), "soltar mudou a cor");

    // O traço, à mão.
    s.path_mut(a).unwrap().stroke.as_mut().unwrap().paint = StrokePaint::Solid(PRETO);
    assert!(s.solta_bases_mexidas());
    assert_eq!(
        s.estilo_de((a, BS)),
        None,
        "a cor à mão não soltou o Stroke"
    );

    // Um degradê à mão também é deixar de ser a cor do estilo.
    ligar(&mut s);
    let p = s.path_mut(a).unwrap();
    p.fill = Some(Paint::Linear {
        stops: Vec::new(),
        start: [0.0, 0.0],
        end: [1.0, 0.0],
    });
    assert!(s.solta_bases_mexidas());
    assert_eq!(s.estilo_de((a, BF)), None, "o degradê não soltou");

    // O «−» da base: o uso morre, deixa de contar e não acorda quando a tinta volta.
    ligar(&mut s);
    assert!(s.path_mut(a).unwrap().remove_base_stroke());
    assert_eq!(s.usos_vivos(id), 1, "a base que saiu ainda conta");
    assert!(s.solta_bases_mexidas());
    s.path_mut(a).unwrap().stroke = Some(StrokeSpec::new(LARANJA, 1.0));
    assert_eq!(s.estilo_de((a, BS)), None, "o uso morto acordou");

    // A forma que saiu leva os usos de base.
    assert!(s.remove_path(a));
    assert!(s.solta_bases_mexidas());
    assert!(s.estilo(id).unwrap().usos.is_empty());
    assert!(!s.solta_bases_mexidas(), "a segunda passada não é neutra");
}

/// Os ids das bases nunca são «por atribuir» nem iguais entre si, e nenhuma camada os recebe
/// (o alocador: `ph2d_ecs::stable_id`, `o_alocador_nunca_entrega_um_id_reservado`).
#[test]
fn the_reserved_ids_are_distinct_and_no_entry_answers_for_them() {
    assert!(BF != BS && !BF.is_none() && !BS.is_none());
    assert!(BF.reservado() && BS.reservado());
    assert!(!ItemId(BS.0 - 1).reservado() && !ItemId::NONE.reservado());
    let (s, a, _) = cena();
    assert_eq!(
        s.path(a).unwrap().item_da_linha(crate::LINHA_BASE_FILL),
        None
    );
    assert_eq!(s.path(a).unwrap().item_da_linha(0), Some(ItemId(1)));
    assert_eq!(s.path(a).unwrap().item_da_linha(2), None);
}

/// Uma base sem alfa (a «sem preenchimento») ganha a do estilo ao LIGAR, como a escolha no seletor;
/// a Opacity a zero numa base já ligada fica a zero quando o estilo repinta.
#[test]
fn linking_shows_a_base_without_alpha_and_a_repaint_keeps_the_zero() {
    let (mut s, a) = cena_de_base();
    s.path_mut(a).unwrap().fill = Some(Paint::Solid(Rgba8::new(0, 0, 0, 0)));
    let id = s.add_estilo("Brand blue", AZUL);
    assert!(s.link_estilo((a, BF), id));
    assert_eq!(bases(&s, a).0, Some(AZUL), "ligar deixou a base invisível");
    s.path_mut(a).unwrap().fill = Some(Paint::Solid(Rgba8::new(AZUL.r, AZUL.g, AZUL.b, 0)));
    assert!(s.set_estilo_color(id, LARANJA));
    let esperada = Rgba8::new(LARANJA.r, LARANJA.g, LARANJA.b, 0);
    assert_eq!(bases(&s, a).0, Some(esperada), "repintar acendeu a base");
}
