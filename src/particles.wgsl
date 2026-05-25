struct VertexInput {
    @location(0) quad_position: vec3<f32>,
    @location(1) particle_position: vec3<f32>,
    @location(2) colour: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) quad_position: vec3<f32>,
    @location(2) colour: vec3<f32>,
};

struct CameraUniform {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    opengl_to_wgpu: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.colour = in.colour;

    let right = vec3<f32>(
        camera.view[0][0],
        camera.view[1][0],
        camera.view[2][0]
    );

    let up = vec3<f32>(
        camera.view[0][1],
        camera.view[1][1],
        camera.view[2][1]
    );

    let world_pos =
        in.particle_position +
        right * in.quad_position.x +
        up * in.quad_position.y;

    out.clip_position =
        camera.opengl_to_wgpu *
        camera.proj *
        camera.view *
        vec4<f32>(world_pos, 1.0);

    out.quad_position = in.quad_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let d = length(in.quad_position);
    let a = exp(-d*d * 8.);
    return vec4<f32>(in.colour,a);
}


@group(0) @binding(0) var<storage, read_write> positions: array<vec3<f32>>;
@group(0) @binding(1) var<storage, read_write> velocities: array<vec3<f32>>;
@group(0) @binding(2) var<storage, read_write> colours: array<vec3<f32>>;

@compute @workgroup_size(256)
fn cs_main(@builtin(global_invocation_id) global_id : vec3<u32>){
    var velocity = velocities[global_id.x];
    var position = positions[global_id.x];

    let flow = normalize(curl_noise(position * 0.01));
    velocity = mix(velocity, flow , 0.01);
    velocity *= 0.96;
    position += velocity * 2.0;

    velocities[global_id.x] = velocity;
    positions[global_id.x] = position;

    var colour = colours[global_id.x];

    let t = dot(flow, vec3<f32>(0.0, 1.0, 0.0)) * 0.5 + 0.5;
    let col1 = vec3<f32>(0.4, 0.05, 0.5);
    let col2 = vec3<f32>(1.0, 0.4, 0.0);
    let col = mix(col1, col2, t);
    colour = mix(colour,col,0.01);

    colours[global_id.x] = colour;
}

fn curl_noise(p: vec3<f32>) -> vec3<f32> {
    let e = 0.001;

    let dx = vec3<f32>(e, 0.0, 0.0);
    let dy = vec3<f32>(0.0, e, 0.0);
    let dz = vec3<f32>(0.0, 0.0, e);

    let p_x0 = noise3(p - dx);
    let p_x1 = noise3(p + dx);

    let p_y0 = noise3(p - dy);
    let p_y1 = noise3(p + dy);

    let p_z0 = noise3(p - dz);
    let p_z1 = noise3(p + dz);

    let dFdy = (p_y1 - p_y0) / (2.0 * e);
    let dFdz = (p_z1 - p_z0) / (2.0 * e);
    let dFdx = (p_x1 - p_x0) / (2.0 * e);

    return vec3<f32>(
       dFdy.z - dFdz.y,
       dFdz.x - dFdx.z,
       dFdx.y - dFdy.x
    );
}

fn hash33(p: vec3<f32>) -> vec3<f32> {
    let q = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 74.7)),
        dot(p, vec3<f32>(269.5, 183.3, 246.1)),
        dot(p, vec3<f32>(113.5, 271.9, 124.6))
    );
    return fract(sin(q) * 43758.5453);
}

fn noise3(p: vec3<f32>) -> vec3<f32> {
    let i = floor(p);
    let f = fract(p);

    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash33(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash33(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash33(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash33(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash33(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash33(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash33(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash33(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);

    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);

    return mix(nxy0, nxy1, u.z) * 2.0 - 1.0;
}