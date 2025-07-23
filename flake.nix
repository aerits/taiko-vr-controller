{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk, fenix }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
        rustToolchain = with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "rustc"
              "cargo"
              "rustfmt"
              "clippy"
              "rust-src"
            ])

            targets.wasm32-unknown-unknown.stable.rust-std
            targets.aarch64-linux-android.stable.rust-std
            targets.x86_64-pc-windows-gnu.stable.rust-std
          ];
          nbi = with pkgs; [
            squashfsTools
            clang
            lldb
            libllvm
            lld

            # the entire rust toolchain
            rustToolchain

            pkg-config
            openssl

            # Common cargo tools we often use
            cargo-deny
            cargo-expand
            cargo-binutils

            # cmake for openxr
            cmake
          ];
          # runtime dependencies
          bi =
            [
              pkgs.zstd
              pkgs.libxml2
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
              # bevy dependencies
              udev
              alsa-lib
              # vulkan
              vulkan-loader
              vulkan-headers
              vulkan-tools
              vulkan-validation-layers
              # x11
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
              # wayland
              libxkbcommon
              wayland
              # xr
              openxr-loader
              libGL
            ])
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Cocoa
              # # This is missing on mac m1 nix, for some reason.
              # # see https://stackoverflow.com/a/69732679
              pkgs.libiconv
            ];
      in
      {
        defaultPackage = naersk-lib.buildPackage {
          src = ./.;
          nativeBuildInputs = nbi;
          buildInputs = bi;
        };
        # yoined devshell code from bevy_oxr
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = nbi;
          # build dependencies
          buildInputs = bi;


          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          # this is most likely not needed. for some reason shadows flicker without it.
          AMD_VULKAN_ICD = "RADV";
        };
      }
    );
}
