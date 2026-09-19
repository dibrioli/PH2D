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
    // ⭐ As oito da ROTAÇÃO (2026-09-18) trazem um CONTROLO dentro: com o
    // pente desligado as quatro leem `1,6e-4`, ou seja a lei base é
    // rotação-invariante e certa — **toda** a dependência da rotação está no
    // pente, e o erro dele DOBRA fora do eixo (`2,8e-2` a `0°` contra
    // `6,1e-2` a `67,5°`). É o termo que o ajuste do §78 não explica.
    ("rotacao/m_a0000_p000", 1.624e-4),
    ("rotacao/m_a0000_p100", 2.815e-2),
    ("rotacao/m_a0225_p000", 1.609e-4),
    ("rotacao/m_a0225_p100", 5.382e-2),
    ("rotacao/m_a0450_p000", 1.603e-4),
    ("rotacao/m_a0450_p100", 6.048e-2),
    ("rotacao/m_a0675_p000", 1.620e-4),
    ("rotacao/m_a0675_p100", 6.146e-2),
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

    let mut fora = 0usize;
    for (fam, nome) in celulas_sem_remalha() {
        let k = chave(&fam, &nome);
        let c = ler(&fam, &nome);

        // ⛔⛔ A família `acumula/` NÃO é do pente — ela varia o interruptor de
        // acumulação e o número de carimbos, com o pente a ZERO. Pô-la neste
        // placar mediria outra coisa com esta régua.
        //
        // ⚠️ **A exclusão é VERIFICADA e não uma condição de nome:** cada
        // célula saltada tem de provar que o pente lhe é inerte, senão um
        // ficheiro novo naquela pasta desaparecia deste placar em silêncio —
        // que é o censo a varrer menos e a ficar verde.
        if fam == "acumula" {
            assert_eq!(
                c.num("pente"),
                0.0,
                "{k}: saltada por ser da família da acumulação, e ela tem o \
                 pente a {} — se uma célula dali passar a exercitar o pente, \
                 ela pertence a este placar",
                c.num("pente")
            );
            fora += 1;
            continue;
        }

        vistas += 1;

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
        fora >= 18,
        "a família da acumulação tem de continuar a ser saltada e CONTADA \
         ({fora} saltadas) — se ela encolher, alguém apagou corpus"
    );
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

/// ⛔⛔ **AFIRMA UMA DIFERENÇA DE PROPÓSITO — e ela é do REGIME DO CORPUS.**
///
/// ⚠️⚠️ **CORRIGIDO (2026-09-18):** a 1.ª redacção chamava a isto *«a
/// contradição do §75»* e lia-o como defeito do nosso pincel base. Não é —
/// ou pelo menos o corpus não o prova. Estas células carimbam com o centro
/// **PREGADO** (o alvo lê a posição 3D e não relança raio, medido), e o nosso
/// produto **repica da superfície viva**. No regime do corpus a nossa lei
/// diverge; no regime do produto ela é a que os gates de `accum_tests`
/// descrevem, e esses continuam verdes.
///
/// *Um número medido num regime que o produto não corre é um facto sobre a
/// bancada.*
///
/// Com o pente **INERTE** (`acumular = False` em todo o corpus), o nosso
/// pincel base carimba na direcção EXACTA sobre os vértices EXACTOS, e cava
/// **fundo de mais**, com o excesso a crescer com o número de carimbos sobre o
/// mesmo vértice:
///
/// | célula | carimbos | `cos` | nós/ele |
/// |---|---|---|---|
/// | `y_umdab` | `1` | `1,000` | **`1,000`** |
/// | `y_doisdab` | `2` | `1,000` | `1,024` |
/// | `y_parado` | `14` | `1,000` | **`1,226`** |
///
/// ⭐ A causa tem endereço: o `GripLaw::from_live` do [`crate::Grip::Stamp`] é
/// `= accumulate`. Forçado a `true`, `y_parado` passa a `0,994` e **catorze**
/// células do placar caem para ruído de `f32`, com zero regressões.
///
/// ⛔⛔ **E não foi mudado, porque DUAS fontes desta casa discordam:** o gate
/// `the_disarmed_brush_saturates_at_one_radius_and_the_armed_one_passes_it`
/// declara que com o interruptor DESLIGADO o pincel satura, e o corpus — que
/// tem **uma só** metade da tabela-verdade — mede que o alvo não satura assim.
/// O que arbitra são células do oráculo com `acumular = True`, que não
/// existem.
///
/// # ⚠️ Porque este gate afirma o defeito
///
/// *Um defeito medido que só vive em prosa é re-derivado do zero pela próxima
/// janela.* Preso aqui, ele tem duas metades:
///
/// 1. **o que está CERTO** (direcção exacta, vértices exactos, e o carimbo
///    único exacto ao bit) — se isto reprovar, a cura partiu o que funcionava;
/// 2. **o que está ERRADO** (o excesso a 14 carimbos) — se **isto** reprovar,
///    alguém resolveu a contradição, e a premissa deste gate morre à vista no
///    diff, que é como ela deve morrer.
#[test]
fn o_pincel_base_sobre_acumula_e_isso_e_a_contradicao_do_p75() {
    let mut linhas = Vec::new();
    for (base, carimbos) in [("y_umdab", 1usize), ("y_doisdab", 2), ("y_parado", 14)] {
        let c = ler("mecanismo", &format!("{base}_p000"));
        assert_eq!(
            c.percurso.len(),
            carimbos,
            "{base}: a fixtura mudou de número de carimbos — a tabela do §75 \
             deixou de a descrever"
        );
        let dentro = entrada(&c);
        let nosso = correr(&c);
        let (mut num, mut den, mut rn, mut re) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let mut movidos = 0usize;
        for ((p, a), b) in dentro.positions().iter().zip(&c.saida).zip(&nosso) {
            let d = |q: &[f32; 3]| {
                [
                    f64::from(q[0] - p[0]),
                    f64::from(q[1] - p[1]),
                    f64::from(q[2] - p[2]),
                ]
            };
            let (u, v) = (d(a), d(b));
            let n = |w: &[f64; 3]| w[0].hypot(w[1]).hypot(w[2]);
            let (nu, nv) = (n(&u), n(&v));
            if nu > 1e-9 {
                movidos += 1;
                if nv > 1e-9 {
                    num += (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]) / (nu * nv) * nu;
                    den += nu;
                }
            }
            re += nu;
            rn += nv;
        }
        assert!(
            movidos > 40,
            "{base}: o oráculo tem de MOVER barro, senão este gate mede o nada \
             ({movidos} vértices)"
        );
        let (cos, razao) = (num / den, rn / re);
        linhas.push((base, carimbos, cos, razao));

        // (1) O QUE ESTÁ CERTO — a direcção. Se isto cair, a cura partiu o
        //     que já funcionava.
        assert!(
            cos > 0.999,
            "{base}: a direcção do pincel base era EXACTA (cos 1,000) e agora \
             lê {cos:.4} — quem mexeu partiu o que estava certo (§75.1)"
        );
    }

    let razao = |n: &str| linhas.iter().find(|l| l.0 == n).unwrap().3;

    // (1-bis) Um carimbo é exacto ao bit — é o controlo que prova que o erro é
    //         da ACUMULAÇÃO e não da lei do carimbo.
    let um = razao("y_umdab");
    assert!(
        (um - 1.0).abs() < 1e-4,
        "y_umdab: UM carimbo tem de ser exacto ({um:.4}) — sem este controlo o \
         excesso dos outros podia ser da lei do carimbo, não da acumulação"
    );

    // (2) O DEFEITO, afirmado de propósito. Se isto reprovar, alguém resolveu
    //     a contradição do §75 — leia-o e mate esta premissa no diff.
    let catorze = razao("y_parado");
    assert!(
        catorze > 1.15,
        "y_parado: o excesso de acumulação era {:.3} e agora lê {catorze:.3}. \
         Se alguém o CUROU, a contradição do §75 foi resolvida: leia o handoff \
         §75, confirme com que metade da tabela-verdade ela foi arbitrada, e \
         APAGUE esta metade do gate com a premissa morta à vista no diff",
        1.226
    );
    assert!(
        catorze > razao("y_doisdab"),
        "o excesso tem de CRESCER com os carimbos ({catorze:.3} a 14 contra \
         {:.3} a 2) — é essa monotonia que o identifica como acumulação",
        razao("y_doisdab")
    );
}

/// ⭐⭐⭐ **A identidade: o nosso ramo LIGADO é o ramo DESLIGADO dele.**
///
/// # ⛔⛔⛔ CORRIGIDO (2026-09-18) — isto NÃO arbitra a polaridade do produto
///
/// A 1.ª redacção deste gate lia-se como *«a nossa polaridade está invertida,
/// prova exacta»*. **É falso**, e a refutação veio do arnês do oráculo:
///
/// São **DUAS** escolhas binárias independentes — **(A)** a polaridade da
/// curva de queda e **(B)** de que posição a distância é medida (viva ou
/// congelada) —, e as duas experiências que eu corri **variaram-nas JUNTAS**:
/// *cursor fixo com alvo vivo* e *cursor vivo com alvo congelado* são a
/// **mesma geometria relativa**. Daí a identidade ao bit, e daí eu quase ter
/// trocado uma lei certa.
///
/// ⛔⛔ **E o corpus não mede o regime do PRODUTO.** O arnês entrega ao alvo a
/// posição **3D** (medido: três células com o mesmo pixel e `z = +2 · 0 · −2`
/// dão `0` · `49` · `0` vértices movidos ⇒ ele lê a posição e **não relança
/// raio**), logo o centro fica **PREGADO** nas `N` repetições. O nosso produto
/// **repica da superfície viva** a cada carimbo. *Comparar um produto que
/// repica contra células de centro pregado é variar duas coisas de uma vez.*
///
/// ⇒ o que este gate afirma é uma **propriedade da nossa lei no regime do
/// corpus**, e **não** um veredito sobre o interruptor. ⚠️ Com o centro
/// pregado, a lei do alvo mede a distância da posição **VIVA** com a curva
/// **não** invertida (ajuste do E: erro `0,00 %` nos treze `N`).
///
/// A família `acumula/` (18 células, colhida 2026-09-18) trouxe a metade da
/// tabela-verdade que faltava, e ela diz que a nossa polaridade está
/// **invertida**. A prova não é estatística — é uma **identidade a oito casas
/// em sete contagens de carimbo**:
///
/// | `N` | ALVO desligado | ALVO ligado | NOSSO ligado |
/// |---|---|---|---|
/// | 1 | `0,08736818` | `0,08736818` | `0,08736818` |
/// | 2 | `0,16113299` | `0,16113299` | `0,16113299` |
/// | 4 | `0,24074651` | **`0,21000004`** | `0,24074651` |
/// | 8 | `0,29316160` | **`0,21000004`** | `0,29316160` |
/// | 14 | `0,31712657` | **`0,21000004`** | `0,31712657` |
/// | 27 | `0,33279914` | **`0,21000004`** | `0,33279914` |
/// | 40 | `0,33833069` | **`0,21000004`** | `0,33833069` |
///
/// # ⚠️ O que este gate protege, e quando ele DEVE reprovar
///
/// Ele tem **três** metades, e a terceira é sobre o CORPUS e não sobre nós:
///
/// 1. **a identidade** — o nosso ramo ligado reproduz o desligado dele. ⛔ No
///    dia em que a polaridade for trocada, ela passa a valer para o nosso ramo
///    DESLIGADO e este gate reprova: *é assim que ele deve morrer*, com a
///    mensagem a dizer o que fazer;
/// 2. **o travão dele** — o lado ligado do alvo é um ponto fixo de `N = 4` a
///    `N = 40`. Se isto cair, o corpus foi corrompido ou recolhido de outro
///    binário;
/// 3. **o nosso travão é OUTRO** (`5 ×` o primeiro carimbo, contra `2,40 ×`
///    dele) — a dívida que sobra, afirmada de propósito para não se perder.
///
/// ⚠️ **As contagens saem do CORPUS**, nunca de uma lista escrita à mão.
#[test]
fn o_nosso_ramo_ligado_e_o_ramo_desligado_do_alvo() {
    let mut ns: Vec<usize> = celulas_sem_remalha()
        .into_iter()
        .filter(|(f, n)| f == "acumula" && n.starts_with("escada_n") && n.ends_with("_on"))
        .filter_map(|(_, n)| n["escada_n".len()..n.len() - 3].parse().ok())
        .collect();
    ns.sort_unstable();
    assert!(
        ns.len() >= 7,
        "a escada encolheu para {} contagens — alguém apagou corpus",
        ns.len()
    );

    let maior = |c: &crate::oraculo_do_pente::Celula, p: &[[f32; 3]]| -> f64 {
        entrada(c)
            .positions()
            .iter()
            .zip(p)
            .map(|(a, b)| {
                f64::from(b[0] - a[0])
                    .hypot(f64::from(b[1] - a[1]))
                    .hypot(f64::from(b[2] - a[2]))
            })
            .fold(0.0, f64::max)
    };

    let mut travao: Vec<f64> = Vec::new();
    let mut nosso_desligado: Vec<(usize, f64)> = Vec::new();
    for n in ns {
        let off = ler("acumula", &format!("escada_n{n}_off"));
        let on = ler("acumula", &format!("escada_n{n}_on"));
        let dele_off = maior(&off, &off.saida);
        let dele_on = maior(&on, &on.saida);
        let nosso_on = maior(&on, &correr(&on));
        nosso_desligado.push((n, maior(&off, &correr(&off))));

        // (1) A IDENTIDADE — a oito casas, que é a precisão em que ela foi
        //     medida. ⛔ Se reprovar, leia a mensagem: ela diz o que mudou.
        assert!(
            (nosso_on - dele_off).abs() < 5e-8,
            "N={n}: o NOSSO ramo ligado ({nosso_on:.8}) deixou de reproduzir o \
             ramo DESLIGADO do alvo ({dele_off:.8}). Se a polaridade do \
             `Grip::Stamp` foi TROCADA — que é a cura medida no handoff §76 — \
             esta identidade passou para o nosso ramo DESLIGADO: reescreva este \
             gate com a premissa morta à vista no diff"
        );
        if n >= 4 {
            travao.push(dele_on);
        }
    }

    // (2) O travão DELE é um ponto fixo — propriedade do CORPUS.
    let (lo, hi) = (
        travao.iter().cloned().fold(f64::MAX, f64::min),
        travao.iter().cloned().fold(0.0, f64::max),
    );
    assert!(
        travao.len() >= 5 && hi - lo < 1e-7,
        "o lado LIGADO do alvo deixou de ser um ponto fixo de N>=4 \
         ({lo:.8}..{hi:.8} em {} contagens) — o corpus foi corrompido ou veio \
         de outro binário",
        travao.len()
    );

    // (3) E o NOSSO travão é outro — a dívida que sobra, afirmada de propósito.
    let primeiro = nosso_desligado[0].1;
    let ultimo = nosso_desligado.last().unwrap().1;
    assert!(
        (ultimo / primeiro - 5.0).abs() < 0.01,
        "o nosso ramo desligado travava em 5,0000x o primeiro carimbo e agora \
         trava em {:.4}x ({primeiro:.8} -> {ultimo:.8}). O do alvo trava em \
         2,40x: se alguém escreveu a lei do tecto, apague esta metade",
        ultimo / primeiro
    );
}
