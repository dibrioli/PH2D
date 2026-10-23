# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 23/10 — F27 Bloco A: o lado B da junta e o motor com faixa

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`, base `main` `972f411a6f`. Anterior:
  [F26](HANDOFF_CONTINUACAO_line_UIUX_2026-10-22_F26_AS_JUNTAS.md); integração a fechar:
  [F18–F26](HANDOFF_INTEGRACAO_line_UIUX_2026-10-22_F18_A_F26.md) (§6 itens 7 e 8 + achados = esta onda).
- Briefing: fechar o ABERTO da integração F18–F26. Este é o **Bloco A** (três verificações); B, C e D ficam no §6.

## 1 — Commits desta onda

| commit | o quê |
|---|---|
| `dd3680dd77` | F27 Bloco A: 13 ficheiros, +566/−24 (código, 4 testes novos, cena `=27`, `fotografa_cena.sh`) |
| (este) | este handoff + regeneração de `README.md` desta pasta |

## 2 — Como foi construído

Três verificações do §6 do F18–F26; as três confirmadas no código. Duas curadas, uma só documentada.

### 2.1 (a) «Ponta no mundo» com `body_b == 0` (item 7)

Dois escritores, uma só porta: a chave `PONTA_NO_MUNDO = 0` (curva, `canais_junta.rs`) e o clique «Object» do seg Anchor B (painel, `fase_physics_edits.rs`) chamam `ph2d_app_physics::joint::anchor_to_world` (`crates/ph2d-app-physics/src/joint.rs`), que tirava o `JointWorldAnchor` sem olhar `body_b` ⇒ `is_anchored(false)` falso ⇒ junta dormente, corpo cai.

- **Cura na porta comum:** `anchor_to_world(world, e, on) -> bool` (era `()`); recusa `!on && !pj.names_two_bodies()`.
- **Painel:** a recusa arma o conta-gotas do corpo B (`self.joint_body_pick = Some((bits, true))`).
- **`set_joint_body(slot_b = true)`** passa a tirar o `JointWorldAnchor`: escolher o corpo B = escolher «Object» (antes o rótulo ficava «World» sobre um corpo já escolhido).

### 2.2 (b1) Trocar o marcador não punha `anchored = false`

`local_b` semeado `[0,0]` contra o prego passava a ler-se contra o corpo B (a ponta saltava para o centro de B); no inverso, o prego herdava o offset de B. Cura: `anchor_to_world` põe `anchored = false` em TODA troca (o mesmo que `set_joint_body` e a troca de tipo já faziam em `joint.rs`).

### 2.3 (b2) Ordem dos escritores: a faixa do motor vs. o re-semear da curva

`timeline_bridge::run` (`apply_prop.rs` escreve `motor_target`/`motor_speed` das faixas) corre ANTES de `canais_junta::aplicar` (`fase_timeline_drain.rs`). A troca de eixo (`MotorAxis`) e de tipo (`Kind`, quando `motor_in_metres` muda) re-semeavam `motor_target = 0` e `motor_speed = fresh`, apagando a chave do mesmo quadro.

- **Cura:** `canais_junta::aplicar(sim, drive, doc: &TimelineDoc)` (parâmetro novo); `motor_das_faixas` (`binding_for` de `JointMotorTarget`/`JointMotorSpeed`; precedente: `has_place_track`) preserva o valor que a faixa escreveu. **Sem faixa o re-semear continua** (controlo no teste).
- **Medido na app real (cena `=27`):** alvo lido quadro a quadro depois de cruzar o q24 — sem a cura `[0.000, 0.300, 0.300, 0.300, 0.300, 0.300, 0.300]` (o zero dura UM quadro: tranco do servo); com a cura `[0.300 × 7]`.

### 2.4 (c) `prop.rs:50-54` (item 8) — só o comentário mudou

O ramo existe: `AutokeyPlan::path_key` (`crates/ph2d-timeline/src/autokey.rs:256`, consumido em `shells/desktop/src/render_loop/autokey_pass.rs:287`), já gateado por `the_anchor_is_refused_under_a_stack` (exige `path_key` `Some([5,3])`). O comentário agora aponta o ramo e o gate; uma asserção nova nesse teste: `Position ∉ AUTOKEYED`.

### 2.5 Testes novos (`crates/ph2d-app-physics/src/joint_lado_b_tests.rs`)

`soltar_do_mundo_sem_corpo_b_e_recusado_pelos_dois_escritores` · `escolher_o_corpo_b_tira_o_pino_de_mundo` · `trocar_o_lado_b_re_deriva_a_ancora_de_b` · `a_faixa_do_motor_ganha_do_re_semear_da_curva`.

### 2.6 Cena `PH2D_AUTOKEY_SMOKE=27` (`crates/ph2d-app-vec/src/autokey_smoke_lado_b.rs`; despacho em `autokey_smoke.rs`, `const LADO_B = 25`)

- **Motor** (Custom, Base estática + Cart): Auto Key grava eixo+alvo nos quadros 12 e 24; a cena cruza 18→24 e lê o alvo quadro a quadro; linha `[autokey-smoke] q24, quadro a quadro…` OK/ZERADO.
- **Lamp Pin:** pino de mundo (`body_b = 0`), Lamp dinâmica + Hook estático.
- **Hinge:** Pin Door–Plank com `JointWorldAnchor`, pivô `[4.5, 2.6]`.
- Fim: Lamp Pin seleccionada, Inspector na secção Joint. Fotografada (1930×1040): na tela o seg chama-se «Anchor B: Object | World», os corpos «Body A / Body B».
- Os passos com clique (Lamp, Hinge) NÃO se fotografam (XTest ignorado na Xwayland virtual) — provados por teste de unidade.

### 2.7 Ferramenta partilhada: `docs/Components/ferramentas/fotografa_cena.sh`

O guarda dentro de `sessao.sh` comparava com `wayland-0`/`:0` fixos; nesta máquina a sessão do dono é `wayland-1` e a tela virtual nasceu `wayland-0` ⇒ a foto recusava-se a si própria. Agora `DONO_WL`/`DONO_X` vêm de fora (`WAYLAND_DISPLAY`/`DISPLAY` do chamador).

## 3 — Superfície de colisão (um merge pode partir)

- **Assinatura pública:** `ph2d_app_physics::joint::anchor_to_world(...) -> bool` (era `()`); `canais_junta::aplicar` ganhou `&TimelineDoc` (único chamador: `fase_timeline_drain.rs`). Outra linha que chame uma das duas quebra a compilar.
- **Shell:** `fase_physics_edits.rs` e `fase_timeline_drain.rs` (+3 linhas no total; `the_shell_only_shrinks` verde, dentro da folga).
- **`joint.rs` / `canais_junta.rs`** (`ph2d-app-physics`): comportamento novo (recusa; `anchored = false` na troca de marcador; faixa vence o re-semear).
- **`fotografa_cena.sh`:** ferramenta partilhada.
- **Nada foundational:** `PROJECT_SCHEMA` 193, `DOC_VERSION` 22, `AUTOKEYED` 15 intocados; nenhum id de canal novo; nenhum contrato §6.
- **Contadores como delta:** testes novos +4 (`joint_lado_b_tests`); cenas de smoke +1 (`=27`; próxima `=28`); ficheiros 13, +566/−24.

## 4 — Smoke do dono (em linguagem simples)

1. Rode `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=27 cargo run -p ph2d-host-desktop --profile smoke`.
2. Espere uns 2 segundos: a cena mexe sozinha na Timeline (o Motor) e pára com «Lamp Pin» seleccionada. No terminal tem de aparecer a linha `q24 … OK`.
3. Lâmpada: no Inspector, secção Joint, em «Anchor B» clique «Object». Tem de acender a escolha do corpo (o conta-gotas do Body B); clique no quadradinho cinzento «Hook». O Body B passa a «Hook».
4. Dobradiça: seleccione «Hinge» na Hierarquia; em «Anchor B» clique «Object» e depois «World». O anel da ponta tem de ficar onde estava.
5. Deu errado se: no passo 3 a junta some (a linha desaparece) e, ao dar Play, a lâmpada cai · no passo 4 o anel salta para o meio do quadrado «Plank» · a linha do terminal diz «ZERADO».

## 5 — Verificação

- Testes `joint` de `ph2d-app-physics`: 176 passed / 6 ignored.
- **Prova de mutação** (cada cura tirada sangra SÓ o seu teste): M1 tirar a recusa → `soltar_do_mundo…`; M2 tirar `anchored = false` → `trocar_o_lado_b…`; M3 o pick deixar de tirar o marcador → `escolher_o_corpo_b…`; M4 deixar de preservar a faixa → `a_faixa_do_motor…`.
- `clippy -D warnings --all-targets` limpo em `ph2d-app-physics`, `ph2d-app-vec`, `ph2d-timeline`, `ph2d-host-desktop`; `cargo fmt` aplicado.
- `the_shell_only_shrinks` verde.
- Suíte `nextest-impacted`: corre no fecho desta onda — **ver relato da onda** (não neste handoff).

## 6 — ABERTO, por ordem

Fechados por esta onda: itens 7 e 8 do §6 do F18–F26.

1. **Bloco B — todo campo editável é um canal.** Censo primeiro: checkboxes, dropdowns e o seletor Solid/Linear sem losango; ESCOPO do Inspector (câmera, nav, vida, HUD); dívida de vocabulário do vetor; `ph2d-panel-painter-layers` em `DIVIDA_CRATE` — a linha PainterWatercolor está igual ao `main` (0 commits): confirmar com o Enio se reabre ANTES de tocar.
2. **Bloco C —** medir `marcas()` e `linhas::layout` com N canais, losango por campo, num lote só.
3. **Bloco D —** menu do botão direito na linha de grupo do dope-sheet; ADR «o canal de qualquer propriedade» (número contado no disco, nunca escolhido).
4. **Item 9 do F18–F26:** as chaves `canal.junta.*` só foram vistas no locale base.
5. **FORA desta linha:** gravidade animável (decisão do Enio), física a meio da simulação, `IkBendSide`, Rope Length da Pulley, áudio do mixer.
6. Próxima cena de smoke: `PH2D_AUTOKEY_SMOKE=28`.
7. Smoke do dono da F27 (§4): ainda não feito.
