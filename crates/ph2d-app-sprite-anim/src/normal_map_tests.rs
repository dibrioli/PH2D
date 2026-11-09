use super::*;

const BRANCO: [f32; 4] = [1.0; 4];

/// A placa dos gates: o que ela tem por alvo.
#[derive(Default)]
struct Placa(BTreeMap<Alvo, Vec<u8>>);

impl PlacaDeNormais for Placa {
    fn definir(&mut self, alvo: Alvo, normal: Option<(u32, u32, &[u8])>) -> Result<(), String> {
        match normal {
            Some((_, _, px)) => self.0.insert(alvo, px.to_vec()),
            None => self.0.remove(&alvo),
        };
        Ok(())
    }
    fn tem(&self, alvo: Alvo) -> bool {
        self.0.contains_key(&alvo)
    }
}

/// Uma imagem 1 × 1 com a cor `c`.
fn imagem(db: &AssetDb, c: u8) -> AssetId {
    db.insert_image_rgba8(1, 1, vec![c, c, c, 255])
}

/// ⭐ **Dois sprites da MESMA imagem acendem pela MESMA normal** — dois com textura própria e um terceiro pela célula
/// do átlas que tem a mesma imagem; um sprite de OUTRA imagem fica sem normal. Tirar a normal tira-a de todos.
///
/// **Mutação que sangra:** `alvos` só com as células do átlas (as texturas próprias ficam escuras).
#[test]
fn dois_sprites_da_mesma_imagem_acendem_pela_mesma_normal() {
    set_normais(&[]);
    let db = AssetDb::new();
    let (cor, outra, normal) = (imagem(&db, 10), imagem(&db, 20), imagem(&db, 200));
    let atlas: BTreeMap<u32, AssetId> = [(7, cor)].into_iter().collect();
    let mut w = World::new();
    let a = w
        .spawn((Sprite::individual(3, [1.0; 2], BRANCO), SpritePixels(cor)))
        .id();
    w.spawn((Sprite::individual(4, [1.0; 2], BRANCO), SpritePixels(cor)));
    w.spawn((Sprite::atlas(7, [1.0; 2], BRANCO),));
    w.spawn((Sprite::individual(5, [1.0; 2], BRANCO), SpritePixels(outra)));

    escolher_normal(&w, &db, &atlas, a, Some(normal)).expect("mesmo tamanho");
    assert_eq!(normal_de(cor), Some(normal), "a normal é da IMAGEM");
    let mut placa = Placa::default();
    let _ = sincronizar(&mut w, &db, &atlas, &mut placa);
    let n = vec![200, 200, 200, 255];
    assert_eq!(placa.0.get(&Alvo::Textura(3)), Some(&n));
    assert_eq!(
        placa.0.get(&Alvo::Textura(4)),
        Some(&n),
        "o 2.º sprite da mesma imagem"
    );
    assert_eq!(
        placa.0.get(&Alvo::Celula(7)),
        Some(&n),
        "a célula do átlas da mesma imagem"
    );
    assert!(
        !placa.0.contains_key(&Alvo::Textura(5)),
        "outra imagem fica sem normal"
    );

    escolher_normal(&w, &db, &atlas, a, None).expect("tirar");
    let _ = sincronizar(&mut w, &db, &atlas, &mut placa);
    assert!(
        placa.0.is_empty(),
        "None tira a normal de todos: {:?}",
        placa.0.keys()
    );
}

/// Outro tamanho que a cor é recusado com a chave em inglês, e a verdade não muda.
#[test]
fn a_normal_de_outro_tamanho_e_recusada_na_escolha() {
    set_normais(&[]);
    let db = AssetDb::new();
    let cor = imagem(&db, 10);
    let grande = db.insert_image_rgba8(2, 1, vec![128; 8]);
    let mut w = World::new();
    let e = w
        .spawn((Sprite::individual(3, [1.0; 2], BRANCO), SpritePixels(cor)))
        .id();
    let err = escolher_normal(&w, &db, &BTreeMap::new(), e, Some(grande)).unwrap_err();
    assert_eq!(err, ph2d_i18n::tr("app.sprite_anim.normal_map.other_size"));
    assert_eq!(normal_de(cor), None);
}

/// A placa que recusa tudo, e conta as tentativas.
#[derive(Default)]
struct PlacaQueRecusa(usize);

impl PlacaDeNormais for PlacaQueRecusa {
    fn definir(&mut self, _: Alvo, normal: Option<(u32, u32, &[u8])>) -> Result<(), String> {
        if normal.is_none() {
            return Ok(());
        }
        self.0 += 1;
        Err("out of memory".into())
    }
    fn tem(&self, _: Alvo) -> bool {
        false
    }
}

/// ⭐ **Uma normal recusada pela placa não repete a cada quadro**: 10 quadros = UMA tentativa e UM aviso (em
/// inglês, com o erro); a verdade a mudar (escolher de novo, undo) tira a cerca e tenta outra vez, uma vez.
///
/// **Mutação que sangra:** o `sincronizar` sem a cerca (`recusadas`) — 10 tentativas e 10 avisos.
#[test]
fn uma_normal_recusada_pela_placa_nao_repete_a_cada_quadro() {
    set_normais(&[]);
    let db = AssetDb::new();
    let (cor, normal) = (imagem(&db, 10), imagem(&db, 200));
    let atlas = BTreeMap::new();
    let mut w = World::new();
    let e = w
        .spawn((Sprite::individual(3, [1.0; 2], BRANCO), SpritePixels(cor)))
        .id();
    escolher_normal(&w, &db, &atlas, e, Some(normal)).expect("mesmo tamanho");
    let mut placa = PlacaQueRecusa::default();
    let mut avisos = Vec::new();
    for _ in 0..10 {
        avisos.extend(sincronizar(&mut w, &db, &atlas, &mut placa));
    }
    assert_eq!(
        placa.0, 1,
        "a placa foi tentada {} vezes em 10 quadros",
        placa.0
    );
    assert_eq!(
        avisos,
        [ph2d_i18n::tr_with(
            "app.sprite_anim.normal_map.gpu_refused",
            &[("e", &"out of memory")]
        )]
    );
    escolher_normal(&w, &db, &atlas, e, Some(normal)).expect("de novo");
    for _ in 0..10 {
        avisos.extend(sincronizar(&mut w, &db, &atlas, &mut placa));
    }
    assert_eq!(
        (placa.0, avisos.len()),
        (2, 2),
        "escolher de novo tenta uma vez"
    );
    set_normais(&normais());
    let _ = sincronizar(&mut w, &db, &atlas, &mut placa);
    assert_eq!(placa.0, 3, "o undo (set_normais) também tira a cerca");
    set_normais(&[]);
}
