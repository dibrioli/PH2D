# 00 — O esqueleto vira MÓDULO

> **Decisão:** [ADR-0169](../architecture/decisions/0169-the-skeleton-is-its-own-module-and-each-medium-answers-only-what-a-point-is.md) ·
> **Nascimento:** 2026-09-06, `line/Vector` · **Origem:** os ossos do vetor
> ([doc 47 do Vector Module](../Vector%20Module/47_o_desenho_ganha_ossos.md), estudo 42 item 5)

---

## §1 — O que este módulo é

Um **esqueleto** que deforma qualquer coisa. Ele responde a **uma** pergunta —
*«para onde vai um PONTO»* — e cada mídia responde, do lado dela, a *«o que é um ponto aqui»*.

⭐ **Isto não é uma invenção nossa: é o consenso das quatro referências.**

| Referência | O MESMO esqueleto deforma | Como a metade por-mídia aparece lá |
|---|---|---|
| **Blender** | malha, curva, texto, superfície, treliça **e desenho 2D** | *Armature modifier* é UM; o Grease Pencil tem a implementação dele **à parte** |
| **Moho** | camada vectorial, **de imagem** e 3D | *Bind Layer* · *Bind Points* · *flexible/region binding* |
| **Rive** | formas vectoriais **e** imagens | `Skin` + `Tendon`, com peso por vértice **e por alça de Bézier** |
| **Spine** | malhas de imagem | pesos por vértice de malha |

## §2 — As peças, e o que cada uma pode saber

| crate | o que é | ⛔ o que ela NÃO pode conhecer |
|---|---|---|
| [`ph2d-affine`](../../crates/ph2d-affine/) | o afim 2D `f64` (`Xform`) | nada — **zero dependências**, e é o ponto |
| [`ph2d-skeleton`](../../crates/ph2d-skeleton/) | **a LEI** (pesos · órfão · LBS · `deform_points`) | o que é um caminho, uma malha ou um pixel |
| [`ph2d-skeleton-ecs`](../../crates/ph2d-skeleton-ecs/) | `Bone` · `SkinBind` · `Tendon` + o registador | idem — a fonte viaja em **bytes opacos** |
| [`ph2d-skeleton-render`](../../crates/ph2d-skeleton-render/) | o losango do osso e o raio da junta | o documento de qualquer mídia |
| [`ph2d-vec-skin`](../../crates/ph2d-vec-skin/) | **o 1.º CLIENTE** — as três metades de um vértice | — (é o lado que sabe do vetor) |

## §3 — As leis que atravessam qualquer cliente

- ⭐ **O osso é uma ENTIDADE.** A cinemática directa é a `propagate_transforms` que a casa já corre;
  undo, save, olho, cadeado, renomear, reparentar e a timeline a animar um osso vêm **de graça**.
  ⛔ Uma árvore de ossos dentro de um componente seria uma segunda hierarquia — o que a
  [ADR-0110](../architecture/decisions/0110-vector-nodes-are-ecs-entities-one-hierarchy.md) rejeita
  pelo nome.
- ⭐ **Os PESOS não se guardam — derivam-se** do bind a cada quadro. Uma tabela indexada por ordem
  de varredura é o *vector paralelo* que o doc do `VecVertex::corner_radius` proíbe por escrito, e
  editar a forma re-pesaria errado. Custo medido: **0,146 % de um quadro** (200 vértices × 12 ossos).
- ⭐ **O repouso é a IDENTIDADE, e cai da álgebra** — `M_j = S⁻¹ ∘ B_j ∘ rest⁻¹` com
  `rest_j = S_bind⁻¹ ∘ B_bind` ⇒ prender **não move um pixel**, sem uma guarda escrita à mão.
- ⭐ **A força é um MÚLTIPLO do comprimento do osso**, nunca uma distância — é o que torna a lei
  adimensional (o mesmo rig dez vezes maior deforma-se igual), e é o *Bone Strength* do Moho.
- ⛔ **Nada de podar pesos pequenos.** O bump `(1 − (d/r)²)²` é C¹ na borda de propósito; um piso
  devolveria o estalo que ele existe para evitar.
- ⛔ **Um ponto ÓRFÃO prende-se rigidamente ao osso mais próximo** (o *point binding* do Moho). Com
  uma lei de suporte infinito (`1/d²`) ele seguiria a **média** do esqueleto, e a aba de um chapéu
  atrasa-se atrás da cabeça.

## §4 — ⚠️ A OUTRA família de rig deste repo (não a confunda)

Existem seis crates [`ph2d-node-rig-*`](../../crates/) — `skeleton` · `fk` · `ik_2bone` · `fabrik` ·
`rubber_hose` · `skin_deformer` — feitas em 2026-07-12 para os **Motion Nodes**, com a conferência
delas contra Rive/Spine/Blender em
[`89_conferencia/16_rig.md`](../Motion%20Nodes/89_conferencia/16_rig.md).

⚠️ **Elas são outro SUBSTRATO:** ali um esqueleto é uma *stream de instâncias* do grafo de nós
(colunas `parent`/`len`/`rot`), não entidades da cena. Servem motion graphics procedural; este módulo
serve personagens autorados. ⛔ **Não as apague e não as funda com este módulo sem ler aquela folha** —
a família está **deferida por decisão do Enio** com 18 células em ⏸️.

⭐ **O que este módulo pode HERDAR delas é a MATEMÁTICA**, e ela é boa: o IK de duas juntas é exacto
(lei dos cossenos **sem `acos`**, HR-5) e o FABRIK é o padrão-ouro para cadeia longa.
⚠️ E há um item que só uma casa comum resolve: os três leaves `fk.rs`/`pose.rs`/`trig.rs` vivem hoje
em **15 cópias byte-idênticas** nas 6 crates, com o doc a declarar que *«a cópia não pode divergir, é
o contrato dela»*.

## §5 — ⏳ O que está ABERTO, na ordem

1. **O painel próprio.** Hoje a secção *SKELETON* e o modo *Bone* vivem na ferramenta vectorial, e os
   ids de a11y dizem `vector.mode.bone` — **honesto enquanto o modo viver ali**, e grátis de renomear
   depois (⛔ eles não viajam em ficheiro nenhum, ao contrário do nome canónico do componente).
   Junto com ele: **arrastar a força no canvas** (a região de influência como mancha, o *Bone Strength*
   do Moho) — hoje é um número no painel, e a mancha é o que o artista usa.
2. **O IK**, e as quatro coisas que o cercam nas referências e que nós **não temos**:
   **Força/Mistura** (Rive põe em 7 de 7 constraints, Spine em 4 de 4 — nós temos zero) ·
   **Softness** (o amortecimento na extensão máxima, sem o qual o joelho estala) ·
   o **cotovelo por alvo** em vez de um bit de lado · **limite de ângulo por junta**.
3. **Smart Bones** (Moho): girar um osso **toca uma animação inteira** — é o que faz uma cabeça virar
   sem entortar.
4. **A segunda mídia** (raster / Flip / 3D) — uma crate cliente cada, respondendo a uma pergunta só.

## §6 — Smoke

O do vetor continua a ser o do módulo enquanto ele for o único cliente:

```
env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Diagnóstico: `PH2D_BONE_LOG=1` (responde às três perguntas que um report de *«não deforma»* não
distingue — *há pele? há osso vivo? a matriz é a identidade?*).
