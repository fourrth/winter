#version 320 es
precision lowp float;

out vec4 outputF;
uniform float timeu;

void main()
{{
	outputF = vec4(normalize(vec3(abs(cos(timeu)),{},{})),1.0);
}};