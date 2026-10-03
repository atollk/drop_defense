shader_type canvas_item;

varying vec2 world_pos;

uniform vec2 portal_center_pos;
uniform vec2 portal_radii;
uniform bool falling_into_portal = false;

void vertex() {
    world_pos = (MODEL_MATRIX * vec4(VERTEX, 0.0, 1.0)).xy;
}

void fragment() {
    if (falling_into_portal) {
        vec2 pos_local_to_hole = world_pos - portal_center_pos;
        vec2 pos_relative_to_radius = vec2(pos_local_to_hole.x / portal_radii.x, pos_local_to_hole.y / portal_radii.y);
        if (pos_relative_to_radius.y > 1.0) {
            discard;
        }
    }
}

