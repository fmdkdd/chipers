// CRT (Tim Lottes) + Phosphor trails (hunterk)
//
// Cause I can't figure how to combine them any other way.

#version 150

in vec2 v_tex_coords;
layout(origin_upper_left) in vec4 gl_FragCoord;
out vec4 fragColor;

uniform vec2 iResolution;
uniform sampler2D tex;
uniform sampler2D prev0_tex;
uniform sampler2D prev1_tex;
uniform sampler2D prev2_tex;
uniform sampler2D prev3_tex;
uniform sampler2D prev4_tex;
uniform sampler2D prev5_tex;
uniform sampler2D prev6_tex;

// This controls how brightly the phosphors glow. The default value is 1, lower
// values reduce the effect.
float response_time = 0.1;

// Emulated input resolution.
#if 0
// Fix resolution to set amount.
vec2 res=vec2(320.0/1.0,160.0/1.0);
#else
// Optimize for resize.
vec2 res=iResolution.xy / 6.0;
#endif

// Hardness of scanline.
//  -8.0 = soft
// -16.0 = medium
float hardScan=-8.0;

// Hardness of pixels in scanline.
// -2.0 = soft
// -4.0 = hard
float hardPix=-3.0;

// Display warp.
// 0.0 = none
// 1.0/8.0 = extreme
vec2 warp=vec2(1.0/40.0,1.0/24.0);

// Amount of shadow mask.
float maskDark=0.5;
float maskLight=1.5;


//------------------------------------------------------------------------
// Phosphor trail pass

mat3 RGB_to_YIQ = mat3(0.299, 0.587, 0.114,
                       0.595716, -0.274453, -0.321263,
                       0.211456, -0.522591, 0.311135);

vec3 fetch(sampler2D text, vec2 pos) {
  return texture2D(text, pos.xy).r * vec3(200, 210, 245);
}

vec3 main_phosphor(vec2 pos) {
  // Sample our textures, with the previous frames in linear color space
  vec3 curr = fetch(tex, pos).rgb;
  vec3 prev0 = pow(fetch(prev0_tex, pos), vec3(2.2));
  vec3 prev1 = pow(fetch(prev1_tex, pos), vec3(2.2));
  vec3 prev2 = pow(fetch(prev2_tex, pos), vec3(2.2));
  vec3 prev3 = pow(fetch(prev3_tex, pos), vec3(2.2));
  vec3 prev4 = pow(fetch(prev4_tex, pos), vec3(2.2));
  vec3 prev5 = pow(fetch(prev5_tex, pos), vec3(2.2));
  vec3 prev6 = pow(fetch(prev6_tex, pos), vec3(2.2));

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

  return curr + trails;
}

//------------------------------------------------------------------------

// sRGB to Linear.
// Assuing using sRGB typed textures this should not be needed.
float ToLinear1(float c){return(c<=0.04045)?c/12.92:pow((c+0.055)/1.055,2.4);}
vec3 ToLinear(vec3 c){return vec3(ToLinear1(c.r),ToLinear1(c.g),ToLinear1(c.b));}

// Linear to sRGB.
// Assuing using sRGB typed textures this should not be needed.
float ToSrgb1(float c){return(c<0.0031308?c*12.92:1.055*pow(c,0.41666)-0.055);}
vec3 ToSrgb(vec3 c){return vec3(ToSrgb1(c.r),ToSrgb1(c.g),ToSrgb1(c.b));}

// Nearest emulated sample given floating point position and texel offset.
// Also zero's off screen.
vec3 Fetch(vec2 pos,vec2 off){
  pos=floor(pos*res+off)/res;
  if(max(abs(pos.x-0.5),abs(pos.y-0.5))>0.5)return vec3(0.0,0.0,0.0);
  return ToLinear(main_phosphor(pos) + vec3(0.022, 0.027, 0.03));}

// Distance in emulated pixels to nearest texel.
vec2 Dist(vec2 pos){pos=pos*res;return -((pos-floor(pos))-vec2(0.5));}

// 1D Gaussian.
float Gaus(float pos,float scale){return exp2(scale*pos*pos);}

// 3-tap Gaussian filter along horz line.
vec3 Horz3(vec2 pos,float off){
  vec3 b=Fetch(pos,vec2(-1.0,off));
  vec3 c=Fetch(pos,vec2( 0.0,off));
  vec3 d=Fetch(pos,vec2( 1.0,off));
  float dst=Dist(pos).x;
  // Convert distance to weight.
  float scale=hardPix;
  float wb=Gaus(dst-1.0,scale);
  float wc=Gaus(dst+0.0,scale);
  float wd=Gaus(dst+1.0,scale);
  // Return filtered sample.
  return (b*wb+c*wc+d*wd)/(wb+wc+wd);}

// 5-tap Gaussian filter along horz line.
vec3 Horz5(vec2 pos,float off){
  vec3 a=Fetch(pos,vec2(-2.0,off));
  vec3 b=Fetch(pos,vec2(-1.0,off));
  vec3 c=Fetch(pos,vec2( 0.0,off));
  vec3 d=Fetch(pos,vec2( 1.0,off));
  vec3 e=Fetch(pos,vec2( 2.0,off));
  float dst=Dist(pos).x;
  // Convert distance to weight.
  float scale=hardPix;
  float wa=Gaus(dst-2.0,scale);
  float wb=Gaus(dst-1.0,scale);
  float wc=Gaus(dst+0.0,scale);
  float wd=Gaus(dst+1.0,scale);
  float we=Gaus(dst+2.0,scale);
  // Return filtered sample.
  return (a*wa+b*wb+c*wc+d*wd+e*we)/(wa+wb+wc+wd+we);}

// Return scanline weight.
float Scan(vec2 pos,float off){
  float dst=Dist(pos).y;
  return Gaus(dst+off,hardScan);}

// Allow nearest three lines to effect pixel.
vec3 Tri(vec2 pos){
  vec3 a=Horz3(pos,-1.0);
  vec3 b=Horz5(pos, 0.0);
  vec3 c=Horz3(pos, 1.0);
  float wa=Scan(pos,-1.0);
  float wb=Scan(pos, 0.0);
  float wc=Scan(pos, 1.0);
  return a*wa+b*wb+c*wc;}

// Distortion of scanlines, and end of screen alpha.
vec2 Warp(vec2 pos){
  pos=pos*2.0-1.0;
  pos*=vec2(1.0+(pos.y*pos.y)*warp.x,1.0+(pos.x*pos.x)*warp.y);
  return pos*0.5+0.5;}

// Shadow mask.
vec3 Mask(vec2 pos){
  pos.x+=pos.y*3.0;
  vec3 mask=vec3(maskDark,maskDark,maskDark);
  pos.x=fract(pos.x/6.0);
  if(pos.x<0.333)mask.r=maskLight;
  else if(pos.x<0.666)mask.g=maskLight;
  else mask.b=maskLight;
  return mask;}

// Draw dividing bars.
float Bar(float pos,float bar){pos-=bar;return pos*pos<4.0?0.0:1.0;}

// Entry.
void main(){
  vec2 pos=Warp(gl_FragCoord.xy/iResolution.xy);
  fragColor.rgb=Tri(pos)*Mask(gl_FragCoord.xy);
  fragColor.a=1.0;
  fragColor.rgb=ToSrgb(fragColor.rgb);
}
