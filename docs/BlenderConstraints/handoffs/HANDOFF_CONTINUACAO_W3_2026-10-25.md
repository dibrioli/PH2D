# Continuação — `line/BlenderConstraints`, onda W3 (poderosas), 25/10/2026

> Anterior: [`W2`](HANDOFF_CONTINUACAO_W2_2026-10-25.md) (ossos, smoke OK). Plano: [`01_plano.md`](../01_plano.md) §7 W3 ·
> desenho, oráculo, recusas medidas e decisões do código: [`03_W3_poderosas.md`](../03_W3_poderosas.md) (§3 = o que o código
> decidiu — **não duplicado aqui**) · ADR [`0192`](../../architecture/decisions/0192-a-pilha-de-constraints-e-um-componente-por-entidade-avaliada-da-pose-autorada.md).

## Identidade

Worktree `Worktrees/line-BlenderConstraints`, branch `line/BlenderConstraints`, merge-base `c3662cd8e4` (o `main` não andou
desde o início da W3; o rebase foi no-op). HEAD = o commit deste handoff, árvore limpa. Commits da onda:
`181f1bb038` W3.0 (Look At = porta única) · `66fe625c3b` W3.1 oráculo · `54acec38ec` W3.2 os 7 tipos · `1504eeafbe` cena 5 em
laço · `0688561463` gate do limite de junta (M3) · `5718c5b69c` + `98fe151f27` + `63175808eb` smoke do dono 1 (a mola entre
quadros, cena natural) · `972fb7c644` smoke do dono 2 (extremos do laço e do vaivém). Binário `target/smoke/ph2d-host-desktop`
compilado no HEAD. `PROJECT_SCHEMA` desta linha = **194** (reconta-se na integração: `python3 scripts/schema-recount.py`).
Shell: **−2 linhas** na onda (`smoke::abrir_escolhido`, `smoke::passo`); catraca `the_shell_only_shrinks` verde.

## Smoke do dono (cena 5) — **OK** (25/10, 3.ª rodada)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-BlenderConstraints && PH2D_CONSTRAINT_SMOKE=5 ./target/smoke/ph2d-host-desktop`
2. Abre com o corpo, a antena (haste visível) e o cartão Constraints. Dar Play: o corpo acelera e trava, a ponta da antena
   balança atrás e assenta; o comprimento da antena não muda.
3. Arrastar a timeline (scrub) para o mesmo quadro dá a mesma pose que tocar. No laço e no vaivém, a ponta não salta na costura.
4. Errado = a ponta anda aos saltos (imagens paradas), salta ao voltar ao início, ou scrub ≠ play.

Foto: [`reports/cena5_antena_2026-10-25.png`](../reports/cena5_antena_2026-10-25.png). Rondas do dono: (1) «não está fluido» →
medido **56/120** imagens do ecrã com a mola parada ⇒ `atraso_em` (interpola entre quadros); cena tornada natural (chaves
suavizadas no corpo, antena de comprimento fixo = Spring + Limit Distance Surface a uma Base, haste Look At; a haste escura era
invisível no fundo escuro). (2) «transição inteligente nos extremos do laço/vaivém» → salto na costura medido **0,527 m** contra
**0,022 m** por imagem ⇒ órbita periódica (`atraso_no_laco`). (3) «Smoke OK».

## Feito (resumo; mecanismo e números em `03_W3_poderosas.md` §3)

| Peça | Nota |
|---|---|
| W3.0 Look At única porta de «olhar» (objectos e ossos); IK `chain=1` converte ao abrir | gate diferencial vs IK de 1 osso; M3 ver abaixo |
| Pivot · Floor · Transformation (mapear canal→canal) · Spline IK | oráculo Blender antes do código |
| Copy Point | `Kind::PontoDeForma` (variante própria por causa do `postcard`) |
| Action | lei do tempo = `ph2d_skeleton::action_time`; recurso-porta `Accoes`; o clip é a **posição** nos clips do dono |
| Spring | `Simulador` (`law::passo`), checkpoints a cada `STRIDE` 10 que guardam a ENTRADA; `atraso_em` interpola entre quadros; `atraso_no_laco` = órbita periódica + tempo virtual que não volta atrás (laço/vaivém) |
| Multi-parent | é composição (vários Child Of ponderados), sem tipo novo |
| `quadro()` | agora leva `Option<Relogio>` (`Relogio::da_vista`, dos 3 playheads) |
| Cartão | `TIPOS` 18 · `MAX_NUMEROS`/`ESCOLHAS`/`OPCOES` 5 · canais **19–37** (append) · `Unidade::Segundos` · `Estado::SemClip` |

## Superfície de colisão (para o integrador; contadores = delta)

Append em partilhados: `Kind` (+7 ao fim: Pivot, Floor, Transformation, SplineIk, PontoDeForma, Action, Spring), `Estado::SemClip`, `Unidade::Segundos`, `TIPOS` 11→18,
`MAX_NUMEROS`/`ESCOLHAS`/`OPCOES` →5 (ids `insp_constraint_*` novos), `canal_constraint` 19..37, recurso `Accoes`,
`ph2d_app_constraints::quadro(.., Option<Relogio>)` (1 chamador na shell: `fase_skeleton_drives.rs:45`).
Outra linha pode pegar os mesmos números de canal ou de `Kind` ⇒ recontar no merge (`schema-recount.py`; os canais lêem-se do código).
Um merge pode partir: ordem do enum `Kind` (postcard), ids de painel, o número de cenas do roteador (`PH2D_CONSTRAINT_SMOKE` 1..5).
Mudança de comportamento: nenhuma intencional além da W3.0 (IK `chain=1` → Look At).

## Prova de fecho

- Mutação W3.0+W3.2: **20/20 sangram**. **M3** (Look At a ignorar o limite de junta) sobreviveu ao gate diferencial; fechada por
  `a_look_at_on_a_bone_stays_inside_the_joint_limit` (vista vermelha). Depois da mutação: `cargo clean -p ph2d-constraints`,
  `-p ph2d-constraints-ecs`, `-p ph2d-app-constraints` (builds velhos enganam).
- Verificador, crates inteiras: constraints 25 · constraints-ecs 27 · app-constraints 27 · panel-inspector 469+26 · i18n 16+7 ·
  ecs lib 506 verdes; `check` do desktop verde; gates de arquitectura do editor-core 7/7; clippy `-D warnings` limpo nas crates
  tocadas; fmt limpo. (O gate `o_vocabulario_das_constraints_nao_renumera`, citado em briefings antigos, **não existe**: os
  testes de `canal_constraint` cobrem o vocabulário.) Suíte COMPLETA não corrida (regra do dono: é do integrador).

## Falta / ABERTO (medido, adiado)

1. **Flip «desenho por ângulo»**: `FlipLayer::drawing_at` é um mapa de retenção — não há canal de desenho animável; a Action no Flip
   não tem o que dirigir.
2. **Prender a ponto de simulação do Motion**: o ponto não tem id estável por ponto (`MotionCookPump`); espera por isso.
3. **Entrada da Spring = `world_pose_at` (clip activo)**: sob Arrange stack ou Autoplay do dono, a pose mostrada vem de
   `apply_from_doc` e pode diferir da entrada da mola.
4. A mola **não reage ao gizmo com o relógio parado** (entrada = animação).
5. **Action por POSIÇÃO do clip**: apagar um clip anterior desloca-a; os clips não têm id estável (o Smart Bone guarda o nome).
6. `MAX_PASSOS_DA_ORBITA = 100 000`: uma mola quase sem amortecimento pode não fechar a órbita a 1e-6.
7. Os abertos do W1d (2–7) continuam. Smoke do dono da W3: **feito**; a linha está pronta para integrar quando o Enio mandar.

## Próxima onda

**W4 — CAD no vetor** (plano §7; abre medindo `ezpz` × `fiksi` no mesmo processo; id estável por vértice = +1
`VEC_SCENE_SCHEMA_VERSION`). **Janela nova**: esta acabou a ~470 mil tokens por passo.

## Armadilhas

- `git checkout -- <ficheiro>` para desfazer um hack temporário descarta também edições NÃO comitadas nesse ficheiro: comite o WIP
  antes de qualquer edição temporária.
- Mutação deixa builds velhos: `cargo clean -p` das crates tocadas no fim.
- Dois agentes no mesmo worktree partilham `lib.rs`/`Cargo.lock`: só um comita, no fim.
- A spring tem de ser testada ENTRE quadros do ecrã, não só nos quadros do documento (foi o que escondeu as 56/120 paradas).
