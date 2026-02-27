// WGSL Shaders for Axiom
// Imposter sphere rendering using ray-sphere intersection

// Vertex shader input
struct VertexInput {
    @location(0) position: vec3<f32>,  // atom position
    @location(1) radius: f32,           // sphere radius
    @location(2) color: vec3<f32>,      // RGB color
}

// Vertex shader output / Fragment shader input
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) sphere_center: vec3<f32>,
    @location(2) radius: f32,
    @location(3) color: vec3<f32>,
}

// Camera uniforms
struct Camera {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

// Vertex shader - Screen-space billboard quad for sphere imposters
@vertex
fn vs_main(
    in: VertexInput,
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // Generate billboard quad corners (screen-space quad)
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );

    let corner = corners[vertex_index % 6u];

    // Transform sphere center to clip space
    let center_clip = camera.view_proj * vec4<f32>(in.position, 1.0);

    // Calculate sphere radius in clip space (perspective-correct scaling)
    // This ensures spheres get smaller with distance
    let radius_clip = in.radius / center_clip.w;  // Divide by w for perspective

    // Offset quad corner in clip space (billboard)
    out.clip_position = center_clip + vec4<f32>(corner.x * radius_clip, corner.y * radius_clip, 0.0, 0.0);

    // Pass world position for ray tracing
    out.world_position = in.position;
    out.sphere_center = in.position;
    out.radius = in.radius;
    out.color = in.color;

    return out;
}

// Fragment shader - Ray-sphere intersection rendering
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate ray from camera through this fragment
    let ray_origin = camera.position;
    let ray_dir = normalize(in.world_position - camera.position);

    // Ray-sphere intersection
    let oc = ray_origin - in.sphere_center;
    let a = dot(ray_dir, ray_dir);
    let b = 2.0 * dot(oc, ray_dir);
    let c = dot(oc, oc) - in.radius * in.radius;
    let discriminant = b * b - 4.0 * a * c;

    // Discard if ray misses sphere
    if (discriminant < 0.0) {
        discard;
    }

    // Calculate intersection point
    let t = (-b - sqrt(discriminant)) / (2.0 * a);
    let hit_pos = ray_origin + t * ray_dir;

    // Calculate surface normal
    let normal = normalize(hit_pos - in.sphere_center);

    // Simple directional lighting
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normal, light_dir), 0.0);
    let ambient = 0.3;
    let lighting = ambient + diffuse * 0.7;

    // Apply lighting to color
    let final_color = in.color * lighting;

    return vec4<f32>(final_color, 1.0);
}
