import sample;

[format("rgba8")]
layout(binding = 0) Texture2D input;

[format("rgba8")]
layout(binding = 2) WTexture2D<float4> output;

struct Petal {
  float3 pos;
  float3 xaxis;
  float3 yaxis;
  uint2 texture_offset;

  private bool is_inside(float3 point, float3 normal, float3 v0, float3 v1) {
    float3 interior_pointing = cross(normal, v1 - v0);
    return dot(point - v0, interior_pointing) >= 0.0;
  }

  
  Optional<RayHitResult> hit(Ray ray) {
    float3 normal = normalize(cross(xaxis, yaxis));
    // Calculate normal component of ray.
    float ray_normal_comp = dot(ray.direction, normal);

    // Normal component of origin.
    float origin_normal_comp = dot(ray.origin, normal);
    float v0_normal_comp = dot(pos, normal);
    float t = (v0_normal_comp - origin_normal_comp) / ray_normal_comp;

    float3 ray_in_plane = ray.origin + t * ray.direction;

    float x_component = dot(ray_in_plane - pos, xaxis) / length(xaxis);
    float y_component = dot(ray_in_plane - pos, yaxis) / length(yaxis);
    float2 uv = float2(x_component, y_component) / float2(length(xaxis), length(yaxis));
    
    // use plane for each triangle edge
    if (
        all(uv >= float2(0.0))
        && all(uv < float2(1.0))
    ) {
        RayHitResult out = { normal, t, uv };
        return Optional<RayHitResult>(out);
    }
    return Optional<RayHitResult>();
  }

  float4 sample_tex(float2 uv, Texture2D input) {
    return bilinearSample(input, float2(texture_offset) + uv * cellSize);
  }
}

uniform uint petalCount;
uniform StructuredBuffer<Petal> petals;
uniform uint2 cellSize;

struct Ray {
  float3 origin;
  float3 direction;
  float2 tRange;
}

struct RayHitResult {
  float3 normal;
  float t;
  float2 uv;
}


[shader("compute")]
[numthreads(16, 16, 1)]
void imagemain(uint2 dispatchThreadID: SV_DispatchThreadID)
{
  uint2 outputDim = getDim(output);
  if (any(dispatchThreadID >= outputDim)) {
  	return;
  }
  output.Store(dispatchThreadID, float4(1.0, 0.0, 1.0, 1.0));


  // for (int i = 0; i < petalCount; i++) {
  //   Petal p = petals[i];
  //   StructuredBuffer<float4> t = input[p.texture];
  //   uint2 bounds = inputDim[p.texture];
  //   if (all(dispatchThreadID < bounds)) {
  //     output.Store(dispatchThreadID, load(t, bounds, dispatchThreadID));
  //   }
  // }
}
