---
name: feedback_python_replace_silent_noop_after_fmt
description: `str.replace()` que não casa é um no-op SILENCIOSO — e `cargo fmt` reflow muda o texto entre uma edição e a próxima
metadata:
  type: feedback
---

Ao editar código com `python3 -c "... s.replace(old, new)"`, **sempre `assert old in s`** antes.

Um `replace` que não encontra o padrão **não falha** — escreve o arquivo inalterado e o script sai
com exit 0. O teste seguinte roda contra o código **antigo** e você interpreta o resultado como se
fosse o novo.

**Why:** custou uma rodada inteira de depuração no ADR-0119. Eu tinha rodado `cargo fmt` num commit
anterior, que **colapsou um `match` multi-linha para uma linha só** — então o padrão que eu copiara
da minha própria escrita anterior deixou de existir. A correção "aplicada" nunca entrou no arquivo, o
gate ficou verde sob mutação, e eu quase concluí que o gate é que estava cego. É a mesma família de
[[feedback_pipe_masks_script_exit_code]]: a operação falha e você lê sucesso.

**How to apply:** `assert old in s, "PATTERN NOT FOUND"` em todo replace. Se o padrão vier de código
que você escreveu antes de um `fmt`, **releia o arquivo primeiro** — não confie na sua memória do
texto. Melhor ainda: use a Edit tool (ela **erra** quando não casa) em vez de python para mutação
pontual. E quando um resultado te surpreender, **olhe o arquivo** antes de teorizar.

---

⛔⛔ **REINCIDIU EM 2026-08-23, e desta vez COM o assert — que passou.** O script
imprimiu `aviso acrescentado` (a última linha depois do `write`), e o ficheiro ficou
**inalterado nas duas árvores**; `grep -c` da agulha nova devolveu `0` no worktree, no
primário e numa busca recursiva pelo repo inteiro. *O mecanismo NÃO foi apurado* — o
que ficou apurado é o facto: **o `assert` não é rede suficiente**, porque ele guarda a
agulha e não o resultado.

⇒ **A regra endureceu:** o `CLAUDE.md` §2 já manda usar a ferramenta `Edit`, que
**falha alto** quando `old_string` não casa. Usar `python3` para uma substituição
única é a escolha errada mesmo com `assert`. ⚠️ **E quando um script escrever ficheiro,
CONFIRME o resultado, não a intenção** — `grep -c` da agulha nova, não o `print` do
próprio script. *Ler o «sucesso» que o script imprime é ler a intenção dele.*

⚠️ **O caso legítimo continua a ser o de N ficheiros** (renomeação, mutação em lote) —
e mesmo aí a confirmação é a contagem no ficheiro, não a saída do script.

## ⛔⛔ Adenda 2026-09-02 — o `assert` de contagem **passou** e o ficheiro ficou PARTIDO

A regra desta memória é *«script só com `assert` de contagem»*. Ela não chega, e o caso é este:

Converti 5 sítios de chamada de `box_row(r, label, value, t, st, accent, deco, style)` para
`box_row(r, PropertyBox { … }, style)` com um regex de grupos nomeados, e com
`assert n == 5, "esperava 5 sítios"`. **O assert passou** — os cinco casaram — e **três ficaram
lixo sintáctico**, porque o 1.º argumento de dois deles era `Rect::new(b.x, b.y, w, row_h)`: as
vírgulas *dentro dos parênteses* alimentaram os grupos seguintes, e o `label` recebeu `b.y`.

⚠️ **O `assert` contou ACERTOS, não CORRECÇÃO.** Um regex sobre argumentos separados por vírgula é
cego a parênteses aninhados — ele não falha, ele **casa e mente**. E o ficheiro não estava no git
(reescrito na mesma sessão), então não havia `git checkout` para desfazer: a reparação foi à mão,
sítio a sítio.

**How to apply:**
- ⛔ **Nunca use regex para reagrupar ARGUMENTOS.** Vírgula não é separador quando há chamadas
  aninhadas, e a linguagem não é regular nesse ponto. Renomear um símbolo, sim; re-parenteizar, não.
- Se um script tocar em estrutura, o `assert` tem de ser sobre a **saída**, não sobre a contagem de
  substituições — o mais barato é **compilar**: `bash scripts/cargo-check-narrow.sh <crate>` no fim
  do próprio script, e restaurar do backup se ele não devolver `0`.
- ⚠️ **Faça o backup mesmo quando o ficheiro é novo.** A rede de segurança que se usa por reflexo
  (`git checkout --`) **não existe** para trabalho não commitado desta sessão — e é precisamente aí
  que o script é mais tentador, porque o ficheiro acabou de ser escrito.
- ⭐ Cinco sítios cabem em cinco `Edit`. *O script começa a compensar muito depois do que parece.*
