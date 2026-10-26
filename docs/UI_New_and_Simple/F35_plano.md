# F35 — A memória da placa: o que o FX segura depois do zoom extremo, e a placa cheia

> Aberto pelo handoff F34 §6 (1, 2, 3). Uma família só (CLAUDE.md §0.10): **a placa sem memória** —
> por nós (o FX que não devolve) e por outro programa (o `ollama`). Os números deste plano são
> MEDIDOS; o que ainda não está medido diz-se.

## 1 — O FX segura o pico para sempre (MEDIDO)

Sonda: `crates/ph2d-app-vec/src/fx_live_memoria_gpu_tests.rs::fx_memoria_sonda` (`--ignored`, placa).
A Nuvem da cena 41 (Bevel Depth 1,4 + Drop Shadow), janela 2560×1080 (o monitor do dono), a roda de um
entalhe por quadro da abertura (`h = 10`) ao zoom máximo (`h = 0,5`), 40 quadros de pan, a roda de volta
e 300 quadros parados. Os bytes são os que o `FxLive` SEGURA (`FxLive::gpu_bytes`), e o `amdgpu` é o
contador do driver (a vez da placa corre na Radeon de bordo; o `nvidia-smi` não vê o processo).

| fase | trabalho da pilha | saída | mostradas | `amdgpu` (máquina) |
|---|---|---|---|---|
| filtro da cena 41 — abertura | 55 MiB | 6 | 8 | — |
| … zoom máximo, pan, volta, parado 300 | **152** | **19** | 12 | — |
| **pior caso** (sombra de borrão `radius 2`, G11) — abertura | 215 | 26 | 11 | 1 069 |
| … zoom máximo | **2 048** | **256** | 12 | 4 426 |
| … de volta à abertura e **parado 300 quadros** | **2 048** | **256** | 12 | **4 169** |

- As texturas de trabalho (4 × `Rgba16Float`) e a `saida` são **grow-only**: o pico fica para sempre.
  O «~2 GiB» do handoff F34 é o pior caso real: a janela do job chega ao lado da placa (8192²) quando a
  margem da sombra cresce com o zoom.
- O driver mostra **~3,1 GiB** a mais depois da volta; os nossos contadores explicam ~2,1. O resto
  (~1 GiB) é o **pool interno do Vello do scratch** (`vello-0.10.0/src/wgpu_engine.rs`, `ResourcePool`:
  os buffers voltam ao pool e nunca saem — não há API para o esvaziar; só largando o `Renderer`). A
  intermédia do scratch já é do tamanho exacto (desceu 257 MiB na volta).
- As texturas MOSTRADAS já encolhem (lei `serve`, F34) — ficam fora da cura.

**O preço de soltar** (`fx_memoria_sonda_do_preco`, mínimo de 7, load ~19): `VelloPass::new` 10,1 ms ·
`FxStackPass::new` 10,7 ms. ⇒ soltar tem preço de um quadro no PRÓXIMO gesto; solta-se só o que SOBRA.

### A cura (feita)

`FxLive` conta os quadros PARADOS (recook sem nenhuma forma a re-cozer). Ao chegar a `N`, solta o que
SOBRA face ao que a vista de agora pede (lei `sobra`: um lado passa da `capacidade(preciso)`; a 1.ª
versão usava a folga do `serve`, `capacidade(2 × preciso)`, e na cena normal deixava 4× a área presa —
171 MiB contra 61):
as texturas de trabalho, a `saida` e o renderer do scratch (pelo maior lote que ele já rasterizou). Uma
cena parada na abertura não solta nada — não paga os 10 ms à toa. As texturas REGISTADAS não se tocam
(o G6 continua a contar as mesmas trocas).

`N` — o gesto não pode soltar a meio: a roda contínua chega a ≥ 5 entalhes/s, < 200 ms entre recooks
(≤ 29 quadros a 144 Hz). `N = 120` (0,8 s a 144 Hz, 2 s a 60 Hz). Uma pausa maior a meio de um gesto paga
UM quadro de ~20 ms ao recomeçar.

**Depois da cura** (mesma sonda, o `N`-ésimo quadro parado):

| cenário | `N - 1` parados (FX / `amdgpu`) | `N`-ésimo (FX / `amdgpu`) |
|---|---|---|
| filtro da cena 41 | 183 MiB / 1 337 | 12 MiB (só as mostradas) / **1 081** |
| pior caso (sombra 8192) | 2 316 MiB / 4 169 | 12 MiB / **1 344** |

Gates: G12 `what_the_extreme_zoom_allocated_is_left_over_back_at_the_opening_view` (CPU) e G13
`after_the_extreme_zoom_a_still_scene_gives_the_memory_back` (placa). Mutações M1–M7 (subagente
`mutacao`): M1, M2, M3, M5, M7 sangram no G13; M6 no G12; **M4 (`||`→`&&`) sobreviveu** — o zoom
extremo passa os dois lados ao mesmo tempo — e passou a sangrar com o caso de UM lado no G12. O G6
ficou verde nas sete.

## 2 — A placa cheia por outro programa (REPRODUZIDO 26/10, ao vivo)

O `ollama` estava de novo com 13 GB na NVIDIA (946 MiB livres). A foto da cena 41 em 2560×1080
(`fotografa_cena.sh`) saiu **cinza liso**: sem painéis, sem forma, sem aviso. O log:

1. `[gpu-erro] Out of Memory` no 1.º quadro — antes de a cena 41 carregar;
2. depois, para sempre: `Buffer with 'vello.lines_buf' label is invalid` / `vello.segments_buf` /
   `TextureView 'mipgen dst' is invalid` (a escada chega a `#1000`).

O Vello pede buffers FIXOS (`vello_encoding-0.10.0/src/config.rs:401-407`: `lines`, `segments`,
`tiles`, `seg_counts` de `2^21` elementos, `ptcl` de `2^23`). Uma alocação recusada não volta como
`Err` do `render_to_texture`: o wgpu devolve um buffer INVÁLIDO e levanta um erro de dispositivo.

**Hipótese (a medir antes de curar):** o buffer inválido VOLTA ao pool do Vello e é reusado em todo
quadro ⇒ o editor **não recupera nem quando a placa liberta** (a assinatura do report de 21/09: *«não
recupera até fechar o app»*). Se confirmada, a cura tem três partes:

1. **Saber:** o `ph2d-gpu` conta as recusas de memória (sem novo campo no `GpuContext`: o literal dele
   existe num teste de outra linha).
2. **Avisar onde o aviso é visível:** o toast é desenhado pelo MESMO Vello que falhou ⇒ o aviso vai
   também ao TÍTULO da janela (desenhado pelo sistema), em texto i18n.
3. **Recuperar e aliviar:** largar os renderers do Vello envenenados (o pool), soltar o transitório
   do FX (a cura do §1) e tentar de novo com intervalo, até um quadro passar.

O reprodutor sem o `ollama`: `crates/ph2d-gpu/examples/ocupa_a_placa.rs` (ocupa até sobrarem `<livre>`
MiB e segura `<segundos>`).

## 3 — O vermelho da cena 3 do CodeEditor

(a correr sozinho, com o `loadavg` ao lado — depois das curas, para não competir com elas.)

## Gates (a escrever, cada um visto vermelho por mutação)

- G12 (CPU): a lei de soltar — parado `N-1` não solta; `N` solta o que sobra; um gesto nunca solta;
  a abertura parada nunca solta.
- G13 (placa): a sonda do §1 com asserção — depois de `N` quadros parados os bytes do FX voltam ao
  que a abertura segura; o G9 continua verde.
- G14/G15 (§2): definidos depois da medição da hipótese.
