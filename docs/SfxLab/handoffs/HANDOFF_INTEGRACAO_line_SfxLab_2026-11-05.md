# HANDOFF DE INTEGRAÇÃO — line/SfxLab, W0 + W1 + W2 (2026-11-05)

**⛔ O dono mandou escrever este handoff para integrar, mas integrar é ordem dele e AINDA NÃO foi dada nesta janela («Escreva handoff mas não integre»). O smoke da W2 NÃO foi feito: o dono ouve amanhã. W0 e W1 têm smoke OK; o kill-criterion do corpo vibrante está ABERTO. Se o integrador for acionado antes do smoke, a W2 entra no main sem ninguém a ter ouvido; quem decide é o dono. «Integrar não é aprovar» (CLAUDE.md §0.7).**

Leitor: o agente integrador. Nada integrado, nada enviado.

## 0. Identidade

Branch `line/SfxLab`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, HEAD (antes do commit deste doc) `b058603a9c`, merge-base `c61f228b05`, 20 commits, 166 arquivos. Mecanismo de cada onda, não copiado aqui: [W0+W1](HANDOFF_CONTINUACAO_W0_W1_2026-11-05.md) · [W2](HANDOFF_CONTINUACAO_W2_2026-11-05.md) · [medição W0](../02_W0_medicao.md) · [medição W2](../03_W2_medicao.md) · [plano](../01_plano.md) · [ADR-0198](../../architecture/decisions/0198-o-motor-do-sfx-lab-e-kernels-proprios-sobre-det-math.md).

`git log --oneline c61f228b05..HEAD`:

```
b058603a9c fix(sfxlab): os achados da auditoria de fecho, cada um com o gate visto vermelho
2fc433e44e fix(sfxlab): o degrau 171 → 172 vai para o arquivo morto: project_schema.rs voltou a 586 de 600
29b221b593 docs(sfxlab): handoff de continuação W2
c0882d5da3 test(sfxlab): os dois sobreviventes da mutação e a medição de W2
b89a8d6fa0 feat(sfxlab): PH2D_SFX_SMOKE=2, os sons da receita para o dono ouvir sem tela
6c5e06bf13 feat(sfxlab): as receitas de som no arquivo do projeto (PROJECT_SCHEMA 216 → 217)
3c2c83e4dd feat(sfxlab): W2, a receita e o motor: render por porta única, cache por camada, gates
67b746aadf docs(sfxlab): smoke da W1 aprovado pelo dono
ed6875edd4 docs(sfxlab): handoff de continuação W0+W1, estado e números no plano
696586d262 test(sfxlab): os espelhos do registo de componentes, delta +2
92d4a8cba3 chore(sfxlab): clippy do cartão Sound
eb7ebe6e17 chore(sfxlab): clippy do fecho da onda W1
eddc2d5e27 test(sfxlab): o gate da âncora da shell volta a compilar com markers e steps em InspectorAnimRow
4553d9d65e feat(sfxlab): PH2D_ANIM_SMOKE=2, a cena de animação com um marcador e Advanced aberto
b219285223 refactor(sfxlab): a catraca das crates app, movendo o que a W1 pôs nelas
fb5832f44e feat(sfxlab): W1, variações na mesma linha e o smoke abre no cartão dos sons
16d722a2f0 feat(sfxlab): W1, cartão Sound com a lista de sons e marcador em quadro no Inspector
01feb92177 feat(sfxlab): W1, vários sons por objeto, sorteio em SceneAudio::play, Autoplay espera o Play
40b48e9bdc docs(sfxlab): W0 medida — kernels próprios sobre det-math (ADR-0198), sonda offline
997883c719 docs(sfxlab): plano em waves, pesquisa e crítica de design de áudio
```

## 1. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, colada, fechamento de 05/11)

```
SUPERFÍCIE DE COLISÃO — line/SfxLab contra main
  merge-base c61f228b05   ·   20 commit(s)   ·   166 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        217   (base: 216)
  ⚠   └ tripla do gate               (217, 13, 29)   (base: (216, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS
▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
  ⚠ ph2d-ecs                              118   (base: 116)
  ⚠ ph2d-render (espelho)                 119   (base: 117)
  ⚠ ph2d-script (espelho)                 119   (base: 117)
▸ CONTRATO CONGELADO (§6): node.rs intocado · tool.rs intocado
▸ ADR: último no disco 0198, próximo livre 0199
  ⚠ esta linha cria ADR: 0198   — reconte contra o main do dia
▸ Cargo.lock — ⚠ 4 pacote(s) '+name' novo(s): "ph2d-sfx-dsp" "ph2d-sfx-recipe" "ph2d-sfx-scene" "ph2d-sound-set"
▸ MARCADORES DE CONFLITO: nenhum nos arquivos da linha
▸ TETOS DE LOC: nenhum arquivo da linha passa do teto
```

Prazo de validade: a coluna `base:` é o merge-base. Leia o `main` no arquivo (`git show main:<arq>`) e conte a linha como DELTA.

## 2. Contadores como DELTA (contar contra o main do dia)

- **PROJECT_SCHEMA +1 (216→217)**, todo de W2: campo `sfx_lab` no FIM do projeto, `PROJECT_SCHEMA_SEM_RECEITAS_DE_SOM`. Tocados: a tripla do gate; `falta_no_fim` e os braços 192/191 e 190/189 levam mais um `0`; `shells/desktop/src/project_geometria_v194_tests.rs` tem o literal 217 e o varint `0xD9` (217). **Recontar por `python3 scripts/schema-recount.py`**, não copiar 217. Atenção à escada e à tripla em arquivos irmãos.
- Registo de componentes: **ecs +2, render +2, script +2**, todos de W1 (`SoundSet`, `SpriteAnimMarkers`). Os três censos (118, 119, 119) só rodam na suíte da própria crate.
- ADR +1 (0198, provisório). Cargo.lock +4 crates internas; nenhum pacote externo novo de W2 (confira na saída).
- `ph2d-det-math`: o gate `o_motor_nao_chama_a_libm_do_sistema` tem `CRATES` = **13**. `ph2d-i18n`: uma chave nova (`shell.project_load.project_refused_its_recipes`) e o gate `toda_chave_do_load_tem_texto` agora é censo de TODA a família `shell.project_load.*`: outra linha que adicione chave `project_load` sem texto reprova ali.
- Cena `PH2D_SFX_SMOKE` com `max_level` 2.

## 3. Foundational tocado e por que é aditivo

`ph2d-ecs`: `SoundSet`, `SpriteAnimMarkers`, componentes irmãos (sem degrau de schema em W1). `ph2d-det-math`: só o gate, lista 13. `ph2d-i18n`: uma chave e um censo. `ph2d-sfx-recipe`, `ph2d-sfx-dsp`, `ph2d-sound-set`, `ph2d-sfx-scene`: folhas novas. `shells/desktop`: 18 arquivos, +100 −37 desde a base; mais provável de colidir: `project.rs` (save/load de `sfx_lab`), `project_schema.rs`, `project_geometria_v194_tests.rs`.

**`InspectorAnimRow` ganhou `markers` e `steps`: quem a construir literal em outra linha quebra a compilar** (gate da âncora da shell ajustado em `eddc2d5e27`).

Comportamentos mudados de propósito (W0/W1 §2): Autoplay só dispara no Play; `Shuffle` no Audio Editor passou de «não repete» a baralho, nas duas pontas. Um teste de outra linha pode tê-los fixado.

## 4. Contratos e tetos

- §6 intocado; greps da §3 do plano: 0 e 0 no fecho; `node.rs` e `tool.rs` intocados.
- **Tetos somam entre linhas (cura por corte/mover, nunca subir):** §5 do `CLAUDE.md` perto dos 14 000 bytes (`o_claude_md_cabe_no_orcamento` reprovou com a nossa entrada; a entrada foi encurtada e §5.0 perdeu o que repetia o §0 e o doc de flakes; com outra linha acrescentando, corte de novo) · shell 189019 de 189041 (**folga 22**, remedir: `the_shell_only_shrinks`) · `project_schema.rs` 586/600 (o degrau `171 → 172` foi arquivado verbatim em `docs/archive/project-schema/escada_v171_a_v172.md`: o nosso degrau o levou a 608 e `shell_files_respect_hr18_loc_cap` reprovou) · `ph2d-app-components` 80866/80869 · `ph2d-app-audio` 10514/10547 · `ph2d-app-game` 2708/2728 · `ph2d-app-sprite-anim` 8345/8352.

## 5. O que mudou depois do handoff de continuação W2

Auditoria com duas lentes independentes (correção do motor; costura e formato). Cada achado teve o gate visto vermelho antes da cura.

- Material vazio dava pânico: `material_vazio_nao_quebra_o_render`.
- Material enorme somava inf/NaN: `MAX_SAMPLE=1e6`, `material_enorme_nao_estoura_a_soma`.
- `modes` sem limite: base do corpo ≥ `MIN_BODY_HZ` = 20 Hz, 1 080 modos no máximo, `corpo_com_modos_absurdos_para_na_banda_audivel`.
- Receita de 23 GB: `MAX_BYTES` = 1 GiB e `RenderError::TooLarge`, `receita_grande_demais_e_recusada_antes_de_alocar`.
- Curva longa errava 4,8 %: conta em `f64`, `curva_longa_chega_ao_valor_declarado`.
- `add` estourava: `contador_no_teto_nao_estoura`.
- Costura: a cena apagava `moeda.ogg` (agora soma); `171 → 172` estava na lista errada da escada.
- **NÃO corrigido, pré-existente:** Preview em objeto sem `Transform` fica mudo sem aviso (`audio_2d.rs:58-61`, os chamadores descartam o retorno).
- O motor mudou de bytes: **`conjunto 86ea4a05df14620b`** em x86-64, x86-64-v3 e wasm32. Hashes de W2 regravados.

## 6. Prova de fecho (medido, 05/11)

- fmt limpo; clippy `-D warnings --all-targets` em 19 crates tocadas + `ph2d-host-desktop` + `ph2d-host-game-scenes` + features `component`.
- Verdes por nome: `file_loc_caps`, `fn_loc_caps`, `architecture_no_downcast`, `project::tests::{sfx,sounds,pixel}`, `geometria_v194`, `project_schema`, `arch_safe_clamp_only`, `the_shell_only_shrinks`, `no_dependency_climbs`, `file_loc_cap`, `contract_surface`, `claude_md_cabe_no_orcamento`, `o_motor_nao_chama`, `as_chaves`, `a_fronteira`, `a_tabela_inglesa`, `sound_set`, `a_lista_de_sons`, `sprite_marcadores`, `receitas`, `a_cena_do_sfx`, os três censos de componente (118, 119, 119).
- Crates novas inteiras: `ph2d-sfx-dsp` 28, `ph2d-sfx-recipe` 3, `ph2d-sfx-scene` 1, `ph2d-sound-set` 17.
- Mutação: 15 de 15 antes da auditoria. Os seis gates da auditoria viram vermelho pelo teste antes da cura; **sem nova rodada do agente de mutação depois**.
- **`nextest-impacted` e suíte completa NÃO correram na linha** (regra do dono 23/10, CLAUDE.md §2): são do integrador. Nota: o `/pd-linha-fechar` ainda manda `nextest-impacted` no passo 1, o que contradiz o CLAUDE.md.
- Corridos no fecho (DIRETRIZ §1.5.9 itens 5 e 5-bis), verdes: `censos-da-arvore-combinada.sh` (132 testes, 12 de 12 censos), `cargo machete` nas crates novas, em `shells/game-scenes` e na sonda (achou uma dependência sem uso na sonda, retirada), `check-standalone-optional.sh`, `check-workflow-packages.sh`. Isto não substitui o portão de integração. A linha não usa `#[cfg(target_os`; confirme por grep.
- Perfil do loop: paralelismo 1,85, `test:check` 5,6x (✗), contexto por passo 307 mil (✗).

## 7. Smokes (comando inteiro)

- W2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=2 ./target/smoke/ph2d-host-desktop`. O card dos objetos mostra «Nothing plays this: Autoplay is off and no Signal Action targets it»: é verdade, não é defeito.
- Sonda W2: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash docs/SfxLab/ferramentas/sonda_w2/corre.sh`
- Mixer: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash scripts/ph2d-run.sh cargo test -p ph2d-host-game-scenes --test it receitas -- --nocapture`
- W1: `PH2D_SFX_SMOKE=1`, `PH2D_ANIM_SMOKE=2` (aprovados pelo dono). Sonda W1: `sonda_mil_disparos`.

## 8. ABERTO, em ordem

1. **Ouvido do dono no corpo vibrante (kill-criterion de W2), amanhã.** Reprovou ⇒ volta ao desenho antes de W3. Nenhum teste diz se soa bem.
2. **Intel/ARM não provado.** No Mac: `cd docs/SfxLab/ferramentas/sonda_w2 && cargo run --profile smoke --bin hashes`, tem de terminar em `conjunto 86ea4a05df14620b`. Receita com efeito: sem garantia entre máquinas, rack não portado.
3. Memória `project-memory/project_sfx_module_no_ai_2026_11_05.md`: sem commit no primário, entra com a integração.
4. **W3 (aba):** precisa do render fora da thread da tela (pior corpo 158 ms em 2 s, 9,4 s em 120 s) e do desfazer (`sfx_lab` está fora do `ProjectState`); o limitador de ganho é de W3; **mover da shell antes de acrescentar** (folga 22).
5. Limitações sabidas (detalhe no handoff W2 §7): gravação a +24 semitons ou mais passa de 30 ms ao editar; «Recording Down» usa som sintético; o clique no Preview não foi clicado; o campo do hash do som aprovado não existe; mudar `start` refaz a camada quando o trecho muda de tamanho; fim sem envelope pode estalar.
6. Pendência do dono: biblioteca gravada e profissional para W4.

## Binário do smoke

Compilado por último, depois de `rm -rf target/*/incremental`. Comando: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`. Segunda saída, sem nenhum `Compiling`:

```
    Finished `smoke` profile [optimized] target(s) in 0.20s
```

Os commits depois disso só tocam docs e a sonda à parte.
