# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-24 — F27 (todo campo do Inspector é um canal; a palavra de uma escolha; uma propriedade, um canal)

- `line/UIUX` → `main` por `--ff-only`. Ordem do Enio (24/10): «smoke OK. Vamos integrar ao main». **Sem push, sem ship** (só por ordem explícita).
- Merge-base `b3d38f53c1`; ponta antes do rebase `08e58f9c73`; `main` na integração `736c0efb76` (97 commits à frente: Luz 2D W6h–W6i,
  MiroClone W24–W26, Macros W17–W19). Só a UIUX integrou nesta rodada (ordem trivial).
- Ficheiros tocados também pelo `main`: `CLAUDE.md`, `crates/ph2d-ecs/src/lib.rs`, `docs/architecture/decisions/README.md`,
  `docs/Components/ferramentas/fotografa_cena.sh`.
- Smokes do dono APROVADOS: `=29` (B2), `=30` (B3+C), `=31` (D), `=32` (E), `=33` (F, 24/10); `=27`/`=28` (A, B1) em 23/10.

## 1 — O que entrou (um handoff de continuação por onda; o mecanismo vive lá)

Comando de todos os smokes: `cd /home/enio/Documentos/Projetos/PH2D && PH2D_AUTOKEY_SMOKE=<n> cargo run -p ph2d-host-desktop --profile smoke`.

| onda | o quê | smoke | handoff |
|---|---|---|---|
| A | lado B da junta («ponta no mundo» com `body_b == 0`) e motor com faixa | `=27` | [Bloco A](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_BLOCO_A.md) |
| B1 | família de canais `COMPONENTE = 6` e a Câmera de ponta a ponta | `=28` | [B1](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B1_COMPONENTE.md) |
| B2 | todo campo do Inspector é um canal (`Rotulo` tipado) | `=29` | [B2](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B2_TODO_CAMPO.md) |
| B3+C | canal de texto na dope-sheet; painéis sem lado; `marcas()` 51,5 → 0,23 ms | `=30` | [B3+C](HANDOFF_CONTINUACAO_line_UIUX_2026-10-23_F27_B3_C.md) |
| D | menu da linha de GRUPO; ADR-0191; dívidas do catálogo | `=31` | [D](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_D.md) |
| E | fonte do HUD, ficheiro do script e props do script são canais (`SCRIPT = 7`, `TimelineDoc` v23) | `=32` | [E](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_E.md) |
| F | a coluna do valor escreve a PALAVRA; um losango no Script; tabela `DONOS` (uma propriedade, um canal) | `=33` | [F](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_F.md) |

Planos: [`F27_bloco_B_plano.md`](../F27_bloco_B_plano.md) · [`F27_E_plano.md`](../F27_E_plano.md) · [`F27_F_plano.md`](../F27_F_plano.md).

## 2 — Rebase e recontagens (contra o `main` `736c0efb76`)

- Rebase de 36 commits, **2 conflitos**: (1) `docs/Components/ferramentas/fotografa_cena.sh` no `f284232c05` (Bloco A): os dois lados
  escreveram a MESMA cura (guarda da sessão do dono lida de `WAYLAND_DISPLAY`/`DISPLAY` de quem chama); ficou a forma do `main` (`:2`),
  a da linha (`DONO_WL`/`DONO_X`) saiu. (2) `docs/architecture/decisions/README.md` (gerado) no `34265829b3`: regenerado por
  `bash scripts/adr-index.sh` (196 ADRs). Nenhum Mergiraf «Solved». `git range-diff`: só esses dois commits diferem, nesses hunks;
  os outros 34 são `=`. Zero marcadores de conflito (incluindo `|||||||`).
- **`PROJECT_SCHEMA` 193 = `main`** (tripla `(193, 13, 23)`); a linha não o move.
- **`TimelineDoc` `DOC_VERSION` 22 → 23** (o `main` está em 22; a fixture `timeline_doc_v22` segue a do degrau anterior).
  Quem a subir depois parte de 23.
- **ADR:** `main` em 0190 ⇒ a linha cria **0191** («o canal de qualquer propriedade é o catálogo de componentes, ligado pelo rótulo»),
  com adendos E e F.
- Registo de componentes `108/109/109` inalterado. Cenas `PH2D_AUTOKEY_SMOKE` vão até **33**; a próxima é **34**.
- Contrato congelado §6 intocado. `Cargo.lock`: nenhum pacote externo novo. Tetos de LOC respeitados.
- Código novo do `main` não usa `options!`/`CampoInfo.discreto`/`.texto`/`View::Options` (varrido): a colisão (a)–(d) do F §3 não disparou.

## 3 — Superfície de colisão (um merge pode partir)

Contra as linhas VIVAS ainda por integrar (par a par: ficheiros que a UIUX tocou ∩ os da outra desde o seu merge-base):

| linha | commits à frente | ficheiros em comum |
|---|---|---|
| `line/CodeEditor` | 35 | 13: `CLAUDE.md`, `ph2d-app-components/src/{lib.rs,script_inspector.rs}`, `ph2d-component-desc/src/catalog/core.rs`, `ph2d-ecs/src/lib.rs`, `ph2d-editor-core/src/{script_edits.rs,widget/mod.rs,widget/property_box/mod.rs}`, `ph2d-panel-inspector/src/sections/{camera.rs,hud_corpo.rs,parallax.rs,rows.rs,script.rs}` |
| `line/Vector` | 24 | 4: `CLAUDE.md`, `ph2d-app-vec/src/lib.rs`, `ph2d-panel-inspector/src/sections/mod.rs`, `ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs` |
| `line/PainterWatercolor` | 33 | 2: `CLAUDE.md`, `fotografa_cena.sh` (já curada, ver §2) |

**Quebras semânticas para quem integra a CodeEditor** (F §3 e E §3):

- `options!` mudou de forma: `options!["x" => X @ "chave do cartão"]`; `View::Options = &[Opcao { nome, valor, rotulo }]`.
  A CodeEditor lê as opções do catálogo (`property.rs` usa `Opcao.nome/valor`): reconciliar.
- `CampoInfo.discreto` / `.texto` são MÉTODOS (campos viraram `especie: Especie`).
- Tabela `DONOS` em `crates/ph2d-ecs/src/canal_componente_donos.rs` (60 campos / 81 faixas): a propriedade de uma família à mão não é
  canal do catálogo. `canal_palavra.rs` novo.
- `ph2d.get(e, "label.source")` devolve o NOME da variante; existe `label.source_name`. `stored_from` ganhou parâmetro (E §3).
- `ph2d-script` **não tocado** por esta linha.

## 4 — Prova de fecho (árvore combinada)

- `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` verde · `cargo clippy --workspace --all-targets -- -D warnings` verde ·
  `cargo fmt --all` limpo · `ph2d-tool-sync`/`node-sync`/`app-sync` sem diferença · staleness dos registos verde.
- Por onda: suíte impactada, provas de mutação (B1 4/4, F 8/8 + 7/7 DONOS) e fotos nos handoffs linkados em §1.

## 5 — Suíte da árvore combinada

`PH2D_SUITE=fecho BASE=main bash scripts/ph2d-run.sh bash scripts/nextest-impacted.sh` sobre a ponta rebaseada (o futuro main),
1 224 s: **19 428 corridos · 19 427 verdes · 1 TIMEOUT · 11 948 saltados**. `loadavg` 4,05 no início, 36,9 (5 min) no fim.

- O único vermelho é o de CARGA conhecido: `ph2d-panel-painter-layers`
  `seam_estado_escondido::cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo` (TIMEOUT 360 s). Sozinho por nome
  (`cargo-test-narrow.sh`), logo a seguir: **verde**, `loadavg` 8,6. Nenhum ficheiro desta linha está nessa crate.
- O `a_cena_3_conduzida_passo_a_passo` (`ph2d-host-code-scenes`) e os `g4_*` da Luz 2D não reprovaram desta vez.
- Antes da suíte: `ph2d-tool-sync`/`node-sync`/`app-sync` sem diferença e a staleness dos três registos verde.

## 6 — Smoke (já feito pelo dono; para reconferir)

Passos do `=33` (os das outras ondas estão no §4 de cada handoff):

1. `cd /home/enio/Documentos/Projetos/PH2D && PH2D_AUTOKEY_SMOKE=33 cargo run -p ph2d-host-desktop --profile smoke`
2. Abre com 4 quadrados e a linha do tempo no quadro 18 com 7 linhas; na coluna do valor leem-se PALAVRAS (On, Front, Tag, Enemy,
   Multiply, bob.luau), e «speed» é número.
3. Spinner seleccionado: na secção Script o losango só aparece ao lado da lista de scripts.
4. Errado: número (3.00, 5.00) numa linha de escolha; duas linhas «Red · Tint Fill»; dois losangos na secção Script.

## 7 — ABERTO, por ordem

Do §6 do [handoff F](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_F.md) (itens 2–4):

1. **FORA do escopo:** itens de LISTA (Timers, Tweens, StateMachine, CounterWatch, SignalActions, VecAnchors, ShakeEmitter,
   resistências), `ph2d-panel-painter-layers` (decisão do dono 23/10), gravidade do mundo, física a meio, `IkBendSide`, Rope Length,
   áudio do mixer.
2. **Achados do censo da palavra, não curados:** `MISTURA`/`PILHA_MISTURA` guardam 22 códigos e o menu lista 19; `ENVELOPE_WARP` pinta
   presets em inglês cru (HR-15); `TINTA_TRACO` com buracos 1–3 e `N_PARAGENS`/`N_PONTOS` discretos sem leitor;
   `PILHA_DILATA_JUNCAO` e `TEXTO_ALINHAMENTO` aceitam 0..255 em vez de 0..2.
3. Próxima cena: `PH2D_AUTOKEY_SMOKE=34`.
4. Herdados do [handoff F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md) §6, **não re-verificados no código nesta
   integração**: 5 (troca de tipo pela curva sem re-seed de limites na escrita directa), 6 (F25 §6 itens 2–9 e 11),
   9 (`canal.junta.*` só vistas no locale base). Os 2–4 desse §6 estão cobertos ou recusados pela F27 (item 1 acima).
