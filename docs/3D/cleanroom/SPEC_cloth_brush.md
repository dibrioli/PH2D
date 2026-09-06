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
  ✅ EMENDAS Q8, Q9 e Q10 de 2026-09-06 — Q10 (§10.5 NOVA): dois traços de APERTO por passo, 12 passos,
  prova do fatiamento `0,000000`. Q9 (§4.3 · §10.4 NOVA · §14 gates 12 e 18): o centro da queda do
  Snake Hook está UM PASSO atrasado, e a força por passo das âncoras é zerada nos DOIS modos de âncora.
  Q8 (§1 fases 0/1 · §2.1 · §3.1 · §3.3 · §5.2-bis NOVA · §10.2 · §10.3 NOVA · §13 ·
  §14 gates 8/16/17): a lista de restrições do ramo *Local* vem em DUPLICADO. Escritas pelo subagente-E
  da mesma janela, com o fonte reaberto só para estas perguntas.
  **As TRÊS auditadas contra §4.2 por R-pré em 2026-09-06** (contexto novo, independente de quem as
  escreveu; leu os dois lados, o fonte por shell) — sweep verde sobre a espec emendada + a pasta inteira
  das fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes caminhos.
  UM achado §4.2 (um nome interno do alvo em forma de identificador, no §10.4) e UMA insuficiência
  (o gate 16 fixava `2×`, que é só o caso de UMA passagem de simetria) **curados no acto pelo R-pré**;
  os factos das três emendas conferidos no fonte, **todos correctos**. Veredictos: LEDGER §Papel R.
  ⭐ ERRATA de 2026-09-06 (`3d621e94b` + `d5844ad5c`: §2.1 · §3.1 · §5.2 · §10 · fixtures de esfera)
  auditada contra §4.2 por R-pré em 2026-09-06 — sweep verde sobre espec + a pasta inteira das fixtures +
  INBOX + ledger + o histórico destes caminhos. TRÊS nomes internos do alvo (dois no §5.2, um no §10)
  re-expressos em vocabulário do domínio e UMA linha órfã de arnês apagada no §10, no acto; os seis
  factos da errata conferidos no fonte pelo R-pré, todos correctos. Detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q11 de 2026-09-06 — auditada contra §4.2 por R-pré em 2026-09-06** (§3.1 · §4.2 · §5.2 ·
  **§5.2-ter NOVA** · §9 nº 20 · **§10.6 NOVA** · §11 · §14 gates 19-21, + a fixture nova
  `plano_apertar_ponto_radial_local_origem_fraco`): o aperto **inverte a malha debaixo do cursor no
  PRIMEIRO passo simulado**, e a partir daí o resultado por vértice é decidido pela ORDEM de
  resolução. Escrita pelo subagente-E da mesma janela, com o fonte reaberto só para estas perguntas
  e com uma corrida NOVA do oráculo (o par de força do §5.2-ter).
  **R-pré: contexto novo, independente de quem a escreveu; leu os dois lados (o fonte por shell).**
  ⛔ **ZERO achados de §4.2** — sem trecho, sem nome interno, sem wording de comentário/manual; a
  ordem de criação do §3.1 é mecanismo (§4.1.11) escrito em vocabulário do domínio, e o nome da
  fixture nova também. Sweep **verde** sobre a espec emendada + a pasta inteira das fixtures + INBOX
  + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** da espec e das fixtures.
  ⭐⭐ **FIDELIDADE: os factos foram conferidos no fonte e os NÚMEROS reconstruídos do zero pelo
  R-pré a partir das fixtures** — a ordem interna de criação (as cinco espécies na ordem que o §3.1
  passou a dizer), o filtro de raio alcançar só duas delas, o anel percorrido face a face, a
  ausência de tecto no factor de correcção, a re-escala a comprimento 1 nos dois apertos, o instante
  em que cada modo lê as posições, e o **§9 nº 20** (a versão que gravou as fixtures MULTIPLICA —
  não há divergência a declarar). As **duas tabelas** do §5.2-ter e do §10.6 reproduzem célula a
  célula (`10 / 18 / 52` · `0,675` · `1,060` · `0,103` · `0,059` · `0,286` · `0,088` · `0,283` ·
  `2 145` · `2 029` · `0,303401` · `0,004082`), e os «9 vértices que passam o cursor» também.
  **QUATRO curas aplicadas no acto pelo R-pré, todas funcionais:** (1) o §3.1 dizia que a âncora e o
  pino nascem «para todo vértice visível», o que contradizia o §2.3 — cada um tem a SUA condição;
  (2) a régua da compressão do §5.2/§10.6 não dizia sobre que pares corria e **subestimava por
  `3,5×`** (`0,052`/`−18,1` só sobre arestas, contra `0,015`/`−64,4` sobre todos os pares da
  construção — as duas ficam agora escritas); (3) as **duas réguas** da tabela do §5.2-ter (o
  quadrilátero invertido e a assimetria de espelho) não estavam definidas em lado nenhum, logo o
  gate 19 não era edificável — ficam definidas, com a leitura ERRADA nomeada (somar metades
  triangulares dá `11 / 26 / 88`); (4) os gates 15 e 17 citavam `51` e `50` traços de memória, com
  `54` no disco. Mais a decisão do §5.2-ter posta nas duas frases que o dono precisa de ler.
  Veredictos e detalhe: LEDGER §Papel R.
  ✅ **EMENDA Q12 de 2026-09-06 — auditada contra §4.2 por R-pré em 2026-09-06** (§4.2-bis NOVA · §4.3 · §4.4 ·
  §4.6 NOVA · §10 contagem · §10.7 NOVA · §14 gates 22-24, + as duas fixtures por passo
  `plano_empurrar_radial_local_origem` e `plano_inflar_radial_local_origem` e o gerador do índice
  delas): a **normal da área** que o Push lê (quando é reavaliada, sobre que malha, em que raio, com
  que peso e com que regra de desempate), o **factor de escala** que a multiplica, a lei do **centro
  da área** (que fecha a pergunta em aberto no fim do §4.4), e o **censo das grandezas que são
  degeneradas num plano visto de frente e vivas numa superfície curva** — entre elas que o
  deslocamento do cursor que os oito modos lêem (no guarda) e de que quatro consumidores tiram
  direcção é a **projecção** no plano do ecrã, e que só o arrasto tira a direcção da diferença dos
  dois pontos 3D. Escrita pelo subagente-E da mesma janela, com o fonte
  reaberto só para estas perguntas e com uma corrida NOVA do oráculo (as 26 corridas-prefixo do §10.7).
  **R-pré: contexto novo, independente de quem a escreveu; leu os dois lados (o fonte por shell).**
  ⛔ **ZERO achados de §4.2** — sem trecho, sem nome interno, sem wording de comentário/manual; a
  regra dos dois baldes, a lei do centro da área e o censo do §4.6 estão em vocabulário do domínio, e
  as fórmulas são matemática (§4.1.2). Sweep **verde** sobre a espec emendada + a pasta inteira das
  fixtures + INBOX + os dois READMEs + `docs/3D/cloth/`, e sobre o **histórico** destes caminhos (os
  dois hits do ledger são os pré-existentes de 2026-09-05, já registados lá para o R-pós).
  ⭐⭐ **FIDELIDADE: os factos foram conferidos no fonte e os NÚMEROS reconstruídos do zero pelo
  R-pré** — a reavaliação por passo sobre a malha **deformada** (com a lista completa do que a
  congela), o disco de **meio** raio, o peso `3p²−2p³`, os dois baldes pelo sinal contra a vista, o
  vector nulo sem `NaN`, o factor de escala de três números fixado no pen-down e aplicado componente a
  componente, a lei do centro da área, e as **duas** metades da §4.6-1 (o deslocamento des-projectado
  à profundidade do pen-down · o arrasto a ler a diferença dos dois pontos 3D). Reproduzem-se do zero:
  `0,3518`/`+0,5 %` e `1,3184`/`+7,6 %` das calotas · `0,05455` e `0,01547` · `15,83°` e `1,039×` ·
  `19,3 %` · a razão `0,06543/0,09347 = 0,7000 = 2R` · as **11** colunas dos dois rastreios do §10.7 ·
  as contagens `0,236509` · `0,463862` · `0,325769` · `0,046715` · e o `gera_indice.py`, que
  **regenera o `indice.json` byte-a-byte** (56 entradas para 56 ficheiros).
  ⚠️ **A correcção à §5.2-bis (o duplicado nas costuras do *Dynamic*) confere**, e o mecanismo também:
  o registo de pares é local a cada construção e cada cópia carrega a sua própria célula.
  **SEIS curas aplicadas no acto pelo R-pré, todas funcionais:** (1) o desempate dos baldes é *o
  primeiro que esteja não-vazio **E** cuja soma não se anule* — a redacção anterior mandava responder
  o vector nulo num caso em que o alvo lê o outro balde; (2) o guarda de «passo parado» vale para os
  **oito** modos (a célula da §4.6-1 lia-se como se o arrasto não o tivesse), e Push/Inflate/Expand
  não lêem `δ` para a direcção nenhuma — não só o arrasto; (3) ⛔ **a §4.6-4 afirmava peso de ÁREA e
  isso NÃO é demonstrável** com o que esta linha tem à mão (nem o corpus o decide: no plano todos os
  pesos coincidem, na esfera UV a simetria em longitude apaga a diferença) — a linha passa a declarar
  a FORMA e a nomear o peso como pergunta aberta, com a régua que a fecharia; (4) o §10.7 dizia que a
  frente a `1R` **ultrapassa** o pen-down do Push e os próprios números dizem o contrário
  (`0,1976 < 0,2195`); (5) o gate 22 dava a sequência de ângulos truncada em `…` — ficam os **11**
  passos e a razão de ela ser simétrica; (6) o censo da §4.6 dizia «as oito fixtures de esfera» e
  listava sete — a oitava é o **arrasto**, que é precisamente o CONTROLO do gate 22.
  ⚠️ **E DUAS no README das fixtures, ambas de contagem, ambas da família que esta linha já pagou
  duas vezes:** `ls *.porpasso.txt.gz` devolve **13** e não `9`, e os quatro que sobram são a 1.ª
  geração — **três deles NÃO passam a prova do fatiamento** (`0.330421` · `0.115064` · `0.004244`) e
  estavam debaixo da frase «o pen-down de TODOS eles está na origem»; ⭐ o quarto passa, e passa
  *porque a área dele é Global e não tem centro para ficar refém do sobrevoo* — que é o mecanismo que
  o próprio README explica. Mais o total do fim, parado em `53` com `56` no disco, e a fixture
  `_fraco` que faltava na tabela das corridas (agora `56` linhas para `56` ficheiros).
  ⭐⭐⭐ **E uma SÉTIMA cura, de SUFICIÊNCIA, que fecha a pergunta que o I acabou de fazer (Q13):** a
  §4.3 descrevia `δ` como uma des-projecção de ecrã sem nunca dizer **qual é a vista**, e sem isso ele
  não era reconstruível deste lado. As fixtures são **ortográficas** (prova nos números) ⇒
  `δ = proj_⊥v̂(c_k − c_{k−1})` sobre o `caminho` que o cabeçalho já traz, com `v̂` = **`z`** no corpus
  do plano (projecção = no-op) e **`y`** no da esfera. Está agora na §4.3 e no README das fixtures,
  com o ⛔ de que o plano do **ECRÃ** não é o plano tangente do pen-down — a rota que o I mediu a
  piorar (`0,265 → 0,605` · `0,351 → 0,663`).
  Veredictos e detalhe: LEDGER §Papel R.
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
| 0 | **Primeiro passo de uma passagem?** ⇒ com área *Local*, constrói as restrições das células da área (§2, §3) e **termina sem simular** (F) — o alvo precisa de um deslocamento do cursor válido para orientar a ponta, e no 1.º passo ele é zero. ⚠️⚠️ **E esta construção NÃO marca as células como construídas** — a marca é a **ACTIVAÇÃO** (fase 3), que este passo nunca alcança (F) ⇒ **a fase 1 do passo seguinte constrói-as OUTRA VEZ, e o conjunto de restrições do *Local* fica com CADA restrição DUAS vezes** (§3.1, §5.2-bis: é a origem MEDIDA do factor `~2` de amplitude entre *Local* e *Global*). ⚠️ Com área *Dynamic*/*Global* o 1.º passo também não simula **e não constrói**: as restrições ficam para o 2.º passo e nascem **uma vez só**. | — | células com restrições, ainda **inactivas** |
| 1 | **Garantir restrições** para toda célula do conjunto afectado que ainda não as tenha (§3) | conjunto de células (§2.1) | restrições novas (só para células **ainda não activadas** — uma célula já activada nunca mais é construída; ⚠️ a fase 0 do *Local* constrói **sem** activar, logo essas células são construídas **duas** vezes) |
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
- as restrições nascem **de uma vez no 1.º passo — e OUTRA VEZ no 2.º**, logo em **DUPLICADO**, no Local (§1 fase 0, §5.2-bis); **incrementalmente e uma vez só**, à medida que
  novas células entram na área, no Dynamic;
- a banda `w` do Local é avaliada com o centro FIXO, logo o fim de um traço longo cai mais longe do
  centro da força ⇒ menos deslocamento lá; no Dynamic o centro segue o cursor e o traço inteiro
  recebe força cheia. ⇒ para o mesmo traço, o Dynamic desloca mais que o Local, e é ≈ Global. (A
  razão exacta **não** é emergente — ver a seguir.)

⭐⭐⭐ **A alavanca DOMINANTE não é o centro fixo: é a lista de restrições DUPLICADA do *Local*
(§1 fase 0, §3.1, §5.2-bis).** O centro fixo e o aro preso explicam o **BORDO** (§10.2), não o
**INTERIOR**: um port com o aro certo e a lista simples lê *Local* ≈ *Global* coluna a coluna no
interior, e no alvo o *Local* entrega no centro `0,34–0,57×` o *Global*, uniformemente ao longo do
traço (M — §10.2 e §10.3).

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

Cada par não ordenado entra **uma vez por CONSTRUÇÃO** — o registo de pares já criados nasce e morre **dentro de uma construção** (a fase 1 de um passo), e não é partilhado entre construções (F) ⇒ ⚠️⚠️ **duas construções sobre a mesma célula deixam DUAS cópias de cada par**, que é exactamente o que a área *Local* faz (§1 fase 0, §5.2-bis). Dentro de UMA construção não há duplicados (H:
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
- **Ordem:** as restrições ficam na ordem de criação e são resolvidas **nessa ordem**,
  sequencialmente (Gauss–Seidel). A ordem de criação é determinística dada a ordem das células (F).
  ⚠️⚠️ **E ela é EXACTAMENTE esta, o que só passou a ser preciso quando se soube que ela decide o
  resultado (§5.2-ter)** (F, 2026-09-06):
  1. **célula a célula**, na ordem em que a busca da árvore espacial as devolve (é a ordem dos nós
     dela, não a ordem dos índices de vértice);
  2. dentro da célula, os **vértices visíveis** dela, na ordem própria da célula;
  3. dentro de um vértice, **nesta ordem**: o **corpo mole** (se houver plasticidade) · as
     `(v, n)` para cada vizinho do anel, **na ordem do anel** · as `(a, b)` de cada par ordenado de
     vizinhos distintos, na mesma ordem do anel · a **âncora de deformação** · o **pino**.
     ⛔ O corpo mole vem ANTES das estruturais e o pino DEPOIS da âncora — a espec anterior não
     dizia a ordem interna, e ela é observável assim que duas espécies disputam um vértice.
  4. o registo de pares já criados é **partilhado pela construção inteira** — é o MESMO registo do
     parágrafo acima: **um por CONSTRUÇÃO**, comum a todas as células dela, e é por isso que duas
     construções deixam duas cópias. Logo a **primeira** ocorrência de um par fixa a posição dele na
     lista, e um par cujos dois vértices vivem em células diferentes cai na célula que lá chegou
     primeiro.
  ⚠️ **O anel de um vértice é percorrido face a face** (para cada face que contém `v`, os dois cantos
  adjacentes, deduplicados), logo a ordem do anel é a ordem das faces à volta do vértice — não uma
  ordem angular nem a dos índices (F).
- ⚠️ **O filtro de raio da construção só vale para as ESTRUTURAIS e para o corpo mole** (F,
  2026-09-06): a **âncora de deformação** e o **pino** ficam **FORA** dele — mas ⛔ isso **não** quer
  dizer «sem condição nenhuma»: cada uma tem a **sua**, e nenhuma delas é o raio da construção.
  A âncora radial do Grab usa o **raio do PINCEL** (`d⁰ < R₀`, filtro próprio e diferente); a do
  Grab-plano e a do Snake Hook não têm condição de distância nenhuma (todo vértice visível da
  célula); e o **pino** tem a dele, que é a **banda** (`w < 1`, §2.3) e só existe com a opção ligada
  e fora da área *Dynamic*.
  ⇒ um port que filtre as quatro espécies pelo mesmo raio cria menos âncoras e menos pinos do que o
  alvo. ⚠️ **Na área *Local* isso é observável só pela ORDEM da lista**, porque o factor por vértice
  dessas restrições extra é `0` (§5.2: `σ = 0` fora do conjunto que o gesto reescreve; `φ = 0` além
  do limite da banda) — mas a lista fica com outro comprimento e outra ordem, e isso basta (§5.2-ter).

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
- Não reconstrói uma célula **depois de ela ter sido ACTIVADA** — a partir daí ela é final (F).
  ⚠️ **Antes disso reconstrói:** a fase 0 do *Local* constrói e não activa, logo o passo seguinte
  constrói a mesma célula de novo e as restrições ficam em duplicado (§1, §5.2-bis).

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

⭐⭐⭐ **A MAGNITUDE DOS DOIS APERTOS NÃO DECRESCE COM A PROXIMIDADE — e é isso que os separa dos
outros quatro modos de força** (F, 2026-09-06 · M — §5.2-ter). Nos dois apertos `u` é o vector
`vértice → alvo` **re-escalado a comprimento 1** (no aperto de linha, `1` antes de se descartarem as
componentes; a projecção deixa-o `≤ 1`), e o único factor que sabe da distância é a **curva de
falloff — que ali está no MÁXIMO**. ⇒ o vértice que está a meia aresta do cursor recebe o mesmo
impulso do que está a meio raio, e **ultrapassa** o cursor. Não há tecto de deslocamento, nem corte
ao ultrapassar, nem amortecimento próprio: tudo o que o aperto tem, o arrasto também tem (F).

⚠️ **Consequência com número, na malha de referência** (aresta `0,0469`, `R = 0,35`, força `1`,
massa `1`): o impulso máximo é `10·α·dt/massa = 0,1` por passo (§4.1/§5.4), **`2,1×` a aresta**.
No **primeiro** passo simulado do aperto de ponto o oráculo põe `9` vértices para lá do cursor e
devolve **`10` quadriláteros com a orientação invertida**; o arrasto, no mesmo passo e com o mesmo
impulso, devolve **zero** (M — §5.2-ter). *A inversão não é um acidente de traço longo: ela nasce no
primeiro passo, a partir do repouso, e é a LEI.*

⚠️ **A direcção nula tem tratamento próprio e é o único caso especial dos apertos:** um vértice
**exactamente** sobre o cursor (ou, no falloff de plano, exactamente SOBRE o plano) dá separação
nula, e a re-escala a comprimento 1 aplicada a essa separação devolve o **vector NULO** ⇒ **força
zero**, sem `NaN`, sem direcção de reserva e sem o vértice ser saltado. Um vértice a um epsilon dele
recebe a força **inteira** (F). ⇒ *o ponto onde o aperto é mais forte é o ponto onde a direcção dele
está pior determinada.*

### §4.2-bis — A NORMAL DA ÁREA e o factor de ESCALA do Push (F, 2026-09-06)

⭐⭐⭐ **O Push é o único modo cuja direcção não é lida da malha vértice a vértice nem do gesto: ela é
UM vector por passo, e é a lei da casa do *Sculpt Plane*.** O deslocamento que ele empilha é

```
u = − n̂_área · R · escala · 2          (R = o raio do pincel neste passo)
```

**(1) Quando `n̂_área` é reavaliada.** A cada passo do traço, na **primeira** passagem de simetria; as
outras passagens recebem-na espelhada/rodada. As únicas coisas que a congelam são as opções
*Original Normal* / *Original Plane* do pincel (ambas **desligadas** nos presets de tecido — A) e o
facto de o pincel ser o *Grab* da casa (⛔ que **não** é o modo Agarrar do tecido: a regra de congelar
nomeia o pincel, não o modo de deformação). ⇒ **nas fixtures ela muda a cada passo.**

**(2) De que malha.** Das posições e normais **ACTUAIS** — a malha como o passo a encontra, já
deformada pelos passos anteriores. ⛔ Não é a malha de repouso do traço: a rota que lê as posições de
partida só é tomada por pincéis que oferecem a opção *Accumulate*, e o pincel de tecido não está
nessa família (F).

**(3) De que vértices, e com que peso.** De cada vértice **visível** das células que o pincel juntou
(§2.1) cuja distância ao cursor seja `d ≤ R · «Normal Radius»`, com a distância medida pela forma de
queda do pincel (3D com *Sphere*, que é a dos presets — A). Cada um contribui com a **normal do
vértice** multiplicada por `3p² − 2p³`, `p = 1 − d/(R·«Normal Radius»)`, saturado a `[0,1]`; o
resultado é a **soma normalizada**.
⚠️⚠️ **O «Normal Radius» dos presets de tecido é `0,5` (A)** ⇒ **a normal é amostrada num disco de
METADE do raio do pincel**, não no disco inteiro. *Um port que a tire do disco inteiro tem uma
direcção diferente assim que a superfície deixa de ser plana ou de estar em repouso.*

**(4) A regra de desempate, que não é uma média.** Os vértices são repartidos em **dois** baldes pelo
sinal de `n̂ · v̂` (`v̂` = a direcção da vista): `> 0` no balde da frente, `≤ 0` no de trás. A resposta é
a soma normalizada do **PRIMEIRO balde que esteja não-vazio E cuja soma tenha comprimento não-nulo,
nesta ordem fixa** — nunca a mistura dos dois, e ⛔ nunca «o balde com mais vértices». ⇒ basta **um**
vértice virado para a vista para que os virados ao contrário não contem.
⚠️ **O balde da frente ser não-vazio não basta:** se a soma dele se anular (normais opostas que se
cancelam), a resposta é a do balde de trás — o teste é *não-vazio **e** soma não-nula*, avaliado balde
a balde e nesta ordem, ⛔ **não** «escolher o balde e só depois olhar para a soma».

**(5) Quando não há resposta.** Se **nenhum** dos dois baldes passa esse teste (nenhum vértice
qualifica, ou as duas somas têm comprimento zero), `n̂_área` é o **vector NULO** — e o Push desse passo
é **força zero**, sem `NaN` e sem direcção de reserva.

**(6) A forma de queda *Projected*** projecta ainda `n̂_área` no plano do ecrã e re-normaliza (com
*Sphere*, a dos presets, não faz nada).

**(7) O `escala`.** É um vector de **três** números fixado no pen-down, e ele não depende do passo,
nem da pressão, nem da malha: `escala_eixo = max(|escala do objecto em x, y, z|) / escala do objecto
nesse eixo`. A multiplicação é **componente a componente** ⇒ num objecto de escala **uniforme** ele é
`(1, 1, 1)` e o módulo do deslocamento é exactamente `2R` (M: `plano_empurrar_radial_local_1passo`
dá `0,06942 = 2 · 0,35 · 0,0992`, §10.1); num objecto de escala **não-uniforme** ele **entorta a
direcção** além de mudar o módulo — não é um factor escalar.

⚠️ **O mesmo `n̂_área` é o `ẑ` do referencial local do traço e a normal que o falloff de plano usa
(§4.4)** — as três leituras são a mesma grandeza, calculada uma vez por passo.

### §4.3 — Os modos de ÂNCORA (dois) — e o delta de agarrar

**O delta de agarrar `δ`** é um vector em espaço do objecto, derivado do ECRÃ (F): o ponto do
cursor é des-projectado à **profundidade da localização original de agarrar** (a do 1.º passo) e
subtraído ao ponto anterior. Para o **Grab** o delta **acumula** desde o pen-down (ponto actual −
ponto original: um vector total); para os **outros sete modos** é **incremental** (ponto actual −
ponto anterior). Com *Normal Weight* `> 0` (omissão `0`) o delta é inclinado para a normal da
área como no Grab da casa. Com falloff *Projected* é achatado no plano da vista.

⭐⭐⭐ **A consequência que só uma superfície CURVA revela: `δ` NÃO é a diferença dos dois pontos 3D do
cursor — é a PROJECÇÃO dessa diferença no plano do ecrã** (F, 2026-09-06). As duas des-projecções são
feitas à **mesma** profundidade (a do pen-down), logo numa vista **ortográfica** a componente do
deslocamento ao longo do eixo da vista é **descartada por construção**, e numa vista em perspectiva
ela é descartada *e* o resto é reescalado pela razão de profundidades. ⇒ numa folha plana vista de
frente o caminho vive nesse plano e `δ` **é** a diferença dos pontos, ao bit; numa superfície curva
os dois vectores separam-se.
⚠️ **Quem lê `δ`:** o guarda de «passo sem movimento» de TODOS os modos · a âncora do Agarrar (que o
acumula) · a âncora e o avanço do centro do Snake Hook · a **normal do plano de queda** (§4.4) · o
`x̂` do referencial local do traço (§4.4). ⛔ **O guarda de «passo sem movimento» vale para os OITO
modos, o arrasto incluído** — ele corre antes de o modo ser escolhido.
⛔ **E o arrasto é o único modo que tira a direcção do MOVIMENTO do cursor e NÃO a tira de `δ`:** ela
é a diferença **dos dois pontos 3D** do cursor, normalizada (§4.2) — e é por isso que o arrasto é o
modo que se comporta igual nas duas superfícies. ⚠️ **Push, Inflate e Expand não lêem deslocamento de
cursor nenhum para a direcção** (normal da área · normal do vértice · nenhuma), e o aperto de ponto
também não (vértice → cursor) — eles só encontram `δ` no guarda e, se o falloff for de plano, na
normal desse plano.
⚠️ **Números na esfera das fixtures** (esfera unitária, caminho de `x = −0,3` a `+0,3` em 12 passos,
vista ao longo do eixo de profundidade — M): `δ` vale `(0,05455, 0, 0)` em **todos** os passos,
enquanto a diferença dos pontos 3D chega a `(0,05455, ∓0,01547, 0)` — **`15,83°`** de diferença de
direcção no 1.º e no último passo, `0°` a meio, e até `1,039×` de módulo. No **Agarrar**, que
acumula, o desvio entre o `δ` acumulado e a diferença acumulada dos pontos chega a **`0,04569`** a
meio do caminho, que é **`19,3 %`** do maior deslocamento daquela fixture (`0,236509`).

⭐⭐⭐ **E por isso `δ` É RECONSTRUÍVEL DO NOSSO LADO, sem câmara nenhuma — falta só o EIXO DA VISTA,
que fica aqui** (R-pré, 2026-09-06, derivado das próprias fixtures e confirmado pela medição
independente do I): as fixtures foram gravadas em vista **ORTOGRÁFICA**, e nessa vista as duas
des-projecções à mesma profundidade dão exactamente *a componente do deslocamento perpendicular ao
eixo da vista*. Logo, com `c_k` os pontos do `caminho` que cada fixture já traz no cabeçalho:

```
δ_k = proj_⊥v̂ (c_k − c_{k−1})            (os sete modos incrementais)
δ_k = proj_⊥v̂ (c_k − c_0)                (o Agarrar, que acumula)
```

⚠️ **O eixo da vista `v̂` NÃO está no cabeçalho das fixtures e é diferente nos dois corpora:**
- **corpus do PLANO** — a folha vive no plano `z = 0` e a vista é ao longo de **`z`** ⇒ a projecção
  é um **no-op** e `δ` é a diferença dos pontos ao bit (é esta a degenerescência que a §4.6-1 nomeia);
- **corpus da ESFERA** — a vista é ao longo de **`y`** (o caminho pousa em `y = −√(1−x²)`), ⇒ `δ` vive
  no plano **`x–z`** e é a componente `y` que se perde.

⭐ **A prova de que é ortográfica está nos próprios números**: o passo do caminho é `0,6/11 =
0,054545…` e `δ` mede `0,05455` em **todos** os 12 passos da esfera, apesar de os pontos do caminho
estarem a profundidades diferentes — numa vista em perspectiva os dois não podiam coincidir.
⛔ **Projectar no plano perpendicular à NORMAL DO PEN-DOWN é outra coisa e está medido a piorar** (o
I mediu `0,265 → 0,605` no Agarrar e `0,351 → 0,663` no gancho): o plano é o do **ECRÃ**, e só numa
folha vista de frente é que ele coincide com o plano tangente.

⚠️ **A localização do cursor** (`c`, centro do disco de influência):
- Drag/Push/Pinch/Inflate/Expand: **re-apanhada na superfície a cada passo** (raio contra a malha).
- **Grab**: **fica no ponto do pen-down** durante todo o traço (é o que faz o Grab «pegar» num
  conjunto fixo de vértices).
- **Snake Hook**: `c ← c + δ` a cada passo — o centro **anda com o gancho no plano de profundidade
  original**, não é re-apanhado na superfície.
  ⚠️⚠️ **E o `δ` desse avanço é o do passo ANTERIOR, não o deste passo (F, 2026-09-06):** o avanço
  acontece **antes** de `δ` ser recalculado, logo quando a queda por-vértice é avaliada o centro está
  **onde o pincel estava no início do passo**, não onde o cursor chegou. Em fórmula:
  `c_k = pen-down + Σ_{i<k} δ_i` — o centro está **um passo atrasado** em relação ao cursor, e como
  a localização **nunca mais é lida do evento depois do 1.º passo**, essa soma é a definição do
  centro (não há re-projecção nem raio contra a malha que possam divergir dela).
  ⭐ **No 1.º passo simulado (o 2.º passo do traço) `δ` do passo anterior é ZERO** ⇒ o centro é
  **exactamente o ponto do pen-down**, e o vértice mais deslocado é o do pen-down — não o que está
  sob o cursor (M — §10.4).
  ⇒ *é isto que faz o pico da deformação ficar ATRÁS do cursor, e um port que centre a queda no
  cursor apanha material novo a cada passo em vez de arrastar o que já pegou.*

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
do traço) — é por isso que o conjunto agarrado não muda quando a malha se mexe. ⚠️ **O Grab é o
ÚNICO assim:** os outros sete modos — o **Snake Hook incluído** — medem a distância, o recorte e a
textura sobre as **posições ACTUAIS** da malha, isto é, o estado deformado com que o passo começa
(F). ⇒ no Snake Hook o material já puxado viaja **com** o centro atrasado, e o gancho continua a
segurar o que agarrou em vez de agarrar o que está debaixo do cursor.

⚠️ **A força por passo das âncoras é ZERADA em TODO o objecto antes de ser reescrita, nos DOIS
modos de âncora** (F — a espec dizia «o Grab não», e estava errado): o passo começa com `σ ≡ 0` em
toda a malha e cada um dos dois preenche o que lhe toca — o Grab põe `1` (radial) ou `clamp(f,0,1)`
(plano) nas células afectadas, o Snake Hook põe `f`. O que os distingue **não é zerar ou não**, é o
valor com que reescrevem e o facto de o conjunto afectado do Grab ser fixo.

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

⭐⭐⭐ **O «CENTRO DA ÁREA» não é o centroide do disco — e é isso que fecha a pergunta que ficou em
aberto no INBOX** (F, 2026-09-06). Ele sai da **mesma** varredura que a normal (§4.2-bis: mesmo
conjunto de células, mesmo disco de raio `R · «Normal Radius»` = **metade** do raio do pincel, mesmos
dois baldes com a mesma regra do primeiro balde não vazio, mesmas posições **actuais**), mas o que se
soma **não é a posição do vértice**:

```
contribuição(v) = c + (p_v − c) · (1 − a_v)      a_v = 3p² − 2p³,  p = 1 − d_v/(R·«Normal Radius»)
centro da área  = média das contribuições do balde escolhido
                = c   se nenhum vértice qualificar
```

⇒ **o peso `1 − a` vale ZERO no cursor e cresce para a borda do disco**: cada vértice é *puxado para
o cursor* antes de entrar na média, e quanto mais perto do cursor está, mais completamente é
substituído por ele. Numa folha em repouso com o cursor sobre ela o centro da área **é praticamente o
cursor**; numa folha já cavada ele fica **muito mais perto do cursor** do que o centroide das
posições.
⚠️⚠️ **É por isso que a medição do I — «o plano pelo CURSOR reproduz o alvo e o plano pelo centro da
área afasta-o» — não refuta esta secção: o que foi medido foi um CENTROIDE, e o alvo não usa um
centroide.** *Uma recusa medida responde a uma pergunta, e aquela respondeu «o centroide não serve»,
não «o centro da área não serve».* O plano pelo cursor é a **aproximação de primeira ordem** do
centro da área, e é por isso que ele passa quase.
⚠️ **A origem do referencial local NÃO é o centro da área — é o cursor `c`** (o bloco acima já o
diz): as duas grandezas têm consumidores diferentes, e só o plano de queda lê o centro da área.
⚠️ Na área *Local* o centro da área e a localização inicial da área (§2.1) são **coisas distintas**:
o centro da área é reavaliado a cada passo à volta do cursor; a localização inicial fica no pen-down
durante todo o traço.

### §4.5 — Expand — o modo que muda o REPOUSO

`Expand` não aplica força nem âncora: para cada vértice sob o pincel, um **desvio de repouso**
por vértice `τ_v` acumula `τ_v += 0,01 · f_v` a cada passo (F). O comprimento de repouso efectivo
de toda restrição `(a, b)` passa a ser `ℓ + (τ_a + τ_b)/2` (§5.2). Com `flip = −1` contrai.
- `τ` vive na simulação ⇒ **morre no fim do traço** (§6.3), mas a geometria que ele produziu fica.
- A ordem de grandeza: a força máxima, `B = 0,1` ⇒ `τ` cresce `0,001` por passo no centro — e é
  **absoluto** (não relativo à aresta): numa aresta de `0,047` (a grelha do oráculo) são `2 %` por
  passo (M, §10).

### §4.6 — O que é DEGENERADO num plano visto de frente e VIVO numa superfície curva (F+M, 2026-09-06)

⛔⛔⛔ **Leia isto antes de procurar uma lei «para superfície curva»: não existe nenhuma.** Nenhuma
decisão do alvo — nem no gesto, nem no solver, nem na construção — pergunta pela curvatura, pela
normal do ponto do cursor ou pelo tipo de malha. O que existe é uma lista **fechada** de grandezas
que, numa grelha plana vista de frente e em repouso, valem sempre a mesma coisa (ou zero), e que
numa esfera passam a variar. *Um port calibrado só no plano acerta nelas por acidente.*

| # | grandeza | no plano visto de frente | numa superfície curva | quem a lê |
|---|---|---|---|---|
| 1 | **o deslocamento do cursor `δ`** (§4.3) | **igual** à diferença dos dois pontos 3D | a componente de profundidade é **descartada**: `15,83°` de diferença de direcção nas fixtures de esfera, `1,039×` de módulo, `0,04569` de desvio acumulado (`19,3 %` do maior deslocamento do Agarrar) | Agarrar · Snake Hook · normal do plano de queda · `x̂` do referencial · e o guarda de «passo parado» dos **oito** modos (o arrasto incluído) — ⛔ o que o arrasto **não** tira de `δ` é a **direcção** |
| 2 | **a normal da área** (§4.2-bis) | exactamente a normal da folha, em todos os passos, até a folha se cavar | média com peso `3p²−2p³` das normais **actuais** num disco de **meio** raio: não é a normal no ponto do cursor, e roda com a vala que o traço vai abrindo | Push · plano de queda · `ẑ` do referencial |
| 3 | **o centro da área** (§4.4) | praticamente o cursor | mistura pesada que **puxa para o cursor**, e não o centroide do disco | plano de queda |
| 4 | **a normal do vértice** | `+ẑ` para todos, seja qual for o peso usado ao somar as faces | **soma das normais das faces que tocam o vértice, sobre a malha ACTUAL, normalizada no fim** (uma soma de comprimento zero cai num eixo fixo do objecto); ⚠️ **o PESO dessa soma (área · ângulo · uniforme) NÃO está estabelecido** — ver a nota abaixo | Inflate |
| 5 | **a repartição em dois baldes** pelo sinal de `n̂ · v̂` (§4.2-bis) | um balde só: o segundo nunca é usado | o segundo balde enche-se assim que o disco alcança a silhueta, e ⛔ ele só é lido se o primeiro estiver **vazio ou somar zero** (§4.2-bis-4) | normal e centro da área |
| 6 | **a distância** ao cursor e à localização da área | a distância no plano | **corda 3D, nunca geodésica**: na esfera unitária o disco de `R = 0,35` é uma calota de arco `0,3518` (`+0,5 %`) e o limite da área de `3,5R = 1,225` é uma calota de arco `1,3184` (**`+7,6 %`**) | a curva de queda, o filtro de raio, a banda da área, o filtro de construção |

⚠️⚠️ **A linha 4 tem uma metade ABERTA, e ela é a única do censo (R-pré, 2026-09-06).** O que está
demonstrado é a FORMA (somar as normais das faces incidentes e normalizar no fim, sobre a malha
actual) e o consumidor (Inflate). O **peso** de cada face nessa soma — área (somar normais de face
não normalizadas), ângulo do canto, ou uniforme — **não é demonstrável com o que esta linha tem à
mão**, e ⛔ **o corpus também não o decide**: no plano todos os pesos dão `+ẑ`, e na esfera UV a
simetria em longitude põe os três a menos de ruído um do outro. ⇒ **um port escolhe um, escreve qual
escolheu ao lado da linha, e a pergunta fica na fila** — a régua que a fecharia é um traço de Inflate
do oráculo sobre uma malha **deliberadamente irregular** (triângulos de áreas e ângulos muito
diferentes à volta do mesmo vértice), que hoje não existe no corpus. *Uma ausência de régua não é uma
resposta, e esta linha está declarada em vez de adivinhada.*

⚠️ **A leitura das oito fixtures de esfera (M) sai desta tabela, e ela NÃO é uma família só:**
- **Agarrar · Snake Hook · Aperto de linha** — linha 1 (e a 2 no aperto de linha, que monta o
  referencial): o `δ` projectado.
- **Push** — linha 2 (e a 5): a normal da área.
- **Inflate** — linha 4: a normal por vértice da malha deformada.
- **Aperto de ponto** — ⛔ **não é curvatura**: o maior deslocamento da fixture de esfera (`0,4639`)
  é **maior** que o do plano (`0,3258`), logo ela está no **regime que o §5.2-ter descreve**, onde a
  malha inverte e quem decide o vértice é a ordem de resolução. A régua ali é a dos **dois regimes**
  do gate 20, não uma barra por vértice.
- **Expand** — ⛔ **também não é curvatura**: ele não lê nenhuma das seis linhas (só o factor por
  vértice), e o maior deslocamento da fixture é `0,046715` sobre uma malha cuja aresta no equador
  mede `0,0491`×`0,0654` ⇒ *o denominador da razão é menor que uma aresta*, exactamente como no
  `plano_expandir_radial_local_1passo`. Ele pertence à pergunta do Expand no plano.
- **Arrasto** — é a **OITAVA** fixture de esfera e não tem linha nenhuma da tabela: a direcção dele é
  a diferença dos dois pontos 3D (linha 1, o ⛔). ⭐ É por isso que ele é o **CONTROLO** da 2.ª metade
  do gate 22: se um port projectar em toda a parte, é esta fixture que o denuncia.

⚠️ **A malha da esfera das fixtures é ANISOTRÓPICA** (96 meridianos × 64 paralelos ⇒ no equador
`0,0654` na longitude contra `0,0491` na latitude — M): os comprimentos de repouso das restrições de
par do anel (§3.1) não são todos iguais, e os pólos são vértices de valência `96`. ⛔ Nada disso é
uma regra nova — é a mesma construção do §3.1 sobre outra topologia —, mas um port que assuma uma
grelha regular ao escrever o anel mede-se bem no plano e mal aqui.

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
- **A correcção é `Δ/2`, não `Δ` inteiro**, para TODA espécie (o `h = Δ/2` do laço acima é o que cada
  extremo recebe). Numa
  restrição estrutural, cada um dos dois vértices leva `Δ/2` ⇒ juntos fecham `Δ`. **Numa âncora, B não
  é vértice e NÃO se move: só A leva `Δ/2`** ⇒ a âncora fecha só metade por varredura (é «mole» de
  propósito, e por isso precisa das 5 varreduras para chegar). *Se um port dá abaixo do oráculo com
  `Δ/2`, o défice está noutro factor (o `σ` por passo do Snake Hook, o `s`, ou a re-ancoragem por
  passo), não em trocar `Δ/2` por `Δ` — o fonte é `Δ/2`.*
- **O `σ` (o factor por passo) multiplica SÓ as âncoras de DEFORMAÇÃO** (Grab, Snake Hook, pincel
  alheio): vale `1` em toda restrição que não seja âncora de deformação, e só é `(σ_A+σ_B)/2` quando a
  restrição é de deformação. ⛔ O **pino** e o **corpo mole** NÃO o levam (o seu peso é a força `s` e,
  no corpo mole, a plasticidade `ρ`).
- **A força `s` do Grab radial é `0,1 · curva(d⁰)` com a curva do PINCEL** (o preset de falloff do
  pincel activo avaliado na distância de repouso ao centro), não uma curva fixa.

⚠️⚠️ **O factor `(1 − ℓ'/D)` NÃO TEM TECTO, e muda de SINAL quando o par fica COMPRIMIDO** (F,
2026-09-06 — o único guarda do fonte é o `D = 0`): com `D > ℓ'` ele vive em `[0, 1)` e a correcção
puxa; com `D < ℓ'` ele é **negativo e cresce sem limite** à medida que `D → 0`, e a correcção
**empurra**, com magnitude `0,6/2 · (ℓ'/D − 1)` **vezes a separação actual** por projecção. ⇒ *é
aqui que a relaxação deixa de ser uma contracção e passa a ser um amplificador — ver §5.2-ter.*

⚠️⚠️ **E a régua da compressão tem de dizer SOBRE QUE PARES é que corre, senão subestima por `3,5×`**
(M, conferido pelo R-pré em 2026-09-06 sobre as fixtures `*.porpasso`): o pior `D/ℓ` dos doze passos
do aperto de ponto vale **`0,052`** (factor `−18,1`) se se olharem só os pares que são **arestas da
malha**, e **`0,015`** (factor **`−64,4`**) se se olharem **todos** os pares que a construção cria —
que é a população certa, porque as restrições de PAR DO ANEL (cisalhamento e dobra, §3.1) são
restrições de distância como as outras e sofrem a mesma projecção. O arrasto, na mesma régua larga,
nunca desce abaixo de `D/ℓ = 0,49` (factor `−1,1`), e a fixture de força fraca fica em `0,830`
(factor `−0,20`). ⇒ um par empurrado a **`19×`** a separação dele numa só projecção, e a lista do
*Local* projecta-o **dez** vezes por passo. ⚠️ **Quando este documento diz «par estrutural» numa
tabela de medição, é a régua LARGA que vale** — a estreita fica aqui só porque foi a primeira a ser
escrita e a diferença entre as duas é ela própria o achado.

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

### §5.2-bis — A AMPLITUDE do *Local*: a lista de restrições vem em DUPLICADO (F; M 2026-09-06)

⭐⭐⭐ **O facto.** Com área *Local*, o conjunto de restrições contém **cada restrição exactamente
duas vezes** — duas cópias idênticas, na ordem «a lista inteira, e a seguir a lista inteira outra
vez». Não é uma decisão de rigidez nem um parâmetro: é a consequência de a fase 0 construir **sem
activar** (§1, §3.1, §3.3). Com *Global* cada restrição aparece **uma** vez.

⚠️⚠️ **CORRECÇÃO de 2026-09-06 (F) — no *Dynamic* a lista NÃO é simples: ela é simples DENTRO de cada
passo e duplicada nas COSTURAS entre passos.** O registo de pares já criados vive **uma construção**
(§3.1) e no *Dynamic* há **uma construção por passo** (a de cada passo constrói as células que
acabaram de entrar no alcance e ainda não foram activadas). Um par cujos dois vértices vivem em
células construídas em passos **diferentes** é criado **duas** vezes: uma pela célula que chegou
primeiro (com o anel do vértice dela a alcançar a vizinha) e outra pela célula que chegou depois.
⇒ **a frente que varre a malha deixa atrás de si uma costura de restrições em duplicado**, uma por
fronteira entre lotes de construção. ⛔ As duas cópias carregam **células diferentes**, logo cada uma
só é projectada nos passos em que a **sua** célula está activa — e num passo em que as duas estejam
activas, o par é projectado a dobrar. *No *Local* isto não acontece* (o conjunto de células é fixo e
constrói-se duas vezes inteiro); no *Global* também não (uma construção só, a malha toda).
⚠️ Isto vale para as **10** fixtures de área *Dynamic* — as 2 do plano e as **8 da esfera** —, e é uma
grandeza que se **conta**, não se escolhe: são os pares cujos dois extremos caíram em lotes de
construção distintos.

**O que isso faz.** As `5` varreduras (§5.2) percorrem a lista sequencialmente (Gauss–Seidel), logo
no *Local* cada restrição é projectada **`10` vezes por passo** em vez de `5`, com a 2.ª projecção a
ver já o resultado da 1.ª. O material fica mais rígido, o deslocamento no INTERIOR cai, e o
**alcance** cresce (numa relaxação sequencial o alcance por passo É o número de passagens).

⚠️ **É constante desde o início do traço, e NÃO acumula:** a lista já está dobrada quando a primeira
relaxação com efeito corre. As duas cópias são iguais (mesma ordem, mesmos comprimentos de repouso,
mesmas forças), porque as duas construções partem do mesmo centro fixo, do mesmo raio inicial e das
mesmas posições de repouso (F).

⚠️ **Porque é INVISÍVEL no primeiro passo simulado dos modos de FORÇA.** A relaxação corre ANTES da
integração (§5.2): nesse passo a malha ainda está em repouso e toda correcção estrutural é zero —
percorrer duas vezes uma lista de correcções nulas não muda nada. Por isso o passo 2 do *Local* e do
*Global* é idêntico (`0,09347` sob o pen-down nas duas fixtures, §10.2) e a divergência começa no
passo 3. ⛔ **Nos modos de ÂNCORA não é invisível já no passo 2**, porque a âncora é escrita na fase
4 e a restrição de âncora **não** está satisfeita quando a relaxação corre — é isso que faz um traço
de Grab de um único passo simulado mover `~1 300` vértices em vez de `~870` (M — §10.3).

⚠️ **Duplicam-se TODAS as espécies** (estrutural, âncora de deformação, corpo mole, pino), porque o
que corre duas vezes é a construção inteira (§3.2).

⚠️ **Com simetria há MAIS cópias:** a fase 0 corre uma vez por passagem de simetria e todas elas
constroem antes de qualquer activação, logo uma célula tocada por `n` passagens fica com `n` cópias
da fase 0 **mais** uma da fase 1 do 2.º passo = `n + 1` (F). Nas fixtures o factor medido é
exactamente `2` (⇒ uma passagem efectiva) — M, §10.3.

⛔ **«Pôr 10 varreduras» NÃO é o mesmo, e a medição diz quanto:** dobrar a lista intercala as duas
projecções de cada restrição de outra maneira do que dobrar o laço exterior. Um traço de arrastar
cruza o oráculo a `10` e um de agarrar entre `9` e `10` (M — §10.3). ⇒ **para um port, a forma FIEL é
a lista dobrada no ramo *Local*, não um número de varreduras diferente por área.**

⭐ **E isto FECHA a suspeita de que a correcção de âncora seria maior do que `Δ/2`** (§5.2): não é —
`Δ/2` está certo, e o défice de um port estava no número de PASSAGENS, porque todas as fixtures de
âncora são de área *Local*.

### §5.2-ter — O APERTO INVERTE A MALHA NO 1.º PASSO, e a partir daí quem decide é a ORDEM (F; M 2026-09-06)

⛔⛔⛔ **Leia isto antes de procurar uma lei de força que falte nos apertos: não falta nenhuma.** As
três coisas que um port procura primeiro **não existem** no alvo (F, conferido linha a linha):
o vértice sobre o cursor **não** tem tratamento especial além da direcção nula (§4.2) · o factor e a
direcção do aperto são avaliados **no mesmo instante** que os dos modos de arrasto — as posições com
que o passo começa, antes da relaxação dele — e só o Grab lê outro instante (§4.3) · e **não há**
tecto de deslocamento, corte ao ultrapassar o cursor nem amortecimento próprio do aperto.
⇒ *a relaxação também não faz NADA de diferente num passo de aperto: ela não sabe qual é o modo de
deformação* (nos modos de força não há sequer âncoras, e a lista de restrições nasce das posições de
REPOUSO, iguais nos dois casos).

⭐⭐⭐ **O que o aperto faz de diferente é ANTES da relaxação: ele vira a malha do avesso debaixo do
cursor, no PRIMEIRO passo simulado.** A magnitude não decresce com a proximidade e a curva de
falloff está no máximo ali (§4.2), logo o vértice ao lado do cursor anda mais do que a distância a
que estava dele. A partir daí a relaxação recebe pares **comprimidos**, onde `(1 − ℓ/D)` inverte o
sinal e cresce sem tecto (§5.2), e **o resultado por vértice passa a ser decidido pela ORDEM em que
a lista é percorrida** — que é a coisa que um Gauss–Seidel não comuta.

⭐⭐ **A PROVA está dentro do próprio oráculo, e não precisa de nós: a SIMETRIA DE ESPELHO.** A malha
de repouso, o caminho do cursor (em `y = 0`), a lei da força e o **conjunto** de restrições são todos
simétricos em relação ao traço. A **ordem** da lista não é.

⚠️⚠️ **AS DUAS RÉGUAS DA TABELA, escritas para se poderem CONSTRUIR** — sem isto o gate 19 não é
edificável, e um número medido cuja régua não está escrita não é um número (R-pré, 2026-09-06: as
duas foram **reconstruídas do zero a partir das fixtures** e devolvem, célula a célula, os valores
da tabela; se um port as escrever de outra maneira, mede outra coisa):

- **Quadrilátero de orientação invertida** = a face cuja **normal depois do passo** aponta ao
  contrário da normal dela **em repouso**. Na grelha plana isso é o sinal da componente `z` do
  **produto vectorial das DIAGONAIS** do quadrilátero, `(p₃ − p₁) × (p₄ − p₂)` com os cantos na
  ordem do quadrilátero — equivalente, na prática, à normal de Newell do polígono. Conta-se sobre
  **todas** as faces da malha, e o valor é um **inteiro**. ⛔ Não é «somar as duas metades
  triangulares»: essa leitura conta também o quadrilátero apenas DOBRADO (que não inverteu) e
  devolve `11 / 26 / 88` onde a régua certa devolve `10 / 18 / 52`.
- **Assimetria de espelho ÷ `|u|max`** = `max_v ‖u(v) − M(u(m(v)))‖∞` a dividir por `max_v ‖u(v)‖₂`,
  com `m` o vértice reflectido no plano do traço e `M` a reflexão do próprio vector de
  deslocamento. Numerador em **norma do máximo por componente**, denominador em **norma euclidiana**
  — as duas normas são diferentes de propósito, e trocá-las muda o número.

Por passo (M — fixtures `*.porpasso`):

| traço | quadriláteros invertidos `k=2 / k=3 / k=12` | assimetria de espelho ÷ `|u|max`, `k=2 / k=3 / k=12` |
|---|---|---|
| **aperto de PONTO** (força `1`) | **`10` / `18` / `52`** | `0,000` / **`0,675`** / **`1,060`** |
| **aperto de PONTO** (força `0,2` — o controlo) | **`0` / `0` / `0`** | `0,000` / `0,103` / `0,144` |
| aperto de LINHA (força `1`) | `6` / `5` / `2` | `0,000` / `0,099` / `0,204` |
| arrastar *Local* | `0` / `0` / `0` | `0,000` / `0,059` / `0,219` |
| arrastar *Global* | `0` / `0` / `57` | `0,000` / `0,064` / `0,286` |
| Snake Hook (`_2passos_origem`) | `0` / `11` / — | `0,088` / `0,283` / — |
| Grab (`_2passos_origem`) | `0` / `0` / — | `0,095` / `0,099` / — |

⭐ **Três leituras que esta tabela fecha:**
1. **A assimetria é fabricada pela relaxação e por mais nada.** Em TODOS os modos de FORÇA ela é
   `0,000000` no 1.º passo simulado — exactamente o passo em que a relaxação corre sobre a malha em
   repouso e não tem o que corrigir (§5.2) — e nasce no passo seguinte. Nos DOIS modos de ÂNCORA ela
   já lá está no 1.º passo simulado, que é exactamente o passo em que a âncora dá trabalho à
   relaxação (§5.2-bis). *A régua concorda com o mecanismo nos dois sentidos.*
2. **O piso da ordem é `6 %` a `10 %`** do maior deslocamento (arrasto e Grab, sem uma única face
   invertida). É o preço que **qualquer** port paga por não ter a mesma ordem, e é a barra honesta do
   gate 15.
3. **A inversão multiplica esse piso por `7` a `11`** (`0,675` contra `0,059`), e é a ÚNICA coisa que
   distingue os traços: as linhas com faces invertidas são as linhas com assimetria grande, na mesma
   ordem (ponto `>` gancho `>` linha `>` arrasto), e essa é **exactamente** a ordem de erro que um
   port mede contra o oráculo.

⭐⭐⭐ **E é uma INTERVENÇÃO, não uma correlação: o par de força.** A fixture nova
`plano_apertar_ponto_radial_local_origem_fraco` é o **mesmo** traço, a mesma malha, o mesmo caminho e
os mesmos parâmetros, com **uma** coisa mudada — força `1 → 0,2`, que põe o impulso máximo em
`0,004` (`0,085×` a aresta, contra `2,1×`). Resultado: **zero** faces invertidas nos doze passos, e a
assimetria cai de `0,675` para `0,103` — o piso do arrasto. ⇒ *tira-se a inversão e a
sensibilidade à ordem desaparece; o modo, a lei e a maquinaria não mudaram.*

⛔⛔ **A consequência para um port, e ela é uma decisão de PRODUTO, não de engenharia:**
- a ordem do alvo é a do §3.1 e é **cell-major**, sobre a partição da árvore espacial DELE; na malha
  destas fixtures (4 225 vértices) essa partição tem **~2 células** (§2.1) e a fronteira entre elas
  passa pela zona do pincel;
- ⇒ **num retalho invertido, o resultado por vértice do aperto não é reproduzível por uma árvore
  espacial diferente da do alvo.** Não é lei em falta: é uma resposta que a ordem define. Um port com
  outra partição fica no piso de `6 %`–`10 %` **fora** da inversão e em `≈ 70 %` **dentro** dela.
- ⇒ a barra de paridade dos modos de aperto **não pode ser por vértice num retalho invertido**
  (gate 20); e a régua que continua a valer por vértice é a de **antes** da inversão — o 1.º passo
  simulado, onde os seis modos de força já são exactos ao bit, e a fixture de força fraca inteira.
- ⚠️ **E o alvo sabe que isto é um defeito dele:** são as duas entradas abertas do §9 nº 23 —
  *artefactos dos pincéis de tecido* e *o aperto do filtro numa superfície plana*. ⇒ **reproduzir o
  oráculo aqui é reproduzir um defeito conhecido e aberto do alvo.** A saída alternativa —
  limitar o impulso do aperto à distância que falta até ao alvo, que é a única linha que a inversão
  pede — **muda o produto e diverge do oráculo de propósito**, e por isso é decisão do dono, com o
  preço já medido nesta tabela. ⛔ Não a tome sozinho.
  ⚠️ **A decisão põe-se em duas frases, e são estas** (sem elas ninguém a pode tomar sem adivinhar):
  **(a) reproduzir** — apertar com força alta vira o retalho debaixo do cursor do avesso, as faces
  atravessam-se e a superfície fica com um nó que nada desfaz depois (§11); é o que o alvo faz hoje,
  e apertar com força baixa continua limpo. **(b) limitar** — o aperto nunca ultrapassa o ponto para
  onde puxa, o nó não aparece em força nenhuma, e a nossa saída deixa de casar com a do alvo
  exactamente nos traços fortes. ⛔ **Não há terceira**: a inversão nasce no PRIMEIRO passo, antes de
  a relaxação correr, logo nenhuma afinação do solver a evita.

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
| 20 | Regressão de 2024 no Pinch: uma multiplicação foi trocada por uma subtracção num refactor — o comportamento «de antes» é a multiplicação (a do §4.2). ⚠️⚠️ **CORRIGIDO NA ESPEC (2026-09-06): ela foi CONSERTADA no mesmo dia em que foi relatada, DOIS ANOS antes da versão que gravou as fixtures, e o consertar foi voltar à multiplicação** — a versão do oráculo lê o aperto tal como o §4.2 o descreve (F: direcção × factor), e **não** há divergência deliberada a declarar aqui. ⇒ *uma entrada de regressão FECHADA numa tabela de história lê-se como dívida viva; esta diz agora a data do conserto.* | #127836 relatada e fechada em 2024-09-19 |
| 21 | Com falloff *Constant* o corte no raio tem de ser explícito (a curva constante não corta) — daí o corte duro do §4.1. | #139846, 2025-06-06 |
| 22 | O Grab com falloff de plano estoirava ao clicar fora da malha (o cache do traço passou a existir antes de haver superfície) — a cerca é «não desenhar/agir sem superfície sob o cursor». | #161820, 2026-07-23 |
| 23 | **Abertos**: artefactos dos pincéis de tecido (#138844) · distorção grande em malhas de densidade irregular (#131510 — ⚠️ consistente com forças ABSOLUTAS e restrições ao anel-1, §4.1/§3.1) · Grab de tecido e Boundary com alvo = simulação não funcionam com simetria (#131122) · o Pinch do filtro numa superfície plana (#132316) · o filtro ignora a espessura exterior do colisor (#96124) · o botão direito não cancela o filtro de imediato (#105335). | tracker, 2024–2026 |

---

## §10 — Vectores de teste (o oráculo)

⭐ **56 traços do binário 5.2.1 sobre malhas NOSSAS** — ⚠️ **CONTE-OS, não cite este número de
memória** (`ls docs/3D/cleanroom/fixtures/cloth/*.deformado.txt.gz | wc -l`): esta linha esteve em
`51` depois de a §10.5 acrescentar dois, a §10.6 mais um e a §10.7 mais dois.
⚠️ **O `indice.json` é DERIVADO e regenera-se** — `python3 fixtures/cloth/gera_indice.py` (uma
entrada por `.deformado.txt.gz`); ⛔ não o edite à mão, e não confie num número escrito aqui. Malhas: grelha plana 64×64 e esfera UV
96×64; um traço por modo e por variante de solver, em `fixtures/cloth/` (proveniência e verificador
no README de lá).
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

⚠️⚠️ **As fixtures de ESFERA são todas de área DINÂMICA (ERRATA de 2026-09-06).** A 1.ª entrega
gravou-as como *Local*, mas um traço scriptado **não dispara o hover** que fixa o centro da área
Local — esse centro fica na ORIGEM do objecto (o valor obsoleto da **localização inicial** guardada —
a célula «centro» do Local na tabela do §2.1). Numa esfera
unitária a origem põe **toda** a malha dentro da banda (todo vértice a `1,0` < início da banda
`1,006`), então a saída lida `6 050 / 6 050` movidos, uniformes — a esfera a deslocar-se como um
CORPO, que é **artefacto do arnês, não comportamento Local do alvo**. ⭐ **O R₀ estava certo (`0,35`,
tamanho travado à cena); o defeito era o CENTRO.** A área Dinâmica, cujo centro é o cursor de cada
passo (que o traço scriptado FORNECE), funciona: as 8 fixtures de esfera param no bordo da banda
(`alcance ≈ 3,5 R`, `0` vértices além), e é sobre elas que se lê o relevo fora do plano numa
superfície curva. A área **Local** fica medida **só no plano** (onde a origem cai na superfície e o
disco de `3,5 R` é exacto). Mecanismo e a errata completa: [ledger](LEDGER_blender-cloth.md).

---

### §10.2 — O instrumento POR PASSO (2026-09-06, a pedido do I)

Quatro traços com as posições **depois de cada passo** (`fixtures/cloth/*_origem.porpasso.txt.gz`,
método e prova no README de lá): Drag *Local* e *Global* (12 passos), Snake Hook e Grab *Local*
(2 passos simulados). ⚠️ **Pen-down na origem** (determinismo do centro *Local*, ver README).
O rastreio de sete vértices por passo (`*.porpasso.rastreio.txt`) é o lado MEDIDO para a Q3 do I
(Local ≈ metade do Dinâmico/Global): ⭐ **o que os números dizem** —

| passo | Local: sob o pen-down | Global: sob o pen-down | Local: no limite 3,5R | Global: no limite 3,5R | Local: fora 4R | Global: fora 4R |
|---|---|---|---|---|---|---|
| 1 | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 2 | `0.09347` | `0.09347` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 3 | `0.16758` | `0.20530` | `0.00000` | `0.00000` | `0.00000` | `0.00000` |
| 4 | `0.21314` | `0.27913` | `0.00000` | `0.00005` | `0.00000` | `0.00001` |
| 5 | `0.24711` | `0.33335` | `0.00000` | `0.00028` | `0.00000` | `0.00011` |
| 6 | `0.27291` | `0.38610` | `0.00002` | `0.00103` | `0.00000` | `0.00049` |
| 7 | `0.29041` | `0.44464` | `0.00004` | `0.00273` | `0.00000` | `0.00157` |
| 8 | `0.29737` | `0.49123` | `0.00008` | `0.00584` | `0.00000` | `0.00388` |
| 9 | `0.29100` | `0.54262` | `0.00013` | `0.01074` | `0.00000` | `0.00797` |
| 10 | `0.27014` | `0.58434` | `0.00019` | `0.01766` | `0.00000` | `0.01427` |
| 11 | `0.24062` | `0.61514` | `0.00025` | `0.02659` | `0.00000` | `0.02293` |
| 12 | `0.22022` | `0.64571` | `0.00032` | `0.03738` | `0.00000` | `0.03378` |

1. **O bordo do Local é uma ÂNCORA e o do Global não existe:** no limite (`3,5R`) o Local move
   `≤ 0,0003` em 12 passos e **fora dele exactamente `0`** (sem restrições, `φ = 0`); o Global move o
   mesmo vértice `0,037` e o de fora `0,034` — a folha inteira desliza. ⇒ o disco Local é uma membrana
   **presa no aro** por `w → 0` DENTRO do conjunto restringido (o raio de construção é o LIMITE, não o
   início da banda).
2. **A consequência está no pen-down:** no Local o vértice do pen-down sobe até `0,297` (passo 8) e
   depois **RECUA** para `0,220` — o material atrás do cursor é puxado de volta para o aro preso; no
   Global sobe monotonamente até `0,646` (a folha acompanha o cursor com a velocidade acumulada).
   Sob o cursor do passo os dois são parecidos (`≈ 0,20–0,23` contra `≈ 0,22–0,31`).
3. **Até ao passo 2 os dois são IGUAIS ao bit** (`0,09347` / `0,00072` / `0`) e só divergem no
   passo 3. ⚠️ **A 1.ª leitura desta linha — «logo o mecanismo é o aro» — está REFUTADA** (§10.3):
   os DOIS mecanismos previam esta igualdade, porque no passo 2 a relaxação corre sobre uma malha em
   repouso (§5.2-bis). O aro explica as colunas `3.5R`/`4R`; o **interior** é a lista duplicada.
⇒ **Um port cujo `limite_3.5R` não seja `≈ 0` e cujo `fora_4R` não seja exactamente `0` no Local tem
o aro LIVRE**: ou cria restrições só até ao início da banda (`R(1+L·F)`) em vez de até ao limite
(`R(1+L)`), ou o `w` não chega a `0` dentro do conjunto restringido, ou `φ` não multiplica a
correcção do lado do vértice do aro. Régua directa: `limite_3.5R` tem de ler `≈ 0` e `fora_4R`
exactamente `0` no Local — e `> 0,03` nos dois no Global.
⚠️ **Mas um port com o aro CERTO e o interior igual ao Global tem outra coisa, e é a §5.2-bis:** as
colunas do aro batem com a lista simples, e o interior não. As duas leituras separam-se pela coluna,
não pelo passo.

### §10.3 — A experiência das varreduras (2026-09-06) — o lado MEDIDO da §5.2-bis

Medição nossa, com a nossa lei, variando **só** o número de varreduras de relaxação por passo (M):

| traço | varreduras que reproduzem o oráculo |
|---|---|
| arrastar radial **Global** (12 passos) | **5** — erro `≤ 4 %` em 12 passos × 5 colunas |
| arrastar radial **Local** (12 passos) | **10** — erro `≤ 3 %`, e só a `10` aparece o **pico-e-recuo** (máximo no passo 8, recuo até ao 12); a `5` a curva é monótona crescente |
| agarrar radial **Local** (2 passos) | cruza entre **9** e **10** |

⭐⭐ **O botão parte o corpus exactamente na linha *Local* / não-*Local*** (50 fixtures, `5` contra
`10` varreduras, erro relativo por traço): *Local* (38) — **27 melhoram**, 7 ficam ao bit em `0` (os
de um passo de força, §5.2-bis), 4 pioram; *Global* (2) — **os dois pioram**; *Dynamic* (10) — **9
pioram**. Ordens de grandeza, não afinação: arrastar *Local* `1,253 → 0,071`; arrastar *Global*
`0,175 → 0,565`.

⭐⭐⭐ **A prova que não é amplitude ajustada são as CONTAGENS de vértices movidos, que são inteiros:**
a `10` varreduras oito traços *Local* passam a mover **exactamente** o número do oráculo, e nos
traços de um e dois passos — onde não há acumulação possível — o alcance salta para o dele: agarrar
`869 → 1307` (oráculo `1324`), expandir `597 → 840` (`848`), arrastar `1050 → 1428` (`1438`). *Numa
relaxação sequencial o alcance por passo É o número de passagens ⇒ a contagem de movidos MEDE as
passagens, e ela diz `~2×` no ramo Local dentro de UM passo de pincel.*

⛔ **Duas coisas que esta medição NÃO explica** (e que não se devem misturar com ela): o Snake Hook
de 2 passos e o apertar-ponto *Local* pioram a `10` — no Hook o pico do port não está sob o cursor,
que é defeito de LOCALIZAÇÃO e não de amplitude; e na esfera os modos que não são arrasto erram em
*Dynamic* sem que as varreduras lhes toquem.

### §10.4 — ONDE está o pico (2026-09-06) — o lado MEDIDO da §4.3 do Snake Hook

A sonda por passo imprime a distância do vértice mais deslocado ao cursor **daquele passo**, em
raios (M):

| traço | pico do port (centro no cursor) | pico do oráculo |
|---|---|---|
| arrastar radial *Local*, passo 8 | `0,82R` | `0,82R` |
| arrastar radial *Local*, passo 11 | `1,02R` | `1,02R` |
| gancho radial *Local*, passo 2 | `0,05R` | **`0,86R`** |
| gancho radial *Local*, passo 3 | `0,24R` | **`0,91R`** |

⇒ **o arrasto está no sítio certo e o gancho não:** o pico do oráculo fica onde o pincel **estava**.
No 1.º passo simulado o vértice mais deslocado do oráculo é **o do pen-down** (`max = c0`), que é o
que a §4.3 prevê com o centro atrasado (`δ` anterior `= 0`).

⭐ **Medido pelo port com o centro atrasado (mutação de uma linha), 7 traços de gancho de 7 melhoram**
(`err_max/max_oráculo`, exemplos: `1passo` `0,999 → 0,467`; `2passos_origem` `0,700 → 0,324`;
o traço longo `0,162 → 0,129`), e a **contagem de movidos** do traço de um passo vai de `1040` para
`1434` contra `1452` do oráculo — outro inteiro a convergir.

⛔ **E o resíduo que sobra NÃO é a força da âncora** (varrida de `0,20` a `1,00` no traço de um
passo): a `0,35` — o valor que a §3.2 já dá — a amplitude bate (`0,4935` contra `0,4894`) e sobra
`0,2036` de erro; a `0,50` o erro desce e a **amplitude estoura `25 %`**. Nenhum valor torna o traço
exacto ⇒ a constante da espec está certa e o resíduo é de forma.

⚠️ **Quanto à forma que falta: no alvo NÃO existe eixo, plano nem limite de profundidade próprios do
Snake Hook** (leitura integral da fase de gesto, 2026-09-06). O que dá forma ao gancho, e que um port
pode não ter, é o par: **(a)** o centro atrasado desta secção e **(b)** a distância medida sobre as
posições **actuais** (§4.3) — juntos fazem o material já puxado viajar com o centro. O «plano de
profundidade» da §4.3 é do **delta** (a des-projecção do cursor), e vale para os oito modos; a queda
por-vértice é a distância comum ao centro, com a forma de queda que o pincel tiver — a **esférica**
por omissão, ou a *Projected* (medida no plano da vista) quando o artista a escolhe. É uma opção do
pincel, comum aos oito modos, **não** uma lei do modo.

### §10.5 — Os dois traços de APERTO por passo (2026-09-06, a pedido do I)

`plano_apertar_ponto_radial_local_origem` e `plano_apertar_linha_radial_local_origem`: 12 passos
cada, corridas-prefixo com `prova_do_fatiamento = 0,000000`, mesmo método e mesmo caminho (pen-down
na origem) da §10.2. Entregues porque os dois modos de aperto são exactos no traço de um passo e
divergem no fim do traço inteiro — só o dump por passo diz **em que** passo nasce.

⭐ **O que o rastreio já mostra, e o estado final escondia** — `|u|` sob o pen-down, passos 2..12:

| modo | `|u|` sob o pen-down, por passo |
|---|---|
| **aperto de PONTO** | `0,093 · 0,184 · 0,118 · 0,106 · 0,197 · 0,208 · 0,201 · 0,187 · 0,160 · 0,149 · 0,154` |
| **aperto de LINHA** | `0,000 · 0,001 · 0,001 · 0,002 · 0,005 · 0,006 · 0,005 · 0,004 · 0,004 · 0,003 · 0,003` |

⚠️ **O aperto de ponto NÃO é monótono** — sobe, **desce** no passo 4, volta a subir e desce de novo:
a força aponta para o **cursor**, que se afasta a cada passo, logo o vértice é puxado e depois
largado, e o pico anda com o cursor. Uma lei que integre monotonamente ultrapassa (é o sinal que o I
mede no fim do traço). ⚠️ **E o aperto de LINHA quase não move o pen-down** (`≤ 0,006`, contra
`0,10` no vizinho a `1R`): ele aperta contra a **linha** do traço, e o que está SOBRE a linha já lá
está.

---

### §10.6 — O CONTROLO do par de força do aperto (corrida NOVA do oráculo, 2026-09-06)

`plano_apertar_ponto_radial_local_origem_fraco` — **`.deformado` + `.porpasso` + `.porpasso.rastreio`**,
12 passos, `prova_do_fatiamento = 0,000000`. É o **mesmo** traço de aperto de ponto da §10.5 (mesma
malha, mesmo caminho a partir da origem, mesma área *Local*, mesmos limite/banda/massa/amortecimento),
com **UMA** coisa mudada: **força `1,0 → 0,2`**. Existe para ser a outra metade do par do §5.2-ter.

| | força `1,0` | força `0,2` |
|---|---|---|
| impulso máximo por passo (`10·α·dt/massa`) | `0,100` = **`2,1×` a aresta** | `0,004` = **`0,085×` a aresta** |
| vértices movidos no fim | `2 145` | `2 029` |
| máx `|u|` no fim | `0,303401` | `0,004082` |
| **quadriláteros invertidos**, passos 2 / 3 / 12 | **`10` / `18` / `52`** | **`0` / `0` / `0`** |
| **assimetria de espelho ÷ `|u|max`**, passos 2 / 3 / 12 | `0,000` / **`0,675`** / `1,060` | `0,000` / `0,103` / `0,144` |
| pior compressão, **só sobre pares que são ARESTAS** (`D/ℓ`, dos 12 passos) | `0,0523` ⇒ factor `−18,1` | `0,8304` ⇒ factor `−0,20` |
| pior compressão, **sobre TODOS os pares da construção** (a régua larga, §5.2) | `0,0153` ⇒ factor `−64,4` | `0,8304` ⇒ factor `−0,20` |

⚠️ **Para que serve, do lado do port:** ela é a régua por vértice que os apertos ainda admitem. Num
traço em que a malha nunca se inverte, o aperto é tão comparável quanto o arrasto; a partir do
momento em que se inverte, a barra tem de ser a do gate 20, não a por vértice. ⛔ **Não a use como
prova de que o aperto está certo** — ela prova o contrário do que parece: prova que o que separava as
duas leituras era a inversão, não uma lei.

### §10.7 — Os dois traços de FORÇA NORMAL por passo (corrida NOVA do oráculo, 2026-09-06, a pedido do I)

`plano_empurrar_radial_local_origem` e `plano_inflar_radial_local_origem` — **`.deformado` +
`.porpasso` + `.porpasso.rastreio`**, 12 passos cada, `prova_do_fatiamento = 0,000000` nos dois
(26 corridas-prefixo: `k = 1..12` de cada traço mais a corrida inteira da mesma sessão).
⚠️ **Pen-down na ORIGEM**, pela mesma razão das §10.2/§10.5: o centro da área *Local* de um traço
scriptado é a **localização inicial** guardada, e um traço cujo pen-down não é a origem depende de o
sobrevoo ter chegado (errata do §10). ⛔ Não confunda estes dois com os `plano_empurrar_radial_local`
e `plano_inflar_radial_local` do corpus (pen-down em `x = −0,3`): são o **mesmo gesto** com outro
ponto de partida, e os números abaixo são os destes.

`|u|` **depois** do passo `k`, dos vértices de repouso nomeados (deslocamentos perpendiculares ao
traço a partir do pen-down) e do vértice mais próximo do cursor do passo:

| passo | Push: pen-down | Push: 1R | Push: 3,5R | Push: sob o cursor | Inflate: pen-down | Inflate: 1R | Inflate: 3,5R | Inflate: sob o cursor |
|---|---|---|---|---|---|---|---|---|
| 2 | `0.06543` | `0.00050` | `0.00000` | `0.06990` | `0.09347` | `0.00072` | `0.00000` | `0.09986` |
| 3 | `0.17038` | `0.00423` | `0.00000` | `0.18733` | `0.22771` | `0.00857` | `0.00000` | `0.24879` |
| 4 | `0.22446` | `0.02977` | `0.00000` | `0.24728` | `0.26348` | `0.05248` | `0.00000` | `0.28459` |
| 5 | `0.23493` | `0.07046` | `0.00001` | `0.25578` | `0.26706` | `0.10238` | `0.00001` | `0.29137` |
| 6 | `0.23822` | `0.10596` | `0.00003` | `0.25839` | `0.26888` | `0.13989` | `0.00004` | `0.30880` |
| 7 | `0.23968` | `0.13452` | `0.00007` | `0.24698` | `0.27012` | `0.16933` | `0.00010` | `0.31915` |
| 8 | `0.23848` | `0.15728` | `0.00013` | `0.23497` | `0.27060` | `0.19245` | `0.00018` | `0.32474` |
| 9 | `0.23546` | `0.17465` | `0.00020` | `0.22357` | `0.27017` | `0.20985` | `0.00026` | `0.32592` |
| 10 | `0.23108` | `0.18680` | `0.00026` | `0.23925` | `0.26880` | `0.22213` | `0.00035` | `0.32387` |
| 11 | `0.22561` | `0.19418` | `0.00032` | `0.23761` | `0.26641` | `0.23007` | `0.00043` | `0.31358` |
| 12 | `0.21945` | `0.19761` | `0.00036` | `0.23225` | `0.26294` | `0.23452` | `0.00049` | `0.31013` |

⭐ **O que eles dizem, e que o traço inteiro esconde:**
1. **O passo 2 confirma a razão `2R` ao bit** — Push `0,06543` contra Inflate `0,09347` dá
   `0,7000` = `2 · 0,35` (§10.1). *No 1.º passo simulado a normal da área e a normal do vértice são a
   MESMA coisa (a folha está plana e em repouso): os dois traços só divergem a partir do passo 3.*
2. **O pen-down do Push satura e RECUA** (`0,2397` no passo 7 → `0,2195` no passo 12) e o do Inflate
   **fica** (`0,2701` → `0,2629`) — ⛔ e a diferença não é o módulo da força: é que a normal da área é
   uma média sobre meio raio (§4.2-bis), logo o Push empurra ao longo de uma direcção que **roda com
   a vala** enquanto o Inflate empurra ao longo da normal **do próprio vértice**.
3. **O aro está preso nos dois** (`3,5R` fica em `0,0004`/`0,0005` ao fim de 12 passos, `4R` em zero
   exacto) — o mesmo aro do §10.2, ⇒ estes dois traços **não** medem a fronteira, medem o interior.
4. **A frente a `1R` cresce monotonamente nos dois** e **quase alcança** o pen-down do Push no passo
   12 (`0,1976` contra `0,2195` — ⛔ não o ultrapassa em passo nenhum dos doze) — é a onda que a relaxação leva para fora, e ela é
   a coluna que mais depende do número de passagens (§5.2-bis).

---

## §11 — Comportamento de borda, caso a caso (F salvo indicação)

| caso | o que o alvo faz |
|---|---|
| dois vértices coincidentes numa restrição | correcção zero (sem divisão por zero) |
| par estrutural COMPRIMIDO (`D < ℓ`) | ⚠️ **sem tecto**: a correcção troca de sinal e cresce como `ℓ/D`; medido `−18,1` num aperto (§5.2) |
| vértice **exactamente** sobre o cursor, num modo de aperto | direcção nula ⇒ **força zero** (sem `NaN`, sem direcção de reserva, sem saltar o vértice); a um epsilon dali recebe a força inteira (§4.2) |
| vértice exactamente **sobre o plano** de falloff, no aperto de ponto com *Force Falloff = Plane* | o mesmo: distância assinada zero ⇒ direcção nula ⇒ força zero |
| retalho já invertido debaixo do cursor | nada o desfaz: não há detecção de inversão, nem tecto de deslocamento, nem corte ao ultrapassar o alvo (§5.2-ter) |
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
- ⚠️ **A área *Local* paga o DOBRO da relaxação** (a lista vem em duplicado, §5.2-bis): `10`
  projecções por restrição e por passo, e o dobro da memória de restrições. Não é uma escolha de
  desempenho do alvo — é o preço do mecanismo, e um port fiel paga-o.

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
| 8 | **O padrão de restrições** numa grelha regular: 4 + 2 + 4 por vértice interior, sem duplicados **DENTRO de uma construção** (⚠️ o conjunto do *Local* tem a lista inteira **duas** vezes — gate 16) | contagem exacta | §3.1 · §5.2-bis |
| 9 | **5 varreduras, `0,6`, meio para cada lado**: numa restrição isolada entre dois vértices livres com `φ = 1`, o erro após `k` varreduras é `(1 − 0,6)^k` do inicial | `0,4^5 = 0,01024 ± f32` | §5.2 |
| 10 | **Damping é retenção**: com damping `d`, um vértice livre em voo perde `d` da velocidade por passo; com `d = 1` pára | `f32` | §5.3 · fixture `plano_arrastar_radial_local_amort05` |
| 11 | **O 1.º passo nunca simula**; o passo seguinte sim | exacta | §1 |
| 12 | **A força por passo das âncoras é zerada em todo o objecto a cada passo e reescrita**: no Snake Hook com a queda `f` (⇒ `0` fora do pincel), no Grab com `1` (radial) nas células afectadas — ⛔ **não** «o Grab não zera» | exacta | §4.3 |
| 18 | **O centro do Snake Hook está um passo ATRASADO**: no 1.º passo simulado o vértice mais deslocado é o do **pen-down**, não o que está sob o cursor; e ao longo do traço a distância do pico ao cursor é `≈` o passo do traço, não `≈ 0` | posição do `arg max`, em raios | §4.3 · §10.4 · fixtures `plano_gancho_radial_local_1passo`, `plano_gancho_radial_local_2passos_origem` |
| 13 | **Grab mede na malha de partida**: mover o pano não muda o conjunto agarrado | exacta | §4.3 |
| 14 | **A simetria é por passagem** e a 2.ª passagem vê a 1.ª | fixture com espelho | §6.6 |
| 15 | **Paridade com o oráculo** — **todos** os traços de §10 (⛔ **conte-os**, `ls fixtures/cloth/*.deformado.txt.gz \| wc -l`; esta célula já esteve parada em `51` com `54` no disco): a barra é a **discretização** e o **`f32`**: a nossa malha é a mesma (gerada pela mesma lei), logo a comparação é por vértice; a barra por vértice é a aresta × a diferença de ordem das restrições (Gauss–Seidel não comuta) — MEDIR primeiro a dispersão entre duas ordens nossas e usar essa dispersão como barra (⛔ não um epsilon de conforto; ⛔ não bit-parity — ADR-0162) | derivada por medição | §10 |
| 16 | **A lista do *Local* vem em duplicado**: no mesmo retalho e no mesmo traço, a contagem de restrições do conjunto *Local* é **exactamente `2×`** a que uma construção só produz (⚠️ `2×` é o caso de UMA passagem de simetria, que é o das fixtures; a lei geral é `n+1` cópias para `n` passagens — §5.2-bis), e a de *Global*/*Dynamic* é `1×`; e a régua de comportamento é a **CONTAGEM DE VÉRTICES MOVIDOS** num traço de âncora de UM passo simulado (inteiro, sem acumulação possível), que tem de bater a do oráculo | contagem exacta (inteiros dos dois lados) | §5.2-bis · §10.3 · fixtures `plano_agarrar_radial_local_1passo`, `plano_expandir_radial_local_1passo`, `plano_arrastar_radial_local_2passos` |
| 19 | **O aperto INVERTE no 1.º passo simulado, e o arrasto não**: no mesmo retalho, mesmo caminho e mesma força, contar os quadriláteros de orientação invertida depois do 1.º passo simulado — aperto de ponto `> 0`, arrasto `= 0`. E a versão de força fraca do MESMO traço tem de dar `0` nos doze passos | contagem exacta (inteiros; o oráculo dá `10` / `0` / `0`) | §4.2 · §5.2-ter · fixtures `plano_apertar_ponto_radial_local_origem`(+`_fraco`), `plano_arrastar_radial_local_origem` |
| 20 | **A barra dos apertos é a da ORDEM, e mede-se, não se escolhe**: a assimetria de espelho da NOSSA saída dividida pelo maior `|u|` não pode passar a do ORÁCULO no mesmo passo (`0,675` no passo 3 do aperto de ponto); e a paridade por vértice do gate 15 **não se aplica** a um passo com faces invertidas — ali a barra é esta. ⛔ Barra derivada do oráculo, ⛔ nunca um epsilon de conforto | a do oráculo, passo a passo | §5.2-ter · §10.6 |
| 21 | **Fora da inversão o aperto é tão comparável quanto o arrasto**: sobre a fixture de força fraca (zero faces invertidas nos 12 passos) a paridade por vértice do aperto de ponto tem de ficar no mesmo patamar da do arrasto — se ficar pior, o defeito **não** é a ordem e há lei em falta | o erro relativo do arrasto no mesmo traço | §10.6 · fixture `plano_apertar_ponto_radial_local_origem_fraco` |
| 17 | **É só o ramo *Local***: a mesma experiência que melhora os traços *Local* tem de **piorar** os *Global* e os *Dynamic* — um port que dobre a relaxação em toda a parte passa o gate 16 e reprova aqui | sinal do erro relativo, em **todos** os traços de §10 (⛔ contados, nunca citados de memória) | §5.2-bis · §10.3 |
| 22 | **O deslocamento do cursor é a PROJECÇÃO, e só o arrasto não o usa**: sobre as fixtures de esfera, o `δ` que alimenta a âncora do Agarrar, a do Snake Hook, a normal do plano de queda e o `x̂` do referencial tem componente **exactamente zero** ao longo do eixo da vista em **todos** os passos; e a direcção do arrasto no mesmo traço **tem** componente de profundidade, com o ângulo entre as duas a reproduzir, nos **11** passos, `15,83° · 12,61° · 9,42° · 6,27° · 3,13° · 0°` **e o espelho** (`3,13° · 6,27° · 9,42° · 12,61° · 15,83°`) — a sequência é simétrica porque o caminho é simétrico em relação ao topo da esfera, ⛔ e não é uma tabela a copiar: sai de `atan(Δy/Δx)` sobre a esfera unitária ⇒ ⚠️ **duas metades**, e a segunda é o controlo: um port que use a diferença dos pontos 3D em toda a parte passa a 1.ª e reprova a 2.ª, e um que projecte em toda a parte faz o inverso | `0` exacto numa metade · a tabela do §4.3 na outra | §4.3 · §4.6 · fixtures de esfera |
| 23 | **A normal do Push é REAVALIADA sobre a malha DEFORMADA**: congelá-la na normal de repouso tem de **mudar** `plano_empurrar_radial_local_origem` acima da barra do gate 15, e tem de deixar `plano_empurrar_radial_local_1passo` **byte-idêntico** ⇒ ⚠️ **duas metades**, e a segunda é o controlo (no 1.º passo simulado a malha ainda está em repouso, logo lá as duas leis coincidem por construção) | mutação: A/B com a normal congelada | §4.2-bis · §10.7 |
| 24 | **A razão `2R` do Push, e a igualdade Push/Inflate no 1.º passo simulado**: no passo 2 dos dois traços do §10.7 o vértice do pen-down move `0,06543` e `0,09347`, razão `0,7000 = 2·R`; e a divergência entre os dois só pode começar no passo **3** — se começar no 2, o port está a ler duas normais diferentes numa folha plana em repouso, onde elas são a mesma | razão `2R ± f32` · igualdade de direcção no passo 2 | §4.2-bis · §10.1 · §10.7 |

---
