#version 300 es
precision highp float;

in vec2 vertex;
in vec2 texcoord;

uniform mat4 transform;

out vec2 vertex_texcoord;

void main() {
    vertex_texcoord = texcoord;
    gl_Position = transform * vec4(vertex, 0.0, 1.0);
}
