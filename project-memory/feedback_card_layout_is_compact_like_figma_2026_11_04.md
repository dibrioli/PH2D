---
name: feedback_card_layout_is_compact_like_figma_2026_11_04
description: Dono 04/11 — todo cartão é compacto como o Figma: irmãos curtos dividem a linha, nada grudado, uma grelha; um campo por linha é defeito
metadata:
  type: feedback
---

Dono, 04/11, olhando Color & Tint no smoke: «O layout figma não é compacto? essa deve ser uma regra!
tint e self tint poderiam estar na mesma linha. O botão [Equalize] está grudado nas cores acima. Opacity e
Tint Fill poderiam estar na mesma linha. É preciso estabelecer regras».

A lei completa está em `docs/CartoesUI/05_ESPECIFICACAO_UI_COMPLETA.md` §4 (generaliza a T56).

**Why:** a T56 só foi aplicada ao Physics Body; sem lei geral, cada agente empilhou um campo por linha, o painel cresceu
e o resultado ficou espaçoso. Teste verde e foto «no padrão» não apanham isto: a pergunta tem de ser feita.

**How to apply:** ao desenhar ou rever um cartão, pergunte de cada linha com um campo só se o vizinho cabe ao lado. Toda
fronteira entre linhas, botão incluído, tem o vão de `metrics`. Só a largura estreita desculpa o par em duas linhas.
