# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, 8b parte 2, 8c, cena 9 e mutação (23/10/2026)

> Leitor: a próxima LLM da linha (a do `/pd-linha-fechar`). **Não integrado, não enviado.** Antecessor:
> [d5 e 8b assets](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-23_D5_E_8B_ASSETS.md). Plano: [07](../07_arrastar_para_o_codigo.md) §3.4, §3.10.

## 1. Identidade

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · `line/CodeEditor` · merge-base `972f411a6f`.
Commits novos: `142e0cb029` (8b-2) · `2ca88f48fd` (8c) · `53224c86ba` (cena 9) · `1ed7ceba82` (testes da mutação).

## 2. Feito

- **8b-2 — linha do Inspector → código.** `ph2d-editor-core widget::arrasto` (o Inspector grava em `Gravacao` cada
  rótulo pintado + o cartão de `paint_section_header` + se é propriedade de script); `DragPayload::Property { script }`
  (0 = componente; `ScriptId`), `Dropping::Property { names, script }`, o nome viaja no rótulo do arrasto
  (`WidgetStore::end_named_drag`). `ph2d-app-components property_drag::carga`: texto → nome pelo MESMO catálogo
  (`tr(label_key)` → `code.name`), desempate pelo título do cartão, nunca à sorte; pose «Position X / Y» → `"x", "y"`.
  `dropping::property_insertion`: `"health.max"` no cursor (Ctrl igual, Godot); `self.<nome>` no texto do próprio script;
  noutro script não cai (`drop_doc::welcomes`). Fora do texto: `DropAction::Cancel`.
- **8c — Alt.** `ScriptValue::Object(nome)` no FIM; no topo `ph2d.object("Porta")` = objecto a DECLARAR (só com
  `DeclSink`); `self.porta` = a entidade (nil até existir, sem cache). Alt = `ph2d.property("porta", ph2d.object("Porta"))`
  no início da linha no topo, por cima da função se dentro (`Syntax::top_statement_line`); balão com 3 linhas. Inspector:
  o campo do nome (`SetObject`), `script_object_slot_pick` → `DropTarget::ScriptObject` → `DropAction::SetScriptObject` →
  `object_drag::into_script_slot`. Renomear reescreve os valores (`LuauScript::rename_object`) no mesmo diff.
  **Decisão técnica:** guarda o NOME (precedente `View::Named(Target::Name)`), não o `StableId` do plano 07 §3.4.
- **Cena `PH2D_CODE_SMOKE=9`** (`code_smoke_inspector.rs`, `CENAS=9` nos dois roteadores), portão
  `a_cena_9_do_editor_ensina_o_que_diz`, corrida conduzida + 14 fotos em `ferramentas/conduz_inspector/corre.sh`.
  Medido: com o script partido o cartão Script não mostra propriedades (o script começa válido); o editor fecha o `(`
  sozinho (larga-se ANTES do `)`).
- **Mutação:** 79 mutações nas leis do 8d/8b/8c, 53 sangraram; os 26 sobreviventes ganharam asserção (`1ed7ceba82`)
  menos 2 equivalentes (b5: `position_m`/`position_px` têm o mesmo texto; b8: não há par com mesmo cartão e rótulo).
  Harness fora do repo (scratchpad da sessão).

## 3. O que um MERGE pode partir

`CENAS` 8 → 9; CLAUDE.md §5.1 → `PH2D_CODE_SMOKE=1..9`. Foundational append-only: `DragPayload::Property`,
`Dropping::Property`, `InspectorScriptValue::Object`, `ScriptFieldEdit::SetObject`, `widget::arrasto`, o gancho em
`paint_property_label` e `paint_section_header`. `ScriptValue::Object` no fio (postcard, no fim). Shell: +~20 linhas em
`asset_drag_wire.rs` (catraca verde).

## 4. ABERTO

1. Smoke do dono da cena 9 (passos na resposta ao dono, 23/10).
2. `/pd-linha-fechar`: gate batched inteiro, auditoria, handoff de integração, §5.1 com `1..9`.
3. **Não curado:** `scripts/cargo-test-narrow.sh` dá «0 falharam · 0 passaram» com exit 1 quando o alvo de teste não
   compila (contrato: exit 2) — mordeu outra vez nesta sessão.
4. Um `ph2d.object("X")` no topo FORA de `ph2d.property` devolve a tabela-marca em silêncio (só falha ao usar);
   candidato a lint no editor.
