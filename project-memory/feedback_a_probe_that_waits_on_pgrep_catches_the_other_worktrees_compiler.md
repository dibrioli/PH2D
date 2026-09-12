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

---

## ⛔ 2.ª INSTÂNCIA — `pgrep -f` apanha a PRÓPRIA espera (2026-09-08)

Para esperar que outra linha acabasse a suíte antes de medir relógios (§5.0, `load ≤ 5`), armei
`until ! pgrep -f 'cargo-nextest' …; do sleep 20; done`. Ela **nunca dispararia**: `-f` casa a
linha de comando inteira, e duas coisas contêm o nome sem ser o processo procurado —

1. **a própria espera** (o `sh -c` que corre o `pgrep` tem a string no argumento), e
2. o **`earlyoom`**, cujo `--prefer ^(rustc|cargo|cargo-nextest|…)$` traz o nome num regex.

⇒ **`pgrep -x <nome>`** (casa o nome do processo, não a linha) devolveu os **dois** nextest reais e
mais nada.

**How to apply:** uma espera por *«aquele programa ainda corre?»* casa pelo **nome do executável**
(`pgrep -x`), nunca pela linha de comando — e antes de a armar, corra `pgrep -a<flag>` uma vez e
**olhe o que ela apanhou**. Uma espera que nunca dispara lê-se exactamente como um trabalho que
nunca acaba.

## ⛔⛔ 3.ª INSTÂNCIA — 2026-09-12, e custou SETE HORAS de processos presos (integração da W2)

O integrador esperou pelo `foundational-integrate.sh` com
`until ! pgrep -f 'scripts/foundational-integrate' >/dev/null; do sleep 20; done` — exactamente a
forma que a 2.ª instância acima proíbe. Cada espera casava com a **própria** linha de comando e com
**todas as outras esperas**, e nunca terminava. Acumularam-se **16** processos, o mais velho com
**7 h 51 min**, e o dono perguntou *«?»* porque a sessão parecia parada — o portão tinha terminado
havia muito, sem veredito visível (a saída fora ainda por cima a um `| head`, que fechou o tubo).
⚠️ **A cura já estava escrita aqui, com o nome (`pgrep -x`), e não foi lida.**
⇒ para esperar por um comando lançado em segundo plano, **use a notificação do próprio lançamento**
(o `run_in_background` avisa quando o processo sai) e leia o ficheiro de saída — nunca uma sonda por
nome de processo. E nunca passe um portão por `| head`/`| tail`: perde-se o veredito e o código de
saída.
