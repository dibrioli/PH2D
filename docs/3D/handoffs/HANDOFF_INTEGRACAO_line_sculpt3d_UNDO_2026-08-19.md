# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d` · o UNDO do filtro (2026-08-19)

> **Branch:** `line/sculpt3d` · **base:** `main` em `ee1432203` · árvore própria
> em `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d`.
> **Escopo:** a ⭐ TAREFA 1 do
> [handoff de continuação](HANDOFF_CONTINUACAO_line_sculpt3d_2026-08-19.md) —
> *"não temos undo para Filter"* (Enio, 2026-08-18). **Fechada.**
>
> **Foundational tocado:** NENHUM. As cinco edições vivem todas em
> `shells/desktop/src/sculpt3d_*` (o shell da própria linha). `ph2d-sculpt3d`,
> `ph2d-mesh` e `ph2d-light` estão **byte-idênticos** ao `main`.
> **Contrato congelado encostado:** nenhum. **Id/const/variant novo:** nenhum
> (só um `const CENTRE` local a um arquivo de teste novo).

---

## §1 — A CAUSA, e por que ela é a TERCEIRA do mesmo mecanismo

O `close_stroke` decidia o canal da entrada de undo perguntando ao **VERBO em
mãos**:

```rust
masks: self.brush.verb.paints_mask().then(|| self.stroke.base_masks().to_vec()),
```

E `paints_mask()` é `matches!(self, Self::Mask)`. ⇒ **com o `Verb::Mask` em mãos,
um gesto de FILTRO gravava a entrada como se fosse um gesto de máscara.** O
`Ctrl+Z` trocava o canal de máscara, a geometria ficava onde estava, e — porque
aquele ramo não chamava `rebuild()` nem `mesh_rebuilt()` — **a tela nem
atualizava**. Do lado do artista isso é exactamente *«não tem undo»*.

⚠️ **O picker da W9b desacoplou a LEI do VERBO**, e todo sítio que ainda inferia
*«que espécie de gesto foi este?»* a partir do verbo passou a estar errado. Os
três, em ordem: o `fill_hc_disp` (o pânico do Surface Smooth, curado) · a
mensagem de recusa do `begin_filter` (**curada aqui**, §3) · **este**.

*Uma condição que enumera os seus leitores apodrece no dia em que o segundo
nasce.*

---

## §2 — A CURA: dois sítios, e cada um é uma lei diferente

### (a) O registo passa a perguntar um FATO — `sculpt3d_history.rs`

`close_stroke` chama `mask_window_changed()`, que **compara a janela congelada
contra o plano vivo**:

```rust
fn mask_window_changed(&self) -> bool {
    let Some(live) = self.mesh().masks() else { return false };
    self.stroke.touched().iter().zip(self.stroke.base_masks())
        .any(|(&v, &m)| live[v as usize] != m)
}
```

⚠️ **O `capture` congela `base_mask` para TODO vértice tocado, seja qual for o
verbo** (`stroke_freeze.rs:34`) — é isso que torna a pergunta respondível como
fato. Sem plano vivo não há o que ter mudado (o congelado é o `DEFAULT_MASK`).

**Custo:** O(janela tocada) de comparação de `f32`, uma vez por gesto — contra a
`rebuild()` da octree que ela evita.

### (b) O desfazer trata DOIS CANAIS INDEPENDENTES — `sculpt3d_undo.rs`

O braço `StrokeUndo::Stroke` era um `if/else` (*ou* máscara *ou* geometria).
Virou:

1. troca as máscaras **se a entrada as tem** → `uploaded = false` + `edits += 1`
   (o par exacto que o braço irmão `StrokeUndo::Mask` já escrevia um degrau
   acima — não é lei nova);
2. troca as posições **sempre**;
3. `rebuild()` + `mesh_rebuilt()` **se e só se as posições de facto mudaram**
   (`positions_now != positions`).

⚠️ **O item 3 é a pergunta certa.** Pagar a octree inteira por um traço de
máscara seria cobrar o preço do canal errado; e amarrá-la à presença das
máscaras era o `if/else` de novo. **A representação apaga o caso especial:** um
gesto que mexesse nos dois canais desfazia METADE, em silêncio, e agora não há
onde essa pergunta more.

---

## §3 — O código MORTO que também mentia — `sculpt3d_input.rs`

O `else` do `begin_filter` no pen-down imprimia
*«o verbo em maos nao filtra a malha — escolha Smooth, Inflate, Slide Relax ou
Surface Smooth»*. Duas coisas erradas ao mesmo tempo:

- **inalcançável:** o ramo vive dentro de `if scene.filter_arm()`, e a única
  recusa do `begin_filter` é `!filter_arm()`;
- **obsoleto:** desde a W9b a lei do filtro vem do **picker**, não do verbo —
  a frase deixou de ser verdadeira sobre este app, e nunca teve como ser
  impressa para alguém a desmentir.

Virou `debug_assert!(false, …)` **com a rede de release de pé** (o gesto continua
a virar órbita em vez de um botão morto). ⚠️ O `begin_filter` **não** foi movido
para dentro do assert — ele tem efeito colateral (congela a foto da malha), e
`debug_assert!` não corre em release.

---

## §4 — OS GATES: red-first, e provados por mutação

Quatro gates novos. **Os dois primeiros foram vistos VERMELHOS antes da cura**;
os dois últimos nasceram depois dela e por isso foram provados por **mutação**.

| gate | arquivo | como foi provado |
|---|---|---|
| `the_filter_undoes_the_geometry_whatever_verb_is_in_hand` | `sculpt3d_filter_tests.rs` | ⭐ **RED-FIRST** — reprovou com a mensagem exacta do report |
| `undoing_a_filter_tells_the_screen` | `sculpt3d_filter_tests.rs` | **RED-FIRST** |
| `a_mask_stroke_undoes_and_tells_the_screen` | `sculpt3d_undo_tests.rs` (**novo**) | **M1** — tirar `uploaded=false`/`edits+=1` do braço da máscara ⇒ sangra |
| `a_geometry_stroke_leaves_the_mask_channel_alone` | `sculpt3d_undo_tests.rs` (**novo**) | **M3** — `mask_window_changed → true` ⇒ sangra |

⚠️ **O `sculpt3d_undo_tests.rs` nasce porque a família inteira do undo tinha UM
teste** — o `swap_window` solto, uma unidade pura no `mod tests` do
`sculpt3d_history.rs`. Os catorze braços que a cena aplica só eram tocados pelo
gate do filtro. Este arquivo é a casa deles.

⚠️ **O caminho de módulo é `sculpt3d::history::undo::tests`**, não
`sculpt3d::undo::tests` (o `undo` é filho do `history`). Um filtro com o nome
errado devolve `0 passed`, que **não é verde: é nada rodou**.

### O que a nota do handoff anterior errou (medido, não suposto)

Ela prescrevia *«o gate que falta é a rota do PONTEIRO»*. **Duas correções:**

1. **A rota do ponteiro NÃO é alcançável de um teste.** `sculpt3d_pointer_down/
   move/up` vivem no `App`, cujo `AppGfx` segura um `SurfaceContext` — uma
   surface wgpu presa a uma janela real (`app_state.rs:51`). Não há como
   construí-lo headless; `App::new()` sozinho deixa `gfx = None`, e
   `sculpt3d_scene_mut()` devolve `None`.
2. **O que a fixture não continha era o VERBO, não a porta de entrada.** O gate
   irmão `the_whole_drag_is_one_undo_step` já percorria o gesto inteiro — com o
   `Verb::Inflate` em mãos, que não pinta máscara. É por isso que ele ficava
   verde sobre o defeito.

Os gates novos dirigem **literalmente a mesma sequência** que aquelas três
funções executam (`aim` + `begin_filter`, `sculpt3d_input.rs:106-111` · `filter_at`
no move · `close_stroke` no `Drag::Filter` do up, `sculpt3d_input.rs:243-244`), e
os doc-comments dizem isso com o `file:line`, para ninguém alegar mais do que
está provado.

---

## §5 — ⚠️ UM VERMELHO PRÉ-EXISTENTE NO `main` (NÃO é desta linha)

```
sculpt3d::bake::light_measure::the_two_lights_agree_where_the_form_turns_away
  → "o aro divergiu no balde 0: media 0.3370 (medido 0,0020)"     ⇐ barra: 0,01
```

**Reproduzido na árvore PRIMÁRIA, em `main` limpo (`ee1432203`), sem uma linha
deste diff.** Ele é `#[ignore]` (precisa de adapter) ⇒ **o CI nunca o roda**.

**A janela, por leitura do log:**

| commit | data | o quê |
|---|---|---|
| `13f0a5e35` | **2026-08-03** | o gate NASCE, medindo **0,0020** |
| `232de15c7` | 2026-08-09 | **W16 — o ambiente ganha DIREÇÃO** (o piso da difusa deixa de ser um número para toda direção) |
| `bf560cb6b` | 2026-08-09 | a fixture do ambiente traz a própria luz |
| `89d4f7e11` | 2026-08-09 | a luz assada só é re-autorada por um GESTO |

⚠️ **A hipótese que a tabela sugere — e que NÃO foi confirmada por bisect:** o
gate mede o **ARO**, que é exactamente onde o ambiente domina (a lâmpada não
alcança). A W16 mudou o modelo do ambiente; se a direção chegou a **um** dos dois
caminhos de luz e não ao outro, o aro diverge muito enquanto o miolo continua a
concordar — que é a assinatura observada. **Confirmar custa um bisect de 3
builds release**; curar exige decidir **qual dos dois lados está certo**, que é
pergunta de produto/arquitetura, não de conserto.

⛔ **Não baixe a barra do gate.** Ela foi medida (0,0020 / pico 0,0049) com três
mutações registadas no doc-comment dele.

---

## §6 — O que NÃO foi feito (e continua na fila)

- ⏸️ **As duas decisões do Enio** sobre o `Sharpen` (§3 do handoff de
  continuação) — já devolvidas com a tabela, **não são trabalho**.
- **W10 (Cloth) · W11 (Handles) · W12 (Geodésica) · marching cubes** — a fila do
  [plano 21](../21_plano_modos_e_ferramentas.md) §7. ⚠️ *A primeira coisa de toda
  wave é MEDIR se a composição já exprime o item.*
- **Os 50 achados não verificados** da auditoria de 18/08 (run `wf_76127d6f-aa1`)
  seguem por ler.
- ⏸️ **O undo do filtro pela rota do PONTEIRO** sai da fila: §4 acima mostra que
  ela não é alcançável de um teste, e o que ela acrescentaria já está gateado.

---

## §6-bis — ⚠️ O `nextest-impacted.sh` NÃO VÊ `shells/`, `tools/` nem `tests/`

**Achado desta sessão, com reprodução.** O conjunto impactado sai de uma linha só
(`scripts/nextest-impacted.sh:29-30`):

```bash
CHANGED=$(git diff --name-only "${BASE}"... \
  | sed -n 's#^crates/\([^/]*\)/.*#\1#p' | sort -u)
```

⇒ **um diff inteiramente em `shells/desktop/src/` produz `CHANGED` VAZIO**, o
script cai no ramo *"no crate changes"* e roda **só o golden de determinismo (4
testes)**. Medido neste diff:

```
BASE=main bash scripts/nextest-impacted.sh
  → Starting 4 tests across 1 binary (1306 binaries skipped)
  → 4 passed   (ph2d-ecs::transform_determinism)
```

⚠️ **Isto é um gate verde por acidente, e o alvo não é pequeno:** `shells/desktop`
é onde vivem o shell inteiro do sculpt3d, o undo, a persistência e o
`input_dispatch`. Todo fechamento de linha cujo diff seja só de shell tem corrido
com esta cobertura.

⚠️ **O comentário do próprio script diz o contrário do que ele faz:** *"If a
changed dir name is not a real package, the filterset errors out — that surfaces
the dir→package mismatch rather than silently under-testing."* Isso vale para um
nome ERRADO dentro de `crates/`; um caminho FORA de `crates/` não gera nome
nenhum, e o silêncio é total.

**Não foi corrigido nesta linha, de propósito:** a cura (derivar o pacote do
`cargo metadata` em vez do prefixo do caminho) muda o que **toda** linha roda ao
fechar — há **5 worktrees vivas** —, e isso é decisão de processo, não um
detalhe que deva viajar dentro de um conserto de undo.

**A cobertura desta linha foi obtida à mão, e é maior que a do script:**

| corrida | resultado |
|---|---|
| `cargo test -p ph2d-host-desktop --release --bins` (a crate INTEIRA) | **2696 passaram · 0 falharam** · 189 ignorados |
| `… --bins sculpt3d -- --include-ignored` (os gates de GPU do módulo) | **91 passaram** · 1 falhou (o §5, pré-existente) |
| a mesma em **DEBUG** (precedente do módulo) | **91 passaram** · o mesmo 1 |
| `cargo clippy -p ph2d-host-desktop --all-targets` | limpo |
| `rustfmt --edition 2024 --check` (toolchain **1.95**, o pin) | limpo |
| `typos` nos arquivos tocados | limpo |

⚠️ O `debug_assert!(false, …)` do §3 **nunca disparou** na corrida de DEBUG — é a
prova executável de que aquele ramo é inalcançável, e não só um argumento.

---

## §7 — O gate de fechamento e o smoke

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=34 cargo run -p ph2d-host-desktop --release
```

O roteiro de 6 passos da W9 tem de aparecer no terminal — **se a lista não
aparecer, PARE**. O passo que este trabalho cura: **pegar o pincel Mask, ligar o
Filter, escolher uma lei, arrastar, e apertar Ctrl+Z** — a malha volta, e volta
**na tela**.

⚠️ **Rode também uma vez SEM a env var** — é a metade que prova a inércia (sem
ela o `AppGfx.sculpt3d` é `None` e o frame 2D é byte-idêntico).
