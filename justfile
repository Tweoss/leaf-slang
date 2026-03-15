run:
    cargo run --release -- data

build-shader:
    slangc src/compute.slang  -target wgsl -o shader_build/compute.wgsl
