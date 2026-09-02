---
name: feedback-a-per-axis-feature-is-conjugation-not-a-law-per-axis
description: "«Ponha os 3 eixos como opção»: escreva UMA lei conjugada (P⁻¹∘f∘P), com P cíclica — e conjugue também a bola de bordo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T23:11:48.319Z
---

Quando um operador ganha *«em que eixo?»*, a resposta **não** é uma lei por eixo — é **conjugar** a
que já existe: `f_A = P⁻¹ ∘ f_canónico ∘ P`. Para operadores que remapeiam o domínio isso são dois
remapeamentos a mais, e para o eixo de omissão é a **identidade ao bit** (nem um nó a mais na
árvore), que é o que faz a feature não ser uma migração.

Caso medido (PH2D, `line/3DModeling`, 2026-08-31, pedido do Enio): cinco modificadores (`Array` em
X, `Taper` em Y, `Radial`/`Twist`/`Bend` em Z) ganharam o eixo por uma função de conjugação só.

**How to apply:**
- ⛔⛔ **`P` tem de ser CÍCLICA, nunca uma troca de dois eixos.** Uma troca tem determinante `−1`:
  ela **espelha** a peça, e uma torção espelhada gira ao contrário. Para qualquer par `(de, para)`
  existe exactamente uma rotação cíclica que serve. Gate: o produto vectorial dos dois primeiros
  eixos permutados tem de dar o terceiro.
- ⚠️ **Conjugue TUDO o que lê coordenadas por ÍNDICE**, não só a lei: aqui a bola de bordo
  (`center[0]`, `hypot(center[0], center[1])`) alimenta as cercas e os divisores, e um índice lido no
  eixo errado dá um raio pequeno de menos ⇒ um divisor pequeno de menos ⇒ **o campo fura**. Uma
  permutação, três leitores, uma porta.
- ⭐ **O eixo de nascimento tem de ser o eixo CANÓNICO**, e os dois têm de ser a **mesma constante**:
  um faz toda peça já gravada ler-se ao bit, o outro é o referencial da conjugação. Escritos em dois
  sítios, um dia divergem e o eixo de omissão passa a rodar a peça.
- ⚠️ **Num formato posicional** (postcard), o campo novo vai no **fim** de cada variante e nunca no
  meio — e colapsar variantes irmãs (três espelhos → um campo) **apaga variantes do meio do enum**,
  o que reescreve o significado de tudo o que está gravado, em silêncio
  ([[feedback_removing_a_middle_variant_from_a_serialized_enum_silently_rewrites_saved_files]]).

**A régua, e a que eu escrevi errado primeiro:** são **três** perguntas independentes — *(1) o
default não muda um bit · (2) outro eixo MUDA a peça · (3) a lei noutro eixo é a canónica
conjugada*. ⛔ A (3) na forma *«a peça no eixo A é a peça canónica rodada»* é **FALSA** e reprova
sobre código correcto: mudar o eixo do deformador **não roda a peça de entrada**. A comparação certa
põe a rotação nos **dois** lados (a primitiva com as meias-extensões permutadas). *Uma régua que roda
só um dos lados mede a rotação, não a conjugação.*
⭐ E a fixtura tem de ser **assimétrica nos três eixos**: numa peça cúbica um controlo morto passa em
(2). Inverter a permutação deixa (1) e (2) verdes e mata só a (3) — é essa a prova de mutação.
