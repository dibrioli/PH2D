# 123 — CICLO 14: os oito abertos que o ciclo 13 deixou, num ciclo

> **Ordem do dono, 2026-10-08:** *«para os demais em aberto … resolvê-los em um único ciclo»* — e, no
> mesmo dia, a DECISÃO: **o tecto de `32 768` por nó FICA** e o tecto por dispositivo está FORA. A lista
> é a do [handoff de continuação de 08/10](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-08_CICLO_14.md).
> Base: `line/motion-value` sobre o `main` `2a925859d` (o ciclo 13 por integrar). Cada item foi
> auditado contra o CÓDIGO antes de construir; o estado medido vem da rodada ANTES (§9), uma só,
> intercalada onde havia variantes.

## §0 — O estado medido, o desenho e o kill-criterion de cada item (escritos ANTES)

| # | o que a lista dizia | o que o código disse (auditoria) | desenho | kill-criterion |
|---|---|---|---|---|
| 1 | a imunidade do `pulse.beat` a FMA não tem régua (a mutação «`quantiza` = identidade» sobreviveu na RTX) | confirmado: os dois gates de paridade só mordem se a placa FUNDIR, e a fixtura de `64` linhas × `91` tiques nunca cai no fio | **régua de CPU** que não depende da placa: `frac − i·sc` contra `(−i).mul_add(sc, frac)` (FMA verdadeiro) para TODA `i < 2^17` nas fracções do fio do empate, com o `sc` do PRODUTO (`compasso`) — igualdade exigida; e o CONTROLO: com o `sc` CRU a mesma varredura TEM de achar divergência | a mutação 3 do ciclo 13 SANGRA no gate de CPU; e a paridade sem quantizar corre na iGPU (`radeon_icd`) e regista se ela funde |
| 2 | `write_the_value_figures` coze a `=117` com um `Cook` cru e lê `0` valores | confirmado no código: `colher` lê o `size` do SINK, que desde 07/10 é a saída do CARIMBO (`source.shape` só existe com as membranas publicadas) | ~~colher antes do carimbo~~ — na `=117` o carimbo vem PRIMEIRO e o valor age depois (§2); ⇒ a porta do produto: as membranas publicadas a cada quadro (`publish_all`, o 1.º passo do `coze_o_quadro`) e o `Cook` da bomba, desenhando a PROPORÇÃO saída/estrela | as quatro figuras regeneradas e a contar a MESMA história das de antes (foto lado a lado), e o gate do item 3 a compará-las com o disco |
| 3 | as figuras de parâmetros estão velhas (`181` contra `1000000`; o `Shape` do `motion.grid`) | **9** geradores (`write_the_*` ×6, `dump_*_figures` ×3), todos `#[ignore]`, todos a escrever no disco sem nenhum gate que compare; `params_campos.html` e `params_sim.html` são gerados e NENHUM tutorial os inclui | cada gerador parte-se em PRODUZIR (`Vec<(nome, texto)>`) e ESCREVER; um registo único dos nove; um gate compara o produzido com o disco, e um censo de texto exige que todo `fn write_the_*` / `dump_*_figures` esteja no registo | o gate VERMELHO sobre o disco de hoje antes de regenerar; verde depois; os PDFs refeitos |
| 4 | cinco sondas `#[ignore]` reprovam sozinhas | quatro herdaram constantes que mudaram em setembro (`d60a8398c`: `LADO_MAX_DE_GRELHA = 181`; `47e991174`: o `SO_COM_FORMA` passou a ligado por omissão); a quinta é a cena `=53`, cujo `run()` chama `advance_tick` ANTES do `cook` — a ordem que o próprio doc do `advance_tick` proíbe (*«after the frame's cook(s)»*) — e coze o `motion.delay` DUAS vezes por tique | cada sonda curada pela lei de hoje (números derivados das constantes, não literais) ou aposentada com o porquê medido; o `run()` da `=53` na ordem do produto, e o gate do tecto SAI do `#[ignore]`; varredura da FAMÍLIA (todo laço de teste que avança antes de cozer) | as cinco verdes pela lei de hoje (ou retiradas com número); o gate do tecto vermelho com a ordem invertida (mutação) |
| 5 | a suíte da placa do `--lib` não cabe na fatia | `327` `#[ignore]` em `136` ficheiros, classe só pela RAZÃO (não filtrável pelo libtest nem pelo nextest); nenhum script nomeia o gate de placa | razões com PREFIXO de classe (`placa:` · `sonda:` · `gerador:`), um censo que reprova razão sem classe, e um script que corre SÓ os `placa:` | o gate de placa inteiro dentro da fatia, medido antes e depois; o número de `placa:` corridos = o do censo |
| 6 | SIGSEGV da RTX ao SAIR do binário `it` do `ph2d-gpu-cook` | `40` `static OnceLock<Option<GpuContext>>`, um por ficheiro, seguram o dispositivo até ao `exit` — a família do doc 121 §9.18 (F) | UM contexto partilhado pelo binário, solto num `atexit` registado depois de o driver carregar (corre ANTES dos destrutores do driver) — ⚠️ medido: o contexto ÚNICO basta e o gancho saiu (§6) | `N` corridas intercaladas antes/depois com o `rc` e a carga ao lado: `139` some |
| 7 | a `=27` esvazia a tela em `~4 s` | **medido** (`sonda_onde_a_cascata_esta`): a cascata NÃO desce — o vento (`angle 90`) e o `curl` levam-na para CIMA e para a DIREITA; à vista `8/11` aos `3 s`, `0/30` aos `5,5 s`; câmera por omissão (`height_world = 10`) | um `force.attractor` ao centro e um `force.drag` entre o `curl` e o `step` (os dois com kernel: a cena continua inteira na placa); variantes medidas numa rodada (§7) | `≥ 90 %` à vista a `2`, `4`, `6` e `8 s` (gate `the_cascade_stays_in_view`) e na foto, com os gates da cena verdes |
| 8 | `PH2D_PAN_DIAG` não lê o MUNDO das cópias do dispositivo | confirmado: `note_device` guarda só contagens | com a sonda ligada, a 1.ª cópia viaja pela leitura dos cartões que já existe | custo ZERO com a sonda desligada (régua de texto, como a do `tap_sem_espera`) e o mundo da 1.ª cópia igual nas duas rotas |

## §1 — O `pulse.beat` e o FMA: uma régua que não precisa de a placa fundir

`the_beat_index_is_the_same_fused_or_not` (`ph2d-node-pulse-beat/src/tests.rs`). A placa pode fundir
`frac − i·sc` (a WGSL permite contrair); a CPU arredonda duas vezes. O FMA da CPU (`f32::mul_add`) é
o FMA verdadeiro, logo a pergunta *«o `⌊⌋` muda se fundir?»* responde-se sem placa nenhuma. A
varredura: as seis fixturas `(period, phase_stagger)` (as duas da paridade da placa e quatro razões
quaisquer), toda `i < 2^(24 − SC_BITS) = 131 072`, e para cada uma as três `f32` em volta de
`frac(i·sc)` exacto — o fio do empate daquela linha.

**Medido (pré-voo, 08/10):** verde com o `sc` do produto; com a mutação «`quantiza` = identidade»
SANGRA já na **linha 5** da fixtura da paridade (`period 0,0537`, `stagger 0,0031`, `frac 0,2886406`).
⇒ o `64 × 91` da paridade da placa só não a apanhava porque nenhum tique caía nesse fio.

A **régua na PLACA** fica também: `FIXTURA_DO_FIO_DO_FMA` (`ph2d-node-pulse-beat`) é um
`(period, phase_stagger, offset)` em que, com o `sc` cru, a linha `11` muda de ciclo no tique `3`
se a placa fundir; o gate `the_device_fixture_sits_on_the_fma_edge` prova-o na CPU, e
`parity_pulse_beat_on_the_fma_edge` (`ph2d-gpu-cook`) corre a paridade sobre a MESMA constante —
numa placa que funde, a mutação sangra também ali. Achada pela `sonda_a_fixtura_do_fio_do_fma`.

**Medido na rodada DEPOIS (§9):** produção verde nas duas placas (`3/3` paridades do `pulse.beat`, RTX
e iGPU `radeon_icd`); com a mutação «`quantiza` = identidade» o gate de CPU reprova (`2` gates) e a
paridade `parity_pulse_beat_on_the_fma_edge` reprova **nas DUAS placas** — ⇒ a RTX e a iGPU FUNDEM
`frac − i·sc` neste kernel. A mutação 3 do ciclo 13 só sobrevivia porque a fixtura nunca caía no fio.

## §2 — O gerador do tutorial 6: o carimbo vem PRIMEIRO

A `=117` carimba as estrelas e SÓ DEPOIS o valor mexe no tamanho (`pano → carimbo → scale/strobe →
output`). Colher a montante do carimbo mediria pontos sem o valor; o `Cook` cru lia `0` peças
porque a `source.shape` só existe com a membrana publicada. A cura é a porta do produto:
`MotionState` + `motion_externals::publish_all` a cada quadro + o `Cook` da bomba. ⚠️ **E a
primeira figura regenerada saiu quase RECTA**, com os gates do gerador verdes (`49` valores contra
`4`): o `size` de uma cópia carimbada é o raio da estrela (`0,16`) vezes o efeito. ⇒ desenha-se a
PROPORÇÃO saída/carimbo (`1 ± amplitude`), e a foto lado a lado das quatro figuras (antes × depois)
conta a mesma história. *Um gerador com asserções verdes pode escrever uma figura que não ensina
nada — a régua final é OLHAR.*

## §3 — As figuras e as tabelas: um registo, três gates

Os nove geradores partem-se em `figuras() -> Vec<(nome, texto)>` (as asserções da história ficam
lá) e no escritor `#[ignore = "gerador: …"]`, que é uma linha. O módulo
`motion_bridge_tutorial_figuras.rs` tem o registo `GERADORES` e três gates:

- `as_figuras_dos_tutoriais_sao_as_que_os_geradores_produzem` — tudo o que cada gerador produz
  está no disco AO BYTE. **Vermelho sobre o disco de partida com `7` figuras** (as três tabelas —
  `digitável até 1000000` contra `181`, `Stagger` contra `Phase Stagger`, a nota do `Shape` — e as
  quatro do tutorial 6); verde depois de regenerar. Custo em DEBUG: `0,5 s`. Os SVG das outras
  oito famílias saem ao byte iguais em DEBUG e em RELEASE.
- `todo_gerador_de_figuras_esta_no_registo` — censo de texto, nos dois sentidos (achou `9`).
- `toda_tabela_gerada_e_incluida_e_nenhuma_e_colada` — cada `params*.html` é INCLUÍDO por um
  tutorial e nenhum tutorial tem uma cópia COLADA. ⚠️ Os tutoriais `04` e `05` tinham a tabela
  colada à mão (a do `05` já velha: sem a tabela do `Shape`, `Strength 0 a 40` contra `-40 a 40`);
  passaram a `<!--#include ../fig/params_*.html-->`.

**A família alargou-se ao TEXTO**: o tutorial `01` mandava clicar no `Shape` do Grid, que saiu
em `54ba1cbaa` (19/09). Os tutoriais `06`–`09` tinham o gate *«toda linha que o tutorial nomeia está
no cartão»*; os `01`–`05` não — ganharam-no neste ciclo (§3.1).

### §3.1 — Os tutoriais 01–05 ganham o gate do cartão, e ele achou sete passos impossíveis

`motion_state_tutoriais_01_a_05_tests.rs`: um gate por tutorial (`every_row_the_<grupo>_tutorial_names_is_on_the_card`),
no molde dos `06`–`09` e mais apertado — abre cada secção antes de ler, troca o modo que o passo
troca, confere as opções de uma escolha, lê os cartões DESENHADOS (`card-demo`) contra o cartão real
e o estado de nascimento de cada gaveta. `16 · 74 · 23 · 7 · 22` linhas conferidas. Os sete passos
velhos, todos curados no TEXTO: o `Shape` do Grid (mock, exemplo de «escolha», «Vá além» → a
`Density Falloff` do Scatter), «três cartões» onde a `=1` abre quatro, `Maximum` → `Max`, `Ticks` →
`Rise` (e o `Fall`), e duas gavetas «abra a secção» que já nascem ABERTAS (clicar fecha-as). ⭐ E a
FAMÍLIA seguinte: `01`/`02` prometiam «dois milhões» (o tecto por nó é `32 768`, decisão do dono) e
`03`/`07`/`08`/`09` citam populações acima dele — ⏳ reescritos com números MEDIDOS.

## §4 — As sondas que reprovavam sozinhas, e a família «avança antes de cozer»

| sonda | porque reprovava | cura |
|---|---|---|
| `audit_the_stamp_frame_split` · `audit_the_corner_radius_cost` | o arnês pedia `320 × 320` (`102 400`) e o `motion.grid` corta em silêncio a `181²` desde `d60a8398c` | o arnês pede `LADO_MAX_DE_GRELHA` e o controlo exige `LINHAS = 181²` (derivado) |
| `audit_what_the_camera_can_see` | o controlo era o literal `~39` (`LADO_N = 300`); hoje `23,4` | o campo esperado é `(LADO_N − 1)·VAO`, ± um vão |
| `measure_the_source_group` | pedia `PH2D_MOTION_SO_COM_FORMA=0` ao PROCESSO e recusava-se sem ele | a lei desligada na BOMBA da sonda (`define_a_lei(false)`, a porta dos arneses de cozimento) |
| `the_ceiling_is_honoured_on_every_tick_including_the_turn` | o arnês da `=53` chamava `advance_tick(t)` ANTES do `cook(t)`: o `motion.delay` cozia duas vezes por tique (`0,1678` contra `0,0800`) | a ordem do produto; o pior passo é `0,0800` ao dígito, e o gate SAIU do `#[ignore]` |

⭐ **A ordem invertida era uma FAMÍLIA**: `45` laços (`23` no `ph2d-app-motion`, `22` nos `tests/it`
do `ph2d-node-registry-init`) cozinham cada nó sequencial DUAS vezes por tique — um passo real e
um re-cozimento com `dt = 0` —, ou seja mediam outro programa que o app. Na ordem do produto, quatro
gates ficaram vermelhos, e cada um era um número calibrado sobre o passo duplo:

| gate | antes → depois | leitura e cura |
|---|---|---|
| `spring_ceiling::far_above_…_silent_failure` | `Snaps` → `Explodes` | ⭐ **defeito de produto**: a guarda de NaN da mola corria só no INÍCIO do passo, e o quadro em que ela diverge saía NÃO FINITO para o ecrã (o re-cozimento com `dt = 0` escondia-o). Guarda também DEPOIS do passo, na CPU e na WGSL; o gate volta verde sem mudar uma letra |
| `soft_body_radius::the_soft_body_takes_a_particle_radius…` | par mais próximo `0,3000 → 0,3448` (passo duplo) · `0,3000 → 0,2890` (produto) | a régua era a que o cabeçalho do próprio gate proibia (pares de malha seguros a `SPACING < 2·RADIUS`); a lei é a EXTENSÃO, `1,92 → 2,76` (`+43 %`), agora exigida a `+20 %`. ⚠️ **Achado, não curado**: o colisor não segura a distância mínima de um corpo mole (pior par não vizinho à SAÍDA do colisor `0,21` contra o diâmetro `0,56`) — ele empurra também os vizinhos de malha; nunca valeu, nem no passo duplo |
| wave `the_injected_producer_moves…` · `the_two_bands_are_comparable…` | pico `−3,00 → +0,50` · razão `1,241 → 0,959` | a injecção da cena `=57` foi calibrada sobre o depósito DUPLO — no app ela sempre teve METADE da força. Varredura na ordem do produto (`probe_the_drive_scale_in_product_order`): `0,25 → 0,959` · `0,37 → 1,067` · **`0,42 → 1,251`** · `0,50 → 1,547` (barra `1,0..1,5`); pico em `−3,50` (a meia célula da caixa) a `240`, `360` e `480` tiques. `motion.drive` `scale 0,25 → 0,42`; o anúncio cita os números novos (`26` peças estouradas contra `21`: a frase «até MENOS» deixou de ser verdade e saiu) |

⭐ **E a sonda das fontes destapou um defeito de DESENHO**: com a lei desligada só na bomba ela ainda
lia `0` linhas do emissor (com a variável `=0`, `512`). A lei tinha DUAS fontes: a rota da CPU
sobrepunha a da bomba (`define_a_lei`) ao estilo, a da placa montava o estilo pelo ambiente
(`sink_style` → `so_com_forma_por_ordem()`), e a camada de brilho, as tomadas do gizmo de pontos e dois
sítios da shell (`present.rs`, `fase_motion_gizmos.rs`) liam o ambiente também. No app coincidiam;
qualquer bomba com a lei mudada punha as rotas a discordar. ⇒ uma fonte só: `estilos_das_saidas`
(a rota da placa com `motion.pump.a_lei()`) e todo leitor com o `MotionState` à mão lê a bomba. Gates:
`the_device_route_reads_the_pumps_law` e `the_environment_law_is_read_in_one_place` (censo de texto
sobre `ph2d-app-motion/src` e a shell: o ambiente é lido SÓ no construtor da bomba — o censo achou o
`present.rs` que a primeira varredura à mão não tinha visto).

O gate de recorrência `nenhum_laco_avanca_o_tique_antes_de_cozer` (`ph2d-app-motion/tests/it`) lê os
`.rs` das duas pastas e recusa, dentro de cada laço ou fecho, um `advance_tick*` antes de um
`cook*` com o MESMO playhead; controlo com fixtura invertida e de produto; piso `3 772` corpos.

## §5 — A suíte da placa: as sondas saem do gate

Medido (binário RELEASE da base, cada teste sozinho, `-P 8`): `296` ignorados; `294` somam `~560 s`
em série (carga alta: `load` até `45`, outra linha a compilar); DOIS nunca acabam em `900 s` —
`audit_whether_the_bar_holds_for_every_shape` e `measure_scene_motion`, ambos SONDAS. As únicas
reprovações eram as seis deste ciclo. ⇒ o gate de placa não cabia por correr sondas. ⏳ Os `#[ignore]`
ganham prefixo de classe e o `scripts/motion-gate-placa.sh` corre só os `placa:`.

**Depois:** `298` `#[ignore]` com classe — `30` `placa:`, `259` `sonda:`, `9` `gerador:` (eram `1`/`20`/`9` e
`268` sem prefixo; os `10` `#[ignore]` nus eram `probe_*` que só imprimem). Gate de censo
`every_ignore_in_the_motion_crate_names_its_class` (`src/` e `tests/`, piso `250`, cada classe `≥ 1`). O
script resolve cada `fn` `placa:` a UM teste do `--list --ignored` e corre-os num processo:
**`30/30` verdes, `96 s` de parede com a compilação RELEASE, `5,68 s` de testes** (`load ~8`) — contra o
prazo de `1800 s` que a suíte inteira estourava.

## §6 — O SIGSEGV ao sair: uma crate de excepção para o `atexit`

Antes (binário RELEASE da base, `--include-ignored`, `load ~3`): **`3/3` com `rc 139`**, `~33 s`
cada; um só teste de placa (`parity_pulse_beat`): `0/3`. ⚠️ Nas três corridas completas reprovou
também `crossing_the_reach_boundary_does_not_step_the_cost` (`3,24×`), o flake de FAN-OUT já
registado em `FLAKES_DE_CARGA` — os testes correm em paralelo na mesma placa; não reaberto.

O desenho escrito antes era soltar o dispositivo num `atexit` (ABI C ⇒ `unsafe`, proibido em todo
alvo ⇒ uma crate de excepção, `ph2d-exit-hook`). Construído junto com o contexto ÚNICO (os `37`
`OnceLock` do `it` passaram a um, `tests/it/placa.rs`), e MEDIDO antes de ficar — as três variantes no
MESMO binário, por variável, intercaladas (`target/prova/c14/segv/`, `load 4–39`):

| variante | `SIGSEGV` ao sair |
|---|---|
| um contexto por ficheiro chamador (o desenho de antes) | **`8/8`** |
| contexto único, solto no gancho de saída | `0/8` |
| contexto único, seguro até ao `exit` (sem gancho) | `0/8` |

(e `0/10` nas cinco + cinco da rodada DEPOIS). ⇒ **a cura é o contexto ÚNICO; o gancho não mudou
nada e SAIU**, com a crate de excepção e a linha de `EXCECOES` — código `unsafe` sem efeito medido não
fica. Gate `only_this_file_holds_a_gpu_context_in_a_static` (censo de `tests/it`, com controlo; sangra
quando um ficheiro volta a declarar o seu). ⚠️ Os vermelhos `rc 101` dessas corridas são todos o
`crossing_the_reach_boundary_does_not_step_the_cost` (`6` em `24`), o flake de fan-out já registado.
⚠️ **Para o integrador**: o `ph2d-render` tem o mesmo desenho (`~20` `OnceLock` de contexto nos
`tests/it`) — não medido aqui (fora do módulo).

## §7 — A `=27` à vista

`sonda_a_cascata_a_vista_por_variante`, uma rodada, `19` variantes (à vista / população, a
`2 · 4 · 6 · 8 s`):

| vento | atração | arrasto | 2 s | 4 s | 6 s | 8 s |
|---|---|---|---|---|---|---|
| 1,4 | 0 | 0 (a cena de antes) | 6/6 | 4/14 | 0/41 | 0/106 |
| 1,4 | 2 | 0,5 | 6/6 | 10/14 | 23/41 | 53/106 |
| 1,4 | 4 | 0,5 | 6/6 | 13/14 | 39/41 | 102/106 |
| 1,4 | 6 | 0 | 6/6 | 10/14 | 27/41 | 69/106 |
| **1,4** | **6** | **0,5** | **6/6** | **14/14** | **41/41** | **106/106** |
| 0,7 | 4 | 0,5 | 6/6 | 14/14 | 41/41 | 104/106 |

Escolhida a de negrito: o vento FICA (as centelhas continuam a subir), a atração (`raio 14`, cobre
o recorte) tra-las de volta e o arrasto tira a energia que a atração devolveria (sem ele `69/106`
aos `8 s`). A população é a MESMA de antes (as forças não tocam no nascimento): `3 → 6` no 1.º
estouro, `106` aos `8 s`. Gate novo `the_cascade_stays_in_view` (`≥ 90 %` nos quatro instantes, com
o controlo de que a população cresce).

## §8 — `PH2D_PAN_DIAG` lê o mundo da cópia do dispositivo

⚠️ O desenho da lista (uma tomada armada na bomba) estava ERRADO, e a auditoria do código disse
porquê: na rota toda-dispositivo uma tomada da bomba vai por `pump.cook_taps_only`, que RECOZINHA a
cadeia na CPU (`2,4 ms` a `32 768`, doc 120 §8.5) — custaria isso e leria o mundo da CPU. A leitura
dos cartões (`motion.gpu_tap`, `tap_sem_espera`) já amostra todo nó encenado, o sink incluído, e a
amostra `0` é a linha `0`. ⇒ `pan_diag_tap::mundo_lido` lê o `P[0]` do primeiro sink dali (a mesma
lei do `lower` da CPU: `world_pos = P`, sem transforme por cima), e `note_device_world` imprime-o
(`1–2` quadros atrás, dito na linha). Gates: `pan_diag_o_mundo_da_primeira_copia_e_o_da_cpu` e
`pan_diag_sem_leitura_dos_cartoes_nao_ha_mundo` (CPU), `pan_diag_a_sonda_desligada_nao_le_nada_da_placa`
(régua de texto: tudo dentro do `if ph2d_pan_diag::on()`, nenhuma descarga nova, nenhuma tomada),
`pan_diag_o_mundo_da_placa_e_o_da_cpu` (placa).

## §9 — A rodada de medição

**ANTES** (08/10, `16:33–16:52`, uma rodada em dois blocos, `target/prova/c14/antes/`): as seis
sondas e o gerador do tutorial 6, a cascata, os nove geradores contra o disco, o SIGSEGV (`3 + 3`
corridas) e o tempo de cada um dos `296` ignorados. **DEPOIS** (`17:18–17:30`, `target/prova/c14/depois/`):
as sondas curadas verdes sozinhas (a das fontes depois da cura da lei de fonte única, §4), o `PAN_DIAG`
na placa, o SIGSEGV `5 + 5` intercalado, o FMA nas duas placas com e sem a mutação, o gate de placa
cronometrado; e um bloco em SÉRIE, com o porquê escrito: o `0/10` da ablação mudava a pergunta do §6, e
as três variantes foram medidas a seguir no mesmo binário (`target/prova/c14/segv/`, `8 × 3`).
