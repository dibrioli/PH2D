# HANDOFF — Inflate: o footprint virou um FECHAMENTO MORFOLÓGICO (2026-07-18)

**Linha:** `line/Painter` (Modo L). **Estado:** fechado, **pendente smoke do Enio**. NÃO integrado (aguarda ordem
explícita). Continuação de [`HANDOFF_line_Painter_inflate_edges_2026-07-16.md`](HANDOFF_line_Painter_inflate_edges_2026-07-16.md)
e do trabalho da bola limitada exata (`e167926d`).

## O que o smoke da bola limitada revelou (Enio, 2026-07-17)

> *"funciona melhor que o modelo anterior, mas não protege as bordas do traço que não deveriam nem ser tocados,
> assim como os cruzamentos também são muito erodidos. mas já é melhor que antes"*

Dois sintomas, **um mecanismo**: a bola limitada fechou o rasgo da junção, mas cresce o footprint o MESMO `ρ`
em toda direção. Nas axilas côncavas isso enche o gash (certo); nas bordas convexas vira uma **saia
translúcida** na tela nua (a queixa 1); no cruzamento as saias sobrepostas leem como sombra (a queixa 2 —
medido: **0 texel perde cobertura**, não é erosão real, é a saia). Diagnóstico por **render-and-look**
(`push_look_probe`, cena 12 = a cruz do Enio com a luz do impasto).

## O conserto: dilatação para o HEIGHT, CLOSING para a MATÉRIA

- **Height:** continua a dilatação da bola (`blob_ball`) — o domo redondo que o Enio aprovou. Intocado.
- **Matéria (cobertura/material/cor):** o footprint é um **fechamento morfológico** (dilata + erode com a
  MESMA bola). Um closing é exatamente *"enche concavidade até `2ρ`, deixa borda convexa onde está"* — sem
  threshold mágico, `ρ` é o único parâmetro (a bola que o artista já escolheu). Na borda convexa a altura
  cresce além da cobertura, onde a luz não a pesa (`relevo sobre cobertura zero não acende`), então o domo
  sobe SEM a silhueta andar.
- **A cobertura vem do campo de ALTURA suave `hbuf`**, não da POSIÇÃO do argmax. `hbuf` é um `max` de bolas
  contínuas ⇒ sem costura de Voronoi (o argmax vazava como **raios radiais** num blob redondo). Opaca com AA
  fino via `COVER_AA` (em loads de relevo).

### Arquivos
- **`crates/ph2d-tool-painter/src/tool/paint/sculpt_close.rs`** (NOVO): o closing. EDT quadrada exata
  (Felzenszwalb-Huttenlocher, `O(área)`, separável; 2 EDTs **fundidos** em 2 transposes; serial p/ `cr`
  pequeno de pincel, paralelo p/ Filter Layer; **early-out** quando o `cr` não tem bare ⇒ closing trivial).
  `CLOSE_AA = 1.5` px, `COVER_MASK = 24`.
- **`sculpt_inflate.rs`**: `render_inflate` chama `closing_fill` e modula `tc = clamp(hbuf/COVER_AA) * cfill`.
  `COVER_AA = 1.6` loads. `ball_fraction` removido do caminho da matéria.
- **`sculpt_offset.rs`**: `blob_ball` inalterado (só docs); a função `ball_fraction` foi **removida** (morta).
- **`impasto_settings.rs`**: `toggle_brush_impasto` troca o falloff Smooth→**Sphere** ao LIGAR (só se ainda no
  Smooth de fábrica — respeita escolha deliberada, não corrompe load).

## Duas tentativas medidas-e-descartadas (não re-tente)
1. **Feather `pre_cover · ball_fraction`** (a shipada): É a própria saia — a fração cavalga o argmax alto-e-
   distante, então decai sobre a bola inteira e salta nas costuras.
2. **Fill opaco cru pela fonte pintada mais próxima:** matou a saia mas expôs a **coerência do argmax**
   (raios no blob) + um **crescente de cliff** onde altura e cobertura discordam. Daí `hbuf`.
3. **Enclausuramento angular** (soma de vetores unitários): **reprovado** — não separa flank-grosso de axila
   (ambos têm span > 180°); o convexo ainda crescia +9~12px. Fiddly = bug de design. O closing não tem
   threshold.

## Perf
`sculpt_perf_kill_criterion` (`#[ignore]`, manual): INFLATE **7,5 ms @2048 · 7,9 ms @4096**, sob o kill 8
(era ~7,2/7,7 antes; o closing custa ~0,3-0,7ms). O early-out zera o custo quando o `cr` é todo-tinta.

## Gates (mutação-testados)
- `the_convex_flank_is_preserved_while_the_armpit_fills` (junction) — Enio 2026-07-17, a propriedade que
  define o closing. Flanco convexo cresce ≤ 3px, axila enche.
- `the_convex_blob_is_domed_not_grown` (edge) — blob 100% convexo não cresce (só doma).
- **Mutação comum aos dois:** ignorar o closing (`* cfill[ci]` → `* 1.0`) ⇒ cresce `ρ` ⇒ RED nos dois
  (medido: blob 27→42, flanco +15).
- `the_armpit_fill_is_opaque_with_a_clean_edge` — nem saia (rampa larga) nem staircase (corte de 1 texel).
- `the_armpit_fill_carries_the_paints_colour` — testa o HUE (a luz sombreia côncavo ≠ plano).
- `the_inflate_fills_the_junctions_armpit_no_gash` — recalibrado ao **filete** (o closing enche o canto, não
  o quadrante inteiro; isso também ataca o *"erodido"*).
- Gates obsoletos removidos: os 2 de rim-feather do blob + os 2 de "traço reto engorda" (o convexo não cresce
  mais). A cobertura/cor migraram para `inflate_junction_probes` (onde está o fixture côncavo).

## Smoke
```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && cargo run -p ph2d-host-desktop --release
```
1. Pinte **dois traços grossos se cruzando** (impasto; o falloff já nasce Sphere agora).
2. SCULP → Inflate → **Filter Layer**.
3. Confira: (a) os braços **não engordam** (bordas convexas preservadas); (b) as axilas enchem com um filete
   liso (sem rasgo, sem escada, sem saia); (c) um blob solto **doma no lugar** sem espalhar tinta.

Render-and-look sem app: `PH2D_PUSH_LOOK_DIR=<dir> cargo test -p ph2d-host-desktop --release
probe_push_render_and_look -- --ignored` (cena 12 = a cruz; 10b = o blob).

## Verificação
tool-painter 711 verde · seam 40 · editor-core 753 + arch/LOC · workspace `check --all-targets` limpo ·
clippy 0. LOC: `sculpt_inflate` 308 · `sculpt_offset` 137 · `sculpt_close` 217 (cap 700).

## Aberto (mesmo backlog do §5, inalterado)
Passe de luz na GPU · relevo do papel (exige ordem do Enio) · a cura do BANCO · Conserve p/ Flatten/Fill ·
perf do Deform não gateada.
