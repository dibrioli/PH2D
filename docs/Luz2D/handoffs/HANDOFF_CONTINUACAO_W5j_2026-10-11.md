# Handoff de continuação — Luz 2D, W5j: o DONO de cada cópia (Flip e Motion na luz) (2026-10-11)

> Regras R1–R9: as do [handoff W5f](HANDOFF_CONTINUACAO_W5f_2026-10-11.md). Linha `line/Luz2D`, base `main`
> `8f20c60de`; W5d–W5j por integrar. Esta janela: `287bc1751` (item 5) · `ed24876b1` (item 4) · `5e123cfcc` (portão) · `d7429b111` (o corte do `quadro.rs`). Os ABERTOS
> 4–7 são os do [handoff W5e](HANDOFF_CONTINUACAO_W5e_2026-10-11.md) §«ABERTO (W5e → W5f)». O miolo ficou
> COMO ESTÁ (ordem do dono, 11/10); os abertos do miolo continuam no [W5h §3](HANDOFF_CONTINUACAO_W5h_2026-10-11.md).

## 1. O que existe agora

- **Item 5 — cada cópia do Motion tem DONO na luz.** `OclusaoDoMotion.donos: Vec<(entidade, cópias)>` substitui
  `objetos` (`motion_shape_luz.rs`); `donos(motion, placa)`: no dispositivo, `gpu_cook.faixas_dos_sinks().1`;
  na CPU, a faixa `vetores` do sink lida por `PlacaDeFormas::fonte` (o índice de origem de cada cópia — uma
  forma Vazia pulada não desloca as seguintes). O dono do sink = `origem::dono_do_no` (a MESMA porta da
  origem). `desenhar` desenha só as faixas pedidas (`pedidas`, juntas quando se tocam): a lista inteira se
  todas; senão upload do subconjunto (CPU) ou `copy_buffer_to_buffer` para `PlacaDeFormas::luz_copias`
  (dispositivo; o buffer do cozido já tinha `COPY_SRC`). `do_quadro` perdeu o `world`.
- **O LOD mantém as faixas**: `apply_object_lod(…, faixas, …)` encolhe `vetores` ao que ficou nítido e põe
  em `FaixaDoSink::ladrilhos` (campo novo, `ph2d-eval-motion`) os ladrilhos do sink no fim de `instances`.
  ⚠️ Por isso a origem (`motion_objeto_origem.rs`) guarda as FAIXAS com as listas no `local` e restaura as
  três juntas — senão a reaplicação com a bomba parada leria faixas pós-LOD sobre listas pré-LOD.
- **Item 4 — `NaoRecebeLuz` por objeto Flip.** `FlipCompose::nao_recebem(Vec<FlipObjectId.0>)` (a shell
  passa `pass::luz::nao_recebem(entidades, cena().nao_recebem)` em `present_luz.rs`); `LayerRef.obj`;
  `trechos()` no passe 3 (`pass.rs`): com alguém de fora e a luz acesa, cada trecho SEGUIDO de camadas com
  a mesma resposta compõe-se à parte (as outras com `opacity: 0`) e sai por `blit_com(…, recebe)`
  (`GrupoDeLuz::ligar(recebe)`). Ninguém de fora ou luz apagada = um trecho = o quadro de sempre.

## 2. Medido (RTX 5060 Ti, `dev`)

| portão | antes (vermelho) | depois |
|---|---|---|
| `so_bloqueiam_as_copias_dos_objetos_pedidos` (CPU e dispositivo; pedidos [42], [43], os dois) | [42] → +30,6 % de área (os dois discos) | −0,184 % nas 6 leituras (a base do contrato) |
| `o_dono_de_cada_copia_e_o_do_sink_dela` (fonte com uma cópia pulada, sink sem dono) | — | `[(42, 0..2), (43, 2..4)]` |
| `as_faixas_dos_sinks_seguem_o_lod` | — | vetores 0..1 / 1..2 · ladrilhos 2..5 / 5..8 |
| `quem_nao_recebe_a_luz_no_flip_fica_na_cor_de_sempre` (luz uniforme F = π) | mutação «um trecho só» → B 0,4998 | B de fora: A 0,4998 · B 1,0000; controlos (½,½) e apagada (1,1) |

O gate batched desta janela: §5.

## 3. ⚠️ ABERTO, na ordem

1. **Item 6 — as cópias de SPRITE do Motion (`source.object`, Swarm, e os LADRILHOS do LOD) não bloqueiam.**
   Não estão no `PresentWorld` (entram em `render_with_streams` como `extra`/`gpu_extra`, `present.rs:235-337`),
   logo a C0 nunca as vê. Desenho: uma `CamadaDeOclusao` que desenhe `RenderInstance` (CPU `pump.instances`,
   ou o buffer `gpu_cook.instances()` + `texture_runs()`) com `SpritePipeline::new(gpu, FORMATO_DA_OCLUSAO)`
   (a MESMA `sprite.wgsl`, luz neutra) e `Camera2d` = a janela `mundo`; os donos por `faixas.instancias ∪
   faixas.ladrilhos` (CPU) ou `faixas_dos_sinks().0` com os `texture_runs` cortados (dispositivo). ⚠️ O
   empréstimo: `gravar_com` recebe `&SpriteRenderer` e o desenho pede `&mut` (o `camera_buffer`) — decidir
   entre passar o renderer à família pelo trait ou partir o `gravar_com`. Medir primeiro: uma fixture com
   cópias de quad branco de dono 42 ⇒ hoje a área na camada é 0. A rota Vello (`PH2D_FORMAS_NA_PLACA=0`)
   continua sem captura.
2. **Item 7 — o «distantes» (F4) e o ENTORNO não veem Flip/Motion fora da grade** (o ABERTO 5 da W5d, para
   os vetores, é a mesma família). Precedente: `Distante` em `ph2d-app-light2d/src/vetor.rs` (caixa por
   caminho → `tapa_fora_da_grade` → janela num atlas → `sprites.distantes`, `quadro.rs` ~554). Pedido ao
   trait: `caixa(entidade)` + desenhar com `mundo` = a caixa numa janela. Flip: a tesselação × `model` (CPU).
   ⚠️ Motion no dispositivo NÃO tem caixa na CPU: uma redução na placa lida um quadro depois, ou a caixa
   por sink no cozido. Medir primeiro no `a_luz_nao_muda_com_a_vista.rs` (o pan com o opaco fora da grade,
   hoje polígono/sprite/vetor) com um objeto Flip.
3. **Da mesma família do item 4 (§0.10), achados nesta janela:** os CAMINHOS do vetor e as FORMAS do Motion
   também não honram `NaoRecebeLuz` por objeto — a colagem acende a faixa inteira (`BandSource::{Vector,
   Formas}.recebe_luz()`, `band_blit.rs:50`). Cura no molde dos `trechos` do Flip: o vetor parte as faixas
   do `frame_order` pela resposta; o Motion usa os `donos` do item 5.
4. A pergunta ao dono continua pendente: o vão FECHADO da pilha (cena 8) conta como exterior?
5. Os do W5h §3 (com os vermelhos de propósito, mais os quatro do §5 abaixo) e os W5e 1–3, 8–11.

## 4. ⛔ Recusas (não reconstruir)

| tentativa | por quê |
|---|---|
| luz aplicada a cada FATIA do Flip antes do compositor | a frescura (`needs_stage`) guarda fatias: re-injetar todas a cada quadro com a luz, ou guardar uma cópia apagada por camada (N × ecrã) |
| decompor por cor (`L·C_r + C_n`, dois compositores) | exato só para blends lineares e o dobro de fatias |
| máscara por píxel de quem não recebe | errada onde os objetos se sobrepõem |
| `Copias` com início (bind com offset) | o offset de storage tem de ser múltiplo de 256 B (4 cópias); a cópia placa→placa não tem restrição |

## 5. Gate batched e binário do smoke

`nextest-impacted` (base `5e9c86b54`, placa, load 16–29): 5 053 / 5 065. Os vermelhos, todos ANTERIORES —
abate feito (os ficheiros desta janela voltados a `5e9c86b54`, a mesma corrida): ⚠️ `a_cena_7_ensina_o_traco_do_flip`
(«sem faces»), `o_traco_do_flip_recebe_a_luz_como_o_sprite` (3,12e-3 > 1,48e-3), `a_cena_5_ensina_o_volume` e
`a_cena_6_ensina_as_formas_desenhadas` («sem faces») reprovam IGUAIS na base — a família dos vermelhos de
propósito da W5h (o miolo por difusão mudou o interior; reescrever contra a lei nova, W5h §3.3) e NÃO estavam
na lista dela: junte-os. Mais os seis conhecidos (`o_interior_do_opaco_nao_depende_da_posicao_na_grade` em
timeout de 180 s) e `o_oclusor_fino` em timeout sob carga (o R9 da W5f). O `workspace_src_files_under_loc_cap`
estava vermelho desde a W5i (`quadro.rs` 779): curado por corte em `d7429b111` (entorno e leituras em irmãos,
674). clippy `--all-targets -D warnings` (as seis crates + a shell) e `fmt --check`: limpos.

Fotos (`fotos_W5j/`, tela virtual): [cena 7](fotos_W5j/cena7.png) · [cena 8](fotos_W5j/cena8.png) — as
formas das duas pilhas fazem sombra; as cópias vermelhas do Swarm NÃO (item 6). ⚠️ A abertura a frio na tela
virtual (HOME isolado ⇒ cache do driver frio) levou 12–45 s até acender sob load 23–58: fotografe com espera ≥ 45.

Binário: `cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida: `Finished smoke profile [optimized]
target(s) in 0.30s`.

## Smoke do dono

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Luz2D && PH2D_LIGHT2D_SMOKE=7 cargo run -p ph2d-host-desktop --profile smoke`
2. Espere a luz acender. Na Hierarchy clique em «Traço do Flip»; no Inspector, «Add Component» → «Unlit».
3. A onda roxa e as duas bolas de CIMA (o desenho do Flip) ficam na cor cheia, sem o claro-escuro da lâmpada;
   as duas de BAIXO (as imagens) continuam com o claro-escuro; as sombras de todas continuam no chão.
4. Errado: as bolas de cima continuam sombreadas, ou as sombras delas somem.
