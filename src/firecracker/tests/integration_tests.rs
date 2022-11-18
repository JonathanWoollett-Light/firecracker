use std::io::BufRead;
use std::process::{Command, Stdio};
use serde_json::json;

// fn sudo<const N: usize>(args: [&str; N]) -> std::io::Result<std::process::Output> {
//     Command::new("sudo").args(args).output()
// }

fn command_output<const N:usize>(args: [&str;N]) -> std::io::Result<std::process::Output> {
    let joined = args.iter().map(|c|format!("{c} ")).collect::<String>();
    print!("`{joined}`: ");
    let output = Command::new(args[0])
        .args(&args[1..])
        .output()?;
    println!("{output:?}");
    Ok(output)
}
fn put(data: serde_json::Value, path: &str) -> std::io::Result<std::process::Output> {
    command_output(["curl","-X","PUT","--data",&data.to_string(),&format!("'http://localhost/{path}'")])
}
fn overwrite(s: &str) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(s)
}

#[cfg(target_arch = "x86_64")]
#[test]
fn test_brand_string() {
    use std::time::Instant;

    const FC_IP: &str = "172.16.0.2";
    const API_SOCKET: &str = "/tmp/test_brand_string.socket";
    const TAP_DEV: &str = "tap0";
    const MASK_SHORT: &str = "/30";
    const ROOT_FS: &str = "ubuntu-18.04.ext4";
    const SSH_KEY: &str = "ubuntu-18.04.id_rsa";
    const KERNEL: &str = "vmlinux.bin";
    const KERNEL_BOOT_ARGS: &str ="console=ttyS0 reboot=k panic=1 ipv6.disable=1 i8042.nokbd 8250.nr_uarts=1 random.trust_cpu=on";
    const CPU_TEMPLATE: Option<&str> = Some("T2S");
    const VCPU_COUNT: u8 = 2;
    const MEM_SIZE_MIB: u16 = 128;
    const LOG_PATH: &str = "/tmp/test_brand_string.log";
    const FC_MAC: &str = "06:00:AC:10:00:02";

    let file = std::fs::File::open("/proc/cpuinfo").unwrap();
    let reader = std::io::BufReader::new(file).lines();
    let cpu_info = reader
        .take(27)
        .map(|a| a.map(|b| format!("{b}\n")))
        .collect::<std::io::Result<String>>()
        .unwrap();
    // println!("cpu_info:\n{}", cpu_info);
    // assert!(false);

    // let data = Command::new("ls").arg("/dev").output().unwrap();
    // dbg!(data);
    let data = Command::new("ls").arg("/dev/kvm").output().unwrap();
    dbg!(data);

    // Set this user's file acess to /dev/kvm to read and write
    // let res = Command::new("setfacl").args(["-m", "u:${USER}:rw", "/dev/kvm"]).output();
    // dbg!(&res);
    // res.unwrap();

    // Remove old API socket if present
    if std::path::Path::new(API_SOCKET).exists() {
        std::fs::remove_file(API_SOCKET).unwrap();
    }

    const STDOUT: &str = "/tmp/test_brand_string.stdout";
    const STDERR: &str = "/tmp/test_brand_string.stderr";

    // Build firecracker
    let _build_output = command_output(["cargo","build"]).unwrap();

    // Run firecracker
    let mut firecracker = Command::new("cargo")
        .args(["run", "--", "--api-sock", API_SOCKET])
        // .stdout(std::process::Stido::piped())
        // .stderr(std::process::Stido::piped())
        .stdout(overwrite(STDOUT).unwrap())
        .stderr(overwrite(STDERR).unwrap())
        .spawn()
        .unwrap();

    // Wait for process to start
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Setup networking (this is `if false` so you can collapse it)
    if false {
        println!("Setting networking");
        // Set up network interface
        {
            Command::new("ip")
                .args(["link", "del", TAP_DEV, "2>", "||", "true"])
                .output()
                .unwrap();
            Command::new("ip")
                .args(["tuntap", "add", "dev", TAP_DEV, "mode", "tap"])
                .output()
                .unwrap();
            Command::new("ip")
                .args([
                    "addr",
                    "add",
                    &format!("{TAP_DEV},{MASK_SHORT}"),
                    "dev",
                    TAP_DEV,
                ])
                .output()
                .unwrap();
            Command::new("ip")
                .args(["link", "set", "dev", TAP_DEV, "up"])
                .output()
                .unwrap();
        }

        // Setup ip forwarding
        {
            Command::new("sh")
                .args(["-c", "\"echo 1 > /proc/sys/net/ipv4/ipforward\""])
                .output()
                .unwrap();
            Command::new("iptables")
                .args([
                    "-t",
                    "nat",
                    "-D",
                    "POSTROUTING",
                    "-o",
                    "eth0",
                    "-j",
                    "MASQUERADE",
                    "||",
                    "true",
                ])
                .output()
                .unwrap();
            Command::new("iptables")
                .args([
                    "-I",
                    "FORWARD",
                    "1",
                    "-m",
                    "conntrack",
                    "--ctstate",
                    "RELATED,ESTABLISHED",
                    "-j",
                    "ACCEPT",
                ])
                .output()
                .unwrap();
            Command::new("iptables")
                .args([
                    "-D", "FORWARD", "-i", TAP_DEV, "-o", "eth0", "-j", "ACCEPT", "||", "true",
                ])
                .output()
                .unwrap();
        }

        // Insert rules to enable internet access for microVM
        {
            Command::new("iptables")
                .args([
                    "-t",
                    "nat",
                    "-A",
                    "POSTROUTING",
                    "-o",
                    "eth0",
                    "-j",
                    "MASQUERADE",
                ])
                .output()
                .unwrap();
            Command::new("iptables")
                .args([
                    "-I",
                    "FORWARD",
                    "1",
                    "-m",
                    "conntrack",
                    "--ctstate",
                    "RELATED,ESTABLISHED",
                    "-j",
                    "ACCEPT",
                ])
                .output()
                .unwrap();
            Command::new("iptables")
                .args([
                    "-I", "FORWARD", "1", "-i", "tap0", "-o", "eth0", "-j", "ACCEPT",
                ])
                .output()
                .unwrap();
        }
        println!("Setup networking");
    };

    println!("Awaiting API server socket creation");

    // Wait for API server to have started
    while !std::path::Path::new(API_SOCKET).exists() {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    println!("Awaiting API server socket created");

    let root_fs_path = format!("/tmp/{ROOT_FS}");
    let ssh_key_path = format!("/tmp/{SSH_KEY}");
    let kernel_path = format!("/tmp/{KERNEL}");

    // Download artefacts
    {
        // Download kernel
        command_output(["wget","-O",&kernel_path,"https://s3.amazonaws.com/spec.ccfc.min/img/quickstart_guide/x86_64/kernels/vmlinux.bin"]).unwrap();
        // Download root fs
        command_output(["wget","-O",&root_fs_path,"https://s3.console.aws.amazon.com/s3/object/spec.ccfc.min?region=us-east-1&prefix=ci-artifacts/disks/x86_64/ubuntu-18.04.ext4"]).unwrap();
        // Download root fs ssh key
        command_output(["wget","-O",&ssh_key_path,"https://s3.console.aws.amazon.com/s3/object/spec.ccfc.min?region=us-east-1&prefix=ci-artifacts/disks/x86_64/ubuntu-18.04.id_rsa"]).unwrap();
    }

    // Sets read permissions for user and group on key
    command_output(["chmod","400",&ssh_key_path]).unwrap();

    // Configure machine
    {
        // Set machine config
        {
            let mut data = json!({
                "vcpu_count": VCPU_COUNT,
                "mem_size_mib": MEM_SIZE_MIB
            });
            if let Some(template) = CPU_TEMPLATE {
                data.as_object_mut().unwrap().insert(String::from("cpu_template"), serde_json::Value::String(String::from(template)));
            }
            put(data, "machine-config").unwrap();
        }

        // Set boot source
        {
            let data = json!({
                "kernel_image_path": kernel_path,
                "boot_args": KERNEL_BOOT_ARGS
            });
            put(data, "boot-source").unwrap();
        }

        // Set rootfs
        {
            let data = json!({
                "drive_id": "rootfs",
                "path_on_host": root_fs_path,
                "is_root_device": true,
                "is_read_only": false
            });
            put(data, "drive/rootfs").unwrap();
        }

        // Set logger
        {
            let data = json!({
                "log-path": LOG_PATH,
                "level": "Debug",
                "show_level": true,
                "show_log_origin": true
            });
            put(data, "logger").unwrap();
        }

        // Set network interface
        {
            let data = json!({
                "iface_id": "net1", 
                "guest_mac": FC_MAC, 
                "host_dev_name": TAP_DEV 
            });
            put(data, "network-interfaces/net1").unwrap();
        }

        // mmds
        {
            let data = json!({
                "version": "V1",
                "network_interfaces": [ "net0" ]
            });
            put(data, "mmds/config").unwrap();
        }
        // mmds
        {
            let data = json!({
                "latest": { 
                    "meta-data": { 
                        "ami-id": "ami-87654321", 
                        "reservation_id": "r-79054aef" 
                    } 
                }
            });
            put(data, "mmds").unwrap();
        }
    }

    std::thread::sleep(std::time::Duration::from_millis(15));

    // Start microVM
    {
        let data = json!({ "action_type": "InstanceStart" });
        put(data, "actions").unwrap();
    }

    // Get cpu info over ssh
    command_output(["ssh","-i",&ssh_key_path,&format!("root@{FC_IP}"),r#""cat /proc/cpuinfo\""#]).unwrap();

    // It will wait for over a minute then the connection will time out
    // Specificlaly `ssh: connect to host 172.16.0.2 port 22: Connection timed out`

    std::thread::sleep(std::time::Duration::from_secs(10));

    // Kill firecracker process.
    firecracker.kill().unwrap();

    assert!(false);
}

#[test]
fn test_blahblah() {
    let file = std::fs::File::open("/proc/cpuinfo").unwrap();
    let reader = std::io::BufReader::new(file).lines();
    let cpu_info = reader
        .take(27)
        .map(|a| a.map(|b| format!("{b}\n")))
        .collect::<std::io::Result<String>>()
        .unwrap();
}
