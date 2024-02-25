#version 460

layout(location = 0) in flat int color;

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


void per_sample() {
    if (gl_SampleID == -1) { // Dummy false check to invoke the shader as per-sample
        discard;
    }
}

void main() {
    per_sample();

    int closest_point_i = 0;
    float min_dist_squared = 100.0;
    float min_dist_diff = 100.0;
    Color = vec4(0.0, 0.0, 0.0, 1.0);

    for (int i=0; i<gen.points_num; i++) {
        float dist_squared = pow(p.p[i].points_pos.x - gl_FragCoord.x / gen.u_resolution.y, 2) + pow(p.p[i].points_pos.y - gl_FragCoord.y / gen.u_resolution.y, 2);
        if (dist_squared < min_dist_squared) {
            min_dist_diff = sqrt(min_dist_squared) - sqrt(dist_squared);
            min_dist_squared = dist_squared;
            closest_point_i = i;
        }
    }

    float dist_squared = pow(p.p[closest_point_i].points_pos.x - gl_FragCoord.x / gen.u_resolution.y, 2) + pow(p.p[closest_point_i].points_pos.y - gl_FragCoord.y / gen.u_resolution.y, 2);
    if (dist_squared < 0.001 / gen.points_num) {
        if (dist_squared < 0.0005 / gen.points_num) {
            Color = vec4(1.0, 1.0, 1.0, 1.0);
        } else {
            Color = vec4(0.0, 0.0, 0.0, 1.0);
        } /*
    } else if (min_dist_diff / dist_squared < 5.0) { // Rounded cells
            Color = vec4(0.0, 0.0, 0.0, 1.0);
    } else if (min_dist_diff / sqrt(dist_squared) < 0.1) { // Contracted surfaces
            Color = vec4(0.0, 0.0, 0.0, 1.0); */
    } else {
        Color = vec4(p.p[closest_point_i].points_color.xyz, 1.0);
    }
}