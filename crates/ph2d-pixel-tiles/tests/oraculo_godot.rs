//! O gabarito de 47 contra o Godot corrido (M2): lê a fixtura, nada copiado para aqui.

use std::collections::BTreeSet;

use ph2d_pixel_core::tiles::new_tilemap_layer;
use ph2d_pixel_core::{Background, ColorMode, History, NewSprite, Sprite, TileRef};
use ph2d_pixel_tiles::mapa::Mapa;
use ph2d_pixel_tiles::terreno::{self, E, N, NE, NW, S, SE, SW, VIZINHOS, W};
use serde_json::Value;

fn fixtura() -> Value {
    serde_json::from_str(include_str!("fixtures/oracle_godot_autotile.json")).unwrap()
}

/// `[lados TRBL, cantos TR,BR,BL,TL]` do Godot nos nossos bits.
fn nossos_bits(m: &Value) -> u8 {
    let (lados, cantos) = (m[0].as_u64().unwrap(), m[1].as_u64().unwrap());
    let mut r = 0;
    for (b, v) in [N, E, S, W].into_iter().enumerate() {
        if lados & (1 << b) != 0 {
            r |= v;
        }
    }
    for (b, v) in [NE, SE, SW, NW].into_iter().enumerate() {
        if cantos & (1 << b) != 0 {
            r |= v;
        }
    }
    r
}

/// A casa do gabarito que a forma do Godot pede.
fn casa_do_godot(m: &Value) -> u32 {
    let forma = nossos_bits(m);
    terreno::formas()
        .iter()
        .position(|&f| f == forma)
        .unwrap_or_else(|| panic!("o Godot deu uma forma fora das 47: {m}")) as u32
}

#[test]
fn o_terreno_bate_com_o_godot_nas_256_vizinhancas() {
    let v = fixtura();
    let linhas = v["M2A"][0]["linhas"].as_array().unwrap();
    assert_eq!(linhas.len(), 256);
    let mut vistas = BTreeSet::new();
    for l in linhas {
        let n = l[0].as_u64().unwrap() as u8;
        vistas.insert(n);
        assert_eq!(terreno::casa(n), casa_do_godot(&l[2]), "vizinhança {n}");
    }
    assert_eq!(vistas.len(), 256);
}

/// O mapa real: o nosso `pinta` sobre as mesmas células dá, em cada uma, a forma que o Godot deu.
#[test]
fn o_terreno_bate_com_o_godot_no_mapa_de_376_celulas() {
    let v = fixtura();
    let linhas = v["M2B"][0]["linhas"].as_array().unwrap();
    assert_eq!(linhas.len(), 376);
    let celulas: Vec<(i32, i32)> = linhas
        .iter()
        .map(|l| (l[0].as_i64().unwrap() as i32, l[1].as_i64().unwrap() as i32))
        .collect();
    let mut s = Sprite::new(NewSprite {
        width: 64,
        height: 64,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer(&mut s, acima, 2, 2).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::novo_terreno(&s, set, "Grass").unwrap();
    h.commit(&mut s, op).unwrap();
    let first = s.tileset(set).unwrap().terrains[0].first;
    let op = terreno::pinta(&s, l, f, &celulas, Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let m = Mapa::de(&s, l, f, s.tileset(set).unwrap());
    let mut formas = BTreeSet::new();
    for li in linhas {
        let (c, r) = (
            li[0].as_i64().unwrap() as i32,
            li[1].as_i64().unwrap() as i32,
        );
        let quer = casa_do_godot(&li[3]);
        formas.insert(quer);
        assert_eq!(
            m.get(c, r),
            TileRef::new(first + quer, false, false, false),
            "célula {c},{r}"
        );
    }
    assert_eq!(formas.len(), 44, "as formas que a fixtura exercita");
    // E à parte, do nosso lado: a vizinhança lida no mapa dá a mesma casa.
    let dentro: BTreeSet<(i32, i32)> = celulas.iter().copied().collect();
    for &(c, r) in &celulas {
        let mut n = 0_u8;
        for (b, (dx, dy)) in VIZINHOS.iter().enumerate() {
            if dentro.contains(&(c + dx, r + dy)) {
                n |= 1 << b;
            }
        }
        assert_eq!(m.get(c, r).id(), first + terreno::casa(n));
    }
}

/// ⛔ Slot por preencher: o Godot escolhe o tile mais próximo (M2C); nós não. A célula fica com a
/// casa dela, ainda que vazia, para o artista ver o buraco (`P10_plano.md`, Recusas MEDIDAS).
#[test]
fn slot_em_falta_nao_escolhe_outro_tile() {
    let v = fixtura();
    let corrida = v["M2C"].as_array().unwrap();
    let divergiu = corrida
        .iter()
        .flat_map(|c| c["amostra_pedida_x_escolhida"].as_array().unwrap());
    let trocas = divergiu
        .filter(|a| nossos_bits(&a[2]) != nossos_bits(&a[3]))
        .count();
    assert!(trocas > 0, "a fixtura mostra o Godot a trocar de tile");
    // O nosso: todas as casas continuam a ser as da forma, com o gabarito só em tiles vazios.
    let mut s = Sprite::new(NewSprite {
        width: 16,
        height: 16,
        mode: ColorMode::Rgba,
        background: Background::Transparent,
    })
    .unwrap();
    let mut h = History::default();
    let (acima, f) = (s.layers()[0].id, s.frames()[0].id);
    let (op, l, set) = new_tilemap_layer(&mut s, acima, 2, 2).unwrap();
    h.commit(&mut s, op).unwrap();
    let op = terreno::novo_terreno(&s, set, "Grass").unwrap();
    h.commit(&mut s, op).unwrap();
    let first = s.tileset(set).unwrap().terrains[0].first;
    let op = terreno::pinta(&s, l, f, &[(1, 1)], Some(0)).unwrap();
    h.commit(&mut s, op).unwrap();
    let m = Mapa::de(&s, l, f, s.tileset(set).unwrap());
    assert_eq!(
        m.get(1, 1).id(),
        first + terreno::casa(0),
        "a casa do isolado, vazia"
    );
}
