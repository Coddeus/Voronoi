#version 460 core

void main() {
    gl_Position = vec4(float(gl_VertexIndex > 0 && gl_VertexIndex < 4) * 2.0 - 1.0, float(gl_VertexIndex % 2) * 2.0 - 1.0, 0.0, 1.0);
}