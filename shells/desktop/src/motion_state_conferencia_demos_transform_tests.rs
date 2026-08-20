//! Os gates da cena `=69` — a família TRANSFORM.
//!
//! ⚠️ **Cada par mede a grandeza que a banda ANUNCIA**, e não *"as duas listas diferem"*:
//! quatro dos cinco knobs mexem em `P`, então uma diferença de posições prova pouco. Os
//! oráculos aqui são a orientação, a razão de aspecto e as colunas de identidade.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Os dez streams da cena, na ordem em que ela os monta.
fn bands() -> Vec<Stream> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_transform_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 10, "cinco pares");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            cook.cook(&doc.graph, &reg, *s, 0.0).expect("coze")[0]
                .as_stream()
                .clone()
        })
        .collect()
}

fn p_of(st: &Stream) -> Vec<[f32; 2]> {
    match st.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("toda banda tem posições"),
    }
}

fn scalar(st: &Stream, name: &str) -> Option<Vec<f32>> {
    match st.get(name) {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

fn size_of(st: &Stream) -> Vec<[f32; 2]> {
    match st.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("size"),
    }
}

fn centroid(p: &[[f32; 2]]) -> [f32; 2] {
    let n = p.len() as f32;
    let s = p
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    [s[0] / n, s[1] / n]
}

/// O raio médio das peças em torno do centro da própria banda.
fn spread(p: &[[f32; 2]]) -> f32 {
    let c = centroid(p);
    p.iter()
        .map(|q| (q[0] - c[0]).hypot(q[1] - c[1]))
        .sum::<f32>()
        / p.len() as f32
}

/// As posições **relativas ao centro da própria banda**.
///
/// ⚠️ **Os dois lados de um par vivem em quadrantes diferentes**, então comparar `P`
/// absoluto responde sempre *"diferem"* — sobre o layout, não sobre o knob. Três destes
/// pares afirmam que a FIGURA é a mesma e só um canal muda, e é esta a forma de o dizer.
fn shape(st: &Stream) -> Vec<[f32; 2]> {
    let p = p_of(st);
    let c = centroid(&p);
    p.iter().map(|q| [q[0] - c[0], q[1] - c[1]]).collect()
}

/// As duas figuras coincidem dentro de `SHAPE_EPS`.
///
/// ⚠️ **A barra existe e tem dono: é a CANCELAÇÃO.** As duas bandas de um par vivem em
/// `x = ±5,6`, então `q − centroide` subtrai ~5,6 de ~5,9 em `f32` — o resultado sai com
/// os bits baixos da magnitude GRANDE, e os dois lados perdem-nos de formas diferentes.
/// Medido nesta cena: `0,3000002` contra `0,2999997`, `5e-7` sobre `0,3`. Uma igualdade
/// exacta aqui estaria a medir o layout, não o nó.
const SHAPE_EPS: f32 = 1e-5;

fn same_shape(a: &Stream, b: &Stream, what: &str) {
    let (x, y) = (shape(a), shape(b));
    assert_eq!(x.len(), y.len(), "{what}: contagens diferentes");
    for (i, (p, q)) in x.iter().zip(&y).enumerate() {
        assert!(
            (p[0] - q[0]).abs() < SHAPE_EPS && (p[1] - q[1]).abs() < SHAPE_EPS,
            "{what}: a peça {i} saiu em {p:?} de um lado e {q:?} do outro"
        );
    }
}

/// **NENHUMA BANDA NASCE VAZIA, e os dois lados de cada par DIFEREM.**
///
/// ⚠️ O gate mais barato da cena e o que mais vezes disparou nas irmãs: um fio esquecido
/// dá uma banda vazia, e um `on` que não chega ao nó dá duas bandas idênticas. Os cinco
/// oráculos abaixo continuariam a ler *números*, só que sobre nada.
#[test]
fn every_band_has_elements_and_every_pair_separates() {
    let b = bands();
    for (i, st) in b.iter().enumerate() {
        assert!(st.count() > 0, "a banda {i} saiu vazia");
    }
    for row in 0..5 {
        let (l, r) = (&b[row * 2], &b[row * 2 + 1]);
        let differs = p_of(l) != p_of(r)
            || size_of(l) != size_of(r)
            || scalar(l, "rot") != scalar(r, "rot")
            || scalar(l, "Index") != scalar(r, "Index");
        assert!(differs, "o par {} saiu igual dos dois lados", row + 1);
    }
}

/// **O PASSO LOCAL ABRE O ANEL; O DE MUNDO SÓ O DESLIZA.**
///
/// ⚠️ O oráculo é a extensão em torno do PRÓPRIO centro, e é isso que o torna uma
/// afirmação sobre o espaço do passo: uma translação rígida — que é tudo o que o modo
/// World pode fazer — deixa essa medida intacta, seja qual for `dx`.
#[test]
fn the_local_step_opens_the_ring_and_the_world_step_only_slides() {
    let b = bands();
    let (world, local) = (spread(&p_of(&b[0])), spread(&p_of(&b[1])));
    assert!(
        (world - 1.5).abs() < 1e-3,
        "o anel de mundo tem de manter o raio de 1,5 e mediu {world:.4}"
    );
    assert!(
        local > world + 0.5,
        "em Local cada peça anda para fora: {local:.3} contra {world:.3}"
    );
}

/// **A SEGUNDA MÁSCARA INVERTE A RAZÃO DE ASPECTO AO LONGO DA FILEIRA.**
///
/// ⚠️ E a esquerda é o controle exacto: com uma máscara só, todos os nove crescem
/// mantendo o quadrado — a razão é `1` em todos, e é isso que prova que a diferença
/// vem do CANAL e não de os dois `amount` serem diferentes.
#[test]
fn the_second_mask_turns_one_end_tall_and_the_other_wide() {
    let b = bands();
    let ar = |st: &Stream| -> Vec<f32> { size_of(st).iter().map(|s| s[0] / s[1]).collect() };
    let one = ar(&b[2]);
    for (i, a) in one.iter().enumerate() {
        assert!(
            (a - 1.0).abs() < 1e-4,
            "com uma máscara a peça {i} tem de continuar quadrada, e deu {a:.4}"
        );
    }
    let two = ar(&b[3]);
    assert!(
        two[0] < 0.5,
        "a ponta esquerda tem de sair ALTA e magra: razão {:.3}",
        two[0]
    );
    assert!(
        two[8] > 2.0,
        "e a direita BAIXA e larga: razão {:.3}",
        two[8]
    );
    assert!(
        two.windows(2).all(|w| w[1] > w[0]),
        "e a passagem tem de ser monótona: {two:?}"
    );
}

/// **O GÊMEO REFLECTIDO APONTA PARA FORA; O COPIADO APONTA PARA DENTRO.**
///
/// ⚠️ O oráculo é a coluna `rot`, não a figura: as POSIÇÕES dos dois lados deste par são
/// idênticas (o espelho é o mesmo), então um gate que medisse `P` estaria a medir zero.
#[test]
fn the_mirror_pair_separates_on_the_heading_of_the_twin() {
    let b = bands();
    same_shape(&b[4], &b[5], "as duas bandas deste par são a MESMA figura");
    let copied = scalar(&b[4], "rot").expect("rot");
    let flipped = scalar(&b[5], "rot").expect("rot");
    let n = copied.len() / 2;
    assert_eq!(&copied[..n], &flipped[..n], "os originais não mudam");
    assert_eq!(
        &copied[n..],
        &copied[..n],
        "sem o knob o gêmeo é uma cópia da orientação"
    );
    for (i, (c, f)) in copied[n..].iter().zip(&flipped[n..]).enumerate() {
        assert!(
            (f - (180.0 - c)).abs() < 1e-3,
            "o gêmeo {i} tem de ser o reflexo `180 − θ`: {f} contra {c}"
        );
    }
}

/// **A MANDALA RENUMERADA É UMA LISTA SÓ** — e a renumeração não move uma peça.
#[test]
fn the_mandala_pair_separates_on_the_identity_columns_and_not_on_the_shape() {
    let b = bands();
    same_shape(&b[6], &b[7], "a figura é a mesma");
    let repeated = scalar(&b[6], "Index").expect("Index");
    let once = scalar(&b[7], "Index").expect("Index");
    assert_eq!(repeated.len(), 36, "seis fatias de seis");
    assert_eq!(
        &repeated[..6],
        &repeated[6..12],
        "sem o knob o Index recomeça em cada fatia"
    );
    let ramp: Vec<f32> = (0..36).map(|i| i as f32).collect();
    assert_eq!(once, ramp, "com ele a lista é uma só");
    assert_eq!(scalar(&b[7], "Count"), Some(vec![36.0; 36]));
}

/// **A ÓRBITA QUE LEVA A ORIENTAÇÃO MANTÉM OS RAIOS RADIAIS.**
///
/// ⚠️ O oráculo é o ÂNGULO ENTRE a orientação da peça e a direção dela ao centro — a
/// grandeza que a banda anuncia. As posições dos dois lados são idênticas (a órbita é a
/// mesma), e sem o knob esse ângulo é exactamente a volta que a órbita deu.
#[test]
fn the_orbit_pair_separates_on_whether_the_spokes_stay_radial() {
    let b = bands();
    same_shape(&b[8], &b[9], "a órbita é a mesma dos dois lados");
    let off_radial = |st: &Stream| -> Vec<f32> {
        let p = p_of(st);
        let c = centroid(&p);
        let rot = scalar(st, "rot").expect("o `align` do anel escreve a orientação");
        p.iter()
            .zip(&rot)
            .map(|(q, r)| {
                let outward = (q[1] - c[1]).atan2(q[0] - c[0]).to_degrees();
                // A diferença dobrada a `(−180, 180]`.
                let d = (r - outward).rem_euclid(360.0);
                if d > 180.0 { d - 360.0 } else { d }
            })
            .collect()
    };
    for (i, d) in off_radial(&b[9]).iter().enumerate() {
        assert!(
            d.abs() < 2.0,
            "com Carry Rotation o raio {i} continua radial, e desviou {d:.2}°"
        );
    }
    let (_, turn) = authored();
    for (i, d) in off_radial(&b[8]).iter().enumerate() {
        assert!(
            (d.abs() - turn).abs() < 2.0,
            "sem ele o raio {i} fica torto pela volta inteira ({turn}°), e desviou {d:.2}°"
        );
    }
}
