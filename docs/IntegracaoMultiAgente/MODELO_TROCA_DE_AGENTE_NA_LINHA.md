# MODELO — Troca de agente numa linha viva (e retomada pós-integração)

> **Fonte única** do bloco que o Enio cola quando um agente NOVO assume uma linha que
> **já existe** — troca de janela de contexto, ou retomada depois de a linha ter
> integrado ao main. Para abrir uma linha do ZERO, use
> [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md). (DIRETRIZ §1.5.1–§1.5.2)

## Por que este doc existe

O agente novo trabalha no **`main`** em vez da linha dele. Já aconteceu várias vezes, com
vários agentes. **Não é desatenção — é o desenho da bancada:**

1. Toda janela abre na **raiz do repo primário**, que está em `main`. É assim de propósito
   (uma pasta só, uma janela por agente).
2. **O mesmo path relativo existe nas duas árvores.** `crates/ph2d-foo/src/lib.rs` abre na
   raiz e na worktree. Editar o da raiz **compila, testa e commita** — sem erro, sem aviso.
3. O agente novo herda um handoff que nomeia arquivos por path relativo. Ele abre o path
   que lhe deram, a partir de onde está — e onde ele está é `main`.

A falha é **silenciosa e só aparece na integração**, quando a linha não tem o trabalho e o
primário tem commits que ninguém pediu. Por isso a defesa não pode ser "lembre-se": tem de
ser uma **verificação barata, feita antes de ler qualquer código, e repetível a qualquer
momento**.

---

## O BLOCO (copie daqui pra baixo; escreva o módulo SÓ na 1ª linha)

```
═══════════════════════════════════════════════════════════════════
TROCA DE AGENTE — linha JÁ EXISTENTE     (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você assume uma linha em andamento. Sua linha: line/<módulo>

O nome após "line/" é o SEU MÓDULO ($MODULO nos comandos: substitua
pelo nome literal — env não persiste entre chamadas de shell).
Sua branch:    line/$MODULO
Sua worktree:  Worktrees/line-$MODULO/   (ela JÁ EXISTE — não crie)

⛔ ANTES DE LER OU EDITAR QUALQUER ARQUIVO, execute a FASE 0.
   Você está começando na RAIZ do repo, que está em `main`. Os MESMOS
   paths relativos existem aqui e na sua worktree: abrir `crates/...`
   daqui edita a ÁRVORE ERRADA, e isso compila e commita sem um único
   erro. Ninguém descobre até a integração.

FASE 0 — ONDE VOCÊ ESTÁ (execute já, sem pedir confirmação):
1. cd Worktrees/line-$MODULO && pwd && git branch --show-current
      → pwd TEM de terminar em /Worktrees/line-$MODULO
      → a branch TEM de ser line/$MODULO
      Deu `main`, ou a pasta não existe? PARE e reporte ao Enio: ou o
      módulo está escrito errado, ou a linha nunca foi aberta (aí o
      bloco certo é o MODELO_ABERTURA_LINHA.md).
2. git log --oneline -5 && git status --short --ignored | grep -v target/
      → é daqui que o agente anterior parou. PRODUTO sujo (`M`, ficheiros
        novos de crates/docs) = trabalho não commitado dele: NÃO descarte,
        commite POR CAMINHO (`--no-verify`). Pastas de INSTRUMENTO dele
        (`.cauda-*`, `target/prova/`): NUNCA commite — leia-as.

FASE 1 — RETOMADA (obrigatória no início de CADA jornada, §1.5.2.3):
3. git cherry main HEAD | grep -c '^+'
      → 0: todo commit seu JÁ está no main — o integrador rebaseou a
        linha num `integ/*` e ela ficou com hashes velhos. Faça
        `git reset --keep main` e siga. ⛔ Um `git rebase main` aqui
        reaplicaria os commits velhos por cima dos que o integrador EMENDOU.
      → >0: `git range-diff main...HEAD`. São seus e ainda não integrados?
        Siga. Algum já está no main com outro patch? PARE e reporte.
   git rebase main 2>&1 | tee target/rebase.log
        Conflito em Cargo.lock ou arquivo GERADO (registry-init,
        chrome/mod.rs): NUNCA na mão — regenere (DIRETRIZ §1.5.5).
        Conflito em código FORA dos seus arquivos = colisão de
        mesmo-símbolo: PARE e reporte ao Enio. E se o log disser
        `Solved` (Mergiraf): `git range-diff ORIG_HEAD...HEAD` ficheiro a
        ficheiro — ele larga a remoção de um lado numa lista e diz
        «Solved» (13/09, 2 de 130).
4. cargo check -p <sua crate principal>
      → confirma que a base nova não quebrou você. 1º build pode ser
        frio (minutos): é esperado, não investigue.

FASE 2 — ESTADO (leia, nesta ordem, DENTRO da worktree):
5. O handoff/tracker do SEU módulo (`ls -t docs/<Módulo>/handoffs/`, a
   linha do módulo no CLAUDE.md §5 e `git log --oneline
   $(git merge-base main HEAD)..HEAD`) — é onde o agente anterior deixou o que já foi
   decidido, medido e REPROVADO. Ler antes evita reconstruir o que já
   foi tentado e re-litigar decisão fechada.
6. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md — inteira, e
   RELEIA a cada passo, como ela manda.
6b. docs/IntegracaoMultiAgente/STACK_VERSOES.md — 1 pagina, gateada
   contra o Cargo.lock (as versoes NAO se copiam para aqui: a copia
   envelheceria sem gate) e as tres regras que um agente novo erra. Uma linha reaberta e' onde uma versao de
   memoria mais mente: o handoff que voce herdou pode ser anterior a'
   subida do stack.
7. As REGRAS PERMANENTES DA SESSÃO (A–I) do
   docs/IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md — elas valem
   IGUAIS para você. Não estão copiadas aqui de propósito: duas cópias
   da mesma regra divergem.
8. Reporte: "Assumi line/$MODULO em Worktrees/line-$MODULO (HEAD <sha>).
   <1 linha do estado>. Aguardo a tarefa." — e PARE.

COMO ESTA SESSÃO TERMINA:
   O fechamento é o da DIRETRIZ §1.5.9 e não muda por você ter assumido a
   linha no meio: gate batched 1× sobre o diff ACUMULADO, handoff de
   integração em docs/<Módulo>/handoffs/, e o item 7 — reclamar o
   `target/*/incremental` da worktree (25 GB, risco zero, o cargo recria).
   ⚠️ E o item 9, que é o ÚLTIMO passo de todos: deixar o binário do smoke
   JÁ COMPILADO na sua worktree — `cargo build -p ph2d-host-desktop
   --profile smoke` (+ features; `--release` só para smoke de PERFORMANCE),
   rodado 2× com a 2ª saída colada no handoff
   ("Finished" em segundos, zero "Compiling"). O Enio não espera build:
   nada no seu dia produz esse binário (o `check` não gera código, o gate
   é perfil `ci-test`), e uma edição posterior o invalida em SILÊNCIO.
   Você NÃO integra e NÃO pusha: entrega o handoff e PARA (CLAUDE.md §0.7).

REGRA DE OURO DESTA SESSÃO (além das A–I):
⛔ Na dúvida sobre onde você está, `pwd`. Antes de qualquer commit,
   `git branch --show-current`. Custa um segundo; a alternativa é
   descobrir na integração que o trabalho foi para o main.
   Se você JÁ escreveu no main sem querer: NÃO apague nada — reporte
   ao Enio e aponte o procedimento de resgate (doc deste bloco, §
   "Resgate").
═══════════════════════════════════════════════════════════════════
```

---

## Resgate — "escrevi no `main` sem querer"

Acontece. **Não apague nada**; o trabalho está inteiro, só está na árvore errada.
Reporte ao Enio e execute o caso que se aplica:

**Já commitou no `main` (local, não pushado):**

```bash
cd ~/Documentos/Projetos/PH2D        # primário
git branch --show-current            # confirme: main
git reflog main | head -20           # ache o último sha do main que NÃO é seu: <ANTES>
git log --oneline <ANTES>..main      # só os seus? confira UM A UM

# leva os commits para a linha, sem duplicá-los no main
git -C Worktrees/line-<módulo> cherry-pick <ANTES>..<sha-mais-novo>
git reset --keep <ANTES>             # ⚠️ só depois do cherry-pick VERDE
```

⛔ **Nunca `origin/main` como alvo.** Entre uma integração e o envio, o `main` local está **à
frente** do `origin` (137 commits em 13/09): `git log origin/main..HEAD` lista o trabalho do
integrador como «seu», e um `reset --hard origin/main` apaga a integração inteira.

**Ainda não commitou (mudanças soltas no `main`):**

```bash
cd ~/Documentos/Projetos/PH2D
git add -N -- <seus ficheiros NOVOS> # sem isto o diff não leva os ficheiros novos
git diff --binary > /tmp/resgate.patch   # inclua --cached se houver staged
git -C Worktrees/line-<módulo> apply /tmp/resgate.patch
git checkout -- <SÓ os seus arquivos>   # NUNCA `git checkout .`
```

⚠️ **`git reset --hard` e `git checkout .` no primário destroem o trabalho de OUTRAS
linhas e sessões** que também vivem ali. Confira `git status` antes e restaure **apenas os
seus arquivos, por caminho**.

---

## Quando NÃO usar este bloco

| Situação | Use |
|---|---|
| Linha nova, do zero | [`MODELO_ABERTURA_LINHA.md`](MODELO_ABERTURA_LINHA.md) |
| Fundir linhas ao main | agente integrador (DIRETRIZ §1.5.3), por ordem do Enio |
| Encerrar linha morta | `MODELO_ABERTURA_LINHA.md` §"Encerrar uma linha" |
| Você é a janela do primário (setup/integração/ship) | não code em `main` — DIRETRIZ §1.5.8 |
