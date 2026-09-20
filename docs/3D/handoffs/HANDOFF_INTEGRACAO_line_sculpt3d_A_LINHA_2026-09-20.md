# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, a LINHA inteira (2026-09-20)

> **Para o agente INTEGRADOR.** Ordem do dono, 2026-09-20: *«vamos integrar ao
> main»*. Este documento é a superfície de colisão **medida**, não um resumo do
> que a linha fez — o mecanismo de cada wave vive nos §§ do
> [`HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md`](HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md)
> (§57 a §102) e a entrada do roteador está no `CLAUDE.md` §5.
>
> ⛔ **A linha NÃO integrou nem pushou nada.** Ela está fechada, commitada e
> parada em `275cf752d`.

---

## §1 — O que é, em números

| grandeza | valor |
|---|---|
| ramo | `line/sculpt3d`, worktree `Worktrees/line-sculpt3d` |
| HEAD | `275cf752d` |
| merge-base com o `main` | `3090cac3f` |
| commits da linha | **102** |
| ficheiros tocados | **594** (`+40 628` / `−1 795`) |
| o `main` andou desde o merge-base | **19** commits |

⚠️ **Os 19 commits do `main` são TODOS `memoria(cascadeur)`** — eles tocam
**só** `project-memory/`. Não há uma linha de produto do outro lado.

---

## §2 — A superfície de colisão, MEDIDA

Corrida com o caminho **absoluto do primário**, como o `CLAUDE.md` §1 manda
(`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`).

### §2.1 — Contadores partilhados: **ZERO se mexem**

| contador | a linha | o `main` de hoje |
|---|---|---|
| `PROJECT_SCHEMA` | `144` | `144` |
| tripla do gate | `(144, 13, 22)` | `(144, 13, 22)` |
| `VEC_SCENE_SCHEMA` | `22` | `22` |
| `FLIP_SCHEMA` | `13` | `13` |
| `DOC_VERSION` (timeline) | `18` | `18` |
| `FIELD_DOC_VERSION` | `23` | `23` |
| registo `ph2d-ecs` | `91` | `91` |
| espelho `ph2d-render` | `92` | `92` |
| espelho `ph2d-script` | `92` | `92` |

⚠️⚠️ **A coluna `base:` do `collision-surface.sh` é o MERGE-BASE e não o `main`
de agora** (`CLAUDE.md` §1). Para este handoff o `PROJECT_SCHEMA` foi **lido no
ficheiro do `main`**, não na coluna:

```
git show main:shells/desktop/src/project_schema.rs | grep -nE 'PROJECT_SCHEMA[^_A-Za-z]*='
  338:pub(crate) const PROJECT_SCHEMA: u32 = 144;
```

⇒ **não há degrau a recontar**, e o `python3 scripts/schema-recount.py` não tem
trabalho nesta fusão. *Isto é um facto sobre ESTA rodada; se o integrador fundir
outra linha primeiro e ela mover um contador, esta continua a não o mover — ela
não escreve em nenhum dos três sítios.*

### §2.2 — Contrato congelado (§6): **intocado**

```
crates/ph2d-nodegraph/src/node.rs      intocado
crates/ph2d-editor-core/src/tool.rs    intocado
```

### §2.3 — ADR: **nenhum**

A linha não cria ADR (último no disco `0170`, próximo livre `0171`) ⇒ **fora de
toda disputa de número**.

### §2.4 — `Cargo.lock`: **um pacote novo, e ele é INTERNO**

`ph2d-rake` — crate-folha desta linha (a lei do pente de topologia, §82–§91).
⛔ **Nenhum pacote externo novo**, logo `cargo deny`/`audit` não ganham sujeito.

### §2.5 — Tectos de LOC

Nenhum ficheiro que a linha tocou passa do tecto. ⚠️ **Isto NÃO fecha a questão
da ACUMULAÇÃO** — ver §5.

---

## §3 — Onde a fusão pode doer (e é pouco)

### §3.1 — Os ficheiros que as DUAS árvores tocaram: **três, todos de memória**

```
project-memory/MEMORY.md
project-memory/reference_topic_gate_discipline.md
project-memory/reference_topic_mutation_proofs.md
```

São as **únicas** sobreposições entre `main` e a linha. As três são markdown de
memória (índice + dois ficheiros de tópico), e o conflito, se houver, é textual e
de **adição de ambos os lados** — a cura é manter as duas metades, nunca escolher
uma. ⚠️ O `MEMORY.md` já está **acima do tecto declarado nele próprio** (17 KB /
140 linhas): quem funde **não** deve aproveitar para compactar — isso é trabalho
do dono da memória e faria um diff de fusão impossível de auditar.

### §3.2 — A ÚNICA mudança de produto numa crate partilhada

`crates/ph2d-gpu/src/context.rs` — o `max_vertex_buffers` passa a subir ao
**máximo que o adaptador anuncia**, pelo mesmo `max(...)` que os três limites
vizinhos já faziam:

```rust
required_limits.max_vertex_buffers = required_limits
    .max_vertex_buffers
    .max(adapter_limits.max_vertex_buffers);
```

⚠️ **É ADITIVO e não pode fazer o `request_device` falhar** (é um superconjunto
do default). A razão está escrita no ficheiro, com a medição: o piso do WebGPU é
`8`, as duas placas desta máquina anunciam `32`, e a malha da escultura alimenta
**nove** buffers por vértice desde que a COR existe (§99). ⛔ Sem isto o pipeline
de malha **não se constrói**, e é por isso que o limite sobe em vez de haver uma
recusa graciosa — a alternativa, nomeada no próprio comentário, é empacotar as
duas curvaturas num `vec2`.

### §3.3 — A shell: **só ficheiros de TESTE**

```
shells/desktop/tests/it/the_dynamic_topology_is_wired.rs
shells/desktop/tests/it/the_matcap_table_and_its_chips_agree.rs
shells/desktop/tests/it/the_sculpt_document_is_wired.rs
shells/desktop/tests/it/the_sculpt_gesture_is_wired.rs
```

⛔ **Zero linhas de produto da shell** ⇒ a catraca `the_shell_only_shrinks` não
tem sujeito nesta linha.

### §3.4 — O resto do diff, por crate

`ph2d-sculpt3d` (72) · `ph2d-app-sculpt3d` (56) · `ph2d-mesh-render` (21) ·
`ph2d-panel-sculpt3d` (19) · `ph2d-quadflow` (12) · `ph2d-mesh` (7) ·
`ph2d-pose` (6) · `ph2d-rake` (3, nova) · `ph2d-trim` (2) · `ph2d-mesh-bool` (2) ·
`ph2d-remesh-iso` (1) · `ph2d-i18n` (1, só `sculpt3d.rs`) · `ph2d-gpu` (1, §3.2) ·
`docs` (380) · `CLAUDE.md` (1) · `Cargo.lock` (1).

---

## §4 — A prova de fecho (corrida sobre o HEAD final, `275cf752d`)

| portão | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` | **16 532 / 16 532** |
| suíte de GPU da `ph2d-mesh-render`, **com adaptador** (`--ignored`) | **76 / 76** |
| `bash scripts/censos-da-arvore-combinada.sh` | **90 / 90** (controlo do filtro: 8 de 8 censos correram) |
| `cargo clippy --all-targets -- -D warnings` (as crates tocadas) | **zero** |
| `cargo fmt --all -- --check` | limpo |
| `bash scripts/doc-index.sh --check` | **20 índices em dia** |
| árvore | `git status --porcelain` → **0** |

⚠️ **O `nextest-impacted` foi corrido DUAS vezes**: antes do commit de código e
outra vez sobre o HEAD com os docs dentro — *um commit de markdown pode acordar
um censo de texto, e afirmar o portão sobre uma árvore anterior seria afirmar
sobre outra coisa*.

### §4.1 — As vassouras da parede clean-room

Corridas as **10** vivas contra as crates da família, no HEAD **e** na árvore do
merge-base (⚠️ com as vassouras de HOJE dos dois lados — *o sweep é propriedade
do PAR (código, vassoura)*, e comparar código velho com vassoura velha não
isolaria nada):

| árvore | ficheiros acusados |
|---|---|
| merge-base `3090cac3f` | **15** |
| HEAD `275cf752d` | **16** |

A diferença é de **ENDEREÇO**, não de texto novo:

- `crates/ph2d-panel-sculpt3d/src/rows.rs` **sai** da lista e
  `crates/ph2d-panel-sculpt3d/src/rows_brush.rs` **entra** — é o corte de tecto
  de LOC da wave da pintura (§99): *a isenção é propriedade do CÓDIGO e VIAJA com
  ele* (`CLAUDE.md` §5.0).
- `crates/ph2d-app-sculpt3d/src/undo_canais_tests.rs` entra — ficheiro criado por
  esta linha.
- ⭐ A crate NOVA (`ph2d-rake`) é acusada **zero** vezes.

⛔⛔ **Eu não li um único achado.** O `I` não faz triagem de vassoura — *os hits
são do `R`*, e é a ele que a reconciliação destes dois endereços pertence.

### §4.2 — O que SÓ a árvore combinada pode reprovar

1. **Os censos de texto (HR-15) e os tectos de LOC são propriedades da SOMA.**
   Corri o `censos-da-arvore-combinada.sh` **nesta** árvore (90/90); ⚠️ ele mede a
   linha contra o `main` de HOJE, e se o integrador fundir **outra** linha antes
   desta, o resultado muda — a corrida tem de ser repetida depois de cada fusão
   (`DIRETRIZ` §1.5.9 item **5-bis**).
2. **O tecto de LOC por ACUMULAÇÃO.** Nenhum ficheiro desta linha passa sozinho;
   duas linhas a somar no mesmo ficheiro passam. Se a catraca acender, a cura é
   **corte por responsabilidade**, nunca uma entrada nova no `FILE_OVERAGE_OK` —
   e o excesso que a catraca IMPRIME é o que se corta, não mais.
3. **Os arch-gates que vivem em `tests/it/` de OUTRAS crates.** Eles não correm
   num `cargo test -p` da família (esta linha pagou essa cegueira cinco vezes);
   quem os alcança é o `nextest-impacted` por `rdeps`, que é o que está na tabela
   do §4.

---

## §5 — O que a linha entrega (índice, para o commit de fusão)

Por ordem de chegada, com o § do handoff da jornada:

| § | o que fechou |
|---|---|
| §57–§58 | o G-20 do pincel de plano — **só gate**, zero linhas de produto |
| §61 | as duas opções que o dono mandou construir e depois **retirou** |
| §62 | as reentrâncias da pose: tecto da transição `100 → 300` |
| §63–§64 | os dois gates que faltavam à espec do plano; o `Plane Offset` **sai** por veredito do dono |
| §65–§66 | as cunhas da costura do Box Trim; a `=46` deixa de escolher a própria peça |
| §67–§68 | a transição da pose deixa de contar ANÉIS; a marcha por faces cura as estrias |
| §80–§91 | **o pente de topologia** (`ph2d-rake`, a crate nova) — cinco reprovações do dono, a retícula do Instant Meshes, a vista da grade, a memória do traço |
| §92–§93 | a **geodésica**: o carimbo deixa de atravessar parede fina (razão superfície/ar + normal) |
| §95–§96 | o gancho com `Connected Only`: *a máscara decide quem ENTRA, nunca quem SAI* |
| §97 | `Pull Along Normal` nos dois verbos de puxão |
| §98 | e a âncora da puxada, que CONGELA no clique |
| §99–§100 | **os três pincéis de COR** (`Paint` · `Blur` · `Smear Color`) + a topologia dinâmica para os três |
| §101 | a cor **sobrevive a toda luz**, e a luz ganha o modo **`Flat`** |
| §102 | a tinta **chega ao device** — o upload incremental esquecia a COR |

---

## §6 — Os smokes, para o dono correr DEPOIS da fusão

⚠️ Depois de integrar, o caminho é o do **primário**, não o da worktree:

```
cd /home/enio/Documentos/Projetos/PH2D && env PH2D_SCULPT3D_SMOKE=51 cargo run -p ph2d-host-desktop --profile smoke
```

- **`=51`** — a PINTURA (o `Paint`, o `Blur`, o `Smear Color`, a fileira
  `Material` com `Flat`/`Rig`). ✅ **Aprovado pelo dono em 20/09.**
- **`=49`** — o pente de topologia (`Edge Flow` + a caixa da vista da grade).
  ✅ Aprovado.
- **`=50`** — a parede fina (a barbatana). ✅ Aprovado.
- **`=47`** · **`=48`** — o pincel de plano e o afiado. ✅ Aprovados (17/09).

---

## §7 — ⏳ ABERTO — e é ORDEM DO DONO que fica para amanhã

### §7.1 — ⛔⛔ Manchas PRETAS ao pintar com topologia dinâmica

**Report de 2026-09-20, com foto:** *«Temos a pintura funcionando. Contudo, temos
problemas ao pintar com Dynamic Topology. Veja as manchas pretas. Acontece em
painter, blur e Smear. Mas a cura fica para amanhã.»*

O que se vê: polígonos **pretos de aresta dura** dentro da zona pintada, com a
forma das faces da malha — não um degradê, **faces inteiras a preto**.

⚠️⚠️ **Isto NÃO foi curado, de propósito** (decisão do dono). O que está medido,
para quem pegar nisto amanhã **não recomeçar do zero**:

1. **O REFINO está gateado e provavelmente ilibado.** Existe
   `ph2d-mesh/src/dyntopo_tests.rs :: the_new_vertices_carry_colour_and_mask`, que
   pinta um hemisfério, refina, e exige que os vértices NOVOS **interpolem** a cor
   e a máscara em vez de nascerem no default. *Um vértice nascido num split não
   é, à primeira vista, o suspeito.*
2. ⭐ **O COLAPSO não menciona os canais por vértice em lado nenhum.**
   `grep -n "colors\|masks" crates/ph2d-mesh/src/collapse.rs` devolve **zero**, e
   o mesmo no `collapse_tests.rs`. E aquele ficheiro **RENUMERA** — o doc dele
   di-lo três vezes (*«estas portas renumeram»*, `Remap`, `MAX_PASSES`). ⇒ *a
   primeira pergunta de amanhã é **quem permuta o plano de cor quando um colapso
   renumera a malha**.*
3. ⚠️ **E a renumeração de um colapso é uma CADEIA, não uma tabela plana** — esta
   linha já pagou essa lei no §27 (a pegada congelada do polegar), e o modo de
   falha lá foi `index out of bounds`; aqui seria **cor no vértice errado**, em
   silêncio.
4. **A régua que falta é a que separa as duas hipóteses:** um traço com o passe
   armado, e a cor de cada vértice comparada contra a dos pais — *o `Draw` não a
   revela, porque ele não escreve no canal*.

⛔ Nenhuma das três leis de cor (`stroke_cor.rs`) é suspeita: elas são medidas
por forma fechada e a pintura **sem** topologia dinâmica está aprovada pelo dono.

### §7.2 — Os abertos que já vinham

- o custo por dab dos dois verbos que lêem o anel (`Blur`, `Smear Color`) **não
  foi varrido**;
- a **cor não viaja no `.ph2dproj`**;
- o selector rico de cor (hoje são três pistas `Color R/G/B`);
- os outros modos de COR do Blender (*Single* · *Random* · *Object*) — nenhum foi
  pedido;
- o `Flat` **não** entra no bake nem na doação de forma (ele é VISTA).

---

## §8 — Sete coisas que uma leitura rápida do diff entende ao contrário

1. **`ph2d-rake` não é um pincel** — é a lei do pente de topologia, e o motor que
   o produto de facto corre desde §82 é a **retícula** da `ph2d-quadflow`. As três
   leis antigas ficam vivas **com a bancada delas** e sem chamador de produto
   (gate `o_carimbo_nao_penteia_e_o_passe_penteia`); *apagá-las levaria as 221
   corridas do corpus do alvo junto*.
2. **A cura do shader (§101) e a do upload (§102) são DOIS elos**, um a jusante do
   outro — a primeira não estava errada, estava incompleta.
3. **O `max_vertex_buffers` do §3.2 não é uma optimização**: sem ele o pipeline de
   malha não se constrói, porque a COR é o 9.º buffer por vértice.
4. **A `RAZAO_MAXIMA` da geodésica ficou SEM TRABALHO** (§93) e **não** foi
   removida: com ela inerte cai **uma** de 697 corridas, e nenhuma fixtura deste
   repo a distingue. A remoção é wave própria, nomeada.
5. **O `Plane Offset` foi RETIRADO da tela e a lei CONTINUA a lê-la** (§64), com
   gate nas duas metades — quem ler a ausência como *«o verbo não tem
   deslocamento»* e apagar a lei leva duas fixturas e a §2.4 da espec.
6. **As quatro fileiras de shell são testes.** A linha não escreve produto na
   shell.
7. **`project-memory/` é a única sobreposição com o `main`** — e o que está lá do
   lado do `main` são 19 commits de outra frente (Cascadeur), não trabalho desta.

---

## §9 — O que este handoff NÃO autoriza

⛔ **Ship.** O `git push` é do dono (`CLAUDE.md` §0.7), e integrar não é aprovar.
O smoke da `=51` está aprovado; o defeito do §7.1 está **aberto e nomeado**, e o
dono decidiu integrar com ele dentro.
