//! **As leis das slices** (P11): keys por quadro (T5, F3), a slice segue a arte (`P11_plano.md` §4.4),
//! geometria (T7), formato. Números: `crates/ph2d-pixel-io/tests/fixtures/oracle_slices_ferramenta.json`.

use std::collections::BTreeMap;

use ph2d_pixel_core::canvas::{self, Borders, SizeMethod};
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::edit::Rotation;
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::slice;
use ph2d_pixel_core::{
    ColorMode, FrameId, NewSprite, Op, Patch, Rect, Slice, SliceId, SliceKey, SliceShape, Sprite,
    apply, compose_rgba,
};

fn sprite(w: u32, h: u32, n_frames: usize) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        ..NewSprite::default()
    })
    .expect("nasce");
    for _ in 1..n_frames {
        let last = s.frames().last().expect("quadro").id;
        let (op, _) = frames::new_frame(&mut s, last, NewFrame::Empty).expect("novo");
        apply(&mut s, op).expect("aplica");
    }
    s
}

fn caixa(x: i32, y: i32, w: u32, h: u32) -> SliceShape {
    SliceShape {
        bounds: Rect::new(x, y, w, h),
        ..SliceShape::default()
    }
}

/// Uma slice com keys nos quadros dados, escrita pela porta única.
fn com_keys(s: &mut Sprite, keys: &[(u32, SliceShape)]) -> SliceId {
    let (op, id) = slice::create(s, keys[0].0, Rect::new(0, 0, 1, 1)).expect("cria");
    apply(s, op).expect("aplica");
    for (f, k) in keys {
        if let Some(op) = slice::set_shape(s, id, *f, *k) {
            apply(s, op).expect("aplica");
        }
    }
    id
}

fn so_a(s: &Sprite, id: SliceId) -> &Slice {
    s.slice(id).expect("a slice")
}

/// O valor em cada quadro, por identidade do quadro.
fn por_quadro(s: &Sprite, id: SliceId) -> BTreeMap<FrameId, SliceShape> {
    s.frames()
        .iter()
        .enumerate()
        .map(|(i, f)| (f.id, so_a(s, id).shape_at(i as u32).expect("tem valor")))
        .collect()
}

/// `keys.ase` do Trial: 0 `8,8`, 2 `24,8`, 4 `24,24,32,16`.
fn moves() -> (Sprite, SliceId) {
    let mut s = sprite(64, 64, 6);
    let id = com_keys(
        &mut s,
        &[
            (0, caixa(8, 8, 16, 16)),
            (2, caixa(24, 8, 16, 16)),
            (4, caixa(24, 24, 32, 16)),
        ],
    );
    (s, id)
}

#[test]
fn a_key_lasts_until_the_next_and_the_first_also_covers_what_comes_before() {
    let (s, id) = moves();
    let c = so_a(&s, id);
    let x = |f| c.shape_at(f).expect("valor").bounds.x;
    assert_eq!([x(0), x(1), x(2), x(3)], [8, 8, 24, 24]);
    assert_eq!(
        c.shape_at(5).expect("valor").bounds,
        Rect::new(24, 24, 32, 16)
    );

    // F3 `late`: a 1.ª key no quadro 2 vale também em 0 e 1; `farframe`: key além do fim vale em todos.
    let late = Slice {
        keys: vec![SliceKey {
            frame: 2,
            shape: caixa(8, 8, 16, 16),
        }],
        ..c.clone()
    };
    assert_eq!(late.shape_at(0), late.shape_at(2));
    let far = Slice {
        keys: vec![SliceKey {
            frame: 99,
            shape: caixa(40, 8, 8, 8),
        }],
        ..c.clone()
    };
    assert_eq!(far.shape_at(5).expect("valor").bounds.x, 40);
}

#[test]
fn an_empty_box_hides_until_the_next_key_even_with_only_one_side_at_zero() {
    let mut s = sprite(64, 64, 6);
    let id = com_keys(
        &mut s,
        &[
            (0, caixa(8, 8, 16, 16)),
            (2, caixa(0, 0, 0, 16)),
            (3, caixa(0, 0, 16, 0)),
            (4, caixa(32, 32, 16, 16)),
        ],
    );
    let c = so_a(&s, id);
    let vis: Vec<bool> = (0..6).map(|f| c.visible_at(f).is_some()).collect();
    assert_eq!(vis, [true, true, false, false, true, true]);
}

/// T5: editar num quadro sem key cria key só lá; sem mudança não há passo; ⛔ o desfazer tira a key
/// (o Trial deixa-a, e a edição seguinte deixa de chegar: defeito medido que não copiamos).
#[test]
fn an_edit_creates_a_key_only_on_its_frame_and_undo_takes_the_key_away() {
    let (mut s, id) = moves();
    let antes = s.clone();
    assert_eq!(
        slice::set_shape(&s, id, 3, caixa(24, 8, 16, 16)),
        None,
        "OK sem mudar"
    );
    let op = slice::set_shape(&s, id, 3, caixa(30, 12, 16, 16)).unwrap();
    let inv = apply(&mut s, op).unwrap();
    let c = so_a(&s, id);
    let frames: Vec<u32> = c.keys.iter().map(|k| k.frame).collect();
    assert_eq!(frames, [0, 2, 3, 4]);
    assert_eq!(c.shape_at(2).unwrap().bounds.x, 24, "o anterior intacto");
    assert_eq!(c.shape_at(4).unwrap().bounds.y, 24, "o seguinte intacto");
    apply(&mut s, inv).unwrap();
    assert_eq!(s, antes, "o desfazer volta tudo");
    assert!(!so_a(&s, id).has_key_at(3));
}

#[test]
fn creating_clips_to_the_canvas_and_names_never_reuse_a_deleted_top_gap() {
    let mut s = sprite(64, 64, 1);
    assert_eq!(
        slice::create(&mut s, 0, Rect::new(70, 70, 4, 4)),
        None,
        "fora do canvas"
    );
    let novo = |s: &mut Sprite, r| {
        let (op, id) = slice::create(s, 0, r).expect("cria");
        apply(s, op).unwrap();
        id
    };
    let a = novo(&mut s, Rect::new(50, 40, 20, 30));
    assert_eq!(
        so_a(&s, a).keys[0].shape.bounds,
        Rect::new(50, 40, 14, 24),
        "T1"
    );
    assert_eq!(so_a(&s, a).name, "Slice 1");
    assert_eq!(so_a(&s, a).color, Some([0, 0, 255, 255]), "T1 #0000FF");
    let b = novo(&mut s, Rect::new(0, 0, 2, 2));
    let c = novo(&mut s, Rect::new(0, 0, 2, 2));
    let d = novo(&mut s, Rect::new(0, 0, 2, 2));
    assert_eq!(so_a(&s, d).name, "Slice 4");
    let op = slice::delete(&s, &[b]).unwrap();
    apply(&mut s, op).unwrap();
    assert!(s.slice(b).is_none());
    assert!(s.slice(c).is_some());
    let e = novo(&mut s, Rect::new(0, 0, 2, 2));
    assert_eq!(so_a(&s, e).name, "Slice 5", "T1: 1,3,4 → 5");
}

#[test]
fn nine_slice_starts_with_a_margin_of_one_and_never_without_a_middle() {
    assert_eq!(
        slice::default_center(Rect::new(4, 4, 24, 24)),
        Rect::new(1, 1, 22, 22)
    );
    assert_eq!(
        slice::default_center(Rect::new(0, 0, 15, 9)),
        Rect::new(1, 1, 13, 7)
    );
    assert_eq!(
        slice::default_center(Rect::new(0, 0, 2, 9)),
        Rect::new(0, 0, 2, 9)
    );
}

#[test]
fn removing_a_change_gives_the_frame_the_previous_value_but_the_last_key_stays() {
    let (mut s, id) = moves();
    assert_eq!(slice::remove_key(&s, id, 1), None, "sem key neste quadro");
    let op = slice::remove_key(&s, id, 2).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).shape_at(3).unwrap().bounds.x, 8);
    let op = slice::remove_key(&s, id, 0).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(
        so_a(&s, id).shape_at(0).unwrap().bounds.x,
        24,
        "a 1.ª que resta cobre o início"
    );
    assert_eq!(slice::remove_key(&s, id, 4), None, "a última não sai");
}

#[test]
fn duplicate_is_named_copy_offset_by_two_with_every_key() {
    let (mut s, id) = moves();
    let (op, copia) = slice::duplicate(&mut s, id).unwrap();
    apply(&mut s, op).unwrap();
    let c = so_a(&s, copia);
    assert_eq!(c.name, "Slice 1 Copy");
    assert_eq!(c.keys.len(), 3);
    assert_eq!(c.keys[1].shape.bounds.x, 26);
}

// --- Os quadros: a slice segue a arte ---

/// Depois de `op`, cada quadro que ficou mantém o seu valor; o desfazer repõe as keys exatas.
fn segue_a_arte(s: &Sprite, id: SliceId, op: Op) -> Sprite {
    let antes = por_quadro(s, id);
    let mut depois = s.clone();
    let inv = apply(&mut depois, op).expect("aplica");
    for (f, v) in por_quadro(&depois, id) {
        if let Some(era) = antes.get(&f) {
            assert_eq!(v, *era, "o quadro {f:?} perdeu o valor");
        }
    }
    let k = &so_a(&depois, id).keys;
    assert!(
        k.windows(2)
            .all(|w| w[0].frame < w[1].frame && w[0].shape != w[1].shape)
    );
    let mut volta = depois.clone();
    apply(&mut volta, inv).expect("desfaz");
    assert_eq!(&volta, s, "o desfazer volta as keys exatas");
    depois
}

#[test]
fn deleting_a_frame_keeps_every_remaining_frame_on_its_own_box() {
    let (s, id) = moves();
    for i in 0..6 {
        let f = s.frames()[i].id;
        let d = segue_a_arte(&s, id, frames::delete(&s, &[f]).unwrap());
        assert_eq!(d.frames().len(), 5);
    }
    // O quadro 2 tem key e o 3 herda dela: ela passa ao seguinte, em vez de o 3 cair na anterior.
    let f2 = s.frames()[2].id;
    let d = segue_a_arte(&s, id, frames::delete(&s, &[f2]).unwrap());
    let x: Vec<i32> = (0..5)
        .map(|i| so_a(&d, id).shape_at(i).unwrap().bounds.x)
        .collect();
    assert_eq!(x, [8, 8, 24, 24, 24]);
}

#[test]
fn inserting_a_frame_pushes_later_keys_and_the_new_frame_inherits() {
    let (mut s, id) = moves();
    let f1 = s.frames()[1].id;
    let mut c = s.clone();
    let (op, novo) = frames::new_frame(&mut c, f1, NewFrame::Empty).unwrap();
    s = c;
    let d = segue_a_arte(&s, id, op);
    assert_eq!(d.frames().len(), 7);
    let at = d.frame_index(novo).unwrap() as u32;
    assert_eq!(at, 2);
    assert_eq!(
        so_a(&d, id).shape_at(at).unwrap().bounds.x,
        8,
        "herda o anterior"
    );
    let k: Vec<u32> = so_a(&d, id).keys.iter().map(|k| k.frame).collect();
    assert_eq!(
        k,
        [0, 3, 5],
        "as seguintes andam 1: o pedido de aseprite#1469"
    );

    // Um quadro novo na posição 0 herda o seguinte.
    let mut z = s.clone();
    let f0 = z.frames()[0].id;
    let (op, novo) = frames::insert(&mut z, 0, f0, NewFrame::Empty, 100).unwrap();
    let d = segue_a_arte(&z, id, op);
    assert_eq!(d.frame_index(novo), Some(0));
    assert_eq!(so_a(&d, id).shape_at(0).unwrap().bounds.x, 8);
}

#[test]
fn moving_and_reversing_frames_carry_each_box_with_its_frame() {
    let (s, id) = moves();
    let f1 = s.frames()[1].id;
    let d = segue_a_arte(&s, id, frames::move_to(&s, f1, 3).unwrap());
    let x: Vec<i32> = (0..6)
        .map(|i| so_a(&d, id).shape_at(i).unwrap().bounds.x)
        .collect();
    assert_eq!(x, [8, 24, 24, 8, 24, 24]);

    let f = [s.frames()[0].id, s.frames()[1].id];
    let d = segue_a_arte(&s, id, frames::move_block(&s, &f, 6).unwrap());
    assert_eq!(so_a(&d, id).shape_at(5).unwrap().bounds.x, 8);

    let d = segue_a_arte(&s, id, frames::reverse(&s, 0, 5).unwrap());
    let x: Vec<i32> = (0..6)
        .map(|i| so_a(&d, id).shape_at(i).unwrap().bounds.x)
        .collect();
    assert_eq!(x, [24, 24, 24, 24, 8, 8]);
}

/// Mudar uma duração ou uma tag não mexe na ordem: as keys ficam exatas, mesmo as de ficheiro alheio.
#[test]
fn a_frame_op_that_keeps_the_order_leaves_the_keys_untouched() {
    let (mut s, id) = moves();
    let mut lista = s.slices().to_vec();
    lista[0].keys[0].frame = 1;
    apply(&mut s, Op::Slices(lista)).unwrap();
    let antes = so_a(&s, id).keys.clone();
    let ids = [s.frames()[0].id];
    let op = frames::set_duration(&s, &ids, 250).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).keys, antes);
}

// --- A geometria: a slice segue a arte ---

const VERMELHO: [u8; 4] = [255, 0, 0, 255];
const VERDE: [u8; 4] = [0, 255, 0, 255];
const AZUL: [u8; 4] = [0, 0, 255, 255];

fn pinta(s: &mut Sprite, x: i32, y: i32, cor: [u8; 4]) {
    let r = Rect::new(x, y, 1, 1);
    let op = Op::Patch(Patch {
        layer: s.layers()[0].id,
        frame: s.frames()[0].id,
        bbox: Some(s.bounds()),
        rect: r,
        pixels: cor.to_vec(),
    });
    apply(s, op).expect("pinta");
}

fn cor_em(s: &Sprite, x: i32, y: i32) -> [u8; 4] {
    let mut px = Vec::new();
    compose_rgba(s, s.frames()[0].id, None, &mut px);
    let i = (y as usize * s.width() as usize + x as usize) * 4;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

/// Sprite NÃO quadrado, slice fora do centro, e o píxel certo pintado em cada marca: o canto da caixa
/// (vermelho), o canto do miolo (verde), o pivot (azul). Separa `w` de `w − 1` e largura de altura.
fn marcado() -> (Sprite, SliceId) {
    let mut s = sprite(40, 24, 1);
    let shape = SliceShape {
        bounds: Rect::new(3, 2, 11, 7),
        center: Some(Rect::new(2, 1, 5, 3)),
        pivot: Some((8, 5)),
    };
    let id = com_keys(&mut s, &[(0, shape)]);
    pinta(&mut s, 3, 2, VERMELHO);
    pinta(&mut s, 5, 3, VERDE);
    pinta(&mut s, 11, 7, AZUL);
    (s, id)
}

/// Em qualquer pose, as três marcas continuam em cantos da caixa e do miolo, e o azul no pivot.
fn marcas_batem(s: &Sprite, id: SliceId, canto: (bool, bool)) {
    let k = so_a(s, id).keys[0].shape;
    let b = k.bounds;
    let px = |r: Rect, (dir, baixo): (bool, bool)| {
        (
            if dir { r.x + r.w as i32 - 1 } else { r.x },
            if baixo { r.y + r.h as i32 - 1 } else { r.y },
        )
    };
    let (x, y) = px(b, canto);
    assert_eq!(
        cor_em(s, x, y),
        VERMELHO,
        "canto da caixa em {x},{y}: {k:?}"
    );
    let c = k.center.expect("centro");
    let (x, y) = px(Rect::new(b.x + c.x, b.y + c.y, c.w, c.h), canto);
    assert_eq!(cor_em(s, x, y), VERDE, "canto do miolo em {x},{y}: {k:?}");
    let p = k.pivot.expect("pivot");
    assert_eq!(cor_em(s, b.x + p.0, b.y + p.1), AZUL, "pivot: {k:?}");
}

#[test]
fn flip_and_rotate_keep_box_center_and_pivot_on_the_same_pixels() {
    let (s, id) = marcado();
    marcas_batem(&s, id, (false, false));
    type Caso = (Op, (bool, bool), (u32, u32));
    let casos: [Caso; 5] = [
        (
            canvas::flip_canvas(&s, true).unwrap(),
            (true, false),
            (11, 7),
        ),
        (
            canvas::flip_canvas(&s, false).unwrap(),
            (false, true),
            (11, 7),
        ),
        (
            canvas::rotate_canvas(&s, Rotation::R180).unwrap(),
            (true, true),
            (11, 7),
        ),
        (
            canvas::rotate_canvas(&s, Rotation::Cw90).unwrap(),
            (true, false),
            (7, 11),
        ),
        (
            canvas::rotate_canvas(&s, Rotation::Ccw90).unwrap(),
            (false, true),
            (7, 11),
        ),
    ];
    for (op, canto, (w, h)) in casos {
        let mut d = s.clone();
        let inv = apply(&mut d, op).unwrap();
        let b = so_a(&d, id).keys[0].shape.bounds;
        assert_eq!((b.w, b.h), (w, h));
        marcas_batem(&d, id, canto);
        apply(&mut d, inv).unwrap();
        assert_eq!(d, s, "1 desfazer volta tudo");
    }
}

/// Uma key escondida fica escondida: a caixa vazia não vira coordenada.
#[test]
fn a_hidden_key_stays_as_it_is_under_geometry() {
    let mut s = sprite(40, 24, 3);
    let id = com_keys(&mut s, &[(0, caixa(3, 2, 4, 4)), (1, caixa(0, 0, 0, 0))]);
    let op = canvas::flip_canvas(&s, true).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).keys[1].shape, caixa(0, 0, 0, 0));
}

/// Tudo isto muda só as slices se a arte é vazia: mesmo assim é um passo.
#[test]
fn flipping_empty_art_still_moves_the_slices() {
    let mut s = sprite(40, 24, 1);
    let id = com_keys(&mut s, &[(0, caixa(3, 2, 4, 4))]);
    let op = canvas::flip_canvas(&s, true).expect("há slice a mexer");
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).keys[0].shape.bounds.x, 33);
}

/// T7, `full.ase`, Canvas Left+Top ±10: a caixa anda, centro e pivot ficam, e negativo sobrevive.
#[test]
fn canvas_size_crop_and_trim_translate_the_box_only() {
    let mut s = sprite(64, 64, 1);
    let shape = SliceShape {
        bounds: Rect::new(4, 4, 24, 24),
        center: Some(Rect::new(8, 8, 8, 8)),
        pivot: Some((12, 24)),
    };
    let id = com_keys(&mut s, &[(0, shape)]);
    let b = |left, top| Borders {
        left,
        top,
        right: 0,
        bottom: 0,
    };
    for (d, x) in [(10, 14), (-10, -6)] {
        let mut c = s.clone();
        let op = canvas::canvas_size(&c, b(d, d), 0).unwrap();
        apply(&mut c, op).unwrap();
        let k = so_a(&c, id).keys[0].shape;
        assert_eq!(k.bounds, Rect::new(x, x, 24, 24));
        assert_eq!((k.center, k.pivot), (shape.center, shape.pivot));
    }
    let mut c = s.clone();
    let op = canvas::crop(&c, Rect::new(5, 5, 8, 8)).unwrap();
    apply(&mut c, op).unwrap();
    assert_eq!(
        so_a(&c, id).keys[0].shape.bounds,
        Rect::new(-1, -1, 24, 24),
        "Crop não apaga"
    );

    let mut t = s.clone();
    pinta(&mut t, 8, 8, VERMELHO);
    pinta(&mut t, 47, 47, VERMELHO);
    let op = canvas::trim(&t).unwrap();
    apply(&mut t, op).unwrap();
    assert_eq!(
        so_a(&t, id).keys[0].shape.bounds,
        Rect::new(-4, -4, 24, 24),
        "T7 Trim"
    );
}

/// G1: `x` par e `w` ímpar separam «cada borda trunca» (7) de «o tamanho sobe» (8).
#[test]
fn sprite_size_truncates_each_edge_and_the_size_is_the_difference() {
    let mut s = sprite(64, 64, 1);
    let id = com_keys(&mut s, &[(0, caixa(4, 0, 15, 9))]);
    let op = canvas::sprite_size(&s, 32, 32, SizeMethod::Nearest).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).keys[0].shape.bounds, Rect::new(2, 0, 7, 4));
}

/// T7: Sprite Size escala tudo; posição e pivot truncam para zero.
#[test]
fn sprite_size_matches_the_trial_numbers() {
    let k = |b: (i32, i32, u32, u32), c: (i32, i32, u32, u32), p| SliceShape {
        bounds: Rect::new(b.0, b.1, b.2, b.3),
        center: Some(Rect::new(c.0, c.1, c.2, c.3)),
        pivot: Some(p),
    };
    let casos = [
        (
            k((4, 36, 24, 20), (4, 4, 16, 12), (0, 0)),
            128,
            k((8, 72, 48, 40), (8, 8, 32, 24), (0, 0)),
        ),
        (
            k((36, 36, 16, 20), (2, 6, 12, 8), (8, 10)),
            32,
            k((18, 18, 8, 10), (1, 3, 6, 4), (4, 5)),
        ),
        (
            k((5, 7, 15, 9), (3, 1, 9, 7), (7, 3)),
            32,
            k((2, 3, 8, 5), (1, 0, 5, 4), (3, 1)),
        ),
        (
            k((33, 11, 13, 21), (1, 5, 11, 11), (-3, 25)),
            32,
            k((16, 5, 7, 11), (0, 2, 6, 6), (-1, 12)),
        ),
    ];
    for (entrada, lado, saida) in casos {
        let mut s = sprite(64, 64, 1);
        let id = com_keys(&mut s, &[(0, entrada)]);
        let op = canvas::sprite_size(&s, lado, lado, SizeMethod::Nearest).unwrap();
        apply(&mut s, op).unwrap();
        assert_eq!(so_a(&s, id).keys[0].shape, saida, "de {entrada:?}");
    }
}

// --- O formato ---

#[test]
fn the_saved_file_keeps_every_slice_exactly() {
    let mut s = sprite(64, 64, 6);
    let id = com_keys(
        &mut s,
        &[
            (
                0,
                SliceShape {
                    bounds: Rect::new(-8, -8, 24, 24),
                    center: Some(Rect::new(12, 12, 20, 20)),
                    pivot: Some((-5, 99)),
                },
            ),
            (3, caixa(0, 0, 0, 0)),
        ],
    );
    let op = slice::set_info(&s, id, "açã ο/x", [1, 2, 3, 128], "hello").unwrap();
    apply(&mut s, op).unwrap();
    let mut lib = BTreeMap::new();
    lib.insert(5, s.clone());
    let bytes = write_library(&lib);
    let lida = read_library(&bytes).expect("lê");
    assert_eq!(lida[&5].slices(), s.slices());
    let mut outra = lida[&5].clone();
    let (op, nova) = slice::create(&mut outra, 0, Rect::new(0, 0, 2, 2)).expect("cria");
    apply(&mut outra, op).unwrap();
    assert!(nova.0 > id.0, "o próximo id não colide depois de ler");
}

#[test]
fn a_bad_list_is_refused_and_nothing_changes() {
    let (mut s, id) = moves();
    let antes = s.clone();
    let mut fora_de_ordem = s.slices().to_vec();
    fora_de_ordem[0].keys.swap(0, 1);
    assert!(apply(&mut s, Op::Slices(fora_de_ordem)).is_err());
    let mut gemeas = s.slices().to_vec();
    gemeas.push(so_a(&s, id).clone());
    assert!(apply(&mut s, Op::Slices(gemeas)).is_err());
    assert_eq!(s, antes);
}

// --- Fixtures que separam hipóteses (prova de mutação P11) ---

/// O quadro novo herda o ANTERIOR (não a primeira key), e a primeira key fica no quadro 0.
#[test]
fn a_new_frame_after_a_later_key_inherits_that_key_and_the_first_key_stays_at_zero() {
    let (s, id) = moves();
    let f2 = s.frames()[2].id;
    let mut c = s.clone();
    let (op, novo) = frames::new_frame(&mut c, f2, NewFrame::Empty).unwrap();
    let d = segue_a_arte(&c, id, op);
    let at = d.frame_index(novo).unwrap() as u32;
    assert_eq!(so_a(&d, id).shape_at(at).unwrap().bounds.x, 24);

    let mut z = s.clone();
    let f0 = z.frames()[0].id;
    let (op, _) = frames::insert(&mut z, 0, f0, NewFrame::Empty, 100).unwrap();
    let d = segue_a_arte(&z, id, op);
    assert_eq!(so_a(&d, id).keys[0].frame, 0, "a key do quadro novo existe");
}

/// Canvas Size com `dx != dy`: cada eixo anda pelo seu.
#[test]
fn canvas_size_translates_each_axis_by_its_own_border() {
    let mut s = sprite(64, 64, 1);
    let id = com_keys(&mut s, &[(0, caixa(4, 4, 8, 8))]);
    let b = Borders {
        left: 3,
        top: 7,
        right: 0,
        bottom: 0,
    };
    let op = canvas::canvas_size(&s, b, 0).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, id).keys[0].shape.bounds, Rect::new(7, 11, 8, 8));
}

/// Uma caixa de 1 px que encolhe não some: mínimo 1.
#[test]
fn sprite_size_never_shrinks_a_box_to_nothing() {
    let mut s = sprite(64, 64, 1);
    let id = com_keys(&mut s, &[(0, caixa(10, 10, 1, 1))]);
    let op = canvas::sprite_size(&s, 32, 32, SizeMethod::Nearest).unwrap();
    apply(&mut s, op).unwrap();
    let b = so_a(&s, id).keys[0].shape.bounds;
    assert_eq!((b.w, b.h), (1, 1));
}

/// `Slice N` é o maior N em uso mais um, não o da última criada.
#[test]
fn the_next_name_follows_the_largest_number_not_the_last_created() {
    let mut s = sprite(32, 32, 1);
    let (op, a) = slice::create(&mut s, 0, Rect::new(0, 0, 4, 4)).unwrap();
    apply(&mut s, op).unwrap();
    let (op, _) = slice::create(&mut s, 0, Rect::new(0, 0, 4, 4)).unwrap();
    apply(&mut s, op).unwrap();
    let op = slice::set_info(&s, a, "Slice 9", [0, 0, 255, 255], "").unwrap();
    apply(&mut s, op).unwrap();
    let (op, n) = slice::create(&mut s, 0, Rect::new(0, 0, 4, 4)).unwrap();
    apply(&mut s, op).unwrap();
    assert_eq!(so_a(&s, n).name, "Slice 10");
}
