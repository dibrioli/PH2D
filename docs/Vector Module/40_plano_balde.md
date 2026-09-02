# Plano 40 — **O BALDE** (a região que o clique aponta vira forma)

> Enio, 2026-08-31: *"Agora O Balde: preenche áreas por linhas fechadas ou linhas sobrepostas."*
> E, 2026-09-01, depois do Weld funcionar: *"Vamos à implementação do Balde?"*

## §1 — A pesquisa (o que a indústria faz, e o que ela ABANDONOU)

| app | modelo | o que sai do clique |
|---|---|---|
| **Illustrator — Live Paint** | um GRUPO especial; as faces são estado VIVO do grupo | a face fica pintada **dentro do grupo**; editar uma linha repinta |
| **CorelDRAW — Smart Fill** | um **objecto novo** por clique | uma forma fechada, colocada por cima/atrás; independente daí em diante |
| **Inkscape — Bucket** | rasteriza a tela e traça o balde | um caminho novo, **traçado de pixels** (grosseiro, e depende do zoom) |
| **Krita / Flip** | raster + fecho de vãos + bola presa | pixels |

⛔ **O modelo do Illustrator foi MEDIDO e recusado para a v1**: um Live Paint Group é um tipo de
objecto novo, com estado próprio (as faces), sincronização por edição e um modo de selecção
próprio (o *Live Paint Selection Tool*). Ele é a feature certa **depois** de haver uma que
preencha; construí-lo primeiro é o mesmo erro que a `line/Vector` já pagou no morph.

⇒ **Adoptamos o CorelDRAW: um clique = um OBJECTO novo.** Ele compõe com tudo o que a casa já tem
(estilo, pose, undo, hierarquia, booleana) e não pede tipo nenhum.

⛔ **E não é o do Inkscape.** Traçar pixels dá uma forma que depende do zoom e que não coincide com
as linhas que a geraram. Aqui a fronteira é feita dos **próprios arcos**.

## §2 — A lei

> **O balde preenche a FACE que o clique aponta, e a fronteira dela é uma sequência de ARCOS
> INTEIROS da rede.**

⭐⭐⭐ **É por isto que o Weld veio primeiro.** Soldar parte todo contorno nos cruzamentos, então
cada arco vai de nó a nó e **nenhum ponto interior de um arco é fronteira de face**. A face é um
ciclo de arcos, e reconstruí-la em bézier é **concatenar arcos** — sem aproximação, sem faceta, sem
depender do zoom.

⚠️ **E o balde não exige que o artista tenha soldado.** Ele faz o mesmo corte **numa cópia**, pela
mesma porta (`trim_tool::crossings_against` + `weld::split_at` + `weld::cluster_endpoints`). Soldar
continua a ser o verbo que torna a rede **autorada**; o balde só precisa dela para o instante do
clique.

## §3 — ⛔ Por que isto NÃO é o Shape Builder

O `ph2d-vec-boolean::arrangement` já responde *"que face é esta?"* — e o doc dele explica que
**deliberadamente não tem DCEL**, porque uma face tem definição conjuntista:

```text
região(M) = (∩ das formas em M) − (∪ das formas fora de M)
```

⛔ **Essa definição não existe para um traço ABERTO** — uma linha não tem dentro, então nenhuma
pertinência a descreve. É exactamente o caso do pedido (*"linhas sobrepostas"*).

⇒ O balde é **o DCEL que o arranjo evitou**, e existe pela razão que o próprio doc do arranjo
nomeia como fronteira. Para formas fechadas os dois sabem responder: o Shape Builder fica com o
**arrasto** sobre faces, o balde com o **clique**. ⚠️ *É uma divergência declarada, não um
descuido*: duas leis diferentes sobre o mesmo caso, e cada uma com o seu gesto.

## §4 — O algoritmo

1. **Os contornos visíveis**, cozidos e no MUNDO (a convenção do `apply_vec_boolean`).
2. **Cortar nos cruzamentos** → arcos, cada um com a geometria bézier e as duas pontas.
3. **Fundir as pontas coincidentes** (`weld::cluster_endpoints`, folga = 2× flecha) → os NÓS.
4. **Meias-arestas**: cada arco dá duas (ida e volta). Em cada nó, ordenadas pelo ângulo da
   tangente de SAÍDA.
5. **`next(h)`** = a meia-aresta imediatamente anterior ao gémeo de `h`, em sentido anti-horário
   à volta do nó de chegada — o passeio clássico que produz as faces com orientação consistente.
6. **A face do clique**: entre os ciclos de área positiva que contêm o ponto, o de **menor área**
   (é o que resolve o aninhamento). A face externa tem área negativa e é descartada por
   construção.
7. **A forma**: os arcos do ciclo concatenados, fechados, com o preenchimento corrente.

⚠️ **A polilinha do passeio é a MESMA que a detecção de cruzamentos usa** (`arc_cut`): duas
amostragens diferentes discordariam sobre a existência de um cruzamento, e a face desapareceria
num sítio e não noutro.

## §5 — O CUSTO, medido (e o que ele refutou)

| contornos | arcos | montar a rede | achar a face |
|---|---|---|---|
| 4 | 8 | `0,06 ms` | `0,01 ms` |
| 10 | 136 | `0,72 ms` | `0,05 ms` |
| 20 | 280 | **`3,80 ms`** | `0,08 ms` |
| 40 | 628 | **`26,3 ms`** | `0,18 ms` |
| 80 | 1293 | **`188 ms`** | `0,35 ms` |

⛔ **Montar a rede por QUADRO está refutado** — ela estoura o orçamento de `16,7 ms` aos ~20 traços.
⭐ **Achar a face por quadro é de graça.** ⇒ a rede é **guardada**, e a chave do cache é o
**CONTEÚDO** (uma soma sobre âncoras e alças), não a contagem de caminhos: mover uma forma não muda
quantas há.

⚠️ **Com o balde na mão o documento é quase estático** — não há gizmo neste modo —, então a
reconstrução acontece uma vez por preenchimento (e num undo). O pico de `188 ms` num desenho de 80
traços é um **soluço no primeiro hover**, não um congelamento por quadro; fica **medido e aceite**,
com a broadphase por grelha nomeada como a saída se alguém a pedir.

## §6 — O que a wave achou fora do balde

⛔⛔ **Um cruzamento na EMENDA de um anel era descartado** (defeito no `weld::split_at`, plano 39):
a fracção `0` de um contorno FECHADO é um ponto interior, e o filtro que serve a um contorno ABERTO
(onde `0` e `1` são as PONTAS) apagava-o. Um círculo cortado exactamente sobre a âncora de partida
saía com **um** arco em vez de dois. ⚠️ A `ellipse` começa em `(cx + r, cy)` — que é justamente onde
uma recta horizontal pelo centro a corta —, então o caso não é exótico: é o primeiro que se desenha.
Vale **também para o Soldar**, que tinha o mesmo buraco.

⚠️ **E a folga dos nós precisou de um segundo piso.** A régua da flecha, agora correcta (ela mede a
distância à CORDA, não ao ponto médio dela), é **zero numa recta** — e aí os dois lados de um
cruzamento, que calculam o mesmo ponto com resíduo de `~1e-15`, **não se juntam**: a rede fica
desligada e não há face nenhuma. O piso é uma **fracção da diagonal** da arte (`1e-5`), que é a lei
que o `ph2d-flip-fill` já usa para a mesma pergunta.

## §7 — ⭐⭐⭐ O REPORT DE 2026-09-01 (com fotos) — três defeitos, e o 2.º muda o modelo

> Enio: *"Ficou interessante mas muito limitado. 1) ao usar o balde nas áreas coloridas, ele para de
> funcionar nas áreas não coloridas. 2) se movo os nós da linha, o preenchimento não acompanha. A
> área deveria permanecer perfeitamente preenchida mesmo modificando o path. 3) o preenchimento está
> acima do stroke, mas deveria estar abaixo."*

### 1. ⛔⛔ Um preenchimento não é uma PAREDE

A forma depositada tem por fronteira **os mesmos arcos** que as linhas. De volta à rede, ela punha
lá arestas **coincidentes**, com direcção de saída idêntica — e o passeio de faces passava a
escolher entre duas meias-arestas indistinguíveis. As regiões vizinhas deixavam de fechar.

⇒ Quem tem `VecBucketFill` **não entra na rede**. *Uma parede é o que o artista desenhou; um
preenchimento é o que ele pediu.* ⚠️ A exclusão vive numa **porta com nome** (`fora_da_rede`), e não
num fecho no sítio da chamada: a 1.ª redacção tinha-a inline, e a mutação que apagava o termo do
preenchimento **sobreviveu** — o gate media o fecho que o **teste** construía.

### 2. ⭐⭐⭐ O preenchimento é VIVO, e a receita é o PONTO

⚠️ **Guardar a lista de ARCOS não resolveria**: um arco nasce de um corte em fracções, e mover um nó
**muda os cruzamentos**, logo muda a própria lista. *Qualquer receita feita de pedaços da rede é uma
receita sobre uma rede que já não existe.*

⇒ A receita é o que o artista de facto fez: **apontou ali**. `VecBucketFill { seed }` guarda o ponto,
e a área é a resposta de hoje — re-cozida sempre que a rede muda, **em qualquer ferramenta** (ele
arrasta um nó com a seta branca). ⚠️ Uma semente que deixou de cair em face nenhuma **congela** a
forma onde ela está, em vez de a fazer sumir — a escolha do conector e do morph.

⭐ **Isto é o Live Paint do Illustrator com outro substrato** — e sem o tipo de objecto novo que o §1
recusou: lá a face é estado vivo de um grupo especial; aqui é uma pergunta que se refaz.

### 3. ⛔ O `insert_path(0, …)` NÃO é o fundo

Quem manda no desenho é o **`RootOrder` da ENTIDADE**, e o `vec_entities::sync` dá a toda entidade
nova **o maior** — a frente. *O índice na cena e a ordem de desenho são duas listas, e a que o olho
vê é a segunda.* ⇒ a forma é mandada para o fundo (`ZOrder::ToBack`) na mesma passagem em que a
receita lhe é presa, logo depois do `sync` (no clique a entidade ainda não existe).

### ⚠️ E o INSTRUMENTO mentiu, no meio disto

O arnês de mutação restaurava o ficheiro com `mv`, e o mtime voltava **para trás**: o cargo ficava
com o **mutante compilado** e a fonte curada no disco, e as corridas seguintes mediam o mutante.
Sintoma: uma função com gate **verde na sua própria crate** devolvia o comportamento **antigo** a
quem dependia dela. ⇒ o arnês faz `touch` depois de restaurar.

### 4. ⛔ E a área re-cozida saía DESLOCADA — pelo próprio centro

> Enio, 2026-09-01 (com foto): *"o preenchimento está nascendo deslocado para fora do stroke."*

⚠️⚠️ **A rede fala MUNDO e o documento guarda LOCAL** — a regra-mãe do módulo, e o re-cozimento
esquecia-a. A forma **nasce certa** (uma entidade nova está na identidade); no quadro seguinte o
`settle_origins` muda a **origem** dela para o centro da própria caixa, e a partir daí escrever
mundo naquele `VecPath` desloca-o **pelo centro dele**.

⭐ **A foto confirma o mecanismo antes de uma linha de código:** cada área estava desviada por um
vector DIFERENTE, e cada vector era o centro da sua própria região — a de cima-esquerda para cima e
para a esquerda, a da direita para a direita, a de baixo para baixo. *Um desvio constante seria uma
câmara; um desvio por-forma é a pose de cada uma.*

⇒ `para_local` desce a área ao espaço do caminho antes de a escrever. ⛔ O `apply_bucket` **não**
estava errado: ali a entidade ainda nem existe — e é por isso que o defeito aparecia *"ao nascer"*,
que é o primeiro re-cozimento.

## §8 — ⏳ Nomeado e fora da v1

- **Vazamento**: se o clique cai na face externa (a região não fecha), o balde **recusa e diz
  porquê**. ⛔ Fechar vãos automaticamente é a lei do `ph2d-flip-fill` (bola presa + extensão de
  pontas) e pertence a uma wave própria — soldar já é o gesto que fecha o que o artista quis.
- **Ilhas**: uma forma solta dentro da face fica por cima; ela ainda não vira buraco (subpath).
- **Live Paint** (a face como estado vivo do grupo) — §1.
