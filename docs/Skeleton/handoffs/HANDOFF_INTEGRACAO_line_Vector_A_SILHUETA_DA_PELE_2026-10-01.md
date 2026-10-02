# HANDOFF de INTEGRAÇÃO — `line/Vector`: o desenho FIEL da forma presa e a SILHUETA da pele (F37–F47, 2026-10-01)

> **Para o agente INTEGRADOR, noutra janela, por ordem do dono** (*«antes de seguir escreva handoff
> para integrar ao main»*). Leia este documento inteiro antes do primeiro comando. ⚠️ Ele descreve
> **só** os commits desta rodada; a anterior (o CAMPO e a pesquisa) já está no `main` e o documento
> dela é o [handoff de 2026-09-24](HANDOFF_INTEGRACAO_line_Vector_O_CAMPO_E_A_PESQUISA_2026-09-24.md).
> O mecanismo de cada wave, com as tabelas, vive na [fila](../01_a_fila.md) §F37–§F47.

## 0. Onde está e o que fazer

| | |
|---|---|
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| ramo | `line/Vector` |
| base | **`912a9652e`** — é o `main` de quando este doc foi escrito (`behind 0`): **nenhum rebase foi preciso** |
| forma | **fast-forward**, sem conflito possível enquanto o `main` não andar |
| commits | **17** (16 de produto + o deste handoff) |
| ⚠️ `CARGO_TARGET_DIR` | a worktree usa o `target/` **dela**. ⛔⛔ Nunca partilhe o target entre worktrees (troca os `.rlib`) |

**Passos, na ordem** (DIRETRIZ §1.5.3; `/pd-integracao`):

1. No primário: `cd /home/enio/Documentos/Projetos/PH2D && git status` — ficheiros alheios
   (`project-memory/` de outras sessões) não entram no merge; esta linha **não toca** em `project-memory/`.
2. Se o `main` tiver andado desde `912a9652e`: `git -C Worktrees/line-Vector rebase main` e **reconte**
   o mapa com `bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh` (⚠️ caminho
   ABSOLUTO; ⚠️ a coluna `base:` é o merge-base, não o `main` de agora).
3. `bash scripts/foundational-integrate.sh line/Vector` e `bash scripts/censos-da-arvore-combinada.sh`.
4. `git merge --ff-only line/Vector` no `main`. **Ship/push só por ordem explícita do dono** (§0.7).
5. Edite **uma linha** do `CLAUDE.md` §5 (a do Vector/Esqueleto) — texto proposto no §7 deste doc.

## 1. A superfície de colisão (`collision-surface.sh`, 2026-10-01, antes do commit deste handoff)

```
merge-base 912a9652e · 16 commit(s) · 57 arquivo(s)
PROJECT_SCHEMA 176 (base: 176) · tripla (176, 13, 22) · VEC_SCENE_SCHEMA 22 (base: 22)
FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23              — todos = base
ph2d-ecs 108 · ph2d-render (espelho) 109 · ph2d-script (espelho) 109 — todos = base
crates/ph2d-nodegraph/src/node.rs intocado · crates/ph2d-editor-core/src/tool.rs intocado
ADR: esta linha não cria ADR (próximo livre no disco: 0176)
Cargo.lock: nenhum '+name' novo (só a aresta interna ph2d-skeleton-live → ph2d-vec-boolean)
marcadores de conflito: nenhum · tectos de LOC nos ficheiros tocados: nenhum passa
```

⇒ **Zero contador partilhado, zero contrato congelado, zero ADR, zero pacote externo, zero schema.**
Nada desta rodada muda bytes gravados: tudo o que ela produz é **derivado por quadro**.

## 2. Foundational / partilhado tocado — e porque é seguro

| ficheiro | o que muda | porque é seguro |
|---|---|---|
| `crates/ph2d-vec-scene/src/stroke_style.rs` (+`lib.rs`) | **nova** `pub const MITER_LIMIT: f64 = 4.0`, re-exportada | é o valor de omissão da `kurbo`/Vello/Skia/SVG ⇒ o ecrã não muda um pixel; é uma PORTA com três leitores |
| `crates/ph2d-vec-render/src/lib.rs` | `kurbo_stroke` escreve `.with_miter_limit(MITER_LIMIT)` | `4` = o que a kurbo já usava em silêncio |
| `crates/ph2d-vec-boolean/src/expand.rs` | o *Outline Stroke* lê a mesma porta | idem |
| `crates/ph2d-app-vec/src/svg_export.rs` | junta `Miter` escreve `stroke-miterlimit="4"` | ⚠️ **o SVG exportado ganha um atributo** (o valor é o de omissão do SVG — o desenho do ficheiro não muda, os BYTES mudam) |
| `crates/ph2d-app-vec/src/state.rs` | campo novo `VecState::skin_desenhado` | derivado, refeito a cada quadro, nunca documento (doc-comment no campo) |
| `shells/desktop/src/render_loop/fase_vector_view_and_drives.rs` | `skeleton_live::recook(..)` → `self.vec.skin_desenhado = skeleton_live::recook_desenhando(..)` | o recook é o mesmo; passa a devolver também o desenho que se VÊ |
| `shells/desktop/src/render_loop/fase_vector_live_geometry.rs` | `skin_desenho::funde(..)` na geometria viva, **por último entre os que estendem e antes da booleana** | ⚠️ ORDEM load-bearing: a forma presa CEDE a um offset/padrão/largura viva, e a booleana consome o que os operandos DESENHAM |
| `crates/ph2d-editor-core/tests/it/architecture_the_pose_motors_run_before_the_mesh.rs` | a agulha passa a `skeleton_live::recook_desenhando(` | só gate — ⛔ a agulha antiga já não casa; não a «restaure» |
| `crates/ph2d-app-motion/src/motion_object_bake_dims.rs` · `shells/desktop/src/texture_pattern_pick_tests.rs` | só doc-comments a citar a porta | — |

As crates do módulo (`ph2d-vec-boolean` — os módulos novos `bola`, `bola_toque`, `gancho`,
`esporao`, `ilha`, e o `overlap` —, `ph2d-skeleton-live`, `ph2d-vec-skin`, e as cenas em
`ph2d-app-vec/src/smoke_bone*.rs`) são **da linha**.

## 3. O que a linha faz — por wave, com os números MEDIDOS

| commit | wave | o quê | número |
|---|---|---|---|
| `9e1977e95` `63687cc81` `c7d10ddf9` `eff707168` | **F37** | **o desenho FIEL**: o caminho da cena fica com os nós do artista e o que se VÊ é o bake segue o padrão-ouro (rotas A+B da pesquisa 04); uma gaveta por forma presa; o ajuste da cúbica confere nos DOIS sentidos (o `fit_to_cubic` da kurbo espetava) | desvio ao padrão-ouro `0,253/0,496/0,460 → 0,0018/0,0033/0,0043` a `30°/60°/90°` (`108–149×`). `PH2D_SKIN_DESENHO=0` bissecta |
| `6333954bb` | — | cena `PH2D_VEC_BONE_SMOKE=3` — o PAR (desenho e imagem com esqueletos iguais) | — |
| `1e5f53f2a` | **F38** | **a dobra forte**: no contacto a pele fica na fronteira da UNIÃO dos membros (`A ∪ ∅` pela `ph2d-vec-boolean`), só no DESENHADO, nunca nos nós | fora do contacto **ao bit** o de antes; `16`–`26 µs`/forma a detectar, `138 µs` a unir. `PH2D_SKIN_CONTACTO=0` bissecta. Cena **`=4`** |
| `7575ac0f5` | **F39** | a quina do contacto obedece à junta do painel: solda dos segmentos minúsculos, alças a ruído limpas, o limite do bico numa porta (`MITER_LIMIT`) | — |
| `1cfde53cd` | **F40** | o vinco que a união cria vira **arco** (decisão do dono: arredondado); `MITER_LIMIT` **volta a `4`** | o `10` fazia o bico atravessar a peça |
| `fcfbb4b95` | **F41** | **a bola que rola**: fecho morfológico de raio `r = 1 %` da diagonal, só no lado côncavo, SEMPRE (contínuo antes e depois do encosto) | mutação `27/30`, as 3 nomeadas |
| `e747d7f18` | **F42** | as quinas do artista vêm do **REPOUSO** (os nós da fonte), não do desenho deformado | gancho no desenho em `16` poses → `0` |
| `a69cdc7ad` `db3710e12` | **F43** | `gancho::desfaz_os_ganchos` antes da união; tangentes EXACTAS das pontas | `1 053` poses sem nó acima de `15°` |
| `49d723367` | **F44** | **as ilhas**: a bola rola por DENTRO do buraco; ilha onde a bola não cabe é cheia | `595 → 0` poses com canto de ilha |
| `a31455065` | **F45** | o desfazer dos ganchos corre também DEPOIS da união; recuo sobre o próprio caminho até ao tamanho da bola | — |
| `3f8afaeea` | **F46** | **o braço dobrado de volta**: abertura (a bola por dentro) entre dois fechos · `esporao` (o recuo de área zero) · arco do nó convexo · centro exacto como recurso | `574` poses curadas, `0` pioradas; fora da família o contorno andou `≤ 0,40` solda |
| `fb327123c` | **F47** | **o esporão DENTRO da cúbica** (aparar na 1.ª passagem pelo nó, só se a quina exposta passar de `150°`); esporão e aparar respeitam as quinas do artista | a varredura FECHA: **`0` de `16 110`** poses com nó acima de `15°` (pior `10,9°`) |

A porta de tudo é [`ph2d_vec_boolean::overlap::silhueta_da_pele(path, quinas)`](../../../crates/ph2d-vec-boolean/src/overlap.rs):
ganchos → união → ganchos → esporões → fecho → abertura → fecho → ilhas.

## 4. O que uma leitura rápida do diff entende AO CONTRÁRIO

1. **`MITER_LIMIT = 4` não muda o ecrã** — é o número que a kurbo já usava. Ele **foi a `10` (F39) e
   voltou (F40)** no mesmo dia; ⛔ não o suba: com `10` o bico do vinco atravessava a peça (doc-comment
   da const tem a medição).
2. **O SVG exportado muda de BYTES** (`stroke-miterlimit="4"` em toda junta `Miter`) e não de desenho.
   Um golden de SVG de outra linha com `Miter` acusa isto — **a mudança é a cura**, não um defeito.
3. **As sondas `skinned_mesh_arap_sonda_tests.rs` e `skinned_mesh_contacto_sonda_tests.rs` (~1 070 linhas,
   todas `#[ignore]`) NÃO são código morto:** são as duas curas do estado da arte na MALHA, construídas,
   medidas e **RECUSADAS** — o registo da recusa. Não as apague.
4. **`resolve_overlap` corre só no DESENHADO** (o que se vê), **nunca no `cru`** (os nós que o artista
   edita). Um gate exige-o.
5. **A ordem dos passos dentro de `silhueta_da_pele` é load-bearing** e cada passo tem gate e mutação; o
   doc-comment de cada módulo diz porquê (ex.: o desfazer dos ganchos corre ANTES e DEPOIS da união; a
   abertura usa TODAS as quinas como parede de `180°`).
6. **Três portas de ambiente, nenhuma é knob de produto:** `PH2D_SKIN_DESENHO` e `PH2D_SKIN_CONTACTO`
   nascem **LIGADAS** (`=0` desliga, para bissecar); `PH2D_SONDA_*` só existem nas sondas.

## 5. Prova de fecho

Sobre a linha inteira (`main..HEAD`), 2026-10-01, `load ~22–33` (gates de relógio: ver a família de
flakes do `CLAUDE.md` §5.0 antes de suspeitar de um merge):

| portão | resultado |
|---|---|
| `BASE=main bash scripts/nextest-impacted.sh` (pela porta `ph2d-run.sh`) | **`17 758 / 17 758` verdes**, `12 610` fora do impacto |
| `bash scripts/censos-da-arvore-combinada.sh` | **`127 / 127` verdes** · controlo do filtro `12` de `12` |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | limpo |
| `cargo clippy --all-targets -D warnings` nas crates da linha + `ph2d-host-desktop` | zero avisos (`ph2d-vec-boolean` · `-skeleton-live` · `-vec-skin` · `-app-vec` · `-vec-scene` · `-vec-render` · `-app-motion` · `ph2d-host-desktop`) |
| `cargo machete` | nenhuma dependência sem uso |
| `scripts/check-standalone-optional.sh` · `scripts/check-workflow-packages.sh` | verdes |
| `cargo fmt --all --check` | limpo (⚠️ o fecho apanhou um ficheiro desta linha por formatar — `smoke_bone_par.rs`, curado no commit deste handoff) |
| `bash scripts/doc-index.sh --check` | `20` índices em dia |
| os três arch-gates que leem docs (`architecture_docs_paths_and_smokes_resolve` · `architecture_docs_reference_live_gates` · `architecture_no_restricted_source_citations`), DEPOIS de escrito este doc | `9 / 9` verdes |
| `#[cfg(target_os` no diff | **zero** ⇒ nada a cruzar para macOS/Windows |
| `collision-surface.sh` | §1 |

⚠️ **Mutação, por wave** (nos commits, cada uma com o arnês e o controlo do filtro): F38 `7/7` · F39
`16/16` · F40 `23/23` · F41 `27/30` (3 nomeadas) · F42 `3/4` (1 equivalente, nomeada) · F43 `5/5` ·
F44 `7/7` · F46 `6/6` · F47 `6/6`. **Varredura da dobra** (`diag_a_varredura_da_dobra`, passo `2`,
`16 110` poses C e Z): `575` poses com nó acima de `15°` antes da F46 → **`0`** depois da F47.

## 6. ABERTO — com dono

- ⏳ **A IMAGEM presa** (a 2.ª mídia) mostra um risquinho no encontro dos membros na pose
  `(36°, −144°)` da cena `=4` — visto na foto do fecho; é o caminho da malha da imagem, que esta
  rodada não tocou. Próximo trabalho da linha.
- ⏳ **O vinco do cotovelo perto de `90°`** é a dobra do mapa do próprio padrão-ouro (`det J`,
  pesquisa 04 §1.3); a cura são as rotas C/D, interrompidas — **decisão do dono**.
- ⏳ O caminho de **GPU** da imagem não conhece as manchas de peso nem a lei nova da junta (dívida com gate).
- ⏳ Forma com **EFEITOS** não passa pelo desenho fiel (o efeito corre sobre os nós) — declarado.
- ⏳ **Decisões do dono paradas:** ligar o bake no desenho (`~135×` mais fiel por `5,7×` o relógio) ·
  o `Strength` do envelope inerte em arte preenchida.

## 7. A linha do `CLAUDE.md` §5 (proposta, para o integrador escrever)

No bullet **Vector**, depois da frase do fecho de 2026-09-24, acrescentar UMA frase:

> ⭐⭐⭐ **E A SILHUETA DA PELE FECHOU (2026-10-01, F37–F47, 16 commits, zero contador/contrato/ADR):** a
> forma presa a ossos mostra o desenho FIEL (`108–149×` mais perto do padrão-ouro) e o contorno passa por
> `ph2d_vec_boolean::overlap::silhueta_da_pele` — união no contacto, a bola que rola (`r = 1 %` da
> diagonal) por fora e por dentro das ilhas, ganchos e esporões tirados —, com **`0` de `16 110`** poses
> com nó acima de `15°`; `MITER_LIMIT = 4` é porta única (ecrã · *Outline* · SVG) —
> [handoff](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A_SILHUETA_DA_PELE_2026-10-01.md).
> Cenas `PH2D_VEC_BONE_SMOKE=3|4` (`PH2D_VEC_BONE_DOBRA`/`DOBRA2`/`TRACO`).

## 8. Smoke — o que o dono vê (aprovado por ele em cada wave; o último: *«não vejo problemas»*)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=4 PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 PH2D_VEC_BONE_TRACO=0.3 cargo run -p ph2d-host-desktop --profile smoke
```

1. O desenho de cima (contorno castanho) é um braço dobrado de volta: no encontro dos membros a borda
   de dentro é uma curva lisa. Errado: um bico fino ou uma pontinha a sair do contorno.
2. Trocar para `PH2D_VEC_BONE_DOBRA=0 PH2D_VEC_BONE_DOBRA2=148`: o canto de dentro sai arredondado.
   Errado: uma pontinha no canto.
3. Sem as duas juntas (`env PH2D_VEC_BONE_SMOKE=4` só): a dobra forte de `120°` sai com cantos em «V»
   arredondados e sem laços dentro das juntas.
