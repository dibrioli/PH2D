---
name: feedback-a-probe-that-waits-on-pgrep-catches-the-other-worktrees-compiler
description: "Num repo com worktrees, esperar por «o compilador acabou» via pgrep apanha o compilador de outra árvore, e mtime não distingue binário velho de novo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-03T18:47:43.423Z
---

Esperar por uma build com `pgrep -f "rustc.*<crate>"` **não** é uma pergunta sobre a tua árvore:
no Modo L há várias worktrees e o dono corre coisas na primária. Medido em 2026-09-03: três
sondas ficaram presas ~20 min à espera que terminasse um `nextest --workspace` que o Enio corria
em `/home/enio/Documentos/Projetos/PH2D` (a primária), enquanto o meu binário já estava pronto.

E o `mtime` também não serve: um binário **velho** pode ser mais recente que o fonte que se
acabou de editar (a build anterior terminou depois da edição), e a sonda corre em silêncio com o
programa errado — foi assim que uma corrida inteira mediu código que não existia.

**Why:** as duas heurísticas respondem *«há um compilador a correr?»* e *«qual é mais novo?»*,
e a pergunta é **«este binário contém a minha mudança?»**.

**How to apply:** espere por uma **string** que só o código novo tem
(`grep -qa "<literal do print novo>" <binario>`), ou colha o caminho da linha
`Executable unittests …` que a própria build imprime. ⛔ Nunca `ls -t` no `target/release/deps`
(apanha o executável do PROGRAMA, ver [[feedback-run-command-include-cd]]) e nunca um `pgrep`
sem filtrar pela worktree. Ver também
[[feedback-bash-cwd-resets-and-slips-to-the-primary]] — o `cd` de um comando não sobrevive ao
seguinte, e um caminho relativo mede a árvore primária.
