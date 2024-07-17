{
  inputs = {
    nixpkgs.url =
      "github:NixOS/nixpkgs/f46237b072307afe5e87a761530aaf67350d54c9";
    flake-parts.url =
      "github:hercules-ci/flake-parts/9126214d0a59633752a136528f5f3b9aa8565b7d";
    rust-overlay.url =
      "github:oxalica/rust-overlay/b970af40fdc4bd80fd764796c5f97c15e2b564eb";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, rust-overlay }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "aarch64-darwin" "x86_64-darwin" "x86_64-linux" ];
      perSystem = { config, self', inputs', pkgs, system, ... }:
        with import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        let
          rust = makeRustPlatform {
            cargo = rust-bin.stable."1.79.0".default;
            rustc = rust-bin.stable."1.79.0".default;
          };
        in {
          packages.default = rust.buildRustPackage {
            name = "valid8";
            version = "v0.0.3";
            src = ./.;
            cargoHash = "";
            buildInputs = [ libiconv pkg-config gcc ]
              ++ lib.optionals stdenv.isDarwin
              (with darwin.apple_sdk.frameworks; [
                System
                Security
                SystemConfiguration
                CoreFoundation
                CoreServices
                Foundation
              ]);
          };
        };

    };
}
