---
name: topic-gate-discipline
description: "Família: ofício de gate — pares ausência/presença, oráculos honestos, paridade numérica, fixtures que contêm o fenômeno"
metadata: 
  node_type: memory
  type: reference
  originSessionId: 85e38f84-1b86-49d2-aee2-91da101e1fd7
  modified: 2026-09-12T22:45:38.485Z
---

# Ofício de gate (índice de família — detalhe em cada arquivo; irmãos: mutation_proofs · oracle_discipline · fixture_discipline)

- ⛔⛔ **Um gate que se DECLARA «uma família, não um sítio» e é uma LISTA ESCRITA À MÃO não tem população para ter piso** — medido 15/09: ele nomeava 6 ficheiros, o censo derivado varreu 18 e achou mais **5** desenhadores com o mesmo defeito, um deles a forma que o próprio roteiro de smoke mandava arrastar. ⚠️ E a 1.ª agulha do censo novo nomeava o CONSTRUTOR (`X::new(`) e leu `7` de `8`: quem RECEBE a porta por parâmetro não a constrói — *a agulha tem de nomear a CONSULTA*.
- [[feedback_absence_gate_needs_a_presence_sibling]] — gate de AUSÊNCIA precisa do de PRESENÇA
- [[feedback_layered_defenses_need_per_layer_gates]] — defesa em camadas = gate POR camada
- [[feedback_a_threshold_must_live_where_the_domain_is_empty]] — limiar mora onde o domínio é VAZIO
- [[feedback_a_ratio_between_two_sick_channels_is_green_by_construction]] — razão entre dois canais doentes é verde; ancore no que o produto promete
- [[feedback_a_ruler_that_deduplicates_cannot_report_duplication]] — regua que DEDUPLICA nao denuncia duplicacao; conte por OCORRENCIA
- [[feedback_a_suite_of_topological_assertions_is_blind_to_geometry]] — conte quantas assercoes olham uma COORDENADA; se zero, a suite e cega
- [[feedback_a_polyline_on_a_mesh_turns_where_the_structure_does_not]] — teste geometrico sobre curva discreta conta o zigue-zague como estrutura
- [[feedback_a_round_that_never_reports_its_residual_is_a_silent_lie]] — `round` sem residuo esconde erro de FORMULA; instrumente as perguntas todas de uma vez
- [[feedback_a_conserved_invariant_cannot_grade_quality]] — invariante CONSERVADA e' verde por construcao; a regua e' a CONTAGEM
- [[feedback_a_green_gate_may_be_green_by_accident]] — verde de 1ª pode ser acidente; suspeite do fixture
- [[feedback_a_fixture_can_land_in_a_chaotic_regime]] — Δ enorme? compare com quantidade física
- [[feedback_an_arch_gate_anchored_on_a_file_fails_when_the_loc_cap_moves_the_code]] — arch-gate ancorado num ARQUIVO morre no corte de LOC; leia a FAMÍLIA por porta única
- [[feedback_a_seam_fixture_must_rest_on_something_uncoverable]] — fixture de costura pede âncora incobrível
- [[feedback_moving_the_law_is_half_the_fix_the_fixture_must_contain_it]] — mover a lei é metade; a fixture tem de conter o fenômeno
- [[feedback_comparing_two_routes_requires_the_same_art]] — comparar duas rotas exige a MESMA arte
- [[feedback_the_approved_reference_may_already_be_in_the_product]] — a referência aprovada pode já estar no produto (oráculo de graça)
- [[feedback_a_gate_red_on_your_correct_code_may_predate_you]] — gate vermelho pode ser herdado; rode contra o HEAD shipado
- [[feedback_seam_gates_need_fractional_advance]] — gate de emenda precisa de frac (1:1 nunca lê o 2º frame)
- [[feedback_wide_mechanical_refactor_use_a_fingerprint]] — refactor mecânico = impressão digital, não golden pinado
- [[feedback_frozen_bar_check_the_arithmetic_before_gaming_it]] — barra congelada? o piso pode já estourar a barra
- [[feedback_a_cache_key_must_key_on_what_varies_the_artifact]] — chave de cache keya no que VARIA o artefato
- [[feedback_determinism_sweep_grep_all_transcendentals]] — determinism sweep: grepe todos os transcendentais
- [[feedback_same_math_different_bookkeeping_diverges]] — mesma conta, escrituração diferente = 1 ulp
- [[feedback_cpu_gpu_rounding_conventions_diverge]] — round CPU (half-away) ≠ GPU (half-even)
- [[feedback_ask_the_same_question_of_the_other_side]] — faça a MESMA pergunta ao outro lado; o gêmeo nasceu vermelho
- [[feedback_test_with_product_numbers_not_convenient_ones]] — números do PRODUTO; `1.0` esconde erro de unidade
- [[feedback_a_rule_that_never_observes_cannot_fire]] — regra que não OBSERVA não dispara (HR-13, 4351 MB)
- [[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]] — arch-gate afirma relação posicional, nunca distância/janela no fonte
- [[feedback_a_capability_without_a_door_passes_every_gate]] — grepe quem ESCREVE o campo, nao quem le
- [[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]] — rota A==rota B só prova o walker; o corpo quer oráculo externo
- [[feedback_a_silenced_instrument_reads_as_a_result]] — zero ≠ não-medido; gateie a presença de cada balde
- [[feedback_a_doc_comment_naming_a_cfg_expires_grep_the_attribute]] — grepe o atributo; e `cargo test --release` LIGA `cfg(test)`
- [[feedback_green_composed_gates_can_hide_an_unproven_connector]] — feature "sem efeito" + gates verdes ≠ percepção; dirija o CLIQUE real
- [[feedback_a_ratio_cannot_rescue_a_max_based_oracle]] — ruído aditivo só no numerador; propriedade estrutural = oráculo no FONTE
- [[feedback_a_gate_that_waits_a_fixed_duration_bets_on_machine_speed]] — vire CONDIÇÃO; se só o outro lado carimba, a espera DIRIGE o produto
- [[feedback_an_ignored_sweep_is_not_the_gpu_gate_sweep]] — ela roda placeholders `unimplemented!()` e sondas lentas junto; rode os NOMES do handoff, por crate
- [[feedback_dropping_ownership_blinds_every_comparator_that_reads_that_side]] — elidir um dado faz o detector que o lê responder "mudou" para sempre
- [[feedback_a_correct_number_can_carry_a_false_story]] — um número medido corretamente pode sustentar uma explicação ERRADA: o gate fica verde e a afirmação do produto é falsa
- [[feedback_a_probe_that_sums_two_signals_cannot_say_which_failed]] — sonda AGREGADA (tinta da cena, soma, hash) fica verde enquanto qualquer emissor funcionar; conte os emissores antes de a escrever
- [[feedback_a_slack_term_in_a_ceiling_is_the_size_of_the_blind_spot]] — a folga num tecto É o tamanho do ponto cego; meça a DIFERENÇA, não o tecto
- [[feedback_a_cost_only_defect_is_invisible_to_every_output_gate]] — defeito só de CUSTO não move nenhuma saída: contador no produto, lido num binário só
- [[feedback_a_test_that_got_slow_is_a_cost_measurement_nobody_asked_for]] — um teste que ficou lento É uma medição de custo; leia-a em vez de a tolerar
- [[feedback_a_loop_that_republishes_the_whole_object_every_round_pays_for_what_it_never_reads]] — laço que republica o objecto inteiro paga o que nunca lê
- [[feedback_a_gate_on_the_reported_quantity_is_green_when_the_product_divides_by_the_other]] — gate sobre o que o relatório PUBLICA ≠ o que o produto DIVIDE; derive do efeito
- [[feedback_a_denominator_above_the_curvature_is_slow_below_is_inf]] — denominador acima da curvatura é lento; abaixo é `inf`
- [[feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing]] — uma afirmação que mutação nenhuma mata é uma afirmação sobre nada
- [[feedback_a_model_change_must_re_ask_what_every_gate_still_measures]] — mudar o modelo re-pergunta o que CADA gate ainda mede
- [[feedback_a_safety_claim_needs_its_fairness_half_or_a_conservative_mutation_survives]] — afirmação de segurança precisa da metade JUSTA, senão a mutação conservadora sobrevive
- [[feedback_counting_the_work_done_is_not_counting_the_work_delivered]] — FEITO ≠ ENTREGUE: a sonda vai no CONSUMIDOR, não no produtor
- [[feedback_a_gate_on_the_mark_i_chose_is_green_when_the_marks_premise_is_false]] — gate sobre a MARCA que escolhi ≠ gate sobre o FIM
- [[feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two]] — duas boas hipóteses a falhar refutam a FAMÍLIA, não as duas
- [[feedback_an_absence_gate_that_names_a_file_is_disarmed_in_silence]] — gate de AUSÊNCIA que nomeia um ficheiro fica verde-e-vazio depois de um corte; o de presença falha alto. Pergunte ao MÓDULO
- [[feedback_a_seam_gate_must_assert_both_sides_or_it_measures_the_wrong_half]] — «todo consumidor PERGUNTA» ≠ «alguém RESPONDE»: 87% da barra deixava passar o clique com o gate verde
- [[feedback_deleting_the_only_painter_leaves_every_registration_gate_green]] — tirar o único PINTOR de um controlo não move gate nenhum; registo e despacho continuam certos
- [[feedback_a_state_nobody_writes_and_someone_reads_is_an_if_with_a_dead_side]] — estado sem escritor: a marca em falta é o sintoma barato; o ramo morto de quem o lê é o caro
- [[feedback_a_symptom_remedy_is_cured_by_the_right_input_not_by_deletion]] — remédio-de-sintoma cura-se dando a ENTRADA certa: inerte onde a causa caiu, vivo onde ela fica (2 gates + controlo)

- [[feedback_a_textual_census_that_cannot_tell_prose_from_code_lies_both_ways]] — censo de fonte que não separa prosa de código acusa a prosa e absolve o código
- [[feedback_a_gate_that_presumes_the_destination_of_an_effect_accuses_the_living]] — censo que pergunta pelo DESTINO (o barramento) acusa de morto o vivo com outro destino, e a mensagem manda construir a doença
- [[feedback_a_census_that_shares_state_measures_the_previous_cases_side_effect]] — instância fresca por caso; e os acusados que sobram são os pontos cegos do ORÁCULO
- [[feedback_when_the_only_consumer_of_an_artefact_is_an_llm_reading_numbers_visual_defects_survive]] — artefacto cujo único leitor é a LLM carrega defeito VISUAL indefinidamente: escreva o gate da classe que o seu leitor não vê
- [[feedback_a_family_that_returns_none_in_a_census_has_its_declaration_unmeasured]] — família que faz `return None` num censo fica com as DECLARAÇÕES dela sem régua; o 1.º membro construível expõe o buraco (`1,0216` num defeito pré-existente)
- [[feedback_a_source_reader_that_ignores_char_literals_reads_the_file_inside_out]] — leitor de fonte que não conhece o literal de CARÁCTER lê `find('"')` como aspa a abrir e vira o ficheiro do avesso (438 publicados, 418 reais; o gate acusou o próprio comentário)
- ⛔⛔ **Uma sonda que ESCREVE o artefacto e depois o afere deixa no disco aquilo que reprovou.**
  O gerador de figuras de um tutorial (09/09) escrevia cada SVG dentro do laço de medição e só
  depois corria as asserções: a corrida de **mutação** escreveu duas figuras com o **mesmo md5**,
  falhou em voz alta — e as figuras ficaram lá. Nada no repositório dizia que o tutorial passara a
  mostrar duas vezes a mesma imagem debaixo de duas legendas diferentes; só um `md5sum` à mão as
  apanhou. ⇒ **medir, afirmar, e só então escrever.** *Uma sonda que falha não pode deixar o
  artefacto que ela reprovou.*
- ⭐ **E o gate dessa família tem DUAS metades, porque a régua natural é cega à segunda:** a
  dispersão (*«o campo morde?»*) lê o **mesmo número** para duas figuras que diferem só na FORMA —
  as duas têm um elemento no cheio e outro no vazio. A metade que falta é *«duas figuras não podem
  ser idênticas»*: se forem, um dos params que as separa não está a ser lido, e o tutorial ensina
  duas coisas com uma imagem só.
- ⛔⛔ **Um CONTROLO POSITIVO calibrado enquanto o leitor estava contaminado ENCODA a
  contaminação.** Um censo de cenas contava *«quantas publicam legenda»* lendo um **global do
  processo** que só é reescrito por quem publica — logo uma cena **muda** herdava a legenda da
  anterior e era contada. O piso do controlo foi escrito nesse mundo (`>= 25`). Ao limpar o
  global antes de cada montagem, a contagem caiu para **21** e o gate acusou… **a sua própria
  calibração**. ⚠️ Baixar o piso ali **não é afrouxar a barra**: `21` é o número verdadeiro, e os
  outros quatro eram ecos. ⇒ ao curar um leitor contaminado, **re-derive todo número que foi
  calibrado através dele**.
- ⛔⛔ **E uma TRAVA só exclui quem a TOMA.** Pôr o cadeado no leitor do global deixava treze
  outros sítios a escrever por cima dele. ⇒ **a trava mora na PORTA por onde se produz o estado**,
  e quem o produz passa por lá. ⚠️ E ela segura-se só durante a produção: a versão que a tomava à
  volta de uma varredura de 112 níveis levou a suíte de **72 s para 1 796 s** — *uma trava que
  protege mais do que o estado partilhado paga o preço de toda a gente*.
- ⚠️ **E o diagnóstico só apareceu quando LI A MENSAGEM.** Duas rondas foram gastas a assumir
  «corrida» (porque o teste passava isolado) sobre uma reprova que era **determinística** e dizia
  o número exacto no texto do `assert`. *Uma reprova que passa isolada pode ter mudado de causa
  entre as duas corridas.*
- ⭐ **Um gate num tamanho SÓ pode morar exactamente na zona em que o defeito some** (Pixel Lab
  W23, 12/09): `rotatedSize` fazia `ceil(w·|cos| + h·|sin|)` e `sin(π)` é `1,2e-16` — num 4×6 o
  resto some no arredondamento do float e o gate dizia «exacto»; varrido de 1×1 a 64×64, **11 292
  de 12 288** tamanhos saíam com uma linha e uma coluna a mais, e o nearest estragava 7 287 de
  8 000 giros de 90°. Irmão no mesmo dia: um corpus que amostra a célula de longe (9–16 amostras)
  é cego à regra que só vale numa LASCA dela — a mutação passou verde no oráculo, e uma lupa ×128
  sobre o canto a matou. ⇒ *varra o tamanho e a densidade; um exemplo escolhido afirma só o exemplo.*
- [[feedback-a-census-gate-that-scans-its-own-tree-counts-itself]] — censo que varre a árvore onde mora conta-se a si mesmo (dados da metade justa, mensagens); salte o próprio ficheiro antes de TODAS as contagens e prove o piso com `piso + 1`
- ⛔ **Uma cura que põe PISO numa grandeza cega o gate que media essa grandeza contra o MESMO número**
  (Pixel Lab W24, 13/09): o gate exigia palco ≥ 100 px (barra do vão medido 0·146); a cura de uma
  janela pequena pôs `minmax(100px, 1fr)` na linha do palco, e duas mutações que antes sangravam
  (a legenda sem teto) voltaram a SOBREVIVER — levavam o palco ao piso em vez de a 0. Quem separa
  «espremido» de «cabe» passou a ser uma propriedade SEM número (a página cabe sem rolar). ⇒ *depois
  de pôr um piso/tecto, re-corra as mutações que os gates DAQUELA grandeza matavam.* Irmão: uma mutação
  de «não quebra a fileira» sobreviveu a 1000 e a 760 px porque um filho quebrava por conta própria
  — a largura do gate sai de MEDIR a mutação (1º botão fora só < 580 px), nunca de um palpite.
- ⛔ **"Vazio" decidido DEPOIS de recortar à caixa do alvo esvazia os dois lados** (Pixel Lab W27, 13/09):
  a comparação `.fnt` × `.bdf` recortava a letra do `.bdf` à célula do `.fnt` e só então perguntava se a do
  `.fnt` estava vazia — as 27 letras exportadas com LARGURA ZERO viravam recorte vazio contra glifo vazio e
  contavam como IGUAIS. ⇒ *a pergunta "falta?" se faz no lado NÃO recortado; o recorte só entra na
  pergunta "é igual?"*, e a barra exige a população exata das ausentes (27, todas na faixa esperada).
- ⛔ **Um gate de RESTAURAÇÃO que reabre UMA vez é cego ao que a restauração faz com o PRÓXIMO passo**
  (Pixel Lab W28, 13/09): o gate do F5 usava a régua mais forte (o documento em CADA posição da fila) e a
  mutação que recomeçava os ids dos passos do 1 SOBREVIVEU — a colisão só aparece quando o artista
  CONTINUA depois de reabrir e reabre de novo (o banco ficava com o passo velho daquele número no lugar do
  novo). ⇒ *restaurar → agir → restaurar*. E as três sobreviventes marcadas «(medir)» eram três coisas
  diferentes: defeito real, correcto-mas-caro (reabrir regravava a fila de 64 MB) e EQUIVALENTE (a mesma
  guarda escrita em três lugares ⇒ uma porta só). Classifique cada uma antes de curar.
- ⛔⛔ [«Contador, logo imune à carga» é FALSO atrás de estado POR THREAD — o gate da superfórmula leu morno 0/4/8 sob fan-out e reprovou o ship sem Rust mudado; lei exacta numa pool de 1 thread, tecto estrutural `4×(threads+1)` no caminho do produto](feedback_a_counter_behind_a_per_thread_memo_depends_on_the_scheduler.md)

- ⛔ **Um gate que ESCREVE no fonte aquilo que ele próprio varre acusa-se a si próprio** (13/09, `line/UIUX`): o teste que separa duas metades de um vocabulário escrevia o prefixo da secção (`"panel.inspector.player."`) e o censo de chaves leu-o como **chave em uso** — o gate reprovou sobre si mesmo. Duas curas: derive o prefixo (`format!("{PREFIX}player.")`) e ensine a régua que uma chave **não acaba em ponto**. *A fixtura de um gate é produto para a régua dele.*

- ⛔ **Um marcador de isenção SEPARA-SE do que isenta quando algo reformata a linha** (13/09, `line/UIUX`): cinco `// LITERAL-PX-OK` do Inspector ficaram na linha do parêntese quando o `rustfmt` reflowou a chamada (o rótulo passou a `tr("…")`), o número ficou sozinho noutra, e o `no_magic_numeric` acusou cinco sítios que já estavam isentos havia meses. *Uma isenção presa a uma LINHA é uma isenção que o formatador pode apagar* — e só se vê no dia em que alguém reformata.

- ⛔⛔ **Um CENSO TEXTUAL e um SEAM DE GESTO medem coisas diferentes, e o nome do primeiro não avisa**
  (Tags W3c, 13/09): apontei duas mutações — *a row não é pintada* e *o controlo morre sob o dedo* — ao
  `every_registered_physics_component_has_a_ui_writer`, e **as duas sobreviveram**. Aquele gate varre a
  FONTE à procura do id e da edição escritos no painel; apagar o `hit_index.register` não muda uma linha
  do que ele lê. ⇒ *um componente pode ter «caminho de escrita na UI» e estar invisível na tela*. As duas
  perguntas precisam de dois gates: o censo prende a FIAÇÃO, o seam com ponteiro real prende a PINTURA e
  a focabilidade. ⚠️ E o seam de um selector tem de **ABRIR a lista** antes de procurar as opções — elas
  só são pintadas pelo passe diferido enquanto o popover está aberto, e um gate que as procura com a
  caixa fechada mede um ecrã onde elas legitimamente não estão.
- ⛔⛔ **Uma linha que nenhuma mutação consegue observar é código morto com cara de rigor** (mesma volta):
  escrevi no `signal_passes` um `if tree.get(q).is_none() { return false }` para *declarar* a falha
  fechada, e a mutação que o apagava não fez nada reprovar — o `TagTree::reaches` (W1) já recusa um id
  fora da árvore, logo o guarda nunca era o que decidia. ⇒ apague-o e **NOMEIE a dependência**: a lei
  passou a ter as duas metades escritas (`a_tag_that_no_longer_exists_reaches_nobody` a montante,
  `a_missing_filter_tag_passes_nobody` aqui), porque se o `reaches` mudar, a armadilha abre em silêncio.
  ⚠️ É o INVERSO de [[feedback_i_write_the_right_guard_and_do_not_gate_it]]: ali escrevo a guarda certa e
  não a gateio; aqui escrevo uma guarda que outra porta já garantia.
- ⛔⛔ **Nenhum portão perguntava se um GESTO é REVERSÍVEL** — 88 verdes sobre um defeito de 155 cm ([[feedback_a_gesture_that_returns_must_give_back_the_pose]]). Todo gesto contínuo precisa do portão do caminho fechado.
- ⚠️ **Subir a barra de um portão pode ser uma TROCA MEDIDA e não um afrouxamento** — mas só se a troca estiver escrita AO LADO da barra, com os dois números: aqui a virada por movimento subiu de 11,1° para 15,4° (barra 15 → 18) e comprou 14 dos 16 círculos que deixavam o corpo fora do lugar. *Sem os dois números ao lado, é armengo.*
- ⛔ **Um controlo que nunca APERTA não segura a barra** (16/09): o «arco de `150°` numa cúbica tem de ser recusado» (erra `22×` a barra) deixou passar uma barra `10×` mais larga; o de `95°` (`1,38×`) mata-a. *O controlo negativo tem de estar logo acima da barra, não longe dela.*
- ⛔⛔ **Gate verde porque NENHUM teste corre com a env do smoke** (16/09, modelador): «fechar o painel desarma» estava provado no caminho do pill, e o caminho `PH2D_*_SMOKE=<n>` — o de todo passo de smoke — armava sem olhar o painel. `set_var` é `unsafe` e o `cargo test` partilha o processo, então a env nunca entra no corpus. ⇒ ler a env por uma porta com sobreposição POR THREAD só nos testes, e a lei pura com a env como argumento.
- ⛔⛔ **Um gate que mede a DECISÃO é cego ao que a TINTA faz com ela.** Medido 2026-09-14
  (`line/UIUX`): três gates defendiam a coluna do rótulo — dois comparavam `prefix_width > coluna`
  e um media a **porta** da decisão (`property_label_origin`) — e os três estavam **verdes** sobre
  uma foto do dono com seis rótulos cortados. A decisão estava certa em 3 600 células; quem errava
  era um **segundo corte dentro do pintor**, que nenhuma das três réguas alcançava. ⇒ quando a
  queixa é sobre o que se VÊ, pelo menos um gate tem de ler a **cena emitida** — aqui a contagem de
  **glifos** (⛔ `n_paths`/`n_path_segments` dão zero: o Vello encaminha texto por `draw_glyphs`,
  lição já paga duas vezes neste repo). ⚠️ E ele precisa de **controlo**: um contador de glifos que
  nunca vê uma reticência passaria também sobre um pintor que não pinta nada — o gate irmão exige
  que a contagem **mude** com a coluna a 60 %.
- ⭐⭐ **Um DOC pode ser gateado, e a mutação que importa é o PARSER a partir-se.** Medido 2026-09-14
  (`line/UIUX`): a spec da linha de propriedade fecha com uma tabela `lei → porta → gate`, e um teste
  extrai as duas últimas colunas e exige que cada nome exista no código — assim um `rename` reprova
  em vez de deixar o doc a prometer o que o app já não faz (*um doc que enuncia a lei que o código
  não implementa lê-se como AUDITADO*). ⚠️ **Das três mutações, a que ensina é a terceira:** partir
  UMA linha da tabela para o parser deixar de a achar — *um parser partido devolve «zero fantasmas»
  sobre uma tabela inteira por verificar* ⇒ **piso de população no próprio gate**, mais um controlo
  que exige que o detector recuse um nome inventado **e** aceite um real (senão ele aceita tudo ou
  recusa tudo). ⛔ E a lista de formas em que um nome nasce tem de incluir `as NOME`: uma porta
  re-exportada (`MIN_W_PX as NUMBER_INPUT_MIN_W_PX`) era acusada de não existir.
- ⛔⛔ **Uma lei curada numa SECÇÃO e não gateada é uma lei que a secção seguinte não conhece.**
  Medido 2026-09-15 (`line/UIUX`): a unidade saíra do rótulo para dentro do campo numa secção do
  Inspector (32 rótulos, `20 de 39` cortados → `1`), e **28** rótulos do resto do app continuavam a
  carregá-la — `"Break Torque (N.m)"`, `"Init Vel X (m/s)"`, `"Non-Spatialized Radius (m)"`. ⇒ *ao
  curar uma lei numa superfície, o mesmo commit escreve o CENSO que a cobra em todas* — e escrevê-lo
  **red-first**, com a tolerância a conter só o que não se vai converter, transforma-o na lista de
  trabalho. ⚠️ **E o piso de população tem de ser MEDIDO**: chutei `> 2000` sobre uma varredura que lê
  `1 801` e o gate reprovou sobre o produto certo. ⚠️⚠️ **E a RAZÃO de cada tolerância também se
  mede:** escrevi *«row de dois campos»* e a medição desmentiu — a porta `field_row` pinta uma row de
  UM campo e também não levava sufixo. *O que separa não é a forma da row, é a PORTA ter por onde a
  coisa entrar.*
- ⛔⛔⛔ **Um censo que ENUMERA as portas pelo nome acusa, a cada wave, exactamente quem fez a coisa
  certa.** Medido 2026-09-15 (`line/UIUX`, terceira ocorrência do MESMO gate): o
  `every_form_row_reserves_the_animation_column` listava `form_row_columns`; quando a linha de
  propriedade nasceu ele acusou quem a adoptou e a lista passou a duas; quando três secções passaram
  a chamar uma porta de 2.ª ordem (`rows::property_label_row`, que chama a de base por dentro) ele
  acusou as três. ⇒ **a lista deriva-se do produto**: é porta toda `fn` do ficheiro de rows cujo
  corpo alcança uma de base — **por PONTO FIXO**, porque há portas a DOIS saltos (uma `num_row` que
  delega na `num_row_unit` não nomeia porta de base nenhuma). ⭐ E o modo de falha fica **alto** de
  propósito: um parse morto encolhe a lista para as de base e o gate passa a acusar quem usa o
  ficheiro — vermelho em voz alta, nunca verde a medir nada. ⚠️ Mesmo assim leva controlo com **piso
  de população** e nomes esperados, senão «derivar» e «devolver tudo» leem-se igual.
- ⛔⛔ **Uma PROVA DE MUTAÇÃO que não casa imprime `test result: ok`, e isso lê-se como «o gate não
  apanha».** Medido 2026-09-15: a agulha da mutação tinha o escape errado depois de um `cargo fmt`
  ter reindentado a linha, casou **zero** vezes, e a corrida saiu verde sobre o código INTACTO —
  eu ia arquivar o gate como fraco. **Quem o apanhou foi o `assert` de contagem no script.** ⇒ *toda
  mutação escrita à mão assere quantas vezes a agulha casou, ANTES de correr o teste* (irmã da lição
  «um filtro que casa ZERO imprime SOBREVIVEU»).

---

## ⛔⛔ Um censo que identifica o sujeito pela FORMA acusa quem tiver a mesma forma (2026-09-15)

O gate do manual da linha de propriedade procurava a tabela `lei → porta → gate` por **forma**:
*qualquer* linha de markdown com quatro colunas cujas duas últimas começam em crase. No dia em que a
§6 do mesmo documento ganhou uma **tabela de medição** com quatro colunas, ele exigiu que
`` `90,0` (a metade) `` fosse uma porta declarada no código — *certo sobre o texto, errado sobre
onde olhar*.

⇒ **um censo recorta o sujeito pelo ENDEREÇO** (aqui: o texto entre `## §9 ` e o `## ` seguinte), e
mantém o **piso de população** para que perder o endereço falhe alto em vez de medir zero. É a mesma
lei que o `CLAUDE.md` §5.0 escreve para o censo que varre um directório por prefixo de nome — ali
ele passa a varrer **zero** e fica verde; aqui ele varre a **mais** e acusa um inocente. *As duas
metades do mesmo erro.*

⭐ Depois do endereço o gate ficou **mais forte**, não mais fraco: uma tabela nova noutra secção
deixa de o partir, e uma §9 renomeada parte-o em voz alta (prova de mutação feita).

---

## ⛔⛔ Uma cerca que presume a FORMA do código de teste é cega à outra forma (2026-09-15)

Dois gates do `ph2d-editor-core` definem *«código de teste»* como **`#[cfg(test)]` no fim do
ficheiro** — e o repo tem **duas** formas: o `mod tests` inline e o ficheiro IRMÃO
(`number_input/tests.rs`, `text_input/tests.rs`). No dia em que o tecto de LOC obrigou a mudar o
`mod tests` do `checkbox/mark.rs` para `checkbox/mark_tests.rs`, os dois acordaram:

- o `no_widget_paints_a_control_body_with_the_card_token` passou a **acusar o teste** (ele cita o
  token dos dois lados, de propósito);
- o `hr12_widgets_a11y` passou a **acusar o produto** — o `mark.rs` era verde só porque o `mod
  tests` dele tinha um `use ph2d_a11y::NodeId`. *Ele estava a ser satisfeito por código de teste.*

⭐ **Os dois são a mesma falha com o sinal trocado**, e a cura é uma: *«o que é código de teste»
responde-se UMA vez e cobre as duas formas* (`base == "tests.rs" || base.ends_with("_tests.rs")`,
mais o `#[cfg(test)]`).

⚠️ E a segunda metade é a lição mais cara: **um bloco de teste que muda de FORMA sem mudar de
natureza acorda todo gate cuja cerca era a forma**. Quem parte um ficheiro pelo tecto de LOC corre
os censos da crate antes de dar o corte por fechado.

- ⛔⛔ **Um censo POR CRATE é cego ao texto que a crate PINTA mas não ESCREVE** (2026-09-16, `line/UIUX`):
  o `every_word_this_panel_shows_comes_from_the_string_table` do painel Painter estava a ZERO com o
  painel a pintar ~90 textos em inglês abreviado (`Shad Amt`, `Preserve Lum.`, os 24 nomes do menu
  «+ Adjustment») — os literais moravam na `ph2d-painter-effects`, do outro lado da fronteira, e
  chegavam ao pintor por uma função. **Why:** a régua lê o `src/` de UMA crate; o texto atravessa a
  fronteira como valor. **How to apply:** quando um painel pinta o retorno de uma função de OUTRA crate
  (`*_params`, `display_name`), trate esse retorno como IDENTIFICADOR — um `match` que o mapeia para a
  chave (a régua isenta braços de `match`) — e gateie a tabela nos dois sentidos (todo rótulo tem chave)
  MAIS o pintor (diferencial de glifos contra uma pilha vazia; a tabela certa com o pintor a ignorá-la
  deixa o primeiro gate verde). Ver [[feedback_a_panel_can_hold_one_name_column_per_family_of_row]].
- ⚠️ **Uma fase-filha do quadro que não se chama `fase_*` DESAPARECE do oráculo das leis de ordem** (`frame_text::render_frame` colhe só essas) — e nenhum teste fica vermelho. A lei que a atravessa deixa de ser medida em silêncio (medido 2026-09-14, `line/components`).
- ⚠️ **Uma fixtura que não contém o fenómeno deixa a lei sem gate:** uma duração de `100 000 µs` não distingue `>=` de `>` num tique de `16 667` (o relógio chega a `100 002` e os dois matam no mesmo tique). ⇒ a fixtura de uma FRONTEIRA cai **exactamente** nela (um múltiplo do tique).
- ⛔⛔ **UM GATE PODE PROVAR QUE O DADO EXISTE E QUE ELE FECHA, E NÃO PROVAR QUE ELE CHEGA AO
  CONSUMIDOR.** Medido 2026-09-15: três gates novos afirmavam que a tabela de pesos de pele existia,
  que ela cobria os ossos certos e que somava `1` por ponto — e a mutação que fazia o desenho
  **ignorar a tabela inteira** deixava os **29 verdes**. *É o terceiro passo que um `grep` não vê,
  o mesmo do §5.0 sobre controlos mortos: quem escreve · quem lê · **o leitor DECIDE, ou entrega a
  quem descarta?*** ⇒ o gate que o mata compara o que o QUADRO desenha com as DUAS leis, e precisa
  das duas metades: o quadro tem de dar a lei nova **e** diferir da velha.
  ⚠️⚠️ **E a fixtura teve de ser refeita DUAS vezes para produzir o fenómeno** — o rectângulo do
  arnês tem os pontos de controlo nos CANTOS, e ali as duas leis concordam (`0,000` de separação);
  com vértices na junta sobe a `0,064`; com a arte alta sobe a `10,47`. *Um gate cuja fixtura não
  produz o fenómeno mede outro programa, e passa verde a dizê-lo.*

- [[feedback_a_never_used_warning_measures_visibility_not_the_law]] — um `never used` apanhou um roteiro de smoke sem chamador, e ficaria mudo se a fn fosse `pub`: escreva o censo com PISO
- [[feedback_a_nextest_filter_matches_the_module_not_the_function]] — ⛔⛔ **no nextest o nome de um teste é `<MÓDULO>::<fn>`, e um `-E 'test(x)'` casa QUALQUER das duas metades.** Medido 17/09 ao validar o `censos-da-arvore-combinada.sh`: o filtro nomeava `every_key_of_this_family_exists` e os painéis chamam aos deles `every_key_of_this_panel_exists` — **eles entravam só porque o módulo se chama `every_word_…`**. *A cobertura era por ACIDENTE*, e os **2** censos cujo MÓDULO tinha outro nome corriam **zero** testes: `83` de `90` testes de censo, sete cegos, com o script a imprimir `✓ verdes`. ⚠️ **Um controlo por PACOTE não o apanha** (o `ph2d-editor-core` corria a catraca da shell e lia-se como coberto enquanto o 2.º censo dele estava fora) ⇒ *a granularidade do controlo tem de ser a do objecto que pode desaparecer*: o **MÓDULO**, derivado do mesmo `git grep` que deriva a lista de pacotes. ⛔⛔⛔ **E o controlo deu TRÊS acusações FALSAS antes de acertar, todas por trocar as duas metades do nome:** o `(n/N)` do nextest vem **alinhado à direita** (`( 3/83)`) e um `\([0-9]` acusa os primeiros de mudos; e pôr `the_shell_only_shrinks` (a **função**) numa lista de MÓDULOS acusa a catraca de muda com ela VERDE três linhas acima (o módulo é `architecture_the_shell_only_shrinks`). ⇒ **nada na lista de exigidos se escreve à mão: é o `basename` do ficheiro que DEFINE a coisa**, que é o que o nextest imprime como módulo
- ⛔⛔⛔ **UMA METADE QUE A CINEMÁTICA JÁ GARANTE NÃO PROVA NADA** (Teste Cascadeur, 19/09): um portão chamado «o cotovelo pára no ponto do CÍRCULO mais perto do rato» media só a DIRECÇÃO, e uma mutação que liberta a raiz do boneco deixava-o VERDE (com o corpo livre o cotovelo alcança o rato, a direcção fica perfeita e o círculo desapareceu). ⛔ E a minha 1.ª cura foi pior: medir o RAIO (cotovelo↔ombro) é uma **tautologia** — o cotovelo É a ponta do osso, logo aquela distância é o comprimento dele SEMPRE, faça o solver o que fizer. ⇒ a afirmação com conteúdo é a **identidade do ponto mais perto**: ele NÃO alcança o alvo e fica a exactamente o quanto o alvo está fora do círculo (fecha a 3,8 µm, a tolerância do solver). ⚠️ *Antes de escrever uma metade nova, pergunte se a estrutura já a garante* — se garante, ela é decoração e o portão continua a medir metade do que o nome dele diz.
- ⛔⛔ **Uma constante de nome de portão DECLARADA e nunca usada em `esperados` é uma lei por provar, e o controlo de órfãos não a vê** (mesma corrida): `PORTAO_CIRCULO` vivia há muito no arnês sem nenhuma mutação a nomeá-la — o controlo que criei em 19/09 verifica que todo nome ESPERADO existe na suíte, e é cego a um nome que ninguém espera. *Um portão que nenhuma mutação mata não está provado, está a ser acreditado* — e aqui nem o instrumento o dizia.

---

## ⭐⭐⭐ Uma CHAVE carrega o ENDEREÇO, e é isso que torna visível a cópia que o gate não via (2026-09-17)

Cinco gates de «vocabulário» comparam os arrays de opções de dois ou mais nós para afirmar que eles
usam as mesmas palavras. Todos passavam porque `&[&str] == &[&str]` compara **conteúdo**. Quando a
migração do HR-15 trocou o texto por **chaves derivadas do sítio de declaração**, os cinco
reprovaram — e a leitura de cada um foi diferente:

- **Quatro** eram divergências legítimas de declaração (um nó inline, o outro numa `const`; ou duas
  `const` gémeas em crates irmãs sem dependência). Cura: comparar o **texto resolvido** — e o gate
  fica **mais forte**, porque deixa de afirmar que dois literais estão escritos igual e passa a
  afirmar que o artista **lê** a mesma palavra.
- ⭐ **O quinto era uma DUPLICAÇÃO REAL:** um nó tinha uma cópia inline do vocabulário que os outros
  três liam de uma porta partilhada, e a mensagem do próprio gate já dizia *«os rótulos são os da
  PORTA»*. Cura: o nó passa a ler a porta.

⇒ *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*, e foi preciso um
identificador que carrega o **endereço** para a duplicação aparecer.

## ⛔ Um censo que FILTRA por texto fica MUDO quando o texto vira chave — só um piso o denuncia

`if labels.first() != Some(&"Sink") { continue; }` deixou de casar, nenhum nó entrou na lista de
vistos, e a varredura passou a medir **nada**. ⭐ Quem a tornou barulhenta foi o
`assert!(vistos.len() >= 3)` que já lá estava. *A metade positiva de um censo é o que o faz falhar
alto no dia em que o filtro dele deixa de descrever o mundo.*

## ⛔⛔ Um gate que varre UMA lista é cego à lista IRMÃ do mesmo snapshot

Medido 2026-09-18 (`line/UIUX`, report do dono com foto). O
`no_card_param_of_any_node_paints_a_raw_key` afirma *«nenhum rótulo do cartão é um identificador»* e
varre `v.params`. O título de uma secção vive em **`v.sections`**, a lista irmã — e o cartão pintava
`node.group.shape` com aquele gate, o do painel e o dos canais **todos verdes**: os três varrem
**uma lista cada**, e ninguém percorria a quarta.

⇒ *Uma família de texto nova não herda régua nenhuma só por o CONSUMIDOR dela já ter uma.* Quem
acrescenta uma lista a um snapshot escreve o teste dela, e o cabeçalho do ficheiro passa a **contar**
as superfícies cobertas (aqui: `QUATRO`), para a quinta ficar visível por ausência.

⭐ **E o gate de um valor que é identidade E legenda tem DUAS metades, que reprovam erros OPOSTOS:**
a legenda não pode ser uma chave (o `tr` em falta) **e** a identidade tem de continuar a ser uma
(a «cura» errada — traduzir a montante — faria a dobra mudar de endereço com o idioma). Só a
primeira metade deixaria a segunda cura passar como correcção. Irmã:
[[feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting]].

## ⛔⛔ Uma LETRA SOZINHA é invisível à régua lexical por CONSTRUÇÃO — a cerca tem de ser o TIPO

Medido 2026-09-18 (`line/UIUX`). O `ph2d_label_census::is_language` exige **duas letras SEGUIDAS**,
senão acusaria todo identificador (este repo tem centenas de params `"x"`, `"n"`, `"b"`). ⇒ dois
painéis tinham o censo de texto **VERDE** com a letra pintada no ecrã: a grelha do 9-slice
(`S` `R` `M` `-` `F` `?`), as células da Região (`X` `Y` `W` `H`) e os chips do `Fixed` (`W` `H`).

⭐⭐ *Quando a régua não consegue ver a diferença entre um rótulo e um identificador, quem a vê é o
TIPO*: o pintor passa a receber `ph2d_i18n::TextKey` e escrever `"W"` ali **deixa de compilar**.
⚠️ O tipo dá **uma metade só** — a outra (*a chave existe na tabela*) continua a ser um gate, com a
população DERIVADA das `const` do produto, nunca de uma segunda lista escrita à mão.

⭐ E o mesmo dia deu a lei irmã: **uma legenda e o que ela explica viajam juntas.** As cinco letras
do 9-slice são INICIAIS, e a legenda que as explica (*«S stretch, R repeat, M mirror»*) já vivia na
tabela — traduzida ela e não as letras, passava a explicar letras que a grelha nunca mostra.
Irmãs: [[feedback_a_key_and_a_text_of_the_same_type_is_a_defect_waiting]] ·
[[reference_topic_measurement_discipline]].

## ⛔⛔ UM PISO SATISFEITO PELA FORMA QUE A RÉGUA CONHECE NÃO AFIRMA NADA SOBRE A OUTRA (2026-09-19)

O gate `a_tabela_inglesa_fala_ingles` declara *«nenhuma frase que o artista lê está em português»*
com piso de `>= 5 000` entradas. O leitor dele conhecia **só** a forma `match` (`"k" => "v"`), e a
régua irmã (`keys_declared`) tinha aprendido a forma **TUPLO** (`("k", "v"),`) dois dias antes.

⇒ **1 432 entradas — 23 % da tabela — nunca foram conferidas** (`node_options` 601 ·
`node_params_motion` 430 · `node_params` 398). E o piso não o podia dizer: a forma conhecida traz
`4 815` sozinha, acima do piso.

**How to apply:** um piso só afirma alguma coisa se estiver **acima do que a parte já coberta
produz sozinha**. Depois da cura: `6 706` lidas, piso em `6 400`. E a cura de fundo é **uma PORTA**
— havia dois leitores da mesma tabela, e só um aprendeu.

⚠️ Da mesma corrida, a irmã: o leitor juntava a continuação de linha com um `' '` que o Rust **não**
insere, e não descodificava `\u{…}` (a tabela declara `"+ Track  \u{25be}"`, o painel pinta
`+ Track ▾`, e a régua comparava `u{25be}` com `▾`). *Inócuo para um censo de LÍNGUA; decisivo para
quem compara ao BIT.*

## ⭐⭐⭐ O CENSO QUE SÓ O DONO CONSEGUIA CORRER VIRA UM GATE (2026-09-19)

Os 30 censos do HR-15 leem o FONTE de uma crate; a pergunta do HR-15 é sobre o **PIXEL**, e um
rótulo escrito à mão pinta-se **exactamente igual** ao que veio da tabela. Quem os distinguia era o
dono, a olho, com `PH2D_LANG=teste` — três defeitos achados assim, um report cada.

A varredura que pinta **todo painel do registo** já existia (`nenhum_rotulo_do_app_pinta_nada`).
Faltava-lhe a pergunta: **a tabela sabe produzir este texto?** (exacto, ou preenchendo um modelo
`{marcador}`). Ela achou três rótulos que nenhuma régua de fonte podia ver.

⚠️ **Filtre pelo que é uma PALAVRA** (`is_language`): sem isso a lista abre com `231` acusados e a
maioria são VALORES (`"0.010"`, `"-9.81"`, `"▶"`). E **texto já ELIDIDO é artefacto da medição**
(`"Mas…"` é `Master` medido outra vez) ⇒ regra, nunca isenção: *uma lista de isenções sobre texto
elidido muda sempre que uma coluna muda de largura.*

## ⛔⛔ UM GATE DE POPULAÇÃO CORRIDO COM `-p` MEDE UM APP COM MENOS PAINÉIS (2026-09-19)

Metade dos painéis do registo deste repo está atrás de uma **feature opcional**
(`panel-wet-tuning`, …). Um `cargo test -p ph2d-panel-registry-init` **não as acende**; a
**unificação de features** de um build de WORKSPACE acende. ⇒ o gate novo fechou VERDE com `-p` e
acusou **dois** rótulos na varredura da árvore inteira.

⚠️ A assimetria já estava MEDIDA no cabeçalho daquele ficheiro desde 18/09 (duas colunas: *«`-p`
sozinho (24 painéis)»* e *«árvore inteira (28)»*), e o piso de painéis está no número **menor** de
propósito, para o gate passar das duas maneiras. *Um piso posto no menor dos dois deixa de afirmar o
que acontece no maior — e é lá que o app corre.*

**How to apply:** um gate cuja população são «todos os X do registo» corre-se `--workspace` antes de
se acreditar nele; e a prova de mutação dele também (com `-p` a mutação não é observável).

⚠️ Da mesma corrida: **uma linha de parágrafo não é um texto da tabela**. Um painel que pinte prosa
com `paint_text_block` quebra-a, e o censo mede **cada linha** ⇒ a régua aceita SUBSTRING contígua,
com o afrouxamento declarado. *A alternativa seria uma lista de isenções sobre pedaços de frase, e
esses mudam sempre que uma coluna muda de largura.*

## ⛔⛔⛔ UM ACESSÓRIO PASSADO COMO VALOR DE FUNÇÃO NÃO TEM A FORMA DE UMA CHAMADA (2026-09-19)

Report do dono: *«prefab e Image ainda errados»* — dois chips em inglês normal ao lado de dois
deformados. O pintor era `map_or(…, AssetKind::label)`: o acessório inglês passado como **valor**,
sem parênteses. A régua do gate procurava `.label()`, a forma de **chamada**, e a mutação que
devolvia o pintor ao inglês **sobreviveu duas vezes** antes de eu ver isso.

**How to apply:** uma régua sobre «quem chama este método» tem de conhecer as DUAS formas —
`x.metodo()` e `Tipo::metodo`. ⚠️ E a mutação que a estreita de volta **não sangra** quando a árvore
não tem nenhum uso da 2.ª forma: *a prova do alargamento é o antes/depois da mesma mutação*, não uma
mutação nova.

⚠️ Da mesma corrida, mais duas:

- **A lista do gate era escrita à mão e eu não a fiz crescer com a migração** — ele tinha os seis
  tipos da manhã e eu migrei mais dois à tarde. ⇒ a lista passa a ser **DERIVADA** da árvore (todo
  par `(tipo, acessório)` cujo corpo é `tr_em(…Ingles…)`), com piso de população. *Uma lista que
  decide o que um gate VÊ só cresce sozinha se for derivada.* E a 1.ª derivação leu `impl
  crate::Verb` como o tipo **`crate`**: o tipo é o ÚLTIMO segmento do caminho.
- **Isentar o par `(ficheiro, tipo)` cegou o pintor**, porque a SONDA e o PINTOR vivem no mesmo
  ficheiro sobre o mesmo tipo. ⇒ a granularidade é a **LINHA**: o ficheiro só é isento se TODA linha
  com o acessório casar com um trecho declarado.

---

## ⛔⛔ Um gate BINÁRIO («ou A ou B») fica vermelho sobre código MELHOR quando nasce um C

Medido 2026-09-19 (`hr12_widgets_a11y`, PH2D). O gate aceitava um ficheiro de painel de **duas**
maneiras: ele próprio fia a acessibilidade, **ou** chama um primitivo canónico da casa. Uma wave
da véspera pôs as cinco secções de um painel a chamar **uma porta da própria crate** — que
delega no primitivo — e **três ficheiros que estavam verdes ficaram vermelhos sobre código
melhor do que o de antes**.

⚠ **A tentação é a lista de tolerância** (o gate até a oferece por escrito). ⛔ Uma tolerância
aqui é permanente e cega: ela diz *«confie neste ficheiro»* e nunca reconfere.

⭐⭐⭐ **A cura é ensinar a TERCEIRA forma, com a entrada VERIFICADA:** uma lista
`(porta, crate dona, ficheiro)` mais um teste irmão que exige que o ficheiro exista, defina a
função e **contenha ele próprio um marcador canónico**. *Uma entrada verificada e uma isenção
leem-se igual numa lista; o que as separa é esse teste* — e com ela o gate fica mais forte do que
era, porque passa a saber que a porta delega mesmo.

⚠ **E o vermelho já vinha do commit anterior**, invisível a ele: o gate vive na crate de
fundação e a wave correu a crate do painel. *Um fecho que só corre as crates que a linha EDITOU
é cego aos gates que vivem noutra* — quinta ocorrência registada neste repo.

---

## ⛔⛔ Um corte numa ponta que ENGORDA a outra não é um corte (2026-09-18)

O `action_bus.rs` estourou o tecto de LOC (`709/700`) e a cura foi mover **onze** variantes de forma
idêntica do `EditorAction` para um sub-enum num ficheiro irmão (`709 → 628`) — o mesmo corte que o
irmão `action_bus_hier.rs` já tinha pago, uma família adiante.

⛔ **Mas a 1.ª redacção reescreveu os onze braços do DRENO um a um**, e o `rustfmt` reflow-os para
seis linhas cada: **`+47 LOC`** no `fase_bus_inspector.rs` **para os mesmos onze destinos**, que o
levou de `570` a `617/600`. ⇒ o corte tinha de continuar até ao fim: os onze braços viraram **UM**
com um `match` interior de uma linha por caso (`570 → 561`, abaixo do que o `main` tinha).

⚠️ **A lição é de MEDIÇÃO e não de estilo:** ao cortar por um tecto, meça **as duas pontas** — a que
encolhe e a que recebe. Uma família que sai de um enum chega a um dreno, e a forma que ela toma lá é
uma escolha com preço em linhas.

⭐ **E a fronteira de uma família é MEDIDA, nunca o calendário:** o enum tinha **27** variantes com a
mesma forma; as **11** que saíram são as que carregam vocabulário de `crate::<x>_edits` (o módulo que
a catraca do DAG obriga a existir), e as **16** que ficaram carregam `screens::hero::*FieldEdit` —
edições do modelo de SPRITE e de AUTORIA. *O que as separa é o assunto.*

---

## ⚠️ Adoptadas na integração de 2026-09-20 (da `teste-cascadeur`)

⚠️ Ver a nota igual no [`reference_topic_measurement_discipline`](reference_topic_measurement_discipline.md):
o índice foi compactado no mesmo dia em que estas nasceram, e ficaram **órfãs**.

- ⛔⛔ [Um portão pode ter um DEFEITO escrito dentro dele como LEI](feedback_a_gate_can_record_a_loading_defect_as_a_law.md) — e fica verde sobre a lista truncada que o defeito criou (menu `6`, lista `14`).
- ⛔⛔⛔ [Duas leis a responder à mesma pergunta em CAMADAS diferentes](feedback_two_laws_answering_the_same_question_in_different_layers.md) — a de cima apaga a de baixo, e o portão de baixo deixa de afirmar **sem ficar vermelho**.
- ⛔⛔ [Uma entrada AUSENTE de uma tabela não fica «sem lei»: fica com a lei de OMISSÃO do motor](feedback_an_omitted_table_entry_is_the_default_law_not_no_law.md) — e o portão de continência salta quem não tem limite (o dorso: `90°` dos `118°` do tronco).
- ⛔⛔⛔ [Arnês que escreve na árvore de VERDADE paga três preços e só um é o relógio](feedback_a_harness_that_writes_to_the_real_tree_cannot_be_parallel.md) — `22 min → 138 s`, e a suíte ao lado mede o **mutante**.
