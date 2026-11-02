# J10 — bisseção dos 9 vermelhos antigos (§4 da J9)

Worktree de bisseção `Worktrees/bisect-j10` (target próprio, `--profile smoke`, testes POR NOME em
`ph2d-tool-painter --lib`). Eixo: `git log --first-parent 422fec2027..1cd27137be` (247 commits; índice
`[n]` = posição nesse eixo; 102–104 são as junções r3/r1/r2). Merge-base `422fec2027`: os 7 testes que
já existiam estão **verdes**. Os testes moram em `ph2d-tool-painter` (`wetpaint_preset/`), não em
`ph2d-wet-paint`.

## Tabela

| teste | nasce em | 1.º vermelho | pai verde? | número pai → culpado (e no HEAD) | família |
|---|---|---|---|---|---|
| `a_transmissao_tinge_o_papel_creme_do_documento` | `c8b319e561` (antes do merge-base, verde lá) | **`e8f35b5710`** [220] | sim, `bf90eb2598` [219] | ok → `NaN × 0.456` | N (NaN) |
| `a_transparencia_deixa_ver_o_grao_e_a_cor_do_papel` | `5d941c8246` (verde no merge-base) | ⚑ ≤ [247], verde em [166] | não conferido no culpado | HEAD `T 0 tapa o grão: NaN × 6.02` | N (pela mensagem) |
| `o_dente_do_papel_nao_atravessa_a_tinta_grossa` | `0c8b9e2b32` (verde no merge-base) | ⚑ idem | não conferido | HEAD `NaN × papel nu 6.02` | N (pela mensagem) |
| `a_transmissao_e_o_relevo_do_papel_mostram_o_grao` | `c8b319e561` (verde no merge-base) | ⚑ em (124,166] | — | [166] `Relevo 1 ≈ tela de antes: 0.482 × 0,67`; HEAD `a 0 … NaN × 0.0236` | P (166) + N |
| `o_papel_nos_vaos_entre_lascas_nao_fica_preto` | `9e361c9605` (verde no merge-base) | ⚑ em (124,166] | — | [166] `Classico: 1.46 % < 70 (tecto 0 %)`; HEAD `Estudio: 54.98 % (tecto 42 %)` | P (166) + ? |
| `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` | `129f57ef04` (verde no merge-base) | **`f70ea44cd9`** [31] | sim, `7bb6110ac5` [30] | [30] ok → [31] `Blender 0,5: L 0.134 × Cycles 0.205` (tol. 8 %); [101] `0.122`; [166] `0.166`; HEAD `L NaN` | A + N |
| `a_fenda_forte_nao_volta_aos_pontos_pretos_na_cena_do_dono` | `a4fa04571d` (verde no merge-base) | ⚑ em (2,101]; **verde de novo em [166]**; vermelho de novo em (166,247] | [2] verde | [101] `força 3: max(R,G,B)=43`; [124] 43; HEAD 68 (tecto 70) | A′ (curado em `a6796fd570` [146]?) + regressão tardia |
| `sonda_oleo_furos::o_oleo_sobre_o_oleo_assentado_nao_encaroca` | `ce913b3909` [166] | ⚑ em (166,247] | **verde ao nascer** | HEAD `Thick Impasto: p99 24.1 → 110.9 em 8 passadas` | S (sondas do óleo) |
| `sonda_oleo_grumos::o_oleo_sobre_oleo_molhado_satura_o_relevo` | `611591d423` [138] | ⚑ em (166,247] | **verde em [166]** | HEAD `p99 1.49 → 10.95` | S |

⚑ = intervalo apurado, culpado exacto não isolado no prazo de 45 min (corridas de 7 min sob load 44
do build do coordenador). Pontos medidos com os 9 nomes: [0]=merge-base, [101] `f3575e0c19`,
[124] `ac01f7c653`, [166] `ce913b3909`, [247] HEAD; [1]/[2]/[26]/[29..32]/[38]/[51] só A; [206]/[216]/[218..221]/[226] só N.

## Família N — o miolo vazio: `NaN` (culpado `e8f35b5710`, E10 no painel)

Mecanismo: todos os testes N amostram o «miolo da tinta grossa» por `cena_papel_texturizado`
(`medida_oleo_real_fenda.rs:252-263`): célula com `g.sett + g.susp > 3000.0` depois de 3 passadas do
Dabber. O `e8f35b5710` trocou a receita do pincel (`crates/ph2d-wet-paint/src/painter/receita.rs`):
`h` deixou de ser `volume × carga × H_REF` (o slider de pigmento × `Knob::Intensity`) e passou a
`volume_um × M_UM` (`Knob::Volume` em µm; `migra_para_controlos` só converte quem passa por ela), e
`phi = 1 − Dilution`. Com isso a massa que o Dabber pousa na cena do teste não chega a 3000 em
célula nenhuma → `miolo` vazio → `canal_linear`/médias dividem 0/0 → `NaN`, e os `assert!` com
faixa (`medida_transparencia.rs:373`, `:175`, `:99`; `medida_oleo_real_fenda.rs:336`;
`medida_transparencia.rs:477`) reprovam por comparação com `NaN`. ⇒ Em primeiro lugar é a **régua**
que está errada: o limiar `3000` é uma massa na unidade antiga e a cena não asserta que o miolo é
não-vazio (silêncio que vira `NaN`). Por conferir antes de só mudar o limiar: se o Dabber de fábrica
com o `Volume` migrado pousa a MESMA altura que antes (se não, o motor/migração também mudou a tinta
— comparar `sett+susp` máx. em [219] × [220]).

## Família A — o modo Blender: culpado `f70ea44cd9` [31] (a ladeira fotografada no início do traço)

Mecanismo: `f70ea44cd9` fez a ladeira do óleo, com o Real Oil na mão, usar a FOTOGRAFIA do óleo de antes
do traço (`Grid::oleo_antes`, `painter/doors.rs` chama `fotografa_a_ladeira` no Down;
`solver/ladeira_do_oleo.rs` `de(g, r0, r1, antes)` lê `oleo_antes[i]` em vez de `altura(g, i)` e devolve
`None` quando nenhuma célula tem altura). A cena do teste (`cena_papel_texturizado(true, …)`: Dabber com
o Real Oil ligado, 3 passadas) pinta sobre papel vazio — a fotografia é 0 e a tinta FRESCA deixou de ser
segurada pela própria ladeira: o relevo do Dabber muda e a luz do miolo cai (L 0,205 → 0,134). O motor fez
o que o dono pediu (a água respeita o óleo de ANTES); o que ficou errado é a **expectativa**: o alvo
`Cycles 0,205/0,744` (`medida_transparencia.rs:469-478`) foi tirado da altura antiga desta cena e não foi
regravado — oráculo por correr outra vez sobre a cena de hoje (e, depois da E10, o miolo vazio da família
N). `a_fenda_forte…` reprova no mesmo intervalo (2,101] (`max=43`, tecto 70) e lê a mesma cena do óleo
com Real Oil → provável o mesmo `f70ea44cd9`, NÃO conferido; foi curada por `a6796fd570` [146] (piso do
ganho do céu, `oleo_luz.rs`) e volta a vermelho (68) em (166,247], provavelmente pela E10 (N).

## Família P — os vãos e o relevo do papel, (124,166]

`o_papel_nos_vaos…` (`Classico 1,46 %` com tecto 0) e `a_transmissao_e_o_relevo…` (`0,482 × 0,67`)
reprovam em [166] e passavam em [124]. Suspeitos: `611591d423` [138] (o óleo molhado espalha o
relevo, `trail/espalha.rs`), `a6796fd570` [146] (o mesmo commit que acrescentou 77 linhas a
`medida_oleo_real_fenda.rs` e mudou `oleo_luz.rs` — pode ter APERTADO o tecto do Classico para 0 %),
`3570226c1e` [145] (ladeira do óleo). Não isolado.

## Família S — as sondas do óleo sobre óleo, (166,247]

Ambas verdes em [166] (ao nascer / logo depois); vermelhas no HEAD com o relevo a empilhar
(`p99 24 → 111`, `1,49 → 10,95`). Suspeitos: a pasta de Bingham `c5e6daa0a9` [190], a E4
`7a4bc540b7` [188] (Volume = só h), e o mesmo `e8f35b5710` [220] (receita nova: a altura por dab muda
com `volume_um × M_UM`). Não isolado.

## O que falta (próxima corrida, ~1,5 min por passo com a máquina calma)

1. `a_fenda_forte…` em [30]/[31] e o seu 2.º vermelho em (166,247].
2. Família P: (124,166] com `a_transmissao_e_o_relevo…` e `o_papel_nos_vaos…`.
3. Família S: (166,247] com as duas sondas; conferir se o culpado é `e8f35b5710`.
4. N: confirmar os outros 3 NaN em [219]/[220] (a mensagem é a mesma assinatura do miolo vazio).
