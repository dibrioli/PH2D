# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, a W9 «Mesh Filter»

> **Para o agente INTEGRADOR.** Escrito segundo a DIRETRIZ §1.5.9.
> A linha **não integra e não pusha** (CLAUDE.md §0.7) — ela fecha, entrega isto
> e para.
>
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d`
> **Branch:** `line/sculpt3d` · **HEAD:** `3eaeb9732`
> **16 commits sobre `main`, 49 arquivos, +5.361/−150.**

---

## ⚠️ §0 — LEIA ISTO PRIMEIRO: a linha fecha com DUAS perguntas ABERTAS

**Integrar não é aprovar, e aqui a distinção tem nome.** Duas coisas que o Enio
reportou no smoke **não estão resolvidas**, e nenhuma delas é dívida de
engenharia — as duas foram medidas até ao fim e devolvidas a ele:

1. **⏸️ *"não temos undo para Filter"*** — a fiação está **toda lá** (o filtro
   fecha pela porta do traço, o traço grava o passo, o Ctrl+Z do modo 3D chama o
   desfazer) e existe gate a prová-lo (`the_whole_drag_is_one_undo_step`).
   ⚠️ **O que NENHUM teste percorre é a rota do PONTEIRO** (`pointer_down →
   move → up`): todos chamam as portas direto. É o vão onde o defeito tem de
   estar, e ele **não foi investigado** — a linha parou aqui por ordem de
   escrever este documento.
2. **⏸️ *"sharpen filter parece alisar o mesh"*** — o porte está **fiel**
   (conferido termo a termo contra o fonte **três** vezes), e a lei da
   referência, como escrita, **não é um afiador de propósito geral**: ela alisa
   detalhe fino e mal toca feição grande. **É pergunta de PRODUTO** e está no §6.

---

## §1 — O que a linha entregou

A **W9 «Mesh Filter»** do módulo 3D/Sculpt: o verbo aplicado à **malha inteira**,
dirigido por arrasto horizontal, em **nove leis** (`FilterKind`).

| wave | conteúdo |
|---|---|
| **W9a** | a FIAÇÃO — o driver, o interruptor no card TOOL, o gesto (força e SINAL, régua `0,001/px` da referência), **UM** passo de undo, e as 4 leis que reusam verbo (Smooth · Relax · SurfaceSmooth · Inflate) |
| **W9b** | as **3 leis que não têm verbo** (Scale · Sphere · Random) + o **PICKER**, que é o que as torna alcançáveis: enquanto a lei era derivada do verbo em mãos elas eram inexprimíveis por gesto nenhum |
| **W9c-a** | o **Enhance Details** — a medição partiu o item em dois: a LEI já era exprimível (o nosso `Smooth` em força negativa, a **1–2 ULP**), e o conteúdo inteiro dela é o **TETO** que a referência não tem |
| **W9c-b** | o **Sharpen** — a única lei que não existia, e o primeiro filtro do módulo com **PRÉ-PASSE** |
| **curas** | as duas do smoke (o panic do Surface Smooth · o Sharpen que alisava) e os **6 achados** da auditoria multiagentica |

**Placar:** o Mesh Filter sai de *«7 dos 9»* para **9 de 9**.

---

## §2 — Foundational tocado, e por que é ADITIVO

**Um arquivo, e ele é sculpt3d-específico PELO NOME:**
`crates/ph2d-editor-core/src/ids/chrome/sculpt3d.rs` — o array
`SCULPT3D_FILTER_KIND` cresceu **7 → 9**.

⚠️ **O `ids/chrome/mod.rs` NÃO foi tocado** ⇒ não há lista compartilhada no
diff. E **os 10 ids novos são todos `hash_node_id`** (medido: `0` literais
`NodeId(<número>)` no diff daquele diretório) ⇒ **nenhum gate de contagem em
risco, e nenhum valor para você grepar contra outra linha.**

`crates/ph2d-i18n/src/sculpt3d.rs` ganhou **+2 linhas** e o **`lib.rs` NÃO foi
tocado** ⇒ a cadeia `vector::tr(k).or_else(sculpt3d::tr)` que a integração de
10/08 instalou fica **intacta**.

---

## §3 — Superfície de colisão, MEDIDA (não auto-relatada)

Cada linha abaixo foi conferida por `git diff --stat main...HEAD -- <path>`
**depois** de confirmar que o path existe (uma busca negativa sobre caminho
inexistente devolve *«intocado»* para qualquer coisa).

| item | medido |
|---|---|
| **`PROJECT_SCHEMA`** | **84 INTOCADO** — ⚠️ os **TRÊS** sítios com diff **vazio**: `project.rs` · `project_schema.rs` (a `line/physics` PARTIU o arquivo em 15/08) · `project_schema_tests.rs`; a **tripla `(84, 13, 14)`** viva |
| **contrato congelado** | **INTOCADO** — `git diff` vazio em `crates/ph2d-nodegraph/` e `crates/ph2d-core/src/tool.rs` |
| **registro do `ph2d-ecs`** | **INTOCADO** ⇒ os **três** espelhos (`ph2d-ecs`, `ph2d-render`, `ph2d-script`) também |
| **`Cargo.toml` / `Cargo.lock`** | **ZERO** ⇒ nenhuma crate nova, **nenhuma dep externa nova**, nenhuma aresta interna |
| **ADR** | **ZERO** ⇒ a linha fica **FORA de toda disputa de número** |
| **ids** | 10 acréscimos, **todos `hash_node_id`**; **0** literais numéricos |
| **scrollbar id** | nenhum novo (o do painel segue **840**) |
| **`VEC_SCENE_SCHEMA` / `FLIP_SCHEMA`** | **14 / 13**, intactos |
| **`rayon`** | nenhum uso novo |
| **crates de GPU** | `ph2d-mesh-render` · `ph2d-render` · `ph2d-paint-gpu` · `ph2d-flip-render` · `ph2d-gpu-cook` com **diff VAZIO** ⇒ **esta linha não alcança os gates de adapter** |
| **cenas de smoke** | censo próprio: **33 reivindicações, 33 DISTINTAS, zero duplicata**, maior **34** ⇒ **próxima livre: 35** |

### O ISOLAMENTO, medido por diff

**Todo** arquivo tocado fora das crates do módulo é `sculpt3d*` **pelo nome** —
os dois foundational (`ids/chrome/sculpt3d.rs`, `i18n/src/sculpt3d.rs`), os cinco
do painel `ph2d-panel-sculpt3d`, e os onze do shell. **Nenhum deles é lista
compartilhada.** É o *«projete o foundational para ISOLAMENTO»* do ADR-0107 a
funcionar.

---

## §4 — O gate batched, rodado 1× sobre o diff acumulado

Máquina em `load 1,63` (⚠️ *nenhuma leitura de relógio desta workstation vale
acima de `load ~5`*).

| gate | resultado |
|---|---|
| `scripts/nextest-impacted.sh` (com `CARGO_INCREMENTAL=0`) | **10.074 correram, 10.074 passaram, 1.345 skipped, EXIT 0** |
| clippy `--workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` | **EXIT 0, zero warnings** |
| `cargo fmt --all -- --check` | **EXIT 0** |
| `typos` | **EXIT 0** (⚠️ pegou **1** — uma palavra portuguesa num doc-comment meu; corrigida reescrevendo a frase, **nunca** alargando a allowlist) |
| `ph2d-host-desktop --test file_loc_caps` (o da SHELL) | **2 passed** |
| `arch_safe_clamp_only` | **2 passed** |
| `architecture_workspace_file_loc_cap` | **2 passed** |
| `architecture_panel_loc_cap` | **3 passed** |
| `architecture_widget_loc_cap` | **1 passed** |
| auditoria | **8 lentes multiagenticas + 8 céticos** (§5) |

⚠️ **Os quatro arch-gates foram rodados por TARGET e com CONTROLE POSITIVO.** A
primeira tentativa usou `cargo test --workspace <nome>` e devolveu **`0 passed`**
— que **não é verde, é *nada rodou***. O nome do teste não é o nome do arquivo.

⚠️ **E os arch-gates da SHELL entraram na varredura impactada** (3.227 testes de
`ph2d-host-desktop`, incluindo `every_sculpt*` e `no_two_sculpt*`) — é a família
que já deixou vermelho latente duas vezes neste repo.

---

## §5 — A auditoria multiagentica, e o que ela achou

**Ordem do Enio.** 8 lentes independentes → dedup → 8 céticos com `refuted: true`
por omissão → síntese. **58 candidatos, 8 verificados, 8 sobreviveram.**

⚠️ **50 achados NÃO foram verificados** (cap de 8 por severidade). Eles estão no
journal do run e **não foram examinados**.

Os seis (três eram o mesmo mecanismo e foram fundidos), **todos curados**:

1. **[ALTA] O `EnhanceDetails` omitia o `-std::abs()` da referência**
   (`sculpt_filter_mesh.cc:1883`, a PRIMEIRA linha de `calc_enhance_details_filter`).
   Arrastar para trás fazia o vértice **atravessar a média do anel** e sair do
   outro lado ao dobro da distância. ⚠️ **Medido: com `f = −1` o resultado era
   BYTE-IDÊNTICO ao `Smooth(+1)`** — o chip dizia *Enhance Details* e a malha
   alisava à força total; com `−2`, **os 830 vértices** da sonda atravessavam.
   ⚠️ **Escapou porque a fixture não continha o fenómeno em TRÊS sítios**: os
   gates varriam só forças positivas, o `the_one_sided_filters_ignore_a_backwards_drag`
   cobre só Relax e HC, e **a sonda de paridade da wave escreveu o oráculo também
   sem o `abs`** — *um oráculo que herda a omissão do produto concorda com ele em
   toda parte*.
2. **[ALTA] O `SHARPEN_MAX = 4,0` media-se a si mesmo.** O `filter_sharpen`
   clampa a entrada pelo próprio teto **antes** da aritmética ⇒ a «saturação» que
   o doc afirmava era o clamp. Pela porta não-clampada a lei **não satura**
   (degrau `0,990× → 1,398×` de força 1 a 64, monotónico). E o gate que o
   defendia **não podia falhar**: cortar o teto a um quarto deixava a suíte
   inteira verde.
3. **[MÉDIA]** um parágrafo dizia que o `Verb::Sharpen` fica **de FORA** da
   tabela de sementes; o `match` **31 linhas abaixo, no mesmo doc-comment**,
   inclui-o.
4. **[BAIXA]** um gate citado (`a_filtering_verb_reads_nothing_from_the_dab`)
   que **nunca existiu** — `grep` devolve só a própria citação.
5. **[BAIXA]** o cabeçalho do Sharpen ainda dizia *«o pré-passe, uma vez por
   SUB-PASSO»* — a lei **errada** que a wave do mesmo dia existiu para corrigir.
6. **[BAIXA]** o `FILTER_DRAG_PER_PX` em **duas cópias**, com a da engine
   (documentada como autoritativa) **sem nenhum consumidor**.

⚠️ **Um falso vermelho a NÃO perseguir:** durante a auditoria o
`the_sharpen_raises_the_step_between_neighbours…` reprovou — era **mutação viva
de outro agente na árvore compartilhada**, não regressão. Re-rodado limpo: verde.
*Lição de processo: não corra uma auditoria que muta a árvore na árvore em que
você está a trabalhar.*

---

## §6 — ⏸️ O que fica para o ENIO decidir, com os números na mão

**(a) O Sharpen afia?** O porte é fiel. Medido na malha do produto (98.306
vértices), o degrau entre vizinhos:

| força | 1,0 | 2,0 | 4,0 | 8,0 | 16,0 | 64,0 |
|---|---|---|---|---|---|---|
| degrau | 0,990× | 1,003× | 1,046× | 1,124× | 1,225× | 1,398× |

E numa malha com **detalhe fino** ela **alisa** (`0,528× → 0,279×`).
⚠️ **Duas hipóteses minhas foram REFUTADAS por medição:** o
`sharpen_intensify_detail_strength` compra **2%** a oito vezes o default dele, e
a teoria de que ela afiaria detalhe fino saiu **ao contrário**.
⇒ Ou aceitamos a lei da referência, ou **divergimos de propósito** — e aí é uma
lei nossa, com o nome dela, um doc e um gate a defender a nossa posição.

**(b) O teto.** Subir compra excursão real (a lei não satura) e **paga em
milissegundos por evento de ponteiro**:

| fatias | 1 | 4 | **8** | 16 | 32 |
|---|---|---|---|---|---|
| tempo | 7,72 ms | 11,47 | **17,17** | 27,93 | 49,16 |

⚠️ **Um quadro de 60 fps são 16,7 ms**, e o teto de hoje (força 4,0 = 8 fatias)
já está **no limite**. A cura do outro lado — cortar o custo por fatia — é **wave
própria**.

---

## §7 — Mudanças de COMPORTAMENTO, nomeadas

1. **O `Enhance Details` realça nos dois sentidos** do arrasto (era: invertia a
   curvatura para trás). ⚠️ **Isto muda o desenho de qualquer gesto para a
   esquerda com aquele chip.**
2. **O `Surface Smooth` deixa de PANICAR** quando escolhido com um pincel de
   outro verbo — defeito **pré-existente**, que nasceu com o picker da W9b e que
   o smoke da `=34` expôs.
3. **O `Sharpen` afia em vez de alisar** (o `sharpen_factor` passou a ser
   congelado por gesto, como no `filter_cache` da referência).
4. **A malha deixa de explodir em valência alta** — guarda de valência,
   **divergência DECLARADA** e **identidade** para `n <= 6` (a valência de um quad
   ou de um triângulo regular) ⇒ a malha do produto é **byte-idêntica**.
5. **O picker tem 9 chips** (era 7).
6. **A régua do arrasto passa a ser a da engine** (as duas valiam `0,001`, então
   nada se move hoje).

---

## §8 — Os smokes, com o comando exato

⚠️ **O `cd` é o da WORKTREE**, não o do primário — o Enio roda de outro diretório
e sem ele o comando testa a árvore errada.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=34 cargo run -p ph2d-host-desktop --release
```

A cena **`=34`** (nova) abre com as CRISTAS de propósito e imprime o roteiro de 6
passos. ⚠️ **Se a lista não aparecer, PARE.** As perguntas: os **nove** chips ·
o arrasto de volta devolver a malha · o teto que a W9c-a removeu (com o
**CONTROLE** primeiro) · o Sharpen · a peça não explodir · e Sharpen ≠ Enhance
Details.

⚠️ **E rode uma vez SEM a env var** — é a metade que prova a inércia (sem ela o
`AppGfx.sculpt3d` é `None` e o frame 2D é byte-idêntico).

⚠️ **As cenas `=1..=33` têm de continuar iguais.**

⚠️ **Rode a suíte do módulo também em DEBUG** — precedente registado nesta casa
(o `ph2d-flip-colorize` panicava só ali).

---

## §9 — ABERTO, com o preço ao lado

- ⏸️ **O undo do filtro pela rota do PONTEIRO** (§0) — a única frente com um
  report do Enio sem diagnóstico fechado.
- ⏸️ **O Sharpen como produto** e **o teto** (§6) — decisões dele, já com a tabela.
- ⚠️ **Os 50 achados não verificados** da auditoria (§5) — estão no journal do run
  `wf_76127d6f-aa1` e **ninguém os leu**.
- **O custo por evento de ponteiro** é o buraco de medição que a auditoria nomeou
  como o mais importante: um filtro percorre a **malha inteira** por evento.
- **Nada de GPU foi exercitado** — o filtro invalida a malha toda quadro e o custo
  de re-upload não foi medido.
- **Multires, simetria e máscara em composição com o filtro** não foram testados.
- O `stroke_filter_tests.rs` está em **689/700** — a próxima adição obriga o corte.
- ⛔ As **três divergências declaradas** da referência (o `Flatten` bilateral e a
  projeção tangencial de `Pinch`/`Crease`) seguem como estavam; **não** são desta
  wave.

---

## §10 — ⚠️⚠️ O PONTO DE MERGE, e ele NÃO é um número: o `main` CORTOU o arquivo

**Medido, não previsto — este achado só apareceu porque tentei rebasar.**

A linha está **21 commits atrás do `main`**, e a interseção *arquivos da linha ∩
arquivos que o `main` moveu* é de **UM**:

```
docs/3D/21_plano_modos_e_ferramentas.md
```

⚠️ **E o `main` ENCURTOU-O de 3.711 para 494 linhas.** O commit `658494e60`
(*"a doença do CLAUDE.md tinha sido REALOCADA, não curada — 1,54 MB sai do
caminho quente"*) moveu a narrativa **verbatim** para
`docs/archive/docs-2026-08-18/3D/21_plano_modos_e_ferramentas.md`, com a
remontagem a conferir **sha256** com o original, e manteve a numeração `§N` de
propósito para os ponteiros internos continuarem a resolver.

⇒ ⚠️ **Ficar com o lado da LINHA neste conflito restauraria 3.200 linhas que o
`main` removeu deliberadamente.** *Um lado escolhido não é um conflito
resolvido* — e aqui o lado errado desfaz a wave de outra pessoa.

**Números para planear:**

| | medido |
|---|---|
| commits da linha que tocam o arquivo cortado | **5** de 16 |
| commits da linha que tocam o `CLAUDE.md` | **0** ⇒ o rebase traz a §5 nova **limpa** |
| resto da interseção | **vazio** |

**Prescrição:** o rebase é seguro em tudo **menos** naqueles 5 commits. Em cada
um, a resolução é **manter o corte do `main`** e re-aplicar a edição na
estrutura nova (ou no arquivo arquivado, conforme o conteúdo seja *estado* ou
*narrativa*). Eu **abortei** o rebase em vez de o resolver: são 5 resoluções de
conteúdo sobre uma reestruturação de outra pessoa, o que é trabalho de
**integração** e não da linha (CLAUDE.md §0.7).

### A entrada da §5 do `CLAUDE.md`, PRONTA

⚠️ **Ela não foi aplicada na worktree de propósito:** esta árvore tem o
`CLAUDE.md` **antigo** (a §5 narrativa), e o `main` tem o **roteador** — escrever
aqui criaria exactamente o conflito que este parágrafo existe para evitar.
Aplique isto na entrada **3D / Sculpt** do roteador, depois do rebase:

> **Aberto:** … *(remover a `**W9** (Mesh Filter — o mais barato: não há kernel
> novo)` da lista de abertos)* …
>
> Acrescentar: **A W9 FECHOU — o filtro tem as 9 leis** (Smooth · Relax ·
> SurfaceSmooth · Inflate · Scale · Sphere · Random · **Enhance Details** ·
> **Sharpen**), com o picker a torná-las escolhíveis (o verbo só SEMEIA). ⚠️ O
> `Sharpen` é o único filtro com **PRÉ-PASSE** e a lei da referência **depende da
> taxa de polling** (ela não restaura a pose entre eventos) — a nossa entrega a
> força em sub-passos **determinísticos**. ⛔ **Duas perguntas são do Enio, já
> devolvidas com a tabela:** se a lei da referência é o afiador que se quer (ela
> alisa detalhe fino e mal toca feição grande) e onde fica o teto (subir compra
> excursão real e paga **17,17 ms** por evento de ponteiro contra um quadro de
> 16,7). ⏸️ **E o undo do filtro pela rota do PONTEIRO segue por investigar.**
> **Smoke:** `PH2D_SCULPT3D_SMOKE=34` (⚠️ próxima cena livre: **35**).
