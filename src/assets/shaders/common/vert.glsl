#version 300 es
precision highp float;

in vec3 vertex;
in vec3 normal;
in vec2 texcoord;

uniform mat4 view;
uniform mat4 position;
uniform mat4 projection;
uniform mat4 normal_matrix;
uniform mat4 lightspace_matrix;
uniform vec3 camera_pos;

out vec2 vertex_texcoord;
out vec3 vertex_normal;
out vec3 vertex_fragpos;
out vec4 vertex_fragpos_shadow_dir;
out vec3 vertex_view_vector;

void default_main()
{
    vertex_texcoord = texcoord;
    vertex_normal = (normal_matrix * vec4(normal, 1.0)).xyz;
    vec3 pos = (position * vec4(vertex, 1.0)).xyz;
    vertex_fragpos = pos;
    vertex_fragpos_shadow_dir = lightspace_matrix * vec4(pos, 1.0);
    vertex_view_vector = camera_pos - pos;
    gl_Position = projection * view * vec4(pos, 1.0);
}
