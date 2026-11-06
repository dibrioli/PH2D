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
