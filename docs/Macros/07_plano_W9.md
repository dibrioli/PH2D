# Macros — W9: a marca do artista (2026-10-17)

> Pedido do dono (17/10), sobre a [auditoria do Record](06_auditoria_W8_record.md) (o smoke dele reprovou
> o Record; causa medida em A1): **opção 1, marcar onde o artista mexe depois de gravar**, mais A3 (custo
> por quadro), A4 (o Loop a cada Stop; o ▷ em Arrange) e os gates que faltavam, com o worker REAL e o
> Painter ligado, num bloco só. Antes: [plano W8](05_plano_W8.md).

## 1 — A lei

Um texel MARCADO (o artista escreveu ali depois da gravação) mostra a imagem atual; os outros, o palco. A
régua por igualdade de píxeis da W7/W8 (`recebeu_a_gravacao`, «≠ fim / ≠ início») saiu: ela lia o minuto
da água depois do Parar como pintura do artista (A1). A água que corre sozinha não marca.

## 2 — Onde a marca vive (e porquê fora do mundo)

| Onde | O quê | Porquê |
|---|---|---|
| `ph2d_tool_painter::macro_track::marca` | `MarcaDoArtista`: 1 bit/texel, a ÉPOCA (FNV-1a 64 estável da gravação crua: estado inicial + eventos; as edições das etapas não a mudam), a caixa, a versão, o formato gravado (corridas de zeros) | tipo partilhado pelo Painter, pelo palco e pela shell |
| `UndoController.marca` (`undo_marca.rs`) | a marca do documento que o Painter tem e o passo em curso | é por documento (viaja com o documento guardado) e TODO registo de passo passa pelas duas portas do controlador (`record_structural_hinted`/`_coalesced`) — vários fechos de forma não passam pelo `commit_structural_edit` |
| `PalcosDoPainter.marcas` (`marcas.rs`) | por (StableId da sprite, época): a marca que o Painter deixou, a BASE (os píxeis quando ele a largou) e a efetiva | fora do `ProjectState`: o desfazer global regista um passo a cada mudança do mundo depois de uma entrada — um componente marcaria um passo global por pincelada |

⛔ **Recusado, medido no código:** a marca como componente do ECS (passos globais duplicados; o `restore`
do desfazer global re-cria as entidades e um componente não registado morria); a marca pela régua do
`mark_dirty` (a água, o desfazer e o papel chamam-no — explorador, 17/10); a marca pelas entradas do
desfazer (a absorção põe a água da gravação no passo seguinte do artista).

## 3 — Quem marca (o Painter)

Um passo REGISTADO marca `((mudou ∨ miolo) ∧ (¬água ∨ disco)) ∨ disco_na_água`:

- **mudou** — os texels onde algum plano (cor de cada camada, relevo, cobertura, material) difere entre o
  `before` e o `after` do passo; a tela da camada ativa só na janela declarada (`hint`).
- **água** — os texels que o composite do Wet Paint mudou desde o início do passo (`agua_mudou`, a caixa
  copiada antes de cada composite, comparada depois). Só com marca ativa.
- **disco** — o pincel cobriu (`stamp_dabs`, raio × alcance da ponta + ½): o traço sobre a água conta.
- **disco_na_água** — os dabs que entram no fluido (`stamp_dabs_wetpaint`): a tinta molhada aparece pela
  água, às vezes depois de o passo se registar — marcam sem esperar.
- **miolo** (smoke do dono 19/10, `macro_track/forca.rs`) — onde o pincel cobriu com força ≥ ½ (perfil da
  ponta × cobertura × Flow, cada sub-dab da cápsula, compostos `1 − Π(1 − aᵢ)` ou pelo máximo sem
  Accumulate), mudado ou não. Sem ele, preto sobre o preto que a gravação deixou no fim não mudava o byte,
  ficava sem marca e o Play abria ali uma faixa da gravação (branca). Medido sem a cura: 136 de 136 texels
  do miolo sobre a faixa sem marca no Digital e no Watercolor; Impasto (o relevo muda) e Wet Paint (o
  disco inteiro) já marcavam. ½ é onde os dois erros de uma marca binária se cruzam (mostrar o fim erra
  pelo que estava por baixo, `1 − a`; mostrar a gravação erra pela tinta, `a`). Ponta com forma, grão ou
  achatada (elipse): só o que mudou (o perfil redondo mentiria para cima).

Os discos mostram-se já (provisórios: o traço em voo aparece por cima do clipe); o registo decide. A
reescrita de uma etapa e o «Recomeçar» suspendem a marca (são da macro). O passo leva o delta (bits que
ligou): o desfazer desliga-o, o refazer liga-o, a absorção da água conserva-o, o run coalescido junta-os.
A reescrita salta os texels marcados (e continua a exigir «ainda é o fim antigo»).

Limite dito ao dono: **a água da pincelada nova que escorre para FORA do pincel não marca** — aparece
quando o clipe acaba (como a água da gravação, W8 2c).

## 4 — O palco

- `liga_a_marca` (por quadro, antes da reescrita): o Painter ligado a uma imagem com macro recebe a marca
  dela ao ligar-se e devolve-a a cada passo; cada sprite sem o Painter lê-se quando os píxeis dela mudam.
- Sem o Painter não há água: a marca efetiva = a que o Painter deixou ∪ o que mudou desde a BASE (a 1.ª
  leitura depois de ele a largar). Desfazer uma ferramenta de imagem devolve os píxeis ⇒ desmarca
  sozinho. A reescrita da macro re-baseia (o que valia passa a ser a marca).
- Gravar de novo = outra época = marca vazia; desfazer a gravação devolve a `Macro` antiga e a marca dela.
- O fio do fim só corre para a reescrita de uma etapa (a mistura já não o espera).

## 5 — A3: custo por quadro

Sonda `sonda_o_custo_da_marca` (1024², gravado no Painter ligado, worker real, 60 Hz, 280 quadros por
variante intercalados; `load` 185 → 195: TETO):

| Variante | média | máx | subidas inteiras |
|---|---|---|---|
| W8, a régua de igualdade (auditoria A3, `smoke`, só CPU) | 6,6 ms | 12,5 ms | 1 por quadro |
| W9, água sem marca (o Rewind de A1) | 0,028 ms | 0,319 ms | 0 |
| W9, com uma pincelada marcada, relendo a imagem INTEIRA | 10,508 ms | 25,150 ms | 0 |
| W9, com uma pincelada marcada, relendo só a CAIXA (`imagem_aplicada_na_caixa`) | 0,704 ms | 4,873 ms | 0 |

⛔ A chave «só a versão da marca» (a proposta da auditoria) foi recusada: o Wet Paint pinta pela água
depois do dab, e a pincelada do artista ficaria invisível no Play até o clipe acabar. A leitura da caixa
dá o mesmo custo baixo sem essa perda.

## 6 — A4

- **Loop a cada Stop — DEFEITO, maior do que a auditoria leu.** O documento já re-cronometrava o laço
  (`retime_macro_clip`); quem faz o laço é a cópia viva no relógio da aba Keys, que só se arma por uma
  intenção — e nem o 1.º Parar a armava (gate vermelho: `loop_range() == None`). Cura: `rearma_o_laco`
  pede `SetLoop` com o laço do documento em cada Parar e em cada re-cronometragem do clipe ativo, só na
  vista Keys (na Arrange a troca de aba re-arma o relógio do clipe).
- **▷ em Arrange — NÃO é defeito, é o desenho da Timeline** (`docs/ObjectsAndComponents/01_objeto_e_componente.md`
  §4.1b): o ▷ da Arrange toca a pilha da cena, e um clipe fora dela não toca, de qualquer tipo; o clipe
  toca na aba Keys ou com o Play da barra de cima (Autoplay). A cena `=5` nasce sem Autoplay e sem faixa.

## 7 — Gravado no projeto

`PROJECT_SCHEMA` 191 → 192: `marcas_do_artista: Vec<u8>` no fim do `ProjectFile` (versão própria). Os braços
191/190/189 acrescentam o `0` do `Vec` vazio. Uma marca ilegível não recusa o projeto (aviso).

## 8 — Gates

Painter (`marca_tests`, `marca_painter_tests`): o FNV-1a publicado; disco, delta, formato; a água sozinha
não marca (worker real); a pincelada marca só no alcance do pincel com a água da gravação a correr, e o
desfazer/refazer depois da absorção; um traço Digital marca exactamente o que mudou; a reescrita e o
Recomeçar não marcam e a reescrita salta o marcado; a imagem na caixa = o recorte da inteira.
Palco (`palcos_fim_tests`): ⭐ com a água real a correr, todo quadro — o Rewind e o clipe inteiro — é o
palco ao byte (a reprodução à parte é o oráculo); a pincelada em voo e levantada por cima, o desfazer
devolve o clipe inteiro; apagar até ao papel fica apagado; a sprite sem Painter (ferramenta e o desfazer
dela); outra gravação e o desfazer dela; a marca gravada volta igual; A3 por contagem (sem marca: zero
leituras e zero subidas; com marca: só caixas dentro da marca). Macros: `o_laco_que_toca_acompanha_cada_parar`.
Shell: `project_marcas_tests`.

## 9 — Prova de mutação (17/10)

**16/20 sangraram na 1.ª prova**; das 4 sobreviventes:

| # | Lei | Desfecho |
|---|---|---|
| M1 | a água fica fora da marca ao fechar o passo | gate novo `um_passo_marca_o_que_mudou_salvo_o_que_so_a_agua_mudou` (o controlador direto) — VERMELHO sob a mutação |
| M2 | a captura da água no composite | **EQUIVALENTE, medido** (256², worker real, 40 quadros): sem pincel a água da gravação muda 47 760 texels; com o traço aberto, ZERO fora dele — o motor congela o resto enquanto o traço está aberto. A captura fica como rede da lei de M1 se o motor mudar |
| M9 | marca vazia ⇒ nada a misturar | equivalente: sem caixa não há leitura do Painter, e a mistura com marca vazia não muda texel |
| M14 | a guarda «o Painter ligou outra imagem» | redundante (ligar outra imagem já limpa a marca do Painter) — saiu |

O gate da pincelada passou a calibrar o raio do disco no 1.º dab e a exigir toda a marca dentro dele.

## 10 — Auditoria (2 lentes) — curado

- A mancha do Solid e os fios na água (`pousa_na_agua`) não marcavam ⇒ `cobertura_na_agua`; gate
  `a_mancha_do_solid_na_agua_marca_o_miolo` (VERMELHO sem a cura).
- Uma ferramenta que muda o tamanho apagava a marca e o desfazer não a devolvia ⇒ a marca fica guardada
  noutro tamanho; gate `mudar_o_tamanho_e_desfazer_devolve_a_marca` (VERMELHO com a lei antiga).
- Os discos da pré-visualização que se re-carimba (Line, Drag Dot, formas) recomeçam a cada carimbo.
- A leitura da caixa confere o tamanho antes de copiar; o formato gravado recusa um lado acima de
  `ph2d_imageio::MAX_RASTER_DIMENSION`.

**Riscos que ficam (documentados, sem cura nesta onda):** uma pincelada no MESMO quadro em que se escolhe
a imagem não marca (a marca chega no quadro seguinte; escolher a imagem e pintar são dois cliques); duas
macros cujo alvo é a mesma sprite partilham-na mal (hoje a gravação vai sempre para o próprio objecto);
mudar só a ordem, opacidade ou visibilidade de uma camada não marca; gravar o projeto com pinceladas por
assentar no Painter grava a marca delas (a política de assentar ao gravar é do Painter).
