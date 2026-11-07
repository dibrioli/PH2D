# Continuação — tinta viva, onda 2 (06/11)

Linha `line/PainterWatercolor`, worktree `Worktrees/line-PainterWatercolor`, **à frente do main e não integrada**. Base `bbfefaf393`. Commits da onda: `7f060b58bf` (a tinta viva dentro do Wet) · `45c9216b56` (Borracha na tinta morta) · `8a030e7f3c` + `0b2a4cb639` (compressão) · `62fb674a33` (fmt, clippy e régua). Onda 1: [HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda1.md](HANDOFF_CONTINUACAO_2026-11-06_tinta_viva_onda1.md). Desenho e decisões do dono: [TINTA_VIVA_2026-11-06.md](../TINTA_VIVA_2026-11-06.md).

**Smoke do dono desta onda: ainda não houve.** A foto da Borracha morta aguarda aprovação.

## Item 1 — a tinta viva dentro do Wet (`7f060b58bf`)

**Premissa da onda 1 derrubada:** o balde NÃO é caso do guarda. `set_paint_tool_mode("fill")` é troca de modo, logo adormece e acorda (`o_balde_nao_mata_a_tinta` prova-o). Os casos de escrita alheia dentro do Wet são: Fill e Cut da selecção, Borracha plana (DragDot), paste e afins, e o undo. «Ajuste por menu» não escreve pixels (camada de ajuste).

- `WriteState::{tela_vigiada, tela_que_saiu}` (`undo_window.rs`): `fork_canvas` e `replace_canvas` chamam `a_tela_sai`, que retém a tela vigiada (1 cópia por operação alheia, a do `fork`). A composição da sessão desliga a vigia antes do seu próprio fork e religa depois. Gate `o_traco_molhado_nao_retem_a_tela` (sem isto cada composição copiava a tela: 9,86 ms a 4096²).
- `wet_atravessa_a_troca` (`adormecida.rs`): com gesto/macro a sessão acaba; com passo aberto (`stroke_undo`/`tracos_fora`) espera e o tique segura; sem tela retida, ou de outra camada/tamanho, acaba (lei antiga; `a_foreign_canvas_swap_ends_the_session_before_the_tick_composites` continua verde porque a troca crua não passa por porta). Senão enterra.
- `wet_enterra_como_passo` (`undo_vivo.rs`): o enterro é um passo do undo (`capture_history` + `PassoVivo.tela_antes`); `enterra_celulas` marca os tiles. Vários traços de outro modo enterram-se de uma vez ao acordar; desfeito um, a volta re-enterra o que ainda difere de `tela_antes`. Registo mais novo não desfeito pelo motor = `esquece_os_passos`.
- Undo sem fotografia (`wet_tira_para_o_restauro` / `wet_volta_do_restauro`, `runtime.rs`): tela igual = continua; outra camada = adormece com a dela e acorda a de agora; redo = enterra como passo; undo de traço molhado sem fotografia (o 1.º traço) = a sessão acaba, de propósito: células com tinta invisível na tela reapareceriam a correr.
- ⛔ **Medido e recusado:** recompor a tela inteira depois do undo com células enterradas (3 862 px mudam, a orla re-ilumina). `sess.enterrou` faz `take_dirty` e `toca_tudo`.
- Lei antiga mudada: `a_re_stamp_eraser_bakes_through_and_keeps_the_session`.

## Item 2 — a Borracha na tinta morta (`45c9216b56`, `wetpaint/borracha_morta.rs`)

- **Simulação antes de construir:** alisar + clarear dava borrão cinza. O que a viva faz é: sulco com parede escura à esquerda, tinta ainda saturada, papel só onde a pressão é forte.
- Lei nos pixels, **nenhuma sessão nasce**. Acumula por traço (`Sulco`) a cobertura medida no escudo (`mede_o_escudo`, extraído de `borracha_na_base.rs`): tamanho, dureza, grão e Flow valem; Erase Strength manda. Pressão = coverage × raio/raio do traço, expoente 8. Constantes: `ALISA` 0,8 · `PAREDE` 0,25 do raio · `LUZ_DA_PAREDE` 0,8 · `LUZ` [0,85, −0,5] · `CLAREIA_NO_MAXIMO` 1,1 · `TIRA` 0,35. Só com o wrapper de selecção por fora.
- Soma \|Δ\| contra a imagem de antes: viva 67 398 (2 422 px) · imitada 70 435 (2 099 px) · corte seco 196 654 (1 036 px). Px diferentes contra a viva: 2 611, soma 66 897 (a textura nunca casa ao pixel; o critério é o olho do dono).
- Foto: [foto_borracha_morta.png](material_2026-11-06/foto_borracha_morta.png) (viva, imitada, corte seco de antes; passadas 1 a 3). É a versão final, tirada depois de `LUZ_DA_PAREDE` passar de 1,3 a 0,8. **Aguarda aprovação do dono.**
- A miniatura mantém a Borracha direita (`na_miniatura`); a macro também.
- **ABERTO (Bug #70):** com sessão viva, a tinta antiga na `base` em células sem tinta da grade continua no corte de `borracha_na_base`: abrir um arquivo velho, pintar longe e apagar a tinta velha corta seco. Não mudei porque mexeria na viva aprovada. Candidato a próxima onda; medir se os portões da viva ficam a 0.

## Item 3 — compressão em RAM (`8a030e7f3c`, `0b2a4cb639`)

Código: `wetpaint/dormente.rs`, `grid/celula_bytes.rs`, `grid/snapshot/registo_bytes.rs` (`ph2d-wet-paint`). Dependência: `zstd` 0.14 em `ph2d-tool-painter` (já estava no lock por `ph2d-pen`).

⛔ **O «577 MB» do doc da onda 1 estava ERRADO:** `bytes_retidos` conta 0 para o motor quando ele está com o worker (`EngineSlot::Away`), e a sonda perguntava antes de o trazer. Conta real, Dabber cheio, 14 passadas:

| | 1080p | 4096×2560 |
|---|---|---|
| Total retido, sem comprimir | 1 385 MB | 5 081 MB |
| — grade | 283 | 1 431 |
| — fotos | 142 | 480 |
| — histórico e papel | 945 | 3 098 |
| Comprimido (bytes separados + zstd 1, ao bit) | 190 MB | 500 MB (10×) |
| Comprimir, num fio | 593 ms | 2 526 ms |
| Descomprimir, num fio | 250 ms | 1 020 ms |

RSS do processo a 4K: 8 961 MB antes, 6 519 MB depois de comprimir (o alocador retém parte).

- Em fio porque acordar síncrono engasgaria 1 s: `wet_acorda_sem_esperar` (troca de modo, de camada, tique) põe a descomprimir e o quadro acorda-a quando pronta; o pen-down espera por `wet_acorda`. Limiar `COMPRIME_A_PARTIR_DE` 64 MiB. O orçamento segue 4× `history_budget_bytes`, contando uma tinta em voo por `bytes / ENCOLHE_PELO_MENOS` (7).
- Provas: `comprimir_a_tinta_adormecida_nao_muda_nada`, `a_agua_comprimida_corre_igual` (0 px), `voltar_com_a_tinta_comprimida_nao_engasga`, `o_pen_down_espera_a_tinta_comprimida`, `um_registo_vai_e_volta_ao_bit`. Tinta/pigmento da lei arquivada ficam fora.
- `8a030e7f3c` entrou com um teste vermelho; corrigido em `0b2a4cb639`: era o teste (acordar pelo quadro dá um quadro a mais de água); com a mesma porta dá 0 px.

## Régua de velocidade

`sonda_tinta_adormecida_nos_outros_modos`, medianas em ms (sem · sem2 · dorme · dorme+tela), load 8–11, 4096×2560:

| | sem | sem2 | dorme | dorme+tela |
|---|---|---|---|---|
| Digital, traço | 29,85 | 29,70 | 28,91 | 27,65 |
| Watercolor, traço | 444,08 | 449,57 | 446,40 | 443,28 |
| Impasto, traço | 58,09 | 58,55 | 58,81 | 58,07 |
| Tique ocioso (µs) | 69,79 | 71,56 | 69,66 | 69,98 |

1080p igual. **Kill-criterion passa.** ⚠️ Relógio acima de load ~5 vale pouco: a comparação é intercalada, mas repita com carga baixa (item 6). A régua precisou de conserto: a variante `sem` agora acaba a sessão (`wetpaint_end_session`), porque sair do Wet já adormecia.

## ABERTO medido: adormecer logo após o traço custa caro

> **Resolvido/medido (06/11 noite):** no Dabber o custo caiu a 0,3–0,5 ms (1080p e 4K); a hipótese confirmou-se, era a recomposição da grade inteira com a imagem no ar. O Wet Round continua caro. Ver «Onda do undo, da mão leve e do fecho», §5. O texto abaixo é o estado de então.

Adormecer custou 424 ms (1080p) e 2 040–2 308 ms (4096×2560) logo a seguir ao traço. O doc da onda 1 dizia 0,01–0,02 ms. Cronometrado por partes, tudo está em `wet_ultima_imagem` (`authored_actions.rs:229`: recompor a tela inteira quando `otica.pendente()`, imagem no ar); adiados e guardar < 6 ms. **Hipótese não isolada:** só acontece trocando de modo no mesmo quadro do soltar; não medi com a imagem assentada, nem comparei com a base. Além do engasgo, com células enterradas essa recomposição re-iluminaria a orla. **Primeiro item a medir.**

## Verificado em `62fb674a33`

fmt · clippy das 2 crates limpo · por nome, verdes: `dentro_do_wet` (10) · `adormecida_tests` (12) · `borracha` · `wetpaint::tests::` · `wetpaint_commit` · `undo_tinta_viva` · `censo_tests` · `macro_track` · os dois alarmes de pincel (nenhum pincel mudou) · `historia` (9) · `um_registo_vai_e_volta` · `architecture_workspace_file_loc_cap` (2) · `seam_wetpaint` em `ph2d-panel-painter-layers` 33/33 · binário smoke compilado. **Sem provas de mutação ainda; suíte completa só na integração.**

## Próximo, por ordem

1. **Veredito do dono:** a foto e o smoke.
2. **Medir o custo de adormecer logo após o traço** (o aberto acima).
3. **Item 5 — opção ao salvar** (ligada por omissão). Mapa levantado:
   - `PaintedDocument` é postcard posicional sem versão própria (`tool/persist.rs:28-58`), sem `#[serde(default)]`. `PROJECT_SCHEMA` = 217 (`shells/desktop/src/project_schema.rs:500`), tripla `project_schema_tests.rs:358`, pin `project_geometria_v194_tests.rs:59`, leitura `project_load_read.rs:57-62` e `:206-216` (`falta_no_fim`).
   - ⛔ Campo novo dentro do `PaintedDocument` reprova `o_documento_da_macro_tem_a_versao_do_seu_formato` (`macro_track/tests.rs` ~:250; `inicial.rs:23`). Precedente mais barato: campo no FIM do `ProjectFile`, como `sfx_lab` (216→217), com o id do documento + camada.
   - Salvar: `project_save_gesture` (`project_io.rs:164`) → `project_save_to` (`project_save.rs:24`), síncrono na UI, `rfd` sem opções. Precedente da opção: `SavedSettings` (`project_settings.rs:43`, gate `:216`). O dono decide onde o checkbox fica se não houver lugar natural.
   - Reusar `celula_bytes`. Por escolher: a sessão precisa de se reconstruir no load (`Engine` + planos + `base`); o derivável e a caixa da tabela do doc; quantizar só com foto aprovada.
4. **Item 6:** preço da compressão com carga baixa, 4K com várias camadas, redo.
5. **Provas de mutação** de `adormecida_tests.rs` e `dentro_do_wet_tests.rs`, e fecho.
6. Depois: [PROXIMA_FASE_2026-11-06.md](../PROXIMA_FASE_2026-11-06.md), itens 0–3.

**Dono 06/11: a Borracha na tinta morta está APROVADA pela foto** (`material_2026-11-06/foto_borracha_morta.png`). As constantes de `borracha_morta.rs` ficam; os passos 2, 3 e 5 do smoke ainda sem veredito.

## Relato do dono 06/11, Relief Eraser (depois da onda 2)

> «Ao trocar de modo e voltar para Relief Eraser, a borracha destrói as sombras simuladas do relevo apagando-as em áreas retangulares.» «Relief Eraser retira a tinta que tem relevo corretamente, paulatinamente, em várias passadas, mas a tinta ao lado feita com os outros modos a borracha retira de vez.»

Foto: [foto_relief_eraser_defeitos.png](material_2026-11-06/foto_relief_eraser_defeitos.png). Portões e sondas: `wetpaint/borracha_defeitos_tests.rs`. Provas de vermelho (cada cura retirada, gate vermelho): as cinco abaixo, medidas.

**Reprodução.** Sair e voltar sem pintar: 0 px, nos dois caminhos (relógio fixo e worker/quadro), em 8 portas, a 240×160 e 1400×900 comprimido: **isto elimina** o worker, a compressão, `chave_do_grao`/`applied` e `toca_tudo`. O defeito precisa de um traço de outro modo na camada: px fora do rasto, antes → depois da cura, relógio fixo e app iguais:

| cena | antes | depois |
|---|---|---|
| Aguada longe (a foto do dono) | 5 246 px, Δ 87, caixa (57,33)-(205,134) | 261 px, Δ 9 |
| Digital colado ao óleo | 3 091 px, Δ 212 | 78 px, Δ 71 |
| Digital e Impasto longe | 142 px, Δ 9 | igual (sem defeito) |

**Causas medidas (defeito A, são três).**
1. `composite.rs` lia `sombra` só com `vrow.is_none()`: com aguada na camada (o vidro, doc 48) a caixa recomposta escrevia a `base` sem a sombra, e a sombra assada na tela sumia num rectângulo. É também a «sobra medida» do doc (6 574 px). Cura: a sombra entra, e cobre o vidro por igual (`cobre`). Píxel que não muda não reescreve o selo.
2. Enterrar não punha os planos da óptica em dia: a caixa de píxeis (com Smooth Edges) é maior do que a de células, e a margem lia o `pigment` velho: o óleo enterrado voltava em colunas. Cura: `wet_planos_em_dia` no enterro e no undo com enterro (com `campo_cache` limpo: o enterro não anda a `revisao`).
3. A orla re-iluminada: a caixa do pincel seguinte refazia células cuja óptica o enterro mudou: uma linha e uma faixa escura que acabavam na borda da caixa (1 345 px com Δ ≥ 10). Cura: `Congelada` (`adormecida.rs`): a tela do enterro fica nas células cuja óptica o enterro mudou, enquanto a óptica e a `base` delas não mudarem. O undo devolve-a (`congelada_antes/depois`). Retém uma tela + 2 planos por sessão com enterro.
- Achado de leitura, com portão: adormecer deixava `otica.no_ar` armado para um compositor que morre com o worker; o tique nunca mais pedia imagem. Agora zera.

**Defeito B.** Causa confirmada: `apaga_a_base_sob_os_dabs` cortava a `base` pela cobertura; o enterro põe o traço de outro modo na `base`. Conserto: `Sulco` + `escreve_o_sulco` (`borracha_morta.rs`, extraído; constantes intactas) sobre a `base`, nas células sem tinta na grade no início do traço (`celulas_com_tinta`); nas com tinta segue o corte, que a tinta cobre. Miniatura e macros ficam com a lei de sempre (`base_na_lei_gradual`). Resolve o caso irmão do arquivo antigo (`a_tinta_velha_sai_aos_poucos_com_sessao_viva`).
- Pressão plena: Digital 255 → 166 numa passada, Watercolor 181 → 119, Impasto 255 → 166; ao papel em 9 passadas.
- Óleo vivo: `a_borracha_no_oleo_vivo_nao_muda`: **0 px** contra a lei antiga (foto, linha de baixo).
- Undo: `desfazer_a_passada_devolve_a_tinta_da_base`, tela e `base` ao bit.

**Portões mudados e reprovados.**
- `a_borracha_no_wet_apaga_a_tinta_antiga_de_cada_meio` (Bug #70) MUDADO: pedia 3 passadas a 2 %. Agora uma passada deixa > 40 % e treze chegam a < 2 %. O relato manda nas passadas.
- ⛔ `a_borracha_leva_o_traco_e_o_oleo_juntos` VERMELHO, não afrouxado: `(38, 0)` de 40 px depois de 3 passadas. Pressão variável (0,35–0,88), resto no rasto por passada: só óleo 40·40·18·5·0; traço por cima 40·40·38·35·33·31·30·27·24·21·19·17 (12 passadas). Causa: a pressão pesa à 8.ª (`EXPOENTE_DA_PRESSAO`): pouca pressão só amassa. **Decisão do dono:** a pouca pressão a tinta dos outros modos não chega ao papel na lei aprovada; a pressão plena chega em 9.
- ⛔ `traco_longe_nao_muda_a_borracha_e_fica_na_tela` VERMELHO, não afrouxado: Watercolor, zona do traço longe: 5 px, todos Δ 1, coluna x = 146 (`[200,83,70] → [199,84,70]`). Causa não isolada; a tolerância da zona de cima (Δ ≥ 10 < 300) já não é precisa, com 261 px Δ 9.

**Por medir / aberto.** Resto visível na foto: manchas escuras curtas na orla junto ao rasto e um fio fino no traço colado (78 px, Δ 71). Custo de `wet_planos_em_dia` a 4K não cronometrado. A compressão pelo fio real acima de `COMPRIME_A_PARTIR_DE` não correu com a cura: o portão comprime à força (`encolhe_ja`, o mesmo código), a 2400×1400. O caminho do app corre em teste com a ponte simulada (`set_compor_por_quadro`, worker, compositor), não com a pré-visualização da placa. `wet_ultima_imagem` agora também passa pela `Congelada`, sem portão próprio.

**Dono 06/11, smoke da rodada da Relief Eraser: OK.** Decisão: com pouca pressão a Borracha TAMBÉM chega ao papel, em mais passadas (hoje o expoente 8 só amassa); mexe na lei aprovada, logo foto nova ao dono. Isto decide `a_borracha_leva_o_traco_e_o_oleo_juntos`. Pergunta aberta do dono: «porque a tinta morre com UNDO?» Lido no código, não reproduzido: `HISTORY_DEPTH = 6` e o orçamento em `undo_vivo.rs` cortam a fotografia, e o undo sem fotografia acaba a sessão (`wet_volta_do_restauro`), como no 1.º traço da sessão.

## Relato do dono 06/11, fronteira tinta viva × tinta morta

> «A borracha está muito boa. Para a regra de matar a tinta continuar, precisamos melhorar o apagamento da área de transição entre uma tinta viva e uma morta. Na imagem o traço superior (tinta morta) e o inferior (tinta viva) ao serem apagados juntos forma uma fronteira pixelada ruim. Se possível, deve ser corrigida.»

Foto antes × depois: [foto_fronteira_viva_morta.png](material_2026-11-06/foto_fronteira_viva_morta.png). Portões e sonda: `wetpaint/borracha_fronteira_tests.rs` (`com_degrau` é a lei de antes, no mesmo processo).

**Causas medidas (são duas).**
1. Tinta morta: `apaga_a_base_sob_os_dabs` decidia por célula, `sett > 0 || susp > 0`. Célula com um resto de tinta, invisível, cortava a `base` de uma vez; a vizinha seguia a gradual. O papel aparecia na orla: pontos a razão 1, blocos de `ratio` px a razão 2 e 4. **Cura:** `PesoDaViva` (`borracha_na_base.rs`): a cobertura de pigmento no início do traço, amostrada como o composite amostra (`SampleU`), dá o peso de cada píxel: o que a tinta esconde sai de uma vez, o que ela deixa ver segue a gradual (`escreve_o_sulco` recebe os dois pesos). O corte usa a cobertura crua: alisada, passava do anel (12 px, `o_clique_da_borracha_fica_dentro_do_anel`).
2. Tinta enterrada: nas células não enterradas da orla, o enterro baixa o pigmento (pa 85) e a `Congelada` mantém a tela (alfa 254) com `base` 0. Ao refazer o píxel, a Borracha destapava o papel: pa 81, `base` 0, alfa 254 → 81 numa passada. Com o Digital da mesma cor há mais pontos (1 069 células vivas na faixa, 83 furos; vermelho: 794 e 3). **Cura:** `wet_orla_vai_para_a_base` (`adormecida.rs`): a cobertura que a tela mostra e a tinta perdeu vai para a `base`, em alfa contínuo. A 1.ª versão copiava o píxel inteiro e deixava contornos quadrados a razão 4 na foto; recusada.

**Hipóteses do pedido.** (a) confirmada. (b) só escala o defeito: o salto é o mesmo em razão 1, 2 e 4 e muda o tamanho do degrau. (c) confirmada na forma acima, sem ser «blocos com tinta assada»: na razão 1 também acontece. (d) eliminada: as candidatas tiraram os pontos sem tocar no grão; o p99 dentro da viva fica em 37 antes e depois.

**Candidatas, intercaladas no mesmo processo** (tinta morta, razão 1, 1 passada, salto p90 na fronteira; dentro da morta 5, da viva 19): lei de antes 65 · cobertura 28 · cobertura alisada 28, com a morta a subir para 8 · sem corte 28, muda a tinta por dentro. Escolhida a cobertura: a que menos muda.

| cena, 1 passada | papel na fronteira (px) | salto p90 / máx |
|---|---|---|
| morta, razão 1 | 103 → 0 | 65/85 → 28/48 |
| morta, razão 2 | 132 → 0 | 62/78 → 32/65 |
| morta, razão 4 | 189 → 11 | 65/78 → 31/42 |
| enterrada, razão 1 | 31 → 0 | 42/104 → 26/100 |
| enterrada, razão 2 | 57 → 1 | 58/70 → 46/70 |
| enterrada, razão 4 | 65 → 12 | 45/64 → 33/40 |
| caminho do app (worker, por quadro), morta, razão 1 | 103 → 0 | não medido |

O que fica: o salto de 28 é a diferença de nível entre a tinta viva e a morta, que saem em velocidades diferentes; igualá-las é mudar uma lei, decisão do dono. Os 11–12 px a razão 4 são buracos da própria tinta, visíveis antes da Borracha (lido na foto, não contado). Digital vermelho: 9 → 3 a razão 1, e **37 → 47 a razão 4**: a contagem mistura vermelho apagado com papel; não isolado. A orla exterior do traço vivo em blocos a razão 4, contra o papel, não é desta fronteira e não mudou.

**Não mudou, medido.** As 7 imagens de `sonda_borracha_morta` (óleo vivo 1, 2 e 3 passadas; tinta morta 1, 2 e 3; a cena de antes) são iguais em `cmp` às do commit `f335fcf315`, com o patch retirado da árvore e a mesma sonda: **0 px**, viva e morta. Com sessão viva e a morta longe: `a_morta_longe_da_viva_nao_muda` 0 px contra `com_degrau`. Sem sessão contra com sessão a mesma lei já difere (3 240 px, Δ 14, cena do portão): não é desta cura, causa não procurada. Undo: `desfazer_na_fronteira_devolve_tudo`, tela e `base`, nas duas cenas. Miniatura e macros: `sulco` nulo, o caminho de sempre.

**Provas de vermelho.** Sem a cura (`com_degrau` forçado): 3 portões vermelhos. Peso binário sobre o pigmento: os mesmos 3. O enterro sem guardar a orla: `a_fronteira_viva_morta_nao_destapa_o_papel` vermelho. Esta última correu na 1.ª versão da orla (píxel inteiro), não repetida na de alfa contínuo.

**Por medir.** Custo de `wet_orla_vai_para_a_base` a 4K: varre a tela a cada enterro, como os dois laços que já lá estão; não cronometrado. A cena do dono é a nossa reprodução, não o quadro dele: falta o olho dele.

**Verificado, por nome:** `borracha` 62 verdes e 2 vermelhos (os conhecidos, `(38, 0)` e 5 px, sem piorar) · `dentro_do_wet` 10 · `adormecida_tests` só os mesmos 2 vermelhos · `macro_track` 86 · `undo_tinta_viva` 9 · os dois alarmes de pincel verdes · `seam_wetpaint` 33/33 · fmt · clippy das 2 crates limpo.

**Dono 06/11: smoke da fronteira tinta viva × morta OK.**

## Onda do undo, da mão leve e do fecho (06/11, noite)

Commits: `2ad96106ad` (undo) · `325d15de9b` (mão leve + alfa) · `e8444946e7` (régua, foto) · `648108c5ee` (engasgo do óleo) · `b798272888` (régua). Fotos e patch em `handoffs/material_2026-11-06/`.

### 1. A tinta morria com o desfazer (`2ad96106ad`)
Reproduzido (`undo_tinta_morre_tests.rs`, `sonda_quando_o_undo_mata`): 12 traços morriam no 7.º undo, e o 1.º traço sem foto fazia o refazer deixar tudo morto. Causas: `HISTORY_DEPTH = 6`; o orçamento só contava as fotos (a 4K cabiam 4 passos; os registos, até 2,8 GB, não entravam na conta); a sessão nascia depois do pen-down.
Cura:
- `Engine::fundo_da_historia` (default 6; a sessão abre com ele).
- `teto_da_historia` = 6 × `history_budget_bytes`, contando registos + fotos. É o que seis passos de largura inteira já retinham a 4K (2,8 + 0,5 GB × 3,4 GB).
- `foto_em_tiles.rs` (`PlanoEmTiles`): tile igual ao bit à foto anterior = mesmo `Arc` (97 % iguais a 4K com óleo); compressão adormecida com `tira_os_tiles`/`poe_os_tiles`, cada tile uma vez.
- `wet_guarda_o_passo` no nascimento com traço aberto.
- `wet_enterra_sem_passo`: para lá do tecto só o que voltou atrás enterra, sem abrir passo, mais a tinta de alfa 0 (medido: 1 827 células, massa < 1 contra 570 no máximo).

Passos vivos, antes → agora: 4K curto 4 → 28 · 4K largura inteira 4 → 7–8 · 1080p curto 6 → 40+ · 1080p largura inteira 6 → 13. Fotos a 4K: 480–560 MB → ~150–180 MB. Pen-down igual ou melhor: 4K curto ~58 → ~52 ms; largo ~80 → ~68 ms (carga ~10). Registos não comprimidos em vida (candidato: zstd, ~10×).
Portões: `desfazer_fundo_nao_mata_a_tinta_que_ficou` · `o_primeiro_traco_desfeito_e_refeito_fica_vivo` · `o_tecto_conta_os_registos_e_as_fotos` · `para_la_do_tecto_so_o_traco_desfeito_acaba` · `para_la_do_tecto_o_traco_que_cruza_nao_deixa_nada` · `desfazer_depois_de_comprimir_nao_muda_nada`.
Foi reprodução por cenário: a cena exacta do dono não foi dada; falta o olho dele. **Achado não tratado:** `Engine::action_*` (wet/dry/clear) abrem registo sem passo no undo vivo se chamados na sessão; não verifiquei se o app as chama.

Redo (carga ~10): 1080p desfazer 45,8/60,0 ms, refazer 45,1/56,7; 4K desfazer 199/351, refazer 206/298.

### 2. Mão leve da Borracha (`325d15de9b`)
`PISO` em `borracha_morta.rs`: `tira = piso + (1-piso)·p^8`. Passadas até o papel, pressão 0,35 / 0,6 / 1:

| piso | tinta morta | traço por cima |
|---|---|---|
| 0 | >60 / >60 / 6 | >60 / >60 / 8 |
| 0,15 | 32 / 27 / 6 | 31 / 26 / 8 |
| 0,30 | 19 / 18 / 6 | 24 / 21 / 8 |

Sem piso, 30 passadas leves queimam a parede a preto e branco. **Ligado 0,30 (candidata B), aguardando o veredito do dono pela foto** `material_2026-11-06/foto_borracha_pouca_pressao.png`. Óleo vivo: a lei não passa pelo piso. `a_pressao_plena_nao_muda` 0 px. `a_borracha_leva_o_traco_e_o_oleo_juntos` verde em 8 passadas (rampa).

### 3. Os 5 pontos (`325d15de9b`)
Não era a aguada. `escreve_o_sulco` fazia floor de `a0/255*255`; em f32 164/255·255 < 164, e o alfa 164 perdia um ponto onde nada saiu, daí a coluna única (x = 146). Agora a conta é sobre o alfa inteiro e `NADA_SAI` (1e-3) ignora o rasto numérico. `traco_longe_nao_muda_a_borracha_e_fica_na_tela` verde.

### 4. O resto da Borracha, sem mudar nada (`e8444946e7`)
Foto ao dono: `material_2026-11-06/foto_resto_da_borracha.png`. Vê-se: manchas escuras curtas e faixa escura no contacto óleo × Digital sob a Borracha; fio claro no contacto; contornos fracos a razão 4. Digital vermelho 37 → 47 é artefacto da contagem: dos 47, 36 são comuns às duas leis e 11 só da de hoje, e são mistura de vermelho apagado, não papel (ex. [140,91,108] × velha [147,89,101]); a foto mostra hoje melhor (o degrau sumiu). Causas das manchas e do fio não isoladas além do que está acima; aguardam o dono escolher.

### 5. O engasgo de adormecer após o traço — RESOLVIDO no Dabber, não no Wet Round (`648108c5ee`, `b798272888`)
Medido, ms mín/mediana, intercalado, carga ~10:

| | antes | agora | main |
|---|---|---|---|
| 1080p Dabber logo | 348/379 | 0,3/0,4 | 376/376 |
| 4K Dabber logo | 1721/1788 | 0,5/0,5 | 1817/1885 |

Causa: `wet_ultima_imagem` recompunha a grade inteira quando `otica.pendente()`; com a água a andar «assentada» quase não existe (passos sobem ~20 por 30 quadros durante 4 s). A base «Acaba» enganava: com a tela cheia o main paga o mesmo. Cura: o sujo que saiu do motor volta (`o_sujo_por_mostrar_volta_ao_motor`, `Correio` em `offthread_otica.rs`) e a saída compõe só isso; tela inteira só com véu ou varredura do Real Oil por acabar. Portões em `adormecer_so_o_que_falta_tests.rs`.

⚠️ **Wet Round NÃO curado:** 1080p 89/333 (main 381/409), 4K 245/1282 (main 1303/1382), cargas 13–14. Não é regressão. Medido: compor por linhas não alcança as metas (ótica 78–94 % do tempo, antes da pista de sujo; previsto 1080p 126/281, 4K 541/1160). Patch das sondas: [regua_censo_do_sujo.patch](material_2026-11-06/regua_censo_do_sujo.patch). Opções: imagem atrasada (decisão do dono) · baixar o custo da ótica · aceitar.

Consertos no enterro (ms): 4K enterrar 90/105 (sem a orla 74/96), planos caixa do traço 61/67, grade inteira 1603/1622, congelar 3,7; 1080p enterrar 35,5/37,3 (sem orla 31,5/34,0), planos 78/84, inteira 363/371, congelar 0,7.
Sombra com a compressão sozinha, 4K: comprimiu em 87 ms, trocar de volta 0,0 ms, 0 px perderam a sombra.
Várias camadas, 4K, 4 camadas de 6 passadas, carga 6–9: adormecer 0 ms, comprimir tudo 959 ms, somam 575 MB, 4 de 4 ficam; traço Digital 15,3/16,1 × 14,8/16,2 ms sem tinta (igual); acordar 740 ms. ⚠️ Uma rodada anterior morreu por OOM (tecto 24 G, várias pinturas 4K vivas no mesmo processo de teste; não explicado por inteiro; o controlo só com `on_tick` mostra ~250 MB por traço nos dois casos).

### Pendente do dono
Veredito da foto da mão leve · o que fazer com o resto da foto · a aguada · Wet Round (opções acima) · o passo de troca de modo sem veredito próprio. **Provas de mutação: outro agente, ainda a correr — não dar como feitas.** Depois de integrar: a opção de manter a tinta viva ao salvar (mapa no item 5 acima, «Próximo»). Suíte completa só na integração.
