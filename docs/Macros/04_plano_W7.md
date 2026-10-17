# Macros — plano da W7: fechar o ABERTO da W6 num bloco (2026-10-16)

> Ordem do dono (16/10): fechar o §8 do [handoff W6](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-15_W6.md)
> e os abertos do [plano da onda 1](01_plano_onda1.md) num bloco só; depois a prova de mutação e o fecho.
> Antes: [plano W6](03_plano_W6.md) §6.3–§6.4 (a régua, as tabelas, os custos).

## Ordem e porquê

3 → 2 → 4 → 5 → 6 → 1. O 3 primeiro porque as sondas e os gates seguintes gravam a água no worker; o 1
por último porque é o único que precisou de decisão do dono.

## Item 1 — pintar depois do Parar fica escondido enquanto o clipe toca

**Porquê** (conferido no código): o palco do clipe não desenha por cima da imagem, TROCA a textura da
sprite pela tela reproduzida do estado inicial (`ph2d-app-painter/src/macro_track/mod.rs` `sobe`; a troca
vai à frente do override do editor em `shells/desktop/src/render_loop/fase_extract_inputs.rs`
`fase_macros_do_painter`, e o 1.º override ganha em `sim_extract.rs`). A tinta pintada depois do Parar
não está na trilha, logo não está na tela reproduzida; não se perde (volta quando o clipe pára). Durante
a gravação o palco já cedia a sprite (`sprite_gravada`); depois do Parar deixava de ceder. Era «por
desenho» na W5 (`02_plano_W5.md:210`).

**Decisão do dono (16/10, «decida pelo melhor… sem olhar custos»):** a imagem é do artista, o clipe anima
só o que gravou. Píxel mostrado = a imagem ATUAL onde ela difere da imagem do FIM da gravação, senão o
píxel do palco — o traço ao vivo incluído. No último quadro do clipe a imagem mostrada é a atual (sem o
salto de quando o clipe acaba). Recusadas: (A) só ceder enquanto se pinta (o salto fica ao sair do modo),
(C) pintar pára o clipe, (D) aviso e fica como está.

## Item 2 — a folga da régua no Wet Paint

**A lei (W7): a MESMA idade da água.** A idade = quadros (60 Hz) desde o último `Up` na tela, contados pelos
`dt` do `on_tick` do tool (`PainterTool::segundos_desde_o_ultimo_up`; viaja no instantâneo). A `Referencia` são
duas CURVAS `(idade, fracção dos novos)`: a gravada desde o `Up` da pincelada k, a base (a trilha sem a k) desde
o `Up` da anterior que lá ficou — as duas lidas a cada evento da reprodução desde esse `Up` até ao corte (o toque
seguinte) e depois `tutorial_base.rs` `assentada` (sem eventos, relógio fixo, até a água parar ou
`QUADROS_ATE_ASSENTAR = 960`), a cada `PASSO_DA_CURVA` quadros. O aluno com idade τ cumpre ⇔
`f − base(τ) ≥ FRACAO · max(gravada(τ) − base(τ), DELTA_MINIMO)`, interpolada, τ preso a cada curva; o Δ vazio
avalia-se no fim. A chave do progresso na família leva a idade presa à última amostrada
(`EnsinoDoPainter::idade_na_chave`).

**Porque as curvas começam no `Up`** (medido): começadas no corte (idade 13), o aluno exacto no `Up` ficava a
37–69 % da régua em 9 das 30 pinceladas — a água corre muito nos 12 quadros antes do toque seguinte.

**Medição 1 — esperar a água PARAR** (parte 1, `sonda_a_fracao_por_meio`, smoke, load 5–16): o «Nada» caía de
67,5 % para 0, mas medir custava 44–59 s por medida e o aluno exacto esperava 0–321 quadros (mediana 101) depois
do `Up` (30 pinceladas, Wet Paint). ⇒ T = 960 (tabela no doc-comment de `QUADROS_ATE_ASSENTAR`) e as curvas.

**Medição 2 — a idade** (`sonda_quanto_espera_o_aluno`, `sonda_a_fracao_por_meio`, `sonda_a_fatia_fina`,
smoke, load 9–28; % da régua):

| Trilha | Exacta: quadros depois do `Up` até cumprir | Torta ½ r (mín.) | Fina (máx.) | Nada: nas curvas / depois (máx.) |
|---|---|---|---|---|
| Splash, Digital / Watercolor / Impasto 512² | 0 | 73,1 / 74,0 / 74,0 | 43,8 / 44,3 / 44,5 | 0 / 0 |
| Splash, Wet Paint 512² | 0 | 74,6 | 55,6 | 0 / 8,3 |
| Splash, Wet Paint 1024² | 0 | 80,2 | 60,0 | 0 / 4,9 |
| 30 pinceladas, Wet Paint 512² | **0** (razão 100 % no `Up`) | — | — | 0 / 7,3 |
| 30 pinceladas, fatia fina, os 4 meios | Exacta e «Mostra-me» 100 % | — | — | 0 (cumpre 0 de 28–30) |

- «Depois» = o aluno que pula e espera além da última idade amostrada (até a água dele parar): o erro da tabela
  de `QUADROS_ATE_ASSENTAR`, ≤ 9 %.
- **`PASSO_DA_CURVA = 1`**: Nada nas curvas 0,0 / 1,8 / 2,2 / 2,9 / 6,4 % com passo 1 / 4 / 8 / 16 / 32 (30
  pinceladas), e ler a cada quadro não custa nada que se meça (`sonda_o_custo_do_passo`: 29,1 × 30,0 s).
- **`FRACAO_DA_GRAVADA_QUE_CUMPRE = 0.6655`** (era 0,703): o meio de 60,0 (a fina, Wet Paint 1024²) e 73,1 (a
  torta, Digital) — 6,55 pontos de margem para cada lado; o Nada (≤ 8,3) deixou de ser o pior «falha».

**O custo** (`sonda_o_custo_das_referencias`, `sonda_o_custo_de_medir_as_trinta`, smoke, sob `ph2d-run` =
16 núcleos ⇒ `fios()` = 4): a água custa ~7 ms por quadro a 512² (a composição do dreno não pesa:
`sonda_o_dreno_da_assentada`, 7,29 × 7,27 s por 960 quadros), e cada medida são 960 quadros.

| Trilha | `mede`, 4 fios: 1.ª medida / total | `mede`, 1 fio: 1.ª / total | load |
|---|---|---|---|
| Splash 512² | 8,8 s / 30,1 s | 15,9 s / 90,6 s | 18 → 13 |
| Splash 1024² | 13,0 s / 27,4 s | 10,2 s / 35,4 s | 18 → 13 |
| 30 pinceladas 512² | 8,8 s / 244,7 s | — | 14 → 17 |
| 30 pinceladas 1024² | 14,3 s / 593,9 s | — | 17 → 17 |

- A 1.ª medida chega em 9–14 s; o total cresce com as pinceladas (W6d: 2,2–22,6 s sem assentar). ⚠️ No
  produto, as 30 pinceladas a 1024² ficam ~10 min «a medir» (cada pincelada cumpre-se assim que a dela chega).

- `mede` (`tutorial_mede.rs`): a reprodução no fio `macro-referencias`; cada medida (a gravada, a base) num de
  `fios()` = núcleos/4 trabalhadores, fila limitada a `fios()` (no máximo ~2× fios instantâneos vivos); as
  medidas chegam pela ordem das pinceladas assim que as duas curvas de uma acabam; um erro ou o `parar` pára
  todos; a reprodução pára depois do último corte.

**Gates** (debug, sozinhos `-j 1`, load 42 → 19; override do nextest 10 × 60 s):

| Gate | O quê |
|---|---|
| `…::o_aluno_exacto_cumpre_no_quadro_do_up` (27,3 s) | 30 pinceladas Wet Paint, medidas as k = 13, 21, 25, 29: o aluno exacto cumpre no quadro do `Up` e o balão passa. ⛔ VERMELHO pela régua da parte 1 (razão −8–8 %) e pelas curvas começadas no corte (37–62 %) |
| `…::pular_a_pincelada_e_esperar_nao_cumpre_nenhuma` (40,3 s) | medidas as k = 5, 9, 13, 18, 22, 26 (as que cumpriam na W6c): o aluno que pula espera até a idade passar as curvas, avaliado a cada quadro (o balão a cada 32) |
| `…::a_mao_torta_cumpre_o_pincel_fino_e_o_nada_nao` (33,9 s) | os quatro meios, avaliados na idade deles depois de a água assentar; os alunos em fios paralelos |
| `sem_medidas_…`, `recomecar_volta_…`, e 5 de `ph2d-app-macros --test it tutorial_painter` | passaram ao Splash **Digital**: ensaiam a chegada e o balão, não a água (cada medida em Wet Paint são 960 quadros); `uma_versao_nova_…` e `mudar_a_macro_…` ficam na água (precisam de uma medição demorada) |

**Fora (com o porquê):** partilhar a corrida da gravada da k com a base da k+1 (as duas são «a trilha até ao
toque de k+1, depois nada» — metade do custo), recusado por agora: a base reproduz a espera pela fita e a gravada
assenta no relógio fixo, e a igualdade não foi medida; a memória por instantâneo a 1024² continua no item 6.

## Item 3 — o `macro_agua` dependia do relógio (feito, `b04faab535`)

`RitmoDaAgua` (`fita.rs`): o worker corre `estagios[i]` estágios por entrega e espera o pedido; os
pedidos sem espera voltam de mãos vazias por `nao_volta[j]`. As duas cópias do roteiro com esperas de
parede viram uma (`macro_comum::grava_no_worker_e_o_tool`). Gate novo
`a_gravacao_no_worker_ritmado_e_a_mesma_duas_vezes`; a fixture tem o toque em voo e um `NaoVoltou` à primeira.

## Item 4 — as ferramentas do trilho no estado inicial (feito, `09c9f5ae8a`)

`macro_estado/trilho.rs`. Red-first (`macro_trilho`): Clone 4 380, Seleção 8 606, Máscara 2 759 texels.
Censo estendido: `o_estado_inicial_leva_as_ferramentas_do_trilho` (quatro meios × Clone/Máscara/Seleção,
a preparação e cada controlo próprio). Fora, com o porquê: o gesto em voo, o anel do Offset, a área de
transferência (copiar/colar são portas recusadas).

## Item 5 — a dica do Motion por quadro (feito, `5215c2dd3b`)

`Tutorial::dica_do_motion` por (hash dos bytes, trilha, k). Gate red-first
`a_dica_do_motion_le_se_uma_vez_por_versao`: 5 descodificações em 5 quadros → 0.

## Item 6 — os abertos da onda 1

Memória por instantâneo a 1024² com a água; o cozimento do Motion na placa na reprodução (o palco usa
`MotionState::new()` — herda `PH2D_GPU_COOK`?); as tabelas de novo numa rodada só, intercaladas, o mínimo,
com o `loadavg` ao lado.
