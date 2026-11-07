//! **As slices no `.ase` e no JSON** (P11): o Trial lê os nossos ficheiros e escreve `meta.slices`, e o
//! nosso escritor tem de dar os mesmos bytes (`docs/PixelArt/P11_oraculos_arquivo.md` F3, F5).

use ph2d_pixel_core::{Slice, Sprite};
use ph2d_pixel_io::ase::{Aviso, escrever, ler};
use ph2d_pixel_io::sheet::{OpcoesFolha, OpcoesJson, json_da_folha, montar};
use serde_json::Value;

const ORACULO: &str = include_str!("fixtures/oracle_slices_meta.json");

fn ase(nome: &str) -> Vec<u8> {
    let p = format!(
        "{}/tests/fixtures/slices/{nome}.ase",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(&p).expect("a fixtura")
}

fn meta_slices(s: &Sprite) -> String {
    let f = montar(s, &OpcoesFolha::default(), "x", "ase");
    let o = OpcoesJson {
        slices: true,
        ..OpcoesJson::default()
    };
    let t = json_da_folha(s, &f, "s.png", "RGBA8888", o);
    t[t.find("\"slices\"").expect("a chave")..].to_owned()
}

fn casos() -> Vec<(String, String)> {
    let v: Value = serde_json::from_str(ORACULO).unwrap();
    let c = v["casos"].as_object().unwrap();
    assert_eq!(c.len(), 11, "os casos da fixtura");
    c.iter()
        .map(|(k, t)| (k.clone(), t.as_str().unwrap().to_owned()))
        .collect()
}

/// ⭐ O mesmo `.ase`, o Trial e nós: os mesmos bytes em `meta.slices`, com keys, escondidas, fora do
/// canvas, gémeas, sem cor, com texto e UTF-8. *Mutação: tirar o `.rev()`, ou trocar a ordem de `data`.*
#[test]
fn our_meta_slices_are_the_bytes_the_trial_writes_for_the_same_file() {
    for (nome, dele) in casos() {
        let (s, _) = ler(&ase(&nome)).unwrap_or_else(|e| panic!("{nome}: {e:?}"));
        assert_eq!(meta_slices(&s), dele, "{nome}");
    }
}

fn sem_ids(v: &[Slice]) -> Vec<Slice> {
    let mut v = v.to_vec();
    for c in &mut v {
        c.id = ph2d_pixel_core::SliceId(0);
        // A cor em falta grava-se: sem ela o Trial não desenha a slice (F3).
        c.color.get_or_insert(ph2d_pixel_core::DEFAULT_SLICE_COLOR);
    }
    v
}

/// Gravar e reabrir não perde nada, salvo o que o formato não diz: pivot por key numa slice com flag
/// geral, que se escreve `0,0` (limitação declarada no plano).
#[test]
fn saving_and_reopening_keeps_every_slice() {
    for (nome, _) in casos() {
        let (s, _) = ler(&ase(&nome)).unwrap();
        let (bytes, _) = escrever(&s).unwrap();
        let (volta, _) = ler(&bytes).unwrap();
        assert_eq!(sem_ids(volta.slices()), sem_ids(s.slices()), "{nome}");
        let (de_novo, _) = escrever(&volta).unwrap();
        assert_eq!(de_novo, bytes, "{nome}: gravar é estável");
    }
}

/// F3: a key fora de ordem ordena-se, e o chunk conta onde quer que esteja.
#[test]
fn unsorted_keys_sort_and_a_chunk_in_a_later_frame_is_read() {
    let (s, _) = ler(&ase("weird")).unwrap();
    let c = s.slices().iter().find(|c| c.name == "unsorted").unwrap();
    assert_eq!(c.keys.iter().map(|k| k.frame).collect::<Vec<_>>(), [1, 3]);
    assert_eq!(
        s.slices().iter().filter(|c| c.name == "twin").count(),
        2,
        "gémeas ficam"
    );
    let far = s.slices().iter().find(|c| c.name == "farframe").unwrap();
    assert_eq!(far.keys[0].frame, 99, "a key além do fim fica");

    let (t, _) = ler(&ase("tarde")).unwrap();
    assert_eq!(t.slices()[0].keys.len(), 2);
}

/// O user data pertence só à slice que o precede: o de outra coisa não a pinta.
#[test]
fn user_data_belongs_only_to_the_chunk_right_before_it() {
    let (s, avisos) = ler(&ase("ud")).unwrap();
    assert!(!s.slices().is_empty());
    assert!(avisos.is_empty(), "{avisos:?}");
    let (r, _) = ler(&ase("realorder")).unwrap();
    assert!(
        r.slices().iter().all(|c| c.color.is_some()),
        "camadas → slice+user → cels"
    );
}

/// Propriedades tipadas ficam para a P12: a slice fica, e o artista é avisado.
#[test]
fn typed_properties_warn_and_the_slice_stays() {
    let (limpo, _) = ler(&ase("full")).unwrap();
    let (bytes, _) = escrever(&limpo).unwrap();
    assert_eq!(ler(&bytes).unwrap().1, Vec::new());

    // O user data de `full` traz a cor (bit 2): acende-se o bit das propriedades (4).
    let mut marcado = bytes.clone();
    let cabecalho = [14, 0, 0, 0, 0x20, 0x20, 2, 0, 0, 0];
    let i = marcado.windows(10).position(|w| w == cabecalho).unwrap();
    marcado[i + 6..i + 10].copy_from_slice(&6_u32.to_le_bytes());
    let (s, avisos) = ler(&marcado).unwrap();
    assert_eq!(avisos, vec![Aviso::PropriedadesDeSlices(1)]);
    assert_eq!(s.slices().len(), 3);
}

// --- O corte por slice: lei nossa, sem paridade (o Trial não grava imagem, `P11_plano.md` §2) ---

/// `keys.ase` com a arte em quadrados de cor diferente em cada caixa, para o píxel dizer de onde veio.
fn sprite_com_keys() -> (Sprite, ph2d_pixel_core::SliceId) {
    use ph2d_pixel_core::frames::{NewFrame, new_frame};
    use ph2d_pixel_core::{NewSprite, Op, Patch, Rect, SliceShape, apply, slice};
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 8,
        ..NewSprite::default()
    })
    .unwrap();
    for _ in 0..3 {
        let f = s.frames().last().unwrap().id;
        let (op, _) = new_frame(&mut s, f, NewFrame::Empty).unwrap();
        apply(&mut s, op).unwrap();
    }
    // Cada quadro tem a linha x=y pintada com o número do quadro no vermelho.
    for i in 0..4 {
        let r = Rect::new(0, 0, 16, 8);
        let mut px = vec![0_u8; 16 * 8 * 4];
        for y in 0..8 {
            for x in 0..16 {
                let p = (y * 16 + x) * 4;
                px[p..p + 4].copy_from_slice(&[10 * (i + 1) as u8, x as u8, y as u8, 255]);
            }
        }
        let op = Op::Patch(Patch {
            layer: s.layers()[0].id,
            frame: s.frames()[i].id,
            bbox: Some(r),
            rect: r,
            pixels: px,
        });
        apply(&mut s, op).unwrap();
    }
    let (op, id) = slice::create(&mut s, 0, Rect::new(1, 2, 3, 2)).unwrap();
    apply(&mut s, op).unwrap();
    let caixa = |x, y, w, h| SliceShape {
        bounds: Rect::new(x, y, w, h),
        ..SliceShape::default()
    };
    // Quadro 1: anda e passa do canvas. Quadro 2: escondida. Quadro 3: volta.
    for (f, k) in [
        (1, caixa(14, 6, 4, 4)),
        (2, caixa(0, 0, 0, 0)),
        (3, caixa(5, 5, 2, 1)),
    ] {
        let op = slice::set_shape(&s, id, f, k).unwrap();
        apply(&mut s, op).unwrap();
    }
    (s, id)
}

fn px(q: &ph2d_pixel_io::render::QuadroSaida, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * q.w + x) * 4) as usize;
    q.rgba[i..i + 4].try_into().unwrap()
}

/// Cada quadro corta na key DESSE quadro, à interseção com o canvas; o escondido não sai. O píxel do
/// canto prova de que quadro e de que lugar veio. *Mutação: cortar com a key do quadro 0.*
#[test]
fn area_slice_cuts_each_frame_at_its_own_key_and_skips_hidden() {
    use ph2d_pixel_io::render::{Area, Pedido, quadros};
    let (s, id) = sprite_com_keys();
    let p = Pedido {
        area: Area::Slice(id),
        ..Pedido::default()
    };
    let q = quadros(&s, &p);
    assert_eq!(q.iter().map(|q| q.indice).collect::<Vec<_>>(), [0, 1, 3]);
    assert_eq!((q[0].w, q[0].h), (3, 2));
    assert_eq!(px(&q[0], 0, 0), [10, 1, 2, 255]);
    assert_eq!((q[1].w, q[1].h), (2, 2), "corta ao canvas");
    assert_eq!(px(&q[1], 1, 1), [20, 15, 7, 255]);
    assert_eq!((q[2].w, q[2].h), (2, 1));
    assert_eq!(px(&q[2], 0, 0), [40, 5, 5, 255]);
}

/// O formato animado tem um só tamanho: a caixa que cobre todas as keys, e o escondido continua fora.
#[test]
fn an_animated_format_uses_one_box_covering_every_key() {
    use ph2d_pixel_io::render::{Area, Pedido};
    use ph2d_pixel_io::saida::{Formato, exportar};
    let (s, id) = sprite_com_keys();
    let p = Pedido {
        area: Area::Slice(id),
        ..Pedido::default()
    };
    let saida = exportar(&s, &p, Formato::Gif, 0).expect("grava");
    assert_eq!(saida.ficheiros.len(), 1);
    // A união de (1,2,3,2), (14,6,2,2) e (5,5,2,1) dentro do canvas é (1,2,15,6).
    let gif = &saida.ficheiros[0].1;
    assert_eq!(
        (
            u16::from_le_bytes([gif[6], gif[7]]),
            u16::from_le_bytes([gif[8], gif[9]])
        ),
        (15, 6)
    );
}

#[test]
fn a_slice_that_never_appears_is_an_error_and_not_an_empty_export() {
    use ph2d_pixel_core::{Op, Rect, SliceShape, apply, slice};
    use ph2d_pixel_io::render::{Area, Pedido};
    use ph2d_pixel_io::saida::{ErroSaida, Formato, exportar};
    let (mut s, id) = sprite_com_keys();
    let mut lista = s.slices().to_vec();
    lista[0].keys.truncate(1);
    lista[0].keys[0].shape = SliceShape {
        bounds: Rect::new(100, 100, 4, 4),
        ..SliceShape::default()
    };
    apply(&mut s, Op::Slices(lista)).unwrap();
    let _ = slice::default_center;
    let p = Pedido {
        area: Area::Slice(id),
        ..Pedido::default()
    };
    assert!(matches!(
        exportar(&s, &p, Formato::Png, 0),
        Err(ErroSaida::NadaNaArea)
    ));
}

/// `Split slices`: uma peça por slice e quadro, com o conteúdo certo e os nomes sem colisão, gémeas
/// incluídas; sem slices nada muda. *Mutação: tirar o número dos gémeos.*
#[test]
fn split_slices_makes_one_piece_per_visible_key_and_twins_do_not_collide() {
    use ph2d_pixel_core::{Rect, apply, slice};
    use ph2d_pixel_io::sheet::{OpcoesFolha, montar};
    let (mut s, _) = sprite_com_keys();
    let o = OpcoesFolha {
        split_slices: true,
        ..OpcoesFolha::default()
    };
    let f = montar(&s, &o, "x", "ase");
    let nomes: Vec<&str> = f.entradas.iter().map(|e| e.nome.as_str()).collect();
    assert_eq!(
        nomes,
        [
            "x [Slice 1] 0.ase",
            "x [Slice 1] 1.ase",
            "x [Slice 1] 2.ase"
        ]
    );
    assert_eq!(
        f.entradas[1].origem,
        [14, 6, 2, 2],
        "onde a peça está no sprite"
    );
    assert_eq!(
        f.entradas[1].tamanho,
        [16, 8],
        "sourceSize é o canvas, como no trim"
    );

    let (op, gemea) = slice::create(&mut s, 0, Rect::new(0, 0, 2, 2)).unwrap();
    apply(&mut s, op).unwrap();
    let c = s.slice(gemea).unwrap();
    let (nome, cor, data) = ("Slice 1".to_owned(), c.color.unwrap(), c.data.clone());
    let op = slice::set_info(&s, gemea, &nome, cor, &data).unwrap();
    apply(&mut s, op).unwrap();
    let f = montar(&s, &o, "x", "ase");
    let mut nomes: Vec<&str> = f.entradas.iter().map(|e| e.nome.as_str()).collect();
    assert_eq!(nomes.len(), 7, "3 de uma, 4 da outra");
    nomes.sort_unstable();
    nomes.dedup();
    assert_eq!(nomes.len(), 7, "nenhum nome repetido");
}

#[test]
fn split_slices_without_slices_is_the_plain_sheet() {
    use ph2d_pixel_core::NewSprite;
    use ph2d_pixel_io::sheet::{OpcoesFolha, montar};
    let s = Sprite::new(NewSprite::default()).unwrap();
    let o = OpcoesFolha {
        split_slices: true,
        ..OpcoesFolha::default()
    };
    assert_eq!(montar(&s, &o, "x", "ase").entradas.len(), 1);
}
