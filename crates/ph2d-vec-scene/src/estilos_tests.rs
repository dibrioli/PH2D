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

/// ⭐ Gate do degrau v29: os bytes de uma cena v28 (sem estilos) lêem-se iguais e sem estilos; e o
/// controlo — os mesmos bytes lidos pela forma viva NÃO abrem.
#[test]
fn a_vec_scene_v28_le_sem_estilos() {
    let (s, _, _) = cena();
    let v28 =
        postcard::to_allocvec(&(28u32, (&s.paths, s.next_id, &s.regras, s.next_regra))).unwrap();
    // O âmbito grava a mesma forma (é como os testes do projeto fabricam um degrau antigo).
    let pelo_ambito = crate::with_wire_layout(28, || postcard::to_allocvec(&(28u32, &s)).unwrap());
    assert_eq!(pelo_ambito, v28);
    let lida = VecScene::from_bytes(&v28).expect("um v28 abre");
    assert_eq!(lida, s);
    assert!(lida.estilos().is_empty());
    assert!(
        postcard::from_bytes::<(u32, VecScene)>(&v28).is_err(),
        "sem o âmbito o v28 leu-se — o gate é cego"
    );
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
