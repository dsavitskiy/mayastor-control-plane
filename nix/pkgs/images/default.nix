# It would be cool to produce OCI images instead of docker images to
# avoid dependency on docker tool chain. Though the maturity of OCI
# builder in nixpkgs is questionable which is why we postpone this step.

{ busybox, dockerTools, lib, utillinux, control-plane, tini }:
let
  image_suffix = { "release" = ""; "debug" = "-dev"; "coverage" = "-cov"; };
  build-control-plane-image = { buildType, name, package, config ? { } }:
    dockerTools.buildImage {
      tag = control-plane.version;
      created = "now";
      name = "mayadata/mcp-${name}${image_suffix.${buildType}}";
      contents = [ tini busybox package ];
      config = {
        Entrypoint = [ "tini" "--" package.binary ];
      } // config;
    };
  build-agent-image = { buildType, name }:
    build-control-plane-image { inherit buildType name; package = control-plane.${buildType}.agents.${name}; };
  build-rest-image = { buildType }:
    build-control-plane-image {
      inherit buildType;
      name = "rest";
      package = control-plane.${buildType}.rest;
      config = {
        ExposedPorts = {
          "8080/tcp" = { };
          "8081/tcp" = { };
        };
      };
    };
  build-operator-image = { buildType, name }:
    build-control-plane-image {
      inherit buildType;
      name = "${name}-operator";
      package = control-plane.${buildType}.operators.${name};
    };
  build-csi-image = { buildType, name }:
    build-control-plane-image {
      inherit buildType;
      name = "csi-${name}";
      package = control-plane.${buildType}.csi.${name};
    };
in
let
  build-agent-images = { buildType }: {
    core = build-agent-image {
      inherit buildType;
      name = "core";
    };
    jsongrpc = build-agent-image {
      inherit buildType;
      name = "jsongrpc";
    };
  };
  build-operator-images = { buildType }: {
    msp = build-operator-image { inherit buildType; name = "msp"; };
  };
  build-csi-images = { buildType }: {
    controller = build-csi-image { inherit buildType; name = "controller"; };
  };
in
let
  build-images = { buildType }: {
    agents = build-agent-images { inherit buildType; } // {
      recurseForDerivations = true;
    };
    operators = build-operator-images { inherit buildType; } // {
      recurseForDerivations = true;
    };
    csi = build-csi-images { inherit buildType; } // {
      recurseForDerivations = true;
    };
    rest = build-rest-image { inherit buildType; };
  };
in
{
  release = build-images { buildType = "release"; };
  debug = build-images { buildType = "debug"; };
}
