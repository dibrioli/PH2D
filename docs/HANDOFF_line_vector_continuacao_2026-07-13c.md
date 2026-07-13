# HANDOFF — `line/Vector`, continuação (2026-07-13, 3ª passagem)

> **O Shape Builder foi consertado.** O handoff anterior
> ([13b](HANDOFF_line_vector_continuacao_2026-07-13b.md)) o passava QUEBRADO; ele está
> **superado** — leia este.
>
> A linha continua **NÃO integrada** e **NÃO shipada** (CLAUDE.md §0.7: integração e ship só
> por ordem explícita do Enio, via agente integrador).

---

## §1 — Estado

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Branch | `line/Vector` (rebasada em `main` = `4cd8ef13`) |
| Commits **não integrados** | 11 (Live Corners **aprovada no smoke** + Shape Builder **v1 reprovada** + o **fix** `7aa9fc7d`) |
| Suíte | **6637/6637 verde**, clippy limpo |
| **Pendente do Enio** | **o smoke do Shape Builder consertado** (§2) |

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && git fetch origin && git rebase main
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo nextest run --workspace --no-fail-fast
```

---

## §2 — O smoke (a cena já vem montada)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && PH2D_BUILD_SMOKE=1 cargo run --release --bin ph2d-host-desktop
```

Abre com **pentágono + estrela + retângulo arredondado** sobrepostos, os três selecionados, no
modo **Build**. O que conferir:

1. **Passe o mouse** — a face sob o cursor ganha um véu fino, e as **silhuetas das três formas
   ficam visíveis por cima dele** (era a queixa 2).
2. **Arraste** por várias regiões e solte — as faces tocadas viram UMA forma; **as formas que o
   dedo não atravessou continuam lá, com a cor delas** (era a queixa 4: um clique dissolvia
   tudo num blob azul).
3. **Alt+arraste** na sobreposição — ela some (a lua crescente), e o véu fica **vermelho**
   durante o gesto (apagar não pode parecer unir).
4. **Ctrl+Z** — desfaz o gesto inteiro num passo.

`PH2D_BUILD_SMOKE=2` dirige o gesto por código (pousa e arrasta por duas faces, sem soltar): é
o harness visual do véu, para quem não puder mexer o mouse.

---

## §3 — O que eu achei, e COMO (a parte que importa para a próxima)

O handoff 13b apontava o *xform stale* como suspeito nº 1 e mandava escrever um gate com formas
do catálogo + `Transform`. **Fiz o gate. Ele nasceu VERDE** (`arrangement_product_shapes`) — a
matemática do arranjo aguenta curvas, poses e rotação. O suspeito nº 1 estava errado, e mais
uma tarde de leitura de código não teria chegado a lugar nenhum.

O que resolveu: **montar a cena do print DENTRO do app** (`shells/desktop/src/build_smoke.rs`),
dirigir o gesto no frame de verdade, imprimir os números e **olhar a tela**. Vinte minutos. As
quatro queixas do Enio saíram de **duas** causas, nenhuma delas em `compute_region`:

### 3.1 — Um clique dissolvia a arte (BUGS #12)

`resolve()` devolvia a sobra como `união(todas as fontes) − o levado`: **uma** forma, **um**
estilo. Medido: um clique na estrela trocava as 3 formas por (a face) + (um blob de 24 verts
com a bbox da união inteira) + (uma lasca de 0,05 unidade). O pentágono laranja e a estrela
verde sumiam. **Era a queixa "as silhuetas somem" E o "problema no algoritmo"** — e não tinha
nada a ver com o véu.

Agora cada fonte sobrevive como **ela menos o que foi levado** (estilo dela, z dela), e a que o
gesto **não atravessou não é sequer tocada** — mantém id, entidade, `Transform`, raio de quina
e params de Live Shape. É o Illustrator: o Shape Builder divide o que você percorre e deixa o
resto em paz.

### 3.2 — A borda do véu tinha 150 px (BUGS #13)

`Stroke::new(1.5)` emitido **sob o afim mundo→tela**. O Vello escala o traço pelo afim: não era
1,5 px, era 1,5 **unidades de mundo**. A crate inteira já fazia o contrário (*"o ponto sobe pelo
afim, a espessura não"*, `draw_corner_handles`) — este era o único sítio que destoava, e era o
mais novo. Agora existe `edge_strokes()` (geometria em tela, largura em px) com gate de zoom.

### 3.3 — O realce pairava sobre nada (BUGS #14)

As faces seguem as bordas das formas sobrepostas — mas **uma forma coberta por outra não está
na tela**. Faltava redesenhar as **silhuetas das fontes por cima do véu**. Bug de desenho
ausente, não de geometria.

### 3.4 — O undo FUNCIONA (e o `push_undo` era numa fila morta)

Provado no app com `PH2D_UNDO_LOG=1`: o Up registra o passo, o Ctrl+Z restaura as três formas.
A queixa do Enio era **consequência** de 3.1 — cada clique era um build destrutivo, então um
Ctrl+Z desfazia *um* clique e a arte seguia arruinada pelos outros. O
`vec_history.push_undo(pre)` do `build_up` saiu: é a fila que o ADR-0110+ declara *"populada
mas não lida"*.

### 3.5 — Dois latentes que o smoke nem chegou a expor

- **`sources` (ids) × `arr.sources()` (geometria) podiam DESALINHAR**: o `open` filtrava as
  formas abertas da geometria mas copiava a lista de ids **inteira**. Uma forma aberta na
  seleção deslocava o índice, e o commit **consumiria a forma errada**.
- **A ordem das fontes era a de CLIQUE, não a de z** — e é o z que decide de quem a forma nova
  herda o estilo (a do topo) e onde ela nasce na pilha.
- **O arranjo só era refeito quando a SELEÇÃO mudava.** Ele é assado em MUNDO: se a pose muda
  (gizmo, undo), o véu descreve a forma onde ela *estava*. A chave da sessão (`source_key`)
  agora inclui **geometria e pose**.

---

## §4 — As lições (custaram uma reprovação; não as pague de novo)

1. **A fixture é parte do gate, e é a parte que ninguém audita.** Os 16 gates verdes usavam
   quadrados na identidade. Mutar o código e ver vermelho **dentro de um universo de quadrados**
   só prova coisas sobre quadrados.

2. **Um gate que olha o RETORNO de uma função não vê o que o artista fica.** Todos os 16 mediam
   `resolve()`; nenhum mediu **a cena depois do gesto**. Por isso o `commit` agora é uma função
   pura em `shape_build.rs` — é ela que decide o que morre, e é ela que os gates exercem.

3. **Mute o CÓDIGO, não só o teste — e desconfie do verde.** O meu gate de estilo
   (`what_is_left_of_a_shape_keeps_that_shapes_style`) **ficou verde na mutação**: ele media
   "algum path que contém o ponto", e pegava a forma NOVA em vez da sobra. Reescrito, morde.
   Um gate que não morde é pior que gate nenhum.

4. **Instrumente o app antes de teorizar.** É o §9.2 do handoff anterior, e ele tinha razão.

---

## §5 — A FILA (o Enio já decidiu a ordem)

1. ~~**Shape Builder**~~ — **feito**, aguardando o smoke dele.
2. **Blend / morph** — interpolação de formas.
3. **Envelope / puppet warp** — deformação.

Material: [`docs/Vector Module/Estudos/`](Vector%20Module/Estudos/) (3 manuais: geral, Figma,
Rive). **Três avisos sobre eles**, levantados cruzando com o código (mantidos do 13b, conferem):

- Assumem `lyon` + wgpu bespoke com shader nodes. **Não é o nosso Vector** — renderizamos por
  **Vello** (ADR-0108) e não temos `lyon`. Afeta sobretudo a recomendação de *gradient mesh*.
- Sugerem **Clipper2** para booleanas. **Já temos** (`linesweeper` + kurbo, Rust puro). Adotar
  Clipper2 seria um 2º motor + FFI C++.
- Listam **Vector Networks (Figma)** como fase 2, enquanto a nossa própria pesquisa
  (`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md` §2.4) **avaliou e RECUSOU**. Os
  dois docs discordam; **a decisão é do Enio**.

**Para o Blend/morph, o achado que muda o cálculo** (pesquisa 20): *ninguém resolveu a
correspondência de formas*. O flubber faz força bruta O(n²); o GSAP tem índice manual e uma
ferramenta de debug que **admite que o automático erra**; o CorelDRAW pede ao usuário para
clicar um nó em cada forma; Lottie e Rive não têm correspondência nenhuma. **O alvo honesto é
bom-automático + escape manual**, e isso é barato.

E um pré-requisito que agora existe e vale ouro: a costura **fonte ≠ cozido** do ADR-0119
(`VecPath::cooked()`). Um blend é exatamente isto — a fonte são as duas formas + o `t`, o
cozido é a interpolada. **Live Path Effects como nós** (o item aberto do 13) é o mesmo
mecanismo, e o blend seria o primeiro deles.

---

## §6 — Pendências e ressalvas

- **`crates/ph2d-flip-render/tests/pack_perf.rs`** — o agente da wave passada mexeu **fora do
  escopo** (o teto de perf virou por-perfil: 700 ms debug / 120 ms release, porque o teto único
  de 120 ms era calibrado para o release e o `nextest --workspace` roda em **debug**, onde o
  mesmo trabalho leva 78 ms ocioso e passa de 130 ms sob carga). **O Enio ainda não vetou nem
  aprovou.** Deixei como está: reverter agora reintroduz um vermelho intermitente na suíte e no
  CI. Se for para devolver ao dono do Flip, é um revert de 3 linhas (`a2313f32`, hoje
  `a95d6446` pós-rebase).
- **A lasca.** Quando a ponta de uma forma escapa da região levada, ela sobra como um path
  minúsculo (medi uma de 0,05 unidade). É **geometria de verdade** (a ponta da estrela que sai
  do retângulo), e o Illustrator faz o mesmo — não filtrei. Se o Enio achar ruído, o lugar de
  descartar é o `commit`, com um piso de área, e **precisa ser decisão dele**: descartar
  geometria em silêncio é pior que uma lasca visível.
- **`build_smoke.rs` fica** (é o "exemplo pronto pra smoke"). Se um dia a autoria real o tornar
  obsoleto, aposente-o como o `timeline_smoke.rs` foi aposentado.
- **O `Cargo.lock`** tinha um `ph2d-tokens` fantasma em `ph2d-vec-boolean` (o manifesto nunca
  teve a dep); o cargo corrigiu no primeiro build. Está no commit.

---

## §7 — Onde o código mora

| | |
|---|---|
| A regra pura (o que morre e o que fica) | `shells/desktop/src/shape_build.rs` — `resolve()` + `commit()` + `source_key()` |
| A ponte com o frame | `shells/desktop/src/shape_build_gesture.rs` — `build_session_upkeep/down/move/up/cancel` |
| O arranjo (as faces) | `crates/ph2d-vec-boolean/src/arrangement.rs` — **não jogue fora e não construa um DCEL**: o realce e o resultado saírem do MESMO motor é a única razão de não poderem divergir |
| O realce | `crates/ph2d-vec-render/src/build_faces.rs` — `edge_strokes()` é onde mora "o ponto sobe pelo afim, a espessura não" |
| Os gates | `shells/desktop/src/shape_build_tests.rs` (a cena do artista) · `crates/ph2d-vec-boolean/tests/arrangement_product_shapes.rs` (o arranjo, com formas do produto) · `build_faces.rs::tests` (o zoom) |
| A cena de smoke | `shells/desktop/src/build_smoke.rs` (`PH2D_BUILD_SMOKE`) |
| A medição | `crates/ph2d-vec-boolean/tests/measure_face_hit.rs` — 19,9 µs/hover com 8 formas de 64 verts. **Não otimize sem medir de novo.** |
