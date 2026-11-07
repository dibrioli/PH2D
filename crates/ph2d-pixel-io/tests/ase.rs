//! O `.ase`: volta igual, e o composto bate com o LibreSprite corrido (`oraculo_export.py`).

use ph2d_pixel_core::{ColorMode, FrameId, Sprite, compose_picked};
use ph2d_pixel_io::ase;
use serde_json::Value;

fn oraculo() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_export.json")).unwrap()
}

fn unhex(h: &str) -> Vec<u8> {
    (0..h.len() / 2)
        .map(|i| u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).unwrap())
        .collect()
}

fn entrada(o: &Value, n: &str) -> Vec<u8> {
    unhex(o["entradas_hex"][n].as_str().unwrap())
}

fn quadro(s: &Sprite, f: FrameId) -> Vec<u8> {
    compose_picked(s, f, &|l| s.is_shown(l.id) && !l.reference)
}

/// O RGB de uma célula de alfa 0 não se vê, e os dois divergem nele sem consequência.
fn sem_cor_no_buraco(mut px: Vec<u8>) -> Vec<u8> {
    for c in px.as_chunks_mut::<4>().0 {
        if c[3] == 0 {
            *c = [0; 4];
        }
    }
    px
}

#[test]
fn o_composto_bate_com_o_libresprite_em_cada_quadro() {
    let o = oraculo();
    for n in o["quadros"].as_object().unwrap().keys() {
        let (s, _) = ase::ler(&entrada(&o, n)).unwrap();
        let q = o["quadros"][n].as_array().unwrap();
        assert_eq!(q.len(), s.frames().len(), "{n}");
        for (i, (f, q)) in s.frames().iter().zip(q).enumerate() {
            let meu = sem_cor_no_buraco(quadro(&s, f.id));
            let dele = sem_cor_no_buraco(unhex(q["rgba_hex"].as_str().unwrap()));
            assert_eq!(
                q["w"].as_u64().unwrap() * q["h"].as_u64().unwrap() * 4,
                dele.len() as u64
            );
            // A folha DELE vem recortada só se o sprite for maior; aqui vem inteira.
            let w = q["w"].as_u64().unwrap() as usize;
            // ⛔ Recusa medida: modo de mistura sobre destino TRANSPARENTE. O LibreSprite dá preto; o
            // Trial (P3, `g*.ase`, 6 144/6 144) dá a fonte, e a lei é a do Trial. Só se ignora ali.
            let fundo = compose_picked(&s, f.id, &|l| l.name == "fundo");
            let misturadas = s.layers().iter().any(|l| l.blend.code() != 0);
            let sobre_vazio: Vec<bool> = fundo
                .as_chunks::<4>()
                .0
                .iter()
                .map(|c| misturadas && c[3] == 0)
                .collect();
            let dif: Vec<_> = meu
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dele.as_chunks::<4>().0)
                .enumerate()
                .filter(|(_, (a, b))| a != b)
                .filter(|(k, _)| !sobre_vazio[*k])
                .map(|(k, (a, b))| (k % w, k / w, *a, *b))
                .collect();
            assert!(
                dif.is_empty(),
                "{n}, quadro {i}: {} células; {:?}",
                dif.len(),
                &dif[..dif.len().min(8)]
            );
        }
    }
}

#[test]
fn o_ase_nosso_volta_igual() {
    let o = oraculo();
    for n in o["entradas_hex"].as_object().unwrap().keys() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (bytes, avisos) = ase::escrever(&a).unwrap();
        assert!(avisos.is_empty(), "{n}: {avisos:?}");
        let (b, _) = ase::ler(&bytes).unwrap();
        assert_eq!(a, b, "{n}");
        let (de_novo, _) = ase::escrever(&b).unwrap();
        assert_eq!(bytes, de_novo, "{n}: não é determinístico");
    }
}

#[test]
fn o_documento_guarda_o_que_o_ficheiro_diz() {
    let o = oraculo();
    let (s, _) = ase::ler(&entrada(&o, "completo")).unwrap();
    let nomes: Vec<_> = s.layers().iter().map(|l| l.name.as_str()).collect();
    assert_eq!(nomes, ["fundo", "grp", "mult", "tela", "oculta"]);
    let (grp, mult) = (&s.layers()[1], &s.layers()[2]);
    assert!(grp.is_group());
    assert_eq!(mult.parent, Some(grp.id));
    assert_eq!(mult.opacity, 128);
    assert_eq!(mult.blend.code(), 1);
    assert!(!s.layers()[4].visible);
    let ms: Vec<_> = s.frames().iter().map(|f| f.duration_ms).collect();
    assert_eq!(ms, [100, 150, 70, 333, 20]);
    let t: Vec<_> = s
        .tags()
        .iter()
        .map(|t| (t.name.as_str(), t.from, t.to, t.repeat))
        .collect();
    assert_eq!(t, [("idle", 0, 1, 0), ("walk", 2, 4, 2), ("back", 1, 3, 3)]);
    // O ligado continua ligado, o dono é o do quadro 1.
    let c = s.entry((s.layers()[0].id, s.frames()[2].id)).unwrap();
    assert_eq!(c.link, Some(s.frames()[1].id));
    let (ix, _) = ase::ler(&entrada(&o, "indexado")).unwrap();
    assert_eq!(ix.mode(), ColorMode::Indexed);
    assert_eq!(ix.palette().len(), 8);
    assert_eq!(ix.palette()[4], [255, 255, 0, 128]);
    let (gr, _) = ase::ler(&entrada(&o, "cinza")).unwrap();
    assert_eq!(gr.mode(), ColorMode::Gray);
}

/// Grava os `.ase` NOSSOS numa pasta, para o `oraculo_export.py --verifica` os dar ao LibreSprite (M2).
#[test]
#[ignore = "ferramenta: PH2D_ASE_SAIDA=<pasta>"]
fn grava_os_nossos_para_o_oraculo() {
    let dir = std::env::var("PH2D_ASE_SAIDA").unwrap();
    let o = oraculo();
    for n in o["entradas_hex"].as_object().unwrap().keys() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (b, _) = ase::escrever(&a).unwrap();
        std::fs::write(format!("{dir}/{n}.ase"), b).unwrap();
    }
}

/// O LibreSprite leu ESTES bytes e deu os mesmos quadros (`oracle_ase_nosso.json`). Mudou o escritor,
/// fica vermelho: corra a verificação de novo, nunca regrave só o hash.
#[test]
fn o_libresprite_le_o_nosso_ase() {
    use sha2::{Digest, Sha256};
    let f: Value = serde_json::from_str(include_str!("fixtures/oracle_ase_nosso.json")).unwrap();
    let o = oraculo();
    for (n, sha) in f["sha256"].as_object().unwrap() {
        let (a, _) = ase::ler(&entrada(&o, n)).unwrap();
        let (b, _) = ase::escrever(&a).unwrap();
        let hex: String = Sha256::digest(&b)
            .iter()
            .map(|x| format!("{x:02x}"))
            .collect();
        assert_eq!(&hex, sha.as_str().unwrap(), "{n}");
    }
}

/// Acrescenta um chunk ao 1.º quadro de um `.ase`, acertando os contadores (spec pública).
fn com_chunk(ase: &[u8], tipo: u16, corpo: &[u8]) -> Vec<u8> {
    const CABECALHO: usize = 128;
    const QUADRO: usize = 16;
    let mut v = ase.to_vec();
    let tam = u32::from_le_bytes(v[CABECALHO..CABECALHO + 4].try_into().unwrap()) as usize;
    let mut c = Vec::new();
    c.extend_from_slice(&(6 + corpo.len() as u32).to_le_bytes());
    c.extend_from_slice(&tipo.to_le_bytes());
    c.extend_from_slice(corpo);
    let fim = CABECALHO + tam;
    v.splice(fim..fim, c.iter().copied());
    let novo = (tam + c.len()) as u32;
    v[CABECALHO..CABECALHO + 4].copy_from_slice(&novo.to_le_bytes());
    let velhos = u16::from_le_bytes([v[CABECALHO + 6], v[CABECALHO + 7]]) + 1;
    v[CABECALHO + 6..CABECALHO + 8].copy_from_slice(&velhos.to_le_bytes());
    let novos = u32::from_le_bytes(v[CABECALHO + 12..CABECALHO + QUADRO].try_into().unwrap()) + 1;
    v[CABECALHO + 12..CABECALHO + QUADRO].copy_from_slice(&novos.to_le_bytes());
    let total = v.len() as u32;
    v[0..4].copy_from_slice(&total.to_le_bytes());
    v
}

/// **Abrir não perde em silêncio:** o que o documento não guarda vira aviso.
#[test]
fn abrir_nao_perde_em_silencio() {
    use ph2d_pixel_core::{ColorMode, NewSprite, Sprite};
    use ph2d_pixel_io::abrir::abrir;
    use ph2d_pixel_io::ase::{Aviso, escrever};
    let s = Sprite::new(NewSprite {
        width: 4,
        height: 4,
        mode: ColorMode::Rgba,
        ..NewSprite::default()
    })
    .unwrap();
    let (limpo, _) = escrever(&s).unwrap();
    assert_eq!(
        abrir(&limpo, "ase").unwrap().1,
        Vec::new(),
        "sem nada a dizer"
    );

    // Chunk de slice: 1 slice, sem flags, reservado, nome «s», uma chave em (0,0) 1×1.
    let mut fatia = Vec::new();
    fatia.extend_from_slice(&1_u32.to_le_bytes());
    fatia.extend_from_slice(&0_u32.to_le_bytes());
    fatia.extend_from_slice(&0_u32.to_le_bytes());
    fatia.extend_from_slice(&1_u16.to_le_bytes());
    fatia.push(b's');
    for campo in [0_u32, 0, 0, 1, 1] {
        fatia.extend_from_slice(&campo.to_le_bytes());
    }
    let com_fatia = com_chunk(&limpo, 0x2022, &fatia);
    let (doc, avisos) = abrir(&com_fatia, "aseprite").unwrap();
    assert_eq!(avisos, Vec::new(), "a slice agora fica no documento");
    assert_eq!(doc.slices().len(), 1);

    // Uma camada de tilemap: flags, tipo 2, filho, largura/altura, modo, opacidade, nome, tileset.
    let mut camada = Vec::new();
    camada.extend_from_slice(&1_u16.to_le_bytes());
    camada.extend_from_slice(&2_u16.to_le_bytes());
    camada.extend_from_slice(&0_u16.to_le_bytes());
    camada.extend_from_slice(&[0; 4]);
    camada.extend_from_slice(&0_u16.to_le_bytes());
    camada.push(255);
    camada.extend_from_slice(&[0; 3]);
    camada.extend_from_slice(&1_u16.to_le_bytes());
    camada.push(b't');
    camada.extend_from_slice(&0_u32.to_le_bytes());
    let com_tilemap = com_chunk(&limpo, 0x2004, &camada);
    let (_, avisos) = abrir(&com_tilemap, "ase").unwrap();
    assert!(avisos.contains(&Aviso::Tilemap), "{avisos:?}");

    // Um cel cru 2×2 RGBA com 4 bytes em vez de 16: recusa ou avisa, nunca some calado.
    let mut cel = Vec::new();
    cel.extend_from_slice(&0_u16.to_le_bytes());
    cel.extend_from_slice(&[0; 4]);
    cel.push(255);
    cel.extend_from_slice(&0_u16.to_le_bytes());
    cel.extend_from_slice(&0_i16.to_le_bytes());
    cel.extend_from_slice(&[0; 5]);
    cel.extend_from_slice(&2_u16.to_le_bytes());
    cel.extend_from_slice(&2_u16.to_le_bytes());
    cel.extend_from_slice(&[9; 4]);
    if let Ok((_, avisos)) = abrir(&com_chunk(&limpo, 0x2005, &cel), "ase") {
        assert!(avisos.contains(&Aviso::CelCorrompido), "{avisos:?}");
    }
}

/// Um sprite com camada de imagem e camada de tilemap: 2 tiles, flips e um quadro ligado.
fn com_tilemap(mode: ColorMode) -> Sprite {
    use ph2d_pixel_core::frames::{NewFrame, new_frame};
    use ph2d_pixel_core::tiles::new_tilemap_layer;
    use ph2d_pixel_core::{Background, Cel, History, NewSprite, Op, TileGrid, TileRef, TilesSet};
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        mode,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer(&mut s, acima, 2, 2).unwrap();
    h.commit(&mut s, op).unwrap();
    let bpp = s.mode().bpp();
    let tile = |base: u8| -> Vec<u8> {
        (0..4)
            .flat_map(|i| {
                let v = base + i;
                match bpp {
                    4 => vec![v, v / 2, 255 - v, 255],
                    2 => vec![v, 255],
                    _ => vec![v],
                }
            })
            .collect()
    };
    let op = TilesSet {
        counts: vec![(set, 3)],
        tiles: vec![(set, 1, tile(10)), (set, 2, tile(20))],
        cels: vec![(
            (l, f),
            Some(Cel::tilemap(TileGrid {
                x: 4,
                y: 2,
                w: 3,
                h: 2,
                tiles: vec![
                    TileRef::new(1, false, false, false),
                    TileRef::EMPTY,
                    TileRef::new(2, true, false, false),
                    TileRef::new(1, false, true, false),
                    TileRef::new(2, false, false, true),
                    TileRef::new(1, true, true, true),
                ],
                ..TileGrid::default()
            })),
        )],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op))).unwrap();
    let (op, _) = new_frame(&mut s, f, NewFrame::Linked).unwrap();
    h.commit(&mut s, op).unwrap();
    s
}

/// O tilemap sobrevive ao `.ase` como tilemap: tileset, flips e ligados, sem aviso.
#[test]
fn a_tilemap_returns_from_the_ase_as_a_tilemap() {
    for mode in [ColorMode::Rgba, ColorMode::Gray, ColorMode::Indexed] {
        let s = com_tilemap(mode);
        let (bytes, avisos) = ase::escrever(&s).unwrap();
        assert!(avisos.is_empty(), "{avisos:?}");
        let (volta, avisos) = ase::ler(&bytes).unwrap();
        assert!(avisos.is_empty(), "{mode:?}: {avisos:?}");
        assert!(ph2d_pixel_core::derived_is_consistent(&volta));
        let camada = volta
            .layers()
            .iter()
            .find(|l| l.tileset().is_some())
            .expect("camada");
        let set = volta.tileset(camada.tileset().unwrap()).unwrap();
        let antes = &s.tilesets()[0];
        assert_eq!(set.tiles, antes.tiles, "{mode:?}");
        assert_eq!((set.tile_w, set.tile_h), (2, 2));
        let (fa, fb) = (s.frames()[0].id, volta.frames()[0].id);
        let la = s
            .layers()
            .iter()
            .find(|l| l.tileset().is_some())
            .unwrap()
            .id;
        assert_eq!(
            s.cel((la, fa)).unwrap().tiles.as_deref(),
            volta.cel((camada.id, fb)).unwrap().tiles.as_deref(),
            "{mode:?}"
        );
        assert!(
            volta
                .entry((camada.id, volta.frames()[1].id))
                .unwrap()
                .link
                .is_some()
        );
        for (a, b) in s.frames().iter().zip(volta.frames()) {
            assert_eq!(composto(&s, a.id), composto(&volta, b.id), "{mode:?}");
        }
    }
}

fn composto(s: &Sprite, f: FrameId) -> Vec<u8> {
    let mut out = Vec::new();
    ph2d_pixel_core::compose_rgba(s, f, None, &mut out);
    out
}
