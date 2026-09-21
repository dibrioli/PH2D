---
name: an-exemption-that-calls-screen-text-terminal-silences-the-census
description: Uma isenção de censo que declara "isto é terminal" sobre código que TAMBÉM fala pela tela cala o instrumento — e uma isenção que herda a premissa de outra herda o erro dela
metadata:
  type: feedback
---

**Medido em 2026-09-21, por uma FOTO do dono:** `✓ [sculpt3d] nao assou: this sprite is fully tra…`
— um prefixo **português** colado a uma frase inglesa, num aviso de ECRÃ, num app que o dono acabara
de mandar ser todo em inglês.

O censo de texto (HR-15) da família tinha o ficheiro isento, com o mecanismo escrito:
*«as linhas `[sculpt3d]` do ASSAR no **TERMINAL**»*. E a fase do quadro fazia:

```rust
eprintln!("{line}");
toasts.push(Toast::success(line));   // ← a MESMA String, na TELA
```

⇒ **ele é as duas coisas, e a isenção era metade da verdade.** A fronteira que o cabeçalho daquele
gate declara — *o terminal é de quem bisseca, o ecrã é do artista* — estava certa; o que estava
errado era a classificação de UM ficheiro, e ela calou o instrumento para sempre naquele caminho.

⚠️ **E o irmão caiu pela mesma linha:** um segundo ficheiro estava isento com a razão *«é a cauda da
MESMA frase do primeiro»*. **Uma isenção que herda a premissa de outra herda o erro dela** — as duas
foram retiradas na mesma corrida, e o gate achou na hora uma **terceira** frase que elas escondiam.

**Como apanhar isto antes do dono:** ao escrever ou ler uma isenção que diz *«terminal»*,
`grep` pelo consumidor da string — se ela chega a um `Toast`, a um `push` de aviso ou a qualquer
superfície, a isenção é falsa. *Um `eprintln!` ao lado de um `toasts.push` com a mesma variável é a
assinatura.*

Relacionado: [[feedback-a-gate-that-only-runs-the-happy-path]] ·
[[feedback-a-refusal-only-the-terminal-sees]]
