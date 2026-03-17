run:
    just build-shader
    cargo run --release -- data

build-shader:
    slangc src/compute.slang  -target wgsl -o shader_build/compute.wgsl
    slangc src/wgpu/warp.slang  -target wgsl -o shader_build/warp.wgsl
    slangc src/wgpu/opacity.slang  -target wgsl -o shader_build/opacity.wgsl
    slangc src/wgpu/render.slang  -target wgsl -o shader_build/render.wgsl
    slangc src/wgpu/combine.slang  -target wgsl -o shader_build/combine.wgsl
watch-shader:
    #!/usr/bin/env nu
    watch --debounce-ms 300 src/wgpu/render.slang {try {just build-shader} }
