use super::*;
use ph2d_render::WHITE_TILE_KEY;

fn px(cor: u8) -> Vec<u8> {
    vec![cor, cor, cor, 255]
}

fn mundo(db: &AssetDb) -> (World, AssetId, Entity, Entity, Entity) {
    let antigo = db.insert_image_rgba8(1, 1, px(10));
    let outra = db.insert_image_rgba8(1, 1, px(20));
    let mut w = World::new();
    let sprite = |tex| {
        let mut s = Sprite::atlas(WHITE_TILE_KEY, [3.0, 3.0], [1.0; 4]);
        s.source = SpriteSource::Individual { texture_id: tex };
        s
    };
    let a = w.spawn((sprite(1), SpritePixels(antigo))).id();
    let b = w.spawn((sprite(1), SpritePixels(antigo))).id();
    let c = w.spawn((sprite(2), SpritePixels(outra))).id();
    (w, antigo, a, b, c)
}

/// ⭐ A lei do dono: a imagem volta para TODO objecto que a usa, e só para esses.
#[test]
fn a_edicao_vale_para_todo_objecto_que_usa_a_mesma_imagem() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, c) = mundo(&db);
    let atlas = BTreeMap::new();
    let mut pedido = None;
    let n = troca_os_pixeis(
        &mut w,
        &mut |_, _, p| {
            pedido = Some(p.to_vec());
            Ok(77)
        },
        &db,
        &atlas,
        a,
        (1, 1, px(99)),
    )
    .expect("troca");
    assert_eq!(n, 2);
    assert_eq!(pedido, Some(px(99)));
    let novo = w.get::<SpritePixels>(a).unwrap().0;
    assert_ne!(novo, antigo);
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, novo);
    assert_ne!(
        w.get::<SpritePixels>(c).unwrap().0,
        novo,
        "a outra imagem não muda"
    );
    assert_eq!(
        w.get::<Sprite>(b).unwrap().source,
        SpriteSource::Individual { texture_id: 77 }
    );
    assert_eq!(
        w.get::<Sprite>(b).unwrap().size,
        [3.0, 3.0],
        "o tamanho fica"
    );
}

/// Sair sem pintar nada não troca coisa nenhuma: o id é o do conteúdo.
#[test]
fn sair_sem_mudar_nada_nao_mexe_em_ninguem() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, _) = mundo(&db);
    let n = troca_os_pixeis(
        &mut w,
        &mut |_, _, _| Ok(77),
        &db,
        &BTreeMap::new(),
        a,
        (1, 1, px(10)),
    )
    .expect("troca");
    assert_eq!(n, 0);
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, antigo);
}

#[test]
fn sem_o_botao_a_pintura_fica_so_no_objecto() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, _) = mundo(&db);
    EDICAO.with(|c| c.set(None));
    let novo = db.insert_image_rgba8(1, 1, px(99));
    w.entity_mut(a).insert(SpritePixels(novo));
    assert_eq!(segue_a_pintura(&mut w, &BTreeMap::new(), a, 77), 0);
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, antigo);
}

#[test]
fn com_o_botao_a_pintura_chega_aos_outros_e_a_seguinte_tambem() {
    let db = AssetDb::default();
    let (mut w, _, a, b, c) = mundo(&db);
    let atlas = BTreeMap::new();
    marca_edicao(&w, a, &atlas);
    let p1 = db.insert_image_rgba8(1, 1, px(50));
    w.entity_mut(a).insert(SpritePixels(p1));
    assert_eq!(segue_a_pintura(&mut w, &atlas, a, 77), 1);
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, p1);
    let p2 = db.insert_image_rgba8(1, 1, px(60));
    w.entity_mut(a).insert(SpritePixels(p2));
    assert_eq!(segue_a_pintura(&mut w, &atlas, a, 78), 1);
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, p2);
    assert_ne!(w.get::<SpritePixels>(c).unwrap().0, p2);
    EDICAO.with(|c| c.set(None));
}

#[test]
fn sair_sem_pintar_nao_muda_ninguem_mesmo_com_a_cor_do_transparente_zerada() {
    let db = AssetDb::default();
    let antigo = db.insert_image_rgba8(2, 1, vec![9, 9, 9, 255, 7, 7, 7, 0]);
    let mut w = World::new();
    let mut s = Sprite::atlas(WHITE_TILE_KEY, [3.0, 3.0], [1.0; 4]);
    s.source = SpriteSource::Individual { texture_id: 1 };
    let a = w.spawn((s, SpritePixels(antigo))).id();
    let mut subiu = false;
    let n = troca_os_pixeis(
        &mut w,
        &mut |_, _, _| {
            subiu = true;
            Ok(2)
        },
        &db,
        &BTreeMap::new(),
        a,
        (2, 1, vec![9, 9, 9, 255, 0, 0, 0, 0]),
    );
    assert_eq!(n, Ok(0));
    assert!(!subiu, "nada sobe à placa");
    assert_eq!(w.get::<SpritePixels>(a).unwrap().0, antigo);
}

/// ⭐ **Editar não muda o nome da imagem** (dono 05/11: a «Platform» editada no Pixel não vira «Platform Low»).
#[test]
fn a_imagem_editada_herda_o_nome_da_de_onde_veio() {
    let id = |n| AssetId::from_digest([n; 32]);
    let (raiz, e1, e2) = (id(1), id(2), id(3));
    let origens: BTreeMap<_, _> = [(e1, raiz), (e2, e1)].into_iter().collect();
    let corre = |ids: &[(AssetId, &str)], live: &[AssetId]| {
        let mut m: BTreeMap<AssetId, Option<String>> = ids
            .iter()
            .map(|(i, n)| (*i, Some((*n).to_owned())))
            .collect();
        let vivos: BTreeSet<AssetId> = live.iter().copied().collect();
        herda(&mut m, &vivos, &|_: &AssetId| None, &origens);
        m
    };
    let m = corre(&[(raiz, "Platform"), (e2, "Platform Low")], &[e2]);
    assert_eq!(m[&e2].as_deref(), Some("Platform"), "duas edições depois");
    assert!(!m.contains_key(&raiz), "a edição toma o lugar: {m:?}");
    let m = corre(&[(raiz, "Platform"), (e1, "Platform Low")], &[raiz, e1]);
    assert!(m.contains_key(&raiz), "a origem ainda em uso fica: {m:?}");
    let m = corre(&[(raiz, "Platform")], &[raiz]);
    assert_eq!(m.len(), 1, "desfeito, nada muda: {m:?}");
}
