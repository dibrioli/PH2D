---
name: feedback-a-hard-clamp-is-not-a-ceiling-it-is-an-eraser
description: Um clamp mapeia TUDO acima do limite no MESMO valor — então detalhe acima dele não é comprimido, é APAGADO; e se o consumidor lê DERIVADA (gradiente/slope), o platô resultante renderiza como NADA
metadata:
  type: feedback
---

O impasto tinha `H_CEIL = 2.0` e um `h.clamp(-2, 2)` no buffer. O doc dele citava a Corel — *"the
accumulated artwork will begin to **top out** and appear as if the strokes are pressed against glass"* —
**topar gradualmente**. Um `clamp` não faz isso.

O Enio esculpiu, mandou print e a frase: **"em 3 pinceladas toda escultura é achatada no teto"**. A borda
(abaixo do teto) linda; o miolo, uma chapa morta.

**Why:** o clamp mapeia **todos** os valores acima do teto no **MESMO número**. Logo:
- as marcas de pincel do topo não são comprimidas — são **apagadas** (viram todas `2.0`);
- um platô de constante tem **gradiente ZERO**;
- e a luz **lê gradiente** (`∇h × DEPTH_UNIT_PX`). Sem gradiente, não há o que desenhar.

**Justo onde o artista mais trabalhou, o render mostra nada.** O clamp não é vidro; é borracha.

**How to apply:**
- **Todo limite superior deve ser uma COMPRESSÃO ASSINTÓTICA, não um clamp** — a menos que colapsar o
  detalhe acima dele seja *literalmente o que você quer*. `soft(h) = knee + span·t/(1+t)`, `t=(h−knee)/span`:
  C¹, monótona, limitada, algébrica (sem transcendental), e **identidade abaixo do joelho** ⇒ toda arte
  anterior fica **byte-idêntica** (é isso que permite introduzir a mudança sem repintar o trabalho do
  usuário — e é gate).
- **Pergunte o que o consumidor LÊ.** Se ele lê **derivada** (luz, normal, slope, contorno), achatar o dado
  não "limita" — **deleta**. Se lê o valor cru, um clamp só satura.
- **O teto é da APARÊNCIA, não do DADO.** Aplique no render (porta única), nunca no buffer: assim as
  ferramentas que raciocinam sobre geometria (ajuste de plano, offset por bola, blur) operam sobre a
  superfície que o artista de fato construiu. Um clamp no buffer **corrompe a geometria** que elas leem.
- Gate obrigatório: *duas alturas acima do teto que diferem por uma marca de pincel têm de continuar
  diferentes DEPOIS do teto* — e a versão de aparência: *uma crista lá em cima ainda ACENDE*.
