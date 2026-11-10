# Z1 — cópia de plano a 4K (B17a, 10/11)

Sonda: `crates/ph2d-tool-painter/src/plane_copy_z1.rs` (`z1_copias_de_plano_a_4k`), `--profile smoke`, um processo,
4 variantes intercaladas em rodadas rodadas, 15 rodadas, o MÍNIMO (ms). 16 fios rayon (fatia do `ph2d-run.sh`),
load 9,15. «frio» = destino segurado (páginas novas: o fork do pen-down com três donos); «reciclado» = o destino
anterior libertado antes. `collect` = `par_iter().copied().collect()` (a rota antiga do `par_clone`); `blocos` =
`vec![T::default(); n]` + `par_chunks_mut(..).copy_from_slice` (256 KiB / 1 MiB).

| plano | MB | frio série | frio collect | frio blocos256K | frio blocos1M | rec. série | rec. collect | rec. blocos256K | rec. blocos1M |
|---|---|---|---|---|---|---|---|---|---|
| u8 3840x2160 (cobertura) | 8,29 | 0,97 | 2,21 | 0,29 | 0,29 | 0,25 | 2,06 | 0,29 | 0,28 |
| rgba 1920x1080 | 8,29 | 0,25 | 2,14 | 0,28 | 0,31 | 0,25 | 1,98 | 0,33 | 0,28 |
| rgba 2048² | 16,78 | 0,90 | 4,45 | 0,67 | 0,63 | 0,60 | 4,21 | 0,63 | 0,61 |
| 24 MiB u8 | 25,17 | 1,32 | 6,38 | 1,14 | 1,12 | 1,02 | 6,01 | 1,11 | 1,10 |
| [u8;7] 2048² (material) | 29,36 | 1,38 | 1,41 | 1,31 | 1,38 | 1,18 | 1,32 | 1,39 | 1,36 |
| rgba 3840x2160 (tela) | 33,18 | 1,78 | 7,72 | 1,67 | 1,64 | 1,37 | 7,80 | 1,46 | 1,53 |
| f32 3840x2160 (altura) | 33,18 | 1,59 | 2,07 | 1,53 | 1,65 | 1,27 | 2,05 | 1,57 | 1,61 |
| [u8;7] 3840x2160 (vidro) | 58,06 | 3,03 | 3,02 | 2,93 | 2,99 | 2,67 | 2,52 | 2,76 | 2,95 |
| rgba 4096² | 67,11 | 10,78 | 15,66 | 3,43 | 3,45 | 10,51 | 16,34 | 3,05 | 3,29 |

Leitura:
- O `collect` escreve elemento a elemento: em `u8` custa 4–7× o `memcpy` em série. Era ELE que «perdia a 2048²», não o
  paralelo. A 4096² o produto pagava 15,66 ms por plano RGBA (já acima dos 32 MiB) contra 3,43 em blocos.
- Em blocos: empata com a série onde as páginas já estão mapeadas (±0,3 ms) e ganha ~3× onde há *first-touch* (a 1.ª
  linha e a 4096², onde o alocador entrega páginas novas — o caso do pen-down no app, ~5,5 ms/plano a 4K no estudo).
- Limiar da cópia e do preenchimento: **7,5 MiB** (`15 << 19`): entram o plano `u8` 4K e a tela 1080p (8 294 400 B);
  o Ctrl+Z de 1024² (planos até 7 340 032 B) fica em série. 256 KiB × 1 MiB por bloco empatam; fica 256 KiB.
- Primeira corrida (load 41, 7 rodadas) deu a mesma ordem (collect u8 4K 10,77 × série 2,22 × blocos 2,38).
