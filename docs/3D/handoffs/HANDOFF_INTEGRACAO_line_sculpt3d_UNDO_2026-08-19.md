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

## §5 — ✅ O VERMELHO PRÉ-EXISTENTE, CURADO — e o produto nunca esteve errado

> **Resolvido.** O gate voltou a **0,0020 / 0,0049** — o número EXATO que o autor
> mediu em 2026-08-03. A suíte `sculpt3d` inteira: **92 passaram, 0 falharam**.

### A causa, numa frase

A sonda tirava a vista de **`Shade::default()`**, e em 2026-08-09 o
`DEFAULT_MATCAP` passou a **`Some(0)`** por **decisão de produto** (o barro abre
aceso pela luz do OLHO, como no SculptGL — pedido do Enio). ⇒ Ela passou a
comparar **matcap contra rig**: duas leis diferentes, e a diferença entre elas
não é um defeito.

⚠️ **O doc do próprio sítio da chamada já dizia isto**, palavra por palavra:
*"o matcap é outra luz inteira — qualquer um dos dois ligado aqui faria a
comparação medir a diferença entre dois MODELOS em vez da diferença entre duas
implementações do mesmo."* A frase estava certa e ninguém a reconferiu quando o
default se mexeu.

⇒ É o `CLAUDE.md` §0.0 na direção inversa: **quem move o número que sustentava
uma nota tem de reconferir a nota** — e um *default compartilhado* é exatamente o
número que ninguém sabe que sustenta alguma coisa.

### A cura é ESTRUTURAL, não um ajuste

A vista da sonda passa a ser escrita **por nome**, `shared_law_shade()`, com os
**sete** campos da [`Shade`] listados e cada zero com o motivo ao lado.

⚠️ **Nada de `..Default::default()`, e não é verbosidade:** com o literal
completo, **um campo NOVO na `Shade` é um erro de COMPILAÇÃO ali** — quem o
acrescentar é obrigado a dizer se ele pertence à lei partilhada ou só ao barro.
Com o `..` o próximo termo entraria mudo, que é precisamente como este entrou.

⚠️ **Zero linhas de produto mudaram** — o módulo é `#[cfg(test)]`
(`sculpt3d_bake.rs:224`).

### ⛔ QUATRO hipóteses REFUTADAS pelo caminho — não as repita

| hipótese | como morreu |
|---|---|
| **W16, o ambiente direcional** | `DEFAULT_ENV = 0.0` — desligado por default; com `env = 0` os dois lados usam o mesmo piso plano |
| **SSAO** (`DEFAULT_SSAO_STRENGTH = 1.0`, e só o barro o tem) | desligado na corrida viva ⇒ números **idênticos**, dígito a dígito (a textura não medida devolve 0 = *"nada oclui"*) |
| **SSS** | `strength = 0` e `sss_diffuse` degenera em `max(n·l, 0)` **exactamente** |
| **a forma não chegar ao passe** (`has_form == 0`) | sonda: a normal do aro chega com `w = 1.0`; os dois structs do uniform batem campo a campo; e **os dois lados chamam `canvas_normal(in.n_view)`** |

⚠️ **A medição que fechou o caso** foi um `return` plantado no shader do barro
que **não executava** — porque o fluxo saía antes, pelo ramo do matcap. *Um
`return` que não muda a saída é a prova de que a leitura do fluxo estava errada*,
e foi mais barato que as quatro leituras que o precederam.

---

## §5-ter — Notas do antigo §5 (histórico do diagnóstico)

```
sculpt3d::bake::light_measure::the_two_lights_agree_where_the_form_turns_away
  → "o aro divergiu no balde 0: media 0.3370 (medido 0,0020)"     ⇐ barra: 0,01
```

**Reproduzido na árvore PRIMÁRIA, em `main` limpo (`ee1432203`), sem uma linha
deste diff.** Ele é `#[ignore]` (precisa de adapter) ⇒ **o CI nunca o roda**.

**A janela:** o gate nasce em `13f0a5e35` (**2026-08-03**) medindo **0,0020**;
entre 05/08 e 10/08 os dois lados da comparação ganham 734 linhas (barro) e 71
(tinta).

### ⛔ QUATRO causas REFUTADAS por medição — não as repita

| hipótese | como morreu |
|---|---|
| **W16, o ambiente direcional** (`232de15c7`) | `DEFAULT_ENV = 0.0` — o termo está **desligado por default**, e o doc dele diz por quê (*"a adoção pela tinta é o follow-up"*). Com `env = 0` os dois lados usam o mesmo piso plano |
| **SSAO** (`DEFAULT_SSAO_STRENGTH = 1.0`, e só o barro o tem) | desligado na corrida viva ⇒ os números saíram **idênticos**, dígito a dígito |
| **SSS** | `strength = 0` por default, e `sss_diffuse` degenera em `max(n·l, 0)` **exactamente** |
| **a forma não chegar ao passe** (`has_form == 0`) | sonda: a normal do aro chega com `w = 1.0` e o valor certo; os dois structs do uniform batem campo a campo |

### O que a MEDIÇÃO diz (sonda em texels do aro, `depth == 0`)

```
normal da forma (-0.058, -0.972, 0.227)   lampada (-0.557, -0.663, 0.500)
  ⇒ n·l = 0,790  — este pedaço do aro esta' ACESO, nao virado de costas
vivo   0.6104   (= o esperado para n·l alto: correto)
assado 1.0000   (SATURADO)
```

| dist | 0 | 1 | 2 | 3 | 4-7 | 8-15 | 16-31 | 32+ |
|---|---|---|---|---|---|---|---|---|
| vivo | 0,2447 | 0,2502 | 0,2536 | 0,2564 | 0,2655 | 0,2846 | 0,3162 | 0,4060 |
| assado | 0,5385 | 0,5461 | 0,5508 | 0,5545 | 0,5614 | 0,5732 | 0,5910 | 0,6506 |

⭐ **O barro está CERTO e o assado está claro demais**, e a aritmética o prende:
no aro `vivo/albedo = 0,346 ≈ AMBIENT (0,35)` — a resposta exacta de uma face
virada de costas. O assado dá **0,7615** do albedo.

Os dois shaders escrevem a **mesma** lei — `m = piso + (1−piso)·clamp(diffuse/flat, 0, 2)`,
com `flat = Σ tint·max(l.dir.z, 0) = 0,500` (`mesh.wgsl:861-891` ·
`impasto_light.wgsl:173-177`). Com `n·l = 0,790` isso dá `ratio = 1,58` e
`m = 1,377` — que é o que a TINTA calcula (e por isso satura). O BARRO chega a
`m ≈ 0,82`.

⇒ **Mesma fórmula, entradas que deviam ser as mesmas, resultados que diferem por
~1,6×.** O próximo passo é UM: imprimir o `nc` que o barro usa para o mesmo texel
e compará-lo com a normal da forma — se os dois viverem em espaços diferentes
(mundo vs vista/canvas), a causa está achada. Isso precisa de saída de debug do
lado do shader, que é apparato próprio.

⚠️ **Curar exige decidir qual das duas luzes é a certa** — e isso muda **todo
sprite já assado**. É pergunta de produto, não de conserto.

⛔ **Não baixe a barra do gate.** Ela foi medida (0,0020 / pico 0,0049) com três
mutações registadas no doc-comment dele.

---

## §5-bis — O que mais entrou nesta linha (ordem do Enio: *"corrija tudo"*)

**(a) O ALPHA vale para o FILTRO** (report: *"filter com alpha ligado desconsidera
alpha"*). O `filter` recebia o `brush` e nunca lhe perguntava pelo alpha. A cura
não inventa semântica: um [`Alpha`] é um **campo avaliado num ponto**, e ele
nunca precisou de um dab — ele entra no **mesmo produto da máscara**, na mesma
posição da ordem da referência, avaliado na pose **congelada**. Dois sítios,
porque o `Sharpen` bifurca do laço genérico.

⚠️ **Consequências honestas, nomeadas:** o alpha é um campo **infinito** (a
imagem de carimbo faz `wrap`), então num gesto de malha inteira ele **ladrilha** o
modelo — num dab isso nunca aparece porque o falloff limita a pegada. E um
carimbo é projetado em **tela**, então ele alcança a frente e o verso igualmente.
As duas são propriedades do **alpha**, não do filtro, e são o modelo do estêncil
do ZBrush. *Se o Enio quiser um carimbo com borda, isso é uma opção do alpha.*

Gates: `the_alpha_scales_every_law_of_the_generic_loop` (8 leis × 2 alphas,
RED-FIRST) · `the_sharpen_honours_the_alpha_too` (RED-FIRST) ·
`an_unarmed_alpha_moves_no_bit_of_the_filter` (verde antes **e** depois). ⚠️ **O
oráculo é a LINEARIDADE das leis**, não a fórmula: duas corridas idênticas — uma
com alpha, outra sem — e o quociente entre elas tem de ser o peso do alpha
naquele ponto. Nenhuma constante escrita no teste.

**(b) O `nextest-impacted.sh`** — §6-bis, corrigido. O seletor passou a derivar o
pacote do `cargo metadata`. Conferido no diff desta linha: **de 4 testes para
3 774**.

**(c) O QUAD REMESH** — [ADR-0160](../../architecture/decisions/0160-quad-remesh-is-a-native-cross-field-port-quadriflow-referenced.md)
(pesquisa, três famílias, alternativas recusadas com o motivo, conjunto de
aceitação e kill-criterion congelados) + **Q1, Q2 e Q3** na crate-folha nova
`ph2d-quadflow`: campo de orientação 4-RoSy · campo de posição + escala
adaptativa · **extração**. **18 gates verdes**, 3 alvos com o número medido.

| | esfera 48×64 | toro 64×32 |
|---|---|---|
| **quads** (das faces emitidas) | **664 / 1 246 = 53,3 %** | **454 / 1 144 = 39,7 %** |
| χ (alvo) | 5 (2) | 2 (0) |
| Hausdorff (barra 1 %) | 3,14 % | — |

⚠️ **A RÉGUA mentiu e corrigiu-se antes do algoritmo.** A `quad_fraction` media
`quads / (quads + CICLOS não-quad)` — um ciclo de **31 lados** contava como UM
não-quad enquanto virava **29 triângulos**. Ela *melhorava* quando as falhas
ficavam maiores: uma tentativa que trocou 582 triângulos por 918 marcou
**60,9 % → 71,9 %** e era, honestamente, **53,3 % → 35,0 %**. Todos os números
deste handoff são os re-medidos.

⚠️ **SETE recusas MEDIDAS, cada uma com o número no sítio** (ADR §5-bis): o
`union-find` transitivo (4 células) · as arestas da entrada (7,2 %) · o
coarsening por média · a **hierarquia** no produto (38,9 % vs 53,3 %) · o
quociente pela retícula (1 célula) · a escolha **mútua** (35,0 %) · a semeadura
de **Poisson** (48,9 %).

⚠️ **A2/A3/A4 NÃO estão verdes**, e os três gates ficam com a **barra do §4
intacta** + `#[ignore]` carregando o número medido. ⛔ **Não os afrouxe:** eles
são a definição de pronto, e afrouxá-los trocaria o alvo pela medição de hoje.

⚠️ **Quatro caminhos morreram MEDIDOS nestas ondas** — a régua euclidiana no
lugar da decomposição por eixo · *"vizinhos partilham a mesma retícula"* (0,205
célula, imóvel entre 32 e 2 048 varreduras) · o `union-find` transitivo (a esfera
de 3 072 vértices em **4 células**) · as arestas da entrada como arestas de saída
(**7,2 %** de quads, porque a entrada é uma triangulação).

⚠️ **A Q3 refutou o plano do ADR, e a Q3.5 refutou a refutação.** A Q3 concluiu
que a hierarquia era pré-requisito; ela foi **construída, gateada e medida** — e
perde do caminho plano em **24/24** combinações. A conclusão da Q3 supunha que a
extração usava a retícula, e ela **não usa**: o crescimento por semente lê o
campo como uma DISTÂNCIA. *A hipótese não era sobre a hierarquia — era sobre a
extração, e a Q3.5 testou a metade errada.*

⇒ **O lever da Q4 é a EXTRAÇÃO.** A hierarquia fica no repo, correta e gateada,
**fora do caminho do produto**, com um gate que impede ligá-la sem re-medir. O
passo concreto que resta é o porte da extração REAL do QuadriFlow
(`extract_graph`/`extract_faces` + o fluxo de custo mínimo) — trabalho grande e
cuidadoso, e é ele que move A2/A3/A4.

⚠️ **Nada é alcançável pelo produto ainda** — dívida DECLARADA no doc-comment da
crate e no ADR §5.

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
