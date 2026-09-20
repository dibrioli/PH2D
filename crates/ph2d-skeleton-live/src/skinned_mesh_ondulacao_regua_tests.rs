//! ⭐⭐⭐ **A RÉGUA DAS ONDAS A PASSO FIXO — e a correcção que ela obriga.**
//!
//! # ⛔⛔⛔ O que estava errado, e eu disse-o ao dono
//!
//! O irmão [`super::ondulacao_tests`] mede as ondas **por amostra**, e a amostragem dele é **por
//! SEGMENTO** ([`b_amostra`], `32` pontos por cúbica). ⇒ a mesma peça com `8` nós dá `256`
//! amostras e com `34` dá `1 088`, e a régua é **`4,25×` mais fina** de um lado.
//!
//! Eu li dali *«os `12` que sobram ESCALAM com o número de segmentos (`8` nós → `6`, `34` → `12`)
//! ⇒ é o erro alternado de um ajuste feito segmento a segmento»* e **reportei-o ao dono como o
//! mecanismo**. ⛔ **A mesma tabela tem o CONTROLO que o desmente, na coluna ao lado:** a LEI
//! IDEAL — que é o campo sozinho e **não sabe quantos nós o caminho tem** — lê `12` numa peça e
//! `68` na outra. *Uma grandeza que não pode depender da contagem de nós escalou na mesma
//! proporção* ⇒ o que escala é a RÉGUA.
//!
//! ⚠️ **A curvatura em si está certa e é física** ([`b_menger_com_sinal`] toma os vizinhos a `±h`
//! de ARCO, e não a `±1` amostra). O que não é físico é **contar as trocas de sinal do sinal
//! AMOSTRADO**: dobrar as amostras resolve ondas que a grelha grossa não via. *Uma régua pode ter
//! o estimador certo e a POPULAÇÃO errada.*
//!
//! # ⭐⭐ A cura: o mesmo passo de ARCO dos dois lados
//!
//! A [`b_no_passo`] amostra as duas peças **densas** na parametrização `(segmento, t)` — que é a
//! mesma na fonte e no produto, logo a correspondência é MATERIAL — e depois escolhe delas os
//! pontos a um passo de arco fixo **medido no REPOUSO**. ⇒ as duas contagens de nós passam a ser
//! comparáveis, e o repouso é a referência comum porque é a única forma que as duas peças
//! partilham ao bit.
//!
//! ⚠️ Ficheiro próprio por tecto de LOC (o irmão está a `668` de `700`), e o corte é por
//! RESPONSABILIDADE: *contar ondas e garantir que duas contagens são comparáveis são dois
//! trabalhos, e foi o segundo que faltava.*

use super::ondulacao_tests::ondulacoes;
use super::ouro_reguas_tests::*;

/// Quantas amostras por segmento alimentam a reamostragem. ⚠️ Ela é a resolução **antes** do
/// passo fixo, logo tem de ser fina o bastante para o passo poder cair onde quer: na peça de `8`
/// nós isto dá `2 048` pontos para um passo de `0,01`, ou seja `~7` candidatos por passo.
pub(super) const DENSO: usize = 256;

/// O passo de arco da régua comum, em unidades do MUNDO.
///
/// ⛔ Ele **não** é escolhido por conforto: a janela da curvatura é a [`B_H`] (`0,05`), e uma
/// régua cujo passo se aproxima da janela que ela mede não resolve o que a janela vê. `B_H / 5`
/// põe **cinco** amostras dentro da janela — ⚠️ e a sonda
/// [`diag_c_a_regua_e_estavel_no_passo`] varre o passo para mostrar que a contagem **assenta**.
pub(super) const PASSO: f64 = B_H / 5.0;

/// ⭐⭐⭐ **AS DUAS PEÇAS AO MESMO PASSO DE ARCO, com correspondência MATERIAL.**
///
/// Recebe a fonte e o produto amostrados **densos** na MESMA parametrização `(segmento, t)` e
/// devolve o par reamostrado a um passo de arco fixo, medido no **repouso**.
///
/// ⛔⛔ **A interpolação é no ÍNDICE e não no arco de cada lado.** Reamostrar o produto pelo arco
/// DELE daria pontos que não são o mesmo material que os do repouso, e a máscara das arestas
/// rectas — que se calcula no repouso — deixaria de dizer nada sobre o produto. *Duas
/// reamostragens independentes são duas peças diferentes.*
///
/// ⚠️ **O passo é uniforme no REPOUSO e não no produto**, de propósito: a deformação estica uns
/// troços e comprime outros, e uma régua uniforme no produto contaria mais amostras exactamente
/// onde a peça esticou.
pub(super) fn b_no_passo(
    rest: &[[f64; 2]],
    prod: &[[f64; 2]],
    passo: f64,
) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
    if rest.len() != prod.len() || rest.len() < 2 || passo <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    let cum = b_cum(rest);
    let per = cum[cum.len() - 1];
    if per <= 0.0 {
        return (Vec::new(), Vec::new());
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "per/passo > 0"
    )]
    let m = ((per / passo).round() as usize).max(8);
    let (mut a, mut b) = (Vec::with_capacity(m), Vec::with_capacity(m));
    for j in 0..m {
        #[expect(clippy::cast_precision_loss, reason = "j < m")]
        let s = j as f64 * per / m as f64;
        let f = indice_no_arco(&cum, s);
        a.push(em_indice(rest, f));
        b.push(em_indice(prod, f));
    }
    (a, b)
}

/// O índice FRACCIONÁRIO cujo arco acumulado vale `s`.
fn indice_no_arco(cum: &[f64], s: f64) -> f64 {
    let (mut lo, mut hi) = (0usize, cum.len() - 1);
    while hi - lo > 1 {
        let m = (lo + hi) / 2;
        if cum[m] <= s { lo = m } else { hi = m }
    }
    let seg = cum[lo + 1] - cum[lo];
    #[expect(clippy::cast_precision_loss, reason = "índice de amostra")]
    let base = lo as f64;
    base + if seg > 0.0 { (s - cum[lo]) / seg } else { 0.0 }
}

/// O ponto de uma polilinha FECHADA num índice fraccionário.
fn em_indice(poli: &[[f64; 2]], f: f64) -> [f64; 2] {
    let n = poli.len();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "f >= 0"
    )]
    let i = f.floor() as usize;
    let u = f - f.floor();
    let (a, b) = (poli[i % n], poli[(i + 1) % n]);
    [u.mul_add(b[0] - a[0], a[0]), u.mul_add(b[1] - a[1], a[1])]
}

/// As colunas de uma peça, todas ao mesmo passo. Devolve
/// `(amostras, vector, chão do modelo, lei ideal)`.
fn colunas(p: &mut BPalco, graus: f32, c1: bool) -> (usize, usize, usize, usize) {
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(graus);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let prod = b_amostra_com(&p.produto(true, true), DENSO);
    let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
    let ideal = ideal_denso(p, &pele, &rest, c1);

    let (r, v) = b_no_passo(&rest, &prod, PASSO);
    let (_, c) = b_no_passo(&rest, &chao, PASSO);
    let (_, i) = b_no_passo(&rest, &ideal, PASSO);
    let rectas = b_rectas(&r);
    (
        r.len(),
        ondulacoes(&v, &rectas),
        ondulacoes(&c, &rectas),
        ondulacoes(&i, &rectas),
    )
}

/// A LEI IDEAL nos pontos densos, com a leitura do campo como parâmetro.
pub(super) fn ideal_denso(
    p: &BPalco,
    pele: &ph2d_skeleton::Skin,
    rest: &[[f64; 2]],
    c1: bool,
) -> Vec<[f64; 2]> {
    let suave = c1
        .then(|| ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo))
        .flatten();
    rest.iter()
        .map(|&x| {
            let mut w = pele.scratch();
            let linha = match suave.as_ref() {
                Some(s) => s.linha(x),
                None => p.campo.linha(x),
            }
            .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
            pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
            pele.blend(x, &w)
        })
        .collect()
}

/// ⭐⭐⭐ **SONDA — AS DUAS CONTAGENS DE NÓS, AGORA COMPARÁVEIS.**
///
/// A coluna que decide é o **CHÃO DO MODELO** ([`b_chao`]): a melhor cúbica possível em cada
/// segmento, com as pontas presas no padrão-ouro. *Nenhum ajuste pode fazer melhor do que ela com
/// esta contagem de nós.*
///
/// - se o **VECTOR** ondular mais que o **CHÃO**, o que sobra é **PROCEDIMENTO** e tem cura no
///   ajuste (é aí que um ajuste com continuidade global compraria alguma coisa);
/// - se ele já estiver no chão, o que sobra é **MODELO** — e a cura é outra coisa (mais nós, ou
///   um alvo menos ondulado).
#[test]
fn diag_c_as_duas_contagens_de_nos_ao_mesmo_passo() {
    println!("\n{:=<100}", "");
    println!("SONDA · AS ONDAS AO MESMO PASSO DE ARCO — as duas contagens de nós, comparáveis");
    println!("  passo {PASSO} do mundo · janela da curvatura {B_H} · dobra em S");
    println!("{:=<100}", "");
    println!(
        "{:<22} {:>6} | {:>8} | {:>9} {:>9} | {:>9} {:>9}",
        "caso", "graus", "amostras", "VECTOR", "CHÃO", "ideal bar", "ideal C¹"
    );
    for graus in [45.0_f32, 90.0] {
        for (rot, sub) in [("8 nós (o artista)", false), ("34 nós (o BIND)", true)] {
            let mut p = b_palco(sub);
            let (n, v, c, i_bar) = colunas(&mut p, graus, false);
            let (_, _, _, i_c1) = colunas(&mut p, graus, true);
            println!("{rot:<22} {graus:>6.0} | {n:>8} | {v:>9} {c:>9} | {i_bar:>9} {i_c1:>9}");
        }
    }
    println!("{:=<100}", "");
    println!("  ⚠️ a LEI IDEAL não sabe quantos nós o caminho tem — se a coluna dela AINDA");
    println!("     mudar entre as duas linhas, a régua continua a medir a amostragem.");
    // ⛔⛔ **O CONTROLO DA PORTA.** A coluna `VECTOR` sai da porta do produto, que lê o ambiente
    // — e uma env que NÃO chegou lê-se exactamente como uma cura que não muda nada. ⇒ o carimbo
    // diz o estado da porta E a soma da saída: entre duas corridas, *o estado tem de virar E a
    // soma tem de mudar*, senão a linha de cima não afirma coisa nenhuma.
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let soma: f64 = b_amostra_com(&p.produto(true, true), DENSO)
        .iter()
        .map(|q| q[0].abs() + q[1].abs())
        .sum();
    println!(
        "  CONTROLO DA PORTA · PH2D_SKIN_C1 activo = {} · soma da saída = {soma:.9}",
        ph2d_vec_skin::curva::lei_c1_activa()
    );
}

/// ⭐⭐ **SONDA — A RÉGUA ASSENTA NO PASSO?**
///
/// Uma contagem que continua a subir com a resolução não é uma contagem: é a resolução. Esta
/// varre o passo e mostra onde a leitura estabiliza.
#[test]
fn diag_c_a_regua_e_estavel_no_passo() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let prod = b_amostra_com(&p.produto(true, true), DENSO);
    let ideal = ideal_denso(&p, &pele, &rest, false);
    println!("\n{:=<76}", "");
    println!("SONDA · A RÉGUA ASSENTA NO PASSO? (34 nós, 90° em S)");
    println!("{:=<76}", "");
    println!(
        "{:>12} | {:>10} | {:>10} | {:>10}",
        "passo", "amostras", "VECTOR", "ideal"
    );
    for passo in [B_H / 1.0, B_H / 2.0, B_H / 5.0, B_H / 10.0, B_H / 20.0] {
        let (r, v) = b_no_passo(&rest, &prod, passo);
        let (_, i) = b_no_passo(&rest, &ideal, passo);
        let rectas = b_rectas(&r);
        println!(
            "{passo:>12.4} | {:>10} | {:>10} | {:>10}",
            r.len(),
            ondulacoes(&v, &rectas),
            ondulacoes(&i, &rectas)
        );
    }
    println!("{:=<76}", "");
}
