#version 320 es
precision mediump float;

out vec4 outputF;

in vec4 vertexColor;

uniform float widthu;
uniform float heightu;

uniform float arena_cell_lengthu;

layout(location = 0) uniform float xposu;
layout(location = 1) uniform float yposu;

void main()
{
	float cell_s = floor((xposu/widthu) * arena_cell_lengthu);
	float cell_t = floor((yposu/heightu) * arena_cell_lengthu);

    float pos_cell_s = floor((gl_FragCoord.x/widthu) * arena_cell_lengthu);
    float pos_cell_t = floor((1.0 - gl_FragCoord.y/heightu) * arena_cell_lengthu);

    // remember we are dealing with values
    // well within float's precision range,
    // so it's either 0 or a whole number
    // never going to be 0.0000005 or something

    float diff_s = abs(cell_s - pos_cell_s);
    float diff_t = abs(cell_t - pos_cell_t);

    // again, we will always be whole numbers
    // mediump float looses whole number precision at
    // like 2000 so we are good for this

    // also always 0 or +, never - at this point
    float condition = sign(diff_s + diff_t);
    outputF = vertexColor * condition;

}