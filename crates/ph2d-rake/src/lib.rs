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

/// Pentear uma vizinhança: **uma** passagem de relaxação.
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
        // O QUADRO no plano tangente deste vértice: `ao_longo` é a direcção do
        // traço projectada, `atraves` fecha a base. ⛔ Se a projecção morre, o
        // vértice fica — ver a degenerescência, acima.
        let Some(ao_longo) = unitario(sem_componente(direccao, normal)) else {
            continue;
        };
        let atraves = produto_vectorial(normal, ao_longo);

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
            // ⭐⭐ **O eixo da grade mais perto desta aresta.** É aqui que a lei
            // é uma GRADE e não uma direcção: a aresta é puxada para o eixo
            // `ao_longo` **ou** para o `atraves`, o que estiver mais perto — e é
            // por isso que as duas famílias crescem à custa das diagonais
            // (espec §3.3), em vez de tudo colapsar numa direcção só.
            let c = produto_escalar(plana, ao_longo);
            let s = produto_escalar(plana, atraves);
            let eixo = if c.abs() >= s.abs() {
                escalar(ao_longo, c.signum())
            } else {
                escalar(atraves, s.signum())
            };
            // Onde este vizinho quer que `v` esteja: a aresta mantém o
            // comprimento e roda para o eixo. ⇒ `v` anda por `plana − L·eixo`.
            soma = somar(soma, subtrair(plana, escalar(eixo, comprimento)));
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

fn produto_vectorial(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
