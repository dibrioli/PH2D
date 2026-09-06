# ADR-0169 — O esqueleto é um MÓDULO PRÓPRIO, e cada mídia responde só a *«o que é um ponto aqui»*

- **Status:** Aceito (2026-09-06)
- **Contexto:** `line/Vector`, wave pós-integração. Decisão do Enio em 2026-09-06, com as palavras
  dele: *«não seria melhor transformar Bones em módulo próprio? Pois será aplicado não só em vetor
  mas em raster, 3d, Flip, etc.»*
- **Sob:** [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) (drop-crate,
  desacoplar por ECS) · [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md) (uma
  hierarquia só) · [ADR-0108](0108-vector-reposition-rive-referenced-native-editor-first.md) (o Rive
  como referência declarada do motor vectorial).

---

## 1 — O problema

Os ossos nasceram em 2026-09-06 dentro do módulo vectorial (estudo 42 item 5, doc 47) e ficaram
corretos: o osso é uma **entidade** com `Transform`, a cinemática directa é a `propagate_transforms`
que a casa já corre, e os pesos são **derivados** do bind a cada quadro. Nada disso é do vetor.

Mas **tudo estava endereçado como se fosse**:

| peça | onde vivia | o que ela sabia do vetor |
|---|---|---|
| a lei LBS | `ph2d-vec-skin` | **nada** — só `Xform` e pontos; e `apply(&mut VecPath)` |
| o osso e a pele | `ph2d-ecs` (`VecBone`/`VecSkin`) | **nada** — a fonte já viajava em bytes opacos |
| o desenho do osso | `ph2d-vec-render::bone` | **nada** — só `ph2d-vector` e `ph2d-tokens` |
| o afim `Xform` | `ph2d-vec-scene` | **nada** — seis `f64` e álgebra |

⚠️ **A prova de que o `Xform` já era foundational-de-facto é um censo:** `ph2d-field`,
`ph2d-field-ecs`, `ph2d-field-eval`, `ph2d-field-render`, `ph2d-flip` e `ph2d-tool-painter` já
escreviam `ph2d_vec_scene::Xform` — **seis crates sem nada de vectorial**, a puxar a cena inteira
para multiplicar duas matrizes.

## 2 — A evidência do mercado, e ela é unânime

Um esqueleto só serve várias mídias, e a metade que muda por mídia é pequena:

| Referência | O MESMO esqueleto deforma |
|---|---|
| **Blender** | malha, curva, texto, superfície, treliça **e desenho 2D** (Grease Pencil) |
| **Moho** | camada vectorial, **camada de imagem** e camada 3D (*Bind Layer* / *Bind Points*) |
| **Rive** | formas vectoriais **e** imagens (por malha) — `Skin` + `Tendon` |
| **Spine** | malhas de imagem |

⭐ **E o Blender diz onde fica a costura:** o *Armature modifier* é **um só** e o Grease Pencil tem a
implementação dele **à parte** — porque *o que é um ponto* ali é outra coisa. É exactamente o corte
que este ADR faz.

## 3 — A decisão

**O esqueleto é um módulo, em quatro crates, e o vetor é o PRIMEIRO CLIENTE.**

| crate | o que é | depende de |
|---|---|---|
| `ph2d-affine` | o afim 2D `f64` (`Xform`), extraído | **nada** |
| `ph2d-skeleton` | **a LEI** — pesos, órfão, LBS, `deform_points` | `ph2d-affine` |
| `ph2d-skeleton-ecs` | `Bone` · `SkinBind` · `Tendon` + `register_skeleton_components` | `ph2d-ecs` |
| `ph2d-skeleton-render` | o losango do osso e o raio da junta | `ph2d-vector`, `ph2d-tokens` |
| `ph2d-vec-skin` | **o 1.º cliente** — as três metades de um vértice | `ph2d-skeleton`, `ph2d-vec-scene` |

⚠️ **`ph2d-vec-scene` RE-EXPORTA o `Xform`** (`pub use ph2d_affine::Xform;`), pelo precedente exacto
do `ph2d-arclen` que já vive no mesmo ficheiro de manifesto ⇒ **os ~300 sítios que escrevem
`ph2d_vec_scene::Xform` não mudam uma letra.** É uma mudança de DONO, não de API.

⚠️ **Os componentes registam-se pela porta do módulo**, espelhando a `ph2d-physics-ecs`
(`register_physics_components`, chamada em `init.rs`). ⛔ Sem essa linha o `WorldSnapshot`
**descarta-os em silêncio** — é o bug `Locked`/`GroupedChildren`/`VecPathRef` que a física já pagou,
e aqui apareceria como *«o personagem perdeu o esqueleto ao desfazer»*.

⭐ **A categoria de componente `Skeleton` é própria**, e não uma prateleira do `Vector`: declará-lo
sob `Vector` prometeria ao artista que ele só serve caminhos.

### 3.1 — Os nomes canónicos deixam de dizer "vector", e a janela era AGORA

O `ComponentBlob` é endereçado por **`blake3(nome canónico)`**. Trocar
`ph2d::ecs::VecBone` → `ph2d::skeleton::Bone` e `ph2d::ecs::VecSkin` → `ph2d::skeleton::Skin` faria,
com projectos gravados, cada um deles **perder o esqueleto em silêncio** ao abrir.

⭐ **MEDIDO em 2026-09-06:** os dois `.ph2dproj` da máquina do dono são de **26/08** — onze dias
antes de os ossos existirem ⇒ nenhum tem esqueleto, e a troca custou **zero**. Há gate
(`the_canonical_names_belong_to_the_module_not_to_one_medium`) para que ninguém devolva a palavra ao
nome sem reabrir esta conta.

⚠️ **O `PROJECT_SCHEMA` NÃO se mexe**, e pela mesma razão de sempre: o `ComponentBlob` é chaveado
por nome, então um ficheiro antigo simplesmente não tem estes blobs. O censo do `ph2d-ecs` **desce**
de `81` para `79` (um componente que SAI conta tanto como um que entra), e os dois censos-espelho
(`ph2d-render`, `ph2d-script`) de `82` para `80`.

### 3.2 — Os nomes dos TIPOS

| stored (ECS) | resolvido (lei, por quadro) |
|---|---|
| `Bone` · `SkinBind` · `Tendon` | `Skin` · `SkinBone` |

⚠️ **`SkinBind` e o rótulo "Skin" são DELIBERADAMENTE diferentes:** o tipo diz o que se **guarda**
(o bind — a fonte mais as matrizes de repouso), o rótulo diz o que a coisa **É**. O catálogo do
`ph2d-component-desc` é onde os dois se encontram, e é ele que impede a colisão de `Skin` entre a
lei e o componente aparecer no mesmo `use`.

⭐ **`Tendon` é a palavra do Rive**, que chama exactamente isto exactamente assim — e o doc do campo
`rest` já lhe chamava «o TENDÃO» antes de a crate existir.

## 4 — O que NÃO mudou (e é o gate desta wave)

⛔ **Nada na tela.** O gesto, o painel, o overlay, o bind, os dois verbos de soltar e o smoke
`PH2D_VEC_BONE_SMOKE` são os mesmos. A única diferença visível é que a paleta do `+` passa a ter um
grupo **Skeleton** em vez de listar o osso e a pele sob *Vector*.

⛔ **A lei não mudou uma linha.** Os oito gates dela migraram com ela, reescritos sobre **pontos
crus** em vez de `VecPath` — e um gate novo (`the_media_door_is_the_same_law_as_the_hand_written_loop`)
prova ao BIT que a porta de mídia `deform_points` é `point` num laço, para que uma optimização futura
lá dentro não passe despercebida a toda mídia.

## 5 — Consequências

- **Raster, 3D e Flip custam UMA crate cada**, e ela responde a uma pergunta só: *o que é um ponto
  aqui, e onde mora a fonte*. ⛔ Nenhum deles toca a lei, os componentes ou o desenho.
- **O `Xform` ganhou dono**, e as seis crates não-vectoriais que já o usavam deixam de arrastar a
  cena vectorial atrás dele. (Elas continuam a escrevê-lo pelo re-export; migrá-las é opcional.)
- ⏳ **O painel do módulo é a etapa seguinte.** Hoje a secção *SKELETON* e o modo *Bone* continuam
  dentro da ferramenta vectorial, e os ids de a11y ainda dizem `vector.mode.bone` — **isso é
  honesto** enquanto o modo viver ali. ⛔ Aqueles ids **não** viajam em ficheiro nenhum, então
  renomeá-los depois continua grátis; o que **não** era grátis depois é o nome canónico do
  componente, e é por isso que só ele foi trocado agora.
- ⏳ **O IK entra DEPOIS, dentro do módulo** — e não do zero: a família `ph2d-node-rig-*` já traz
  FK, IK de duas juntas (lei dos cossenos **sem `acos`**, HR-5), FABRIK, mangueira de borracha e um
  deformador. ⚠️ **Ela é outro SUBSTRATO** (um esqueleto ali é uma *stream* de instâncias do grafo de
  nós, não entidades) — ⛔ não a confunda com este módulo. ⭐ Mas os três leaves dela
  (`fk.rs`/`pose.rs`/`trig.rs`) estão hoje em **15 cópias byte-idênticas** em 6 crates, com o doc a
  declarar que *«a cópia não pode divergir, é o contrato dela»*: a `ph2d-skeleton` é a casa que acaba
  com isso, e é o item que a folha 16 da conferência do Motion pedia sem ter onde o pôr.

## 6 — Alternativas consideradas

- ⛔ **Deixar a lei em `ph2d-vec-skin` e fazer o raster depender dela.** Rejeitado: o raster passaria
  a compilar a cena vectorial inteira para misturar dois afins, e a primeira feature de vetor que
  mexesse na `ph2d-vec-scene` recompilaria o módulo de imagem.
- ⛔ **A `ph2d-skeleton` declarar o próprio afim de seis floats.** Rejeitado: a composição
  `rest⁻¹ ∘ osso ∘ forma⁻¹` passaria a existir em dois sítios, e *duas portas divergem em silêncio*
  — que é a lei que o próprio `SkinBone::new` já tinha escrita no doc dele.
- ⛔ **Manter os componentes no `ph2d-ecs` e só mover a lei.** Rejeitado: a fundação cresceria um
  componente por mídia, e o precedente da `ph2d-physics-ecs` mostra que uma crate satélite pode
  possuir e registar os dela sem perder undo, save, olho, cadeado, reparentar nem timeline.
- ⛔ **Adiar a troca dos nomes canónicos para quando o painel existir.** Rejeitado **com medição**:
  ela é grátis exactamente hoje (zero projectos gravados com esqueleto) e destrutiva a partir do
  primeiro personagem que o dono salvar.
