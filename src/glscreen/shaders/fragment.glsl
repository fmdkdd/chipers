#version 140

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;

void main() {
  color = vec4(texture(tex, v_tex_coords).x * 255, 0.0, 1.0, 1.0);
}
