// Phosphor trails shader from hunterk:
// http://libretro.com/forums/showthread.php?t=450&p=8698&viewfull=1#post8698
//
// I just ported it from Cg to GLSL.

#version 150

// This controls how brightly the phosphors glow. The default value is 1, lower
// values reduce the effect.
float response_time = 1.0;

mat3 RGB_to_YIQ = mat3(0.299, 0.587, 0.114,
                       0.595716, -0.274453, -0.321263,
                       0.211456, -0.522591, 0.311135);

in vec2 v_tex_coords;
out vec4 color;

uniform sampler2D tex;
uniform sampler2D prev0_tex;
uniform sampler2D prev1_tex;
uniform sampler2D prev2_tex;
uniform sampler2D prev3_tex;
uniform sampler2D prev4_tex;
uniform sampler2D prev5_tex;
uniform sampler2D prev6_tex;

vec3 fetch(sampler2D text) {
  return texture2D(text, v_tex_coords).r * vec3(200, 210, 245);
}

void main() {
  // Sample our textures, with the previous frames in linear color space
  vec3 curr = fetch(tex).rgb;
  vec3 prev0 = pow(fetch(prev0_tex), vec3(2.2));
  vec3 prev1 = pow(fetch(prev1_tex), vec3(2.2));
  vec3 prev2 = pow(fetch(prev2_tex), vec3(2.2));
  vec3 prev3 = pow(fetch(prev3_tex), vec3(2.2));
  vec3 prev4 = pow(fetch(prev4_tex), vec3(2.2));
  vec3 prev5 = pow(fetch(prev5_tex), vec3(2.2));
  vec3 prev6 = pow(fetch(prev6_tex), vec3(2.2));

  // Convert each previous frame to a grayscale image based on luminance value
  // (i.e., convert from RGB to YIQ colorspace, where Y=luma)
  vec3 luma0 = vec3((prev0 * RGB_to_YIQ).r);
  vec3 luma1 = vec3((prev1 * RGB_to_YIQ).r);
  vec3 luma2 = vec3((prev2 * RGB_to_YIQ).r);
  vec3 luma3 = vec3((prev3 * RGB_to_YIQ).r);
  vec3 luma4 = vec3((prev4 * RGB_to_YIQ).r);
  vec3 luma5 = vec3((prev5 * RGB_to_YIQ).r);
  vec3 luma6 = vec3((prev6 * RGB_to_YIQ).r);

  // Add each previous frame's luma value together with a linear decay
  vec3 trails = vec3(0.0, 0.0, 0.0);
  trails += luma0 * (response_time / 100.0);
  trails += luma1 * (response_time / 200.0) / 2.0;
  trails += luma2 * (response_time / 300.0) / 2.0;
  trails += luma3 * (response_time / 400.0) / 2.0;
  trails += luma4 * (response_time / 500.0) / 2.0;
  trails += luma5 * (response_time / 600.0) / 2.0;
  trails += luma6 * (response_time / 700.0) / 2.0;

  // Bring the previous frames back to sRGB color space
  trails = pow(trails, vec3(1.0 / 2.2));

  color = vec4((curr + trails), 1.0);
}
