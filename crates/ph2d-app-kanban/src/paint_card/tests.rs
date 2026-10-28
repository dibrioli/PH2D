//! A capa de IMAGEM pela costura frente → pintor: os bytes do conjunto descodificam-se fora do fio,
//! a imagem cai recortada na capa (metade ou inteira), o véu do título lê-se a 4,5:1 sobre a imagem
//! REAL, e a altura que a pintura percorre é a medida.

use super::*;
use crate::front::{Counts, cover, measure};
use ph2d_kanban_model::{
    Card, CardId, Cover, CoverFill, FracKey, KanbanId, KanbanOp, KanbanSet, List, ListId, Meta,
    Moment, Record,
};
use std::collections::BTreeMap;

fn crc32(bytes: &[u8]) -> u32 {
    let mut c = 0xFFFF_FFFF_u32;
    for &b in bytes {
        c ^= u32::from(b);
        for _ in 0..8 {
            c = if c & 1 == 1 {
                (c >> 1) ^ 0xEDB8_8320
            } else {
                c >> 1
            };
        }
    }
    !c
}

/// Um PNG RGBA mínimo (deflate «stored»), com a cor de cada píxel dada por `px(x, y)`.
pub(crate) fn png(w: u32, h: u32, px: impl Fn(u32, u32) -> [u8; 4]) -> Vec<u8> {
    let mut raw = Vec::new();
    for y in 0..h {
        raw.push(0);
        for x in 0..w {
            raw.extend_from_slice(&px(x, y));
        }
    }
    let mut z = vec![0x78, 0x01];
    let chunks: Vec<&[u8]> = raw.chunks(65_535).collect();
    for (i, ch) in chunks.iter().enumerate() {
        z.push(u8::from(i + 1 == chunks.len()));
        let n = u16::try_from(ch.len()).expect("bloco");
        z.extend_from_slice(&n.to_le_bytes());
        z.extend_from_slice(&(!n).to_le_bytes());
        z.extend_from_slice(ch);
    }
    let (mut a, mut b) = (1_u32, 0_u32);
    for &x in &raw {
        a = (a + u32::from(x)) % 65_521;
        b = (b + a) % 65_521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut chunk = |kind: &[u8; 4], data: &[u8]| {
        out.extend_from_slice(&u32::try_from(data.len()).expect("len").to_be_bytes());
        let mut body = kind.to_vec();
        body.extend_from_slice(data);
        out.extend_from_slice(&body);
        out.extend_from_slice(&crc32(&body).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(b"IHDR", &ihdr);
    chunk(b"IDAT", &z);
    chunk(b"IEND", &[]);
    out
}

/// Um Kanban com um cartão de capa de imagem `bytes`.
fn with_image(bytes: Vec<u8>, size: [u32; 2], full: bool) -> (KanbanSet, KanbanId, CardId) {
    let mut set = KanbanSet::default();
    let id = set.create("K");
    let file = set.put_file(bytes, Some(size));
    let (list, card) = (ListId(10), CardId(11));
    let pos = FracKey::between(None, None);
    let ops = vec![
        KanbanOp::put(Record::List(List {
            id: list,
            title: "L".into(),
            pos: pos.clone(),
            color: None,
            card_limit: None,
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
        })),
        KanbanOp::put(Record::card(Card {
            id: card,
            list,
            pos,
            title: "Forest background".into(),
            description: String::new(),
            labels: Vec::new(),
            members: Vec::new(),
            start: None,
            due: None,
            cover: Some(Cover {
                fill: CoverFill::Image(file),
                full,
            }),
            fields: BTreeMap::new(),
            location: String::new(),
            votes: Vec::new(),
            created: Moment(0),
            archived: false,
            meta: Meta::default(),
            watchers: Vec::new(),
            origin: None,
            template: false,
        }.into())),
    ];
    set.apply(id, ops);
    (set, id, card)
}

/// Pinta o cartão quando a capa chegou; devolve o que a pintura percorreu e a altura medida.
fn paint(set: &KanbanSet, id: KanbanId, card: CardId) -> (probe::Painted, f32) {
    let k = set.get(id).expect("kanban");
    let counts = Counts::of(k).with_files(set, k);
    let mut f = front::front(k, &counts, card).expect("frente");
    for _ in 0..400 {
        if f.cover_px.is_some() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
        f = front::front(k, &counts, card).expect("frente");
    }
    assert!(f.cover_px.is_some(), "a capa descodificou em 2 s");
    assert!(!cover::pending());
    let mut ts = TextSystem::without_system_fonts();
    let h = measure(&mut ts, &f, true);
    crate::docs::KanbanDocs::default().probe_arm_fronts();
    let theme = Theme::Dark;
    let look = CardLook {
        front: f,
        hover: false,
        focus: false,
        age: 0,
    };
    let r = Rect::new(20.0, 30.0, m::CARD_W, h);
    super::card(
        &mut VectorScene::new(),
        &mut ts,
        theme,
        &Ink::of(theme),
        &look,
        r,
        true,
    );
    let p = probe::last("Forest background");
    (p.expect("pintado"), h)
}

#[test]
fn a_half_image_cover_fills_the_band_cropped_and_centred() {
    // Larga: o recorte corta dos lados, a altura enche.
    let (set, id, card) = with_image(
        png(400, 100, |x, _| {
            [u8::try_from(x % 256).unwrap_or(0), 90, 40, 255]
        }),
        [400, 100],
        false,
    );
    let (p, h) = paint(&set, id, card);
    let (dest, band) = p.image.expect("a imagem foi pintada");
    assert_eq!(
        (band.y, band.w, band.h),
        (30.0, m::CARD_W, front::COVER_IMAGE_H)
    );
    assert!(
        (dest.h - band.h).abs() < 1e-3 && dest.w >= band.w,
        "{dest:?}"
    );
    let centre = |r: Rect| r.x + r.w / 2.0;
    assert!((centre(dest) - centre(band)).abs() < 1e-3);
    assert!((p.h - h).abs() < 1e-3, "pintada {} == medida {h}", p.h);
}

#[test]
fn a_full_image_cover_keeps_the_title_legible_on_the_real_image() {
    // Branca: o pior fundo — o véu sobe até o título se ler a 4,5:1.
    let (set, id, card) = with_image(png(64, 64, |_, _| [255, 255, 255, 255]), [64, 64], true);
    let (p, h) = paint(&set, id, card);
    let (dest, area) = p.image.expect("a imagem foi pintada");
    assert!((area.h - front::FULL_MIN_H).abs() < 1e-3 && (area.h - h).abs() < 1e-3);
    assert!(dest.w >= area.w && dest.h >= area.h);
    let (alpha, ratio) = p.veil.expect("o véu");
    assert!(
        ratio >= cover::MIN_CONTRAST - 1e-3,
        "contraste {ratio} com véu {alpha}"
    );
    // Escura: o véu fica no da maquete.
    let (set, id, card) = with_image(png(64, 64, |_, _| [10, 12, 20, 255]), [64, 64], true);
    let (p, _) = paint(&set, id, card);
    let (alpha_dark, _) = p.veil.expect("o véu");
    assert!(alpha_dark < alpha, "{alpha_dark} < {alpha}");
    assert!((p.h - h).abs() < 1e-3);
}

#[test]
fn a_big_photo_is_halved_down_to_the_card_texels() {
    let t = cover::decode(&png(2100, 300, |_, _| [200, 100, 50, 255])).expect("png");
    assert!(
        t.size[0] < 2 * cover::TEXELS_W && t.size[0] >= cover::TEXELS_W,
        "{:?}",
        t.size
    );
    assert_eq!(t.row_max.len(), t.size[1] as usize);
}

/// ⚠️ RÉGUA (ignorada; `--profile smoke -- --ignored medir_capas`): 2 000 cartões (o tamanho da
/// cena 2) com capa de imagem de 20 ficheiros × sem capa, intercalados, o MÍNIMO de 7 blocos — a
/// frente (cache), a altura e a pintura de um quadro inteiro; e a descodificação de uma foto.
#[test]
#[ignore = "régua de custo: docs/TrelloClone/loop/MEDICOES.md §W3 D2"]
fn medir_capas() {
    use std::time::Instant;
    let mut set = KanbanSet::default();
    let id = set.create("K");
    let files: Vec<_> = (0..20_u8)
        .map(|i| {
            set.put_file(
                png(1600, 900, move |x, y| {
                    [i, (x % 256) as u8, (y % 256) as u8, 255]
                }),
                Some([1600, 900]),
            )
        })
        .collect();
    let mut ops = Vec::new();
    let list = ListId(1);
    let mut pos = FracKey::between(None, None);
    ops.push(KanbanOp::put(Record::List(List {
        id: list,
        title: "L".into(),
        pos: pos.clone(),
        color: None,
        card_limit: None,
        archived: false,
        meta: Meta::default(),
        watchers: Vec::new(),
    })));
    for n in 0..2000_u64 {
        pos = FracKey::between(Some(&pos), None);
        let mut c = with_image(Vec::new(), [1, 1], false)
            .0
            .get(KanbanId(1))
            .and_then(|k| k.cards.values().next().cloned())
            .expect("molde");
        c.id = CardId(100 + n);
        c.list = list;
        c.pos = pos.clone();
        c.title = format!("Shot {n:04} — layout, anim, light");
        c.cover = Some(Cover {
            fill: CoverFill::Image(files[(n % 20) as usize]),
            full: n % 4 == 0,
        });
        ops.push(KanbanOp::put(Record::Card(c.into())));
    }
    set.apply(id, ops);
    let k = set.get(id).expect("k").clone();
    let mut bare = k.clone();
    for c in bare.cards.values_mut() {
        c.cover = None;
    }
    let t0 = Instant::now();
    let one = cover::decode(&set.file(files[0]).expect("f").bytes.0).expect("png");
    let decode_ms = t0.elapsed().as_secs_f64() * 1e3;
    let counts = Counts::of(&k).with_files(&set, &k);
    let ids: Vec<CardId> = k.cards.keys().copied().collect();
    for _ in 0..800 {
        let ready = ids
            .iter()
            .all(|c| front::front(&k, &counts, *c).is_some_and(|f| f.cover_px.is_some()));
        if ready {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    let mut ts = TextSystem::without_system_fonts();
    let theme = Theme::Dark;
    let ink = Ink::of(theme);
    let mut frame = |kk: &ph2d_kanban_model::Kanban, counts: &Counts| {
        let mut scene = VectorScene::new();
        let t = Instant::now();
        let fronts: Vec<Front> = ids
            .iter()
            .filter_map(|c| front::front(kk, counts, *c))
            .collect();
        let t_front = t.elapsed();
        let hs: Vec<f32> = fronts.iter().map(|f| measure(&mut ts, f, true)).collect();
        let t_measure = t.elapsed() - t_front;
        for (f, h) in fronts.into_iter().zip(hs) {
            let look = CardLook {
                front: f,
                hover: false,
                focus: false,
                age: 0,
            };
            super::card(
                &mut scene,
                &mut ts,
                theme,
                &ink,
                &look,
                Rect::new(0.0, 0.0, m::CARD_W, h),
                true,
            );
        }
        let all = t.elapsed();
        [t_front, t_measure, all - t_front - t_measure].map(|d| d.as_secs_f64() * 1e3)
    };
    let bare_counts = Counts::of(&bare);
    let (mut a, mut b) = ([f64::MAX; 3], [f64::MAX; 3]);
    for _ in 0..7 {
        let x = frame(&k, &counts);
        let y = frame(&bare, &bare_counts);
        for i in 0..3 {
            a[i] = a[i].min(x[i]);
            b[i] = b[i].min(y[i]);
        }
    }
    println!("decode 1600x900 PNG -> {:?}: {decode_ms:.1} ms", one.size);
    println!("com capas  (front, altura, pintura) ms: {a:.2?}");
    println!("sem capas  (front, altura, pintura) ms: {b:.2?}");
}
