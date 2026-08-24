# HANDOFF DE INTEGRAÇÃO — `line/Sprite`, 2026-08-23 (FECHO)

> ⚠️ **Este é o documento do INTEGRADOR** (DIRETRIZ §1.5.9): identidade, superfície de colisão,
> números que somam, e o que só o `ship.sh` apanha. **O MECANISMO de cada wave não está aqui** —
> ele vive em [`HANDOFF_INTEGRACAO_line_Sprite_ANIM_AUDIT_2026-08-23.md`](HANDOFF_INTEGRACAO_line_Sprite_ANIM_AUDIT_2026-08-23.md),
> §§1–18-ter, que é o registo da jornada. *Ler aquele para integrar é procurar um número dentro de
> uma narrativa.*

## 1. Identidade

| | |
|---|---|
| branch | `line/Sprite` |
| HEAD | `691d86d10` |
| merge-base com `main` | `35f937cb2` |
| commits da linha | **40** |
| diff | 145 ficheiros · +20 418 / −1 407 |
| gates novos | **215** `#[test]` · 23 ficheiros de teste novos |

⛔ **A linha está 103 commits ATRÁS do `main`** (entrou a linha `3DModeling` inteira, waves 35–55,
e outras). Ela **não foi rebaseada** — o `--ff-only` da §1.5.3 exige o rebase, e ele é do integrador
por desenho (é ele que resolve TODOS os conflitos, com as outras linhas à vista).

## 2. Superfície de colisão REAL — só **9** ficheiros os dois lados tocaram

Medido com `comm -12` entre `diff(base, HEAD)` e `diff(base, main)`:

| ficheiro | natureza | o que fazer |
|---|---|---|
| `Cargo.lock` | gerado | ⛔ **nunca resolver à mão** — regenerar |
| `docs/architecture/decisions/README.md` | **derivado** | `bash scripts/adr-index.sh` |
| `CLAUDE.md` | §5, os dois lados acrescentam | textual |
| `shells/desktop/Cargo.toml` | 2 deps minhas, append | textual |
| `shells/desktop/src/app_state.rs` | 2 campos meus, append | textual |
| `shells/desktop/src/main.rs` | 3 `mod` + 2 inits, append | textual |
| `shells/desktop/src/render_loop/mod.rs` | blocos dos dois lados | textual (o maior) |
| `shells/desktop/src/project_load.rs` | ⚠️ **conferido**: o `main` só INSERE (o «esquecer o documento anterior» do 3DModeling W23/W43); eu **apaguei** o `project_load()` sem caminho. Funções diferentes | conferir que nada no `main` novo chama `project_load()` |
| `shells/desktop/src/render_loop/present.rs` | ⚠️ **conferido**: o `main` insere a LUT do halo do `fx.glow`; eu mudo o **título da janela**. Regiões distintas | textual |

⚠️ **Tudo o resto é meu sozinho** — `crates/ph2d-aseprite` (novo), `panel-inspector` (23),
`editor-core` (18), `ph2d-ecs` (11), `runtime`/`render`/`asset`/`script` (2+2+2+1).

⚠️ **Cole a saída fresca de `collision-surface.sh`, não esta tabela.** A que está aqui mede o
`main` de **2026-08-23**; se a integração for noutro dia, todo número da coluna «base» mudou e este
documento **não reclama**. A divergência entre as duas leituras é ela própria um achado.

## 3. Números que SOMAM entre linhas — recontar contra o `main` do dia

| número | `main` em 23/08 | esta linha | nota |
|---|---|---|---|
| **`PROJECT_SCHEMA`** | **89** | **94** | ⚠️ **cinco degraus meus** (90 §12 montagem · 91 · 92 §11 Animation · 93 sinais · 94 duração por-quadro). Se outra linha subir antes, **recontar os cinco** |
| tripla do gate | `(89, 13, 14)` | `(94, 13, 14)` | ⚠️ **TRÊS sítios**: a escada em `project_schema.rs`, a tripla em `project_schema_tests.rs`. O `FLIP` e o `VEC_SCENE` **não** mexi |
| registo `ph2d-ecs` | 66 | **70** | +4 (`AnchorMount`, `SpriteAnimations`, `SpriteAnimator`, +1) — o gate conta, e os **dois espelhos** (`ph2d-render`, `ph2d-script`) contam também |
| ADR | último `0163` | máx. `0162` | ✅ **não crio número novo** — só `0072-amendment-1` (emenda, não número) |
| pacote novo no `Cargo.lock` | — | `ph2d-aseprite` | ✅ **nenhum pacote de TERCEIROS novo**: o `miniz_oxide` que ela usa já estava na árvore |

⚠️ **Ids de widget novos** (não colidem por valor — são hashes de string, mas o gate
`node_id_collisions` conta): `INSP_ANIM_*` (a §11 inteira, incl. `FRAME_MS_THIS`,
`SIGNAL_FINISH`, `SIGNAL_LOOP`), `INSP_SHEET_PREVIEW`, e o `CTX_MENU_*` já existia.

## 4. Contratos congelados (§6)

✅ **Nenhum encostado.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` **intocados**
(confirmado pelo `collision-surface.sh`).

## 5. Gate batched — VERDE, e o que ele NÃO apanha

```
CARGO_INCREMENTAL=0 bash scripts/nextest-impacted.sh --no-fail-fast
  → 10 753 / 10 753  ·  1 198 skipped
cargo fmt --all --check → limpo
```

⚠️ **A primeira corrida (sem `--no-fail-fast`) parou em 9 803/10 753** com **um** ✗:
`ph2d-tool-painter … the_mask_stroke_cost_does_not_follow_the_canvas` — **950 testes ficaram por
correr**. Ele passou **3 de 3** sozinho, o diff não toca uma linha do Painter, e **o `main` já o
nomeia** no §5.0 como membro da família de flakes de relógio (a nota chegou depois do meu fork).
*Um vermelho de flake esconde o resto da suíte.*

**O que só o `ship.sh` apanha** (o gate de integração não roda):
* `machete` — a shell ganhou **`miniz_oxide`** como dep DIRECTA (o `ase_smoke` escreve o ficheiro
  de demonstração). Ela é usada, mas é uma dep nova numa shell que não a tinha.
* `deny`/`audit` — `miniz_oxide` (MIT) e `ph2d-aseprite` (nossa) são as duas entradas novas.
* `typos` e o `doc-index --check` — este último confirmado em dia aqui (14 índices).
* clippy `--all-targets` da workspace: **a correr no fecho**; as quatro crates da linha
  (`host-desktop`, `editor-core`, `panel-inspector`, `ecs`, `aseprite`, `runtime`) estão limpas.

## 6. Ordem, dependências e o que RE-SMOKAR

**Ordem interna:** os 40 commits são sequenciais e não há dependência cruzada com outra linha.
⚠️ **Um único ponto de ordem:** o `PROJECT_SCHEMA` tem de ser recontado **antes** do gate da tripla
correr, senão ele reprova com o número certo no ficheiro errado.

**Smoke feito pelo Enio (OK):** `PH2D_ANIM_SMOKE`, `PH2D_ASE_SMOKE`, o import por drag & drop e pelo
diálogo, `Save`/`Save As`/`Open`, o Ctrl+Z das três famílias, a duração por-quadro.

**⚠️ O que NÃO foi smokado, e o integrador deve pedir:**
1. **Ctrl+Z com a TIMELINE a tocar** sobre uma cena com curvas do 3DModeling integrado — a wave da
   timeline foi construída contra o `main` antigo.
2. **Abrir um projeto GRAVADO ANTES** — o `PROJECT_SCHEMA` subiu cinco degraus; um ficheiro de v89
   **não abre**, e é isso que ele tem de fazer (falhar alto).
3. **`.ase` com tilemap ou paleta indexada** vindo de um artista real — os 18 ficheiros testados
   cobrem os cantos, mas nenhum é arte de produção dele.

## 7. Pendências que a linha DEIXA (nenhuma é bloqueio)

* ⛔ **`SpriteFrames` da spec §8.3** — recusa **medida**: o pool já é a grelha. Não reconstruir.
* ⏳ **4 goldens da spec** em `unimplemented!()` — falta um arnês de render headless.
* ⏳ **Luau · MCP · âncoras em modo de jogo** — os três dependem de subsistemas que hoje são casca
  (`ScriptHost` corre um placeholder, `McpHost` é um `MemoryHost`, não há `shells/game`).
* ⏳ **`AnchorData::user_data`** sem UI, com o `variant_editor` órfão a apontar-lhe.

## 8. Reclamado

`rm -rf target/*/incremental` corrido no fecho (DIRETRIZ §1.5.9 item 7).

---

*Linha `Sprite` pronta (HEAD `691d86d10`, 40 commits). Gate batched 10 753/10 753. Aguardo ordem de
integração.*
