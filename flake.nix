{
  description = "Spin up Linux virtual machines with a single command";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
      cargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.rustPlatform.buildRustPackage {
          pname = "cubic";
          version = cargo.package.version;
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.unions [ ./Cargo.toml ./Cargo.lock ./src ];
          };
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--bin" "cubic" ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postInstall = ''
            wrapProgram $out/bin/cubic --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.qemu ]}
          '';
          meta = {
            description = cargo.package.description;
            homepage = cargo.package.homepage;
            license = with pkgs.lib.licenses; [ mit asl20 ];
            mainProgram = "cubic";
          };
        };
      });
    };
}
