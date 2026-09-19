//! **O PENTE DE TOPOLOGIA** — a lei, e só ela.
//!
//! Clean-room sob [`docs/3D/cleanroom/SPEC_pente_de_topologia.md`], atestada à
//! 3.ª passagem do R-pré em 2026-09-17.
//!
//! # O que ele é
//!
//! Enquanto o traço corre com a topologia dinâmica armada, a malha debaixo dele
//! reorganiza-se de modo que as arestas deixam de apontar em todas as direcções
//! e passam a formar uma **GRADE**: uma família ao longo do traço, outra através
//! dele.
//!
//! ⭐⭐⭐ **E o nome engana — o alvo NÃO troca uma única diagonal** (espec §3.1,
//! medido: com o operador de topologia parado o conjunto de arestas é idêntico
//! entre o controlo a `0` e a `1` em `1`, `2`, `4`, `8` e `16` passagens). Ele
//! **move vértices**, tangencialmente, e o que o artista lê como topologia é o
//! refino a decidir sobre um terreno já relaxado.
//!
//! # ⭐⭐⭐⭐ A LEI SÃO DUAS METADES, e nenhuma delas alinha sozinha
//!
//! 1. **O DESLOCAMENTO** ([`pentear`]) é uma relaxação **ISOTRÓPICA**: cada
//!    vértice anda para o centróide do anel dele, no plano tangente. Ela
//!    reproduz o campo do alvo (`cos 0,971`) e **não lê a direcção do traço**.
//! 2. **O ALINHAMENTO** ([`campo_do_pente`]) mora no **alvo de aresta que o
//!    passe de topologia recebe**: mais fino sobre os eixos da grade, mais
//!    grosso na diagonal. *Um corte preserva a direcção da aresta que ele parte,
//!    e faz DUAS dela* — é assim que a grade cresce.
//!
//! ⛔⛔⛔ **E elas ANDAM JUNTAS, com número:** a relaxação isotrópica sozinha lê
//! `Q = +0,0000` contra a barra de `+0,0465` — *o campo de deslocamento do alvo
//! reproduzido ao cosseno `0,97`, e grade nenhuma*. Com as duas, `Q = +0,0657`.
//! **O alinhamento não mora no deslocamento.**
//!
//! # ⛔⛔ Onde ela TEM de correr, e isto está medido
//!
//! **Dentro do laço por-carimbo**, sobre a malha que o refino acabou de
//! produzir. ⛔ **Nunca como um passe separado antes ou depois:** compor as duas
//! metades em série **falha** a barra em todas as granularidades que o
//! instrumento do oráculo alcança (o melhor caso lê `Q = +0,0028` contra
//! `+0,0465`), e grosseira sai com o **sinal trocado** (`−0,2190`) — *relaxar
//! uma malha grosseira e só depois subdividir destrói o alinhamento, porque os
//! vértices que o passe insere não sabem nada do traço* (espec §14).
//!
//! # A régua pela qual esta lei é julgada
//!
//! `Q = média(cos 4α)` sobre as arestas da pegada, com `α` o ângulo entre a
//! aresta e a direcção do traço. ⛔ **A régua de DUAS dobras é cega por
//! construção** — uma grade tem as duas famílias, logo ela lê `≈ 0` — e foi a
//! primeira que se escreveu (espec §3.3 e o gate `G-13`).

/// Um ponto ou um vector.
pub type V3 = [f32; 3];

/// ⚠️ **O TECTO DA VIAGEM DE UM VÉRTICE NUM CARIMBO**, em múltiplos do
/// comprimento médio das arestas **locais** dele.
///
/// O número vem da espec §3.2: a maior norma de deslocamento que o oráculo
/// produz num traço inteiro é `0,34` do comprimento médio de aresta da malha de
/// ENTRADA — e a espec nomeia a população de propósito, porque sobre a saída o
/// mesmo deslocamento lê `0,34` e restrito à pegada lê `0,32`.
///
/// ⚠️ **Ele é um CARRIL, não a magnitude.** Quem decide quanto um vértice anda
/// é a relaxação a convergir; isto só impede um passo isolado de saltar mais do
/// que o oráculo alguma vez saltou num traço inteiro. *Sem ele, uma vizinhança
/// degenerada (duas arestas quase colineares) manda o vértice para longe num
/// carimbo só.*
///
/// ⭐ **E ele é LOCAL de propósito** — média das arestas do próprio vértice, não
/// da malha: um tecto global faria o pente andar mais numa zona fina do que numa
/// grossa, que é o defeito que a transição da pose e o alvo da densidade desta
/// casa já pagaram, cada um à sua maneira.
pub const TECTO_DA_VIAGEM: f32 = 0.34;

/// Pentear uma vizinhança: **uma** passagem de relaxação ISOTRÓPICA.
///
/// O destino de cada vértice é o **centróide do anel dele**, projectado no plano
/// tangente e pesado pela queda do pincel. ⛔ **A direcção do traço não entra
/// nesta conta** — ela decide só *SE* o vértice obedece (ver a degenerescência,
/// abaixo), e quem alinha é o [`campo_do_pente`], do outro lado.
///
/// # ⭐⭐⭐ O que esta lei substituiu, e com que número
///
/// Até 2026-09-18 ela encaixava cada aresta no eixo da grade mais próximo e
/// puxava o vértice para uma cruz regular. Medido contra o campo do alvo, célula
/// a célula, com o cosseno ponderado pela norma:
///
/// | | `x_man_x01` | `x_man_x04` |
/// |---|---|---|
/// | encaixe duro em 4 eixos | `0,583` | `0,557` |
/// | **o centróide do anel** | **`0,971`** | **`0,974`** |
///
/// ⭐⭐ E as **26** células `p100` do placar do oráculo melhoraram, nenhuma
/// regrediu: `x_man_x08` de `5,374e-2` para `2,595e-2`, a banda de `6,2e-2` para
/// `4,58e-2`, a rotação a `67,5°` de `6,146e-2` para `3,647e-2`.
///
/// # ⛔⛔⛔ AS RECUSAS MEDIDAS — não reconstrua nenhuma destas
///
/// Todas foram construídas e medidas contra o campo do alvo, e todas **piores**
/// que o centróide:
///
/// - **mais varreduras** por carimbo (`1 · 2 · 3 · 5 · 8`): a amplitude sobe
///   `0,654 → 1,116` e o cosseno **DESCE** `0,583 → 0,535` — *não era uma
///   relaxação parada a meio, era um óptimo diferente*;
/// - **encaixe duro** (`0,583`) · **rodar mantendo o comprimento** (`0,154`) ·
///   **quatro dobras suaves** (`0,750`) · **mistura contínua** (monótona a
///   descer de `0,886`) · **anisotrópica** (`0,887`, indistinguível do
///   isotrópico puro em `k = 0`).
///
/// ⭐⭐⭐ **E a leitura das seis, junta, era a resposta e ninguém a leu assim:**
/// *o que reproduz o campo é REGULARIZAR, e a direcção do traço não entra em
/// forma nenhuma do deslocamento.* Ela não entra porque **não é ali que ela
/// mora** — e a prova é que a lei isotrópica, com o cosseno a `0,971`, lê
/// `Q = +0,0000` contra a barra de `+0,0465`. *O alinhamento não mora no
/// deslocamento.*
///
/// ⏳ **O que fica por explicar:** movemos `0,74` do que ele move
/// (`|nós|/|ele|`), e esse excesso de `~26 %` aponta em direcções que uma
/// relaxação isotrópica não prevê. ⚠️ Ele **não é alinhamento** — o `ΔQ` dele é
/// zero —, e a pista é que o erro ainda **dobra** fora do eixo do traço
/// (`1,41e-2` a `0°` contra `4,06e-2` a `45°`), com o controlo ao lado: com o
/// pente desligado as quatro rotações leem `1,6e-4`.
///
/// - `posicoes` — lidas **e** escritas.
/// - `normais` — uma por vértice, já normalizadas; elas definem o plano tangente
///   em que o quadro do traço é escrito.
/// - `inicio`/`vizinhos` — a adjacência em CSR: os vizinhos de `v` são
///   `vizinhos[inicio[v]..inicio[v+1]]`.
/// - `direccao` — a direcção do traço em MUNDO, entre os dois últimos carimbos.
///   ⚠️ **Não precisa de estar normalizada** e não precisa de ser tangente: ela é
///   projectada no plano de cada vértice.
/// - `pesos` — quanto cada vértice obedece, em `[0, 1]`. Zero deixa-o intacto.
///
/// Devolve **quantos vértices se moveram**.
///
/// # ⭐ Jacobi limpo, e a razão é o DETERMINISMO
///
/// Todas as leituras saem de uma fotografia do início da passagem. Um
/// Gauss-Seidel faria metade dos vértices ler o valor novo e metade o velho, e a
/// saída passaria a depender da **ordem de varredura** da região — que é a mesma
/// cura que a suavização da pose e a do esfregão desta casa já pagaram.
///
/// ⚠️ **Isto NÃO contradiz a espec §3.4** (*«a ordem importa»*): lá a ordem é a
/// do CAMINHO percorrido pelo artista, que continua a decidir tudo, porque cada
/// carimbo relaxa sobre o que o anterior deixou. O que fica determinístico é a
/// ordem de varredura **dentro** de um carimbo, que nenhum artista escolhe.
///
/// # ⛔ A degenerescência é INERTE, nunca inventada
///
/// Se a direcção do traço for nula, ou se ela for paralela à normal de um
/// vértice (logo sem projecção no plano tangente dele), esse vértice **não se
/// move**. A espec §4.3 mede que o alvo faz o mesmo: com menos de dois carimbos
/// não há direcção, e ele *não faz nada* — não inventa uma nem usa a última.
pub fn pentear(
    posicoes: &mut [V3],
    alvos: &[u32],
    normais: &[V3],
    pesos: &[f32],
    inicio: &[u32],
    vizinhos: &[u32],
    direccao: V3,
) -> usize {
    let n = posicoes.len();
    let k = alvos.len();
    if n == 0 || k == 0 || normais.len() < k || pesos.len() < k || inicio.len() < k + 1 {
        return 0;
    }
    if norma(direccao) <= 0.0 {
        return 0;
    }
    // ⭐⭐ **JACOBI: nada é escrito enquanto alguma coisa é lida.** O laço só
    // LÊ `posicoes` e acumula os destinos; a escrita acontece toda depois.
    //
    // ⛔ A 1.ª redacção lia o vizinho VIVO e o próprio de uma fotografia, o que
    // é um Gauss-Seidel disfarçado: um vizinho que também está na pegada e vem
    // antes na lista já teria andado quando alguém o lesse, e a saída passaria a
    // depender da ORDEM da pegada. *Copiar só a pegada era barato e errado; ler
    // tudo e escrever no fim é barato e certo.*
    let mut destinos: Vec<(usize, V3)> = Vec::new();

    for i in 0..k {
        let v = alvos[i] as usize;
        if v >= n {
            continue;
        }
        let peso = pesos[i];
        if peso.partial_cmp(&0.0) != Some(std::cmp::Ordering::Greater) {
            continue;
        }
        let (a, b) = (inicio[i] as usize, inicio[i + 1] as usize);
        if b <= a || b > vizinhos.len() {
            continue;
        }
        let normal = normais[i];
        // ⛔⛔ **A DIRECÇÃO DO TRAÇO NÃO ENTRA NO DESLOCAMENTO — ela é só a
        // GUARDA.** Se a projecção dela no plano deste vértice morre, o vértice
        // fica (a degenerescência do cabeçalho, medida na espec §4.3). *Ela
        // decide SE o vértice obedece, nunca PARA ONDE ele anda* — para onde é o
        // centróide do anel, e o alinhamento mora no refino (ver o cabeçalho).
        if unitario(sem_componente(direccao, normal)).is_none() {
            continue;
        }

        let mut soma = [0.0f32; 3];
        let mut comprimentos = 0.0f32;
        let mut contados = 0u32;
        for &u in &vizinhos[a..b] {
            let u = u as usize;
            if u >= n {
                continue;
            }
            let aresta = subtrair(posicoes[u], posicoes[v]);
            // No plano tangente, que é onde a grade vive.
            let plana = sem_componente(aresta, normal);
            let comprimento = norma(plana);
            if comprimento <= 0.0 {
                continue;
            }
            comprimentos += norma(aresta);
            contados += 1;
            // ⭐⭐⭐ **O DESTINO É O CENTRÓIDE DO ANEL, e mais nada.** A soma das
            // arestas planas dividida pela contagem **é** `tangencial(centróide
            // − p)`, porque projectar é linear. Não há eixo, não há encaixe, não
            // há direcção: o deslocamento é uma relaxação ISOTRÓPICA.
            soma = somar(soma, plana);
        }
        if contados == 0 {
            continue;
        }
        let media = escalar(soma, 1.0 / contados as f32);
        // O carril, em unidades das arestas DESTE vértice.
        let tecto = TECTO_DA_VIAGEM * (comprimentos / contados as f32);
        let passo = escalar(media, peso);
        let andado = norma(passo);
        if andado <= 0.0 {
            continue;
        }
        let passo = if andado > tecto {
            escalar(passo, tecto / andado)
        } else {
            passo
        };
        // ⚠️ **O passo é tangencial por construção** — `plana` e o eixo vivem os
        // dois no plano de `normal` —, e é isso que faz o pente não poder
        // engordar nem comer a forma. Ver a divergência declarada no gate.
        destinos.push((v, somar(posicoes[v], passo)));
    }

    let movidos = destinos.len();
    for (v, destino) in destinos {
        posicoes[v] = destino;
    }
    movidos
}

/// ⭐⭐⭐ **O VIÉS DE QUATRO DOBRAS — a METADE DA LEI QUE MORA NO PASSE.**
///
/// ⭐⭐ **O número sai de uma varredura com DUAS colunas, e as duas são
/// obrigatórias** (`diag_a_varredura_do_vies`, em `ph2d-sculpt3d`): o `Q` da
/// grade **e** o pior triângulo da faixa. *Um `k` que compra `Q` em lascas não é
/// uma grade* — e o `Q` sozinho não distingue as duas coisas.
///
/// | `k` | `Q` (barra `+0,0465`) | pior triângulo (chão `2°`) | vértices (base `6 759`) |
/// |---|---|---|---|
/// | `0,45` | `0,0384` ⛔ | `25,63°` | `6 161` |
/// | `0,55` | `0,0433` ⛔ | `20,21°` | `6 065` |
/// | `0,65` | `0,0435` ⛔ | `19,82°` | `5 973` |
/// | **`0,75`** | **`0,0517`** | **`20,45°`** | **`5 835`** |
/// | `0,80` | `0,0452` ⛔ | `12,58°` | `5 731` |
///
/// ⇒ `0,75` é o **único** ponto medido que passa a barra do `Q` com o triângulo
/// intacto, e a `0,80` o ângulo começa a cair. ⚠️ A margem é de `11 %` e o `Q`
/// tem ruído de `±0,005` nesta fixtura — *isto é uma barra apertada, não
/// folgada*, e está escrito aqui para ninguém a ler como conforto.
///
/// # ⛔⛔⛔ E a TENSÃO é dura: alinhamento contra qualidade de triângulo
///
/// A `Q` cresce quando o campo pede arestas mais FINAS sobre os eixos, e isso
/// adensa a malha e afina os triângulos. Medido, com o colapso já conservador:
///
/// | normalização do refino | `Q` | vértices | pior triângulo |
/// |---|---|---|---|
/// | **crua** (sem normalizar) | `0,064`–`0,084` | `11 317`–`36 896` | **`0,19°`–`0,39°`** ⛔ |
/// | neutra em DENSIDADE (`(1−k²)^(−3/4)`) | `0,070`–`0,084` | `9 092`–`12 955` | **`0,32°`–`2,10°`** ⛔ |
/// | **mínimo em `base`** (a de hoje) | `0,038`–`0,052` | `5 835`–`6 161` | `19,8°`–`25,6°` ✓ |
///
/// ⇒ *toda configuração com `Q` alto entrega triângulos sem normal utilizável*,
/// e a normalização conservadora é o único ponto da curva em que as duas colunas
/// sobrevivem. **As duas primeiras linhas são RECUSAS MEDIDAS.**
///
/// ⏳ **E a saída de fundo está nomeada e não construída:** alinhar **trocando**
/// diagonais (um *flip*) muda a direcção de uma aresta **sem criar nenhuma**, ou
/// seja sem pagar densidade nem afinar triângulo. ⛔ Ela diverge do mecanismo que
/// a espec §3.1 mede no alvo (*«ele não troca uma única diagonal»*), logo é obra
/// com espec própria — não a construa por dentro desta.
///
/// ⛔ A cerca `< 1` é de representação: com `≥ 1` o alvo de uma família ficaria
/// **zero ou negativo**, e `h·h` devolveria um limiar que se lê como positivo —
/// *um alvo negativo não recusa em voz alta, ele comporta-se como o módulo
/// dele*.
pub const VIES_DA_GRADE: f32 = 0.75;

const _: () = assert!(
    VIES_DA_GRADE > 0.0 && VIES_DA_GRADE < 1.0,
    "o vies tem de deixar o alvo ESTRITAMENTE positivo nas duas famílias"
);

/// **`+1` sobre um eixo da grade, `−1` sobre a diagonal** — o parâmetro de ordem
/// de quatro dobras, para UMA aresta.
///
/// ⭐⭐ **É a MESMA forma que a régua `Q` usa** (`cos 4α` por Chebyshev a partir
/// do produto escalar 3D cru, `8c⁴ − 8c² + 1`), e isso não é elegância: a lei e
/// a régua que a julga têm de falar da mesma grandeza, senão o que se optimiza
/// não é o que se mede. ⭐ E a forma de Chebyshev é o que **dispensa a base
/// tangente** — não é preciso saber a normal da superfície aqui.
///
/// ⚠️ **Ela é PAR em `aresta` por construção** (só `c²` entra), que é o contrato
/// que o [`ph2d_mesh::Sizing`] exige: a mesma aresta é proposta pelas duas faces
/// que a dividem, cada uma no sentido oposto.
///
/// ⚠️ Um vector nulo — de qualquer dos lados — devolve `0`, que faz o campo cair
/// no valor **isotrópico**. *Uma aresta degenerada não tem direcção, e inventar
/// uma seria decidir por ruído.*
#[must_use]
pub fn quatro_dobras(direccao: V3, aresta: V3) -> f32 {
    let (ld, la) = (norma(direccao), norma(aresta));
    if ld <= 0.0 || la <= 0.0 {
        return 0.0;
    }
    let c = produto_escalar(direccao, aresta) / (ld * la);
    let c2 = (c * c).min(1.0);
    8.0 * c2 * c2 - 8.0 * c2 + 1.0
}

/// ⭐⭐⭐ **O CAMPO DE TAMANHO DO PENTE — a metade da lei que mora no PASSE DE
/// TOPOLOGIA, e é ela que alinha.**
///
/// `h(u) = base · (1 − k·cos 4α)`, com `k = pente · [`VIES_DA_GRADE`]`:
/// **arestas mais FINAS sobre os eixos da grade, mais grossas na diagonal.**
///
/// # ⭐⭐⭐ O mecanismo, e ele foi MEDIDO ao contrário primeiro
///
/// **Um corte preserva a direcção da aresta que ele parte, e faz DUAS dela.**
/// ⇒ baixar o alvo sobre os eixos faz o refino partir preferencialmente arestas
/// alinhadas, e cada corte devolve **duas** arestas alinhadas à população que a
/// régua `Q` conta. Do outro lado, subir o alvo na diagonal faz o colapso fundir
/// preferencialmente as diagonais.
///
/// ⛔⛔⛔ **A hipótese oposta — «partir a diagonal para a apagar» — foi
/// construída e REFUTADA por medição** (18/09): partir uma diagonal devolve
/// **duas diagonais**, e a saída lê `Q −0,0599` com a malha a DOBRAR
/// (`6 059 → 13 279` vértices). *Uma coluna de `Q` sozinha não distinguiria as
/// duas coisas — é por isso que a varredura imprime a contagem de vértices ao
/// lado* ([`diag_a_varredura_do_vies`] em `ph2d-sculpt3d`).
///
/// # ⭐⭐ Porque é UMA função e não uma por porta
///
/// O refino parte o que é mais LONGO que o alvo e o colapso funde o que é mais
/// CURTO — os dois sentidos opostos vêm da PORTA, não da lei —, e medido, é o
/// **mesmo** sinal que serve os dois: a varredura das quatro combinações põe
/// `(refino, colapso)` a concordar no melhor par. ⛔ *Duas funções com nomes
/// diferentes para a mesma fórmula são duas respostas à mesma pergunta, e a que
/// alguém esquecer de emendar é a que envelhece.*
///
/// # ⚠️ `pente = 0` devolve `base` AO BIT
///
/// Não «aproximadamente»: o termo inteiro é multiplicado por `k`, logo com o
/// pente desligado o passe de topologia recebe **exactamente** o número que
/// recebia antes de este campo existir. É o controlo, e há gate.
///
/// # ⭐⭐⭐ O campo NUNCA pede mais fino do que o slider — e isso é uma
/// PROPRIEDADE, não uma constante ajustada
///
/// A forma crua (`base·(1 − k·cos 4α)`) pede `base·(1−k)` sobre os eixos, e
/// medido isso **adensa a malha em `+75 %`** (`6 759 → 11 824` vértices na
/// mesma chapa e no mesmo traço). ⛔ O `Detail` é ancorado numa CONTAGEM de
/// triângulos por ordem do dono (14/09) — *um pente que multiplica o orçamento
/// por `1,75` faz o slider mentir*.
///
/// ⇒ o campo é dividido por `(1 − k)`, o que põe o mínimo dele **exactamente**
/// em `base`: o alvo é `base` sobre os eixos da grade e **mais grosso** em todo
/// o resto, logo a densidade só pode DESCER. ⭐ *Um tecto derivado da própria
/// forma não precisa de ser calibrado numa fixtura*, que é o que separa isto de
/// um factor ajustado.
///
/// ⭐⭐ **E a `Q` quase não paga**, porque ela é PLANA na escala: medido em
/// `k = 0,45`, a `Q` lê `0,0702 · 0,0697 · 0,0678 · 0,0683 · 0,0748` enquanto a
/// contagem de vértices vai de `11 824` a `5 227`. *O que compra a grade é a
/// anisotropia; a densidade é um parâmetro livre ao lado dela.*
///
/// ⛔⛔ **E há CONTROLO para essa frase:** refinando **isotropicamente** até à
/// mesma contagem, a `Q` fica em `−0,0019` (a `V = 13 004`), `+0,0263`
/// (`10 111`) e `−0,0163` (`21 319`) — *vagueia à volta de zero, que é o que
/// «sem alinhamento» parece*. Sem esta metade a leitura estaria confundida.
///
/// ⚠️ O `pente` é preso a `[0, 1]` porque um valor fora dele poria o alvo de uma
/// família **negativo**, e o consumidor eleva ao quadrado: *um alvo negativo não
/// recusa em voz alta, ele comporta-se como o módulo dele.*
/// # ⛔⛔⛔ SEM TRAÇO NÃO HÁ GRADE, e o campo tem de ser o alvo NU
///
/// Um `k` vivo com direcção nula deixa `cos 4α = 0` em toda aresta e a
/// normalização sozinha de pé ⇒ o campo devolvia `base/(1−k)`, ou seja
/// **`1,82×` mais grosso, no primeiro carimbo de todo traço**. *A inércia da
/// espec §4.3 não é «não enviesar»: é não fazer NADA* — e o gate
/// [`sem_direccao_o_campo_nao_enviesa`] apanhou-o antes de ele shipar.
pub fn campo_do_pente(
    base: f32,
    direccao: V3,
    pente: f32,
    porta: Porta,
) -> impl Fn(V3, V3) -> f32 + Sync {
    let k = if norma(direccao) > 0.0 {
        k_do_pente(pente)
    } else {
        0.0
    };
    let normaliza = normalizacao(k, porta);
    move |_posicao, aresta| base * (1.0 - k * quatro_dobras(direccao, aresta)) * normaliza
}

/// ⭐⭐⭐ **De que lado da comparação está quem pede o campo** — e é isto que
/// escolhe a normalização.
///
/// A forma é a MESMA nas duas (`1 − k·cos 4α`); o que muda é **qual extremo dela
/// é pregado no alvo isotrópico**, e a razão é que cada porta compara ao
/// contrário da outra.
///
/// ⛔⛔⛔ **A 1.ª redacção normalizou as duas pelo MÍNIMO, e o gate das lascas
/// apanhou-o:** o colapso passou a fundir arestas até `(1+k)/(1−k)` vezes mais
/// longas do que antes, e fundir uma aresta longa arrasta um vértice para longe
/// — o pior triângulo da faixa caiu de `4,56°` para **`1,96°`**. *O `Q` subia
/// enquanto isso acontecia*, que é exactamente porque aquele gate existe.
///
/// ⇒ a lei que fica é uma só e vale para as duas: **o viés só pode fazer o passe
/// trabalhar MENOS do que o passe isotrópico faria.** O refino nunca pede mais
/// fino que o slider; o colapso nunca funde mais do que fundia.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Porta {
    /// Parte o que é **mais longo** que o alvo ⇒ o mínimo do campo é pregado em
    /// `base`, e o passe **nunca pede mais fino do que o slider**.
    Refino,
    /// Funde o que é **mais curto** que o alvo ⇒ o **máximo** é pregado em
    /// `base`, e o passe **nunca funde uma aresta que não fundiria sem pente**.
    Colapso,
}

/// ⭐⭐⭐ **A PREFERÊNCIA DE DIRECÇÃO — a terceira metade da lei, e a única que
/// não paga nada.**
///
/// Devolve, para a direcção unitária de uma aresta, quanto ela é desejada
/// (maior é melhor). O consumidor é o [`ph2d_mesh::alinha_arestas`], que troca
/// uma diagonal por outra **a contagem constante** quando a nova é mais
/// desejada e o pior triângulo do par não piora.
///
/// # ⛔⛔⛔ Porque ela existe, com a medição que a obrigou
///
/// As outras duas metades alinham **criando** arestas, e isso tem preço: medido
/// na chapa, toda configuração do [`campo_do_pente`] com `Q` alto entrega
/// triângulos de `0,2°`–`2,1°` ou adensa a malha `5,5×` (ver a tabela em
/// [`VIES_DA_GRADE`]). E na BOLA da cena de smoke — *a peça que o artista vê* —
/// a resposta ao botão é **não-monótona**: ela tem pico a `pente ≈ 0,4` e
/// DEGRADA no topo, e a `30°` do traço nunca chega à barra com `k` nenhum.
///
/// ⇒ *o corte não é o mecanismo certo para a última fatia do alinhamento*, e a
/// troca é: ela muda a direcção de uma aresta **sem criar nenhuma**.
///
/// ⚠️ **É a MESMA função de quatro dobras** que o campo e a régua `Q` lêem — as
/// três metades não podem discordar sobre o que é uma diagonal.
pub fn preferencia_do_pente(direccao: V3, pente: f32) -> impl Fn(V3) -> f32 + Sync {
    let k = if norma(direccao) > 0.0 {
        k_do_pente(pente)
    } else {
        0.0
    };
    move |aresta| k * quatro_dobras(direccao, aresta)
}

/// O `k` da lei a partir do botão — ver [`campo_do_pente`].
///
/// ⚠️ **Público porque a SONDA da varredura precisa dele**, e a alternativa era
/// a sonda reescrever `pente · VIES_DA_GRADE`: *uma régua que recalcula a lei
/// que julga mede a cópia dela.*
#[must_use]
pub fn k_do_pente(pente: f32) -> f32 {
    pente.clamp(0.0, 1.0) * VIES_DA_GRADE
}

/// O factor que prega um extremo do campo exactamente em `base` — ver
/// [`Porta`]. Em `k = 0` ele é `1,0` **ao bit** nas duas.
#[must_use]
pub fn normalizacao(k: f32, porta: Porta) -> f32 {
    match porta {
        // O mínimo do campo está na DIRECÇÃO DO EIXO (`cos 4α = +1`).
        Porta::Refino => 1.0 / (1.0 - k),
        // E o máximo na DIAGONAL (`cos 4α = −1`).
        Porta::Colapso => 1.0 / (1.0 + k),
    }
}

fn subtrair(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn somar(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn escalar(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn produto_escalar(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norma(a: V3) -> f32 {
    produto_escalar(a, a).sqrt()
}

fn sem_componente(a: V3, normal: V3) -> V3 {
    let k = produto_escalar(a, normal);
    [
        a[0] - normal[0] * k,
        a[1] - normal[1] * k,
        a[2] - normal[2] * k,
    ]
}

fn unitario(a: V3) -> Option<V3> {
    let l = norma(a);
    (l > 0.0).then(|| escalar(a, 1.0 / l))
}

#[cfg(test)]
#[path = "leis_tests.rs"]
mod leis_tests;
