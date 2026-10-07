//! ⭐⭐ **O GANCHO MICROSCÓPICO** — um nó em que a direcção do contorno se INVERTE sem que a forma
//! recue nada que se veja (F43, report do dono de 2026-09-30, com foto: *«quase perfeito, artefatos
//! curados na quina dobrada; resquício quando quase reto»*).
//!
//! # O que é, medido
//!
//! Na dobra do MAPA a velocidade do contorno deformado chega a zero (`det J = 0`), e o assado pousa
//! um nó ali: a cúbica seguinte nasce com a primeira alça EM CIMA do nó e a segunda `0,006` ATRÁS
//! dele (cena `=4` a `(125°, 34°)`). A curva recua uma fracção de micrómetro e volta — invisível como
//! forma —, mas a TANGENTE inverte-se `180°`, e o traçador desenha a inversão como uma meia-lua com
//! fatias em falta. ⛔ A bola não lhe toca: o gancho não é um vinco côncavo, é uma inversão num lado
//! que é quase recto (a junta de cima a `32°`–`36°`).
//!
//! # A cura
//!
//! Uma cúbica que DOBRA — a tangente inverte-se num nó, ou as alças cruzam-se e ela faz um laço por
//! dentro (medido a `(110°, 17,5°)`: a 1.ª alça passa à frente do fim e a 2.ª fica atrás do início) —
//! é trocada pela Hermite que segue as tangentes dos vizinhos, e a troca só se faz se a forma nova
//! ficar a menos de `tol` da velha (distância de Hausdorff). ⇒ um recuo que se vê fica; uma inversão
//! que só existe na tangente sai. ⚠️ Antes da UNIÃO (e portanto da bola): a união lê o zigue-zague
//! como cruzamento e reescreve-o num dardo real, e a bola corta cúbicas nos pés do arco — uma cúbica
//! com laço cortada a meio entrega a inversão num nó NOVO.

use ph2d_vec_scene::VecVertex;

/// A viragem a partir da qual um passo é uma INVERSÃO — `150°`. As medidas são todas `179,7°`–`180°`
/// (a tangente volta para trás); entre passos de uma cúbica amostrada `32×` uma curva lisa vira poucos
/// graus. ⚠️ Não `90°`: um canto RECTO de uma caixa não é um gancho, e a `90°` exactos o co-seno
/// arredondado lia-o como tal (o gate da caixa reprovou assim).
pub const VIRAGEM_DO_GANCHO: f64 = 150.0;

/// Amostras por cúbica na comparação da forma velha com a nova.
const AMOSTRAS: u32 = 32;

/// ⭐ **Até onde um recuo SOBRE O PRÓPRIO CAMINHO sai** — em soldas: `RAIO_DO_VINCO / SOLDA_DA_QUINA`,
/// o tamanho da bola (F45). Medido a `(166°, 74°)`: a união deixa uma cúbica que passa do nó
/// `0,0059` e volta pela MESMA recta — `1,4×` a solda, logo a régua da cúbica sozinha recusava a
/// troca; e o pedaço a mais fica EM CIMA do resto do contorno (a `0,0003` dele), logo como forma é
/// invisível e só o traço o desenha, como meia-lua. ⇒ a troca também se faz quando o CONTORNO
/// inteiro mudar menos que a solda, desde que a cúbica não mude mais que a bola: um recuo maior
/// que a bola é uma feição que a própria bola deixaria ficar, e não se mexe (o gate do recuo de
/// meio lado continua de pé).
const RECUO_SOBRE_SI: f64 = crate::overlap::RAIO_DO_VINCO / crate::overlap::SOLDA_DA_QUINA;

type P = [f64; 2];

fn ponto(c: &[P; 4], t: f64) -> P {
    let u = 1.0 - t;
    let f = |k: usize| {
        u * u * u * c[0][k]
            + 3.0 * u * u * t * c[1][k]
            + 3.0 * u * t * t * c[2][k]
            + t * t * t * c[3][k]
    };
    [f(0), f(1)]
}

fn amostras(c: &[P; 4]) -> Vec<P> {
    (0..=AMOSTRAS)
        .map(|k| ponto(c, f64::from(k) / f64::from(AMOSTRAS)))
        .collect()
}

/// A distância de um ponto a um segmento.
fn ao_segmento(p: P, a: P, b: P) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 {
        (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (p[0] - a[0] - t * dx).hypot(p[1] - a[1] - t * dy)
}

/// A distância de Hausdorff entre duas polilinhas densas — cada amostra contra os SEGMENTOS da
/// outra, nos dois sentidos. ⚠️ Ponto a ponto ela lia a distância entre amostras de ritmo diferente
/// (`~0,008` numa recta amostrada `32×`), e o gancho nunca saía.
fn hausdorff(a: &[P], b: &[P]) -> f64 {
    let lado = |x: &[P], y: &[P]| {
        x.iter()
            .map(|p| {
                y.windows(2)
                    .map(|s| ao_segmento(*p, s[0], s[1]))
                    .fold(f64::INFINITY, f64::min)
            })
            .fold(0.0, f64::max)
    };
    lado(a, b).max(lado(b, a))
}

/// A distância de `p` ao conjunto de polilinhas `resto` mais a polilinha `c`.
fn ao_contorno(p: P, c: &[P], resto: &[Vec<P>]) -> f64 {
    std::iter::once(c)
        .chain(resto.iter().map(Vec::as_slice))
        .flat_map(|y| y.windows(2).map(move |s| ao_segmento(p, s[0], s[1])))
        .fold(f64::INFINITY, f64::min)
}

/// ⭐ **Quanto muda o CONTORNO** ao trocar a cúbica `velha` pela `nova`, com o `resto` dele parado:
/// cada amostra de uma contra a outra MAIS o resto, nos dois sentidos. Um pedaço que só refaz o
/// caminho de outro pedaço não muda o conjunto de pontos desenhado ([`RECUO_SOBRE_SI`]).
fn hausdorff_no_contorno(velha: &[P], nova: &[P], resto: &[Vec<P>]) -> f64 {
    let lado = |x: &[P], y: &[P]| {
        x.iter()
            .map(|p| ao_contorno(*p, y, resto))
            .fold(0.0, f64::max)
    };
    lado(velha, nova).max(lado(nova, velha))
}

/// Quantas vezes a polilinha `x` cruza as do `resto` — só cruzamentos PRÓPRIOS (no interior dos
/// dois segmentos): o toque nos nós partilhados com os vizinhos não conta.
fn cruzamentos(x: &[P], resto: &[Vec<P>]) -> usize {
    let cruza = |a: P, b: P, c: P, d: P| {
        let (r, s) = ([b[0] - a[0], b[1] - a[1]], [d[0] - c[0], d[1] - c[1]]);
        let den = r[0] * s[1] - r[1] * s[0];
        if den.abs() < 1e-300 {
            return false;
        }
        let q = [c[0] - a[0], c[1] - a[1]];
        let (u, w) = (
            (q[0] * s[1] - q[1] * s[0]) / den,
            (q[0] * r[1] - q[1] * r[0]) / den,
        );
        const E: f64 = 1e-9;
        u > E && u < 1.0 - E && w > E && w < 1.0 - E
    };
    x.windows(2)
        .map(|a| {
            resto
                .iter()
                .flat_map(|y| y.windows(2))
                .filter(|b| cruza(a[0], a[1], b[0], b[1]))
                .count()
        })
        .sum()
}

/// A direcção de `a` para `b` — ou `None` se a distância for ruído que a PLACA não vê.
///
/// ⛔ **O limiar é a precisão do `f32` À ESCALA das coordenadas, nunca um epsilon absoluto:** quem
/// desenha o traço é a placa, em `f32`, e uma alça a `1,2e-11` do nó (medido no assado, a `45°`, num
/// desenho sem contacto) é o MESMO ponto para ela — o traçador usa a alça seguinte. Com o `1e-12` de
/// antes, essa distância de arredondamento lia-se como uma tangente a apontar ao contrário, o passe
/// «curava» um gancho que não existe e o desenho mudava fora do contacto
/// (`numa_dobra_forte_o_desenho_nao_se_cruza`). O recuo real da F43-bis mede `8,8e-4`.
fn dir(a: P, b: P) -> Option<P> {
    let (x, y) = (b[0] - a[0], b[1] - a[1]);
    let l = x.hypot(y);
    let escala = a[0]
        .abs()
        .max(a[1].abs())
        .max(b[0].abs())
        .max(b[1].abs())
        .max(1.0);
    (l > escala * f64::from(f32::EPSILON)).then(|| [x / l, y / l])
}

/// A Hermite de `p0` a `p3` com as tangentes unitárias `t0` (a sair) e `t3` (a chegar).
fn hermite(p0: P, t0: P, t3: P, p3: P) -> [P; 4] {
    let l = (p3[0] - p0[0]).hypot(p3[1] - p0[1]) / 3.0;
    [
        p0,
        [p0[0] + t0[0] * l, p0[1] + t0[1] * l],
        [p3[0] - t3[0] * l, p3[1] - t3[1] * l],
        p3,
    ]
}

/// A direcção de saída da cúbica `c` no início, pelos pontos de controlo.
fn inicio(c: &[P; 4]) -> Option<P> {
    dir(c[0], c[1])
        .or_else(|| dir(c[0], c[2]))
        .or_else(|| dir(c[0], c[3]))
}

/// A direcção de chegada da cúbica `c` ao fim, pelos pontos de controlo.
fn fim(c: &[P; 4]) -> Option<P> {
    dir(c[2], c[3])
        .or_else(|| dir(c[1], c[3]))
        .or_else(|| dir(c[0], c[3]))
}

/// **A cúbica DOBRA?** — a sequência «tangente de chegada ao nó de partida · a tangente EXACTA de
/// partida · os passos da cúbica amostrada · a tangente EXACTA de chegada · tangente de saída do nó
/// de chegada» vira mais que [`VIRAGEM_DO_GANCHO`] em algum par seguido. `None` numa ponta = essa
/// junção não conta (é quina do artista).
///
/// ⛔⛔ **As tangentes exactas das pontas não são redundantes com as amostras** (F43-bis, report do
/// dono de 2026-09-30, foto com a junta de cima a `10,5°` NO MESMO SENTIDO da de baixo): com a 2.ª
/// alça EM CIMA do nó e a 1.ª `0,0009` ALÉM dele, a cúbica recua só no último `1,4 %` do parâmetro
/// (`s < 0,0136`) — `32` amostras saltam-no, os passos leem-se todos no mesmo sentido, e o traço
/// desenha a meia-lua sobre a tangente de chegada, que aponta ao CONTRÁRIO. Medido a `(84°, −10,5°)`.
fn dobra(antes: Option<P>, c: &[P; 4], depois: Option<P>) -> bool {
    let pts = amostras(c);
    let passos = pts.windows(2).filter_map(|w| dir(w[0], w[1]));
    let seq: Vec<P> = antes
        .into_iter()
        .chain(inicio(c))
        .chain(passos)
        .chain(fim(c))
        .chain(depois)
        .collect();
    let limiar = VIRAGEM_DO_GANCHO.to_radians().cos();
    seq.windows(2)
        .any(|w| w[0][0] * w[1][0] + w[0][1] * w[1][1] < limiar)
}

/// ⭐⭐ **Desfaz os ganchos microscópicos do contorno fechado `verts`** — ver o módulo. `quinas` são as
/// quinas do artista (as mesmas da bola, [`crate::bola::quina_do_artista`]); `tol` é a escala abaixo
/// da qual uma diferença de forma é ruído (a solda). Nada a desfazer ⇒ `verts` intacto, ao bit.
///
/// Por segmento: se a cúbica DOBRA — num nó (a inversão da tangente) ou por dentro (um laço das
/// alças cruzadas) —, ela é trocada pela Hermite que segue as tangentes dos vizinhos (ou as dela, onde
/// a ponta é quina do artista); de entre as candidatas que não dobram, a mais perto da velha, e só se
/// essa ficar a menos de `tol`.
#[must_use]
pub fn desfaz_os_ganchos(
    mut verts: Vec<VecVertex>,
    quinas: &[([f64; 2], f64)],
    tol: f64,
) -> Vec<VecVertex> {
    let n = verts.len();
    if n < 3 || !tol.is_finite() || tol <= 0.0 {
        return verts;
    }
    for k in 0..n {
        let kb = (k + 1) % n;
        let c = [
            verts[k].anchor,
            verts[k].out_handle,
            verts[kb].in_handle,
            verts[kb].anchor,
        ];
        let livre = |i: usize, v: &[VecVertex]| !crate::bola::quina_do_artista(v, i, quinas);
        let antes = livre(k, &verts)
            .then(|| crate::overlap::tangentes_do_vertice(&verts, k).map(|t| t.0))
            .flatten();
        let depois = livre(kb, &verts)
            .then(|| crate::overlap::tangentes_do_vertice(&verts, kb).map(|t| t.1))
            .flatten();
        if !dobra(antes, &c, depois) {
            continue;
        }
        let velha = amostras(&c);
        let resto: Vec<Vec<P>> = (0..n)
            .filter(|&j| j != k)
            .map(|j| {
                let jb = (j + 1) % n;
                amostras(&[
                    verts[j].anchor,
                    verts[j].out_handle,
                    verts[jb].in_handle,
                    verts[jb].anchor,
                ])
            })
            .collect();
        let cruzava = cruzamentos(&velha, &resto);
        let partidas = [antes, inicio(&c)];
        let chegadas = [fim(&c), depois];
        let melhor = partidas
            .iter()
            .flatten()
            .flat_map(|&t0| chegadas.iter().flatten().map(move |&t3| (t0, t3)))
            .map(|(t0, t3)| hermite(c[0], t0, t3, c[3]))
            .filter(|h| !dobra(antes, h, depois))
            // ⭐ A13: desfazer um gancho nunca acrescenta um cruzamento. Medido no braço a
            // `(160°, 134°)`: a Hermite que parte pela tangente de chegada dá a volta na ponta de um
            // esporão de `0,034` e corta o flanco de ida — um laço de área zero que nem o esporão
            // (flancos a `1,8` soldas) nem a bola (nenhum par de paralelas se cruza) tiram: `178°`.
            .filter(|h| cruzamentos(&amostras(h), &resto) <= cruzava)
            .map(|h| (hausdorff(&velha, &amostras(&h)), h))
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let aceita = |d: f64, h: &[P; 4]| {
            if d <= tol {
                return true;
            }
            if d > RECUO_SOBRE_SI * tol {
                return false;
            }
            hausdorff_no_contorno(&velha, &amostras(h), &resto) <= tol
        };
        if let Some((d, h)) = melhor
            && aceita(d, &h)
        {
            verts[k].out_handle = h[1];
            verts[kb].in_handle = h[2];
        }
    }
    verts
}

#[cfg(test)]
#[path = "gancho_tests.rs"]
mod tests;
