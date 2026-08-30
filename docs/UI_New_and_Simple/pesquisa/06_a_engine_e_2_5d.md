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
| objecto **2D** | `Vec2` + um ângulo + `Vec2` de escala | **metros** (⚠️ §1-bis) |
| objecto **3D** | três eixos + rotação nos três + escala | **metros** |

⇒ **O `Transform` NÃO sobe para 3D.** Ele descreve o **plano** do canvas, e o canvas é 2D — que é o
que o Enio acabou de fixar. Um `Transform` 3D aplicado a toda a cena tornaria 3D o que é
deliberadamente plano.

### §1-bis — ⛔ CORREÇÃO minha: eu escrevi «em pixels», e o `Transform` já está em METROS

A primeira redacção desta tabela dizia que o objecto 2D se posiciona **em pixels**. Está errado, e
o código di-lo na cara (`crates/ph2d-ecs/src/transform.rs:55`):

> *"Local-space 2D affine: translation (**meters**), rotation (**radians**, CCW from +X)"*

E o `CLAUDE.md` §5 já o afirmava, no módulo de Física: *"**Sem porta de escala:** o `Transform` já
é metros; a única px→m é `ProjectSettings.pixels_per_meter`, do projeto"* (ADR-0131).

⭐⭐ **A correcção torna a resposta do Enio MAIS simples, não mais complexa.** Ele respondeu
*"temos no app pixel/metro, logo em 3d usaremos metros"* — e **a cena já é métrica por inteiro**.
Não há ponte de unidade para inventar: o 3D só acrescenta **um eixo à mesma régua**.

⚠️ **E as duas frases dele não se contradizem** — vale escrever porquê, porque lidas juntas parecem
contradizer-se. *"A unidade principal é o pixel"* é sobre a **arte**: a resolução do canvas, o
tamanho de uma sprite, o `pixels_per_meter` do projecto. *Metros* é a unidade da **cena**. São dois
níveis, com uma ponte que já existe e tem dono declarado.

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

## §6 — As quatro perguntas, RESPONDIDAS pelo Enio (2026-08-30)

### 6.1 — z-index: **o objecto 3D fica ENTRE camadas**

> *"O objeto 3d terá um z-index como o 2d, logo ficará entre camadas."*

⭐⭐ **O mecanismo JÁ EXISTE** — `crates/ph2d-ecs/src/sorting.rs:43`:

```rust
/// Manual Z override (Godot `z_index`). Presença força a ordenação;
pub struct ZIndexOverride(pub i32);
```

…com recuo para o contador de DFS quando não há override. É literalmente o modelo do Godot, já
implementado. *A resposta dele cai sobre maquinaria pronta.*

⛔⛔ **MAS a consequência de renderização é grande, e é o achado desta secção: o 3D deixa de poder
ser um passe FINAL.** Hoje ele compõe **depois** do 2D, no mesmo alvo (§2). Para ficar *entre*
camadas, o objecto 3D tem de **desenhar para uma textura própria** e entrar na pilha **como uma
camada**, ordenada pelo mesmo `ZIndexOverride` de todas as outras.

⭐⭐⭐ **E esse primitivo é EXACTAMENTE o que o *W-Saída* do Flip já pede:** *"assar um quadro em
pixels destrava três features que já existem do outro lado… **É UM buraco, não três:** a entidade
de um objeto Flip não tem `Sprite` nem pixels, e as três portas só sabem o que é um pixel"*
(`docs/Flip/01_plano_waves.md`). ⇒ **um primitivo, três consumidores** — o Flip, os exportadores, e
agora o 3D-entre-camadas.

⚠️ **E ele NÃO contradiz a recusa do `pesquisa/05`** (onde assar era a cura errada para pintar
sobre vetor). A diferença é o tempo de vida: ali seria um **assado guardado**, que mata a
editabilidade; aqui é um **render por quadro**, e o objecto 3D continua 3D. *Assar e renderizar
lêem-se igual e não são a mesma coisa.*

### 6.2 — Unidades: **metros** para o espaço, **graus** para o giro

> *"Temos no app pixel/metro, logo em 3d usaremos metros. Para rot graus."*

⭐ **Não há ponte a inventar: a cena já é métrica** (§1-bis). O 3D acrescenta um eixo à mesma régua,
e o `pixels_per_meter` do projecto continua a ser a única px↔m.

⚠️ **«Graus» é unidade de AUTORIA, não de armazenamento.** O `Transform` guarda radianos
(`transform.rs:55`) e a matemática toda assume-os. ⇒ os graus vivem no **painel e na timeline**, com
a conversão na porta — que é a convenção que o app já usa. ⛔ Guardar graus faria toda função
trigonométrica converter, e é onde nascem erros de arredondamento acumulados.

⛔ **E há uma lacuna concreta:** o `Xform` do modelador tem **escala UNIFORME**
(`pub scale: f32`, `ph2d-field/src/xform.rs`). O Enio pediu *"pos, rot e scale"* em 3D ⇒ a escala
tem de virar **três números**. É uma mudança de formato, com degrau de migração.

### 6.3 — Deformações: **as três**

> *"as 3"* — em resposta a *«é esqueleto, poses-alvo, ou gaiola?»*

| | é | usa-se para |
|---|---|---|
| **Esqueleto** (*skinning*) | ossos + pesos por vértice | personagens, membros, articulação |
| **Poses-alvo** (*shape keys* / *blend shapes*) | malhas alternativas misturadas por peso | expressões faciais, correcções |
| **Gaiola** (*lattice*) | uma grelha exterior que deforma o que está dentro | esticar, achatar, deformação de conjunto |

⛔ **São três obras independentes, e nenhuma está desenhada.** Ficam **nomeadas**, com o aviso de
que a ordem entre elas importa: o esqueleto é o que a timeline anima com mais frequência, e as
outras duas compõem-se **sobre** ele.

### 6.4 — Câmera: **por objecto, com uma da cena por omissão** — e esta é uma resposta com razão

> *"Não sei dizer. Mas será a que dá mais possibilidades de usos."*

⭐⭐ **O critério que ele deu decide a questão sozinho, e a favor de por-objecto** — porque a
relação entre as duas não é simétrica:

- Com **câmera por objecto**, «todos partilham a mesma» exprime-se: cada objecto aponta para a
  mesma. ⇒ o caso da câmera única é um **caso particular** do modelo geral.
- Com **uma câmera da cena**, «este objecto tem a sua» **não se exprime de todo**.

⇒ *O modelo geral contém o particular; o particular não contém o geral.* Sendo «mais
possibilidades» o critério, a resposta é **por objecto** — com uma câmera de cena como o **valor
por omissão** para que o caso comum não custe uma decisão ao artista.

⭐ **E há um ganho concreto de 2.5D nisto:** um adereço pode ser quase ortográfico enquanto uma
personagem tem perspectiva forte, na mesma cena. Com uma câmera só, a composição inteira fica presa
a uma projecção — que é precisamente a rigidez que se paga ao pôr 3D dentro de 2D.

⚠️ **É a resposta desta linha, não uma decisão do Enio** — ele disse que não sabia. Fica marcada
como **derivada do critério dele**, e ⛔ reversível se o critério mudar.

---

## §7 — O que isto muda no modelo de áreas

⭐ **Nada de estrutural.** O modelo de seis encaixes e regiões continua igual — a 2.5D é uma
propriedade do **documento**, não do chrome.

⚠️ **Mas muda um item da D6:** a tabela de modos dizia *malha 3D → Object · Edit · Sculpt · Paint*
com o **Object** a significar «mover/rodar/escalar» em 2D. Para um objecto 3D o modo **Object** tem
de responder às duas perguntas do §4 — e é aí que o gizmo de 3 eixos vive. ⏳ Não desenhado.
