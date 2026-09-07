# HANDOFF DE INTEGRAÇÃO — `line/components`, 2026-09-07

> **O PREFAB ABERTO** (as quatro portas · o vidro jateado · o palco · a barra que tranca)
> **e O PASSO FANTASMA** (o `Ctrl+Z` que o dono viu não funcionar na reordenação da Hierarquia).
>
> A linha fecha aqui e **PARA** ([`CLAUDE.md §0.7`](../../../CLAUDE.md), DIRETRIZ §1.5.9).
> ⛔ Não integrei, não fiz `push`, não corri `foundational-integrate.sh`.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/components` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-components` |
| último commit de **CÓDIGO** | `75fd9871c500bc9eca42adb5024041ddfaf11856` |
| HEAD do ramo | este documento, **um** commit acima do anterior (só `.md`) |
| merge-base com `main` | `815555aed` |
| commits | **29** de código + este handoff |
| diff | **101 ficheiros**, `+7 520 / −840` |
| testes novos | **90** `#[test]` |

**Smoke compilado** (item 9 da §1.5.9), o comando **byte a byte** o que foi entregue ao dono:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && cargo run -p ph2d-host-desktop --release
```

2.ª corrida da prova:

```
    Finished `release` profile [optimized] target(s) in 0.18s
```

zero linhas `Compiling`. ⚠️ Ela foi tirada **depois do último commit de código** (`75fd9871c`) e
**depois** de reclamar o `incremental/` — qualquer edição de fonte posterior a invalida em
silêncio. As edições que vieram a seguir são só deste `.md`, que não está em crate nenhuma.

---

## 2. Foundational / partilhado tocado, e porquê

Tudo **aditivo**, exceto o item marcado ⚠️.

### `crates/ph2d-render` (foundational de desenho)

| ficheiro | o quê |
|---|---|
| `frost.rs` · `shaders/frost.wgsl` (**novos**) | o passe de **vidro jateado**: quatro passagens (down → blur H → blur V → up+véu) a meia resolução, quatro *uniform buffers* porque as quatro entram no **mesmo** `submit`. `pub fn run(..)` · `pub fn work_size(size) -> (u32,u32)` (nunca zero) · `pub const FROST_VEIL_ALPHA: f64 = 0.35` · `pub const FROST_VEIL_DIM: f64 = 0.35`. |
| `sprite_collect.rs` | `collect_sorted_instances` ganha `held_back: Option<&BTreeSet<Entity>>` — **`pub(crate)`**, não atravessa a crate. |
| ⚠️ `renderer_draw.rs` | **`SpriteRenderer::render_with_streams` — que é `pub` — ganhou um parâmetro no FIM** (`held_back`). Ver §3. |
| `lib.rs` | dois `mod` novos. |

⭐ **O que tornou o vidro possível, e é a nota que uma leitura rápida do diff perde:** os painéis
**não estão** no `world_rt` — eles vivem na cena Vello do chrome, composta depois. Logo *«borrar o
canvas e tudo o que está nele, e mais nada»* **é exactamente o conteúdo daquela textura**: sem
máscara, sem conhecimento de layout, sem uma segunda resposta a *«onde está o canvas?»*.

⚠️ **`held_back` é por ENTIDADE e não por faixa de rank, e a razão é medida:** o rank canónico
**não** garante corrida contígua para uma sub-árvore (o próprio `sprite_collect` já o diz a
propósito do agrupamento de recorte). Uma janela de rank levantaria uma sprite alheia junto com a
receita, e o artista veria um objecto estranho nítido por cima do vidro.

⚠️ **E é uma RETENÇÃO, não uma ocultação:** quem a passa fica **obrigado** a desenhar aquelas
entidades noutra passagem. A alternativa — desenhar duas vezes e deixar a nítida por cima — deixa
o borrão da peça a escapar por fora da silhueta dela, como um halo.

### `crates/ph2d-vec-scene` (foundational do documento vetorial)

`VecViewState` ganha o campo **`isolated: Vec<VecPathId>`** + `is_isolated()` + `isolating()`.
**Puramente aditivo** (`+36 / −0`).

⚠️ **É estado de VISTA, nunca do documento** — o precedente é o `isolated` do modelador 3D.
Escrevê-lo nas formas seria uma **edição**, com passo de undo e bytes no ficheiro, por uma coisa
que só existe enquanto o artista está a olhar.
⚠️ **É uma lista de LEVANTADAS, não de recuadas:** a lei é *«tudo fica atrás do vidro menos
isto»*. Guardar o complemento faria toda forma nova nascer no lado errado.

### `crates/ph2d-vec-render`

`lib.rs` **779 → 570 LOC** por extracção de `dispatch.rs` (novo): `dispatch` (o mundo **menos** a
receita) + `dispatch_isolated` (**só** a receita) + `draw_one`. Corte por responsabilidade, feito
porque o portão de fecho acusou o tecto de 700.

### `crates/ph2d-editor-core` (foundational de UI)

- `screens/hero/prefab_bar.rs` (**novo**) — `PrefabEditView { name, copies }` · `PrefabExit{Done,Cancel}` ·
  `bar_rect` · `done_rect` · `cancel_rect` · `title` · `paint` · `apply_event`.
- `screens/hero/chrome/prefab_bar.rs` (**novo**) — `// ph2d-chrome-sync:z=225`. A lista de `mod` e a
  cadeia `dispatch_all` foram **regeneradas** por `cargo run -p ph2d-chrome-sync`, nunca à mão.
- `ids/chrome/prefab.rs` (**novo**) + ids em `ids/chrome/vector_components.rs`,
  `ids/inspector_instance.rs`, `ids/menus_asset.rs`, `ids/menus_hierarchy.rs`.
- `action_bus*.rs` — três variantes novas (§3).
- `interaction/dispatch/{hierarchy,pointer_down,pointer_up}.rs` + `interaction/state/store_hierarchy.rs`
  — diagnóstico (`diag_on` / `diag_down` / `hierarchy_row_count`), atrás de `PH2D_UNDO_LOG`.

### `shells/desktop`

Ficheiros novos: `prefab_stage.rs` · `canvas_area.rs` · `render_loop/present_frost.rs` ·
`render_loop/present_fx.rs`.
⚠️ **`present.rs` foi de 596 para 511 LOC** por causa do `present_fx.rs` — as passagens de luz
(emissivo de sprite + brilho do Motion) saíram para o irmão com uma `FxGear`. Corte por tecto.

`undo_route.rs` · `input_dispatch/keyboard.rs` · `input_dispatch/keyboard_tail.rs` — o diagnóstico
do `Ctrl+Z` (§5 do mecanismo abaixo).

⚠️ **`render_loop/mod.rs`: o dreno do reparent MUDOU DE SÍTIO** — ver §6, é a mudança de maior
alcance da linha e a única que muda a **ordem do quadro**.

---

## 3. Símbolos que podem COLIDIR

### 3.1 A saída de `collision-surface.sh` (corrida em `line-components`, 2026-09-07)

```
SUPERFÍCIE DE COLISÃO — line/components contra main
  merge-base 815555aed   ·   28 commit(s)   ·   101 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                        121   (base: 121)
      └ tripla do gate               (121, 13, 22)   (base: (121, 13, 22))
    VEC_SCENE_SCHEMA                      —   (base: —)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  82   (base: 82)
    ph2d-script (espelho)                  82   (base: 82)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0168   próximo livre: 0169
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
```

⭐ **Nenhum schema se mexeu, nenhum registo de componente, nenhum contrato congelado, nenhum ADR,
nenhuma dependência externa.** Esta linha está **fora de toda disputa de número**.

⚠️ **A tabela acima é REFERÊNCIA, não EVIDÊNCIA** (§1.5.9 item 3): ela mede contra o `main` de
**2026-09-07**. Re-rode `collision-surface.sh` nesta worktree **imediatamente antes** de fundir; a
divergência entre as duas leituras é ela própria um achado.

### 3.2 ⚠️ A assinatura pública que MUDOU — o único ponto de atenção real

```rust
// crates/ph2d-render/src/renderer_draw.rs
pub fn render_with_streams(
    &mut self, target, present, camera, window, clear_color,
    extra, gpu_extra, scene_viewport,
    rank_window: Option<(u32, u32)>,
    held_back: Option<&BTreeSet<ph2d_ecs::Entity>>,   // ← NOVO, no FIM
)
```

- Chamadores no `main` de hoje: `shells/desktop/src/render_loop/present.rs:226` e
  `present_bands.rs:92` — os **dois** já actualizados nesta linha.
- **Um chamador novo vindo de outra linha não funde mal — ele não compila**, e isso é o desenho
  certo (falha alta). O integrador acrescenta `None` (o caminho de sempre, byte-idêntico).
- ⛔ **Se outra linha também apender um parâmetro a esta função, é colisão de mesmo-símbolo**
  (DIRETRIZ §1.5.5): o `git` funde os dois `+` limpo e a ordem posicional fica trocada **sem erro
  de compilação** se os tipos coincidirem. Grepe `render_with_streams(` nas outras worktrees.

### 3.3 Ids de widget novos (FNV-1a de slug — colisão é de SLUG, não de número)

```
PREFAB_EDIT_DONE                 = hash_node_id("prefab_edit_done")
PREFAB_EDIT_CANCEL               = hash_node_id("prefab_edit_cancel")
VECTOR_COMPONENT_PLACE_LINKED    = hash_node_id("vector.component.place_linked")
VECTOR_COMPONENT_EDIT            = hash_node_id("vector.component.edit")
INSP_INSTANCE_OPEN_PREFAB        = hash_node_id("insp_instance_open_prefab")
CTX_MENU_ASSET_INSTANTIATE_LINKED= hash_node_id("ctx_menu_asset_instantiate_linked")
CTX_MENU_HIER_EDIT_PREFAB        = hash_node_id("ctx_menu_hier_edit_prefab")
```

O gate `node_id_collisions` (em `ph2d-editor-core/tests/`) corre sobre a **árvore combinada** e é
quem apanha um choque real — ⚠️ ele **não** pode ser corrido só nesta worktree para responder pela
integração.

### 3.4 Variantes de enum novas (append-only)

| enum | variante |
|---|---|
| `EditorAction` (`action_bus.rs`) | `InspectorOpenPrefab { root_bits: u64 }` |
| `HierRequest` (`action_bus_hier.rs`) | `EditPrefab { row: NodeId }` |
| (`action_bus_kinds.rs`) | `InstantiateLinked` |

⚠️ Se outra linha apender variantes aos MESMOS enums, o merge é limpo e a ORDEM final é a do
integrador — nenhum destes é serializado, então a ordem não é *load-bearing*. Confira mesmo assim.

### 3.5 Prioridade de chrome

`// ph2d-chrome-sync:z=225` para o `prefab_bar`. Vizinhos ocupados hoje: `240` (`falloff_handle`),
`270`–`273`, `280`, `290`, `300`. ⚠️ **Se outra linha escolher `225`, os dois módulos entram na
cadeia com a mesma prioridade** — a lista e o `dispatch_all` são **gerados**
(`cargo run -p ph2d-chrome-sync`), então a cura é re-gerar depois de escolher outro número, nunca
editar a lista à mão.

### 3.6 Chaves de i18n novas

18 chaves sob o prefixo `panel.vector.component.*` + `panel.vector.section.component`
(`crates/ph2d-i18n/src/vector.rs`). ⚠️ **Uma delas RENOMEIA o que o artista lê:** a palavra do app
para a coisa reutilizável passou a ser **«prefab»** em todas as superfícies (gate
`one_word_for_the_reusable_thing`). Se outra linha escrever «component» numa string de UI nova, o
gate a apanha.

---

## 4. Contratos congelados encostados

**Nenhum.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` **intocados** (medido, §3.1).

---

## 5. O que só o `ship.sh` pega — já corrido nesta worktree

| verificação | resultado |
|---|---|
| `cargo fmt --all -- --check` | ✅ limpo |
| `clippy --all-targets` nas **10** crates que a linha tocou | ✅ **zero** erros, **zero** warnings |
| `typos` sobre os 101 ficheiros da linha | ✅ limpo — ⚠️ **duas ocorrências foram CURADAS no fecho** (`75fd9871c`): uma grafia errada do verbo *instanciar*, dentro de uma citação do dono em doc-comment. Elas vinham de commits de 06/09 e teriam ficado **vermelhas no `ship.sh`**, que é o único portão que corre `typos` — nenhum outro portão desta linha lê ortografia. |
| dependências novas (machete / deny / audit) | **nenhuma** — `Cargo.lock` sem `+name` novo |
| suíte do shell | ✅ `5 429 passaram · 297 ignorados` |
| gate impactado (`nextest-impacted.sh`) | ver §9 |

⚠️ **O que esta worktree NÃO pode responder:** o `node_id_collisions` e os censos de árvore só
valem sobre a **árvore combinada**. E o `physics_ecs_c9` compara os **três SO entre si** — só o CI
o mede.

---

## 6. ⭐⭐⭐ A mudança de maior alcance: **a ordem do quadro**

> Report do dono, 2026-09-07: *«Undo não funciona plenamente — reordenei objetos na hierarquia e
> não funcionou o undo.»*

**Ele estava certo, e o defeito não era o undo.** O dreno do reparent
(`hero_intents::drain_reparent`) corria dentro do `hierarchy::dispatch`, em
`render_loop/mod.rs ~11700` — **~2 340 linhas depois** da projecção de z (`~9342`), que é quem
**lê** a árvore para reordenar a pilha do vetor.

Sequência medida com `PH2D_UNDO_LOG=1`, no app do dono:

1. **quadro N** — a árvore muda (`RootOrder` `[(1,0),(2,1),(3,2)]` → `[(1,0),(2,2),(3,1)]`), mas a
   projecção já correu sobre a árvore **velha** ⇒ a captura do fim do quadro guarda `world` novo
   com `vec` **velho**: `partes: ["world"]`;
2. **quadro N+1** — a projecção lê a árvore nova e reescreve a pilha; **sem entrada**, o passo é
   SUPRIMIDO (*«sem entrada neste quadro»*) e funde-se no seguinte;
3. nasce um passo cujo conteúdo inteiro é `partes: ["vec"]`
   (`base=[0,1,2] atual=[0,2,1] · só a ORDEM=true`) — um **FANTASMA**;
4. `Ctrl+Z` repõe a pilha e **não** a árvore; a projecção do quadro seguinte re-deriva a pilha da
   árvore que ninguém desfez, e **o fantasma renasce**.

No log dele o ciclo repete-se com **a fila parada em `5`**: cada `Ctrl+Z` gasta um passo que o
próprio quadro volta a criar, e o passo REAL nunca chega a ser alcançado.

**CURA:** o dreno passa a correr onde os outros escritores da árvore já moram — ao lado do
`restack` e dos três `assign_missing_*`, **depois do `sync` e antes de a árvore ser lida**. O
`hierarchy::dispatch` continua a receber o parâmetro e vê `None`.
⚠️ **O `take()` é *load-bearing*:** aplicar duas vezes reordenaria duas.

⭐ **A lei não é sobre QUEM escreve, é sobre QUANDO:** *todo escritor da árvore corre antes de ela
ser lida, e a leitura antes da captura.* É a **mesma doença** que o `vec_entities::z_order` já
documenta (*«a captura deixava de ser ponto fixo dos sistemas»*, BUGS #15) a voltar **por outra
porta** — ali era a forma recém-nascida contra a lista do painel, aqui é a árvore reordenada
contra a projecção do mesmo quadro.

**Gate:** `a_hierarchy_drag_leaves_the_capture_a_fixed_point`, em `vec_zorder_fixpoint_tests.rs` —
o ficheiro que **já declarava** essa lei.
⚠️ **A lacuna que deixou o defeito passar era do ARNÊS:** o `Frame` ali diz-se *«o pedaço do quadro
que muta o estado que o undo fotografa»* e **não continha o dreno do reparent**; logo nenhum gate
daquele ficheiro podia ver a ordem do produto. Agora contém (`run_with_drag`), com um **controlo**
de que o arrasto de facto reordenou antes de medir o ponto fixo.
**Prova de mutação:** mover o dreno para depois do `reorder_to` — que é literalmente o produto de
antes desta cura — deixa o gate VERMELHO **no controlo** (`left: [0,1,2] right: [2,0,1]`).
Restaurado por backup + `touch`; verde de novo.

⏳ **ABERTO e nomeado (não medido):** os outros verbos tardios da Hierarquia — apagar, duplicar,
*Remove from Sheet* — escrevem a árvore no **mesmo sítio tardio** e têm a mesma latência
estrutural. **Não foi medido** se produzem o fantasma. A cura geral seria **re-projectar antes da
captura**, e ela paga uma travessia da árvore por quadro; a cura desta linha é cirúrgica e custa
zero.

---

## 7. Ordem / dependências entre commits

Linear, sem dependências fora de si. Três blocos:

1. **`5fa9cb7ee` … `52ff772d4`** — o modelo geral do prefab e as **quatro superfícies** de autoria
   (painel vetorial · menu da Hierarquia · cartão da biblioteca · linha *Instance of «X»* do
   Inspector), mais o conta-gotas do *Swap*.
2. **`376c13e4f` … `184a9c167`** — o **prefab aberto**: isolamento → vidro jateado → o palco →
   a barra *Done/Cancel* → a identidade durável → as réguas do reordenar.
3. **`68a902375` … `88fe0391d`** — o diagnóstico em três estações (arrasto **e** `Ctrl+Z`) e a cura
   da ordem do quadro.

⚠️ **O bloco 2 tem uma dependência interna que uma leitura rápida do diff inverte:** o commit
`e9558fdf5` levava a **CÂMERA** à receita, e o `bbcd5f3f9` **substituiu** essa lei por ordem do
dono (*«o canvas busca a posição inicial do prefab. não deve ser assim»*) — a receita é que vem ao
centro da área visível, como **pré-visualização** (`PreviewDrive`), sem tocar na câmera. Ler os
dois como camadas somadas leva a reconstruir a que foi retirada.

---

## 8. O que smoke-testar (e o que NÃO foi smokado)

### ✅ Smokado pelo dono e aprovado

| gesto | veredito |
|---|---|
| *Edit Prefab* pelas quatro portas, com o vidro jateado e a receita ao centro da área visível | *«smoke ok»* |
| `Done`/`Enter` e `Cancel`/`Esc` na barra, e a barra logo **abaixo** da régua de pixel | *«SMoke OK»* |
| reordenar na Hierarquia + `Ctrl+Z` (a cura da §6) | *«smoke ok»* (2026-09-07) |

### ⏳ NÃO smokado — o integrador ou o dono decide

- **O prefab aberto com uma receita que contém objectos FLIP ou uma escultura 3D.** O `lift` varre
  o `PresentWorld` por entidade, então em princípio é indiferente ao tipo — mas nenhuma cena de
  smoke o encena.
- **O vidro jateado numa janela muito pequena** (`work_size` nunca devolve zero e há gate, mas o
  aspecto do desfoque a meia resolução não foi visto abaixo de ~400 px).
- **A composição do vidro com uma banda de `WorldRt` activa** (`compositor_reads_world`): o
  `present_frost::glass` empilha o mundo **quando não está bandado**, e a rota bandada não foi
  exercida à mão.
- **Os outros verbos tardios da Hierarquia** (§6, aberto).

---

## 9. Portão de fecho — o que correu, e o resultado

| | |
|---|---|
| `cargo fmt --all -- --check` | ✅ |
| `clippy --all-targets` (10 crates da linha) | ✅ zero |
| `typos` (101 ficheiros) | ✅ (duas curadas no fecho — §5) |
| `cargo test -p ph2d-host-desktop` | ✅ **5 429 passaram · 297 ignorados** |
| `CARGO_INCREMENTAL=0 scripts/nextest-impacted.sh` | ✅ **14 444 correram · 14 444 passaram · 0 falharam · 1 469 saltados** (405 s) |
| `rm -rf target/*/incremental` | ✅ **27 GB devolvidos** (`debug/incremental`; `ci-test` e `release` já estavam a `0 B`) |
| build de release ×2 | ✅ 2.ª corrida `Finished … in 0.18s`, zero `Compiling` |

```
     Summary [ 405.365s] 14444 tests run: 14444 passed (4 slow), 1469 skipped
```

⚠️ **Carga da máquina nas corridas de confirmação:** `load ~10`. Nenhum gate desta linha mede
razão de relógio, então ela não é candidata à família de flakes de recurso do `CLAUDE.md §5.0` —
mas se alguma reprovar na árvore combinada, **re-rode sozinha com o `loadavg` ao lado antes de
olhar para o diff**.

---

## 10. Cinco coisas que uma leitura rápida do diff entende ao contrário

1. **O vidro jateado não é uma camada do Vello, é um passe sobre a textura do mundo.** Foi por isso
   que o `isolating()` do `VecViewState` **encolheu** para `!self.isolated.is_empty()`: a metade
   *«e há caixa»* existia porque o recuo antigo era uma camada e uma caixa a zero recortava o mundo
   inteiro. **O renderer deixou de precisar de saber onde o canvas está.**
2. **A receita não é movida — ela é CONDUZIDA.** O `prefab_stage` escreve a pose pelo
   `PreviewDrive`: vê-se, não se grava, não se desfaz. `capture_project` repõe a pose de bastidores
   durante a fotografia. ⚠️ **A lei é *declare todo quadro*** — falhar um quadro faz o `settle`
   esquecer, e o valor de pré-visualização vira documento.
3. **A trava da sessão guarda `StableId`, nunca bits.** A 1.ª versão guardava bits e um `Ctrl+Z`
   dentro da sessão **expulsava o artista** (o undo respawna tudo com bits novos). O gate é
   `the_session_survives_an_undo_step`, e o `hold` **remonta o palco** quando a entidade volta com
   outros bits.
4. **`Cancel` não é «fechar sem gravar».** Ele **restaura** o estado de entrada da sessão pela fila
   de undo, e a captura correspondente entra logo a seguir ao `preview_drive.settle()` — ver
   `undo_app.rs`, `prefab_cancel_pending`. Fechar sem restaurar deixaria as edições dentro.
5. **O `held_back` obriga quem o passa.** Não é um filtro de conveniência: as entidades retidas
   **têm** de ser desenhadas noutra passagem, senão elas desaparecem. `present_frost::lift` produz
   **de uma varredura só** o conjunto retido **e** as instâncias que sobem — duas respostas da
   mesma pergunta produzidas juntas de propósito.

## 11. Quatro premissas minhas que a medição derrubou

1. *«Borrar só o que está atrás do prefab é impossível — o mundo e os painéis partilham uma
   textura.»* **Falso.** Eles partilham a textura do **Vello**; o `world_rt` existe e é exactamente
   o conteúdo que se quer borrar.
2. *«Mover a receita seria uma edição.»* **Falso, com duas premissas erradas:** o razão de
   pré-visualização (`preview_drive`) já existia, e a posição de MUNDO de uma receita é
   **bastidores** — `instance_sync::ROOT_IS_ITS_OWN` já garante que o `Transform` de um `MasterRoot`
   nunca alcança uma cópia.
3. *«A ausência de log do `Ctrl+Z` significa que a tecla foi engolida.»* **Indeterminado, e era a
   pergunta errada** — «não foi premido» e «foi engolido» imprimem a mesma coisa: nada. Só depois
   de instrumentar as **três estações** (`RECEBIDA` · `SOBREVIVEU A CADEIA` · `respondido por`) é
   que o log mostrou que a tecla chegava inteira e o defeito estava **a jusante**.
4. *«O painel vetorial pode estar a tapar as linhas da Hierarquia.»* **Falso** — as duas colunas são
   disjuntas por construção (`layout.rs`: `hierarchy_x = viewport.x + rail_w`,
   `inspector_x = viewport.x + viewport.w − insp_w`). A hipótese custou uma volta; o que a matou foi
   imprimir a **moldura** das linhas registadas ao lado da posição da pressão.

---

## 12. Linha para o `CLAUDE.md §5`

O §5 recebe **uma linha** (§1.5.9 item 8). A narrativa é este documento.

> ⭐⭐ **O PREFAB ABRE-SE, e o `Ctrl+Z` da Hierarquia volta a funcionar** (07/09) — *Edit Prefab* por
> **quatro** portas, com **vidro jateado** sobre o mundo (`ph2d-render/frost.rs`, o passe que só é
> possível porque os painéis **não** estão no `world_rt`), a receita conduzida ao centro da área
> **visível** como pré-visualização (⛔ a câmera **não** se move — ordem do dono) e uma barra
> `Done`/`Cancel` (`Enter`/`Esc`) que **tranca** a saída. ⛔⛔ **E o report *«reordenei objetos na
> hierarquia e não funcionou o undo»* NÃO era o undo: era a ORDEM DO QUADRO** — o dreno do reparent
> corria ~2 340 linhas **depois** da projecção de z, a captura ficava com `world` novo e `vec` velho,
> e nascia um passo **fantasma** `partes: ["vec"]` que o `Ctrl+Z` gastava e o quadro seguinte
> recriava (fila parada em `5`). ⇒ *todo escritor da árvore corre antes de ela ser lida, e a leitura
> antes da captura*; gate `a_hierarchy_drag_leaves_the_capture_a_fixed_point`. ⏳ Os outros verbos
> tardios (apagar · duplicar · *Remove from Sheet*) têm a mesma latência e **não** foram medidos.
> [Handoff de 07/09](docs/Components/handoffs/HANDOFF_INTEGRACAO_line_components_PREFAB_ABERTO_2026-09-07.md)

---

## 13. Estado

**Linha `components` pronta.** HEAD `75fd9871c`, 29 commits, base `815555aed`.
Smoke compilado: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && cargo run -p ph2d-host-desktop --release` (2.ª corrida: `Finished` em `0.18s`, zero `Compiling`).

⛔ **Aguardo ordem de integração.** Não integrei, não pushei.
