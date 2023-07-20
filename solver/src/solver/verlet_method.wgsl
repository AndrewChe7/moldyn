struct Particle {
    position: vec4<f64>,
    velocity: vec4<f64>,
    force: vec4<f64>,
    potential_mass_radius_id: vec4<f64>,
    temp: vec4<f64>,
};

struct GlobalUniform {
    boundary_conditions: vec4<f64>,
    dt_count: vec4<f64>,
};

@group(0)
@binding(0)
var<uniform> globals: GlobalUniform;

@group(0)
@binding(1)
var<storage, read> input: array<Particle>;

@group(0)
@binding(2)
var<storage, write> output: array<Particle>;

fn lennard_jones_u_cut (r_cut: f64, sigma: f64, eps: f64) -> f64 {
    let sigma_r = sigma / r_cut;
    let sigma_r_6 = pow(sigma_r, f64(6.0));
    let sigma_r_12 = sigma_r_6 * sigma_r_6;
    return f64(4.0) * eps * (sigma_r_12 - sigma_r_6);
}

fn lennard_jones(r: f64, u_cut: f64, sigma: f64, eps: f64) -> vec2<f64> {
    let sigma_r = sigma / r;
    let sigma_r_6 = pow(sigma_r, f64(6.0));
    let sigma_r_12 = sigma_r_6 * sigma_r_6;
    return vec2<f64> (
        f64(4.0) * eps * (sigma_r_12 - sigma_r_6) - u_cut,
        (f64(24.0) * eps / r) * (sigma_r_6 - f64(2.0) * sigma_r_12),
    );
}

@compute
@workgroup_size(8)
fn step1(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    let particles_count = u32(globals.dt_count.y);
    if (i >= particles_count) {
        return;
    }
    let mass = input[i].potential_mass_radius_id.y;
    let acceleration = input[i].force.xyz / mass;
    let delta_time = globals.dt_count.x;

    var velocity = input[i].velocity.xyz;
    var pos = input[i].position.xyz;
    pos += velocity * delta_time + acceleration * (delta_time * delta_time / f64(2.0));
    pos = pos % globals.boundary_conditions.xyz;
    velocity += acceleration * delta_time / f64(2.0);
    output[i].position = vec4<f64>(pos, f64(1.0));
    output[i].velocity = vec4<f64>(velocity, f64(0.0));
}

@compute
@workgroup_size(8)
fn step2(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let i = global_id.x;
    output[i].position = vec4<f64>(f64(0.0), f64(0.0), f64(0.0), f64(1.0));
    let particles_count = u32(globals.dt_count.y);
    if (i >= particles_count) {
        return;
    }
    let pos1 = input[i].position.xyz;
    let bb = globals.boundary_conditions.xyz;
    let sigma = f64(0.3418);
    let eps = f64(1.712);
    let r_cut = sigma * f64(2.5);
    let u_cut = lennard_jones_u_cut(r_cut, sigma, eps);
    for (var j = 0u; j < particles_count; j++) {
        if (i == j) {
            continue;
        }
        let pos2 = input[j].position.xyz;
        var r = pos2 - pos1;
        if (r.x < -bb.x / f64(2.0)) {
            r.x += bb.x;
        } else if (r.x > bb.x / f64(2.0)) {
            r.x -= bb.x;
        }
        if (r.y < -bb.y / f64(2.0)) {
            r.y += bb.y;
        } else if (r.y > bb.y / f64(2.0)) {
            r.y -= bb.y;
        }
        if (r.z < -bb.z / f64(2.0)) {
            r.z += bb.z;
        } else if (r.z > bb.z / f64(2.0)) {
            r.z -= bb.z;
        }
        let lj = lennard_jones(length(r), u_cut, sigma, eps);
        let potential = lj.x;
        let force = lj.y;
        let force_vec = normalize(r) * force;
        let t = force_vec.x * r.x + force_vec.y * r.y + force_vec.z * r.z;
        let p_force = input[i].force.xyz + force_vec;
        output[i].potential_mass_radius_id.x = input[i].potential_mass_radius_id.x + potential;
        output[i].temp.x = input[i].temp.x - t;
        output[i].force = vec4<f64>(p_force, f64(0.0));
    }
}
