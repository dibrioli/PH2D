---
name: project_flip_stroke_analytic_coverage_gp
description: "Traço do Flip = UNIÃO GLOBAL da polilinha num passe (janela p0/p3 + vizinhos GEOMÉTRICOS por broadphase + capsule_dn única + clamp/fade sub-pixel). A mordida morreu"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9af6224e-0122-415e-9f5d-9b462d7c6128
---

O rasterizador de traço do Flip (`ph2d-flip-render`) é o port clean-room do Grease
Pencil **com uma divergência deliberada**: a cobertura de um fragmento é a
**UNIÃO GLOBAL da polilinha** (`dn = min` sobre as cápsulas que alcançam o pixel),
não a distância ao próprio segmento. O GP usa a segunda e por isso tem a "mordida"
(artefato ABERTO no Blender, issue #140075 — lá o default hardness=1.0 + SMAA a
escondem; aqui o pincel macio é o caso comum). **Fechado 2026-07-12.**

**Por que a união conserta:** o perfil de hardness é monótono decrescente, então
`min-distância ⇔ max-cobertura` — os quads que se sobrepõem num pixel passam a
computar o MESMO valor, e o depth first-wins volta a ser invisível (esse é o
invariante que sustenta o tripé: *quads sobrepostos têm de computar a mesma máscara*).

**O fix tem 4 peças — todas obrigatórias:**
1. **Janela de sequência (`p0`/`p3`)** por varying flat (o vertex já os busca pro
   miter). Fecha a quina QUEBRADA (`miter_break`).
2. **Vizinhos GEOMÉTRICOS** (`neighbors.rs`): a janela ±1 NÃO basta — todo traço que
   volta sobre si mesmo (zigzag, laço, letra) tem a mordida de LONGO ALCANCE: a borda
   macia de um segmento (alpha 1/255!) vence o depth e apaga o NÚCLEO de outro
   não-adjacente. Broadphase por grid no `pack` (cacheado por desenho) emite, por
   segmento, a lista dos que podem alcançar seus pixels; o fragment soma ao `min`.
   **União global em UM passe, zero render passes extras.** Critério conservador
   `dist(i,j) < 2·r_i + r_j` — ASSIMÉTRICO (o raio do dono do quad entra dobrado);
   no grid isso vira pad de inserção `r_j` e pad de consulta `2·r_i` (pad igual dos
   dois lados PERDE vizinhos mais grossos e a mordida volta em silêncio).
3. **UMA `capsule_dn`** para o próprio segmento e os vizinhos (raio interpolado pelo
   `t` CLAMPADO). Usar o `thickness` interpolado no QUAD para o próprio segmento
   quebra o invariante com largura por-ponto (pressão) — a mordida sobrevive em 2ª
   ordem. Só o teste com TAPER pega isso.
4. **Par clamp+fade sub-pixel** (`MIN_WIDTH_PX = 1.3` + `thickness` cru no fade): o
   fade do GP sozinho NÃO salva a linha fina (ela não cobre o centro de pixel nenhum
   e SOME). E o AA de borda correto é `clamp(0.5 + (1-dn)/fwidth(dn), 0, 1)` = a
   FRAÇÃO do pixel coberta — a forma antiga (`1-smoothstep`) subestimava traço fino
   em 10×.

Mais: `safe_dir` no miter — ponto DUPLICADO fazia `normalize(0)` = NaN e RASGAVA o
traço (bug latente desde o W1).

**O tripé segue** (miter+`miter_break` · depth por-stroke + GREATER estrito · discard
`a<0.001`). Descoberta: com a união, o **discard deixou de ser load-bearing** (a
mutação não sangra mais) — fica por proteger a degradação do cap/budget.

**Degradações declaradas e determinísticas:** `MAX_EXTRAS_PER_SEGMENT=16` (desempate
por ÍNDICE é obrigatório — dezenas empatam em distância 0 e sem ele o buffer varia
com a ordem de descoberta: quebra replay-hash) e `PAIR_BUDGET` (teto de trabalho; só
o borrão sólido o atinge, onde a mordida é invisível).

**Perf (release):** traço real de 4000 pontos = 1.7 ms de pack; rabisco patológico =
14 ms (bounded). `pack_perf.rs` guarda a ORDEM. Se o preview travar, o próximo passo é
o **pack incremental** (só a cauda muda), não afrouxar o broadphase.

**A escalada de 2 passes (scratch + blend MAX) NÃO foi necessária** — custaria ~2
render passes por traço (~3 ms de CPU com 300 traços) e daria a mesma união.

Verificado em GPU real: 15 testes GPU + 18 unit + 2 composite, debug e `--release`,
com **5 mutações provadas**. Doc definitivo: `docs/Flip/03_traco_rasterizacao.md`.
Ver [[project_flip_module_grease_pencil_2d]] e
[[feedback_oracle_must_model_appearance_not_implementation]].
