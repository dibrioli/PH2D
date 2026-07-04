---
name: feedback-audit-commit-msg-claim-verification
description: "Claims numéricos em commit message (\"9 sites swept\", \"Zero código tocado em W0\") precisam ser re-verificáveis via grep/diff EM CADA round; senão envelhecem em paralelo com a verdade"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 2145cc4f-66b3-4eb1-b4ee-05d0486ac094
---

Commit messages constroem narrativa do trabalho. Mas claim numérico ("9 sites swept", "Zero código tocado", "1094 LOC delta") torna-se ESTALE no momento em que o seguinte commit/audit estende o escopo. T1.3.5 commit `5974a84` afirmou "9 sites sin_cos() → 0 (workspace-wide; verificado via grep final)" — verificação foi com pattern restrito `\.sin_cos()` que perdeu 24 sites split-form. Claim ficou MENTIROSO no momento que R2 audit rodou grep amplo.

Similar: HANDOFF §10 dizia "Zero código tocado durante W0 (docs-only)" — verdade no momento da ratificação, falsa após cef1959 + 5974a84 introduzirem 1290 LOC. Texto não foi atualizado quando virou falso.

**Why:** Commit messages são WRITE-ONCE no git log; HANDOFF/SESSION_ACTIVE/spec docs são WRITE-MANY. Claim numérico ou de-cobertura sobrevive como ARTIFACT a partir do momento que foi escrito; verdade subjacente continua mudando. Próximo LLM lê o claim e propaga o EQUÍVOCO.

**How to apply:**
- **Em commit message:** evitar claims numéricos absolutos (`Total: 9 sites → 0`); preferir framing relativo (`Removed all f32::sin_cos and split f32::sin/cos calls reachable from SimWorld writes`) que envelhece melhor.
- **Quando IS necessário claim numérico** (auditoria total, e.g. "23 sites swept"), incluir LITERAL do grep usado: "verified via `grep -rn '\\.sin_cos()\\|\\.sin()\\|\\.cos()' crates/ tools/ shells/`". Auditor de próximo round re-roda o exato comando — se output diff, claim falsificou.
- **Em HANDOFF/SESSION_ACTIVE:** versionar o claim com timestamp (`Status @ 2026-05-28 cef1959`). Próximo agente que atualiza puxa a versão e flag stale.
- **Lens A audit responsabilidade:** brief incluir "verify each numerical claim in the commit message by re-running the cited tool/grep". Sem isso, Lens A só re-lê descrição, não verifica.
- **Padrão batch-end re-grep:** antes de stage + commit, re-rodar o grep amplo que vai EVENTUALMENTE ser feito por auditor R2. Se output não-zero, é cosmetic delivery quase-falha.

**Reference:** sessão Sprite Inspector v2 2026-05-28. Commit `5974a84` claim "9 sites swept" → R2 audit pegou 24 missed. HANDOFF §10 "Zero código W0" → meta-review META-H1 flagou após 1290 LOC committed.
