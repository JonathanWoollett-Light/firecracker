// Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

const BINARY: &str = env!("CARGO_BIN_EXE_firecracker");

#[test]
fn test_api_socket_in_use() {
    // Create a unix socket with a temporary file.
    let file = utils::tempfile::TempFile::new().unwrap();
    let socket = std::os::unix::net::UnixListener::bind(file.as_path());

    // Start firecracker process pointing to this file as the API socket.
    let socket_path = file.as_path().as_os_str().to_str().unwrap();
    let output = std::process::Command::new(BINARY)
        .args(["--api-sock", socket_path])
        .output()
        .unwrap();

    // Assert the firecracker process exited with expected results.
    assert_eq!(output.status.code().unwrap(), 1);

    let expected_stdout = format!(
        "RunWithApiError error: Failed to open the API socket at: {socket_path}. Check that it is \
         not already used.\n"
    );
    let actual_stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert!(
        actual_stdout.contains(&expected_stdout),
        "\"{actual_stdout}\" does not contain \"{expected_stdout}\""
    );

    let expected_stderr = format!("Error: RunWithApi(FailedToBindSocket(\"{socket_path}\"))");
    let actual_stderr = std::str::from_utf8(&output.stderr).unwrap();
    assert!(
        actual_stderr.contains(&expected_stderr),
        "\"{actual_stderr}\" does not contain \"{expected_stderr}\""
    );

    // This will happen implicitly, but it helps to explicitly ensure and document the unix socket
    // is dropped after the firecracker process.
    drop(socket);
}
