use super::*;

const POSES: &str =
    include_str!("../../../../docs/GameEngineFuncional/ferramentas/boneco_poses.json");

fn sheet() -> (Vec<SrgbRgba>, usize) {
    (
        hero_sheet_pixels(),
        (HERO_SHEET_CELL_PX * HERO_SHEET_COLS) as usize,
    )
}

fn cell_alpha(px: &[SrgbRgba], w: usize, row: u32, col: u32) -> Vec<u8> {
    let n = HERO_SHEET_CELL_PX as usize;
    (0..n * n)
        .map(|k| px[(row as usize * n + k / n) * w + col as usize * n + k % n].a())
        .collect()
}

fn usada(row: u32, col: u32) -> bool {
    let tag = &HERO_SHEET_TAGS[row as usize];
    (tag.from..=tag.to).contains(&(row * HERO_SHEET_COLS + col))
}

/// A linha mais baixa com cobertura de uma célula, em texels a partir do topo.
fn baixo(a: &[u8]) -> usize {
    let n = HERO_SHEET_CELL_PX as usize;
    (0..n)
        .rev()
        .find(|y| (0..n).any(|x| a[y * n + x] > 64))
        .expect("desenho")
}

/// A folha é a mesma em toda corrida — grava-se pela chave e reabre igual.
#[test]
fn a_folha_e_deterministica() {
    assert_eq!(hero_sheet_pixels(), hero_sheet_pixels());
    let (px, w) = sheet();
    assert_eq!(
        px.len(),
        w * (HERO_SHEET_CELL_PX * HERO_SHEET_ROWS) as usize
    );
}

/// Cada célula usada tem margem transparente de 2 texels (o filtro bilinear não traz a vizinha) e desenho; as
/// que nenhuma tag usa ficam vazias.
#[test]
fn cada_celula_tem_margem_transparente() {
    let (px, w) = sheet();
    let n = HERO_SHEET_CELL_PX as usize;
    for row in 0..HERO_SHEET_ROWS {
        for col in 0..HERO_SHEET_COLS {
            let a = cell_alpha(&px, w, row, col);
            if !usada(row, col) {
                assert!(
                    a.iter().all(|v| *v == 0),
                    "linha {row} célula {col} devia estar vazia"
                );
                continue;
            }
            for (k, alfa) in a.iter().enumerate() {
                let (x, y) = (k % n, k / n);
                if x < 2 || y < 2 || x >= n - 2 || y >= n - 2 {
                    assert_eq!(
                        *alfa, 0,
                        "linha {row} célula {col} pinta a margem em ({x},{y})"
                    );
                }
            }
            assert!(
                a.iter().filter(|v| **v > 128).count() > n * n / 8,
                "linha {row} célula {col} vazia"
            );
        }
    }
}

/// Uma tag por linha, a começar na 1.ª célula, com um tempo por célula.
#[test]
fn as_tags_sao_as_linhas() {
    for (i, tag) in HERO_SHEET_TAGS.iter().enumerate() {
        assert_eq!(tag.from, i as u32 * HERO_SHEET_COLS, "{}", tag.name);
        assert!(
            tag.to >= tag.from && tag.to < tag.from + HERO_SHEET_COLS,
            "{}",
            tag.name
        );
        assert_eq!(
            tag.per_frame_ms.len() as u32,
            tag.to - tag.from + 1,
            "{}",
            tag.name
        );
        assert!(tag.per_frame_ms.iter().all(|ms| *ms > 0), "{}", tag.name);
    }
}

/// ⭐ Os desenhos e os tempos são os da referência do Blender: o número de desenhos, os ms de cada um e se a
/// animação repete — quem refaz a referência e não refaz a folha (ou o contrário) fica vermelho aqui.
#[test]
fn os_tempos_sao_os_da_referencia_do_blender() {
    let poses: serde_json::Value = serde_json::from_str(POSES).expect("poses.json");
    for tag in HERO_SHEET_TAGS {
        let ciclo = &poses["ciclos"][tag.name];
        let ms: Vec<u32> = ciclo["desenhos"]
            .as_array()
            .expect("desenhos")
            .iter()
            .map(|d| d["ms"].as_u64().expect("ms") as u32)
            .collect();
        assert_eq!(ms, tag.per_frame_ms, "{}", tag.name);
        assert_eq!(ciclo["repete"].as_bool(), Some(!tag.once), "{}", tag.name);
    }
    let margem = poses["cabecalho"]["celula"]["sola_margem"]
        .as_f64()
        .expect("margem") as f32;
    assert!((HERO_SHEET_SOLE_BELOW_CENTRE - (0.5 - margem)).abs() < 1e-6);
}

/// Correr MEXE: duas células da linha não são iguais (uma animação parada mentiria no smoke).
#[test]
fn cada_animacao_mexe() {
    let (px, w) = sheet();
    for tag in HERO_SHEET_TAGS {
        let row = tag.from / HERO_SHEET_COLS;
        let a0 = cell_alpha(&px, w, row, 0);
        let differ = (1..=tag.to - tag.from).any(|c| cell_alpha(&px, w, row, c) != a0);
        assert!(differ, "{} tem as células todas iguais", tag.name);
    }
}

/// A sola declarada é a do desenho (o personagem paira a esta altura — um número errado põe-no a flutuar ou
/// a afundar): parado, os pés estão nela em TODA célula; a correr, o pé de apoio pousa nela e nenhum desce
/// abaixo dela.
#[test]
fn a_sola_declarada_e_a_do_desenho() {
    let (px, w) = sheet();
    let n = HERO_SHEET_CELL_PX as usize;
    let sola = 0.5 * n as f32 + HERO_SHEET_SOLE_BELOW_CENTRE * n as f32;
    for tag in &HERO_SHEET_TAGS[..2] {
        let row = tag.from / HERO_SHEET_COLS;
        let baixos: Vec<f32> = (0..=tag.to - tag.from)
            .map(|c| baixo(&cell_alpha(&px, w, row, c)) as f32 + 1.0)
            .collect();
        for (c, b) in baixos.iter().enumerate() {
            assert!(
                *b <= sola + 1.5,
                "{} {c}: afunda — sola no texel {b}, a declarada {sola}",
                tag.name
            );
            if tag.name == "idle" {
                assert!(
                    (b - sola).abs() <= 1.5,
                    "idle {c}: sola no texel {b}, a declarada {sola}"
                );
            }
        }
        let apoio = baixos.iter().filter(|b| (**b - sola).abs() <= 1.5).count();
        assert!(
            apoio * 4 >= baixos.len() * 3,
            "{}: só {apoio} desenhos pousam na sola",
            tag.name
        );
    }
}

/// `PH2D_HERO_SHEET_RGBA=<ficheiro>` grava a folha em RGBA cru (para a olhar; não é um gate).
#[test]
fn grava_a_folha_para_olhar() {
    if let Ok(path) = std::env::var("PH2D_HERO_SHEET_RGBA") {
        let bytes: Vec<u8> = hero_sheet_pixels()
            .iter()
            .flat_map(|p| [p.r(), p.g(), p.b(), p.a()])
            .collect();
        std::fs::write(path, bytes).expect("grava");
    }
}
