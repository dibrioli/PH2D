# ARQUIVO — parte_1.md (história, 2273 linhas)

> ⚠️ **Isto NÃO é o estado atual de nada.** É a história recortada de
> [`parte_1.md`](parte_1.md) em 2026-10-11, **verbatim** — nenhuma
> linha foi editada, e a remontagem das duas metades bate sha256 com o original.
>
> Use para responder *"por que isto ficou assim?"* — **nunca** para decidir a próxima
> ação. O que vale hoje está no doc vivo e no [`CLAUDE.md §5`](../../../../CLAUDE.md).
>
> ⛔ O que estiver aqui marcado **«medido e REJEITADO»** continua rejeitado: uma
> recusa com medição atrás não volta à fila por ter mudado de arquivo.
>
> Recorte: linhas fora de `1-475` do original.
>
> ⚠️ **A única alteração ao corpo:** 0 alvo(s) de link relativo foram
> **reancorados** para apontarem ao MESMO arquivo de antes — o corpo desceu de pasta e
> todo `../x` passaria a resolver noutro sítio. Texto, números e estrutura são
> byte-idênticos; a partição foi provada por sha256 **antes** desta reancoragem.

---

### §9.4 — A FAIXA: o traço esticado sem peças de junta (2026-10-01)

**O que mudou** ([`eixo.rs`](../../../../crates/ph2d-shape-gpu/src/eixo.rs) · `peca_do_eixo` no
[`shape.wgsl`](../../../../crates/ph2d-shape-gpu/src/shape.wgsl)): até aqui cada vértice do eixo era uma
peça PRÓPRIA (a junta), com caixa, três pontos e quatro arestas. ⇒ **o vértice passa a ser do
troço que CHEGA a ele.** Os dois troços que lá se encontram acabam na MESMA bissectriz quando ela
serve, e a aresta partilhada cancela-se na soma; quando não serve, quem chega põe a junta. A
bissectriz serve:

- num ponto **LISO** se a esquadria fica a `FAIXA_FOLGA = 0,1 px` do arco verdadeiro;
- numa **QUINA** se a junta autorada É a esquadria dentro do limite — e então ela é exactamente a
  junta (`2 ≤ (1 + cos θ)·limite²`, o teste do Vello);
- nos dois casos, se a esquadria não recua mais de metade de um troço pelo lado de dentro (senão o
  quadrilátero deixa de ser convexo).

`EixoItem` ganha `d` (o vizinho de trás, `48 → 56` bytes) e os bits `FAIXA_INICIO`/`FAIXA_FIM`/
`QUINA_INICIO`/`QUINA_FIM` no `ponta` do troço; `ITEM_JUNTA` sai (o tipo `1` fica vago, os outros
números não se mexem). Os pontos deduplicam-se em **`f32`** — dois pontos distintos em `f64` que
caem no mesmo `f32` dariam um troço de comprimento zero, e a faixa partia-se lá.

**Medido no proxy de telemóvel** (Radeon integrada, `release`, dois rounds iguais):

| sonda (`sonda_relogio_das_estrelas_grandes`), ms | antes | faixa | Vello |
|---|---:|---:|---:|
| `324` pequenas, contorno esticado (o arranjo da `=127`) | `2,21` | **`1,80`** | `2,08` |
| `72` grandes, contorno esticado | `4,69` | **`3,26`** | `1,00` |
| `72` grandes, contorno conforme (não passa pelo eixo) | `1,67` | `1,72` | `0,90` |

| app, `=127` densa, iGPU, perfilador de placa | `render.formas` |
|---|---:|
| antes (só pontos lisos na faixa) | `30,4`–`30,9 ms` |
| **faixa nas quinas** | **`25,3`–`25,6 ms`** |
| a cena TODA pelo Vello (`PH2D_FORMAS_NA_PLACA=0`) | `19,7 ms` (o quadro de placa inteiro) |

⚠️ **A `=127` usa estrelas de quinas VIVAS** (`source.shape` `Star`), e a 1.ª versão da faixa só
cobria pontos lisos: a sonda (estrelas ARREDONDADAS) melhorou `19 %` e o app **não se mexeu**. *Uma
sonda cuja forma não é a da cena mede outro programa* — foi a diferença entre as duas que levou a
faixa às quinas.

⭐⭐ **E a faixa é MAIS EXACTA que o Vello, com prova.** A paridade da estrela esticada subiu de alfa
`40` para `73`, e a réplica da conta do shader em Python (`f32`, a mesma `contribuicao`) mostrou
onde: nos pixels de BORDA do lado de dentro de uma quina. Lá os dois rectângulos sobrepõem-se, e uma
rasterização por área que os SOMA conta a sobreposição duas vezes — é o que o Vello faz (o contorno
do kurbo passa pelo vértice) e o que o passe fazia com uma peça por troço. Medido por
supersamostragem `64²` no pior pixel: **área verdadeira `0,4756` · faixa `0,4755` · soma `0,772`**
(e no traço fino `0,693` · `0,694` · `1,0`). ⇒ gate novo
[`quina_exacta`](../../../../crates/ph2d-shape-gpu/tests/it/quina_exacta.rs): nas quinas côncavas das
duas fixturas a faixa fica a `0,002` da área verdadeira, com o CONTROLO de que a soma sobreconta
`0,296` e `0,169` (⚠️ uma `V` feita à mão não continha o fenómeno — a régua leu `0,03` e o controlo
reprovou-a). As barras dessas duas famílias passam às das curvas (`100`), com o defeito (o passe sem
eixo) a ler `255`.

**Duas famílias novas na paridade exercitam as cercas que nenhuma estrela grande toca:** estrelas de
`10`–`24 px` com traço grosso (a esquadria recua mais de metade do troço) e a estrela esticada com
`limite 2` (a ponta passa do limite e cede ao chanfro). Antes → faixa: `79`·`20`·`887 px` →
`71`·`28`·`1 497 px` e `51`·`18`·`17` → `61`·`32`·`93`.

**Mutação `9` de `11`** ([arnês](../../ferramentas/mutacao_a_faixa_do_traco_2026-10-01.py), pré-voo
`11/11`, corrida LIMPA verde com `21` testes): a faixa desligada · a quina lida como lisa · a junta
de recurso sem o estilo · sem a cerca do recuo · sem o limite da esquadria · a caixa da peça sem a
esquadria · sem os bits de quina · sem deduplicar em `f32` · o bloco sem a esquadria. **Sobrevivem,
NOMEADAS:** `S6` (a folga de `0,1 px` num liso) e `S8` (a folga de `0,1 px` na caixa da peça) — as
duas são cercas à escala de `0,1 px`, abaixo do ruído de `0,25 px` de aplanamento que as barras de
curva toleram.

⛔⛔ **RECUSA MEDIDA — HERDAR a bissectriz do troço anterior** (o vértice calculado uma vez por
pixel em vez de duas, com o bit `FAIXA_CONTINUA` e o ponto/bissectriz passados pelo laço): saída
idêntica ao bit, e **nenhum ganho** — sonda `1,84 → 1,81` (densa) e `3,28 → 3,36 ms` (grande), app
`25,3–25,6 → 25,2–25,9 ms`. ⇒ *a conta geométrica por pixel NÃO é o custo*; revertido.

⏳ **ABERTO, com o número: na `=127` densa o proxy de telemóvel continua atrás do Vello** — placa
inteira `30,3` contra `19,7 ms` (era `35,2`), com a CPU a cair `8,2 → 6,2 ms`. Na RTX a placa ganha
em todas as células. O que sobra é o custo POR PEÇA lida por pixel (caixas e arestas de ~`12`
itens de `56` bytes por cópia), e a cura de fundo continua a do Vello: o contorno de cada cópia
calculado UMA vez (uma passagem de cálculo antes do desenho) em vez de em cada pixel.

### §9.5 — O CONTORNO CALCULADO uma vez por cópia (2026-10-01)

**O que mudou** ([`contorno.wgsl`](../../../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
[`contorno.rs`](../../../../crates/ph2d-shape-gpu/src/contorno.rs)): antes do desenho, três passes de
cálculo — `cs_conta` (quantas arestas cada cópia escreve), `cs_soma` (o prefixo, num grupo só e
determinístico) e `cs_escreve` — percorrem as peças do eixo de cada cópia e escrevem as ARESTAS do
contorno já no ECRÃ, em blocos de `8` com a caixa de cada bloco. O fragmento (`traco_do_contorno`)
só soma arestas prontas. A geometria das peças (bissectrizes, juntas, leques, pontas) deixa de ser
refeita em cada pixel.

- ⭐ **As arestas que a FAIXA cancela não chegam a existir**: as duas metades de uma aresta
  partilhada decidem com os MESMOS argumentos e saltam juntas.
- ⭐ **DUAS correntes**: o lado que anda com o eixo escreve-se para a frente a partir do início da
  cópia, o que anda contra ele para trás a partir do fim — lidas por ordem de memória, as duas são
  correntes. Um bloco cujas oito arestas se tocam ponta a ponta (conferido AO BIT) e que fica todo à
  esquerda do pixel soma `clamp(y₀) − clamp(y₈)`, guardados ao lado da caixa (`(y₀, y₈, encadeado)`
  — **zero ligações novas**, o bloco passa a dois `vec4`).
- ⚠️ **A capacidade é MEDIDA, nunca adivinhada:** o total sai do prefixo, é copiado e lido DOIS
  quadros depois, e a capacidade cresce para ele (tecto = `max_storage_buffer_binding_size`). Até lá,
  uma cópia que não cabe — ou cuja escrita não bate na contagem — é desenhada pelo caminho de
  sempre, **por cópia**: nunca um contorno truncado.
- `PH2D_CONTORNO_CALCULADO=0` bissecta (o caminho pixel a pixel do eixo, §9.4).

**Gate** [`contorno_calculado`](../../../../crates/ph2d-shape-gpu/tests/it/contorno_calculado.rs) — as
sete famílias do traço esticado pelos DOIS caminhos, `4` quadros do mesmo passe: alfa máx. **`1`**
e **zero** pixels a desviar mais de `1` em todos os quadros; no 1.º quadro **três** famílias
transbordam a capacidade de fábrica (círculos `12/40`, zigue-zague redondo `38/40`, estrelas pequenas
de traço grosso `101/120`) e a MISTURA dos dois caminhos desenha a mesma imagem; a partir do 3.º
todas as cópias ganham contorno; o CONTROLO desligado lê `0`. ⚠️ *A imagem sozinha não prova que o
caminho novo correu* — o recurso desenha o mesmo —, por isso o gate lê de volta quantas cópias o
ganharam.

**Medido no proxy de telemóvel** (Radeon integrada, `release`, dois rounds iguais, sonda
`sonda_relogio_das_estrelas_grandes` — ⚠️ a `=2` usa a estrela da CENA, §9.4):

| sonda, ms de passe | eixo (§9.4) | contorno | contorno + correntes | Vello |
|---|---:|---:|---:|---:|
| `1225` estrelas da `=127` (vivas), contorno esticado | `3,56` | `2,42` | **`2,49`** | `2,69` |
| `324` pequenas (arredondadas) | `1,92` | — | `2,05` | `2,16` |
| `72` grandes (arredondadas) | `3,31` | `3,88` | **`3,57`** | `1,00` |
| `72` grandes, conforme (CONTROLO, sem eixo) | `1,74` | — | `1,73` | `0,90` |

| app, `=127` densa, perfilador de placa, quadro de placa INTEIRO (mínimo) | iGPU | RTX |
|---|---:|---:|
| eixo pixel a pixel (`PH2D_CONTORNO_CALCULADO=0`) | `27,8 ms` | `1,57 ms` |
| **contorno calculado** | **`22,3 ms`** (formas `15,6` + cálculo `3,1`) | **`1,16 ms`** |
| a cena TODA pelo Vello (`PH2D_FORMAS_NA_PLACA=0`) | `19,4 ms` | `1,99 ms` |

⚠️ *O `render.vello` do perfilador CONTÉM o `render.formas` e o `render.contorno`* — é o quadro de
placa inteiro, e é ele que se compara com a rota do Vello. Na RTX o quadro oscila `1,2`–`7,4 ms` pelo
relógio da placa a esta carga, logo compara-se o MÍNIMO.

**Mutação `5` de `5` + `1` NOMEADA** ([arnês](../../ferramentas/mutacao_o_contorno_calculado_2026-10-01.py),
pré-voo `6/6`, corrida LIMPA verde): o contorno nunca escrito · sem a aresta de ponta da frente ·
todo bloco lido como encadeado · o telescópio ao contrário · a capacidade sem crescer — as cinco
sangram no gate. **Sobrevive, NOMEADA:** `M6` (o lado de baixo na corrente da frente) — quebra só as
correntes, e as correntes são RELÓGIO: a soma é a mesma em qualquer ordem, logo nenhuma régua de
imagem a vê.

**A ablação que mandou para as correntes** (`abla2.sh`, iGPU): saltar os blocos à esquerda (só
relógio, imagem errada) levava as `72` grandes de `3,88` a `3,17 ms` — o teto do que as correntes
podiam comprar; elas compraram `3,57`.

⛔⛔ **RECUSA MEDIDA — COSTURAR as correntes por um passo guloso** (no `cs_escreve`, pôr a seguir a
cada aresta a que começa onde ela acaba, numa janela de `32`): **pior em todas as células** —
`2,49 → 3,11` (densa), `2,05 → 2,62`, `3,57 → 4,00 ms` (grandes). O laço serial por cópia sobre a
memória custa mais do que os blocos que encadeia. Revertido.

⛔ **RECUSA MEDIDA — os testes de lado sem divisão por aresta** no `contribuicao` (`max(px) ≤ 0` ⇒
faixa, `min(px) ≥ 1` ⇒ zero): sem efeito no relógio (a divergência da onda paga o caminho caro na
mesma); mantidos por serem exactos e baratos.

⏳ **ABERTO, com o número: no proxy de telemóvel a `=127` densa continua atrás do Vello**
(`22,3` contra `19,4 ms` de placa; era `27,8`), e nas estrelas GRANDES o contorno ainda perde para o
eixo (`3,57` contra `3,31`). O que sobra está no desenho (`15,6 ms`), não no cálculo (`3,1`): os blocos
à direita/dentro pagam a conta do Vello aresta a aresta, e a cura seguinte é a do Vello — ladrilhos
(as arestas de cada cópia ordenadas por faixa de ecrã) em vez de blocos por cópia.

### §9.6 — AS CÉLULAS E O FUNDO: toda a cópia no ecrã (2026-10-01)

**O que mudou** ([`contorno.wgsl`](../../../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
`cobertura_de_ecra` no [`shape.wgsl`](../../../../crates/ph2d-shape-gpu/src/shape.wgsl) ·
[`contorno.rs`](../../../../crates/ph2d-shape-gpu/src/contorno.rs)):

- ⭐ **As três famílias no ecrã.** O cálculo de §9.5 escrevia só o contorno do eixo; o preenchimento e
  as marcas continuavam a ser transformados EM CADA PIXEL (a ablação na sonda densa: `~1/3` do
  passe). Agora `cs_escreve` transforma-os uma vez por cópia e escreve-os à frente do contorno, cada
  família completada até um múltiplo de `8` (os blocos não atravessam um trecho). Sob afim conforme
  as «marcas» são o traço INTEIRO (as marcas e o contorno expandido), e a soma do pixel é a mesma
  `min(min(|marcas|, 1) + min(|contorno|, 1), 1)` do caminho de sempre.
- ⭐⭐ **As CÉLULAS e o FUNDO** (o `backdrop` do Vello): cada fileira de pixels da cópia é partida em
  células de `LARGURA_DA_CELULA = 32 px`; cada célula guarda três fundos (a soma, NESSA fileira, dos
  blocos que acabam todos à esquerda dela — não depende do `x` do pixel) e uma máscara com um bit por
  bloco que lhe toca. O pixel soma o fundo e só esses blocos. A fileira de um bloco é exacta em
  `f32` (`floor(lo.y)` a `ceil(hi.y) − 1`); o fundo soma-se na primeira célula toda à direita do
  bloco e depois faz-se o prefixo ao longo da linha.
- ⭐ **A contagem sem a geometria.** `cs_conta` corria o `percorre` inteiro só para contar; agora usa
  o pior caso de cada ramo de `emite_peca` (o leque de `k` passos emite no máximo `3k`), e
  `cs_escreve` desce a corrente de trás para o fim do que de facto usou. O cálculo da `=127` densa no
  proxy: `4,08 → 2,93 ms`.
- ⭐ **A ROTA:** uma cópia esticada com traço vai sempre; uma CONFORME só a partir de
  `AREA_MINIMA_CONFORME = 1024 px²` de caixa estimada (abaixo disso o caminho de sempre já a desenha
  bem, e o cálculo é pago por cópia). `ShapePass::area_minima_conforme` deixa os gates de pixel medirem
  o caminho novo em todas as cópias; `PH2D_AREA_MINIMA_CONFORME` é o instrumento da varredura.

**Medido no proxy de telemóvel** (Radeon integrada, `release`; a placa leu estável sob carga — o
caminho de sempre deu os números de §9.5 ao dígito na mesma corrida):

| sonda, ms de passe | §9.5 | arestas no ecrã | + células e fundo | + contagem barata | Vello |
|---|---:|---:|---:|---:|---:|
| `1225` estrelas da `=127` | `2,49` | `2,33`–`2,45` | `2,57` | **`2,22`** | `2,6` |
| `324` pequenas | `2,05` | `1,77`–`1,82` | `1,82` | **`1,67`** | `2,08` |
| `72` grandes esticadas | `3,57` | `2,69` | `2,38` | **`2,32`** | `0,98` |
| `72` grandes conformes | `1,73` | `1,39` | `1,38` | **`1,35`** | `0,90` |

| app, `=127` densa, quadro de placa (mínimo) | iGPU | RTX |
|---|---:|---:|
| §9.5 (contorno calculado) | `22,3 ms` | `1,16 ms` |
| arestas no ecrã + células (formas `11,89` + cálculo `4,08`) | `19,87 ms` | `1,11 ms` |
| **+ contagem barata** (formas `11,84` + cálculo `2,93`) | **`18,76 ms`** | — |
| a cena toda pelo Vello | `19,42 ms` | `2,03 ms` |

**A varredura da rota** (cena `17`, escada de `32 768` estrelas pequenas conformes, iGPU, quadro de
placa MEDIANO; a cena anima, logo o mínimo não serve): caminho de sempre `9,70` · área mínima `0`
(todas pelo cálculo) **`13,53`** · `256`, `1024`, `4096`, `16384` → `9,58`–`9,59`. E as `72` grandes
conformes: `0`/`1024` → `1,36 ms`, `4096` → `1,74` (perde o ganho). ⇒ a janela é `[256, 4096)`, e o
`1024` está dentro dela.

**A largura da célula é medida** (sonda, iGPU, ms): grandes esticadas `8 → 2,81` · `16 → 2,50` ·
**`32 → 2,33`** · `64 → 2,40`; grandes conformes `2,13 · 1,58 ·` **`1,36`** `· 1,45`; a `=127`
`32` e `64` empatam (`2,22`).

**Gates:** [`contorno_calculado`](../../../../crates/ph2d-shape-gpu/tests/it/contorno_calculado.rs) ganha
SETE famílias — estrelas pequenas só preenchidas · anéis even-odd · círculos com traço conformes ·
círculos GRANDES com traço (conformes e esticados — as máscaras passam de uma palavra, `> 32` blocos)
· estrela com traço e MARCAS (conforme e esticada) — todas a alfa máx. `≤ 1` e zero pixels a
desviar mais de `1`, catorze famílias a transbordar no 1.º quadro e todas as cópias no ecrã no
último. A paridade com o Vello passa a medir o REGIME DO PRODUTO (`3` quadros, exige todas as cópias
no ecrã). E `so_as_copias_conformes_grandes_pagam_o_calculo`: pequenas ficam, grandes vão, esticadas
com traço vão sempre, com o CONTROLO a área `0`.

**Mutação `10` de `10`** ([arnês](../../ferramentas/mutacao_as_celulas_do_ecra_2026-10-01.py), pré-voo
`10/10`, corrida LIMPA verde): o fundo sem o prefixo · a máscara só com a 1.ª palavra (sangra nos
círculos GRANDES — a família que a exerce) · o fundo na célula que o bloco ainda toca · o pixel sem o
fundo · todos os blocos no preenchimento · a corrente de trás sem descer · o limite da contagem sem a
junta (sangra pela capacidade que não cresce) · a rota ao contrário · a 1.ª linha de um bloco
arredondada para cima · a célula do pixel uma à direita.

⛔ **RECUSA MEDIDA — PARTIR as arestas longas em pedaços de `32 px`** (para cada bloco ficar local,
como o Vello recorta por ladrilho): sem efeito — grandes conformes `1,35 → 1,39 ms`. A estrela da sonda
tem cantos arredondados e as arestas JÁ são curtas; o custo dos pixels de borda não é aresta longa.
Revertido.

⏳ **ABERTO, com o número: as estrelas GRANDES no proxy de telemóvel** — `2,32` contra `0,98` ms
(esticadas) e `1,35` contra `0,90` (conformes). A eliminação por partes (esticadas): o cálculo
(`~0,3` contagem + `~0,36` escrita + `~0,42` células) corre num FIO por cópia, e com `72` cópias a
placa fica quase parada; os pixels de BORDA (`~1,2`) lêem os blocos inteiros da célula. A cura
seguinte tem endereço: as células e as máscaras construídas EM PARALELO por bloco (não por cópia), e
os pixels de uma célula a partilhar os blocos (memória do grupo), que é o que o rasterizador fino do
Vello faz.


### §9.7 — AS CÉLULAS EM PARALELO: um fio por FILEIRA do ecrã (2026-10-01)

A cura nomeada no fim do §9.6, metade dela. O `cs_celulas` deixa de correr num fio por CÓPIA e passa a
correr num fio por **fileira de ecrã** de cada cópia; a contagem (`contagem`) ganhou um terceiro terço
`(n+1)` com as fileiras por cópia, o `cs_soma` faz o terceiro prefixo e escreve os argumentos do despacho
**indirecto** (`despacho_rw`, binding 7). ⚠️ O `wgpu` recusa um buffer que é ao mesmo tempo armazenamento
de escrita e argumento indirecto no MESMO despacho ⇒ o `cs_celulas` tem um grupo de ligação PRÓPRIO
(`escrita_celulas`, sem o binding 7). Um fio acha a sua cópia por busca binária no prefixo, zera a faixa,
escreve as máscaras e o fundo de cada bloco na fileira dele, e faz o prefixo do fundo ao longo da fileira.

Medido no proxy de telemóvel (iGPU, `radeon_icd`, máquina calma, `sonda_relogio_das_estrelas_grandes`, ms):

| cena | antes (§9.6) | fileiras em paralelo | Vello |
|---|---|---|---|
| estrelas grandes esticadas | `2,32` | **`2,02`** | `0,98` |
| estrelas grandes conformes | `1,35` | **`1,17`** | `0,90` |
| densas | `2,22` | **`2,11`** | — |

GPU `ph2d-shape-gpu --ignored` **5/5**. ⚠️ **Por ablação (dobrar o desenho), quem manda agora é o DESENHO**
(`~1,38 ms` nas esticadas), e dentro dele o laço de blocos por pixel (`~1,2 ms`) — é a outra metade da cura.

⛔ **Recusas MEDIDAS (não reconstruir):**
- **blocos de 4 arestas** em vez de 8 — resultado misto, e as esticadas PIORAM.
- **células mais altas** (`ALTURA_DA_CELULA`): `H=1 2,02` · `2 2,17` · `4 2,30` · `8 2,69` — piora em
  monotonia; o registo cresce com a altura e cada pixel lê-o inteiro.
- ⏳ **pré-calcular a cobertura 8 px por fio no compute** (para o desenho ler 2 palavras em vez do laço de
  blocos): o protótipo foi construído e reprovou **4 de 5** gates de paridade antes de ser diagnosticado ⇒
  REVERTIDO, não refutado. ➜ **Substituído pelo §9.8** (as listas das células): o protótipo não estava
  guardado, e só dividia a leitura — a lista corta a conta.

### §9.8 — AS LISTAS DAS CÉLULAS: o que o Vello guarda por ladrilho, montado por ARESTA (2026-10-02)

**O porquê.** Depois do §9.7 quem mandava nas estrelas grandes era o DESENHO, e dentro dele o laço por
pixel: a máscara da célula dizia que BLOCOS tocavam a fileira, e cada pixel de borda percorria os oito
segmentos de cada um — quase sempre um só cruzava a fileira. ⇒ cada célula passa a guardar a **LISTA
das arestas que cruzam a fileira dentro dela** (o `backdrop` continua: as que ficam todas à esquerda
somam-se no fundo). É a lista de segmentos por ladrilho do rasterizador fino do Vello, com ladrilhos de
`32 × 1 px`.

**O protótipo do §9.7 («cobertura 8 px por fio no compute») NÃO foi reconstruído:** não estava guardado
em lado nenhum, e só dividia a LEITURA por oito pixels — a conta por pixel×segmento ficava igual. A
lista corta as duas.

**O que mudou** ([`contorno.wgsl`](../../../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
`cobertura_de_ecra` no [`shape.wgsl`](../../../../crates/ph2d-shape-gpu/src/shape.wgsl) ·
[`contorno.rs`](../../../../crates/ph2d-shape-gpu/src/contorno.rs)):

- O registo de célula é fixo: `REGISTO = 7` palavras — os três fundos e `(inicio, fim_f, fim_m, fim)`
  da lista de cada família. As máscaras de blocos, as caixas dos blocos e as DUAS correntes da escrita
  (que só existiam para as caixas serem finas) saíram.
- A montagem é por **ARESTA**, em quatro despachos indirectos: `cs_zera` (fio por fileira) ·
  `cs_conta_listas` (fio por aresta: fundo e contagem por atómicos) · `cs_lugar_das_listas` (fio por
  fileira: reserva as listas dela de uma vez, prefixo do fundo, cursores) · `cs_escreve_listas` (fio por
  aresta). Os argumentos saem do `cs_soma` (`despacha`: `[0,3)` por linha, `[3,6)` por aresta).
- ⭐ **Determinístico por construção:** o fundo soma-se em PONTO FIXO (`ESCALA_FIXA = 2¹⁶`) e o
  fragmento soma a lista também em ponto fixo — a ORDEM em que os atómicos arrumaram uma lista não muda
  um bit. Cada parcela erra `≤ 7,6e-6`.
- O intervalo de uma aresta numa fileira é o dos DOIS PONTOS, sem recortar (exacto: a `contribuicao`
  é contínua no `x`; a lista só fica mais longa numa aresta que atravessa fileiras) — recortar custava
  uma divisão por aresta e só valia `1,52 → 1,40 ms`.
- ⚠️ **Uma fileira sem lugar fica `SEM_LISTA`** e o pixel refaz-se pelo caminho de sempre. As listas só
  se contam DEPOIS de as arestas existirem ⇒ a capacidade delas chega **dois quadros depois** da das
  arestas (5.º quadro numa cena nova); até lá a imagem é a mesma (gate). `ShapePass::limita_as_listas`
  e `listas_do_ultimo_quadro` são as portas dos gates.
- O cálculo tem agora TRÊS relógios no perfilador: `render.contorno.conta`, `.escreve`, `.celulas`.

**Medido na sonda** (`sonda_relogio_das_estrelas_grandes`, que ganhou `PH2D_FLUID_PROFILE=1` — `250`
quadros e o relógio da placa por passe; [`mede_sonda_das_estrelas.sh`](../../ferramentas/mede_sonda_das_estrelas.sh)
corre os três arranjos só com `load < 4`). Proxy de telemóvel (iGPU), ms:

| decomposição (iGPU, perfilador) | desenho | cálculo | total |
|---|---:|---:|---:|
| esticadas, §9.7 | `1,36` | `0,57` | `1,93` |
| esticadas, listas montadas por FILEIRA (1.ª tentativa) | `0,81` | `1,52` | `2,33` |
| esticadas, listas montadas por ARESTA | **`0,82`** | **`0,69`** | **`1,51`** |
| conformes, §9.7 → por aresta | `0,875 → 0,47` | `0,22 → 0,29` | `1,10 → 0,76` |

⛔ **RECUSA MEDIDA — montar as listas com um fio por FILEIRA** (o regime do §9.7): `~2 900` fios na sonda,
cada um a percorrer centenas de arestas em série, DUAS vezes (contar e escrever) — a placa integrada
ficava à espera da memória e o cálculo subia `0,57 → 1,52 ms`, comendo todo o ganho do desenho. Por
aresta são `~70 000` fios de uma ou duas fileiras.

**A largura da célula foi re-medida para as listas** (desenho + células, ms; esticadas · conformes ·
densas): `16 → 1,34 · 0,79 · 1,61` · **`32 → 1,25 · 0,68 · 1,41`** · `64 → 1,60 · 0,85 · 1,60`. Fica `32`.

**A tabela calma** (a mesma sonda, o binário de ANTES e o de DEPOIS, `load < 4` em cada corrida — `2`
corridas por célula, a média; placa · Vello, ms):

| arranjo | iGPU antes | iGPU depois | RTX antes | RTX depois |
|---|---:|---:|---:|---:|
| `72` grandes esticadas | `2,04` · `1,01` | **`1,74`** · `1,05` | `0,42` · `0,37` | **`0,36`** · `0,40` |
| `72` grandes conformes | `1,18` · `0,92` | **`0,94`** · `0,92` | `0,17` · `0,37` | `0,18` · `0,37` |
| `1225` densas da `=127` | `2,13` · `2,69` | **`1,71`** · `2,81` | `0,19` · `0,52` | `0,21` · `0,52` |

⇒ no proxy de telemóvel as três descem (`−15 %`, `−20 %`, `−20 %`); as conformes grandes EMPATAM com o
Vello e as densas ficam `1,6×` à frente; ⏳ as grandes ESTICADAS ainda perdem (`1,74` contra `1,05`). Na
RTX as esticadas passam à FRENTE do Vello; ⚠️ as densas sobem `0,19 → 0,21 ms` (quatro despachos a
mais, numa placa onde o cálculo era quase nada) — continuam `2,5×` à frente.

**A W5 repetida, no APP, com a máquina calma** ([`mede_formas_na_placa.sh`](../../ferramentas/mede_formas_na_placa.sh),
a MESMA build `release` com e sem `PH2D_FORMAS_NA_PLACA`, cada célula com `load < 4` antes e depois;
as três últimas janelas `[frame]` de `120` quadros). Fecha o item «W0/W5 como tabela única» do handoff de
01/10:

| placa | cena | objectos | quadro sem → com | CPU (encode) sem → com | Motion sem → com |
|---|---|---:|---:|---:|---:|
| RTX | escada, estrela | 4 096 | 16,69 → 16,65 | 4,31 → **3,34** | 1,31 → **0,74** |
| RTX | escada, estrela | 16 384 | 16,65 → 16,66 | 6,79 → **3,44** | 2,91 → **0,78** |
| RTX | escada, estrela | 32 768 | 16,66 → 16,67 | 9,79 → **3,68** | 4,27 → **0,88** |
| RTX | `=127` densa | 16 384 | 16,71 → 16,70 | 6,96 → **3,07** | 1,97 → **0,66** |
| iGPU | escada, estrela | 4 096 | 16,70 → 16,70 | 4,33 → **4,26** | 1,41 → **0,80** |
| iGPU | escada, estrela | 16 384 | 16,68 → 16,67 | 6,82 → **3,29** | 2,78 → **0,68** |
| iGPU | escada, estrela | 32 768 | **18,04 → 16,71** | 11,14 → **3,00** | 6,12 → **0,62** |
| iGPU | `=127` densa | 16 384 | **20,68 → 17,59** | 7,98 → **4,29** | 2,34 → **0,75** |

⇒ ⭐ **o achado do §9.3 está curado no app:** no proxy de telemóvel a `=127` densa ia de `20,7` a
**`35,2 ms`** pela placa; hoje vai a **`17,6 ms` (57 fps contra 48 sem ela)**. Todas as outras células
batem no tecto de `60 Hz` nas duas rotas, com a CPU a metade ou menos. (O `acquire` da iGPU continua
`~12–13 ms` com e sem placa: é a apresentação a `60 Hz`, não trabalho.)

**Gates:** `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` (tecto em METADE do pedido em todos os
quadros; cópias TODAS pelas células, fileiras `SEM_LISTA`, imagem do eixo à barra; controlo: o tecto
morde) · `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior` (duas etapas no MESMO passe) · e o
regime do produto passa a exigir as listas a caber (`QUADROS_DO_PRODUTO = 5`, `QUADROS = 6`). GPU RTX:
`ph2d-shape-gpu` **7/7** · `ph2d-app-motion` placa **4/4** + ponte **1/1** · `ph2d-gpu-cook` formas **2/2**.

**Mutação `14` de `14`** ([arnês](../../ferramentas/mutacao_as_listas_das_celulas_2026-10-02.py), pré-voo
`14/14`, corrida LIMPA verde): fundo na `ka` · lista uma célula a mais à esquerda / a menos à direita ·
o fragmento ignora `SEM_LISTA` · sem o prefixo do fundo · cursor das marcas no início · a escrita não
salta `SEM_LISTA` · escala do ponto fixo dobrada · `min/max` trocados · `floor` na última fileira ·
`cs_zera` mudo · sem a verificação da capacidade · marcas e contorno trocados · e a **M10** (sem a guarda
das arestas reservadas e NÃO escritas), que SOBREVIVEU aos seis gates de então — num passe novo a
reserva é zero e não soma nada. ⭐ *Uma régua de quadro único não vê o que um buffer traz do quadro
anterior*: o gate da cena que muda foi escrito por ela, e é o único que a mata.

### §9.9 — O TRACEJADO NO ECRÃ: o traço tracejado sob escala não uniforme vai à placa (2026-10-02)

Commits `3abf99b77` (o tracejado) e `4aa06e738` (a variante da `=127`); antes, `7a58e7aaf` (gate das
LETRAS, abaixo).

**O porquê.** Até aqui o tracejado sob afim não conforme ficava no Vello (cerca `FLAG_SO_CONFORME` →
`RECUSA_FORMA_TRACEJADA` no dispositivo, `so_conforme` na rota da CPU); a cena `=76` (as bolas com
contorno tracejado) ia INTEIRA à CPU só por ter tracejado, mesmo conforme.

**A lei da casa (fonte).** `stroke_uniform::pen_for` traceja a geometria JÁ transformada com caneta
`w·√|det|` e padrão `× √|det|`; o Vello (`vello::Scene::stroke`) corta pelo `kurbo::dash` (CPU) e o
traçador da placa dele desenha as pontas. Semântica do `kurbo::dash` 0.13.1 portada: a fase recomeça em
cada sub-caminho; cada traço tem `start_cap` no início e `end_cap` no fim; num FECHADO o último traço
EMENDA no primeiro (junta, sem pontas) só quando atravessa o início; um traço que acaba exactamente num
vértice não liga.

**O que mudou.**

| onde | o quê |
|---|---|
| `EixoItem` `56 → 72 B` | `traco`, `vao` (locais, `[traço, vão]` com fase 0 — `tracejado_do_eixo`; outro padrão mantém `FLAG_SO_CONFORME`) · `flecha` (a flecha de cada corda: o ponto da curva no meio dela menos o meio da corda, por `PathSeg::nearest`) |
| bits no `ponta` do troço | `SUB_INICIO = 16` (o `_pad` do primeiro troço = quantos troços) · `SUB_FECHADO = 32` · pontas do início em `TAMPA_INICIO_BIT = 6` e do fim em `TAMPA_FIM_BIT = 8` |
| cabeçalho de bloco | `BLOCO_TRACEJADO` (o pixel a pixel não o salta pela caixa) |
| itens de ponta | nenhum num sub-caminho tracejado |
| `shape.wgsl` (partilhado pelas duas passagens) | `sub_tracejado` (pré-passo de um fechado: comprimento total e se emenda) · `troco_tracejado` · `pedaco` (o traço `n` ocupa `[n·período, n·período + traço]`, é do troço onde COMEÇA, `[s0, fim)`) · `tracejado_px` (pixel a pixel) |
| `contorno.wgsl` (o contorno calculado) | `emite_tracejado` / `emite_pedaco` |

O arco de cada troço no ecrã é `corda + 8h²/3c` (`arco`, `h` = a flecha levada pelo afim); as decisões de
faixa ficam nas CORDAS (os dois lados de um vértice recalculam-nas com os mesmos argumentos) e o recuo da
bissectriz é limitado pela metade do menor dos dois PEDAÇOS que se tocam (`bissectriz_ate`).
`TRACOS_POR_TROCO_MAX = 65536` é o tecto do vigia do dispositivo (um laço sem fim perde a placa), não
escolha de desenho.

**Na Motion.** A cerca saiu das duas rotas (`RECUSA_FORMA_TRACEJADA` e o `so_conforme` apagados); uma
geometria com `FLAG_SO_CONFORME` (que o `kurbo_stroke` da casa nunca produz) passa a `Entrada::Recusada`
→ `RECUSA_FORMA_DO_VELLO`. **Censo de rota:** a `=76` vai à PLACA, **`19` das `23`** cenas com forma; as
`4` que ficam: `fx.glow` (`=70`) · passagem (`=120`) · colisor lido (`=114`, `=115`).

**Medido** (RTX, meio-float; gate `ph2d-shape-gpu --test it tracejado`, contra o Vello; alfa · cor):

| família | alfa | cor |
|---|---:|---:|
| estrela esticada | `71` | `47` |
| traços longos (faixa e junta dentro do traço, emenda) | `84` | `45` |
| círculo, pontas redondas (era `70` · `89` antes da flecha) | `66` | `64` |
| zigue-zague, pontas diferentes (redonda/quadrada) | `50` | `47` |
| zigue-zague, chanfro e pontas quadradas | `4` | `3` |
| controlo conforme (o caminho pré-expandido, intocado) | `75` | `80` |

Contorno calculado contra pixel a pixel: alfa `≤ 1`, todas as cópias com contorno. **Rota do PRODUTO**
(`motion_shape_placa::gpu_tests::tracejado`, `StrokeSpec` com o tracejado AJUSTADO, a câmara): `87` · `69`,
`2 070` px `> 16` sobre `190 140`; sem a flecha lia `134`.

**A barra.** O vale é no ALFA — limpo `≤ 84`, mutações `126`–`255` ⇒ `100`/`100`. ⚠️ A fracção de pixels
com alfa `> 1` NÃO separa no tracejado (o controlo conforme lê `5,7 %` — cada traço com pontas redondas é
borda curva — e a emenda que falta `2,4 %`), por isso o gate do tracejado não a usa.

**Mutação `17` de `17`** ([arnês](../../ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py), sobre os `9`
gates GPU de `ph2d-shape-gpu` e os `6` de paridade do produto): padrão sem a caneta · nunca emenda ·
emenda sempre · nunca liga atrás · nunca liga à frente · faixa sem o recuo dos pedaços · fase por troço ·
pontas de início e fim trocadas · sem a ponta do início · arco no LOCAL × caneta · arco = corda (só o
gate do PRODUTO a mata) · pixel a pixel sem a junta dentro do traço · calculado sem a ponta do fim · recuo
da emenda de um troço do meio · pixel a pixel salta blocos tracejados · fechado não marcado (Rust) ·
contagem de troços a menos (Rust). ⭐ A T8 (pontas trocadas) SOBREVIVEU à 1.ª corrida — todas as famílias
tinham as duas pontas iguais; a família «pontas diferentes» foi escrita por ela.

**⚠️ Divergência DECLARADA — a mordida.** Um traço RENTE que acaba a menos de meia largura depois de uma
quina sai do traçador da casa com uma MORDIDA no lado de dentro (a junta interior passa pelo pivô e o
pedaço curto cruza-se; a régua EXACTA — `kurbo::stroke` a expandir e o Vello só a preencher — tem a mesma
mordida, alfa `52` do traço do Vello); a placa desenha a união verdadeira. Medido numa cópia (pedaço de
`9,75 px`, raio `11,4`): `140` de alfa num pixel; na rota do produto, estrela com pontas rentes, `203`. Os
gates usam ponta quadrada nessas famílias. Mesma família da quina exacta do §9.4.

**⚠️ A régua tem defeitos próprios.** Com traço grosso numa volta apertada o Vello abre uma rachadura de
um pixel dentro do traço (alfa `0`–`136` onde a placa pinta `255`); as fixturas evitam-na (largura `0,07`).

**✅ O AJUSTE DA EMENDA NO ECRÃ (`042327a6a`, curado na mesma jornada).** O ajuste do tracejado (`dash_fit`,
a cura da emenda de 22/08) media o contorno no espaço LOCAL e o padrão era depois escalado por `√|det|`;
sob escala não uniforme o comprimento no ecrã não é `√|det|·L`, e a emenda voltava — nas DUAS rotas (e no
vetor de documento). Agora, no ramo NÃO conforme:

- **a casa** (`ph2d_vec_render::ajusta_no_ecra`, chamada por `stroke_uniform` e `stroke_uniform_image`)
  ajusta o padrão da caneta (já `× √|det|`) ao sub-caminho mais LONGO da geometria JÁ transformada, pela
  mesma `dash_fit::fit` (`n` períodos num fechado; `n` mais um traço num aberto). Sob afim conforme o ecrã
  é o local escalado e nada muda (esse caminho nem passa lá). Gates `o_tracejado_esticado_fecha_no_contorno_do_ecra`
  (com CONTROLO: o ajuste local deixava a emenda a `0,7` de período) e `…_aberto_acaba_com_traco_inteiro`;
- **a placa** (`ajuste_do_tracejado`, por cópia) faz a MESMA conta — `floor(x + 0,5)`, porque o `round` do
  WGSL arredonda as metades para o par e o `f64::round` para longe do zero;
- ⭐ **a folga** `FOLGA_DO_AJUSTE = 1e-4`: o ajuste exacto punha o fim de um fechado EXACTAMENTE na
  fronteira traço/vão, onde o `f64` do kurbo e o `f32` da placa decidiam a emenda por arredondamento;
  alongado o período, o fim cai sempre DENTRO do último vão (`≤ 0,3 px` numa volta de `3 000 px`). As
  mutações «sem a folga» SANGRAM nas duas rotas (placa: alfa `204`–`255`; casa: `255`) — a ambiguidade
  era real, não teórica;
- ⚠️ consequência: o contorno mais LONGO nunca mais emenda; a emenda só acontece num sub-caminho mais
  curto (um furo). As mutações da emenda (`T2`, `T13`) SOBREVIVERAM até à família «estrela com furo»
  (furo HEXAGONAL: numa ponta de estrela a faixa nunca serve e o recuo da emenda não decide nada);
- a régua da crate passou a CHAMAR `pen_for` + `ajusta_no_ecra` (dev-dep `ph2d-vec-render`) em vez de
  copiar a lei.

Medido (RTX): famílias esticadas alfa `≤ 84` · cor `≤ 63`; estrela com furo `63` · `57`; produto `85` ·
`58`. **Mutação `21` de `21`** (as `17` de cima com as âncoras novas, mais «placa sem o ajuste» · «placa
sem a folga» · «casa sem o ajuste» · «casa sem a folga»).

**Smoke.** `=127` com `PH2D_TRACO_ESTICADO_TRACEJADO=1` (o `Dash 2` e `Dash Gap 1,5` do cartão postos à
partida); fotografado na tela virtual: `1024` cópias pela PLACA (do dispositivo).

**Os glifos (`7a58e7aaf`).** Os glifos do `source.text` já iam à placa desde a W3 — o censo dava `6` das
`7` cenas com texto na placa, e a 7.ª era a `=76` pelo tracejado; mas nenhum gate de pixel tinha um
glifo. Gate das LETRAS: alfa `63` · cor `62` · `2 990` px `> 16` sobre `81 229`; as mutações «sem
furos» (`38 006` px) e «aplanamento 8×» (`8 121` px) reprovam.

