# 18 — O mapa das mídias: o que o Wet Paint já pinta COMO a mídia, medido contra o Rebelle (2026-10-11)

> Prompt [17](17_PROMPT_janela_presets_todas_as_midias.md), Fase A. Pedido do dono: presets de TODAS as mídias
> (óleo, acrílica, lápis, caneta, aquarela, nanquim, giz de cera…) — no Wet Paint (decisão de 07/10: só ele
> evolui). Antes de construir, **meça se a composição já o exprime** (CLAUDE.md §5.0): para cada mídia, a
> melhor composição dos knobs de HOJE contra a foto do Rebelle da mesma mídia, à largura medida igual.
> ⛔ **CORRIGIDO no mesmo dia pelo dono (11/10):** *«toda sua avaliação sobre o que não é possível é falsa … basta
> aumentar pigment e o pincel deixa quanta tinta quisermos … em tuning havia … parâmetros de relevo … é só trazer de
> volta»*. A 1.ª versão desta tabela varreu os secos com pigmento 80–600 (nunca alto) e o óleo sem o relevo, que
> estava inalcançável (o slider saíra a 09/10 com o grupo Paper e o render só o desenhava com papel escolhido,
> 10/10). Refeita com pigmento até 2000, o relevo de volta (secção **Relief** do Tuning) e o grão do pincel no
> lugar das cerdas — 428 receitas em 4 rodadas, secas, contra as fotos: **óleo, acrílica, Express Oil, lápis,
> carvão, pastel seco e pastel de óleo ENTRAM**. Ficam de fora a tinta/caneta/marcador (o cruzamento) e o
> aerógrafo (o pó fino). A lição: §4.5.

## §1 — Os dados

- **Os 281 presets do Rebelle em texto** — [`fixtures/rebelle_presets_281.json`](fixtures/rebelle_presets_281.json)
  (cabeçalho: fonte, data, comando; um pincel por linha; ⛔ sem as texturas embutidas, que são assets do alvo).
  Gerador: `ferramentas/extrai_pinceis.py <Rebelle 8> <saída>.json`. `gera_eventos.py` lê dele o `max_brush_size`.
- **25 fotos do oráculo** nos nossos gestos (a bancada do [03](03_tracos_medidos_rebelle_x_ph2d.md)), em
  [`fixtures/rebelle/`](fixtures/rebelle/README.md): as 12 de 07/10 + 13 de 11/10 — Watercolor Round · Wash Round ·
  Flat · Liner, Gouache Round · Gouache Flat, Sumi-e soft, Ink Dip Pen · Ink Brush, Oil Round Hard, Charcoal, Oil
  Pastel 2 (o «Oil Pastel» pintou 0,83 % da tela: a textura OVERLAY dele vai a 0 com a pressão e o papel não se
  desenha sob o Wine — trocado pelo vizinho), Marker Bullet. Todas no estado A = B (já secas depois do `SIMULATION`).
- **A sonda NOSSA** — `crates/ph2d-tool-painter/tests/it/sonda_midias.rs` (`#[ignore]`): um `Preset` em memória
  por receita (de `CANDIDATOS` ou de um JSON `PH2D_SONDA_RECEITAS`, para varrer sem recompilar), a lei raio →
  largura do candidato, o raio da largura da foto, a cena da bancada; `PH2D_SONDA_SECA=1320` = seca (30 s),
  `PH2D_SONDA_KM=1` = as duas caixas K–M da tela ligadas. A régua: [`ferramentas/midias_regua.py`](ferramentas/midias_regua.py)
  (as métricas do `mede_traco.py` no recto + o **cruzamento**).

## §2 — A régua e a barra

- **O vale (05 §3):** passa quem fica do lado do Rebelle do ponto médio entre o Rebelle e o **controlo** — o Wet
  Round à mesma largura. Uma métrica só conta se o controlo estiver longe da foto: **3× o ruído do próprio
  Rebelle** (recto × rápido da mesma foto, p90 sobre as 25: cobertura 0,012 · textura 0,37 % · borda/miolo 0,031 ·
  orla 1 px · CV 0,012).
- ⭐ **O cruzamento** (métrica nova): luminância do miolo do cruzamento vermelho × azul ÷ a menor das duas fora
  dele. **É a identidade da tinta e do marcador no Rebelle:** marcador 0,016 (multiplica: quase preto), Ink Bullet
  0,12, Dip Pen 0,22, Ink Brush 0,24 — contra aquarela 0,93 e guache 0,99 (o de cima tapa). Os nossos: **1,00**
  em todo candidato de tinta (o vermelho tapa o azul).
- ⚠️ **A textura das mídias de papel não se compara** (aquarela, guache): sob o Wine o papel do Rebelle não se
  desenha (memória `reference_rebelle_trial_installed_under_wine`), e a nossa textura vem também do papel a
  fechar o depósito (`paperGate`). Compara-se só onde o preset do Rebelle quase não tem papel
  (`paper_structure_influence` ≤ 0,1: Liner, tinta, marcador).
- ⚠️ **As duas bancadas do Rebelle discordam do Round**: a das gotas (`fixtures/rebelle_presets/`, 150 px) dá tinta
  54 (pálido — é contra ela que o Wet Round foi calibrado a 10/10); a dos gestos (500 px, `wcround.png`) dá
  cobertura 0,78 (carregado). Não investigado (aberto); o Wet Round não mudou.

## §3 — O mapa (mídia do dono → oráculo → o que a faz parecer o que é → hoje)

À largura medida igual; números do recto, seco. **R** = Rebelle · **C** = controlo (Wet Round) · **N** = o nosso
melhor candidato. O que falta e a onda que o entrega são os do [01 §4](01_o_que_o_rebelle_tem_e_nos_nao.md) e do
05 §3.

| mídia do dono | ferramenta · presets-oráculo (Rebelle) | o que a faz parecer o que é (02 §3) | melhor candidato hoje (N · R · C) | veredito · o que impede |
|---|---|---|---|---|
| **aquarela** | Watercolor · Round, Wash Round, Flat, Flat 2, Liner, Mop, Flat Dry, Granulation, Spatter | transparente, orla escura ao secar, o papel fura com pouca pressão | Wash: cob 0,14 · 0,15 · 0,19, borda 1,16 · 1,32 · 1,00 ✓ · **Liner: cob 0,72 · 0,65 · 0,18, textura 3,9 · 1,7 · 8,5 ✓** | ✅ **DENTRO** (os 4 de 10/10 + o **Wet Liner**). ✗ Flat: o Rebelle tem a FORMA chata de pontas retas — o nosso `dab_flatten` só achata (um Round mais claro com outro nome) → W3 (formas) |
| **guache** | Watercolor › Gouache · Gouache Round, Flat, Wash | corpo chapado, sem orla escura | o Gouache Round do Rebelle nos volumes do preset (opacidade 0,65) dá cob 0,22, borda 0,99 — **igual ao nosso Wet Round** (0,19 · 1,00) | ✗ FORA: um preset «Guache» seria o Round renomeado. Pede o «corpo que cobre» (Paint & Mix / Glaze por preset, W5) |
| **nanquim** (tinta com água) | Ink › Brush · Brush 1–3; Watercolor › Sumi-e | escuro, liso, **escurece onde cruza** | Brush: cob 0,79 · 0,72 · 0,19, textura 2,8 · 1,3 · 8,1, borda 0,85 · 0,79 · 1,00 ✓ — **cruzamento 0,87 · 0,24 · 0,95 ✗**. Com as caixas K–M da TELA ligadas: **0,245 ✓** (todas as métricas dentro) | ✗ FORA por defeito: o escurecer do cruzamento só existe com a mistura de pigmentos da OBRA ligada — o preset não a guarda (decisão do dono, §5). Sumi-e: borda macia sai manchada (orla 8–25 · 9, textura 11–24 · 3,4) · ⭐ **42 (16/10): ✅ DENTRO** com o Glaze do pincel — [42_medicao](42_medicao_a_lei_da_tinta.md); o «traço seco tapa» estava errado: o azul está MOLHADO e o carimbo denso substitui a cor (42 §2) |
| **caneta** | Ink › Pen · Bullet, Dip Pen, Fountain, Reed, Wood | linha limpa, escura, borda dura, escurece no cruzamento | Dip Pen: cob 0,91 · 0,86 · 0,20, textura 0,3 · 1,4 · 7,8, borda 0,87 · 0,91 · 1,00 ✓ — **cruzamento 1,01 · 0,22 · 0,96 ✗** (com K–M também 1,00: o traço seco tapa) | ✗ FORA: a lei da tinta por preset (W5: Paint & Mix com o limiar da queda; Glaze) · ⭐ **42 (16/10): ✅ DENTRO** com o Glaze do pincel — [42_medicao](42_medicao_a_lei_da_tinta.md); o «traço seco tapa» estava errado: o azul está MOLHADO e o carimbo denso substitui a cor (42 §2) |
| **marcador** | Marker · Bullet, Chisel, Liner, Dry, Effect | sobreposição ESCURECE (glaze multiplicativo), grão preso à tela | Bullet: cob 0,91 · 0,91, textura 0,8 · 0,7, orla 1 · 1 ✓ — **cruzamento 1,01 · 0,016 ✗** (com K–M também) | ✗ FORA: o glaze multiplicativo por preset (W5.4) — um «Marcador» que tapa ensina o contrário · ⭐ **42 (16/10): ✅ DENTRO** com o Glaze do pincel — [42_medicao](42_medicao_a_lei_da_tinta.md); o «traço seco tapa» estava errado: o azul está MOLHADO e o carimbo denso substitui a cor (42 §2) |
| **óleo · acrílica** | Oil & Acrylic (a ferramenta do Rebelle é «Oil & Acrylic»; a acrílica é ela, não o Express Oil — óleo de secagem rápida) · Bristle Flat, Round Hard; Express Oil › Hairy | estrias das cerdas, relevo, corpo cheio | **Oil Bristle**: textura 6,1 · 5,9, borda 0,83 · 0,87, cob 0,92 · 0,91 ✓ (pigmento 2000, água 0,05, cerdas 4 × 1 500 força 2, **Emboss 2**) · **Acrylic Round**: textura 3,8 · 3,1, borda 0,79 · 0,60, cob 0,91 · 0,88 ✓ · **Express Oil**: textura 8,5 · 9,0, borda 0,84 · 0,80, cob 0,88 · 0,70 ✓ (o «período 47 %» do Hairy é o carimbo dele — 01 §2 L2 —, não a mídia: não conta) | ✅ **DENTRO** (2.º lote) |
| **lápis · grafite · carvão** | Pencil · Graphite 2H–8B, Colored, Charcoal, Dry Media | o grão abre na orla (borda/miolo 0,32), grão 9 %, orla 7 | **Graphite 8B**: textura 9,1 · 9,1, borda 0,60 · 0,32, orla 7 · 7, cob 0,85 · 0,80 ✓ (água 0, pigmento 500, grão Noise 0,4) · **Charcoal**: borda 0,49 · 0,38, orla 12 · 15, cob 0,44 · 0,38 ✓ (pigmento 100, Noise 0,15) | ✅ **DENTRO** (2.º lote). ⚠️ O grão Stucci apaga o traço (o mecanismo do #47) |
| **pastel seco · giz · pastel de óleo · giz de cera** | Pastel · Dry Thin, Soft, Square; Oil Pastel 2, Wax Pencil, Wax Soft; Scrape | bastão com grão, pó, orla 6–11 | **Soft Pastel**: borda 0,61 · 0,44, orla 5 · 6, cob 0,85 · 0,75 ✓ (Noise 0,4) · **Oil Pastel**: textura 15,4 · 17,4, borda 0,56 · 0,42, orla 13 · 11, cob 0,76 · 0,65 ✓ (Noise 0,15 — o grão mais fino que o do seco; o Voronoi passava mas a largura variava até 42 % com a posição) — receitas DIFERENTES (a mesma passava nos dois: seria um preset com dois nomes) | ✅ **DENTRO** (2.º lote; o giz de cera é o Oil Pastel) |
| **aerógrafo** | Airbrush · Spray 1–5 | névoa de pontos finos, orla larga (12), textura 23 % | grão Noise + dispersão: orla 13 · 12, cob 0,33 · 0,38, borda 0,38 · 0,54 ✓ — **textura 7,2 · 22,7 ✗** (manchas, não pó; o lado do grão 0,02 = 0,05 dá o mesmo) | ✅ **DENTRO** (prompt 43, 17/10): o **pó do pincel** (`dust`) — gotas de 2 px; o Airbrush 7/7 (cob 0,36 · tex 21,9 · orla 12). ⛔ «o grão não desce abaixo da célula» era falso: a razão é 1 e o lado 0,02/0,05 caía no corte de 0,1 ([43 §1](43_medicao_o_po_fino.md)) |

**A tabela é o que impede um preset de mentir** (⛔ «feature pior que não existir»): o Liner no 1.º lote; óleo,
acrílica, Express Oil, grafite, carvão, pastel seco e pastel de óleo no 2.º (cada um com o portão
`cada_preset_parece_a_sua_midia`, o Wet Round dentro como controlo). As receitas: `tests/it/receitas_de_fabrica.rs`.

## §4 — O que se aprendeu (lições com número)

1. **A largura de um traço molhado só se mede DEPOIS da água**: no instante do traço o Wet Round a raio 30 lê
   8 px (um miolo escuro + franja); aos 8 s, 32. As tabelas `width_table` antigas (0 quadros) prometiam o dobro.
2. **A identidade da tinta é o CRUZAMENTO, não o traço**: as 7 métricas do traço aprovavam a «caneta» e o
   «marcador» que tapam; só a métrica do cruzamento os reprovou.
3. **A guache do Rebelle nos volumes de fábrica é o nosso Round** (cob e borda dentro do ruído): um preset com esse
   nome seria um nome, não uma mídia.
4. **O teto de 345 px que a sonda leu era o slider** (`BRUSH_SIZE_MAX_PX = 512` no `set_brush_size_px`), não o
   motor: mede-se além dele com a lei crua × 2.
5. ⛔⛔ **«Não é possível» sem varrer o eixo inteiro é um palpite.** A 1.ª versão concluiu que sem água «o depósito
   quase some» depois de varrer pigmento 80–600; o dono, com pigmento alto, pintava traços cheios. E concluiu que
   o óleo não tinha relevo sem perguntar ao CÓDIGO onde o relevo estava (vivo no motor, sem slider e preso ao
   papel). Antes de declarar uma mídia fora: o eixo do knob inteiro (do mínimo ao máximo) e o código do controlo
   que a faria (quem escreve, quem lê, o leitor decide?).

## §5 — As mídias que ficam de fora: a onda, a barra e a ordem proposta

| ordem | bloco | desbloqueia | a barra (fotos) | o que a mede |
|---|---|---|---|---|
| **A** ✅ 42 | **A lei da tinta POR PRESET** (feita: o Glaze do pincel; a mistura ficou da obra — 42 §4): Glaze multiplicativo e Paint & Mix por preset — o que hoje só existe como as caixas K–M da OBRA | caneta (Bullet, Dip Pen), nanquim (Ink Brush), marcador (Bullet, Liner) | `ink` · `dippen` · `inkbrush` · `marker` · `markerbullet` | o cruzamento: 0,016 (marcador) · 0,12–0,24 (tinta) contra 1,00 hoje |
| **B** ✅ 43 | **O pó fino** (o grão abaixo da célula do motor, ou pontos por carimbo) | aerógrafo | `airbrush` | textura 23 % com orla 12 |
| **C** | **Formas** (a ponta reta do Flat; o corpo que cobre da guache) | aquarela Flat, guache | `wcflat` · `gouache` · `gouacheflat` | a extensão nas pontas; cobertura × borda |

✅ **Resolvido no prompt 42 (16/10) pela opção (2), padrão-ouro:** **Decisão do dono que A pede:** a tinta e a caneta do Rebelle escurecem no cruzamento SEM nenhuma opção ligada;
as nossas só escurecem com a «mistura de pigmentos» da obra ligada (medido: o Ink Brush entra na barra inteira com
ela). Ou (1) o preset de tinta liga a mistura da obra ao ser escolhido, ou (2) a lei vai para o pincel (A acima) e a
obra fica como está. (2) é o padrão-ouro técnico; (1) é imediato mas muda a obra inteira por um pincel.
