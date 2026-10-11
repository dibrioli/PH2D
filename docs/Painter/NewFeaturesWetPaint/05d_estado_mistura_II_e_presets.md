# 05 §4b — Estado, pedaço 4 de 4: a Mistura II, os presets de todas as mídias, a cor real e o relevo (11/10)

> Pedaço do §4b de [`05_plano_da_linha_e_o_antes.md`](05_plano_da_linha_e_o_antes.md) (o índice), cortado em
> 2026-10-11 por `scripts/doc-split.py`: corpo **verbatim** (a remontagem bateu sha256 com o original;
> 1 alvo(s) de link reancorado(s) para o MESMO ficheiro). É o pedaço mais recente: entrada nova do §4b entra AQUI (ou num 05e quando este passar de 40 KB).
>
> ⛔ O que aqui estiver marcado **«medido e REJEITADO»** continua rejeitado.

---

- ✅ **Mistura II — a representação da cor (11/10, prompt [16](16_PROMPT_janela_auditoria_mistura_simplificacao.md), Fases A–F; commit `d8a488185`).**
  Uma entrada só para a auditoria inteira (fundiu a «Mistura — Wet Drip: o azul-claro com as duas caixas K–M», Fases B–E,
  que a implementação escrevera à parte). A pergunta do prompt: o defeito do Wet Drip (o azul-claro com as duas caixas K–M)
  nasce da MISTURA ou da CONVERSÃO entre as representações da cor? Resposta medida: da conversão (a cor de ecrã invertida e guardada como cor de pigmento),
  nunca da K–M. Pranchas: [1:1](prancha_mistura_wet_drip.png) · [8×](prancha_mistura_wet_drip_8x.png) (as quatro capturas do
  dono · antes · depois · Rebelle; gerador [`ferramentas/prancha_mistura_ii.py`](ferramentas/prancha_mistura_ii.py));
  fixtures [`ph2d_mistura_wet_drip/`](fixtures/ph2d_mistura_wet_drip/README.md) · [`rebelle_mistura_wet_drip/`](fixtures/rebelle_mistura_wet_drip/README.md).
  - **O defeito, as 4 capturas do dono** ([`smoke_dono_mistura_wet_drip/`](fixtures/smoke_dono_mistura_wet_drip)): (1) pontas dos fios
    ciano · (2) faixa ciano na orla de baixo · (3) fios ciano pixelados · (4) halo claro na fronteira azul/vermelho;
    azul escuro (20,50,110) em Wet Drip sobre vermelho, as duas caixas K–M ligadas. **Reproduz-se** (`sonda_mistura_wet_drip`,
    5 serpentinas, razão 1): índice fora do cone **9 001** (máx 0,56) contra 0 sem caixas; pontilhado (> 25 níveis) **2 778**
    contra 76; «mais clara à mesma massa» 21 938 contra 67; seco, fora do cone 5 844. **O Rebelle (oráculo corrido, 4 telas)
    não tem nenhum dos quatro:** tetraedro {branco, vermelho, azul, vermelho×azul} 0 %, ciano 0; as capturas do dono, medidas
    na mesma régua, dão 10–19 % fora, quase tudo ciano (o controlo da régua).
  - **Mapa de conversões (Fase A, leitura; as linhas com ⚠ foram confirmadas pela Fase C):**

    | sítio | representação lida → escrita | onde divide / prende | caixa |
    |---|---|---|---|
    | `colorops::km_weighted_mean_color` | sRGB straight → K/S por canal → sRGB (média por massa) | divide por `Σp`; **sem clamp** (monótona por canal: nunca sai da caixa das entradas) | `km_mixing` |
    | `solver/advect{,_jacobi}` | `susp_rgb` dos 4 cantos → REPLACE da cor residente | `1/want` só se `want ≥ 1e-5` (série) / `> 0` (Jacobi) | `km_mixing` |
    | `solver::diffusion_pass` · `sedimenta` | straight, peso de massa | `film > 0,1` · `a ≤ 0,45` | `km_mixing` |
    | backrun (`solver.rs`, `flow_jacobi`) | `sett_rgb` (**cor de ECRÃ**) misturada com `susp_rgb` (straight) | `lift = 0,1·sett` | `p.mix` |
    | `trail/transfer` (pouso) | straight, peso de COBERTURA (`a/(e9+a)`), massa somada linear | soft cap 3000 | `km_mixing` |
    | `assenta::cor` (ecrã) | straight → luz linear (Camada) / lerp + velatura (Cobre) | Camada **ignora a cobertura do assentado** ⚠ | lei da sessão |
    | `assenta::assenta` Camada | ecrã-alvo → cor de pigmento, inversa fechada | `den ≤ 1e-9`; `clamp(0,1)` ⚠ | `km_glaze` |
    | `assenta::levanta` Camada | idem, raiz da quadrática | `disc.max(0)`; `u` presa em [0,1] | `km_glaze` |
    | `assenta::resolve` (Cobre) | Newton numérico sobre a lei | passo ±48, 4 iterações, `|d| < 0,1` fica o palpite; `x` presa a [0,255] | Cobre |
    | `drying::dry_cell` · `canvas_ops::dry_canvas`/`settle_composite` | massa → `assenta`, zera `susp`/`film` | `dm` 0,3..1,7 | lei |
    | `render::render_region` | straight + grão ±40 por camada → over premultiplicado | `clamp_u8` por canal | lei |
    | `composite` · `smooth_edges` | u8 straight ↔ premultiplicado | divide por `pa`; cast saturante | Smooth Edges |
    | pickup do pincel (`transfer.rs:88`) | `sett_rgb` + `susp_rgb` por `alpha` | não lê a lei do ecrã | `pickup_assentado` |

    Leitura da Fase A: nenhuma média K–M sai da caixa; a cor fora do casco só pode vir de um prender 0/255 ou de uma cor
    de ecrã escrita como cor de pigmento. Foi o que a Fase C isolou.
  - **Tabela H (Fase C, `bateria_c1`: 39 variantes × 3 cenas num processo; controlo: índice 0,077, pontilhado 25):**

    | H | hipótese | medida (fora · àM · pontilhado) | veredicto |
    |---|---|---|---|
    | H1 | o `molha`/Smooth Edges clareia | Smooth Edges 0: 9 001 · 21 938 · 2 778 (igual ao bit; o Wet Drip já o tem a 0); render Cobre sobre armazenamento Camada: 9 179 | ⛔ a lei do ecrã não é a causa: a cor GUARDADA é |
    | H2 | a inversa da Camada ao assentar escreve cor fora do casco | sem a inversa (`CAMADA_FISICO`): 432 · 0 · 236; armazenamento Porto ou Cobre com render Camada: 1 426 · 0 · 49 / 535 · 1 026 · 52 | ✅ **a causa** |
    | H3 | a K–M sobre cores | `km_mixing` desligado: 9 100 · 22 120 · 2 787 (= só Glaze) | ⛔ |
    | H4 | Heavy Edge · sedimenta | borda 0: 8 974 · 22 028 · 2 727; sedimenta sem K–M: 9 001 | ⛔ |
    | H5 | backrun | `ExtBackrun` 0: igual ao bit (o preset já tem 0) | ⛔ não actua |
    | H6 | a razão da grelha | razão 3: 17 764 (blocos 3×3); com a cura 471 | ⛔ só aumenta o grão |
    | H7 | o pincel sujo | `Pickup` 0 + `pickup_assentado` 0: igual | ⛔ |

    Mecanismo (as linhas do HEAD de então, a lei `Ecra::Camada` do «Glaze layering»): `cor()` mostra o assentado como fundo OPACO qualquer que seja a cobertura dele (`a_s`
    não entra; salto em `a_s = 0`); sob filme grosso (`a_f2 ≥ 0,96`) a inversa fechada pede um fundo de reflectância > 1 (no traço,
    linear `[0,24 · 1,07 · 2,95]`) e o `clamp(0,1)` escreve CIANO (sob o azul) ou LILÁS (vermelho sob azul) — um assentado preso a 255,
    escondido; o escorrimento leva o filme e a mesma `cor()`, que dá a um assentado de `a_s = 0,018` o peso de um fundo inteiro,
    destapa-o (os pontos e faixas claras). **Aterra** na advecção; **provoca-o** `assenta::assenta`, ramo Camada.
  - **Decisão D (janela principal, sobre as Fases D e D′; tabelas em `faseD_tabelas.txt` da sessão).**

    | alternativa | resultado (as duas caixas) | decisão |
    |---|---|---|
    | **Densidade óptica como representação ÚNICA** (bit 64; Beer–Lambert, sem inversa) | caixas idênticas ao bit, mas W1e γ seco **1,07–1,17** (barra 0,30–0,70), camadas −0,20 / −0,07 (miolo `[9,4,0]` PRETO), seco 2× mais escuro que o Rebelle, tetraedro 52 % | ⛔ recusada |
    | Densidade «satura» (bit 128; o controlo que separa escala de ordem) | Drip limpo, Round 99→55; mas camadas 0,18 · 0,31, W1e 1,13·1,17 | ⛔ a falta de ORDEM de camadas; sem ela não há «tapar» |
    | `CAMADA_CONTINUA` (o ecrã pesa o assentado pela cobertura) | pontilhado 55/62, àM 0; mas sangria seca **+0,129**, camadas seco **0,54**, molhada **1,25×**, tetraedro seco 16 % | ⛔ recusada |
    | Média da densidade NA K–M do «Pigment mixing» (bit 256) | índice do Drip 546 → 3, tetraedro 0,053 % → 0; γ molhado 0,81 → 0,88 | ⛔ recusada pela janela: a mistura é a MESMA porta `ColorMix` nos três meios; K–M fica (limite por canal; a régua de Beer é que está enviesada) |
    | **Glaze layering = lei Cobre com a camada a tapar menos** (`TINTA_ESCONDE_EM_VELATURA` = 2, uma porta só: `Sim::camadas_transparentes`) | ver as barras | ✅ **escolhida** |
    | **Caixa das tintas no RE-MOLHAR** (`assenta::levanta`) | re-molhar fora da caixa 44,6 → 0 | ✅ |
    | Caixa das tintas também no ASSENTAR (inteira, ou só o tecto) | o cruzamento azul sobre vermelho SECO fica malva: tetraedro do Rebelle `0,000 % → 15,3 %` com o produto SEM caixas (o Rebelle 0,000 % nas quatro capturas) — a tinta clara que tapa a escura (a lei do Rebelle) guarda-se mais clara que a escura, e o tecto prende-a | ⛔ medida e recusada |

    Esconde do Glaze, mesmo processo (o salto no Dry canvas com Δ +600 de massa): 1 → W1e seco γ 0,98·1,00, camadas 0,82·0,86 ✗ (o molhado),
    salto 26,7; **2** → W1e seco γ 0,78·0,81, camadas 0,85·0,87, salto 20,8; sem a caixa (5): γ 0,44·0,44, salto 13,2; a lei de antes
    (Camada): γ 0,86 ✗, camadas seco 0,44 ✗.
  - **A cura no código (Fase E):** (1) o «Glaze layering» é a lei `Cobre` com a camada a tapar menos — `Sim::camadas_transparentes` →
    `colorops::TINTA_ESCONDE_EM_VELATURA` = **2** no lugar de `TINTA_ESCONDE` = 5, lida SÓ por `Sim::gather_params` (o ecrã e as
    transferências verticais vêem a mesma lei; `PigmentVisual::km_glaze` saiu e o clique recompõe depois do reconcile). (2) A caixa das
    tintas só no RE-MOLHAR (`assenta::levanta`): o filme que a água leva fica, por canal, entre as duas tintas. (3) `Lei::Camada` fica só
    como o CONTROLO dos portões (é a lei de antes ao byte: `cmp` dos PPM contra a Fase D).
  - **Barras, lado a lado (antes sem caixas · antes duas caixas · depois duas caixas · Rebelle):**

    | barra | antes, sem caixas | antes, duas | depois, duas | Rebelle |
    |---|---|---|---|---|
    | W1e seco γ ×2 | 0,44·0,43 | **0,86·0,86 ✗** | 0,78·0,81 (Glaze: sem barra) | 0,42–0,51 |
    | sangria molhada / seca | +0,279 / +0,017 | +0,208 / +0,007 | +0,255 / +0,008 | +0,27 / +0,02…0,05 |
    | camadas molhado · seco | 0,86 · 0,88 | 0,78 · **0,44 ✗** | 0,85 · 0,87 | 0,89 · 0,96 |
    | acumulação | 1,79 | 1,74 | 1,79 | ≥ 1,5 |
    | molhada mais forte | 1,80× | **1,27× ✗** | 1,80× | 2,06× |
    | tom ao secar (parada / Live) | 0,061 / 0,122 | 0,092 / 0,065 | 0,059 / 0,077 | 0,05 |
    | Wet Drip fora do cone (molhado \| vermelho seco) | 0 \| 0 | 9 001 \| 5 844 | 511 \| 0 | 0 |
    | pontilhado | 92 \| 53 | 3 010 \| 2 568 | 94 \| 54 | — |
    | tetraedro | 0 \| 0 % | 19,2 \| 16,8 % | 0,05 \| 0 % | 0 % |
    | mais clara à mesma massa (luz linear) | +0,048 \| +0,129 | +0,565 \| +0,633 | +0,043 \| +0,100 | — |
    | ciano | — | 0,080 \| 0,132 % | 0 \| 0 | 0 |

    Sem caixas o produto não muda de bytes no Wet Drip além do re-molhar (tetraedro 0,000 %): as barras do produto ficam as de antes.
  - **Portões com o vermelho de antes** (o «antes» = o produto de hoje com a cura desligada; o controlo reprova; em `mistura_wet_drip_tests`
    salvo onde dito): `uma_mistura_nunca_e_mais_clara_que_as_tintas` (tolerância = o produto sem caixas, +0,148; cone ≤ 0,20; +0,646 / cone
    0,557 → ≤ +0,148 / ≤ 0,138) · `sem_celulas_soltas` (miolo > 25 níveis ≤ 150 e N32 ≤ 0,13 %; 2 778 / 2 498 → 81 / 45) ·
    `assenta::tests::as_transferencias_verticais_ficam_na_caixa_das_tintas` (4 480 re-molhares, o pior do adversário por nome; 44,6 níveis → 0) ·
    `o_glaze_layering_deixa_ver_a_camada_de_baixo` (γ 0,44 → 0,78·0,81; VERDE também antes, γ 0,86: o vermelho dele é uma caixa que não chega
    à tela) · `km_flags_reach_the_sim_and_the_composite` (o adversário D′ viu-o partir: o clique no Glaze mudava 5 890 bytes e passou a 0;
    curado pondo o 2.º traço de OUTRA tinta — a mesma tinta sobre si é uma camada só, o Glaze age onde o pigmento muda).
  - **A caixa com o filme VAZIO (11/10, apanhado pelo agente do portão M7b, lido e curado na janela principal, `dac663ab7`):** a 1.ª
    redacção de `assenta::na_caixa` devolvia, com `a_f = 0`, a cor guardada do filme (`susp_rgb` de um filme sem massa — a de uma tinta que
    já saiu) em vez da do assentado que sobe: re-molhar tinta seca de duas cores herdava uma cor parada (Δ 8,41 num traço de Wet depois de um
    Dry canvas). Cura: sem tinta a ficar, a única tinta é a que vem. Portão `assenta::tests::re_molhar_tinta_seca_leva_a_cor_do_assentado`
    (assentado azul, a cor velha vermelha no vazio, os dois tapares; a 1.ª redacção devolvia o vermelho). Regravadas UMA vez, com a razão, as
    impressões do S e da bancada (`opcoes_tests`); as 4 miniaturas de fábrica não mudaram. Cercas depois: ph2d-wet-paint (50 + 99),
    `--lib wetpaint` 165, `--test it` 6, censo dos controlos 133, painter-layers 179, `cerca.sh` VERDE, fmt e clippy `-D warnings`.
  - **O portão das transferências (11/10, o que faltava para M7b):** `ph2d-wet-paint/tests/it/glaze_vertical_transfers.rs`. Cena de duas tintas
    sobrepostas (4 pares em faixas, 40 linhas de massas, uniforme em `x` para o arrasto lateral do Smear ser nulo), a lei da sessão do produto;
    a régua é o ecrã SECO do render (`sobre_branco(p.ecra.seco(), …)`) célula a célula, antes e depois de cada passe vertical. Medido:
    - **assentar** (4 passes da secagem Jacobi; Dry canvas), 1 920 células: Δ 1,4e-4 sem a caixa · 1,3e-4 com → tecto **0,01**
      (`com_o_glaze_assentar_guarda_o_ecra_seco`);
    - **re-molhar tintas diferentes NÃO preserva o ecrã, nem sem a caixa** — a caixa das tintas prende a inversa: um traço de Wet muda até
      **40,2** níveis sem o Glaze (4,13 com), de Smear **47,7** (13,84 com); sem o Glaze a caixa prende 1 853 de 1 853 células-dab no Wet (17 dabs).
      Por isso o re-molhar mede-se pela lei dele, dab a dab (`Engine::dispatch_pressure_dab`): onde a caixa NÃO prendeu, com o Glaze, 484
      células-dab no Wet (Δ 0,0095; 1 369 presas, Δ ≤ 0,34) e 476 no Smear (Δ 0,264; 1 268 presas, Δ ≤ 0,78) → tecto **1,5** (o de `assenta::tests`) e **≥ 100** livres; e o traço inteiro com tecto 8 (Wet)
      e 25 (Smear) (`com_o_glaze_re_molhar_guarda_o_ecra_seco_onde_a_caixa_das_tintas_nao_prende`).
  - **Regravado com a razão escrita:** as impressões do S e da bancada (`opcoes_tests`) e as 4 miniaturas de fábrica (o re-molhar com a caixa).
    O censo dos controlos: a pega do ângulo do carimbo no Wet Paint passou a inerte (era «viva» por arredondamento que saía pela inversa do
    re-molhar) — linha com o motivo, ao lado do Rotate.
  - **Mutações (Fase F, 13): 12 sangraram, 1 sobreviveu — e M7b passou a sangrar com o portão das transferências (11/10): 13 de 13.** M1 e M1b (armazenamento / lei inteira de volta à Camada) · M2 (só o ecrã) · M3 (esconde do Glaze = o do produto) ·
    M5 (re-molhar sem a caixa) · M6 (caixa presa a [0,255]) · M7a (ecrã à parte sem a caixa) · M8 (constante 2 → 1) · M9 (o reconcile não passa a flag) ·
    M10 (`wet_recomposite_full` sem reconciliar) · M11 (a caixa do re-molhar sempre «vem») sangram; M4 (caixa ADICIONADA ao assentar, o inverso do pedido: o assentar
    não tem caixa) sangra só na impressão digital. **M7b** (o armazenamento calculado à parte, esconde 5, com o ecrã a 2) sobreviveu à Fase F — nenhum portão
    observava o armazenamento do Glaze depois de o ecrã o ler (corridos: `ph2d-wet-paint` `--lib` e `--test it`, `ph2d-tool-painter --lib wetpaint`). **Sangra
    desde o portão das transferências (acima), uma âncora de cada vez:** M7b.1 secagem (`DryConst::new`) → `com_o_glaze_assentar_guarda_o_ecra_seco`, Δ **51,3**
    (tecto 0,01) · M7b.2 Dry canvas (`dry_canvas_now`) → o mesmo, Δ **51,3** · M7b.3 Wet (`apply_wet` → `lift_settled`) →
    `com_o_glaze_re_molhar_…_nao_prende`, **0** células-dab livres (≥ 100), Δ ≤ 1,79 nas presas, traço Δ **25,6** (tecto 8) · M7b.4 Smear (`apply_smear` →
    `lift_settled`) → o mesmo, **6** livres com Δ **2,33** (tecto 1,5), traço Δ **43,4** (tecto 25); as quatro juntas, os dois testes. Restauro por cópia + `touch`,
    `git diff` vazio nos ficheiros mutados.
    Cobertura fina: M4/M5/M6/M11 só vêem a caixa do re-molhar pela impressão digital do S/bancada (M5 e M6 também pela unidade); o tetraedro que justificou deixar o
    assentar sem caixa só existe na sonda `#[ignore]`; M2 e M10 têm um único portão cada.
  - ⛔ **Recusas MEDIDAS (não se reconstroem):** a densidade como representação única (acima) · `CAMADA_CONTINUA` (acima) · a média da densidade na K–M
    (decisão de desenho, não de medida) · **`CAMADA_FISICO`** (assentar sem inversa, palpite físico): fecha os 4 sintomas mas o pontilhado fica 3× o controlo
    (215–236 contra 76) e o assentar deixa de preservar o ecrã, contra «Assentar não muda o que se vê» · **`CAMADA_GAMA`** (inversa presa por canal à caixa das
    entradas): 1 e 3 somem, ficam 2 e 4 (lilás; fronteira 255/123) · **`CAMADA_RESOLVE`** (Newton com guarda sobre a lei de antes): àM 16 926 · **a outra raiz da quadrática**
    ao re-molhar: a tinta quase desaparece (a raiz pequena é a certa) · caixa nas duas transferências, ou só o tecto, no assentar (tetraedro 15,3–15,5 %).
  - **Aberto:** (a) o estado pior do adversário — filme lilás `[231,186,222]` ao re-molhar azul escuro sob quase branco — está dentro da caixa por canal e fora do
    casco subtractivo (+0,135); só a cena 13 (Wet Wash quase branco sobre azul seco) o mostra, a 0,05 % dos pixels a 0,05 e 0 a 0,10 (sem caixas faz o mesmo); pede um portão de unidade
    sobre `assenta`/`levanta` que reprova hoje a 0,10 · (b) a inversa Cobre com tintas diferentes muda o ecrã seco > 1,5 níveis em 36 de 2 560 estados (até 16,8) — a caixa no
    assentar baixava o máximo (16,78 → 8,19) mas subia para 53 estados e partia o tetraedro · (c) o salto do Glaze no Dry canvas (20,8) é maior que o do produto (13,2) ·
    (d) o `solver::advect` isolado custa +27 % com a média da densidade (recusada; o passo inteiro não mudou). **Diagnosticada (11/10), já não aberta:** a falha
    intermitente de `cargo test -p ph2d-tool-painter --lib wetpaint` (164 + 1) é `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs`, membro
    conhecido de [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md) (razão de relógios; está no `filter` da faixa isolada do nextest, que o `cargo test` não
    aplica): três corridas seguidas da suíte → 164/1 (`0,7147 ms` a 2048² contra `1,7646` a 4096², 2,47× para a barra 2×; load 0,43 → 3,92), 165/0
    (3,92 → 4,97), 165/0 (4,97 → 2,53); sozinho 3 de 3 verde a load 1,8. **BUGS_painter #47** (depósito com subtracção absoluta) NÃO é este mecanismo; a auditoria não disse nada de novo sobre ele.
- **Falta (só com ordem do dono):** o fecho da linha (`/pd-linha-fechar`, handoff).
  (O degrau 187 → 188 das gotas foi desfeito com elas: a linha não muda o schema.)

- ✅ **Presets de todas as mídias — o mapa, a W2 e o 1.º lote (11/10, prompt [17](17_PROMPT_janela_presets_todas_as_midias.md)).**
  O mapa inteiro, a régua e as lições: **[18_mapa_das_midias.md](18_mapa_das_midias.md)**.
  - **Fase A — o mapa, medido.** Os 281 presets em texto ([`fixtures/rebelle_presets_281.json`](fixtures/rebelle_presets_281.json),
    com cabeçalho) e **13 fotos novas do oráculo** nos nossos gestos (25 em [`fixtures/rebelle/`](fixtures/rebelle/README.md)); ~90 receitas
    varridas por `tests/it/sonda_midias.rs` (JSON sem recompilar) e medidas por `ferramentas/midias_regua.py` contra a foto da mesma
    mídia e o controlo (o Wet Round à mesma largura), secas. **Veredito:** hoje o Wet Paint só pinta a família **aquarela**. Caneta,
    nanquim e marcador passam as 7 métricas do traço mas **tapam no cruzamento** (nosso 1,00 · Rebelle 0,016–0,24 — a métrica nova que
    os reprovou); o nanquim entra inteiro só com a mistura de pigmentos da OBRA ligada (0,245 · 0,24). Óleo/acrílica: sem estrias
    (textura 1,0 · 5,9). Lápis, carvão, pastéis, giz e aerógrafo: sem água o depósito quase some. A guache do Rebelle nos volumes de
    fábrica é o nosso Round (cob 0,22 · 0,19, borda 0,99 · 1,00) e o Flat dele tem a forma de pontas retas que não temos ⇒ nenhum
    preset com esses nomes. A acrílica do Rebelle é a ferramenta «Oil & Acrylic» (o Express Oil é óleo de secagem rápida).
  - **Fase B — a W2, a largura prometida** (`wetpaint/largura.rs`, entra no `dab_spec.rs::authored_spec` do Wet Paint): o traço
    nasce com o raio do MOTOR cuja largura medida é o número do painel (e o espaçamento dos carimbos é o desse raio — na 1.ª versão,
    a troca no `dab_route` deixava o espaçamento do raio pedido e errava +50 % a 110 px); o anel continua no raio pedido. A lei de
    cada preset é a `width_table` MEDIDA pelo gerador: **64 raios do motor de 1,5 a 1 100**, com a água assentada (480 quadros; a
    0 quadros o Round a raio 30 lê 8 px — as tabelas antigas prometiam o dobro), a tela direita (o Drip escorria e lia 6 px a raio
    200) e acima de 512 pela lei crua × 2 (`set_wet_largura_crua`: o slider pára em `BRUSH_SIZE_MAX_PX` — o «teto de 345 px» que a
    sonda leu era ele). ⭐ **A régua é a largura a ¼ do máximo da coluna** (`sonda_gestos::largura_que_se_ve`): igual à de meia
    altura em todo traço de borda definida (as 4 fotos do Rebelle e o nosso Round/Wash/Drip: 50 · 35 · 25 · 15 % dão o mesmo), e a
    única honesta na cerda partida (o Dry Brush a meia altura lê o miolo, 32 px, e a nuvem de fios que se vê vai a 52). Sem preset,
    a lei do Wet Round (`LARGURA_DO_WET_ROUND`, gate `a_lei_de_omissao_e_a_do_wet_round`); o pincel guardado leva a do preset de onde
    veio (a ponte já não a apaga); o preview/miniatura levam-na (sem isso a miniatura do Drip mudava com a tabela embutida).
    **Portão `a_largura_e_a_prometida`** (5 presets × 2, 3,5, 7, 13, 26, 55, 120, 300): **antes VERMELHO** — tabelas antigas: Round
    raio 26 → 94 px, 120 → 343; controlo dentro: a lei crua do Round reprova. **Depois VERDE**: até 100 px ± 1 px; acima ≤ 1,8 %
    (Round 605/600, Dry 589/600, Drip 243/240). A barra: ± 2 px, ± 2 % acima de 100 px — a lei ondula de raio para raio (franja das
    cerdas): 64 e 128 pontos erram o mesmo, e cinco traços ao mesmo raio variam ±1–3 px (`sonda_ruido_da_largura`).
    ⛔ **Medido e recusado:** a troca do raio no `dab_route` (espaçamento errado) · a régua de meia altura (o Dry Brush: −3,6 % a
    +6,7 % e uma isenção nomeada, que saiu com a régua de ¼) · tabelas de 128 pontos (sem ganho, o dobro do gerador: 12 min).
    **A W2 re-rotula o número**: o tamanho 40 pede ao motor o carimbo que antes se pedia com ~70. ⇒ os portões da FÍSICA que medem
    uma lei num carimbo de tamanho fixo (mistura seco/molhado, Live Water, camadas, Wet Drip, Heavy Edge, achatamento, a célula maior
    que o pincel, o Smooth Edges no zero, o S e a bancada ao byte) passam a medi-la na lei crua (`set_wet_largura_crua(1.0)` depois
    de aplicar o preset — 21 pontos em 8 ficheiros); sem isso 9 deles reprovavam (sangria seca +0,42 contra +0,02…+0,05: a cena
    estava desenhada para o carimbo antigo). **Os 4 de fábrica re-medidos à largura MEDIDA do Rebelle** (05 §4.1; plano, 30 s):
    carga 52 · 36 · 100 · 60 (Rebelle 54 · 39 · 102 · 60 — sem recalibrar), largura 32 · 41 · 45* · 33 (32 · 41 · 34 · 32), orla
    1,06 · 1,36 · 0,23 · 1,04 (0,92 · 1,41 · 1,00 · 0,91), escorre inclinado 35 · 4 · 0 · 68 (350 · 0 · 1 · 348). *A extensão absoluta
    do Dry Brush (escuro > 25) inclui os fios soltos; a ¼ do máximo é 34. O Dry Brush continua o mais longe do Rebelle (orla 0,23,
    estrias partidas onde o dele é estriado e cheio) — é a franja com física própria, o resto da W2. A fábrica regerou-se no mesmo
    commit da lei (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` vermelho antes, como esperado).
    **Os fios com física própria NÃO entraram:** a Fase A mostrou que o óleo/acrílica reprovam por três razões (estrias, relevo,
    tinta finita) — os fios sozinhos não os põem na barra; plano no 18 §5 (bloco C).
  - **Fase C — o 1.º lote: o Wet Liner** (`Watercolor/Fluid/Wet Liner`, 5.º de fábrica): o Liner do Rebelle (duro, carregado, papel
    0,10, 28,86 px) ⇒ dureza 0,9, pigmento 100, água 0,5, Paper Gate 0,05, Edge Darkening 100, 29 px. À largura 38, seco: cobertura
    **0,716** (Rebelle 0,653 · Round 0,184), textura **3,9 %** (1,7 · 8,5); a borda (0,93 · 1,05 · 1,00) não distingue a mídia do Round
    acima do ruído do Rebelle e não conta. ⭐ **Portão `cada_preset_parece_a_sua_midia`** (`tests/it/midias_de_fabrica.rs`, a régua do
    Python portada em `regua_midia.rs`, conferida na mesma imagem: 0,713/4,07 contra 0,71/4,1) com o CONTROLO dentro: o Wet Round tem
    de estar a mais de 3× o ruído do Rebelle da foto em cada métrica usada. [Prancha 1:1](prancha_presets_midias.png)
    (nosso · Rebelle · controlo; `ferramentas/prancha_midias.sh`). 02 §6.9 com a linha nova.
  - **A Strength no traço do Wet Paint é 1** (`dab_spec.rs::authored_spec`): o carimbo só a usava para recuperar a pressão
    (`cobertura ÷ strength`), e a divisão não volta exata — com o carimbo da W2, `o_pincel_guardado_pinta_o_mesmo_traco` (o
    pincel a 0,85 contra o mesmo pincel guardado, que não leva a Strength) passou a diferir em **1 px**, 3 de 3. A 1 a cobertura
    é a pressão; quem pintava a 1 (a fábrica, todo preset) não muda ao bit. **Flakes de carga confirmadas sozinhas:**
    `the_water_runs_at_a_usable_rate_under_a_real_frame_loop` (7,5 Hz a load 41–65; sozinho 2/2) e
    `a_wet_move_costs_what_the_footprint_costs_not_what_the_canvas_costs` (sozinho 3/3 a load 11) — membros da família.
  - **HR-15:** o navegador mostrava as pastas de fábrica cruas («Watercolor · Fluid»); agora pela tabela
    (`app.painter.presets.folder.*`, `rotulo_da_pasta`), gate `cada_pasta_de_fabrica_tem_rotulo` no idioma de teste (a pasta do
    artista mostra-se como ele a chamou). O teste da ponte que dizia «4» conta a fábrica.
  - **Aberto:** (a) a ordem das mídias de fora e a decisão da tinta (18 §5 — do dono) · (b) as duas bancadas do Rebelle discordam do
    Round (tinta 54 nas gotas, cobertura 0,78 nos gestos; 18 §2) · (c) o Dry Brush longe do Rebelle (orla, estrias) — os fios ·
    (d) a W2 prometida a ±2 px até 100 px e ±2 % acima (a lei ondula; a cura é a franja determinística dos fios).

- ✅ **A cor real, o relevo de volta e o 2.º lote de mídias (11/10, a seguir ao 17; ordens do dono).**
  O dono corrigiu a avaliação do 18: *«toda sua avaliação sobre o que não é possível é falsa … basta aumentar pigment
  … em tuning havia uma seção paper e dentro dela parâmetros de relevo … é só trazer de volta»* e pediu, antes:
  *«atualmente nossos traços clareiam após se secarem. isso não é legal. melhor pintar com a tinta já em sua cor real»*.
  - **A cor real** (`ph2d_wet_paint::render`): a célula molhada desenha-se com a lei SECA (`Ecra::seco()`); a sessão
    deixa o `molhada_escurece` no neutro e `TINTA_MOLHADA_ESCURECE` fica só para os testes da lei (doc no `colorops`).
    **Substitui a escolha de 10/10** («a tinta molhada parece mais forte e clareia ao secar, como o Rebelle»). Medido
    (miolo, 2 s → 30 s): Round 99 → 55 (1,8×) **→ 57 → 48 (1,19)**, Wash 1,32, Drip 1,16. O que sobra é MASSA que se
    desloca, não ecrã: a soma do pigmento conserva-se (0,97–1,00) e o perfil molhado, mais carregado no miolo,
    alisa-se ao secar (sem a orla escura: 1,16 contra 1,19 — não é ela). Portão novo
    `a_tinta_molhada_ja_e_a_cor_que_fica` (≤ 1,35; antes 1,8–2,1) no lugar de
    `a_tinta_molhada_parece_mais_forte_e_clareia_ao_secar`; regravados, com a razão escrita: os hashes do S e da
    bancada molhados (`opcoes_tests`, 3 de 6) e do Smooth Edges no zero (`grid_ratio_tests`).
  - **O relevo de volta** (secção **Relief** do Tuning, o knob `emboss`): saíra a 09/10 com o grupo Paper (W1e F2) e
    o render só o desenhava com papel escolhido e «Paint Texture» ligada (10/10, `a5e025cb3`). O relevo é da TINTA:
    `PigmentVisual::relevo` (o produto liga-o) desenha o emboss sem papel; o grão do papel continua preso ao papel.
    Nasce a **0** (o traço de hoje não muda: `WetKnobs::DEFAULT`). «Visual grain» e «Paper visibility» são do papel
    impresso e ficam fora do Tuning (ordem de 09/10). Ids append-only (`group_relief` = cabeçalho 6, `reset_relief` =
    reset 5), `WET_TUNING_RELEVO` partilhado pelo painel e pela ferramenta; catraca de comandos `wet_tuning` 14 → 16
    (com a razão); portão `o_emboss_morde_sem_papel_e_nasce_desligado` (o slider pelo evento real).
  - **A reavaliação** (18 §3, refeita): 428 receitas em 4 rodadas (`tests/it/sonda_midias.rs` por JSON), secas, contra
    as fotos, com a régua do vale e o ruído do Rebelle: **entram óleo, acrílica, Express Oil, grafite, carvão, pastel
    seco e pastel de óleo** — pigmento até 2000, o relevo, e o grão do pincel (Noise/Voronoi no slot Grain) no lugar
    das cerdas para os secos. Ficam de fora: tinta/caneta/marcador (o cruzamento), aerógrafo (o pó fino: textura 7
    contra 23) e guache/Flat. ⛔ O grão Stucci apaga o traço (o mecanismo do #47). Lição: 18 §4.5 e a memória
    `feedback_impossible_needs_the_whole_knob_axis_and_the_code`.
  - **O 2.º lote** (`tests/it/receitas_de_fabrica.rs`, as receitas saíram do gerador por LOC; fábrica = **12**):
    `Oil/Bristle/Oil Bristle` · `Oil/Round/Acrylic Round` · `ExpressOil/Round/Express Oil` · `Pencil/Graphite/Graphite
    8B` · `Pencil/Charcoal/Charcoal` · `Pastel/Dry/Soft Pastel` · `Pastel/Oil/Oil Pastel`, o tamanho de cada um o do
    preset do Rebelle; `cada_preset_parece_a_sua_midia` com 8 mídias e a régua Rust com borda e orla. Rótulos das
    pastas novas pela tabela de texto.
  - ⚠️ **A largura dos pincéis com GRÃO depende de onde o traço cai** (`sonda_ruido_da_largura`, o traço a descer
    30 px de cada vez): Graphite a «52» lê 49 · 51 · 52 · 53 · 53, Charcoal 47 · 49 · 50 · 51 · 53 (até 9,6 %), 3,8 % a
    240; os sem grão ±1 px. O grão e o papel de fábrica têm estrutura grande. ⇒ o portão da W2 dá a esses pincéis
    **± 10 %** (regra de classe nomeada: o slot Grain armado), os outros ± 2 px / 2 %. ⛔ Medido e recusado: medir
    numa tela de ≥ 480 px (trocava um viés por outro e o Round a 4 px passava a ler 0) e o grão Voronoi no Oil
    Pastel (passava as métricas, mas a largura variava até 42 % com a posição). Aberto: um grão sem estrutura grande.

- ✅ **A onda da VELOCIDADE (11/10, prompt [19](19_PROMPT_janela_linha_rapida.md); `545254755`).** `ph2d-wet-paint` e
  `ph2d-pigment` a `opt-level = 2` em `dev` (4–7× nos portões de cena; nenhum bit mudou nos portões ao byte); a suíte
  do `ph2d-tool-painter` 377 s com 9 TIMEOUT → 174 s, 1574/1574. Os dois portões da fábrica correm as cenas de uma
  vez. O `o_tooth_morde_no_wet_paint` estava vermelho desde a W2 (a largura prometida punha o corpo do traço na
  faixa do «halo»; a bateria `--lib wetpaint` não o via) — agora mede na lei crua. ⛔ Recusados com medida: o
  gerador com fila paralela (545 → 577 s) e as mídias com o botão Dry (muda a régua). O que ficou > 10 s e as
  ideias medidas: [handoff](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_VELOCIDADE.md) §4.


- ✅ **A 2.ª onda da VELOCIDADE (11/10, prompt [20](20_PROMPT_janela_velocidade_2.md)).** ⚠️ O `paper_rows` não é carga: reprova
  100/100 no perfil `ci-test` (a rota paralela põe o texel errado em 7 403 células) e só com a crate ≥ o2 +
  `lto = "off"` + `overflow-checks` — código mau do compilador, exposto pela o2 do prompt 19; ABERTO, 1.º da próxima
  janela (o produto e o smoke não caem). R9: dez testes de cenas independentes em série passam a uma
  cena por thread, as mesmas contas (poças da Aquarela 32,9 → 7,3 s, por isenção do dono; S 19,8 → 7,6 s; chave do preview 28,6 → 7,7; sem células 19,9 → 7,7; …). Ficam
  acima de 10 s, já paralelos: camadas 44,7 · largura 24,7 · mídias 14,8 s. ⛔ Medidos: o «ponto fixo» das mídias
  (os quadros depois de secar já custam 0 ms) e menos quadros na largura (a do Wet Round anda até ao q480).
  [handoff](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-11_VELOCIDADE_2.md).
