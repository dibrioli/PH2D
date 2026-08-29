# HANDOFF DE INTEGRAÇÃO — `line/components` (F4 fecho + F5) — 2026-08-27

> Para o **agente integrador**. DIRETRIZ §1.5.9. A linha fechou e **PAROU**: ela não integra, não
> roda `foundational-integrate.sh` e não faz ship.

---

## 1 — Identidade

| | |
|---|---|
| Branch | `line/components` |
| HEAD | `2b533a9d789e2bf91c6d4df03d931e19fa5b1190` |
| Base do fork (merge-base com `main`) | `330582deb` |
| Commits | **19** |
| Arquivos | **83** (+7 773 / −680) |
| `main` andou desde o fork? | **NÃO** (`git log HEAD..main` vazio) ⇒ hoje é **`--ff-only` puro** |

⚠️ **O `--ff-only` puro é a leitura de 2026-08-27, não uma garantia.** Se outra linha integrar
antes desta, tudo abaixo vira referência e o integrador **re-roda o `collision-surface.sh` nesta
worktree** (§1.5.3).

---

## 2 — Foundational / compartilhado tocado, e por quê

**Todos os toques são ADITIVOS** salvo os dois cortes de LOC (item 2.4), que são movimentação.

### 2.1 `crates/ph2d-ecs/` — o modelo
| Arquivo | O quê |
|---|---|
| `instantiate.rs` | `ObjectInstance.orphans: BTreeMap<OverrideKey, Vec<u8>>` (campo NOVO no fim) · componente novo **`LinkedArt`** (marcador) |
| `scene/registry.rs` | `register_default::<LinkedArt>` — **+1 no registo** (ver item 3) |
| `scene/snapshot.rs` · `master.rs` · `sibling_order.rs` · `transform.rs` | `root_key` novo (função) · `propagate_transforms` passa a ordenar raízes por `root_key` e filhos por `sibling_key` |
| `lib.rs` | re-export de `LinkedArt` / `root_key` |

⚠️ **`sibling_order.rs` + `transform.rs` são a cura do report de z-order do Enio** — a Hierarquia e
a ordem de desenho tinham **duas** respostas. Hoje é **uma porta, dois consumidores**. Quem tocar
em ordem de desenho noutra linha colide aqui.

### 2.2 `crates/ph2d-editor-core/` — chrome
| Arquivo | O quê |
|---|---|
| `action_bus.rs` | **2 variants novos** no `EditorAction` (item 3) + a fila SAIU (item 2.4) |
| `action_bus_queue.rs` | **ARQUIVO NOVO** — só `ActionBus` + impl, re-exportado por `action_bus.rs` |
| `ids/inspector_instance.rs` | 1 const + 1 tabela de 8 (item 3) |
| `ids/menus.rs` | `CTX_MENU_HIER_INSTANTIATE_LINKED` (item 3) |
| `screens/hero/inspector_model_instance.rs` | **ARQUIVO NOVO** — `InspectorInstanceInfo` + `VariantChoice` |
| `screens/hero/menu_rows.rs` · `pre_populate.rs` · `hero.rs` · `lib.rs` | a row nova do menu + re-exports |
| `tests/architecture_panel_wiring_parity.rs` | **gate NOVO** `table_driven_chips_are_registered_too` + a catraca `TABLE_PARITY_PENDING` (item 5) |
| `tests/architecture_panel_loc_cap.rs` | catraca DESCEU: `paint_inspector` 289 → **278** |
| `tests/hr12_widgets_a11y.rs` | `paint.rs` do inspector entra no `PANEL_A11Y_DELEGATE_OK` — ele deixou de nomear um primitivo **porque deixou de pintar** |
| `tests/node_id_collisions.rs` | os ids novos entram no censo |

### 2.3 Painéis e shell
`ph2d-panel-inspector` (7 arquivos, incl. `paint_body.rs` **novo**) · `ph2d-panel-hierarchy/event.rs`
(a row *Instantiate Linked*) · `ph2d-render/registry.rs` + `ph2d-script/registry.rs` (os dois
espelhos do contador) · `ph2d-component-desc/catalog/core.rs` (descritor do `LinkedArt`) ·
`ph2d-physics-ecs/src/bin/physics_ecs_c9/` (**lane nova** `instances.rs`, item 6) ·
`shells/desktop/` (30 arquivos, quase todos `instance_*`).

### 2.4 ⚠️ Dois cortes de LOC — movimentação, e ambos DESENHADOS para isolamento
1. **`action_bus.rs` 708 → 658.** Saiu a **FILA** (`ActionBus` + impl, 54 linhas **do FIM** do
   arquivo). ⛔ O corte óbvio seria tirar o `EditorAction` (560 das 708 linhas, e é o que cresce) —
   e poria **toda linha paralela que acrescenta uma acção** em conflito textual. A fila fica onde
   ninguém escreve.
2. **`instance_verbs_tests.rs` 677 → 562**, com os gates de autoria de variante em
   `instance_variant_verb_tests.rs` (novo).
⛔ **Nenhuma tolerância foi levantada**; as duas catracas de painel **desceram**.

---

## 3 — Símbolos que podem COLIDIR (mesmo-símbolo — §1.5.5)

### 3.1 Saída de `bash /home/enio/…/PH2D/scripts/collision-surface.sh` (2026-08-27, contra `330582deb`)

```
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        100   (base: 99)
  ⚠   └ tripla do gate               (100, 13, 14)   (base: (99, 13, 14))
    VEC_SCENE_SCHEMA  14 · FLIP_SCHEMA  13 · DOC_VERSION (timeline)  18   (sem mudança)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                                —   (base: —)
  ⚠ ph2d-render (espelho)                  79   (base: 78)
  ⚠ ph2d-script (espelho)                  79   (base: 78)
▸ CONTRATO CONGELADO (§6)   node.rs intocado · tool.rs intocado
▸ ADR   último 0167 · próximo livre 0168 · esta linha NÃO cria ADR
▸ Cargo.lock   nenhum '+name' novo
▸ MARCADORES DE CONFLITO   nenhum
▸ TETOS DE LOC   nenhum arquivo da linha passa do teto
```

### 3.2 ⚠️ Onde a sonda é CEGA, medido à mão

- **O contador do `ph2d-ecs` sai como `—` nos dois lados.** Ele existe e MOVEU:
  `crates/ph2d-ecs/src/scene/registry_tests.rs:159` — **77 → 78**. ⇒ **os TRÊS contadores mudaram**
  (`ecs` 77→78 · `render` 78→79 · `script` 78→79), e **têm de somar juntos**. *A sonda não vê o
  primeiro; um handoff que a copiasse de memória mandaria o integrador conferir dois de três.*

### 3.3 Números que se CONTAM, nunca se escolhem

| Símbolo | Base | Esta linha | Nota |
|---|---|---|---|
| `PROJECT_SCHEMA` | 99 | **100** | ⚠️ **TRÊS sítios** (`project_schema.rs` + a escada + a tripla em `project_schema_tests.rs`). **Sem degrau de migração**, decisão do Enio de 26/08 (não há projetos gravados); o bump fica porque o postcard é posicional. ⚠️ Se outra linha também subir, o valor certo é `99 + n`, contado — **não** o de nenhum dos lados. |
| Registo de componentes | 77/78/78 | **78/79/79** | +1 = `LinkedArt`. Os três mexem juntos. |

### 3.4 `NodeId` novos (o censo `node_id_collisions` já os cobre)

```
INSP_INSTANCE_CLEAR_ORPHANS            hash_node_id("insp_instance_clear_orphans")
INSP_INSTANCE_VARIANT[0..8]            hash_node_id("insp_instance_variant_0".."_7")
MAX_INSTANCE_VARIANTS = 8              (teto de TABELA DE IDS, não do catálogo)
CTX_MENU_HIER_INSTANTIATE_LINKED       hash_node_id("ctx_menu_hier_instantiate_linked")
```

### 3.5 Variants de enum novos (append-only, **no fim** de cada enum)

```
EditorAction::InspectorClearUnusedOverrides { root_bits: u64 }
EditorAction::InspectorSwapVariant { root_bits: u64, master: u64 }
HierarchyEvent::HierInstantiateLinked { row: NodeId }
```
Tipos novos e privados da shell (fora de disputa): `instance_variant::{SwapRefusal, SwapReport}`,
`instantiate::ArtLink`, `hierarchy_duplicate::DuplicateKind`, `instance_structure::StructureReport`.

### 3.6 ⚠️ A lista `TABLE_PARITY_PENDING` é uma CATRACA partilhada

`crates/ph2d-editor-core/tests/architecture_panel_wiring_parity.rs` ganha uma lista com **9**
entradas, de **4 painéis de outras linhas** (`bgremoval` · `color-equalization` · `painter-layers`
· 1 do `inspector` que não é desta linha). ⛔ **Ela só encolhe.** Uma linha paralela que registe um
desses chips tem de **apagar a linha correspondente** — o gate tem a metade *«já não descreve
nada»* e fica vermelho se ela sobrar.

---

## 4 — Contratos congelados encostados

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` **intocados** (confirmado pela sonda). Esta linha **não cria ADR** ⇒ fora da disputa
do 0168.

---

## 5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **Nenhuma dependência nova** (`Cargo.lock` sem `+name`) ⇒ `machete`/`deny`/`audit` sem superfície
  nova desta linha.
- `cargo fmt --all` rodado; `clippy --all-targets` **limpo** em `ph2d-ecs`, `ph2d-editor-core`,
  `ph2d-panel-inspector`, `ph2d-host-desktop`.
- ⚠️ **`typos`, `fmt` da árvore inteira e RUSTSEC pré-fork não foram medidos** — é a deriva que o
  `main` `330582deb` acabou de pagar (*«os dois ✗ que só o ship vê»*).
- ⚠️ **`physics_ecs_c9` NÃO corre na varredura impactada** (zero menções no log do gate). Ele tem
  uma **lane nova** nesta linha (item 6) e a comparação 3-OS **só o CI mede**. Localmente prova-se
  o que se pode: o binário **corre e é estável** (mesmo hash em 2 de 2 corridas). ⛔ Não há
  baseline a re-capturar — o `spike.yml` compara os **três OS entre si**.

---

## 6 — Ordem, dependências e o que smokar

### 6.1 Ordem
Os 19 commits são **sequenciais e não reordenáveis** (cada fatia assenta na anterior). Não há
dependência de outra linha.

### 6.2 O que o Enio JÁ smokou ✅
instâncias + *Instantiate Linked* · os 3 reports (propriedades ligadas, pontos de shape, z-order) ·
a duplicata invisível · o undo das peças apagadas · o cartão do Inspector · **as variantes + a troca
de versão** · o cartão a dizer *Variant of* e o nome da biblioteca.

### 6.3 O que **NÃO** foi smokado ⚠️
1. **A lane nova do `physics_ecs_c9`** — determinismo entre os 3 OS. Só o CI.
2. **Abrir um `.ph2dproj` gravado antes de 24/08** — tem de migrar `95 → 100`. ⚠️ Um **v97/v98 é
   RECUSADO** de propósito (decisão da `line/Vector`).
3. **Órfãos com o botão *Clear*** — a cena exige apagar uma peça do mestre com uma excepção viva.
4. **Renomear + salvar + reabrir com animação** (o item de 2b533a9d7): gateado, não smokado à mão.

### 6.4 Comando de smoke (⚠️ o caminho é o da worktree)
```
cargo run -p ph2d-host-desktop --release
```
Diagnóstico: `PH2D_INSTANCE_LOG=1`.

---

## 7 — ⚠️ CORREÇÕES ao `CLAUDE.md §5` que a integração TEM de aplicar

O bullet **Aberto** de *Componentes / instâncias* está errado em três pontos, e **dois deles mandam
reconstruir trabalho já pago**:

1. ⛔ **«a F1 continua PELA METADE: … a timeline ainda não — renomear um objeto animado desliga o
   binding»** — **FALSO nas duas metades.** A 5a (física) fechou em 24/08 e a 5b (timeline) em
   25/08; o plano já o registava e o roteador não. Medido e **gateado** por
   `renaming_an_animated_object_does_not_unbind_it` + `a_stranger_with_the_old_name_does_not_capture_the_animation`.
   *A frase pôde envelheceu três dias porque nenhum gate a contradizia.*
2. ⛔ **«A cópia profunda SALTA quatro componentes»** — são **três** desde a F4.6a (o `VecPathRef`
   passou a ser clonado pela porta dos documentos possuídos).
3. ⏳ **«F4.6 (o `VecInstance` subsumido) e F4.7»** — a **F4.7 FECHOU** (os 3 smoke-gates), e a
   **F4.6c DESBLOQUEOU**: das três features que só o vetor tinha, `Swap` e `UpdateMain` passaram a
   existir no mecanismo geral e as variantes também — **menos os EIXOS de propriedade**
   (`Size=Small, State=Idle`), que é a fatia que a F4.6c passou a conter.
4. ⏳ **«nada na tela MOSTRA que campo está overridado»** — **fechado**: é o cartão do Inspector.

### A UMA LINHA proposta para o §5 (item 8 da §1.5.9)

> **Aberto:** ⭐⭐ **as INSTÂNCIAS têm variantes** (F5 critério 2 — *Make Component* sobre uma cópia
> faz uma **variante** que segue a base; a troca base↔variante **preserva as excepções** por re-key
> lido dos próprios elos, sem nomes nem heurística) e o **cartão no topo do Inspector** diz o que a
> cópia é (*Instance* / *Variant of*), o que ela possui e os órfãos ·
> ⏳ **F5 critério 4** (*Apply to inner master* apagar o override nos níveis intermediários) e a
> troca para mestre **NÃO aparentado** (3 modos + relatório, ⛔ nunca automática) ·
> ⏳ **F4.6c DESBLOQUEADA** e passou a **conter uma fatia**: portar os **eixos de propriedade** do
> `vec_variants.rs` para o cartão geral **antes** de apagar os 24 ficheiros do `VecInstance` —
> *um porte que apaga uma feature não é um porte* · ⏳ **F6–F8** ·
> ⛔ **a pose de repouso de uma peça DINÂMICA não propaga, e é DECLARADO** ·
> ⚠️ **`hit_indexed_ids_are_registered` era CEGO aos chips guiados por TABELA** (ele só lê
> `.register(ids::LITERAL, …)`); o gate irmão novo `table_driven_chips_are_registered_too` fecha-o,
> com **catraca de 9 tabelas por registar em 4 painéis de outras linhas** — ela **só encolhe**.
> **Ler:** [handoff de 27/08](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_F5_2026-08-27.md)

---

## 8 — A narrativa: o que esta linha mediu, e o que o plano não dizia

*(o mecanismo detalhado vive em `docs/Components/05_plano_de_implementacao.md`, §F5 e §F4)*

### 8.1 As leis que custaram um vermelho cada
- ⭐⭐⭐ **A cadeia de variantes JÁ propagava** — medido por sonda **antes** de escrever código: uma
  variante é um `MasterRoot` que também é `InstanceOf`, e o sync já procurava toda entidade com elo
  para um mestre vivo. Editar a base alcança a variante **e as instâncias dela num passe**.
  ⇒ o mecanismo custou **zero**; faltavam o **gesto** e o **re-key**.
- ⭐⭐ **O mapa de re-key já vivia no mundo: são os próprios elos.** ⛔ Sem nomes, sem caminhos, sem
  heurística — é o que separa isto do `ByName`/`ByHierarchy` do Unity.
- ⚠️ **A troca tem de ESQUECER o eco** das peças do mestre novo, senão a diferença contra o mestre
  NOVO lê-se como *«a instância mexeu-se»*. Mesmo mecanismo do *Revert*.
- ⚠️ **Uma chave de override SEM imagem fica como está** — apagá-la perdia a excepção antes de o
  `entomb` da F5.3 lhe serializar os bytes.
- ⛔ **Uma variante NÃO pode perder uma peça da base** (o passe estrutural põe-na de volta) — a
  regra do Unity dita por outro caminho, e uma 1.ª versão de gate escolheu a direcção impossível.
- ⚠️ **O cartão nomeava a RELAÇÃO e nunca o que o objeto É**, e o report do Enio nomeou-o:
  *«Instance of "ele mesmo"»*. Duas curas: a **biblioteca** ganha o nome (`"<base> Variant"`) e o
  que fica na **tela mantém o nome do artista**; e a palavra passa a distinguir *Instance* de
  *Variant*.

### 8.2 ⚠️ Fixturas que mordiam ANTES de medirem o produto (quatro, e a lição de cada uma)
1. **O eco sai da fixtura com o mundo** — deitá-lo fora entre passes cai na regra do 1.º encontro
   e apaga a excepção da variante. *Dois vermelhos.*
2. **Dois `SimWorld` novos alocam `StableId` do mesmo contador** — o «estranho» nasceu com
   **exactamente** o id do herói e o gate reprovou sobre produto correto.
3. **`bindings()[0]` numa lista que a purga esvaziou** — *um gate que só sabe ler um dos desfechos
   certos reprova metade deles.*
4. **Uma fixtura plana** não distingue *«aterra no pai certo»* de *«aterra na raiz»* — o gate que
   mata essa mutação tem uma peça **neta**.

### 8.3 ⭐⭐⭐ O achado repo-wide
**`hit_indexed_ids_are_registered` é cego a chips guiados por tabela.** A mutação que apagava o
`populate` deles **SOBREVIVEU** — e a leitura só foi honesta porque o **controlo do filtro** veio
primeiro: a 1.ª corrida deu *«ok»* sobre **zero** testes, porque o gate vive noutra crate.
⇒ gate novo + catraca. ⭐ A metade *«só encolhe»* apanhou o próprio autor: **duas** das 11 entradas
iniciais não descreviam nada.

### 8.4 Prova de mutação
**12 mutações, 12 mortes** — e **uma delas só morreu depois de o gate que a vê existir**, que é a
leitura honesta: a nº 7 (apagar o `populate` dos chips) **SOBREVIVEU** contra o
`hit_indexed_ids_are_registered` e só cai contra o `table_driven_chips_are_registered_too`. *Contar
12/12 sem dizer isto seria contar a morte de uma mutação para o gate errado.*

Sete na fatia das variantes, três no cartão/nome, duas no substrato de identidade da timeline — a
última com o controlo do filtro feito **só sobre os gates novos**, senão a morte podia ser de um
gate que já existia.

---

## 9 — Estado do gate batched

```
NO_FAIL_FAST=1 CARGO_INCREMENTAL=0 bash scripts/nextest-impacted.sh
Summary [27.406s] 11377 tests run: 11377 passed, 1226 skipped
```
⚠️ **A 1.ª corrida (com fail-fast) parou em 3 384 com 7 993 por correr**, num ✗ da flake abaixo — e
um resultado assim **não diz** se o resto está verde. A corrida que vale é a de cima.
`cargo fmt --all` ✅ · `clippy --all-targets` ✅ nas 4 crates tocadas.
⚠️ **Flake conhecida e PRÉ-EXISTENTE** encontrada nesta jornada:
`flip_smooth::resample_measurement::precisao::orcamento::{a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget, the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke}`
— ⚠️ **DOIS membros diferentes em duas corridas do mesmo binário**, que é a assinatura de carga e
não de lógica. Membros **nomeados** da família de flakes de recurso sob fan-out (`CLAUDE.md` §5.0). Verde sozinho
(`--test-threads=1`), commit sem uma linha no módulo dele. ⇒ **re-rode sozinho antes de suspeitar
do merge.**

---

## 10 — Aguardando

Linha **fechada**. Não integra, não faz ship, não pusha. Aguardo ordem.
