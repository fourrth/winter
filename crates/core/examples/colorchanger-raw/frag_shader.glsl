#version 320 es
precision mediump float;

out vec4 outputF;
uniform float timeu;

void main()
{{
	outputF = vec4(normalize(vec3(abs(cos(timeu)),{},{})),1.0);
}};