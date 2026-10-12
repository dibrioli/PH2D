# ARQUIVO — 114_ciclo_9_rig_e_corpos_moles.md (história, 830 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`114_ciclo_9_rig_e_corpos_moles.md`](../../Motion%20Nodes/114_ciclo_9_rig_e_corpos_moles.md) em 2026-10-12, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-92,119-138,213-273,477-542,615-619,648-696,726-742,825-858,914-976,991-1012,1124-1141,1167-1225,1258-1280` do original.
>
> ⚠️ **A única alteração ao corpo:** 7 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §2.1 — E agora quem o diz é o PLANEADOR (W0, fechada)

⭐ As duas leituras acima são do **registo**. A que vale para o artista é a do **planeador** sobre a
cadeia montada, e ela existe: `the_rig_group_route_only_improves`
([`motion_rig_probe.rs`](../../../crates/ph2d-app-motion/src/motion_rig_probe.rs)), com a lista
`NA_CPU` a nascer com **nove** nomes e a razão de cada um. **Ela passa** ⇒ o planeador confirma, nó
a nó, o que o registo dizia.

⚠️⚠️ **A forma da cadeia é diferente para cada metade, e a escolha é o MÉTODO:**

| metade | cadeia medida | porquê |
|---|---|---|
| **PRODUZ** | `X → scale → output` | ela é o primeiro nó; um `X` sem kernel derruba tudo |
| **AGE** | `motion.grid → scale → X → output` | mede-se atrás de uma fonte que **já está no dispositivo** |

⛔ **A metade que AGE não pode ser medida atrás do `rig.skeleton`**, que é a cadeia que o artista de
facto escreve: ele próprio não tem kernel, logo a cadeia cai para a CPU **por causa da fonte** e o
nó medido nunca é a causa — *uma régua em que o sujeito não pode falhar sozinho não mede o sujeito*.

⭐ **E a catraca tem CONTROLO POSITIVO dentro:** sem ele, um `cadeia_no_dispositivo` que devolvesse
sempre `false` deixaria as nove asserções verdes e a décima nunca correria — *a catraca ler-se-ia
como a funcionar sobre um instrumento morto*. **Prova de mutação 2 de 2**: a régua cravada em
`false` mata o controlo, e tirar **um** nome da `NA_CPU` faz o laço acusá-lo pelo nome.

---

### ⛔⛔ A CANETA EXISTE — e a §0 da folha está REFUTADA por medição (2026-09-17)

A frase acima era verdade **em 2026-08-09** e já não é. O `motion.drive` ganhou desde então o canal
**`Custom…`** (`CH_CUSTOM = 9`) mais o text param `column`, e com eles escreve **qualquer** coluna
com os **oito** modos dele (`Add · Set · Multiply · Subtract · Divide · Min · Max · Remap`). A folha
mede-o como *«cobre X/Y/Rotation/Size/Opacity e nada mais»* — eram cinco, hoje são cinco **mais
todas**.

⭐ **MEDIDO pelo caminho do produto**
(`a_caneta_que_a_folha_diz_nao_existir_ja_escreve_o_comprimento_do_osso`), na cadeia
`rig.skeleton(joints=4, length=1) → motion.drive(Custom, "len") ← value.instance_field(Ramp)`, com o
`rig.fk` a resolver a pose:

| | comprimentos que o `rig.fk` resolve |
|---|---|
| **sem a caneta** (o CONTROLO) | `1,000 · 1,000 · 1,000` |
| **com** `drive(Custom, "len")` | **`0,333 · 0,667 · 1,000`** |

⚠️ **Mede-se a GEOMETRIA e não a coluna:** ler `len` de volta provaria que o `motion.drive`
escreveu, e a pergunta é se **o solver obedece**. São duas afirmações e só a segunda fecha a célula.
⚠️ E o **CONTROLO é metade do valor** — sem a cadeia sem-escritor a devolver comprimentos iguais,
*«eles variam»* não distinguiria a caneta de um esqueleto que já nascia irregular.

⛔⛔ **MAS a caneta é NECESSÁRIA e não SUFICIENTE, e é aqui que o diagnóstico fica mais preciso que
o da folha.** Das seis células que ela atribui à caneta ausente:

| célula | o que falta HOJE |
|---|---|
| comprimento por osso | ✅ **nada** — medido acima |
| ramificação | ✅ nada — o text param `branches` fechou-a em 2026-08-12 (a própria folha o diz) |
| **peso por osso** (P0) | ⛔ o **LEITOR**: o `rig.skin_deformer` lê `P`·`parent`·`len`·`rot`·`wrot` e **nenhuma coluna de peso** |
| limite de junta | ⛔ o LEITOR, no `rig.fk`/`rig.fabrik` |
| rigidez por junta | ⛔ o LEITOR, no `rig.fabrik` |
| stretch / compress | ⛔ o LEITOR, no `rig.ik_2bone` |

⇒ ⭐⭐⭐ **o que bloqueia quatro das seis não é escrever a coluna: é o SOLVER não a ler.** *Uma
ausência afirmada sem olhar a API é um palpite com cara de medição* — e desta vez a nota errada era
sobre um nó que a própria casa tinha alargado três semanas depois de ela ser escrita.

**Os três buracos de maior alcance, na ordem em que a folha os mede:**

1. **`Strength` / `Mix`** — Rive tem em **7 de 7** constraints, Spine em **4 de 4**, nós em **0 de
   6**. *Um item, sete lugares.* ⭐ E a §SUPERAR da folha mostra que aqui ele nasce **melhor que a
   referência**: nas três, o `Mix` é um número keyado; aqui um esqueleto é uma corrente ordinária,
   logo o strength é uma **COLUNA** — e a família `field.*` inteira (com gizmo de canvas) já a
   produz. *«Um IK cuja influência desvanece com a distância de uma caixa que o artista arrasta»*
   não existe em nenhuma das três.
2. **O peso por osso do `rig.skin_deformer`** (P0 na folha) — *«o item que todo rigger encontra no
   primeiro dia»*. Hoje há **um** expoente global (`falloff`, inteiro 1..8) onde Spine e Rive
   **pintam** peso por vértice e por osso.
3. **A ALÇA no canvas** (P0) — hoje o alvo do IK é *«o primeiro elemento de uma corrente»*, uma
   convenção **invisível** onde as três referências têm uma alça que se arrasta. ⚠️ É UI e não
   param, e é o que torna a família **usável**.

⚠️ **E um item da folha é um DEFEITO, não um pedido:** o `motion.look_at` escreve o heading de
**MUNDO** numa coluna que o `rig.fk` lê como **LOCAL** ⇒ acerta a raiz e **rasga** tudo abaixo dela
(cerca 2 da folha: `rot` é LOCAL, `P` é DERIVADO). Ele está derivado do código e **pendente de gate
red-first**.

### §3.2 — Os quatro corpos moles: MUITOS dials, nenhuma placa

Params pelo **registry** (§2), 2026-09-17: `soft_body` **10** · `verlet_rope` **11** · `wave`
**10** · `boids` **16**, e os quatro são `Temporal`. ⇒ **a doença é a oposta** da dos `rig.*`: aqui
os dials existem — e quase todos chegam ao cartão — e o que falta é a **rota**.

| nó | o que está aberto hoje | fonte |
|---|---|---|
| `motion.wave` | ⏳ **[Bug #7](../../Motion%20Nodes/BUGS_motion_nodes.md)**, report do dono: a fileira de mar de **4 ondas não mostra cristas diferentes** — a BOIA é um passa-baixo e apaga as camadas finas (excursão `0,228` contra `0,377` da de 1 onda). A alavanca medida é o **calado**; ⛔ **não** a densidade, que reabre o Bug #6 | CLAUDE.md §5 |
| `motion.wave` · `motion.boids` | ⏳ **tectos por medir** — o bloco Z do [doc 91](../../Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md) fechou 7 células e deixou estas duas de fora | doc 91 |
| `motion.verlet_rope` | a composição **sub-passos × `damping`**, medida e **não curada de propósito**; ⚠️ o sub-passo local dele chama-se `solver_substeps` e **não** `substeps` — enquanto usava a mesma chave, o app corria as duas leis e a corda caía **4,8× menos** que os gates dela medem | CLAUDE.md §5 |
| `motion.soft_body` | ✅ o P1 dele FECHOU (a porta `shape`: a nuvem que chega é a forma de repouso) — a folha 03 está a **zero P1** | CLAUDE.md §5 |

---

## §7 — ✅ W5: A MEDIÇÃO do grupo (passo 5)

Corrida em **RELEASE**, `load 22,88` (`/proc/loadavg` impresso pela própria sonda, §0.0), pela ponte
do produto: `<nó> → motion.output` com a aresta de estado ATRASADA sobre si mesmo, mediana de **9**
quadros depois de **60** tiques de aquecimento. Um quadro de 60 fps tem **16,67 ms**.

⚠️⚠️ **A carga LÊ-SE NA TABELA e não se corrige:** cada número aqui é um **tecto**, nunca um valor
nominal — a máquina estava a `22,88` com outra linha a cozinhar. Isso é conclusivo **numa direcção
só**: o que já cabe no quadro AQUI cabe sempre; o que não cabe, não se pode absolver com esta
corrida. (O `motion.wave` a `512` lê `1,322` aqui e leu `1,001` a `load 16,17` na W4-bis — `+32 %`
de deriva de carga sobre o MESMO binário, que é a escala do ruído que esta coluna carrega.)

| nó | lado | linhas | quadro/ms | ns/linha | % de um quadro |
|---|---:|---:|---:|---:|---:|
| `motion.wave` | 16 | 256 | 0,001 | 5,6 | 0,01 % |
| `motion.wave` | 32 | 1 024 | 0,004 | 3,6 | 0,02 % |
| `motion.wave` | 60 | 3 600 | 0,010 | 2,9 | 0,06 % |
| `motion.wave` | 128 | 16 384 | 0,046 | 2,8 | 0,28 % |
| `motion.wave` | 256 | 65 536 | 0,207 | 3,2 | 1,24 % |
| **`motion.wave`** | **512** | **262 144** | **1,322** | **5,0** | **7,93 %** |
| `motion.soft_body` | 16 | 256 | 0,002 | 8,3 | 0,01 % |
| `motion.soft_body` | 32 | 1 024 | 0,006 | 6,1 | 0,04 % |
| `motion.soft_body` | 60 | 3 600 | 0,020 | 5,5 | 0,12 % |
| `motion.soft_body` | 128 | 16 384 | 0,094 | 5,7 | 0,56 % |
| `motion.soft_body` | 256 | 65 536 | 0,373 | 5,7 | 2,24 % |
| **`motion.soft_body`** | **512** | **262 144** | **2,259** | **8,6** | **13,55 %** |
| `motion.verlet_rope` | 16 | 256 | 0,070 | 272,7 | 0,42 % |
| `motion.verlet_rope` | 32 | 1 024 | 0,281 | 274,0 | 1,69 % |
| `motion.verlet_rope` | 60 | 3 600 | 0,992 | 275,5 | 5,95 % |
| `motion.boids` | 16 | 256 | 0,044 | 172,3 | 0,26 % |
| `motion.boids` | 32 | 1 024 | 1,099 | 1 073,4 | 6,59 % |
| **`motion.boids`** | **60** | **3 600** | **13,770** | **3 825,0** | **82,6 %** |

⚠️ **As duas escadas não são a mesma escada, e a coluna `linhas` é quem o diz:** um campo 2D cresce
com a ÁREA (`lado²` células) e uma corda é 1D — a sonda dá aos nós de CONTAGEM o **lado ao quadrado**
como contagem, justamente para as duas colunas serem comparáveis. ⛔ O `ns/linha` **só** se compara
entre linhas da mesma contagem.

---

### §7.1 — O que a tabela decide, uma leitura por nó

⭐⭐⭐ **(1) O IRMÃO MAIS CARO É QUEM TINHA O TECTO MAIS LARGO — e agora está medido na escada
inteira, não num ponto.** O `motion.soft_body` custa `1,71×` o `motion.wave` a `512²` (`2,259`
contra `1,322`) e `1,80×` a `256²`, e **já shipava com `MAX_SIDE = 512`** enquanto o mais barato
estava preso em `60`. *A W4-bis não escolheu um número: ela copiou o do irmão, e esta tabela mostra
que o irmão o suporta com folga maior do que o novo dono precisa.*

⭐⭐ **(2) O `ns/linha` do campo tem VALE, e o vale é a memória.** `5,6 → 3,6 → 2,9 → 2,8` e depois
`3,2 → 5,0`: o custo por célula **desce** enquanto a grelha cabe em cache (o estêncil de 5 pontos lê
os quatro vizinhos) e **volta a subir** quando ela deixa de caber — a `512²` são `262 144` células ×
(estado + saída), e o passeio deixa de ser servido pelo L2. ⇒ *o recurso do tecto do campo é a
LARGURA DE BANDA de memória, e não a aritmética* — o que torna `512` um sítio honesto para parar
**hoje**, e o kernel de GPU a resposta certa para o degrau seguinte (lá a banda é outra ordem de
grandeza).

⭐⭐⭐ **(3) A CORDA É PLANA, e é isso que fecha a decisão de a deixar em Gauss-Seidel.** `272,7 ·
274,0 · 275,5` ns/ponto sobre uma faixa de **14×** na contagem — variação de `1,0 %`. ⇒ o custo é
**estritamente linear** nos pontos, logo uma corda do tamanho que alguém de facto faz (100–200
pontos) custa **`0,027`–`0,055 ms`**, `0,2 %`–`0,3 %` de um quadro. *Paralelizar a relaxação
compraria um número que já é ruído e pagaria com a CONVERGÊNCIA, que é o resultado.* ⛔ O gatilho de
reabertura fica nomeado e é o mesmo: cordas de dezenas de milhares de pontos — e a tabela diz quando
isso morde (`3 600` pontos já são `5,95 %` de um quadro).

⛔⛔⛔ **(4) O PREÇO DO RECUO ESTÁ MEDIDO, e é o número mais violento desta tabela: `13,770 ms`.**
O `motion.boids` é o **único** dos dez nós do grupo que chega ao dispositivo (§2), e este arnês coze
com o `Cook` da **CPU** — logo o que esta linha mede é a **rota lenta de quem tem placa**, que é
exactamente o que a lei 1 do doc 103 §2 manda medir. A `3 600` agentes ele come **82,6 % de um
quadro** sozinho, e a escada diz porquê:

| de → para | vezes mais agentes | vezes mais caro | expoente |
|---|---:|---:|---:|
| 256 → 1 024 | 4,00× | 25,0× | **2,32** |
| 1 024 → 3 600 | 3,52× | 12,5× | **2,01** |

⇒ **`O(N²)` confirmado pela porta do produto** (cada agente consulta todos os outros), com o
expoente a assentar em `2,0` assim que a contagem sai do regime em que o custo fixo ainda pesa.
⭐ *É por isso que a escada dos nós de contagem pára em `60` e não é preguiça: `512² = 262 144`
agentes na rota da CPU custariam `~5,3 × 10⁶ ms` pela mesma lei — a corrida penduraria, e uma escada
que ninguém sobe mede um programa que ninguém corre.*

⚠️ **E isto é a MELHOR notícia da tabela, não a pior:** o número grande é o preço de **não** estar na
placa, e ele mede-se **num nó que tem para onde ir**. Os outros nove não têm — é isso que a §8 nomeia
um a um.

---

### §7.2 — O que a tabela NÃO diz

⛔ **Ela não mede a cadeia.** Cada linha é **um** nó entre a fonte e o `motion.output`; uma cena real
empilha `field.*`, `motion.scale` e o resto por cima, e o custo do grupo numa cena cheia é outra
medição (a do doc 98, que já existe e mede o MÓDULO).

⛔ **Ela não mede o dispositivo.** A coluna `quadro/ms` é sempre CPU, **incluindo** a do
`motion.boids` — *um controlo que não percorre o mesmo caminho não controla nada*, e a sonda diz
isso de si mesma no doc-comment da tabela `MOLES`.

⚠️ **E o `ns/linha` da corda e do bando não se compara com o do campo:** a contagem de um nó 1D é o
lado ao quadrado **por construção da escada**, para as duas caberem no mesmo eixo. Um ponto de corda
custa `~273 ns` e uma célula de campo `~3 ns` porque são trabalhos diferentes (a corda faz
`solver_substeps` passagens de relaxação sobre as restrições; o campo faz **um** estêncil).


---

## §6 — ✅ W1′: o ENVELOPE POR OSSO (o P0 da folha 16)

A lei do `rig.skin_deformer` passa de `w_j ∝ 1/d_j^falloff` para **`w_j ∝ envelope_j / d_j^falloff`**,
com o `envelope_j` a sair da coluna opcional **`bone_weight`** do `rest`. É o *envelope weight* por
osso do Blender, os *Tendons* do Rive e o *dropoff* por influência do Maya — onde nós tínhamos **um
expoente global**.

⭐ **Ausente ⇒ `1,0` ⇒ a lei de ontem AO BIT**, e o gate afirma-o com `assert_eq!` sobre os bits, não
com uma barra de tolerância: `x * 1,0` é exacto em IEEE-754, logo **nenhum documento já autorado
muda de pixel**.

⛔ **O nome NÃO é `weight`, e isso foi medido:** `weight` já é coluna deste repo — a espessura da
fonte, escrita pelo `source.text`. *Uma colisão de nome de coluna passa MUDA* (§5.0), e ali ela
juntaria a pele de um esqueleto ao peso de um glifo.

⚠️ **O envelope viaja no OSSO e não num índice**, porque o construtor de ossos **FILTRA**: a junta
sem pai não produz osso, logo **o osso `k` não é a junta `k`**. Quem indexasse a coluna pelo índice
do osso daria a cada um o envelope do vizinho — em silêncio, com a pele a deformar-se com ar de
certa. Há gate cuja fixtura separa as duas leituras (o envelope da RAIZ não pode mudar nada).

⛔⛔ **A FIXTURA era metade do gate, e a primeira redacção estava degenerada:** os três gates usavam
uma rotação **RÍGIDA** do esqueleto, e numa rotação rígida todo osso sofre a MESMA mudança de
referencial ⇒ *a pele sai no mesmo sítio seja qual for o peso*. O teste vizinho, escrito há meses,
**afirma isso por escrito**. Com ela, o gate do envelope a zero reprovava sobre produto **correcto**
e o gate da raiz passava por **vácuo**. ⇒ a pose passou a DOBRAR numa junta do meio.

⭐⭐ **E a costura foi medida pelo caminho do produto** — o §5.0 diz que *nenhum instrumento deste
repo pergunta se o VALOR chega a um consumidor*:

```text
motion.grid ──────────────────────────────────────────> skin.in
rig.skeleton ─[drive(Custom,"bone_weight") ← rampa]─> fk ──> skin.rest
             └─[drive(Custom,"rot")        ← rampa]─> fk ──> skin.posed
```

A caneta escreve, o valor atravessa um `rig.fk` e o solver obedece. **Prova de mutação 3 de 3**: com
o leitor a ignorar a coluna o gate de costura lê **`0e0` de desvio** — exactamente zero, que é o
sinal mais forte que uma régua destas pode dar.

⏳ **ABERTO e nomeado:**

- **Todos os envelopes a zero** caem no ramo que já existia (*repartir por igual em vez de dividir
  por zero*), escrito para a inalcançabilidade GEOMÉTRICA. Dizer *«o ponto não se move»* é o que um
  rig zerado quer, e a função de pesos não o sabe exprimir — **decisão de produto**, com o preço no
  cabeçalho do nó.
- **Descoberta:** a coluna escreve-se digitando `bone_weight` no campo *Column* do `motion.drive`,
  como toda coluna deste catálogo. Não há fileira de painel — e isso é a cerca 3 da folha (*um dial
  aqui seria uma 2.ª fonte de verdade*), não um esquecimento.

---

## §7 — ✅ W2: a FORÇA de uma restrição, e ela é a coluna que já existia

O `Strength` que o **Rive** põe em **7 de 7** constraints e o `Mix` que o **Spine** põe em **4 de 4**
— nós tínhamos **0 de 6**. Agora os três solvers (`rig.ik_2bone` · `rig.fabrik` ·
`rig.rubber_hose`) misturam a pose resolvida com a que entrou, junta a junta.

⭐⭐⭐ **E o canal NÃO é uma coluna nova: é o `falloff`.** Foi a segunda vez neste ciclo que medir
antes de construir poupou trabalho — a folha pedia um `strength`, e `falloff` é exactamente *«quanto
este elemento participa»* no vocabulário desta casa: é o que o `motion.falloff` escreve, o que a
família `field.*` inteira produz **com gizmo de canvas**, e o que o `motion.scale` já lê com a mesma
convenção (*ausente vale `1,0`, efeito cheio*).

⇒ ⭐⭐ **o item `SUPERAR:` nº 1 da folha cai de graça.** Nas três referências o `Strength` é **um
número keyado**; aqui *«um IK cuja influência desvanece com a distância de uma caixa que o artista
arrasta na tela»* é **um fio**, e não existe em nenhuma delas.

**A escada, gateada nos três nós:**

| força | o que sai |
|---|---|
| `0` | a pose que ENTROU, **ao bit** — a restrição não faz nada |
| `½` | *estritamente* entre as duas (senão «mistura» seria um interruptor) |
| `1` **ou ausente** | o solve de sempre, **ao bit** |

⚠️ **A mistura é de ÂNGULOS e nunca de posições** — a cerca 4 da folha, que a própria célula do
`Strength` já invocava: um `P` misturado discorda do `rot` e a cadeia sai **rasgada**. É também por
isso que o `motion.mixer` não servia como rota.

⛔⛔ **E uma mutação REFUTOU um comentário meu, escrito minutos antes.** Eu justifiquei o atalho de
força-cheia como *«o que mantém a saída byte-exacta»*; apagando-o, **os três gates da escada
continuam verdes** — a exactidão vem da FORMA (`a·(1−t) + b·t` em `t = 1` é `a·0 + b·1 = b`,
exacto), e não daquela linha. *Uma linha que a mutação não consegue matar não é lei, é comentário
com sintaxe de código* — o atalho fica com o nome que merece (poupa uma alocação e uma passagem no
caso comum) e o comentário foi corrigido no mesmo commit.

⚠️ **A forma importa e a alternativa é a armadilha:** o `a + (b−a)·t` dos manuais dá, em `t = 1`,
`a + (b−a)` — que **arredonda** quando `a` e `b` estão longe. *Uma das duas formas mexe em silêncio
com todo rig do repositório.*

⚠️ O `t` é preso a `0..1`: uma força é uma **mistura**, e um campo que entregue `2,0` não pode fazer
uma restrição ultrapassar o próprio solve.

**Prova de mutação 2 de 2** (a segunda documentada como não-sangrante de propósito, acima), e a
folha `pose.rs` continua **byte-idêntica** nas três crates, que é a cerca 10.

---

## §9 — ✅ W4-bis: o TECTO DO CAMPO, medido — e as três decisões do grupo

Ordem do dono (*«decida pelo padrão ouro buscando o melhor resultado e performance»*). As três
decisões abaixo saem todas da mesma tabela, medida pelo caminho do produto em **RELEASE**.

### A medição

| lado | células | quadro/ms | ns/célula | % de um quadro de 60 fps |
|---|---|---|---|---|
| 16 | 256 | 0,001 | 5,7 | 0,01 % |
| 32 | 1 024 | 0,004 | 3,6 | 0,02 % |
| **60** | **3 600** | **0,010** | 2,8 | **0,06 %** ← o tecto ANTIGO |
| 128 | 16 384 | 0,046 | 2,8 | 0,28 % |
| 256 | 65 536 | 0,240 | 3,7 | 1,44 % |
| **512** | **262 144** | **1,001** | 3,8 | **6,00 %** |

⭐⭐ **A leitura correu a `load 16,17` e é conclusiva pelo lado SEGURO.** Sob carga um relógio só
pode ler **pior**, logo um número que cabe ali cabe na máquina calma — é a lei do `CLAUDE.md` §5.0
(*um green sob carga é conclusivo; um red sob carga não prova nada*) usada na direcção em que ela
funciona. ⭐ E o `ns/célula` é **plano** (`2,8`–`3,8`): o estêncil é `O(células)` sem joelho nenhum.

### Decisão 1 — `motion.wave::MAX_SIDE` sobe de `60` para **`512`**

⛔ A `60` este nó usava **`0,06 %`** de um quadro: o tecto estava **~100× abaixo de qualquer
recurso**, e a justificação escrita ao lado dele (*«field cost is O(rows·cols)»*) era uma lei de
crescimento e não um recurso — §0.0 exactamente.

⭐⭐⭐ **E o número não foi escolhido: é o do IRMÃO.** O `motion.soft_body` é a outra simulação de
grelha 2D desta casa, o tecto dele é **`512`**, e ele custa **`2,32 ms`** a `512²` contra os
**`1,00`** deste. ⇒ *o nó mais BARATO tinha o tecto mais apertado*. Alinhá-los faz a família
responder a mesma coisa à mesma pergunta, e o recurso passa a estar nomeado: **o orçamento do
quadro** (`6 %` para o campo sozinho).

⚠️ **E o tecto DIGITÁVEL passou a existir.** Antes, `clamp` e slider valiam ambos `60`: *a
capacidade do motor acabava onde o dedo acabava*. Agora são o par que o irmão já shipa e que o
[doc 91](../../Motion%20Nodes/91_os_tetos_que_ninguem_mediu.md) pôs em 25 params — clamp `512`, slider `64` (a faixa de
autoria, estritamente abaixo).

⛔⛔ **E uma MUTAÇÃO SOBREVIVEU, e ela mudou o desenho dos gates.** O gate que escrevi dentro da
crate do nó monta os `Params` **à mão** e chama o `simulate` — ele nunca atravessa o
`clamp(2, MAX_SIDE)`, que vive no `eval`. Encolhi esse clamp para `60` e o gate ficou **VERDE**:
*um arnês que monta o estado à mão mede a LEI e não a PORTA*, a forma que esta casa já pagou quatro
vezes noutros módulos. ⇒ o gate do produto vive agora onde um grafo se coze de verdade
(`o_campo_chega_ao_tecto_pela_porta_do_produto`), e com a mutação ele diz *«o artista pediu 512×512
e recebeu 3 600 células»*. **3 de 3 mutações sangram.**

### Decisão 2 — a `motion.verlet_rope` FICA em Gauss-Seidel

Medido: **`278` ns por ponto, e a coluna é PLANA** (`276,9` · `277,8` · `278,1`). Uma corda com os
`100`–`200` pontos que alguém de facto autora custa **`0,03`–`0,06 ms`** — `0,3 %` de um quadro.

⇒ ⭐⭐ **trocar a lei por Jacobi ou por coloração compraria paralelismo numa contagem que ninguém
alcança, e pagaria com o RESULTADO**: para o mesmo número de iterações a corda fica **mais mole**,
porque uma correcção deixa de ver a anterior. O padrão-ouro aqui é *não mexer* — a convergência
exacta é a feature, e os gates que medem quanto ela cai ficam todos de pé.

⚠️ **É uma decisão e não uma omissão**, e fica registada com o número ao lado. O gatilho que a
reabriria está nomeado: alguém querer cordas de dezenas de milhares de pontos (tecido, cabelo em
massa), que é outro produto.

### Decisão 3 — o kernel de GPU do `motion.wave` é o PRÓXIMO tecto, não este

O §0.0 diz que *quem manda no tecto é o dispositivo*. Hoje não há kernel, logo `512` é o que o
**caminho de referência** sustenta — e está nomeado como tal. ⭐ O que a medição mudou é o **preço da
espera**: com `512²` a custar `1 ms` na CPU, o artista ganha **`73×` mais células hoje**, sem uma
linha de WGSL e sem nenhum risco de divergência CPU/GPU.

⇒ **o kernel deixa de ser o que destrava o nó e passa a ser o que o leva de `512` a milhões** — com
o `ns/célula` plano a dizer exactamente quanto ele compraria. *Uma optimização com o número ao lado
é uma decisão; sem ele era uma aposta.*
---

### §10.1 — A cena, e por que ela é esta

`PH2D_GPU_COOK_DEMO=120` — seis panos em três fileiras, cada fileira um par *«o base | o base mais
uma coisa»*:

| fileira | a pergunta | esquerda | direita |
|---|---|---|---|
| **cima** | e se a coisa **se segurar sozinha**? | `Verlet Rope` | `Wave` |
| **meio** | e se **eu** quiser segurá-la? | `FK` (eu digo o ângulo) | `IK 2-Bone` (eu digo a mão) |
| **baixo** | quanto é que **cada osso** segura? | a pele com todos iguais | a pele com **quinhão** |

⭐ **A partição em duas categorias É o grupo** (§1) e não arrumação: a fileira de cima são nós
**Source**, as outras duas são **Transform**. É essa a premissa do tutorial.

⭐⭐ **Cada wave deste ciclo tem um passo que o dono EXECUTA**, e é isso que a torna entregue:

| wave | o passo | a alavanca |
|---|---|---|
| **W4-bis** (§9) | escrever `512`/`512`/`0,004` no campo | `Rows` · `Cols` · `Spacing` |
| **W2** (§7-W2) | apertar o raio do cartão `Strength` | `Radius`, `20 → 1` |
| **W1′** (§6) | mexer no fim da banda do quinhão | `End` do `Range: que ossos puxam` |

⛔ **QUATRO dos dez nós não estão na cena, com motivo:** `motion.soft_body` e `motion.boids`
produzem como a corda e o campo e já têm cena própria; `rig.fabrik` e `rig.rubber_hose` são **a
mesma lei do `ik_2bone`** com outra contagem de juntas — pô-los lado a lado ensinaria *«há três
nomes»*, que é o oposto de *«a mão vai ao alvo»*. ⭐ E a força dos três é a **mesma** coluna, logo o
passo da W2 vale para todos.

### §10.4 — O que fica GATEADO

| gate | o que ele afirma |
|---|---|
| `a_cena_monta_seis_panos_e_nenhum_vem_vazio` | os seis panos, com piso de população |
| `a_corda_balanca_e_o_campo_ondula` | a fileira de cima mexe-se — pela **assinatura** (posição **e** tamanho) |
| `a_cinematica_directa_e_a_inversa_dao_panos_diferentes` | o par do meio não é o mesmo nó duas vezes |
| `a_mao_segue_o_alvo_e_o_pano_do_fk_nao_se_mexe` | a mão segue — com o FK `Pure` como CONTROLO |
| `apertar_a_forca_da_restricao_muda_a_pose_do_ik` | a W2, pelo BARRO e não pelo param |
| `o_quinhao_por_osso_muda_a_pele` · `o_quinhao_a_zero_devolve_a_pele_ao_repouso` | a W1′, com a barra do que se **vê** |
| `o_passo_do_tecto_entrega_meio_milhao_de_celulas` | a W4-bis, pela porta da cena |
| `every_row_the_rig_tutorial_names_is_on_the_card` | cada linha que o PDF nomeia está no cartão |
| `every_figure_the_rig_tutorial_shows_exists` | as seis figuras existem, contadas |
| `the_rig_tutorial_opens_the_scene_this_cycle_built` | ele abre a `=120` (metade em COMPILAÇÃO) |
| `the_cost_table_the_rig_tutorial_prints_is_the_one_that_was_measured` | os números da §7 do PDF são os da §7 deste doc |

⚠️ **O último NÃO re-mede:** medir num gate fá-lo-ia membro da família de flakes de carga
(`CLAUDE.md` §5.0), e o que se defende ali é a **honestidade do texto**, não o relógio. *Uma tabela
de custo num PDF é a afirmação mais fácil de deixar apodrecer do repo — ela não compila, não corre
e ninguém a relê.*

⚠️ **E um gate do ciclo 8 teve a premissa MORTA**, que é o gate a funcionar: ele afirmava
`!is_cycle_scene("120")` — *«a tabela não pode responder por uma cena que não existe»* — e este
ciclo construiu-a. ⛔ O defeito não era o número, era a **forma**: um gate escrito sobre o SUCESSOR
de hoje reprova no dia em que alguém escrever o ciclo seguinte, **sobre produto correcto**. ⇒
reescrito DERIVADO do tecto (`MAX_DEMO_LEVEL + 1`), com a morte visível no diff.

---

### §11.2 — (1) A corda não colide consigo mesma, e vê-se no solver

A relaxação do `motion.verlet_rope` tem **exactamente duas** restrições: `i↔i+1` (a distância de
repouso) e `i↔i+2` (a flexão, só com `bend > 0`). **Nenhuma `i↔j`** para pares afastados — uma volta
do laço atravessa a outra porque *nada no solver olha para esse par*.

Medido pela porta do produto (25 pontos, âncora chicoteada, 240 tiques, discos de raio `0,07` ⇒ a
barra é `0,140`):

| arranjo | menor vão entre não-vizinhos | pares sobrepostos |
|---|---:|---:|
| **a corda NUA** | **`0,0079`** | **13** |
| `+ motion.collide` depois | `0,1051` | 9 |
| `+ motion.collide` no laço de estado | `0,1063` | 7 |

⇒ `0,0079` sobre uma barra de `0,140` é **um ponto praticamente em cima do outro**.

⚠️ **O chicote é parte da régua, não cenografia:** uma corda pendurada em repouso é uma catenária e
**nunca se toca**. Medi-la em paz responderia *«não há sobreposição»* sobre um arranjo que não a
pode ter — *uma régua que não vê o fenómeno acontecer não prova que ele não aconteceu.*

### §11.3 — (2) O `Collide` da forma não tem por onde chegar

O `source.shape` **declara** o colisor em COLUNAS (`COLLIDER_COLUMN` · `COLLIDER_BOX_COLUMN` ·
`COLLIDER_OFFSET_COLUMN`), e o censo de quem as **lê** dá três crates, todas da família da
simulação: `ph2d-node-sim-collide` · `ph2d-node-sim-step` · `ph2d-contact`.

⛔ **E o `source.shape` é uma FONTE — `inputs: &[]`.** Ela não pode estar a jusante da corda; o
caminho por que um colisor declarado chega a um solver é `source.shape → sim.spawn(template) →
sim.zone/sim.step → sim.collide`, que é a pilha do **ciclo 5**.

As portas de entrada de cada nó, contadas do manifesto:

| nó | entradas | as portas |
|---|---:|---|
| `motion.verlet_rope` | 3 | `anchor_x` · `anchor_y` · `state` |
| `motion.wave` | 3 | `drive` · `state` · `inject` |
| `motion.soft_body` | 4 | `anchor_x` · `anchor_y` · `state` · **`shape`** |
| `motion.boids` | 4 | `target_x` · `target_y` · `state` · **`obstacle`** |
| `rig.skeleton` | 0 | — |
| `rig.skin_deformer` | 3 | `in` · `rest` · `posed` |
| `sim.collide` | 1 | `in` |

⚠️⚠️ **DUAS portas parecem ser o que não são, e é por isso que esta tabela está aqui:**

- O **`shape`** do `motion.soft_body` é a **forma de REPOUSO** (a wave da folha 03), não um colisor.
- O **`obstacle`** do `motion.boids` é uma nuvem de **pontos a evitar**, e o que ele aplica é uma
  **força de direcção** (`avoid_accel`, com `avoid_radius` e `lookahead`) — ⛔ *evitar não é
  colidir*: dois agentes continuam a poder ocupar o mesmo sítio, e nada ali lê as colunas do
  colisor.

⇒ **a segunda frase do report está CERTA:** nenhum dos dez lê o colisor que a forma declara.

### §11.4 — ⭐⭐⭐ (3) E a composição entrega-o POR INTEIRO, a um cartão de distância

O `motion.collide` é um separador de não-penetração a sério (PBD, Müller et al. 2007), é
`Effect::Pure` e aceita **qualquer** nuvem — inclusive a da corda. Pondo-o **dentro do laço de
estado** (`rope → collide → (atrasada) → rope.state`) e varrendo as iterações dele:

| iterações do `collide` | menor vão | pares sobrepostos | % da barra |
|---:|---:|---:|---:|
| 8 (o valor de fábrica) | `0,1063` | 7 | 76 % |
| 16 | `0,1284` | 3 | 92 % |
| **32** | `0,1383` | **0** | **99 %** |
| 64 | `0,1399` | 0 | 100 % |
| 128 | `0,1399` | 0 | 100 % |

⇒ **a `32` iterações a corda deixa de se atravessar, medido: ZERO pares sobrepostos.**

E o preço (RELEASE, mediana de 9 quadros em regime; um quadro tem `16,67 ms`):

| pontos da corda | a corda só | `+ collide` a 32 | % de um quadro |
|---:|---:|---:|---:|
| 25 | `0,006 ms` | `0,017 ms` | `0,1 %` |
| 50 | `0,013 ms` | `0,060 ms` | `0,4 %` |
| 100 | `0,027 ms` | `0,211 ms` | `1,3 %` |
| 200 | `0,053 ms` | `0,785 ms` | `4,7 %` |

⇒ uma corda do tamanho que alguém faz (100–200 pontos) com auto-colisão **completa** custa
`1,3 %`–`4,7 %` de um quadro. ⚠️ O `motion.collide` é `O(n²·iterações)` na CPU — a `200` pontos e
`32` iterações são `1,28 M` testes de par por quadro, e é daí que vem o `4,7 %`.

### §12.1 — O censo DERIVADO: quem é duplicador, e o que ele faz ao colisor

⛔ **A população sai do REGISTO, nunca de uma lista escrita à mão** — *«todo e qualquer»* não se
adivinha. Sonda
[`motion_colisor_duplicador_probe.rs`](../../../crates/ph2d-app-motion/src/motion_colisor_duplicador_probe.rs):
cada nó recebe `source.shape(Collide)` e conta-se quem devolve **mais peças do que recebeu**.

**14 fontes saltadas** (não têm porta de entrada — não são duplicadores) · **121 varridos** ·
**13 multiplicam**:

| o colisor declarado | quantos | quem |
|---|---:|---|
| ✅ **chega a TODAS as peças** | **5** | `motion.clone` · `motion.mirror` · `motion.kaleidoscope` · `fx.drop_shadow` · `fx.rgb_split` |
| — geram nuvem NOVA (nunca receberam a forma) | 8 | `motion.boids` · `motion.distribute_curve` · `motion.distribute_radial` · `motion.lattice` · `motion.soft_body` · `motion.verlet_rope` · `motion.voronoi` · `motion.wave` |
| ⛔ deitam fora um colisor que receberam | **0** | — |

⭐⭐⭐ **O achado: a declaração JÁ chegava intacta a todas as peças dos cinco duplicadores
verdadeiros. O que não existia era um LEITOR.** As três crates que consumiam aquelas colunas eram
todas da família `sim.*` ⇒ o botão só fazia alguma coisa dentro do laço da simulação — *um controlo
vivo e inalcançável a partir de todo o resto do catálogo*, que dá exactamente o mesmo report que um
controlo morto.

⚠️⚠️ **E o discriminador que separa as duas últimas linhas é o `geometry_id`.** Sem ele as duas
leem-se iguais numa tabela de *«o colisor não chegou»* e **as curas são opostas**: uma é um defeito
de passagem, a outra é a pergunta de produto *«uma peça que este nó INVENTA deve herdar o colisor da
forma que entrou?»*. ⛔ A minha primeira leitura acusou o `motion.scatter` de deitar o colisor fora,
e o discriminador **refutou-a**: ele é uma FONTE e nunca recebeu nada.

### §12.2 — A cura: o `motion.collide` honra o colisor DECLARADO

[`declarado.rs`](../../../crates/ph2d-node-motion-collide/src/declarado.rs) — o nó passa a perguntar à
[`ph2d_contact`] (o motor de peça-contra-peça da casa: grelha espacial, caixas orientadas, rotação)
em vez de separar discos de raio uniforme. Medido pela rota do produto
(`source.shape(Collide) → motion.clone(distance 0) → motion.collide`):

| o botão | como as cinco cópias ficam |
|---|---|
| **desligado** | espalhadas na DIAGONAL — `(−0,64,−0,64) … (0,64,0,64)`. *Um disco não tem orientação.* |
| **ligado** | numa COLUNA, todas no mesmo `x` — `(0,00,−1,39) … (0,00,1,39)`. *Caixas arrumam-se como caixas.* |

**Três decisões, cada uma com o porquê no código:**

1. **A declaração GANHA do `Radius` do cartão** — *só quem desenha sabe o tamanho do que desenha*.
2. **Uma peça SEM declaração cai no `Radius`, como disco** — sem isto uma corrente MISTA deixaria
   metade das peças inertes e caladas, e um nó que separa umas e não outras é pior que um que não
   separa nenhuma.
3. ⛔⛔ **O `Strength` e o `falloff` entram por MISTURA no fim, e NUNCA nos pesos.** Multiplicar o
   `inv_mass` das duas peças de um par pelo `strength` fá-lo **CANCELAR** (`λ = pen/(k_a+k_b)`,
   `Δp = n·λ·w`) e o knob ficaria **inerte** — é a armadilha que o `push_apart` do mesmo ficheiro já
   documenta um nível acima, a morder outra vez noutra aritmética.

⭐ **Sem declaração a porta nem abre** (`colisores()` devolve `None`) ⇒ **todo o catálogo que já
existe sai AO BIT**. As 31 do `motion.collide` e a varredura impactada (**4 441 corridos, 4 441
verdes**) afirmam-no.

### §13.1 — O tamanho REAL, contra o que o pedido estimava

| | O pedido dizia | Medido |
|---|---|---|
| mover para fora da crate | «a caixa de correio» (~85 linhas) | **592** linhas em 3 ficheiros — o `ParamRow` inteiro (522) mais a caixa (70) |
| gates de outras crates a partir | «0» | **4** censos textuais (o de rótulos que embrulham, o de hashes nomeados, o registo tipado, o catálogo da paleta) |
| apagar | 7 473 linhas | **7 551** linhas em 35 ficheiros, com **66** testes |

⭐⭐ **A correcção que mudou a wave é a primeira linha:** o que o painel guardava não era só um
canal — era o **vocabulário de rows** de que o CARTÃO se serve. *Ele PINTAVA o `ParamRow`; não o
definia.* Foi por isso que a mudança de casa foi limpa (zero imports no ficheiro movido) e é por
isso que apagar a crate sem o mover teria parado o cartão inteiro, que é exactamente o que o
`CLAUDE.md` §5 avisava por escrito desde 2026-09-07.

### §13.3 — ⛔⛔ E o mesmo aconteceu com DOZE IDS, mas eu só o vi à terceira medição

O ficheiro `motion_param_row_ids.rs` (397 linhas) tinha doze ids e os dois tectos. A primeira
leitura resgatou **três** — as amostras de cor — apoiada numa frase do doc da
[`ph2d_param_editors::EditorKey`]: *«as AMOSTRAS de cor têm prefixo próprio, e não é arrumação: a
shell também as deriva»*.

⛔⛔⛔ **A frase era verdadeira quando foi escrita e falsa no dia em que a li.** Ela descrevia os
dois hospedeiros de então — a row do painel e o cartão. Medido nos SÍTIOS DE CHAMADA, o cartão usa
`motion-card/swatch/{nó}/{canal}` e `card/editor_swatch/{nó}/{param}`: prefixos **com o nó lá
dentro**, que é a razão de ele nem precisar de o nó estar seleccionado. *Nenhum `motion_param/*`
atravessa fronteira nenhuma.*

⇒ o censo por porta deu `produto = 0` para os **doze**, e o ficheiro foi apagado inteiro.

⚠️ **A lei que isto paga, e que esta casa já tem escrita:** *uma ausência — ou uma presença —
afirmada pela PROSA é um palpite com cara de medição.* Eu li o doc-comment de uma chave em vez de
`git grep` nos chamadores dela, e construí uma justificação inteira, com gate novo, sobre o palpite.
O gate teria ficado verde para sempre a medir três ids que ninguém regista.

### §13.4 — O que a remoção de facto encostou, gate a gate

- **`motion_bridge_dock_height_tests.rs`** (555 linhas, 6 testes) — **apagado**: cada teste pintava
  o painel por `MockPanelHost::with_panel::<MotionParamsPanel>` e media rects. *Um teste que se
  apaga com o sujeito é o caso certo.*
- **`motion_bridge_enumcap_tests.rs`** (137 linhas, 3 testes) — **apagado** com o tecto de opções.
- **`motion_bridge_rowcap_tests.rs`** — cortado a meio: saem os dois gates do tecto e a sonda;
  **ficam** a ordenação por secções e a integridade da tabela de grupos, que são leis do
  **snapshot** e nunca foram sobre quem pinta.
- **`the_census_measures_the_fattest_panel_a_gesture_can_reach`** — perde a metade que comparava
  contra o censo do tecto; **fica** a que mede que um gesto revela linhas que o estado de fábrica
  não tem.
- **`the_channel_picker_fits_the_panels_ceiling`** → renomeado
  **`the_channel_picker_offers_the_weight_the_fields_write`**. ⚠️ *Um gate cujo nome promete uma
  propriedade que já não existe é pior que a ausência dele.*
- **`with_the_side_panel_out_the_card_still_writes`** — **fica inteiro**, e é o gate mais
  load-bearing da wave: ele é quem prova que truncar o `publish` não parou o cartão. A metade
  `current_params().is_none()` deixou de ser uma asserção e passou a ser o **compilador** — *uma
  asserção que se torna trivialmente verdadeira lê-se como cobertura e não é nenhuma*.
- **`two_cards_of_the_same_type_never_ask_for_the_same_colour_picker`** — a metade *«mesmo com A
  seleccionado»* virou **estrutural**: o `picker_target_of` deixou de receber o nó seleccionado.

### §13.5 — A ponte de cor perdeu METADE, e o código já a tinha separada

O `motion_bridge_color.rs` tinha, escrito nos próprios comentários, **dois ramos rotulados**: `(a) a
row do painel` e `(b) uma amostra de cartão`. Saiu o (a), e com ele `seed_color_swatches`,
`gradient_picker_stop`, `palette_picker_index`, `gradient_params`, `palette_params` e quatro
argumentos de duas portas (`sel`, `groups`, `grad_params`, `pal_params`).

⭐ **O cartão semeia as amostras dele sozinho** (`register_card_swatches` regista no índice de hits
**e** escreve a cor, por cartão, a cada pintura) — medido antes de cortar a semeadura do painel.

### §13.6 — Os quatro vermelhos que só a varredura IMPACTADA viu

Nenhum deles vive numa crate que a wave editou, e os quatro são **censos de obsolescência a
funcionar**:

| gate | era | é |
|---|---|---|
| `…never_gives_a_row_label_a_wrap_budget` | `ELIDED_TODAY = 30` | **16** (o painel levava 14 rótulos que cortam) |
| `every_non_literal_hash_is_named` | uma entrada nomeando `motion_param_row_ids.rs :: fnv_id` | apagada |
| `build_typed_registry_matches_enabled_features` | 28 painéis | **27** |
| `the_two_ugly_derived_titles_are_named_here` | *"Motion Params"* na paleta | fora |

⚠️ **O terceiro expôs um defeito anterior:** o contador do painel de params era um `{ n += 1; }`
**nu**, sem `cfg` — ele tinha perdido a feature dele numa wave anterior e ninguém reparou. *Um
censo derivado que não consegue explicar uma das suas próprias linhas já não é derivado.*

### §13.7 — E TRÊS vermelhos de clippy severo que eram MEUS, deste ciclo

O `-D warnings` só corre no portão de fecho, e apanhou três coisas que os `cargo check` desta linha
nunca podiam ver:

1. `#[expect(clippy::cast_sign_loss)]` **incumprido** no `ph2d-node-motion-wave` (W4-bis) — a lente
   prova o literal não-negativo e a isenção nunca dispara. *Uma isenção que não é usada lê-se como
   uma cerca a proteger alguma coisa*; virou `usize::try_from`.
2. `items after a test module` em **três** `pose.rs` das crates de rig (W2) — o módulo de teste
   vivia a meio, com `const` e funções por baixo.
3. `empty line after doc comment` no ficheiro movido — um doc-comment que ficou **sem item por
   baixo** quando as declarações de módulo do hub saíram. *O clippy disse à letra o que a nota
   dizia: um hub que sobrevive ao seu edifício documenta o vazio.*

### §13.8 — A porta de escape foi APAGADA, e essa é a diferença entre 09-07 e hoje

Em 2026-09-07 o painel foi **desligado** e o `PH2D_MOTION_PANEL=1` trazia-o de volta: um
interruptor de bissecção honesto, porque a crate existia. Com ela apagada, ele passaria a pôr
**VISÍVEL** um painel que nenhum pintor conhece.

⇒ `painel_lateral()`, a env e a escrita `panel_visibility["motion_params"]` saíram juntas. *Uma
porta de escape que já não pode cumprir o que promete é pior que nenhuma*, porque quem a usar lê o
ecrã inalterado como prova de que o defeito não estava ali.

### §13.9 — Prova de fecho

- `cargo check --workspace --all-targets` — verde, **zero avisos**.
- `scripts/nextest-impacted.sh` — **15 030** testes, 15 030 verdes (a 1.ª corrida acusou os 4 da §13.6).
- `cargo clippy --workspace --all-targets -- -D warnings` — verde (a 1.ª corrida acusou os 3 da §13.7).
- `scripts/censos-da-arvore-combinada.sh` — **87/87**, com o controlo do filtro a `8 de 8`.
- Contadores partilhados: `PROJECT_SCHEMA`, os três registos de componente e os schemas de documento
  **intocados** — esta wave não escreve um único número que some entre linhas.
- Diff: **97 ficheiros, +508 −8 539**.

⏳ **ABERTO e NOMEADO** (nada disto é desta linha):
- o `INSPECTOR_MAX_H` do `ph2d-editor-core` e o `MOTION_PARAMS_SCROLLBAR_ID` ficam **órfãos com a
  nota ao lado** — o segundo permanece no livro-razão de propósito, porque *um id retirado ocupa uma
  linha e um id reusado ocupa duas barras ao mesmo tempo*;
- o ramo do `dispatch/scroll.rs` que despacha aquele id fica **inerte por construção** (ninguém o
  regista, logo nenhum `id ==` casa).

---

### §14.1 — As três metades da ordem

1. **O nó `motion.collide` sai do catálogo.** A colisão passa a ser sempre o botão `Collide` da
   forma — que é a decisão de 2026-09-10 (*«vou preferir colocar na shape»*) levada até ao fim.
   A §12 deixou o substrato: o leitor do colisor declarado já existe e já honra a declaração em
   todos os duplicadores.
2. **Toda visualização passa pelo Duplicador.** Hoje vários nós desenham por si.
3. **`Grid`, `rope` e os irmãos passam a ser POSIÇÕES e mais nada** — *«sem nenhuma capacidade de
   gerar pixels na tela»*.

### §14.2 — O que a §11 e a §12 já mediram, e que quem pegar nisto herda

- A composição `rope → motion.collide → rope.state` entrega **auto-colisão completa** a 32
  iterações, por `1,3 %`–`4,7 %` de um quadro (§11). Retirar o nó **tem de** manter essa
  capacidade por outro caminho, ou o grupo perde-a.
- Os **8 nós que geram nuvem NOVA** (corda, campo, corpo mole, bando, distribuições) não recebem
  forma de ninguém: as peças deles não são cópias de nada. *A ordem 3 é exactamente a pergunta de
  quem lhes dá o colisor* — e a §12 já a nomeia como pergunta de produto, não defeito.
- O `sim.collide` (peça contra MUNDO) é outro nó e outra pergunta; a ordem fala do irmão.

⚠️ **Nada disto está construído.** Quem abrir esta wave começa por medir o que a composição já
exprime (§5.0), como a §12 fez — foi isso que mostrou que faltava um LEITOR e não um motor.

---

### §16.1 — A lei, e onde ela mora

O [`ph2d_node_rig_bones::veste`](../../../crates/ph2d-node-rig-bones/src/lib.rs) escreve, depois de
derivar o quadro de cada osso, **`size = [len/2, len/2]`** por elemento. O `½` **não é escolhido**:
é o contrato da receita do `source.shape`, que constrói toda forma numa caixa de **largura
`2 × size`** e deixa a escala para a instância ⇒ `2 × len/2 = len`, o osso exacto.

⚠️ **A lei é dos OSSOS e não de todo carimbo.** O nó é a identidade num stream que não é um rig
(sem `parent` não há osso), logo ele **não inventa** um `size` — sem essa metade, a peça de um
`motion.grid` mudava de tamanho em silêncio. É o CONTROLO do gate da crate.

### §16.2 — A tabela, medida pela porta do produto

A sonda `diag_a_peca_contra_o_vao` (cena `=120`, os `TIQUES` de queda na corda) media a quebra
célula a célula; hoje as **sete** leem `1,00×`:

| knob do painel | vão | desenhado ANTES | razão ANTES | desenhado HOJE | razão HOJE |
|---|---|---|---|---|---|
| `Count = 10`   | `0,21201` | `0,10000` | **`0,47×`** | `0,21201` | `1,00×` |
| `Count = 20`   | `0,10252` | `0,10000` | `0,98×` | `0,10252` | `1,00×` |
| `Count = 30`   | `0,06944` | `0,10000` | `1,44×` | `0,06944` | `1,00×` |
| `Count = 40`   | `0,05393` | `0,10000` | **`1,85×`** | `0,05393` | `1,00×` |
| `Length = 0,2` | `0,20000` | `0,45000` | **`2,25×`** | `0,20000` | `1,00×` |
| `Length = 0,45`| `0,45000` | `0,45000` | `1,00×` | `0,45000` | `1,00×` |
| `Length = 0,9` | `0,90000` | `0,45000` | **`0,50×`** | `0,90000` | `1,00×` |

⭐ **A barra do gate desceu de `0,05` para `1e-5`, e a premissa do número antigo MORREU à vista no
diff:** ele tolerava o esticão do solver (*«às `TIQUES` de queda o pior segmento estica 2,52 %»*),
que era um erro enquanto o comprimento da peça fosse um número escrito à mão. O `len` sai das
**mesmas posições** de que o vão é medido ⇒ a peça já não se afasta do vão: **ela segue-o**, e a
barra passa a medir só uma divisão e uma multiplicação por dois.

### §16.5 — ⛔⛔⛔ *«Porque a corda afina no final?»* — o comprimento é do OSSO, a espessura é da CADEIA

Report do dono, 2026-09-20, uma hora depois do smoke da §16. Medido na `=120`, do 1.º segmento
ao último, em regime:

| `Count` | 1.º vão | último vão | último/1.º | pior par VIZINHO |
|---|---|---|---|---|
| `20` | `0,10252` | `0,10000` | `0,975×` | `0,998×` |
| `40` | `0,05393` | `0,04872` | `0,903×` | `0,996×` |
| `80` | `0,03334` | `0,02405` | **`0,721×`** | `0,994×` |

⭐ **O mecanismo:** numa corda pendurada o segmento de cima suporta o peso de todos os de baixo,
logo estica mais — e com a escala **uniforme** a ESPESSURA esticava com ele. Quantos mais
segmentos, maior o gradiente; e ele é **suave** (o pior par vizinho é `0,994×`), que é porque se
lê como *«a corda afina»* e não como um defeito pontual. ⚠️ E o passo 3 do smoke que eu lhe dei
mandava arrastar exactamente esse knob.

⇒ **o comprimento é do osso; a espessura é da CADEIA.** Um vão é facto do solver e muda por
quadro; *a espessura de uma corda não muda quando ela estica* — a mesma lei que o Painter desta
casa pagou seis vezes.

**Porque é o MÍNIMO e não a mediana**, medido sobre 280 tiques de corda a balançar:

| estatística | mín | máx | oscilação |
|---|---|---|---|
| mediana | `0,10066` | `0,10217` | `1,502 %` |
| média | `0,10063` | `0,10192` | `1,283 %` |
| **mínimo** | `0,10000` | `0,10000` | **`0,000 %`** |

⭐⭐ **E ele não ganha por pouco — ganha por MECANISMO:** num solver de distância um segmento
estica sob tensão e não comprime abaixo do repouso, logo o mínimo **É** o comprimento de repouso,
ao bit.

⭐⭐⭐ **E as duas ordens do dono não colidem no produto, medido:** das **cinco** cadeias de
`rig.bones` das cenas, **quatro** têm `len` exactamente uniforme (`max/min = 1,000×`) — ali o
mínimo é o `len` e a saída é **byte-idêntica** à que ele aprovou. A única não-uniforme é a corda.
⏳ **Fronteira declarada:** numa cadeia de ossos AUTORADOS com comprimentos diferentes as duas
ordens pedem coisas opostas e esta lei dá a todos a espessura do mais curto — **decisão do dono**,
e o caso não existe hoje para a forçar.

⛔⛔ **E a sonda que mediu isto não via a cura:** ela imprimia só `size.x` (o COMPRIMENTO, que
**deve** seguir o vão), logo com a lei curada a tabela dela saiu **idêntica** e a leitura ingénua
era *«a cura não fez nada»*. *Uma régua que lê um eixo não vê o outro.* Com a coluna da espessura:
`1,000×` nas **vinte** células da varredura (`Count` `10`–`80` × `1`–`120` tiques).

### §16.6 — ⭐ E a performance da corda, medida (*«avalie performance de rope»*)

Pela porta do QUADRO (`pump`, nunca `cook` — o memo é chaveado pelo tique), em `--release`,
`load 4,46`:

| `count` | peças | solver | pano inteiro | % de 16,67 ms | ns/peça |
|---|---|---|---|---|---|
| `20` | `19` | `0,020` | `0,024` | `0,1 %` | `1 265` |
| `320` | `319` | `0,102` | `0,116` | `0,7 %` | `364` |
| `1 280` | `1 279` | `0,355` | `0,396` | `2,4 %` | `310` |
| `5 120` | `5 119` | `1,371` | `1,526` | `9,2 %` | `298` |
| `20 480` | `20 479` | `5,446` | `6,442` | **`38,6 %`** | `315` |

⭐⭐ **Duas leituras, e as duas são o contrário do resto do módulo:** o custo é **LINEAR**
(`~300 ns/peça`, plano sobre `64×`) e **`85 %` dele é o SOLVER**, não o carimbo. *A corda é o único
pano desta cena em que o `motion.duplicator` não domina* — e a razão é estrutural: ali é **uma**
forma × `n` pontos, logo não há produto cartesiano a explodir. ⇒ **a corda não é um problema de
performance**, e o tecto dela (`20 480` segmentos em `38,6 %` de um quadro) está `1 000×` acima do
valor de fábrica.

### §16.7 — Provas

`veste` na crate do nó (com o controlo da nuvem) · `a_peca_veste_o_osso_em_todo_o_curso_dos_knobs`
na `=120` (as sete células + os dois eixos + o controlo do pano do CAMPO, que não passa por
`rig.bones` e lê `3,9×`) · `a_peca_desta_cena_veste_o_osso` na `=125` · **`a_corda_nao_afina_no_final`**, que é o único que
mede a lei da espessura onde ela se distingue da anterior (o `len` a variar).
**Mutação: 9 de 9 sangram**, com os TRÊS controlos verdes — e elas cobrem as três camadas: a LEI
(o meio do comprimento · o mínimo a virar máximo · a espessura a voltar ao osso · o comprimento a
congelar com ela), a FIAÇÃO (*a lei não existe* · *a lei existe e não CHEGA*, porque o valor de
fábrica do duplicador deita a escala do ponto fora) e a inércia.

⚠️ **E a fixtura da crate tem `len` VARIADO de propósito:** numa cadeia uniforme — que é o que
**quatro das cinco** do produto têm — as duas leis dão o mesmo par, e um gate escrito ali passaria
verde sobre qualquer uma. *Uma fixtura que não contém o fenómeno não distingue a lei da anterior.*
