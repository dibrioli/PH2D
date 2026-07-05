# MODELO — Abertura de Linha Paralela (Modo L)

> **Fonte única** do bloco que o Enio cola na 1ª mensagem de cada sessão-de-linha
> (DIRETRIZ §1.5.8). O agente cria a própria worktree e prepara tudo; a tarefa vem depois.

## Como usar (Enio — 4 passos, sempre iguais)

1. Abra uma **janela nova** do VSCode/Claude **na pasta do repo** (`~/Documentos/Projetos/PH2D`
   — sempre a mesma; uma janela por agente).
2. Copie o bloco abaixo inteiro e troque **só** `<módulo>` (1 palavra, kebab-case curto —
   ex.: `grayscale`, `painter`, `vector`, `foundational`). Aparece em 3 lugares; buscar-e-
   substituir resolve.
3. Cole como **1ª mensagem** da sessão. O agente faz o setup sozinho e responde
   **"Linha pronta. Aguardo a tarefa."**
4. Mande a tarefa na mensagem seguinte (o que construir + em qual pasta `crates/...`).
   Docs/tracker do módulo nascem depois, dentro da própria worktree.

**Nunca** abra duas linhas pro mesmo módulo. Pra fechar uma linha que terminou de vez:
peça ao agente "encerre a linha" (ele roda `git worktree remove` + `git branch -d` após
a integração).

---

## O BLOCO (copie daqui pra baixo, troque `<módulo>`)

```
═══════════════════════════════════════════════════════════════════
ABERTURA DE LINHA PARALELA — Modo L        (PH2D · DIRETRIZ §1.5)
═══════════════════════════════════════════════════════════════════
Você é um agente-de-linha. Sua linha: line/<módulo>
Sua worktree (você vai criá-la agora): Worktrees/line-<módulo>/

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
   git worktree add -b line/<módulo> Worktrees/line-<módulo> main
      → branch já existe (linha reaberta)? Então:
        git worktree add Worktrees/line-<módulo> line/<módulo>
        e em seguida, DENTRO dela: git rebase main
5. cd Worktrees/line-<módulo>
   git branch --show-current        # DEVE imprimir: line/<módulo>
6. cargo check -p ph2d-core
      → warm-up do target/ próprio desta worktree; o 1º build é frio
        (minutos). NÃO otimize/investigue a demora — é esperada.
7. Leia INTEIRAS (dentro da worktree):
      docs/IntegracaoMultiAgente/DIRETRIZ.md            → §0, §1.5, §2, §6
      docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md  → tudo
        (e RELEIA a cada passo do trabalho, como ela manda)
8. Reporte: "Linha line/<módulo> pronta em Worktrees/line-<módulo>.
   Aguardo a tarefa." — e PARE. A tarefa vem na próxima mensagem.

REGRAS PERMANENTES DA SESSÃO (valem até o fim, sem exceção):
A. TODO read/edit/git/cargo acontece DENTRO de
   Worktrees/line-<módulo>/. A raiz do repo é o checkout primário
   compartilhado: o MESMO path relativo existe nas duas árvores —
   editar crates/... na raiz é editar a árvore ERRADA. Na dúvida,
   `pwd` antes de editar.
B. Edite só a(s) pasta(s) do seu módulo (nomeadas na tarefa).
   Precisou de QUALQUER coisa fora (foundational, contrato congelado,
   shell, outra crate)? PARE e reporte ao Enio — vai pra
   line/foundational (DIRETRIZ §1.5.4). Nunca negocie com outra linha.
C. Commits locais frequentes: git commit --no-verify (fast mode).
   NUNCA push. NUNCA --force. NUNCA git add -A.
D. git rebase main no início de cada jornada e antes de integrar.
   Conflito em Cargo.lock ou arquivo GERADO (registry-init): NUNCA
   resolva na mão — regenere (DIRETRIZ §1.5.5). Conflito em código
   fora da sua pasta = você violou a regra B.
E. Fechamento do módulo = gate batched (DIRETRIZ §6.6.A.2: nextest-
   impacted + clippy --all-targets + audit ≥2 lentes + DIRETIVA §3-§5)
   e SÓ ENTÃO a integração (DIRETRIZ §1.5.3):
       git rebase main
       cargo run -p ph2d-tool-sync && cargo run -p ph2d-node-sync
       cargo test -p ph2d-tool-registry-init -p ph2d-node-registry-init
       cargo test -p <suas crates>
       git -C ../.. merge --ff-only line/<módulo>
   --ff-only falhou = outra linha integrou antes de você → repita
   desde o rebase. Módulo verde que não integrou NÃO fechou.
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
