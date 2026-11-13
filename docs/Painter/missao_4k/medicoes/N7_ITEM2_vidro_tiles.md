# N.º 7, item 2 — o vidro no journal por tile (ADR-0204): estado

> ADR: [`0204`](../../../architecture/decisions/0204-o-vidro-entra-no-journal-por-tile-e-o-desfazer-do-traco-deixa-de-segurar-o-plano.md).
> Linha `line/PainterWatercolor`, 13/11. 2.º agente (45 min): a cópia do 2.º traço saiu; D3 (passo 4) por fazer.

## Passos feitos

| passo | commit | o quê |
|---|---|---|
| 1 (D1) | `d8e8502075` | `VidroJournal` (`undo_window_vidro.rs`, molde do `ReliefJournals`) no `WriteState`; porta `plane_fork::fork_vidro`; os três sítios (`abre_o_vidro` aguada e Wet, `continua_o_vidro_do_traco`) passam por ela com a região do `declare_wrote`. Comportamento igual: o `before` ainda segura o plano. |
| 2+3 (D2+D4) | `5c6766c81e` | Chave `VIDRO_PELO_JOURNAL` (`undo::window::{set_,}vidro_pelo_journal`, `true`). O vidro é a 4.ª família de `ElidedRelief` (`vidros: Weak`); `snapshot_model_eliding_relief`/`set_cursor` esvaziam o mapa forte; commit por `UndoEntry::vidro_map` (`undo_planes.rs`, ANTES do `relief_maps`, que zera as testemunhas) → `StoredMap::from_journal` via `WriteState::with_vidro_before`; elidido+recusado = indescritível; `side` do vidro do VIVO quando o cursor elidiu; `continua_o_vidro_do_traco` lê `vidro_antes_na_janela` (journal+vivo) antes do `fork_vidro`; `base_for_top` exige `vidro_untouched`; a absorção limpa `vidros` com a chave. |
| 2.º traço | `fa5f8d7f83` | `StoredEntry::Aparece { len, win, after }`: sob a chave, o `OnlyAfter` do vidro (o plano NASCEU no passo) troca o `Arc` vivo pela janela fora do neutro (`larga_os_planos_que_aparecem`, `undo_delta_aparece.rs`, depois do `guarda_janelas_que_aparecem`); o refazer `recria` neutro + janela. Alcance = a janela (não `Whole`). |
| D2 Digital | `d733a21555` | Gate `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`. |
| sonda A×B | `85c5ca59db` | `sonda_primeiro_traco`: `PH2D_SONDA_VIDRO=1\|0\|ab` + `PH2D_SONDA_REPETE`; o script pede `ab` × 4. NÃO corrida. |
| 0 (medição) | `40516925ce` | `quem_segura_o_vidro_no_segundo_pen_down` (teste leve, não ignorado) + acessor `cfg(test)` `UndoController::cursor_segura_o_vidro`. |

Sonda pesada do passo 0 a 4K: **na fila** (`bash docs/Painter/missao_4k/medicoes/N7_ITEM2_sonda.sh`, posição 5): o
subagente não pode correr `--ignored`. É a mesma sonda para o «depois» do alvo.

## Medições

**Passo 0, donos do plano** (256×160; a contagem não depende do tamanho; load 7,6):

| momento | `strong` | donos |
|---|---|---|
| repouso depois do 1.º traço | 3 | mapa · cursor · a entrada `OnlyAfter` do 1.º traço (que criou o plano) |
| repouso depois do 2.º traço | 2 | mapa · cursor |
| cópias do plano por traço (2.º e 3.º) | 1 | dentro do EVENTO de pen-down (o 1.º quadro corre nele) |

No pen-down somam-se o `before` (`stroke_undo`) e o congelado da aguada, que recongela na base nova com
`Arc::clone(&plano)`: **4 donos fortes no N-ésimo traço, 5 no 2.º**. Conclusão para o ADR:
- **D3 é necessário para o ALVO**, não só para a correcção: sem ele o congelado continua a forçar a cópia;
- no 2.º traço a entrada `OnlyAfter` do 1.º segura o plano (o `guarda_janelas_que_aparecem` não larga o `Arc`):
  o passo 3 tem de fazer o commit de um plano `absent` sem guardar o `Arc` vivo, ou o 2.º traço continua a copiar
  (o 3.º em diante não).

**Passo 1, gates** (por nome, `ph2d-run.sh cargo test … --lib -- <nomes>`):
- `o_journal_do_vidro_guarda_os_tiles_que_a_regiao_toca_e_so_a_primeira_vez` (unitário, 300×200, tiles parciais
  44 e 72; 2.ª captura não retoma; outra camada ⇒ `mixed`);
- `o_traco_que_cruza_tiles_guarda_so_os_tiles_tocados` (WB 520×390: diagonal, colado a x = 127|128, canto
  parcial; antes reconstruído `journal.get(i).unwrap_or(vivo[i])` = plano de antes AO BIT; bytes ≤ tiles da caixa
  declarada; o 1.º traço = `absent` e 0 B).
- Mutações: área −1 tile ⇒ VERMELHO (antes difere); captura de plano inteiro ⇒ VERMELHO (1 419 600 B > 1 376 256 B).
- Vizinhos verdes por nome: `o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela`,
  `o_desfazer_do_composite_wet_guarda_a_caixa_e_nao_a_tela`, `o_undo_e_o_redo_dao_o_bit_do_caminho_antigo`,
  `undo_os_gemeos_da_camada_adormecida_…`, `o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado`,
  `o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano`, `mask_gate_copia_tests::*`,
  `regiao_do_journal::*` (12 verdes).

**Passos 2+3** (256×160, `quem_segura_o_vidro_no_segundo_pen_down` com a chave ligada):

| momento | antes (oráculo) | com `VIDRO_PELO_JOURNAL` |
|---|---|---|
| repouso depois do 1.º traço | 3 (mapa · cursor · `OnlyAfter`) | **2** (mapa · `OnlyAfter` do 1.º traço) |
| repouso depois do 2.º traço | 2 | **1** (só o mapa) |
| cópias do plano no 2.º traço | 1 | **1** — a entrada `OnlyAfter` do 1.º traço ainda segura o `Arc` |
| cópias do plano no 3.º traço | 1 | **0** |

O congelado da aguada NÃO apareceu como dono neste teste (sem Smudge): o D3 continua a ser preciso para o `vbase`
correcto com Smudge e para a sonda 4K, mas aqui não força cópia.

Gates (verdes por nome, 14): `o_vidro_do_traco_volta_ao_bit_no_desfazer_e_no_refazer` (NOVO: 8 traços WB sobrepostos e
disjuntos; desfaz tudo, refaz tudo, desfaz metade, pinta outro, desfaz; vidro e tela ao bit do oráculo `false` no mesmo
processo E do vidro/tela GRAVADOS depois de cada traço — o `side` é partilhado, a paridade sozinha não pega mutação nele;
`VIDRO_FROM_JOURNAL ≥ 7`) + todos os vizinhos do passo 1. Mutações: «o undo escreve o `after`» ⇒ VERMELHO (desfeito o
traço 8); «o `side` lê o cursor elidido» ⇒ VERMELHO (pânico `undo.rs:488`). Falta a 3.ª do ADR («janela sem
`intersect`» deve ficar verde).

**2.º traço sem cópia** (`fa5f8d7f83`, 520×390):

| | sem a chave | com a chave |
|---|---|---|
| cópias do plano, traços 2.º..8.º (`o_segundo_traco_nao_copia_o_plano_do_vidro`) | `[1,1,1,1,1,1,1]` | **`[0,0,0,0,0,0,0]`** |
| `quem_segura…`: `strong` em repouso depois do 1.º / 2.º traço | 3 / 2 | **1 / 1** (só o mapa) |
| cópias no 2.º / 3.º traço | 1 / 1 | **0 / 0** |

- `o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado`: o controlo dele («o 2.º traço copia») era o
  defeito que saiu; agora corre as duas rotas: 1.º = 0 nas duas, 2.º ≥ 1 sem a chave e = 0 com ela.
- Mutações (gates por nome): **M1** sem a troca (`larga_…` não chamado) ⇒ VERMELHO
  (`o_segundo_traco_nao_copia_o_plano_do_vidro`); **M2** o refazer recria sem a janela ⇒ VERMELHO
  (`o_vidro_do_traco_volta_ao_bit…`, vidro ≠ oráculo); **M3 (a 3.ª pendente do ADR)** «janela sem o `intersect`»
  (`Some(_h) => jwin` em `from_journal`) ⇒ **VERDE** em `o_vidro_do_traco_volta_ao_bit…` e
  `o_traco_que_cruza_tiles…`, como previsto (superconjunto).
- **Digital sobre aguada** (`d733a21555`): 3 aguadas + 4 Digital (fluxo 0,3, vai e volta), vidro e tela ao bit do
  oráculo depois de cada traço, desfeito e refeito; 0 cópias por traço Digital com a chave. Mutação «o ramo sem
  `before` segurado devolve `None`» ⇒ VERMELHO (o ramo do journal é alcançado). ⚠️ **ABERTO:** a mutação «o
  antes lê o VIVO, não o journal» (`vidro_antes_na_janela` sem `j.tiles.get`) **SOBREVIVE** — nas duas cenas
  tentadas (alfa saturado; fluxo 0,3 a voltar sobre si) reler o vidro já selado no quadro anterior dá o mesmo
  selo. Ou a releitura é idempotente por construção (`sela∘cobre∘alfas` sobre o próprio selo) e então o journal
  não é preciso ali — provar no papel —, ou falta a cena que a separa (papel de cor? apagar dentro do traço?).

Gates verdes por nome depois disto (17): os 14 de cima + `o_segundo_traco_nao_copia_o_plano_do_vidro`,
`o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`, `quem_segura_o_vidro_no_segundo_pen_down`
(asserção ajustada: com a chave a entrada do 1.º traço já não segura o plano).

## Armadilhas achadas

1. **O plano que nasce no 1.º quadro era capturado nos quadros seguintes do MESMO traço** (1 MB a 520×390, sem
   antes a descrever). Cura no `capture_vidro`: com `absent` daquela camada, não toma nada. O commit (passo 3) tem
   de ler `absent` ANTES dos tiles: `absent` ⇒ `OnlyAfter`.
2. **`journal_heap_bytes` (cfg(test)) é lido por cinco gates como «os bytes da TELA»** (`captura_da_caixa_tests`,
   `mask_gate_copia_tests`, `regiao_do_journal`, `measure_stroke_owners`). Somar o vidro nele reprovou
   `o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela` (917 504 > 786 432). Ficou tela+relevo; o vidro soma no
   `journal_bytes_retidos` (o de produção, D5) e tem `vidro_journal_bytes`.
3. **`fork_vidro` NÃO abre acesso na janela** (`open_access`): a escrita do vidro vai com a da tela, fechada pelo
   mesmo `declare_wrote`; abrir um 2.º acesso deixava `undeclared = 1` e o commit sem janela.
4. A cópia do 2.º traço acontece **dentro do evento de pen-down**, não no 1.º `on_tick`: quem medir donos tem de
   medir ANTES do `on_canvas_pointer(Down)`.
5. `--ignored` e `fila-pesada.sh pôr '…&&…'` são recusados no subagente: a sonda vai num script (`N7_ITEM2_sonda.sh`).
6. `VIDRO_PELO_JOURNAL` ainda NÃO existe: o passo 1 não muda comportamento. Nasce no passo 2/3.

7. **O `relief_maps` zera `before.relief_elided` inteiro** no sucesso: o vidro tem de sair ANTES (é o que o
   `vidro_map` faz, com `mem::take` das testemunhas do vidro).
8. Uma mutação no `side` atinge as DUAS rotas: o gate compara também com o estado GRAVADO.

## Próximo passo exato

1. **Passo 4 (D3)**: `vbase` por journal da sessão (`BaseCongelada` com `TileJournal`, janela densa no `Quadro`, o
   congelado `molhada`); gate `o_vidro_da_base_da_sessao_e_o_do_inicio_da_sessao_ao_bit` (WB + Smudge, 3 traços na
   sessão; oráculo = cópia inteira tirada na congelação, só em teste; mutação «o `vbase` lê o journal do passo» ⇒
   vermelho); `o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano` e `o_primeiro_traco_…` verdes.
2. Decidir o ABERTO do Digital (acima): prova de idempotência ou cena que reprova a mutação.
3. Coordenador: `bash docs/Painter/missao_4k/medicoes/N7_ITEM2_sonda.sh` (agora A×B intercalado, `ab` × 4).
4. Passos 5 (Wet sobre aguada), 6 (camada nova/duplicar), 7 (tecto de bytes) do ADR.
