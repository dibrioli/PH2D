# ADR-0197 — O Painter é uma lei de pigmento em VOLUMES; os meios são pontos dela, não ramos

- **Status:** Proposed — o desenho técnico é delegado (padrão-ouro); o PRODUTO é do dono: a refatoração
  (31/10, noite: *«profunda, completa, brutal … cada parâmetro plenamente funcional, estável, reprodutível,
  confiável»*), o fim do Rebelle como referência (31/10: *«nos tornar nós mesmos como nossa própria cara»*),
  a foto do defeito reproduzido (paragem F0) e a maquete da ajuda (paragem F5).
- **Data:** 2026-10-31
- **Linha:** `line/PainterWatercolor`
- **Amparo:** [`Painter/Arquitetura/`](../../Painter/Arquitetura/README.md) — F0 varredura · F1 mapa dos 4 modos ·
  F1 censo dos 177 controlos · F1 presets de fábrica · F2 grandezas físicas e o modelo `lei_unica.py`.
  Prompt: [`PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md`](../../Painter/49_realshader/PROMPT_REFATORACAO_WETPAINT_E_AJUDA.md).

> ⚠️ Número por recontar na integração (§5.0): escrito como `0197` com o `main` em `0196`.

## Contexto

Captura do dono `dono_6.png`: Thick Impasto com pigmento ↓ e água ↑, amarelo sobre azul assentado ⇒ azul MAIS ESCURO que
o azul puro, ferrugem na orla, verde em degraus. As curas sintoma-a-sintoma do óleo (branco, caroços, orla ferrugem — cada
uma abriu a vizinha) mostram que o defeito é de ARQUITETURA. Medido na F1:

| achado | número | onde |
|---|---|---|
| leis de MISTURA de cor | 5 (RGB, K-M, espectral, RYB, pré-multiplicada) | F1 mapa §3 |
| leis de RELEVO/altura | ≥ 6, dois campos em unidades diferentes | F1 mapa §3 |
| leis de secagem · papel · cobertura | 4 · 4 · 4 | F1 mapa §3 |
| modelos de material+luz para «tinta grossa iluminada» | 2 (rig do Impasto, Óleo Real) | F1 mapa §3 |
| controlos do Wet Paint | 177 (144 no registo + 33 fora) · 5 mortos · 22 famílias de duplicados | F1 censo |
| caminhos para «quanto uma tinta cobre a outra» | ≥ 7 (`esconde`, `Opaque`, `WetCover`, `DryCover`, `Transparency`, `PINCEL_DE_OLEO`+`OLEO_PLENO`, carga) | F1 censo D21 |
| ramos por meio na lei de camada/assentar | 23; o «meio» não é um tipo, é uma combinação de 3 flags e sentinelas | F1 censo · `T/media.rs:10-13` |
| o assentado resolve a cor por INVERSA de `Ecra::Cobre` por Newton presa a `[0,255]` | sai do envelope das duas tintas (mecanismo provável do azul escuro) | `assenta.rs:393/439` (F2 §0) |

Os portões mediam pontos isolados do espaço do painel; o dono mexe em dois botões e cai num canto que nenhum mede.

## Decisão

### 1. Uma lei de PIGMENTO, em volumes, para todos os meios físicos

O estado da célula deixa de ser cor RGB + alfa + modos e passa a ser **pigmento conservado** em três fatias
(F2 §3.1): o **fechado** $R_g$ (reflectância por banda do que curou, papel incluído), o **aberto** $v^a_i$ (assentado
que ainda re-mistura) e o **filme** $v^f_i$ + veículo $w$ com meio $m$. O ecrã é UMA conta (F2 §3.2):

1. Duncan: $\mathcal K=\sum_i (v^f_i+v^a_i)K_i$, $\mathcal S=s(n_v)\sum_i (v^f_i+v^a_i)S_i$.
2. Kubelka 1948, camada finita sobre $R_g$ (limite $\mathcal S\to0$: $R=R_g e^{-2\mathcal K}$).
3. Cobertura **derivada** $C=1-(R|_{R_g=1}-R|_{R_g=0})$ — nunca um knob.
4. Transferências conservam volume (depósito, pick-up $k_u\,o\,A_c\,v$, fecho $R_g\leftarrow R$).
5. O **meio** $m\in[0,1]$ (água → óleo) é UMA escala contínua que escolhe PARÂMETROS ($n_v$, $\eta$, $\tau_y$, $\tau$,
   evaporação, relevo seco), nunca a lei.

Consequências provadas (F2 §3.3, Lemas 1–3 e o teorema; 4 000 casos no modelo): por banda
$R\ge\min(R_g,\min_i R_{\infty,i})$ — **diluir/molhar nunca escurece além do pigmento puro, a mistura nunca sai do
envelope dos ingredientes (sem ferrugem), $N$ passadas são monótonas (sem degraus)**; assentar não muda o que se vê
POR CONSTRUÇÃO (filme e aberto entram juntos em (1)) — zero Newton, zero inversa.

Somem como casos de (1)–(5): `Ecra::{Porto,Cobre,…}` e os campos `filme·esconde·molhada·vela·oleo·opaca`,
`PINCEL_DE_OLEO`, `OLEO_PLENO`, `com_oleo`/`com_opaca`/`com_vela`/`com_vidrado`, a mistura RGB da aguada sobre o óleo, o
clamp por canal à tinta mais clara, a ladeira do óleo com 8 rotas num `AtomicU8` (fica UMA lei de escoamento com
$\tau_y$ — Orchard), a velatura que depende do Live Water.

### 2. Os meios são PONTOS (dados), não ramos

`Meio` passa a ser um TIPO com uma tabela de dados (crate/pasta `paint-meios`), nunca 3 flags + predicados no tronco:

| meio | lei de cor | $m$ | onde vive o resto |
|---|---|---|---|
| **Digital** | `Lei::Digital` — Porter-Duff em linear (a lei que o artista digital espera; o ÚNICO outro ramo, nomeado e explícito) | — | sem água, sem tempo |
| **Watercolor** | pigmento (§1) | ≈ 0, $\phi$ baixo, $h$ de água grande, evapora, orla por Curtis 97 | a aquarela de hoje (`T/watercolor_*`, 33 ficheiros) migra para o tempo comum |
| **Impasto** | pigmento (§1) | alto, $\tau_y$ alto, sem água móvel | o relevo do Impasto vira o relevo comum |
| **Wet Paint** (aquarela · guache · óleo · Óleo Real) | pigmento (§1) | contínuo; guache = $m≈0$ com $\phi$ perto do CPVC | os presets escolhem o ponto |
| **Composite** | composição de passes dos anteriores | — | — |

### 3. Cinco áreas, uma lei por área (a árvore-alvo)

**Tinta** (pigmento $(K,S)$ por banda derivado da cor $R_\infty$ escolhida + transparência do PIGMENTO; $\phi$; $m$;
mistura e cobertura — UMA porta cada) · **Pincel** (forma, cerdas, pressão → $A_c$, carga, depósito, pick-up) ·
**Superfície** (papel/tela: absorção Washburn, relevo $z_s$; UM campo de altura por camada, UMA dobra) · **Tempo**
(UM relógio: evaporação, cura/abertura $o(t)=e^{-t/\tau(m)}$, escoamento/nivelamento; o `ph2d-wet-gpu` é backend
DESTA área) · **Luz** (UM material Principled simplificado e UMA luz; o rig do Impasto vira preset). A árvore de crates
é a da F1 mapa §5; a instrumentação (~39 k linhas de `measure_*/diag_*/sonda_*/medida_*` em `src/`) sai para uma crate
dev-only `ph2d-paint-sondas`. O painel do Wet Paint agrupa-se pelas MESMAS cinco áreas, cada controlo → UMA grandeza
com unidade (a tabela de migração 177 → N vive em `Painter/Arquitetura/F3_controlos.md`, escrita na F4 a partir da
matriz dos presets).

### 4. Portões (a lei nova só entra se os passar inteiros)

1. **Portão-mãe = a varredura da F0** sobre o MOTOR real (não o modelo), em render plano E com luz: por banda
   $R\ge\min(\text{fundo},\text{masstones})$ (zero manchas), ferrugem 0, furos 0 (condicionados à cobertura a pressão 1),
   monotonia em $N$ (zero degraus), para cada pincel de fábrica de cada meio × grelha pigmento × água × pressão × N.
2. **Paridade com o modelo de referência** `lei_unica.py` (fixture gravada com cabeçalho, tolerância por banda) — o
   oráculo é NOSSO.
3. **Mesmo bit por placa** (CPU × backend GPU) como hoje.
4. **Cada controlo vivo em cada meio** (o censo de controlos re-corrido: zero mortos, zero duplicados).
5. **Miniaturas de fábrica = golden**; só mudam onde mostravam um defeito, e o doc diz qual (foto antes/depois).

Portões de PARIDADE COM O REBELLE (fixtures do Rebelle, «28 aprovados contra o Rebelle») deixam de ser lei: os que
medem uma propriedade física reescrevem-se contra a física; os que medem só semelhança saem (lista na F4).

### 5. Migração (F4) — drop-in atrás de uma escolha EM EXECUÇÃO, depois a lei velha sai no mesmo bloco

Ordem (cada passo dá o mesmo resultado onde o antigo estava certo, provado pelas miniaturas): sondas → tinta →
superfície → luz → tempo → meios-como-dados → painel por áreas. A lei nova entra atrás de um `override` em execução
(nunca `const` recompilada) até o portão-mãe passar inteiro; então a velha sai **no mesmo bloco** — nada de duas leis
vivas. Cada preset de fábrica mapeia para a lei nova por função escrita (não à mão), com a matriz da F1 como prova.

### 6. Documentação viva

`docs/Painter/Arquitetura/` é a especificação: cada área um doc, cada lei com a FOTO que o motor pinta hoje e o comando
que a regera. A ajuda ilustrada (F5) usa as MESMAS imagens geradas pelo motor (a ajuda nunca mente).

## Alternativas rejeitadas

| alternativa | porquê não |
|---|---|
| Continuar a curar sintoma a sintoma (rota 7 da ladeira, `OLEO_PLENO`, …) | medido: cada cura abriu a vizinha (3 rodadas em 31/10); a varredura acha cantos que nenhum ponto mede |
| Manter cor RGB no estado e só trocar a mistura por K-M | o assentado continuaria a precisar da inversa da lei do ecrã — a origem do azul escuro (F2 §0) |
| Espectral MyPaint (`espectral.rs`) como lei única | não tem camada finita sobre fundo nem espalhamento ⇒ não dá cobertura derivada; fica como possível escolha de BANDAS (abaixo) |
| Aquarela e óleo como duas leis separadas | é o estado de hoje; a guache e o «óleo aguado» do dono_6 caem no meio e herdam o pior das duas |
| Digital também por pigmento | o artista digital espera Porter-Duff (claro sobre escuro cobre a 100 %); forçá-lo é feature pior que não existir |
| Rebelle como oráculo | revogado pelo dono (31/10) |

## Em aberto (decide-se por MEDIÇÃO na F4)

- **Bandas:** 3 bastam para o teorema; o modelo a 3 bandas dá verdes mais baços (`fotos/f2_lei_unica/a_*.png`). 8
  bandas (IMPaSTo) ou pigmentos latentes — medir custo × limpeza e mostrar ao dono.
- **Calibração $(K,S)$** por pigmento a partir da cor escolhida + transparência; o Óleo Real e a guache calibrada
  conferem-se contra as miniaturas.
- Desempenho: a lei (2) por banda por célula custa mais que o RGB de hoje — medir na Radeon antes de qualquer tecto.

## Preço

A maior mudança do Painter desde o ADR-0099: estado do Wet Paint migrado de cor para volumes, ~23 ramos e ~7 caminhos
de cobertura apagados, a aquarela e o Impasto realojados no tempo e no relevo comuns, um painel reorganizado e
miniaturas regeradas. Várias janelas de trabalho. Em troca: uma lei com prova, um portão que varre o painel inteiro, e
uma árvore que uma LLM lê por área.
