{ pkgs, ... }:

let 
  dyn_libs = with pkgs; [
    libxkbcommon
    wayland
    mesa
    mesa_glu
    libGL
    libGLU

    # X11 (needed even if you’re using Wayland)
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr
    xorg.libXext
    xorg.libXinerama
  ]; # Required for winit/wgpu windowing
in {
  packages = dyn_libs ++ [ 
    pkgs.rust-analyzer
    pkgs.pkgsCross.mingwW64.stdenv.cc
    pkgs.pkgsCross.mingwW64.windows.mingw_w64_headers
  ];

  languages.rust = {
    enable = true;
    channel = "stable";
    # We need to add the language server manually so we can add it to the path correctly
    components = [ "rustc" "cargo" "clippy" "rustfmt" ];
    targets = [ "x86_64-pc-windows-gnu" ];
    rustflags = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dyn_libs}";
  };

  # Exposes path for nvim lsp
  env.RUST_ANALYZER = "${pkgs.rust-analyzer}/bin/rust-analyzer";
}
