---
name: feedback_a_handoff_can_promise_one_line_in_the_router_and_ship_eighty_nine
description: O §8 de um handoff cita o parágrafo a pôr no CLAUDE.md §5 — meça o diff do arquivo, não leia a promessa
metadata:
  type: feedback
---

Na integração da `line/3DModeling` (2026-08-26) o §8 do handoff dizia *«A linha para o
`CLAUDE.md §5` (**UMA**, e a narrativa fica aqui)»* e citava um parágrafo. O branch trazia
**89 linhas** (+7,8 KB): a narrativa inteira das waves W59–W80. O parágrafo citado no §8 nem
sequer existia no arquivo — `grep "jornada de 26/08"` deu **0**.

**Why:** o §5 é o único arquivo cujo custo **todo** agente paga em **toda** janela (466 k tokens
de contexto inicial, medido 2026-08-18), e a compactação não o alcança. Uma linha que escreve a
narrativa ali cobra-a de todos, para sempre — e o integrador é o último portão antes disso.
O handoff descreve a INTENÇÃO da linha; o diff descreve o que ela fez. Ver
[[feedback_a_handoff_can_be_wrong_about_its_own_dirty_file]].

**How to apply:** na integração, depois do `--ff-only`, meça sempre:
`git diff <base>..HEAD -- CLAUDE.md | grep -cE '^\+'`. Acima de ~10 linhas, comprima — mas só
depois de confirmar que a narrativa tem **casa endereçável** (aqui: `docs/3DModeling/06_*.md`
§69–§81, uma seção por wave). Preserve item a item o que é ROTEADOR — o que está aberto, as
recusas medidas com o número que as mata, os bugs cujo dono é outra linha — e mande para a casa
só o MECANISMO. Ver [[feedback_archiving_without_indexing_the_refusals_deletes_them]].
