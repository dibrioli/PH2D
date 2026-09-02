---
name: feedback-a-control-range-derived-from-the-camera-moves-under-a-gesture-that-changes-nothing
description: "Faixa de slider tirada do enquadramento: o zoom mexe em todos os controlos do objeto — derive da PEÇA, e em oitavas"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T17:50:13.478Z
---

Uma faixa de controlo tirada do **enquadramento** (`camera.half_extent`, o que cabe no quadro) faz
um gesto que **não toca no objecto** — a roda — mexer em **todos** os controlos dele. Quem estiver a
arrastar um vê o número mudar de escala debaixo do dedo, e ajustar a mesma coisa com dois
enquadramentos dá dois resultados.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30, report do Enio em duas metades):
*«o ZOOM muda os parâmetros do objeto no painel»* e *«Bend não funcionou e esticou a peça»* — **o
mesmo defeito**. A banda da dobra é uma POSIÇÃO (faixa aberta), logo a faixa vinha inteira da
câmera: afastado, um arrasto minúsculo punha a banda fora da peça (*«não funciona»*) ou dava um
falloff enorme (*«estica»*).

⚠️ **A razão original era boa** e estava escrita: *«uma dimensão maior do que o quadro é uma cujo
efeito não se vê»*. Uma razão boa não salva uma consequência inaceitável.

**How to apply:**
- Derive a faixa da **PEÇA**, nunca da vista. Uma faixa que qualquer gesto de câmera move é uma
  affordance que mente.
- ⭐⭐ **E em OITAVAS** (potência de dois acima do alvo), senão vem o defeito **espelhado**: uma faixa
  contínua na peça muda enquanto se arrasta uma largura, e o botão foge do dedo
  ([[feedback_a_knob_whose_range_is_derived_from_the_object_it_rewrites_is_not_idempotent]]).
  Dentro de uma oitava a faixa é constante; quando muda, muda **uma vez, entre gestos**.
- ⚠️ **Um piso**, senão uma peça minúscula dá um curso inteiro invisível.
- ⭐ A régua do gate é o **TEXTO da chamada**, não um número: o defeito é uma *dependência*, e uma
  dependência lê-se na chamada — um gate de valor precisaria de duas câmeras e de um quadro inteiro
  para dizer o mesmo.
- ⚠️ E a **fixtura** do gate da oitava tem de cair dentro de uma: a 1.ª versão metia um raio da
  oitava seguinte no meio da lista e reprovou sobre código correcto
  ([[reference_topic_fixture_discipline]]).
