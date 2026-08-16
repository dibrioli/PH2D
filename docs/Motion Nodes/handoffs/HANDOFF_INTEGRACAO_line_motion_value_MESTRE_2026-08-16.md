# HANDOFF DE INTEGRAÇÃO — `line/motion-value` (2026-08-16)

> **Este documento SUPERSEDE**, como *o que integrar agora*, o
> [`HANDOFF_line_motion_value_TROCA_2026-08-10.md`](HANDOFF_line_motion_value_TROCA_2026-08-10.md)
> — ⚠️ *o detalhe de mecanismo de cada wave continua LÁ e nas entradas da §5 do
> `CLAUDE.md`, e não foi copiado para cá.* Este arquivo responde três perguntas e
> só elas: **onde isto colide**, **como se verifica a árvore combinada** e **o que
> o Enio ainda não julgou**.

---

## 1. O que landa, em uma frase

**A CONFERÊNCIA DOS NÓS fechou a fila de P0 e percorreu a segunda volta em
GRUPOS** — mais a cauda do W7 do plano 13 (o pulso, a morte, o colisor, o
substep). Medido: **162 commits · 345 arquivos · +50.127 / −5.078**.

Quatro blocos:

**(A) A cauda do plano 13 — o W7 fechou.** O `sim.spawn` nasce por **PULSO** (e
a porta `reset`, que tira o congelamento de estado ao mudar um param em runtime)
· o `sim.lifetime` **emite a MORTE** como evento · o colisor sabe o **TAMANHO**
do que ele colide (um ponto era o raio sempre errado) e o plano dele deixa de ser
horizontal (a forma carrega a própria orientação) · a colisão deixa de ser
invisível (o canal **`hit`**) · o `sim.step` ganha um **teto de velocidade** que
capa a DISTÂNCIA, não o número · e o **SUBSTEP por-zona**, que fecha o último P1
da folha 13 — ⚠️ ele é **ESCOPO**, não um número global, e o device volta a
marchar sob ele.

**(B) A fila de P0 da conferência ESVAZIOU.** As 17 folhas foram percorridas e a
fila acionável fechou: a família `force.*` herda o cluster de ruído · o
`look_at` honra `falloff`/`strength` (era a exceção do contrato de família) · o
blend do mixer vira **CAMPO** · o duplicator ganha a variante por ponto · a
fórmula enxerga a POSIÇÃO · o `spline_wrap` **SEGUE** a curva e ganha o trecho
From/To · o pivô da escala de layout · o `make_point` constrói a coluna que se
pede · o `rig.skeleton` vira **ÁRVORE** · a rampa do falloff vai a qualquer
ângulo · o modo de composição é do **SINK** · o `value.noise` ganha catálogo de
kernels · o `source.object` ganha **offset de tempo** · o **`source.text`** (uma
instância por glifo) · e o **`audio.bands`** — o som vira campo de valor, com a
FFT **fora do cook** por cerca estrutural.

**(C) A SEGUNDA VOLTA, em GRUPOS — uma cena por grupo** (ordem do Enio: *"se
continuarmos nesse ritmo nunca acabaremos"*): **A** a aritmética do domínio de
valor (`=41`) · **B** o ruído e o relógio (`=42`) · **C** as estatísticas (`=43`)
· **D** a tabela e a semente (`=44`) · **E** a comparação e o nome que não
resolve (`=45`) · **F** o envelope (`=46`) · **G** a velocidade (`=47`) · **H** o
disco que tem o tamanho que se vê (`=48`).

**(D) O que a MEDIÇÃO recusou, e fica escrito para ninguém reconstruir:** o
**Simplex** (a anisotropia que ele cura é menor que a diferença entre dois
vizinhos do próprio menu) · a **variância de um passo** (num campo constante de
magnitude `1e5` ela reporta desvio **71,6**) · o `iterations` do `smooth` (a
relação estava INVERTIDA: `N` box são uma B-spline, logo o **peso** é o
parâmetro geral) · a **faixa de barras** do `value.pattern` (construída, smokada,
**REVERTIDA inteira** por veredito do Enio — a árvore sobrevive em `ae35416bd`) ·
e a *idade normalizada*, que já era exprimível.

---

## 2. A superfície de COLISÃO — MEDIDA, não auto-relatada

| O que | Medido | Como |
|---|---|---|
| `PROJECT_SCHEMA` | **ZERO** — a linha **não toca** `project*.rs` | `git diff --stat main...HEAD -- shells/desktop/src/project*.rs` → **vazio** |
| Contrato congelado (nodes) | **3/3 verde** | `cargo test -p ph2d-nodegraph --test architecture_contract_surface` |
| Contrato congelado (tools) | **4/4 verde** | `cargo test -p ph2d-editor-core --test architecture_tool_contract_surface` |
| Registro do `ph2d-ecs` | **INTOCADO** ⇒ os **três** espelhos também | `git diff --stat main...HEAD -- crates/ph2d-ecs/` → vazio |
| `ph2d-i18n` | **INTOCADO** | idem |
| ADRs | **NENHUM** ⇒ fora de toda disputa de número | `git diff --name-status main...HEAD -- docs/architecture/decisions/` → vazio |
| Scrollbar ids | **nenhum novo** | grep no diff da `ph2d-editor-core` |
| Crates novas | **7**, todas folha/drop-in | `ph2d-arc-length` · `ph2d-fbm` · `ph2d-node-audio-bands` · `ph2d-node-motion-velocity` · `ph2d-node-pulse-adsr` · `ph2d-node-pulse-signal` · `ph2d-node-source-text` |
| Pacotes EXTERNOS novos | **ZERO** | os únicos `+name` do `Cargo.lock` são **as sete crates acima** |
| `Cargo.toml` tocados | **20** (as 7 novas + 13 arestas internas) | `git diff --name-only main...HEAD -- '*/Cargo.toml'` |
| Censo de params | **124 nós · 535 params · 516 com hint · 153 com unidade** | `cargo test -p ph2d-node-registry-init --test param_census -- --ignored` |

⚠️ **O censo acima é o desta ÁRVORE** (fork + linha). O valor da árvore
**combinada** tem de ser re-medido pelo integrador: se o `main` acrescentou
crates-nó nestes 390 commits, o número sobe, e é o número combinado que vai para
a §5.

### 2.1 O `PROJECT_SCHEMA`, e por que esta linha está fora da disputa

O `main` de hoje diz **84**, e ele **PARTIU o `project.rs`** — a constante e a
escada mudaram-se para o irmão **`project_schema.rs`** (a `line/physics` de
2026-08-15). Esta linha **não toca nenhum dos dois**, então:

- não há degrau a CONTAR;
- não há a colisão MUDA que este repo já pagou quatro vezes (duas linhas
  escrevendo o **mesmo literal**, com o git sem opinião sobre o que o número
  significa);
- e o corte do `project.rs` **não a alcança**.

---

## 3. O REBASE — a interseção é de TRÊS arquivos

⚠️ **O `main` está 390 commits à frente do fork.** Isso **não** é a tabela de
colisão (o `main...HEAD` de três pontos é merge-base-relativo, logo imune) — é o
que se espera do rebase. Medido, a interseção *arquivos da linha ∩ arquivos que o
`main` moveu desde o fork* é:

```
Cargo.lock
CLAUDE.md
shells/desktop/src/render_loop/mod.rs
```

- **`Cargo.lock`** — os dois lados só ACRESCENTAM pacotes; resolva pelos estágios
  e re-gere se preciso.
- **`CLAUDE.md`** — as duas metades são entradas de §5 de módulos **diferentes**;
  ⚠️ resolver *"fique com um lado"* apaga a entrada do outro. Conferir a FRASE,
  não o arquivo.
- **`render_loop/mod.rs`** — a linha soma **+41 linhas, todas ACRÉSCIMO** (zero
  remoções); o `main` somou **+351 / −16**. É acréscimo contra acréscimo.

⚠️ **E nenhum arquivo que a linha edita foi APAGADO no `main`** (varrido com
`--diff-filter=M` NUL-safe). O `motion_state_demo_router.rs` é **novo desta
linha** (extração por teto de LOC do `match` que no `main` ainda vive dentro do
`motion_state.rs`) — e o `main` **não tocou** o `motion_state.rs` desde o fork,
então a extração funde limpa.

### 3.1 Os números de cena de smoke — sem colisão, e o mecanismo protege

O `main` reclama `PH2D_GPU_COOK_DEMO` **1..23**; a linha reclama **1..48**.
Nenhuma colisão. ⚠️ **E aqui a colisão não pode passar em silêncio**, ao
contrário do `build_smoke_router.rs`: aquele é uma cadeia de `if level == N`
onde o **primeiro vence** (o defeito que a `line/Vector` pagou em 02/08 e que o
gate `no_two_smoke_scenes_claim_the_same_level` fecha), e **este é um `match`
sobre literais** — um número duplicado é `unreachable pattern`, ou seja um aviso
do compilador, nunca uma cena inalcançável em silêncio.

---

## 4. Como verificar a árvore combinada

Rodado **nesta** árvore, todos verdes:

```
cargo check --workspace --all-targets            EXIT 0
cargo test -p ph2d-nodegraph          --release    123 passed, 0 failed
cargo test -p ph2d-gpu-cook           --release     87 passed, 0 failed
cargo test -p ph2d-node-registry-init --release     75 passed, 0 failed
cargo test -p ph2d-editor-core        --release  1.013 passed, 0 failed
cargo test -p ph2d-host-desktop       --release  2.969 passed, 0 failed
```

⚠️ **Depois do rebase, rode a suíte INTEIRA e não a varredura impactada.** Os
gates de `shells/desktop/tests/` e de `ph2d-editor-core/tests/` **só correm
quando aquela crate é impactada**, e esta linha já pagou essa lição três vezes:
um vermelho-latente de LOC, um `no_tofu_glyphs` e um `PANEL_A11Y_DELEGATE_OK` só
apareceram na árvore combinada.

⚠️ **Os gates de GPU são `#[ignore]` e exigem adapter** — sem ele fazem *skip
gracioso*, **que não é verde**:

```
cargo test -p ph2d-gpu-cook --release -- --ignored --test-threads=1
```

Medido nesta árvore, na RTX: **95 passed, 0 failed**.

⚠️ **E nenhuma leitura de relógio desta máquina significa coisa nenhuma com o
`load average` acima de ~5** (precedente medido: o mesmo binário mediu 11,36 e
5,50 ms para o mesmo passe sob cargas diferentes).

---

## 5. Smokes — o que o Enio já julgou, e o que não

| Cena | Grupo / wave | Status |
|---|---|---|
| `=24`..`=31` | a cauda do plano 13 (pulso · sinal · morte · raio · rampa · hit · teto) | aprovadas à medida que fecharam |
| `=32`..`=35` | as quatro cenas da conferência (fita · pivô · fórmula · mira parcial) | aprovadas (duas foram REPROVADAS e reescritas antes) |
| `=36`..`=40` | sink blend · kernels de ruído · direção · texto · som | **smoke OK** (as entradas da §5 trazem a data) |
| `=41` | **A** — a aritmética | **smoke OK** |
| `=42` | **B** — o ruído e o relógio | **smoke OK** |
| `=43` | **C** — as estatísticas | ⚠️ **PENDENTE** |
| `=44` | **D** — a tabela e a semente | ⚠️ **PENDENTE** |
| `=45` | **E** — a comparação | **smoke OK** |
| `=46` | **F** — o envelope | ⚠️ **PENDENTE** |
| `=47` | **G** — a velocidade | **smoke OK** |
| `=48` | **H** — o disco que se vê | **smoke OK** (2026-08-16, já com a correção do Output) |

Mais: **`PH2D_MOTION_OBJ_SMOKE=7`** (as DUAS FASES — o mesmo Flip trazido em dois
tempos), **smoke OK**, e as duas correções que ele produziu (o piscar por chave
de tile e o piscar por despejo).

⚠️ **Integrar não é aprovar:** `=43`, `=44` e `=46` **não foram smokadas**.

---

## 6. Armadilhas — leia antes de mexer

1. ⚠️ **`crates/ph2d-node-pulse-signal/src/tests.rs` está SUJO na worktree e não
   está em commit nenhum.** É um diff de **formatação pura de outro dono**,
   deixado deliberadamente fora de todos os 162 commits. **Não o stage.**
   Confirme com `git status --porcelain` antes de qualquer `git add`.

2. ⚠️ **Uma cena de demonstração sem `motion.output` desenha NADA.** O laço de
   render **não usa** a lista de sinks que o construtor devolve — ele a
   **re-resolve a cada quadro** dos nós de saída do grafo (`motion_bridge.rs`:
   `motion.sinks = output_nodes(&doc.graph)`). A cena `=48` nasceu assim: seis
   bandas terminadas no `motion.collide`, cozinhando certo, com os seis gates
   **verdes** e a tela vazia. Fechado por dois gates mutação-provados — o local
   (o `type_name` do sink) e a **rede da FAMÍLIA**
   (`tests/every_demo_scene_ends_in_an_output_node.rs`, que varre
   `motion_state_conferencia_demos*.rs` **por diretório**, com controle positivo
   de ≥ 10 cenas: *a cena que nascer amanhã entra sozinha*).

3. ⚠️ **Todo canal novo é side-metadata no REGISTRY, nunca o manifest.** Esta
   linha acrescentou canais (`Coupling::ProducesWhen`, o `node_key`, a
   identidade do nó no uniforme, o `reduces` do collide) e **nenhum** toca
   `NodeOp`/`OpResolver`/`NodeManifest` — o padrão de `param_gates`/`luts`.

4. ⚠️ **A lista de params do KERNEL não é derivada do manifesto.** Um param novo
   compila, coza na CPU e o **device recusa o shader**
   (`invalid field accessor`). É falha alta, não silêncio — mas o sítio se
   esquece, e agora está nomeado no próprio arquivo.

5. ⚠️ **O gerador do `ph2d-node-registry-init` foi RODADO** (commit
   `2b2bb7f3f`): a wave de áudio tinha editado **a saída dele à mão**, que é
   exatamente o que deixa o gate de staleness vermelho.

6. ⚠️ **Os `pre` self-loops são plumbados pelo EDITOR ao soltar um nó** — um
   documento montado por `add_node` não os ganha. Toda fixture que mede
   simulação precisa de `advance_tick`, senão a aresta `pre` nunca carrega
   estado e o gate fica **verde sobre uma cena morta** (a lição que as cenas
   `=38`, `=46` e `=47` pagaram, cada uma por sua vez).

---

## 7. Aberto — o que a próxima janela pega

- **A folha 03 (simulação) tem 15 P1**, incluindo o resto do `motion.collide`
  (modos **Push / Scale / Hide**, linha 61) — ⚠️ e a primeira coisa é **MEDIR se
  a composição já os exprime**: com o `size` a dirigir o raio, *Scale* pode ser
  `value.attribute → motion.drive(Size)` e *Hide* um `motion.cull`, o que
  dissolveria o `motion.push_apart` que o doc 63 propõe como nó novo.
- A coluna **`neighbours`** — o número **já é computado e jogado fora nas duas
  rotas** (`var neighbours` no WGSL, `inv_n` na CPU); emiti-la custa uma coluna
  na CPU **mais um harness de paridade que leia COLUNAS** (o de hoje compara
  `world_pos` de instâncias) ⇒ wave com cena própria, não rodapé.
- `verlet_rope`: sub-passos e pino de índice arbitrário · `soft_body`: forma
  inicial e peso de alvo por-partícula · `spring`: massa por-elemento e o
  `motion.lag` · `pin_constraint`: limiar de ruptura.
- A folha 07 tem 3 P1 (`motion.trail` eco para a frente · Echo Operator ·
  `motion.step` pareado por id).
- ⚠️ **O `motion.collide` tem espessura de corda FECHADA por medição** e o P1
  virou **P2** (ergonomia): o gesto existe (dois nós e duas arestas à mão, com o
  `pre` na aresta que ENTRA no colisor), o que falta é o atalho.
- ⚠️ **A faixa de barras do `value.pattern` foi REVERTIDA por veredito de
  produto e nenhum mecanismo foi nomeado** — uma segunda tentativa **começa
  perguntando o que ficou pior**, não reconstruindo: a árvore inteira sobrevive
  em `ae35416bd` (13 gates, 8 mutações já escritos).

---

## 8. Nota de processo

A linha **fecha aqui e PARA** (§0.7): ela **não integra e não pusha**.
Integração e ship são **ordem explícita do Enio**, por um agente integrador
dedicado munido deste documento.

⚠️ **E a §1 deste handoff envelhece:** ela mede o `main` do dia em que foi
escrita. Quando a ordem chegar, **re-meça a interseção** antes de rebasear — foi
exatamente essa caixa que envelheceu nas duas últimas integrações da
`line/sculpt3d` e da `line/physics`.
