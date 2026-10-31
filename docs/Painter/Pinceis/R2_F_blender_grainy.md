# R2-F — Blender Grainy (L-2, Rebelle Blend/Default/Textured)

**Item.** O Blender Grainy NÃO é o «Blender Textured» de fábrica: este é o Blender Soft com força 0,3 e toque 0,35,
SEM grão (mistura.rs `WET_FERRAMENTAS`); o Grainy é o preset de ESTUDO (fora da fábrica)
`pranchas/mistura/presets/blender_grainy.ph2d-brush.png` (J6 `0319d50dcf`: + Grain Noise 24, intensity 0,2,
blendForce 1, blendReach 0,75). Os dois miram a mesma foto (`rebelle_bateria/BLEND/default_textured`).

**Diagnóstico (foto J6).** Os grumos rasgados são o GRÃO DA MÁSCARA: o `texv` do Noise (lado 24 texels) substitui as
cerdas na máscara do Blend (`Trail::accumulate_blend_grao`), e a máscara satura (`mask → 1`) onde o grão passa o
limiar do papel ⇒ manchas de ~24 px com buracos brancos. Não é o alcance nem o arrasto. Os lados direitos da coluna
são a pegada sem toque espalhado.

**Cura = composição, zero motor** (nenhum ficheiro do motor tocado). Sonda `familias::mistura::sonda_r2f::sonda_r2f_blender_grainy`
(#[ignore]; o exame.sh é recusado pelo hook do subagente). Cena `sobre` A, x ±30/±45 do cruzamento do meio:

| | acima | abaixo | buraco médio px | tinta média px | opaco (verde < 60) |
|---|---|---|---|---|---|
| Rebelle | 31 | 37 | 8,3 | 10,6 | 350 |
| v0 J6 (Noise 24) | 37 | 29 | 6,5 | 7,8 | 184 |
| v1 Noise 6 | 27 | 35 | 4,3 | 3,3 | 0 |
| v2 Noise 3 | — | — | (visto: fino, coluna recta) | | |
| v3 sem grão + paperGate 0,8 | quase nada pousa | | | | |
| v4 Noise 6 + paperGate 0,7 | pálido, ralo | | | | |
| v5 Noise 6 + toque 0,35 | 26 | 29 | 3,1 | 3,4 | 0 |
| v6 Noise 6 + toque 0,2 + gate 0,55 | 22 | 19 | 6,1 | 1,8 | 0 |
| **v7 Noise 4 + toque 0,35** | **35** | **35** | **4,7** | **11,4** | **80** |

**Escolhida v7** (gravada no preset de estudo por `SONDA_R2F_GRAVA=v7`): o alcance do Rebelle (35·35 contra 31·37),
grão fino (buraco 4,7 px contra 6,5 do J6), nuvem com lados macios em vez da coluna recta, sem grumos rasgados.
Prancha `pranchas/r2_blender_grainy.png` (antes | depois | Rebelle, cena inteira + ampliação ×2).

**Diferença que fica (para o dono afinar).** A nossa nuvem é ~1,5× mais LARGA que a do Rebelle (o toque 0,35 abre-a)
e mais clara no miolo (opaco 80 contra 350): o Rebelle faz grumos opacos; a meta pedida foi o grão fino, sem grumos.

**Fila (coordenador).** Entrar na fábrica (como «Blender Grainy» ou como a receita do «Blender Textured») = regerar
o preset e `the_factory_presets_are_counted` / `cada_preset_de_fabrica_e_a_sua_receita` — pesado, vai pela
`scripts/fila-pesada.sh`; não foi corrido. Receita: Blender Soft + `texture` Noise 4×4 + `jitter 0.35` +
`intensity 0.2` + `blendForce 1` + `blendReach 0.75`.
