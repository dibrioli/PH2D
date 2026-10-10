# Handoff de continuação — Luz 2D, W4 → W5 (2026-10-09)

- **Linha:** `line/Luz2D` · worktree `Worktrees/line-Luz2D` · merge-base `2a925859d` (sem rebase pendente) ·
  **HEAD:** o commit deste doc.
- **Commits da W4:** `9887a8665` a lâmpada de fábrica, menu Add, dreno, avisos, re-coleta · `baa32fbb9` a secção
  LUZ do Inspector · `352d72d6d` a composição na shell · `00d030546` as cenas 2 e 4, sonda, censo HR-15 ·
  `e4ce65561` o portão de fecho (1 lint + 6 vermelhos) e o G8 da shell · este doc.
- **Estado:** W4 FECHADA (onda, não a linha). Próxima: **W5 — o teto** (rebote, HRC, C3, governador, Flip,
  e os dois ABERTOS de baixo). **Janela nova.**
- **Pergunta de produto feita ao dono** (formas do Motion bloquearem luz): resposta **«Ainda não»** — fica
  como está (recebem, não bloqueiam).

## O que fechou

| entregável | onde | prova |
|---|---|---|
| `Luz::default()` = `I = 2π` (branco a 1 m, derivado do §2.3), `tamanho 0,25 m`, `∞`, Ponto; `Luz::ambiente()` = forno branco; `register_default` | `ph2d-light2d-ecs/src/componentes.rs` | `w4_artista`: branco a 1 m pelo gabarito (`asin(a)/a` = 1,0107), forno branco |
| Menu Add → **Light** / **Ambient Light** (`AddGroup::Game`) | `ph2d-app-light2d/src/object_add.rs` + 1 linha na lista e 1 braço em `fase_object_add.rs` | `w4_artista::o_menu_add…`; shell: o menu oferece as duas, criar+desfazer volta ao bit |
| Inspector **Light**: Type · Color · Intensity · Size · Limit Range/Range + avisos + «branco a d m»/«% do forno» | painel: `ids/inspector_luz.rs`, `populate_luz.rs`, `sections/luz.rs`, `paint_optional_luz.rs`, `event_luz.rs`, `sync_luz.rs`; vocabulário `ph2d-editor-core/src/luz_edits.rs` (`ComponentEdit::Luz`); família `ph2d-app-light2d/src/inspector.rs` (`build_info`, `apply_all`) | `seam_luz` (7, gesto real: clique, digitar, ARRASTAR, seletor); shell `luz_tests` (2: dreno → fila; fila → mundo); `editing_a_light_then_undoing…` (bit) |
| Paleta `+` → Light Occluder / Unlit | **já existia** no catálogo (conferido no código) | `a_paleta_da_imagem_oferece_e_anexa_os_dois_da_luz` (o pick anexa) |
| Gizmo | **já existia**: uma `Luz` sem sprite é um objeto VAZIO (`group_gizmo_view::is_empty_object`) → anel, clique (`pick_empty_at_world`) e arrasto (`ramo_gizmo_pick_arrasta`), igual à Câmera — medido no código, nada novo | as fotos mostram o anel na lâmpada |
| Recusa à UI: `avisos_de(e)` (solver recusou · ponto sem tamanho · menor que o texel · fora da vista · sprites pelo quad) e `recusa_nova()` (toast UMA vez, na transição) | `ph2d-app-light2d/src/quadro.rs`; `CenaDeLuz::luzes_emissoras` (qual emissor é cada Luz) | `w4_artista::os_avisos_chegam_a_ui` (GPU, a recusa forçada por uma janela 8× o limite) |
| **A edição chega no MESMO quadro**: `LuzDoQuadro::recolher` antes do solver (a shell aplica o Inspector DEPOIS da codificação) | `quadro.rs` + 1 linha em `present.rs` | `w4_artista::a_edicao_depois_da_codificacao…` — controlo: sem ela a lâmpada apagada ainda acendia (fluência 78,9); com ela 0; a última Luz apagada deixa o quadro NEUTRO, nunca preto |
| Cenas `PH2D_LIGHT2D_SMOKE` 1 · 2 · 4 (`max_level` 4, contado no `match`; a 3 responde «não há») | `smoke.rs` | `smoke_cena_1`, `smoke_cenas_w4` (gabarito ×2 + placa) + fotos |
| Censo HR-15 da família (3) | `tests/it/every_word_this_family…` | vermelho na 1.ª corrida (rótulo de depuração do wgpu, isento com o mecanismo) |

## Números medidos

**Rodada** (`medida_luz2d_w4`, perfil `smoke`, RTX 5060 Ti, `loadavg` 5,4; 15 rodadas intercaladas, MÍNIMO; placa
pelo relógio dela, CPU de parede), 1920×1080, vista de fábrica:

| cena | emissores | placa C0+C1+C2 | C0 | C1+C2 | CPU preparar+gravar |
|---|---|---|---|---|---|
| 1 Uma luz | 1 | 1,307 | 0,011 | 1,296 | 0,101 |
| 2, 1 vaga-lume | 1 | 1,662 | 0,011 | 1,650 | 0,079 |
| 2, 1 000 vaga-lumes | 1 000 | 1,988 | 0,014 | 1,974 | 0,590 |
| 4 Tamanho da lâmpada | 2 | 1,802 | 0,012 | 1,790 | 0,118 |

⇒ **a captura é plana** (0,011 → 0,014 ms com 1 000); o que sobe +20 % é a MARCHA (C1+C2), que depende da
GEOMETRIA (o campo de distâncias encolhe perto de mil formas) — o S3 do W3 «ABERTO» 6, não um custo por luz.
Pelo K3 (> 5 %) não é «plano»: é o caso do governador da W5.

**Cena 2** (gabarito): ao lado do enxame 0,431 · lado aberto 0,256 · atrás da árvore 0,116 · longe 0,196 ·
ambiente 0,018. **Cena 4** (penumbra 10–90 %, linha a 1,7 m do canto):

| lâmpada | geometria | gabarito | placa (texel 4 px) | placa (texel 2 px) |
|---|---|---|---|---|
| 0,15 m | 0,3283 | 0,3283 | 0,945 | 0,610 |
| 0,6 m | 1,4064 | 1,4064 | 1,708 | 1,572 |

**A lâmpada de fábrica na grade** (1920×1080, degrau padrão): texel 0,0625 m na vista de fábrica; possui texel
até o zoom ÷4 (0,25 m) e sai a ÷8 (0,5 m) → o Inspector avisa «menor que o texel».

**Delta da shell:** **+143** (185 568 → 185 711; tecto 189 041). Composição: menu Add (+1 família, +1 braço),
dreno (+1 braço), filas (+4), aplicação (+2), instantâneo + toast (+11), `recolher` (+1), 2 ficheiros de teste.

## Prova de mutação (agente `mutacao`)

Pré-voo: as 17 âncoras casaram 1× cada; filtros verdes e povoados (`seam_luz` 7 · `w4_artista` 9 · shell
`luz_tests` 2 · `fase_object_add` 5). Restauro por `checkout` + `touch`; árvore = HEAD no fim.

| # | mutação | resultado |
|---|---|---|
| M1 · M2 | chip do Tipo manda sempre 0 · sem o registo dos chips | **sangram** (`seam_luz`) |
| M3 | Intensidade ↔ Tamanho no despacho | **sangra** (digitar e ARRASTAR) |
| M4 | a cor do seletor com um canal trocado | **sangra** |
| M5 | a Ambiente pinta tamanho/alcance | **sangra** |
| M6 · M7 | cor sem sRGB → linear · aceita intensidade negativa | **sangram** (`cada_edicao…`) |
| M8 | `recolher` sem re-coletar | **sangra** (o mesmo quadro) |
| M9 · M10 | o aviso sobe a cada quadro · menor-que-o-texel lido como fora-da-vista | **sangram** |
| M11 · M12 | a Ambiente do menu nasce Ponto · `I = π` na de fábrica | **sangram** |
| M13 · M14 | o dreno da shell engole a edição · a aplicação esquece a fila | **sangram** (`luz_tests`) |
| M15 | o braço da Luz no menu Add da shell apagado | **sobreviveu** — precisa de `gfx` (ABERTO 5) |
| C1 · C2 | `!v.is_nan()` a mais · o valor de reserva da caixa do alcance | sobreviveram (equivalentes — certo) |

## Portão de fecho

1.ª corrida: 1 lint (`needless_range_loop`) + 6 vermelhos determinísticos (a11y da moldura nova, duas
literais sem marca, o teste de ordem da coleta sem o campo novo, a porta da Luz fora da varredura armar/
desarmar ×2, a tabela da luz fora do censo de palavras do painel) — curados em `e4ce65561`. 2.ª corrida:
`fmt --check` ✓ · clippy `-D warnings` (8 crates + shell) ✓ · `machete` ✓ · `CARGO_BUILD_WARNINGS=deny check
--workspace --all-targets` ✓ · `nextest-impacted` **16 887 / 16 887** ✓ (`loadavg` 11–17) · `the_shell_only_shrinks`,
`the_shell_links_exactly_the_registered_families`, `every_registered_family_declares_a_reachable_router`,
`architecture_interactive_crate_has_behavioral_test` ✓.

## Premissas do briefing que a medição derrubou

1. «criam o objeto pela porta `attach_by_name`» — ela é da `ph2d-app-components` e família não chama família
   (ADR-0075); a `Luz` não tem `requires`, então nascer com ela é o mesmo (o esqueleto faz igual).
2. «Gizmo da Luz: MEÇA antes» — medido: nada a construir; o objeto vazio já tem anel, clique e arrasto.
3. «Paleta `+` → Bloqueia luz / Não recebe luz» — já existia; o entregável foi o gate do pick.
4. «Arrastar Intensidade muda a luz no MESMO quadro» — NÃO mudava: a coleta corria antes da aplicação do
   Inspector (um quadro de atraso, medido). Curado por `recolher`.
5. «Inspector na crate da família» — o molde vivo (Vida/Abanão) põe ids/populate/paint/evento no
   `ph2d-panel-inspector` (o painel não vê o ECS, ADR-0029) e só o instantâneo + o dreno na família. Seguido.
6. «1 000 vaga-lumes voando» — PARADOS (voar pede um relógio por vaga-lume; o que a cena mede é a luz). E em
   dez enxames espalhados o chão acendia por igual (1,5–1,7: no plano a luz cai só com 1/r) → quatro enxames.
7. «a curva de custo tem de ser plana» — a captura é; a marcha não (+20 %, geometria).
8. «a pequena faz sombra nítida» — no gabarito sim (= geometria); na PLACA a pequena é 0,945 m contra 0,328
   (resolução ANGULAR das cascatas: o degrau fino só a leva a 0,610). E a 0,08 m a lâmpada manchava a sala.

## ⚠️ ABERTO (para a W5, por ordem de dano ao artista)

1. **Um objeto que BLOQUEIA a luz e a recebe sai PRETO.** Lei da W1 («dentro, a partida conta como uma
   entrada», `ph2d-light2d-ref::transmissao_ate`), igual na placa; e a saída antecipada da W2 («um intervalo
   que começa dentro de um opaco devolve 0», gate `g4_o_interior_de_um_opaco_nao_custa`) depende dela. Medido
   na cena 1: luz no interior da parede = 0,0000 (gabarito). As cenas usam `BloqueiaLuz + NaoRecebeLuz`
   (`smoke::PAREDE`). Cura candidata: um passe de PREENCHIMENTO depois da C2 (o interior recebe a fluência
   do ponto exterior mais próximo — um JFA do complemento) e a mesma regra no `luz_em` do gabarito.
2. **Lâmpada pequena: penumbra larga e manchas** (tabela acima) — o limiar da C3 analítica sai daqui.
3. **A lâmpada some ao afastar o zoom** (÷8 com a de fábrica) — avisada no Inspector; a C3 cura-a também.
4. **A cena 2 corre a ~17 fps no editor** (status bar da foto: 55–57 ms/quadro, 1 006 objetos). A luz mede
   ≤ 2,6 ms (placa + CPU); o resto é o editor com mil objetos — NÃO medi quem.
5. **O braço da Luz no menu Add** (`fase_object_add.rs`) não tem gate que o alcance (precisa de `gfx`) — a
   mutação dele sobrevive, como a de toda família.
6. Herdados da W3: Flip não recebe nem bloqueia; formas do Motion não bloqueiam (dono: «ainda não»);
   `NaoRecebeLuz` não vale para o vetor; K7 > 1080p (governador).

## ⛔ Recusas MEDIDAS (não reconstruir)

| tentativa | número | por quê |
|---|---|---|
| dez enxames espalhados (cena 2) | `luz_em` 1,5–1,7 em todo lado | nem poça nem sombra; 1/r soma tudo |
| enxames densos `k = 4`, raio 0,7 m | chão saturado branco | não se via vaga-lume |
| lâmpada pequena 0,08 m (cena 4) | penumbra na placa 0,77 m, manchas na sala | fraqueza das cascatas com fonte minúscula |
| cenas de 18 × 10 m | metade fora da tela (vista de fábrica ≈ 12 × 9 m) | a foto mostrou |
| paredes que recebem a luz | pretas | ABERTO 1 |

## Para a integração (somam entre linhas)

Foundational tocado (ADR-0107, append-only): **`ph2d-editor-core`** — `luz_edits.rs` (novo), `ComponentEdit::Luz`
(apendada), `INSP_LIVE_LUZ_SECTION/GRIP` + `LIVE_SECTIONS` 46 → **47** (contado), gates
`hr12_widgets_a11y` (+1 entrada). **`ph2d-panel-inspector`** — a secção (6 ficheiros novos), `any_live_section`
35 → **36**, `LiveSnapshots::luz_info`, `InspectorState::last_luz_sig`, a 6.ª tabela do censo de palavras.
**`ph2d-i18n`** — `luz2d.rs` (novo), `object_add.light.*`. **`ph2d-panel-registry-init`** (teste) — a porta
`set_current_inspector_luz` armada/desarmada. **`ph2d-app-components`** (teste) — o gate do pick da paleta.
`ph2d-light2d-ecs`: `Default for Luz`, `Luz::ambiente`, `register_default`, `CenaDeLuz::luzes_emissoras` (literais
de teste com `..Default::default()`). Shell: `fase_object_add.rs`, `fase_bus_drain_out.rs`,
`fase_bus_inspector.rs`, `fase_hero_commits.rs`, `fase_inspector_commits{,_top20}.rs`,
`fase_snapshots_publish.rs`, `present.rs`, 2 ficheiros de teste. `PROJECT_SCHEMA` intocado; nenhum contrato
congelado (`Tool=12`/`PanelEvent=4` intactos; o Inspector vai por `EditorAction::InspectorComponentEdit`).

## O que a W5 precisa saber

1. As portas da shell agora são QUATRO: `codificar` · `recolher` · `gravar` · `ligar` (+ `avisos_de`,
   `recusa_nova` para a UI). Um passe novo do solver entra entre `recolher` e `ligar`.
2. A UI da luz está toda ligada: um campo novo da `Luz` custa `LuzFieldEdit` + `aplicar` + linha da secção +
   id/populate/evento/semente — os gates `seam_luz` e `cada_edicao_muda_o_seu_campo` acusam a ponta que faltar.
3. A sonda `medida_luz2d_w4` e o `smoke_cenas_w4::penumbras_na_placa` são a régua do ABERTO 1–3: a C3 tem de
   levar a pequena (0,15 m) de 0,945 m a ≈ 0,33 m na placa, e o passe de preenchimento tem de pôr luz no
   interior da parede da cena 1 sem custo no `g4_o_interior…`.
4. Fotos da W4: `docs/Luz2D/handoffs/fotos_W4/cena{1,2,4}.png` (`fotografa_cena.sh`, 1920×1080).

## Smoke compilado (regra I, depois do `rm -rf target/*/incremental`)

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª saída:

```
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

## Smoke do dono (09/10) e as curas — «muita coisa não está boa»

Report: *«Resolução muito baixa. Luz sem gizmo, mover a lâmpada não atualiza a posição da luz, etc.»*

| report | mecanismo (medido) | cura · gate |
|---|---|---|
| mover a lâmpada não move a luz | no editor o `GlobalTransform` do MUNDO não é mantido (a propagação escreve-o só no `PresentWorld`, `sim_extract`); a coleta lia-o → ficava no valor de nascença, e a lâmpada do **menu Add nem acendia** (0 emissores: a cena ESCURECIA). Os gates da W4 punham o `GlobalTransform` à mão (a lição «onde os objetos NASCEM») | a coleta lê `ph2d_ecs::world_transform_into` (a porta do clique/gizmo) · `a_pose_e_a_do_editor` (3, vermelho-primeiro) · `3e366ba27` |
| luz sem gizmo | o anel do objeto vazio dizia ONDE, não que é luz, o tamanho, nem o alcance | `gizmo::{glifo, desenhar}`: lâmpada no tamanho real (piso 3 px), raios fora do anel, alcance tracejado na selecionada, a Ambiente com o seu símbolo · `o_gizmo_da_luz` · `c7c75a117` · foto `cena4_gizmo.png` |
| resolução muito baixa | a grade a 4 px (K7, orçamento de CELULAR) dava degraus na borda da sombra e uma faixa escura de 4 px em volta das paredes | dono escolheu: **Settings ▸ 2D Light** (Low · Medium · High = 4 · 3 · 2 px) **+ Automatic** (ligado; o nível é o TETO; desce após 3 leituras > 5,56 ms = 1/3 do quadro a 60 Hz, sobe se o nível acima, previsto por `1/célula²`, cabe em 75 %). Leitura dos carimbos SEM ESPERAR. Preferência do editor em `~/.ph2d/prefs.txt`; o Inspector diz «Quality: High (automatic), x ms» · `a_nitidez_da_luz` (3, um na placa), `settings_luz`, `every_light_quality_survives_a_round_trip` · `fe4373252` · foto `cena4_antes_depois_nitidez.png` (degraus somem; cena 1 a 60 fps em High) |

**Mutação das curas** (agente `mutacao` + re-prova): 12/12 sangram depois de três gates novos (`a6dcaa6bf`) —
G2 (raios começando dentro de uma lâmpada maior que o anel), Q2 (subir além de um teto Médio), Q4 (o
automático que leria UMA vez e travaria). Controlo (folga 4 → 5 px) sobreviveu — certo.

**Portão** (depois de `1c511c1d6`): fmt · clippy `-D warnings` (9 crates + shell) · machete · `check
--workspace` sem avisos ✓; `nextest-impacted` 16 893/16 896 — os 3 vermelhos eram do `chrome-sync` (um
ficheiro de TESTE dentro de `chrome/` conta como handler) e curados em `1c511c1d6` (re-corridos: 3/3 ✓).
Shell: 185 758 linhas (+190 sobre 185 568; tecto 189 041).

**Fundação nova desta volta** (ADR-0107, append): `ph2d-editor-core` — `NivelDaLuz`/`PreferenciaDaLuz`,
`ViewState::luz2d`, `ContextMenuKind::SettingsLightSubmenu`, `CTX_MENU_SETTINGS_LIGHT`/`_LIGHT_NIVEL[3]`/
`_LIGHT_AUTO`, a linha no Settings, a marca (parâmetro a mais em `id_is_currently_selected` e
`paint_context_menu_overlay`), `chrome/settings_luz.rs` (chain regenerada). `ph2d-i18n`: `chrome.menu.light_2d`
(«2D Light» — `chrome.menu.light` já era o TEMA) e os níveis. `ph2d-light2d-gpu`: `Tempos::ler_sem_esperar`,
`Solver::tempos_sem_esperar`. Shell: `prefs.rs` (`light_quality`, `light_auto`), `init.rs`,
`forwarding_persist.rs`, `fase_snapshots_publish.rs` (`preferir`), `fase_selection_highlight.rs` (gizmo).

**Ainda aberto, visível** (para a W5, nº 1): a **faixa/pontilhado escuro** na face iluminada das paredes
(agora 2 px em High) e as paredes PRETAS que recebem luz — a mesma regra «o interior de um opaco recebe
0»; o «etc» do dono ainda não foi listado.

Smoke recompilado depois das curas (regra I), 2.ª saída:

```
    Finished `smoke` profile [optimized] target(s) in 0.33s
```
