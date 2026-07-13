---
name: feedback-duplicate-allowlist-key-kills-the-gate-at-parse
description: "Allowlist é ímã de conflito — a união duplica a chave, o TOML morre no parse, e o gate inteiro para de escanear (escondendo erros reais debaixo)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6316633f-521c-4b1d-a255-7662e2fda363
---

No ship da integração das 6 linhas (2026-07-12), o `typos` falhou com **`duplicate key`** no
`.typos.toml` (linha 110): três palavras pt-BR (`discriminante`, `repete`, `responde`) foram
apendadas por **duas linhas diferentes** e a união do merge as duplicou.

O `typos` **nem chegou a escanear um arquivo** — morreu no parse do TOML. E debaixo dele havia um
typo **real** em inglês (`lifes` → `lives`, na crate nova `ph2d-node-sim-lifetime` e num teste do
Motion) que só apareceu depois de deduplicar o arquivo.

**Why:** um gate que morre na **configuração** não reporta "nada encontrado" — reporta *erro*, e
é fácil confundir os dois num sumário. Se o gate estiver marcado como opcional/`continue-on-error`
em algum lugar do pipeline, isso vira **verde por não-execução**: o gate mais perigoso que existe.

**How to apply:**
- Allowlists (`.typos.toml`, `hr12_widgets_a11y.rs::PANEL_A11Y_DELEGATE_OK`, `FILE_OVERAGE_OK`,
  `architecture_panel_loc_cap`) são **ímãs de conflito**: toda linha apenda no mesmo bloco. Na
  integração, **deduplique** — não basta unir.
- Depois de mexer numa allowlist, **rode o gate** e confirme que ele *escaneou* (leia a saída, não
  só o exit code) — [[feedback_pipe_masks_script_exit_code]].
- Typo em palavra **inglesa** = renomeie o identificador. Allowlist é só para palavra **pt-BR** de
  comentário; usá-la para esconder inglês errado é dívida.
