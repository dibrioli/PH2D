---
name: two-engines-that-both-ignore-a-feature-agree-perfectly
description: Paridade é uma RELAÇÃO, não uma propriedade — e uma FRACÇÃO sobre a imagem afoga o fenómeno que ocupa 10 % dela; o discriminador é o EXTREMO
metadata:
  type: feedback
---

Um gate de paridade entre dois motores afirma *«os dois dizem o mesmo»*. Ele **nunca** afirma
*«e a coisa está lá»*. ⇒ **dois motores que ignoram a mesma feature são a configuração mais verde
que existe.**

**Medido (PH2D, Render3d W10, 2026-09-19), e a 1.ª cura NÃO chegou.** A borda mole da sombra só
corria no traçado de CPU; ao levá-la ao dispositivo escrevi um gate de paridade com **duas**
metades — *(a)* o canal mole afasta-se do duro em `> 500` píxeis (medido `2 043`) e *(b)* os dois
motores concordam a `≥ 99,5 %` dentro de `1` nível. A prova de mutação derrubou-o:

| com o dispositivo a NÃO pedir o canal | leitura | veredito |
|---|---:|---|
| fracção `≤ 1` nível (barra `99,5 %`) | **`99,579 %`** | **passa** — `0,079` acima da barra |
| pior byte | **`12`** (contra `1` na árvore que ship) | reprova, se alguém o medir |
| quebra na banda do terminador | **`9,20`** (referência `1,00`) | reprova, e é o que o dono vê |

**Why:** a metade *(a)* prova que o canal se move **no buffer**, não que ele chega ao **pixel**; e
uma **fracção** sobre a imagem inteira afoga na média um fenómeno que ocupa `10 %` dos píxeis. *O
extremo não afoga* — e uma régua desenhada para a feição (aqui a segunda diferença da luminância na
banda do terminador) separa `1,00` de `9,20` com um vale de `9×`.

**How to apply:** um gate de paridade precisa de **três** coisas, não duas — os dois lados
concordam · o lado que eu controlo produz o fenómeno · e a régua que decide é **sensível à feição**
(um extremo, ou a grandeza que o report nomeia), nunca só uma média. ⛔ E se a condição de fecho da
wave está escrita como uma **tabela impressa** («as duas colunas lêem o mesmo»), ela ainda não é um
gate: *uma tabela que passa não é lida por ninguém*.

Irmãs: [[a-parity-measured-upstream-of-a-conversion-says-nothing-about-it]] ·
[[a-fixed-scene-no-longer-contains-the-phenomenon]] ·
[[a-ruler-that-only-sees-the-sign-does-not-see-the-magnitude]]
