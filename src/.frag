#version 450

layout(location = 0) in flat int color;

layout(location = 0) out vec4 Color;

#define POINTS_NUM 20

struct Point {
    vec4 points_pos;
    vec4 points_color;
};
layout(set = 0, binding = 0) buffer PointsData {
    Point p[POINTS_NUM];
} p;

layout(push_constant) uniform GeneralInfo {
    vec2 u_resolution;
    float u_time;
} gen;

void main() {
    if (gl_SampleID == -1) { // Dummy false check to invoke the shader as per-sample
        discard;
    }

    int closest_point_i = 0;
    float min_dist_squared = 100.0;
    Color = vec4(0.0, 0.0, 0.0, 1.0);

    for (int i=0; i<POINTS_NUM; i++) {
        float dist_squared = pow(p.p[i].points_pos.x - gl_FragCoord.x / gen.u_resolution.y, 2) + pow(p.p[i].points_pos.y - gl_FragCoord.y / gen.u_resolution.y, 2);
        if (dist_squared < min_dist_squared) {
            min_dist_squared = dist_squared;
            closest_point_i = i;
        }
    }

    float dist_squared = pow(p.p[closest_point_i].points_pos.x - gl_FragCoord.x / gen.u_resolution.y, 2) + pow(p.p[closest_point_i].points_pos.y - gl_FragCoord.y / gen.u_resolution.y, 2);
    if (dist_squared < 0.001 / POINTS_NUM) {
        Color = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        Color = vec4(p.p[closest_point_i].points_color.xyz, 1.0);
    }
}