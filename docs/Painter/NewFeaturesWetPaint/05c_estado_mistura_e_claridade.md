# 05 §4b — Estado, pedaço 3 de 4: a mistura amarelo sobre azul, os 4 pincéis e a claridade é o pigmento (10/10)

> Pedaço do §4b de [`05_plano_da_linha_e_o_antes.md`](05_plano_da_linha_e_o_antes.md) (o índice), cortado em
> 2026-10-11 por `scripts/doc-split.py`: corpo **verbatim** (a remontagem bateu sha256 com o original;
> 2 alvo(s) de link reancorado(s) para o MESMO ficheiro). Continua em [`05d_estado_mistura_II_e_presets.md`](05d_estado_mistura_II_e_presets.md).
>
> ⛔ O que aqui estiver marcado **«medido e REJEITADO»** continua rejeitado.

---

- 🔎✅ **Mistura — amarelo sobre azul seco (10/10, prompt [15](15_PROMPT_janela_auditoria_mistura.md)).**
  Sonda `wetpaint/mistura_camadas_tests.rs::sonda_camadas` (a cena da bancada: referência amarela no branco,
  mancha azul (30, 100, 220) de 9 traços + Dry canvas, 10 passadas de amarelo (245, 205, 25) em serpentina,
  molhado = 0,5 s entre passadas, seco = Dry canvas entre passadas; miolo x 285–315, y 185–215; «escuro» = 2.º
  percentil da luminância em x 240–360, y 155–245; «→amarelo» = fracção azul → tinta em luz linear, R e G).
  Oráculo: Rebelle 8.3.4 pela porta WebSocket, a MESMA cena (+200, +120): `ferramentas/camadas.py`,
  `camadas_tela.sh`, `camadas_regua.py`; fotos em [fixtures/rebelle_camadas/](fixtures/rebelle_camadas).
  - **O que o Rebelle faz:** nunca escurece; molhado o miolo vai (201,203,134) → (235,206,56) à 3.ª → a cor
    da tinta (245,205,25) à 9.ª; seco (camada a camada) (214,217,169) → (244,225,127) à 3.ª → creme
    (249,226,120) estável desde a 5.ª. O amarelo COBRE o azul por volta da 3.ª passada nos dois modos.
  - **A causa do PRETO é a «Pigment mixing (K-M)» do Tuning ligada.** O preto da captura é (14,14,14) — o
    chão de reflectância da K–M, que a sonda alcança — e os fios verdes (41,180,12) são a cor da K–M. Com a
    mistura de fábrica NENHUMA das 8 condições (tela direita/meia/máxima, Heavy Edge 1, Live Water, esfregar a
    60 px, tudo junto) põe o miolo ou o 2.º percentil abaixo do azul. Com a K–M + a cena do dono (Live
    Water, Tilt meia, Heavy Edge 1, Smooth Edges 1, 60 px, 3 idas por traço) o assentado do miolo vai
    (12,98,43) → (12,80,18) → (12,12,13) → **(2,3,12)** — e o próprio traço amarelo no BRANCO acaba (86,122,19).

    | hipótese | medida (molhado salvo dito) | veredicto |
    |---|---|---|
    | **H1** a velatura compõe-se | K–M, cena do dono: desligar a velatura SÓ na junção «assenta» (drying) → (237,203,25); só no pouso, só no re-molha ou só no render → o mesmo preto. De fábrica nunca desce abaixo do azul | ✅ **sob a K–M**: a K–M já é subtractiva, a velatura por cima conta a absorção duas vezes e o ciclo assenta ↔ re-molha realimenta-a |
    | **H2** sRGB sem tecto | a velatura com tecto no mínimo dos dois por canal = o produto ±2 níveis; a média geométrica em luz linear também | ⛔ não é a causa |
    | **H3** o pincel sujo | de fábrica, simples: o suspenso (189,177,75) contra a tinta (245,205,25); com o pincel a apanhar só o MOLHADO (244,205,25), o miolo à 10.ª 0,63 → **0,81**; cena do dono 1.ª passada 0,53 → **0,79** | ✅ **é o «nunca cobre»**: o bico apanhava o azul ASSENTADO (peso `0,5·α(sett)`) a cada dab |
    | **H4** a cobertura | a massa azul do miolo 1984 → 1707 em 10 passadas (a re-molha tira 14 %); sem re-molha o miolo de fábrica sobe só +7 níveis; com a K–M é decisivo (seco final 0,35 → 0,98 com Staining 1) | ✅ residual, só sob a K–M — é a lei do **Staining** (doc 23: «quão permanente é a tinta seca», 0,5 de fábrica) |
    | **H5** o render | sem a velatura do render o ecrã molhado sobe (157 → 182 no 1.º) mas o ASSENTADO preto é o mesmo | ⛔ o preto é pigmento, não render |

  - **A cura (duas leis, as duas no produto; o port e a impressão digital não mudam):**
    1. **Com a K–M não há velatura** — `WetSession::reconcile_facts` troca `sim.glaze` com a caixa
       (`VELATURA_MEDIDA` ↔ 0), como a W1g troca a do filme com a Live Water. A velatura do render fica (não
       se guarda, não realimenta).
    2. **O pincel sujo só apanha tinta MOLHADA** — `Sim::pickup_assentado` (o peso do assentado no que o
       bico recolhe, `trail/transfer.rs::tip_rows`): port 0,5 (ao bit: a mesma multiplicação), produto 0.
       O Rebelle já o dizia na W1e (rasto − azul ≈ 0 sobre o seco).

    | miolo, passadas 1 · 3 · 10 · seco (→amarelo) | antes | depois | Rebelle |
    |---|---|---|---|
    | de fábrica, molhado | (157,157,79) · (173,165,62) · (199,177,44) · (180,166,53) — 0,39 · 0,48 · 0,63 · 0,50 | (197,177,32) · (206,181,26) · (224,190,27) · (205,178,36) — 0,62 · 0,68 · **0,81** · 0,66 | (201,203,134) · (235,206,56) · (245,205,25) · (245,205,26) |
    | de fábrica, cena do dono | 0,53 · 0,82 · 0,96 · 0,97 | **0,79 · 0,99 · 1,00 · 1,00** | idem |
    | de fábrica, seco | (128,138,73) · (229,192,25) · (245,205,25) — 0,24 · 0,84 · 1,00 | (149,148,28) · (244,203,24) · (245,205,25) — 0,33 · **0,98** · 1,00 | (214,217,169) · (244,225,127) · (249,226,120) |
    | K–M, cena do dono | (98,151,35) · (102,140,20) · (99,92,17) · **(2,3,12)** | (161,182,21) · (223,200,23) · (243,205,25) · (244,205,25) — 0,54 · 0,87 · **0,99** · 0,99 | — |
    | K–M, molhado simples | (148,158,75) · (97,162,40) · (57,153,22) · (30,134,23) — escurece a cada passada | (175,180,26) · (131,182,23) · (133,185,24) · (94,173,27) — 0,57 · 0,47 · 0,49 · 0,35, **nunca abaixo do azul** | — |
    | K–M, seco | (63,135,64) · (180,181,21) · (227,202,24), orla 2.º percentil 0,10 < azul | (138,192,25) · (243,205,25) · (245,205,25) — 0,54 · 0,99 · 1,00 | — |

  - **Portão** `n_camadas_de_amarelo_sobre_azul_seco_convergem_sem_preto` (6 cenas: de fábrica e K–M ×
    molhado simples · cena do dono · seco). Barra do Rebelle: (a) o miolo nunca abaixo da luminância do azul
    e o 2.º percentil nunca abaixo de 0,8× dela; (b) à 10.ª passada `≥ 0,70` do caminho para o amarelo e
    não recuou da 1.ª. **Nasceu VERMELHO** no produto de hoje («de fábrica, molhado: 0,63»; «K–M, cena do
    dono: (2,3,12)»; «K–M, seco: 2.º percentil 0,10»). Controlos DENTRO: a velatura por cima da K–M na
    cena do dono e o pincel que apanha o assentado (de fábrica, molhado) reprovam. Mutações no produto
    (restauro por cópia + `touch`): «a K–M volta a levar a velatura» (`reconcile_facts`) ⇒ VERMELHO (K–M,
    cena do dono, miolo (104,103,17) à 3.ª); «o pincel volta a apanhar o assentado» (`pickup_assentado`
    0,5 no nascimento da sessão) ⇒ VERMELHO (de fábrica, molhado: 0,63); restaurado ⇒ verde.
    ⚠️ Custa ~86 s no perfil smoke (9 cenas de 600×400 a 10 passadas) — o portão mais caro da família.
  - **Cercas, verdes depois:** `cerca.sh` · `os_outros_meios_nao_mudam` + miniaturas de fábrica · a suíte de
    `ph2d-wet-paint` (as impressões do port) · o censo dos controlos (4 features) · a ponte do preview
    (`--features panel-painter-layers`) · `--lib wetpaint` inteiro (158, com `opcoes_tests` — os bytes do
    S e da bancada NÃO mudaram: num traço de uma cor o bico apanha a própria cor, e as miniaturas idem) ·
    `mistura_tests` (a barra de uma camada: γ seco 0,59·0,42, sangria +2,7 · molhado +13 — igual com e sem
    as curas, medido no mesmo processo). «Antes»: as cercas fecharam verdes na W1g (`711513d42`) e o rebase
    sobre `782f90b69` não tocou no Painter; não as re-corri antes da cura.
  - ⛔ **Recusas medidas:** o tecto da velatura no mínimo dos dois por canal (= produto, não cura nada) · a
    velatura pela média geométrica em luz linear (100 % e 200 %: = produto de fábrica) · zerar a re-molha no
    produto (de fábrica +7 níveis; é a lei do Staining, o controlo do artista desde o doc 23, e mexer-lhe
    move os lavados molhados) · desligar a velatura de fábrica (o seco da W1e volta a 0,06).
  - **Aberto — decisão do dono:** com a K–M ligada, a tinta MOLHADA sobre o azul seco re-dissolve ~14 % dele e
    a K–M (um pouco de azul domina o amarelo) segura o miolo verde-amarelado (0,35–0,49, nunca abaixo do azul);
    com a Live Water ou camada a camada cobre (0,99). **Staining 1** faz cobrir também ali (0,98). Subir o
    Staining de fábrica quando a K–M liga seria uma lei nova — não pedida, não construída.
  - Pranchas: [1:1](prancha_mistura_camadas.png) · [8× no miolo](prancha_mistura_camadas_8x.png) (Rebelle ·
    de fábrica antes · depois · K–M cena do dono antes · depois; colunas azul · 1 · 2 · 3 · 5 · 10 · seco).
    Gerador `ferramentas/prancha_camadas.py`.
- 🔎 **Conferência dos 4 pincéis de fábrica (dono 10/10: «o Wet Wash é seco mesmo? o nome confere?»).**
  A bancada de `gotas.py` (4 traços de 150 px a 40 px) dos dois lados: Rebelle `--sem_motor` (Round · Wash
  Round · Flat Dry · Round água 100; [fixtures/rebelle_presets/](fixtures/rebelle_presets)) e a sonda
  `wetpaint/sonda_presets.rs` (Wet Round · Wet Wash · Wet Dry Brush · Wet Drip, cada um numa tela sua, 30 s);
  régua `ferramentas/presets_regua.py` (tinta = mediana de `255 − min(rgb)` no centro; orla = borda ÷ centro);
  [prancha](prancha_presets_conferencia.png).

  | plano, 30 s (inclinado: escorre) | tinta | largura (40) | orla | papel a aparecer | escorre inclinado |
  |---|---|---|---|---|---|
  | Wet Round · Rebelle Round | 210 · 54 | 25 · 32 | 0,87 · 0,92 | 0 · 0,01 | 12 · 340 |
  | **Wet Wash · Rebelle Wash Round** | **214 · 39** | 30 · 41 | **0,90 · 1,41** | 0 · 0 | **115 · 0** |
  | Wet Dry Brush · Rebelle Flat Dry | 160 · 102 | 19 · 34 | 0,64 · 1,00 | **0,20 · 0** (estrias) | 1 · 1 |
  | Wet Drip (o Tilt dele) · Rebelle Round água 100 | 210 · 60 | 30 · 32 | 0,92 · 0,91 | 0 · 0 | 70 · 344 |

  ⇒ **O nome não confere no Wet Wash:** a aguada do Rebelle é a MAIS clara, lisa, a mais larga, com a orla
  fina e escura ao secar, e não escorre (água 24); a nossa carrega tanta tinta como o Round e é a que MAIS
  escorre. Round, Dry Brush e Drip conferem com o nome (o Dry Brush é fios partidos onde o do Rebelle é
  estriado e cheio; os fios do Drip são grossos e curtos — o «fio único» aberto da W1e). ⚠️ Os quatro
  carregam 3–4× a tinta do Rebelle e saem 25–30 px a 40 (a largura prometida, W2). Dono: «aproxime do
  Rebelle e ajuste tudo que for necessário» ⇒ a entrada seguinte.
- ⛔ **SUBSTITUÍDA pela entrada «A claridade é o PIGMENTO» (o dono recusou a diluição, 10/10)** — fica como
  registo. **A claridade do Rebelle — a Opacity do Wet Paint (10/10).** Medido nas fotos do oráculo: o traço claro do
  Rebelle é a cor misturada com BRANCO numa fracção constante nos três canais (Round azul 0,667 · 0,671 ·
  0,657; amarelo 0,70 · 0,72 · 0,72; vermelho da W1e 0,69 · 0,69) — tinta DILUÍDA que cobre, não pouca tinta.
  - ⛔ **Recusa medida — clarear tirando pigmento** (Round 800 → 80, Wash 400 → 28…): dá a mesma carga num
    traço sozinho, mas a tinta fica transparente e as camadas morrem — o cruzamento azul/vermelho ia a γ
    0,95–0,97 MESMO com a velatura a 0 (Rebelle 0,42–0,51) e o amarelo sobre o azul ficava mais escuro que o
    azul (lum 0,75–0,82 dele).
  - **A lei:** a **Opacity** (volume do preset, `0..1`) é do Wet Paint (a força do pincel é partilhada entre
    os meios — `a_opacidade_nao_tem_leitor_no_wet_paint` continua verde): `dab_route` deposita
    `255 − op·(255 − cor)`; `1` = a cor ao bit. Nasce 0,26 (`OPACIDADE_DE_FABRICA`, a do Wet Round). Painel:
    a linha **Opacity** sob o Water, com valores rápidos (`PAINTER_WETPAINT_VOL_OPACITY`, `VOL_QUICK[2]`).
  - **Os pincéis** (varredura `sonda_presets_varre`, mesma régua dos dois lados; tabela no 02 §6.9):

    | 30 s, tela direita | carga · largura (40) · orla · papel | inclinado: escorre | Rebelle (carga · orla · escorre) |
    |---|---|---|---|
    | Wet Round (Opacity 0,26) | 55 · 24 · 0,96 · 0 | 13 | 54 · 0,92 · 350 |
    | Wet Wash (0,20, pigmento 200, água 0,4) | 39 · 25 · 1,03 · 0 | 2 | 39 · 1,41 · 0 |
    | Wet Dry Brush (0,50, Paper Gate 0,4) | 105 · 22 · 0,42 · 0,13 | 1 | 102 · 1,00 · 1 |
    | Wet Drip (0,28, Tilt máx.) | 59 · 29 · 0,96 · 0 | 71 | 60 · 0,91 · 348 |

    ⚠️ A régua do escorrimento passou a contar a partir de 30 % da tinta do próprio traço (mínimo 6): com os
    25 absolutos ela lia 0 no Wash diluído que a foto mostrava a escorrer (124 px com a água 1,0).
  - **A tinta diluída mudou três leis, re-medidas no mesmo bloco (§0.10):**
    1. **A diferença de pigmento da velatura é o ÂNGULO entre os perfis de absorção**
       (`colorops::diferenca_de_pigmento`): a distância RGB encolhia com a diluição (o rosa e o azul-claro
       ficam «perto») e o cruzamento caía a γ 0,20. Mutação «volta à distância RGB» ⇒ a barra W1e VERMELHA.
    2. **As velaturas re-medidas** (grelha `sonda_velatura_recalibra`, as DUAS barras juntas — a W1e e as
       camadas; tabela no doc de `VELATURA_MEDIDA`): `VELATURA_MEDIDA` 0,47 → **1,0**, `VELATURA_DO_FILME`
       0,25 → **0,40**, `VELATURA_DO_FILME_VIVA` 0,35 → **0,50**. Água parada: seco γ 0,41 · 0,36, molhado
       0,48; camadas 0,71 molhado · 1,00 seco. Com a Live Water e o Cold: seco 0,43 · 0,46, molhado 0,41,
       camadas 0,85. ⚠️ Margens justas (0,71 contra 0,70; a sangria seca no tecto 0,017).
    3. **As réguas passaram a não depender da diluição:** a sangria da W1e em FRACÇÃO do caminho (as barras
       antigas, +8 e ±3 níveis sobre o caminho de 180 da tinta cheia, são 0,044 e 0,017 — o mesmo rigor); as
       camadas pelo canal VERMELHO contra o amarelo como sai sozinho (o verde junto confundia cobrir com
       concentrar: dava ao PRÓPRIO Rebelle 0,46 à 10.ª; pelo vermelho ele dá 0,30 · 0,75 · 0,89, seco 0,96).
  - **Testes que mediam outra coisa** passam a pintar com a cor cheia (Opacity 1, escrito em cada um): o
    `tool_in_mode` e o `wet_tool` dos gates de grade/encaminhamento, a identidade do Blend, o Smooth Edges
    sobre tinta (o caso do 4.º smoke do dono era tinta cheia). Regravadas UMA vez, com a razão: as impressões
    do S e da bancada (`opcoes_tests`) e as 4 miniaturas.
  - **Portões novos:** `a_opacity_do_wet_paint_dilui_a_cor_para_o_papel` (a 0,26 ~¼ do escuro da cheia;
    mutação «o pincel ignora a Opacity» ⇒ VERMELHO) · `a_opacity_e_um_volume_do_wet_paint` (painel: o campo e o
    valor rápido chegam ao pincel e a força partilhada não mexe) · `os_volumes_sao_o_que_o_painel_mostra`
    (mutação «o preset põe a Opacity na força» ⇒ VERMELHO). Custo das leis novas (assentar, a 4096², três
    cores, mínimo de 7 rondas, `measure_what_the_vertical_transfer_law_costs`): Porto 0,666 · PorCima 0,650 ·
    Camada 0,712 ms — desprezável.
  - **Aberto (medido, para o dono decidir a ordem):** (a) a **sangria no molhado** anda 0,06–0,10 do caminho,
    o Rebelle **0,27** — sempre foi ~3× mais fraca (a tinta cheia escondia-o nos níveis absolutos); é
    dinâmica da água, calibrada nas ondas das gotas e dos fios, e a difusão do motor move MASSA e não cor
    entre tintas de carga igual. (b) A **largura**: 22–29 px a 40 (Rebelle 32–41) — a W2. (c) O **Wash** não
    tem a linha escura fina ao secar (orla 1,03; Rebelle 1,41). (d) Os fios do Round/Drip inclinados: grossos
    e curtos (13 · 71 px; Rebelle 350, finos) — o «fio único» da W1e. (e) Com a tinta diluída o **Smooth
    Edges** amacia menos o degrau entre duas tintas claras (−18 %; os limiares são de contraste absoluto).
  - Prancha: [conferência calibrada](prancha_presets_calibrados.png) (Rebelle · nós, tela direita e inclinada).
  - **Smoke do dono (10/10): «o preview ficou mais claro, os traços na pintura não».** Duas causas, a família
    «copiar um componente deixa os irmãos» outra vez: (1) a `CenaDoPreview`/`ChaveDoPreview` não levavam a
    Opacity — o preview pintava a de fábrica; (2) o `estado.json` do dono lembrava, por preset, `"opacity":
    1.0` gravado ANTES desta mudança, quando o campo era a força partilhada (sem leitor); a política «manter
    volumes» punha-o por cima do preset como a Opacity nova ⇒ a cor cheia na tela. Curas: a Opacity na cena
    e na chave do preview (caso novo em `a_chave_muda_com_cada_parametro_e_o_traco_tambem`: «opacity: a chave
    não viu» antes da cura); o `estado.json` ganha `versao` (`VERSAO_DO_ESTADO` = 1; um estado novo nasce
    nela) e um ficheiro sem ela perde o `opacity` lembrado ao ler (fica a do preset) —
    `um_estado_antigo_nao_traz_a_forca_como_opacity`, mutação «sem a migração» ⇒ VERMELHO. O ficheiro do
    dono não foi tocado: migra-se no próximo arranque.
- ✅ **Assentar não muda o que se vê (dono 10/10: «Pigment mixing + Glaze layering pintando sobre tinta seca
  ainda tem uma aparência em 2 etapas — isso não pode acontecer»).** Sonda e portão em
  `wetpaint/mistura_camadas_tests.rs` (`duas_etapas`: uma passada de amarelo sobre o azul seco, o miolo com o
  rato em baixo contra +1 · +30 · +120 · +600 ticks e o Dry canvas).
  - **Causa medida:** o ecrã mostra o filme molhado sobre o assentado por UMA lei (o `lerp` + a velatura do
    filme, ou a camada K–M com o «Glaze layering»), e o motor, ao assentar e ao re-molhar, recalculava a cor
    por OUTRA (a junção mistura + velatura, ou a K–M com o «Pigment mixing»). Pior Δ (níveis sRGB), antes:
    fábrica 29–35 · K–M 81 · Glaze 28 · as duas **126** (o miolo (230,194,45) → (106,180,26)); com a Live
    Water 45 · 49 · 37 · 103. O «Glaze layering» nem chegava ao motor (só ao ecrã).
  - **Cura** (`crates/ph2d-wet-paint/src/assenta.rs`): as transferências VERTICAIS (assentar, Dry canvas,
    re-molhar — o mesmo ponto, a mesma coluna de tinta) resolvem a cor nova pela inversa da lei do ecrã
    (`assenta::cor`, que é agora a porta do render: ⛔ uma só). Palpite físico (a parte que assenta é uma
    camada do filme por baixo do filme que fica — exacto para o `lerp`), afinado por Newton com derivada
    numérica onde o ecrã vê a cor (sensibilidade ≥ 0,1); a camada K–M por fórmula fechada (assentar) e pela
    raiz da quadrática (re-molhar). `Sim::lei` (port `Porto` ao bit; produto `PorCima`, `Camada` com a caixa,
    levada pelo reconcile como facto novo `km_glaze`). A mistura fica onde acontece: na água (o pouso, a
    advecção). ⛔ Duas armadilhas medidas: assumir a derivada `1 − a` dava passos 4× grandes e o miolo ia a
    PRETO; corrigir onde o filme grosso esconde o assentado escrevia uma cor ESCONDIDA que aparecia quando o
    filme afinava (+24 de verde ao secar) — por isso o limiar.
  - **Depois:** com a Live Water ≤ 7 nas 4 combinações (fábrica 6 · K–M 6 · Glaze 5 · as duas 5); sem ela a
    água só nivela depois do Up (a escolha da W1f para a 4K) e todas mudam ~20 nos primeiros 0,5 s — a
    mesma em todas, não é a mistura. A K–M deixa de mudar o que quer que seja ao secar (linhas iguais).
  - **A velatura do filme passou a valer também depois** (o seco fica com a cor do ecrã): `VELATURA_DO_FILME`
    0,20 → **0,25** (seco 0,62 · 0,50, sangria +2,9; a 0,20 a sangria seca dava +3,1 ✗; tabela no doc da
    constante). Live Water 0,35 continua (com o Cold verde; sem papel +4,7, tecto +5).
  - **Portão** `o_traco_nao_muda_de_tom_ao_secar` (`mistura_camadas_tests.rs`; chamou-se
    `…_de_aparencia_ao_secar` até à auditoria de 11/10, que o achou já renomeado) (Live: ≤ 12 nas 4; sem Live, as duas da K–M ≤ 30),
    CONTROLO dentro (a lei de antes com as duas caixas reprova). O portão das camadas ganhou o controlo das
    DUAS leis de antes (cada cura sozinha já tira o preto). Unidade `assenta::tests` (≤ 1,5 níveis em 7
    estados, incluindo filme a 99 % e opaco). Cercas verdes: suíte de `ph2d-wet-paint` (impressões),
    `--lib wetpaint` 159 (bytes do S e da bancada, `mistura_tests`, Live Water), miniaturas, ponte do preview.
- ✅ **A claridade é o PIGMENTO — a lei do Rebelle (dono 10/10).** O dono recusou a diluição da entrada
  «A claridade do Rebelle»: *«os parâmetros que vc usou para clarear a cor não permitem o acúmulo de cor (a cor
  nunca vai chegar à cor escolhida no painel). o correto para clarear não seria usar pigment? … é como se vc
  tivesse mudado a cor do pincel e não a quantidade de pigmento»*; escolheu «como o Rebelle» (a tinta molhada
  parece mais forte e clareia ao secar; a camada de cima tapa; a cor acumula até à escolhida).
  - ⛔ **Recusa do DONO — a diluição** (`dab_route` a depositar `255 − op·(255 − cor)`): o tecto de um traço é a
    cor diluída e nenhuma pilha passa dele (medido: 12 passadas a 0,26 vão `13 → 57`, o tecto 60). A recusa
    MEDIDA de antes (tirar pigmento ⇒ camadas transparentes, γ 0,95) fica válida para a lei de ecrã de então;
    a lei nova abaixo é o que a desfaz.
  - **A lei do ecrã `Cobre`** (`assenta::Ecra::Cobre`, `Sim::lei` do produto; o «Glaze layering» continua
    `Camada`): (1) a claridade sai da QUANTIDADE (a cobertura da massa, `1 − 0,998^m`); (2) a camada de cima
    tapa a de baixo mais do que a força dela, `h = 1 − (1 − a)^esconde`, e o extra pesa pela diferença de
    pigmento (o seno do ângulo dos perfis de absorção) — a mesma tinta sobre si mesma é UMA camada com a massa
    das duas; (3) a tinta molhada escurece como DENSIDADE de cobertura do ponto inteiro (`p = 1 + (K − 1)·a`,
    a cor da camada fixa): mais forte molhada, clareia ao secar sem mudar de tom; o assentar preserva o ecrã
    SECO (`Ecra::seco()`). As velaturas multiplicativas vão a 0 (só afastavam o γ). Constantes com as grelhas
    no doc de cada uma: `TINTA_ESCONDE` = **5**, `TINTA_MOLHADA_ESCURECE` = **3**, `VELATURA_*` = **0**.
  - **A Opacity é a quantidade de pigmento** (`Sliders::pigmento`, levada pelo reconcile): escala SÓ o
    `PigmentPerDab` da célula (`Params::com_pigmento`, nas três portas que pintam — faixa, a do motor, a
    máscara). ⛔ **Medido e recusado: escalar o carimbo** (a `intensity`) — o portão do papel é uma
    subtracção absoluta e comia a tinta clara (12 passadas a 0,26: `2 → 12`), e a água ia junto. Pelo pigmento:
    `11 → 108` e a subir, no tom do pincel (sin 0,005).
  - **Os pincéis** com a Opacity 1 e o pigmento do Rebelle (Round 78 · Wash 21, água 0,4 · Dry Brush 300,
    Paper Gate 0,4 · Drip 63); o pincel sem preset nasce no Wet Round (pigmento 78, Edge Darkening 12).
    Heavy Edge re-medida com a tinta clara: `SEDIMENTA_DA_BORDA` 10 → **0,3** (Cold a meia 1,27 → 1,49 ao meio
    do slider; Rebelle 1,6). Wet Round a 2 s → seco **99 → 55** (Rebelle 111 → 54); W1e seco γ 0,48 · 0,44 e
    sangria molhada 0,26–0,28 (Rebelle 0,42–0,49 · 0,44–0,51 e 0,27); camadas 0,87 · 0,88 (Rebelle 0,89 ·
    0,96); o tom ao secar 0,06 (Rebelle 0,05).
  - **Portões novos (controlo DENTRO de cada um):** `com_a_opacity_baixa_a_cor_acumula_ate_a_escolhida` (o
    controlo é a diluição) · `a_tinta_molhada_parece_mais_forte_e_clareia_ao_secar` (Round ≥ 1,4×; controlo
    `molhada_escurece = 1`: 69 → 55) · `o_traco_nao_muda_de_tom_ao_secar` (≤ 0,15) ·
    `assenta::tests::a_mesma_tinta_nao_muda` (duas camadas da mesma tinta = uma). Renomeado:
    `a_opacity_do_wet_paint_e_a_quantidade_de_pigmento`. **Mutações (7, todas VERMELHAS):** a Opacity não chega
    ao pigmento · volta ao carimbo · o tapar sem a diferença de pigmento (sobreviveu ao 1.º portão — o da mesma
    tinta ganhou a cláusula) · o tapar só com a força · sem o reforço molhado · o preset na força partilhada ·
    o reconcile sem a Opacity.
  - **Réguas que mudaram com a tinta clara:** o controlo da meia inclinação passou a 4× a gravidade (o dobro
    escorria 19 px, barra 20); o Smooth Edges sobre tinta mede o caso do dono com o pigmento de então (800):
    com o do Rebelle o miolo MOLHADO amacia −10 % (o grão claro fica abaixo da mancha `SS0` — aberto (d)). O
    censo dos controlos lia a Heavy Edge «viva e inerte» com o Shape Noise: a tela estava VAZIA desde antes
    (BUGS_painter **#47**, aberto); o censo arma o Checker no Wet Paint. Regravadas, com a razão escrita: as
    impressões do S e da bancada (`opcoes_tests`) e do Smooth Edges no zero (`grid_ratio_tests`).
  - **Aberto:** (a) a largura (W2); (b) o Wash molhado mais escuro que o do Rebelle aos 2 s (64 → 38 contra
    46 → 39); (c) os fios finos e longos do Drip; (d) o Smooth Edges no máximo amacia o grão da tinta clara
    molhada (−10 %); (e) o Shape Noise no Wet Paint (#47).
