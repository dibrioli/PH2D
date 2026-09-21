//! ⭐⭐ **AS SONDAS DAS ONDULAÇÕES** — elas IMPRIMEM; as leis são o irmão
//! [`super::ondulacao_tests`].
//!
//! ⚠️ **Saíram por tecto de LOC (`714` contra `700`) e o corte é por RESPONSABILIDADE** — o mesmo
//! que a [`ph2d_vec_skin`] já fez com as sondas do campo: *uma medição exploratória e uma lei são
//! coisas diferentes, e juntá-las faz o ficheiro crescer a cada report sem lei nova.*

use super::ondulacao_tests::ondulacoes;
use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **SONDA — AS ONDULAÇÕES, o vector contra a lei ideal.**
///
/// Se a lei ideal (a da mídia IMAGEM, amostrada densa) ondular igual, a ondulação é da LEI; se só
/// o caminho vectorial ondular, ela nasce no ajuste das cúbicas — e aí tem cura.
#[test]
fn diag_b_as_ondulacoes_ao_longo_da_aresta() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    println!("\n{:=<100}", "");
    println!("SONDA · AS ONDULAÇÕES — quantas vezes a curvatura troca de sinal nas arestas RECTAS");
    println!("  «várias curvas ao longo do caminho» (report do dono) · um arco tem ZERO");
    println!("{:=<100}", "");
    println!(
        "{:>6} | {:>8} {:>9} {:>11} {:>10} | {:>10} | {:>8}",
        "graus", "VECTOR", "sem campo", "Bone Reach", "só pontos", "LEI IDEAL", "repouso"
    );
    for graus in [0.0_f32, 26.0, 45.0, 70.0, 90.0] {
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let ouro = ondulacoes(&p.ouro(&pele, &rest), &rectas);
        let vector = ondulacoes(&b_amostra(&p.produto(true, true)), &rectas);
        let sem_campo = ondulacoes(&b_amostra(&p.produto(true, false)), &rectas);
        // ⚠️ `curva = false` volta ao caminho dos PONTOS DE CONTROLO (a lei antes da F30) — se a
        // ondulação nascer no ajuste das cúbicas, é aqui que ela desaparece.
        let so_pontos = ondulacoes(&b_amostra(&p.produto(false, true)), &rectas);
        p.lei_do_peso(true);
        p.dobra_em_s(graus);
        let reach = ondulacoes(&b_amostra(&p.produto(true, true)), &rectas);
        println!(
            "{graus:>6.0} | {vector:>8} {sem_campo:>9} {reach:>11} {so_pontos:>10} | \
             {ouro:>10} | {:>8}",
            ondulacoes(&rest, &rectas)
        );
    }
    println!("{:=<100}", "");
    println!(
        "  amostras nas arestas rectas: {} de {}",
        rectas.len(),
        rest.len()
    );
}

/// ⭐⭐⭐ **SONDA — DE ONDE VÊM AS ONDAS: os triângulos do lattice.**
///
/// A [`diag_b_as_ondulacoes_ao_longo_da_aresta`] diz que a LEI IDEAL ondula `68` vezes e que o
/// caminho vectorial, ao ajustar cúbicas por cima dela, a alisa para `12`. ⇒ *as ondas nascem
/// ANTES do vector*, e a hipótese com endereço é a forma do campo: o
/// [`ph2d_vec_skin::pesos::CampoDoDominio`] resolve os pesos numa **malha de triângulos** e o valor
/// num ponto é a interpolação **LINEAR** dentro do triângulo que o contém.
///
/// Um campo linear por triângulo tem segunda derivada **nula dentro** e um salto **em cada
/// aresta** ⇒ a deformação é feita de arcos que trocam de curvatura em cada travessia. **A régua é
/// contar quantos triângulos o contorno atravessa** e comparar com as ondas.
#[test]
fn diag_b_de_onde_vem_a_onda() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();

    // ⛔⛔ **A malha do lattice NÃO vive no espaço do caminho** — ela é guardada numa grelha
    // própria e a `regua` converte (`local_do_vertice`). A 1.ª redacção comparou os dois crus e
    // leu **`770` de `770` amostras FORA da malha**, que é o número que denuncia o engano.
    let m = &p.campo.malha;
    let v: Vec<[f64; 2]> = (0..m.rest.len())
        .map(|i| p.campo.local_do_vertice(i).expect("régua válida"))
        .collect();
    let dentro = |t: &[u32; 3], q: [f64; 2]| {
        let (a, b, c) = (v[t[0] as usize], v[t[1] as usize], v[t[2] as usize]);
        let s = |u: [f64; 2], v: [f64; 2], w: [f64; 2]| {
            (v[0] - u[0]).mul_add(w[1] - u[1], -((w[0] - u[0]) * (v[1] - u[1])))
        };
        let (d1, d2, d3) = (s(q, a, b), s(q, b, c), s(q, c, a));
        !((d1 < 0.0 || d2 < 0.0 || d3 < 0.0) && (d1 > 0.0 || d2 > 0.0 || d3 > 0.0))
    };
    let qual = |q: [f64; 2]| m.tris.iter().position(|t| dentro(t, q));

    let (mut travessias, mut fora, mut ant) = (0usize, 0usize, None);
    for &i in &rectas {
        let t = qual(rest[i]);
        if t.is_none() {
            fora += 1;
        }
        if ant.is_some() && t != ant {
            travessias += 1;
        }
        ant = t;
    }
    let ondas_ideal = ondulacoes(&p.ouro(&pele, &rest), &rectas);
    let ondas_vector = ondulacoes(&b_amostra(&p.produto(true, true)), &rectas);

    println!("\n{:=<92}", "");
    println!("SONDA · DE ONDE VEM A ONDA — as travessias de triângulo contra as ondulações");
    println!("{:=<92}", "");
    println!(
        "  a malha do lattice tem {} vértices e {} triângulos",
        m.rest.len(),
        m.tris.len()
    );
    println!("  o contorno atravessa {travessias} triângulos ao longo das arestas rectas");
    println!("  ({fora} de {} amostras caem FORA da malha)", rectas.len());
    println!("  ondulações da LEI IDEAL ....... {ondas_ideal}");
    println!("  ondulações do VECTOR .......... {ondas_vector}  (a cúbica alisa)");
    println!(
        "  razão ondas/travessias ........ {:.3}",
        f64::from(u32::try_from(ondas_ideal).unwrap_or(0))
            / f64::from(u32::try_from(travessias.max(1)).unwrap_or(1))
    );
    println!("{:=<92}", "");
}

/// ⭐⭐⭐ **SONDA — DESPEJA O LATTICE E O CONTORNO para o oráculo.**
///
/// §0.9: *quando outro app já faz isto, ele é um ORÁCULO que se CORRE sobre entradas NOSSAS*. A
/// pergunta desta wave é de **interpolação de dados dispersos**, e o interpolante `C¹` de
/// referência desde os anos 60 é o **Clough–Tocher** — que a SciPy (**BSD-3**) implementa em
/// `CloughTocher2DInterpolator`, ao lado do `LinearNDInterpolator`, que é **o que nós fazemos
/// hoje**. ⇒ os dois lados da comparação saem do MESMO oráculo, sobre a NOSSA malha e o NOSSO
/// contorno.
///
/// `PH2D_ORACULO_DIR=<pasta>`; sem a variável não faz nada.
#[test]
fn diag_b_despeja_para_o_oraculo() {
    use std::fmt::Write as _;
    let Some(dir) = std::env::var_os("PH2D_ORACULO_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let rest = b_amostra(&p.fonte);
    let campo = &p.campo;
    let ossos = campo.ossos();

    // (1) Os vértices do lattice, **no espaço do caminho** (a `regua` converte).
    let mut v = String::new();
    for i in 0..campo.malha.rest.len() {
        let q = campo.local_do_vertice(i).expect("régua");
        let linha = campo.linha_do_vertice(i).expect("linha");
        let _ = write!(v, "{:.12},{:.12}", q[0], q[1]);
        for w in linha {
            let _ = write!(v, ",{w:.12}");
        }
        v.push('\n');
    }
    std::fs::write(dir.join("lattice.csv"), v).expect("escreve o lattice");

    // (2) Os pontos onde o contorno é amostrado.
    let mut q = String::new();
    for x in &rest {
        let _ = writeln!(q, "{:.12},{:.12}", x[0], x[1]);
    }
    std::fs::write(dir.join("consulta.csv"), q).expect("escreve as consultas");

    println!(
        "despejado: {} vértices × {ossos} ossos · {} pontos de consulta → {}",
        campo.malha.rest.len(),
        rest.len(),
        dir.display()
    );
}

/// ⭐⭐⭐ **SONDA — A RESPOSTA DO ORÁCULO, DEFORMADA PELO MOTOR DO PRODUTO.**
///
/// A [`diag_b_despeja_para_o_oraculo`] escreve o lattice e o contorno; o `oraculo_do_campo.py` corre a
/// SciPy (**BSD-3**) e devolve **dois** campos de peso nos MESMOS pontos — o `LinearNDInterpolator`
/// (o que fazemos hoje) e o `CloughTocher2DInterpolator` (o `C¹` de referência). Esta lê os dois
/// de volta e passa-os pela **porta do produto** ([`ph2d_skeleton::Skin::weights_from`] seguida do
/// `blend`), que é a única forma de a comparação ser sobre a LEI e não sobre duas aritméticas.
///
/// ⚠️ **Os dois lados saem do MESMO oráculo** — se eu tivesse posto o nosso interpolador de um
/// lado e a SciPy do outro, a diferença incluiria toda divergência de implementação.
#[test]
fn diag_b_o_oraculo_deformado_pelo_produto() {
    let Some(dir) = std::env::var_os("PH2D_ORACULO_DIR") else {
        return;
    };
    let dir = std::path::PathBuf::from(dir);
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    let pele = p.pele();

    println!("\n{:=<86}", "");
    println!("SONDA · O ORÁCULO (SciPy, BSD-3) deformado pelo MOTOR DO PRODUTO");
    println!("{:=<86}", "");
    println!(
        "{:<22} | {:>12} | {:>12}",
        "campo de peso", "ondulações", "pior quina"
    );
    for nome in [
        "linear",
        "clough_tocher",
        "molificado_1.0",
        "molificado_2.0",
        "molificado_4.0",
    ] {
        let Ok(txt) = std::fs::read_to_string(dir.join(format!("pesos_{nome}.csv"))) else {
            println!("  (falta pesos_{nome}.csv — corra o oraculo_do_campo.py primeiro)");
            continue;
        };
        let linhas: Vec<Vec<f64>> = txt
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.split(',').filter_map(|v| v.trim().parse().ok()).collect())
            .collect();
        assert_eq!(
            linhas.len(),
            rest.len(),
            "o oráculo devolveu outra contagem"
        );
        let saida: Vec<[f64; 2]> = rest
            .iter()
            .zip(&linhas)
            .map(|(&x, row)| {
                let mut w = pele.scratch();
                pele.weights_from(x, row, &mut w);
                pele.blend(x, &w)
            })
            .collect();
        println!(
            "{nome:<22} | {:>12} | {:>11.1}°",
            ondulacoes(&saida, &rectas),
            super::regularidade_tests::pior_quina(&saida)
        );
    }
    println!("{:=<86}", "");
}

/// ⭐⭐⭐ **SONDA — ONDE MORAM OS `12` QUE SOBRAM: no CAMPO ou no AJUSTE das cúbicas?**
///
/// A leitura `C¹` ([`ph2d_vec_skin::pesos_suave`]) tira a faceta do CAMPO — o oráculo mediu
/// `68 → 16` — e o caminho vectorial **não se mexe** (`12` com ela e `12` sem ela). Esta sonda
/// separa as duas metades:
///
/// - a **LEI IDEAL** com cada leitura, que é o campo sozinho;
/// - o **VECTOR** com `8` e com `34` nós, que é o ajuste sozinho (o campo é o mesmo).
///
/// *Se o campo melhora e o vector não, o que sobra nasce no ajuste — e mais nós têm de o baixar.*
#[test]
fn diag_b_onde_moram_os_doze() {
    let mut p8 = b_palco(false);
    let mut p34 = b_palco(true);
    println!("\n{:=<92}", "");
    println!("SONDA · ONDE MORAM OS 12 — o campo ou o ajuste das cúbicas?");
    println!("{:=<92}", "");
    println!(
        "{:<26} | {:>14} | {:>14} | {:>14}",
        "caso", "ondulações", "nós", "amostras"
    );
    for (rot, p) in [
        ("8 nós (o artista)", &mut p8),
        ("34 nós (o BIND)", &mut p34),
    ] {
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(90.0);
        let rest = b_amostra(&p.fonte);
        let rectas = b_rectas(&rest);
        let pele = p.pele();
        let prod = p.produto(true, true);
        // A LEI IDEAL com as DUAS leituras do campo — o campo sozinho, sem ajuste nenhum.
        let suave = ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo);
        let ideal = |c1: bool| -> usize {
            let v: Vec<[f64; 2]> = rest
                .iter()
                .map(|&x| {
                    let mut w = pele.scratch();
                    let linha = match (c1, suave.as_ref()) {
                        (true, Some(s)) => s.linha(x),
                        _ => p.campo.linha(x),
                    }
                    .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                    pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                    pele.blend(x, &w)
                })
                .collect();
            ondulacoes(&v, &rectas)
        };
        println!(
            "{rot:<26} | vector {:>7} | {:>14} | {:>14}",
            ondulacoes(&b_amostra(&prod), &rectas),
            prod.verts_all().count(),
            rest.len()
        );
        println!(
            "{:<26} | ideal baricêntrico {:>2} · ideal C¹ {:>2}",
            "",
            ideal(false),
            ideal(true)
        );
    }
    println!("{:=<92}", "");
}

/// ⚠️ **SONDA — a AMPLITUDE da ondulação e o PREÇO da leitura `C¹`.**
///
/// A contagem diz *quantas* ondas e não *de que tamanho*. E uma cura que não move a contagem pode
/// ainda assim baixar a amplitude — ou não, e aí ela não se paga.
#[test]
fn diag_b_a_amplitude_e_o_preco_do_c1() {
    use std::time::Instant;
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);

    println!("\n{:=<92}", "");
    println!("SONDA · a AMPLITUDE da ondulação e o PREÇO da leitura C¹");
    println!("{:=<92}", "");
    println!(
        "{:<22} | {:>10} {:>12} {:>12} | {:>12}",
        "leitura", "ondas", "|k| p90", "|k| MÁX", "ms/recozer"
    );
    // ⚠️ A porta é lida por `aplica_pela_curva_com`, e uma env **não** se muda a meio de um teste
    // (a suíte corre em threads). ⇒ mede-se a LEI directamente, pelas duas portas.
    let suave = ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo);
    for (rot, c1) in [("baricêntrica", false), ("C¹", true)] {
        let pele = p.pele();
        let t0 = Instant::now();
        let v: Vec<[f64; 2]> = rest
            .iter()
            .map(|&x| {
                let mut w = pele.scratch();
                let linha = match (c1, suave.as_ref()) {
                    (true, Some(s)) => s.linha(x),
                    _ => p.campo.linha(x),
                }
                .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                pele.blend(x, &w)
            })
            .collect();
        let ms = t0.elapsed().as_secs_f64() * 1e3;
        let mut k: Vec<f64> = rectas.iter().map(|&i| b_menger(&v, B_H)[i].abs()).collect();
        let (_, p90, max) = b_pct(&mut k);
        println!(
            "{rot:<22} | {:>10} {p90:>12.4} {max:>12.4} | {ms:>12.3}",
            ondulacoes(&v, &rectas)
        );
    }
    // ⭐ O preço de DERIVAR os gradientes — uma vez por forma por quadro.
    let t0 = Instant::now();
    for _ in 0..20 {
        let _ = ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo);
    }
    println!(
        "  derivar os gradientes ({} vértices × {} ossos): {:.3} ms",
        p.campo.malha.rest.len(),
        p.campo.ossos(),
        t0.elapsed().as_secs_f64() * 1e3 / 20.0
    );
    println!(
        "  loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("{:=<92}", "");
}
