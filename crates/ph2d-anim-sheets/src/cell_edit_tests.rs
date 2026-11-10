//! Trim, Padding e Make Square numa folha cortada (dono 09/11, opção A): a grelha fica inteira e cada quadro fica no
//! lugar.

use super::*;
use ph2d_ecs::{ExtraSheet, ShownSheet, SpriteSheets, Transform};

/// Uma folha 2 × 1 de células 8 × 8: o quadro 0 com desenho em `x 2..5, y 4..8`, o 1 em `x 3..6, y 2..7`. Cada
/// píxel opaco tem uma cor só dele (`x`, `y`, o quadro), para a cópia se poder seguir.
fn folha() -> Vec<u8> {
    let mut px = vec![0_u8; 16 * 8 * 4];
    for (q, [x0, x1, y0, y1]) in [[2, 5, 4, 8], [3, 6, 2, 7]].into_iter().enumerate() {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * 16 + q * 8 + x) * 4;
                px[i..i + 4].copy_from_slice(&[x as u8, y as u8, q as u8, 255]);
            }
        }
    }
    px
}

const CORTE: Cut = Cut {
    cols: 2,
    rows: 1,
    cell: [8, 8],
    m_per_px: [0.01, 0.01],
};

/// Onde cada píxel opaco de cada quadro cai em metros LOCAIS (o quad centrado no Pivot `anchor`, o flip a virar os
/// píxeis dentro dele), pela cor que o identifica.
fn no_lugar(
    px: &[u8],
    w: u32,
    cut: &Cut,
    anchor: [f32; 2],
    [fx, fy]: [bool; 2],
) -> Vec<([u8; 3], [f32; 2])> {
    let mut v = Vec::new();
    for q in 0..cut.cols {
        for y in 0..cut.cell[1] {
            for x in 0..cut.cell[0] {
                let i = ((y * w + q * cut.cell[0] + x) * 4) as usize;
                if px[i + 3] == 0 {
                    continue;
                }
                let lx = (x as f32 + 0.5 - cut.cell[0] as f32 * 0.5) * cut.m_per_px[0];
                let ly = (cut.cell[1] as f32 * 0.5 - y as f32 - 0.5) * cut.m_per_px[1];
                let s = |f: bool, a: f32| if f { -a } else { a };
                v.push((
                    [px[i], px[i + 1], px[i + 2]],
                    [anchor[0] + s(fx, lx), anchor[1] + s(fy, ly)],
                ));
            }
        }
    }
    v.sort_by_key(|a| a.0);
    v
}

fn perto(a: &[([u8; 3], [f32; 2])], b: &[([u8; 3], [f32; 2])]) -> bool {
    a.len() == b.len() && sobra_no_lugar(a, b)
}

/// Todo píxel que SOBROU em `b` está onde estava em `a` (um corte tira píxeis, nunca os move).
fn sobra_no_lugar(a: &[([u8; 3], [f32; 2])], b: &[([u8; 3], [f32; 2])]) -> bool {
    !b.is_empty()
        && b.iter().all(|q| {
            a.iter().any(|p| {
                p.0 == q.0 && (p.1[0] - q.1[0]).abs() < 1e-5 && (p.1[1] - q.1[1]).abs() < 1e-5
            })
        })
}

/// ⭐ **Cada píxel de cada quadro fica onde estava**, com e sem flip, nos três verbos: a grelha continua `2 × 1` e só o
/// Pivot da folha anda.
///
/// *Mutações: tirar o termo `at` do `pivot_shift` ⇒ o desenho anda e reprova; ignorar o flip ⇒ reprova com flip;
/// trocar o sinal do `y` ⇒ reprova; cortar a imagem INTEIRA (o defeito do smoke) ⇒ a célula deixa de ser inteira.*
#[test]
fn cada_quadro_fica_no_lugar_nos_tres_verbos() {
    let px = folha();
    let antes_do_pivo = [0.03, -0.02];
    let trim = trim(&px, 16, &CORTE).expect("há margem");
    assert_eq!(
        trim,
        CellChange {
            cell: [4, 6],
            at: [-2, -2]
        },
        "a união das caixas"
    );
    let square = square(&Cut {
        cell: [8, 6],
        ..CORTE
    });
    assert_eq!(
        square,
        Some(CellChange {
            cell: [8, 8],
            at: [0, 1]
        })
    );
    let pad = padding(&px, 16, &CORTE, PaddingSpec::uniform(2)).expect("cresce");
    assert_eq!(
        pad,
        CellChange {
            cell: [12, 12],
            at: [2, 2]
        }
    );
    let corta = padding(&px, 16, &CORTE, PaddingSpec::uniform(-1)).expect("corta");
    assert_eq!(
        corta,
        CellChange {
            cell: [6, 6],
            at: [-1, -1]
        }
    );
    for ch in [trim, pad, corta] {
        for flip in [[false, false], [true, false], [false, true], [true, true]] {
            let antes = no_lugar(&px, 16, &CORTE, antes_do_pivo, flip);
            let (novo, nw, nh) = recompose(&px, 16, &CORTE, ch);
            assert_eq!((nw, nh), (ch.cell[0] * 2, ch.cell[1]));
            let d = pivot_shift(&CORTE, ch, flip);
            let depois_do_pivo = [antes_do_pivo[0] + d[0], antes_do_pivo[1] + d[1]];
            let novo_corte = Cut {
                cell: ch.cell,
                ..CORTE
            };
            let depois = no_lugar(&novo, nw, &novo_corte, depois_do_pivo, flip);
            let ok = if ch == corta {
                depois.len() < antes.len() && sobra_no_lugar(&antes, &depois)
            } else {
                perto(&antes, &depois)
            };
            assert!(ok, "{ch:?} {flip:?}: {antes:?} ≠ {depois:?}");
        }
    }
}

/// O Make Square numa folha de células `8 × 6`: a moldura de cada uma, o desenho no mesmo lugar.
#[test]
fn o_make_square_emoldura_cada_celula() {
    let corte = Cut {
        cell: [8, 4],
        ..CORTE
    };
    let px = {
        let mut p = vec![0_u8; 16 * 4 * 4];
        for (i, v) in p.iter_mut().enumerate() {
            *v = if i % 4 == 3 { 255 } else { (i / 4) as u8 };
        }
        p
    };
    let ch = square(&corte).expect("não é quadrada");
    assert_eq!(
        ch,
        CellChange {
            cell: [8, 8],
            at: [0, 2]
        }
    );
    let (novo, nw, nh) = recompose(&px, 16, &corte, ch);
    assert_eq!((nw, nh), (16, 8));
    let antes = no_lugar(&px, 16, &corte, [0.0, 0.0], [false, false]);
    let d = pivot_shift(&corte, ch, [false, false]);
    let depois = no_lugar(
        &novo,
        nw,
        &Cut {
            cell: ch.cell,
            ..corte
        },
        d,
        [false, false],
    );
    assert!(perto(&antes, &depois));
    assert!(
        square(&Cut {
            cell: [8, 8],
            ..corte
        })
        .is_none(),
        "já quadrada"
    );
}

/// Os 16 bits atravessam a mesma cópia (o verbo não calcula valor de píxel nenhum).
#[test]
fn os_dezasseis_bits_passam_intactos() {
    let px: Vec<u16> = (0..16 * 8 * 4_u16).map(|i| i.wrapping_mul(97)).collect();
    let (oito, ..) = recompose(
        &folha(),
        16,
        &CORTE,
        CellChange {
            cell: [4, 6],
            at: [-2, -2],
        },
    );
    let (dezasseis, nw, nh) = recompose(
        &px,
        16,
        &CORTE,
        CellChange {
            cell: [4, 6],
            at: [-2, -2],
        },
    );
    assert_eq!((nw, nh, dezasseis.len()), (8, 6, oito.len()));
    // O píxel (0, 0) do quadro 1 novo é o (2, 2) do quadro 1 velho: índice (2·16 + 8 + 2)·4.
    let i = ((2 * 16 + 8 + 2) * 4) as u16;
    let v = |k: u16| (i + k).wrapping_mul(97);
    assert_eq!(&dezasseis[4 * 4..4 * 4 + 4], &[v(0), v(1), v(2), v(3)]);
}

/// Nada a cortar: as caixas já tocam as bordas, ou a folha é toda transparente.
#[test]
fn sem_margem_nao_ha_trim() {
    let cheia = vec![255_u8; 16 * 8 * 4];
    assert!(trim(&cheia, 16, &CORTE).is_none());
    assert!(trim(&vec![0; 16 * 8 * 4], 16, &CORTE).is_none());
    assert!(padding(&cheia, 16, &CORTE, PaddingSpec::uniform(0)).is_none());
}

/// O corte da folha visível: a `0` pela grelha do objeto, a extra pela dela; uma célula só ou píxeis partidos não são
/// corte (o verbo age na imagem inteira).
#[test]
fn o_corte_e_o_da_folha_visivel() {
    let mut w = World::new();
    let sheet = ExtraSheet {
        asset: ph2d_asset::AssetId::from_digest([7; 32]),
        size: [0.48, 0.48],
        anchor: [0.0, 0.0],
        hframes: 6,
        vframes: 2,
        offsets: Default::default(),
    };
    let e = w
        .spawn((
            ph2d_render::Sprite::atlas(0, [0.5, 0.25], [1.0; 4]),
            Transform::default(),
            SpriteGrid {
                hframes: 4,
                vframes: 1,
                frame: 0,
                sheet: 0,
            },
            SpriteSheets([sheet].into_iter().collect()),
            ShownSheet(1),
        ))
        .id();
    assert_eq!(
        cut_of(&w, e, 0, 200, 25),
        Some(Cut {
            cols: 4,
            rows: 1,
            cell: [50, 25],
            m_per_px: [0.01, 0.01]
        })
    );
    assert_eq!(
        cut_of(&w, e, 1, 288, 96),
        Some(Cut {
            cols: 6,
            rows: 2,
            cell: [48, 48],
            m_per_px: [0.01, 0.01]
        })
    );
    assert_eq!(
        cut_of(&w, e, 1, 269, 90),
        None,
        "o corte de 44,8 px não é corte"
    );
    w.entity_mut(e).insert(SpriteGrid::SINGLE);
    assert_eq!(cut_of(&w, e, 0, 200, 25), None, "uma célula só é um Still");
}

/// O tamanho que o funil grava e o Pivot que anda: a folha `0` pelo `Sprite`, a extra pela dela e sem tocar no resto.
#[test]
fn o_pivot_que_anda_e_o_da_folha_editada() {
    let mut w = World::new();
    let sheet = ExtraSheet {
        asset: ph2d_asset::AssetId::from_digest([7; 32]),
        size: [0.48, 0.48],
        anchor: [0.1, 0.2],
        hframes: 6,
        vframes: 2,
        offsets: Default::default(),
    };
    let e = w
        .spawn((
            ph2d_render::Sprite::atlas(0, [0.5, 0.25], [1.0; 4]),
            Transform::default(),
            SpriteSheets([sheet].into_iter().collect()),
        ))
        .id();
    let ch = CellChange {
        cell: [31, 42],
        at: [-9, -6],
    };
    let corte = Cut {
        cols: 6,
        rows: 2,
        cell: [48, 48],
        m_per_px: [0.01, 0.01],
    };
    let [a, b] = commit_size(&corte, ch, 1);
    assert!(
        (a - 1.86).abs() < 1e-5 && (b - 0.84).abs() < 1e-5,
        "a imagem inteira numa folha extra"
    );
    let [a, b] = commit_size(&corte, ch, 0);
    assert!(
        (a - 0.31).abs() < 1e-5 && (b - 0.42).abs() < 1e-5,
        "uma célula na folha 0"
    );
    keep_drawing(&mut w, e, 1, &corte, [0.5, -0.5]);
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [0.6, -0.3]);
    assert_eq!(w.get::<ph2d_render::Sprite>(e).unwrap().anchor, [0.0, 0.0]);
    assert_eq!(
        w.get::<Transform>(e).unwrap().translation,
        Transform::default().translation
    );
    keep_drawing(&mut w, e, 0, &corte, [0.5, -0.5]);
    assert_eq!(w.get::<ph2d_render::Sprite>(e).unwrap().anchor, [0.5, -0.5]);
}

/// Folha 0 = o `Sprite` 2 × 2 com Pivot no centro; folha 1 = um quad 3 × 3 com Pivot fora do centro, à vista.
fn animada_de_duas_folhas(w: &mut World, animada: bool) -> Entity {
    let extra = ExtraSheet {
        asset: ph2d_asset::AssetId::from_digest([7; 32]),
        size: [3.0, 3.0],
        anchor: [0.3, 0.1],
        hframes: 1,
        vframes: 1,
        offsets: Default::default(),
    };
    let mut t = Transform::default();
    (t.translation.x, t.translation.y) = (5.0, 7.0);
    let e = w
        .spawn((
            ph2d_render::Sprite::atlas(0, [2.0, 2.0], [1.0; 4]),
            t,
            SpriteSheets([extra].into_iter().collect()),
            ShownSheet(1),
        ))
        .id();
    if animada {
        w.entity_mut(e).insert(ph2d_ecs::SpriteAnimations::new());
    }
    e
}

/// O centro do quad da folha 0 no mundo (sem rotação nem escala).
fn centro_da_folha_0(w: &World, e: Entity) -> [f32; 2] {
    let t = w.get::<Transform>(e).unwrap().translation;
    let a = w
        .get::<ph2d_render::Sprite>(e)
        .unwrap()
        .resolve_anchor(100.0);
    [t.x + a[0], t.y + a[1]]
}

/// ⭐ **Padding + Recenter numa Animated Sprite de várias folhas não mexe em desenho nenhum** (GEF 29: o Pivot de uma
/// Animated só se edita no editor de animações). Com a folha 1 à vista, o `set_pivot` antigo levava a origem e o
/// desenho da folha 0 saltava no mundo. Controlo: num Still a mesma chamada ainda centra o Pivot da folha visível.
#[test]
fn o_recenter_numa_animated_nao_mexe_em_folha_nenhuma() {
    let (db, atlas) = (
        ph2d_asset::AssetDb::default(),
        std::collections::BTreeMap::new(),
    );
    let mut w = World::new();
    let e = animada_de_duas_folhas(&mut w, true);
    let antes = centro_da_folha_0(&w, e);
    let extra_antes = w.get::<SpriteSheets>(e).unwrap().0[0].anchor;
    recenter(&mut w, e, &db, &atlas, 100.0);
    let depois = centro_da_folha_0(&w, e);
    assert!(
        (antes[0] - depois[0]).abs() < 1e-5 && (antes[1] - depois[1]).abs() < 1e-5,
        "a folha 0 andou no mundo: {antes:?} → {depois:?}"
    );
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, extra_antes);
    let mut w = World::new();
    let e = animada_de_duas_folhas(&mut w, false);
    assert!(
        recenter(&mut w, e, &db, &atlas, 100.0),
        "controlo: o Still centra"
    );
    assert_eq!(w.get::<SpriteSheets>(e).unwrap().0[0].anchor, [0.0, 0.0]);
}

/// ⭐ **A folha 0 sem `centered` (ou com `offset`): Trim/Padding/Make Square deixam cada quadro no lugar.** O
/// `pivot_shift` conta um quad centrado; sem normalizar, o `Sprite.size` novo levava cada quadro (novo − velho) / 2.
#[test]
fn a_folha_0_sem_centered_fica_no_lugar() {
    let px = folha();
    let ppm = 100.0;
    for ch in [
        trim(&px, 16, &CORTE).unwrap(),
        padding(&px, 16, &CORTE, PaddingSpec::uniform(2)).unwrap(),
    ] {
        for (centered, offset) in [
            (false, [0.0, 0.0]),
            (true, [3.0, -2.0]),
            (false, [3.0, -2.0]),
        ] {
            let mut w = World::new();
            let mut s = ph2d_render::Sprite::atlas(0, [0.16, 0.08], [1.0; 4]);
            (s.centered, s.offset, s.anchor) = (centered, offset, [0.03, -0.02]);
            let e = w.spawn((s, Transform::default())).id();
            // A folha é 2 × 1: o `Sprite.size` é UMA célula.
            w.get_mut::<ph2d_render::Sprite>(e).unwrap().size = [0.08, 0.08];
            let antes = no_lugar(
                &px,
                16,
                &CORTE,
                w.get::<ph2d_render::Sprite>(e).unwrap().resolve_anchor(ppm),
                [false, false],
            );
            // O funil de gravação: o tamanho novo, depois o Pivot anda.
            w.get_mut::<ph2d_render::Sprite>(e).unwrap().size = commit_size(&CORTE, ch, 0);
            keep_drawing(
                &mut w,
                e,
                0,
                &CORTE,
                pivot_shift(&CORTE, ch, [false, false]),
            );
            let (novo, nw, _) = recompose(&px, 16, &CORTE, ch);
            let novo_corte = Cut {
                cell: ch.cell,
                ..CORTE
            };
            let a = w.get::<ph2d_render::Sprite>(e).unwrap().resolve_anchor(ppm);
            let depois = no_lugar(&novo, nw, &novo_corte, a, [false, false]);
            assert!(
                perto(&antes, &depois),
                "{ch:?} centered={centered} offset={offset:?}: os quadros andaram"
            );
        }
    }
}
