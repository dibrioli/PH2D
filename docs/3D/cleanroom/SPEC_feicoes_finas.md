# SPEC — feições finas e pontiagudas: o que a cadeia de referência faz (e o que ela **não** faz)

```
Alvo: a cadeia de remalhagem quad de referência da família do quad remesh (SIGGRAPH 2021 +
      a variante de quantização por fluxo, SIGGRAPH 2023), na revisão do clone local de 2026-08-20
Licença: GPL-3.0 · Degrau: T2 (a concessão do §2 da GPLv3 cobre ler/modificar/correr em privado;
      nenhum acto desta espec é *convey*)
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md — ADENDO-E nº2, aberto 2026-08-30 ANTES da 1ª leitura
Patente (§8.1): buscado em 2026-08-24 (LEDGER + TRIAGEM §3) — ⛔ NÃO repetido neste adendo, e
      nada aqui alarga o alcance daquele veredito: nenhuma patente viva bloqueia campo → mapa →
      extracção; as duas vivas (redução de singularidades por gabarito · quad por esboço autorado)
      continuam fora do nosso caminho
Filtragem §4.3: executada em 2026-08-30 · Sweep: verde em 2026-08-30
Auditoria §4.2 (R-pré): ⏳ PENDENTE — esta espec é entregue com a auto-filtragem apenas; ela
      descreve comportamento e não prescreve implementação nova, então não é condição de abrir wave
Mapa de leitura da literatura (tudo público, lícito a TODOS os papéis):
  · Pietroni, Nuvoli, Alderighi, Cignoni, Tarini — *Reliable Feature-Line Driven Quad-Remeshing*,
    ACM TOG 40(4), 2021 · doi:10.1145/3450626.3459941 · PDF aberto:
    https://vcgdata.isti.cnr.it/Publications/2021/PNACT21/ReliableQuad.pdf
    → §3 (visão geral e parâmetro principal) · **§4.2 (preparação da malha — a fase adaptativa)** ·
      §4.3 (campo) · §6.1 (quantização) · §8.1 (limitações). ⏭️ PULAR: §5 (traçado do layout) e
      §7 (resultados) — fora do alcance desta pergunta.
  · Heistermann et al. — *Min-Deviation-Flow in Bi-directed Graphs for T-Mesh Quantization*,
    SIGGRAPH 2023 · https://www.algohex.eu/publications/bimdf-quantization
  · Fio público de perguntas do repositório, sobre densidade de quads — resposta do 1º autor.
    ⚠️ **PROSA pública, não código; a citação já está re-dita no §1.5 e o Implementador NÃO
    precisa de a abrir.** Endereço guardado para o ledger e para o R:
    `<repositório do alvo>/issues/9` (o anfitrião está na denylist abaixo).
Denylist de URLs (⛔ o Implementador NUNCA abre):
  · https://github.com/nicopietroni/quadwild (e qualquer *fork*/espelho/*code search* dele)
  · https://github.com/cgg-bern/quadwild-bimdf (idem)
  · qualquer motor de busca de CÓDIGO apontado a esses repositórios
  · o clone local em ~/Documentos/Projetos/ph2d-quadbench/oracle/ (o `deny` do Passo 0 já o cobre)

"Este documento descreve comportamento; não contém expressão do alvo."
```

---

## §0 — A pergunta, e as três respostas em uma linha cada

> **Como é que a cadeia de referência trata uma feição fina e pontiaguda cuja espessura é menor
> que o passo alvo da grade?** (o caso: uma bola de espinhos.)

| | resposta curta |
|---|---|
| **(a) adapta a densidade perto da ponta?** | **Sim, mas não onde procurávamos, e não pelo motivo que supúnhamos.** A adaptação é **inteiramente da fase de PREPARAÇÃO** (a remalhagem triangular), é **local** e o critério é a **FORMA do triângulo**, não a curvatura nem a espessura. ⛔ Nada a jusante tem passo variável: a densidade dos quads é **UM escalar global**. |
| **(b) protege contra amputar a ponta?** | **NÃO existe protecção explícita, e não existe régua nenhuma de fidelidade.** O que existe é uma **garantia estrutural de cobertura**: a saída não é *extraída* de um mapa — ela é **preenchida** sobre uma partição da superfície, com piso de **1** quad por sub-lado. Cobertura é propriedade da construção, não de uma verificação. |
| **(c) o que impede a saída de partir em componentes soltos?** | **A construção, e mais nada.** Não há verificação nem reparação de componentes no caminho do produto. Um componente solto é **impossível por construção**; o modo de falha real é o **BURACO** (um retalho cuja quantização não fechou é **saltado**, com aviso na consola). A contagem de componentes só existe numa ferramenta de medição **fora** da cadeia. |

⚠️ **A consequência mais importante para nós está no (b)**: a diferença não é uma afinação — é de
**classe**. Uma cadeia que *preenche uma partição* não pode amputar; uma cadeia que *extrai isolinhas
de um mapa* pode, e a nossa é da segunda espécie. ⇒ **a nossa cura para a amputação não pode ser
copiada de lá; ela tem de ser inventada, ou o preenchimento tem de mudar de classe.**

---

## §1 — (a) A densidade: onde a adaptação entra, e o que a conduz

### §1.1 — A arquitectura de fases, e onde cada número mora

| fase | o que ela decide sobre densidade | é local? |
|---|---|---|
| **1. preparação (remalhagem triangular)** | ⭐ **AQUI mora TODA a adaptação.** Duas passagens: uniforme, depois adaptativa | **sim** |
| 2. campo de direcções | nada | — |
| 3. traçado / partição em retalhos | nada de densidade; decide **onde** ficam as fronteiras | — |
| 4. quantização (quantos quads por lado) | recebe **um comprimento-alvo por retalho** — e a cadeia preenche **o mesmo número em todos** | ⛔ **não** |
| 5. preenchimento de cada retalho | a densidade interior é **ditada pela fronteira**; o interior é um mapa conforme de mínimos quadrados com **toda** a fronteira presa por comprimento de arco | ⛔ não |
| 6. acabamento | projecção e alisamento; **não** muda contagens | ⛔ não |

⇒ ⛔ **A cadeia de referência NÃO resolve nenhum mapa escalar com passo variável.** Ela **não tem
mapa de grade inteira global** no caminho de omissão (existe uma rota alternativa desse tipo, mas
está atrás de um interruptor de compilação e não é a que produz os resultados publicados).
**A segunda metade da nossa sub-pergunta — *«como é que a variação entra sem que a projecção de
mínimos quadrados a descarte?»* — simplesmente NÃO SE PÕE lá.** Isso é um resultado de primeira
classe: *o problema que medimos é um preço da nossa classe de algoritmo, não uma falha de
afinação nossa.*

### §1.2 — A lei da adaptação, na fase de preparação

**Duas passagens de operações locais** (dividir aresta · colapsar aresta · trocar aresta · alisar ·
projectar), no molde clássico da remalhagem isotrópica:

1. **Passagem uniforme** — um comprimento-alvo único para a malha inteira.
2. **Passagem adaptativa** — o mesmo comprimento-alvo **multiplicado por um factor local**.

**O factor local, passo a passo (a lei, não a escrita):**

- Para cada triângulo, mede-se a **razão entre o raio inscrito e o circunscrito**, normalizada de
  modo que o equilátero valha `1`. Toma-se o **complemento** (`1 −` essa razão) ⇒ um número que é
  **grande onde o triângulo é uma lasca** e **≈ 0 onde ele é bem-formado**.
- Esse valor por face é **transferido para os vértices** (média das faces incidentes).
- **Normaliza-se ao intervalo `[0, 1]`** pelo mínimo e máximo da distribuição, e **eleva-se ao
  quadrado** — o que empurra a maioria dos vértices para perto de `0` e reserva a ponta alta
  para os poucos casos genuinamente maus.
- O factor é a **interpolação linear entre `3` (onde o valor é 0, isto é, malha boa) e `0,3`
  (onde o valor é 1, isto é, lasca)**, saturada nas duas pontas.
- Esse factor multiplica **os dois** limiares: o de **dividir** (aresta longa demais) e o de
  **colapsar** (aresta curta demais).

⇒ **Faixa medida: `0,3×` a `3×` o comprimento-alvo — um espalhamento de `10×`.**
⇒ **Direcção:** onde a malha **não consegue** ser bem-formada àquele comprimento, ela **afina**
(até `0,3×`); onde ela já é boa, **engrossa** (até `3×`).

**Proveniência dos números:** os valores `0,3` e `3` e a régua da razão de raios estão **no §4.2 do
paper de 2021 e são os defaults do código lido em 2026-08-30**; o *paper* também nomeia a
**clamagem no percentil 10** da distribuição de qualidade. O quadrado da normalização e a
transferência face→vértice **só se lêem no código** (o *paper* não os menciona).

⚠️⚠️ **E o RACIONAL dos autores é o achado que muda a nossa leitura.** Eles **não** dizem que isto
serve para pontas finas. Eles dizem — §4.2 do *paper*, re-dito em palavras nossas — que a
preservação das **linhas de feição** força triângulos pequenos e mal-formados **onde muitas linhas
de feição se juntam**, e que essa má-formação é lida como *«aqui é preciso mais resolução, porque o
campo de direcções vai precisar de detalhe fino»*. ⇒ **a régua é um PROXY da complexidade das
linhas de feição, não da espessura da peça.** Numa bola de espinhos **lisa** (sem quinas), não há
linha de feição nenhuma, e este mecanismo **não é accionado pela ponta** — ele só é accionado
indirectamente, se a ponta produzir lascas na primeira passagem.

### §1.3 — ⭐ A adaptação GLOBAL que ninguém anuncia: o alvo sai da **esfericidade**

O comprimento-alvo da preparação **não é dado pelo utilizador** no caminho de linha de comando: ele
é **derivado da malha**, e a derivação tem um termo que reage exactamente ao caso «bola de
espinhos». Com `A` = área da superfície e `V` = volume:

```
esfericidade  =  π^(1/3) · (6·V)^(2/3) / A          (1 para a esfera, → 0 para uma peça espinhosa)

L₀  =  aresta do equilátero de área  (A / 2000) · esfericidade²
L₁  =  aresta do equilátero de área  (A / 10000)

comprimento-alvo  =  min(L₀, L₁)
```

*(a conversão área→aresta do equilátero é `L² = (4/√3)·A ≈ 2,309·A`)*

**O que isto faz, em contagem de faces:** `L₀` corresponde a `2000 / esfericidade²` triângulos e
`L₁` a `10000`. O `min` escolhe **o mais fino**, logo:

| esfericidade da peça | quem manda | triângulos da preparação |
|---|---|---|
| `1,00` (esfera) | `L₁` | `10 000` |
| `0,45` (o ponto de troca, `esfericidade² = 0,2`) | empate | `10 000` |
| `0,30` | **`L₀`** | **≈ 22 000** |
| `0,20` | **`L₀`** | **≈ 50 000** |

⇒ ⭐ **Uma peça espinhosa recebe uma preparação globalmente MUITO mais fina, e o mecanismo é um
descritor de forma — não uma medida local de espessura.** Isto é adaptação a feições finas: só que
**global**, e paga na malha inteira.

⚠️ **E o *paper* diz outra coisa** — nele o parâmetro principal é *«a aresta do quadrado com `1/K`
da área da malha, `K = 10⁴`»*, que é exactamente o `L₁` acima **sem** o termo de esfericidade.
⇒ *o código que shipa é mais agressivo que o método publicado, e o termo extra é precisamente o que
reage a uma peça fina.* **Confira sempre o código antes de citar o paper como se fosse o produto.**

### §1.4 — Como a densidade da preparação chega (e não chega) aos quads

- O tamanho do quad é **um escalar só**: a **aresta média da malha preparada**, vezes um factor de
  escala do utilizador (que por omissão é `1`). A cadeia entrega esse mesmo número **para todos os
  retalhos**, embora a interface aceite **um por retalho**.
- ⇒ **o acoplamento é GLOBAL**: refinar perto de um espinho baixa a aresta **média**, e isso encolhe
  **todos** os quads da peça. ⛔ Não existe caminho por onde a densidade **local** da preparação
  produza quads **localmente** menores.
- Dentro de um retalho, a contagem de quads sai **só da fronteira** (quantos segmentos em cada
  sub-lado) e o interior é um **mapa conforme de mínimos quadrados** com **toda** a fronteira presa
  por comprimento de arco. ⚠️ **É exactamente o desenho que a nossa linha construiu, mediu e
  arquivou como pior** — e a referência sai melhor com ele. ⇒ *a diferença entre nós e ela, nessa
  coluna, não está no interior do retalho; está no LAYOUT que decide onde os retalhos ficam.*
- Para o lote de 10 mil peças do *paper*, os autores declaram ter usado como aresta-alvo do quad
  **o dobro** da aresta média da malha já preparada. O caminho de linha de comando que shipa usa
  **o factor `1`** por omissão — *o mesmo pipeline entrega metade do tamanho de quad que o paper
  descreve.*

### §1.5 — A palavra dos autores (§4.1.12), com o link

No fio público de perguntas do repositório (a entrada nº 9; endereço no cabeçalho, e ⛔ o
Implementador não precisa de a abrir — a substância está aqui), o primeiro autor responde a alguém
que estranhou quads de tamanhos diferentes na mesma peça. Re-dito em palavras nossas: **o tamanho é
controlável só até certo ponto, porque depende do campo**; num retalho com lados de comprimentos
diferentes, exigir regularidade obriga os quads de um lado a serem menores que os do outro; e que
**mais isometria — quads do mesmo tamanho — custa necessariamente MAIS singularidades**; daí
existir um parâmetro que mistura **regularidade** e **isometria**.

⇒ ⭐ **A não-uniformidade do tamanho do quad é, na cadeia de referência, uma consequência ACEITE do
layout, não um defeito.** Ela troca-se por singularidades, e a troca é um botão.

---

## §2 — (b) A ponta amputada: o que existe, e o que **não** existe

### §2.1 — ⛔ O que NÃO existe (e é o resultado principal)

1. ⛔ **Nenhuma noção de «preservar extremos».** Não há detecção de ponta, de protuberância, de
   extremidade, de espessura mínima, nem qualquer termo que puxe a saída para os pontos mais
   distantes da entrada. Procurado explicitamente; **não existe em fase nenhuma**.
2. ⛔ **Nenhuma régua de FIDELIDADE, em lugar nenhum.** A ferramenta de métricas que acompanha a
   cadeia mede **topologia** (vértices, arestas, faces, característica de Euler, género, buracos,
   componentes, estanqueidade, manifoldidade) e **qualidade de forma por face** (área, desvio
   angular, planaridade, dobra, torção, comprimento de aresta, área de Voronoi). ⛔ **Nenhuma
   distância à entrada. Nenhuma cobertura. Nenhum Hausdorff.**
   ⇒ *uma amputação não seria apanhada pelos instrumentos deles também.* ⚠️ É a mesma família de
   cegueira que o nosso `QuadShape` mediano teve com os três quads emaranhados na ponta.

### §2.2 — ⭐ O que existe em vez disso: **cobertura por construção**

A garantia é arquitectural, e vale a pena escrevê-la como uma cadeia de quatro elos:

1. O traçado **particiona a superfície inteira** em retalhos: **toda** face triangular pertence a
   **exactamente um** retalho. Não há «zona não coberta» possível.
2. A quantização atribui a cada **sub-lado** um número **inteiro ≥ 1** de segmentos — o piso é
   **1**, imposto como limite inferior da variável em ambos os solucionadores (o de programação
   inteira e o de fluxo). ⇒ **um retalho não pode encolher a zero.**
3. Cada retalho é **preenchido** por um molde de quads escolhido pela sua lista de contagens de
   lados. Os moldes cobrem **de 3 a 6 lados**; o gerador aceita `2..6` lados, exige **soma par**
   (garantida por uma restrição de paridade que é dura por omissão) e **soma ≥ 4**.
4. Cada vértice do molde é localizado no mapa do retalho e levado a 3D por coordenadas
   baricêntricas. ⚠️ **Se um vértice do molde cair FORA do domínio do mapa, ele é encostado ao
   triângulo mais próximo — nunca descartado.** ⇒ *a construção nunca deixa de emitir um quad.*

⇒ **Uma ponta fina sobrevive porque ela está dentro de algum retalho, e todo retalho é preenchido
com pelo menos um quad de travessia.** Se o traçado der um retalho próprio ao espinho, o espinho é
representado; se o espinho ficar no interior de um retalho grande, ele é representado **mal** (a
geometria fica ao critério do mapa conforme), mas **não desaparece**.

### §2.3 — As três guardas geométricas que existem, e o alcance real de cada uma

| guarda | o que ela mede | protege contra amputar? |
|---|---|---|
| **Tolerância de desvio de superfície** na preparação: toda operação candidata (colapso, troca, alisamento, projecção) é **recusada** se o ponto novo ficar a mais de **`diagonal da caixa / 2500`** da malha **original** | distância **do novo para o velho** | ⛔ **NÃO.** É **unilateral**. Cortar uma ponta deixa todos os pontos restantes perto da superfície original — a guarda passa. Ela impede **inchaço e deriva**, não **perda**. |
| **Linhas de feição preservadas por construção** — nenhuma operação que destrua uma aresta de feição é permitida, em fase nenhuma; e as feições reaparecem como fronteiras de retalho na saída | as **quinas** | ⚠️ **só se a ponta for uma QUINA.** Numa bola de espinhos **lisa** não há feição nenhuma a detectar (o limiar é um **ângulo diedro**, `35°` por omissão, e um perfil «orgânico» **desliga-o**). *A protecção mais forte da referência é exactamente a que uma ponta lisa não activa.* |
| **Reprojecção final**: cada vértice do quad é encostado ao ponto mais próximo da malha triangular preparada, com raio de busca = **a diagonal da caixa** (i.e. sem limite prático) | distância **do quad para a superfície** | ⛔ **NÃO.** Também unilateral. Ela põe os quads **sobre** a superfície; não põe quads **onde não há nenhum**. |

⚠️ **Nota de escala:** a guarda de desvio é **desligada** acima de `400 000` faces, declaradamente
por custo. ⇒ *numa peça grande, nem a guarda unilateral corre.*

### §2.4 — A limitação que os autores **declaram** (§8.1 do *paper*), e a que fica implícita

Sob o título de limitações, eles declaram **duas** coisas, nenhuma delas sobre pontas:
a **ausência de garantia estrita** de que as condições mínimas do layout sejam sempre satisfazíveis
(o procedimento apoia-se em heurísticas), e a **susceptibilidade a linhas de feição mal
classificadas ou ruidosas** — com a nota de que a rotulagem por ângulo diedro funciona bem em peças
mecânicas e exige mais cuidado em malhas digitalizadas. As falhas do lote de ~10 mil peças
(`< 0,5 %`) são atribuídas sobretudo a **malhas não-orientáveis**.

⚠️ **E há uma fraqueza declarada na discussão que é a nossa pergunta vista do outro lado:** escolher
a tesselação interior de um retalho **só a partir da fronteira dele** pode, em teoria, custar
isometria (re-dito em palavras nossas a partir do §8 do *paper*). Os autores acrescentam — e é a metade que interessa — que **isso não pode acontecer
quando o interior do retalho tem feição geométrica significativa, porque a feição induziria
singularidades de campo, e essas provocariam a divisão do retalho.**

⇒ ⭐⭐ **Aí está a defesa REAL da ponta, e ela é indirecta:** *um espinho, se for geometricamente
significativo, cria singularidades no campo; o traçado parte o retalho ali; e a ponta ganha
fronteira própria — logo, contagem própria de quads.* **A protecção da ponta não é uma verificação:
é o CAMPO a acordar o traçado.** Se o campo não acordar (espinho demasiado fino para o passo do
campo, ou campo demasiado alisado), a ponta cai dentro de um retalho grande e a referência
degrada-se exactamente como nós — só que sem partir, porque preenche.

---

## §3 — (c) Componentes soltos: por que a referência não os produz

### §3.1 — A montagem, e a razão de a saída ser conexa

A malha de quads é montada **retalho a retalho** sobre uma partição **conexa** da superfície. Os
vértices de fronteira **não são criados duas vezes**: o primeiro retalho a chegar a um sub-lado cria
os vértices dele e **regista-os**; o retalho vizinho **reutiliza os mesmos**. Os cantos são
partilhados pela mesma via. ⇒ **a conectividade da saída herda a do grafo de adjacência dos
retalhos, e esse cobre a superfície.** Um pedaço fechado a flutuar solto **não tem por onde
nascer**.

⚠️ **Corolário para nós:** a nossa ilha de 22 faces **não é um defeito que a referência tenha
curado — é um defeito que a classe dela não pode ter.** ⛔ Não há lá cura para importar.

### §3.2 — O modo de falha REAL: o buraco, e ele é anunciado

Se a quantização **não fechou** para algum sub-lado de um retalho, esse retalho é **saltado inteiro**
— com uma linha na consola a nomear o retalho. Resultado: **um buraco na malha**, com bordo, não uma
ilha. E há um segundo ponto de aborto: se, já dentro do preenchimento, aparecer um sub-lado sem
valor, a rotina **desiste da malha toda** e devolve o que tinha.

⇒ **Buraco anunciado ≠ ilha silenciosa.** ⚠️ *A referência escolheu falhar em voz alta na fase que
sabe se falhou.*

### §3.3 — As limpezas que existem, e o que cada uma faz

**Na malha TRIANGULAR (preparação), num laço que repete até estabilizar, no máximo `10` rondas:**
faces colineares · faces de área zero · abertura de arestas não-manifold · **remoção de componentes
ligados com `≤ 10` faces** · orientação coerente · reposicionamento de vértices duplicados ·
divisão e remoção de dobras de 180°. E, entre as duas passagens de remalhagem, um **colapso de
micro-arestas** que ataca triângulos com razão de raios `≤ 0,001` (e `≤ 0,01` na chamada do caminho
principal).

⇒ ⚠️ **A única remoção de componentes da cadeia inteira é esta, é na ENTRADA, e o limiar é `10`
faces.** A nossa ilha de **22** faces **passaria** por ela mesmo que a tivéssemos — *um limiar de
contagem não separa lixo de peça; ele separa lixo pequeno de lixo grande.*

**Na malha de QUADS, depois da montagem e antes da projecção:**

| limpeza | o que faz | liga/desliga |
|---|---|---|
| vértices duplicados | funde por coincidência | sempre |
| faces degeneradas | remove vértices repetidos dentro de uma face; **apaga a face** se sobrarem `< 3`, e por omissão apaga-a de qualquer modo quando algo mudou | sempre |
| orientação | reorienta as faces coerentemente | sempre |
| ⭐ **doublets** | **funde os dois quads que partilham DUAS arestas consecutivas num só quad**, removendo o vértice de valência 2 — **recursivamente, até estabilizar** | **ligada por omissão** |
| vértices sem referência | remove | sempre |

⇒ ⭐⭐ **A remoção recursiva de doublets é a MESMA cura que a nossa linha construiu em 29/08** —
e a referência corre-a **sempre**, no caminho de omissão, **antes** de projectar. *A nossa
`dissolve_doublets` não é uma invenção nossa nem um remendo: é a peça padrão que faltava.*
⚠️ E a referência **não** repara doublets que já venham na **entrada** — a entrada dela é uma malha
triangular. *A nossa segunda metade (reparar o que a peça já traz) continua a ser nossa, e é
obrigatória porque o nosso botão come a própria saída.*

### §3.4 — Onde a contagem de componentes vive

Só na **ferramenta de métricas**, que é um **binário separado**, corre **depois**, e escreve um
relatório. Ela conta componentes, característica de Euler, género, buracos, estanqueidade e
manifoldidade. ⛔ **Nada disso é lido pela cadeia; nada disso decide coisa nenhuma; não há
reparação associada.**

⇒ ⚠️ *Medir e não agir é a postura declarada da referência. A nossa é diferente por decisão — mas a
régua deles nomeia exactamente o conjunto de grandezas que a nossa linha teve de descobrir uma a
uma (a almofada por contagem de componentes, a mordida por não-manifold, o rasgo por bordo).*

---

## §4 — As constantes observadas, com proveniência

⚠️ Cada linha é **um facto de comportamento medido pela leitura de 2026-08-30**, não uma
recomendação. ⛔ **Nenhuma delas entra no nosso código sem uma medição nossa ao lado** (§0.0 da casa).

| grandeza | valor observado | onde | comentário |
|---|---|---|---|
| passagens de remalhagem na preparação | **2** (uniforme, depois adaptativa) | código + *paper* §4.2 | |
| iterações por passagem | **15** | config do produto | |
| factor adaptativo do comprimento-alvo | **`0,3×` .. `3×`** | código + *paper* §4.2 | espalhamento `10×` |
| régua do factor | razão raio inscrito / circunscrito, complementada, normalizada `[0,1]`, **ao quadrado** | código (o quadrado **não** está no *paper*) | |
| clamagem da distribuição | percentil **10** e **90** | código + *paper* (só o 10) | |
| alisamento da régua | Laplaciano, **2** rondas | código | |
| razão de aspecto-alvo da remalhagem | **`0,35`** | config do produto | |
| tolerância de desvio de superfície | **diagonal da caixa `/ 2500`** | código | **unilateral** |
| desligar essa tolerância | acima de **`400 000`** faces | código | por custo |
| limiar de colapso de micro-aresta | razão de raios **`≤ 0,01`** (o default interno é `0,001`) | código | |
| alvo global da preparação | `min( aresta_eq((A/2000)·esfericidade²), aresta_eq(A/10000) )` | código | ⚠️ **o *paper* só tem o segundo termo** |
| conversão área→aresta do equilátero | `L² ≈ 2,309·A` | código | `4/√3` |
| limiar de ângulo diedro das feições | **`35°`** (perfil mecânico igual; perfil **orgânico = desligado**) | config do produto | o *paper* usou **`45°`** no lote |
| erosão/dilatação das feições | **4** | código | remove feições curtas e ruidosas |
| piso de subdivisão por sub-lado | **1** | ambos os solucionadores | ⭐ a garantia anti-colapso |
| paridade da soma dos lados | **dura**, por omissão | config do produto | |
| lados admissíveis por retalho | **3 .. 6** (o gerador de moldes aceita `2..6`, soma **par**, soma **≥ 4**) | código | |
| tamanho do quad | **aresta média da malha preparada × factor de escala** (`1` por omissão), **igual para todos os retalhos** | código | ⚠️ o *paper* declara **`2×`** no lote |
| escala de comprimento das zonas «estreitas» do traçado | **`0,05 × √(área total) × deriva`** | código | ⚠️ ver §5 |
| remoção de componentes pequenos (triângulos) | **`≤ 10` faces**, laço de até **10** rondas | código | ⛔ só na entrada |
| remoção de doublets (quads) | **recursiva**, **ligada** por omissão | código | ⭐ |
| reprojecção final | ponto mais próximo, raio = **diagonal da caixa** | código | unilateral |
| alisamentos finais (3 knobs) | **`5`** cada, por omissão da biblioteca — mas o caminho de linha de comando põe os três a **`0`** | código | *o produto shipa sem os alisamentos que a biblioteca oferece* |
| falhas declaradas no lote de ~10 mil peças | **`< 0,5 %`**, sobretudo malhas **não-orientáveis** | *paper* §8.1 + resumo | |

---

## §5 — ⛔ Duas armadilhas de leitura, nomeadas para ninguém as pisar

1. ⛔ **A palavra «estreito» aparece no traçado e NÃO é sobre espessura geométrica.** Existe lá uma
   classificação de vértice com esse nome, e ela é sobre **topologia de corte**: depois de a malha
   ser cortada ao longo das linhas de feição, um vértice de bordo cuja **posição** aparece uma única
   vez entre os vértices de bordo é rotulado assim. É um **corte que deixou uma fita de um lado só**
   — não é a ponta de um espinho. ⚠️ *Procurar «narrow» à espera de uma cura para pontas devolve a
   coisa errada, e ela até tem constantes bonitas ao lado.*
2. ⛔ **Existe no repositório um caminho alternativo de parametrização global** (a família do mapa de
   grade inteira, que é a NOSSA classe). Ele está **atrás de um interruptor de compilação**, não é o
   caminho de omissão, e **não** é o que produz os resultados publicados. ⚠️ *Comparar a nossa cadeia
   com «o que eles fazem» exige saber qual dos dois se está a olhar.*

---

## §6 — O que isto muda para a nossa linha (e o que NÃO muda)

⚠️ Esta secção é **nossa**: não descreve o alvo, decide o que fazer com o que ele mostrou.

### §6.1 — O que fica DESACONSELHADO por este achado

- ⛔ **Não construa «passo variável no mapa escalar» a partir da referência.** Ela não tem um.
  O nosso achado de 28/08 — *o alvo com `h` variável deixa de ser gradiente integrável e a
  adaptação é projectada fora* — **não tem contra-exemplo lá**. A cura publicada que já
  nomeámos (o factor de escala **conforme por construção**) continua a ser o único caminho com
  endereço, e continua a ser **nosso**.
- ⛔ **Não copie um limiar de contagem de faces para matar ilhas.** O único que existe lá é `≤ 10`,
  na **entrada**, e a nossa ilha tinha `22`.

### §6.2 — O que fica RECOMENDADO, com o mecanismo

1. ⭐⭐ **A régua que falta é de COBERTURA, e a referência prova-o pela ausência:** nem ela nem os
   instrumentos dela medem distância da saída à entrada. A nossa amputação de `−20 %` a `−35 %` de
   alcance de espinho é invisível a **toda** régua que os dois lados têm hoje. ⇒ **antes de tocar
   em código, construa a régua unilateral que falta: para cada vértice da ENTRADA, a distância ao
   quad mais próximo da SAÍDA** (a direcção que ninguém mede — a inversa, saída→entrada, já é
   medida pelos dois lados e é a que passa numa amputação).
2. ⭐ **O piso de `1` por sub-lado é barato e é a garantia estrutural deles.** Se a nossa cadeia tem
   qualquer sítio onde uma contagem inteira pode chegar a `0`, esse sítio é candidato a amputação
   silenciosa e o piso é a cura conhecida.
3. ⭐ **A remoção recursiva de doublets pertence ao caminho de omissão, não a uma opção.** A
   referência corre-a sempre, antes de projectar. A nossa já existe desde 29/08 — **confirme que ela
   corre no caminho de omissão e antes do acabamento**, que é onde ela corre lá.
4. ⭐ **O acoplamento densidade↔forma da peça pode ser copiado, e é uma linha:** ancorar o alvo na
   **área** já fazemos (foi a cura da idempotência); o termo que nos falta é o de **esfericidade**,
   que faz uma peça espinhosa nascer globalmente mais fina. ⚠️ **Mas isto é um GLOBAL, e o report do
   Enio é LOCAL** — ele paga a peça inteira para curar as pontas. ⛔ Não o adopte sem medir o preço
   contra o tecto de contagem que já temos.
5. ⚠️ **A defesa real da ponta é o CAMPO a partir o retalho.** Antes de construir densidade
   adaptativa, meça se o nosso campo **acorda** numa ponta fina (há singularidade ali?) e se o nosso
   traçado **reage** a ela. Se a resposta for não, a densidade não vai salvar a ponta — ela vai
   apenas encarecer a peça inteira.

### §6.3 — A pergunta que fica em aberto, e é para o dono

⏳ A referência aceita **explicitamente** trocar isometria por singularidades, e tem um botão para
essa mistura. A nossa cadeia **não expõe essa troca**. *Quads de tamanhos diferentes numa peça é,
lá, comportamento correcto declarado.* ⇒ **é decisão de produto** se as pontas finas devem ganhar
quads menores (mais singularidades, malha menos regular) ou manter o tamanho (menos singularidades,
pontas mais grosseiras). ⛔ Não é uma pergunta que um gate responda.

### §6.4 — ⛔⛔ ACHADO COLATERAL, dentro do alcance: uma constante NOSSA de densidade tem a PROVENIÊNCIA errada

⚠️ Isto apareceu ao varrer a árvore e é **exactamente** sobre densidade, logo dentro do bound.

`crates/ph2d-remesh-iso/src/lib.rs` declara `pub const ALPHA: f32 = 0.02` com o doc-comment a
dizer que é *«a fracção da diagonal da caixa que vira o lado do triângulo»* e a atribuir o número
a um preset do oráculo, nomeando o ficheiro de configuração dele.

⛔ **No oráculo, esse número NÃO é um comprimento de remalhagem — em nenhuma das duas leituras
possíveis:**

| onde | o que o número significa lá |
|---|---|
| no binário de linha de comando que produz os resultados publicados | **a mistura regularidade ↔ isometria** do objectivo da quantização (`0` = mais regular · `1` = mais singularidades inseridas). É o que a documentação pública dele diz, com essas palavras. |
| no programa **separado** de campo (GUI) | o **peso de alinhamento à curvatura** do alisador de campo |

⇒ **A nossa constante pode continuar certa** — o doc dela diz que foi **MEDIDA**, com tabela sobre
o cubo — **mas a frase de proveniência está errada e tem de sair ou ser corrigida.** ⚠️ É a
armadilha do §4.3.2 da disciplina de espec e do §0.0 da casa ao mesmo tempo: *um número com uma
proveniência falsa lê-se como número com medição*, e o próximo agente que quiser mexer nele vai
procurar autoridade no sítio errado — ou pior, **importar o valor de um knob que controla outra
coisa** quando quiser ajustar a densidade.

⛔ **Isto NÃO é um vazamento de parede** (o nome do ficheiro de configuração é público, vem do
`README` do alvo, e já estava na árvore antes deste adendo) — é um **defeito de proveniência**.
A cura é uma linha de doc-comment e é do Implementador.

---

---

## §7 — Cobertura desta travessia (o que foi lido, e o que NÃO foi)

**Lido em 2026-08-30, com alcance dirigido (§3.E do skill — a cobertura integral já tinha sido
decidida como não-insumo pelo ADR-0167):**

| área | o que se procurou |
|---|---|
| binário de linha de comando + funções de condução | a ordem das fases e de onde sai cada número |
| ficheiros de configuração do produto (**6**) | os defaults como factos |
| remalhador de preparação | a lei da adaptação e as guardas |
| biblioteca de remalhagem isotrópica (o motor de baixo) | a definição exacta do factor adaptativo, dos limiares e da guarda de desvio |
| gestor de preparação da malha | as limpezas, os limiares e a ordem |
| alisador do campo | se a densidade entra pelo campo (⇒ **não entra**) |
| condutor da quantização + parâmetros da biblioteca de retalhos | o piso de subdivisão, a paridade, os alisamentos |
| solucionador por programação inteira e solucionador por fluxo | os limites inferiores das variáveis |
| montagem e limpeza da malha de quads | a conectividade, os doublets, a reprojecção |
| mapeamento retalho→molde | como o interior é decidido, e o que acontece fora do domínio |
| classificador de vértices do traçado | a armadilha do §5.1 |
| ferramenta de métricas | o que a referência mede sobre si própria |
| *paper* de 2021 (§3, §4.2, §4.3, §6.1, §8.1) e fio público de perguntas | o racional dos autores |

⛔ **NÃO lido, de propósito** (fora do alcance deste adendo): o traçado do layout em profundidade ·
a construção do campo · o solucionador de fluxo por dentro · a geração dos moldes de retalho · toda
a camada de visualização.

⚠️ **O oráculo NÃO foi compilado nem corrido nesta sessão** (ordem do despacho). Todos os números
acima vêm de **leitura de fonte, de configuração e da literatura pública** — ⛔ nenhum é uma
medição de corrida, e nenhum substitui uma medição nossa.
