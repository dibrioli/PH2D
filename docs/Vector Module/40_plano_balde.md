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

## §7 — ⏳ Nomeado e fora da v1

- **Vazamento**: se o clique cai na face externa (a região não fecha), o balde **recusa e diz
  porquê**. ⛔ Fechar vãos automaticamente é a lei do `ph2d-flip-fill` (bola presa + extensão de
  pontas) e pertence a uma wave própria — soldar já é o gesto que fecha o que o artista quis.
- **Ilhas**: uma forma solta dentro da face fica por cima; ela ainda não vira buraco (subpath).
- **Live Paint** (a face como estado vivo do grupo) — §1.
