---
name: reference-session-transcripts-are-a-measurable-instrument
description: Os transcripts em ~/.claude/projects/ são JSONL com timestamps e tool_use — dá para MEDIR o comportamento do agente em vez de opinar sobre ele.
metadata: 
  node_type: memory
  type: reference
  originSessionId: 7ff8a7ad-aecc-4842-b7fc-a1193f7e419d
  modified: 2026-08-18T21:56:15.123Z
---

`~/.claude/projects/-home-enio-Documentos-Projetos-PH2D*/*.jsonl` — uma linha por evento,
com `type` (`assistant`/`user`), `timestamp` ISO, e o `message.content` carregando os blocos
`tool_use` (nome + input completo) e `thinking`/`text`. **101 sessões estavam gravadas ali em
2026-08-18**, incluindo as das worktrees (cada linha tem o próprio diretório de projeto).

**Por que importa:** perguntas como *"por que a LLM demora?"*, *"quais docs os agentes de fato
leem?"*, *"a disciplina do §2 está sendo seguida?"* têm resposta **medida**, não inferida.
Foi assim que se descobriu que o paralelismo de ferramenta estava em **1,00 chamada/turno**
e que **52% das edições iam por script** em vez da ferramenta `Edit` — dois fatos que nenhuma
leitura de código revelaria.

A sonda pronta é `bash scripts/agent-loop-profile.sh` (repo). Ela lê só os transcripts e não
escreve nada.

⚠️ **Três armadilhas, todas pagas em 2026-08-18:**
- O diretório começa com `-`, então `ls`/`glob` sem `./` ou caminho absoluto falha
  (o `ls` lê como opção).
- A **cwd do Bash persiste entre chamadas** — um `cd` numa chamada anterior faz o glob de
  transcripts devolver **vazio em silêncio**, que parece "não há dados". Caminho absoluto sempre.
- `git ls-files` + `.split()` por whitespace **parte caminhos com espaço** (`docs/Motion Nodes/`,
  `docs/Vector Module/`) ⇒ varredura cega. Use `.split('\n')`, e desconfie quando um total
  **sobe** depois de uma operação que só podia mantê-lo ou baixá-lo.

Ver também [[feedback_bash_cwd_resets_and_slips_to_the_primary]] e
[[feedback_a_negative_search_needs_a_positive_control]].
