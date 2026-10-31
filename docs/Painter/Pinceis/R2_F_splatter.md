# R2-F — J-3 Splatter (gotas opacas, composição)

**Mecanismo (foto).** O de hoje (`aerografo::EM_ESTUDO[0]`) é a mancha do Rebelle (imagem `mancha`, opacidade 0,55,
jitter de opacidade): salpicos pálidos e todos do mesmo tamanho. Falta a gota OPACA e a GAMA de tamanhos. A
composição já tem as duas coisas: um disco redondo opaco no spray, mais a 2.ª população do véu (gotas maiores, à
opacidade 1, que não precisa da lei `VEU_OPACIDADE_MAX`), mais o tecto `particle_max_px` na tela.

Sonda `familias::mistura::sonda_r2f::sonda_r2f_splatter` (#[ignore], `gestos` + `tamanhos`). Fiz 2 rondas, 6
variantes. Todas começam do disco: `shape None`, opacidade 1, sem texturas, `count 0.6`, `veu_opacidade 1`.

| | spray · jitter · j.escala · particle · véu × escala · tecto | visto |
|---|---|---|
| s1 | 4 · 0,9 · 0,9 · 0,3 · 0,15 × 2,5 · — | gotas opacas; a 200 px, discos gigantes |
| s2 | 6 · 1,2 · 0,95 · 0,2 · 0,1 × 4 · — | idem, e pior |
| s3 | 3 · 0,7 · 0,8 · 0,45 · — · — | a 200 px, gigantes |
| s4 | s3 + tecto 36 px | uniforme, sem gotas grandes |
| **s5** | **s1 + `particle_max_px` 36** | **gotas opacas de vários tamanhos, longe do traço, a 5–200 px** |

Prancha `pranchas/r2_splatter.png`: antes | s5 | MyPaint tanda/splatter-04 | deevad/spray2.

**Ressalva.** Nenhum dos dois oráculos é este pincel. O tanda/splatter-04 é um cordão de discos AO LONGO do traço. O
deevad/spray2 é uma nuvem de discos translúcidos. O s5 segue a definição do coordenador (gotas opacas, de tamanhos
vários, longe do traço). As gotas têm borda dura e saem redondas demais: falta a borda do salpico (cauda, coroa).

**Receita pronta (não promovida, `aerografo.rs` não tocado).** Em `EM_ESTUDO[0]`:
- `shape` → `{"kind":"None"}`, `spray_count` 4, `jitter` 0.9, `jitter_scale` 0.9, `opacity` 1;
- `dab_jitter` = `{"count":0.6,"opacity":0,"particle":0.3,"veu":0.15,"veu_escala":2.5,"veu_opacidade":1,"particle_max_px":36}` (mais os zeros);
- retirar `texturas`.

Para promover: mover para `MEIOS`, depois `PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Splatter"` sobre
`gera_os_presets_de_fabrica` (pelo `scripts/ph2d-run.sh`), e por nome `the_factory_presets_are_counted` e
`cada_preset_de_fabrica_e_a_sua_receita`.
