# ADR-0205 — O vidro entra no journal por tile e o desfazer do traço deixa de segurar o plano

- **Status:** Accepted, **Implementado** (13/11, passos 0–8; ver «Implementado» no fim) — missão Painter 4K; o dono autorizou em 13/11 («2- pode»: desfazer em janela/tiles do vidro e do
  canvas por traço); desenho técnico delegado (padrão-ouro).
- **Data:** 2026-11-13
- **Linha:** `line/PainterWatercolor` (onda n.º 7, item 2)
- **Estende:** o S3 do desfazer (doc 28 §5.58–§5.60: journal por tile + elisão do `before`, degrau 4 do relevo) e o B17.4
  (a tela elidida). Não mexe em contrato congelado (§6).
- **Precedente directo:** `8627ecf68a` (n.º 6) — o `fork_canvas` da aguada passa a caixa do `declare_wrote` e o journal
  guarda os tiles dela (33 MB → a caixa); `4e4486cb30` — o mesmo no composite Wet.
- **Amparo:** sonda `sonda_primeiro_traco` (`ph2d-tool-painter`, `PH2D_SONDA_PARTES=1`), handoff n.º 6 §3/§4/§6.

> Número recontado na integração de 13/11 (§5.0): a linha escreveu `0204`; a linha JointEditor tomou o `0201`.

## Contexto

### O número

Sonda `sonda_primeiro_traco`, Watercolor Basic, 3840×2160, preparo, `--profile smoke` (handoff n.º 6 §3, load 6–10):
depois das curas da n.º 6 o pen-down do 2.º traço caiu de 28–29 para 14,4–15,4 ms, e o que sobra é dominado por uma fase:

| fase do 2.º pen-down (WB 4K) | tempo | memória nova (faltas de página) |
|---|---|---|
| `vidro_do_quadro` (`abre_o_vidro` → `par_clone` do plano) | **11,5 ms** | **115 MB** |

O plano do vidro tem `3840 × 2160 × 7 B = 58,1 MB`. Os 115 MB são ≈ 2× o plano: a cópia escrita mais as faltas de
LEITURA das páginas-zero de um plano só lido (handoff n.º 6 §7: «MB novos contam também faltas de leitura»). A
decomposição exacta fica para o passo 0 do plano. O 1.º traço da camada nova não copia (`88263d549a`: o plano que nasce
zerado não se copia). A partir do 2.º traço, **todo** traço copia o plano inteiro.

### O mecanismo (lido no código, 13/11)

`abre_o_vidro` (`tool/paint/watercolor_vidro.rs:256`) tira o plano do mapa `PainterTool::vidros` e, se
`Arc::strong_count > 1`, faz `par_clone` do plano inteiro antes de o escrever no lugar. Fora do gesto o plano tem até
**três donos fortes**:

1. **o `before` do traço** — `paint.stroke_undo: ModelSnapshot` tem `vidros: Vidros` (`undo.rs:105`), um
   `BTreeMap<LayerId, Arc<Vec<Vidro>>>`. O `snapshot_model_eliding_relief` (`tool/layers/undo.rs`) elide o relevo e a
   tela, mas **copia o mapa do vidro forte** (`vidros: self.vidros.clone()`);
2. **o `cursor` do histórico** — o `set_cursor` (`undo_record.rs:137`) elide `heights`/`covers`/`mats` e não elide o
   vidro;
3. **o vidro da base da sessão** — `VidroCongelado::aguada`/`molhada` = `(Weak<base>, camada, Arc<Vec<Vidro>>)`. Na 1.ª
   congelação ele fica com `Arc::clone(&plano)`, isto é, o plano de antes da sessão. É ele que o texel lê como `vbase`
   (`watercolor_render/texel.rs:195,393,454`, sempre no índice do próprio texel `gy*fw+gx`).

O commit do traço deriva o delta do vidro por `StoredMap::split(before.vidros, after.vidros)` (`undo_planes.rs:224`),
que só existe porque o `before` segura o plano. As outras três famílias de plano por camada (relevo) e a tela já
saíram deste regime: o lado `before` delas são os bytes velhos que o `TileJournal` capturou na primeira escrita de cada
tile (`undo_journal.rs`), e o snapshot guarda só uma testemunha `Weak` (`undo_elide.rs`). **O vidro é a única família
do documento que ficou de fora.** Nunca foi uma escolha: o vidro chegou depois (doc 48) e entrou pelo caminho
clássico.

Há outros dois leitores do «antes» do vidro:
- `continua_o_vidro_do_traco` (`watercolor_vidro.rs:118`): os outros meios (Digital, Impasto, Solid) continuam o vidro
  a partir de `stroke_undo.vidros[camada]` (`vantes[i]`, no próprio índice, dentro da região). Tem a sua própria cópia
  `strong_count > 1 ⇒ par_clone`;
- o composite Wet sobre uma aguada (`wetpaint/composite.rs:261`): `abre_o_vidro` com o congelado `molhada`.

E dois sítios que tornam `Arc::get_mut` frágil: o laço da aguada (`watercolor_render.rs:400`) e o do Wet
(`wetpaint/composite.rs` ~276) fazem `Arc::get_mut(p).expect("o plano é só deste quadro")`. `get_mut` devolve `None`
com **qualquer** `Weak` vivo (`plane_fork.rs:286`, a regressão de 4× do Wet). Com uma testemunha `Weak` do vidro, o
`expect` entraria em pânico.

### A tela («e do canvas»)

No traço WB e no Wet a tela já está neste regime: elidida no `before` e no cursor (B17.4), com o `fork_canvas` da
aguada e do composite a passar a caixa (n.º 6). Falta só o censo dos sítios do gesto que ainda chamam
`fork_canvas(.., None)` e capturam a tela inteira. O fork de 6 ms · 33 MB do 1.º traço Wet **não** é do desfazer: é
o COW contra a base da sessão Wet (strong=3, handoff n.º 6 §1) e fica fora deste ADR.

## Decisão

**O vidro passa ao regime das outras famílias do documento: journal por tile na primeira escrita, elisão no `before`
e no cursor, e o delta do commit tirado do journal.** O vidro da base da sessão deixa de ser um `Arc` do plano e
passa a um segundo journal ancorado no início da sessão. Nenhum dono forte além do mapa `vidros` sobrevive a um
gesto, e por isso o 2.º traço em diante escreve o plano no lugar.

### D1 — O journal do vidro (o passo)

- `WriteState` ganha `vidro: RefCell<VidroJournal>`, o molde de `ReliefJournals` (`undo_window.rs:199`): um
  `TileJournal<Vidro>` mais a camada de que fala (`note_layer`), `mixed`, `incomplete` e `absent`. É um mapa por
  camada, como o relevo: dois planos de camadas diferentes têm a mesma forma, e um passo que tocasse dois vidros
  misturaria bytes no mesmo índice. Um 2.º dono marca o journal como `mixed` e o commit cai no caminho de sempre
  (*lento, nunca errado*).
- **Tile:** o `TILE = 128` de sempre (`undo_journal.rs:69`), em elementos do plano. Como o elemento do vidro é o texel,
  um tile são 128×128 texels = **114 688 B** (a tela, com elemento = byte, tem tiles de 32 px × 128 linhas = 16 KB).
  A grade coincide com a das `TileMarks` de 128² (B17.5). ⛔ Nenhum tamanho novo sem medição: trocar o `TILE` muda
  as quatro famílias e a tela, e não há tabela que o justifique.
- **Copy-on-write por tile, antes da primeira escrita:** uma porta nomeada `plane_fork::fork_vidro(arc, win, camada,
  size, area)`, irmã de `fork_heights`/`fork_covers`/`fork_mats`. Captura no journal os tiles que `area` toca e que
  ainda não foram tomados (a primeira captura é a que vale, `undo_journal.rs:29`) e depois devolve o plano para
  escrita por `fork_par_raw` (`strong_count`, depois `make_mut`). `area = None` captura o plano inteiro: correcto e
  caro, é o fallback de quem não sabe onde escreve. Plano sem forma de canvas (1.º toque da camada) =
  `note_absent` (`OnlyAfter`), não `incomplete`.
- **A área é a do `declare_wrote`**, a mesma `region` que a n.º 6 já passa ao `fork_canvas` da aguada
  (`watercolor_render.rs:330`) e do composite Wet. Um superconjunto é seguro e um subconjunto perde a edição em
  silêncio (`plane_fork.rs:166`). Por isso a área é a caixa FINAL do laço, a que o `declare_wrote(Some(region))`
  declara.
- **Os três sítios de escrita** passam pela porta: `abre_o_vidro` (aguada e Wet) e `continua_o_vidro_do_traco`.
  Saem os seus `par_clone` locais. ⛔ Os dois `Arc::get_mut(p).expect(..)` dos laços passam a `Arc::make_mut` através
  da porta. Com a testemunha `Weak` viva, o `make_mut` MOVE o `ArcInner` (24 B, o buffer fica) e não copia
  (§5.12: 0,0000 ms).

### D2 — A elisão (o `before` e o cursor)

- `ModelSnapshot` ganha `vidros_elided: BTreeMap<LayerId, Weak<Vec<Vidro>>>`, dentro de `ElidedRelief` como 4.ª
  família ou ao lado dele. A escolha é da implementação: o critério é «uma porta só baixa as testemunhas das quatro
  famílias».
- `snapshot_model_eliding_relief` e `set_cursor` elidem o vidro **no mesmo passo** em que elidem o relevo: baixam as
  testemunhas e esvaziam o mapa forte. Um snapshot que elidisse e segurasse carregaria o fato duas vezes (o aviso de
  `tool/layers/undo.rs:74`).
- **A regra da testemunha é a do `canvas_held`** (`undo_elide.rs:149`), não a de «o `Weak` tem de dar upgrade». A 1.ª
  escrita depois da elisão MOVE o plano de alocação e a testemunha morre. Com o journal a falar, quem responde é
  ele: `antes[i] = journal.get(i).unwrap_or(vivo[i])`, a identidade da §5.28. Com o journal calado, a testemunha tem
  de ser o vivo (`Weak::upgrade` + `Arc::ptr_eq`), ou o antes não existe e quem pergunta recusa.
- **Os leitores do antes passam o VIVO**, como o undo/redo já fazem para o relevo (`undo_record.rs:134`).
  `continua_o_vidro_do_traco` lê `vantes[i]` por uma `VistaDoAntes` (journal + vivo) materializada na janela `r` que
  ele vai escrever (abaixo, D3). A rede do `PH2D_UNDO_AUDIT` ganha `vidro_audit`, o molde do `canvas_audit`: a cópia
  que a elisão largou, para conferir o journal.

### D3 — O vidro da base da sessão (o `vbase`)

O `vbase` **não é o antes do traço**: é o vidro do início da sessão molhada, que pode atravessar vários traços (a
mesma base congelada reaproveita-se, `watercolor_vidro.rs:19`). Por isso tem âncora própria:

- `BaseCongelada = (Weak<Vec<u8>>, LayerId, Arc<Vec<Vidro>>)` passa a `(Weak<Vec<u8>>, LayerId, TileJournal<Vidro>)`:
  um journal **da sessão**, zerado na congelação e alimentado pela mesma porta `fork_vidro` (que captura nos dois
  journals; cada um toma um tile no máximo uma vez). A chave de frescura (`Weak` da base + camada) e o
  `a_base_mudou_de_arc` (`a0b5c360e6`) ficam como estão.
- O texel lê o `vbase` só no próprio índice (`b[gy*fw+gx]`, três sítios em `texel.rs`). Por isso o `Quadro.vbase`
  passa a uma **janela densa materializada** da região do quadro, rw×rh texels, extraída do journal da sessão com o
  vivo por baixo, antes do laço e antes do `fork_vidro` do quadro. Fica do tamanho da região, sem lookup por tile
  dentro do laço e sem empréstimo partilhado do plano que o laço escreve por `&mut`. O índice passa a
  `(gy-ry0)*rw + (gx-rx0)`. Em `cfg(test)`, um `debug_assert` de que todo índice cai na janela.
- Mesmo desenho para o congelado `molhada` do composite Wet.

### D4 — O commit e o undo/redo AO BIT

- O commit do traço tira o delta do vidro por `StoredMap::from_journal(vivo, journal_do_vidro, hint)`. É a porta
  genérica que o relevo já usa (`undo_delta_journal.rs:155`): `Patch` da caixa de tiles apertada pela janela
  declarada (`PlaneWindow::intersect`), `Unchanged` sem captura, `Whole` só para a captura de plano inteiro. O
  `guarda_janelas_que_aparecem(linha_sem_vidro, vidro_neutro)` de hoje (`undo_planes.rs:226`) aplica-se ao resultado,
  igual.
- **Undo** = escrever o lado `before` do `Patch` sobre o plano vivo, nas mesmas linhas×colunas da janela. **Redo** =
  o lado `after`. Os dois lados ficam guardados e correspondem-se índice a índice (a ordem do `extract_journal`). O
  bit é exacto porque fora da janela o antes **é** o agora (§5.28) e dentro dela os dois lados foram copiados, não
  recalculados.
- A proveniência fica igual: o journal do vidro zera no `begin_step`/`set_cursor` junto com os outros
  (`reset_journal`). Se o journal não descreve o passo (`journal_describes_step_at` falso, `mixed`, `incomplete`), o
  commit materializa o antes como hoje e o passo fica lento, nunca errado.

### D5 — O tecto de bytes do desfazer

- **Nenhuma constante nova.** O orçamento continua `history_budget_bytes` (`UndoController::set_max_bytes`,
  `undo.rs:567`), derivado do documento.
- O que muda é o que cada entrada pesa. Hoje o `StoredMap::split` do vidro já devolve um `Patch` (a entrada não
  guarda o plano), mas o `before` vivo e o cursor seguram 58 MB fora da contabilidade. Depois deste ADR a entrada
  guarda `2 × tiles_tocados × 114 688 B` no máximo (os dois lados da janela apertada; menos, porque o `intersect`
  aperta a caixa de tiles) e ninguém segura o plano. `heap_bytes`/`copiado_bytes` (`undo_planes.rs:567,592`) já somam
  `vidros`.
- `WriteState::journal_bytes_retidos` e `journal_heap_bytes` (`undo_window.rs:410,426`) passam a somar o journal do
  vidro, e o dono do congelado reporta o journal da sessão. A memória do gesto aberto fica visível à régua do tecto e
  à sonda.

### D6 — Fronteira com o «+» (item 3, NÃO resolvido aqui)

O «+» passa por `commit_structural_edit` com `snapshot_model()` (o que SEGURA), e a sua entrada sai de
`UndoEntry::split` (14 ms a 4K comparando a tela velha com a nova zerada, handoff n.º 6 §1). Este ADR **não** muda as
entradas estruturais. A fronteira fica dita assim:
- a camada nova nasce sem vidro (`vidros` sem a chave): o 1.º traço nela é `note_absent` ⇒ `OnlyAfter` neste
  regime, e desfazer o traço remove a chave, como hoje;
- uma entrada estrutural que guarde `Whole` com o `Arc` do vidro de OUTRA camada volta a ser um dono forte. O 1.º
  traço nessa camada depois do «+» copiaria o plano uma vez. Quem decide se a entrada estrutural elide (ou guarda
  `Unchanged` para planos que não mudou) é o item 3 (`UndoEntry::split` sem diff, com a contabilidade do tecto). O
  gate D-cam abaixo mede o que acontece, para o item 3 partir de um número;
- duplicar a camada (`tool/layers/estrutura.rs:121`) PARTILHA o `Arc` do vidro entre duas camadas: aí `strong > 1` é
  legítimo e a 1.ª escrita copia o plano inteiro. Fica assim (gesto estrutural raro); o gate garante só que pintar uma
  não muda a outra.

### D7 — A tela (o resto do «e do canvas»)

Censo, no traço WB e no Wet, dos `fork_canvas(.., None)` e dos `capture_canvas_arc` (captura inteira) que o gesto
atravessa a 4K: um contador de teste por sítio (`#[track_caller]`). Cada sítio do gesto que capture a tela inteira
passa a sua caixa, com o mesmo gate «guarda a caixa e não a tela» da n.º 6. Os sítios fora do gesto (filtros,
ferramentas estruturais) ficam `None`, que é correcto.

## Alternativas recusadas

| # | alternativa | porquê não |
|---|---|---|
| a | **Manter o `before` forte e copiar só a caixa no `abre_o_vidro`** (COW parcial à mão) | Com `strong > 1` não há escrita no lugar sem copiar o plano: quem segura o `Arc` vê a escrita. Seria preciso um tipo de plano com tiles próprios (abaixo, b), ou continuar a copiar. |
| b | **O plano do vidro em tiles `Arc` próprios** (`Vec<Arc<[Vidro; TILE²]>>`, COW por tile no tipo) | Muda o tipo de um plano que o compositor lê em fatias contíguas (`compose.rs`, `barreira.rs`, os laços `par_chunks_mut(fw)`): toca o compositor inteiro e a GPU. Medido no journal (§5.25): 1024 alocações de 16 KB em série perdem 3,73× para uma cópia paralela. O journal já resolve a mesma coisa sem mudar o plano. |
| c | **Journal partilhado com dono único sem a guarda `strong > 1`** (recusa §4c do handoff n.º 6) | MEDIDO: moveu 34 MB para o 2.º traço (11 → 45 MB), porque o `make_mut` passava a mudar o ponteiro da tela que o guarda de identidade do Wet lê. Aqui o vidro não tem esse guarda, mas a regra mantém-se: a porta nunca troca o `Arc` de um plano com dono único por iniciativa própria. A captura de plano inteiro pelo `Arc` (`capture_shared`) fica só para `strong > 1`, como na tela. |
| d | **Reserva de planos entre sessões / zerar só a caixa suja** (recusa §4b) | MEDIDO na aguada: vidro 8,4 → 14,6 ms, pen-down igual (50,1 × 50,9). Com 33,2 MB a tela fica abaixo dos 32 MiB do `mmap`, e as faltas só mudam de buffer. Não se reconstrói para o vidro. |
| e | **Nascer zerado no fork em vez de copiar** (recusa §4a) | MEDIDO: a varredura «é tudo zero?» custa o mesmo que a cópia (5,2 × 5,4–6,4 ms). Nada a ver com este ADR, que nunca varre o plano. |
| f | **Um retângulo por plano em vez de tiles** | O passo escreve em regiões que crescem, e um retângulo obriga a capturar o complemento (até quatro pedaços) com perda silenciosa ao errar (`undo_journal.rs:22`). A grade com bit de «já capturado» é idempotente. |
| g | **Derivar o `vbase` do journal DO PASSO** | O `vbase` é o vidro do início da SESSÃO, e a sessão atravessa traços. O journal do passo zera em cada commit e daria o antes do traço, não o da sessão: pixels errados com a sessão aberta em dois traços (Smudge ligado, `a0b5c360e6`). |
| h | **Ler o `vbase` por lookup de tile dentro do laço** | Lookup por texel no laço quente, e um empréstimo partilhado do plano que o laço escreve por `&mut`. A janela densa (D3) tem o tamanho da região, e o texel só lê o próprio índice. |

## O preço

- **Complexidade:** uma 5.ª família no journal (`VidroJournal`), uma 4.ª testemunha na elisão e uma porta nomeada
  nova. O commit e o undo usam as portas genéricas que já existem (`from_journal`, `StoredPlane::Patch`), sem motor
  de delta novo.
- **Custo por quadro:** a captura dos tiles novos da região (a 1.ª vez que o traço toca cada tile; ≤ 2 cópias por tile,
  a do passo e a da sessão) e a materialização da janela do `vbase` (rw×rh×7 B por quadro do laço). As duas escalam
  com a REGIÃO, não com a tela. Se a janela do `vbase` pesar no quadro, o passo 4 mede-a contra a captura da sessão
  feita uma vez.
- **Memória:** o journal do gesto aberto cresce com os tiles tocados. Um traço que cubra a tela inteira em tiles
  volta aos 58 MB por journal (dois, passo e sessão), o mesmo que hoje paga por traço, mas só nesse caso.
- **Risco:** um sítio de escrita do vidro que escape à porta perde a edição em silêncio no desfazer. A rede é dupla:
  a testemunha (`incomplete` ⇒ caminho lento) e o `vidro_audit`. Censo dos sítios que escrevem `vidros`: `grep -n
  "vidros\.\(get_mut\|insert\|remove\)"` (hoje: `abre_o_vidro`, `continua_o_vidro_do_traco`, `devolve_o_vidro`,
  composite Wet, duplicar camada, restauro do snapshot).
- **Determinismo:** nenhuma redução nova; a captura paralela por tile é a do `plane_copy` (ADR-0109: tiles disjuntos,
  leitura pura).

## Plano de implementação

Cada passo fecha verde por nome (`bash scripts/cargo-test-narrow.sh ph2d-tool-painter <nome>`). A suíte inteira é do
integrador. O A×B vive no MESMO processo: chave `thread_local` `VIDRO_PELO_JOURNAL` (`#[doc(hidden)]`, `true` por
omissão; `false` = o caminho de hoje, o oráculo).

0. **Medir antes (sem código de produto):** na sonda, `strong_count` do plano em `abre_o_vidro` no 2.º pen-down, por
   dono (`before`, cursor, congelado), e a partição dos 115 MB (cópia × faltas de leitura). Isto decide se D3 é
   necessário para o alvo ou só para a correcção. Comando (fila pesada):
   `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && PH2D_SONDA_SO="Watercolor Basic" PH2D_SONDA_TELA=3840x2160 PH2D_SONDA_PREPARO=1 PH2D_SONDA_PARTES=1 PH2D_SONDA_NOVA=1 bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_primeiro_traco -- --ignored --nocapture`
1. **D1 — o journal e a porta:** `VidroJournal` em `undo_window.rs` (+ `journal_bytes_retidos`), `fork_vidro` em
   `plane_fork.rs`, os três sítios a passar pela porta com a região do `declare_wrote` e os dois `get_mut().expect`
   → porta. Gates:
   - `o_journal_do_vidro_guarda_os_tiles_que_a_regiao_toca_e_so_a_primeira_vez` (unitário do journal, `[u8;7]`);
   - `o_traco_que_cruza_tiles_guarda_so_os_tiles_tocados` — diagonal que cruza as fronteiras de 128, um traço colado
     a `x = 127|128` e um no canto da tela (tile parcial): bytes retidos = `tiles × 114 688` (tiles parciais pelo
     tamanho real) e o antes reconstruído = o oráculo ao bit.
2. **D2 — a elisão:** `vidros_elided` em `ModelSnapshot`/`undo_elide.rs`; `snapshot_model_eliding_relief` e
   `set_cursor`; `continua_o_vidro_do_traco` pela `VistaDoAntes`; `vidro_audit`. Gates:
   - `o_segundo_traco_nao_copia_o_plano_do_vidro` — `COPIAS_DO_VIDRO == 0` no 2.º..8.º traço WB; controlo
     (`VIDRO_PELO_JOURNAL=false`) ≥ 1 por traço; mutação «o `before` segura o vidro» (não esvaziar o mapa) ⇒ vermelho;
   - `o_traco_digital_sobre_a_aguada_continua_o_vidro_lendo_o_journal` — Digital por cima de uma aguada, vidro = oráculo
     ao bit.
3. **D4 — o commit pelo journal e o round-trip:** `StoredMap::from_journal` para `vidros` em `undo_planes.rs`, com o
   fallback de hoje quando o journal não descreve o passo. Gate principal:
   - `o_vidro_do_traco_volta_ao_bit_no_desfazer_e_no_refazer` — WB em papel de cor (o vidro só conta aí), traços 1.º,
     2.º e N-ésimo (N = 8, sobrepostos e disjuntos). Em CADA passo de «desfaz tudo, refaz tudo, desfaz metade, pinta
     outro», o vidro E a tela comparam-se ao bit com o oráculo `VIDRO_PELO_JOURNAL=false` no mesmo processo, e os
     bytes da entrada ficam ≤ `2 × tiles × 114 688`. Mutações: «o undo escreve o `after`», «a janela sem o
     `intersect`» (deve continuar verde: superconjunto) e «a área do `fork_vidro` encolhida 1 tile» ⇒ vermelho.
4. **D3 — o vidro da base da sessão:** `BaseCongelada` com `TileJournal`, janela densa no `Quadro`, o congelado
   `molhada`. Gates:
   - `o_vidro_da_base_da_sessao_e_o_do_inicio_da_sessao_ao_bit` — sessão WB com Smudge ligado, três traços na mesma
     sessão: o `vbase` materializado = o plano do início da sessão (oráculo: cópia inteira tirada na congelação, só em
     teste); mutação «o `vbase` lê o journal do passo» ⇒ vermelho;
   - `o_vidro_segue_a_base_que_o_borrao_move_sem_copiar_o_plano` (existente, n.º 5) e
     `o_primeiro_traco_da_camada_nao_copia_o_vidro_que_nasce_zerado` (existente) continuam verdes.
5. **Wet:** gates
   - `o_composite_wet_sobre_aguada_volta_ao_bit_no_desfazer` — Wet por cima de uma camada com vidro, undo/redo ao bit
     contra o oráculo;
   - existentes que têm de continuar verdes: `o_desfazer_do_composite_wet_guarda_a_caixa_e_nao_a_tela`,
     `undo_os_gemeos_da_camada_adormecida_dao_o_mesmo_bit_em_qualquer_largura_do_pool`,
     `o_undo_e_o_redo_dao_o_bit_do_caminho_antigo`, `o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela`.
6. **Camada nova (fronteira D6):** gate `a_camada_nova_e_o_vidro_elidido_dao_o_bit` — dois traços WB na base, «+», dois
   traços na nova, um na base. Desfaz tudo até antes do «+» e refaz tudo: vidro e tela ao bit contra o oráculo. Grava
   (sem asserir, para o item 3) quantas cópias de plano o 1.º traço na base depois do «+» paga. Gate
   `duplicar_a_camada_com_vidro_e_pintar_uma_nao_muda_a_outra`.
7. **D5 — o tecto:** gate `o_tecto_do_desfazer_conta_o_journal_do_vidro` — `journal_bytes_retidos` com o gesto aberto
   = a soma dos tiles capturados (passo e sessão), e com o mesmo `max_bytes` a pilha retém ≥ os passos do oráculo.
8. **D7 — a tela:** censo dos `fork_canvas(.., None)` / `capture_canvas_arc` no gesto WB e Wet a 4K e caixa em cada
   um. Gate por sítio curado, no molde de `o_desfazer_da_aguada_guarda_a_caixa_e_nao_a_tela`.
9. **Fecho da onda:** os alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`,
   `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`): a lista tem de ser IDÊNTICA à de antes (77, handoff n.º 6 §5),
   porque este ADR não muda um píxel; clippy/fmt das crates tocadas 1×, as mutações acima, e a matriz do app
   (`bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel`), com o load anotado.

### A medição alvo

Mesma sonda do passo 0, binário único, `VIDRO_PELO_JOURNAL` intercalado (mínimo de ≥ 5 rodadas, load anotado; acima de
~5 o relógio é indicativo e conta mais a memória nova, que não depende da carga):

| 2.º pen-down WB 4K | hoje | alvo |
|---|---|---|
| `vidro_do_quadro` | 11,5 ms · 115 MB | **≤ 1 ms · ≤ `2 × tiles_da_caixa × 114 688 B`** (um dab de 256 px ≈ 9 tiles ≈ 2 MB); zero `par_clone` do plano |
| pen-down inteiro | 14,4–15,4 ms | ≈ 4 ms menos (o vidro sai; o resto é de outros donos) |
| N-ésimo traço (N = 8) | igual ao 2.º | igual ao alvo do 2.º: o custo deixa de depender do número de traços |

Para o dono, no smoke: no Watercolor Basic a 4K, o 2.º traço e os seguintes começam sem o engasgo que ainda havia
sobre papel de cor, e o Ctrl+Z tira exactamente um traço.

## Ficheiros que a implementação toca

`crates/ph2d-tool-painter/src/`: `undo_window.rs` (+ um `undo_window_vidro.rs` filho, se o tecto de LOC morder) ·
`tool/paint/plane_fork.rs` · `tool/paint/watercolor_vidro.rs` · `tool/paint/watercolor_render.rs` ·
`tool/paint/watercolor_render/texel.rs` · `tool/paint/wetpaint/composite.rs` · `undo.rs` (`ModelSnapshot`) ·
`undo_elide.rs` · `tool/layers/undo.rs` · `undo_record.rs` · `undo_planes.rs` · `undo_planes_inspect.rs` ·
`undo_confine.rs` (o braço `vidros: _`) · testes novos ao lado (`watercolor_vidro_journal_tests.rs`). Zero ficheiros em
`shells/`: o undo por DIFF da shell (`shells/desktop/src/undo.rs`, `ProjectState`) só encadeia a transação do Painter
no seu degrau (W8, `undo.rs:417`) e não segura os planos dele; o contrato da transação não muda.

## Implementado (13/11)

Estado e números: [`N7_ITEM2_vidro_tiles.md`](../../Painter/missao_4k/medicoes/N7_ITEM2_vidro_tiles.md). Chave
`VIDRO_PELO_JOURNAL` (`true`); `false` continua o oráculo dos portões.

**O que difere do plano**
- **D2:** o 2.º traço ainda copiava pela entrada `OnlyAfter` do 1.º; cura `StoredEntry::Aparece` (a janela fora do
  neutro troca o `Arc` vivo).
- **D3:** a sessão é um `TileJournal<Vidro>` na `BaseCongelada`, e o `vbase` uma janela densa da caixa do quadro.
  A guarda da sessão molhada passou a levar a CAMADA (`GuardaDaSessao`): a cópia da camada partilhava o `Arc` e
  herdava a sessão da original.
- **D5:** o journal do PASSO soma em `journal_bytes_retidos`; o da SESSÃO (`VidroCongelado::bytes`) soma em
  `bytes_do_instantaneo`, não em `journal_bytes_retidos` (não é do passo e não reseta com ele). O `journal_heap_bytes`
  (só em teste) continua tela+relevo, porque cinco gates o lêem como «os bytes da tela».
- **D7:** não houve sítio a curar. No gesto WB/Wet, os três `fork_canvas` (`watercolor_render.rs`,
  `wetpaint/composite.rs`, `borracha_morta.rs`) já passavam a caixa desde a n.º 6. Os `None` que restam (Fill,
  Sculpt, Inpaint, Region, Selection) estão fora do gesto, como o D7 previa.
- **Cura fora do plano:** sem `debug_assertions` (perfis `smoke` e `release`) a crate não compilava, porque
  `vidro_antes_na_janela` chamava `TileJournal::get`, só de debug. Passou a ler por corrida com `read_row_into`.

**Números medidos**
- Cópias do plano do vidro do 2.º ao 8.º traço: `[1;7]` → **`[0;7]`**. Donos do plano em repouso: 3/2 → **1/1**.
  No 1.º traço na base depois do «+»: oráculo 1, journal **0** (asserido).
- Gesto aberto (520×390): passo 688 128 B, sessão 917 504 B, plano 1 419 600 B. Com o mesmo `max_bytes`, a pilha
  retém 4 passos contra 4 do oráculo.
- Sonda a 4K, `ab` intercalado (r2, depois do D3, load 5–8, relógio só indicativo), 2.º pen-down WB: mínimo
  **11,8 ms pelo journal × 14,0 ms pela rota antiga**, média 13,6 × 17,2. O alvo de ≈ 4 ms a menos no pen-down
  inteiro fica perto (−2,2 ms no mínimo, −3,6 ms na média). O alvo de ≤ 1 ms para o `vidro_do_quadro` sozinho está
  por ler nas partes da sonda.
- `medida_instantaneo_memoria` (retido ÷ declarado): 1,022 e 1,017.

**Aberto**
- **Tela:** `fork_par_raw` ainda copia a tela INTEIRA (33 MB a 4K) quando outro dono forte segura o `Arc`. A caixa
  só reduz o que o journal guarda, não essa cópia. O mesmo vale para `undo_vivo/casco.rs` (`make_mut` se houver
  pixel fora da caixa). Copiar só a caixa exige uma porta nova (fork por tile), que não cabe numa troca de uma
  linha. Custo estimado, NÃO medido: ≈ 3–5 ms por cópia a 4K, se ela acontecer no gesto (falta confirmar o `strong` da tela no pen-down; a tela elidida do B17.4 já larga o `before`). Medir com `PH2D_SONDA_PARTES=1`.
- O «+» e as entradas estruturais (D6, item 3).
- Os alarmes de pincel e a matriz do app (passo 9) ficam com o fecho da onda.
