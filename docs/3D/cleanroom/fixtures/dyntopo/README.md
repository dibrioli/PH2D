# Fixtures — quem muda a topologia no modo de TOPOLOGIA DINÂMICA, do ORÁCULO, sobre uma malha NOSSA

⭐ **Este corpus responde a UMA pergunta binária, por tipo de pincel:** com a topologia dinâmica
**armada**, um traço muda a contagem de vértices? E, em separado, ela **SOBE** (refino) ou
**DESCE** (colapso)? É o insumo do [plano 22](../../../22_plano_quem_subdivide_no_dyntopo.md), cujo
§3 exige **duas colunas por pincel, nunca uma** — o refino e o colapso são leis independentes, e um
verbo pode querer uma sem a outra.

⛔ **O que este README NÃO diz:** *porquê*. Ninguém leu uma linha do alvo. O que está aqui é o que
saiu de o **correr**, e cada célula é um vector de teste.

## Proveniência (SKILL_Cleanroom §5 — a da ENTRADA decide a da saída)

| | |
|---|---|
| **Malha de entrada** | ⭐ **nossa**, gerada pelo próprio arnês: plano triangulado por uma diagonal **fixa**, `20×20` células de lado `2,0` (**441** vértices, aresta `0,1`), com relevo senoidal (amplitude `0,08`, frequência `6`) e jitter determinístico de `0,35` da célula nos vértices interiores. ⛔ Nenhum asset do alvo entra como entrada |
| **Quem calculou** | o binário **Blender 5.2.1 LTS**, corrido pelo subagente-E **fora da árvore** (`~/Referencias/blender-dyntopo/oracle/`, ⛔ negado ao Implementador), por script, dentro de um compositor **virtual** — nunca o ecrã de ninguém. Um pincel do catálogo instalado é activado **só para existir um pincel do tipo certo**; todos os ajustes que decidem o resultado são reescritos e vão no cabeçalho |
| **Estatuto legal** | ⭐ **dados** — GPLv2 §0: a saída só é coberta se o **conteúdo** dela for obra baseada no programa, e contagens de vértices de uma malha nossa não são |
| **Regenerar** | ⛔ acto de **E**, nunca do Implementador (o arnês vive na zona negada). O I pede pelo Enio, como emenda |
| **Data** | 2026-09-14 |
| **Ficheiro** | `quem_subdivide.txt.gz` (`sha256` começa em `79e05e2c449e58e3`), texto gzip com `mtime` fixo a `0` ⇒ reprodutível byte a byte |

## O formato

Cabeçalho de linhas `# chave: valor`; depois uma linha por célula, campos separados por `|`.
**Os cinco primeiros campos são o contrato** (`pincel | traço | detalhe | v_antes | v_depois`); do
sexto em diante é o **CONTROLO**, e ele não é decoração:

⚠️⚠️ **Sem o controlo, «não mudou» não se distingue de «o verbo não agiu».** Foi exactamente isso
que mordeu **três** vezes na construção deste corpus — e nas três o veredito inicial era um
**falso «não»**:

| célula | o que a 1.ª leitura dizia | o que estava a acontecer | a cura |
|---|---|---|---|
| `ROTATE` | *não refina, não colapsa* | uma recta que **passa pela âncora** varre **ângulo zero** ⇒ o verbo não rodava nada (`movidos = 0`) | o traço `circulo` |
| `BOUNDARY` | *não refina, não colapsa* | o gesto corria no **centro** da peça e a borda aberta está a `1,0` ⇒ não havia fronteira ao alcance | o traço `arrasto_na_borda` |
| `SCENE_PROJECT` | *refina e colapsa* (mas com `movidos = 0`) | **não havia nada na cena para onde projectar** ⇒ a coluna estava certa e o controlo era vazio | o traço `arrasto_com_alvo` |

⭐ E para os verbos que **não movem um único vértice** o controlo não é a posição: é o **canal**
por-vértice que eles escrevem (coluna `canais`). Sem ele, `MASK` e `DRAW_FACE_SETS` leriam-se como
«o verbo não agiu» — e `SIMPLIFY` é o caso oposto, em que a mudança de topologia **é** o efeito.

## A TABELA — o veredito, derivado das células

⚠️ **Derivada, não escrita à mão** (`analisa.py` no arnês): `REFINA` lê a célula
`fino_so_subdividir` (só o refino ligado, alvo mais fino que a malha) e pergunta se
`v_depois > v_antes`; `COLAPSA` lê a célula `grosso_so_colapsar` (só o colapso ligado, alvo mais
grosso que a malha) e pergunta se `v_depois < v_antes`. *Isolar as duas metades é o que torna a
resposta binária*: na configuração de fábrica as duas correm juntas e a contagem final é um número
só, que não distingue «não refinou» de «refinou e colapsou por igual».

| pincel (identificador público) | rótulo na tela | REFINA? | COLAPSA? | controlo | evidência |
|---|---|---|---|---|---|
| `DRAW` | Draw | **SIM** | **SIM** | mov=80 | ref 441->3337  col 441->324 |
| `DRAW_SHARP` | Draw Sharp | não | não | mov=80 | ref 441->441  col 441->441 |
| `CLAY` | Clay | **SIM** | **SIM** | mov=80 | ref 441->2184  col 441->324 |
| `CLAY_STRIPS` | Clay Strips | **SIM** | **SIM** | mov=82 | ref 441->2182  col 441->324 |
| `CLAY_THUMB` | Clay Thumb | **SIM** | **SIM** | mov=73 | ref 441->2135  col 441->324 |
| `LAYER` | Layer | não | não | mov=79 | ref 441->441  col 441->441 |
| `INFLATE` | Inflate | **SIM** | **SIM** | mov=80 | ref 441->2398  col 441->324 |
| `BLOB` | Blob | **SIM** | **SIM** | mov=80 | ref 441->3294  col 441->324 |
| `CREASE` | Crease | **SIM** | **SIM** | mov=80 | ref 441->4020  col 441->323 |
| `SMOOTH` | Smooth | não | não | mov=80 | ref 441->441  col 441->441 |
| `PLANE` | Plane | **SIM** | **SIM** | mov=73 | ref 441->2154  col 441->324 |
| `MULTIPLANE_SCRAPE` | Multi-plane Scrape | **SIM** | **SIM** | mov=49 | ref 441->2146  col 441->324 |
| `PINCH` | Pinch | **SIM** | **SIM** | mov=73 | ref 441->2846  col 441->322 |
| `GRAB` | Grab | não | não | mov=37 | ref 441->441  col 441->441 |
| `ELASTIC_DEFORM` | Elastic Deform | não | não | mov=441 | ref 441->441  col 441->441 |
| `SNAKE_HOOK` | Snake Hook | **SIM** | **SIM** | mov=72 | ref 441->2723  col 441->333 |
| `THUMB` | Thumb | não | não | mov=37 | ref 441->441  col 441->441 |
| `POSE` | Pose | não | não | mov=219 | ref 441->441  col 441->441 |
| `NUDGE` | Nudge | **SIM** | **SIM** | mov=73 | ref 441->2853  col 441->322 |
| `ROTATE` | Rotate | não | não | mov=37 | ref 441->441  col 441->441 |
| `TOPOLOGY` | Slide Relax | não | não | mov=73 | ref 441->441  col 441->441 |
| `BOUNDARY` | Boundary | não | não | mov=80 | ref 441->441  col 441->441 |
| `SCENE_PROJECT` | Scene Project | **SIM** | **SIM** | mov=80 | ref 441->2476  col 441->324 |
| `CLOTH` | Cloth | não | não | mov=441 | ref 441->441  col 441->441 |
| `SIMPLIFY` | Simplify | **SIM** | **SIM** | n/a(topo) | ref 441->2065  col 441->324 |
| `MASK` | Mask | não | não | canal | ref 441->441  col 441->441 |
| `DRAW_FACE_SETS` | Draw Face Sets | não | não | canal | ref 441->441  col 441->441 |
| `DISPLACEMENT_ERASER` | Multires Displacement Eraser | **?** | **?** | NAO CORREU |  |
| `DISPLACEMENT_SMEAR` | Multires Displacement Smear | **?** | **?** | NAO CORREU |  |
| `PAINT` | Paint | **?** | **?** | canal |  |
| `SMEAR` | Smear | **?** | **?** | canal |  |
| `BLUR` | Blur | **?** | **?** | canal |  |

⭐ **14 tipos mudam a topologia · 13 não mudam · 5 não puderam ser exercitados.**
⛔ **Nenhum tipo separa as duas colunas** — todo o que refina também colapsa, e vice-versa. Isso é um
**facto sobre este alvo**, não uma lei: o plano §3 exige as duas colunas precisamente porque a
separação é *exprimível*, e o modo de refino da cena (`só subdividir` / `só colapsar` / os dois)
prova que o alvo a sabe fazer — ele só não a usa por pincel.

## As barras, e de onde saem

⚠️ **A resolução do alvo não foi escolhida: saiu de uma varredura.** Com um verbo de depósito sobre
uma malha plana da mesma densidade, os dois modos ligados: `1 → 308` · `2 → 329` · `3 → 361` · `6 → 492` · `12 → 949` · `24 → 2779` · `48 → 10069` (de `441`). O cruzamento
está entre `3` e `6` ⇒ `24,0` é «alvo mais fino que a malha» com folga e `2,0` é «alvo mais grosso»
com folga.

⚠️ **E um «não» só vale com o extremo medido ao lado.** Os 13 tipos que não mudam a topologia
foram re-corridos a `48,0` (refino) e a `0,5` (colapso) — *o dobro e um quarto das barras do corpus*
— e **continuam em `441 → 441`**, com o controlo positivo (o mesmo par de extremos com um verbo de
depósito) a dar `441 → 11820` e `441 → 224`. *Sem isso, «não refina» não se distinguiria de
«a barra do corpus era fraca».*

⚠️⚠️ **E o extremo tem de correr com o GESTO que exercita o verbo.** A primeira redacção deste
parágrafo corria os extremos com o arrasto recto para todos — e nesse gesto o verbo de **torção** e o
de **contorno** não fazem nada (§ do controlo, acima) ⇒ o extremo deles media o nada. As linhas
`circulo_extremo_*` e `naborda_extremo_*` do corpus são a correcção: o mesmo extremo, com o gesto
próprio de cada um. *Um controlo que não percorre o mesmo caminho do caso positivo afirma sobre
código que não corre.*

## As quatro coisas que uma leitura rápida entende ao contrário

1. ⛔ **`441 → 441` NÃO quer dizer «o pincel não fez nada».** Quer dizer «a contagem de vértices não
   mudou». O verbo pode ter movido 441 vértices (é o caso de dois deles) — leia a coluna `movidos`.
2. ⛔ **As células `arrasto_nativo` não são uma repetição:** nelas **nada** do preset de fábrica é
   reescrito (só o raio, que tem de ser em unidades de objecto para a célula ser reprodutível). Elas
   existem para responder *«o veredito é do TIPO de pincel ou do preset?»* — e a resposta medida é
   **do tipo**: dos **54** pares (nativo × normalizado) comparados, **0** discordam. O
   mesmo vale para as cinco variantes do tipo `PLANE` e para as nove variantes de outros tipos:
   mesmo veredito.
3. ⚠️ **`parado` não é «o mesmo com menos dabs»** — são `12` dabs no mesmo sítio contra `6` ao longo
   de um arrasto, e ela refina **menos** (a esfera do pincel não viaja). Ela está aqui para a
   pergunta *«o passo de topologia precisa que o cursor ANDE?»*, e a resposta é não: quem refina a
   arrastar refina parado.
4. ⚠️ **O método do traço não muda o veredito** (`arrasto_espacado` contra `arrasto`), e o traço
   `nenhum` — a topologia dinâmica armada e **nenhum** dab — deixa a malha em `441 → 441`. *Armar não
   é mudar.*

## ⛔ As células que NÃO puderam ser exercitadas, nomeadas

| pincel | o que acontece | leitura |
|---|---|---|
| `DISPLACEMENT_ERASER` · `DISPLACEMENT_SMEAR` | o alvo **termina** (o processo morre) em **todas** as células, incluindo a de controlo com a topologia dinâmica desarmada | são os dois verbos de **multirresolução**, e multirresolução e topologia dinâmica **excluem-se** neste alvo: a combinação não existe para o artista |
| `PAINT` | a célula de controlo (topologia dinâmica **desarmada**) corre e **escreve** o canal de cor (`74` valores distintos); **toda** célula com a topologia dinâmica armada **termina o processo** | verbo de **cor**: a combinação não é exercitável neste build |
| `SMEAR` · `BLUR` | ⚠️ **nem o controlo vale**: a malha nasce de cor **uniforme**, e borrar ou desfocar o uniforme é um no-op (`1` valor distinto, `movidos = 0`). Com a topologia dinâmica armada, o processo **termina** | verbos de **cor** *derivados* — para os exercitar seria preciso pintar antes, e a combinação continua a terminar o processo |

⚠️ **Estas cinco linhas de `?` não são buracos do corpus: são a resposta.** *Uma ausência medida vale
mais que um palpite* — e as duas famílias acima dizem, cada uma, que a pergunta do plano §4 **não tem
sujeito** naquele tipo de pincel neste alvo.

## ⚠️ Uma observação que NÃO reproduziu, e por isso não é um achado

Numa corrida em **lote**, as quatro células do pincel de contorno junto da borda aberta devolveram
**todas as posições `NaN`** (a contagem ficou em `441`). Re-corridas **sozinhas**, na mesma
configuração, **`2` de `2` saem limpas** (`z_max = 0,2423`, `movidos = 80`), e as sete células
`naborda_f05_*` — que são as que a tabela usa como evidência — saem limpas nas sete, incluindo os
dois extremos. ⇒ *o `NaN` é um facto sobre aquela sessão em lote, não sobre a pergunta*; fica no
corpus como dado e **não** entra no veredito. Isto é a lei da casa sobre reprovações sob carga
aplicada a um oráculo: **re-corra sozinho antes de olhar para a sua hipótese**.

## Como regenerar (acto de E)

O arnês vive em `~/Referencias/blender-dyntopo/oracle/` (fora da árvore): `harness.py` conduz a API
pública do binário instalado, `run.sh` põe-no dentro de um compositor virtual, `analisa.py` deriva a
tabela e `fixtures.py` escreve este directório. ⛔ Nada disso é produto, e nada disso entra no repo.

**Células medidas: 343.**
