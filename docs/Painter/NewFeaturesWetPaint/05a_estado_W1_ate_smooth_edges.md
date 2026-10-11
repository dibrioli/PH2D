# 05 §4b — Estado, pedaço 1 de 4: W1, as gotas, a borda e o Smooth Edges (07–09/10)

> Pedaço do §4b de [`05_plano_da_linha_e_o_antes.md`](05_plano_da_linha_e_o_antes.md) (o índice), cortado em
> 2026-10-11 por `scripts/doc-split.py`: corpo **verbatim** (a remontagem bateu sha256 com o original;
> 1 alvo(s) de link reancorado(s) para o MESMO ficheiro). Continua em [`05b_estado_W1e_W1f_W1g.md`](05b_estado_W1e_W1f_W1g.md).
>
> ⛔ O que aqui estiver marcado **«medido e REJEITADO»** continua rejeitado.

---


- ✅ **Cerca** (07/10): `tests/it/os_outros_meios_nao_mudam.rs` (FNV dos pixels da sonda a 28; perfis
  `test` e `smoke` iguais; 0,7 s) + `ferramentas/cerca.sh` (ficheiros protegidos só ganham linhas).
  Mutações: `impasto_light.rs:48 DEPTH_UNIT_PX` 16→17 ✗ impasto · `watercolor_render.rs:40 SS0` 0,12→0,13 ✗
  watercolor · `ph2d-painter-brush/src/falloff.rs:115` Smooth 3,0→3,1 ✗ digital · controlo
  `ph2d-wet-paint/src/brush.rs:21 BRISTLE_SEED` +1 ⇒ cerca VERDE com o Wet Paint mudado (fnv
  `0x2c12…` → `0xaa36…`). `cerca.sh`: linha alterada ✗, só acrescentar ✓.
- A sonda passou para o binário único `tests/it/` (era `tests/*.rs` solto); os gestos vivem em
  `tests/it/sonda_gestos.rs`; os bytes dos quatro meios não mudaram com a mudança de sítio.
- ✅ **W1** (07/10, `5fcf7d826` + `c8824cd35`) — onde vive cada coisa:
  - **Formato** — crate `ph2d-brush-presets` (só formato + biblioteca, não conhece o motor): PNG à mão,
    parâmetros num `iTXt ph2d-brush` comprimido (⚠️ `iTXt` e não o `zTXt` do 02 §5: o `zTXt` é Latin-1 e os
    nomes são UTF-8), origem num `tEXt`, texturas NOSSAS num chunk privado `phTx` (PNG cinzento). zlib por
    `miniz_oxide` e **não** `flate2`: no lock o flate2 tem dois motores e a unificação de features escolhe-o
    pelo conjunto do build — o mesmo preset sairia com bytes diferentes num `-p` e num `--workspace`.
  - **Preset do Wet Painter** — `ph2d-tool-painter/src/tool/paint/wetpaint_preset.rs`: os knobs por CHAVE
    do `KNOB_DEFS` + `erase`, os 28 campos do `BrushSpec` que o traço lê (`CAMPOS_DO_PINCEL`), as máscaras da
    forma e do grão, o modo e os volumes; os outros 66 campos em `CAMPOS_FORA` com o porquê (o gate lê o
    `spec.rs`: campo novo sem destino reprova). ⭐ O gate `o_pincel_guardado_pinta_o_mesmo_traco` (TODOS os
    campos mexidos, pelo ficheiro, ao byte) foi visto vermelho por um defeito REAL: 3 153 px — `paper_depth`
    estava fora e o traço lê-o (é a «Paper Texture Strength» por pincel do Rebelle).
  - **Miniatura** = o S da bancada (rampa 0,15→1→0,15, 2 s de água) a 256², no relógio FIXO (renderização
    fora do ecrã; o relógio fixo deixou de ser só `test-support`). Igual no perfil `test` e no `smoke`.
  - **Fábrica** — `assets/brushes/WetPainter/Watercolor/Fluid/` (4, §1.2), gerador
    `presets_de_fabrica::gera_os_presets_de_fabrica` (`PH2D_REGERA_PRESETS=1`); gates contados · ao byte ·
    miniatura = a que o motor pinta hoje (o corpus de regressão: a W2 vai pô-lo vermelho e REGERÁ-LO de
    propósito) · duas miniaturas nunca iguais. Tabela raio → largura por preset (a do Wet Round = §1.1).
  - **Painel** — `ph2d-panel-painter-layers/src/paint_wetpaint_presets.rs`: no Wet Paint a fileira «Preset»
    dos meios dá lugar ao navegador (Favoritos · Recentes · grupos), Favorite, miniatura, Save as New ·
    Save as Default · Reset, os volumes VIVOS **Size px** (= diâmetro; o Size normalizado do topo sai no Wet
    Paint) e **Water** (saiu da secção para o topo) com valores rápidos, e a política dos volumes.
    ⛔ **Opacity fora**: a força não tem leitor no Wet Paint (dono 12/08; gate
    `a_opacidade_nao_tem_leitor_no_wet_paint` fica vermelho quando tiver ⇒ o volume entra). Pressure e
    Length entram com W6/W5.
  - **Disco** — `ph2d-app-painter/src/painter_bridge_presets.rs` (feature do painel): o tool regista o
    `PedidoPreset`, a ponte executa (biblioteca, `estado.json`, política, Save/Reset, avisos visíveis).
  - **Sonda** `PH2D_SONDA_PRESET=<nome>|todos` em `tests/it/sonda_tracos_por_meio.rs`.
  - **Mutações (13/13 sangram, controlos verdes antes):** `serde(flatten)`→`skip` no `extra` ✗ campo
    desconhecido · `NIVEL_ZLIB` 9→8 ✗ ao byte · utilizador não sobrepõe ✗ biblioteca · `paper_depth` não
    escrito ✗ 3 153 px · máscara da forma não instalada ✗ 4 550 px · Size sem ½ ✗ volumes + 2 seams ·
    `QUADROS_DEPOIS` 120→119 ✗ miniatura · guarda do clique do navegador ✗ seam · decoder da política
    trocado ✗ seam · popovers não pintados ✗ 3 seams · ponte sem «lembrar» ✗ política · Reset sem apagar ✗
    ponte · fábrica com 3 ✗ contagem.
  - **Portão batched** (07/10): cerca ✓ · fmt ✓ · clippy `-D warnings` ✓ (apanhou `chunks_exact` e um tipo
    complexo — curados) · nextest das 8 crates tocadas ✓ (apanhou a secção do «On preset change», que
    elidia a 220 px ⇒ «On switch», e «Watercolor»/«User» no HR-15 ⇒ excepção declarada: são PASTAS) ·
    `check --workspace --all-targets` ✓ · `architecture_*` ✓.
- ✅ **Preview VIVO do pincel** (08/10, pedido do dono: *«não se atualizam em tempo real… não consideram o
  paper»*, `b1cc63d2f`): a faixa do painel deixou de ser a miniatura do FICHEIRO e passou a ser o pincel
  de AGORA sobre a obra de AGORA.
  - **Cena** (`wetpaint_preset/preview.rs`): o `BrushSpec`, os knobs, as máscaras, o papel de cor do
    documento, o papel do pincel e a imagem dele, o relevo do substrato, inclinação, grade, mistura K–M, o
    papel visível. **Chave** = os mesmos ingredientes, imagens pela VERSÃO. **Saída** = a do ecrã
    (`take_preview_arc`: papel de cor, vidro, luz do relevo) — o papel aparece porque é a mesma porta. A
    miniatura dos ficheiros é o último quadro do mesmo pintor sobre a obra neutra (bytes da fábrica
    iguais, sem regerar).
  - **Custo medido** (`sonda_custo_da_miniatura`, `smoke`, load 2): 33–80 ms por S ⇒ **thread própria**
    (`ph2d-preview-pincel`, só o pedido mais recente conta) e os quadros **tocados** a 33 ms (a 80 ms a
    animação não se via). Lei: mudança isolada anima já; mudanças a menos de 180 ms (um arrasto) mostram o
    traço FINAL ao vivo e animam uma vez ao sossegar; nunca pisca para o vazio.
  - **Mutações 9/10 sangram**; a que sobreviveu (apagar `tool = Paint` na cena) era EQUIVALENTE — a cena
    não leva a ferramenta molhada — e a linha saiu; o gate do Smear fica contra quem a meter na cena.
- 🔎 **Gotas: a lei do Rebelle medida** (08/10, pedido do dono) — [06](06_gotas_rebelle_x_ph2d.md): no
  Rebelle a gravidade NÃO é do pincel (0 de 281 presets); o escorrimento é da TELA (Tilt, desligado por
  omissão) e a forma das gotas é do PAPEL (Create Drips/Size/Length); a água de fábrica da aquarela é 9–49.
  O nosso: Gravity por pincel, dial ligado por omissão, pente de dedos curtos em vez de 1–2 gotas longas; os
  knobs escondidos `extFingering`/`extRivulets` não fazem gotas. ✅ **Decidido** (§4.3): onda W1b.
- ✅ **W1b — as gotas pela lei do Rebelle** (08/10) — onde vive cada coisa:
  - **Motor** — `ph2d-wet-paint/src/drips.rs` (ficheiro irmão; o solver não cresceu): `DripPaper`
    {create, size, length} (1–10, 5), a **linha de contacto** `pin_downhill` (entre o `prepare_rows` e o
    `outflow_rows` do advect de Jacobi: célula ativa SECA perde a componente de descida do `uv` — a massa
    fica exacta porque saída e transporte leem o mesmo `uv`) e a **gota** `drip_pass` (no estágio final do
    passo, o da cadência: o teste dos seis pontos de retoma fica). `Sim::gotas: Option<DripPaper>` —
    `None` no boot (o port 1:1: fingerprints intactos), o tool empurra sempre `Some`. Sem gravidade as
    duas peças são INERTES ao bit. `Grid::drips` viaja no snapshot; `Grid::drip_dirty` junta-se ao sujo
    nas DUAS portas do passo (`step_simulation`, `step_stage` — `merge_drip_dirty`).
  - **A lei da gota (medida, constantes com a tabela no `drips.rs`)**: a orla de baixo é célula molhada com
    8 células SECAS a jusante (não um buraco da franja); a gota solta-se quando a tela passa o ângulo
    mínimo `SLIDE` (meio geométrico entre a meia e a máxima: anéis 6–8) **e** `gravidade × água da orla >
    HOLD = 0,039` (meio geométrico entre 0,7 = `0,1405` e 0,4 = `0,0109` na máxima). ⛔ Um só limiar de
    peso NÃO separa: na meia inclinação a água 1,0 junta MAIS na orla (`0,180`) que a 0,7 na máxima
    (`0,141`) — o Rebelle só pinga na máxima ⇒ o ângulo é uma condição própria. A gota bebe 25 % do
    excesso das orlas da SUA bacia (`walk_edge`: a orla seguida coluna a coluna, ±`15·Size` px, até 3
    colunas sem orla), desce a 1 px/passo na máxima, e o rasto é `270·(Length/5)^1,6·(bacia/HOLD)^0,1` px;
    cada célula do rasto assenta `0,045 ×` o pigmento da orla onde a gota nasceu (o fio é PÁLIDO e
    uniforme — Rebelle: 40–52 % do escuro do traço; com o pigmento da gota espalhado o nosso dava 82–95 %
    e o funil escurecia ao estreitar), a cabeça espera pela tinta da fileira enquanto a orla ainda dá, e
    quando deixa de dar o resto reparte-se igual pelo que falta (clareia até à ponta, nunca se corta); a
    largura é `0,08·Size²·(falta/total)²` (o funil). Uma bacia não volta a pingar no mesmo episódio molhado, e nada nasce DENTRO da bacia de uma
    gota (os buracos de uma poça a escoar: eram centenas de gotas natimortas).
  - **Tela** — o dial é a porta ÚNICA: nasce DESLIGADO (o tool e o Reset; o motor continua a nascer ligado
    e o reconcile empurra o desligado — `the_tool_default_reaches_the_engine_exactly` reescrito de
    propósito). A gravidade da tela é `Knob::Gravity = 0,01` no `WetKnobs::DEFAULT` (o anel 8 = ×2 = o
    Rebelle no máximo: a orla sem gotas dá baixo/cima **3,03 / 2,11** contra **3,07 / 2,28** dele; a
    `0,005` de antes dava 1,64 / 1,24). A fileira Gravity saiu da secção e da tabela de ajuste
    (`KNOBS_DA_OBRA` em `wetpaint_preset.rs`: não se captura nem se aplica; o painel de ajuste filtra-a).
  - **Papel** — `PainterTool::gotas` ao lado do `papel` (`papel.rs`: `gotas()`/`set_gotas()`), na secção
    Paper só no Wet Paint (`paint_watercolor_paper.rs::paint_drip_rows`; ids
    `PAINTER_WETPAINT_DRIPS_CREATE` em `PAINTER_WETPAINT_CLICKS`, `_DRIP_SIZE`/`_DRIP_LENGTH` em
    `PAINTER_WETPAINT_FIELDS`). Persistem com o documento: `PaintedDocument.gotas` (último campo, 3 bytes,
    `serde(remote)` — o motor não tem serde), degrau **187 → 188** com migração (`ProjectFileV187`,
    `PaintedDocumentSemGotas`: um 187 abre com as gotas de fábrica). O preview vivo segue-as
    (`CenaDoPreview`/`ChaveDoPreview`).
  - **Fábrica** — a água do Rebelle (Wet Round 0,29 · Wash 0,24 · Dry Brush 0,09 · Drip 0,49), sem
    `gravity`; regerada (`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje` ficou vermelho e voltou verde
    no mesmo commit). ⚠️ O Wet Drip a 0,49 fica no vale (0,4 não pinga, 0,7 pinga): decisão do dono pendente
    se deve pingar com a tela no máximo.
  - **A bancada como gate** (`tests/it/gotas_pela_lei_do_rebelle.rs`, 1000×640, 30 s, ~11 s as cinco):

    | caso | água 1,0 | 0,7 | 0,4 | 0,2 | Rebelle |
    |---|---|---|---|---|---|
    | tela direita | 0 | 0 | 0 | 0 | 0 — e o traço de antes AO BYTE (fnv `0xafa3…b7fa`) |
    | meia inclinação | 0 | 0 | 0 | 0 | 0 |
    | máxima | **296** px, 2 fios de 1 px, pálidos e uniformes | **260**, 2 de 1 px | 0 | 0 | 283 · 249 · 0 · 0 (1–3 px) |
    | Create Drips off | 0 · orla baixo/cima 3,0× | 0 · 2,1× | 0 | 0 | 0 · 3,1× · 2,3× (direita: ~1×) |
    | Size/Length 10/10 | **464** (funil 7 px → 1) | **464** | 0 | 0 | 463 · 465 |

  - **Custo a 4096²** (`ph2d-wet-paint/tests/it/drips.rs::measure_drip_pass_cost`, `smoke`, as duas
    variantes intercaladas no mesmo processo, Size/Length 10/10):

    | cena | estágio das gotas p50 / p99 / máx | advect com / sem a linha de contacto (p50) | load |
    |---|---|---|---|
    | 1.ª tentativa (varredura em CAIXA por gota), três poças | 6,1 / 13,5 / **15,5 ms** ✗ | — | 8,5 |
    | 2.ª (a orla seguida, `walk_edge`), três poças, 30–36 gotas | 0,24 / 0,78–0,87 / **0,99–1,62 ms** ✓ | 3,7 / 3,9 ms | 4,1–5,6 |
    | 2.ª, a tela INTEIRA molhada (16 M células, sem gotas) | 0,002 / 2,2 / 4,3 ms | 74,6 / 73,4 ms | 17,8 → 41 (INVÁLIDO) |
    | 2.ª final (tinta por célula), três poças, 30 gotas | 0,27 / 0,74 / **0,76 ms** ✓ | 3,55 / 3,77 ms | 6,6 → 11 |
    | 2.ª final, a tela INTEIRA molhada | 0,001 / 1,49 / **1,49 ms** ✓ | 70,4 / 68,9 ms | 6,6 → 11 |

    ✅ O pior tick das gotas fica ≤ 2 ms a 4096² na 2.ª tentativa (a varredura das orlas e a linha de
    contacto correm por linhas em paralelo; a bacia é a orla seguida, não uma caixa). Os 4,3 ms eram a
    carga de outras linhas (load 17–41). A linha de contacto custa ~1,5 % do advect.
  - **Mutações (17 sangram de 19; controlos verdes antes):** sem o ângulo mínimo ✗ meia · sem linha de contacto
    ✗ Create Drips off + motor · `HOLD` 0,005 ✗ água 0,4 · `FRONT_DRY` 1 ✗ funil 10/10 · bacia desligada ✗
    1–2 fios · sem `merge_drip_dirty` no `step_stage` ✗ · sem `drip_pass` ✗ bancada + determinismo · tool
    a nascer inclinado ✗ `the_tool_default…` · Reset inclinado ✗ · `knob_da_obra` falso ✗ · sem o
    reconcile das gotas ✗ · chave do preview sem gotas ✗ · sem o clique Create Drips ✗ seam · rows não
    pintadas ✗ seam · tabela de ajuste sem o filtro ✗ · lei `GotasDaTela` falsa ✗ censo · `TRAIL_TINT`
    0,2 ✗ fio pálido (0,93). Sobreviveram duas, e as duas mudaram o código: a forma ANTIGA do rasto
    (`pigmento·película/água`) passava em todos os gates de comprimento — a medição mostrou o fio a 82–95 %
    do traço contra 40–52 % do Rebelle, e o rasto passou a ser a tinta por célula com o gate do fio pálido
    (que a `TRAIL_TINT` 0,2 sangra); e o tecto «o fio não escurece», EQUIVALENTE (sem beber,
    `pigmento / células` é constante por construção) — saiu.
  - ✅ **Easy Drips** (pedido do dono 08/10: *«deixe as duas versões funcionando com um checkbox»*):
    `DripPaper::easy`, a caixa na secção Paper ao lado de Create Drips. Ligada, o papel segura menos
    (`HOLD_EASY = 0,00277`, o meio geométrico entre o traço de água 0,2 da bancada, `0,0014`, e o Wet
    Round, `0,0055`) e absorve menos o fio (a película na mesma razão): o Wet Round de fábrica pinga
    **255 px** na máxima (Rebelle 302), nada a meio, o Dry Brush nunca. ⚠️ O Wet Wash pinga também — ele
    junta MAIS na orla (`0,0062`) que o Round, e nenhum limiar separa os dois (no Rebelle o Wash Round não
    pinga). Gate `com_easy_drips_o_wet_round_de_fabrica_pinga_como_o_round_do_rebelle` (mutação «o hold
    ignora `easy`» ✗ 0 px). Persistido no mesmo degrau 188 (o campo entra no `GotasNoFio`).
  - **Censo dos controlos**: as três rows esmaecem com a tela abaixo do ângulo mínimo (`GotasDaTela`) e
    Size/Length com Create Drips off (`FormaDaGota`); a linha «o ângulo do carimbo não muda a tinta da
    água» saiu — com a tela direita ele MUDA (a água a correr apagava a diferença).
- ✅ **2.ª ronda das gotas, pelo smoke do dono** (08/10, duas fotos: *«ficou invertido: sem easy drips
  escorre… a tinta se assenta formando bases retas e degraus… o escorrimento fino parece artificial
  demais»*). A cena dele vive em `tests/it/sonda_gotas_do_dono.rs` (o S e o L, Wet Round a 140 px, água
  1,0). Os quatro defeitos e a cura de cada um:
  - **Degraus e bases planas** — a 1.ª linha de contacto cortava só a DESCIDA no papel seco, e a água
    entrava de LADO por baixo das encostas (26 433 px de tinta fora da mancha na cena do dono). Agora o
    papel seco POR FORA da mancha (seco a jusante, `FRONT_DRY`) não recebe água de lado nenhum; os buracos
    de DENTRO (a franja, com água por baixo) deixam-na passar. ⛔ Fechar também os de dentro prendia a
    água (orla 1,0×, nenhuma gota). Gate `com_a_tela_inclinada_a_mancha_guarda_a_forma_e_nao_faz_pente`:
    0 px fora da mancha (barra 300), mutação da 1.ª forma ✗ 24 848.
  - **O pente** (20 fios) — cada degrau de uma encosta era uma orla. A gota nasce só no PONTO BAIXO com
    água da bacia (`LOW_SLACK`, a caminhada com `EDGE_SEARCH` 12), e a bacia é ±`30·Size` px (Size 5 ⇒ 1
    fio num traço de 150, o Rebelle). A cena do dono: 4 fios (gate: 1–6; a 1.ª forma ✗ 8).
  - **Invertido** — no papel que segura menos a gota nascia cedo e esvaziava a orla rasa, e a tinta dela
    vinha da orla (16–22 de escuro contra 76–116). A tinta do fio é agora uma fração da OPACIDADE do
    traço (`TRAIL_ALPHA` 0,4, pela curva do motor: imune ao amontoado da orla e à franja) e SAI da orla
    e do traço acima dela fileira a fileira; a orla só dá água. Na cena do dono: 4 fios com e sem Easy, ~93
    de escuro nos dois.
  - **Artificial** — o fio ganhou ombros macios (0,35), um desvio lento (0,7 px, onda de 70 px, fase pelo
    hash da origem), um pescoço (3 células a afinar em 12 px) e a contra-gota (disco de raio 2).
  - **Recalibrado pela linha de contacto final** (a gota já não incha para fora da mancha): gravidade da
    tela `0,02` (orla 3,03 / 2,12 contra 3,07 / 2,28 do Rebelle), `SLIDE 0,0283` (meio entre a meia e a
    máxima), `HOLD 0,337` (meio entre 0,7 = 0,91 e 0,4 = 0,125), `HOLD_EASY 0,012` (meio entre o traço de
    0,2 = 0,006 e o Wet Round = 0,025). Bancada: 265 / 258 px na máxima, 464 / 464 no 10/10, 0 a meio, o
    Wet Round com Easy 258 px. Custo a 4096² (load 32–56, inflado): pior 1,13 ms (poças) / 1,51 ms (a tela
    inteira molhada).
- 🔎 **3.ª ronda, pelo smoke do dono** (08/10, foto ampliada ~3×: *«borda inferior pixelada/quadriculada
  assim como escorrimento… isso não acontecia antes»*). O fio: perfil transversal contínuo com o centro ao
  décimo de célula (`SOFT_EDGE`), anti-serrilhado por construção — feito. A borda: é a barreira firme da
  linha de contacto, à célula; antes de hoje a água corria livre e a borda era difusa (e fazia os degraus).
  ⛔ **Recusa MEDIDA:** uma faixa de transição (fração do fluxo para o papel seco de fora, em quatro formas:
  12 % a todas; só a 1.ª fila vizinha de água de verdade; com tecto de película; a faixa a contar como
  seca) inunda a folha em riscos verticais — 106–127 mil px de tinta fora da mancha na cena do dono, contra
  0 com a barreira firme. ABERTO: suavizar a borda sem a abrir (p. ex. difusão lateral do pigmento
  SUSPENSO ao longo da orla, `ExtDiffusion` do solver, ou um dente da borda à meia-célula) — decisão do
  dono se vale a 3.ª tentativa.
- ✅ **4.ª ronda — o escorrimento natural de volta** (dono, 08/10: *«vc aniquilou aquele escorrimento
  bonito e natural de todos os traços com essa barreira??»*). **Create Drips desligado = o motor de antes
  da W1b, AO BYTE** (sem linha de contacto, sem gotas, sem a força da lei): gate
  `sem_create_drips_e_o_escorrimento_natural_de_antes_ao_byte` contra as impressões que a bancada mediu
  com o código de antes (`0x7cc9…6d0e` a meio, `0x0747…3d4e` no máximo). A força da tela da lei passou a
  ser `DRIP_LAW_GRAVITY = 4×` sobre o `Knob::Gravity` do motor (`0,005`, de volta ao boot) e SÓ com
  Create Drips ligado — a calibração da lei não mudou (0,005 × 4 = 0,02). O esmaecido: Create Drips só
  não age com a tela direita; Easy/Size/Length só agem com a lei ligada e a tela além do ângulo mínimo.
  A prova dos degraus corre agora com as gotas ligadas e conta só corridas largas (≥ 8 px) fora da
  mancha. Mutações: a barreira sem a caixa ✗ natural; a força da lei a 1× ✗ 4 da bancada.
  ⚠️ Pendente do dono: qual nasce ligado — hoje Create Drips nasce LIGADO (a lei do Rebelle).
- ✅ **5.ª ronda — a orla de baixo sem brechas** (dono, 08/10: *«vamos tentar corrigir a borda
  inferior»*). A causa, vista de perto: os buracos da franja junto à orla têm papel seco por baixo, e a
  linha de contacto tratava-os como FORA da mancha — a faixa escura ficava cortada por cunhas claras
  verticais. Cura: uma brecha estreita (água dos dois lados na transversal, a até `NOTCH_W` = 4 células)
  enche — a tensão superficial; debaixo de uma encosta (água de um lado só) ou de um arco (abertura
  larga) continua fechada, logo os degraus não voltam (o gate da forma continua a 0 px). Gate
  `a_orla_de_baixo_e_continua_sem_brechas` (colunas com um pixel claro no NÚCLEO da faixa): 4 / 12 agora,
  23 / 26 com `NOTCH_W = 0` (mutação ✗), barra 17. ⛔ Recusa medida: a difusão do pigmento
  (`ExtDiffusion` 0,05 e 0,15) não muda a orla (o pigmento da faixa já está ASSENTE; a difusão só mexe
  no suspenso). `TRAIL_FILM` 0,1 → 0,07: com as brechas a encher a orla dá menos água à gota e o funil
  10/10 da água 0,7 parava a 394 px (Rebelle 465). Custo a 4096² (load 46, inflado): estágio das gotas
  p99 1,34 ms, um pico de 2,1 ms.
- 🔎 **Auditoria da borda** (08/10, pedido do dono: *«nada feito. auditoria com agentes»*): 2 lentes +
  o oráculo do S a 1:1 (fixtures `s_*.png`). 5 achados de correção (L1) e 9 de costura (L2), com o
  mecanismo, a reprodução e o gate que faltava — no prompt da janela seguinte
  [08_PROMPT_janela_W1c_borda.md](08_PROMPT_janela_W1c_borda.md). Dono: os controlos das gotas FICAM no
  Paper; «cuidado para não piorar nossa engine» ⇒ as quatro cercas (incluindo o escorrimento natural ao
  byte) do princípio ao fim.
- ✅ **W1c — a borda e as gotas pela auditoria** (08/10, prompt [08](08_PROMPT_janela_W1c_borda.md); commits
  `4ffbc7432` física · `b6d0b21e0` costuras · `6abfef287` censo). A régua: `ferramentas/borda.py`, portada para
  `tests/it/borda_do_s.rs` (`mede_borda`, `fios_do_s`) — a descida do S (x 440–632), tudo em proporção ao corpo
  com a tela DIREITA do mesmo motor (Rebelle ~102, Wet Round ~213: pigmentam diferente, compara-se a forma).
  Reproduz o README das fixtures (orla do Rebelle 4 / 1,10 · 5 / 0,92 · 8 / 2,0).
  - **L1-1 — a causa da escada não é a força, é a POÇA.** Aos 3 s o nosso S já é o do Rebelle (corpo claro,
    rampa, linha fina); de 3 a 30 s o pigmento SUSPENSO espreme-se atrás da linha de contacto numa faixa de
    cor pura (o nosso pigmento já está a 0,92 da saturação com a tela direita) cujo topo serrilhado é a escada.
    Cura: a gota escoa a orla da sua bacia até `DripLaw::recede` e o canal CONTINUA a escoar depois de o fio
    acabar (`drain_after`) — a água sai, o pigmento fica na linha (o anel). A força fica 4×.

    | S, tela no máximo | degraus | faixa ≥ B0 | rampa | corpo | orla salto / rms | fios a y 330 |
    |---|---|---|---|---|---|---|
    | antes (W1b) | 84–88 % | 11–12 px | 14 | 0,43 | 5 / 0,9 | 2 (1–2 px) |
    | força ×2 · ×1,5 · ×1 | 39 · 23 · 3 % | 12 · 12 · 0 | 16 · 17 · 20 | 0,56 · 0,62 · **0,76** | — | — |
    | `recede` 0,003, sem canal | 31 % | 7 | 19 | 0,43 | 5 / 1,0 | — |
    | **W1c** (`recede` 0,001, `feed` 0,5, canal) | **8 %** | **3** | **23** | 0,44 | 7 / 1,1 | **12 (3 px)** |
    | Rebelle `s_inclinado` | 8 % | 0 | 28 | 0,25 | 4 / 1,1 | 11 (3–4 px) |

    ⚠️ ABERTO: o corpo não clareou para o do Rebelle (0,44 contra 0,25) — é a carga de pigmento do Wet Round e
    a textura das cerdas, fora desta onda.
  - **L1-3 — gotas de toda a orla.** Saiu a regra «só o ponto baixo» (`LOW_SLACK`). A bacia de uma gota é a orla
    que recolhe `basin_water` = 3000 (película × px²) atrás de si — um alcance FIXO não dá os dois oráculos (9 px
    por Size: 9 fios no S e 3 por traço recto; o Rebelle dá 10–11 e 1). Fio ≥ 3 px (`trail_min_px`), funil
    escuro (`neck_dark` 2: raiz ÷ fio 1,69, Rebelle ~2, antes 1,1). A reserva de tinta do fio é a MAIOR orla
    que a bacia teve (escoada, a orla seca aos bocados e o fio clareava: água 0,7 a 189 px < 250).
    S: 12 fios, 9 em encostas (Rebelle 6–8), ~280 px (228–430). Bancada: 259 / 258 px, 1 fio por traço, 10/10
    464 / 464 — todos os portões de antes verdes sem mexer na barra.
  - **L1-2 — `NOTCH_W` fica.** Orla recta: brechas 0 / 10 com ele, 110 / 110 sem (barra 17). Encosta do S, com a
    lei nova: inerte (37 contra 40 colunas com um claro na linha da orla; patamares 1 contra 2) — a
    regularização da escada que a auditoria mediu (2 → 9 patamares) era da escada, que já não existe.
  - **L1-4 — um predicado**, `Folha::por_fora` (edge.rs), para a linha de contacto e a orla de baixo
    (`is_front(e)` ⟺ a célula de baixo está presa). Teste-propriedade com a corrida de 8, o fundo da folha, o
    buraco da franja e a brecha — visto vermelho: «corrida de 8 secas: orla = true, presa = false». A linha de
    contacto ficou a mesma conta; fora da folha conta como seco nos dois (a água cai dela).
  - **L1-5 — a lei em px**: `DripLaw` em dados (`Sim::drip_law`; a calibração varia-a no mesmo processo por
    `PainterTool::set_wet_drip_law`, `#[doc(hidden)]`), convertida por `Sim::drip_px_per_cell` (a razão do tool,
    posta ao nascer a sessão). Grade 2: fios de 275 px contra 280 na grade 1 (antes 352 contra 296, e só não
    mais porque batia no fundo da tela).
  - **L2-2** — o Wet Drip de fábrica: água 0,49 → 0,7 (0,49 → 0 px; 0,6 → 260; 0,7 → 259; 1,0 → 278); a fábrica
    regerada — só o Wet Drip mudou de bytes (com a tela direita a lei não age, as miniaturas são iguais).
  - **L2-1, L2-3..L2-9** (costuras, agente próprio): set_source repõe as gotas e a selecção (censo que
    destrutura `StashedDoc` sem `..`); a dica de Create Drips diz o anel da lei (`wet_primeiro_anel_das_gotas`);
    Create Drips esmaece pela gravidade EFECTIVA; o Tilt tem a dica que aponta o Paper; o Reset do Paper repõe as
    gotas; o Ctrl+Z desfaz as gotas (`ModelSnapshot.gotas`, um passo por clique, arrasto coalescido); os campos
    Drip Size/Length arrastam inteiros (`set_number_integer`); nenhum id da Gravity é encaminhado. 21/21
    mutações sangram. ⚠️ Aberto: as gotas de uma sprite desmontada vivem só na sessão (persistir pedia degrau).
  - **O censo dos controlos** arma o Length no mínimo no anel 8: com a água da bacia a gota corre o Length
    inteiro e, a 128², 5 e 8 davam fios além do fundo da tela («esmaecida=false mas a tinta NÃO mudou»).
  - **Portões novos** (vistos VERMELHOS no `bb2bb1bf4`, numa worktree própria): `a_borda_do_s_e_a_do_rebelle`
    (87 % de degraus), `as_gotas_saem_de_toda_a_orla_do_s` (1 fio), `a_grade_2_da_os_mesmos_fios_em_px` (51 %
    de degraus), `o_wet_drip_de_fabrica_pinga_na_tela_no_maximo` (0 px). Reescrito DE PROPÓSITO:
    `com_a_tela_inclinada_a_mancha_guarda_a_forma_e_nao_faz_pente` — a barra 1–6 fios era a regra do ponto
    baixo, que contradiz o oráculo; agora 4–17 (hoje 14; o pente era 20).
  - ⛔ **Recusas MEDIDAS (W1c):** baixar a força da lei (tira os degraus e o corpo deixa de clarear: 0,76) ·
    assentamento do suspenso na água parada (`sediment` 0,02–0,2: degraus 0 % mas o corpo congela escuro,
    0,76–1,05) · `EdgeDarkening` 100 / 200 (degraus 88 %) · gotas de toda a orla SEM o canal (a poça volta,
    degraus 84 %) · funil `neck_dark` ≥ 3 (brechas 0 → 21 na orla recta) · alcance fixo da bacia (ver L1-3).
  - **Mutações (9 de 10 sangram, controlos verdes antes; agente numa worktree própria):** `drain_after` falso ✗
    27,5 % de degraus · `recede` = `hold` ✗ 86,5 % (e a grade 2, 46,6 %) · `basin_water` 1e9 ✗ 2 fios no S ·
    `trail_min_px` 1 ✗ fios de 1 px · `neck_dark` 0 ✗ funil 1,10 · `Cells::c` sem a razão ✗ grade 2 a 398 px
    contra 280 · a reserva = a última orla ✗ água 0,7 a 194 px · `is_front` sem a brecha ✗ propriedade ·
    `notch_w` 0 ✗ 110 brechas. Sobreviveu `FRONT_DRY` 8 → 7 nos DOIS lados do predicado (a consistência
    fica, e nenhuma cena mede 7 contra 8): constante da W1b, onde 8 → 1 sangrava — tolerância, não lei nova.
  - **Custo a 4096²** (`measure_drip_pass_cost`, `smoke`, Size/Length 10/10):

    | tentativa | três poças: estágio das gotas p50 / p99 / máx | tela molhada máx | load |
    |---|---|---|---|
    | W1b final | 0,27 / 0,74 / 0,76 ms | 1,49 | 6,6 → 11 |
    | W1c 1.ª (bacia por volume, soma o alcance inteiro) | 0,94 / 1,53 / **141 ms** ✗ | 1,93 | 6,5 → 14 |
    | W1c 2.ª (bacia nos dois lados até juntar a água) | 0,91–0,93 / 1,27–1,49 / **3,9–4,3 ms** ✗ | 1,6–3,0 | 3,6 → 13 (3 corridas) |

    ⛔ **Kill-criterion atingido** (> 2 ms depois da 2.ª tentativa): PARADO e reportado ao dono antes da 3.ª.
    O pico é raro (p99 ≤ 1,5 ms) e repete-se no mesmo tique das poças (544 gotas vivas, 3× a W1b). Hipótese
    para a 3.ª, NÃO medida: o `na_bacia` do `spawn` compara cada candidata com TODAS as gotas (O(orla × gotas))
    — um índice das gotas pela coordenada transversal tira-o.
  - Pranchas: [antes · depois · Rebelle, a 3×](prancha_borda_w1c.png) · [o S inteiro](prancha_s_w1c.png).
- 🔎 **Auditoria da W1c, 09/10** (smoke do dono sobre `f216fd97c`: *«borda pixelada e artificial. veja o
  escorrimento. Que app pro faria isso?»* — [capturas](fixtures/smoke_dono_w1c), prompt
  [09](09_PROMPT_janela_auditoria_W1c.md)). Uma rodada: a régua NOVA da cobertura do pixel e da raiz
  (`ferramentas/borda_cobertura.py`, portada em `tests/it/auditoria_borda_w1c.rs` com a sonda
  `sonda_auditoria_w1c`, 9 cenas no mesmo processo, load 3–7), o Rebelle a 400 % (`s_zoom400.png`).
  - **A escada é o CONTEÚDO da tela, não o ecrã.** O dono olha a ~8,5× (cada degrau = 1 px da tela; a
    rampa de cada degrau mede 8 px de ecrã = o filtro Linear, `ImageFilterMode::Smooth` por omissão); o
    Rebelle a 400 % também amplia suave. A orla com a lei ligada é **0/1 em 72,5 %** das colunas da descida
    (meio-tom 9,8 %) — com Create Drips desligado 9,8 / 43,5, com a tela direita 16,6 / 67,4, na grade 2
    (o composite reconstrói por smoothstep) **0 / 68,9**; Rebelle 5,7–19,7 / 67–89. A linha de contacto
    (`pin_downhill`, fluxo 0 na célula `por_fora`) faz a orla binária e nada no motor nem no composite
    (cobertura = `pa`) lhe dá meio-tom.
  - **E o contraste multiplica-a:** o degrau orla → papel vale 1,04·B0 = 222 de escuro num pixel
    (Rebelle 0,44·B0 = 45: **4,9×**); a faixa de ≥ 88 % da cor pura tem **4 px** na bancada e no azul do dono a
    100 px (a captura do dono: 4–5 px; Rebelle 0). O portão da W1c (faixa ≥ B0 ≤ 6) aprova-a.
  - **A raiz:** a orla NÃO muda de altura no fio (degrau 0,7 px contra 1,3 longe dos fios; Rebelle 1,8 / 1,3;
    `drain_after` e `recede` não mudam nem uma raiz). O que o dono vê é o fio RECTANGULAR de topo plano
    (3 px desde 1 px abaixo da orla; Rebelle 5 → 4 → 3, um funil) colado por baixo de uma orla OBLÍQUA de
    cor pura: o lado de cima faz o canto. O «funil» da W1b é só tinta (`neck_dark`), não forma. O fio é
    vertical também no Rebelle (desvio 0).
  - **Os pontinhos** por baixo da orla existem com a tela DIREITA (72 por 100 colunas; Rebelle 2–57): vêm
    do carimbo do pincel (hipótese por leitura: o rabo do falloff × as cerdas × `pigmentPerDab` 600, sem
    piso), não das gotas — e mexer ali muda as duas cercas «ao byte».
  - Portões novos, VERMELHOS e `#[ignore]` (a cura tira-o): `a_orla_do_s_tem_meio_tom_como_a_do_rebelle`
    (72,5 > 25 %) · `o_fio_alarga_para_a_orla_como_o_funil_do_rebelle` (3,5 → 3 contra ≥ 1,5 px) · e a guarda
    `a_raiz_do_fio_nao_faz_degrau_na_orla` (verde hoje). Os achados inteiros e a cura:
    [10_PROMPT_janela_W1d_cura.md](10_PROMPT_janela_W1d_cura.md) · [prancha a 8×](prancha_auditoria_w1c.png).
  - **O «Smooth Edges» do Watercolor avaliado** (pedido do dono, 09/10, antes da cura: *«talvez ajude a
    nossa própria solução… smooth edges»*; e *«não sei se essa imagem do rebelle serve… não tem muito
    pigmento nas bordas»* — a régua do meio-tom é relativa ao pico LOCAL, não depende da carga; o contraste
    sim). A lei (`watercolor_aa.rs::aa_coverage`: 3×3 amostras a ±0,667 texel, `alpha = média ÷ máx` linear
    sobre o pixel acabado, nada de água nem pigmento movido) aplicada FORA do produto às telas da sonda, com a
    mancha 0/1 como campo de cobertura (`ferramentas/smooth_edges_no_wet.py`, emulação):

    | descida do S | binária % | meio-tom % | salto/B0 | último px ÷ puro | faixa pura (12 px acima) | pontinhos |
    |---|---|---|---|---|---|---|
    | azul do dono 100 px, hoje | 72,5 | 9,8 | 1,04 | 1,00 | 5 | 94 |
    | + Smooth Edges | **14,0** | **61,7** | **0,47** | 0,46 | 4 | 38 |
    | azul do dono 60 px, hoje | 57,0 | 23,8 | 0,97 | 1,00 | 3 | 18 |
    | + Smooth Edges | 31,6 | 41,5 | 0,44 | 0,67 | 3 | 4 |
    | tela direita, hoje · + SE | 16,6 · 1,6 | 67,4 · 82,4 | 0,57 · 0,23 | — | — | 73 · 27 |

    ⇒ cura o PIXEL da orla (A1) e apaga boa parte dos pontinhos (A4), mas NÃO a faixa de cor pura (A3) nem o
    rectângulo da raiz (B2) — [prancha a 8×](prancha_smooth_edges.png): a escada fica de cantos macios, a faixa
    fica. Aplicado em toda a mancha muda as cercas «ao byte» (tela direita: 4 204 px; Create Drips off: 6 532).
- ✅ **Smooth Edges no Wet Paint** (dono 09/10: *«opção B - vamos tentar»* — em todos os traços, ligado de
  fábrica). Onde vive: `wetpaint/smooth_edges.rs` (a lei é a do Watercolor, `aa_coverage`, chamada — não
  copiada; o campo é a mancha `pa ≥ SS0`, o pé 3×3 lido do plano de pigmento; a fração entra na COBERTURA do
  pigmento antes do `over`, como o Watercolor escala o alfa acrescentado — misturar a cor straight com a
  base escurecia a orla numa camada transparente, apanhado por `papel_nasce`/`vidro_por_cima`); o
  checkbox **Smooth Edges** abaixo de Paper (`PAINTER_WETPAINT_SMOOTH_EDGES`, `WetPaintState::smooth_edges`,
  o Reset volta a ligá-lo, o preview vivo segue-o); o rectângulo do composite cresce 1 px (o papel ao lado
  da orla). Nada de água nem pigmento se move: o motor fica ao byte.
  - **Cercas:** `com_a_tela_direita…` e `sem_create_drips…` correm com o Smooth Edges DESLIGADO e dão as
    MESMAS impressões de antes (o motor não mudou); gate novo
    `smooth_edges_so_toca_a_silhueta_e_desligado_e_o_over_de_antes` (só pixels de pé misto mudam; o clique
    desliga e volta ao `over` de antes, 0 bytes). `os_outros_meios_nao_mudam` e `cerca.sh` verdes.
  - **Medido** (sonda `sonda_auditoria_w1c`, S na tela no máximo; antes → agora):

    | | binária % | meio-tom % | salto/B0 | pontinhos /100 col. | faixa pura (12 px acima) |
    |---|---|---|---|---|---|
    | bancada | 72,5 → **10,9** | 9,8 → **72,0** | 1,04 → **0,44** | 83 → 35 | 4 → 4 |
    | azul do dono 100 px | 72,5 → 10,4 | 9,8 → 72,0 | 1,04 → 0,44 | 94 → 49 | 5 → 4 |
    | azul do dono 60 px | 57,0 → 22,3 | 23,8 → 58,5 | 0,97 → 0,41 | 18 → 4 | 3 → 3 |
    | Rebelle | 5,7–19,7 | 67–89 | 0,32–0,44 | 2–57 | 0 |

    `a_orla_do_s_tem_meio_tom_como_a_do_rebelle` saiu do `#[ignore]` (verde; vermelho antes). Continuam
    vermelhos e ignorados, para a W1d: a faixa de cor pura (4 px) e o funil. ⚠️ A régua `faixa_pura` contava
    a corrida que COMEÇA na orla e leu «0» com o Smooth Edges (o último pixel fica com meio-tom) — passou a
    maior corrida nos 12 px acima; a «faixa ≥ B0» da W1c (`mede_borda`) tem o MESMO defeito e lê agora 0
    em todas as cenas: não a use para a faixa.
  - **Réguas ajustadas, com o porquê no código:** o fio da bancada mede-se a MEIA ALTURA
    (`fios_a_meia_altura`; tudo > 25 contava a franja de ~⅓ de tom: 5 px); o piso do papel branco do Wet
    Paint é 1 nível (arredondamento do alfa da fração numa camada transparente: pior 1, 0 bytes > 1).
    Miniaturas de fábrica regeradas (`PH2D_REGERA_PRESETS=1`).
  - **Custo** (`medida_do_smooth_edges`, 4096², desligado/ligado intercalados, o mínimo de 7; load 6–13,
    inflado): 1.ª forma (máscara da região em série) tela inteira 10 → **106 ms** ✗; pé 3×3 local 9 → 39;
    máscara por linha 10 → 15; máscara da região em PARALELO + linhas sem borda saltadas **11,3 → 15,3 ms**
    (a recomposição inteira: o clique, a tela toda molhada) e **0,14 → 0,31 ms** no rectângulo de um traço.
  - [prancha a 8×: Rebelle · antes · agora](prancha_smooth_edges_wet.png).
