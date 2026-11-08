//! P12 S: a revisão do `History` e a entrada guardada do codec (`EntryCache`) — a cópia de
//! recuperação reaproveita os documentos que não mudaram e dá os MESMOS bytes do Save.

use std::collections::BTreeMap;

use ph2d_pixel_core::codec::{
    Entries, EntryCache, write_entry, write_library, write_library_from_entries,
};
use ph2d_pixel_core::frames::{self, NewFrame};
use ph2d_pixel_core::{
    Background, ColorMode, ExportPreset, History, NewSprite, Op, Owner, Paint, PixelPrefs, Sprite,
    Stroke, UserData, layers,
};

fn sprite(w: u32, h: u32) -> Sprite {
    Sprite::new(NewSprite {
        width: w,
        height: h,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .expect("sprite")
}

fn traco(s: &Sprite, y: i32, cor: u32) -> Op {
    let (l, f) = (s.layers()[0].id, s.frames()[0].id);
    let mut st = Stroke::begin(s, l, f, cor, Paint::default(), (0, y));
    st.to(s, i32::try_from(s.width()).expect("w") - 1, y);
    st.op(s).expect("op")
}

#[test]
fn a_revisao_muda_em_todo_passo_e_nao_num_passo_vazio() {
    let mut s = sprite(16, 16);
    let mut h = History::default();
    let mut rev = h.revision();
    let mut muda = |h: &History, o_que: &str| {
        assert_ne!(h.revision(), rev, "{o_que} não mudou a revisão");
        rev = h.revision();
    };
    let ops: Vec<(&str, fn(&mut Sprite) -> Op)> = vec![
        ("Patch", |s| traco(s, 3, 0xff00_00ff)),
        ("Palette", |s| Op::Palette {
            colors: vec![[0, 0, 0, 0], [9, 9, 9, 255]],
            transparent: 0,
        }),
        ("Remap", |s| Op::Remap {
            perm: (0..s.palette().len()).rev().collect(),
        }),
        ("Layers", |s| {
            let top = s.layers()[0].id;
            layers::new_layer(s, top).expect("camada").0
        }),
        ("Frames", |s| {
            let f = s.frames()[0].id;
            frames::new_frame(s, f, NewFrame::Copy).expect("frame").0
        }),
        ("UserData", |_| Op::UserData {
            owner: Owner::Sprite,
            data: Box::new(UserData {
                text: "hp".into(),
                ..UserData::default()
            }),
        }),
        ("Batch", |s| Op::Batch(vec![traco(s, 5, 0xff00_ff00)])),
    ];
    for (nome, faz) in ops {
        let op = faz(&mut s);
        h.commit(&mut s, op).expect(nome);
        muda(&h, nome);
    }
    assert!(h.undo(&mut s));
    muda(&h, "undo");
    assert!(h.redo(&mut s));
    muda(&h, "redo");
    assert!(h.go_to(&mut s, 2));
    muda(&h, "go_to");
    h.touch();
    muda(&h, "touch");

    let recusado = Op::Palette {
        colors: Vec::new(),
        transparent: 0,
    };
    assert!(h.commit(&mut s, recusado).is_err());
    assert_eq!(h.revision(), rev, "um passo recusado não muda nada");
    while h.redo(&mut s) {}
    let r = h.revision();
    assert!(!h.redo(&mut s));
    assert_eq!(h.revision(), r, "nada a refazer não muda a revisão");
    assert!(!h.go_to(&mut s, 99));
    assert_eq!(h.revision(), r, "go_to fora da lista não muda a revisão");
    let mut vazio = History::default();
    let r0 = vazio.revision();
    assert!(!vazio.undo(&mut s));
    assert_eq!(vazio.revision(), r0, "nada a desfazer não muda a revisão");
    assert_ne!(
        History::default().revision(),
        History::default().revision(),
        "cada histórico nasce com a sua: uma chave nunca serve a outro documento"
    );
}

/// O que muda o gravado SEM passo do desfazer (prefs, próximo id, último export) também troca a
/// entrada: a cache compara o registo sem células.
#[test]
fn o_que_muda_sem_passo_tambem_reescreve_a_entrada() {
    let mut s = sprite(16, 16);
    let h = History::default();
    let mut c = EntryCache::default();
    let antes = c.entry(7, h.revision(), &s).to_vec();
    assert_eq!(antes, write_entry(7, &s));
    let mut p = PixelPrefs::default();
    p.grid.w = 5;
    s.set_prefs(p);
    assert_eq!(
        c.entry(7, h.revision(), &s),
        write_entry(7, &s),
        "set_prefs"
    );
    assert_ne!(c.bytes(), antes.as_slice());
    let _ = s.reserve_id();
    assert_eq!(
        c.entry(7, h.revision(), &s),
        write_entry(7, &s),
        "reserve_id"
    );
    s.set_export_preset(Some(ExportPreset {
        path: "/x.png".into(),
        options: "scale=2".into(),
    }));
    assert_eq!(c.entry(7, h.revision(), &s), write_entry(7, &s), "export");
    assert_eq!(
        c.entry(8, h.revision(), &s),
        write_entry(8, &s),
        "outra chave"
    );
}

/// Propriedade: sequências pseudo-aleatórias de edições sobre três documentos; depois de cada uma,
/// a biblioteca pelas entradas guardadas = a escrita inteira, byte a byte.
#[test]
fn a_cache_da_os_mesmos_bytes_que_a_escrita_inteira() {
    let mut seed: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut rnd = |n: u64| {
        seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (seed >> 33) % n
    };
    for corrida in 0..6 {
        let mut docs: Vec<(Sprite, History, EntryCache)> = (0..3)
            .map(|_| (sprite(12, 12), History::default(), EntryCache::default()))
            .collect();
        for passo in 0..60 {
            let i = usize::try_from(rnd(3)).expect("i");
            let (s, h, _) = &mut docs[i];
            match rnd(8) {
                0 | 1 => {
                    let y = i32::try_from(rnd(12)).expect("y");
                    let op = traco(
                        s,
                        y,
                        0xff00_0000 | u32::try_from(rnd(0xff_ffff)).expect("c"),
                    );
                    h.commit(s, op).expect("traço");
                }
                2 => {
                    h.undo(s);
                }
                3 => {
                    h.redo(s);
                }
                4 => {
                    let mut p = s.prefs().clone();
                    p.grid.w = 1 + u32::try_from(rnd(30)).expect("w");
                    s.set_prefs(p);
                }
                5 => {
                    let top = s.layers()[0].id;
                    if let Some((op, _)) = layers::new_layer(s, top) {
                        h.commit(s, op).expect("camada");
                    }
                }
                6 => {
                    let op = Op::UserData {
                        owner: Owner::Sprite,
                        data: Box::new(UserData {
                            text: format!("t{}", rnd(5)),
                            ..UserData::default()
                        }),
                    };
                    h.commit(s, op).expect("user data");
                }
                _ => {
                    let _ = s.reserve_id();
                }
            }
            let lib: BTreeMap<u64, Sprite> = docs
                .iter()
                .enumerate()
                .map(|(k, d)| (k as u64, d.0.clone()))
                .collect();
            for (k, (s, h, c)) in docs.iter_mut().enumerate() {
                c.entry(k as u64, h.revision(), s);
            }
            let cached = write_library_from_entries(docs.iter().map(|d| d.2.bytes()));
            assert!(
                cached == write_library(&lib),
                "corrida {corrida}, passo {passo}: a cache divergiu da escrita inteira"
            );
        }
    }
}

/// **A medição do P12 S** (`--profile smoke`, MESMO processo, intercalado, mínimo de N): 16
/// sprites 512² × 8 quadros = 128 MiB, um editado entre cópias. «antes» = o `persist::to_bytes`
/// de antes (clona cada sprite + `write_library`); «depois» = `EntryCache` + entradas.
#[test]
fn o_codec_reaproveita_os_documentos_que_nao_mudaram() {
    const N: usize = 5;
    let mut docs: Vec<(Sprite, History, EntryCache)> = (0..16)
        .map(|k| {
            let mut s = sprite(512, 512);
            let mut h = History::with_cap(usize::MAX);
            for _ in 0..7 {
                let f = *s.frames().last().map(|f| &f.id).expect("f");
                let (op, _) = frames::new_frame(&mut s, f, NewFrame::Empty).expect("frame");
                h.commit(&mut s, op).expect("frame");
            }
            let l = s.layers()[0].id;
            // Cada cel cheio: os 128 MiB que o artista teria.
            for f in s.frames().iter().map(|f| f.id).collect::<Vec<_>>() {
                let op = Op::Patch(ph2d_pixel_core::Patch {
                    layer: l,
                    frame: f,
                    bbox: Some(ph2d_pixel_core::Rect::new(0, 0, 512, 512)),
                    rect: ph2d_pixel_core::Rect::new(0, 0, 512, 512),
                    pixels: vec![u8::try_from(k).expect("k"); 512 * 512 * 4],
                });
                h.commit(&mut s, op).expect("cheio");
            }
            (s, h, EntryCache::default())
        })
        .collect();
    let antes = |docs: &[(Sprite, History, EntryCache)]| {
        let lib: BTreeMap<u64, Sprite> = docs
            .iter()
            .enumerate()
            .map(|(k, d)| (k as u64, d.0.clone()))
            .collect();
        write_library(&lib)
    };
    // O fio da janela da cópia: só as entradas (a junção corre no fio da escrita, `Entries`).
    let entradas = |docs: &mut [(Sprite, History, EntryCache)]| {
        for (k, (s, h, c)) in docs.iter_mut().enumerate() {
            c.entry(k as u64, h.revision(), s);
        }
        Entries(docs.iter().filter_map(|d| d.2.shared()).collect())
    };
    let total = antes(&docs).len();
    entradas(&mut docs);
    let mut t = [Vec::new(), Vec::new(), Vec::new()];
    for i in 0..3 * N {
        let (s, h, _) = &mut docs[i % 16];
        let op = traco(s, i32::try_from(i % 512).expect("y"), 0xff12_3456);
        h.commit(s, op).expect("edita");
        let (v, t0) = ((i + i / 3) % 3, std::time::Instant::now());
        let n = match v {
            0 => antes(&docs).len(),
            1 => entradas(&mut docs).library().len(),
            _ => entradas(&mut docs).0.iter().map(|e| e.len()).sum::<usize>() + 14,
        };
        t[v].push(t0.elapsed());
        assert_eq!(n, total, "variante {v}");
    }
    assert_eq!(
        entradas(&mut docs).library(),
        antes(&docs),
        "a cache dá os bytes do Save"
    );
    let m = t.map(|v| *v.iter().min().expect("min"));
    eprintln!(
        "P12 S codec, {} MiB, um editado, mínimo de {N}: antes {:?} · cache + junção {:?} · só entradas {:?}",
        total >> 20,
        m[0],
        m[1],
        m[2]
    );
}
