# O material, a ruga e a memória — o report de 2026-09-09

> **Report do dono, com três fotos:**
> 1. *«se eu fizer mais de uma simulação mesmo com preserve volume no máximo o
>    objeto continua esticando … Mesmo na primeira simulação o pano cresce e
>    estica.»*
> 2. *«Nunca consigo uma configuração onde o pano passa a ter ondulação maiores
>    como se fosse um pano duro ou um couro. Sempre as ondulações são finas.»*
> 3. *«Plasticity parece ter efeito parecido com Damping, resistindo à
>    simulação.»*

Ele abriu com *«o resultado com Gravity se tornou muito decente, de boa
qualidade, mas ainda não perfeito»*. Os três reports são **três mecanismos
diferentes**, e um deles não tem cura de botão nenhum.

---

## §1 — «O pano continua esticando» — o repouso REBASELINAVA

Cada abertura do filtro lia a malha de AGORA como repouso, logo o segundo gesto
media o tecto de `1,10` sobre um comprimento que já era `1,10`. Medido em três
gestos de gravidade sobre uma esfera com três pontos mascarados:

| gesto | área/A₀ | altura | esticão máx (contra o original) |
|---:|---:|---:|---:|
| 1 | `1,133` | `2,65` | `2,04` |
| 2 | `1,229` | `3,08` | `2,22` |
| 3 | **`1,306`** | **`3,46`** | **`2,54`** |

⭐ **A cura é o MATERIAL atravessar os gestos** — e a maquinaria já existia: a
**base persistente** da lei ([`Verlet::base`]) entra em **quatro** leituras da
*construção*, entre elas o comprimento de repouso de cada restrição, e **não**
nos alvos nem nos pesos. *O comprimento do material é do material; onde ele está
é do gesto.*

| gesto | área/A₀ | altura | esticão máx |
|---:|---:|---:|---:|
| 1 | `1,133` | `2,65` | `2,04` |
| 2 | `1,147` | `2,68` | `1,47` |
| 3 | `1,147` | `2,69` | `1,52` |

⚠️ **E a outra metade é a INVALIDAÇÃO:** o material é re-semeado quando a
assinatura da malha não é a que o último gesto deixou — *«alguém esculpiu»*. Sem
ela a cura seria pior que o defeito: o pano lutaria contra a forma acabada de
esculpir. ⛔ Perguntar *«que ferramenta correu?»* seria a segunda resposta, e
envelhece com cada ferramenta nova; perguntar à MALHA não envelhece.

### 1.1 O que sobra no PRIMEIRO gesto, e por que não é afinação

Fica `+9 %` de área sem conservar volume e `+13 %` com ela. Duas metades:

- ⚠️ **com *Preserve Volume* a área TEM de crescer** — a peça alonga e, para
  manter o volume, a pele estica. Os dois pedidos puxam em sentidos opostos, e
  isso é física, não defeito;
- o resto é o limitador local a não convergir. ⭐ **A sobre-relaxação (SOR `1,9`)
  é de GRAÇA** e é a maior parte do que havia a ganhar — o relógio não se move:

| `ω` | esticão máx (`4 514` vért.) | esticão máx (`24 386`) | ms/passo |
|---:|---:|---:|---:|
| `1,0` | `1,563` | `6,717` | `19,14` |
| `1,5` | `1,441` | `5,043` | `19,10` |
| **`1,9`** | **`1,427`** | **`3,467`** | `19,69` |

⏳ **ABERTO e nomeado:** a `24 386` vértices o pior esticão continua em `3,47`
com o tecto em `1,10` — a convergência de um limitador **local** é `O(diâmetro da
malha em arestas)`, e quem segura o global é a âncora de longo alcance, que não
fala do vizinho.

---

## §2 — «Nunca consigo ondulações maiores» — ⛔⛔ e nenhum botão podia

Não havia **modelo de dobra nenhum**: a única coisa que resistia a uma prega era
a rede de restrições de distância, cujo alcance é **uma aresta**. Foi construída
a restrição de ângulo diedro da família PBD (§4.3), sobre o ângulo de repouso do
material — ⭐ e a matemática **já vivia na crate sem consumidor** (o
[`bending.rs`](../../../crates/ph2d-cloth/src/bending.rs), escrito para o caminho
VBD que a auditoria de 05/09 refutou): o ângulo com sinal por `atan2`, as quatro
derivadas cuja soma é zero por construção, e a diferença dobrada para `(−π, π]`.
*A lei mudou de família e a geometria não.*

Ela funciona — o desvio de dobra `p99` cai `73,0° → 47,7°` com a rigidez no
máximo. **Mas não é isso que ele pediu**, e a medição diz porquê:

| vértices por lado | aresta | onda (dobra `0`) | onda (dobra `1`) |
|---:|---:|---:|---:|
| `20` | `0,100` | `0,667` | `0,667` |
| `40` | `0,050` | `0,333` | `0,400` |
| `80` | `0,025` | `0,250` | `0,333` |

⭐⭐⭐ **O comprimento de onda de uma prega é `~7` a `10` ARESTAS, sempre.** Ele
acompanha a malha, não a lei; a rigidez move-o `+33 %` e nada mais. ⇒ *«nunca
consigo uma configuração»* estava certo por construção: **não existe
configuração**.

⚠️ **A alavanca de hoje é a densidade da malha** (o botão de retopologia), e a
cura publicada que decoupa as duas é um solver **hierárquico** (Müller,
*Hierarchical Position Based Dynamics*, 2008) — obra com nome, não afinação.

### 2.1 ⛔ Duas RÉGUAS minhas mentiram antes de o produto ser medido

- **contar travessias de sinal sem limiar de amplitude** lê `12 → 36` pregas com
  a rigidez a subir — o contrário do fenómeno: quanto mais rígido, mais plano, e
  um plano com ruído de `f32` cruza a média a cada vértice. *Uma régua de
  contagem sem amplitude conta o ruído quando a amplitude morre.*
- **uma cortina ESTICADA não tem prega para medir** — sem material a sobrar ela
  mede a lei de esticão. A fixtura é uma cortina **franzida**.

---

## §3 — «Plasticity parece Damping» — o nome dizia o CONTRÁRIO do efeito

Ele leu o controlo certo. Com o valor alto o vértice é puxado de volta à forma
inicial, logo ele **resiste** — e a espec do alvo diz exactamente isso
(*«plasticidade `1`: o vértice é sempre puxado à memória de forma inicial (o pano
recupera)»*).

⛔⛔ **Em materiais, *plasticidade* é deformação PERMANENTE — o oposto do retorno
elástico.** Aqui `1` = volta inteira à forma e `0` = a memória segue o vértice e
nada volta. ⇒ o rótulo do FILTRO passa a ***Shape Memory***.

⚠️ **O do PINCEL fica com o nome do alvo** (*Soft Body Plasticity*), e a
divergência é deliberada: ali cada rótulo tem uma fixture do oráculo com o mesmo
nome, e um artista que siga um tutorial do alvo tem de o encontrar. ⚠️ E a LEI
mantém o nome nos dois (`Solver::plasticidade`) — é por ele que as `103` fixtures
falam.

---

## §4 — ⛔ Recusas MEDIDAS

- **Um tecto GLOBAL sobre a ÁREA da peça**, derivado do tecto de esticão
  (`A ≤ A₀·t²`). Construído e medido: a `t²` ele **nunca morde**, e apertá-lo
  PIORA o local — `t¹` leva o esticão máximo de `2,04` a `2,30`, `t⁰` (área
  congelada) a **`4,79`**, enquanto a área mal desce (`1,13 → 1,11`). O
  mecanismo: uma restrição escalar tem **um** gradiente, logo encolhe onde o pano
  está FROUXO — que é onde não há esticão — e com a conservação de volume ligada
  as duas globais puxam em sentidos opostos.
- **Gauss-Seidel na lei de dobra.** Cada vértice pertence a ~`12` dobradiças, e
  uma projecção inteira por dobradiça soma doze correcções sobre o mesmo vértice:
  com rigidez `1,0` a área ia a **`4,04×`** e o volume caía a `0,27`. É a mesma
  divergência que o tecto de esticão já tinha pago, e a cura é a mesma — Jacobi
  com média.
- **Mais passagens de dobra.** De `1` para `16` a onda vai de `0,1055` a `0,1079`
  (`+2 %`) e o passo de `6,9` para `36,4 ms`. *Iterar mais não coarsa a prega;
  o que a fixa é a aresta.*

---

## §5 — ⏳ ABERTO

- ✅ **O `Expand` FECHOU em 2026-09-09** — o mecanismo estava certo (ele desloca o
  REPOUSO, e o tecto media-se contra um comprimento que a própria lei alarga) e
  ⚠️ **os números `155×`/`59×` estavam errados**: medido de fresco, era `7,743`.
  Cura, tabela e prova de mutação no [doc 10 §5](10_o_elastico_que_nao_para_e_o_volume.md).
- ⚠️ **A onda presa à malha — a nota do §2 está PARCIALMENTE REFUTADA (09/09).**
  Ela dizia `~7`–`10` arestas *«faça o artista o que fizer»*, e essa varredura
  variou a RIGIDEZ com as PASSAGENS presas em `1`: a rigidez diz com que força a
  restrição empurra **ali**, a passagem diz até onde a resistência **viaja**. Numa
  malha fina, `8`–`32` passagens levam a onda de `10` para `20`–`27` arestas com a
  amplitude a SUBIR ([`sonda_da_onda_da_prega`](../../../crates/ph2d-sculpt3d/tests/it/sonda_da_onda_da_prega.rs)).
  ⛔ Ainda **não** é um botão: na malha grossa as mesmas passagens achatam a peça
  (`0,329 → 0,0016` de amplitude) e a régua conta lobos, que é grosseiro. ⇒ o
  solver hierárquico continua a ser a cura publicada, mas como a via **barata** de
  ter muitas passagens, **não** como pré-requisito. Uma wave começa por uma régua
  melhor (FFT ou autocorrelação) e pelo preço das passagens.
- **A convergência do limitador local em malha densa** — §1.1.

---

## §5.1 — ⚠️ Um membro NOVO da família das flakes de carga

`alpha::scale::tests::the_recommendation_does_not_walk_the_whole_mesh`
([`ph2d-sculpt3d`](../../../crates/ph2d-sculpt3d/src/alpha_scale_tests.rs)) reprovou
no pico do fan-out com `8,27×` contra uma barra de `4×`, e é **`3` de `3` verde
sozinho** — com o `/proc/loadavg` a `27`–`42` ao lado, e **zero linhas de diff**
naquele ficheiro. Ele tem as três assinaturas do `CLAUDE.md` §5.0: mede uma
**razão de dois relógios**, o commit não lhe toca, e sozinho é verde.

⚠️ O vizinho `only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos
de áudio) reprovou na mesma corrida e **já está na lista** — é a mesma família.

---

## §6 — Onde está

- Lei: [`verlet_limites.rs`](../../../crates/ph2d-cloth/src/verlet_limites.rs)
  (`resistir_a_dobra`, `SOBRE_RELAXACAO`, `conhecer_as_caras`) ·
  [`verlet.rs`](../../../crates/ph2d-cloth/src/verlet.rs) (`Solver::dobra`).
- Produto: [`stroke.rs`](../../../crates/ph2d-sculpt3d/src/stroke.rs)
  (`cloth_material`, `cloth_left`) ·
  [`stroke_cloth_filter.rs`](../../../crates/ph2d-sculpt3d/src/stroke_cloth_filter.rs).
- Gates: [`mede_o_tecido_que_estica.rs`](../../../crates/ph2d-sculpt3d/tests/it/mede_o_tecido_que_estica.rs)
  — dez, e as **três** novas provadas por mutação (a persistência do material, a
  invalidação dela, e a lei de dobra).
