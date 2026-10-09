# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-08 (4.ª volta) — F7: O QUICKSHAPE

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), só por ordem do Enio, e a próxima
> LLM desta linha. A linha FECHOU e PARA: nada integrado, nada enviado.
> Tarefa: [`vetor_nova_ui/06_prompt_F7_quickshape.md`](../vetor_nova_ui/06_prompt_F7_quickshape.md);
> plano: [`03_plano.md`](../vetor_nova_ui/03_plano.md) §4 F7. F0–F6: os handoffs de
> [07/10](HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md),
> [08/10 F4–F5](HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F4_F5_ALCANCE_E_TOQUE.md) e
> [08/10 F6](HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F6_NAVEGAR_COM_OS_DEDOS.md) (citados, não reescritos).

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **base (merge-base = `main`)** `2a925859d`
  · F0–F6 (até `c10eb1a18`) continuam por integrar e vão JUNTOS com esta jornada.
- Commits desta jornada: `c10eb1a18..HEAD` (o `main` não andou — sem rebase).

## §0 — Para o `CLAUDE.md` §5.1 (≤ 700 bytes) — JÁ APLICADO nesta linha

UI/UX: «Último:» → este ficheiro; smoke `…|11` → `…|11|12`; a frase ganha «QuickShape (segurar = forma)».

## §1 — O que a linha entregou

| peça | o que é | onde mora |
|---|---|---|
| ⭐ **a lei** | `recognise(samples, px_to_world, regular) -> Option<QuickShape>` (`Line` · `Polygon{corners, closed}` · `Ellipse{centre, rx, ry, rot}`): reamostra por comprimento (256), RDP a 4 % da diagonal, funde viragens < 25° (com SENTIDO: um bico não é uma recta), absorve lados < 94 px numa QUINA-FAIXA (`Zone`: o chanfro, a curva de uma quina arredondada, o gancho na ponta — os lados correm ENTRE faixas, e uma faixa maior que o mínimo não é quina), flecha ≤ 5,5 % = recto ⇒ polígono; senão, fechado e com UMA volta de viragem, a elipse dos **momentos de área** da região (exacta para uma elipse perfeita, sem solver). Rectângulo = quinas 90 ± 8°, endireitado aos eixos até 5°; `regular` = quadrado, equilátero, N-gono regular, círculo, linha a 15° | `ph2d-vec-edit/src/quickshape.rs` |
| **o gatilho** | `Pencil::hold_tick(scene, now, regular, enabled)`: parado `HOLD_NS` (= `LONG_PRESS_NS`, gate) ⇒ o caminho VIVO passa a ser a forma (o traço cru some); o arrasto sem soltar ESCALA e RODA à volta do `pivot` (início da linha/polilinha, centro do fechado), com `regular` a rotação anda a 15° e o `Shift` a meio regulariza AO VIVO; soltar comete a forma (nós de quina; a elipse com as alças canónicas); `quick_dismiss` (o `Esc`) devolve o traço livre e o gesto não volta a reconhecer; a forma tem a largura do estilo (`width_stops` uniforme) | `quickshape_hold.rs`, `pencil.rs` |
| **a ponte** | `quickshape_frame::frame` (relógio dos carimbos = `timestamp_ns`, `Shift`, o interruptor) chamada no quadro ANTES do perfil de largura; nasceu ⇒ `UiSound::Click`. `quickshape_frame::escape` | `ph2d-app-vec/src/quickshape_frame.rs`; shell `fase_vector_live_geometry.rs`, `keyboard_escapes.rs` |
| **a porta visível** | interruptor **QuickShape** na fila do Lápis (o par `_ON/_OFF` troca como o da Simetria; ligado por omissão) pelas 7 pontas; a dica ⑤ «Hold at the end: perfect shape» SÓ com ele ligado (`hint_key_for`) | ids `vector_pencil.rs`; `populate_pencil`, `bars_row`, `event_clicks`, `tool_panel_event`; `draw_config`/`ui_snapshot.pencil_quickshape`; i18n |
| ⛔ **ACHADO e curado: a dica do Lápis mentia** | «Right-click or Esc: cancel» — o `Esc` não fazia nada durante o traço (só o botão direito). Agora: com forma, devolve o traço livre; sem forma, desiste | `keyboard_escapes.rs` |
| **cena 12** | layout Vetor, o Lápis pelo botão da coluna ①, três traços SINTÉTICOS (rectângulo tosco, círculo tosco, linha trémula) carimbados com o `clock_ns` (= o relógio do ponteiro); a SHELL mede o segurar e cada um vira a forma; acaba com o Lápis na mão e as três seleccionadas | `ph2d-app-vec/src/vector_mode_quick.rs` (estágios 50–54) |

## §2 — Medido e decidido (técnico, delegado)

- **O oráculo correu outra vez** (2.ª leva, 22 traços, `gera_entradas.py` acrescentado no FIM, as 43
  antigas byte a byte iguais): paralelogramos ⇒ a fronteira do rectângulo é **(6°, 10°)**; rodados 2–8°
  ⇒ o endireitar é **(4°, 6°)**; mão real (cantos arredondados, começar a meio, passar do início) ⇒ o
  Xournal++ recusa TUDO. 65 fixtures, 39 reconhecidas; repetibilidade e as duas lições do instrumento em
  [`00_resultados.md`](../vetor_nova_ui/oraculo_quickshape/00_resultados.md) §3b–§3c, §5.
- **Protótipo em Python primeiro** (a lei afinada contra as fixtures em minutos), depois Rust: o gate
  passou à primeira em Rust. **Cada constante foi varrida sozinha** sobre as 65 + 12 traços de mão; a faixa
  válida vai no doc-comment (ex.: fecho `0,05..0,07`, recta `0,04..0,075`, círculo `1,05..1,50`).
- **Onde o produto do dono manda mais que o oráculo** (memória `feedback_the_owners_reference_product_decides_the_law`):
  5, 6, 8 lados ⇒ polígono (o Xournal++ faz círculo); elipses; polilinhas abertas (o *polyline* do
  Procreate); quadriláteros como desenhados; os rectângulos de mão. **22 excepções**, cada uma com a
  geometria do gerador como verdade e o motivo no nome.
- **O centro do círculo NÃO é o do oráculo** quando o traço tem buraco ou sobreposição: o dele é o
  centróide (foge 3–5 un. do traço); o nosso são os momentos de área — o gate prova que o nosso desenha o
  traço melhor que o dele.
- **`MIN_SIZE_PX = 94`**: o mínimo do oráculo (40 un.) à escala do corpus; faixa medida `(86, 113]` px.
  ⚠️ Nenhuma MÃO real o mediu no nosso app — se o dono sentir que formas pequenas não reconhecem, é este
  número (§8).
- **Sem setas nem arcos** (FORA no prompt); um arco segurado fica traço livre.

## §3 — Foundational tocado e contratos

- **`ph2d-vec-edit`** (crate do motor novo, não congelada): módulos `quickshape`, `quickshape_hold`
  (novos, `pub`); `Pencil` ganha `hold_tick`, `is_quick`, `quick_dismiss`; `on_drag`/`rebuild`/
  `width_stops`/`on_release`/`cancel` sabem da forma; `rebuild` passa a escrever `closed`.
- **`ph2d-tool-vector`**: `VectorDrawConfig.pencil_quickshape`, `VectorStyleSnapshot.pencil_quickshape`
  (campos NOVOS em structs públicos com literais — quem construa um destes por literal noutra linha
  parte; dentro do workspace só os `..Default::default()`); ids `VECTOR_PENCIL_QUICKSHAPE_ON/_OFF`.
- **`ph2d-panel-vector`**: `bars::hint_key_for` (novo); `hint_key` intocado.
- **`ph2d-i18n`**: `panel.vector.pencil.quickshape`, `panel.vector.hint.pencil_quickshape` (+2 chaves;
  `vector.rs` a 650 de 700).
- **Contratos congelados (§6): nenhum.** `Tool=12`/`PanelEvent=4` intocados (o segurar coube no
  `Pencil` + o quadro, como o prompt mediu).

## §4 — Superfície de colisão e contadores (DELTA sobre `2a925859d`, somando F0–F6)

- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+12**. `PROJECT_SCHEMA`, `MODULE_TRUTHS`, `OWN_ROWS`, `layout.txt`:
  inalterados nesta jornada.
- **Shell (`numstat`)**: esta jornada `+21 −0` (o quadro e o `Esc`); somado a F0–F6 sobre a base: `src`
  net **−1**. ⚠️ `the_shell_only_shrinks` mede a árvore (ver §5).
- **Em silêncio**: outra linha que construa `VectorDrawConfig`/`VectorStyleSnapshot` por literal sem
  `..`; outra linha que mexa no `Esc` de `keyboard_escapes.rs` (o braço do lápis vem ANTES do da caneta);
  quem leia `panel.vector.hint.pencil` esperando a dica do Lápis com o QuickShape ligado (é a
  `_quickshape`).

## §5 — Gate final

Uma corrida do `verificador` sobre o diff acumulado (`BASE=2a925859d`), outras linhas a compilar ao lado
(`loadavg` ~40–48):
- fmt ✓ (um `cargo fmt --all` antes, commit `79bb1bef7`) · warnings-deny do workspace com todos os alvos ✓
  · clippy como o `ship.sh` (`--features ph2d-spike/bevy_ecs -- -D warnings`) ✓ · `machete` ✓ ·
  standalone-optional ✓ · workflow-packages ✓ · censos da árvore combinada ✓ (12/12) · `doc-index --check` ✓.
- `nextest-impacted` (`NO_FAIL_FAST=1`): **16 991 / 16 992**, 9 766 pulados. O único ✗ foi determinístico e
  DELIBERADO: `the_highlight_has_one_source::only_the_listed_gestures_arm_a_sound` — o quadro arma 6 sons
  e a lista dizia 5; o 6.º é o clique do QuickShape ⇒ a lista ganhou-o com o que confirma (`d30772992`),
  re-corrido verde.
- Catracas: `the_shell_only_shrinks` ✓ · `workspace_src_files_under_loc_cap` ✓ (⚠️ o `quickshape_tests.rs`
  chegou a 705 antes do gate — o irmão `*_tests.rs` NÃO é isento; partido) · `file_loc_caps` ✓ ·
  `architecture_claude_md_cabe_no_orcamento` ✓ (a entrada UI/UX tem 699 de 700 bytes — a próxima linha
  que lhe acrescente uma palavra corta outra).

### Mutação

Agente `mutacao`, verde visto antes de cada uma, âncoras pré-voadas (1×), restauro por `git checkout` +
`touch`, árvore limpa no fim. **1.ª rodada: 36 de 42 sangraram** — a lei (tamanho, fecho, fusão,
flecha, fusão do 1.º/último lado, rectângulo, ângulo recto, eixos, sentido do regular, viragem total,
resíduo da elipse, círculo, momentos), o gatilho (relógio, interruptor, `Shift`, degrau de 15°, arrasto,
largura, `closed`, `Esc`), a ponte, o par do painel, a dica, e as duas chamadas da shell. Das 6
sobreviventes, **1 equivalente** (`tried_at`: só poupa trabalho) e **5 buracos reais** — os testes que
os fecharam **apanharam dois defeitos da lei** (rectângulo chanfrado ⇒ `None`; gancho inclinava a linha
0,6°), curados pela QUINA-FAIXA (`3ee2dfc0d`). **2.ª rodada, sobre o código novo: 8 de 9**; a
sobrevivente (a união do lado partido pelo início perder a 1.ª metade) ganhou o teste
`a_side_split_by_the_start_is_fitted_from_both_halves`, visto vermelho sob ela e verde limpo.
⚠️ Depois da cura: `nextest` das quatro crates tocadas verde, clippy `-D warnings` delas verde, a
foto da cena 12 refeita (igual à vista; a linha mexeu 0,002 un.). O `nextest-impacted` inteiro NÃO
foi re-corrido depois de `3ee2dfc0d` (só `ph2d-vec-edit` mudou de lei; os consumidores dela
correram).

## §6 — Smoke do dono

Ver o relatório ao dono (passos numerados). Foto:
[`f7_01`](../vetor_nova_ui/fotos/f7_01_cena12_tres_tracos_seguros_viram_formas.png).

## §7 — O que fica ABERTO

- **Uma mão de verdade**: as leis estão provadas por 65 medições e a cena 12 por foto, mas o tamanho
  mínimo (94 px) e a folga do segurar (0,5 s, o do toque) nunca foram sentidos por uma mão no nosso app.
- **Arcos e setas** (o Procreate faz arcos) — fora por decisão do prompt; pergunta ao dono.
- **«Edit Shape»** do Procreate (os nós já são editáveis no Node; não há botão próprio) — não pedido.
- Do F6: quick-pinch/enquadrar tudo, um dedo real, customizar a barra (§8 do handoff de F6).

## §8 — Binário de smoke (2.ª corrida)

(BINARIO)

## §9 — `agent-loop-profile.sh`

(PERFIL)
