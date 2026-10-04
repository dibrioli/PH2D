//! Os gates da camada que o glow bright-passa (bug do Enio, 2026-08-20).

use super::*;
use crate::motion_object_bake::ObjectBake;
use crate::motion_shape_bake::ShapeBake;

/// Uma instância vetorial viva daquela geometria, na posição `x`.
fn vi(geometry_id: u32, x: f32) -> VectorInstance {
    VectorInstance {
        geometry_id,
        texture_id: 0,
        atlas_uv: [0.0, 0.0, 1.0, 1.0],
        premultiplied: 0.0,
        world_pos: [x, 0.0],
        size: [1.0, 1.0],
        basis: [1.0, 0.0, 0.0, 1.0],
        tint: [1.0, 1.0, 1.0, 1.0],
        anchor: [0.0, 0.0],
        sampling: 0,
        blend_linha: 0,
        mistura: Default::default(),
    }
}

/// Um sprite comum, para o gate distinguir as duas metades.
///
/// ⚠️ Construído pela MESMA porta que o produto usa ([`vector_instance_as_tile`])
/// sobre uma geometria de tile `0`: o `RenderInstance` tem dezenas de campos sem
/// `Default`, e uma cópia à mão deles envelheceria no primeiro campo novo.
fn sprite(x: f32) -> RenderInstance {
    vector_instance_as_tile(&vi(0, x), 0)
}

/// Um bake com tile assado para as geometrias pedidas.
fn bake_with(gids: &[u32]) -> ObjectBake {
    let mut b = ObjectBake::default();
    for (i, gid) in gids.iter().enumerate() {
        #[expect(clippy::cast_possible_truncation, reason = "uma fixture pequena")]
        let id = i as u32;
        b.seed_for_test(u64::from(id) + 1, *gid, 900 + id, [1.0, 1.0]);
    }
    b
}

/// **A FORMA VIVA ENTRA NA CAMADA DO GLOW** — o bug, curado.
///
/// ⚠️ O oráculo é a CONTAGEM da lista derivada contra a de sprites: antes desta
/// wave a camada ERA `pump.instances`, então um gate que só olhasse "a lista não
/// está vazia" ficaria verde sobre o defeito.
#[test]
fn live_vector_geometry_reaches_the_glow_layer() {
    let bake = bake_with(&[5]);
    let sprites = vec![sprite(0.0)];
    let vectors = vec![vi(5, 1.0), vi(5, 2.0)];
    let layer = layer_instances(&sprites, &vectors, &bake, &ShapeBake::default());
    assert_eq!(layer.len(), 3, "um sprite + duas formas: {layer:?}");
    assert_eq!(layer[0].texture_id, 0, "o sprite vem primeiro");
    assert!(
        layer[1..].iter().all(|i| i.texture_id == 900),
        "as formas entram pelo tile assado daquela geometria"
    );
}

/// **NÃO HÁ MAIS DEGRAU DE CONTAGEM** — a metade cruel do bug.
///
/// ⚠️ A partição de LOD move para `instances` acima de `LOD_COUNT = 16_000`, e era
/// isso que fazia *a mesma forma não brilhar com 16 000 cópias e brilhar com
/// 16 001*. Esta camada não consulta contagem nenhuma: **duas** cópias entram
/// exactamente como vinte mil entrariam.
#[test]
fn the_layer_asks_no_question_about_how_many_copies_there_are() {
    let bake = bake_with(&[5]);
    for n in [1usize, 2, 17, 100] {
        #[expect(clippy::cast_precision_loss, reason = "uma fixture pequena")]
        let vectors: Vec<VectorInstance> = (0..n).map(|i| vi(5, i as f32)).collect();
        let layer = layer_instances(&[], &vectors, &bake, &ShapeBake::default());
        assert_eq!(layer.len(), n, "{n} cópias têm de dar {n} quads");
    }
}

/// **O `tint` HDR SOBREVIVE** — é a razão de a rota ser o tile e não uma ponte por
/// Vello (que clamparia a 8 bits).
///
/// ⚠️ Um `tint` de `40` tem de chegar ao bright-pass como `40`: é ele que decide se
/// a peça passa do `threshold`. Um tile que já viesse tingido multiplicaria a cor
/// duas vezes — o tile é BRANCO de propósito.
#[test]
fn an_hdr_tint_survives_the_trip_through_the_tile() {
    let bake = bake_with(&[5]);
    let hot = VectorInstance {
        tint: [40.0, 32.0, 24.0, 1.0],
        ..vi(5, 0.0)
    };
    let layer = layer_instances(&[], &[hot], &bake, &ShapeBake::default());
    assert_eq!(layer.len(), 1);
    assert_eq!(
        layer[0].tint,
        [40.0, 32.0, 24.0, 1.0],
        "o tile é branco e o tint viaja verbatim"
    );
}

/// **UMA GEOMETRIA SEM TILE É PULADA, e a sonda CONTA-A.**
///
/// ⚠️ Sem a sonda, «pular» e «não existir» leriam igual, e o buraco que sobra (o
/// `source.shape`, ainda não assado) viveria só na prosa. Quando alguém o assar
/// este gate fica vermelho e obriga a reconferir a nota do módulo — que é o que
/// impede a nota de envelhecer.
#[test]
fn a_geometry_with_no_baked_tile_is_skipped_and_counted() {
    let bake = bake_with(&[5]);
    let vectors = vec![vi(5, 0.0), vi(77, 1.0), vi(77, 2.0)];
    let layer = layer_instances(&[], &vectors, &bake, &ShapeBake::default());
    assert_eq!(layer.len(), 1, "só a geometria com tile contribui");
    assert_eq!(
        unreachable_geometries(&vectors, &bake, &ShapeBake::default()),
        2,
        "e as outras duas são contadas, não engolidas"
    );
}

/// **A FORMA PARAMÉTRICA TAMBÉM CHEGA** — pelo assador irmão, e com a colocação
/// dele (o tamanho vem do tile, não da instância).
///
/// ⚠️ É o caso EXACTO do report do Enio (*"Glow não funciona com shape"*): uma
/// geometria que o `object_bake` não conhece, porque ela não está na cena vetorial.
#[test]
fn a_parametric_shape_reaches_the_layer_through_its_own_bake() {
    let bake = ObjectBake::default(); // nenhum objeto: a rota é a do shape
    let mut shapes = ShapeBake::default();
    shapes.seed_for_test(
        9,
        crate::motion_shape_bake::ShapeTile {
            texture_id: 77,
            world_size: [2.0, 3.0],
            local_center: [0.0, 0.0],
        },
    );
    let layer = layer_instances(&[], &[vi(9, 0.0)], &bake, &shapes);
    assert_eq!(layer.len(), 1, "a forma tem de entrar");
    assert_eq!(layer[0].texture_id, 77);
    assert_eq!(
        layer[0].size,
        [2.0, 3.0],
        "e com o tamanho do TILE — a instância do shape emite `size` unitário"
    );
    assert_eq!(
        unreachable_geometries(&[vi(9, 0.0)], &bake, &shapes),
        0,
        "e deixa de ser contada como inalcançável"
    );
}

/// **O OBJETO GANHA do shape quando a geometria está nos DOIS** — as duas rotas
/// partilham o `shape_store`, e a conversão do objeto é a que o publicador
/// dimensionou.
#[test]
fn an_object_tile_wins_over_a_shape_tile_for_the_same_geometry() {
    let bake = bake_with(&[5]);
    let mut shapes = ShapeBake::default();
    shapes.seed_for_test(
        5,
        crate::motion_shape_bake::ShapeTile {
            texture_id: 77,
            world_size: [9.0, 9.0],
            local_center: [0.0, 0.0],
        },
    );
    let layer = layer_instances(&[], &[vi(5, 0.0)], &bake, &shapes);
    assert_eq!(layer[0].texture_id, 900, "o tile do objeto é o que vale");
}

/// **A CAMADA NÃO MOVE NADA** — as duas listas de origem saem intactas.
///
/// ⚠️ É o que separa esta função da partição de LOD, e o que mantém o caminho
/// crispo do quadro visível byte-a-byte como estava. Um `apply_*` aqui mudaria o
/// que o artista VÊ, e o pedido era só fazer aquilo brilhar.
#[test]
fn deriving_the_layer_leaves_the_visible_frame_untouched() {
    let bake = bake_with(&[5]);
    let sprites = vec![sprite(0.0)];
    let vectors = vec![vi(5, 1.0)];
    let (before_s, before_v) = (sprites.clone(), vectors.clone());
    let _ = layer_instances(&sprites, &vectors, &bake, &ShapeBake::default());
    assert_eq!(sprites.len(), before_s.len());
    assert_eq!(vectors.len(), before_v.len());
    assert_eq!(vectors[0].geometry_id, before_v[0].geometry_id);
}

/// **SEM NADA, A CAMADA É VAZIA** — o guarda do passe continua a saber pular.
#[test]
fn an_empty_motion_layer_stays_empty() {
    assert!(layer_instances(&[], &[], &ObjectBake::default(), &ShapeBake::default()).is_empty());
}

/// ⭐⭐ doc 121 §9.14 (c) — **NUM QUADRO DO DISPOSITIVO O HALO NÃO LÊ AS LISTAS DA CPU.** A bomba não
/// correu (`gpu_live`), e as listas dela são de um quadro VELHO: o halo brilhava onde as sprites
/// estiveram. Com o CONTROLO de que, no quadro da CPU, as mesmas listas entram.
#[test]
fn a_device_frame_halo_never_reads_the_stale_cpu_lists() {
    let mut m = MotionState::new();
    m.pump.instances.push(sprite(3.0));
    m.pump.vector_instances.push(vi(9, 0.0));
    m.shape_bake.seed_for_test(
        9,
        crate::motion_shape_bake::ShapeTile {
            texture_id: 77,
            world_size: [1.0, 1.0],
            local_center: [0.0, 0.0],
        },
    );
    let cpu = halo_do_quadro(&m, false);
    assert_eq!(
        cpu.cpu.len(),
        2,
        "CONTROLO: no quadro da CPU a sprite e o tile entram"
    );
    assert!(cpu.placa.is_none());
    m.gpu_live = true;
    let placa = halo_do_quadro(&m, true);
    assert!(
        placa.cpu.is_empty(),
        "num quadro do dispositivo as listas da CPU são de um quadro velho"
    );
    assert!(placa.formas_da_placa && !placa.vazio());
}

/// ⭐ doc 121 §9.14 (c) — **as formas que o passe de formas desenhou não pedem o tile**: o halo
/// redesenha a camada dele (a silhueta exacta, o mesmo HDR). Sem a camada, o tile de sempre.
#[test]
fn shapes_drawn_by_the_shape_pass_leave_the_tile_path() {
    let mut m = MotionState::new();
    m.pump.instances.push(sprite(3.0));
    m.pump.vector_instances.push(vi(9, 0.0));
    m.shape_bake.seed_for_test(
        9,
        crate::motion_shape_bake::ShapeTile {
            texture_id: 77,
            world_size: [1.0, 1.0],
            local_center: [0.0, 0.0],
        },
    );
    let com_a_placa = halo_do_quadro(&m, true);
    assert_eq!(
        com_a_placa.cpu.len(),
        1,
        "só a sprite: a forma vem da camada"
    );
    assert!(com_a_placa.cpu.iter().all(|r| r.texture_id != 77));
    let sem = halo_do_quadro(&m, false);
    assert!(
        sem.cpu.iter().any(|r| r.texture_id == 77),
        "CONTROLO: sem a camada, o tile"
    );
}
