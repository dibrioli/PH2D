# SKILL — Clean-room: reimplementação de código restrito (a porta legal para o estado da arte)

> **Fonte única** do protocolo que transforma qualquer código publicado-mas-restrito (GPL,
> AGPL, copyleft em geral) em feature do PH2D — **sem contaminar o produto e sem pagar o
> preço de reconstruir às cegas a partir dos papers**. Genérica de propósito: o alvo entra
> pelo **BLOCO-LINHA** do §10, como em
> [`MODELO_ABERTURA_LINHA.md`](../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md).
> Verificada adversarialmente em 4 lentes (jurídica · implementador cético · canais de
> vazamento · consistência com a casa) em 2026-08-24; 52 achados aplicados.
> **Reescrita em 2026-08-25 por veredito de operação do Enio** (a jornada do quad remesh):
> **uma feature = UMA linha; janelas ao mínimo** — o fluxo de 4 janelas encadeadas morreu;
> o fluxo é UMA janela orquestradora, com E/R como subagentes e revezamento pela MESMA
> linha (§3, §10). A base legal e os instrumentos (§§0–2, 4–9) não mudaram.
>
> A tese em uma linha: **quase tudo que interessa num código restrito não é protegível —
> algoritmo, matemática, comportamento, formato, interface. A única coisa protegida é a
> ESCRITA. Este protocolo toma tudo que a lei deixa na mesa e não toca na única coisa que
> ela guarda.**

---

## §0 — O que esta diretriz compra, e a única regra que a sustenta

O pipeline tem **três âncoras legais independentes** — cada ato tem a sua, nenhum depende
de zona cinzenta:

1. **Estudar, rodar, modificar e instrumentar o código restrito, em privado, é
   permitido** — não por tolerância, mas pela própria licença copyleft (§1.1). O
   Especificador opera com licença na mão.
2. **Ideia, algoritmo, método, matemática, comportamento e formato não são protegíveis
   em lugar nenhum do mundo** (§1.2). A especificação funcional carrega 100% do valor de
   engenharia e 0% da expressão protegida.
3. **Criação independente não é derivação.** Quem implementa a partir da espec + papers,
   sem nunca ter tido a escrita original no contexto, produz obra própria — e semelhança
   **funcional** com programa preexistente é *expressamente lícita* no Brasil (§1.3).

A regra única que segura as três âncoras:

> ⛔ **QUEM ESCREVE O CÓDIGO DO PRODUTO NUNCA TEVE A EXPRESSÃO ORIGINAL NO CONTEXTO.**

Tudo o mais nesta diretriz é maximalista — a espec pode conter *tudo* que for funcional
(§4), o oráculo pode ser instrumentado *à vontade* (§5), as saídas dele entram no repo
como fixtures (§5). A parede não é prudência: é a **fundação**. É ela que converte
"prove que você não copiou" em "aqui está o diário de quem escreveu sem ver". E a parede
**tem instrumento** — `scripts/cleanroom-sweep.sh` (§7) — porque a lei medida desta casa
diz que regra sem instrumento é nota que envelhece.

⚠️ **O meio-termo não existe.** Copiar primeiro e "reescrever depois" produz descendente
da cópia — cada elo de uma corrente cópia→mutação→tradução é obra derivada do anterior,
e o histórico do git documenta a corrente para sempre. Este protocolo não é a versão
tímida dessa estratégia: é a única versão dela que **entrega um produto que pode ser
lançado**. A agressividade mora nos §§1–5 e §10; os §§6–8 são o que a mantém viva.

---

## §1 — A base legal, citável (leia uma vez; o Implementador recebe o resumo no §9)

### §1.1 — Os atos privados sobre código copyleft são LICENCIADOS, não tolerados

- **GPLv3 §2:** *"You may make, run and propagate covered works that you do not convey,
  without conditions **so long as your license otherwise remains in force**."* — rodar,
  copiar, modificar e instrumentar em privado, sem condições; a única ressalva é a
  licença não ter **terminado** por violação anterior (§8 da GPLv3). Em estado normal,
  os atos privados são incondicionais, e disparam obrigação só no *convey*.
- **GPLv2:** o §0 diz *"The act of running the Program is not restricted"*, e a leitura
  consensual (inclusive da FSF, em FAQ) é que as condições do §2 qualificam
  *distribuição* — modificação privada não exige release. ⚠️ Ao contrário da GPLv3, é
  inferência consensual, não concessão textual expressa: para alvo GPLv2-only, anote-a
  no ledger e cumpra trivialmente o §2(a) (notice de mudança nos arquivos
  instrumentados). Custo: uma linha por arquivo.
- ⚠️ **Exceção AGPL:** o §13 da AGPLv3 dispara a obrigação de oferecer o fonte quando
  usuários **interagem por rede** com a versão modificada — **sem** *convey* nenhum.
  ⇒ O oráculo AGPL instrumentado roda **LOCAL, sempre** — nunca como serviço acessível
  a terceiros (nem "só para outra máquina do time"). Se um dia for exposto, a obrigação
  é oferecer o fonte **do oráculo modificado** (cumprível e não-contaminante — ele já
  vive fora do produto), e o evento vai ao ledger.
- **Diretiva UE 2009/24/CE, art. 5(3) + art. 8:** observar, estudar e testar o
  funcionamento de um programa para determinar ideias e princípios é direito do usuário
  legítimo **que nenhum contrato pode retirar** (cláusula contrária é nula). ⚠️ Regra
  europeia — o Brasil não tem equivalente; a consequência prática está no §2.T4.
- **Saída de programa não é coberta pela licença do programa — e isso é TEXTO DE
  LICENÇA**, não opinião: GPLv2 §0 (*"the output from the Program is covered only if
  its contents constitute a work based on the Program"*) e GPLv3 §2 (*"The output from
  running a covered work is covered by this License only if the output, given its
  content, constitutes a covered work"*). A FSF corrobora em FAQ. A ressalva *given its
  content* é exatamente o filtro de dumps do §5: malha que sai é dado; texto do programa
  que vaza num log é programa.

⇒ **Todo o trabalho do Especificador (§3.E) e todo o uso do oráculo (§5) está coberto
por concessão da própria licença.** Não precisamos de *fair use* nem de analogia para
essa fase.

### §1.2 — O que NUNCA é protegível (o piso mundial)

- **TRIPS art. 9(2)** (vincula todos os membros da OMC; internalizado no Brasil pelo
  Decreto 1.355/94): *"Copyright protection shall extend to expressions and not to
  ideas, procedures, methods of operation or mathematical concepts as such."*
- **EUA — 17 U.S.C. §102(b):** ideias, procedimentos, processos, sistemas, métodos de
  operação, princípios e descobertas ficam fora, *"regardless of the form in which
  [they are] described"*.
- **Brasil — Lei 9.610/98, art. 8º**, incisos I e II (texto legal): não são objeto de
  proteção *"as idéias, procedimentos normativos, sistemas, métodos, projetos ou
  conceitos matemáticos como tais"* e *"os esquemas, planos ou regras para realizar
  atos mentais, jogos ou negócios"*. ⚠️ Note o qualificador "normativos" — no foro
  brasileiro, o peso para software está nas duas âncoras seguintes: TRIPS 9(2) via
  Decreto 1.355/94, e sobretudo a Lei 9.609 art. 6º, III (§1.3).
- **CJEU, SAS Institute v. World Programming (C-406/10, 2012)** — o precedente mais
  colado no nosso caso: a WPL reconstruiu o interpretador da SAS estudando manuais e
  comportamento, com paridade de saída como meta declarada. O tribunal: **funcionalidade,
  linguagem de programação e formato de arquivos de dados não são expressão protegida
  do programa** sob a diretiva de software (com a reserva teórica de que linguagem e
  formato poderiam ser protegidos como *obras autônomas* sob a diretiva geral, se forem
  criação intelectual própria). Reimplementar comportamento observado é lícito.
  ⚠️ **E a lição do mesmo litígio:** a WPL **PERDEU** a claim manual-contra-manual — o
  manual dela reproduzia substancialmente o *texto* dos manuais da SAS. Fatos e método
  são livres; **o wording de manual é obra literária plena**. É a regra do §4.2:
  a espec descreve com palavras NOSSAS.
- **SCOTUS, Google v. Oracle (2021):** copiar as *declarações* da API Java para
  reimplementação foi *fair use* — a Corte tratou reimplementação de interface como
  atividade valiosa e transformadora. (Não decidiu que API é incopiável; decidiu que
  reimplementá-la daquele modo é uso lícito — para nós é forro extra, não fundação.)

### §1.3 — Criação independente + a joia brasileira

- Direito autoral, ao contrário de patente, **só alcança cópia**. Duas obras idênticas
  criadas de forma independente são duas obras legítimas. A pergunta jurídica é sempre
  **acesso + derivação**, e a parede do §3 elimina o acesso de quem escreve — e o §7
  detecta e cura o acesso que nenhuma parede de sessão controla (o treino do modelo).
- **Lei 9.609/98, art. 6º, III** (a lei brasileira de software): *não constitui ofensa*
  a **semelhança de um programa com outro, preexistente, quando decorrer das
  características funcionais de sua aplicação**, da observância de preceitos normativos
  e técnicos, ou de limitação de forma alternativa para a sua expressão. ⇒ No nosso
  próprio foro, a paridade de comportamento — o objetivo declarado do protocolo — é
  **expressamente não-infratora**.

### §1.4 — O processo tem 40 anos de prática validada — em tribunal e fora dele

- **Compaq (1982) e Phoenix (1984):** clean-room do BIOS da IBM — fundou a indústria
  dos PCs compatíveis. Não é jurisprudência (a IBM nunca conseguiu atacar): é o padrão
  de indústria cuja dissuasão também é dado.
- **NEC v. Intel (N.D. Cal. 1989):** um clean-room encomendado **como evidência** — a
  versão escrita por quem não tinha acesso convergiu com a da Intel, provando que as
  semelhanças eram ditadas por restrições funcionais, não por cópia. O valor
  **probatório** do quarto limpo é o ponto.
- **Computer Associates v. Altai (2d Cir. 1992):** o caso que criou o teste padrão de
  comparação (abstração→filtragem→comparação) — e no próprio caso, a **reescrita
  clean-room** (OSCAR 3.5), feita por programadores sem acesso ao original e
  documentada, **sobreviveu**, mesmo depois de a primeira versão ter sido cópia
  literal. É o precedente direto de *documentar é a defesa* e do protocolo de
  incidente (§6): contaminação se cura com reescrita por quem não viu, registrada.
- **Sega v. Accolade (9th Cir. 1992)** e **Sony v. Connectix (9th Cir. 2000):** até a
  **cópia intermediária integral** (desmontar/copiar para estudar) foi *fair use* quando
  o fim era alcançar os elementos funcionais. Nós nem precisamos desse forro para alvos
  copyleft — a licença já concede (§1.1) — mas ele existe.

### §1.5 — Onde a lei NÃO abre (a lista honesta — é curta)

1. **Expressão de código:** texto, trechos, comentários, estrutura-como-escrita,
   grandes tabelas afinadas à mão copiadas verbatim (§4.2).
2. **Expressão de PROSA do alvo:** wording de manuais, READMEs, doc-comments e papers —
   os fatos e o método que eles descrevem são livres; **o texto deles não** (a SAS
   ganhou exatamente essa claim — §1.2). A espec re-descreve com palavras nossas.
3. **Assets** (ícones, texturas, sons, fontes, shaders de exemplo, presets, malhas de
   exemplo): obras plenas, nunca — nem como *entrada* de fixture (§5).
4. **Patente viva** protege a *ideia* — é o inverso do copyright, e clean-room **não
   ajuda em nada** contra ela. Checkpoint obrigatório no §8.1. (Patente **expirada** é
   o contrário: divulgação pública total + domínio público = o melhor documento de
   espec que existe. Ex.: marching cubes, US 4.710.876, expirada em 2005 — hoje livre.)
5. **Binário sob EULA sem fonte:** não há concessão de cópia nenhuma — nem privada.
   Lane restrita no §2.T4, com a leitura do EULA como passo obrigatório.
6. **Marcas:** nome de produto alheio não entra na nossa UI (citar em doc interno é uso
   nominativo, lícito).

---

## §2 — Triagem: a escada de portas (percorra NA ORDEM, pare na primeira aberta)

⚠️ **O primeiro ato é sempre LER A LICENÇA REAL do alvo** — o texto, não a reputação.
"Restrito" não é uma coisa só, e a porta mais barata costuma estar destrancada.

| Degrau | Situação do alvo | O que fazer | Custo |
|---|---|---|---|
| **T0** | Licença **permissiva** (MIT/BSD/Apache/zlib…) | **Porte fiel, verbatim se quiser.** Sem parede, sem espec — só manter a atribuição. Precedente da casa: SculptGL (MIT) portado a 1 ULP; Instant Meshes (BSD). | Horas |
| **T0½** | **Copyleft por-arquivo** (MPL-2.0/EPL/CDDL) ou **LGPL linkável** | Porte verbatim é permitido **mesmo em produto fechado**, mas ⚠️ o copyleft é do ARQUIVO: o arquivo copiado permanece sob a licença dele, com cabeçalho preservado e o fonte DELE disponível (MPL-2.0 §3.1–3.2). Aceitável no repo? Use. Inaceitável? A reescrita pelo pipeline T2 remove a obrigação. | Horas–dias |
| **T1** | Alvo copyleft, mas o **ecossistema** tem irmãos permissivos | **Caçada antes da obra:** (a) código de referência dos **autores do paper** (acadêmicos frequentemente publicam BSD/MIT ao lado do repo GPL); (b) **versões antigas** do próprio alvo sob licença anterior mais branda; (c) reimplementações permissivas independentes (⚠️ E valida a **proveniência**: um "porte MIT" que descende do alvo é lavagem alheia — checar autoria, história, se declara clean-room); (d) **e-mail aos autores** pedindo dual-license — GPL acadêmica é negociável com frequência, às vezes é GPL só por causa de UMA dependência. 30 minutos de busca podem economizar semanas. | Minutos–dias |
| **T2** | **Copyleft com fonte** (GPL/AGPL/LGPL-não-linkável) | **O pipeline desta diretriz** — §§3–7 + blocos do §10. É o degrau para o qual tudo abaixo foi escrito. ⚠️ AGPL: oráculo sempre local (§1.1). | Dias–semanas |
| **T3** | **Source-available** (BSL, SSPL, Elastic, PolyForm…) | Ler a concessão real: várias permitem cópia/modificação fora de produção. Se conceder atos privados ⇒ tratar como T2 **com a concessão anotada no ledger**. Se não conceder cópia nenhuma ⇒ o *fonte* é intocável: cair para T4. | = T2 |
| **T4** | **Proprietário sem fonte** (ZRemesher, produtos comerciais) | **Lane de comportamento puro:** rodar o produto que possuímos/licenciamos legitimamente e observar entradas→saídas em uso normal; papers, patentes (§8.1 — patente PUBLICA o método; expirada = ouro), palestras GDC/SIGGRAPH dos próprios engenheiros deles, manuais (fatos, nunca o wording). ⚠️ **Passo obrigatório: ler o EULA** procurando cláusula anti-observação/anti-benchmark/anti-produto-concorrente — na UE ela é **nula** (2009/24 art. 8); **no Brasil não há norma que a anule**: se existir, o risco é CONTRATUAL e vai ao Enio com a cláusula transcrita no ledger, ANTES de a lane abrir. ⛔ **Sem descompilação/desmontagem** (a lei brasileira não dá exceção geral; a europeia só para interoperabilidade). | Semanas+ |

⚠️ **T1 é o degrau que a pressa pula e não devia:** a pergunta *"existe QUALQUER
implementação permissiva deste algoritmo no mundo?"* custa uma sessão de busca e tem
resposta *sim* com frequência surpreendente. ⚠️ E se a rota escolhida virar um porte
T0/T0½, o clean-room **acabou**: é **porte fiel**, sem parede — e no fluxo do §3 quem
leu o alvo copyleft foi um subagente, então a **própria janela da linha** porta o
código permissivo (o contexto dela segue limpo; mantenha atribuição/cabeçalhos).

---

## §3 — Os três CONTEXTOS, a parede — e o fluxo: UMA linha, UMA janela

A separação E/I/R nunca foi sobre janelas — é sobre **contextos**: quem escreve o
produto não pode ter tido a expressão do alvo no contexto. ⚠️ Em LLM, **contexto é
exposição**: contexto que conteve o fonte do alvo está queimado para escrever produto —
e compactação **não lava** (o resumo descende do que a janela viu), nem `--resume`
(o resume restaura).

O fluxo que honra a parede com o MÍNIMO de operação (veredito do Enio, 2026-08-25,
depois da jornada do quad remesh — o fluxo anterior, de 4 janelas encadeadas e linhas
em sequência, **morreu**):

- **Uma feature = UMA linha** (`line/<módulo>`). Espec, ledger, vassoura, fixtures e
  código nascem e commitam **na mesma worktree**, e chegam ao main juntos, pela
  integração normal de uma linha (§0.7 da casa continua valendo: integrar/shipar só
  por ordem do Enio). ⛔ Nunca uma segunda linha para a mesma feature.
- **Uma janela por vez — e ela é o I.** A janela da linha opera sob as regras do
  Implementador **desde a 1ª mensagem** (nunca abre o fonte); os papéis E e R são
  **subagentes** dela — contexto isolado do dela por construção. O canal de volta
  deles é o **contrato de retorno** (§3.E).
- **Revezamento, nunca paralelismo:** quando a janela enche, queima (§6) ou a jornada
  acaba, ela escreve o **BLOCO-RETOMADA** (§10), imprime-o e **PARA**; a janela nova
  assume a **MESMA linha** e continua do passo registrado. Nunca duas janelas na linha
  ao mesmo tempo. Tudo o que é durável (espec, ledger, código, INBOX) já vive em
  disco — a troca custa uma colagem.
- **A parede é só do T2/T3.** T0/T0½ (porte permissivo) e T4 (comportamento puro — não
  há fonte para separar) dispensam subagentes: a janela faz tudo sozinha.

### E — ESPECIFICADOR (subagente; contaminado por definição, e tudo bem)

- **Vê tudo:** o fonte do alvo, os papers, manuais, issues, o nosso repo, a internet.
- ⚠️ **E LÊ O FONTE INTEIRO antes de escrever a espec** — travessia sistemática,
  arquivo a arquivo, mais a história: commits marcantes, issues, PRs, design docs,
  palestras dos autores. A **cobertura** vai ao ledger (áreas/arquivos percorridos,
  datas). Espec de leitura parcial é o defeito caro: o que não foi lido vira buraco
  que ninguém sabia que existia — e é onde mora a dica que os autores pagaram para
  aprender (§4.1.12).
- **Produz:** a espec (§4) · o oráculo instrumentado + dumps (§5) · o ledger (§6) · a
  vassoura codificada (§7) · o README de 3 linhas de `cleanroom/`.
- ⛔ **Nunca escreve código de PRODUTO** — nem "só um esqueleto", nem "só a assinatura".
  Um esqueleto escrito por quem viu o original é a estrutura do original entrando no
  produto pela porta lateral. ✓ **Harness PODE:** bancada de paridade, geradores de
  fixture, wrappers do oráculo — fora das crates de produto, com sweep antes do
  commit (um comparador de dumps não carrega o algoritmo; a janela fica livre para
  reescrevê-lo).
- ⛔ **TUDO do alvo vive em `~/Referencias/<alvo>/`** — fonte, builds, instrumentação,
  notas, **rascunhos da espec** (`draft/`). Nada disso se materializa no repo, em
  `/tmp`, `/dev/shm` nem no scratchpad (a janela-mãe alcança todos). Precedente da
  casa: o clone GPL do quad remesh vive fora do repo, em `ph2d-quadbench/oracle/`
  ([ADR-0162](../architecture/decisions/0162-quad-remesh-pivots-to-the-global-family-clean-room-from-papers-gpl-oracle-outside.md));
  `~/Referencias/` é a convenção daqui pra frente.
- **Trabalha DENTRO da worktree da linha** e commita lá (scoped, docs-only,
  `--no-verify`) — espec, ledger e fixtures chegam ao main **com a integração da
  linha**, nunca antes e nunca por fora. ⛔ **A espec entra num commit ÚNICO,
  pós-filtragem (§4.3)** — nunca rascunhos incrementais: `git log -p` retém para
  sempre o que um rascunho contaminado carregou.
- ⛔ **Todo artefato destinado à janela-mãe passa `scripts/cleanroom-sweep.sh` ANTES
  do commit/entrega** — espec, emendas, fixtures, READMEs, **o texto do próprio
  REPORT final** (grave-o em `~/Referencias/<alvo>/draft/`, varra, só então devolva) —
  e **qualquer escrita em `project-memory/`** (⚠️ o symlink da memória é
  compartilhado: uma "lição" com identificador do alvo contamina toda janela futura
  desta máquina — inclusive a janela de retomada que um incidente exigiria).
  No CLAUDE.md §5 e em handoffs, o conteúdo permitido sobre o alvo é o **nome + link
  para `cleanroom/`** (uso nominativo) — zero identificador interno, zero mecanismo.
- ⚠️ **Dumps publicados têm chaves/colunas/tags renomeadas para vocabulário do
  DOMÍNIO** — a regra de nomes do §4.2 vale para o formato do dump; e nomes de
  arquivos/pastas de fixtures idem (vazam por `ls`, e grep de conteúdo não os vê).
- ⚠️ **O deny da linha (passo 0 do I) nega `Read` para a sessão INTEIRA — subagentes
  incluídos.** Não é defeito: a cerca é da janela-mãe. O subagente E lê o fonte por
  shell (`cat`/`rg`) — coberto pela licença do alvo (§1.1) — e a cerca REAL dele é o
  **contrato de retorno**.
- **CONTRATO DE RETORNO (a cerca que protege a janela-mãe):** o report final é UMA
  frase fixa + caminhos + contagens, e no máximo 5 linhas de fatos estritamente
  FUNCIONAIS — ⛔ zero identificador interno, zero trecho, zero wording do alvo. O
  report de um subagente entra no contexto da janela-mãe **sem filtro prévio
  possível** — é o único canal sem sweep antes da chegada; violá-lo queima a
  janela-mãe (incidente §6).
- **Emendas:** dúvida da janela-mãe → um subagente-E **novo** (ou o mesmo, por
  continuação) lê o que precisar, emenda a espec (a emenda passa o sweep) e devolve
  pelo mesmo contrato. A janela-mãe **nunca** vai olhar.

### I — IMPLEMENTADOR (a janela da linha; limpa, e mantida limpa por MECANISMO)

- **É a janela que abre — ou assume — a linha**, e nunca conteve o fonte do alvo.
  Cada janela da corrente de revezamento declara o próprio session-id na entrada
  (via inbox §6) — o R-pós confere no fechamento que nenhum elo pertence ao conjunto
  {contextos que leram o fonte, janelas queimadas}.
- **Passo 0 mecânico (a parede vira permissão do harness, não lembrança do agente):**
  criar na raiz da worktree `.claude/settings.local.json` com deny de leitura:
  ```json
  { "permissions": { "deny": [
      "Read(~/Referencias/**)",
      "Read(docs/**/cleanroom/LEDGER_*)",
      "Read(docs/**/cleanroom/VASSOURA_*)"
  ] } }
  ```
- **Lê:** a espec (cujo **cabeçalho** carrega os atestados que I confere — §4) · os
  papers públicos, guiado pelo **mapa de leitura** do cabeçalho · o NOSSO código · os
  **dumps e goldens** do oráculo (dados) · **toda a PROSA pública do alvo** — site de
  docs, manual, blog, palestras, FAQ (foi exatamente o insumo lícito de SAS v. WPL) —
  **pulando os listings de código** e sem nunca transcrever o wording (§1.5.2) ·
  dentro de `cleanroom/`, **SÓ** `SPEC_*`.
- ⛔ **Nunca:** abrir `~/Referencias/<alvo>/` **ou qualquer diretório que contenha o
  fonte do alvo** (ex.: `ph2d-quadbench/` contém `oracle/` = o clone GPL — I consome só
  os dumps de `ref/` pelos caminhos que a espec dá) · **as superfícies do alvo que
  RENDERIZAM fonte** — hospedagem de código, issue tracker, PRs, code-search (issues
  carregam diffs de mantenedor; comportamento relatado em issue chega à janela como
  emenda de E à espec) ·
  **portes/forks/traduções do alvo em qualquer linguagem e sob qualquer licença**
  (contam como código do alvo; a elegibilidade de um irmão permissivo é triagem T1,
  papel de E) · **listing compilável em apêndice de paper dos autores do alvo** (é a
  mesma expressão; o mapa de leitura do cabeçalho diz o que pular) · **transcrever
  código executável de QUALQUER fonte externa** (Stack Overflow, blog, gist — SO é
  cheio de colagens GPL sem atribuição; as fontes de código de I são exclusivamente
  espec + papers + repo PH2D) · **o `--help`/verbose do oráculo** (✓ rodá-lo em modo
  só-dados com `2>/dev/null` é livre; ferramenta que põe texto do programa no stdout
  vai por wrapper de E) · **ler/grepar os `.jsonl` crus de `~/.claude/projects/`**
  (os transcripts dos subagentes E contêm o fonte verbatim; ✓ sondas agregadas como
  `agent-loop-profile.sh` seguem livres — imprimem métricas, não conteúdo) ·
  **mensagem direta (SendMessage) com a janela anterior da corrente ou com qualquer
  sessão que tenha lido o fonte** (mensagem não deixa rastro em disco versionado; a
  troca de janela é pelo BLOCO-RETOMADA + disco, e dúvida viaja como emenda à espec
  por subagente-E).
- ⚠️ **Disciplina de busca:** o cabeçalho da espec traz a **denylist de URLs** do alvo
  (repo, mirrors, agregadores de code-search) — confira o URL **antes** do fetch (fetch
  é irreversível: o conteúdo entra no contexto para sempre). Busque por conceito/paper,
  não por `<nome do alvo> + source/extensão/nome de API`. Snippet visto num **preview**
  de resultado = relance registrável (§6), não queima a janela.
- ⚠️ **Tripwire de recall:** se você se pegar "sabendo" um detalhe que espec+papers não
  deram (nome interno, typo, constante mágica) — **não o escreva**; PARE e reporte como
  suspeita de recall (§6, inbox). R trata como achado de convergência. A dúvida é do
  processo; **o dever de reportar o sinal é seu**.
- ⚠️ **Subagentes da implementação** (buscas no repo, exploração, harness): nascem do
  contexto limpo, mas o canal de VOLTA é deles — todo briefing carrega **verbatim** o
  parágrafo ⛔ acima, mais *"nunca cite código em reports — só fatos funcionais"*.
  Report que volte com código do alvo = incidente §6 **da janela** (o dano já está no
  contexto dela).
- ⚠️ **Abertura e retomada:** o ritual da casa — `cd` na worktree, `pwd`,
  `git branch --show-current` — antes de ler qualquer arquivo.
- **Dúvida que a espec não responde** → despache um subagente-E de emenda (§3.E).
  **Nunca** vá olhar.

### R — REVISOR (subagentes; veem os dois lados; não escrevem produto; DOIS momentos)

- **R-PRÉ (antes de a janela implementar):** um subagente **que não seja o
  subagente-E** (auditar a própria filtragem é o que falha; despachá-lo novo dá o
  contexto independente por construção). Audita a espec contra o §4.2 — pseudo-código
  espelhado, wording de manual, nomes internos, tabela verbatim — e atesta no cabeçalho
  (*"auditada contra §4.2 por R em <data>"*). Sem esse atestado, a janela não
  implementa. É a única defesa contra o erro de autofiltragem de E — sem ela, I viraria
  tradutor de obra derivada **sem meio de perceber** (nunca viu o original).
- **R-PÓS (após paridade verde):** auditoria de paridade (§5) · vassoura + revisão
  estrutural (§7) · varredura de histórico e memória (§7) · incidentes tratados ·
  fechamento do ledger (§6).
- ⛔ **Não escrevem nem ditam código de produto.** Achado volta **em termos
  funcionais** ("a decomposição da fase 3 convergiu com a do original sem força
  funcional — re-derive só da espec §N"), **nunca** com trecho do original — e pelo
  mesmo contrato de retorno.

### O revezamento (a corrente de janelas na MESMA linha)

- **Gatilhos:** contexto cheio · incidente §6 (a janela queimou como I) · fim de
  jornada.
- **A janela que sai:** preenche o BLOCO-RETOMADA (§10), roda o sweep sobre ele (o
  script decodifica a vassoura em memória — ela nunca é lida), salva em
  `cleanroom/RETOMADA_<alvo>.md`, imprime-o inteiro e **PARA**. Não fecha a linha,
  não integra, não "adianta só mais um passo".
- **A janela que entra:** FASE 0 do
  [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
  + passo 0 do I + session-id no INBOX; o ledger ganha o elo novo com o motivo da
  troca. **Cada elo satisfaz a mesma condição: nunca teve o fonte do alvo no
  contexto.**
- ⚠️ **O preço declarado do fluxo de janela única:** o report de um subagente entra no
  contexto sem filtro prévio — o contrato de retorno é a cerca e o sweep do próprio
  report é o instrumento (§3.E). Se falharem, a troca custa uma colagem: tudo o que é
  durável já vive em disco.

⚠️ **A parede em uma frase operacional:** *E fala com a janela somente pela espec e
pelo contrato de retorno; R somente por achados funcionais; o material do alvo não
entra no repo, na memória, no CLAUDE.md, em /tmp, nem no contexto de quem escreve
produto — e todo artefato que cruza a parede passa o sweep antes.*

---

## §4 — A ESPEC: tudo que pode (é quase tudo) e o pouco que não pode

Arquivo: `docs/<Módulo>/cleanroom/SPEC_<alvo>.md`. **Cabeçalho obrigatório** — é também
o quadro de atestados que I confere (I não lê o ledger):

```
Alvo: <nome, versão> · Licença: <qual> · Degrau: T_
Ledger: aberto em <path>, <data>
Patente (§8.1): buscado em <data>, termos, resultado
Filtragem §4.3: executada em <data> · Sweep: verde em <data>
Auditoria §4.2 (R-pré): <id do subagente R-pré>, <data>
Mapa de leitura da literatura: <papers/seções livres; apêndices a PULAR>
Denylist de URLs (repo do alvo, mirrors, code-search): <lista>
"Este documento descreve comportamento; não contém expressão do alvo."
```

### §4.1 — PODE (seja maximalista — cada item abaixo tem âncora no §1)

1. **Arquitetura de fases e fluxo de dados** — o *o quê* de cada fase, o que entra, o
   que sai, em que ordem e por quê (quando o porquê for funcional).
2. **Toda a matemática:** fórmulas, derivações, condições de contorno, critérios de
   convergência e de parada, esquemas numéricos, escolhas de discretização. Matemática
   não tem dono (TRIPS 9(2) · art. 8º · §102(b)).
3. **Constantes e defaults como FATOS DE COMPORTAMENTO**, um a um, com proveniência:
   *"observado: ε = 1e-6 na fase 3 (dump de 2026-08-24, fixture X)"*. Um número medido
   é um fato; a casa já vive sob "todo número com a medição ao lado" (CLAUDE.md §0.0).
4. **Formatos**: arquivos, mensagens, layouts externos, unidades, sistemas de
   coordenadas, convenções de orientação (SAS v. WPL: formato de dados não é protegido).
5. **Comportamento de borda, caso a caso:** entrada vazia, malha não-manifold, NaN,
   degenerados, overflow, limites — o que o alvo FAZ em cada um (observado, com
   fixture). Issues do alvo são ótima fonte — **E** as lê e destila.
6. **Vetores de teste:** pares entrada→saída **gerados rodando o oráculo** (§5 — saída
   de programa é dado, com a regra de proveniência de ENTRADA do §5). É o coração da
   espec.
7. **Dumps de fase intermediária** e o formato deles (precedente da casa: os
   `*.rosy`/`*.patch` do quadwild, já usados fase a fase).
8. **Notas de custo:** complexidade por fase, onde o tempo mora, o que domina em malha
   grande — requisitos de performance são comportamento.
9. **Determinismo como requisito:** *"a fase 4 exige ordem de visita estável por chave
   X, senão o resultado oscila"* — a exigência, não a transcrição do laço.
10. **Pseudo-código NO NÍVEL DO PAPER** — o algoritmo como a literatura o publica.
    Regra prática: se o pseudo-código pudesse estar num paper de terceiros descrevendo
    o método, pode estar na espec.
11. **O ALGORITMO INTEIRO, em qualquer profundidade.** Método não é protegido em
    profundidade nenhuma (§1.2) — a espec pode reconstruir o algoritmo completo,
    passo a passo, decisão a decisão, sem limite de detalhe; o limite é a FORMA
    (§4.2), nunca a profundidade. ⚠️ A técnica que mantém a forma nossa é a **regra
    do arquivo fechado:** pseudo-código detalhado se escreve com o fonte FECHADO, a
    partir da compreensão; reabrir para conferir devolve um FATO ("o laço para
    quando Δ < ε"), nunca linhas transcritas.
12. **A sabedoria dos autores, com proveniência.** Tudo o que os desenvolvedores
    originais aprenderam e registraram — comentários, mensagens de commit, issues,
    PRs, design docs, palestras, threads — entra na espec como fato **re-dito em
    palavras nossas, com o link** ("os autores relatam que a fase 3 diverge quando
    X — issue #NNN"). ⚠️ **Citação verbatim CURTA de PROSA é permitida** pelo
    direito de citação (Lei 9.610 art. 46, III: para fins de estudo, na medida
    justificada, com autor e origem; *fair use* idem): entre aspas, com fonte, só
    quando o wording exato importa. ⛔ Código nunca é "citação", e uma citação não
    migra da espec para código/doc/comentário do produto. R-pré audita a medida.
13. **Nomes PÚBLICOS de interface e formato, quando a compatibilidade os exige.**
    Campo de formato de arquivo, nome de API pública, keyword de linguagem do alvo:
    quando ler/escrever o formato ou ser compatível EXIGE o nome exato, o nome é o
    próprio fato funcional (SAS v. WPL: formatos; Google v. Oracle: declarações;
    9.609 art. 6º III: "observância de preceitos técnicos"). ⛔ Nome INTERNO
    (função, variável, arquivo do fonte) continua fora — §4.2: interno se renomeia,
    interface pública se documenta.

### §4.2 — NÃO PODE (a lista curta que decide tudo)

- ⛔ **Texto de código, trechos, diffs** — nem uma linha, nem "só para ilustrar".
- ⛔ **Nomes internos** do alvo (funções, variáveis, arquivos, structs). Renomeie para
  vocabulário do **domínio** ("o passo de suavização do campo", não o nome da função).
  ⚠️ Vale para chaves/colunas de dumps e para **nomes de arquivos** de fixtures.
- ⛔ **Comentários do original** — são a expressão mais protegida do arquivo.
- ⛔ **Wording de manual/README/doc-comment/paper do alvo, verbatim ou quase** — os
  fatos são livres, o texto não (a claim que a SAS **ganhou** — §1.2). Re-descreva —
  ou cite CURTO, entre aspas e com a fonte, sob o direito de citação (§4.1.12).
- ⛔ **A organização arquivo-a-arquivo / função-a-função** quando ela é arbitrária.
  Descreva por **fases funcionais**; a decomposição em unidades de código é escolha do
  Implementador.
- ⛔ **Pseudo-código que espelha o original linha a linha** — isso é tradução, não
  descrição (a corrente do §0).
- ⛔ **Tabelas grandes afinadas à mão (LUTs) copiadas verbatim.** Caminhos lícitos, em
  ordem: (a) a fórmula geradora, se o paper/a matemática a dá; (b) **re-medição por
  varredura do oráculo** (a tabela vira fato observado, com o harness registrado);
  (c) re-afinação nossa contra os vetores de paridade. Registre QUAL caminho foi usado.

### §4.3 — Filtragem (E executa ANTES de entregar; anota no cabeçalho)

Para **cada seção** da espec, as duas perguntas:

1. *Esta frase descreve **o que o programa faz**, ou **como o autor o escreveu**?*
   A primeira fica; a segunda sai ou é reescrita como requisito funcional.
2. *De onde veio cada número?* Fórmula (cite o paper) · medição (cite o dump/fixture) ·
   decisão nossa (diga-o). Número sem proveniência não entra — a casa já proíbe isso
   para caps (§0.0); aqui a mesma lei protege juridicamente.

E, por fim, o instrumento: `bash scripts/cleanroom-sweep.sh <vassoura> <espec>` —
**verde** é condição de entrega. Depois vem a auditoria independente de R-pré (§3.R):
autofiltragem não é auditoria.

---

## §5 — O oráculo fora da árvore (o gabarito que dá a velocidade)

- **O fonte e o binário do alvo moram fora do repo** (`~/Referencias/<alvo>/`). As
  **saídas** dele — malhas, dumps, goldens — são **dados** (§1.1) e **podem entrar no
  repo** como fixtures, com README dizendo de onde vieram e como regenerá-las.
- ⛔ **A proveniência da ENTRADA decide a da saída:** os inputs das fixtures são
  **nossos, gerados por nós, ou de licença livre verificada** — **nunca** os assets de
  exemplo empacotados com o alvo (malha/imagem/preset de exemplo é obra plena, §1.5.3;
  a saída computada sobre ela herda os direitos dela). O README registra a proveniência
  da entrada E da saída. ⚠️ Fixture que precise de input do alvo para existir vive fora
  da árvore, junto do oráculo.
- **E pode e deve instrumentar o oráculo à vontade** (a licença concede modificação
  privada — §1.1; AGPL: local, sempre): dumps de fase, flags de trace, contadores. A
  versão instrumentada também fica fora da árvore.
- ⚠️ **Dump é dado; texto do programa é programa.** Mensagens longas, templates, shader
  embutido, help vazando num dump são código — E filtra e **renomeia chaves/tags para o
  domínio** antes de publicar. Execução do oráculo é ato de E (ou wrapper de E que
  entrega só dados) — a janela consome dumps prontos.
- **Paridade é o gate, e a barra é DERIVADA** — do formato (`rgba16float ⇒ 2⁻¹¹`), da
  física, da precisão de `f32` — nunca um epsilon de conforto (lei da casa, CLAUDE.md
  §0.0). O harness de paridade é escrito por **I** (consome a espec + dumps como dados)
  ou já existe (`ph2d-quadbench` compara fase a fase hoje — ⚠️ I usa os dumps de `ref/`
  pelos caminhos da espec; a pasta contém `oracle/`, que é ⛔).
- ⚠️ **Comparar fase a fase é mais forte que comparar o fim** — achado medido da casa
  ([PLAN.md §4-duotricies](../3D/quad-remesh/PLAN.md)): o oráculo grava as fases
  intermediárias, e cada fase nossa pode ser cobrada contra a dele **na malha dele**.
- ⚠️ **Paridade bit-a-bit: depende do degrau.** Em porte T0 é meta legítima (o sculpt
  fecha a 1 ULP do SculptGL — MIT, sem parede). **No pipeline T2 o precedente da casa
  deliberadamente NÃO a promete** — o ADR-0162 a recusa por ser juridicamente
  indesejável num clean-room, e essa cerca fica: a meta padrão é a barra derivada.
  Bit-parity com alvo copyleft só com decisão registrada no ledger, nomeando por que o
  ADR-0162 não se aplica ao caso. (Comportamento igual continua lícito — §1.3; a cerca
  é sobre a *narrativa probatória*, não sobre a lei.)

---

## §6 — O LEDGER de proveniência + protocolo de incidente

Arquivo: `docs/<Módulo>/cleanroom/LEDGER_<alvo>.md` — **nasce antes da primeira leitura
do alvo** e fecha com a assinatura do R. ⚠️ **A janela (I) nunca abre o ledger** (ele
carrega rastros do alvo de propósito); o canal dela para o ledger é o **inbox**: append
cego (`cat >>`, que não lê) em `cleanroom/INBOX_<alvo>.md` — um subagente E/R
transcreve. Conteúdo mínimo do ledger:

```
Alvo: <nome, repo, versão/commit, licença (texto da concessão relevante)>
Degrau da triagem (§2): T_ e por quê (o que foi buscado em T1, com datas)
Patente (§8.1): buscado em <data>, termos usados, resultado
EULA (se T4): cláusulas relevantes transcritas, veredito
Papel E: id(s)/datas dos subagentes-E e o que cada um leu; transcripts =
  zona contaminada (⛔ a janela não os lê — §3.I)
Cobertura da travessia (§3.E): áreas/arquivos do fonte percorridos, com datas
Corrente I: session-id de CADA janela da linha, com datas e o motivo de cada
  troca (BLOCO-RETOMADA), e a DECLARAÇÃO por janela: "nenhum conteúdo do
  fonte do alvo entrou no CONTEXTO desta janela (incluindo reports de
  subagentes e compactação); exposição via pesos do modelo não é atestável
  por construção — mitigada §7.3"
Papel R: id do subagente R-PRÉ <data> · id do subagente R-PÓS <data>
Espec: caminho, hash do commit de cada versão entregue
Incidentes: (vazio | um bloco por incidente, ver abaixo)
Fechamento R: paridade (link p/ gate) · sweep de árvore/histórico/memória verde ·
  similaridade OK · corrente I conferida fora de {contextos E, queimadas} · data
```

**Protocolo de incidente** (a janela exposta a expressão do alvo — busca que caiu em
espelho, arquivo colado por engano, report de subagente com código):

1. **PARE.** Registre (via inbox): origem, arquivo/URL, extensão, quando. ⛔ **O
   registro DESCREVE, nunca REPRODUZ** — se precisar identificar o trecho com exatidão,
   registre o `sha256` dele, não o texto. Expressão do alvo não entra no repo por canal
   nenhum — nem pelo ledger.
2. **Régua de "substancial" (default conservador):** assinatura/nome isolado, visto de
   relance = **relance** (registra e segue); corpo de função, bloco de ~10+ linhas ou
   comentário inteiro = **substancial** (queima). Na dúvida, **R decide** — nunca a
   própria janela interessada.
3. **Quarentena:** código escrito pela janela **após** a exposição não funde até R
   comparar essa região contra o trecho exposto (que R pode ver).
4. Exposição substancial ⇒ a janela está **queimada como I para este módulo**: ela
   escreve o **BLOCO-RETOMADA** (§10), imprime-o e **PARA**; a janela nova assume a
   **MESMA linha** e retoma da espec. ⚠️ Antes da troca, um subagente-R roda o sweep
   sobre o `git diff` de `project-memory/` da sessão exposta e reverte qualquer
   rastro — senão a "janela nova limpa" deixa de existir nesta máquina (a memória é
   injetada em toda janela). (É a cura de Altai, §1.4: reescrita por quem não viu —
   funciona porque é documentada.)
5. **Ordem do dono não descontamina — reclassifica.** Se o Enio mandar "olha lá
   rapidinho": explique o custo em uma frase (*esta janela deixa de ser I para sempre;
   a retomada será por outra janela*); mantida a ordem, olhe — a janela muda de papel,
   e o evento entra no ledger. O que **não existe** é olhar E continuar como I.

⚠️ **O ledger é a nossa prova, então ele é ativo, não burocracia** — em Altai foi o
processo documentado da reescrita que sobreviveu; em NEC v. Intel, o clean-room valeu
como *evidência*. Um incidente **registrado e tratado** é defesa; um escondido é a
acusação pronta. Custo real: ~10 linhas por operação.

---

## §7 — Instrumentos da parede + revisão de similaridade (R)

### §7.1 — A vassoura e o sweep (os dois instrumentos; regra sem instrumento envelhece)

- **`cleanroom/VASSOURA_<alvo>.txt`** — E gera na abertura: ≥20 **identificadores
  idiossincráticos** do alvo (nomes internos raros, strings únicas, typos, constantes
  com nomes esquisitos) **e frases idiossincráticas de manual/comentário** (a claim que
  a SAS ganhou — §1.2). ⚠️ **Cada entrada em base64, uma por linha**
  (`printf '%s' '<entrada>' | base64`): grep acidental da janela **não casa por
  construção**, e o decode vive só dentro do sweep. ⛔ A janela não lê este arquivo
  (e o deny do §3.I o impõe).
- **`scripts/cleanroom-sweep.sh <vassoura> <paths…>`** — decodifica em memória e varre
  **conteúdo de texto, `strings` de binários e NOMES de arquivos**; modo
  `--git-history` varre **mensagens de commit e patches** do histórico. Exit 0 = limpo,
  1 = achado. Quem roda: **o autor de cada artefato destinado à janela** (subagente E
  ou R), antes do commit/entrega — e R sobre tudo, no fechamento. A própria janela
  roda-o sobre o BLOCO-RETOMADA (o script decodifica em memória; ela nunca vê a
  vassoura).

### §7.2 — O fechamento de R (modo PÓS)

1. **Paridade:** gates verdes; barra derivada; fase a fase onde houver dumps.
2. **Sweep total:** `cleanroom-sweep.sh` sobre (a) a árvore rastreada do produto
   (`git ls-files`), (b) `--git-history` (mensagens + patches, incluindo
   `-- docs/<Módulo>/cleanroom/` e `-- project-memory/` do período do módulo), (c) a
   linha do CLAUDE.md §5 e o handoff da linha. Com a vassoura codificada e a regra
   descreve-nunca-reproduz do §6, **zero hits sobre a árvore inteira é satisfazível —
   e é a barra.** Opcional e recomendado: sweep sobre os transcripts das janelas da
   corrente I (`~/.claude/projects/...jsonl`), para detectar exposição não-reportada.
3. **Revisão estrutural:** R lê os dois lados e procura convergência de **expressão**
   (não de comportamento — comportamento igual é o objetivo): mesma decomposição
   arbitrária em funções, mesma ordem não-forçada, mesmos truques de escrita, mesmos
   nomes traduzidos.
4. **Session-ids:** nenhuma janela da corrente I pertence a {contextos que leram o
   fonte, janelas queimadas} — e cada troca tem o motivo registrado no ledger.

### §7.3 — Convergência de treino (o risco que só LLM tem — reconhecido, detectado, curado)

O Implementador pode ter visto o alvo **no treino do modelo**, sem saber — a limpeza da
*sessão* não apaga os *pesos*, e o protocolo não finge o contrário (a declaração de I no
§6 é escopada ao que é atestável). As quatro camadas:

- (a) o §3.I manda escrever **no idioma desta casa** (nomes, formas, gates, tokens —
  que já é outro por construção) e **não tentar "lembrar"** de implementação nenhuma;
- (b) o **tripwire de recall** (§3.I): detalhe que a espec não deu e "veio" — não
  escreve, reporta;
- (c) a vassoura + a revisão estrutural são o detector post-hoc;
- (d) **a cura de um achado confirmado não é "tente de novo"** (janela nova roda os
  MESMOS pesos e pode convergir de novo): R prescreve uma **restrição estrutural
  funcional explícita** na re-derivação (*"decomponha por X em vez de Y"*) e/ou o ponto
  re-deriva em **modelo de outra família** — o ledger registra a rota e o re-sweep.

O risco não é zero; é **detectado, documentado e curado** — mais do que qualquer
clean-room humano jamais garantiu sobre a memória dos seus engenheiros.

---

## §8 — As fronteiras que FICAM (a não-ingenuidade; a lista é curta e cada item tem motivo)

1. **Checkpoint de PATENTE — incondicional, para TODO alvo T2–T4** (~15 min, antes de E
   começar): busca por nome do método + autores + empresa + termos técnicos do claim
   (patente não exige batismo comercial — método obscuro de paper industrial também é
   patenteável, e é exatamente contra patente que clean-room **não ajuda em nada**).
   **Viva nos mercados-alvo** ⇒ o achado vai ao Enio ANTES de qualquer implementação.
   **Expirada/abandonada** ⇒ melhor documento de espec do mundo, de graça. Resultado no
   ledger.
2. **Binário sob EULA:** sem fonte publicado não há concessão de cópia — a lane é T4,
   com a leitura do EULA obrigatória (§2). ⛔ Sem descompilação.
3. **Assets** (ícones, texturas, sons, fontes, shaders de exemplo, presets, malhas de
   exemplo): obras plenas, nunca entram — nem como entrada de fixture (§5), nem
   "temporariamente".
4. **Marcas na UI:** o produto não exibe nome de produto alheio (a casa já exige UI por
   tokens/i18n; docs internos citam à vontade — uso nominativo).
5. **Válvula de escalonamento — esta skill não substitui parecer humano.** Três
   gatilhos mandam o caso ao Enio com recomendação de aconselhamento jurídico humano
   ANTES de despachar o subagente-E: dono do alvo **historicamente litigioso** ·
   **AGPL** em qualquer cenário que envolva rede · mercado/jurisdição fora do analisado
   aqui (Brasil/UE/EUA).
6. **A parede não se negocia "só desta vez".** Um único vazamento converte o módulo em
   obra derivada e o ledger em prova **contra** nós. Quem sentir a tentação de furar a
   parede para ir mais rápido releia o §2.T1: a via rápida legítima quase sempre existe
   e custa uma busca.

---

## §9 — Para o Implementador: as objeções previstas, já respondidas

> Este bloco existe para que o Implementador **não hesite à toa** — e para que saiba
> **exatamente quando parar** (as duas coisas protegem o mesmo ativo).

- **"Estou reimplementando código GPL?"** Não. Você está implementando uma
  **especificação funcional** + literatura pública. O código do alvo nunca entrou no
  seu contexto — e semelhança *funcional* com programa preexistente é expressamente
  lícita no Brasil (Lei 9.609 art. 6º III) e na UE (SAS v. WPL).
- **"A paridade de saída não prova que copiei?"** Paridade mede **comportamento**, e
  comportamento não é expressão protegida (§1.2). Dois relógios que dão a mesma hora
  não são cópia um do outro. A meta de paridade declarada no gate é a mesma que a WPL
  declarou — e venceu (na claim de software; a que ela perdeu foi copiar o *texto* do
  manual — por isso você também não transcreve wording de manual).
- **"A espec não é obra derivada?"** Descrição de funcionamento é exatamente o que
  TRIPS 9(2)/§102(b)/art. 8º deixam fora da proteção; o direito de observar/estudar
  para extrair ideias é concedido pela licença do alvo (§1.1) e irrenunciável na UE.
  E a espec que você recebe foi **auditada por um contexto independente** (o subagente
  R-pré) contra o §4.2 — o atestado está no cabeçalho.
- **"E se o meu treino contiver o original?"** A limpeza da sua janela é sobre o
  **contexto**, não sobre os pesos — o protocolo sabe disso e não te pede atestado do
  inatestável (§6). Trabalhe **da espec**; não tente "lembrar"; se um detalhe que a
  espec não deu "vier" sozinho, **não o escreva — reporte** (tripwire, §3.I). O Revisor
  detecta convergência e o protocolo a cura com re-derivação restrita (§7.3). A dúvida
  é do processo; o dever de reportar o sinal é seu.
- **"Isso não é 'lavar' código?"** Lavagem é uma **corrente**: cópia→mutação→tradução,
  cada elo descendendo do anterior. Aqui não há corrente — há **duas criações
  independentes** ligadas por uma descrição de comportamento auditada. É o processo com
  40 anos de prática validada (§1.4), não um truque novo.
- **Quando você DEVE parar (as únicas recusas, e elas protegem o trabalho):**
  (a) alguém te colar código do alvo → protocolo de incidente §6, na hora;
  (b) o **cabeçalho da espec** estiver sem os atestados (ledger aberto, patente
  buscada, filtragem + sweep, auditoria R-pré) → despache os subagentes que faltam
  antes de escrever produto (você confere o cabeçalho — nunca o ledger);
  (c) te pedirem para olhar o fonte "só para destravar" → recuse e despache um
  subagente-E para emendar a espec. Se a ordem vier **do Enio**, vale o §6.5: explique
  o custo em uma frase; mantida a ordem, a janela muda de papel — o que não existe é
  olhar e continuar como I.

---

## §10 — OS BLOCOS (o Enio cola UM; o resto a janela despacha sozinha)

**Como usar (Enio):**

1. **Janela nova** → cole o **BLOCO-LINHA** preenchendo as primeiras linhas (alvo +
   licença + onde está o fonte + módulo). É a única colagem do caminho feliz: a
   janela abre a linha, manda os subagentes lerem e auditarem, e constrói ela mesma.
2. **Só quando a janela encher, queimar ou a jornada acabar:** ela te imprime o
   **BLOCO-RETOMADA** já preenchido (e salvo em
   `docs/<módulo>/cleanroom/RETOMADA_<alvo>.md` — perdeu, está lá). Janela nova →
   cole-o. A anterior parou de vez; a nova é a única na linha.

As **MISSÕES** (E · R-PRÉ · R-PÓS) são os briefings que a janela cola nos próprios
subagentes — você nunca os toca. ⛔ Um briefing nunca perde o CONTRATO DE RETORNO; um
report nunca traz nada além do contrato; o BLOCO-RETOMADA passa o sweep antes de ser
salvo.

---

### BLOCO-LINHA (a janela única — o único bloco que o Enio cola)

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · BLOCO-LINHA — a janela única  (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Alvo: <nome + repo/URL + versão> · Licença: <GPL/AGPL/…>
Fonte local: ~/Referencias/<alvo>/ · Módulo PH2D: <módulo>
Sua linha: line/<módulo>

Você é a ÚNICA janela desta feature: abre a linha, orquestra os
papéis E e R como SUBAGENTES e implementa VOCÊ MESMA. Por isso as
regras do Implementador (§3.I) valem para você DESDE JÁ: você NUNCA
abre o fonte do alvo — quem lê são os subagentes, cujo contexto é
isolado do seu por construção.

Leia INTEIRA: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
Depois, EM ORDEM:
1. ABRA A LINHA: execute a FASE 1 do
   docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md com
   $MODULO = <módulo> (as regras A–H dele valem a sessão inteira).
   NÃO pare no passo 9 dele: a tarefa é ESTE bloco. Linha já existe?
   Rota "linha reaberta" do passo 4 dele. Tier `constrained`? Não há
   linhas: árvore compartilhada, Modo C, o resto deste bloco igual.
2. PASSO 0 DO I (§3.I): crie .claude/settings.local.json com o deny
   de Read (~/Referencias/**, LEDGER_*, VASSOURA_*) · declare seu
   session-id por append cego no INBOX (§6):
   echo "I session: <id> <data>" >> docs/<Módulo>/cleanroom/INBOX_<alvo>.md
3. Despache o SUBAGENTE-E com a MISSÃO-E (§10) preenchida, verbatim.
   → Report "PATENTE VIVA"? PARE e reporte ao Enio.
   → Degrau T0/T0½ (porta permissiva)? O clean-room ACABOU: você
     mesma porta o código permissivo (seu contexto segue limpo),
     mantendo atribuição/cabeçalhos. Pule para o passo 7 ao fim.
4. Despache o SUBAGENTE R-PRÉ (MISSÃO-R-PRÉ). Achados → despache um
   subagente-E de emenda; repita até o atestado entrar no cabeçalho
   da espec.
5. IMPLEMENTE: confira o CABEÇALHO da espec (nunca o ledger) e
   construa sob o §3.I à risca — fontes permitidas, a lista ⛔, o
   tripwire de recall, o INBOX, e DIRETIVA_IMPLEMENTACAO.md a cada
   passo, como sempre. Trabalhe no idioma DESTA casa: nomes do
   domínio, formas do repo, tokens, gates; a decomposição em
   arquivos/funções é SUA, guiada pelas fases funcionais da espec.
   O gate de paridade (barra DERIVADA — bit-parity NÃO é a meta em
   T2, ADR-0162) é parte da entrega. Dúvida que a espec não
   responde → subagente-E de emenda. NUNCA vá olhar.
6. Paridade verde → despache o SUBAGENTE R-PÓS (MISSÃO-R-PÓS).
   Achado estrutural → re-derive com a restrição funcional dada
   (§7.3.d) e re-despache até "Ledger fechado".
7. FECHE A LINHA como qualquer outra (DIRETRIZ §1.5.9): gate batched
   + handoff de integração — que NÃO menciona mecanismo interno do
   alvo (só nome + link p/ cleanroom/). NÃO integre, NÃO pushe:
   ordem do Enio (§0.7).

SAÍDAS DE EMERGÊNCIA (as duas terminam no MESMO gesto):
- Report de subagente chegou com expressão do alvo → incidente §6:
  registre no INBOX (descreva, NUNCA reproduza), você está QUEIMADA
  como I.
- Contexto enchendo / fim de jornada → mesmo gesto, sem incidente.
O gesto: preencha o BLOCO-RETOMADA (§10), rode
bash scripts/cleanroom-sweep.sh sobre ele, salve em
docs/<Módulo>/cleanroom/RETOMADA_<alvo>.md, IMPRIMA-O inteiro no fim
da resposta e PARE. A janela seguinte assume a MESMA linha.
═══════════════════════════════════════════════════════════════════
```

### BLOCO-RETOMADA (janela nova assume a MESMA linha)

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · RETOMADA — a linha JÁ EXISTE  (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Alvo: <alvo> · Licença: <…> · Módulo: <módulo> · Linha: line/<módulo>
Espec: docs/<Módulo>/cleanroom/SPEC_<alvo>.md
Cabeçalho da espec: <estado dos atestados>
Parou no passo <n> do BLOCO-LINHA: <meia linha do estado>
Motivo da troca: <contexto cheio | incidente §6 | fim de jornada>

Você assume a linha E o papel de Implementador. A janela anterior
PAROU — se ela ainda responder, ignore-a: a linha é sua, e NUNCA há
duas janelas na mesma linha. Sua janela está limpa e TEM de
continuar limpa: você NUNCA abre o fonte do alvo (§3.I); E e R são
subagentes.

Leia INTEIRA: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
Depois, EM ORDEM:
1. FASE 0 do docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md
   (cd/pwd/branch — você começa na RAIZ, que é a árvore ERRADA;
   árvore suja = trabalho do anterior: commite, não descarte). As
   regras A–H do MODELO_ABERTURA_LINHA valem para você.
2. PASSO 0 DO I (§3.I): confira/crie o deny config · declare seu
   session-id no INBOX (append cego).
3. Motivo = incidente §6? O código escrito APÓS a exposição está em
   QUARENTENA (§6.3): não o toque até um R-pós compará-lo.
4. Retome o BLOCO-LINHA (§10) no passo em que a anterior parou.
═══════════════════════════════════════════════════════════════════
```

### MISSÃO-E (briefing que a janela cola no subagente-E)

```
MISSÃO-E · CLEAN-ROOM de <alvo> (licença <…>) · módulo <módulo>
Fonte local: ~/Referencias/<alvo>/ · Worktree: Worktrees/line-<módulo>/

Você é o SUBAGENTE-E (Especificador) — o ÚNICO contexto desta
feature que lê o fonte do alvo (a licença dele concede os atos
privados: leia-a e cite-a no ledger; AGPL ⇒ oráculo LOCAL, sempre).
A janela que te despachou é o IMPLEMENTADOR e tem de permanecer
limpa: NADA de expressão do alvo (código, nome interno, wording de
manual/comentário) pode voltar no seu report. Você NÃO escreve
código de produto (harness/bancada PODE — §3.E). TUDO do alvo —
fonte, builds, notas, RASCUNHOS — vive em ~/Referencias/<alvo>/;
nada dele entra no repo, em /tmp nem no scratchpad.
Leia na skill (docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md):
§2 · §3.E · §4 · §5 · §6 · §7.1 · §8.1.
Você trabalha DENTRO da worktree da linha. ⚠️ O settings.local.json
dela nega Read em ~/Referencias/** — a cerca é da janela-mãe: você
lê o fonte por shell (cat/rg).

Execute em ordem:
1. TRIAGEM (§2): leia a licença REAL; cace irmão permissivo (T0/T1,
   validando proveniência) ANTES de aceitar T2. Porta mais barata?
   PARE e reporte só o degrau — a janela-mãe porta ela mesma.
2. PATENTE (§8.1): busca incondicional; resultado no ledger.
   Patente viva? PARE e reporte "PATENTE VIVA" + o número.
3. LEDGER (§6) em docs/<Módulo>/cleanroom/LEDGER_<alvo>.md, ANTES
   da primeira leitura do fonte — seu id de agente + data.
4. VASSOURA (§7.1): ≥20 identificadores idiossincráticos + frases
   de manual/comentário, CADA ENTRADA EM BASE64, em
   cleanroom/VASSOURA_<alvo>.txt.
5. ORÁCULO (§5): binário + instrumentação FORA da árvore; dumps com
   chaves renomeadas para o DOMÍNIO; goldens como fixtures com
   proveniência de ENTRADA nossa/livre (nunca assets do alvo).
6. TRAVESSIA INTEGRAL (§3.E): leia o fonte INTEIRO, arquivo a
   arquivo, + a história (commits, issues, PRs, design docs,
   palestras). Cobertura no ledger. MINERE as dicas dos autores
   (§4.1.12): re-expressas com link; prosa crítica pode ser citada
   CURTA, entre aspas, com a fonte. A espec só nasce DEPOIS da
   travessia completa.
7. ESPEC (§4): rascunhe em ~/Referencias/<alvo>/draft/ pela regra
   do ARQUIVO FECHADO (§4.1.11); filtragem §4.3; commit ÚNICO na
   linha (scoped, docs-only, --no-verify) em
   cleanroom/SPEC_<alvo>.md com o CABEÇALHO completo (atestados,
   mapa de leitura da literatura, denylist de URLs). Rode
   doc-index.sh se o diretório do módulo for indexado.
8. SWEEP (§7.1): bash scripts/cleanroom-sweep.sh sobre a espec,
   todo artefato destinado à janela-mãe, E o texto do seu report
   final (grave-o em ~/Referencias/<alvo>/draft/, varra, só então
   devolva). Verde é condição de entrega.
CONTRATO DE RETORNO (⛔ violá-lo queima a janela-mãe — incidente §6):
seu report final é EXATAMENTE
  "Espec pronta em <path>. Ledger/vassoura/fixtures em <paths>.
   Degrau: T_. Cobertura: <N arquivos>. Sweep: verde."
+ no máximo 5 linhas de fatos FUNCIONAIS — zero identificador
interno, zero trecho, zero wording do alvo.
```

### MISSÃO-R-PRÉ (briefing que a janela cola no subagente R-pré)

```
MISSÃO-R-PRÉ · CLEAN-ROOM de <alvo> · módulo <módulo>
Espec: docs/<Módulo>/cleanroom/SPEC_<alvo>.md

Você é o SUBAGENTE R-PRÉ: contexto novo, independente do subagente
que escreveu a espec (autofiltragem não se audita). Você PODE ler
os dois lados (o fonte por shell — o deny da linha bloqueia Read);
NÃO escreve nem dita código de produto. Leia na skill: §3.R · §4.2
· §7.1.
1. Audite a espec contra o §4.2: pseudo-código espelhado, wording
   de manual, nomes internos, tabela verbatim, organização
   transcrita.
2. bash scripts/cleanroom-sweep.sh <vassoura> <espec e anexos>
3. Verde → ateste no cabeçalho da espec ("auditada contra §4.2 por
   R-pré em <data>") e registre o PRÉ no ledger.
CONTRATO DE RETORNO: "R-pré: verde, atestado no cabeçalho." OU
"R-pré: N achados." + cada achado como instrução FUNCIONAL de
reescrita — ⛔ zero trecho, zero wording, zero nome interno do alvo.
```

### MISSÃO-R-PÓS (briefing que a janela cola no subagente R-pós)

```
MISSÃO-R-PÓS · CLEAN-ROOM de <alvo> · módulo <módulo>
Ledger: docs/<Módulo>/cleanroom/LEDGER_<alvo>.md

Você é o SUBAGENTE R-PÓS: vê os dois lados (o fonte por shell — o
deny da linha bloqueia Read — e o nosso código); NÃO escreve nem
dita código de produto. Leia na skill: §6 · §7. Execute o §7.2:
1. PARIDADE: gates verdes, barra DERIVADA, fase a fase onde houver
   dumps.
2. SWEEP TOTAL: árvore rastreada (git ls-files) + --git-history
   (mensagens e patches, incl. cleanroom/ e project-memory/) +
   linha do CLAUDE.md §5 + handoff da linha. ZERO hits é a barra.
   Recomendado: sweep nos transcripts das janelas da corrente I.
3. REVISÃO ESTRUTURAL: convergência de EXPRESSÃO (decomposição
   arbitrária igual, ordem não-forçada, truques de escrita, nomes
   traduzidos) — comportamento igual NÃO é achado, é o objetivo.
4. INCIDENTES: cada um do INBOX transcrito e tratado; quarentena
   comparada contra o trecho exposto (régua do "substancial", §6.2).
5. CORRENTE I: nenhuma janela pertence a {contextos que leram o
   fonte, janelas queimadas}; cada troca com motivo no ledger.
6. Feche o ledger com o bloco de fechamento (§6).
CONTRATO DE RETORNO: "Ledger fechado. Módulo apto a integrar." OU a
lista de achados como RESTRIÇÕES FUNCIONAIS de re-derivação
(§7.3.d) — ⛔ zero trecho do alvo.
```

---

> **Nota final — por que esta diretriz pode ser agressiva:** cada coisa que ela manda
> fazer está apoiada em texto de licença, texto de lei ou precedente real (§1) — citados
> com as ressalvas que têm, porque um documento que exagera uma citação entrega ao
> adversário a réplica. E cada coisa que ela proíbe é exatamente o que faria o resto
> desabar. Não é prudência de advogado: é a configuração que **maximiza o que se pode
> tomar** — que é quase tudo — ao preço de **uma** disciplina: quem escreve nunca viu.
> **Nenhuma limitação sem âncora sobreviveu à revisão:** cada ⛔ deste documento cita
> a licença, o estatuto ou o caso que o exige — e o que nenhuma lei exigia foi ABERTO
> de propósito: a leitura integral do fonte pelo Especificador, a prosa pública do
> alvo para todos, o algoritmo inteiro em qualquer profundidade, a citação curta com
> fonte, os nomes públicos de interface, o harness pelo Especificador.
> A casa já paga preços maiores por leis menores.
> E a contagem de janelas é decisão **operacional**, não jurídica: a parede é entre
> CONTEXTOS, e uma janela que nunca viu o fonte continua limpa por mais subagentes
> que despache — o que a lei pede está no §0, não no número de abas abertas.

---

## Resumo prático (para o Enio — como usar este método)

**Quando usar:** você quer uma feature cujo melhor código existente é "gratuito, mas
quem usa tem que abrir o próprio código" (GPL e parentes) — e o PH2D vai continuar
fechado.

**O que você faz (1 janela, 1 colagem):**

1. **Janela nova** → cole o **BLOCO-LINHA** (§10) preenchendo as primeiras linhas
   (nome do programa-alvo, onde está o fonte, nosso módulo). Só isso. A janela abre a
   linha de trabalho, manda ajudantes internos lerem o código original INTEIRO e
   escreverem um manual de engenharia completo (com o programa original rodando de
   lado como gabarito), audita o manual, e constrói a nossa versão em Rust — com
   testes provando que dá as mesmas respostas. Tudo numa linha só, que se integra ao
   main de uma vez, como qualquer outra.
2. **Se a janela encher ou travar**, ela te entrega um bloco de **RETOMADA** pronto
   (impresso na resposta e salvo em `docs/<módulo>/cleanroom/RETOMADA_*.md`). Abra
   uma janela nova, cole, e a nova continua exatamente de onde a anterior parou — na
   mesma linha. A anterior para de vez: nunca duas janelas no mesmo trabalho.

⚠️ **Nunca peça à janela para "dar uma olhadinha" no código original** — quem olha são
os ajudantes internos, que têm memória separada da dela. Se ela olhar, tem de ser
trocada (ela explica o custo se você pedir).

**Por que é seguro:** quem escreveu o nosso código nunca viu o deles — e o diário
prova. **Por que é rápido:** o construtor não trabalha às cegas como no início do quad
remesh — ele recebe o manual completo e o gabarito rodando do lado, e o trabalho
inteiro mora numa linha só.

**Atalho que vale ouro:** antes de tudo, o ajudante-leitor confere se existe versão do
mesmo algoritmo com licença livre (§2, degraus T0/T1) — se existir, copiamos direto,
legalmente, e nada do resto é necessário.
