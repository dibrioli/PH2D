//! **O sombreamento** do painel da cena 3D (`SCULPT3D_*`) — cortado do `sculpt3d.rs` por assunto,
//! pelo tecto de 600 linhas por ficheiro de painel.
//!
//! ⚠️ **Desceu de `ph2d-editor-core/src/ids/chrome/sculpt3d.rs` em 2026-09-12** (auditoria de arquitectura
//! A5b): quem LÊ estes ids mora nesta crate, e a fundação que 60 crates recompilam deixou de os
//! carregar.

use ph2d_a11y::NodeId;
use ph2d_tool_registry::hash_node_id;

// ── O sombreamento ──────────────────────────────────────────────────────────
/// **A CAVIDADE** — quanto a curvatura escurece a fresta e clareia a crista.
pub const SCULPT3D_CAVITY: NodeId = hash_node_id("sculpt3d.cavity");

/// Chip ligado a [`SCULPT3D_CAVITY`].
pub const SCULPT3D_CAVITY_NUM: NodeId = hash_node_id("sculpt3d.cavity_num");

/// **QUANTO DO AMBIENTE COM DIREÇÃO ENTRA** — o piso da difusa dizendo de onde
/// a luz de preenchimento vem.
///
/// ⚠️ **Ele NÃO é uma segunda luz**, e é por isso que o rótulo diz *ambiente* e
/// não *intensidade*: o número é o MESMO `ph2d_light::AMBIENT` de sempre,
/// redistribuído — céu em cima, ricochete do chão embaixo —, com a média sobre
/// todas as normais preservada. Subi-lo não clareia a peça; ele tira luz de baixo
/// e põe em cima.
pub const SCULPT3D_ENV: NodeId = hash_node_id("sculpt3d.env");

/// Chip ligado a [`SCULPT3D_ENV`].
pub const SCULPT3D_ENV_NUM: NodeId = hash_node_id("sculpt3d.env_num");

/// **QUANTO DO AO ASSADO ENTRA** — irmão da cavidade no painel, e o oposto dela
/// na origem: a cavidade é derivada e existe sempre, o AO só existe depois de um
/// bake explícito.
pub const SCULPT3D_AO: NodeId = hash_node_id("sculpt3d.ao");

/// Chip ligado a [`SCULPT3D_AO`].
pub const SCULPT3D_AO_NUM: NodeId = hash_node_id("sculpt3d.ao_num");

/// **QUANTO DO AO DE TELA ENTRA** — o irmão MEDIDO do de cima.
///
/// ⚠️ Dois knobs e não um, e a diferença não é gosto: o assado é exato, viaja no
/// arquivo e ENVELHECE a cada pincelada; este é medido todo frame, nunca fica
/// velho e só vê o que está na tela. Colapsá-los num knob só obrigaria o artista
/// a escolher entre a oclusão que ele VÊ enquanto trabalha e a que ele EXPORTA.
pub const SCULPT3D_SSAO: NodeId = hash_node_id("sculpt3d.ssao");

/// Chip ligado a [`SCULPT3D_SSAO`].
pub const SCULPT3D_SSAO_NUM: NodeId = hash_node_id("sculpt3d.ssao_num");

/// **Quanto do espalhamento sub-superficial entra** (`ph2d_mesh_render::sss`).
pub const SCULPT3D_SSS: NodeId = hash_node_id("sculpt3d.sss");

/// Chip ligado a [`SCULPT3D_SSS`].
pub const SCULPT3D_SSS_NUM: NodeId = hash_node_id("sculpt3d.sss_num");

/// **Até onde a luz viaja dentro do material**, como FRAÇÃO do maior lado da
/// peça — nunca um comprimento absoluto (ver a row).
pub const SCULPT3D_SSS_SCATTER: NodeId = hash_node_id("sculpt3d.sss_scatter");

/// Chip ligado a [`SCULPT3D_SSS_SCATTER`].
pub const SCULPT3D_SSS_SCATTER_NUM: NodeId = hash_node_id("sculpt3d.sss_scatter_num");

/// **ASSAR O AO** — o botão que mede quanto do céu cada vértice enxerga.
///
/// ⚠️ É um BOTÃO e não um passe automático porque o bake não cabe num pen-up:
/// ~338 ms na malha que a cena `=16` abre (`ph2d-sdf/tests/it/measure_ao.rs`).
pub const SCULPT3D_BAKE_AO: NodeId = hash_node_id("sculpt3d.bake_ao");

/// **ASSAR A FORMA NO SPRITE** — o objetivo 2 do módulo (`docs/3D/02.2`).
///
/// ⚠️ **Não confundir com o [`SCULPT3D_BAKE_AO`] acima:** aquele mede um canal e
/// o escreve **na MALHA**, este escreve o G-buffer inteiro **num objeto da cena
/// 2D**, que passa a acender pela forma e a sobreviver à escultura. Os dois
/// carregam a palavra *bake* e são gestos diferentes — é por isso que moram em
/// seções diferentes e os rótulos dizem o ALVO, nunca só o verbo.
///
/// ⚠️ **Ele existe porque o gesto tinha uma porta só, e ela era um atalho**
/// (`Shift+B`). Um verbo cuja única forma de ser pedido é uma combinação de
/// teclas que nada na tela menciona é um verbo que só quem o escreveu alcança.
pub const SCULPT3D_BAKE_SPRITE: NodeId = hash_node_id("sculpt3d.bake_sprite");

/// **USAR O SPRITE SELECIONADO COMO PADRÃO** — o alpha por IMAGEM.
///
/// ⚠️ **Um BOTÃO e não um chip, e a diferença não é de gosto:** a fileira de
/// chips lista NOMES (os nove padrões que são fórmulas), e uma imagem não é um
/// nome — é uma coisa para a qual se aponta. Um chip *"Image"* teria de existir
/// antes de haver pixels, e é exatamente esse estado que o
/// [`ph2d_sculpt3d::Alpha::Image`] torna inexprimível ao carregar a imagem
/// dentro de si.
///
/// ⚠️ **Ele é o irmão do *"Use as Brush Shape"* do Painter 2D**, e o gesto é o
/// mesmo: o artista seleciona um sprite no canvas e aperta. Sem sprite
/// selecionado o botão **não é pintado** — um botão que só pode falhar é a
/// forma de o artista aprender que ele não funciona.
pub const SCULPT3D_ALPHA_SPRITE: NodeId = hash_node_id("sculpt3d.alpha_sprite");

/// **QUAL LEI ACENDE O SPRITE ESCOLHIDO** — a TINTA (o passe do Painter) ou a
/// FORMA (o OpenPBR), por objecto e GRAVADA no ficheiro.
///
/// ⚠️ **A escolha é do OBJECTO e não do pincel nem da cena**, e é por isso que
/// ela não vive no `Sculpt3dUi` como os outros chips desta crate: ela viaja no
/// `.ph2dproj` (degrau `161` do `PROJECT_SCHEMA`), logo o clique é um **pedido
/// ao shell** — a mesma forma do `SCULPT3D_BAKE_SPRITE` acima.
///
/// ⚠️ **A fileira só é pintada quando o sprite escolhido JÁ está assado**: sem
/// canais não há lei para escolher, e um selector que não governa nada é o
/// controlo morto que esta crate já pagou sete vezes.
///
/// ⛔ **A POSIÇÃO é a tag**, como em toda fileira segmentada desta casa: ela é
/// comparada com o `ph2d_form_donation::lei_da_luz::Lei::ALL` por um gate da
/// crate da família (a única que vê os dois lados), logo uma lei nova que não
/// passe por aqui nasce **inalcançável** em vez de sumir em silêncio.
pub const SCULPT3D_BAKE_LAW: [NodeId; 2] = [
    hash_node_id("sculpt3d.bake_law.0"),
    hash_node_id("sculpt3d.bake_law.1"),
];

/// **COM QUE LUZ o barro é mostrado** — o modo PLANO, o RIG DO ARTISTA, e os
/// matcaps de [`ph2d_mesh_render::MATCAPS`].
///
/// ⚠️ O tamanho é `MATCAPS.len() + 2`, e os `+ 2` são o plano e o rig — que
/// **não** são matcaps. ⛔ **Era `+ 1` até 2026-09-20**, e a premissa morreu por
/// ordem do dono (*«modos de shaders além do matcap para pintar»*): a igualdade
/// continua gateada, e um chip a mais pinta uma opção que o shader não tem.
pub const SCULPT3D_MATCAP: [NodeId; 13] = [
    hash_node_id("sculpt3d.matcap.flat"),
    hash_node_id("sculpt3d.matcap.rig"),
    // ⚠️⚠️ **A POSIÇÃO é a tag.** O `event` resolve o chip por
    // [`crate::state::LightMode::from_option_index`], logo este id entra
    // EXACTAMENTE onde a porta o põe (`2`) — e não no fim, que é onde um id
    // novo costuma entrar. Pô-lo no fim ligava o chip do PBR ao último matcap
    // e deslocava os dez, em silêncio.
    hash_node_id("sculpt3d.matcap.pbr"),
    hash_node_id("sculpt3d.matcap.0"),
    hash_node_id("sculpt3d.matcap.1"),
    hash_node_id("sculpt3d.matcap.2"),
    hash_node_id("sculpt3d.matcap.3"),
    hash_node_id("sculpt3d.matcap.4"),
    hash_node_id("sculpt3d.matcap.5"),
    hash_node_id("sculpt3d.matcap.6"),
    hash_node_id("sculpt3d.matcap.7"),
    hash_node_id("sculpt3d.matcap.8"),
    hash_node_id("sculpt3d.matcap.9"),
];

/// **ACUMULAR na mesma pincelada** — o *Accumulate* do Blender.
pub const SCULPT3D_ACCUMULATE: NodeId = hash_node_id("sculpt3d.accumulate");

/// **SÓ AS FACES DA FRENTE** — a opção *Front Faces Only* do Blender, oferecida
/// nos painéis comuns de pintura dele.
pub const SCULPT3D_FRONT_FACES: NodeId = hash_node_id("sculpt3d.front_faces");

/// ⭐⭐⭐ **SÓ O QUE A SUPERFÍCIE LIGA** — o pincel deixa de agarrar o que está
/// perto no AR e longe pela superfície (o dedo vizinho, a outra metade de uma
/// dobra, a peça de trás de um modelo importado em duas partes).
///
/// ⚠️ **Nasce LIGADO, por decisão do dono** (2026-09-10: *«as duas opções devem
/// existir com a segunda como default»*). Medido: com ele desligado, `47 %` do
/// peso de um carimbo cai na peça errada quando há duas a `0,05` de distância.
/// Cena **`=39`**.
pub const SCULPT3D_SURFACE_ONLY: NodeId = hash_node_id("sculpt3d.surface_only");

/// ⭐ **PUXAR PELA NORMAL** — a caixa do [`ph2d_sculpt3d::Brush::puxa_pela_normal`]
/// (ordem do dono, 2026-09-19), oferecida aos dois verbos que têm puxão.
pub const SCULPT3D_PUXA_PELA_NORMAL: NodeId = hash_node_id("sculpt3d.puxa_pela_normal");

/// A malha de arestas desenhada por cima da forma.
pub const SCULPT3D_WIREFRAME: NodeId = hash_node_id("sculpt3d.wireframe");

/// ⭐⭐⭐⭐ **A VISTA DA GRADE do arame** — esconder a diagonal de cada triângulo.
///
/// ⛔ Ela nasceu de o dono não conseguir VER o que a medição via: as fileiras do
/// pente de topologia estão lá (`10`–`23` arestas de mediana contra `2`–`4` do
/// controlo) e afogam-se no meio das outras duas famílias de arestas.
pub const SCULPT3D_WIRE_GRADE: NodeId = hash_node_id("sculpt3d.wire_grade");

/// Azimute da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_AZ: NodeId = hash_node_id("sculpt3d.light_az");

/// Chip ligado a [`SCULPT3D_LIGHT_AZ`].
pub const SCULPT3D_LIGHT_AZ_NUM: NodeId = hash_node_id("sculpt3d.light_az_num");

/// Elevação da lâmpada selecionada, em graus.
pub const SCULPT3D_LIGHT_ELEV: NodeId = hash_node_id("sculpt3d.light_elev");

/// Chip ligado a [`SCULPT3D_LIGHT_ELEV`].
pub const SCULPT3D_LIGHT_ELEV_NUM: NodeId = hash_node_id("sculpt3d.light_elev_num");
