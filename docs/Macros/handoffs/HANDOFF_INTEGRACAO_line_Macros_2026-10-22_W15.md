# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W15 (o passo da macro leva a cabeça ao losango; uma chave só cruzada no arrasto sobrevive) → `main` (2026-10-22)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo agente integrador (DIRETRIZ §1.5.3).
> Mecanismo, medições e recusas: [doc W15](../13_W15_o_passo_e_as_chaves.md). Pedido:
> [PROMPT_W15](PROMPT_W15_o_passo_e_as_chaves_na_timeline_2026-10-22.md).
> ⚠️ **W8–W14 ainda NÃO estão no `main`** (mesma branch, integram juntas): os handoffs
> [W8](HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md) … [W14](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W14.md)
> continuam válidos para a parte deles. Este só acrescenta a W15.
> ⚠️ A linha **NÃO foi rebaseada** (ordem do dono: «sem reset nem rebase»): o integrador rebasa.

## §1 — Base e HEAD

- Branch `line/Macros-Automação`, worktree `Worktrees/line-Macros-Automação/`. Merge-base `ad5f8fe39c`.
- Commits da W15 (`git log --oneline 1c4dcba432..HEAD`, do mais velho): `8e13848b73` (fix: os dois defeitos + testes) ·
  `a0efa9c7ae` (test: o losango pintado debaixo da cabeça — fecha a M11; `=6 etapa`; fotos) · `a34b06d4ae` (refactor:
  `caminho_cef.rs` e `intent_seek.rs` — tecto de 700 LOC; doc W15) · o commit deste handoff (último).

## §2 — O que a W15 entrega

Relatos do dono, smoke de 22/10 (verbatim no prompt):

1. **O passo leva a cabeça EXACTAMENTE ao losango.** Antes: o painel MACROS mostrava e levava ao FIM da etapa (`t1`),
   arredondado ao quadro (`Scrub`: 1,63 → 1,625); o losango da linha «Macro steps» era pintado no INÍCIO (`t0`).
   Agora `EtapaDaMacroView::fim` (renomeado de `t0`) = `t1`; o losango pinta em `TimeView::x(fim)`; painel e losango
   pedem `TimelineIntent::ScrubExact(fim)` (variante nova, nunca arredondada). `TimelineIntent::is_seek`
   (`ph2d-timeline/src/intent_seek.rs`) é a porta única dos leitores da shell que tratam um salto
   (`timeline_bridge` `jumped`, `fase_timeline_drain` `scrub_now`).
2. **Uma chave só atravessada no arrasto sobrevive.** O arrasto emite um `MoveSelectedKeys` por quadro num bracket;
   `Track::move_keys` absorvia a chave parada no tempo exacto a cada quadro. Agora, com o bracket aberto:
   `Track::move_keys_unmerged` + `TimelineHistory::note_moved`; `absorb_where_released` (no `EndEdit`, e num
   `BeginEdit` que encontra um bracket nunca fechado) absorve UMA vez onde o gesto largou
   (`Track::absorb_under_moved`). Largar em cima de outra chave continua a absorvê-la. O movimento fora de bracket
   não muda.

### §2-bis — Prova

- Vermelho com a cura retirada (`move_keys` no bracket): `[1.5, 2.0]` ≠ `[1.0, 1.5, 2.0]` e `[0.5, 0.75]` ≠
  `[0.5, 0.75, 1.0]`.
- **Mutação:** 11; 9 sangraram à 1.ª. M11 (losango pintado −0,5 s) sobrevivia → gate novo (centro do losango a < 1 px
  da linha da cabeça, lida pelo valor do slider da régua com `clip_time = fim`), re-corrida sangra. M5 (`begin` sem
  limpar `moved`) equivalente → a linha saiu (o `absorb_where_released` esvazia antes de cada `BeginEdit`).
  Final: 10/10 sangram + 1 equivalente removida.
- **Testes novos:** `ph2d-timeline` it `intents`: `a_key_dragged_across_another_and_released_past_it_keeps_both`,
  `a_group_dragged_backwards_across_a_neighbour_keeps_it`, `a_key_released_on_another_still_absorbs_it`,
  `scrub_exact_lands_off_the_frame_grid_and_is_a_seek` · `ph2d-panel-timeline`
  `macro_steps::tests::clicar_num_losango_leva_a_cabeca_ao_instante_dele` · `ph2d-app-macros` it
  `gate7_clicar_numa_etapa_leva_a_cabeca_ao_losango_dela_sem_arredondar` · `caminho_tests` inclui `Variante::Etapa`.
  Alterados: `gate7_gravar_pincelar_parar…` espera `ScrubExact`; `gate7` arrastar/apagar leem `.fim`.

## §3 — Foundational / partilhado tocado (aditivo) — ⚠️ código da TIMELINE curado aqui (Modo L)

| Crate | O quê |
|---|---|
| `ph2d-anim` | `Track::{move_keys_unmerged, absorb_under_moved}` (+ `shift_keys` privado); `move_keys` igual por fora |
| `ph2d-timeline` | `TimelineIntent::ScrubExact` (variante NOVA) · `intent_seek.rs` (novo, `is_seek`) · `TimelineHistory` campo privado `moved` + `note_moved`/`take_moved` · `intent_apply` (`MoveSelectedKeys` no bracket, `BeginEdit`/`EndEdit`, `absorb_where_released`) · `EtapaDaMacroView.t0` → **`.fim`** (campo renomeado) |
| `ph2d-panel-timeline` | `macro_steps.rs` (pinta e clica no `fim`) |
| `ph2d-app-macros` | `etapas_na_linha` enche `fim: t1_s`; `VaiPara` → `ScrubExact` |
| `ph2d-app-painter` | `caminho.rs` (`Gesto::{Losango, Chaves}`, `Variante::Etapa`, `CHAVES_S`), `caminho_cef.rs` (novo: o roteiro do C&F movido) |
| `host-desktop` (shell) | `macro_smoke.rs` (dois braços), `timeline_bridge.rs` e `fase_timeline_drain.rs` usam `is_seek` (a shell encolhe) |

Contratos congelados (§6) **não tocados**. Sem `Cargo.toml`, crate nova, ADR ou `PROJECT_SCHEMA`.

## §4 — O que um merge pode partir

- ⛔ **`TimelineIntent::ScrubExact` é uma variante nova:** um `match` exaustivo noutra linha sobre `TimelineIntent`
  deixa de compilar (fácil de ver); pior, um `matches!(…, Scrub(_) | SeekFrame(_))` NOVO noutra linha compila e
  esquece o `ScrubExact` — varra `rg 'Scrub\(_\)'` na árvore combinada e troque por `is_seek()`.
- **`EtapaDaMacroView.t0` → `.fim`:** quem leia o campo noutra linha rompe na compilação (bom).
- `TimelineHistory` ganhou um campo privado (sem construção por literal fora da crate; `Default` cobre).
- Os itens da [W14 §4](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W14.md) continuam: os `Passo` 53–56 colidem com a
  `line/PainterWatercolor`, e a superfície Wet Paint.

## §5 — Contadores (DELTA sobre o merge-base, só a W15)

Crates novas 0 · chaves i18n 0 · ADR 0 · `PROJECT_SCHEMA` 0 · contratos 0 · cenas de smoke: nenhum número novo (`=6`
ganhou `PH2D_MACRO_CAMINHO=etapa`) · ficheiros novos 2 (`caminho_cef.rs`, `intent_seek.rs`).

## §6 — Prova de fecho

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| clippy `-D warnings --all-targets`: anim, timeline, panel-timeline, app-macros, app-painter, host-desktop | ✓ (timeline e app-painter re-corridos depois de `a34b06d4ae`) |
| `PH2D_SUITE=fecho` `nextest-impacted.sh` (BASE = merge-base, em `a0efa9c7ae`) | «Summary [ 955.462s] 19374 tests run: 19373 passed (32 slow), 1 failed, 10583 skipped», `load` 155 → 11 |
| a 1 falha: `architecture_workspace_file_loc_cap` (`caminho.rs` 710, `intent.rs` 710 > 700) | curada por MOVER em `a34b06d4ae` (656 / 698); re-corrido ✓, com `architecture_no_orphan_source_file` ✓, `caminho` ✓, `scrub_exact` ✓ |
| `the_shell_only_shrinks` · `file_loc_caps` · `architecture_claude_md_cabe_no_orcamento` | ✓ |
| `censos-da-arvore-combinada.sh` | ✓ 12/12, 121/121 |

⚠️ A suíte inteira não foi re-corrida depois de `a34b06d4ae` (só código movido + os portões dele) — o integrador corre a sua.

`agent-loop-profile.sh`:
```
  ✗ paralelismo de ferramenta              1.19/passo   alvo: >= 1,5  (11% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                138   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                308 : 126   alvo: <= 1,0  razao 2.4x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             0.5%   alvo: <= 2%  (4 de 804; o guarda R6 recusa-as desde 14/10)
  ✗ contexto relido por passo (media)         275 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               65 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

Smoke compilado (2.ª corrida, depois de `rm -rf target/*/incremental`, 15 G debug + 2,8 G smoke libertados):
«Finished `smoke` profile [optimized] target(s) in 0.33s», zero `Compiling`.

## §7 — Smoke do dono e fotos

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=6 PH2D_MACRO_CAMINHO=etapa ./target/smoke/ph2d-host-desktop`
2. Não mexa no rato ~7 s: a cena grava duas pinceladas, pára, põe três chaves na linha «Canvas · Opacity» (0,5 · 1 ·
   1,5 s) e clica no 2.º losango da linha «Macro steps».
3. A linha azul da cabeça passa EM CIMA do 2.º losango, e o «Time(s)» diz o mesmo número que a «2. Stroke 2» no painel
   MACROS. Clique depois na «1. Stroke 1» do painel: a cabeça salta para o 1.º losango.
4. Arraste o losango de 0,5 s da linha Opacity para a direita, por cima do de 1 s, e largue por volta de 1,25 s:
   ficam TRÊS losangos. Largue um em cima de outro: ficam dois (o arrastado ganha).
5. Errado: a cabeça ao lado do losango; um losango a desaparecer depois de só passar por cima de outro.

Fotos (`smoke_2026-10-22_W15/`): [a cabeça no 2.º losango](smoke_2026-10-22_W15/etapa_01_a_cabeca_no_losango.png) ·
[log](smoke_2026-10-22_W15/etapa.log) (`playhead=3.27 … steps=["1.65", "3.27"]`, `t=3.2711`). O arrasto das chaves
não se fotografa (a tela virtual ignora eventos sintéticos): a prova é o teste `intents` + o passo 4 do dono.

### §7-bis — Depois do handoff: o 2.º relato do dono (22/10)

«arrastar keyframes de modo que as animações dos traços se sobreponham apaga os keyframes … deve permitir
sobreposições». Mecanismo: o losango arrastado para trás do fim da etapa anterior gravava uma espera negativa maior que
o intervalo (`pi::espera` não valida); `efetiva` recusava-a e a Macro inteira deixava de se ler — painel e linha
3 → 0 etapas. `1b6bc725e1`: `ph2d-app-macros` `etapas::edita` efectiva a trilha antes de gravar e devolve o erro (a
Macro fica a de antes); gate `gate7_sobrepor_duas_pinceladas_nunca_apaga_as_etapas`, vermelho sem a guarda. Clippy
app-macros ✓, `gate7_` 9/9. A SOBREPOSIÇÃO em si (duas pinceladas a tocar ao mesmo tempo) pede o motor de pintura com
mais de um traço aberto: [PROMPT_W16](PROMPT_W16_pinceladas_sobrepostas_2026-10-22.md).

## §8 — ABERTO

0. **W16 — tocar duas pinceladas sobrepostas** ([prompt](PROMPT_W16_pinceladas_sobrepostas_2026-10-22.md)); até lá o
   arrasto que sobrepõe é recusado e o painel diz porquê.

1. **Dur(s) 6,208 com um clipe de 4,21 s** (captura do dono): não reproduz no `=6 etapa` (Dur = duração do clipe:
   4,207 · 4,311). `Dur` = `length_override` ou a última chave de QUALQUER faixa (`doc_extent.rs`); arrastar um
   losango estica o clipe pela espera. **Perguntar ao dono a sequência de gestos.**
2. **O veredito do smoke da W14** (a água a correr depois do fim do clipe) **não foi dado** — perguntar.
3. Para o integrador: a W14 §4 (os `Passo` 53–56 e a superfície Wet Paint) e o §4 acima.
4. Seguem de pé os §8 da [W14](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W14.md), [W13](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W13.md)
   e [W12](HANDOFF_INTEGRACAO_line_Macros_2026-10-20_W12.md).

## Para o integrador (`CLAUDE.md` §5.1, entrada «Macros & Automação»)

Novo «Último» = este handoff (W8–W14 continuam a valer para a parte deles). A entrada da Timeline não muda de natureza.

## Retomar

`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`.
