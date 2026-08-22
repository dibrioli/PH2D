//! Os gates da COLOCAÇÃO do tile de uma forma (bug do Enio, 2026-08-20).
//!
//! ⚠️ **O assar em si é GPU e não é testável aqui** — ele espelha linha a linha o
//! `bake_rgba` do irmão, que tem o seu próprio gate de orientação. O que ESTES
//! gates defendem é [`tile_quad`], que é pura e é onde um halo torto nasceria.

use super::*;

/// Uma instância vetorial: pose autorada, escala unitária (que é o que o
/// `source.shape` emite — a dimensão vive na geometria).
fn vi(pos: [f32; 2], basis: [f32; 4], size: [f32; 2]) -> VectorInstance {
    VectorInstance {
        geometry_id: 1,
        world_pos: pos,
        size,
        basis,
        tint: [1.0, 1.0, 1.0, 1.0],
    }
}

/// Um tile de `w × h` mundo cujo bbox está centrado em `c` (unidades locais).
fn tile(w: f32, h: f32, c: [f32; 2]) -> ShapeTile {
    ShapeTile {
        texture_id: 42,
        world_size: [w, h],
        local_center: c,
    }
}

/// **O TAMANHO VEM DO TILE, e não da instância.**
///
/// ⚠️ É a primeira das três correcções sobre a conversão dos objetos, e o sintoma
/// de a esquecer é o mais grosso: o `source.shape` emite `size` = **unidade**
/// (a dimensão está na geometria), então um quad que copiasse `vi.size` mediria
/// `1 × 1` e o halo nasceria do tamanho errado — quase sempre minúsculo.
#[test]
fn the_quad_takes_its_size_from_the_baked_tile() {
    let q = tile_quad(
        &vi([0.0, 0.0], [1.0, 0.0, 0.0, 1.0], [1.0, 1.0]),
        tile(3.0, 2.0, [0.0, 0.0]),
    );
    assert_eq!(q.size, [3.0, 2.0]);
    assert_eq!(q.texture_id, 42);
}

/// **E ELE MULTIPLICA A ESCALA DA INSTÂNCIA** — senão um `motion.scale` a jusante
/// não alcança o halo, e ele descolaria da forma ao animar o tamanho.
#[test]
fn a_downstream_scale_still_reaches_the_halo() {
    let q = tile_quad(
        &vi([0.0, 0.0], [1.0, 0.0, 0.0, 1.0], [2.0, 0.5]),
        tile(3.0, 2.0, [0.0, 0.0]),
    );
    assert_eq!(q.size, [6.0, 1.0]);
}

/// **UMA FORMA CENTRADA NÃO SE DESLOCA** — o caso comum, e o controle.
#[test]
fn a_centred_shape_keeps_the_instances_position() {
    let q = tile_quad(
        &vi([5.0, -3.0], [1.0, 0.0, 0.0, 1.0], [1.0, 1.0]),
        tile(2.0, 2.0, [0.0, 0.0]),
    );
    assert_eq!(q.world_pos, [5.0, -3.0]);
}

/// **UMA FORMA NÃO-CENTRADA ANDA PELO SEU BBOX** — o sintoma que isto impede é o
/// halo AO LADO da forma, que parece um bug de desenho.
#[test]
fn an_off_centre_shape_moves_the_quad_by_its_bbox() {
    let q = tile_quad(
        &vi([10.0, 0.0], [1.0, 0.0, 0.0, 1.0], [1.0, 1.0]),
        tile(2.0, 2.0, [0.75, -0.25]),
    );
    assert_eq!(q.world_pos, [10.75, -0.25]);
}

/// **E O DESLOCAMENTO PASSA PELA BASE** — a terceira correcção, e a única que um
/// teste sem rotação não pega.
///
/// ⚠️ O oráculo é escolhido para separar as duas contas: com a base a `90°` um
/// bbox deslocado em `+x` tem de mover o quad em `+y`. Somar o offset **antes** da
/// base (o erro fácil) o moveria em `+x`, e a forma rodada teria o halo do lado
/// errado — visível só quando alguém liga um `motion.rotate`.
#[test]
fn the_bbox_offset_travels_through_the_instance_basis() {
    // basis de 90°: (x, y) → (−y, x). Coeficientes [b0,b1,b2,b3] = [0,1,−1,0].
    let q = tile_quad(
        &vi([0.0, 0.0], [0.0, 1.0, -1.0, 0.0], [1.0, 1.0]),
        tile(2.0, 2.0, [0.5, 0.0]),
    );
    assert!(
        q.world_pos[0].abs() < 1e-6 && (q.world_pos[1] - 0.5).abs() < 1e-6,
        "a 90° um bbox deslocado em +x tem de mover o quad em +y, e deu {:?}",
        q.world_pos
    );
}

/// **A ESCALA DA INSTÂNCIA TAMBÉM PESA O DESLOCAMENTO** — um bbox a meia unidade
/// numa forma dobrada está a uma unidade.
#[test]
fn the_offset_is_scaled_before_it_is_rotated() {
    let q = tile_quad(
        &vi([0.0, 0.0], [1.0, 0.0, 0.0, 1.0], [4.0, 1.0]),
        tile(1.0, 1.0, [0.5, 0.0]),
    );
    assert_eq!(q.world_pos, [2.0, 0.0]);
}

/// **O `tint` CONTINUA A VIAJAR VERBATIM** — o tile é a arte da forma, e a cor da
/// cópia multiplica-o no shader. Pintar aqui aplicaria a cor duas vezes.
#[test]
fn the_instance_tint_is_not_baked_into_the_quad() {
    let hot = VectorInstance {
        tint: [12.0, 6.0, 3.0, 1.0],
        ..vi([0.0, 0.0], [1.0, 0.0, 0.0, 1.0], [1.0, 1.0])
    };
    assert_eq!(
        tile_quad(&hot, tile(1.0, 1.0, [0.0, 0.0])).tint,
        [12.0, 6.0, 3.0, 1.0]
    );
}

/// **O CACHE ASSA UMA VEZ POR GEOMETRIA** — e responde `None` para o que não assou.
///
/// ⚠️ Sem GPU o `bake_missing` não consegue assar nada, e é isso que este gate usa:
/// ele prova que a AUSÊNCIA é reportada em vez de inventada. O caminho positivo é
/// o smoke.
#[test]
fn an_unbaked_geometry_reports_absence_rather_than_a_guess() {
    let bake = ShapeBake::default();
    assert_eq!(bake.tile_for_gid(1), None);
    assert_eq!(bake.tile_for_gid(0), None);
}

/// **O QUE SAIU DE CENA É LARGADO** — a lei que faltava, e que custou um OOM de GPU.
///
/// ⚠️ **Medido** (Enio, 2026-08-21): `wgpu error: Out of Memory` no quadro **19706** da cena
/// `=76`, cujo `trim_offset` é conduzido pelo relógio. Cada quadro dava um `geometry_id`
/// novo, este cache guardava mais um tile, e **cada tile é uma textura de GPU**. O
/// doc-comment do cache já dizia *"o antigo fica órfão até o `release`"* — e o `release` não
/// existia. *Uma frase de doc que descreve uma disciplina não é a disciplina.*
///
/// O gate mede a metade PURA (quem sai), porque a outra é uma linha de `release` e exigiria
/// uma placa de vídeo para correr.
#[test]
fn a_geometry_that_left_the_scene_is_dropped() {
    let mut bake = ShapeBake::default();
    for gid in 1..=4u32 {
        bake.seed_for_test(gid, tile(1.0, 1.0, [0.0, 0.0]));
    }
    let live: std::collections::BTreeSet<u32> = [2u32, 3].into_iter().collect();
    assert_eq!(
        bake.stale_for_test(&live),
        vec![1, 4],
        "os que sairam de cena, e so' eles"
    );
    // E o controle: com tudo vivo, nada é largado — um despejo que largasse o que está
    // em cena apagaria o halo da forma no quadro em que ela ainda se vê.
    let all: std::collections::BTreeSet<u32> = (1..=4u32).collect();
    assert!(bake.stale_for_test(&all).is_empty());
}
