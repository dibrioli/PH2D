# Plano — Pattern Along Path (arte repetida ao longo do caminho)

> Item #11 da pesquisa [`20_pesquisa_ferramentas_de_artista.md`](20_pesquisa_ferramentas_de_artista.md)
> ("arte repetida ao longo do caminho"). Irmão direto do **texto em caminho** (doc 22): onde o
> texto cavalga uma curva com **glifos rígidos**, o pattern cavalga a MESMA curva com **cópias
> rígidas de um motivo**. Um glifo é uma forma; um motivo é uma forma. O motor é o mesmo.

## §0 — A decisão, antes das waves

**O que é:** um **motivo** (um `VecPath` qualquer) é copiado `N` vezes ao longo de um
**caminho-guia**, cada cópia transladada ao ponto de arco e **rodada para a tangente** (rígida —
transladada + rodada, **não** deformada). `N` preenche o comprimento da guia pelo avanço do motivo
(largura × espaçamento), como as letras avançam num `<textPath>`.

**Por que rígido, e não deformado (a bifurcação do produto):** deformar um motivo para *dobrar* ao
longo da curva (o "Pattern along Path" do Inkscape / o *Art Brush* do Illustrator) é **outra
operação**, e nós já a temos: é o **Envelope/Warp** ([ADR-0129](../architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md)),
cujo mapa **não é afim** e por isso precisa de `fit_to_bezpath`. O gap que falta é a versão
**rígida** — o *Scatter Brush* / *Pattern Brush* rígido —, e é exatamente o que o motor de arco
que o texto em caminho construiu entrega **sem refit** (um afim comuta com a avaliação de Bézier,
`text_path.rs` §Rigidez). Fazer o rígido primeiro é reusar o que existe; o *bending* é uma wave
futura que compõe com o Envelope, **não** um flag deste. Nomeado em §7.

**Onde mora o vínculo — componente, não efeito de pilha.** Pattern Along Path é uma relação de
**dois objetos** (motivo + guia), como texto + guia. A pilha de efeitos (ADR-0132) é de **uma
entrada** — um efeito come o caminho em que está. Então o vínculo é um **componente ECS opcional**
no MOTIVO, `VecPatternPath { path, … }` — o **irmão exato** do `VecTextPath`. Consequências, todas
já pagas pela linha de texto:

- **Zero bump de schema** — componente novo cunha `stable_type_id` própria (blob-key), não apenda
  campo posicional (o oposto do que recusaria todo projeto salvo).
- O motivo continua um path normal: pode ter a **própria pilha de efeitos** rodando ANTES do
  pattern (autoradas → pilha → cozido → **o componente copia o cozido `N` vezes** → cozido final).
  O pattern é um transform POS-pilha dirigido por componente, exatamente como o ride do texto.
- Reuso: a **resolução da guia** (`ArcPath` do cozido-em-mundo), a **alça** de canvas e o
  **padrão de seção de painel** saem da infra de texto. ⚠️ A resolução da guia é a MESMA pergunta
  geométrica dos dois vínculos → **uma porta** chaveada pelo *id do caminho-guia*, não pelo tipo de
  componente (§W2). Duas cópias divergiriam no dia em que uma ganhasse um cuidado.

**Kill-criterion (antes do build, DIRETIVA §5):** o texto mede **0,72 ms** o layout de arco contra
o kill de 8. O pattern é `N × verts(motivo)` cópias de afim puro — mais barato por cópia que um
glifo (sem hinting, sem fonte). **Se o re-cook de uma tecla de espaçamento passar de 8 ms @ um
motivo de ~40 verts numa guia que caiba ~200 cópias, a feature não existe nesta forma** e o próximo
passo é cache por-params, não subir o teto.

✅ **MEDIDO na W1 (2026-07-22): 0,597 ms** para 200 cópias × 40 verts — ~13× de folga sob o kill,
sem cache nenhum. O recuo previsto **não é preciso** (gate `a_keystroke_recook_stays_under_the_kill`,
`#[ignore]`, `--release`).

## §1 — Waves

| Wave | Entrega | Crate | Estado |
|---|---|---|---|
| **W1** | O **motor** `pattern_along(motivo, guia, spec) -> contornos` (arco → afim rígido por cópia; sem refit) + gate red-first + gate de perf | `ph2d-vec-scene` (`pattern_path.rs`, arquivo próprio) | ✅ **0,597 ms/200 cópias** |
| **W2** | O **vínculo**: componente `VecPatternPath` + porta única de guia (`ArcPath` do cozido-em-mundo, compartilhada com texto) + o render que faz o cozido do motivo = saída do motor | `ph2d-ecs` + shell | ✅ **vivo + smoke 24** |
| **W3** | Seção de painel **Pattern on Path** (Spacing · Offset · Flip + Link/Detach), ids, i18n — espelha a seção de texto | `ph2d-panel-vector` + `ph2d-editor-core` + `ph2d-i18n` | — |
| **W4** | **Alça** de canvas (o ponto de início no arco), modo Select — avaliar reuso da alça de texto vs irmã | `ph2d-vec-render` + shell | — |
| **W5** | Cena(s) de smoke auto-verificáveis | shell (`build_smoke.rs`) | — |

## §2 — O motor (W1), em uma frase

`ArcPath::from_contour(guia)` → a cópia `k` ocupa a fatia `[start + avanço·k, start + avanço·(k+1)]`
(centro em `start + avanço·(k + ½)`), e **só entra se a fatia inteira couber** em `[0, total]` — nada
transborda as pontas, a cauda sobra (§3). No centro, `GlyphFrame::on_path(guia, centro, offset, flip)`
(⚠️ `None` numa cúspide → **pula a cópia**, como o texto pula o glifo), e cada vértice do motivo —
âncora **E alças** — mapeado por `frame.apply([mx − cx, my − cy])` (o motivo centrado no ponto de
arco, o `cx/cy` do bbox do motivo). Avanço = `largura(bbox) × spacing`, saturado longe de zero. O
afim é rotação + translação ⇒ `corner_radius` (comprimento LOCAL) sobrevive intacto; a curva que sai
é a **imagem exata** da que entra.

## §3 — O que NÃO faz, de propósito (§7 expandido)

- **Não deforma** (bending) — é o Envelope (ADR-0129), wave futura que compõe, não flag.
- **Não escala/roda cópia por progressão** — isso é o **Repeater** (`fx_repeat`, grade/radial por
  afim cumulativo); o pattern é dirigido pela GUIA, não por parâmetro. São ferramentas distintas.
- **Não faz *fit within path* / esticar o avanço para fechar exato** — v1 tila pelo avanço nominal
  e para onde a próxima cópia não cabe (a cauda pode sobrar); *fit* é refinamento com dono próprio.
- **Uma orientação (Rainbow/rígida)** — as demais herdam a mesma limitação normativa do texto
  (`text_path.rs` §"O que este módulo NÃO faz": só a spec aberta entra).

## §4 — Decisões de arquitetura da W2 (o que a integração vê)

- **Não é `replace_cooked` como o texto — é `LiveGeometry` como o `VecOffset`.** O texto sobrescreve
  `verts` porque não tem fonte autorada a perder (a fonte são os params). O motivo TEM geometria
  autorada (é ela que o Node edita), então as cópias são **desenho derivado** (`pattern_live` coze
  `Vec<VecPath>` por frame → `dispatch` no z do motivo), a fonte intocada. É o precedente do Offset.
- **A pose do motivo é IGNORADA (v1).** As cópias vêm da `cooked()` LOCAL do motivo (a forma), não da
  de mundo. Mexer no gizmo do motivo não muda as cópias — é o *"mover o objeto vinculado não quer
  dizer nada"* do texto/conector. Para redimensionar as cópias: editar os nós do motivo, ou o
  Spacing. Escalar por gizmo (assar o linear do `Transform`) é **decisão de produto adiada** — a
  troca é de uma linha (`pattern_live::recook`) se o smoke pedir.
- **Guia degenerado / cauda que não cabe ⇒ mostra a FONTE** (o motivo), não nada — ao contrário do
  Offset (onde vazio É a aniquilação). O `pattern_live` deixa a entrada AUSENTE nesses casos.
- **Sem memo (v1), medido** (0,597 ms/200 cópias, dentro do orçamento). Memoizar exige detectar a
  mudança do guia; adicioná-lo sem medir que uma cena parada custa é otimização prematura.
- Contador de componentes: **ecs 34→35, render 35→36, script 35→36** (o número se CONTA na
  integração). **Sem bump de schema** (componente novo cunha blob-key própria). Smoke: **24**.
