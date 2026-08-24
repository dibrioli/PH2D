# Os BLOCOS prontos a colar — o que o Enio cola, e em que ordem

> ⭐ A `SKILL_Cleanroom` §10 define os blocos; este arquivo tem-nos **já preenchidos** para
> esta obra, para não haver um passo entre a decisão e a execução.
> ⚠️ **A ordem é obrigatória** e cada bloco vai numa **janela NOVA**.

---

## 1️⃣ AGORA — o R-pré (auditor da espec). ⚠️ Janela que **não** seja a que escreveu a espec.

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL R — REVISOR            (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Modo: PRÉ · Módulo: 3D (quad remesh) · Alvo: extração de malha quad
Ledger: docs/3D/cleanroom/LEDGER_quadwild.md

Você é o REVISOR: pode ver OS DOIS lados (o fonte do alvo e o nosso
código). Você NÃO escreve nem dita código de produto. Seus achados
voltam ao Implementador em termos FUNCIONAIS, nunca com trecho do
original, e nunca por mensagem direta — via emenda/handoff.
Modo PRÉ exige janela que NÃO seja a E (autofiltragem não se audita).

Leia: docs/_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md
§7 e §4.2.

Modo PRÉ (antes de o Implementador abrir):
1. Audite docs/3D/cleanroom/SPEC_extracao_de_malha_quad.md contra
   §4.2: pseudo-código espelhado, wording de manual, nomes internos,
   tabela verbatim, organização transcrita. Achado → E reescreve;
   verde → ateste no cabeçalho.
1-bis. ⭐ DECIDA a pergunta que o ledger deixa para você: o
   "INVENTÁRIO DE EXPOSIÇÃO, fase a fase". A janela E declarou-se
   queimada; o inventário mostra que o único T2 lido não é algoritmo
   e é de fase fora desta obra, enquanto a exposição REAL (o laço de
   arredondamento) é de um alvo T0½ e cobre só a SPEC §5/§5.1.
   ⛔ A decisão é SUA (§6.2: nunca a janela interessada). Registre-a
   no ledger, e diga o ESCOPO — não basta "queimada".
   ⚠️ Atenção particular: a espec foi escrita a partir de DOIS papers
   (QEx 2013, MIQ 2009) e o §5.1 foi CORRIGIDO contra o segundo.
   Confira que descreve COMPORTAMENTO, não a escrita deles.
2. Rode: bash scripts/cleanroom-sweep.sh \
     docs/3D/cleanroom/VASSOURA_quadwild.txt \
     docs/3D/cleanroom/ CLAUDE.md project-memory/
3. Confira o cabeçalho completo (§4) e registre o PRÉ no ledger.
═══════════════════════════════════════════════════════════════════
```

Espere: *"Espec auditada. Abra o Implementador."*

---

## 2️⃣ DEPOIS — o Implementador. ⚠️ Janela **NOVA**, em **duas** mensagens.

**1ª mensagem:** o bloco de [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md).
Espere *"Linha pronta. Aguardo a tarefa."*

**2ª mensagem:**

```
═══════════════════════════════════════════════════════════════════
CLEAN-ROOM · PAPEL I — IMPLEMENTADOR      (PH2D · SKILL_Cleanroom)
═══════════════════════════════════════════════════════════════════
Espec: docs/3D/cleanroom/SPEC_extracao_de_malha_quad.md
Módulo: 3D (quad remesh)

Você é o IMPLEMENTADOR e esta JANELA está limpa: o código do alvo
nunca entrou neste contexto — e o protocolo inteiro (espec auditada,
detecção de convergência, ledger) é o que torna o trabalho
independente. O que você vai fazer é lícito e tem 40 anos de prática
validada. Objeções previstas e respondidas: SKILL_Cleanroom §9 —
leia-o, e leia o §3.I inteiro (suas cercas operacionais).

PASSO 0 (mecânico, antes de tudo):
- cd na worktree · pwd · git branch --show-current
- Crie .claude/settings.local.json com deny de Read para
  ~/Referencias/**, docs/**/cleanroom/LEDGER_*, VASSOURA_*
- Confira o CABEÇALHO da espec: ledger aberto · patente buscada ·
  filtragem+sweep verdes · auditoria R-pré. Falta algum? PARE e
  peça — você nunca abre o ledger para conferir.
- Declare seu session-id por append CEGO no INBOX:
  echo "I session: <id> <data>" >> docs/3D/cleanroom/INBOX_quadwild.md

SUAS FONTES (só estas): a espec · os DOIS papers do mapa de leitura
do cabeçalho (URLs lá; ⚠️ o de 2013 é PDF de imagem — use
pdftotext -layout) · o código do PH2D · os FIXTURES em
docs/3D/cleanroom/fixtures/ (dados, com o verificador).

⛔ NUNCA: ~/Referencias/** · ph2d-quadbench/oracle/** · as
superfícies do alvo que RENDERIZAM fonte (hospedagem de código,
issues, PRs, code-search) · portes ou forks do alvo em qualquer
linguagem ou licença · transcrever código executável de fonte
externa (SO/blog/gist) · ler/grepar os .jsonl crus de
~/.claude/projects/ · SendMessage com E ou R · "lembrar"
implementação vista em treino.
Busca na web: confira o URL contra a DENYLIST do cabeçalho ANTES do
fetch. Busque por conceito, não por <alvo>+source.
TRIPWIRE: detalhe que espec+papers não deram e "veio" (nome interno,
typo, constante)? NÃO escreva — reporte no INBOX.
SUBAGENTES: todo briefing carrega este bloco ⛔ verbatim + "nunca
cite código em reports — só fatos funcionais".

A ORDEM DA OBRA (a espec explica porquê):
  A. §2-§6, a EXTRAÇÃO, contra os fixtures — ⭐ pode começar JÁ,
     não depende do §5.
  B. §5 + §5.1, o arredondamento inteiro, na ph2d-gridmap.
  C. A costura das duas + a FASE ZERO (§0) no caminho do produto.
⛔ Tudo novo shipa DESLIGADO, com a tabela da medição ao lado.

Fluxo: DIRETIVA_IMPLEMENTACAO.md a cada passo. Os 11 gates da espec
§9 (+ o 9-bis) são parte da entrega, com as barras DERIVADAS que ela
dá. Dúvida que a espec não responde → devolva a pergunta via Enio;
NUNCA vá olhar.
Entregável: código + gates verdes + handoff normal da casa (que NÃO
menciona mecanismo interno do alvo — só o link p/ cleanroom/).
═══════════════════════════════════════════════════════════════════
```

---

## ⚡ Alternativa — o **BLOCO-SOLO** (uma janela do início ao fim)

A skill ganhou um **Modo SOLO** (§3): uma janela **nasce** sob as regras do BLOCO-I, **nunca
abre o fonte**, delega E e R a **subagentes** (contexto isolado por construção) e implementa
ela mesma. O bloco está na skill, §10.

⚠️ **A recomendação desta linha é NÃO usá-lo aqui**, e o motivo está na própria skill:
*«SOLO serve a alvo pequeno/médio (um filtro, um algoritmo); obra grande, de dias, prefere
janelas separadas»*. **Esta obra é de dias** — a extração são seis fases com aritmética exacta,
mais o arredondamento, mais a costura.

⭐ **E aqui há uma economia que o SOLO não tem:** a espec **já está escrita e a exposição já
foi paga**. O SOLO gastaria um subagente-E a refazer o que existe.
⇒ **Use os blocos 1️⃣ e 2️⃣.** O SOLO fica registado como a rota certa para a *próxima* obra
pequena deste género.

---

## 3️⃣ NO FIM — o R-pós. Pode ser a janela que escreveu a espec.

Mesmo bloco do 1️⃣, com `Modo: PÓS`, e a lista de fecho da
[`SKILL_Cleanroom §7.2`](../../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).
