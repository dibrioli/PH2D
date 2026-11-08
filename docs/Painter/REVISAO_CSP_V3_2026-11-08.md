# Revisão v3 dos 22 pincéis «à moda do Clip Studio» — CSP corrido em 3 fundos (08/11)

> Veredito do dono (08/11): vários pincéis novos estavam COMPLETAMENTE diferentes do original; os Multiply (Wet Wash
> Mix, Watercolor Splash) não pintavam na tela branca do PH2D. A comparação de 08/11 só usou blocos opacos
> ([memória](../../project-memory/feedback_brush_oracle_must_include_blank_canvas.md)). [M] = medido · [D] = dedução.

## Método

- **Cena única** ([`cena_v3.py`](handoffs/material_2026-11-08_revisao/cena_v3.py)): por pincel, três fundos de
  800×880 — (a) camada VAZIA (alfa 0) sobre o papel branco, o caso real do dono · (b) branco opaco · (c) blocos
  `0046BE` | `F0C800` | `C81414`. Quatro traços por fundo: reta, S, zigue-zague, toque. Cor `00A03C`, tamanho de
  fábrica do CSP (no PH2D raio = diâmetro / 2), pressão 1 (rato).
- **Oráculo CORRIDO:** CSP 1.13.2 PRO (avaliação) sob Wine, cópia do prefixo por agente, Xvfb próprio
  (`3600x1100`), reset de fábrica confirmado por pincel. ⚠️ A avaliação **não exporta** (diálogo de licença) e a área
  de transferência só tem formato privado: a saída é a FOTO a 100 % (`import -window root`), recortada ao píxel, uma
  tela por fundo (o canvas de 2400 não cabe a 100 %). Cada tela abre UMA vez; entre pincéis, Ctrl+Z ×4 e a foto
  confere 0 píxeis diferentes da tela em branco. Fotos em `scratchpad` da sessão (não versionadas).
- **PH2D:** sonda [`sonda_csp_v3_revisao`](../../crates/ph2d-tool-painter/tests/it/sonda_csp_v3_revisao.rs) pela
  RECEITA; `PH2D_V3_VARIANTES=<tsv>` afina sem regravar presets (`campo=json`, `mistura.<k>`, `size`, `opacity`,
  `mao.<k>`, `base=<Nome>`, `-campo`). As cinco rodadas: `var1..5.tsv` na pasta de material.
- **Régua** ([`folha_v3.py`](handoffs/material_2026-11-08_revisao/folha_v3.py),
  [`afina_v3.py`](handoffs/material_2026-11-08_revisao/afina_v3.py)), por fundo, sobre o papel branco: cobertura,
  largura mediana da reta, cor média do pintado (por bloco em c). **igual** = razões ≤ 1,2 e Δcor ≤ 15 · **perto** =
  ≤ 1,6 e ≤ 40 · senão **diferente**. ⚠️ Ela lê mal o borrão dos esbatedores (Finger Tip, Gouache Blender) e a névoa
  larga: nesses o veredito final é a olho, sobre a folha.

## Resultado [M]

Folhas (CSP à esquerda, PH2D à direita, uma linha por fundo): `handoffs/material_2026-11-08_revisao/folha_<Nome>.png`;
números em [`medidas_v3.json`](handoffs/material_2026-11-08_revisao/medidas_v3.json).

| Pincel | Antes (vazio · branco · blocos) | Depois | Causa medida → cura |
|---|---|---|---|
| Wet Wash Mix | **não pinta** · dif · dif | dif* · igual · perto | receita: `blend_sobre_vazio` desligado (o Multiply guarda o alfa da camada) + nuvem de spray onde o CSP faz um tubo liso ~4,5× que apanha a cor de baixo → véu 4,5× sem tremor, Amount 0,3. *o corpo casa (RGBA 0,160,60,237 × CSP 16,166,72); a régua pesa a orla esfumada e o toque pálido |
| Watercolor Splash | **não pinta** · dif · dif | igual · igual · perto | receita: o mesmo `blend_sobre_vazio` + espalhamento de 2 diâmetros (600 px) onde o CSP agrupa manchas de 80–120 px no caminho; matiz puxava ao ciano → `across` 0,3, partícula 0,25, achatamento sorteado 0,3 (a 0,9 o app mostrava «folhas»), sem matiz, grão 0,6 |
| Round Watercolor | dif · dif · dif | igual · igual · igual | receita: era aguada (borda rasgada, ciano, não multiplica); o CSP é disco duro em Multiply a α 0,44 (bloco azul (0,59,125) previsto e medido) → **mudou de meio**: Digital, Multiply, `blend_sobre_vazio`. O ficheiro do meio Watercolor saiu |
| G-Pen | perto · perto · igual | igual ×3 | orla `borda_px` 1,5 |
| Turnip Pen | perto ×3 | perto ×3 (Δcor 25 → 20) | orla 1,2 |
| Soft Airbrush | perto ×3 | sem mudança | o CSP traça 171 px a 150 (a gaussiana passa o raio); o nosso 119. Falloff/Flow pioraram a cor (recusado) |
| Sketch Pencil | dif · dif · perto | perto ×3 | núcleo `Sharp` dureza 0, grão 0,5, opacidade 1 (o Pencil 10 do CSP traça 6 px; o nosso 7) |
| Charcoal Stick | perto ×3 | perto ×3 (Δcor 9 → 4) | dureza 0,9, sem achatado nem tremor |
| Round Mixing | perto ×3 | perto · igual · igual | Amount 0,9 · Density 0,8 (apanhava branco a mais) |
| Gouache | perto · dif · perto | igual ×3 | Amount 0,85 · Density 0,7 · dureza 0,6 |
| Dry Gouache | igual ×3 | sem mudança | — |
| Thin Gouache | perto · dif · dif | perto · igual · igual | opacidade 1 · dureza 0,9 |
| Gouache Blender | — · — · dif | — · — · perto (olho) | tamanho de fábrica do CSP é **80** (não 60) · Density 0,15 |
| Oil Paint | perto ×3 | igual ×3 | Amount 0,45 · Density 0,7 |
| Paint And Apply | dif ×3 | perto ×3 | dureza 0 `Smooth`, Density 0,5, sem afinar as pontas; ainda 21 % mais estreito (a orla macia do CSP passa o raio) |
| Thick Oil Paint | dif ×3 | igual · perto · igual | o CSP é chapado: sem as cerdas (riscas meio transparentes), Density 0,9 |
| Crayon | dif · perto · perto | perto · igual · igual | grão 0,3 (o CSP tem miolo sólido, rugoso só na orla) · dureza 0,9 · Amount 0,45 |
| Pastel | perto · perto · dif | perto ×3 | Amount 0,3 |
| Chalk | dif ×3 | perto · **dif** · perto | dureza 0,9 · achatado 0,2 · grão 0,3 · Amount 0,3 · opacidade 1. No branco o CSP apanha ainda mais branco (207,236,218); Amount < 0,25 deixaria de cobrir a camada vazia (`AMOUNT_QUE_COBRE`) — não casado |
| Pointillism | perto ×3 | igual · igual · perto | 4 pontos por carimbo |
| Brush Pen | dif ×3 | igual ×3 | a 2.ª ponta de cerdas cortava o traço a 54 % da largura → sem ela |
| Finger Tip | — · — · dif | — · — · perto (olho) | Color stretch 0,7 → 0,2: o CSP faz riscos curtos nas fronteiras; o nosso arrastava a cor pelo traço inteiro |

## O defeito «Multiply não pinta na camada vazia» — mecanismo [M]

Um traço num modo de cor pinta num PLANO próprio e compõe-se com a camada uma vez por lote
(`traco_plano.rs::compoe_o_plano` → `blend_over_pigment_modo(brush.blend, base, …, brush.blend_sobre_vazio)`). Com o
campo desligado (o de fábrica de todo pincel), `blend_over` guarda o alfa do destino nos modos de cor: sobre alfa 0 o
resultado é alfa 0. O motor JÁ tem a lei do CSP/W3C (`blend_vazio.rs`, «L7-vazio»: sobre o vazio a cor pousa em
Normal) e um pincel antigo já a liga (`efeitos.rs`, Dodge). **É defeito de RECEITA, não de motor** — as duas receitas
novas não pediam o campo. Nenhum código do motor mudou nesta revisão; com a mistura Off nada muda por construção.

## ⛔ Recusas MEDIDAS (não refazer)

- Splash com partícula maior / menos carimbos sem baixar o `across`: cobertura 2,2–2,7× a do CSP (s1–s8).
- Soft Airbrush por `falloff` (Smooth/Linear/Smoother) ou Flow 0,65–1: largura sobe só a 0,73–0,83 e a cor erra 22–44.
- Charcoal sem a 2.ª ponta: cor Δ61. Sketch Pencil sem a 2.ª ponta: largura 1,57×.
- Thick Oil com cerdas fracas (Strength 0,3): cor Δ201.
- Wet Wash por tamanho 137 sem véu (w4): melhor régua no branco, mas muda o Size de fábrica do CSP (30) — recusado.

## No app (Xvfb, `PH2D_WETPAINT_SMOKE=1`, perfil smoke, xdotool) [M]

Objecto Canvas → Draw Mode (Digital) → Layers «+» (camada 2 VAZIA sobre a 1 branca) → cor verde → Preset por nome →
um S → foto → Undo (a foto pós-undo confere a tela limpa). Os 20 pintam na camada vazia; os dois esbatedores (Gouache
Blender, Finger Tip) não pintam, como no CSP. Round Watercolor e Pastel claros, como no CSP. Folha:
[`no_app_20_pinceis.png`](handoffs/material_2026-11-08_revisao/no_app_20_pinceis.png) (a Splash dela é a de antes do
achatamento 0,3; a de depois: `no_app_watercolor_splash_v2.png`).

## Portões (verdes, por nome)

`cada_preset_de_fabrica_e_a_sua_receita` · `the_factory_presets_are_counted` · `a_largura_de_fabrica_e_a_aprovada` ·
`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` · `cada_familia_e_um_ficheiro` · `a_mistura*` (18, entre eles
`o_watercolor_basic_nao_mudou_com_o_pigment_mix`). Gerados com `PH2D_REGERA_SO` só os 20 afectados; nenhum outro
ficheiro de `assets/brushes` mudou.
