# R2-F — J-3 Splatter (gotas opacas, composição)

**Mecanismo (foto).** O de hoje (`aerografo::EM_ESTUDO[0]`) é a mancha do Rebelle (imagem `mancha`, opacidade 0,55,
jitter de opacidade): salpicos pálidos e todos do mesmo tamanho. Falta a gota OPACA e a GAMA de tamanhos. A
composição já tem as duas coisas: um disco redondo opaco no spray, mais a 2.ª população do véu (gotas maiores, à
opacidade 1, que não precisa da lei `VEU_OPACIDADE_MAX`), mais o tecto `particle_max_px` na tela.

Sonda `familias::mistura::sonda_r2f::sonda_r2f_splatter` (#[ignore]); 2 rondas, todas do disco opaco sem texturas.

| | spray · jitter · j.escala · particle · véu × escala · tecto | visto |
|---|---|---|
| s1–s3 | 4·0,9·0,9·0,3·0,15×2,5 / 6·1,2… / 3·0,7… · sem tecto | a 200 px, discos gigantes |
| s4 | s3 + tecto 36 px | uniforme |
| s5 | s1 + `particle_max_px` 36 | discos opacos de vários tamanhos |

s5 RECUSADA (coordenador: círculos perfeitos). Volta 3 = a MANCHA de hoje (rodada, `flip_x` 0,5), opaca, véu e
tecto: m1 (= s5) · m2 · m3 (6 · 1 · 0,9 · 0,25 · 0,2 × 3) a tecto 36 = salpicos certos mas MIÚDOS (a mancha ocupa
~½ da caixa) · **m4 = m3 + tecto 72 px**: manchas opacas com coroa, vários tamanhos, longe do traço; nos gestos
finos, pontilhado ralo. Prancha `r2_splatter.png`: antes | s5 | m4 | tanda | deevad (nenhum oráculo é isto).

**Receita m4 (não promovida, `aerografo.rs` intacto).** `EM_ESTUDO[0]` com a mancha: `spray_count` 6, `jitter`
1.0, `jitter_scale` 0.9, `opacity` 1; `dab_jitter` `{"count":0.6,"flip_x":0.5,"opacity":0,"particle":0.25,
"veu":0.2,"veu_escala":3,"veu_opacidade":1,"particle_max_px":72}` (resto 0).

Promover: mover para `MEIOS`, depois `PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Splatter"` sobre
`gera_os_presets_de_fabrica` (pelo `scripts/ph2d-run.sh`), e por nome `the_factory_presets_are_counted` e
`cada_preset_de_fabrica_e_a_sua_receita`.
