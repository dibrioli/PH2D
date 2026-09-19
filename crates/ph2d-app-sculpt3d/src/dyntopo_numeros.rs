//! ⭐⭐⭐⭐ **OS TRÊS NÚMEROS MEDIDOS DO PENTE** — as varreduras da retícula, as
//! alternâncias com a troca de ligação, e o lado da célula.
//!
//! ⚠️ Eles saíram do [`super::dyntopo`] por **TECTO DE LOC** (`722` contra
//! `700`), e o corte é por responsabilidade: ali fica o passe, aqui ficam as
//! **tabelas que escolheram cada número** — e nenhuma delas é uma constante que
//! alguém escreveu, todas trazem a varredura ao lado.

/// Quantas passagens cada um dos dois campos da retícula leva por carimbo.
///
/// ⛔⛔⛔ **A PREMISSA ANTERIOR DESTA CONSTANTE MORREU, e o diff mostra-a a
/// morrer.** Ela dizia, com escada ao lado, *«o patamar é em `2`, e o que sobra
/// acima dele é relógio»* — e era **verdade sobre a `grade`**, que foi a régua
/// com que a escada foi lida. A régua da FILEIRA
/// ([`ph2d_sculpt3d::medida_da_fileira`], escrita em 20/09 como passo zero da
/// ordem seguinte do dono) mede a grandeza que ele de facto vê — *quantas
/// arestas seguidas continuam a MESMA linha* — e ela **não assenta em `2`**:
///
/// | rondas | grade | vinco p50 | vinco p90 | **fil p90** |
/// |---|---|---|---|---|
/// | `2` | `64,03 %` | `0,973°` | `2,724°` | `47,0` |
/// | **`4`** (shipa) | **`64,52 %`** | **`0,966°`** | **`2,727°`** | **`56,0`** |
/// | `8` | `64,92 %` | `0,964°` | `2,703°` | `64,0` |
/// | `16` | `65,17 %` | `0,943°` | `2,681°` | `68,5` |
/// | `32` | `64,93 %` | `0,963°` | `2,704°` | `68,5` |
///
/// ⚠️⚠️ **Cada linha é a MÉDIA dos quatro rumos, e isso não é decoração:** num
/// rumo só a coluna da fileira salta entre degraus adjacentes (`fil90` leu `63`
/// na ronda `3` e `34` na `4`), porque o percurso dela é guloso e uma aresta a
/// entrar ou a sair do balde alinhado funde ou parte duas cadeias longas de uma
/// vez. *Uma coluna de alta variância lida numa amostra só fabrica uma
/// tendência* — e eu quase escrevi uma.
///
/// ⇒ a fileira **sobe monotonamente e satura em `16`**; quem escolhe o degrau é
/// o **RELÓGIO**. Medido em `--release`, o mínimo de cinco
/// (`diag_o_relogio_da_reticula`), contra o orçamento de `8 ms` do carimbo:
///
/// | vértices | pegada | `2` | **`4`** | `8` | `16` |
/// |---|---|---|---|---|---|
/// | `5 276` | `21` | **`0,102 ms`** | `0,162 ms` | `0,298 ms` | `0,559 ms` |
/// | `21 098` | `115` | **`0,518 ms`** | `1,140 ms` | `1,634 ms` | `3,237 ms` |
/// | `84 386` | `539` | `2,402` (`30 %`) | **`4,180` (`52 %`)** | `7,543` (**`94 %`**) | `14,013` (**`175 %`**) |
///
/// ⭐⭐⭐⭐ **E A RECUSA DO DEGRAU `4` FOI LEVANTADA — mas NÃO pela razão que a
/// dívida de 20/09 previa.** Ela dizia *«a cura é uma cerca que veja PARES,
/// nunca um número maior»*, e apontava para o sítio certo **pela razão errada**:
/// a cerca por-vértice não era fraca, era **ERRADA**. Ela veta um subconjunto
/// de um passo Jacobi cujos alvos são todos nós da MESMA grade, e deixa a malha
/// **meio-movida** — *uma configuração que nem a entrada nem o alvo têm*.
///
/// **Medido pela porta do produto** (quatro rumos, `4` rondas, knob no topo):
///
/// | cerca | lascas | portão da cena |
/// |---|---|---|
/// | **por-vértice** (a de ontem) | **`4`** | ⛔ **VERMELHO** |
/// | nenhuma | `0` | ✅ |
/// | **combinada** (a de hoje) | `0` | ✅ |
///
/// ⇒ *o que bloqueava o degrau não era o número: era a cerca a fabricar aquilo
/// que ela existia para impedir.* Mecanismo e as duas metades da lei:
/// [`ph2d_quadflow::regiao::veta_combinado`].
///
/// ⇒ o chão da forma deixou de ser o que aperta, e quem aperta volta a ser o
/// **RELÓGIO**: `4` fica em **`52 %`** do orçamento do carimbo e `8` em `94 %`,
/// que não deixa nada para o colapso, o refino e o `dab` do mesmo quadro.
///
/// ⚠️ **O recurso é o orçamento do carimbo (`8 ms`)** e o custo é **linear na
/// PEGADA** (nunca na peça) — é isso que a [`ph2d_quadflow::regiao`] existe
/// para garantir.
///
/// ⚠️⚠️ **A primeira corrida da escada original leu «sem tendência» e estava a
/// medir outro programa:** ela varreu as rondas com o [`LADO_DA_CELULA`] em
/// `1,0`, onde TODAS as leituras são más — *uma escada corrida no regime errado
/// responde sobre um produto que não existe*. Sonda: `diag_a_escada_das_rondas`.
///
/// # ⭐⭐⭐⭐ E EM 21/09 ELA DESCEU A `2`, PORQUE A MEMÓRIA MUDOU A ESCADA
///
/// Ordem do dono, depois de eu lhe ter posto na mesa a troca de `−26 %` de
/// relógio por `0,7` pontos de regularidade: ***«quero o melhor possível, o
/// padrão ouro»***. A obra que ela destravou é a que estava nomeada aqui em
/// baixo — **carregar o campo de posição ENTRE dabs**
/// ([`ph2d_quadflow::regiao::CampoDoTraco`]) —, e ela não é uma optimização: é
/// a **fase** que o §86 nomeou como o que um pincel precisa e uma hierarquia não
/// dá.
///
/// ⛔⛔ **E ela MATA a premissa desta constante.** A tabela acima diz *«a fileira
/// sobe monotonamente e satura em `16`; quem escolhe o degrau é o RELÓGIO»* —
/// **verdade sobre um campo que recomeça do zero a cada carimbo, e FALSA sobre
/// um que se lembra**. Medido pela porta do produto, oito rumos, com a memória
/// ligada:
///
/// | rondas | alt | memória | grade % | fil p50 | fil p90 | 4 braços % | ms/traço |
/// |---|---|---|---|---|---|---|---|
/// | `4` | `2` | **não** (o que shipava) | `65,36` | `35,1` | `70,9` | `92,74` | `207,8` |
/// | **`2`** | **`2`** | **sim** (shipa) | **`65,91`** | **`50,4`** | **`71,9`** | **`95,38`** | **`161,4`** |
/// | `4` | `2` | sim | `65,63` | `42,1` | `71,6` | `95,28` | `238,7` |
/// | `2` | `1` | sim | `65,35` | `43,4` | `71,0` | `93,94` | `124,8` |
/// | `2` | `2` | não | `64,64` | `16,5` | `57,2` | `88,46` | `184,9` |
///
/// ⭐⭐⭐ **Duas leituras, e a segunda é a que muda o desenho:**
///
/// 1. `2`/`2` com memória **ganha ao `4`/`2` de ontem em TODAS as colunas** —
///    grade, as duas da fileira e os quatro braços — e custa **`−22 %`**. *Não é
///    a troca que o dono recusou; é o contrário dela.*
/// 2. ⛔ **`4` COM memória é PIOR que `2` com memória**, e mais caro: com uma
///    semente já coerente, varrer mais é **sobre-relaxar** — o campo afasta-se
///    da retícula que a malha de facto tem para servir uma média mais larga.
///
/// ⭐⭐ **E a memória baixa a DISPERSÃO entre rumos**, que é uma coluna de
/// qualidade por si: `grade ±0,17` contra `±0,31`, `4 braços ±0,72` contra
/// `±1,98`. *O resultado deixa de depender tanto da direcção em que o artista
/// risca.*
///
/// ⚠️ **O que a alternância `2` continua a comprar** está na linha `2`/`1`: ela
/// é `23 %` mais barata e paga `1,4` pontos de quatro braços — a troca de
/// ligação continua a ser metade do trabalho ([`ALTERNANCIAS`]).
pub(crate) const RONDAS_DA_GRELHA: usize = 2;

/// ⭐⭐⭐ **Quantas vezes a retícula e a troca de ligação se ALTERNAM.**
///
/// As duas são complementares — uma move vértices e **nunca re-liga**, a outra
/// re-liga e **nunca move** —, e alternar dá à segunda passagem da retícula um
/// grafo melhor do que o que a primeira encontrou. **Medido** (fracção de
/// cruzamentos com os quatro braços na vista da grade, quatro rumos, com as
/// rondas totais da retícula constantes):
///
/// | alternâncias | 4 braços |
/// |---|---|
/// | `1` | `92,32 %` |
/// | **`2`** | **`93,01 %`** |
/// | `4` | `91,73 %` |
///
/// ⛔ **E pôr a troca no FIM do passe — a ordem clássica da remalhagem
/// isotrópica (*partir → fundir → trocar → alisar*) — foi CONSTRUÍDO, MEDIDO e
/// REVERTIDO:** `4 braços 93,01 → 92,38`, `fil50 38,8 → 27,5`, grade
/// `65,44 → 65,25`. *No fim não a segue mais nada; dentro da alternância a
/// retícula ainda trabalha sobre o grafo que a troca acabou de arrumar.*
///
/// # ⛔⛔⛔ A CAÇA AO RELÓGIO: cinco tentativas, UMA sobrevivente
///
/// Report do dono (21/09): *«algoritmo mais lento que o modo padrão. tem que
/// otimizar»* — e ele tem razão: medido pela porta do produto na peça grande, o
/// passe de topologia custa **`0,104 ms` sem pente e `7,692` com**.
///
/// **O perfil**, por alternância (pegada `626`): `mancha 0,330` · `orientação
/// 0,553` · **`posição 2,797`** · `relax 1,442`.
///
/// | tentativa | resultado |
/// |---|---|
/// | **pentear a cada `N` dabs** | ⛔ `fil50` `38,8 → 18,8` só a saltar UM — *a qualidade vem da ACUMULAÇÃO, e saltar dabs é fazer menos trabalho, não o mesmo trabalho mais barato* |
/// | **içar as 16 quinas** do miolo do campo (16 avaliações onde bastam 4+4) | ⛔ **`1 %`** — o compilador já o fazia |
/// | **conjunto activo** nas varreduras (saltar quem não mudou) | ⛔ **`0 %`** — a semente do campo é a POSIÇÃO do vértice, logo a 1.ª varredura move toda a gente |
/// | **encolher a pegada** do pente (`0,9` · `0,8` · `0,7` do raio) | ⛔ a qualidade cai **proporcionalmente** (`−29 %` de relógio por `−4,9` pontos) |
/// | **semear o relax nos MOVIDOS** | ⛔ `−5 %` a `−11 %` por `1,2` pontos — a retícula move quase toda a pegada |
/// | ⭐ **`ALTERNANCIAS` `2 → 1`** | **`−26 %`** (`212 → 158 ms` no traço), por `0,7` pontos de regularidade e `fil90 70,8 → 68,5` |
///
/// # ⭐⭐⭐⭐ E A SOBREVIVENTE NÃO FOI TOMADA — o dono escolheu a outra saída
///
/// A troca do `2 → 1` foi-lhe posta com o preço ao lado (ela desfaz parte do que
/// ele aprovara), e a resposta foi ***«quero o melhor possível, o padrão
/// ouro»***. ⇒ **ela fica RECUSADA**, e o que shipou foi a última linha desta
/// tabela: **carregar o campo de posição ENTRE dabs**.
///
/// ⭐ A nota dizia que a memória valeria *«`~1` varredura em vez de `4`»* e a
/// medição deu-lhe razão pela metade que importa: o produto corre hoje **`2`**
/// varreduras ([`RONDAS_DA_GRELHA`]) e entrega **mais** qualidade em todas as
/// colunas por **`−22 %`** de relógio. ⛔ *A sobrevivente teria custado
/// qualidade para comprar `26 %`; a memória compra `22 %` e ainda paga
/// qualidade de volta.*
///
/// ⚠️ **E a 3.ª recusa desta tabela — o «conjunto activo» a `0 %`** — era o
/// mesmo defeito visto do lado errado: com a semente a ser a posição do vértice
/// a 1.ª varredura move toda a gente, logo não há quem saltar. *A cura não era
/// saltar trabalho: era não deitar fora a resposta do carimbo anterior.*
pub(crate) const ALTERNANCIAS: usize = 2;

/// O lado da célula, em aresta média da pegada.
///
/// ⭐⭐⭐ **É ESTA a alavanca, e a escada é brutal** (peça da cena `=49`, rumo
/// `x`, com o vinco a ser o que a LUZ mostra):
///
/// | `k` | grade | vinco p50 | vinco p90 |
/// |---|---|---|---|
/// | `0,80` | **`64,9 %`** | **`0,96°`** | **`2,70°`** |
/// | `0,90` | `64,9 %` | `1,13°` | `3,27°` |
/// | `0,931` | `64,1 %` | `1,20°` | `3,49°` |
/// | `1,00` | `57,3 %` | `2,13°` | `20,64°` |
/// | `1,10` | `45,4 %` | `5,90°` | `98,49°` |
/// | `1,25` | `39,8 %` | `15,99°` | `126,42°` |
///
/// ⚠️ **O recurso é a CAPACIDADE da célula:** um quadrado de lado `e` cobre `e²`
/// por vértice e um triângulo equilátero de aresta `e` cobre `0,866 e²` ⇒ pedir
/// a uma malha de triângulos que pouse numa grelha quadrada do **mesmo** lado
/// empilha vértices, e um empilhamento é uma dobra. O ponto de empate teórico é
/// `√0,866 = 0,931`, e a medição põe o joelho **abaixo** dele — entre `0,931` e
/// `1,00` o vinco p90 salta `5,9×`.
///
/// ⛔ **Abaixo de `0,80` não se mediu ganho:** `0,80` e `0,90` leem a mesma
/// grade (`64,9 %`), logo o que `0,80` compra é só o vinco, e ele já está ao
/// nível da malha **por pentear**.
pub(crate) const LADO_DA_CELULA: f32 = 0.80;
