---
name: a-badge-is-painted-text-and-the-key-of-its-own-colour
description: Um selo de 3 letras é exibição E identidade — o `badge_tone` casa contra o texto pintado, logo traduzi-lo apaga a cor em silêncio
metadata:
  type: feedback
---

⛔⛔ **Um SELO (`ISO`, `SUB`, `LNK`, `PRF`) é pintado E é a chave do TOM dele.** O
`badge_tone` da linha da Hierarquia (`ph2d-panel-hierarchy/src/row.rs`) faz `match badge { "ISO" =>
Warn, … }` sobre o **texto pintado** — o mesmo `&str` que o artista lê. Traduzir o selo devolve
`Neutral` **em silêncio**: a letra muda, a cor cai, e nenhum gate de registo ou de pintura acusa.

**Why:** a nota do `bool_shape::badge` usava exactamente este facto como JUSTIFICAÇÃO para manter o
literal no fonte (*«não passa por i18n, de propósito»*) — e é ao contrário: um literal não conserta
a dupla vida, só a esconde num sítio onde ninguém a lê. É a lição *identidade ≠ exibição*
([[a-key-and-a-text-of-the-same-type-is-a-defect-waiting]]) numa forma nova: aqui a identidade não é
um ficheiro gravado, é **a cor**.

**How to apply:** quando um texto curto também DECIDE alguma coisa (cor, ordem, agrupamento),
pergunte *quem casa contra ele?* antes de o traduzir. A cura certa é o código viajar como **ID** até
quem decide, e o texto ser resolvido só na pintura. Enquanto isso não existir, a cerca é executável:
`the_badge_tone_still_matches_what_the_table_paints` (2026-09-16) reprova no dia em que a tabela e o
`badge_tone` deixarem de concordar, e diz qual é a cura. Ver [[a-refusal-only-the-terminal-sees]].
