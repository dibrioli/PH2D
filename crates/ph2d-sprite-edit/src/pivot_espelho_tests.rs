//! ⭐ A auditoria do Mirror (smoke do dono, 06/11), no mundo real, com a conta do render: nove pontos × espelho × Keep in
//! place × Flip H. O oráculo são as duas engines corridas em `docs/GameEngineFuncional/ferramentas/espelhar_oraculo.jsonl`.

use super::*;
use ph2d_ecs::ChildOf;

const PPM: f32 = 100.0;
const OK: f32 = 1.0e-3;

/// A moeda do dono: a Platform High (3 × 0,5 m em (8,5; 2)) com a Coin 4 presa em (0; 0,8).
fn cena(flip_x: bool) -> (World, Entity, Entity) {
    let mut w = World::new();
    let mut s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [3.0, 0.5], [1.0; 4]);
    s.flip_x = flip_x;
    let t = Transform {
        translation: Vec2::new(8.5, 2.0),
        ..Default::default()
    };
    let e = w.spawn((s, t, Collider::default())).id();
    let local = Transform {
        translation: Vec2::new(0.0, 0.8),
        ..Default::default()
    };
    let moeda = w.spawn((local, ChildOf(e))).id();
    (w, e, moeda)
}

fn pivo(w: &mut World, e: Entity, ponto: [u8; 2]) {
    let (db, atlas) = (AssetDb::default(), BTreeMap::new());
    set_pivot(w, e, &db, &atlas, PPM, SpritePivot::Ponto(ponto));
}

/// O centro do desenho, a colisão, a moeda e onde cai o `u = 0` da textura, como o extract e a física os compõem.
fn mede(w: &World, e: Entity, moeda: Entity) -> [Vec2; 4] {
    let t = *w.get::<Transform>(e).unwrap();
    let s = w.get::<Sprite>(e).unwrap();
    let a = s.resolve_anchor(PPM);
    let c = w.get::<Collider>(e).unwrap().offset;
    let lado = if s.flip_x { 1.5 } else { -1.5 };
    let filho = *w.get::<Transform>(moeda).unwrap();
    [
        no_pai(t, a),
        no_pai(t, c),
        Transform::compose(t, filho).translation,
        no_pai(t, [a[0] + lado, a[1]]),
    ]
}

/// O Mirror sem Keep in place: o `giro_click` troca o sinal da escala e nada mais.
fn mirror(w: &mut World, e: Entity, eixos: [bool; 2]) {
    let mut t = w.get_mut::<Transform>(e).unwrap();
    if eixos[0] {
        t.scale.x = -t.scale.x;
    }
    if eixos[1] {
        t.scale.y = -t.scale.y;
    }
}

fn reflete(p: Vec2, em_torno: Vec2, eixos: [bool; 2]) -> Vec2 {
    Vec2::new(
        if eixos[0] {
            2.0 * em_torno.x - p.x
        } else {
            p.x
        },
        if eixos[1] {
            2.0 * em_torno.y - p.y
        } else {
            p.y
        },
    )
}

/// As 144 células. Sem Keep, tudo espelha à volta do pivô. Com Keep, o desenho fica, e a colisão, a moeda e a imagem
/// espelham à volta do centro do desenho. Nos dois, a imagem inverte: `u = 0` passa para o outro lado.
/// *Mutação: `espelha_no_lugar` sem compensar a posição ⇒ as células com Keep e pivô fora do centro sangram.*
#[test]
fn a_tabela_do_mirror() {
    let mut celulas = 0;
    let pontos = (0..3_u8).flat_map(|l| (0..3_u8).map(move |c| [c, l]));
    for ponto in pontos {
        for eixos in [[false, false], [true, false], [false, true], [true, true]] {
            for keep in [false, true] {
                for flip in [false, true] {
                    let (mut w, e, moeda) = cena(flip);
                    pivo(&mut w, e, ponto);
                    let antes = mede(&w, e, moeda);
                    let origem = w.get::<Transform>(e).unwrap().translation;
                    assert!((antes[0] - Vec2::new(8.5, 2.0)).length() < OK);
                    assert!((antes[1] - Vec2::new(8.5, 2.0)).length() < OK);
                    assert!((antes[2] - Vec2::new(8.5, 2.8)).length() < OK);
                    if keep {
                        for (i, ligado) in eixos.into_iter().enumerate() {
                            if ligado {
                                espelha_no_lugar(&mut w, e, PPM, i);
                            }
                        }
                    } else {
                        mirror(&mut w, e, eixos);
                    }
                    let depois = mede(&w, e, moeda);
                    let centro = if keep { antes[0] } else { origem };
                    for (a, d) in antes.into_iter().zip(depois) {
                        let quer = reflete(a, centro, eixos);
                        assert!(
                            (quer - d).length() < OK,
                            "{ponto:?} {eixos:?} keep {keep} flip {flip}: {d:?}, queria {quer:?}"
                        );
                    }
                    let (db, atlas) = (AssetDb::default(), BTreeMap::new());
                    let casa = pivot_de(&w, e, &db, &atlas, PPM).unwrap().0;
                    assert_eq!(casa, Some(ponto), "a grade bate com o pivô");
                    celulas += 1;
                }
            }
        }
    }
    assert_eq!(celulas, 144);
}

/// Um número do oráculo: `"nome":[x,y]` ou `"nome":x`.
fn campo(linha: &str, nome: &str) -> Vec<f32> {
    let de = linha.find(&format!("\"{nome}\":")).expect(nome) + nome.len() + 3;
    let resto = linha[de..].trim_start_matches('[');
    let fim = resto.find(|c| c == ']' || c == '}').unwrap();
    let fim = if linha[de..].starts_with('[') {
        fim
    } else {
        resto.find([',', '}']).unwrap()
    };
    resto[..fim]
        .split(',')
        .map(|x| x.parse().expect(nome))
        .collect()
}

fn texto<'a>(linha: &'a str, nome: &str) -> &'a str {
    let de = linha.find(&format!("\"{nome}\":\"")).expect(nome) + nome.len() + 4;
    let fim = linha[de..].find('"').unwrap();
    &linha[de..de + fim]
}

/// As linhas das duas engines, com a mesma imagem de 2 × 1 m, a mesma colisão e o mesmo filho. Elas só discordam no Flip
/// com o pivô de lado: uma vira no sítio, a outra à volta do pivô, e a casa segue a primeira (decisão 14 da ficha).
#[test]
fn o_mirror_e_o_flip_dao_o_mesmo_que_o_oraculo() {
    let oraculo =
        include_str!("../../../docs/GameEngineFuncional/ferramentas/espelhar_oraculo.jsonl");
    let mut linhas = 0;
    for l in oraculo.lines().filter(|l| l.starts_with('{')) {
        if texto(l, "motor") == "unity"
            && texto(l, "modo").contains("flip")
            && texto(l, "pivo") != "centro"
        {
            continue;
        }
        let (modo, ancora) = (
            texto(l, "modo"),
            match texto(l, "pivo") {
                "esquerda" => [1.0, 0.0],
                "pe" => [0.0, 0.5],
                _ => [0.0, 0.0],
            },
        );
        let mut s = Sprite::atlas(ph2d_render::WHITE_TILE_KEY, [2.0, 1.0], [1.0; 4]);
        s.anchor = ancora;
        s.flip_x = modo == "flip_x" || modo == "flip_e_escala_x";
        s.flip_y = modo == "flip_y";
        let t = Transform {
            rotation: campo(l, "rot")[0].to_radians(),
            scale: Vec2::new(
                if modo.contains("escala_x") { -1.0 } else { 1.0 },
                if modo == "escala_y" { -1.0 } else { 1.0 },
            ),
            ..Default::default()
        };
        // `u` da cor no quad: vermelho na metade esquerda da imagem, verde na direita.
        let cor = |u: f32| {
            let u = if s.flip_x { 1.0 - u } else { u };
            no_pai(t, [ancora[0] + (u - 0.5) * 2.0, ancora[1]])
        };
        let medido = [
            cor(0.25),
            cor(0.75),
            no_pai(t, [0.5, 0.0]),
            no_pai(t, [3.0, 0.0]),
        ];
        for (m, nome) in medido
            .into_iter()
            .zip(["verm", "verde", "colisao", "filho"])
        {
            let o = campo(l, nome);
            assert!(
                (m.x - o[0]).abs() < 2.0e-3 && (m.y - o[1]).abs() < 2.0e-3,
                "{nome} {m:?}: {l}"
            );
        }
        linhas += 1;
    }
    assert_eq!(
        linhas, 60,
        "72 menos as 12 do Flip com o pivô de lado na segunda engine"
    );
}

/// Pivot depois de Mirror não move nada, nos dois espelhos e na ordem que o dono fez.
#[test]
fn o_pivo_depois_do_mirror_nao_move_nada() {
    for eixos in [[true, false], [false, true], [true, true]] {
        for ponto in [[0, 0], [1, 0], [2, 1], [1, 2]] {
            let (mut w, e, moeda) = cena(false);
            pivo(&mut w, e, [2, 2]);
            mirror(&mut w, e, eixos);
            let antes = mede(&w, e, moeda);
            pivo(&mut w, e, ponto);
            let depois = mede(&w, e, moeda);
            for (a, d) in antes.into_iter().zip(depois) {
                assert!((a - d).length() < OK, "{eixos:?} {ponto:?}: {a:?} → {d:?}");
            }
        }
    }
}

/// Só o Mirror V (com ou sem Keep) põe a moeda em y negado. Mirror H, Flip e os pontos do Pivot deixam a altura dela.
#[test]
fn so_o_mirror_v_leva_a_moeda_para_baixo() {
    let pontos = (0..3_u8).flat_map(|l| (0..3_u8).map(move |c| [c, l]));
    for ponto in pontos {
        for (i, keep) in [(0, false), (0, true), (1, false), (1, true)] {
            let (mut w, e, moeda) = cena(false);
            pivo(&mut w, e, ponto);
            if keep {
                espelha_no_lugar(&mut w, e, PPM, i);
            } else {
                mirror(&mut w, e, [i == 0, i == 1]);
            }
            let y = mede(&w, e, moeda)[2].y;
            let linha = ph2d_core::Vec2::new(0.0, 2.0 - f32::from(ponto[1]) * 0.25 + 0.25);
            let quer = match (i, keep) {
                (0, _) => 2.8,
                (_, true) => 1.2,
                _ => 2.0 * linha.y - 2.8,
            };
            assert!((y - quer).abs() < OK, "{ponto:?} eixo {i} keep {keep}: {y}");
        }
    }
}
