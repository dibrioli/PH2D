# HANDOFF de continuação — line/UIUX, onda F33 (filtros × efeitos), 26/10

> Linha NÃO integrada. Base: `main` c3662cd8e4 (não andou). Ponta: ver `git log -1` (F33 = os 6 commits
> depois de `3b981486f0`). **Smoke do dono 26/10: OK.** Formato inalterado nesta onda
> (`PROJECT_SCHEMA` 197, tripla `(197, 13, 24)` — a mesma da F32).

## 1 — Commits da F33 (`git log --oneline 3b981486f0..HEAD`)

| commit | o quê |
|---|---|
| `d8e54627bf` | **a cura**: `ph2d-vec-scene/src/fx_resample.rs` (porta nova) + Warp ×9 / Twist / Zig Zag / Sketch por ela; `ArcPath::anchor_sides`; gates `effect_geometry_tests.rs`; plano `F33_plano.md` |
| `1418d62f1f` | a Nuvem DO APP nas réguas; sonda de placa `ph2d-render/tests/it/fx_efeito_sob_filtro_gpu.rs` (dev-dep `ph2d-vec-scene` na `ph2d-render`, sem ciclo); cena `PH2D_OBJECT_MODE_SMOKE=40` |
| `4daf3888ca` | fmt + clippy |
| `4326f57df3` | a cena 40 nomeia os efeitos pelo tipo (HR-15) |
| `82d91b2803` | foto da cena 40 |

## 2 — Mecanismo e números medidos (detalhe: [`F33_plano.md`](../F33_plano.md))

- Um filtro vê a forma COZIDA: raster pelo Vello + silhueta exacta em segmentos (`silhouette.rs`,
  `FLATTEN_TOL 0,02`); o Bevel lê a DIRECÇÃO do campo. A caixa do filtro já seguia o efeito (medido no
  código: `path_screen_bounds` → `cooked()`).
- O defeito era o reamostrador partilhado (`resample_displace`): alças Catmull-Rom `(next−prev)/6`
  comiam as quinas e faziam laços entre amostras coladas (razão de vizinhos `5,8e-2`). Com
  deslocamento NULO já dava 3 recuos e 0/6 quinas. **Nuvem do app + Flag (código antigo): 0/11 quinas
  dos vales, 6 segmentos que recuam → cura: 11/11, 0.**
- Cura: amostra sabe se é âncora (ganha a fusão); duas tangentes exactas na âncora (Corner se
  diferem); alças de Hermite com o vão de cada lado / 3; nos campos a tangente TRANSPORTADA (`J·t`,
  diferença central, Falloff incluído); acima do tecto caem amostras da grade, nunca âncoras.
- Gates (`effect_geometry_tests`, todo o catálogo × nuvem / Nuvem do app / círculo):
  `no_effect_folds_a_segment_back`, `a_continuous_deformer_keeps_every_corner` (quina = qualquer
  quebra `1e-6` rad: o Twist leva 9,5° a 3,5°), `a_smooth_vertex_out_of_an_effect_is_g1`. Vermelhos no
  código antigo; 4 mutações sobre a cura, as 4 vermelhas.
- Zig Zag: A/B 5 de 6 casos byte a byte; só `open_corner` Smooth, só nas 2 quinas (impressão re-pinada
  com a tabela). Warp: o teste antigo exigia tudo `Smooth` (era o defeito).
- A sonda de imagem por «vincos» NÃO separa antes/depois (o laço é um gancho, a quina preservada vinca
  por direito) — fica SONDA; o oráculo é a foto [`img_F33/vale_flag_antes_depois_controlo.png`](img_F33/vale_flag_antes_depois_controlo.png).
- Suíte do diff (`nextest-impacted`, 22 327): 2 vermelhos — o censo HR-15 (curado em `4326f57df3`) e
  `a_cena_3_conduzida_passo_a_passo` (code-scenes, loadavg ~90; verde sozinho a load 7,9).

## 3 — Superfície de colisão (DELTA)

- `ph2d-vec-scene`: `fx_resample.rs` (novo, `pub(crate)`), `fx_warp_presets.rs`, `fx_zigzag.rs`,
  `fx_sketch.rs`, `arc_path.rs` (+`anchor_sides`), `effect.rs` (+mod de teste), `lib.rs` (+mod).
  Quem consome `resample_displace`: Warp e Twist (`fx_twist.rs` intocado).
- `ph2d-render/Cargo.toml`: dev-dep `ph2d-vec-scene` (+`Cargo.lock`); `tests/it/main.rs` +1 mod.
- `ph2d-app-vec`: cena 40 = estágio **45** (`vector_mode_filtro_efeito.rs`), `FAIXAS` 14 → 15,
  `handles_first(26)`, `scenes.rs` 9..=40. Contador de cena: `PH2D_OBJECT_MODE_SMOKE` vai até **40**.
- Shell intocada (tecto 189041).

## 4 — Smoke do dono (26/10, OK)

`cd …/Worktrees/line-UIUX && PH2D_OBJECT_MODE_SMOKE=40 ./target/smoke/ph2d-host-desktop` — seis Nuvens
com Bevel + Drop Shadow (controlo, Flag com os números do dono, Wave, Twist, Zig Zag, Sketch). Foto:
[`img_F33/cena40.png`](img_F33/cena40.png). ⚠️ A cena pede a seleção do Flag pela caneta mas o painel
diz «No selection» — o dono clica na lista.

## 5 — Decisões (não reabrir)

- A régua é da GEOMETRIA cozida; a imagem é foto (a contagem de vincos foi medida e recusada como gate).
- Zig Zag / Sketch num vale côncavo: os pontos deslocados pela normal cruzam-se (corda a 124° da alça)
  — lei do efeito, não da reamostragem; fora do G1 com a medição. Pucker & Bloat: pétalas com bico são
  o efeito.

## 6 — ABERTO, por ordem

1. ⭐ **Achado do dono no smoke F33 (26/10): zoom muito perto numa forma vetorial com filtro → a forma
   SOME e o FPS cai** («parece ligado ao Bevel; outros filtros não testados»). Hipótese a MEDIR antes de
   curar (só lida no código): o job de FX dimensiona a textura pela caixa da forma INTEIRA no ecrã
   (`ph2d-app-vec/src/fx_live_memo.rs:159-166`, `job_for`), sem recorte pela VISTA; ao passar de
   `MAX_FX_SIDE = 8192` (`fx_live_memo.rs:56`) o lado é cortado (`.min`) e a imagem de FX — que TOMA O
   LUGAR do desenho — cobre só o canto superior-esquerdo da forma (o resto some). O custo do passe de
   campo é `O(texels × segmentos)` (`silhouette.rs` doc: 3,94 ms a 512² com 4096 segmentos) — a 8192²
   são 256× os texels ⇒ a queda de FPS. A família: TODO filtro (o recorte é do job, não do Bevel; o
   Bevel é o que mais custa no campo) e talvez o Motion/camada (`ph2d-fx-cells`). Cura provável: recortar
   o job à vista + o alcance da pilha (`stack_reach`) e semear o campo só com os segmentos que alcançam
   a janela. Régua: forma a zoom crescente (1×…64×), cobertura desenhada vs esperada e ms do recook.
2. Seleção pela cena 40 não acende o painel (§4) — ver se `vec.pen.select_many` é a porta certa.
3. Zig Zag / Sketch no vale côncavo (§5) — decisão de desenho do dono, com o oráculo a correr.
4. Da F32 §6: 2 (pose dos estados alinha por índice), 3 (chave sobre `from`/`to` de uma seta), 4 (F31/F30).
5. `CLAUDE.md` §5 no fecho/integração: `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..40`, `PH2D_AUTOKEY_SMOKE=1..36`.
6. A suíte inteira antes da integração (do integrador). **NÃO integrar nem fazer push sem ordem do Enio.**
