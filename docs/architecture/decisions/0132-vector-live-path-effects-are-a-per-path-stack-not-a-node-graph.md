# ADR-0132 — Live Path Effects são uma PILHA por-path dentro do `cooked()`, não um grafo de nós — e a quina é o estágio ZERO

**Status:** proposto · **Data:** 2026-07-18 · **Linha:** `line/Vector`
**Pesquisa:** [`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`](../../Vector%20Module/20_pesquisa_ferramentas_de_artista.md) §1 e §5 (o item #1 da lista, "a espinha")
**Precede:** [ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) (fonte≠cozido) · [ADR-0128](0128-vector-blend-object-live-virtual-steps-editable-spine.md) · [ADR-0129](0129-vector-envelope-warp-one-spine-cage-as-container-entity.md) (que se declarou *"o primeiro Live Path Effect"*)
**Não toca:** [ADR-0039](0039-nodegraph-contract-freeze-w2t4.md) (contrato de nós, congelado) — e o §1 explica por que isso **não** foi o motivo da decisão

---

## Contexto

A fila da linha (`HANDOFF_line_vector_continuacao_2026-07-18.md` §4.1) abre em **"Live Path Effects
como NÓS"**, chamado *o multiplicador*. A pesquisa `20_*` §5 é enfática: *"a ferramenta mais
transformadora que podemos construir não é uma ferramenta — é a espinha que faz cada ferramenta
futura custar uma drop-crate"*, e §1 afirma que a arquitetura do Inkscape *"é literalmente um sistema
de nós"*.

O handoff, porém, marca a palavra: *"'Como nós' é a palavra da pesquisa, não uma decisão tomada"*, e
manda decidir **antes de escrever código** — com uma ⚠️ sobre o contrato congelado. Este ADR fecha as
duas perguntas, na ordem em que elas de fato dependem uma da outra.

---

## §1 — A pergunta do contrato: MEDIDA, e ela não bloqueia nada

O handoff temia que *"como nós"* encostasse em `NodeOp=2` / `OpResolver=1` / `NodeManifest=8`. **Não
encosta.** Levantado no fonte:

| Medo | Realidade medida |
|---|---|
| "geometria não trafega em aresta" | **Trafega.** `CookValue::Opaque(Arc<dyn Any + Send + Sync>)` ([`value.rs:38`](../../../crates/ph2d-nodegraph/src/value.rs)) existe **precisamente** para valor de topologia variável que não decompõe em colunas — o padrão Houdini/USD. Acessores `EvalCtx::input_any`/`emit_any` já existem, **fora do cap** |
| "precisaria de um domínio novo" | `Domain::Vector` já está no vocabulário congelado ([`port.rs:17`](../../../crates/ph2d-nodegraph/src/port.rs)) |
| "param não-`f32` obrigaria bump" | Dois escapes já shipados: o canal de **TEXT PARAM** (`Graph::set_text_param`, params no `Graph`) e a convenção de **discriminante `f32`** + `ParamWidget::Enum`, que guarda o índice da opção no próprio `f32` — os nós vetoriais aposentados já faziam isso (`kind`: `0`=Rect … `4`=Spiral) |

O próprio substrato escreve a conclusão: *"the frozen node-author contract … is untouched:
`CookValue` is substrate-internal cook plumbing, not a node crate's surface."*

**Isto é registrado por uma razão de método, não de conveniência:** a decisão do §2 **não é** "não
usar nós porque o contrato proíbe". O contrato **permite**. A escolha é livre, e por isso tem de se
defender pelo desenho. Se um dia alguém quiser o nó, o caminho está aberto e **este ADR o mantém
aberto de propósito** (§4).

> ⚠️ Uma ressalva honesta para quem for por ali: a doc de `Domain::Vector` nomeia `VectorNetwork` —
> o modelo de dados **aposentado** pelo [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md),
> não o `VecPath` de hoje. A faixa está **vazia**: varredura no workspace inteiro devolve **dois**
> usos, ambos cosméticos (um token de cor e o colorizador de porta do painel de grafo). Nenhum nó
> produz ou consome geometria. Reanimar a faixa exige re-documentá-la, não bumpá-la.

---

## §2 — A decisão: uma PILHA por-path, avaliada dentro do `cooked()`

Um Live Path Effect é uma **função pura `VecPath -> VecPath`**, e um path carrega uma **lista
ordenada** delas. A avaliação acontece onde a geometria derivada já é servida hoje: `VecPath::cooked()`.

```
verts autorados → [estágio 0: quina] → efeito₁ → efeito₂ → … → mundo
```

Que é, letra por letra, o pipeline que o **ADR-0129 §3 já havia declarado inegociável**. Este ADR não
inventa a forma; ele constrói o que faltava dela e nomeia por que a forma é uma **lista** e não um
grafo.

### Por que não nós

1. **Uma pilha não é um grafo — e o LPE é uma pilha em toda ferramenta shipada.** O Inkscape expõe
   uma **lista** na caixa de diálogo do objeto selecionado (empilha, reordena por arrastar, achata os
   N primeiros). Os *path operators* do After Effects são uma **lista dentro da shape layer**. Os
   *Behaviours* do Cavalry são uma **lista no objeto**. A pesquisa acertou a **forma do dado**
   (`Piecewise → Piecewise`, encadeado) e a chamou de "sistema de nós" pela topologia do fluxo — mas
   a topologia de uma pilha é uma **corrente**, e representar corrente com motor de DAG é pagar por
   ramificação que ninguém pediu.
2. **O grafo de nós é uma tela; o artista de vetor está no CANVAS.** Pedir que ele abra um editor de
   grafo para arredondar quina ou revelar um traço é trocar o gesto por uma segunda superfície. É
   exatamente a crítica que a Fase C do ADR-0128 pagou com 5 tentativas revertidas.
3. **Não existe grafo POR objeto.** O Motion Nodes tem **um** grafo para a cena. LPE por-path exigiria
   *N* subgrafos, com formato de save, ciclo de vida e undo próprios — um mecanismo novo **maior** que
   a pilha que ele hospedaria.
4. **`Cow::Borrowed` é a propriedade que sustenta tudo, e ela é trivial numa lista.** Pilha vazia +
   raio zero = **mesmo ponteiro, zero alocação** — foi isso que permitiu ligar o `cooked()` em TODO
   consumidor (render, hit-test, bbox, booleana, gradiente) sem mudar comportamento. Pela cozedura de
   um grafo, com chave de memo, o caso vazio deixa de ser um `if`.
5. **É o desenho que já funcionou quatro vezes nesta linha** — Live Shape, conector, Blend, Envelope:
   dado no documento/ECS, cozimento por frame, função pura no meio. Um quinto mecanismo seria a
   **segunda resposta** a *"como uma forma ganha geometria derivada"*, e a §5 mostra que ter
   **quatro** respostas já produziu um bug silencioso.

---

## §3 — O estágio ZERO: por que a quina é fixa, e por que zerar o raio depois está CERTO

Hoje, `envelope` e `blend` escrevem `corner_radius: 0.0` na saída (4 sítios:
[`ph2d-vec-envelope/src/lib.rs:229,239`](../../../crates/ph2d-vec-envelope/src/lib.rs),
[`ph2d-vec-blend/src/lib.rs:354,364`](../../../crates/ph2d-vec-blend/src/lib.rs)). A leitura fácil é
*"então os cozimentos não são endomorfismos, e uma pilha exige que sejam"*. **Essa leitura está
errada, e a distinção é a espinha deste ADR.**

Arredondar quina é uma operação sobre o **vértice AUTORADO**: o raio mora dentro do vértice, e
`round_authored_corners` **divide uma quina em dois vértices**. Envelope e blend **resampleiam** (o
envelope subdivide por tolerância; o blend corta na união das posições de âncora) — a **contagem de
vértices é saída deles**, não entrada. Não há para onde levar o raio da quina que deixou de existir.

Este argumento não é novo aqui: é literalmente o que
[`corner_handles.rs:16-19`](../../../shells/desktop/src/corner_handles.rs) já escreve para as Live
Shapes — *"a CONTAGEM de vértices é função dos parâmetros … Não há para onde levar o raio da quina que
deixou de existir."*

**Consequência de projeto, e ela simplifica:**

- A quina é o **estágio 0**, fixo e primeiro. Ele consome estado autorado por-vértice e emite
  **geometria plana**.
- A **pilha opera sobre geometria plana**. Um efeito da pilha é `VecPath -> VecPath` sem contrato
  sobre `corner_radius` da saída.
- Portanto **zerar o raio a jusante é correto**, e os 4 sítios ficam como estão.

---

## §4 — A fronteira: efeito por-path vs. objeto RELACIONAL

A linha tem hoje dois tipos de coisa viva, e confundi-los é o que faria a pilha inchar:

| | Pergunta que responde | Onde vive |
|---|---|---|
| **Efeito por-path** (quina, trim, zig-zag, roughen, offset, contour) | função de **UM** path | a **pilha**, no `VecPath` |
| **Objeto relacional** (Blend, Envelope, conector, rótulo) | função de **VÁRIOS** paths, ou de um **container** | entidade + componente ECS, como hoje |

Um envelope tem **uma gaiola compartilhada por N filhos**; um blend **relaciona 2..=5 fontes**. Nada
disso cabe numa lista pendurada num path, e tentar encaixá-los seria refazer três sistemas shipados
para provar uma tese. **Eles ficam onde estão.**

**E o caminho do nó fica aberto sem custo:** porque cada efeito nasce como **função pura numa crate
própria**, um nó que o embrulhe é `manifest()` + `eval()` lendo `input_any` e chamando a mesma
função. Se o dia do grafo chegar, ele não reimplementa nada — ele **envelopa**. Escolher a pilha hoje
não fecha a porta; escolher o grafo hoje construiria a pilha assim mesmo, por dentro, e com uma tela
a mais.

---

## §5 — O bug vivo que este ADR obriga a fechar

A pergunta *"quem pode oferecer a alça de raio?"* tem **uma** resposta correta — *quem autora os
vértices* — e o código faz **outra** pergunta:
[`corner_handles.rs:62`](../../../shells/desktop/src/corner_handles.rs) recusa **só**
`is_live_shape`.

Um **filho de envelope** não carrega `VecShape`. Logo **ganha a alça** — e
`envelope_live::recook` reescreve `verts` a cada frame a partir do `source` congelado dentro do
componente. **O raio autorado sobrevive exatamente um frame, e some sem erro nenhum.**

É, palavra por palavra, o modo de falha que o mesmo arquivo documenta em 19 linhas para as Live
Shapes — *"pior que não funcionar: funciona, o usuário confia, e um arrasto … desfaz o trabalho dele
em silêncio"* — reproduzido no vizinho, **sem o guard**.

A causa raiz é a que o §2.5 nomeia: **quatro respostas para "onde mora a geometria autorada"**, e um
guard que enumerou **uma** delas. A correção não é somar um `||`: é perguntar **a coisa certa uma vez
só** — *este path tem geometria DERIVADA?* — por porta única, e gateá-la por-caso.

---

## Consequências

**Boas**
- A espinha do item #1 fica construída, e cada efeito da Faixa A/B da pesquisa passa a custar uma
  **drop-crate pura** — testável sem shell, sem GPU, sem janela.
- Undo, save e keyframes vêm de graça: a pilha é dado de documento, e o undo global já é diff de
  `ProjectState`.
- `Cow::Borrowed` sobrevive: documento sem efeito nenhum passa pelo `cooked()` sem alocar.
- O bug do §5 fecha **com** a espinha, não depois dela.

**Ruins / aceitas**
- **Sem ramificação e sem reuso**: a saída de um efeito não alimenta duas formas. É o preço de
  escolher corrente em vez de DAG — e é o que Inkscape/AE/Cavalry também não oferecem.
- **`VEC_SCENE_SCHEMA_VERSION` 8 → 9** (postcard é posicional). Save v8 não carrega.
- A **ordem** da pilha é do artista, e ordens diferentes dão desenhos diferentes. Isso é feature
  (é o que "reordenar por arrastar" significa), mas é superfície de confusão nova.
- O **Flatten** do Inkscape (assar só os N primeiros) fica fora deste corte.

---

## Aceitação

1. Pilha vazia é **`Cow::Borrowed`** — mesmo ponteiro, zero alocação (gate de identidade de ponteiro).
2. Documento sem efeito é **byte-idêntico** ao de hoje em render, hit-test, bbox, booleana e gradiente.
3. A alça de raio **não é oferecida** sobre geometria derivada — Live Shape **e** filho de envelope —
   pela **mesma** função, com gate por-caso (a mutação de um caso não pode ficar verde pelo outro:
   [[feedback_layered_defenses_need_per_layer_gates]]).
4. Um efeito no ponto neutro é **no-op byte-idêntico** (o invariante que a rack de áudio provou valer
   a pena: 42 efeitos, 5 gates por-efeito).
5. A ordem da pilha é honrada: `[A, B]` e `[B, A]` produzem geometrias diferentes onde as operações
   não comutam, e o gate **contém** um par que não comuta.
