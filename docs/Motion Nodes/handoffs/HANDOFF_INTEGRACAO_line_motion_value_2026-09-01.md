# Handoff de integração — `line/motion-value` (2026-08-30 → 2026-09-01)

> **DIRETRIZ §1.5.9.** A linha fecha, entrega isto e **PARA**. ⛔ Não integra, não pusha
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)). O handoff anterior desta linha
> ([2026-08-29](HANDOFF_INTEGRACAO_line_motion_value_2026-08-29.md)) **já está no `main`**;
> tudo aqui é posterior a ele.

---

## 1. Identidade

| campo | valor |
|---|---|
| branch | `line/motion-value` |
| HEAD | **este commit de fecho** (docs); o último commit de CÓDIGO é `d99f0a724` — ⚠️ um handoff não pode nomear o seu próprio hash, e escrever o do commit anterior como se fosse o HEAD manda o integrador fundir uma árvore sem este ficheiro |
| merge-base com `main` | `066b4f92e` |
| commits | **44** (43 de código + este) |
| ficheiros | **161** (17 424 inserções, 839 remoções) |
| smoke do Enio | ✅ **OK em 2026-09-01** (cena `=108`, o arrasto do `Growth`) |

---

## 2. Foundational / compartilhado tocado — **tudo ADITIVO**

| ficheiro | o quê | aditivo? |
|---|---|---|
| `ph2d-nodegraph/src/attr.rs` | `TINT_MASK_COLUMN` — coluna nova, **ausente ⇒ `1`** | sim (toda corrente que não a escreve é byte-idêntica) |
| `ph2d-node-registry/src/{lib,ui}.rs` | `table_external_key` + o gate das chaves derivadas | sim |
| `ph2d-node-registry-init` | registo das 3 crates novas + 2 gates | sim |
| `ph2d-editor-core/src/text_elide.rs` | a porta que **CORTA** um rótulo (elipse) em vez de o quebrar | sim |
| `ph2d-editor-core/src/paint_text.rs` · `paint.rs` | `paint_text_weighted` `pub(crate)` para o `text_elide` medir e pintar no MESMO peso | sim |
| `ph2d-editor-core/src/widget/slider_with_chip.rs` | passa a usar a porta que corta | comportamento (o rótulo deixa de escrever por cima) |
| `ph2d-editor-core/tests/architecture_motion_chrome_never_wraps_a_row_label.rs` | **gate novo** (censo exacto, `ELIDED_TODAY = 27`) | novo |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | 1 entrada em `PANEL_A11Y_DELEGATE_OK` | aditivo |
| `ph2d-expr-parse/src/lib.rs` | profundidade máxima — uma expressão funda **RECUSA** em vez de abortar o editor por estouro de pilha | comportamento (o modo de falha era `SIGABRT`) |
| `ph2d-text/src/system.rs` | medição por peso, para o elide | sim |
| `shells/desktop/` | membranas, cenas de smoke, testes | sim |

⚠️ **`ph2d-expr-parse` é a única mudança de comportamento fora do módulo**: o editor de
expressões deixa de matar o app com `~14 KB` de texto. Achado da auditoria de seis lentes (§3.1),
e o recurso nomeado é a PILHA.

---

## 3. Símbolos que podem COLIDIR — saída de `collision-surface.sh` (2026-09-01)

```
SUPERFÍCIE DE COLISÃO — line/motion-value contra main
  merge-base 066b4f92e   ·   42 commit(s)   ·   161 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        103   (base: 103)
      └ tripla do gate               (103, 13, 17)   (base: (103, 13, 17))
    VEC_SCENE_SCHEMA                       17   (base: 17)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  79   (base: 79)
    ph2d-script (espelho)                  79   (base: 79)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — último no disco: 0168 · esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 3 pacote(s) '+name' novo(s), TODOS internos:
      "ph2d-node-source-table"   "ph2d-node-value-table"   "ph2d-table"
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⚠️ **PRAZO DE VALIDADE (§1.5.9 item 3):** esta tabela mede contra o `main` de **2026-09-01**.
O integrador **RE-RODA** `collision-surface.sh` na worktree antes de fundir; a divergência
entre as duas leituras é ela própria um achado.

**Símbolos novos a grepar:**

| símbolo | valor | onde |
|---|---|---|
| `TINT_MASK_COLUMN` | `"tint_mask"` | `ph2d-nodegraph/src/attr.rs` |
| `ELIDED_TODAY` | `27` | `architecture_motion_chrome_never_wraps_a_row_label.rs` |
| crates novas | `ph2d-table` · `ph2d-node-source-table` · `ph2d-node-value-table` | workspace |
| módulos novos | `ph2d-node-source-lsystem/src/{alphabet,width}.rs` · `ph2d-panel-motion-params/src/paint_seed.rs` | — |

⛔ **Nenhum schema se moveu, nenhum contrato congelado foi tocado, nenhum ADR foi cunhado.**

---

## 4. Contratos congelados (§6) — **NENHUM**

`NodeOp` / `OpResolver` / `NodeManifest` e `Tool` intocados (verificado pelo
`collision-surface.sh`). Todo canal novo é **side-metadata no registry**, como o §5 manda.

---

## 5. O que só o `ship.sh` pega

- **3 crates novas** ⇒ `cargo machete` e `cargo deny` nunca as viram.
- `typos` e `cargo fmt --all -- --check` correram aqui e estão **verdes**, mas contra a árvore
  desta linha — um merge textual pode reintroduzir deriva.
- `RUSTSEC`: sem dependência externa nova (as 3 são internas) ⇒ risco baixo, não nulo.

---

## 6. Ordem, dependências e o que smokar

Os 43 commits são **sequenciais e não reordenáveis** — cinco waves, cada uma sobre a anterior:

| # | wave | commits |
|---|---|---|
| 1 | **Ramos** — o esqueleto vira tronco contínuo (`Branches`, o padrão), a forquilha, a ponta, e os `55×`/`13×` de perf | `633bffdae`..`2c62bd1be` |
| 2 | **Folhas** — a LETRA planta o objecto (`J`/`K`/`M`), cinco controlos, a `TINT_MASK_COLUMN`, a terceira média | `d272766e0`..`490d6ee69` |
| 3 | **Fonte de dados** — CSV/JSON (3 crates novas), o campo do caminho, o rótulo que CORTA | `6e7a630a8`..`9e2a58744` |
| 4 | **Auditoria de seis lentes + curas** — 24 achados listados, 14 curados | `3313fa00c`..`30d148588` |
| 5 | **O crescimento** — o alfabeto como dado, a legenda, a lei do recém-nascido, a escada de tamanhos | `506a40152`..`cbe46810f` |

**Smokado pelo Enio (✅ 2026-09-01):** o arrasto do `Growth` na cena `=108`, nos moldes do
catálogo. Veredito: *«smoke ok»*.

**⚠️ NÃO smokado — o integrador ou a próxima janela tem de o fazer:**

- a **Data Source** (CSV/JSON) ponta a ponta com um ficheiro real do Enio — ela tem cena de demo
  e gates, e **nenhum veredito de produto**;
- a cena `=107` (sujidade na lente) depois da lei do recém-nascido;
- o editor de **expressões** com um texto fundo (a cura do `SIGABRT`), que é caminho de outro nó.

---

## 7. A NARRATIVA — as duas leis que esta jornada pagou

### 7.1 A régua não podia existir (a lei do recém-nascido)

Report do Enio: *«pequenos pulos»* ao arrastar o `Growth`. ⛔⛔ **Nenhuma régua desta linha os
via, e a cegueira é ESTRUTURAL:** `probe_flicker` e `probe_drift` medem um escalar de **TAMANHO**,
que é exactamente o que o `build` **normaliza** ⇒ *a régua partilhava a lei do produto, e um
espelho não acusa*. E a imagem rasterizada é cega à **SOBREPOSIÇÃO** (5 segmentos colineares sobre
o caminho do pai tocam as mesmas células). A grandeza livre é a **TINTA** — a soma dos
comprimentos desenhados.

A cura é ABOP §6.2.2 eq(6.10)/(6.11): material que **não retraça nada** nasce com comprimento
zero e cresce da fracção, em vez de aparecer inteiro. Bush: salto relativo de tinta
`0,6699 → 0,0027` (**248×**); factor de encolhimento `0,966× → 0,125×` — que é o valor **teórico**
do movimento a 8× de refinamento. Seis dos oito moldes ficam **byte-idênticos**.

⚠️ **O Houdini tem o mesmo defeito** (*"scales the geometry generated by the last substitution"*).

### 7.2 Suave e linear são DUAS perguntas (a lei da escada)

Report seguinte: *«está mais suave mas não é perfeitamente linear»*. ⛔ **Uma recusa medida
isentava metade dos moldes** — e a justificação escrita nela media **ondulação** (a razão entre o
maior e o menor passo, que é SUAVIDADE) quando a pergunta era **LINEARIDADE** (o afastamento da
recta). Medida a segunda, os quatro isentos iam `+6,9 %` a **`+21,3 %`** adiantados a meio do
arrasto.

Duas causas, uma por família: quem **refina** errava DENTRO das gerações (a remapagem resolvia
`size = rᵍ` e o `build` entrega a **CORDA** entre `rᵏ` e `rᵏ⁺¹` — a `r = 3` isso é `(1+2)/√3 =
1,155`, **+15,5 %**); quem cresce pela **ponta** errava em toda a parte, por nunca ter sido
linearizado.

A cura é **medir a escada de tamanhos e invertê-la** — sem modelo nenhum. Densidade por família:
refinador com `step_scale` neutro ⇒ 1 degrau/geração (a normalização já força a corda, ±0,02 %);
ponta ⇒ `TIP_SUBSTEPS = 3`; refinador com `step_scale ≠ 1` ⇒ 3 degraus, 2 passeios cada.

| molde | antes | depois |
|---|---|---|
| Wild | `+21,33 %` | `+0,13 %` |
| Sprig | `+14,17 %` | `+1,35 %` |
| Tree | `+11,00 %` | `−0,26 %` |
| Koch | `+9,76 %` | `−0,01 %` |
| Bush | `+9,75 %` | `+0,01 %` |
| Fern | `+6,88 %` | `+0,11 %` |
| Weed | `+5,73 %` | `+0,02 %` |
| Dragon | `+3,76 %` | `−0,46 %` |

Custo: `0,017`–`0,085 ms` em sete moldes, `0,628 ms` no Dragon (3,8 % de um quadro) — e **mais
barato que a lei que substituiu** (Bush `1,124 → 0,085 ms`).

Três sub-defeitos caíram no caminho: a escada derivava com **semente fixa** (um molde estocástico
media outra planta: Wild `−7,69 % → −0,29 %`); o **planalto** do Sprig (o ponto mais alto é um
galho lateral, o `y`-max fica preso ¼ de geração — colapsá-lo faria **20 %** da tinta aparecer de
uma vez); e o `step_scale` **pré-existente** (`0,3737 → 0,0244`).

### 7.3 A auditoria de seis lentes — 24 achados, 14 curados

[Doc 96](../96_auditoria_do_lsystem_2026-08-31.md). O que **fica aberto** está no §8.

⚠️ **Duas mutações SOBREVIVERAM** e a causa era a mesma: *um corpus no ponto NEUTRO de um knob
não testa esse knob* — os 8 moldes deixam o `step_scale` em `1,0` e nenhum tem planalto de mais
de ⅓ de geração. Fechadas com duas fixturas derivadas do MECANISMO, não do catálogo.

---

## 8. O que fica ABERTO

| item | estado |
|---|---|
| **`Grow Angle` para Bush/Weed** | ⏳ a lei existe e está medida; falta o **veredito do dono** (§0.8 — a pergunta foi-lhe entregue em 2026-09-01) |
| **§2.2** arrastar o `Generations` numa planta grande | ⛔ `17,88 ms` contra `16,67` a `78 124` ramos — e uma geração fraccionária deriva a SEGUINTE (`5×`) |
| **§2.3** o aviso «uma vez só» a cada quadro | ⛔ `SAID` é chaveado por 31 params ⇒ nova chave a 60 Hz, `~280 B/quadro` sem varredura |
| **§2.4** o memo das fitas | ⛔ `0 de 237` quadros evitaram uma reconstrução (o `sweep()` despeja no mesmo quadro) |
| **§2.5** a leitura de pixels da folha | ⚠️ lê o atlas INTEIRO: `268 MB` de staging + `Vec<u8>` retido pela vida do processo |
| **§3.2** trocar de molde e voltar a `Guided` | ⛔ apaga o fio e esconde controlos vivos |
| **§3.3** duas plantas iguais com folhas diferentes | ⛔ partilham a corrente |
| **§3.4** `NaN`/`Inf` a jusante | ⛔ de uma gramática que o parser ACEITA |
| **§3.5** `MAX_GENERATIONS = 32` | ⛔ é cerca de PAINEL; o modelo aceita `65 535` |
| **§4.2** o gate de pixel | ⛔ mede a **linha reservada**, não a tinta |
| `Step Scale ≠ 1` | ⚠️ deixa `2,4 %` de curvatura (contra `≤1,35 %` no neutro) — **nomeado, não curado** |
| Sprig `+1,35 %` | ⚠️ **não é defeito** — é o planalto da figura dele (§7.2) |

⛔ **§2.6 SAIU da lista:** a `measure_ratio` deixou o caminho do produto (a escada substituiu-a) e
hoje só é chamada pelo `probe.rs`. *A auditoria estava certa no dia em que foi escrita.*

---

## 9. ⚠️ O que uma leitura rápida do diff entende ao CONTRÁRIO

1. **`TINT_MASK_COLUMN` não é «mais um `falloff`».** A 1.ª cura usou o `falloff` e **PARTIU a
   planta**: ele é a máscara de TODOS os modificadores, então livrar uma linha do tint
   deixava-a **parada enquanto o resto se movia**. *O canal escolhido era muito mais largo do
   que a pergunta feita.*
2. **A escada de tamanhos não é «um cache do `measure_ratio`»** — é a substituição dele. Ela
   mede `size(g)` numa grelha e **inverte a curva por partes**; não há modelo `rᵍ` nenhum.
3. **`TIP_SUBSTEPS = 3` não é afinação** — é a densidade MEDIDA pela tabela de `M`. Um refinador
   com `step_scale` neutro precisa de **1** degrau por geração e não de 3, porque a normalização
   já força a corda (±0,02 %).
4. **O `from_drawing` do `Module` não é «o símbolo desenha?»** — é *«o símbolo que me PRODUZIU
   desenhava?»*, escrito no laço do sucessor. É a diferença entre o `F` de um `X` mudo (material
   novo, nasce a zero) e o `F` de um `F` (retraça, nasce inteiro).
5. **O `paint_seed.rs` não é arrumação** — é o corte que o tecto de LOC de FUNÇÃO impôs, e ele
   separa *«que pixels saem?»* de *«que estado o quadro seguinte encontra?»*.
6. **O `width.rs` não «tirou código da tartaruga»** — a régua do tamanho responde a outra
   pergunta e a dependência é de sentido único. A tartaruga não a chama.
7. ⛔ **A cena `=12` foi corrigida porque ENSINAVA O CONTRÁRIO**, não porque estava feia
   (§5.0: *uma cena que ensina o contrário é pior que uma cena ausente*).

---

## 10. As premissas que a implementação REFUTOU

1. *«a régua de tamanho vê os pulos»* — **não pode**: ela mede o escalar que o `build` normaliza.
2. *«a imagem rasterizada é a régua honesta»* — cega à sobreposição.
3. *«a remapagem piora quem cresce pela ponta»* (a recusa medida) — media **ondulação**; em
   **linearidade** ela é a única coisa que os endireita.
4. *«o `Growth` custa `2,6×`–`31×`»* (auditoria §2.6) — verdade no dia, e a escada tornou-o falso.
5. *«a legenda do alfabeto não tem superfície»* — a 1.ª medição concluiu-o a partir de **onde a
   função vive** em vez de **onde ela corre**: o `paint_hover_tooltip` corre depois de todos os
   painéis. *Uma ausência afirmada pelo endereço do código é um palpite com cara de medição.*

---

## 11. O portão do fecho

| lente | resultado |
|---|---|
| `nextest-impacted.sh` (`NO_FAIL_FAST=1`, `BASE=main`) | **14 550 passed · 1 failed · 1 357 skipped** |
| a 1 falha | `ph2d-node-motion-soft-body::the_shape_match_is_linear_in_the_mesh` — **flake de carga do §5.0** |
| `cargo clippy --all-targets` (3 crates do corte) | limpo |
| `cargo fmt --all -- --check` | limpo |
| `typos` | `exit 0` |
| `doc-index.sh --check` | ✓ 14 índices em dia |

⚠️ **A flake é membro NOVO da família e está confirmada pelas três assinaturas do §5.0:** gate de
RAZÃO · **zero linhas de diff** desta linha naquela crate · **5 de 5 verde sozinha** · e o
**CONJUNTO de reprovadas MUDOU entre duas corridas da mesma árvore** (a anterior falhou o
`hr12_widgets_a11y`, que era real e foi curado).

⛔⛔ **E o portão apanhou QUATRO vermelhos que a jornada tinha criado** — tofu, o censo de
elisão, e dois tectos de LOC —, todos curados por **corte por responsabilidade**, nunca por
isenção. ⚠️ **A 1.ª corrida do portão imprimiu `exit 0` sobre uma suíte que parou em 1 540 de
14 551**: eu canalizei por `| tail`, que substitui o exit code, e o `nextest` cancela na 1.ª
falha. *A mesma armadilha do `CLAUDE.md §2`, pela 2.ª vez nesta linha.*

---

## ⛔ Recusas MEDIDAS (desta jornada)

| Item | Motivo |
|---|---|
| Colapsar o planalto do Sprig | ⛔ **20 %** da tinta apareceria de uma vez — é o defeito que a §7.1 acabou de curar |
| `lat = 0` no braço do `Grow Angle` desligado | ⛔ **apaga os ramos laterais** da figura |
| Uma lei só para as duas famílias de crescimento | ⛔ no braço da ponta a viragem contínua é **obrigatória**, não desligável; fundi-las daria ao artista de um refinador o poder de desligar o que nas outras é lei |
| Densidade de escada uniforme (3 degraus para todos) | ⛔ paga `3×` num refinador neutro para uma resposta que a normalização já dá a ±0,02 % |
| Entrada de isenção para os dois tectos de LOC | ⛔ a lei da casa é **corte por responsabilidade** |
| Semente fixa (`1`) na escada | ⛔ um molde estocástico media **outra planta** (Wild `−7,69 %`) |
