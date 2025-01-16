#version 300 es
precision highp float;

uniform mat4 view;
uniform mat4 position;

out vec2 vertex_texcoord;

void main() {
    const vec2 positions[4] = vec2[](
        vec2(-1, -1),
        vec2(+1, -1),
        vec2(-1, +1),
        vec2(+1, +1)
    );
    const vec2 coords[4] = vec2[](
        vec2(0, 0),
        vec2(1, 0),
        vec2(0, 1),
        vec2(1, 1)
    );
    vec4 vertex = vec4(positions[gl_VertexID], 0.0, 1.0);

    vertex_texcoord = coords[gl_VertexID];
    gl_Position = view * position * vertex;
}
