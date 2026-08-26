---
name: a-mode-whose-entry-excludes-nobody-is-just-another-producer
description: Declarar «já existe um modo» sem medir o que ele EXCLUI dá dois consumidores da mesma tecla
metadata:
  type: feedback
---

Vector / máquina de estados do Morph, 2026-08-25. A wave anterior tinha escrito, com
todas as letras, que não era preciso um modo novo:

> *"E o modo já existe: neste editor, «o jogo a correr» é o **playhead a andar** — a mesma
> porta pela qual o dedo do jogador alcança a física. Uma terceira noção de runtime seria
> uma terceira coisa para o artista aprender."*

O smoke do Enio: *"precisamos de um modo preview (com botão) (…) pois senão temos
conflitos de atalhos (como setas do teclado movendo as formas)"*.

**Why:** o argumento era bom sobre a coisa errada. **O playhead não tranca o teclado do
editor** — com ele a andar, as teclas continuam a chegar a todos os atalhos. Então a mesma
tecla fazia duas coisas, que é precisamente o que aquela nota dizia estar a evitar. *Um
modo cuja entrada não EXCLUI os outros consumidores não é um modo — é mais um produtor.*

**How to apply:**
- Antes de escrever *«já existe um modo para isto»*, pergunte **o que esse modo exclui** e
  vá ver. Um modo define-se pelo que ele cala, nunca pelo que ele liga.
- Um dispositivo de entrada de cada vez: o modo que toma o **rato** não toma o teclado, e
  vice-versa. Reutilizar um pelo outro é herdar a exclusão errada.
- A guarda que toma a entrada vive entre **duas** fronteiras, e as duas são load-bearing:
  DEPOIS do alimentador do estado que o modo lê (barrar antes deixa o modo inerte **com** a
  entrada tomada — o pior dos dois mundos) e ANTES do primeiro consumidor do editor. Escreva
  gate sobre a ORDEM, não só sobre a existência.
- A porta de saída tem de ser anunciada **e** continuar clicável: um modo que toma o teclado
  come exactamente as teclas com que o artista tentaria escapar dele.
- ⛔ Uma porta, não duas. Deixar a antiga a dirigir também mantém o conflito viva na porta
  que não tranca nada.

Ver [[feedback-a-shared-section-header-is-a-regression-to-whoever-arrived-first]] ·
[[feedback-a-parameter-that-changes-nothing-is-discarded-downstream]] ·
[[reference-topic-authored-state-and-clocks]]
