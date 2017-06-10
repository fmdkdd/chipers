#version 120

varying vec2 v_tex_coords;

uniform sampler2D tex;

void main() {
  gl_FragColor = vec4(texture2D(tex, v_tex_coords).x * 255, 0.0, 1.0, 1.0);
}
