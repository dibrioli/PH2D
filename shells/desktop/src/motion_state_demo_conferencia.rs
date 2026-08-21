//! **As cenas de GRUPO da conferência** (doc 89, a segunda volta) — o documento que
//! cada uma monta e a PROSA que ela imprime.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE, a mesma série que o `motion_state_demo_router`
//! já fez uma vez:** o construtor responde *como um `MotionState` nasce*, o roteador
//! responde *que documento o ambiente pediu*, e este arquivo responde *o que o artista
//! tem de OLHAR*. Cada grupo tem a mesma forma — o cabeçalho, as bandas nomeadas, as
//! leituras e a linha do `PARE` — e é isso que o torna uma família, não uma pilha.
//!
//! ⚠️ **NÃO há um segundo `match` aqui, de propósito.** O roteador continua a ser a
//! ÚNICA lista de níveis: dois `match` em dois arquivos deixariam um nível reivindicado
//! duas vezes passar **em silêncio** (o compilador só vê `unreachable pattern` dentro de
//! um mesmo `match`), que é exactamente o defeito que o cabeçalho do roteador nomeia.

use super::*;

/// A ARITMETICA do dominio de valor (doc 89, o grupo A): cinco nos irmaos,
/// dez perfis, e cada modo NOVO ao lado do seu CONTROLE.
pub(super) fn arith(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_arith::build_arith_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[arith-demo] CADA FILEIRA E' UM GRAFICO: {} pecas por fileira, e o Y de cada peca E' o valor.",
        conferencia_demos_arith::COLS as u32,
    );
    for (i, label) in conferencia_demos_arith::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Nenhuma fileira esta' sozinha -- cada modo NOVO tem o vizinho do MESMO no' ao lado,
  sobre a MESMA entrada. A pergunta nao e' \"apareceu alguma coisa?\" e sim \"apareceu coisa
  DIFERENTE?\": dois perfis identicos sao um param de modo que o kernel ignorou.
  (!) As tres leituras que valem: os dois DENTES DE SERRA diferem so' na metade ESQUERDA (o
  truncado mergulha abaixo do eixo, o aterrado nunca) - as duas ESCADAS diferem so' no MEIO
  (o Truncate tem um degrau de largura DUPLA sobre a origem) - e as fileiras 5-7 sao a MESMA
  rampa como reta, escada e S."
    );
    sinks
}

/// O RUIDO e o RELOGIO (doc 89, o grupo B): os dois geradores TEMPORAIS,
/// dez perfis, e a unica leitura desta jornada que so' o PLAY responde.
pub(super) fn noise_clock(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_time::build_time_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[time-demo] CADA FILEIRA E' UM GRAFICO: {} pecas por fileira, e o Y de cada peca E' o valor.",
        conferencia_demos_time::COLS as u32,
    );
    for (i, label) in conferencia_demos_time::row_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
            "  (!) DE' PLAY -- esta cena tem uma leitura que uma foto nao responde. Um campo que fecha
  o laco e um que nao fecha sao INDISTINGUIVEIS parados, e o laco e' o item de maior valor
  da familia (um ruido que nao fecha nao faz um GIF).
  (!) As quatro leituras: (1-2) a de baixo volta a MESMA forma a cada {loop_s:.0}s, a de cima nunca -
  (3-4) a mesma pilha de 5 oitavas com detalhe mais FINO em baixo - (5-7) a 6 e' a 5
  DESLIZADA ao longo da fila (as mesmas feicoes, 0,4 de celula adiante) e a 7 e' outra
  FATIA do campo, no eixo do TEMPO, onde nao existe seed nenhum - (8-10) a 9 anda em
  LOCK-STEP com a 8 (0,5s por ciclo e 120 BPM sao o MESMO numero em duas reguas) e a 10
  e' visivelmente mais rapida.
  (!) As fileiras 3-7 estao CONGELADAS de proposito: uma comparacao de FORMA nao pode ser
  tambem uma comparacao de instante.",
            loop_s = conferencia_demos_time::loop_seconds(),
        );
    sinks
}

/// AS ESTATISTICAS (doc 89, o grupo C): os agregados novos do reduce, as
/// duas portas que os escopam, e os pesos da janela do smooth.
pub(super) fn stats(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_stats::build_stats_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[stats-demo] CADA BANDA E' UM GRAFICO: {} pecas, e o Y de cada peca E' o valor. \
             As pecas PEQUENAS sao o campo; as GRANDES sao a estatistica sobre ele.",
        conferencia_demos_stats::COLS as u32,
    );
    for (i, label) in conferencia_demos_stats::BAND_LABELS.iter().enumerate() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Esta cena julga-se PARADA -- nada aqui depende do relogio.
  (!) O campo das bandas 1-4 e' ENVIESADO de proposito (x^4: quase tudo perto do chao, uma
  cauda alta). Num campo simetrico a media e a mediana cairiam no MESMO lugar e a banda 1
  desenharia duas retas coincidentes -- verde por vacuo, no sentido visual.
  (!) As quatro leituras: (1) as retas da Mean e da Median NAO coincidem - (2) ligar a mask
  SOBE a reta da media, e ela continua a ser desenhada por TODAS as pecas (a mascara
  escolhe quem e' CONTADO, nunca quem e' RESPONDIDO) - (3) ligar o group transforma a reta
  numa ESCADA de {bins} degraus - (4) o Range mede o vao inteiro e o Std Dev a dispersao,
  bem mais baixa. E as bandas 6-8 filtram o MESMO degrau com o mesmo raio: a de cima tem
  rampa RETA com duas QUINAS, a de baixo e' um S sem quina nenhuma.
  (!) Se a lista de 8 bandas acima nao aparecer, PARE: o resto da cena nao diz nada.",
        bins = conferencia_demos_stats::group_bins() as u32,
    );
    sinks
}

/// A COMPARACAO E O NOME QUE NAO RESOLVE (doc 89, o grupo E): as duas
/// perguntas que o grafo nao sabia fazer -- "a > b?" em um no' so', e
/// "este nome nao resolve" em voz alta.
pub(super) fn compare(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_compare::build_compare_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[compare-demo] CADA FILEIRA E' UM GRAFICO: {cols} pecas, e o Y de cada peca \
             E' a MASCARA (0 em baixo, 1 em cima).",
        cols = conferencia_demos_compare::COLS as u32,
    );
    for (i, label) in conferencia_demos_compare::BAND_LABELS.iter().enumerate() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Esta cena julga-se PARADA -- nada aqui depende do relogio.
  (!) O OP (bandas 1-2): a MESMA rampa contra o MESMO limiar ({thr}), uma com Greater e outra
  com Less. Os dois degraus tem de ser COMPLEMENTARES (a de cima sobe onde a de baixo
  desce). Medido: 24 levantadas de cada lado numa fileira de 48 -- eles PARTICIONAM a
  fileira. Se desenharem o mesmo degrau, o kernel nao esta a ler o `op`.
  (!) A TOLERANCIA (bandas 3-4): as duas em Equal contra o mesmo limiar, e SO' o epsilon difere
  ({narrow} contra {wide}). Medido: 4 pecas levantadas contra 18 -- uma banda visivelmente
  mais larga, e um kernel cego ao epsilon desenharia as duas IGUAIS.
  (!) O NOME (banda 5): um `value.attribute` le' '{missing}' (o typo de `vel`) de uma GRADE, que
  nao carrega nenhum dos dois. A fileira sai PLANA, e esse e' o desenho CERTO -- a escada
  devolve zeros no comprimento certo. O QUE A WAVE ACRESCENTA NAO ESTA NO CANVAS: abra o
  painel de GRAFO, e o no' tem um badge (!). Clique nele: ele diz o nome que voce escreveu.
  (!) Se a lista de 5 bandas acima nao aparecer, PARE: o resto da cena nao diz nada.",
        thr = conferencia_demos_compare::THRESHOLD,
        narrow = conferencia_demos_compare::EPS_NARROW,
        wide = conferencia_demos_compare::EPS_WIDE,
        missing = conferencia_demos_compare::MISSING_NAME,
    );
    sinks
}

/// O ENVELOPE (doc 89, o grupo F): que FORMA tem uma coisa que acende e
/// apaga -- do lado do PULSO (motion.strobe) e do lado do DEGRAU
/// (motion.delay).
pub(super) fn envelope(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_envelope::build_envelope_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[envelope-demo] CINCO PARES: cada par e' o MESMO rig com UM knob de diferenca. \
             {cols} pecas por fileira.",
        cols = conferencia_demos_envelope::COLS as u32,
    );
    for (i, label) in conferencia_demos_envelope::BAND_LABELS.iter().enumerate() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY. Um envelope e' uma forma no TEMPO -- uma foto de um instante mostra
  dois tamanhos e nao diz nada sobre como se chegou a eles. As batidas sao de {beat}s.
  (!) ATTACK (1-2): a de cima POPA no instante da batida; a de baixo INCHA ao longo de {half}
  ticks (meio segundo). Era o trecho que o no' nao tinha -- ele subia sempre em UM tick.
  (!) HOLD (3-4): a de baixo fica CHEIA por meio segundo e SO' ENTAO cai. Se ela apenas cair
  mais devagar, o plato' virou uma queda lenta e nao e' isto.
  (!) SHAPE (5-6): a MESMA queda, uma exponencial e a outra atraves de um DEGRAU desenhado --
  a de baixo fica cheia e CORTA, o que nenhuma exponencial faz.
  (!) PROBABILITY (7-8): a de cima acende a fileira INTEIRA em toda batida; a de baixo acende
  ~{some:.0}% das pecas -- e PECAS DIFERENTES a cada batida. Se forem sempre as mesmas, o
  sorteio travou (a pista tem de avancar em todo pulso que CHEGA, nao so' nos aceitos).
  (!) A CONTAGEM BALANCA, de proposito: um sorteio por-peca sobre {cols} pecas tem desvio
  de ~2 pecas, entao 6 a 11 e' o que uma moeda justa entrega. E' a MEDIA que converge.
  (!) RISE != FALL (9-10): as duas seguem o MESMO degrau quadrado. A de cima sobe e desce no
  mesmo tempo; a de baixo SALTA (regua {fast}) e ESCORRE (regua {slow}).
  (!) Se a lista de 10 bandas acima nao aparecer, PARE: o resto da cena nao diz nada.",
        beat = conferencia_demos_envelope::BEAT,
        cols = conferencia_demos_envelope::COLS as u32,
        half = conferencia_demos_envelope::HALF_SECOND as u32,
        some = conferencia_demos_envelope::SOME * 100.0,
        fast = conferencia_demos_envelope::FAST,
        slow = conferencia_demos_envelope::SLOW,
    );
    sinks
}

/// PARA ONDE ISTO VAI (doc 89, o grupo G): a coluna `vel` ganha um PRODUTOR,
/// e os leitores que ja' existiam deixam de receber zeros.
pub(super) fn velocity(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_velocity::build_velocity_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[velocity-demo] TRES PARES: cada par e' o MESMO rig com um NO' (ou um knob) de \
             diferenca. {cols} pecas por fileira.",
        cols = conferencia_demos_velocity::COLS as u32,
    );
    for label in &conferencia_demos_velocity::BAND_LABELS {
        eprintln!("  {label}");
    }
    eprintln!(
        "  (!) DE' PLAY. Uma velocidade e' a diferenca entre DOIS instantes -- uma foto ja'
  mostra a fileira variada (cada peca esta' num ponto distinto do percurso), mas so' o
  movimento mostra a peca a INCHAR quando acelera.
  (!) TAMANHO (1-2): a de cima tem UM tamanho so' (medido: 0,170 em toda peca); a de baixo
  incha no MEIO do vaivem e encolhe nas pontas, onde a peca para para voltar
  (0,203 .. 0,435 -- duas vezes e meia). A de cima e' literalmente o que o canal Speed
  do `value.attribute` desenhava antes desta wave: ZEROS.
  (!) DIRECAO (3-4): as pecas sao TRACOS, e as de baixo apontam para onde vao -- numa orbita,
  a tangente, que varre o circulo (medido: -162,1 a +178,2 graus) contra um controle
  que nao tem `rot` nenhuma. Se elas ficarem todas no mesmo angulo, o alinhamento
  nao chegou.
  (!) SMOOTH (5-6): o MESMO driver tremido. A de cima usa a diferenca crua e o tamanho
  PISCA; a de baixo passa pelo one-pole e RESPIRA -- medida a agitacao de um tick para
  o seguinte na MESMA peca, 0,0274 contra 0,0095 (tres vezes mais calma).
  (!) Se a lista de 6 bandas acima nao aparecer, PARE: o resto da cena nao diz nada."
    );
    sinks
}

/// O DISCO TEM O TAMANHO QUE SE VE (doc 89, o grupo H): o `motion.collide`
/// passa a ler a coluna `size` -- a MESMA que o renderer desenha -- e o
/// `falloff`, o peso que cinquenta nos ja honram.
pub(super) fn collide(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_collide::build_collide_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[collide-demo] TRES PARES: a MESMA fileira apertada com uma COLUNA de \
             diferenca. {cols} pecas por fileira.",
        cols = conferencia_demos_collide::COLS,
    );
    for label in &conferencia_demos_collide::BAND_LABELS {
        eprintln!("  {label}");
    }
    eprintln!(
        "  (!) Esta cena julga-se PARADA. O `motion.collide` e' Pure: ele relaxa a
  entrada a cada cook, e nada aqui depende do relogio.
  (!) TAMANHO (1-2): as duas nascem no MESMO amontoado. Em cima o vao e' o mesmo em toda a
  fileira (medido 0,4095 .. 0,4179, um alvo de 0,42) porque toda peca mede o mesmo; em
  baixo o disco cresce da esquerda para a direita e o VAO CRESCE COM ELE (0,4441 ..
  0,8851). Ponha os olhos nos vaos, nao nas pecas: e' `r_i + r_j`, e a fileira de baixo
  mede 5,18 de ponta a ponta contra 3,31 da de cima.
  (!) FALLOFF (3-4): a de cima empacota inteira. Na de baixo um `field.box` cobre o MIOLO --
  as tres pecas de dentro se ATRAVESSAM (vao 0,22, meio diametro) e as de fora seguem
  exatamente a tocar (0,42). Se a fileira inteira ficar amontoada, a caixa esta' grande
  demais e engoliu todo par.
  (!) MUTAR != PINAR (5-6): a MESMA peca do meio, o MESMO no' de constraint -- so' muda para
  onde o numero vai. Em cima ela e' TRANSPARENTE e uma vizinha passa por dentro dela
  (menor vao 0,2093); em baixo ela e' OBSTACULO, ninguem a atravessa, e a fileira
  empacota em volta (0,4145 .. 0,4186). Sao coisas diferentes, e era essa a linha 62.
  (!) Se a lista de 6 bandas acima nao aparecer, PARE: o resto da cena nao diz nada."
    );
    sinks
}

/// A VIZINHANCA VIRA UM NUMERO (doc 89, o grupo I): o `motion.proximity`
/// publica `neighbours` e `overlap`, e os modos Scale/Hide do Push Apart do
/// C4D passam a ser COMPOSICAO de nos que ja existem -- nao um param novo
/// dentro do `motion.collide`, que era o que a folha 03 prescrevia.
pub(super) fn proximity(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_proximity::build_proximity_demo_document(doc, registry)
        .unwrap_or_default();
    eprintln!(
        "[proximity-demo] TRES PARES: a MESMA fileira, e so' a de baixo mede a \
             vizinhanca. {cols} pecas por fileira.",
        cols = conferencia_demos_proximity::COLS,
    );
    for label in &conferencia_demos_proximity::BAND_LABELS {
        eprintln!("  {label}");
    }
    eprintln!(
        "  (!) Esta cena julga-se PARADA. O `motion.proximity` e' Pure: ele mede a entrada
  a cada cook, e nada aqui depende do relogio.
  (!) A FILEIRA TEM AS DUAS ESPECIES, e e' isso que torna as tres leituras validas: o
  tamanho cresce de 0,40 a 2,40 sobre um passo FIXO, entao as QUATRO da esquerda
  ficam livres (vaos +0,176 / +0,119 / +0,062 / +0,006) e as nove da direita se
  amontoam, ate' -0,448 na ultima. Ponha os olhos nos VAOS, nao nas pecas.
  (!) SCALE (1-2): o pior vao vai de -0,448 a -0,000 -- as sobrepostas encolhem ate'
  APENAS SE TOCAR (a maior cai de 2,40 para 1,04) -- e os tres primeiros vaos saem
  IDENTICOS aos de cima: quem ja' estava livre nao foi tocado. Se a fileira inteira
  encolher, isso e' um `motion.scale` global, nao a composicao.
  (!) HIDE (3-4): a de baixo fica com QUATRO pecas das treze, e sao exatamente as quatro
  da esquerda -- a fileira encolhe de 4,08 para 1,02 de ponta a ponta. Se sobrar zero,
  o limiar comeu ate' os livres; se sobrarem treze, o cull nao leu o `falloff`.
  (!) CONTAGEM (5-6): em baixo o tamanho DEIXA de ser o gradiente e vira um medidor -- as
  pecas passam a ter QUATRO tamanhos so' (0,60 · 1,00 · 1,40 · 1,80), um por numero de
  vizinhos tocados, com a ponta livre no menor. E' a segunda coluna que a wave publica:
  se a fileira 6 desenhar o mesmo gradiente da 5, o `neighbours` nao chegou.
  (!) Se a lista de 6 bandas acima nao aparecer, PARE: o resto da cena nao diz nada."
    );
    sinks
}

/// A TABELA E A SEMENTE (doc 89, o grupo D / W-E): a lista que o artista
/// DIGITA, sem o teto de oito, e a semente que a identidade do no separa.
pub(super) fn table_seed(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_table_seed::build_table_seed_demo_document(doc, registry)
        .unwrap_or_default();
    eprintln!(
        "[table-seed-demo] CADA FILEIRA E' UM GRAFICO: {cols} pecas, e o Y de cada peca \
             E' o valor.",
        cols = conferencia_demos_table_seed::COLS as u32,
    );
    for (i, label) in conferencia_demos_table_seed::BAND_LABELS.iter().enumerate() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) Esta cena julga-se PARADA -- nada aqui depende do relogio.
  (!) A TABELA (bandas 1-2): a de baixo autora {steps} passos por TEXT PARAM, acima do teto de
  OITO que o nome `v0..v7` impunha. O dente de serra dela e' {ratio}x mais largo que o de
  cima -- se os dois tiverem a MESMA largura, a tabela nao chegou ao cozido.
  (!) A SEMENTE (bandas 3-6): as quatro tem a MESMA semente autorada (7). As DUAS DE CIMA tem
  de ser IDENTICAS -- e' o defeito que a wave cura, e sem ele a vista `ligado eles diferem`
  nao provaria nada. As DUAS DE BAIXO nao podem ser.
  (!) Aqui o olho compara SILHUETA, nao altura -- por isso cada fileira tem a propria linha de
  base, ao contrario da cena =43.
  (!) Se a lista de {bands} fileiras acima nao aparecer, PARE: o resto da cena nao diz nada.",
        steps = conferencia_demos_table_seed::TABLE_STEPS,
        ratio = conferencia_demos_table_seed::TABLE_STEPS as f32
            / conferencia_demos_table_seed::LEGACY_STEPS,
        bands = conferencia_demos_table_seed::BANDS,
    );
    sinks
}

/// O PINO alcanca as SIMULACOES (doc 89, o grupo J): tres pares, um por gerador,
/// e em cada par so' uma coisa difere -- um `motion.pin_constraint` na cadeia de
/// estado.
pub(super) fn pin(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_pin::build_pin_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[pin-demo] TRES PARES ({} bandas). Em cada par so' UMA coisa difere: um `motion.pin_constraint`
  na CADEIA DE ESTADO do gerador.",
        conferencia_demos_pin::PAIRS * 2,
    );
    for (i, label) in conferencia_demos_pin::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY. As tres familias sao `Effect::Temporal`, e uma foto de um instante nao
  distingue \"segurou\" de \"ainda nao caiu\".
  (!) A cerca que esta cena derruba estava escrita em DOIS docs -- o do `motion.pin_constraint`
  (\"um pino a montante nao tem fio por onde os alcancar\") e o doc 34 SS7 (\"nao ha' como um
  stream entrar neles\"). As duas eram verdade sobre a porta `in` e FALSAS sobre a cadeia de
  estado, que e' um fio -- e que ja' era o fio pelo qual o `accel` entra.
  (!) As tres leituras, MEDIDAS: (1-2) a corda da direita DOBRA sobre o ponto 12, que nao
  e' ponta nenhuma -- o maior desvio entre as duas e' 14,37, e aquele ponto sai de
  [-6,71, 8,90] para [7,63, 9,00] - (3-4) o corpo da direita PENDE da primeira linha em
  vez de cair (topo -11,57 contra 1,75) - (5-6) tres agentes da direita andam 0,000000
  enquanto o resto do bando anda ate' 3,51, e o bando CONTORNA-OS: um agente de massa
  infinita continua a ser VISTO pelos vizinhos."
    );
    sinks
}

/// O PINO SEGUE A PRESCRICAO, E O BANDO GANHA ESPACO PESSOAL -- os dois reports do
/// smoke da cena `=50`, que sao defeitos DIFERENTES e por isso vivem em dois pares.
pub(super) fn space(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_space::build_space_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[space-demo] DOIS PARES ({} bandas). Cada par isola UM dos dois reports.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_space::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY nas bandas 1-2 (o mastro varre com o tempo) e olhe as 3-4 PARADAS.
  (!) 1-2, A BANDEIRA: as duas TEM de desenhar a mesma coisa. O pino da banda 2 vem de um
  `motion.pin_constraint` e o da 1 e' o param `pin` do proprio no'; antes desta wave o
  generico segurava ONDE ESTAVA, entao a banda 2 ficava PARADA enquanto o mastro varria
  (medido: o pino generico andava 0,0000 contra 3,0000 do intrinseco) e mexer no `spacing`
  deixava-a na largura antiga. Medido AGORA, as duas varrem {:.1} de ponta a ponta e a pose
  relativa delas difere em menos de 0,001 -- se divergirem, a lei voltou a ser duas.
  (!) 3-4, O BANDO: as DUAS bandas levam o peso de separacao no TOPO do slider (6,0), entao
  a unica coisa em que elas diferem e' o ALCANCE. Cada agente desenha a {:.1} de largura, e
  a de cima -- com o peso maximo -- ainda empacota-os a {:.3} de mediana, {}/{} SOBREPOSTOS,
  porque um peso e' quao FIRME e' o empurrao e nao ate' ONDE ele chega (zerar a sobreposicao
  so' com peso pedia 51,2, 8,5x acima do topo). A de baixo tem `separation_radius` {:.1} e
  mede {:.3} de mediana, {}/{} sobrepostos: ponha os olhos nos VAOS, nao nas pecas.
  (!) O preco da banda 4 esta NOMEADO: um espaco pessoal maior que a percepcao e' invisivel
  a' grade do device, entao ela RECUSA a GPU e coze na CPU -- a alternativa era divergir em
  silencio entre as duas rotas.",
        5.0,
        conferencia_demos_space::drawn_size(),
        1.182,
        11,
        40,
        4.0,
        1.614,
        0,
        40,
    );
    sinks
}

/// O PESO POR PARTICULA E OS SUB-PASSOS (doc 89, o grupo K): duas waves
/// independentes, dois pares, cada um com o seu CONTROLE ao lado.
pub(super) fn weight(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_weight::build_weight_demo_document(doc, registry).unwrap_or_default();
    eprintln!(
        "[weight-demo] DOIS PARES ({} bandas). Cada par e' uma wave, e cada um tem o seu CONTROLE ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_weight::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY: as duas metades sao simulacoes, e uma foto de um instante nao distingue
  'ainda nao caiu' de 'nunca vai cair'.
  (!) 1-2, O PESO POR PARTICULA: um `field.index_range` no laco de estado da' peso ZERO as duas
  ultimas linhas do corpo de baixo. A cauda tem de DESPRENDER e o resto tem de FICAR -- medido, a
  extensao do corpo vai de {:.2} para {:.2} e a linha do MEIO anda {:.4} relativa ao pino. Se o
  corpo inteiro descer atras da cauda, o peso chegou ao puxao e nao ao ajuste da forma; se nada se
  soltar, ele nao chegou a lado nenhum.
  (!) A cauda cai RIGIDA, e isso e' correto: no shape matching nao ha restricao de distancia entre
  particulas -- o unico fio e' o puxao ao goal, e peso zero e' fio nenhum. Ela nao se deforma, ela SAI.
  (!) 3-4, OS SUB-PASSOS: a MESMA corda chicoteada, e so' muda quantas vezes por tique ela volta a
  perguntar onde esta'. Ponha os olhos nos VAOS entre nos: a de cima ({} sub-passo) estica {:.0}% acima
  do repouso e a de baixo ({}) estica {:.0}%. As duas correm as MESMAS 3 iteracoes -- se voce so' subir
  as iteracoes, a de cima nao alcanca a de baixo, e e' esse o achado.",
        3.53,
        14.12,
        0.0099,
        1,
        51.7,
        conferencia_demos_weight::substeps(),
        0.6,
    );
    sinks
}

/// **A CENA `=53` — O TETO DA TAXA** (grupo L).
pub(super) fn rate(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks = conferencia_demos_rate::build_rate_demo_document(doc, registry).unwrap_or_default();
    let (max_step, max_accel) = conferencia_demos_rate::ceilings();
    eprintln!(
        "[rate-demo] DOIS PARES ({} bandas). Cada par tem o seu CONTROLE ao lado.",
        sinks.len(),
    );
    for (i, label) in conferencia_demos_rate::band_labels() {
        eprintln!("  {}. {label}", i + 1);
    }
    eprintln!(
        "  (!) DE' PLAY: a pergunta desta cena e' quao DEPRESSA, e uma foto de um instante mostra
  quatro fileiras a alturas diferentes sem dizer porque.
  (!) 1-2, O TETO DE PASSO ({max_step:.2} por tique): as duas seguem o MESMO vaivem, e a de baixo
  NAO ALCANCA os picos -- medido, o maior passo cai de {:.4} para {:.4} e a excursao de {:.2} para
  {:.2}. Olhe o CAMINHO, nao a altura: a de cima faz uma senoide, a de baixo um vaivem de lados
  RETOS, porque uma taxa constante desenha uma reta.
  (!) ABERTO e NOMEADO: o teto e' honrado ao digito na rampa (0,0800) e sobe a 0,1678 no tique da
  INVERSAO -- um gate `#[ignore]` na cena carrega o numero e o mecanismo. Nao e' o kernel: os cinco
  gates de unidade do no' clampam por construcao e sangram sob mutacao.
  (!) 3-4, O TETO DE ACELERACAO ({max_accel:.3} por tique ao quadrado): a de baixo PARTE devagar e
  depois acompanha -- {:.4} nos doze primeiros tiques contra {:.4} depois. E' a diferenca entre
  partir devagar e IR devagar; se ela ficar lenta o tempo todo, isto virou um teto de passo.",
        WORST_FREE, WORST_CAPPED, SPAN_FREE, SPAN_CAPPED, RAMP_EARLY, RAMP_LATE,
    );
    sinks
}

// Os numeros que a sonda `measure_what_the_scene_shows` imprime, medidos na
// arvore. Eles vivem em consts para a mensagem citar a MEDICAO e nao um numero
// que alguem lembrou.
const WORST_FREE: f32 = 0.4188;
const WORST_CAPPED: f32 = 0.1678;
const SPAN_FREE: f32 = 9.6234;
const SPAN_CAPPED: f32 = 3.2184;
const RAMP_EARLY: f32 = 0.1200;
const RAMP_LATE: f32 = 0.2600;

/// **A CENA `=61` — O SUB-PASSO DO INTEGRADOR** (doc 89, folha 17, a linha 76).
///
/// ⚠️ **UMA banda, e isso é o desenho:** o ritmo do sub-passo é do GRAFO, então dois
/// integradores lado a lado a pedir `1` e `16` correriam **os dois a 16**. O que separa as
/// duas respostas é o slider, e a prosa manda mexer nele.
pub(super) fn substep(doc: &mut MotionDoc, registry: &NodeRegistry) -> Vec<NodeId> {
    let sinks =
        conferencia_demos_substep::build_substep_demo_document(doc, registry).unwrap_or_default();
    let (ring, subs, strength) = conferencia_demos_substep::numbers();
    eprintln!(
        "[substep-demo] UMA banda ({} saida): o ANEL cinzento e' o alvo, a bolinha laranja e' a\n  \
         simulacao. Atrator de forca {strength:.0} no centro; ela nasce em cima do anel (raio \
         {ring:.1}).",
        sinks.len(),
    );
    eprintln!(
        "  (!) DE' PLAY: a bolinha cai para o centro, atravessa e volta. Sem erro de integracao ela
  tocaria o anel a CADA meia volta, para sempre.
  (!) A cena nasce com Substeps = {subs:.0} (o topo da faixa do slider): ela volta a {SUB_TOP:.2},
  16% para la' do anel. Clique na bolinha, ache o slider \"Substeps\" no painel de parametros e
  ARRASTE-O ATE 1: ela passa a subir a {SUB_ONE:.2}, TRES vezes o anel, e cada volta vai mais
  longe que a anterior. Suba de novo e ela encolhe de volta.
  (!) O que voce esta' a ver nao e' ruido, e' ENERGIA: um passo grande ACRESCENTA velocidade a
  cada ida e volta, e por isso o erro cresce em vez de se cancelar.
  (!) DEU ERRADO se arrastar o slider nao mudar NADA (o numero nao chegou ao relogio), ou se a
  bolinha sumir da tela ja' no {subs:.0} (isso e' outra coisa, nao o passo).
  (!) UMA banda de proposito: o ritmo e' do GRAFO INTEIRO, entao duas bolinhas a pedir numeros
  diferentes correriam as duas no maior -- a cena diria que o botao nao faz nada. A cena `=52`
  mostra 1 contra 8 lado a lado porque o `Substeps` da CORDA e' outro mecanismo (local a ela)."
    );
    sinks
}

// Os numeros medidos por `measure_integrate_substeps::where_one_substep_breaks_and_eight_hold`
// na forca que a cena autora — raio maximo em 3 s, partindo de 4,0.
const SUB_TOP: f32 = 4.65;
const SUB_ONE: f32 = 12.86;

#[path = "motion_state_demo_conferencia_utilidade.rs"]
mod utilidade;
pub(super) use utilidade::{
    cursor, deform, drizzle, field_family, force_family, fx_family, join, sortkey, taper,
    transform_family,
};
