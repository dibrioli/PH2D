//! **A PORTA DE ENTRADA da escultura** — um arquivo de malha soltado na janela
//! vira peças da cena.
//!
//! Filho (`#[path]`) de [`super`] pelo motivo dos outros irmãos: ele constrói
//! [`SceneObject`]s e mexe na lista.
//!
//! ⚠️ **O `import_obj` existia e NÃO tinha chamador nenhum** (conferido por
//! grep, não por leitura): o leitor de OBJ shipava desde a W1, com onze gates, e
//! o artista não tinha um gesto que trouxesse um arquivo. Uma porta sem
//! corredor é código morto com a suíte verde — esta wave é o corredor.
//!
//! ## As duas dívidas que o import pagava, e o que decidiu cada uma
//!
//! **CENTRAR é exigido por um MECANISMO.** O espelho da escultura reflete
//! negando uma coordenada (`ph2d_sculpt3d::Symmetry::signs`), então **o plano de
//! simetria É a origem local**: uma malha que o autor deixou longe do zero
//! espelha em torno de um plano que não passa por ela. A `Pose` não resolve —
//! ela move o objeto no MUNDO, não a origem local em relação à geometria. Por
//! isso a geometria é reescrita ([`ph2d_mesh::Mesh::recenter`]) e o
//! deslocamento retirado vai para a `Pose`, onde preserva o ARRANJO do arquivo:
//! uma cabeça e um corpo continuam onde o autor os pôs.
//!
//! **NORMALIZAR não é exigido por nenhum, então mora na `Pose`.** O pincel mede
//! PIXELS DE TELA (W4) e a câmera enquadra, então o tamanho absoluto não alcança
//! gesto nenhum; o que ele alcança é a **convivência** — desde a W8.1 a cena é
//! uma lista, e uma peça de 300 unidades ao lado de uma esfera de 1 torna a
//! segunda invisível. Um fator para o arquivo INTEIRO, na escala da pose: os
//! tamanhos relativos das peças sobrevivem e os números do autor não são
//! reescritos.

use ph2d_mesh::{ImportedPiece, Pose};

use super::Sculpt3dScene;

/// O que uma peça importada mede no mundo, no maior eixo.
///
/// ⚠️ **Número de CONVIVÊNCIA, não teto de recurso.** Ele é o `2.0` do
/// diâmetro das primitivas que a cena já cria (`shapes::uv_sphere(_, _, 1.0)`),
/// para um arquivo chegar do tamanho do que já está na mesa. Não há recurso que
/// ele proteja: o pincel mede pixels de tela e a câmera enquadra o que houver.
const IMPORT_SPAN: f32 = 2.0;

/// As extensões que este módulo reconhece.
///
/// ⚠️ **Por EXTENSÃO, e não por conteúdo** — e aqui isso não é preguiça: um
/// `.obj` nunca é uma imagem, então a pergunta *"de quem é este arquivo?"* não
/// tem ambiguidade a resolver. (O roteador de decode do áudio olha o CONTEÚDO
/// porque lá as extensões mentem — um `.ogg` pode ser Vorbis ou Opus.)
pub(crate) fn is_mesh_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("obj"))
}

/// **Onde cada peça do arquivo vai parar** — a pose de cada uma, já centrada e
/// já na escala de convivência.
///
/// ⚠️ Função PURA, e é ela que os gates dirigem: montar as peças na cena exige
/// um `wgpu::Device`, e a decisão que pode estar errada (*a origem local ficou
/// no centro? o arranjo sobreviveu? os tamanhos relativos sobreviveram?*) não
/// tem nada a ver com um device.
pub(crate) fn place(pieces: &mut [ImportedPiece], anchor: [f32; 3]) -> Vec<Pose> {
    // 1. Cada peça ganha a própria origem no centro dela — o plano do espelho.
    //    O que sai daqui é o ARRANJO: onde a peça estava no arquivo.
    let offsets: Vec<[f32; 3]> = pieces.iter_mut().map(|p| p.mesh.recenter()).collect();

    // 2. A caixa do ARQUIVO INTEIRO, medida depois de centrar (é a união das
    //    caixas já deslocadas). Um fator por arquivo, e não por peça: por peça,
    //    um olho e um corpo chegariam do mesmo tamanho.
    let mut lo = [f32::MAX; 3];
    let mut hi = [f32::MIN; 3];
    for (p, o) in pieces.iter().zip(&offsets) {
        let (b_lo, b_hi) = (p.mesh.bounds().min, p.mesh.bounds().max);
        for k in 0..3 {
            lo[k] = lo[k].min(b_lo[k] + o[k]);
            hi[k] = hi[k].max(b_hi[k] + o[k]);
        }
    }
    let span = (0..3).fold(0.0f32, |m, k| m.max(hi[k] - lo[k]));
    // ⚠️ Um arquivo degenerado (tudo num ponto) não pode virar uma divisão por
    // zero que leva a escala a infinito — e a `Pose::new` a clamparia no piso,
    // o que é pior: o objeto sumiria sem ninguém saber por quê.
    let scale = if span > 1e-6 { IMPORT_SPAN / span } else { 1.0 };
    let mid = [
        (lo[0] + hi[0]) * 0.5,
        (lo[1] + hi[1]) * 0.5,
        (lo[2] + hi[2]) * 0.5,
    ];

    offsets
        .iter()
        .map(|o| {
            Pose::new(
                [
                    anchor[0] + (o[0] - mid[0]) * scale,
                    anchor[1] + (o[1] - mid[1]) * scale,
                    anchor[2] + (o[2] - mid[2]) * scale,
                ],
                scale,
            )
        })
        .collect()
}

impl Sculpt3dScene {
    /// **Onde o próximo arquivo encosta** — à DIREITA do que já está na mesa,
    /// nunca por cima.
    pub(crate) fn import_anchor(&self) -> [f32; 3] {
        let b = self.world_bounds();
        let right = if b.is_empty() { 0.0 } else { b.max[0] };
        [right + IMPORT_SPAN, 0.0, 0.0]
    }

    /// **Põe na cena peças já colocadas** ([`place`]).
    ///
    /// ⚠️ Cada uma entra pelo `push_object`, que é a porta que GRAVA UNDO — um
    /// import trazido por engano some com um Ctrl+Z, como qualquer peça.
    pub(crate) fn push_placed(
        &mut self,
        pieces: impl IntoIterator<Item = (ImportedPiece, Pose)>,
        aspect: f32,
    ) {
        for (p, pose) in pieces {
            self.push_object(p.mesh, pose);
        }
        self.frame_all(aspect);
    }

    /// A pose da peça `i` — a porta que o import usa para assentar a PRIMEIRA
    /// peça de uma cena recém-criada.
    ///
    /// ⚠️ Ela existe porque o `Sculpt3dScene::new` **não recebe pose**: a cena
    /// nasce de uma malha, e a colocação é decisão de quem importa. Sem isto, a
    /// primeira peça de um arquivo seria a única a não ser centrada — o defeito
    /// exato que esta wave paga, sobrevivendo no caso mais comum (um arquivo, uma
    /// peça, nenhuma cena aberta).
    pub(crate) fn set_pose(&mut self, i: usize, pose: Pose) {
        if let Some(o) = self.objects.get_mut(i) {
            o.pose = pose;
        }
    }
}

impl crate::app_state::App {
    /// **Lê cada arquivo de malha soltado e o põe na cena.**
    ///
    /// ⚠️ **Soltar uma malha ARMA o módulo**, mesmo sem a variável do smoke — é
    /// a mesma lei do load de projeto (W8.3): a alternativa seria o artista
    /// soltar um modelo e não acontecer nada, com o app sabendo ler o arquivo.
    ///
    /// ⚠️ **Um arquivo ilegível não derruba os outros.** Aqui não vale a lei da
    /// timeline que o load de projeto segue (*recusar tudo*): lá o risco é o
    /// próximo Ctrl+S gravar o vazio por cima da obra; aqui cada arquivo é um
    /// gesto independente, e recusar o lote puniria os que estão bons.
    pub(crate) fn sculpt3d_import_files(&mut self, paths: &[std::path::PathBuf]) {
        if paths.is_empty() {
            return;
        }
        let mut loaded: Vec<ImportedPiece> = Vec::new();
        for path in paths {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
            match std::fs::read_to_string(path)
                .map_err(|e| e.to_string())
                .and_then(|t| ph2d_mesh::import_obj(&t).map_err(|e| e.to_string()))
            {
                Ok(pieces) => loaded.extend(pieces),
                Err(e) => self.sculpt3d_toast(format!("Mesh refused: {name} ({e})")),
            }
        }
        if loaded.is_empty() {
            return;
        }
        let n = loaded.len();
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let size = gfx.surface.size();
        let aspect = size.width as f32 / size.height.max(1) as f32;
        // ⚠️ **A colocação roda sobre a lista INTEIRA, antes de qualquer peça
        // entrar** — inclusive a que vai abrir uma cena nova. Colocá-la depois
        // deixaria a primeira peça de um arquivo sem centrar, que é o defeito
        // desta wave sobrevivendo no caso mais comum.
        let anchor = gfx
            .sculpt3d
            .as_ref()
            .map_or([0.0, 0.0, 0.0], Sculpt3dScene::import_anchor);
        let poses = place(&mut loaded, anchor);
        let mut placed = loaded.into_iter().zip(poses);
        if let Some(scene) = gfx.sculpt3d.as_mut() {
            scene.push_placed(placed, aspect);
        } else {
            // ⚠️ A 1ª peça abre a cena e as demais entram pela porta da lista —
            // o mesmo desenho do smoke, pelo mesmo motivo: um construtor que
            // recebesse a lista inteira seria a segunda resposta a *como um
            // objeto entra na cena*.
            let device = std::sync::Arc::clone(&gfx.surface.gpu().device);
            let (first, pose) = placed.next().expect("a lista não está vazia");
            let mut scene = Sculpt3dScene::new(&device, first.mesh, aspect);
            scene.set_pose(0, pose);
            scene.push_placed(placed, aspect);
            gfx.sculpt3d = Some(scene);
        }
        self.sculpt3d_toast(format!("Imported {n} mesh piece(s)"));
    }

    fn sculpt3d_toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor::Toast::info(msg));
        }
    }
}

#[cfg(test)]
#[path = "sculpt3d_import_tests.rs"]
mod tests;
