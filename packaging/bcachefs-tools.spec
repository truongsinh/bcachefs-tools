Name:           bcachefs-tools
# define with i.e. --define '_version 1.0'
Version:        %{_version}
Release:        1%{?dist}
Summary:        Userspace tools for bcachefs

License:        GPLv2
URL:            https://github.com/koverstreet/bcachefs-tools

BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  cargo
BuildRequires:  clang-devel
BuildRequires:  keyutils-libs-devel
BuildRequires:  libaio-devel
BuildRequires:  libattr-devel
BuildRequires:  libblkid-devel
BuildRequires:  libsodium-devel
BuildRequires:  libuuid-devel
BuildRequires:  libzstd-devel
BuildRequires:  lz4-devel
BuildRequires:  systemd-devel
BuildRequires:  userspace-rcu-devel
BuildRequires:  zlib-devel

%description
The bcachefs tool, which has a number of subcommands for formatting and managing bcachefs filesystems. Run bcachefs --help for full list of commands.

%prep
%setup -q

%build
%make_build V=0 --no-print-directory

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT%{_sbindir}
mkdir -p $RPM_BUILD_ROOT%{_mandir}/man8
%make_install PREFIX=%{_exec_prefix} ROOT_SBINDIR=%{_sbindir}

# These may be debian-specific, and for unlocking encrypted root fs
rm -f %{buildroot}/%{_datadir}/initramfs-tools/hooks/bcachefs
rm -f %{buildroot}/%{_datadir}/initramfs-tools/scripts/local-premount/bcachefs
# The library is not needed by userspace
rm -f %{buildroot}/usr/lib/libbcachefs.so

%files
%{_sbindir}/mount.bcachefs
%{_sbindir}/bcachefs
%{_sbindir}/fsck.bcachefs
%{_sbindir}/mkfs.bcachefs
%{_mandir}/man8/bcachefs.8.gz

%changelog
* Tue Nov 15 2022 Eric Sandeen <sandeen@sandeen.net> - 2022.11.15-1
- NOTE: This binary RPM has been built directly from the bcachefs-tools
  git tree with "make rpm" from the git hash indicated in the package version.
- Update spec file to allow in-tree rpm builds
- Remove maually added Requires: and unneeded build-requires

* Tue Jan 21 2020 Michael Adams <unquietwiki@gmail.com> - 2020.01.21-1
- Updated RPM package definition to reflect that changes in codebase have occurred.

* Tue Jan 07 2020 Michael Adams <unquietwiki@gmail.com> - 2020.01.07-1
- Initial RPM package definition
- Makefile needs further work to accomodate RPM macros.
