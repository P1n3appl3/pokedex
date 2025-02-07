{
  inputs = {
    utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, utils }: utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
        ];
      };
    }
  );
}

# { lib, rustPlatform, fetchFromGitHub }: rustPlatform.buildRustPackage rec {
#   pname = "dsrom";
#   version = "0.4.2";
#
#   src = fetchFromGitHub {
#     owner = "aetiashax";
#     repo = "ds-rom";
#     rev = "v${version}";
#     hash = "sha256-zu+5YDcKgPfh5UI3KegmpYfMwgxwBLl8bW1fK0n+91s=";
#   };
#
#   cargoHash = "sha256-bLjrVVNuWlMUQJkDcQoWkcXhGzA3VrIzXQeH7EwuUQQ=";
#
#   meta = {
#     description = "Library/CLI for extracting and building DS ROMs";
#     homepage = "https://github.com/aetiashax/ds-rom";
#   };
# }
