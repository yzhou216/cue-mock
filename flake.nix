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
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{ flake-parts, nixpkgs, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];
      systems = nixpkgs.lib.systems.flakeExposed;
      perSystem =
        {
          config,
          lib,
          pkgs,
          ...
        }:
        let
          inherit (pkgs.stdenvNoCC.hostPlatform) isFreeBSD isLinux;
          cargo = (lib.importTOML ./Cargo.toml).package;

          # Libraries that cinq loads at run time rather than links: those
          # of winit and wgpu for Wayland alone, and libdbus, through which
          # rfd reaches the file chooser portal
          runtimeLibs = lib.optionals isLinux (
            with pkgs;
            [
              dbus
              libGL
              libxkbcommon
              vulkan-loader
              wayland
            ]
          );
        in
        {
          packages.default = config.packages.cinq;

          packages.cinq = pkgs.rustPlatform.buildRustPackage {
            pname = cargo.name;
            inherit (cargo) version;

            src = lib.fileset.toSource {
              root = ./.;
              fileset = lib.fileset.unions [
                ./.cargo
                ./Cargo.toml
                ./Cargo.lock
                ./build.rs
                ./src
                ./man
                ./share
              ];
            };

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              pkg-config
              installShellFiles
              scdoc
              desktop-file-utils
            ];

            # ALSA, which cpal plays sound through on GNU/Linux and FreeBSD
            buildInputs = lib.optionals (isLinux || isFreeBSD) [ pkgs.alsa-lib ];

            # Manual pages dated by the latest commit rather than by the
            # fixed date that builds take otherwise
            env.SOURCE_DATE_EPOCH = toString inputs.self.lastModified;

            # Manual pages, and the desktop entry with its icon and the type of
            # show files, whose entry the build checks against the specification
            postInstall = ''
              installManPage target/man/cinq.1 target/man/cinq.5 target/man/cinq-show.5
              install -Dm644 -t "$out/share/applications" share/applications/cinq.desktop
              install -Dm644 -t "$out/share/icons/hicolor/scalable/apps" target/icons/cinq.svg
              install -Dm644 -t "$out/share/mime/packages" share/mime/packages/cinq.xml
              desktop-file-validate "$out/share/applications/cinq.desktop"
            '';

            postFixup = lib.optionalString isLinux ''
              patchelf --add-rpath ${lib.makeLibraryPath runtimeLibs} "$out/bin/cinq"
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
            ];

            env = lib.optionalAttrs isLinux {
              LD_LIBRARY_PATH = lib.makeLibraryPath runtimeLibs;
            };
          };

          # Formatters of every kind of file in the repository that has one,
          # which nix fmt runs, and which nix flake check holds the files to
          treefmt = {
            projectRootFile = "flake.nix";
            programs = {
              nixfmt.enable = true;
              rumdl-format.enable = true;
              rustfmt = {
                enable = true;
                inherit (cargo) edition;
              };
              taplo.enable = true;
              xmllint.enable = true;
            };
          };
        };
    };
}
