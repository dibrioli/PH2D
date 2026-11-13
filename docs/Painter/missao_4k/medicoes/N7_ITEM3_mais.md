# N.º 7 item 3 — o «+» a 4K sem diff (autorizado pelo dono 13/11: «sim»)

## Antes (handoff n.º 6 §1, não re-medido aqui)

«+» a 4K: 55–65 ms = `add_raster_layer` 51 ms. Dentro dele, `commit_structural_edit` leva 15–30 ms, dos quais
`UndoEntry::split` leva 14 ms a comparar a tela velha com a nova zerada. Sobre a base opaca a entrada saía
`Whole{A, Z}`: segurava a tela nova `Z`, e por isso o 1.º traço na camada nova copiava a tela inteira (33 MB).

## O que mudou

- `StoredPlane::Nova { before }` (`undo_delta.rs`): guarda a tela que saiu (`A`) inteira, sem diff. O lado
  `after` é todo neutro e não se segura. O refazer recria-o (`vec![0; len]`, só no refazer do «+»).
  `reach` = `Whole` (repinta tudo, como antes), `copiado_bytes` = 0, `heap_bytes` = `len`.
- `UndoController::tela_nova_neutra` é um sinal de uso único. Acende-o `add_raster_layer_com` quando a camada
  nasce transparente (`rgba = None`); `add_raster_layer_with_pixels` não o acende. `record_structural_hinted`
  apaga-o.
- `PlaneDeltas::tira_a_tela_nova` age antes do `split`: dá aos dois endpoints o MESMO plano vazio (o `split`
  lê `Unchanged` sem varrer). Depois do `split`, `poe_a_tela_nova` grava o `Nova`. Recusa (e cai na rota
  antiga) com tela elidida, comprimentos diferentes ou o mesmo `Arc`. Em debug confirma que a tela nova é toda
  zero.
- A chave `MAIS_SEM_DIFF` (`undo_planes.rs`, por fio, ligada por omissão): desligada, volta a rota antiga. A
  sonda ganhou `PH2D_SONDA_MAIS=ab`.

## A contabilidade: quantos passos de desfazer se perdem a 4K

Plano RGBA 4K = 3840×2160×4 = **33,18 MB**. A entrada do «+» pesa agora `Nova` (1 plano) + `OnlyAfter` da
imagem da camada que saiu (1 plano) = **66,4 MB**, seja qual for a camada de baixo.

| «+» sobre… | rota antiga | nova | diferença |
|---|---|---|---|
| camada opaca (a base da imagem aberta) | `Whole` × 2 + imagem = 99,5 MB | 66,4 MB | **−33,2 MB** (lembra MAIS) |
| camada rala (poucos traços) | `Patch` da janela × 2 + imagem ≈ 33,2 MB + 2·janela | 66,4 MB | até **+33,2 MB** |

No pior caso (um «+» sobre camada rala) perde-se **≈ 1,5 traço** (22,45 MB por traço de lado a lado a 4K) por cada
«+» desses que esteja no histórico, tanto no piso de 509 MiB (23 traços) como na máquina do dono (3 943 MiB,
176 traços). O 1.º «+» sobre a base opaca ganha o mesmo tanto. O tecto conta o `before` pelo tamanho inteiro,
embora seja o mesmo `Arc` da imagem viva (conservador, como o `Whole`).

## Gates (`tool/paint/mais_sem_diff_tests.rs`, 520×390)

- `o_mais_sem_diff_da_o_bit_no_desfazer_e_no_refazer_wb`: base opaca, 2 traços, «+», 2 traços, «+» sobre a
  rala, 1 traço. Desfaz tudo e refaz tudo. A cada passo compara ao bit com a rota antiga (a chave desligada)
  no mesmo processo, e com os estados gravados. Assere o «+» = 2 planos (antigo 3 sobre opaca, menos sobre a
  rala). No pen-down do 1.º traço da camada nova, ninguém segura a tela além da viva e do cursor: chave = 0
  donos, controlo antigo = 1 (a entrada).
- `…_wet`: o mesmo no Wet Paint. Duas ferramentas Wet NÃO dão o mesmo bit entre si: medido com a chave igual
  nos dois lados, diverge no «+» (o motor corre fora do fio). Por isso cada rota confere-se contra o PRÓPRIO
  histórico gravado; os donos e os bytes asserem-se como no WB.
- `o_tecto_conta_o_mais_pelo_tamanho_real`: grava o peso de cada passo nas duas rotas e põe um tecto que cabe
  tudo menos o 1.º «+» antigo. A profundidade retida é igual à simulação do `cap` nas duas rotas (antiga 8,
  nova 6). Controlo: a nova < a antiga.
- Mutações: (M1) o refazer do `Nova` devolve a tela velha → 2 vermelhos; (M2) `tira_a_tela_nova` sempre `None`
  → 3 vermelhos.
- Vizinhos por nome, verdes: `camadas_integridade`, `duplicate`, `o_undo_e_o_redo_dao_o_bit`,
  `watercolor_vidro_journal_tests` (inclui `a_camada_nova_e_o_vidro_elidido_dao_o_bit` e
  `quem_segura_o_vidro_no_segundo_pen_down`), `every_plane_of_a_snapshot`, mais os filtros `undo`, `layers`,
  `camada`. `cargo check --profile smoke` compila.

## Próximo passo

1. O coordenador corre `N7_ITEM3_sonda.sh` pela fila pesada (A×B intercalado, WB e Wet Wash, 4K). Esperado:
   o «+» perde os ~14 ms do split, e o 1.º traço da camada nova perde a cópia da tela (≈ 33 MB · ~6 ms).
2. O resto do «+» fica por curar: `flush_active_to_images` 6,5 ms e `a_tinta_que_adormeceu_muda` 7,5 ms.
3. O Wet não é bit-determinístico entre duas ferramentas no «+»: isto existia antes desta mudança. Fica
   registado para quem fizer um oráculo cruzado do Wet.
4. ⚠️ Vizinho vermelho que NÃO é deste item: `undo_confine_tests::o_ctrl_z_de_um_traco_no_papel_e_confinado_em_cada_meio`
   (sem «+»). O controlo dele espera `v=WHOLE` no 1.º traço do Watercolor; com o `Aparece` do ADR-0204 (item 2) o
   vidro que aparece guarda só a janela (`v=win`). Ou seja, o controlo ficou velho com o item 2: a cura é atualizar o
   controlo para `v=win`. Fica para o item 2 / coordenador.

## Continuação (13/11): o controlo do Ctrl+Z confinado, o «+» do Wet, o pen-down

### 1. `o_ctrl_z_de_um_traco_no_papel_e_confinado_em_cada_meio`: era só a expectativa (`b8be38f548`, `e024c322ba`)

A lei do teste (o Ctrl+Z do 1.º traço é confinado nos 4 meios) é a que o ADR-0204 passou a cumprir. O que estava
velho era o controlo do Watercolor, que pedia `v=WHOLE`. Agora pede `v=<janela>` com a chave `VIDRO_PELO_JOURNAL`
ligada. No mesmo processo, com a chave desligada (a rota antiga), continua a pedir `v=WHOLE`, e esse controlo ficou
dentro do teste. Verdes por nome: `confin` 27 e `undo` 182 (29 ignorados), sem outros vermelhos novos.

### 2. O «+» do Wet Wash a 4K (sonda r1 do coordenador: 36–44 ms, 214–838 MB novos)

**As faltas da sonda r1 são do PROCESSO inteiro.** `faltas()` lê `/proc/self/stat`, que conta também o fio do motor
Wet e o fio que `wet_guarda_adormecida` lança (`encolhe_num_fio`, a compressão da sessão adormecida quando ela
retém ≥ 64 MB). As centenas de MB variam de 214 a 838 de uma corrida para a outra, e isso cabe num fio em
segundo plano, não num custo fixo do fio principal. Isto é uma hipótese, NÃO uma medição: confirma-se com a sonda
abaixo.

- **Instrumentação nova** `crate::sonda_partes` (só em `cfg(test)`; fora dele `parte()` não faz nada): ms e MB
  **do fio principal** (`/proc/thread-self/stat`) por parte do «+». As partes são `mais.snapshot_model`,
  `mais.flush` (com `flush.guard`, `flush.pousa_adiados`, `flush.ultima_imagem`, `flush.bring_home`,
  `flush.worker_none`, `flush.guarda`), `mais.tela_zero`, `mais.replace_canvas`, `mais.commit`, `mais.troca`,
  `mais.adormeceu_muda`, `select_layer`, e `tique`/`preview` × 3. A diferença entre o total da linha `«+»` e a
  soma das partes é o que os outros fios tocaram.
- **Cura do `a_tinta_que_adormeceu_muda` (7,5 ms no handoff):** ele é a varredura em série do alfa da camada
  inteira (`janela_da_tinta`, 33 MB). Passou a correr em faixas de 64 linhas em paralelo (rayon) com redução da
  caixa. Dá a mesma janela por construção. A chave em execução é `JANELA_DA_TINTA_PARALELA`
  (`undo_delta_confine.rs`, por fio, ligada por omissão). Desligada, volta a busca em série.
  - Gate `a_janela_da_tinta_paralela_e_a_da_serie`: tinta na 1.ª e na última linha, a cavalo da fronteira de
    faixa (63/64), um texel só, colunas extremas, plano vazio, RGB sem alfa. Controlo = a série. Mutações: (M1)
    a linha sem o deslocamento da faixa → vermelho; (M2) a coluna final sem `+ 1` → vermelho.
  - Vizinhos verdes por nome: `adormec` 29, `confin` 28, `troca` 31, `mais_sem_diff` 3, `janela` 17. `cargo check
    --profile smoke --tests` compila. Curado também o aviso `mut` em `mais_sem_diff_tests.rs:256`.
- **`flush_active_to_images` (6,5 ms): NÃO curado, falta a medição.** O flush propriamente dito é um `Arc::clone`.
  O custo está no `wet_adormece`: `wetpaint_pousa_adiados` + `wet_ultima_imagem` (`bring_home` + composite da
  janela por mostrar) + `bring_home`. As duas curas pedidas («só a caixa suja», «saltar se nada mudou») já existem
  aí: `wet_ultima_imagem` sai cedo sem imagem pendente e, sem véu, compõe só o sujo por mostrar
  (`o_sujo_por_mostrar_volta_ao_motor`). O que sobra é a espera pelo motor, a mesma família do item 5. Decide-se
  com as partes `flush.*`.
- **A sonda, para o coordenador (o guarda recusa `--ignored` neste subagente):**
  `bash scripts/fila-pesada.sh pôr 'bash docs/Painter/missao_4k/medicoes/N7_ITEM3_sonda_wet_partes.sh'`. Wet Wash
  4K, `PH2D_SONDA_JANELA=ab` × 4 blocos rodados e as partes. Para ler: (a) `mais.adormeceu_muda` com
  `janela_paralela=true` × `false`; (b) a soma das partes × o total (quanto é de outro fio); (c) qual `flush.*` leva
  os 6,5 ms. A `N7_ITEM3_sonda.sh` não mudou e imprime as partes também (já leva `PH2D_SONDA_PARTES=1`).

### 3. Item 5: a espera do pen-down pelo motor (3,8 ms + `abre_o_passo` 2 ms): NÃO implementado

- **Onde espera:** `WetSession::bring_home` (`wetpaint/offthread.rs:224`) faz `want.store(true)` e depois
  `back.recv()`, bloqueando até o worker acabar o estágio em curso (≤ um estágio). Com o motor em casa
  (`EngineSlot::Here`) já sai na 1.ª linha: «não esperar com o motor ocioso» **já é a lei**. Os 3,8 ms são o
  estágio que o worker estava a correr quando a caneta desceu.
- **Sobrepor `abre_o_passo` com a espera:** exige que nada do que o passo de undo fotografa (a tela, o journal,
  o `before` da sessão molhada do `undo_vivo`) esteja a ser escrito pelo worker durante a abertura. A tela só se
  escreve no fio principal (o composite), mas o `before` do passo Wet lê o estado da sessão que o worker tem na
  mão. Abrir antes do `recv` fotografaria um estado a meio do estágio. Não há prova de que seja ao bit, e o
  handoff já o julgou arriscado (§1: «A espera só some adiando o pen-down inteiro»).
- **Falta medir:** a decomposição 3,8 + 2 é do handoff, não foi re-medida (sem sonda neste subagente). Quem
  retomar: ponha `crate::sonda_partes::parte` em volta do `bring_home` e do `abre_o_passo` no pen-down Wet e use
  a mesma sonda com `PH2D_SONDA_TOQUES`.

## Continuação 2 (13/11): o «+» do Wet por dentro (sonda `N7_ITEM3_sonda_wet_partes_r1.log`, load 2,7–6)

Medição própria: um teste temporário, não comitado, com a cena da sonda (Wet Wash 4K, 2 traços, «+», 2 quadros).
As variantes eram intercaladas no MESMO processo, 8 blocos, perfil `smoke`, e o resumo é o mínimo.

- **`tela_zero`: curado** (chave `TELA_NOVA_PELO_MMAP`, `tool/layers/mutate.rs`). Já era `vec![0u8; n]`, um calloc.
  O defeito é do glibc. 4K são 31,6 MiB, abaixo do tecto de 32 MiB do limiar dinâmico do mmap. Depois de se
  libertar um plano destes, o limiar sobe e o calloc seguinte sai do heap com `memset`: 18–29 ms e 33,2 MB tocados.
  `tela_transparente` pede `PISO_DO_MMAP` (32 MiB + 4 KiB) e faz `truncate(n)`, então vai sempre ao mmap. A folga
  é só endereço, e os clones copiam o `len`. A partir de 1 MiB. Sem `unsafe` (o crate o proíbe). A×B: na rota nova
  a parte nunca passou de 0,3 ms; na antiga, 2,5–29 ms. Nada lê a tela nova inteira a seguir: o commit
  (`tela_nova_neutra`) não a varre, o adormeceu lê a camada que saiu, e o preview compõe só a caixa suja.
  Gate `a_tela_nova_de_4k_nasce_sem_tocar_paginas`: o alocador é aquecido com 3 planos, o gate pede menos de 1 MB
  tocado, a capacidade e os mesmos bytes da chave desligada. Mutação: `vec![0; n]` deu vermelho.
- **`adormeceu_muda`: o custo é CONTENÇÃO, não a varredura.** A mesma `janela_da_tinta`, repetida 30 ms depois,
  custa 2–3 ms. Logo após o «+», custa 12–14 ms, porque um fio de fundo nascido no flush disputa a memória. O
  provável é `encolhe_num_fio` do `wet_guarda_adormecida`; isto não está provado. A paralela (rayon) era a PIOR,
  com 14–46 ms contra 9–12 ms da série, porque o pool está ocupado. Por isso SAIU, com o gate dela. No lugar dela
  entrou `janela_do_alfa_por_palavras`: OR de `u64` com a máscara do alfa, e nas linhas da faixa só se procura onde
  a caixa pode alargar. Chave `JANELA_DA_TINTA_POR_PALAVRAS`. Mínimo sob contenção: 5,7 contra 12,2 ms; em calma,
  2,0 contra 3,0 ms. Gate `a_janela_da_tinta_por_palavras_e_a_da_serie` com largura par e ímpar, a última coluna
  fora das palavras e os dois píxeis de uma palavra. Mutação: o 1.º píxel do par sempre, deu vermelho.
  **Próximo alvo:** adiar o fio de compressão para depois do «+», ou fazer a varredura antes de ele nascer.
- **`preview` (5,5–31 ms): NÃO curado, só medido.** O braço é `PartialComposite` sobre a caixa suja, que é a janela
  da tinta. A cena da sonda tem 2 traços, cerca de 400 × 360 px. O custo varia de 4 a 31 ms com a mesma caixa, a
  mesma família de contenção. O cache do fundo cobre: a pilha não é recomposta inteira.
- ⚠️ **Defeito visto, não tocado:** `a_tinta_que_adormeceu_muda` → `mark_dirty` sobe a versão da camada ACTIVA, que
  depois do «+» já é a nova. Com isso o `camada_nova_limpa` (a fatia sem envio da pista GPU) nunca casa no «+» do
  Wet. Pode ser por acaso o certo, porque a pista GPU tem de ver a mudança da janela da tinta. Não mexer sem um gate
  da pista GPU.
- Vizinhos verdes por nome: `adormec` 29, `confin` 28, `janela` 17, `mais_sem_diff` 3. A sonda tem o eixo novo
  `PH2D_SONDA_TELA_ZERO=ab`; `PH2D_SONDA_JANELA` passa a ser a chave por palavras.

## Continuação 3 (13/11): o fio de fundo do «+» é a compressão da adormecida (PROVADO e curado)

Medição própria: um teste temporário (não comitado) que chama a sonda. Wet Wash 4K, `PH2D_SONDA_ENCOLHE=ab` × 4
blocos rodados no MESMO processo, perfil `smoke`. A sonda imprime agora a hora de início e de fim do fio da compressão
relativa ao «+» (`ULTIMO_FIO`, só `cfg(test)`). Para o coordenador: `N7_ITEM3d_sonda.sh` (Wet Wash e Watercolor Basic).

| chave `ENCOLHE_NO_SILENCIO` | início do fio | «+» total (ms) | `adormeceu_muda` | 1.º `preview` |
|---|---|---|---|---|
| desligada (antes) | 1,7–12,5 ms após o «+» | 5,6 · 33,6 · 38,1 · 39,0 | 1,6–14,9 | 1,7–29,7 |
| ligada | 301,9–305,3 ms | 6,0 · 6,0 · 8,9 · 9,8 | 1,58–1,69 | 1,97–5,9 |

O fio comprime ~460 ms (na rota antiga, de 2 a 470 ms após o «+»). `encolhe` usa `into_par_iter`, que rouba o pool
rayon, e varre centenas de MB. Na rota antiga as faltas de página do processo no «+» são de 241–484 MB, na nova de
11–53 MB. Isto confirma a hipótese da Continuação 2: as faltas eram do fio, e o «+» do fio principal é de ~6 ms.

- **Cura** (`wetpaint/adormecida/silencio.rs`): o fio nasce igual, mas só comprime depois de `SILENCIO_MS = 300` sem
  evento de caneta, contados a partir do adormecer. `marca_gesto()` é chamado em `on_canvas_pointer`, em todo evento
  que não é hover. A RAM fica com tecto: `ESPERA_MAX_MS = 3000`, ou seja, sob traço contínuo a sessão comprime na
  mesma no máximo 3 s depois. A conta do tecto das adormecidas (`bytes / 7`) não mudou, e o porquê dos 64 MB fica
  igual. Acordar cancela a espera: o canal acorda o fio, e quem baixa `espera` primeiro decide. O `pronta()` trata a
  tinta por começar como pronta, para o acordar não esperar 300 ms. O resultado é ao bit, porque a compressão é a
  mesma ou não acontece. Chave em execução `ENCOLHE_NO_SILENCIO` (por fio, ligada).
- **Gates** (`adormecida/silencio_tests.rs`):
  - `a_compressao_espera_o_silencio_e_acontece`: começa ≥ 290 ms depois, a sessão fica comprimida (é o controlo de que
    a compressão continua a acontecer), e a Borracha depois de acordar dá o bit de quem nunca saiu.
  - `sem_a_chave_a_compressao_comeca_logo`: controlo da rota antiga.
  - `acordar_na_espera_cancela_a_compressao`: no relógio da parede acorda logo, o fio nunca comprime, e dá o mesmo bit.
- **Mutações:**
  - (M1) sem espera → vermelho.
  - (M3) `pronta` sem «por começar» → vermelho.
  - (M2) o `acorda` sem `cancela()` → verde, uma mutação EQUIVALENTE nesta cena. O `acorda` consome a tinta, o
    `Sender` cai, e o fio sai por `Disconnected`. O `cancela()` explícito só decide quando uma cópia do estado (a macro)
    segura outro `Sender`.
- Vizinhos verdes por nome: `silencio` 3 e `adormec` 32 (2 ignorados).
- ⚠️ **Aberto:** um traço que comece entre 300 ms e ~760 ms depois do «+» ainda encontra o fio. O silêncio mede-se
  desde o último evento de caneta, então o fio só começa com a caneta parada 300 ms. Mas se começou e a caneta desce
  depois, ele corre até ao fim, ~460 ms. A prioridade baixa ou um pool próprio não foram feitos: `nice` por fio pede
  `libc`/`unsafe`, que o crate proíbe.

### O 1.º traço na camada nova (a mesma corrida, ev0..2 e o máximo por evento)

- **Wet Wash 4K:** 1.º traço na nova, máximo por evento 9,8–13,0 ms (ev2, o 1.º `Move` com tinta), contra a base
  9,4–21,7 ms. Os «100–160 ms» da nota são o GESTO inteiro: 32 eventos × ~2–3 ms + ev0/ev2. Os dois traços na nova
  levam ~150 ms de relógio (977 → 1128 ms). Não é um engasgo e não tem parte gorda nova. Nas duas rotas é igual.
- **Watercolor Basic 4K:** o fio não nasce (a sessão retém < 64 MB). O 1.º traço na nova tem o ev0 (pen-down) a
  40,9–47,2 ms, contra 25–36 ms na base. O 2.º traço na nova fica a 11,6 ms. A parte gorda é o `ponteiro` do pen-down
  (33–41 ms, 100–240 MB novos), não os tiques (3–5 ms). Na nova é ~10 ms pior que na base. Não foi decomposto por
  dentro neste subagente (sem prazo). Próximo: `sonda_partes::parte` dentro do pen-down do Watercolor (o vidro, o
  `abre_o_passo`, a sessão).

### O `mark_dirty` da camada nova no «+» (item 3 da tarefa)

Não foi investigado. Não há um gate da pista GPU que fixe a versão da camada nova no «+», e por isso, como na nota,
fica documentado e não mexido.

## Continuação 4 (13/11): o 1.º pen-down do Watercolor Basic por dentro (o fundo aquece na troca, curado)

Medição própria: um teste temporário (não comitado) que chama a sonda. Watercolor Basic 4K, `PH2D_SONDA_PARTES_DOWN`
(marcas `down.*` em `stroke_lifecycle.rs`, no-op fora de teste), `PH2D_SONDA_AQUECE=ab` × 2 blocos rodados no MESMO
processo, perfil `smoke`, load 3–5. Para o coordenador: `N7_ITEM3e_sonda.sh` (6 blocos, WB e Wet Wash como vizinho).

**A decomposição (antes da cura):**

| parte do pen-down | base, 1.º traço | camada nova, 1.º traço |
|---|---|---|
| `freeze_ground` (o fundo de baixo) | 2,2–6,9 ms | 10,8–12,4 ms (0 MB: composição, não alocação) |
| `apply_watercolor` | 5,1–12,6 | 10,5–19,6 (o chão preto do vidro) |
| `stamp` | 7,3–13,5 | 5,9–8,9 |
| `zera_sessao` (limpezas antes do `freeze`) | < 0,3 | 0,7–6,0 (às vezes 33 MB) |
| `begin_undo_step`, `snapshot`, `compoe_o_pendente` | < 0,3 cada | < 0,3 cada |

Os ~10 ms a mais da nova eram o `fundo_de_baixo` a falhar a cache, porque a pilha mudou no «+», e a compor 4K inteiro
(já em paralelo, em faixas). Somava-se o chão preto do vidro, chaveado pelo `Arc` desse fundo e recomposto no 1.º quadro.
`fork_par_raw`/`make_mut` e `abre_o_passo` NÃO aparecem neste pen-down (< 0,3 ms cada): o «aberto» do ADR-0204 não é
o custo do WB.

- **Cura** (`watercolor_vidro.rs::aquece_o_fundo_do_toque`, chave em execução `FUNDO_AQUECIDO_NA_TROCA`, por fio,
  ligada): no fim do «+» (`add_raster_layer_com`) e do `set_active_layer`, com a aguada armada (`brush.watercolor`,
  Paint, sem borracha) e nenhum traço aberto, compõe o fundo de baixo pela própria cache dele e o chão preto chaveado
  pelo mesmo `Arc`. O pen-down acerta as duas caches. É ao bit por construção: as mesmas funções com as mesmas chaves;
  o que mudar entre a troca e o toque só as invalida.

| chave | «+» | 1.º pen-down na nova (`ponteiro`) | máx por evento | `freeze_ground` | `apply_watercolor` |
|---|---|---|---|---|---|
| desligada | 0,2 ms | 32,7 · 33,6 ms | 37,8 · 38,8 | 10,8 · 11,0 | 10,5 · 14,9 |
| ligada | 16,5 · 22,5 ms | 12,0 · 17,6 ms | 17,4 · 23,2 | 0 · 1,7 | 5,0 · 5,0 |

O custo MUDA de sítio para o clique do «+» ou da linha (16–22 ms, num clique, não num traço). A nova fica igual à base
(15,5–16,8 ms). O 2.º traço não muda, porque já acertava a cache.

- **Gate** `o_fundo_do_toque_aquece_na_troca_e_e_o_mesmo_bit` (`watercolor_fundo_aquecido_tests.rs`): «+», clique na
  base (o chão puro) e clique de volta na nova. Com a chave, o 1.º toque usa o `Arc` aquecido e o chão preto dele. O
  controlo (sem a chave) compõe o seu. O fundo, a tela e o quadro dos dois caminhos são iguais ao bit.
- **Mutações:** (M1) sem aquecer no «+» → vermelho · (M2) sem guardar o chão preto → vermelho · (M3) sem aquecer no
  clique → vermelho.
- Vizinhos verdes por nome: `fundo` 3, `vidro` 32 (5 ignorados), `troca` 32 (5 ignorados).
- **Aberto:**
  - O `stamp` do 1.º traço (6–13 ms, 0–51 MB) é o próximo gordo, também na base.
  - O `zera_sessao` às vezes toca 33 MB. A origem, entre `compoe_o_pendente` e `freeze_ground`, não foi isolada.
  - Trocar de pincel para a aguada SEM trocar de camada não aquece: o 1.º toque compõe como antes (2–7 ms numa camada
    só, o `flat_fill`).
  - O «+» de 16–22 ms poderia ir a um fio de fundo, mas segurar lá os `Arc` das imagens faria o `make_mut` copiar. Não
    foi feito.

## Continuação 5 (13/11): o aquecimento do 3e sai do clique (o fundo compõe-se num fio)

O 3e tirou ~20 ms do 1.º toque, mas pô-los no clique do «+» e da linha (0,2 → 16–22 ms). Uma cura que só muda o
trabalho de gesto não acabou.

- **Cura** (`watercolor_aquece.rs`, chave em execução `FUNDO_AQUECIDO_EM_FUNDO`, por fio, ligada; `false` = a rota do
  clique do 3e, o oráculo). O clique tira um instantâneo do que o fundo de baixo lê: a pilha, os `Arc` das camadas
  NÃO activas (a activa vive em `canvas_rgba`, então o traço nela não copia o plano por causa do fio) e o papel, com
  a `ChaveDoFundo` do n.º 6. Um fio compõe o fundo e o chão preto com `composite_below` num **pool rayon próprio**, do
  tamanho do global. Nunca rouba o pool a um traço ou preview. Só lança com nenhum traço aberto e sai cancelável entre
  as duas fases (`AtomicBool`, sem `unsafe`, o molde de `adormecida/silencio.rs`). Larga o instantâneo antes de
  devolver.
- **O pen-down colhe** (`fundo_de_baixo` → `colhe_o_aquecimento`). Chave igual: espera o fio se ainda corre e guarda
  fundo e chão preto nas caches de sempre. Chave diferente (a pilha mudou entre o clique e o toque): cancela e compõe
  como hoje. Outro clique descarta o anterior e lança o seu. Se a cache já serve, não lança.

Medição: um teste temporário (não comitado), 4K, WB, perfil `smoke`, load 4–5, 8 blocos rodados no MESMO processo. A
pilha cresce 1 camada por amostra (2 → 33), por isso o síncrono sobe com ela.

| rota | «+» | 1.º pen-down 300 ms depois | 1.º pen-down colado ao clique (0 ms, caso-limite) |
|---|---|---|---|
| clique (3e) | 40 → 171 ms | 9,8–11,7 ms | 10,7–16,0 ms (+ o «+» de 47–154) |
| fio (3f) | **0,06–0,18 ms** | 9,7–19,5 ms | 45 → 204 ms (espera o fio ≈ o «+» síncrono + o toque) |

O clique fica ≤ 1 ms. Com a pausa humana o toque é o mesmo. Colado ao clique, o toque espera o resto da composição,
≈ o que o síncrono pagava no clique (não há ganho nem perda de monta, com o ruído do load). Para o coordenador:
`N7_ITEM3f_sonda.sh` (eixo novo da sonda `PH2D_SONDA_EM_FUNDO=ab`, 6 blocos, WB e Wet Wash).

- **Gate** `o_fundo_do_toque_compoe_se_num_fio_e_e_o_mesmo_bit` (`watercolor_fundo_em_fundo_tests.rs`). Três casos:
  acabado, a meio (o fio atrasa 150 ms e o pen-down espera) e descartado (a base muda entre o «+» e o toque). Cada um
  contra a rota do clique: fundo do toque, tela, chão preto congelado e quadro iguais ao bit, e contas colhidos ×
  descartados × «o fio corria». O controlo (a rota do clique) não lança fio. O gate do 3e fixa a rota do clique.
- **Mutações:** (M1) papel do fundo errado no fio → vermelho · (M2) colher sem conferir a chave → vermelho · (M3) não
  lançar → vermelho · (M4) chão preto `[0,0,1]` → vermelho, mas só depois de a foto do chão preto entrar no gate: os
  píxeis desta fixture não o leem.
- Vizinhos verdes por nome: `fundo` 4, `vidro` 32 (5 ignorados), `troca` 32 (5), `adormec` 32 (2), `mais_sem_diff` 3.
- **Aberto:**
  - Trocar de pincel para a aguada sem trocar de camada ainda não aquece. O mecanismo serve, falta chamá-lo na
    troca de pincel. Não houve tempo.
  - O «+» síncrono cresce com o número de camadas (40 → 171 ms de 2 a 33 camadas a 4K): `composite_below` sobre a
    pilha inteira. O fio esconde-o do clique, mas não do toque colado.
  - Um fio descartado corre até ao fim da fase em curso (≤ 1 composição) no seu pool, ao lado do traço que começou.
