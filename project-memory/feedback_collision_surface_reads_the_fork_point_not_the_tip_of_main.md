---
name: feedback-collision-surface-reads-the-fork-point-not-the-tip-of-main
description: A collision-surface.sh compara a linha com o merge-base, não com o main de AGORA — da 2ª linha em diante da jornada ela não pode ver uma colisão de mesmo-literal
metadata:
  type: feedback
---

A `scripts/collision-surface.sh` calcula `MB = git merge-base HEAD main` e lê a coluna
**`base`** *nesse commit* — o **ponto de fork da linha**, não o tip do `main`. Como o
merge-base só se move quando a linha **rebaseia**, a segunda linha de uma jornada continua
a ler o `main` do dia em que ela nasceu, mesmo depois de outra linha ter integrado.

**Medido em 2026-08-24** (integração `line/components` + `line/Vector`): as duas apendaram
um campo ao `ProjectFile` e as duas escreveram `PROJECT_SCHEMA = 96`. Depois de a primeira
integrar (main = 96), a sonda na segunda worktree ainda imprimia:

```
⚠ PROJECT_SCHEMA    96   (base: 95)
```

— que se lê como um bump **normal e incontestado**. O valor certo era **97** (95 + 1 + 1).

**Why:** a DIRETRIZ §1.5.3/§1.5.9 promete que re-rodar a sonda imediatamente antes de fundir
faz a divergência aparecer (*"se a coluna base divergir, a divergência é ela própria um
achado"*). Essa promessa **só se cumpre se a base for o tip do main**. Com o merge-base, a
coluna `base` é imóvel por construção, e o caso que a sonda existe para apanhar — ⚠️ **a
colisão MUDA, em que as duas linhas escrevem o MESMO literal** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]])
— é exactamente o que ela deixa passar. *Uma sonda cuja coluna de referência não pode mudar
não está a comparar nada.*

**How to apply:** como integrador, **não confie na coluna `base` da sonda a partir da 2ª
linha**. Leia o valor no `main` de AGORA, à mão, antes de rebasear:

```bash
grep -n 'PROJECT_SCHEMA: u32' /…/PH2D/shells/desktop/src/project_schema.rs
grep -n '(9[0-9], 1[0-9], 1[0-9])' /…/PH2D/shells/desktop/src/project_schema_tests.rs
```

Ou rode a sonda **depois** do rebase, quando o merge-base já é o main integrado. Vale para
todo número que soma: `PROJECT_SCHEMA`, contadores de registro, números de ADR.
Irmãs: [[feedback_a_shared_list_is_merged_against_todays_main]] ·
[[reference_topic_integration_discipline]].
