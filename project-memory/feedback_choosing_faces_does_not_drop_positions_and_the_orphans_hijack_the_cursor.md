---
name: feedback-choosing-faces-does-not-drop-positions-and-the-orphans-hijack-the-cursor
description: Recortar uma peça por FACES deixa os vértices da parte descartada como órfãos — e quem procura «o vértice mais próximo daqui» aterra num deles
metadata:
  type: feedback
---

Filtrar faces (`faces.iter().filter(...)`) e passar o **pool inteiro** de posições
ao construtor da malha compila, constrói e **desenha igual**: nenhuma face cita os
vértices da parte deitada fora, logo nenhum índice fica fora de alcance e o
validador aceita calado. O que muda é tudo o que pergunta *«qual é o vértice mais
próximo deste ponto?»*.

Medido (2026-09-14, cena `=42` do pincel de contorno): meia esfera recortada de
uma `uv_sphere(32, 48)` ficava com **1 490 posições para 768 faces — 721 órfãos**.
O cursor da cena é *o ponto mais alto da peça*, e o ponto mais alto passou a ser o
**pólo norte da metade que não existe**. A busca da âncora devolvia esse órfão,
ele não tem aresta nenhuma (portanto não é de borda), e a lei recusava o traço
inteiro: **0 vértices movidos**, com a queixa a apontar para o **verbo**.

**Why:** o defeito é **mudo em toda régua estrutural** — `V−E+F` não o vê, o
bordo não o vê, o desenho não o vê. Ele só aparece através de um consumidor que
faz busca espacial, e aí o sintoma já não se parece com a causa; e o órfão também
infla o octree, a caixa do mundo e qualquer janela por-vértice (undo incluído).

**How to apply:** ao recortar uma peça por faces, compacte **sempre** — e por uma
PORTA que devolva também *de onde cada posição veio*, para os canais paralelos
(cor, máscara, vectores por vértice de quem chama) seguirem sem uma segunda cópia
da aritmética. Neste repo é `ph2d_mesh::compact_for_faces`. ⚠️ A lei já estava
escrita, com este mecanismo no doc, **privada** dentro do importador de OBJ —
*uma lei privada é como o segundo consumidor a reescreve mal*. Ver
[[feedback_a_gate_only_proves_what_its_fixture_contains]].
