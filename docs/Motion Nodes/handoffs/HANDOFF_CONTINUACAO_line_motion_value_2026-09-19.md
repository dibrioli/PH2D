# HANDOFF DE CONTINUAÇÃO — `line/motion-value`, 2026-09-19

> **Troca de agente a meio da linha** (DIRETRIZ §1.5, `MODELO_TROCA_DE_AGENTE_NA_LINHA.md`).
> ⛔ **A linha NÃO está fechada e NÃO deve ser integrada** — o dono mandou passá-la a uma janela
> nova com **três defeitos abertos**, e dois deles são regressões desta jornada.

**Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` · **branch**
`line/motion-value` · **84 commits** sobre o merge-base · árvore limpa.

---

## §1 — O QUE ESTA JORNADA FEZ (doc 115 §32, lá está o mecanismo)

A ordem do dono de 2026-09-17, **reaberta em 19/09 como report de defeito**:

> *«Não deveriam renderizar nada na tela, mas deveriam apenas disponibilizarem a posição e direção
> […] e deveriam ser dependentes de Duplicator e Shape para aparecer na tela. OU seja, sem o
> duplicator só aparece um gizmo de osso ou segmento de corda […] que não renderiza em runtime.»*

| wave | o que entrou | estado |
|---|---|---|
| **W0** | a medição: **111 de 123** cenas desenham só posições | — |
| **W1** | a LEI (`tem_aparencia` + `SinkStyle::so_com_forma`), **desligada** | ✅ |
| **W2** | o GIZMO (`ponto_gizmo` + `_overlay`), osso · corda · cruz | ✅ |
| **W2b–d** | a pegada da peça · o device obedece · o osso deixa de seguir o zoom | ✅ **smoke aprovado** |
| **W2e** | a tomada é vazia com a lei desligada (custo de fábrica a zero) | ✅ |
| **W3** | a **secção do gizmo** no cartão (`ph2d-gizmo-params`), só no `motion.grid` | ⛔ **REPROVADA** |

**A porta do produto é `PH2D_MOTION_SO_COM_FORMA=1`.** Com ela: o que não veio de um
`source.object`/`source.shape` não vira pixel, e aparece como gizmo. Sem ela: **nada muda** (há
`const _: () = assert!` a prender o default, e gate a provar que o gizmo não aparece).

---

## §2 — ⛔⛔⛔ OS TRÊS DEFEITOS ABERTOS (o report literal de 19/09)

> *«tudo errado. Se coloco o tamanho, para de animar. Gap y quebrou e movimenta tudo em vez de
> criar espaço.. O campo shape não é mais necessário..»*

### D1 — «Se coloco o tamanho, para de animar» — ⭐ CAUSA CONHECIDA, cura por decidir

**É um defeito de DESENHO meu, não um bug.** O `Gizmo Size` foi escrito para **GANHAR** da peça
(`ponto_gizmo_overlay::caminhos`, o fecho `peg`): com ele preenchido, a coluna `size` deixa de
dimensionar o glifo — e é a `size` que o oscilador anima. ⇒ *o absoluto matou a animação*.

⚠️ **As duas coisas que ele pediu são compatíveis e eu escolhi uma:** o tamanho absoluto devia ser
a **BASE** que o grafo MODULA, não um substituto. A forma provável:

```
glifo = gizmo_size × (size_do_elemento / size_de_REFERÊNCIA)
```

⛔ **E a referência é a pergunta que falta responder** — a mesma que a §32.4-ter já enfrentou e
resolveu com a `pegada_px`: ali a âncora é *«o tamanho da peça no zoom de fábrica»*. Com um
`gizmo_size` absoluto, a referência tem de ser algo que **não** anule uma pulsação uniforme (a
mediana da corrente anularia — está medido e recusado no raciocínio da §32.4-ter).

**Ficheiros:** `crates/ph2d-app-motion/src/ponto_gizmo_overlay.rs` (o fecho `peg` e `pegada_px`) ·
os gates `o_tamanho_absoluto_ganha_da_peca_e_nao_ve_o_zoom` e `a_coluna_de_escala_engorda_o_glifo`
**vão ter de mudar de premissa** — escreva a morte delas no diff.

### D2 — «Gap y quebrou e movimenta tudo em vez de criar espaço» — ⛔ NÃO REPRODUZ

**TRÊS hipóteses medidas, as três LIMPAS** (as sondas ficam no repo, `--ignored`):

| sonda | o que diz |
|---|---|
| `o_gap_y_ainda_espaca` | extensão Y `1,5 → 3 → 6 → 12` e centro **`0,000`** nas quatro ⇒ o **nó** espaça certo na CPU |
| `o_que_o_cartao_do_grid_pinta` | as 8 linhas na ordem, `Gap Y` ligado a `gap_y`, as duas do gizmo na secção ⇒ o **cartão** não trocou de dono |
| leitura do kernel | `GPU_KERNEL.params` do Grid é a lista **PRÓPRIA** de 4 nomes, não o manifesto ⇒ acrescentar params **não** mexe no struct do WGSL nem no `encode` |

⇒ **comece por REPRODUZIR no app**, não por re-medir isto. A hipótese que sobra e que eu **não**
consegui testar sem olhos: o `motion.grid` tem um param **`shape`** (a REGIÃO: que células existem)
com `inner` (buraco) de default **`0,5`** — e a região é derivada das **extensões** da grelha, que o
`gap_y` muda. Mexer no `gap_y` de uma região não-rectangular re-escolhe **quais** células existem, e
isso lê-se como *«moveu tudo»*. ⚠️ **Se for isso, é PRÉ-EXISTENTE** e só ficou visível porque o
gizmo tornou cada ponto legível — o que muda a cura inteira. **Meça antes de curar.**

### D3 — «O campo shape não é mais necessário» — ⛔ AMBÍGUO, NÃO DECIDA SOZINHO

Duas leituras, e elas levam a trabalhos opostos:

1. **o `shape` do `motion.grid`** (a região: Rect/círculo/anel) deixou de fazer sentido agora que a
   forma vem do `motion.duplicator` ⇒ retirar o param (e o `inner` com ele);
2. **o `Gizmo Shape`** que esta wave acrescentou não é necessário ⇒ retirar metade da W3.

⚠️ **A 2.ª leitura é compatível com o *«tudo errado»* que abre o report.** Pergunte ao dono qual é,
com as duas frases na mesa — *uma retirada feita na leitura errada apaga trabalho aprovado*.

---

## §3 — ⚠️ O QUE UMA LEITURA RÁPIDA DO DIFF ENTENDE AO CONTRÁRIO

1. **A lei ship DESLIGADA e isso é erro de compilação**, não um teste (`const _: () = assert!` em
   `ph2d-render/src/sink_style.rs`). Um `assert!` de teste sobre uma const é dobrado pelo compilador.
2. **O gizmo e a arte são UM interruptor**: ou se vêem as peças, ou se vê o gizmo. Não são dois.
3. **O device tem a sua própria metade da lei** (`ponto_gizmo::a_arte_desenha`, lida em
   `present.rs`): ela responde pela **FRONTEIRA**, sem readback. ⛔ A partição de texturas **não**
   serve para isso — ela também fica vazia num grafo de objectos todo-atlas.
4. **O que segue o zoom é a GEOMETRIA** (onde as juntas estão, o comprimento do osso). O glifo não.
   Há gate com controlo dentro.
5. **A secção do gizmo entra NA tabela de dicas do nó**, nunca num 2.º `register_param_ui` — o
   registo é *«a última escrita vence»* e apagaria as dicas que o nó já tinha (aconteceu; **seis**
   censos acusaram).
6. **O `SECTION_FLOOR` continua em `9`** — a secção do gizmo sai do censo por referência à constante
   que a declara. Baixá-lo para `8` **cascateia** para o `force.buoyancy` e o `force.vortex`.
7. **`quem_desenha_sem_forma` pergunta ao GRAFO e `colunas_que_chegam_ao_sink` coze** — a segunda lê
   `0 linhas` nas cenas de `source.shape` porque o `motion_shape_gen` corre no QUADRO. Não é um censo
   de população.

---

## §4 — ⏳ O QUE FICA ABERTO, ALÉM DOS TRÊS

- **A secção está só no `motion.grid`** (raiz de 97 das 111 cenas). As outras **13** fontes derivadas
  herdam-na com três linhas cada (`SPEC_FORMA`/`SPEC_TAMANHO` no manifesto · `HINT_*` na tabela de
  dicas · `escreve` no `eval`); os produtores COM entrada (corda, corpo mole, bando, `distribute_*`)
  recebem as colunas por **propagação**.
- **As 111 cenas não migraram** — ver doc 115 §32.5. Uma cena migrada (`source.shape` +
  `motion.duplicator` antes do sink) desenha certo com a lei **ligada ou desligada**, logo a migração
  é incremental e a porta fecha-se no fim. O funil da `=120` é o `pousa` (uma função, seis painéis).
- **O tutorial do ciclo 9 muda de premissa** sob a regra nova. ⛔ Migrar a cena sem reescrever o
  tutorial é o defeito que o `CLAUDE.md` §5.0 chama de *pior que uma cena ausente*.
- **O instrumento que separa `cozer` de `separar` no readout do MOTION** continua por construir
  (doc 115 §31) — ele vem antes de qualquer cura de performance.

---

## §5 — PROVA DO ESTADO (corrido nesta árvore, hoje)

| portão | resultado |
|---|---|
| `cargo test -p ph2d-app-motion --lib` | **1 208** verdes |
| `cargo test -p ph2d-host-desktop --test it` | **848** verdes |
| `cargo test -p ph2d-gizmo-params` | 5 verdes |
| `cargo clippy --workspace --all-targets -- -D warnings` | **zero** |
| contadores partilhados | `PROJECT_SCHEMA`, os três registos e os schemas de documento **intocados** |
| contrato congelado (§6) | **intocado** — a secção é side-metadata no registo |

**Mutação desta jornada: 29 mutações, 29 sangram** (8 na W1 · 7 na W2 · 5 na W2c · 6 na W2d · 1 na
W2e · 6 na W3), mais **um controlo inerte que sobrevive**, de propósito.

⚠️ **Crate nova:** `ph2d-gizmo-params` (folha, zero deps além de `nodegraph` + `node-registry`). A
membresia da workspace é por **glob** — nenhuma edição central.
