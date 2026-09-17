//! ⭐⭐⭐ **OS GATES DA MALHA ASSADA NO REPOUSO** — o critério que não olha para pose nenhuma
//! ([`crate::refine_rest_by_attrs`], F9 W1).
//!
//! ⚠️ Filho do arnês do [`super`] (`#[path]`) para herdar a [`malha`] — a mesma densidade de bind
//! que os outros gates desta crate medem.
//!
//! # A afirmação que estes gates existem para sustentar
//!
//! > *uma malha assada UMA vez serve TODAS as poses* — a premissa da F9 inteira.
//!
//! A W0 mediu-a empiricamente (assar no pior caso e re-posar em `5 × 4` células). Aqui ela é
//! **demonstrada**: com ossos afins o erro de qualquer pose é `Σ_j Δw_j · T_j(meio)`, logo ele é a
//! curvatura do peso vezes a dispersão das poses — e o gate [`o_desvio_de_peso_LIMITA_o_de_qualquer_pose`]
//! corre a desigualdade sobre poses sorteadas.

use super::*;
use crate::{AttrLaw, hermite_attrs, refine_rest_by_attrs};

/// As opções, com a tolerância lida como fracção de PESO (ver o doc da porta).
fn assar(tol: f64, max_pieces: usize) -> RefineOptions {
    RefineOptions {
        tolerance_px: tol,
        max_pieces,
        adaptativo: true,
    }
}

/// ⭐ **DOIS OSSOS, com a troca toda numa banda** — a forma que uma pele de esqueleto tem: o peso é
/// ~constante ao longo de cada osso e vira na articulação.
///
/// ⚠️ O `smoothstep` tem derivada nula nas pontas ⇒ fora da banda o campo é **constante de
/// verdade**, e não «quase»: é ali que o critério tem de ler ZERO.
fn pesos_articulados(mesh: &Mesh2d) -> Vec<f64> {
    const BANDA: f64 = 12.0;
    let mut out = Vec::with_capacity(mesh.rest.len() * 2);
    for p in &mesh.rest {
        let u = ((p[0] - 160.0) / BANDA).clamp(-1.0, 1.0);
        let t = f64::midpoint(u, 1.0);
        let w = t * t * (3.0 - 2.0 * t);
        out.push(1.0 - w);
        out.push(w);
    }
    out
}

/// Pesos que são função AFIM da posição — o campo que a lei de Hermite representa **exactamente**.
fn pesos_lineares(mesh: &Mesh2d) -> Vec<f64> {
    let mut out = Vec::with_capacity(mesh.rest.len() * 2);
    for p in &mesh.rest {
        let w = (p[0] / 320.0).clamp(0.0, 1.0);
        out.push(1.0 - w);
        out.push(w);
    }
    out
}

/// ⛔⛔ **UM CAMPO DE PESOS LINEAR NÃO PARTE NADA** — e a saída é byte-idêntica à entrada.
///
/// ⚠️ **É a metade NEGATIVA, e ela vale metade do valor:** um assador que parta onde o campo é
/// plano gasta o orçamento do quadro inteiro na parte da arte que nenhuma pose dobra.
#[test]
fn um_campo_de_pesos_linear_nao_parte_nada() {
    let m = malha();
    let pesos = pesos_lineares(&m);
    let attrs = hermite_attrs(&m, &pesos, 2);
    let (saida, _, rel) = refine_rest_by_attrs(
        &m,
        &attrs,
        6,
        AttrLaw::Hermite { values: 2 },
        assar(1e-3, m.tris.len() * 8),
    );
    assert_eq!(
        saida.tris.len(),
        m.tris.len(),
        "um campo LINEAR nao tem curvatura nenhuma para perseguir, e a malha partiu na mesma"
    );
    assert_eq!(saida.rest, m.rest, "a malha devia sair byte-identica");
    assert!(
        rel.desvio.is_some_and(|d| d <= 1e-9),
        "o desvio de um campo linear tem de ser ZERO, e leu {:?}",
        rel.desvio
    );
}

/// ⭐⭐ **ONDE O PESO CURVA, A MALHA PARTE — e só ali.**
#[test]
fn onde_o_peso_curva_a_malha_parte_e_so_ali() {
    let m = malha();
    let pesos = pesos_articulados(&m);
    let attrs = hermite_attrs(&m, &pesos, 2);
    let lei = AttrLaw::Hermite { values: 2 };
    let (saida, _, rel) = refine_rest_by_attrs(&m, &attrs, 6, lei, assar(0.02, m.tris.len() * 8));
    assert!(
        saida.tris.len() > m.tris.len(),
        "o campo articulado CURVA na banda e a malha nao partiu ({} pecas)",
        saida.tris.len()
    );
    assert!(
        rel.desvio.is_some_and(|d| d <= 0.02),
        "o assador parou acima da tolerancia: {:?}",
        rel.desvio
    );
    // ⭐⭐ **E só ali — medido contra a LEI UNIFORME, que é o controlo.**
    //
    // ⚠️ **A 1.ª redacção deste gate exigia ZERO vértices novos fora da banda e leu `47` de `388`**
    // — e a exigência é que estava errada: a bissecção da aresta mais longa **propaga** pela cadeia
    // LEPP (é isso que impede um nó pendurado), logo ela arrasta vizinhos de propósito. *Uma barra
    // escrita a olho sobre uma lei que eu não tinha medido acusa a lei.*
    //
    // ⇒ a barra sai do VALE entre as duas leis, as duas calculadas aqui: uma lei uniforme espalha
    // os vértices novos pela arte, logo a fracção dela dentro da banda é a fracção da LARGURA que a
    // banda ocupa (`24 / 320 = 7,5 %`). A adaptativa mede `~88 %`, mais de dez vezes isso.
    let novos = &saida.rest[m.rest.len()..];
    assert!(!novos.is_empty(), "nenhum vertice novo — nada a medir");
    let largura = f64::from(m.size[0]);
    let folga = 12.0 + largura / 24.0;
    let dentro = novos
        .iter()
        .filter(|p| (p[0] - 160.0).abs() <= folga)
        .count();
    let fraccao = dentro as f64 / novos.len() as f64;
    let uniforme = 2.0 * folga / largura;
    assert!(
        fraccao >= 4.0 * uniforme,
        "so' {:.1} % dos {} vertices novos nasceram na banda da junta; uma lei UNIFORME poria \
         {:.1} % la' por acaso — o assador esta' a pagar refinamento onde a arte e' rigida",
        fraccao * 100.0,
        novos.len(),
        uniforme * 100.0
    );
}

/// ⭐⭐⭐ **A LEI DA F9: o desvio de PESO limita o de QUALQUER pose.**
///
/// Com ossos afins, `P(meio) − corda = Σ_j Δw_j · T_j(meio)` e `Σ_j Δw_j = 0`, logo
/// `|erro| ≤ (Σ_j |Δw_j|) · max_j |T_j(meio) − T_ref(meio)|` — o primeiro factor é exactamente o
/// que este critério mede, e o segundo não depende da malha.
///
/// ⇒ assar com tolerância `τ` garante `|erro| ≤ τ · dispersão` em **toda** pose. É esta
/// desigualdade que é corrida aqui, sobre poses que o gate sorteia.
///
/// ⚠️ **O CONTROLO está dentro:** a malha de ENTRADA (não assada) viola a mesma desigualdade, nas
/// mesmas poses. *Sem ele, uma desigualdade folgada passaria sobre qualquer malha.*
#[test]
fn o_desvio_de_peso_limita_o_de_qualquer_pose() {
    let m = malha();
    let pesos = pesos_articulados(&m);
    let attrs = hermite_attrs(&m, &pesos, 2);
    let lei = AttrLaw::Hermite { values: 2 };
    const TAU: f64 = 0.02;
    let (assada, attrs_assados, _) =
        refine_rest_by_attrs(&m, &attrs, 6, lei, assar(TAU, m.tris.len() * 8));

    // Uma pose: dois afins, o 2.º rodado por `ang` à volta da junta. `T_0` é a identidade.
    let pose = |ang: f64| {
        move |j: usize, p: [f64; 2]| -> [f64; 2] {
            if j == 0 {
                return p;
            }
            let (s, c) = ang.sin_cos();
            let (x, y) = (p[0] - 160.0, p[1] - 48.0);
            [(x * c - y * s) + 160.0, (x * s).mul_add(1.0, y * c) + 48.0]
        }
    };
    // O maior erro nos meios das arestas de uma malha, sob uma pose, e a dispersão das poses ali.
    let mede = |malha: &Mesh2d, attrs: &[f64], ang: f64| -> (f64, f64) {
        let t = pose(ang);
        let peso = |v: usize, j: usize| attrs[v * 6 + j];
        let posa = |p: [f64; 2], w: [f64; 2]| {
            let (a, b) = (t(0, p), t(1, p));
            [
                w[0].mul_add(a[0], w[1] * b[0]),
                w[0].mul_add(a[1], w[1] * b[1]),
            ]
        };
        let (mut pior, mut disp) = (0.0_f64, 0.0_f64);
        for tri in &malha.tris {
            for k in 0..3 {
                let (u, v) = (tri[k] as usize, tri[(k + 1) % 3] as usize);
                let (pa, pb) = (malha.rest[u], malha.rest[v]);
                let meio = [f64::midpoint(pa[0], pb[0]), f64::midpoint(pa[1], pb[1])];
                // Os pesos do MEIO pela mesma porta que o refinador usa — uma segunda derivação
                // aqui mediria outro campo que não o dele.
                let mut s = vec![0.0; 6];
                crate::attr_law::midpoint(
                    lei,
                    pa,
                    pb,
                    &attrs[u * 6..(u + 1) * 6],
                    &attrs[v * 6..(v + 1) * 6],
                    &mut s,
                );
                let real = posa(meio, [s[0], s[1]]);
                let ca = posa(pa, [peso(u, 0), peso(u, 1)]);
                let cb = posa(pb, [peso(v, 0), peso(v, 1)]);
                let corda = [f64::midpoint(ca[0], cb[0]), f64::midpoint(ca[1], cb[1])];
                pior = pior.max((real[0] - corda[0]).hypot(real[1] - corda[1]));
                let (a, b) = (t(0, meio), t(1, meio));
                disp = disp.max((a[0] - b[0]).hypot(a[1] - b[1]));
            }
        }
        (pior, disp)
    };

    for graus in [15.0_f64, 45.0, 90.0, 150.0] {
        let ang = graus.to_radians();
        let (erro, disp) = mede(&assada, &attrs_assados, ang);
        let tecto = TAU * disp;
        assert!(
            erro <= tecto,
            "a {graus}°: a malha ASSADA erra {erro:.4} px contra o tecto {tecto:.4} \
             (tau {TAU} x dispersao {disp:.2}) — a lei da F9 nao se sustenta"
        );
        // ⭐ O CONTROLO: a malha de entrada viola o mesmo tecto, na mesma pose.
        let (erro_cru, _) = mede(&m, &attrs, ang);
        assert!(
            erro_cru > tecto,
            "a {graus}°: a malha CRUA tambem cabe no tecto ({erro_cru:.4} <= {tecto:.4}) — \
             a desigualdade esta' folgada e este gate nao afirma nada"
        );
    }
}

/// ⏱️ **A ESCADA τ ↔ PEÇAS** — o número que escolhe a tolerância do produto.
///
/// `cargo test -p ph2d-poly2d -- --ignored --nocapture escada_da_tolerancia`
#[test]
#[ignore = "MEDICAO, nao gate — imprime a escada que a proxima wave lê"]
fn escada_da_tolerancia() {
    let m = malha();
    let pesos = pesos_articulados(&m);
    let attrs = hermite_attrs(&m, &pesos, 2);
    let lei = AttrLaw::Hermite { values: 2 };
    println!("\n  entrada: {} pecas\n", m.tris.len());
    println!("       tau |    pecas | x entrada |   desvio | travado");
    println!("  ---------+----------+-----------+----------+--------");
    for tau in [0.2_f64, 0.1, 0.05, 0.02, 0.01, 0.005] {
        let (s, _, r) = refine_rest_by_attrs(&m, &attrs, 6, lei, assar(tau, 200_000));
        println!(
            "  {tau:>8.3} | {:>8} | {:>8.1}x | {:>8.4} | {}",
            s.tris.len(),
            s.tris.len() as f64 / m.tris.len() as f64,
            r.desvio.unwrap_or(f64::NAN),
            r.travado_pelo_orcamento
        );
    }
    println!();
}

/// ⛔⛔ **O ASSADOR CABE NUM ORÇAMENTO, e DIZ quando o gastou.**
///
/// ⚠️⚠️ **Este gate nasceu de uma mutação que NÃO sangrou numa asserção — sangrou no RELÓGIO.**
/// Somar também os GRADIENTES ao critério (`0..valores` → `0..stride`) infla o desvio, o assador
/// deixa de convergir e a corrida ficou **vinte minutos** a refinar, com um binário órfão a
/// sobreviver a quem o lançou. Os quatro gates acima ficavam pendurados em vez de reprovar.
///
/// ⇒ *um gate cuja única testemunha é o tempo de parede não afirma nada* — o que se afirma aqui é
/// que a assadura **converge dentro de um orçamento realista** (`4 ×` a malha de entrada) e que o
/// relatório **não mente** sobre isso.
#[test]
fn o_assador_cabe_no_orcamento_e_diz_quando_o_gastou() {
    let m = malha();
    let pesos = pesos_articulados(&m);
    let attrs = hermite_attrs(&m, &pesos, 2);
    let lei = AttrLaw::Hermite { values: 2 };
    // ⭐ **`8 ×` sai da ESCADA medida** ([`escada_da_tolerancia`]), não de um palpite: a `τ = 0,02`
    // a assadura pede `4,5 ×` a malha de entrada, e a `τ = 0,01` pede `9,4 ×`. ⚠️ A 1.ª redacção
    // deste gate escreveu `4 ×` a olho e reprovou sobre um assador CORRECTO — *uma barra escolhida
    // antes da medição acusa a lei*.
    let orcamento = m.tris.len() * 8;
    let (saida, _, rel) = refine_rest_by_attrs(&m, &attrs, 6, lei, assar(0.02, orcamento));
    assert!(
        !rel.travado_pelo_orcamento,
        "a assadura gastou o orcamento de {orcamento} pecas e parou a meio ({} pecas, desvio \
         {:?}) — ela devia CONVERGIR muito antes",
        saida.tris.len(),
        rel.desvio
    );
    assert!(
        saida.tris.len() <= orcamento,
        "a assadura entregou {} pecas com um tecto de {orcamento}",
        saida.tris.len()
    );
}

/// ⛔ **A lei linear RECUSA em voz alta** — ela seria inerte, e devolver a entrada em silêncio é
/// indistinguível de uma malha que já estava boa.
#[test]
#[should_panic(expected = "lei de Hermite")]
fn a_lei_linear_recusa_em_voz_alta() {
    let m = malha();
    let pesos = pesos_articulados(&m);
    let _ = refine_rest_by_attrs(&m, &pesos, 2, AttrLaw::Linear, assar(0.02, 100_000));
}
