#version 460

// Layout data
layout(location = 0) out vec4 Color;

struct Point {
    vec2 points_pos;
    vec2 points_dir;
    vec4 points_color;
};
layout(set = 0, binding = 0) buffer PointsData {
    Point p[];
} p;

layout(push_constant) uniform GeneralInfo {
    vec2 u_resolution;
    //
    float u_time;
    float u_delta_time;
    //
    uint points_num;
    float points_speed;
} gen;

// Utilities
void per_sample() {
    if (gl_SampleID == -1) { // Dummy false check to invoke the shader as per-sample
        discard;
    }
}

// Distance functions  // return (float(closest_point_i), dist)
vec3 distance_manhattan(float power) {
    int closest_point_i = 0;
    float min_dist = 100.0;
    float min_dist_diff = 100.0;
    Color = vec4(0.0, 0.0, 0.0, 1.0);

    float fragx = gl_FragCoord.x / gen.u_resolution.y;
    float fragy = gl_FragCoord.y / gen.u_resolution.y;
    for (int i=0; i<gen.points_num; i++) {
        float dx = abs(p.p[i].points_pos.x - fragx);
        float dy = abs(p.p[i].points_pos.y - fragy);
        float dist = pow(abs(pow(dx, power)) + abs(pow(dy, power)), 1.0/power);
        if (dist < min_dist) {
            min_dist_diff = min_dist - dist;
            min_dist = dist;
            closest_point_i = i;
        }
    }

    return vec3(closest_point_i, min_dist, min_dist_diff);
}
vec3 distance_euclidean() {
    return distance_manhattan(2.0);
    // int closest_point_i = 0;
    // float min_dist_squared = 100.0;
    // float min_dist_diff = 100.0;
    // Color = vec4(0.0, 0.0, 0.0, 1.0);

    // for (int i=0; i<gen.points_num; i++) {
    //     float dist_squared = pow(p.p[i].points_pos.x - gl_FragCoord.x / gen.u_resolution.y, 2) + pow(p.p[i].points_pos.y - gl_FragCoord.y / gen.u_resolution.y, 2);
    //     if (dist_squared < min_dist_squared) {
    //         min_dist_diff = sqrt(min_dist_squared) - sqrt(dist_squared);
    //         min_dist_squared = dist_squared;
    //         closest_point_i = i;
    //     }
    // }

    // return vec3(closest_point_i, sqrt(min_dist_squared), min_dist_diff);
}

// Coloring functions
// Cell
void color_full(vec3 closest) {
    Color = vec4(p.p[int(closest.x)].points_color.xyz, 1.0);
}

// Border
void color_border_cells(vec3 closest, float cellify_strength) {
    if (closest.z / (closest.y * closest.y) < cellify_strength) { // Rounded cells
        Color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}
void color_border_contracted(vec3 closest) {
    if (closest.z / closest.y < 0.1) { // Contracted surfaces
        Color = vec4(0.0, 0.0, 0.0, 1.0);
    }
}

// Seed
void color_point(vec3 closest) {
    if (closest.y < 0.03 / sqrt(gen.points_num)) {
        if (closest.y < 0.02 / sqrt(gen.points_num)) {
            Color = vec4(1.0, 1.0, 1.0, 1.0);
        } else {
            Color = vec4(0.0, 0.0, 0.0, 1.0);
        }
    }
}


void main() {
    per_sample();

    vec2 st = gl_FragCoord.xy / gen.u_resolution;
    float p = (st.x * st.x + 0.033) * 3.0;
    vec3 closest = distance_manhattan(p);
    color_full(closest);
    p = (st.y * st.y * st.y * st.y) * 5.0;
    color_border_cells(closest, p);
    color_point(closest);
}