use super::*;
use ph2d_ecs::AnimationTag;

/// Uma imagem RGBA `w × h` opaca onde `opaco(x, y)`.
fn imagem(w: u32, h: u32, opaco: impl Fn(u32, u32) -> bool) -> Vec<u8> {
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| [200, 90, 40, u8::from(opaco(x, y)) * 255])
        .collect()
}

/// A `idle.png` do oráculo (128 × 32, 4 × 1 de 32 px, toda opaca) a 32 px/m, Pivot no pé (Bottom).
fn idle() -> FirstSheet {
    FirstSheet {
        size: [1.0, 1.0],
        pivot: [0.0, -0.5],
        flip: [false, false],
        cell_px: [32, 32],
        opaque: Some([0.0, 0.0, 1.0, 1.0]),
    }
}

fn run_png() -> Vec<u8> {
    imagem(288, 96, |_, _| true)
}

fn run(rgba: &[u8]) -> NewSheetImage<'_> {
    NewSheetImage {
        asset: AssetId::from_bytes(b"run.png"),
        rgba,
        px: [288, 96],
        grid: [6, 2],
    }
}

/// O pé do desenho, em m a partir do pivô: o fundo do quad (`anchor` é do pivô ao centro do quad).
fn pe(anchor: [f32; 2], size: [f32; 2]) -> f32 {
    anchor[1] - size[1] / 2.0
}

/// O oráculo (Godot 4.7.2, `godot_saida.txt`): `centered=true` põe o centro de cada quadro no pivô e o pé salta
/// de −16 para −24 px ao passar de `idle` a `run`. A nossa lei leva a FRAÇÃO do Pivot: o pé fica.
///
/// *Mutação: o Pivot da folha nova ser o centro (o do Godot) ⇒ o pé desce 0,25 m e reprova.*
#[test]
fn os_pes_ficam_no_sitio_ao_trocar_de_folha() {
    let png = run_png();
    let first = idle();
    let s = new_sheet(&first, run(&png));
    let pe_idle = pe([-first.pivot[0], -first.pivot[1]], first.size);
    assert!((pe(s.anchor, s.size) - pe_idle).abs() < 1e-6, "{s:?}");
    let godot = pe([0.0, 0.0], s.size) - pe([0.0, 0.0], first.size);
    assert!(
        (godot + 0.25).abs() < 1e-6,
        "o salto do oráculo: 8 px a 32 px/m"
    );
}

/// *Mutação: o tamanho da folha nova ser o da primeira ⇒ o pixel encolhe e reprova.*
#[test]
fn o_pixel_tem_o_mesmo_tamanho_nas_duas() {
    let png = run_png();
    let s = new_sheet(&idle(), run(&png));
    assert_eq!((s.hframes, s.vframes), (6, 2));
    assert_eq!(s.size, [1.5, 1.5], "48 px a 32 px/m");
}

/// A sobra transparente não conta: o pé é o fundo do DESENHO, não da célula.
#[test]
fn a_folha_nova_copia_o_pivot_da_primeira_pela_caixa_opaca() {
    // Cada célula de 48 px tem desenho só nas linhas 0..36 (o resto transparente).
    let png = imagem(288, 96, |_, y| y % 48 < 36);
    let s = new_sheet(&idle(), run(&png));
    // O pivô (y = −anchor) cai no fundo do desenho: 36/48 da célula a partir de cima.
    let pivo_y = -s.anchor[1];
    assert!((pivo_y - (0.5 - 36.0 / 48.0) * 1.5).abs() < 1e-5, "{s:?}");
    // E à esquerda (Bottom Left) com espelho: o lado esquerdo do que se vê.
    let mut first = idle();
    first.pivot = [-0.5, -0.5];
    first.flip = [true, false];
    let png = imagem(288, 96, |x, _| x % 48 >= 12);
    let s = new_sheet(&first, run(&png));
    // Na imagem o desenho começa no px 12; espelhada, acaba a 12 px da direita — o «esquerdo» visto é a borda −0,75.
    assert!((-s.anchor[0] + 0.75).abs() < 1e-5, "{s:?}");
}

fn biblioteca() -> (SpriteAnimations, SpriteAnimFrames) {
    let mut lib = SpriteAnimations::new();
    lib.insert(AnimationTag::new("idle", 0, 3)).unwrap();
    let mut run = AnimationTag::new("run", 0, 5);
    run.sheet = 1;
    lib.insert(run).unwrap();
    let mut jump = AnimationTag::new("jump", 0, 1);
    jump.sheet = 2;
    lib.insert(jump).unwrap();
    lib.insert(AnimationTag::new("mix", 0, 0)).unwrap();
    let mut frames = SpriteAnimFrames::default();
    frames.set(
        "mix",
        vec![
            FrameRef::cell(0),
            FrameRef::Cell { sheet: 1, cell: 4 },
            FrameRef::Cell { sheet: 2, cell: 1 },
        ],
    );
    (lib, frames)
}

/// *Mutações: apagar em vez de esvaziar ⇒ «mix» encolhe; não reindexar ⇒ a «jump» aponta para a folha 2 que já não
/// existe; o intervalo não virar lista ⇒ a «run» toca a folha 1 nova (a que era a 2).*
#[test]
fn delete_sheet_deixa_quadros_vazios_no_lugar() {
    let (mut lib, mut frames) = biblioteca();
    delete_sheet(&mut lib, &mut frames, &[4, 12, 2], 1);
    assert_eq!(
        lib.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["idle", "run", "jump", "mix"],
        "nenhuma animação sai"
    );
    assert_eq!(
        frames.get("mix").unwrap(),
        [
            FrameRef::cell(0),
            FrameRef::Empty,
            FrameRef::Cell { sheet: 1, cell: 1 }
        ]
    );
    assert_eq!(
        frames.get("run").unwrap(),
        [FrameRef::Empty; 6],
        "o intervalo vira buracos do mesmo comprimento"
    );
    assert_eq!(lib.get("jump").unwrap().sheet, 1, "a folha 2 desce para 1");
    assert_eq!(lib.get("idle").unwrap().sheet, 0);
    assert_eq!(
        frames.get("idle"),
        None,
        "o intervalo de outra folha fica intervalo"
    );
}

#[test]
fn a_animacao_com_vazios_fica_em_alerta() {
    let (mut lib, mut frames) = biblioteca();
    assert_eq!(empty_count(Some(&frames), "mix"), 0);
    delete_sheet(&mut lib, &mut frames, &[4, 12, 2], 1);
    assert_eq!(empty_count(Some(&frames), "mix"), 1);
    assert_eq!(empty_count(Some(&frames), "run"), 6);
    assert_eq!(empty_count(Some(&frames), "idle"), 0);
    assert!(all_empty(Some(&frames), "run") && !all_empty(Some(&frames), "mix"));
}

/// *Mutação: tirar os vazios também da toda vazia ⇒ a «run» volta ao intervalo `0..=5` da folha 0 e reprova.*
#[test]
fn remove_empty_frames_tira_so_os_vazios() {
    let (mut lib, mut frames) = biblioteca();
    delete_sheet(&mut lib, &mut frames, &[4, 12, 2], 1);
    assert!(remove_empty_frames(&lib, &mut frames));
    assert_eq!(
        frames.get("mix").unwrap(),
        [FrameRef::cell(0), FrameRef::Cell { sheet: 1, cell: 1 }]
    );
    assert_eq!(
        frames.get("run").unwrap(),
        [FrameRef::Empty; 6],
        "a toda vazia fica"
    );
    assert!(!remove_empty_frames(&lib, &mut frames), "nada mais a tirar");
}

#[test]
fn delete_empty_animations_so_apaga_as_todas_vazias() {
    let (mut lib, mut frames) = biblioteca();
    assert!(empty_animations(&lib, Some(&frames)).is_empty());
    delete_sheet(&mut lib, &mut frames, &[4, 12, 2], 1);
    assert_eq!(empty_animations(&lib, Some(&frames)), ["run"]);
}
