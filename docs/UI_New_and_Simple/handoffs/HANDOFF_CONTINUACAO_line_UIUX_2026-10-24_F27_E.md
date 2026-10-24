# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 24/10 — F27 Bloco E: fonte do HUD, ficheiro do script e props do script são canais

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `f78ced22f8`. **Sem integrar.** Anterior:
  [F27 D](HANDOFF_CONTINUACAO_line_UIUX_2026-10-24_F27_D.md); integração a fechar: [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md).
- Desenho e medição: [`../F27_E_plano.md`](../F27_E_plano.md). **Não está copiado aqui.**
- Decisão do dono (23/10), mantida: `ph2d-panel-painter-layers` fica FORA.
- Estado: construído e verificado por nome; **smoke do dono (§4) por fazer**. Próxima LLM: janela nova.

## 1 — Commits (`git log --oneline f78ced22f8..HEAD`)

| commit | o quê |
|---|---|
| `57b9829f87` | a fonte do rótulo do HUD, o ficheiro do script e as props que o script declara são canais |
| `b0655d06b1` | teste: o «Source Name» pelo script vale para toda variante (a mutação do `any_variant` sobrevivia) |
| `a912ae2eb3` | cena `PH2D_AUTOKEY_SMOKE=32` — o Auto Key grava a camada, a fonte do HUD e o script |
| `cfda0a0574` | fecho de onda: fmt, `track_label` só para os testes, revinculação do teste do funil (clippy) |
| `339a856d20` | `script_controlo.rs` sai de `script.rs` (tecto de 600 LOC dos painéis: 635 → 433 + 212) |
| (este) | plano `F27_E_plano.md`, adendo ao ADR-0191, este handoff |

## 2 — Como foi construído

### 2.1 Desenho

Ver [`F27_E_plano.md`](../F27_E_plano.md): (a) `LabelSource` em dois canais com a chave `top:*`; (b) ficheiro do script = canal
discreto pelo `ScriptId` (`Target::Script`); (c) família `SCRIPT = 7` para as props declaradas; `TimelineDoc` v23; tabela
«⛔ Recusas MEDIDAS». Cura no mesmo lote: o canal genérico comparava opções com `==` (NavAgent «Target», Factory «Where»).

### 2.2 A cena `PH2D_AUTOKEY_SMOKE=32`

`crates/ph2d-app-components/src/autokey_smoke_componentes.rs`, despachada na shell por `fase_object_mode.rs` (um `.or_else`);
o app-vec reserva "32" como `COMPONENTES = 30` devolvendo `None`. A cena aumenta o dock inferior em 110 px
(`set_dock_bottom_h`) para as 5 linhas caberem. Log medido (verbatim):

```
[autokey-smoke] q18 Red layer Some(LayerId(5)) (Front = LayerId(5)): OK
[autokey-smoke] q18 Score source Some(TagCount("Enemy")): OK
[autokey-smoke] q18 Spinner script Some(Library(ScriptId(2))) (bob = ScriptId(2)): OK
[autokey-smoke] q18 Spinner speed Some(3.5) (perto de 3.5): OK
[autokey-smoke] 5 faixas (tem de ser 5)
```

Foto lida (1930×1040): quadrado vermelho sobre o azul no quadro 18; linhas da timeline «Spinner · Script…» 2.00,
«Spinner · speed» 3.50, «Red · Sorting La…» 5.00, «Score · Source» 3.00, «Score · Source…» Enemy, chaves a 0,5 s e 1,0 s;
secção Script do Inspector com o losango ao lado de «speed». **A foto não cobre os cliques do §4 (passos 4–7).**

## 3 — Superfície de colisão (um merge pode partir)

- **`ph2d-ecs`:** `scene/property.rs` + `property_ref.rs` — **partilhados com a lei de propriedades do script do CodeEditor**
  (`stored_from` ganhou um parâmetro: quem a chama noutra linha parte); `canal.rs` família 7 (append); `canal_script.rs` novo.
- **`ph2d-component-desc`:** `catalog/hud.rs`, `catalog/script.rs`, `field.rs` (`Target::Script`).
- **`ph2d-timeline`:** `DOC_VERSION` 22 → 23 (outra linha que também suba: recontar).
- **Shell:** `fase_snapshots_tardios.rs` e `fase_object_mode.rs` (1 linha cada).
- **UI:** `key_mark.rs`, `marcas_inspector.rs` (devolve `Cow`), `script.rs` partido em `script_controlo.rs`, `snapshot.rs` (699/700 linhas).
- **Scripts (CodeEditor vivo):** `ph2d.get(e,"label.source")` devolve "counter"/"tag"… (antes JSON cru) e há `label.source_name`.
- **Gate** `architecture_every_editable_field_is_a_channel`: `DIVIDA_FICHEIRO` fica VAZIO; entradas `hud_corpo` (Canal) e
  `script_controlo.rs::controlo` (Canal).
- **Delta de contadores:** cenas de smoke +1 (`=32`); `DOC_VERSION` +1; família de canais +1 (`SCRIPT = 7`); ADR: só adendo (0191).
  **Número de cena 32 e do ADR provisórios — o integrador reconta.** `PROJECT_SCHEMA` inalterado.

## 4 — Smoke do dono (em linguagem simples)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=32 ./target/smoke/ph2d-host-desktop`
2. Abrem quatro quadrados: vermelho (Red) em parte por cima do azul (Blue), amarelo (Score, o placar) e verde (Spinner, com
   um script). A linha do tempo, em baixo, mostra 5 linhas, parada no quadro 18.
3. Na linha do tempo: «Spinner · Script…», «Spinner · speed», «Red · Sorting La…», «Score · Source», «Score · Source Name».
   Cada uma tem dois losangos (quadros 12 e 24). Arraste o cursor de tempo entre 0 e 1 s: antes do 12 o vermelho fica por baixo
   do azul; entre 12 e 24, por cima.
4. No Inspector (à direita), com o Spinner: secção Script «bob.luau» e a linha «speed» 3.5 com um losango. Clique no losango
   de «speed»: vira chave (cheio) naquele quadro.
5. Clique no quadrado amarelo (Score) na Hierarquia: a secção HUD mostra Source «Tag» e Source Name «Enemy», cada um com o seu losango.
6. Clique no Red: a secção Ordering mostra Sorting Layer «Front» (a cena criou esta camada).
7. Carregue em Play: o verde segue o script do momento (spin.luau antes do quadro 12, bob.luau entre 12 e 24).

Deu errado se: falta uma linha na linha do tempo; falta o losango ao lado de «speed», «Source» ou «Source Name»; o vermelho
nunca passa para cima; no Play o verde para de se mexer.

Sabido (diga-o claro): na coluna do valor da linha do tempo, as escolhas aparecem como número (Source 3, Sorting Layer 5,
Script 2) e não pelo nome — é o próximo item. Na secção Script há dois losangos (o campo de texto e a lista são a mesma coisa).
O script reinicia do zero no instante em que a animação troca de script (perde a memória dele); os valores postos no Inspector ficam.

## 5 — Verificação

- Clippy `-D warnings` limpo nas 12 crates tocadas.
- Testes por nome verdes: `ph2d-ecs` `canal_componente::tests` (12) + `canal_script::tests` (2); `ph2d-timeline` `canal_textos` (7) ·
  `doc_roundtrip` (8) · `canal_field` (7) · teste de schema; `ph2d-app-components` `canais_script::tests` (3) ·
  `property_names_tests` (6, com `o_nome_da_fonte_do_rotulo_vale_para_toda_variante`) · `marcas_inspector` (9) ·
  `canais_componente::tests` (5) · `script_inspector::tests` (17) · `hud_inspector::tests` (12); `ph2d-panel-inspector`
  `a_seccao_script_esta_viva` (15); `ph2d-panel-registry-init` `o_inspector_armado` (3); `ph2d-panel-timeline` tracks/linhas;
  `ph2d-component-desc` crate inteira após a cura; `ph2d-editor-core` gate de canais (6), catraca da shell, tecto de LOC dos painéis.
- **Suíte (`PH2D_SUITE=onda nextest-impacted`, 2139,6 s):** 19 324 corridos, 19 320 passaram, 1 falhou =
  `architecture_panel_loc_cap::panel_files_under_loc_cap` (`script.rs` 635 LOC, minha — curada pelo corte, re-corrida verde);
  3 TIMEOUT que não são desta linha: Painter `seam_estado_escondido::cada_interruptor_que_neutraliza_pinta_como_o_pincel_limpo`
  (briefing: não perseguir) e Luz2D GPU `g4_custo_independe_das_luzes::{g4_o_custo_do_rc_nao_depende_do_numero_de_luzes, g4_o_interior_de_um_opaco_nao_custa}`.
- **Mutação, 8/8 vermelhas:** `ler_valor` com `o == v` · `partilhado=false` · Script fora das faixas · chave (0,0) ·
  laço da `retabela` removido · guarda do `script_existe` desligada · valores só-own · `any_variant` removido do `slot_in`
  (sobreviveu a princípio ao `property_names_tests` ⇒ teste novo `o_nome_da_fonte_do_rotulo_vale_para_toda_variante`, depois vermelho).
- Curado de passagem: `key_mark_tests.rs` (F27 B2) usava chaves inventadas `component.field.a.0/b.1` que o gate de strings do catálogo recusava.
- Binário do smoke: 2.ª compilação `Finished smoke profile [optimized] target(s) in 0.29s`.

## 6 — ABERTO, por ordem

1. Smoke do dono `=32` (§4).
2. A coluna do valor mostra a escolha como NÚMERO (Source 3.00, Sorting Layer 5.00, Script 2.00): é da família (todo canal
   Enum/Named). Cura = o publicador de valores (TrackValues, shell/timeline-persist) dar o rótulo da opção / nome da camada /
   nome do script — medir onde o canal de texto já resolve «Enemy».
3. Dois losangos na secção Script (campo de texto + seletor = o mesmo canal): decidir se a coluna do campo de texto fica vazia.
4. `CLAUDE.md` §5 UI/UX diz `PH2D_AUTOKEY_SMOKE=1..26` (desatualizado: agora 1..32) — corrigir ao fechar a linha.
5. **FORA** (igual): itens de LISTA, `ph2d-panel-painter-layers`, gravidade, física a meio, IkBendSide, Rope Length, áudio do mixer;
   «Emit signals» NaoAnimavel.
6. Próxima cena: `PH2D_AUTOKEY_SMOKE=33`.
7. **NÃO integrar nem fazer push sem ordem do Enio.** Na integração recontar: número do ADR, número da cena, versão do `TimelineDoc`.
