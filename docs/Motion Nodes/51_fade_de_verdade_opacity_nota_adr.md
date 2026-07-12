# 51 — **Desvanecer de verdade**: o canal Opacity — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **O4** (parte 4 — smoke do Enio)
**Status:** implementado, testado, **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** nenhum

---

## 1. O smoke

> *"desvanece … ficou claro mas não menor nem transparente. faça direito"*

Estava certo. O doc 50 colore o floco pela idade (rampa Ice) — e **clarear não é desvanecer**. Desvanecer é
**encolher** e **ficar transparente**. E aí apareceu o buraco: o `motion.drive` sabia dirigir **X · Y · Rotação ·
Tamanho** e **não a OPACIDADE**. A biblioteca inteira não tinha caminho para "some aos poucos".

## 2. O canal que faltava

`motion.drive` ganha o canal **Opacity (4)**: escreve o **alpha da coluna `tint`** (o renderer alpha-blenda; o
`tint` é rgba e chega ao lowering).

- **Sem `tint` = branco opaco.** Dirigir a opacidade de um stream que ninguém coloriu faz **exatamente o que diz**,
  em vez de silenciosamente não fazer nada.
- **Alpha é clampado em [0,1].** Um alpha de 1.4 ou −0.2 não é uma partícula mais brilhante nem mais escura: é uma
  partícula que **lê como bug**.

## 3. A neve, agora: colore · **encolhe** · **some**

`sim.lifetime` escreve `life` (0→1) → `value.attribute` a lê → `value.map_range` a inverte (1 → 0.1) → o **mesmo**
valor dirige **três** coisas: a rampa (cor), o `drive(Size)` e o `drive(Opacity)`.

**O piso não é zero** (`0.1`): um floco que encolhesse a zero **um frame antes de morrer** pipoca — lê como glitch,
não como derretimento.

### `Set`, **não** `Multiply` — e isto é o erro que teria passado despercebido

`size` e `tint` **viajam no ESTADO**. Um drive multiplicativo **compõe a cada tick**: em quatro segundos os flocos
estariam em `1e-30` — um fade que é função do **número de frames**, não da **idade**. E ele **passaria** num teste
que só perguntasse *"o velho está diferente do novo?"*. `Set` é idempotente: o canal é função pura da vida do
floco, re-derivada todo tick. Guarda: `sizes.iter().all(|q| q[0] > 0.01)`.

## 4. O verde que eu não entendi (e por isso não aceitei)

A primeira rodada da guarda **passou** — e a matemática não fechava (o `motion.scale` do render deveria multiplicar
por 0.18 e a asserção que escrevi seria impossível). Medi: o meu `replace` de patch **não casou** com o arquivo (o
`fmt` já tinha reformatado a âncora) e **as asserções novas nunca entraram no arquivo**. O teste passou porque era
**o teste antigo**.

> **Verde que você não sabe explicar não é verde.** Foi a aritmética — não o runner — que pegou.

## 5. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| `motion.drive` | canal **4 = Opacity** (`channel::CH_OPACITY`, escreve `tint.a`; sem tint → branco opaco; clamp [0,1]) |
| shell | a neve dirige **cor + tamanho + opacidade** pela mesma fração de vida (`value.map_range` inverte) |
| gates | produto: o floco mais velho é o **menor** E o mais **fraco**; e nada compôs abaixo do piso (`Set` ≠ `Multiply`) |
