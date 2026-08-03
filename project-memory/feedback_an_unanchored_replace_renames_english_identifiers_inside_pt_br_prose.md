---
name: feedback-an-unanchored-replace-renames-english-identifiers-inside-pt-br-prose
description: Corrigir grafia pt-BR com replace cego morde a palavra INGLESA dentro de nomes de teste
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 55ac7554-e543-455e-be55-15b5f3eb4809
  modified: 2026-08-03T05:56:56.219Z
---

Num arquivo que mistura **prosa pt-BR** e **identificadores em inglês** (o caso normal deste
repo: comentários em português, nomes de teste em inglês), um `s.replace("reflecte", "reflete")`
para consertar a grafia portuguesa também acerta o `reflect` dentro de
`fn the_reflected_copy_keeps_the_winding()` → `the_refleted_…`.

**Why:** aconteceu na integração de 2026-08-02, drenando a dívida de `typos` das 5 linhas. O
compilador **não reclama** (o teste continua válido), a suíte continua verde, e o que mudou foi
o **NOME DE UM GATE** — que é como um teste some de uma busca por nome, ou como um `-E 'test(...)'`
de CI deixa de casar. É a irmã do
[[feedback_python_replace_silent_noop_after_fmt]]: lá o replace não casava nada, aqui casa
DEMAIS.

**How to apply:** substitua a **frase inteira** onde a palavra vive, nunca o token solto —
`("reflectir vértices, repor o winding", "refletir vértices, restaurar o winding")`. E depois
de qualquer correção ortográfica em massa, **`git diff | grep -E "^[-+].*fn "`**: se um `fn`
aparece nos dois lados, você renomeou um teste sem querer.
