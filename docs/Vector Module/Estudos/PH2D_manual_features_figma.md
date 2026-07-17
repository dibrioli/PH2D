# Manual de Implementação — Features de Sistema (Figma) para o PH2D

> **Documento técnico / manual para agente de implementação. Volume 2 da série** (o Vol. 1 cobre features de desenho vetorial expressivo).
> Escopo: as features que tornam o Figma *especial* não são de geometria, são de **sistema** — topologia de edição (vector networks), layout responsivo (auto layout + constraints), reuso com herança (components/variants), indireção semântica (variables & modes) e transição automática (smart animate). Para o PH2D — um **game engine** para o artista solo — cada uma tem um análogo direto no mundo de engine (prefabs, HUD responsivo, theming/dificuldade, state transitions). Este manual mapeia cada feature à sua implementação de referência open-source, fixa **um algoritmo principal + um alternativo**, e dá notas de integração no stack Rust/wgpu/lyon.
>
> **Como usar (para o agente):** diferente do Vol. 1 (geometria pura), aqui a maior parte do valor está no **modelo de dados**, não no algoritmo. Priorize acertar as estruturas (grafo, árvore de layout, mapa de overrides, grafo de resolução de tokens) antes de otimizar. Reuse componentes do Vol. 1 onde indicado (o arrangement/DCEL da Live Paint reaparece nas vector networks; a indireção de swatch reaparece em variables).

---

## 0. Recomendação de stack e a tese central

| Necessidade | Crate/base recomendada | Papel |
|---|---|---|
| Layout flexbox/grid responsivo | **`taffy`** (DioxusLabs) | Flexbox + CSS Grid + Block em Rust. Já usado por Dioxus, Zed, Bevy, Blitz. |
| Constraints de ancoragem | **`cassowary-rs`** / kiwi | Solver Cassowary incremental para regras "alinhe A com B". |
| Topologia de vector network | DCEL/half-edge sobre arena (`slotmap`) | Mesmo substrato do arrangement do Vol. 1 (Live Paint). |
| Interseção de arestas (expandir o grafo) | `kurbo` (curva×curva) + sweep-line | Reusa o motor de interseção do Vol. 1. |
| Resolução de cor perceptual | **OKLab** | Para variables de cor e interpolação em Smart Animate. |
| Interpolação/tweening | `keyframe` / spring físico próprio | Smart Animate e transições de estado. |

**Tese central deste volume:** todas as cinco features reduzem a **três primitivas de engine** que o PH2D precisa de qualquer jeito:

1. **Identidade estável.** Todo objeto tem um ID persistente (não uma posição na árvore, não um nome). Components, Smart Animate e undo/redo dependem disso.
2. **Indireção/resolução.** Um valor semântico (`primary`, `spacing.md`) resolve para um valor concreto por *contexto* (mode). É o mesmo mecanismo para swatches, temas, dificuldade e localização.
3. **Passe de layout/constraint.** Dado um conjunto de regras, computar posições/tamanhos finais. Serve para UI responsiva, HUD, e recomposição de arte modular.

> ⚠️ **Recomendação opinativa forte (identidade):** o Figma casa camadas por **nome + hierarquia + ordem** — tanto em components quanto em Smart Animate. Isso é frágil e falha *em silêncio*: renomear ou reagrupar uma camada quebra a animação/instância sem aviso (as próprias fontes chamam isso de "silent failure" e "it worked yesterday"). **O PH2D deve casar por ID estável**, não por nome. É estritamente melhor: robusto a rename/reorder, trivial de depurar, e já necessário para undo/serialização. Ofereça o matching-por-nome só como *fallback* de importação (ex.: colar SVG externo). Esta única decisão evita a classe inteira de bugs que atormenta usuários de Figma.

---

## 1. Vector Networks (topologia de grafo em vez de path)

**O quê:** em vez do path hierárquico tradicional (uma cadeia fechada de um endpoint a outro), vértices e arestas formam um **grafo livre** — mais de duas arestas podem se encontrar num vértice, arestas podem ser compartilhadas, e regiões preenchíveis emergem dos ciclos. Edição topológica muito mais fluida ("conecte qualquer coisa a qualquer coisa, delete qualquer coisa"). Tecnicamente é a feature mais difícil de replicar do Figma.

**Referência open-source e conceitual:**
- **Figma Dev Docs — `VectorNetwork`**: o modelo de dados canônico, em três arrays. **Vertices** (pontos), **Segments** (arestas não-direcionadas que indexam vértices, com tangentes Bézier opcionais), **Regions** (loops de índices de segmentos + winding rule + fills). É um **multigrafo com identidade de aresta** (dois vértices podem ter várias arestas entre si).
- **Alex Harri — "The Engineering behind Figma's Vector Networks"**: descreve o "expandir o grafo" (graph expansion): em cada interseção, cria-se um nó e as arestas cruzadas são quebradas ali. Cúbicas podem gerar até 9 interseções e auto-interseção, exigindo checagem por-aresta.
- **Figma Blog — "Delete and Heal"**: ao deletar um vértice com nº *par* de arestas, "cura" emparelhando arestas opostas (ordenadas pelo **ângulo da tangente**, não pela reta ao outro vértice) e aproximando duas cúbicas por uma via **Schneider** (o mesmo fit do Vol. 1 §1.3).
- **Vector Graphics Complex (VGC)** de Boris Dalstein: a formalização acadêmica quase idêntica (chegou por caminho independente ~mesma época), com vértices/arestas/faces por half-edges — a base teórica se quiseres compartilhamento de arestas e sobreposição.
- **Penpot** e o tutorial *infinitecanvas* (Lesson 22): implementações abertas do modelo.

Modelo de dados (espelhando a API do Figma, adaptado a Rust):

```rust
struct VectorNetwork {
    vertices: SlotMap<VertexId, Vertex>,   // IDs estáveis (não índices posicionais)
    segments: SlotMap<SegmentId, Segment>,
    regions:  Vec<Region>,
}
struct Vertex { pos: Point /*, radius, corner_style… */ }
struct Segment {                 // aresta NÃO-direcional
    start: VertexId, end: VertexId,
    tangent_start: Vec2,         // handle Bézier (default 0,0 => reta)
    tangent_end:   Vec2,
}
struct Region {                  // ciclo(s) de segmentos = área preenchível
    loops: Vec<Vec<SegmentId>>,  // externo + buracos (ex.: a letra "o")
    winding: FillRule,
    fills: Vec<Paint>,
}
```

**Algoritmo principal — half-edge (DCEL) + graph expansion por sweep-line, e detecção de regiões por faces.**
1. **Substrato:** represente o grafo como **DCEL** (doubly-connected edge list): cada segmento vira duas half-edges gêmeas; cada half-edge tem `next`/`prev`/`twin`/`origin`/`face`. Isso dá travessia de faces em O(1) local — essencial para achar regiões.
2. **Expansão do grafo:** ao inserir/mover arestas, compute interseções (sweep-line Bentley-Ottmann; para Béziers use `kurbo` curva×curva) e **quebre** as arestas nos cruzamentos, inserindo vértices. Auto-interseção de cúbica: ache os dois `t` e quebre.
3. **Regiões automáticas (o truque de UX do Figma):** em vez de exigir winding number manual, **preencha automaticamente todo espaço fechado** — cada face limitada do DCEL (exceto a face externa infinita) é preenchida por default; o usuário depois "fura" regiões (toggle da face) com o balde. Isso resolve o problema histórico de o usuário ter que manipular espaço negativo.
4. **Delete-and-heal:** ao remover um vértice de grau par, emparelhe half-edges opostas por ângulo de tangente e refite uma cúbica (Schneider) preservando a curvatura aproximada.

**Algoritmo alternativo — planar map com faces implícitas (livarot-style), sem DCEL explícito.**
Mantenha só vertices+segments e **recompute o arrangement** (o mesmo do Vol. 1 §2.2, Live Paint) sob demanda para achar as regiões, em vez de manter um DCEL incremental sempre consistente. Mais simples de implementar (o estado editável é "burro": só listas), ao custo de recomputar faces a cada mudança topológica. Bom para começar; migra para DCEL incremental quando a performance de edição ao vivo exigir.

**Trade-off:**

| | DCEL incremental | Recompute do arrangement |
|---|---|---|
| Edição ao vivo (drag) | Rápida (update local) | Recomputa tudo |
| Complexidade de implementação | Alta (manter invariantes) | Baixa |
| Robustez a casos degenerados | Exige cuidado | Reusa Clipper2/sweep testado |

**Notas de integração PH2D:** as vector networks são o **substrato de edição do canvas criativo**, e as *regions* conectam direto à Live Paint (Vol. 1 §2.2) e aos booleanos (Vol. 1 §3.1) — não implemente três estruturas separadas; **uma DCEL serve às três**. Mantenha compatibilidade retroativa com paths (o Figma faz questão disso: toda path é uma vector network degenerada — uma cadeia sem ramificação). Guarde tangentes Bézier no segmento, não no vértice, para permitir descontinuidade (cantos) e mirroring configurável (angle / angle+length). Use **IDs estáveis** (slotmap), não índices — undo/redo e colaboração dependem disso.

---

## 2. Auto Layout + Constraints (recomposição responsiva)

**O quê:** duas coisas relacionadas. **Auto Layout** = layout tipo flexbox (empilhamento com direção, gap, padding, alinhamento, crescimento) que recompõe filhos automaticamente. **Constraints** = regras de ancoragem no resize (left/right/center/scale, pin a bordas). Juntos: arte/UI que se recompõe sozinha em vez de posições fixas.

**Referência open-source:**
- **Taffy** (DioxusLabs) — motor de layout Rust de alta performance implementando **Flexbox, CSS Grid e Block** da spec CSS. É a base de Dioxus, Zed, Bevy e Blitz. Bench: comparável/superior ao Yoga (bindings Rust). Este é o crate certo para o Auto Layout do PH2D.
- **Yoga** (Meta) — o motor flexbox C++ por trás do React Native / do Auto Layout de várias ferramentas; alternativa madura via FFI.
- **Cassowary** (`cassowary-rs`, port do `kiwi`/nucleic) — solver de constraints lineares incremental, o mesmo do Auto Layout da Apple (iOS/macOS). Usado por Ratatui, Iced-likes, etc.

**Algoritmo principal (Auto Layout) — Flexbox via Taffy.**
Modele cada frame/objeto como um nó com um `Style` (direção, gap, padding, `flex_grow`, alinhamento, tamanho `auto`/fixo). Rode o passe de layout uma vez; leia posição/tamanho computados.

```rust
use taffy::prelude::*;
let mut tree: TaffyTree<()> = TaffyTree::new();
let header = tree.new_leaf(Style {
    size: Size { width: length(800.0), height: length(100.0) }, ..Default::default()
})?;
let body = tree.new_leaf(Style {
    size: Size { width: length(800.0), height: auto() }, flex_grow: 1.0, ..Default::default()
})?;
let root = tree.new_with_children(Style {
    flex_direction: FlexDirection::Column,
    size: Size { width: length(800.0), height: length(600.0) }, ..Default::default()
}, &[header, body])?;
tree.compute_layout(root, Size::MAX_CONTENT)?;
let body_box = tree.layout(body)?; // {location, size} — alimenta o transform do objeto
```

Para leaf nodes com conteúdo intrínseco (texto via **parley**, imagem, arte vetorial), use a API low-level de Taffy com uma **measure function** que reporta o tamanho do conteúdo — é como você pluga o layout de texto/arte no fluxo.

**Algoritmo principal (Constraints) — Cassowary (simplex incremental).**
Para ancoragem estilo "esta borda gruda na direita", "este objeto centraliza", "B tem 3× a largura de A": exprima como **inequações/equações lineares** com prioridades (required vs preferred). O solver Cassowary mantém a solução incrementalmente ao editar (rápido no resize ao vivo). Cassowary não conhece "retângulos" ou "2D" — você monta variáveis `x_left, x_right, width…` e as relações; uma camada fina traduz constraints de UI para o solver.

```
// pseudo (cassowary-rs): centralizar 'child' em 'parent', preferir largura 200
child.left  - parent.left  == parent.right - child.right   // simétrico => centrado
child.width == 200.0 | WEAK
child.left  >= parent.left | REQUIRED
```

**Algoritmo alternativo — layout imediato/procedural (sem solver).**
Para 90% do HUD de jogo, um layout **imediato** codificado (anchors + offsets calculados à mão a cada frame, estilo Dear ImGui / Unity anchors) é suficiente e mais barato que um solver Cassowary geral. Anchors = par (min, max) normalizado por eixo; a posição final é interpolação linear dentro do retângulo pai + offset em pixels. Trivial, determinístico, sem dependência. Use o solver só quando as regras forem *relacionais* entre irmãos (alinhamentos mútuos), onde o procedural vira espaguete.

**Notas de integração PH2D:** para um game engine mirando **iPad + desktop** (viewports muito diferentes), layout responsivo do **HUD/UI** é requisito, não luxo — adote **Taffy** para isso desde já (é o mesmo motor que engines Rust usam). Auto Layout também serve à **arte modular**: nine-slice, balões de diálogo que crescem com o texto, inventários em grid. O resultado do layout é apenas um `transform` por objeto → alimenta o teu pipeline wgpu normalmente. Constraints (Cassowary) reserve para o editor (alinhar objetos na canvas) e casos relacionais; não sobre-engenheire o HUD comum com ele.

---

## 3. Components + Variants + Properties (prefabs com herança)

**O quê:** instâncias reutilizáveis de um "componente mestre". Editar o mestre propaga para todas as cópias; cada cópia pode ter **overrides** locais (texto, cor, ícone) que sobrevivem à propagação. **Variants** agrupam versões (size/state/color) sob um componente com **properties** (boolean, swap de instância, variant enum). Poderoso para tilesets, personagens modulares e UI de jogo — é, essencialmente, o **sistema de prefabs** de um game engine.

**Referência open-source:** **Penpot** (alternativa open-source ao Figma). Modelo: **main component** (fonte da verdade) ↔ **copy/instance** (herda tudo). Overrides = modificações na cópia ausentes no main; "Reset overrides" volta ao main; "Update main" empurra mudanças da cópia para o mestre. Variants = versões com propriedades escolhíveis; swap troca a instância mantendo overrides compatíveis.

**Algoritmo principal — herança prototipal + mapa de overrides esparso.**
Uma instância **não copia** a árvore do mestre; ela **referencia** o mestre e guarda apenas um **mapa esparso de overrides** por-propriedade, endereçado por **caminho de sub-objeto estável** (ID, não índice — ver §0).

```rust
struct MainComponent { id: CompId, tree: ObjectTree /* com IDs internos estáveis */ }

struct Instance {
    main: CompId,
    // override: (sub-objeto interno) -> (propriedade -> valor)
    overrides: HashMap<SubObjId, HashMap<PropKey, Value>>,
    props: HashMap<PropName, PropValue>, // variant/boolean/swap escolhidos
}

// Resolução de uma propriedade de um sub-objeto da instância:
fn resolve(inst: &Instance, sub: SubObjId, key: PropKey, mains: &Mains) -> Value {
    inst.overrides.get(&sub).and_then(|m| m.get(&key)).cloned()
        .unwrap_or_else(|| mains[&inst.main].value_of(sub, key)) // fallback ao mestre
}
```

Editar o mestre propaga automaticamente (a instância lê o mestre no que não foi sobrescrito). "Reset" = remover a entrada do mapa. "Update main" = mover o override para o mestre e limpar das instâncias. **Variants** = um main "guarda-chuva" que seleciona entre sub-mestres por combinação de properties; a instância só guarda a combinação escolhida + overrides.

**Algoritmo alternativo — cópia profunda com diff/merge (snapshot + patch).**
Materialize a instância como cópia completa e, na atualização do mestre, faça um **3-way merge** (base = mestre antigo, ours = overrides da instância, theirs = mestre novo) para reaplicar mudanças. Mais simples de renderizar (a instância é auto-contida, sem resolução em tempo de leitura) e robusto a mudanças estruturais do mestre, mas o merge é mais caro e propenso a conflitos, e o uso de memória cresce com o nº de instâncias. Bom quando instâncias divergem muito do mestre; ruim para milhares de tiles idênticos.

**Trade-off:**

| | Prototipal + overrides esparsos | Cópia + diff/merge |
|---|---|---|
| Memória (N instâncias) | O(overrides) — ótimo p/ tilesets | O(N × tamanho) |
| Custo de render | Resolução por leitura | Zero (auto-contida) |
| Propagação do mestre | Automática | Merge explícito |
| Robustez a reestruturação do mestre | Precisa de IDs estáveis | Melhor |

**Notas de integração PH2D:** **isto É o sistema de prefabs do engine** — projete-o como tal, não como feature de UI. Um tileset = um main + milhares de instâncias (a herança prototipal com overrides esparsos é a única que escala em memória aqui). Personagem modular = componente com **swap properties** (trocar cabeça/arma). Use **IDs de sub-objeto estáveis** para endereçar overrides (senão renomear/reordenar quebra tudo — ver §0). Conecte com o Vol. 1: a arte dentro de um componente pode ter seu próprio node graph não-destrutivo; overrides podem sobrescrever parâmetros de node (ex.: um variant "danificado" muda o `roughen.amplitude`). Isso liga components a variables (§4): properties de componente podem *bind* a tokens.

---

## 4. Variables & Modes (indireção semântica / design tokens)

**O quê:** tokens semânticos (`color.primary`, `spacing.md`, `radius.sm`) com **múltiplos modos** (tema claro/escuro, escala, estados, densidade). Um valor semântico **resolve para um valor concreto por contexto**. Suporta **aliases** (um token referencia outro) e **operações** (math). É a generalização da indireção de swatch do Vol. 1 §2.3 para *qualquer* propriedade.

**Referência open-source:** **Penpot Design Tokens** — segue o formato padrão **W3C DTCG** (Design Tokens Community Group), com **sets** (grupos), **themes/modes**, **aliases** (`{color.brand.500}`), math, import/export JSON, e remapeamento automático de referências ao renomear. Escolha o DTCG para interoperar com o resto do mundo (Style Dictionary, Tokens Studio, export web/iOS/Android).

**Algoritmo principal — grafo de resolução (DAG de aliases) + seleção por modo, com detecção de ciclo.**
Tokens formam um **DAG**: folhas são valores concretos; nós internos são aliases/expressões apontando para outros tokens. Cada token pode ter um valor **por modo**. Resolver = escolher o modo ativo e seguir os aliases até uma folha, avaliando math no caminho. Memoize por `(token, mode)`; invalide o subgrafo a jusante ao editar (mesma disciplina de cache do node graph do Vol. 1 §7).

```rust
enum TokenValue { Concrete(Value), Alias(TokenId), Expr(Vec<Term>) } // math: {spacing.md} * 2

struct Tokens {
    // valor por modo: modes[mode][token]
    values: HashMap<Mode, HashMap<TokenId, TokenValue>>,
}

fn resolve(t: &Tokens, id: TokenId, mode: Mode, seen: &mut HashSet<TokenId>) -> Value {
    assert!(seen.insert(id), "ciclo de alias detectado!"); // proteção obrigatória
    match &t.values[&mode][&id] {
        TokenValue::Concrete(v) => v.clone(),
        TokenValue::Alias(a)    => resolve(t, *a, mode, seen),
        TokenValue::Expr(terms) => eval(terms, |a| resolve(t, a, mode, seen)),
    }
}
```

Modos podem ser **escopados por subárvore** (como CSS variables no `:root` sobrescritas por `.dark-theme` num nó): o objeto resolve o token subindo até o primeiro ancestral que define aquele modo. Cor **sempre em OKLab** para interpolação/derivação perceptual.

**Algoritmo alternativo — resolução por tabela achatada (flattened lookup) por modo.**
Pré-compute, para cada modo, uma **tabela plana** `token → valor concreto` (resolvendo todos os aliases de uma vez). Render vira lookup O(1) direto, sem seguir o grafo por leitura. Trocar de modo = trocar a tabela ativa. Mais rápido em runtime e trivial de bindar a shaders (é um uniform buffer de valores), ao custo de recomputar a tabela ao editar qualquer token. **Esta é a rota recomendada para o runtime do jogo** (o grafo fica no editor; o jogo carrega tabelas achatadas).

**Notas de integração PH2D:** variables & modes é a feature de **maior alavancagem** para um game engine, porque generaliza para coisas que o Figma nem usa:
- **Theming** (claro/escuro, paletas sazonais) — o caso óbvio.
- **Modos de dificuldade / balanceamento** — `enemy.hp`, `spawn.rate` como tokens por modo (easy/normal/hard). Designer troca sem tocar em código.
- **Localização** — strings como tokens por locale.
- **Data binding em runtime (a ponte para Rive):** um token pode ser dirigido por **estado do jogo** (não só por modo estático). Ligue tokens a inputs do runtime (vida, tempo, score) e a UI/arte reage sozinha — é exatamente o "data binding" do Rive que apareceu na conversa anterior, e cai naturalmente aqui.

Adote **DTCG** para import/export (interop) mas guarde a tabela achatada por modo para o runtime. Faça toda cor em OKLab. Conecte a §3: properties de componente que bindam a tokens dão tilesets/UI que reharmonizam com o tema.

---

## 5. Smart Animate (tween automático por correspondência)

**O quê:** anima a transição entre dois estados (frames) **sem keyframes manuais**: casa camadas entre os dois estados, detecta o que mudou (posição, tamanho, opacidade, rotação, escala, cor de fill, raio de canto) e interpola. É a mesma ideia do **FLIP** (First-Last-Invert-Play) da animação web, e o análogo direto das transições de **state machine** do Rive.

**Referência conceitual:** **Figma "Announcing smart animate"** e docs. Mecânica: (1) **matching** de camadas entre origem e destino por **nome + hierarquia + ordem**; (2) para cada par casado, **interpola as propriedades que diferem**; (3) camadas sem par fazem **dissolve** (fade); (4) camadas idênticas não animam. Suporta easing e (na web) **spring** (stiffness/bounciness/speed).

**Algoritmo principal — matching por identidade estável + interpolação de propriedades (FLIP generalizado).**
1. **Matching.** Para cada objeto no destino, ache o correspondente na origem. **No PH2D: case por ID estável** (não por nome — ver §0). Objetos com match → tween; sem match no destino → fade-in; sem match na origem → fade-out.
2. **Diff.** Para cada par, compute o conjunto de propriedades que diferem (transform, opacity, fill, corner radius, etc.).
3. **Interpolate.** Ao longo de `t ∈ [0,1]` com uma easing/spring, interpole cada propriedade:
   - Posição/tamanho/rotação/escala: LERP/SLERP no **transform** (decomponha em translate/rotate/scale para não distorcer — evite interpolar matrizes cruas).
   - Cor: LERP em **OKLab** (nunca sRGB).
   - Forma (path): correspondência de nós + LERP de controles (Vol. 1 §3.2, o mesmo do morph).

```rust
fn smart_animate(from: &Scene, to: &Scene, t: f64, ease: impl Fn(f64)->f64) -> Scene {
    let mut out = to.clone_structure();
    for obj_to in to.objects() {
        match from.by_id(obj_to.id) {                 // matching por ID estável
            Some(obj_from) => {
                let e = ease(t);
                out.set_transform(obj_to.id, lerp_trs(obj_from.trs, obj_to.trs, e));
                out.set_opacity(obj_to.id,   lerp(obj_from.opacity, obj_to.opacity, e));
                out.set_fill(obj_to.id,      lerp_oklab(obj_from.fill, obj_to.fill, e));
                // …demais propriedades que diferem
            }
            None => out.set_opacity(obj_to.id, ease(t)), // fade-in dos novos
        }
    }
    // objetos só na origem => fade-out (1 - ease(t))
    out
}
```

**Algoritmo alternativo — timeline com keyframes explícitos (tweening tradicional).**
Em vez de inferir a transição de dois estados, use uma **timeline** com keyframes por propriedade e curvas de easing (como o Figmotion, o Rive, ou `bevy_tweening`). Mais controle (overlap, staggering, motion path), necessário para animação "de verdade" (logo, personagem, micro-motion complexo). Trade-off: exige autoria manual de keyframes — perde a mágica "sem keyframes". A escolha certa é **oferecer os dois**: Smart Animate para transições de estado/UI (barato, automático) e timeline para animação de conteúdo.

**Notas de integração PH2D:** Smart Animate é a ponte natural para **state machines estilo Rive** (que apareceu na conversa anterior como a feature mais relevante para tua tese). Modele estados como cenas e as transições como Smart Animate por ID: o runtime alimenta inputs (bool/number/trigger), o engine interpola entre estados casados sem o artista desenhar quadros intermediários. Isso **elimina o round-trip arte→engine** — o artista autora estados, o jogo dirige a transição. Combine:
- com **§3 (variants):** transição entre variants de um componente = Smart Animate entre suas cenas (toggle, accordion, hover — os casos que o Figma cita).
- com **§4 (variables):** um token dirigido por runtime dispara/parametriza a transição (data binding).
- com **Vol. 1 §5.1 (warp/rigging):** para deformação, interpole os **pontos de controle do rig**, não a arte rasterizada.

Use **spring físico** (não só easing cúbica) para o feel de game UI moderno — stiffness/damping dá o "bounce" que easing curves não dão bem. E **case por ID estável**: a fragilidade do matching-por-nome do Figma é a maior reclamação de usuários; não a herde.

---

## 6. Tabela-resumo: algoritmo principal × alternativo

| Feature | Análogo em game engine | Algoritmo principal | Alternativo | Base Rust |
|---|---|---|---|---|
| Vector networks | Substrato de edição do canvas | DCEL/half-edge + graph expansion (sweep-line) | Planar map recomputado sob demanda | `slotmap`, `kurbo`, Clipper2 |
| Auto Layout | HUD/UI responsivo, arte modular | Flexbox (Taffy) | Layout imediato por anchors | `taffy` |
| Constraints | Alinhamento no editor | Cassowary (simplex incremental) | Anchors procedurais | `cassowary-rs` |
| Components/variants | **Prefabs / tilesets** | Herança prototipal + overrides esparsos | Cópia + 3-way merge | modelo de dados |
| Variables & modes | Theming, dificuldade, i18n, data binding | Grafo de resolução (DAG) + modo | Tabela achatada por modo (runtime) | DTCG, OKLab |
| Smart Animate | **Transições de state machine** | Matching por ID + interpolação (FLIP) | Timeline com keyframes | `keyframe`, spring próprio |

---

## 7. Ordem de implementação sugerida (dependências)

1. **Identidade estável (§0)** — pré-requisito de tudo. `SlotMap`/IDs persistentes em todo objeto e sub-objeto, antes de escrever components ou animação.
2. **Vector networks (§1)** sobre a DCEL — reusa o arrangement da Live Paint (Vol. 1). É o substrato do canvas; quanto antes, melhor.
3. **Variables & modes (§4)** — generaliza a indireção de swatch (Vol. 1 §2.3); habilita theming e é barato. Faça a tabela achatada por modo para runtime.
4. **Auto Layout (§2)** via Taffy — desbloqueia UI responsiva iPad/desktop (requisito de plataforma).
5. **Components/variants (§3)** — o sistema de prefabs; depende de IDs estáveis (§0) e conecta a variables (§4). Alto valor para tilesets/UI.
6. **Smart Animate (§5)** — depende de matching por ID (§0) e de variants (§3); ponte para state machines runtime.
7. **Constraints (§2, Cassowary)** — só quando surgir necessidade relacional no editor; não bloqueie o resto por isso.

---

## 8. Como este volume conecta com o Vol. 1 (síntese)

O Vol. 1 dá o **quê se desenha** (traçado, cor, forma, deformação, tudo não-destrutivo). Este volume dá o **como se organiza, reusa e anima**. A visão completa do PH2D emerge da composição:

- **Vetor pintado** (Vol. 1: gradient mesh, brushes, appearance stack) →
- **estruturado como grafo** (§1 vector networks) →
- **empacotado em prefab** (§3 components) →
- **parametrizado por tokens** (§4 variables, incl. data binding runtime) →
- **disposto responsivamente** (§2 auto layout) →
- **transicionado por estado** (§5 smart animate → state machine) →
- **tocado no engine** sem sair do canvas.

Isso é exatamente a síntese "vetor pintado → riggado → dirigido por state machine → tocado no engine" da tua tese — com a camada de **pintura profissional** (Vol. 1) que Rive/Figma não têm, e a camada de **sistema/comportamento** (Vol. 2) que Procreate/Illustrator não têm. O PH2D ocupa a interseção dos dois volumes.

---

## Referências principais

- Wallace, E. *Introducing Vector Networks* (Figma Blog/Medium) e *Delete and Heal for Vector Networks* (fit via Schneider).
- Harri, A. (2019). *The Engineering behind Figma's Vector Networks* (graph expansion, interseções).
- Figma Dev Docs — *VectorNetwork* (modelo vertices/segments/regions; multigrafo com identidade de aresta).
- Dalstein, B. et al. *Vector Graphics Complexes* (formalização acadêmica por half-edges).
- DioxusLabs — **Taffy** (Flexbox/Grid/Block em Rust; powers Dioxus, Zed, Bevy, Blitz).
- Badros, Borning & Stuckey — *Cassowary* constraint solver; impl. `kiwi` (nucleic) e `cassowary-rs`.
- Penpot — docs de *Components* (main/copy/override, variants, swap) e *Design Tokens* (formato W3C DTCG, aliases, math).
- W3C Design Tokens Community Group (DTCG) — formato de tokens.
- Figma Blog — *Announcing Smart Animate and advanced transitions* (matching por nome+hierarquia+ordem; combinar com transições).
- Vol. 1 deste manual (features de desenho vetorial): `kurbo`, `lyon`, `Clipper2`, `vtracer`, gradient mesh, appearance stack.
