# Plano 36 — **o PINCEL de contorno** (*"a coisa precisa funcionar sem limitações"*)

> Ordem do Enio, 2026-08-28, depois de o [plano 35](35_plano_padrao_no_traco.md) fechar o padrão
> como TINTA: *"mas não posso usar o dash com pattern?"* → *"a coisa precisa funcionar sem
> limitações. Qual estado da arte?"*

---

## §1 — O ESTADO DA ARTE: são **dois modelos**, e todo aplicativo sério entrega os DOIS

### §1.1 — Modelo **A: o padrão é uma TINTA** (papel de parede)

A norma é explícita. SVG 2, *Painting*:

> *"When stroking is performed using a complex paint server, such as a gradient or a **pattern**,
> the stroke operation **must be identical** to the result that would have occurred if the geometric
> shape defined by the geometry of the current graphics element and its associated stroking
> properties were converted to an equivalent `path` element and then **filled using the given paint
> server**."*

⇒ um traço tracejado com padrão é, por definição, **a silhueta do tracejado preenchida com o papel
de parede**. Os traços são BURACOS; a arte não os conhece.

| Quem | Como |
|---|---|
| **SVG / navegadores** | `stroke="url(#pattern)"` + `stroke-dasharray` — normativo acima |
| **Figma** | *"use patterns as a fill or stroke"* — a mesma lista de tintas nos dois |
| **Illustrator** | um *pattern swatch* aplicado ao traço |
| **Inkscape** | *Stroke paint* → padrão |

⭐ **É EXACTAMENTE o que o plano 35 entregou**, e ele está **conforme à norma**. ⛔ O que o Enio
fotografou não era um defeito: era o modelo A a funcionar.

### §1.2 — Modelo **B: a arte PERCORRE o caminho** (pincel)

| Quem | Como | O que custou |
|---|---|---|
| **Illustrator — *Pattern Brush*** | até **CINCO** ladrilhos: *side · outer corner · inner corner · start · end*, mais geração automática de quinas (**Auto-Centered · Auto-Between · Auto-Sliced · Auto-Overlap**) | ⚠️ as quinas são *"a stumbling block"* declarado, só **parcialmente** resolvido pela geração automática — o *Auto-Between* deixa emenda visível. ⛔ E ele *"cannot leave out parts of the brush shape, it can only stretch or shrink it if it doesn't fit"* |
| **Illustrator — pincel + tracejado** | a arte **reinicia em CADA traço** | ⚠️ Fóruns pedem exactamente isto e recebem contornos (um 2.º traço com *knockout group*) — a combinação é reconhecidamente desajeitada |
| **Inkscape — *Pattern Along Path* (LPE)** | repetir **ou** esticar; *normal offset* e *tangential offset*; motivo e guia continuam editáveis | ⛔ **um caminho só** (sem grupos), e **não afina** (*taper*) |
| **Affinity — *Vector Brush*** | aceita grupos, sombreado, sobreposição | — |

### §1.3 — ⭐⭐ A síntese, e é ela que responde ao Enio

**Nenhum aplicativo trata isto como um botão.** São **duas ferramentas**: uma TINTA que o contorno
revela, e um PINCEL que corre ao longo dele. *"Sem limitações"* quer dizer **ter as duas, e poder
escolher qual**.

⛔ E é por isso que a resposta *"diminua o Width da estampa"* que eu dei estava errada: ela é um
contorno para a ausência do modelo B, não uma propriedade do modelo A.

---

## §2 — Onde NÓS estamos (medido, não estimado)

| Modelo | Estado nesta casa |
|---|---|
| **A — tinta** | ✅ **entregue** (plano 35): preenchimento e traço, `Tile/Mirror/Clamp`, reticulados, fase, ângulo, vão, e a arte a viajar no ficheiro |
| **B — pincel** | ⚠️ **o MOTOR existe e está medido; o PINCEL não** |

O motor do B é o [Pattern Along Path](23_plano_pattern_along_path.md) (plano 23): cópias **rígidas**
do motivo sobre o comprimento de arco, cada uma rodada para a tangente, sem *refit* — **0,597 ms
para 200 cópias × 40 vértices**, ~13× de folga sob o *kill* de 8 ms.

⛔ **Mas ele é endereçado como uma RELAÇÃO ENTRE DOIS OBJECTOS** (`VecPatternPath { path }`: *"este
motivo cavalga aquela guia"*), e não como uma **propriedade do traço**. ⇒ o artista **não consegue
dizer *"desenha este contorno com esta arte"***, que é a frase inteira da feature.

⇒ **O buraco, numa frase:** falta o **pincel**. O motor está pago.

---

## §3 — O DESENHO

### §3.1 — A fileira *Type* do traço ganha a terceira opção

```
Solid | Pattern | Brush
```

- **`Solid`** — uma cor (hoje).
- **`Pattern`** — a TINTA que o contorno revela (hoje; modelo A, conforme à norma).
- **`Brush`** — a arte **percorre** o contorno (modelo B), pelo motor do plano 23.

⭐ **A escolha da arte é a MESMA porta**: `Source…` (ficheiro) e `Use Shape…` (uma forma do
documento). *Um pincel novo não é um fluxo novo — é a mesma arte noutra lei.*

### §3.2 — ⭐⭐ O pincel corre no PRÓPRIO contorno, sem segundo objecto

O plano 23 exige uma **guia** (outro `VecPath`). Aqui a guia é o **próprio caminho da forma** — o
mesmo que o traço já percorre. ⇒ nada de vínculo, nada de segunda entidade, nada de `stable_name_id`.

⚠️ **E a `LiveGeometry` é o sujeito**, não a fonte autorada: o pincel corre sobre o que a forma É
depois dos efeitos vivos (cantos vivos, largura viva, booleana), pela mesma razão que o traço.

### §3.3 — As três perguntas que o modelo B tem e o A não

| Pergunta | A resposta que o estado da arte dá | A nossa |
|---|---|---|
| **O tracejado?** | Illustrator **reinicia a arte em cada traço** | idem — e é o que o Enio pediu ao juntar as duas coisas |
| **As quinas?** | Illustrator: **ladrilho de quina** (interno/externo) + 4 modos automáticos, e ainda assim *"a stumbling block"* | ⏳ **wave própria**, com os 4 modos MEDIDOS antes de escolher. ⛔ O plano 23 hoje **pula a cópia** numa cúspide — é o buraco conhecido |
| **A emenda de um contorno FECHADO?** | Illustrator estica/encolhe para fechar; Inkscape deixa sobrar | ⭐ **a maquinaria já existe**: a `dash_fit` faz exactamente isto para o tracejado (ajusta ao comprimento para a emenda fechar). *Uma porta, dois consumidores.* |

### §3.4 — ⛔ O que este plano NÃO faz

- **Não deforma** a arte para dobrar na curva (*bending*) — é o **Envelope** (ADR-0129), que já
  existe e **compõe**; o plano 23 §0 já nomeou esta bifurcação.
- **Não substitui o modelo A.** Os dois ficam, e a fileira *Type* é onde se escolhe. *Um aplicativo
  que só tem um dos dois é o que tem limitação.*
- **Não faz os 5 ladrilhos do Illustrator na v1** — *side* primeiro; *start/end/corner* são waves com
  a medição ao lado.

---

## §4 — As waves

| Wave | Entrega | Onde |
|---|---|---|
| **W1** | `StrokePaint::Brush(Box<BrushStroke>)` + o schema (o postcard é posicional ⇒ `VEC_SCENE_SCHEMA_VERSION` +1, `PROJECT_SCHEMA` +1, a tripla) | `ph2d-vec-scene` |
| **W2** | O **motor**: correr o `pattern_along` sobre o próprio contorno da forma, com **fit** de emenda pela porta da `dash_fit` | `ph2d-vec-scene` + `-render` |
| **W3** | O **tracejado**: a arte reinicia em cada traço (a lei do Illustrator), pela porta que já parte o traço em peças (`stroke_plan`) | `ph2d-vec-render` |
| **W4** | A UI: a 3.ª opção da fileira *Type* + a secção **Brush** (tamanho · espaçamento · offset normal/tangencial · flip), irmã das outras duas | `ph2d-panel-vector` + shell |
| **W5** | As **QUINAS**: os 4 modos do Illustrator medidos lado a lado, e o nosso escolhido **com a tabela** | `ph2d-vec-scene` |
| **W6** | Persistência + smoke + gates + mutações | shell |

**Kill-criterion (DIRETIVA §5):** o plano 23 mede **0,597 ms / 200 cópias**. Um contorno de forma
típica pede a mesma ordem. ⇒ **se o re-cook de uma tecla passar de 8 ms, a feature não existe nesta
forma** e o passo seguinte é cache por-params — ⛔ **não** subir o teto.

---

## §5 — ⚠️ A lição que trouxe este plano

Eu respondi ao report do Enio com *"diminua o Width da estampa até caberem várias cópias por
traço"*. Ele devolveu: *"a coisa precisa funcionar sem limitações"*.

⭐ **Ele estava certo, e o defeito era meu:** eu ofereci um **contorno** para a ausência de uma
feature, em vez de ir ver o que o estado da arte faz. A pesquisa levou quatro buscas e mostrou que
*todo* aplicativo sério tem os dois modelos — e que nós já tínhamos o motor do segundo, pago e
medido, há um mês.

*Um workaround oferecido no lugar de uma pesquisa é uma limitação transformada em política.*

---

## ⛔ Recusas MEDIDAS

| O que | Porquê | Onde |
|---|---|---|
| Deformar a arte para dobrar na curva dentro deste plano | é o **Envelope** (ADR-0129), que já existe e compõe | §3.4 |
| Substituir o modelo A pelo B | a norma SVG exige o A, e o Figma entrega-o como *fill or stroke* | §1.1 |
| Os 5 ladrilhos do Illustrator na v1 | as quinas são *"a stumbling block"* declarado até para eles; entram com medição | §3.3 |
| *"Diminua o Width da estampa"* como resposta ao artista | é um contorno para a ausência do modelo B | §5 |
