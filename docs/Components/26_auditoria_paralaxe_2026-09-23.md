# 26 — Auditoria da paralaxe (plano 24, W1–W7) — 2026-09-23

> Pedida pelo dono depois do smoke aprovado. **Quatro lentes independentes, em paralelo, nenhuma a
> consertar** (`/pd-auditoria`): (1) correcção da lei · (2) costura de UI · (3) persistência, undo,
> integração e cenas · (4) gates, tectos e custo. Achados confirmados por **duas ou mais** lentes
> estão marcados `×N`. Os dois P0 de maior alcance (§1.1 e §1.2) foram re-conferidos por leitura
> directa da ponte depois das lentes.
>
> ⛔ **Os §1–§3 são a LISTA como a auditoria a entregou, e ficam intactos.** A cura — ordem do dono,
> *«vamos corrigir tudo»* — está no **§5**, item a item, com o gate que a prende e a mutação que o
> prova.

## §1 — P0: defeitos que o artista vê

### 1.1 A REPETIÇÃO prende o fundo ao MUNDO e não à VISTA (lente 3; re-conferido)

`parallax_bridge.rs` envolve o DESLOCAMENTO `d = c·(1−k)` por ladrilhos inteiros ⇒ `|d| ≤ tile/2` ⇒
a camada nunca se afasta mais de meio ladrilho da pose **autorada no mundo**. Quem tem de ficar
limitada é a posição **relativa à câmera** (`autorada + d − c`), que é o que o oráculo faz (a tabela
da sonda do Godot já o mostrava: ecrã `−564` estável contra o nosso `−820 → −1332`).

- Cena `=1`: árvores visíveis por posição de câmera `0/20/30/60 m` ⇒ `5/4/2/0`. O céu também some.
  O roteiro promete *«as árvores e o céu NUNCA acabam»* ⇒ **a cena ensina o contrário a partir de
  ~30 m** (§5.0: pior que ausente).
- ⚠️ A «divergência declarada» da W2 (*«a janela é nossa»*) descreve mal o que acontece: o
  **referencial** está trocado, não só a janela.
- Gates verdes por cima: `a_fileira_que_repete_cobre_a_vista…` e `uma_nuvem_que_deriva_e_repete_nao_foge`
  correm com a câmera em `0`, onde mundo e ecrã coincidem. A régua gravada na memória (`4673726c4`,
  `|p − autorada| ≤ tile/2`) **é a grandeza errada**.
- Gate em falta: a ponte com a câmera em `0..70 m`, a fileira tem de cobrir `[c − meia, c + meia]`.
- Direcção da cura: envolver `e = d − c` (o relativo ao ecrã) e somar `c` de volta. `k = 0` fica
  colado ao ecrã e `k = 1` fica no mundo, a repetir.

### 1.2 Largar a condução COZE a pose deslocada no DOCUMENTO (×3: lentes 1, 3, 4)

Três saídas mudas da ponte: `vista == None` (`return 0`), `k` neutro (`continue`) e dolly a
atravessar (`continue`). Por leitura há mais duas: tirar o `ScrollFactor` e anexar `UiCanvas`. Em
todas o `PreviewDrive::settle` só **esquece** a entrada. Quem devolve o autorado é o
`release_to_authored`, e a ponte nunca o chama.

- A captura seguinte grava a pose deslocada (passo de undo + `Ctrl+S`). Ao voltar a conduzir,
  desloca **duas vezes**. Medido: `k 0,5`, câmera em `400` ⇒ `x = 200` ⇒ `k → 1` ⇒ fica `200` ⇒
  `k → 0,5` ⇒ **`400`**.
- Gestos do produto que o disparam: desligar ou apagar a câmera do jogo (`Active`), pôr `k = 1`,
  dolly `≥ 0,5` com uma camada `k = 2`.
- Doc falso: o cabeçalho da ponte diz *«o `settle` devolve-lhe a pose autorada no mesmo quadro»*.
- Gates verdes por cima: partem de um ledger virgem e nenhum chama `settle`.
- O gate prometido no plano W1 (*«o ledger não deixa passo de undo»*) **não existe**.
- A ponte do HUD tem a mesma frase falsa (família pré-existente).

### 1.3 O Dolly de volta a `0` deixa os fundos encolhidos (×3: lentes 1, 2, 4)

Com `esc == [1,1]` escreve-se `era.scale`, a escala VIVA, que ainda é a do dolly anterior. O
`still_driving` mantém a condução, logo nada a devolve. Medido: `k 0,5`, dolly `0,5 ⇒ 0,667 ⇒`
dolly `0` ⇒ fica `0,667`.

- Gate verde por cima: `com_dolly_zero_a_saida_e_byte_identica` corre com dolly `0` desde o início.

### 1.4 O Dolly escala à volta do PIVÔ do objecto, e não do centro da vista (lente 1; re-conferido)

A lei de pinhole dá `X = c + (P + d − c)·esc` para cada ponto da camada. A ponte translada o pivô e
escala **em torno dele**, com erro `(esc − 1)·(pivô' − c)`. Medido: céu autorado em `y = 3,2`,
`k 0,12`, dolly `0,9` ⇒ código `3,2`, geometria `0,359`.

- Na cena `=2` os fundos têm pivô em `y = 3,2 / 1,2 / −0,9` e **encolhem no lugar** em vez de
  convergirem para o centro da vista.
- Gates verdes por cima: pose e câmera na origem, onde as duas leis coincidem.
- ⚠️ O plano §2/W5 também não tem o termo de posição: **o erro está no MODELO**, não só no código.

## §2 — P1: defeitos latentes

| # | achado | lentes | mecanismo em uma linha |
|---|---|---|---|
| 2.1 | O **autorado deriva em `f32`** enquanto a câmera anda | 3 | `era − (escrito − memo) ≠ memo` em `f32`; o `driven()` regrava a cada quadro. Medido: `5 995/6 000` quadros diferentes ⇒ passo de undo espúrio por quadro com input, ficheiro alterado |
| 2.2 | Um **`.ph2dproj` v128 com `GameCamera` perde o mundo** | 3 | o blob de 5 campos não descodifica com o `dolly` apendado; o `migrate_v128_to_v129` não o conhece, e o `snapshot_to_world` pára na 1.ª linha que falha. ⚠️ A regra do degrau 148→149 já o prescrevia |
| 2.3 | **Repetição e limites ignoram a escala do dolly** | 1, 3 | período no ecrã `tile·esc` contra correcção por `tile` ⇒ a costura salta; a cerca é medida em metros autorados |
| 2.4 | **O painel mente ou cala-se** | 1, 2, 4 | a queixa `Neutra` com deriva activa · câmera inactiva sem queixa (`tem_camera_do_jogo` conta inactivas, a ponte usa `active_camera_of`) · dolly a atravessar sem queixa · `k` NaN mudo · pré-visualização desligada (o padrão) sem aviso |
| 2.5 | A **paleta oferece Repeat/Limits/Drift sem `requires: ScrollFactor`** | 2 | anexado num objecto sem `Parallax`, fica inerte, invisível e irremovível |
| 2.6 | **Números digitados não são travados**, e o clamp do dolly vive em dois sítios | 2, 4 | `commit_number_buffer` não aplica a faixa; o applier da paralaxe escreve cru (Repeat `−5` fica gravado); `(−1; 0,9)` está em `populate_camera` **e** em `camera_inspector` sem constante partilhada (a mutação `0,99` sobrevive) |
| 2.7 | **Segundo condutor** no mesmo objecto (timeline, física, script, tween) corrompe o autorado | 1, 4 | a timeline e a física correm depois da paralaxe e antes do extract; não há guarda nem aviso |
| 2.8 | O **snapshot do Inspector varre o mundo** a cada quadro | 4 | `iter_entities().any(GameCamera)`: `529 µs` a 100 k entidades (o `active_camera_of` custa `≈1 µs`) |
| 2.9 | O gate de custo é da **família de flakes** | 4 | `o_custo_do_passe_e_linear…`: razão de dois relógios de dezenas de µs; reprovou `1/25` a `load 26–36`. A cura é contar trabalho |

## §3 — P2: régua, docs e processo

- **Arnês de mutação:** `4/60` âncoras já não casam no HEAD ⇒ o *«60 de 60»* do handoff não se
  reproduz. Refeitas, as quatro sangram. **Três mutações novas SOBREVIVEM:** `floor` só em Y (todas
  as fixturas de repetição são `[tile, 0]`) · dolly ignorado com `k < 0` · clamp do applier `0,9 → 0,99`.
- **A deriva lê o relógio antes dos intents de scrub/rewind** ⇒ um quadro de atraso num scrub.
- **Velocidade não finita** (ficheiro/script) vira `Transform` NaN; `δ = 1` fixa a escala autorada
  em `0` para sempre; um eixo que atravessa congela o outro.
- **Docs colados ao item errado** por inserção de `mod`: `render_loop/mod.rs` (*«A secção CAMERA…»*),
  `ph2d-panel-inspector/src/lib.rs` (o doc do RAY passou para `event_parallax`/`sync_parallax`) e
  `ph2d-app-components/src/lib.rs`.
- **Handoff:** são `37` commits e não `36`; não declara a folga que consome na catraca da shell
  (`196 692` contra `196 990` ⇒ `298` linhas para a rodada).
- **Plano §4 W5** ainda fala de `focal_distance`; link de doc partido em `scroll_motion.rs`
  (`parallax_w4_probe_tests`).
- **UI:** a deriva sem unidade (`Unit::MetersPerSecond` existe); `"X"`/`"Y"` e `format!` de frase
  fora do i18n em `sections/parallax.rs`; limites de fábrica `[0,0]/[0,0]` inertes sem voz; a
  semente pode ficar velha depois de um blur (herdado das irmãs).
- **Gates de texto:** o `frame_text` não tira comentários (um literal num comentário passa); o nome
  `…_e_so_recebe_o_centro` afirma o contrário do que o gate mede.
- **Cena `=2`:** o cabeçalho diz *«a única coisa que se mexe é uma pista»* e o céu deriva
  (`cenario()` partilhado).
- **Não há gate de ida-e-volta (gravar → abrir)** para os quatro componentes nem para `dolly ≠ 0`.
- `scripts/schema-recount.py` só corre dentro de um conflito de rebase.

## §4 — Verificado sem defeito

- A lei `autorada + c(1−k)` é a mesma na crate e na ponte, simétrica em X/Y.
- `k < 0`, `k > 1` e `k = 0` dão a fórmula esperada.
- Há guarda em `k·δ = 1`; `min ≥ max` desliga o eixo; uma região estreita vai para o centro sem pânico.
- A ponte e a câmera usam a mesma `active_camera_of`, e a rotação é preservada.
- Precisão `f32` com o centro em `1e5`: o erro é o ULP do próprio centro.
- Não há quadro de atraso em relação à câmera.
- Os 10 campos `Parallax` são pintados, estão registados e estão vivos sob o dedo.
- O fio evento → barramento → applier escreve o par e o eixo certos.
- Multi-selecção edita só o primário e di-lo; há um passo de undo por gesto.
- A mudança do `inspector_camera` é um rename limpo (6/6 testes); rewind e `RestartRun` voltam ao tique 0.
- O `dolly` é o último campo da `GameCamera`; os três registos batem `107/108/108`.
- O custo da ponte é `O(camadas)` (não percorre filhos).

## §5 — A CURA (23/09, *«vamos corrigir tudo»*)

⭐⭐⭐ **A lei passou a ser UMA função por eixo** ([`scroll_factor::saida_eixo`](../../crates/ph2d-ecs/src/scroll_factor.rs)):
`saída = c + esc·(autorada + deriva − k·confinado)`, com a repetição a envolver o parêntese (a
posição **no ecrã**) e o dolly a escalar à volta do **centro da vista**. Sem dolly e sem repetição o
ramo é a soma de sempre **ao bit** (gate `sem_repeticao_nem_dolly_a_saida_e_a_de_sempre_ao_bit`),
logo a paridade da W1 com o oráculo não foi re-medida — foi **preservada por construção**.

| # | cura | gate (mutação que sangra) |
|---|---|---|
| **1.1** | a repetição envolve `deriva − k·confinado` (o relativo à VISTA) | `a_repeticao_fica_presa_a_vista_e_nao_ao_mundo` · `a_repeticao_em_y_fica_presa_a_vista` · ⭐ `andando_muito_a_fileira_que_repete_continua_a_cobrir_a_vista` (a CENA, com a câmera a andar `0 → 70 m` pela porta do produto) |
| **1.2** | a ponte e a do HUD **largam** quem deixam de conduzir (`release_to_authored` sobre o `driven_by` do ledger), nas cinco saídas | `quem_deixa_de_ser_conduzido_volta_a_pose_autorada` · `um_canvas_que_perde_a_camera…` |
| **1.3** | a escala escrita é `autorada × esc`, nunca a viva | `o_dolly_de_volta_a_zero_devolve_a_escala` |
| **1.4** | escala à volta do CENTRO DA VISTA; o **plano §2/W5 corrigido** (o erro estava no modelo) | `o_dolly_escala_a_volta_do_centro_da_vista` (oráculo = a pinhole escrita por extenso) · `o_dolly_leva_o_fundo_para_o_centro_da_vista` |
| **2.1** | ramo EXACTO «nada mudou» ⇒ o autorado sai do memo sem aritmética | `o_autorado_nao_deriva_com_a_camera_a_andar` |
| **2.2** | `GameCameraV128` congelado + `migrate_game_camera_blobs` no `migrate_v128_to_v129`; a linha do load diz quantas | `a_frozen_v128_camera_migrates_and_the_world_restores` (com o CONTROLO: o tipo vivo não lê os cinco campos) |
| **2.3** | a meia-vista vista pela camada é `h(1/esc − 1 + k)/k`; a repetição é no espaço anterior à escala | `com_dolly_a_cerca_mostra_a_mesma_faixa_de_conteudo` · `a_cerca_com_dolly_usa_a_meia_vista` |
| **2.4** | `CameraDoJogo { Nenhuma · Desligada · Activa }` pela porta `active_camera_of`; queixas do dolly a atravessar, do outro condutor, do `k` não-finito; notas da pré-visualização e da cerca inerte; a deriva com `m/s` | `as_queixas_seguem…` · `o_neutro_com_deriva…` · `o_dolly_que_atravessa_e_o_outro…` · `as_notas_da_pre…` |
| **2.5** | as três irmãs `requires: ScrollFactor` no catálogo | `as_irmas_da_paralaxe_requerem_o_factor` |
| **2.6** | o applier trava ao domínio da lei (finitos; ladrilho `≥ 0`); a faixa do dolly é UMA porta (`DOLLY_MIN`/`DOLLY_MAX`) | `o_applier_trava…` · `a_faixa_do_dolly_e_uma_porta_dentro_do_dominio_da_lei` — ⚠️ a mutação `0,9 → 0,99` que sobrevivia deixou de ser **exprimível**: não há literal para mutar num sítio só |
| **2.7** | o painel NOMEIA o outro condutor (`drives_other_than` no ledger) | `o_dolly_que_atravessa_e_o_outro…` |
| **2.8** | o instantâneo lê `active_camera_of` + `camera_count` (consulta de UM componente), nunca `iter_entities` | coberto pelo gate da queixa |
| **2.9** | o gate de custo CONTA TRABALHO: a estrutura (consulta que EXIGE `&ScrollFactor`, zero `iter_entities`) + o trabalho (`10` conduzidos com `20 000` objectos sem o componente) | `o_passe_e_uma_consulta_filtrada_e_conta_so_as_camadas`; o relógio fica no irmão `#[ignore]` |
| §3 arnês | as âncoras refeitas; **modo SECO** (`SECO=1`) que conta as âncoras sem compilar | o próprio arnês |
| §3 scrub | a `fase_paralaxe` passa para DEPOIS do `fase_timeline_drain` (é ele que aplica scrub e rebobinar) | o gate de ordem do quadro ganhou a metade `dreno < paralaxe` |
| §3 `NaN`/`δ = 1`/eixo | deriva não-finita é inerte por eixo · `δ ≥ 1` é recusa (`None`), nunca escala `0` · cada eixo decide sozinho | `uma_deriva_nao_finita_e_inerte` · `a_camera_no_plano_focal_e_recusa` |
| §3 docs | os docs colados ao item errado (três ficheiros, e dois deles **pré-existentes** na secção RAY) · o link partido · o cabeçalho do `scroll_repeat` e da cena `=2` | — |
| §3 texto | os gates de texto da fase leem **sem linhas de comentário** · o nome `…_e_so_recebe_o_centro` → `…_e_recebe_o_rectangulo_inteiro` | mutação «a chamada comentada passa por viva» |
| §3 ida-e-volta | gravar → abrir os quatro componentes e o dolly, pelo registo real | `gravar_e_abrir_devolve_os_quatro_componentes_e_o_dolly` |

⏳ **Fica fora, com o motivo:** o `commit_number_buffer` genérico não trava a faixa do widget (é da
casa inteira, não desta linha — aqui quem trava é o applier, que é a última porta antes do
documento) · o `scripts/schema-recount.py` só correr dentro de um conflito (ferramenta do
integrador) · a semente velha depois de um blur (herdada das irmãs, pré-existente).
