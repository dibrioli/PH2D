# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 20/10 — F24a: o motor dos canais feito (o Auto Key grava o desenho); o dope-sheet agrupado e o losango por fazer

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `100d34c8e0` (sem rebase). Anterior:
  [F23a](HANDOFF_CONTINUACAO_line_UIUX_2026-10-19_F23a_O_PAINEL_LE_A_CURVA.md) (§6 = a ordem do dono que abriu esta onda).
- Plano: [`35_plano_F24_o_auto_key_grava_tudo.md`](../vetor_nova_ui/35_plano_F24_o_auto_key_grava_tudo.md) (§3.0 = o desenho COMO CONSTRUÍDO; §9 = decisões do dono 20/10: (1) mover um ponto grava a forma INTEIRA (AE/Spine), inserir/apagar propaga a todas as chaves; (2) a pintura do Painter passa a quadro-a-quadro como o Flip — Bloco 2).
- Oráculos corridos nesta máquina (Godot 4.7.2 MIT; Blender 5.2.2 GPL, só oráculo): [`../vetor_nova_ui/oraculos/`](../vetor_nova_ui/oraculos/) (scripts + `.saida.txt`).
- Ordem do dono: memória `project_autokey_records_everything_2026_10_19.md` (decisões acrescentadas).

## 1 — Commits (do mais antigo)

| commit | o quê |
|---|---|
| `b32bf06c12` | plano 35 |
| `a45c8ee8ef` | decisões do dono (plano §9) |
| `fc5d23ec2c` | `PropKind::Field` + `CanaisLidos`/`CanaisEscritos` + vocabulário do vetor + discreto Hold e SUBSTITUI na pilha (ADR-0115 R7) + `DOC_VERSION` 21 com fixtura v20 |
| `67bdbe7e56` | `autokey_campos` (o grupo grava-se inteiro) + `curve_seen_b`/`key_value_b` + ECO |
| `c5e76204e3` | família VETOR: `canais_campos`, `canais` (publicar/aplicar/`ConducaoVetor`) |
| `58e6e2e651` | shell: aplicar depois do apply (`fase_timeline_drain`), publicar antes do autokey (`fase_autokey`: população + entidades com binding Field), `autokey_pass` compara campos (`AutokeyState.campos`/`base_campos`), `undo_app`: `conducao.settle()` junto do `preview_drive.settle()` e substituir/repor à volta do `ProjectState::capture`; amostragem movida para `autokey_amostra.rs` (tecto 600) |
| `cef459d6cd` | cena `PH2D_AUTOKEY_SMOKE=3\|4` (`crates/ph2d-app-vec/src/autokey_cena_desenho.rs`, chamada por `autokey_cena.rs`) |
| `35cb238ae9` | `canais_topologia`: inserir/apagar ponto propaga a todas as chaves; `propagar_topologia` no `autokey_pass` |

## 2 — O desenho como construído (o que a próxima LLM herda)

- `PropKind::Field(ph2d_ecs::FieldKey)` = variante 19, APPENDED (fio congelado só por append). `target()` passou a `match`; `Field` → `u64::MAX`, nunca `from_target`.
- `FieldKey{familia,campo,i,j}` (u16), números DECLARADOS, append-only: gate `o_vocabulario_do_vetor_nao_renumera` (`crates/ph2d-ecs/src/canal_tests.rs`).
- Porta única «o que é este campo»: `ph2d_ecs::canal::info(k)` → `CampoInfo{rotulo i18n, discreto, grupo, por_i}`. Vocabulário do vetor em `crates/ph2d-ecs/src/canal_vetor.rs`: Forma 1–9 (`i`=contorno, `j`=vértice), REGRA 10, tinta em blocos de largura 38 a partir de 20 (fill) e 60 (stroke paint), traço 100–111, MISTURA 140, pilha 150–156, efeito 170–171, forma paramétrica 180–182, texto 190–196.
- Tudo é Float. Cor = 4 campos sRGB u8 0..255 do MESMO grupo. Discreto = índice com `Interp::Hold` forçado por `PropKind::interp_of` em `doc.insert_key`/`upsert_key`. `stack_eval`: o discreto SUBSTITUI (membro de maior peso na lane; a lane substitui com influência ≥ 0,5) em vez de misturar.
- Leitura/escrita: `write_prop` braço Field → recurso `CanaisEscritos`; `read_prop_kind` braço Field → `CanaisLidos`; `sample_prop_value` (shell) braço Field → `CanaisLidos`.
- **ECO** (`CanaisLidos`): a família regista (escrito, ficou) depois de escrever; `publicar` devolve o valor da curva enquanto o doc == `ficou` — esconde a quantização (u8/inteiro/clamp), logo sem chave-fantasma. `canais_campos::escrever` só escreve se `ler` difere (identidade f64 doc × f32 canal).
- Opacidade do vetor continua `PropKind::Opacity` (overlay F23): sem Field duplicado.
- Formato: timeline `DOC_VERSION` 20→21; `src/doc_v20.rs` recarimba a versão; fixtura `tests/fixtures/timeline_doc_v20.{bin,json}` (+ linha no README). `PROJECT_SCHEMA` e schema do `VecScene` intocados. `AnimValue` (§6), contrato Tool e nodegraph intocados.
- Topologia: `canais_topologia` alinha as âncoras intocadas por LCS exacto; o ponto novo divide o segmento de cada chave no mesmo `u` (de Casteljau); o apagado sai de todas. Corre no `autokey_pass` num passo de undo da timeline próprio ou no do gesto aberto.

## 3 — Superfície de colisão (um merge pode partir)

- **`PropKind` (variante nova): todo `match` exaustivo do workspace, testes incluídos** — `prop_readback.rs`, `clip_stack_autokey.rs`, `ph2d-panel-timeline` `tracks_label.rs`, shell `timeline_bridge_keys.rs`. `cargo check --workspace` NÃO corrido: outras crates podem partir.
- **Timeline `DOC_VERSION` 21**: outra linha que suba o doc da timeline recontar e encadear migrações.
- `ph2d-ecs` `lib.rs` (`mod canal`, `canal_vetor`, `pub use`); `ph2d-i18n` `lib.rs` (`mod canais` + `.or_else(canais::tr)` na cadeia) + `crates/ph2d-i18n/src/canais.rs` novo.
- `ph2d-app-vec` `lib.rs`/`state.rs` (`VecState.conducao`).
- Shell: `autokey_pass.rs` (570/600), `autokey_state.rs`, `fase_autokey.rs`, `fase_timeline_drain.rs`, `undo_app.rs`. `stack_eval.rs` `eval_frame` (ramo discreto). `autokey.rs` refatorado (`curve_seen_b`, `shown_value_b`, `curve_value_b`, `key_value_b`) — manter as chamadas num merge.
- Não há intents novos (`intent.rs` segue a 695/700).

## 4 — Smoke (o dono, depois que §5.1+§5.2 existirem)

Ainda NÃO é para o dono: o dope-sheet mostra UMA LINHA POR CAMPO («Path 0 · Fill» ×6; a Forma seriam 81 linhas) e o losango não existe nos campos. Smoke interino para quem desenvolve:
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=4 PH2D_AUTOKEY_LOG=1 ./target/smoke/ph2d-host-desktop` (`=3` fica parada, `=4` mexe sozinha).
2. Estrela livre selecionada, Auto Key armado: mude a cor do preenchimento, a largura do traço, o canto, mova um ponto; pare o relógio noutro quadro e repita.
3. Tem de acontecer: cada edição cria chave na timeline; a log `PH2D_AUTOKEY_LOG` mostra faixas novas; no quadro do meio a cor é o meio por componente.
4. Errado = uma edição sem chave, chave só por parar o relógio, ou a forma saltar para o valor gravado.

Medido no app real (cena `=4`, tela virtual, ANTES de a cena ser reescalada em `cef459d6cd`: estrela 0,6→1,8, traço 0,04→0,14, tracejado 0,12/0,06, canto 0,15, ponto +0,4): fill verde ⇒ 6 faixas (existe, tipo, rgba), 1 chave; stroke ⇒ 1 faixa; canto+ponto ⇒ Forma com 81 faixas (10 vértices × 8 + fechado), 1 chave; roxo + stroke de volta ⇒ 2.ª chave; q18 meio: cor (65,120,145) = meio exacto por componente de (0,200,90) e (130,40,200); traço 0,04 = meio de 0,06 e 0,02. As fotos ficaram só no scratchpad (não commitadas).

## 5 — Verificação (por nome, tudo verde)

`ph2d-ecs` canal 6 · `ph2d-timeline` canal_field 7, autokey 33 (28 + 5 `autokey_campos`), clip_entry 8, clip_owner 6, doc_roundtrip 8, prop_readback 4, clip_stack_autokey 14, the_schema_is 1 · `ph2d-app-vec` canais_campos 6 (inclui `escrever_o_que_se_leu_e_identidade_em_todo_campo` sobre >60 campos), canais 3, canais_topologia 4 · shell `--bin` autokey 55 (inclui 5 `campos_vetor_tests`), undo 49, timeline_preview 8, preview_drive 12, opacidade_vetor 6 · `ph2d-editor-core` `the_shell_only_shrinks`.

RED-first provado desligando a cura: discreto substitui + Hold (2 vermelhos), regra do grupo (3), eco (2). **NÃO corridos:** clippy, `nextest-impacted`, mutação completa, `cargo check --workspace`.

## 6 — ABERTO, por ordem

1. **Linhas de GRUPO no dope-sheet.** Mapa do explorador: linhas = `TrackView` por binding (`crates/ph2d-timeline/src/snapshot.rs:40`, laço `:628`). Colapsar SÓ no painel, módulo novo em `crates/ph2d-panel-timeline` (`tracks.rs` está a 598/600, não crescer): `group_rows(snap, expanded) -> Vec<RowGroup{key,label,members,keys união}>`, agrupando bindings Field por (entidade, familia, grupo, `i` se `por_i`) via `FieldKey::mesmo_grupo`. `geom::row_bands` (`geom.rs:285`) itera grupos. Press na chave de grupo reutiliza `summary.rs` `select_column` (`:165`) restrito aos membros ⇒ os intents multi-chave existentes (MoveSelectedKeys, DeleteSelection, SetSelectedInterp, copy/paste) funcionam sem mudança. ▶ expande os membros; coluna de valor do grupo vazia; o graph editor mostra só alvos expandidos.
2. **O losango (F23 metade B, plano 34 §9, generalizado):** `channel_mark` (`crates/ph2d-timeline/src/channel_mark.rs`) por `PropKind` incluindo Field (valor Field via `CanaisLidos`); registo `key_mark` em `ph2d-editor-core` `widget/property_box/key_mark.rs`; tabela por painel campo-id → `FieldKey` (representante do grupo); clique → `AddKey` do grupo inteiro / remover; gate `architecture_every_editable_field_is_a_channel` (plano 35 §5.2). Os 4 passos da metade B do F23a §5.1 seguem válidos.
3. Smoke do dono depois de 1+2: fotografar `=3`/`=4` com `fotografa_cena.sh` para `docs/UI_New_and_Simple/vetor_nova_ui/fotos/F24_*.png`, números da `PH2D_AUTOKEY_LOG`, passos em PT simples.
4. **Lacunas de vocabulário no vetor:** CONTEÚDO do texto e FAMÍLIA da fonte (strings — pedem tabela de strings, Texto/Hold), FONTE do padrão (asset), parâmetros do Brush stroke; `StrokePaint` tipo índice 5 = Brush.
5. **Bloco 2 (plano 35 §10):** todas as outras famílias §A.2–A.9 + Painter quadro-a-quadro. Antes: colisão com a missão dos pincéis do Painter (`git worktree list`).
6. **Perf T0(b):** cena grande (100 paths × 100 vértices × 8 ≈ 80 mil bindings); critério de morte 2 ms ⇒ avaliação em lote por grupo. `binding_for` é linear; custo de `publicar` por quadro por medir.
7. **ADR** «o canal de qualquer propriedade» (`PropKind::Field` + `CanaisLidos`/`CanaisEscritos` + eco + `ConducaoVetor`): contar o próximo número em `docs/architecture/decisions/README.md`.
8. **Gate de fecho:** clippy `--all-targets`, `nextest-impacted`, `cargo check --workspace`, lista de mutação (plano 35 §6 + leitura devolve a curva · escrita no-op · topologia só na 1.ª chave · Hold desligado · cor por OKLCH · eco desligado · `settle` esquecido).
9. `CLAUDE.md` §5.1 UI/UX ao fechar a linha: trocar o link do último handoff e o intervalo do smoke (inclui `PH2D_AUTOKEY_SMOKE=1..4`). Não alterado nesta onda.
10. **Conhecido, não é defeito novo:** o undo da timeline da propagação é separado do undo global da edição do vetor (duas histórias, desenho existente); mudanças de topologia não alinháveis (simplify path) recebem mapeamento LCS de melhor esforço.
