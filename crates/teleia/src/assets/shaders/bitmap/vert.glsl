#version 300 es
precision highp float;

in vec3 vertex;
in vec2 texcoord;
in vec3 color;

uniform mat4 view;
uniform mat4 position;
uniform mat4 projection;

out vec2 vertex_texcoord;
out vec3 vertex_color;

void main() {
    vertex_texcoord = texcoord;
    vertex_color = color;
    vec3 pos = (position * vec4(vertex, 1.0)).xyz;
    gl_Position = projection * view * vec4(pos, 1.0);
}
