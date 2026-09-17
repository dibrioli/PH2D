---
name: feedback_a_contrast_cure_carries_the_surface_it_was_calibrated_against
description: "Uma cura de CONTRASTE é calibrada contra uma SUPERFÍCIE — escrevê-la como absoluta faz o mesmo report voltar quando a peça muda de superfície, com a MESMA assinatura (metade dos controlos)"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-16T02:54:18.572Z
---

Quando se cura *«não se vê»* escolhendo uma cor, a resposta é **relativa à superfície debaixo**. Se
o doc a escrever como uma propriedade da peça (*«a marca desmarcada é um degrau abaixo»*), ela
sobrevive a ser lida e **não sobrevive a a peça mudar de sítio**.

Caso medido (`line/UIUX`, dois dias seguidos, o MESMO dono e o MESMO report):

| | 2026-09-14 | 2026-09-15 |
|---|---|---|
| report | *«Checkbox invisível»* | *«O Checkbox desmarcado é invisível»* |
| a marca assentava em | o CARTÃO (`Bg1`) | uma CAIXA DE CAMPO (`Chrome::field_fill`) |
| a marca pintava | `Bg1` | `Chrome::field_fill` |
| distância | `0`/255 | **`0`/255** |

A cura do 1.º dia foi *«um degrau abaixo da superfície mais funda»* — que é literalmente o fundo da
caixa que o 2.º dia pôs por baixo dela. **Quem moveu a superfície foi a wave da véspera, minha.**

⭐⭐ **A ASSINATURA que identifica a família, e é o mais útil daqui:** as duas queixas eram sobre
**metade** dos controlos — a marcada lia-se sempre, porque é `Accent`. *Um report que fala de metade
de uma população uniforme é sobre o ESTADO que não tem cor própria, e a causa é contraste contra a
superfície, nunca o widget.*

**Why:** num tema moderno não há moldura de repouso, logo a única coisa que separa duas superfícies
é o degrau de tom (`SURFACE_STEP`, `10`/255). Duas superfícies que se tocam com `0`/255 **não são
duas**, e nenhum teste de geometria, registo ou alcance vê isso.

**How to apply:**
1. Uma cura de contraste vive numa **porta que nomeia a superfície** (`body_fill` = num cartão ·
   `on_field_fill` = num campo), nunca num token escolhido no sítio de pintura.
2. Quem **move** uma peça para outra superfície reconfere a nota da cor, na mesma wave
   (`CLAUDE.md` §0.0).
3. O gate tem **duas** metades: a porta devolve algo distante o bastante, **e o pintor chama essa
   porta** — a segunda mede a TINTA que ele pousou (`encoding().draw_data`), porque *uma porta certa
   que ninguém chama produz exactamente o app do report*.
4. E um **controlo** que afirme que a porta antiga seria invisível ali: sem ele, unificar as duas
   «porque fazem quase o mesmo» apaga a cura sem nada reprovar.

Ver [[feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site]] (a
irmã geométrica: lá o que muda com a superfície é a LARGURA) e
[[reference_topic_measurement_discipline]].
