# Exame da família `aguarela` (J4–J5, catálogo I — aguarela e sumi-e)

Receitas: `crates/ph2d-tool-painter/tests/it/familias/aguarela.rs` (`WET`; os reprovados em `EM_ESTUDO`, sem ficheiro);
portões em `familias/aguarela/{portoes,molhado}.rs`; estudos `familias/aguarela/estudo.rs` (J5: lê também as receitas do
guache e do óleo). Pranchas `pranchas/aguarela/` (+ `cegas/`, `chaves/`). Oráculo: Rebelle `rebelle_bateria/WATERCOLOR/`.
⚠️ **O histórico da J4 (I-1…I-13, o motor do filme, as três rondas, as recusas medidas) está em `96868e656e`**
(`git show 96868e656e:docs/Painter/Pinceis/exames/aguarela.md`) — consulte as recusas lá antes de propor uma cura.

**Estado (J5, 22/10):** aprovados nenhum · curados não aprovados **Wet Liner, Wet Wash** (com a lei nova), Wet Flat,
Wet Round, Wet Drip, **Wet Dry Brush (fica — não se funde, abaixo)** · reprovados em estudo Mop, Bristle Watercolor,
Sumi-e, Granulation, Fringe Bloom, Spatter Watercolor, Optical Wash.

## Motor J5 — molhado × molhado (`Knob::WetOpen` + `Knob::WetOpenPush`, `crates/ph2d-wet-paint/src/trail/molhado.rs`)

**A medição** (cena `sobre`, foto A; régua `blot.py` do scratchpad, a mesma conta no portão `mancha()`): a largura da
mancha no meio da faixa molhada ÷ a do vertical no papel, e a escuridão do miolo ÷ a do azul no papel.

| | Rebelle | nosso ANTES (parede) | sem parede | **nosso DEPOIS** |
|---|---|---|---|---|
| Liner | 1,42 · 1,06 · [98,123,175] | 1,00 · 1,27 · [75,97,155] | 1,16 · 1,24 | **1,51 · 1,06 · [106,124,173]** (0,42 · 0,5) |
| Wash Round | 1,17 · 1,30 | 1,00 · 1,84 | 1,07 · 1,78 | 1,39 · 1,40 (0,15 · 0,4; a régua conta o halo) |
| Mop | 1,44 · 1,06 | 0,98 · 1,68 (estudo) | — | 1,75 · 1,13 (0,42 · 0,8) |
| Gouache Round | 1,25 · 1,00 | 1,27 · 1,11 (anel 1,14) | — | não serve (abaixo) |

A composição NÃO o exprimia (medido: `extDiffusion` 0,25, `extBackrun` 2, as duas, sem a parede — o quadrado fica).

**A lei** (desligada por omissão; os presets que não a ligam ficam ao bit — fingerprint `ph2d-wet-paint` e
`a_lei_da_tinta` verdes): (1) a cada dab a sonda olha `r + 2` à frente da pegada; na 1.ª vez que cai na tinta molhada de
OUTRA cor (`outra_tinta`), mede pela linha do traço a faixa `[ta, tb]` e guarda o disco no meio dela, de raio
`min(meia faixa, r·(1 + WetOpen))`; e empurra pela normal a fracção `WetOpenPush·(1 − (s/0,8·meia)²)` da tinta de baixo
para a orla molhada fora do disco (um destino por píxel até 2 × o raio do disco, pesos em triângulo, bilinear); (2) cada janela que
pousa dentro do disco estica o seu pigmento e a água pela normal (8 destinos, cauda em triângulo, tecto 1,3 × a meia).
⛔ **Recusas medidas nesta janela:** destinos discretos da orla (anéis concêntricos); orla de 1–4 px (crescente escuro);
esticar sem tecto (o Mop: um olho de 95 px contra 59); pesos em trapézio até ao disco (núcleo rectangular, tampas
pálidas — o pouso é pela opacidade da massa que chega); esticar na GRADE depois de o traço passar (pela massa o vermelho
pesado engole o azul: 1,05; pela opacidade 1,30); deixar a água no lugar (manchas e contorno).
**Portão** `familias::aguarela::molhado::o_azul_abre_uma_mancha_redonda_no_vermelho_molhado` (o Wet Liner de fábrica;
controlo `wetOpen` 0 — < 1,1 e escuro > 1,18). Red-first e mutação: M0 lei desligada → VERMELHO (mancha), M1 sem
empurrão → VERMELHO (escuro somado), M2 sem esticão → VERMELHO (mancha); restaurado, verde. Custo: Liner p95 4,67 ms ·
Wash 5,90 ms a 200 px, RSS ≤ 100 MiB (load 11–5; smoke).

## Os pincéis (ronda do motor novo; revisores cegos `sonnet`, um por ronda)

- **Wet Liner** — receita J4 + `MANCHA_DO_LINER` (0,42 · 0,5). Régua 419 → **430/454**; vizinho Hard Round **0,725**
  (≥ 0,596). Revisor m1 (`cegas/wet_liner_j5.png`): tomou o NOSSO pelo original nos gestos (baixa), achou o Rebelle nos
  tamanhos e no «sobre» (4/6) e lista: a 200 px a borda difusa e o miolo carregado dele contra o nosso chapado de borda
  dura (FILA 2), os anéis das voltas da espiral, a nossa mancha do cruzamento mais escura, redonda e com halo (a dele mais
  clara, de cantos quadrados), o vermelho seco mais denso. Já não lista o arrasto nem o quadrado. ⇒ **CURADO, NÃO
  APROVADO** (a lei fica na receita: a régua e o cruzamento melhoraram).
- **Wet Wash** — receita J4 + `MANCHA_DO_WASH` (0,15 · 0,4). Régua 373 → **384/454**; vizinho Glaze Round 1,16. Revisor
  m2 (`cegas/wet_wash_j5.png`): o Rebelle 5/6 (baixa/média): o nosso contorno escuro (orla 100), a rampa que entra tarde
  (decisão 9), a nossa mancha nítida em losango contra a difusa dele, as pontas em cápsula. ⇒ **CURADO, NÃO APROVADO**.
- **Mop** (estudo; receita + 0,42 · 0,8) — régua 390 → 383/454; vizinho Wet Wash 0,797. Revisor m3 (`cegas/mop_j5.png`):
  o nosso tomado pelo original nos gestos e nos tamanhos (3/6); o Rebelle no «sobre» (alta): a nossa mancha redonda e
  escura de aro nítido («um carimbo») contra o véu difuso dele que arrasta o vermelho; a faixa vermelha dele manchada.
  ⇒ **REPROVADO ainda** (a lei NÃO entra no `EM_ESTUDO`: a régua desce).
- **Wet Dry Brush → NÃO se funde no Wet Flat.** Medido na matriz: o 1.º vizinho é o Airbrush a **1,236** (Wet Flat
  longe; nenhum pincel da biblioteca o pinta); só a CURA para o Rebelle `Flat Dry` (308/454) virava o Wet Flat (F1 6/6).
  Fundir tiraria o único pincel de riscos soltos e mexia em 10 ficheiros partilhados. Fica como estava, curado não
  aprovado contra a origem (régua 220/454), com a razão de uso: o pincel seco de riscos soltos.
- Flat Bristle (óleo): ver `exames/oleo_acrilica.md` — a lei não se aplica (o Rebelle `Flat` não abre mancha, 0,86).

## Para a fila (coordenador)

1. **A borda da mancha** (Liner, Wash, Mop): a nossa é redonda e de aro nítido com um halo escuro (anel 1,09–1,25 contra
   1,04); a do Rebelle esbate em ~10 px e no Liner tem cantos quase quadrados. O aro nasce na fronteira entre a tinta
   esticada e a empurrada (a parede e o pouso pela opacidade) — precisa de um degradê de mistura na fronteira, não de
   valores (variadas a borda do empurrão 0,35–0,95, a orla 0,35–2 r, a cauda: o aro fica).
2. **O traço de 200 px do filme** (`Knob::StrokeFilm`): pontas e borda em degradê, miolo carregado (Liner, Wash, Mop).
3. As voltas da espiral do Liner; o recto de 5 px do Mop; o Sumi-e (cerdas em pente); granulação; franja; escorrido.
4. O Wet Round com pigmento 125 (régua 297 → 419): só com o `painter_default_knobs` e o controlo das MIDIAS.
