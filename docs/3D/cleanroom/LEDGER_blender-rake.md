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

**Instrumento provado com CONTROLO POSITIVO antes de qualquer veredito:** um canário plantado
(`topology_rake_factor`) sai `exit = 1`; uma frase de prosa do alvo sai `exit = 1`.

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

### 📌 ACHADO PRÉ-EXISTENTE, que NÃO é desta obra — triagem é do R

`git grep` mede **uma** citação de nome interno do alvo sobre este assunto na árvore:
`docs/3D/20_divergencias_tools.md`, linha `422`, introduzida pelo commit `99e129b60` (o estudo
comparativo da época em que se lia o fonte). **Zero adições desta obra** (o `git status` desta
worktree tem só os quatro ficheiros novos). ⚠️ É a mesma forma do achado `tip_roundness` de 14/09:
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
| **R-pré** | — | **por despachar** — ⛔ sem o atestado dele a janela I não implementa |
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
