# HANDOFF de CONTINUAÇÃO — `line/Painter` (2026-07-15)

> **Para o PRÓXIMO IMPLEMENTADOR da linha (cold start).** Substitui o
> [`HANDOFF_line_Painter_continuacao_2026-07-14.md`](HANDOFF_line_Painter_continuacao_2026-07-14.md)
> como documento vivo — aquele continua valendo para tudo que NÃO for o Inflate/Sculpt (leia-o para o
> resto da linha). Plano vivo do Sculpt: [`docs/Painter/18_plano_sculpt_relevo.md`](Painter/18_plano_sculpt_relevo.md).
>
> **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter` · **branch:** `line/Painter`.
> **Modo L** (ADR-0107): você fecha a linha, escreve o handoff e **PARA**. Integração/ship só por ordem
> EXPLÍCITA do Enio. Fast mode: `git commit --no-verify -- <seus paths>`, zero push/CI.
>
> **HEAD relevante:** `c8ea47de` (reach-cap do Inflate — a última coisa que landou; **NÃO reverta**, §2).

---

## 0. TL;DR

O **Inflate** (o Blob do Sculpt) teve 3 smokes do Enio. Os dois primeiros bugs (era Layer; não engordava)
e o do 3º (topo chato → virou o Blob) estão **fechados**. O 3º smoke abriu uma cauda: **"falloff de
influência retangular bizarra"** — um **quadrado duro** em volta do domo em tinta grossa. Eu matei a parte
GRANDE dele (a saia ilimitada da parábola, `c8ea47de`, §2). **Mas o Enio re-smokou e o quadrado AINDA
aparece** — menor, mais sutil, mas lá: *"ainda levanta área retangular abaixo do brush"* (screenshot no §1).

**Este handoff existe por causa desse bug (§1) + a fila (§3).** O item nº 1 é o quadrado residual.

---

## 1. 🔴 O BUG — o retângulo residual do Inflate (P0)

### O sintoma

Tinta **grossa** (blob acumulado por vários inflates), pincel **grande**. Inflar deixa uma **área
retangular** — axis-aligned, bordas duras — em volta/abaixo do domo. É um **degrau tonal**: o interior do
retângulo tem sombreamento diferente do resto do blob, com o domo circular dentro dele. Screenshot do Enio
(2026-07-15) mostra isso claramente no domo superior-esquerdo, com o retângulo bem maior que o domo.

### O que EU já fiz (e por que não bastou)

`c8ea47de` (§2) matou a **saia ilimitada da parábola** — o mecanismo do quadrado GRANDE (100+ px). O gate
`the_inflate_offset_reach_is_bounded_not_a_runaway_rectangle` prova que a saia de uma parábola sobre um `pre`
alto é cortada. **Ele passou.** E mesmo assim o retângulo persiste.

**Por que o gate não pegou o resíduo:** o fixture dele é **limpo demais** — um platô ISOLADO pequeno (raio
10), pincel PEQUENO (r=12), tela nua em volta, **um dab**. O repro real é **pincel GRANDE** sobre **tinta
grossa SOBREPOSTA**, **multi-traço**. *"Um gate só prova o que a fixture contém"*
([[feedback_a_gate_only_proves_what_its_fixture_contains]], [[feedback_harness_reproduces_mechanism_not_context]]).
**O primeiro passo é um gate/repro com os números do PRODUTO** (pincel ≥ 60 px, `pre` ≥ 10 loads e
sobreposto, ≥ 2 dabs).

### As DUAS hipóteses (e como distingui-las — faça isto ANTES de codar)

O retângulo lê como **sombreamento**, não como altura óbvia. Há dois mecanismos plausíveis e eles pedem
correções OPOSTAS, então **NÃO comece a corrigir antes de saber qual é** ([[feedback_render_and_look_when_a_green_gate_is_contradicted]]):

**(A) Resíduo de ALTURA** — um degrau real no buffer `heights`.
- *Candidato A1:* o **re-sample no aro** (§2) pra pincel GRANDE ainda levanta texels do `kr` perto do
  pincel — a posição re-cravada (ρ√2 na direção do argmax) cai DENTRO do disco elevado, então `rim` é alto e
  `value = rim − |Depth|` continua acima do `pre` nu. O disco é círculo, mas o `kr` é retângulo, e a borda
  onde `value` cruza `pre`… investigue se ela é o retângulo.
- *Candidato A2:* a **janela de restore** (`restamp_reset_sculpt` / `restore_sculpt_window`,
  `sculpt_session.rs`) não cobre a UNIÃO de tudo que dabs anteriores escreveram, então um levantamento
  anterior persiste como resíduo retangular e **assa no commit**. O pad de restore do Inflate já foi crescido
  uma vez (ghost-ring), mas pode ser insuficiente pro caminho do re-sample OU pro pincel grande.

**(B) Emenda de RELIGHT** — a altura é lisa, o degrau está só no SOMBREAMENTO.
- `render_inflate` termina com `mark_dirty(grow(moved,1))` (bbox retangular dos texels mudados) OU
  `mark_dirty(kr)` (o `kr` inteiro, no ramo `matter_ok` sem mudança de altura) + `invalidate_composite()` no
  ramo da matéria. **A luz NÃO é idempotente** (CLAUDE.md: *"o dirty-rect recompõe a região não-iluminada e
  ilumina aquilo"*). Se o relight sobre esse **retângulo** (bbox) deixa uma emenda tonal na borda — porque a
  advecção escreveu `rgba` direto (`sculpt_blur.rs:506`) e o relight sobre o bbox re-ilumina uma base que já
  estava iluminada, ou lê normais de vizinhos fora do bbox — o retângulo é **puramente de luz**, sem degrau
  de altura. Sítios: `sculpt_blur.rs:512-520` (`mark_dirty`), `:509` (`invalidate_composite`), e o consumidor
  do dirty-rect no relight (`impasto_light` + o caminho de `mark_dirty` no shell/`PainterTool`).

### O experimento que decide A vs B (faça primeiro)

1. **Rode com o Smoothness > 0** (o card do Inflate tem o slider Smooth 0..16). Se o retângulo **amacia** →
   é ALTURA (o Smooth borra `heights`). Se **não muda** → é LUZ.
2. **Instrumente o `mark_dirty`** em `render_inflate`: logue o rect. Se o rect logado == o retângulo visível
   → o caminho do dirty-rect/relight (B) é o suspeito nº 1.
3. **Dump do buffer `heights`** sobre a região (probe como os gates de `inflate.rs` fazem, `heights_of`):
   se a altura tem um degrau retangular → (A); se é lisa → (B).

> **Não confie no meu palpite.** Eu não consegui rodar o app vivo daqui, então não fechei A vs B. O
> screenshot *parece* B (degrau tonal, não relevo), mas *"levanta"* na fala do Enio sugere A. **Meça.**

### Onde o código está

| Coisa | Arquivo | Sítio |
|---|---|---|
| O render do Blob (altura + advecção + reach-cap + dirty) | `crates/ph2d-tool-painter/src/tool/paint/sculpt_blur.rs` | `render_inflate` (~344-521) |
| O motor separável (Felzenszwalb) + `pack_src`/`unpack_src` | `.../sculpt_offset.rs` | `blob_dilate`, `ParabolaScratch::transform` |
| Restore/freeze da sessão (4 planos) + pad do Inflate | `.../sculpt_session.rs` | `restore_sculpt_window`, `restamp_reset_sculpt`, `ensure_sculpt_session` |
| Gates do Inflate | `.../sculpt_tests/inflate.rs` | (o reach-cap é `the_inflate_offset_reach_is_bounded_not_a_runaway_rectangle`) |
| Luz / dirty-rect | `.../impasto_light.rs` + o consumidor de `mark_dirty` | (relight não-idempotente) |

### Repro (uma tela)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```

Pinte um blob grosso (vários traços sobrepostos), chip **SCULP** no rail, verbo **Inflate**, pincel
GRANDE, Depth alto, e infle sobre o blob. O retângulo aparece em volta do domo.

---

## 2. O que landou (`c8ea47de`) — NÃO reverta

**Reach-cap circular do Inflate.** A dilatação parabólica separável (o Blob) **não tem suporte**: uma fonte
de altura `H` levanta tudo num raio `√(H/a)`, que pra tinta grossa (`H` ~ 15-20 loads) é **100+ texels**,
recortado no retângulo `kr` = o quadrado GRANDE. Uma bola de raio ρ alcança ρ; a parábola não.

Fix (`render_inflate`, 1 `sqrt` por texel recortado): o **argmax composto já diz a distância** que a matéria
vencedora viajou, então `dx²+dy² > 2ρ²` (raio ρ√2, onde a parábola caiu o `|Depth|` inteiro e a bola de
verdade termina) **re-crava a fonte no aro da bola e lê a altura ali** (`rim − drop`, reconciliado com `pre`).
- **Circular** (limita `dx²+dy²`, não cada eixo → nunca desenha quadrado por conta própria).
- **Simétrico**: dilatação de tela nua cai em `pre`; erosão de sulco fino ainda alcança o ombro dentro de
  ρ√2 (um `fall-to-pre` cru MATARIA a erosão — a forma cavada cerca o texel; foi o gate da erosão que pegou).
- Em `pre` chato o clamp não muda nada. Perf intacto (**4,57 ms/move @4096**, kill 8).

Gate + mutação verificada (clamp off → probe a 35 px = 19,78 loads). **⚠️ Esse gate NÃO cobre o resíduo do
§1** (fixture limpa demais). Se você mexer no re-sample pro §1, **mantenha esse gate verde** — ele guarda a
saia grande.

---

## 3. A FILA DE IMPLEMENTAÇÃO (depois do §1)

Em ordem. O §1 é P0 e bloqueia declarar o Sculpt "smokado".

| # | Item | O que é | Onde |
|---|---|---|---|
| **P0** | **Retângulo residual do Inflate** | §1 acima | `sculpt_blur.rs` / relight |
| **P1** | **Smoke completo do Sculpt** | A linha inteira do impasto+sculpt foi INTEGRADA sem smoke (ver §0 do handoff 07-14). Rodar o roteiro de lá + o do §1. | roteiro `PH2D_IMPASTO_SMOKE=1` |
| **W4** | **Família ADVECTIVA** (Grab/Pinch/Nudge/Rotate/Thumb) | **NÃO construir motor novo.** Fazer o warp do Deform carregar `h`+`covers`+`mats`+RGBA junto ⇒ 5 pincéis de uma vez. A advecção do Inflate (matter loop em `render_inflate`) já é o padrão: o que chega num texel traz cobertura+material+cor de onde veio. | `18_plano_sculpt_relevo.md` §5; `sculpt_blur.rs` matter loop |
| **W5** | **Conserve (bow wave)** | O kernel do Scrape/Flatten **já computa** o volume deslocado (`height_push`, conservativo). Empilhar na borda da espátula = um **flag**, não motor novo. | `18_plano` §6 |
| **D** | **Bugs de DISPLAY (diferidos, precisam do app vivo)** | (a) relevo *anchored* some no pen-**up**; (b) relevo do *jitter* estica. **Provados corretos na TOOL** (buffers + composto iluminado certos antes/depois do up) — vivem no **pipeline GPU de preview do shell** (`render_loop/painter_gpu_preview`, o caminho GPU que difere pro CPU quando o impasto está visível). Precisa instrumentar o app real. | shell `render_loop` |

Contexto extra que pode ajudar no §1: o `matter_ok`/`invalidate_composite` no ramo da advecção
(`sculpt_blur.rs:509,516-519`) é o único lugar do Sculpt que escreve `canvas_rgba` direto E chama
`invalidate_composite` (composite inteiro, não rect) — se o §1 for (B), esse é o epicentro.

---

## 4. Invariantes da linha (não quebre)

- **Isolamento (Modo L):** edite a pasta do seu módulo (`ph2d-tool-painter`, `ph2d-painter-brush`,
  painéis). Foundational você PODE tocar sob o protocolo testado; contrato congelado (§6 do CLAUDE.md) exige
  ADR + PARE. O Sculpt **não** toca contrato (`cook.rs`/`NodeManifest`/`Tool` intactos).
- **Velocidade:** inner loop = `cargo check -p ph2d-tool-painter`. Teste/clippy 1× no fechamento.
- **Perf gate:** `sculpt_perf_kill_criterion` (`--release --ignored`), kill **8 ms/move**. Inflate hoje
  **4,57 ms**. Qualquer coisa O(ρ²) por texel estoura (a esfera exata era 73 ms — por isso o separável). Se
  o §1 exigir um filtro bounded de verdade, **meça antes** — o orçamento é apertado.
- **Gates:** toda correção precisa de um RED provado por mutação ([[feedback_mutate_the_code_not_just_the_test]]),
  e a fixture com os **números do produto** ([[feedback_test_with_product_numbers_not_convenient_ones]]).
- **UI:** zero hex / zero string hardcoded / labels em inglês (HR-15).
- **Git:** `git commit --no-verify -- <seus paths>`. **NÃO** integre nem pushe (Enio-only, ordem explícita).

---

## 5. Estado / commits

- `c8ea47de` fix(sculpt): Inflate — reach-cap circular contra a saia da parábola (§2).
- `83884063` feat(sculpt): Inflate virou o Blob (raio segue o falloff, separável O(N)).
- Suite `ph2d-tool-painter` **verde** (676 lib tests, 23 ignored), clippy limpo, LOC sob o teto.
- **Aberto:** §1 (P0) + a fila §3.
