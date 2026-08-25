//! Os gates da migração v97 → v98 — o corte da `Sprite` dentro de um ficheiro de projeto.
//!
//! ⚠️ **A fixtura é construída com o espelho CONGELADO** ([`ph2d_render::SpriteV4`]), e não com o
//! tipo vivo: são estes os bytes que estão em disco. Uma fixtura feita com a `Sprite` de hoje
//! testaria a minha ideia do formato, não o formato — a lição que as fixturas escritas à mão da
//! F1 já pagaram uma vez.

use super::*;
use ph2d_ecs::StableId;
use ph2d_ecs::scene::{EntitySnapshotRow, WorldSnapshot};

/// Um `SpriteV4` neutro — os 20 campos como o v4 os gravava, sem nada autorado.
fn v4_plain() -> ph2d_render::SpriteV4 {
    ph2d_render::SpriteV4 {
        version: 4,
        source: ph2d_render::SpriteSource::Atlas { key: 7 },
        size: [2.0, 3.0],
        tint: [0.1, 0.2, 0.3, 0.4],
        anchor: [1.0, -1.0],
        premultiplied: false,
        self_tint: [1.0; 4],
        per_corner_tint: [[1.0; 4]; 4],
        tint_fill: false,
        opacity: 1.0,
        flip_x: false,
        flip_y: false,
        centered: true,
        offset: [0.0; 2],
        hframes: 1,
        vframes: 1,
        frame: 0,
        region_enabled: false,
        region_rect: [0.0; 4],
        region_filter_clip: true,
    }
}

/// Um snapshot de uma entidade que carrega este blob de `Sprite`.
fn world_with(v4: ph2d_render::SpriteV4) -> WorldSnapshot {
    let mut w = WorldSnapshot::new();
    w.entities.push(EntitySnapshotRow {
        id: StableId(1),
        components: vec![ComponentBlob {
            type_id: ph2d_ecs::scene::stable_type_id(SPRITE),
            data: postcard::to_allocvec(&v4).expect("v4 serializa"),
        }],
        parent: None,
    });
    w
}

fn blob<'a>(w: &'a WorldSnapshot, name: &str) -> Option<&'a ComponentBlob> {
    let id = ph2d_ecs::scene::stable_type_id(name);
    w.entities[0].components.iter().find(|b| b.type_id == id)
}

/// ⭐ **Um ficheiro v97 com grelha, janela e degradê AUTORADOS devolve os três componentes** — a
/// afirmação central da migração, e a única que mede o caso que importa.
///
/// (Mutação: não apendar os blobs ⇒ a sprite abre sem grelha e a folha some — RED.)
#[test]
fn an_authored_v97_sprite_splits_into_four_blobs() {
    let mut v4 = v4_plain();
    v4.hframes = 4;
    v4.vframes = 2;
    v4.frame = 5;
    v4.region_enabled = true;
    v4.region_rect = [1.0, 2.0, 8.0, 8.0];
    v4.region_filter_clip = false;
    v4.per_corner_tint = [[1.0, 0.0, 0.0, 1.0]; 4];
    let mut w = world_with(v4);

    let report = split_sprite_blobs(&mut w);
    assert_eq!(report.sprites, 1);
    assert_eq!(report.components, 3, "grelha + janela + cantos");
    assert_eq!(report.unreadable, 0);
    assert_eq!(w.entities[0].components.len(), 4);

    // A `Sprite` reescrita lê-se com o tipo VIVO e preserva o que ficou.
    let s: ph2d_render::Sprite =
        postcard::from_bytes(&blob(&w, SPRITE).expect("a sprite").data).expect("v5 le-se");
    assert_eq!(s.version, 5);
    assert_eq!(s.source, v4.source);
    assert_eq!(s.size, v4.size);
    assert_eq!(s.tint, v4.tint);
    assert_eq!(s.anchor, v4.anchor);

    let g: ph2d_ecs::SpriteGrid =
        postcard::from_bytes(&blob(&w, GRID).expect("a grelha").data).expect("grelha");
    assert_eq!((g.hframes, g.vframes, g.frame), (4, 2, 5));

    let r: ph2d_ecs::SpriteRegion =
        postcard::from_bytes(&blob(&w, REGION).expect("a janela").data).expect("janela");
    assert_eq!(r.rect, [1.0, 2.0, 8.0, 8.0]);
    assert!(
        !r.filter_clip,
        "o bool GRAVADO, nao o derivado da fonte (esta e' uma sprite de Atlas)"
    );

    let c: ph2d_ecs::SpriteCornerTint =
        postcard::from_bytes(&blob(&w, CORNER_TINT).expect("os cantos").data).expect("cantos");
    assert_eq!(c.0, [[1.0, 0.0, 0.0, 1.0]; 4]);
}

/// ⚠️ **Uma sprite NEUTRA sai com UM blob só** — nada é materializado. É o controle positivo do
/// gate acima: sem ele, uma migração que anexasse sempre os três passaria lá e encheria toda cena
/// antiga de secções que o artista nunca pediu (o oposto do ADR-0166).
#[test]
fn a_plain_v97_sprite_gains_no_components() {
    let mut w = world_with(v4_plain());
    let report = split_sprite_blobs(&mut w);
    assert_eq!(report.sprites, 1);
    assert_eq!(report.components, 0, "o neutro nao se materializa");
    assert_eq!(w.entities[0].components.len(), 1);
}

/// ⚠️ **Correr duas vezes não estraga nada.** A segunda passagem não reconhece o blob v5 como v4
/// e deixa-o intacto — o que importa é que ela **não o reescreve**, porque reescrever com um
/// palpite é pior que não tocar.
#[test]
fn running_the_split_twice_leaves_the_second_pass_inert() {
    let mut v4 = v4_plain();
    v4.hframes = 3;
    let mut w = world_with(v4);
    split_sprite_blobs(&mut w);
    let after_first = w.entities[0].components.clone();

    let second = split_sprite_blobs(&mut w);
    assert_eq!(second.sprites, 0, "a segunda passagem nao reescreve nada");
    assert_eq!(second.unreadable, 1, "…e diz que nao reconheceu o blob");
    assert_eq!(
        w.entities[0].components, after_first,
        "e os bytes ficaram exatamente como estavam"
    );
}

/// Uma entidade **sem** `Sprite` passa incólume — a travessia não pode tocar no que não é dela.
#[test]
fn a_row_without_a_sprite_is_untouched() {
    let mut w = WorldSnapshot::new();
    w.entities.push(EntitySnapshotRow {
        id: StableId(1),
        components: vec![ComponentBlob {
            type_id: ph2d_ecs::scene::stable_type_id("ph2d::ecs::Name"),
            data: vec![1, 2, 3],
        }],
        parent: None,
    });
    let before = w.entities[0].components.clone();
    let report = split_sprite_blobs(&mut w);
    assert_eq!(report, SpriteSplit::default());
    assert_eq!(w.entities[0].components, before);
}
