# Handoff — linha `line/FLIP`, continuação (2026-07-18) · **COMECE AQUI**

> **Para o próximo agente-de-linha do Flip** (o 4º meio do PH2D: animação quadro-a-quadro,
> fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **Regime:** Modo L (workstation), worktree `Worktrees/line-FLIP`, branch `line/FLIP`.
> **Você NÃO integra nem pusha** (§0.7 do CLAUDE.md) — fecha o bloco, escreve o handoff, PARA.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e
> releia a cada passo) → **este arquivo** → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) (as sagas)
> → a rodada anterior [`…2026-07-17`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-17.md) (o detalhe
> de cada fatia do §4.C — este é o delta).
>
> **Sua tarefa: a wave COLORIZE** (§3 abaixo).

---

## 1. Estado — **§4.C INTEGRADO; a linha está SINCRONIZADA com a main**

O §4.C inteiro (6 fatias) entrou na `main` em 2026-07-18, com smoke aprovado pelo Enio em
todas. A branch foi fast-forwardada para a main integrada: **`ahead: 0 · behind: 0`**.

O que entrou (detalhe em [`…2026-07-17`](HANDOFF_line_FLIP_CONTINUACAO_2026-07-17.md) §1):
§4.C.1 halo por-peça + hover do Segment · §4.C.2 Duplicate Layer · §4.C.3 Rename Layer ·
§4.C.4 link de raio/força da borracha · §4.C.5 borracha macia idempotente + cena de boot
vazia · §4.C.6 **o Size mede o MUNDO** + Strength Soft-only + bump de schema.

**Verificado nesta base integrada:** `cargo check --workspace` limpo · flip 101 · panel seam
20 · tool 14 · shell 0 falhas · node_id_collisions · wiring parity · no_magic · LOC caps
(crate e shell) — **tudo verde**.

### 1.1 Schemas na base integrada (conte contra ESTES, não contra os do handoff velho)

| | valor |
|---|---|
| `PROJECT_SCHEMA` | **18** |
| `FLIP_SCHEMA_VERSION` | **8** |
| `VEC_SCENE_SCHEMA_VERSION` | **8** |
| pin em `project_tests.rs` | **`(18, 8, 8)`** |

⚠️ O `PROJECT_SCHEMA` saiu de 15 e o Flip pediu 16 — **entrou como 18** porque a linha de
física (ADR-0131 W1/W2: `RigidBody`/`Collider` no `ComponentRegistry`, depois
`restitution`/`friction`) bumpou na mesma jornada. É a regra
[[feedback_numbers_that_sum_across_lines_count_dont_pick]] funcionando: **o valor certo não
estava em nenhum dos dois lados — ele se contou.** Se a sua rodada bumpar, conte de novo
contra a main **do dia**.

### 1.2 O que as linhas IRMÃS trouxeram (contexto, não tarefa sua)

- **Timeline (8 commits)** — todos do **Arrange/strips** da composição de clips (gizmos por
  quina, fade que alcança o vão, loop por-vista). ⚠️ **A timeline segue em desenvolvimento
  ativo**, então a **W6 do Flip continua adiada** com razão (ver §4).
- **Vector (12)** · **GPU nodes (10, com handoff de integração próprio)** · **Painter/sculpt**
  · **anim/ui**.

### 1.3 ⚠️ Lição de processo desta rodada (custou trabalho ao integrador)

O topo da main traz um commit **`style: cargo fmt --all (drift da line/FLIP)`** — ou seja,
**esta linha entregou drift de formatação** e o integrador teve de limpar. O implementador
usa `--no-verify` no fast mode (correto), mas **rode `rustfmt` nos SEUS arquivos antes de
fechar o bloco** — nunca `cargo fmt -p`, que reformata WIP alheio
([[feedback_cargo_fmt_p_reformats_foreign_wip]]). E lembre que **o fmt RE-EXPANDE**: rode-o
*antes* de medir LOC, senão um arquivo passa o teto só depois
([[feedback_loc_cap_split_not_allowlist_and_fmt_reexpands]]).

---

## 2. Onde o módulo está (para você não reconstruir o que existe)

**Todas as waves do plano estão FECHADAS** — WT (traço), W0–W5 (frames/ghost/tween, fill,
reshape), W7 (multiframe) — **menos a W6**. **Nenhum bug aberto** em `BUGS_flip.md`.

Ergonomia de camadas completa: Add · **Duplicate** · **Rename** (double-click) · Delete ·
reorder ↑↓ · blend · opacity · lock · visibility.
Modos: Select · Draw · Erase (Soft/Hard/Stroke, com **link** de raio/força) · Fill · Sculpt
(8 pincéis) · Edit (domínios Stroke/Point/**Segment**).

**A lei nova do §4.C.6, que vale para tudo que você escrever:** o **Size mede o MUNDO**
(`ph2d_tool_flip::size_to_world`, `SIZE_PX_PER_WORLD = 100`) — porta ÚNICA, usada por traço,
Edit, borracha e sculpt; o render projeta (`thickness_px = raio_mundo · px_per_world`) e o
anel do cursor faz o caminho de volta. **Posições e larguras finalmente na mesma unidade.**
Se você precisar de uma medida de tela, ela é a exceção e tem de se justificar (o único caso
vivo é o *delta* do arrasto do Reshape).

---

## 3. ► SUA TAREFA: a wave **COLORIZE**

Escolhida pelo Enio (2026-07-18). É a única entrada do backlog qualificado que o plano chama
de **"wave própria"**, e a pesquisa **já está paga** — não re-pesquise, leia
[`Flip/04_alem_do_blender.md` §3](Flip/04_alem_do_blender.md), que traz as constantes.

**Por que ela:** ataca o maior custo do frame-by-frame (colorir quadro a quadro); é
auto-contida no Flip (nenhum contrato de outro módulo); e constrói sobre o solver de fill do
W4 e a vetorização (marching-squares → RDP → **fit Schneider**, que o PH2D já tem).

**A peça que justifica a wave — o *onion fill*:** um scribble atravessando várias poses
empilhadas pinta **o range de quadros inteiro**. O plano registra: *"a feature de flipbook
mais valiosa da literatura (só o TVPaint entrega hoje)"*.

### Fatiamento proposto (uma fatia por smoke, como no §4.C)

| Fatia | O quê | Referência |
|---|---|---|
| **C1** | **Trapped-ball** — pré-segmentação do line-art com gaps (flood → erode raio R → dilate; best-first com raios decrescentes, **R₀ = 8 px**). É o "colorir tudo" em lote. | Zhang et al., TVCG 2009 (`04 §3`) |
| **C2** | **LazyBrush** num quadro — colorização por scribbles via multiway cut; a fronteira é ATRAÍDA pro pixel mais escuro ⇒ a cor entra por baixo da linha com AA de graça, e **gaps nem precisam fechar**. | Sýkora et al., EG 2009 (`04 §3`) |
| **C3** | **Onion fill** — o scribble atravessa o range de quadros. É a fatia que ninguém mais tem. | idem |
| **C4** | A **UI**: ferramenta de scribble, paleta, e a semântica do *Update* (o Krita admite na doc que o solver é lento — decida se o nosso é síncrono ou usa o `progress` do `editor-core`). | `04 §3` |

**Constantes já cravadas** (`04 §3`, não re-derive): `K = 2(w+h)` · scribble soft **λ = 0,95**
· pré-filtro **LoG** p/ lápis · **guloso um-contra-todos ≈ 9–18× mais rápido que
α-expansion, com ΔE ≤ 0,04%** (é este que se implementa, não o α-expansion).

**Decisões que o plano JÁ tomou (não re-litigue):** o fill é **raster-then-vectorize** (fill
analítico direto no vetor esbarra em patente e é frágil com pontas abertas — o caso NORMAL);
**NÃO fazer fill em GPU** (JFA é o primitivo errado: salta paredes, não é geodésico; o fill é
operação de CLIQUE, não de frame).

**Antes de escrever código:** leia `04 §3` inteiro + [`06_fill_balde.md`](Flip/06_fill_balde.md)
(o que o W4 já resolveu e como) e escreva o plano da wave em `docs/Flip/` — as waves
anteriores todas têm o seu, e é ele que impede a fatia 3 redescobrir a decisão da fatia 1.

### 3.1 O que o W4 já te dá em `flip_fill.rs` — e onde ficam as costuras

> Escrito pelo agente do §4.C, que acabou de mexer neste arquivo. É o que **não** salta aos
> olhos lendo frio, e é exatamente o que o Colorize vai encostar.

- **`boundaries(drawing) -> Vec<(pts, half, closed)>`** é a porta que converte o documento no
  que o solver entende. ⚠️ **O `half` agora é MUNDO** — o `× px_to_world` que morava ali
  SUMIU no §4.C.6 (posições e larguras passaram a falar a mesma língua). Se você precisar de
  meia-espessura, é `w * 0.5` e ponto.
- **Quem é fronteira, e quem não é** — a distinção mora em dois predicados e ela não é óbvia:
  um **fill anterior** (`hide_stroke` **+** `fill.is_some()`) **NÃO** barra (senão a 2ª cor
  não entraria por baixo da 1ª); um **fechamento de gap** (`hide_stroke`, **sem** `fill`)
  **É** fronteira — é para isso que ele existe. Os dois são `hide_stroke`; o que os separa é
  a COR.
- **Fechamentos de gap são traços invisíveis PERSISTENTES** (o twist do Harmony): entram no
  desenho como qualquer outro traço, com largura ~0. Então o vão fica fechado para sempre —
  re-preencher com outra cor, preencher o quadro vizinho ou reabrir amanhã não dependem do
  estado da tool. O Colorize herda isso de graça.
- **A âncora do solver é o EIXO da linha, nunca a silhueta** (BUGS #14). Foi a cura de um bug
  em que a cor transbordava `(w/2)·(zoom−1)` px ao aproximar a câmera depois do clique. Não
  re-ancore na silhueta.
- **O contorno do fill é a DILATAÇÃO da cor por baixo do line-art**, não um contorno de
  verdade (`hide_stroke` fica ligado): sem dilatar, a metade externa da linha fica sem cor
  por baixo e um pincel macio ganha halo escuro. A margem `FILL_TUCK_PX = 0.5` **saiu de uma
  varredura no pixel**, não do olho (tabela no doc-comment) — 0 vaza fundo, ≥1,5 transborda.
- **O fill entra ATRÁS** (índice 0 da lista de traços); em `PaintBehind`, por baixo também
  dos fills que já existem.
- **`mean_line_width`** dá a espessura média do line-art (ignora regiões e fechamentos) — é
  ela que veste o contorno do fill.
- **O autokey do Flip é por-tool:** o balde usa a política `Modify` (`flip_autokey`), que no
  rabo de um hold trabalha numa DUPLICATA do desenho na tela — nunca num quadro em branco. O
  Colorize deve entrar pela MESMA porta, senão colorir no meio de um hold pinta o nada.
- **Para o onion fill (C3), o osso é este:** o solver é por-DESENHO e já roda por-desenho; o
  que falta é o wiring do RANGE (a multi-seleção de chaves na tira existe desde o W7 e é o
  canal natural). Está listado como carry-over do W4 — *"fill multiframe"*.

---

## 4. O que continua ADIADO (e por quê)

- **W6 — integração com a timeline global.** Adiada por ordem do Enio *"até a timeline
  principal fechar"*, e **a evidência desta jornada é que ela não fechou**: 8 commits novos,
  todos do Arrange/strips. Além disso exige coordenar com o dono dela (`PropKind` é enum
  **fechado**) e tem uma decisão de UX que é do Enio (T6.3: reconciliar os **dois** toggles
  de autokey homônimos). **Só com ordem explícita dele.**
- **Carry-overs conscientes** (spec pronta, ~1 unidade cada): drag de célula/borda na tira
  (mover chave e esticar hold — hoje só pelos botões ◀/▶ e a caixa Hold) · light table +
  Shift & Trace · instância de desenho na UI (o modelo já suporta; falta gesto + marcador) ·
  ajuste modal ao vivo do Gap Closure · modo Radius do Gap Closure.
- **Congelar o contrato do `ph2d-flip`** (gate de superfície) — o plano pede "quando o modelo
  assentar", e ele acabou de levar mudança real (`rename_layer`, `duplicate_layer`, e a
  UNIDADE do `width`). Candidato depois do Colorize.
- **Não-objetivos declarados** (não perguntar de novo): VFX do GP · modifiers/geometry-nodes ·
  lineart · rig/armature · import-export SVG/PDF · trace de imagem · viewport 3D.

---

## 5. Comandos

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP

# inner loop
cargo check -p ph2d-flip -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-host-desktop

# fechamento do bloco (1× sobre o diff acumulado)
rustfmt <os SEUS arquivos>        # ANTES de medir LOC — o fmt re-expande
cargo test -p ph2d-flip -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-host-desktop
cargo test -p ph2d-editor-core --test node_id_collisions \
  --test architecture_panel_wiring_parity --test no_magic_numeric \
  --test architecture_panel_loc_cap --test architecture_workspace_file_loc_cap
cargo test -p ph2d-host-desktop --test file_loc_caps
cargo clippy -p <suas crates> --all-targets
typos

# smoke (a cena de demo do Flip)
cargo build --release -p ph2d-host-desktop
PH2D_FLIP_DEMO=1 ./target/release/ph2d-host-desktop
```

**LOC a vigiar:** `flip_select.rs` é o mais apertado do módulo — campo novo ali → orce o
split em módulo irmão (`flip_select_pick.rs` / `_points.rs` / `_segment.rs` já são os irmãos;
o `flip_erase.rs` e o `paint_sections.rs` ganharam os seus nesta rodada).
