# HANDOFF DE INTEGRAÇÃO — `line/Vector` (2026-08-01)

**Para:** o agente integrador, por ordem explícita do Enio.
**Estado:** linha FECHADA, **10 commits**, todos os smokes aprovados pelo Enio.
**Base:** `main` = `98eb502a2`. ⚠️ **`main` NÃO andou desde o fork** — `git merge-base --is-ancestor main HEAD` é verdade, então a integração é **fast-forward** e não há rebase a fazer. Confira isto de novo antes de começar: se o `main` tiver andado, as duas notas de colisão do §7 passam a valer.

```
git -C Worktrees/line-Vector log --oneline main..HEAD
79b9450a5 chore(vector): os dois arquivos de painel no teto de LOC -- split por RESPONSABILIDADE
7d4dc5ea7 feat(vector): W6.1 -- o snap ganha a reivindicacao 2-D, e o Newton que a torna exata
a7514f906 feat(vector): W5 -- as quatro ops do Pathfinder, e o motor deixa de PANICAR em silencio
c4b59f255 feat(vector): a FITA aberta tambem e' cortada -- e a topologia da fonte escolhe a lei
210b6a560 fix(vector): os quatro ajustes do smoke -- a lamina nao tem estilo, e' visivel em todo modo
443bd0ee0 feat(vector): a LINHA DE CORTE e' um objeto, e uma forma fechada cortada da' formas FECHADAS
d30bd37d4 fix(vector): os dois pills de corte estavam MORTOS sob o mouse
fe6ba6edf feat(vector): W4 fatia C -- a FACA, e ela nao tem geometria propria
fc812b5f6 feat(vector): W4 fatia B -- a TESOURA, o 13o modo, e a cena =44
96d7c7930 feat(vector): W4 fatia A -- o CORTE existe, e as tres de selecao ganham maos
```

**65 arquivos, +6955 / −646.**

---

## §1 — O que entra (plano 25, W4 · W5 · W6.1)

### W4 — O CORTE

**A linha de corte é um OBJETO, não um gesto.** Ela é um `VecPath` normal marcado por um componente ECS (`VecCutPath`), então a Pen a desenha · o Select a move · o Node a edita · o undo e o save a carregam — tudo de graça. O botão **Cut** corta com ela; **Discard Cut Line** a descarta.

⚠️ **Cortar uma forma FECHADA dá formas FECHADAS.** O `linesweeper` exige contornos fechados (`Topology::from_paths` devolve `Result<_, NonClosedPath>`), então o trabalho inteiro do `cut_closed` é transformar a linha aberta desenhada num **cortador fechado `H`** cuja fronteira dentro da forma É a linha. As peças são as componentes dos **DOIS lados** (`S ∩ H` ∪ `S − H`) — de que lado uma peça caiu não é informação que alguém use, e é isso que torna o desenho imune ao que `H` faz longe da forma.

⚠️ **Duas recusas que são TOPOLOGIA, não limitação:** um corte que não atravessa (uma região menos uma fenda continua conexa) e uma ponta `Trapped` (estender cruzaria a forma).

Mais as três operações de nó (**Join · Reverse · Average**) e o `Select Subpath` / `Select Same`.

### W5 — O PATHFINDER

**Minus Back · Trim · Crop · Merge**, e nenhuma trouxe geometria nova: são composições do fold que já existia.

⚠️ **Dois enums, e não é cerimônia:** `BoolOp` é o vocabulário do MOTOR (o que o `linesweeper` entende, o que o Build e o Expand consomem); `PathfinderOp` é o do ARTISTA. Os quatro novos **não são operações de conjunto** — Trim devolve uma forma *por fonte*, cada uma com o seu estilo.

⚠️ **E o motor deixou de PANICAR em silêncio.** Escrevendo o gate da recusa descobriu-se que o `linesweeper` **panica** com `NaN` em vez de devolver `Error::NaN`: o `binary_op` dele examina só o **bounding box**, e `min`/`max` com `NaN` devolvem o outro operando, então o `NaN` atravessa a checagem e explode no sweep (`geom.rs:63`). A guarda de finitude é **nossa**, no choke point único — e por ser único ela cobre também o Expand e o Shape Builder, que não sabem que ela existe. A entrada é alcançável (um `Transform` degenerado assado na geometria): **é a diferença entre um toast e um crash.**

### W6.1 — A REIVINDICAÇÃO 2-D

O motor de snap ganhou uma **segunda espécie de alvo**.

O que já existia é **ALINHAMENTO**: restrição 1-D por eixo, e é por isso que ela se decompõe. Encaixar **sobre uma curva** é uma **POSIÇÃO**, 0-D — *"alinhar meu X com o X do ponto mais próximo daquela curva"* não quer dizer nada. Uma posição vence os dois eixos ou nenhum.

⚠️ **A lei que mantém as quinas alcançáveis:** *vértice vence curva*, enunciada como propriedade do **RESULTADO** — se o alinhamento já pousa exatamente sobre UM alvo, a reivindicação 2-D se retira. **Corolário: sem curvas na lista de alvos a lei nunca dispara, e o encaixe é byte-idêntico ao que já shipava** (gate `geometry_in_the_target_list_changes_nothing_while_the_toggles_are_off`).

Kernel novo `ph2d-vec-scene::curve_probe` (**puro, sem kurbo** — esta crate é o modelo de documento): amostra para escolher a bacia e **refina por Newton**. Mais os **cruzamentos** e os **sprites** como alvo.

---

## §2 — O que NÃO se mexe (conferido por grep + gate, não por auto-relato)

| Coisa | Estado | Como foi conferido |
|---|---|---|
| `PROJECT_SCHEMA` | **46, INTOCADO** | `git diff main..HEAD -- shells/desktop/src/project.rs \| grep PROJECT_SCHEMA` = vazio |
| `VEC_SCENE_SCHEMA_VERSION` | **13, INTOCADO** | idem |
| `FLIP_SCHEMA_VERSION` | **12, INTOCADO** | a linha não toca Flip |
| Contrato congelado §6 (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4`) | **INTOCADO** | `architecture_tool_contract_surface` 4/4 verde |
| Contrato congelado §6 (Vector doc/traits) | **INTOCADO** | nenhum arquivo de `ph2d-vector-doc`/`-traits` no diff |
| **`Cargo.toml`** | **ZERO tocados** | `git diff --name-only \| grep Cargo` = vazio |
| Crates novas | **nenhuma** | idem |
| Deps novas | **nenhuma** | idem |
| **ADR** | **nenhum** | esta linha fica **FORA** de qualquer disputa de número de ADR nesta janela |

⚠️ **Por que zero schema:** os ajustes de snap são **estado de FERRAMENTA** (não serializados); o `VecCutPath` é um **componente marcador**, que cunha blob-key própria (`stable_type_id` do NOME) em vez de apendar campo — e apendar campo seria postcard posicional, ou seja bump, e **um bump RECUSA todo projeto já salvo**.

---

## §3 — O que MUDA e o integrador precisa saber

### 3.1 — O registro do ECS é TRÊS, não um

`VecCutPath` entrou: **`ph2d-ecs` 39 → 40**, e os dois espelhos **`ph2d-render` e `ph2d-script` 40 → 41**.

⚠️ **Esta é a família que já ficou vermelho-latente DUAS vezes nesta linha** (integrações de 21/07 e 23/07): a MESMA contagem é afirmada em três crates, cada uma rodando só na suíte da própria, então duas ficam verdes numa corrida por-crate e só aparecem no gate da árvore combinada. **Os três estão atualizados neste diff** — se o `main` tiver andado e outra linha tiver registrado um componente, os três números se **CONTAM** contra o `main` do dia, não se escolhem.

### 3.2 — Superfície pública nova

- `ph2d-vec-scene::curve_probe::{CubicSeg, world_segs, nearest_on_segs, crossings_near}` — módulo novo.
- `ph2d-vec-boolean::{cut::*, pathfinder::*, engine::{SweepFailed, …}}` — módulos novos; `apply_many_checked` e `pathfinder` devolvem `Result`.
- `ph2d-vec-edit::snap::{SnapSource, SnapTargets.segs}` · **`SnapAxis.grid: bool` → `SnapAxis.from: SnapSource`** (com `is_grid()` preservando a pergunta) · **`collect_targets` ganhou um 5º parâmetro `curves: bool`**.
- `ph2d-vec-render::{GuideKind}` · **`Guide.grid: bool` → `Guide.kind: GuideKind`**.
- `ph2d-panel-vector::set_current_snap_position`.
- `ph2d-vec-render::dispatch` **não** mudou de assinatura nesta linha.

⚠️ Os dois campos renomeados (`SnapAxis.grid`, `Guide.grid`) têm **um único consumidor cada** (o shell), já ajustado.

### 3.3 — Ids novos

Em `crates/ph2d-editor-core/src/ids/chrome/`:
- `vector_cut.rs` (módulo do W4, já existente na linha): `VECTOR_MODE_CUT`, `VECTOR_CUT_APPLY`, `VECTOR_CUT_DISCARD`, `VECTOR_PATH_JOIN`, `VECTOR_PATH_REVERSE`, `VECTOR_VERT_AVERAGE`, `VECTOR_BOOL_MINUS_BACK`, `VECTOR_BOOL_TRIM`, `VECTOR_BOOL_CROP`, `VECTOR_BOOL_MERGE`.
- **`vector_snap.rs` (módulo NOVO)**: `VECTOR_SNAP_PATH_OFF/_ON`, `VECTOR_SNAP_CROSS_OFF/_ON`.

⚠️ O módulo novo nasceu porque **`vector.rs` está em 685/700** — não cabia mais nada lá. O corte é por responsabilidade (a família da PRECISÃO), e ele é o lugar onde as guias e o mirror da W6 vão entrar.

`node_id_collisions` verde (7/7).

### 3.4 — Mudanças de comportamento (todas smokadas)

1. **O balde de cores não é afetado** — nada disto toca `fill_at`.
2. **`apply_many` deixou de engolir o erro do motor:** `Ok(vec![])` e `Err` eram o mesmo nada na tela; agora quem chama distingue *"não havia resposta"* de *"o motor desistiu"*.
3. **Snap:** duas linhas novas na seção Snap, **ambas nascendo DESLIGADAS** — um ímã que agarra a linha inteira muda como todo gesto se comporta, e ligá-lo por default mudaria o app debaixo de quem não pediu.
4. **A caixa de um SPRITE virou alvo de snap** (antes não havia alvo nenhum do lado raster).

---

## §4 — A bateria de fechamento (rodada sobre o diff CUMULATIVO)

| Gate | Resultado |
|---|---|
| `cargo fmt --all --check` | **OK** |
| `cargo clippy --workspace --all-targets` | **zero warning, zero erro** |
| `cargo test --workspace` | **11.087 passaram, 0 falharam** |
| `architecture_workspace_file_loc_cap` | 2/2 |
| `shells/desktop/tests/file_loc_caps` | 2/2 |
| `architecture_panel_loc_cap` | 3/3 |
| `node_id_collisions` | 7/7 |
| `architecture_tool_contract_surface` | 4/4 |

**Mutações:** 14 na W6.1 (14 sangram) + as das waves anteriores, cada uma registrada no commit da sua wave.

### ⚠️ 4.1 — DUAS FLAKES PRÉ-EXISTENTES, e nenhuma é desta linha

Sob a workspace inteira em paralelo, dois gates de **wall-clock / razão** de OUTROS módulos reprovam, e passam quando rodados sozinhos:

- `ph2d-host-desktop::flip_smooth::…::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` — mediu 5,47 ms contra uma barra entre 0,72 e 64. **3/3 verde isolado.**
- `ph2d-mesh::measure_normals_parallel_speedup` — **3/3 verde isolado.**

**A linha não toca um único arquivo de Flip nem de 3D** (`git diff main..HEAD --name-only | grep -i "flip\|mesh\|sculpt3d"` = vazio). É a mesma família do `the_cost_of_depth_is_linear_not_explosive` que o `CLAUDE.md` já nomeia: **re-rode sozinho antes de suspeitar do merge.**

---

## §5 — ⚠️ A armadilha de gate que esta linha já pagou DUAS vezes

**Há TRÊS réguas de LOC independentes, e uma corrida `cargo test -p` por crate não alcança nenhuma delas:**

| Régua | Onde mora | Cobre |
|---|---|---|
| `architecture_workspace_file_loc_cap` | `ph2d-editor-core/tests` | `crates/**` (700) |
| `file_loc_caps` | `shells/desktop/tests` | a shell (600) |
| `architecture_panel_loc_cap` | `ph2d-editor-core/tests` | `ph2d-panel-*` (600) |

O último **sangrou no fechamento desta linha** e não no de nenhuma wave — os dois arquivos já vinham do `main` a beirar o teto (595 e 585). O commit `79b9450a5` os partiu **por responsabilidade, nunca por allowlist**.

**E os arch-gates de `shells/desktop/tests/` só correm na varredura impactada** — foi assim que a integração de 23/07 desta linha encontrou dois deles vermelhos no próprio tip. Rode a suíte da shell inteira na árvore combinada.

---

## §6 — Os smokes

Todos aprovados pelo Enio. Todos `--release`.

- **`env PH2D_BUILD_SMOKE=44 cargo run -p ph2d-host-desktop --release`** — o CORTE. A cena dá o MATERIAL (um anel, dois pares de fitas, uma seta) e **não arma modo nenhum**: escolha `Cut`, desenhe a lâmina com a caneta, e clique **Cut**. Julgue: uma forma **fechada** cortada dá formas **FECHADAS** · a lâmina é visível em todo modo e **não recebe Fill** · move-se com o Select mantendo a aparência rachurada · corta **qualquer** forma sobreposta, não só a selecionada · e só corta com a lâmina **selecionada** e o botão Cut marcado.
- **W5 não tem cena própria** — as oito ops vivem na seção **BOOLEAN** com 2+ formas fechadas selecionadas (Shift+clique). Julgue: **Trim** deixa todas lá sem sobreposição · **Crop** guarda só o que estava dentro da de cima e **a de cima some** · **Minus Back** guarda a da FRENTE · **Merge** solda as da mesma cor que se tocam.
- **W6.1 não tem cena própria** — seção **SNAP**, ligue `Path` e depois `Cross`. Julgue: um nó arrastado para perto de uma curva pousa **SOBRE** ela, com um **ANEL** na marca · perto de uma **ÂNCORA** a âncora vence (a quina continua alcançável, e a marca volta ao tracejado) · com duas curvas em **X** o cruzamento vence a linha e a marca vira **+** · **Alt** ignora tudo · e uma forma arrastada para perto de um **SPRITE** encaixa na caixa dele.

---

## §7 — Colisões possíveis com outras linhas nesta janela

Nada a reivindicar — mas confira, porque estas três já custaram renumeração no repo:

1. **`PROJECT_SCHEMA`** — esta linha **não bumpa**, então não entra na disputa. Se outra linha bumpar, o valor dela se **CONTA** a partir do `main` do dia ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
2. **ADR** — esta linha **não cria nenhum**. (A `line/Vector` já perdeu o 0145 para o 0148 numa janela anterior: *um número escolhido numa linha paralela é PROVISÓRIO*.)
3. **O registro do ECS (os TRÊS espelhos)** — ver §3.1. Se outra linha registrar um componente na mesma janela, os três números se contam, não se escolhem.

---

## §8 — O que fica ABERTO (não bloqueia a integração)

Da **W6**, na ordem da tabela do plano §9:

- **Guias e réguas** — o único item **G** (grande) da tabela, e hoje o repo tem **ZERO** (o único "guide" existente é o caminho-guia do pattern). Pede estado persistido, chrome de régua, gesto de arrastar-da-régua e provavelmente uma decisão de schema.
- **Mirror / simetria VIVA** — hoje só há Flip H/V destrutivo; o desenho é um `Mirror` na pilha de LPE (o `fx_repeat` já multiplica com `spin`/`orbit`).
- **Rótulo de distância** nos smart guides.

Das waves anteriores, herdado e **não** desta linha:
- O **caminho do tablet** (a fonte `Pen` é oferecida e não chega — a shell não recebe pressão de dispositivo; afeta o Flip igual, custa **uma função**).
- O **lasso** · X/Y numérico do nó · editar nós de VÁRIAS formas (ausência **por construção**: `selected_verts` pertence a um `selected` único).
- **Divide** e **Outline** do Pathfinder — nomeados e FORA, com as razões escritas no `pathfinder.rs` (Divide exige a varredura N-ária única, que via `Arrangement` são `2^N` regiões; Outline exige saída de caminho ABERTO, hoje estruturalmente impossível).
- Métodos de shape em modo máscara não pintam nada (pré-existente, do Painter).

---

## §9 — Erro de processo registrado

⚠️ **Nesta linha a cwd do Bash escorregou para a árvore PRIMÁRIA** (a mesma armadilha que a `line/Painter` documentou). Nada foi commitado lá, e a única corrida na primária foi um `cargo test` de leitura para comparar uma flake. **No Modo L, todo comando começa com o `cd` da worktree** — o mesmo path relativo existe nas duas árvores, e editar a errada compila e commita **sem erro**.
