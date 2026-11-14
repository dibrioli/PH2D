# N10 — família 1: toque (pen-down) e Ctrl+Z a 4K (agente F1, 14/11)

Base: HEAD `36a312482a` = `target/ph2d-base-36a312`. Uma linha por id: estado · antes → depois · sha · porquê.

## Conferência do código (regra 3)

- O que o plano chama «aberto» do Ctrl+Z do Wet (`poe` serial em `Registo::restaura`, `reconta_teto`/`reconta_real`) **já está
  feito** no HEAD: `d0f75b0730` (inverso/restauro no pool, recontagens por pedaço), `3f4cd0a704` (`Registo::troca`: o tile do
  registo troca de conteúdo com a grade, sem alocar), `560576a150` (a troca chega aos esparsos), `1f8b6127d6` (o Ctrl+Z vivo não
  zera o fluxo). Nenhum destes era antepassado de `234ef11b64` (a matriz n.º 7 dos 46–51 ms): o número do BACKLOG é anterior.
- `FotoDoMotor::de` (`PlanoEmTiles::de`) já compara/copia por tile no rayon; `repoe_sobre` já salta os tiles partilhados.

## Itens

Medição de partida (14/11, `mede_pen_up_4k.sh matriz 3840x2160 papel target/ph2d-base-36a312` com `CURTO=1`, Radeon RADV,
1 amostra por pincel, **load 1,5 → 10,5** com outra matriz de outro agente a correr no mesmo binário em `:93` ⇒ indicativo,
não veredito; saída em `target/` não comitada):

| pincel | toque ms | traço p50/p95 | soltar | Ctrl+Z ms | carga 1-min |
|---|---|---|---|---|---|
| Express Oil (wet) | 34,7 | 24,1 / 28,5 | 28,3 | **71,9** | 1,5 |
| Filbert (wet) | **75,3** | 60,2 / 79,1 | 61,7 | **60,1** | 7,6 |
| Wet Liner (wet) | 48,1 | 26,4 / 34,5 | 31,3 | **73,7** | 9,6 |
| omissão wet | 28,2 | 16,7 / 19,0 | 28,3 | 32,0 | 10,5 |
| omissão digital | 15,9 | 14,6 / 16,1 | 15,5 | 18,9 | 9,3 |
| Finger Tip (digital) | 21,9 | 17,0 / 27,4 | 17,8 | **44,9** | 8,3 |

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 (Ctrl+Z Wet 46–51; Finger Tip 35; toque Filbert 43) | **parcial (medido, sem cura)** | Ctrl+Z Express Oil 71,9 · Wet Liner 73,7 · Filbert 60,1 · omissão wet 32,0 · Finger Tip 44,9; toque Filbert 75,3 (load 1,5–10,5, 1 amostra) | este commit | A decomposição por fase é a sonda `sonda_as_fases_do_ctrl_z_wet_a_4k` (fila n.º 6). A hipótese do código: depois das curas da troca, o que resta do quadro do Ctrl+Z vivo é `FotoDoMotor::de` (lê `oleo_antes`+`flow_x`+`flow_y` inteiros das camadas × a foto anterior, e aloca `Arc` novo por tile mudado: faltas de página) + `repoe_sobre` (no redo sem `ja`: reescreve os 3 planos) — memória, não aritmética. |
| B17.4 (dono único da tela) | parcial (cura no HEAD; falta a medição pedida) | 1.º toque digital 4K no app = 15,9 ms (o quadro inteiro; o 4,91 do BACKLOG é a sonda) | — | a cura (journal da tela em release, `before`/cursor elididos) está no HEAD; a medição ao nível da sonda é `--ignored` (não corre num subagente). |
| B17.5 (undo por tiles) | parcial (inalterado) | — | — | o que falta é A×B no app de dois toques opostos (`CANTOS=`), com o binário do pai da onda 15 D construído; a sonda `what_a_diagonal_ctrl_z_copies_at_4k` está na fila (n.º 8). |

## Bloqueio do método nesta rodada

O hook `prazo-do-agente` recusa a um subagente todo teste `--ignored` (as sondas de fase). Postas na fila da linha
(`bash scripts/fila-pesada.sh ver`, n.º 6–8): `sonda_as_fases_do_ctrl_z_wet_a_4k` (tool-painter), `sonda_o_ctrl_z_wet_a_4k`
(wet-paint), `what_a_diagonal_ctrl_z_copies_at_4k` (B17.5). Sem a decomposição por fase não há red-first para uma cura nova.

## Próximo passo exato (B18.11)

1. Correr a fila n.º 6 e 7 (o coordenador) e ler as linhas `volta: FotoDoMotor::de`, `volta: caixa + motor undo/redo`,
   `volta: repoe` de Express Oil/Wet Liner a 4K.
2. Se `de` + `repoe` dominam: fundir no undo (`undo_vivo.rs::wet_volta_do_undo`, `refaz == false`) a foto e o repoe numa
   passada por tile em `foto_em_tiles.rs` (`PlanoEmTiles::de_e_repoe(v, antes)`: tile igual ⇒ partilha o `Arc` e não escreve;
   diferente ⇒ copia o vivo para o `depois` e escreve o `antes`), com os `Arc` novos tirados de UMA alocação (sem faltas de
   página por tile); oráculo `#[cfg(test)]` = o par `de` + `repoe_sobre` de hoje, ao bit, nas duas leis.
3. Se o motor domina: o que falta é saber QUE tiles a água escreveu (geração por tile no `Grid`, escrita em `solver.rs` /
   `flow_jacobi.rs`) — pegada do F4; reportar.
4. Filbert toque 75 ms: a matriz não separa o pen-down do 1.º dab; correr `mede_pen_up_4k.sh` com `PERF=1` (`STRIP=none`) só no
   Filbert antes de curar.

## F1b (14/11) — a «caixa» é a troca do motor; decomposição + poupança das linhas iguais

Fila n.º 6/7 (load 8–10): `volta: caixa + motor undo/redo` = 30,9 ms (Wet Flat 240) / 48,0 (Dabber 120). O código da fase
é só `Engine::undo_com`/`redo_com`; `caixa_do_restauro`, `forma_global`, `Registo::abre` e `depois_do_restauro` são escalares
⇒ é a `Registo::troca` de ~958 tiles × todos os planos (~47 B/célula no aquarela, mais os esparsos no óleo): **largura de
banda**, não aritmética. A troca cega lê E escreve os dois lados mesmo nos planos que o passo não mudou.

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 sub-fases | feito (instrumento) | — | este commit | `ph2d_wet_paint::sonda_fases` (feature `sonda-fases`, só no dev-dependency do Painter; fora disso as marcas são vazias): a sonda s6 imprime `motor: …` e `troca: …` (lista, copia_os_outros, ajusta, troca_os_planos, poe_no_pool + larga, reconta, fluxo, finos + escalares) e o CENSO por plano (tiles iguais ao bit à grade / tiles com o plano). |
| B18.11 cura 1 | **feita, por medir a 4K** | a medir (s6 com e sem a chave) | este commit | `troca_no_pool`: a linha de tile igual ao bit à grade não se troca (só se lê); `PH2D_TROCA_POUPA=0` = antes. Gate `a_troca_poupa_as_linhas_iguais_e_da_o_bit_da_troca_cega` (bit × troca cega × oráculo `ANTIGO`, 3 leis; vermelho com `PH2D_TROCA_POUPA=0`); `a_troca_da_o_bit_do_inverso_e_do_restauro` verde. tinta/pigmento trocam sempre. |

Próximo passo exato: o coordenador corre a sonda s6 (`sonda_as_fases_do_ctrl_z_wet_a_4k`, o mesmo comando da fila n.º 6) duas
vezes, com e sem `PH2D_TROCA_POUPA=0` (o hook recusa ao subagente até o `fila-pesada.sh pôr` com essa sonda). Ler o censo: se os
planos iguais são poucos, a cura 1 vale ~0 e o caminho é o motor **não marcar** tiles que o passo não mudou (o solver marca a
água viva das 3 pinceladas: pegada F4) ou guardar no `TileFino` só os PLANOS que mudaram (`Registo::marca`); se a sub-fase maior
for `copia_os_outros`/`poe_no_pool` (os esparsos do óleo, Dabber 48 ms), levar os esparsos ao pool por pedaço (hoje
`CampoEsparso::poe_no_pool` é UM job em série).

## F1c (14/11) — a medição da cura 1, o censo fora do TOTAL, o fluxo no pool

Medição (janela principal): s6 `sonda_as_fases_do_ctrl_z_wet_a_4k`, 7 rodadas, 2 pares intercalados `PH2D_TROCA_POUPA=1/0`,
load 5,8–7,6 (`uptime` em `target/n10/f1c/load.txt`; saídas `target/n10/f1c/poupa{1,0}_r{1,2}.txt`, não comitadas). Mínimo, ms:

| fase (undo) | Wet Flat 240 poupa=0 → 1 | Dabber 120 poupa=0 → 1 |
|---|---|---|
| `troca: troca_os_planos` | 22,8 / 23,0 → 16,4 / 16,1 | 41,6 / 42,3 → 30,8 / 30,4 |
| `troca: fluxo` | 6,5 / 7,0 → 6,4 / 5,9 | 5,2 / 5,6 → 5,4 / 5,1 |
| `troca: reconta` · `FotoDoMotor::de` · `repoe` (redo) · `restore_model` | 0 · 2,9 · 3,3 · 0 | 2,4 · 4,1 · 5,6 · 2,3 |
| TOTAL **com** o censo (29 / 73 ms) | 71,1 / 73,3 → 60,7 / 59,6 | 132,9 / 135,4 → 122,8 / 120,9 |
| TOTAL sem o censo (estimado: menos a linha do censo) | ~42–44 → ~30–31 | ~59–62 → ~48–49 |

A cura 1 vale **−7 a −11 ms** por undo/redo. ⚠️ A linha `troca: (censo da sonda)` (29–76 ms) era o PRÓPRIO censo e entrava no
TOTAL: a sonda mentia. Censo (tiles iguais ao bit à grade / tiles com o plano, undo+redo): **Wet Flat** — relevo, vela, bloom
100 %; active, susp_rgb 59 %; wet 53 %; film/susp/sett/sett_rgb 18 %. **Dabber** — film, susp, susp_rgb, vela, active, bloom
100 %; sett/sett_rgb/relevo 79 %, altura 78 %, espessura 81 %.

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 cura 1 | **feita, medida** | troca_os_planos 23,0 → 16,1 (Wet Flat) · 42,3 → 30,4 (Dabber) | `84c5611da8` | acima |
| B18.11 sonda honesta | feito | censo 29–76 ms fora do TOTAL | este commit | o censo só corre com `PH2D_SONDA_CENSO=1` (lida uma vez) e a fase `sonda_fases::CENSO_FASE` sai do TOTAL da s6. |
| B18.11 cura 2 (`fluxo`) | **feita, por medir a 4K** | 6–7 ms em série → a medir (`PH2D_FLUXO_SERIE=1` = antes) | este commit | `troca_o_fluxo`: os tiles de `vel_x`/`vel_y` trocam no pool (`troca_no_pool`, um job por faixa, com a poupança das linhas iguais); plano vazio ⇒ tile vazio (o `tira` de antes). Gate `o_fluxo_no_pool_da_o_bit_do_fluxo_em_serie` (× série `FLUXO_EM_SERIE` × oráculo `ANTIGO`, 3 leis, vel ≠ 0 conferida); mutação «não troca `vel_y`» ⇒ vermelho «undo (lei 0): × série»; os 4 gates `troca` verdes. |
| B18.11 cura 3 (máscara de planos por tile) | **aberto** | — | — | ver próximo passo. Decidir no undo já é o que a cura 1 faz (lê os dois lados); decidir na ESCRITA pede que cada sítio que marca diga QUE planos vai escrever — os do solver (`marca_passo`/`marca_do_passo` em `painter/historia.rs`, chamados de `solver.rs`/`flow_jacobi.rs`, com edições por comitar de outro agente nesta rodada) não foram tocados. |

Próximo passo exato: (1) a janela corre a s6 A×B do fluxo (comando no retorno do F1c). (2) cura 3: `Registo::marca_planos(g, ret,
Planos)` (máscara de bits por plano do `tile_fino!`); `TileFino` ganha `guardados: Planos` e só tira os planos da máscara (os
outros ficam «não guardado», distinto de «vazio na base»); `marca` sem máscara = todos (o de hoje); a troca/`copia_os_outros`/
`poe_no_pool`/`inverso`/`restaura_com` saltam o plano não guardado; uma 2.ª marca do mesmo tile com planos novos tira-os NESSE
instante (a grade ainda é a base deles). Sítios: `marca_dab` (o dab: quais planos o `deposit`/`stamp` escreve) e o passo do
solver (por estágio: o Wet Flat nunca escreve relevo/vela/bloom ⇒ ~28 % dos bytes da troca). Oráculo = máscara cheia, ao bit.

## F1d (14/11) — a medição da cura 2 (fluxo) e a foto + repoe numa passada

Medição (janela principal): s6, 7 rodadas, `PH2D_FLUXO_SERIE=0/1` × 2, load **10–23** (`target/n10/f1d/load.txt`; saídas
`serie{0,1}_r{1,2}.txt`, não comitadas). ⚠️ As corridas `serie1_*` compilaram já com a passada única (abaixo) a meio de ser
escrita: só a linha `troca: fluxo` é comparável entre `serie0` e `serie1` (as fases `de`/`repoe` não). Mínimo, ms (undo / redo):

| fase | Wet Flat 240 série → pool | Dabber 120 série → pool |
|---|---|---|
| `troca: fluxo` | 8,0–8,4 / 8,2–10,9 → **2,9–3,0 / 2,9** | 5,5–5,6 / 5,4 → **2,7–2,9 / 2,8–3,1** |
| `FotoDoMotor::de` + `repoe` (undo, pool, sem a passada) | 2,8 + 1,9 | 4,2 + 0,3 |

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 cura 2 (`fluxo`) | **feita, medida** | 5,4–8,4 → 2,7–3,0 ms por undo/redo | `e2a0076f8d` | acima |
| B18.11 cura 4 (foto + repoe numa passada) | **feita, por medir A×B** | `de` 2,8/4,2 + `repoe` 1,9/0,3 (undo) → a medir com `PH2D_FOTO_FUNDE=0/1` | este commit | `PlanoEmTiles::de_e_repoe` (uma passada por faixa de tiles: igual ao bit ⇒ partilha o `Arc` de `antes`, não escreve; diferente ⇒ o vivo vai ao «depois», o `antes` à grade, com o tile quente na cache) e `FotoDoMotor::de_e_repoe` (o relógio do «depois» tirado antes do motor). Só no undo com o fluxo inteiro reescrito (a condição do `ja`); o redo e o resto ficam no par. O tile novo junta-se num tampão reusado por fio (`copia_tile`): sem o `Vec` por tile que o `Arc::from` recopiava (também no `de`). «Arcs de uma alocação só» RECUSADO sem `unsafe` (o crate é `forbid(unsafe_code)`; `Arc<[f32]>` por tile tem o seu cabeçalho). Gates: `a_passada_unica_da_o_bit_do_par` (plano, bits e partilha do «depois», bytes novos; mesma grade e fora dela) e `a_passada_unica_do_undo_da_o_bit_do_par_de_e_repoe` (aguada e Dabber, undo/redo ×6, tela + planos + bytes retidos; oráculo `SEPARADA`); `o_repoe_sobre_o_depois_da_o_bit_do_repoe_inteiro` verde (agora o oráculo é o repoe inteiro × a passada). Mutação «a passada não escreve o `antes`» ⇒ vermelho «o plano» (unidade) e vermelho no e2e (`repoe_sobre_tests.rs:94`, tela/planos ≠ ao 1.º undo). |

Próximo passo exato: (1) a janela corre a s6 com `PH2D_FOTO_FUNDE=1` e `=0`, 2 pares intercalados, e lê `volta: FotoDoMotor::de` +
`volta: repoe` do undo. Se a passada não baixar a soma (ela é memória: lê o vivo e o `antes`, escreve o diferente), reverter a
chamada em `undo_vivo.rs` e deixar o `copia_tile`. (2) Filbert toque, B17.4/B17.5 e a família 7: não iniciados nesta janela (F1e).

## F1e (14/11) — a cura 4 medida e retirada; o toque do Filbert localizado

**Cura 4 (A×B, janela principal):** s6, 7 rodadas, `PH2D_FOTO_FUNDE=1/0` × 2 intercalados, load **8–42**
(`target/n10/f1d/load_funde.txt`; saídas `funde{0,1}_r{1,2}.txt`, não comitadas). Undo, mínimo, `de` + `repoe` (ms):

| pincel | funde=0 r1 · r2 | funde=1 r1 · r2 | mínimo 0 → 1 |
|---|---|---|---|
| Wet Flat 240 | 3,49+2,33 = 5,82 · 2,84+1,85 = **4,69** | 0+4,69 = **4,69** · 5,08 | 4,69 → 4,69 (igual) |
| Dabber 120 | 4,20+0,30 = **4,50** · 5,69+0,36 = 6,05 | 4,74 · 6,15 | 4,50 → 4,74 (+0,24, pior) |

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 cura 4 (foto + repoe numa passada) | **recusada, medida** | soma `de`+`repoe` 4,69/4,50 → 4,69/4,74 | este commit | a passada única lê o vivo E o `antes` e escreve o diferente: os mesmos bytes do par, só reordenados — memória, sem ganho. Tirados `PlanoEmTiles::de_e_repoe`, `FotoDoMotor::de_e_repoe`, a chave `PH2D_FOTO_FUNDE` e os dois gates dela; ⛔ não reconstruir. Fica o `copia_tile` num tampão reusado, agora também no `PlanoEmTiles::de` (`map_init`: um tampão por fio, não um `Vec` por tile). |
| Filbert toque 57–75 ms | **localizado, não curado (pegada F2b/F4d)** | `INPUT down` máx. 31–64 ms (fora do quadro) | — | `mede_pen_up_4k.sh perf 3840x2160 Filbert:wet` (binário com símbolos do F6, load 25–46, indicativo: os quadros PARADOS também davam 16/60 ms). Nos ~87 ms depois do `down` o fio principal tem 3 amostras (espera) e o operário do Wet faz o 1.º dab SÍNCRONO: `trail::bico::pesos_da_recolha`, `grid::aparencia::pousa`, `grid::oleo_celula::pousa` (o depósito: **F2b**), `oleo_real::CampoAltura::de_com_em::<EmSerie>` (o relevo em série: **F4d/render**), `render::pigment_row`, `oleo_luz::luz_directa`, `oleo_sombra::rebote_no_papel`, `composite::pixel::laco_de_pixel`. Do toque/undo só `undo_delta::confine::janela_da_tinta` (~2 amostras) e um `copia_tile` (foto do antes): ≤ 3 ms. |

Próximo passo exato (toque Filbert): F2b leva o 1.º dab do Óleo Real para fora do fio do evento; F4d põe `CampoAltura::de_com_em`
no pool (`EmSerie` → por faixa) para o toque. Medir com `mede_pen_up_4k.sh perf` em load < 5 (`STRIP=none`).

## F1f (14/11) — cura 3 do B18.11, lado do registo

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 cura 3 (máscara de planos por tile) | **feita do lado do registo; o ganho a 4K espera a cura 3b** | tile do dab guardava 19 planos → 18 (sem `bloom`, −1 B/célula); troca/cópia/restauro saltam o plano não guardado | este commit | `Planos` (um `bool` por plano do `tile_fino!`; `bits`/`de_bits` no `em_bytes`), `TileFino.guardados`, `Registo::marca_planos` (a 2.ª marca tira os planos novos NESSE instante: `acrescenta`), o `inverso` guarda a união dos planos do par, restauro/`troca` juntam ao tile do registo os planos que só o `extra` guarda (`junta`: a base deles é a do registo; `snapshot/restauro.rs`), `copia_os_outros`/`troca_os_planos`/censo saltam o não guardado; oráculos `ANTIGO` com a mesma junção. Só o `marca_dab` estreita (`PLANOS_DO_DAB` = todos − `bloom`): o `bloom` só o escrevem o passo (`solver/flow_jacobi/backrun.rs`) e acções de tela. `relevo`/`vela` NÃO saem do dab — o depósito escreve-os (`trail/transfer.rs:166-169`, `:286-288`) quando a ponta os traz (o censo 100 % do Wet Flat é dado, não lei). Gate `a_mascara_do_dab_da_o_bit_da_mascara_cheia` (3 leis, traço direto sem água + undo/redo com `extra` + história inteira; oráculo `MASCARA_CHEIA`, ao bit; controlo: há tiles estreitos); os 7 de `grid::snapshot::` verdes; mutação «o dab também sem `wet`» ⇒ vermelho «1: undo (lei 0)», plano `wet`. |

**Cura 3b (o ganho a 4K; pegada do solver, F4f a editar):** o sítio é `painter/historia.rs::marca_do_passo` (chama
`marca_ret`, máscara cheia) ← `marca_passo`, chamada por estágio em `solver.rs`/`flow_jacobi.rs`. Trocar por
`self.marca_ret_planos(x0, y0, x1, y1, PLANOS_DO_PASSO)`. Achado estático (grep das escritas na crate): o passo **nunca**
escreve `relevo` nem `vela` (só `trail/transfer.rs`, `grid/canvas_ops.rs`, `grid/patch.rs`) ⇒ `PLANOS_DO_PASSO` = todos −
{`relevo`, `vela`} no mínimo (8 B/célula dos ~47 da aguada, ~17 % da troca); confirmar por estágio os preguiçosos/esparsos
(`altura`…`pigmento`, `material`, `aparencia`, `oleo`) antes de os tirar. O gate desta cura serve tal e qual (o oráculo cobre
toda marca); depois a s6 A×B com uma chave em execução `PH2D_MASCARA_CHEIA=1` (a criar no `marca_planos`, lida uma vez — hoje
só o `thread_local` de teste).

Não feitos nesta janela (prazo 45 min): B17.6 resto (`mask.rs:381` + 18 `None`), B17.5 (ler `target/n10/f1/s8.limpo.txt`),
B17.7, B17.8, B42.4, B42.5.

## F1g (14/11) — cura 3b do B18.11 e o commit estrutural da troca

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B18.11 cura 3b (o passo marca sem `relevo`/`vela`) | **feita ao bit; o A×B a 4K é `--ignored` (comando abaixo)** | o tile que o passo da água abre guardava 19 planos → 17 (−8 B/célula dos ~47 da aguada, ~17 % da cópia por tile do passo) | este commit | `historia.rs::marca_do_passo` → `marca_ret_planos(.., PLANOS_DO_PASSO)` (todos − {`relevo`, `vela`}). Censo das escritas na crate: `relevo`/`vela` só em `trail/transfer.rs` (o depósito), `grid/canvas_ops.rs` (acções de tela, `marca_tudo`) e `grid/patch.rs`; o sim e a secagem só os LÊEM (`drying/tinta.rs` desestrutura `vela` e re-empresta-o como `&[f32]`; `espelho.rs` escreve o espelho, não a grade). Os preguiçosos `altura`…`oleo` ficam: têm escritores em `grid/<plano>.rs` chamados por solver/secagem (`crista` por `oleo_campo.rs`, `material` por `oleo_cache.rs`) — tirá-los pede o censo por estágio. Chave em execução `PH2D_MASCARA_CHEIA=1` (lida 1×, `OnceLock`, `grid/snapshot.rs::mascara_cheia_pela_chave`): toda marca guarda tudo — a variante A. Gate novo `a_mascara_do_passo_da_o_bit_da_mascara_cheia` (3 leis; o passo que abre o tile ANTES do dab com o registo aberto, o dab antes do passo, undo/redo, a história inteira; oráculo `MASCARA_CHEIA` ao bit; controlo: há tiles estreitados PELO passo) e `a_mascara_do_dab_da_o_bit_da_mascara_cheia` verdes; `historia` (21) e `grid::snapshot::` (8) verdes; mutação «o passo também sem `film`» ⇒ vermelho «3: undo (lei 0)». |
| troca.commit da 1.ª troca Wet (49–72 ms, F5d) | **parcial: sonda fina instalada, causa por medir** | — | este commit | O commit estrutural NÃO copia a camada nem fotografa o motor: o `snapshot_model` é de `Arc`s e a alocação é a mesma 3–4 MB nos casos de 4 ms e de 72 ms (`target/n10/f5d/sonda_troca.txt`), e o `volta` do Wet Wash também deu 51 ms (não é só a 1.ª). Logo o custo é VARREDURA, não cópia. Candidatos em `UndoController::record_structural_hinted`: `PlaneDeltas::split` sem `hint` — o `StoredImages::split` faz `diff_window` da imagem da camada de saída (33 MB × 2 a 4K) quando o `flush_active_to_images` acabou de a reescrever (a água que correu depois do pointer_up), e `relief_maps`/`vidro_map` o mesmo nos planos de relevo da camada Wet. Partes novas (`cfg(test)`, `sonda_partes`): `commit.auditoria` · `commit.after` · `commit.marca` · `split.tela` · `split.vidro` · `split.relevo` · `split.janelas` · `split.imagens` · `commit.split` · `commit.descarta` · `commit.cursor` · `commit.redo` · `commit.cap`. Cura de estado da arte quando a parte gorda aparecer: o flush declara a janela que escreveu (o casco dos tiles do Wet) no `WriteState` e o `split` das imagens/relevo usa-a (só o sujo) — ⚠️ o flush/`wetpaint.rs` é pegada do F2e. |
| B17.6 `mask.rs:381` | **feito antes** | — | `223ab9393b` (B40.A4) | a proteção já não tem `fork_canvas(.., None)`: `mask_gate.rs::regiao_do_fork` dá a caixa do lote ao fork. Restam os `None` da lista do F1e (sculpt, fill ×3, inpaint ×2, region ×2, composite_*: pegadas alheias ou condicionais). |
| B17.5 | **decidido, cura aberta** | undo Digital 3,33 ms (1 cópia) · Impasto 10,48 ms (4 cópias), todas em `undo_delta.rs:334` (`target/n10/f1/s8.limpo.txt`) | — | o topo já é `StoredPlane::Tiles` (o delta por tiles está feito); a cópia que sobra é o `par_clone(cursor)` do `materialize` de `Tiles`/`Window` — 1 plano inteiro por plano mudado, porque a tela tem 2 donos (cursor + documento). Cura: no Ctrl+Z largar o `Arc` do cursor ANTES de materializar e `Arc::make_mut` no do documento (1 dono ⇒ blit dos tiles no lugar, 0 cópias). Mexe na ordem de `undo_apply`/`set_cursor` — próximo agente, com `the_journal_delta_describes_the_same_two_endpoints_as_two_snapshots` e a s8. |
| B17.7 · B17.8 · B42.4 · B42.5 | não iniciados nesta janela | — | — | prazo; o estado do F1e mantém-se (N10_memoria). |

Comandos (`--ignored`, para a fila da janela principal): s6 A×B intercalado A B B A com `PH2D_MASCARA_CHEIA=0|1` sobre
`sonda_as_fases_do_ctrl_z_wet_a_4k` (features `ph2d-wet-paint/sonda-fases`, `PH2D_SONDA_RODADAS=7`); a troca com as partes
finas = `sonda_primeiro_traco` com `PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PAPEL=1 PH2D_SONDA_NOVA=1 PH2D_SONDA_TROCA=1
PH2D_SONDA_PARTES=1` (ambos `--profile smoke -- --ignored --nocapture --test-threads=1`, por `scripts/ph2d-run.sh`).

## F1h (14/11) — B17.5: o Ctrl+Z escreve a tela no lugar

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B17.5 (a tela) | **feita ao bit; o número a 4K é `--ignored`** | undo Digital 3,33 ms / 1 cópia · Impasto 10,48 ms / 4 cópias (`undo_delta.rs:334`) → esperado Digital 0 cópias, Impasto 3 (o relevo) | este commit | `UndoController::undo_na_tela`/`redo_na_tela` (chamados em `tool/runtime.rs`): o tool entrega o `Arc` da tela (e o do `live`), a história larga o do cursor e o do `held` ⇒ um dono ⇒ `StoredPlane::side_tomando` (`undo_delta_tiles.rs`) faz `Arc::get_mut` + blit dos tiles/janela no lugar. `get_mut`, não `make_mut`: um `Weak` vivo (testemunha da elisão) ⇒ cópia. Tela vigiada pela sessão molhada pelo ponteiro (`WriteState::vigia`) ⇒ cópia. A tela vai por ÚLTIMO em `PlaneDeltas::side` e a recusa de tamanho é perguntada antes: a recusa devolve a tela intacta ao tool (`Err(tela)`). Gate `o_side_tomando_escreve_no_lugar_com_um_dono_e_da_o_bit_da_copia` (oráculo `side` verbatim; ponteiro igual com um dono; outro forte e `Weak` ⇒ cópia, base intacta; tamanho errado recusa); ablação `COPIA_SEMPRE` (thread-local de teste). Ponta a ponta: `um_ctrl_z_copia_so_os_planos_que_o_passo_mudou` (1536², Digital: 0 cópias do lado do delta no undo e no redo; CONTROLO no mesmo teste: com `COPIA_SEMPRE` a cópia volta e o contador vê-a = a mutação «sem a cura» fica vermelha). |
| B17.5 (o relevo, Impasto) | aberto | 3 cópias (heights/covers/mats, base = `live`) | — | a mesma cura por plano do mapa: o tool entrega `heights`/`covers`/`mats` e um `StoredMap::side_tomando`; a recusa devolve os mapas. |
| troca.commit | sem número novo | `target/n10/lote2/troca.txt` só tinha a compilação (08:38, load 12–14) | — | ler quando a sonda acabar. |

Comando da medição (fila): a s8 `what_a_diagonal_ctrl_z_copies_at_4k` (`--profile smoke -- --ignored --nocapture
--test-threads=1`); esperado «sitios do undo: []» em `impasto=false`.

## F1i (14/11) — B17.5: o relevo do Impasto no lugar; troca.commit e 3b medidos

| id | estado | antes → depois | sha | porquê |
|---|---|---|---|---|
| B17.5 (o relevo, Impasto) | **feita ao bit; o número a 4K é `--ignored` (s8)** | 1536², 2 traços Impasto, Ctrl+Z e redo: 3 cópias do lado do delta (`undo_delta.rs` 334 ×2 = relevo em tiles, 321 = tela) → **0** | este commit | `PlaneDeltas::side` toma `Base { tela, relevo }` (`undo_planes.rs`): toda recusa (tamanho da tela, chave/tamanho de cada `Both` do relevo por `StoredMap::cabe`) é perguntada ANTES de escrever; depois `StoredMap::side_tomando` (`undo_delta_tiles.rs`) faz `StoredPlane::side_tomando` por camada. O tool entrega os três mapas (`tool/runtime.rs`: `tira_o_relevo_para_o_undo`, a recusa devolve-os por `devolve_o_relevo_do_undo`); `undo_anda.rs` só os toma se forem os MESMOS `Arc` do `live` (então tira os do `live`), e o cursor e o `held` largam o relevo e as testemunhas `Weak` dele (`larga_o_relevo`). Gate `um_ctrl_z_copia_so_os_planos_que_o_passo_mudou` agora com `impasto=true` a 0 cópias (2 traços: o 1.º cria a chave, o 2.º faz o `Both`); CONTROLO no mesmo teste: a ablação `COPIA_SEMPRE` dá ≥ 3 cópias no Impasto e ≥ 1 no Digital. Mutação «o tool não entrega o relevo» (`relevo_vai = false`) ⇒ vermelho «impasto=true: um plano copiou — `undo_delta.rs:334` ×2»; restaurada ⇒ verde. 135 testes (`undo_`, `journal_delta`, os dois gates) verdes. |
| s8 (a sonda) | controlo trocado | a s8 de 14/11 (`target/n10/f1i/s8.txt`) reprovava no controlo «o undo nao copiou plano nenhum» = a cura da tela (`bd78b026b1`) a funcionar a 4K | este commit | `what_a_diagonal_ctrl_z_copies_at_4k`: cura × ablação `COPIA_SEMPRE` no MESMO processo, intercaladas (10 rodadas, ordem alternada), o mínimo de cada; controlo = a ablação copia; veredito = 0 sítios do lado do delta na cura, Digital e Impasto. Antes (14/11 07:26): undo Digital 3,33 ms/1 cópia · Impasto 10,48 ms/4 cópias → **s8b (14/11 09:18, load 6–20, `target/n10/f1i/s8b.txt`): cura Digital undo 0,12 / redo 0,10 ms, 0 cópias · Impasto 0,74 / 0,70 ms, 0 cópias; ablação `COPIA_SEMPRE` no mesmo processo 2,89/2,32 (1 cópia) e 10,41/10,30 ms (4 cópias)** ⇒ B17.5 3,33 → 0,12 ms (Digital), 10,48 → 0,74 ms (Impasto). |
| troca.commit | **resolvido pelas curas desta rodada** | 1.ª troca Wet Wash 90,6 → **12,1 ms** · Wet Round 8,3 · Watercolor Basic 2,6; todas as partes `commit.*`/`split.*` 0,01 ms (`sonda_primeiro_traco` TROCA/PARTES, 4K papel, `target/n10/lote2/troca.txt`) | — | medido pela janela principal. |
| Cura 3b (`PH2D_MASCARA_CHEIA`) | **feito ao bit, ganho não medível com esta carga** | `troca_os_planos` Wet Flat 15,2/19,2 × 15,8/16,2 ms · Dabber 32,6/28,6 × 29,0/31,0 ms (0/1, load ~10–20; `target/n10/lote2/mascara*.txt`) | — | medido pela janela principal. |
| B17.7 · B17.8 · B42.4 · B42.5 | não iniciados nesta janela (F1i, prazo) | — | — | o estado do F1e mantém-se (N10_memoria). |

Comando da s8 (fila da janela; imprime `impasto=false` e `impasto=true`, cura × ablação): a s8
`what_a_diagonal_ctrl_z_copies_at_4k` com `--profile smoke -- --ignored --nocapture --test-threads=1`.
