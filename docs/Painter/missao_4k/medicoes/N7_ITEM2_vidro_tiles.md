# N.º 7, item 2 — o vidro no journal por tile (ADR-0205): estado

> ADR: [`0205`](../../../architecture/decisions/0205-o-vidro-entra-no-journal-por-tile-e-o-desfazer-do-traco-deixa-de-segurar-o-plano.md).
> Linha `line/PainterWatercolor`, 13/11. **FECHADO** pelo 6.º agente (passos 7 e 8, mutações dos 5 e 6). ADR-0205 «Implementado».

## Passos feitos

| passo | commit | o quê |
|---|---|---|
| 1 (D1) | `d8e8502075` | `VidroJournal` (`undo_window_vidro.rs`, molde do `ReliefJournals`) no `WriteState`; porta `plane_fork::fork_vidro`; os três sítios (`abre_o_vidro` aguada e Wet, `continua_o_vidro_do_traco`) passam por ela com a região do `declare_wrote`. Comportamento igual: o `before` ainda segura o plano. |
| 2+3 (D2+D4) | `5c6766c81e` | Chave `VIDRO_PELO_JOURNAL` (`undo::window::{set_,}vidro_pelo_journal`, `true`). O vidro é a 4.ª família de `ElidedRelief` (`vidros: Weak`); `snapshot_model_eliding_relief`/`set_cursor` esvaziam o mapa forte; commit por `UndoEntry::vidro_map` (`undo_planes.rs`, ANTES do `relief_maps`, que zera as testemunhas) → `StoredMap::from_journal` via `WriteState::with_vidro_before`; elidido+recusado = indescritível; `side` do vidro do VIVO quando o cursor elidiu; `continua_o_vidro_do_traco` lê `vidro_antes_na_janela` (journal+vivo) antes do `fork_vidro`; `base_for_top` exige `vidro_untouched`; a absorção limpa `vidros` com a chave. |
| 2.º traço | `fa5f8d7f83` | `StoredEntry::Aparece { len, win, after }`: sob a chave, o `OnlyAfter` do vidro (o plano NASCEU no passo) troca o `Arc` vivo pela janela fora do neutro (`larga_os_planos_que_aparecem`, `undo_delta_aparece.rs`, depois do `guarda_janelas_que_aparecem`); o refazer `recria` neutro + janela. Alcance = a janela (não `Whole`). |
| D2 Digital | `d733a21555` | Gate `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`. |
| sonda A×B | `85c5ca59db` | `sonda_primeiro_traco`: `PH2D_SONDA_VIDRO=1\|0\|ab` + `PH2D_SONDA_REPETE`; o script pede `ab` × 4. NÃO corrida. |
| 4 (D3) | `7d170f223d` | `BaseCongelada = (Weak, camada, SessaoDoVidro)`: `TileJournal<Vidro>` ancorado na congelação, alimentado no `abre_o_vidro` (antes da escrita do quadro) e no `continua_o_vidro_do_traco` (aguada e `molhada` da mesma camada). `Quadro.vbase` = `JanelaDoVidro` densa da caixa do quadro, `Index` pelo índice GLOBAL (os sete leitores em `texel.rs`/`pixel.rs` só mudam de tipo). O congelado deixa de ser dono do plano. |
| 0 (medição) | `40516925ce` | `quem_segura_o_vidro_no_segundo_pen_down` (teste leve, não ignorado) + acessor `cfg(test)` `UndoController::cursor_segura_o_vidro`. |
| 6 (cópia) | `d1d8940867` | `GuardaDaSessao { tela, camada }`: a guarda da sessão molhada leva a camada dona; a CÓPIA (que partilha o `Arc`, B18.1) não herda a sessão. |
| D2 Digital | `cd08bc8ef3` | Cena de aguada alfa < 255 (camada transparente) em papel de cor, com dreno por evento: separa o journal do vivo. |
| 5 + 6 | `f0996ce78b` | `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer`, `a_camada_nova_e_o_vidro_elidido_dao_o_bit`. |
| 7 (D5) + mutações 5/6 | `80559a3331` | `VidroCongelado::bytes` em `bytes_do_instantaneo`; gate `o_tecto_do_desfazer_conta_o_journal_do_vidro`; «0 cópias depois do +» asserido; cura do build sem `debug_assertions`. |
| 8 (D7) | — | Censo: nada a curar (abaixo). |

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

Gates por nome depois disto (**16 passam + 1 ignorado**, `sonda_b40a4_custo_da_protecao_4k`; o «17» de antes contava o ignorado): os 14 de cima + `o_segundo_traco_nao_copia_o_plano_do_vidro`,
`o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`, `quem_segura_o_vidro_no_segundo_pen_down`
(asserção ajustada: com a chave a entrada do 1.º traço já não segura o plano).

**D3** (`7d170f223d`, 256×160): gate `o_vidro_da_base_da_sessao_e_o_do_inicio_da_sessao_ao_bit` (3 traços WB +
Smudge 0,4 na mesma sessão; cada janela que `abre_o_vidro` deu = o plano inteiro tirado na congelação, ao bit; oráculo
só em teste por `GUARDA_O_INICIO_DA_SESSAO`). Controlos verdes: a sessão atravessa traços (mesmo `inicio` em traços
diferentes) e há janelas onde o vivo de antes do traço ≠ início. Mutação «o `vbase` lê o journal do passo»
(`win.vidro_antes_na_janela`) ⇒ **VERMELHO** (traço 0, linha 57). `o_vidro_segue_a_base…` (n.º 5): o controlo media «o
caminho sem B18.4 recopia o plano»; com o D3 recongelar já não copia, então a régua passou a `CONGELACOES` (sem a cura
≥ moves−2; com ela = 1) — o mesmo defeito, na unidade nova. Corrida de vizinhos por nome: 24 verdes + 1 ignorado
(inclui os 16 acima, `vidro_tests::*`, `regiao_do_journal::*`, `mask_gate_copia_tests::*`, `plane_copy::…vidro…`).

**ABERTO do Digital — decidido no papel: NÃO é idempotente, o journal fica.** `alfas(o, v)` só devolve os alfas do selo
quando `v[..4] == o`; um texel selado no quadro k guarda `px_k ≠ o`, e reler o vivo em k+1 cai no ramo «o alfa
desceu ⇒ escala por `a'/a`» (ou na cobertura `f`) em vez de dar `alfas(o, vantes)`. As duas cenas tentadas passaram
porque com `o` de alfa 255 sobre aguada saturada o ramo `a0 = 1` dá o mesmo selo. Cena que falta: Digital sobre uma
aguada de **alfa < 255** (Dilution alta, ou camada transparente) com o MESMO texel tocado em dois quadros (vai e volta
devagar), fluxo < 1. Não simplificar o código.

**Risco do D3: censo dos escritores do plano do vidro** (4.º agente, 13/11). Fora do `abre_o_vidro` e do
`continua_o_vidro_do_traco` só escrevem o restauro (`layers/undo.rs` `self.vidros = m.vidros`), duplicar
(`estrutura.rs`, insere o `Arc` na camada NOVA), apagar (`poda_os_mapas_por_camada`) e a troca de documento
(`documents.rs`/`trait_impls_raster.rs`, que já zeravam o congelado). O composite Wet escreve pelo `abre_o_vidro` da
`molhada`.
- **Ctrl+Z / Ctrl+Shift+Z: correcto por construção.** O restauro chama `wetpaint_end_session` e `dry_session_now`, então
  a aguada seguinte recongela a partir da tela restaurada; se a tela restaurada for o MESMO `Arc` da base antiga, o
  journal dela guarda valores do início e o vivo restaurado também é o início. Gate
  `o_ctrl_z_no_meio_da_sessao_molhada_da_o_vidro_da_base_ao_bit` (oráculo independente do journal: cada janela do
  `vbase` = o vidro vivo no pen-down cuja tela é a base da sessão; 3 traços na sessão (controlo: base = pen-down 0),
  Ctrl+Z, desfazer até ao 1.º, desfazer+refazer, desfazer tudo; > 20 janelas). Mutação «a janela da sessão lê o vivo»
  ⇒ **VERMELHO**.
- **Curas defensivas** (mutação SOBREVIVE, por desenho): o restauro larga as duas sessões
  (`VidroCongelado::larga_as_sessoes(None)`), a poda larga a da camada apagada (o id reaproveita-se), e o
  `abre_o_vidro` captura também na sessão do OUTRO meio da mesma camada (o mesmo que o `continua_o_vidro_do_traco` já
  fazia). Gate `trocar_de_meio_a_meio_da_sessao_da_o_vidro_da_base_ao_bit` (aguada ×3, Wet na mesma camada, aguada ×2):
  verde, mas o traço Wet muda a tela, a guarda da sessão da aguada quebra e a aguada seguinte abre sessão NOVA
  (base = o pen-down depois do Wet) — a mutação «sem a captura cruzada» fica VERDE: não há cena conhecida em que as
  duas sessões vivas se cruzem.
- ⚠️ **ACHADO (anterior ao D3, passo 6): a sessão molhada atravessa para a CÓPIA da camada.** `duplicate_layer`
  partilha o `Arc` da tela; a guarda (`wet_session_canvas`) bate e a aguada na cópia continua a sessão da original
  (base = a tela do início da sessão), mas o congelado (camada ≠) recongela do plano da cópia, que já tem os traços
  da sessão: o `vbase` ≠ o vidro da base (reprova o oráculo acima, linha 43). Antes do D3 era igual (`Arc::clone` do
  vivo). Cura a decidir no passo 6: duplicar/trocar de camada seca a sessão, ou a cópia herda a sessão (journal) da
  original.

**5.º agente (13/11).**
- **Cópia da camada (passo 6), CURADO.** Decisão: a cópia é um objecto novo e começa SEM sessão; a sessão é da
  original. Coerente com a troca de camada de hoje: A→B→A sem pintar B continua a sessão de A (a tela volta com o
  mesmo `Arc`, `flush_active_to_images`/`own_image`); pintar aguada em B abre sessão nova (a de A acaba), como antes.
  Cura sem custo (copiar o plano para dar `Arc` novo à cópia custaria 67 MB a 4K): `wet_session_canvas` passa a
  `GuardaDaSessao { tela, camada }`, e `wet_session_continues` exige `camada == layers.active()`
  (`watercolor_backdrop.rs`, `arma_a_guarda_da_sessao` nos três armadores). Gate
  `duplicar_a_camada_com_vidro_e_pintar_uma_nao_muda_a_outra` (2 aguadas, duplicar; a cópia começa sessão própria;
  pintar a cópia não muda a original e vice-versa; ir à original e voltar sem pintar CONTINUA a sessão da cópia;
  `vbase` ao bit em > 20 janelas) + a duplicação de volta em `trocar_de_meio_a_meio_da_sessao_da_o_vidro_da_base_ao_bit`.
  Mutação «a guarda sem a camada» ⇒ **VERMELHO nos dois** («a cópia herdou a sessão»). Vizinhos verdes por nome:
  `watercolor_session` (26), `vidro` (29), `troca` (31), `adormec` (29), `copias_de_plano` (4), `duplicate` (2),
  `watercolor_liga_a_meio`, `wet_session`.
- **Cena do Digital, FECHADA.** A mutação sobrevivia por um motivo de RÉGUA, não de cena: o vidro só segue o traço
  QUADRO a QUADRO no dreno da pré-visualização (`segue_o_vidro_no_quadro`, `dreno.rs`, e só com papel de cor), e o
  teste nunca chamava `take_preview_arc` — então `continua_o_vidro_do_traco` corria uma vez, no pen-up, com
  vivo = antes (sonda: 8 leituras, 0 diferenças). Cena nova no mesmo gate: camada transparente (alfa < 255,
  controlo) + papel de cor + `take_preview_arc` por evento: 108 leituras, 100 com journal ≠ vivo. Mutação «o
  antes lê o vivo» ⇒ **VERMELHO** («o vidro difere do oráculo»).
- **Passo 5** `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer`: 3 aguadas + 3 Wet (vai e volta, dreno) em
  papel de cor; controlo: o Wet escreveu o vidro; desfaz 6 e refaz 6, o documento inteiro (todos os vidros e telas) ao
  bit do oráculo a cada passo. **Verde** (sem mutação própria).
- **Passo 6** `a_camada_nova_e_o_vidro_elidido_dao_o_bit`: verde; desfaz até ao fim (≥ 6 passos, a camada nova sem
  vidro) e refaz tudo ao bit. **Cópias do plano no 1.º traço na base depois do «+»: 0** (gravado, não asserido).
- Gates do ficheiro do journal: 7 verdes por nome (`watercolor_vidro_journal_tests`), do restauro: 3.

## Resumo final (6.º agente, 13/11)

- **Passo 7 (D5).** `SessaoDoVidro::bytes` deixou de ser só de teste. `VidroCongelado::bytes` (as duas sessões)
  soma em `bytes_do_instantaneo`. Gate `o_tecto_do_desfazer_conta_o_journal_do_vidro` (520×390, 3.º traço aberto):
  - o passo tem 688 128 B, contados em `journal_bytes_retidos`;
  - largar a sessão baixa o instantâneo em **917 504 B**, entre os 802 816 B dos tiles mudados desde o início
    (oráculo `GUARDA_O_INICIO_DA_SESSAO`) e os 1 419 600 B do plano;
  - com `max_bytes` = metade da pilha cheia do oráculo, a pilha retém 4 passos contra 4 (controlo: o orçamento morde,
    4 < 8).
  Mutação «sem a sessão no instantâneo» ⇒ **VERMELHO**. `medida_instantaneo_memoria` está verde por nome
  (`--profile smoke`): retido ÷ declarado 1,022 (splash 1024²) e 1,017 (worker 512²).
- **Cura achada pelo caminho:** a crate não compilava sem `debug_assertions` (`smoke`/`release`). O
  `vidro_antes_na_janela` (D2, produção) chamava `TileJournal::get`, que só existe em debug. Agora lê por corrida com
  `read_row_into`, guardado por `is_empty`, como `SessaoDoVidro::janela`; `vidro_before` e `vidro_speaks_for` ganharam
  o `cfg` dos portões. A mutação «o antes lê o vivo» continua **VERMELHA** em
  `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal`. O build `smoke` sai sem avisos.
- **Mutações próprias dos passos 5 e 6:**
  - passo 5, «a escrita do vidro sem a porta» (`Arc::make_mut` em vez de `fork_vidro`): **VERMELHO**, mas pelo
    `debug_assert` «indescritível» de `undo_record.rs:71`, não pela comparação ao bit. O sistema acusa o journal
    recusado; em release o passo seria descartado.
  - passo 6, «o cursor segura o vidro» (`at.vidros.clear()` desligado): **VERMELHO** pela asserção nova das cópias.
  - Sobreviveram duas no passo 6, ambas registadas: «sem `note_absent_vidro`» e «sem a troca do `Aparece`». Na base,
    o traço 1 já tinha trocado o `Arc`, então a entrada do traço 0 não o segura no traço 4.
- **«0 cópias no 1.º traço na base depois do «+»» agora é asserção.** O journal dá 0; o oráculo dá 1 (controlo).
- **Passo 8 (D7), censo:** no gesto WB e Wet só há três `fork_canvas`, e todos passam a caixa
  (`watercolor_render.rs:333`, `wetpaint/composite.rs:302`, `borracha_morta.rs:236` com `Some(regiao)`). O `None` só
  aparece com o A/B `CAPTURA_PELA_CAIXA` desligado. O `capture_canvas_arc` só existe dentro da porta (`plane_fork.rs:207`, que
  passa o `Arc` sem copiar) e no `replace_canvas` (Fill, crop, resize). Os `None` que restam (Fill, Sculpt, Inpaint,
  Region, Selection) estão fora do gesto. **Nada a implementar.**
- **Vizinhos verdes por nome:** 119 passaram + 15 ignorados (`vidro`, `watercolor_session`, `troca`, `adormec`,
  `copias_de_plano`, `duplicate`, `captura_da_caixa`, `mask_gate_copia`, `regiao_do_journal`,
  `o_undo_e_o_redo_dao_o_bit_do_caminho_antigo`).
- **Sonda 4K do coordenador** (`N7_ITEM2_sonda_r1.log` = antes do D3; `N7_ITEM2_sonda_r2_depois_D3.log` = depois),
  r2, 2.º pen-down WB 4K, `ab` intercalado:

  | | mínimo | média |
  |---|---|---|
  | journal | **11,8 ms** | 13,6 ms |
  | rota antiga | 14,0 ms | 17,2 ms |

  Load 5–8: o relógio é só indicativo.

### O que fica aberto
1. **A tela inteira no fork.** `fork_par_raw` copia 33 MB a 4K quando outro dono forte segura o `Arc` da tela; o
   mesmo vale para `undo_vivo/casco.rs` (`make_mut` com pixel fora da caixa). A caixa reduz só o que o journal
   guarda. Falta confirmar se isto acontece no pen-down WB (o `strong` da tela). Se acontecer, a cura é uma porta de
   fork por tile. Custo estimado, NÃO medido: ≈ 3–5 ms por cópia.
2. Ler o `vidro_do_quadro` nas partes da sonda r2. O alvo do ADR é ≤ 1 ms.
3. A mutação do passo 5 que pegue pela comparação ao bit, e não pelo `debug_assert`.
4. O «+» e as entradas estruturais (D6, item 3).
5. O passo 9: alarmes de pincel, clippy/fmt da crate 1× e a matriz do app, no fecho da onda.

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
9. **Verde em `dev` não é verde em `smoke`/`release`:** um método `cfg(any(test, debug_assertions))` chamado da produção só reprova sem `debug_assertions`. O `--test medida_instantaneo_memoria` (`--profile smoke`) pegou-o.

7. **O `relief_maps` zera `before.relief_elided` inteiro** no sucesso: o vidro tem de sair ANTES (é o que o
   `vidro_map` faz, com `mem::take` das testemunhas do vidro).
8. Uma mutação no `side` atinge as DUAS rotas: o gate compara também com o estado GRAVADO.

## Próximo passo exato

1. **Passo 7 (D5)**: `o_tecto_do_desfazer_conta_o_journal_do_vidro`. O journal do PASSO já soma em
   `WriteState::journal_bytes_retidos` (`undo_window.rs:428`, `+ self.vidro.borrow().heap_bytes()`). Falta a
   SESSÃO: `SessaoDoVidro::bytes` (`watercolor_vidro.rs:54`) é `cfg(test)`; tirá-lo do `cfg`, somar as duas sessões de
   `vidro_congelado` num `vidro_congelado.bytes()` e esse no `bytes_do_instantaneo` (`instantaneo.rs:188`) — ⚠️ o gate
   `medida_instantaneo_memoria` compara esse número com o alocador: correr por nome depois. Gate: gesto aberto ⇒ os
   bytes = tiles capturados (passo + sessão); com o mesmo `max_bytes` a pilha retém ≥ os passos do oráculo.
2. **Passo 8 (D7)**: censo dos `fork_canvas(.., None)` / `capture_canvas_arc` no traço WB e Wet a 4K (não começado).
3. Coordenador: `bash docs/Painter/missao_4k/medicoes/N7_ITEM2_sonda.sh` (A×B, `ab` × 4).

### Lista anterior (feita até ao 4)

0. Risco do D3: FEITO (acima); falta o achado da cópia (passo 6).
1. **Cena do Digital** (acima): aguada de alfa < 255 + Digital lento, fluxo 0,3, mesmo texel em 2 quadros; acrescentar
   ao `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal` e ver a mutação «lê o vivo» VERMELHA.
3. **Passo 5 (Wet)**: `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer` (o `molhada` já vai pelo D3).
4. **Passo 6**: `a_camada_nova_e_o_vidro_elidido_dao_o_bit`, `duplicar_a_camada_com_vidro_e_pintar_uma_nao_muda_a_outra`.
5. **Passo 7 (D5)**: `o_tecto_do_desfazer_conta_o_journal_do_vidro` — somar `SessaoDoVidro::bytes` no dono do
   congelado e em `journal_bytes_retidos`.
6. Coordenador: `bash docs/Painter/missao_4k/medicoes/N7_ITEM2_sonda.sh` (A×B, `ab` × 4) — agora mede também o D3.
