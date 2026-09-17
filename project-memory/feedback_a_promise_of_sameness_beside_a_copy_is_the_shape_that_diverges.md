---
name: feedback-a-promise-of-sameness-beside-a-copy-is-the-shape-that-diverges
description: "Um comentário que promete «é a MESMA conta que X faz» escrito ao lado de uma CÓPIA é o cheiro de uma lei duplicada — e a promessa faz o leitor não procurar a porta"
metadata:
  type: feedback
---

Medido em 2026-09-17 (`W5` do render, `ph2d-field-render`): a conversão da normal de espaço de
VISTA para MUNDO estava escrita **à mão em três sítios** — o passe da sombra, o da oclusão e uma
sonda dos testes — e eu ia escrever a quarta.

⭐ **O achado não é a duplicação: é que os três a DECLARAVAM.** O comentário do passe da sombra diz,
por extenso:

> *«é a MESMA conversão que o `shade_render` faz para o `N·L`, senão a sombra e a luz discordariam
> sobre quem vê quem»*

— escrito **ao lado da cópia**. A promessa lê-se como se a lei fosse partilhada, e é exactamente o
que impede quem passa por ali de ir procurar a porta.

**Why:** uma cópia anónima é achada por `grep`; uma cópia **com a promessa ao lado** é lida como
*«isto já está tratado»*. E ela diverge no dia em que alguém corrigir **uma** delas — que é
precisamente o dia em que a promessa passa a ser falsa e ninguém reprova.

**How to apply:** trate *«é a mesma X que Y faz»* num comentário como **acusação**, não como
tranquilização: se é a mesma, tem de ser uma chamada. A cura é uma porta com N chamadores, e ela é
barata quando a expressão é idêntica (aqui, a mesma ordem de operações ⇒ os dois passes de produto
ficaram **bit a bit iguais**, com a suíte a prová-lo). ⚠️ E hoiste-a para fora do laço: a porta
vizinha (`world_to_view`) já avisava por escrito que *«uma base reconstruída dentro do laço corre
onde o laço corre»*. Ver [[feedback-the-door-with-the-right-law-had-no-caller-and-the-consumer-used-a-third]]
para a forma irmã (a porta certa existe e ninguém a chama).
