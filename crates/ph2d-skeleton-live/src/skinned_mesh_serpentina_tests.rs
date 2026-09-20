//! ⭐⭐⭐ **A SERPENTINA — o TAMANHO de uma onda, e o que ela custa em nós.**
//!
//! # ⛔⛔ Porque ela é um ficheiro à parte do irmão
//!
//! O [`super::ondulacao_regua_tests`] responde *«estas duas contagens de nós são comparáveis?»* e
//! esta responde *«esta onda VÊ-SE?»*. São dois trabalhos: o primeiro é uma reamostragem, o
//! segundo é uma amplitude — e foi o segundo que faltava.
//!
//! ⛔⛔⛔ **A CONTAGEM dizia o CONTRÁRIO do que o dono vê.** Com a régua já corrigida, a `90°` em
//! S, o caminho vectorial tem `12` ondas e a lei ideal — a mídia IMAGEM, que é o padrão a copiar —
//! tem `68`. Lido assim, *o vector é cinco vezes melhor*. Medida a FLECHA de cada uma:
//!
//! | | ondas | flecha p50 | **arco** p50 |
//! |---|---:|---:|---:|
//! | lei ideal (a IMAGEM) | `68` | `0,0017` (**`0,17 %`** da espessura) | `0,083` |
//! | VECTOR (o desenho) | `12` | `0,0505` (**`5,0 %`**) | `1,39` |
//!
//! ⇒ *as `68` são facetas curtas e invisíveis e as `12` são serpentinas fundas.* **Contar ondas
//! responde à pergunta errada.**
//!
//! ⚠️ E a flecha SOZINHA também mente: entre duas inflexões pode estar a **dobra inteira** da
//! peça, cuja flecha é grande *por construção* — é por isso que a [`Onda`] carrega o ARCO.

use super::ondulacao_regua_tests::{DENSO, PASSO, b_no_passo, ideal_denso};
use super::ouro_reguas_tests::*;
/// ⭐⭐⭐ **A ALTURA DE CADA ONDA, em unidades do MUNDO — a grandeza que o olho lê.**
///
/// ⛔⛔ **Contar ondas não diz se elas se veem.** A [`ondulacoes`] responde *quantas vezes a linha
/// troca de lado* e o dono fotografou *uma linha que serpenteia* — são a mesma família e não a
/// mesma grandeza: `12` ondas de um milésimo da espessura são invisíveis e `2` de um décimo são o
/// report dele.
///
/// A régua é a **FLECHA** de cada onda: entre duas inflexões consecutivas, o maior afastamento da
/// curva à CORDA que as une. *Para uma senoide de amplitude `A` a flecha é exactamente `A`*, e é
/// esse o controlo positivo da [`super::ondulacao_tests::a_regua_das_ondulacoes_le_zero_no_repouso`]
/// transposto para aqui.
///
/// ⚠️ A banda morta é a mesma da irmã e pela mesma razão — sem ela o ruído numérico parte a aresta
/// recta em centenas de «ondas» de altura nula.
///
/// ⛔⛔ **E ela DEVOLVE O COMPRIMENTO ao lado, porque a flecha sozinha é AMBÍGUA.** Entre duas
/// inflexões pode estar a **dobra inteira** da barra, cuja flecha é grande *por construção* — não
/// é uma ondulação, é a peça a dobrar, que é o que o dono PEDIU. A 1.ª redacção devolvia só a
/// flecha e lia `0,456` (`46 %` da espessura) no caminho vectorial contra `0,004` na lei ideal, o
/// que se lê como *«o vector ondula 100× mais»* e é o CONTRÁRIO: o vector tem `12` ondas LONGAS e
/// a lei ideal `68` CURTAS. *Uma flecha grande num arco longo e uma flecha grande num arco curto
/// são coisas opostas, e a segunda é a única que o olho chama de onda.*
pub(super) fn alturas_das_ondas(poli: &[[f64; 2]], rectas: &[usize]) -> Vec<Onda> {
    let k = b_menger_com_sinal(poli, B_H);
    if k.is_empty() || rectas.is_empty() {
        return Vec::new();
    }
    let mut todas: Vec<f64> = k.iter().map(|v| v.abs()).collect();
    todas.sort_by(f64::total_cmp);
    #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
    let p90 = todas[((todas.len() - 1) as f64 * 0.9).round() as usize];
    let morto = p90 * 0.02;

    // As inflexões, na ordem do arco, e só dentro de um troço CONTÍGUO de aresta recta.
    let (mut out, mut ant_sinal, mut ant_i, mut inicio) = (Vec::new(), 0.0_f64, usize::MAX, None);
    for &i in rectas {
        if ant_i != usize::MAX && i != ant_i + 1 {
            ant_sinal = 0.0;
            inicio = None;
        }
        ant_i = i;
        if k[i].abs() < morto {
            continue;
        }
        let sinal = k[i].signum();
        if ant_sinal != 0.0 && sinal != ant_sinal {
            if let Some(a) = inicio {
                out.push(Onda {
                    flecha: flecha(poli, a, i),
                    arco: arco(poli, a, i),
                });
            }
            inicio = Some(i);
        } else if inicio.is_none() {
            inicio = Some(i);
        }
        ant_sinal = sinal;
    }
    out
}

/// Uma onda: o quanto ela se afasta da corda, e **o quanto de linha ela ocupa**.
///
/// ⭐ A grandeza que o olho lê é a RAZÃO. Um arco de curvatura `k` sobre um comprimento `L` tem
/// flecha `≈ kL²/8`, logo `flecha/arco ≈ kL/8`: uma dobra lenta e larga dá razão pequena e uma
/// serpentina curta e funda dá razão grande. *É adimensional, logo compara peças e escalas.*
#[derive(Clone, Copy)]
pub(super) struct Onda {
    pub(super) flecha: f64,
    pub(super) arco: f64,
}

impl Onda {
    pub(super) fn razao(self) -> f64 {
        if self.arco > 0.0 {
            self.flecha / self.arco
        } else {
            0.0
        }
    }
}

/// O comprimento de `poli[a..=b]`.
fn arco(poli: &[[f64; 2]], a: usize, b: usize) -> f64 {
    (a..b)
        .map(|i| (poli[i + 1][0] - poli[i][0]).hypot(poli[i + 1][1] - poli[i][1]))
        .sum()
}

/// O maior afastamento de `poli[a..=b]` à corda que une as duas pontas.
fn flecha(poli: &[[f64; 2]], a: usize, b: usize) -> f64 {
    let (p, q) = (poli[a], poli[b]);
    let (dx, dy) = (q[0] - p[0], q[1] - p[1]);
    let l = dx.hypot(dy);
    if l <= 0.0 {
        return 0.0;
    }
    (a..=b)
        .map(|i| ((poli[i][0] - p[0]) * dy - (poli[i][1] - p[1]) * dx).abs() / l)
        .fold(0.0_f64, f64::max)
}

/// ⭐⭐⭐ **SONDA — QUÃO ALTAS SÃO AS ONDAS QUE SOBRAM?**
///
/// A espessura da barra é `1,0` do mundo, logo a coluna `% da espessura` é directamente *quanto
/// da peça a serpentina ocupa*.
#[test]
fn diag_c_a_altura_das_ondas_que_sobram() {
    println!("\n{:=<104}", "");
    println!("SONDA · A ALTURA DAS ONDAS — a flecha de cada uma, em unidades do MUNDO");
    println!("  a barra tem espessura 1,0 · passo {PASSO} · janela {B_H}");
    println!("{:=<104}", "");
    println!(
        "{:<22} {:>5} {:<10} | {:>5} | {:>9} {:>9} | {:>8} {:>8} | {:>9} {:>9}",
        "caso",
        "gr",
        "caminho",
        "ondas",
        "flecha p50",
        "MÁX",
        "arco p50",
        "MÁX",
        "razão p50",
        "MÁX"
    );
    for graus in [45.0_f32, 90.0] {
        for (rot, sub) in [("8 nós (o artista)", false), ("34 nós (o BIND)", true)] {
            let mut p = b_palco(sub);
            p.reparte_com(1, false);
            p.lei_do_peso(false);
            p.dobra_em_s(graus);
            let pele = p.pele();
            let rest = b_amostra_com(&p.fonte, DENSO);
            let prod = b_amostra_com(&p.produto(true, true), DENSO);
            let ideal = ideal_denso(&p, &pele, &rest, false);
            for (nome, denso) in [("VECTOR", &prod), ("lei ideal", &ideal)] {
                let (r, v) = b_no_passo(&rest, denso, PASSO);
                let rectas = b_rectas(&r);
                let h = alturas_das_ondas(&v, &rectas);
                let n = h.len();
                let col = |mut c: Vec<f64>| -> (f64, f64) {
                    c.sort_by(f64::total_cmp);
                    (
                        c.get(c.len() / 2).copied().unwrap_or(0.0),
                        c.last().copied().unwrap_or(0.0),
                    )
                };
                let (f50, fmax) = col(h.iter().map(|o| o.flecha).collect());
                let (a50, amax) = col(h.iter().map(|o| o.arco).collect());
                let (r50, rmax) = col(h.iter().map(|o| o.razao()).collect());
                println!(
                    "{rot:<22} {graus:>5.0} {nome:<10} | {n:>5} | {f50:>9.5} {fmax:>9.5} | \
                     {a50:>8.3} {amax:>8.3} | {r50:>9.5} {rmax:>9.5}"
                );
            }
        }
    }
    println!("{:=<104}", "");
}

/// ⭐⭐ **GATE — a régua da ALTURA lê a amplitude de uma onda PLANTADA.**
///
/// ⛔ Sem isto ela pode devolver zeros para sempre e ninguém saber: *uma régua de tamanho que
/// nunca viu um tamanho conhecido não afirma nada sobre os números que imprime.*
#[test]
fn a_regua_da_altura_mede_uma_senoide_plantada() {
    const A: f64 = 0.25;
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(0.0);
    let rest = b_amostra_com(&p.fonte, DENSO);
    let (r, _) = b_no_passo(&rest, &rest, PASSO);
    let rectas = b_rectas(&r);
    // Uma senoide de amplitude `A` ao longo do arco: a flecha de cada meia onda é `A`.
    let cum = b_cum(&r);
    let ondulado: Vec<[f64; 2]> = (0..r.len())
        .map(|i| [r[i][0], (cum[i] * 6.0).sin().mul_add(A, r[i][1])])
        .collect();
    let h = alturas_das_ondas(&ondulado, &rectas);
    assert!(
        h.len() >= 4,
        "a régua devia ver várias ondas numa senoide plantada e viu {}",
        h.len()
    );
    let max = h.iter().map(|o| o.flecha).fold(0.0_f64, f64::max);
    assert!(
        (max - A).abs() < A * 0.25,
        "a flecha de uma senoide de amplitude {A} devia ler ~{A} e leu {max}"
    );
    // ⚠️ O CONTROLO: no repouso não há onda nenhuma, logo não há altura nenhuma.
    assert!(
        alturas_das_ondas(&r, &rectas).is_empty(),
        "a régua inventou ondas na forma de repouso"
    );
}

/// O comprimento acima do qual uma «onda» é, na verdade, a DOBRA da peça.
///
/// ⛔⛔ **Ele é a ESPESSURA da barra e não um número escolhido:** uma serpentina cujo período é
/// maior do que a peça é grossa deixa de se ler como serpentina e passa a ler-se como a peça a
/// curvar — que é o que o dono PEDIU. Medido na tabela da [`diag_c_a_altura_das_ondas_que_sobram`]:
/// a lei ideal tem arco `p50 = 0,083` (facetas) e o caminho vectorial `p50 = 1,39` (a dobra), e
/// **não há nada entre `0,3` e `1,2`** — a fronteira cai num vale, não num corte.
const ARCO_DA_DOBRA: f64 = 1.0;

/// ⭐⭐⭐ **SONDA — A ONDA CURTA, varrida no ÂNGULO.**
///
/// A [`diag_c_a_altura_das_ondas_que_sobram`] mostrou que as `12` do caminho vectorial têm arco
/// `1,39` — elas **são** a dobra. Esta pergunta o que sobra depois de as tirar: *existe, em algum
/// ponto do curso, uma ondulação CURTA e FUNDA no que o dono vê?*
///
/// ⚠️ A coluna `ideal` é o CONTROLO: se ela tiver ondas curtas e o vector não, o ajuste das
/// cúbicas está a alisar — que é o que as contagens já diziam.
#[test]
fn diag_c_a_onda_curta_ao_longo_do_curso() {
    println!("\n{:=<96}", "");
    println!("SONDA · A ONDA CURTA (arco < {ARCO_DA_DOBRA}) ao longo do curso da dobra");
    println!("  a barra tem espessura 1,0 · uma onda de flecha 0,01 é 1 % dela");
    println!("{:=<96}", "");
    println!(
        "{:>5} {:<10} | {:>7} {:>7} | {:>11} {:>11} | {:>11}",
        "graus", "caminho", "curtas", "longas", "flecha p50", "flecha MÁX", "arco MÁX"
    );
    let mut p = b_palco(true);
    for graus in [0.0_f32, 26.0, 45.0, 70.0, 90.0, 110.0, 130.0] {
        p.reparte_com(1, false);
        p.lei_do_peso(false);
        p.dobra_em_s(graus);
        let pele = p.pele();
        let rest = b_amostra_com(&p.fonte, DENSO);
        let prod = b_amostra_com(&p.produto(true, true), DENSO);
        let ideal = ideal_denso(&p, &pele, &rest, false);
        let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
        for (nome, denso) in [("VECTOR", &prod), ("CHÃO", &chao), ("lei ideal", &ideal)] {
            let (r, v) = b_no_passo(&rest, denso, PASSO);
            let rectas = b_rectas(&r);
            let todas = alturas_das_ondas(&v, &rectas);
            let (curtas, longas): (Vec<Onda>, Vec<Onda>) =
                todas.iter().partition(|o| o.arco < ARCO_DA_DOBRA);
            let mut f: Vec<f64> = curtas.iter().map(|o| o.flecha).collect();
            f.sort_by(f64::total_cmp);
            let p50 = f.get(f.len() / 2).copied().unwrap_or(0.0);
            let fmax = f.last().copied().unwrap_or(0.0);
            let amax = curtas.iter().map(|o| o.arco).fold(0.0_f64, f64::max);
            println!(
                "{graus:>5.0} {nome:<10} | {:>7} {:>7} | {p50:>11.5} {fmax:>11.5} | {amax:>11.3}",
                curtas.len(),
                longas.len()
            );
        }
    }
    println!("{:=<96}", "");
}

/// ⭐⭐⭐ **SONDA — A ONDA CURTA CONTRA O NÚMERO DE NÓS: o EXPOENTE.**
///
/// A [`diag_c_a_onda_curta_ao_longo_do_curso`] mostra que o **CHÃO DO MODELO** tem a mesma onda
/// que o produto ⇒ *nenhum ajuste melhor existe com estes nós*. A pergunta que sobra é se ela cai
/// ao acrescentar nós, e **quão depressa** — porque o expoente NOMEIA o que a cúbica está a
/// tentar representar:
///
/// | expoente de `flecha ∼ h^n` | o que está do outro lado |
/// |---|---|
/// | `n ≈ 4` | uma curva LISA — a cúbica converge depressa e mais nós resolvem |
/// | `n ≈ 2` | uma quebra de CURVATURA (`G¹` mas não `G²`) |
/// | `n ≈ 1` | uma QUINA — uma cúbica nunca a faz, e mais nós só a espremem |
///
/// ⭐ A varredura corre sobre o **CHÃO** e não sobre o produto: assim ela mede o MODELO e não o
/// procedimento, e não precisa de reconstruir o bind a cada degrau.
#[test]
fn diag_c_o_expoente_da_onda_contra_os_nos() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    println!("\n{:=<88}", "");
    println!("SONDA · A ONDA CURTA CONTRA O NÚMERO DE NÓS (CHÃO do modelo, 90° em S)");
    println!("{:=<88}", "");
    println!(
        "{:>8} {:>7} {:>9} | {:>12} {:>12} | {:>10}",
        "alvo", "nós", "h médio", "flecha p50", "flecha MÁX", "expoente"
    );
    let mut ant: Option<(f64, f64)> = None;
    for alvo in [1.6_f64, 0.8, 0.4, 0.2, 0.1, 0.05] {
        let mut fonte = p.fonte.clone();
        crate::subdivisao::subdivide(&mut fonte, alvo, crate::subdivisao::VERTICES_MAX);
        let nos = fonte.verts_all().count();
        let rest = b_amostra_com(&fonte, DENSO);
        let (chao, _, _) = b_chao_com(&fonte, &pele, &p.campo, &p.correcoes, DENSO);
        let (r, v) = b_no_passo(&rest, &chao, PASSO);
        let rectas = b_rectas(&r);
        let curtas: Vec<Onda> = alturas_das_ondas(&v, &rectas)
            .into_iter()
            .filter(|o| o.arco < ARCO_DA_DOBRA)
            .collect();
        let mut f: Vec<f64> = curtas.iter().map(|o| o.flecha).collect();
        f.sort_by(f64::total_cmp);
        let p50 = f.get(f.len() / 2).copied().unwrap_or(0.0);
        let fmax = f.last().copied().unwrap_or(0.0);
        let cum = b_cum(&r);
        #[expect(clippy::cast_precision_loss, reason = "contagem de nós")]
        let h = cum[cum.len() - 1] / nos as f64;
        // O expoente local `n` de `flecha ∼ h^n`, entre este degrau e o anterior.
        let exp = ant.map_or(f64::NAN, |(h0, f0)| {
            if f0 > 0.0 && p50 > 0.0 && (h0 - h).abs() > 0.0 {
                (f0 / p50).ln() / (h0 / h).ln()
            } else {
                f64::NAN
            }
        });
        println!("{alvo:>8.2} {nos:>7} {h:>9.4} | {p50:>12.6} {fmax:>12.6} | {exp:>10.2}");
        ant = Some((h, p50));
    }
    println!("{:=<88}", "");
    println!("  ⚠️ o expoente é LOCAL (entre dois degraus) — leia a COLUNA, não uma célula.");
}

/// ⭐⭐⭐ **SONDA — O JOELHO DO `DIVISOES_POR_OSSO`, com o PREÇO ao lado.**
///
/// A [`diag_c_o_expoente_da_onda_contra_os_nos`] diz que a onda mediana cai `28×` entre `34` e
/// `62` nós e depois **pára**. O `K = 3` que ship põe a barra em `34` — e ele foi calibrado contra
/// *«a lei dos pontos de controlo concorda com a lei da CURVA»*, que é uma grandeza que **as duas
/// leis partilham**: elas concordam e as duas ondulam.
///
/// ⚠️ O recurso é o **relógio do recook**, que corre todo quadro — e o cabeçalho da
/// [`crate::subdivisao`] mede que subdividir o TORNA MAIS BARATO (o refit deixa de arder). ⇒ a
/// coluna do relógio tem de estar aqui, senão o joelho é escolhido às cegas.
#[test]
fn diag_c_o_joelho_das_divisoes_por_osso() {
    use std::time::Instant;
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    // O osso mais curto desta barra, para a coluna `K` ser legível.
    let osso = 6.4 / 3.0;
    println!("\n{:=<100}", "");
    println!("SONDA · O JOELHO DO `DIVISOES_POR_OSSO` (90° em S) — a onda e o relógio");
    println!(
        "  o osso mais curto mede {osso:.3} · `K = 3` (o que ship) pede alvo {:.3}",
        osso / 3.0
    );
    println!("{:=<100}", "");
    println!(
        "{:>8} {:>6} {:>6} | {:>12} {:>12} | {:>12}",
        "alvo", "K", "nós", "flecha p50", "flecha MÁX", "recook µs"
    );
    for alvo in [
        osso / 3.0,
        0.60,
        0.50,
        osso / 5.0,
        0.40,
        osso / 6.0,
        0.30,
        osso / 8.0,
    ] {
        let mut fonte = p.fonte.clone();
        crate::subdivisao::subdivide(&mut fonte, alvo, crate::subdivisao::VERTICES_MAX);
        let nos = fonte.verts_all().count();
        let rest = b_amostra_com(&fonte, DENSO);
        let (chao, _, _) = b_chao_com(&fonte, &pele, &p.campo, &p.correcoes, DENSO);
        let (r, v) = b_no_passo(&rest, &chao, PASSO);
        let rectas = b_rectas(&r);
        let mut f: Vec<f64> = alturas_das_ondas(&v, &rectas)
            .into_iter()
            .filter(|o| o.arco < ARCO_DA_DOBRA)
            .map(|o| o.flecha)
            .collect();
        f.sort_by(f64::total_cmp);
        let p50 = f.get(f.len() / 2).copied().unwrap_or(0.0);
        let fmax = f.last().copied().unwrap_or(0.0);
        // ⚠️ O relógio é o do AJUSTE das cúbicas sobre esta contagem de nós — o que corre por
        // quadro. *Medido em debug; o que interessa é a RAZÃO entre as linhas.*
        //
        // ⛔⛔ **O campo é REUSADO e não reconstruído, e isso não é um atalho — é a lei.** O
        // [`ph2d_vec_skin::pesos::CampoDoDominio`] é um campo do DOMÍNIO (a malha BBW e a régua),
        // e responde em qualquer ponto do interior; ele não sabe quantos nós o caminho tem. ⚠️ A
        // 1.ª redacção chamava `campo_do_caminho(&fonte, &[])` — **com os eixos VAZIOS** —, o que
        // dá um campo degenerado e mede outro programa: *um relógio tirado de uma fixtura sem
        // ossos não é o relógio do produto.*
        let pesos = ph2d_vec_skin::pesos::pesos_dos_pontos(&fonte, &p.campo);
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut alvo_path = fonte.clone();
            ph2d_vec_skin::curva::aplica_pela_curva_com(
                &pele,
                &mut alvo_path,
                &pesos,
                &p.correcoes,
                true,
                Some(&p.campo),
            );
        }
        let us = t0.elapsed().as_secs_f64() * 1e6 / 20.0;
        println!(
            "{alvo:>8.3} {:>6.1} {nos:>6} | {p50:>12.6} {fmax:>12.6} | {us:>12.1}",
            osso / alvo
        );
    }
    println!("{:=<100}", "");
    println!(
        "  loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}

/// A flecha mediana, em unidades do mundo, acima da qual a serpentina do report existe.
///
/// ⛔ **Ela não é escolhida: sai do VALE medido** entre os dois lados da tabela do joelho — o
/// `K = 3` lê `0,0486` e o planalto lê `0,00175`, e **não há nada entre `0,0035` e `0,046`**. A
/// barra fica em `0,005` (`0,5 %` da espessura da barra), `2,9×` acima do planalto e `9,7×` abaixo
/// do defeito. *Uma barra num vale de uma ordem de grandeza não precisa de ser afinada.*
const SERPENTINA_MAX: f64 = 0.005;

/// A flecha mediana das ondas mais curtas que a peça é grossa, no caminho dado.
fn serpentina(rest: &[[f64; 2]], denso: &[[f64; 2]]) -> f64 {
    let (r, v) = b_no_passo(rest, denso, PASSO);
    let rectas = b_rectas(&r);
    let mut f: Vec<f64> = alturas_das_ondas(&v, &rectas)
        .into_iter()
        .filter(|o| o.arco < ARCO_DA_DOBRA)
        .map(|o| o.flecha)
        .collect();
    f.sort_by(f64::total_cmp);
    f.get(f.len() / 2).copied().unwrap_or(0.0)
}

/// ⭐⭐⭐ **GATE — A SERPENTINA DO DESENHO MORRE COM OS NÓS, e o `K = 3` é o CONTROLO.**
///
/// Report do dono (2026-09-20): *«a imagem vetorial deforma mal, com várias curvas ao longo do
/// caminho»*. A régua que o traduz é a flecha das ondas **mais curtas que a peça é grossa** — a
/// contagem sozinha diz o contrário (o caminho vectorial tem `12` e a lei ideal `68`), porque as
/// `68` dela são facetas de `0,12 %` e as do vector são serpentinas de `4,9 %`.
///
/// # As três metades, e nenhuma sozinha é honesta
///
/// 1. **A LEI**, com o controlo dentro: ao alvo que a constante de hoje pede a flecha fica abaixo
///    da barra, e ao alvo que o `K = 3` pedia ela fica **uma ordem de grandeza acima**. ⛔ Sem o
///    controlo, alguém baixa o `DIVISOES_POR_OSSO` e este gate continua verde por outro motivo.
/// 2. **O FIO**: o caminho que o PRODUTO entrega (o bind corre a constante por dentro) também fica
///    abaixo da barra. *A lei certa com a constante não ligada lê-se como uma lei que não funciona.*
/// 3. **O TECTO**: a lei ideal — a mídia IMAGEM, que é o que o dono quer copiar — é o piso, e o
///    desenho não pode estar pior do que ela por mais de `4×`. ⚠️ Sem isto a barra seria um número
///    solto; com isto ela é *«tão liso quanto o padrão-ouro»*.
#[test]
fn a_serpentina_do_desenho_morre_com_os_nos() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let osso = 6.4 / 3.0;

    // (1) A LEI, sobre o CHÃO do modelo — e o controlo do `K = 3` ao lado.
    //
    // ⛔⛔ **A base é a peça SEM a subdivisão do bind, e a 1.ª redacção deste gate reprovou-me por
    // não o fazer.** O `BPalco::fonte` já vem subdividida (é o que o bind guardou), e a
    // [`crate::subdivisao::subdivide`] só ACRESCENTA — logo pedir-lhe o alvo do `K = 3` sobre uma
    // peça já cortada a `K = 5` **não corta nada**, e o controlo lia exactamente o mesmo número
    // que o lado que ele devia contradizer (`0,001752` contra `0,001752`). *Não se pode
    // des-subdividir, e um controlo que mede o próprio sujeito não é um controlo.*
    let base = b_palco(false);
    let chao_a = |k: f64| -> f64 {
        let mut fonte = base.fonte.clone();
        crate::subdivisao::subdivide(&mut fonte, osso / k, crate::subdivisao::VERTICES_MAX);
        let rest = b_amostra_com(&fonte, DENSO);
        let (chao, _, _) = b_chao_com(&fonte, &pele, &p.campo, &p.correcoes, DENSO);
        serpentina(&rest, &chao)
    };
    let (hoje, antigo) = (chao_a(crate::subdivisao::DIVISOES_POR_OSSO), chao_a(3.0));
    println!("  lei: hoje {hoje:.6} · o K=3 de antes {antigo:.6} · barra {SERPENTINA_MAX}");
    assert!(
        hoje < SERPENTINA_MAX,
        "a serpentina do CHÃO devia ficar abaixo de {SERPENTINA_MAX} e leu {hoje}"
    );
    assert!(
        antigo > SERPENTINA_MAX * 4.0,
        "o CONTROLO (`K = 3`, o que shipava) devia reproduzir o report do dono e leu {antigo} — \
         esta fixtura deixou de conter o fenómeno que a constante existe para tirar"
    );

    // (2) O FIO: a constante CHEGA ao caminho que o produto entrega, e compra pelo menos METADE.
    let rest = b_amostra_com(&p.fonte, DENSO);
    let prod = serpentina(&rest, &b_amostra_com(&p.produto(true, true), DENSO));
    let chao = serpentina(
        &rest,
        &b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO).0,
    );
    println!("  fio: produto {prod:.6} · chão desta peça {chao:.6}");
    assert!(
        prod < antigo * 0.5,
        "o caminho do PRODUTO ({prod}) devia ficar bem abaixo do que a densidade de antes dava \
         ({antigo}) — a lei está certa e a constante não chega ao bind"
    );

    // (3) ⛔⛔⛔ **A DÍVIDA, AFIRMADA DE PROPÓSITO: com estes nós o TECTO mudou de lado.**
    //
    // A `34` nós o produto e o chão liam o mesmo (`0,0500` contra `0,0486`) ⇒ *o ajuste estava no
    // limite do modelo e nenhum ajuste melhor existia*. Com `54` o chão desce `28×` e o produto
    // desce `3,1×` ⇒ **o que sobra é PROCEDIMENTO**, e a
    // [`diag_c_quem_e_o_tecto_com_nos_a_mais`] nomeia o suspeito: a lei INGÉNUA (sem ajuste de
    // alças) lê `0,0056`, ou seja **`2,9×` mais lisa que o produto**.
    //
    // ⚠️ Esta metade **exige que a dívida exista**. No dia em que alguém curar o ajuste ela
    // reprova, e a cura é reescrever esta prosa com o número novo — *nunca afrouxar a razão*.
    assert!(
        prod > chao * 4.0,
        "o produto ({prod}) deixou de estar acima de 4× o chão ({chao}) — o ajuste das alças \
         deixou de ser o tecto, e o §5 e o cabeçalho da `subdivisao` têm de dizer isso"
    );
}

/// ⭐⭐ **GATE — a régua da serpentina SEPARA a onda da DOBRA.**
///
/// ⛔ Sem isto ela é indistinguível de uma régua que mede a peça a curvar: a 1.ª redacção lia
/// `0,456` (`46 %` da espessura) no caminho vectorial e `0,004` na lei ideal, o que se lê como
/// *«o vector ondula 100× mais»* e é o CONTRÁRIO — o vector tem poucas ondas LONGAS (a dobra) e a
/// lei ideal muitas CURTAS (as facetas).
#[test]
fn a_regua_da_serpentina_nao_conta_a_dobra() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let (r, v) = b_no_passo(&rest, &ideal_denso(&p, &pele, &rest, false), PASSO);
    let rectas = b_rectas(&r);
    let todas = alturas_das_ondas(&v, &rectas);
    let longas: Vec<&Onda> = todas.iter().filter(|o| o.arco >= ARCO_DA_DOBRA).collect();
    let curtas: Vec<&Onda> = todas.iter().filter(|o| o.arco < ARCO_DA_DOBRA).collect();
    assert!(
        !longas.is_empty() && !curtas.is_empty(),
        "a fixtura tem de conter as DUAS espécies — leu {} longas e {} curtas",
        longas.len(),
        curtas.len()
    );
    // ⭐ A DOBRA é funda e comprida; a faceta é rasa e curta. Sem esta separação a mediana das
    // duas juntas é dominada por quem for mais numeroso, que muda com a contagem de nós.
    let dobra = longas.iter().map(|o| o.flecha).fold(0.0_f64, f64::max);
    let faceta = curtas.iter().map(|o| o.flecha).fold(0.0_f64, f64::max);
    assert!(
        dobra > faceta * 2.0,
        "a onda LONGA devia ser a dobra da peça e ser bem mais funda que a mais funda das curtas \
         ({dobra} contra {faceta}) — se elas se confundem, o corte em {ARCO_DA_DOBRA} não separa nada"
    );
}

/// ⭐⭐⭐ **SONDA — COM NÓS A MAIS, QUEM É O TECTO: o modelo ou o procedimento?**
///
/// A `34` nós o caminho do produto e o **CHÃO** liam o mesmo (`0,0500` contra `0,0486`) ⇒ *o
/// ajuste estava no limite do modelo*. Subindo os nós o chão desce `28×` e o produto **não o
/// acompanha** — logo a partir de certa densidade o tecto muda de lado, e esta sonda diz de
/// quanto e a partir de onde.
///
/// ⚠️ A coluna `só pontos` é a lei INGÉNUA (a de antes do ajuste das alças): se ela for igual ao
/// produto, o ajuste não está a comprar nada; se for pior, ele compra e o que falta é outra coisa.
#[test]
fn diag_c_quem_e_o_tecto_com_nos_a_mais() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();
    let rest = b_amostra_com(&p.fonte, DENSO);
    let (chao, _, _) = b_chao_com(&p.fonte, &pele, &p.campo, &p.correcoes, DENSO);
    println!("\n{:=<80}", "");
    println!(
        "SONDA · QUEM É O TECTO com {} nós (90° em S)",
        p.fonte.verts_all().count()
    );
    println!("{:=<80}", "");
    for (rot, v) in [
        (
            "o PRODUTO (lei da curva)",
            b_amostra_com(&p.produto(true, true), DENSO),
        ),
        (
            "só pontos (lei ingénua)",
            b_amostra_com(&p.produto(false, true), DENSO),
        ),
        ("o CHÃO do modelo", chao),
        ("a LEI IDEAL", ideal_denso(&p, &pele, &rest, false)),
    ] {
        println!("{rot:<26} | serpentina p50 {:>10.6}", serpentina(&rest, &v));
    }
    println!("{:=<80}", "");
}

/// A [`serpentina`] como porta para os irmãos — ver
/// [`super::ondulacao_tests::a_leitura_c1_cura_o_campo_e_nao_chega_ao_desenho`].
pub(super) fn serpentina_para_teste(rest: &[[f64; 2]], denso: &[[f64; 2]]) -> f64 {
    serpentina(rest, denso)
}
