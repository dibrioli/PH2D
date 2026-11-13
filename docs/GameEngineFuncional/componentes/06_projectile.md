# Projectile Motion

**Estado: `fechado`** (13/11) — versão B escolhida pelo dono; smoke do dono OK 13/11. Handoffs: [06 Projectile](../handoffs/HANDOFF_CONTINUACAO_2026-11-13_06_PROJECTILE.md) · [corrigir tudo](../handoffs/HANDOFF_CONTINUACAO_2026-11-13b_CORRIGIR_TUDO.md). O texto abaixo é a ficha de ANTES da obra (o «Como está» é o estado de partida).

## Como ficou (13/11, commits `5142bb156b` e `96c68552ba`)

- **Cartão B** (`ph2d-panel-inspector/src/sections/projectile.rs`): avisos; Speed, Gravity, Range, «Faces path»; checkbox Bounce → Max Bounces + Bounciness; checkbox Homing → Target (texto + conta-gotas dentro da caixa) + aviso «No object…» + Pull; dobra «Speed change» fechada (resumo None · Speeds up · Slows down · Capped).
- **Os checkboxes gravam os números:** Bounce liga = `max_bounces` 3 (`BOUNCES_AO_LIGAR`), desliga = 0; Homing liga = `homing_accel` 30 m/s² (`PULL_AO_LIGAR`), desliga = 0. O motor não mudou.
- **Conta-gotas:** `AlvoArmado` (`ph2d-app-components/src/projectile_inspector.rs`) é recurso de sessão, sem passo de undo; o clique no palco (`escolher_alvo`) pega o objeto nomeado do topo (nunca a bala) e empurra `HomingTarget` pelo bus (um passo de undo); Esc desarma.
- **Target** agora sincroniza o nome (`sync_fabrica.rs`, `escreve_texto`) — antes ficava sempre vazio.
- **Cena** `PH2D_GAME_SMOKE=25` (`ph2d-physics-scene/src/cena_projectile.rs`); fotos em `componentes/fotos_06/`.
- **Onda «corrigir tudo» (13/11, [handoff](../handoffs/HANDOFF_CONTINUACAO_2026-11-13b_CORRIGIR_TUDO.md))**:
  - **Homing só vira:** o Pull roda a velocidade mantendo a rapidez (`advance()`); a bala não acelera mais.
  - **Target aceita arrastar** da Hierarquia (além de digitar e conta-gotas): `ObjectRef{field: Homing}`, a mesma edição do conta-gotas, só com Homing ligado.
  - **Cena 25:** fundo «Background», «Floor» e «Back Wall» sólidos (Arc pousa, Bouncy ricocheteia na parede); Missile sem Max Speed (resumo None), acaba a 0,55 m do Target.
  - As caixas Bounce/Homing agora são moldura à Godot (fotos em `fotos_checkbox/`).
- **No Play** a bala que acabou some (RunOnCopy, `ph2d-ecs/src/lifetime.rs:204`; Stop restaura): regra da casa (moedas), não defeito.

Estudos lidos:
- crítica `critica_ph2d.html#projectile`, §1–§7, com o «Vale rever?» das miniaturas.
- O estudo não tem seção própria; a crítica usou só a doc das engines, sem foto.

Base: `main` `4f46a3573c` (linha em `04e6b30234`).

## Como está (conferido no código)

- **A lei** é `ProjectileLaw`, em `crates/ph2d-projectile/src/lib.rs:58-94`. O passo é `advance()` (140-198), o fim do voo é `ended()` (218) e o ricochete fica em `bounce.rs`.
  - Não tem tempo de vida próprio: quem mata a bala pelo relógio é o componente `Lifetime`.
- **A ponte** é `drive_projectiles()`, em `crates/ph2d-physics-ecs/src/bridge/projectile.rs:89-211`.
  - O corpo é **Kinematic**: a bala anda pelo shape-cast `move_character_from` e a pose sai por `set_next_kinematic_pose` (196).
  - O alvo é resolvido pelo NOME (`posicao_por_nome`, 214) a partir do `stable_name_id` (`components/projectile.rs:37-64`).
- **Os campos:** são 10 no catálogo (`ph2d-component-desc/src/catalog/physics_fields.rs:376-386`) e todos têm leitor que decide. Nenhum controlo está morto.

  | # cat. | Campo | Padrão | Nota |
  |---|---|---|---|
  | 0 | Speed | 12 m/s (1200 px/s) | rapidez ao nascer, na direção do corpo |
  | 1–2 | Acceleration, Max Speed | 0 · 0 = sem tecto | |
  | 3 | Gravity | 0 = reto | |
  | 4–5 | Bounciness, Max Bounces | 1 · 0 | Bounciness só aparece com saltos acima de 0 |
  | 6 | Range | 0 = sem limite | em metros percorridos; uma bala barrada não gasta alcance |
  | 7 | Face Velocity | ligado | |
  | 8 | Homing | 0 = desligado | **é um número** (aceleração de perseguição), não uma chave |
  | 9 | Target | — | **texto**: digita-se o nome; só aparece com Homing acima de 0; avisa se o nome não existe |

- **O painel** é uma lista de linhas, não cartões: `ph2d-panel-inspector/src/sections/projectile.rs` (322 LOC) e `populate_projectile.rs` (faixas 24-34).
  - Avisos: No body · The body must be Kinematic · The flight is over · The clock is stopped.
- **Ao adicionar**, o corpo nasce Kinematic (`physics_seed.rs`).
- **No Add** não há item «Projectile». Há só a ajuda `object_add.teach.bullet`, que ensina a montar a bala: Sprite + Projectile Motion + Lifetime, e Weapon em quem atira.
- **O smoke** é `PH2D_PROJECTILE_SMOKE=1|2`, com quatro balas: reta, ricochete, arco e alcance curto contra uma parede. A cena não seleciona a bala, por isso não há foto do cartão.
  - A próxima cena do Game Engine é `PH2D_GAME_SMOKE=25` (`smoke.rs`: `CENAS = topdown::CENA` = 24).
- **O conta-gotas já existe** nas secções da câmera e das juntas (`sections/camera.rs`, `ids/inspector_camera.rs`, `sections/joint_pair_rows.rs`). Dá para reaproveitá-lo no Target.
- **Luau:** não achei API própria para disparar ou ler uma bala. Os campos têm nome no catálogo (`named("speed")`…), pelo caminho genérico.

### Da crítica, o que não vale

- «13 campos, com 2 condicionais» são **10 campos**, com 2 condicionais (Bounciness e Target).
- «Homing + Target, uma chave»: hoje o Homing não é chave, é um número.
  - Na versão B, a chave «Homing» precisa de um número ao lado, «Pull». O motor não muda: a chave desligada grava Pull = 0.

## O que as 9 engines fazem (crítica §2, só doc)

- Só o Unreal (ProjectileMovement) e o Construct (Bullet) têm componente pronto. Os campos deles são quase os nossos: Speed, Max Speed, Bounce, Bounciness, Homing e Gravity.
- Nas outras engines é código ou extensão: no GDevelop, a extensão traz distância máxima e tempo de vida.
- O nosso é o mais completo dos lidos; falta só parecer simples.

## Cortar e acrescentar (crítica §3–§4)

- **Juntar e dobrar:**
  - Acceleration e Max Speed viram um cartão fechado, «Speed change».
  - Homing e Target viram uma chave com título.
- **Renomear:** Face Velocity passa a «Faces path».
- **Acrescentar:**
  - Conta-gotas no Target: arrastar, escrever ou conta-gotas, como na sua decisão da câmera (02/11).
  - O mesmo aviso de nome inexistente, agora sobre um campo que também aceita o gesto.
- **Fica para depois:** as miniaturas de trajetória (versão C). Podem entrar em cima da B sem mudar nada.

## As três versões ([maquetes da crítica](file:///home/enio/Documentos/GameEngineSamples/critica_ph2d.html#projectile))

| | Hoje | A · quatro números | B · chaves Bounce e Homing | C · quatro balas |
|---|---|---|---|---|
| Campos à vista (fechado) | 8 a 10, em lista | **5** + More | 7 + 2 chaves | 2 + etiqueta |
| Gestos: quicar 3 vezes | 2 | 2 | 3 | 1 |
| Gestos: perseguir o herói | 3 (digitar) | 3 | 3 (com conta-gotas) | 4 |
| Bounciness | condicional | em More | sob a chave Bounce | miniatura |
| Homing, Target | à vista | em More | chave Homing | etiqueta |

- **A:** Speed, Gravity, Range e Max Bounces à vista, com «More · Faces its path · no homing» embaixo.
- **B:** cartão «Flight» com Speed, Gravity, Range e Faces path; abaixo, as chaves.
  - Bounce: Max Bounces e Bounciness.
  - Homing: Target, com conta-gotas, e Pull.
  - «Speed change» (Acceleration e Max Speed) fica fechado.
- **C:** quatro miniaturas, Straight · Ricochet · Arc · Short, que só preenchem os números, mais Speed e uma etiqueta.
  - Esbarra na sua decisão de 02/11 sobre «objeto pronto no Add»: precisa da sua palavra.

## Recomendação

**B**, nos cartões da casa (molde `topdown.rs` + `grupo_dobravel.rs`).
- As chaves são checkboxes, regra da casa.
- O Target com conta-gotas é o único ponto que hoje contraria uma decisão sua.
- A cena nova `PH2D_GAME_SMOKE=25` abre com a bala selecionada e o cartão à vista: as quatro balas do smoke de hoje.
