---
titulo: "Cloth (W10) — a espec do COMPORTAMENTO: para onde cada uma das oito deformações APONTA, só de fonte pública"
tags: [modulo/3d, tipo/espec, status/ativo, wave/W10]
status: ativo
modulo: 3D
atualizado: 2026-09-05
resumo: "Os oito tipos de deformação da referência, um a um, com a DIREÇÃO de cada um; a geometria do gesto (a referência NUNCA fixa um vetor de mundo — o referencial é um controlo declarado); a fronteira da área simulada (banda graduada em 0,75 do limite, DESENHADA A TRACEJADO, e o pino DESLIGADO por omissão); o modelo, no que dele é público; e a lista do que só código ou medição responde."
---

# Cloth (W10) — a espec do comportamento

> **Alvo:** a ferramenta de escultura de tecido do Blender (GPLv2+), mais os apps de
> referência do §E. **Degrau deste doc: espec de COMPORTAMENTO a partir de fonte
> PÚBLICA apenas.**
>
> ⛔ **Nenhum código-fonte do alvo foi aberto por este agente** — nem repositório, nem
> diff, nem revisão de código, nem excerto. As fontes são: **manual oficial**,
> **documentação da API pública**, **notas de versão**, **blog dos programadores** e
> material descritivo de terceiros. A triagem de licença é de outro agente.
>
> ⛔ **Este documento não contém identificador interno do alvo** (função, variável,
> ficheiro, constante de enumeração, sigla de estrutura de dados). Todo nome aqui é um
> **rótulo que o artista lê na tela**, que é facto funcional publicado
> ([SKILL §4.1.3 e §4.1.13](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md)).
>
> **"Este documento descreve comportamento; não contém expressão do alvo."**
>
> **Filtragem §4.3:** executada em 2026-09-05 — varredura própria por identificador
> interno, sigla de estrutura, `snake_case` e caminho de ficheiro: **zero achados**
> (os únicos `snake_case` do doc são fragmentos de URL do manual público).
> **Sweep §7.1:** ⚠️ **não é executável para este alvo** — o `cleanroom-sweep.sh` exige
> uma VASSOURA, e a única que existe no repo é a de outro alvo (ela corre limpa aqui,
> mas isso é **controlo negativo, não prova**). ⛔ **A vassoura deste alvo não pode ser
> escrita por mim:** ela é uma lista dos identificadores INTERNOS do alvo, e escrevê-la
> exigiria tê-los lido — exatamente o que a parede proíbe. **Quem a escreve é o agente
> da triagem de licença**, que tem autorização para abrir o fonte.
>
> ⚠️ **Proveniência honesta (SKILL §4.3):** três fontes recusaram leitura direta
> (`403`/JS) — o blog dos programadores, o portal de revisões arquivado e o portal de
> commits. O blog foi recuperado por descarga direta e está citado **verbatim**; as duas
> últimas só existem aqui **através do resumo de um motor de busca**, e cada linha que
> depende delas está marcada **⚠️ (via resumo de busca)**. Nenhuma delas sustenta
> sozinha uma afirmação estrutural.

---

## §0 — ⭐⭐⭐ A leitura em uma linha

**Na referência, «que direção» e «que vértices» são DOIS controlos separados, e nenhum
dos dois é um vetor de mundo.** O tipo de deformação escolhe **o alvo geométrico**
(um ponto, uma linha, a normal, a área de repouso, o cursor); o *Force Falloff* escolhe
a **forma espacial** do peso (esfera ou plano); e o referencial em que o gesto vive é,
ele próprio, um controlo declarado da casa (*Sculpt Plane*: plano da área · plano da
vista · eixo global). **Cinco dos oito tipos não podem ser expressos por uma translação
uniforme**, porque o alvo deles não é um deslocamento — é um ponto, uma linha, uma
normal ou um comprimento de repouso.

⇒ isto é a resposta direta ao `0.000e0` da
[auditoria §0-bis](03_auditoria_2026-09-05.md): não falta «mais um modo» — falta a
**separação entre alvo e peso**, que é o que torna sete dos oito modos exprimíveis.

---

## §A — Os tipos de deformação, um a um

### §A.1 — O censo, com a versão em que cada um entrou

A referência oferece **oito**. Sete nasceram na primeira versão da ferramenta (2.83,
2020) e **um** chegou depois (2.91). ⚠️ Uma nota de versão diz *«novo pincel de tecido
Grab que usa restrições»* na mesma entrada que anuncia o Snake Hook — **o Grab não é
novo em 2.91, é RE-IMPLEMENTADO**: ele já está no manual da 2.83, e o que 2.91 lhe faz é
trocar o mecanismo por restrições. *Ler aquela linha como «entrou em 2.91» dá nove
modos, e são oito.*

| # | tipo (rótulo na tela) | entrou | o que o manual declara que acontece |
|---|---|---|---|
| 1 | **Drag** *(o de omissão)* | 2.83 | *«Simula puxar o pano PARA o cursor, como pôr um dedo numa toalha de mesa e puxar.»* |
| 2 | **Push** | 2.83 | *«Simula empurrar o pano PARA LONGE do cursor, como pôr um dedo numa toalha de mesa e empurrar.»* |
| 3 | **Pinch Point** | 2.83 | *«Simula puxar o pano PARA DENTRO DE UM PONTO.»* |
| 4 | **Pinch Perpendicular** | 2.83 | *«Simula puxar o pincel PARA DENTRO DE UMA LINHA.»* |
| 5 | **Inflate** | 2.83 | *«Simula ar soprado por baixo do pano, de modo que o pano LEVANTA.»* |
| 6 | **Grab** | 2.83 (re-implementado com restrições em 2.91) | *«Simula pegar no pano e movê-lo.»* |
| 7 | **Expand** | 2.83 | *«Simula ESTICAR o pano para fora.»* |
| 8 | **Snake Hook** | **2.91** | *«Simula mover o pano sem produzir artefatos na superfície, e cria dobras de aspeto mais natural do que qualquer um dos outros modos de deformação. Isto consegue-se ajustando a força das restrições de deformação a cada passo do pincel, para afetar o resultado da simulação o menos possível.»* |

Fontes: [manual, página da ferramenta](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html)
· [manual 2.83 (sete modos)](https://docs.blender.org/manual/en/2.83/sculpt_paint/sculpting/tools/cloth.html)
· [manual 2.93 (oito modos)](https://docs.blender.org/manual/en/2.93/sculpt_paint/sculpting/tools/cloth.html)
· [notas 2.83](https://developer.blender.org/docs/release_notes/2.83/sculpt/) (*«tem 7 modos de deformação com 2 tipos de falloff»*)
· [notas 2.91](https://developer.blender.org/docs/release_notes/2.91/sculpt/).

### §A.2 — ⭐⭐⭐ PARA ONDE APONTA (o item que decide tudo)

⚠️ **Nenhuma fonte pública dá a fórmula.** O que segue é a leitura das descrições
oficiais, com a coluna `estado` a separar **DECLARADO** (a frase está no manual) de
**INFERIDO** (a frase não está, e a evidência ao lado é o que a sustenta).

| tipo | alvo geométrico | tem componente FORA do plano tangente? | muda a área de repouso? | estado / evidência |
|---|---|---|---|---|
| **Drag** | **um PONTO móvel** — o cursor de agora. Os vértices são puxados *para* ele | ⚠️ só o que a curvatura der: numa toalha plana o ponto está no plano, e o campo é **radial dentro do plano**, ⛔ **não** uma translação uniforme | não | **DECLARADO** (*«puxar o pano PARA o cursor»*) |
| **Push** | **o mesmo ponto, com sinal trocado** — afastar dele | idem, radial para fora | não | **DECLARADO** |
| **Pinch Point** | **um PONTO** para onde o pano converge | não (converge sobre a superfície e o material é que se enruga) | não | **DECLARADO** + a ferramenta-irmã de filtro declara *«aperta o pano para o ponto onde o cursor ESTAVA quando o filtro começou»* ⇒ o ponto é **congelado**, não o cursor de agora ([manual do filtro](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/tools/cloth_filter.html)) |
| **Pinch Perpendicular** | **uma LINHA** (o eixo do traço) para onde o pano converge | não | não | **DECLARADO** (*«puxar o pincel para dentro de uma LINHA»*) · um tutorial descreve-o como *«toma a direção do traço e aperta a geometria vizinha ao longo dessa direção»* ([RenderGuide](https://renderguide.com/blender-cloth-brushes-tutorial/)) |
| **Inflate** | **a NORMAL da superfície**, para fora | ⭐ **SIM, e é a razão de existir do modo** (*«o pano LEVANTA»*) | não | **DECLARADO** |
| **Grab** | **o cursor no plano da vista** — a pegada **congelada** no pen-down segue o rato | por omissão não; o controlo *Normal Weight* (omissão `0,0`) inclina-o para a normal | não | **DECLARADO** para o pincel Grab base (*«arrasta a geometria pelo ECRÃ, seguindo o cursor»* + *«só move os vértices que estão sob o raio NO INÍCIO do traço»*) — **INFERIDO** que a variante de tecido herda a semântica ([manual do Grab](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/grab.html)) |
| **Expand** | **o COMPRIMENTO DE REPOUSO** — não há deslocamento a apontar para lado nenhum | n/a (a dobra nasce da flambagem, não de um empurrão) | ⭐ **SIM** | **INFERIDO.** O manual do pincel diz só *«esticar o pano para fora»*; a evidência é a ferramenta-irmã de filtro, que declara *«Expand: expande as DIMENSÕES do pano»* — dimensão é repouso, não posição |
| **Snake Hook** | **o cursor**, com a pegada **re-pegada continuamente** (não congelada) | idem Grab | não | **DECLARADO** para o Snake Hook base (*«puxa os vértices com o movimento do pincel; durante o traço a geometria é dinamicamente APANHADA e LARGADA»*) + **DECLARADO** no manual de tecido que a força das restrições é reajustada **a cada passo do pincel** |

⭐⭐⭐ **A conta que interessa:** dos oito, **três** (Inflate · Expand · e o Push quando a
superfície não é plana) produzem relevo **sem depender da curvatura**, e **quatro**
(Drag · Push · Pinch Point · Pinch Perpendicular) são campos **de convergência ou
divergência a partir de um alvo**, não translações. **Só dois — Grab e Snake Hook — são
«segue o rato», e mesmo esses diferem entre si por QUEM está na pegada.**

### §A.3 — O que modula cada um

Os controlos abaixo são **ortogonais ao tipo**: valem para os oito. Faixas e valores de
omissão são factos publicados na documentação da API pública.

| controlo (rótulo na tela) | o que a documentação diz que ele é | faixa | **omissão** |
|---|---|---|---|
| **Force Falloff** | *«Forma usada no pincel para aplicar força ao pano.»* **Radial** = *«aplica a força como uma esfera»* · **Plane** = *«aplica a força como um plano»* | 2 valores | **Radial** |
| **Cloth Mass** | *«Massa de cada partícula da simulação.»* | `0,01`–`2` | **`1,0`** |
| **Cloth Damping** | *«Quanto as forças aplicadas são PROPAGADAS através do pano.»* | `0,01`–`1` | **`0,01`** |
| **Soft Body Plasticity** | *«A quantidade em que o pano preserva a forma original, agindo como um corpo mole.»* | `0`–`1` | **`0,0`** |
| **Simulation Area** | que parte da malha simula: **Local** (raio fixo à volta do pincel) · **Global** (a malha inteira) · **Dynamic** (*«a área ativa MOVE-SE com o pincel, ainda limitada por um raio fixo»*) | 3 valores | **Local** |
| **Simulation Limit** | *«Fator, relativo ao tamanho do raio, para limitar os efeitos da simulação.»* | `0,1`–`10` | **`2,5`** |
| **Simulation Falloff** | §C | `0`–`1` | **`0,75`** |
| **Pin Simulation Boundary** | §C | booleano | **DESLIGADO** |
| **Use Collisions** | colisão com objetos de cena que declarem física de colisão | booleano | **DESLIGADO** |
| **Persistent** / *Set Persistent Base* | *«permite que o pincel NÃO acumule deformação a cada traço… simular sempre a partir da mesma forma inicial, aplicando forças diferentes»* | booleano | (não publicado) |

Fonte das faixas e omissões: [documentação da API pública do pincel](https://docs.blender.org/api/current/bpy.types.Brush.html)
(⚠️ citada aqui **só pelos rótulos de UI e pelos números**; os identificadores da API
ficam de fora por decisão de parede).

⚠️ **`Cloth Damping` com omissão no PISO da própria faixa (`0,01` de `0,01`–`1`) é um
facto forte**: a referência entrega o pano **quase sem amortecimento**, e a descrição
dele não é *«quanto a velocidade decai»* — é *«quanto as forças se PROPAGAM»*, que é uma
grandeza de acoplamento, não de dissipação. ⇒ ler aquele número como um amortecimento de
Rayleigh é uma tradução, não um facto (ver §F).

### §A.4 — ⭐⭐ Qual deles a documentação diz que faz as dobras mais naturais, e porquê

**Snake Hook**, e o manual diz **porquê**, verbatim:

> *«Simula mover o pano **sem produzir quaisquer artefatos na superfície** e cria dobras
> de aspeto **mais natural do que qualquer um dos outros modos de deformação**. Isto é
> conseguido **ajustando a força das restrições de deformação a cada passo do pincel**,
> para afetar o resultado da simulação **o menos possível**.»*
> — [manual, secção *Deformation*](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html)

E o blog dos autores repete-o pelo outro lado, na lista de melhorias de 2.91:

> *«There is a new Snake Hook deformation mode, which moves the cloth **without creating
> artefacts in the simulation**.»*
> — [Pablo Dobarro, *Cloth Sculpting improvements in Blender 2.91*](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/)

⭐⭐⭐ **A lei escondida nessa frase é a mais importante de toda a espec:** o modo que dá
as melhores dobras é o que **interfere MENOS com o solver**. O gesto não impõe posições;
ele **modula a força de uma restrição**, e deixa a simulação decidir onde a matéria
acaba. *Um pincel que escreve a posição alvo de cada vértice está a fazer exatamente o
contrário do que a referência declara ser o caminho para a dobra natural.*

---

## §B — A geometria do gesto

⚠️ **Não existe declaração pública da fórmula.** Existe, porém, uma coisa mais forte para
o nosso caso: **a referência trata «em que referencial o deslocamento vive» como um
CONTROLO DECLARADO, não como uma constante do código.**

| pergunta | o que a documentação pública responde | fonte |
|---|---|---|
| O deslocamento é um vetor de ecrã? | Para a família «segue o rato», **sim, por omissão**: o pincel Grab base *«arrasta a geometria pelo ECRÃ, seguindo o cursor»* | [manual do Grab](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/grab.html) |
| Existe um referencial escolhível? | ⭐ **Sim, e é um controlo da casa: *Sculpt Plane*** — *«o plano em que a escultura acontece; por outras palavras, a **direção primária** em que os vértices se vão mover»*. Valores: **Area Plane** (*«a direção da NORMAL MÉDIA de todos os vértices ativos dentro da área do pincel — a direção depende da SUPERFÍCIE por baixo do pincel»*) · **View Plane** (*«esculpir no plano da vista 3D»*) · **X, Y, Z Plane** (eixo global) | [manual, definições de pincel](https://docs.blender.org/manual/en/latest/sculpt_paint/brush/brush_settings.html) |
| E o referencial pode ser congelado? | Sim: ***Original — Normal*** *«mantém a normal da superfície ONDE O TRAÇO FOI INICIADO, em vez da normal da superfície que está agora sob o cursor»*; e ***Original — Plane*** faz o mesmo para a origem do plano | idem |
| Há mistura explícita entre plano da vista e normal? | ⭐ **Sim: *Normal Weight*** — *«CONSTRANGE o movimento do pincel ao longo da normal da superfície… aplica-se aos pincéis Grab e Snake Hook»*, com omissão **`0,0`** (⇒ arrasto puro no plano da vista) e alcance `0`–`1` | [manual](https://docs.blender.org/manual/en/latest/sculpt_paint/brush/brush_settings.html) + [API pública](https://docs.blender.org/api/current/bpy.types.Brush.html) |
| E a superfície CURVA? | ⛔ **Nenhuma declaração pública sobre o pincel de tecido numa superfície curva.** O que existe é indireto e vale: o modo de omissão do plano de escultura da casa é o **plano da ÁREA**, cuja direção é declarada como *«dependente da superfície por baixo do pincel»* — ou seja, **a casa já rejeita, por omissão e por escrito, empurrar uma direção fixa através de uma superfície** | idem |
| A ferramenta-irmã (filtro) expõe o referencial? | ⭐⭐ **Explicitamente.** O filtro tem **Orientation** com três valores — **Local** / **World** / **View** — descritos como *«usar o eixo … para limitar a força **e definir a direção da gravidade**»*, mais **Force Axis** (*«aplicar a força ao longo do eixo selecionado»*). E o gesto do filtro é **escalar**: *«clique e arraste PARA LONGE do objeto para efeito positivo e PARA ELE para efeito negativo»* | [manual do filtro de tecido](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/tools/cloth_filter.html) |
| O peso espacial é separado da direção? | ⭐ **Sim, e é o *Force Falloff*:** *«forma usada no pincel para aplicar força ao pano»* — **esfera** ou **plano**. É a segunda metade do desenho: **tipo de deformação = alvo · Force Falloff = forma do peso** | [manual da ferramenta](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html) |
| Alguma nota sobre a evolução do falloff plano? | ⚠️ *(via resumo de busca)* uma mensagem de commit pública diz que o falloff de plano **existia na primeira versão** do pincel e **causava artefatos** nos modos de deformação por falta de funcionalidade no solver, e que passou a estar *«corretamente implementado usando as restrições de deformação»* | [commit público, mensagem](https://projects.blender.org/blender/blender/commit/c53b606) |

⭐⭐ **A escala da dobra não é do gesto — é da MALHA, e está escrito:**

> *«A resolução da topologia é a principal responsável pelo **tamanho das dobras** e pelo
> nível de detalhe da simulação. Portanto, uma topologia ótima e **uniformemente
> distribuída** é importante.»*
> — [manual, introdução à escultura de tecido](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/introduction/cloth_sculpting.html)

⇒ o mesmo documento avisa que a ferramenta é *«especialmente útil para criação de malha
base e para dobras e drapeados GRANDES»*, e que *«detalhe é possível, mas o desempenho
mais lento em malhas de alta resolução e a física de pano SIMPLIFICADA podem não levar a
resultados desejáveis»*. **A referência declara o próprio regime.**

---

## §C — A fronteira da área simulada

⭐⭐⭐ **A ordem histórica é o achado desta secção, e ela inverte a nossa:** a **banda
graduada existe desde o primeiro dia** (2.83 nasce com *dois* raios extra — um que limita
e outro que gradua); o **pino chegou onze meses depois, como opção, e nasce DESLIGADO**.

| controlo | o que a documentação declara | faixa | **omissão** | como é desenhado no cursor |
|---|---|---|---|---|
| **Simulation Limit** | *«O fator acrescentado, relativo ao tamanho do raio, para LIMITAR os efeitos da simulação.»* | `0,1`–`10` | **`2,5`** (⇒ `2,5 × raio`) | um anel à volta do cursor (a documentação de terceiros chama aos círculos cinzentos *«a área da simulação»*) |
| **Simulation Falloff** | *«A ÁREA onde aplicar o falloff da deformação aos efeitos da simulação. Esta definição é um FATOR DO Simulation Limit e é mostrada como uma LINHA TRACEJADA à volta do cursor.»* | `0`–`1` | **`0,75`** | ⭐ **linha TRACEJADA** |
| **Pin Simulation Boundary** | *«TRAVAR a posição dos vértices na área de falloff da simulação, para evitar artefatos e criar uma transição mais suave com as áreas não afetadas.»* | booleano | ⭐⭐ **DESLIGADO** | — |

**Aritmética derivada dos valores de omissão:** com raio `R`, a simulação termina em
`2,5 R`; a banda de transição começa em `0,75 × 2,5 R = 1,875 R` e vai até `2,5 R`.
⇒ **a faixa graduada mede `0,625 R` de largura, isto é, 62,5 % do raio do pincel**, e é
ela que o artista vê tracejada.

**Porque a banda existe — pelas palavras dos autores:**

> *«Thanks to the **fade areas** defined by the simulation falloff, the brush can
> **blend the simulated and not simulated mesh seamlessly**.»*
> — [Dobarro, 2.91](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/)

E, na mesma página, o pino aparece como **acréscimo**, não como o mecanismo:

> *«There is a new "pin simulation boundary" which allows creating **better anchored**
> stroke cloth brushes.»*

⚠️ **Nota de versão de 2.83, quando ainda não havia pino nenhum:** *«tem dois controlos
de raio adicionais para limitar a INFLUÊNCIA e o FALLOFF da simulação»* e *«vértices
mascarados são PREGADOS na simulação»* — ou seja, **desde o primeiro dia a pregagem é o
que a MÁSCARA do artista faz, e a fronteira da simulação é graduada**, não pregada.
([notas 2.83](https://developer.blender.org/docs/release_notes/2.83/sculpt/))

**Um terceiro mecanismo, vizinho e graduado, que a referência criou no mesmo dia e para
este fim:** a proteção automática da **fronteira topológica da malha**, que a nota de
2.83 apresenta explicitamente *«para proteger as fronteiras da malha ao esculpir tecido»*,
e que traz um controlo de **passos de propagação** *«para controlar a suavidade do
falloff perto das arestas da escultura»*. ⇒ ⭐ **até a fronteira do OBJETO recebe rampa,
com número de passos regulável — não um degrau.**

⇒ **Resumo para nós:** a referência tem **três** camadas na borda, e as três são
graduadas ou opcionais — (1) rampa de peso na faixa `0,75..1,0` do limite, sempre ligada;
(2) pino da mesma faixa, **opcional e desligado**; (3) rampa por passos junto à fronteira
topológica. A [auditoria §2](03_auditoria_2026-09-05.md) mede que temos **a (2) sempre
ligada e nenhuma das outras duas**.

---

## §D — O modelo, no que dele é público

⚠️ **A família do solver NÃO é pública.** Não há, em manual, notas de versão ou blog,
qualquer declaração de que seja projeção de restrições, mola-massa, dinâmica projetiva
ou um método de descida — e este doc **não vai inventar uma**. O que segue é o conjunto
completo do que está publicado.

| o que se sabe | a declaração pública | fonte |
|---|---|---|
| É **simplificado**, e assumidamente | *«O pincel de tecido usa um **Cloth Solver SIMPLIFICADO** para simular física de pano na malha sob o pincel»* · *«este pincel tem um **solver de física SIMPLES**»* | [manual 2.83](https://docs.blender.org/manual/en/2.83/sculpt_paint/sculpting/tools/cloth.html) · [notas 2.83](https://developer.blender.org/docs/release_notes/2.83/sculpt/) |
| ⭐⭐⭐ **O pincel NÃO escreve posições: ele deforma RESTRIÇÕES** | O controlo *Deformation Target* tem dois valores, e o segundo é declarado assim: **Geometry** *«a deformação do pincel desloca os vértices da malha»* · **Cloth Simulation** *«o pincel deforma a malha **deformando as restrições de uma simulação de pano**»* (omissão: Geometry) | [API pública](https://docs.blender.org/api/current/bpy.types.Brush.html) |
| ⭐⭐ **E o gesto vai para uma cópia de posições, não para a malha** | ⚠️ *(via resumo de busca)* o sumário público da revisão que trocou os modos de deformação para restrições descreve: quando o alvo é a simulação, a deformação do pincel é aplicada **às restrições do solver**, cabendo à simulação aplicar a deformação final; e os pincéis de deformação escrevem **um array de posições SEPARADO**, ao qual o solver acrescenta restrições, de modo que **os vértices reais só se movem ao resolver as restrições** | [revisão pública arquivada D8424](https://developer.blender.org/D8424) — ⛔ **o diff não foi lido** |
| A força das restrições é **modulada por passo do pincel** | *«…ajustando a força das restrições de deformação **a cada passo do pincel**, para afetar o resultado da simulação o menos possível»* | [manual](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html) |
| As restrições **nascem e ativam-se com o movimento** | *«a área de simulação dinâmica **inicializa e ativa as restrições e a simulação à medida que o pincel se move**. À medida que o traço é desenhado, **mais restrições são criadas e adicionadas ao solver**, mas a simulação **só corre usando as restrições que estão mais perto da ponta do pincel**»* | [Dobarro, 2.91](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/) |
| **Plasticidade** é preservação da forma **DEFORMADA** | *«quando o solver deforma a malha, ele **tenta preservar a forma DEFORMADA durante a simulação**. Isto ajuda a tornar alguns pincéis mais controláveis e previsíveis, e permite simular materiais diferentes»* (omissão `0,0`) | idem |
| **Gravidade** entra no solver, não depois | *«vértices mascarados são pregados na simulação, e ela aplica a **gravidade da escultura diretamente NO solver**»* · em 2.91, *«o pincel pode aplicar gravidade global e simular a malha inteira durante o traço»* | [manual 2.83](https://docs.blender.org/manual/en/2.83/sculpt_paint/sculpting/tools/cloth.html) · [notas 2.91](https://developer.blender.org/docs/release_notes/2.91/sculpt/) |
| **Colisões** são por raio lançado | *«as colisões do solver de pano em 2.91 são **baseadas em raycast**. Isto significa que podem colidir com qualquer tipo de geometria (mesmo não-manifold) e os vértices serão parados pela superfície do colisor… **Como desvantagem, se um vértice do pano estiver DENTRO do colisor, o solver não o conseguirá mover para fora.**»* | [Dobarro, 2.91](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/) |
| **Auto-colisão NÃO existe**, e o bloqueador é nomeado | *«As auto-colisões também estão planeadas. De momento, não podem ser implementadas eficientemente por razões técnicas»* — o obstáculo nomeado é o tamanho da folha da estrutura de aceleração espacial da escultura | idem |
| A **escala da dobra é a topologia** | §B | [introdução](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/introduction/cloth_sculpting.html) |
| A simulação **é isolada por região** | *(nota de versão)* o pincel cria as restrições **apenas nas regiões da estrutura espacial que são precisas**, de modo que a simulação fica isolada em malhas densas | [notas 2.83](https://developer.blender.org/docs/release_notes/2.83/sculpt/) |
| Outros pincéis podem **conduzir** o solver | em 2.91 os pincéis de pose e de fronteira ganham o alvo *Cloth Simulation*: *«a deformação que estes pincéis produzem permite criar efeitos comuns de tecido, como cortinas ou dobras de mangas… sem depender de colisões»*, e a intenção declarada é que **todos** os pincéis o suportem | [Dobarro, 2.91](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/) |

⛔ **O que NÃO é público, e portanto não está aqui:** a família do integrador; o número de
iterações ou sub-passos por passo do pincel; o passo de tempo; se existe (e qual é) um
modelo de dobra; a forma da curva da rampa de falloff; o que exatamente *Cloth Damping*
faz na equação. **Ver §F.**

---

## §E — Os outros apps

### §E.1 — Open source

| app | tem pincel/ferramenta de tecido? | o que é |
|---|---|---|
| **Blender** | ✅ **sim** — o alvo desta espec | pincel + filtro + alvo de deformação para outros pincéis |
| **Nomad Sculpt** *(proprietário, mas manual público e completo)* | ⛔ **NÃO.** A lista de ferramentas do manual oficial tem Clay · Brush · Move · Drag · Smooth · Facegroup · Vertex · Mask · Paint · Smudge · Flatten · Planar · Crease · Pinch · Trim · Split · Project · Layer · Inflate · Nudge · Stamp · Tube · Lathe · Extract · Carve — **nenhuma de tecido, dobra ou drapeado** | a comunidade fabrica «pincéis de dobra» com alfas, o que confirma a ausência ([manual](https://nomadsculpt.com/manual/tools)) |
| **SculptGL** | ⛔ não documentado | — |

⇒ **não se achou nenhum outro programa open-source com pincel de tecido funcional.**
A referência é, na prática, única na sua categoria.

### §E.2 — ZBrush (proprietário — só manual, ⛔ nunca engenharia reversa)

⭐⭐ **O desenho é o OPOSTO do da referência open-source, e as duas leis publicadas no
manual dele são as mais úteis de todo este documento.**

O ZBrush não tem *um* pincel com oito modos: tem **treze pincéis nomeados** mais uma
variante de gizmo, todos por cima de um **motor de dinâmica** partilhado.

| pincel | o que o manual declara |
|---|---|
| **ClothNudge** | *«Move ligeiramente a superfície, causando ONDULAÇÕES.»* |
| **ClothPull** | *«ARRASTA a superfície. Bom para controlar como o pano DRAPEIA.»* |
| **ClothInflate** | *«Molda a superfície como se estivesse a cobrir uma ESFERA.»* |
| **ClothBall** | *«Semelhante ao Inflate; porém baseado no pincel Standard, e por isso NÃO infla ao longo das normais.»* |
| **ClothFold** | *«Amarrota; torce ligeiramente a superfície, para rugas e dobras.»* |
| **ClothHook** | *«Puxa o pano como se o tivesses agarrado e arrastado.»* |
| **ClothMove** | *«MUITO MENOS simulação, dando mais controlo a ti do que à dinâmica do pano. Haverá MENOS ESTICÃO com este pincel do que com o ClothPull.»* |
| **ClothPinch** | *«Puxa a superfície sobre si própria, criando um VINCO.»* (com inversão por modificador) |
| **ClothPinchTrails** | *«Repete o apertar ao longo do PERCURSO do traço»* — bom para rugas de almofada, feridas, cicatrizes |
| **ClothSlide** | *«LEVANTA a superfície e move-a, criando dobras à medida que se AMONTOA.»* |
| **ClothTwister** | *«Torce a superfície com movimento circular»*, com taxa de torção assinada |
| **ClothWind** | *«Cria um efeito como se o pano estivesse a ser SOPRADO. Útil para efeitos naturais em tecidos soltos.»* |
| **ClothDimple** | *«Junta o pano, como um BOTÃO numa almofada.»* |
| **TransposeCloth** | o gizmo de transformação a correr sobre a simulação; *«ao escalar para baixo, a malha ONDULA»* |

⭐⭐⭐ **Lei publicada nº 1 — o orçamento de iterações é um controlo POR PINCEL, e o preço
está escrito:**

> *«Qualquer pincel de escultura pode ser convertido num pincel de tecido ativando o
> deslizador de **iterações de simulação** do pincel. Este deslizador funciona como uma
> **percentagem** do valor global… **Definições mais baixas tornam o pincel mais
> RESPONSIVO mas podem resultar em MAIS ESTICÃO da malha. Inversamente, definições mais
> altas dão maior PRECISÃO mas menos responsividade.** Um valor de 0 desativa a simulação
> para esse pincel.»*
> — [manual do ZBrush, *Cloth Brushes*](https://help.maxon.net/zbr/en-us/Content/html/user-guide/3d-modeling/cloth-simulation/cloth-brushes/cloth-brushes.html)

⇒ **é exatamente o eixo que a [auditoria §1.2](03_auditoria_2026-09-05.md) mede** —
orçamento contra esticão —, e a referência proprietária **entrega-o ao artista como
deslizador** em vez de o congelar numa constante.

⭐⭐⭐ **Lei publicada nº 2 — a simulação COMPETE com o verbo, e o manual diz em que caso:**

> *«…o comportamento do pincel Dam Standard é que ele **tenta apertar**, mas como a
> **simulação tenta MANTER A ÁREA DA SUPERFÍCIE**, os dois efeitos tendem a **lutar um
> contra o outro**.»* · *«Como a simulação de pano tenta manter a malha igual, as
> alterações feitas pelo pincel **podem ser anuladas**. Além disso, o efeito dos alfas
> tende a ser reduzido.»*
> — idem

⇒ é a mesma física que a [auditoria §0-bis.2](03_auditoria_2026-09-05.md) nomeia por
*membrane locking*, publicada como conselho de utilizador por um fornecedor. **Um verbo
de apertar sobre uma membrana que preserva área é uma composição que o manual do
concorrente já declara má.**

### §E.3 — 3D-Coat (proprietário — só manual)

**Não é um pincel: é uma FERRAMENTA de drapeado.** O manual descreve um simulador que
lança uma malha por cima de outro objeto, com **Gravidade** (*«fá-la cair mais depressa
ou mais devagar, e é também o peso geral do drapeado»*), **Fricção** (*«quão “pegajoso”
o drapeado é ao objeto sobre o qual cai»*) e **Espessura**; botões de **iniciar/repor**;
⭐ **é possível interagir com a simulação EM CURSO pincelando por cima dela**; e no fim há
um passo explícito de **assar** o resultado no objeto.
([documentação 3DCoat](https://3dcoat.com/documentation/manual/workspaces-rooms/sculpt/surface-mode/objects-tools/))

### §E.4 — Marvelous Designer

**Categoria diferente e deliberadamente fora da conta.** O artista costura **padrões 2D**
que viram roupa simulada; não existe «pincel de tecido sobre uma escultura». Serve como
fronteira: *o que a nossa ferramenta faz é enrugar uma superfície que já existe, não
vestir um corpo.*

---

## §F — O que a documentação pública NÃO responde

⚠️ Cada linha abaixo é uma **ausência medida** — foi procurada em manual, API, notas de
versão e blog dos autores, e não existe. Só um estudo do código (outra missão, outro
agente) ou **medição do binário** as fecha.

| # | pergunta em aberto | como se responderia sem ler código |
|---|---|---|
| 1 | A **fórmula exata da direção** de cada um dos oito modos | ⭐ **medição do binário:** aplicar UM dab, num retalho **plano** e depois numa **esfera**, com traço de comprimento zero, e **exportar as posições antes e depois** — o campo de deslocamento resultante identifica ponto / linha / normal / repouso sem ambiguidade |
| 2 | Se **Expand** muda comprimento de repouso ou aplica força no plano | mesma medição: um repouso alterado sobrevive ao pen-up de forma diferente de uma força; e a **área total** da pegada é o discriminador |
| 3 | **Qual é o plano** do *Force Falloff → Plane* (normal do pen-down? plano da vista? plano da área?) | varrer: rodar a câmara **sem mexer no traço** e ver se a saída muda |
| 4 | A **família do solver**, o **passo de tempo**, e o **número de iterações/sub-passos por passo de pincel** | ⛔ inacessível por observação; é a pergunta que justifica a missão de código |
| 5 | Se existe **modelo de DOBRA** (e qual) ou se a rigidez de dobra é só um efeito da malha | medição: comparar o raio da dobra em duas densidades de malha com o mesmo gesto |
| 6 | A **forma da rampa** de *Simulation Falloff* (linear? suave? a curva de falloff do pincel?) | medição: um dab, e a magnitude do deslocamento por raio, na faixa `0,75..1,0` do limite |
| 7 | Como o **Snake Hook** calcula «a força da restrição por passo do pincel» | medição indireta: variar a velocidade da mão e medir a diferença contra o Grab |
| 8 | Se o pincel **re-pica a superfície** a cada passo, ou trabalha num plano de profundidade congelado *(o defeito da nossa [auditoria §3](03_auditoria_2026-09-05.md))* | medição: arrastar por cima de uma **esfera** e ver se a área simulada morre ao afastar-se do polo |
| 9 | O que *Cloth Damping* faz de facto na equação — a descrição publicada (*«quanto as forças são propagadas»*) **não é** a de um amortecimento | varredura do parâmetro com o mesmo traço, medindo alcance espacial contra decaimento temporal |
| 10 | Se **massa** interage com um passo de tempo fixo (⇒ se `massa` é, na prática, um ganho inverso) | varredura: dobrar a massa e ver se o deslocamento **se divide exatamente por dois** |
| 11 | Qual é o valor de omissão da **gravidade de escultura** neste contexto, e se ela está ligada durante um traço normal | leitura da UI numa instalação limpa |
| 12 | Se a simetria é expandida **antes** ou **depois** do solver *(a família da nossa [auditoria §4](03_auditoria_2026-09-05.md))* | medição: um traço exatamente sobre o plano de espelho, e verificar se a saída sai simétrica |

---

## §G — ⛔ As três coisas desta espec que uma leitura rápida entende ao contrário

1. **«O Grab é novo em 2.91»** — não é. A nota de versão anuncia *«novo pincel de tecido
   Grab que usa restrições»* na mesma linha do Snake Hook, e o Grab já está no manual de
   2.83. **São oito modos, não nove**; o que 2.91 lhe fez foi trocar o mecanismo.
2. **«A referência prega a fronteira»** — **não prega por omissão.** O controlo existe,
   chama-se *Pin Simulation Boundary*, e nasce **DESLIGADO**. O que está sempre ligado é
   a **rampa** de peso na faixa `0,75..1,0` do limite, que é anterior ao pino em onze
   meses. *Temos o opcional sempre ligado e não temos o obrigatório.*
3. **«Drag é arrastar, logo é uma translação»** — o manual diz *«puxar o pano PARA o
   cursor»*. O alvo é **um ponto**; numa toalha plana isso dá um campo **radial**, e o
   campo radial e a translação uniforme **só coincidem no limite de um raio infinito**.
   O modo que de facto translada com o rato chama-se **Grab**, e é outro.

---

## Fontes

**Primárias (manual e API oficial):**
- [Cloth — manual da ferramenta (versão corrente)](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/cloth.html)
- [Cloth — manual 2.83](https://docs.blender.org/manual/en/2.83/sculpt_paint/sculpting/tools/cloth.html) · [manual 2.93](https://docs.blender.org/manual/en/2.93/sculpt_paint/sculpting/tools/cloth.html)
- [Cloth Filter — manual](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/tools/cloth_filter.html)
- [Cloth Sculpting — introdução](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/introduction/cloth_sculpting.html)
- [Grab — manual](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/grab.html) · [Snake Hook — manual](https://docs.blender.org/manual/en/latest/sculpt_paint/sculpting/brushes/snake_hook.html)
- [Definições de pincel (Sculpt Plane, Normal Weight, Original Normal)](https://docs.blender.org/manual/en/latest/sculpt_paint/brush/brush_settings.html)
- [Documentação da API pública do pincel](https://docs.blender.org/api/current/bpy.types.Brush.html) — faixas e valores de omissão

**Notas de versão e blog dos programadores:**
- [Notas de versão 2.83 — Sculpting](https://developer.blender.org/docs/release_notes/2.83/sculpt/)
- [Notas de versão 2.90 — Sculpting](https://developer.blender.org/docs/release_notes/2.90/sculpt/)
- [Notas de versão 2.91 — Sculpting](https://developer.blender.org/docs/release_notes/2.91/sculpt/)
- [Pablo Dobarro — *Cloth Sculpting improvements in Blender 2.91*](https://code.blender.org/2020/10/cloth-sculpting-improvements-in-blender-2-91/)

**Terceiros (descritivo):**
- [RenderGuide — Blender Cloth Brushes Tutorial](https://renderguide.com/blender-cloth-brushes-tutorial/)
- [BlenderNation — Development News: Sculpt Cloth brush](https://www.blendernation.com/2020/01/30/sculpt-cloth-brush/)

**Outros apps:**
- [ZBrush — *Cloth Brushes* (manual Maxon)](https://help.maxon.net/zbr/en-us/Content/html/user-guide/3d-modeling/cloth-simulation/cloth-brushes/cloth-brushes.html)
- [Nomad Sculpt — manual, lista de ferramentas](https://nomadsculpt.com/manual/tools)
- [3DCoat — ferramentas de objetos em modo superfície](https://3dcoat.com/documentation/manual/workspaces-rooms/sculpt/surface-mode/objects-tools/)

**⛔⛔ Fontes que NÃO abriram — e que o IMPLEMENTADOR NÃO PODE ABRIR.** As duas ligações
abaixo apontam para **páginas de código do alvo (GPL)**. Elas estão aqui só para dar
proveniência às duas linhas marcadas `⚠️ (via resumo de busca)`; **nenhum diff foi lido
por mim, e abri-las contamina a janela que escreve o produto** (SKILL §3.I):
- [Revisão pública arquivada D8424](https://developer.blender.org/D8424) — ⛔ o diff não foi lido
- [Mensagem de commit pública sobre o falloff de plano](https://projects.blender.org/blender/blender/commit/c53b606) — ⛔ o diff não foi lido
