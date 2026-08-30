# O alcance das timelines — o que elas conseguem animar hoje (2026-08-30)

> **Enio, 2026-08-30:** *"As timelines existentes devem funcionar com todos os modos de criação 2d
> e 3d."*
>
> ⚠️ A medição diz que isto **não é um trabalho, são três**, com preços muito diferentes — e o
> maior deles não é da Timeline.

---

## §1 — Quantas timelines existem: **quatro** faixas de tempo

| painel | o que é |
|---|---|
| `ph2d-panel-timeline` | **a** Timeline — dope-sheet, curvas, transporte, clips, nesting |
| `ph2d-panel-flip-frames` | a **tira de quadros** do Flip |
| `ph2d-panel-motion-graph` | o grafo de nós, que lê o playhead |
| `ph2d-panel-audio-editor` | a onda, que lê o playhead |

(`grep -rln 'Playhead' crates/ph2d-panel-*/src/`)

⭐ **As quatro já partilham o relógio** (`ph2d_core::Playhead`). *O relógio não é o problema.*

---

## §2 — ⛔ O que a Timeline sabe animar: **13 propriedades, e NENHUMA tem Z**

`crates/ph2d-timeline/src/prop.rs`:

```rust
pub enum PropKind {
    TranslationX, TranslationY,          // ⛔ sem Z
    Rotation,                            // ⛔ UM escalar, não um quaternião
    ScaleX, ScaleY,                      // ⛔ sem Z
    Opacity, TimeRemap, Morph, Position,
    JointMotorTarget, JointMotorSpeed,   // física 2D
    JointRestLength, JointMaxLength,
}
```

E o `SpriteProp` (5 variantes) repete os mesmos cinco canais 2D.

---

## §3 — ⭐⭐⭐ E a causa NÃO é da Timeline: o `Transform` da cena é 2D

`crates/ph2d-ecs` — o componente de pose que **todo objeto do mundo** carrega:

```rust
pub struct Transform {
    pub translation: Vec2,   // ⛔ Vec2, não Vec3
    pub rotation: f32,       // ⛔ um escalar
    pub scale: Vec2,
    pub skew_x: f32,
    pub skew_y: f32,
}
```

⇒ **A Timeline é 2D porque o MUNDO é 2D.** Ela anima exactamente o que existe para animar.
*Acusar a Timeline aqui seria acusar o instrumento pelo que a régua não tem.*

---

## §4 — ⭐⭐ Os 3D não são um caso: são DOIS, e só um está no mundo

| módulo | onde a pose 3D vive | está no ECS? | a Timeline alcança? |
|---|---|:--:|:--:|
| **3D Modeling** (SDF) | `FieldPose { Xform }` — componente **próprio** | ⭐ **sim** | ⛔ não |
| **3D / Sculpt** | `AppGfx.sculpt3d: Option<Sculpt3dScene>` — um **campo do app** | ⛔⛔ **não é entidade** | ⛔ não |

O `Xform` do campo (`crates/ph2d-field/src/xform.rs`):

```rust
pub struct Xform {
    pub translation: [f32; 3],
    pub rotation:    [f32; 4],   // ⭐ quaternião
    pub scale:       f32,        // ⚠️ uniforme, um escalar
}
```

⚠️ **São DOIS vocabulários de pose no mesmo app**, e nenhum é conversível no outro sem perda:
`Transform` tem skew que o `Xform` não tem; `Xform` tem um eixo e um quaternião que o `Transform`
não tem.

⛔⛔ **E a escultura não tem pose no mundo NENHUMA.** Medido: `grep -rn 'Sculpt' crates/ph2d-ecs/`
devolve **zero**. Ela vive num campo do estado do app, fora da hierarquia — logo **nada** na cena a
pode endereçar, muito menos animá-la. *Não é que a Timeline não a alcance: é que não há o que
alcançar.*

---

## §5 — ⇒ O pedido do Enio parte em TRÊS, com preços diferentes

| # | alvo | estado | o que falta |
|---|---|---|---|
| **1** | **2D** (sprite · vetor · Flip · física) | ✅ **funciona hoje** | nada — 13 propriedades vivas |
| **2** | **3D Modeling** (SDF) | ⏳ o objecto **está** na hierarquia com pose própria | a Timeline aprender um **segundo vocabulário** (`FieldPose`/`Xform`): canais Z, rotação por quaternião, escala uniforme. Trabalho **real e delimitado** |
| **3** | **3D / Sculpt** | ⛔⛔ **não é entidade** | ⭐ **primeiro** a escultura tem de entrar no mundo (uma ponte-componente, como o `PaintedDoc` faz pelo Painter). Só depois a pergunta da Timeline existe |

⭐⭐ **O item 3 é pré-requisito do item 3, não parte dele.** Enquanto a escultura for um campo do
app, ela é inalcançável por **tudo** — não só pela Timeline: também pelo undo por-componente, pela
persistência do mundo, pelas instâncias, e por qualquer coisa que pergunte *«que objectos existem?»*.

⚠️ **E há precedente exacto para a cura:** `ph2d_ecs::PaintedDoc(u32)` é a ponte do Painter —
*"não põe pixels no ECS; o `PainterTool` continua dono"*, e o componente carrega **só a identidade
estável**. O mesmo molde serve para a escultura.

---

## §6 — A decisão que isto força, e que é do Enio

Para o item 2 há **duas saídas**, e elas divergem no que custam ao resto do app:

- **(a) A Timeline aprende o segundo vocabulário.** Canais novos no `PropKind` que escrevem em
  `FieldPose`. ⭐ Barato e local; ⛔ o app fica com **duas** noções de pose para sempre, e todo
  sistema futuro (instâncias, física, sinais) escolhe uma.
- **(b) Unificar o `Transform` em 3D.** ⛔⛔ **Foundational profundo** — `Transform` é o componente
  mais carregado do repo, viaja em todo `WorldSnapshot`, e a física 2D (`rapier2d`) fala Vec2.
  ⚠️ Isto **não** é uma decisão desta linha; é um ADR, e provavelmente uma jornada própria.

⏳ **Não medido:** quantos sítios leriam um `Transform` 3D. Sem esse número a (b) não tem preço, e
⛔ **uma escolha entre (a) e (b) sem ele seria escolher em vez de contar** (`CLAUDE.md` §0.0).

---

## §7 — O que isto muda no modelo de áreas

⭐ **Nada de estrutural — e isso é a boa notícia.** A exigência *«as timelines funcionam em todos
os modos»* traduz-se, no modelo, em **uma linha**: a Timeline é uma **área** que pode ocupar o
encaixe `BOTTOM` em **qualquer Layout**, e liga-se ao que está seleccionado.

⚠️ E ela reforça uma escolha já tomada: **a área é do Layout, o conteúdo é da selecção.** Uma
timeline que só existisse no Layout *Animação* seria o mesmo erro dos 9 toggles de módulo — um
sistema alcançável só a partir de um sítio.
