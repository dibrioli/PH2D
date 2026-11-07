//! **A aceitação congelada da P10** (`docs/PixelArt/P10_plano.md` §7): um sprite animado com camada de
//! tilemap, desenhado nos 3 modos e em Draw Tiles, com flips e terreno 47 sobre um mapa real, que grava,
//! reabre igual, vai a `.ase` e a PNG + JSON que o Godot remonta (`oraculo_godot_remonta.py`).

use std::collections::BTreeMap;
use std::path::Path;

use ph2d_pixel_core::canvas::{Borders, canvas_size};
use ph2d_pixel_core::codec::{read_library, write_library};
use ph2d_pixel_core::frames::{NewFrame, new_frame};
use ph2d_pixel_core::tiles::new_tilemap_layer_forma;
use ph2d_pixel_core::{
    Background, ColorMode, Forma, FrameId, History, LayerId, NewSprite, Op, Sprite, TileRef,
    TilesSet, TilesetId, derived_is_consistent,
};
use ph2d_pixel_io::ase;
use ph2d_pixel_io::exporta::{Export, ExportFolha, gravar, lembrar, o_ultimo, repetir};
use ph2d_pixel_io::render::{Camadas, composto};
use ph2d_pixel_io::sheet::{OpcoesFolha, montar};
use ph2d_pixel_io::tiles_saida::{ExportTiles, atlas, json_do_mapa};
use ph2d_pixel_tiles::desenho::{Modo, aplica};
use ph2d_pixel_tiles::exemplo::celulas_reais;
use ph2d_pixel_tiles::mapa::{Mapa, carimba};
use ph2d_pixel_tiles::terreno;
use serde_json::Value;
use sha2::{Digest, Sha256};

const TILE: u16 = 16;
const COLUNAS: u32 = 8;
const VERDE: u32 = 0xff3c_a050;
const AZUL: u32 = 0xffd0_6020;
const VERMELHO: u32 = 0xff20_20e0;
const CANVAS: (u32, u32) = (448, 320);

struct Obra {
    s: Sprite,
    h: History,
    l: LayerId,
    f: FrameId,
    set: TilesetId,
}

impl Obra {
    fn faz(&mut self, op: Option<Op>) {
        self.h.commit(&mut self.s, op.unwrap()).unwrap();
        assert!(derived_is_consistent(&self.s));
    }

    fn mapa(&self) -> Mapa {
        Mapa::de(&self.s, self.l, self.f, self.s.tileset(self.set).unwrap())
    }

    fn tile(&mut self, cor: u32, marca: u32) -> u32 {
        let n = self.s.tileset(self.set).unwrap().tiles.len() as u32;
        let (w, h) = (usize::from(TILE), usize::from(TILE));
        let mut px = cor.to_le_bytes().repeat(w * h);
        // Assimétrico: é assim que um flip errado aparece.
        for i in 0..4 {
            px[i * 4..i * 4 + 4].copy_from_slice(&marca.to_le_bytes());
        }
        let op = TilesSet {
            counts: vec![(self.set, n + 1)],
            tiles: vec![(self.set, n, px)],
            ..TilesSet::default()
        };
        self.faz(Some(Op::Tiles(Box::new(op))));
        n
    }

    /// Um píxel no meio da célula `(c, r)` no modo dado: o traço real, célula a célula.
    fn pixel(&mut self, c: i32, r: i32, cor: u32, modo: Modo) {
        let (x, y) = self.mapa().origem(c, r);
        let m = i32::from(TILE) / 2;
        let op = aplica(&self.s, self.l, self.f, &[(x + m, y + m, cor)], modo);
        self.faz(op);
    }
}

/// A obra: o terreno no mapa real, tiles à mão com flips, e o traço nos três modos, em dois quadros.
fn obra(forma: Forma) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: CANVAS.0,
        height: CANVAS.1,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer_forma(&mut s, acima, TILE, TILE, forma).unwrap();
    h.commit(&mut s, op).unwrap();
    let mut o = Obra { s, h, l, f, set };
    let a = o.tile(AZUL, VERMELHO);
    let b = o.tile(VERMELHO, AZUL);
    let tem_terreno = forma.tem_terreno();
    if tem_terreno {
        let op = terreno::novo_terreno_com_guia(&o.s, set, "Grass", VERDE);
        o.faz(op);
        let celulas: Vec<_> = celulas_reais().iter().map(|&(c, r)| (c, r + 2)).collect();
        assert!(celulas.len() >= 200);
        let op = terreno::pinta(&o.s, l, o.f, &celulas, Some(0));
        o.faz(op);
    }
    // Draw Tiles, nas oito combinações de flips.
    for k in 0..8 {
        let t = TileRef::new(a, k & 1 != 0, k & 2 != 0, k & 4 != 0);
        let op = carimba(&o.s, l, o.f, &[(k, 0)], t);
        o.faz(op);
    }
    let op = carimba(
        &o.s,
        l,
        o.f,
        &[(9, 0), (10, 0), (11, 0)],
        TileRef::new(b, false, false, false),
    );
    o.faz(op);
    // Os três modos: Manual muda as três cópias, Auto só uma, Stack faz um tile novo.
    let n0 = o.s.tileset(set).unwrap().tiles.len();
    o.pixel(9, 0, VERDE, Modo::Manual);
    assert_eq!(
        o.s.tileset(set).unwrap().tiles.len(),
        n0,
        "Manual não cria tile"
    );
    o.pixel(10, 0, AZUL, Modo::Auto);
    assert_eq!(
        o.s.tileset(set).unwrap().tiles.len(),
        n0 + 1,
        "Auto separa a cópia"
    );
    o.pixel(11, 0, VERMELHO, Modo::Stack);
    assert_eq!(
        o.s.tileset(set).unwrap().tiles.len(),
        n0 + 2,
        "Stack faz um tile novo"
    );
    // Animado: o quadro 2 tem outro mapa.
    let (op, f2) = new_frame(&mut o.s, o.f, NewFrame::Empty).unwrap();
    o.faz(Some(op));
    o.f = f2;
    let t = TileRef::new(b, true, false, false);
    let op = carimba(&o.s, l, f2, &[(1, 1), (2, 1), (3, 1), (2, 2)], t);
    o.faz(op);
    assert_eq!(o.s.frames().len(), 2);
    // O canvas desloca a grelha: a fase tem de chegar ao JSON (`origin`) e ao Godot.
    let bordas = Borders {
        left: 5,
        top: 3,
        right: 5,
        bottom: 3,
    };
    let t = o.s.transparent_value();
    let op = canvas_size(&o.s, bordas, t);
    o.faz(op);
    o.s
}

fn sha(partes: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in partes {
        h.update(p);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

type Saida = (String, (u32, u32, Vec<u8>), Vec<Vec<u8>>);

/// O que o Godot leu: o JSON, a imagem dos tiles e o composto de cada quadro.
fn saida(forma: Forma) -> Saida {
    let s = obra(forma);
    let set = s.tilesets()[0].clone();
    let j = json_do_mapa(&s, &set, "tiles.png", COLUNAS);
    let img = atlas(&s, &set, COLUNAS);
    let quadros = s
        .frames()
        .iter()
        .map(|f| composto(&s, f.id, Camadas::Visiveis))
        .collect();
    (j, img, quadros)
}

#[test]
fn o_godot_remonta_o_mapa() {
    let o: Value = serde_json::from_str(include_str!(
        "../../ph2d-pixel-tiles/tests/fixtures/oracle_godot_remonta.json"
    ))
    .unwrap();
    for c in o["casos"].as_array().unwrap() {
        let forma = Forma::TODAS
            .into_iter()
            .find(|f| f.nome() == c["forma"])
            .unwrap();
        let (j, (_, _, img), quadros) = saida(forma);
        assert_eq!(
            c["json_sha"],
            sha(&[j.as_bytes()]),
            "{forma:?}: refazer o oráculo"
        );
        assert_eq!(c["tiles_sha"], sha(&[&img]), "{forma:?}: refazer o oráculo");
        assert_eq!(c["quadros"], quadros.len(), "{forma:?}");
        for (q, d) in c["diferem"].as_array().unwrap().iter().enumerate() {
            assert_eq!(*d, 0, "{forma:?}, quadro {q}: píxeis diferentes do Godot");
        }
        let qs: Vec<&[u8]> = quadros.iter().map(Vec::as_slice).collect();
        assert_eq!(c["nossos_sha"], sha(&qs), "{forma:?}");
    }
    assert_eq!(o["casos"].as_array().unwrap().len(), Forma::TODAS.len());
}

/// Grava o que o oráculo lê. Corre só com `PH2D_P10C_SAIDA`, pela mão do script.
#[test]
fn escreve_a_entrada_do_oraculo() {
    let Ok(pasta) = std::env::var("PH2D_P10C_SAIDA") else {
        return;
    };
    for forma in Forma::TODAS {
        let d = Path::new(&pasta).join(forma.nome());
        std::fs::create_dir_all(&d).unwrap();
        let s = obra(forma);
        let set = s.tilesets()[0].id;
        let e = Export::Tiles(ExportTiles {
            conjunto: set,
            colunas: COLUNAS,
        });
        let g = gravar(&s, &e, &d.join("tiles.png")).unwrap();
        assert_eq!(g.ficheiros.len(), 2);
        std::fs::rename(d.join("tiles.json"), d.join("mapa.json")).unwrap();
        let (_, _, quadros) = saida(forma);
        for (q, px) in quadros.iter().enumerate() {
            std::fs::write(d.join(format!("nosso_{q}.rgba")), px).unwrap();
        }
        let (j, (_, _, img), _) = saida(forma);
        std::fs::write(d.join("json_sha.txt"), sha(&[j.as_bytes()])).unwrap();
        std::fs::write(d.join("tiles_sha.txt"), sha(&[&img])).unwrap();
    }
    // O `.ase` da aceitação, para o Trial abrir (`trial_tiles_p10c.py`).
    let (bytes, _) = ase::escrever(&obra(Forma::Square)).unwrap();
    std::fs::write(Path::new(&pasta).join("aceitacao.ase"), bytes).unwrap();
}

#[test]
fn a_obra_grava_e_reabre_igual() {
    for forma in Forma::TODAS {
        let s = obra(forma);
        let bytes = write_library(&BTreeMap::from([(1, s.clone())]));
        let lido = read_library(&bytes).unwrap().remove(&1).unwrap();
        assert!(derived_is_consistent(&lido), "{forma:?}");
        assert_eq!(lido.tilesets(), s.tilesets(), "{forma:?}");
        for f in s.frames() {
            assert_eq!(
                composto(&lido, f.id, Camadas::Visiveis),
                composto(&s, f.id, Camadas::Visiveis),
                "{forma:?}"
            );
        }
    }
}

#[test]
fn o_ase_de_aceitacao_volta_igual_e_iso_hex_saem_como_imagem() {
    let s = obra(Forma::Square);
    let (bytes, avisos) = ase::escrever(&s).unwrap();
    assert!(avisos.contains(&ase::Aviso::TerrenosFicam));
    assert!(!avisos.contains(&ase::Aviso::FormaVaiComoImagem));
    let (lido, _) = ase::ler(&bytes).unwrap();
    assert_eq!(lido.tilesets()[0].tiles, s.tilesets()[0].tiles);
    assert!(lido.layers().iter().any(|l| l.tileset().is_some()));
    for (a, b) in s.frames().iter().zip(lido.frames()) {
        assert_eq!(
            composto(&s, a.id, Camadas::Visiveis),
            composto(&lido, b.id, Camadas::Visiveis)
        );
    }
    for forma in [Forma::Iso, Forma::Hex, Forma::HexFlat] {
        let s = obra(forma);
        let (bytes, avisos) = ase::escrever(&s).unwrap();
        assert!(
            avisos.contains(&ase::Aviso::FormaVaiComoImagem),
            "{forma:?}"
        );
        let (lido, _) = ase::ler(&bytes).unwrap();
        assert!(
            lido.tilesets().is_empty(),
            "{forma:?}: sem tileset no ficheiro"
        );
        assert!(
            lido.layers().iter().all(|l| l.tileset().is_none()),
            "{forma:?}"
        );
        for (a, b) in s.frames().iter().zip(lido.frames()) {
            assert_eq!(
                composto(&s, a.id, Camadas::Visiveis),
                composto(&lido, b.id, Camadas::Visiveis),
                "{forma:?}: a arte não muda"
            );
        }
    }
}

/// Um sprite 44 × 24 de tile 16 × 8 com um só píxel; `fase` desloca a grelha por píxeis.
fn folha_de_um_pixel(fase: i32) -> Sprite {
    folha_com_pixel(fase, 4, (1, 1))
}

/// O píxel no `lx` da linha 2 do tile, e o tile na célula dada.
fn folha_com_pixel(fase: i32, lx: usize, celula: (i32, i32)) -> Sprite {
    let mut s = Sprite::new(NewSprite {
        width: 40,
        height: 24,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer_forma(&mut s, acima, 16, 8, Forma::Square).unwrap();
    h.commit(&mut s, op).unwrap();
    // Com `lx` 4 e a célula (1, 1): em (20, 10) do sprite, longe das linhas da grelha.
    let mut px = vec![0; 16 * 8 * 4];
    px[(2 * 16 + lx) * 4..(2 * 16 + lx) * 4 + 4].copy_from_slice(&VERDE.to_le_bytes());
    let op = TilesSet {
        counts: vec![(set, 2)],
        tiles: vec![(set, 1, px)],
        ..TilesSet::default()
    };
    h.commit(&mut s, Op::Tiles(Box::new(op))).unwrap();
    let op = carimba(&s, l, f, &[celula], TileRef::new(1, false, false, false)).unwrap();
    h.commit(&mut s, op).unwrap();
    if fase != 0 {
        let b = Borders {
            left: fase,
            top: 0,
            right: 0,
            bottom: 0,
        };
        let op = canvas_size(&s, b, s.transparent_value()).unwrap();
        h.commit(&mut s, op).unwrap();
    }
    s
}

/// `--split-grid`, medido no Trial (M10): linha a linha; as parciais à direita e em baixo saem, as à
/// esquerda e em cima ficam inteiras; os recortes não mudam nada.
#[test]
fn split_grid_segue_o_trial() {
    let s = folha_de_um_pixel(0);
    let base = OpcoesFolha {
        split_grid: true,
        ..OpcoesFolha::default()
    };
    let fo = montar(&s, &base, "t", "png");
    // 40 × 24 em tiles de 16 × 8: 2 colunas (a parcial sai) × 3 linhas.
    assert_eq!(fo.entradas.len(), 6, "{:?}", fo.entradas);
    assert_eq!(fo.entradas[0].origem, [0, 0, 16, 8]);
    assert_eq!(fo.entradas[1].origem, [16, 0, 16, 8]);
    assert_eq!(
        fo.entradas[3].origem,
        [16, 8, 16, 8],
        "o tile cai no índice 3"
    );
    let trim = OpcoesFolha {
        trim_cels: true,
        trim_by_grid: true,
        ..base.clone()
    };
    assert_eq!(
        montar(&s, &trim, "t", "png").entradas.len(),
        6,
        "os recortes não mudam nada"
    );
    let cheias = OpcoesFolha {
        ignore_empty: true,
        ..base.clone()
    };
    let fo = montar(&s, &cheias, "t", "png");
    assert_eq!(fo.entradas.len(), 1);
    assert_eq!(fo.entradas[0].origem, [16, 8, 16, 8]);
    // Fase 4: a coluna de x = -12 fica inteira, transparente fora do canvas.
    let s = folha_de_um_pixel(4);
    let fo = montar(&s, &base, "t", "png");
    assert_eq!(fo.entradas.len(), 9, "{:?}", fo.entradas);
    assert_eq!(fo.entradas[0].origem[2..], [16, 8]);
    assert_eq!(fo.entradas[1].origem, [4, 0, 16, 8]);
    // Tinta na coluna 0 do canvas: o que sai do canvas fica transparente, não copia a borda.
    let s = folha_com_pixel(4, 12, (-1, 1));
    let ver = OpcoesFolha {
        ignore_empty: true,
        ..base.clone()
    };
    let fo = montar(&s, &ver, "t", "png");
    assert_eq!(fo.entradas.len(), 1, "{:?}", fo.entradas);
    let [fx, fy, ..] = fo.entradas[0].frame;
    let alfa =
        |x: u32, y: u32| fo.rgba[((fy + y) as usize * fo.w as usize + (fx + x) as usize) * 4 + 3];
    assert_eq!(alfa(12, 2), 255, "a tinta na coluna 0 do canvas");
    assert_eq!(alfa(11, 2), 0, "fora do canvas não se copia a borda");
    // Sem camada de tilemap não há grelha: a folha fica como sem a opção.
    let sem = Sprite::new(NewSprite {
        width: 20,
        height: 20,
        ..NewSprite::default()
    })
    .unwrap();
    assert_eq!(montar(&sem, &base, "t", "png").entradas.len(), 1);
}

/// `--trim-by-grid`, medido no Trial (M10): alarga o recorte às linhas da grelha, e sozinho não faz nada.
#[test]
fn trim_by_grid_segue_o_trial() {
    let s = folha_de_um_pixel(0);
    let origem = |o: OpcoesFolha| montar(&s, &o, "t", "png").entradas[0].origem;
    let so = |t: bool, st: bool, g: bool| OpcoesFolha {
        trim_cels: t,
        trim_sprite: st,
        trim_by_grid: g,
        ..OpcoesFolha::default()
    };
    assert_eq!(origem(so(true, false, false)), [20, 10, 1, 1]);
    assert_eq!(origem(so(false, true, false)), [20, 10, 1, 1]);
    assert_eq!(origem(so(true, false, true)), [16, 8, 16, 8]);
    assert_eq!(origem(so(false, true, true)), [16, 8, 16, 8]);
    assert_eq!(
        origem(so(false, false, true)),
        [0, 0, 40, 24],
        "sem recorte não faz nada"
    );
    // Com a grelha deslocada, as linhas andam com ela: o píxel está agora em (24, 10), e a coluna em 20.
    let s = folha_de_um_pixel(4);
    let f = montar(&s, &so(true, false, true), "t", "png");
    assert_eq!(f.entradas[0].origem, [20, 8, 16, 8]);
}

/// Repeat Last Export volta a escrever os mesmos ficheiros, e a forma de sempre continua a ler-se.
#[test]
fn o_repeat_lembra_o_export_tileset_e_as_opcoes_da_grelha() {
    let dir = std::env::temp_dir().join(format!("ph2d_repeat_tiles_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = obra(Forma::Iso);
    let e = Export::Tiles(ExportTiles {
        conjunto: s.tilesets()[0].id,
        colunas: 5,
    });
    let c = dir.join("t.png");
    lembrar(&mut s, &e, &c);
    assert_eq!(o_ultimo(&s).unwrap().0, Some(e));
    let g = repetir(&s).unwrap().unwrap();
    assert_eq!(g.ficheiros, [c.clone(), dir.join("t.json")]);
    let j: Value = serde_json::from_slice(&std::fs::read(dir.join("t.json")).unwrap()).unwrap();
    assert_eq!(j["columns"], 5);
    assert_eq!(j["shape"], "iso");
    let f = Export::Folha(ExportFolha {
        opcoes: OpcoesFolha {
            split_grid: true,
            trim_by_grid: true,
            ..OpcoesFolha::default()
        },
        json: None,
    });
    lembrar(&mut s, &f, &c);
    assert_eq!(o_ultimo(&s).unwrap().0, Some(f));
    std::fs::remove_dir_all(&dir).unwrap();
}
