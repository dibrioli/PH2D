//! O PLACAR da bancada malha-a-malha do pente — a catraca.
//!
//! A bancada (motor, leitor e sondas) vive em [`crate::oraculo_do_pente`];
//! aqui está só o veredito, numa tabela **por célula com o número MEDIDO**,
//! no idioma da bancada do tecido (`VERDE_N` / `ABERTO_N`).
//!
//! # O que este placar responde
//!
//! A pergunta do dono era *«a nossa lei do pente é a dele?»*, e o corpus
//! responde-a com um CONTROLO que é metade do valor:
//!
//! | | com o pente DESLIGADO | com o pente no máximo |
//! |---|---|---|
//! | `x_man_x01` (1 passagem) | **`1,624e-4`** | `2,814e-2` |
//! | `x_man_x02` | `1,164e-3` | `3,143e-2` |
//! | `x_man_x04` | `6,240e-3` | `3,765e-2` |
//! | `x_man_x08` | `2,135e-2` | `5,374e-2` |
//! | `x_man_x16` | `6,966e-2` | `9,905e-2` |
//!
//! ⭐ **Com o pente desligado a nossa lei bate a dele a `1,6e-4` a UMA
//! passagem**, e o resíduo cresce `429×` ao longo da escada ⇒ ele é
//! **acumulação** de `f32`, não erro de lei. A malha, o pincel, o traço e
//! este arnês estão portanto ILIBADOS por resultado.
//!
//! ⛔⛔⛔ **Com o pente ligado o desvio começa em `2,8e-2` na MESMA passagem
//! — `173×` pior — e quase não cresce.** O erro do pente é uma diferença de
//! FORMA fixa. E medido contra o efeito do próprio knob no alvo
//! (`diag_o_pente_contra_o_efeito_dele`), a razão `erro / efeito` é `0,88` a
//! `2,96`: *o nosso erro tem o tamanho do efeito inteiro do controlo*.
//!
//! ⭐ **E a MAGNITUDE está certa:** a `x01` o pente move `3,199e-2` nele e
//! `3,177e-2` em nós — `0,7 %` de diferença. *A força acerta; a forma não.*
//! Em traços de várias passagens e no arco o nosso é `1,4×`–`2,1×` forte
//! demais (`y_arco`: `9,848e-2` contra `4,728e-2`).
//!
//! # As três tabelas, e porque são três
//!
//! - **`VERDE`** — dentro da barra. ⚠️ São só as que MOVEM barro: uma célula
//!   que não move nada entra no `VACUO`, nunca aqui.
//! - **`ABERTO`** — desvio medido, com o número ao lado. Catraca nos DOIS
//!   sentidos: uma célula que piora reprova, e uma que **fecha** reprova
//!   também, para ser movida para o `VERDE` (a catraca sem censo de
//!   obsolescência vira licença).
//! - **`VACUO`** — nenhum dos dois lados moveu um vértice. ⛔⛔ Elas são
//!   `0,000e0` com QUALQUER lei, logo contá-las como paridade seria mentir:
//!   *um zero de «igual» e um de «nada aconteceu» são o mesmo byte*.
//!
//! # ⚠️ Três células do `ABERTO` são dívida do ARNÊS, não da lei
//!
//! `composicao/m_rot`, `verbos/v_grab` e `verbos/v_thumb` são verbos
//! ANCORADOS e este arnês conduz-os como CARIMBO: nós movemos `0` vértices
//! onde o oráculo move `49`/`56`/`56`. ⭐ Para a pergunta do pente elas são
//! vácuo de qualquer maneira — o `diag_o_pente_contra_o_efeito_dele` lê
//! `dele = 0,000e0` nas três, ou seja **o pente não lhes toca no ALVO**, o
//! que confirma o censo da §6.3. A dívida é do `p000`.
//!
//! # ⭐ Uma lei que o oráculo CONFIRMOU depois
//!
//! `porta/c_nodyn` lê `dele = 0,000e0`: **sem topologia dinâmica o pente do
//! alvo é inerte**, que é exactamente o que o `space::pente_do_traco` já fazia
//! por raciocínio de domínio. *Uma cura que o oráculo depois confirma é a
//! melhor prova de que o raciocínio era do DOMÍNIO e não do programa.*

use crate::oraculo_do_pente::{
    celulas_sem_remalha, correr, entrada, ler, maior_distancia, movidos,
};

/// A barra. As mesmas unidades de objecto das bancadas do tecido e da pose.
const BARRA: f32 = 1e-5;

/// Uma célula do `ABERTO` que mede MENOS do que isto da tolerância melhorou
/// o suficiente para a tabela estar a mentir ⇒ reprova, para o número descer.
const FOLGA: f32 = 2.0;

/// ⚠️ A tabela carrega **quatro algarismos significativos**, logo o valor
/// medido fica até `5e-4` relativos acima do que está escrito. Esta cerca é
/// de TRANSCRIÇÃO e não da barra — sem ela toda a tabela reprova contra si
/// mesma, e com ela um agravamento real (que é de ordens de grandeza, não de
/// um milésimo) continua a acusar.
const EPS_ARRED: f32 = 1e-3;

/// Dentro da barra **e** com barro a mover-se.
const VERDE: &[(&str, f32)] = &[
    ("mecanismo/y_umdab_p000", 0.000e0),
    ("mecanismo/y_umdab_p100", 0.000e0),
    // ⭐⭐⭐ As quatro que FECHARAM ao conduzir o gesto ancorado pela regra
    // provada: de `6,874e-1` e `3,437e-1` para ruído de `f32`, com `56/56`
    // vértices movidos contra os `0/56` de antes. *A lei estava certa; o
    // arnês entregava um CARIMBO a um verbo que segura.*
    ("verbos/v_grab_p000", 1.192e-7),
    ("verbos/v_grab_p100", 1.192e-7),
    ("verbos/v_thumb_p000", 1.192e-7),
    ("verbos/v_thumb_p100", 1.192e-7),
];

/// Desvio medido, em unidades de objecto. ⛔ Só ENCOLHE.
const ABERTO: &[(&str, f32)] = &[
    ("banda/b_man1_p000", 2.984e-2),
    ("banda/b_man1_p100", 6.203e-2),
    ("banda/b_man2_p000", 2.984e-2),
    ("banda/b_man2_p100", 6.193e-2),
    ("banda/b_man3_p000", 2.984e-2),
    ("banda/b_man3_p100", 6.195e-2),
    ("composicao/m_collapse_p000", 2.984e-2),
    ("composicao/m_collapse_p100", 6.197e-2),
    ("composicao/m_manual_p000", 2.984e-2),
    ("composicao/m_manual_p100", 6.206e-2),
    ("composicao/m_rot_p000", 7.824e-2),
    ("composicao/m_rot_p100", 7.824e-2),
    ("mecanismo/x_man_x01_p000", 1.624e-4),
    ("mecanismo/x_man_x01_p100", 2.814e-2),
    ("mecanismo/x_man_x02_p000", 1.164e-3),
    ("mecanismo/x_man_x02_p100", 3.143e-2),
    ("mecanismo/x_man_x04_p000", 6.240e-3),
    ("mecanismo/x_man_x04_p100", 3.765e-2),
    ("mecanismo/x_man_x08_p000", 2.135e-2),
    ("mecanismo/x_man_x08_p100", 5.374e-2),
    ("mecanismo/x_man_x16_p000", 6.966e-2),
    ("mecanismo/x_man_x16_p100", 9.905e-2),
    ("mecanismo/y_arco_p000", 9.078e-2),
    ("mecanismo/y_arco_p100", 1.398e-1),
    ("mecanismo/y_doisdab_p000", 9.157e-3),
    ("mecanismo/y_doisdab_p100", 2.688e-2),
    ("mecanismo/y_ida_p000", 2.984e-2),
    ("mecanismo/y_ida_p100", 6.192e-2),
    ("mecanismo/y_parado_p000", 1.197e-1),
    ("mecanismo/y_parado_p100", 1.197e-1),
    ("mecanismo/y_volta_p000", 2.954e-2),
    ("mecanismo/y_volta_p100", 6.252e-2),
    ("porta/c_nodyn_p000", 2.234e-2),
    ("porta/c_nodyn_p100", 2.234e-2),
    ("verbos/r_collapse_p000", 2.984e-2),
    ("verbos/r_collapse_p100", 6.197e-2),
    ("verbos/t_manual_p000", 2.984e-2),
    ("verbos/t_manual_p100", 6.203e-2),
    ("verbos/v_draw_sharp_p000", 5.712e-3),
    ("verbos/v_draw_sharp_p100", 5.712e-3),
    ("verbos/v_layer_p000", 2.218e-2),
    ("verbos/v_layer_p100", 2.986e-2),
    ("verbos/v_smooth_p000", 1.938e-2),
    ("verbos/v_smooth_p100", 2.388e-2),
];

/// ⛔ Nenhum dos dois lados move um vértice ⇒ `0,000e0` com qualquer lei.
const VACUO: &[&str] = &[
    "composicao/m_mask_p000",
    "composicao/m_mask_p100",
    "mecanismo/x_esf_x01_p000",
    "mecanismo/x_esf_x01_p100",
    "mecanismo/x_esf_x08_p000",
    "mecanismo/x_esf_x08_p100",
    "mecanismo/y_esf8_p000",
    "mecanismo/y_esf8_p100",
    "mecanismo/y_esf_p000",
    "mecanismo/y_esf_p100",
    "verbos/v_mask_p000",
    "verbos/v_mask_p100",
    "verbos/v_rotate_p000",
    "verbos/v_rotate_p100",
];

/// O piso de população: abaixo disto a varredura do corpus partiu-se e o
/// gate ficaria verde a medir quase nada.
const PISO: usize = 60;

fn chave(fam: &str, nome: &str) -> String {
    format!("{fam}/{nome}")
}

/// O que uma célula do `ABERTO` acusa, se acusar.
///
/// ⚠️ É uma lei PURA de propósito: **três dos quatro regimes não são
/// alcançáveis pelo corpus de hoje** (nenhuma célula fechou, nenhuma melhorou
/// e todas estão classificadas), e uma cerca que o produto não exercita não
/// tem fixtura que a imponha. Extraída, ela ganha controlo em
/// [`os_quatro_regimes_do_aberto`].
#[derive(Debug, PartialEq, Eq)]
enum Veredito {
    Bem,
    /// Regrediu — o número da tabela ficou para trás.
    Piorou,
    /// Caiu dentro da barra ⇒ mude-a para o `VERDE`.
    Fechou,
    /// Melhorou muito ⇒ desça o número, senão a tabela vira licença.
    Desca,
}

fn julga_aberto(d: f32, tol: f32) -> Veredito {
    if d > tol * (1.0 + EPS_ARRED) {
        Veredito::Piorou
    } else if d <= BARRA {
        Veredito::Fechou
    } else if d * FOLGA < tol {
        Veredito::Desca
    } else {
        Veredito::Bem
    }
}

/// O controlo dos quatro ramos, um a um — ⛔ sem ele, apagar o ramo do
/// «FECHOU» ou o do «DESÇA» não parte um único teste, porque o corpus de hoje
/// não tem uma célula em nenhum dos dois regimes.
#[test]
fn os_quatro_regimes_do_aberto() {
    let tol = 1e-2;
    assert_eq!(
        julga_aberto(2e-2, tol),
        Veredito::Piorou,
        "o dobro da tolerância"
    );
    assert_eq!(
        julga_aberto(9e-3, tol),
        Veredito::Bem,
        "abaixo, e sem folga de sobra"
    );
    assert_eq!(julga_aberto(1e-6, tol), Veredito::Fechou, "dentro da barra");
    assert_eq!(
        julga_aberto(2e-3, tol),
        Veredito::Desca,
        "cinco vezes melhor"
    );
    // A cerca de TRANSCRIÇÃO: o valor exacto por cima do arredondamento passa,
    // e um milésimo acima do que ela cobre já acusa.
    assert_eq!(
        julga_aberto(tol * 1.0005, tol),
        Veredito::Bem,
        "arredondamento"
    );
    assert_eq!(
        julga_aberto(tol * 1.01, tol),
        Veredito::Piorou,
        "um por cento acima"
    );
    // ⚠️ E a fronteira entre FECHOU e DESÇA é a BARRA, não a folga: um valor
    // dentro da barra manda para o VERDE mesmo estando muito abaixo da
    // tolerância, senão as duas curas trocam-se.
    assert_eq!(julga_aberto(BARRA, tol), Veredito::Fechou);
}

#[test]
fn o_placar_do_pente_nao_regride() {
    let mut falhas = Vec::new();
    let mut vistas = 0usize;

    for (fam, nome) in celulas_sem_remalha() {
        let k = chave(&fam, &nome);
        vistas += 1;
        let c = ler(&fam, &nome);

        // ⭐⭐⭐ A premissa que torna um placar de POSIÇÕES completo: nestas
        // células o pente do alvo não vira uma única aresta. *Uma grade é
        // propriedade da CONECTIVIDADE*, logo se uma célula nova mudar a
        // ligação este placar fica cego exactamente onde a lei vive — e
        // reprova aqui, em vez de ficar verde a medir metade.
        let dentro = entrada(&c);
        assert!(
            dentro.faces().len() == c.faces.len()
                && dentro.faces().iter().zip(&c.faces).all(|(x, y)| x == y),
            "{k}: a saída do oráculo mudou a LIGAÇÃO — este placar compara \
             posições e deixou de ser completo para esta célula"
        );

        let nosso = correr(&c);
        let d = maior_distancia(&nosso, &c.saida);
        let nos_movidos = movidos(&c, &nosso);
        let ele_movidos = movidos(&c, &c.saida);

        if VACUO.contains(&k.as_str()) {
            // A metade que impede o vácuo de ser lido como paridade: ele tem
            // de CONTINUAR vácuo, senão passa a medir alguma coisa e muda de
            // tabela.
            if nos_movidos != 0 || ele_movidos != 0 {
                falhas.push(format!(
                    "{k}: estava no VACUO e agora move ({nos_movidos}/{ele_movidos}) \
                     — mova-a para o VERDE ou para o ABERTO"
                ));
            }
            continue;
        }

        if let Some((_, tol)) = VERDE.iter().find(|(n, _)| *n == k) {
            if ele_movidos == 0 {
                falhas.push(format!(
                    "{k}: está no VERDE e o oráculo não move um vértice — \
                     isso é VACUO, não paridade"
                ));
            }
            if d > BARRA {
                falhas.push(format!(
                    "{k}: VERDE regrediu — {d:.3e} > {BARRA:.0e} (tol {tol:.3e})"
                ));
            }
            continue;
        }

        if let Some((_, tol)) = ABERTO.iter().find(|(n, _)| *n == k) {
            match julga_aberto(d, *tol) {
                Veredito::Bem => {}
                Veredito::Piorou => {
                    falhas.push(format!("{k}: ABERTO piorou — {d:.3e} > {tol:.3e}"));
                }
                Veredito::Fechou => falhas.push(format!(
                    "{k}: ABERTO FECHOU ({d:.3e}) — mova-a para o VERDE"
                )),
                Veredito::Desca => falhas.push(format!(
                    "{k}: ABERTO melhorou muito ({d:.3e} contra {tol:.3e}) — \
                     desça o número da tabela"
                )),
            }
            continue;
        }

        falhas.push(format!(
            "{k}: célula do corpus que nenhuma das três tabelas classifica"
        ));
    }

    assert!(
        vistas >= PISO,
        "piso de população: {vistas} células, esperadas ao menos {PISO} — \
         a varredura do corpus partiu-se"
    );
    assert_eq!(
        vistas,
        VERDE.len() + ABERTO.len() + VACUO.len(),
        "as três tabelas têm de somar o corpus: {vistas} células contra \
         {} + {} + {} classificadas",
        VERDE.len(),
        ABERTO.len(),
        VACUO.len()
    );
    assert!(falhas.is_empty(), "{}", falhas.join("\n"));
}

/// ⭐⭐ **O CONTROLO que torna o placar de confiar: com o pente DESLIGADO a
/// nossa lei bate a dele a uma passagem.**
///
/// Sem esta metade, um placar todo em `ABERTO` lê-se como *«a bancada não
/// funciona»*. Com ela, o `1,624e-4` do `x_man_x01_p000` prova que a malha, o
/// pincel, o traço e o arnês estão certos — e que **o que está errado é a
/// coluna do pente**.
///
/// ⚠️ A barra é `5e-4` e não `1e-5`: o que sobra é acumulação de `f32` ao
/// longo do traço (a escada `x01 → x16` cresce `429×` com a lei parada).
#[test]
fn com_o_pente_desligado_a_lei_bate_a_dele_a_uma_passagem() {
    let c = ler("mecanismo", "x_man_x01_p000");
    assert_eq!(c.num("pente"), 0.0, "esta célula é a do pente DESLIGADO");
    let nosso = correr(&c);
    let d = maior_distancia(&nosso, &c.saida);
    assert!(
        movidos(&c, &nosso) > 100,
        "o controlo tem de MOVER barro, senão mede o nada"
    );
    assert!(
        d <= 5e-4,
        "o controlo do arnês reprovou: {d:.3e} — se ele está vermelho, o \
         defeito NÃO é do pente"
    );

    // E a metade que nomeia o contraste: com o pente LIGADO, a mesma
    // passagem, a mesma malha, o mesmo traço — duas ordens de grandeza pior.
    let on = ler("mecanismo", "x_man_x01_p100");
    let d_on = maior_distancia(&correr(&on), &on.saida);
    assert!(
        d_on > d * 50.0,
        "a assinatura do achado é o CONTRASTE entre as duas colunas: \
         {d_on:.3e} com o pente contra {d:.3e} sem ele"
    );
}

/// ⭐⭐ **A consequência OBSERVÁVEL da lei do interruptor, e ela é mais forte
/// que um número:** com a topologia dinâmica desarmada o pente é inerte, logo
/// as duas células do par `c_nodyn` — uma com o pente a `0`, outra a `1` — têm
/// de dar o **MESMO** bloco de posições.
///
/// ⛔ O oráculo diz o mesmo pelo lado dele (`max |p100 − p000| = 0,000e0`), e
/// é essa concordância que torna isto um porte e não uma cerca inventada.
///
/// ⚠️ **Enquanto a lei viveu na crate da app, este gate era inexprimível
/// aqui** e a bancada media um caminho que o produto não percorre.
#[test]
fn com_o_interruptor_desarmado_o_par_da_o_mesmo_bloco() {
    let off = ler("porta", "c_nodyn_p000");
    let on = ler("porta", "c_nodyn_p100");
    assert_eq!(off.chave("topologia_dinamica"), "desarmada");
    assert_eq!(
        on.num("pente"),
        1.0,
        "a outra metade do par pede pente CHEIO"
    );

    let (a, b) = (correr(&off), correr(&on));
    assert!(
        movidos(&off, &a) > 100,
        "o controlo tem de MOVER barro, senão o par é igual por ser vazio"
    );
    assert_eq!(
        a, b,
        "o pente não pode mover nada com o interruptor desarmado"
    );
    assert_eq!(
        maior_distancia(&off.saida, &on.saida),
        0.0,
        "e o ORÁCULO diz o mesmo — se isto reprovar, o porte é que está errado"
    );
}
