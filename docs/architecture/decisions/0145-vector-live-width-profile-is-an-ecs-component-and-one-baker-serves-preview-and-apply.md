# ADR-0145 — A largura variável é um COMPONENTE ECS, e UM motor serve o preview e o Apply

- **Status:** aceito (2026-07-29)
- **Contexto:** `line/Vector`, plano [`25_plano_ferramentas_de_desenho.md`](../../Vector%20Module/25_plano_ferramentas_de_desenho.md) §4 (W1, A MÃO) e §5 (W2, o Width Tool)
- **Decisão do Enio:** *"Width Tool completo — vamos criar"* (2026-07-29)
- **Relacionados:** [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md) (o norte do módulo) · [ADR-0121](0121-vector-live-corners-authored-source-cooked-geometry.md) (fonte ≠ cozido) · [ADR-0128](0128-vector-blend-object-live-steps-are-virtual-and-the-spine-is-the-scene.md) (*uma porta só produz um passo*) · [ADR-0132](0132-vector-live-path-effects-are-a-stack-on-the-path.md) (a pilha de LPE)

## O que se decide

**Onde mora o perfil de largura VIVO** de um caminho, e **quem o transforma em desenho** — as duas
perguntas que a W1 (a pressão do lápis) e a W2 (as alças do Width Tool) fazem ao mesmo tempo, porque
as duas escrevem no MESMO perfil.

## Por que exige um ADR

É o único item da auditoria de 2026-07-29 onde a pesquisa do próprio repo
(`20_pesquisa_ferramentas_de_artista.md` §96-118) diz que **não há caminho de prateleira**: nem a
kurbo nem a Skia têm traço de largura variável. Existe o caminho do Levien (*stroke expansion*, arXiv
2405.00127) e mais nada. E há uma decisão de **representação** cujo erro custa todo projeto salvo.

## Decisão

### 1. O perfil vivo é um **componente ECS** (`VecStrokeProfile`), não um campo do `StrokeSpec`

| rota | preço | veredito |
|---|---|---|
| campo novo no **`StrokeSpec`** | ele é `Serialize` e a `VecScene` viaja **EMBUTIDA** no `ProjectState` (o `project.rs` o diz na entrada v10) ⇒ bumpa **`VEC_SCENE_SCHEMA_VERSION` 13→14 E `PROJECT_SCHEMA` 38→39**, e um schema divergente **RECUSA o arquivo inteiro** — todo projeto já salvo | ⛔ |
| **componente ECS `VecStrokeProfile`** | cunha `stable_type_id = blake3(NOME)[..8]` próprio, **não move layout posicional de nada** ⇒ **zero bump** | ✅ |

O componente é o padrão que este módulo já usou **sete vezes** (`VecOffset` · `VecTextPath` ·
`VecEnvelope` · `VecBlend` · `VecFilter` · `VecConnector` · `VecMorph`), e a lei que o justifica está
escrita nas sete: **um bump recusa todo projeto já salvo**, e jogar fora trabalho real para evitar um
componente é o trade errado. Semanticamente também casa — um perfil de largura é atributo
**por-caminho**, que é exatamente o que o `VecOffset` é.

### 2. A representação é uma **LISTA de paradas** `(posição, multiplicador)`, e há UMA

O `WidthProfile` de 4 números (`ph2d-vec-scene/src/width_profile.rs`) **fica**, e o próprio header
dele já prevê isto: *"as alças na linha são um GESTO de canvas … outra wave, e que consome este
perfil em vez de o substituir"*. Mas ele passa a ser uma **face de PRESET** — o que a tabela de
parâmetros desenha e o que o artista escolhe por nome (*afina-no-fim*, *afina-nos-dois*) — e o que o
documento guarda é a lista.

⚠️ **Uma representação, não duas.** O preset **constrói** a lista (`WidthProfile::to_stops()`); nada
lê os dois e tenta reconciliá-los. Duas representações do mesmo fato divergem em silêncio, e aqui a
divergência apareceria como *"o traço mudou de forma quando eu arrastei a alça"*.

⚠️ **Multiplicadores, nunca medidas absolutas** — a lei que o `WidthProfile` já carrega: o artista
escolheu a largura no slider de Width, e o perfil diz o que acontece com **ELA** ao longo do caminho.
`1.0` em toda parte é o traço uniforme de sempre, o que faz do perfil ausente um neutro **de verdade**
e não um valor que por acaso não faz nada.

### 3. **UM motor** produz a silhueta: o `power_stroke` que o Apply já usa

O traço de largura variável **não é um traço** para o rasterizador — é uma região PREENCHIDA cuja
borda é a envoltória das duas offsets. Já temos quem a compute exatamente:
`ph2d_vec_boolean::power_stroke`, hoje consumido pelo `Expand::PowerStroke` (destrutivo).

A decisão é a lei do ADR-0128: **`recook` e `expand` chamam a MESMA função**. O preview vivo produz
`LiveGeometry` (o padrão do `VecOffset`) a partir do mesmo `power_stroke`; o **Apply** materializa. Uma
segunda rota — um approximador "só para o preview" — faria a forma **SALTAR** no instante do Apply, que
é o defeito que o ADR-0128 pagou cinco vezes.

⛔ **Não construímos um stroker de largura variável novo** (o caminho do Levien). Se um dia a
performance o exigir, ele entra **atrás desta mesma porta**, com gate de paridade contra o
`power_stroke` — não como um segundo desenho.

### 4. O custo é **memoizado por CHAVE**, e a chave carrega o que é desenhado

O cozimento por frame é o modelo de todo produtor vivo deste módulo, e o **preço de um memo mal
chaveado acabou de ser medido**: o memo do FX raster era `(pilha, w, h)` e **mudar a cor do fill de uma
forma filtrada não mudava a tela** (W0.1 do plano 25, 2026-07-29). Então a chave deste é escrita com
essa lição: **a geometria autorada + o perfil + a largura**, comparados por valor.

⚠️ E fica registado o que aquela wave também mediu: **a translação NÃO entra na chave** (o desenho é o
mesmo, o afim é aplicado na saída), senão panhar re-cozinha a cena inteira.

## Consequências

- **Zero bump de schema** nesta wave e na W2 (`PROJECT_SCHEMA` fica **38**, `VEC_SCENE` fica **13**) —
  e é o que as mantém fora da disputa de número com as outras linhas.
- ⚠️ **O `Expand::PowerStroke` JÁ tem um perfil autorado — nos SLIDERS do painel** (`W Start` · `W
  Mid` · `W End` · `W Pos`, lidos pela `render_loop` porque é ela que tem o store; o
  `WidthProfile::UNIFORM` do `vec_expand.rs` é só o lugar reservado que ela preenche). Isso torna a
  adjacência a decisão que esta wave TEM de tomar em vez de herdar: **os sliders passam a escrever no
  perfil do caminho** e o Apply assa o que está lá. Deixar os dois — um perfil no componente e quatro
  sliders com valores próprios — é a falha de duas-portas na forma mais caras: o preview mostraria
  uma espessura e o Apply assaria outra.
- A **pressão** (W1) e as **alças** (W2) são duas maneiras de escrever a MESMA lista. Uma terceira
  (importar um preset) também. Nenhuma delas é o dono.
- ⚠️ **O `WidthProfile` do `ph2d-vector-doc` (`style.rs:74`) é OUTRO tipo, num crate CONGELADO** (§6,
  ADR-0056..0068) e **não é tocado** — ele serve o `vector_network.rs`, que é o motor legado. Os dois
  nomes iguais em crates diferentes são uma armadilha de leitura, e é por isso que ela está escrita
  aqui.

## Alternativas rejeitadas

- **Assar a largura no momento do traço** (o que o Flip faz, e o que a `line/FLIP` shipou em 27/07): o
  raster não tem o que preservar, mas aqui a fonte é editável — assar mata o *"mudei de ideia sobre a
  espessura"*, que é a razão de o módulo ser não-destrutivo (ADR-0121).
- **Guardar a largura POR VÉRTICE** (o `CubicVertex` do Rive tem espaço): amarra o perfil à
  parametrização, e este módulo já pagou essa lição duas vezes (a correspondência do Blend: *"duas
  formas que se veem iguais têm de blendar igual"*). Uma parada tem **posição de ARCO**, não índice de
  vértice.
- **Um `PathEffect` na pilha de LPE:** a pilha é `VecPath -> VecPath` (ADR-0132) e a largura variável
  produz uma REGIÃO a partir de um traço — ela não é uma transformação de caminho, é uma
  interpretação do ESTILO. Pôr lá faria o efeito seguinte operar sobre a silhueta em vez do caminho.
