# TRIAGEM — LICENÇA E ARQUITETURA DE DISTRIBUIÇÃO (a pergunta do «modelo Rive»)

```
Data: 2026-08-30 · Papel: E (Especificador, §3.E da SKILL_Cleanroom) · subagente da
  janela 7499b0f4-218e-489b-879b-1e5a1c8b851f (line/quadextract).
Objecto: ⛔ este documento NÃO descreve algoritmo nenhum. Ele responde SEIS perguntas de
  LICENÇA e de ARQUITETURA DE DISTRIBUIÇÃO, levantadas pelo dono do produto em 30/08:
  «Poderíamos fazer como Rive, que deixou os algoritmos OpenSource mas o editor fechado,
  de modo que possamos usar os códigos opensource sem restrição?»
O que foi LIDO para o escrever (⛔ nenhum fonte foi lido para entender método — §2 da
  missão; só ficheiros de licença, cabeçalhos de licença, READMEs, CMakeLists, logs de
  build, manifestos e o nosso próprio Cargo.toml):
  · ficheiros de licença de 16 componentes de terceiros, no clone local do oráculo
    (`ph2d-quadbench/oracle/libs/*`) e no clone da biblioteca permissiva (`~/Referencias/`)
  · READMEs e CMakeLists dos mesmos, para a pergunta «quem depende de quem»
  · o `cmake.log` da NOSSA própria compilação do oráculo (2026-08-20)
  · texto integral da GPLv3 (gnu.org) e o §3.3 da MPL-2.0 (mozilla.org)
  · GPL FAQ da FSF (anchors MereAggregation · GPLPlugins · GPLAndPlugins · NFUseGPLPlugins ·
    GPLInProprietarySystem · MoneyGuzzlerInc · LinkingOverControlledInterface)
  · `Cargo.toml` de 12 crates nossas + `shells/desktop/Cargo.toml` + `deny.toml`
  · ADR-0162, ADR-0167, TRIAGEM_quad_remesh.md, LEDGER_quadwild.md
  · páginas públicas: licença do runtime da Rive · página oficial de uma biblioteca de
    extracção universitária · noticiário do caso Artifex v. Hancom · páginas de
    conformidade GPL de três produtos fechados.
Denylist de URLs (⛔ NÃO abrir): qualquer hospedagem de código, issue tracker, PR ou
  code-search dos alvos nomeados no `SPEC_restricoes_por_eliminacao.md`. ⚠️ Nada aqui
  precisou delas: licença lê-se do ficheiro, e o ficheiro está no disco.
Denylist de CAMINHOS (para o papel I): `~/Referencias/**` · `ph2d-quadbench/oracle/**`.
Sweep §7.1: verde (94 entradas) em 2026-08-30, sobre este documento e sobre o report.
⚠️⚠️ NÃO SOMOS ADVOGADOS. Cada afirmação abaixo traz a FONTE ao lado e está marcada
  como **[FACTO LIDO]** (texto de licença, ficheiro, log) ou **[INTERPRETAÇÃO CORRENTE]**
  (leitura consensual da FSF / prática de indústria / comentário jurídico público).
  O §7.3 lista o que exige parecer humano de verdade.
```

---

## §0 — A resposta em cinco linhas

1. ⛔ **A premissa da pergunta é falsa, e tem de ser dita:** abrir código **NOSSO** não
   concede direito nenhum sobre código **COPYLEFT ALHEIO**. A Rive pode abrir o runtime dela
   porque a Rive **escreveu** o runtime dela.
2. ⭐⭐ **Mas a disposição do dono muda uma decisão real** — só que não a que ele espera: ela
   dissolve a razão nº 1 da recusa do ADR-0167 (*«obrigação de publicar arquivos»*) e reabre
   o porte fiel **T0½** da biblioteca **MPL-2.0**. ⚠️ As outras duas razões daquela recusa são
   **medições**, e continuam de pé.
3. ⭐⭐⭐ **E há uma porta mais barata que ninguém abriu:** a via **§2.T1(d)** — escrever aos
   autores. Uma das bibliotecas da fase que esta linha persegue **publica, na página oficial
   da universidade, um convite explícito a negociar licença comercial**. Custo: um e-mail.
4. ⭐ **O split «algoritmos abertos / editor fechado» é, aqui, um acto de EMPACOTAMENTO, não
   uma refactoração** — as 10 crates da cadeia já não dependem do editor de coisa nenhuma
   (medido no `Cargo.toml`, §5.1).
5. ⚠️ **E a peça que o registo dava por indistribuível NÃO está no caminho** — o §2 confirma
   que aquela implementação de referência não é software livre, **e refuta** que o remalhador
   de produção dependa dela: ela está **desligada por omissão**, e a nossa própria compilação
   do oráculo diz, no log, que foi construída **sem** ela.

---

## §1 — INVENTÁRIO DE LICENÇAS (lidas do FICHEIRO, nunca do README)

Lido em **2026-08-30**, ficheiro a ficheiro. A coluna «lida de» é a prova.

### §1.1 — O remalhador de produção (o nosso oráculo de bancada) e a sua árvore

| componente (nome público) | lida de | licença **[FACTO LIDO]** | degrau §2 |
|---|---|---|---|
| **quadwild-bimdf** (o umbrella, o binário que corremos) | `LICENSE` no topo + `README.md` §License | **GPL-3.0** | **T2** |
| **quadwild** (o programa original, dentro do umbrella) | `README_orig.md` §License | **GPL-3.0** | **T2** |
| **vcglib** (malha, I/O) | `LICENSE.txt` | **GPL-3.0** | **T2** |
| **xfield_tracer** (traçado) | `LICENSE` | **GPL-3.0** | **T2** |
| **CoMISo** (solver misto-inteiro) | `COPYING` + banner por-ficheiro | **GPL-3.0-or-later** (o banner diz *«either version 3 … or (at your option) any later version»*) | **T2** |
| **quadretopology** (preenchimento por padrões) | ⚠️ **nenhum** `LICENSE`, **nenhum** banner nos fontes | ⇒ herda a GPL-3.0 do umbrella; **ambiguidade a nomear** se alguém a tomar isolada | **T2** |
| **libigl** | `LICENSE.MPL2` + `LICENSE.GPL` | **MPL-2.0**, com o sub-directório `copyleft/` sob **GPL** | T0½ / **T2** |
| **libsatsuma** (quantização) | `LICENSE` | ⭐ **MIT** | **T0** |
| **lemon** (fluxo e emparelhamento) | `LICENSE` | ⭐ **Boost Software License 1.0** | **T0** |
| **OpenMesh** (half-edge) | `LICENSE` | **BSD-3-Clause** | **T0** |
| **libTimekeeper** (instrumentação) | `LICENSE` | **MIT** | **T0** |
| **nlohmann/json** | `LICENSE.MIT` | **MIT** | **T0** |
| **eigen** (álgebra) | `COPYING.MPL2` | **MPL-2.0** | T0½ |
| **glew** (só visualização) | `LICENSE.txt` | **BSD-3-Clause** (estilo) | **T0** |
| **lp_solve** (solver linear) | `lp_solve_5.5/README.txt` (⚠️ não há ficheiro de licença próprio no submódulo) | **LGPL** (declarado no README) | T0½ |
| **blossom5-cmake** (o *wrapper* do emparelhamento) | `LICENSE` do repositório | **Unlicense** — ⚠️ **mas isso cobre só o wrapper**; ver §2 | — |
| ⛔ **a implementação de referência do emparelhamento** | ⚠️ **não está no disco** — o wrapper traz **só um patch**, e o CMake **descarrega** o fonte do sítio do autor no momento da compilação | **não-livre** (ver §2) | **T4** |

⚠️ **A leitura que a tabela obriga:** o copyleft desta família entra por **quatro portas
independentes** — o `LICENSE` do próprio umbrella, a biblioteca de malha, o traçador e o
solver misto-inteiro — mais um submódulo sem licença que herda do umbrella. **Não** por uma.
(Isto responde à segunda metade da pergunta 6; ver §6.3.)

### §1.2 — A biblioteca permissiva (o «segundo oráculo» do ADR-0167)

| propriedade | valor **[FACTO LIDO]** |
|---|---|
| licença declarada | **MPL-2.0** — declarada na documentação oficial do projecto (*«primarily MPL2 licensed. Some files contain third-party code under other licenses.»*) |
| banner por-ficheiro | **123 dos 145 cabeçalhos** trazem o banner MPL-2.0 (contagem feita hoje) |
| ⚠️ lacuna a nomear | ⛔ **não existe ficheiro `LICENSE` na raiz do repositório.** A declaração vive na documentação e nos banners. Para um porte real, isto é exactamente o tipo de ponta solta que se resolve **perguntando ao autor**, não presumindo |
| degrau | **T0½** (copyleft por-ARQUIVO) |

### §1.3 — O que mais vive em `~/Referencias/`

| item | natureza | regra |
|---|---|---|
| três *papers* em PDF/texto | literatura pública (ACM / autores) | ⭐ os **factos e o método** são livres (§1.2 da skill: TRIPS 9(2), 17 USC §102(b), Lei 9.610 art. 8º); ⛔ o **wording** não é (a claim que a SAS ganhou) |
| `directional-bench/` | **NOSSO** arnês de medição | nosso |
| `draft/`, `papers/`, notas | zona contaminada | fica fora do repo, sempre |
| `ph2d-quadbench/corpus/` | **NOSSAS** malhas | nosso, lícito |

### §1.4 — ⚠️ O nosso próprio lado

**[FACTO LIDO]** `deny.toml`: a lista de licenças aceites é
`MIT · Apache-2.0 (+LLVM-exception) · BSD-2 · BSD-3 · ISC · Zlib · Unlicense · MPL-2.0 ·
Unicode-3.0 · Unicode-DFS-2016 · CC0-1.0 · LicenseRef-Proprietary`.
⛔ **Nenhuma variante de GPL/LGPL está lá.** ⚠️ E a **Boost Software License 1.0 também não**
— ela entra hoje só por **excepção nomeada** para uma crate (`error-code`). ⇒ adoptar uma
dependência Boost (§2.4) custa **uma linha no `deny.toml`**, e é uma decisão consciente, não
um acidente.

**[FACTO LIDO]** `Cargo.toml` da workspace: `license = "LicenseRef-Proprietary"`,
`publish = false`, autor único.

---

## §2 — ⭐⭐ REDISTRIBUIBILIDADE: a afirmação do ADR-0167, conferida

> O ADR-0167 (linha de alternativas rejeitadas) diz: *«Portar a implementação de referência
> do emparelhamento — ⛔ não é software livre: avaliação e pesquisa apenas, redistribuição
> proibida, licença comercial à parte. É T4.»*

### §2.1 — CONFIRMADO na metade que descreve aquela biblioteca

**[FACTO LIDO]** — do `README.md` do repositório de empacotamento (o *wrapper*), verbatim:

> *«That code is **not** under a free license, but available for evaluation and research
> purposes. […] Notably, **redistribution of the code is not permitted**, thus this
> repository only includes a patch file with our changes.»*
> e, mais abaixo: *«**Note: Be sure to obey the terms of the Blossom-V license!**»*

**[FACTO LIDO]** — corroboração estrutural, e é a mais forte que existe: o repositório
**não contém o fonte**. O `CMakeLists.txt` dele faz `FetchContent_Declare` de um `.tar.gz`
no sítio do autor e aplica o patch localmente. *Um projecto que não se atreve a vendorizar
o fonte está a dizer, por acto, que não pode redistribuí-lo.*

**[FACTO LIDO]** — a **Unlicense** do repositório cobre **só o wrapper**: *«The contents of
this repository (**excluding Blossom-V itself**), e.g., our contributions, are available
under the Unlicense.»*

⇒ **A classificação T4 está CORRECTA.** Aquela peça é indistribuível por nós em **qualquer**
arquitectura — aberta, fechada, processo separado ou não. Nem sequer podemos *incluí-la no
instalador*.

### §2.2 — ⭐⭐⭐ REFUTADO na metade que decidiria a pergunta do dono

**O remalhador de produção NÃO depende dela.** Três medições independentes:

| prova | **[FACTO LIDO]** |
|---|---|
| o `CMakeLists.txt` do **umbrella** | `option(SATSUMA_ENABLE_BLOSSOM5 "Enable Blossom-V (non-free license)" **OFF**)` — e o `add_subdirectory` dela está dentro de um `if()` |
| o `CMakeLists.txt` da **biblioteca de quantização** | a mesma opção, o mesmo `OFF`; sem ela o cabeçalho de configuração é gerado com `HAVE_BLOSSOM5 = 0` e o `target_link_libraries` **não corre** |
| o **README de instalação do próprio umbrella** | a linha de compilação publicada por eles é `cmake . -B build -D SATSUMA_ENABLE_BLOSSOM5=0` — ⭐ *os autores mandam desligá-la* |
| o **nosso** `cmake.log`, de 2026-08-20 | linha 108: `libSatsuma: building WITHOUT blossom-v` |
| o `Dockerfile` do umbrella | `ARG WITH_BLOSSOM5=0` |

⇒ **O binário que usamos como oráculo desde 20/08 nunca a conteve.** A frase do registo que
a chama *«o solver exato que a nossa quantização nomeia como a cura»* precisa de uma emenda:
ela é **uma** implementação daquela cura, e é a **única** indisponível.

### §2.3 — ⇒ Consequência para a pergunta do dono

⛔ **Este item, sozinho, NÃO decide nada.** A afirmação que o decidiria seria *«o remalhador
de produção é indistribuível em qualquer arquitectura»* — e ela é **falsa por esta via**.
O remalhador de produção é **GPL-3.0 puro**: distribuível, sim, mas **sob a GPL**, com todas
as obrigações do §4 deste documento.

⚠️ **A restrição que fica é OUTRA, e é de negócio, não de lei:** shipar o remalhador de
produção significa entregar, ao lado de um editor pago e fechado, um programa que o comprador
pode copiar, modificar e redistribuir de graça — e que faz *exactamente* o que o botão
`Quad Retopology` promete. Isso é decisão do dono, e este documento não a toma.

### §2.4 — ⭐ Existe substituto LIVRE para a peça de emparelhamento? **SIM, e já está na árvore**

**[FACTO LIDO]** A biblioteca de grafos que o umbrella já usa está sob **Boost Software
License 1.0** (permissiva) e traz, ela própria, o algoritmo de emparelhamento por contracção
de flores para grafos gerais: `MaxMatching`, `MaxWeightedMatching` e
`MaxWeightedPerfectMatching` (lidos dos nomes de classe e da documentação `groups.dox` dela).
É **por isso** que a opção não-livre é *opcional*: ela é uma **alternativa**, não o motor.

**[INTERPRETAÇÃO CORRENTE]** O algoritmo em si (contracção de flores, Edmonds, 1965) é
matemática publicada há 60 anos — não é protegível (§1.2 da skill), e há implementações
permissivas independentes. ⇒ **Nenhuma fase desta cadeia está bloqueada por falta de um
emparelhamento livre.**

---

## §3 — O MODELO RIVE, verificado — e a premissa que tem de ser escrita

### §3.1 — O que a Rive de facto licenciou **[FACTO LIDO]**

| peça | licença |
|---|---|
| **runtime** (o que carrega e reproduz o ficheiro: C++, Swift, Flutter, Android, JS/WebGL, React, Rust) | ⭐ **MIT** — lido do `LICENSE` do repositório público do runtime |
| **formato de ficheiro** | aberto, documentado, implementado pelos runtimes MIT |
| **editor** | ⛔ **produto proprietário e hospedado**; não há repositório público, não há licença open source |

⇒ O modelo existe, é real, e é exactamente o que o dono descreveu: **runtime aberto e
permissivo, editor fechado.** ⭐ E é, aliás, **o modelo que este repositório já segue** para
o módulo Vector (ADR-0108: *referenciado no runtime MIT da Rive*).

### §3.2 — ⛔⛔ A resposta que o dono precisa de ver escrita

> **Abrir código PRÓPRIO não concede direito NENHUM sobre código COPYLEFT ALHEIO.**

**[FACTO LIDO — GPLv3 §5(c), verbatim]**, e é o texto que fecha a porta:

> *«You must license the entire work, as a whole, under this License to anyone who comes
> into possession of a copy. This License will therefore apply, along with any applicable
> section 7 additional terms, **to the whole of the work, and all its parts, regardless of
> how they are packaged**. This License **gives no permission to license the work in any
> other way**, but it does not invalidate such permission if you have separately received
> it.»*

O mecanismo, em três frases:

1. A permissão para usar um código vem de **quem detém o direito de autor DAQUELE código**.
2. A Rive pode abrir o runtime dela porque **a Rive o escreveu**. Nós podemos abrir o nosso
   porque **nós o escrevemos**. Nenhum desses actos cria licença sobre o código de terceiro.
3. A GPL é uma condição sobre **a obra que CONTÉM código GPL**. A única pessoa que a pode
   dispensar é **o autor do código GPL** — e a via para isso tem nome: **§2.T1(d), pedir**
   (§6 deste documento).

⚠️ **[INTERPRETAÇÃO CORRENTE]** O modelo Rive é um modelo de **negócio** (runtime aberto ⇒
adopção; editor fechado ⇒ receita), **não** um mecanismo de lavagem de licença. Ele responde
*«posso abrir os meus algoritmos e manter o editor fechado?»* — **sim**. Ele **não** responde
*«posso, por isso, usar os algoritmos GPL de outra pessoa?»* — **não**.

---

## §4 — ⭐⭐⭐ A ARQUITECTURA DE PROCESSO SEPARADO

*Um editor proprietário que invoca um binário GPL como processo separado (pipe / ficheiro /
linha de comando), sem ligar código.*

### §4.1 — A posição da FSF **[FACTO LIDO — GPL FAQ oficial]**

| anchor | o que diz, verbatim |
|---|---|
| **MereAggregation** | *«Pipes, sockets and command-line arguments are communication mechanisms **normally used between two separate programs**. So when they are used for communication, the modules normally are separate programs.»* — e, do outro lado: *«if the semantics of the communication are **intimate enough, exchanging complex internal data structures**, that too could be a basis to consider the two parts as combined into a larger program.»* |
| **GPLPlugins** | *«If the program uses fork and exec to invoke plug-ins, then the plug-ins are **separate programs**, so the license for the main program makes no requirements for them.»* — contra: *«If the main program dynamically links plug-ins, and they make function calls to each other and share data structures, we believe they form a **single combined program**.»* e *«Using shared memory to communicate with complex data structures is pretty much equivalent to dynamic linking.»* |
| **NFUseGPLPlugins** | um programa não-livre pode carregar um plug-in GPL **se forem obras separadas**; se formarem um só programa, *«the main program must be released under the GPL or a GPL-compatible free software license»* |
| **GPLInProprietarySystem** | ⭐ a resposta directa: *«in many cases you can **distribute the GPL-covered software alongside** your proprietary system»*, desde que *«they communicate **at arms length**, that they are not combined in a way that would make them effectively a single program.»* |

**[FACTO LIDO — GPLv3 §5, último parágrafo, verbatim]** — a definição de agregado, que é a
fronteira **de texto de licença**, não de FAQ:

> *«A compilation of a covered work with **other separate and independent works, which are
> not by their nature extensions of the covered work, and which are not combined with it
> such as to form a larger program**, in or on a volume of a storage or distribution medium,
> is called an "aggregate" if the compilation and its resulting copyright are not used to
> limit the access or legal rights of the compilation's users beyond what the individual
> works permit. **Inclusion of a covered work in an aggregate does not cause this License to
> apply to the other parts of the aggregate.**»*

**[FACTO LIDO — GPLv3 §0]** *«To "convey" a work means any kind of propagation that enables
other parties to make or receive copies.»* ⇒ ⭐ **enquanto o binário GPL não sai da nossa
máquina, não há obrigação nenhuma** — é o §1.1 da skill, e é o regime em que o oráculo vive
hoje (ADR-0162). O §4.4 abaixo só se aplica no dia em que ele entrar no instalador.

### §4.2 — A prática comercial estabelecida **[FACTO LIDO / prática documentada]**

| produto fechado | o que distribui | como | o que publica |
|---|---|---|---|
| **macOS (Apple)** | sistema proprietário com utilitários **GPLv2** de linha de comando (interpretador de comandos, compressores, arquivadores) | binários separados, invocados por `exec` | fonte completo em `opensource.apple.com` |
| **Plex Media Server** | servidor proprietário + um *fork* do transcodificador de vídeo livre, executado como **processo separado** («Plex Transcoder») | processo separado, ficheiros e argumentos de linha de comando | o fonte modificado do transcodificador, em `downloads.plex.tv` |
| **Synology DSM / QNAP QTS** | interface web proprietária sobre um espaço de utilizador **GPL** completo | processos separados no mesmo sistema | pacotes de fonte GPL publicados (repositório `dsgpl` / `archive.synology.com`) |

⚠️ **A lição que o exemplo da Apple ensina, e é a mais útil dos três:** eles congelaram o
interpretador de comandos na **última versão GPLv2** e mudaram o padrão do sistema para outro
programa, em vez de shipar **GPLv3**. **[INTERPRETAÇÃO CORRENTE]** o que recusaram não foi a
*agregação* — foi o resto da GPLv3 (§11 patentes, §6 «User Product»). ⇒ *a arquitectura é
aceite pela indústria; o que se negoceia é o resto do texto.*

⛔ **Um exemplo que NÃO serve, e é preciso dizer porquê:** extensões comerciais de um
modelador 3D livre que empacotam remalhadores GPL de terceiros como executáveis. Elas
existem e fazem exactamente isto — mas a extensão é ela própria argumentada como obra
derivada GPL do hospedeiro, então **não é exemplo de produto fechado**.

### §4.3 — ⛔ Onde a fronteira se torna INSEGURA

Por ordem de perigo, e cada uma com o mecanismo:

1. ⛔⛔ **Ligar** (estática ou dinamicamente) no espaço de endereços do editor. É o padrão de
   facto do litígio **Artifex Software v. Hancom** (N.D. Cal., 2017): um produto de escritório
   proprietário que incorporou um interpretador GPL em vez de comprar a licença comercial.
   **[FACTO — noticiado]** o tribunal negou a moção do réu e sustentou que a GPL é exigível
   **como contrato**, e não só por direito de autor; **[INTERPRETAÇÃO]** o caso **acabou em
   acordo confidencial** em Dezembro de 2017, portanto **não há decisão de mérito** sobre a
   combinação. O que ele prova é que a exigência é real e cara — não onde exactamente está a
   linha.
2. ⛔ **Comunicação que não é «à distância de um braço»** — passar estruturas internas,
   memória partilhada, um protocolo privado desenhado para nós. O texto do §5 e a FAQ batem
   no mesmo ponto por dois caminhos.
3. ⛔ **O auxiliar ser «by their nature an extension»** da nossa obra: escrito por nós para o
   nosso editor, inútil sozinho, sem existência independente, entregue só dentro do nosso
   instalador. ⭐ **A recíproca é a nossa defesa:** quanto mais o auxiliar for um programa de
   linha de comando **de terceiros, de uso geral, que existe e é usado fora do nosso
   produto**, mais forte é o agregado. *O remalhador de produção é exactamente esse caso.*
4. ⚠️ **AGPL** — o §13 dispara com **interacção por rede**, sem *convey* nenhum. ⛔ Nenhum
   componente desta cadeia é AGPL (§1.1), mas a regra fica escrita para o próximo alvo.
5. ⚠️ **GPLv3 §6, «User Product» / Installation Information** — morde se um dia shipparmos
   em hardware que trancamos. Instalador de desktop: **não** é esse caso.
6. ⚠️ **GPLv3 §11 (patentes) e §3 (anti-contorno)** — obrigações que viajam com a agregação e
   que nada têm a ver com ligar ou não ligar. É o que a Apple recusou.

### §4.4 — As obrigações CONCRETAS que ficam, mesmo na agregação mais segura

**[FACTO LIDO — GPLv3 §4, §5, §6]**

1. **Notices (§4/§5):** manter intactos **todos** os avisos de direito de autor e de licença;
   entregar a **cada** destinatário uma cópia da GPL.
2. **Fonte correspondente (§6):** entregar a *Corresponding Source* do binário GPL, por uma
   destas vias:
   - **6a** — fonte no mesmo meio físico;
   - **6b** — **oferta escrita, válida ≥ 3 anos**, de entregar o fonte a quem possuir o
     binário;
   - **6d** — ⭐ **a via normal para produto descarregável:** oferecer o fonte *«from a
     designated place (gratis or for a charge)»*, com **acesso equivalente, no mesmo sítio,
     sem custo adicional**, e *«clear directions next to the object code saying where to find
     the Corresponding Source»*. ⚠️ E: *«you remain obligated to ensure that it is available
     for as long as needed»*.
3. ⚠️ **A fonte tem de ser a DO BINÁRIO QUE SHIPAMOS** — incluindo **os nossos patches** e os
   *scripts* de compilação. Se o compilarmos com opções nossas, publicamos **essa** árvore.
4. ⛔ **Não podemos acrescentar restrições ao binário GPL** — nenhuma cláusula do nosso EULA
   pode proibir extraí-lo, estudá-lo ou redistribuí-lo. Uma cláusula genérica *«é proibido
   descompilar qualquer parte deste software»* **entra em conflito** com isto.
5. **Prática (não texto):** um ecrã/ficheiro *«Open Source Licenses»* no instalador. É a forma
   habitual de cumprir o nº 1 — o texto exige os avisos e o fonte, não uma UI.
6. ⚠️ **Brasil:** ⛔ **não há norma brasileira análoga ao art. 8 da Directiva UE 2009/24**
   (que anula cláusula contratual contrária à observação). ⇒ qualquer sobreposição contratual
   é vector de risco **próprio**, separado do direito de autor (§2.T4 da skill).

---

## §5 — ⭐⭐ O QUE UM SPLIT «ALGORITMOS ABERTOS / EDITOR FECHADO» EXIGIRIA AQUI

### §5.1 — (a) O que seria publicável, MEDIDO no `Cargo.toml` (não desejado)

**[FACTO LIDO]** As dez crates da cadeia de retopologia e as suas dependências **inteiras**:

| crate | depende de (interno) | de terceiros |
|---|---|---|
| `ph2d-mesh` | — (folha) | `rayon`, `serde`, `dhat` (dev) |
| `ph2d-remesh-iso` | `ph2d-mesh` | — |
| `ph2d-crossfield` | `ph2d-mesh`, `ph2d-remesh-iso` | — |
| `ph2d-quantize` | — (folha pura) | — |
| `ph2d-trace` | `ph2d-mesh`, `ph2d-crossfield`, `ph2d-quantize` | — |
| `ph2d-gridmap` | `ph2d-mesh`, `ph2d-crossfield`, `ph2d-trace` | — |
| `ph2d-quadextract` | `ph2d-mesh` | — |
| `ph2d-quadfill` | `ph2d-mesh`, `ph2d-quantize`, `ph2d-trace`, `ph2d-remesh-iso` | `rayon` |
| `ph2d-quadflow` | `ph2d-mesh` | — |
| `ph2d-quadchain` | as sete acima | `rayon` (só num exemplo) |

⭐⭐⭐ **A conclusão arquitectural, e é o facto mais forte deste documento:** ⛔ **zero**
dependência do editor, do ECS, do `wgpu`, dos tokens, da i18n, do `shells/desktop`.
⇒ **Publicar esta cadeia é um acto de EMPACOTAMENTO, não uma refactoração.** O corte já
existe, e foi o ADR-0075 (desacoplar por drop-crate) que o pagou.

**Inseparáveis do editor** — os **consumidores**, e são só dois:
- `shells/desktop` (o botão `Quad Retopology`, as cenas de smoke, os painéis);
- `ph2d-field-eval` (o módulo 3D Modeling, que chama a cadeia). **[FACTO LIDO]** ele consome
  as crates **pela API pública**, como qualquer cliente externo — nada nele exige estar dentro
  da mesma árvore de licença.

⚠️ **Nomeado, não recomendado:** `ph2d-sdf` e `ph2d-sculpt3d` também são folhas sobre
`ph2d-mesh` e caberiam num pacote aberto. ⛔ `ph2d-mesh-render` **não** (é `wgpu` e é o passe
de desenho). *Escolher o recorte é decisão de produto; este documento só diz onde ele é
possível sem dor.*

### §5.2 — (b) Publicar as crates sob GPL deixaria o editor FECHADO ligá-las? — **A REGRA**

⚠️ Respondo com a regra, não com o desejo. São **dois** casos e eles não se parecem:

| caso | resposta | mecanismo |
|---|---|---|
| **As crates são 100 % nossas** (autoria única, sem contribuições externas) | ⭐ **SIM** — mas **não por causa da GPL** | A GPL vincula **quem recebe**, nunca **quem detém** o direito de autor. O detentor pode licenciar o mesmo código a si próprio sob outros termos. É o modelo de dupla licença clássico (o padrão Qt/MySQL). ⚠️ Ele **deixa de funcionar** no dia em que houver contribuição de terceiro sem cessão de direitos (CLA) — ou uma linha de código GPL alheio |
| **As crates absorvem código GPL de terceiro** | ⛔⛔ **NÃO — para ninguém, o dono incluído** | O que falta é a permissão **do terceiro**. A GPLv3 §5(c) (citada no §3.2) diz que a licença se aplica *«to the whole of the work, and all its parts»* e *«gives no permission to license the work in any other way»* |

**[INTERPRETAÇÃO CORRENTE]** publicar sob GPL e simultaneamente ligar o mesmo código num
binário fechado próprio é lícito e comum, mas é **notado** pela comunidade, e obriga a exigir
CLA de qualquer contribuidor — ou a recusar contribuições. Não é grátis em atenção.

### §5.3 — (c) Que licença o dono teria de escolher, e as duas coisas são compatíveis?

| objectivo | licença que o serve |
|---|---|
| **(i)** o **editor fechado** poder ligar as crates abertas | **MIT / Apache-2.0 / BSD** (já na lista da casa) **ou** ⭐ **MPL-2.0** |
| **(ii)** podermos **absorver** código copyleft alheio dentro delas | **GPL-3.0-or-later** (ou exactamente a licença do que for absorvido) |

⭐⭐ **A MPL-2.0 é a resposta exacta para (i), e sem truque de dupla licença.**
**[FACTO LIDO — MPL-2.0 §3.3, verbatim]**:

> *«You may create and distribute a **Larger Work under terms of Your choice**, provided that
> You also comply with the requirements of this License for the Covered Software.»*

⇒ crate aberta MPL-2.0 + editor fechado a ligá-la = **expressamente permitido**, publicando
apenas **os ficheiros da crate**. E `MPL-2.0` **já está na lista de aceites do `deny.toml`**.

⛔⛔ **(i) E (ii) AO MESMO TEMPO, NA MESMA CRATE: IMPOSSÍVEL.** A razão é uma frase, e é texto
de licença, não opinião: absorver código GPL torna **a obra inteira** GPL (§5(c)), e a GPL
*«gives no permission to license the work in any other way»*. Nenhuma escolha de licença
nossa contorna isso, porque a permissão em falta não é nossa para dar.

⭐ **A ÚNICA forma de ter as duas é ter DOIS ARTEFACTOS:**
- a crate **permissiva/MPL** que o editor **liga**; e
- um **executável GPL separado**, que o editor **invoca** — nunca liga.

⇒ ⭐⭐ **É exactamente a arquitectura do §4, alcançada pelo outro lado.** *A pergunta do dono
e a arquitectura de processo separado são a mesma resposta vista de dois ângulos.*

---

## §6 — A VIA T1 QUE NINGUÉM PERCORREU

### §6.1 — A caçada foi feita? **Metade dela.**

**[FACTO LIDO — LEDGER e TRIAGEM]**

| alínea do §2.T1 | feita? | evidência |
|---|---|---|
| (a) código de referência dos autores do *paper* | ✅ | tabela de licenças de 2026-08-24 |
| (b) versões antigas sob licença mais branda | ⚠️ não registada | nada no ledger |
| (c) reimplementações permissivas independentes | ✅ ⭐ | **foi assim que a biblioteca MPL-2.0 foi encontrada** |
| **(d) e-mail aos autores a pedir dupla licença** | ⛔⛔ **NUNCA FOI FEITA** | `grep` por *contato / contact / e-mail / dual / comercial* no `LEDGER_quadwild.md` e no `TRIAGEM_quad_remesh.md`: **zero** ocorrências de qualquer tentativa de contacto. O ledger diz *«a caçada T1 começa daqui»* e a alínea (d) nunca aparece |

### §6.2 — ⭐⭐⭐ E o convite está PUBLICADO

**[FACTO LIDO — página oficial do grupo universitário que assina o *paper* da extracção]**,
verbatim:

> *«libQEx is free software: you can redistribute it and/or modify it under the terms of the
> GNU General Public License … either version 3 of the License, or (at your option) any later
> version.»*
> *«**Commercial licensing under negotiable terms is available upon request.**»*
> — com endereço de contacto institucional na mesma página.

⚠️ **Precisão obrigatória:** essa biblioteca é **outra implementação**, do **mesmo grupo** que
assina o solver misto-inteiro da árvore do oráculo — e cobre **a fase em que esta linha está**
(a extracção). ⛔ Ela **não** é o remalhador de produção inteiro.

⭐ **O que isto custa:** um e-mail. **O que pode devolver:** o oposto exacto de um clean-room —
um porte **licenciado, verbatim, atribuído e suportado**, a custo **T0**, sem parede, sem
vassoura, sem ledger de contaminação, sem semanas.
⛔ **O que NÃO pode devolver:** licença permissiva para o **pipeline** de produção — esse tem
mais donos (§6.3) e **nenhuma oferta publicada**.

### §6.3 — A GPL do remalhador de produção é herdada de UMA dependência?

⛔ **NÃO — e a resposta é medida** (§1.1). A GPL entra por **quatro portas independentes**:

1. o `LICENSE` do **próprio umbrella** (escolha dele, não herança);
2. a biblioteca de **malha** (GPL-3.0);
3. o **traçador** (GPL-3.0);
4. o **solver misto-inteiro** (GPL-3.0-or-later);
   \+ um submódulo **sem licença** que herda do umbrella.

⇒ **O caso que a skill descreve — *«às vezes é GPL só por causa de UMA dependência»* — NÃO se
aplica aqui.** Trocar uma dependência não muda a licença do todo.

⭐ **Mas o facto útil é o inverso:** o direito de autor está **concentrado em dois grupos
universitários** — um eixo italiano/suíço (umbrella + traçador + padrões) e o grupo alemão
(solver misto-inteiro + a biblioteca de extracção + a de half-edge, esta já BSD). **Um dos
dois já publica oferta de licenciamento comercial.** ⇒ a negociação é plausível **por
componente**, e o componente de que precisamos é o do grupo que já convida.

---

## §7 — VEREDITO

### §7.1 — O que a disposição do dono muda, e o que não muda

| afirmação | veredito |
|---|---|
| «abrindo os nossos algoritmos, podemos usar código GPL alheio sem restrição» | ⛔ **FALSO** (§3.2). Nada no acto de abrir código nosso toca a licença do de terceiro |
| «podemos abrir os algoritmos e manter o editor fechado» | ⭐ **VERDADEIRO**, e aqui é barato: o corte já existe no `Cargo.toml` (§5.1) |
| «podemos distribuir um binário GPL ao lado do editor fechado» | ⭐ **VERDADEIRO**, com a arquitectura do §4 e as obrigações do §4.4 — e é prática de indústria com três exemplos documentados |
| ⭐⭐ «a recusa do ADR-0167 ao porte T0½ pode ser reaberta» | ⭐ **SIM, na razão nº 1** — *«obrigação de publicar arquivos no subsistema mais valioso»* **dissolve-se** se publicar deixou de ser um custo. ⚠️ **MAS o ADR-0167 deu TRÊS razões**, e as outras duas são **medições**, não licença: *«herda falhas em 3 de 7 peças nossas»* e *«descarta a cadeia que já temos»*. ⇒ **a premissa moveu-se; o veredito NÃO segue automaticamente.** Reabrir aquela decisão exige a medição, não este documento |
| ⚠️ e uma ironia a registar | **a MPL-2.0 nunca exigiu publicar o EDITOR** (§3.3, §5.3). O que o dono agora oferece é **mais** do que aquele porte alguma vez precisou — a recusa de 24/08 media o custo de publicar **N ficheiros de crate**, não o produto |

### §7.2 — A ordem que este documento recomenda (custo crescente)

1. ⭐⭐⭐ **Escrever o e-mail** (§6.2). Custo: minutos. É a única acção com hipótese de tornar
   todo o resto desnecessário, e a skill manda fazê-la **antes da obra**.
2. ⭐ **Se o dono aceita publicar ficheiros:** reabrir o porte **T0½** da biblioteca MPL-2.0
   com a medição das razões 3 e 5 do ADR-0167 na mão — ⛔ não sem ela.
3. **Se o produto puder entregar um binário livre ao lado:** a agregação do §4, com as
   obrigações do §4.4 escritas no instalador **antes** do primeiro *ship*.
4. **Caso contrário:** o clean-room T2 que a linha já corre, **inalterado**.

### §7.3 — ⛔ O que EXIGE parecer jurídico humano (não adivinhámos)

1. **Se a nossa combinação concreta é «agregado»** — a fronteira do §4.3 é qualitativa e a
   FSF é **parte interessada**, não tribunal. Um produto pago que entrega um binário GPL
   dentro do próprio instalador merece leitura de advogado **antes do primeiro ship**.
2. **A redacção do EULA** contra o item 4 do §4.4 (não podemos restringir o binário GPL) e
   contra a ausência, no Brasil, de norma que anule cláusula anti-observação.
3. **A dupla licença sobre código próprio** (§5.2) — a política de CLA, e a exposição de
   publicar sob GPL algo que também vendemos fechado.
4. **A lacuna nomeada no §1.2** (biblioteca sem `LICENSE` na raiz) — antes de qualquer porte,
   confirmação por escrito do autor.

---

## §8 — FACTO LIDO vs INTERPRETAÇÃO (o índice honesto)

| afirmação | classe | fonte |
|---|---|---|
| as 16 licenças da tabela §1.1 | **FACTO LIDO** | ficheiro de licença / banner, no disco, 2026-08-30 |
| a implementação de referência do emparelhamento não é livre e não pode ser redistribuída | **FACTO LIDO** | `README.md` do repositório de empacotamento, verbatim |
| o remalhador de produção não depende dela | **FACTO LIDO** | dois `CMakeLists.txt` (`OFF`), o README de instalação (`=0`), o `Dockerfile` (`=0`) e o **nosso** `cmake.log:108` |
| existe emparelhamento livre na mesma árvore | **FACTO LIDO** | licença Boost + nomes de classe da biblioteca de grafos |
| runtime da Rive é MIT; editor é fechado | **FACTO LIDO** | `LICENSE` do repositório público + ausência de repositório do editor |
| abrir código próprio não concede direito sobre GPL alheio | **FACTO LIDO** | GPLv3 §5(c), verbatim |
| pipes/linha de comando ⇒ normalmente programas separados | **FACTO LIDO** (texto de FAQ) + **INTERPRETAÇÃO** (a FAQ não é lei) | GPL FAQ, anchors nomeados |
| a definição de «aggregate» | **FACTO LIDO** | GPLv3 §5, último parágrafo |
| as obrigações 6a/6b/6d | **FACTO LIDO** | GPLv3 §6 |
| MPL-2.0 permite Larger Work fechada | **FACTO LIDO** | MPL-2.0 §3.3, verbatim |
| os três produtos fechados do §4.2 | **FACTO** (prática documentada e pública) | páginas de conformidade dos próprios |
| a lição do congelamento na GPLv2 | **INTERPRETAÇÃO CORRENTE** | leitura consensual; a Apple não publicou o motivo |
| Artifex v. Hancom | **FACTO** (decisão sobre exigibilidade contratual) + ⚠️ **acabou em acordo**, sem mérito sobre combinação | noticiário jurídico público |
| a cadeia de 10 crates é separável | **FACTO LIDO** | `Cargo.toml` de cada uma |
| a GPL do remalhador não vem de uma dependência só | **FACTO LIDO** | quatro ficheiros de licença independentes |
| a oferta de licença comercial da biblioteca de extracção | **FACTO LIDO** | página oficial da universidade, verbatim |
| a alínea T1(d) nunca foi tentada | **FACTO** (ausência medida) | `grep` no ledger e na triagem: zero ocorrências |

---

## ⛔ Recusas e cercas MEDIDAS neste documento

| nº | recusa / cerca | mecanismo | §|
|---|---|---|---|
| 1 | ⛔ **«abrir o nosso código liberta o GPL alheio»** — não reconstruir este raciocínio | a permissão vem do detentor do direito daquele código; GPLv3 §5(c) proíbe relicenciar | §3.2 |
| 2 | ⛔ **«o emparelhamento não-livre bloqueia a cadeia»** — REFUTADO | está `OFF` por omissão em dois níveis, os autores mandam desligá-lo, e a **nossa** compilação foi sem ele | §2.2 |
| 3 | ⛔ **«a GPL do remalhador é herdada de uma dependência»** — REFUTADO | quatro portas independentes de GPL | §6.3 |
| 4 | ⛔ **crate aberta que liga ao editor fechado E absorve GPL alheio** — impossível, não tentar | GPLv3 §5(c): a licença aplica-se «to the whole of the work, and all its parts» | §5.3 |
| 5 | ⚠️ **cerca nomeada:** ligar (estática ou dinâmica) qualquer código GPL no editor | padrão de facto do Artifex v. Hancom | §4.3.1 |
| 6 | ⚠️ **cerca nomeada:** um auxiliar escrito por nós, para nós, inútil sozinho, **não** é agregado | GPLv3 §5: *«not by their nature extensions of the covered work»* | §4.3.3 |
| 7 | ⚠️ **cerca nomeada:** cláusula de EULA que proíba extrair/estudar o binário GPL entregue | GPLv3 proíbe restrições acrescentadas | §4.4.4 |
| 8 | ⚠️ **a reabertura do ADR-0167 precisa da MEDIÇÃO, não deste documento** | só **1 das 3** razões dele era de licença | §7.1 |
