//! ⭐⭐⭐ **OS GATES DO MODO ABSOLUTO** ([`Especie::Alvo`], F29 — ordem do dono de 2026-09-19).
//!
//! ⚠️ **Irmão do [`super::correccao_tests`] por ASSUNTO e não por tecto:** aquele mede a mancha que
//! SOMA (o modo de sempre, aprovado em smoke) e este a que FIXA. *Duas leis no mesmo ficheiro
//! partilham fixturas, e uma fixtura partilhada por duas leis acaba a servir mal as duas.*
//!
//! ⛔ **O que este ficheiro NÃO mede está medido ao lado:** o no-op da lista vazia, a bossa e a
//! renormalização são do irmão, e correm nas DUAS espécies pela mesma porta.

use super::*;

/// **TRÊS ossos rectos em fila sobre o `+X`**, cada um com alcance `1` e tendão próprio.
///
/// ⚠️⚠️ **TRÊS e não dois, e a razão é a lei desta wave:** o modo absoluto prende o osso em mãos e
/// reparte `1 − v` pelos **outros mantendo a proporção entre eles** — com um outro só não há
/// proporção nenhuma para preservar, e *uma fixtura que não contém o fenómeno não prova nada sobre
/// ele*. (É a mesma correcção que a fixtura do irmão pagou ao passar de um osso para dois.)
fn tres_ossos() -> Skin {
    let bone = |k: usize| SkinBone {
        rest_a: [k as f64, 0.0],
        rest_b: [k as f64 + 1.0, 0.0],
        radius: 1.0,
        pose: Xform::IDENTITY,
        sub: (0, 1),
        tendon: u32::try_from(k).unwrap_or(0),
    };
    Skin::new(vec![bone(0), bone(1), bone(2)]).expect("a pele nasce")
}

/// O ponto onde os três ossos mandam ao mesmo tempo.
const MEIO: [f64; 2] = [1.5, 0.2];

/// ⭐ **A tabela do padrão-ouro que dá aos três pesos DIFERENTES** — é dela que sai a proporção que
/// o modo absoluto tem de preservar. ⛔ Três pesos iguais tornariam a régua da proporção vácua.
const TABELA: [f64; 3] = [0.2, 0.3, 0.5];

fn pesos(pele: &Skin, p: [f64; 2], cs: &[Correccao]) -> Vec<f64> {
    let mut w = pele.scratch();
    pele.weights_corrected(p, Some(&TABELA), &mut w, cs);
    w
}

fn alvo(tendon: u32, centro: [f64; 2], raio: f64, v: f64) -> Correccao {
    Correccao {
        tendon,
        centro,
        raio,
        especie: Especie::Alvo(v),
    }
}

fn soma(tendon: u32, centro: [f64; 2], raio: f64, v: f64) -> Correccao {
    Correccao {
        tendon,
        centro,
        raio,
        especie: Especie::Soma(v),
    }
}

/// ⭐⭐⭐ **NO CENTRO, O PESO É EXACTAMENTE O QUE O ARTISTA PEDIU** — a frase do dono, à letra: *«o
/// valor de Brush Strength é posto imediatamente no osso em mãos»*.
///
/// ⚠️ **A barra é `1e-12` e não um epsilon generoso:** no centro a bossa vale `1` exactamente
/// (`(1 − 0)² = 1`), logo a mistura degenera no valor pedido **sem arredondamento nenhum**. *Uma
/// barra frouxa aqui aceitaria uma lei que quase põe o valor, que é outra lei.*
///
/// ⛔ **E a metade do CONTROLO é obrigatória:** sem ela, uma tabela em que o osso já tivesse `0,6`
/// deixaria este gate verde sobre uma mancha inerte.
#[test]
fn uma_absoluta_poe_o_peso_pedido_no_centro() {
    let pele = tres_ossos();
    let base = pesos(&pele, MEIO, &[]);
    assert!(
        (base[0] - 0.6).abs() > 0.1,
        "a fixtura ja' nasce perto do alvo ({}) e o gate nao mede nada",
        base[0]
    );
    let w = pesos(&pele, MEIO, &[alvo(0, MEIO, 0.5, 0.6)]);
    assert!(
        (w[0] - 0.6).abs() < 1e-12,
        "o centro devia ler exactamente 0,6 e leu {}",
        w[0]
    );
}

/// ⭐⭐⭐ **O RESTO REPARTE-SE PELOS OUTROS MANTENDO A PROPORÇÃO ENTRE ELES** — a segunda metade da
/// ordem do dono, e a que separa este modo da renormalização que já existia.
///
/// ⛔⛔ **É isto que a normalização NÃO faz:** ela divide **todos** pela soma, incluindo o osso em
/// mãos — o peso pedido sairia diminuído. Aqui o osso fica **preso** e só os outros escalam.
///
/// ⚠️ **As três metades:** o osso em mãos tem o valor, a soma continua `1` (senão a mistura encolhe
/// o ponto para a origem — ver [`Skin::blend`]) e a **razão entre os outros dois** é a de antes.
#[test]
fn o_resto_reparte_se_pelos_outros_na_proporcao_deles() {
    let pele = tres_ossos();
    let antes = pesos(&pele, MEIO, &[]);
    let razao_antes = antes[1] / antes[2];
    let w = pesos(&pele, MEIO, &[alvo(0, MEIO, 0.5, 0.6)]);

    let total: f64 = w.iter().sum();
    assert!(
        (total - 1.0).abs() < 1e-12,
        "os pesos deixaram de somar 1 ({total}) — a mistura encolheria o ponto"
    );
    assert!(
        (w[1] / w[2] - razao_antes).abs() < 1e-12,
        "a proporcao entre os OUTROS mudou: {razao_antes} -> {}",
        w[1] / w[2]
    );
    // ⚠️ E eles de facto encolheram — senão a razão preservada seria a de uma mancha inerte.
    assert!(
        w[1] < antes[1] - 1e-9 && w[2] < antes[2] - 1e-9,
        "os outros nao cederam espaco: {antes:?} -> {w:?}"
    );
}

/// ⭐⭐⭐ **DUAS ABSOLUTAS SOBREPOSTAS: VENCE A ÚLTIMA** — ordem do dono, 2026-09-19: *«a última
/// manda (é o comportamento normal de um pincel absoluto)»*.
///
/// ⚠️⚠️ **A lei não precisa de bookkeeping nenhum, e é por isso que ela cabe numa frase:** as
/// manchas aplicam-se **em sequência** e fixar é idempotente, logo no CENTRO da última a bossa vale
/// `1` e o valor dela apaga tudo o que veio antes. *A ordem da lista passou a ter significado, e é
/// só isso.*
///
/// ⛔ **As duas metades são obrigatórias:** sem a segunda (a mesma lista ao contrário responde
/// outra coisa), uma lei que ignorasse a segunda mancha passaria a primeira.
#[test]
fn duas_absolutas_sobrepostas_a_ultima_manda() {
    let pele = tres_ossos();
    let (ca, cb) = ([1.5, 0.2], [1.8, 0.2]);
    let a = alvo(0, ca, 0.6, 0.9);
    let b = alvo(0, cb, 0.6, 0.1);

    // (a) No centro de B, com B em ÚLTIMO, o peso é o de B — exacto.
    let w = pesos(&pele, cb, &[a, b]);
    assert!(
        (w[0] - 0.1).abs() < 1e-12,
        "no centro da ultima o peso devia ser 0,1 e leu {}",
        w[0]
    );
    // (b) A MESMA lei do outro lado: com A em último, o centro de A lê o valor de A.
    let outra = pesos(&pele, ca, &[b, a]);
    assert!(
        (outra[0] - 0.9).abs() < 1e-12,
        "com A em ultimo o centro de A devia ler 0,9 e leu {}",
        outra[0]
    );

    // (c) ⭐⭐⭐ **O CONTROLO, e é ele o discriminador:** no centro de A, com A em PRIMEIRO, o peso
    // **não** é o de A — a mancha seguinte puxa-o. ⛔ Sem esta metade, uma lei em que cada mancha
    // simplesmente «manda no próprio centro» passaria as duas de cima, e ela não é a que o dono
    // pediu: *a última manda*, não *cada uma manda na sua*.
    //
    // ⚠️⚠️ **A 1.ª redacção deste gate media o centro de B nas duas ordens e esperava `0,9` na
    // invertida — e o portão apanhou-a.** No centro de B a bossa de A vale `0,5625` e não `1`, logo
    // ali A **mistura** em vez de fixar: o número certo era `0,55`. *Uma régua de recência tem de
    // ser lida no centro de quem se quer ver ganhar.*
    let primeiro = pesos(&pele, ca, &[a, b]);
    assert!(
        (primeiro[0] - 0.9).abs() > 0.3,
        "a mancha seguinte nao puxou o centro da anterior ({}) — a lista nao e' aplicada em ORDEM",
        primeiro[0]
    );
}

/// ⭐⭐⭐ **COM OS OUTROS A ZERO, O OSSO FICA COM 100% — DECISÃO DO DONO** (2026-09-20: *«o osso
/// fica com 100% independente do valor»*).
///
/// Num ponto que só o osso em mãos governa não há por onde repartir o `1 − v`, e as duas saídas
/// honestas eram *«o pincel não faz nada ali»* e *«o `v` é ignorado e o peso fica 1»*. Ele escolheu
/// a segunda.
///
/// ⭐⭐⭐ **E o «INDEPENDENTE DO VALOR» só é observável em `v = 0` — MEDIDO por uma mutação
/// SOBREVIVENTE.** A 1.ª redacção deste gate pedia `0,3` e a mutação que apaga a regra do dono
/// passava: com `Σoutros == 0` a renormalização final **devolve `1` a qualquer valor positivo**
/// (`w = [0,3, 0, 0]` soma `0,3` e divide-se por si mesma). ⇒ *a única posição do curso em que a
/// regra dele decide alguma coisa é o ZERO*, e ali as duas leituras são opostas: com a regra o osso
/// fica com tudo, sem ela o ponto fica **sem dono** e a arte congela ali.
///
/// ⚠️ **Nos outros valores ela é um NO-OP**, e é isso que a torna segura — a metade `(a)` mede-o.
///
/// ⭐⭐⭐ **A metade `(c)` é o CONTROLO que mede a outra leitura de `Σoutros == 0`:** um ponto que
/// **ninguém** governa (a tabela guardada não traz peso nenhum). Ali a regra dá a posse ao osso — e
/// a [`Especie::Soma`] **já fazia exactamente o mesmo** no mesmo arranjo, porque ela soma num vector
/// de zeros e a renormalização final leva o resultado a `1`. *As duas espécies concordam, e é isso
/// que prova que a regra do dono não inventou um regime novo.*
#[test]
fn com_os_outros_a_zero_o_osso_fica_com_cem_por_cento() {
    let pele = tres_ossos();
    let so_um = [1.0, 0.0, 0.0];
    let com = |tabela: &[f64], c: Correccao| {
        let mut w = pele.scratch();
        pele.weights_corrected(MEIO, Some(tabela), &mut w, &[c]);
        w
    };

    // (a) Um valor qualquer: NO-OP, e é o que torna a regra segura.
    assert_eq!(
        com(&so_um, alvo(0, MEIO, 0.5, 0.3)),
        vec![1.0, 0.0, 0.0],
        "com os outros a zero o osso devia ficar com 100% (e o valor 0,3 ser ignorado)"
    );

    // (b) ⭐ O ZERO — a única posição em que a regra do dono é observável.
    assert_eq!(
        com(&so_um, alvo(0, MEIO, 0.5, 0.0)),
        vec![1.0, 0.0, 0.0],
        "pedir peso ZERO onde nao ha' por onde repartir deixou o ponto SEM DONO — a regra do dono \
         diz «o osso fica com 100% independente do valor», e o zero e' o unico sitio onde ela decide"
    );

    // (c) A outra leitura de `Σoutros == 0`: NINGUÉM governa. As duas espécies concordam.
    let ninguem = [0.0, 0.0, 0.0];
    let abs = com(&ninguem, alvo(0, MEIO, 0.5, 0.3));
    let cum = com(&ninguem, soma(0, MEIO, 0.5, 0.3));
    assert_eq!(
        abs, cum,
        "as duas especies discordam num ponto sem dono: {abs:?} contra {cum:?}"
    );
    assert_eq!(
        abs,
        vec![1.0, 0.0, 0.0],
        "num ponto sem dono o osso pintado devia ficar com ele"
    );
}

/// ⭐⭐⭐ **A ABSOLUTA NÃO ESTALA NA BORDA** — a bossa entra como uma MISTURA (`lerp(actual, v,
/// bump)`) e não como um multiplicador.
///
/// ⛔⛔ **Um `v · bump` daria peso ZERO na borda da mancha**: o artista pediria `0,8` e receberia um
/// buraco à volta, que é exactamente o estalo que a continuidade C¹ da casa existe para não ter.
///
/// ⚠️ **A régua é a SEGUNDA DIFERENÇA da contribuição da mancha** ao atravessar a borda — a mesma
/// do irmão, e pela mesma razão: o peso final carrega a curvatura da lei de base, que afogaria o
/// que se quer medir.
#[test]
fn a_absoluta_nao_estala_na_borda() {
    let pele = tres_ossos();
    let raio = 0.4;
    let c = [alvo(0, [1.5, 0.0], raio, 0.8)];
    let em = |x: f64| {
        let p = [x, 0.0];
        pesos(&pele, p, &c)[0] - pesos(&pele, p, &[])[0]
    };
    let salto = |h: f64| {
        let b = 1.5 + raio;
        (em(b - h) + em(b + h) - 2.0 * em(b)).abs()
    };
    let (grosso, fino) = (salto(0.05), salto(0.0125));
    assert!(
        fino < grosso * 0.5,
        "a borda da mancha ABSOLUTA estala: segunda diferenca {fino:.9} a passo fino contra \
         {grosso:.9} a passo grosso — num bump C¹ ela tinha de encolher com o passo"
    );
    // ⚠️ E ela tem de FAZER alguma coisa no centro, senão o teste acima é sobre o nada.
    assert!(
        em(1.5).abs() > 0.1,
        "a mancha nao move nada no centro ({}) e a regua da borda mede o vazio",
        em(1.5)
    );
}

/// ⭐⭐ **UMA `Soma` DEPOIS DE UMA `Alvo` SOMA POR CIMA DELA** — a lei sequencial, vista do lado em
/// que as duas espécies se compõem.
///
/// ⚠️ **É o que faz o modo cumulativo continuar a ser cumulativo num desenho já corrigido à mão:**
/// o artista fixa `0,5` num ponto e depois empurra mais um pouco, e o *mais um pouco* parte de
/// `0,5` — não do que a lei automática dava.
#[test]
fn uma_soma_depois_de_uma_absoluta_soma_por_cima() {
    let pele = tres_ossos();
    let fixo = pesos(&pele, MEIO, &[alvo(0, MEIO, 0.5, 0.5)]);
    let mais = pesos(
        &pele,
        MEIO,
        &[alvo(0, MEIO, 0.5, 0.5), soma(0, MEIO, 0.5, 0.2)],
    );
    assert!(
        mais[0] > fixo[0] + 0.05,
        "a soma nao empurrou por cima da fixacao: {} -> {}",
        fixo[0],
        mais[0]
    );
    // ⛔ E a ORDEM oposta apaga a soma, que é a lei e não um defeito: fixar vem depois.
    let apagada = pesos(
        &pele,
        MEIO,
        &[soma(0, MEIO, 0.5, 0.2), alvo(0, MEIO, 0.5, 0.5)],
    );
    assert!(
        (apagada[0] - 0.5).abs() < 1e-12,
        "com a fixacao em ultimo o centro devia ler 0,5 e leu {}",
        apagada[0]
    );
}

/// ⭐⭐⭐ **UMA `Alvo` MEDE-SE SOBRE PESOS JÁ NORMALIZADOS** — e isto foi achado por uma mutação
/// SOBREVIVENTE.
///
/// A `Alvo` raciocina em `1 − v`, logo precisa que `Σw = 1`; uma `Soma` que a preceda deixa a soma
/// fora de `1`, e é por isso que a lei normaliza **antes** dela.
///
/// ⚠️⚠️ **E isso é invisível NO CENTRO, que é onde a 1.ª redacção media:** ali a bossa vale `1`, o
/// osso é **pregado** no valor pedido e o `escala = (1 − novo)/Σoutros` re-normaliza os outros para
/// `1 − novo` *seja qual for* a soma de onde partiram — as duas leis dão o mesmo número. ⇒ o
/// discriminador é a leitura **FORA do centro**, onde o `lerp` parte de *quão longe o ponto já
/// está*, e um `w_t` não-normalizado torna essa distância maior do que é.
///
/// ⭐ **A referência é RE-DERIVADA pela porta pública:** os pesos pós-`Soma` (já normalizados) são
/// re-injectados como TABELA, e a `Alvo` corre sozinha por cima deles. *Num rig de ossos rectos a
/// [`Skin::weights_from`] devolve a tabela ao bit*, logo os dois caminhos só podem divergir na
/// grandeza que se quer medir.
#[test]
fn uma_absoluta_mede_se_sobre_pesos_ja_normalizados() {
    let pele = tres_ossos();
    let (s, a) = (soma(0, MEIO, 0.6, 0.3), alvo(0, MEIO, 0.6, 0.5));
    // FORA do centro das duas, e dentro do alcance das duas.
    let p = [1.7, 0.2];

    let juntas = pesos(&pele, p, &[s, a]);
    let apos_soma = pesos(&pele, p, &[s]);
    let mut separadas = pele.scratch();
    pele.weights_corrected(p, Some(&apos_soma), &mut separadas, &[a]);

    assert!(
        (juntas[0] - separadas[0]).abs() < 1e-12,
        "a absoluta mediu-se sobre uma soma NAO normalizada: {} contra {}",
        juntas[0],
        separadas[0]
    );
    // ⚠️ E a fixtura tem de conter o fenómeno: a soma sozinha muda mesmo o peso ali, e a leitura é
    // fora do centro (senão o `escala` esconde tudo — ver o doc acima).
    let base = pesos(&pele, p, &[]);
    assert!(
        (apos_soma[0] - base[0]).abs() > 0.02,
        "a soma nao mexe em {p:?} ({} -> {}) e este gate mede o nada",
        base[0],
        apos_soma[0]
    );
    assert!(
        (juntas[0] - 0.5).abs() > 0.01,
        "o ponto esta' no centro da absoluta ({}) — ali a fixacao e' total e a normalizacao \
         pendente e' inobservavel",
        juntas[0]
    );
}

/// ⭐⭐⭐ **UMA LISTA SÓ DE `Soma` DÁ EXACTAMENTE O QUE A LEI DE SEMPRE DAVA** — a garantia que
/// protege o smoke que o dono já aprovou.
///
/// ⚠️⚠️ **A referência é RE-DERIVADA aqui à mão**, e não uma chamada à mesma função: *um gate que
/// compara uma função consigo mesma não afirma nada*. A conta escrita abaixo é a lei pré-F29 —
/// acumular `delta · bump · quota` com saturação e normalizar **uma vez no fim** —, e a barra é a
/// igualdade **ao bit**.
///
/// ⛔ Se alguém puser a normalização a correr entre as `Soma` (o que a espécie `Alvo` precisa), este
/// gate reprova: a ordem das divisões muda os últimos bits.
#[test]
fn uma_lista_so_de_soma_e_byte_identica_a_lei_de_sempre() {
    let pele = tres_ossos();
    let cs = [
        soma(0, [1.4, 0.0], 0.6, 0.3),
        soma(1, [1.6, 0.0], 0.6, -0.2),
        soma(0, [1.5, 0.1], 0.5, 0.25),
    ];
    for p in [MEIO, [1.4, 0.0], [1.7, -0.1], [0.5, 0.0]] {
        // A referência: a lei de base, depois o acumulador, depois UMA normalização.
        let mut esperado = pele.scratch();
        pele.weights_from(p, &TABELA, &mut esperado);
        let mut mexeu = false;
        for c in &cs {
            let d2 = (p[0] - c.centro[0]).powi(2) + (p[1] - c.centro[1]).powi(2);
            let x2 = d2 / (c.raio * c.raio);
            if x2 >= 1.0 {
                continue;
            }
            let t = 1.0 - x2;
            let bump = t * t;
            let Especie::Soma(delta) = c.especie else {
                unreachable!("esta fixtura e' toda cumulativa")
            };
            // ⚠️ Os ossos desta fixtura são rectos ⇒ a quota é `1.0` AO BIT, e escrevê-la seria
            // repetir a lei em vez de a medir.
            for (i, b) in [0usize, 1, 2].iter().enumerate() {
                if u32::try_from(*b).unwrap_or(0) != c.tendon {
                    continue;
                }
                esperado[i] = (delta * bump).mul_add(1.0, esperado[i]).clamp(0.0, 1.0);
                mexeu = true;
            }
        }
        if mexeu {
            let s: f64 = esperado.iter().sum();
            if s > 0.0 {
                for v in &mut esperado {
                    *v /= s;
                }
            }
        }
        assert_eq!(
            pesos(&pele, p, &cs),
            esperado,
            "a lei CUMULATIVA deixou de ser byte-identica em {p:?}"
        );
    }
}

/// ⚠️ **Uma absoluta com valor absurdo é SALTADA** — a mesma cerca do irmão, e pela mesma razão: um
/// ficheiro editado à mão chega aqui, e a resposta certa é não fazer nada em vez de escrever `NaN`
/// na arte inteira.
///
/// ⭐ **E um valor FORA de `0..1` é coagido, não saltado:** ele é um peso, e `1,5` quer dizer *«tudo
/// para este osso»*. *Saltar seria um pincel que não faz nada e não diz porquê.*
#[test]
fn uma_absoluta_degenerada_e_saltada_e_uma_fora_da_faixa_e_coagida() {
    let pele = tres_ossos();
    let base = pesos(&pele, MEIO, &[]);
    assert_eq!(
        pesos(&pele, MEIO, &[alvo(0, MEIO, 0.5, f64::NAN)]),
        base,
        "uma absoluta com valor NaN mexeu nos pesos"
    );
    let cheio = pesos(&pele, MEIO, &[alvo(0, MEIO, 0.5, 1.5)]);
    assert!(
        (cheio[0] - 1.0).abs() < 1e-12,
        "um alvo de 1,5 devia ser coagido a 1 e leu {}",
        cheio[0]
    );
}
