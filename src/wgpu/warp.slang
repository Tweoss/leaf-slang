import sample;

typealias vec2 = float2;
typealias vec3 = float3;
typealias vec4 = float4;

[format("rgba8")]
layout(binding = 0) Texture2D<float4> inputImage;

[format("rgba8")]
layout(binding = 1) WTexture2D<float4> outputImage;

// Length 9
layout(binding = 2) StructuredBuffer<float> quad_points;

[shader("compute")]
[numthreads(16, 16, 1)]
void imageMain(uint2 dispatchThreadID: SV_DispatchThreadID)
{

	float width;
	float height;
	outputImage.GetDimensions(width, height);

	vec2 uv = dispatchThreadID.xy / vec2(width, height);
	// Flip output image.
	// vec2 img_uv = vec2(uv.x,  uv.y);
	vec2 img_uv = vec2(uv.x, 1.0 - uv.y);

    // quad_points in column order
    float3x3 quadTransform = float3x3(
		float3(quad_points[0], quad_points[3], quad_points[6]),
		float3(quad_points[1], quad_points[4], quad_points[7]),
		float3(quad_points[2], quad_points[5], quad_points[8]),
    );
    float3 sample_uv_3 = mul(quadTransform, float3(img_uv, 1));
    float2 sample_uv = sample_uv_3.xy / sample_uv_3.z;
    // Flip input image to target the right corners. 
    sample_uv.y = 1.0 - sample_uv.y;
	
    float input_width;
    float input_height;
    inputImage.GetDimensions(input_width, input_height);
    float2 input_size = float2(input_width, input_height);

	outputImage.Store(dispatchThreadID, float4(bilinearSample(inputImage, sample_uv).rgb, 1.0));
	
	return;
}
