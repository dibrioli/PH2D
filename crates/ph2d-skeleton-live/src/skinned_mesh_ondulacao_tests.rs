//! ⭐⭐⭐ **AS ONDULAÇÕES — a aresta que devia ser um arco e vai para os dois lados.**
//!
//! Report do dono (2026-09-20, 3.ª foto, com o arco que ele esperava marcado a VERDE):
//! *«a imagem vetorial deforma mal, com várias curvas ao longo do caminho»*.
//!
//! ⛔⛔ **«Várias curvas» não é o CANTO** — que é o assunto do irmão [`super::regularidade_tests`]
//! e o que eu gastei uma ronda inteira a medir. É a **ARESTA a ondular**, e a régua que traduz a
//! frase dele é contar quantas vezes a curvatura troca de **SINAL**.
//!
//! ⚠️ Ficheiro próprio por tecto de LOC (`884` contra `700`), e o corte é por RESPONSABILIDADE:
//! *um canto e uma onda são dois defeitos diferentes, com duas réguas e duas curas.*

use super::ouro_reguas_tests::*;

/// ⭐⭐⭐ **AS ONDULAÇÕES — quantas vezes a linha troca de LADO ao longo de uma aresta.**
///
/// ⛔⛔⛔ **Esta é a régua que o report de 2026-09-20 de facto pedia, e eu gastei uma ronda inteira
/// a medir a coisa errada.** O dono marcou a VERDE o arco que ele esperava e a VERMELHO onde ele
/// não sai: *«a imagem vetorial deforma mal, com várias curvas ao longo do caminho»*. **«Várias
/// curvas» não é o CANTO** — que foi o que eu medi com a [`pior_quina`] — **é a aresta a ondular**:
/// ela vai para um lado, volta para o outro, e outra vez.
///
/// A régua traduz a frase dele à letra: **quantas vezes a curvatura TROCA DE SINAL** ao longo dos
/// troços que em repouso são as arestas rectas da barra. Um arco tem `0`; uma onda tem uma por
/// crista.
///
/// ⚠️ **Só sobre os troços RECTOS do repouso** ([`b_rectas`]): nas pontas redondas a curvatura é
/// grande e de um sinal só, e incluí-las diluiria o sinal no meio do que está certo.
///
/// ⛔⛔ **A BANDA MORTA SAI DA PEÇA INTEIRA E NÃO DO TROÇO MEDIDO, e a 1.ª redacção errou nisso.**
/// Ela tirava o limiar do `p90` de `|k|` **nas próprias arestas rectas** — onde a curvatura
/// verdadeira é ~zero —, logo o limiar era ~zero e o RUÍDO contava como onda: a sonda lia **`20`
/// ondulações com o esqueleto em REPOUSO**, onde a saída é a forma original. *Uma banda morta
/// calibrada na população onde o sinal é nulo não é uma banda morta.*
///
/// ⇒ o limiar é `2 %` da curvatura característica da PEÇA (o `p90` sobre o contorno **todo**, que
/// inclui as pontas redondas). ⚠️ E o CONTROLO desta régua é o repouso: ali as três colunas leem
/// **zero**, e é o gate `a_regua_das_ondulacoes_le_zero_no_repouso` que o afirma.
pub(super) fn ondulacoes(poli: &[[f64; 2]], rectas: &[usize]) -> usize {
    let k = b_menger_com_sinal(poli, B_H);
    if k.is_empty() || rectas.is_empty() {
        return 0;
    }
    let mut todas: Vec<f64> = k.iter().map(|v| v.abs()).collect();
    todas.sort_by(f64::total_cmp);
    #[expect(clippy::cast_precision_loss, reason = "contagem de amostras")]
    let p90 = todas[((todas.len() - 1) as f64 * 0.9).round() as usize];
    let morto = p90 * 0.02;
    // Os índices vêm ordenados; um salto quebra o troço, e uma onda não atravessa dois troços.
    let (mut trocas, mut ant_sinal, mut ant_i) = (0usize, 0.0_f64, usize::MAX);
    for &i in rectas {
        if ant_i != usize::MAX && i != ant_i + 1 {
            ant_sinal = 0.0;
        }
        ant_i = i;
        if k[i].abs() < morto {
            continue;
        }
        let sinal = k[i].signum();
        if ant_sinal != 0.0 && sinal != ant_sinal {
            trocas += 1;
        }
        ant_sinal = sinal;
    }
    trocas
}

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

/// ⭐⭐⭐ **GATE — a régua das ondulações lê ZERO no REPOUSO.**
///
/// ⛔ Sem ele a régua conta RUÍDO: a 1.ª redacção tirava a banda morta das próprias arestas
/// rectas, onde a curvatura verdadeira é nula, e lia **`20`** ondulações num esqueleto que não
/// deformou nada. *Uma régua que vê ondas onde não há nada não pode contar as que há.*
#[test]
fn a_regua_das_ondulacoes_le_zero_no_repouso() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(0.0);
    let pele = p.pele();
    for (rot, poli) in [
        ("a forma de repouso", rest.clone()),
        ("o VECTOR em repouso", b_amostra(&p.produto(true, true))),
        ("a LEI IDEAL em repouso", p.ouro(&pele, &rest)),
    ] {
        assert_eq!(
            ondulacoes(&poli, &rectas),
            0,
            "{rot} devia ler ZERO ondulações — a régua está a contar ruído"
        );
    }
    // ⚠️ O CONTROLO POSITIVO: ela TEM de ver uma onda que exista. Uma senoide de meia amplitude
    // da espessura ao longo da aresta dá uma crista por período.
    let ondulado: Vec<[f64; 2]> = rest
        .iter()
        .enumerate()
        .map(|(i, q)| {
            #[expect(clippy::cast_precision_loss, reason = "índice de amostra")]
            let t = i as f64;
            [q[0], (t * 0.25).sin().mul_add(0.25, q[1])]
        })
        .collect();
    assert!(
        ondulacoes(&ondulado, &rectas) > 10,
        "a régua não viu uma senoide plantada na aresta — ela não contém o fenómeno"
    );
}

/// ⭐⭐ **GATE — uma onda NÃO atravessa dois troços desligados.**
///
/// A [`ondulacoes`] recebe índices que formam **dois** troços (as duas arestas rectas da barra), e
/// a guarda que os separa é uma linha. ⛔⛔ **Ela é INERTE na fixtura do produto** — medido:
/// apagá-la deixa a contagem em `12` contra `12`, porque ali os dois troços se juntam com o mesmo
/// sinal. *Uma linha que a mutação não consegue matar é comentário com sintaxe de código* ⇒ ela
/// ganha aqui a fixtura sintética que a torna observável, em vez de ser apagada ou escondida.
///
/// ⚠️⚠️ **E a 1.ª fixtura sintética também não a exercia:** eu colei dois arcos e a mutação
/// *«põe os dois com o MESMO sinal»* **sobreviveu** — porque dois arcos colados fazem uma QUINA na
/// junção, e uma quina tem curvatura do sinal oposto. *O meu controlo media a JUNÇÃO e não o
/// sinal.* ⇒ a fixtura é uma **senoide inteira**, que é um S liso com **uma** inflexão e nenhuma
/// quina, e a mutação que a troca por meia senoide (uma corcova só, sem inflexão) mata o controlo.
#[test]
fn uma_onda_nao_atravessa_dois_trocos_desligados() {
    const N: usize = 80;
    /// `inflexao = true` ⇒ uma senoide INTEIRA: um S liso, com **uma** troca de sinal ao meio.
    /// `false` ⇒ meia senoide: uma corcova, **sem** inflexão nenhuma.
    fn curva(inflexao: bool) -> Vec<[f64; 2]> {
        (0..N)
            .map(|i| {
                #[expect(clippy::cast_precision_loss, reason = "i < 80")]
                let t = i as f64 / (N - 1) as f64;
                let voltas = if inflexao { 2.0 } else { 1.0 };
                [t * 4.0, 0.35 * (t * voltas * std::f64::consts::PI).sin()]
            })
            .collect()
    }
    let contiguo: Vec<usize> = (0..N).collect();
    // ⚠️ O salto salta o MEIO, que é onde a inflexão mora — como as pontas redondas da barra
    // ficam de fora das arestas rectas.
    const BORDA: usize = 6;
    let com_salto: Vec<usize> = (0..N / 2 - BORDA).chain(N / 2 + BORDA..N).collect();

    let s = curva(true);
    assert!(
        ondulacoes(&s, &contiguo) >= 1,
        "um S liso tem uma inflexão e a régua tem de a ver — sem isto o resto não afirma nada"
    );
    assert_eq!(
        ondulacoes(&s, &com_salto),
        0,
        "com o meio DE FORA, os dois troços não são vizinhos e a junção deles não é uma onda"
    );
    // ⚠️ O CONTROLO: sem inflexão nenhuma a régua lê zero nos dois modos.
    let corcova = curva(false);
    assert_eq!(
        ondulacoes(&corcova, &contiguo),
        0,
        "uma corcova de um sinal só não tem onda — se a régua a vir, ela conta a QUINA ou o ruído"
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

/// ⭐⭐⭐ **GATE — A ARESTA ONDULA, E A ONDA NASCE NO LATTICE, NÃO NO VECTOR.**
///
/// Report do dono (2026-09-20, 3.ª foto, com o arco que ele esperava marcado a VERDE):
/// *«a imagem vetorial deforma mal, com várias curvas ao longo do caminho. Baixa qualidade para um
/// app pro»*.
///
/// ⛔⛔ **«Várias curvas» não é o CANTO — é a ARESTA a ondular**, e eu gastei a ronda anterior a
/// medir o canto. A régua que traduz a frase dele é a [`ondulacoes`]: quantas vezes a curvatura
/// troca de **sinal** ao longo dos troços que em repouso são rectos.
///
/// # A ablação, a `90°` em S
///
/// | caminho | ondulações |
/// |---|---:|
/// | **o que o produto faz hoje** | **`12`** |
/// | com o campo desligado | `24` |
/// | *Deform By: Bone Reach* | `36` |
/// | o caminho dos pontos de controlo (antes da F30) | `24` |
/// | **a LEI IDEAL — a mídia IMAGEM, ponto a ponto** | **`68`** |
///
/// ⇒ *o dono tem razão e nada disto é uma regressão*: a configuração de hoje é a **menos ondulada
/// de todas**, e cada wave que shipou aqui melhorou o número. Mas são `12` e não zero.
///
/// # ⭐⭐ O MECANISMO, medido
///
/// A onda **nasce antes do vector**: a lei ideal ondula `68` e o ajuste das cúbicas alisa-a para
/// `12`. E ela nasce no **lattice**: o contorno atravessa **`119` triângulos** ao longo das
/// arestas rectas, e o peso é **linear DENTRO de cada triângulo** ⇒ a curvatura da deformação é
/// constante lá dentro e **salta em cada aresta atravessada** (`68/119 = 0,57` — cerca de uma onda
/// por cada duas travessias).
///
/// ⇒ **a cura tem nome e não é no vector: é a interpolação do campo deixar de ser linear por
/// triângulo.** Enquanto ela for, a melhor saída possível é a cúbica alisar por cima, que é
/// exactamente o que o produto já faz.
#[test]
fn a_aresta_ondula_e_a_onda_nasce_no_lattice() {
    let mut p = b_palco(true);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let pele = p.pele();

    let hoje = ondulacoes(&b_amostra(&p.produto(true, true)), &rectas);
    let sem_campo = ondulacoes(&b_amostra(&p.produto(true, false)), &rectas);
    let so_pontos = ondulacoes(&b_amostra(&p.produto(false, true)), &rectas);
    let ideal = ondulacoes(&p.ouro(&pele, &rest), &rectas);
    println!(
        "  ondas: hoje {hoje} · sem campo {sem_campo} · só pontos {so_pontos} · ideal {ideal}"
    );

    // (1) A aresta ONDULA — o report é reproduzido, e sem isto o resto não afirma nada.
    assert!(
        hoje >= 8,
        "as arestas rectas deviam ondular (o report do dono) e leram {hoje} — esta fixtura \
         deixou de conter o fenómeno"
    );
    // (2) ⭐ E a configuração de HOJE é a melhor de todas — nenhuma ablação a bate.
    for (rot, v) in [
        ("o campo desligado", sem_campo),
        ("o caminho dos pontos de controlo", so_pontos),
        ("a lei ideal ponto a ponto", ideal),
    ] {
        assert!(
            v > hoje,
            "{rot} devia ondular MAIS que o que o produto faz hoje, e leu {v} contra {hoje} — \
             se isto se inverter, há uma saída melhor que a que ship"
        );
    }
    // (3) ⭐⭐ A onda nasce ANTES do vector: a cúbica alisa a lei ideal para menos de metade.
    assert!(
        hoje * 2 < ideal,
        "o ajuste das cúbicas devia alisar a lei ideal para menos de metade ({ideal} → {hoje})"
    );
}
