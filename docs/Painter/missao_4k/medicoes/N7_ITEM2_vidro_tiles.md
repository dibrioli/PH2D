# N.º 7, item 2 — o vidro no journal por tile (ADR-0204): estado

> ADR: [`0204`](../../../architecture/decisions/0204-o-vidro-entra-no-journal-por-tile-e-o-desfazer-do-traco-deixa-de-segurar-o-plano.md).
> Linha `line/PainterWatercolor`, 13/11. Agente com 45 min impostos: parou no passo 1, verde e comitado.

## Passos feitos

| passo | commit | o quê |
|---|---|---|
| 1 (D1) | `d8e8502075` | `VidroJournal` (`undo_window_vidro.rs`, molde do `ReliefJournals`) no `WriteState`; porta `plane_fork::fork_vidro`; os três sítios (`abre_o_vidro` aguada e Wet, `continua_o_vidro_do_traco`) passam por ela com a região do `declare_wrote`. Comportamento igual: o `before` ainda segura o plano. |
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

## Próximo passo exato

**Passos 2 + 3 juntos** (são acoplados: elidir o `before` sem tirar o delta do journal faz o `split` ver o mapa
vazio ⇒ `OnlyAfter` ⇒ o undo apagaria o vidro), sob a chave `VIDRO_PELO_JOURNAL` (`thread_local`, `true`; `false` =
oráculo):
1. `ElidedRelief` (`undo_elide.rs`) ganha `vidros: BTreeMap<LayerId, Weak<Vec<Vidro>>>`; `snapshot_model_eliding_relief`
   (`tool/layers/undo.rs:26`) e `set_cursor` (`undo_record.rs:137`) elidem o vidro no mesmo passo do relevo;
2. `undo_planes.rs:224`: `StoredMap::from_journal(&before.vidros, &el.vidros, &after.vidros, camada, absent, journal,
   win_scalar)` no molde do `relief_maps` (`:385`), com fallback ao `split` só se o `before` não elidiu (senão
   `indescribable`, como o relevo `:455`); expor o journal por um `with_vidro_before(writes, layer, |j| …)` irmão do
   `with_relief_before` (`undo_window_relief.rs`);
3. `side()` (`undo_planes.rs:500`) do vidro a partir do VIVO, como o relevo; restauro `tool/layers/undo.rs:199`
   (`self.vidros = m.vidros`) não pode trocar o vivo por um mapa elidido vazio;
4. `continua_o_vidro_do_traco` lê `vantes[i]` por `vidro_before(i).unwrap_or(vivo[i])` materializado na janela `r`
   ANTES do `fork_vidro` (tirar o `cfg(test)` de `vidro_before`/`vidro_speaks_for`);
5. Gates: `o_segundo_traco_nao_copia_o_plano_do_vidro` (só verde depois do passo 4/D3 — ver medição: o congelado
   copia), `o_vidro_do_traco_volta_ao_bit_no_desfazer_e_no_refazer`, `o_traco_digital_sobre_a_aguada_…`.
Depois: **passo 4 (D3)** — sem ele o alvo de 0 cópias não chega.
