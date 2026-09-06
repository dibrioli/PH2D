# 05 — A VITÓRIA, medida (o pincel de tecido, 2026-09-06)

> ⚠️ **Este documento não celebra: ele PRESTA CONTAS.** Cada afirmação aqui traz o número que a
> sustenta e o gate que a defende, porque a única coisa que separa «o pincel ficou bom» de «o pincel
> reproduz o alvo» é uma régua com o lado aprovado dentro.
>
> Leia antes: [`04_espec_do_comportamento.md`](04_espec_do_comportamento.md) (o comportamento pela
> doc pública) · a espec clean-room [`SPEC_cloth_brush.md`](../cleanroom/SPEC_cloth_brush.md) (o
> comportamento pelo fonte, atestado) · o [plano do que falta](06_o_plano_do_que_falta.md).

## §1 — O que se ganhou, em uma tabela

| | 2026-09-05 (a lei VBD) | 2026-09-06 (a lei da referência) |
|---|---|---|
| traços do oráculo **dentro da barra** de paridade | **11** de 51 | **29** de 56 |
| traços **exactos ao bit** | 7 | 7 |
| pior erro do ARRASTO (o gesto de omissão) | `1,253` | **`0,071`** |
| modos de deformação **alcançáveis pela tela** | 0 de 8 | **8** de 8 |
| áreas de simulação alcançáveis | 0 de 3 | **3** de 3 |
| gates sobre o pincel | 3 | **12** |
| veredito do dono | reprovado **três vezes**, com foto | aprovado **três vezes**, em smoke |

⚠️ **A barra é `0,13`** — o pior erro por vértice de um traço, em unidades do maior deslocamento do
oráculo — e ela **saiu de um vale medido**: com a lei de hoje os traços partem-se em `29` com
`≤ 0,095` e `27` com `≥ 0,175`, sem nada entre os dois. ⛔ Não é um epsilon de conforto, e não se
aperta em direcção ao grupo: inverter a ordem de resolução das restrições — uma escolha nossa, e
Gauss–Seidel não comuta — move a nossa resposta em média `0,0985` e até `0,256` nos mesmos traços.
*Estamos a bater o oráculo com folga MENOR que o ruído de ordenação, o que só é possível se a nossa
ordem for a dele.*

## §2 — As QUATRO leis que faltavam, e nenhuma era o que se procurava

Cada uma foi achada pelo mesmo laço: medir contra o oráculo passo a passo → escrever a pergunta
com o número → um subagente-E lê o fonte e devolve o **comportamento** → implementar → medir outra
vez. ⛔ Nenhuma janela que escreveu produto abriu o fonte do alvo.

| # | o que se procurava | o que era | a medição que o prova |
|---|---|---|---|
| **Q8** | uma força em falta, ou mais varreduras na área *Local* | a **construção da lista de restrições corre `passagens + 1` vezes** ali, e o registo de duplicados vive UMA construção ⇒ cada restrição está lá em dobro | 27 dos 38 traços *Local* melhoram; os 12 não-*Local* ficam **byte-idênticos**; a contagem de vértices movidos passa a bater **exacta** em oito traços |
| **Q9** | a amplitude do Snake Hook | o **centro da queda está UM PASSO atrasado** — mede-se de onde o pincel *estava* | o pico passa a cair no mesmo vértice do oráculo (`0,86R` contra `0,86R`); toca em 7 traços e melhora os 7 |
| **Q11** | a lei que falta no aperto | **não falta nenhuma** — a força não decresce com a proximidade, o vértice ultrapassa o cursor, faces invertem, e a partir daí a **ORDEM** da lista decide | o mesmo traço com força `0,2` sai com erro **`0,000` nos doze passos** |
| **Q12** | uma lei que só a esfera pedisse | **`δ` não é a diferença dos dois pontos 3D do cursor — é a PROJECÇÃO dela no plano do ECRÃ**, e só o arrasto lê os pontos 3D | melhora 3 traços de esfera e deixa **53 inalterados**: numa folha vista de frente a projecção é um no-op |

⭐⭐⭐ **A aritmética que fecha o Q8:** a lista dobrada é `[c₁..c_N, c₁..c_N]`, logo cinco varreduras
sobre ela são, **na ordem**, exactamente dez sobre a simples. O botão que eu media (`PH2D_VARREDURAS`)
e o mecanismo que o especificador achou são a mesma coisa nos modos de força — e é por isso que as
duas leituras coincidiram ao número. ⚠️ **Nos modos de ÂNCORA não são:** repetir só os pares deixa as
âncoras no meio da lista e piora o Grab de `0,050` para `0,415`. *Onde uma restrição está na lista é
tão load-bearing quanto quantas vezes ela lá está.*

⭐⭐ **E o Q11 é a vitória mais estranha de todas: ela é uma ABSOLVIÇÃO.** O aperto errava `1,38` e a
resposta foi que a nossa lei está certa. O alvo, com força alta, vira o retalho debaixo do cursor do
avesso e a resposta dele passa a depender da ordem em que ele percorre uma árvore espacial que não é
a nossa — e o próprio tracker dele tem isso em aberto. *Perseguir aquele número era perseguir um
defeito conhecido.* A prova é uma **intervenção**: o mesmo traço, a mesma malha, o mesmo caminho, com
UMA coisa mudada — a força de `1` para `0,2` — dá zero faces invertidas e erro `0,000`.

## §3 — Por que se pode acreditar nisto: as réguas que se corrigiram

⛔⛔ **Duas das três barras do gate de artefactos foram RETIRADAS, e não afrouxadas.** Elas
reprovavam a saída do **próprio alvo**. Medido nele, nos traços de arrasto:

| grandeza | o defeito de 05/09 | a lei VBD (dita «sã») | **o ALVO** | veredito |
|---|---|---|---|---|
| espinho (maior deslocamento) | `0,690` | `0,052` | **`0,900`** | ⛔ não discrimina |
| rasgo (salto entre vizinhos) | `0,387` | `0,018` | **`0,219`** | ✅ discrimina |
| estica (aresta / repouso) | `2,98×` | `1,14×` | **`3,72×`** | ⛔ não discrimina |

*O alvo deforma MAIS do que o defeito deformava em duas das três colunas.* As barras vinham da lei
VBD, que era tímida, e liam como saúde o pano mal se deformar — é a armadilha que esta casa já tem
registada: **uma barra calibrada sem o lado aprovado mede os nossos próprios defeitos**. Sobra o
**rasgo**, que discrimina por mecanismo (uma agulha é um vértice que anda enquanto os vizinhos
ficam), com a barra `0,30` na banda medida entre o pior arrasto do alvo e o defeito reproduzido.

⚠️ **E um gate que era NOMEADO em dois doc-comments não existia:** o
`the_panel_offers_every_falloff_the_engine_has` era citado como se corresse, e ninguém o tinha
escrito. *Uma promessa de gate lê-se exactamente como um gate, e a diferença só aparece no dia em
que ele devia sangrar.*

## §4 — Os doze gates, e o que cada um morre a provar

| gate | onde | a mutação que o mata |
|---|---|---|
| os traços de um passo de força saem **ao bit** | `ph2d-cloth` | a massa contada duas vezes · o Push a perder o `2R` **na componente Z** |
| a **lista do *Local*** vem em duplicado (duas metades idênticas) e os inteiros batem | `ph2d-cloth` | `construcoes()` a devolver sempre `1` |
| o **centro do Snake Hook** está um passo atrasado | `ph2d-cloth` | o centro no cursor |
| a **paridade** não regride (duas listas, censo dos dois lados) | `ph2d-cloth` | qualquer uma das anteriores |
| o **aperto inverte** no 1.º passo e o arrasto não | `ph2d-cloth` | travar o impulso do aperto |
| a **assimetria de espelho** fica no patamar do oráculo (dois regimes) | `ph2d-cloth` | idem |
| **fora da inversão** o aperto é tão comparável quanto o arrasto | `ph2d-cloth` | a força do aperto pela metade |
| a **força por passo das âncoras** é zerada em toda a malha | `ph2d-cloth` | apagar o zeramento nos dois sítios |
| os **oito modos** dão oito panos diferentes, e cada um move | `ph2d-sculpt3d` | o adaptador a ignorar o modo |
| as **três áreas** dão três panos diferentes | `ph2d-sculpt3d` | o adaptador a ignorar a área |
| o **OLHO define o plano** em que o tecido lê o gesto (as duas metades) | `ph2d-sculpt3d` | o adaptador a entregar o caminho 3D cru |
| o **painel oferece** o que o motor tem (censo nos dois sentidos) | `ph2d-panel-sculpt3d` | um valor a menos no `ALL` **ou** um id a menos no array |

⚠️ **Três destes gates existem porque a jornada descobriu que o que os substituía media outra
coisa**, e um deles — o do undo — estava a percorrer o braço do gesto que o produto já não toma.

## §5 — O que o dono aprovou, e quando

| momento | o que ele viu | o que ele disse |
|---|---|---|
| 05/09 | a lei VBD, três vezes | *«um drag com artefatos»* · *«papel amassado»* · *«pior que o anterior»* |
| 06/09, 1.ª | a lei da referência como omissão, só o arrasto | **«Smoke OK»** |
| 06/09, 2.ª | os oito modos e as três áreas na tela | **«Smoke OK. Muito bom!»** |

⛔⛔ **E uma decisão continua a ser dele**, com as duas frases que a põem e sem terceira saída — está
no [plano do que falta, §4](06_o_plano_do_que_falta.md).
