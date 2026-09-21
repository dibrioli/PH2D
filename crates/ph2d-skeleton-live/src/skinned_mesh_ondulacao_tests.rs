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
//!
//! ⚠️ **E ele foi cortado outra vez em 2026-09-20 (`714`), pela mesma lei:** as SONDAS mudaram-se
//! para [`super::ondulacao_sondas_tests`] e aqui ficam só as leis.

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

/// ⭐⭐⭐ **GATE — A LEITURA `C¹` CURA O CAMPO, E NÃO CHEGA AO DESENHO.**
///
/// As duas metades são a wave inteira, e **nenhuma sozinha é honesta**:
///
/// 1. o campo melhora de facto (`68 → 22` ondulações, amplitude `2,76 → 1,62`) — sem isto a
///    [`ph2d_vec_skin::pesos_suave`] seria código morto;
/// 2. **o caminho vectorial não se mexe** (`12` com ela e `12` sem ela) — e é por isso que a porta
///    [`ph2d_vec_skin::curva::lei_c1_activa`] nasce **DESLIGADA**.
///
/// ⚠️⚠️ *Uma cura medida na grandeza errada é indistinguível de uma cura.* Se um dia este gate
/// reprovar na 2.ª metade, é porque o ajuste das cúbicas deixou de dominar — e aí o valor de
/// fábrica da porta muda, com este número no diff.
#[test]
fn a_leitura_c1_cura_o_campo_e_nao_chega_ao_desenho() {
    let mut p = b_palco(true);
    p.reparte_com(1, false);
    p.lei_do_peso(false);
    p.dobra_em_s(90.0);
    let rest = b_amostra(&p.fonte);
    let rectas = b_rectas(&rest);
    let pele = p.pele();
    let suave = ph2d_vec_skin::pesos_suave::CampoSuave::novo(&p.campo).expect("campo válido");

    let campo_so = |c1: bool| -> Vec<[f64; 2]> {
        rest.iter()
            .map(|&x| {
                let mut w = pele.scratch();
                let linha = if c1 { suave.linha(x) } else { p.campo.linha(x) }
                    .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                pele.blend(x, &w)
            })
            .collect()
    };
    let (bar, c1) = (campo_so(false), campo_so(true));
    let (o_bar, o_c1) = (ondulacoes(&bar, &rectas), ondulacoes(&c1, &rectas));
    println!("  campo: baricêntrico {o_bar} ondas · C¹ {o_c1}");
    assert!(
        o_bar >= 50,
        "a leitura baricêntrica devia ondular muito ({o_bar}) — a fixtura deixou de conter o \
         fenómeno que a cura existe para tirar"
    );
    assert!(
        o_c1 * 2 < o_bar,
        "a leitura C¹ devia cortar as ondulações do CAMPO para menos de metade ({o_bar} → {o_c1})"
    );

    // (2) ⛔⛔⛔ **A METADE QUE ESTAVA AQUI MORREU EM 2026-09-20, e foi ela própria que me avisou.**
    //
    // Ela afirmava que *«o desenho serpenteia MUITO MAIS do que o campo, logo alisar as facetas do
    // campo não tem por onde mover o desenho»*, e trazia escrita a condição da morte: *«se ele
    // passar a segui-lo, o ajuste deixou de dominar e a porta muda de valor de fábrica»*. Passou.
    // No mesmo dia a **conciliação das alças** foi apagada (report do dono: *«muito curvado»* —
    // ver [`ph2d_vec_skin::curva::aplica_pela_curva`]) e o desenho caiu para o CHÃO do modelo:
    // `0,001727` contra `0,001179` do campo, `1,5×` e não `4×`.
    //
    // ⭐⭐ **E a porta CONTINUA desligada, agora por um PREÇO e não por ausência de efeito.**
    // Medido pela [`super::rive_tests`] com as duas leituras, o `C¹` corta o EXCESSO de curvatura
    // do desenho de `3,16°` para `1,98°` e **não move a amplitude** (`0,001727 → 0,001789`).
    // `3,16°` de quina sobre a janela de `B_H = 0,05` é uma flecha de `~0,14 %` da espessura da
    // barra — abaixo do que se vê — e o preço é `17 %` de um quadro a oito formas presas.
    // *Uma cura invisível não paga um sexto do quadro.*
    //
    // ⏳ **DÍVIDA NOMEADA:** aquele número não é gateado porque a porta do `C¹` é lida **DENTRO**
    // da lei ([`ph2d_vec_skin::curva::aplica_pela_curva_com`]) e não no sítio de chamada — o
    // módulo escreve a lei contrária três vezes, para o `rigido` e para o `campo`. ⇒ um gate só a
    // pode mexer por variável de ambiente, e sob `cargo test` (que corre os testes em THREADS do
    // mesmo processo) isso é um **canal entre testes**. A cura é ela viajar como parâmetro, como
    // as duas irmãs.
    //
    // ⭐ O que fica aqui é a morte, afirmada: o desenho SEGUE o campo.
    use super::serpentina_tests as regua;
    let denso_rest = b_amostra_com(&p.fonte, 256);
    let s_desenho =
        regua::serpentina_para_teste(&denso_rest, &b_amostra_com(&p.produto(true, true), 256));
    let s_campo = regua::serpentina_para_teste(&denso_rest, &{
        denso_rest
            .iter()
            .map(|&x| {
                let mut w = pele.scratch();
                let linha = p
                    .campo
                    .linha(x)
                    .unwrap_or_else(|| b_mais_proximo(&p.campo, x));
                pele.weights_corrected(x, Some(&linha), &mut w, &p.correcoes);
                pele.blend(x, &w)
            })
            .collect::<Vec<_>>()
    });
    println!("  serpentina: desenho {s_desenho:.6} · campo {s_campo:.6}");
    assert!(
        s_campo > 0.0,
        "o campo leu serpentina ZERO — a fixtura deixou de conter o fenómeno e a asserção abaixo \
         passa a ser trivial"
    );
    assert!(
        s_desenho < s_campo * 2.0,
        "o desenho ({s_desenho}) voltou a serpentear muito mais do que o campo que ele copia \
         ({s_campo}) — algum passe voltou a dominar sobre a lei, e é ELE o tecto e não o campo"
    );
}
