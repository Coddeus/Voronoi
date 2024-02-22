#version 450 core

layout(location=0) out flat int color;

void main() {
    gl_Position = vec4(float(gl_VertexIndex > 0 && gl_VertexIndex < 4) * 2.0 - 1.0, float(gl_VertexIndex % 2) * 2.0 - 1.0, 0.0, 1.0);
    color = int(gl_VertexIndex<3);
}