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
| M0 contrato + remoções + Roughness absoluta (workspace compila, neutro bit a bit) | em curso | | | |
| K1 bancada rápida com óleo | em curso | | | |
| M1 SSS × Cycles | à espera de M0 | | | |
| M2 controlos no pincel e no cartão, leitor tolerante | à espera de M0 | | | |
| K2 textura que escala · K3 faca/cerda · K4 sombreado | à espera | | | |
| P 13 presets | à espera de M e K | | | |
| Fecho | à espera | | | |
