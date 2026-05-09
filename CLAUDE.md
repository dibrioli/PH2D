# CLAUDE.md — diretrizes operacionais para LLM no PH2D

Vide [`SKILL_Stack_PH2D_Definitiva.md`](SKILL_Stack_PH2D_Definitiva.md) para
arquitetura, stack, Hard Rules (HR-1..HR-17), convenções e tudo que é
**técnico do projeto**. Este arquivo é APENAS workflow operacional —
quem faz o quê, quem confere o quê.

## CI / GitHub Actions

- **Conferência visual do status do CI é responsabilidade do Enio.** Ele
  tem a aba do GitHub aberta e verifica direto. LLM não polla.
- Após `git push`, **forneça SEMPRE o link da run**:
  `https://github.com/dibrioli/PH2D/actions/runs/<run-id>`.
  Use `gh run list --workflow=spike.yml --limit=1` para pegar o ID.
- Se um job falhar, forneça também link direto do job que falhou
  (`gh run view --job=<job-id>`).
- **Não monitore CI em loop** quando não há próxima ação dependente do
  resultado. Push, fornece link, prossegue para o próximo trabalho.
- Se a próxima ação realmente depende de CI verde (ex: merge automático
  via `gh pr merge --auto`), perguntar a Enio antes de bloquear.

**Quando monitorar é OK:**
- Enio explicitamente pediu ("monitore o CI")
- Próxima ação exige CI verde e Enio confirmou que está OK aguardar

## Memória persistente da LLM

Vide [`~/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`](file:///Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md)
para feedback acumulado, perfil do Enio, estado do projeto, paths canônicos.
LLM nova chegando lê esse índice antes de tomar ações.

## Plano operacional ativo

[`docs/plans/2026-05-post-spike.md`](docs/plans/2026-05-post-spike.md) — 13
marcos M1..M13 para implementação real do core pós-spike. Iniciar do M1.
