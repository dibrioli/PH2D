# 10 — PROMPT da janela W1d: a orla com meio-tom, sem faixa de cor pura, e a raiz em funil

> Cole o bloco abaixo numa janela NOVA. É auto-contido. Origem: a auditoria de 09/10
> ([09](09_PROMPT_janela_auditoria_W1c.md), 05 §4b «🔎 Auditoria da W1c») sobre o smoke do dono:
> [1_borda](fixtures/smoke_dono_w1c/1_borda.png) · [2_raiz_da_gota](fixtures/smoke_dono_w1c/2_raiz_da_gota.png)
> — *«borda pixelada e artificial. veja o escorrimento. Que app pro faria isso?»*. A prancha que o dono viu
> da auditoria: [prancha_auditoria_w1c.png](prancha_auditoria_w1c.png) (o Rebelle e o nosso, 8×, filtro suave).

```
TROCA DE AGENTE — line/PainterWatercolor («Wet Painter»), onda W1d: a orla de baixo com MEIO-TOM, sem a
faixa de cor pura, e a raiz do fio em FUNIL — pela auditoria de 09/10.

FASE 0 — ONDE VOCÊ ESTÁ (antes de ler QUALQUER ficheiro):
1. cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
   → .../line-PainterWatercolor e line/PainterWatercolor. Outra coisa: PARE e reporte.
2. git log --oneline -4 && git status --short → a ponta é «audit(Painter 09/10): a orla e a raiz da W1c…»;
   árvore limpa.
3. git fetch -q origin; se o main andou: git rebase main (conflito fora dos seus ficheiros: PARE e reporte).
   ⚠️ O degrau do schema da linha é 187 → 188 com migração (ProjectFileV187, braço `187 =>` em
   shells/desktop/src/project_load_read.rs): se o main passou de 187, renumere À MÃO os dois.
4. bash scripts/hw-profile.sh · bash scripts/btrfs-health.sh. Comando pesado SEMPRE por bash scripts/ph2d-run.sh;
   um hook recusa todo texto de Bash que contenha «cargo test» fora da porta — inclusive em heredoc: escreva
   os scripts num ficheiro (ferramenta Write) e corra `bash scripts/ph2d-run.sh bash <ficheiro>`.
5. AS QUATRO CERCAS verdes ANTES de tocar em produto, e até ao fim:
   a) bash docs/Painter/NewFeaturesWetPaint/ferramentas/cerca.sh
   b) os_outros_meios_nao_mudam (Digital/Watercolor/Impasto ao byte)
   c) sem_create_drips_e_o_escorrimento_natural_de_antes_ao_byte (fnv 0x7cc9…6d0e · 0x0747…3d4e)
   d) com_a_tela_direita_o_traco_e_o_de_antes_ao_byte (fnv 0xafa3…b7fa)
   + a suíte do motor (cargo test -p ph2d-wet-paint) e os portões da W1c (borda_do_s, gotas_pela_lei_do_rebelle).
   «NÃO PIORAR A ENGINE» = estas cercas verdes do princípio ao fim; toda mudança SÓ onde a linha de contacto
   existe (Create Drips ligado E a tela além do ângulo mínimo). Se uma cura precisar de tocar no caminho sem
   a lei, PARE e reporte.

LEIA (nesta ordem; só isto, depois o código)
1. docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md (a cada passo; §4: efeito perceptual não fecha sem OLHAR).
2. docs/Painter/NewFeaturesWetPaint/05_plano_da_linha_e_o_antes.md §4b — da 3.ª ronda das gotas até à
   «🔎 Auditoria da W1c, 09/10» (as recusas medidas da W1b/W1c: não as reproponha).
3. fixtures/rebelle_gotas/README.md (o S a 1:1: s_inclinado.png, s_inclinado_1.png, s_zoom100.png, e a
   MESMA tela a 400 %: s_zoom400.png).
4. A régua: docs/Painter/NewFeaturesWetPaint/ferramentas/borda_cobertura.py (Python, corre sobre o oráculo e
   sobre os PPM) e crates/ph2d-tool-painter/tests/it/auditoria_borda_w1c.rs (a mesma conta, os portões e a
   sonda `sonda_auditoria_w1c` — PH2D_SONDA_OUT=<dir>; cenas `aud_*.ppm`).
5. Código: crates/ph2d-wet-paint/src/drips/edge.rs (pin_downhill :15, Folha::por_fora :96, is_front :134);
   solver/advect_jacobi.rs:163 (onde o pin entra); drips.rs (DripLaw, drip_pass, feed/bebe — só película);
   drips/trail.rs (largura :75–78, primeira fileira na orla + 1, pescoço :120–123, tinta do funil :135,
   settle :193); drying.rs:143 (o edge darkening da secagem); o composite do tool
   crates/ph2d-tool-painter/src/tool/paint/wetpaint/composite.rs (:202 grade 1 = cópia; :251 cobertura = `pa`)
   e grid_map.rs:231 (smooth_weight — a grade 2 reconstrói por smoothstep e por isso já tem meio-tom).

DECISÕES DO DONO JÁ TOMADAS (não re-perguntar)
- Só o Wet Paint evolui; Digital/Watercolor/Impasto não mudam um byte.
- Create Drips DESLIGADO = o escorrimento natural de antes, ao byte. Create Drips LIGADO = a lei do Rebelle.
- Os controlos das gotas ficam na secção Paper; Easy Drips fica.
- Pendentes (NÃO decida): o custo a 4096² (pior tique das poças 3,9–4,3 ms > 2 ms — parado); Create Drips
  ligado ou não de fábrica; e NOVO: os pontinhos do carimbo (A4) — curá-los muda as cercas c) e d).
- Decisões técnicas são suas (padrão-ouro); produto e envio são do dono.

OS ACHADOS (09/10; descida do S x 440–632, tela no máximo, 30 s; o nosso = Wet Round de fábrica a 100 px,
água 1,0; B0 = o escuro do corpo com a tela direita do MESMO pincel)

A1 ⭐ A ORLA É 0/1. O dono olha a ~8,5× (cada degrau da captura = 1 px da tela; a rampa de cada degrau mede
   8 px de ecrã = o filtro Linear por omissão, ImageFilterMode::Smooth, ph2d-render/src/image_filter.rs:17).
   O ecrã do Rebelle a 400 % também amplia suave (s_zoom400.png): a escada é o CONTEÚDO da tela. A linha de
   contacto zera o fluxo na célula `por_fora` (edge.rs:15/96) e nada lhe dá meio-tom — nem o motor nem o
   composite (cobertura = `pa`, composite.rs:251; a grade 1 é cópia, :202).
   | orla da descida | binária % | meio-tom % |
   |---|---|---|
   | W1c, lei ligada (bancada = azul do dono a 100 px) | 72,5 | 9,8 |
   | azul do dono a 60 px | 57,0 | 23,8 |
   | Create Drips desligado (o natural) | 9,8 | 43,5 |
   | tela direita | 16,6 | 67,4 |
   | grade 2 (smoothstep) | 0 | 68,9 |
   | Rebelle s_inclinado · _1 · s_zoom100 | 5,7 · 19,7 · 19,7 | 89 · 67 · 67 |
   PORTÃO: a_orla_do_s_tem_meio_tom_como_a_do_rebelle (binária ≤ 25 %, meio-tom ≥ 55 %, bancada e dono).

A3 ⭐ O CONTRASTE MULTIPLICA A ESCADA. O degrau orla → papel vale 1,04·B0 = 222 de escuro NUM pixel
   (Rebelle 0,32–0,44·B0 = 33–45: 4,9×), porque o pigmento suspenso espremido atrás da linha seca em cor
   pura: a faixa de ≥ 88 % da cor pura tem 4 px (p75 6) na bancada e no azul do dono a 100 px (a captura do
   dono: 4–5 px), 2 px a 60 px; Rebelle 0. O portão da W1c (faixa ≥ B0 ≤ 6) aprova-a: a barra ficou «no
   vale» entre 11 e 0. A drenagem do canal (drips.rs feed/bebe) tira só PELÍCULA: «a água sai, o pigmento
   fica na linha». ⚠️ O Rebelle PERDE tinta com a tela inclinada (Σ escuro tarde ÷ cedo 44–51 %, plano 93 %,
   README das fixtures) — o nosso NÃO foi medido: meça-o primeiro.
   PORTÃO: a_orla_nao_e_um_degrau_de_cor_pura (faixa ≤ 1 px, salto ≤ 0,6·B0; bancada e dono a 100 px).

B2 ⭐ A RAIZ É UM RECTÂNGULO. O fio tem 3 px desde 1 px abaixo da orla (largura a 1·2·4·8·16·32 px da raiz:
   3,5 · 3 · 3 · 3 · 3 · 3; Rebelle 5 · 4 · 3 · 3 · 3 · 3 e, a 400 %, um «Y» que se funde na orla). A largura
   é `max(wmax·(falta/run)², piso)` (trail.rs:75–78); o pescoço (trail.rs:122–123) dá `round(3·pescoço)`
   ≤ 3 = o piso: não alarga nada — o «funil» da W1b é só TINTA (`neck_dark`, trail.rs:135). A primeira
   fileira é plana (orla + 1) sob uma orla OBLÍQUA: do lado de cima a orla fica acima do topo do fio e faz o
   canto que o dono circulou.
   PORTÃO: o_fio_alarga_para_a_orla_como_o_funil_do_rebelle (largura a 1 px − a 16 px ≥ 1,5).

B1 (sem defeito — guarda) A orla NÃO muda de altura no fio: degrau extrapolado 0,74 px nas raízes contra 1,26
   longe dos fios (Rebelle 1,8 / 1,29); `drain_after` falso e `recede` = `hold` não mudam nenhuma raiz. O
   «degrau» da captura é o B2 (topo plano sob orla oblíqua) + a faixa de cor pura que continua do lado de
   baixo. GUARDA: a_raiz_do_fio_nao_faz_degrau_na_orla (verde; veja-a VERMELHA por mutação antes de confiar).

B3 (sem defeito) O fio é vertical também no Rebelle (desvio 0 a 2–32 px da raiz): não o curve.

A4 (pendente do dono) OS PONTINHOS por baixo da orla existem com a tela DIREITA: 72 px de tinta por 100
   colunas, 3–20 px abaixo da orla (Rebelle plano 2; inclinado 4–57); na grade 2 ~6. Hipótese por leitura,
   NÃO medida: o rabo do falloff (brush.rs:131–140) × a textura das cerdas × `pigmentPerDab` 600, sem piso
   (trail/deposit.rs DepositLaw::cell). Curá-los muda o carimbo ⇒ as cercas c) e d) mudam de bytes:
   NÃO cure sem ordem do dono; meça a hipótese (variantes no mesmo processo) e leve-lhe o número.

C (as réguas que não viram) a_borda_do_s_e_a_do_rebelle mede a POSIÇÃO da orla (salto/rms) e a faixa ≥ B0
   — uma escada de 1 px perfeita passa «lisa», e a barra da faixa (≤ 6) aprova 4 px de cor pura;
   as_gotas_saem_de_toda_a_orla_do_s mede o funil pela TINTA (1,69 ≥ 1,5) — um rectângulo com 3× tinta passa;
   a_grade_2_da_os_mesmos_fios_em_px compara comprimentos — não vê que a grade 2 tem meio-tom e a 1 não;
   a bancada (traços rectos de 20 px) não mede a orla pixel a pixel. Todas a 1:1, com o azul da bancada; o
   dono olha a 8,5× com o azul puro dele.

A CURA (direcções — meça antes de escolher; todas SÓ onde há linha de contacto)
- A1: ⭐ meio-tom SEM deixar passar água (a faixa de transição que deixa passar fluxo está RECUSADA: inunda).
  ⭐ PRIMEIRO CANDIDATO (dono, 09/10): o NOSSO «Smooth Edges» do Watercolor (`watercolor_aa.rs::aa_coverage`)
  — avaliado em emulação (05 §4b, `ferramentas/smooth_edges_no_wet.py`): binária 72,5 → 14 %, meio-tom 9,8 →
  62 %, salto 1,04 → 0,47·B0, pontinhos 94 → 38; NÃO tira a faixa pura (5 → 4 px) nem o rectângulo da raiz.
  Reuse a LEI (uma porta só — não copie a função: «a mesma conta ao lado de uma cópia» diverge), com a mancha do
  Wet Paint como campo de cobertura, no composite. ⚠️ Decisão do DONO antes de portar: só onde há linha de
  contacto (as cercas c/d ficam) ou um «Smooth Edges» no Wet Paint como no Watercolor, em toda a mancha (cura
  também os pontinhos da tela direita, mas as cercas c/d mudam de bytes e são regravadas de propósito).
  Alternativa a medir contra ele: anti-serrilhado MORFOLÓGICO da máscara da orla (MLAA, Reshetov 2009) — a orla por
  coluna ao longo da transversal à gravidade, uma recta local sobre ±4 colunas, e a cobertura de cada pixel
  da orla e do pixel seco abaixo dela = a área do pixel do lado molhado dessa recta; aplicada à cobertura do
  pigmento (`pa`) no composite, só nas células de orla (`is_front`) — sem orla (Create Drips off, tela
  direita) o composite é o mesmo ao byte. Alternativa a medir contra ela: a cobertura no motor (o pigmento
  assenta na célula seca de baixo na proporção da cobertura, nunca a água). O preview e o run_full têm de
  dar o mesmo. Olhe a 8× (§4 da DIRETIVA): `borda_cobertura.py --amplia`.
- A3: a faixa de cor pura sem reabrir o que foi recusado. Candidato: a drenagem do canal leva também o
  pigmento SUSPENSO da orla da bacia na proporção da película que tira (feed/bebe), e o Σ escuro tarde ÷ cedo
  aproxima-se do Rebelle (44–51 %); confira que o fio continua pálido (40–52 %, gate da bancada) e o corpo não
  escurece. Meça também a faixa ao longo da orla LONGE das bacias (a drenagem só age nelas).
- B2: o funil em FORMA: nas primeiras NECK_PX fileiras a largura cresce para a orla (Rebelle 5–7 → 3 em 2–4
  px) e cada coluna do pescoço começa na orla DA SUA coluna (enche a cunha entre a orla oblíqua e o fio) — o
  topo deixa de ser plano. O fio continua vertical (B3) e o degrau da guarda B1 não pode aparecer.

GATES (red-first; os três vermelhos já existem em tests/it/auditoria_borda_w1c.rs com #[ignore] — tire-o)
- a_orla_do_s_tem_meio_tom_como_a_do_rebelle · a_orla_nao_e_um_degrau_de_cor_pura ·
  o_fio_alarga_para_a_orla_como_o_funil_do_rebelle · a guarda a_raiz_do_fio_nao_faz_degrau_na_orla.
- Os portões da W1c continuam verdes SEM mexer na barra (borda_do_s: degraus ≤ 20 %, rampa ≥ 18, corpo ≤ 0,55;
  fios 8–14, 3–4 px, 228–430 px; grade 2; a bancada 259 / 258 / 464 px).
- Mutações no fecho (controlos verdes antes): o anti-serrilhado desligado ✗ meio-tom · aplicado também sem
  orla ✗ cercas c)/d) · a drenagem sem o pigmento ✗ faixa · o pescoço sem a largura ✗ funil · o pescoço a
  começar na fileira plana ✗ (meça qual portão) — uma mutação que SOBREVIVE é equivalente provado ou a lei sai.

RECUSAS MEDIDAS (05 §4b — não reproponha): a faixa de transição que deixa passar fluxo para o papel seco
(inunda: 106–127 mil px) · a difusão do pigmento (inerte sobre o assente) · cortar só a componente de descida
(degraus e bases planas) · baixar a força da lei (o corpo não clareia: 0,76) · assentamento na água parada
(o corpo congela escuro) · EdgeDarkening 100/200 · gotas sem o canal · neck_dark ≥ 3 (brechas na orla recta)
· alcance fixo da bacia.

CUSTO: o anti-serrilhado corre no composite a cada quadro sujo — meça-o a 4096² (as três poças e a tela
inteira molhada; com e sem, intercalados no MESMO processo, p50/p99/máx, load ao lado). Kill-criterion: > 2 ms
no pior quadro depois da 2.ª tentativa ⇒ PARE e reporte (como a W1c). A drenagem com pigmento: o estágio das
gotas não pode subir acima do que a W1c mediu (p99 1,27–1,49 ms).

FECHO
1. Tabela no 05 §4b (antes · depois · Rebelle) com as réguas de borda_cobertura.py e as da W1c; prancha a 8×
   (o nosso contra o Rebelle, a orla e a raiz), como prancha_auditoria_w1c.png.
2. Commit por caminho (--no-verify; ficheiro novo: git add antes); nunca push.
3. AO DONO, curto e sem jargão: o que mudou na borda e na raiz, a imagem ampliada lado a lado, e o smoke em
   passos numerados (1) o comando com o cd · (2) onde clicar (Tilt no máximo; Paper → Create Drips) · (3) o
   que tem de ver ampliado · (4) como saber que deu errado. Leve-lhe também a pergunta dos pontinhos (A4).
```

## Os achados no template da DIRETIVA §3 (auditoria de 09/10)

LOC lidas (a janela e os quatro agentes): `drips.rs` 468 · `drips/edge.rs` 348 · `drips/trail.rs` 250 ·
`borda_do_s.rs` 489 · `gotas_pela_lei_do_rebelle.rs` 478 · `sonda_lei_das_gotas.rs` 223 · `sonda_gotas_do_dono.rs`
155 · `deposit.rs` 250 · `borda.py` 182 · trechos de `sim.rs`, `advect_jacobi.rs`, `drying.rs`, `composite.rs`,
`grid_map.rs`, `image_filter.rs`, `project.rs`, `renderer.rs`, `painter_bridge*.rs`, `brush.rs`, `dab_route.rs`,
`defs.rs` (~1 400) — ≈ 4 500.

```
LENTE:  correção perceptual (A1)
CLAIM:  a orla de baixo com a lei ligada tem meio-tom como a do Rebelle — FALSO
TRAÇO:  advect_jacobi.rs:163 → edge.rs:15 pin_downhill (uv = 0 na célula por_fora, edge.rs:96) → a célula seca
        não recebe água nem pigmento → composite.rs:202 (grade 1 = cópia) → :251 cobertura = pa → o ecrã amplia
        Linear (image_filter.rs:17) → cada pixel 0/1 vira um degrau de 8,5 px na captura
ASSERÇÃO-VERMELHA: auditoria_borda_w1c::a_orla_do_s_tem_meio_tom_como_a_do_rebelle — 72,5 % > 25 (visto vermelho 09/10)
NÃO-CHECADO-PELA-COMPILAÇÃO: a régua da W1c (posição da orla) aprova a escada; nenhum portão olha a ampliação
LOC LIDAS: ver acima
```
```
LENTE:  correção perceptual (A3)
CLAIM:  a orla não é um degrau de cor pura — FALSO
TRAÇO:  pin → o suspenso espreme-se atrás da linha → drying.rs:143 (secagem de borda) assenta-o → feed/bebe
        (drips.rs) escoam só película → faixa de ≥ 88 % da cor pura de 4 px, salto 1,04·B0 num pixel
ASSERÇÃO-VERMELHA: auditoria_borda_w1c::a_orla_nao_e_um_degrau_de_cor_pura — 4 px / 1,04 (visto vermelho 09/10)
NÃO-CHECADO-PELA-COMPILAÇÃO: a barra da W1c (faixa ≥ B0 ≤ 6) aprova 4 px; o Σ de tinta tarde/cedo não é medido
LOC LIDAS: ver acima
```
```
LENTE:  correção perceptual (B2)
CLAIM:  o fio alarga para a orla (funil) — FALSO
TRAÇO:  trail.rs:75–78 largura = max(wmax·(falta/run)², piso 3) → :122–123 pescoço round(3·p) ≤ piso → :135 só
        tinta → settle :193 fileira plana em orla + 1 sob orla oblíqua → o canto da captura 2
ASSERÇÃO-VERMELHA: auditoria_borda_w1c::o_fio_alarga_para_a_orla_como_o_funil_do_rebelle — 3,5 → 3,0 (visto vermelho 09/10)
NÃO-CHECADO-PELA-COMPILAÇÃO: as_gotas_saem_de_toda_a_orla_do_s mede o funil pela tinta (1,69 ≥ 1,5)
LOC LIDAS: ver acima
```
```
LENTE:  correção (B1 — guarda)
CLAIM:  a orla não muda de altura na raiz do fio — VERDADEIRO (0,74 contra 1,26; Rebelle 1,8 / 1,29)
TRAÇO:  spawn (drips.rs) nasce na célula de orla; feed/bebe mudam só película nas células da bacia; nenhuma
        variante (drain_after, recede = hold) move uma raiz
ASSERÇÃO-VERMELHA: auditoria_borda_w1c::a_raiz_do_fio_nao_faz_degrau_na_orla — VERDE, nunca vista vermelha:
        a janela de cura vê-a vermelha por mutação antes de confiar nela
NÃO-CHECADO-PELA-COMPILAÇÃO: o funil novo (B2) pode criar o degrau
LOC LIDAS: ver acima
```
```
LENTE:  correção (A4 — pendente do dono)
CLAIM:  sem pontinhos de tinta abaixo da orla — FALSO, e independe das gotas
TRAÇO:  medido com a tela DIREITA (72 por 100 colunas; Rebelle plano 2); hipótese por leitura: brush.rs:131–140
        (rabo do falloff) × textura das cerdas × pigmentPerDab 600 em trail/deposit.rs (sem piso)
ASSERÇÃO-VERMELHA: nenhuma escrita — curar muda as cercas c)/d) ao byte; ordem do dono primeiro
NÃO-CHECADO-PELA-COMPILAÇÃO: a cerca ao byte trava a cura e também esconde o defeito
LOC LIDAS: ver acima
```
