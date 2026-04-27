#version 330 core
layout (location = 0) in vec2 a_position;
uniform float u_aspect;
void main() {
    vec2 pos = a_position;
    if (u_aspect > 1.0) {
        pos.x /= u_aspect;
    } else {
        pos.y *= u_aspect;
    }
    gl_Position = vec4(pos, 0.0, 1.0);
}
