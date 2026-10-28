//! ⭐ **O posto esparso (Householder) dá a MESMA resposta que o denso (Gram–Schmidt, a referência)** — W6 ponta 3
//! (`docs/BlenderConstraints/07_W6_pontas.md` §2): a ordem de nascimento inteira em toda cena da W4 (na pose do
//! desenho e na resolvida), e `a_mais` de regras sorteadas (semente fixa) nas cenas de 50 e 200 pontos.

#[allow(dead_code, reason = "o carregador é o dos gates do solver")]
#[path = "../tests/it/cena.rs"]
pub(crate) mod cena;

use ph2d_vec_scene::cad::{Lei, PontoCad, RegraCad, SegCad};

use super::posto::{Base, Posto, PostoDenso};
use super::{Folha, Sistema, a_mais, a_mais_com, posto, redundantes, redundantes_com, resolve};
use cena::Cena;

/// As cenas cujo denso cabe num teste `dev` (as de 800 pontos custam minutos a `opt-level 0`).
const CENAS: [&str; 18] = [
    "bem_composto__medida",
    "bem_composto__partida",
    "bem_perfil__medida",
    "bem_perfil__partida",
    "bem_retangulo__medida",
    "bem_retangulo__partida",
    "bem_rodado__medida",
    "bem_rodado__partida",
    "bem_simetrico__partida",
    "bem_tangente__medida",
    "bem_tangente__partida",
    "conflito_hv",
    "conflito_inconsistente",
    "conflito_redundante",
    "escada_202",
    "escada_50",
    "misto_200",
    "misto_50",
];

/// A folga entre as duas distâncias (linha normalizada): o erro de arredondamento de ambas fica abaixo de `1e-13`
/// nestas cenas; um reflexo esquecido desloca-a muito acima disto antes de virar um veredito.
const FOLGA: f64 = 1e-11;

fn compara(folha: &Folha<'_>, regras: &[RegraCad], onde: &str) -> usize {
    let s = Sistema::monta(folha, regras, None, None);
    let esparso = redundantes(&s, regras, &s.valores);
    let denso = redundantes_com(&s, regras, &s.valores, PostoDenso::new(s.valores.len()));
    assert_eq!(esparso, denso, "{onde}: as redundantes diferem");
    // Linha a linha, a MESMA distância.
    let f = &s.valores;
    let (mut a, mut b) = (Posto::new(f.len()), PostoDenso::new(f.len()));
    for &i in &s.ativas {
        let (vars, res) = posto::residuos(&regras[i].lei, &|v| f[s.global(v)]);
        let ls = posto::linhas(&vars, &res, &|v| s.global(v));
        for (k, l) in ls.iter().enumerate() {
            let (sa, sb) = (a.aceita(vec![l.clone()]), b.aceita(vec![l.clone()]));
            let (da, db) = (a.ultima(), b.ultima());
            assert!(
                sa == sb && (da - db).abs() <= FOLGA,
                "{onde}, regra {i} linha {k}: esparso {da:e} × denso {db:e}"
            );
        }
    }
    esparso.len()
}

#[test]
fn the_sparse_rank_gives_the_dense_answer_on_every_fixture_scene() {
    let mut redundantes_vistas = 0;
    for nome in CENAS {
        let c = cena::cena(nome);
        redundantes_vistas += compara(&c.folha(), &c.regras, nome);
        let r = resolve(&c.folha(), &c.regras, None, None);
        let paths = c.na_pose(&c.posicoes(&r.solucao));
        let folha = Folha::new(paths.iter().map(|p| (p, ph2d_vec_scene::Xform::IDENTITY)));
        redundantes_vistas += compara(&folha, &c.regras, &format!("{nome} resolvida"));
    }
    assert!(redundantes_vistas > 0, "a régua viu regras a mais");
}

/// Gerador congruente de semente fixa.
struct Sorteio(u64);

impl Sorteio {
    fn ate(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).expect("31 bits") % n
    }
}

fn sorteia(c: &Cena, g: &mut Sorteio) -> Lei {
    let mut p = || c.ponto[g.ate(c.ponto.len())];
    let (a, b, q, d) = (p(), p(), p(), p());
    let s = SegCad(a, b);
    let t = SegCad(q, d);
    let circulos: Vec<_> = c
        .ponto
        .iter()
        .filter_map(|p| match *p {
            PontoCad::Centro(id) => Some(id),
            PontoCad::Vert(..) => None,
        })
        .collect();
    match g.ate(if circulos.is_empty() { 11 } else { 13 }) {
        0 => Lei::Horizontal(s),
        1 => Lei::Vertical(s),
        2 => Lei::Coincidente(a, b),
        3 => Lei::Distancia(a, b, 10.0),
        4 => Lei::Paralelo(s, t),
        5 => Lei::Perpendicular(s, t),
        6 => Lei::Igual(s, t),
        7 => Lei::SobreRecta(q, s),
        8 => Lei::Simetrico(q, d, s),
        9 => Lei::Angulo(s, t, 0.5),
        10 => Lei::Fixo(a, [0.0, 0.0]),
        11 => Lei::Tangente(s, circulos[g.ate(circulos.len())]),
        _ => Lei::Raio(circulos[g.ate(circulos.len())], 5.0),
    }
}

#[test]
fn the_sparse_rank_gives_the_dense_answer_for_random_new_rules() {
    let mut g = Sorteio(0x5eed_0003);
    let (mut sim, mut nao) = (0, 0);
    for (nome, sorteadas, copias) in [
        ("escada_50", 40, 12),
        ("misto_50", 40, 12),
        ("escada_202", 12, 4),
        ("misto_200", 12, 4),
    ] {
        let c = cena::cena(nome);
        let folha = c.folha();
        let mut veredito = |regras: &[RegraCad], nova: RegraCad, k: &str| {
            let esparso = a_mais(&folha, regras, nova);
            let denso = a_mais_com(&folha, regras, nova, PostoDenso::new);
            assert_eq!(esparso, denso, "{nome} {k}: {:?}", nova.lei);
            if esparso { sim += 1 } else { nao += 1 }
        };
        for k in 0..sorteadas {
            let nova = RegraCad::nova(sorteia(&c, &mut g));
            veredito(&c.regras, nova, &format!("sorteada {k}"));
        }
        // Uma regra que já está lá (a mais) e a mesma tirada do conjunto (sobe o posto, se não era redundante).
        for _ in 0..copias {
            let k = g.ate(c.regras.len());
            veredito(&c.regras, c.regras[k], &format!("cópia da {k}"));
            let sem: Vec<_> = c
                .regras
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != k)
                .map(|(_, r)| *r)
                .collect();
            veredito(&sem, c.regras[k], &format!("sem a {k}"));
        }
    }
    assert!(
        sim > 0 && nao > 0,
        "a régua viu os dois vereditos: {sim} a mais, {nao} não"
    );
}
