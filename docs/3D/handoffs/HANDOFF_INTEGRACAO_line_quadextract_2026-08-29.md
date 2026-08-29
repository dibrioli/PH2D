# HANDOFF DE INTEGRAÇÃO — `line/quadextract` (2026-08-29)

> **Entregável de fecho** ([DIRETRIZ §1.5.9](../../IntegracaoMultiAgente/DIRETRIZ.md) ·
> [`CLAUDE.md §0.7`](../../../CLAUDE.md)). A linha **NÃO integra e NÃO pusha** — este documento
> vai para o **agente integrador**, por ordem explícita do Enio (29/08).
>
> ⚠️ **O mecanismo das waves vive no [handoff de 28/08](HANDOFF_INTEGRACAO_line_quadextract_2026-08-28.md)**
> (§8-bis..§8-nonies, §9/§9-bis) e no [de 26/08](HANDOFF_INTEGRACAO_line_quadextract_2026-08-26.md).
> Este aqui é o **mapa da integração** — o que colide, o que só o `ship.sh` vê, e o que ficou aberto.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/quadextract` |
| HEAD | `git rev-parse line/quadextract` — ⚠️ **este fecho É o último commit do ramo**, então um sha escrito aqui apontaria sempre para o commit anterior a si próprio |
| merge-base com `main` | `330582deb` (*fix(ship): os dois ✗ que só o ship vê*) |
| commits | **43** |
| ficheiros no diff | **87** |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-quadextract` |

⚠️ **Esta linha carrega DUAS jornadas.** Os primeiros ~14 commits são a obra dos **arcos no sistema
dos fechos** (`ph2d-gridmap`, fechada e **desligada**, `PH2D_GRIDMAP_ARCLINE`); os restantes são a
caça ao defeito que o Enio fotografou (idempotência · almofada · mordida · agulha · acabamento).
*O integrador não precisa de as separar — elas não se cruzam em ficheiro nenhum.*

---

## §2 — Foundational / partilhado tocado, e por quê

### ⭐ `crates/ph2d-mesh/` — **APPEND-ONLY, e o caminho antigo é byte-idêntico**

| símbolo novo | ficheiro | forma |
|---|---|---|
| `pub type Sizing<'a>` | `collapse.rs` | `Option<&dyn Fn([f32;3]) -> f32 + Sync>` |
| `pub fn collapse_in_sphere_sized` | `collapse.rs` | a função antiga **delega** com `sizing = None` |
| `pub fn refine_in_sphere_sized` | `dyntopo.rs` | idem |

⚠️ **A assinatura antiga não se mexeu**, e é isso que mantém os outros consumidores da `ph2d-mesh`
(o sculpt, o `ph2d-sdf`) fora do caminho: `collapse_in_sphere(..)` é hoje uma linha que chama a
`_sized` com `None`, e o `None` faz o `limit2` sair exactamente do `edge_min` de antes.
⛔ **Nenhum outro consumidor foi tocado** — o único chamador das portas `_sized` é o
`ph2d-remesh-iso`, e ele passa `None` no caminho de omissão (a cerca nasce desligada, §5).

### `shells/desktop/`

| ficheiro | o que mudou |
|---|---|
| `Cargo.toml` | **`rayon = "1"`** — dep NOVA para esta crate (⚠️ ver §5) |
| `sculpt3d_history_retopo_extract.rs` | o botão: alvo por CONTAGEM, reparo de entrada, `catch_unwind`, `worse` por `open_edges` |
| `sculpt3d_retopo_target.rs` · `sculpt3d_retopo_rulers.rs` | **ficheiros novos** — o corte por HR-18 (§7) |
| `sculpt3d_history_retopo_extract_tests.rs` | os gates do acima |
| `sculpt3d_history_remesh.rs` · `sculpt3d_history_retopo_global.rs` · `sculpt3d_quad_shape.rs` | `QuadRemeshReport` ganha `mirrored` e `doublets`; a linha do log nomeia-os |
| `sculpt3d_scenes_quad.rs` | o **roteiro** da cena `=35` (passos 0, 4 e 8) |
| `sculpt3d_photo_probes.rs` + `_rulers` + `_button` + `_measure` | as sondas, e o corte por HR-18 (§7) |

### `CLAUDE.md`

**`+2 / −0`** — duas linhas no §5, ambas no bullet *3D / Sculpt*. ⚠️ **Nenhum parágrafo de jornada**
(DIRETRIZ §1.5.9 item 8). ⛔ **Conflito garantido** se outra linha também editar aquele bullet: as
duas adições são no MESMO parágrafo, e o resíduo textual é para o Mergiraf.

---

## §3 — Superfície de colisão (saída de `collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/quadextract contra main
  merge-base 330582deb   ·   42 commit(s)   ·   82 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         99   (base: 99)
      └ tripla do gate               (99, 13, 14)   (base: (99, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  78   (base: 78)
    ph2d-script (espelho)                  78   (base: 78)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3)
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
  ✗  1310 / 700   crates/ph2d-quadextract/examples/chain_info.rs
  ✗   737 / 600   shells/desktop/src/sculpt3d_history_retopo_extract.rs
  ✗  1134 / 600   shells/desktop/src/sculpt3d_photo_probes.rs
```

⚠️⚠️ **PRAZO DE VALIDADE (DIRETRIZ §1.5.9 item 3):** esta tabela mede contra o `main` de **29/08**.
**Re-rode `collision-surface.sh` imediatamente antes de fundir** — a divergência entre as duas
leituras é ela própria um achado.

⭐ **Os DOIS últimos `✗` de LOC estão CURADOS** neste fecho (§7) — a tabela acima é a leitura de
**antes** do commit de fecho, colada de propósito porque é ela que motiva a §7. O `chain_info.rs`
continua ✗ e **não tem gate** (nenhuma varredura cobre `examples/`); ele já chegava **acima** do
tecto na base (`953 / 700`) — ver §6.

### Símbolos novos que podem colidir (a lista curta)

⛔ **Nenhum id numérico, nenhum schema, nenhum variant de enum partilhado, nenhum token.** O que
esta linha acrescenta ao espaço de nomes global são **env vars** e **módulos internos**:

| env var NOVA | dono | omissão |
|---|---|---|
| `PH2D_ISO_ADAPT` | `ph2d-remesh-iso` | **desligada** (cerca medida e recusada) |
| `PH2D_ISO_FACING` | `ph2d-remesh-iso` | **desligada** (idem) |
| `PH2D_F1_TARGET` | shell | **desligada** (idem) |
| `PH2D_EXTRACT_MIRROR` | `ph2d-quadextract` | **ligada** (`=0` bissecta) |
| `PH2D_PIECE` · `PH2D_DETAIL` · `PH2D_PRESSES` · `PH2D_PROBE_LOCAL` · `PH2D_REF` | sondas `#[ignore]` | — |

Módulos novos (todos `mod` privado, nenhum na API pública de crate nenhuma):
`ph2d-quadextract::doublets` · `ph2d-remesh-iso::{sizing, project}` ·
`shell::sculpt3d::{retopo_target, retopo_rulers, photo_{rulers,button,measure}}`.

⚠️ **UMA porta pública mudou de dono, com a assinatura intacta:**
`ph2d_quadextract::repair_doublets` era reexportada de `cells` e passa a sê-lo de `doublets`.
Nenhum chamador nota; um merge que traga um `pub use cells::repair_doublets` de outra linha
**duplicaria o símbolo** — é o único ponto de mesmo-símbolo desta linha.

---

## §4 — Contratos congelados encostados

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest`, `Tool`/`RasterEditTool`/`CanvasPaintTool`, a
superfície do `ph2d-vector-doc` e os quatro schemas estão **intocados** (§3). ⇒ **não exige ADR.**

---

## §5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

| risco | estado |
|---|---|
| **`cargo fmt --all`** | ⭐ **ERA O RISCO REAL, e está curado.** No `main` dava `0`; nesta worktree dava **40 pontos em 32 ficheiros** (§7) |
| **`cargo machete`** (dep declarada e não usada) | ⚠️ **`rayon` é NOVA em `shells/desktop/Cargo.toml`** e É usada (`sculpt3d_history_retopo_extract.rs`, as duas tentativas em paralelo). O machete deve passar — mas é o único candidato desta linha |
| **`cargo deny` / `cargo audit`** | ⭐ sem risco: `Cargo.lock` não ganhou **nenhum pacote externo** (o `rayon` já estava na árvore) |
| **`typos`** | ⚠️ não corrido aqui; o diff tem muito texto em PT com acentuação |
| **clippy latente** | ⭐ `0` avisos nas 8 crates da linha, `--all-targets`, `--release` |
| **`doc-index.sh --check`** | ⚠️ **este fecho acrescenta 1 doc** a `docs/3D/handoffs/`, cujo índice é **à mão** — actualizar |

---

## §6 — Ordem, dependências, e o que smokar

### Ordem
Os 43 commits são **lineares e independentes entre si** — nenhum precisa de outro fora da própria
linha. ⭐ Um `--ff-only` resolve tudo se nenhuma outra linha tocar os ficheiros da §2.

### O que **NÃO** foi smokado
- ⛔ **O `PH2D_GRIDMAP_ARCLINE`** (a obra dos arcos, ~14 commits) — nasce **desligado** e nunca foi
  visto pelo Enio. A saída do botão com ele desligado é a mesma de antes.
- ⛔ **As três cercas desligadas** (`PH2D_ISO_ADAPT`, `PH2D_ISO_FACING`, `PH2D_F1_TARGET`) —
  medidas, recusadas, com a tabela ao lado. *Ligar qualquer uma parte a cadeia*, e está documentado.
- ⚠️ **O motor `Fast`** do menu (`RetopoMode` local) não foi re-medido desde 28/08: na peça do
  artista devolve `437` quads + **`150` não-quads** contra `1 494` e `100 %` do de omissão. *Ele
  fica a um clique, com o nome que um artista escolhe depois de ouvir que o bom é lento* — a
  pergunta de produto (removê-lo?) está com o Enio **sem resposta**.

### O que smokar depois de integrar
```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=35 cargo run -p ph2d-host-desktop --release
```
1. **Ctrl+Shift+O** → o `.obj` do artista. (⚠️ arrastar **não** funciona: o Wayland não entrega
   `DroppedFile`, medido em `sculpt3d_import.rs`.)
2. Crase (`` ` ``) → secção **Topology** → **Quad Retopology**.
3. O terminal escreve o roteiro numerado inteiro na abertura.
4. **Clicar duas vezes seguidas**: a contagem de quads tem de ficar parada (a régua da
   idempotência, §8-ter de 28/08).

---

## §7 — ⛔⛔⛔ O QUE ESTE FECHO CUROU, e é a razão de ele existir

O pedido foi *«deixe pronto para usar»* — **a primeira vez nesta linha que alguém pergunta pela
ÁRVORE em vez de pela crate**. Caíram quatro vermelhos que **nenhum** dos portões desta linha via.

### A causa é UMA, com TRÊS variantes de filtro

Um gate que **VARRE** vive na crate onde a **regra** mora, nunca onde o ficheiro mora:

| gate | onde vive | que filtro o esconde |
|---|---|---|
| `workspace_src_files_under_loc_cap` | `ph2d-editor-core/tests/` | `-p ph2d-quadextract` |
| `shell_files_respect_hr18_loc_cap` | `shells/desktop/tests/` | **`--bins`** (é um `--test`!) |
| `cargo fmt --all -- --check` | a árvore | `cargo fmt -p <crate>` |

⚠️⚠️ **A 3.ª variante é a mais traiçoeira e é NOVA:** `cargo test -p ph2d-host-desktop --bins`
corre 3 834 testes e **não toca** em `shells/desktop/tests/`. *Um portão pode correr quase quatro
mil testes da crate certa e não alcançar o gate dela.*

### O que estava vermelho

| ficheiro | base | antes do fecho | tecto |
|---|---|---|---|
| `ph2d-remesh-iso/src/lib.rs` | `~600` | **`875`** | `700` |
| `ph2d-quadextract/src/cells.rs` | `~520` | **`758`** | `700` |
| `shells/.../sculpt3d_history_retopo_extract.rs` | `480` | **`737`** | `600` |
| `shells/.../sculpt3d_photo_probes.rs` | `581` | **`1134`** | `600` |
| `cargo fmt --all` | `0` pontos | **`40` pontos / `32` ficheiros** | `0` |

### A cura: CINCO splits por responsabilidade (⛔ nunca allowlist)

A fronteira de cada corte é **uma pergunta por módulo**, e o sinal de que ele é o certo é as duas
metades **cruzarem-se num sítio só**:

| ficheiro (antes → depois) | irmão novo | a pergunta que o irmão responde |
|---|---|---|
| `remesh-iso/lib.rs` `875 → 573` | `sizing.rs` | *qual é o alvo **AQUI**?* |
| | `project.rs` | *onde é que este vértice **POUSA**?* |
| `quadextract/cells.rs` `758 → 524` | `doublets.rs` | *que vértice **não devia existir**?* |
| `retopo_extract.rs` `737 → 500` | `retopo_target.rs` | *qual é o alvo, e onde é mais fino?* |
| | `retopo_rulers.rs` | *esta saída é **pior** que a anterior?* |
| `photo_probes.rs` `1134 → 188` | `photo_measure.rs` | *ele diz que a ponta sumiu — **quanto**?* |
| | `photo_rulers.rs` | *com que **régua** se mede isso?* |
| | `photo_button.rs` | *e pela **porta do produto**, o que sai?* |

⚠️ **Detalhes que o integrador deve saber:**
- O `dot` **fica** no `remesh-iso/lib.rs` (tem 2.º consumidor fora da projecção) e o `project.rs`
  importa-o. Mover teria sido arrumação a fingir de desenho.
- Em `retopo_extract.rs` os `use rulers::{…}` / `use target::{…}` são **load-bearing**: o `mod tests`
  irmão chama tudo por `super::worse` / `super::boundary_edges`, e um nome trazido por `use`
  continua alcançável por `super::`.
- As sondas movidas continuam a correr (`photo_probes::measure::what_do_the_photos_measure`) — o
  **nome do teste mudou de caminho**, então um `--bins <filtro>` antigo pode deixar de casar.

### E o `chain_info.rs` fica ✗, declarado

`crates/ph2d-quadextract/examples/chain_info.rs` está a **`1310 / 700`** e **nenhum gate o cobre**
(as duas varreduras olham `crates/*/src/**` e `shells/desktop/src/**`; `examples/` não é território
de nenhuma). ⚠️ Ele **já chegava acima do tecto na base** (`953`), e esta linha levou-o a 1310.
*Não é regressão desta jornada, é dívida que ela aumentou* — e fica aqui escrito porque
`collision-surface.sh` o mostra e o integrador vai perguntar.

---

## §8 — ⛔⛔⛔ ABERTO — o report do Enio de 29/08, DEPOIS deste trabalho todo

> *«buracos nas pontas. faces emboladas nas pontas.»* — duas fotos, três setas verdes: a ponta
> superior direita, a do meio e a inferior.

⚠️ **É um report NOVO sobre a build de hoje, e NÃO está curado.** Ele vem depois de a mordida, a
almofada e a idempotência estarem fechadas — ou seja, **é o que sobra**, e as fotos separam-no em
duas queixas que podem ser dois defeitos:

| o que se vê | a régua que o mede hoje | quem já a corre |
|---|---|---|
| **buraco** na ponta | `boundary_edges` / `open_edges` | o `worse` do botão, e a sonda `the_artists_piece_through_the_button` (ela imprime `furo #N` com o centro) |
| **faces emboladas** na ponta | ⛔ **NENHUMA** — `QuadShape` mede aspecto/enviesamento **medianos**, e três quads emaranhados na ponta de um espinho não movem uma mediana de milhares |

⭐⭐ **A primeira coisa que a próxima janela deve fazer NÃO é código, é uma régua:** um censo
**local** de forma na vizinhança de cada vértice de alta curvatura. *É exactamente a lição que esta
linha já pagou duas vezes* — o `edge_max` global não via o quad de `0,02 × 0,30`, e o `χ` não via a
almofada. **Uma queixa que nenhuma régua vê não tem como ser fechada.**

⚠️ **E há uma hipótese com endereço**, escrita para não se perder: o que o §8-octies de 28/08 mediu
é que a **fase zero entrega ao campo cruzado uma malha que ele sabe ler** só enquanto ela é
isotrópica; na ponta de uma agulha o F1 **não** consegue ser isotrópico e fino ao mesmo tempo sem a
cerca que foi recusada. ⇒ as duas queixas podem ser o **mesmo** mecanismo que a wave do **factor de
escala conforme** (`Δ log h` contra a curvatura de Gauss, `h = h₀·e^{−s}`) existe para resolver.
⛔ **Não construir a cura antes de ter a régua** — a família de saídas já custou duas construções
medidas e recusadas.

### O resto do aberto (herdado, sem alteração)

- ⏳ o **ápice** de um espinho sculptado: nenhuma malha finita representa um ponto — *feature-preserving
  remeshing* (pinar o canto), outra wave, outra régua.
- ⏳ o `Follow Curvature` **não tem consumidor** no motor de omissão.
- ⏳ o motor **`Fast`** a um clique, com a saída pior (§6).
- ⏳ `PH2D_GRIDMAP_ARCLINE` desligado, com o plano em
  [`PLANO_arcos_no_sistema_dos_fechos.md`](../quad-remesh/PLANO_arcos_no_sistema_dos_fechos.md).

---

## §9 — Portão de fecho (a árvore, não a crate)

| | |
|---|---|
| `cargo fmt --all -- --check` | ⭐ limpo (era **40** pontos) |
| `cargo test -p ph2d-editor-core --tests` (62 binários) | ⭐ `0` reprovados |
| `cargo test -p ph2d-host-desktop` (bins **e** tests) | ⭐ `0` falhas |
| `file_loc_caps` · `workspace_file_loc_cap` · `widget` · `panel` | ⭐ verdes (dois eram **✗**) |
| `cargo test` nas 7 crates da linha | ⭐ `0` falhas |
| `cargo clippy --all-targets --release` nas 8 | ⭐ `0` avisos |
| `cargo check --workspace --all-targets` | ⭐ limpo |
| `scripts/cleanroom-sweep.sh` sobre todo o diff | ⭐ limpo (vassoura de 56 entradas) |
| app abre e imprime o roteiro | ⭐ verificado (`PH2D_SCULPT3D_SMOKE=35`) |


⚠️ **UMA reprovada na corrida cheia, e é FLAKE DE CARGA já catalogada:**
`only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos de áudio) está **nomeada no
`CLAUDE.md` §5.0** como membro confirmado da família. Verde **3 de 3** sozinha com
`--test-threads=1`, e o diff desta linha toca **zero** ficheiros de áudio. ⇒ *re-rode antes de
olhar para o commit*, como a regra manda.

⚠️ **`target/*/incremental` reclamado** antes de parar (DIRETRIZ §1.5.9 item 7).

---

## §10 — Resumo colável

> Linha `quadextract` pronta (`git rev-parse line/quadextract`, 43 commits sobre `330582deb`).
> **Foundational:** só `ph2d-mesh` **append-only** (`Sizing` + duas portas `_sized`; as antigas
> delegam com `None` ⇒ byte-idêntico) e `shells/desktop` (o botão de retopologia + 5 módulos novos
> por HR-18). **Colisão:** nenhum schema, nenhum id, nenhum ADR, nenhum contrato congelado, nenhum
> pacote externo novo; o único mesmo-símbolo possível é `repair_doublets` ter mudado de módulo
> reexportador. **`CLAUDE.md`:** `+2 / −0`, no bullet *3D / Sculpt*. **Só o ship vê:** `typos` e o
> `machete` sobre o `rayon` novo do shell. **Aberto:** o report do Enio de 29/08 (buracos e faces
> emboladas nas pontas) — **sem régua que o veja**, e é isso que a próxima janela deve construir
> primeiro. Aguardo ordem de integração.
