# Estudo — a mistura do Watercolor Basic trazida ao Digital, no lugar do Color mixing (07/11)

> Pedido do dono (07/11): *«a mescla no modo watercolor com o Watercolor Basic […] me parece muito superior ao do
> seu novo Color mixing. Estude […] e veja se o modo como ele faz pode ser trazido para o digital»*. Dado novo do
> dono: o Watercolor Basic com **Mixing → Pigment** dá azul + amarelo = verde real, «ótimo resultado».
> ⛔ Regra do dono: o Watercolor Basic **não pode mudar** (bytes). Só estudo: nenhum código de produto mudou.
> [M] = medido nesta sessão · [D] = deduzido do código. Sonda: `crates/ph2d-tool-painter/tests/it/sonda_watercolor_no_digital.rs`
> (`#[ignore]`). Imagem: [estudo_watercolor_no_digital.png](handoffs/material_2026-11-07/estudo_watercolor_no_digital.png).

## 1. O Watercolor Basic — o que importa para a mistura

Preset `assets/brushes/Watercolor/Watercolor/Basic/Watercolor Basic.ph2d-brush.png` (receita de fábrica
`state_default.rs:85` `aquarela_de_fabrica`; `familias/aguarela.rs:589`). Lido do ficheiro [M]:

| campo | valor | papel na mistura |
|---|---|---|
| `hardness` | **0,0** | ponta macia (o G-Pen é 1,0) |
| `spacing` | 0,1 | |
| `fill` · `depth` · `opacity` | **0,25 · 2,0 · 0,4** | aguada FINA e TRANSLÚCIDA: o que está por baixo continua a ver-se |
| `wet_charge` · `wet_pull` | 0,5 · 0,6 | o pincel apanha metade da cor de baixo e arrasta-a |
| `wet_rewet` · `wet_smudge` · `wet_dilution` | 0,5 · 0,5 · 0,2 | dissolve / arrasta a tinta de baixo |
| `pigment` · `pigment_mix` | **false** · 0,5 | o slider «Mixing → Pigment» do dono (desligado de fábrica) |
| `edge_gain` · `granulation` · `warp` | 0,2 · 0,3 · 12 | orla, grão, contorno orgânico (só aspecto) |

## 2. Como o Watercolor mistura — e PORQUÊ parece contínuo

1. **Não há dab depositado.** No meio Watercolor o carimbo normal é saltado (`stamp_route.rs:177`, antes de
   qualquer outra rota): o traço só acumula uma **máscara de cobertura** (discos em MAX, nunca empilhados —
   `watercolor_accum.rs:330`) e um buffer de cor; a aparência é **reconstruída por píxel, num campo só**, sobre a
   tela congelada no pen-down (`watercolor_render.rs:1-19`). ⇒ não existe «um dab por cima do outro»: não há
   grumos nem anéis por construção [D].
2. **A cor final é óptica, por PÍXEL, contra o píxel de baixo**: `out = base·T + pigmento·(1−T)`, `T = pigmento^(D·depth)`
   (Beer–Lambert, `watercolor_render/aparencia.rs:56-66`). Com `fill 0,25` a aguada é fina: o vermelho de baixo
   ATRAVESSA o azul e o resultado segue o vermelho píxel a píxel — o gradiente vem da própria tinta de baixo [D].
3. **Mistura de pigmento (o slider Pigment)**: com `pigment` ligado, o mesmo píxel é puxado para
   `ph2d_pigment::mix_unit(base, pigmento, film_a)` (Kubelka–Munk, `ph2d-pigment/src/lib.rs:160`) pelo peso
   `pigment_mix × presença de tinta` (`watercolor_render.rs:556-566`, `aparencia.rs:69-92`;
   `watercolor_mistura.rs:47` — o papel não é pigmento). Molhado sobre molhado: `watercolor_mistura.rs` (a fracção
   é `aₚ/(aₚ+aₐ)`, satura, não lava). Lei espectral opcional: `pigment_espectral` (`spec.rs:467`, `espectral.rs`).
4. **Apanha e arrasto** (`watercolor_mixer.rs:130-207`): reservatório por traço em **absorvância** (subtractivo),
   carrega depressa e descarrega devagar (`RETAIN_LOAD` / `unload` por Pull); amostra a tela CONGELADA (sem se
   alimentar de si mesmo); peso pela presença de tinta.
5. **Rewet / Smudge** dissolvem e arrastam a tinta de baixo por píxel (`watercolor_mistura_agua.rs`,
   `watercolor_mistura_arrasto.rs`, `watercolor_smudge.rs`).

**Em resumo [M+D]:** a continuidade vem de (a) **translucidez** (fill 0,25 / opacity 0,4 — o de baixo vê-se),
(b) **campo único por traço** (cobertura em MAX, composta 1×), (c) **cor calculada por píxel contra o píxel de
baixo**, (d) **mistura subtractiva** (Beer–Lambert sempre; K–M com o Pigment). Medido: azul sobre amarelo, centro
`150,168,176` (cinza-azulado) sem Pigment → `153,171,123` (verde) com Pigment 1 [M, painéis 12 e 7].

## 3. Porque o Color mixing novo sai pior

`ph2d-painter-brush/src/mistura_opaca.rs:319-505`, por dab:
- **Uma cor por dab:** amostra a MÉDIA do disco inteiro (`:362-394`) e deposita essa cor única no disco todo
  (`:426-460`). Sobre uma fronteira vermelho/branco o disco leva a média — a cor salta dab a dab; com a ponta dura
  (G-Pen, hardness 1) cada disco tem borda franca ⇒ manchas por dab (a foto 2 do dono). O `perfil_da_cor` (`:117`)
  só disfarça [D].
- **Mistura em RGB linear de bytes sRGB** (`:407`, `:428`): azul + amarelo dá **cinza-violeta** `120,105,148`, nunca
  verde [M, painel 8]. Azul + vermelho dá um roxo escuro `104,34,148` [M, painel 2].
- **Opaco:** `density 1` cobre a tela — o que estava em baixo some debaixo da mistura; só a cor do reservatório leva
  a memória do vermelho.
- **O Pigment do Digital não ajuda com ponta opaca:** `blend_over_pigment` (`blend.rs:206-236`) mistura K–M por
  píxel mas pela cobertura `a` do dab; com `a = 1` o alvo K–M É a cor do pincel ⇒ azul puro `0,0,220` em cima do
  vermelho e do amarelo [M, painéis 3 e 9]. O slider «Mixing → Pigment» já existe no Digital
  (`paint_pigment.rs:84`, oferecido por `offers_pigment_mixing_in`), mas só se vê em pincéis de baixa opacidade.

## 4. Dá para trazer o modo do Watercolor ao Digital?

**Tentativa barata — o meio Watercolor com a transparência desligada** (fill 1, depth 4, opacity 1, orla/grão/warp
0, os knobs de mistura do Basic): **REPROVADA [M]** (painéis 4, 5, 10, 11). Beer–Lambert só subtrai: aguada
espessa = **vidrado escuro** (`75,1,5` sobre vermelho; `89,77,9` sobre amarelo), não tinta opaca; o «Opacity»
(body) não a torna opaca. O caminho do Watercolor é óptico por natureza — não é reutilizável «com a transparência
desligada» [M].

**O que é portátil (compatível com tinta opaca) [D]:**

| peça do Watercolor | no Digital? |
|---|---|
| Mistura K–M / espectral (`mix_unit`, `blend_over_espectral`) | **sim** — já é do Digital (Pigment) |
| Cor por PÍXEL contra a tela de antes do traço | **sim** — a rota já guarda a tela do pen-down (`stamp_route/mistura.rs` `Ladrilhos`) |
| Campo único por traço (cobertura em MAX, sem empilhar) | **sim** — o tecto por traço (`stroke_mask`) já existe na rota (`mistura_opaca_dab_na_pegada_com_tecto`) |
| Reservatório subtractivo, carga rápida / descarga lenta, peso por presença de tinta | **sim** — aritmética de `watercolor_mixer.rs:150-187` |
| Translucidez, Beer–Lambert, orla, granulação, warp, rewet/bloom do papel | **não** — são a aguada |

**Proposta — «Color mixing: Off · Pigment mix» (substitui Blend/Running):** por píxel `p` da pegada,
`out(p) = mix_unit(antes(p), cor_levada, k(p))`, com `antes` = tela do pen-down nesse píxel (não a média do disco),
`cor_levada` = mistura K–M do pincel com o reservatório (apanha presença-pesada, carga rápida/descarga lenta à
`watercolor_mixer`), `k(p)` = cobertura do TRAÇO (tecto, MAX) × Paint amount; a lei espectral se `pigment_espectral`.
Painel: **Color mixing** = `Off · Pigment mix`; knobs **Amount** (quanta tinta nova), **Pickup** (= 1−Charge),
**Pull** (comprimento do arrasto), **Density**; a lei K–M/espectral herdada do slider Pigment. O que dá a continuidade
do Basic: a cor sai de `antes(p)` píxel a píxel (o gradiente é o da tinta de baixo) e a mistura é subtractiva.

**Watercolor Basic intocado — porquê e como provar:**
- [D] A rota da aguada sai em `stamp_route.rs:177` antes da mistura opaca (`:520`); `mistura_opaca.rs` e
  `stamp_route/mistura.rs` não são lidos pelo meio Watercolor. Mexer só nesses dois ficheiros + o painel do Digital
  não toca a aguada. ⛔ Não tocar `blend.rs::blend_over_pigment`, `ph2d-pigment`, `espectral.rs` nem `watercolor_*`
  (partilhados: a aguada lê o `mix_unit`).
- Prova: por nome `a_largura_de_fabrica_e_a_aprovada` e `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` (as dos
  dois Watercolor verdes = mesmo byte) + um teste novo «o traço do Watercolor Basic dá o mesmo hash antes/depois»
  com esta cena (painéis 1 e 7), e `Off` = rota de sempre ao bit (o teste que já existe para a mistura opaca).

**Custo [M]** (perfil `smoke`, traço de 280 px a 40 px, 60 eventos, mínimo de 3, intercalado — relativo, não
absoluto): Color mixing **~20 ms**/traço · Watercolor Basic **~263–275 ms** · meio Watercolor «opaco» ~105 ms.
K–M por píxel (`mix_unit` em `f64`) na pegada a spacing 0,1 deve somar ~3–10× ao Color mixing [D, não medido];
continua muito abaixo da aguada. Medir na implementação (`MEDIR_VELOCIDADE.md`).

**Riscos:** (1) K–M sobre o papel branco tinge (o `blend_over_pigment` resolve com `PAPEL_DO_PIGMENTO`; o Watercolor
com a presença — copiar a regra); (2) `mix_unit` em `f64` por píxel: custo e o «mesmo byte em qualquer máquina» —
usar a mesma porta, sem transcendentais novos; (3) o painel antigo (Blend/Running) já gravado em presets do dia
(`a_mistura_opaca_nos_presets.rs`) — migrar `Blend`/`Running` → `Pigment mix`; (4) a translucidez é metade da beleza
da foto 1: o Digital opaco nunca vai ser igual — o dono deve comparar com `Density < 1`.

## 5. Recomendação (passos)

1. Dono vê a imagem e decide: **substituir Blend/Running por «Pigment mix»** no Digital (recomendado).
2. `mistura_opaca.rs`: trocar a cor-por-dab pela cor-por-píxel `mix_unit(antes(p), levada, k(p))`; reservatório em
   K–M com carga/descarga assimétrica e peso por presença (copiar a aritmética, não chamar o código da aguada).
3. Painel: `Color mixing: Off · Pigment mix` + Amount · Pickup · Pull · Density; ajuda em inglês simples.
4. Prova: hash do Watercolor Basic igual (painéis 1/7), alarmes de largura e miniatura por nome, `Off` ao bit, e esta
   sonda com o painel novo ao lado dos painéis 1, 2, 7 e 8 (azul+amarelo tem de dar VERDE sem grumos).
5. Smoke do dono: G-Pen e Round, azul sobre amarelo e sobre vermelho, com Density 1 e 0,6.

## Painéis da imagem

1 Watercolor Basic (vermelho) · 2 G-Pen Color mixing · 3 G-Pen + Pigment 1 · 4/5 meio Watercolor «opaco» duro/macio ·
6 Basic sobre G-Pen opaco · 7 Basic + Pigment 1 (amarelo — o «ótimo» do dono) · 8 Color mixing (amarelo) ·
9 G-Pen + Pigment 1 (amarelo) · 10/11 Watercolor «opaco» + Pigment 1 · 12 Basic sem Pigment (amarelo).
Nota [M]: nesta cena (rato, pressão 1) o Color mixing sai liso, não em grumos; os grumos da foto 2 vêm da ponta dura
com a pressão a mudar o tamanho/cor dab a dab — a sonda não os reproduz, e a falha que ela mede é a da COR (RGB).
