# SPEC — o PINCEL DE TECIDO e o FILTRO DE TECIDO (clean-room do alvo `blender-cloth`)

```
Alvo: Blender 5.2.0 (tag v5.2.0) — Sculpt Mode, pincel de tecido + filtro de tecido + o alvo de
  deformação «simulação de pano» dos pincéis Pose/Boundary · Licença: GPL-2.0-or-later · Degrau: T2
Ledger: aberto em docs/3D/cleanroom/LEDGER_blender-cloth.md, 2026-09-05
Papel E: subagente-E da janela 1246816c-63cf-414b-842d-663a8baa86ca (2026-09-05). ⛔ Este subagente
  leu o fonte; a janela-mãe NÃO, e é ela a I.
Patente (§8.1): buscado em 2026-09-05 (termos no ledger). Nenhuma patente viva alcança o método.
  Duas CERCAS nomeadas: US 10 586 401 B2 (Pixar — pincéis por soluções ANALÍTICAS de elasticidade
  linear; nunca implementar um modo «elástico» por Kelvinlet) e US 10 713 855 B2 (Audaces, BRASIL —
  vestuário sobre MANEQUIM com escultura + pano + impedir a entrada no manequim; se a casa um dia
  fizer «vestir um corpo» com colisão contra o corpo, refazer a busca com parecer humano).
Filtragem §4.3: executada em 2026-09-05 (E) · Sweep: verde em 2026-09-05 (E), vassoura de 70 entradas
Auditoria §4.2 (R-pré): ✅ auditada contra §4.2 por R-pré em 2026-09-05 — sweep verde sobre espec +
  fixtures + INBOX + READMEs + docs/3D/cloth/02-04 (e sobre o histórico destes caminhos). UM achado de
  expressão (uma frase de comentário do fonte citada como (F) no §7) e CINCO higienes §4.3 (detalhe de
  implementação descrito como comportamento) curados no acto pelo R-pré; os nomes de fixture do §14 e a
  contagem do gate 15 alinhados aos ficheiros. Veredictos e curas, um a um: LEDGER §Papel R.
Mapa de leitura da literatura (⭐ pública e lícita a TODOS os papéis):
  · Jakobsen, "Advanced Character Physics", GDC 2001 — integração de Verlet por posições + relaxação
    de restrições de distância por projecção. É EXACTAMENTE a família do solver do alvo.
  · Müller, Heidelberger, Hennix, Ratcliff, "Position Based Dynamics", VRIPhys 2006 / JVCIR 2007 —
    a restrição de distância (§3.3), a rigidez por iteração (§3.4), o laço (§3.1).
  · Goldenthal et al., "Efficient Simulation of Inextensible Cloth", SIGGRAPH 2007 — contexto sobre
    limitação de esticão (o alvo NÃO a tem; ver §5.7).
  ⛔ Nenhum destes tem apêndice de código do alvo. ⛔ NÃO procurar «PBD cloth brush» em hospedagem de
  código: uma implementação encontrada por esse caminho é, com probabilidade alta, um fork do alvo.
Denylist de URLs (⛔ NÃO abrir): projects.blender.org · developer.blender.org · git.blender.org ·
  github.com/blender/* e qualquer espelho · code-search por «sculpt cloth», «cloth brush solver»,
  «sculpt cloth simulation» · builds/branches de terceiros do Sculpt Mode.
Denylist de CAMINHOS (⛔ os dois estão NESTE disco): ~/Referencias/** (o oráculo, os dumps crus, as
  notas, os rascunhos) · /home/enio/Documentos/Recursos/BlenderSculpt/** (o fonte). ⇒ o Passo 0 do
  BLOCO-I nega `Read` E `Bash` sobre os dois.
"Este documento descreve comportamento; não contém expressão do alvo."
```

> **Como ler.** Cada secção diz **o que o programa faz**, em vocabulário do NOSSO domínio. A
> proveniência de cada número está ao lado dele: **(F)** = lido do fonte como facto de
> comportamento · **(A)** = lido dos *presets* que o artista vê (a biblioteca de pincéis do
> binário) · **(M)** = medido no oráculo (fixture em `fixtures/cloth/`) · **(H)** = história
> (mensagem de commit / blog / issue, com a referência no §9). Um número sem letra não existe aqui.
>
> ⚠️ **As nossas próprias docs anteriores enganam em cinco pontos**, e este documento corrige-os
> no §12 — leia-o antes de reutilizar qualquer frase de [`04`](../cloth/04_espec_do_comportamento.md).

---

## §0 — A leitura em uma linha

**O pincel não é um solver de pano com um pincel em cima: é um solver de RESTRIÇÕES DE DISTÂNCIA
por relaxação (a família Jakobsen/PBD), com integração de Verlet por posições, que corre UM passo
por passo do pincel, sobre um sub-conjunto de células da estrutura espacial da escultura — e o
pincel fala com ele por TRÊS canais só: uma FORÇA por vértice (seis dos oito modos), uma ÂNCORA
por vértice com uma força de restrição própria (Grab · Snake Hook · e todo pincel alheio que
escolha «simulação de pano» como alvo), ou um DESVIO DO COMPRIMENTO DE REPOUSO por vértice
(Expand).** Tudo o resto — a área simulada, a banda graduada, o pino, a plasticidade, a colisão —
é quem decide *quais* restrições existem, *quais* estão activas e *com que peso* cada vértice as
obedece.

⇒ A [auditoria §8-ter](../cloth/03_auditoria_2026-09-05.md) tinha razão no diagnóstico e errava
no remédio: o que separa o nosso pincel do alvo **não é** «escrever alvos por vértice» (o Grab do
alvo faz exactamente isso) — é que (a) **cinco modos são forças e não alvos**, (b) a força entra
numa simulação que **guarda velocidade entre passos** e cujas restrições **estruturais são de
distância entre vizinhos e entre pares de vizinhos** (anel-1 completo), e (c) o solver **nunca
recomeça do zero dentro de um traço**: a posição, a velocidade, o desvio de repouso e as âncoras
sobrevivem de passo a passo.

---

## §1 — Arquitectura de fases (um passo do pincel, por passagem de simetria)

O traço é uma sequência de **passos** (§6.2). A cada passo, e **separadamente para cada passagem
de simetria** (espelho · radial · ladrilho), corre o seguinte, nesta ordem (F):

| # | fase | entra | sai |
|---|---|---|---|
| 0 | **Primeiro passo de uma passagem?** ⇒ com área *Local*, constrói as restrições das células da área (§2, §3) e **termina sem simular** (F) — o alvo precisa de um deslocamento do cursor válido para orientar a ponta, e no 1.º passo ele é zero. ⚠️ Com área *Dynamic*/*Global* o 1.º passo também não simula, e as restrições ficam para o 2.º. | — | células com restrições, ainda **inactivas** |
| 1 | **Garantir restrições** para toda célula do conjunto afectado que ainda não as tenha (§3) | conjunto de células (§2.1) | restrições novas (só para células novas — uma célula é construída UMA vez por traço) |
| 2 | **Guardar o estado**: a posição de simulação de TODOS os vértices ← posições actuais da malha | malha | `x` |
| 3 | **Activar** as células do conjunto afectado | — | células activas |
| 4 | **Aplicar o gesto** (§4): forças → aceleração; ou âncoras + força de âncora; ou desvio de repouso | cursor, delta, normal, factores | `a`, âncoras, desvios |
| 5 | **Passo de simulação** (§5): 5 varreduras de relaxação sobre as restrições das células ACTIVAS; depois, por célula activa: integrar, colidir, **escrever na malha**, e **desactivar a célula** | `x`, `x_prev`, `a` | malha deformada; `x_prev`, `x_colisão` |

⚠️ **Três consequências que decidem o desenho:**
- **A escrita na malha é por passo, não por traço** — o que se vê é sempre o estado da simulação
  (§6.1); não há «preview» separado.
- **Cada passagem de simetria vê o resultado da anterior** (a fase 2 relê a malha), e as restrições
  são partilhadas entre passagens: uma célula tocada por duas passagens tem as restrições das duas.
- **A simulação avança mesmo sem forças**: se o cursor não se mexeu no ecrã, a fase 4 não aplica
  nada, mas a fase 5 corre na mesma (velocidade + restrições) — o pano continua a assentar
  enquanto houver passos (com o método de traço *Dots* dos presets, um passo por evento de rato).

---

## §2 — A área simulada

### §2.1 — De que é feita: CÉLULAS, não vértices

A escultura vive numa árvore espacial de células-folha (a mesma que serve a pintura e o *picking*).
A área simulada é um **conjunto de células**, escolhido a cada passo (F):

| área (rótulo na UI) | conjunto de células | centro `c` da área | raio usado |
|---|---|---|---|
| **Local** *(omissão do código)* | as que intersectam a esfera de centro `c` e raio `R₀·(1+L)` | a **localização inicial** do traço (fixa, por passagem de simetria) — ⚠️ é o ponto da superfície sob o cursor **no último HOVER antes do pen-down** (escrito pelo desenho do cursor, não pelo início do traço: M — um traço scriptado sem hover fica com o centro num ponto velho) | **`R₀` = raio no 1.º passo** (o raio por pressão não muda a área) |
| **Global** | **todas** as células-folha | — | — |
| **Dynamic** *(o que os presets usam — ver §8)* | as que intersectam a esfera de centro `c` e raio `R·(1+L)` | a **localização actual** do cursor | `R` = raio actual (com pressão) |

`L` = *Simulation Limit* (omissão `2,5`, faixa `0,1..10` (F)). O teste célula-esfera é o do ponto
da caixa da célula mais próximo do centro (distância² < raio²), sobre as caixas **actuais** (F).

⭐⭐ **A célula-folha tem no máximo 2 500 FACES (F), logo numa malha pequena há POUCAS células:** a
grelha de 4 225 vértices (~4 096 quads) é ~2 células; a esfera de 6 050 (~6 144 quads) ~3. ⇒ a
activação é **grossa** — activar uma célula que a esfera da área toca traz consigo um pedaço grande
da malha. ⚠️ **O que impede esses vértices longínquos de se moverem é o peso de banda `w` DENTRO do
factor por vértice `φ` (§5.2), não a granularidade da célula:** há DOIS portões, e são diferentes —
o **grosso** (a célula inactiva: as suas restrições são saltadas e os seus vértices não são
integrados) e o **fino** (`w = 0` além do limite ⇒ `φ = 0` ⇒ nem a correcção de restrição nem a
velocidade retida movem o vértice, mesmo que a célula dele esteja activa). No plano, o Local pára
**exactamente** no disco de `3,5 R` por causa do portão fino (M — §10).

⭐⭐ **Local contra Dynamic — o que difere no fonte, além do centro fixo e de `R₀` (F):**
- **o centro de TUDO** (a esfera de células, a banda `w`, a distância da força, a retenção de
  velocidade) é a **localização inicial fixa** no Local e a **do cursor, a cada passo** no Dynamic;
- **a criação de restrições é filtrada por raio** no Local (só vértices com `|p⁰ − c| < R₀·(1+L)`, em
  posições de repouso), e **sem filtro** (todos os vértices das células tocadas) no Dynamic/Global;
- as restrições nascem **de uma vez, no 1.º passo** no Local; **incrementalmente**, à medida que
  novas células entram na área, no Dynamic;
- a banda `w` do Local é avaliada com o centro FIXO, logo o fim de um traço longo cai mais longe do
  centro da força ⇒ menos deslocamento lá; no Dynamic o centro segue o cursor e o traço inteiro
  recebe força cheia. ⇒ para o mesmo traço, o Dynamic desloca mais que o Local, e é ≈ Global. (A
  razão exacta é emergente; a alavanca dominante é o centro fixo vs. móvel.)

⚠️ **A célula é a unidade de activação, e isso tem um efeito visível:** todo vértice de uma célula
activa é integrado (fase 5), mesmo que esteja fora da esfera — o que o segura é o **peso de
banda** (§2.2), que vale `0` fora do limite. E uma restrição pertence à célula onde foi CRIADA
(a do vértice de origem), mesmo que ligue a um vizinho de outra célula: ela só é resolvida quando
**essa** célula está activa.

### §2.2 — A banda graduada (*Simulation Falloff*) — a fórmula

Peso de banda `w(p)` de um ponto `p`, para as áreas *Local* e *Dynamic* (F):

```
limite   = R·(1 + L)
início   = R·(1 + L·F)          F = Simulation Falloff, omissão 0,75, faixa 0..1
d        = |p − c|
w(p) = 1                                 se d < início
     = 0                                 se d > limite
     = 3t² − 2t³,  t = 1 − (d − início)/(limite − início)     na banda   (smoothstep)
```

Para *Global*, `w ≡ 1`. Para qualquer pincel que **não** seja o pincel de tecido mas use a
simulação como alvo (Pose/Boundary), `w ≡ 1` (F — eles não têm área; H: D8885).

⚠️ **Três leitores, DOIS pontos de avaliação diferentes — e isso é load-bearing:**

| quem lê `w` | avaliado em que POSIÇÃO | com que `R` |
|---|---|---|
| a força do gesto (§4) | posição **actual** do vértice | raio actual |
| o factor por vértice das restrições (§5.2) | posição de **REPOUSO do traço** (a de quando a simulação nasceu) | raio actual |
| a retenção de velocidade (§5.4) | posição de **repouso do traço** | raio actual |

⇒ Um vértice que a simulação arrasta para fora da banda **continua** a obedecer às restrições e a
reter velocidade com o peso do sítio de onde partiu; só a força nova o deixa de tocar.

Com os valores de omissão: a banda vai de `1,875·R` a `2,5·R` — largura `0,625·R`.

### §2.3 — O pino opcional (*Pin Simulation Boundary*)

Só existe para a área **Local** (a UI só o mostra aí; o código recusa-o em *Dynamic*) e nasce
**desligado** (F). Ligado, ao construir as restrições de uma célula, todo vértice cuja banda
`w < 1` (avaliada na localização **actual** do cursor e no raio inicial) recebe uma **restrição de
âncora à posição de repouso do traço** com força `1 − w` (§3.2). ⇒ na banda o vértice é puxado de
volta ao sítio com força crescente até `1` no limite; fora do limite (`w = 0`) o pino é total.
(H: D8435 — nasceu porque forças grandes, anchored ou com pinch/grab a força máxima, rompiam a
simulação na fronteira; é opção porque, num pincel que deforma a malha inteira, o pino adiciona
restrições indesejadas.)

### §2.4 — A fronteira TOPOLÓGICA da malha

⛔ **Não é do pincel de tecido.** O que a nota de versão de 2.83 chama «proteger as fronteiras da
malha ao esculpir tecido» é o **auto-mascaramento de arestas de fronteira** da casa (opção
genérica de todo pincel), com o seu número de *passos de propagação*: ele entra no pincel de tecido
como mais um **factor por vértice** (§4.1), em TODOS os sítios onde o mascaramento entra — forças,
restrições e integração. O pincel de tecido não tem nenhum código próprio de fronteira.
A lei dele, lida no módulo de auto-mascaramento (F): com `N` = *passos de propagação*
(omissão `1`), calcula-se para cada vértice a distância `k` em ARESTAS até ao vértice de fronteira
mais próximo (busca em largura, `N` rondas; quem fica a mais de `N` passos não é tocado), e o
multiplicador do vértice é `1 − (1 − k/N)²` — `0` na própria fronteira, `1` a `N` passos, rampa
quadrática entre os dois. Tudo o que este documento chama «auto-máscara» já o inclui, e ele entra
nos três sítios (forças, restrições, integração).

---

## §3 — As restrições

### §3.1 — Estruturais: o padrão de vizinhança

Ao construir uma célula, para cada vértice **visível** `v` da célula que esteja dentro do raio de
construção (*Local*: `|p⁰(v) − c| < R₀·(1+L)`, avaliado em posições de repouso; *Dynamic*/*Global*:
todos) (F):

1. uma restrição de distância `(v, n)` para cada vizinho topológico `n` de `v`;
2. uma restrição de distância `(a, b)` para **cada par ordenado de vizinhos distintos** `a ≠ b` de `v`.

⭐⭐ **O anel-1 é o das ARESTAS DAS FACES POLIGONAIS, não de uma triangulação (confirmado por leitura
do fonte, 2026-09-06):** o vizinho de `v` é, por cada face que o contém, os DOIS cantos adjacentes a
`v` NAQUELA face (o anterior e o seguinte), deduplicados. ⇒ **numa grelha de quads um vértice
interior tem exactamente 4 vizinhos** (N, S, E, O) — ⛔ **não** 6, e **nenhuma diagonal do quad é
vizinha**. Numa malha de triângulos regular são 6. A diagonal só aparece como restrição de PAR
(passo 2): dos 4 vizinhos de um vértice de grelha saem `4·3 = 12` pares ordenados ⇒ 6 não ordenados
= as 2 «diâmetros» N-S / E-O (comprimento `2h`, o papel de dobra) + as 4 diagonais N-E… (`√2·h`,
cisalhamento). O sistema de escultura NÃO triangula a malha para escolher vizinhos.

Cada par não ordenado entra **uma vez** por simulação (conjunto global de arestas já criadas — H:
D8007 curou a duplicação). ⭐ **O que isso produz** (grelha de quads, vértice interior de 4 vizinhos):
as **4** arestas (estrutural) + as **2** «diagonais longas» N-S / E-O (dobra, `2h`) + as **4**
diagonais N-E… (cisalhamento, `√2·h`) — o «4 + 2 + 4» do gate 8. Numa malha de triângulos (6
vizinhos): as 6 arestas + os pares do anel — os 6 pares vizinhos são arestas do anel (já estruturais
de outro vértice) e os restantes atravessam-no (3 «diâmetros» `2h` + 6 cordas `√3·h`). ⇒ **a rigidez
de dobra NÃO tem modelo próprio: é a restrição de distância ao segundo vizinho pelo anel.** Os
autores chamam-lhe, por escrito, «básico» e sabem que repete restrições (H: D6715).

- **Comprimento de repouso:** a distância entre os dois vértices nas **posições de repouso do
  traço** — ou nas posições da **base persistente** se o pincel estiver em modo *Persistent* e
  ela existir (§6.4). Força `1`.
- **Ordem:** as restrições ficam na ordem de criação (célula a célula, vértice a vértice, vizinho
  a vizinho) e são resolvidas **nessa ordem**, sequencialmente (Gauss–Seidel). A ordem de criação
  é determinística dada a ordem das células (F).

### §3.2 — As quatro espécies, e o que cada uma liga

Toda restrição é «distância entre o ponto A e o ponto B tem de valer `ℓ`», com A sempre um
vértice da malha (F). As quatro espécies diferem em **quem é B** e em **`ℓ`**:

| espécie | B | `ℓ` | força `s` | quem cria |
|---|---|---|---|---|
| **estrutural** | outro vértice | distância de repouso | `1` | §3.1 |
| **âncora de deformação** | um ponto por vértice que o gesto escreve (§4.3) | `0` | ver §4.3: `0,1·fade` (Grab radial) · `0,1` (Grab plano) · `0,35` (Snake Hook) · `0,01` (pincel alheio com alvo = simulação) | Grab, Snake Hook, Pose/Boundary |
| **corpo mole** (plasticidade) | um ponto por vértice, «memória de forma», que nasce na posição de repouso do traço | `0` | `1` (o efeito é repartido pela plasticidade, §5.3) | quando *Soft Body Plasticity* `> 0` |
| **pino** | a posição de repouso do traço | `0` | `1 − w` | §2.3 |

⚠️ **Uma restrição de âncora só puxa o vértice** (B não é um vértice; só A se move). E a força de
uma âncora de deformação é ainda **multiplicada, no solver, por um segundo factor por vértice**
(§5.2) que o Grab fixa a `1` (radial) ou ao seu peso de plano, e que o Snake Hook **reescreve a
cada passo** — é *isso* a «força ajustada a cada passo do pincel» da documentação.

### §3.3 — O que a construção NÃO faz

- Não cria restrições para vértices escondidos (F). Vértices **mascarados** entram nas restrições
  como vértices normais — o que os pinta como «pregados» é o factor por vértice `1 − máscara` que
  multiplica toda correcção e todo movimento deles (§5.2, §5.4): máscara `1` ⇒ imóvel.
- Não limita o número de restrições por vértice (F).
- Não separa estrutural de cisalhamento de dobra: é UM tipo com `ℓ` diferente.
- Não reconstrói nunca dentro do traço: uma célula construída é final (F).

---

## §4 — O gesto, modo a modo

### §4.1 — O factor por vértice `f` que multiplica QUALQUER gesto

Para cada vértice de cada célula afectada, o PRODUTO dos factores abaixo (F — só uma ordem importa:
a dureza remapeia a distância ANTES de a curva a ler; e a gravidade entra a meio, §4.2):

```
f = (1 − máscara) · (0 se escondido) · (0 se fora dos planos de recorte da vista)
  · w(posição actual)                                  ← banda (§2.2), com o raio actual
  · (0 se virado para trás e «Front Faces Only»)
  · (0 se distância ≥ R)                                ← o corte duro no raio do pincel
  · curva(distância remapeada pela dureza, R)           ← a curva de falloff do pincel
  · auto-máscara · textura do pincel
  · B                                                  ← a força do pincel (abaixo)
```

- **distância**: para todos os modos excepto Grab, do vértice **actual** ao cursor (esférica; ou no
  plano da vista se a forma do falloff for *Projected*); para o **Grab**, da posição de **repouso do
  traço** ao cursor (o Grab mede tudo na malha de partida). Com **Force Falloff = Plane** a
  distância passa a ser `|distância assinada ao plano|` (§4.4).
- **dureza** `h`: distância `< h·R` ⇒ `0`; senão remapeada linearmente para `[0, R]` (F).
- **curva**: os presets da casa (*Smooth* = `3u²−2u³` com `u = 1 − d/R`; *Sharp* = `u²`; *Root*,
  *Linear*, *Constant*, *Sphere*, *Pow4*, *InvSquare*, *Smoother*, ou a curva desenhada) (F).
- **`B`, a força do pincel** — com `α = força²` (a UI mostra `força`; o alvo eleva ao quadrado para
  dar sensibilidade aos valores baixos), `flip = ±1` (direcção *Add/Subtract* × Ctrl),
  `pressão`, `overlap` (`1` salvo atenuação de espaçamento) e `feather` (simetria) (F):

| modo | `B` |
|---|---|
| Drag · Push · Pinch Point · Pinch Perpendicular · Inflate | `10 · α · flip · pressão · overlap · feather` |
| Grab | `força · feather` (⚠️ sem quadrado, sem pressão: «o mesmo falloff de um Grab normal») |
| Snake Hook | `força · feather · pressão · overlap` |
| Expand | `0,1 · α · flip · pressão · overlap · feather` (H: «Expand é mais sensível à força porque continua a expandir ao passar sobre os mesmos vértices») |

⭐ **Consequência de escala:** a força dos cinco modos de força vale, no centro do pincel e a força
máxima, `10` por passo, e o solver converte-a em `10 · dt / massa = 0,1 / massa` unidades de
DESLOCAMENTO por passo (§5.4) — **um número absoluto, independente do raio e da malha**. O único
modo cuja força cresce com o raio é o Push (§4.2). (F; o oráculo confirma a ordem de grandeza —
§10.)

### §4.2 — Os modos de FORÇA (seis): direcção, referencial, sinal

A força de um vértice é `F = f · u`, com `u` o vector abaixo; ela entra como `a += F / massa` (F).

| modo (rótulo) | `u` — para onde | referencial | o que o *Force Falloff = Plane* muda |
|---|---|---|---|
| **Drag** | ⭐ **a direcção UNITÁRIA do movimento do cursor entre este passo e o anterior** — a MESMA para todos os vértices. ⛔ NÃO é «para o ponto do cursor» | espaço do objecto (o cursor é re-apanhado na superfície a cada passo) | só a forma de `f` (faixa em vez de disco) |
| **Push** | ⭐ **`− n̂_área · 2R · escala`** — para DENTRO, ao longo da **normal da área** do pincel, com magnitude proporcional ao raio (`escala` corrige objectos com escala não-uniforme) | normal da área (média das normais sob o pincel; congelável com *Original Normal*) | idem |
| **Pinch Point** | vector unitário do vértice **para o cursor** (actual) | objecto | ⭐ **muda o alvo**: `u` passa a ser o unitário **para o PLANO** (perpendicular ao plano, com o sinal da distância assinada) |
| **Pinch Perpendicular** | do unitário «vértice → cursor», **só as componentes** ao longo de `x̂ = n̂ × d̂` (perpendicular ao traço, no plano tangente) e de `ẑ = n̂` — a componente ao longo do traço é descartada ⇒ converge para a **LINHA** do traço e para o **plano** dela. Não re-normalizado (magnitude ≤ 1) | referencial local do traço (§4.4) | só a forma de `f` |
| **Inflate** | a **normal do vértice** (actual) | objecto | só a forma de `f` |
| **Gravidade** (todos os modos) | `− ĝ · g`, com `ĝ` a normal +Z do objecto de gravidade (ou +Z do mundo) trazida ao espaço do objecto, `g` = *Gravity* da escultura (omissão **`0`** — M, instalação limpa) | mundo → objecto | — |

⚠️ **A gravidade é aplicada ANTES do corte no raio e da curva**: o seu factor é só
`(1 − máscara) · recorte · w` ⇒ actua na **área simulada inteira**, não no disco do pincel
(H: D8406 — «para a maioria dos modos faz mais sentido aplicar gravidade a toda a simulação»).

⚠️ **Nenhuma força é aplicada num passo em que o deslocamento do cursor no ECRÃ seja zero** (F) —
o teste é sobre o delta de agarrar (§4.3), não sobre a posição 3D.

### §4.3 — Os modos de ÂNCORA (dois) — e o delta de agarrar

**O delta de agarrar `δ`** é um vector em espaço do objecto, derivado do ECRÃ (F): o ponto do
cursor é des-projectado à **profundidade da localização original de agarrar** (a do 1.º passo) e
subtraído ao ponto anterior. Para o **Grab** o delta **acumula** desde o pen-down (ponto actual −
ponto original: um vector total); para os **outros sete modos** é **incremental** (ponto actual −
ponto anterior). Com *Normal Weight* `> 0` (omissão `0`) o delta é inclinado para a normal da
área como no Grab da casa. Com falloff *Projected* é achatado no plano da vista.

⚠️ **A localização do cursor** (`c`, centro do disco de influência):
- Drag/Push/Pinch/Inflate/Expand: **re-apanhada na superfície a cada passo** (raio contra a malha).
- **Grab**: **fica no ponto do pen-down** durante todo o traço (é o que faz o Grab «pegar» num
  conjunto fixo de vértices).
- **Snake Hook**: `c ← c + δ` a cada passo — o centro **anda com o gancho no plano de profundidade
  original**, não é re-apanhado na superfície.

| modo | âncora de deformação do vértice `v` (o ponto B, §3.2) | força da restrição `s` (fixa na criação) | factor por passo `σ_v` (§5.2) |
|---|---|---|---|
| **Grab** | `p⁰(v) + δ · f_v` — a partir da posição de **repouso do traço**, deslocada pelo delta TOTAL pesado | radial: `0,1 · curva(d⁰, R₀)` e **só** para vértices com `d⁰ < R₀` (raio inicial); plano: `0,1` para todos os vértices da célula | radial: `1` · plano: `clamp(f_v, 0, 1)` reescrito a cada passo |
| **Snake Hook** | `x(v) + δ · f_v` — a partir da posição **actual**, deslocada pelo delta INCREMENTAL pesado | `0,35` para todos os vértices da célula | ⭐ **`f_v` reescrito a cada passo, e ZERO para quem não está sob o pincel** |
| pincel alheio com alvo = simulação (Pose, Boundary) | a posição que esse pincel calcularia para o vértice (ele escreve-a na âncora em vez de na malha) | `0,01` para todos | `1` |

⭐⭐ **A lei do Snake Hook, que é a que a documentação diz dar as dobras naturais:** como B é «onde
o vértice está + δ·f» e a correcção é proporcional a `s · σ_v = 0,35 · f_v` (§5.2), o puxão
efectivo por varredura é `≈ 0,3 · 0,35 · f_v² · δ` — **quadrático no falloff**, re-ancorado no
estado actual, e **nulo fora do pincel**: a simulação nunca vê um alvo velho. O Grab, ao contrário,
guarda âncoras «de partida + delta total» para um conjunto fixo, com força fixa por vértice.
(H: D8621 — «muda a força das restrições de deformação por passo para afectar o resultado da
simulação o menos possível».)

⚠️ **O Grab mede o falloff na malha de PARTIDA** (distâncias e recorte sobre as posições de repouso
do traço) — é por isso que o conjunto agarrado não muda quando a malha se mexe.

⚠️ **Nenhum dos dois modos de âncora aplica força.** A aceleração fica a zero; o que move é a
restrição de âncora dentro das 5 varreduras (§5.2).

### §4.4 — O plano de falloff e o referencial local do traço

Quando o modo é Pinch Perpendicular **ou** o *Force Falloff* é *Plane*, o alvo calcula, no passo,
**a normal e o centro da área** sob o pincel (a lei da casa: *Sculpt Plane* — normal da área,
vista, ou eixo; *Original Normal/Plane* congelam) e monta um referencial (F):

```
ẑ = n̂_área      x̂ = n̂_área × δ̂      ŷ = n̂_área × x̂      origem = c
```

O **plano de falloff** passa pelo **centro da área** com normal **`δ̂`** (a direcção do
movimento no ecrã, des-projectada). ⇒ A faixa de influência é perpendicular ao traço, de
meia-espessura `R`, **sem limite ao longo do plano** (só o conjunto de células a limita). O cursor
desenha-o como um segmento com setas nas duas pontas, de comprimento `2R`, ao longo de `x̂`
(para o Grab, transladado pelo delta) (F).

⚠️ Respostas a duas perguntas do [`04` §F](../cloth/04_espec_do_comportamento.md): **(3)** o plano
NÃO é o do pen-down nem o da vista — é o plano do centro da área com a normal = direcção do
movimento, **recalculado a cada passo** (salvo *Original Plane/Normal*); **(12)** a simetria é
expandida **por passagem** e cada passagem corre gesto+solver inteiros (§1).

### §4.5 — Expand — o modo que muda o REPOUSO

`Expand` não aplica força nem âncora: para cada vértice sob o pincel, um **desvio de repouso**
por vértice `τ_v` acumula `τ_v += 0,01 · f_v` a cada passo (F). O comprimento de repouso efectivo
de toda restrição `(a, b)` passa a ser `ℓ + (τ_a + τ_b)/2` (§5.2). Com `flip = −1` contrai.
- `τ` vive na simulação ⇒ **morre no fim do traço** (§6.3), mas a geometria que ele produziu fica.
- A ordem de grandeza: a força máxima, `B = 0,1` ⇒ `τ` cresce `0,001` por passo no centro — e é
  **absoluto** (não relativo à aresta): numa aresta de `0,047` (a grelha do oráculo) são `2 %` por
  passo (M, §10).

---

## §5 — O solver

### §5.1 — A família, em uma frase

Verlet por posições + relaxação sequencial de restrições de distância (Jakobsen 2001; PBD §3),
**com a relaxação ANTES da integração** dentro do mesmo passo, uma rigidez global de `0,6` por
restrição, `5` varreduras, e passo de tempo `0,01` (F). Não há massa por vértice (uma massa
global), não há limitação de esticão, não há dobra própria, não há auto-colisão.

### §5.2 — As 5 varreduras de relaxação

Antes das varreduras, um **factor por vértice** `φ_v = (1 − máscara) · auto-máscara ·
w(p⁰_v)` é pré-calculado para TODOS os vértices (`w` na posição de repouso do traço; sem banda
nas ferramentas sem área) (F).

Para `k = 1..5`, para cada restrição `(A, B, ℓ, s)` **cuja célula esteja activa**, em ordem (F):

```
d  = B − A                     (B: vértice, âncora, memória de forma ou repouso — §3.2)
D  = |d|
ℓ' = ℓ + (τ_A + τ_B)/2         (Expand; τ = 0 sem Expand)
Δ  = 0,6 · d · (1 − ℓ'/D)      (se D = 0: Δ = 0,6·d = 0 — H: D7184 curou a divisão por zero)
h  = Δ/2
σ  = (σ_A + σ_B)/2 se a restrição é uma ÂNCORA DE DEFORMAÇÃO (A = B ⇒ σ = σ_A), senão 1
```

- **estrutural / âncora / pino:** `A += h · φ_A · s · σ`; e, **só se B é outro vértice**,
  `B −= h · φ_B · s · σ`.
- **corpo mole (plasticidade `ρ` = *Soft Body Plasticity* do pincel activo):**
  `A += h · φ_A · s · ρ` e **a memória de forma `B −= h · φ_A · s · (1 − ρ)`** — com `ρ = 0` a
  memória segue o vértice e nunca o puxa; com `ρ = 1` o vértice volta à memória e ela não se
  mexe; entre os dois, a forma «lembra-se» parcialmente do que foi deformado (H: D9187 — a
  1.ª versão pregava à posição ORIGINAL e rompia com gravidade e com o Grab).

⚠️ **Confirmação de fonte (2026-09-06), para o arnês de paridade:**
- **A correcção é `Δ/2`, não `Δ` inteiro**, para TODA espécie (`correction_vector_half = Δ·0,5`). Numa
  restrição estrutural, cada um dos dois vértices leva `Δ/2` ⇒ juntos fecham `Δ`. **Numa âncora, B não
  é vértice e NÃO se move: só A leva `Δ/2`** ⇒ a âncora fecha só metade por varredura (é «mole» de
  propósito, e por isso precisa das 5 varreduras para chegar). *Se um port dá abaixo do oráculo com
  `Δ/2`, o défice está noutro factor (o `σ` por passo do Snake Hook, o `s`, ou a re-ancoragem por
  passo), não em trocar `Δ/2` por `Δ` — o fonte é `Δ/2`.*
- **O `σ` (o factor por passo) multiplica SÓ as âncoras de DEFORMAÇÃO** (Grab, Snake Hook, pincel
  alheio): `deformation_strength = 1` por omissão, e só é reescrito para `(σ_A+σ_B)/2` quando a
  restrição é de deformação. ⛔ O **pino** e o **corpo mole** NÃO o levam (o seu peso é a força `s` e,
  no corpo mole, a plasticidade `ρ`).
- **A força `s` do Grab radial é `0,1 · curva(d⁰)` com a curva do PINCEL** (o preset de falloff do
  pincel activo avaliado na distância de repouso ao centro), não uma curva fixa.

⚠️ **Só a POSIÇÃO é corrigida — não há projecção de velocidade separada**: a velocidade do passo
seguinte sai da diferença de posições (§5.4), logo as correcções das restrições **entram na
velocidade**.

⭐ **E a relaxação vem ANTES da integração, no mesmo passo — logo a resposta das restrições a uma
força chega com UM PASSO de atraso** (F; M — §10, os dumps de um passo): no primeiro passo simulado
a relaxação corre sobre a malha ainda em repouso (correcção zero), depois a integração aplica a
força e escreve na malha; só no passo seguinte as restrições vêem o vértice deslocado e puxam a
vizinhança. ⇒ um dab isolado (dois passos, o 1.º nunca simula) é **força pura sem resposta
elástica**, e o «pano» só existe a partir do 3.º passo do traço.

⚠️ `0,6` era `0,5` até 2020-10-18 (H: D9202 — «reduz artefactos quando restrições de tipos
diferentes disputam o mesmo vértice»). ⚠️ As forças de âncora do Grab eram `1,0` e desciam a
`0,1` em 2020-10-15 (H: D9201 — «instabilidade na zona onde o fade dava 1»; «encontrado
empiricamente, pode precisar de afinação»); a dos pincéis alheios era mais alta e desceu a `0,01`
em 2020-09-14 (H: D8884 — «impedia as dobras de se formarem»).

### §5.3 — O que «damping» é, de facto

Não é um amortecimento de Rayleigh nem uma viscosidade: é **a fracção de velocidade PERDIDA por
passo**, aplicada multiplicativamente à velocidade de Verlet (§5.4). Omissão do código `0,01`
(⇒ `99 %` de retenção); faixa `0,01..1` no pincel, `0..1` no filtro (F). E é **modulado pela
banda**: retenção efectiva `= (1 − damping) · w(p⁰_v)` (H: D9084 — «ajuda a fundir artefactos com
áreas dinâmicas, porque o amortecimento cresce quando o vértice se afasta»). ⇒ a documentação
(«quanto as forças se propagam») é a leitura invertida de «quanta velocidade sobrevive».

### §5.4 — A integração (por célula activa, por vértice, com `φ_v = (1 − máscara) · auto-máscara`)

```
v̄        = x − x_prev                    (x já corrigido pelas 5 varreduras deste passo;
                                          x_prev = o x corrigido do passo ANTERIOR)
x_prev   ← x
x       += a · φ_v · 0,01                 (a = Σ forças / massa; dt = 0,01)
x       += v̄ · φ_v · (1 − damping) · w(p⁰_v)
colisão (§5.6)
x_col    ← x                              (origem do raio de colisão do próximo passo)
a        ← 0
```

⚠️ O termo de aceleração **não** é multiplicado por `w` aqui (a força já o trazia, §4.1) — mas é
pelo `φ_v` de máscara. ⚠️ **A massa é um ganho inverso puro** sobre um `dt` fixo: dobrar a massa
divide exactamente por dois o deslocamento por força num passo (F; M — §10). Faixa `0,01..2`,
omissão `1`.

### §5.5 — Massa, passo de tempo, iterações: os números e o RECURSO que cada um nomeia

| constante | valor | de que recurso é | proveniência |
|---|---|---|---|
| varreduras de relaxação por passo | `5` | tempo por passo × rigidez aparente (mais varreduras = pano mais inextensível) | F |
| rigidez por restrição | `0,6` | estabilidade quando espécies de restrição diferentes disputam um vértice | F · H D9202 |
| passo de tempo | `0,01` | escala do deslocamento por força (`0,1/massa` a força máxima) | F |
| força da âncora Grab | `0,1 × fade` (radial) · `0,1` (plano) | estabilidade na zona de fade = 1 | F · H D9201 |
| força da âncora Snake Hook | `0,35` (× `f_v` por passo) | — | F |
| força da âncora de pincel alheio | `0,01` | «deixar as dobras formarem-se» | F · H D8884 |
| incremento de repouso do Expand | `0,01 · f` por passo | — | F |

### §5.6 — Colisão (opcional, *Use Collisions*, nasce desligada)

Colisores = todos os objectos visíveis da cena **com modificador de colisão** e árvore de
aceleração construída, na pose do quadro (F). Por vértice, depois da integração, para cada
colisor (F):

1. raio, em espaço do MUNDO, de `x_col` (posição pós-integração do passo anterior) até `x`
   (actual), com comprimento `|x − x_col|`; o teste é raio-vs-triângulo (watertight na
   precalculação, mas o cast é feito sem a bandeira watertight) com **espessura de raio `0,3`**
   (uma constante absoluta, em unidades de mundo — F).
2. Se há impacto **dentro do comprimento do raio**: `x ← ponto de impacto + n̂_impacto · 0,005 +
   0,35 · (projecção de x no plano do impacto − ponto de impacto)` — i.e., o vértice pára na
   superfície, é afastado `0,005` pela normal, e **conserva 35 %** do deslizamento tangencial
   que pretendia (fricção `0,65`) (F).
3. Volta ao espaço do objecto.

Limitações **declaradas pelos autores** (H: blog 2020-10): um vértice que já esteja DENTRO do
colisor nunca é expulso (colisão por raio, não por campo de distância); sem auto-colisão, e a
razão nomeada é o tamanho da célula-folha da estrutura espacial. Os autores nomeiam «fricção» e
«distância à superfície» como parâmetros que ficaram por expor (H: D8019). Uma issue aberta
(#96124) reporta que o filtro ignora a espessura exterior do colisor.

### §5.7 — O que o solver NÃO tem (ausências afirmadas por leitura integral)

Sem limitação de esticão · sem massa por vértice · sem modelo de dobra · sem sub-passos (um passo
de solver por passo de pincel) · sem projecção de velocidade · sem auto-colisão · sem detecção
contínua (o raio de colisão é o único CCD, e só contra colisores) · sem re-malhagem (o pincel
recusa topologia dinâmica — F) · sem cache entre traços.

---

## §6 — Como a deformação é cometida, o que sobrevive, undo, simetria

### §6.1 — A escrita na malha

A cada passo, por célula activa: `Δx_v = x_v − posição avaliada actual`; `Δx` passa pelo
**bloqueio de eixos** e pelo **recorte do modificador de espelho** da casa; depois é somado às
posições originais, às avaliadas (quando há modificadores por cima) e às *shape keys*
dependentes, e as caixas da célula são actualizadas (F). ⇒ O que a malha guarda é **sempre** o
estado da simulação — não há passo de «assar».

### §6.2 — Passos, espaçamento, âncora

- Um **passo** = uma chamada do laço do §1. Com o método de traço *Dots* (o dos 13 presets — A),
  é **um passo por evento de movimento**. Com *Space*, o espaçamento é calculado sobre um raio
  fictício de `100 px` e não sobre o raio do pincel, **para o ritmo da simulação não depender do
  raio** (F): passo a cada `2·espaçamento` px (espaçamento `10` ⇒ `20 px`; os presets de Grab usam
  `3` ⇒ `6 px`). A casa mantém o espaçamento ligado para os modos de âncora **de propósito**, para
  que se possa escolher entre «simula sempre» e «simula só quando a mão anda» (F).
- Traço **anchored** / *drag dot*: ao contrário de todos os outros pincéis, a malha **NÃO** é
  reposta a cada passo — a simulação continua do estado anterior (F; H: D8348).
- **Pressão**: muda a força (`pressão` em `B`) e, se *Size Pressure*, o raio actual `R` (que a
  área *Dynamic* usa e a *Local* ignora).

### §6.3 — O que sobrevive

| entre PASSOS do mesmo traço | entre TRAÇOS |
|---|---|
| `x` (relida da malha), `x_prev`, `x_col`, `a` (zerada), as restrições e os estados das células, as âncoras de deformação e os seus `σ`, a memória de forma (corpo mole), os desvios de repouso `τ`, a lista de colisores | ⛔ **nada da simulação** — ela nasce com o traço e morre com ele. A MALHA fica deformada, e o traço seguinte constrói restrições **da malha deformada** (deformação acumula) — salvo *Persistent* (§6.4) |

### §6.4 — Base persistente

O operador *Set Persistent Base* copia as posições (e normais) actuais para um atributo da malha
(F). Com *Persistent* ligado no pincel, **os comprimentos de repouso e o teste de raio da
construção** usam essas posições em vez das actuais ⇒ o mesmo pano de partida pode ser simulado
vezes seguidas com forças diferentes sem acumular (H: D8428). ⚠️ As âncoras do Grab, o pino e a
memória de forma continuam a partir das posições **actuais** do início do traço (F). ⚠️ Em malhas
sem atributos persistentes (multires) a base vive só na sessão de escultura (H: #133267); e
houve um período em que a opção era no-op para o pincel de tecido (H: #134781, 2025-02).

### §6.5 — Undo

Não há nada de especial: o traço inteiro é **um** passo de undo (posições das células tocadas,
registadas na primeira vez que cada célula é tocada no traço) (F). A única particularidade é a
do §6.2 — o pincel de tecido está **excluído** do «repor a malha antes do passo» dos traços
anchored. Uma issue de 2020 sobre undo (#82388) foi do sistema de undo, não do pincel (H).

### §6.6 — Simetria

Espelho, radial e ladrilho são **passagens sequenciais** do laço inteiro (§1), cada uma com
localização, delta, normal e gravidade reflectidos/rodados. Com área *Local*, no 1.º passo, **todas
as passagens constroem as restrições ANTES de qualquer activação** — senão a 2.ª passagem
encontrava a célula já activada pela 1.ª e não lhe acrescentava as suas (H: D9303, T81904).
⚠️ Uma issue **aberta** (#131122) reporta que o Grab de tecido e os pincéis de fronteira com alvo
de simulação não funcionam com simetria — o mecanismo não está descrito nela.

### §6.7 — Multires e topologia dinâmica

Multires: o mesmo laço sobre as grelhas de subdivisão (vizinhos = os da grelha, com costuras) (F).
Topologia dinâmica: o pincel **recusa** (não está na lista dos que a suportam) (F).

---

## §7 — O filtro de tecido

Mesmo solver, sem pincel (F):

| aspecto | o que o filtro faz |
|---|---|
| **área** | todas as células não totalmente mascaradas/escondidas; restrições construídas UMA vez, para TODOS os vértices (raio infinito), ao carregar; sem banda (`w ≡ 1`); sem pino |
| **um passo** | a cada movimento do rato: guardar estado (§1 fase 2) → forças → activar todas → passo de simulação |
| **força escalar `S`** | `S = força_base · (x_rato − x_pressão) · 0,001 · escala_UI` — ⭐ arrastar para a **direita** é positivo, e a magnitude é **pixels** (`0,001` por px a força base `1`); `força_base` é o parâmetro *Strength* (omissão `1`, faixa `−10..10`) |
| **factor por vértice** | `(1 − máscara) · auto-máscara · (0 se fora do face set activo, com *Use Face Sets*) · S` |
| **Gravity** | força `= M · (0, 0, −f)` — ou `(0, −f, 0)` na orientação *View* (na vista, o eixo da gravidade é o −Y do ecrã, para que a queda seja o «baixo» que o artista vê e não a profundidade — F) — com `M` a matriz da orientação (*Local* = identidade · *World* = inversa da matriz do objecto · *View* = inversa da vista × inversa do objecto) |
| **Inflate** | força = normal actual do vértice × f |
| **Expand** | `τ_v += 0,01 · f` (§4.5) |
| **Pinch** | força = unitário do vértice **para o vértice activo no momento em que o filtro começou** (o ponto NÃO segue o rato) × f |
| **Scale** | ⭐ é o único filtro por ÂNCORA: âncora `= p⁰_v + p⁰_v · f` com as componentes dos eixos desligados anuladas (no referencial da orientação) ⇒ escala em torno da **origem do objecto**; força de âncora `0,01` |
| **gravidade da cena** | somada em TODOS os tipos: `ĝ · g · S` com `ĝ` = −Z do objecto de gravidade ou `(0,0,−1)`, `g` = *Gravity* da escultura |
| **Force Axis** | bandeiras X/Y/Z (omissão: as três) — para o Scale anulam componentes da âncora; para as forças, ⚠️ o código lido só as aplica ao Scale (a limitação de eixos das forças passa pela orientação) |
| **Orientation** | *Local* (omissão) · *World* · *View* — define `M` acima **e** a direcção da gravidade do tipo *Gravity* |
| **massa / damping** | omissão `1,0` (faixa `0..2`) / **`0,0`** (faixa `0..1`) — ⚠️ o filtro nasce **sem** perda de velocidade nenhuma |
| **colisões** | idem §5.6, opção nasce desligada |
| undo | um passo por uso do filtro (início ao carregar, fim ao largar) |
| cancelar | há uma issue aberta (#105335): o botão direito não cancela de imediato |

⚠️ O filtro **não** tem os modos Drag/Push/Grab/Snake Hook/Pinch Perpendicular (são gestos de
pincel — precisam de um cursor com direcção).

---

## §8 — Constantes e omissões, com o recurso e a proveniência

### §8.1 — Do CÓDIGO (as omissões de um pincel novo)

| controlo (rótulo) | omissão | faixa | recurso que nomeia |
|---|---|---|---|
| Deformation | Drag | 8 | — |
| Force Falloff | Radial | Radial · Plane | — |
| Simulation Area | **Local** | Local · Global · Dynamic | tempo (Global simula tudo) |
| Simulation Limit `L` | `2,5` | `0,1..10` | tempo × alcance |
| Simulation Falloff `F` | `0,75` | `0..1` | — |
| Pin Simulation Boundary | **off** | bool | estabilidade vs. liberdade (§2.3) |
| Cloth Mass | `1,0` | `0,01..2` | ganho inverso (§5.4) |
| Cloth Damping | `0,01` | `0,01..1` | retenção de velocidade (§5.3) |
| Soft Body Plasticity | `0,0` | `0..1` | — |
| Use Collisions | off | bool | tempo |
| Persistent | off | bool | — |
| Normal Weight (só Grab/Snake Hook) | `0` | `0..1` | — |
| Gravity (cena) | `0` (M) | — | — |

### §8.2 — Dos PRESETS que o artista vê (A — os 13 pincéis de tecido da biblioteca do binário)

⭐ **Os presets contradizem as omissões do código em quatro colunas, e são eles que o artista
recebe.** Nomes = os rótulos dos presets; tipo = o modo de deformação (ou o pincel alheio com alvo
= simulação):

| preset | modo | área | `L` | damping | plasticidade | força | espaçamento | curva | pressão→força | colisões |
|---|---|---|---|---|---|---|---|---|---|---|
| Drag Cloth | Drag | **Dynamic** | 2,5 | 0,01 | **0,5** | 0,6 | 10 | Smooth | sim | **on** |
| Push Cloth | Push | Dynamic | 2,5 | 0,01 | 0,3 | 0,85 | 10 | **Sharp** | sim | on |
| Pinch Point Cloth | Pinch Point | Dynamic | **3,5** | 0,01 | 0,4 | 0,3 | 10 | Smooth | sim | on |
| Pinch Folds Cloth | Pinch Perpendicular | Dynamic | 2,5 | 0,01 | 0,4 | 0,5 | 10 | Smooth | sim | on |
| Inflate Cloth | Inflate | Dynamic | 2,5 | **0,4** | 0 | 0,3 | 10 | Smooth | sim | on |
| Expand/Contract Cloth | Expand | Dynamic | 2,5 | **0,4** | 0 | 0,6 | 10 | Smooth | sim | on |
| Grab Cloth | Grab | **Local** | **5,0** | **0,6** | 0 | 1,0 | 3 | Smooth | não | on |
| Grab Random Cloth | Grab (com jitter) | Local | 5,0 | 0,6 | 0 | 1,0 | 3 | Smooth | não | on |
| Grab Planar Cloth | Grab, **Force Falloff = Plane** | Dynamic | 2,5 | 0,01 | 0 | 0,5 | 3 | Smooth | não | on |
| Stretch/Move Cloth | pincel **Pose** (escala/translação) com alvo = simulação | — | — | 0 (o pincel alheio cria a simulação com damping `0`) | 0 | 0,5 | 10 | Smooth | não | — |
| Bend/Twist Cloth | pincel **Pose** (rotação/torção) com alvo = simulação | — | — | 0 | 0 | 0,5 | 10 | Smooth | não | — |
| Bend Boundary Cloth | pincel **Boundary** (dobrar) com alvo = simulação | — | — | 0 | 0 | 0,5 | 3 | Sharp | não | — |
| Twist Boundary Cloth | pincel **Boundary** (torcer) com alvo = simulação | — | — | 0 | 0 | 0,5 | 3 | Sharp | não | — |

Todos: falloff `F = 0,75`, massa `1`, pino **off**, *Front Faces Only* off, raio em pixels `70`
(não travado à cena), método de traço **Dots**. ⚠️ **Não há preset de Snake Hook** — o modo só
existe no menu. ⚠️ Os pincéis Pose/Boundary com alvo = simulação criam a simulação com **massa 1,
damping 0, sem plasticidade, sem colisão** e força de âncora `0,01`, e o passo de simulação corre
**só na passagem principal de simetria, depois de todas as passagens terem escrito as âncoras** (F).

### §8.3 — O cursor (o que o artista vê da área)

Círculo **tracejado** em `R·(1 + L·F)` (alfa `0,5×`) e círculo **contínuo** em `R·(1+L)` (alfa
`0,7×`), sempre que a área não é *Global* (F). Durante um traço *Local*/radial, se o cursor sair da
área, os dois círculos são redesenhados **a vermelho, na localização inicial** (F). Com *Force
Falloff = Plane*, em vez dos círculos: o segmento com setas do §4.4 (F).

### §8.4 — O painel (ordem exacta, F)

Simulation Area · [se ≠ Global] Simulation Limit · Simulation Falloff · [se Local] Pin Simulation
Boundary · Deformation · Force Falloff · Cloth Mass · Cloth Damping · Soft Body Plasticity · Use
Collisions — mais os genéricos da casa (Strength, Radius, Persistent + Set Persistent Base, Front
Faces Only, Normal Weight nos modos de âncora, curva, dureza, textura, auto-mascaramento).

---

## §9 — A sabedoria dos autores, re-dita, com proveniência

⛔ **Os links abaixo são para páginas do repositório do alvo — o I NÃO os abre.** Estão aqui
como proveniência (as mensagens de commit são públicas; o texto foi re-dito). Referências por
`D<n>` (revisão), `T<n>`/`#<n>` (issue), data.

| # | o que os autores aprenderam | fonte |
|---|---|---|
| 1 | A construção das restrições é «extremamente básica», repete restrições, e podia ser mais barata e multi-thread — eles sabem, e deixaram assim porque «funciona ok». | D6715, 2020-02-28 |
| 2 | O Grab original acumulava o delta a partir da posição do passo anterior e usava força errada; foi refeito para **escrever posições** a partir das originais, com a simulação a resolver o resto. Os autores **tentaram e rejeitaram** um Grab por forças — «isto é mais controlável, e o falloff do Grab afina-se pelo falloff da simulação». | D7756, 2020-05-19 |
| 3 | Dois vértices coincidentes davam divisão por zero na correcção — daí o teste `D = 0` do §5.2. | D7184 / T74808, 2020-03 |
| 4 | Gravidade só dentro do raio do pincel «não faz sentido» — passou a actuar em toda a área simulada, sem escalar pelo raio. | D8406, 2020-07-28 |
| 5 | O «corpo mole» nasceu como pino à posição ORIGINAL com força regulável; **rompia** com gravidade e com o Grab; foi substituído pela **plasticidade** com memória de forma deformável — «muito melhor e mais previsível», e «aumenta a estabilidade». | D7845 → D9187, 2020-07/10 |
| 6 | O pino da fronteira existe porque forças grandes (anchored, pinch, grab a força total) **rompiam** a simulação na fronteira; com ele «a simulação não se parte seja qual for a força»; é opção porque estraga pincéis que deformam a malha inteira. | D8435, 2020-07-31 |
| 7 | Área *Global* nasceu porque as pessoas punham `L = 10` para simular tudo — «um hack, porque os limites escalam com o raio». | D8481, 2020-08-06 |
| 8 | A força das âncoras dos pincéis alheios era alta demais e «impedia as dobras de se formarem» — desceu a `0,01`, com a intenção (não cumprida) de a expor por pincel. | D8884, 2020-09-14 |
| 9 | Pincéis alheios com alvo = simulação **não têm área nem banda** — afectam a malha inteira. | D8885, 2020-09-17 |
| 10 | Área *Dynamic*: activa células e constrói restrições **durante** o traço; «sem restrições de comprimento do traço, área, nem número de vértices»; as células fora da área não são resolvidas nem colididas. | D8726, 2020-10-01 |
| 11 | O damping passou a ser modulado pela banda «porque ajuda a fundir artefactos com áreas dinâmicas». | D9084, 2020-10-01 |
| 12 | As âncoras do Grab a força `1` na zona de fade `= 1` davam instabilidade — força escalada por um factor **«encontrado empiricamente»**, que «pode precisar de afinação». | D9201, 2020-10-15 |
| 13 | A rigidez por restrição passou de `0,5` a `0,6`·(½) para reduzir artefactos quando várias espécies de restrição disputam um vértice. | D9202, 2020-10-18 |
| 14 | O falloff de plano existia desde o início mas dava «imensos artefactos» nos modos de âncora por falta de funcionalidades no solver; o Grab usou radial à força até o plano ser reimplementado **pelas âncoras** (com a força reescrita a cada passo). | D9320, 2020-10-22 |
| 15 | Com espelho e área *Local*, a 2.ª passagem encontrava a célula já activada e não acrescentava as suas restrições — construir tudo **antes** de activar. | D9303 / T81904, 2020-10-23 |
| 16 | Uma regressão de desempenho: o ramo *Local* corria também o código *Global* por um `if` que devia ser `else if`. | D9762 / T83201, 2020-12 |
| 17 | Snake Hook: «muda a força das restrições de deformação por passo para afectar o resultado da simulação o menos possível»; «agarra o pano sem produzir artefactos na superfície e cria dobras mais naturais do que qualquer outro modo». | D8621, 2020-08-24 |
| 18 | Colisões por raio: colidem com **qualquer** geometria (mesmo não-manifold) e o vértice pára na superfície; a desvantagem nomeada: um vértice dentro do colisor nunca sai; o plano era colisão por SDF; auto-colisão bloqueada pelo tamanho da célula-folha. | blog 2020-10-20 |
| 19 | O pincel foi posicionado, pelo autor, como «substituto de pincéis com alfas de tecido, quando só se quer FINGIR detalhe de pano em partes da malha»; «um solver de pano a sério que corre a simulação completa dá resultados mais exactos». | blog 2020-02-25 |
| 20 | Regressão de 2024 no Pinch: uma multiplicação foi trocada por uma subtracção num refactor — o comportamento «de antes» é a multiplicação (a do §4.2). | #127836, 2024-09-19 |
| 21 | Com falloff *Constant* o corte no raio tem de ser explícito (a curva constante não corta) — daí o corte duro do §4.1. | #139846, 2025-06-06 |
| 22 | O Grab com falloff de plano estoirava ao clicar fora da malha (o cache do traço passou a existir antes de haver superfície) — a cerca é «não desenhar/agir sem superfície sob o cursor». | #161820, 2026-07-23 |
| 23 | **Abertos**: artefactos dos pincéis de tecido (#138844) · distorção grande em malhas de densidade irregular (#131510 — ⚠️ consistente com forças ABSOLUTAS e restrições ao anel-1, §4.1/§3.1) · Grab de tecido e Boundary com alvo = simulação não funcionam com simetria (#131122) · o Pinch do filtro numa superfície plana (#132316) · o filtro ignora a espessura exterior do colisor (#96124) · o botão direito não cancela o filtro de imediato (#105335). | tracker, 2024–2026 |

---

## §10 — Vectores de teste (o oráculo)

⭐ **47 traços do binário 5.2.1 sobre malhas NOSSAS** (grelha plana 64×64, esfera UV 96×64), um por
modo e por variante de solver, em `fixtures/cloth/` (proveniência e verificador no README de lá).
Colunas: **movidos** = vértices com `|u| > 1e-5` · **máx `|u|`** em unidades de objecto · **alcance/R**
= distância máxima de um vértice movido ao caminho, sobre o raio · **fracção normal** = `Σ|u·n⁰|/Σ|u|`
(quanto levanta contra quanto desliza) · **coerência** = módulo do vector unitário médio dos
deslocamentos grandes (`1` = uma direcção só; `0` = radial) · **Δárea** = variação da área da grelha.

| fixture | modo | falloff | área | passos | movidos | máx `|u|` | alcance/R | fracção normal | coerência | Δárea |
|---|---|---|---|---|---|---|---|---|---|---|
| `plano_arrastar_plano_local` | Drag | plane | local | 12 | 2146 | `0.8996` | `3.49` | `0.00` | `0.99` | +15.41 % |
| `plano_arrastar_radial_dinamica` | Drag | radial | dynamic | 12 | 2508 | `0.6128` | `3.49` | `0.00` | `1.00` | +3.21 % |
| `plano_arrastar_radial_dinamica_preset` | Drag | radial | dynamic | 12 | 2455 | `0.3296` | `3.45` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_global` | Drag | radial | global | 12 | 4225 | `0.6446` | `5.49` | `0.00` | `1.00` | +1.17 % |
| `plano_arrastar_radial_local` | Drag | radial | local | 12 | 2144 | `0.3316` | `3.49` | `0.00` | `1.00` | -0.07 % |
| `plano_arrastar_radial_local_1passo` | Drag | radial | local | 2 | 171 | `0.0992` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_2passos` | Drag | radial | local | 3 | 1438 | `0.1359` | `3.25` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_amort1` | Drag | radial | local | 12 | 2141 | `0.2199` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_arrastar_radial_local_amort05` | Drag | radial | local | 12 | 2142 | `0.2544` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_massa2` | Drag | radial | local | 12 | 2143 | `0.1546` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_massa2_1passo` | Drag | radial | local | 2 | 171 | `0.0496` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_arrastar_radial_local_pino` | Drag | radial | local | 12 | 2144 | `0.3235` | `3.49` | `0.00` | `1.00` | -0.03 % |
| `plano_arrastar_radial_local_plast05` | Drag | radial | local | 12 | 2141 | `0.2343` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_arrastar_radial_local_forca05` | Drag | radial | local | 12 | 2139 | `0.0733` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_arrastar_radial_local_forca05_1passo` | Drag | radial | local | 2 | 168 | `0.0248` | `0.99` | `0.00` | `1.00` | +0.00 % |
| `plano_expandir_radial_local` | Expand | radial | local | 12 | 2134 | `0.0115` | `3.49` | `0.00` | `0.35` | +0.00 % |
| `plano_expandir_radial_local_1passo` | Expand | radial | local | 2 | 848 | `0.0019` | `2.94` | `0.00` | `0.10` | +0.00 % |
| `plano_agarrar_plano_local` | Grab | plane | local | 12 | 2146 | `0.3076` | `3.49` | `0.00` | `1.00` | +1.24 % |
| `plano_agarrar_radial_local` | Grab | radial | local | 12 | 2139 | `0.1699` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_agarrar_radial_local_1passo` | Grab | radial | local | 2 | 1324 | `0.1341` | `3.20` | `0.00` | `1.00` | +0.00 % |
| `plano_agarrar_radial_local_24passos` | Grab | radial | local | 24 | 2142 | `0.1585` | `3.49` | `0.00` | `1.00` | -0.05 % |
| `plano_agarrar_radial_local_2passos` | Grab | radial | local | 3 | 1872 | `0.1461` | `3.41` | `0.00` | `1.00` | -0.00 % |
| `plano_agarrar_radial_local_amort06` | Grab | radial | local | 12 | 2131 | `0.1315` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `plano_agarrar_radial_local_preset` | Grab | radial | local | 12 | 4123 | `0.1326` | `5.49` | `0.00` | `1.00` | -0.14 % |
| `plano_inflar_radial_local` | Inflate | radial | local | 12 | 2146 | `0.3172` | `3.49` | `0.95` | `1.00` | +3.02 % |
| `plano_inflar_radial_local_1passo` | Inflate | radial | local | 2 | 171 | `0.0992` | `0.99` | `1.00` | `1.00` | +0.20 % |
| `plano_apertar_linha_radial_local` | Pinch Perpendicular | radial | local | 12 | 2135 | `0.1005` | `3.49` | `0.00` | `0.03` | +0.11 % |
| `plano_apertar_linha_radial_local_1passo` | Pinch Perpendicular | radial | local | 2 | 156 | `0.0876` | `0.99` | `0.00` | `0.00` | +0.16 % |
| `plano_apertar_ponto_plano_local` | Pinch Point | plane | local | 12 | 2146 | `0.6239` | `3.49` | `0.00` | `1.00` | +33.42 % |
| `plano_apertar_ponto_radial_local` | Pinch Point | radial | local | 12 | 2146 | `0.3258` | `3.49` | `0.00` | `0.77` | +5.90 % |
| `plano_apertar_ponto_radial_local_1passo` | Pinch Point | radial | local | 2 | 171 | `0.0992` | `0.99` | `0.00` | `0.02` | +0.19 % |
| `plano_empurrar_plano_local` | Push | plane | local | 12 | 2146 | `0.5201` | `3.49` | `0.98` | `0.99` | +8.25 % |
| `plano_empurrar_radial_local` | Push | radial | local | 12 | 2145 | `0.2590` | `3.49` | `0.94` | `1.00` | +2.08 % |
| `plano_empurrar_radial_local_1passo` | Push | radial | local | 2 | 171 | `0.0694` | `0.99` | `1.00` | `1.00` | +0.10 % |
| `plano_gancho_radial_local` | Snake Hook | radial | local | 12 | 2140 | `0.0915` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_gancho_radial_local_1passo` | Snake Hook | radial | local | 2 | 1452 | `0.4894` | `3.37` | `0.00` | `0.99` | +2.26 % |
| `plano_gancho_radial_local_24passos` | Snake Hook | radial | local | 24 | 2142 | `0.0293` | `3.49` | `0.00` | `1.00` | -0.02 % |
| `plano_gancho_radial_local_2passos` | Snake Hook | radial | local | 3 | 1950 | `0.3648` | `3.43` | `0.00` | `0.99` | +1.07 % |
| `plano_gancho_radial_local_amort06` | Snake Hook | radial | local | 12 | 2135 | `0.0634` | `3.49` | `0.00` | `1.00` | -0.01 % |
| `esfera_arrastar_radial_dinamica` | Drag | radial | dynamic | 12 | 2183 | `0.5828` | `3.48` | `0.59` | `0.96` | — |
| `esfera_expandir_radial_dinamica` | Expand | radial | dynamic | 12 | 2096 | `0.0467` | `3.44` | `0.83` | `0.98` | — |
| `esfera_agarrar_radial_dinamica` | Grab | radial | dynamic | 12 | 1863 | `0.2365` | `3.49` | `0.65` | `0.99` | — |
| `esfera_inflar_radial_dinamica` | Inflate | radial | dynamic | 12 | 2181 | `0.2670` | `3.48` | `0.70` | `0.91` | — |
| `esfera_apertar_linha_radial_dinamica` | Pinch Perpendicular | radial | dynamic | 12 | 2162 | `0.2497` | `3.47` | `0.59` | `0.21` | — |
| `esfera_apertar_ponto_radial_dinamica` | Pinch Point | radial | dynamic | 12 | 2183 | `0.4639` | `3.47` | `0.61` | `0.76` | — |
| `esfera_empurrar_radial_dinamica` | Push | radial | dynamic | 12 | 2102 | `0.4794` | `3.47` | `0.93` | `1.00` | — |
| `esfera_gancho_radial_dinamica` | Snake Hook | radial | dynamic | 12 | 2234 | `0.1690` | `3.52` | `0.62` | `0.88` | — |

### §10.1 — O que os dumps confirmam (cada número casa com a secção citada)

- **A força é absoluta e a massa é ganho inverso (§4.1, §5.4):** um passo de Drag a força `1` desloca o vértice sob o cursor `0.09917` (previsto `0,1·f`, com `f = 0,99` no vértice mais próximo do cursor ⇒ `0,099`); com massa `2`: `0.04958` (razão `0.5000`, prevista `0,5`); com força `0,5`: `0.02479` (razão `0.2500`, prevista `0,25` — a força entra ao QUADRADO). Inflate e Pinch Point dão o mesmo módulo no 1.º passo (`0.09917` · `0.09917`); **Push dá `0.06942`** = `2R·0,099` com `R = 0,35` (razão Push/Drag `0.700`, prevista `2R = 0,70`).
- **A relaxação atrasa um passo (§5.2):** no dab de um passo só os `171` vértices dentro do raio se movem (`alcance/R = 0.99`) e o Expand de um passo move **`848`** vértices; com dois passos simulados o Drag já arrasta `1438` vértices até `3.25 R`.
- **Direcção (§4.2):** coerência (módulo do vector unitário médio dos deslocamentos grandes) — Drag `1.00` (uma direcção só), Pinch Point `0.02` (radial ⇒ soma nula), Pinch Perpendicular `0.00`; fracção normal Inflate `1.00` (para cima), Push `1.00` (para baixo: `u_n` mínimo `-0.0694`), Drag `0.00` (no plano).
- **Velocidade retida (§5.3):** Drag 12 passos, `máx|u|` = `0.3316` com damping `0,01`, `0.2544` com `0,5`, `0.2199` com `1,0`; com o falloff de PLANO a faixa inteira acumula velocidade e passa do percurso do cursor (`0.8996` para um percurso de `0,6`).
- **Área (§2.1):** Local — alcance `3.49 R` (a esfera de células `R₀(1+L) = 3,5 R` a partir do pen-down); Dynamic — `3.49 R` a partir do caminho, `máx` `0.6128` (segue o cursor); Global — todos os `4225` vértices.
- **Âncoras (§4.3):** Grab radial 12 passos `máx` `0.1699` e 24 passos `0.1585` (mesmo percurso: a resposta depende do percurso, não do número de passos); Grab de plano `0.3076`; Snake Hook `0.0915` (12) e `0.0293` (24 — metade do delta por passo ⇒ ~metade da resposta: lei quadrática no falloff); com damping `0,6` (o dos presets de Grab): Grab `0.1315`, e com `L = 5` `0.1326`.
- **Expand (§4.5):** 12 passos, `máx|u|` `0.0115` (`0.25` arestas) e Δárea `+0.002 %`.
(em falta: 'sphere_drag_radial_local')

⚠️⚠️ **As fixtures de ESFERA são todas de área DINÂMICA (ERRATA de 2026-09-06).** A 1.ª entrega
gravou-as como *Local*, mas um traço scriptado **não dispara o hover** que fixa o centro da área
Local — esse centro fica na ORIGEM do objecto (o valor obsoleto de `initial_location`). Numa esfera
unitária a origem põe **toda** a malha dentro da banda (todo vértice a `1,0` < início da banda
`1,006`), então a saída lida `6 050 / 6 050` movidos, uniformes — a esfera a deslocar-se como um
CORPO, que é **artefacto do arnês, não comportamento Local do alvo**. ⭐ **O R₀ estava certo (`0,35`,
tamanho travado à cena); o defeito era o CENTRO.** A área Dinâmica, cujo centro é o cursor de cada
passo (que o traço scriptado FORNECE), funciona: as 8 fixtures de esfera param no bordo da banda
(`alcance ≈ 3,5 R`, `0` vértices além), e é sobre elas que se lê o relevo fora do plano numa
superfície curva. A área **Local** fica medida **só no plano** (onde a origem cai na superfície e o
disco de `3,5 R` é exacto). Mecanismo e a errata completa: [ledger](LEDGER_blender-cloth.md).

---

## §11 — Comportamento de borda, caso a caso (F salvo indicação)

| caso | o que o alvo faz |
|---|---|
| dois vértices coincidentes numa restrição | correcção zero (sem divisão por zero) |
| vértice escondido | sem restrições, factor `0` |
| vértice totalmente mascarado | tem restrições, mas `φ = 0` ⇒ nunca se move — os vizinhos vêem-no como âncora rígida |
| célula sem vértices visíveis | construída vazia; activada/desactivada sem efeito |
| cursor parado (delta de ecrã zero) | sem forças; o passo de simulação corre na mesma |
| cursor fora da malha (Drag/Push/…) | sem localização ⇒ o passo não acontece (o traço não «apanha»); para Grab/Snake Hook a localização é do pen-down e o traço continua fora da malha |
| 1.º passo de cada passagem de simetria | nunca simula |
| 2.º passo (o 1.º simulado) | força pura: as restrições ainda não respondem (§5.2); nos modos de âncora, ao contrário, a âncora É resolvida nesse passo (o alvo é escrito antes da relaxação) |
| raio por pressão a variar | *Local* ignora (usa `R₀`); *Dynamic* usa `R` a cada passo; o factor do gesto usa `R` sempre |
| *Local* e o cursor sai da área | a área NÃO segue; o cursor pinta os limites a vermelho na localização inicial |
| área *Dynamic* + pino | o pino é recusado (a opção não se aplica) |
| área *Global* + pino | sem efeito (`w ≡ 1` ⇒ ninguém está na banda) |
| *Persistent* sem base definida | comporta-se como sem *Persistent* |
| traço anchored | a malha não é reposta por passo; a simulação continua |
| topologia dinâmica activa | o pincel recusa |
| multires | funciona sobre a grelha do nível activo |
| vértice dentro de um colisor | nunca é expulso (H) |
| massa `0,01` (o mínimo) | deslocamento por força `100×` o de massa `1` (`10/passo` a força máxima) — a UI impede `0` |
| damping `1` | velocidade zerada a cada passo (só forças e restrições) |
| plasticidade `1` | o vértice é sempre puxado à memória de forma inicial (o pano «recupera») |
| filtro com força negativa | inverte todas as forças e o Scale encolhe |
| filtro com *Use Face Sets* | vértices fora do face set activo: factor `0` |

---

## §12 — ⛔ As cinco coisas que a nossa doc pública [`04`](../cloth/04_espec_do_comportamento.md) inferiu ao contrário

1. **«Drag puxa o pano PARA o cursor (campo radial)»** — ⛔ não. O Drag aplica a **mesma direcção
   unitária a todos os vértices**: a direcção do movimento do cursor entre passos (§4.2). Quem é
   radial é o **Pinch Point**. *A translação uniforme pesada pelo falloff era o Drag — só que como
   FORÇA numa simulação com velocidade, e não como posição.*
2. **«Push afasta o pano do cursor»** — ⛔ não. O Push empurra **para dentro, ao longo da normal da
   área**, com magnitude `2R` (§4.2). É o único modo cuja força escala com o raio.
3. **«Damping é acoplamento/propagação»** — ⛔ é **perda de velocidade por passo**, modulada pela
   banda (§5.3). A frase da documentação descreve o EFEITO visto de fora (com retenção alta, o
   movimento propaga-se mais longe).
4. **«A fronteira topológica tem rampa por passos no pincel de tecido»** — é o **auto-mascaramento
   de fronteira** da casa, genérico (§2.4); o pincel não tem código de fronteira.
5. **«O Grab de tecido é `Deformation Target = Cloth Simulation`»** — o controlo *Deformation
   Target* é de **outros** pincéis (Pose, Boundary…); o pincel de tecido não o tem. O Grab de
   tecido é um **modo** do pincel, e o mecanismo comum é a **âncora com restrição** (§4.3). A
   frase «o pincel deforma as restrições» é verdadeira para 2 dos 8 modos; os outros 6 são
   forças ou repouso.

E uma correcção à [auditoria](../cloth/03_auditoria_2026-09-05.md) §8-ter: «escrever a posição
alvo de cada vértice é o oposto do caminho da dobra natural» — o Grab do alvo escreve exactamente
isso e é o modo que os presets mais usam (3 de 13); a diferença está em **como a restrição da
âncora é resolvida** (0,3·0,1·fade por varredura, contra as restrições estruturais a 0,3), e em o
Snake Hook **re-ancorar** no estado actual com força quadrática no falloff.

---

## §13 — Notas de custo

- Por passo: `O(#restrições activas × 5)` na relaxação + `O(#vértices das células activas)` na
  integração + colisão `O(#vértices activos × #colisores × log)`. A construção de uma célula é
  `O(Σ_v grau(v)²)` (pares de vizinhos) — numa malha de triângulos regular, `~21` restrições
  candidatas por vértice, `~10` novas após dedup.
- A área *Local* com `L = 2,5` simula um disco de `3,5R`; os autores dizem que se mantém em tempo
  real «enquanto o raio não for grande demais» (H: D6715). A área *Dynamic* limita o trabalho ao
  disco actual e é o que os presets usam.
- A relaxação é sequencial (Gauss–Seidel) — o determinismo vem da ordem de células e de
  vértices.

---

## §14 — Gates propostos (a barra é DERIVADA — CLAUDE.md §0.0)

| # | gate | barra | de onde |
|---|---|---|---|
| 1 | **Drag é uma direcção uniforme**: num retalho plano, um passo de Drag dá deslocamentos cuja direcção, nos vértices com `f > 0,5`, é a do movimento do cursor a `< 1°` de desvio ANTES das restrições | `1°` (é um `normalize` exacto; a tolerância é o `f32`) | §4.2 · fixture `plano_arrastar_radial_local_1passo` |
| 2 | **Pinch Point é radial**: idem, direcção = do vértice para o cursor | idem | §4.2 · fixture `plano_apertar_ponto_radial_local_1passo` |
| 3 | **Inflate levanta**: componente normal do deslocamento `> 0` em todo vértice com `f > 0` | exacta | §4.2 · fixture `plano_inflar_radial_local_1passo` |
| 4 | **Push entra**: componente normal `< 0`, magnitude `∝ 2R` (dois raios ⇒ razão `2 ± f32`) | derivada do `2R` | §4.2 |
| 5 | **Expand muda o repouso**: a área do retalho cresce; e a força é `0,1×` a dos outros modos | `Σ área > área₀` · razão `10 ± 1 %` | §4.5 · fixture `plano_expandir_radial_local` |
| 6 | **A força é absoluta**: o deslocamento no 1.º passo simulado é `0,1 · α / massa` no centro, independente de `R` e da aresta — massa `2` ⇒ metade exacta | `f32` (a conta é `a·dt`) | §4.1/§5.4 · fixtures `plano_arrastar_radial_local_1passo`, `plano_arrastar_radial_local_massa2_1passo` |
| 7 | **A banda é smoothstep** em `[R(1+LF), R(1+L)]`, nas três leituras, e `w ≡ 1` em *Global* | exacta (polinómio) | §2.2 |
| 8 | **O padrão de restrições** numa grelha regular: 4 + 2 + 4 por vértice interior, sem duplicados | contagem exacta | §3.1 |
| 9 | **5 varreduras, `0,6`, meio para cada lado**: numa restrição isolada entre dois vértices livres com `φ = 1`, o erro após `k` varreduras é `(1 − 0,6)^k` do inicial | `0,4^5 = 0,01024 ± f32` | §5.2 |
| 10 | **Damping é retenção**: com damping `d`, um vértice livre em voo perde `d` da velocidade por passo; com `d = 1` pára | `f32` | §5.3 · fixture `plano_arrastar_radial_local_amort05` |
| 11 | **O 1.º passo nunca simula**; o passo seguinte sim | exacta | §1 |
| 12 | **Snake Hook zera as forças de âncora fora do pincel a cada passo**; o Grab não | exacta | §4.3 |
| 13 | **Grab mede na malha de partida**: mover o pano não muda o conjunto agarrado | exacta | §4.3 |
| 14 | **A simetria é por passagem** e a 2.ª passagem vê a 1.ª | fixture com espelho | §6.6 |
| 15 | **Paridade com o oráculo** — os 47 traços (§10): a barra é a **discretização** e o **`f32`**: a nossa malha é a mesma (gerada pela mesma lei), logo a comparação é por vértice; a barra por vértice é a aresta × a diferença de ordem das restrições (Gauss–Seidel não comuta) — MEDIR primeiro a dispersão entre duas ordens nossas e usar essa dispersão como barra (⛔ não um epsilon de conforto; ⛔ não bit-parity — ADR-0162) | derivada por medição | §10 |

---
