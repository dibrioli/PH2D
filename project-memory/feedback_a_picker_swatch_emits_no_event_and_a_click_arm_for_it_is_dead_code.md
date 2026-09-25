---
name: feedback-a-picker-swatch-emits-no-event-and-a-click-arm-for-it-is-dead-code
description: "O despacho curto-circuita toda amostra registada por register_picker_swatch — ele abre o selector e devolve, logo NENHUM Click chega ao painel e um braço escrito ali é código morto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 50c6f100-3d87-4634-ab58-979e99957aed
  modified: 2026-09-20T23:58:44.635Z
---

O `pointer_down` tem um ramo GENERALIZADO: `if store.is_picker_swatch(id)` → semeia
`widget_color`, aponta `set_picker_target`, semeia o `INSP_BLENDER_PICKER` e **`return`** — de
propósito, *«para o clique não focar nem arrastar o canvas»*. ⇒ **nenhum `WidgetEvent` é emitido**.

**Why:** ao dar uma amostra de cor a uma secção nova (as propriedades de script, 2026-09-20) eu
escrevi o braço de `Click` à mão — a semente, o alvo, o valor do widget — e ele era **CÓDIGO
MORTO**: *a porta generalizada já existia e eu escrevi a segunda resposta à mesma pergunta*. O que
faz a amostra funcionar é **uma linha no `populate`** (`register`+`register_picker_swatch`).

E o gate de costura padrão (`Down`+`Up`, `assert!(!evs.is_empty())`) **reprova sobre produto
CERTO** ali — a ausência de evento é o desenho. Quem o distinguiu foi o **CONTROLO**: um campo
numérico vivo há waves emite `Focus + Click`, logo o instrumento estava bom e o sujeito é que era
outro.

**How to apply:** numa amostra de cor, (1) não escreva braço de clique nenhum; (2) o gate mede o
**EFEITO** — `picker_target == Some(id)` e a semente igual ao valor gravado —, nunca um evento; e
(3) a semente por-quadro é quem manda a edição enquanto o selector apontar ali (o regime das
amostras de tinta da Sprite). ⚠️ E não esqueça a lista do `is_sprite_color_swatch`: sem a amostra
nova lá, trocar de objecto com o selector aberto **escorre a cor** para o objecto seguinte, em
silêncio. Ver [[feedback-a-gesture-written-in-two-halves-accepts-a-new-variant-in-only-one]] e
[[feedback-the-door-with-the-right-law-had-no-caller-and-the-consumer-used-a-third]].
