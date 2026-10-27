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
| M1 SSS × Cycles | ✅ | `fa11213a91` `ea4b55fd2e` | lei: irradiância E desfocada por canal (gaussiana separável, σ = raio × (1; 0,2; 0,1), raio por célula), `difuso_c += peso·max(0, B_c − E)` — cor da tinta, tecto = a irradiância mais forte da vizinhança. Recusadas (sobre a irradiância do Cycles): conservadora `E+w(B−E)` (encosta −0,080 × +0,023), volta do papel `exp(−h/(p·r))` (rms piora), perda para o papel (borda −0,37 × −0,04). Régua R × Cycles r4: sombra +0,184 × +0,157 · tinta fina +0,033 × +0,055 · borda +0,022 × −0,025 · corr 0,50; r16: sombra +0,266 × +0,313. Raio padrão 4 px, tecto 16 px (`RAIO_SUBSUPERFICIE_MAX`, recurso: CPU do recompor — a região cresce 3×raio). Custo ns/célula: tela 42 → 2 511 (r4) · pincelada 65² 372 → 35 586. Gates `a_subsuperficie_zero_e_a_imagem_de_antes`, `a_subsuperficie_enche_a_sombra_com_a_cor_da_tinta_e_a_energia_tem_tecto` (3 mutações vermelhas). Folha `out/M1_sss_folha.jpg` (olhada pela janela: sombras cheias de vermelho, como o Cycles). Oráculo consertado para os 8 da W3 | borda +0,02..0,04 × Cycles −0,03..−0,06 (o Cycles escurece a borda a peso 0,25–0,75); irradiância calculada 2× (custo) → K4; knob `subsurfaceRadius` em `tuning/defs.rs` e cartão em `ids/painter_wetpaint.rs` ainda a 64 → 16; `composite.rs` só alarga a região suja pelo cartão (SSS de pincel com cartão 0 deixa orla velha) → K4 |
| K2 textura que escala com o pincel | ✅ | `7677042b2d` | knobs `textureSizeExp` γ (0 = neutro) e `textureSizeRef` (41 px), Tuning › Stroke: grão `Tiled` e pó × `(D÷ref)^γ`, D = diâmetro sem pressão (grão preso à tela não pode seguir a pressão). Razão (tamanho do grão ÷ largura) 200 px ÷ 40 px, Rebelle · antes · depois: Dabber 3 1,30 · 0,51 · 1,20 · Dabber 2 0,55 · 0,34 · 0,58 · Splats 0,34 · 0,24 · 0,33 (tom 0,21 → 0,16 × 0,15). `spray_count>1`: marca menor que o pincel era recortada da textura da cerda (0,02–0,05) e o portão do papel apagava-a → marca pequena = disco liso: 54 → 9 059 de tinta. Gates: miniaturas da fábrica ao byte (verde), `tests/it/textura_do_tamanho.rs` (3), 5 mutações vermelhas. Tuning 85 → 87. Relatório `docs/Painter/Pinceis/W3_K2_textura.md` | Dabber 2 ~6× área de buraco a 200 px (valor da P); lascas angulares do Dabber 3 = TIPO de textura (carimbo de lasca) → K3b; Splats: Rebelle tem 2 populações (pontos grandes escuros + pó) e escurece a 200 px; marca de 0,5 px no traço de 5 px não pousa |
| K4 sombreado (halo, metal nas encostas, restos do SSS) | ✅ 3 de 4 | `03048b3e7d` `9a4991540f` `9a0e1bc1c5` `7c52d210f5` | **halo**: a lei f64 estava certa; o composite escurecia o BYTE sRGB pela fracção (≈ 2,2× a luz) → `sombra_em_u8` guarda `1 − sRGB(1 − s)`. Papel p50 linear, antes → depois × Cycles: lisos Estúdio lado iluminado 0,087 → 0,044 × 0,036 · lado da sombra 0,216 → 0,104 × 0,107 · cena do dono 0,227 → 0,107 × 0,101 (Rebelle: zero além de 3 px; o Estúdio do Cycles TEM halo). Gate `a_sombra_no_papel_tira_a_luz_linear_e_nao_o_byte`. Recusado: rebote das paredes no papel (apaga a sombra do empaste alto, 0,010 × 0,069). **SSS**: irradiância guardada com o campo (excesso do SSS 4 357 → 837 ns/célula, load 5,3; gate `a_luz_do_ceu_guardada_e_a_de_sempre`, 3 mutações); `alcance_na` alarga a região suja pelo raio do PINCEL (gate `a_subsuperficie_do_pincel_recompoe_como_a_tela_inteira`, vermelho antes da cura); cartão Subsurface Radius ≤ 16. Folha `out/K4_folha.jpg` (olhada pela janela) | **metal nas encostas NÃO curado**: o «p90 0,60 × 1,16» era régua errada (comparava depois do `rola` contra Cycles sem tecto; antes do `rola` R p50·p90·p99 0,29·2,19·6,9 × 0,20·2,16·9,8); o vinho com riscas pretas também está no Cycles (16 % × 8 %); o defeito real é o meio-tom nas paredes íngremes R 0,34 × 0,16. Recusas medidas: joelho no `rola` (erro 0,1123 → 0,1149), metal vizinho só com céu (0,1131). Aberto: sombra projectada longa atrás do empaste alto (12–24 px: Cycles 0,276 Estúdio) que não desenhamos; Clássico liso +0,03 mais escuro a 3–6 px; `lamina_tests.rs` ainda escreve o papel pelo byte |
| K3a faca · K3b cerda | em curso | | | |
| P 13 presets | à espera de M e K | | | |
| Fecho | à espera | | | |
