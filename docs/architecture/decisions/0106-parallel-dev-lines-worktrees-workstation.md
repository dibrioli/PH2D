# ADR-0106 — Linhas de desenvolvimento paralelas via `git worktree` no tier `workstation` (Modo L)

- **Status:** Accepted (Enio, 2026-07-05)
- **Contexto arquitetural:** estende [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md)
  (paralelismo multi-agente por desacoplamento ECS/drop-crate) e [ADR-0104](0104-hardware-tiered-speed-strategy.md)
  (estratégia de velocidade em função do hardware). Operacionaliza: DIRETRIZ v8.0 §1.5 +
  [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md).

## Contexto

O modelo v7.1 (1 Coordenador único + N Implementadores num **shared tree**) nasceu no tier
`constrained` (Mac mini 8 GiB): N checkouts com N `target/` não cabiam em RAM/disco, então
todos os agentes dividiam um working tree e um índice git — e a DIRETRIZ §7 inteira legisla
as colisões resultantes (stash envenenado, reset alheio, commit fundido, `fmt -p` sobre WIP
alheio), com o Coordenador como árbitro de posse e gargalo de ship.

Com o desktop Linux 128 GB (tier `workstation`, ADR-0104), o custo que justificava o shared
tree sumiu. O requisito do Enio: **≥3 linhas de desenvolvimento paralelas em módulos
diferentes sem Coordenador atuando como gargalo**, preservando a operação `constrained`
porque o projeto continua indo ao Mac para smoke/hotfix.

## Decisão

**O modo de operação multi-agente passa a ser função do hardware** (`scripts/hw-profile.sh`):

1. **Modo L** (tier `workstation`): N linhas = N branches `line/<módulo>` + N `git worktree`
   em `Worktrees/line-<módulo>/` **dentro do repo** (gitignorado via `/Worktrees/`), uma
   sessão por linha, todas as janelas abertas na raiz do repo primário. O próprio agente
   cria sua worktree na 1ª mensagem, guiado pelo bloco único de `MODELO_ABERTURA_LINHA.md`.
   - **Colisão de git extinta por construção:** cada linha tem índice, HEAD, working tree e
     `target/` próprios. Colisão de **merge** continua prevenida pelo que já existia:
     isolamento físico de pasta (drop-crates, ADR-0031/0040) + wiring por codegen.
   - **Integração self-service serializada por `merge --ff-only`** no checkout primário
     (sempre `main`, sempre limpo): rebase → re-sync → gates → ff-merge; falha do ff = outra
     linha integrou antes → rebase e repete. Race auto-detectada, sem árbitro humano.
   - **Coordenador deixa de ser papel de plantão:** (a) arbitragem de posse — extinta;
     (b) foundational/contratos — continuam seriais por natureza, viram `line/foundational`
     com prioridade de merge (contrato congelado segue exigindo ADR, DIRETRIZ §4);
     (c) ship/push/babysit — 1× por jornada, executado por quem fecha a última integração.
2. **Modo C** (tier `constrained`): o modelo v7.1 **preservado intacto** (Coordenador único +
   N Implementadores, shared tree, DIRETRIZ §1.1–1.4 + §7). É o modo das sessões de
   smoke/hotfix no Mac; hotfix de Mac é sempre sobre `main` e volta por push — branches
   `line/*` não viajam para o Mac.

Conflitos de rebase/merge legítimos são apenas os enumerados (DIRETRIZ §1.5.5): `Cargo.lock`
e `*-registry-init/` **regenerados, nunca resolvidos à mão**; `icons.rs` (IconId) resolvido
por união alfabética (gate `enum_order_matches_svgs` confirma).

## Alternativas rejeitadas

- **Branches num checkout único:** não paraleliza — um working tree tem uma branch em
  checkout por vez; as sessões continuariam dividindo índice e arquivos.
- **Clones separados:** estritamente pior que worktree — objetos duplicados, branches
  invisíveis entre clones (exigiria push interno), setup por máquina.
- **Manter shared tree no `workstation`:** o histórico de incidentes (memórias git &
  colisão) é o argumento; o único benefício (economia de RAM/disco) não existe neste tier.
- **Modo L universal (inclusive Mac):** N worktrees × `target/` frios não cabem em 8 GiB;
  o Mac é sessão curta de smoke/hotfix, onde o shared tree simples é adequado.

## Consequências

- CLAUDE.md §0 (itens 2/4/5/7), §1 e §3 tornaram-se mode-aware; DIRETRIZ v8.0 ganhou §1.5;
  DIRETIVA ganhou o check de modo + "fechado inclui integração"; SKILL_Stack atualizado.
- `/Worktrees/` gitignorado; lint tools que honram `.gitignore` (typos, machete) ignoram as
  worktrees aninhadas; `cargo` do primário não desce nelas (workspace glob é relativo à raiz).
- Slots CoW (`slot-env.sh`/`slot-seed.sh`) ficam **Modo C only** — no Modo L cada worktree
  já tem `target/` próprio.
- Risco residual documentado: agente de linha editar o path relativo homônimo na **raiz**
  (árvore errada) — mitigado pela regra A do MODELO (todo trabalho dentro da worktree,
  `pwd` na dúvida) e pelo sanity `git branch --show-current`.
- Rollback: se o Modo L falhar na prática, voltar ao v7.1 é remover §1.5 e o MODELO —
  nenhum contrato de código foi tocado.
