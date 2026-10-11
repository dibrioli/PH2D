# 18 — O mapa das mídias: o que o Wet Paint já pinta COMO a mídia, medido contra o Rebelle (2026-10-11)

> Prompt [17](17_PROMPT_janela_presets_todas_as_midias.md), Fase A. Pedido do dono: presets de TODAS as mídias
> (óleo, acrílica, lápis, caneta, aquarela, nanquim, giz de cera…) — no Wet Paint (decisão de 07/10: só ele
> evolui). Antes de construir, **meça se a composição já o exprime** (CLAUDE.md §5.0): para cada mídia, a
> melhor composição dos knobs de HOJE contra a foto do Rebelle da mesma mídia, à largura medida igual.
> Resultado em uma linha: **hoje o motor só pinta a família aquarela; o resto pede motor** (tabela §3).

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
| **nanquim** (tinta com água) | Ink › Brush · Brush 1–3; Watercolor › Sumi-e | escuro, liso, **escurece onde cruza** | Brush: cob 0,79 · 0,72 · 0,19, textura 2,8 · 1,3 · 8,1, borda 0,85 · 0,79 · 1,00 ✓ — **cruzamento 0,87 · 0,24 · 0,95 ✗**. Com as caixas K–M da TELA ligadas: **0,245 ✓** (todas as métricas dentro) | ✗ FORA por defeito: o escurecer do cruzamento só existe com a mistura de pigmentos da OBRA ligada — o preset não a guarda (decisão do dono, §5). Sumi-e: borda macia sai manchada (orla 8–25 · 9, textura 11–24 · 3,4) |
| **caneta** | Ink › Pen · Bullet, Dip Pen, Fountain, Reed, Wood | linha limpa, escura, borda dura, escurece no cruzamento | Dip Pen: cob 0,91 · 0,86 · 0,20, textura 0,3 · 1,4 · 7,8, borda 0,87 · 0,91 · 1,00 ✓ — **cruzamento 1,01 · 0,22 · 0,96 ✗** (com K–M também 1,00: o traço seco tapa) | ✗ FORA: a lei da tinta por preset (W5: Paint & Mix com o limiar da queda; Glaze) |
| **marcador** | Marker · Bullet, Chisel, Liner, Dry, Effect | sobreposição ESCURECE (glaze multiplicativo), grão preso à tela | Bullet: cob 0,91 · 0,91, textura 0,8 · 0,7, orla 1 · 1 ✓ — **cruzamento 1,01 · 0,016 ✗** (com K–M também) | ✗ FORA: o glaze multiplicativo por preset (W5.4) — um «Marcador» que tapa ensina o contrário |
| **óleo · acrílica** | Oil & Acrylic (a ferramenta do Rebelle é «Oil & Acrylic»; a acrílica é ela, não o Express Oil — óleo de secagem rápida) · Bristle Flat, Round Hard, Flat, Knife… | fios com física própria, relevo (HEIGHT), a tinta acaba | Bristle Flat: cob 0,91 · 0,91, borda 0,95 · 0,87 ✓ — **textura 1,0 · 5,9 ✗, CV 0,001 · 0,011 ✗** (um tubo liso, sem estrias); Round Hard: textura 9,0 · 3,1 ✗, borda 0,53 · 0,60 ✗ | ✗ FORA: os fios (resto da W2) + a altura (W3) + a tinta finita (W5) |
| **lápis · grafite · carvão** | Pencil · Graphite 2H–8B, Colored, Charcoal, Dry Media | o papel fura nos vales (borda/miolo 0,32), grão 9 %, orla 7 | sem água o depósito quase some (traços de 1–2 px de largura em r 5–14; textura sem miolo) | ✗ FORA: o caminho sem água (W7) + o papel × pressão (W4) + o Shape Noise (#47) |
| **pastel seco · giz · pastel de óleo · giz de cera** | Pastel · Dry Thin, Soft, Square; Oil Pastel 2, Wax Pencil, Wax Soft; Scrape | bastão com grão aleatório, pó (dual brush), orla 6–11 | Dry Thin: textura 18 · 7,2, borda 0,60 · 0,44, orla 10 · 6 ✗; Oil Pastel 2: o traço não aparece | ✗ FORA: idem lápis + dual brush (W3) |
| **aerógrafo** | Airbrush · Spray 1–5 | névoa de pontos, orla larga (12), textura 23 % | orla 17 · 12, textura 8,9 · 22,7 ✗ (cob 0,34 · 0,38 ✓) | ✗ FORA: fluxo por pressão + cone por inclinação, caminho sem água (W7) |

**A tabela é o que impede um preset de mentir** (⛔ «feature pior que não existir»): só o Liner entrou no 1.º lote.

## §4 — O que se aprendeu (lições com número)

1. **A largura de um traço molhado só se mede DEPOIS da água**: no instante do traço o Wet Round a raio 30 lê
   8 px (um miolo escuro + franja); aos 8 s, 32. As tabelas `width_table` antigas (0 quadros) prometiam o dobro.
2. **A identidade da tinta é o CRUZAMENTO, não o traço**: as 7 métricas do traço aprovavam a «caneta» e o
   «marcador» que tapam; só a métrica do cruzamento os reprovou.
3. **A guache do Rebelle nos volumes de fábrica é o nosso Round** (cob e borda dentro do ruído): um preset com esse
   nome seria um nome, não uma mídia.
4. **O teto de 345 px que a sonda leu era o slider** (`BRUSH_SIZE_MAX_PX = 512` no `set_brush_size_px`), não o
   motor: mede-se além dele com a lei crua × 2.

## §5 — As mídias que ficam de fora: a onda, a barra e a ordem proposta

A barra de cada uma já está nas fotos de `fixtures/rebelle/` (a régua `midias_regua.py`, o vale com o controlo
dentro, o portão `cada_preset_parece_a_sua_midia` ganha uma linha por mídia).

| ordem | bloco (§0.10: uma família, uma janela) | desbloqueia | a barra (fotos) | o que a mede |
|---|---|---|---|---|
| **A** | **A lei da tinta POR PRESET** (W5 reduzida): Glaze multiplicativo e Paint & Mix por preset — o que hoje só existe como caixas da OBRA | caneta (Bullet, Dip Pen), nanquim (Ink Brush), marcador (Bullet, Liner) | `ink` · `dippen` · `inkbrush` · `marker` · `markerbullet` | o cruzamento: 0,016 (marcador) · 0,12–0,24 (tinta) contra 1,00 hoje |
| **B** | **O caminho sem água + o papel × pressão** (W7 + W4 + o Shape Noise #47) | lápis, grafite, carvão, pastel seco, giz, pastel de óleo, giz de cera, aerógrafo | `pencil` · `penciltilt` · `charcoal` · `pastel` · `oilpastel2` · `airbrush` | borda/miolo 0,32–0,44, textura 7–23 %, orla 6–15 px |
| **C** | **Os fios com física própria + a altura** (resto da W2 + W3) | óleo, acrílica, Express Oil; o Flat de pontas retas da aquarela (W3 formas) | `oil` · `oilround` · `express` · `wcflat` | textura 5–10 % com CV ≤ 0,02, período < 15 %; borda/miolo 0,8–0,9 |

A ordem é do dono (o que ele vê primeiro): **A** é a mais curta e dá três mídias que ele pediu pelo nome (caneta,
nanquim, marcador); **B** dá o maior número de mídias (oito); **C** é a maior (≈ 2 janelas) e dá óleo e acrílica.

⚠️ **Decisão do dono que A pede:** a tinta e a caneta do Rebelle escurecem no cruzamento SEM nenhuma opção ligada;
as nossas só escurecem com a «mistura de pigmentos» da obra ligada (medido: o Ink Brush entra na barra inteira com
ela). Ou (1) o preset de tinta liga a mistura da obra ao ser escolhido, ou (2) a lei vai para o pincel (a W5 reduzida
acima) e a obra fica como está. (2) é o padrão-ouro técnico; (1) é imediato mas muda a obra inteira por um pincel.
