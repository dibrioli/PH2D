//! ⭐⭐⭐ **A SONDA QUE DECIDE A ROTA DA 2.ª SAÍDA** — *deformar a forma por uma MALHA* (ordem do dono,
//! 2026-09-19: *«siga para o (2)»*).
//!
//! # ⛔⛔⛔ Ela existe porque a NOTA que descreve essa saída já foi medida e estava ERRADA
//!
//! A F26 escreveu: *«o preço NÃO é “a `ph2d-poly2d` já existe”: ela parte de uma **grelha de ALFA**,
//! logo a forma teria de ser RASTERIZADA»*. ⚠️ **Falso, medido:** a
//! [`ph2d_poly2d::triangulate`] recebe um **anel de pontos** e a [`ph2d_poly2d::grid_mesh_of`]
//! também — a grelha de alfa é **uma** das entradas ([`ph2d_poly2d::mesh_of`]), não a única. É a
//! terceira nota minha que a medição derruba nesta jornada.
//!
//! # ⭐⭐⭐ E ao medir apareceu uma rota MELHOR, que não precisa de malha nenhuma
//!
//! A [`ph2d_vec_envelope`] já sabe deformar geometria **Bézier** por um mapa **não-afim** — e o
//! cabeçalho dela avisa, por escrito, contra exactamente o que a pele faz hoje:
//!
//! > *«Só transformações afins comutam com a avaliação de Bézier … isto está errado:
//! > `for v in verts { v.anchor = warp(v.anchor) }` … a curva resultante não é a imagem da curva
//! > original … ela acerta em `t=0` e `t=1` exactamente, e no interior nunca.»*
//!
//! ⇒ **É a descrição exacta do defeito que o dono viu na F28** (o desenho a saltar ao ganhar um
//! ponto), e a razão de a arte não responder a peso pintado ENTRE os nós: *hoje só os nós são
//! amostrados*.
//!
//! # As TRÊS perguntas desta sonda, escritas antes de correr
//!
//! 1. **Peso pintado ENTRE dois nós move a arte?** Hoje: não. Pela rota do warp: ?
//! 2. **O fit CONVERGE?** O `Warp` exige a jacobiana REAL, e a doc dele mede que uma inconsistente
//!    faz o `fit_to_bezpath` **não convergir** — ela falha ALTO, o que é bom saber de antemão.
//! 3. **Quanto CUSTA?** O `recook` corre uma vez por quadro, por forma presa.

use super::*;
use ph2d_skeleton::{SkinBone, Xform};
use ph2d_vec_scene::{ShapeKind, VecPath, cook};

/// Um osso deitado no `+X`, de `(x0,0)` a `(x0+len,0)`, com a pose `d` aplicada.
fn osso(x0: f64, len: f64, d: [f64; 2], rot: f64) -> SkinBone {
    let (c, s) = (rot.cos(), rot.sin());
    SkinBone::new(
        Xform([1.0, 0.0, 0.0, 1.0, x0, 0.0]),
        len,
        1.0,
        Xform([c, s, -s, c, x0 + d[0], d[1]]),
        Xform::IDENTITY,
    )
    .expect("repouso nao-singular")
}

/// A pele do palco: dois ossos ao longo de um rectângulo de `40 × 10`, o segundo dobrado.
fn pele() -> Skin {
    Skin::new(vec![
        osso(0.0, 20.0, [0.0, 0.0], 0.0),
        osso(20.0, 20.0, [0.0, 0.0], 0.8),
    ])
    .expect("2 ossos")
}

/// ⭐ **A PELE COMO UM MAPA** — o que a [`ph2d_vec_envelope::Warp`] pede.
///
/// ⚠️ A jacobiana é por **diferença finita**, e o doc do trait diz que isso troca uma derivada exacta
/// por um epsilon inventado. *É de propósito: esta sonda mede se a ROTA existe, não a lei final.* Se
/// ela convergir com a diferença finita, a lei fechada é trabalho conhecido; se não convergir, a
/// rota morre aqui e a malha volta à mesa.
struct PeleWarp<'a> {
    pele: &'a Skin,
    correcoes: Vec<ph2d_skeleton::Correccao>,
}

impl PeleWarp<'_> {
    fn ponto(&self, p: [f64; 2]) -> [f64; 2] {
        let mut w = self.pele.scratch();
        self.pele
            .weights_corrected(p, None, &mut w, &self.correcoes);
        self.pele.blend(p, &w)
    }
}

impl ph2d_vec_envelope::Warp for PeleWarp<'_> {
    fn map(&self, p: [f64; 2]) -> [f64; 2] {
        self.ponto(p)
    }
    fn jacobian(&self, p: [f64; 2]) -> [[f64; 2]; 2] {
        const H: f64 = 1e-5;
        let dx = self.ponto([p[0] + H, p[1]]);
        let mx = self.ponto([p[0] - H, p[1]]);
        let dy = self.ponto([p[0], p[1] + H]);
        let my = self.ponto([p[0], p[1] - H]);
        [
            [(dx[0] - mx[0]) / (2.0 * H), (dy[0] - my[0]) / (2.0 * H)],
            [(dx[1] - mx[1]) / (2.0 * H), (dy[1] - my[1]) / (2.0 * H)],
        ]
    }
}

/// A arte do palco: um rectângulo deitado, os quatro nós nas quinas.
fn forma() -> VecPath {
    cook(ShapeKind::Rectangle, [0.0, 0.0], [40.0, 10.0], &[])
}

/// A curva desenhada, amostrada densamente — o que o olho vê.
fn polilinha(p: &VecPath) -> Vec<[f64; 2]> {
    const N: usize = 200;
    let cozido = p.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, fechado)) = cozido.contour(c) else {
            continue;
        };
        let n = verts.len();
        let ultimo = if fechado { n } else { n.saturating_sub(1) };
        for i in 0..ultimo {
            let (a, b) = (&verts[i], &verts[(i + 1) % n]);
            for k in 0..N {
                let t = k as f64 / N as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w0 * a.anchor[0]
                        + w1 * a.out_handle[0]
                        + w2 * b.in_handle[0]
                        + w3 * b.anchor[0],
                    w0 * a.anchor[1]
                        + w1 * a.out_handle[1]
                        + w2 * b.in_handle[1]
                        + w3 * b.anchor[1],
                ]);
            }
        }
    }
    out
}

/// O quanto `b` se afasta de `a`, em unidades de mundo.
fn desvio(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    b.iter()
        .map(|p| ph2d_skeleton::dist2_to_polyline(*p, a).sqrt())
        .fold(0.0_f64, f64::max)
}

/// ⭐⭐⭐ **AS TRÊS RESPOSTAS, numa corrida.** Ela IMPRIME e julga o mínimo: é uma sonda, e o que
/// decide a rota é o número, não um veredito escrito de antemão.
#[test]
fn a_pele_como_warp_responde_as_tres_perguntas() {
    let k = pele();
    // Uma MANCHA no meio da aresta de baixo — entre os nós `(0,0)` e `(40,0)`, onde hoje não há
    // peso nenhum para corrigir. É o sítio exacto da pergunta do dono.
    let mancha = ph2d_skeleton::Correccao {
        tendon: 0,
        centro: [20.0, 0.0],
        raio: 14.0,
        especie: ph2d_skeleton::Especie::Soma(0.6),
    };

    // (a) HOJE: os pontos de controlo, com e sem a mancha.
    let mut hoje_sem = forma();
    apply(&k, &mut hoje_sem);
    let mut hoje_com = forma();
    aplica_corrigido(&k, &mut hoje_com, &[], &[mancha]);
    let hoje = desvio(&polilinha(&hoje_sem), &polilinha(&hoje_com));

    // (b) PELA ROTA DO WARP: a curva inteira amostrada e refitada.
    let sem = ph2d_vec_envelope::warp_path(
        &forma(),
        &PeleWarp {
            pele: &k,
            correcoes: Vec::new(),
        },
        0.05,
    );
    // ⚠️ **O MÍNIMO de cinco corridas**, e não uma: o relógio desta máquina não vale nada sob carga,
    // e o mínimo é o que mais se aproxima do trabalho sem contenção.
    let mut relogio = std::time::Duration::MAX;
    let mut com = forma();
    for _ in 0..5 {
        let t0 = std::time::Instant::now();
        com = ph2d_vec_envelope::warp_path(
            &forma(),
            &PeleWarp {
                pele: &k,
                correcoes: vec![mancha],
            },
            0.05,
        );
        relogio = relogio.min(t0.elapsed());
    }
    let warp = desvio(&polilinha(&sem), &polilinha(&com));

    eprintln!(
        "[sonda-warp] uma mancha ENTRE dois nos move a arte: hoje={hoje:.6} · pela rota do \
         WARP={warp:.6} unidades de mundo"
    );
    eprintln!(
        "[sonda-warp] custo de UM warp_path (4 nos, accuracy 0,05): {:.3} ms · nos da saida: {} \
         (a fonte tem {})",
        relogio.as_secs_f64() * 1e3,
        com.verts_all().count(),
        forma().verts_all().count()
    );

    // ⛔ A única coisa que esta sonda JULGA é que ela mediu alguma coisa — o resto é o número.
    assert!(
        !polilinha(&com).is_empty(),
        "o warp devolveu uma forma VAZIA: a rota morre aqui e a malha volta a' mesa"
    );
}
