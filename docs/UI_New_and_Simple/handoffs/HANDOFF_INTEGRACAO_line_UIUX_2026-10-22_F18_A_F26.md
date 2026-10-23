# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-22 — F18 a F26 (Auto Key em todo campo, canais por família, losango)

- Branch `line/UIUX` → `main` por `--ff-only`. Merge-base antigo `100d34c8e0`; rebaseada sobre `main` `6bbffdb64a`
  (Luz2D W6b–W6g) **sem conflito**. Backup: branch `line/UIUX-pre-rebase-2026-10-22`. **Sem push.**
- `git range-diff` de 70 commits: 69 `=`, 1 `!` — F24 `0f53447086` («linhas de GRUPO no dope-sheet»), só por contexto vizinho em
  `crates/ph2d-i18n/src/lib.rs` (o `mod board_live;` do MiroClone); conteúdo idêntico.
- Ondas F18→F26, todas com smoke do dono OK (a F26 em 22/10).

## 1 — O que entrou

Comando de todos os smokes: `cd /home/enio/Documentos/Projetos/PH2D && <ENV>=<n> cargo run -p ph2d-host-desktop --profile smoke`.

| onda | o quê | smoke |
|---|---|---|
| F18 | botões, interruptores e campos legíveis sobre o desenho; Show de Default azul | `PH2D_OBJECT_MODE_SMOKE=23/25/26/27/28` |
| F19 | painel de estilo lê cor e traço da forma seleccionada, segue Ctrl+Z; Preview bloqueia edição | `=28/29/30` |
| F20 | multi-selecção mostra «—» onde difere; Width não muda com o zoom | `=29..32` |
| F21 | seletor de cor e pilha de camadas mostram «misto» | `=33/34` |
| F22 | arrastar durante um Show fica onde se soltou; olho de camada misto | `=35/36` |
| F22b | opacidade do objecto anima por estado (`PROJECT_SCHEMA` 191→192) | `=37` |
| F23a | painel mostra o valor da curva; Auto Key não cria chave ao parar o relógio | `PH2D_AUTOKEY_SMOKE=1/2` |
| F24a | motor dos canais (`PropKind::Field`, `DOC_VERSION` 21) | `PH2D_AUTOKEY_SMOKE=3/4` |
| F24b | dope-sheet com linhas de grupo e losango por canal | `=3/4` |
| F24c | losango em todo campo com canal | `=5/6/7` |
| F24d | losango nos campos sem coluna; `L` = Timeline em todo modo (Elipse = `Shift+L`) | `=8/9/10` |
| F25 | todo campo de filtros, perfil, texto, objecto, flip e física grava no Auto Key (`DOC_VERSION` 22) | `=11..15` |
| F26 | todo campo de junta e roldana grava no Auto Key | `=16..26` |

Handoffs de onda: o último é [F26 — as juntas](HANDOFF_CONTINUACAO_line_UIUX_2026-10-22_F26_AS_JUNTAS.md); o mecanismo de cada onda vive nos
respectivos `HANDOFF_CONTINUACAO_line_UIUX_2026-10-2*` desta pasta (índice em [README](README.md)).

## 2 — Foundational (aditivo) e porquê

- **`ph2d-timeline`:** `DOC_VERSION` 20→22 (o `main` estava em 20 ⇒ sem recontagem). `from_bytes` migra v18..v21
  (`doc_v20.rs`, `doc_v21.rs`); fixtures `tests/fixtures/timeline_doc_v20.{bin,json}` e v21. `textos` é apêndice posicional; v21 abre com a tabela
  vazia. `PropKind::Field` é variante NOVA no fim (=19), 0–18 iguais. `PropKind::AUTOKEYED` 11→15 (`JointMotorTarget`/`Speed`/`RestLength`/`MaxLength`
  — entradas, não variantes).
- **`ph2d-ecs`:** `FieldKey` e vocabulários de canal, só append. `canal.rs`: famílias VETOR=1, OBJETO=2, JUNTA=3, FLIP=4, FISICA=5.
  `canal_vetor.rs` é ficheiro NOVO nesta linha (nenhum id da base existia); o gate `canal_tests.rs` fixa ~30 ids. `TINTA_LARGURA` 38→39 dentro da
  linha (F24→F25, nunca no `main`) — é LARGURA do bloco, não id gravado; as const-asserts `canal_tests.rs:113-114` impedem sobreposição; travá-lo
  bloquearia o próximo append (decisão do fecho). JUNTA: 1–4, 10–12, 20–22, 30–32, 40–48, 50–53, 60–67.
- **`ph2d-preview-drive`:** `Driver`/`Driven` + `SpriteCor`, `PontaNoMundo`, `Roldana`, no fim; sem braços fora da crate além de testes.
- **`PROJECT_SCHEMA` 191→192** (F22b; degrau `PROJECT_SCHEMA_OPACIDADE_FORA_DOS_ESTADOS = 191`; tripla `(192, 13, 23)`). O `main` estava em 191 ⇒
  sem recontagem. ⚠️ O prompt de integração dizia «`PROJECT_SCHEMA` intocado (192)» — está ERRADO: foi esta linha que o subiu.
- **Cargo:** `ph2d-app-flip` +`ph2d-timeline`; `ph2d-app-components` passa `ph2d-panel-inspector` e `ph2d-timeline` de dev-dep para dep;
  `ph2d-panel-timeline` +deps (F24b); `ph2d-panel-authored` +`ph2d-text` (F18, o gate do corpo pinta pela porta real). `Cargo.lock`: nenhum pacote
  externo novo.
- **Contratos §6:** `collision-surface`: `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` INTOCADOS; diff vazio em `ph2d-nodegraph*`,
  `ph2d-vector-traits`, `ph2d-vector-doc`. `ph2d-tool-vector` mudou API inerente (`take_apply_to_selected`/`take_width_authored` →
  `take_authored() -> StyleFields`), sem chamadores partidos — não é o trait `Tool`. Registo de componentes `108/109/109` = base. Sem ADR novo.

## 3 — Superfície de colisão (um merge pode partir)

`collision-surface.sh` contra `main` `6bbffdb64a`: 515 ficheiros; **7 tocados também pelo `main`** desde a base: `Cargo.lock`,
`ph2d-editor-core` (`interaction/dispatch/key.rs`, `interaction/state/mod.rs`, `store_core.rs`), `ph2d-i18n/src/lib.rs`, `ph2d-vec-edit/src/pencil.rs`,
`shells/desktop/src/main.rs`, `shells/desktop/src/undo_app.rs`. O rebase fundiu limpo. `main.rs` fica com 1059 linhas, na isenção de
`file_loc_caps.rs:36` (era 1060 na base). Se o `main` andar antes do ff: reconte o schema (`python3 scripts/schema-recount.py`) e o `DOC_VERSION`.

## 4 — Prova de fecho (22/10, 1× sobre o diff desde `100d34c8e0`)

- `fmt`: 1 ficheiro (`canais_junta_tests.rs`) formatado. `clippy` workspace `--all-targets --features ph2d-spike/bevy_ecs -D warnings` verde (1m02s).
  `check-standalone-optional` verde (11 crates); `check-workflow-packages` verde (32 nomes / 404 membros). `file_loc_caps` da shell 4/4;
  `the_shell_only_shrinks` verde.
- `PH2D_SUITE=fecho` `nextest-impacted`: **20 422 corridos, 20 419 passaram, 3 falharam**, 9 722 saltados, 1 058 s. As 3:
  - `workspace_src_files_under_loc_cap` — `ph2d-preview-drive/src/lib.rs` 752/700 (as variantes F25–F26). **CURADO** movendo
    `Driven::driver/read/write` para o irmão `facto.rs` (lib 474, facto 287), commit `3ad878845f` (pré-rebase); re-corrido verde, `clippy -p` e
    `fmt` verdes; nenhum gate lê o ficheiro pelo caminho.
  - `the_cost_of_sampling_a_path_is_flat_in_its_anchors` (2,18×) e `no_expression_allocates_no_link_frame` (4 blocos): JÁ na tabela de
    [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md) e na faixa isolada do nextest; ficheiros intocados pela linha; `loadavg` ~137 durante
    a suíte (outras linhas); verdes sozinhos (1/1 e 4/4).
- Depois do rebase: censos da árvore combinada 121/121, 12 de 12 censos correram.
- **Auditoria do fecho, 2 lentes** (foundational/formato gravado · caminho de gravação e undo): sem defeito novo; achados abaixo, em §6.

## 5 — Smoke (já feito pelo dono; para reconferir)

`cd /home/enio/Documentos/Projetos/PH2D && PH2D_AUTOKEY_SMOKE=<16..26> cargo run -p ph2d-host-desktop --profile smoke` (junta/roldana: ligue o Auto Key,
mexa um campo da junta, a curva ganha chave; `Ctrl+Z` desfaz). Errado = campo da junta muda e não aparece losango/chave.

## 6 — ABERTO, por ordem

Copiado do §6 do [handoff F26](HANDOFF_CONTINUACAO_line_UIUX_2026-10-22_F26_AS_JUNTAS.md):

1. ~~Smoke do dono da F26~~ — feito em 22/10 (cenas 16–26 OK).
2. **O ESCOPO do Inspector é uma lista:** outras secções (câmera, nav, vida, HUD, …) estão fora do gate e fora de `DIVIDA_CRATE` — o mesmo buraco
   que escondeu as juntas, por família (Componentes).
3. `IkBendSide` (chip do esqueleto) continua fora do `AUTOKEYED` de propósito — rever à luz de «grava tudo».
4. **Rope Length de uma Pulley** não se grava por Auto Key (só K), porque a ponte o re-deriva (§2.1 do F26).
5. A troca de tipo pela curva não passa pelo re-seed de limites quando escrita direto no componente (a cena escreve direto; o canal passa pelo funil).
6. Os itens 2–9 e 11 do §6 do [handoff F25](HANDOFF_CONTINUACAO_line_UIUX_2026-10-21_F25_O_BLOCO_2.md) continuam (o 10 está fechado pela F26).

Achados da auditoria do fecho:

7. Junta criada como «ponta no mundo» (`body_b == 0`) com chave `PONTA_NO_MUNDO = 0` fica dormente até o artista escolher o corpo B
   (`canais_junta.rs:321-324`; a mesma regra existe no painel, `fase_physics_edits.rs:47-49` — a curva reproduz a UI).
8. `prop.rs:50-54` diz que `Position` é auto-keyed por um ramo de geometria 2D, e esse ramo não foi achado — verificar.
9. As chaves `canal.junta.*` só foram vistas no locale base.

## 9 — Integração

_A preencher pelo integrador (depois do `ff-only`)._
