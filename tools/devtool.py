# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

import sys
import os
import argparse
import subprocess

# Firecracker devtool
#
# Use this script to build and test Firecracker.

# TODO: Port over other comments.

# Development container image (without tag)
DEVCTR_IMAGE_NO_TAG = "public.ecr.aws/firecracker/fcuvm"
# Development container tag
DEVCTR_IMAGE_TAG = "v35"
# Development container image (name:tag)
# This should be updated whenever we upgrade the development container.
# (Yet another step on our way to reproducible builds.)
DEVCTR_IMAGE = DEVCTR_IMAGE_NO_TAG + ":" + DEVCTR_IMAGE_TAG

# Parser
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
# ------------------------------------------------------------------------------
# ------------------------------------------------------------------------------
args = parser.parse_args()
print("args:", args)

# Install docker
subprocess.run(["sudo", "apt-get", "install", "docker", "-y"])
# Run docker
subprocess.run(["sudo", "systemctl", "start", "docker"])
# Pull docker image
subprocess.run(["sudo", "docker", "pull", DEVCTR_IMAGE])

# Handle command
if args.command == "build":
    print("building")
    # # Runs build within docker container
    # subprocess.run([
    #     "sudo", "docker", "run", 
    #     "--rm",
    #     "--volume","/dev:/dev",
    #     "--volume",""
    # ])
    
elif args.command == "test":
    print("testing")
elif args.command == "shell":
    print("shell")