# J13b P6 — o tique pede e aplica as imagens do compositor

Passo P6 do [plano](J13b_plano_otica_fora.md) §2 (protocolo 1, «Validade», «Casos») e §4, sobre o HEAD `b1a3bcbb95`. O
interruptor `PH2D_WET_OTICA_FORA` continua DESLIGADO por omissão; o P7 mede e decide.

## O que mudou

- **`wetpaint/otica_no_tique.rs`** (novo):
  - `OticaDoTique` (:17) vive na sessão: `no_ar` (a época do pedido por responder), `epoca`, `uniao_local`, `divida`.
    Em `cfg(test)` tem ainda os contadores `aplicadas` e `tocadas`.
  - Métodos: `compos_local` (:42), `toca_tudo` (:47, para a pausa da autoria), `pendente` (:52) e `deve` (:56, junta à dívida).
  - `wet_otica_fora` (:63): o compositor está de pé e `ligada()`. Se o interruptor for desligado a meio, continua até não
    haver nada pendente.
  - `wet_recebe_imagens` (:70) faz `try_recv` até esvaziar.
    - Resposta `Nada` da época certa: põe `seen_steps ← passos` e `composta ← marca`.
    - `Imagem` de outra época: a caixa vai para a dívida.
  - `wet_aplica_imagem` (:104) descarta a imagem e põe a caixa na `divida` em dois casos: a caixa toca a `uniao_local`, ou
    `show_wet` está ligado e a imagem não trouxe véu.
    - Senão faz `Imagem::aplica` nos `planos`, `seen_steps ← passos`, e depois `wetpaint_antes_da_otica` (restaurando o
      `composite_pendente`) e `wetpaint_depois_da_otica(…, VeuDe::DaImagem)`, que escreve `oleo_na_tela` e `composta`.
  - `wet_pede_imagem(done)` (:170) só publica com nada no ar, `base.len() == w·h·4` e (`fresh` ou dívida).
    - Ao publicar: `uniao_local ← None`, `divida ← None` e a época avança.
    - O `done` vem do tique: `sim_steps()` não se chama duas vezes, porque grava na fita da macro.
- **`ciclo.rs`** `wetpaint_tick`:
  - Pausa da autoria: `toca_tudo` (:160).
  - `fora` e a recepção, depois do `wet_compoe_o_pendente` e do `wet_varre_oleo` (:177–:179).
  - `mostra = fresh && !fora` (:193). Só `mostra || facts_moved || adiados` traz o motor para casa.
  - `seen_steps ← done` só sem `fora` (:218). Composição local só com `mostra` (:225).
  - O pedido sai antes do `hand_off_sim` (:236).
  - Interruptor desligado: `fora` é falso e o tique é o de hoje, linha a linha.
- **`composite.rs`**:
  - `wetpaint_depois_da_otica` recebe `VeuDe` (`Motor`, `Grade` só em `cfg(test)`, `DaImagem`) em vez de `Option<&Grid>`.
  - Tudo o que não é `DaImagem` é composição LOCAL e chama `compos_local(caixa_cel)` (:214). Isto cobre o traço, o
    undo, a varredura, os adiados, o Fast dry e a porta de teste.
  - O véu (:516) resolve a fonte.
- **`veu.rs`**: `FilmeEMolha { film, wet, s, linha0 }` (:15) e `VeuDe` (:34). `pinta_o_veu` (:45) lê `i − linha0·s`; da grade
  inteira, `linha0 = 0` dá o mesmo índice de hoje.
- **`authored_actions.rs`** `wetpaint_end_session` (:197): com `otica.pendente()` traz o motor e recompõe a tela inteira em
  casa sem véu. A imagem no ar ou a dívida levaram sujo que a tela ainda não mostra.
- **`offthread_otica.rs`**:
  - Saiu o `expect(dead_code)`.
  - `ligada`/`forca_no_teste` passam a `pub(in …::wetpaint)` (:185, :190).
  - `WetSession::tem_compositor` (:435).
- **`offthread.rs`**: saiu o `cfg_attr(expect(dead_code))` do campo `otica`.
- **`session.rs`**: campo `otica` e o clone dele.
- **`otica_fora_porta.rs`**: passa `VeuDe::Grade`.
- **`wetpaint.rs`**: `mod otica_no_tique` e `mod otica_no_tique_tests`. O `rustfmt` reordenou alguns `mod` vizinhos.

Fim de sessão, troca de camada e undo:
- `session = None` derruba o worker e o compositor, e a imagem no ar morre.
- O undo (`mark_dirty_full` mais a composição local) põe a tela inteira na `uniao_local`, e a imagem no ar é descartada.
- A pausa da autoria faz o mesmo por `toca_tudo`. `offthread_tests` `:431` (o Fast dry com o motor fora) e `:564` (a
  pausa traz o motor) continuam verdes.

## Portões (`otica_no_tique_tests.rs`, thread real, interruptor por `forca_no_teste`)

O interruptor liga-se por `forca_no_teste`, que vale só para o fio do teste, e não por `set_otica_fora`, que é global: com
o global, os outros portões que correm em paralelo no mesmo processo também ficavam com o compositor.

Cena: 192², Dabber com Real Oil, depois Wet Drip, com quadros de 16 ms. Assentar quer dizer abrir o traço do motor
(`pointer_down`, como no P5), deixar o motor ir para o worker e correr tiques até não haver nada pendente e
`sim_steps == seen_steps`.

| portão | o que prova | resultado |
|---|---|---|
| `a_agua_pelo_compositor_e_a_tela_de_casa_ao_byte` | Corre a água e assenta. A tela fica igual a `mark_dirty_full` mais a composição de casa, ao byte. Variantes: controlo sem compositor; com compositor; com compositor + Smooth 0.6 + véu. Exige ≥ 5 imagens aplicadas e `oleo_na_tela`. | verde |
| `o_fio_da_tela_nao_paga_a_otica` | Em 40 quadros com a água a correr, o `composite_contagem` do fio do teste conta 0 chamadas e chegam > 5 imagens. Controlo sem compositor: > 5 chamadas. | verde |
| `a_imagem_velha_nao_deixa_pixel_velho_debaixo_do_dab` | Com uma imagem servida no canal, um dab de Dabber é composto em casa e a água pára. A imagem chega e toca a `uniao_local`. Depois de assentar, a tela fica igual à de casa ao byte. | verde |
| `a_divida_leva_a_caixa_da_imagem_descartada` | Mesmo caso, sem corrida: a água está parada; o `fast_dry_now` do motor suja a tela inteira sem compor; a imagem fica no canal; um dab pequeno é composto em casa; a imagem chega depois. No fim, a tela é igual à de casa ao byte. | verde |

Mutações (aplicadas e restauradas; `cmp` com a cópia):

| mutação | `a_imagem_velha…` | `a_divida_leva…` |
|---|---|---|
| M1: não descartar por `uniao_local` (`if tocou \|\| …` → `if show_wet && …`) | vermelho | vermelho (a tela difere da de casa) |
| M2: descartar sem juntar à `divida` | verde (a caixa do dab cobre a da imagem, porque a água muda a mesma área a cada passo) | vermelho (a tela difere da de casa) |

## Antes / depois (binários copiados, `--test-threads=8`)

Com o interruptor desligado, as 151 linhas `test … ok/FAILED/ignored` são iguais teste a teste antes e depois (`diff`
das listas ordenadas: só mudam as linhas de resumo, pelo número de testes filtrados e pelo tempo).

| filtro | antes | depois |
|---|---|---|
| `por_quadro_tests` | 14 ok | 14 ok |
| `offthread` | 14 ok · 3 ign | 14 ok · 3 ign |
| `showwet` | 2 ok | 2 ok |
| `smooth_edges` | 15 ok · 5 ign | 15 ok · 5 ign |
| `oleo_cache_tests` | 2 ok · 1 ign | 2 ok · 1 ign |
| `a_otica_pelo_espelho` | 1 ok | 1 ok |
| `undo_tinta_viva` | 9 ok · 3 ign | 9 ok · 3 ign |
| `macro` | 94 ok · 1 ✗ (`o_splash_tocado…`, conhecido) · 19 ign | o mesmo |
| `leak` · `ritmo` · `otica::tests` (P5) | — | 4 · 1 · 3 ok |

Com `PH2D_WET_OTICA_FORA=1` no ambiente: `offthread` 14 ok · 3 ign, `por_quadro` 16 ok · 2 ign, `showwet` 2 ok — as mesmas
contagens de antes (o binário antes do P6 dava o mesmo). O que muda é o caminho que os testes percorrem:
- `offthread_tests` sobe o worker ao relógio, por isso o compositor existe e o tique mostra a água por imagens, sem trazer
  o motor para casa;
- `the_tick_leaves_the_engine_with_the_worker` e `the_frame_does_not_run_a_sim_stage` ficam mais fortes;
- `the_authoring_pause…` (`toca_tudo`) e `a_canvas_action…` continuam verdes;
- `por_quadro_tests` usa o relógio fixo (`wet_step_sync`, sem compositor), por isso fica no caminho de hoje;
- `showwet_tests` não dá tiques: recompõe em casa.

## Aberto

- **P7**:
  - medir `passo ms` com o compositor ocupado e as imagens/s;
  - medir o custo de `wetpaint_antes_da_otica` por imagem (`wet_splat_gates` corre no fio da tela a cada imagem aplicada);
  - imprimir `take_otica()` no relatório do shell;
  - decidir o interruptor por omissão.
- **O descarte é conservador.** Um pedido publicado antes de um traço só é servido depois dele (o motor fica em casa com o
  traço aberto), e a imagem, embora mais nova, toca a `uniao_local` do traço e é recomposta pela dívida: uma imagem a
  mais. A época por cópia (o worker carimbar a ordem da cópia contra as composições locais) tirava essa imagem; fica
  para medir no P7.
- A varredura do painel (`oleo_no_quadro.rs`) continua local.
