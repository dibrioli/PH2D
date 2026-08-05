# Plano — o Vector vira a ferramenta de UI/UX do PH2D

> **Volume de PLANO**, não de estudo. Os três manuais desta pasta (`_vetoriais`, `_figma`, `_rive`)
> dizem *o que existe no mundo*; este diz **o que construir aqui, em que ordem, com que porta única,
> e com que número ao lado**. Ele consome o Vol. 2 (Figma) inteiro, mais o §7 do Vol. 3 (Rive:
> layouts responsivos) e o item aberto *booleana não-destrutiva animável*.
>
> **Pedido do Enio (2026-08-01), verbatim no que decide:** *"tornar o nosso módulo Vector em algo tão
> eficiente e capaz como o Figma para gerar UI/UX/interfaces que podem ser usadas diretamente para
> criação de apps funcionais … deve ser capaz de gerar arquivos UI/UX úteis para esse nosso projeto
> (PH2D engine) pois a versão final da UI eu mesmo desenharei … UI animada, responsiva e com layout
> responsivo … também será usada para criar a UI dos games."*
>
> Escrito na `line/Vector` em 2026-08-01. Todo número aqui foi **medido nesta worktree** ou tem a
> linha de código citada; o que não foi medido está marcado **MEDIR** e é pré-requisito da wave.

---

## §0 — O pedido, e as três coisas que ele decide

A frase do Enio tem três metades, e cada uma corta o plano num lugar diferente:

1. **"gerar UI que pode ser usada diretamente para criação de apps funcionais"** — o produto não é
   um *mockup*. Um retângulo azul com a palavra "Salvar" não é um botão; é o desenho de um botão. A
   diferença entre este plano e o Vol. 2 do Figma inteiro está no **§2**.
2. **"gerar arquivos UI/UX úteis para esse nosso projeto … a versão final da UI eu mesmo desenharei"**
   — o consumidor nº 1 é **o próprio editor do PH2D**, que é Rust escrito à mão com 44 widgets, um
   `Panel` tipado e uma bateria de gates de costura. Um exportador que cospe SVG não serve para nada
   aqui. O que serve está no **§4/W6 e W8**.
3. **"UI animada, responsiva e com layout responsivo … também para os games"** — o consumidor nº 2 é
   o **runtime**, que não tem editor, não tem painel e não pode pagar um passe de autoria. Isso
   obriga o modelo a ser **dado**, não código, e obriga a resolução (layout, token, estado) a ser um
   passe determinista dentro de um `advance(dt)` (Vol. 3 §6).

E há uma quarta coisa, que o Enio não precisou dizer porque é a lei do repo (CLAUDE.md §0): **o teto
é o do hardware, e todo limite se mede antes de se escrever.** Toda constante deste plano ou vem com
a medição ao lado, ou vem com a palavra **MEDIR** e a sonda que a produz.

---

## §1 — O CENSO: o que já existe (medido hoje), e o que falta

⚠️ **Este censo é a metade mais valiosa do documento.** Seis achados mudaram o plano depois de
escrito o primeiro esboço, e cinco deles mudaram-no *para menos trabalho*. Quem retomar isto: **não
re-derive o censo por leitura do Vol. 2** — ele descreve o Figma, não este repo.

### 1.1 O que JÁ EXISTE e não deve ser reconstruído

| Peça | Onde | Estado |
|---|---|---|
| Modelo de documento vetorial | `ph2d-vec-scene` | `VecScene` = pilha plana `Vec<VecPath>` + `next_id`; a **ÁRVORE** é a Hierarquia ECS (ADR-0110) |
| Identidade que sobrevive a undo e a save | `VecPathId = u64` (`lib.rs:376`) | ✅ e há **precedente explícito** (`vec_label.rs`, `vec_connector.rs`): *"`Entity::to_bits()` é id de ALOCAÇÃO, e o undo respawna"* |
| Pose = `Transform` da entidade, geometria LOCAL | ADR-0111 | ✅ |
| Fonte autorada ≠ geometria cozida | ADR-0121 (`VecPath::cooked()`) | ✅ — é o pré-requisito de tudo que é não-destrutivo |
| Pilha de efeitos não-destrutivos | ADR-0132 (`effect.rs`, 19 kinds) | ✅ |
| Geometria DERIVADA no z da fonte | `LiveGeometry = BTreeMap<VecPathId, Vec<VecPath>>` (`ph2d-vec-render/src/lib.rs:114`) | ✅ **6 produtores** (offset/pattern/contour/symmetry/profile/**align**) |
| Formas VIVAS paramétricas | `VecShape::Param{kind,w,h,values:[f64;8]}` | ✅ — forma nova entra no catálogo e já salva/desfaz/re-cozinha |
| **TEXTO** como objeto vivo | `VecShape::Text(VecTextParams)` | ✅ família, tamanho, peso, entrelinha, tracking, alinhamento, **eixos variáveis** |
| Booleana robusta | `ph2d-vec-boolean` (`apply`, `apply_many`, `Arrangement`) | ✅ mas **destrutiva** (consome os operandos) |
| Recorte de subárvore | `ph2d_ecs::{ClipChildren, ClipMode, Mask2D}` | ✅ — a moldura ganha *clip* de graça |
| **Tokens de design** | crate **`ph2d-tokens`** + `docs/design/tokens.json` | ✅ **350 folhas, 4 temas, OKLCH, `build.rs` que gera os consts, e um gate de sync com parser INDEPENDENTE** |
| Cor perceptual | `ph2d-color` (`oklab.rs`, `oklch.rs`) | ✅ |
| Easing | `ph2d-anim::{Easing, EasingFamily, EasingMode}` | ✅ (inclui `Elastic`, que é uma *curva*, não um solver de mola) |
| Timeline com clips, containers, faixas, crossfade, sinais | `ph2d-timeline` + `ph2d-anim` + painel | ✅ |
| Correspondência de formas + interpolação | `ph2d-vec-blend` (Hungarian + espiral logarítmica) | ✅ — **é meio Smart Animate já construído** |
| Widgets nativos | `ph2d-editor-core/src/widget/` | ✅ **44**: slider, dropdown, tree_view, color picker, tabs, modal, … |
| Contrato de painel | `Panel` (`panel/panel_trait.rs`): `State: Default`, `ID`, `NODE_ID`, `paint`, `apply_event`, `populate` | ✅ |
| Texto real (shaping) | `ph2d-text` (**parley** 0.6), `ph2d-vector-font` | ✅ |
| Envelope de arquivo por SEÇÕES | `line/runtime` (`37ff53467`, `LEGACY_SCHEMA_FINAL = 48`) | ⚠️ **noutra linha** — ver §6.3 |

### 1.2 O que NÃO existe (verificado por grep, não por memória)

| Falta | Prova |
|---|---|
| **Moldura / artboard / frame** | `grep -rn "artboard\|Artboard" crates/ shells/` → **0 ocorrências** |
| Motor de layout | `grep "taffy\|cassowary"` em todo `Cargo.toml` → **0** |
| Arena de ids estáveis | `grep "slotmap"` → **0** |
| **Largura de quebra do texto** | `VecTextParams` tem `size/weight/line_height/tracking/align/axes` e **nenhum `width`** ⇒ o texto não é uma CAIXA, é uma linha |
| Componentes / instâncias / variants | nenhum componente ECS deles (registro tem **41**, `scene/registry.rs:374`) |
| Tokens no DOCUMENTO do artista | o `fill` de uma forma é `Paint::Solid(Rgba8)` **literal**; `ph2d-tokens` só serve o chrome do editor |
| Aliases / math / modos autoráveis / DTCG | `ph2d-tokens` tem 4 temas **fixos no enum `Theme`**, sem alias e sem import/export |
| Canais animáveis de UI | `PropKind` tem **9** variantes (`TranslationX/Y`, `Rotation`, `ScaleX/Y`, `Opacity`, `TimeRemap`, `Morph`, `Position`) — **nada de tamanho, raio, fill, stroke ou token** |
| Máquina de estados de UI | nenhuma (a de física é outra coisa; o nodegraph é CONGELADO, §6) |
| Booleana viva | `apply()` consome os operandos; não há componente nem produtor |
| Runtime que toca UI sem editor | `ph2d-runtime` não existe (Front 2 não construída) |

### 1.3 Os seis achados que mudaram o plano

1. ⚠️ **`ph2d-tokens` já é metade do §4 do Vol. 2, e está mais adiantado do que o estudo supõe.** 350
   folhas, **4 modos** (`Forge`/`Workshop`/`Sunstone`/`Blueprint`), **OKLCH na fonte**,
   `ColorToken::resolve(theme)` como porta de resolução, um **`build.rs`** que gera os consts a
   partir do JSON, e `tests/design_token_sync.rs`, que **re-parseia o JSON com serde_json — um
   parser independente** — e afirma que a API pública concorda com ele. Isto é exatamente a
   disciplina que o Vol. 2 §4 pede, já shipada. **A wave dos tokens deixa de ser "construa o sistema
   de tokens" e passa a ser "faça o DOCUMENTO referenciar o que já existe, e faça a tabela ser
   AUTORÁVEL".**
2. ⚠️ **E o doc-comment do `ph2d-tokens` MENTE sobre isso.** `lib.rs` diz *"(manual sync — future
   codegen via build.rs)"* e *"Automatic codegen tokens.json → this crate (planned; sync is manual
   for now)"* — o `build.rs` **existe** e o gate documenta que ele existe. Comentário velho mente
   (CLAUDE.md), e este mandou-me planejar uma wave que já estava feita. **Corrigir no primeiro
   commit que tocar a crate.**
3. ✅ **Suprimir o desenho de um filho já é possível, sem uma linha nova.** `dispatch`
   (`ph2d-vec-render/src/lib.rs:174`) faz `if let Some(items) = live.get(&path.id)` e itera `items`
   — um `Vec` **vazio** no mapa desenha **nada**. É o mecanismo de que a booleana viva precisa para
   apagar os operandos sem os apagar do documento.
4. ✅ **A booleana cabe no frame, e o número é de hoje** (`cargo test -p ph2d-vec-boolean --test
   measure_aligned_stroke --release -- --ignored`, nesta worktree):

   | cena | traço centrado | banda 2w | **INNER** | **OUTER** |
   |---|---|---|---|---|
   | rosquinha (w=0.2) | 0.053 ms | 0.053 | **0.067** | 0.067 |
   | estrela 5 pontas (w=0.2) | 0.157 | 0.158 | **0.185** | 0.181 |
   | estrela 24 pontas (w=0.2) | 0.799 | 1.376 | **1.440** | 1.453 |

   Contra **16,6 ms** de um quadro a 60 fps. O caso patológico (24 pontas) é **8,7% do quadro**, e o
   caso real é **0,4%**. ⇒ *booleana não-destrutiva animada em tempo real* não é aposta: é
   aritmética. (O que MEDIR ainda: **N** booleanas animadas ao mesmo tempo — §9.)
5. ⚠️ **A identidade que o Vol. 2 §0 exige já existe aqui, mas o repo escolheu o OPOSTO em dois
   lugares.** A timeline (`wire_id`) e os joints de física casam por **`stable_name_id` = hash FNV-1a
   do `Name`** — matching por NOME, exatamente a fragilidade que o Vol. 2 §0 manda não herdar
   (*"renomear um corpo DESACOPLA os joints dele"*, CLAUDE.md). A escolha foi certa **lá** (uma
   entidade sem path não tem outro id durável). Para UI **não é**: toda peça de UI carrega um
   `VecPathRef` ⇒ tem `VecPathId`, que sobrevive ao clone do undo e ao postcard do save. **Todo
   endereçamento deste plano — override de instância, matching de Smart Animate, alvo de binding —
   usa `VecPathId`.** O precedente já está escrito em `vec_label.rs` e `vec_connector.rs`; este plano
   só o segue.
6. ⚠️ **A UI do editor é feita de WIDGETS, não de retângulos.** 44 widgets com estado, foco,
   teclado, a11y e cinco gates de costura cada. Isto é o §2 inteiro.

---

## §2 — A decisão que o estudo do Figma NÃO podia tomar: desenho ≠ widget

O Vol. 2 descreve uma ferramenta que produz **imagens de interfaces**. O Figma não tem esse problema
porque o produto dele *é* a imagem: o engenheiro do outro lado reimplementa tudo à mão. O Enio pediu
o contrário — *"UI que pode ser usada diretamente"*, e *"a versão final da UI eu mesmo desenharei"*,
onde "a UI" é a deste app, que já existe, funciona, e é feita de 44 widgets tipados.

Há três respostas possíveis e só uma sobrevive:

| Resposta | O que acontece |
|---|---|
| **(a) O desenho VIRA o widget** (o retângulo desenhado passa a ser um slider) | ⛔ Um slider tem *drag*, *focus ring*, *teclado*, *a11y*, *clamp*, *undo*. Desenhar isso é reimplementar o `ph2d-editor-core` no canvas. Duas respostas a *"o que é um slider?"*, e a do canvas nasce pior. |
| **(b) O widget VIRA desenho** (o editor passa a interpretar um documento vetorial) | ⛔ Troca 44 widgets testados por um interpretador; e o Enio quer **desenhar** a UI final, não trocar o motor dela. |
| **(c) O desenho é a PELE; o widget é o COMPORTAMENTO; o token é a ponte** | ✅ |

**A escolha é (c), e ela tem uma consequência que organiza o plano inteiro:**

> **O artista não desenha um slider. Ele desenha a APARÊNCIA de um slider — e essa aparência é um
> conjunto de tokens e uma geometria nomeada que o widget nativo consome ao pintar.**

Isso é possível **hoje, e barato**, porque o caminho já está construído e gateado:
`tokens.json → build.rs → consts → 44 widgets`. Editar um token re-veste **todo o app**, com um gate
de parser independente provando que o número que o widget usa é o número que o artista escreveu.

Daí a escada de três degraus da W6, e a razão de o degrau 1 valer sozinho:

1. **Re-vestir por TOKEN** (a tabela inteira, autorável no canvas com preview ao vivo). Zero código
   de pintura novo. Já entrega *"eu mesmo desenharei a UI"* para cor, espaçamento, raio, tipografia,
   densidade e movimento — **350 folhas**.
2. **Pele por-widget** (`VecWidget`): a geometria autorada de um botão vira os parâmetros de pintura
   daquele widget — cantos, sombra, gradiente, estados. O comportamento continua sendo o do
   `ph2d-editor-core`.
3. **Layout e composição** (W8/codegen): a árvore de frames autorada emite o `populate`/`paint`/
   `apply_event` de um `Panel` real, usando os widgets do catálogo.

E a UI dos **jogos** não tem esse problema: lá não há catálogo nativo a respeitar, o desenho **é** a
UI, e o consumidor é o runtime (W8a). ⇒ **um documento, dois consumidores**, e é por isso que o §4
não bifurca o modelo.

---

## §3 — A espinha: uma pergunta, uma porta

As portas únicas que este plano institui. Cada linha é uma pergunta que **não pode** ter duas
respostas no código; a coluna da direita é onde ela vive.

| Pergunta | Porta ÚNICA |
|---|---|
| *Que tamanho tem esta moldura?* | o `w`/`h` do `VecShape::Param{Rect}` da própria moldura — **`VecFrame` não guarda tamanho** |
| *Onde este objeto está, depois do layout?* | `LayoutPose` (mapa derivado por frame), **nunca** o `Transform` autorado |
| *Que geometria este caminho desenha?* | `LiveGeometry` (a booleana viva é o **7º produtor**) |
| *Que valor concreto tem este token, neste modo?* | `ph2d_tokens::resolve` (estendida), **nunca** uma 2ª tabela |
| *Qual é o valor desta propriedade?* | `bind::resolve(entity, prop)` — literal, ou token, se houver binding |
| *Este objeto é o mesmo objeto do outro estado?* | `VecPathId` (nunca nome, nunca bits de entidade) |
| *Que aparência tem este widget?* | o catálogo de widgets, alimentado pelos tokens (**um** caminho de pintura) |
| *Quanto tempo passou?* | o `Playhead` (relógio único, W4.T7) — a máquina de estados **não** tem relógio próprio |

⚠️ **A regra que o repo cobra e que este plano herda:** *todo canal novo é side-metadata no registry,
nunca contrato.* Nenhuma wave aqui apenda campo a `Paint`, a `StrokeSpec` ou a `NodeManifest` — o
binding é uma **tabela lateral**, exatamente como o `KernelResolver` ganhou 6 canais sem mover
`NodeOp`/`OpResolver`/`NodeManifest`.

---

## §4 — As waves

Cada wave declara: **o quê · por quê · a porta única · o modelo · schema/contrato · a UI (4
condições) · os gates red-first · a cena de smoke · o custo**.

As **4 condições de UI** (a política que a `line/physics` fixou e que vale aqui): o componente
**EXISTE** · é **pintado e registrado** · o **clique chega ao barramento** · e a **SEQUÊNCIA leva a
algum lugar**. As quatro são independentes; nenhuma implica a outra.

---

### W0 — A MOLDURA (Frame) ✅ **CONSTRUÍDA (2026-08-01)**

> Fechada por ordem do Enio (*"Próximo"*, depois do smoke da W1). O que shipou está abaixo; onde a
> construção corrigiu o plano, a correção está marcada ⚠️ no texto.

⚠️ **A construção derrubou TRÊS afirmações deste plano, e a terceira é a mais cara:**
>
> 1. *"O recorte delega ao `ClipChildren` que já existe."* **Falso.** `ClipChildren` é do pipeline
>    de **SPRITE** (o passe de stencil em `ph2d-render`) e não alcança um caminho vetorial — quem
>    desenha vetor é o Vello. O recorte é uma **camada de clip do Vello** (`push_clip`), a mesma
>    que os painéis roláveis já usam. Corolário: uma moldura recorta os descendentes **vetoriais**;
>    um sprite filho continua com o `ClipChildren` dele.
> 2. *"O preenchimento da moldura desenha onde ela está."* **Não pode.** O DFS lista o pai ANTES
>    dos filhos e a pilha de z é o **inverso** disso (`z_order`: `entries…rev()`), então **um pai
>    desenha na FRENTE dos filhos** — invisível para um grupo (sem geometria), fatal para uma
>    moldura, que cobriria o próprio conteúdo. O desenho dela é antecipado para a ABERTURA do
>    intervalo: é isso que "fundo do card" quer dizer, e é o que o Figma faz.
> 3. *"Os presets escrevem os pontos do aparelho (390×844)."* **A câmera não deixa.**
>    `Camera2d::ZOOM_MAX_HEIGHT_WORLD = 100.0` (e ela abre em `10.0`): um telefone de 844 seria
>    **8,4× mais alto do que a maior distância a que se pode afastar** — o artista veria 12% da
>    moldura e nunca a moldura. A tabela guarda os **pontos reais** (auditáveis, e é o que a
>    exportação vai querer) e uma porta única os converte para o documento com o lado maior em
>    **`LONG_SIDE = 8`** — medido contra a câmera, aspecto EXATO. É esta função que passa a dizer
>    "uma unidade vale N pixels" no dia em que a W8 decidir.

⚠️ **E o `is_screen` NÃO foi construído.** Nada a jusante o consumiria hoje, e um checkbox que não
muda nada é o controle morto que a política de UI deste repo existe para impedir. Ele nasce com a
exportação — e o custo do adiamento está nomeado: apender campo a componente **existente** bumpa o
schema, então a wave que o trouxer paga um bump.


**O quê.** O contêiner: uma tela, um card, um painel. Tem tamanho autorado, recorta o que
transborda, e é a raiz de tudo que é responsivo.

**Por quê primeiro.** Sem moldura não há *"redimensionar para quê?"*: layout, âncora, exportação,
preview de dispositivo e a própria noção de "uma tela" penduram-se nela. É a peça que o censo mostrou
**não existir** (`grep artboard` = 0).

**A porta única — e é a decisão inteira da wave:**

> **Uma moldura é um RETÂNGULO VIVO que ganhou um componente.** Não é um tipo novo de objeto.

```rust
// crates/ph2d-ecs/src/vec_frame.rs  (componente NOVO)
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecFrame {
    /// Recorta os descendentes à moldura. Delega ao `ClipChildren` que já existe.
    pub clip: bool,
    /// A moldura é uma RAIZ de exportação (uma "tela"), ou só um contêiner interno?
    pub is_screen: bool,
}
```

⚠️ **`VecFrame` NÃO tem `size`.** O tamanho é o `w`/`h` do `VecShape::Param{kind: Rect}` que a
entidade já carrega. Dois tamanhos divergem no primeiro arrasto de alça, e o modo de falha é o pior
que existe: o desenho concorda com um e o layout com o outro, e nada parece errado. Consequências
**de graça** por a moldura ser um retângulo: fill, gradiente, traço (com o alinhamento novo), raio de
canto vivo, a pilha de efeitos, o gizmo de escala, o hit-test, o z-order, o undo e o save.

**Schema/contrato.** +1 componente no registro (**41 → 42**). ⚠️ **Sem bump de `PROJECT_SCHEMA`** —
um componente novo cunha `stable_type_id = blake3(NOME)[..8]` e **não move nada**, o precedente
literal da W3 da física. `VEC_SCENE_SCHEMA_VERSION` **intocado** (o retângulo já existe no catálogo).
Nenhum contrato do §6 é tocado.

**UI.** Ferramenta **Frame** no rail do Vector (o 14º modo) + seção "Frame" no painel: `W`/`H`
(chips numéricos, já existem), **Clip** (checkbox), **Screen** (checkbox), e um **dropdown de
presets de dispositivo** (⚠️ **MEDIR não se aplica; os presets são DADO** — iPad, desktop 1080p,
etc. — e nascem de uma tabela, não de literais espalhados). As 4 condições valem para os 4
controles; o gate de seam **clica** cada um (`painted_rect` + `click_at`, nunca `WidgetEvent`
sintético — o sintético pula a checagem de focabilidade e passa sobre um chip que o `populate`
esqueceu).

**Gates (red-first).**
- `a_frame_is_a_rectangle_that_gained_a_component` — a moldura tem `VecPathRef` **e** `VecShape::Param{Rect}`; mutação: um `size` próprio em `VecFrame` ⇒ o gate do tamanho único sangra.
- `the_frames_size_has_exactly_one_answer` — redimensionar pela alça muda o número que o painel mostra E o que o layout lê.
- `a_clipping_frame_hides_what_overflows` — o filho fora da caixa não é desenhado (oráculo: o retângulo desenhado, não a flag).
- `an_unclipped_frame_hides_nothing` — o controle.

**Smoke** (`PH2D_BUILD_SMOKE=49`): uma moldura 390×844 (um telefone) com três filhos, um deles a
transbordar. Roteiro: ligar Clip (o transbordo some) · arrastar a alça (o número acompanha) · trocar
o preset (a moldura muda de tamanho e **o conteúdo NÃO se move** — sem layout ainda, e é isso que
torna a W2 visível).

**Custo.** Zero por frame (a moldura é um retângulo como qualquer outro).

---

### W1 — A BOOLEANA VIVA ✅ **CONSTRUÍDA (2026-08-01)**

> Fechada por ordem do Enio (*"W1 booleana viva"*). O que shipou está abaixo; onde a
> construção corrigiu o plano, a correção está marcada ⚠️ no texto.

**O quê.** Duas ou mais formas dentro de um grupo, e o grupo desenha `A − B` (ou ∪, ∩, XOR). Os
operandos continuam **editáveis, animáveis e vivos**; a booleana re-cozinha quando eles mudam.

**Por quê agora.** O Enio nomeou-a; é a menor wave do plano; e é **independente** de todas as
outras — não espera moldura, nem layout, nem token. Fecha um item aberto desde o cutover.

⚠️ **A construção corrigiu o modelo deste plano num ponto, e ele estava ERRADO:** o texto abaixo
dizia que *"o resultado entra no id do PAI (que é uma entidade-grupo, logo tem `VecPathRef`)"*. Um
grupo é, por definição do repo, *"uma entidade **sem geometria própria** … que tem filhos"*
(`vec_entities::ungroup_entities`) — ele nasce com `Transform`, `Name` e `RootOrder`, e **não tem
`VecPathId`**. Como a `LiveGeometry` é chaveada por `VecPathId`, o resultado passou a pousar no id da
**BASE** (o operando mais ao fundo), com os demais recebendo lista vazia — que é exatamente a regra
que a booleana destrutiva já segue, e o que faz o Apply não mover a arte.

**A porta única.** O **7º produtor de `LiveGeometry`**, irmão exato do `align_live` que acabou de
shipar — e, como ele, um produtor que **COMPÕE em vez de estender**:

```rust
// crates/ph2d-ecs/src/vec_bool_group.rs
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecBoolGroup { pub op: u8 }   // espelha ph2d_vec_boolean::BoolOp (u8, append-only)
```

```rust
// shells/desktop/src/bool_live.rs   — o produtor
// 1. a ENTRADA de cada operando é o que o mapa já diz (offset/pattern/… vivos), ou a fonte
//    assada em MUNDO — a mesma regra do align_live;
// 2. o resultado entra no id do PAI (que é uma entidade-grupo, logo tem VecPathRef);
// 3. cada operando recebe um Vec VAZIO ⇒ dispatch (lib.rs:174) desenha NADA por ele.
```

⚠️ **A ORDEM dos produtores é a lei da wave, e ela não é gosto:**
`5 extends (offset/pattern/contour/symmetry/profile)` → **booleana** → `alinhamento` →
`fx_silhouette`. Por quê: a booleana consome o que os filhos *de facto desenham* (um filho com
offset vivo tem de entrar deslocado), o alinhamento é um campo do `StrokeSpec` do **resultado**, e a
silhueta é *do que se desenha*, que já é isto. Trocar dois desses termos dá arte diferente sem
nenhum gate vermelho ⇒ **gate de ordem sobre o `render_loop`** (arch-gate, como o
`the_z_projection_reads_the_tree_after_the_sync`).

**Schema/contrato.** +1 componente (**41 → 42**), **sem bump de `PROJECT_SCHEMA`**. O código de
operação é `PathfinderOp as u8`, append-only. Nada do §6.

**UI.** Os botões booleanos **já existem** no painel (Union/Subtract/Intersect/Exclude + Pathfinder).
A wave acrescenta **um** controle: **"Live"** (checkbox ao lado deles) — marcado, o botão cria um
grupo com `VecBoolGroup` em vez de consumir os operandos. Mais, na seção do grupo selecionado, um
dropdown **Op** (trocar a operação sem refazer) e um botão **Apply** (assar: vira o que a booleana
destrutiva sempre deu). ⚠️ *Escolher "Live" não pode mudar o que o botão destrutivo faz* — os dois
verbos coexistem, e o gate de seam prova os dois caminhos.

**Gates (red-first).**
- `a_live_boolean_draws_one_shape_and_its_operands_draw_nothing` (mutação: não inserir o `Vec` vazio ⇒ os operandos reaparecem por cima).
- `changing_the_op_recooks_without_touching_the_document` (o `VecScene` sai byte-idêntico).
- `ungrouping_gives_the_operands_back_untouched`.
- `applying_a_live_boolean_equals_the_destructive_one` — **byte a byte**, é o gate que impede duas respostas à mesma pergunta.
- `the_producers_run_in_the_order_the_art_depends_on` (arch-gate; mutação: booleana depois do alinhamento ⇒ sangra).
- **Gate de RAZÃO** (nunca de relógio): re-cozinhar `N` booleanas custa `N ×` uma, e uma cena parada custa **zero** (o memo).

**Smoke** (`PH2D_BUILD_SMOKE=48`): três rigs — o par (quadrado + círculo), a rosquinha+barra (onde
`Subtract` e `Intersect` dão figuras claramente diferentes, para o re-mirar ser visível num clique) e
o rig do **CONTROLE**, com uma **linha ABERTA** por cima. A cena **nasce com o modo Live em OFF**, o
default do produto — quem liga é o artista. A pergunta é de olho: *depois da operação os operandos
continuam lá, e arrastar um deles muda o resultado enquanto a mão se move.*

**Custo (MEDIDO na porta do PRODUTO, release).** O `recook` INTEIRO — a caminhada da árvore, o
assamento em mundo, o motor e o mapa — com o memo **invalidado a cada volta** (o caso do arrasto, o
único em que o custo importa):

| operação | par simples | dez operandos |
|---|---|---|
| Union | **0,055 ms** | 1,530 ms |
| Subtract | **0,051** | 0,355 |
| Intersect | **0,051** | 0,229 |

Contra **16,6 ms** de quadro a 60 fps: o caso real é **0,3%** e o pior medido, 9%. ⇒ *animável em
tempo real* é aritmética, não aposta. (O `pathfinder` isolado já media 0,053–1,453 ms; o produtor
inteiro custa a mesma ordem, que é o que a medição confirma.)

---

### W2 — O AUTO LAYOUT

**O quê.** Uma moldura que **empilha** os filhos (direção, gap, padding, alinhamento, crescimento) e
os recompõe quando ela ou eles mudam de tamanho. O *Auto Layout* do Figma, o *flexbox* do Rive §7.

**A escolha de motor: `taffy`.** É o que o Vol. 2 §2 recomenda, é Rust puro, é o motor que Dioxus,
Zed, Bevy e Blitz usam, e implementa Flexbox + Grid + Block da spec CSS. **Escrever o nosso seria a
otimização prematura ao contrário:** gastar semanas para reproduzir uma spec que uma crate madura já
passa. ⚠️ Precisa de **ADR próprio** (é dep nova + escolha de motor foundational) — §7.

**A porta única — e o trap que ela evita:**

> **O resultado do layout é uma POSE DERIVADA, publicada por frame, e NUNCA o `Transform` autorado.**

⚠️ Isto não é preferência de estilo: o undo deste editor é **por DIFF do mundo ECS**. Um passe de
layout que escrevesse `Transform` faria **cada redimensionamento virar um passo de undo**, e faria o
layout brigar com o arrasto do artista dentro do mesmo frame. É exactamente a disciplina do ADR-0111
(a pose é publicada) e do `LiveGeometry` (a geometria é derivada), aplicada a um terceiro fato.

```rust
// crates/ph2d-ecs/src/vec_layout.rs
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLayout {
    pub dir: u8,          // 0 = row, 1 = column, 2 = wrap-row, 3 = grid
    pub gap: [f64; 2],
    pub pad: [f64; 4],    // t r b l
    pub align: u8,        // cross-axis
    pub justify: u8,      // main-axis
}
// e, no FILHO:
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLayoutItem { pub grow: f32, pub shrink: f32, pub basis: Option<f64> }
```

**A árvore do taffy é DERIVADA, memoizada na versão de estrutura** — nunca autorada. Uma segunda
árvore autorada é uma segunda resposta a *"quem é filho de quem"*, e a Hierarquia ECS já é a
resposta.

⚠️ **O texto tem de saber medir-se, e hoje não sabe.** `VecTextParams` não tem largura de quebra ⇒ um
bloco de texto não tem tamanho intrínseco e a *measure function* do taffy não tem o que devolver.
**Sub-wave W2a:** `wrap_width: Option<f64>` + a quebra de linha real (o **parley** já está na
árvore). ⚠️ **Apender campo a um componente EXISTENTE custa hoje um bump global** (postcard é
posicional dentro do blob) — ver §6.3: esta é a única wave do plano com essa dívida, e a F1.W1 da
`line/runtime` (*versão por `ComponentBlob`*) a apaga.

⚠️ **Arrastar um filho dentro de uma moldura com layout é REORDENAR, não mover** (é o que o Figma
faz, e é a única leitura coerente: a posição é derivada, então mover não tem onde pousar). Isto é
**gesto**, não desenho, e tem gate próprio.

**Schema/contrato.** +2 componentes (**43 → 45**) · +1 campo em `VecTextParams` (bump, §6.3) ·
`Cargo.toml` novo (`taffy`) · **ADR**. Nada do §6.

**UI.** Seção "Layout" na moldura: direção (segmentada de 4) · gap (2 chips) · padding (4 chips, com
o cadeado de "todos iguais") · alinhamento (2 segmentadas) · e, no filho selecionado, **Grow/Shrink**.
Preview: arrastar a alça da moldura recompõe **ao vivo**.

**Gates.** o passe **não escreve** `Transform` (arch-gate + mutação: escrever ⇒ o gate do undo sangra
com um passo espúrio) · uma linha com gap conhecido põe 3 filhos nas posições aritméticas exactas ·
`grow` reparte a sobra · o texto reporta o tamanho que ele **desenha** (oráculo = a bbox dos glifos,
não o que o layout achou) · arrastar reordena · uma moldura **sem** `VecLayout` fica **byte-intocada**
(é o que mantém todo documento existente idêntico).

**Smoke** (`=50`): uma barra de ferramentas com 5 botões e um espaçador `grow`, dentro de uma moldura
que o artista redimensiona; e um card cujo texto quebra e **empurra** o botão para baixo.

**Custo. MEDIR antes de escrever qualquer teto:** custo do passe com 10 / 100 / 1000 nós, e o custo
de reconstruir a árvore do taffy contra o de memoizá-la. O teto de profundidade/nós **sai da
medição**, não do gosto.

---

### W3 — AS ÂNCORAS (constraints sem solver) ✅ **CONSTRUÍDA (2026-08-03)**

> **O que MUDOU do que está escrito abaixo** (a §0 manda quem move um número reconferir a nota):
>
> - ⚠️ **`offset` NÃO existe no componente.** O plano previa o par da Unity (âncora + offsets em
>   píxeis); guardar os offsets tornaria a pose do filho função **só** do componente, e **arrastar
>   o filho deixaria de fazer alguma coisa** — o passe sobrescreveria o gesto a cada frame. O
>   offset é **derivado** por frame da caixa que o filho de facto tem. No lugar dele entrou
>   **`base`**: a caixa **LOCAL** da moldura quando a regra foi armada, que é o que torna
>   *"mudou de tamanho"* uma pergunta com resposta.
> - ⚠️ **A UI não é o widget de 9 pontos**, é **duas segmentadas de quatro** (Left/Center/Right/
>   Stretch e Top/Center/Bottom/Stretch) — o que o Figma de facto tem, e o superset do que o
>   modelo exprime. Sem chip *Off*: *"colado na aresta mínima"* já É a resposta neutra, e um Off
>   ao lado dela seria um segundo chip com o mesmo efeito.
> - ⚠️ **A ordem `âncoras depois do fluxo` é load-bearing pela MEDIÇÃO, não pela colocação:** para
>   um `translate ∘ scale` as duas ordens dão o mesmo ponto; o que discrimina é o tamanho **medido**
>   do nó (a âncora lê a caixa que o fluxo decide ⇒ ao contrário é um laço).
> - Smoke: **`=52`**, e ela **não arma regra nenhuma** — quem escolhe *Right* e *Stretch* é o
>   artista, que é a costura inteira da wave.


**O quê.** A regra de ancoragem no resize para os filhos que **não** estão num fluxo: *gruda à
direita*, *centraliza*, *estica com o pai*. O outro metade do Vol. 2 §2.

**A escolha: âncoras procedurais, NÃO Cassowary.** O próprio Vol. 2 diz que para 90% do HUD o
procedural basta e o solver vira over-engineering; e o caso relacional que justificaria o simplex
(*alinhe A com B*) **já tem resposta neste app** — align/distribute e o snap de reivindicação 2-D da
W6.1. ⇒ **Cassowary fica FORA**, com a cerca de Chesterton escrita (§5).

```rust
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecAnchors {          // o modelo da Unity: min/max normalizados + offsets em px
    pub min: [f64; 2],
    pub max: [f64; 2],
    pub offset: [f64; 4],        // l t r b
}
```

**A porta única.** O **mesmo passe** da W2 resolve as âncoras — um filho de moldura ou está num fluxo
(`VecLayout` no pai) ou está ancorado, nunca os dois. Um segundo passe teria uma segunda opinião
sobre a pose derivada.

**Schema.** +1 componente (**45 → 46**), sem bump. **UI:** o widget de 9 pontos + 2 eixos (o
*constraint picker* do Figma), que já existe em espírito no `rect2_editor`. **Gates:** esticar a
moldura move o filho ancorado à direita **exactamente** o delta; o centrado fica no centro; o
esticado ganha o delta na largura; sem `VecAnchors` **nada muda**.

**Smoke** (`=51`): um HUD com vida (topo-esquerda), pontuação (topo-direita, ancorada) e uma barra
que estica; redimensionar a moldura de telefone para desktop.

---

### W4a — OS TOKENS CHEGAM AO DOCUMENTO ✅ **CONSTRUÍDA (2026-08-02)**

> A metade **(a)** desta wave — *a referência no documento* — shipou. As metades **(b)** (a tabela
> autorável: modos como DADO, aliases, math, ciclo, DTCG) e **(c)** (`PropKind::Token`, o animável)
> **não**, e o porquê está no fim desta secção.

⚠️ **O censo do §1.3.1 estava certo e a construção o levou mais longe: `ColorToken::resolve(theme)`
já ERA a porta única**, com 80 folhas chaveadas por string kebab que casa com o JSON. A wave só
precisou da **inversa** — `ColorToken::from_key`, gerada pela MESMA macro que gera a `key()`, e é
isso que impede as duas de discordarem — mais `ColorToken::ALL`, que faz o picker ser DADO.

⚠️ **E o plano supunha construir o seletor de modo; ele JÁ EXISTE.** A tecla **`M`** cicla
Forge→Workshop→Sunstone→Blueprint em runtime (`input_handlers.rs`), com toast, e todo leitor de
tema lê `hero.theme` por frame. A frase *"trocar de modo `Forge → Sunstone` re-veste o card **e o
app inteiro**"* do smoke abaixo é literal, e não precisou de uma linha de UI nova.

**As três portas únicas:**

| Pergunta | Porta |
|---|---|
| *Que cor tem este token, neste modo?* | `ph2d_tokens::ColorToken::from_key` + `.resolve(theme)` — três consumidores (o resolvedor, o painel, os gates) |
| *Que tinta esta forma desenha?* | **`VecPath::painted(&bound) -> Cow`** — o irmão EXACTO do `cooked()` (ADR-0121), agora na TINTA |
| *Que propriedade segue um token?* | `ph2d_ecs::VecBindings` — tabela **LATERAL**, nunca um campo em `Paint`/`StrokeSpec` |

⚠️ **O `Cow` é a peça que torna a wave barata:** sem binding ele devolve `Borrowed` — o mesmo
ponteiro, zero cópia — e por isso a porta pôde ser ligada no ponto único de desenho sem mover um
byte de nenhum documento que já existe. É literalmente o que permitiu ao ADR-0121 ligar o cozido em
TODO consumidor de geometria.

⚠️ **A chave é o NOME do token, nunca o índice.** Guardar o índice do variant amarraria todo
projeto salvo à ORDEM da lista. Corolário que a wave seguinte colhe: quando a tabela virar
autorável, os tokens do ARTISTA entram **sem migração** — a chave já é o endereço.

⚠️ **Preenchimento e traço não se comportam igual, e o motivo é geometria:** um `Paint::Solid`
descreve um preenchimento por INTEIRO (bindá-lo numa forma sem preenchimento é autoria completa),
enquanto um traço precisa também de LARGURA — então o token do traço **pinta o traço que existe** e
nunca inventa um. É por isso que a row do traço só é oferecida quando há traço.

**Schema/contrato.** +1 componente no registro (**43 → 44**, e os dois espelhos `ph2d-render`/
`ph2d-script` **44 → 45** — ⚠️ o contador é TRÊS, a família que já ficou vermelho-latente duas vezes
nesta linha). **Sem bump de `PROJECT_SCHEMA`** (componente novo cunha `stable_type_id` próprio) ·
`VEC_SCENE_SCHEMA` **intocado** (é o ponto da tabela lateral) · contrato congelado intacto · nenhuma
dep nova · nenhum ADR.

**UI.** A row **Token** vive DENTRO das secções Fill e Stroke, ao lado da swatch — ⚠️ e não numa
secção à parte, porque é essa adjacência que responde à pior pergunta que a feature pode gerar
(*"por que a cor que eu escolhi não aparece?"*). O picker é um popover rolável de 81 linhas (as 80
da tabela + **Unbind**), pintado no passe DIFERIDO como o de mistura dos filtros.

**Gates (8 + 5 de seam + 2 arch).** ⚠️ **Uma mutação SOBREVIVEU e nomeou um buraco real:** trocar
`path.painted(bound)` de volta por `path` no laço do `dispatch` deixava os sete gates de unidade
**verdes** e todo binding inerte na tela — eles chamam a porta directamente. O gate que faltava é
`the_dispatch_draws_the_token_colour_not_the_literal`, e o oráculo dele não é *"os encodes
diferem"* (verdade em qualquer mudança) e sim a IDENTIDADE: desenhar a forma **bindada** ao token
tem de encodar exactamente como desenhar uma forma cujo LITERAL já é aquela cor. Ele nasceu
VERMELHO sobre a mutação. **7 mutações, 7 sangram.**

**Smoke** (**`PH2D_BUILD_SMOKE=50`** — ⚠️ o plano dizia `=52`; o número se **CONTA**, e 50 é o
próximo livre): um card (fundo + borda + texto), um **CONTROLE idêntico** ao lado que fica no
literal, e uma forma **SEM traço**. Bindar três propriedades e apertar **`M`**.

**Aberto, e é o resto da W4:**
- **(b) A tabela AUTORÁVEL** — modos como DADO (hoje são 4 variantes de `enum`), aliases
  (`{color.brand.500}`), math, detecção de ciclo, import/export DTCG. É uma reforma do
  `ph2d-tokens`, que 44 widgets consomem por consts gerados: wave própria, com o gate
  `design_token_sync` como rede.
- **(c) O ANIMÁVEL** (`PropKind::Token`) — cruza a fronteira para a timeline (`DOC_VERSION` +1) e
  quer a (b) primeiro: animar um token anônimo pressupõe que um token possa ser criado.
- **Tokens de ESCALA** (`CornerRadius`, `StrokeWidth`, `LayoutGap`): o `BoundProp` é append-only e
  os espera, mas cada um precisa do canal que o resolve — acrescentá-los agora seria oferecer um
  alvo que nada preenche.
- **Multi-seleção**: as rows só aparecem com UMA forma selecionada. Bindar várias de uma vez
  precisa de resposta a *"e se elas discordarem?"*, que é decisão de produto.

---

### W4 (original) — OS TOKENS CHEGAM AO DOCUMENTO

**O quê.** Uma propriedade qualquer (cor, raio, espessura, tamanho, gap) deixa de ser um literal e
passa a **referenciar um token**, resolvido por **modo**. É a feature de maior alavancagem do Vol. 2
(§4), e aqui ela tem **um segundo consumidor que nenhuma outra ferramenta tem: o próprio editor**.

**O que MUDA em relação ao estudo (achado 1.3.1).** `ph2d-tokens` já entrega tabela, 4 modos, OKLCH,
resolução, codegen e gate de sync. **A wave não constrói o sistema de tokens; ela constrói três
coisas que faltam:**

**(a) A REFERÊNCIA no documento — e ela é uma tabela LATERAL.**

```rust
// crates/ph2d-ecs/src/vec_bindings.rs
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecBindings { pub entries: Vec<(PropPath, TokenRef)> }
// PropPath = um endereço estável dentro do objeto: Fill, StrokeColor, StrokeWidth,
//            CornerRadius, ShapeValue(i), LayoutGap(axis), …  (u16, append-only)
```

⚠️ **Nenhum campo é apendado a `Paint`, a `StrokeSpec` ou a `VecShape`.** Se o binding morasse dentro
do `Paint`, **todo** save de vetor mudaria de forma e `VEC_SCENE_SCHEMA` bumparia por uma feature que
90% dos documentos não usa. A tabela lateral é a lei que o repo já aplica em quatro lugares (*"todo
canal novo é side-metadata no registry, nunca contrato"*). Custo: uma resolução por propriedade
bindada, por frame — **MEDIR**, e o caso comum (nenhum binding) é um `Option` vazio.

**(b) A TABELA passa a ser AUTORÁVEL e a ganhar alias/math/DTCG.** Hoje ela é um JSON editado à mão e
4 modos fixos num `enum`. Precisa de: modos **de dados** (não variantes de enum), **aliases**
(`{color.brand.500}`), **math** (`{spacing.md} * 2`), **detecção de ciclo** (obrigatória — o Vol. 2 §4
é explícito), e **import/export DTCG** (W9). ⚠️ **A tabela achatada por modo continua sendo a forma
de runtime** (Vol. 2 §4, alternativo): o grafo vive no editor; o jogo carrega a tabela plana.
⚠️ **Compatibilidade dura:** os 4 temas e as 350 folhas de hoje têm de continuar a resolver
**byte-idênticos** — o gate `design_token_sync` é o que prova isso, e ele já existe.

**(c) O ANIMÁVEL.** ⚠️ Aqui está a decisão que evita dezenas de variantes novas em `PropKind`:

> **Não se anima uma propriedade; anima-se um TOKEN. Uma propriedade animada é uma propriedade
> bindada a um token (possivelmente anônimo, criado pelo gesto de keyframar).**

`PropKind` ganha **UMA** variante apendada (`Token = 9`, depois do `Position = 8` que a `line/anim`
levou) em vez de uma por propriedade (tamanho ×2,
raio ×4, fill ×4 canais, stroke, gap ×2, …). E o que se ganha de graça é o **data binding do Rive**
(Vol. 3 §4): o mesmo token que a timeline anima pode ser dirigido por **estado de jogo** — vida,
score, tempo — e a arte reage sozinha. ⚠️ Bump de `DOC_VERSION` (variante apendada; a política de
quebra dura da timeline).

**Schema/contrato.** +1 componente (**46 → 47**) · `DOC_VERSION` +1 · a tabela de tokens vira uma
**seção** do arquivo (§6.3) · **`VEC_SCENE_SCHEMA` intocado** (é o ponto da tabela lateral). Nada do
§6.

**UI.** Painel **Tokens** novo (categoria MUNDO, como o de física): árvore de tokens, editor de valor
por modo, seletor de **modo ativo** (que re-veste o app inteiro ao vivo), e o **conta-gotas de
token** — selecionar uma forma, clicar no alvo de uma propriedade e escolher o token. ⚠️ Uma
propriedade bindada tem de **dizer** que está bindada (o número fica cinza com o nome do token ao
lado); um valor que não obedece ao que se digita e não explica por quê é a pior UI possível.

**Gates.** os 350 tokens resolvem hoje e continuam a resolver depois (o gate existente, sem tocar) ·
um alias resolve · um **ciclo** é recusado com erro nomeado (nunca stack overflow) · trocar de modo
muda a cor resolvida e **não** toca o documento · o `PropKind::Token` anima e a propriedade bindada
segue · **sem binding, o documento é byte-intocado**.

**Smoke** (`=52`): um card com fundo, borda e texto, os três bindados a `color.surface`,
`color.border`, `color.text`; trocar de modo `Forge → Sunstone` re-veste o card **e o app inteiro**;
keyframar `color.accent` e ver a UI toda pulsar.

---

### W5 — OS COMPONENTES (o prefab) ✅ **W5a + W5b CONSTRUÍDAS (2026-08-03/04)**

> **W5b — AS DIFERENÇAS (2026-08-04).** A W5a shipou `OverrideSlot::Fill`/`Hidden` com gates e
> **sem porta que os produzisse**: o único escritor era um teste, e o *Reset Overrides* era um
> botão que nunca podia ser preciso ([[feedback_a_capability_without_a_door_passes_every_gate]] —
> e nenhum gate a via, porque cada metade estava certa sozinha).
>
> - **A porta é uma LISTA de PEÇAS** na seção Component: uma linha por peça do mestre, com um
>   interruptor de visibilidade (cujo rótulo é o `Name` da Hierarquia) e uma swatch de cor.
>   ⚠️ A lista é a **sub-árvore INTEIRA** do mestre, nunca as peças visíveis — se fosse o
>   `visible_pieces`, esconder uma peça tirar-lhe-ia a própria linha e o gesto seria de mão única.
> - ⚠️ **A cor mora numa linha PRÓPRIA** porque a swatch é alvo de PICKER e o interruptor é um
>   botão: **um id só pode ter um tipo de widget no store** (a cicatriz do `vector_fx_toggle_id`).
> - **Update Main** absorve o que o mestre SABE guardar — a cor. ⚠️ O `Hidden` **não sobe**: um
>   mestre não tem *"peça escondida"*, e a única forma de a esconder lá seria apagá-la. A recusa é
>   por ESPÉCIE dentro da porta, e é **contada e dita** (recusar no botão levaria junto as cores
>   que ele pode absorver).
> - **Swap Main** é o conta-gotas — uma variante nova do `PathPick`, e não um modal próprio: arma,
>   o clique resolve, o Escape desiste, o hover realça. ⚠️ **A regra de compatibilidade é a única
>   honesta que este endereço permite:** um override aponta o `VecPathId` de uma peça do mestre
>   ANTIGO, e esses ids não existem no novo — os que não resolvem caem, e o terminal diz quantos.
>   Casar por NOME seria o endereço que a W5a recusou.
> - **`MAX_INSTANCE_PIECES = 16`** é teto de **TABELA DE IDS** (o `populate` regista o intervalo, o
>   roteador varre o mesmo), **não** do mestre: as peças além dele continuam a desenhar e a herdar,
>   e o excedente é **escrito no painel** em vez de truncado em silêncio.
> - **Zero schema, zero ADR, zero dep** — `VecInstance` já tinha tudo. Smoke **`=56`**.

> **O que MUDOU do que está escrito abaixo** (a §0 manda quem move um número reconferir a nota):
>
> - ⚠️ **O plano carregava DOIS endereços para a mesma pergunta** — `VecComponentMain { key: String }`
>   no mestre **e** `VecInstance { main: VecPathId }` na instância. Duas respostas a *"qual mestre é
>   este?"* divergem no dia em que uma ganha um caso, então uma morreu: **o id vence** (o precedente
>   literal do `VecPatternPath::path` / `VecTextPath::path`), e o mestre ficou **marcador puro** — o
>   nome de exibição é o `Name` que a Hierarquia já mostra.
> - ⚠️ **`PropPath` NÃO EXISTE** (a W4b não foi construída), então o override tem vocabulário
>   FECHADO: `Fill([u8;4])` e `Hidden`, endereçados pelo id da peça no mestre. Inventar uma
>   enumeração geral sem o consumidor que a exige seria desenhar no escuro; quando a W4b chegar ela
>   estende **esta** lista ou a absorve — o que não pode é nascerem duas.
> - ⚠️ **A ordem dos overrides é LEI, não arrumação:** o `canonicalize` do undo compara os BYTES do
>   componente, então duas instâncias logicamente iguais com a lista em ordens diferentes
>   comparariam DIFERENTE — e cada frame viraria um passo de undo espúrio. `VecInstance::set` é a
>   porta única que a mantém.
> - ⚠️ **Uma instância ESCALA a pose, mesmo dentro de uma moldura** (isenção no `resize_box_default`):
>   a caixa guardada dela é um **suporte** e o que se vê é derivado, então reescrever o retângulo
>   mudaria um número que ninguém olha e deixaria o desenho onde estava.
> - **Registro 48 → 50** (o plano previa 47 → 49: o `VecResizeBox` da W3b não estava contado), e os
>   dois espelhos (`ph2d-render`/`ph2d-script`) **49 → 51**. Sem bump de `PROJECT_SCHEMA`.
> - ~~**NÃO construídos, nomeados:** *Update Main* e *Swap*~~ — **os dois LANDARAM na W5b**
>   (2026-08-04, ver o bloco no topo desta seção). ⚠️ E a premissa de então estava errada num
>   ponto: *"o Update Main pede editar uma instância no lugar"* — não pede. As edições de instância
>   **são** os overrides, e absorver é lê-los e escrevê-los no mestre. O que faltava não era um
>   modo de edição, era a **porta que produz um override**. Segue de fora só os **variants**
>   (`VecInstance.props`), que são a **W5c** e custam um bump (apender campo ao componente é
>   postcard posicional).
> - Smoke: **`=53`**, e ela **não cria componente nenhum** — quem promove, coloca e destaca é o
>   artista, que é a costura inteira da wave.


**O quê.** Mestre + instâncias com **overrides esparsos**; **variants** (size/state/color) sob
properties. O Vol. 2 §3 — e, como o próprio estudo diz, **isto é o sistema de prefabs do engine**.

**A porta única.** Herança prototipal com override esparso endereçado por **`VecPathId` do
sub-objeto do MESTRE** (achado 1.3.5). A alternativa (cópia + 3-way merge) é `O(N × tamanho)` em
memória e perde a propagação automática — a tabela do Vol. 2 §3 já a precifica, e para um tileset com
milhares de instâncias ela é insustentável.

```rust
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecComponentMain { pub key: String }          // o mestre

#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecInstance {
    pub main: VecPathId,                                  // o mestre, por id do documento
    pub overrides: Vec<(VecPathId, PropPath, Value)>,     // ESPARSO, endereçado no mestre
    pub props: Vec<(u16, Value)>,                         // variant / boolean / swap escolhidos
}
```

⚠️ **`PropPath` é o MESMO tipo da W4.** O endereço de *"que propriedade deste sub-objeto"* é uma
pergunta só, e duas enumerações dela divergiriam no dia em que uma ganhasse um caso.

⚠️ **A instância é DESENHO derivado**, não uma cópia no documento: ela entra pelo `LiveGeometry`
(8º produtor) resolvendo mestre + overrides. É o que faz *"editar o mestre propaga"* ser verdade por
construção, em vez de um passe de propagação que alguém esquece de chamar.

**Schema.** +2 componentes (**47 → 49**), sem bump de projeto.

**UI.** *Create Component* · *Detach* · *Reset Overrides* · *Update Main* · o painel de **properties**
da instância · e o **swap** (trocar a instância mantendo overrides compatíveis).

**Gates.** editar o mestre muda **todas** as instâncias · um override sobrevive à edição do mestre ·
*Reset* volta ao mestre · **memória**: 1000 instâncias custam `O(overrides)` e não `O(1000 × árvore)`
(gate de razão, com dhat como no ADR-0117) · uma instância cujo mestre sumiu **não desaparece em
silêncio** (mostra-se destacada, como a binding *missing* da timeline).

**Smoke** (`=53`): uma paleta de 3 variants de botão, 12 instâncias numa moldura com layout;
editar o mestre re-veste as 12; um override de texto sobrevive.

---

### W5c — OS VARIANTS ✅ **CONSTRUÍDA (2026-08-04)** — e o BUMP que ela ia custar não existe

> A W5b fechou nomeando o que faltava: *"segue de fora só os **variants** (`VecInstance.props`),
> que são a **W5c** e custam um bump (apender campo ao componente é postcard posicional)"*.
> ⚠️ **Custavam um bump no desenho que o plano tinha; o desenho mudou e o preço sumiu** —
> `PROJECT_SCHEMA` **intocado**, registro do `ph2d-ecs` **intocado**, **nenhum ADR**, zero dep.

**A espinha, e é uma frase:** ⚠️ **um conjunto de variants é DERIVADO — são os mestres IRMÃOS**,
os caminhos marcados `VecComponentMain` que pendem do mesmo pai. É literalmente o que um *component
set* do Figma é (uma moldura com componentes dentro), e é o que o artista já sabe autorar com os
gestos que existem: desenhar as versões, *Create Component* em cada uma, agrupá-las. Um marcador
`VecVariantSet` no pai seria uma **segunda** resposta a *"isto é um conjunto?"*, e divergiria no dia
em que alguém agrupasse dois mestres sem o pôr.

⚠️ **Corolário que decide a ausência da row:** um mestre-RAIZ não tem irmãos ⇒ nenhuma fileira. Sem
exigir o pai, *"os irmãos"* seria *todos os mestres do documento* — e cada componente do projeto
viraria variant de todos os outros. **E um variant é um IRMÃO, nunca um FILHO:** um componente
aninhado é uma PEÇA, não uma versão (gate próprio, e a fixture dele precisou de um mestre-raiz **com
dois mestres dentro** — a primeira era verde por vácuo, porque uma folha solta não tem `Children` e
as duas leis davam a mesma lista vazia).

**A porta de escrita já existia.** `VecInstance.main` é o mestre de que a cópia deriva, então
escolher outro variant é **religá-la a um irmão** — ou seja, é o *Swap Main* da W5b, restrito aos
irmãos e oferecido como chips em vez de conta-gotas. Zero schema, e o **descarte dos overrides que o
mestre novo não conhece** (a regra que a W5b mediu) vem junto por não haver segunda escrita
(arch-gate: o braço não pode conter `.main =`).

**Os EIXOS saem do NOME** — `Size=Small, State=Idle`, a convenção do Figma. Quando **todos** os
irmãos declaram as MESMAS propriedades, a seção mostra uma fileira por propriedade; senão, **uma**
fileira `Variant` com os nomes crus. ⚠️ A tabela de propriedades **não é guardada em lado nenhum**:
ela é lida dos nomes, que já viajam no `Name` da Hierarquia e que o artista já renomeia ali.
⚠️ E a exigência é de **IGUALDADE de conjunto de chaves, não interseção** — com `{Size}` num irmão e
`{Size, State}` noutro, uma interseção esconderia um eixo sem nada a dizer porquê.

⚠️ **Um valor que não leva a lado nenhum não é OFERECIDO.** Numa matriz com buracos
(`Size=Large, State=Hover` que ninguém desenhou) o chip existiria, seria clicável e não faria nada —
o botão-morto que este repo persegue. A lista de cada eixo é *o que se alcança a partir da combinação
vigente*, e por isso é sempre honesta.

**UI.** Uma fileira **segmentada** por eixo (chips e não dropdown: um eixo tem tipicamente dois a
quatro valores, e mostrá-los todos deixa o artista *ver* o catálogo em vez de o abrir; a fileira
quebra em linhas sozinha). Ela vem **ANTES** da lista de peças — *que versão é esta?* precede *e o
que nela difere?*, e a ordem é funcional: trocar o variant reescreve a lista de peças.
`MAX_VARIANT_AXES = 4` × `MAX_VARIANT_VALUES = 8` é **teto de TABELA DE IDS** (o `populate` regista
o intervalo, o roteador varre o mesmo) e o excedente é **escrito**, nunca truncado em silêncio.

**Gates:** 11 no kernel + 4 de seam + 3 arch + 6 de fixture da cena. **11 mutações, 10 sangram**
(a 11ª é **inválida**: `it.next()?` e o guard `v.is_empty()` são a mesma defesa, então trocar um pelo
outro é no-op semântico). ⚠️ **Duas sobreviveram e as duas eram FIXTURE minha:** a lei do pai (acima)
e o chip aceso — medido só do PRIMEIRO variant, `selected = 0` está certo por acidente em todo eixo.
⚠️ **E uma sobrevivente nomeou a quarta condição da política de UI:** com o roteador cego ao id do
chip, ele é pintado, vive sob o mouse, o Click atravessa o barramento **e não vira nada** — os doze
gates de projeção e os quatro de seam ficavam todos verdes (`a_variant_chip_id_becomes_a_verb`).

**Smoke** (**`=58`**): uma moldura com as quatro versões (`Size` muda a GEOMETRIA, `State` muda a
COR — cada eixo com uma consequência que o olho separa) + um mestre **SOLITÁRIO** (a metade da
ausência) + um controle. Medido pela sonda `measure_the_smoke_rows`: de `Small/Idle` o painel mostra
`Size: [Small] Large | State: [Idle] Hover`, as quatro combinações são alcançáveis **em um clique
cada**, e o solitário dá **0 fileiras**.

**Aberto, e é o resto dos *properties* do Figma:** o **boolean property** já tem porta (a lista de
PEÇAS da W5b esconde qualquer peça — construir um checkbox nomeado seria a segunda resposta a
*"esconder uma peça"*) · o **instance-swap property** exige nesting (uma instância DENTRO de um
mestre), que segue nomeado em aberto · e o **text property** não tem consumidor (o override tem
vocabulário fechado: `Fill` e `Hidden`).

---

### Z-ORDER — a lei das GAME ENGINES: o filho desenha SOBRE o pai, e o Z é GLOBAL ✅ **(2026-08-04)**

> Enio, em duas rodadas: *"o filho é desenhado por trás do pai … em game engines como Godot os
> filhos são renderizados acima dos pais"* · *"na hierarquia os objetos mais abaixo aparecem atrás,
> como nas camadas de um app de imagem. Mas veja as game engines! Em Godot é ao contrário"* · *"ao
> criar a instância, os filhos que no mestre aparecem na frente dos pais vão para trás dos pais"* ·
> *"quando se cria um objeto novo ele vai para o último abaixo na hierarquia, mas aqui ele aparece
> no topo"* · *"a ordem só conta se o Z dos objetos for igual. O Z index deve ser global e sobrepõe
> a ordem na hierarquia"*.
>
> **As cinco queixas são UMA, e a cura é uma linha na projeção.**
>
> - **A LEI: a ordem do DFS *É* a ordem de desenho.** `vec_zorder::z_order` deixou de inverter. O
>   pai passa a ser o PRIMEIRO membro da própria sub-árvore ⇒ o filho desenha sobre ele **em toda
>   rota**, mais abaixo na Hierarquia = mais à frente, e o objeto novo entra no FIM da lista (que é
>   a frente) — o `sync` apenda em vez de abrir espaço no começo.
> - ⚠️ **A versão anterior impunha a lei por DOIS remendos**, e o terceiro consumidor não os
>   conhecia: o renderer **antecipava** o desenho de todo pai, o apontar **demovia** cada
>   antecipado, e a **INSTÂNCIA** — que percorre a cena e não tem renderer por trás — cobria os
>   próprios filhos. Invertida a projeção, os dois remendos MORRERAM (`demote_hoisted_frames` e a
>   antecipação) e a instância ficou certa **sem uma linha de código dela**.
> - **O Z-INDEX é o `ph2d_ecs::ZIndexOverride`**, o MESMO componente dos sprites — um componente
>   próprio seria a segunda resposta a *"quem está na frente?"*, e como um caminho pode ser filho de
>   um sprite (ADR-0110) as duas conviveriam na MESMA árvore. `effective_z_index` virou a porta
>   pública dessa pergunta. **Zero schema** (o componente já é registado e serializado).
> - **A pilha tem DUAS chaves:** o Z efetivo (cascata de Godot) ordena, o DFS **desempata** — o
>   `sort_by_key` é estável, então um documento sem um único `ZIndexOverride` produz a MESMA lista
>   que o DFS cru. Campo **Z** no topo da Arrange, `0` **destaca** o componente.
> - ⚠️ **O RECORTE mudou de forma junto.** `VecClipSpan { frame, last }` volta a ser só sobre
>   recortar (o `clip: bool` sumiu com a antecipação), e o intervalo vai até onde a sub-árvore é
>   **CONTÍGUA** na pilha final. Duas propriedades caem daí: um estranho que o Z pousou no meio
>   **nunca** é recortado por um card que não é dele, e a lista fica **laminar** de graça. O preço é
>   o que a frase promete — *o filho que o Z tirou da corrida deixa de ser recortado*.
> - **Renomes de honestidade:** `VecParentSpan` → `VecClipSpan` (campo `parent` → `frame`, `first` →
>   `last`), `VecViewState.parent_spans` → `clips`, `parent_spans` → `clip_spans`. O tipo voltou a
>   nomear o que faz.
> - **17 gates, 12 mutações, 12 sangram. Zero schema, zero ADR, zero dep.** Smoke **`=57`**.

#### A terceira rodada: **o Z é a ÚNICA porta, e o campo estava MUDO** ✅ **(2026-08-04)**

> Enio: *"Z-index não funcionou e os botões de Arrange não foram convertidos para funcionar com
> Z-index. A partir de agora o objeto não deve ser movido na hierarquia, apenas o Z muda e o Z
> determina na frente de quem ele é mostrado. A ordem na hierarquia só define a ordem se o Z for
> igual."*
>
> **(A) O campo estava PINTADO, REGISTADO, VIVO SOB O MOUSE — e MUDO.** O `ValueChanged` do commit
> caía no catch-all do `apply_event` do painel e **nunca virava `SetValue`**, então a shell nunca
> escrevia o `ZIndexOverride`: o artista clicava, ganhava foco, digitava, via o número mudar, e
> nada acontecia. ⚠️ **O gate que eu tinha media o CLIQUE (o foco), não o COMMIT** — é a metade
> errada da mesma costura, e é por isso que a wave anterior fechou verde sobre um campo morto. O
> gate novo (`the_typed_z_reaches_the_bus`) afirma o **valor** que chega ao barramento, porque
> encaminhar o id com o número de outro campo é o mesmo defeito com outra roupa.
>
> **(B) Os quatro botões escrevem o Z e MAIS NADA.** A metade de árvore
> (`siblings`/`sibling_move`/`write_sibling_order`) **morreu**: ela dava aos botões duas maneiras
> de mudar a mesma pilha, e a que o artista não pediu era a destrutiva — pressionar *Forward*
> re-arrumava a Hierarquia dele por baixo. O oráculo desta wave é **a lista da Hierarquia**, que
> tem de sair de todo gesto **intacta**.
>
> - **Os quatro verbos são UMA regra com uma REFERÊNCIA diferente** (Forward → o vizinho da frente
>   · To Front → o da frente de tudo · Backward → o vizinho de trás · To Back → o do fundo): *"que
>   número me põe do outro lado daquela forma?"*. Um `match` com aritmética por verbo teria quatro
>   sítios onde o sinal pode estar trocado.
> - **O passo é o MENOR que entrega** — empatar já basta quando a árvore me favorece (o desempate é
>   dela). É isso que impede o Z de inflar um número por clique e o que faz do *Backward* o inverso
>   **exacto** do *Forward*.
> - ⚠️ **Um DESCENDENTE é incruzável, e o botão RECUSA.** O Z é cascata: subir o meu número sobe o
>   dos meus filhos pelo mesmo tanto, e a distância não muda. A primeira versão mirava no vizinho
>   sem perguntar de quem ele era, e o *To Front* de um pai escrevia `z = 1`, devolvia `true` e
>   **não movia um pixel** — um Ctrl+Z gasto por nada. ⚠️ E a assimetria é real: o **filho PODE** ir
>   para trás do pai (Z próprio negativo); a cascata só corre num sentido.
> - ⚠️ **O limite que é ARITMÉTICA, não desleixo, e fica MEDIDO:** com três formas empatadas em
>   `z = 0` não existe inteiro que ponha a do fundo *entre* as outras duas — `0` deixa-a atrás das
>   duas e `1` põe-na à frente das duas, então o *Forward* passa **dois** lugares. As duas saídas
>   foram recusadas pela própria lei (renumerar os vizinhos é escrever no número de um objeto que o
>   artista não selecionou; mexer na árvore é o que ele proibiu).
> - ⚠️ **DOIS gates meus reprovaram código CORRETO** antes de eu ler a cascata, e as duas correções
>   ficaram como gates: o pai que não cruza o filho, e o filho que cruza o pai. **E uma fixture
>   nasceu cega** — com tudo empatado em 0, *passar o vizinho* e *ir para a frente* dão o mesmo
>   número, então a mutação "To Front vira um passo só" ficava VERDE no gate que **afirma** que
>   cada verbo entrega, e só sangrava num gate sobre o readout.
> - **9 mutações, 9 sangram. Zero schema, zero ADR, zero dep, zero id novo.** Smoke **`=57`**,
>   passos 5 e 6.

---

### W6.1 — A TABELA DE COR VIRA AUTORÁVEL ✅ **CONSTRUÍDA (2026-08-04)** — o degrau 1 do §2

> O §2 decide que *o desenho é a PELE, o widget é o COMPORTAMENTO, e o token é a PONTE*. Este é o
> primeiro degrau, e o entregável dele é literal: **o artista muda a cara do app inteiro, ao vivo,
> e a mudança sobrevive ao arquivo.**

**A porta já existia, e é essa a razão de a wave ser barata.** `ColorToken::resolve(theme)` é o
funil por onde os **44 widgets** passam — a camada de override entra ali e mais nada precisa mudar.
⚠️ **Vazio ⇒ BYTE-IDÊNTICO** (gate que varre a tabela inteira nos quatro modos), e o
`design_token_sync` continua a medir a tabela **gerada**, porque é ela que o override cobre e nunca
substitui.

⚠️ **É estado global mutável lido por uma função de aparência pura, e isto está escrito no
`overrides.rs`.** O padrão é o que o repo já usa entre shell e folha; `thread_local` **não é por
performance**: cada teste corre na própria thread, então um gate que arma um override não pode
envenenar o vizinho. A consequência honesta — *quem PINTA tem de ser quem PUBLICOU* — está nomeada
lá em vez de descoberta num screenshot.

**MEDIDO** (`measure_override_layer`, min de 3 varreduras): `resolve` custa **~52-55 ns** com a
camada vazia; **1,02×** com um override e **1,09×** com vinte. ⚠️ A 1ª versão da sonda media UMA
varredura e as razões saíam entre **0,97× e 1,45×** — uma delas *abaixo de 1*, impossível como custo
real. O número absoluto era estável e só as razões eram ruído: **o sinal é menor que o ruído da
própria medida**, e é isso que o resultado diz.

⚠️ **A FRONTEIRA é estrutural, não escolha:** só a COR é autorável. `Spacing::px()` / `Radius` /
`StrokeToken` / `Motion` são **`const fn`** lidos em **14 sítios `const`** do app; torná-los de
runtime quebraria o contexto const e removeria a garantia de compile-time. É o resto da W4b, e agora
tem o mecanismo escrito ao lado.

**O painel** (`ph2d-panel-tokens`, categoria MUNDO, tecla **`T`**): uma linha por token de
`ColorToken::ALL` — swatch (alvo de PICKER) + a chave + um *Reset* **só na linha autorada** + *Reset
This Mode* **só com o que resetar**. ⚠️ **Ele não guarda cópia de token nenhum** (a lista é a tabela,
o valor sai da porta, o modo vem do host): um snapshot por frame seria uma segunda cópia da tabela
de cor, num painel cuja razão de existir é ser a autoridade sobre ela. ⚠️ E **um escritor só** — o
painel emite intents, a shell escreve (ela já é obrigada a fazê-lo no read-back do picker).

⚠️ **A `T` era um scaffold de debug** (`"Toast key (T) pressed"`), órfão como o `KeyB` que a timeline
aposentou no W4.T5 — varrido no repo antes de ser tomado.

**Persistência:** `ProjectFile.tokens` (**`PROJECT_SCHEMA` 50→51**, campo apendado, postcard
posicional). ⚠️ **FORA do `ProjectState`**, o precedente das settings de física: um Ctrl+Z do canvas
não rebobina a cara do editor. O preço honesto — *editar um token não entra na fila do Ctrl+Z* —
tem o *Reset* como resposta. ⚠️ E viaja a **CHAVE**, nunca o índice do variant: guardar o índice
amarraria todo projeto salvo à ORDEM da lista.

**Gates: 8 (camada) + 7 (seam) + 4 (ponte) + 4 (persistência) + 4 (arch) — 12 mutações, 12
sangram.** ⚠️ **Duas sobreviveram à 1ª rodada e as duas viviam na PONTE, que não tinha gate**
(*Reset This Mode* podia apagar os quatro modos; o read-back podia escrever a cada frame). A cura
não foi um arch-gate: `HeroScreen::new` é **construtor puro** e o estado do picker é semeável, então
a ponte virou dirigível headless — *"isto exige janela"* era a resposta preguiçosa, e a janela não
era exigida.

⚠️ **Dois gates MEUS nasceram vermelhos sobre produto correto:** um media o ramo FECHADO chamando o
`painted_rect`, que **força o painel visível** (o helper certo é o `paint_hidden`, que a auditoria
de 2026-07-29 criou para este ramo); e o outro ancorava no literal `"Toast key (T) pressed"` — que o
doc-comment desta mesma wave **cita**. *Um oráculo que casa com a documentação de si mesmo não está
a olhar para o produto* (a lição do `[frame]` do Flip, §5.48).

**O abridor VISÍVEL** (pergunta do Enio, 2026-08-04 — *"onde fica o botão para abrir esse painel?"*):
o pill **TOK** no topbar, ao lado do PHYS. ⚠️ **É a mesma queixa que ele fez sobre o painel de
física em 2026-07-27**, e a resposta é a mesma: um painel de MUNDO não tem chip no rail (que é de
FERRAMENTAS), então sem pill ele é *uma feature que só existe para quem já sabe que ela existe*. O
pill **LÊ** a visibilidade que a tecla escreve — um bool próprio é como o botão passa a dizer
*fechado* sobre um painel aberto pelo atalho. ⚠️ E com dois pills irmãos nasce um modo de falha
novo: o handler saiu de um `cp` do de física, e um `if id != …` que sobrevivesse à cópia faria o TOK
abrir o painel VELHO — gate próprio (`each_pill_opens_only_its_own_panel`), e a mutação sangra em 3
dos 4.

**Smoke** (**`=59`**): a cena **não monta geometria de assunto** — o sujeito é o próprio editor. Ela
abre o painel, põe três formas de CONTROLE (a arte não é chrome) e **imprime quantos tokens o painel
vai listar**; se for zero, PARE.

**Aberto, e é o resto da W4b:** os **aliases** (`{color.brand.500}`) ⚠️ **não** foram construídos
como variante do modelo, e a razão é uma lei deste repo: um variante sem porta que o produza é o
`OverrideSlot` da W5a outra vez ([[feedback_a_capability_without_a_door_passes_every_gate]]) — eles
nascem com o seu gesto ou não nascem · math · detecção de ciclo (que os aliases exigem) · DTCG · os
tokens de ESCALA (a fronteira `const fn` acima) · e um readout de CONTRASTE (o `tokens.json` valida
WCAG 2.2 AA nos gates, e um valor autorado pode quebrá-lo sem nada dizer).

---

### W6 — O VÍNCULO COM O WIDGET (a metade funcional)

**O quê.** O degrau que transforma *"desenho de UI"* em *"UI"*. Ver §2 para o porquê da forma.

**Degrau 1 — re-vestir por token.** Não precisa de código novo de pintura: é a W4 aplicada à tabela
que o chrome já consome. **Entregável:** o artista muda a cara do app inteiro do canvas, com preview
ao vivo, e o gate de parser independente prova que o número que o widget usa é o que ele escreveu.

**Degrau 2 — a pele por-widget.** ✅ **CONSTRUÍDA (2026-08-04)** — cena `=60`.

> ⚠️ **Uma correção ao que este parágrafo dizia, medida na construção:** os pintores do catálogo
> **não aceitam** cantos/sombra/gradiente — a assinatura deles é `(dados, rect, scene, text,
> theme)` e toda cor e todo raio saem do `ph2d_tokens`. Dar-lhes um canal de estilo por-widget
> seriam 44 assinaturas novas **e** uma segunda porta para a aparência, no dia seguinte ao da W6.1
> ter feito a tabela de cor autorável. ⇒ **o desenho responde ONDE e O QUÊ; os tokens respondem
> COMO**, e o mapeamento por-tipo fica NÃO construído até um tipo precisar de algo que o token não
> exprime.
>
> ⚠️ E o componente tem **um campo só**: o `key` do esboço abaixo não existe — o rótulo é o `Name`
> da entidade, a mesma lei que a W5c usou para os eixos de variant.
>
> ⚠️ A lista de tipos é **doze**, e a fronteira é ESTRUTURAL: um widget cuja aparência é função de
> *(retângulo, rótulo, estado)* é vestível hoje; um cuja aparência é função de uma **LISTA** (Tabs,
> TreeView, RadioGroup, Dropdown) precisa de filhos autorados — e isso é o degrau 3.
>
> ⚠️ **O smoke APROVOU** (*"Muito bom!"*, 2026-08-04) **e expôs uma premissa que este parágrafo não
> tinha**: dois dos doze tipos **não preenchem a moldura** — o `Checkbox` ancora o quadrado num
> TOKEN (`CHECKBOX_BOX_PX.min(rect.h)`) e o `Slider` clampa a pista em 8 px. **A lei está certa
> dentro de um painel** (é ela que faz todo checkbox do app ter o mesmo tamanho); o que mudou é que
> **esta wave é o primeiro chamador que dá ao pintor uma moldura arbitrária** — até aqui quem a dava
> era o layout, no tamanho natural do widget. A bifurcação (escalar o fragmento × declarar o tamanho
> intrínseco × dar canal de tamanho aos 44 pintores) é decisão de PRODUTO e está em
> [`BUGS_vector.md` #26](../BUGS_vector.md) com o preço de cada saída. Seja qual for, a resposta é
> **por-tipo e mora numa porta só** — e ela responde uma pergunta nova: *a moldura de uma pele é uma
> **CAIXA** (o widget preenche) ou um **SLOT** (o widget assenta no tamanho natural)?*


```rust
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecWidget { pub kind: u16, pub key: String }  // kind = o catálogo dos 44
```

O desenho autorado vira os **parâmetros de pintura** daquele widget (raio, sombra, gradiente,
espessura de foco, cores por estado). ⚠️ **O comportamento continua no `ph2d-editor-core`** — o
canvas nunca implementa *drag*, *foco* ou *teclado*. E ⚠️ **o que o canvas mostra tem de ser o que o
widget pinta**: a prévia no canvas chama o **pintor real** do widget, senão temos duas aparências
para a mesma coisa e a divergência só aparece numa screenshot.

**Degrau 3 — o layout vira `Panel`.** É a W8b. 🔨 **A W8b.1 CONSTRUIU a primeira metade
(2026-08-05)** — cena `=62`.

> ✅ **A árvore autorada descreve um painel, e o app escreve o código dele.** `ui_panel_spec::of`
> responde *que painel esta moldura descreve* (porta única) e o `ph2d-ui-codegen` escreve a tabela
> de rows. **Só quem VESTE vira row** — um filho sem `VecWidget` continua a ser desenho, e um
> gerador que transformasse todo filho em linha daria um painel com uma row que não faz nada.
>
> ⚠️ **A contenção que decide a wave: o gerador NÃO depende do `ph2d-editor-core`.** Sem alcance ao
> catálogo ele **não CONSEGUE** ter opinião sobre o que um `Slider` é — o identificador chega
> pronto no `RowSpec`, resolvido por `WidgetKind::ident`, que nasce ao lado do `code` (onde a
> lista tem dono). Uma tabela `código → nome` do lado do gerador seria a segunda resposta a *"quais
> são os tipos vestíveis?"*, e ela driftaria **em silêncio** — um número desconhecido não falha,
> ele só não casa. **Arch-gate sobre o `Cargo.toml` E sobre o fonte** (uma tabela dessas não
> precisa da dep para existir).
>
> ⚠️ **O risco §10.4 (*"codegen que o CI recusa"*) está DESCARREGADO**: o golden é commitado **e
> COMPILADO**, então Rust inválido não chega ao `main` — o build do teste cai antes. Os dois gates
> sobre ele **não são redundantes**: um compara BYTES (staleness), o outro lê as consts *depois de
> o compilador as aceitar* (conteúdo).
>
> ⚠️ **O gerado é o que VARIA entre dois painéis, e nada mais.** Emitir o chrome — moldura, título,
> alças, gate de visibilidade — copiaria umas cem linhas **idênticas em todo painel** para dentro
> do gerado, e um dia em que ele mudasse, todo painel gerado ficaria para trás.
>
🔨 **E a W8b.2 CONSTRUIU a segunda metade (2026-08-05)** — mesma cena `=62`.

> ✅ **O código gerado é COMPILADO e vira um painel vivo.** Crate nova `ph2d-panel-authored`, os
> cinco sítios de fiação de painel, e um **runtime de rows** que percorre a tabela emitida —
> `paint` · `populate` · `event` · a varredura de seam, **uma lista, quatro consumidores**, a lei
> do `SECTIONS` do painel de física. É ela que torna **estrutural** (e não disciplinar) o
> requisito mais duro da W8b: uma row não pode ser pintada-e-não-registada, porque as duas metades
> leem a mesma lista.
>
> ⚠️ **NÃO há um segundo `match` sobre os doze tipos.** A `paint_widget_skin` do degrau 2 ganhou a
> irmã **`paint_widget_skin_with(kind, label, id, live, …)`** e a antiga passou a **delegar** com
> `live = None` — a prévia do canvas é literalmente esta função sem estado. Um despacho próprio no
> painel seria a segunda resposta a *"que aparência tem um Slider?"*, e a divergência só apareceria
> numa screenshot; há gate de BYTES pinando a delegação.
>
> ⚠️ **O valor de uma row mora no `WidgetStore`, em lugar nenhum mais** — o `AuthoredPanelState` é
> **vazio**. O store é quem o ponteiro escreve, e uma cópia no painel faria o arrasto mover uma e o
> `paint` ler a outra: o controle não se mexeria sob o dedo. O `WidgetEvent` do repo já diz isto
> (*"value-bearing variants carry only the `NodeId` — the caller re-reads from the store"*).
>
> ⚠️ **Só quem RESPONDE vira widget interativo** — a lei do §4 um degrau abaixo. Um `Divider`, um
> `SectionHeader`, um `Spinner` desenham e não têm o que dizer sobre um clique; registá-los faria o
> clique acendê-los e não fazer nada. A pergunta é feita **uma vez** (`Row::is_control`) pelos três
> consumidores, e as duas metades têm gates independentes (registo · retângulo de hit).
>
> ⚠️ **O TÍTULO é do artista; o ID é da plumbing.** Derivar o `Panel::ID` do desenho foi construído
> e **desfeito**: ele é a chave do mapa de visibilidade e o literal que toda lista de painéis do
> shell carrega, então derivá-lo deixaria entrada órfã no mapa a cada rename, tornaria aquelas
> listas impossíveis de escrever, e o app passaria a pensar *"é outro painel"* quando o artista só
> trocou um rótulo. Quem o pergunta usa a porta `visibility_key()` — **nunca** um literal.
>
> **O abridor é o chip `Show as Panel`** da seção **Frame**, docado ao LADO do inspector (a doca do
> Wet Tuning): se ele tomasse a mesma vaga, abrir cobriria o abridor. ⚠️ O X do painel e o chip
> escrevem o **MESMO** fato, então fechar por um apaga o outro — um bool próprio no painel seria a
> segunda resposta que fica acesa sobre um painel fechado.
>
> ⚠️ **DUAS enumerações apodrecidas foram encontradas e curadas de carona:** a lista de unicidade
> dos ids de scrollbar não tinha as **duas** últimas entradas (Wet Tuning 837, Tokens 838) — o gate
> que existe para impedir uma colisão era cego aos ids mais propensos a colidir; ela ganhou um
> irmão que **lê o próprio fonte** e faz da omissão uma falha. E o `SHELL_DRIVEN_PANELS` não tinha
> o painel de **tokens**, cuja tecla `T` alterna a visibilidade.
>
> **FALTA:** a row **não MEXE em nada** — ela emite `(chave, valor)` pela fila de intents, e quem
> escuta é a W4b/W8a. A fronteira está **nomeada na cena de smoke**, não escondida; e um slider que
> se arrasta e se move já é a entrega desta fatia.

🔨 **E a W8b.3 fechou esse FALTA (2026-08-05)** — mesma cena `=62`.

> ✅ **A row MEXE na arte.** Componente novo `VecWidgetBind { target }` no ECS (registro **52 →
> 53**, os dois espelhos **53 → 54**), o conta-gotas **Bind Shape…** na seção Widget Skin, e o
> valor VIVO da row projetado no que a cena desenha: um **Slider** dá a **opacidade** da forma, um
> **Toggle** dá o **aparecer**.
>
> ⚠️ **O que uma row FAZ é DERIVADO do tipo dela** (`vec_widget_drive::drive_of`), nunca um campo
> autorado ao lado — um segundo controle teria de ser mantido de acordo com o tipo, e no dia em
> que discordassem o painel pintaria um slider que apaga a forma. É a lei que a W7 já usou para o
> PAPEL de um estado em vez de um nome livre.
>
> ⚠️ **Quem não produz VALOR não dirige, e o conjunto é MENOR que o dos controles:** um `Button`
> responde ao ponteiro e produz um **evento** — vinculá-lo exigiria um vocabulário de VERBOS que
> nenhum consumidor pede, e vocabulário sem consumidor é desenhar no escuro (a lei do
> `OverrideSlot` da W5a). A linha *Drives* **nem aparece** nele, em vez de aparecer inerte.
>
> ⚠️ **VISTA, nunca documento** — o resultado vai para o `VecViewState`, a projeção por-frame, e
> **nada escreve a cena**. Três consequências e as três são o motivo: arrastar um slider **não
> vira passo de undo** (o undo é por DIFF do mundo — a lição que o ADR-0153 pagou no auto layout) ·
> a tinta autorada **sobrevive** (levar a zero e voltar devolve a arte AO BIT) · e o fato tem UM
> dono. `BoundPaint` ganhou `alpha`, e ela **ESCALA** o alfa existente em vez de o substituir: é o
> que preserva a ESPÉCIE da tinta — o atalho (trocar o `fill` por uma cor com alfa) achataria todo
> gradiente em silêncio.
>
> ⚠️ **O gesto reusa o `PathPick`** (variante `WidgetBind`), e não um segundo modal: arma, o
> clique resolve, o Escape desiste, o hover realça — o segundo modal é o que esquece o Escape.
> **Despir leva o vínculo junto** (um `VecWidgetBind` sobre uma forma que já não veste é um fio
> pendurado em nada, que viajaria no save e ressuscitaria dirigindo).
>
> ⚠️ **DUAS mutações sobreviveram, e as duas acusaram DEFEITO MEU:** (a) o gate da chave usava o
> rótulo `"Opacity"` — **uma palavra alfanumérica só**, onde `key_of` e um `to_lowercase()` cru
> dão a MESMA resposta ⇒ ele era verde sobre a segunda porta que existe para proibir (fixture que
> não contém o fenômeno; o rótulo virou `"Fill Opacity"`); (b) eu afirmara num doc-comment que sem
> o arredondamento `255 × 255` daria 254 — **é falso**, `255 × 255` é divisível por 255 e a
> identidade vale nas duas contas. O `+127` é sobre **VIÉS** (truncar erra sempre para baixo), e o
> gate novo mede onde o erro se vê (`100 × 130/255 = 50,98`).
>
> **11 mutações, 11 sangram** (depois das duas correções). **Zero schema** (`PROJECT_SCHEMA` **56**
> e `VEC_SCENE_SCHEMA` **14** intocados — um componente novo cunha blob-key própria), **zero
> `Cargo.toml`**, **zero ADR**, contrato congelado intacto.
>
> **FALTA, e é a fronteira honesta:** o valor de um controle vive no `WidgetStore`, que é de
> runtime ⇒ **ele não é salvo**. Reabrir devolve os controles ao default e a arte ao que o artista
> autorou. Guardar a POSIÇÃO de um controle é a W4b/W8a — o fio existe, a memória dele não.

**Schema.** +1 componente (**49 → 50**). **Gates:** o desenho e o widget pintam **os mesmos bytes**
(readback) · um `kind` desconhecido degrada para o desenho, nunca para um painel vazio · trocar um
token muda os dois lados na mesma direção.

**Smoke** (`=54`): um slider desenhado ao lado do slider nativo, com os mesmos tokens — indistinguíveis;
mudar `radius.md` move os dois.

---

### W7 — A INTERAÇÃO: máquina de estados + Smart Animate 🔨 **A METADE DE AUTORIA CONSTRUÍDA (2026-08-05)** — a de RUNTIME não

> ✅ **A crate `ph2d-ui-state` existe e o Smart Animate é uma função pura.** O censo confirmou as
> quatro peças que este parágrafo prometia (`Plan::new`/`at`, `VecMorph.t`, `OklabColor::lerp`,
> `EasingFamily::eval`) e confirmou que **HSM não existia** — ela é o que falta.
>
> ⚠️ **Uma medição decidiu a FORMA da API antes de uma linha ser escrita:** `Plan::new` custa
> **0,64 ms mesmo com as duas geometrias IGUAIS** (a busca de fase roda de qualquer maneira),
> contra 0,0001 ms de um passo. Um `smart_animate(from, to, t)` que casasse por frame seria
> inutilizável, e um par só-de-cor que construísse `Plan` custaria **12,79 ms em vinte objetos** —
> 77% de um quadro — para não mover um vértice. ⇒ `Transition::new` (o casamento, uma vez) +
> `Transition::at` (o passo), o mesmo idioma do `Plan`; e **geometria idêntica não constrói `Plan`
> nenhum** (medido depois: transição só-de-cor **0,0001 ms**, 8114× mais barata; 20 objetos =
> **0,00 ms**). O gate `a_colour_only_change_builds_no_plan` **CONTA** em vez de cronometrar, e a
> sonda que imprime os números é `measure_what_a_transition_costs`.
>
> ⚠️ **`mix_paint`/`mix_stroke` do `ph2d-vec-blend` viraram `pub`** — o consumidor novo precisa
> interpolar tinta sem pagar um `Plan`, e reimplementá-la seria a segunda resposta a *"como duas
> tintas interpolam neste app"*.
>
> ⚠️ **A rotação vai pelo ARCO MAIS CURTO, e a consequência é nomeada:** uma volta inteira autorada
> (0 → 360°) é o mesmo ângulo, logo ela **não gira** — girar N voltas é percurso, e percurso é
> keyframe, não estado.
>
> ✅ **A máquina existe** (`Machine`): `go_to` + `advance(dt)` + `pose()`. Ela **não conta o tempo**
> (o `dt` vem do `Playhead`) e **não sabe o que é um mouse** — *o que aconteceu* é do shell, *que
> pose a cena tem* é dela; uma tabela de gatilhos aqui teria de inventar um modelo de entrada.
>
> ⚠️ **E ela é PLANA, não hierárquica — este plano chamou-a de HSM e a construção não entregou
> uma.** O que shipou é `Vec<UiState>` + um índice: não há sub-estado, não há aninhamento, não há
> transição entre máquinas. Ela é o que a W7 precisava (*um botão tem quatro poses*), e o nome
> errado é caro do jeito que só um doc consegue ser — a próxima LLM assumiria uma capacidade que
> não existe e desenharia em cima dela. **Um menu que abre com sub-estados é a hierarquia**, e ela
> continua por construir.
>
> ⚠️ **Três leis que o plano não tinha escrito, e que decidem se aquilo parece produto:**
> **(a)** uma transição parte da pose **VIVA**, nunca da autorada — sair do hover a meio volta *de
> onde está*, e partir do estado autorado faria a cena SALTAR para a ponta antes de voltar (o
> defeito que qualquer um vê e ninguém sabe nomear); **(b)** a **chegada é EXATA** (a pose vira o
> estado autorado ao bit, nunca o resultado de um `t = 1` numérico) — sem isso a cena **deriva**, e
> depois de algumas dezenas de hovers o botão já não está onde o artista o desenhou; **(c)** o
> **easing deforma o `t`, nunca o relógio** — deformar o `dt` faria duas transições com a mesma
> duração autorada acabarem em instantes diferentes.
>
> ⚠️ E *"duas no mesmo frame não empilham"* é a lei que o plano pedia: a segunda **substitui** a
> primeira. Uma fila faria a UI perseguir gestos que o artista já abandonou.
>
> **16 gates, 13 mutações, 13 sangram** — ⚠️ e **duas** delas sobreviveram à primeira rodada por
> defeito de FIXTURE meu, não por buraco de produto: o gate do empilhamento pedia `1, 0, 1` com
> dois estados, onde *"a primeira ganha"* e *"a última ganha"* dão a MESMA resposta; e o da duração
> zero movia um objeto só, onde um ramo próprio produz exactamente o mesmo resultado que a porta
> certa — as duas portas só divergem onde a `arrive` faz algo a mais: **remover quem saiu**.
>
> ✅ **A AUTORIA fechou** (2026-08-05): a seção **States** com os quatro papéis, **Rec / Show /
> Clear** por papel, o slider de duração e o readout do papel vivo. O estado tem um **PAPEL**
> (`Default/Hover/Pressed/Disabled`), não um nome livre — com um nome, o gatilho exigiria uma
> **segunda tabela** a manter em dia com a lista; com o papel ele é **derivado**.
>
> ✅ **A FORMA entra no estado** (2026-08-05, 2ª rodada de smoke): modo Node, **Fillet**, **Chamfer**,
> a pilha de LPE e o **Width Tool** animam. ⚠️ O defeito que o smoke expôs é o mais caro deste
> plano até agora e vale como lei: `ObjectPose::geometry` **existia**, a `Transition` sabia casá-la,
> o `install` sabia escrevê-la — e **nenhum produtor preenchia o campo**. *Uma capacidade sem PORTA
> passa em todos os gates*, porque eles leem quem CONSOME.
>
> ⛔ **A MOLA foi MEDIDA e o solver NÃO se constrói** (a M6 do §9, fechada). A forma já está no
> catálogo (`Elastic Out`: pico 1,373 / assenta 0,631 / 4 travessias, contra 1,309 / 0,600 / 3 de
> um oscilador massa-mola macio), e a pergunta de verdade — a **interrupção** — o default passa:
> revertendo a 30% do caminho, a volta arranca a **1,34×** a velocidade com que a ida chegava.
> ⚠️ Os dois regimes onde ela morde (`InOut` **0,00×**, `Elastic` **7,02×**) são **inalcançáveis
> hoje** porque o seletor de curva não existe — **o dia em que ele nascer, esta medição volta à
> mesa**. Sonda: `measure_spring`.
>
> **FALTA, e o que falta não é polimento:**
>
> - ⛔ **O mouse não dirige nada** — e a ausência é **decisão**, não esquecimento. Um hover que
>   animasse enquanto o artista trabalha tornaria o editor inutilizável (o Figma põe a interação num
>   **modo de apresentação** separado), e há um segundo motivo que é nosso: o undo deste editor é
>   por **DIFF do mundo**, então uma pose escrita por hover viraria passo de undo a cada passagem do
>   rato. **Ligar o mouse exige um modo de preview com história própria.**
> - ⛔ **O input de runtime → token → arte** (a ponte do Vol. 3 §4 que o smoke desta wave pedia) é a
>   **W8a**, não esta.
> - ⛔ **A hierarquia** (um menu que abre, um card que expande com sub-estados) — ver acima.
> - ⛔ **O seletor de CURVA**: 11 famílias × 3 modos = **33 combinações**, e o `ph2d-anim` **não dá
>   nome a nenhuma**; um dropdown hoje pintaria identificadores em inglês crus (HR-15). *O catálogo
>   precisa de nomes antes do knob* — e é esse knob que re-abre a medição da mola.

**O quê.** Um botão tem *idle/hover/press/disabled*; um menu abre; um card expande. Estados +
transições + o tween **automático** entre eles (Vol. 2 §5 e Vol. 3 §1).

**O que já existe e não se reconstrói (achado 1.1).** `ph2d-vec-blend` já casa formas (Hungarian +
espiral logarítmica, com quinas preservadas) e o `VecMorph` já tem `t` animável. `ph2d-color` já tem
OKLab. `ph2d-anim` já tem as curvas. **Falta o nível de CENA:** casar *objetos* entre dois estados e
interpolar as propriedades que diferem.

**A porta única.**

```rust
fn smart_animate(from: &State, to: &State, t: f64) -> Pose
// matching por VecPathId (achado 1.3.5) — NUNCA por nome, e o Vol. 2 §0 explica o porquê
// sem par no destino → fade-out; sem par na origem → fade-in; iguais → não animam
// transform: decomposto em T/R/S (nunca lerp de matriz) · cor: OKLab · forma: ph2d-vec-blend
```

⚠️ **A máquina de estados NÃO tem relógio próprio.** O relógio é o `Playhead` (a lição W4.T7 do
Motion: *o `MotionTransport` morreu*). Dois relógios divergem, e o modo de falha é a UI a andar
noutra velocidade que a cena.

⚠️ **O nodegraph é CONGELADO** (§6: `NodeOp=2`/`OpResolver=1`/`NodeManifest=8`) ⇒ a máquina de estados
**não** é um grafo de nós; é uma HSM própria numa crate nova (`ph2d-ui-state`), pequena, com
`advance(dt)` — exactamente o que o Vol. 3 §1 descreve.

**Molas.** `EasingFamily::Elastic` é uma **curva**, não um solver. Para o *feel* de UI moderna o
Vol. 2 §5 pede mola física (stiffness/damping). ⚠️ **MEDIR primeiro** se a curva basta no smoke — se
bastar, não se constrói o solver (e a nota fica escrita para ninguém o re-propor).

**Schema.** `DOC_VERSION` (a máquina viaja no documento) · +1 crate.

**Gates.** casar por id sobrevive a **renomear** e a **reordenar** (o gate que o Vol. 2 §0 pede, e
que a fragilidade do Figma justifica) · uma transição sem par faz fade · a forma interpola pelo
motor do `blend` (e não por um segundo) · **duas transições disparadas no mesmo frame não empilham**.

**Smoke** (`=55`): um botão com 4 estados e um menu que abre; o mouse dirige; e um **input de
runtime** (um número) dirige um token que dirige a arte — a ponte do Vol. 3 §4.

---

### W8 — OS DOIS BACKENDS

#### W8a — O runtime (para os jogos, e para o engine embarcado)

Um `advance(dt)` que roda **layout → binding → estado → geometria derivada** e desenha pelo
`ph2d-vec-render` que já existe. ⚠️ **É aqui que este plano encontra a Front 2 (`ph2d-runtime`)**: o
documento de UI é uma **seção** do envelope que a `line/runtime` construiu (F1.W0), não um segundo
formato. Um `.ph2dui` avulso seria um segundo carregador, um segundo versionamento e um segundo
lugar onde o carry de seções desconhecidas pode ser esquecido.

**Gates.** o runtime **não** depende do editor (o gate irmão do `no_codec_reaches_the_mixer` do
áudio) · uma UI carregada e uma UI autorada desenham **os mesmos bytes** · o `advance` é
**determinista** (mesmo `dt`, mesmos bytes — a política do `physics_ecs_c9`).

#### W8b — O codegen (para o editor do PH2D)

Emite Rust: os `NodeId` (hash-de-string), as chaves de i18n, os consts de token e um `impl Panel` com
`populate`/`paint`/`apply_event` sobre os 44 widgets.

⚠️ **O código emitido tem de passar os gates que o repo já cobra de código escrito à mão** — e isto é
o requisito mais duro e mais valioso da wave: sem números mágicos (`no_magic_numeric`), todo id
registrado **e clicável** (`architecture_panel_wiring_parity` + varredura de seam), paridade
`populate`↔`paint`↔`event`, e o **teto de LOC** (600 na shell / 700 nas crates) — ou seja, o gerador
tem de **saber dividir um painel grande em irmãos**. Um gerador que produz código que o CI recusa não
serve para nada.

**Gates.** o painel gerado compila e passa os gates · re-gerar duas vezes dá o **mesmo** arquivo
(determinismo) · editar o desenho e re-gerar **preserva** o que foi escrito à mão fora das marcas
(ou o gerador é *write-once* e diz isso na cara — decisão de produto, §9).

**Smoke** (`=56`): desenhar um painel simples, gerar, compilar, e **abri-lo no app** ao lado dos
painéis escritos à mão.

---

### W9 — INTEROP

**DTCG** (import/export da tabela de tokens — o formato W3C que o Penpot/Tokens Studio/Style
Dictionary falam) · **SVG** de entrada com hierarquia (hoje `ph2d-imageio-svg` serve *imagem*, não
documento) · **exportação de sprites/atlas** da UI para o jogo.
⚠️ **Importar arquivo `.fig` fica fora** (formato fechado, exigiria rede/plugin).

---

## §5 — O que fica FORA, e por quê (cercas de Chesterton)

| Fora | Motivo |
|---|---|
| **Vector networks (Vol. 2 §1)** | É o item **mais caro** do estudo (o próprio Vol. 2 o chama de *"a feature tecnicamente mais difícil de replicar"*) e o **menos alinhado** com o pedido: ele serve a *edição de canvas*, não a *geração de UI*. E o que ele entrega em UX já tem resposta aqui: o **Shape Builder** já pinta regiões de um arrangement, o compound path já existe, `path_cut`/`path_join` já fazem topologia, e o *Live Corners* já dá a quina. ⇒ **linha própria, quando o desenho expressivo for a prioridade** — não neste plano. |
| **Cassowary / solver de constraints** | O próprio Vol. 2 §2 diz que âncoras bastam para 90% do HUD; o caso relacional já tem resposta (align/distribute, snap 2-D). Um simplex incremental para o que dois `f64` resolvem é over-engineering. |
| **Colaboração multi-usuário** | Fora da tese (o PH2D é o engine do artista solo — `user_role`). |
| **Import de `.fig`** | Formato fechado. |
| **Solver de mola físico** | ⚠️ **Condicional**: só se o smoke da W7 mostrar que a curva `Elastic` não dá o *feel*. MEDIR antes. |

---

## §6 — Contratos congelados, schema, e o preço de cada bump

### 6.1 Nada do §6 é tocado (verificado estruturalmente)

| Contrato | Gate | Este plano |
|---|---|---|
| Nodes (`NodeOp=2`/`OpResolver=1`/`NodeManifest=8`) | `ph2d-nodegraph/tests/architecture_contract_surface.rs` | ⚠️ **intocado** — a HSM da W7 é crate nova, **não** um grafo de nós |
| Tools (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`) | `ph2d-editor-core/tests/architecture_tool_contract_surface.rs` | intocado — os modos novos são internos ao `VectorTool` |
| Vector doc (`VectorOp≤16`/`Vertex`/`Segment`/…) | `ph2d-vector-doc/tests/architecture_vector_contract_surface.rs` | intocado — **o gate varre só `ph2d-vector-doc` + `-traits`**, e todas as waves vivem em `ph2d-vec-*`, `ph2d-ecs`, `ph2d-panel-vector` e `shells/desktop` |

### 6.2 O registro de componentes

**41 hoje** (`ph2d-ecs/src/scene/registry.rs:374`). O plano acrescenta **9**: `VecFrame`,
`VecBoolGroup`, `VecLayout`, `VecLayoutItem`, `VecAnchors`, `VecBindings`, `VecComponentMain`,
`VecInstance`, `VecWidget` → **50**. ⚠️ **O contador é TRÊS** (`ph2d-ecs`, `ph2d-render`, `ph2d-script`,
cada um rodando na suíte da própria crate) — a família que já ficou **vermelho-latente duas vezes** na
`line/Vector`. Todo commit que acrescenta componente bate os três.

### 6.3 Os bumps, e o que cada um custa

⚠️ **A regra que este plano segue e que a §5 do CLAUDE.md documenta em sete lugares:** *componente
NOVO não bumpa nada* (cunha `stable_type_id = blake3(NOME)[..8]`); *campo apendado a componente
EXISTENTE bumpa*, porque o postcard é posicional **dentro** do blob — e **um bump recusa todo projeto
já salvo**.

| Wave | O que muda | Bump |
|---|---|---|
| W0, W1, W3, W5, W6 | componentes NOVOS | **nenhum** |
| W2a | `VecTextParams.wrap_width` — campo apendado a componente **existente** | ⚠️ **um bump global hoje** |
| W4 | `PropKind::Token` (variante apendada) | `DOC_VERSION` +1 (quebra dura, a política da timeline) |
| W4, W8a | a tabela de tokens e o documento de UI viajam no arquivo | **seções novas** do envelope |

⚠️ **A dependência entre linhas, e é a mais importante deste plano:** a **F1.W1** da `line/runtime`
(*uma versão por `ComponentBlob`* — 18 dos 37 bumps históricos pousaram lá dentro) **apaga o custo da
W2a**. Se a W2a for antes, ela paga um bump global por um campo que 99% dos documentos não têm.
⇒ **ordenar a `line/runtime` antes da W2**, ou aceitar o bump com o motivo escrito.

⚠️ **E há uma colisão de número já em voo:** esta linha tem `PROJECT_SCHEMA = 50` (provisório) e a
`line/runtime` **matou o número** (`LEGACY_SCHEMA_FINAL = 48` + envelope). *O valor se CONTA contra o
`main` do dia, e aqui o que se conta é qual das duas formas sobrevive* — decisão de integração, não
de linha.

---

## §7 — Dependências novas

| Crate | Wave | Por quê | Risco |
|---|---|---|---|
| **`taffy`** | W2 | Flexbox/Grid/Block da spec CSS, Rust puro, sem build-deps; o motor de Dioxus/Zed/Bevy/Blitz. Escrever o nosso é reproduzir uma spec inteira | **ADR próprio** (dep + escolha foundational). ⚠️ Fixar a versão no ADR e **MEDIR o custo de build** — o repo é sensível a isso (§2 do CLAUDE.md) |
| `serde_json` | W9 (DTCG) | ✅ **já está na árvore** (`ph2d-mcp`, `ph2d-asset`, `ph2d-tokens`) | nenhum |
| `parley` | W2a (quebra de linha) | ✅ **já está na árvore** (`ph2d-text` 0.6) | nenhum |

⚠️ `deny.toml` barra `unknown-git` ⇒ tudo vem de crates.io com licença conhecida. E `machete` cobra
dep declarada e não usada — a dep entra **no commit que a usa**.

---

## §8 — A ordem, e por que ela

```
W1 booleana viva ────────────────────────────► (independente; o Enio pediu; a menor)
W0 moldura ──┬──► W2 layout ──┬──► W3 âncoras
             │   (+ W2a texto)│
             │                └──► W8b codegen ──► (o editor)
             └──► W5 componentes ──► W6 widget ──► W7 interação ──► W8a runtime ──► (os jogos)
W4 tokens ───┴──► (atravessa TUDO: W4 é pré-requisito de W6 e de W7)
```

1. **W1 (booleana viva)** primeiro: é independente, foi pedida, e o número que a viabiliza já está
   medido. Fecha um item aberto sem esperar nada.
2. **W0 (moldura)**: nada de responsivo existe sem ela.
3. **W4 (tokens)** cedo, e não tarde: é a de **maior alavancagem** (Vol. 2 §4), já tem metade
   construída, e é a que sozinha entrega *"eu mesmo desenharei a UI"* (degrau 1 da W6). ⚠️ Fazê-la
   depois da W5/W6 significaria autorar componentes com literais e migrá-los depois.
4. **W2 + W3**: a responsividade, que é requisito de plataforma (iPad × desktop), não luxo.
5. **W5 → W6**: o prefab e depois o vínculo funcional (o vínculo só faz sentido sobre instâncias).
6. **W7**: depende de casar por id (W0/W5) e de tokens (W4).
7. **W8**: os dois backends, quando há o que exportar.
8. **W9**: interop por último — ninguém importa DTCG antes de haver tabela autorável.

⚠️ **A `line/runtime` (F1.W1) atravessa isto na W2a** — §6.3.

---

## §9 — As medições que faltam (fazer ANTES da wave, não depois)

Cada uma é uma sonda `#[ignore]`, e cada uma decide um número que este plano **deliberadamente não
escreveu**:

| # | Medição | Decide |
|---|---|---|
| M1 | `N` booleanas vivas animadas em simultâneo (5 / 20 / 100) | se a W1 precisa de orçamento por frame, ou se o memo basta |
| M2 | passe de layout com 10 / 100 / 1000 nós; árvore reconstruída × memoizada | o teto de nós de uma moldura, e se a memoização é necessária |
| M3 | custo da quebra de linha (parley) por bloco de texto, por frame | se o texto precisa de cache de medição |
| M4 | resolução de binding: 0 / 100 / 1000 propriedades bindadas | o custo do §4, e se a tabela achatada é obrigatória já no editor |
| M5 | memória de 1000 instâncias (dhat) — prototipal × cópia | confirma a tabela do Vol. 2 §3 **neste** modelo |
| M6 | `Elastic` × mola física num smoke lado a lado | se o solver de mola se constrói ou fica fora |
| M7 | custo de build do `taffy` (cold + warm) | entra no ADR |
| M8 | tamanho do arquivo: uma tela de UI típica como seção | o formato, e o carry |

---

## §10 — Riscos nomeados

1. ⚠️ **Duas árvores.** A do ECS (autorada) e a do taffy (derivada). Mitigação: a segunda é
   **memoizada e reconstruída**, nunca autorada — e há gate provando que ninguém escreve nela.
2. ⚠️ **Duas poses.** O `Transform` autorado e a pose de layout. Mitigação: a de layout **nunca**
   toca o `Transform`; arrastar dentro de um fluxo **reordena**. Sem isso, o undo enche de passos
   espúrios (é o defeito exacto que o `canonicalize` do undo global existe para matar).
3. ⚠️ **Duas aparências para um widget** (o desenho e o pintor nativo). Mitigação: a prévia do canvas
   chama **o pintor real**.
4. ⚠️ **Codegen que o CI recusa.** Mitigação: os gates do repo correm sobre o código gerado, e isso é
   requisito de aceitação da W8b, não polimento.
5. ⚠️ **O bump da W2a** (§6.3) — ordenar a `line/runtime` antes, ou pagar com o motivo escrito.
6. ⚠️ **Escopo.** São nove waves; o Vol. 2 inteiro é maior que qualquer wave já feita nesta linha.
   Mitigação: **W1 e W0 entregam valor sozinhas**, e a W4 entrega *"desenhe a UI do app"* antes de
   qualquer componente existir. Nada aqui precisa de um *big bang*.

---

## §11 — Tabela-resumo

| Wave | Entrega | Componentes | Schema | Dep | ADR | Smoke |
|---|---|---|---|---|---|---|
| **W1** ✅ | booleana não-destrutiva, animável | +1 (**42**) | — | — | — | `=48` |
| **W0** ✅ | a moldura | +1 (**43**) | — | — | — | `=49` |
| **W4a** ✅ | o binding de token no documento + o modo re-veste | +1 (**44**) | — | — | — | `=51` |
| **W4b/c** | tabela autorável (aliases/math/DTCG) + animar token | — | `DOC_VERSION`, seção | — | sim (indireção) | — |
| **W2** ✅ | auto layout | +2 (**46**) | — (W2a fica) | **taffy** | **0153** | `=50` |
| **W3** ✅ | âncoras | +1 (**47**) | — | — | — | `=52` |
| **W5a** ✅ | mestre + instância + override esparso | +2 (**50**) | — | — | — | `=53` |
| **W5b** ✅ | a lista de PEÇAS (a porta do override) · Update Main · Swap | — | — | — | — | `=56` |
| **Z-order** ✅ | a lei das game engines: DFS = ordem de desenho + o **Z global** na Arrange | — | — | — | — | `=57` |
| **W5c** ✅ | variants (a instância escolhe QUAL versão) | — | **—** | — | **—** | `=58` |
| **W6.1** ✅ | a tabela de COR vira AUTORÁVEL (o degrau 1 do §2) | — | ⚠️ **bump** | — | **—** | `=59` |
| **W6.2** ✅ | pele por-widget (`VecWidget`): a forma veste um controle do catálogo | +1 | — | — | — | `=60` |
| **W6.3** | layout: a árvore autorada vira `Panel` | — | — | — | sim (§2) | — |
| **W7** 🔨 | estados + Smart Animate (**autoria**; runtime não) | — | ⚠️ `PROJECT_SCHEMA`, **não** `DOC_VERSION` | — | **nenhum** | `=61` |
| **W8b.1** ✅ | o codegen: a árvore descreve um painel e o app escreve o código dele | — | — | — | — | `=62` |
| **W8b.2** ✅ | o gerado vira painel VIVO (crate, registro, runtime das rows) | — | — | — | **nenhum** | `=62` |
| **W8b.3** ✅ | a row MEXE na arte (o vínculo row → forma, derivado do tipo) | +1 (**53**) | — | — | **nenhum** | `=62` |
| **W8a** | o runtime (para os jogos) | — | seção | — | sim (fronteira) | — |
| **W9** | DTCG / SVG / export | — | — | — | — | — |

⚠️ **A linha da W7 traz TRÊS previsões que a construção refutou, e elas são o padrão desta
tabela:** o schema seria `DOC_VERSION` e foi **`PROJECT_SCHEMA`** (a tabela de estados viaja no
`ProjectState`, que é a unidade do undo — não num blob que carrega a própria versão); o ADR era
dado como certo (*"sim (HSM)"*) e **nenhum foi escrito** (nada do §6 é tocado, nenhuma dep externa
entra, e as decisões vivem no doc da crate — a linha fica **fora** da disputa de número); e a cena
prevista era `=55`, mas o número livre no roteador era o **`=61`**. *Uma tabela de previsões é útil
enquanto a construção puder contradizê-la por escrito.*

⚠️ **Os números de CENA desta tabela são previsões, e uma delas já colidiu.** O W2 e o W4a
previam os dois `=50`; o auto layout chegou primeiro e a cena dos TOKENS ficou **inalcançável em
silêncio** (o roteador é uma lista de `if level == N` e o primeiro vence). Ela mudou para `=51`, e
as previsões seguintes deslizaram um. O gate `no_two_smoke_scenes_claim_the_same_level` é o que
torna a próxima colisão alta em vez de silenciosa — **confira o número livre no roteador**, não
nesta tabela.

⚠️ Os números de ADR são **PROVISÓRIOS** até a integração — o valor se conta contra o `main` do dia
(seis renumerações já aconteceram neste repo). ⚠️ **E o mesmo vale para os números de cena de smoke:**
`=48..=56` estão livres *nesta linha* (a `=47` é o alinhamento, a última daqui), mas outra linha
paralela pode reivindicar os mesmos — conte contra o `build_smoke_router` do `main` do dia.

---

## Referências

- `Estudos/PH2D_manual_features_figma.md` — Vol. 2 (vector networks, auto layout, components,
  variables, smart animate).
- `Estudos/PH2D_manual_features_rive.md` §1, §4, §6, §7 — state machines, data binding, formato
  runtime-first, layouts responsivos.
- `Estudos/PH2D_manual_features_vetoriais.md` — Vol. 1 (o que se desenha).
- ADR-0108 (reposicionamento), ADR-0110 (paths são entidades ECS), ADR-0111 (geometria local, pose no
  `Transform`), ADR-0121 (fonte autorada ≠ cozida), ADR-0132 (pilha de efeitos), ADR-0148 (perfil de
  largura é componente ECS + um assador).
- `docs/design/tokens.json` (350 folhas, 4 temas) + `crates/ph2d-tokens/` (`build.rs`,
  `tests/design_token_sync.rs`).
- `docs/Runtime/` (`line/runtime`) — o envelope por seções (F1.W0) e a versão por `ComponentBlob`
  (F1.W1).
