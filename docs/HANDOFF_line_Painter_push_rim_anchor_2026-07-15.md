# HANDOFF — o aro do Push ancora no CÍRCULO do pincel, não no alpha do falloff (`line/Painter`, 2026-07-15)

> **Para o PRÓXIMO agente da linha.** O smoke do Enio REPROVOU o desenho do bow wave com o
> diagnóstico exato: *"é usada a circunferência do gizmo do brush para empurrar a massa e não o
> alpha do falloff"*. As duas screenshots mostram o colar de tinta empurrada como um **anel duro e
> perfeitamente circular colado no raio geométrico do pincel** (o anel branco do gizmo senta
> exatamente nele), enquanto a tinta do próprio traço tem borda MACIA que termina bem antes.
> A mecânica da onda (viajar com o bico, descansar na fronteira, ledger zero) está certa e gateada;
> **o que está errado é ONDE o aro nasce.** Não recomece a física — corrija a âncora.

## 0. Protocolo (não pule)

Modo L (worktree `Worktrees/line-Painter`, base `main = 12ccaecd` + 20+ commits da linha).
**Você NÃO integra, NÃO pusha, NÃO roda ship** — fecha, escreve handoff, PARA.
Fast mode: `git commit --no-verify -- <seus paths>`. Inner loop `cargo check -p`.
Mutações por caminho ABSOLUTO (o cwd reseta pro repo primário a cada turno — já mordeu 2×).
Restaure mutação por replace reverso com `assert old in s`, NUNCA `git checkout`.
Mutação-RED só vale sobre gate visto VERDE (ciclo verde→RED→verde).

## 1. O MECANISMO (leia isto antes de qualquer código)

O aro lateral (`bank_dab_push`) e o lóbulo da onda (`wave_lobe`), ambos em
`crates/ph2d-painter-brush/src/height_push.rs`, definem seu domínio assim:

```rust
let t = dab.footprint.falloff_t(dx * inv_radius, dy * inv_radius);
if t <= 1.0 { continue; }                       // dentro do disco: nada
let k = push_rim_weight((t - 1.0) * inv_reach); // perfil C¹ começando EM t = 1
```

`t = 1` é a **borda GEOMÉTRICA do dab** — a circunferência do gizmo. Mas o alpha da tinta
(`silhouette_at`) num falloff macio morre muito antes: o doc `16_impasto_plano_implementacao.md`
§14.1 tem os números medidos — **`W_TAIL` (0.35 de cobertura) cai em t = 0,61 no Smooth** e
t = 0,94 no Sphere. Ou seja: num pincel Smooth de raio 40, a tinta visível termina a ~24 px do
centro e o aro empurrado nasce a 40 px — **16 px de tela nua entre a tinta e um anel de massa com
borda interna perfeitamente circular**. É exatamente o colar das screenshots.

A MORDIDA está certa (ela pesa por `m = w × coverage`, alpha-weighted). Só a DEVOLUÇÃO ancora
errado. Com falloff **Constant** (borda dura), t=1 É a borda do alpha — por isso ninguém viu isso
nos fixtures de kernel (chão uniforme + falloff default) nem no gate de zonas: **as zonas medem
PARA ONDE o volume vai (frente/lado/trás), não ONDE a borda interna do aro nasce.** Gate verde,
contradito pelo olho — o pixel é o oráculo, e a fixture não continha o fenômeno (falloff macio).

## 2. A DIREÇÃO DE FIX (recomendada) — o aro nasce onde o CORPO termina

A regra da casa já existe e já resolveu o problema gêmeo uma vez (§14 do doc 16, o FILME):
*"um pincel que não deposita corpo não deposita tinta"* — o pigmento foi recuado pra onde a luz
dá sombra, usando `height_film`/`W_TAIL`. O aro precisa da MESMA lei, do outro lado:

> **A tinta desloca para onde o CORPO termina, não para onde o gizmo termina.**

Implementação candidata (porta única): um helper `rim_t0(spec, dab) -> f32` que devolve o `t` onde
a silhueta cruza o limiar do corpo (o mesmo `W_TAIL`/`body_profile` que o filme usa — se houver
duas portas para "onde a tinta termina", elas divergem; faça o aro PERGUNTAR à lei do filme).
O domínio do aro vira `t > t0` e o perfil `push_rim_weight((t - t0) * radius / reach)`, nos
DOIS sítios (`bank_dab_push` e `wave_lobe`) — extraia o domínio para UMA função compartilhada,
senão o aro lateral e a onda nascem em bordas diferentes.

Complicações reais que você vai encontrar:

- **`t0` depende do falloff, do hardness e da Shape image.** Para falloffs analíticos dá para
  resolver `silhouette(t) = limiar` por bisseção 1× por dab (barato). Para **Shape image**
  (silhueta por imagem) não existe t0 radial — o doc do `bank` já diz que stamp é stamp; decida e
  DOCUMENTE (sugestão: Shape image mantém t=1 — um carimbo tem borda dura por definição — e o gate
  pina só os falloffs analíticos).
- **`sweep_residual`/cápsula:** o t é avaliado no residual do sweep — t0 vale igual (o corpo varrido
  termina no mesmo t), mas confira com o gate de zonas que nada regrediu.
- **O reach:** hoje `push_reach_px(radius)` mede a partir de t=1. Recuar a âncora SEM recuar o
  alcance externo estica o aro; decida se `reach` conta a partir de t0 (aro de largura constante,
  recomendado) e meça o kink (MUT T história: aro estreito = espícula, 0.8×raio foi o fix).

## 3. ⚠️ O CONSERVE COMPARTILHA O MOTOR — e foi APROVADO no smoke

`bank_dab_push` é chamado pelo W5 (Scrape/Chisel + Conserve, `sculpt_session.rs`, share 0.0).
Mudar a âncora do aro MUDA o desenho do Conserve também. O pincel de sculpt default tem falloff
próprio — **verifique qual** e:
- se a mudança melhora o Conserve pela mesma lógica, re-smoke DECLARADO no handoff;
- se quiser blindar o aprovado, a âncora vira parâmetro (t0 vs 1.0) como o `forward_share` foi —
  mas a física diz que a lei certa vale pros dois; prefira UMA lei + re-smoke.

## 4. Estado do código (o que já está de pé e NÃO deve ser refeito)

- **A onda-escalar** (`2b44eaf2`): `DEPOSIT_FORWARD_SHARE = 0.6`; escalar por cópia de Symmetry em
  `relief_state::stroke_wave`; lóbulo posto/REMOVIDO por dab (remoção ANTES do depósito do dab
  tocar `stroke_paint` ⇒ pesos `(1−paint)` recomputam idênticos ⇒ negação bit-exata, livro único);
  no pen-up a onda descansa na fronteira. Zonas medidas: 42,8% à frente / 45,8% esteira / 6,9%
  swath / ledger 0 por passo.
- **1ª tentativa REPROVADA POR MEDIÇÃO** (não re-tente): bancar à frente por-dab + mordida re-lendo
  o plano ⇒ 0,4% chegava à frente, 16% fossilizava no swath — a mordida-envelope (`ground×Δm`) é
  saque único, não transporte.
- **Armadilhas pagas hoje** (cada uma custou um vermelho):
  - `sweep_axis` aponta PRA TRÁS (eixo da varredura) — o lóbulo usa o negativo.
  - **O bico é o último DAB, não o ponteiro** (stabilizer segura ~1 raio) — gates medem do
    `stroke_wave[..].1.center`.
  - Warm-up de heading: Push>0 é o 4º leitor de `Dab::dir` (derivado do spec em `stroke.rs`).
  - 1º dab banca ORIENTADO via prev sintético de `d.dir` (senão anel radial no pen-down).
  - Lei da cápsula (`fc96ef27`): sweep só com `dist ≤ min(r, r_prev)`.
- **Gates existentes** (todos verdes; rode-os depois do fix):
  `the_ploughed_paint_waits_at_the_strokes_frontier` + `the_wave_travels_with_the_tip`
  (kernel, `ph2d-painter-brush/src/height_tests.rs`) ·
  `the_deposits_wave_travels_through_the_real_stroke` (tool, `tests.rs`) ·
  os 5 `impasto_push_*` históricos · gates W5 do Conserve.
  **Eles NÃO pinam a âncora** — o gate NOVO que você deve escrever primeiro (red-first):
  *num falloff Smooth, a borda interna do aro (primeiro texel positivo do plano, varrendo do
  centro pra fora no perpendicular do traço) fica a ≤ ~2 px da borda do CORPO (t0), não em t=1* —
  com os números do §14.1 (Smooth t0≈0,61: a distância hoje é ~0,39×raio ⇒ nasce VERMELHO).
  E o irmão: **Constant fica byte-idêntico** (t0 = 1 lá — impressão digital antes/depois).

## 5. O INSTRUMENTO — renderize e olhe VOCÊ MESMO (foi o que destravou hoje)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_PUSH_LOOK_DIR=/tmp/push_look cargo test -p ph2d-host-desktop \
  probe_push_render_and_look -- --ignored
```
(`shells/desktop/src/render_loop/push_look_probe.rs` — 6 PNGs; leia-os com a tool de Read.)
**Acrescente uma cena com falloff SMOOTH explícito** — a sonda de hoje usa o default, que é
exatamente por que meu olho não viu o colar que o Enio viu (fixture sem o fenômeno, DE NOVO).
Compare com o Scrape+Conserve (cena 6, o visual aprovado).

Smoke final (Enio): `PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop` → tinta
grossa → pincel MACIO menor, Push alto → atravessar e parar no meio. O certo: o aro/onda nascendo
da borda MACIA da tinta (sem tela nua entre tinta e colar, sem círculo perfeito).

## 6. Fechamento

Perf: kill gates isolados (`--test-threads=1`): sculpt 3,3-3,8; IMPASTO 4,24/4,86 (kill 8; a
bisseção do t0 por dab não pode pesar — meça). Suites: 696 tool + 252 brush + workspace.
Clippy 0. `impasto.rs` está a 496 LOC (split `impasto_live.rs` recém-feito); `height_push.rs` 224+.
Commits do dia (contexto): `8b21acb8` P0 · `cac2db77` W4 · `b9d0ef28` W5 · `923ba951` display ·
`fc96ef27` cápsula · `63e7cf2f` raio+undo · `2b44eaf2` bow wave · `4615ba38` split.
Ao fechar: handoff novo + atualizar este e o `HANDOFF_line_Painter_integracao_2026-07-15.md` + a
entrada Painter do CLAUDE.md §5. **PARE e espere a ordem do Enio.**
