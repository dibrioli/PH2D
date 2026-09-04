# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, 2026-09-03

> ⛔ **Integrar e shipar são ordem EXPLÍCITA do Enio** (§0.7). Esta linha **fecha, entrega, e
> PARA**. Nada aqui foi integrado nem enviado.

## 1. Identidade

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value` |
| ramo | `line/motion-value` |
| merge-base | `066b4f92e` |
| commits | **62** · 192 ficheiros · `+22 659 / −4 585` |
| HEAD | `0b186dbc7` |
| binário release | **compilado** |

⚠️ **Os 48 primeiros commits já têm handoff** —
[`HANDOFF_INTEGRACAO_line_motion_value_2026-09-01.md`](HANDOFF_INTEGRACAO_line_motion_value_2026-09-01.md),
que cobre o `source.lsystem`, a Data Source e a auditoria de seis lentes. **Este documento
cobre os 14 seguintes** e vale para a integração da linha inteira.

---

## 2. Superfície de colisão — `collision-surface.sh`, 2026-09-03

```
SCHEMAS          PROJECT_SCHEMA 103 (base 103) · tripla (103, 13, 17) idêntica
                 VEC_SCENE 17 · FLIP 13 · DOC_VERSION 18 — TODOS na base
CONTRATO §6      node.rs INTOCADO · tool.rs INTOCADO
ADR              esta linha não cria nenhum ⇒ fora da disputa de número (próximo livre: 0169)
Cargo.lock       3 pacotes novos, os TRÊS internos: ph2d-table · -node-source-table · -node-value-table
MARCADORES       nenhum (inclui `|||||||`)
TETOS DE LOC     nenhum ficheiro da linha passa
```

⇒ **Zero disputa de número.** Nenhum schema se moveu, nenhum contrato congelado foi tocado.

---

## 3. Foundational tocado nesta 2.ª metade — **tudo ADITIVO**

| ficheiro | o que entrou | por que é aditivo |
|---|---|---|
| `ph2d-tokens/src/color.rs` + `docs/design/tokens.json` | **`PortValue`** (teal, matiz 190) nos **3 temas** | um token novo no fim da lista; nenhum existente mudou de valor |
| `ph2d-editor-core/src/paint_shapes.rs` | `fill_polygon(scene, pts, color)` | primitiva nova ao lado do `fill_diamond`/`fill_slash`; nenhuma assinatura mexida |
| `ph2d-node-registry/src/lib.rs` | `register_primary_input` / `primary_input` + o mapa | side-metadata, **ausente ⇒ `0`** — os 133 tipos que não a declaram ficam byte-idênticos |
| `ph2d-editor-core/tests/hr12_widgets_a11y.rs` | **1 entrada** em `PANEL_A11Y_DELEGATE_OK` | acrescenta uma linha; a justificação é uma propriedade **verificada por contagem** (zero `HitIndex`/`WidgetStore` no ficheiro) |

⚠️ **O `NodeManifest` NÃO foi tocado** — e a decisão é load-bearing: a porta principal de um nó
entrou no **registry** exactamente porque o contrato está congelado (§6). Um campo novo ali
pediria ADR.

---

## 4. Ordem, e o que smokar

Sem dependências entre commits: os 14 são independentes uns dos outros e do resto da linha.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-motion-value \
  && env PH2D_GPU_COOK_DEMO=108 cargo run -p ph2d-host-desktop --release
```

1. **Pinos por espécie** — teal = um número · roxo = uma corrente · o pulso tem cor própria.
2. **Selo de papel** no cabeçalho dos nós de fonte / decisão / junção / terminal (⚠️ os 106 nós
   `Rect` **não** têm selo, e é a lei).
3. **Balão no socket** — passar o rato diz `Target X · a number`.
4. **A recusa que ensina** — largar um `motion.oscillator` no `Angle` de um `motion.rotate` diz
   *«…insert a `value.attribute` to read one number from it»*.
5. **O splice do duplicator** — inserir um `motion.duplicator` num fio: ele entra em `points`, o
   selo ⚠ diz `MissingInput("shape")`, e ligar uma forma faz as cópias aparecerem.
6. `PH2D_MOTION_ROUTE_LOG=1` imprime a rota do cook (device / híbrido / CPU, **com o motivo**).

---

## 5. A NARRATIVA — o que estes 14 commits são

### 5.1 A auditoria de performance ([doc 98](../98_auditoria_de_performance_2026-09-01.md))

⭐ **O teto do módulo não está num kernel.** Medido: o device faz **4,19 M objectos em 3,85 ms**
(23% de um quadro) contra **195,9 ms da CPU** — `50,9×`. E **69,7% das 109 cenas do produto
nunca chegam lá**: `67%` por uma escada que **não nomeia recurso nenhum** (o doc dela diz
*«F1.1's scope; F2+ territory»*, que é escopo de wave). Das 73 cenas multi-sink, **23 já teriam
TODOS os sinks no device** — falta compor dois planos num buffer.

⭐ **E a queda era MUDA.** Hoje toda recusa passa por `fell(motion, "…")`, que a NOMEIA, e o
`PH2D_MOTION_ROUTE_LOG=1` imprime-a. Gate textual **que descasca comentários** + gate de borda,
as duas mutações mortas.

⭐ **O lowering perguntava a mesma coisa 4,19 M vezes**: `49,0 ms` (três quadros) eram dois
`BTreeMap<String,_>` repetidos por linha. `MediaColumns` iça-os; o lowering vectorial, que era o
único serial dos dois, paralelizou (`2,47×`, byte-idêntico com gate dos dois lados do
`PAR_THRESHOLD`).

### 5.2 Os três reports do Enio sobre o `motion.duplicator`

1. **O fio entrava na porta errada** e o TIPO não podia acusar (as duas entradas são
   `INST_VEC2`) ⇒ `register_primary_input`, side-metadata declarada.
2. **A simulação MORRIA**: das oito colunas do emissor chegava **uma**. `Transfer::ShapeWins`
   saía **antes do laço**, deitando fora também as colunas que **ninguém disputa**. ⇒ *um modo
   que resolve CONFLITO não decide sobre o que ninguém disputa.* Queda em 40 tiques:
   `0,000 → 0,782`, idêntica ao controlo.
3. **O `size random` «parava»** — ⛔ **não é defeito**: o `Point Scale` existe, e a `1` o `size`
   volta com a faixa exacta. A cura óbvia foi **construída, medida e REVERTIDA** (apagava o peso
   interpolado do knob, e dois gates existentes apanharam-na).

### 5.3 O estudo do Mini Cavalry ([doc 99](../99_estudo_do_mini_cavalry_2026-09-02.md))

⚠️ **O `visual-tokens.js` dele abre com «Doc PH2D §6»: o sistema visual dele é uma spec NOSSA.**
Empate no catálogo (**134 nós cada**), e ele fá-lo em **22 584 LOC** contra 136 093 de UMA crate
nossa.

Dos cinco canais visuais, **dois estavam mortos** e os dois foram curados:

- **a COR do socket** tomava **um valor só** (`Instances` em 100% das 138 portas). Hoje diz a
  espécie — pulso (`PortEvent`, um token que **existia e nunca fora usado**) · número
  (`PortValue`, novo) · corrente (a cor de sempre, **93 das 138 portas não mudam um pixel**).
- **a SILHUETA** era declarada por 132 nós, transportada até ao pintor e **nunca lida**. Hoje é
  um selo no cabeçalho, e `Rect` (**80,3%**) não veste nenhum.

---

## 6. ⚠️ O que uma leitura rápida do diff entende ao CONTRÁRIO

1. **`primary_input = 1` no duplicator não é «a porta certa» — é a porta que deixa a cena
   VAZIA.** Medido: com o fio nos `points` a saída é **0 linhas**. Só se sustenta porque o selo
   ⚠ `MissingInput("shape")` **já existia** e aparece, com cura clicável, e porque fica a **um**
   fio do que o artista quer contra três.
2. **A nota `drops` no cartão FOI REVERTIDA.** O diff mostra `dropped_at` vivo — ele é
   `#[cfg(test)]`, um **instrumento**, não uma superfície. Quem o ligar a um cartão repete uma
   recusa medida (§10d do doc 99).
3. **As duas cercas do `dropped_at` também foram tiradas**, e não por descuido: elas existiam
   para salvar a nota, e as duas mutações que as apagavam **sobreviveram** ao gate.
4. **O `socket_tip` não é posto em todos os sockets.** É **um** por quadro, o do `hot_id` — e a
   1.ª versão punha ~45, o que o diff de `hits.rs` ainda deixa entrever no histórico.
5. **`fell(...)` substitui `GpuOutcome::FellThrough` e há um gate a PROIBIR o literal** — mas há
   um segundo gate (`the_gpu_cook_recusal_placement`) que antes o **exigia**. Ele foi reescrito
   para pedir `fell(`; ⛔ **não** o reverta para o literal, senão um dos dois fica impossível de
   satisfazer.
6. **O `motion.lattice` na fixtura do gate de perda não é arbitrário** — saiu da varredura
   `which_nodes_drop_a_streams_columns`, que o mediu.
7. **O `Point Scale` NÃO foi mexido.** O diff do duplicator mostra `apply_point_scale` intocado;
   a cura que o tocava foi revertida.

---

## 7. As premissas que a implementação REFUTOU

| eu escrevi | a medição disse |
|---|---|
| *«os lookups são ~73% do caminho misto»* | aquele caminho **já era paralelo**: 49 ms por 32 fios somem no ruído |
| *«pintar a silhueta é grátis»* | **não**: o contorno do CARTÃO arrasta hits e sockets. Grátis só no cabeçalho |
| *«o balão no socket é uma wave — os sockets não estão no `HitIndex`»* | **estão**, e o a11y deles sai de lá. Custou **um `set_tooltip`** |
| *«a cor do socket tem dois valores»* | **três** — o `Clock::Event` distingue 8 portas |
| *«ele rotula todas as saídas e nós não»* | a lei dele é **a mesma**: saída única não leva rótulo |
| *«ele tem mais nós»* | **134 de cada lado** |
| *«3 de 3 vermelho sozinho ⇒ não é flake»* | a máquina estava a **`load 82`**. A `load 3,2`: 3 de 3 verde |

---

## 8. O que fica ABERTO

1. ⛔⛔⛔ **O multi-sink no device** — 67% das cenas, `50,9×`, **23 a um passo de composição**.
   Wave com desenho próprio (dois planos, um buffer; o que significa a ORDEM dos sinks no
   device). **Decisão do Enio.**
2. ⏳ **`motion.mixer` e `motion.lattice`** perdem as colunas exclusivas na porta **0** (a
   varredura mediu; perder na porta 1 é quase sempre correcto).
3. ⏳ **O `Point Scale` deve nascer em `1`?** Agora que tudo o resto chega por omissão, ele é o
   único hold-out. **Decisão do dono** (muda arte gravada).
4. ⏳ **O rótulo `Shape` do `motion.emitter`** colide com o do duplicator — é o `shape_mode`
   (*onde a partícula nasce*). Rótulo, não param.
5. ⏳ **A oferta CLICÁVEL da conversão** — o `Toast` não tem acção (wave de chrome) e o menu
   filtra antes da recusa.
6. ⏳ **O FIO pela espécie** — a `GraphEdgeView` só carrega `out_domain`.
7. ⏳ **Os ~29 nós por-elemento seriais** e os **60 sem kernel** (doc 98 §4).
8. ⏳ **Os 20 tutoriais** — o único buraco em que temos **nada**, e custa conteúdo, não código.
9. ⏳ **§2.2 da auditoria do L-System** — arrastar `Generations` numa planta grande (`17,88` ms).

---

## 9. O portão do fecho

`cargo nextest run --workspace --no-fail-fast`, 2026-09-03:

```
20 321 testes · 20 319 passaram · 2 falharam
  ph2d-flip-render::pack_perf::packing_a_dense_scribble_is_bounded
  ph2d-timeline::nesting_clock::the_cost_of_depth_is_linear_not_explosive
```

⚠️ **Os dois são membros NOMEADOS da família de flakes do §5.0**, confirmados **3 de 3 VERDES a
`load 3,95`**, com **zero linhas** do diff nas crates deles — e o **conjunto de reprovadas MUDOU
entre corridas da mesma árvore** ao longo do dia (uma corrida deu `panel_loc_cap` +
`offset_live_cost`, outra deu três diferentes, uma deu **zero**), que é a assinatura da família.

⚠️⚠️ **E uma lição de régua que ficou no §5.0:** o *«sozinho»* da assinatura quer dizer **com a
CARGA MEDIDA**, não *sem filtro*. Li um deles como `3 de 3 VERMELHO` sozinho e quase o arquivei
como defeito real — a máquina estava a `load 82`.

**Quatro vermelhos foram criados e curados por CORTE ao longo da jornada**, nenhum por isenção:
dois tetos de LOC (⇒ `paint_role.rs`, `hits_tests.rs`, `motion_bridge_drive_param_tests.rs`,
`motion_bridge_rewire_duplicator_tests.rs`), um `no_magic_numeric` (⇒ constantes nomeadas) e um
tofu (⇒ ASCII nos **literais**, os doc-comments ficam).

---

## ⛔ Recusas MEDIDAS desta 2.ª metade

| recusa | mecanismo |
|---|---|
| a nota **`drops`** no cartão | dispara em quase todo nó e quase sempre sobre comportamento correcto; falta a INTENÇÃO, que não é derivável (doc 99 §10d) |
| a escala do ponto passar quando a forma não tem `size` | apaga o peso interpolado do `Point Scale`; dois gates existentes apanharam-na |
| declarar `reads/writes` à mão nos 67 nós mudos | a diferença de conjuntos sobre correntes vivas cobre os **134** sem declaração |
| bilinguizar o cartão como ele | contraria a lei do próprio dono ([[feedback_app_ui_english_only]]) |
| as duas cercas do `dropped_at` | eram para salvar a superfície revertida, e as mutações que as apagavam **sobreviveram** |
