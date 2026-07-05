# MODELO — Abertura de Linha Paralela (Modo L)

> **Fonte única** do bloco que o Enio cola na 1ª mensagem de cada sessão-de-linha
> (DIRETRIZ §1.5.8). O agente cria a própria worktree e prepara tudo; a tarefa vem depois.

## Como usar (Enio — 4 passos, sempre iguais)

1. Abra uma **janela nova** do VSCode/Claude **na pasta do repo** (`~/Documentos/Projetos/PH2D`
   — sempre a mesma; uma janela por agente).
2. Copie o bloco abaixo inteiro e escreva o nome do módulo **UMA vez só**, na 1ª linha
   (`Sua linha: line/…` — 1 palavra, kebab-case curto: ex. `grayscale`, `painter`,
   `vector`, `foundational`). O resto do bloco se refere a ele como **"o novo módulo"**
   (`$MODULO` nos comandos) — não precisa trocar mais nada.
3. Cole como **1ª mensagem** da sessão. O agente faz o setup sozinho e responde
   **"Linha pronta. Aguardo a tarefa."**
4. Mande a tarefa na mensagem seguinte (o que construir + em qual pasta `crates/...`).
   Docs/tracker do módulo nascem depois, dentro da própria worktree.

**Nunca** abra duas linhas pro mesmo módulo. Pra fechar uma linha que terminou de vez:
peça ao agente "encerre a linha" (ele roda `git worktree remove` + `git branch -d` após
a integração).

---

## O BLOCO (copie daqui pra baixo; escreva o módulo SÓ na 1ª linha)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/<módulo>

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
   git worktree add -b line/$MODULO Worktrees/line-$MODULO main
      → a branch do novo módulo já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-$MODULO line/$MODULO
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-$MODULO
   git branch --show-current        # DEVE imprimir a sua branch
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
   (ph2d-core/editor-core/tokens/host/…) É PERMITIDO sob o protocolo
   testado (ADR-0107): a integração roda scripts/foundational-integrate.sh
   (gate da árvore combinada) e o Mergiraf funde o resíduo textual. PARE
   e reporte ao Enio SÓ se: (a) for contrato congelado (§4, exige ADR),
   ou (b) o rebase conflitar em código FORA dos seus arquivos (colisão de
   mesmo-símbolo com outra linha). Nunca negocie com outra linha.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5). Conflito em código
   fora da sua pasta = você violou a regra B.
E. Fechamento do módulo = gate batched (DIRETRIZ §6.6.A.2: nextest-
   impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3-§5)
   e SÓ ENTÃO a integração (DIRETRIZ §1.5.3) — UM comando:
       bash scripts/foundational-integrate.sh
   Ele faz: rebase main → re-sync (tool/node) → staleness → gate da
   árvore COMBINADA (cargo check --workspace se a linha tocou
   foundational; senão -p das crates mudadas) → nextest-impacted →
   merge --ff-only no primário. Aborta com a orientação certa em cada
   falha. --ff-only falhou = outra linha integrou antes → só RE-RODE o
   script (rebase+retesta). Módulo verde que não integrou NÃO fechou.
F. Ship (ship.sh + push + babysit CI) SÓ se o Enio disser que você
   fecha a ÚLTIMA integração da jornada (DIRETRIZ §1.5.4 + §8).
G. UI canônica sempre: zero hex, zero f32 literal de UI, tudo por
   tokens/i18n (CLAUDE.md §0.3). Contratos congelados (CLAUDE.md §6)
   são intocáveis nesta linha.
═══════════════════════════════════════════════════════════════════
```

---

## Encerrar uma linha (quando o módulo morreu de vez, pós-integração)

```bash
cd ~/Documentos/Projetos/PH2D          # raiz (ou git -C ../.. de dentro dela)
git worktree remove Worktrees/line-<módulo>
git branch -d line/<módulo>            # -d só passa se tudo foi integrado
```

Linha que continua na próxima jornada **não precisa** disso — fica aberta; o agente
seguinte usa o mesmo bloco (o passo 4 tem a rota "linha reaberta").
