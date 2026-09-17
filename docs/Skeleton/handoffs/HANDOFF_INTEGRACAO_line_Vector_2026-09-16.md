# HANDOFF DE INTEGRAÇÃO — `line/Vector` (o ESQUELETO), 2026-09-16

> **Para o agente INTEGRADOR.** Este é o ponto de ENTRADA da linha: o que ela entrega, o que ela
> toca que outra linha também pode tocar, e o que fazer quando os dois lados discordam.
>
> ⚠️ **Leia o §1 antes do primeiro `git grep`** — a superfície de colisão está medida ali, e é a
> lista que uma integração redescobre ~1 000 vezes.
>
> ⛔ **O smoke é do dono e ele já o deu por bom** (2026-09-16: *«smoke OK»*). Integrar não é aprovar
> nem shipar: o `push` é ordem explícita dele (`CLAUDE.md` §0.7).

**Merge-base:** `1d43da737` · **148 commits** · **329 ficheiros** · `+41 265 / −4 580`.
**Régua:** tudo abaixo é contado contra o **merge-base**, nunca contra um `main` a andar.

---

## §1 — SUPERFÍCIE DE COLISÃO (o que outra linha também pode ter tocado)

Medida por `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` **dentro desta
worktree**, em 2026-09-16.

| grandeza | nesta linha | no merge-base | o que fazer |
|---|---:|---:|---|
| `PROJECT_SCHEMA` | **133** | `128` | ⚠️ **conte o DELTA: `+5`.** Some-o ao valor que o `main` tiver **no dia**, e escreva os degraus nos **três** sítios (a const, a escada ao lado, a tripla em `project_schema_tests.rs`) |
| tripla do gate | `(133, 13, 22)` | `(128, 13, 22)` | só o 1.º número se mexe |
| `VEC_SCENE_SCHEMA` · `FLIP_SCHEMA` · `DOC_VERSION` · `FIELD_DOC_VERSION` | `22` · `13` · `18` · `22` | iguais | **intocados** |
| registo de componentes (`ph2d-ecs` + os 2 espelhos) | `85` · `86` · `86` | iguais | **intocados** — nenhum componente novo se regista nesta linha |
| ADR | nenhum | — | fora de toda disputa de número |
| pacotes novos no `Cargo.lock` | `ph2d-skin-weights`, `ph2d-timeline-onion` | — | **internos** (path deps), zero dependência externa nova |

### Os cinco degraus de schema, e por que cada um é obrigatório

O postcard é **posicional**: um campo apendado a um componente faz um ficheiro velho ser lido com um
campo a mais, comendo os bytes do vizinho. ⛔ `#[serde(default)]` **não** salva (ele serve formatos
com nomes).

| degrau | o quê |
|---|---|
| `128 → 129` | o `Bone` ganhou `segments` + `curve` (o osso que DOBRA) |
| `129 → 130` | a malha de uma imagem presa leva os **pesos** do padrão-ouro dentro |
| `130 → 131` | a forma **vectorial** presa também os leva — fecha a divergência que o `130` abriu |
| `131 → 132` | o `Bone` ganhou `handles` (`Authored` · `Auto`) |
| `132 → 133` | o `Bone` ganhou `curve_tip` (`Chain` · `Straight` · `Bone(StableId)`) |

⛔ **Sem degrau de migração** nos cinco — a decisão do dono de 26/08 (não há projectos gravados). ⚠️
**A tripla não os vê**: os bytes mudam DENTRO de um `ComponentBlob`, opaco para ela.

### ⚠️ O contrato CONGELADO (§6) aparece no diff — e NÃO foi tocado

`crates/ph2d-editor-core/src/tool.rs` tem **44 linhas mudadas**, e são **44 inserções** de duas
funções livres (`is_image_edit_tool`, `palette_visible_tool_indices`) que **mudaram da shell** para
cá, verbatim menos os caminhos. O `trait Tool` não perdeu nem ganhou um método: quem o afirma é o
gate `architecture_tool_contract_surface` (`Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/
`PanelEvent=4`), verde. ⇒ **nenhum ADR é preciso.**

### Tectos de LOC — os dois que o mapa de colisão acusa são ISENÇÕES ANTIGAS

`shells/desktop/src/app_state.rs` (`997`) e `main.rs` (`1114`) estão no `FILE_OVERAGE_OK` do
`file_loc_caps.rs` **desde antes desta linha** (linhas `31` e `36` da allowlist) e ela **não os fez
crescer**. Os tectos que esta linha estourou foram curados por **CORTE**, nunca por isenção — ver §5.

---

## §2 — O QUE A LINHA ENTREGA (uma linha por obra, com o endereço do mecanismo)

A narrativa de cada uma está no handoff datado; ⛔ **não a reescreva aqui**.

| obra | o que fecha | onde ler |
|---|---|---|
| **A pele no passe de sprites** | a imagem presa deixou de ser uma camada do Vello por cima do quadro e passou a ser uma **sprite desenhada como malha** — com a ORDEM, o onion e o orçamento do quadro | [`…A_PELE_NO_PASSE_DE_SPRITES_2026-09-13`](HANDOFF_INTEGRACAO_line_Vector_A_PELE_NO_PASSE_DE_SPRITES_2026-09-13.md) |
| **O atlas e o quadro** | o `Smooth` «bugado» era o **atlas** (uma cópia da imagem por peça) e um quadro sem recurso tardio apagava o atlas do Vello | [`…O_ATLAS_E_O_QUADRO_2026-09-13`](HANDOFF_INTEGRACAO_line_Vector_O_ATLAS_E_O_QUADRO_2026-09-13.md) |
| **O padrão-ouro dos pesos** | *Bounded Biharmonic Weights*, clean-room do paper, nas **duas** mídias (crate nova `ph2d-skin-weights`) | [`…A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14`](HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14.md) §pesos |
| **O pincel paga a dobra** | o dab nasce como a elipse que a malha endireita, e a **curvatura** dentro dele é um polinómio de grau 3 | [`…A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14`](HANDOFF_A_CURVATURA_DEBAIXO_DO_DAB_2026-09-14.md) |
| **O osso que DOBRA (B-Bones)** | `segments` + duas alças, a fábrica de sub-ossos, o desenho, o dedo e o painel — e o ponto neutro **exacto por construção** | [`HANDOFF_O_OSSO_QUE_DOBRA_2026-09-15`](HANDOFF_O_OSSO_QUE_DOBRA_2026-09-15.md) |
| **O resto do app achava a arte PLANA** | o censo das 19 superfícies + as ferramentas curadas (conta-gotas, remoção de fundo, gizmo, anel do pincel), a **regra do dono** dos pixels planos, e o `Smooth` sob cena cheia | [`HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15`](HANDOFF_O_RESTO_DO_APP_ACHAVA_A_ARTE_PLANA_2026-09-15.md) §1–§29 |
| **O *custom handle* e a cor das alças** (hoje) | o artista escolhe **qual filho** manda na curva; as alças ganham token próprio e passam a ser o **último** passe do rig | [`docs/Skeleton/01_a_fila.md`](../01_a_fila.md) F6-v + §4 abaixo |

---

## §3 — FOUNDATIONAL TOCADO, e por que é ADITIVO

| crate | o quê | aditivo porquê |
|---|---|---|
| `ph2d-editor-core` | `tool_activation` (módulo novo) · `layout_switch::install_at_startup` · duas funções livres em `tool.rs` · ids do osso | tudo **acrescenta**: o `apply` do clique na aba continua a existir e a pedir pelo barramento (gate), e o `trait Tool` é intocado |
| `ph2d-tokens` | token `bone-handle` (apelido, **sem valor próprio**) · `color_space.rs` (corte por LOC, com `pub use` no sítio antigo) | um apelido não escreve no `tokens.json`; o endereço público das quatro funções de cor **não muda** |
| `ph2d-ecs` | nada de registo — só leitura | os três contadores ficam onde estavam |
| `ph2d-render` | a sprite desenhada como **malha** no passe que já existia; `vello_keepalive` | pipeline nova **zero** — é o mesmo passe com um `mesh` opcional |
| `ph2d-skeleton-ecs` | `Bone::handles`, `Bone::curve_tip`, `CurveTip` | campos apendados ⇒ os degraus `132` e `133` |

⚠️ **Duas crates NOVAS** (`ph2d-skin-weights`, `ph2d-timeline-onion`) são folhas: nascem com os
consumidores já ligados e não têm dependência externa.

---

## §4 — A OBRA DE HOJE (2026-09-16), que ainda não tem handoff próprio

Quatro entradas da fila, nesta ordem: **F6-s** (a regra dos pixels planos), **F6-t** (o `Smooth` sob
cena cheia), **F6-u** (a cena abria sem ossos), **F6-v** (o *custom handle* + a cor das alças).
O mecanismo de cada uma está na [fila](../01_a_fila.md); o que o integrador precisa de saber:

1. ⛔⛔ **A cena de smoke abria SEM OSSOS na máquina do dono**, e a causa é de ORDEM e vale para
   **toda** cena que escolhe uma ferramenta: o arranque instalava o layout gravado pelo mesmo verbo
   do clique numa aba, que **pede** a ferramenta pelo barramento — drenado a meio do 1.º quadro,
   *depois* do prólogo onde a cena escolhe a dela. ⇒ `layout_switch::install_at_startup` pega a
   ferramenta **antes** do primeiro quadro e não deixa pedido pendente.
   ⚠️ **A lei de activação saiu do dreno da shell para a `editor-core`** (`tool_activation`): dois
   leitores, uma lei.
2. ⭐ **`Bone::curve_tip`** — o filho é um `StableId` (nunca os bits) e resolve-se **por comparação
   entre os filhos**; um escolhido alheio ou apagado deixa a ponta **recta**.
3. ⭐ **O token `bone-handle`** — apelido do `success`, escolhido pelo **pior caso medido** nos 8
   temas (OKLab): `CurveG 0,166` · `Text1 0,135` · **`Success 0,114`** · `Warn 0,080` ·
   `Info`/`Danger 0,062`. O gate mede as **portas** do desenho, não dois tokens escritos no teste.
4. ⭐ **As alças são o ÚLTIMO passe do rig** (depois dos ossos **e** das âncoras), porque no
   `bone_pick::hover` elas ganham aos dois: *o que o dedo apanha primeiro pinta-se por último*.

---

## §5 — OS TECTOS QUE ESTA LINHA CUROU POR CORTE (e o que sobrou na shell)

| tecto | cura |
|---|---|
| `ph2d-tokens/src/color.rs` (`702/700`) | a aritmética OKLCH⇄sRGB saiu para `color_space.rs` (**com `pub use` no sítio antigo** — a `ph2d-viewport3d` escreve `ph2d_tokens::color::srgb_to_oklch`) |
| `shells/desktop/src/project_schema.rs` (`605/600`) | a escada arquivou as faixas `v83..v98` e `v99..v108` em `docs/archive/project-schema/`, verbatim, com o `sha256` de cada original |
| painéis e ficheiros do esqueleto (waves anteriores) | cortes por responsabilidade — ver os handoffs datados |

⭐ **A shell ENCOLHEU:** `−40` linhas contra o merge-base (medido por
`git diff --numstat 1d43da737 -- shells/desktop`), com a catraca `the_shell_only_shrinks` verde.

⚠️ **E o tracker do módulo foi CORTADO no fecho** (protocolo §1.5.9 item 5): a
[`01_a_fila.md`](../01_a_fila.md) passou de **264 KB** para **27 KB**, com a história verbatim em
[`docs/archive/skeleton-fila-2026-09-16/`](../../archive/skeleton-fila-2026-09-16/01_a_fila.md)
(`doc-split.py`, remontagem `sha256` idêntica, 32 links reancorados). ⛔ **As recusas medidas e os
seis itens abertos dentro de waves fechadas ficaram INDEXADOS no doc vivo** — arquivar uma recusa
sem índice é apagá-la.

---

## §6 — OS SMOKES, com o comando exacto

⚠️ O caminho é o **desta worktree**: o dono corre de outro directório, e sem o `cd` o comando testa a
árvore errada.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```
A cena do rig: braço e tentáculo já presos, a folha solta (o gesto do *Bind*), o **braço pintado**
(a 2.ª mídia), a barra que ensina a ORDEM, e — desde hoje — **a BIFURCAÇÃO** (dois esqueletos
pequenos no meio de baixo: `Curve: one child` e `Curve: two children`).

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_PAINT_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```
O canvas do Painter **preso a ossos e dobrado** — a cena que o censo das superfícies planas obrigou
a existir.

**Diagnóstico** (não são cenas): `PH2D_BONE_LOG=1` · `PH2D_SKIN_PIECES=<n>` (o orçamento do QUADRO)
· `PH2D_SKIN_REFINE=uniforme` (bissecta a lei antiga) · `PH2D_PICK_LOG=1` · `PH2D_BONE_UNDO_PROBE=1`
· `PH2D_BONE_SMART_PROBE=1`.

---

## §7 — O QUE ESTÁ ABERTO (a linha PARA aqui)

1. **F9** — a pele deformada na **GPU** (pedido do dono, 2026-09-16): é o que faz o `Smooth` alisar
   numa cena cheia, onde hoje ele cai no `Fast` com aviso. ⚠️ **W0 é MEDIR** (o `PH2D_SKIN_PIECES`
   fecha isso numa corrida), nunca subir o `SKIN_FRAME_PIECES`.
2. **F10** — o **AutoKey** com a corrente de ossos, *«como no Blender última versão»* (decisão do
   dono). ⚠️ **Passo 1 é correr o Blender instalado por script** (`CLAUDE.md` §0.9), não ler o fonte.
3. **F11** — imagens em **9 fatias** e **folhas de quadros** deformam com os ossos (decisão do dono).
   Hoje desenham-se sem deformar, com aviso único no terminal.
4. Os **seis** itens abertos dentro de waves fechadas, indexados no topo da
   [fila](../01_a_fila.md) com a linha do arquivo onde cada mecanismo vive.
5. **A pergunta que o dono ainda não respondeu:** nenhuma. As quatro de 2026-09-16 foram respondidas
   por ele (undo suficiente · AutoKey como o Blender · a cena da bifurcação · 9-slice e folha
   deformam) e estão na fila como F10/F11.

---

## §8 — O QUE UMA LEITURA RÁPIDA DO DIFF ENTENDE AO CONTRÁRIO

1. **`tool.rs` mudou ⇒ contrato congelado mexido.** Não: são duas funções livres que VIERAM da
   shell; a superfície do trait tem gate e está verde.
2. **`PROJECT_SCHEMA 133` ⇒ escreva `133`.** Não: **conte `+5`** contra o `main` do dia. Duas linhas
   que escrevam o mesmo literal fundem **mudas**.
3. **`layout_switch::apply` deixou de ser chamado.** Não: ele é a porta do **clique numa aba** e
   continua a pedir a ferramenta pelo barramento (gate `clicking_a_layout_tab_still_requests_its_tool`).
   O que nasceu ao lado é a porta do **arranque**.
4. **O token novo precisa de valores no `tokens.json`.** Não: ele é **apelido** e resolve pelo pai;
   um valor escrito à mão ali reprova no gate `no_alias_carries_a_hand_written_value`.
5. **A `ph2d-tokens/src/color.rs` perdeu funções públicas.** Não: elas mudaram de ficheiro e o
   `color.rs` re-exporta-as — `ph2d_tokens::color::srgb_to_oklch` continua a resolver.
6. **O `draw_bend` mudou de sítio ⇒ refactor cosmético.** Não: é a cura de um report do dono, com
   gate posicional que fica **vermelho** com a ordem antiga.
7. **`01_a_fila.md` encolheu 90 % ⇒ perdeu-se história.** Não: `doc-split.py` prova a remontagem por
   `sha256`, e o doc vivo indexa o arquivo, as recusas e os abertos.

---

## §9 — O PORTÃO DE FECHO

Corrido **1× sobre o diff acumulado** (nunca por task), em 2026-09-16, com a régua no MERGE-BASE.

| passo | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **15 610 testes · 15 610 passaram · 0 falharam** (9 690 saltados, 523 binários, `99,3 s`) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | **exit 0** — um `dead_code` que só a workspace inteira vê não aparece num `check -p` |
| `cargo clippy --workspace --all-targets` | **0 avisos, 0 erros** |
| `cargo machete` | *«didn't find any unused dependencies»* — ⚠️ obrigatório porque a linha **moveu código** |
| `bash scripts/check-standalone-optional.sh` · `check-workflow-packages.sh` | **exit 0** nos dois |
| leitores da shell por CAMINHO (`git grep 'shells/desktop/src' -- crates tools`) | `118` ficheiros, **nenhum** aponta para o que esta linha moveu (`app_state_image_tools.rs`, `project_schema_history_*.rs`) — e os `include_str!` deles falhariam **alto** no `check` acima |
| tectos de LOC (`file_loc_caps` + `workspace_file_loc_cap`) | **verdes** (os dois estouros do dia curados por corte) |
| `the_shell_only_shrinks` | **verde**, com a shell a `−40` contra o merge-base |

### As lentes da auditoria (≥2, protocolo §1.5.9)

1. **O PRODUTO na tela do dono** — a cena fotografada na configuração DELE (`kwin_wayland --virtual`
   + `import -window`), que é o que apanhou a cena a abrir **sem ossos** e a peça nova **fora do
   ecrã**; ⛔ a 1.ª foto correu com um `HOME` limpo e mostrava tudo bem — *uma cena fotografada só na
   arrumação de fábrica foi medida num programa que o dono não corre*.
2. **Os GATES e as mutações** — cada lei nova nasceu vermelha e cada cura foi provada por mutação
   (as três da ponta da curva, a da cor das alças, a da ordem do desenho, a do arranque).
3. **A superfície de COLISÃO** — o §1, corrido nesta worktree antes do primeiro `grep`.

---

## §9-bis — O SMOKE fica COMPILADO (protocolo §1.5.9 item 9)

O binário do comando exacto do §6 está construído **nesta worktree**, depois do commit final e
depois de reclamar o incremental (`rm -rf target/*/incremental`). A prova é a **2.ª corrida**:

```text
$ cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

*Finished* em `0,21 s` e **zero** linhas `Compiling` — o dono não espera build. ⚠️ Uma env `PH2D_*`
não é outro build; **feature, perfil e a árvore do `cd` são**.

---

## §10 — A **UMA LINHA** proposta para o `CLAUDE.md` §5 (quem a aplica é o integrador)

> ⭐⭐⭐ **O ESQUELETO DOBRA A ARTE, e o resto do app deixou de a achar PLANA** (16/09,
> [handoff](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_2026-09-16.md)): pesos do
> **padrão-ouro** (BBW, clean-room) nas duas mídias · a imagem presa é uma **sprite desenhada como
> malha** no passe que já existia (zero pipeline nova) · **B-Bones** com alças que se pegam no
> canvas e um *custom handle* que escolhe **qual filho** manda na curva · o pincel, o conta-gotas, a
> remoção de fundo e os gizmos seguem a arte **dobrada**, e ⭐ **editar PIXELS acontece na imagem
> PLANA** (regra do dono: tamanho/margem soltam dos ossos; Liquify, cor e filtros não). ⚠️
> `PROJECT_SCHEMA` **+5** — conte o DELTA. ⏳ **ABERTO:** a pele na **GPU** (F9), o **AutoKey** com a
> corrente (F10) e o **9-slice/folha** a deformar (F11). Cenas: `PH2D_VEC_BONE_SMOKE=1` ·
> `PH2D_VEC_BONE_PAINT_SMOKE=1`.
