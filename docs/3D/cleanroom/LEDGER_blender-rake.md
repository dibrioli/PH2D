# LEDGER de proveniência — clean-room do PENTE DE TOPOLOGIA (alvo `blender-rake`)

> Aberto conforme [SKILL_Cleanroom §6](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
> ⛔ **O Implementador NUNCA abre este arquivo** — ele carrega rastros do alvo de propósito.
> O canal de I para cá é o `INBOX_blender-rake.md` (append cego).

> ⛔⛔ **REGRAS DE HIGIENE, NO TOPO DE PROPÓSITO** (herdadas do `LEDGER_blender-pincel-afiado.md`,
> que as herdou do `-trim-pincel`, onde a mesma classe de incidente aconteceu DUAS vezes por a cura
> estar escrita no fim):
>
> 1. **A vassoura descodifica-se em MEMÓRIA, por cano.** ⛔ Nunca para ficheiro — nem `/tmp`, nem o
>    scratchpad, nem `/dev/shm`.
> 2. **Um relatório de sweep cita em claro os termos que acusaram** ⇒ é material do alvo e vive em
>    `~/Referencias/blender-rake/`, nunca no repo.
> 3. **Tudo do alvo** — fonte lido, notas, harness, dumps crus, rascunhos da espec — vive em
>    `~/Referencias/blender-rake/`.
> 4. **O report final do E** é gravado em `~/Referencias/blender-rake/draft/`, varrido, e só então
>    devolvido, sob o CONTRATO DE RETORNO (§3.E).

⏱️ **Aberto em 2026-09-17, ANTES da primeira leitura de CONTEÚDO do fonte do alvo.** Antes desta
abertura o E fez apenas:

1. **Triagem de licença** por metadados de pacote desta máquina (`pacman -Qoq` sobre cada binário,
   depois `pacman -Qi`), `blender --version`, e as três primeiras linhas do `COPYING` do checkout —
   o primeiro acto obrigatório (§2: *ler a licença real*);
2. leitura de artefactos **NOSSOS**: a SKILL, `_ComoInvestigarApps/{00,01}`, o `README` desta pasta,
   `LEDGER_blender-pincel-afiado.md`, `SPEC_pincel_afiado.md`, `docs/3D/20_divergencias_tools.md`,
   `crates/ph2d-mesh/src/dyntopo_flip*`;
3. leitura do **fonte permissivo** do SculptGL (MIT) para a triagem T1 — ver abaixo;
4. a criação da zona fora da árvore `~/Referencias/blender-rake/{notes,draft,oracle,fixtures,out}`.

⚠️ **Um caminho de ficheiro do alvo entrou no terminal do E antes desta abertura**, por um
`grep -rl` sobre `/usr/share/` que faz parte da triagem («existe isto neste sistema?»). Só NOMES de
ficheiro, zero conteúdo. Registado aqui por completude; não é exposição substancial (§6.2: relance).

---

## Alvo

| campo | valor |
|---|---|
| Nome | Blender — a **opção de pincel** que, com topologia dinâmica armada, alinha o fluxo das arestas da malha ao longo da direcção do traço (rótulo público no painel: *Topology Rake*). Nesta casa: o **pente de topologia**. |
| Versão do fonte disponível | checkout esparso e *grafted* em `/home/enio/Documentos/Recursos/BlenderSculpt/`, `git describe --tags` ⇒ **v5.2.0**, commit `fbe6228777e7d9afefcd61a413844e790ae75db7` — o MESMO das obras `-cloth`, `-pose`, `-boundary`, `-pull`, `-unblocked`, `-trim`, `-trim-pincel`, `-pincel-afiado` |
| Oráculo (binário) | `/usr/bin/blender` = **Blender 5.2.2 LTS**, pacote `blender 17:5.2.2-1`, build **2026-09-15** |
| ⚠️ **MUDANÇA DE VERSÃO DO ORÁCULO** | Todas as obras anteriores desta linha registaram `17:5.2.1-2`. **O pacote subiu para `5.2.2-1` em 2026-09-15**, entre a obra do pincel afiado e esta. ⇒ um corpus desta obra e um corpus das anteriores **não foram colhidos do mesmo binário**, e isso vai escrito no cabeçalho de cada fixtura desta obra. |
| Licença do ALVO | **GPL-2.0-or-later** (`pacman -Qi blender` lista também Apache-2.0, BSD-2/3, LGPL-2.1+, MIT, MPL-2.0, Zlib para partes vendorizadas; o `COPYING` do checkout remete à GPL) |
| Degrau da triagem | **T2** — copyleft com fonte ⇒ pipeline com parede |
| Repo de origem | `https://projects.blender.org/blender/blender.git` (⛔ denylist do I) |
| Zona fora da árvore | `~/Referencias/blender-rake/` |

### A concessão relevante (GPLv2 §0)

> *"Activities other than copying, distribution and modification are not covered by this License;
> they are outside its scope. The act of running the Program is not restricted, and the output from
> the Program is covered only if its contents constitute a work based on the Program (independent of
> having been made by running the Program)."*

⇒ Correr e instrumentar **em privado** é licenciado. A **saída** — a lista de índices de triângulos
de uma malha **NOSSA** — é dado. Nenhum acto desta obra envolve distribuição. Não é AGPL.

---

## §2 — Triagem: a escada de portas, percorrida NA ORDEM (2026-09-17)

⚠️ A unidade da triagem é o **ARTEFACTO instalado**, nunca o nome do projecto.

| degrau | o que foi perguntado | resposta MEDIDA |
|---|---|---|
| **T0** — existe artefacto **permissivo** que já faça isto? | *«Algum app deste sistema alinha o fluxo de arestas por troca de diagonal durante o traço?»* | **NÃO.** Ver a tabela de artefactos abaixo. |
| **T0½** — copyleft por-ficheiro / LGPL linkável? | — | Não se aplica: nenhum candidato. |
| **T1** — irmão permissivo no ecossistema? | Reimplementação permissiva independente do alinhamento de fluxo por flip durante escultura | **NÃO ACHADA.** O que existe em permissivo nesta máquina é **outra CLASSE** — ver a nota abaixo. |
| **T2** — copyleft com fonte | Blender, GPL-2.0-or-later | ⇒ **É AQUI.** Pipeline com parede. |

### Os artefactos varridos, com a licença lida NELES

| artefacto | pacote / caminho | licença lida | tem alinhamento de fluxo por flip? |
|---|---|---|---|
| SculptGL | `/home/enio/Documentos/Recursos/SculptGL` | **MIT** (`LICENSE`, © 2019 Stéphane Ginier) | ⛔ **NÃO.** A topologia dinâmica dele tem exactamente **dois** ficheiros — subdivisão e decimação. Zero ocorrências de troca de diagonal em todo o `src/`. **Confirma** a célula `❌` de [`20_divergencias_tools.md` §8/§9](../20_divergencias_tools.md). |
| Godot | `godot 4.7.2-1.1` | **MIT** | Não esculpe malha. |
| OpenToonz | `opentoonz 1.8.0-3.1` | **BSD-3-Clause** | Animação 2D. |
| `libmypaint` | `libmypaint 1.6.1-2.1` | **ISC** | Pintura 2D. |
| MeshLab | `meshlab-bin 2025.07-2` | **GPL** | Tem optimização de malha por troca de diagonal — ⛔ **mas é GPL** (não é porta aberta) **e é outra classe**: uma passagem global de limpeza, sem traço, sem direcção e sem interacção. |
| Krita · GIMP · Inkscape · FreeCAD · OpenSCAD · Synfig · MyPaint (o app) | — | GPL / LGPL | Não fazem isto. |

⭐ **O irmão permissivo MAIS PRÓXIMO já vive nesta casa e é de OUTRA CLASSE** — o porte do Instant
Meshes (BSD-3) em `ph2d-quadflow`, e a cadeia global de retopologia. Ele alinha arestas a um campo
de direcções, mas: corre **uma vez, sobre a peça inteira**, **reconstrói** a malha em vez de a
editar, e o campo dele vem da **curvatura**, não do gesto do artista. *Um remalhador global e um
pente por-traço não são a mesma ferramenta, e portar o primeiro não entrega o segundo.*

⭐⭐ **E o SUBSTRATO já é NOSSO e permissivo:** a troca de diagonal local durante o refino existe em
[`crates/ph2d-mesh/src/dyntopo_flip.rs`](../../../crates/ph2d-mesh/src/dyntopo_flip.rs), portada do
SculptGL/Botsch–Kobbelt, com as quatro recusas dela gateadas. ⇒ **o que falta não é o operador, é o
CRITÉRIO que escolhe qual diagonal fica** — e é sobre esse critério que esta espec pergunta.

**Veredito da triagem: T2, parede obrigatória.** Não houve porta aberta a que parar.

---

## §8.1 — Checkpoint de PATENTE (2026-09-17)

**Termos buscados:** troca de diagonal alinhada à direcção do traço · relaxação de vértices por
fluxo de arestas durante escultura · alinhamento de fluxo de arestas por pincel · o nome público da
opção · os detentores plausíveis (o dono do alvo; os donos dos dois escultores comerciais).

**Resultado: NENHUMA patente viva alcança o método.**

| achado | estado | porque NÃO alcança |
|---|---|---|
| **US 9 349 216 B2** (pedido US2015/0002510), **Disney Enterprises**, prioridade 2013-06-28, concedida 2016-05-24, **viva até 2034-07-12** | ⚠️ **viva** | A reivindicação 1 é sobre **amostrar pontos de curva numa superfície, guardá-los numa rede de curvas e GERAR retalhos de quadriláteros** quando meias-cadeias fecham um laço com três ou mais cantos. ⛔ Ela **não** reivindica trocar diagonais de uma malha triangular existente, nem mover vértices durante um traço de escultura. É geração de retalhos a partir de curvas desenhadas — outro acto. |
| US 7 310 097 / US 7 528 832 (parametrização global dinâmica de malhas triangulares) | **expiradas** | Prioridade de 2003; e o assunto é parametrização, não o gesto. |

**Arte anterior pública, esmagadora e citável:** a troca de diagonal como melhoria local de uma
triangulação é de **Lawson, 1977**; a cadeia incremental *partir · colapsar · **trocar** · alisar* é
de **Botsch–Kobbelt, 2004** — as duas **já portadas nesta casa**, com atribuição, em
`crates/ph2d-mesh/src/dyntopo_flip.rs`. A própria opção do alvo é pública desde **Janeiro de 2019**.

⭐ E há um facto que reduz o risco a quase nada: **medido, o método não é uma troca de diagonal** —
é uma relaxação tangencial de vértices (SPEC §3.1). Nem a patente viva nem a arte anterior de flip
descrevem o que esta obra vai implementar.

**Veredito: prosseguir.**

---

## §7 — Sweep e isenções (2026-09-17)

**Instrumento provado com CONTROLO POSITIVO antes de qualquer veredito:** um canário plantado sai
`exit = 1` (duas espécies: um identificador público do assunto, e uma frase de prosa do alvo); o
corpus limpo sai `exit = 0`.
⚠️⚠️ **E o `--git-history` apanhou ESTA MESMA LINHA na sua 1.ª redacção**, que escrevia o canário
**em claro** — a violação da **regra nº 2 do topo deste ficheiro**, pela mão de quem a escreveu.
⇒ curada pela lei do §6.1 (*o registo DESCREVE, nunca REPRODUZ*). ⛔ O commit `45dd47832` retém a
redacção antiga no patch, e isso **não se cura reescrevendo o histórico** (INC-U1 desta linha, 13/09):
fica **registado**, e o `--git-history` desta pasta acusa esse patch **por desenho**, não por defeito
novo.

**Corridas, todas VERDES:**

1. as **dez** vassouras vivas da pasta sobre `SPEC_pente_de_topologia.md` + as **195** fixturas de
   `fixtures/rake/` (a lei de 13/09: quem fecha corre **todas** as vivas, não só a sua);
2. `--git-history` da pasta `docs/3D/cleanroom/` com a vassoura desta obra.

### ⚠️ ISENÇÃO NOMEADA nº 1 — o rótulo público, na vassoura e fora dela

A primeira redacção da vassoura trazia o rótulo público da opção **nu**, e ele acusou **uma** linha:
o cabeçalho da própria espec, que o nomeia para que o Implementador saiba por que nome procurar o
**manual público** do alvo (uso nominativo, SKILL §8.4; é a mesma forma que o cabeçalho da
`SPEC_pincel_afiado.md` usa). ⇒ o rótulo **nu** sai da vassoura e no lugar dele entram **três**
formas de PROSA que só o texto do alvo tem. **Controlo dos dois lados, medido:** a prosa sai
`exit = 1`; o rótulo nu sai `exit = 0`, e isso é **decisão declarada**, não silêncio. ⛔ Se o R
discordar, a cura é tirar a linha do cabeçalho da espec — não repor a entrada sem a tirar.

### ⚠️⚠️ ISENÇÃO NOMEADA nº 2 — a vassoura NOVA nasceu mais larga que as vizinhas

Esta vassoura foi escrita depois de oito irmãs, e trazia **cinco** identificadores públicos de
propriedade que são **vocabulário partilhado de toda a família da escultura**: eles acusavam
artefactos de **outras obras, já ATESTADAS**, que estão autorizadas a usá-los como **chave de
regeneração** (§4.1.13). Medido: `5` de `56` entradas colidiam.

⇒ as cinco **saíram**, e a razão é de método: *manter na minha vassoura um identificador que outra
espec atestada tem autorização para usar converte uma autorização concedida num falso positivo
PERMANENTE sobre texto que um revisor já julgou.* É a lei de 13/09 — *o sweep é propriedade do PAR
(artefacto, vassoura)* — aplicada no sentido inverso: **uma vassoura nova tem de correr sobre a
árvore inteira antes de ser adoptada, senão ela acusa o passado.** As outras `51` ficam.

### 📌 ACHADO PRÉ-EXISTENTE — ⚠️ E A CONTAGEM ABAIXO ESTÁ SUBCONTADA; ver a correcção no §4.3

`git grep` mede **uma** citação de nome interno do alvo sobre este assunto na árvore:
`docs/3D/20_divergencias_tools.md`, linha `422`, introduzida pelo commit `99e129b60` (o estudo
comparativo da época em que se lia o fonte). **Zero adições desta obra** (o `git status` desta
worktree tem só os quatro ficheiros novos). ⚠️ É a mesma forma do achado de 14/09 (o token de duas
palavras da obra `-pull`, ⛔ não reproduzido aqui — §6.1):
`docs/**` fica fora do censo do gate por construção, logo ele nunca a viu.
⛔ **Não curada aqui de propósito** — é doc de outro assunto e a decisão é de triagem.

⭐ **E a ironia é o próprio achado:** essa linha é a que o briefing desta obra mandou **confirmar**,
e ela está **CERTA** (ver a triagem acima). O mesmo texto é, ao mesmo tempo, a afirmação que se
verificou e uma dívida §4.2 do repo.

---

## Papéis e corrente

| papel | id / data | o que fez |
|---|---|---|
| **E** (este) | subagente-E, 2026-09-17 | triagem · ledger · patente · harness · corpus · espec |
| **I** | — | (a janela da linha; nunca abre este ficheiro) |
| **R-pré** | subagente-R independente, 2026-09-17 | ⛔ **REPROVADA** — ver §4.2 no fim deste ficheiro (13 achados, 5 triagens de vassoura, e o limite que o §12 da espec não lista) |
| **R-pós** | — | por despachar |

## Cobertura da travessia (§3.E) — ⚠️ LEIA ISTO

⛔⛔ **O fonte do alvo NÃO FOI LIDO NESTA OBRA — nem uma linha, nem um ficheiro.** A cobertura é
**zero por escolha**, e não por descuido: o método da casa (`docs/_ComoInvestigarApps/00`) diz que
ler é o método **pior**, e aqui ele foi dispensável. Tudo o que a espec afirma veio de:

1. **CORRER** o alvo sem interface, por script, sobre malhas **nossas** — `195` ficheiros de corpus;
2. um **despejo da superfície PÚBLICA de propriedades**, também corrido;
3. **prosa pública** (manual, arquivo público de mensagens de commit, discussão pública de
   programadores), lida e **re-dita em palavras nossas**, com a fonte registada na espec §10.

⇒ o risco de convergência de expressão nesta obra é o mais baixo de toda a linha: **não houve
expressão do alvo em contexto nenhum, em momento nenhum.**

## Incidentes

- *(nenhum até à abertura)*

---

## §4.2 — AUDITORIA R-PRÉ: ⛔ **REPROVADA** (subagente-R independente, 2026-09-17)

> **Quem:** um R que **não escreveu esta espec** e não falou com o E. Método: re-derivar cada número
> **do ficheiro publicado** com um parser e uma implementação da régua escritos só a partir do
> `fixtures/rake/README.md`, nunca a partir do código do E; mais varredura por **FORMA** das
> promessas (§4.3 #2) e das quatro triagens de vassoura.
>
> ⛔ **REPROVADA não quer dizer «o desenho está errado».** A tese central resistiu a todos os
> ataques que lhe fiz (§4.2.3) e a barra está honestamente calibrada (§4.2.4). O que reprova é o
> **documento**: um gate nasce vermelho como está escrito, dois gates apontam para um ficheiro que a
> própria espec descreve em dois regimes opostos, sete endereços de fixtura não existem, uma coluna
> inteira de tabela e a medição citada de um gate não re-derivam, e o corpus **não é regenerável
> pelo próprio cabeçalho** no eixo que o G-4 mede. A janela I implementaria sobre isso.

### §4.2.1 — O que foi CONFIRMADO (o que eu tentei quebrar e NÃO consegui)

Re-derivado **exactamente**, com implementação independente da régua (escrita só do README):

| o que | resultado |
|---|---|
| **a escada inteira do §5.1** | **21 de 21 células** batem à 4.ª casa — ⭐ o teste mais forte do corpus: a régua do README está descrita com detalhe suficiente para um terceiro a reconstruir e obter os mesmos 21 números |
| §3.2 (`media|dx|`, `|dy|`, `|dz|`, razão `4,37`, maior deslocamento `0,0336`) | batem ao dígito |
| §3.3 as três faixas de ângulo (`14,0/16,7 · 38,8/29,8 · 9,8/16,1`) | batem, e **só** na região `raio/2` |
| §3.4 (`186` com pente, `139` sem) | batem |
| §4.1 a tabela das quatro rotações (8 valores de `Q`) | batem |
| §4.3 os degenerados (`49`/`56`/`70`/`53`) + a igualdade byte-a-byte | batem |
| §5.2 as duas tabelas (com refino e sem) | batem |
| §6.2 `SMOOTH +0,027` · `LAYER +0,203`, conjunto de arestas invariante | batem |
| §6.3 os CINCO byte-idênticos **e** o controlo «o verbo agiu» | batem |
| §7 seis das oito linhas · §8 `g_quads` e `g_furo` | batem |
| §9 a banda (`2525` × 6 · `2444`–`2448` · amplitude `0,011135`) | bate |
| §2.1 o `sha256 5db23f…` | **bate** — é o corpo depois do cabeçalho (as linhas sem `#`). ⚠️ a espec **não diz o que foi hasheado**; tive de descobrir a receita por varredura de 6 definições |
| §13 **G-10** (monotonia nas 3 rotações) e **G-11** (saturação `≤` banda) | passam |
| §13 **G-13** (`ΔS` troca de sinal: `+0,0183 · −0,0111 · +0,0211 · +0,0012`) | bate |
| **o VALE e a BARRA** | `[+0,02982270 , +0,06323128]`, largura `0,0334`, meio `+0,046527` ⇒ `+0,0465`. As **duas** células extremas são exactamente as que o README nomeia |
| a parede, por instrumento | **10 de 10** vassouras vivas fecham `exit 0` sobre a espec + as 195 fixturas (reproduzido por mim) |
| **isenção nº2** (as 5 de 56) | **LEGÍTIMA, verificada pelo lado que importa**: corri a vassoura do rake contra **as 13 especs** e **os 10 corpora** da pasta — `exit 0` em todos. Já não acusa artefacto atestado nenhum |
| a afirmação do ledger *«`docs/**` fica fora do censo do gate»* | **VERDADEIRA** — conferida no código: `fn ficheiros()` varre `["crates","shells"]`; `docs` só entra no discriminador `nomes_da_nossa_arvore` |

⭐ **E a tese central resistiu a um ataque que o E não montou** — ver §4.2.3.

### §4.2.2 — Os ACHADOS, com endereço

**A-1 ⛔⛔ `G-3` nasce VERMELHO sobre a saída do próprio alvo.** A barra é `Q ≤ +0,0298` e o pior do
lado desligado — `escada/k_a0450_p0000`, a célula que o README nomeia como o **piso do vale** — mede
`Q = +0,02982270`. Margem = **`−0,0000227`**. A barra foi escrita **arredondando o extremo para
baixo** a 4 casas, o que a põe **abaixo do valor de que foi derivada**. ⇒ um gate escrito à letra do
§13 reprova no dia um, sobre produto correcto. (O G-2 não tem esse defeito: margem `+0,0167`.)

**A-2 ⛔⛔ O §7 descreve `composicao/m_manual_*` como o regime OPOSTO ao que o §2.2 lhe atribui — e é
a fixtura de DOIS gates.** A linha *«partir + colapsar»* do §7 dá `−0,046 → +0,122` e cita
`composicao/m_manual_*`; o cabeçalho daquela célula diz `modo_de_detalhe=MANUAL` e ela mede
**`+0,1511 → +0,3572`**, que é o que o §2.2 correctamente chama de *refino **desarmado***. Os números
`−0,046 → +0,122` são os de `verbos/t_constant` / `escada/k_a0000`. ⚠️ **`composicao/m_manual_*` é a
fixtura declarada de `G-5` e de `G-6`** — quem implementar lê a descrição errada do regime.

**A-3 ⛔⛔ O corpus NÃO é regenerável pelo próprio cabeçalho no eixo que o `G-4` mede.** As dez células
`mecanismo/x_man_x01..x16` e as oito `escada/n_x1..n_x8` existem para **variar o número de passagens**
— as saídas de facto diferem (`2525/2648/2627/2627` vértices) — e **todas declaram `passagens=1`**,
com `pontos_do_percurso=10` idêntico. Os cinco cabeçalhos de `x_man_x*` são indistinguíveis campo a
campo. ⇒ uma corrida futura a partir do cabeçalho reproduz **uma** passagem em todos. É **à letra** o
defeito que a 1.ª linha do `fixtures/rake/README.md` avisa contra (*«uma grandeza nova que não entre
no cabeçalho esconde-se debaixo da frase que descreve o corpus, e nenhuma varredura a acusa»*), e
atinge o §3.1, o §5.2 e o **G-4**.

**A-4 ⛔ SETE endereços de fixtura não existem.** §5.2 cita `rotacao/n_x1|x2|x4|x8` — vivem em
`escada/`. §7 cita `composicao/{t_constant,t_relative,t_brush,t_manual,q_res*,s_sim}_*` — vivem em
`verbos/`. São os endereços contra os quais um gate compila.

**A-5 ⛔ A coluna «maior deslocamento» do §3.1 não re-deriva — nenhum dos cinco valores.**

| passagens | espec | medido 3D | medido XY |
|---|---|---|---|
| 1 | `0,02509` | `0,03199` | `0,03186` |
| 2 | `0,02557` | `0,03083` | `0,03039` |
| 4 | `0,02861` | `0,03731` | `0,03651` |
| 8 | `0,03519` | `0,04324` | `0,04051` |
| 16 | `0,03787` | `0,05284` | `0,04090` |

⚠️ A coluna **«vértices que o pente moveu»** da mesma tabela re-deriva **exactamente**
(`170/185/186/187/187`) — mas só com `eps = 1e-7`, que a espec não declara. *Metade da tabela é
reprodutível e a outra metade não, e nada no texto separa as duas.*

**A-6 ⛔⛔ A medição citada do `G-6` não re-deriva.** §3.2 e o §13 dão *«a norma da soma dos Δ sobre a
soma das normas … medido `2,6 %`»*; sobre `composicao/m_manual_*` mede **`3,84 %`** (XY `3,92 %`,
só-Z `2,99 %`). A barra do gate é `≤ 10 %`, logo ele **passa** — mas a única prova de que a barra tem
folga é um número que não existe. *Uma barra larga não é só uma afirmação fraca: é o sítio onde uma
régua errada sobrevive.*

**A-7 ⛔ §9, a «armadilha de instrumento»: o numerador e o denominador vêm de populações DIFERENTES.**
*«`522` de `~4 200` arestas»*: o `522` é a diferença simétrica sobre **todas** as `7 460` arestas da
malha; o `~4 200` é a contagem de arestas **na pegada** (medido `4 292`). A fracção sobrestima o
fenómeno ~`1,8×`. E *«Ordenadas as posições, `10` de `2 525` diferem (`0,4 %`)»* **não re-deriva**:
`269` exacto · `124` a `1e-8` · `2` a `1e-7` · `0` a `1e-6`. Nunca `10`.

**A-8 ⛔ §8: *«a malha mede `1 917` faces diferentes»* (esfera) não re-deriva** — `1 038` de diferença
simétrica (`519` de cada lado; `1 018` por índices ordenados). ⭐ O **facto** que a linha carrega — a
régua planar lê `+0,0453` nos dois lados sobre uma malha que mudou — **confirma-se**.

**A-9 ⛔ §2.2: *«com o efeito MAIOR de todo o corpus»* é contradito pela própria espec.** A célula
citada dá `ΔQ = +0,2061` (`+0,1511 → +0,3572`); o §5.2, três secções abaixo, regista **`+0,3725`** a
4 passagens no mesmo regime (`ΔQ = +0,2188`), e `verbos/v_layer` dá `+0,2026`. A **lei** do §11.3 (o
regime sem refino tem efeito maior que o regime com refino) sobrevive; o **superlativo** não.

**A-10 ⛔ §6.1: *«difere … por `40` a `109`»* — medido `37` a `109`** (`CREASE` = `37`). A frase
seguinte (*«muito acima da banda de ±4»*) continua verdadeira.

**A-11 ⛔ `README.md` §3: a população declarada dá `35` por lado, não `37`.** Os sete critérios
escritos (malha `9fb3d9dea0d0` · dyntopo armada · `CONSTANT` · `SUBDIVIDE_COLLAPSE` · resolução `18`
· percurso inteiro · o controlo `v_saída > 1,5 × v_entrada`) seleccionam `35`/`35`. Largar **qualquer
um** de {malha, refino, resolução} devolve `37`/`37`. ⭐ **O vale não muda** — as duas extremas
reproduzem-se — logo a barra fica de pé; o que não re-deriva é o `n`.

**A-12 ⛔ `README.md` §5, a tabela das famílias, está errada em três células:** `escada/` tem **29**
ficheiros (declara `21`), `rotacao/` **não contém nenhum** `n_x*`, e `verbos/` tem **38** células
`v_*` (declara `42`) mais **20** de composição; **2 dos «21 verbos»** (`ROTATE`, `MASK`) vivem em
`composicao/`. ⚠️ E as células **superadas** `verbos/v_rotate_*` e `verbos/v_mask_*` **continuam no
corpus sem marca nenhuma** (as duas medem *o verbo moveu 0*) ao lado das refeitas `composicao/m_rot_*`
/ `m_mask_*` — é exactamente a confusão *«verbo inerte × verbo que ignora o pente»* que o próprio
README §5 diz ter apanhado e curado.

**A-13 ⛔ PAREDE — §10.1 e §10.2 re-dizem prosa pública do alvo SEM o link que o §4.1.12 exige, e o
cabeçalho da espec afirma que a fonte está registada no §10.** Os itens 3 e 4 dão o número do relato
público; **os itens 1 e 2 não dão fonte nenhuma**, e o §10.2 traz um fragmento em **negrito** que se
lê como paráfrase colada. ⚠️ **Isto não é um achado de colagem — é um achado de INAUDITABILIDADE:**
sem a fonte, nem eu nem o próximo R conseguem decidir se aquilo é um facto re-dito ou uma tradução, e
**a tradução é a única classe que o sweep não apanha por construção** (`CLAUDE.md` §5.1: três especs
desta linha fecharam sweep VERDE sobre prosa traduzida do alvo). A cura é uma linha por item: o link.

### §4.2.3 — A AFIRMAÇÃO CENTRAL: verificada, **mais forte** do que a espec a defende, e com um limite que a espec NÃO declara

**Verificada.** Em `mecanismo/x_man_x01..x16`, com `1·2·4·8·16` passagens, o conjunto de arestas é
**idêntico** entre pente `0` e pente `1` — e é idêntico à **malha de entrada**: a lista de faces das
dez células é, literalmente, a lista de faces da entrada (`841` V, `1 568` F, `2 408` E).

**A régua é invariante à ordem, e a resposta é mais forte do que «a ordem calhou bater»:** naquele
regime a contagem de vértices é constante e igual à da entrada, logo o índice `i` denota o **mesmo**
vértice dos dois lados — e os `671` vértices que não se movem são byte-idênticos no mesmo índice. A
outra régua (`Q`) é uma estatística de distribuição, invariante à ordem por construção. **O controlo
existe em cada célula** (o verbo moveu `187`; o pente moveu `170`–`187`).

⭐⭐ **E o ataque que montei contra a afirmação DEVOLVEU-LHE uma prova melhor do que a que ela usa.**
A objecção óbvia é: *no modo manual o passe de topologia do alvo não corre, logo o teste não
distingue «o pente não troca diagonais» de «trocar está atrás do passe que foi desligado»*. Varridas
as `161` células da malha canónica, existem células em que o passe está **ARMADO e autorizado a
subdividir** e cuja lista de faces sai **igual à da entrada dos DOIS lados**:

| célula | dyntopo | refino | detalhe | F == F(entrada) | `ΔQ` do pente |
|---|---|---|---|---|---|
| `verbos/v_layer_*` | armada | `SUBDIVIDE_COLLAPSE` | `CONSTANT` | **sim, nos dois lados** | **`+0,2026`** |
| `verbos/v_smooth_*` | armada | `SUBDIVIDE_COLLAPSE` | `CONSTANT` | **sim, nos dois lados** | `+0,0271` |

⇒ com o operador de topologia do alvo **armado e com autoridade**, o pente reorganizou a malha ao
ponto de mover `Q` em `+0,2026` e **não mexeu numa única aresta**. Isso mata a objecção do modo
manual. ⚠️ **O §6.2 diz isto numa linha de passagem** (*«são o §3.1 outra vez, por outra porta»*) —
ele é a prova **principal** e devia estar no §3.1, ao lado da tabela.

⛔⛔ **O LIMITE, que o §12 não lista e que decide o desenho.** Em **todas** as `161` células da malha
canónica o conjunto de arestas é ou (a) idêntico ao da entrada nos dois lados, ou (b) **incomparável**
(as contagens de vértices diferem). **Não existe uma célula em que o operador de topologia do alvo
tenha demonstravelmente trabalhado E o conjunto de arestas seja comparável** — e não pode existir, por
construção. ⇒ o corpus prova *«quando o operador de topologia do alvo não faz nada, o pente não muda
uma aresta»*; ele **não** prova a frase do §3.1 sem qualificação, e **não** prova o §11.1
(*«a topologia diferente é obra do passe de refino a decidir sobre geometria já relaxada»*), que é uma
**inferência sobre o interior do alvo**, não uma medição. Se no alvo o pente também **enviesar as
decisões de troca do passe de refino**, a nossa implementação só-relaxação diverge exactamente no
regime que o artista usa — e **nenhum dos 14 gates do §13 o veria**, porque com o refino armado a
nossa topologia não bate com a dele de qualquer maneira. ⇒ **item novo obrigatório no §12**, e o
título do `G-4` não pode prometer mais do que a fixtura contém.

### §4.2.4 — A BARRA: **sã**, e verificada pelo lado que costuma faltar

`VALE = [+0,02982270 , +0,06323128]`, largura `0,0334`; `banda` medida `0,011135` ⇒ `vale/banda =
3,0` ✓. **Os dois lados são saída do próprio alvo**, e o lado «aprovado» é o alvo **com o pente no
máximo** — a condição do `CLAUDE.md` §0.9 que retirou duas barras do corpus do tecido está cumprida.
A banda de repetição está medida nos **dois** regimes (com refino `0,0111`; sem refino o alvo é
determinístico, `Q` repete a `3e-5`). ⚠️ O único defeito é o **arredondamento** do `G-3` (A-1), não a
calibração.

### §4.2.5 — TRIAGEM DOS ACHADOS DE VASSOURA (é do R; aqui está o veredito de cada um)

**T-1 — isenção nomeada nº1 (o rótulo público nu).** ✅ **LEGÍTIMA.** Uso nominativo, §4.1.13 +
§8.4 (*«docs internos citam à vontade»*), mesma forma já atestada no cabeçalho da
`SPEC_pincel_afiado.md`. O controlo é automático e eu corri-o: a espec traz o rótulo em claro na
linha 8 e as 10 vassouras fecham `exit 0` ⇒ a entrada saiu mesmo. ⛔ E a decisão do E de a substituir
por **três formas de prosa** é a troca certa: o que a parede guarda é o texto do alvo, não a palavra
que o artista lê no painel.

**T-2 — isenção nomeada nº2 (5 de 56 entradas retiradas).** ✅ **LEGÍTIMA, e verificada pelo teste
que a decide:** corri a vassoura do rake, como ela está hoje, contra **as 13 especs** e **os 10
corpora** desta pasta — `exit 0` em todos. Já não há entrada que acuse artefacto atestado. ⭐ A lei
que o E escreveu ao lado (*uma vassoura nova tem de correr sobre a árvore inteira antes de ser
adoptada, senão acusa o passado*) é a leitura certa do §5.0.

**T-3 — `docs/3D/20_divergencias_tools.md:422`.** ⛔ **É DÍVIDA REAL, e o ledger SUBCONTA-A.** Medido:
a vassoura do **rake** acusa **duas** linhas naquele ficheiro (`422` **e** `454`), não uma; e **7 das
10** vassouras vivas acusam-no, com **87** ocorrências no total (`-pull` 28 em 15 linhas ·
`-trim-pincel` 19 em 9 linhas · `-pincel-afiado` 15 · `-unblocked` 10 · `-cloth` 9 · `-boundary` 2 ·
`-rake` 4). ⇒ aquele ficheiro é a **maior concentração de nomes internos de alvo restrito da árvore**,
e não uma linha isolada. ⚠️ **Nem tudo ali é dívida:** a coluna do SculptGL é **MIT** e a atribuição
**fica** (a triagem é por ARTEFACTO, com a licença lida nele); parte dos acertos da `-pull` é o
falso positivo do T-4. Mas a coluna do alvo restrito carrega nomes internos de função/ficheiro, e o
ficheiro é, por confissão do próprio E, *«o estudo comparativo da época em que se lia o fonte»*.
✅ **E a afirmação do ledger de que o gate não o vê é VERDADEIRA** — conferi no código: o censo varre
`["crates","shells"]`, e `docs` só alimenta o discriminador de nomes próprios. ⇒ **NÃO é isenção; é
dívida com dono.** Ela **não é desta obra** (zero adições) e **não se cura aqui**: é um acto de
reescrita à maneira da `SPEC_reescrita_dos_comentarios_com_nomes_do_alvo.md`, sobre um doc de outro
assunto. ⛔ O que **é** desta obra é corrigir a contagem: o ledger diz «uma citação, linha 422».

**T-4 — `tip_roundness` em 7 ficheiros.** ✅ **ISENÇÃO LEGÍTIMA — e a decisão JÁ ESTÁ TOMADA noutro
ledger; o que falta é executá-la.** O `LEDGER_blender-pull.md` (§ da reescrita) já triou este token,
com medição: é **propriedade pública** do alvo alcançável pela linguagem de script dele (⇒ §4.1.13),
é uma expressão de **duas palavras comuns** que este repo cunhou por conta própria, e hoje é um
**rótulo de interface** com chave de i18n e dois `NodeId` hasheados da string. O mesmo ledger regista
que **a vassoura contradiz o atestado que a acompanha**: o R-pré daquela obra declarou admissíveis os
identificadores públicos e o E que colheu a vassoura descartou **8** entradas por esse exacto motivo
— *esta devia ter sido a 9.ª*. ⇒ **Veredito: a entrada é o defeito, não o código.** A cura é remover
`tip_roundness` de `VASSOURA_blender-pull.txt`, e o acto é do **E/R da obra `-pull`** — ⛔ não desta
linha, e ⛔ não do agente de reescrita. Até lá o `exit 1` daquela vassoura sobre a família é **SABIDO**.
⇒ **Não é dívida da `line/sculpt3d`. O item pode sair da lista de abertos dela.**

**T-5 — os 13 ficheiros acusados em 5 de 9 varreduras (handoff §58.4-bis).** ⛔ **Ruído de vassoura e
dívida de outras obras, misturados — e a reconciliação do número está feita:** `8` dos `13` são da
vassoura `-pull` e são **o mesmo token do T-4** (`ids/sculpt3d.rs`, `rows.rs`, `brush_default.rs`,
`brush_magnitudes.rs`, `brush.rs`, `stroke_shape.rs`, `verb_strip_law_tests.rs`,
`verb_strip_tests.rs` — os oito ficheiros de produto que carregam `tip_roundness`) ⇒ **caem com o
T-4, sem tocar numa linha de código**. O 13.º (`ph2d-mesh-bool/src/lib.rs`) é a **isenção nomeada do
§44.8** (API pré-existente da dependência Apache-2.0) e o próprio handoff o diz. Ficam **4** por
triar, de **três** obras que não são esta (`-cloth`/`-trim-pincel` sobre `verb_scrape_tests.rs`,
`-pincel-afiado` sobre `brush_verb_filter.rs`, `-trim` sobre três ficheiros da `ph2d-app-sculpt3d`).
⚠️ E a discrepância «três em 16/09, treze em 17/09» explica-se **sem mistério**: o conjunto de
caminhos alargou quando a §65 tocou a `ph2d-mesh-bool`, e a vassoura do `-pull` foi colhida depois —
*o sweep é propriedade do trio (código, vassoura, conjunto de caminhos)*. ⇒ **zero adições desta
linha confirmadas; nenhum dos 13 bloqueia esta obra.**

### §4.2.6 — O que o E tem de emendar para o atestado passar

⛔ **Acto do E, nunca do R nem do I** (esta auditoria não tocou uma linha da espec além do cabeçalho):

1. **A-1** — reescrever a barra do `G-3` acima do extremo que a gerou (ou declarar o extremo como o
   valor e usar `≤`), com a margem escrita ao lado. Como está, o gate reprova sobre o alvo.
2. **A-2** — corrigir a linha *«partir + colapsar»* do §7 (fixtura e números) e reconferir que
   `G-5`/`G-6` descrevem o regime da célula que citam.
3. **A-3** — **pôr o número de passagens no cabeçalho de cada célula** e regenerar/re-emitir as 18
   afectadas; sem isso o `G-4` mede um corpus que ninguém consegue reproduzir.
4. **A-4** — os sete endereços de fixtura.
5. **A-5 / A-6 / A-7 / A-8** — re-medir e reescrever (ou apagar) cada número; e declarar o `eps` das
   contagens de «movidos» e a receita do `sha256` do §2.1.
6. **A-9 / A-10 / A-11 / A-12** — corrigir o superlativo, a faixa `40`–`109`, o `n = 37` e a tabela
   das famílias do README; e **marcar no cabeçalho** as duas células superadas `v_rotate_*`/`v_mask_*`.
7. **A-13** — o **link** dos itens 1 e 2 do §10 (§4.1.12), ou re-escrevê-los como facto sem fonte
   declarada — mas então o cabeçalho não pode dizer que a fonte está registada no §10.
8. **§4.2.3** — acrescentar ao **§12** o limite nomeado (*o corpus não separa «o pente não troca» de
   «o pente enviesa as decisões do passe de refino»*), e promover as células `v_layer`/`v_smooth` de
   nota de rodapé do §6.2 a prova principal do §3.1.

⭐ **Nada nesta lista é de desenho.** Fechados os oito pontos, a espec passa: a tese é sólida, a régua
é reprodutível por terceiros e a barra é honesta.

### §4.2.7 — Actualização da corrente

| papel | id / data | o que fez |
|---|---|---|
| **R-pré** | subagente-R independente, 2026-09-17 | auditoria §4.2 — ⛔ **REPROVADA**, 13 achados (§4.2.2) + 5 triagens (§4.2.5) + o limite não declarado (§4.2.3) |

⚠️ **Higiene desta auditoria:** nada do alvo tocou o disco. As vassouras correram pelo
`cleanroom-sweep.sh` (descodificação em memória), a saída foi consumida **em cano** e deste ledger
saem apenas **contagens e números de linha** — nunca os termos que casaram. Os scripts de
re-derivação viveram no scratchpad da sessão, fora da árvore.


---

## §4.3 — RESPOSTA DO E À 1.ª PASSAGEM DE R-PRÉ (2026-09-17)

⛔ **Acto do E.** O R não emendou nada além do cabeçalho da espec, como manda a SKILL. ⚠️ **Toda a
tabela de números da espec foi re-derivada das fixturas PUBLICADAS** por um conferidor escrito só a
partir do `README` delas — `44` asserções, todas verdes.

| ponto do §4.2.6 | o que se fez |
|---|---|
| **1 (A-1)** | ⭐ **a barra passou a ser UMA SÓ, a do MEIO do vale** (`+0,0465`), com `≥` de um lado e `≤` do outro e margem **`±0,0167`** escrita ao lado. A lei que faltava está na espec §13 e no README §3: *uma barra tirada de um EXTREMO tem margem zero por construção, e é o arredondamento que lhe decide o sinal* |
| **2 (A-2)** | a linha *«partir + colapsar»* do §7 passa a citar `verbos/t_constant_*` + `escada/k_a0000_*` (`−0,0456 → +0,1291`, re-derivados), com a razão escrita: `composicao/m_manual_*` é a fixtura de **G-5/G-6** e estava descrita no regime oposto |
| **3 (A-3)** | ⭐⭐ **corpus RE-EMITIDO**: o cabeçalho passa a declarar `PASSAGENS=`, `rotacao_do_traco_graus=`, `comprimento_do_percurso=`, `percurso_invertido=`, `forma_do_percurso=`, `pontos_usados_truncagem=`, `malha_familia=`, `malha_params=` e, onde há, `SEQUENCIA_DE_TRACOS=`. As 18 células de passagens ficam distinguíveis campo a campo |
| **4 (A-4)** | os sete endereços corrigidos (`escada/n_x*`, `verbos/{t_*,q_res*,s_sim}`), conferidos contra a árvore |
| **5 (A-5/6/7/8)** | a coluna do §3.1 passa a ser a **norma** de `Δ` (`0,03199`…`0,05284`) com `eps = 1e-7` declarado; `G-6` passa a `3,84 %`; o §9 nomeia as duas populações (`522` de **`7 460`** = `7,0 %`) e dá a escada de tolerâncias (`266 · 118 · 2 · 0`); a esfera passa a `1 260` de **diferença simétrica de arestas**; a receita do `sha256` do §2.1 está escrita |
| **6 (A-9/10/11/12)** | superlativo trocado, com os dois contra-exemplos; faixa `37`–`109` com os verbos nomeados; **`n = 34`/`34`** com os **nove** critérios escritos como CHAVES DE CABEÇALHO; a tabela das famílias **contada** da pasta; e as duas células superadas trazem `SUPERADA_POR=`/`SUPERADA_PORQUE=` |
| **7 (A-13)** | os quatro itens do §10 trazem **fonte**; os dois que não tinham ganharam o endereço público, com ⛔ a lembrar que o Implementador não o abre |
| **8 (§4.2.3)** | ⭐⭐⭐ **o limite não foi só declarado: foi MEDIDO** (abaixo), e as células `v_layer`/`v_smooth` subiram de nota de rodapé do §6.2 a **prova principal do §3.1** |

### ⭐⭐⭐ O limite do §4.2.3, MEDIDO — e a inferência do §11.1 está REFUTADA

O R nomeou o buraco certo. Mediu-se **obrigando o próprio alvo a fazer as duas metades em série**
(família nova `lei_unica/`, 26 células, cada configuração `2`–`3` vezes):

| composição | `ΔQ` | % do alvo |
|---|---|---|
| **JUNTO** (o alvo, como o artista o usa) | **`+0,1670`** | `100 %` |
| em série `1×` | `−0,2081` | `−125 %` |
| em série `3×` | `−0,0496` | `−30 %` |
| em série `4×` | `−0,0128` | `−8 %` |
| em série `9×` (o mais fino que a porta sem interface alcança) | `+0,0084` | **`5 %`** |

⇒ **NÃO é uma lei só.** A composição em série converge para **zero**, não para o alvo, e a diferença
é **14×** a banda de repetição (`0,0115`). As duas causas possíveis ficam **nomeadas e não
separadas**, com **o instrumento que as separaria escrito na espec §14.4** — e a impossibilidade de
as separar por esta porta é ela própria **medida**: o gesto público é um traço inteiro, logo `9×` é o
chão da granularidade.

### Correcções às triagens, pedidas pelo R

- **T-3** — ⛔ este ledger **subcontava**: não é *«uma citação, linha 422»*. Medido pelo R: **87**
  ocorrências em `docs/3D/20_divergencias_tools.md`, em **7 das 10** vassouras vivas (`-pull` 28 ·
  `-trim-pincel` 19 · `-pincel-afiado` 15 · `-unblocked` 10 · `-cloth` 9 · `-boundary` 2 · `-rake` 4,
  estas em **duas** linhas: `422` e `454`). É a maior concentração de nomes internos de alvo restrito
  da árvore. **Não é desta obra** (zero adições) e **não se cura aqui**.
- **T-4** — o token de duas palavras da obra `-pull` (⛔ **não o escrevo: a vassoura dela acusa-o, e
  escrevê-lo aqui era reproduzir em vez de descrever — §6.1**; ele está nomeado no
  `LEDGER_blender-pull.md`) **sai da lista de abertos desta linha**: já está triado lá, com medição, e
  o veredito é que **a entrada da vassoura é que é o defeito**, não o código. O acto é do E/R daquela
  obra. ⚠️ **Auto-achado:** a 1.ª redacção desta mesma linha escrevia-o em claro e fez a vassoura
  `-pull` acusar **este ledger** — a segunda vez, na mesma sessão, que a regra nº 2 do topo mordeu
  quem a escreveu. *Uma triagem que NOMEIA o token acusado transforma o registo dela num achado.*

### Corrente

| papel | id / data | o que fez |
|---|---|---|
| **E** (emenda) | subagente-E, 2026-09-17 | curou os 8 pontos · re-emitiu o corpus (`207` células + `14` entradas = 221 ficheiros) · **mediu** o §14 · re-derivou 44 números das fixturas publicadas |
| **R-pré, 2.ª passagem** | subagente-R independente, 2026-09-17 | ⛔ **REPROVADA** — ver §4.4 no fim deste ficheiro (8 achados, 2 materiais; os 8 pontos da 1.ª passagem curados e conferidos) |

⚠️ **Higiene da emenda:** as corridas novas do oráculo viveram em `~/Referencias/blender-rake/`. O
sweep das dez vassouras fecha **`exit 0` sobre o conjunto que o Implementador vê** — a espec
emendada, as 221 fixturas e o INBOX.

⛔⛔ **E sobre ESTE ficheiro a vassoura `-pull` fecha `exit 1`, o que é SABIDO e está declarado:** ela
acusa **três** linhas da §4.2.5 escritas pelo **próprio R** (a triagem T-4, que nomeia o token ao
declarar que **a entrada da vassoura é que é o defeito**) e **uma** que era minha e foi curada.
⛔ **As três do R NÃO se emendam:** reescrever o texto de uma auditoria para a fazer passar o
instrumento que ela própria julga seria falsificar o registo. ⇒ fica **condição declarada**, com a
cura no dono certo (o E/R da obra `-pull`, que retira a entrada da vassoura dele).
⭐⭐ **E o par de auto-achados desta sessão é a lei, não o incidente:** a regra nº 2 do topo deste
ficheiro mordeu **duas vezes quem a escreveu** — uma ao registar o canário do controlo positivo, outra
ao registar uma triagem. *Um registo que NOMEIA o termo acusado converte-se, ele próprio, num achado;
o §6.1 existe exactamente para isso e aplica-se ao ledger antes de se aplicar a qualquer outra coisa.*

---

## §4.4 — AUDITORIA R-PRÉ, **2.ª PASSAGEM**: ⛔ **REPROVADA** (subagente-R independente, 2026-09-17)

> ⛔⛔ **Esta passagem NÃO herdou o «limpo» da 1.ª.** A emenda mexeu na espec inteira e re-emitiu as
> 195 células, logo **toda linha é população nova**. Re-medi de raiz, com o meu próprio conferidor,
> e varri por **FORMA** (não por endereço nomeado). É a lei que esta linha pagou cinco vezes numa
> espec só: *numa espec já auditada, toda linha nova é população nova a auditar.*
>
> ⭐ **Os OITO pontos do §4.2.6 estão curados, e verifiquei cada um.** O que reprova é **curto**: duas
> frases de conclusão da §14 nova, e quatro números residuais. A fatia de substância — a tese, a
> régua, a barra, os gates e a medição nova da §14 — resiste inteira.

### §4.4.1 — O RISCO MAIOR da emenda NÃO se materializou (e era este)

A cura do **A-3** obrigou a mexer nos 195 ficheiros. A pergunta que decide tudo o resto é *«a
re-emissão mudou SAÍDAS, e a espec ficou a citar as antigas?»*. **Medido, ficheiro a ficheiro, contra
o commit `2a44662aa`:**

| | |
|---|---|
| células com **corpo byte-idêntico** ao de antes | **195 de 195** |
| células com corpo diferente | **0** |
| células novas | **26** (a família `lei_unica/`) |

⇒ a re-emissão **enriqueceu cabeçalhos e não tocou numa malha**. Nenhum número citado ficou órfão,
e as minhas medições da 1.ª passagem continuam a valer para as 195. ⚠️ **Nota de vocabulário:** o
ledger diz *«corpus RE-EMITIDO»*, que se lê como *«re-corrido no alvo»*; o que houve foi cabeçalhos
reescritos sobre malhas preservadas — que é a opção **mais segura** (zero risco de deriva do binário)
e merece ser dita assim.

### §4.4.2 — Os oito pontos, conferidos um a um

| ponto | veredito | o que eu medi |
|---|---|---|
| **A-1** barra | ✅ **curado, e melhor do que eu pedi** | barra **ÚNICA** `+0,046527 → +0,0465`, do meio do vale. Corri os dois gates sobre a população inteira: **zero reprovações** dos dois lados, em **todas** as três leituras de população (32, 34 e 35 células). Margens reais: `+0,0167` (pior ligada `+0,06323128`) e `−0,0167` (pior desligada `+0,02982270`) |
| **A-2** §7 | ✅ curado | a linha *«partir + colapsar»* cita agora `verbos/t_constant_*` + `escada/k_a0000_*` e mede `−0,0456 → +0,1291` — re-derivado por mim ao dígito; `m_subdiv` `−0,0327 → +0,1033` ✓ |
| **A-3** cabeçalhos | ✅ **curado** | `PASSAGENS=` traz o valor REAL nas dezoito (`1·2·4·8·16` e `1·2·4·8`), e as cinco `x_man` ficam distinguíveis **campo a campo**. Mais oito chaves de regeneração (rotação · comprimento · inversão · forma · truncagem · família · params · sequência) |
| **A-4** endereços | ✅ curado | conferi os sete contra a árvore: todos existem |
| **A-5** coluna §3.1 | ✅ curado | a coluna é agora a **norma** de `Δ` e bate **exactamente** com a minha medição (`0,03199 · 0,03083 · 0,03731 · 0,04324 · 0,05284`), com `eps = 1e-7` declarado |
| **A-6** `G-6` | ✅ curado | `3,84 %` ✓ (e o `96,2 %` que o acompanha ✓) |
| **A-7** §9 | ⚠️ **meio curado** | as duas populações estão nomeadas e certas (`522` de **`7 460`** = `7,0 %`; pegada `1 852`) ✓ — **mas ver B-3** |
| **A-8** esfera | ✅ **curado, e exacto** | `1 260` é a diferença simétrica de **arestas** (`2 032` F e `3 048` E dos dois lados) — re-derivei o número ao inteiro |
| **A-9/10/12** | ✅ curados | superlativo trocado com os dois contra-exemplos ✓ · faixa **`37`–`109`** com `CREASE`/`NUDGE` nomeados ✓ · tabela de famílias contada da pasta ✓ · as duas células superadas trazem `SUPERADA_POR=` ✓ |
| **A-11** população | ⚠️ **ver B-4** |
| **A-13** parede | ✅ **curado** | os **quatro** itens do §10 trazem fonte, com a advertência de que o I não abre os endereços. A classe que o instrumento não apanha passa a ser auditável |
| **§4.2.3** | ✅ **curado, e excedido** | `v_layer`/`v_smooth` subiram a **prova principal do §3.1** ✓, e o limite deixou de ser um item por declarar: foi **medido** (§14) |

### §4.4.3 — A §14 (a medição que decide): o NÚMERO resiste, a CONCLUSÃO não

**Re-derivei a tabela inteira com o meu próprio conferidor — os cinco `ΔQ` batem ao dígito:**

| composição | traços p0/p1 | `Q(p0)` | `Q(p1)` | `ΔQ` medido | espec | passa o `G-2`? |
|---|---|---|---|---|---|---|
| **JUNTO** | 1 / 1 | `−0,0456` | `+0,1214` | **`+0,1670`** | `+0,1670` | ✅ **sim** |
| série `1×` | 2 / 2 | `−0,0110` | `−0,2190` | `−0,2081` | `−0,2081` | ✗ |
| série `3×` | 6 / 6 | `−0,0137` | `−0,0633` | `−0,0496` | `−0,0496` | ✗ |
| série `4×` | 8 / 8 | `−0,0106` | `−0,0234` | `−0,0128` | `−0,0128` | ✗ |
| série `9×` | 18 / 18 | `−0,0057` | **`+0,0028`** | `+0,0084` | `+0,0084` | ✗ |

✅ **O CONTROLO é honesto, e testei-o pelo lado que o poderia derrubar:** em cada configuração a
`SEQUENCIA_DE_TRACOS` do lado `p0` é **estruturalmente idêntica** à do `p1` com o pente a zero em
**todos** os traços, e as contagens de traços batem (`1/1 · 2/2 · 6/6 · 8/8 · 18/18`). ⇒ o trabalho
extra do verbo está subtraído. Amplitude máxima de repetição das seis: **`0,0115`** ✓.
⭐ **E o veredito nem precisa do `ΔQ`:** em valor **ABSOLUTO** o JUNTO chega a `+0,1214` (passa a
barra do `G-2`) e a série chega, no melhor caso, a `+0,0028` (falha). *Isso torna a conclusão imune à
objecção de que os `ΔQ` partem de linhas de base diferentes* (`−0,0456` contra `−0,006`..`−0,014`) —
e é a forma que devia estar escrita, porque é a que o `G-2` mede.

✅ **O 2.º canal É independente do `Q`** (era a dúvida): a `9×` o `Q` diz *ligeiramente melhor*
(`+0,0084`) e os irregulares dizem *ligeiramente pior* (`0,741 → 0,754`). Dois canais que discordam
no sinal no mesmo ponto não são o mesmo facto lido duas vezes. **Ver B-8.**

✅ **O CHÃO do `9×` é REAL** — e o argumento forte está no corpus e **não** é o que a espec dá.
Medido do cabeçalho (`percurso 1,400` · `raio 0,350` · `espaçamento 10 %` ⇒ passo `0,0700`):

| granularidade | troço | carimbos por troço |
|---|---|---|
| `4×` | `0,3500` | `5,00` |
| **`9×`** | `0,1556` | **`2,22`** |
| `18×` | `0,0778` | **`1,11`** ⇒ **abaixo de dois** |

E a **§4.3 desta mesma espec** mede que com **menos de dois carimbos o pente é INERTE e a saída é
byte-idêntica**. ⇒ ir mais fino que `9×` zeraria o `ΔQ` **pela lei da inércia**, não por a composição
falhar. *Com esse argumento o chão é uma propriedade MEDIDA do alvo; com o argumento que está escrito
(«o gesto público é um traço inteiro») ele é uma limitação do arnês.* **Ver B-7.**

### §4.4.4 — Os ACHADOS desta passagem

**B-1 ⛔⛔ *«a composição em série converge para ZERO»* é contradito pelos próprios dados — e pela
§14.5.2, três parágrafos abaixo.** A sequência de `Q(p1)` é `−0,2190 → −0,0633 → −0,0234 → +0,0028`:
**monótona crescente**, e no ponto mais fino ela **já passou** o zero. Uma sequência que cruza zero e
continua a subir não *converge* para zero — o zero é onde ela **passou**. E a §14.5.2 diz do mesmo
`+0,0084` que ele *«sugere estar a convergir»* (para o alvo, que é o que justifica recomendar a
hipótese **(b)**). **As duas frases não podem ser ambas verdadeiras**, e a contradição está sobre a
frase que decide o desenho: se converge para zero, **(b)** é inútil e só resta **(a)**; se está a
convergir, **(b)** é a aposta certa — que é o que a §14.5 manda fazer.
⭐ **E o argumento FORTE está no corpus do E e ele não o extraiu:** ajustando `ΔQ` em `1/n` sobre os
quatro pontos, o limite em granularidade infinita dá **`+0,0393`** — ainda **abaixo** da barra do
`G-2` (`+0,0465`). *Isso* é o que sustentaria «falta uma segunda lei», e é falível (quatro pontos),
logo tem de ir escrito como indicação com a sua própria fraqueza ao lado — nunca como veredito.

**B-2 ⛔⛔ O título da §14 e o §11.1 concluem *«NÃO é uma lei só»*, e a medição não sustenta essa
conclusão — a §14.4 di-lo por escrito, no mesmo documento.** A §14.4 nomeia duas causas e declara
que o instrumento **não as separa**; e a causa **(b)** — *«a relaxação tem de correr DENTRO do laço
por-carimbo»* — é explicitamente *«uma questão de ONDE ela corre, não de que lei é»*. Sob (b) é
**uma** lei, no sítio certo. O que a medição sustenta é: *«compor as duas metades EM SÉRIE, na
granularidade que o instrumento alcança, não reproduz o alvo — e falha o `G-2` em todas as
granularidades medidas»*. Isso é uma afirmação sobre **COMPOSIÇÃO**, não sobre o **número de leis**.
⛔⛔ **É a mesma forma de defeito que esta emenda acabou de curar:** o §11.1 tinha uma inferência
sobre o interior do alvo, ela foi refutada, e no lugar dela entrou **outra inferência** do mesmo
tipo — agora no §11, que se chama *«As leis que a MEDIÇÃO deu»*. ⚠️ *Uma inferência refutada
substituída por outra inferência não é progresso.*
⚠️ **E o dano é concreto:** um Implementador que leia o §11.1 primeiro (*«não é uma lei só … reproduz
ZERO»*) conclui que a relaxação é inútil e vai procurar a segunda lei **antes** de tentar a (b) — o
inverso da ordem que a própria §14.5 prescreve.

**B-3 ⛔ §9: o `266 · 118` não re-deriva** (a 1.ª redacção dizia `10`; a correcção landou noutro
número). Toda definição natural dá **`269` · `124`**: elemento-a-elemento após ordenar `269` ·
`|A − B|` `269` · `|A ^ B| / 2` `269` · multiconjunto `269`; e `124` acima de `1e-8`. As duas outras
colunas (`2` acima de `1e-7`, `0` acima de `1e-6`) batem. Conferi também os outros pares de corridas
(`1-3` dá `380`, `2-3` dá `370`): **nenhum** par dá `266`.

**B-4 ⛔ `README` §3: aplicando os NOVE critérios À LETRA dá `32`/`32`, não `34`/`34` — e isso
falsifica a frase que ANUNCIA a cura.** O README escreve: *«todos os nove critérios são chaves do
cabeçalho, de propósito: a 1.ª redacção descrevia-os em prosa e um terceiro a aplicá-los obteve outro
`n` que o declarado»* — e um terceiro a aplicar as nove chaves obtém, outra vez, outro `n`. Largar
**qualquer um** de `malha_de_entrada`, `passe_de_refino` ou `resolucao_do_detalhe` devolve `34`.
⭐ **A barra é ROBUSTA e isso está medido:** as duas células extremas e os dois vereditos são
**idênticos** com `32`, `34` e `35`, com **zero** reprovações em qualquer delas. ⇒ é escrituração, não
segurança — mas é a **terceira** redacção seguida deste mesmo número, agora depois da cura desenhada
para o tornar reprodutível.

**B-5 ⛔ §3.2: *«o comprimento médio de aresta (`0,0995`)»* não re-deriva** — sobre a entrada mede
`0,0990`, sobre a saída `p000` `0,1002`, e restrito à pegada `0,1060`. É o **denominador** do
`0,34` da linha acima dele.

**B-6 ⚠️ §7: dois dos quatro `ΔQ` de `q_res` arredondam no sentido errado** — medido `+0,109557`
(escrito `+0,1095`) e `+0,167718` (escrito `+0,1678`). Os outros dois batem. *Sinal de que o
conferidor trunca onde o texto arredonda; vale a pena fixar a regra, porque 44 asserções passaram.*

**B-7 ⚠️ A §14 sub-argumenta o chão do `9×`** — ver a tabela de carimbos por troço no §4.4.3. O
argumento medido (o limiar de **dois** carimbos da §4.3) é mais forte do que o dado e já está no
documento; falta ligá-los.

**B-8 ⚠️ A §14.3 cita só o par do 2.º canal que CONCORDA com o `Q`.** Ela dá JUNTO (`0,791 → 0,755`)
e série `1×` (`0,788 → 0,890`) — os dois pontos onde os dois canais apontam para o mesmo lado. Medido
nas cinco: a `3×` `0,765 → 0,807`, a `4×` `0,758 → 0,836`, e a **`9×` `0,741 → 0,754`**, que é onde o
`Q` diz *melhor* e os irregulares dizem *pior*. ⇒ o canal é mesmo independente (isso confirma-se), mas
a amostra publicada esconde o ponto em que os dois discordam — e é justamente o ponto que decide.

### §4.4.5 — A MINHA §4.2.5 e a vassoura `-pull`: o E TEM RAZÃO, e eis como se regista

⛔ **Medido:** a vassoura `-pull` acusa **exactamente três** linhas deste ficheiro — `421`, `429` e
`437`, todas na **§4.2.5** (as triagens T-4 e T-5), escritas por mim. A linha do E foi curada.
⭐ **E o conjunto que o Implementador vê fecha `exit 0` nas DEZ** (espec emendada + as 221 fixturas +
o INBOX), reproduzido por mim. A parede não está enfraquecida.

**Veredito: o argumento do E procede, e assino-o.** Reescrever o texto de uma auditoria para a fazer
passar o instrumento que ela própria julga **falsificaria o registo** — e por três razões, não uma:
o ledger é, pela regra nº 1 do seu próprio topo, o ficheiro que **carrega rastros de propósito** e
que o I **nunca abre**; a triagem T-4 tem de poder **dizer sobre o que decidiu**, senão deixa de ser
uma decisão auditável; e o token acusado é, pelo próprio `LEDGER_blender-pull.md`, um **falso
positivo reconhecido** — *emendar o registo para contornar um falso positivo é curar o instrumento no
sítio errado*.

⚠️ **Mas a parte difícil da pergunta é a outra, e ela tem resposta em três peças:**

1. ⛔ **O vermelho NÃO é permanente — ele tem dono e tem data.** A cura já está decidida (§4.2.5 T-4,
   e o `LEDGER_blender-pull.md` chegou lá primeiro): **retirar a entrada da `VASSOURA_blender-pull.txt`**.
   No dia em que o E/R daquela obra a executar, este ficheiro fica verde com **zero edições ao texto
   da auditoria**. A condição é *«vermelho até uma obra vizinha executar uma decisão que já tomou»*.
2. ⛔⛔ **O que falta é registo LOCALIZÁVEL POR MÁQUINA, não mais prosa.** A lei está escrita no
   próprio `LEDGER_blender-pull.md`: *«um sweep vermelho por isenção registada e um vermelho por
   dívida leem-se iguais numa corrida»*. Prosa no fim de um ficheiro de 560 linhas não separa os dois
   para quem corre o portão. ⇒ **acrescento abaixo uma declaração de forma fixa**, para que o fecho
   compare **esperado × observado** em vez de vermelho × verde.
3. ⚠️ **E o âmbito que tem de ficar verde é o do Implementador**, que é o que eu medi e está verde.
   *Um vermelho num ficheiro que o I nunca abre não é a mesma coisa que um vermelho no que ele lê* —
   e é exactamente isso que a declaração tem de dizer, senão o próximo integrador ou ignora o
   vermelho (e deixa de ler vermelhos) ou «cura»-o editando a auditoria.

```
SWEEP-ESPERADO: VASSOURA_blender-pull → LEDGER_blender-rake.md → exit=1, 3 achados
  linhas 421 · 429 · 437, todas na §4.2.5 (triagens T-4 e T-5 do R-pré, 1.ª passagem)
  porquê: a triagem NOMEIA o token para declarar que a ENTRADA DA VASSOURA é o defeito
  dono da cura: o E/R da obra `-pull` (retirar a entrada de VASSOURA_blender-pull.txt)
  âmbito do Implementador (espec + fixtures/rake + INBOX): exit=0 nas DEZ — medido 2026-09-17
  ⇒ um fecho compara ESPERADO × OBSERVADO. Divergir deste bloco é que é achado.
```

✅ **E os dois auto-achados estão REGISTADOS e não apagados** (§4.3, fim) — confirmei no ficheiro. A
lei que eles deixam é boa e vale para todo ledger desta casa: *um registo que NOMEIA o termo acusado
converte-se, ele próprio, num achado.*

### §4.4.6 — O que falta para o atestado passar (curto, e nada é de desenho)

⛔ **Acto do E.** Esta passagem não tocou uma linha da espec além do cabeçalho.

1. **B-1 + B-2 — as duas frases de conclusão da §14.** Trocar *«converge para ZERO»* e *«NÃO é uma
   lei só»* pelo que está medido: **«compor as duas metades EM SÉRIE falha o `G-2` em TODAS as
   granularidades que o instrumento alcança (melhor caso `Q = +0,0028` contra a barra `+0,0465`), e a
   tendência é monótona crescente com a granularidade»**. A pergunta *uma lei ou duas* fica **EM
   ABERTO** e com dono (a §14.4 já tem o instrumento). ⭐ O ajuste em `1/n` (`+0,0393`, abaixo da
   barra) entra como **indicação com a sua fraqueza declarada**, e é o que dá força ao caso da
   segunda lei. ⚠️ Corrigir também o §11.1 e a 1.ª célula do §12, que repetem a conclusão.
2. **B-3 · B-4 · B-5 · B-6** — os quatro números: `269`/`124`; o `n` da população (ou as chaves, ou o
   número); o comprimento médio de aresta; e a regra de arredondamento dos dois `ΔQ`.
3. **B-7 · B-8** — ligar o chão do `9×` ao limiar de dois carimbos da §4.3, e publicar as **cinco**
   linhas do 2.º canal em vez de duas.

⭐ **Tudo o resto está atestado por esta passagem.** A tese, a régua de quatro dobras, a barra única,
os catorze gates, o corpus regenerável e a medição nova da §14 são sólidos e re-derivam de terceiros.
*O que falta é redacção sobre números que já existem.*

### §4.4.7 — Corrente

| papel | id / data | o que fez |
|---|---|---|
| **R-pré, 2.ª passagem** | subagente-R independente, 2026-09-17 | ⛔ **REPROVADA** — 8 achados (`B-1`..`B-8`), dos quais **2 materiais**; os oito pontos do §4.2.6 conferidos e **curados**; o corpus provado byte-idêntico nas 195; a §14 re-derivada com conferidor próprio; e o argumento do E sobre a §4.2.5 **aceite**, com a forma de registo acrescentada |

⚠️ **Higiene:** nada do alvo tocou o disco. As vassouras correram pelo `cleanroom-sweep.sh`
(descodificação em memória), a saída foi consumida **em cano**, e deste registo saem apenas
contagens, números de linha e grandezas — nunca os termos que casaram.


---

## §4.5 — RESPOSTA DO E À 2.ª PASSAGEM DE R-PRÉ (2026-09-17)

⛔ **Acto do E.** ⚠️ **Nada da §4.2.5 nem do bloco `SWEEP-ESPERADO:` foi tocado** — o R deu razão ao
argumento e assinou-o, e o bloco dele é o instrumento que separa um vermelho por isenção de um
vermelho por dívida. Esta secção **casa** com ele e não o substitui.

### Os oito achados

| # | o que se fez |
|---|---|
| **B-1** | ⛔⛔ *«converge para ZERO»* **SAIU**. A sequência é `−0,2190 → −0,0633 → −0,0234 → +0,0028`: **monótona crescente**, e no ponto mais fino já **passou** o zero. O que fica escrito é o que a tabela mostra, e a §14.5.2 passou a dizer *«favorece, não demonstra»* em vez de *«sugere estar a convergir»* |
| **B-2** | ⛔⛔ *«NÃO é uma lei só»* **SAIU** do título da §14, do §11.1 e da 1.ª célula do §12. A medição é sobre **COMPOSIÇÃO**; o número de leis **fica EM ABERTO**, com o instrumento na §14.4. ⚠️ *Era a mesma forma de defeito que a emenda anterior curou — uma inferência vestida de medição — e o R tem razão em que substituir uma inferência por outra não é progresso* |
| **B-3** | ⭐⭐ o `266 · 118` deu lugar à **explicação**: as três redacções (`10`, `266`, `269`) são **três PRECISÕES de arredondamento aplicadas ANTES de ordenar**, e o arredondamento **reordena**. A definição publicada é a das triplas **cruas** (`269`), e a métrica do `1e-8` vai dita (`118` na maior componente · **`124`** na norma euclidiana, que é a leitura do R). *A secção é sobre a ordem ser o que difere, e as suas próprias contagens divergiam pela ordem* |
| **B-4** | `n = 32`/`32`, das nove chaves **à letra**, com a receita ao lado e a robustez medida (`32`, `34` e `35` dão as mesmas extremas e **zero** reprovações). ⚠️ É a **terceira** redacção do número e o README di-lo |
| **B-5** | o denominador passa a nomear a **população**: `0,0990` é a aresta média da **malha de entrada** (saída `0,1002`, pegada `0,1060`; o `0,0995` anterior não é nenhuma das três) |
| **B-6** | `+0,1096` e `+0,1677` (arredondados, não truncados), e a **regra** fica escrita no §7 |
| **B-7** | ⭐ o chão do `9×` passa a ser uma propriedade **MEDIDA DO ALVO**: `2,22` carimbos por troço a `9×` contra `1,11` a `18×`, **abaixo do limiar de dois** que a §4.3 mede como o ponto em que o pente é inerte. *Ir mais fino zeraria o `ΔQ` pela lei da inércia, não por a composição falhar* |
| **B-8** | as **cinco** linhas do 2.º canal, **incluindo a que discorda** (`9×`: `0,741 → 0,761`, onde o `Q` diz melhor e os irregulares dizem pior). *Citar só o par que concorda é escolher a testemunha* |

### O que entrou de novo, e é o que dá força ao caso

- ⭐⭐ **O argumento em valor ABSOLUTO**, promovido a coluna que decide: o junto chega a `+0,1214` e
  **passa** o `G-2`; a série, no melhor caso, a `+0,0028` e **falha**. Imune à objecção de que os
  `ΔQ` partem de linhas de base diferentes.
- ⭐ **A extrapolação `1/n`** (`a = +0,0393`, abaixo da barra) como **indicação com a fraqueza ao
  lado** — quatro pontos, forma escolhida e não derivada, declive dominado pelo ponto extremo, e um
  segundo ajuste que dá outro número (`+0,0304`).

### Verificação e higiene

- **40 asserções** re-derivadas das fixturas publicadas (a tabela absoluta, a extrapolação, as dez
  contagens do 2.º canal, os carimbos por troço, a aresta média, os quatro `ΔQ` e as cinco leituras
  da ordem) — todas verdes. ⛔ **Nenhuma fixtura foi mexida nesta passagem.**
- **Varredura por FORMA de raiz**, títulos incluídos e com o vocabulário alargado às palavras de
  CONCLUSÃO (`sustenta`, `converge`, `indica`, `conclui`, `confirma`). ⭐ Ela apanhou **um resíduo
  real**: o número do **A-8** tinha sido curado na espec e **sobrevivia no README das fixturas**
  (`1 917` onde a medição é `1 260` de diferença simétrica de arestas). *Uma cura aplicada a um
  documento e não ao seu par é exactamente o que uma varredura por endereço nomeado não apanha.*
- **Sweep:** `exit 0` nas dez vassouras sobre o âmbito do Implementador (espec + 221 fixturas +
  INBOX). Sobre **este** ficheiro a `-pull` continua a sair `exit 1` nas três linhas da §4.2.5 — que
  é **exactamente o que o bloco `SWEEP-ESPERADO:` acima declara**, e não um achado novo.

### Corrente

| papel | id / data | o que fez |
|---|---|---|
| **E** (2.ª emenda) | subagente-E, 2026-09-17 | as duas conclusões da §14 reescritas para o que a medição sustenta · B-3..B-8 · 40 asserções re-derivadas · varredura por forma de raiz |
| **R-pré, 3.ª passagem** | — | **por despachar** |

---

## §4.6 — AUDITORIA R-PRÉ, **3.ª PASSAGEM**: ✅ **APROVADA** (subagente-R independente, 2026-09-17)

> ⛔⛔ **Não herdei o limpo das minhas duas passagens.** Varri a emenda de raiz, por **FORMA** e com
> títulos incluídos, e re-medi tudo o que ela reescreveu com o meu próprio conferidor.
>
> ✅ **ATESTADO: a espec `SPEC_pente_de_topologia.md`, as `221` fixturas de `fixtures/rake/` e o
> `README.md` delas estão APTOS a ser entregues à janela I.** A parede está intacta, a régua é
> reprodutível por terceiros, a barra é honesta e todo número citado re-deriva do ficheiro
> publicado. **A janela de implementação está ABERTA.**

### §4.6.1 — O que a janela I está autorizada a ler e a construir

| | |
|---|---|
| **LER** | `SPEC_pente_de_topologia.md` · `fixtures/rake/**` (221 ficheiros) · `INBOX_blender-rake.md` (só para escrever, `cat >>`) |
| ⛔ **NÃO LER** | este LEDGER · `VASSOURA_blender-rake.txt` · a denylist de URLs do cabeçalho — **incluindo os dois endereços públicos do §10**, que existem para o revisor e não para o implementador |
| **CONSTRUIR** | a relaxação tangencial do §3.2 · os **14** gates do §13 com as barras como estão · ⭐ e a ordem da §14.5: **(b) PRIMEIRO** — a relaxação **dentro do laço por-carimbo** —, medindo contra o `G-2` |
| ⛔ **NÃO CONSTRUIR** | uma segunda lei antes de (b) estar no sítio e medida. *A pergunta «uma lei ou duas» está EM ABERTO por decisão medida, não por omissão* |

⭐ **Sweep verificado por mim:** as **dez** vassouras vivas fecham `exit 0` sobre exactamente esse
conjunto (espec + 221 fixturas + INBOX).

### §4.6.2 — Os TRÊS pontos em que a medição era minha: ⛔ **PERDI DOIS**

**B-8 — ⛔ EU ESTAVA ERRADO; o número dele está certo.** Eu publiquei `0,741 → 0,754` e ele
`0,741 → 0,761`. Medido corrida a corrida: `wf_n9_p1` lê **`0,7543` (r1)** e **`0,7668` (r2)**, média
**`0,7606` → `0,761`**. ⇒ **eu reportei UMA corrida onde a célula tem duas**, e ele fez a média, que
é a leitura certa num regime que ele próprio mediu como não-repetível. *Uma régua que lê uma
realização de uma família de repetições não é a régua da família.* ⭐ **A substância do achado
sobrevive e ele publicou-a**: a `9×` o `Q` sobe (`+0,0084`) e os irregulares **pioram** (`0,741 →
0,761`) — os dois canais discordam no sinal no ponto que decide, e essa linha está agora na tabela.

**B-3 — ⛔ EU ESTAVA ERRADO, e o achado CAI. A explicação dele é correcta e não é óbvia.** Eu escrevi
*«toda definição dá `269 · 124` e nenhum par de corridas dá `266`»*. **Medido, arredondando ANTES de
ordenar:**

| precisão do arredondamento | posições ordenadas que diferem |
|---|---|
| cru (o ficheiro grava até **12** casas) | **`269`** |
| a `9` casas | **`266`** |
| a `8` casas | `195` |
| a `7` casas | `43` |
| a `6` casas | **`10`** |

⇒ as **três** redacções (`10`, `266`, `269`) são três precisões da **mesma** grandeza, e o
arredondamento **reordena**. ⚠️ **A minha leitura tinha uma premissa escondida — a ordem** —, que é
exactamente o assunto da secção que eu estava a auditar. *Uma auditoria que não aplica ao seu próprio
método a lei que a secção ensina é a forma mais cara de errar aqui.*
⭐⭐ **E ele fechou o único número que eu não conseguia achar:** o `118` que eu procurei em nove
precisões e em quinze pares de corridas **re-deriva** — é o critério da **maior componente**
(`118`), contra a **norma euclidiana** (`124`, a minha leitura). Medi as duas: `118` e `124`, com
`2` acima de `1e-7` e `0` acima de `1e-6` nas duas. A espec publica agora **as duas, com a métrica
nomeada em cada uma** — que é mais do que eu pedi.

**B-7 — ✅ CONFIRMADO, e há uma forma MAIS FORTE do argumento, disponível no corpus.** Medi a §4.3
pelo caminho do produto: `y_umdab` (**1** ponto de percurso) ⇒ o verbo moveu `56` e **o pente moveu
`0`**; `y_doisdab` (**2** pontos) ⇒ o verbo moveu `70` e **o pente moveu `53`**. O limiar de **dois**
está medido. E a aritmética do chão re-deriva do cabeçalho (`percurso 1,400` · `raio 0,350` ·
`espaçamento 10 %` ⇒ passo `0,0700`): `9×` dá **`2,22`** carimbos por troço, `18×` dá **`1,11`**.
⭐⭐ **E a ligação é mais forte do que uma analogia — é uma IDENTIDADE GEOMÉTRICA:** um troço do `9×`
mede `1,4/9 = 0,155556`, e o vão dos dois pontos de `y_doisdab` mede **`0,155556`** (concordam a
`4,4e-7`, que é a precisão com que o ficheiro grava o percurso). ⇒ *a célula que a §4.3 usa para
medir o limiar **É**, geometricamente, um troço do `9×`.* Sugestão ao E, não condição.

### §4.6.3 — A conclusão da §14 ficou HONESTA: as quatro passagens dizem o mesmo

Confiri as quatro e varri o documento inteiro por frase residual. **Nenhuma delas afirma que são
duas leis**, e todas apontam ao mesmo sítio:

| passagem | o que diz hoje |
|---|---|
| **título §14** | *«COMPOR EM SÉRIE **FALHA**; a pergunta ‹uma lei ou duas› fica **EM ABERTO**»* |
| **§14.3** | separa *o que a medição sustenta* de *o que ela NÃO sustenta*, e nomeia as **duas** afirmações caídas, incluindo a da própria emenda anterior |
| **§11.1** | *«o que o §14 mede é COMPOSIÇÃO, não o número de leis … ⇒ a pergunta fica EM ABERTO»* |
| **§12** (1.ª célula) | *«a pergunta está EM ABERTO, e agora com um resultado por baixo»*, com a indicação e a fraqueza |
| **§14.5.2** | *«Favorece, não demonstra»* |

⭐ **A frase que fica, e que eu re-derivei inteira:**

> *«Compor as duas metades EM SÉRIE — relaxar com o passe desarmado, depois refinar com o pente a
> zero — FALHA o `G-2` em TODAS as granularidades que este instrumento alcança. O melhor caso, a
> `9×`, lê `Q = +0,0028` contra a barra de `+0,0465`; o alvo a fazer as duas coisas junto lê
> `+0,1214` e passa. A tendência com a granularidade é monótona crescente e, no ponto mais fino, ela
> já passou o zero.»*

✅ **Ela é sustentada pelo corpus, número a número** — medi os três valores e a monotonia. E **não**
troca uma inferência por outra mais bem vestida: ela não afirma nada sobre o interior do alvo nem
sobre o número de leis; afirma um facto sobre **composição**, que é o que o instrumento mede.
⭐ **E o resíduo que eu tinha apontado na ORDEM de leitura está curado sem eu o pedir:** a §14.5.3 diz
agora *«Faça (b) PRIMEIRO»*, o que impede o Implementador de sair à caça de uma segunda lei antes de
a hipótese barata estar no sítio.

**O valor ABSOLUTO promovido a coluna que decide — ✅ e a razão é correcta.** Medi as linhas de base:
`−0,0456` (junto, um traço) contra `−0,0110` · `−0,0137` · `−0,0106` · `−0,0057` (série, 2 a 18
traços) — as bases **não** são a mesma, porque o lado desligado assenta com as passagens (é o §5.2
desta espec). ⇒ comparar `ΔQ` entre configurações compara deltas sobre pontos de operação
diferentes; comparar o **absoluto contra o `G-2`** não tem esse problema, e é o que o gate mede.

**A extrapolação `1/n` — ✅ e é honestidade na medida CERTA, não a mais.** Re-derivei os dois ajustes:
`ΔQ` em `1/n` dá **`a = +0,0393`, `b = −0,2475`** e `Q(p1)` em `1/n` dá **`+0,0304`** — os dois ao
dígito do que a espec publica. ⚠️ **E os dois estão do MESMO lado da barra** (`+0,0465`): a escolha do
ajuste muda o número e **não** muda o sentido da indicação. ⇒ a indicação indica; declarar a fraqueza
sem esconder que ela não inverte o sinal é exactamente o registo certo. *Uma quarta fraqueza
declarada que não derruba a conclusão é mais forte do que três escondidas.*

### §4.6.4 — Os outros achados, e a caça aos IRMÃOS do resíduo

| achado | veredito | o que eu medi |
|---|---|---|
| **B-1 · B-2** | ✅ curados | ver §4.6.3 |
| **B-4** | ✅ curado | `n = 32`/`32` aplicando as nove chaves à letra — **bate com a minha medição**; e a robustez está publicada (com `32`, `34` e `35` as extremas são as mesmas e as reprovações são zero) |
| **B-5** | ✅ curado | `0,0990` é a aresta média da **entrada** ✓; saída `0,1002` ✓; pegada `0,1060` ✓ — as três que eu medi, com a população nomeada |
| **B-6** | ✅ curado | `+0,109557 → +0,1096` e `+0,167718 → +0,1677` ✓, com a regra *arredondar, nunca truncar* escrita |
| **A-8, o irmão que ELE achou** | ✅ curado | o `1 917` do README deu lugar a `1 260` ✓ — re-derivei como diferença simétrica de arestas |

⭐ **Procurei os IRMÃOS, como pedido, e a árvore está coerente.** Cruzei os números partilhados entre
a espec, o `README` das fixturas e o INBOX:

- **a tabela das famílias do README bate com a pasta nas NOVE células** (`14 · 6 · 29 · 10 · 62 · 26
  · 30 · 18 · 26`), contadas por mim — e a nota diz que são contadas, com o comando;
- a tabela `S × Q` das quatro rotações do README bate com a minha medição nas oito células;
- `1 917` não sobrevive em documento nenhum; `0,0995` aparece **uma** vez e é a menção de que foi
  retirado; `10`/`266`/`269`/`118`/`124` aparecem todos, com a métrica de cada um;
- o INBOX continua vazio de factos (só o formato), logo não há terceira cópia a divergir.

### §4.6.5 — As verificações de integridade

| o que | resultado |
|---|---|
| **fixturas mexidas nesta emenda** | **ZERO** células (`git diff` sobre `fixtures/` devolve só o `README.md`) |
| **corpos ainda byte-idênticos ao commit ORIGINAL** `45dd47832` | **195 de 195** ⇒ as minhas medições das três passagens medem todas o mesmo corpus |
| **linhas removidas do ledger** | **ZERO** — a §4.2.5 e o bloco `SWEEP-ESPERADO:` estão **intactos**, e a §4.5 do E **casa** com eles em vez de os substituir |
| **esperado × observado** | `-pull` sobre este ficheiro: `exit = 1`, linhas **`421 · 429 · 437`** — **exactamente** o que o bloco declara |
| **âmbito do Implementador** | `exit = 0` nas **dez** vassouras |
| **varredura por FORMA das linhas NOVAS** | cada promessa (*medido · prova · favorece · robusta*) traz o número ao lado; re-derivei as **cinco** que carregam peso |

### §4.6.6 — O que fica ABERTO (e é do E ou do dono, não bloqueia a implementação)

Nada disto impede a janela I de trabalhar; está no §12 da espec com dono escrito:

1. **A pergunta «uma lei ou duas»** — instrumento na §14.4, e só decidível **depois** de (b) estar
   implementada e medida contra o `G-2`. *É a primeira coisa que o I devolve.*
2. A régua para **malha curva** · o **custo** · a **máscara** · a **corda contra a direcção local**
   num arco · a **não-monotonia** entre 1 e 2 passagens — todos **E**.
3. Os cinco excluídos e a faixa do botão parar em `0,5` — **o dono**.
4. ⏳ **E a dívida que não é desta obra continua com dono:** `docs/3D/20_divergencias_tools.md`
   (§4.2.5 T-3) e a entrada da `VASSOURA_blender-pull.txt` (T-4). Nenhuma bloqueia esta espec.

### §4.6.7 — Corrente

| papel | id / data | o que fez |
|---|---|---|
| **R-pré, 3.ª passagem** | subagente-R independente, 2026-09-17 | ✅ **APROVADA** — três passagens, `21` achados no total, todos curados ou adjudicados. ⛔ **Dois dos meus caíram nesta passagem** (`B-8` por eu ler uma corrida de duas; `B-3` por a minha leitura ter a premissa escondida que a secção auditada ensina). A janela de implementação **abre** |

⚠️ **Higiene:** nada do alvo tocou o disco em nenhuma das três passagens. As vassouras correram pelo
`cleanroom-sweep.sh` (descodificação em memória), a saída foi consumida **em cano**, e destes três
registos saem apenas contagens, números de linha e grandezas — nunca os termos que casaram.
