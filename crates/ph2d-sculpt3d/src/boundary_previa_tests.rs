//! Os gates do indicador do contorno — ver [`super`].

use super::TrechoDaBorda;
use crate::{Brush, Dab, SculptStroke, Symmetry, Verb};

fn tigela() -> ph2d_mesh::Mesh {
    let cheia = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    let pos = cheia.positions().to_vec();
    let escolhidas: Vec<ph2d_mesh::Face> = cheia
        .faces()
        .iter()
        .filter(|f| f.verts().iter().all(|&v| pos[v as usize][1] <= 0.0))
        .copied()
        .collect();
    let (p, f, _) = ph2d_mesh::compact_for_faces(&pos, &escolhidas);
    ph2d_mesh::Mesh::from_parts(p, f).expect("a tigela é construída aqui")
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Boundary,
        radius: 0.3,
        strength: 1.0,
        ..Brush::default()
    }
}

/// O ponto mais alto — a boca da tigela, que é onde o cursor da cena `=42` cai.
fn na_boca(malha: &ph2d_mesh::Mesh) -> [f32; 3] {
    malha
        .positions()
        .iter()
        .copied()
        .max_by(|a, c| a[1].total_cmp(&c[1]))
        .expect("a malha tem vértices")
}

/// ⭐⭐ **A FIGURA DESCREVE A BORDA, e o anel do cursor não podia.**
///
/// Ela tem de ser um laço fechado sobre a boca — tantos pedaços quantos vértices
/// de cadeia — e a linha da profundidade tem de entrar na peça.
#[test]
fn o_indicador_desenha_a_boca_e_a_profundidade() {
    let malha = tigela();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let p = s.boundary_contorno(&malha, &b, Symmetry::default(), na_boca(&malha));

    assert!(
        p.borda().len() >= 40,
        "a boca desta tigela tem 48 vértices de borda e a cadeia inteira entra \
         com `Constant`: {} pedaços",
        p.borda().len()
    );
    // ⭐ **FECHADO:** a boca é um laço, e o passeio devolve a cadeia aberta —
    // sem o pedaço de fecho falta sempre um vão, do lado oposto ao cursor.
    let n = p.borda().len();
    let fecha = p.borda()[n - 1].b == p.borda()[0].a;
    assert!(fecha, "a cadeia é um laço e o indicador tem de o fechar");

    let [a, o] = p.profundidade().expect("há borda ao alcance");
    let fundura = ((a[0] - o[0]).powi(2) + (a[1] - o[1]).powi(2) + (a[2] - o[2]).powi(2)).sqrt();
    assert!(
        fundura > 0.1,
        "a linha da profundidade tem de entrar na peça: {fundura:.4}"
    );
    // ⚠️ Ela entra PARA BAIXO nesta peça (a boca está em cima), e é isso que o
    // dono vê: o eixo mergulha na tigela.
    assert!(o[1] < a[1], "o ponto-origem é mais fundo que a âncora");
}

/// ⭐⭐⭐ **O `Falloff along the edge` MUDA a figura** — é a razão de ela existir.
///
/// ⚠️ A régua é o **peso**, e não a contagem de pedaços: a cadeia é a mesma nas
/// duas configurações (as fases A–D não leem este selector), e o que muda é
/// quanto de cada pedaço entra. *Uma régua que contasse pedaços leria as duas
/// iguais e o gate ficaria verde sobre um indicador cego.*
#[test]
fn a_queda_ao_longo_da_borda_muda_o_que_a_figura_mostra() {
    let malha = tigela();
    let soma = |queda| {
        let mut b = pincel();
        b.boundary.queda_no_contorno = queda;
        let mut s = SculptStroke::default();
        s.begin(&malha);
        let p = s.boundary_contorno(&malha, &b, Symmetry::default(), na_boca(&malha));
        let n = p.borda().len();
        let total: f32 = p.borda().iter().map(|t: &TrechoDaBorda| t.peso).sum();
        (n, total)
    };
    let (n_const, w_const) = soma(ph2d_boundary::QuedaNoContorno::Constante);
    let (n_raio, w_raio) = soma(ph2d_boundary::QuedaNoContorno::Raio);
    assert_eq!(
        n_const, n_raio,
        "a CADEIA é a mesma — o selector não entra nas fases A–D"
    );
    assert!(
        w_raio < w_const * 0.75,
        "com `Radius` só um pedaço da boca entra, e a figura tem de o dizer: \
         {w_raio:.3} contra {w_const:.3}"
    );
    assert!(
        w_const > 0.0 && w_raio > 0.0,
        "as duas têm de mostrar alguma coisa: {w_const:.3} / {w_raio:.3}"
    );
}

/// ⭐⭐ **UMA porta: o que se vê ao sobrevoar é o que o traço vai deformar.**
///
/// Durante o gesto a figura sai da estrutura **viva** (a que o pen-down
/// fotografou) e não de uma segunda construção. ⚠️ *Um indicador que mostra
/// outra cadeia é pior que nenhum* — é a mesma lei que o osso da pose paga.
#[test]
fn o_que_se_ve_ao_sobrevoar_e_o_que_o_traco_deforma() {
    let mut malha = tigela();
    let b = pincel();
    let alvo = na_boca(&malha);
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let antes: Vec<TrechoDaBorda> = s
        .boundary_contorno(&malha, &b, Symmetry::default(), alvo)
        .borda()
        .to_vec();
    assert!(!antes.is_empty());

    s.dab(
        &mut malha,
        &b,
        &Dab::pulling(alvo, b.radius, [0.0, 0.0, -1.0], [0.0, 0.05, 0.05]),
        Symmetry::default(),
    );
    let durante: Vec<TrechoDaBorda> = s
        .boundary_contorno(&malha, &b, Symmetry::default(), alvo)
        .borda()
        .to_vec();

    assert_eq!(
        antes.len(),
        durante.len(),
        "é a MESMA cadeia — ela foi fotografada no pen-down"
    );
    for (a, d) in antes.iter().zip(&durante) {
        assert!(
            (a.peso - d.peso).abs() < 1e-6,
            "os pesos são os mesmos: {} contra {}",
            a.peso,
            d.peso
        );
    }
    // ⭐ E ela ANDOU com a deformação: a boca que se vê é a boca que está a dobrar.
    let andou = antes
        .iter()
        .zip(&durante)
        .any(|(a, d)| (a.a[1] - d.a[1]).abs() > 1e-6);
    assert!(
        andou,
        "a figura viva tem de seguir os vértices que se movem"
    );
}

/// ⭐⭐ **O indicador NÃO reconstrói quando nada muda** — a vantagem sobre o
/// alvo, medida por contador e não prometida num cabeçalho.
///
/// ⚠️ O controlo negativo está dentro: o laço corre `60` vezes (muito acima de
/// qualquer orçamento) e mesmo assim o contador fica em `1`.
#[test]
fn o_indicador_nao_reconstroi_quando_nada_muda() {
    let malha = tigela();
    let b = pincel();
    let alvo = na_boca(&malha);
    let mut s = SculptStroke::default();
    s.begin(&malha);
    for _ in 0..60 {
        let _ = s.boundary_contorno(&malha, &b, Symmetry::default(), alvo);
    }
    assert_eq!(
        s.boundary_previa.construcoes, 1,
        "60 quadros de sobrevoo PARADO construíram {} estruturas",
        s.boundary_previa.construcoes
    );
    assert_eq!(
        s.boundary_previa.censos, 1,
        "e {} censos de bordas",
        s.boundary_previa.censos
    );
    // ⭐ E mover o cursor para OUTRA âncora reconstrói — senão o gate acima
    // estaria a afirmar que o indicador é inerte.
    let outro = malha
        .positions()
        .iter()
        .copied()
        .max_by(|a, c| a[0].total_cmp(&c[0]))
        .expect("a malha tem vértices");
    let _ = s.boundary_contorno(&malha, &b, Symmetry::default(), outro);
    assert!(
        s.boundary_previa.construcoes >= 2,
        "mover o cursor tem de reconstruir"
    );
}

/// ⛔ **Sem borda ao alcance a figura é VAZIA, e isso é informação:** numa peça
/// fechada este pincel não move um único vértice, e o indicador tem de dizê-lo
/// em vez de desenhar uma promessa.
#[test]
fn numa_peca_fechada_a_figura_e_vazia() {
    let malha = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let p = s.boundary_contorno(&malha, &b, Symmetry::default(), na_boca(&malha));
    assert!(
        p.borda().is_empty(),
        "{} pedaços numa esfera",
        p.borda().len()
    );
    assert!(p.profundidade().is_none());
}

/// ⛔ **Outro verbo na mão não desenha contorno nenhum.**
#[test]
fn outro_verbo_nao_desenha_contorno() {
    let malha = tigela();
    let b = Brush {
        verb: Verb::Draw,
        ..pincel()
    };
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let p = s.boundary_contorno(&malha, &b, Symmetry::default(), na_boca(&malha));
    assert!(p.borda().is_empty());
    assert!(p.profundidade().is_none());
}

/// ⭐⭐⭐ **NENHUM PEDAÇO ATRAVESSA A BOCA** — o gate que a contagem não era.
///
/// ⛔⛔ **A cadeia NÃO vem em ordem de passeio.** A fase C anda a borda nos dois
/// sentidos ao mesmo tempo e devolve-a ordenada por **distância à âncora**,
/// alternando os lados: medido nesta tigela, `[94, 0, 92, 1, 90, 4, …]` — **`45`
/// de `47`** pares consecutivos **não** são vizinhos de borda. Uma polilinha
/// feita com `windows(2)` sobre ela desenha um ziguezague de **cordas através da
/// boca**, e não a boca.
///
/// ⚠️ **E as réguas que já existiam ficavam verdes sobre ela:** a contagem de
/// pedaços é a mesma, os pesos são os mesmos, o laço fecha na mesma. *Quem
/// desenha uma ligação tem de gatear a RELAÇÃO, não o número de linhas.*
#[test]
fn nenhum_pedaco_atravessa_a_boca() {
    let malha = tigela();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&malha);
    let borda: Vec<TrechoDaBorda> = s
        .boundary_contorno(&malha, &b, Symmetry::default(), na_boca(&malha))
        .borda()
        .to_vec();
    assert!(!borda.is_empty());

    let escondido = vec![false; malha.positions().len()];
    let topo = ph2d_boundary::Topologia::construir(
        malha.positions().len(),
        malha.faces().iter().map(ph2d_mesh::Face::verts),
        &escondido,
    );
    let indice = |p: [f32; 3]| {
        u32::try_from(
            malha
                .positions()
                .iter()
                .position(|q| *q == p)
                .expect("o ponto desenhado é um vértice desta malha"),
        )
        .expect("malha pequena")
    };
    let mut cordas = 0;
    for t in &borda {
        let (a, b) = (indice(t.a), indice(t.b));
        if !topo.vizinhos_de_borda(a).contains(&b) {
            cordas += 1;
        }
    }
    assert_eq!(
        cordas,
        0,
        "{cordas} de {} pedaços ligam vértices que NÃO são vizinhos na borda — \
         a figura atravessa a boca em vez de a seguir",
        borda.len()
    );
}

#[test]
#[ignore]
fn diag_ordem_da_cadeia() {
    let malha = tigela();
    let escondido = vec![false; malha.positions().len()];
    let topo = ph2d_boundary::Topologia::construir(
        malha.positions().len(),
        malha.faces().iter().map(ph2d_mesh::Face::verts),
        &escondido,
    );
    let alvo = na_boca(&malha);
    let anc = ph2d_boundary::ancora::mais_proximo(malha.positions(), &escondido, alvo).unwrap();
    let curva = |p: f32| crate::Falloff::default().weight(1.0 - p);
    let cr: ph2d_boundary::Curva<'_> = &curva;
    let ctrl = ph2d_boundary::Controlos {
        raio_inicial: 0.3,
        raio_dinamico: 0.3,
        forca: 1.0,
        ..Default::default()
    };
    let k = ph2d_boundary::Contorno::comecar(
        &topo,
        malha.positions(),
        malha.normals(),
        &escondido,
        anc,
        alvo,
        &ctrl,
        cr,
        ph2d_boundary::Fatores::default(),
    )
    .unwrap();
    let e = k.estrutura();
    println!(
        "cadeia={} ancora={} primeiro={} ultimo={}",
        e.cadeia.len(),
        e.ancora,
        e.cadeia[0],
        e.cadeia[e.cadeia.len() - 1]
    );
    let mut saltos = 0;
    for par in e.cadeia.windows(2) {
        if !topo.vizinhos_de_borda(par[0]).contains(&par[1]) {
            saltos += 1;
            if saltos < 5 {
                println!("  SALTO {} -> {}", par[0], par[1]);
            }
        }
    }
    println!("saltos na ordem: {saltos} de {}", e.cadeia.len() - 1);
    let u = e.cadeia[e.cadeia.len() - 1];
    println!(
        "vizinhos_de_borda(ultimo={u}) = {:?}  contem primeiro? {}",
        topo.vizinhos_de_borda(u),
        topo.vizinhos_de_borda(u).contains(&e.cadeia[0])
    );
    println!(
        "primeiros 8 da cadeia: {:?}",
        &e.cadeia[..8.min(e.cadeia.len())]
    );
    println!(
        "distancia_de_cadeia dos primeiros 8: {:?}",
        e.cadeia[..8.min(e.cadeia.len())]
            .iter()
            .map(|&v| e.distancia_de_cadeia[v as usize])
            .collect::<Vec<_>>()
    );
}
