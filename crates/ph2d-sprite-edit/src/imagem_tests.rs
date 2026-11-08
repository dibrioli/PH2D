use super::*;
use ph2d_asset::AssetDb;
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

/// ⭐ A lei do dono: a imagem volta para TODO objecto que a usa, e só para esses, sem mudar de tamanho.
#[test]
fn a_edicao_vale_para_todo_objecto_que_usa_a_mesma_imagem() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, c) = mundo(&db);
    let atlas = BTreeMap::new();
    marca_edicao(&w, a, &atlas);
    let novo = db.insert_image_rgba8(1, 1, px(99));
    w.entity_mut(a).insert(SpritePixels(novo));
    let t = segue_a_pintura(&mut w, &atlas, a, 77).expect("segue");
    EDICAO.with(|c| c.set(None));
    assert_eq!((t.quem, t.antigo, t.novo), (vec![b], antigo, novo));
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

#[test]
fn sem_o_botao_a_pintura_fica_so_no_objecto() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, _) = mundo(&db);
    EDICAO.with(|c| c.set(None));
    let novo = db.insert_image_rgba8(1, 1, px(99));
    w.entity_mut(a).insert(SpritePixels(novo));
    assert_eq!(segue_a_pintura(&mut w, &BTreeMap::new(), a, 77), None);
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
    assert_eq!(
        segue_a_pintura(&mut w, &atlas, a, 77).expect("segue").quem,
        [b]
    );
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, p1);
    let p2 = db.insert_image_rgba8(1, 1, px(60));
    w.entity_mut(a).insert(SpritePixels(p2));
    assert_eq!(
        segue_a_pintura(&mut w, &atlas, a, 78).expect("segue").quem,
        [b]
    );
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, p2);
    assert_ne!(w.get::<SpritePixels>(c).unwrap().0, p2);
    EDICAO.with(|c| c.set(None));
}

/// ⭐ **Edit in Painter na folha extra VISÍVEL** (plano GEF 26 F13): marca a imagem DELA, e a pintura chega também a
/// quem a mostra como folha `0` (com a textura reta das folhas). *Mutações: `marca_edicao` pela folha `0` ⇒ marca
/// `antigo` e `b` não segue; `premultiplied` da folha 0 ⇒ `b` fica pré-multiplicado.*
#[test]
fn com_a_folha_extra_visivel_o_painter_marca_e_devolve_a_imagem_dela() {
    let db = AssetDb::default();
    let (mut w, antigo, a, b, _) = mundo(&db);
    let run = db.insert_image_rgba8(1, 1, px(30));
    let folha = |asset| ph2d_ecs::ExtraSheet {
        asset,
        size: [1.0, 1.0],
        anchor: [0.0, 0.0],
        hframes: 1,
        vframes: 1,
    };
    w.entity_mut(a).insert((
        ph2d_ecs::SpriteSheets([folha(run)].into_iter().collect()),
        ph2d_ecs::ShownSheet(1),
    ));
    w.entity_mut(b).insert(SpritePixels(run));
    // A folha 0 de `a` já foi pintada (pré-multiplicada): não é ela que diz a bandeira da folha extra.
    w.get_mut::<Sprite>(a).unwrap().premultiplied = true;
    let atlas = BTreeMap::new();
    marca_edicao(&w, a, &atlas);
    assert_eq!(EDICAO.with(Cell::get), Some((a, run)));
    let pintada = db.insert_image_rgba8(1, 1, px(90));
    w.get_mut::<ph2d_ecs::SpriteSheets>(a).unwrap().0[0].asset = pintada;
    let t = segue_a_pintura(&mut w, &atlas, a, 99).expect("segue");
    assert_eq!((t.antigo, t.novo, t.quem), (run, pintada, vec![b]));
    assert_eq!(w.get::<SpritePixels>(b).unwrap().0, pintada);
    assert!(!w.get::<Sprite>(b).unwrap().premultiplied, "a textura da folha é reta");
    assert_eq!(w.get::<SpritePixels>(a).unwrap().0, antigo, "a folha 0 de a fica");
    EDICAO.with(|c| c.set(None));
}

/// ⭐ **Editar não muda o nome da imagem** (dono 05/11: a «Platform» editada não vira «Platform Low»).
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

/// ⭐ GEF 26 F3/F13: a imagem pintada volta também a toda FOLHA EXTRA que a mostrava (de outro objecto e do próprio),
/// e só a essas. Mutação que sangra: `segue_a_pintura` sem `aponta_folhas` (o Painter só reescreve a folha `0`).
#[test]
fn o_painter_devolve_a_imagem_a_folha_certa() {
    let db = AssetDb::default();
    let (mut w, antigo, a, _, c) = mundo(&db);
    let outra = w.get::<SpritePixels>(c).unwrap().0;
    let folha = |asset| ph2d_ecs::ExtraSheet {
        asset,
        size: [1.0; 2],
        anchor: [0.0; 2],
        hframes: 2,
        vframes: 1,
    };
    let folhas = ph2d_ecs::SpriteSheets([folha(outra), folha(antigo)].into_iter().collect());
    w.entity_mut(c).insert(folhas.clone());
    w.entity_mut(a).insert(folhas);
    let atlas = BTreeMap::new();
    marca_edicao(&w, a, &atlas);
    let novo = db.insert_image_rgba8(1, 1, px(99));
    w.entity_mut(a).insert(SpritePixels(novo));
    segue_a_pintura(&mut w, &atlas, a, 77).expect("segue");
    EDICAO.with(|c| c.set(None));
    for e in [a, c] {
        let s = w.get::<ph2d_ecs::SpriteSheets>(e).unwrap();
        assert_eq!(
            (s.0[0].asset, s.0[1].asset),
            (outra, novo),
            "a folha 2 de {e:?} segue a pintura, a 1 não"
        );
    }
}
