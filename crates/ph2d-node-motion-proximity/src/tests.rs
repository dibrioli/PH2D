//! Gates do `motion.proximity`.
//!
//! Filho (`#[path]`), não irmão: `use super::*` continua alcançando os privados
//! (`measure`, `radius_scale`) que os gates medem.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fila de discos igualmente espaçados em `x`, todos de tamanho `1`.
fn row(n: usize, pitch: f32) -> Vec<[f32; 2]> {
    (0..n).map(|i| [i as f32 * pitch, 0.0]).collect()
}

fn uniform(n: usize, r: f32) -> Vec<f32> {
    vec![r; n]
}

/// **O CONTROLE.** Uma nuvem folgada não reporta vizinho nenhum — sem ele, um nó que
/// devolvesse "toda a gente toca toda a gente" passaria em metade dos gates abaixo.
#[test]
fn a_free_cloud_reports_zero_in_both_channels() {
    // Passo 1.0 contra `min_dist = 0.6`: ninguém se toca.
    let p = row(5, 1.0);
    let (neighbours, overlap) = measure(&p, &uniform(5, 0.3));
    assert_eq!(neighbours, vec![0.0; 5], "ninguem toca ninguem nesta fila");
    assert_eq!(overlap, vec![0.0; 5], "sem toque, sobreposicao zero");
}

/// A sobreposição é a FRACÇÃO da distância do par, não a distância nem a área.
#[test]
fn the_overlap_is_the_fraction_of_the_pair_distance() {
    // Dois discos de raio 0,3 (min_dist 0,6) a 0,45 ⇒ 1 − 0,45/0,6 = 0,25.
    let p = vec![[0.0, 0.0], [0.45, 0.0]];
    let (neighbours, overlap) = measure(&p, &uniform(2, 0.3));
    assert_eq!(neighbours, vec![1.0, 1.0]);
    for o in &overlap {
        assert!(
            (o - 0.25).abs() < 1e-6,
            "1 - 0.45/0.6 = 0.25, veio {o} -- a lei nao e' a fraccao do par"
        );
    }
}

/// Coincidentes: a direcção é indefinida e a sobreposição é CHEIA — o mesmo canto que o
/// `motion.collide` trata como penetração `min_dist`.
#[test]
fn coincident_discs_overlap_fully() {
    let p = vec![[1.5, -2.0], [1.5, -2.0]];
    let (_, overlap) = measure(&p, &uniform(2, 0.3));
    assert_eq!(
        overlap,
        vec![1.0, 1.0],
        "coincidentes sobrepoem-se por inteiro"
    );
}

/// A contagem é POR VIZINHO e simétrica: numa fila de três a que se tocam em cadeia, o do
/// meio tem dois e as pontas um. (Uma contagem de PARES daria 1/1/1; uma contagem que
/// esquecesse o lado esquerdo daria 1/1/0.)
#[test]
fn the_count_is_per_neighbour_and_symmetric() {
    // Passo 0,5 contra min_dist 0,6: cada um toca só o vizinho imediato.
    let p = row(3, 0.5);
    let (neighbours, _) = measure(&p, &uniform(3, 0.3));
    assert_eq!(neighbours, vec![1.0, 2.0, 1.0]);
}

/// **A lei é `r_i + r_j`, e a coluna `size` entra nela** — um disco GRANDE alcança um
/// vizinho que um raio uniforme não alcançaria. É a lei do `motion.collide`, e partilhá-la
/// é o que faz *o que este nó reporta* ser *o que aquele empurraria*.
#[test]
fn the_size_column_sets_each_discs_own_radius() {
    // A 0,7 de distância, dois discos de 0,3 NÃO se tocam (min_dist 0,6)...
    let p = vec![[0.0, 0.0], [0.7, 0.0]];
    let (uniform_n, _) = measure(&p, &uniform(2, 0.3));
    assert_eq!(
        uniform_n,
        vec![0.0, 0.0],
        "0.3 + 0.3 = 0.6 < 0.7: nao tocam"
    );
    // ...e com o primeiro ao dobro do tamanho, tocam (0,6 + 0,3 = 0,9 > 0,7).
    let (mixed_n, mixed_o) = measure(&p, &[0.6, 0.3]);
    assert_eq!(mixed_n, vec![1.0, 1.0], "0.6 + 0.3 = 0.9 > 0.7: tocam");
    for o in &mixed_o {
        assert!(
            (o - (1.0 - 0.7 / 0.9)).abs() < 1e-6,
            "a fraccao usa o min_dist do PAR (0,9), veio {o}"
        );
    }
    // E o raio de cada disco sai da coluna `size` pela mesma lei do renderer.
    let s = Stream::new(2).with(SIZE_COL, Column::Vec2(vec![[2.0, 1.0], [-1.0, f32::NAN]]));
    assert_eq!(
        radius_scale(&s, 2),
        vec![2.0, 1.0],
        "max(|x|,|y|); nao-finito le como a identidade 1"
    );
}

/// **A resposta segue uma PERMUTAÇÃO do input, ao bit.** É esta propriedade — `max` e
/// contagem, nunca uma soma — que torna a ORDEM DE VISITA não-observável, e é o que o irmão
/// `motion.collide` teve de conquistar trocando Gauss–Seidel por Jacobi médio.
///
/// ⚠️ **Ela NÃO é a paridade com o device, e a distinção é o que a medição ensinou:** a ordem
/// deixa de importar, mas o VALOR que entra no `max` sai de um produto escalar que o WGSL pode
/// fundir num `fma` ⇒ a contagem sai exacta e a fracção a um ULP (`gpu_proximity`).
#[test]
fn the_answer_follows_a_permutation_of_the_input_bit_for_bit() {
    let p = vec![
        [0.0, 0.0],
        [0.4, 0.1],
        [0.15, -0.35],
        [1.9, 2.0],
        [0.55, 0.2],
    ];
    let r = [0.3, 0.2, 0.35, 0.3, 0.25];
    let (n0, o0) = measure(&p, &r);
    // A mesma cena, listada ao contrário.
    let rev_p: Vec<_> = p.iter().rev().copied().collect();
    let rev_r: Vec<_> = r.iter().rev().copied().collect();
    let (n1, o1) = measure(&rev_p, &rev_r);
    for i in 0..p.len() {
        let j = p.len() - 1 - i;
        assert_eq!(n0[i], n1[j], "a contagem de {i} mudou com a ordem da lista");
        assert_eq!(
            o0[i].to_bits(),
            o1[j].to_bits(),
            "a sobreposicao de {i} mudou com a ordem da lista: {} vs {}",
            o0[i],
            o1[j]
        );
    }
}

/// **Encolher por `1 − overlap` faz a sobreposição DESAPARECER** — o modo *Scale* do Push
/// Apart do C4D, por composição (`value.math(1−x) → motion.drive(Size, Multiply)`).
///
/// ⚠️ A barra é `1e-6` e não zero **por representação, não por folga**: `(1−o)·min_dist`
/// reconstrói `d` a menos de um ulp, então o par fica a tocar-se por dentro de um
/// arredondamento. Um par que continuasse sobreposto de verdade mediria a fracção que
/// ficou por resolver, ordens acima disto.
#[test]
fn shrinking_by_one_minus_the_overlap_removes_every_overlap() {
    let p = vec![
        [0.0, 0.0],
        [0.35, 0.05],
        [0.1, -0.3],
        [0.6, 0.25],
        [0.2, 0.4],
    ];
    let r0 = uniform(5, 0.3);
    let (_, o) = measure(&p, &r0);
    assert!(
        o.iter().any(|v| *v > 0.1),
        "a fixture TEM de conter o fenomeno"
    );
    // O que a cadeia faz: `size *= 1 − overlap`, logo o raio também.
    let r1: Vec<f32> = r0.iter().zip(&o).map(|(r, o)| r * (1.0 - o)).collect();
    let (_, o1) = measure(&p, &r1);
    for (i, v) in o1.iter().enumerate() {
        assert!(
            *v <= 1e-6,
            "o disco {i} continua sobreposto por {v} depois de encolher"
        );
    }
    // ⚠️ **O CONTROLE, e a minha primeira versão dele afirmava algo FALSO sobre código
    // correto:** eu pedi que sobrasse pelo menos um CONTACTO, e não sobra nenhum — a lei
    // conta toque com `<` ESTRITO, então dois discos que acabam exactamente encostados
    // medem zero vizinhos, legitimamente. O que separa *encolher o justo* de *encolher
    // demais* não é a contagem, é a FOLGA: o par globalmente pior (o único cujos dois
    // discos têm ESTE par como o seu pior) tem de acabar a tocar-se, `d/min_dist' = 1`.
    // Sem esta metade, encolher tudo a zero passaria no laço acima.
    let (mut wi, mut wj, mut worst) = (0usize, 1usize, -1.0f32);
    for i in 0..p.len() {
        for j in (i + 1)..p.len() {
            let d = dist(p[i], p[j]);
            let f = 1.0 - d / (r0[i] + r0[j]);
            if f > worst {
                (wi, wj, worst) = (i, j, f);
            }
        }
    }
    let touch = dist(p[wi], p[wj]) / (r1[wi] + r1[wj]);
    assert!(
        (touch - 1.0).abs() < 1e-6,
        "o pior par ({wi},{wj}) devia acabar a TOCAR-SE; a razao d/min_dist' veio {touch} \
         -- abaixo de 1 sobrou sobreposicao, acima de 1 o Scale encolheu demais"
    );
    assert!(
        r1.iter().all(|r| *r > 0.0),
        "nenhum disco pode encolher a nada: {r1:?}"
    );
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    (dx * dx + dy * dy).sqrt()
}

/// **Ele MEDE, então ignora a máscara.** O `falloff` é a espinha de quem MODIFICA; uma
/// medição mascarada mente ao `value.attribute` que a lê. (O `motion.collide`, que
/// modifica, honra-a — é a distinção, não um esquecimento.)
#[test]
fn the_measurement_ignores_the_falloff_mask() {
    let p = row(3, 0.5);
    let bare = Stream::new(3).with("P", Column::Vec2(p.clone()));
    let masked = Stream::new(3)
        .with("P", Column::Vec2(p))
        .with("falloff", Column::Scalar(vec![0.0, 0.0, 0.0]));
    assert_eq!(cook_counts(&bare), cook_counts(&masked));
}

// ── nível de eval: o nó dentro de um cook ────────────────────────────────────

const SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.proximity.test.src"),
    name: "motion.proximity.test.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Src(Stream);
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(self.0.clone());
    }
}

struct Ops(Src);
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&self.0),
            t if t == MANIFEST.id => Some(&MotionProximity),
            _ => None,
        }
    }
}

/// Coze `src → proximity(radius 0.3)` e devolve o stream de saída.
fn cook_stream(input: &Stream) -> Stream {
    let ops = Ops(Src(input.clone()));
    let mut g = Graph::new();
    let src = g.add_node("motion.proximity.test.src");
    let prox = g.add_node("motion.proximity");
    g.set_param(prox, "radius", 0.3);
    g.connect(Edge {
        from: (src, 0),
        to: (prox, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    cook.cook(&g, &ops, prox, 0.0).unwrap()[0]
        .as_stream()
        .clone()
}

fn cook_counts(input: &Stream) -> Vec<f32> {
    match cook_stream(input).get(NEIGHBOURS_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("o no' TEM de emitir a coluna `{NEIGHBOURS_COL}`"),
    }
}

/// As duas colunas chegam ao stream, e todo o resto passa intacto.
#[test]
fn the_two_columns_land_and_every_other_column_rides_through() {
    let input = Stream::new(3)
        .with("P", Column::Vec2(row(3, 0.5)))
        .with("tint", Column::Vec4(vec![[1.0, 0.0, 0.0, 1.0]; 3]));
    let out = cook_stream(&input);
    assert_eq!(out.count(), 3, "a contagem nao muda: o no' nao apaga nada");
    assert!(
        matches!(out.get("tint"), Some(Column::Vec4(_))),
        "o `tint` sobreviveu"
    );
    assert!(
        matches!(out.get("P"), Some(Column::Vec2(_))),
        "o `P` sobreviveu"
    );
    match (out.get(NEIGHBOURS_COL), out.get(OVERLAP_COL)) {
        (Some(Column::Scalar(n)), Some(Column::Scalar(o))) => {
            assert_eq!(n, &vec![1.0, 2.0, 1.0]);
            assert!(o.iter().all(|v| *v > 0.0), "a fila esta' sobreposta: {o:?}");
        }
        other => panic!("as duas colunas tem de ser escalares: {other:?}"),
    }
}

/// Um segundo `proximity` SUBSTITUI a resposta do primeiro — não a soma nem a mantém.
/// Duas medições com raios diferentes são duas perguntas, e a última é a que o artista
/// acabou de fazer.
#[test]
fn a_second_proximity_replaces_the_first_answer() {
    // O primeiro reporta 1/2/1; o segundo recebe esse stream e mede de novo.
    let input = Stream::new(3).with("P", Column::Vec2(row(3, 0.5)));
    let once = cook_stream(&input);
    let twice = cook_stream(&once);
    match (once.get(NEIGHBOURS_COL), twice.get(NEIGHBOURS_COL)) {
        (Some(Column::Scalar(a)), Some(Column::Scalar(b))) => {
            assert_eq!(a, b, "a segunda medicao acumulou sobre a primeira");
        }
        _ => panic!("as colunas tem de estar la'"),
    }
}
