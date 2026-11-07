# HANDOFF de continuação — line/SfxLab, onda W4 «Exploração» (2026-11-06)

Leitor: a próxima LLM da linha (ou o integrador). Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab`, branch `line/SfxLab`, base `bbfefaf393`. **16 commits na linha** (`git log bbfefaf393..HEAD --oneline | wc -l`), código da onda em `d454b6ce09` (W4) e `aed16b1e9d` (duas mutações fechadas). **Nada enviado, nada integrado.** Estado: **construída; smoke do dono PENDENTE.** O plano foi aprovado pelo dono, que delegou as quatro decisões de desenho (memória `project_sfx_w4_owner_delegated_decisions_2026_11_06.md`, em `project-memory/` do checkout primário; não está nesta árvore ainda). Desenho, números e teste às cegas NÃO são copiados aqui: [05_W4_plano.md](../05_W4_plano.md) — §2 desenho, §6 números, §7 teste às cegas. Antecedente: [W3](HANDOFF_CONTINUACAO_W3_2026-11-06.md).

## 1. Superfície de colisão (saída de `bash scripts/collision-surface.sh`, resumida; rode de novo no main do dia)

```
merge-base bbfefaf393 · 16 commit(s) · 118 arquivo(s)
PROJECT_SCHEMA 217 (base 217) · tripla (217, 13, 29) (base igual) · VEC 29 · FLIP 13 · DOC_VERSION 25
ph2d-ecs 118 (base 118) · ph2d-render 119 (119) · ph2d-script 119 (119)
node.rs intocado · tool.rs intocado
ADR: nenhum criado (último 0198, próximo livre 0199)
Cargo.lock: +ph2d-app-sfxlab, +ph2d-host-sfxlab-scenes, +ph2d-object-picker, +ph2d-panel-sfxlab,
            +ph2d-sfx-explore, +ph2d-sfx-explore-tela, +ph2d-sfx-lab   (7 no total desde a base)
Marcadores de conflito: nenhum · Tetos de LOC: nenhum arquivo passa
```

## 2. Delta para o integrador (só o que a W4 somou; o da W3 está no handoff dela)

- **PROJECT_SCHEMA +0**, registo de componentes **+0 / +0 / +0**, ADR **+0**. Recontar mesmo assim (`python3 scripts/schema-recount.py`).
- **Formato gravado:** blob `PH2DSFXL` **v2 → v3**, lendo v1 e v2. Fixture da v2: `crates/ph2d-sfx-recipe/src/fixtures/library_v2.bin`. Não é o schema do projeto; é o blob da biblioteca.
- **Cargo.lock da W4:** `+ph2d-sfx-explore` (motor), `+ph2d-sfx-explore-tela` (tela).
- `PH2D_SFX_SMOKE` agora **`1..6`** (5 = «Impact» com a primeira camada travada; 6 = a 5 com o Explore aberto).
- API que muda de forma: `Feito` ganhou `nivelada`; `ph2d_sfx_dsp::Rng` virou **público**; `maior_momento` novo em `crates/ph2d-audio/src/dsp/loudness.rs`; `Layer` ganhou `variacao`, `travada`, `travas`; `SfxLibrary` ganhou `ouvidos`.

## 3. O que um merge pode partir

- **Teto de `ph2d-app-sfxlab`:** ~2471 linhas contra o teto 2692 (`find crates/ph2d-app-sfxlab -name '*.rs' | xargs wc -l | tail -1`; na W3 estava em 2690). A folga veio de MOVER os gates para `shells/sfxlab-scenes/tests/familia` (teste novo `familia`). **Shell sem linha nova**; o integrador reconta `the_shell_only_shrinks` contra o main do dia.
- `ph2d_sfx_dsp::Rng` público e o hash do `ph2d-sfx-dsp`: outra linha que mexa no DSP tropeça no gate de hash.
- Tocadas fora da família: as da W3 (§3 do handoff dela) continuam valendo; a W4 acrescenta `ph2d-audio` (`maior_momento`) e `ph2d-sfx-recipe` (blob v3). Chaves i18n novas: `toda_chave_do_load_tem_texto` deve seguir verde.
- `ph2d-object-picker` depende de `Field` de `ph2d-kanban-views` (aberto da W3 §6.3, inalterado).

## 4. O que a W4 entregou

Roll, Mutate, Lock (por camada e por campo), Next, Keep, Heard e Explore (`Tab`), com os filhos no mesmo volume. Mecanismo e números: plano §2 e §6. Não repetidos.

## 5. Prova de fecho

- Clippy `-D warnings` limpo nas 11 crates tocadas.
- Testes (contagens da corrida do verificador; não rodei de novo): `ph2d-sfx-explore` 18 · `familia` 38 · `it` 48 + 4 ignorados · `ph2d-sfx-recipe` 7, mais os 2 testes novos da mutação.
- Gates verdes por nome: `the_shell_only_shrinks`, `no_dependency_climbs_a_layer`, `no_magic_numeric`, `o_motor_nao_chama_a_libm_do_sistema`, `toda_chave_do_load_tem_texto`, `a_fronteira_dos_motores`, hash do `ph2d-sfx-dsp`, `a_tool_panel_never_hides_the_inspector`, `no_family_crate_hides_the_inspector_to_take_its_slot`, `a_lista_de_sons`. Por pedaço de nome: `interactive_crate` e `panel_wiring` (os nomes `architecture_interactive_crate_has_behavioral_test` e `architecture_panel_wiring_parity` não existem como fn). Greps do §6 do plano: 0 e 0.
- **Suíte completa NÃO rodada** (é do integrador).
- **Mutação: 15 de 15 sangraram**, depois de fechar duas que sobreviveram: `recebe` aceitando qualquer ticket (novo `render_velho_que_ja_terminou_tambem_nao_entra`) e `troca_por` sem preservar o aprovado (novo `trazer_do_heard_nao_desaprova_o_som_do_jogo`).
- **Lacuna honesta do vermelho-primeiro:** os testes de lei e de mão nasceram junto com o código e passaram de primeira, salvo o Lock por campo, visto vermelho (bug real: `hover_targets` conta botão focado; cura lê só `Hovered`). A força deles veio da mutação, não do vermelho.

## 6. ABERTO, em ordem

1. **Smoke do dono** (§7 abaixo).
2. **Teste às cegas** (plano §7): depende do dono — convidar o profissional, 1h30 para fazer os sons. A W5 pode seguir sem ele, com o teste marcado antes de congelar a tela.
3. **No fecho da LINHA, atualizar o §5 do `CLAUDE.md`** (NÃO feito; ordem de não tocar): a entrada do SFX Lab diz «ainda sem tela» e `1|2`; deve virar `1..6`, e trocar o link para o handoff de integração (≤ 700 bytes).
4. O medidor Loudness segue na medida de arquivo inteiro; ele e o `nivelada` leem coisas diferentes de propósito.
5. Tamanho do Roll (até ×4,5) e o alvo −23 são medidos por máquina; passam pelo ouvido do dono.
6. Não provado: a lista Heard tem rolagem na roda sem teste próprio; `L` sobre campo só foi provado no banco de cartões; a tela virtual não aceita clique, então arrasto e Heard cheio não foram fotografados (só as cenas 5 e 6, paradas).
7. Os abertos da W3 que seguem valendo (`Field` do Kanban, itens 4–8): [W3 §6](HANDOFF_CONTINUACAO_W3_2026-11-06.md).
8. A linha continua **sem rebase**; o `Field` e as tocadas da W3 valem.

## 7. Smokes (comando inteiro)

Binário já montado em `target/smoke/ph2d-host-desktop` (remonte com `--profile smoke` se a árvore andou).

1. Cena 5 (Impact, primeira camada travada): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=5 ./target/smoke/ph2d-host-desktop`
2. Cena 6 (a 5 com o Explore aberto): `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-SfxLab && env PH2D_SFX_SMOKE=6 ./target/smoke/ph2d-host-desktop`

## Smoke do dono

**Pendente.** Os passos para o Enio (curtos, sem jargão) saem do plano §6; a janela que fechar a onda os entrega depois de fotografar as cenas 5 e 6.
