# 23 — Estudo: como Blend · Smear · Erase · Wet · Dry · Blow devem tratar o PIGMENTO

> Pedido do Enio (2026-07-22): *"estudo sério em app de física como Rebelle… como o pigmento
> reage a cada um deles. Exemplo disfuncional: pinto com tinta pura sem água, pego o pincel
> Wet e passo sobre a tinta — nada acontece. Certamente esse não é o comportamento normal."*
>
> Método: leitura do CÓDIGO do app modelo (vendorizado em `docs/Painter/ph2d_wet_paint`,
> o mesmo de escapemotions.com/experiments/rebelle) + conferência do nosso porte
> (`crates/ph2d-wet-paint`) + manual/blog do Rebelle real + física de aquarela.
> **Este doc é estudo e proposta — nada foi implementado.**

## 1. O veredito do caso reportado

**O "nada acontece" é fiel ao MODELO — não é bug de porte.** Três fatos, com linha:

1. **`applyWet` só escreve água** (`tools.js:51-74` · nosso `tools.rs::apply_wet`): adiciona
   `film` e acorda o byte `wet`. **Nunca toca `sett`** (o pigmento assentado/seco). Zero
   interação imediata com a tinta que já está lá.
2. **O único caminho `sett → susp` é o re-wet PASSIVO** do `dryingPass`
   (`drying.js:74-97` · `drying.rs:139-160`): água parada lifta pigmento assentado a
   `b = rewetBase · rewet · (1 + excess·50) · staining_mult`, com `rewetBase = 0.0001`
   por passe, um passe a cada 6 frames de 40 Hz. **Conta feita:** com bastante água
   (film ≈ 0,3) e knob `rewet` no default 1.0, ≈ **1%/segundo** — meia-vida de ~65 s.
   No máximo do knob (8): ~8%/s. Homeopático na escala de um gesto.
3. **Mesmo o que lifta é invisível**: `susp` liftado herda a cor do `sett` embaixo — parado,
   compõe para a MESMA cor na tela. Re-wet só se VÊ quando o pigmento re-suspenso **flui**
   (bleed/gravidade/Blow) ou é arrastado — e sem lift não há o que fluir: círculo vicioso.

## 2. A referência de verdade

**Física** (Winsor & Newton, Jackson's): aquarela é **re-solúvel** — a goma-arábica
encapsula o pigmento e re-dissolve com água; *lifting* (molhar o seco, agitar, absorver) é
técnica padrão. Pigmentos **staining** (Phthalo, Quinacridone) penetram a fibra e resistem;
non-staining (Cerulean) liftam limpo. ⇒ o knob `extStaining` do modelo (hoje **Hidden**,
default 0,5, `stainingMultiplier` 0→×8 · 1→×0) já é o gancho físico certo.

**Rebelle real** (manual 8, Visual Settings): **Re-wet (1-10)** — *"If it is set high, the
new paint **rewets the paint below** faster and creates strong watercolor edges."* O re-wet
do Rebelle é **dirigido pelo TRAÇO** — imediato, sob o pincel — não um processo ambiente.
Water tool molha local ("correct the edges by adding water"); botões **Wet the Layer / Wet
All Visible**; Dry *"removes water… drying any brush stamps"*; Blow *"blows wet colors and
drips"*; Smudge *"drags and distorts colors… without blending"*; Blend *"blends colors"*, e
o blog nota que blend sobre camada SECA funciona (*"brings a different look"*).

## 3. Tool a tool: o que o modelo faz × o que deveria

| Tool | Hoje (modelo = porte) | Gap × referência | Proposta |
|---|---|---|---|
| **Wet** | `film += stamp·add`; `wet = f(paper)`. Nunca lê `sett`. | Rebelle re-wets SOB o traço; física: pincel molhado dissolve o topo do seco na hora. | **P1 — lift ATIVO no stamp** (a proposta-núcleo, §4). |
| **Smear** | Arrasta `susp`+`film` (snapshot bilinear mass-weighted). `sett` **intocado de propósito** (`tools.js:158`). Sobre seco puro: **nada**. | Rebelle Smudge arrasta também o seco (é o smudge da imagem); dedo sobre aquarela seca + água esfrega de fato. | **P2**: em célula com `film>0`, dissolve primeiro (mesma porta do P1) e arrasta; em célula seca, arrasta fração de `sett` diretamente com resistência `smear·(1−staining)`. `staining=1` recupera o comportamento atual. |
| **Blend** | O único que já mexe no seco: relaxa `susp` E `sett` para a média da janela (`trail.js:276-346`). | Certo no essencial (blend seco existe no Rebelle). Falta coerência: com ÁGUA presente, blend deveria também re-suspender (blend molhado ≠ blend seco). | **P3** (menor): com `film>0`, parte do relax de `sett` sai via a porta de lift (`sett→susp`) em vez de homogeneizar in place — o blend molhado passa a sangrar depois. |
| **Erase** | Remoção multiplicativa de `susp`+`sett`+`film`, paper-gated, nunca zera. | Adequado. Refinamento físico: apagar `sett` deveria custar mais com staining alto. | **P4** (opcional): `force_sett = force·(1−0.5·staining)`. |
| **Dry** | Encolhe `film` (piso 0.001) e **sela** (`wet=0`). | Bate com o Rebelle ("stop the spread"). | Nenhuma mudança. |
| **Blow** | Injeta velocidade só onde `film>0`; arrasta o byte `wet`. Sobre seco: nada. | Fisicamente certo — ar não move pigmento seco. O problema é o COMBO: Wet→Blow devia funcionar (molhar e soprar) e hoje não, porque o Wet não lifta. | Nenhuma mudança própria — **o P1 o conserta em cascata**. |

## 4. P1 — o lift ativo do Wet (o conserto do caso do Enio)

No `apply_wet`, por pixel do stamp, DEPOIS de escrever o film:

```
lift = sett[i] · stamp · wetLift · stainingMult(staining)   // clamp a sett[i]
sett[i] -= lift;  susp[i] += lift;                          // cor: MESMA aritmética
                                                            // do re-wet passivo
                                                            // (drying.js:83-95) — reusar,
                                                            // porta única de lift
```

- **Knob novo `wetLift`** (grupo Tools, ao lado de `eraser`/`dryer`/`blow`/`smear`;
  range 0..1). Calibrar pelo gesto: **2-3 passadas do Wet dissolvem visivelmente** um traço
  de tinta pura seca (o caso literal do reporte). `wetLift = 0` recupera o modelo byte-a-byte.
- **Por que no TOOL e não no `dryingPass`:** (a) é onde o Rebelle o põe (re-wet dirigido a
  traço); (b) física: a agitação mecânica do pincel é o que dissolve — água parada lifta
  devagar (o passivo atual fica como está, é o ambiente); (c) **não toca o caminho do modo
  Paint** ⇒ o fingerprint pinado (G0/G0b) e a suíte de aceitação §18 ficam intactos — a
  mudança é aditiva por-tool, exatamente como as portas do doc 22.
- **Efeito em cascata:** o `susp` re-suspenso entra no solver ⇒ bleed nas bordas, gravidade
  do tilt puxa, **Blow passa a soprar tinta re-molhada**, Smear tem o que arrastar. Um
  mecanismo, quatro tools curados.
- **`extStaining` sai do Hidden** e entra no painel Tuning (grupo PAINT) — com lift ativo,
  staining vira o controle de produto de "quão permanente é a tinta seca" (e o
  `stainingMultiplier` já existe e está testado).

**Extensão opcional (decisão de produto, wave própria):** o mesmo lift sob o stamp do
**Paint** com água alta — é literalmente o knob "Re-wet" do Rebelle ("new paint rewets the
paint below… strong watercolor edges"). Entra atrás de knob com default 0 para não mover o
fingerprint até o Enio aprovar o look no smoke.

## 5. Gates red-first prescritos (quando a implementação for ordenada)

1. `wet_over_dry_pure_paint_reactivates_it` — o reporte literal: deposita tinta pura
   (water 0), `fast_dry`, 3 dabs de Wet ⇒ `sett` caiu, `susp > 0` no local, e um Blow
   subsequente MOVE a cor (oráculo de aparência: o centro de massa da cor viaja).
2. `wet_lift_zero_is_the_old_model_to_the_byte` — `wetLift=0` ⇒ fingerprint da sessão de
   tool idêntico ao de hoje (a lei do doc 22: off é off ao bit).
3. `staining_one_pins_the_paint_down` — `staining=1` ⇒ lift 0 (multiplier já dá 0).
4. `smear_drags_dry_paint_when_not_staining` (P2) + irmão `staining=1` = comportamento atual.
5. Fingerprint G0/G0b do modo Paint **intocados** em todos os acima.

## 6. Fontes

- Código do modelo: `docs/Painter/ph2d_wet_paint/js/engine/{tools,drying,sim,trail}.js`
  (porte 1:1 em `crates/ph2d-wet-paint`, divergência zero nos pontos citados).
- Rebelle 8 Manual — Visual Settings (Re-wet 1-10): escapemotions.com/products/rebelle/manual/8/interface/panel-visual-settings/
- Rebelle 8 Manual — Working with Water (Wet the Layer/Wet All Visible/Fast Dry): …/manual/latest/starting-painting/water/
- Rebelle 7/8 Manual — Panel Tools (Water/Dry/Blow/Blend/Smudge): …/manual/interface/panel-tools/
- Blog Escape Motions — "Creation Methodology of Real Watercolors in Rebelle 4.1" (blend
  seco "different look") e "10 Essential Rebelle Tips" (Blow, dry-to-stop-spread).
- Winsor & Newton — "How to lift watercolour" (re-solubilidade, staining vs non-staining);
  Jackson's Art — "Guide to Watercolour Mediums" (goma-arábica re-solúvel).
