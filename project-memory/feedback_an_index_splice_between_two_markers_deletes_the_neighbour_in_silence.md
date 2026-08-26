---
name: feedback-an-index-splice-between-two-markers-deletes-the-neighbour-in-silence
description: Reescrever um bloco por `s[:a] + novo + s[b:]` apaga tudo o que estiver entre os dois marcadores — e compila, passa nos testes e commita sem um sinal.
metadata:
  type: feedback
---

Para substituir um gate obsoleto usei duas vezes o mesmo padrão:

```python
a = s.index("/// marcador do início")
b = s.index("/// marcador do fim")
s = s[:a] + novo + s[b:]        # ⛔ e o que estava ENTRE os dois evaporou
```

Nas duas vezes havia **outra função no meio** — a sonda `measure_the_export_wall_clock` — e ela
desapareceu. ⛔ **Nada avisou:** o arquivo compila, `cargo fmt` aceita, a suíte fica verde (um teste
a menos ainda é «ok»), o clippy cala-se, e o commit passa. Só a descobri **três waves depois**, ao
tentar correr a sonda e não haver nada para correr.

**Why:** um `str.index` é uma posição, não uma âncora semântica. *A operação parece uma substituição
e é uma remoção de intervalo* — e o intervalo cresce sempre que alguém acrescenta código entre os
dois marcadores, que é exactamente o que uma base viva faz.

**How to apply:**
- **Substitua por `str.replace(old, new)` com o texto INTEIRO do bloco** e um `assert count == 1`.
  O `Edit` da ferramenta faz isto por construção e **falha alto** quando não casa — é por isso que
  o `CLAUDE.md` §2 manda usá-lo ([[feedback-python-replace-silent-noop-after-fmt]]).
- Quando o corte por índice for mesmo a forma certa (blocos grandes), **afirme o que está a sair**:
  `assert "outra_fn" not in s[a:b]`, ou conte as funções antes e depois.
- ⚠️ **Uma suíte verde não prova que os testes existem.** Depois de reescrever um arquivo de testes,
  confira a **contagem** (`--list`, ou `grep -c "^fn "`) contra a de antes — um teste apagado e um
  teste a passar leem-se igual no sumário.
