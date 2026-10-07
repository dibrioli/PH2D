# HANDOFF DE INTEGRAÇÃO — `line/Vector`: A5-a, A13, A15 e A16, duas ondas (2026-10-06/07)

> Leitor: o agente integrador (e a próxima LLM da linha). Nada aqui foi integrado nem enviado. Smoke do
> dono: **3 (A15) «OK» e 4 (A16) «parece OK» ⇒ aprovados 07/10**; os smokes 1 e 2 deram dois reports
> (BUGS #36 e #37), curados na 2.ª onda e por voltar a fotografar/smokar. Mecanismo, medições e recusas:
> fila [`01_a_fila.md`](../01_a_fila.md) §F65. Continuação que esta fecha:
> [`…_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md`](HANDOFF_line_Vector_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md).
> ⚠️ **A 1.ª onda de A5-a (anel+ordem) e de A13 (refino `k = 2`) foi SUBSTITUÍDA**: o que está no código é
> o produto da 2.ª (§2).

## 0. Onde está e o que fazer

- Worktree `Worktrees/line-Vector`, ramo `line/Vector`, HEAD `a96d73ee0`; base `0910f5315`
  (`git log --oneline 0910f5315..line/Vector`).
- Sobre DUAS ondas ainda por integrar, que entram juntas: F60–F63
  [`…_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md`](HANDOFF_INTEGRACAO_line_Vector_A_PONTA_DO_VINCO_E_AS_PASSAGENS_2026-10-05.md)
  e F64 [`…_O_ESQUELETO_E_UM_OBJECTO_2026-10-05.md`](HANDOFF_INTEGRACAO_line_Vector_O_ESQUELETO_E_UM_OBJECTO_2026-10-05.md)
  (as duas com smoke aprovado). ⚠️ **EMPILHADA sobre a `line/UIUX`** (ainda não no `main`): a UIUX entra
  ANTES; os commits dela saem do rebase por igualdade.
- Commits-chave. 1.ª onda: `16abdf7cb`/`7a95107d5` (A16) · `7fd187694` (A15) · `5b97431cf`/`5d9acc3c2`
  (A13, depois substituído) · `496af6cdd` (A5-a, depois removido) · `cbae7cd25` · `12a9c7c97`. 2.ª onda:
  `3f32ffaa3` (sem costura entre membros) · `905971af4` (meio ângulo) · `e07efbdff` · `6a7ef0a03` ·
  `c32a0d330` · `d6e15d3e7` · `a888b0318` (produto final) · `a96d73ee0` (gates da mutação).
- Ramos de experiência guardados: `a5a-lei`, `exp/a5a-lado`, `exp/a5a-marching`, `exp/a13-tiques`,
  `exp/a13-continuidade`, `exp/a13-uniao`, `a13-lei`, `exp/a13-matriz`.
- Integrar: `/pd-integracao line/Vector`, só por ordem do dono.

## 1. Superfície de colisão

| sítio | o quê | natureza |
|---|---|---|
| `Cargo.lock`, `crates/ph2d-app-vec/Cargo.toml` | 1 linha; dev-deps de teste | teste |
| **`crates/ph2d-editor-core/src/screens/hero/mode_drive.rs`** | `ModeFamily::keeps_parts_selected` (omissão `false`) + `restore_parts` (+29 linhas) | ⚠️ FUNDAÇÃO da `line/UIUX`, só ANEXO; outras famílias ao bit |
| `crates/ph2d-app-skeleton/` | `loose.rs` (`adopt_loose_roots(sim, &TimelineDoc)`), `skeleton_mode.rs` (`true`) + testes | família |
| **`crates/ph2d-skeleton/`** | `centro.rs`, `lib.rs`: a LEI DA PELE (`MisturaDoAngulo::MeioAngulo`, `PH2D_SKIN_ANGULO=circulo`) | lei |
| **`crates/ph2d-render/`** | `sprite.wgsl` e `SpriteMeshSkin::posa` (mesma lei na GPU) | GPU |
| `crates/ph2d-vec-boolean/` | `bola.rs`, `overlap`, `gancho`, `laco.rs` (módulo NOVO) | lei |
| `crates/ph2d-vec-skin/` | amostragem; vs `5d9acc3c2^` igual ao bit (o refino saiu) | lei |
| `crates/ph2d-skeleton-live/` (muitos) | `skin_desenho_amostras.rs`; `skinned_mesh.rs` volta a `{mesh, pesos}`; SAEM `skin_image_costura.rs`/`skin_image_arte.rs`, `le_malha`, os gates da costura | lei |
| `crates/ph2d-app-vec/` | cenas/sondas de teste `smoke_bone_copias_*` (fresta e réguas SAÍRAM) | teste |
| `shells/desktop/` | `render_loop/fase_object_mode.rs` (passa `&self.timeline.doc`), `tests/it/the_loose_bone_roots_get_a_skeleton.rs` (agulha `loose::adopt_loose_roots(sim,`) | shell |

- Contrato congelado (§6): **nenhum**. **`PROJECT_SCHEMA`: SEM degrau** (a `SkinnedMesh` voltou à forma
  antiga; a máscara da 1.ª onda nunca chega ao ficheiro). Sem registos, ADR nem cena nova.
- O que um merge pode partir: (1) a UIUX mexer no `mode_drive.rs` (anexo em `Step::Enter` e passo 0);
  (2) outra linha que desenhe ou posa a pele pela lei circular (agora `PH2D_SKIN_ANGULO` por omissão
  meio ângulo; gates CPU↔GPU); (3) quem reintroduzir costura entre membros — o gate
  `nada_de_um_membro_e_puxado_para_o_outro` reprova.

## 2. O que mudou para o artista

- **A5-a/A13 (cor e junta):** as partes de uma imagem presa NÃO se puxam umas às outras (cada membro
  desenha só a sua imagem; sobreposição por ordem das faces). Onde dois membros só se tocam pode ver-se
  um fio fino: são os contornos reais (decisão do dono 07/10).
- **A13:** a volta de fora de uma dobra apertada é REDONDA e anda sem saltos ao arrastar o osso; todas as
  dobras ficam um pouco mais cheias (o «Arredondar» do dono). A cor enche até à borda (vão `0,0067–0,0080`).
- **A15 ✅** o anel de mover de cada esqueleto antigo fica na cabeça do 1.º osso.
- **A16 ✅** `Tab`/seletor *Mode* com um osso escolhido mantém o osso.

## 3. Prova de fecho (2.ª onda; BASE `0910f5315`, em `a888b0318`)

| portão | resultado |
|---|---|
| `nextest-impacted.sh` | ✅ 16 100/16 100 |
| `cargo clippy --workspace --all-targets -D warnings` · `cargo fmt --all --check` | ✅ · ✅ |
| `architecture` (inclui `the_shell_only_shrinks`, tecto de LOC, censo da malha posada) | ✅ 105/105 |
| `censos-da-arvore-combinada.sh` | ✅ 114/114 |
| `ph2d-panel-registry-init --workspace` | ✅ 86/86 |
| gates de pele na GPU (paridade `2e-6`; píxel `0 px`, adaptador real) | ✅ |

- **Mutação 2.ª onda:** 36, 28 sangraram, 7 sobreviventes → gates em `a96d73ee0` (vermelho visto em
  cada; o gate de continuidade foi reescrito para conduzir o desenho do PRODUTO, antes media uma réplica
  do orçamento); a restante (orçamento só vira a `< 35°`) morre nos gates *Twist*/*Bloat*−200.
- **Números:** vão da tampa `0,0067–0,0080`, 0 células sem cor (controlo círculo `0,266–0,290`); chão de
  continuidade `0,0015` (medido `0,0013`), produto `0,0000`, controlo `0,0214`; braço a 2°: 0 más em
  16 110; 387 poses: 0 dentes a `×1/×2/×4`; µs por bake `589/581/616` (`90/110/170-140`).
- **Fotos** do binário final (`kwin --virtual`, scratch): `=3`, `=4` `36/−144`, `=6` `170/170` e
  `170/140`, `=7`. A18 visível a `170/170` (um coto curto dentro da barra esquerda).
- `rm -rf target/*/incremental` por fazer na janela de integração.

## 4. ABERTO

- **A18** pontas livres do traço na ABA da dobra extrema (`≥ ~170°`, já a `110°` uma): `Posada::tapado`
  conta triângulos virados e vizinhos da mesma folha; L1 e L2 medidas e recusadas (`exp/a13-tiques`,
  `2038739fb`). **Pergunta de produto ao dono:** deixar, ou desenhar o vinco da aba como linha.
- Só ACIMA da densidade do produto `×4`: um nó de `17,9°` em `(36,118)` da bola de fecho (gate da banda
  do braço aberto) e o gate de identidade fora do contacto falha a `×4`.
- **A17** raiz com posição ANIMADA: o anel fica na origem; re-alvejar a faixa de translação é exacto mas
  muda as linhas da timeline — pergunta de produto, não feita.
- Flip com ossos: excluído pelo dono. O precedente divergente do olho (sprites × esqueleto/Flip): herdado.

## 5. A linha do `CLAUDE.md` §5 (para o integrador; não toquei no ficheiro)

```
- **Vector + Esqueleto** — motor vetorial nativo kurbo/Vello ([ADR-0108](docs/architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)), crates `ph2d-app-vec`/`ph2d-app-skeleton`; o esqueleto é um objecto (Object·Edit·Pose); a pele mistura o MEIO ângulo. Smokes `PH2D_BUILD_SMOKE=<n>` · `PH2D_VEC_BONE_SMOKE=<n>`. Último: [handoff 07/10](docs/Skeleton/handoffs/HANDOFF_INTEGRACAO_line_Vector_A5a_A13_A15_A16_2026-10-06.md) · [docs](docs/Vector%20Module/README.md) · [BUGS](docs/Vector%20Module/BUGS_vector.md) · [história](docs/archive/estado-2026-10-02/vector.md)
```

## 6. Smoke (o dono) — ✅ APROVADO: 3 e 4 a 07/10; 1 e 2 refeitos depois da 2.ª onda, «SMoke OK» (07/10)

Comando base: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo run --profile smoke -p ph2d-host-desktop`
(com as variáveis à frente).

**6.1 As partes da imagem não se puxam**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=4 PH2D_VEC_BONE_DOBRA=36 PH2D_VEC_BONE_DOBRA2=-144 cargo run --profile smoke -p ph2d-host-desktop`
2. Olhe onde a barra de imagem de baixo encontra a outra parte.
3. Tem de acontecer: cada parte tem só a sua imagem, sem esticar nem colar na outra. Onde elas só se
   encostam pode ver-se uma linha fina entre as duas: agora é esperado (foi a sua decisão).
4. Deu errado se: um bico ou calombo de uma parte entra na outra, ou aparece cor da outra parte esticada.

**6.2 Dobra apertada: cor até à borda, volta redonda, sem saltos**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && env PH2D_VEC_BONE_SMOKE=6 PH2D_VEC_BONE_DOBRA=170,170 cargo run --profile smoke -p ph2d-host-desktop`; depois com `PH2D_VEC_BONE_DOBRA=170,140`.
2. Olhe a volta de fora de cada junta. Depois, na Hierarquia clique em **Copias bone 2**, abra o menu
   **Mode ▸ Pose Mode** e arraste a ponta desse osso devagar, para lá e para cá.
3. Tem de acontecer: a cor chega à borda; a volta de fora de cada junta é REDONDA; ao arrastar o contorno
   anda suave, sem saltos. Todas as dobras ficam um pouco mais cheias e redondas que antes (é o
   «Arredondar» que escolheu).
4. Deu errado se: a volta fica em ponta, o contorno salta ao arrastar, ou sobra uma faixa sem cor. Uma
   pontinha de traço solta dentro da cor, na dobra mais apertada, é CONHECIDA (vou perguntar-lhe sobre ela).

(Os smokes 6.3 `=3` e 6.4 `=7`, ponto de mover e `Tab` com osso, estão aprovados.)

## 7. LISTA VIVA dos abertos (copiada da continuação, actualizada)

> Regra: ao fim da janela cada item fica com o estado novo; aberto novo entra com o próximo número. O
> registo de antes dos itens fechados antes desta rodada (A1–A12, A14) vive na
> [continuação](HANDOFF_line_Vector_CONTINUACAO_A5a_A13_A15_A16_2026-10-06.md) §1.

- **A1** ✅ DECIDIDO pelo dono (04/10): o osso mais PERTO. **A2** ✅ F52 · **A3** ✅ F53 · **A4** ✅ F54 ·
  **A6** ✅ F55 · **A7** ✅ F56 · **A8** ✅ F58 · **A9** ✅ F57 · **A10** ✅ F60 · **A11** ✅ `eaa53edb1` ·
  **A12** ✅ F61 · **A14** ✅ F64.
- **A5** — (b) ⛔ recusado pelo dono (`501daabf4`). **(a) ⛔ SUPERADO (07/10, `3f32ffaa3`):** a 1.ª onda
  fechou-o com anel+ordem (`496af6cdd`), mas o dono viu que a costura colava partes de um osso na do
  outro e decidiu que NENHUMA costura entre membros fica (BUGS #36). *Registo de antes:* a cúspide da
  arte junto a uma tampa redonda; no *Zig Zag* a `~110°` os dentes de dentro fecham buraquinhos reais.
- **A13** — ✅ FEITO (07/10, `905971af4` + `a888b0318`): meio ângulo + amostragem uniforme `×2`, sem
  refino (o refino `k = 2` saltava com a pose; BUGS #37). *Registo de antes:* a `=6` com as duas juntas
  `≥ 140°` tinha zonas sem cor, traço sem cor por baixo e tiques soltos. Os tiques viraram **A18**.
- **A15** ✅ FEITO `7fd187694`, **aprovado pelo dono 07/10** (esqueleto adoptado com a origem na cabeça
  da raiz, pose ao bit; 0/12 000 fora do bit). *Registo de antes:* o anel caía na origem.
- **A16** ✅ FEITO `7a95107d5`, **aprovado 07/10** (`keeps_parts_selected` + `restore_parts`, só anexo).
  *Registo de antes:* `Tab`→Edit com um osso escolhido trocava a selecção pelo esqueleto.
- **A17** raiz com posição animada: o anel fica na origem; medir se re-alvejar a translação serve o
  artista — pergunta de produto.
- **A18** pontas livres do traço na aba da dobra extrema — ⏳ aguarda o dono (deixar, ou desenhar o vinco
  da aba como linha); ver §4.
