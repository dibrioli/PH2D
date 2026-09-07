# HANDOFF DE INTEGRAÇÃO — `line/Vector` · O ESQUELETO VIRA MÓDULO, E GANHA A ÂNCORA

> **2026-09-07** · DIRETRIZ §1.5.9. A linha fecha aqui e **PARA** — não integra, não pusha
> ([`CLAUDE.md §0.7`](../../../CLAUDE.md)).

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/Vector` |
| HEAD | `d12ad1e7b` |
| merge-base com `main` | `004150bea` (**rebasada em 07/09**, ver ⚠️ abaixo) |
| commits | **19** |
| ficheiros | **86** |

⚠️ **A linha foi REBASADA sobre o `main` de hoje antes deste handoff.** Ela estava a `815555aed` e
o `main` tinha andado um commit (`004150bea`, só `.github/workflows/miri.yml`) — **zero
sobreposição** com os 86 ficheiros. O rebase correu limpo, 19/19. ⇒ a integração `--ff-only` da
§1.5.3 tem por onde entrar.

---

## §2 — Foundational / partilhado tocado, e por quê

| ficheiro | o quê | aditivo? |
|---|---|---|
| `crates/ph2d-ecs/src/lib.rs` · `scene/registry.rs` · `scene/registry_tests.rs` | ⛔ **os componentes do esqueleto SAÍRAM** do `register_ecs_components` | **não** — remoção |
| `crates/ph2d-ecs/src/vec_skin.rs` | **APAGADO** (o tipo mudou-se para a `ph2d-skeleton-ecs`) | não |
| `crates/ph2d-render/src/registry.rs` · `ph2d-script/src/registry.rs` | os dois **espelhos** da contagem: `82 → 80` | não |
| `crates/ph2d-editor-core/src/ids/chrome/vector_bone.rs` | ids novos (§3) | sim (append-only) |
| `crates/ph2d-editor-core/src/ids/chrome/mod.rs` | `pub use vector_bone::*` já existia | — |
| `crates/ph2d-i18n/src/vector.rs` | 8 chaves novas (§3) | sim |
| `crates/ph2d-component-desc/src/catalog/` | família `Skeleton` (`mod.rs`, `skeleton.rs`, `vector.rs`) | mista — 2 descritores saíram do `vector`, 4 entraram no `skeleton` |
| `crates/ph2d-panel-vector/` (9 ficheiros) | a seção SKELETON, o registo, o encaminhamento e o `seam_bone` | sim |
| `crates/ph2d-tool-vector/` (6) | o `DrawMode::Bone` e o `BoneAction` | sim |
| `crates/ph2d-vec-render/src/bone.rs` | **APAGADO** (mudou-se para `ph2d-skeleton-render`) | não |
| `crates/ph2d-vec-scene/src/xform.rs` | vira `pub use ph2d_affine::Xform;` — **~300 sítios de chamada intocados** | sim (re-export) |
| `shells/desktop/src/` (24) | o gesto, o passe, o undo, o smoke, a sonda, o schema | mista |
| `shells/desktop/src/undo_selection.rs` | ⚠️ **o filtro da selecção que sobrevive ao undo** ganhou duas famílias | sim |
| `shells/desktop/src/preview_drive.rs` | porta nova `release_to_authored` | sim |

**Crates NOVAS (4):** `ph2d-affine` · `ph2d-skeleton` · `ph2d-skeleton-ecs` · `ph2d-skeleton-render`.

---

## §3 — Símbolos que podem COLIDIR

### `collision-surface.sh`, colado (⚠️ REFERÊNCIA, não evidência — re-rode antes de fundir)

```text
SUPERFÍCIE DE COLISÃO — line/Vector contra main
  merge-base 004150bea   ·   19 commit(s)   ·   86 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        123   (base: 121)
  ⚠   └ tripla do gate               (123, 13, 22)   (base: (121, 13, 22))
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-render (espelho)                  80   (base: 82)
  ⚠ ph2d-script (espelho)                  80   (base: 82)
▸ CONTRATO CONGELADO — os dois INTOCADOS
▸ ADR — último no disco: 0169   próximo livre: 0170
  ⚠ esta linha cria ADR: 0169
▸ Cargo.lock — 4 pacote(s) novo(s): ph2d-affine, ph2d-skeleton,
                                    ph2d-skeleton-ecs, ph2d-skeleton-render
▸ MARCADORES DE CONFLITO — nenhum
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

### ⚠️ Os números que se CONTAM, não se copiam

| número | esta linha | base | delta |
|---|---|---|---|
| `PROJECT_SCHEMA` | `123` | `121` | **+2** |
| registo `ph2d-ecs` (e os dois espelhos) | `79` / `80` | `81` / `82` | **−2** |
| registo `ph2d-skeleton-ecs` (novo) | `4` | — | +4 |
| `ComponentCategory::ALL` | `13` | `12` | **+1** |
| catálogo, família vector | `30` | `32` | **−2** |

⛔ **O delta é o que se soma, nunca o literal.** Os dois degraus são `121→122` (o `Tendon` passa a
nomear o `StableId`) e `122→123` (os componentes `IkGoal`/`IkTarget`). Se outra linha bumpou no
meio, **conte 2 a partir do `main` do dia**.

⛔⛔ **Um componente que SAI conta tanto como um que entra.** O `−2` do `ph2d-ecs` é o esqueleto a
mudar-se de casa; os **três** contadores (`ph2d-ecs`, e os espelhos em `ph2d-render` e
`ph2d-script`) correm cada um só na suíte da própria crate — bumpar um e esquecer os outros passa no
`cargo test -p`.

### Ids novos (`ph2d-editor-core`, append-only)

`VECTOR_MODE_BONE` · `VECTOR_SECTION_BONE` · `VECTOR_BONE_BIND` · `VECTOR_BONE_EXPAND` ·
`VECTOR_BONE_RELEASE` · `VECTOR_BONE_LENGTH` · `VECTOR_BONE_STRENGTH` · `VECTOR_BONE_ACTION` ·
`VECTOR_BONE_ACT_CREATE` · `VECTOR_BONE_ACT_TRANSFORM` · `VECTOR_BONE_IK_ADD` ·
`VECTOR_BONE_IK_REMOVE` · `VECTOR_BONE_IK_MIX` · `VECTOR_BONE_IK_SOFTNESS` · `VECTOR_BONE_IK_CHAIN`

⭐ **Três TABELAS**, e elas são a cura de um defeito que voltou quatro vezes (§5):
`VECTOR_BONE_ACTION_IDS: [NodeId; 2]` · `VECTOR_BONE_VERBS: [NodeId; 5]` ·
`VECTOR_BONE_FIELDS: [NodeId; 5]`.

⚠️ **Um `NodeId` é o hash de uma STRING** — colisão só acontece se outra linha usar a mesma string,
e todas começam por `vector.bone.`.

### Nomes canónicos de componente (chave `blake3`, ⛔ irreversível depois de gravado)

`ph2d::skeleton::Bone` · `ph2d::skeleton::Skin` · `ph2d::skeleton::IkGoal` ·
`ph2d::skeleton::IkTarget` — **nenhum diz «vec»**, e há gate a impedir que alguém devolva a palavra.

### Chaves de i18n (8)

`panel.vector.bone.{action,create,transform,ik.add,ik.remove,ik.mix,ik.softness,ik.chain}`

### ADR

**0169** — *the skeleton is its own module and each medium answers only what a point is*.
⚠️ Número **provisório**: reconte contra o `main` do dia (o script diz que `0170` é o próximo livre).

---

## §4 — Contratos congelados

**NENHUM.** Os dois ficheiros gateados (`ph2d-nodegraph/src/node.rs`,
`ph2d-editor-core/src/tool.rs`) estão **intocados** — confirmado pelo `collision-surface.sh`.

---

## §5 — O que só o `ship.sh` apanha

- **Deps novas:** as 4 crates internas entram no `Cargo.lock`; **zero** dependência externa nova ⇒
  `machete`/`deny`/`audit` não têm sujeito novo.
- **`fmt` / `typos` pré-fork:** a linha correu `cargo fmt --all` no fecho; typos não foi corrido
  isolado (o `ship.sh` fá-lo).
- **Clippy:** `cargo clippy --workspace --all-targets` **limpo** no fecho.
- ⚠️ **`.github/workflows/miri.yml`:** a linha **não o toca**; ele aparece no `git diff main..HEAD`
  só porque a base era anterior ao commit do `main` — depois do rebase, não aparece.

---

## §6 — Ordem, dependências e o que smokar

**Os 19 commits são sequenciais e não se reordenam** — o corte de crates (`5d60bcc88`) é a base de
tudo o resto. Grupos:

⚠️ **Os hashes são os de DEPOIS do rebase** (§1) — a 1.ª redacção desta tabela trazia os de antes, e
eles já não existem. *Um handoff com hashes de antes de um rebase manda o integrador procurar
commits que o `git` não conhece.*

| # | commits | o quê |
|---|---|---|
| 1 | `5d60bcc88` | ⭐ o esqueleto **vira módulo** (4 crates, o `ph2d-ecs` encolhe) |
| 2 | `f5115ab78` … `ef2f069dc` | os 5 reports de smoke do dono sobre o desenho e o gesto |
| 3 | `f39a74fec` | ⛔ **defeito medido:** a pele morria no 1.º Ctrl+Z (o tendão guardava id de alocação) |
| 4 | `3a5d94849` `beb7484f2` | ⭐ **a ÂNCORA** (IK persistente) + o degrau 123 |
| 5 | `2457d8a3d` | ⛔ *«Add IK não funciona»* — a rota morta do bug #29, 4.ª vez |
| 6 | `8b85855e1` `8486703fd` | a fila do módulo + a sonda do undo |
| 7 | `7b3412ff2` | ⭐ duas âncoras numa cadeia deixam de brigar |
| 8 | `4388f1250` … `d12ad1e7b` | *«não funcionam plenamente»* + o losango que disputava o dedo |

### Smokes (todos passados pelo dono)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --release
```

Diagnóstico: `PH2D_BONE_LOG=1` · `PH2D_BONE_UNDO_PROBE=1` (o roteiro do undo) ·
`PH2D_BONE_UNDO_PROBE=2` (contagem de passos).

### ⚠️ O que NÃO foi smokado

- **Save/load de um `.ph2dproj` com esqueleto** — o degrau 123 recusa ficheiros anteriores e
  **não há nenhum com esqueleto** (medido 06/09: os dois `.ph2dproj` da máquina são de 26/08).
- **A âncora na timeline** — ela é um objecto com `Transform`, logo animável *por construção*; o
  caminho não foi exercido.
- **Um esqueleto dentro de uma instância/componente** (a `line/components` fechou a F5 no mesmo dia).

---

## §7 — SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O `ph2d-ecs` ENCOLHE, e isso é a wave, não uma regressão.** `VecBone`/`VecSkin` não foram
   apagados: mudaram-se para a `ph2d-skeleton-ecs`, com os **nomes canónicos sem «vec»** — porque um
   esqueleto serve raster, 3D e Flip, e o nome é a chave `blake3` do blob.
2. **O `ph2d-vec-scene/src/xform.rs` virou um `pub use` de uma linha** — o `Xform` mudou-se para a
   `ph2d-affine` (zero dependências). ⭐ **~300 sítios de chamada ficam intocados**, e é o precedente
   da `ph2d-arclen`.
3. **O `preview_drive` ganhou `release_to_authored`, e ele NÃO é um `settle` melhor.** A `settle`
   trata de um motor que **largou** (e aí o vivo *é* o documento); a porta nova trata de um motor
   **DESLIGADO**, e aí o vivo é dele. Dois factos com a mesma forma.
4. **O `undo_selection` ganhou duas famílias e a generalização ficou DE FORA de propósito** — é
   cerca de Chesterton: a nota original diz que alargar mudaria módulos que não o pediram.
5. **A ordem das âncoras é DERIVADA da hierarquia, não autorada.** O Spine deixa o artista
   reordenar; entre as duas, esta não precisa de UI nem de estado a gravar.
6. **O losango NÃO se enche quando seleccionado**, ao contrário de toda outra alça: ele é o de fora
   de um par concêntrico, e enchê-lo tapava a bolinha que vive por dentro.
7. **A `.github/workflows/miri.yml` não é desta linha** (ver §5).

---

## §8 — NOVE premissas minhas que a medição derrubou

1. *«os bits da entidade sobrevivem ao snapshot»* — a 1.ª redacção do gate restaurava para um
   `SimWorld::new()` e passava, porque num mundo virgem o alocador entrega os mesmos índices. O undo
   real re-spawna **no mesmo mundo**: `0 de 2` tendões. ⛔ **A fixtura não produzia o fenómeno.**
2. *«o `DEFAULT_ITERATIONS = 10` chega»* — a 12 ossos precisa de 22 e a 24 de **40**.
3. *«fora de alcance o FABRIK produz a recta sozinho»* (a doc herdada dizia isto) — medido `6,07` /
   `2,75` / `1,18` unidades fora de linha a 1, 10 e 64 passagens.
4. ⭐⭐⭐ *«o solver preserva a dobra»* — a álgebra diz `v1 × v2 = −l1·d·sin a`, e o código igualava os
   dois sinais: **cada passagem INVERTIA a dobra**. ⛔ E o gate que a defendia **codificava a
   inversão** (chamava `lado = +1` a um cotovelo cujo produto de entrada é negativo).
5. *«uma corrente parada não escreve»* — o braço da cena, **parado**, resolvia em `1485 de 1485`
   quadros com o punho a oscilar `±5e-4 rad`, e a amplitude a **crescer**.
6. *«há uma deriva por quadro»* (`913` supressões em 15 s) — **falso**: duas capturas no mesmo quadro
   são idênticas. *Um contador de supressões conta a mesma diferença N vezes, não N diferenças.*
7. *«o Add IK não é desfazível»* — pelo caminho completo do artista **é**. O que faltava era a
   **selecção**.
8. ⭐⭐ *«a restrição não declara condução quando assente, logo a pose do artista é engolida»* —
   construí a cura e o desenho **original** passou o mesmo gate ⇒ **revertida**. A regra da *outra
   mão* já a cobria.
9. *«convergir separa a ordem certa da errada»* (escrito na própria fila) — **não separa**: a cena
   assenta nas duas leis. Quem separa é a **independência da ordem**.

---

## §9 — QUATRO vermelhos que o portão do fecho apanhou

| o quê | cura |
|---|---|
| `every_descriptor_names_a_registered_component` | o teste do shell tem **cópia própria** da lista do `init.rs` (aquela vive no binário) — e a cópia **não pode ser fundida** |
| teto de LOC do shell (`628`, `680`, `718`) | **três cortes por responsabilidade** (`bone_pose_tests`, `skeleton_agenda_tests`, `skeleton_handle_tests`) |
| teto de LOC da escada (`618`) | corte por **IDADE**, a 2.ª vez: `project_schema_history_v83.rs` (mover a faixa para o arquivo que já existia punha-o em `730`) |
| teto de LOC do workspace (`723`) | `ph2d-skeleton-render/src/goal.rs` — o osso e a âncora são dois assuntos |

⛔ **Nenhum curado por isenção.** E uma `#[expect(dead_code)]` que eu escrevi era **falsa** (a função
é usada): apagada.

---

## §10 — Estado do portão batched

| | |
|---|---|
| `cargo clippy --workspace --all-targets` | **limpo** |
| `nextest-impacted.sh --no-fail-fast` | **14 425 / 14 427** · 1473 saltados |
| `collision-surface.sh` | sem marcadores, sem contrato tocado, sem teto estourado |

⚠️ **As 2 reprovadas são FLAKES DE CARGA catalogadas no `CLAUDE.md §5.0`**, e as três assinaturas
batem:

- `ph2d-tool-painter … a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` — **3 de
  3 verde sozinha** a `load 9,90`;
- `ph2d-host-desktop … flip_smooth::…::orcamento::the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke`
  — **3 de 3 verde sozinha** a `load 4,81`;
- **zero linhas do diff** em `ph2d-tool-painter`, `ph2d-flip*` ou `flip_fit_budget_tests.rs`;
- ⭐ **o CONJUNTO de reprovadas MUDOU entre duas corridas do mesmo binário** (a 1.ª acusou uma, a 2.ª
  as duas) — a assinatura que o §5.0 nomeia para distinguir carga de defeito.

---

## §11 — O que fica ABERTO

A fila viva é [`docs/Skeleton/01_a_fila.md`](../01_a_fila.md), com o mecanismo (ou o instrumento) de
cada item. Resumo:

| # | o quê | estado |
|---|---|---|
| F4 | *«undo tem poucos passos»* | ⏳ **ABERTO** — 5 arrastos ⇒ 5 passos, *Add IK* ⇒ 1 passo. Não reproduz; falta a sequência do dono. O instrumento existe. |
| F3 | pole target (que lado o joelho aponta) | a lei do lado existe; falta o alvo autorado |
| — | Smart Bones (Moho) · limites de ângulo por junta | nunca começados |
| — | a 2.ª mídia (raster/Flip) | ⛔ **bloqueada**: precisa de uma malha sobre a imagem |
| — | o painel próprio do módulo | ⏸️ adiado até haver conteúdo (3 botões + 5 campos hoje) |

⛔ **As cinco RECUSAS MEDIDAS** do módulo estão na mesma fila — não as reconstrua.

---

## §12 — Higiene do fecho

- `rm -rf target/*/incremental` — **feito** (§1.5.9 item 7). Medido: **44 GB** reclamados
  (`target/debug/incremental`); a árvore ficou em `82 GB`.
- Binário de release **compilado e quente** (§1.5.9 item 9), com a mesma linha de comando que o
  smoke entrega. A 2.ª corrida é a prova:

```text
$ cargo build -p ph2d-host-desktop --release
   Compiling ph2d-host-desktop v0.0.0 (…/Worktrees/line-Vector/shells/desktop)
    Finished `release` profile [optimized] target(s) in 3m 49s

$ cargo build -p ph2d-host-desktop --release
    Finished `release` profile [optimized] target(s) in 0.39s      ← ZERO "Compiling"
```
