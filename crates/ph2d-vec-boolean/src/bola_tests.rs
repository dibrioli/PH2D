//! Os gates da BOLA — ver o cabeçalho de [`super`].

use super::rola_a_bola;
use ph2d_vec_scene::{VecPath, VecVertex};

type P = [f64; 2];

fn canto(p: P) -> VecVertex {
    VecVertex::corner(p)
}

/// O entalhe da barra dobrada: um rectângulo com uma ponta CÔNCAVA em `(2, 0.3)` que vira `~166°`.
const PONTA: P = [2.0, 0.3];
const ENTALHE: [P; 7] = [
    [0.0, 0.0],
    [4.0, 0.0],
    [4.0, 3.0],
    [2.3, 3.0],
    PONTA,
    [1.7, 3.0],
    [0.0, 3.0],
];
const RAIO: f64 = 0.05;
const SOLDA: f64 = 0.1 * RAIO;

fn cantos(ps: &[P]) -> Vec<VecVertex> {
    ps.iter().map(|p| canto(*p)).collect()
}

/// Os nós do desenho com a viragem que tinham.
fn com_viragem(ps: &[P], vs: &[VecVertex]) -> Vec<(P, f64)> {
    ps.iter()
        .map(|p| {
            let i = vs.iter().position(|q| q.anchor == *p).expect("nó");
            (*p, crate::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0))
        })
        .collect()
}

fn area(vs: &[VecVertex]) -> f64 {
    crate::area(&VecPath {
        verts: vs.to_vec(),
        closed: true,
        ..VecPath::default()
    })
}

fn cubica(vs: &[VecVertex], i: usize) -> [P; 4] {
    let n = vs.len();
    let (c, q) = (&vs[i], &vs[(i + 1) % n]);
    [c.anchor, c.out_handle, q.in_handle, q.anchor]
}

fn avalia(c: &[P; 4], t: f64) -> (P, P, P) {
    let u = 1.0 - t;
    let p = |k: usize| {
        u * u * u * c[0][k]
            + 3.0 * u * u * t * c[1][k]
            + 3.0 * u * t * t * c[2][k]
            + t * t * t * c[3][k]
    };
    let d = |k: usize| {
        3.0 * u * u * (c[1][k] - c[0][k])
            + 6.0 * u * t * (c[2][k] - c[1][k])
            + 3.0 * t * t * (c[3][k] - c[2][k])
    };
    let dd = |k: usize| {
        6.0 * u * (c[2][k] - 2.0 * c[1][k] + c[0][k])
            + 6.0 * t * (c[3][k] - 2.0 * c[2][k] + c[1][k])
    };
    ([p(0), p(1)], [d(0), d(1)], [dd(0), dd(1)])
}

/// O menor raio de curvatura CÔNCAVA no interior dos segmentos (as pontas são vértices, e a
/// viragem deles lê-se à parte).
fn raio_concavo_minimo(vs: &[VecVertex]) -> f64 {
    let n = vs.len();
    let sinal: f64 = (0..n)
        .map(|i| {
            let (a, b) = (vs[i].anchor, vs[(i + 1) % n].anchor);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum::<f64>()
        .signum();
    let mut m = f64::INFINITY;
    for i in 0..n {
        let c = cubica(vs, i);
        for k in 1..64 {
            let (_, v, a) = avalia(&c, f64::from(k) / 64.0);
            let cruz = v[0] * a[1] - v[1] * a[0];
            let nv = v[0].hypot(v[1]);
            if nv > 1e-9 && cruz * sinal < 0.0 {
                m = m.min(nv * nv * nv / cruz.abs());
            }
        }
    }
    m
}

/// A maior viragem dos vértices que NÃO estavam na entrada (as quinas convexas do rectângulo
/// ficam, e não são o que se mede).
fn viragem_max(vs: &[VecVertex], entrada: &[P]) -> f64 {
    (0..vs.len())
        .filter(|&i| !entrada.contains(&vs[i].anchor))
        .map(|i| crate::overlap::viragem_do_vertice(vs, i).unwrap_or(0.0))
        .fold(0.0, f64::max)
}

/// ⭐⭐⭐ **GATE — o VINCO vira o ARCO da bola, e a quina do artista fica.** Um entalhe CÔNCAVO
/// cuja ponta não é quina do desenho sai sem quina nenhuma: os toques a `r·tan(α/2)` da ponta, todo
/// ponto novo a `r` do centro (e o meio de cada pedaço também), e a área só CRESCE. CONTROLOS: a
/// mesma ponta como quina do artista fica intacta, ao bit; uma ponta que o motor ENCAIXOU num nó
/// liso (viragem `0°` no desenho) arredonda igual; e uma ponta CONVEXA nunca é tocada.
#[test]
fn o_vinco_vira_o_arco_da_bola_e_a_quina_do_artista_fica() {
    let v = cantos(&ENTALHE);
    let fora: Vec<P> = ENTALHE.iter().copied().filter(|p| *p != PONTA).collect();
    let nos_fora = com_viragem(&fora, &v);
    let alfa = crate::overlap::viragem_do_vertice(&v, 4)
        .expect("tangentes")
        .to_radians();
    let s = rola_a_bola(v.clone(), &nos_fora, RAIO, SOLDA);
    assert!(
        s.len() > v.len(),
        "o vinco não foi trocado pelo arco: {s:?}"
    );
    assert!(
        viragem_max(&s, &ENTALHE) < 0.5,
        "ainda há uma quina: viragem máxima {}°",
        viragem_max(&s, &ENTALHE)
    );
    // Os toques: a `r·tan(α/2)` da ponta, e o centro a `r` deles, na bissectriz.
    let d = RAIO * (0.5 * alfa).tan();
    let novos: Vec<&VecVertex> = s.iter().filter(|p| !fora.contains(&p.anchor)).collect();
    let dist = |a: P, b: P| (a[0] - b[0]).hypot(a[1] - b[1]);
    let toques: Vec<P> = novos
        .iter()
        .map(|p| p.anchor)
        .filter(|a| (dist(*a, PONTA) - d).abs() < 1e-6)
        .collect();
    assert_eq!(
        toques.len(),
        2,
        "os toques não estão a r·tan(α/2) = {d} da ponta: {novos:?}"
    );
    let meio = [
        0.5 * (toques[0][0] + toques[1][0]),
        0.5 * (toques[0][1] + toques[1][1]),
    ];
    let sobre = [meio[0] - PONTA[0], meio[1] - PONTA[1]];
    let l = sobre[0].hypot(sobre[1]);
    let para_centro = RAIO / (0.5 * (std::f64::consts::PI - alfa)).sin();
    let centro = [
        PONTA[0] + sobre[0] / l * para_centro,
        PONTA[1] + sobre[1] / l * para_centro,
    ];
    let k = s
        .iter()
        .position(|p| !fora.contains(&p.anchor))
        .expect("o arco");
    let pedacos = novos.len() - 1;
    for q in 0..pedacos {
        let c = cubica(&s, k + q);
        for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let (p, _, _) = avalia(&c, t);
            let r = dist(p, centro);
            assert!(
                (r - RAIO).abs() < 1e-3 * RAIO,
                "o pedaço {q} do arco está a {r} do centro em t={t}, e o raio é {RAIO}"
            );
        }
    }
    assert!(area(&s) > area(&v), "a bola tirou área a um vinco côncavo");
    // CONTROLO: a mesma ponta como quina do ARTISTA fica intacta, ao bit.
    assert_eq!(
        rola_a_bola(v.clone(), &com_viragem(&ENTALHE, &v), RAIO, SOLDA),
        v,
        "uma quina que o artista desenhou foi arredondada"
    );
    // O ENCAIXE: a ponta é um nó que no desenho era LISO — vira mais do que virava, e arredonda.
    let mut encaixado = nos_fora.clone();
    encaixado.push((PONTA, 0.0));
    assert_eq!(
        rola_a_bola(v.clone(), &encaixado, RAIO, SOLDA),
        s,
        "um vinco que o motor encaixou num nó liso ficou em quina"
    );
    // CONTROLO: uma ponta CONVEXA nunca é tocada.
    let convexo = cantos(&[[0.0, 0.0], [2.0, 0.0], [0.12, 0.68], [-1.0, 1.0]]);
    assert_eq!(
        rola_a_bola(convexo.clone(), &[], RAIO, SOLDA),
        convexo,
        "uma ponta CONVEXA foi arredondada"
    );
}

/// ⭐⭐⭐ **GATE — uma CURVA mais apertada que a bola vira o arco dela; uma mais larga fica ao
/// bit.** É o que a F40 não fazia e o que as três fotos do dono mostravam: antes do contacto a
/// pele aperta até ao bico SEM vértice nenhum — uma curva lisa de raio `0,002 r`. A fixtura é o
/// entalhe arredondado a `ρ`; a bola de raio `r > ρ` alarga-o até `r`, e a de raio `r < ρ` não
/// lhe toca.
#[test]
fn uma_curva_mais_apertada_que_a_bola_vira_o_arco_dela() {
    let fora: Vec<P> = ENTALHE.iter().copied().filter(|p| *p != PONTA).collect();
    let v = cantos(&ENTALHE);
    let nos = com_viragem(&fora, &v);
    let rho = 0.2 * RAIO;
    let apertada = rola_a_bola(v, &nos, rho, 0.1 * rho);
    assert!(
        raio_concavo_minimo(&apertada) < 0.3 * RAIO,
        "a fixtura não é apertada: {}",
        raio_concavo_minimo(&apertada)
    );
    let s = rola_a_bola(apertada.clone(), &nos, RAIO, SOLDA);
    // ⚠️ `0,995` e não o limiar `0,99`: o arco da bola tem de ficar mais perto de `r` do que do
    // [`super::APERTO`], senão rolar outra vez lê-o à beira de apertado — a cúbica de um quarto de
    // círculo desce a `0,992 r` (um nada acima do limiar), a de `45°` fica em `0,9996`.
    assert!(
        raio_concavo_minimo(&s) > 0.995 * RAIO,
        "a curva continua mais apertada que a bola: {} contra {RAIO}",
        raio_concavo_minimo(&s)
    );
    let entrada: Vec<P> = apertada.iter().map(|v| v.anchor).collect();
    assert!(
        viragem_max(&s, &entrada) < 0.5,
        "a troca deixou uma quina: {}°",
        viragem_max(&s, &entrada)
    );
    assert!(area(&s) >= area(&apertada), "a bola tirou área");
    // CONTROLO: uma bola MENOR que a curva não lhe toca — ao bit.
    assert_eq!(
        rola_a_bola(apertada.clone(), &nos, 0.5 * rho, 0.05 * rho),
        apertada,
        "a bola mexeu numa curva mais larga que ela"
    );
    // E o fecho é IDEMPOTENTE: rolar outra vez não muda nada.
    assert_eq!(
        rola_a_bola(s.clone(), &nos, RAIO, SOLDA),
        s,
        "rolar a bola duas vezes não é o mesmo que uma"
    );
}

/// ⭐⭐ **GATE — os nós LISOS dentro do arco SAEM.** O assado põe nós a `~0,01` junto da junta, e o
/// toque cai além deles: se ficassem, o contorno iria ao toque, voltaria ao nó e seguiria — um
/// laço que nenhuma viragem acusa. A fixtura é o entalhe com um nó liso a meio de cada lado, DENTRO
/// do alcance do arco; a saída tem os MESMOS pontos do entalhe sem eles.
#[test]
fn os_nos_lisos_dentro_do_arco_saem() {
    let (esq, dir) = ([1.7, 3.0], [2.3, 3.0]);
    let a_meio = |a: P, t: f64| {
        [
            PONTA[0] + (a[0] - PONTA[0]) * t,
            PONTA[1] + (a[1] - PONTA[1]) * t,
        ]
    };
    let com_nos = [
        [0.0, 0.0],
        [4.0, 0.0],
        [4.0, 3.0],
        dir,
        a_meio(dir, 0.05),
        PONTA,
        a_meio(esq, 0.05),
        esq,
        [0.0, 3.0],
    ];
    let nos = |ps: &[P]| -> Vec<(P, f64)> {
        ps.iter()
            .copied()
            .filter(|p| *p != PONTA)
            .map(|p| (p, 0.0))
            .collect()
    };
    let a = rola_a_bola(cantos(&ENTALHE), &nos(&ENTALHE), RAIO, SOLDA);
    let b = rola_a_bola(cantos(&com_nos), &nos(&com_nos), RAIO, SOLDA);
    let alcance = a
        .iter()
        .map(|v| (v.anchor[0] - PONTA[0]).hypot(v.anchor[1] - PONTA[1]))
        .fold(f64::INFINITY, f64::min);
    let no = a_meio(dir, 0.05);
    assert!(
        (no[0] - PONTA[0]).hypot(no[1] - PONTA[1]) < alcance,
        "o nó a meio do lado está FORA do arco — a fixtura não o testa"
    );
    assert_eq!(b.len(), a.len(), "um nó liso dentro do arco ficou: {b:?}");
    for (x, y) in a.iter().zip(&b) {
        assert!(
            // ⚠️ `1e-7` e não `1e-9`: o centro sai do cruzamento das paralelas AMOSTRADAS, e as
            // amostras dos dois lados rectos não são as mesmas com e sem os nós — o ângulo estreito
            // do entalhe (`12,7°`) amplia o arredondamento. Medido: `1,5e-9` numa peça de `4`.
            (x.anchor[0] - y.anchor[0]).hypot(x.anchor[1] - y.anchor[1]) < 1e-7,
            "o arco muda com um nó liso dentro dele: {:?} contra {:?}",
            x.anchor,
            y.anchor
        );
    }
}

/// ⭐⭐ **GATE — o ruído do assado NÃO é tocado.** As micro-quinas de `1,4°` que o assado deixa
/// nos nós são côncavas e «mais apertadas que a bola» (raio zero) — e o toque delas fica a
/// `r·tan(0,7°) = 0,012 r`, dentro da solda. Trocá-las deixaria segmentos na solda por nada.
#[test]
fn o_ruido_do_assado_nao_e_tocado() {
    // Um quadrado com um lado partido numa micro-quina côncava de `1,4°`.
    let dy = 0.5 * (1.4_f64.to_radians()).tan();
    let v = cantos(&[
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.5, 1.0 - dy],
        [0.0, 1.0],
    ]);
    assert_eq!(rola_a_bola(v.clone(), &[], RAIO, SOLDA), v);
}

/// ⭐⭐⭐ **GATE — um FUNDO mais estreito que a bola é engolido INTEIRO, dente incluído.** Entalhes
/// de fundo plano, em W e com um DENTE convexo por dentro: cada canto do fundo semeia a própria
/// corrida, e a bola verdadeira pousa nos dois FLANCOS. ⛔ O dente foi o defeito que a sonda
/// achou (F41): o toque mais barato pousava num vale dele e a bola atravessava o outro lado —
/// dois arcos cruzados por cima do dente e uma meia-volta de `180°`. A lei que o cura é a bola
/// VAZIA; e os fundos que se partem em várias corridas exercitam a fusão e a poda dos vãos.
#[test]
fn um_fundo_mais_estreito_que_a_bola_e_engolido_inteiro() {
    let mut falhas = Vec::new();
    for (nome, fundo) in [
        ("plano 0.2r", vec![[1.99, 0.3], [2.01, 0.3]]),
        ("plano 0.6r", vec![[1.985, 0.3], [2.015, 0.3]]),
        ("plano 1.2r", vec![[1.97, 0.3], [2.03, 0.3]]),
        ("W 3", vec![[1.98, 0.32], [2.0, 0.3], [2.02, 0.32]]),
        (
            "W 4",
            vec![[1.97, 0.33], [1.99, 0.3], [2.01, 0.3], [2.03, 0.33]],
        ),
        (
            "W 5",
            vec![
                [1.96, 0.34],
                [1.98, 0.31],
                [2.0, 0.3],
                [2.02, 0.31],
                [2.04, 0.34],
            ],
        ),
        ("dente", vec![[1.98, 0.3], [2.0, 0.34], [2.02, 0.3]]),
    ] {
        let mut ps: Vec<P> = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [2.3, 3.0]];
        ps.extend(fundo.iter().rev());
        ps.extend([[1.7, 3.0], [0.0, 3.0]]);
        let v = cantos(&ps);
        let fora = [
            [0.0, 0.0],
            [4.0, 0.0],
            [4.0, 3.0],
            [2.3, 3.0],
            [1.7, 3.0],
            [0.0, 3.0],
        ];
        let nos = com_viragem(&fora, &v);
        let s = rola_a_bola(v.clone(), &nos, RAIO, SOLDA);
        let r = raio_concavo_minimo(&s);
        let q = viragem_max(&s, &fora);
        let idem = rola_a_bola(s.clone(), &nos, RAIO, SOLDA) == s;
        let engolido = s.iter().all(|q| !fundo.contains(&q.anchor));
        println!(
            "{nome}: n {} raio {:.4} r quina {q:.2} idem {idem} engolido {engolido}",
            s.len(),
            r / RAIO
        );
        if r < 0.99 * RAIO || q > 0.5 || !idem || !engolido {
            falhas.push(nome);
        }
    }
    assert!(falhas.is_empty(), "{falhas:?}");
}

/// O rectângulo `4 × 3` com uma FENDA de largura `largura` e fundo `fundo` a descer do meio do lado
/// de cima — a boca são dois nós CONVEXOS.
fn com_fenda(largura: f64, fundo: f64) -> (Vec<P>, [P; 4]) {
    let (e, d) = (2.0 - 0.5 * largura, 2.0 + 0.5 * largura);
    let fenda = [[d, 3.0], [d, 3.0 - fundo], [e, 3.0 - fundo], [e, 3.0]];
    let mut ps: Vec<P> = vec![[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]];
    ps.extend(fenda);
    ps.push([0.0, 3.0]);
    (ps, fenda)
}

/// ⭐⭐ **GATE — a fenda de boca mais estreita que a bola é FECHADA** (F46, o entalhe do braço
/// dobrado de volta, medido a `(176°, 100°)`). A bola pousa nos DOIS nós da boca, e o centro é onde
/// os dois ARCOS de raio `r` à volta deles se cruzam ([`super::toque::ArcoDoNo`]); a corda entre as
/// normais de um nó passa por DENTRO do arco, o centro saía perto demais da boca, a bola nunca
/// estava vazia, e a fenda ficava aberta até ao fundo.
///
/// ⚠️ O CONTROLO: uma fenda mais LARGA que a bola fica aberta (a bola cabe; só o fundo arredonda) —
/// senão o gate passaria com um fecho que enche tudo.
#[test]
fn uma_fenda_de_boca_estreita_e_fechada() {
    let fora = [[0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0]];
    let (ps, fenda) = com_fenda(0.6 * RAIO, 4.0 * RAIO);
    let v = cantos(&ps);
    let s = rola_a_bola(v.clone(), &com_viragem(&fora, &v), RAIO, SOLDA);
    assert!(
        // A boca são os TOQUES da bola (ficam); o fundo é o que ela engole.
        s.iter()
            .all(|q| q.anchor != fenda[1] && q.anchor != fenda[2]),
        "a fenda estreita ficou aberta"
    );
    assert!(
        area(&s) > area(&v) + 0.9 * 0.6 * RAIO * 4.0 * RAIO,
        "a fenda não foi cheia: área {} contra {}",
        area(&s),
        area(&v)
    );
    let (ps, larga) = com_fenda(3.0 * RAIO, 4.0 * RAIO);
    let v = cantos(&ps);
    let s = rola_a_bola(v.clone(), &com_viragem(&fora, &v), RAIO, SOLDA);
    assert!(
        s.iter().any(|q| q.anchor == larga[0]) && s.iter().any(|q| q.anchor == larga[3]),
        "a boca da fenda larga saiu — o fecho enche o que a bola alcança"
    );
}
