// Sprite pipeline (M5).
//
// Bind groups (per LLM1 audit + toji.dev convention):
//   @group(0) frame:    camera view+proj uniform
//   @group(1) material: atlas texture + sampler
// Per-instance data goes via vertex attributes (instance step mode),
// not a third bind group — cheaper.

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var atlas_tex: texture_2d<f32>;
@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) quad_pos: vec2<f32>,  // unit quad corner in [-0.5, 0.5]
    @location(1) quad_uv:  vec2<f32>,  // [0, 1]
};

struct InstanceInput {
    @location(2) world_pos: vec2<f32>,
    @location(3) size:      vec2<f32>,
    @location(4) atlas_uv:  vec4<f32>,  // u_min, v_min, u_max, v_max
    @location(5) tint:      vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv:   vec2<f32>,
    @location(1) tint: vec4<f32>,
};

@vertex
fn vs_main(v: VertexInput, i: InstanceInput) -> VertexOutput {
    let world = vec2<f32>(
        i.world_pos.x + v.quad_pos.x * i.size.x,
        i.world_pos.y + v.quad_pos.y * i.size.y,
    );
    let uv = vec2<f32>(
        i.atlas_uv.x + v.quad_uv.x * (i.atlas_uv.z - i.atlas_uv.x),
        i.atlas_uv.y + v.quad_uv.y * (i.atlas_uv.w - i.atlas_uv.y),
    );
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(world, 0.0, 1.0);
    out.uv = uv;
    out.tint = i.tint;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(atlas_tex, atlas_sampler, in.uv);
    return tex * in.tint;
}
