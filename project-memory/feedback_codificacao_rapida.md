---
name: feedback-codificacao-rapida
description: Não duplique pre-commit hook; cargo check só após ~1200 LOC editados; batch grande de commits (1 por Phase) em waves; delete > re-exports; doc canônico consolidado em docs/IntegracaoMultiAgente/DIRETRIZ.md §5.
metadata: 
  node_type: memory
  type: feedback
  originSessionId: ccf00d9f-668e-4155-aa63-3143cac9273c
---

# Codificação rápida — não duplicar trabalho do pre-commit hook

**Regra-mãe:** o pre-commit hook T2 (PH2D) já roda `cargo clippy
--workspace + cargo test --workspace`. Rodar manualmente antes do
commit é gastar o tempo duas vezes. Em 2 commits de Wave 8 Phase 1
+ 2.A.0, foram ~25-30min de validação duplicada antes de Enio
cobrar (2026-05-18 manhã + tarde).

**LOC threshold (v1.2, 2026-05-18 noite):** não rode `cargo check`
até ter editado/movido pelo menos **~1200 LOC novos/movidos** OU
completar uma operação lógica coesa (criar módulo inteiro, fechar
um Phase). 0-400 LOC = nada. 400-1200 = opcional se incerto. 1200+
= sane stop. (v1.1 era 600; v1.2 dobrou pra mais editing burst entre
validações — Enio cobrou velocidade.)

**Comandos certos por escopo:**

| Situação | Comando | Tempo |
|----------|---------|-------|
| Editou ≤400 LOC em 1 crate | nada — segue editando | 0s |
| Editou ≥1200 LOC OU módulo inteiro | `cargo check -p <crate>` | 3-15s |
| Quer rodar testes do crate | `cargo test -p <crate>` | 5-30s |
| Antes do commit | **nada** — hook | 0s |
| Fim do Wave (antes do push) | `cargo test --workspace` 1x | 3-5min |

**Commits em Waves (v1.1 revisão):** acumular MUITO mais que em
v1.0. Wave 8 Phase 2 (originalmente 5 sub-stages 2.A-F) pode ser
**1 commit único**. Cada hook custa 5-10min — 5 commits = 25-50min
de overhead. 1 commit = 5-10min. Trade-off de bisect é aceitável
porque Enio testa manualmente.

**Editing burst:** edit 5+ arquivos seguidos sem cargo entre.
`cargo check -p <crate>` UMA vez no fim. Fix all errors em 1 burst
novo. Compilador erra em batch também.

**Delete > back-compat re-exports:** quando move algo de path A
pra B, atualize todos os call sites direto. Re-exports custam +N
linhas + manutenção futura + disfarçam onde a verdade vive. Exceção:
API pública estável consumida por terceiros.

**Commit messages curtas (v1.1):** 1-3 parágrafos no body, não 30
linhas. Doc comments mid-refactor: NÃO escreva. Save closeout doc
para o fim do Wave.

**Reads cirúrgicos:** `grep -n` primeiro, `Read offset/limit` na
seção relevante. NUNCA re-Read arquivo que acabou de Edit. Parallel
reads quando independentes.

**Indicadores que está lento (revisado v1.1):**
- Rodando `cargo test` ou `cargo build --workspace` mais de 1x por commit.
- `cargo check` antes de ter editado 600 LOC.
- Comitando a mesma mudança lógica em 4+ sub-sub-stages.
- Validação ANTES do hook em vez de deixar o hook reportar.
- `Bash cat` quando deveria `Read`.
- Re-Read de arquivo que acabou de editar.
- Doc comment longo em commit que é refactor preserve-comportamento.

**ANTI-PADRÃO: `cargo test --workspace` no meio de sessão de correção
(2026-05-26):** Enio cobrou diretamente — "estamos no meio de
correções e vc faz testes extremamante demorados? Evite isso. Faça
testes rápidos e deixe que eu testo visualmente". Eu rodei
`cargo test --workspace --lib --tests` interpretando "continue" como
permissão de validação macro depois de tocar 3 arquivos foundational
(widget/scrollbar.rs, widget/mod.rs, dispatch/scroll.rs). ERRADO —
mesmo após editar foundational, basta `cargo test -p <crate-tocado>`
para cada crate; workspace test é só no ship (`./scripts/ship.sh`)
sob ordem explícita. Em sessão ativa de correção iterativa o Enio
testa visualmente. Compilou enquanto isso, gastando contexto e RAM.

Heurística: depois de uma sequência de fixes que compila + tem
`cargo test -p <crate>` verde nos crates tocados, o trabalho está
PRONTO pra smoke. Não tente uma validação "extra" pra ter certeza —
isso só atrasa. Se algo quebrou em outro crate downstream, o smoke
ou o ship pega.

**Doc canônico:** [`docs/IntegracaoMultiAgente/DIRETRIZ.md`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/docs/IntegracaoMultiAgente/DIRETRIZ.md) §5 (v6.1+).
Standalone `docs/DIRETRIZ_CODIFICACAO_RAPIDA.md` foi arquivado em
2026-05-19 — consolidado dentro da DIRETRIZ universal. CLAUDE.md
§"Cadência de validação" aponta pra lá. LLM lê antes de começar
refactor multi-arquivo.

**Cortes A+B no pre-commit T2 (2026-05-19 noite):** hook NÃO roda
mais `cargo test --doc --workspace` nem `clippy --all-targets` —
esses ficam pro CI. T2 multi-crate isolado agora escopa `nextest -p`
nos crates tocados (em vez de `--workspace`). Estimativa: T2 caiu de
~40min pra ~5min cache-frio, segundos cache-quente. **Implicação:**
doctest novo só é verificado em CI; quem cria doctest valida manual
com `cargo test --doc -p <crate>`. Vide [[project-perf-audit-2026-05-19]]
+ DIRETRIZ §5.4.

**Test slow é proibido (DIRETRIZ §5.6):** não use `TextSystem::new()`
em test (CoreText scan 25-77s) — use `TextSystem::without_system_fonts()`.
Não aloque gigante pra exercitar limit-check — use dimensão 1 acima
do limite. GPU init compartilhe via `OnceLock<Option<GpuContext>>`
module-level.

Vide também [[feedback-commit-cadence]], [[feedback-ci-batching]],
[[feedback-smoke-at-end]], [[feedback-ci-handling]],
[[project-perf-audit-2026-05-19]].
