#version 300 es
precision highp float;

in vec2 vertex;
in vec2 texcoord;
in vec3 color;

uniform mat4 transform;

out vec2 vertex_texcoord;
out vec3 vertex_color;

void main() {
    vertex_texcoord = texcoord;
    vertex_color = color;
    gl_Position = transform * vec4(vertex, 0.0, 1.0);
}
