# Exame da família `basicos` (J3, catálogo A — os primeiros presets do meio Digital)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/basicos.rs`. Pranchas: `pranchas/basicos/` (cegas e
chaves em `cegas/`, `chaves/`). Régua: `exames/<slug>_regua.txt`. Régua do portão por foto: `exames/basicos_foto.py`
(as mesmas contas do `Identidade::de` do teste). Matriz final (`exame.sh --matriz`, a biblioteca de 20/10 ~17h).
Custo: `exame.sh --custo … 5,40,200`, perfil smoke, **load 17–35** (a linha inteira a correr — o p95 é o número; ficou
≤ 3,4 ms em todos, longe dos 16,7). Pasta `Digital/Basic/<Hard|Soft|Pixel|Glaze>` (rótulos já registados).

**Aprovados: Hard Round · Soft Round · Pixel 1px · Glaze Round. Reprovados: Round Pressure · Flat Block-in.**

## Motor (uma mudança, aditiva)

`ph2d-painter-brush/src/dab/bands.rs`: o kernel de faixas da CPU media a orla pela norma do disco e **ignorava a
`dab_quina`** no Digital (a ponta recta só valia no Wet): uma quina 1 pintava um disco. Com quina > 0 usa-se o
`falloff_t` (o quadrado no referencial); sem quina o `sqrt` de sempre, ao bit. Portão red-first
`a_ponta_reta_do_digital_pinta_um_rectangulo` (vermelho antes: cantos 0,0; verde: 1,0; a mutação = tirar a linha).
Vizinhos verdes: `a_miniatura_de_fabrica…` (só o Flat mudava), `os_outros_meios_nao_mudam`, `as_formas::*` (Wet).

## A-1 Hard Round — APROVADO

- Receita (Digital): `hardness 0.93`, cápsula (`continuous_spacing`), `pressureSize 0.98` curva 1,2 (a do Rebelle
  ≈ p^1,2: PL 0,11·0,21·…·0,87 × 0,11·0,21·…·0,89), opacidade fixa; **86 px** = o preset do Rebelle (83,9) + os 2 px
  de orla que ele pinta a mais.
- Oráculo: Rebelle `PENCIL/Digital/Round Hard` (escala 1,025). Régua **452/454** (só a textura do recto de 5 px).
- Vizinho fora da família: Soft Pastel **0,716** (claramente longe); a origem é o 1.º dos 228 (Digital/Round Hard 0,198).
- Razão de uso: o disco duro opaco que afina pela pressão — bloquear formas e linha limpa.
- Revisão cega (5 rondas; a cada cura um revisor novo). Curado: largura a 200 (escala), a ponta da rampa em contas de
  1 px (→ cápsula), o aro esfumado a 200 (dureza 0,85 → 0,93). Última (r5): *«ORIGINAL: 1 (o NOSSO) — confiança
  baixa … nos gestos e no sobre os dois são quase idênticos … diferenças no limite: o início fraco da rampa (fio × bico),
  a orla a 200»*. Cruzamento: escolheu o nosso como original nos gestos; nos tamanhos não separou oráculo, nosso e
  vizinho (sem pressão os três são discos duros). Sobra no limite do visível e ao contrário da r1 (a ponta).
- Custo p95: 0,15 · 0,53 · 3,37 ms; RSS 20–24 MiB.

## A-2 Soft Round — APROVADO

- Receita: `hardness 0`, queda **`Sharp`** (o perfil gaussiano do Rebelle: 0,04·0,12·0,24·0,37·0,51… × 0,06·0,14·
  0,25·0,37·0,51…; a `Smooth` acumulava num planalto), sem tamanho pela pressão, `pressureOpacity 1` curva **2,6**,
  tecto da mão 0,8 (PC 0,04·0,13·0,29·0,49·0,69·0,86 × Rebelle 0,03·0,12·0,31·0,52·0,70·0,86); **163 px** = o preset
  (152,6) × 30/28.
- Oráculo: Rebelle `Round Soft` (escala 1,071). Régua **448/454** (o recto de 5 px 0,78 × 0,86).
- Vizinho: Graphite 8B **0,918**; origem 1.ª (Digital/Round Soft 0,573).
- Razão de uso: luz e sombra suaves sem textura — o único disco macio opaco da biblioteca.
- Revisão cega r2: *«Não consegui separar o ORIGINAL da IMITAÇÃO … a única coisa que se aproxima de uma diferença é
  suspeita e não confirmada (a borda a 200)»*. r1 tinha listado a cauda fraca (curada: sem piso, curva 2,6).
- Custo p95: 0,17 · 0,27 · 1,74 ms; RSS 21–25 MiB.

## A-5 Pixel 1px — APROVADO

- Receita: a ponta de píxel **Pixel-perfect** (`pixel 2`, L18), `hardness 1`, sem pressão; **1 px** (MyPaint `1pixel`,
  raio e^−0,58 → 1,1 px).
- Oráculos: MyPaint `experimental/1pixel` (principal: o recto de 5 px = 5 px como o nosso; o Krita pinta 6) e Krita
  `u) Pixel Art`. Régua 440/454 contra o MyPaint: os ✗ são as emendas do MyPaint nos traços rápidos (ele acaba antes e
  ondula 0,4–0,6 px); o Krita não as tem (rugosidade 0 nos rápidos).
- Vizinho: Hard Round 0,825 (família) · Mapping Pen **0,869** fora (claramente longe).
- Razão de uso: pixel art — zero píxeis parciais, um píxel por coluna. A 40 px a matriz vê um disco duro (é por
  desenho); a identidade mede-se no 1 px: portão `o_pixel_1px_e_o_unico_que_pinta_o_pixel_exacto` (0 parciais,
  460/460 colunas; o Hard 44 parciais e 82/460, o Soft idem). Mutação: `pixel 1` (sem tirar as quinas) → 459/460 SANGRA.
- Revisão cega r2: cruzamento — achou o MyPaint pelas *«emendas nos verticais e traços que acabam antes»*, defeitos do
  oráculo que o Krita não tem; *«cor, opacidade, pressão, cruzamento e secagem: nenhuma diferença»*. Decisão (produto):
  não se imitam as falhas de amostragem de UM oráculo que o outro não tem.
- Custo p95: 0,16 · 0,24 · 0,49 ms; RSS 20–25 MiB.

## A-6 Glaze Round — APROVADO

- Receita: dabs duros quase transparentes que SE SOMAM (`hardness 0.8`, Accumulate, `flow 0.03` — o perfil macio do
  MyPaint é a corda do disco: 0,12·0,40·0,59·0,70·0,77 × 0,17·0,41·0,55·0,66·0,73) e o **tecto do traço 0,37** (L8):
  a espiral que se cruza sobe a 0,337 como a do MyPaint e pára; entre traços acumula. Pressão: tamanho 0,7, opacidade
  curva 1,3. **14 px** = o MyPaint à pressão cheia.
- Oráculo: MyPaint `deevad/detail_brush_large_glazing` (o Krita `Bristles-4 Glaze` é uma cerda opaca: não é velatura).
  Escala 2,1. Régua **444/452**.
- Vizinho: Wet Wash **1,188**.
- Razão de uso: velaturas digitais por camadas — a única translúcida com tecto por traço.
- Revisão cega r2: *«Não consigo separar o original da imitação … as únicas diferenças: o centro do recto de 200 px e
  a força dos anéis nos cruzamentos azul sobre vermelho»* (os anéis são os carimbos do MyPaint). r1 tinha visto a orla,
  a espiral fundida e a largura (curado: dureza 0,8 + fluxo 0,03 à escala certa).
- Custo p95: 0,19 · 0,53 · 2,79 ms; RSS 23–30 MiB.

## A-3 Round Pressure — REPROVADO

- Contra o Krita `b) Basic-5 Size Opacity` (régua 440/454, d 0,41 do Hard Round — o eixo pressão vê-se). Três
  revisores cegos (r1–r3) acharam o Krita e o nosso como o par e listaram SEMPRE a mesma coisa: **a cauda fraca da
  rampa é mais curta** (Krita: 0,03 → 0,50 ao longo de 200 px, linear; nós: começa ~20 px depois e sobe depressa).
  Mecanismo medido: a opacidade pela pressão do Krita é por TRAÇO (cada dab no máximo a sua opacidade — alpha
  darken); a nossa multiplica a cobertura de cada dab e os dabs somam-se até à Strength. Nenhuma curva/piso/tecto da
  mão a tornou linear (6 variantes: ou começa tarde ou o meio satura, PC 0,94 × 0,70). Também as festas de carimbo do
  Krita a 200 px (espaçamento automático ∝ √D): imitá-las a 0,22 punha rugosidade 0,21 nos rectos de 40 (que no Krita
  são lisos). ⇒ **fila de motor**: «pressão → opacidade do TRAÇO (máximo por dab, sem somar)».
- Mutação do portão de foto com ele: sem a opacidade pela pressão → rampa 1,00 × 0,267 e «pinta o Hard Round» SANGRA.

## A-4 Flat Block-in — REPROVADO

- Contra o Rebelle `Digital/Flat` (lâmina FIXA, L 0,98·D × T 0,4·D, ângulo 90°; régua 444/454; origem 1.ª, 0,412).
  Pediu a ponta recta (o motor acima). Revisão cega r1: *«1. Traço fino (5 px): o original sai escuro e opaco, a
  imitação clara (0,44 × 0,87). 2. As pontas dos verticais afinam e desvanecem no original. 3. A 200 px o esbatido do fim
  é metade»*. Mecanismo: as pontas esbatidas são tinta que acumula (dabs a 0,28) e a 5 px o lado fino (2 px) leva dois
  contributos por píxel, seja qual for o passo (`spacing` 0,01 dá o mesmo 0,48; baixar o passo mínimo do acumulo a
  0,25 px também — revertido); sem Accumulate o miolo enche mas as pontas ficam secas. ⇒ **fila de motor**:
  «acumulação do Digital independente do tamanho na lâmina fina».

## Portões (verdes por nome, 20/10)

`cada_basico_se_reconhece_pela_foto` (borda · opacidade · rampa da largura · rampa da cobertura · tecto, contra a foto
de cada oráculo, e a lei dos irmãos entre os quatro) · `o_pixel_1px_e_o_unico_que_pinta_o_pixel_exacto` ·
`a_ponta_reta_do_digital_pinta_um_rectangulo` · os da fábrica (`the_factory_presets_are_counted`,
`cada_preset_de_fabrica_e_a_sua_receita`, `…_volta_ao_byte`, `a_miniatura…`, `a_contagem_por_meio`).

## Para a fila (coordenador)

1. Motor: pressão → opacidade POR TRAÇO (alpha darken) — para o Round Pressure (e o PS «Hard Round Pressure Opacity»).
2. Motor: acumulação do Digital a ≤ 5 px na lâmina fina; o esbatido do fim a crescer com o tamanho — para o Flat.
3. A orla do Hard é relativa ao raio (dureza): 1 px de AA fixo como o Rebelle não existe; 0,93 + cápsula é o meio-termo
   (binária a 40, 2–3 px a 200).
