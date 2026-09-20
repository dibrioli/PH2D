---
name: feedback-an-off-knob-implemented-as-an-index-still-reflects-at-grazing
description: Um botao DESLIGADO cuja implementacao e' modular um INDICE deixa o lobo vivo no rasante — e a mesma superficie devolve 0,0000 a uma lampada e 0,41 ao ceu
metadata:
  type: feedback
---

⛔⛔⛔ **Um botão «desligado» que é implementado modulando um ÍNDICE em vez de multiplicar o lobo
continua a reflectir no RASANTE — e nenhuma paridade contra o oráculo o acusa, porque o oráculo faz
igual.**

Medido 2026-09-19 (`line/3DModeling`, report do dono: *«temos um tipo de rim sem que o rim esteja
ligado. Isso é o normal? Veja que no blender não há isso»*). A peça fotografada é autorada com
`specular_weight: 0` e o comentário que a escreve diz porquê. O `specular_weight` do OpenPBR **não
multiplica o lobo**: ele modula o índice de refracção, e a zero o índice fica exactamente `1` — um
interface que **não existe**. A partir daí os dois caminhos da MESMA superfície respondem coisas
opostas:

| caminho | o que ele usa | `η = 1`, `n·v = 0,405` |
|---|---|---:|
| LUZ directa | a Fresnel **exacta** | **`0,0000`** |
| CÉU indirecto | o albedo direccional com **`F90 = 1` cravado** | **`0,0702`**, e `0,41` no rasante |

⇒ *uma superfície que não reflecte uma lâmpada não pode reflectir o céu*, e no produto isso é um
rebordo claro ao longo de toda a silhueta de um material cujo realce o artista desligou.

⭐⭐ **A porta era FIEL: o defeito é da lei de origem** (MaterialX 1.39.5, Apache-2.0, corrido nesta
máquina — o `F90 = 1.0` está cravado nos dois ramos dele). ⇒ a cura é **divergência DECLARADA** com
gate, e não um bug de porte.

⭐ **A cura tem de ser derivada do próprio índice** (`1` para todo interface, `0` quando não há
interface), e então **todo material com um índice a sério fica byte a byte o mesmo** — as duas
paridades contra o oráculo ficaram verdes sem serem tocadas.

⚠️⚠️ **O gate achou DUAS metades que o report não continha:** o `throughput` do lobo desligado
**comia `37,1 %` do difuso no rasante** (energia destruída por uma camada que nunca devolve nada), e
a própria Fresnel **exacta** devolve `1,0` no rasante com `η = 1`, porque `η·η + c·c − 1` **cancela**
em `f32` (a `c = 1e-4`, `1 + 1e-8` arredonda para `1`). *Um limite tomado com um epsilon dentro de
uma expressão que cancela mede o cancelamento, não o limite* — a guarda é uma IDENTIDADE.

**Why:** três das quatro leituras plausíveis deste sintoma estão erradas, e as réguas que existiam
não separavam nenhuma delas. Eu diagnostiquei duas causas falsas antes desta (a sombra a mentir, e o
chão que devia tapar o céu por baixo), cada uma com uma cura construída inteira.

**How to apply:**
- Quando um knob «desligado» ainda produz efeito, pergunte **como é que o zero é implementado**. Se
  ele entra por um parâmetro físico (um índice, um IOR, uma escala), a aproximação a jusante pode não
  saber que a resposta certa passou a ser zero.
- A régua que separa o difuso do especular **sem modelo nenhum** é partir o próprio ambiente:
  `irradiance` alimenta um lobo e `radiance` o outro, logo desligar uma metade isola o termo.
  ⛔ Desligar a LUZ INTEIRA mede «a luz» e não diz qual lobo.
- Compare **os dois caminhos da mesma superfície no mesmo ângulo** (uma lâmpada na direcção
  espelhada contra o céu): a discordância entre eles é o achado.
- ⚠️ E isole os lobos na régua: uma chamada que devolve difuso **e** especular numa soma não pode
  afirmar que um deles está desligado (a 1.ª redacção do gate leu `0,017507` = `albedo/π`, o difuso,
  e acusou-o de reflexo).

Irmãos: [[feedback-an-edge-pixel-asks-the-centre-for-what-the-centre-does-not-have]] ·
[[reference-topic-control-design-hazards]] · [[reference-topic-measurement-discipline]]
