#!/usr/bin/python3
#
# Spits out a .deb file while avoiding all the Debian Project rigmarole.
#
import gzip
import hashlib
import io
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile


# configure me
PACKAGE_NAME = "ripcalc"
FILES = {
    "target/release/ripcalc": "usr/bin/ripcalc",
}
STRIP_TARGET_FILES = {
    "usr/bin/ripcalc",
}
ARCH_SRC_FILE = "target/release/ripcalc"
DEPS = []
BIN_DEP_TARGET_FILES = {
    "usr/bin/ripcalc",
}
AUTHOR = "Ond\u0159ej Ho\u0161ek <ondra.hosek@gmail.com>"
AUTHOR_YEARS = "2021"
SECTION = "net"
PRIORITY = "optional"
HOMEPAGE = "https://gitlab.tuwien.ac.at/ondrej.hosek/ripcalc"
LICENSE = "CC0-1.0"
SHORT_DESCR = "IPv4 and IPv6 subnet calculator"
LONG_DESCR = """It calculates subnets."""


# constants
READELF_MACHINE_RE = re.compile("(?m)^\\s+Machine:\\s+(\\S.*)$")
VERSION_LINE_RE = re.compile("^\\s+(?:0+|0x[0-9a-f]+):\\s+Name: ([A-Z]+)_([0-9.]+)\\s+Flags: .+\\s+Version: .+$")


def run_cmd(cmd_args, work_dir=None):
    subproc = subprocess.Popen(
        cmd_args,
        stdout=subprocess.PIPE,
        cwd=work_dir,
    )
    (stdout, _stderr) = subproc.communicate()
    result = subproc.wait()
    if result != 0:
        raise ValueError("{c} returned exit code {e}".format(c=cmd_args, e=result))
    return stdout.decode()


def get_elf_machine(file_name):
    with open(file_name, "rb") as f:
        magic = f.read(4)
        if magic != b"\x7FELF":
            raise ValueError("ELF magic does not match")

        bitness = f.read(1)
        bits = 32
        if bitness == b"\x02":
            bits = 64
        else:
            raise ValueError("invalid bitness: {b}".format(b=repr(bitness)))

        endianness = f.read(1)

        machine_struct = "<H"
        if endianness == b"\x02":
            machine_struct = ">H"
        elif endianness != b"\x01":
            raise ValueError("invalid endianness value: {ev}".format(ev=repr(endianness)))

        version = f.read(1)
        if version != b"\x01":
            raise ValueError("unsupported ELF version: {v}".format(v=repr(version)))

        _abi = f.read(1)
        _abi_ver = f.read(1)
        _pad = f.read(7)
        _file_type = f.read(2)
        machine = f.read(2)

        # https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#File_header
        # https://www.debian.org/ports/
        (machine_num,) = struct.unpack(machine_struct, machine)
        return {
            0x28: "armel",
            0x3E: "amd64",
            0xB7: "arm64",
            0x03: "i386",
            0x08: "mipsel" if bits == 32 else "mips64el",
            0x15: "ppc64el",
            0x16: "s390x",
        }[machine_num]


def get_elf_versions(file_name, lib_to_ver):
    elf_output = run_cmd(["readelf", "-V", file_name])
    elf_lines = elf_output.strip().split("\n")

    correct_section = False
    for ln in elf_lines:
        if not ln.startswith(" "):
            # section header
            correct_section = ln.startswith("Version needs section '.gnu.version_r' contains ")
            continue

        if not correct_section:
            # skip this section fully
            continue

        # content line
        m = VERSION_LINE_RE.match(ln)
        if m is None:
            continue

        library = m.group(1)
        ver_str = m.group(2)
        ver = tuple(int(piece) for piece in ver_str.split("."))

        exist_ver = lib_to_ver.get(library, None)
        if exist_ver is None or exist_ver < ver:
            lib_to_ver[library] = ver


def libver_to_deps(lib_to_ver):
    deb_mapping = {
        "GLIBC": "libc6",
        "GCC": "libgcc1",
    }
    return [
        "{n} (>= {v})".format(
            n=deb_mapping[lib],
            v=".".join(str(v) for v in ver)
        )
        for (lib, ver) in lib_to_ver.items()
    ]


def get_mail_date_time():
    # does not work with older Python versions:
    #return datetime.datetime.now().astimezone().strftime("%a, %d %b %Y %H:%M:%S %z")
    return run_cmd(["date", "+%a, %d %b %Y %H:%M:%S %z"])


def compress_gzip_without_timestamp(bytes_to_compress):
    # older versions of Python do not have the mtime argument on gzip.compress()
    gzip_buffer = io.BytesIO()
    gzip_file = gzip.GzipFile(fileobj=gzip_buffer, mode="w", mtime=0)
    gzip_file.write(bytes_to_compress)
    gzip_file.close()
    return gzip_buffer.getvalue()


def fake_changelog(code_revision):
    changelog_fmt = """
{pn} ({cr}) unstable; urgency=medium

  * Just read the git log, no need to maintain a second changelog.

 -- {au}  {dt}
"""
    changelog_str = changelog_fmt.lstrip().format(
        pn=PACKAGE_NAME,
        cr=code_revision,
        au=AUTHOR,
        dt=get_mail_date_time(),
    )
    changelog_bytes = changelog_str.encode("utf-8")
    changelog_gz_bytes = compress_gzip_without_timestamp(changelog_bytes)
    return changelog_gz_bytes


def make_copyright():
    copyright_fmt = """
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: {pn}
Upstream-Contact: {au}
Source: {hp}

Files: *
Copyright: {ay} {au}
License: {li}
"""
    copyright_str = copyright_fmt.lstrip().format(
        pn=PACKAGE_NAME,
        au=AUTHOR,
        hp=HOMEPAGE,
        ay=AUTHOR_YEARS,
        li=LICENSE,
    )
    copyright_bytes = copyright_str.encode("utf-8")
    return copyright_bytes


def collect_data(temp_dir, code_dir, generated_files):
    # directory to assemble data files
    data_dir = os.path.join(temp_dir, "data")
    os.mkdir(data_dir)

    # copy over the files
    source_and_target_paths = sorted(
        FILES.items(),
        key=lambda st: (st[1], st[0]),
    )
    for source_rel_path, target_rel_path in source_and_target_paths:
        source_path = os.path.join(code_dir, source_rel_path)
        target_path = os.path.join(data_dir, target_rel_path)

        # ensure the subdirectory exists
        target_dir = os.path.dirname(target_path)
        os.makedirs(target_dir, exist_ok=True)

        # copy over!
        shutil.copy(source_path, target_path)

        if target_rel_path in STRIP_TARGET_FILES:
            # also strip the file
            run_cmd(["strip", target_path])

    # also the autogenerated files
    for target_rel_path, bs in generated_files.items():
        target_path = os.path.join(data_dir, target_rel_path)

        target_dir = os.path.dirname(target_path)
        os.makedirs(target_dir, exist_ok=True)

        with open(target_path, "wb") as f:
            f.write(bs)

        if target_rel_path in STRIP_TARGET_FILES:
            # also strip the file
            run_cmd(["strip", target_path])

    # tar up the data archive
    run_cmd(
        [
            "tar",
            "-cJ",
            "-f", "../data.tar.xz",
            "--owner=root:0",
            "--group=root:0",
            ".",
        ],
        work_dir=data_dir,
    )

    # return the data directory path for further processing
    return data_dir


def collect_control(temp_dir, data_dir, generated_files, code_revision, deb_arch):
    # calculate the total size and MD5 checksum of all the files
    target_rel_path_to_md5 = {}
    total_size_bytes = 0
    for target_rel_path in FILES.values():
        target_path = os.path.join(data_dir, target_rel_path)
        total_size_bytes += os.stat(target_path).st_size

        md5 = hashlib.md5()
        with open(target_path, "rb") as f:
            while True:
                bs = f.read(1024)
                if not bs:
                    break
                md5.update(bs)
        target_rel_path_to_md5[target_rel_path] = md5.hexdigest()

    for target_rel_path, bs in generated_files.items():
        total_size_bytes += len(bs)
        target_rel_path_to_md5[target_rel_path] = hashlib.md5(bs).hexdigest()

    total_size_kib = total_size_bytes // 1024

    # directory to assemble control files
    control_dir = os.path.join(temp_dir, "control")
    os.mkdir(control_dir)

    # collect shared lib dependency info
    lib_to_ver = {}
    for target_rel_path in BIN_DEP_TARGET_FILES:
        get_elf_versions(os.path.join(data_dir, target_rel_path), lib_to_ver)
    deb_deps = libver_to_deps(lib_to_ver)

    # add the static dependency info
    deb_deps.extend(DEPS)

    # the control file
    long_descr_lines = LONG_DESCR.strip().split("\n")
    long_descr_lines_fmt = [
        " ." if ln == "" else " {ln}".format(ln=ln)
        for ln in long_descr_lines
    ]
    long_descr_fmt = "\n".join(long_descr_lines_fmt)
    control_file_tpl = """
Package: {pn}
Source: {pn}
Version: {cr}
Architecture: {da}
Maintainer: {au}
Installed-Size: {tsk}
Depends: {d}
Section: {se}
Priority: {pr}
Homepage: {hp}
Description: {sd}
{ldf}
"""
    control_file_contents = control_file_tpl.lstrip().format(
        pn=PACKAGE_NAME,
        cr=code_revision,
        da=deb_arch,
        tsk=total_size_kib,
        d=", ".join(sorted(deb_deps)),
        au=AUTHOR,
        se=SECTION,
        pr=PRIORITY,
        hp=HOMEPAGE,
        sd=SHORT_DESCR,
        ldf=long_descr_fmt,
    )

    with open(os.path.join(temp_dir, "control", "control"), "wb") as f:
        f.write(control_file_contents.encode("utf-8"))

    # the md5sums file
    with open(os.path.join(temp_dir, "control", "md5sums"), "w", encoding="utf-8") as f:
        target_paths_and_md5s = sorted(
            target_rel_path_to_md5.items(),
            key=lambda tm: (tm[1], tm[0]),
        )
        for target_path, md5 in target_paths_and_md5s:
            f.write("{m}  {p}\n".format(m=md5, p=target_path))

    # tar up the control archive
    run_cmd(
        [
            "tar",
            "-cz",
            "-f", "../control.tar.gz",
            "--owner=root:0",
            "--group=root:0",
            ".",
        ],
        work_dir=control_dir,
    )


def main():
    # find our path
    script_path = os.path.realpath(sys.argv[0])
    script_dir = os.path.dirname(script_path)
    code_dir = os.path.dirname(script_dir)

    # what version do we have?
    code_revision = run_cmd(["git", "rev-list", "--count", "HEAD"], code_dir).rstrip("\n")

    # fake a changelog, make a copyright
    changelog_gz_bytes = fake_changelog(code_revision)
    copyright_bytes = make_copyright()

    # list the autogenerated files
    docdir = "usr/share/doc/{pn}".format(pn=PACKAGE_NAME)
    generated_files = {
        "{dd}/changelog.gz".format(dd=docdir): changelog_gz_bytes,
        "{dd}/copyright".format(dd=docdir): copyright_bytes,
    }

    # find out what architecture we have
    if ARCH_SRC_FILE is None:
        deb_arch = "all"
    else:
        deb_arch = get_elf_machine(os.path.join(code_dir, ARCH_SRC_FILE))

    # prepare a directory in which to assemble everything
    with tempfile.TemporaryDirectory() as temp_dir:
        # debian-binary
        with open(os.path.join(temp_dir, "debian-binary"), "wb") as f:
            f.write(b"2.0\n")

        # collect the data
        data_dir = collect_data(temp_dir, code_dir, generated_files)

        # collect the control files
        collect_control(temp_dir, data_dir, generated_files, code_revision, deb_arch)

        deb_name = "{pn}_{cr}_{da}.deb".format(
            pn=PACKAGE_NAME,
            cr=code_revision,
            da=deb_arch,
        )
        deb_path = os.path.join(temp_dir, deb_name)

        # ar the whole thing
        run_cmd(
            [
                "ar",
                "rcD",
                deb_name,
                "debian-binary",
                "control.tar.gz",
                "data.tar.xz",
            ],
            work_dir=temp_dir,
        )

        # copy the archive out
        deb_target_name = os.path.join(code_dir, deb_name)
        shutil.copy(deb_path, deb_target_name)

        # print out the name
        print(deb_name)


if __name__ == "__main__":
    main()
