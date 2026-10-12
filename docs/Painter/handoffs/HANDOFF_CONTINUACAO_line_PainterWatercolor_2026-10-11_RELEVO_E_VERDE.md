# HANDOFF DE CONTINUAÇÃO — `line/PainterWatercolor`, o relevo que some (#48) e o verde no soltar (#49) (2026-10-11, prompt 22)

## As regras (R1–R9, [PROMPT_LINHAS_RAPIDAS §2](../../archive/integracao-jornadas/PROMPT_LINHAS_RAPIDAS_2026-10-11.md))

- R1 janela curta: ~150 chamadas ou a onda fechada ⇒ commit, handoff ≤ 8 KB, PARE.
- R2 edite pelo `Edit` (novo = `Write`); script só para a MESMA troca em N ficheiros, com `assert` de contagem.
- R3 edição e teste nunca no mesmo comando: `Edit` → `cargo-check-narrow.sh` (⚠️ cego a `tests/it`: use
  `cargo check -p ph2d-tool-painter --test it`; e para os testes da lib `--lib --tests`) → teste.
- R4 no laço, teste por NOME; a suíte e o `nextest-impacted.sh` 1× no fecho, pelo `verificador`.
- R5 nada de `until … sleep`: > 2 min = `run_in_background`. R6 chamadas independentes na MESMA mensagem.
- R7 doc > 40 KB lê-se por `grep -n` + `Read` com offset. R8 escreva menos.
- R9 nenhum teste da suíte normal passa de 10 s em `dev`. Cura: (a) `opt-level = 2`; (b) fixture menor que
  ainda morda (prove com mutação); (c) só `diag_*`/`sonda_*`/`measure_*` vão para `#[ignore]`. ⛔ Lei nunca.
- Comando pesado por `bash scripts/ph2d-run.sh …`. Fish/zsh: listas de caminhos num `bash -c '…'`.

## §1 — Coordenadas

Base `d4a0499be` (os presets do dono; `main` = `8f20c60de`, não andou). Um commit desta janela. Sem push.
O dono, nesta janela: depois do Undo os traços seguintes saíam AZUIS (separa estado da sessão de lei do soltar);
«Keep volumes» — **decide depois** (não mexer); regravar as referências — «faça como achar melhor e pare de me
fazer perguntas» (⇒ decida o técnico sem perguntar; pergunte só produto).

## §2 — Bug #48: o relevo é da TINTA (FECHADO)

Mecanismo: o `Emboss` era um knob do RENDER (`render_pigment_row_visual`) lido com o valor do pincel de AGORA sobre
toda a tinta. Cura: [`Grid::relevo`](../../../crates/ph2d-wet-paint/src/grid.rs) — o Emboss por célula, escrito SÓ
em [`transfer_paint`](../../../crates/ph2d-wet-paint/src/trail/transfer.rs) (a única porta por onde tinta entra no
grid; média pela massa do que estava e do que chega; não escreve se já é igual ⇒ um só pincel = o valor exacto);
viaja no snapshot, no `GridPatch` do undo e no `clear_canvas`; o render lê a célula. O `render_region` (a
referência do modelo, sem chamador de produto) continua com o knob — é o port do modelo.
- Portão [`relevo_tests::o_relevo_fica_com_a_tinta_ao_trocar_para_um_pincel_sem_relevo`](../../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/relevo_tests.rs):
  Oil Bristle, Dry, troca para Wet Round, traço ao lado ⇒ 0 px do 1.º traço (vermelho 5 424; mutação «render lê o
  knob» sangra 5 421). ⚠️ Sem o Dry a tinta continua a secar entre os traços e o portão mede o relógio (2 478 px).
- `product_tool_doors::the_paper_visual_…` dizia «Emboss 0 no render = no-op»; agora diz a lei nova (zerar o knob
  NÃO apaga o relevo; tinta pousada a 0 é no-op).
- Limite conhecido: a água que leva tinta a células que o pincel não pousou leva-a SEM relevo (a orla: 75 px da
  miniatura do Oil Bristle). Transportar o relevo com a massa = mais um plano nos 13 sítios do advect — não feito.

## §3 — Bug #49: o verde no soltar NÃO era K–M (FECHADO)

Mecanismo: no soltar a sim volta a andar e o filme escuro ASSENTA sobre o claro seco; o
[`assenta::resolve`](../../../crates/ph2d-wet-paint/src/assenta.rs) (Newton por canal) media a derivada de cada canal
contra o ecrã do INÍCIO da volta — mas o tapar extra (`diferenca_de_pigmento`) acopla os três canais: mexer no R
mudava o G e a derivada do G media isso. O assentado ia a verde-água (83,178,211). O Undo «curava» porque o claro
passa a BASE congelada (sem assentado no motor). Cura: cada canal lê o ecrã depois do anterior (custo: ≤ 3 `ecra`
a mais por volta). O comentário antigo «o miolo esverdeava +24 ao secar» (a guarda `|d| < 0,1`) era o mesmo defeito.
- Lei: [`assentar_azul_sobre_azul_nao_muda_o_tom`](../../../crates/ph2d-wet-paint/src/assenta.rs) — 4×4 azuis, 60
  passes a `d` 2–5 %: pior +15 (antes) → ≤ 1,4. Mutação (ecrã velho) sangra.
- Produto: [`verde_no_soltar_tests::o_azul_sobre_azul_nao_fica_verde_no_soltar`](../../../crates/ph2d-tool-painter/src/tool/paint/wetpaint/verde_no_soltar_tests.rs)
  — claro seco + escuro a 0,1 do pigmento, Oil Bristle e Express Oil, caixa on/off: antes +17 / +10, agora ≤ 3.
  Sonda `--ignored sonda_verde --nocapture`: os 12 pincéis × 3 forças.
- O portão que estava verde: `na_lei_do_rebelle_assentar_preserva_o_ecra_seco` media UM passe, amarelo sob azul.
- O explorador confirmou: nenhum K–M incondicional no caminho do soltar; o `render_region` com `km_glaze` não tem
  chamador de produto.

## §4 — Referências regravadas (o dono viu as pranchas `comparar_oleo/janela22/`)

Miniaturas (`PH2D_REGERA_PRESETS=1 … --profile smoke … gera_os_presets_de_fabrica`): Oil Bristle 139 px (orla do
relevo, até 20 níveis), Express Oil 30 (±1), Wet Dry Brush 108 (±1), Wet Liner 86 (±1); as outras oito ao byte.
Tabela de larguras do Wet Round: 2 pares grandes 325→326, 391→392 (ficheiro E `largura.rs::LARGURA_DO_WET_ROUND`).
As seis cenas de `com_as_opcoes_desligadas…` (hashes novos): até 9 níveis em 4–7 % (S) e < 1 % (bancada).

## §5 — O que NÃO foi feito (a próxima janela)

- **Família C** (tarefa 3 do [prompt 22](../NewFeaturesWetPaint/22_PROMPT_janela_relevo_e_verde.md)): o chato fino de
  lado (D4–D6), o Express Oil opaco e liso, a Acrylic rala, cerdas de duas espessuras. Comece pelas pranchas
  (`ferramentas/prancha_comparar.sh`, fotos `fixtures/rebelle/{oil,express,oilround}.png`) DEPOIS desta janela.
- O R9 que sobra (camadas 44,7 s · largura 24,7 s · mídias 14,8 s) · famílias A e B · tinta/caneta e Staining K–M
  (dono) · «Keep volumes» (dono decide).

## §6 — Fecho

Bateria (`verificador`, `load` 15–79): cerca ✓ · `ph2d-wet-paint` 150/150 `dev` e `ci-test` · `ph2d-tool-painter`
lib 1565/1565 e `it` em `dev` e `ci-test` (o `a_lei_de_omissao_e_a_do_wet_round` reprovou até a constante ir a
326/392; recorrido verde) · censos 134 · 237 · 17 ✓ · `ph2d-app-painter` 103/103 · fmt ✓ · clippy `-D warnings` ✓ ·
typos ✓ · tecto 700 ✓ (maior: `grid.rs` 612) · `nextest-impacted` 17 644/17 645 (a única falha era a constante).
Perfil do laço (10 sessões, antes → depois): paralelismo 1,09 → 1,14 · test:check 3,0 → 3,0 × · Edit 40 → 46 % ·
contexto/passo 432 → 270 mil.

## §7 — Depois do fecho (12/10): o Bug #50 FECHADO

A orla «esbranquiçada» do escuro fino sobre o claro seco (dono, fotos; sumia depois do Undo). Mecanismo: o tapar
extra do `Ecra::Cobre` pinta-se com a tinta diluída em BRANCO; onde o filme é mais escuro, clareava além das duas
tintas. O Rebelle nunca passa da mais clara das duas (`fixtures/rebelle/{oil,express}.png`, borda do vermelho
sobre o azul seco). Cura: cada canal do ecrã pára na mais clara das duas, depois da velatura
([`cor_e_alfa`](../../../crates/ph2d-wet-paint/src/assenta.rs)). Cinco formas medidas no MESMO build (seletor por
env, retirado): só esta passa a caixa E a barra do Rebelle — a tabela das recusas no índice dos fechados (#50).
Tectos re-medidos com a lei nova e escritos ao lado: re-molhar com Glaze Wet 8 → 16 (15,36), livres 1,5 → 3
(2,97); Wet Drip «mais clara» 0,15 → 0,16 (0,1516). Miniaturas regravadas: Oil Bristle e Express Oil, poucos px.
⚠️ Aberto, por decidir: o realce do Emboss por diferença central põe metade da encosta FORA da aresta (+22 sobre o
claro). Uma atribuição «a encosta é de quem sobe» foi medida e não entrou (muda o relevo afinado pelo dono).
⚠️ O dono: «faça como achar melhor e pare de me fazer perguntas» — decida o técnico sem perguntar.
