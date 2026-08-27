---
name: feedback_when_two_lines_pick_the_same_literal_the_collision_probe_goes_blind
description: collision-surface compara valor-contra-base — duas linhas no MESMO literal dão base==valor e ele não avisa
metadata:
  type: feedback
---

Jornada de 2026-08-26: `line/Vector` e `line/components` subiram, cada uma por sua conta,
`PROJECT_SCHEMA` de **97 para 98** — o mesmo literal, para DUAS mudanças de formato diferentes.

**Why:** o `collision-surface.sh` imprime `valor (base: N)` e só põe `⚠` quando os dois diferem.
Enquanto as duas linhas estão por integrar, a base é o `main` velho (97) e ele avisa nas duas.
⛔ **Depois de a primeira entrar, a base passa a ser 98 e a segunda lê `98 (base: 98)` — sem `⚠`
nenhum.** O git também não vê nada: os dois lados escrevem o mesmo texto, o merge é limpo, e a
árvore fica com **um** número para **duas** mudanças de formato. Um ficheiro gravado por uma seria
lido errado pela outra, em silêncio — que é exactamente o que o bump existia para impedir.

O irmão dele funde **alto** e engana de outra maneira: os espelhos do registo de componentes eram
`71`, a `Vector` pôs `72` (+1) e a `components` pôs `77` (+6). Aí o git CONFLITA — e o valor certo
é **78**, que não está em nenhum dos dois lados. Ver
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]].

**How to apply:** antes de fundir a PRIMEIRA de duas linhas que tocam um número que soma, meça o
número **em todas as linhas vivas**, não só na que vai entrar:

```bash
for r in main line/A line/B; do
  git show "$r:shells/desktop/src/project_schema.rs" | sed -n 's/.*PROJECT_SCHEMA: u32 = \([0-9]*\).*/\1/p'
done
```

⚠️ Em zsh, `"$r:shells/..."` precisa das aspas — `:s` e `:c` são modificadores de história e
comem o caminho **em silêncio**, devolvendo vazio. Escreva a varredura num ficheiro e corra-a com
`bash`. Guarde o **delta de cada linha** (+1, +6), não o valor: o valor certo é `base + Σ deltas`,
e a escada de migração da segunda tem de ser **re-numerada** junto. Ver
[[feedback_a_pastable_bash_loop_never_iterates_under_zsh]] e o `CLAUDE.md` §5.0.
