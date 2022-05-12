# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# Firecracker devtool
#
# Use this script to build and test Firecracker.
# TODO: Port over other comments.

# Imports
# ------------------------------------------------------------------------------
# ------------------------------------------------------------------------------
import sys
import os
import argparse
import subprocess
import platform
# Constants
# ------------------------------------------------------------------------------
# ------------------------------------------------------------------------------
# TODO Add reason for every dependency.
x86_dependencies = [
    "binutils-dev"
    # Needed in order to be able to compile `userfaultfd-sys`.
    "clang"
    "cmake"
    "curl"
    "file"
    "g++"
    "gcc"
    "gcc-aarch64-linux-gnu"
    "git"
    "iperf3"
    "iproute2"
    "jq"
    "libdw-dev"
    "libiberty-dev"
    "libssl-dev"
    "libcurl4-openssl-dev"
    "lsof"
    "make"
    "musl-tools"
    "net-tools"
    "openssh-client"
    "pkgconf"
    "python"
    "python3"
    "python3-dev"
    "python3-pip"
    "python3-venv"
    "ruby-dev"
    "zlib1g-dev"
    "screen"
    "tzdata"
    "xz-utils"
    "bc"
    "flex"
    "bison"
]
# TODO Add reason for every dependency.
aarch64_dependencies = [
    "binutils-dev"
    # Needed in order to be able to compile `userfaultfd-sys`.
    "clang"
    "cmake"
    "curl"
    "file"
    "g++"
    "gcc"
    "git"
    "iperf3"
    "iproute2"
    "jq"
    "libbfd-dev"
    "libcurl4-openssl-dev"
    "libdw-dev"
    "libfdt-dev"
    "libiberty-dev"
    "libssl-dev"
    "lsof"
    "make"
    "musl-tools"
    "net-tools"
    "openssh-client"
    "pkgconf"
    "python"
    "python3"
    "python3-dev"
    "python3-pip"
    "python3-venv"
    "zlib1g-dev"
    "screen"
    "tzdata"
    "xz-utils"
    "bc"
    "flex"
    "bison"
]

# Main
# ------------------------------------------------------------------------------
# ------------------------------------------------------------------------------
parser = argparse.ArgumentParser()
parser.add_argument(
    "-y",
    "--unattended",
    help='Run unattended. Assume the user would always answer "yes" to any confirmation prompt.',
    action="store_true",
)
subparsers = parser.add_subparsers(required=True, dest="command")
# test
# ------------------------------------------------------------------------------
test_parser = subparsers.add_parser("test")
test_parser.add_argument(
    "-c",
    "--cpuset-cpus",
    metavar="cpulist",
    help="Set a dedicated cpulist to be used by the tests.",
)
test_parser.add_argument(
    "-m",
    "--cpuset-mems",
    metavar="memlist",
    help="Set a dedicated memlist to be used by the tests.",
)
test_parser.add_argument(
    "-r",
    "--ramdisk",
    metavar="size[k|m|g]",
    help="Use a ramdisk of `size` MB for the entire test session (e.g stored artifacts, Firecracker binaries, logs/metrics FIFOs and test created device files).",
)
test_parser.add_argument(
    "--pytest",
    metavar="args",
    nargs="*",
    help="The Firecracker testing system is based on pytest. Arguments passed through to pytest.",
)
# build
# ------------------------------------------------------------------------------
build_parser = subparsers.add_parser("build")
build_parser.add_argument(
    "--release", help="Build the release binaries.", action="store_true"
)
build_group = build_parser.add_mutually_exclusive_group()
build_group.add_argument(
    "--gnu",
    help="Build with gnu libc instead of musl libc.",
    action="store_true",
)
build_group.add_argument(
    "--ssh",
    nargs=2,
    metavar=("public-key", "private-key"),
    help="Provide the paths to the public and private SSH keys on the host (in this particular order) required for the git authentication.",
)
build_parser.add_argument(
    "--cargo",
    metavar="args",
    nargs="*",
    help="Firecracker is built using the Rust build system. Arguments passed through to cargo.",
)
# shell
# ------------------------------------------------------------------------------
shell_parser = subparsers.add_parser("shell")
# Parse
# ------------------------------------------------------------------------------
args = parser.parse_args()
print("args:", args)

# Handle command
if args.command == "build":
    print("building")
    # Sets build environment variables
    os.environ["profile"] = "release" if args.release else "debug"
    libc = "gnu" if args.gnu else "musl"
    os.environ["libc"] = libc
    # Sets ssh
    [public_key, private_key] = args.ssh
    os.environ["host_pub_key_path"] = public_key
    os.environ["host_priv_key_path"] = private_key
    # Sets toolchain
    machine = platform.machine()
    cpu_architecture = "x86_64" if machine == "AMD64" else machine
    assert (
        cpu_architecture == "x86_64"
        or cpu_architecture == "i686"
        or cpu_architecture == "aarch64"
    )
    toolchain = "{}-unknown-linux-{}".format(cpu_architecture, libc)
    print("toolchain:", toolchain)
    os.environ["target"] = toolchain
    # os.environ["cargo_args"] = args.cargo

    # Gets package manager
    package_manager = None
    try:
        subprocess.run(["apt-get", "--version"])
        package_manager = "apt-get"
    except:
        try:
            subprocess.run(["yum", "--version"])
            package_manager = "yum"
        except:
            raise BaseException(
                "Could not find known package manager (apt-get or yum)."
            )
    print("package_manager:", package_manager)

    # Updates package manager and installs required packages
    subprocess.run([package_manager, "update"])
    if cpu_architecture == "aarch64":
        # Installs all aarch64 dependencies we can from the package manager.
        subprocess.run([package_manager, "-y", "install"] + aarch64_dependencies)
        # TODO Add reasons for "setuptools", "setuptools_rust" and "wheel".
        subprocess.run(
            ["python3", "-m", "pip", "install"]
            + ["setuptools", "setuptools_rust", "wheel"]
        )
        # TODO What does this do? Why do we need it?
        subprocess.run(["rm", "-rf", "/var/lib/apt/lists/*"])
    else:
        # Installs all x86_64 dependencies we can from the package manager.
        subprocess.run([package_manager, "-y", "install"] + x86_dependencies)
        # TODO Add reasons for "setuptools" and "wheel".
        subprocess.run(["python3", "-m", "pip", "install"] + ["setuptools", "wheel"])
        # TODO What does this do? Why do we need it?
        subprocess.run(["gem", "install", "chef-utils:16.6.14", "mdl"])
    # Upgrade pip to newest stable version.
    subprocess.run(["python3", "-m", "pip", "install", "--upgrade pip"])
    # TODO What does this do? Why do we need it?
    subprocess.run(["python3", "-m", "pip", "install", "poetry"])

    # Install Rust
    subprocess.run(["curl", "--proto", "'=https'", "--tlsv1.2", "-sSf", "https://sh.rustup.rs", "|", "sh","-y"])

    # # Runs build within docker container
    # subprocess.run([
    #     "sudo", "docker", "run",
    #     "--rm",
    #     "--volume","/dev:/dev",
    #     "--volume",""
    # ])

elif args.command == "test":
    print("testing")
    assert False, "Unimplemented"
elif args.command == "shell":
    print("shell")
    assert False, "Unimplemented"
