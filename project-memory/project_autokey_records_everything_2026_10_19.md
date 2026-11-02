---
name: project-autokey-records-everything-2026-10-19
description: "Ordem do dono (19/10) — o Auto Key grava ABSOLUTAMENTE TUDO no app (cor, traço, pontos do desenho, ...), não só transformações e opacidade"
metadata:
  node_type: memory
  type: project
  originSessionId: 81a2b1d3-d233-422c-8495-3a378b125a65
  modified: 2026-10-20T00:07:32.355Z
---

Ordem do dono, 19/10 (smoke F23 OK, `line/UIUX`): *«O auto key deve gravar absolutamente tudo neste app, não só as transformações»*.

**Why:** no smoke do Auto Key sobre uma forma vetorial ele viu que só a opacidade e a posição do OBJECTO gravam; a Posição X/Y do painel do vetor, o arrasto e o empurrão da ferramenta Vetor movem os PONTOS do desenho (`translate_path_world`), e a timeline só anima os 19 canais do `PropKind` (Transform, Opacity, Morph, juntas, ossos, macro) — cor, largura do traço, tracejado, cantos, a geometria e o resto não têm canal.

**How to apply:** toda propriedade que o artista edita é animável e o Auto Key armado grava-a no quadro (o modelo do Rive/After Effects: cada propriedade tem chave). Não proponha «mexer no objeto em vez dos pontos» como cura — a edição dos pontos também tem de virar chave. Antes de desenhar, inventarie as propriedades editáveis por família e meça o custo do modelo de canais (o `PropKind` é fio congelado só por APPEND). Liga com [[feedback-the-owners-reference-product-decides-the-law]].

**Decisões do dono (20/10), sobre o plano 35 da `line/UIUX`:** (1) mexer um ponto grava a FORMA INTEIRA numa chave (After Effects/Spine), com inserir/apagar ponto propagado a todas as chaves — a chave por ponto (Rive/Moho) está recusada; (2) a pintura do Painter também anima QUADRO A QUADRO, como o Flip (pintar num quadro sem chave com o Auto Key armado cria o quadro). Não reabra estas duas.
