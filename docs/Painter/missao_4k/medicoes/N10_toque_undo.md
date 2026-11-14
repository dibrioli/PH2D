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
