//! Os gates da LEI do pente — a unidade, sem malha e sem pincel.
//!
//! ⚠️ **A régua é a MESMA do corpus do oráculo**, `Q = média(cos 4α)`, escrita
//! aqui sobre o anel de um vértice só: ligar a lei de unidade à régua pela qual
//! o produto é julgado é o que impede as duas de divergirem.
//!
//! # ⛔⛔⛔ A LEI PARTIU-SE EM DUAS em 2026-09-18, e estes gates com ela
//!
//! Até aqui havia **uma** lei — o deslocamento encaixava cada aresta no eixo da
//! grade mais próximo — e estes gates mediam-na. Hoje há duas, em sítios
//! diferentes: o **deslocamento** ([`pentear`]) é uma relaxação ISOTRÓPICA que
//! não lê a direcção do traço, e o **alinhamento** mora no campo de tamanho que
//! o passe de topologia recebe ([`crate::campo_do_pente`]).
//!
//! ⇒ *todo gate deste ficheiro que dizia «a lei alinha» tinha a premissa
//! MORTA*, e está reescrito com a morte à vista. Os que mediam uma propriedade
//! do PASSO (tangencial · carril · degenerescência) sobreviveram como estavam.

use crate::{Porta, V3, pentear};

/// A régua da espec, sobre as arestas de um vértice: `Q = média(cos 4α)`, com
/// `α` o ângulo entre a aresta e a direcção do traço, no plano tangente.
///
/// ⭐ **Ela lê a MESMA função que o produto usa** ([`crate::quatro_dobras`]) —
/// até 18/09 era uma segunda cópia da fórmula, escrita com `atan2` e uma base
/// tangente própria. *Duas escritas da mesma grandeza divergem no dia em que
/// uma delas ganhar uma cerca.*
fn q_do_anel(posicoes: &[V3], v: usize, vizinhos: &[u32], normal: V3, direccao: V3) -> f64 {
    let ao_longo = crate::unitario(crate::sem_componente(direccao, normal)).expect("quadro vivo");
    let mut soma = 0.0f64;
    let mut n = 0u32;
    for &u in vizinhos {
        let plana =
            crate::sem_componente(crate::subtrair(posicoes[u as usize], posicoes[v]), normal);
        if crate::norma(plana) <= 0.0 {
            continue;
        }
        soma += f64::from(crate::quatro_dobras(ao_longo, plana));
        n += 1;
    }
    soma / f64::from(n.max(1))
}

const CIMA: V3 = [0.0, 0.0, 1.0];
const TRACO: V3 = [1.0, 0.0, 0.0];

/// ⭐⭐⭐ **UM ANEL REGULAR É PONTO FIXO** — a propriedade que diz que isto é uma
/// relaxação e não um arrasto.
///
/// ⛔⛔ **O NOME E A RAZÃO deste gate mudaram em 18/09.** Ele chamava-se *«uma
/// grade já ALINHADA não se mexe»* e a razão era que os quatro vizinhos já
/// estavam sobre os eixos do traço. Com a lei isotrópica isso deixou de ser o
/// que ele mede: o que o torna ponto fixo é o **centróide do anel coincidir com
/// o vértice**, e o alinhamento com o traço é irrelevante — *a mesma fixtura
/// rodada de `30°` continua a não se mexer*, e é isso que a segunda metade
/// afirma.
#[test]
fn um_anel_regular_e_ponto_fixo_e_a_direccao_nao_conta() {
    let cruz: Vec<V3> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ];
    // (1) — com o traço sobre um eixo do anel.
    let mut pos = cruz.clone();
    let movidos = pentear(
        &mut pos,
        &[0],
        &[CIMA],
        &[1.0],
        &[0, 4],
        &[1, 2, 3, 4],
        TRACO,
    );
    assert_eq!(movidos, 0, "um anel regular foi mexido");
    assert_eq!(pos, cruz, "a saida nao e' byte-identica a' entrada");

    // (2) — e com o traço a `30°` dele. ⛔ **Sem esta metade o gate lê-se como
    //       «a lei guarda a grade alinhada»**, que é a leitura da lei ANTIGA e
    //       hoje é falsa: ela guarda o anel REGULAR, alinhado ou não.
    let mut pos = cruz.clone();
    let obliquo = [0.866_025, 0.5, 0.0];
    let movidos = pentear(
        &mut pos,
        &[0],
        &[CIMA],
        &[1.0],
        &[0, 4],
        &[1, 2, 3, 4],
        obliquo,
    );
    assert_eq!(movidos, 0, "o mesmo anel mexeu-se com o traco a 30 graus");
    assert_eq!(pos, cruz);
}

/// ⭐⭐⭐ **O DESLOCAMENTO NÃO LÊ A DIRECÇÃO DO TRAÇO — e é esta a frase inteira
/// da wave de 18/09.**
///
/// A mesma vizinhança, com o traço a `0°`, `30°`, `90°` e `217°`: as quatro
/// saídas são **byte-idênticas**. A direcção continua a decidir *SE* o vértice
/// obedece (a degenerescência da espec §4.3, gate abaixo) e deixou de decidir
/// *PARA ONDE* ele anda.
///
/// ⛔⛔ **Com a lei ANTIGA este gate seria impossível:** ela projectava a direcção
/// no plano tangente e encaixava cada aresta no eixo mais próximo, logo rodar o
/// traço trocava os eixos e a saída mudava. *É este o gate que discrimina as
/// duas leis*, e por isso ele existe — as outras fixturas deste ficheiro dão o
/// mesmo número nas duas (ver [`um_anel_torto_relaxa_e_a_conta_fecha`]).
#[test]
fn o_deslocamento_nao_le_a_direccao_do_traco() {
    let arranjo = |direccao: V3| {
        let mut pos: Vec<V3> = vec![
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [-2.0, 0.0, 0.0],
            [0.3, -1.4, 0.0],
        ];
        let movidos = pentear(
            &mut pos,
            &[0],
            &[CIMA],
            &[1.0],
            &[0, 3],
            &[1, 2, 3],
            direccao,
        );
        (movidos, pos[0])
    };
    let (movidos, referencia) = arranjo(TRACO);
    // O controlo POSITIVO: sem ele as comparações abaixo afirmam sobre um
    // vértice que nunca andou.
    assert_eq!(movidos, 1, "o arranjo deixou de conter o fenomeno");
    assert!(
        crate::norma(crate::subtrair(referencia, [0.0, 0.0, 0.0])) > 1e-3,
        "o vertice mal andou ({referencia:?}) — a comparacao seria vacuo"
    );
    for direccao in [
        [0.866_025, 0.5, 0.0],
        [0.0, 1.0, 0.0],
        [-0.798_6, -0.601_9, 0.0],
    ] {
        let (n, onde) = arranjo(direccao);
        assert_eq!(n, 1, "com a direccao {direccao:?} o vertice nao andou");
        assert_eq!(
            onde, referencia,
            "com a direccao {direccao:?} o vertice foi para {onde:?} em vez de \
             {referencia:?} — o deslocamento voltou a ler a direccao do traco"
        );
    }
}

/// ⭐⭐ **UM ANEL TORTO RELAXA, e a conta fecha à mão.**
///
/// Dois vizinhos: um a `45°` do traço (comprimento `√2`) e outro a `180°`
/// (comprimento `2`). O destino é o **centróide** do anel: a média de `(1,1,0)`
/// e `(−2,0,0)` é `(−0,5 , 0,5)`, que mede `0,70711` — **acima do carril**
/// (`0,34 × 1,70711 = 0,58042`) ⇒ o passo é encolhido por `0,82084` e o vértice
/// pára em **`(−0,41042 , 0,41042)`**. Com isso o `Q` do anel sobe de `0,000`
/// para **`+0,259`**: os ângulos passam de `(45°, 180°)` para `(22,69°,
/// 165,53°)`.
///
/// ⚠️ **Esta fixtura exercita a lei E o carril**, e os dois números estão na
/// conta acima — o carril tem gate próprio, e aqui ele aparece porque um anel
/// de duas arestas muito assimétrico pede mesmo um passo grande.
///
/// ⛔⛔⛔ **E ela dá o MESMO destino sob as DUAS leis — é por isso que ela não
/// pode ser a única.** Sob a lei antiga o termo de encaixe era
/// `Σ(raio_do_anel · eixo)`, e aqui os dois eixos são `+x̂` e `−x̂`: eles
/// **cancelam-se ao bit**, e o que sobra é exactamente o centróide. *Uma fixtura
/// que dá o mesmo número antes e depois de uma mudança de lei não afirma nada
/// sobre a mudança* — quem discrimina é o
/// [`o_deslocamento_nao_le_a_direccao_do_traco`].
///
/// ⚠️ **A conta MUDOU de razão, não de número:** a redacção anterior derivava-a
/// dos alvos `±1,70711·x̂`, que já não existem.
#[test]
fn um_anel_torto_relaxa_e_a_conta_fecha() {
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-2.0, 0.0, 0.0]];
    let antes_q = q_do_anel(&pos, 0, &[1, 2], CIMA, TRACO);
    let movidos = pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], TRACO);
    assert_eq!(movidos, 1);
    assert!(
        (pos[0][0] - -0.410_418).abs() < 1e-5 && (pos[0][1] - 0.410_418).abs() < 1e-5,
        "o vertice foi para {:?}, e a conta a' mao da' (-0,41042 , 0,41042)",
        pos[0]
    );
    let depois_q = q_do_anel(&pos, 0, &[1, 2], CIMA, TRACO);
    assert!(
        antes_q.abs() < 1e-6 && (depois_q - 0.2590).abs() < 1e-3,
        "o Q do anel foi de {antes_q:.4} para {depois_q:.4}; a conta a' mao da' 0,000 -> +0,259"
    );
}

/// ⭐ **O passo é TANGENCIAL** — ele não engorda nem come a forma.
///
/// ⚠️⚠️ **E isto é uma DIVERGÊNCIA DECLARADA, não paridade:** o oráculo mede uma
/// componente normal real (razão tangencial/normal `4,37`, espec §3.2) e a nossa
/// é **exactamente zero por construção**, porque o passo é montado no plano
/// tangente do próprio vértice. ⇒ o gate `G-5` da espec é **vácuo** sobre esta
/// lei, e o que ele protege — *o pente não mexe na forma* — mede-se assim.
#[test]
fn o_passo_nao_sai_do_plano_tangente() {
    let normal = crate::unitario([0.3, -0.5, 0.81]).expect("normal viva");
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.2], [-2.0, 0.3, -0.4]];
    let antes = pos[0];
    let movidos = pentear(&mut pos, &[0], &[normal], &[1.0], &[0, 2], &[1, 2], TRACO);
    assert_eq!(movidos, 1, "o arranjo deixou de conter o fenomeno");
    let passo = crate::subtrair(pos[0], antes);
    let fora = crate::produto_escalar(passo, normal).abs();
    assert!(
        fora < 1e-6 && crate::norma(passo) > 1e-3,
        "o passo mede {:.3e} fora do plano sobre um andar de {:.3e}",
        fora,
        crate::norma(passo)
    );
}

/// ⛔ **A degenerescência é INERTE, e são DUAS** — a espec §4.3 mede que o alvo
/// não inventa direcção nenhuma.
///
/// ⚠️⚠️ **Este é o ÚNICO sítio onde a direcção do traço ainda decide alguma
/// coisa no deslocamento**, e ela decide *SE*, nunca *PARA ONDE* (ver
/// [`o_deslocamento_nao_le_a_direccao_do_traco`]). Sem este gate, alguém que
/// lesse a lei nova concluiria — com razão aparente — que a direcção deixou de
/// ser lida e apagaria a guarda, e com ela a inércia do primeiro carimbo.
#[test]
fn sem_direccao_no_plano_o_vertice_fica() {
    let arranjo = |direccao: V3| {
        let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-2.0, 0.0, 0.0]];
        let movidos = pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], direccao);
        (movidos, pos)
    };
    // (1) — o controlo POSITIVO: com direcção viva, este arranjo move-se.
    assert_eq!(arranjo(TRACO).0, 1, "o arranjo nao contem o fenomeno");
    // (2) — sem carimbo anterior não há direcção nenhuma.
    assert_eq!(arranjo([0.0, 0.0, 0.0]).0, 0);
    // (3) — e uma direcção PARALELA à normal não tem projecção no plano: ela
    //       não é «para a frente» nenhuma, e inventar uma seria escolher por ele.
    assert_eq!(arranjo(CIMA).0, 0);
}

/// ⚠️ **O CARRIL morde, e é LOCAL.**
///
/// Uma vizinhança degenerada — duas arestas quase colineares e muito compridas —
/// pede um passo enorme; o tecto prende-o em `0,34` da aresta média **daquele**
/// vértice.
#[test]
fn o_carril_prende_o_passo_em_unidades_das_arestas_locais() {
    let mut pos: Vec<V3> = vec![[0.0, 0.0, 0.0], [3.0, 3.0, 0.0], [3.1, 2.9, 0.0]];
    let antes = pos[0];
    assert_eq!(
        pentear(&mut pos, &[0], &[CIMA], &[1.0], &[0, 2], &[1, 2], TRACO,),
        1
    );
    let andado = crate::norma(crate::subtrair(pos[0], antes));
    let aresta_local = (crate::norma([3.0, 3.0, 0.0]) + crate::norma([3.1, 2.9, 0.0])) / 2.0;
    let tecto = crate::TECTO_DA_VIAGEM * aresta_local;
    assert!(
        (andado - tecto).abs() < 1e-5,
        "o passo andou {andado:.5} contra o tecto local de {tecto:.5} — se ele \
         ficou ABAIXO, o arranjo deixou de pedir um salto e o carril nao esta' \
         a ser medido"
    );
}

/// ⛔⛔ **O LIMITE DECLARADO: um anel SIMÉTRICO é ponto fixo, e a `45°` isso
/// lê-se como a lei a falhar.**
///
/// Com quatro vizinhos em losango exacto o centróide **é** o vértice, logo o
/// passo é zero — e o anel fica a `45°` do traço para sempre.
///
/// ⛔⛔⛔ **A RAZÃO deste gate mudou em 18/09, e a mudança é o achado da wave.**
/// Ele dizia *«`45°` é um equilíbrio INSTÁVEL de qualquer energia de
/// alinhamento»*, que era verdade sobre a lei antiga. Sobre a de hoje a frase é
/// mais forte e mais simples: **o deslocamento nunca alinha coisa nenhuma**, aos
/// `45°` como a qualquer outro ângulo. Quem endireita é o campo de tamanho do
/// refino ([`crate::campo_do_pente`]), que sobre esta diagonal pede um alvo
/// `1 − k` e a manda partir.
///
/// ⚠️ **Isto está aqui para ninguém escrever o gate óbvio** — *«um losango a 45°
/// endireita-se»* — e concluir que a lei está partida quando ela lê `0`.
#[test]
fn um_losango_exacto_a_quarenta_e_cinco_graus_e_ponto_fixo_declarado() {
    let mut pos: Vec<V3> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [1.0, -1.0, 0.0],
    ];
    let antes = pos.clone();
    assert_eq!(
        pentear(
            &mut pos,
            &[0],
            &[CIMA],
            &[1.0],
            &[0, 4],
            &[1, 2, 3, 4],
            TRACO,
        ),
        0,
        "o losango exacto mexeu-se — se isto passou a acontecer, a lei mudou e \
         a declaracao deste gate tem de ser re-medida, nao apagada"
    );
    assert_eq!(pos, antes);
}

/// ⭐⭐⭐ **O CAMPO DE TAMANHO É PAR NA ARESTA** — o contrato que o
/// [`ph2d_mesh::Sizing`] exige, e sem ele o passe de topologia fica
/// NÃO-DETERMINÍSTICO.
///
/// A mesma aresta é proposta pelas DUAS faces que a dividem, cada uma no sentido
/// oposto. A porta da malha orienta-a pelo índice, o que já bastaria; esta
/// metade afirma que a LEI também não distingue, para que a cura do lado de lá
/// possa mudar sem abrir um defeito silencioso aqui.
#[test]
fn o_campo_de_tamanho_e_par_na_aresta() {
    let campo = crate::campo_do_pente(0.05, [0.7, -0.3, 0.1], 1.0, Porta::Refino);
    let mut viu_diferenca = false;
    for aresta in [
        [1.0, 0.0, 0.0],
        [0.6, 0.8, 0.0],
        [
            0.0,
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
        ],
        [-0.4, 0.2, 0.894_4],
    ] {
        let oposta = [-aresta[0], -aresta[1], -aresta[2]];
        assert_eq!(
            campo([0.0; 3], aresta),
            campo([0.0; 3], oposta),
            "o campo distingue {aresta:?} de {oposta:?}"
        );
        // O controlo: se o campo devolvesse sempre o mesmo número, a igualdade
        // acima seria trivial e este gate afirmaria o vazio.
        if (campo([0.0; 3], aresta) - 0.05).abs() > 1e-6 {
            viu_diferenca = true;
        }
    }
    assert!(
        viu_diferenca,
        "o campo devolveu o alvo nu em TODAS as arestas — a paridade acima e' vacuo"
    );
}

/// ⭐⭐⭐ **O CAMPO NUNCA PEDE MAIS FINO QUE O SLIDER, e o mínimo dele é `base`
/// EXACTAMENTE.**
///
/// É esta propriedade que protege o orçamento de triângulos que o `Detail`
/// promete (ordem do dono, 14/09: o alvo é ancorado numa CONTAGEM). ⚠️ Ela não é
/// um número escolhido: sai da normalização por `(1 − k)`, que põe o mínimo da
/// forma exactamente em `base` — *um tecto derivado da própria forma não precisa
/// de ser calibrado numa fixtura*.
///
/// ⛔⛔ **A forma CRUA foi construída e medida:** sem a normalização o campo pede
/// `base·(1−k)` sobre os eixos e adensa a malha em **`+75 %`**
/// (`6 759 → 11 824` vértices na mesma chapa e no mesmo traço), com a `Q` a
/// ganhar `0,0702` contra `0,0665`. *`3 %` de `Q` não paga `75 %` de malha.*
#[test]
fn o_campo_nunca_pede_mais_fino_que_o_slider() {
    let base = 0.05f32;
    let campo = crate::campo_do_pente(base, TRACO, 1.0, Porta::Refino);
    // Varre a esfera de direcções: nenhuma pode pedir abaixo de `base`.
    let mut menor = f32::INFINITY;
    for i in 0..64 {
        let a = std::f32::consts::TAU * i as f32 / 64.0;
        for (x, y, z) in [(a.cos(), a.sin(), 0.0), (a.cos(), 0.0, a.sin())] {
            menor = menor.min(campo([0.0; 3], [x, y, z]));
        }
    }
    assert!(
        menor >= base * 0.999_9,
        "o campo pediu {menor:.6} contra o alvo de {base:.6} — a normalizacao \
         deixou de proteger o orcamento de triangulos"
    );
    // E o mínimo é ATINGIDO sobre os eixos da grade — senão o campo seria só
    // «mais grosso em todo o lado», que é outra lei (e não alinha nada).
    for eixo in [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]] {
        assert!(
            (campo([0.0; 3], eixo) - base).abs() < 1e-6,
            "sobre o eixo {eixo:?} o campo le' {} e nao {base}",
            campo([0.0; 3], eixo)
        );
    }
}

/// ⭐⭐⭐ **O CAMPO DO COLAPSO NUNCA FUNDE MAIS DO QUE O PASSE NU, e o máximo
/// dele é `base` EXACTAMENTE.**
///
/// O gémeo do [`o_campo_nunca_pede_mais_fino_que_o_slider`], do outro lado da
/// comparação: o colapso funde o que é mais CURTO que o alvo, logo é o **máximo**
/// do campo que tem de ser pregado — e com ele pregado o viés só pode fazer o
/// passe fundir MENOS.
///
/// ⛔⛔⛔ **Este gate nasceu de um VERMELHO, e o vermelho foi de outro gate.**
/// Com as duas portas normalizadas pelo mínimo, o colapso passou a fundir
/// arestas até `(1+k)/(1−k)` vezes mais longas — e o
/// `o_pente_nao_compra_alinhamento_com_lascas` leu o pior triângulo da faixa a
/// **`1,96°`** contra o chão de `2°`. ⚠️ *O `Q` subia enquanto isso acontecia*:
/// sem aquela segunda coluna esta wave teria shipado uma malha em lascas.
#[test]
fn o_campo_do_colapso_nunca_funde_mais_do_que_o_passe_nu() {
    let base = 0.05f32;
    let campo = crate::campo_do_pente(base, TRACO, 1.0, Porta::Colapso);
    let mut maior = 0.0f32;
    for i in 0..64 {
        let a = std::f32::consts::TAU * i as f32 / 64.0;
        for (x, y, z) in [(a.cos(), a.sin(), 0.0), (a.cos(), 0.0, a.sin())] {
            maior = maior.max(campo([0.0; 3], [x, y, z]));
        }
    }
    assert!(
        maior <= base * 1.000_1,
        "o colapso pediu {maior:.6} contra o alvo de {base:.6} — ele voltou a \
         fundir arestas que o passe nu nao fundiria, e e' dai que vem a LASCA"
    );
    // E o máximo é ATINGIDO na diagonal — senão o campo seria só «mais fino em
    // todo o lado», que fundiria de menos por igual e não alinharia nada.
    assert!(
        (campo(
            [0.0; 3],
            [
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        ) - base)
            .abs()
            < 1e-6,
        "na diagonal o colapso le' {} e nao {base}",
        campo(
            [0.0; 3],
            [
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        )
    );
    // ⛔ **E as DUAS portas discordam**, senão a distinção seria decoração: o
    // refino sobre a diagonal pede muito mais grosso do que o colapso ali.
    let refino = crate::campo_do_pente(base, TRACO, 1.0, Porta::Refino);
    assert!(
        refino(
            [0.0; 3],
            [
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        ) > campo(
            [0.0; 3],
            [
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        ) * 2.0,
        "as duas portas devolveram o mesmo campo — a normalizacao por porta \
         deixou de ter efeito"
    );
}

/// ⭐⭐⭐ **A DIAGONAL RECEBE UM ALVO MAIS GROSSO, e a razão é `(1+k)/(1−k)`.**
///
/// ⛔⛔⛔ **A PREMISSA DESTE GATE MORREU E RENASCEU no mesmo dia.** A 1.ª redacção
/// chamava-se *«as duas metades empurram contra a diagonal»* e afirmava que o
/// refino e o colapso precisavam de **sinais OPOSTOS** — o refino a partir a
/// diagonal, o colapso a fundi-la. **Medido, isso está errado nas duas pontas:**
/// partir uma diagonal devolve **duas diagonais** (`Q −0,0599` com a malha a
/// dobrar), e a varredura das quatro combinações põe os dois campos a concordar
/// no mesmo sinal. ⇒ há **uma** função, e o que muda entre as portas é o sentido
/// da comparação, que é da PORTA e não da lei.
///
/// O mecanismo que fica: *um corte preserva a direcção da aresta que ele parte, e
/// faz DUAS dela* — logo a grade cresce partindo sobre os eixos, não contra eles.
#[test]
fn a_diagonal_recebe_um_alvo_mais_grosso() {
    let base = 0.05f32;
    let campo = crate::campo_do_pente(base, TRACO, 1.0, Porta::Refino);
    let k = crate::k_do_pente(1.0);
    let diagonal = campo(
        [0.0; 3],
        [
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ],
    );
    let eixo = campo([0.0; 3], [1.0, 0.0, 0.0]);
    let esperado = base * (1.0 + k) / (1.0 - k);
    assert!(
        (diagonal - esperado).abs() < 1e-6,
        "a diagonal le' {diagonal:.6} e a conta a' mao da' {esperado:.6}"
    );
    assert!(
        diagonal > eixo * 2.0,
        "a diagonal ({diagonal:.6}) nao e' notavelmente mais grossa que o eixo \
         ({eixo:.6}) — sem separacao o passe nao tem por onde escolher"
    );
}

/// ⭐⭐⭐ **COM O PENTE A ZERO O CAMPO É O ALVO, AO BIT** — o controlo sem o qual
/// nada do que está acima afirma que o caminho de omissão ficou intacto.
///
/// ⚠️ Não é «aproximadamente»: o termo inteiro é multiplicado por `k` e a
/// normalização vale `1,0` em `k = 0`, logo o passe de topologia com o pente
/// desligado recebe **exactamente** o número que recebia antes de este campo
/// existir.
#[test]
fn com_o_pente_a_zero_o_campo_e_o_alvo_ao_bit() {
    let base = 0.037_512_1;
    let campo = crate::campo_do_pente(base, [0.7, -0.3, 0.1], 0.0, Porta::Refino);
    for aresta in [
        [1.0, 0.0, 0.0],
        [
            std::f32::consts::FRAC_1_SQRT_2,
            std::f32::consts::FRAC_1_SQRT_2,
            0.0,
        ],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
    ] {
        assert_eq!(campo([1.0, 2.0, 3.0], aresta), base);
    }
}

/// ⛔ **Uma aresta sem direcção cai no valor ISOTRÓPICO do campo** — a
/// degenerescência que o [`ph2d_mesh::Sizing`] declara.
///
/// ⚠️ **E sem TRAÇO o campo é o alvo NU em toda aresta**, que é o que faz o
/// primeiro carimbo não enviesar nada — a mesma inércia que o deslocamento tem, e
/// pela mesma porta ([`crate::SculptStroke::direccao_do_traco`] do lado de lá).
#[test]
fn sem_direccao_o_campo_nao_enviesa() {
    let base = 0.05;
    let sem_traco = crate::campo_do_pente(base, [0.0; 3], 1.0, Porta::Refino);
    assert_eq!(
        sem_traco(
            [0.0; 3],
            [
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
                0.0
            ]
        ),
        base
    );
    assert_eq!(sem_traco([0.0; 3], [1.0, 0.0, 0.0]), base);
    // ⚠️⚠️ **Uma aresta degenerada COM traço é outra coisa**, e a distinção é o
    // achado: ali a grade EXISTE e esta aresta é que não tem direcção, logo ela
    // recebe o valor isotrópico do campo (`base/(1−k)`) e não o alvo nu.
    // *Confundir os dois leria a normalização como um defeito — e foi ao separar
    // os dois casos que o defeito a sério apareceu (ver `campo_do_pente`).*
    let campo = crate::campo_do_pente(base, TRACO, 1.0, Porta::Refino);
    let k = crate::k_do_pente(1.0);
    assert!((campo([0.0; 3], [0.0; 3]) - base / (1.0 - k)).abs() < 1e-6);
}
