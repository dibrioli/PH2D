---
name: feedback_a_tree_scanning_gate_is_never_reached_by_a_name_filter
description: "Gate que VARRE uma árvore vive noutra crate e nunca é alcançado por `cargo test -p <crate> <filtro>` — o alvo do fecho tem de incluir quem VARRE o que a linha tocou, não só quem ela editou"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-24T01:09:20.244Z
---

No fecho da `line/3DModeling` (2026-08-23) corri a suíte **inteira** das crates tocadas pela
primeira vez em várias waves, e encontrei **duas cercas vermelhas desde as W38–W51**:

- `shell_files_respect_hr18_loc_cap` (em `shells/desktop/tests/`) — **quatro** arquivos acima de
  600 LOC, três deles antes da wave do dia (em `main` cabiam todos: 506, 555, 585);
- `no_tofu_glyphs_in_ui_strings` (em `crates/ph2d-editor-core/tests/`) — **nove** `→` em mensagens
  de `assert!` de arquivos do shell.

Nenhuma das waves as viu, e nenhuma foi descuidada: cada uma correu `cargo test -p ph2d-host-desktop
<filtro-do-que-mudei>`, que é a corrida dirigida certa. **Um gate que VARRE uma árvore não tem nome
que case com filtro nenhum** — e o segundo nem sequer vive na crate que a linha editava.

**Why:** [[feedback_the_closing_clippy_must_cover_every_crate_the_line_touched]] diz *derive o alvo
do DIFF*, e isso resolve **quem a linha editou**. Falta a outra metade: quem **varre** o que ela
tocou. Um gate de árvore mora onde a *regra* mora (o teto de LOC no shell, o tofu no `editor-core`),
nunca onde o *arquivo* mora — então a crate que o hospeda pode não aparecer no diff nenhuma vez.
⚠️ E o modo de falha compõe: as duas cercas ficaram vermelhas **waves inteiras** sem ninguém notar,
e quem as pagaria era o integrador, com cinco linhas em cima da mesa.

**How to apply:** no fecho, corra a **suíte inteira** (sem filtro) de cada crate do diff **e** das
que hospedam gates de árvore — hoje `ph2d-host-desktop` e `ph2d-editor-core`:

```bash
cargo test -p ph2d-host-desktop            # sem filtro: apanha os `tests/` de árvore
cargo test -p ph2d-editor-core --test no_tofu_glyphs
```

⚠️ **Meça LOC DEPOIS do `fmt`, nunca antes**: acrescentar um argumento a 32 chamadas não muda o
número de linhas — o `cargo fmt` é que parte a chamada longa e **cria** as linhas que estouram o
teto ([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).

⚠️ A cura de um teto é **cortar para o irmão** numa fronteira que já existia por dentro, nunca uma
allowlist — e a fronteira costuma estar escrita nos doc-comments do próprio arquivo.

*O alvo do fecho inclui quem VARRE o que a linha tocou, não só quem ela editou.*
