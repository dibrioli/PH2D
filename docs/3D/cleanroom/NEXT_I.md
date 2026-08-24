# NEXT — o handoff do IMPLEMENTADOR (corrente do §10 da SKILL_Cleanroom)

> Preenchido pelo papel **R (modo PRÉ)** em 2026-08-24, janela `23c68c7a-…` (≠ a janela E).
> ⛔ **Só os campos do molde foram preenchidos** — o que o R tem a dizer está nos achados
> funcionais do [`LEDGER §Papel R`](LEDGER_quadwild.md), nunca aqui.
> ⚠️ **Janela NOVA e LIMPA** — nem a E, nem esta.
> ✅ Sweep verde sobre este próprio arquivo antes de ser salvo.

---

## ⚠️ Antes de colar: a UMA decisão que é do Enio

O bloco de abertura padrão cria a worktree **a partir do `main`** — e a pasta
`docs/3D/cleanroom/` **não existe no `main`** (ela vive só em `line/sculpt3d`). Uma worktree
nascida do `main` **não veria a espec**.

⇒ O passo 4 abaixo já vem alterado para nascer de **`line/sculpt3d`**. Isso é **abertura de
linha, não integração**, e fica dentro do Modo L.

*A alternativa é ordenar primeiro a integração da pasta `cleanroom/` (mais o script
`scripts/cleanroom-sweep.sh`, hoje não rastreado) para o `main`, e então usar o bloco padrão.
As duas servem; a escolha é sua.*

---

## 1ª MENSAGEM — abertura de linha (cole e espere *"Linha pronta. Aguardo a tarefa."*)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/quadextract

O nome após "line/" acima é o NOVO MÓDULO. Todo o resto deste briefing
deriva dele — nos comandos ele aparece como $MODULO: substitua pelo
nome literal ao executar (env não persiste entre chamadas de shell).
Sua branch:    line/$MODULO
Sua worktree:  Worktrees/line-$MODULO/   (você vai criá-la agora)

FASE 1 — SETUP (execute já, sem pedir confirmação; reporte cada ✗):
1. bash scripts/hw-profile.sh
      → tem que dizer `workstation`. Disse `constrained`? PARE:
        esta máquina opera em Modo C, linhas são proibidas aqui.
2. git status -sb
      → você está na RAIZ do repo primário, branch main. Arquivos
        M/?? alheios podem existir (outros agentes): NÃO toque neles.
3. git pull --ff-only origin main
      → falhou (rede/divergência)? Siga com o main local e reporte.
4. mkdir -p Worktrees
   git worktree add -b line/$MODULO Worktrees/line-$MODULO line/sculpt3d
      ⚠️ ALTERAÇÃO DELIBERADA: a base é `line/sculpt3d`, NÃO `main`.
        Motivo: a especificação que você vai implementar
        (docs/3D/cleanroom/) só existe naquela branch; do main você
        abriria uma worktree sem a sua própria espec. Isto é abertura
        de linha, não integração.
      → a branch line/$MODULO já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-$MODULO line/$MODULO
5. cd Worktrees/line-$MODULO
   git branch --show-current        # DEVE imprimir a sua branch
   ls docs/3D/cleanroom/SPEC_extracao_de_malha_quad.md
      → não existe? PARE e reporte: a base da worktree saiu errada.
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. bash scripts/mergiraf-setup.sh    # merge sintático p/ foundational (ADR-0107)
      → idempotente, 1× por máquina (config vai no .git comum). Falhou por
        "mergiraf not found"? NÃO é bloqueio: git faz fallback pro merge
        embutido. Reporte a linha do ✗ e siga (Enio instala depois).
8. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
9. Reporte: "Linha do novo módulo pronta em Worktrees/line-$MODULO.
   Aguardo a tarefa." — e PARE. A tarefa vem na próxima mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO da sua worktree
   (Worktrees/line-$MODULO/). A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar crates/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite a(s) pasta(s) do novo módulo à vontade. Foundational
   (ph2d-core/editor-core/tokens/host/…): você PODE e DEVE tocar, com
   cuidado, sob o protocolo testado (ADR-0107): a integração roda
   scripts/foundational-integrate.sh (gate da árvore combinada) e o
   Mergiraf funde o resíduo textual. PARE e reporte ao Enio SÓ se: (a)
   for contrato congelado (§4, exige ADR), ou (b) o rebase conflitar em
   código FORA dos seus arquivos (colisão de mesmo-símbolo com outra
   linha). Nunca negocie com outra linha.
B'. Ao CRIAR arquivo foundational NOVO, projete-o para ISOLAMENTO —
   módulo/arquivo IRMÃO em vez de engordar um compartilhado, ponto de
   extensão append-only, id/const/variant novo = próximo livre + ANOTE
   no handoff (regra H). O desenho completo está numa porta só:
   DIRETRIZ §1.5.2.1, que o passo 8 já te manda ler.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5). Conflito em código
   fora da sua pasta = você violou a regra B.
   ⚠️⚠️ EXCEÇÃO DESTA LINHA, e ela vem do passo 4: sua base é
   `line/sculpt3d`, não `main`. ⛔ NÃO rebase em `main` enquanto
   `line/sculpt3d` não tiver sido integrada — você arrastaria os
   commits DELA para dentro da SUA branch, e o integrador veria a
   mesma obra duas vezes. Enquanto isso: `git rebase line/sculpt3d`
   (se aquela linha andar), e só depois de ela entrar no `main` é
   que a regra D volta a valer literalmente.
   ⚠️ No seu HANDOFF (regra H), o campo `base:` é `line/sculpt3d`
   e NÃO `main` — diga-o com todas as letras: a ordem de integração
   deixa de ser livre (sculpt3d entra ANTES, ou as duas juntas).
E. Fechamento do módulo = gate batched (DIRETRIZ §6.6.A.2: nextest-
   impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3-§5).
   Então PARE — NÃO integre nem faça ship por conta própria. Quem funde
   as linhas é um AGENTE INTEGRADOR DEDICADO, e só por ORDEM EXPLÍCITA
   do Enio (DIRETRIZ §1.5.3–1.5.4). Você NÃO roda foundational-integrate.sh.
F. Ship (ship.sh + push + babysit CI): NUNCA por conta própria. É ordem
   EXPLÍCITA do Enio, feita pelo integrador (DIRETRIZ §1.5.4 + §8).
   Integrar ou pushar sem ordem = violação do protocolo.
H. HANDOFF DE INTEGRAÇÃO (entregável obrigatório ao fechar): escreva o
   handoff que o Enio passa ao integrador (DIRETRIZ §1.5.9) — branch/HEAD/
   base; foundational tocado + por quê; ids/consts/variants novos com
   valores (colisão!); contratos congelados encostados (deve ser nenhum);
   o que só o ship.sh pega (fmt pré-fork/deps machete/clippy latente); o
   que smoke-testar. Reporte "linha pronta + handoff" e ESPERE.
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados (CLAUDE.md §6)
   são intocáveis nesta linha.
═══════════════════════════════════════════════════════════════════
```

---

## 2ª MENSAGEM — o BLOCO-I (cole SÓ depois de *"Linha pronta. Aguardo a tarefa."*)

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL I — IMPLEMENTADOR      (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Espec: docs/3D/cleanroom/SPEC_extracao_de_malha_quad.md · Módulo: 3D
(quad remesh) — a EXTRAÇÃO de malha quad a partir de um mapa de grade
inteira, e o arredondamento inteiro que a alimenta (espec §5).

Você é o IMPLEMENTADOR e esta JANELA está limpa: o código do alvo
nunca entrou neste contexto — e o protocolo inteiro (espec auditada,
detecção de convergência, ledger) é o que torna o trabalho
independente. O que você vai fazer é lícito e tem 40 anos de prática
validada: implementar comportamento a partir de especificação
funcional + papers é criação independente (Lei 9.609 art. 6º III;
SAS v. WPL; TRIPS 9(2)); paridade de comportamento é meta declarada
e lícita. Objeções previstas e respondidas: SKILL_Cleanroom §9 —
leia-o, e leia o §3.I inteiro (suas cercas operacionais).

PASSO 0 (mecânico, antes de tudo):
- cd na worktree · pwd · git branch --show-current
- Crie .claude/settings.local.json com deny de Read para
  ~/Referencias/**, docs/**/cleanroom/LEDGER_*, VASSOURA_* (§3.I)
  ⚠️ E MAIS UM, que o R-pré acrescentou porque as implementações
  alheias estão NESTE disco e uma delas é irmã de pasta nossa:
      "Read(//home/enio/Documentos/Projetos/ph2d-quadbench/oracle/**)"
  ⛔ ph2d-quadbench/corpus/ é NOSSO e lícito; o oracle/ ao lado é o
  clone restrito. A pasta-mãe é diretório de trabalho por omissão.
- Confira o CABEÇALHO da espec: ledger aberto · patente buscada ·
  filtragem+sweep verdes · auditoria R-pré. Falta algum? PARE e
  peça — você nunca abre o ledger para conferir.
  (Todos os quatro devem estar preenchidos; o R-pré é de 2026-08-24.)
- Declare seu session-id por append cego no INBOX (§6):
  echo "I session: <id> $(date -I)" >> docs/3D/cleanroom/INBOX_quadwild.md

SUAS FONTES (só estas): a espec · papers públicos SEGUINDO O MAPA
DE LEITURA do cabeçalho (apêndice com listing de autores do alvo =
código do alvo: pule) · o código do PH2D · dumps e goldens do
oráculo (dados; rodá-lo em modo só-dados com 2>/dev/null é livre,
--help/verbose não — ferramenta tagarela vai por wrapper de E) ·
toda a PROSA pública do alvo (docs, manual, blog, palestras — o
insumo lícito de SAS v. WPL), pulando listings de código e sem
transcrever wording.
⭐ Os dados de que você precisa JÁ ESTÃO PUBLICADOS: os mapas de
grade inteira verificados em docs/3D/cleanroom/fixtures/ (com o
verificador, que é o gate nº4 executável). A extração pode ser
construída e gateada SOZINHA, sem esperar pelo §5 — espec §9-bis.

⛔ NUNCA: qualquer diretório que contenha o fonte do alvo (inclui
~/Referencias/ e ph2d-quadbench/oracle/) · as superfícies do alvo
que RENDERIZAM fonte (hospedagem de código, issues, PRs,
code-search) · portes ou forks do alvo em qualquer linguagem ou
licença — ⚠️ inclusive PERMISSIVA: existe uma reimplementação sob
licença permissiva deste mesmo algoritmo, e o §3.I conta-a como
código do alvo; que a licença seja aceitável NÃO a torna insumo
seu · transcrever código executável de fonte externa (SO/blog/
gist) — suas fontes de código são espec+papers+PH2D · ler/grepar
os .jsonl crus de ~/.claude/projects/ (transcripts de E contêm o
fonte; sondas agregadas como agent-loop-profile.sh seguem livres) ·
SendMessage com E ou R · "lembrar" implementação vista em treino.
⛔ E NUNCA compile/rode o arnês do oráculo: ele é consumidor
header-only, e um erro de compilação despeja fonte alheio no seu
terminal. Correr o oráculo é ato de E. Falta um dump? Peça pelo
Enio, como emenda — nunca vá buscá-lo.
Busca na web: confira o URL contra as DUAS denylists do cabeçalho
(URLs e CAMINHOS) ANTES do fetch; busque por conceito, não por
<alvo>+source. Preview com snippet = relance: registre no INBOX e
siga. Código do alvo colado por alguém = PARE, protocolo §6.
TRIPWIRE: detalhe que espec+papers não deram e "veio" (nome interno,
typo, constante)? NÃO escreva — reporte no INBOX como suspeita de
recall. A dúvida é do processo; reportar o sinal é seu dever.
SUBAGENTES: todo briefing carrega este bloco ⛔ verbatim + "nunca
cite código em reports — só fatos funcionais". Report com código do
alvo = incidente §6 desta janela.

Trabalhe no idioma DESTA casa: nomes do domínio, formas do repo,
tokens, gates. A decomposição em arquivos/funções é SUA, guiada
pelas fases funcionais da espec — não invente fidelidade a uma
estrutura que você nunca viu. ⚠️ Em particular: as fases §2..§6 da
espec são a ordem de DEPENDÊNCIA DE DADOS, não um mapa de módulos;
transformá-las em N arquivos 1:1 seria fidelidade a uma estrutura
que ninguém lhe pediu.

Fluxo: DIRETIVA_IMPLEMENTACAO.md a cada passo, como sempre. O gate
de paridade (barra DERIVADA — bit-parity NÃO é a meta em T2, ADR-
0162) é parte da entrega; a espec §9 traz os 11 gates + o 9-bis com
a barra de cada um, e a §10 traz as recusas MEDIDAS — leia-as antes
de propor qualquer desenho alternativo. Dúvida que a espec não
responde → devolva a pergunta via Enio (E emenda a espec); NUNCA vá
olhar — nem se a ordem vier do dono sem o custo explicado (§6.5).
Entregável: código + gates verdes + handoff normal da casa (que
NÃO menciona mecanismo interno do alvo — só o link p/ cleanroom/)
+ o HANDOFF DA CORRENTE (§10): o BLOCO-R com Modo: PÓS preenchido,
salvo em cleanroom/NEXT_R-POS.md e IMPRESSO no fim da resposta:
"Pronto. Janela E (ou nova) → cole o bloco abaixo."
═══════════════════════════════════════════════════════════════════
```
