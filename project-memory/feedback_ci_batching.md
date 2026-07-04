---
name: feedback-ci-batching
description: "Durante migrações multi-PR planejadas (Wave 1, Wave 2, etc.), NÃO fazer push + CI a cada PR. Acumular todos os commits localmente; push + CI babysit + STATE.md update SÓ no fim da Wave inteira."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 446b5c87-0504-45d9-a6cd-6f2040a1ab8e
---

Durante migrações Wave-style com plano canônico documentado em
`docs/Migracao/*.md` (múltiplos PRs sequenciais), commits são LOCAIS
até a Wave inteira completar.

**Why:** PR + CI a cada PR (10-20 vezes em uma Wave) gasta tempo
desnecessário em babysit, custa minutos × N de CI matrix, e
acumula merge conflicts mínimos. Wave 2 (17 PRs) seria 17× ~30min
de CI babysit = 8.5h gastas em waiting. Acumular tudo: 1× CI no fim.

**How to apply:**
- Implementar cada PR localmente (commit local).
- Pre-commit hook valida T2 (clippy workspace, nextest, fmt) — isso é
  suficiente para garantir cada commit válido.
- NÃO chamar `git push origin main` até a Wave estar 100% completa.
- NÃO monitorar CI runs durante a Wave.
- No fim da Wave: single push de todos os commits, single CI babysit
  (PRCI loop §10), single STATE.md update.

**Exceção:** Wave 1 já foi pushada antes dessa convenção. Wave 2 começa
com essa política. Estabelecido em sessão 2026-05-16 após PR 11.1.0
(cuja CI run 25967463679 deixei rodando mas não monitoro).

Mesma lógica vale para qualquer migração futura: 10+ PRs planejados →
batch push no fim.

Vide [[feedback-ci-handling]] (regra geral de CI handling).
