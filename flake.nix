# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  description = "CINQ Is Not QLab, a minimal cue-based show controller";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          inherit (pkgs.stdenv.hostPlatform) isLinux;
          cargo = (lib.importTOML ./Cargo.toml).package;

          # Libraries that winit and wgpu open at run time rather than link
          runtimeLibs = lib.optionals isLinux (
            with pkgs;
            [
              libGL
              libxkbcommon
              vulkan-loader
              wayland
              libx11
              libxcursor
              libxi
              libxrandr
            ]
          );

          # Sounds of the sample show.  Git LFS stores them, so the flake
          # source holds only their pointers, and they come from the master
          # branch instead, whose history is squashed rather than kept.  The
          # hash pins them: update it whenever the sounds change.
          sounds = pkgs.fetchgit {
            url = "https://github.com/yzhou216/cinq.git";
            rev = "refs/heads/master";
            rootDir = "sounds";
            sparseCheckout = [ "/sounds/*.ogg" ];
            fetchLFS = true;
            hash = "sha256-/6pfy8SAfuOcDmjTWGn4myztbi4AcyUuK/ZLUdqNsl8=";
          };
        in
        {
          packages.default = config.packages.cinq;

          packages.cinq = pkgs.rustPlatform.buildRustPackage {
            pname = cargo.name;
            inherit (cargo) version;

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./Cargo.toml
                ./Cargo.lock
                ./src
                ./man
              ];
            };

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.installShellFiles
            ];
            buildInputs = lib.optionals isLinux [ pkgs.alsa-lib ];

            # Directory that the sample show finds its sounds in
            env.CINQ_DATA_DIR = "${placeholder "out"}/share/cinq";

            postInstall = ''
              mkdir -p $out/share/cinq
              ln -s ${sounds} $out/share/cinq/sounds
              installManPage man/cinq.5
            '';

            postFixup = lib.optionalString isLinux ''
              patchelf --add-rpath ${lib.makeLibraryPath runtimeLibs} $out/bin/cinq
            '';

            meta = {
              inherit (cargo) description;
              license = lib.licenses.gpl3Plus;
              mainProgram = "cinq";
            };
          };

          devShells.default = pkgs.mkShell {
            inputsFrom = [ config.packages.cinq ];

            packages = with pkgs; [
              clippy
              rustfmt
              rust-analyzer
              ffmpeg-headless
              git-lfs
            ];

            env = lib.optionalAttrs isLinux {
              LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
            };
          };

          formatter = pkgs.nixfmt;
        };
    };
}
