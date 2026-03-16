@binding(1) @group(0) var outputImage_0 : texture_storage_2d<rgba8unorm, write>;

@binding(2) @group(0) var<storage, read> quad_points_0 : array<f32>;

@binding(0) @group(0) var inputImage_0 : texture_2d<f32>;

fn bilinearSample_0( input_0 : texture_2d<f32>,  uv_0 : vec2<f32>) -> vec4<f32>
{
    var width_0 : f32;
    var height_0 : f32;
    {var dim = textureDimensions((input_0));((width_0)) = f32(dim.x);((height_0)) = f32(dim.y);};
    var size_0 : vec2<f32> = vec2<f32>(width_0, height_0);
    var coord_0 : vec2<f32> = max(uv_0 * size_0 - vec2<f32>(0.5f), vec2<f32>(vec2<i32>(i32(0))));
    var _S1 : vec2<u32> = vec2<u32>(floor(coord_0));
    var w_0 : vec2<f32> = coord_0 - vec2<f32>(_S1);
    var wi_0 : vec2<f32> = vec2<f32>(1.0f) - w_0;
    var _S2 : vec3<i32> = vec3<i32>(vec3<u32>(_S1, u32(0)));
    var _S3 : vec2<u32> = vec2<u32>(size_0) - vec2<u32>(vec2<i32>(i32(1)));
    var _S4 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(0), u32(1)), _S3), u32(0)));
    var _S5 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(1), u32(0)), _S3), u32(0)));
    var _S6 : vec3<i32> = vec3<i32>(vec3<u32>(min(_S1 + vec2<u32>(u32(1), u32(1)), _S3), u32(0)));
    var _S7 : vec4<f32> = vec4<f32>(w_0.x);
    var _S8 : vec4<f32> = vec4<f32>(wi_0.x);
    return ((textureLoad((input_0), ((_S6)).xy, ((_S6)).z)) * _S7 + (textureLoad((input_0), ((_S4)).xy, ((_S4)).z)) * _S8) * vec4<f32>(w_0.y) + ((textureLoad((input_0), ((_S5)).xy, ((_S5)).z)) * _S7 + (textureLoad((input_0), ((_S2)).xy, ((_S2)).z)) * _S8) * vec4<f32>(wi_0.y);
}

@compute
@workgroup_size(16, 16, 1)
fn imageMain(@builtin(global_invocation_id) dispatchThreadID_0 : vec3<u32>)
{
    var dispatchThreadID_1 : vec2<u32> = dispatchThreadID_0.xy;
    var width_1 : f32;
    var height_1 : f32;
    {var dim = textureDimensions((outputImage_0));((width_1)) = f32(dim.x);((height_1)) = f32(dim.y);};
    var uv_1 : vec2<f32> = vec2<f32>(dispatchThreadID_1.xy) / vec2<f32>(width_1, height_1);
    var sample_uv_3_0 : vec3<f32> = (((vec3<f32>(vec2<f32>(uv_1.x, 1.0f - uv_1.y), 1.0f)) * (mat3x3<f32>(vec3<f32>(quad_points_0[i32(0)], quad_points_0[i32(3)], quad_points_0[i32(6)]), vec3<f32>(quad_points_0[i32(1)], quad_points_0[i32(4)], quad_points_0[i32(7)]), vec3<f32>(quad_points_0[i32(2)], quad_points_0[i32(5)], quad_points_0[i32(8)])))));
    var _S9 : vec2<f32> = sample_uv_3_0.xy / vec2<f32>(sample_uv_3_0.z);
    var sample_uv_0 : vec2<f32> = _S9;
    sample_uv_0[i32(1)] = 1.0f - _S9.y;
    textureStore((outputImage_0), (dispatchThreadID_1), (vec4<f32>(bilinearSample_0(inputImage_0, sample_uv_0).xyz, 1.0f)));
    return;
}

