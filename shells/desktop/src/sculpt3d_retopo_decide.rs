//! ⭐⭐⭐ **A ESCOLHA ENTRE DUAS CANDIDATAS** — irmão de [`super::rulers`] por
//! RESPONSABILIDADE, e o corte é o que o doc daquele módulo já dizia.
//!
//! ⚠️ **Lá MEDE-SE, aqui DECIDE-SE.** As réguas (`open_edges`, `components`, `bowties`,
//! `tip_ratio`, `reach`) respondem *«quanto?»*; esta função responde *«qual?»*, e é a única
//! do caminho que ordena defeitos uns contra os outros. *Uma ordem de preferências é uma
//! decisão de produto, e ela merece um ficheiro em que se lê sem passar por aritmética.*
//!
//! ⛔ Nasceu do tecto de LOC do shell (HR-18, 600 — o irmão chegou a `614` ao ganhar a chave
//! da densidade da ponta em 2026-09-01).

use ph2d_mesh::Mesh;

use super::rulers::{components, inside_out, open_edges, tip_key_on, tip_ratio};

/// ⭐⭐⭐ **A FOLGA da chave das faces do avesso** — ver o uso em [`worse`], com a tabela dos dois
/// lados que o dono julgou (`125` reprovadas · `6`–`8` aprovadas).
const INSIDE_OUT_SLACK: usize = 20;

/// ⭐⭐ **O QUE UMA TENTATIVA DO BOTÃO DEVOLVE** — a malha, o relatório da extracção, o resíduo
/// de costura, a forma, e as **duas** réguas da ponta.
///
/// ⚠️ **Um alias e não um `struct`, de propósito:** a `attempt` continua a devolver um tuplo,
/// e um tipo com nome aqui só existe para que [`melhor`] o possa receber sem repetir seis
/// linhas de assinatura em cada sítio.
pub(super) type Candidata = (
    Mesh,
    ph2d_quadextract::ExtractReport,
    f32,
    ph2d_quadfill::QuadShape,
    ph2d_quadfill::TipDeviation,
    ph2d_quadfill::TipDensity,
);

/// ⭐⭐⭐ **ENTRE A ACTUAL E UMA ALTERNATIVA, FICA A MELHOR** — a porta única da cascata.
///
/// ⛔⛔ **Ela existe porque a mesma comparação estava escrita CINCO vezes**, com dez argumentos
/// cada, metade deles índices de tuplo (`a.3.skew_over_60`, `a.4`, `a.5`). ⚠️ *Uma chamada de
/// dez argumentos posicionais em que os cinco primeiros descrevem uma candidata e os cinco
/// seguintes a outra é um sítio onde trocar dois deles compila, passa a suíte, e inverte a
/// escolha* — e conferir isso foi trabalho manual numa auditoria de 2026-09-01. Com esta porta
/// há **um** sítio a montar os argumentos, e o gate irmão conta-o.
///
/// `won` diz se a candidata actual veio do campo alinhado; a alternativa traz o seu.
pub(super) fn melhor(
    won: bool,
    atual: Candidata,
    alt: Option<Candidata>,
    alt_won: bool,
) -> (bool, Candidata) {
    let Some(alt) = alt else {
        return (won, atual);
    };
    if worse(
        &atual.0,
        atual.3.skew_over_60,
        atual.3.skew_p50,
        atual.4,
        atual.5,
        &alt.0,
        alt.3.skew_over_60,
        alt.3.skew_p50,
        alt.4,
        alt.5,
    ) {
        (alt_won, alt)
    } else {
        (won, atual)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn worse(
    a_mesh: &Mesh,
    a_over60: usize,
    a_skew: f32,
    a_dev: ph2d_quadfill::TipDeviation,
    a_den: ph2d_quadfill::TipDensity,
    b_mesh: &Mesh,
    b_over60: usize,
    b_skew: f32,
    b_dev: ph2d_quadfill::TipDeviation,
    b_den: ph2d_quadfill::TipDensity,
) -> bool {
    let (a_holes, b_holes) = (open_edges(a_mesh), open_edges(b_mesh));
    if a_holes != b_holes {
        return a_holes > b_holes;
    }
    let (a_parts, b_parts) = (components(a_mesh), components(b_mesh));
    if a_parts != b_parts {
        return a_parts > b_parts;
    }
    // ⭐⭐⭐ **AS FACES QUE SE AUTO-INTERSECTAM — a 3.ª chave, e o report de 30/08 é a razão.**
    //
    // ⛔⛔⛔ **A régua que via a destruição JÁ EXISTIA e o produto não a consultava.**
    // [`ph2d_quadfill::local_shape`] vive numa crate do produto desde 30/08 e o seu único
    // leitor era a **sonda** da foto. Medido no A/B daquele dia: o caminho de omissão dá
    // **`0`** gravatas e o caminho novo dá **`125`** — e o dono descreveu a saída como
    // *«destruiu completamente a malha»*, enquanto as colunas que esta função lia diziam
    // apenas *pior* (`χ` `1 → 0`, bordo `4 → 12`, que se leem como brandos).
    //
    // ⚠️ **É a família do §5.0 do `CLAUDE.md`:** *nenhum instrumento do repo pergunta se o
    // valor chega a um consumidor.* Uma régua na prateleira não protege ninguém.
    //
    // ⚠️ **Aqui ela é ORDINAL, não veto**, e de propósito: uma tentativa com gravatas perde
    // sempre para uma sem, mas se **todas** as tentativas as tiverem ainda se escolhe a menos
    // má. *Um veto absoluto pede prova de corpus que esta linha ainda não tem* — e inventar
    // um limiar sem medir é o que o §0.0 proíbe.
    // ⭐⭐⭐ **AS DUAS ESPÉCIES DE FACE DO AVESSO SÃO UMA SÓ CHAVE, e ela tem FOLGA (2026-09-03).**
    //
    // ⛔⛔ **A gravata e a DOBRA são a mesma coisa vista de dois lados** — uma face que se cruza
    // a si própria e uma face que aponta contra a vizinhança — e as duas lêem-se na foto do dono
    // como uma **fenda escura**. Contar só as gravatas deixava a outra metade invisível: a saída
    // que ele fotografou em 03/09 tinha `0` gravatas e **cinco dobras no mesmo ponto**.
    //
    // ⛔⛔⛔ **E a IGUALDADE ESTRITA era o defeito.** Medido nas nove candidatas da peça dele:
    // *todas* as candidatas com as pontas boas têm `6`–`8` faces do avesso, e as que têm `0`
    // amputam `2`–`3` pontas. Com a chave a decidir por qualquer diferença, meia dúzia de faces
    // ganhava sempre a três pontas cortadas — e as pontas são o que ele fotografou **duas
    // vezes**.
    //
    // ⭐ **A folga vive no VAZIO MEDIDO entre os dois lados que o dono julgou:**
    //
    // | saída | faces do avesso | o veredito DELE |
    // |---|---|---|
    // | 30/08, a que motivou esta chave | **`125`** | ⛔ *«destruiu completamente a malha»* |
    // | 03/09, com as pontas curadas | **`6`–`8`** | ⭐ *«melhor resultado até agora»* |
    //
    // ⇒ `20` fica no meio, e ela continua **ordinal** acima disso: uma malha destruída perde
    // sempre. ⚠️ *Uma barra colada a um dos lados muda de veredito com a peça seguinte.*
    let (a_bow, b_bow) = (inside_out(a_mesh), inside_out(b_mesh));
    if a_bow.abs_diff(b_bow) > INSIDE_OUT_SLACK {
        return a_bow > b_bow;
    }
    // ⭐⭐⭐ **A AMPUTAÇÃO — e ela vem ANTES da forma, porque é o que o dono fotografou.**
    //
    // ⛔⛔⛔ **Medida em 2026-08-31, e é a razão de esta chave existir:** numa varredura do teto
    // de graduação da fase zero, a célula `ADAPT_RATIO = 8` entregou uma fase zero **perfeita**
    // (`0` de `4` pontas cortadas, pior `−0,5 %`) e a **saída** cortou a ponta mais longa em
    // ⛔ **`−43 %`**. As duas candidatas estavam limpas na topologia, e o `worse` escolheu a que
    // comia o espinho — *porque nada aqui olhava para o alcance.*
    //
    // ⛔⛔⛔ **E ATÉ 2026-08-31 ELA MEDIA O ALCANCE, QUE É UM EXTREMO GLOBAL — e sujo.**
    // Duas coisas mudaram no mesmo dia, as duas medidas:
    //
    // 1. **A régua estava contaminada.** O alcance tirava o centroide da média dos
    //    **vértices**, que é uma propriedade da amostragem: na escultura do dono o centroide
    //    derivava `0,2129` entre entrada e saída e a régua lia `−6,5 %` onde a verdade era
    //    `−0,1 %`; duas densidades da mesma peça diferiam `1,06 %` contra uma banda de `2 %`.
    //    ⚠️ E o sinal era o pior: quem **corta** a ponta perde vértices longe do corpo, o
    //    centroide afasta-se e o alcance medido **sobe**. Curado em
    //    [`ph2d_quadfill::reach`] (centroide de **área**), que o [`log_candidate`] imprime.
    // 2. **Um extremo global não conta QUANTAS pontas morreram** — é a limitação que esta
    //    linha nomeou três vezes. A régua por ponta existe agora
    //    ([`ph2d_quadfill::tip_deviation`]) e mede a distância da **escultura** à superfície
    //    de cada candidata junto de cada ápice, em unidades do quad pedido.
    //
    // ⭐⭐⭐ **E a troca MUDA uma escolha medida.** `_base_sculpt.obj` a `Detail 0,40`, onde as
    // duas primeiras candidatas **empatam** em bordo (`4`) e a chave decide:
    //
    // | candidata | alcance | **pontas acima da barra** |
    // |---|---|---|
    // | ⛔ `w = 0,000` (a que o alcance escolhia) | `2,8644` | **`2` de `4`** |
    // | ⭐ `w = 0,030` | `2,7869` | **`1` de `4`** |
    //
    // *A régua velha preferia a candidata com mais pontas partidas, porque a ponta que ela
    // media era a que sobrevivia nas duas.*
    //
    // ⚠️ **A barra é o chão da discretização** ([`ph2d_quadfill::TIP_DEVIATION_MAX`] = `1`
    // quad), não um número escolhido: medido, as pontas sãs ficam em `máximo 0,45` e a
    // partida em `p50 1,15`.
    //
    // ⛔ **A amostra vazia NÃO decide** — `tips = 0` é *«não medido»*, e lê-se igual a
    // *«perfeito»* em toda régua que devolva só a média.
    //
    // ⛔ **Depois dos FUROS e antes da forma:** um espinho cortado ao meio é mais visível que
    // uma face com canto pior que `60°`, e menos que um buraco — *que foi a queixa mais antiga
    // dele.*
    // ⛔⛔⛔ **E A CONTAGEM SOZINHA DEITA FORA A GRAVIDADE — corrigido em 2026-09-01.**
    //
    // A `over` conta **quantas** pontas passaram da barra e não diz **quão** longe. Uma
    // candidata que come uma ponta *por inteiro* (`p90 = 3,0`, que é o piso do «mais longe do
    // que eu olhei» de [`ph2d_quadfill::tip_deviation`]) e uma que a arranha (`p90 = 1,02`)
    // contam **`1` as duas**: empatam aqui, e a escolha cai para as chaves da beleza — faces
    // `>60°` e enviesamento —, que é decidir uma amputação por quão quadrados ficaram os quads.
    //
    // ⚠️ **A barra da `over` é a MEDIANA da ponta** (`TIP_DEVIATION_MAX`), logo meia ponta
    // comida não a arma sequer; a gravidade é a única coluna que a vê. *Os três números já
    // eram calculados e impressos no log — nada aqui os lia.*
    //
    // ⚠️ **`p90` e não `max`**: o `max` é o vértice mais afastado de uma amostra, e um único
    // ponto da escultura que caia numa fenda entre dois quads move-o sem que nada esteja
    // amputado. O `p90` é o mesmo extremo com a cauda de amostragem aparada, e continua a
    // separar `3,0` de `1,02` por larga margem.
    //
    // ⛔ **Depois da contagem, nunca à frente:** duas pontas partidas de raspão são um defeito
    // pior que uma partida a fundo — foi por «amputa **uma** ponta» / «amputou **2**» que o
    // dono nomeou os dois reports, nessa ordem.
    if a_dev.tips > 0 && b_dev.tips > 0 {
        // ⭐⭐⭐ **O ÁPICE PRIMEIRO — a amputação medida no próprio bico** (2026-09-02,
        // [`ph2d_quadfill::TIP_GAP_MAX`]). ⛔ A `over` abaixo conta pontas cuja MEDIANA de
        // vizinhança passou de `1,0`, e a agulha da foto que ele reprovou lê `p50 0,84` com
        // o bico a `1,11` da superfície: *a mediana afoga o ponto que define a ponta.* A
        // contagem de bicos a mais de meia célula da saída decide antes dela.
        if a_dev.cut != b_dev.cut {
            return a_dev.cut > b_dev.cut;
        }
        if a_dev.over != b_dev.over {
            return a_dev.over > b_dev.over;
        }
        if (a_dev.p90 - b_dev.p90).abs() > 1.0e-3 {
            return a_dev.p90.total_cmp(&b_dev.p90) == std::cmp::Ordering::Greater;
        }
    }
    // ⭐⭐⭐ **A GRADE QUE TERMINA ANTES DO BICO — a 5.ª chave, e o report de 2026-09-01 é a
    // razão** (foto com seta): *«essa área deveria ser levada à ponta, mas veja que ela fica a
    // meio caminho e a ponta fica cada vez menos densa em polígonos»*.
    //
    // ⛔⛔⛔ **Ele tinha razão por um factor de `3,85×`, e TODAS as réguas desta função diziam o
    // contrário.** A saída que ele exportou nesse dia é topologicamente impecável, sem ponta
    // amputada, com `0,23 %` de irregulares — classe do oráculo — e mesmo assim a ponta da foto
    // recebe quads **quase quatro vezes maiores** que a mediana da peça, a engrossar em direcção
    // ao bico. ⚠️ A régua que devia vê-lo — a `ENTREGA` ([`tip_ratio`]) — mede **cinco coroas
    // radiais à volta do centroide e faz média de todas as pontas**: cinco pontas certas afogam
    // a que colapsou, e ela imprimiu `0,553` (*«afina na ponta»*) sobre aquela peça.
    //
    // ⭐⭐ **O mecanismo, medido:** nas pontas boas há `183`–`246` vértices a menos de `6` quads
    // do bico e `1,2`–`1,6 %` deles são irregulares; nas duas más há **`8` e `26`**, com
    // **`37,5 %`** e `11,5 %` de irregulares e o ápice de valência `3`. *As linhas da grade não
    // convergem para o bico — terminam todas de uma vez, a meio do espinho.*
    //
    // ⚠️ **É a CONTAGEM e não o gradiente**, como na amputação: a `ENTREGA` continua a ser a
    // chave contínua, mais abaixo. E o lugar é o mesmo pelo mesmo motivo — uma ponta que perdeu
    // a grade é um defeito que o dono FOTOGRAFA, e uma face com canto pior que `60°` não.
    //
    // ⛔ **A amostra vazia NÃO decide** (`tips == 0` é *«não medido»*).
    if a_den.tips > 0 && b_den.tips > 0 && a_den.over != b_den.over {
        return a_den.over > b_den.over;
    }
    if a_over60 != b_over60 {
        return a_over60 > b_over60;
    }
    // ⭐⭐⭐ **A DENSIDADE DA PONTA — a chave que faltava, e a medição que a exige.**
    //
    // ⛔⛔⛔ **A cura do report de 28-29/08 já estava a ser produzida e era DEITADA FORA
    // aqui.** Medido em 2026-08-30 (`sculpt_antes.obj`, `Detail 0,85`), as candidatas do
    // caminho de omissão, **sem knob nenhum**:
    //
    // | candidata | quads | bordo | `>60°` | **`ENTREGA`** |
    // |---|---|---|---|---|
    // | campo liso | `9 484` | `28` | `2` | `1,585` |
    // | ⛔ campo alinhado (**a escolhida**) | `9 414` | `4` | `2` | `1,502` |
    // | ⭐ **campo com linhas de feição** | `9 121` | `4` | `2` | ⭐ **`0,851`** |
    //
    // ⭐⭐⭐ **A terceira EMPATA em furos, peças, gravatas e faces `>60°`** — ela perdia
    // **só** no enviesamento mediano, que era a última chave. *O eixo de que o dono se
    // queixou três vezes não estava na função que escolhe*, e o desempate era feito por uma
    // grandeza que ele não vê.
    //
    // ⚠️ **O lugar é DEPOIS de `>60°` e ANTES do enviesamento**, e isso é uma decisão: uma
    // face com canto pior que `60°` é um defeito local visível, uma ponta grosseira é um
    // defeito de **cobertura** (o dono fotografou-a), e a mediana do enviesamento é a única
    // das três que ele nunca nomeou.
    //
    // ⛔ **Nunca à frente dos FUROS.** Com `Follow Curvature` ligado, a candidata de feições
    // chega a `0,543` — o alvo é `0,59` — mas traz `6` arestas de bordo contra `4`. *Buracos
    // foram a queixa dele três vezes; esta chave não os compra.*
    //
    // ⚠️ **Menor é melhor, e sem banda** — pela mesma razão que a chave seguinte (o
    // enviesamento) não tem: inventar um limiar aqui seria escolher um número sem o medir.
    // ⛔ **A amostra vazia NÃO decide** (`0,0` de «não medido» lê-se como o melhor resultado
    // possível — é a armadilha que o doc do [`ph2d_quadfill::tip_body_ratio`] nomeia).
    //
    // ⚠️ `PH2D_RETOPO_TIPKEY=0` desliga a chave, para bissectar.
    if tip_key_on() {
        let ((a_tip, a_n), (b_tip, b_n)) = (tip_ratio(a_mesh), tip_ratio(b_mesh));
        if a_n > 0 && b_n > 0 && a_tip.total_cmp(&b_tip) != core::cmp::Ordering::Equal {
            return a_tip > b_tip;
        }
    }
    a_skew.total_cmp(&b_skew) == core::cmp::Ordering::Greater
}
