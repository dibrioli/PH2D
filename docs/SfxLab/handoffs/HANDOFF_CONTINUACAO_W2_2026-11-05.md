# HANDOFF de continuação — line/SfxLab, onda W2 (2026-11-05)

Leitor: a próxima LLM da linha (ou o integrador). Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, branch `line/SfxLab`, base `c61f228b05`. **Nada integrado, nada enviado.** Estado: W2 feita, **à espera do ouvido do dono**. Suíte completa e impactados só na integração. Mecanismo e números completos: [03_W2_medicao.md](../03_W2_medicao.md) (não copiados aqui). Antecedente: [W0+W1](HANDOFF_CONTINUACAO_W0_W1_2026-11-05.md).

Commits de W2: `git log --oneline 67b746aadf..HEAD` (4: `3c2c83e4dd` receita e motor · `6c5e06bf13` receitas no arquivo do projeto · `b89a8d6fa0` `PH2D_SFX_SMOKE=2` · `c0882d5da3` sobreviventes da mutação e medição).

## 1. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, colada; cobre W0+W1+W2)

```
SUPERFÍCIE DE COLISÃO — line/SfxLab contra main
  merge-base c61f228b05   ·   17 commit(s)   ·   164 arquivo(s)
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        217   (base: 216)
  ⚠   └ tripla do gate               (217, 13, 29)   (base: (216, 13, 29))
    VEC_SCENE_SCHEMA                       29   (base: 29)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 25   (base: 25)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              118   (base: 116)
  ⚠ ph2d-render (espelho)                 119   (base: 117)
  ⚠ ph2d-script (espelho)                 119   (base: 117)
▸ CONTRATO CONGELADO (§6): node.rs intocado · tool.rs intocado
▸ ADR: último no disco 0198, próximo livre 0199
  ⚠ esta linha cria ADR: 0198   — reconte contra o main do dia
▸ Cargo.lock — ⚠ 4 pacote(s) '+name' novo(s): "ph2d-sfx-dsp" "ph2d-sfx-recipe" "ph2d-sfx-scene" "ph2d-sound-set"
▸ MARCADORES DE CONFLITO: nenhum
▸ TETOS DE LOC
  (depois do corte) 586 / 600   shells/desktop/src/project_schema.rs
```

**`project_schema.rs` passou de 600 (608) com o degrau, e o gate `shell_files_respect_hr18_loc_cap` reprovou de verdade.** Cura por corte: o degrau `171 → 172` foi arquivado verbatim em `docs/archive/project-schema/escada_v171_a_v172.md`; 586/600, gate verde por nome.

## 2. Delta para o integrador contar contra o main do dia

- **PROJECT_SCHEMA +1 (216→217)** por W2: campo `sfx_lab` no FIM do projeto, `PROJECT_SCHEMA_SEM_RECEITAS_DE_SOM`. Tocados: a tripla do gate; `falta_no_fim` e os braços 192/191 e 190/189 levam mais um `0`; `project_geometria_v194_tests.rs` tem o literal 217 e o byte varint `0xD9`. **Recontar por `python3 scripts/schema-recount.py`**, não copiar 217.
- Registo de componentes: **W2 +0** (o +2/+2/+2 acima é de W1).
- ADR: W2 0 novo (0198 é de W0, provisório).
- Cargo.lock: W2 +2 crates (`ph2d-sfx-recipe`, `ph2d-sfx-dsp`) + dependências da `ph2d-sfx-scene`.
- Gate `o_motor_nao_chama_a_libm_do_sistema` (`ph2d-det-math`): `CRATES` passou a **13**.
- Cena `PH2D_SFX_SMOKE` com `max_level` 2.

## 3. O que um merge pode partir

- ⚠️ **A shell está quase no teto**: `the_shell_only_shrinks` 189019 de 189041, folga 22, medido na árvore. É número que soma entre linhas: o integrador reconta, e qualquer linha que cresça a shell reprova contra esta. Para caber, os testes de blob foram para a crate e o marcador do `PH2D_ANIM_SMOKE=2` saiu da shell para `ph2d_sfx_scene::marcadores_do_anim_smoke`. **W3 (aba) tem de MOVER antes de acrescentar.**
- `ph2d-app-components` intocada em W2 (80866/80869).
- `shells/desktop`: 18 arquivos, +99 −16 desde a base (W1 e W2 juntas). Mais provável de colidir: `project.rs` (save/load de `sfx_lab`), `project_schema.rs`, `project_geometria_v194_tests.rs`.
- Load: blob de receitas ilegível **recusa o load** com o toast `shell.project_load.project_refused_its_recipes`; o gate `toda_chave_do_load_tem_texto` (`ph2d-i18n`) agora é censo de TODA a família `shell.project_load.*`: outra linha que adicione chave `project_load` sem texto reprova ali.
- `sfx_lab` está fora do `ProjectState`: sem desfazer até W3.

## 4. O que W2 entregou (mecanismo; detalhe no doc de medição)

- **`crates/ph2d-sfx-recipe`** (folha, sem FFT; feature `component` dá o `Resource`): `Recipe{seed, engine, seconds, layers, next_layer}`, `Layer{id, gain, start, source, envelope}`, `Source::{Noise, Tone, Body, Recording}`, `Envelope` por pontos, `Take` (PCM decodificado, endereçado por blake3 de taxa+canais+bits; não-finito vira 0 na entrada), `SfxLibrary` e blob `PH2DSFXL` v1, `sanitize` (idempotente), `factory` (8 receitas + `recorded`: fixture e prova de ouvido, NÃO presets do usuário).
- Limites (medição ao lado no código): `MAX_SECONDS=120` (RAM do cache), `MAX_SEMITONES=48`, `MAX_HZ=21600` (0,9 × Nyquist), `MAX_GAIN=1e6` (só soma finita; faixa de tela é de W3).
- **`crates/ph2d-sfx-dsp`**: `render(recipe, seed, engine, takes) -> Result<Pcm, RenderError>` é `RenderCache::default().render(...)`: **uma porta só**. Soma em ordem de id (reordenar a pilha não muda um bit). Ganho e início fora da chave do cache. `layer_seed(seed, id)`. Motor desconhecido é erro. Material ausente: silêncio e `Pcm.missing`.
  - Tom: degrau de banda limitada de tabela, escrito também antes do salto (offline); FM com banda de Carson presa.
  - Corpo: polos complexos; modos por `Stretched`, `Bar`, `Membrane`; ganhos somam 1; pára em 2⁻²⁴.
  - Gravação: sinc janelado com corte que desce com a razão; mesma taxa e altura = cópia.
- **Cena** `crates/ph2d-sfx-scene/src/receitas.rs`: um objeto por receita (corpos primeiro), Preview toca o WAV `f32` do render, moedas em Sequence no objeto «Recipe Sounds». Sem controle novo. Foto: `target/foto-sfx/receitas.png`.

## 5. Prova de fecho

Gates novos, por nome (em `crates/ph2d-sfx-dsp/tests/it/` salvo indicação): `hash_dourado_das_receitas_de_fabrica` · `hash_dourado_da_receita_de_gravacao` · `aliasing_de_tom_agudo_varrido` · `a_regua_bate_com_o_sox` · `fm_nao_dobra_em_nyquist_com_indice_alto` · `gravacao_acelerada_nao_dobra` · `nan_nao_entra_em_estado_por_nenhum_campo_da_fonte` · `nan_nao_entra_pelos_campos_da_camada_nem_do_envelope` · `nan_no_material_nao_envenena_o_resto` · `a_semente_nao_muda_quando_o_vizinho_e_editado` · `contador_atrasado_no_arquivo_nao_repete_um_id` · `o_cache_da_os_mesmos_bytes_e_refaz_so_a_camada_editada` · `reordenar_a_pilha_nao_muda_um_bit`; `um_v216_abre_sem_receitas` (`shells/desktop/src/project_sfx_tests.rs`); `toda_chave_do_load_tem_texto` (`ph2d-i18n`); `sonda_cada_receita_sai_do_mixer` (`shells/game-scenes/tests/it/receitas.rs`).

- **Red-first visto de verdade:** hash (0 gravado), aliasing (−21 e −55 dB: defeitos reais de tabela), oráculo (fixture vazia e `sox` saturado). **Semente e NaN: vermelho visto só por mutação.**
- **Mutação: 15 de 15 sangraram** — M1 semente pela posição · M2 sem sanitize · M3 Take · M4 e M5 chave do cache · M6 ordem · M7 Carson · M8 corte da gravação · M9a rampa · M9b salto · M10a guarda de `next_layer` · M10b `add` · M11 `falta_no_fim` · M12 chave do toast · M13 piso. M10a e M12 sobreviveram na 1.ª rodada e ganharam gate; os dois gates novos foram vistos vermelhos sob a mesma mutação.
- Clippy limpo: sfx-recipe (com e sem `component`), sfx-dsp, sfx-scene, i18n, det-math, game-scenes, host-desktop. fmt limpo. Greps de contrato da §3: 0 e 0 (o 2.º agora também sobre as crates sfx). `no_dependency_climbs`, `file_loc_cap`, `the_shell_only_shrinks` verdes por nome.

## 6. Smokes (comando inteiro)

- Cena do dono: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=2 ./target/smoke/ph2d-host-desktop` (binário confirmado em `target/smoke/`; remonte com `--profile smoke` se a árvore andou).
- Sonda offline: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash docs/SfxLab/ferramentas/sonda_w2/corre.sh`
- Mixer: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && bash scripts/ph2d-run.sh cargo test -p ph2d-host-game-scenes --test it receitas -- --nocapture`
- O card dos objetos mostra «Nothing plays this: Autoplay is off and no Signal Action targets it»: é verdade (eles existem para o Preview), não é defeito.

## 7. ABERTO, em ordem

1. **Ouvido do dono no corpo vibrante: kill-criterion de W2.** Reprovou ⇒ volta ao desenho antes de W3. Nenhum teste diz se soa bem.
2. **W3 (Tela).** Ler antes a receita de cartões. O render ainda não corre em thread de trabalho nem tem desfazer; o limitador é de W3 (o motor pode passar de 1,0 com ganho alto). Mover da shell antes de acrescentar (§3).
3. **Provar Intel/ARM:** no Mac, `cd docs/SfxLab/ferramentas/sonda_w2 && cargo run --profile smoke --bin hashes` tem de terminar em `conjunto 86ea4a05df14620b`. Receita com efeito: sem garantia entre máquinas, rack não portado.
4. **Limitações sabidas:** gravação a +24 semitons ou mais passa de 30 ms ao editar (32 a 40 ms) · «Recording Down» usa `coin_noise.wav`, som sintético no papel de gravação (só prova o caminho) · o clique no Preview é rota lida e passada pela mesma função no teste, não clicada · o campo do hash do som aprovado (plano §2) ainda não existe no documento · mudar `start` refaz a camada quando o trecho muda de tamanho · fim de duração sem envelope pode estalar (as receitas de fábrica terminam quase em silêncio).
5. **Pendências do dono:** biblioteca gravada e profissional para W4. A memória `project-memory/project_sfx_module_no_ai_2026_11_05.md` fica para a integração.

## Smoke do dono

**Aprovado em 06/11** (kill-criterion do corpo vibrante fechado): «gostei dos resultados». Ressalva dele: o Drum Thump «quase não se ouve» nas caixas de som pequenas. O clique no Preview foi feito no app pela primeira vez neste smoke.
