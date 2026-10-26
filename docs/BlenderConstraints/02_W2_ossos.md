# W2 — a pilha nos ossos (25/10/2026)

> Plano: [`01_plano.md`](01_plano.md) §2.2, §5 (onda 2), §6 cena 4, §7 W2. Oráculo:
> [`fixtures/blender_5_2_3_constraints_2d.json`](fixtures/blender_5_2_3_constraints_2d.json) (casos `ik_dois_ossos`,
> `ik_estica`, `ik_cadeia_3`, `stretch_to`, `follow_path`, corridos por
> [`ferramentas/oraculo_blender.py`](ferramentas/oraculo_blender.py) ANTES do código).

## 1. Medido antes de desenhar

- **O `skeleton_goal` JÁ É a IK.** `ph2d_skeleton_ecs::IkGoal` (alvo `StableId`, `chain`, `mix`, `softness` em fracção
  do alcance, `bend: BendSide`, `offset` do apontar) resolvido por `ph2d_app_skeleton::goal::solve` com
  `ph2d_skeleton::reach` (2 ossos fechado, ≥3 FABRIK, `Mixed` por junta, limite de junta depois da mistura).
  ⇒ **o goal VIRA o item IK** (plano §2.3): `Kind::Ik` na pilha, e o `IkGoal` passa a ser só leitura de ficheiro
  antigo, convertido ao abrir. O passe `goal::solve` deixa de existir.
- **A Softness já tem lei e dono:** `ph2d_skeleton::softened_distance` (racional C¹). O Blender 5.2 não tem
  Softness e o Spine só corre com interface ⇒ sem oráculo corrível; a lei fica a que o módulo já tem.
- **A composição é de herança TRS** (`Transform::compose`: rotação soma, escala multiplica por eixo, translação pelo
  afim do pai) — é ela que dá a lei do Stretch sem tocar no `reach` (§2).

### ⛔ Recusas MEDIDAS no oráculo

| Caso | O Blender 5.2.3 faz | Porque recusamos | A lei do PH2D |
|---|---|---|---|
| IK + Copy Rotation abaixo | o Copy Rotation some (W1) | a dor nº 1 | a IK corre na posição da lista |
| `ik_estica.perto` (alvo AO ALCANCE, `ik_stretch` 1) | **encolhe** o braço: 0,631 e 0,952 do comprimento, e a dobra muda | o artista liga «esticar» e o braço encolhe sem o alvo pedir | só estica além do alcance, por igual na corrente |
| `ik_estica.fora` | estica desigual (1,603 / 1,662) e dobra o braço (3,7° / 32,6°) | solver iterativo, sem lei legível | corrente recta, factor `k` igual (Spine) |
| `follow_path.L.0.25` (trecho recto de (0,0) a (4,0)) | roda o objecto **19,69°** a meio do trecho | a tangente de uma poligonal é suavizada pelo caminho inteiro | a tangente do segmento; no canto muda de uma vez |

### ✔ Leis do oráculo que adoptamos (viram gate)

| Caso | Medido | Gate |
|---|---|---|
| `ik_dois_ossos.*` | o cotovelo fica do lado em que estava (sem pólo); fora de alcance a corrente estica-se recta, sem alongar | `ik_matches_the_blender_two_bone_reach` |
| `ik_dois_ossos.espelho.*` | o rig sob `sx = −1` dá o resultado espelhado ao 1e-6 (a dobra troca de lado no mundo) | `bend_side_flips_with_the_mirror` |
| `ik_cadeia_3` | a cauda chega ao alvo, comprimentos 1 (o ângulo de cada osso não se compara: uma cadeia de 3 tem infinitas soluções) | idem, por chegada |
| `stretch_to.*` | rumo = alvo; comprimento = `d / repouso`; largura `1/√s` (`VOLUME_XZX`), `1/s` (`VOLUME_X`), `1` (`NO_VOLUME`; `VOLUME_Z` é fora do plano) | `stretch_to_matches_blender_volume` |
| `follow_path.meio_circulo.*` | posição pelo comprimento do arco; rumo = tangente (90°, 135°, 180°, −135°, −90°) | `follow_path_goes_by_arc_length_and_faces_the_tangent` |
| `follow_path.L.*` (posição) | 0,25 → (1,75; 0) · 0,75 → (4; 1,25) | idem, só a posição |

## 2. O desenho

- **`Kind::Ik(Ik { chain, softness, bend, offset, stretch })`** em `ph2d-constraints`; o alvo e a força são os do
  `Item` (a força É o *Mix*). Avaliado na posição da lista (`ph2d-constraints-ecs/src/ik.rs`): junta a corrente
  (`chain` ossos da ponta para cima), resolve com `ph2d_skeleton::reach`, escreve as rotações dos ossos da corrente
  (mistura pelo ângulo, limite de junta depois), e o dono continua a pilha com a pose nova (um Copy Rotation abaixo
  roda a mão).
- **Stretch** (`stretch = true`, a lei em `ph2d_skeleton::stretch_factor`): além do joelho da Softness
  (`alcance − s`) o factor é `k = d / joelho`; resolve-se para `raiz + (alvo − raiz)/k` (triângulos semelhantes:
  os mesmos ângulos) e o osso RAIZ da corrente ganha `sx·k` — sob a herança TRS os descendentes esticam ao longo
  de si e a ponta chega ao alvo. Contínuo no joelho (`k = 1`).
- **A lei dos 2 ossos tem UM dono:** `ph2d-node-rig-ik-2bone` passa a chamar `reach` (a cópia privada sai).
- **Ordem** (`ordem()`): quem lê a pose de um osso da corrente (pai ou alvo) e não está no caminho até à ponta
  depende da ponta; alvo dentro da corrente = `Estado::Ciclo`.
- **Quadro:** osso inteligente → pilha (objectos e ossos, topológica) — o `goal::solve` sai de
  `fase_skeleton_drives`.
- **Dois motores no mesmo osso** (osso inteligente `SolverPose` + pilha `ConstraintPose`): `PreviewDrive::driven`
  não lê a escrita de OUTRO motor da mesma entidade como «outra mão», e `substitute_authored` escreve a camada de
  baixo por último (gravar a meio do play grava o autorado, não a correcção).
- **Stretch To** (`Kind::StretchTo { rest, volume }`): aponta o +X, `sx·d/rest`, `sy·vol(s)`; `rest` guarda-se ao
  nascer (sem salto de comprimento).
- **Follow Path** (`Kind::FollowPath { by, value, follow, offset }`): alvo = uma forma; a família dá a poligonal
  em mundo (a mesma porta do `PathFollow`); por fracção ou distância, orientar pela tangente + desvio.
- **IKs acopladas** (duas IKs a puxar ossos comuns, report do dono 07/09): o goal antigo convergia AO LONGO
  dos quadros, a partir da pose viva; a IK da pilha parte do autorado em cada quadro, por isso a fase repete
  o grupo NO MESMO quadro até assentar (`assentou`: ≤ 4 ulp). Medido em
  `two_anchors_on_one_chain_reach_a_fixed_point` (Δrot máx. por passagem): 9,8e-3 · 2,3e-4 · 5,4e-6 · 2,5e-7
  · 1,19e-7 (= 1 ulp, ciclo-limite) ⇒ pára na 5.ª–6.ª; guarda `MAX_PASSAGENS = 64` só para cena que não
  converge.
