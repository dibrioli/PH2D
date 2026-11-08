# Estudo — a mescla ESPACIAL do Watercolor Basic e o «Digital wet blend» (08/11)

> Dono (08/11), depois do «Pigment mix» do Digital: *«O que eu gostaria de ter trazido do watercolor era o modo como
> ele mescla as tintas, o smear, o blur, etc. não apenas a cor.»* E: *«mesmo com tinta seca, com as configurações do
> Watercolor basic, a mescla fica muito boa»* ⇒ o que importa age DENTRO de um traço sobre a tela do pen-down.
> Só estudo: nenhum código de produto mudou. [M] medido nesta sessão · [D] deduzido do código.
> Sonda `#[ignore]`: `crates/ph2d-tool-painter/tests/it/sonda_mescla_espacial.rs` (`PH2D_MESCLA_DIR=<pasta>`).
> Folha: [ablacao_watercolor_basic.png](handoffs/material_2026-11-07/ablacao_watercolor_basic.png) (rótulos na própria imagem).
> Antecessor (a COR): [ESTUDO_MISTURA_DO_WATERCOLOR_PARA_O_DIGITAL_2026-11-07.md](ESTUDO_MISTURA_DO_WATERCOLOR_PARA_O_DIGITAL_2026-11-07.md).

## 1. Cena e réguas

- **Azul SECO:** mancha azul pintada com o Watercolor Basic, `run_full()`, e esses píxeis entram como a tela de uma
  ferramenta NOVA (`set_source`) — não há mapa de molhado nem sessão: «seco» = só píxeis na tela do pen-down. (No
  código, «molhado» é o `canvas_wet` por píxel, seco por `dry_canvas_wet` em `on_tick`, `watercolor_dry.rs:23`,
  chamado em `stroke_lifecycle.rs:386`; e a sessão molhada `wet_session_base` + `wet_mistura.ha_tinta_da_sessao`.)
- Depois **4 traços amarelos paralelos** (y 80..125, passo 15, ponta 40 → sobrepõem-se) do branco para dentro do azul,
  e um **5.º em diagonal**; corrida «mouse» (pressão 1) e «pressure» (o 5.º com 0,2 → 1 → 0,2). Os 5 amarelos são
  traços separados na mesma ferramenta (sessão molhada ENTRE eles; o azul está seco).
- **Largura** = distância 10 %→90 % do canal R ao longo do traço, do amarelo à mancha (y 95 e 110, média): quanto o
  traço ARRASTA/funde a cor. **Degrau** = maior diferença entre vizinhos no miolo (x 170..280, y 75..130, longe da
  orla seca). **Placas** = % de píxeis do miolo com degrau > 24.

## 2. Ablação [M] (Watercolor Basic de fábrica; UM ingrediente desligado por vez; perfil `smoke`)

| # | variante | largura px | degrau máx | placas | leitura da foto |
|---|---|---|---|---|---|
| 01 | **Watercolor Basic** | **84** | 11 | 0 % | o amarelo PUXA o azul em estrias macias para a esquerda; contínuo |
| 02 | Smudge 0 | **6** | 10 | 0 % | as estrias SOMEM: véu cinza-azulado plano, sem arrasto |
| 03 | Charge 1 (sem pickup/pull) | 68 | 20 | 0 % | estrias ficam; cor mais verde/amarela (o reservatório levava azul) |
| 04 | Rewet 0 | 81 | 12 | 0 % | escuro/oliva, vidrado: o azul deixa de ser levantado/dissolvido |
| 05 | Dilution 0 | 84 | 16 | 0 % | ≈ fábrica |
| 06 | Spread 1 (blur do dissolve) | 84 | 7 | 0 % | ≈ fábrica |
| 07 | Edge 0 | 82 | 11 | 0 % | ≈ fábrica (orla escura some) |
| 08 | Granulation 0 + Warp 0 | 92 | 20 | 0 % | contorno liso; mescla igual |
| 09 | **Ponta dura** (auto off, hardness 1) | 84 | **52** | **0,92 %** | o traço diagonal vira FAIXA de borda visível |
| 10 | Fill 1 + Opacity 1 | 98 | 10 | 0 % | vidrado oliva espesso; estrias fracas |
| 11 | Smudge 0 + Rewet 0 + Charge 1 | **5** | 17 | 0 % | vidrado oliva liso: SEM arrasto, mas TAMBÉM sem placas |
| 12 | Digital Hard Round, Pigment mix | 52 | 16 | 0 % | **placas empilhadas**, um anel por traço (a foto do dono) |
| 13 | Digital Soft Round, Pigment mix | 12 | 10 | 0 % | faixa macia, mas é «pintar por cima», nada é arrastado |
| 14 | Digital G-Pen, Pigment mix | 114¹ | **143** | **2,99 %** | placas de borda dura (a foto do dono) |
| 15 | protótipo: Watercolor Fill 1 Op 1 Pigment 1 | 100 | 10 | 0 % | contínuo, mas oliva-escuro (Beer–Lambert) |
| 16 | Watercolor Basic, azul MOLHADO | 123 | 22 | 0 % | muito mais amarelo/verde; a sessão funde mais |

¹ no G-Pen a «largura» mede os degraus das placas, não fusão. Pressão a variar: mesma ordem; o 09 fica em 0,81 %
de placas, o G-Pen em 3,17 % [M]. ⚠️ Uma cena, um tamanho (40 px), rato sintético: ordem de grandeza, não lei.

## 3. Os ingredientes, por contribuição visual [M+D]

1. **Smudge sobre a tela do pen-down** (`wet_smudge` 0,5) — `smear_wet_base`, `watercolor_smudge.rs:63`, chamado em
   `stamp_route.rs:205`: cada dab levanta a BASE congelada sob o centro do dab ANTERIOR e carimba-a no seu, peso
   `smudge × cobertura` pela queda da ponta, pré-multiplicado (`ph2d-painter-brush/src/smear.rs:72`). É o único que
   MOVE píxeis: sem ele a largura cai **84 → 6 px**. Funciona igual sobre opaco (o comentário: «sobre opaco é o mesmo
   byte»). Entre traços da sessão, `arrasto_da_sessao` (`:31`) arrasta também o molhado.
2. **Campo único por traço: cobertura em MAX, composta UMA vez sobre a base** — `accumulate_wet_coverage`
   (`watercolor_accum.rs:330`) junta discos de ponta macia por max; `watercolor_render.rs` reconstrói o traço inteiro
   por píxel sobre a base (`cover = smoothstep(cov)`). Mesmo com TODA a mistura desligada (11) há **0 % de placas**:
   é ISTO que impede o «prato sobre prato» do Digital (12, 14). Sem depender de transparência.
3. **Ponta macia** — `Falloff::Watercolor` (`falloff.rs:133`: planalto 1→0,92 até 0,62, depois rampa), hardness 0,
   `watercolor_shape_auto`. Dura (09): degrau **11 → 52**, placas **0 → 0,92 %**.
4. **Rewet / dissolve** (`wet_rewet` 0,5) — `rewet_px` (`watercolor_rewet_px.rs:450`, campos de
   `build_wet_field` `:150`, `box_blur4` pelo raio do Spread) e `Dissolucao::dissolve`
   (`watercolor_mistura_agua.rs:72`): levanta e dilui a tinta de baixo ⇒ o azul clareia e entra no traço. Sem ele o
   resultado fica vidrado escuro (04). Parte é óptica (lift de pigmento); a ideia «levantar a tinta de baixo e
   espalhá-la num raio» serve a opaco.
5. **Reservatório (Charge/Pull)** — `watercolor_mixer.rs` (`RETAIN_LOAD` `:85`, `RETAIN_UNLOAD_MIN` `:89`,
   `MIX_DEPLETE_SPAN` 120 px `:98`): apanha a cor da base congelada, carga rápida/descarga lenta. Leva o azul pelo
   caminho (03: 84 → 68 px e a cor esverdeia). Já portado ao Digital no Pigment mix (`mistura_opaca.rs`).
6. **Só aspecto, quase nada na mescla:** Dilution, Spread, Edge (orla), Granulation, Warp/Flow, Fill/Opacity (a
   transparência dá a «luz» da aguada, não a fusão; 10 e 15 continuam contínuos mesmo espessos).

**Molhado entre traços** (16) aumenta a fusão (84 → 123 px), mas o dono já aprova o resultado sobre seco: não é
condição. O Digital pode deixá-lo para depois.

## 4. Porque o Digital «Pigment mix» parece artificial [M+D]

`mistura_opaca.rs` mistura por píxel contra a tela de antes, mas **cada traço é um prato novo**: o 2.º amarelo mistura
sobre o 1.º já composto, com a borda da ponta (dura no G-Pen/Hard Round) ⇒ anéis empilhados (12, 14: degrau 143,
placas 3 %). E **nada se move**: a cor de baixo nunca é arrastada na direção do traço (largura do Soft Round 12 px
contra 84). Falta exactamente os ingredientes 1–3.

## 5. Desenho — «Digital wet blend» (substitui o Pigment mix no Color mixing)

Por traço, na rota da mistura opaca (`stamp_route.rs:520`, `stamp_dabs_mistura_opaca`), sem tocar a aguada:
1. **Base do traço** = cópia da tela no pen-down (a rota já a guarda, `stamp_route/mistura.rs`).
2. **Esfrega a base** a cada dab com `smear_dab_premultiplicado` (função partilhada, só CHAMADA, nunca alterada),
   peso `Blend × queda da ponta`, do dab anterior para o actual (ingrediente 1).
3. **Cobertura do traço em MAX** com a queda da ponta (o `stroke_mask`/tecto da rota já existe) — ingrediente 2.
4. **Compõe uma vez por píxel**: `out = mix_km(base_esfregada(p), cor_levada, Amount × cobertura(p))`, `cor_levada` =
   reservatório K–M já existente (Pickup/Pull). Opaco: sem Beer–Lambert, sem orla, sem grão.
5. **Suaviza a fronteira do traço** com a ponta macia por omissão no modo (Softness → hardness/feather), e um
   «Bleed» opcional = `box_blur` da base esfregada no raio pequeno, só dentro da cobertura (ingrediente 4 sem a óptica).

**Knobs que o dono vê** (cartão Color mixing, `Off · Wet blend`): **Blend** (quanto arrasta a tinta de baixo,
= Smudge, fábrica 0,5) · **Pull** (até onde a leva) · **Pickup** (quanto da cor de baixo o pincel carrega) ·
**Amount** (quanta tinta nova) · **Softness** (borda do traço) · **Bleed** (espalha a tinta de baixo dentro do traço).
A lei de cor K–M/espectral herda o slider Pigment.

## 6. Watercolor Basic ao bit, riscos e custo

- **Ao bit [D]:** a aguada sai em `stamp_route.rs:196-205` antes da rota da mistura opaca (`:520`); o código novo
  vive só em `mistura_opaca.rs` / `stamp_route/mistura.rs` / painel do Digital. ⛔ Não alterar `smear.rs`,
  `watercolor_*`, `ph2d-pigment`, `espectral.rs`, `blend.rs` (só chamar). Prova: o teste existente
  `o_watercolor_basic_nao_mudou_com_o_pigment_mix` (`a_mistura_pigmento_digital.rs:64`, 4 hashes) + uma 5.ª hash com
  ESTA cena (01) + os alarmes `a_largura_de_fabrica_e_a_aprovada` / `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`.
- **Riscos:** (a) Smudge opaco sobre camada TRANSPARENTE: usar a versão pré-multiplicada (a cura do 23/10); (b) a
  base esfregada é por traço — o undo restaura a tela inteira do traço (já é assim na rota); (c) traço sobre o próprio
  traço anterior da mesma sessão ainda empilha 2 pratos se o dono quiser fusão molhada (16) — fica como passo 2
  (sessão Digital com `wet_session_base`); (d) previews que re-carimbam (Drag Dot/Anchored/Line) não podem esfregar
  (a mesma porta `wet_smudge_live`); (e) Pickup + Blend juntos podem lavar a cor nova — fábrica Blend 0,5 Amount 0,7.
- **Custo [M]** (5 traços amarelos, 40 px, mínimo de 5 blocos intercalados, `smoke`, ms por 5 traços):
  Watercolor Basic **1001** · Rewet 0 **410** (o Rewet é **~59 %** do custo) · Granulation+Warp 0 750 · Edge 0 864 ·
  Charge 1 923 · Smudge 0 942 (o Smudge custa **~58 ms/5 traços ≈ 12 ms/traço**) · tudo off 286 ·
  Digital Pigment mix: Hard Round **43**, G-Pen 91, Soft Round 103. ⇒ o Wet blend ≈ Pigment mix + Smudge
  ≈ **20–35 ms/traço** [D a partir de M], 6–10× mais barato que o Watercolor e longe do Wet Paint
  ([ESTUDO_CUSTO_WET_PAINT](ESTUDO_CUSTO_WET_PAINT_2026-11-07.md): Dabber p50 12 → 34 ms no traço a 1080p sem → com Real
  Oil, mais a água depois de soltar, 7–14 núcleos parada). O Bleed por `box_blur` (prefix sums) é O(pegada), não do raio. Usar o Wet Paint no Digital
  daria a fusão molhada mas a outra lei de tinta e o custo dele — não recomendado para isto.
- **Medir na implementação** (`MEDIR_VELOCIDADE.md`): esta sonda já tem o laço intercalado; acrescentar a variante nova.

## 7. Próximos passos

1. Dono olha a folha (01 × 02 × 09 × 11 × 12/14) e confirma: «o que eu quero é o 01, opaco».
2. Implementar §5 no Color mixing do Digital (`Off · Wet blend`), com a 5.ª hash do Watercolor Basic antes de tudo.
3. Pôr a variante na sonda e mostrar lado a lado com 01 e 14 (mesma cena), com pressão; smoke do dono com G-Pen, Hard
   Round e Soft Round, azul seco e amarelo por cima.
