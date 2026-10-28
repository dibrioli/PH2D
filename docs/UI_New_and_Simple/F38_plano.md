# F38 — A família «memória da placa» fechada: a Radeon onboard, os blocos sob pintura, a recusada sem trabalho

> Aberto pelo handoff F37 §8 (itens 1, 2, 8, 9). Uma rodada só (CLAUDE.md §0.10), 27/10, binários `smoke`
> (F36 = `7b7ebf0ecd`, F37 = `fea3340c24`, F38 = esta onda), roteiro
> [`handoffs/img_F38/rodada_f38.sh`](handoffs/img_F38/rodada_f38.sh), logs e fotos em
> [`handoffs/img_F38/`](handoffs/img_F38/).

## 0 — Como se mede na Radeon

- A Radeon 610M (`0000:0e:00.0`, `1002:13c0`) tem **2 GiB de VRAM** reservada e **~61,6 GiB de GTT** (RAM
  do sistema que a placa usa). Sem `nvidia-smi`: o uso por processo vem do `amdgpu` (`/proc/<pid>/fdinfo`,
  `drm-memory-vram` · `drm-memory-gtt` · `amd-evicted-vram`, um cliente por `drm-client-id`); a livre, de
  `mem_info_vram_{total,used}` (o `ocupa_a_placa` passou a lê-la numa AMD).
- O app vai para a Radeon pelo carregador, sem código: `VK_ICD_FILENAMES=…/radeon_icd.json` e
  `__EGL_VENDOR_LIBRARY_FILENAMES=…/50_mesa.json`.
- ⛔ **A tela virtual tem de renderizar na MESMA placa.** Medido (prova de 15 s): com o `kwin --virtual` na
  NVIDIA, o app na Radeon dá `Surface::configure: Invalid surface` e cai no 8.º quadro. Com o kwin e o
  Xwayland na Radeon (`DRI_PRIME=pci-0000_0e_00_0`, `KWIN_DRM_DEVICES=/dev/dri/card0`) abre e desenha. ⇒ uma
  sessão de kwin por placa, no mesmo comando.
- A fila de testes das outras árvores usa a Radeon: a rodada pede-lhe a vez (`porta-testes`) e segura-a.

## 1 — A pegada na Radeon (F36 × F37 × F38)

`amdgpu` por processo, MiB, aos 12 e aos 20 s (iguais), duas corridas intercaladas por variante
([`rodada.log`](handoffs/img_F38/rodada.log)); o alocador (`PH2D_GPU_PEGADA=12`) ao lado:

| cena | F36 VRAM + GTT | F37 VRAM + GTT | F38 VRAM + GTT | alocado · reservado (F36 → F37/F38) | NVIDIA (F37 §0) |
|---|---|---|---|---|---|
| 1 (Canvas) | 819 + 90–96 | 366 + 46–48 | 366 + 48–53 | 575,2 · 790,7 → 254,0 · 337,5 | 377 |
| 41 (Nuvem com FX) | 1 331 + 150–163 | 494 + 54–66 | 494 + 53–73 | 1 030,0 · 1 366,7 → 370,8 · 465,5 | 507 |

- **Cabe, e não diverge da NVIDIA:** 366/494 MiB de VRAM contra 377/507 no `nvidia-smi` (este conta também o
  contexto do driver). O GTT (48–73 MiB) são os buffers visíveis à CPU (subidas, staging), na RAM.
- A F38 não muda a pegada (a substituta só existe sob recusa).

**A placa cheia** (`ocupa_a_placa` na Radeon, 25 s por corrida; o bloco de 256 MiB faz o alvo `300` parar
em 546 livres):

| livre antes | programa | cena 41 VRAM + GTT | livre depois | recusas | imagem |
|---|---|---|---|---|---|
| 975 | F36 | 820 + **682** | 171 | 0 | a cena inteira |
| 975 | F37 · F38 | 494 + 53 · 494 + 65 | 349 · 331 | 0 · 0 | a cena inteira |
| 546 | F36 | 477 + **1 020** | 77 | 0 | a cena inteira — [`radeon_546livre_cena41_F36.png`](handoffs/img_F38/radeon_546livre_cena41_F36.png) |
| 546 | F37 · F38 | 494 + 51 · 494 + 63 | 51 · 43 | 0 · 0 | a cena inteira — [`radeon_546livre_cena41_F38.png`](handoffs/img_F38/radeon_546livre_cena41_F38.png) |

- ⭐ **Porque a Radeon nunca recusa** (0 erros em 30 corridas, contra 283/287 da F36 na NVIDIA): o `amdgpu`
  não falha a alocação quando a VRAM acaba — **despeja-a no GTT** (a RAM do sistema). A F36 com 546 MiB livres
  ficou com 1 020 MiB no GTT; a F37/F38 cabe toda na VRAM. Numa APU a VRAM também é RAM (reservada), logo o
  despejo custa pouco; numa placa dedicada da AMD custaria o barramento PCIe. Fotos F36 × F38 iguais (0 px,
  `magick compare -fuzz 2%`), também entre F37 e F38 com a placa livre.

## 2 — Os blocos do alocador sob pintura

**O que a pintura cria por quadro** (auditoria do código): nenhuma textura do tamanho da tela — as do preview,
do compositor, da luz do impasto e o slot nascem no bind e só mudam com o tamanho. Por lote de dabs nascem e
morrem três buffers (`base`/`out`/`read`, `ph2d-paint-gpu` `StampPass::run`) do tamanho da CAIXA do lote.

**No app** (o rabisco automático do Composite, `PH2D_COMPOSITE_RABISCO=0.6`, o MESMO binário F38, 3 × 3
intercalado, [`rabisco_resumo.log`](handoffs/img_F38/rabisco_resumo.log)), período do quadro durante o traço,
o MÍNIMO das três janelas de cada corrida:

| placa | `uso` (8–64 MiB, omissão) | `desempenho` (128–256 MiB) | VRAM + GTT uso · desempenho |
|---|---|---|---|
| Radeon | 20,5 · 20,5 · (27,0 a 1.ª, a frio) | 20,8 · 20,8 · 21,5 | 434 + 62–75 · 477 + 122–127 |
| NVIDIA | 20,3 · 20,8 · 21,2 | 19,6 · 19,7 · 26,0 | 447 · 490 |

O traço desta cena corre na CPU (`GPU 0/CPU 90` no `PH2D_PAINT_PERF`): o alocador não entra no quadro. **Sem
diferença**; o `uso` poupa 43 MiB de VRAM e ~55 MiB de GTT.

⛔ **A tela 4K automática não existe, e fazê-la cruza família:** as três cenas que pintam sozinhas (Composite,
Wet Paint) são de 1024²; o `PH2D_IMPASTO_SMOKE=2` abre 4096² mas o traço é humano. Pôr um rabisco numa tela
4K é código do Painter (`ph2d-app-painter`), de outra linha — não o fiz. O mecanismo mediu-se pela sonda.

**A sonda** (`ph2d-gpu/examples/blocos_do_alocador.rs`: dois dispositivos no mesmo adaptador e processo,
12 rondas intercaladas; cria, limpa e larga 3 recursos; ms, mínimo · mediana;
[`sonda_nvidia.log`](handoffs/img_F38/sonda_nvidia.log), [`sonda_radeon.log`](handoffs/img_F38/sonda_radeon.log)):

| caso (×3) | NVIDIA uso | NVIDIA desempenho | Radeon uso | Radeon desempenho |
|---|---|---|---|---|
| textura 4096² rgba8 (64 MiB) | **1,92** · 2,39 | 2,40 · 3,02 | 1,05 · 12,22 | 0,93 · 10,84 |
| textura 4096² rgba16f (128 MiB) | **3,85** · 4,80 | 5,49 · 6,80 | **3,07** · 3,71 | 4,44 · 4,63 |
| textura 1024² rgba8 (4 MiB) | 0,50 · 0,55 | **0,09** · 0,16 | 4,44 · 6,23 | **0,07** · 0,15 |
| lote 512² (1 MiB) | 0,05 · 0,05 | 0,08 · 0,09 | 0,15 · 0,21 | 0,14 · 0,17 |
| lote 2048² (16 MiB) | 0,61 · 0,70 | **0,10** · 0,19 | 1,05 · 3,01 | 1,45 · 1,62 |

- As telas grandes ficam IGUAIS ou mais baratas no `uso` (passam do bloco, alocação dedicada nos dois).
- O preço do `uso` é o recurso MÉDIO (4–16 MiB) que nasce e morre em série: cada um pede um bloco novo ao
  driver (até ~1,5 ms por textura na Radeon).
- **Decisão: fica o `uso`.** No app ninguém cria recursos médios por quadro (o lote médio de dabs, 512², custa
  o mesmo); só um pincel com caixa de lote ≥ ~1 450² px cai no caso médio, ~0,5 ms por lote no pior, contra um
  quadro de ~20 ms. Do outro lado: −237 MiB na cena 41 da NVIDIA (F37 §0) e −43 MiB de VRAM e −55 MiB de GTT
  no rabisco da Radeon. Se um traço gigante vier a engasgar, `PH2D_GPU_BLOCOS=desempenho` é a prova A/B, e a
  cura certa é um pool para os buffers do lote no `StampPass` (família do Painter), não blocos grandes para
  toda a placa.

## 3 — A textura recusada não recebe trabalho de placa (o resto do F35 §7.1)

**A auditoria do mipgen** (`crates/ph2d-render/src/mipgen.rs`): o `run()` não aloca textura nem buffer —
cria uma view de origem, uma de destino e um bind group POR NÍVEL e descarta-os; nada fica em cache, logo
nada fica preso recusado. O pipeline, o layout e o sampler nascem uma vez no `MipGenerator::new`. **Mas o
defeito estava ao lado:** a textura recusada (átlas ou individual) continuava a receber trabalho de placa —
a subida dos pixels (`upload_region`, `write_pixels*`), o mipgen, as cópias de placa para placa (o preview
do Painter) — e a sua `view`, o material e todo passe que a amostrasse eram inválidos. Cada um era um
`[gpu-erro] … Se o ecrã ficou vazio e não volta, é isto` FALSO (os bytes já estavam guardados para o
renascer), à escada do contador (o `TextureView 'mipgen dst' is invalid` do F35 §1). E a leitura de volta
(`readback`, a porta das ferramentas de imagem) devolvia a textura vazia: editar e gravar por cima
apagaria a imagem.

**A cura** (`renasce.rs`, `atlas/renasce.rs`, `atlas/mod.rs`, `individual*.rs`):

- a recusada vive numa **substituta de 1×1** (mesmo formato e usos, limpa a transparente) até renascer: a
  `view`, o material e o desenho ficam válidos;
- nenhum trabalho de placa sobre ela: a subida só guarda a fonte; o mipgen, as escritas de região e as
  cópias saltam-se (o `refaz*` volta a subir e a gerar os mips);
- o `readback` da recusada devolve a fonte guardada (8 bits, 16 bits, ou transparente se era vazia).
- ⚠️ O `readback_mip` do átlas (só testes e diagnóstico) não foi mudado: sobre a recusada lê a substituta.

**Gate G28** `a_refused_texture_gets_no_gpu_work_until_it_is_reborn` (`renasce_tests.rs`, placa): um
escopo de `Validation` aberto ANTES de nascerem o átlas, uma individual de 8 e uma de 16 bits recusados;
`insert` e substituição no átlas, `replace_pixels`, `replace_pixels_region`, `copy_from_texture`,
`copy_region_from_texture` e `regen_mips` nas individuais ⇒ zero erros; o `readback` devolve os pixels
escritos (com o remendo) e os meios-float; os dois renascem. Visto vermelho antes da cura (o
`create_view` sobre a textura destruída). G17 e G18 seguem verdes.

## 4 — Mutações (as 12 sangraram, todas no G28)

| mutação | vermelho |
|---|---|
| M13 a recusada sem substituta · M14 o átlas sobe à recusada · M15 mipgen do átlas recusado | G28 |
| M16 mipgen da individual recusada · M17/M18 a escrita no nascimento (8 e 16 bits) | G28 |
| M19 `replace_pixels` · M20 `replace_pixels_region` · M21/M22 as duas cópias de placa | G28 |
| M23 o `readback` lê a substituta | G28 (o assert dos pixels guardados) |
| M24 a limpeza com os níveis da textura pedida e não da substituta | G28 |

G17 e G18 seguem verdes nas 12: cobrem o renascer, não esta lei.

## 5 — Aberto

1. Uma tela 4K que pinte sozinha (rabisco numa tela grande) — código do Painter, de outra linha (§2).
2. Um pool para os buffers do lote no `StampPass` (`ph2d-paint-gpu`, família do Painter), se um pincel de
   caixa ≥ ~1 450² px vier a engasgar com os blocos pequenos (§2).
3. O `readback_mip` do átlas (testes e diagnóstico) lê a substituta enquanto ele está recusado (§3).
4. Medir numa AMD DEDICADA (o despejo no GTT atravessa o PCIe); aqui só há a APU.
