# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests the format of human readable logs.

It checks the response of the API configuration calls and the logs that show
up in the configured logging FIFO.
"""

import datetime
import re
from pathlib import Path

import pytest

# Maps log levels to their numeric values.
LOG_LEVELS = {"ERROR": 0, "WARN": 1, "INFO": 2, "DEBUG": 3, "TRACE": 4}


# pylint: disable=anomalous-backslash-in-string
def check_log_message_format(log_str, instance_id, log_level, show_level, show_origin):
    """Ensure correctness of the logged message.

    Parse the string representing the logs and look for the parts
    that should be there.
    The log line should look like:
    > {year}-{month}-{day}T{hour}:{minute}:{second}.{nanosecond} [{instance id}:{thread name}:{level}:{file}:{line number}] {message}
    where `level`, `file` and `line number` are optional e.g.
    > 2023-07-19T12:10:54.123608814 [fc_api:INFO:src\main.rs:18] yak shaving completed.
    """
    split = iter(log_str.split())
    now = datetime.datetime.now()

    # 2023-07-19T12:10:54.123608814
    timestamp = next(split)
    date, time = timestamp.split("T")
    year, month, day = date.split("-")
    assert len(month) == 2
    assert len(day) == 2

    hour, minute, secs = time.split(":")
    second, nanosecond = secs.split(".")
    assert len(hour) == 2
    assert len(minute) == 2
    assert len(second) == 2
    assert len(nanosecond) == 9

    # Assert the time in the logs is less than or equal to the current time
    log_time = datetime.datetime(
        year=int(year),
        month=int(month),
        day=int(day),
        hour=int(hour),
        minute=int(minute),
        second=int(second),
        microsecond=int(nanosecond) // 1000,
    )
    assert log_time <= now

    # [fc_api:INFO:src\main.rs:18]
    data = next(split)
    data_split = iter(data[1:-1].split(":"))

    log_instance_id = next(data_split)
    assert log_instance_id == instance_id

    # Thread names are not optional.
    _thread_name = next(data_split)

    if show_level:
        level = next(data_split)
        assert level in LOG_LEVELS
        assert LOG_LEVELS[level] <= LOG_LEVELS[log_level.upper()]

    if show_origin:
        _file = next(data_split)
        line = next(data_split)
        assert line.isnumeric()


def test_no_origin_logs(test_microvm_with_api):
    """
    Check that logs do not contain the origin (i.e file and line number).
    """
    _test_log_config(microvm=test_microvm_with_api, show_level=True, show_origin=False)


def test_no_level_logs(test_microvm_with_api):
    """
    Check that logs do not contain the level.
    """
    _test_log_config(microvm=test_microvm_with_api, show_level=False, show_origin=True)


def test_no_nada_logs(test_microvm_with_api):
    """
    Check that logs do not contain either level or origin.
    """
    _test_log_config(microvm=test_microvm_with_api, show_level=False, show_origin=False)


def test_info_logs(test_microvm_with_api):
    """
    Check output of logs when minimum level to be displayed is info.
    """
    _test_log_config(microvm=test_microvm_with_api)


def test_warn_logs(test_microvm_with_api):
    """
    Check output of logs when minimum level to be displayed is warning.
    """
    _test_log_config(microvm=test_microvm_with_api, log_level="Warn")


def test_error_logs(test_microvm_with_api):
    """
    Check output of logs when minimum level of logs displayed is error.
    """
    _test_log_config(microvm=test_microvm_with_api, log_level="Error")


def test_log_config_failure(test_microvm_with_api):
    """
    Check passing invalid FIFOs is detected and reported as an error.
    """
    microvm = test_microvm_with_api
    microvm.spawn(log_file=None)
    microvm.basic_config()

    # only works if log level is Debug
    microvm.time_api_requests = False

    expected_msg = re.escape("No such file or directory (os error 2)")
    with pytest.raises(RuntimeError, match=expected_msg):
        microvm.api.logger.put(
            log_path="invalid log file",
            level="Info",
            show_level=True,
            show_log_origin=True,
        )


def test_api_requests_logs(test_microvm_with_api):
    """
    Test that API requests are logged.
    """
    microvm = test_microvm_with_api
    microvm.spawn(log_file=None)
    microvm.basic_config()

    # Configure logging.
    log_path = Path(microvm.path) / "log"
    log_path.touch()
    microvm.api.logger.put(
        log_path=microvm.create_jailed_resource(log_path),
        level="Info",
        show_level=True,
        show_log_origin=True,
    )
    microvm.log_file = log_path
    # only works if log level is Debug
    microvm.time_api_requests = False

    # Check that a Patch request on /machine-config is logged.
    microvm.api.machine_config.patch(vcpu_count=4)
    # We are not interested in the actual body. Just check that the log
    # message also has the string "body" in it.
    microvm.check_log_message(
        "The API server received a Patch request " 'on "/machine-config" with body'
    )

    # Check that a Put request on /machine-config is logged.
    microvm.api.machine_config.put(vcpu_count=4, mem_size_mib=128)
    microvm.check_log_message(
        "The API server received a Put request " 'on "/machine-config" with body'
    )

    # Check that a Get request on /machine-config is logged without the
    # body.
    microvm.api.machine_config.get()
    microvm.check_log_message(
        "The API server received a Get request " 'on "/machine-config".'
    )

    # Check that all requests on /mmds are logged without the body.
    dummy_json = {"latest": {"meta-data": {"ami-id": "dummy"}}}
    microvm.api.mmds.put(json=dummy_json)
    microvm.check_log_message('The API server received a Put request on "/mmds".')

    microvm.api.mmds.patch(json=dummy_json)
    microvm.check_log_message('The API server received a Patch request on "/mmds".')

    microvm.api.mmds.get()
    microvm.check_log_message('The API server received a Get request on "/mmds".')

    # Check that the fault message return by the client is also logged in the
    # FIFO.
    fault_msg = (
        "The kernel file cannot be opened: No such file or directory (os error 2)"
    )
    with pytest.raises(RuntimeError, match=re.escape(fault_msg)):
        microvm.api.boot.put(kernel_image_path="inexistent_path")
    microvm.check_log_message(
        "Received Error. "
        "Status code: 400 Bad Request. "
        "Message: {}".format(fault_msg)
    )


# pylint: disable=W0102
def _test_log_config(microvm, log_level="Info", show_level=True, show_origin=True):
    """Exercises different scenarios for testing the logging config."""
    microvm.spawn(log_file=None)
    # only works if log level is Debug
    microvm.time_api_requests = False

    # Configure logging.
    log_path = Path(microvm.path) / "log"
    log_path.touch()
    microvm.api.logger.put(
        log_path=microvm.create_jailed_resource(log_path),
        level=log_level,
        show_level=show_level,
        show_log_origin=show_origin,
    )
    microvm.log_file = log_path

    microvm.basic_config()
    microvm.start()

    lines = microvm.log_data.splitlines()
    for line in lines:
        check_log_message_format(line, microvm.id, log_level, show_level, show_origin)
