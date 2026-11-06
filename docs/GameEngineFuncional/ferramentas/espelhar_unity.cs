// Oráculo do Espelhar da Sprite (CLAUDE.md §0.9): corre-se, nunca se lê o fonte. Par: espelhar_godot.gd.
// Copie para Assets/Editor/ da CÓPIA de experimentos (nunca o projecto do dono):
//     xvfb-run -a <Editor>/Unity -batchmode -force-glcore -projectPath <copia> -executeMethod EspelharOraculo.Correr \
//         -quit -logFile <log>
// Mesma entrada do par: 64 × 32 px, metade esquerda vermelha, direita verde; 32 px = 1 unidade; objecto na origem;
// filho em (3, 0); colisão 1 × 1 com centro em (0,5; 0). Linhas `[EXP]` + JSON, com o que saiu nos píxeis.
using System.Globalization;
using System.Text;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.Rendering;

public static class EspelharOraculo
{
    const int W = 512, H = 288;
    const float PX = 32f;

    static string N(float v) => (Mathf.Round(v * 1000f) / 1000f).ToString(CultureInfo.InvariantCulture);
    static string P(Vector2 v) => "[" + N(v.x) + "," + N(v.y) + "]";
    static string B(bool v) => v ? "true" : "false";

    static string Le(Texture2D img)
    {
        var px = img.GetPixels32();
        Vector2 sr = Vector2.zero, sg = Vector2.zero;
        int nr = 0, ng = 0, x0 = W, x1 = -1, y0 = H, y1 = -1;
        for (int y = 0; y < H; y++)
            for (int x = 0; x < W; x++)
            {
                var c = px[y * W + x];
                if (c.a < 128) continue;
                var p = new Vector2(x + 0.5f, y + 0.5f);
                if (c.r > c.g) { sr += p; nr++; } else { sg += p; ng++; }
                x0 = Mathf.Min(x0, x); x1 = Mathf.Max(x1, x); y0 = Mathf.Min(y0, y); y1 = Mathf.Max(y1, y);
            }
        Vector2 Mundo(Vector2 s, int n) => new Vector2((s.x / n - W / 2f) / PX, (s.y / n - H / 2f) / PX);
        var b = new StringBuilder();
        b.Append("\"verm\":").Append(nr > 0 ? P(Mundo(sr, nr)) : "null");
        b.Append(",\"verde\":").Append(ng > 0 ? P(Mundo(sg, ng)) : "null");
        if (x1 >= 0)
            b.Append(",\"caixa\":[" + N((x0 - W / 2f) / PX) + "," + N((y0 - H / 2f) / PX) + "," +
                     N((x1 + 1 - W / 2f) / PX) + "," + N((y1 + 1 - H / 2f) / PX) + "]");
        return b.ToString();
    }

    public static void Correr()
    {
        // A cópia usa URP; aqui interessa o desenho do SpriteRenderer, e o pipeline embutido responde a Camera.Render.
        GraphicsSettings.defaultRenderPipeline = null;
        QualitySettings.renderPipeline = null;
        EditorSceneManager.NewScene(NewSceneSetup.EmptyScene);
        Physics2D.simulationMode = SimulationMode2D.Script;

        var tex = new Texture2D(64, 32, TextureFormat.RGBA32, false) { filterMode = FilterMode.Point };
        for (int y = 0; y < 32; y++)
            for (int x = 0; x < 64; x++)
                tex.SetPixel(x, y, x < 32 ? Color.red : Color.green);
        tex.Apply();

        var rt = new RenderTexture(W, H, 24, RenderTextureFormat.ARGB32);
        var cam = new GameObject("cam").AddComponent<Camera>();
        cam.orthographic = true;
        cam.orthographicSize = H / 2f / PX;
        cam.transform.position = new Vector3(0, 0, -10);
        cam.clearFlags = CameraClearFlags.SolidColor;
        cam.backgroundColor = new Color(0, 0, 0, 0);
        cam.targetTexture = rt;
        var leitura = new Texture2D(W, H, TextureFormat.RGBA32, false);
        var mat = new Material(Shader.Find("Sprites/Default"));

        var pivos = new (string nome, Vector2 p)[]
        {
            ("centro", new Vector2(0.5f, 0.5f)), ("esquerda", new Vector2(0f, 0.5f)), ("pe", new Vector2(0.5f, 0f)),
        };
        var modos = new[] { "nada", "flip_x", "escala_x", "flip_e_escala_x", "flip_y", "escala_y" };

        foreach (var pivo in pivos)
        {
            var sprite = Sprite.Create(tex, new Rect(0, 0, 64, 32), pivo.p, PX);
            foreach (var modo in modos)
                foreach (var rot in new[] { 0f, 30f })
                {
                    var o = new GameObject("o");
                    var s = o.AddComponent<SpriteRenderer>();
                    s.sharedMaterial = mat;
                    s.sprite = sprite;
                    var col = o.AddComponent<BoxCollider2D>();
                    col.size = Vector2.one;
                    col.offset = new Vector2(0.5f, 0);
                    var f = new GameObject("f").transform;
                    f.SetParent(o.transform, false);
                    f.localPosition = new Vector3(3, 0, 0);
                    o.transform.rotation = Quaternion.Euler(0, 0, rot);
                    var e = Vector3.one;
                    if (modo == "flip_x") s.flipX = true;
                    if (modo == "flip_y") s.flipY = true;
                    if (modo == "flip_e_escala_x") s.flipX = true;
                    if (modo == "escala_x" || modo == "flip_e_escala_x") e.x = -1;
                    if (modo == "escala_y") e.y = -1;
                    o.transform.localScale = e;
                    Physics2D.SyncTransforms();

                    cam.Render();
                    RenderTexture.active = rt;
                    leitura.ReadPixels(new Rect(0, 0, W, H), 0, 0);
                    leitura.Apply();
                    RenderTexture.active = null;

                    var fim = Le(leitura);
                    Debug.Log("[EXP] {" + fim + ",\"motor\":\"unity\",\"pivo\":\"" + pivo.nome + "\",\"modo\":\"" + modo +
                              "\",\"rot\":" + N(rot) + ",\"filho\":" + P(f.position) + ",\"colisao\":" + P(col.bounds.center) +
                              ",\"toca_+0,5\":" + B(Physics2D.OverlapPoint(new Vector2(0.5f, 0)) != null) +
                              ",\"toca_-0,5\":" + B(Physics2D.OverlapPoint(new Vector2(-0.5f, 0)) != null) +
                              ",\"campo_escala\":" + P(o.transform.localScale) + ",\"campo_rot\":" + N(o.transform.eulerAngles.z) + "}");
                    Object.DestroyImmediate(o);
                }
        }
    }
}
