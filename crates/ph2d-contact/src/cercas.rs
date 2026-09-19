//! **AS DUAS CERCAS DO LAÇO DE SEPARAÇÃO** — irmã do [`super`] pelo tecto de LOC (HR-18) e por
//! ASSUNTO: ali mora a LEI, aqui moram os dois números que decidem **quando o laço muda de
//! comportamento**, cada um com a tabela de onde saiu.
//!
//! ⚠️ Os dois nasceram de reports do dono em 2026-09-18, e nenhum deles é «um valor razoável»: um é
//! o ponto onde o `fork/join` passa a pagar, o outro é o joelho onde a saída deixa de depender do
//! limiar. *Um limite sem a medição ao lado é um palpite à espera de um smoke* (§0.0).

/// ⭐⭐ **A partir de quantas peças uma varredura paga o fork/join** — MEDIDO na sonda
/// [`custo_probe::onde_o_paralelo_passa_a_pagar`], não herdado.
///
/// ⚠️ O [`ph2d_nodegraph::attr::PAR_THRESHOLD`] (`8192`) é o equilíbrio de um nó que corre UMA
/// passagem por quadro com um corpo por-elemento pequeno. Aqui o corpo é o vizinhado mais o SAT de
/// cada par, e a passagem repete-se `varreduras` vezes — *o mesmo `n` tem outro ponto de
/// equilíbrio*, e usar o de lá deixava `500` peças num núcleo com 31 parados.
pub const PECAS_PARA_PARALELIZAR: usize = 128;

/// ⭐⭐⭐ **O REPOUSO VISÍVEL** — abaixo desta fracção do ALCANCE de uma peça, mais uma varredura não
/// muda o que se vê, e o [`separate`] pára.
///
/// ⚠️ **O número é MEDIDO** ([`custo_probe::parar_quando_nada_mais_se_ve`]), e a régua não é o
/// resíduo de uma varredura — é o **DESVIO da saída** contra varrer o tecto inteiro, mais a
/// contagem de pares ainda sobrepostos, que é o que o artista vê:
///
/// | limiar | varreduras (campo de 500) | desvio | pares sobrepostos |
/// |---|---|---|---|
/// | `1e-3` | 50 | `2,1e-2` | **178** (era 177) ⇠ já muda a resposta |
/// | `1e-4` | 100 | `2,1e-3` | 177 |
/// | **`1e-5`** | **149** | **`2,4e-4`** | **177** (era 177) |
/// | `1e-6` | 207 | `2,7e-5` | 177 |
///
/// ⇒ `1e-5` é o joelho: a última coluna **não muda** e a conta cai `6,9×`. ⛔ A `1e-3` a resposta
/// já é outra — *o número não é «um epsilon razoável», é onde a saída deixa de depender dele*.
pub const REPOUSO_VISIVEL: f32 = 1e-5;

/// ⭐⭐ **Quantas peças leva uma TAREFA do caminho paralelo** — a partição explícita que substituiu a
/// árvore do `collect` do rayon.
///
/// ⚠️ **Ele é um compromisso entre dois custos medidos:** um grão pequeno faz tarefas de mais (cada
/// uma com o seu roubo de trabalho e a sua espera — foi isso que pôs `4`–`5` núcleos de 32 a
/// trabalhar com `5×` o CPU da série), e um grão grande deixa núcleos sem trabalho no fim. A
/// tabela de onde ele sai está na sonda [`crate::custo_probe::o_grao_da_tarefa`].
/// ⭐⭐⭐ **O PISO de uma tarefa do caminho paralelo** — abaixo disto o rayon não parte mais.
///
/// ⚠️⚠️ **Ele é um PISO e não uma partição, e a diferença foi medida NO APP DO DONO.** A 1.ª
/// redacção fixava o número de pedaços (`n / (núcleos × 4)`): numa máquina **ocupada** isso não se
/// nota, porque os trabalhadores já estão acordados, e numa máquina **PARADA** acordar `125` tarefas
/// de `2 µs` custa mais do que o trabalho — o relatório dele piorou de `18,6` para `30,4 ms`.
/// ⛔ *As minhas tabelas foram todas tiradas com a máquina entre `load 22` e `32`, e a conclusão não
/// transferia.*
///
/// ⇒ o rayon decide a partição (ele parte quando há um trabalhador livre) e isto só o impede de
/// descer a UM elemento. Medido a `load 4,5`, `1000` discos e `64` varreduras:
///
/// | piso | parede | núcleos de facto |
/// |---|---|---|
/// | **8** | **13,2 ms** | 9,1× |
/// | 16 | 11,8 ms | 11,9× |
/// | 64 | 21,7 ms | 5,1× |
/// | 256 | 41,4 ms | 1,9× |
///
/// ⚠️ E a `4000` discos o piso pequeno ganha por mais (`43,0` contra `51,8` a `64`): *um piso
/// grande tira ao rayon a liberdade de que ele precisa quando o trabalho por peça varia* — numa
/// pilha o número de vizinhos varia muito de peça para peça.
pub const PISO_DA_TAREFA: usize = 8;

/// ⭐⭐⭐ **QUÃO GRANDE TEM DE SER O GANHO PREVISTO para a grelha partir em DUAS CAMADAS.**
///
/// ⚠️⚠️ **Ela aplica-se à CONTAGEM REAL, e a história de porquê é o valor desta cerca.** A 1.ª
/// redacção aplicava-a ao número que o MODELO prevê (`9 · ρ · lado²`), e ele **sobrestima a coluna
/// de uma camada em `24 %`** — o bloco `3 × 3` está cortado nas bordas da nuvem. Isso obrigava a
/// margem a cobrir um erro que ninguém consegue medir na cena de OUTRA pessoa.
///
/// ⇒ hoje o modelo só **PROPÕE** qual corte tentar, e o plano **CONSTRÓI as duas hipóteses e
/// CONTA-AS** (`candidatos_previstos`, três leituras do CSR por peça, uma vez por passe — nunca por
/// varredura). *Uma decisão que se mede não precisa de acreditar num modelo numa cena que eu nunca
/// vi*, e o preço é um `constroi` a mais por passe contra as dezenas que ele evita.
///
/// ⚠️ A margem FICA porque o candidato não é o único custo: a malha fina tem `k²` vezes mais
/// CÉLULAS, e zerá-las é `O(células)` por varredura. Sem ela, uma dispersão de `1,25 ×` — que em
/// candidatos REAIS já perde (`0,91 ×`) — passaria por um empate técnico.
///
/// Medido em [`crate::custo_probe::contagens::onde_o_corte_dos_grandes_paga`] (`1000` discos, passo
/// `1,8 · R`, candidatos totais de uma varredura):
///
/// | dispersão | uma camada | duas camadas | ganho | com `MARGEM = 2` |
/// |---|---|---|---|---|
/// | `1,00 ×` | `12 132` | `12 132` | `1,00 ×` | não parte |
/// | `1,25 ×` | `18 212` | `20 076` | **`0,91 ×`** ⇠ piora | não parte |
/// | `1,50 ×` | `26 338` | `20 076` | `1,31 ×` | não parte (deixa `1,31 ×` na mesa) |
/// | `2,00 ×` | `44 590` | `20 076` | `2,22 ×` | **parte** |
/// | `4,00 ×` | `159 396` | `20 076` | **`7,94 ×`** | **parte** ⇠ *a cena do dono* |
/// | `8,00 ×` | `483 584` | `20 076` | `24,09 ×` | **parte** |
///
/// ⛔ **E a segunda metade da cerca não é uma constante, é o termo `g · m` do modelo:** promover uma
/// peça não é de graça — ela passa a ver a nuvem inteira. A coluna vira sozinha, e a tabela mostra
/// onde (`1000` discos, dispersão `4 ×`): `1` grande dá `11,3 ×`, `64` dão `1,18 ×`, **`128` dão
/// `0,64 ×`**. *O minimizador encontra esse joelho sem que ninguém escreva o número.*
pub const MARGEM_DO_CORTE: f32 = 2.0;

/// ⭐⭐⭐ **QUANTAS CÉLULAS VALEM UM CANDIDATO** — o segundo termo do custo de uma varredura, e o que
/// faltava à decisão do plano.
///
/// ⚠️⚠️ **Ela nasceu de um report do dono** (*«motor anterior mais rápido»*, 19/09): a decisão
/// contava CANDIDATOS e a malha fina paga também `O(células)` por varredura — zerar o `inicio` e
/// correr a soma acumulada. *Uma regra que só olha metade do custo escolhe pela metade errada
/// exactamente onde a outra metade explode.*
///
/// Medido varrendo o LADO da célula sobre a MESMA nuvem (`1000` discos, perfil `smoke`,
/// `load 4,17`): à medida que ele encolhe os candidatos caem e as células sobem.
///
/// | lado | células | candidatos | uma varredura |
/// |---|---|---|---|
/// | `4,00` | `70` | `159 396` | `1 594,4 µs` |
/// | `1,00` | `782` | `12 132` | `219,7 µs` |
/// | `0,25` | `11 438` | `1 000` | **`25,7 µs`** ⇠ o joelho |
/// | `0,125` | `45 315` | `1 000` | **`56,8 µs`** ⇠ *o relógio DOBRA com os candidatos parados* |
///
/// ⇒ as duas últimas linhas isolam a célula (`+33 877` células por `+31,1 µs` ⇒ **`0,92 ns`**), e o
/// par `1,00 → 0,25` isola o candidato (**`9,16 ns`**, já descontada a célula). A razão dá **`10,0`**,
/// e é esse o número — ⛔ não um peso escolhido.
pub const CELULAS_POR_CANDIDATO: usize = 10;
