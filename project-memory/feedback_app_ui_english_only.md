---
name: app-ui-english-only
description: "Botões/labels/toasts/strings user-facing do app PH2D ficam SEMPRE em inglês, mesmo quando o Enio descreve a feature em pt-BR. Comentários de código podem ser mistos."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7d5e6481-e38a-41fd-b4ce-ae6413dd4bc6
---

User-facing strings em qualquer crate do PH2D (button labels, toast messages, snapshot label fields, panel titles, tooltip text — tudo que aparece na UI) ficam **sempre em inglês**, independente de o Enio descrever a feature em pt-BR ou usar termos pt-BR no pedido.

**Why:** o app PH2D inteiro é em inglês como design-decision. Enio reagiu fortemente quando coloquei "Acrescentar área" como label de botão (2026-05-26): "Tire todo PT do app .... tudo em inglês". Comentários técnicos no código (doc-comments + inline) seguem o padrão existente que MISTURA pt-BR + inglês — não traduzir esses, eles ficam.

**How to apply:** ao implementar feature nova:
- Mesmo se o Enio escrever "Acrescentar área" no prompt, usar **"Add area"** como label do botão.
- Nomes técnicos pt-BR que ele cita (e.g., "Detect Subject") → manter conforme já existe no app (caso a caso; checar antes).
- Comentários de doc/código: livre escolha (segue padrão do arquivo).
- Toast messages, button labels, panel titles, error strings → SEMPRE inglês.

Linka com [[feedback-communication-style]] (que descreve estilo da comunicação Claude↔Enio, não strings do app).

**Incidente 2026-05-27 R7/R9 — auditor adversarial mis-cite HR-15:**
durante o T1.6 R7 audit, lens J1 marcou os toasts EN do
`painter.rs` drain como "HR-15 violation" e recomendou tradução
pra pt-BR. Aceitei sem conferir esta memória e traduzi 5 strings.
R9 lens V1 detectou o "i18n split" (painter pt-BR vs outros tools
EN) e recomendou padronizar — escalei traduzindo bgremoval, padding,
color_eq, upscale (4 drain files). Total: ~10+ strings user-facing
em pt-BR, todas erradas. Enio apontou. Revert via commit `7fed63b`.

**How to apply (regra reforçada):** quando um auditor adversarial citar
HR-15 OR "i18n violation" OR "inconsistent language" em strings
user-facing, **CONFIRMA AQUI ANTES** de aceitar. Esta memória é
autoritativa; auditor que sugere o oposto está errado. HR-15 não
mandata pt-BR — é regra de internacionalização (i18n keys), não de
language-of-default. Default-language-of-PH2D = English, fim.
