# A engine é 2.5D — a articulação do Enio, e o que ela resolve (2026-08-30)

> **Enio, 2026-08-30:**
> *"Essa engine será uma engine 2.5d mais 2d que 3d. O canvas no runtime será 2d e a unidade
> principal é o pixel. Mas objetos 3d serão desenhados sobre o canvas 2d e esses objetos existirão
> e serão animados em 3d, ou seja, teremos uma sobreposição de canvas 3d ao 2d. O que teremos com
> isso é a riqueza do volume, da luz e da textura 3d se movendo, contudo, em canvas 2d. Então para
> objetos 3d teremos todas as coordenadas 3d (pos, rot e scale, além de deformações e animações)."*

---

## §1 — ⭐⭐⭐ Isto RESOLVE a pergunta que eu ia medir, e a resposta é «nenhuma das duas»

Eu tinha posto a decisão como uma escolha entre duas saídas
([`medicoes/04 §6`](../medicoes/04_o_alcance_das_timelines.md)):

- **(a)** a Timeline aprende um segundo vocabulário de pose — *"barato e local; o app fica com duas
  noções de pose **para sempre**"*
- **(b)** o `Transform` sobe para 3D — *"foundational profundo"*

⛔ **A moldura estava errada.** Eu apresentei (a) como um remendo e a «duas noções de pose» como um
preço a pagar. Com a engine sendo **2.5D por desenho**, duas noções de pose **é a arquitetura**:

| | pose | unidade |
|---|---|---|
| objecto **2D** | `Vec2` + um ângulo + `Vec2` de escala | **pixel** |
| objecto **3D** | `[f32;3]` + rotação nos três eixos + escala | a sua própria |

⇒ **O `Transform` NÃO sobe para 3D.** Ele descreve o canvas, e o canvas é 2D em pixels — que é o
que o Enio acabou de fixar. Um `Transform` 3D aplicado a toda a cena tornaria 3D o que é
deliberadamente plano.

⭐ **E a medição que eu ofereci fica CANCELADA** — contar quantos sítios leem o `Transform` era a
régua da opção (b), e a (b) não é o caminho. *Uma medição só vale enquanto a pergunta que ela
responde continua a ser a pergunta.*

---

## §2 — ⭐⭐ E a direcção já está implementada, para um caminho

Não é rumo novo: é a **articulação** do que o módulo Sculpt já faz.

- **Uma luz, dois materiais.** `ph2d-mesh-render`: *"as mesmas quatro lâmpadas que iluminam a tinta
  do Painter, resolvidas pela mesma função (`ph2d-light`), com o mesmo piso ambiente e o mesmo
  modelo RELATIVO"* — e o `CLAUDE.md` §5 diz a razão de existir do módulo: **a malha DOA a normal**,
  e a tinta 2D chapada sai acesa pela forma.
  ⭐ *É exactamente «a riqueza do volume e da luz, em canvas 2D».*
- **O passe 3D compõe no MESMO alvo, sem tocar no compositor 2D.** `MeshRenderer::new(device,
  ph2d_render::GameRt::FORMAT)`, e o doc-comment da crate: *"apagar esta crate apaga o 3D da tela
  **sem tocar no compositor 2D**"*.

⇒ **A sobreposição já existe.** O que não existe é o *objecto*.

---

## §3 — ⛔ O que falta, medido: hoje o 3D é uma TELA CHEIA, não um objecto

`crates/ph2d-mesh-render/src/pipeline.rs:43`:

```rust
/// `(largura, altura, 0, 0)` — ver [`CameraRaw::viewport`].
fn viewport_of(size: (u32, u32)) -> [f32; 4]
```

E no modelador SDF, `shells/desktop/src/field_gizmo.rs:258`:

```rust
canvas: ph2d_editor::zones::Rect::new(0.0, 0.0, win_w, win_h)
```

⇒ **Os dois módulos 3D tomam a janela inteira.** Hoje o 3D é um **modo de edição em ecrã cheio**,
não uma coisa que se coloca no canvas 2D e tem tamanho e lugar.

---

## §4 — ⭐⭐ A consequência de desenho: um objecto 3D tem **DUAS** poses, e é isso que 2.5D significa

| pergunta | responde | unidade |
|---|---|---|
| **onde na página?** | uma pose **2D** — posição no canvas, tamanho | **pixel** |
| **como está virado?** | uma pose **3D** — rotação nos 3 eixos, escala própria, deformação | a sua |

⭐ **Não são duas noções em conflito: são duas perguntas diferentes.** É o mesmo modelo de uma
camada 3D numa composição 2D — a camada tem um sítio na tela, e o conteúdo dela tem uma orientação
no espaço.

⇒ **Um objecto 3D carrega o `Transform` (onde, em pixels) E o componente de pose 3D.** O
`Transform` não deixa de servir; ele passa a responder «onde na página» também para objectos 3D.

⚠️ **E isto arruma a Timeline sem contradição:** o `PropKind` ganha **canais 3D** (`TranslationZ`,
rotação por eixo ou quaternião, `ScaleZ`, deformação), **gateados ao tipo do objecto** — que é
exactamente a lei da **D6**: *os canais disponíveis dependem do tipo*, como os modos. Os 13 canais
2D ficam onde estão.

---

## §5 — O que NÃO muda

⛔ **A escultura continua a ter de virar ENTIDADE primeiro**, e a razão ficou mais forte, não mais
fraca. Se um objecto 3D vai ter lugar no canvas, tamanho, animação e deformação, ele **é** um
objecto do documento — e hoje `grep -rn 'Sculpt' crates/ph2d-ecs/` devolve **zero**: a escultura
vive em `AppGfx.sculpt3d`, um campo do estado do app.

⭐ O molde está pronto e é o mesmo de sempre: `PaintedDoc(u32)`, a ponte do Painter — *"não põe
pixels no ECS; o `PainterTool` continua dono"*, e o componente carrega **só a identidade estável**.

---

## §6 — ⏳ O que a articulação ABRE (e não estava na mesa antes)

1. ⭐⭐ **Um objecto 3D pode ficar ENTRE duas camadas 2D, ou só por cima?** É a pergunta que decide
   o compositor. *"Sobreposição de canvas 3D ao 2D"* lê-se como **por cima**; mas um personagem 3D
   atrás de um cenário 2D é o caso normal de um jogo 2.5D. ⛔ **Pergunta de produto, não medida.**
2. **Qual é a unidade do espaço 3D, e como se converte em pixel?** O 2D é pixel; o `Xform` do
   modelador tem escala **uniforme escalar**; a física tem `ProjectSettings::pixels_per_meter`.
   ⏳ Três réguas, e ninguém as ligou.
3. **«Deformações»** — o Enio nomeou-as ao lado de pos/rot/scale. Deformar uma malha animada é
   *skinning*, *shape keys* ou *lattice*, e são três obras diferentes. ⛔ **Nomeado, não desenhado.**
4. **Uma câmera ou várias?** Se cada objecto 3D tem lugar próprio no canvas, ou partilham uma
   câmera 3D da cena, muda o que se pode compor.

---

## §7 — O que isto muda no modelo de áreas

⭐ **Nada de estrutural.** O modelo de seis encaixes e regiões continua igual — a 2.5D é uma
propriedade do **documento**, não do chrome.

⚠️ **Mas muda um item da D6:** a tabela de modos dizia *malha 3D → Object · Edit · Sculpt · Paint*
com o **Object** a significar «mover/rodar/escalar» em 2D. Para um objecto 3D o modo **Object** tem
de responder às duas perguntas do §4 — e é aí que o gizmo de 3 eixos vive. ⏳ Não desenhado.
