#version 460

// Layout
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

// Structs
struct Point {
    vec2 point_pos;
    vec2 point_dir;
    vec4 point_color;
};

// Data
layout(set = 0, binding = 0) buffer PointsData {
    Point p[];
} p;
layout(push_constant) uniform GeneralInfo {
    vec2 resolution;
    //
    float time;
    float delta_time;
    //
    uint points_num;
    float points_speed;
} gen;

// Utilities
float rand(float seed) {
    return fract(sin(seed)*10000.0);
}

// Update functions
// ----------------------------------------------------------------------------------------------------
// Colors
void new_colors(uint idx) {
    p.p[idx].point_color.x = rand((p.p[idx].point_color.x + 4.0) * 4.0);
    p.p[idx].point_color.y = rand((p.p[idx].point_color.y + 4.0) * 4.0);
    p.p[idx].point_color.z = rand((p.p[idx].point_color.z + 4.0) * 4.0);
}
void new_bright_colors(uint idx) {
    p.p[idx].point_color.x = (rand((p.p[idx].point_color.x + 4.0) * 4.0) + 0.5) * 2.0 / 3.0;
    p.p[idx].point_color.y = (rand((p.p[idx].point_color.y + 4.0) * 4.0) + 0.5) * 2.0 / 3.0;
    p.p[idx].point_color.z = (rand((p.p[idx].point_color.z + 4.0) * 4.0) + 0.5) * 2.0 / 3.0;
}
void new_dark_colors(uint idx) {
    p.p[idx].point_color.x = rand((p.p[idx].point_color.x + 4.0) * 4.0) * 2.0 / 3.0;
    p.p[idx].point_color.y = rand((p.p[idx].point_color.y + 4.0) * 4.0) * 2.0 / 3.0;
    p.p[idx].point_color.z = rand((p.p[idx].point_color.z + 4.0) * 4.0) * 2.0 / 3.0;
}
// Position
void contain_pos(uint idx) {
    float width = gen.resolution.x / gen.resolution.y;
    if (p.p[idx].point_pos.x > width) {
        p.p[idx].point_pos.x -= width;
    } else if (p.p[idx].point_pos.x < 0.0) {
        p.p[idx].point_pos.x += width;
    }
    if (p.p[idx].point_pos.y > 1.0) {
        p.p[idx].point_pos.y -= 1.0;
    } else if (p.p[idx].point_pos.y < 0.0) {
        p.p[idx].point_pos.y += 1.0;
    }
}
void update_pos_contained(uint idx) {
    p.p[idx].point_pos.x += gen.delta_time * gen.points_speed * p.p[idx].point_dir.x;
    p.p[idx].point_pos.y += gen.delta_time * gen.points_speed * p.p[idx].point_dir.y;
    contain_pos(idx);
}
// [Direction]
// [   ???   ]

// Entry point
void main() {
    uint idx = gl_GlobalInvocationID.x;

    if (gen.time > 5.0) {
        update_pos_contained(idx);
    }
}