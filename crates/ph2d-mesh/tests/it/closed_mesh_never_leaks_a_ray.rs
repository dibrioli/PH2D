//! **UMA MALHA FECHADA NÃO DEIXA UM RAIO PASSAR** — a propriedade que o pick
//! precisa e que o Möller–Trumbore estrito não entregava.
//!
//! ⚠️ **O oráculo é TOPOLOGIA, não uma tolerância escolhida.** Um raio que parte
//! de fora de uma superfície fechada e aponta para dentro dela **tem** de
//! encontrá-la — não há número a calibrar, não há épsilon a afinar: ou o teste
//! de interseção é estanque ou ele tem furos. É por isso que este gate pode
//! afirmar `0` misses em vez de *"poucos"*.
//!
//! # O defeito que ele nasceu vermelho a medir
//!
//! `ray::ray_triangle` testava `u` e `v` contra `0.0..=1.0` **sem folga**, e o
//! cabeçalho do irmão `tri_geom.rs` justificava a escolha assim:
//!
//! > *"um falso positivo na aresta partilhada elege o triângulo vizinho, e o
//! > artista não distingue"*
//!
//! ⚠️ **A justificativa raciocina sobre os DOIS triângulos aceitarem, e nunca
//! sobre NENHUM aceitar.** Um teste estrito numa aresta exata recusa dos dois
//! lados — o resultado não é uma escolha ambígua, é um **buraco**. E o buraco
//! não é raro nesta malha: [`Mesh::raycast`] parte cada QUAD em `(0,1,2)` e
//! `(0,2,3)`, que partilham a diagonal e a testam com ordens de vértice
//! diferentes, então cada uma das 98.304 faces da esfera de fábrica carrega uma
//! aresta interna por onde vazar.
//!
//! **Medido pela porta do produto** (`measure_stroke_ripple`, §3): de 41 dabs
//! emitidos num traço, os APLICADOS eram `40, 40, 39, 39, 39, 38, 35` — e
//! missavam **também contra a esfera pristina**, então não era o barro que tinha
//! subido. No shell isso não custava um carimbo: `sculpt3d_input.rs` faz `break`
//! quando um pick erra e a âncora avança para o fim do percurso mesmo assim, de
//! modo que **um** furo no dab 7 de 41 apagava os dabs 8..41.

use ph2d_mesh::{Mesh, Ray, shapes};

/// Direções distribuídas por espiral de Fibonacci — determinísticas e sem
/// alinhamento com nenhum eixo da malha, que é o que faz esta varredura
/// encontrar arestas partilhadas em vez de as evitar por sorte.
fn fibonacci_dirs(n: usize) -> Vec<[f32; 3]> {
    // O ângulo de ouro em `f64`, arredondado uma vez — a espiral tem de ser a
    // mesma em qualquer máquina para o número de misses ser reproduzível.
    let ga = std::f64::consts::PI * (3.0 - 5.0f64.sqrt());
    (0..n)
        .map(|i| {
            let z = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let r = (1.0 - z * z).max(0.0).sqrt();
            let th = ga * i as f64;
            [(r * th.cos()) as f32, (r * th.sin()) as f32, z as f32]
        })
        .collect()
}

/// Quantos raios de um leque centrado NÃO encontram a malha.
fn leaks(mesh: &Mesh, n: usize, distance: f32) -> Vec<[f32; 3]> {
    fibonacci_dirs(n)
        .into_iter()
        .filter(|d| {
            let origin = [d[0] * distance, d[1] * distance, d[2] * distance];
            let dir = [-d[0], -d[1], -d[2]];
            mesh.raycast(&Ray::new(origin, dir)).is_none()
        })
        .collect()
}

/// **A ENTREGA.** A esfera de fábrica — a que todo smoke deste módulo abre — não
/// pode deixar um único raio passar.
#[test]
fn the_factory_sphere_never_leaks_a_ray_aimed_at_its_centre() {
    let mesh = shapes::sculpt_sphere(1.0);
    let missed = leaks(&mesh, 4096, 3.0);
    assert!(
        missed.is_empty(),
        "{} de 4096 raios atravessaram uma superfície FECHADA de {} faces; \
         o primeiro foi {:?}",
        missed.len(),
        mesh.face_count(),
        missed.first()
    );
}

/// **O MESMO, com a malha de TRIÂNGULOS** — o quad não é a única fonte de
/// aresta partilhada, e sem esta metade a cura podia ser um caso especial de
/// quad em vez da estanqueidade do teste.
#[test]
fn a_triangle_sphere_never_leaks_a_ray_either() {
    let mesh = shapes::uv_sphere(48, 72, 1.0);
    let missed = leaks(&mesh, 4096, 3.0);
    assert!(
        missed.is_empty(),
        "{} de 4096 raios atravessaram a esfera de triângulos; o primeiro foi {:?}",
        missed.len(),
        missed.first()
    );
}

/// **O CONTROLE que impede o gate de passar por vácuo.** Um leque que erra a
/// malha de propósito tem de errar TODAS — sem esta metade, um `raycast` que
/// devolvesse `Some` para qualquer raio satisfaria os dois gates acima.
#[test]
fn a_fan_that_misses_the_mesh_misses_all_of_it() {
    let mesh = shapes::sculpt_sphere(1.0);
    // Origens no leque, mas apontando para FORA — nenhuma pode encontrar nada.
    let outward = fibonacci_dirs(512)
        .into_iter()
        .filter(|d| {
            let origin = [d[0] * 3.0, d[1] * 3.0, d[2] * 3.0];
            mesh.raycast(&Ray::new(origin, *d)).is_some()
        })
        .count();
    assert_eq!(outward, 0, "{outward} raios apontando para fora acertaram");
}

/// **O HIT/MISS NÃO PODE VIRAR NUM ULP.** A assinatura do vazamento medida na
/// sonda era exactamente esta: perturbar a direção no último bit trocava a
/// resposta. Um teste estanque é *estável* sob perturbação de um ULP porque a
/// aresta deixa de ser uma fronteira de decisão.
///
/// ⚠️ O oráculo **não** é *"o `t` é o mesmo"* — dois triângulos vizinhos podem
/// legitimamente devolver `t` a alguns ULPs de distância. É *"acertou"* que tem
/// de ser estável.
/// Möller–Trumbore **sem octree**, varrendo TODA face — o discriminante entre
/// *o teste de triângulo tem furo* e *a poda do octree pulou a folha certa*.
///
/// ⚠️ Ele repete a aritmética de propósito: chamar a porta do produto responderia
/// a mesma pergunta com o mesmo mecanismo, e o que se quer aqui é **um segundo
/// caminho** que não conhece caixa nenhuma.
fn brute_hit(mesh: &Mesh, origin: [f32; 3], dir: [f32; 3]) -> bool {
    const SLACK: f32 = 1e-6;
    let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let cross = |a: [f32; 3], b: [f32; 3]| {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    };
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let tri = |a: [f32; 3], b: [f32; 3], c: [f32; 3]| {
        let (e1, e2) = (sub(b, a), sub(c, a));
        let p = cross(dir, e2);
        let det = dot(e1, p);
        if det.abs() < 1e-12 {
            return false;
        }
        let inv = 1.0 / det;
        let tv = sub(origin, a);
        let u = dot(tv, p) * inv;
        if !(-SLACK..=1.0 + SLACK).contains(&u) {
            return false;
        }
        let q = cross(tv, e1);
        let v = dot(dir, q) * inv;
        if v < -SLACK || u + v > 1.0 + SLACK {
            return false;
        }
        dot(e2, q) * inv >= 0.0
    };
    mesh.faces().iter().any(|f| {
        let vs = f.verts();
        let p = |k: usize| mesh.positions()[vs[k] as usize];
        tri(p(0), p(1), p(2)) || (vs.len() == 4 && tri(p(0), p(2), p(3)))
    })
}

#[test]
fn a_one_ulp_nudge_never_flips_a_hit_into_a_miss() {
    let mesh = shapes::sculpt_sphere(1.0);
    let mut flipped = 0usize;
    let mut brute_agrees = 0usize;
    for d in fibonacci_dirs(1024) {
        let origin = [d[0] * 3.0, d[1] * 3.0, d[2] * 3.0];
        let base = [-d[0], -d[1], -d[2]];
        let hit0 = mesh.raycast(&Ray::new(origin, base)).is_some();
        for axis in 0..3 {
            for up in [true, false] {
                let mut n = base;
                n[axis] = if up {
                    f32::from_bits(n[axis].to_bits().wrapping_add(1))
                } else {
                    f32::from_bits(n[axis].to_bits().wrapping_sub(1))
                };
                // ⚠️ **UM ULP ABAIXO DE `+0.0` É NaN, e é a minha fixture, não o
                // produto.** `0.0f32.to_bits()` é `0`, e `wrapping_sub(1)` dá
                // `0xFFFF_FFFF` — um NaN. `Ray::new` recusa direção não-finita
                // por contrato, então o "flip" seria a porta a honrar a própria
                // recusa. *Um raio que não é um raio não é contra-exemplo.*
                if !n.iter().all(|c| c.is_finite()) {
                    continue;
                }
                let r = Ray::new(origin, n);
                let hit = mesh.raycast(&r).is_some();
                if hit != hit0 {
                    flipped += 1;
                    // ⚠️ O DISCRIMINANTE: a força-bruta concorda com a porta?
                    // Se sim, o furo é do TESTE DE TRIÂNGULO; se não, é da PODA.
                    if brute_hit(&mesh, r.origin(), r.dir()) == hit {
                        brute_agrees += 1;
                    }
                }
            }
        }
    }
    assert_eq!(
        flipped,
        0,
        "{flipped} de 6144 empurrões de um ULP trocaram acerto por erro \
         (força-bruta CONCORDA com a porta em {brute_agrees} deles ⇒ \
         {} é o suspeito)",
        if brute_agrees == flipped {
            "o TESTE DE TRIÂNGULO"
        } else {
            "a PODA do octree"
        }
    );
}
