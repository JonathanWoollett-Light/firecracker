#!/bin/bash

# Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# fail if we encounter an error, uninitialized variable or a pipe breaks
set -eu -o pipefail

TOOLS_DIR=$(dirname $0)
source "$TOOLS_DIR/functions"

# Set our TMPDIR inside /srv, so all files created in the session end up in one
# place
say "Create TMPDIR in /srv"
export TMPDIR=/srv/tmp
mkdir -pv $TMPDIR

# Some of the security tests need this (test_jail.py)
# Convert the Docker created cgroup so we can create cgroup children
# From https://github.com/containerd/containerd/issues/6659
say "cgroups v2: enable nesting"
CGROUP=/sys/fs/cgroup
if [ -f $CGROUP/cgroup.controllers -a -e $CGROUP/cgroup.type ]; then
    # move the processes from the root group to the /init group,
    # otherwise writing subtree_control fails with EBUSY.
    # An error during moving non-existent process (i.e., "cat") is ignored.
    mkdir -p $CGROUP/init
    xargs -rn1 < $CGROUP/cgroup.procs > $CGROUP/init/cgroup.procs || :
    # enable controllers
    sed -e 's/ / +/g' -e 's/^/+/' < $CGROUP/cgroup.controllers \
        > $CGROUP/cgroup.subtree_control
fi

say "Copy CI artifacts to /srv, so hardlinks work"
cp -ruvf build/img /srv

cd tests
pytest "$@"
exit $?
