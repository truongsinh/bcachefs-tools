{ lib
, doCheck ? true
, stdenvNoCC
, callPackage
, nixosTests
, autoPatchelfHook
, binary
, mount
, versionString ? "0.1"
, inShell ? false
, debugMode ? inShell
, testWithValgrind ? true
, fuseSupport ? false
, fuse3 ? null }:

stdenvNoCC.mkDerivation {
  pname = "bcachefs-tools";

  version = "v0.1-flake-${versionString}";

  nativeBuildInputs = [
    binary
    mount
  ];

  buildInputs = mount.propagatedBuildInputs;

  phases = [ "installPhase" ];

  installPhase = ''
    mkdir $out
    mkdir $out/bin
    mkdir $out/lib
    mkdir $out/share
    mkdir $out/etc
    cp -pr "${binary}/bin/"* $out/bin
    cp -pr "${binary}/lib/"* $out/lib
    cp -pr "${binary}/share/"* $out/share
    cp -pr "${binary}/etc/"* $out/etc
    cp -pr "${mount}/bin/"* $out/bin/
    chmod u+w $out/bin/*
    patchelf --add-rpath $out/lib $out/bin/bcachefs-mount
    ln -s "$out/bin/bcachefs-mount" "$out/bin/mount.bcachefs"
    ln -s "$out/bin" "$out/sbin"
  '';
  doCheck = doCheck; # needs bcachefs module loaded on builder

  passthru = {
    tests = {
      smoke-test = nixosTests.bcachefs;
    };
  };

  enableParallelBuilding = true;
  meta = with lib; {
    description = "Userspace tools for bcachefs";
    homepage    = http://bcachefs.org;
    license     = licenses.gpl2;
    platforms   = platforms.linux;
    maintainers =
      [ "Kent Overstreet <kent.overstreet@gmail.com>"
      ];

  };
}
