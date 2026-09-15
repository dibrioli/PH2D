//! Os gates da deformação medida AO TAMANHO DO DAB — ver [`super`].

use crate::SpriteMesh;
use crate::sprite_mesh::{barycentric, warp_of};
use crate::sprite_mesh_warp::warp_over;

const SW: f32 = 2.0;
const SIZE: [f32; 2] = [SW, SW];

/// ⭐ **O LEQUE** — a arte dobrada da foto do dono: `u` corre ao longo do braço, `v` atravessa-o, e
/// o raio decide quanto cada faixa comprime. É a fixtura que tem termos fora da diagonal (a lição
/// da W11b) **e** vários triângulos debaixo de um dab (a desta wave).
fn leque(n: usize, theta: f32, r0: f32) -> SpriteMesh {
    let ponto = |u: f32, v: f32| {
        let (ang, r) = (theta * u, r0 + (0.5 - v) * SW);
        [r * ang.sin(), r * ang.cos()]
    };
    let (mut local, mut uv, mut tris) = (Vec::new(), Vec::new(), Vec::new());
    for j in 0..=n {
        for i in 0..=n {
            let (u, v) = (i as f32 / n as f32, j as f32 / n as f32);
            uv.push([u, v]);
            local.push(ponto(u, v));
        }
    }
    let idx = |i: usize, j: usize| (j * (n + 1) + i) as u32;
    for j in 0..n {
        for i in 0..n {
            tris.push([idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)]);
            tris.push([idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)]);
        }
    }
    SpriteMesh { local, uv, tris }
}

/// O ponto POSADO da UV de repouso `p`, pela malha (o que o rasterizador desenha ali).
fn ecra(mesh: &SpriteMesh, p: [f32; 2]) -> Option<[f32; 2]> {
    mesh.triangles().find_map(|t| {
        let w = barycentric(p, [mesh.uv[t[0]], mesh.uv[t[1]], mesh.uv[t[2]]])?;
        let l = [mesh.local[t[0]], mesh.local[t[1]], mesh.local[t[2]]];
        Some([
            w[0] * l[0][0] + w[1] * l[1][0] + w[2] * l[2][0],
            -(w[0] * l[0][1] + w[1] * l[1][1] + w[2] * l[2][1]),
        ])
    })
}

/// ⭐⭐⭐ **A RÉGUA É O PRODUTO:** a marca que o pincel pinta com esta deformação, levada ao ecrã
/// PELA MALHA, é redonda? Devolve `maior/menor` raio do contorno (`1` = disco perfeito).
fn redondeza(mesh: &SpriteMesh, centro: [f32; 2], raio_uv: f32, w: crate::MeshWarp) -> f32 {
    marca(mesh, centro, raio_uv, w, 0.0, 0).0
}

/// A MARCA que chega ao ecrã, para um dab que o artista autorou com `flatten`/`angle`: o seu
/// `maior/menor` e a DIRECÇÃO do eixo maior, em graus (`0..180`, no referencial do ecrã).
///
/// ⚠️ **Ela percorre o contorno do PRODUTO** ([`ph2d_painter_brush::FootprintDeform::outline_at`]),
/// e não uma elipse reconstruída aqui: desde que a pegada carrega curvatura, a fronteira do que o
/// motor pinta já não é uma elipse, e uma régua que a supusesse mediria outra coisa.
fn marca(
    mesh: &SpriteMesh,
    centro: [f32; 2],
    raio_uv: f32,
    w: crate::MeshWarp,
    flatten: f32,
    angle: u16,
) -> (f32, f32) {
    let d = ph2d_painter_brush::canvas_warp::warped_dab(
        ph2d_painter_brush::canvas_warp::CanvasWarp {
            linear: w.linear,
            curve: w.curve,
        },
        flatten,
        angle,
    );
    let pegada =
        ph2d_painter_brush::FootprintDeform::new(d.flatten, d.angle_deg).with_curve(d.curve);
    let c0 = ecra(mesh, centro).expect("o centro cai sobre a malha");
    let (mut lo, mut hi) = (f32::INFINITY, 0.0f32);
    let mut eixo = [1.0f32, 0.0];
    for k in 0..360 {
        let p = pegada.outline_at(k as f32 / 360.0);
        let q = [
            centro[0] + p[0] * raio_uv * d.radius_scale,
            centro[1] + p[1] * raio_uv * d.radius_scale,
        ];
        let s = ecra(mesh, q).expect("o bordo do dab cai sobre a malha");
        let r = ((s[0] - c0[0]).powi(2) + (s[1] - c0[1]).powi(2)).sqrt();
        lo = lo.min(r);
        if r > hi {
            hi = r;
            eixo = [s[0] - c0[0], s[1] - c0[1]];
        }
    }
    let mut g = eixo[1].atan2(eixo[0]).to_degrees();
    while g < 0.0 {
        g += 180.0;
    }
    while g >= 180.0 {
        g -= 180.0;
    }
    (hi / lo, g)
}

/// ⭐⭐⭐ **UM DAB QUE CABE NUMA FACETE RECEBE A FACETE, AO BIT.** Dentro de um triângulo o afim é
/// exacto, logo não há nada para ajustar — e uns mínimos quadrados sobre ele devolveriam o mesmo
/// número com o ruído de um `f32` por cima, que o atalho da identidade do pincel não perdoa.
///
/// ⛔ O CONTROLO é o mesmo ponto com um footprint GRANDE: ali a resposta **tem** de mudar, senão
/// este gate passaria sobre uma porta que ignora o footprint.
#[test]
fn a_dab_inside_one_facet_gets_the_facet_bit_for_bit() {
    let mesh = leque(8, 1.2, 1.4);
    // O centroide da facete `(i=4, j=5)` — a `0,0295` de UV da aresta mais próxima.
    let centro = [0.5 + 0.125 * 2.0 / 3.0, 0.625 + 0.125 / 3.0];
    let p = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
    let facete = warp_over(&mesh, p, SIZE, [0.0, 0.0]).expect("a facete do ponto");
    let pequeno = warp_over(&mesh, p, SIZE, [0.02, 0.02]).expect("um dab pequeno");
    assert_eq!(
        pequeno, facete,
        "um dab que cabe na facete tem de receber a facete AO BIT"
    );
    let grande = warp_over(&mesh, p, SIZE, [0.09, 0.09]).expect("um dab grande");
    assert!(
        grande != facete,
        "o CONTROLO: um dab que atravessa facetes TEM de mover a resposta ({grande:?})"
    );
}

/// ⭐⭐⭐ **O QUE A FOTO PEDIA: com um pincel GRANDE sobre uma malha grossa, a facete pinta uma marca
/// MENOS redonda do que não corrigir nada — e medir ao tamanho do dab tira esse caso.**
///
/// ⚠️ **A régua é o PRODUTO** — a marca levada ao ecrã PELA MALHA —, nunca os três números que o
/// pincel consome. Medido no leque de `1,2 rad` sobre uma malha `8×8`, a redondeza (`1` = disco):
///
/// | ponto | raio | sem correcção | facete | **ao tamanho do dab** |
/// |---|---|---|---|---|
/// | `(0,53 · 0,72)` | `0,060` | `1,863` | `1,163` | **`1,099`** |
/// | `(0,72 · 0,81)` | `0,060` | `2,147` | `1,216` | **`1,128`** |
/// | `(0,40 · 0,65)` | `0,060` | `1,602` | `1,303` | **`1,115`** |
/// | `(0,53 · 0,72)` | `0,125` | `1,750` | `1,231` | **`1,078`** |
/// | `(0,72 · 0,81)` | `0,125` | `2,138` | `1,267` | **`1,078`** |
/// | `(0,40 · 0,65)` | `0,125` | `1,529` | `1,299` | **`1,065`** |
///
/// ⚠️ **Os números da coluna da direita MUDARAM** quando a porta passou a devolver um polinómio de
/// grau `3` (eram `1,13` · `1,16` · `1,12` na linha de `0,125`): *o gate mede a lei, e a lei
/// melhorou por baixo dele.* A barra continua a ser a mesma — ela pergunta se medir ao tamanho do
/// dab BATE a facete, não por quanto.
///
/// ⚠️ **A margem de `0,02` e o raio PEQUENO são o que pinam a lei, e não decoração:** amostrar a
/// METADE do raio (a variante natural) deixa a coluna do raio `0,06` em `1,162` contra `1,162` da
/// facete — *ganho zero*. É por isso que o gate exige a melhoria em **todas** as seis células.
#[test]
fn a_big_dab_is_rounder_when_the_deformation_is_measured_at_its_size() {
    let mesh = leque(8, 1.2, 1.4);
    for raio in [0.06_f32, 0.125] {
        for centro in [[0.531_f32, 0.719_f32], [0.719, 0.806], [0.40, 0.65]] {
            let p = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
            let facete = warp_over(&mesh, p, SIZE, [0.0, 0.0]).expect("a facete");
            let ajustada = warp_over(&mesh, p, SIZE, [raio, raio]).expect("o footprint");
            let (rf, ra) = (
                redondeza(&mesh, centro, raio, facete),
                redondeza(&mesh, centro, raio, ajustada),
            );
            assert!(
                ra <= rf - 0.02,
                "medir ao tamanho do dab tem de deixar a marca MAIS redonda: {ra} contra {rf} \
                 (raio {raio}, centro {centro:?})"
            );
            // O chão da wave anterior: continua muito melhor do que não corrigir nada.
            let crua = redondeza(&mesh, centro, raio, crate::MeshWarp::rest());
            assert!(
                ra < crua,
                "a correcção tem de bater o não-corrigir: {ra} contra {crua} em {centro:?}"
            );
        }
    }
}

/// ⭐⭐⭐ **A ELIPSE QUE O ARTISTA AUTOROU CHEGA AO ECRÃ COMO ELE A DESENHOU** — e este é o gate que
/// mede a BASE do ajuste, porque **um dab REDONDO não consegue medi-la**.
///
/// ⛔⛔ *Uma matriz multiplicada por um factor ORTOGONAL tem os MESMOS valores singulares e os
/// MESMOS eixos.* Com `flatten = 0` a elipse autorada é o círculo unitário, então trocar a base do
/// ajuste por um espelho devolve a mesma resposta **ao bit** — a mutação que o faz sobrevive a
/// todos os gates de redondeza. É a lição da W11b uma volta mais fundo: lá a fixtura alinhada aos
/// eixos não media a base; aqui é o **pincel redondo** que não a mede.
///
/// Com `flatten = 0,4` e `angle = 30°` o espelho salta à vista: o eixo maior chega ao ecrã a
/// **`29,8°`–`38,3°`** com a lei certa e a **`152,6°`–`164,2°`** com a base trocada. A barra é
/// `15°` — o dobro do pior desvio medido, e `114°` abaixo do mutante.
#[test]
fn an_authored_ellipse_arrives_on_screen_as_the_artist_drew_it() {
    let mesh = leque(8, 1.2, 1.4);
    const FLATTEN: f32 = 0.4;
    const ANGULO: u16 = 30;
    for raio in [0.06_f32, 0.125] {
        for centro in [[0.531_f32, 0.719_f32], [0.719, 0.806], [0.40, 0.65]] {
            let p = ecra(&mesh, centro).map(|s| [s[0], -s[1]]).expect("posado");
            let w = warp_over(&mesh, p, SIZE, [raio, raio]).expect("o footprint");
            let (aspecto, graus) = marca(&mesh, centro, raio, w, FLATTEN, ANGULO);
            let desvio = (graus - f32::from(ANGULO))
                .abs()
                .min(180.0 - (graus - f32::from(ANGULO)).abs());
            assert!(
                desvio < 15.0,
                "o eixo maior da marca chegou a {graus}° e o artista desenhou {ANGULO}° \
                 (raio {raio}, centro {centro:?})"
            );
            let quer = 1.0 / (1.0 - FLATTEN);
            assert!(
                (aspecto - quer).abs() < 0.2,
                "a marca chegou com aspecto {aspecto} e o artista desenhou {quer} \
                 (raio {raio}, centro {centro:?})"
            );
        }
    }
}

/// ⛔ **Uma malha de UM triângulo responde pelo afim dele, mesmo com o dab a transbordar** — as
/// amostras que caem fora da arte não puxam o ajuste, e a borda comporta-se como antes desta wave.
#[test]
fn samples_off_the_mesh_answer_by_the_facet_they_left() {
    let mesh = SpriteMesh {
        local: vec![[3.0, 0.0], [4.0, 0.0], [3.0, 2.0]],
        uv: vec![[0.0, 1.0], [1.0, 1.0], [0.0, 0.0]],
        tris: vec![[0, 1, 2]],
    };
    let facete = warp_of(&mesh, [0, 1, 2], SIZE).expect("o afim do triangulo");
    for r in [0.01_f32, 0.2, 5.0] {
        assert_eq!(
            warp_over(&mesh, [3.2, 0.2], SIZE, [r, r]),
            Some(crate::MeshWarp::linear(facete)),
            "com raio {r} o unico triangulo continua a ser a resposta"
        );
    }
}

/// ⭐ **As SONDAS que mediram o que sobra** — e as duas explicações que elas refutaram. Ver o
/// cabeçalho de [`probe`].
#[path = "sprite_mesh_warp_probe.rs"]
mod probe;
