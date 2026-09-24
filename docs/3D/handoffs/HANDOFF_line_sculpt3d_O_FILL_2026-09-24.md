# HANDOFF — `line/sculpt3d` · o `Fill Piece` (2026-09-24)

> Ordem do dono (24/09): *«1 e 2 no mesmo ciclo»* — a **1** é este botão (pintar a
> peça inteira com a cor do pincel, respeitando a máscara, desfeito com um
> `Ctrl+Z`) e a **2** é a memória que o `Even Detail` deixou reservada, que fechou
> no commit anterior (handoff da jornada, §33).
>
> ⚠️ **Handoff próprio, e não o §34 do da jornada:** aquele tem `205 KB`, o dobro
> do joelho medido em que um `Read` deixa de o alcançar (CLAUDE.md §5.0). O da
> jornada leva UMA linha a apontar para aqui.
>
> **Contadores:** `PROJECT_SCHEMA` 0 · os três registos 0 · `SCULPT_DOC_VERSION` 0 ·
> zero contrato · zero ADR · zero dependência nova. `Sculpt3dIntent` e
> `StrokeUndo` ganham UMA variante cada (append-only nos dois).

## §1 — O que o artista faz agora

Com o pincel de pintura na mão, logo abaixo da caixa de cor há um botão
**`Fill Piece`**. Carregar nele pinta a peça activa inteira com a cor da caixa:

* a **máscara** é respeitada, com a MESMA conta que o pincel faz — mascarado fica,
  livre pinta, a meio pinta a meio;
* com a **tinta fina** armada, o plano de amostras é pintado também, e o prefixo
  por vértice sai ao bit o que a cor por vértice recebe;
* **um** `Ctrl+Z` desfaz tudo, nos dois canais, ao bit; o `Ctrl+Shift+Z` refaz.

Passos no roteiro das cenas `=51` (passos 8–10: pintar tudo · pintar tudo menos a
mancha mascarada · desfazer de uma vez) e `=52` (passo 10: o `Ctrl+Z` devolve a
marca FINA, não grossa).

## §2 — ⭐⭐ Porque é UMA função e não uma lei nova

O carimbo do pincel já responde *«quanto desta amostra a máscara deixa
pintar?»* — o `keep` do `tinta_fina::apanha` (a soma dos pesos da retícula contra
a máscara dos cantos, e o `free_weight` por cima). Escrever essa conta outra vez
para o `Fill` seria a **segunda resposta à mesma pergunta**, e as duas divergem no
dia em que alguém mexer numa.

⇒ `ph2d_sculpt3d::preenche::keep_da_amostra` é a PORTA, e os dois chamadores são o
carimbo e o preenchimento. O carimbo passou a chamá-la **com a mesma aritmética
que fazia em linha** (o mesmo `zip`/`sum`), logo nenhum corpus de oráculo se move;
e um censo de texto (`o_carimbo_do_pincel_pergunta_a_mesma_porta`) reprova a
recaída — a conta a voltar a ser feita em linha é inobservável a toda régua de
valor.

## §3 — As decisões, com o porquê

| decisão | porquê |
|---|---|
| mistura `a·(1−k) + c·k`, **não** `a + (c−a)·k` | a segunda devolve `a + (c − a)` com `k = 1`, que em `f32` não é `c`; a primeira é **exacta nas duas pontas** (gate `a_mistura_e_exacta_nas_duas_pontas`) |
| os **vértices** decididos antes das faces, pela chamada de um só peso | a amostra de um vértice é o próprio índice dele (`de_vertice`), logo o prefixo por vértice do plano sai **ao bit** o preenchimento por vértice; e cobre o vértice que nenhuma face alcança |
| uma amostra partilhada é decidida pela **primeira** face | os pesos baricêntricos e os bilineares sobre a MESMA aresta podem diferir no último bit (`i/l` contra `1 − j/l`); um carimbo em ordem de face é a regra que o pincel já segue |
| recusa `NaoDescreve` antes de uma escrita, pelas contagens **e** pela forma de cada face | a lei do §14 da tinta fina: faces a mais estouram, faces a menos (ou de outra forma) escrevem tinta válida no sítio errado, em silêncio |
| a entrada de desfazer é de **peça inteira** (`StrokeUndo::Fill`, irmã da `Mask`) | o gesto age na peça toda por definição, e uma janela seria uma mentira sobre o que mudou — a razão que o `mask_op` já escreve |
| o plano de antes é um `PlanoInteiro` **sem índices**, trocado no sítio (`swap_with_slice`) | uma janela `0..n` pagaria `4 B` por amostra para dizer «todas», e a `16x` o plano são `~300 MB`: uma cópia pediria outro tanto de pico |
| a cerca do plano é a mesma `IdDoPlano` **mais a contagem** | a troca é de fatias inteiras, e fatias de tamanhos diferentes não se trocam — aqui a contagem não é redundante |
| nada mudou ⇒ **nenhuma** entrada, e um plano de cor materializado à toa é devolvido | um passo de `Ctrl+Z` sobre um gesto que não mudou nada gasta o passo ANTERIOR |
| a meio de um traço, **recusa** (`TracoAberto`) | o plano está emprestado ao gesto; preencher por baixo dele deixaria o traço a devolver um plano antigo por cima do preenchido. O botão não é alcançável com o dedo em baixo — a recusa é a rede |
| a força do pincel **não** entra | o gesto é *«esta peça fica desta cor»*; a força gradua traços, e um `Fill` a meia força seria um véu que o artista não pediu. ⚠️ **Divergência declarada**, e decisão do dono se ele quiser a outra |

## §4 — ⭐ O nome: `Fill Piece`, e não `Fill`

O PINCEL que enche covas já se chama `Fill` (`sculpt3d.verb.fill`) e vive na
fileira de ferramentas do MESMO painel. Dois controlos com o mesmo nome e gestos
diferentes mandariam o artista ao errado — ele leria «`Fill`» no roteiro e podia
carregar no chip do pincel. O nome diz o SUJEITO (a peça), como a secção `Pieces`
do mesmo painel.

## §5 — ⛔⛔ Um defeito de TECTO achado pelo caminho

O braço do traço no `StrokeUndo::footprint_bytes` terminava num `..`, que
**engolia o campo `finas`**: um traço de tinta fina punha na fila `16 B` por
amostra tocada que a poda não via. O cabeçalho do mesmo `match` diz-se *«EXAUSTIVO
de propósito»* — e o `..` dentro de um braço anula essa exaustividade campo a
campo. ⇒ `JanelaFina::bytes`, o `finas` contado, e o `level` nomeado (`level: _`)
em vez de engolido. Gate `o_tecto_da_historia_conta_a_janela_fina`: a diferença
entre a mesma entrada com e sem a janela é **exactamente** o peso dela.

## §6 — ⛔ E um defeito de INSTRUMENTO

Dois arneses de mutação (`muta_o_r_por_face.sh`, `muta_a_tinta_que_sai.sh`)
filtravam o gate da fiação pelo **número** no nome dele — `vinte_e_quatro` e
`vinte_e_tres`. O segundo **já não casava nada** desde uma renomeação anterior, e
*um filtro que casa zero lê-se, num placar, como uma mutação que sobreviveu*
(aqui, pior: as mutações observadas só por aquele gate passavam a sangrar ou a
sobreviver por outro motivo). ⇒ o gate passa a chamar-se
`a_cura_da_tinta_fina_esta_ligada_nos_sitios_todos`, o piso (`28`) vive DENTRO
dele, e os dois arneses filtram por `esta_ligada_nos`.

## §7 — Gates

| onde | gate | o que afirma |
|---|---|---|
| `ph2d-sculpt3d/preenche_tests.rs` | `sem_mascara_toda_amostra_sai_exactamente_a_cor_do_pincel` | livre ⇒ `c` exacta, nos dois canais |
| 〃 | `toda_mascarada_nada_muda_e_as_portas_o_dizem` | e as portas devolvem `false` |
| 〃 | `meia_mascara_da_meia_cor` | com o controlo de que é um ponto do MEIO |
| 〃 | `os_vertices_do_plano_sao_ao_bit_o_preenchimento_por_vertice` | com uma máscara que VARIA |
| 〃 | `o_centro_de_um_quad_le_a_mascara_interpolada` | `keep = ¾` no centro de um quad com um canto mascarado |
| 〃 | `um_plano_de_outra_malha_e_recusado_sem_escrita` | outra malha inteira |
| 〃 | `um_plano_com_faces_a_menos_e_recusado` | a metade das CONTAGENS, isolada (o prefixo da malha) |
| 〃 | `um_plano_com_as_mesmas_contagens_e_outra_forma_e_recusado` | a metade da FORMA (um quad lido como triângulo) |
| 〃 | `um_vertice_que_nenhuma_face_alcanca_tambem_e_pintado` | a razão da passagem dos vértices |
| 〃 | `a_mistura_e_exacta_nas_duas_pontas` | |
| 〃 | `o_carimbo_do_pincel_pergunta_a_mesma_porta` | o elo, por texto |
| `ph2d-app-sculpt3d/history_tinta_fina_tests.rs` | `a_troca_do_plano_inteiro_e_involutiva` | desfazer e refazer ao bit |
| 〃 | `um_plano_inteiro_de_outro_degrau_e_largado` | e sem plano também |
| 〃 | `o_tecto_da_historia_conta_a_janela_fina` | §5 |
| 〃 | `a_entrada_do_fill_pesa_os_dois_planos` | |
| `ph2d-app-sculpt3d/tinta_no_produto_fill.rs` (placa) | `o_fill_pinta_os_dois_canais_e_o_ctrl_z_devolve_os_dois` | pelo caminho do produto, com o plano armado como CONTROLO |
| 〃 | `a_meio_de_um_traco_o_fill_recusa` | |
| 〃 | `com_a_peca_ja_pintada_o_ctrl_z_devolve_a_cor_de_antes_do_fill` | o ramo que DEVOLVE a cor (§8.1, `A4`) |
| `ph2d-app-sculpt3d/tinta_fiacao_tests.rs` | 4 elos `F1`–`F4` | os que a placa prova e o CI não corre |
| `ph2d-panel-sculpt3d/tests/it/seam_cor.rs` | `o_fill_esta_colado_a_caixa_e_o_dedo_chega_a_shell` | pintado · dono dos pixels · o clique real chega como UM `ColorFill` |
| 〃 | `um_pincel_que_puxa_a_cor_do_anel_nao_mostra_o_fill` | a mesma cerca da caixa |
| `ph2d-panel-sculpt3d/tests/it/seam.rs` | `every_command_reaches_the_shell` | `24 → 25` comandos |

## §8 — Placar e portão

### §8.1 — A prova de mutação (`docs/3D/ferramentas/muta_o_fill.sh`)

**`16 de 17`**, e a 17.ª é o `C0`, o CONTROLO inerte. ⛔⛔ **A 1.ª corrida deu
`13 de 17`, e as TRÊS sobreviventes eram FIXTURAS minhas — nenhuma era a lei:**

| mutação | porque sobreviveu | a fixtura que a matou |
|---|---|---|
| `L2` a mistura ingénua | ⭐ **a mutação estava no canal VERMELHO, e com `c = 0,9` a forma `a + (c − a)` é EXACTA para todo `a` em `[0, 1]`** — medido, `0` de `999` valores (a subtracção é exacta pelo lema de Sterbenz na maior parte do intervalo, e o resto arredonda de volta). Com `c = 0,1` ela erra em `783` de `999` | a mutação passou para o canal VERDE; o gate diz porquê |
| `L4` a recusa por CONTAGENS | a fixtura dela (`uv_sphere(6,12)` contra `(8,12)`) era apanhada primeiro pela pergunta da FORMA — *duas cercas que se tapam uma à outra leem-se como uma cerca a funcionar* | `um_plano_com_faces_a_menos_e_recusado`: o PREFIXO da malha, onde toda face tem a forma certa e só a contagem pode recusar |
| `A4` o desfazer da cor por vértice | a peça do gate de produto **nunca tinha tido cor**, logo só o ramo *«não havia cor, e desfazer TIRA o plano»* era exercido | `com_a_peca_ja_pintada_o_ctrl_z_devolve_a_cor_de_antes_do_fill`: um traço real pinta primeiro, e o CONTROLO é a cor existir antes |

⚠️ O arnês controla-se nos QUATRO pontos dos irmãos (âncora única · a mutação
compila · `N > 0` testes correram · a corrida limpa está VERDE), e o pré-voo
dos **12** arneses da linha lê **172 âncoras, todas a casar uma vez**.

### §8.2 — Portão

| régua | resultado |
|---|---|
| `nextest-impacted` | **18 641 / 18 642** — o vermelho é `the_cost_of_a_player_is_linear_in_their_number`, membro CONFIRMADO da família de flakes de carga (CLAUDE.md §5.0): `3 de 3` verde sozinho a `load 65–67`, e **zero** linhas deste diff em `ph2d-physics-ecs` |
| gates de placa (`--run-ignored`, filtro `tinta\|preenche\|fill\|undo\|ctrl_z`) | os `46` que não são de retopologia verdes, os `17` da tinta e do `Fill` entre eles. ⚠️ O filtro `undo` apanhou também `history::undo::{global_retopo,quad_shape,simplest_case,…}` (os testes vivem num módulo com esse nome): `4` vermelhos **deterministas e conhecidos** (o `27°` da orelha contra os `5–6°` do oráculo é à letra o gate que o §5 regista) e `5` estouros do tecto de `180 s` sob carga — **zero** linhas deste diff tocam retopologia |
| clippy `-D warnings` (as quatro crates, `--all-targets`) | zero — depois de dois avisos meus (`needless_range_loop`, `manual_slice_size_calculation`) |
| censos da árvore COMBINADA | **127 / 127**, controlo do filtro `12 de 12` |
| as 10 vassouras sobre os `30` ficheiros do diff | **zero achados NOVOS** — os acusados são linhas antigas (o `tip_roundness` do i18n e do handoff da jornada, o `layer.cc` do índice), nenhuma escrita aqui |
| pré-voo dos 12 arneses | **172 / 172** âncoras |
| maior ficheiro tocado | `history.rs` a **679** de `700` |
| `fmt` | limpo |

## §9 — Aberto

* ⏳ **A força do pincel** não entra (§3, última linha) — decisão do dono.
* ⏳ **O custo a `16x`:** a entrada guarda `~300 MB`; o tecto em bytes poda a
  história antiga para a caber. Medido só no plano; o relógio do preenchimento a
  `16x` **não foi varrido** (é um gesto por clique, não por quadro).
* ⏳ O `Fill` age na peça **activa**; não há «todas as peças».
