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

## Smoke do dono (09/10, 2.ª volta) — «mesmo com alta qualidade, a sombra não fica lisa/reta»

Sonda `medida_luz2d_w4::medida_borda_reta` (ignorada): a borda de CIMA da sombra da cena 1, o ponto de
50 % de luz a cada 10 cm ao longo dela, reta de mínimos quadrados, ondulação. 1920×1080, vista de fábrica,
`smoke`, `loadavg` 9,8, placa pelo relógio dela:

| variante | célula na tela | ondulação rms | máx | placa |
|---|---|---|---|---|
| gabarito | — | 0,00 px | 0,00 px | — |
| Low (4 px) · correção bilinear | 6,75 px | 4,08 | 9,49 | 1,32 ms |
| Low · fusão clássica | 6,75 | 13,94 | 36,55 | 0,44 |
| Medium (2 px) · bilinear | 3,38 | 1,19 | 2,91 | 5,40 |
| High (1 px) · bilinear | 1,69 | 0,77 | 1,58 | 21,06 |

(Intervalo da c0 ½ e 2 medidos com 4/3/2 px: pioram — 1 continua o melhor.)

**Dois achados:**
1. **Bug meu, curado:** com níveis 4 · 3 · 2 px a Média e a Alta caíam na MESMA célula (a grade arredonda
   para a potência de 2 acima — G11); o «High» do dono era o «Medium». Níveis agora por OITAVA (4 · 2 · 1 px):
   cada um tem metade da célula e 4× o custo, em qualquer zoom — gate
   `cada_nivel_tem_metade_da_celula_do_anterior_em_todo_zoom` (vermelho com 3 px: `[0,0156, 0,0156, 0,0039]`
   a 2,85 m).
2. **O limite do método:** afinar a grade reduz a ondulação devagar (×0,65 por 4× o custo); as cascatas
   NUNCA dão a borda reta a custo aceitável — é a resolução das sondas das cascatas de longe, que a correção
   bilinear só atenua. ⇒ **W5, nº 1 (antes do rebote):** a luz DIRETA das `Luz` por um passe próprio (o C3
   do plano, agora para TODA lâmpada, não só as minúsculas): visibilidade do disco por raios para a lâmpada
   através da cena capturada (a mesma marcha da C2), as cascatas ficando com o céu, os sprites que brilham e
   o rebote. Alvo medido nesta sonda: **ondulação ≤ 0,2 px rms** e penumbra = geometria (`smoke_cenas_w4`)
   em QUALQUER nível, com o custo por lâmpada na rodada. Cura junto o ABERTO 2 (lâmpada pequena) e 3 (some
   no zoom).

## Smoke do dono (10/10, 3.ª volta) — «pontilhado como Cycles 1 sample», «Size afeta qualidade», «o gizmo deve ter prioridade»

**Gizmo:** `gizmo::glifos` é a ÚNICA conta do ícone (desenho e clique); `gizmo::pick_at` + a shell pergunta-lhe
PRIMEIRO no `pick_objects_at` — gate da shell: vazio e lâmpada no mesmo ponto → só a lâmpada (vermelho sem a
prioridade: `[vazio, lâmpada]`). `961b55387`.

**C3 — a luz DIRETA das lâmpadas** (`bf0a0dddd`; o W5 nº 1 trazido para cá pelo report). `c3_direta.wgsl`,
concatenado depois da C2 e chamado na c0: «cone livre» (fora da penumbra, visibilidade 1 exata) e, na penumbra,
9 raios FIXOS + 5 bissecções da borda (determinístico — o pontilhado era o sorteio de uma 1.ª versão com 32
raios). As `LAMPADAS_DIRETAS_MAX = 8` mais fortes saem da captura e são opacas ANALITICAMENTE na marcha
(`primeira_lampada`). Medido (RTX 5060 Ti, `smoke`, 1920×1080):

| régua | antes da C3 | com a C3 |
|---|---|---|
| ondulação da borda (Low · Medium · High) | 4,08 · 1,19 · 0,77 px | **0,46 · 0,17 · 0,10 px** (gate `a_borda_da_sombra_e_reta_na_placa`) |
| penumbra da cena 4 na placa, lâmpada 0,15 m (geometria 0,328) | 0,945 m | **0,330 m** em qualquer nível (gate a 5 %) |
| G3 penumbra · Alto / Baixo | 0,048 / 0,076 | 0,018 / 0,064 |
| faísca (lâmpada de 2 px) | só no Alto | acende em todo degrau (0,033 · 0,058 · 0,093) |
| ponto (`tamanho 0`) | não acendia | `I/r` a 0,09 % |
| custo por lâmpada (Low · Medium) | — | 0,42 · 1,64 ms (com o teste analítico na marcha); 1ª tentativa com o disco na captura: 1,1 ms (o cone falhava no próprio disco) |
| G4 (1 → 10 000 luzes) | 1,21 % | 0,93 % |

Recusas medidas: 32 raios sorteados (o pontilhado; e 2,3× o custo); a lâmpada direta como disco opaco na captura
(cone falha em todo ponto: 1,1 ms/lâmpada); a lâmpada fora da captura SEM o teste analítico (o céu atravessa-a:
G3 céu e vidro 0,0399 × 0,0126). Leis reescritas (não afrouxadas) nos gates: o ponto, a faísca e a lâmpada pequena
ACENDEM e não são «fora»; o que passa do teto volta a ser relatado; o controlo do anel usa um emissor POLÍGONO;
duas tolerâncias de captura seguem o medido (fenda 0,020; vetor 0,006 — o caminho direto é nítido e vê a
diferença entre as duas capturas). Mutação: 9/10 sangraram de 1.ª; a D7 (o cone não ver OUTRA lâmpada) ganhou o
gate `uma_lampada_direta_faz_sombra_na_luz_de_outra` e sangra (39,5 × 0,0005). Portão: 16 898/16 898 ✓.

⚠️ **ABERTO nº 1 (novo, visto na foto `cena4_low_medium_high.png`):** a sombra NÍTIDA de uma lâmpada pequena,
guardada na grade do nível, sai em ESCADA em Low (células de ~7 px) e em degraus finos em Medium; High é liso. A
cura: a luz direta numa grade FINA própria (≈ 1–2 px, independente do nível), com a fluência das cascatas
(suave) somada por interpolação — os receptores leem uma textura só, como hoje. Custo previsto ~1 ms/lâmpada a
1080p (medir). E a grade inteira é montada para a JANELA, não para a área da cena: ~40 % de texels sob os
painéis (medir antes de cortar).

## W4b — a grade FINA da luz direta (10/10)

- **HEAD:** o commit deste doc · commits `9752e21f7` (placa) · `3313e42da` (família) · `0d1618f6a` (clippy) · `b51e0991f`
  (o joelho) · `6bbabce40` (sondas no passo fino, teto 6) · `5fde2af7c` (os sobreviventes da mutação) · este doc + fotos.
- **Estado:** W4b FECHADA (onda). Próxima: **W5 — janela NOVA** (rebote, HRC, governador acima de 1080p, Flip
  receber/bloquear, e os ABERTOS abaixo). Merge-base `2a925859d`
  (sem rebase pendente). **Delta da shell: 0** (nenhum ficheiro da shell, de `ph2d-render` ou de `ph2d-editor-core`).
- **Desenho** (o do briefing, medido): a c0 deixa de somar a luz direta; `c3_visibilidade` grava em cada sonda da
  c0 a fração à vista de cada lâmpada direta (8 × u8, `Rg32Uint`; `CERTA` = cone livre de uma célula inteira);
  `c3_fina` escreve cada texel fino (texel ÷ `Config::fina`, a MESMA origem ⇒ G11 herdado) = a c0 bilinear +
  Σ parte SUAVE analítica × visibilidade — interpolada onde os cantos diferem ≤ 0,1 e não misturam o fim da
  penumbra com o meio dela (o joelho), medida no texel fino onde não. Os raios diretos marcham no passo FINO
  (nas sondas também). A `LuzRecebida` aponta a fina (API igual). `Degrau::fino_px = 1` ⇒ célula fina de 1–2 px
  em todo nível e zoom. **Só a região da janela se calcula** (`Solver::regiao_fina`, `regiao_da_janela`).

**Antes → depois** (RTX 5060 Ti, `smoke`, 1920×1080, vista de fábrica; rodada `medida_luz2d_w4b`, mínimo de 9
intercaladas; a régua da escada segue os contornos 2 · 10 · 50 · 90 · 98 %, píxel a píxel, do canto até o muro):

| régua | antes | W4b |
|---|---|---|
| escada (degrau rms dos 5 contornos, cena 4, lâmpada pequena) Low | 0,67 · 0,74 · 0,47 · 0,66 · 1,12 px | **0,16 · 0,11 · 0,09 · 0,14 · 0,18** |
| idem Medium | 0,44 · 0,34 · 0,20 · 0,31 · 0,42 | **0,18 · 0,12 · 0,08 · 0,12 · 0,17** |
| idem High (fina = grossa) | 0,18 · 0,11 · 0,06 · 0,12 · 0,18 | igual |
| borda reta da cena 1 (gate, caminho do PRODUTO) Low · Medium | 0,46 · 0,17 px | **0,11 · 0,09** |
| penumbra da lâmpada pequena (geometria 0,3283 m) | 0,330 | 0,337 (2,7 %) |
| o que o receptor lê × gabarito no texel fino (Baixo) penumbra · faísca · céu e vidro | 0,523 · 1,866 · 0,057 | **0,053 · 0,080 · 0,028** |
| custo Low: 0 · 1 · 8 lâmpadas | 1,30 · 1,56 · 2,87 ms (C3 grossa, com a região) | 1,32 · 2,10 · 6,81 ms |
| ms por lâmpada Low · Medium | 0,20 · 0,49 (W4 sem região: 0,42 · 1,64) | **0,69 · 0,67** |
| memória da luz Low | 9,3 MB | 40,8 MB (a fluência fina inteira: 31,5 MB) |

**`LAMPADAS_DIRETAS_MAX` 8 → 6** (medido): `1,32 + 0,686·N ≤ 5,56` no Low a 1080p ⇒ N ≤ 6,2 — a 7.ª lâmpada mais
fraca fica nas cascatas (sombra macia) e o Inspector avisa. O governador vê o custo TOTAL (`TemposMs::total` inclui
C3 vis + fina) e prevê a subida por `por_nivel·r² + fixo` (a fina não muda com o nível).

Fotos (`fotografa_cena.sh`, `HOME` próprio com `light_quality`, `light_auto=0`): `fotos_W4b/cena{1,2,4}_{low,medium,high}.png`
e o recorte junto ao canto `fotos_W4b/cena4_low_medium_high.png`; antes (W4, em cima) × depois:
`fotos_W4b/cena4_antes_depois_W4b.png`.

### Duas voltas que a FOTO deu à régua (lição)

1. Depois do 1.º verde a foto em Low mostrou **manchas claras de uma célula** no lado ESCURO da borda: células de
   cantos «0, 0, 0, 0,08» abaixo do limiar eram interpoladas (até 8 % de luz sobre o preto). A régua de 10 · 50 ·
   90 % não as via — vivem abaixo de 10 %. Contornos de 2 % e 98 % (vermelho: 0,352 px) e o joelho ligado (0,165).
2. Depois, um **salto da borda junto ao muro**, além do fim da régua (que parava na linha de medida). Estendida até
   o muro (parando o perfil antes dele: dentro, 0 ÷ 0 dava cruzamentos falsos de 8 px), mediu o Medium em 0,26 ·
   0,31 · 0,22: onde a penumbra é larga a visibilidade das sondas é INTERPOLADA e, no passo grosso, traz o tremor
   da marcha. Sondas no passo fino: 0,18 · 0,12 · 0,08 (o exato: 0,17 · 0,11 · 0,06).

### ⛔ Recusas MEDIDAS (W4b)

| tentativa | número | por quê |
|---|---|---|
| célula fina de 2 px | escada Low 0,29 · 0,12 · 0,30 (10 · 50 · 90) | é o Medium de hoje — os «degraus finos» do dono |
| visibilidade exata em todo texel fino | 1,22 ms/lâmpada (Low), 11,0 ms com 8 | 1,8× o padrão pela mesma escada |
| grade fina sem a região da janela | 1,77 ms/lâmpada | 82 % da grade fica fora da janela na vista de fábrica |
| raios diretos finos no passo das CASCATAS | 0,53 px no contorno de 50 % (Low) | as amostras entram e saem do canto conforme o texel fino anda |
| sondas no passo grosso | Medium 0,26 · 0,31 · 0,22 px | o tremor da marcha onde a penumbra se interpola |
| limiar 0,25 | Low 0,51 px no de 10 % | o meio da penumbra fica interpolado |
| sem o joelho | Low 0,35 px no de 2 % | as manchas sobre o preto |
| 8+5 raios | +50 % | a mesma régua |
| 2+6 raios | −20 %, a mesma régua | 3 raios fixos deixam passar sem sombra um oclusor mais fino que meia abertura (com 4+5, um quarto) |
| 7 ou 9 bissecções | +20 · +45 % | não curavam o Medium (a causa era o passo, não a bissecção) |
| certificar a sonda (cone livre com a folga de uma célula) | empate de custo E de resposta (fio 0,515 · 0,515) | o `livre` já é conservador em ~2 texels; a mutação dele sobrevivia — saiu |

### Prova de mutação (agente `mutacao`)

Pré-voo: 14 âncoras casaram 1× cada; filtros povoados (placa 7 testes · família 18); restauro por `checkout` +
`touch` (e `shaders.rs` depois de cada `.wgsl`); árvore = HEAD no fim.

| # | mutação | resultado |
|---|---|---|
| M1 | nunca refinar | **sangra** — bordas × tudo 0,0376 (teto 0,0014); escada 0,486 px; penumbra 0,3451 |
| M2 | sem o joelho | **sangra** — escada Low 0,442 px |
| M3 | raios do passe fino no passo grosso | **sangra** — escada Low 0,524 px |
| M4 | sondas no passo grosso | **sangra** — escada Medium 0,258 px |
| M5 | `fina` sempre 1 | **sangra** — texel fino 6,75 px; a régua acha só 101 pontos |
| M6 | região sem a folga de 1 texel | **sangra** — o píxel (0, 0) lê 710..=711, fora de [711, 1850) |
| M7 | sondas da região sem a folga | **sangra** — um texel dentro da região muda de bit |
| M8 | texel fino sem o ½ | **sangra** — 0,519 × 0,943; ponto `I/r` 0,25 |
| M9 | a subida prevê o custo inteiro ×4 | **sangra** — `o_governador_preve_a_subida_com_a_grade_fina_fixa` |
| M10 | `LuzRecebida` com a grade das cascatas | sobreviveu — **equivalente** (o receptor amostra em UV e as duas cobrem o mesmo retângulo); a paridade passa a afirmar a lei (`5fde2af7c`) |
| M11 | G11 sem o encaixe da origem | **sangra** — a ancorada tremeu 12,8 |
| M12 | código das lâmpadas trocado de palavra | **sangra** — bordas × tudo 2,43 |
| M13 | sem certificar a sonda | sobreviveu — numa cena feita para ela (o fio de sombra de 0,8 texel entre as sondas) dá 0,515 com, sem e no exato ⇒ **saiu** (`5fde2af7c`); a cena fica como gate (sangra com M1: 0,085 × 0,515) |
| C1 | `>` → `>=` no limiar | sobreviveu — equivalente, certo |

### Portão de fecho

`fmt --check` ✓ · clippy `-D warnings` (`ph2d-light2d-gpu`, `ph2d-app-light2d`, `ph2d-render`, `ph2d-host-desktop`)
✓ (1 lint num teste, curado em `0d1618f6a`) · `machete` ✓ · `CARGO_BUILD_WARNINGS=deny check --workspace
--all-targets` ✓ · `nextest-impacted` **16 908 / 16 908** ✓.
2.ª corrida, depois dos commits do joelho, das sondas no passo fino e dos sobreviventes (`5fde2af7c`): fmt ✓ ·
clippy ✓ · machete ✓ · deny-check ✓ · `nextest-impacted` **16 909 / 16 909** ✓. Suítes da família: placa 20 + 3
ignorados · família 37 + 6 · ECS verde. Smoke recompilado (regra I, depois do `rm -rf target/*/incremental`),
2.ª saída: `Finished \`smoke\` profile [optimized] target(s) in 0.20s`.

### ⚠️ ABERTO (W4b → W5)

1. **A borda inteira desloca-se ~½–¾ de texel GROSSO** para o lado da sombra (Low −4,2 px · Medium −2,1 · High
   −1,2 no contorno de 50 %; gate: ≤ 0,95 texel). Causa medida: a captura grossa ARREDONDA o canto do oclusor
   (`meio` = cobertura bilinear ≥ ½); a marcha com passo ¼ só tira ~16 %. Cura: uma captura FINA para a luz
   direta (2.ª C0 + JFA na grade fina, com sprites e a camada do vetor desenhados também nela) — custo por medir.
2. **Oclusor mais fino que ~1 texel grosso some da captura** (Low: uma barra de 5 cm não faz sombra; Medium faz) — a
   mesma cura do 1.
3. **A faixa escura na face acesa da parede** (o interior do opaco recebe 0 e a bilinear mistura-o): agora 1 px em
   TODO nível (Low antes 1,25 px a 0,73 do gabarito; agora 1,0 px a 0,45 = o High). Não curou — é o ABERTO 1 da W4
   (o passe de preenchimento).
4. **As cascatas ainda calculam a grade inteira**: 81,5 % dela fica fora da janela na vista de fábrica (cobre o pior
   zoom da oitava + a margem); fora do CANVAS 87,7 %. A região C3 pelo canvas tiraria mais ~⅓ — o retângulo só
   existe do quadro anterior (`HeroScreen.last_canvas`, escrito no paint depois do `codificar`).
5. **O teto de lâmpadas diretas caiu para 6** (era 8): o custo do Low com a grade fina. As curas do 4 devolvem-no.
6. **`PassList::bytes_por_quadro` (K7) conta a textura fina inteira** (31,5 MB lidos + escritos); a região move
   ~5,8 MB. Governador > 1080p (W5).

### O que a W5 precisa saber (W4b)

- Um passe novo que precise da fluência final lê `luz2d/tex/fluencia` (`Solver::fluencia`, grade
  `Solver::grade_da_fluencia`); a das cascatas é `PassList::indice_da_c0`.
- Os testes leem a fluência COM a grade dela (`comum::Fluencia`); toda régua que compara com um solver feito à mão
  dá-lhe a MESMA configuração e a MESMA região (`regiao_da_janela`) — senão compara outro programa (as réguas de
  captura mudaram de população por isso: a fenda mede 0,0247 sem a fina e 0,0233 com ela; teto 0,026).
- A rodada `medida_luz2d_w4b` (custo · escada · painéis · faixa) é a régua dos ABERTOS 1–5; as variantes do refino
  são `Bordas` (configuração em execução, nunca `const`).
- **Fotografe antes de acreditar na régua**: as duas voltas acima.

### Smoke do dono da W4b (10/10) — «demais SMOKE OK» e um defeito

**Report:** *«Cor escura na lâmpada faz com que a lâmpada se comporte como um objeto que gera sombra de outra
lâmpada.»* Mecanismo: a lei do gabarito desde a W1 fazia todo emissor com área OPACO; uma lâmpada clara escondia
com a sua luz a sombra que fazia, uma escura não. Lei nova, no gabarito e na placa (`4f590e1c8`): **uma lâmpada é
luz, não corpo** — o raio que a atravessa soma a radiância dela e segue; os CORPOS que brilham (polígonos) continuam
opacos. Vermelho-primeiro: a placa respondia igual à lâmpada-corpo (0,9924); agora 0,0003 (direta) · 0,0004 (fina
×4) · 0,0019 (pelas cascatas, além do teto). ⚠️ O G4 (K3) passou de 0,93 % para 4,12 % (limite 5 %): lâmpadas
capturadas são atravessadas em vez de pararem o raio — sob `load` 27 ele reprovou uma vez fora da faixa isolada.

**Ordem do dono:** «coloque na fila de implementação o que ainda não está bom» ⇒ `01_plano.md` §7.1 (F1–F3), a
próxima janela começa por ela.

**Smoke do dono (10/10, 2.ª volta da W4b): «smoke OK»** — a lâmpada escura já não faz sombra. W4b fechada; a
próxima janela começa pela fila `01_plano.md` §7.1.
