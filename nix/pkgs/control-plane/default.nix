{ stdenv, git, lib, pkgs, allInOne, incremental }:
let
  versionDrv = import ../../lib/version.nix { inherit lib stdenv git; };
  version = builtins.readFile "${versionDrv}";
  project-builder =
    pkgs.callPackage ../control-plane/cargo-project.nix { inherit version allInOne incremental; };
  installer = { name, src, suffix }:
    stdenv.mkDerivation {
      inherit src;
      name = "${name}-${version}";
      binary = "${name}-${suffix}";
      installPhase = ''
        mkdir -p $out/bin
        cp $src/bin/${name} $out/bin/${name}-${suffix}
      '';
    };

  components = { buildType, builder }: rec {
    agents = rec {
      recurseForDerivations = true;
      agents_builder = { buildType, builder }: builder.build { inherit buildType; cargoBuildFlags = [ "-p agents" ]; };
      agent_installer = { name, src }: installer { inherit name src; suffix = "agent"; };
      jsongrpc = agent_installer {
        src = agents_builder { inherit buildType builder; };
        name = "jsongrpc";
      };
      core = agent_installer {
        src = agents_builder { inherit buildType builder; };
        name = "core";
      };
    };

    rest = installer {
      src = builder.build { inherit buildType; cargoBuildFlags = [ "-p rest" ]; };
      name = "rest";
      suffix = "api";
    };

    operators = rec {
      operator_installer = { name, src }: installer { inherit name src; suffix = "operator"; };
      msp = operator_installer {
        src = builder.build { inherit buildType; cargoBuildFlags = [ "-p msp-operator" ]; };
        name = "msp-operator";
      };
      recurseForDerivations = true;
    };

    csi = rec {
      csi_installer = { name, src }: installer { inherit name src; suffix = "csi"; };
      controller = csi_installer {
        src = builder.build { inherit buildType; cargoBuildFlags = [ "-p csi-controller" ]; };
        name = "csi-controller";
      };
      recurseForDerivations = true;
    };
  };
in
{
  LIBCLANG_PATH = project-builder.LIBCLANG_PATH;
  PROTOC = project-builder.PROTOC;
  PROTOC_INCLUDE = project-builder.PROTOC_INCLUDE;
  inherit version;

  release = components { builder = project-builder; buildType = "release"; };
  debug = components { builder = project-builder; buildType = "debug"; };
}
