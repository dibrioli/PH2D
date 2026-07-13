# HANDOFF — `line/Vector`, continuação (2026-07-13, 2ª passagem)

> ⚠ **SUPERADO** por [`HANDOFF_line_vector_continuacao_2026-07-13c.md`](HANDOFF_line_vector_continuacao_2026-07-13c.md)
> (o Shape Builder foi consertado, commit `7aa9fc7d`). Este arquivo fica como HISTÓRICO — e
> vale a leitura pela §3 e pela §9, que são as lições. Mas o **diagnóstico** dele está errado:
> o suspeito nº 1 (xform stale) não era a causa, e a matemática do arranjo estava certa. As
> causas reais estão na §3 do 13c.

> Do agente que quebrou o Shape Builder, para **você**, que vai consertá-lo.
>
> **A linha NÃO está fechada.** O Shape Builder está commitado, compila, tem 16 gates verdes
> e **não funciona**. O Enio smokou e reprovou. A sua primeira tarefa é consertá-lo; a fila
> vem depois.
>
> **Leia a §3 antes de escrever uma linha de teste.** É a razão de 16 gates verdes não terem
> pego nada — e se você não a internalizar, vai escrever o 17º.

---

## §1 — Prepare a linha

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && git fetch origin && git rebase main
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo nextest run --workspace --no-fail-fast
```

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Branch | `line/Vector` |
| Commits **não integrados** | 8 (duas waves: Live Corners **aprovada no smoke** + Shape Builder **reprovada**) |
| Suíte | 6631/6631 verde — **e isso não quer dizer nada, ver §3** |

> ⚠ A wave anterior (**Live Corners** — a alça de raio de quina) foi **smokada e aprovada
> pelo Enio**. Não a mexa. O handoff dela é
> [`HANDOFF_line_vector_integracao_2026-07-13.md`](HANDOFF_line_vector_integracao_2026-07-13.md);
> o ADR é o [0119](architecture/decisions/0119-vector-live-corners-authored-source-cooked-geometry.md).

> ⚠ Não rode cargo no repo primário (`/home/enio/Documentos/Projetos/PH2D`). Modo L: você
> **pode** tocar foundational (ADR-0107), **não pode** integrar nem pushar sem ordem
> explícita do Enio (CLAUDE.md §0.7).

---

## §2 — O que o Enio viu (e é o alvo)

Ele desenhou formas sobrepostas (um pentágono, uma estrela, e uma forma grande arredondada),
entrou no modo **Build**, e reportou:

1. **Undo/redo não funciona.**
2. **As silhuetas das formas sobrepostas somem** — deveriam continuar visíveis no Build.
3. **O "véu" está estranho e grande, e não bate com as formas.**
4. **"Há problemas no algoritmo."**

O print está no histórico da conversa. Nele dá para ver: um véu **rosa** grande cobrindo as
formas, e dentro dele um trapézio pequeno com **um buraco preto** — geometria que não
corresponde a região nenhuma.

---

## §3 — **POR QUE 16 GATES VERDES NÃO PEGARAM NADA** (leia isto primeiro)

Eu medi, e os números são estes:

| | |
|---|---|
| Gates que usaram uma forma do **catálogo** (com curvas) | **0** |
| Gates que usaram um `Transform` **não-identidade** | **0** |
| Gates que passaram pela **ordem real do frame** (`build_down`/`move`/`up`/`upkeep`) | **0** |

**Todos os 16 usaram quadrados eixo-alinhados, construídos à mão, na identidade, chamando a
lógica pura direto.** Provei a matemática do arranjo com fixtures que o produto nunca produz:
o Enio desenha com a **Shape tool**, e toda forma dela é uma **Live Shape** — geometria
**curva**, **centrada no local 0**, com a pose num **`Transform`** (ADR-0111). Nenhum dos meus
testes viu isso.

É exatamente a memória [`feedback_test_with_product_numbers_not_convenient_ones`] e a
regra-mãe da [DIRETIVA_IMPLEMENTACAO](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md):
**verde-de-compilação é velocidade; no audit vale ZERO.** Eu até mutei o código e me
convenci de que os gates mordiam — e mordiam mesmo, mas **só dentro do universo de
quadrados** em que eu os escrevi.

**A sua primeira ação: escreva um gate que reproduza o print do Enio.** Formas do catálogo
(`ph2d_vec_scene::cook(ShapeKind::Polygon, …)`, `Star`, `RoundRect`), com `Transform`
não-identidade, atravessando `build_session_upkeep → build_down → build_move → build_up`.
Ele tem que nascer **VERMELHO**. Se nascer verde, o gate está errado — não o produto.

---

## §4 — O que eu **confirmei** (não é palpite)

1. **O rosa É o meu overlay.** `ColorToken::Accent` no tema Forge é `rgba(216,132,189)`.
   Medido, não deduzido.
2. **A matemática do arranjo está CERTA** para um polígono + uma elipse do catálogo, na
   identidade: as bboxes das faces batem, cada face contém o seu ponto, as componentes saem
   separadas. Rodei. **Então o bug não está em `compute_region`.** Ele está entre o arranjo e
   a tela.
3. **`held_button` é limpo ANTES dos meus early-returns** (`input_dispatch.rs:2160` vs 2501 e
   2628), então o undo global *deveria* disparar. A causa do undo é outra — ver §5.1.

---

## §5 — Os suspeitos, em ordem, com o código

### 5.1 — Undo (CERTO que está errado, e é o mais fácil)

`shape_build_gesture.rs::build_up` chama `self.vec_history.push_undo(pre)`.

**O `vec_history` é uma fila MORTA.** O CLAUDE.md diz, sobre o undo global (`undo.rs`):
> *"O `vec_history` foi **subsumido** (a geometria está na captura; ainda é populado mas
> **não lido**)."*

Ou seja: eu empurrei o undo para um lugar que ninguém lê. O undo real é o **global, por
DIFF**, em `App::post_frame_undo` — e ele registra um passo quando `had_input && held_button
== None`.

**O que investigar:** o diff *deveria* pegar a mudança na `VecScene`. Rode com
`PH2D_UNDO_LOG=1` e veja se o passo é registrado. Duas hipóteses:
- o `Ctrl+Z` está sendo consumido por outro dono antes de chegar ao global
  (`input_handlers.rs:184` — a cadeia audio/painter/motion/timeline);
- ou o passo é registrado e o **restore** não reconstrói a ponte path↔entidade (o CLAUDE.md
  avisa: *"o restore reconstrói `vec_entities::rebuild_map`, senão o `sync` duplica as
  formas"*).

Comece tirando o `push_undo` morto e confiando no global — depois prove que ele dispara.

### 5.2 — O véu esconde a arte (CERTO, e é a queixa 2 e 3 do Enio)

`ph2d-vec-render/src/build_faces.rs`:
- a face marcada é preenchida a **`MARKED_ALPHA = 0.45`**;
- e recebe um **contorno TOTALMENTE OPACO** (`tint(1.0)`).

Sobre uma forma azul, 45% de rosa + uma borda opaca **apagam a silhueta**. O Enio pediu
exatamente o contrário: *"as silhuetas das formas sobrepostas devem ficar visíveis"*.

**O desenho está errado, não a calibragem** — não fique mexendo no alpha. *"Difícil de
ajustar" é um bug de DESIGN* ([memória](../project-memory/feedback_ergonomics_verdict_is_a_design_bug.md)).
Um véu **opaco o bastante para ser visto** é opaco o bastante para esconder. As ferramentas
que resolvem isso não usam véu sólido:

- **Illustrator (Shape Builder):** hachura diagonal sobre a região, e o traço das formas
  continua por cima.
- **Alternativa:** desenhar só o **contorno** da face (grosso, animado, "marching ants") e
  deixar o preenchimento intacto.
- **Alternativa:** véu bem fraco (≤0.15) + **redesenhar os traços das formas de origem POR
  CIMA do véu** (é o que garante a silhueta, e é barato: as fontes estão no
  `Arrangement::sources()`).

Escolha uma e **mostre ao Enio antes de gastar tempo** — é decisão visual dele.

### 5.3 — O buraco preto dentro do véu (é o "problema no algoritmo")

No print, dentro do véu rosa há um trapézio com um **buraco preto**. Um buraco = a
`fill_rule` da face está `EvenOdd` com um sub-contorno que **não deveria ser buraco**.

Olhe `ph2d-vec-boolean/src/lib.rs::compound_from`:
```rust
fill_rule: if compound { FillRule::EvenOdd } else { FillRule::NonZero },
```
Toda saída com subpaths vira `EvenOdd`. A minha `compute_region` **encadeia** booleanas (a
interseção alimenta a subtração), e cada passo re-agrupa contornos. Uma face que sai com um
contorno interno **de mesma orientação** (não um buraco de verdade) é renderizada como buraco.

**O gate que falta:** cozinhe as formas do catálogo, compute cada face, e asserte que a face
**não tem buraco onde o ponto de dentro está preenchido** — isto é, `contains_point(face, p)`
para uma grade densa DENTRO da face, e compare com o `fill_rule` que ela declara. Um teste
que só olha a bbox (como os meus) nunca vê isso.

### 5.4 — O `Transform` fica STALE no meio do gesto (CERTO, e ninguém pegaria olhando)

`shape_build_gesture.rs::build_session_upkeep`:
```rust
// Só reconstrói quando a SELEÇÃO muda.
if self.vec_build.as_ref().is_some_and(|s| s.sources == sel && !s.dragging) { return; }
```
**Mover uma forma no modo Build não reconstrói o arranjo** — ele continua com a geometria
antiga, e o véu aparece onde a forma *estava*. É um candidato forte para *"não bate com as
formas"*.

E pior: o `upkeep` roda no **PRÓLOGO do frame**, antes de `vec_entities::sync` (2657),
`settle_origins` (2755) e `vec_transform::build` (2772). Ele usa os `xforms` do frame
**anterior**. Para uma forma recém-criada ou recém-movida, o `bake_xform` usa a pose errada —
e como a geometria de toda Live Shape é **centrada no local 0**, um xform errado empilha as
formas na origem do mundo, e as "faces" viram um borrão no meio da tela.

**Suspeito nº 1 do véu grande e deslocado.** Verifique cedo: logue a bbox de
`arr.sources()[i]` e compare com a bbox de mundo real da forma
(`scene.path_world_curve_bbox(id, &xf)`).

**A cura provável:** mover o `upkeep` para DEPOIS de `vec_transform::build` no frame, e
invalidar o arranjo quando **a geometria ou a pose** mudar — não só quando a seleção mudar.

### 5.5 — A ordem de `sources` não é z-order

`BuildSession::open` recebe `pen.selected_paths()`, que é a **ordem de clique**, não a de z.
O doc-comment do `Arrangement` promete "fundo → topo", e a regra do estilo ("herda o do
topo") depende disso. Ordene por z (`scene.paths().iter().position(...)`) como o
`selected_closed_z` da booleana normal já faz.

---

## §6 — O que **está bom** e você pode confiar

- **`Arrangement` (`ph2d-vec-boolean/src/arrangement.rs`)** — a ideia de que uma face é
  `(∩ das que cobrem) − (∪ das que não cobrem)`, com as componentes desconexas saindo de
  graça do `apply_many`. Testei com formas de catálogo e sai certo. **Não jogue fora e não
  construa um DCEL** — o realce e o resultado saírem do MESMO motor é a única razão de eles
  não poderem divergir.
- **A medição** (`ph2d-vec-boolean/tests/measure_face_hit.rs`): 19,9 µs/hover com 8 formas de
  64 verts; ~140 µs frio por região nova. O `Topology`+`WindingNumber` do linesweeper é o
  escape hatch se um dia o número mudar. **Não otimize sem medir de novo.**
- **O seam do pill** (`clicking_build_pill_reaches_the_tool`) — o modo chega à tool. Isso está
  provado.
- **O modo nunca fica mudo:** sem 2 formas selecionadas, o clique no Build **seleciona**
  (Shift soma). Mantenha.

---

## §7 — O que eu fiz FORA do escopo, e você pode precisar reverter

`crates/ph2d-flip-render/tests/pack_perf.rs`: o teto de perf era **único (120 ms)**, calibrado
para os ~14 ms do **release** — mas o `nextest --workspace` roda em **debug**, onde o mesmo
trabalho leva **78 ms ocioso** e passa de **130 ms** sob a carga paralela da suíte. Ficava
vermelho na suíte cheia e verde isolado, e ia bloquear o integrador e o CI.

Fiz o teto ser **por perfil** (700 ms debug / 120 ms release, mesma folga relativa de ~9×). É
arquivo do Flip — linha **já integrada**, e conferi que nenhuma linha viva tem trabalho
pendente nele. **O Enio ainda não vetou nem aprovou. Se ele preferir devolver ao dono do
Flip, é um revert de 3 linhas** (commit `a2313f32`).

---

## §8 — A FILA (o Enio já decidiu)

1. **Shape Builder** — consertar (você está aqui).
2. **Blend / morph** — interpolação de formas.
3. **Envelope / puppet warp** — deformação.

O material de referência está em
[`docs/Vector Module/Estudos/`](Vector%20Module/Estudos/) (3 manuais: geral, Figma, Rive), e o
resumo do que já existe vs. o que falta está no meu levantamento na conversa. **Dois avisos
sobre esses manuais**, que eu levantei cruzando com o código:

- Eles assumem um pipeline **`lyon` + wgpu bespoke com shader nodes**. **Não é o nosso
  Vector** — ele renderiza por **Vello** (ADR-0108), e não temos `lyon` no workspace. As
  "Notas de integração PH2D" deles estão escritas contra uma stack que não é a nossa
  (afeta principalmente a recomendação de **gradient mesh**).
- Eles sugerem **Clipper2** para booleanas. **Já temos** (`linesweeper` + kurbo, Rust puro).
  Adotar Clipper2 seria um segundo motor + FFI C++.
- E eles listam **Vector Networks (Figma)** como fase 2, enquanto a **nossa própria pesquisa
  anterior** (`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md` §2.4) **avaliou e
  RECUSOU**. Os dois docs discordam; a decisão é do Enio.

Para o **Blend/morph**, o achado que muda o cálculo (da pesquisa 20): **ninguém resolveu a
correspondência de formas.** O flubber faz força bruta O(n²); o GSAP tem índice manual e uma
ferramenta de debug que *admite que o automático erra*; o CorelDRAW pede ao usuário para
clicar um nó em cada forma; Lottie e Rive não têm correspondência nenhuma. **O alvo honesto é
bom-automático + escape manual**, e isso é barato.

---

## §9 — As três lições que eu pago para você não repetir

1. **Um gate só prova o que a fixture dele contém.** Mutei o código, vi vermelho, e me
   convenci. Mas mutação dentro de um universo de quadrados só prova coisas sobre quadrados.
   **A fixture é parte do gate, e é a parte que eu não auditei.**

2. **"Simulei o smoke no papel" não é smoke.** Eu fiz isso — e achei um bug real (o modo que
   ficava mudo). Isso me deu a sensação de ter coberto o caminho do usuário, e me fez **pular
   o exercício de rodar o app**. Um passeio mental percorre o roteiro que você IMAGINOU; o
   Enio percorre o que existe.

3. **Não escreva o overlay por último.** Eu tratei o realce como acabamento e a matemática
   como o trabalho. É o inverso: **o realce É a feature** (a booleana já existia), e ele é a
   única parte que o artista experimenta. Ele foi a última coisa que escrevi, sem gate
   nenhum, e é onde estão duas das quatro queixas.
