# ESTADO — Missão W3 do Óleo Real (autónoma, 27/10)

> Retoma-se DAQUI. Prompt: [`PROMPT_MISSAO_W3_material_motor_presets.md`](PROMPT_MISSAO_W3_material_motor_presets.md).
> Base: `5a01640d94` (W2 aprovada, tag `backup/oleo-real-W2-2026-10-27`).
> ⛔ Ordem do dono: no fim de TUDO (fecho + mensagem final), confirmar que nada desta linha corre e `systemctl poweroff`.

## Contrato do material (Etapa M) — decidido pela janela ANTES dos agentes

Os 8 controlos, nomes do Blender na UI, mesma ordem no pincel (Tuning → Material) e no cartão Real Oil:

| UI (Blender) | campo `MaterialCelula` / `MaterialCartao` | NEUTRO | combinação célula × cartão |
|---|---|---|---|
| Metallic | `metal` | 0 | max |
| Roughness | `rugosidade` (ABSOLUTA, 0..1) | 0,5 | `cartão × célula / 0,5` |
| IOR Level | `nivel_ior` (= Specular do Blender 4) | 0,5 (IOR 1,5) | `cartão × célula / 0,5` |
| Coat | `verniz` | 0 | max |
| Coat Roughness | `verniz_rugosidade` (absoluta) | 0,03 | `cartão × célula / 0,03` |
| Sheen | `veludo` | 0 | max |
| Subsurface | `subsuperficie` (peso) | 0 | max |
| Subsurface Radius | `raio_subsuperficie` (px de tela) | `RAIO_SUBSUPERFICIE_PADRAO` (4,0; a M1 pode re-decidir MEDINDO) | `cartão × célula / padrão` |

- Célula NEUTRA ⇒ o valor do cartão; cartão no padrão ⇒ o valor da célula (é a lei de hoje, reescrita para os 8).
- **Roughness absoluta:** o Gloss do cartão sai; o cartão passa a ter Roughness = g(Gloss) (a lei de hoje), e o padrão
  do cartão = g(Gloss padrão) ⇒ imagem de hoje bit a bit. O `roughnessMul` m antigo do pincel ↔ célula `0,5·m`.
  Preset/macro antigo com `Gloss` converte-se em Roughness por g; com `roughnessMul` em `0,5·m`.
- **Saem:** Spec tint, Anisotropy, Translucency, Reflectivity, Reflect map (chip), Map scale, Map strength. Decisão:
  o reflexo usa o **Environment** (o Principled não tem mapa de reflexo à parte — o mundo é um só). Leitor tolerante:
  campo desconhecido ignorado; `Translucency` antigo → `Subsurface` com o mesmo peso (a SSS nova inclui a volta do
  papel sob a película fina, que era o que a Translucency fazia).

## Etapas

| etapa | estado | commits | números | o que ficou |
|---|---|---|---|---|
| M0 contrato + remoções + Roughness absoluta (workspace compila, neutro bit a bit) | ✅ | `8966e951b2` `2e24c8d9b1` | cartão guarda Roughness ABSOLUTA, padrão `RUGOSIDADE_PADRAO = 0,28` (o valor fixo de `Afinacao::rugosidade`, que saiu); célula neutra 0,5; `roughnessMul` m → `0,5·m`; multiplicador antigo do cartão k → `0,28·k`; combinação única `material::efectivo` (max nos de neutro 0; `célula × (cartão/padrão) × (padrão/neutro)` nos outros 4 — no padrão dá `0,5m × 0,56` = `0,28m` bit a bit); Coat Roughness padrão `f64::from(0.03f32)`. Gates: `o_material_neutro_e_bit_identico_a_de_antes`, novo `o_roughness_do_cartao_no_padrao_e_a_imagem_de_antes`, `o_preset_de_antes_da_w3_abre_com_o_material_convertido`, `a_macro_de_antes_da_w3_abre_com_o_material_convertido`. 5 testes saíram com os lóbulos (Spec tint, Anisotropy, película translúcida, Strength/Scale/Reflection map, chip). Tuning 86 → 85 linhas, grupo Material 10 → 9 (`ph2d-panel-wet-tuning/src/rows.rs`) | **Decisão da janela: o Gloss FICA** — no código ele é a força do brilho (`OleoReal::brilho`), nunca mexeu na rugosidade; a fórmula «Roughness = g(Gloss)» do contrato não tinha objecto. Fica no cartão ao lado de Depth e Shadow. A etapa M2 (UI) ficou feita dentro da M0: não se lança |
| K1 bancada rápida com óleo | ✅ | `c3d0e0c4a7` | causa medida: o óleo recompunha a cada evento/quadro (~1 700×/foto). Cura: pinta sem óleo e liga o Real Oil NA foto (recompõe 1×, bit a bit igual). Oil Bristle 19,53 → 1,63 s (sem óleo 1,44; 1,14×) · Blender Soft sobre 27,15 → 3,43 s (load 93–143: leia as razões). `PH2D_EXAME_OLEO=sempre` = caminho antigo. Gate `a_foto_com_oleo_so_na_foto_e_a_de_sempre` (vermelho sob mutação, 43 474 bytes) | gate corrido num export de `1e8664310f` (M0 partia a árvore): re-correr por nome no fecho. No APP o óleo custa ~10× por traço e por quadro — o passe de placa aberto desde a W2 |
| M1 SSS × Cycles | em curso (com K2 em paralelo, ficheiros disjuntos) | | | |
| K2 textura que escala com o pincel | em curso | | | |
| K3 faca/cerda (depois de K2: mesmos ficheiros de traço) · K4 sombreado (depois de M1: mesmos ficheiros do óleo) | à espera | | | |
| P 13 presets | à espera de M e K | | | |
| Fecho | à espera | | | |
