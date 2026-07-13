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
