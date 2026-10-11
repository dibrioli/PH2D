# 05 §4b — Estado, pedaço 2 de 4: o desfeito, a água/papel/Tilt (W1e), o achatamento (W1f) e as opções (W1g) (09–10/10)

> Pedaço do §4b de [`05_plano_da_linha_e_o_antes.md`](05_plano_da_linha_e_o_antes.md) (o índice), cortado em
> 2026-10-11 por `scripts/doc-split.py`: corpo **verbatim** (a remontagem bateu sha256 com o original;
> 0 alvo(s) de link reancorado(s) para o MESMO ficheiro). Continua em [`05c_estado_mistura_e_claridade.md`](05c_estado_mistura_e_claridade.md).
>
> ⛔ O que aqui estiver marcado **«medido e REJEITADO»** continua rejeitado.

---

- ⏪ **DESFEITO — as gotas, a linha de contacto e a água de fábrica delas** (dono, 09/10: *«parece que o
  traço perdeu qualidade desde o momento que vc introduziu os limites e o sistema de drips. Consegue
  desfazer isso tudo?»* → escolheu «voltar tudo menos o Smooth Edges»). O CÓDIGO (80 ficheiros fora de
  `docs/`) voltou ao de `69b4f1a75` (08/10, o preview vivo, antes da W1b): sem `drips*`, sem Create/Easy
  Drips nem Drip Size/Length no Paper, sem `DripLaw`/`DRIP_LAW_GRAVITY`, a Gravity e a água dos 4 pincéis de
  fábrica as de antes, o dial Tilt como era, o degrau do schema de novo **187** (sem o 188 nem a migração
  `ProjectFileV187` — ⚠️ um projecto gravado durante os testes das gotas, a 188, não abre nesta build). Ficou
  SÓ o **Smooth Edges** do Wet Paint, como OPÇÃO **desligada de fábrica** (o checkbox abaixo de Paper;
  desligado = o composite de antes ao byte, gate `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes`).
  - Ficaram em `docs/` (sem efeito no programa): os docs 06–10, as fixtures do Rebelle (`rebelle_gotas/`,
    `s_zoom*.png`), as capturas do dono e as réguas Python (`borda.py`, `borda_cobertura.py`,
    `smooth_edges_no_wet.py`). Os testes Rust das gotas e da auditoria (`borda_do_s.rs`,
    `auditoria_borda_w1c.rs`, `gotas_pela_lei_do_rebelle.rs`, as sondas) saíram com o código; vivem no
    histórico (`8d3e8f68a`).
  - ⛔ **Para quem voltar às gotas:** o dono julgou que a BARREIRA (a linha de contacto que prende a água no
    papel seco) e o que veio com ela pioraram o TRAÇO, não só a borda. Antes de reconstruir, compare o traço
    normal (tela direita e inclinada, os pincéis de fábrica) contra este estado, com o dono, e leia a
    tabela de recusas da W1b/W1c. O prompt [10](10_PROMPT_janela_W1d_cura.md) está OBSOLETO.
  - Verificado depois de desfazer: `check --workspace --all-targets` ✓ · clippy `-D warnings` (6 crates) ✓ ·
    suítes de ph2d-tool-painter / panel-painter-layers / i18n / wet-paint / panel-wet-tuning ✓ · testes do
    projecto da shell (98) ✓ · `cerca.sh` ✓ (0 ficheiros protegidos no diff).
- 🔎 **Depois de desfazer (09/10): o escorrimento, o papel e o Tilt** — ordens do dono e medições no
  [11_PROMPT_janela_W1e_agua_papel_tilt.md](11_PROMPT_janela_W1e_agua_papel_tilt.md): o motor escorre em PENTE
  (7–10 dedos por traço) onde o Rebelle não escorre (a meio) ou faz UM fio (no máximo); o Wet Paint tem dois
  papéis (o do painel quando armado, o procedural do motor sem Kind — o dono: obedecer ao painel); o pincel
  seco não pega nos picos com o papel do motor (picos ÷ vales 1,06; com o do painel 1,81); o preset não guarda
  o Tilt; a MISTURA: azul sobre vermelho dá azul (molhado ≈ seco), a água só corre depois do Up. ✅ Feito: o
  menu de presets lista cada pincel uma vez (`3d8e40be1`). Sondas:
  `tests/it/sonda_tilt_do_preset.rs`, `wetpaint/sonda_papel_seco.rs`, `wetpaint/sonda_mistura.rs`, régua `ferramentas/tilt_rebelle.py`.
- ✅/⚠️ **W1e (09/10, prompt [11](11_PROMPT_janela_W1e_agua_papel_tilt.md))** — `2e92420be` (F6) · `0b64130fa`
  (F2+F3) · `c91dade9c` (F1+F4+F5). Rebelle 8.3.4 pela porta WebSocket (`ferramentas/mistura_papel*.{py,sh}`),
  réguas `ferramentas/velatura.py` (γ em luz linear) e `ferramentas/fios.py`. Pranchas 1:1 e 8×:
  [escorrimento](prancha_w1e_escorrimento.png) · [8×](prancha_w1e_escorrimento_8x.png) ·
  [mistura](prancha_w1e_mistura.png) · [8×](prancha_w1e_mistura_8x.png) · [pincel seco](prancha_w1e_papel_seco.png).

  | | antes | depois | Rebelle |
  |---|---|---|---|
  | **F6** seco: γ durante · depois (0 = por cima, 1 = multiplicar) | 0,17 · 0,06 | **0,47 · 0,58** | 0,42–0,49 · 0,44–0,51 |
  | F6 molhado: γ G/B durante · sangria no vermelho a 15 px | 0,48/0,31 · +7 | 0,47/0,42 · **+19** | 0,67/0,49 · +12…+21 |
  | F6 seco: sangria · pincel sujo (rasto − azul) | +6 · (5, 0, −5) | **−3…0** · (6, −1, −7) | +1…+2 · ≈ 0 |
  | F6 a água corre com o traço (quadro a pintar, 4096², poça pesada, pior) | 1,6 ms, água parada | **11,7 ms** (sem pintar: 11,1) | corre |
  | **F2** fontes de papel do Wet Paint | 2 (motor sem Kind · painel) | **1** (sem Kind = o Cold de fábrica do painel) | 1 (o papel) |
  | F2 controlos de papel fora da secção Paper | 6 (grupo PAPER do Tuning + «Paper») | **0** («Paint Texture» na secção Paper) | — |
  | **F3** Wet Dry Brush sem Kind: picos ÷ vales · r(escuro, altura) | 1,06 · +0,05 | **1,34 · +0,28** | sem barra (ver abaixo) |
  | F3 molhados (Round · Wash · Drip) picos ÷ vales | 0,95 · 0,98 · 0,94 | 0,95 · 0,98 · 0,94 | — |
  | **F1** Wet Round, bancada: tela direita · meia · máxima (fios ≥ 20 px) | 0 · 0 · 1 | **0 (ao bit) · 0 · 0** | 0 · 0 · 1 fio fino de 250–300 px |
  | F1 orla de baixo ÷ de cima a meia (água 1) | 0,76 | **1,30** | 1,6 |
  | F1 pincéis de fábrica, meia (Round · Wash · Dry · Drip) | 0 · 0 · 0 · 4 | 0 · **4** · 0 · **0** | — |
  | F1 pincéis de fábrica, máxima | 1 · 4 · 0 · 6 | **0 · 1–6 · 0 · 3** (dentinhos < 20 px no Round) | Round 1 fio |
  | **F5** gravidade dos pincéis (Round · Wash · Dry · Drip) | 0,005 · 0,003 · 0,002 · 0,01 | **0,005 em todos** | não existe por pincel |
  | **F4** o preset guarda o Tilt | não | **sim** (preset antigo não mexe; Round/Wash/Dry direitos, Drip no máximo) | a tela nasce direita |

  - **F6:** `colorops::velatura` — a junção «por cima» ganha o termo de Beer–Lambert na fração γ = 0,47
    (`VELATURA_MEDIDA`), × a distância entre as cores (a MESMA tinta não se vela: sem o factor um traço de
    uma cor escurecia 202 → 156). Em todas as junções (pouso sobre suspenso, assentamento, Dry canvas,
    re-wet, render). `Sim::sim_while_painting`: a água corre com o traço; o lote que chega com o motor na
    simulação espera numa fila e pousa no tick (sem ela o quadro esperava o estágio: 19 ms). Duas fugas
    que a água viva expôs, curadas: o push do Impasto ligava o aquecimento do motor de traço no Wet Paint;
    um carimbo redondo rodava o último bit do raio. ⛔ γ = 1 escurece demais (0,80–0,91) e mata a sangria.
  - **F2/F3:** sem Kind, o motor é semeado pelo `papel_de_fabrica()` do painel (o Cold que o Relief arma e
    acende) pela mesma lei do Kind armado; o papel procedural do motor deixou de ser fonte no produto e o
    grupo PAPER do Tuning saiu inteiro; o «Paper» do Wet Paint é a **Paint Texture** da secção Paper (o nome
    do Rebelle, `paint_texture`); o Tooth aparece sem Kind no Wet Paint. Relief/Roughness já iluminavam o Wet
    Paint (a luz é da tela; o censo dos controlos o prova). Custo: semear o papel a 4096² 11 ms (re-assar o
    do motor: 1 ms), uma vez por sessão. ⛔ Recusa medida: o ruído de recurso da Aquarela como papel sem
    Kind (o Tooth deixava de ser monótono: halo 81 / 60 / 91).
  - ⚠️ **F3 sem barra do Rebelle:** pela porta, sob o Wine, o papel não se desenha (branco com desvio 0), o
    tamanho do Flat Dry não pega depois de um `SET_ENGINE_PARAMS` (saiu um fio de 12 px numa faixa de 30 —
    os «73 % de papel branco» eram faixa vazia) e o registo da altura do papel é ambíguo (|r| 0,20 em duas
    posições de sinais opostos). A barra ficou a lei física (seco > 1 nos picos, molhados < 1). ⛔ Não
    reabra por esta porta: `SET_PAPER` derruba o 8.3.4 («bad optional access»).
  - **F1 (PARCIAL):** `solver::Gota` — numa célula SECA (a advecção é por recolha: é ela que puxa), a
    componente que desce leva `min(1, 2·hill(h/h_c))`, `hill = q⁸/(1+q⁸)`, `h` = a conta logo acima,
    `h_c = 0,2/|g|` (`ALTURA_DA_GOTA`); nos construtores travado e suavizado. Tira o pente do Round e do Drip
    a meia inclinação e escurece a orla de baixo como o Rebelle; uma poça de 4 passadas continua a correr.
    ⛔ **Recusas medidas** (bancada Round, `sonda_gota`): a lei só no FREIO (morde 1 quadro em 4: nada muda) ·
    pela altura da própria célula seca (zero — uma barreira, prende tudo) · o fluxo INTEIRO da célula seca
    (8–9 dedos e riscos brancos no topo do traço) · `q⁴` (a conta do Wash, 17,7 contra a crítica 40, fugia
    7 %) · mais nivelamento (2 dedos mas largos, e muda a tela direita) · `extFingering` está a 0 nos 4
    pincéis (não é a origem do pente).
  - **Aberto (F1, a próxima onda):** o fio ÚNICO, fino e longo do Rebelle no máximo não sai de nenhuma lei
    local — 3 topologias medidas, parei pela regra dos dois tropeços. Precisa de um mecanismo NÃO local (a
    conta de baixo a equalizar a pressão ao longo da orla, para o primeiro fio drenar os outros). E o Wet
    Wash e o Wet Drip (água de fábrica 1,0; o Rebelle usa ~0,24) fazem contas de 40–60 de altura e ainda
    abrem 3–6 dedos no máximo — o Wash também a meia, desde que a F5 lhe subiu a gravidade de 0,003 para
    0,005. A água de fábrica foi revertida com a W1b; decidir com o dono.
- 🔎✅ **W1f — auditoria do ACHATAMENTO (10/10, prompt [12](12_PROMPT_janela_auditoria_achatamento.md))**
  (smoke do dono sobre `a9ce1a4c6`: *«áreas retas, parece que retângulos virtuais influenciam»* —
  [captura](fixtures/smoke_dono_w1e/achatamento_s_vermelho.png)). Cena: o S do oráculo a 1:1 (Wet Round
  100 px, água 1, 30 s; `wetpaint/sonda_achatamento.rs`), tela direita · meia · máxima · diagonal (raio 2).
  Régua NOVA `ferramentas/recto.py`, portada em `wetpaint/achatamento_tests.rs` (a mesma conta): a orla de
  baixo plana mais longa (±0 px) e a fracção de fios LARGOS (≥ 8 colunas) com a ponta cortada a direito
  (≥ 70 % das colunas na mesma última linha ±1). Calibrada no Rebelle (5 telas `s_*.png`): plano 19–22,
  pontas quadradas 0 (os fios dele têm 3–7 colunas); a captura do dono: plano **62** (e 33, 27, 24 nos
  três sítios marcados).
  - **Causa (medida célula a célula, `sonda_ponta`): a lei da gota da W1e.** Na ponta de um fio a água
    está no MEIO (filme 10–19 no centro, 0–3 nas bordas), mas todas as colunas descem juntas, uma linha de
    cada vez: (1) a altura crítica (`h_c` 40–80) quase nunca é atingida — a frente avança porque a célula
    «seca» destrava com `film > 1e-3` (um degrau binário); (2) a lei só trava a componente que DESCE, a
    de lado fica livre — a linha nova enche-se de lado até à largura do fio antes de a seguinte abrir.
    Resultado: fundo plano, cantos a 90°, paredes verticais. Aos 8–15 s as pontas ainda são redondas; o
    corte aparece no rastejar final (a 30 s).

    | H | teste | resultado | veredicto |
    |---|---|---|---|
    | **H1** a lei da gota | S com `drip_head` 0 × 0,2 | com a lei: meia plano 33 (com a água no traço) / 20, pontas quadradas 3 de 3; máxima 20 de 22. Sem ela: 13 · 0; 10 · 1 de 24 | **a causa** |
    | H1 «ĝ arredondado» | fonte bilinear ao longo do fluxo · máximo dos 3 de cima · máximo de 4 na coluna | meia plano 33 / 33 / 20, pontas iguais | ⛔ não é o arredondamento |
    | **H2** recorte por eixo | `maxVelocity` 3 (e + `levelClamp` 1) | 33 (76 com o nivelamento) — pontas quadradas iguais | ⛔ morta |
    | **H3** rectângulos reais | `spans_enabled = false`; ecrã × recomposição inteira; `rf` | byte-idêntico; 0 px de diferença em 60 cenas; `rf = 1` de fábrica | ⛔ morta |
    | **H4** o fio que parou | a ponta ao longo do tempo (`PH2D_SONDA_X`) | redonda a correr, cortada no rastejar (o mecanismo acima) | ✅ sintoma da H1 |

  - **⛔ Recusas MEDIDAS (curas da lei que mantêm a trava na entrada do seco — ~40 variantes, 6 rodadas):**
    altura crítica modulada pelo papel (`h_c·(1 ± 0,5…1)`: plano 33 → 23, pontas quadradas iguais) ·
    limiar de «seca» relativo (`0,05–0,25·h_c`: a orla vira ESCADA, cantos 49–50) · conta presa abaixo de
    `0,6·h_c` (plano 36) · o fluxo INTEIRO da célula seca com `h_c/cos θ` (escadas; em todas as direcções
    os riscos brancos e 9–10 fios no máximo da bancada) · transição seco→molhado contínua (escadas e fios
    na MEIA) · a lei na GRAVIDADE em vez da entrada (piso 0: nada se move, igual à tela direita mesmo no
    máximo; piso 0,25–0,5: a orla de baixo deixa de escurecer, 0,76–0,93). ⇒ uma porta célula a célula na
    entrada do papel seco imprime a grelha; não há variante dela sem rectângulos ou escadas.
  - **Decisão (pelo número): a lei sai do produto** (`drip_head = 0` no nascimento da sessão; o código
    fica no motor como instrumento das sondas, o porquê no doc de `ALTURA_DA_GOTA`). Portão novo
    `o_escorrimento_do_s_nao_se_corta_em_rectangulos` (meia e máxima; controlo DENTRO: com a lei reprova;
    mutação «a lei de volta no produto» ⇒ VERMELHO). `a_meia_inclinacao_nao_escorre_e_a_orla_de_baixo_escurece`
    passou a `a_meia_inclinacao_nao_escorre` (o produto, ≤ 20 px; controlo: G 0,01 no máximo escorre).

    | S / bancada | antes (W1e) | depois (W1f) | Rebelle |
    |---|---|---|---|
    | S meia: orla plana · pontas quadradas | 33 · 2 de 3 (com a água no traço) | **13 · 0 de 3** | 19–22 · 0 |
    | S máxima: pontas quadradas | 19 de 21 | **1 de 24** (redondas) | 0 (fios de 3–7 px) |
    | bancada Round: direita · meia · máxima (fios ≥ 20 px) | 0 · 0 · 0 | **0 · 0 · 1** (21 px) | 0 · 0 · 1 (250–300 px) |
    | ⚠️ orla de baixo ÷ de cima a meia (água 1) | 1,30 | **0,77** | 1,6 |
    | pincéis de fábrica, meia (Round · Wash · Dry · Drip), fios ≥ 20 px | 0 · 4 · 0 · 0 | 0 · 5 · 0 · 0 | — |
    | pincéis de fábrica, máxima | 0 · 1–6 · 0 · 3 | 1 · 5 · 0 · 3 | Round 1 |

    ⚠️ **O que se perdeu** (aberto, a decidir com o dono): a orla de baixo a meia já não escurece (a lei
    prendia a água na orla; sem ela rasteja, como no port). E o S no MÁXIMO continua com ~20 fios
    grossos (antes e depois; o Rebelle 10–11 finos e longos) — o «fio único» da W1e continua aberto.
  - **A água só corre depois do Up (dono, 10/10: *«prefiro tudo em tempo real, mas precisamos pintar
    imagens de até 4k»*).** Medido a 4096² (poça pesada, `measure_a_painting_frame_with_the_water_running`,
    7 blocos intercalados, load 25–40): o quadro a pintar com a água parada p50 1,2–1,6 ms · pior 2,1–4,0;
    com ela a correr no traço p50 4,1–4,7 · pior **14,1–44,6 ms** ⇒ `sim_while_painting` desligado no
    produto. A mistura continua a ver-se a pintar (o pouso do dab vela), mas o render do filme MOLHADO
    sobre o seco velava demais sem a água a assentar no traço (seco durante γ 0,76): ganhou constante
    própria, `VELATURA_DO_FILME` = 0,20 (`Sim::glaze_filme`; varrida em `sonda_velatura_do_filme`):
    seco γ durante · depois 0,76 · 0,62 → **0,55 · 0,43** (Rebelle 0,42–0,49 · 0,44–0,51), molhado
    sangria +25 (Rebelle +12…+21). `azul_sobre_vermelho…` verde.
  - Pranchas: [1:1](prancha_w1f_achatamento.png) · [8× nas pontas](prancha_w1f_achatamento_8x.png)
    (antes · depois · Rebelle · a captura do dono).
- ✅ **W1g — OPÇÕES NOVAS, desligadas de fábrica (10/10, prompt [13](13_PROMPT_janela_W1g_opcoes.md))**
  (dono 10/10: *«se vc sabe como melhorar sem danificar e sem provocar limitações e sim criando novas
  opções»*). Duas opções entregues na secção Wet Paint, uma recusada pelo número.
  - **A cerca: desligadas = o traço de hoje AO BYTE.** `opcoes_tests::com_as_opcoes_desligadas_o_s_e_a_bancada_sao_os_de_hoje_ao_byte`
    fixa o FNV do S e da bancada nas três inclinações, gravados em `fcc685e43` ANTES do primeiro
    commit (verde antes e depois). Miniaturas de fábrica (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`),
    `os_outros_meios_nao_mudam`, `cerca.sh`, a suíte de `ph2d-wet-paint` (impressões do port) e o censo
    dos controlos: verdes antes e depois.
  - **O1 «Live Water»** (caixa, `PAINTER_WETPAINT_LIVE_WATER`): a água corre com o traço aberto
    (`Sim::sim_while_painting`, levado ao motor vivo pelo `reconcile_facts`). Dica no painel com o custo
    MEDIDO na W1f (4096²: pior quadro 2–4 ms → 14–45 ms). ⚠️ **A velatura do filme depende do modo**:
    com a água ao vivo o `VELATURA_DO_FILME` 0,20 sangrava o azul `+3,2` sobre o vermelho seco (barra
    ≤ 3) ⇒ `VELATURA_DO_FILME_VIVA` = 0,35 (tabela no doc da constante, `colorops.rs`), trocada pelo
    reconcile com a opção.

    | mistura (seco γ durante · depois · sangria; molhado sangria) | água parada (fábrica) | Live Water | Rebelle |
    |---|---|---|---|
    | velatura 0,20 | 0,55 · 0,43 · +3 · +26 | 0,42 · 0,41 · **+3,2 ✗** · +28 | 0,42–0,49 · 0,44–0,51 · +1…+2 · +12…+21 |
    | velatura 0,35 | 0,67 · 0,54 · +2 · +25 | **0,57 · 0,51 · +2,9** · +28 | idem |

    Portões: `com_a_live_water_a_agua_corre_com_o_traco` (pelo autorado: fábrica 0 passos com o traço
    aberto, ligada > 0) · `com_a_live_water_a_mistura_continua_na_barra_do_rebelle` (controlo DENTRO: a
    velatura da água parada com a água ao vivo reprova). ⚠️ Aberto: a melhoria do custo (simular só a
    janela perto do pincel durante o traço) é um item à parte, não medido.
  - **O2 «Heavy Edge»** (linha `0..1`, `PAINTER_WETPAINT_HEAVY_EDGE`, logo abaixo do disco do Tilt;
    esmaece com a dica «Acts when Tilt is on» — `Dependente::BordaSemTilt`): **o pigmento desce mais que
    a água** — `solver::sedimenta` (filho de `solver/`), um passo depois da advecção: a fracção
    `a·|ĝ_eixo|` do pigmento EM SUSPENSÃO passa à vizinha de baixo, pesada por `min(f_c, f_d)/f_c` (a
    água das duas a tocar-se; seca = 0 ⇒ o pigmento acumula-se na última água da orla). Não move ÁGUA,
    nenhuma decisão por célula (contínua no filme), conservativa (gather), independente de ordem;
    `a = 0` ou `|g| = 0` não entra. `a = k·|g|`, `k = slider × SEDIMENTA_DA_BORDA` (10).
    Rodada única (`sonda_opcoes`, bancada Wet Round + S, água parada × ao vivo), orla de baixo ÷ de cima
    do traço de água 1:

    | `k` no motor | 0 (fábrica) | 1 | 2 | **5 (slider 0,5)** | 10 (slider 1) | 20 | 40 |
    |---|---|---|---|---|---|---|---|
    | meia | 0,77 | 0,85 | 1,07 | **1,56** (ao vivo 1,81) | 4,14 | 5,08 | 5,78 |
    | máxima | 0,78 | 0,97 | 1,30 | 2,88 | 2,95 | 3,49 | 0,72 (drena) |
    | direita | 0,81 | = | = | = | = | = | = |
    | S meia: plano · fios · quadrados | 13 · 3 · 0 | 13 · 3 · 0 | 13 · 3 · 0 | 13 · 3 · 0 | 14 · 3 · 0 | 12 · 3 · 0 | 20 · 6 · 0 |
    | S máxima: plano · fios · quadrados | 10 · 24 · 1 | 10 · 23 · 1 | 10 · 23 · 3 | 10 · 23 · 2 | 10 · 22 · 2 | 10 · 22 · 2 | 11 · 21 · 1 |

    Rebelle a meia: 1,6. Portão `a_heavy_edge_a_meio_pesa_na_orla_de_baixo_como_no_rebelle` (slider 0,5
    pelo autorado: barra 1,3–1,6; controlo DENTRO: a fábrica 0,77 reprova; a régua do recto verde no S a
    meia e no máximo; a tela DIREITA com o slider a 1 = o hash de hoje). ⚠️ Os traços de água 0,4 descem
    a 0,75 com k 5 (o pigmento sai do topo E da orla de baixo, que é fina) — a barra do Rebelle é a da
    água 1.
  - **Painel e preset:** as duas no painel (i18n + tokens), no Reset (voltam ao neutro), no preset
    (`Parametros::live_water` / `heavy_edge`, opcionais: um preset de antes delas NÃO mexe — a regra do
    Tilt; `um_preset_sem_as_opcoes_da_w1g_nao_mexe_nelas`, e `o_pincel_guardado_pinta_o_mesmo_traco`
    agora com as duas ligadas). Costura: `tests/it/seam_wetpaint_opcoes.rs` (presentes só armado, vivas
    sob o ponteiro, o clique real liga/desliga na ferramenta, a edição vira `SetValue` e prende-se a
    `0..1`, as duas dicas).
  - **⛔ O3 «fios finos e compridos no máximo» — NÃO ENTREGUE** (a regra do prompt: nenhuma lei passou
    (a)–(c) numa rodada). Antes de construir lei, medida a COMPOSIÇÃO: 14 variantes dos knobs que o
    motor já tem (`sonda_fios`), S a meia e no máximo, régua nova `ferramentas/fios_s.py` (fios = colunas
    cuja tinta passa a base da orla por ≥ 20 px), calibrada no Rebelle: `s_inclinado` 11 fios, largura
    mediana 5 (máx 7), comprimento mediano 306; `s_inclinado_1` 8 · 5 · 273; `s_zoom100` 8 · 5 · 292;
    `s_plano` 0.

    | variante | máxima: fios · largura · comprimento · finos-e-longos | meia: fios (fábrica 3) | veredicto |
    |---|---|---|---|
    | fábrica | 22 · 10 · 51 · 0 | 3 | a referência |
    | `extFingering` 0,3 · 1,0 · 1,0 + `extRivulets` 8 / 40 | 22–23 · 10 · 48–51 · 0 | 2–3 | ⛔ não afina nem alonga |
    | `leveling` 0,2 · 0,1 | 25 · 10 · 53 · 0 / 33 · 8 · 48 · 0 | **10 / 16** | ⛔ fios a mais a meia |
    | `levelClamp` 0,05 | 30 · 8 · 42 · 0 | **16** | ⛔ fios a mais a meia |
    | `viscosity` 0,05 | 23 · 10 · 60 · 0 | 3 | ⛔ não afina |
    | `capillary` 0,6 · `maxVelocity` 2 | 22 · 10 · 51 · 0 | 3 | ⛔ nada muda |
    | `brake` 0,5 | 24 · 10 · 66 · **1** | **10** | ⛔ 1 fio fino, fios a mais a meia |
    | fingering 1 + rivulets 10 + leveling 0,2 · leveling 0,1 + viscosity 0,05 | 25 · 10 · 46 · 0 / 36 · 8 · 45 · 0 | 7 / 19 | ⛔ |

    Nenhuma variante passa de 1 fio fino e longo (Rebelle 8–11), e todas as que afinam acrescentam fios
    a meia. Confirma a W1e: o fio único/longo é NÃO LOCAL (a conta de baixo a equalizar a pressão ao
    longo da orla) — um mecanismo novo, a medir numa onda própria; os vetos continuam (canal, linha de
    contacto, Create/Easy Drips, DripLaw, trava célula a célula na entrada do seco).
  - Pranchas: [1:1](prancha_w1g_opcoes.png) · [8×](prancha_w1g_opcoes_8x.png) (cada opção desligada ·
    ligada · Rebelle; a linha da Live Water é o S no instante do Up — o Rebelle não tem foto no Up, a
    coluna dele é 1 min depois). Gerador `ferramentas/prancha_w1g.py`.
  - **Smoke do dono (10/10): OK**, com dois achados e um pedido, curados na mesma janela:
    - ⛔ **O preview do painel ignorava a Heavy Edge** (e a Live Water): `CenaDoPreview`/`ChaveDoPreview`
      copiam o estado do fluido para o pintor do preview, e as opções novas ficaram de fora — a família
      «copiar um componente deixa os irmãos». Cura: `opcoes` na cena e na chave. Red-first:
      `a_chave_muda_com_cada_parametro_e_o_traco_tambem` com os casos novos deu «heavy edge: a chave não
      viu» antes da cura.
    - **O Undo com as opções** (`medida_undo`, 4096², quatro ferramentas no MESMO processo, 7 rodadas
      de ordem rodada, mínimo · mediana, load ~37 — o relógio absoluto vale pouco, a comparação vale):

      | | Ctrl+Z | 1.º quadro do traço seguinte | undo guardado |
      |---|---|---|---|
      | fábrica | 55,7 · 69,4 ms | 63,5 · 72,3 ms | 3,1 MB |
      | Heavy Edge 1 | 48,5 · 68,5 | 64,3 · 67,4 | 3,1 MB |
      | Live Water | 53,5 · 66,4 | 66,4 · 78,8 | 3,1 MB |
      | as duas | 49,2 · 70,8 | 62,5 · 82,5 | 3,1 MB |

      ⇒ as opções não pesam no Undo (o Undo não passa por elas: restaura a tela e a água renasce no
      traço seguinte). ⚠️ O Ctrl+Z de ~50 ms a 4096² é anterior à W1g — um item à parte, por medir sem
      carga.
    - **Smooth Edges com intensidade** (pedido do dono: «ajustar a intensidade deste smooth»; ⚠️ a LEI
      abaixo foi substituída no 2.º smoke — «no máximo não se vê» — pela borda suave por raio): a caixa
      virou uma linha `0..1` (o mesmo id, de `CLICKS` para `FIELDS`). A lei: a cobertura premultiplicada
      do pixel da borda entre o desligado (o `dele`) e o ligado, `d + (l − d)·força`; `0` não monta a
      silhueta (o desligado de antes) e `1` é o ligado sem conta nenhuma — os dois AO BYTE contra o FNV
      gravado com o código do checkbox em `362938a99` (`smooth_edges_nos_extremos_e_o_checkbox_de_antes_ao_byte`,
      razão 1 e 2); `smooth_edges_a_meio_fica_entre_o_desligado_e_o_ligado` (a 0,5 cada byte entre os
      dois, nenhum muda onde eles concordam). O Smooth Edges do Watercolor não muda (a cerca). ⚠️ Mais
      forte que o ligado de antes (um pé maior que 3×3) seria outra lei — não pedida, não construída. Mutação «o
      slider vira interruptor» (`forca > 0` = ligado) ⇒ o gate do meio VERMELHO. Como linha numérica o
      nome corta no mínimo do dock (220 px): a catraca `cada_nome_deste_painel_cabe_na_coluna_da_seccao`
      sobe 12 → 13 pelo precedente de 23/09 (linha que ENTRA na população; de 245 px para cima nada muda).
  - **2.º smoke do dono (10/10), três pedidos:**
    - **O preview tem de ser EXATO** («não escorre como o brush real»). O preview pintava num quadrado de
      256 px, o diâmetro preso a 96 px, pressão a subir e a descer, e só **2 s** de água — o escorrimento
      real leva dezenas de segundos (medido, `medida_preview`, perfil smoke, até o motor ficar sem
      fluido: Wet Round 33 s, Wet Wash 60 s; custo 1–8 s numa thread). O preview vivo (`pintar_preview`)
      passou a: o pincel no tamanho REAL numa tela 2:1 de `clamp(10·d, 512, 1024)` px (1:1 com a obra),
      o S no terço de cima (por baixo cabem ~2,9 d de fios: na tela 1:1, Tilt no máximo, água 1, os fios
      descem 0,7–0,8 d no Wet Round e 2,4–2,8 d no Wet Wash), pressão 1 (a do rato), a água até PARAR
      (tecto 120 s) e cada quadro reduzido pela média a 384×192. A receita antiga ficou só para a
      miniatura dos ficheiros (`pintar_miniatura`). Portão `o_preview_escorre_como_a_tela` (Wet Round
      60 px, Tilt máximo: 444 px de fios no último quadro; controlo: a receita de 2 s, 0 px). ⚠️ A
      sonda tinha dado 0 porque o preset traz a tela DIREITA (F4) — o portão liga o Tilt.
    - **O Smooth Edges no máximo não se via.** A 1.ª lei (o 3×3 do Watercolor) é um anti-serrilhado de
      1 px; a 2.ª tentativa (o raio a alisar a FORMA da mancha) também quase não se via — no S a
      silhueta já é lisa; o que se vê duro é o contorno ESCURO, por dentro da mancha. Lei entregue
      (`Suave`): a mancha desfocada por uma tenda de meia largura `raio = slider × 6` dá a faixa da orla
      (`B ∈ (0,1)`); nela a tinta (premultiplicada) é desfocada pela mesma tenda com peso `4·B·(1−B)`. No
      S: ¼ tira a escadinha, ½ amacia, 1 dá uma borda suave. Portões: o zero ao byte (razão 1 e 2);
      `mais_smooth_edges_e_uma_borda_mais_suave` (a aspereza — soma das diferenças entre vizinhos — cai
      27 % → 37 % → 45 % a ¼ · ½ · 1; a tinta total dentro de 2,3 %; mutação «raio fixo» ⇒ VERMELHO); só
      a orla muda (o pé do alcance). ⛔ Recusada: o raio a alisar só a forma (contorno −55 % mas sem efeito
      visível no S). **Custo** (`medida_do_smooth_edges`, perfil smoke, no máximo, mínimo de 7): a tela
      4096² inteira (a recomposição ao mexer no slider) 9,5 → 82 ms na 1.ª versão (load 42) → 45 ms
      saltando as linhas longe da tinta (o desfocado ali é exatamente zero) → **31 ms** guardando o
      pigmento da região num plano (load 9,9); o retângulo de 300² (o quadro normal) 0,13 → **1,1 ms**.
    - **«Paper: None» e Relief 0 ainda mexiam no traço:** sem Kind o motor era semeado com o Cold de
      fábrica (F2), e ligar o Relief arma um papel — então None nunca era «nenhum». Rodada de leis
      (`prancha_w1g_sem_papel.png`; orla de baixo a meia · tinta da bancada · mistura):

      | sem Kind | aspeto | orla meia | tinta | molhado sangra | veredicto |
      |---|---|---|---|---|---|
      | plano a 1 | liso | 2,36 | +65 % | — | ⛔ deixa passar tudo (o Solid e o «pincel menor que a célula» mudam) |
      | plano à média (0,5) | liso | 0,98 | +1,4 % | — | ⛔ o Solid ganha 3 buracos (o limiar come os toques fracos) |
      | plano + lei ESPERADA (depósito e humidade) | liso, vêem-se as cerdas | 3,2 | −8 % | **−5** | ⛔ a água deixa de sangrar |
      | relevo do Cold só na água + lei esperada | idem | 3,2 | −8 % | −4 | ⛔ não é a capilaridade: é o limiar por célula |
      | Cold (o de antes) | as manchas do papel | 0,77 | — | +26 | o pedido do dono |
      | **as alturas do Cold ao acaso por célula** | grão fino, sem desenho | **1,45** | −1 % | **+13** | ✅ **escolha do dono** |

      Entregue: sem Kind, `seed_paper_with` com `alturas_do_papel_de_fabrica()` (64 quantis de uma janela
      512² do Cold) em `celula_ao_acaso` (misturador de 64 bits, determinístico); o Tooth não age sem papel
      (`Dependente::ToothSemPapel`, a dica «Acts when a Paper is chosen»); o «Texture on Paint» só imprime
      um papel escolhido. Portão `sem_papel_o_traco_nao_tem_o_desenho_de_um_papel` (correlação com o
      vizinho: Cold 0,96 · sem papel 0,001; o Tooth não muda um byte; tinta 1,017 da do Cold). O motor
      não mudou (só `paper::quantis_do_papel`).
      **Consequências medidas:** a mistura com a água parada continua na barra (γ 0,59·0,42, sangria seca
      +2,7, molhada +13); a orla a meia sem papel nasce a 1,45 (Rebelle 1,6) — a Heavy Edge foi calibrada
      com o Cold escolhido (0,77 → 1,56) e sem papel leva 1,45 → 2,99; o Solid deixa um bolso de 3×3 na
      FRANJA da orla fora do polígono do rabisco (21, 105) — o portão do Solid passou a contar só dentro
      do polígono (regra não-zero), que é a promessa dele. ⚠️ **Aberto:** com a Live Water e sem papel a
      sangria no seco fica em +3,4…+5 para qualquer velatura (Rebelle +1…+2; com o Cold +2,9) — a água ao
      vivo corre sobre o grão; o portão exige a barra inteira com o Cold e, sem papel, tecto +5.
      As cercas de bytes (`opcoes_tests`, o zero do Smooth Edges) e as 4 miniaturas + tabelas de largura
      de fábrica foram regravadas UMA vez por esta ordem; os presets de fábrica não fixam a Live Water nem
      a Heavy Edge.
      Testes antigos que liam o Cold escondido: `paper_visual_prints_grain…` passa a escolher o Cold; o S
      dos rectângulos corre a régua no Cold (com o controlo) E sem papel (meia 21·2·0, máxima 10·15·1 —
      verde); `the_artists_paper_drives_the_wet_tooth`: o Checker rejeita os vales a 0,602 desde sempre e
      o controlo passou de ~1,01 a 1,002 — a barra 0,6× estava na beira, ficou 0,65×. Na Aquarela
      (`watercolor_papel_nos_vales.rs`, protegido) os dois testes do Tooth do WET PAINT perdem o caso sem
      Kind — 2 linhas, a cerca ganhou `ferramentas/cerca_isencoes.tsv` (o caminho, o número EXATO, a
      razão: uma linha a mais volta a reprovar). O censo dos controlos: o Texture on Paint sem papel
      esmaece com o Tooth (`Dependente::SemPapel`); saiu a exceção «escolher o Cold a partir de None não
      muda a tinta» (agora muda).
  - **3.º e 4.º smoke do dono (10/10):** o preview demorava até 10 s — o 1.º quadro sai em 0,01–0,05 s e
    o traço em 0,2–0,4 s; era a apresentação a esperar a água parar num arrasto (agora mostra o mais
    recente; `o_preview_nao_espera_a_agua_parar`). O Smooth Edges («maravilhoso») documentado em
    [14](14_smooth_edges_a_borda_suave.md), e estendido às bordas SOBRE tinta (o degrau de cor dentro da
    mancha — a lei e a rodada dos limiares no 14 §2–§3). 5.º smoke: as linhas escuras da Heavy Edge por
    dentro da tinta — o detetor de LINHA COMPRIDA (14 §2 passo 5).
  - **Smoke do dono sobre `711513d42`: OK** — e um defeito novo para a PRÓXIMA janela: a mistura de pigmentos
    (amarelo sobre azul seco dá PRETO e nunca cobre; [captura](fixtures/smoke_dono_w1g/amarelo_sobre_azul_seco.png)).
    Prompt: [15](15_PROMPT_janela_auditoria_mistura.md) — auditada e curada na entrada «Mistura» abaixo.
