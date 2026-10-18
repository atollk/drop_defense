shader_type canvas_item;
render_mode blend_premul_alpha;

group_uniforms Circle;
uniform vec3 inner_color : source_color;
uniform vec3 rim_color : source_color;
uniform float radius : hint_range(0.0, 0.5);
uniform float rim_width : hint_range(0.0, 0.2);
uniform float softness : hint_range(0.0, 0.1);

group_uniforms Breathing;
uniform float breathing_speed : hint_range(0.0, 5.0, 0.1);
uniform float breathing_strength : hint_range(0.0, 0.2);

group_uniforms Noise;
uniform float noise_angle : hint_range(0, 360);
uniform float noise_speed : hint_range(0, 1);
uniform float noise_strength : hint_range(0.0, 0.1);


vec2 rotate(vec2 v, float angle)
{
    mat2 rotation = mat2(vec2(sin(angle), -cos(angle)), vec2(cos(angle), sin(angle)));
    return v * rotation;
}

void fragment() {
    float dist = distance(UV, vec2(0.5, 0.5));

    // Noise
    vec2 noise_vector = TIME * noise_speed * rotate(vec2(1, 0), radians(noise_angle));
    float noise_offset = noise_strength * texture(TEXTURE, UV + noise_vector).r;
    dist += noise_offset;

    // Breathing
    float breathe_offset = breathing_strength * sin(TIME * breathing_speed);
    dist += breathe_offset;

    // Compute masks
    float inner_radius = radius - rim_width / 2.;
    float outer_radius = radius + rim_width / 2.;

    float inner_mask = smoothstep(inner_radius, inner_radius - softness, dist);
    float outer_mask = smoothstep(outer_radius, outer_radius - softness, dist);
    float rim_mask = outer_mask - inner_mask;

    // Compute color
    COLOR = vec4(vec3(rim_color * rim_mask + inner_color * inner_mask), outer_mask);
}

